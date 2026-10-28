#!/usr/bin/env python3
"""Manual public-recipient patch preparation and disposable LOCAL custody drill.

prepare binds the three operational files (four sites) to an exact Git commit.
It emits only public material and never applies a patch. drill applies prepared
bytes only to new committed-source snapshots outside the source checkout.
"""
import argparse
import difflib
import hashlib
import io
import json
import os
from pathlib import Path
import platform
import re
import shutil
import stat
import subprocess
import sys
import tarfile

PATHS = (".github/release-evidence-recipient.txt", "xtask/src/release/custody.rs",
         ".github/workflows/release.yml")
PINS = {"age": "eb7dd1b518f0a307c99cd97782623c5321da049154b04acd2d98d21aa7bc9b2c",
        "age-keygen": "0a0009db842259d6717f7eeb30acb6b90d2a2eb924c6acd0a0db0ca1f1537899"}
PUBLIC = rb"age1[0-9a-z]{58}\n"
SHA_ANCHOR = rb'pub const RECIPIENT_SHA: &str = "([0-9a-f]{64})";'
AGE_ANCHOR = rb'"(age1[0-9a-z]{58})"\.into\(\),'
WORKFLOW_ANCHOR = (rb"printf '%s  %s\\n' ([0-9a-f]{64}) \\\n"
                   rb"            \.github/release-evidence-recipient\.txt \| sha256sum --check --strict")


def require(condition, message):
    if not condition:
        raise ValueError(message)


def sha(data):
    return hashlib.sha256(data).hexdigest()


def file_bytes(path):
    require(stat.S_ISREG(path.lstat().st_mode), "regular non-symlink file required")
    return path.read_bytes()


def write(path, data):
    with path.open("xb") as stream:
        os.chmod(path, 0o600)
        stream.write(data)


def json_bytes(value):
    return (json.dumps(value, indent=2) + "\n").encode()


def git(source, *args):
    result = subprocess.run(["git", "-C", str(source), *args], capture_output=True)
    require(result.returncode == 0, "read-only Git source binding failed")
    return result.stdout


def bound_files(source, base):
    require(re.fullmatch(r"[0-9a-f]{40}", base), "exact 40-digit commit required")
    require(git(source, "rev-parse", "HEAD").decode().strip() == base,
            "source HEAD differs from expected base")
    require(git(source, "rev-parse", "--show-toplevel").decode().strip() == str(source),
            "source-tree must be the repository root")
    result = {}
    for name in PATHS:
        data = file_bytes(source / name)
        require(data == git(source, "show", f"{base}:{name}"),
                "operational file differs from expected committed source")
        result[name] = data
    return result


def replacement(files, public):
    require(re.fullmatch(PUBLIC, public), "one public age recipient line required")
    old = files[PATHS[0]]
    require(re.fullmatch(PUBLIC, old), "old public recipient line rejected")
    old_sha = sha(old).encode()
    custody, workflow = files[PATHS[1]], files[PATHS[2]]
    for pattern, data, expected in ((SHA_ANCHOR, custody, old_sha),
                                    (AGE_ANCHOR, custody, old.rstrip(b"\n")),
                                    (WORKFLOW_ANCHOR, workflow, old_sha)):
        matches = list(re.finditer(pattern, data))
        require(len(matches) == 1 and matches[0].group(1) == expected,
                "missing, duplicated or inconsistent operational anchor")
    require(custody.count(old.rstrip(b"\n")) == 1 and custody.count(old_sha) == 1
            and workflow.count(old_sha) == 1, "duplicated operational binding")
    require(public != old, "replacement recipient must differ")
    updated = dict(files)
    updated[PATHS[0]] = public
    updated[PATHS[1]] = custody.replace(old_sha, sha(public).encode()).replace(
        old.rstrip(b"\n"), public.rstrip(b"\n"))
    updated[PATHS[2]] = workflow.replace(old_sha, sha(public).encode())
    return updated


def prepare(source, base, recipient, output):
    require(not output.exists() and not output.is_symlink(), "output already exists")
    files = bound_files(source, base)
    public = file_bytes(recipient)
    updated = replacement(files, public)
    patch = "".join("".join(difflib.unified_diff(
        files[name].decode().splitlines(True), updated[name].decode().splitlines(True),
        fromfile="a/" + name, tofile="b/" + name)) for name in PATHS).encode()
    manifest = {"schema": 1, "base": base, "source_tree": str(source),
                "input_sha256": {n: sha(b) for n, b in files.items()},
                "old_recipient": files[PATHS[0]].decode().strip(),
                "old_recipient_sha256": sha(files[PATHS[0]]),
                "new_recipient": public.decode().strip(), "new_recipient_sha256": sha(public),
                "new_file_sha256": {n: sha(b) for n, b in updated.items()},
                "patch_sha256": sha(patch),
                "retention_checklist": [
                    "Keep the old helper binary and exact public file/pin with original bindings.",
                    "Keep old ciphertext, transport receipt and readback receipt.",
                    "Retain the old identity privately while old readback is needed.",
                    "A new-pin verifier rejects old transport even with the old identity.",
                    "Review this public patch; historical proof fingerprints stay unchanged.",
                    "This preparation does not rotate an installed secret or authorize publication."],
                "applied": False, "public_shape_only": True}
    # Recheck before publication; a failure never leaves a partial preparation.
    require(bound_files(source, base) == files, "source changed during preparation")
    output.mkdir(mode=0o700)
    try:
        write(output / "rotation.patch", patch)
        write(output / "manifest.json", json_bytes(manifest))
    except BaseException:
        shutil.rmtree(output)
        raise
    return manifest, updated


def drill(args):
    source, root = args.source_tree.resolve(), args.work_dir.absolute()
    require(platform.system() == "Linux" and platform.machine() == "x86_64",
            "drill supports Linux amd64 only")
    require(not root.resolve().is_relative_to(source), "private drill must be outside checkout")
    before = bound_files(source, args.base)
    authentication = file_bytes(args.authentication_evidence)
    record = json.loads(authentication)
    require(record.get("proof_pass") is True, "authenticated bootstrap PASS required")
    tools = {"age": args.age.resolve(), "age-keygen": args.age_keygen.resolve()}
    for name, path in tools.items():
        require(sha(file_bytes(path)) == PINS[name]
                and record["authenticated_tool_hashes"]["age/" + name] == PINS[name],
                "authenticated age binary hash mismatch")
    root.mkdir(mode=0o700)  # Exclusive: never reuse any evidence or disposable keys.
    old_umask = os.umask(0o077)
    proof = {"schema": 1, "status": "RUNNING", "context": "LOCAL synthetic rotation/loss drill",
             "complete_product_custody": False, "base": args.base,
             "fixture_root": str(root), "tool_sha256": PINS,
             "authentication_evidence": str(args.authentication_evidence.resolve()),
             "authentication_evidence_sha256": sha(authentication),
             "commands": [], "cases": {}, "source_input_sha256": {n: sha(b) for n, b in before.items()}}

    def run(label, command, *, success=True, cwd=None, stdout=None):
        command = list(map(str, command))
        safe = command.copy()
        for flag in ("--identity",):
            if flag in safe:
                safe[safe.index(flag) + 1] = "<LOCAL_TEST_IDENTITY>"
        # All diagnostics are retained privately, never embedded in the safe proof.
        result = subprocess.run(command, cwd=cwd, stdout=stdout or subprocess.PIPE,
                                stderr=subprocess.PIPE)
        write(root / (label + ".stderr"), result.stderr)
        if stdout is None:
            write(root / (label + ".stdout"), result.stdout)
        proof["commands"].append({"label": label, "command": safe, "cwd": str(cwd) if cwd else None,
                                   "exit": result.returncode})
        write_proof()
        require((result.returncode == 0) == success, "unexpected command status: " + label)
        return result

    def write_proof():
        # Only this task's fresh proof is updated; failed commands remain in it.
        (root / "proof.json").write_bytes(json_bytes(proof))

    try:
        for name, path in tools.items():
            run(name + "-version", [path, "--version"])
        archive = git(source, "archive", "--format=tar", args.base)
        write(root / "source.tar", archive)
        proof["source_archive_sha256"] = sha(archive)
        proof["source_archive_command"] = ["git", "-C", str(source), "archive", "--format=tar", args.base]
        snapshots = {}
        for name in ("A", "B"):
            keys = root / (name + "-keys")
            keys.mkdir(mode=0o700)
            identity = keys / "active.txt"
            with identity.open("xb") as key_output:
                run(name + "-keygen", [tools["age-keygen"]], stdout=key_output)
            public = run(name + "-public", [tools["age-keygen"], "-y", identity]).stdout
            public_file = root / (name + "-public.txt")
            write(public_file, public)
            # A separate LOCAL TEST copy is used only for the explicit loss exercise.
            if name == "B":
                backup = root / "B-recovery"
                backup.mkdir(mode=0o700)
                shutil.copyfile(identity, backup / "held-copy.txt")
            prep = root / (name + "-preparation")
            manifest, updated = prepare(source, args.base, public_file, prep)
            proof.setdefault("preparations", {})[name] = manifest
            snapshot = root / (name + "-source")
            snapshot.mkdir(mode=0o700)
            with tarfile.open(fileobj=io.BytesIO(archive)) as tar:
                require(all(m.isfile() or m.isdir() or m.issym() for m in tar.getmembers()),
                        "source archive has unsupported members")
                tar.extractall(snapshot, filter="data")
            # Apply only to the unpublished isolated snapshot, rechecking the full input binding.
            require({n: sha(file_bytes(snapshot / n)) for n in PATHS} == manifest["input_sha256"],
                    "isolated source differs from preparation")
            for path, data in updated.items():
                (snapshot / path).write_bytes(data)
            require({n: sha(file_bytes(snapshot / n)) for n in PATHS} == manifest["new_file_sha256"],
                    "isolated prepared bytes mismatch")
            target = root / "build-cache"
            run(name + "-build", ["cargo", "build", "--offline", "--locked", "--manifest-path",
                                  snapshot / "xtask/Cargo.toml", "--target-dir", target], cwd=snapshot)
            helper = root / (name + "-yetlet-xtask")
            shutil.copyfile(target / "debug/yetlet-xtask", helper)
            os.chmod(helper, 0o700)
            snapshots[name] = (helper, snapshot / PATHS[0], identity)
            proof.setdefault("helper_sha256", {})[name] = sha(file_bytes(helper))
        private, extras = root / "private", root / "extras"
        private.mkdir(mode=0o700)
        extras.mkdir(mode=0o700)
        write(private / "LOCAL-FIXTURE.txt", b"LOCAL SYNTHETIC ROTATION FIXTURE\n")
        write(private / "BUILD-RECORD.json", json_bytes({"context": "LOCAL synthetic", "complete_product_custody": False}))
        write(extras / "openssl-build.log", b"LOCAL synthetic; no OpenSSL build performed\n")
        write(extras / "openssl-build-result.json", json_bytes({"context": "LOCAL synthetic", "native_exit_code": None}))
        helper_a = snapshots["A"][0]
        run("manifest-create", [helper_a, "release", "checksums", "create", private])
        manifest_bytes = file_bytes(private / "RELEASE-CHECKSUMS.json")
        write(root / "original-RELEASE-CHECKSUMS.json", manifest_bytes)
        original = {p.name: file_bytes(p) for p in private.iterdir()}
        proof["original_manifest_sha256"] = sha(manifest_bytes)
        proof["synthetic_binding"] = {"source": args.base, "workflow_sha": args.base,
            "workflow": "bohewu/yetlet/.github/workflows/release.yml@refs/tags/v0.9.0-alpha.1",
            "git_ref": "refs/tags/v0.9.0-alpha.1", "run": "1", "attempt": "1",
            "notice": "LOCAL fixture values only; no GitHub run or release candidate claim"}

        def custody(label, who, operation, transport, identity=None, success=True):
            helper, recipient, default_identity = snapshots[who]
            output = root / label
            command = [helper, "release", "custody", operation, "--age", tools["age"],
                       "--recipient", recipient, "--source", args.base, "--run", "1", "--attempt", "1"]
            command += (["--private", private, "--extras", extras, "--exit", "0"]
                        if operation == "capture" else
                        ["--transport", transport, "--identity", identity or default_identity])
            run(label, command + ["--output", output], success=success)
            if not success:
                require(not output.exists(), "rejected readback left partial output")
                proof["cases"][label] = {"rejected": True, "partial_output": False}
            elif operation == "verify":
                restored = {p.name: file_bytes(p) for p in (output / "private").iterdir()}
                require(restored == original, "restored full LOCAL manifest/member bytes differ")
                run(label + "-manifest", [helper, "release", "checksums", "verify", output / "private"])
                for p in extras.iterdir():
                    require(file_bytes(output / "extras" / p.name) == file_bytes(p), "restored extra differs")
                receipt = output / "local-readback-receipt.json"
                proof["cases"][label] = {"full_manifest_and_members_byte_identical": True,
                    "receipt": json.loads(file_bytes(receipt)), "receipt_sha256": sha(file_bytes(receipt))}
            else:
                receipt = output / "transport-receipt.json"
                proof["cases"][label] = {"receipt": json.loads(file_bytes(receipt)),
                    "receipt_sha256": sha(file_bytes(receipt)),
                    "ciphertext_sha256": sha(file_bytes(output / "private-evidence.tar.age"))}
            write_proof()
            return output

        a = custody("A-capture", "A", "capture", None)
        custody("A-readback", "A", "verify", a)
        b = custody("B-capture", "B", "capture", None)
        custody("B-readback", "B", "verify", b)
        custody("wrong-key", "B", "verify", b, snapshots["A"][2], success=False)
        custody("new-pin-old-authentic-transport", "B", "verify", a, snapshots["A"][2], success=False)
        custody("retained-A-readback", "A", "verify", a)
        rollback = custody("rollback-A-capture", "A", "capture", None)
        custody("rollback-A-readback", "A", "verify", rollback)
        snapshots["B"][2].unlink()  # Only the newly created LOCAL TEST active copy.
        custody("lost-test-key", "B", "verify", b, success=False)
        shutil.copyfile(root / "B-recovery/held-copy.txt", snapshots["B"][2])
        custody("recovered-test-key", "B", "verify", b)
        require(bound_files(source, args.base) == before, "checkout operational files changed")
        # Source/tool binaries need executable modes; all key/plaintext/evidence files are0600.
        bad = []
        source_links = []
        for path in root.rglob("*"):
            if path.is_symlink():
                require(any(path.is_relative_to(root / (name + "-source"))
                            and path.resolve().is_relative_to(root / (name + "-source"))
                            for name in ("A", "B")), "unsafe source symlink")
                source_links.append(str(path.relative_to(root)))
                continue
            if path.is_dir():
                os.chmod(path, 0o700)
            elif path.is_file():
                os.chmod(path, 0o700 if path in [v[0] for v in snapshots.values()] else 0o600)
            else:
                bad.append(str(path))
        require(not bad, "unexpected nonregular private resource")
        proof["private_modes"] = {"directories": "0700", "files": "0600", "retained_helpers": "0700",
                                   "nonregular_resources": bad, "safe_source_symlinks": source_links}
        proof["checkout_operational_files_unchanged"] = True
        proof["cleanup"] = {"all_subprocesses_awaited": True, "servers_watchers_started": 0,
                            "retained_private_fixture": True}
        proof["status"] = "PASS"
        write_proof()
    except BaseException as error:
        proof["status"] = "FAILED"
        proof["failure"] = str(error)
        write_proof()
        raise
    finally:
        os.umask(old_umask)
    return root / "proof.json"


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest="command", required=True)
    for name in ("prepare", "drill"):
        command = commands.add_parser(name, allow_abbrev=False)
        command.add_argument("--source-tree", type=Path, required=True)
        command.add_argument("--base", required=True, help="exact expected Git HEAD commit")
        if name == "prepare":
            command.add_argument("--recipient-public", type=Path, required=True)
            command.add_argument("--output", type=Path, required=True)
        else:
            for option in ("age", "age-keygen", "authentication-evidence", "work-dir"):
                command.add_argument("--" + option, type=Path, required=True)
    args = parser.parse_args()
    if args.command == "prepare":
        prepare(args.source_tree.resolve(), args.base, args.recipient_public, args.output)
        print("Public preparation created; patch not applied")
    else:
        print("LOCAL drill proof:", drill(args))


if __name__ == "__main__":
    try:
        main()
    except (ValueError, OSError, KeyError, tarfile.TarError) as error:
        print("Rotation refused:", str(error), file=sys.stderr)
        sys.exit(1)
