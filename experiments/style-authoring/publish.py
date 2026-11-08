"""YT-238 developer-only Python stdlib publisher; no public API or CLI integration.

Trusted single owner, stable filesystem, Linux only. Generator stdout is staged
privately; this tool does not sandbox Rust or supervise arbitrary process trees.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import stat
import subprocess
import sys
import tempfile

LIMIT = 2 * 1024 * 1024
OWNER = "yt238-style-authoring-v1"


class PublicationError(Exception):
    pass


def digest(data):
    return hashlib.sha256(data).hexdigest()


def protected(root, relative):
    if not root.is_absolute() or root.resolve() != root:
        raise PublicationError("app root must be absolute and unredirected")
    current = Path(root.anchor)
    for part in (*root.parts[1:], *relative.parts):
        current /= part
        if current.is_symlink():
            raise PublicationError(f"symlink component: {current}")
        if current.exists() and current != root / relative and not current.is_dir():
            raise PublicationError(f"non-directory ancestor: {current}")
    if current.exists() and not stat.S_ISREG(current.stat().st_mode):
        raise PublicationError(f"nonregular destination: {current}")
    return current


def locations(root, output):
    if sys.platform != "linux":
        raise PublicationError("atomic replace observed only on Linux; Windows deferred")
    if not re.fullmatch(r"assets/generated/[A-Za-z0-9_-]+\.generated\.css", output):
        raise PublicationError("expected dedicated app-relative assets/generated/*.generated.css")
    target = protected(root, Path(output))
    record = protected(root, Path(".stylegen/publication.json"))
    pending = protected(root, Path(".stylegen/pending.json"))
    return target, record, pending


def read_record(record, output):
    if not record.exists():
        return None
    try:
        data = json.loads(record.read_text(encoding="utf-8"))
        if set(data) != {"owner", "output", "sha256"} or data["owner"] != OWNER or data["output"] != output or not re.fullmatch("[0-9a-f]{64}", data["sha256"]):
            raise ValueError("record identity")
        return data
    except (ValueError, TypeError) as error:
        raise PublicationError("invalid/unowned private record") from error


def atomic_write(path, data):
    path.parent.mkdir(parents=True, exist_ok=True)
    name = None
    try:
        with tempfile.NamedTemporaryFile(dir=path.parent, prefix=".yt238-", delete=False) as temp:
            name = temp.name
            temp.write(data)
            temp.flush()
            os.fsync(temp.fileno())
        os.replace(name, path)
    finally:
        if name is not None and os.path.exists(name):
            os.unlink(name)


def encoded(data):
    return (json.dumps(data, sort_keys=True) + "\n").encode("utf-8")


def publish(root, output, css):
    target, record, pending = locations(root, output)
    if pending.exists():
        raise PublicationError("unaccepted partial publication/intent; explicit reconciliation required")
    if not css or len(css) > LIMIT:
        raise PublicationError("CSS must be nonempty and at most 2 MiB")
    try:
        css.decode("utf-8")
    except UnicodeDecodeError as error:
        raise PublicationError("CSS is not UTF-8") from error
    previous = read_record(record, output)
    if previous is None:
        if target.exists():
            raise PublicationError("refusing unowned existing destination")
    elif not target.exists() or digest(target.read_bytes()) != previous["sha256"]:
        raise PublicationError("accepted destination missing/manually changed; ownership conflict")
    accepted = {"owner": OWNER, "output": output, "sha256": digest(css)}
    if previous == accepted:
        return "unchanged"
    # Persist intent before rename: a crash cannot silently accept a candidate,
    # including a first publication that has no prior ownership record.
    atomic_write(pending, encoded({"candidate": accepted, "previous": previous}))
    replaced = False
    try:
        atomic_write(target, css)
        replaced = True
        atomic_write(record, encoded(accepted))
        pending.unlink()
    except BaseException as error:
        if replaced:
            raise PublicationError("unaccepted partial publication: CSS replaced; explicit reconciliation required") from error
        pending.unlink()
        raise
    return "published"


def reconcile(root, output):
    """Explicit operator acceptance of the matching pending candidate only."""
    target, record, pending = locations(root, output)
    if not pending.exists():
        raise PublicationError("no pending publication to reconcile")
    intent = json.loads(pending.read_text(encoding="utf-8"))
    candidate = intent["candidate"]
    if candidate.get("owner") != OWNER or candidate.get("output") != output or not target.exists() or digest(target.read_bytes()) != candidate.get("sha256"):
        raise PublicationError("pending candidate/destination mismatch; manual investigation required")
    current = read_record(record, output)
    if current not in (intent["previous"], candidate):
        raise PublicationError("pending ownership mismatch")
    atomic_write(record, encoded(candidate))
    pending.unlink()
    return "explicitly reconciled"


def capture(command, cwd):
    # Disk staging avoids an unbounded in-memory subprocess capture. Only <=2 MiB
    # complete output is accepted; stderr remains on the diagnostic channel.
    with tempfile.TemporaryFile() as stage:
        result = subprocess.run(command, cwd=cwd, stdout=stage, check=False)
        if result.returncode:
            raise PublicationError(f"generator failed with exit {result.returncode}; no publication")
        if stage.tell() > LIMIT:
            raise PublicationError("generator output exceeds 2 MiB")
        stage.seek(0)
        css = stage.read(LIMIT + 1)
    if not css:
        raise PublicationError("generator emitted empty CSS")
    try:
        css.decode("utf-8")
    except UnicodeDecodeError as error:
        raise PublicationError("generator output is not UTF-8") from error
    return css


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--target-dir", type=Path, required=True)
    parser.add_argument("--reconcile", action="store_true")
    args = parser.parse_args()
    root = Path(__file__).resolve().parent
    output = "assets/generated/app.generated.css"
    if args.reconcile:
        print(reconcile(root, output), file=sys.stderr)
        return
    if not args.target_dir.is_absolute():
        raise PublicationError("explicit absolute owned target cache required")
    generator = root / "stylegen"
    identity = subprocess.run(["rustc", "+1.99.0", "-vV"], cwd=generator, check=True, capture_output=True, text=True).stdout
    host = next(line.removeprefix("host: ") for line in identity.splitlines() if line.startswith("host: "))
    command = ["cargo", "+1.99.0", "run", "--frozen", "--quiet", "--manifest-path", str(generator / "Cargo.toml"), "--bin", "app-stylegen", "--no-default-features", "--target", host, "--target-dir", str(args.target_dir)]
    print(publish(root, output, capture(command, generator)), file=sys.stderr)


if __name__ == "__main__":
    try:
        main()
    except (PublicationError, OSError, ValueError, subprocess.SubprocessError) as error:
        print(f"style publication: {error}", file=sys.stderr)
        sys.exit(1)
