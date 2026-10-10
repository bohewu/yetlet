#!/usr/bin/env python3
"""YT-064: bounded SHA-256 manifest for an explicit, flat local directory."""
import argparse
from contextlib import contextmanager
import hashlib
import json
import os
from pathlib import Path
import re
import stat
import sys

MANIFEST = "RELEASE-CHECKSUMS.json"
NOTICE = "unsigned local verification; not trusted provenance"
MAX_FILES = 128
MAX_FILE_BYTES = 64 * 1024 * 1024
MAX_TOTAL_BYTES = 256 * 1024 * 1024
MAX_MANIFEST_BYTES = 64 * 1024
NAME = re.compile(r"[A-Za-z0-9][A-Za-z0-9._-]{0,127}\Z")
SHA256 = re.compile(r"[0-9a-f]{64}\Z")


def require(condition, message):
    if not condition:
        raise ValueError(message)


def safe_name(name):
    require(isinstance(name, str) and NAME.fullmatch(name) is not None,
            "unsafe artifact name")
    require(name != MANIFEST, "manifest cannot list itself")


@contextmanager
def directory_fd(directory):
    # Walk every ancestor without resolving/following links. The directory is
    # explicitly supplied; absolute paths are allowed here, never in entries.
    require(os.name == "posix" and hasattr(os, "O_NOFOLLOW"),
            "requires POSIX no-follow directory support")
    require(directory and ".." not in Path(directory).parts,
            "unsafe directory path")
    absolute = Path(os.getcwd()) / directory
    flags = os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW
    fd = os.open(absolute.anchor, flags)
    try:
        for part in absolute.parts[1:]:
            next_fd = os.open(part, flags, dir_fd=fd)
            os.close(fd)
            fd = next_fd
        yield fd
    finally:
        os.close(fd)


def read_file(fd, name, limit):
    before = os.stat(name, dir_fd=fd, follow_symlinks=False)
    require(stat.S_ISREG(before.st_mode), "nonregular file rejected")
    require(before.st_size <= limit, "file size limit exceeded")
    handle = os.open(name, os.O_RDONLY | os.O_NOFOLLOW | os.O_NONBLOCK,
                     dir_fd=fd)
    return handle, before


def fingerprint(info):
    return (info.st_dev, info.st_ino, info.st_size,
            info.st_mtime_ns, info.st_ctime_ns)


def artifacts(fd):
    names = []
    with os.scandir(fd) as entries:
        for entry in entries:
            if entry.name == MANIFEST:
                continue
            safe_name(entry.name)
            names.append(entry.name)
            require(len(names) <= MAX_FILES, "artifact count limit exceeded")
    require(names, "nonempty artifact input required")
    result = []
    total = 0
    for name in sorted(names):
        handle, before = read_file(fd, name, MAX_FILE_BYTES)
        with os.fdopen(handle, "rb") as source:
            require(fingerprint(os.fstat(source.fileno())) == fingerprint(before),
                    "file changed while opening")
            total += before.st_size
            require(total <= MAX_TOTAL_BYTES, "total size limit exceeded")
            digest = hashlib.sha256()
            size = 0
            while chunk := source.read(64 * 1024):
                size += len(chunk)
                require(size <= MAX_FILE_BYTES, "file size limit exceeded")
                digest.update(chunk)
            require(size == before.st_size and
                    fingerprint(os.fstat(source.fileno())) == fingerprint(before),
                    "file changed while hashing")
        result.append({"path": name, "size": size, "sha256": digest.hexdigest()})
    return result


def unique_object(pairs):
    result = {}
    for key, value in pairs:
        require(key not in result, "duplicate JSON key")
        result[key] = value
    return result


def load_manifest(fd):
    handle, before = read_file(fd, MANIFEST, MAX_MANIFEST_BYTES)
    with os.fdopen(handle, "rb") as source:
        require(fingerprint(os.fstat(source.fileno())) == fingerprint(before),
                "manifest changed while opening")
        raw = source.read(MAX_MANIFEST_BYTES + 1)
        require(len(raw) <= MAX_MANIFEST_BYTES, "manifest size limit exceeded")
        require(fingerprint(os.fstat(source.fileno())) == fingerprint(before),
                "manifest changed while reading")
    data = json.loads(raw, object_pairs_hook=unique_object)
    require(isinstance(data, dict) and set(data) == {"schema_version", "notice", "artifacts"},
            "invalid manifest schema")
    require(type(data["schema_version"]) is int and data["schema_version"] == 1 and
            data["notice"] == NOTICE, "unsupported manifest identity")
    entries = data["artifacts"]
    require(isinstance(entries, list) and 1 <= len(entries) <= MAX_FILES,
            "invalid manifest artifact count")
    names = set()
    total = 0
    for entry in entries:
        require(isinstance(entry, dict) and set(entry) == {"path", "size", "sha256"},
                "invalid manifest entry")
        safe_name(entry["path"])
        require(entry["path"] not in names, "duplicate artifact entry")
        names.add(entry["path"])
        require(type(entry["size"]) is int and 0 <= entry["size"] <= MAX_FILE_BYTES,
                "invalid artifact size")
        total += entry["size"]
        require(total <= MAX_TOTAL_BYTES, "total size limit exceeded")
        require(isinstance(entry["sha256"], str) and SHA256.fullmatch(entry["sha256"]),
                "invalid SHA-256")
    return entries


def run(command, directory):
    with directory_fd(directory) as fd:
        if command == "verify":
            expected = load_manifest(fd)
            actual = artifacts(fd)
            require(sorted(expected, key=lambda entry: entry["path"]) == actual,
                    "artifact set, size or SHA-256 mismatch")
        else:
            try:
                os.stat(MANIFEST, dir_fd=fd, follow_symlinks=False)
            except FileNotFoundError:
                pass
            else:
                raise ValueError("manifest already exists; refusing overwrite")
            data = {"schema_version": 1, "notice": NOTICE, "artifacts": artifacts(fd)}
            content = (json.dumps(data, sort_keys=True, indent=2) + "\n").encode("ascii")
            require(len(content) <= MAX_MANIFEST_BYTES, "manifest size limit exceeded")
            handle = os.open(MANIFEST, os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW,
                             0o600, dir_fd=fd)
            try:
                with os.fdopen(handle, "wb") as destination:
                    destination.write(content)
            except BaseException:
                os.unlink(MANIFEST, dir_fd=fd)
                raise


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("command", choices=("create", "verify"))
    parser.add_argument("directory", help="explicit flat local artifact directory")
    args = parser.parse_args(argv)
    try:
        run(args.command, args.directory)
    except (OSError, ValueError, RecursionError):
        # Never disclose supplied paths, manifest entries, filesystem content,
        # or exception traces on failure.
        print("release checksum preflight failed", file=sys.stderr)
        return 1
    print(f"release checksum {args.command}: PASS ({NOTICE})")
    return 0


if __name__ == "__main__":
    sys.exit(main())
