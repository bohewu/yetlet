#!/usr/bin/env python3
"""Bounded localhost smoke test for the SSR Session cart."""
from __future__ import annotations

from contextlib import closing
import http.client
import os
from pathlib import Path
import re
import socket
import subprocess
import sys
import tempfile
import time
from urllib.parse import urlencode

ROOT = Path(__file__).resolve().parent.parent
BINARY = ROOT / "target/debug/yetlet-cart-server"
CSRF_RE = re.compile(r'name="csrf" value="([0-9a-f]{64})"')


def free_port() -> int:
    with closing(socket.socket()) as listener:
        listener.bind(("127.0.0.1", 0))
        return int(listener.getsockname()[1])


def call(
    port: int,
    method: str,
    path: str,
    *,
    cookie: str | None = None,
    fields: dict[str, str] | None = None,
) -> tuple[int, dict[str, str], bytes]:
    body = None
    headers: dict[str, str] = {}
    if cookie is not None:
        headers["Cookie"] = cookie
    if fields is not None:
        body = urlencode(fields).encode()
        headers["Content-Type"] = "application/x-www-form-urlencoded"
    connection = http.client.HTTPConnection("127.0.0.1", port, timeout=3)
    try:
        connection.request(method, path, body=body, headers=headers)
        response = connection.getresponse()
        response_headers = {name.lower(): value for name, value in response.getheaders()}
        return response.status, response_headers, response.read()
    finally:
        connection.close()


def wait_ready(port: int) -> None:
    deadline = time.monotonic() + 10
    while time.monotonic() < deadline:
        try:
            status, _, body = call(port, "GET", "/healthz")
            if status == 200 and body == b"ok":
                return
        except OSError:
            pass
        time.sleep(0.05)
    raise RuntimeError("cart sample did not become ready")


def cookie_pair(headers: dict[str, str]) -> str:
    return headers["set-cookie"].split(";", 1)[0]


def csrf(body: bytes) -> str:
    match = CSRF_RE.search(body.decode())
    if match is None:
        raise AssertionError("CSRF field not found")
    return match.group(1)


def main() -> int:
    if not BINARY.is_file():
        raise RuntimeError("Build the cart sample before running the smoke test")
    port = free_port()
    env = os.environ.copy()
    env["YETLET_CART_LISTEN"] = f"127.0.0.1:{port}"

    with tempfile.TemporaryDirectory(prefix="yetlet-cart-") as folder:
        log_path = Path(folder) / "server.log"
        with log_path.open("w") as log:
            process = subprocess.Popen(
                [str(BINARY)], cwd=ROOT, env=env, stdout=log, stderr=subprocess.STDOUT
            )
            try:
                wait_ready(port)
                status, headers, _ = call(port, "GET", "/healthz")
                assert status == 200 and "set-cookie" not in headers

                status, headers, body = call(port, "GET", "/cart")
                assert status == 200
                assert b"Yetlet Session Cart" in body and b"<script" not in body
                old_cookie = cookie_pair(headers)
                token = csrf(body)
                set_cookie = headers["set-cookie"]
                assert "HttpOnly" in set_cookie and "SameSite=Lax" in set_cookie
                assert "Path=/" in set_cookie and "Secure" not in set_cookie
                assert headers["cache-control"] == "no-store"

                status, headers, _ = call(
                    port,
                    "POST",
                    "/cart/add",
                    cookie=old_cookie,
                    fields={"csrf": token, "product": "rust-book"},
                )
                assert status == 303 and headers["location"] == "/cart"

                status, _, body = call(port, "GET", "/cart", cookie=old_cookie)
                assert status == 200
                assert b"Quantity: 1" in body and b"Cart total: $39.99" in body
                token = csrf(body)

                status, _, body = call(
                    port,
                    "POST",
                    "/cart/add",
                    cookie=old_cookie,
                    fields={"csrf": "0", "product": "rust-book"},
                )
                assert status == 403 and body == b"Invalid CSRF token"
                status, _, body = call(port, "GET", "/cart", cookie=old_cookie)
                assert b"Quantity: 1" in body and b"Quantity: 2" not in body

                status, headers, _ = call(
                    port,
                    "POST",
                    "/cart/clear",
                    cookie=old_cookie,
                    fields={"csrf": token},
                )
                assert status == 303
                new_cookie = cookie_pair(headers)
                assert new_cookie != old_cookie

                status, _, body = call(port, "GET", "/cart", cookie=new_cookie)
                assert status == 200 and b"Your cart is empty." in body
                new_token = csrf(body)

                status, headers, body = call(port, "GET", "/cart", cookie=old_cookie)
                assert status == 200 and b"Your cart is empty." in body
                assert cookie_pair(headers) != old_cookie

                status, headers, _ = call(
                    port,
                    "POST",
                    "/cart/end",
                    cookie=new_cookie,
                    fields={"csrf": new_token},
                )
                assert status == 303 and "Max-Age=0" in headers["set-cookie"]
                assert call(port, "GET", "/")[0] == 303
                assert call(port, "GET", "/app")[0] == 404

                print("PASS: SSR cart cookie, refresh persistence and bounded forms")
                print("PASS: CSRF rejection, rotation, old-ID invalidation and removal")
                print("PASS: health isolation, strict SSR and no browser route")
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
        print(f"cart smoke FAILED: {error}", file=sys.stderr)
        sys.exit(1)
