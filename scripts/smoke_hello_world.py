#!/usr/bin/env python3
"""Ephemeral localhost-only smoke test for the native Hello World sample."""
from __future__ import annotations

from contextlib import closing
import os
from pathlib import Path
import socket
import subprocess
import tempfile
import time
from urllib.error import HTTPError
from urllib.request import ProxyHandler, Request, build_opener

ROOT = Path(__file__).resolve().parent.parent
BINARY = ROOT / "target/debug/yetlet-hello-world"


def free_port() -> int:
    with closing(socket.socket()) as listener:
        listener.bind(("127.0.0.1", 0))
        return int(listener.getsockname()[1])


def get(base: str, path: str, method: str = "GET") -> tuple[int, dict[str, str], str]:
    request = Request(base + path, method=method)
    opener = build_opener(ProxyHandler({}))
    try:
        with opener.open(request, timeout=3) as response:
            return (
                response.status,
                dict((key.lower(), value) for key, value in response.headers.items()),
                response.read().decode("utf-8"),
            )
    except HTTPError as error:
        return (
            error.code,
            dict((key.lower(), value) for key, value in error.headers.items()),
            error.read().decode("utf-8"),
        )


def main() -> None:
    if not BINARY.is_file():
        raise RuntimeError("Build yetlet-hello-world first")
    port = free_port()
    base = f"http://127.0.0.1:{port}"
    env = os.environ.copy()
    env["YETLET_HELLO_LISTEN"] = f"127.0.0.1:{port}"
    with tempfile.TemporaryDirectory(prefix="yetlet-hello-") as directory:
        with (Path(directory) / "server.log").open("w") as logfile:
            proc = subprocess.Popen(
                [str(BINARY)], cwd=ROOT, env=env, stdout=logfile, stderr=subprocess.STDOUT
            )
            try:
                deadline = time.monotonic() + 8
                while time.monotonic() < deadline:
                    if proc.poll() is not None:
                        raise RuntimeError(f"Hello server exited: {proc.returncode}")
                    try:
                        status, _, html = get(base, "/")
                        if status == 200 and "Hello, world!" in html:
                            break
                    except OSError:
                        pass
                    time.sleep(0.05)
                else:
                    raise RuntimeError("Hello server did not start")

                status, headers, html = get(base, "/")
                assert status == 200 and "<main>" in html
                assert "default-src 'none'" in headers["content-security-policy"]
                assert headers["content-type"].startswith("text/html")
                assert not any(token in html for token in ("<script", "wasm"))
                assert "set-cookie" not in headers
                assert "x-yetlet-request-id" not in headers

                status, headers, html = get(
                    base, "/hello/%3Cimg%20src%3Dx%20onerror%3Dalert(1)%3E"
                )
                assert status == 200
                assert "&lt;img src=x onerror=alert(1)&gt;" in html
                assert "<img" not in html
                assert "set-cookie" not in headers

                assert get(base, "/missing")[0] == 404
                status, headers, _ = get(base, "/hello/Yetlet", "POST")
                assert status == 405 and "allow" in headers
                print("PASS: native SSR, escaped path, strict CSP, no browser/session provider")
                print("PASS: native 404/405 and ephemeral server cleanup")
            finally:
                proc.terminate()
                try:
                    proc.wait(timeout=4)
                except subprocess.TimeoutExpired:
                    proc.kill()
                    proc.wait(timeout=4)


if __name__ == "__main__":
    main()
