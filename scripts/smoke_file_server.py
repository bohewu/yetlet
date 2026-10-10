#!/usr/bin/env python3
"""One bounded localhost end-to-end file-service run; owns and reaps its server."""
from __future__ import annotations

import http.client
import os
from pathlib import Path
import secrets
import signal
import socket
import subprocess
import tempfile
import time

ROOT = Path(__file__).resolve().parent.parent
BINARY = ROOT / "target/debug/yetlet-file-server"
MAX_FILE_BYTES = 1024 * 1024


def payload(name: str, data: bytes, *, close: bool = True) -> bytes:
    start = (f'--sample\r\nContent-Disposition: form-data; name="file"; '
             f'filename="{name}"\r\nContent-Type: text/plain\r\n\r\n').encode()
    return start + data + (b"\r\n--sample--\r\n" if close else b"")


def main() -> None:
    if not BINARY.is_file():
        raise RuntimeError("Run cargo build --locked -p yetlet-file-server first")
    token = secrets.token_hex(32)
    with socket.socket() as listener:
        listener.bind(("127.0.0.1", 0))
        port = listener.getsockname()[1]

    def call(method: str, path: str, body: bytes | None = None,
             headers: dict[str, str] | None = None, *, authenticated: bool = False):
        outgoing = dict(headers or {})
        if authenticated:
            outgoing["Authorization"] = f"Bearer {token}"
        if body is not None:
            outgoing.setdefault("Content-Type", "multipart/form-data; boundary=sample")
        client = http.client.HTTPConnection("127.0.0.1", port, timeout=4)
        try:
            client.request(method, path, body=body, headers=outgoing)
            response = client.getresponse()
            return response.status, dict((k.lower(), v) for k, v in response.getheaders()), response.read()
        finally:
            client.close()

    with tempfile.TemporaryDirectory(prefix="yetlet-file-smoke-") as directory:
        directory = Path(directory)
        temporary = directory / "storage"
        temporary.mkdir()
        env = os.environ.copy()
        env.update(YETLET_FILES_LISTEN=f"127.0.0.1:{port}", YETLET_FILES_TOKEN=token, TMPDIR=str(temporary))
        with (directory / "server.log").open("w") as log:
            proc = subprocess.Popen([str(BINARY)], cwd=ROOT, env=env, stdout=log, stderr=log)
            try:
                end = time.monotonic() + 8
                while time.monotonic() < end:
                    if proc.poll() is not None:
                        raise RuntimeError("File sample exited before readiness")
                    try:
                        if call("GET", "/healthz")[0] == 200:
                            break
                    except (OSError, http.client.HTTPException):
                        pass
                    time.sleep(0.05)
                else:
                    raise RuntimeError("File sample did not become ready")

                status, headers, html = call("GET", "/")
                assert status == 200 and b'<main lang="en">' in html
                assert token.encode() not in html and b"<script" not in html
                assert "default-src 'none'" in headers["content-security-policy"]
                status, headers, data = call("GET", "/static/welcome.txt")
                assert status == 200 and data.startswith(b"Yetlet")
                static_length = len(data)
                assert headers["x-content-type-options"] == "nosniff"
                status, headers, data = call("HEAD", "/static/welcome.txt")
                assert status == 200 and not data and int(headers["content-length"]) == static_length
                status, _, data = call("GET", "/static/welcome.txt", headers={"Range": "bytes=0-5"})
                assert status == 206 and data == b"Yetlet"

                text = "A streamed UTF-8 upload.\n".encode()
                good = payload("example.txt", text)
                assert call("POST", "/uploads", good)[0] == 401
                assert call("POST", "/uploads", good, {"Origin": f"http://127.0.0.1:{port}"}, authenticated=True)[0] == 403
                status, headers, _ = call("POST", "/uploads", good, authenticated=True)
                assert status == 201
                resource = headers["location"]
                assert resource.startswith("/files/")
                assert call("GET", resource)[0] == 401
                for _ in range(2):
                    status, headers, data = call("GET", resource, authenticated=True)
                    assert status == 200 and data == text
                    assert headers["content-type"] == "application/octet-stream"
                    assert headers["content-disposition"] == 'attachment; filename="example.txt"'
                    assert headers["cache-control"] == "no-store"
                status, headers, data = call("HEAD", resource, authenticated=True)
                assert status == 200 and not data and int(headers["content-length"]) == len(text)
                status, second, _ = call("POST", "/uploads", good, authenticated=True)
                assert status == 201 and second["location"] != resource

                assert call("POST", "/uploads", payload("../escape.txt", b"bad"), authenticated=True)[0] == 400
                assert call("POST", "/uploads", payload("run.exe", b"bad"), authenticated=True)[0] == 400
                assert call("POST", "/uploads", payload("binary.txt", b"binary\x00"), authenticated=True)[0] == 415
                assert call("POST", "/uploads", payload("large.txt", b"x" * (MAX_FILE_BYTES + 1)), authenticated=True)[0] == 413
                assert call("GET", "/static/%2e%2e%2fCargo.toml")[0] == 404
                assert call("GET", "/files/%2e%2e%2fetc%2fpasswd", authenticated=True)[0] == 404
                assert call("GET", "/static/")[0] == 404
                assert call("GET", "/uploads", authenticated=True)[0] == 405

                # A real truncated HTTP body must leave no partially accepted file.
                roots = list(temporary.glob("yetlet-files-*"))
                assert len(roots) == 1
                files = roots[0] / "uploads"
                baseline = set(files.iterdir())
                partial = payload("aborted.txt", b"unfinished", close=False)
                with socket.create_connection(("127.0.0.1", port), timeout=3) as raw:
                    raw.sendall((f"POST /uploads HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\n"
                                 f"Authorization: Bearer {token}\r\n"
                                 "Content-Type: multipart/form-data; boundary=sample\r\n"
                                 "Content-Length: 100000\r\nConnection: close\r\n\r\n").encode() + partial)
                    deadline = time.monotonic() + 3
                    while set(files.iterdir()) == baseline and time.monotonic() < deadline:
                        time.sleep(0.01)
                    assert len(set(files.iterdir()) - baseline) == 1
                deadline = time.monotonic() + 12
                while set(files.iterdir()) != baseline and time.monotonic() < deadline:
                    time.sleep(0.02)
                assert set(files.iterdir()) == baseline, "disconnected upload left a partial file"
                assert call("POST", "/uploads", good, authenticated=True)[0] == 201
            finally:
                if proc.poll() is None:
                    proc.send_signal(signal.SIGINT)
                    try:
                        proc.wait(timeout=15)
                    except subprocess.TimeoutExpired:
                        proc.kill()
                        proc.wait(timeout=4)
                if proc.returncode != 0:
                    raise RuntimeError(f"File sample did not shut down gracefully: {proc.returncode}")
            assert not list(temporary.iterdir()), "owned temporary root was not removed"
    print("PASS: SSR/static GET/HEAD/Range; token-gated UTF-8 upload and repeat streaming downloads")
    print("PASS: origin/auth/path/type/size/method rejection and real interrupted-upload cleanup")
    print("PASS: owned server reaped and temporary storage removed; no UI/browser or external effects")


if __name__ == "__main__":
    main()
