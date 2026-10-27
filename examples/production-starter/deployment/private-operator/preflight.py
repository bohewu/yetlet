#!/usr/bin/env python3
"""Optional Starter fixture checks; no database initialization or service control."""
import argparse
import hashlib
import http.client
import json
import os
from pathlib import Path
import signal
import sqlite3
import stat
import sys
import time


def snapshot(path):
    if not path.is_absolute() or path.is_symlink():
        raise ValueError("use an absolute regular database path")
    info = path.stat()
    if not stat.S_ISREG(info.st_mode) or info.st_size == 0:
        raise ValueError("database missing or empty")
    parent = path.parent.stat()
    with sqlite3.connect(path.as_uri() + "?mode=ro", uri=True, timeout=1) as db:
        if db.execute("PRAGMA user_version").fetchone()[0] != 2:
            raise ValueError("only the selected schema 2 is supported")
        schema = db.execute(
            "SELECT type,name,tbl_name,sql FROM sqlite_schema ORDER BY type,name"
        ).fetchall()
        count = db.execute("SELECT count(*) FROM issues").fetchone()[0]
    return {
        "path": str(path), "device": info.st_dev, "inode": info.st_ino,
        "parent_device": parent.st_dev, "parent_inode": parent.st_ino,
        "schema_sha256": hashlib.sha256(json.dumps(schema).encode()).hexdigest(),
        "allow_empty": count == 0,
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest="mode", required=True)
    first = commands.add_parser("init-check", help="explicit first initialization only")
    first.add_argument("database", type=Path)
    for mode in ("record", "restart"):
        command = commands.add_parser(mode)
        command.add_argument("database", type=Path)
        command.add_argument("receipt", type=Path)
        if mode == "record":
            command.add_argument("--allow-empty", action="store_true")
    ready = commands.add_parser("ready", help="private loopback readiness, at most 60 s")
    ready.add_argument("port", type=int, choices=range(1, 65536), metavar="PORT")
    args = parser.parse_args()
    if args.mode == "ready":
        def expired(_signum, _frame):
            raise ValueError("private readiness exceeded 60 seconds")
        signal.signal(signal.SIGALRM, expired)
        signal.setitimer(signal.ITIMER_REAL, 60)
        deadline = time.monotonic() + 60
        while time.monotonic() < deadline:
            connection = http.client.HTTPConnection(
                "127.0.0.1", args.port, timeout=min(2, deadline - time.monotonic())
            )
            try:
                connection.request("GET", "/health/ready", headers={"Connection": "close"})
                response = connection.getresponse()
                if response.status == 200 and response.read(16) == b"ready":
                    return
            except (OSError, http.client.HTTPException):
                pass
            finally:
                connection.close()
            time.sleep(min(0.2, max(0, deadline - time.monotonic())))
        raise ValueError("private readiness did not pass within 60 seconds")
    path = args.database
    if args.mode == "init-check":
        if not path.is_absolute() or not path.parent.is_dir():
            raise ValueError("create and confirm the absolute database parent first")
        if any(p.exists() or p.is_symlink() for p in
               (path, Path(str(path) + "-wal"), Path(str(path) + "-shm"))):
            raise ValueError("initialization requires an absent database and sidecars")
        return
    current = snapshot(path)
    if args.mode == "record":
        if current["allow_empty"] and not args.allow_empty:
            raise ValueError("empty initialization requires explicit --allow-empty")
        # Exclusive creation: never replace an old receipt implicitly.
        descriptor = os.open(args.receipt, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
        with os.fdopen(descriptor, "w") as output:
            json.dump(current, output, indent=2)
            output.write("\n")
    else:
        expected = json.loads(args.receipt.read_text())
        allow_empty = expected.pop("allow_empty")
        empty = current.pop("allow_empty")
        if current != expected or (empty and not allow_empty):
            raise ValueError("database identity/schema or expected nonempty data changed")


if __name__ == "__main__":
    try:
        main()
    except (OSError, ValueError, KeyError, sqlite3.Error) as error:
        # Fixed category avoids exposing paths, SQL or credentials in service output.
        print("Starter reference preflight refused: " + type(error).__name__, file=sys.stderr)
        sys.exit(1)
