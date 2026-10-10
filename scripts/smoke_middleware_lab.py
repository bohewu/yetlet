#!/usr/bin/env python3
"""Bounded real localhost checks for the native Yetlet middleware lab."""
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
BINARY = ROOT / "target/debug/yetlet-middleware-lab"


def free_port() -> int:
    with closing(socket.socket()) as listener:
        listener.bind(("127.0.0.1", 0))
        return int(listener.getsockname()[1])


def call(base: str, path: str, headers: dict[str, str] | None = None,
         method: str = "GET") -> tuple[int, dict[str, str], bytes]:
    opener = build_opener(ProxyHandler({}))
    request = Request(base + path, headers=headers or {}, method=method)
    try:
        with opener.open(request, timeout=3) as response:
            return response.status, dict((key.lower(), value) for key, value in response.headers.items()), response.read()
    except HTTPError as error:
        return error.code, dict((key.lower(), value) for key, value in error.headers.items()), error.read()


def wait_ready(base: str, proc: subprocess.Popen) -> None:
    deadline = time.monotonic() + 8
    while time.monotonic() < deadline:
        if proc.poll() is not None:
            raise RuntimeError(f"Middleware lab exited early: {proc.returncode}")
        try:
            if call(base, "/outside")[0] == 200:
                return
        except OSError:
            pass
        time.sleep(0.05)
    raise RuntimeError("Middleware lab did not start")


def main() -> None:
    if not BINARY.is_file():
        raise RuntimeError("Build the middleware lab before the smoke test")
    port = free_port()
    base = f"http://127.0.0.1:{port}"
    env = os.environ.copy()
    env["YETLET_MW_LISTEN"] = f"127.0.0.1:{port}"
    with tempfile.TemporaryDirectory(prefix="yetlet-middleware-") as directory:
        with (Path(directory) / "server.log").open("w") as logfile:
            proc = subprocess.Popen(
                [str(BINARY)], cwd=ROOT, env=env, stdout=logfile, stderr=subprocess.STDOUT
            )
            try:
                wait_ready(base, proc)
                status, headers, body = call(base, "/lab/guarded", {"x-lab-user": "Alice"})
                assert status == 200
                assert body == b"Service: Yetlet; Request: Alice"
                assert headers["x-mw-order"] == (
                    "global.before,second.before,group.before,route.before,handler,"
                    "route.after,group.after,second.after,global.after"
                )
                assert headers["x-native-tower"] == "tower-http"

                status, headers, _ = call(base, "/lab/guarded", {"x-stop": "1"})
                assert status == 403
                assert "handler" not in headers["x-mw-order"]

                status, headers, body = call(base, "/outside")
                assert status == 200 and body == b"Outside: Yetlet"
                assert "group.before" not in headers["x-mw-order"]

                status, headers, _ = call(base, "/lab/missing")
                assert status == 404
                assert headers["x-mw-order"] == (
                    "global.before,second.before,second.after,global.after"
                )
                status, headers, _ = call(base, "/lab/guarded", method="POST")
                assert status == 405 and "GET" in headers["allow"]
                assert "route.before" in headers["x-mw-order"]

                status, headers, body = call(base, "/lab/stream")
                assert status == 200 and body == b"first|second"
                assert "group.after" in headers["x-mw-order"]
                print("PASS: global/group/route scope, reverse unwind and short circuit")
                print("PASS: typed request state, native Tower header, 404/405 and HTTP stream")
            finally:
                proc.terminate()
                try:
                    proc.wait(timeout=4)
                except subprocess.TimeoutExpired:
                    proc.kill()
                    proc.wait(timeout=4)


if __name__ == "__main__":
    main()
