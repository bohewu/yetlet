#!/usr/bin/env python3
"""Bounded localhost smoke for stateless native SSR form validation."""
from __future__ import annotations

from contextlib import closing
import os
from pathlib import Path
import socket
import subprocess
import tempfile
import time
from urllib.error import HTTPError
from urllib.parse import urlencode
from urllib.request import HTTPRedirectHandler, ProxyHandler, Request, build_opener

ROOT = Path(__file__).resolve().parent.parent
BINARY = ROOT / "target/debug/yetlet-forms-validation"


class NoRedirect(HTTPRedirectHandler):
    def redirect_request(self, request, fp, code, msg, headers, newurl):
        return None


def free_port() -> int:
    with closing(socket.socket()) as listener:
        listener.bind(("127.0.0.1", 0))
        return int(listener.getsockname()[1])


def call(base: str, path: str, fields: dict[str, str] | None = None) -> tuple[int, dict[str, str], str]:
    headers: dict[str, str] = {}
    method = "GET"
    payload = None
    if fields is not None:
        method = "POST"
        payload = urlencode(fields).encode()
        headers["Content-Type"] = "application/x-www-form-urlencoded"
    opener = build_opener(ProxyHandler({}), NoRedirect())
    request = Request(base + path, data=payload, method=method, headers=headers)
    try:
        with opener.open(request, timeout=3) as response:
            return response.status, {key.lower(): value for key, value in response.headers.items()}, response.read().decode()
    except HTTPError as error:
        return error.code, {key.lower(): value for key, value in error.headers.items()}, error.read().decode()


def wait_ready(base: str, proc: subprocess.Popen) -> None:
    deadline = time.monotonic() + 8
    while time.monotonic() < deadline:
        if proc.poll() is not None:
            raise RuntimeError(f"Forms server exited: {proc.returncode}")
        try:
            if call(base, "/form")[0] == 200:
                return
        except OSError:
            pass
        time.sleep(0.05)
    raise RuntimeError("Forms server did not start")


def main() -> None:
    if not BINARY.is_file():
        raise RuntimeError("Build the forms sample before the smoke test")
    port = free_port()
    base = f"http://127.0.0.1:{port}"
    env = os.environ.copy()
    env["YETLET_FORMS_LISTEN"] = f"127.0.0.1:{port}"
    with tempfile.TemporaryDirectory(prefix="yetlet-forms-") as directory:
        with (Path(directory) / "server.log").open("w") as logfile:
            proc = subprocess.Popen(
                [str(BINARY)], cwd=ROOT, env=env, stdout=logfile, stderr=subprocess.STDOUT
            )
            try:
                wait_ready(base, proc)
                status, headers, html = call(base, "/form")
                assert status == 200
                assert "form-action 'self'" in headers["content-security-policy"]
                assert "for=\"name\"" in html and "for=\"email\"" in html
                assert "<script" not in html and "set-cookie" not in headers
                assert html.startswith('<!doctype html><html lang="en">')
                assert '<main lang=' not in html

                status, headers, html = call(base, "/form/submit", {
                    "name": "<script>", "email": 'a" onfocus=alert(1)'
                })
                assert status == 422
                assert html.startswith('<!doctype html><html lang="en">')
                assert '<main lang=' not in html
                assert "&lt;script&gt;" in html and "&quot;" in html
                assert "aria-describedby" in html and "email-error" in html
                assert "autofocus" in html
                assert "<script>" not in html
                assert headers["cache-control"] == "no-store"
                status, _, html = call(base, "/form/submit", {
                    "name": "x", "email": "not-an-email"
                })
                assert status == 422
                assert "name-error" in html and "email-error" in html
                assert 'aria-describedby="name-error"' in html
                assert "autofocus" in html

                status, headers, _ = call(base, "/form/submit", {
                    "name": "Ada Lovelace", "email": "ada@example.com"
                })
                assert status == 303 and headers["location"] == "/form/success"
                assert "set-cookie" not in headers
                status, _, html = call(base, "/form/success")
                assert status == 200 and "No data was saved." in html
                assert html.startswith('<!doctype html><html lang="en">')
                assert '<main lang=' not in html
                assert "Ada Lovelace" not in html
                assert call(base, "/form")[0] == 200
                print("PASS: native SSR forms, escaped field errors, strict CSP and 422")
                print("PASS: stateless 303 PRG, no cookie or saved state, process cleanup")
            finally:
                proc.terminate()
                try:
                    proc.wait(timeout=4)
                except subprocess.TimeoutExpired:
                    proc.kill()
                    proc.wait(timeout=4)


if __name__ == "__main__":
    main()
