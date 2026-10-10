#!/usr/bin/env python3
"""Restart-persistent localhost smoke for the SQL-first SQLite CRUD sample."""
from __future__ import annotations

from concurrent.futures import ThreadPoolExecutor
from contextlib import closing
import json
import os
from pathlib import Path
import signal
import socket
import subprocess
import sys
import tempfile
import time
from urllib.error import HTTPError
from urllib.request import HTTPRedirectHandler, ProxyHandler, Request, build_opener

ROOT = Path(__file__).resolve().parent.parent
BINARY = ROOT / "target/debug/yetlet-sqlite-crud"


class NoRedirect(HTTPRedirectHandler):
    def redirect_request(self, request, fp, code, msg, headers, newurl):
        return None


def free_port() -> int:
    with closing(socket.socket()) as listener:
        listener.bind(("127.0.0.1", 0))
        return int(listener.getsockname()[1])


def call(
    base: str,
    method: str,
    path: str,
    body: object | bytes | None = None,
    headers: dict[str, str] | None = None,
) -> tuple[int, dict[str, str], bytes]:
    request_headers = dict(headers or {})
    if body is None or isinstance(body, bytes):
        payload = body
    else:
        payload = json.dumps(body).encode()
        request_headers.setdefault("content-type", "application/json")
    opener = build_opener(ProxyHandler({}), NoRedirect())
    request = Request(base + path, data=payload, headers=request_headers, method=method)
    try:
        with opener.open(request, timeout=5) as response:
            return (
                response.status,
                {key.lower(): value for key, value in response.headers.items()},
                response.read(),
            )
    except HTTPError as error:
        return (
            error.code,
            {key.lower(): value for key, value in error.headers.items()},
            error.read(),
        )


def decoded(payload: bytes) -> object:
    return json.loads(payload.decode())


def launch(db_path: Path, port: int, log_path: Path) -> tuple[subprocess.Popen, str]:
    env = os.environ.copy()
    env["YETLET_SQLITE_LISTEN"] = f"127.0.0.1:{port}"
    env["YETLET_SQLITE_PATH"] = str(db_path)
    log = log_path.open("w")
    process = subprocess.Popen(
        [str(BINARY)], cwd=ROOT, env=env, stdout=log, stderr=subprocess.STDOUT
    )
    process._yetlet_log = log  # type: ignore[attr-defined]
    base = f"http://127.0.0.1:{port}"
    deadline = time.monotonic() + 12
    while time.monotonic() < deadline:
        if process.poll() is not None:
            log.close()
            raise RuntimeError(f"SQLite sample exited early: {log_path.read_text(errors='replace')}")
        try:
            status, _, body = call(base, "GET", "/healthz")
            if status == 200 and decoded(body) == {"status": "ok", "schema_version": 1}:
                return process, base
        except OSError:
            pass
        time.sleep(0.05)
    stop(process)
    raise RuntimeError("SQLite sample did not become ready")


def stop(process: subprocess.Popen) -> None:
    if process.poll() is None:
        process.send_signal(signal.SIGINT)
        try:
            process.wait(timeout=6)
        except subprocess.TimeoutExpired:
            process.kill()
            process.wait(timeout=5)
    log = getattr(process, "_yetlet_log", None)
    if log is not None and not log.closed:
        log.close()
    if process.returncode not in (0, -signal.SIGINT):
        raise RuntimeError(f"SQLite sample exited with {process.returncode}")


def main() -> int:
    if not BINARY.is_file():
        raise RuntimeError("Build yetlet-sqlite-crud before running the smoke test")

    temporary_path: Path
    with tempfile.TemporaryDirectory(prefix="yetlet-sqlite-crud-") as folder:
        temporary_path = Path(folder)
        database_path = temporary_path / "notes.sqlite3"

        first, first_base = launch(database_path, free_port(), temporary_path / "first.log")
        try:
            status, headers, body = call(
                first_base,
                "POST",
                "/api/notes",
                {"title": "Persistent note", "body": "survives process restart"},
            )
            assert status == 201
            created = decoded(body)
            assert created["id"] == 1 and created["version"] == 1
            assert headers["location"] == "/api/notes/1"
            assert headers["x-yetlet-sqlite-schema"] == "1"
            assert headers["cache-control"] == "no-store"

            status, _, body = call(
                first_base,
                "POST",
                "/api/notes",
                {"title": "Browser write", "body": "rejected"},
                {"Origin": "https://other.example"},
            )
            assert status == 403
            assert decoded(body)["error"]["code"] == "browser_origin_forbidden"

            status, _, body = call(
                first_base,
                "PUT",
                "/api/notes/1",
                {"title": "stale", "body": "no", "expected_version": 9},
            )
            assert status == 409
            assert decoded(body)["error"]["code"] == "stale_version"

            status, _, body = call(
                first_base,
                "PUT",
                "/api/notes/1",
                {"title": "Persisted update", "body": "version two", "expected_version": 1},
            )
            assert status == 200 and decoded(body)["version"] == 2
            assert database_path.is_file()
        finally:
            stop(first)

        second, second_base = launch(database_path, free_port(), temporary_path / "second.log")
        try:
            status, _, body = call(second_base, "GET", "/api/notes/1")
            persisted = decoded(body)
            assert status == 200
            assert persisted["title"] == "Persisted update" and persisted["version"] == 2

            def update(title: str) -> tuple[int, dict[str, str], bytes]:
                return call(
                    second_base,
                    "PUT",
                    "/api/notes/1",
                    {"title": title, "body": title, "expected_version": 2},
                )

            with ThreadPoolExecutor(max_workers=2) as executor:
                results = list(executor.map(update, ["Concurrent A", "Concurrent B"]))
            assert sorted(status for status, _, _ in results) == [200, 409]

            status, _, body = call(second_base, "GET", "/api/notes/1")
            current = decoded(body)
            assert status == 200 and current["version"] == 3

            status, _, body = call(
                second_base, "DELETE", "/api/notes/1?expected_version=2"
            )
            assert status == 409 and decoded(body)["error"]["code"] == "stale_version"
            status, _, body = call(
                second_base, "DELETE", "/api/notes/1?expected_version=3"
            )
            assert status == 204 and body == b""
            assert call(second_base, "GET", "/api/notes/1")[0] == 404
            assert call(second_base, "GET", "/app")[0] == 404
        finally:
            stop(second)

        third, third_base = launch(database_path, free_port(), temporary_path / "third.log")
        try:
            status, _, body = call(third_base, "GET", "/api/notes")
            listing = decoded(body)
            assert status == 200 and listing == {"items": [], "count": 0}
        finally:
            stop(third)

        print("PASS: migration, parameterized CRUD, stale/conflicting writes and JSON policy")
        print("PASS: committed state survives restart; delete survives second restart")
        print("PASS: all owned servers stopped and temporary database cleanup follows")

    assert not temporary_path.exists()
    return 0


if __name__ == "__main__":
    try:
        sys.exit(main())
    except (AssertionError, OSError, RuntimeError, ValueError) as error:
        print(f"SQLite CRUD smoke FAILED: {error}", file=sys.stderr)
        sys.exit(1)
