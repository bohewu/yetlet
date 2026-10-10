#!/usr/bin/env python3
"""Real localhost integration smoke for the modular shop reference app."""
from __future__ import annotations

from contextlib import closing
import http.cookiejar
import os
from pathlib import Path
import re
import signal
import socket
import subprocess
import tempfile
import time
from urllib.error import HTTPError
from urllib.parse import urlencode
from urllib.request import (
    HTTPRedirectHandler,
    HTTPCookieProcessor,
    ProxyHandler,
    Request,
    build_opener,
)

ROOT = Path(__file__).resolve().parent.parent
BINARY = ROOT / "target/debug/yetlet-modular-shop"


class NoRedirect(HTTPRedirectHandler):
    def redirect_request(self, request, fp, code, msg, headers, newurl):
        return None


def free_port() -> int:
    with closing(socket.socket()) as listener:
        listener.bind(("127.0.0.1", 0))
        return int(listener.getsockname()[1])


def client():
    jar = http.cookiejar.CookieJar()
    return build_opener(ProxyHandler({}), HTTPCookieProcessor(jar), NoRedirect())


def call(opener, base: str, method: str, path: str, fields=None):
    headers = {}
    payload = None
    if fields is not None:
        payload = urlencode(fields).encode()
        headers["content-type"] = "application/x-www-form-urlencoded"
    request = Request(base + path, data=payload, method=method, headers=headers)
    try:
        with opener.open(request, timeout=4) as response:
            return response.status, {k.lower(): v for k, v in response.headers.items()}, response.read().decode()
    except HTTPError as error:
        return error.code, {k.lower(): v for k, v in error.headers.items()}, error.read().decode()


def hidden(html: str, name: str) -> str:
    match = re.search(rf'name="{re.escape(name)}" value="([^"]+)"', html)
    if not match:
        raise AssertionError(f"missing hidden field: {name}")
    return match.group(1)


def wait_ready(opener, base: str, process: subprocess.Popen) -> None:
    deadline = time.monotonic() + 10
    while time.monotonic() < deadline:
        if process.poll() is not None:
            raise RuntimeError(f"shop exited early: {process.returncode}")
        try:
            if call(opener, base, "GET", "/healthz")[0] == 200:
                return
        except OSError:
            pass
        time.sleep(0.05)
    raise RuntimeError("shop did not become ready")


def start(database: Path, disable_orders: bool = False):
    port = free_port()
    base = f"http://127.0.0.1:{port}"
    environment = os.environ.copy()
    environment.update(
        {
            "YETLET_SHOP_LISTEN": f"127.0.0.1:{port}",
            "YETLET_SHOP_DB": str(database),
            "YETLET_SHOP_ALICE_CODE": "a" * 64,
            "YETLET_SHOP_BOB_CODE": "b" * 64,
        }
    )
    if disable_orders:
        environment["YETLET_SHOP_DISABLE_ORDERS"] = "1"
    log = database.parent / f"shop-{port}.log"
    stream = log.open("w")
    process = subprocess.Popen(
        [str(BINARY)], cwd=ROOT, env=environment, stdout=stream, stderr=subprocess.STDOUT
    )
    opener = client()
    try:
        wait_ready(opener, base, process)
    except BaseException:
        cleanup_failed_start(process, stream)
        raise
    return process, stream, opener, base


def cleanup_failed_start(process: subprocess.Popen, stream) -> None:
    try:
        if process.poll() is None:
            process.terminate()
            try:
                process.wait(timeout=4)
            except subprocess.TimeoutExpired:
                process.kill()
                process.wait(timeout=4)
    finally:
        stream.close()


def stop(process: subprocess.Popen, stream) -> None:
    if process.poll() is None:
        process.send_signal(signal.SIGINT)
        try:
            process.wait(timeout=5)
        except subprocess.TimeoutExpired:
            process.kill()
            process.wait(timeout=5)
    stream.close()
    if process.returncode != 0:
        raise RuntimeError(f"shop exit code: {process.returncode}")


def login(opener, base: str, username: str, code: str) -> None:
    status, _, html = call(opener, base, "GET", "/identity/login")
    assert status == 200
    csrf = hidden(html, "csrf")
    status, headers, _ = call(
        opener,
        base,
        "POST",
        "/identity/login",
        {"csrf": csrf, "username": username, "code": code},
    )
    assert status == 303 and headers["location"] == "/shop/catalog"


def add_and_checkout(opener, base: str) -> int:
    status, _, html = call(opener, base, "GET", "/shop/cart")
    assert status == 200
    csrf = hidden(html, "csrf")
    revision = hidden(html, "expected_revision")
    status, _, _ = call(
        opener,
        base,
        "POST",
        "/shop/cart/add",
        {
            "csrf": csrf,
            "product_id": "rust-book",
            "expected_revision": revision,
        },
    )
    assert status == 303
    status, _, html = call(opener, base, "GET", "/shop/cart")
    assert status == 200 and "data-cart-product=\"rust-book\"" in html
    csrf = hidden(html, "csrf")
    revision = hidden(html, "expected_revision")
    status, headers, _ = call(
        opener,
        base,
        "POST",
        "/shop/orders/checkout",
        {"csrf": csrf, "expected_revision": revision},
    )
    assert status == 303
    location = headers["location"]
    assert location.startswith("/shop/orders/")
    order_id = int(location.rsplit("/", 1)[1])
    status, _, html = call(opener, base, "GET", location)
    assert status == 200 and "The Rust Programming Language" in html
    return order_id


def main() -> None:
    if not BINARY.is_file():
        raise RuntimeError("build yetlet-modular-shop before running the smoke test")
    with tempfile.TemporaryDirectory(prefix="yetlet-modular-shop-") as directory:
        root = Path(directory)
        database = root / "shop.sqlite3"

        process, stream, alice, base = start(database)
        try:
            status, headers, html = call(alice, base, "GET", "/shop/catalog")
            assert status == 200 and "Modular Shop Catalog" in html
            assert "<script" not in html
            assert "default-src 'none'" in headers["content-security-policy"]
            login(alice, base, "alice", "a" * 64)
            order_id = add_and_checkout(alice, base)
        finally:
            stop(process, stream)

        process, stream, alice_after_restart, base = start(database)
        try:
            login(alice_after_restart, base, "alice", "a" * 64)
            status, _, html = call(alice_after_restart, base, "GET", "/shop/orders")
            assert status == 200 and f"Order #{order_id}" in html

            bob = client()
            login(bob, base, "bob", "b" * 64)
            status, _, _ = call(bob, base, "GET", f"/shop/orders/{order_id}")
            assert status == 403
        finally:
            stop(process, stream)

        process, stream, disabled, base = start(database, disable_orders=True)
        try:
            status, _, html = call(disabled, base, "GET", "/shop/catalog")
            assert status == 200 and 'href="/shop/orders"' not in html
            assert call(disabled, base, "GET", "/shop/orders")[0] == 404
            login(disabled, base, "alice", "a" * 64)
            status, _, html = call(disabled, base, "GET", "/shop/cart")
            assert status == 200
            assert 'data-orders-disabled="true"' in html
            assert 'action="/shop/orders/checkout"' not in html
        finally:
            stop(process, stream)

        assert database.is_file()
        print("PASS: synthetic login, Session cart, transactional checkout and order SSR")
        print("PASS: SQLite order survives restart; Bob cross-user read is 403")
        print("PASS: disabled Orders removes route, navigation and checkout UI; all servers reaped")


if __name__ == "__main__":
    main()
