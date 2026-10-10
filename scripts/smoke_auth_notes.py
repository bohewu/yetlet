#!/usr/bin/env python3
"""Bounded localhost smoke for synthetic auth, Session rotation, CSRF and IDOR."""
from __future__ import annotations

from contextlib import closing
from html import unescape
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
from urllib.request import HTTPRedirectHandler, ProxyHandler, Request, build_opener

ROOT = Path(__file__).resolve().parent.parent
BINARY = ROOT / "target/debug/yetlet-auth-notes"
ALICE_CODE = "a" * 64
BOB_CODE = "b" * 64


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
    cookie: str | None = None,
    fields: dict[str, str] | None = None,
) -> tuple[int, dict[str, str], str]:
    headers: dict[str, str] = {}
    payload = None
    if cookie:
        headers["Cookie"] = cookie
    if fields is not None:
        payload = urlencode(fields).encode()
        headers["Content-Type"] = "application/x-www-form-urlencoded"
    opener = build_opener(ProxyHandler({}), NoRedirect())
    request = Request(base + path, method=method, data=payload, headers=headers)
    try:
        with opener.open(request, timeout=3) as response:
            return (
                response.status,
                {name.lower(): value for name, value in response.headers.items()},
                response.read().decode(),
            )
    except HTTPError as error:
        return (
            error.code,
            {name.lower(): value for name, value in error.headers.items()},
            error.read().decode(),
        )


def cookie(headers: dict[str, str]) -> str:
    return headers["set-cookie"].split(";", 1)[0]


def csrf(html: str) -> str:
    match = re.search(r'name="csrf" value="([^"]+)"', html)
    if not match:
        raise RuntimeError("CSRF field missing")
    return unescape(match.group(1))


def login(base: str, username: str, code: str) -> tuple[str, str, str, str]:
    status, headers, html = call(base, "GET", "/auth/login")
    assert status == 200
    anonymous_cookie = cookie(headers)
    anonymous_csrf = csrf(html)
    status, headers, _ = call(
        base,
        "POST",
        "/auth/login",
        anonymous_cookie,
        {"csrf": anonymous_csrf, "username": username, "code": code},
    )
    assert status == 303 and headers["location"] == "/auth/notes"
    authenticated_cookie = cookie(headers)
    assert authenticated_cookie != anonymous_cookie
    status, _, notes = call(base, "GET", "/auth/notes", authenticated_cookie)
    assert status == 200
    return authenticated_cookie, csrf(notes), anonymous_cookie, anonymous_csrf


def wait_ready(base: str, process: subprocess.Popen) -> None:
    deadline = time.monotonic() + 10
    while time.monotonic() < deadline:
        if process.poll() is not None:
            raise RuntimeError(f"Auth Notes server exited: {process.returncode}")
        try:
            if call(base, "GET", "/healthz")[0] == 200:
                return
        except OSError:
            pass
        time.sleep(0.05)
    raise RuntimeError("Auth Notes server did not become ready")


def main() -> None:
    if not BINARY.is_file():
        raise RuntimeError("Build yetlet-auth-notes before the smoke test")
    port = free_port()
    base = f"http://127.0.0.1:{port}"
    env = dict(
        os.environ,
        YETLET_AUTH_LISTEN=f"127.0.0.1:{port}",
        YETLET_AUTH_ALICE_CODE=ALICE_CODE,
        YETLET_AUTH_BOB_CODE=BOB_CODE,
    )
    with tempfile.TemporaryDirectory(prefix="yetlet-auth-notes-") as directory:
        log_path = Path(directory) / "server.log"
        with log_path.open("w") as log:
            process = subprocess.Popen(
                [str(BINARY)], cwd=ROOT, env=env,
                stdout=log, stderr=subprocess.STDOUT,
            )
            try:
                wait_ready(base, process)
                status, headers, _ = call(base, "GET", "/healthz")
                assert status == 200 and "set-cookie" not in headers
                status, headers, login_html = call(base, "GET", "/auth/login")
                assert status == 200
                assert ALICE_CODE not in login_html and BOB_CODE not in login_html
                assert "<script" not in login_html
                assert "HttpOnly" in headers["set-cookie"]
                assert "SameSite=Lax" in headers["set-cookie"]
                assert "Secure" not in headers["set-cookie"]

                bad_cookie = cookie(headers)
                bad_csrf = csrf(login_html)
                status, headers, body = call(
                    base, "POST", "/auth/login", bad_cookie,
                    {"csrf": bad_csrf, "username": "alice", "code": BOB_CODE},
                )
                assert status == 401 and "Invalid synthetic identity" in body
                assert "set-cookie" not in headers

                alice_cookie, alice_csrf, alice_old_cookie, alice_old_csrf = login(
                    base, "alice", ALICE_CODE
                )
                assert alice_csrf != alice_old_csrf
                assert call(base, "GET", "/auth/notes", alice_old_cookie)[0] == 401
                status, _, _ = call(
                    base, "POST", "/auth/notes", alice_cookie,
                    {"csrf": alice_old_csrf, "title": "blocked", "body": "blocked"},
                )
                assert status == 403

                status, headers, _ = call(
                    base, "POST", "/auth/notes", alice_cookie,
                    {
                        "csrf": alice_csrf,
                        "title": "<script>alert(1)</script>",
                        "body": "Alice private body",
                    },
                )
                assert status == 303 and headers["location"] == "/auth/notes/1"
                status, _, alice_note = call(base, "GET", "/auth/notes/1", alice_cookie)
                assert status == 200
                assert "&lt;script&gt;alert(1)&lt;/script&gt;" in alice_note
                assert "<script>alert" not in alice_note
                alice_note_csrf = csrf(alice_note)

                bob_cookie, bob_csrf, _, _ = login(base, "bob", BOB_CODE)
                status, _, bob_list = call(base, "GET", "/auth/notes", bob_cookie)
                assert status == 200 and 'data-note-id="1"' not in bob_list
                assert call(base, "GET", "/auth/notes/1", bob_cookie)[0] == 403
                assert call(
                    base, "POST", "/auth/notes/1/update", bob_cookie,
                    {"csrf": bob_csrf, "title": "stolen", "body": "stolen"},
                )[0] == 403
                assert call(
                    base, "POST", "/auth/notes/1/delete", bob_cookie,
                    {"csrf": bob_csrf},
                )[0] == 403

                assert call(
                    base, "POST", "/auth/notes/1/update", alice_cookie,
                    {"csrf": "bad", "title": "changed", "body": "changed"},
                )[0] == 403
                status, _, body = call(base, "GET", "/auth/notes/1", alice_cookie)
                assert status == 200 and "Alice private body" in body
                status, headers, _ = call(
                    base, "POST", "/auth/logout", alice_cookie,
                    {"csrf": alice_note_csrf},
                )
                assert status == 303 and headers["location"] == "/auth/login"
                anonymous_cookie = cookie(headers)
                assert anonymous_cookie != alice_cookie
                assert call(base, "GET", "/auth/notes", anonymous_cookie)[0] == 401
                assert call(base, "GET", "/auth/notes", alice_cookie)[0] == 401

                print("PASS: synthetic code auth, login/logout session rotation and old CSRF retirement")
                print("PASS: per-user note isolation, IDOR 403, HTML escaping and mutation CSRF")
                print("PASS: local cookie policy, no secret SSR leakage and owned process cleanup")
            finally:
                if process.poll() is None:
                    process.send_signal(signal.SIGINT)
                    try:
                        process.wait(timeout=5)
                    except subprocess.TimeoutExpired:
                        process.kill(); process.wait(timeout=5)
                if process.returncode not in (0, -signal.SIGINT):
                    raise RuntimeError(log_path.read_text(errors="replace"))
                log_text = log_path.read_text(errors="replace")
                assert ALICE_CODE not in log_text and BOB_CODE not in log_text


if __name__ == "__main__":
    main()
