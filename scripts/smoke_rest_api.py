#!/usr/bin/env python3
"""Bounded localhost smoke test for the API-only REST sample."""
from __future__ import annotations

from contextlib import closing
import json
import os
from pathlib import Path
import socket
import subprocess
import sys
import tempfile
import time
from urllib.error import HTTPError
from urllib.request import Request, urlopen

ROOT = Path(__file__).resolve().parent.parent
BINARY = ROOT / "target/debug/yetlet-rest-api"


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
    payload: bytes | None
    request_headers = dict(headers or {})
    if body is None or isinstance(body, bytes):
        payload = body
    else:
        payload = json.dumps(body).encode()
        request_headers.setdefault("content-type", "application/json")
    request = Request(base + path, data=payload, headers=request_headers, method=method)
    try:
        with urlopen(request, timeout=3) as response:
            response_headers = {name.lower(): value for name, value in response.headers.items()}
            return response.status, response_headers, response.read()
    except HTTPError as error:
        response_headers = {name.lower(): value for name, value in error.headers.items()}
        return error.code, response_headers, error.read()


def decoded(payload: bytes) -> object:
    return json.loads(payload.decode())


def wait_ready(base: str) -> None:
    deadline = time.monotonic() + 10
    while time.monotonic() < deadline:
        try:
            status, _, body = call(base, "GET", "/healthz")
            if status == 200 and decoded(body) == {"status": "ok"}:
                return
        except OSError:
            pass
        time.sleep(0.05)
    raise RuntimeError("REST sample did not become ready")


def main() -> int:
    if not BINARY.is_file():
        raise RuntimeError("Build the REST sample before running the smoke test")
    port = free_port()
    base = f"http://127.0.0.1:{port}"
    env = os.environ.copy()
    env["YETLET_REST_LISTEN"] = f"127.0.0.1:{port}"

    with tempfile.TemporaryDirectory(prefix="yetlet-rest-api-") as folder:
        log_path = Path(folder) / "server.log"
        with log_path.open("w") as log:
            process = subprocess.Popen(
                [str(BINARY)], cwd=ROOT, env=env, stdout=log, stderr=subprocess.STDOUT
            )
            try:
                wait_ready(base)
                status, headers, body = call(base, "GET", "/api/books")
                assert status == 200
                assert decoded(body) == {"items": [], "count": 0}
                assert headers["x-yetlet-api-version"] == "1"
                assert headers["cache-control"] == "no-store"
                assert "x-yetlet-request-id" in headers

                status, headers, body = call(
                    base,
                    "POST",
                    "/api/books",
                    {"title": "API Design", "author": "Yetlet", "year": 2026},
                )
                assert status == 201
                created = decoded(body)
                assert created["id"] == 1 and created["version"] == 1
                assert headers["location"] == "/api/books/1"

                status, _, body = call(base, "GET", "/api/books/1")
                assert status == 200 and decoded(body)["title"] == "API Design"
                status, _, body = call(base, "GET", "/api/books?q=yetlet&limit=5")
                assert status == 200 and decoded(body)["count"] == 1

                stale = {
                    "title": "Changed",
                    "author": "Yetlet",
                    "year": 2026,
                    "expected_version": 0,
                }
                status, _, body = call(base, "PUT", "/api/books/1", stale)
                assert status == 409 and decoded(body)["error"]["code"] == "stale_version"
                stale["expected_version"] = 1
                status, _, body = call(base, "PUT", "/api/books/1", stale)
                assert status == 200 and decoded(body)["version"] == 2

                status, _, body = call(
                    base,
                    "POST",
                    "/api/books",
                    {"title": "Attack", "author": "Other", "year": 2020},
                    {"Origin": "https://other.example", "Host": f"127.0.0.1:{port}"},
                )
                assert status == 403 and decoded(body)["error"]["code"] == "forbidden_origin"

                status, _, body = call(
                    base,
                    "POST",
                    "/api/books",
                    b"{}",
                    {"content-type": "text/plain"},
                )
                assert status == 415 and decoded(body)["error"]["code"] == "unsupported_media_type"

                status, _, body = call(base, "DELETE", "/api/books/1")
                assert status == 204 and body == b""
                status, _, body = call(base, "GET", "/api/books/1")
                assert status == 404 and decoded(body)["error"]["code"] == "not_found"
                assert call(base, "GET", "/app")[0] == 404
                assert call(base, "GET", "/")[0] == 404

                print("PASS: API-only health, CRUD, filtering and optimistic update")
                print("PASS: typed policy/errors, request correlation and no browser routes")
                return 0
            finally:
                if process.poll() is None:
                    process.terminate()
                    try:
                        process.wait(timeout=5)
                    except subprocess.TimeoutExpired:
                        process.kill()
                        process.wait(timeout=5)
                if process.returncode not in (0, -15):
                    print(log_path.read_text(errors="replace"), file=sys.stderr)


if __name__ == "__main__":
    try:
        sys.exit(main())
    except (AssertionError, OSError, RuntimeError, ValueError) as error:
        print(f"REST smoke FAILED: {error}", file=sys.stderr)
        sys.exit(1)
