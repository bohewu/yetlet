#!/usr/bin/env python3
"""One bounded headless-Chrome E2E smoke for Yetlet SSR and browser Wasm.

No third-party Python packages. Starts/stops only its own local server and
ephemeral headless Chrome instance. Never touches the user's browser session.
"""
from __future__ import annotations

import base64
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import socket
import struct
import subprocess
import tempfile
import time
import urllib.request
from urllib.parse import urlparse

ROOT = Path(__file__).resolve().parent.parent


def port_number() -> int:
    with socket.socket() as sock:
        sock.bind(("127.0.0.1", 0))
        return sock.getsockname()[1]


def http(url: str) -> tuple[int, dict[str, str], bytes]:
    request = urllib.request.Request(url, headers={"Cache-Control": "no-store"})
    with urllib.request.build_opener(urllib.request.ProxyHandler({})).open(
        request, timeout=4
    ) as response:
        return response.status, {k.lower(): v for k, v in response.headers.items()}, response.read()


class ChromeDevTools:
    def __init__(self, url: str):
        parsed = urlparse(url)
        self.sock = socket.create_connection((parsed.hostname, parsed.port), timeout=8)
        self.sock.settimeout(12)
        self.buffer = b""
        self.next_id = 0
        self.requests: list[str] = []
        self.websockets: list[str] = []
        self.exceptions: list[str] = []
        self.failed_responses: list[str] = []
        key = base64.b64encode(os.urandom(16)).decode()
        handshake = (
            f"GET {parsed.path} HTTP/1.1\r\n"
            f"Host: {parsed.hostname}:{parsed.port}\r\n"
            "Upgrade: websocket\r\nConnection: Upgrade\r\n"
            f"Sec-WebSocket-Key: {key}\r\nSec-WebSocket-Version: 13\r\n"
            "Origin: http://127.0.0.1\r\n\r\n"
        ).encode()
        self.sock.sendall(handshake)
        while b"\r\n\r\n" not in self.buffer:
            part = self.sock.recv(4096)
            if not part:
                raise RuntimeError("Chrome websocket handshake closed")
            self.buffer += part
        header, self.buffer = self.buffer.split(b"\r\n\r\n", 1)
        if b" 101 " not in header.split(b"\r\n", 1)[0]:
            raise RuntimeError(f"Chrome rejected websocket: {header[:200]!r}")
        actual_accept = re.search(br"(?im)^sec-websocket-accept:\s*(\S+)", header)
        expected_accept = base64.b64encode(
            hashlib.sha1((key + "258EAFA5-E914-47DA-95CA-C5AB0DC85B11").encode()).digest()
        )
        if not actual_accept or actual_accept.group(1) != expected_accept:
            raise RuntimeError("Chrome websocket accept mismatch")

    def _read(self, count: int) -> bytes:
        while len(self.buffer) < count:
            part = self.sock.recv(65536)
            if not part:
                raise RuntimeError("Chrome websocket disconnected")
            self.buffer += part
        data, self.buffer = self.buffer[:count], self.buffer[count:]
        return data

    def _send(self, content: bytes, opcode: int = 1) -> None:
        size = len(content)
        if size < 126:
            prefix = bytes((0x80 | opcode, 0x80 | size))
        elif size < 65536:
            prefix = bytes((0x80 | opcode, 0xFE)) + struct.pack("!H", size)
        else:
            prefix = bytes((0x80 | opcode, 0xFF)) + struct.pack("!Q", size)
        mask = os.urandom(4)
        self.sock.sendall(
            prefix + mask + bytes(b ^ mask[i % 4] for i, b in enumerate(content))
        )

    def _recv(self) -> dict | None:
        first, second = self._read(2)
        opcode, size = first & 0x0F, second & 0x7F
        if size == 126:
            size = struct.unpack("!H", self._read(2))[0]
        elif size == 127:
            size = struct.unpack("!Q", self._read(8))[0]
        mask = self._read(4) if second & 0x80 else None
        content = self._read(size)
        if mask:
            content = bytes(b ^ mask[i % 4] for i, b in enumerate(content))
        if opcode == 8:
            raise RuntimeError("Chrome closed its websocket")
        if opcode == 9:
            self._send(content, opcode=10)
            return None
        if opcode == 10:
            return None
        if opcode != 1:
            raise RuntimeError(f"Unexpected Chrome websocket opcode {opcode}")
        return json.loads(content)

    def call(self, method: str, params: dict | None = None) -> dict:
        self.next_id += 1
        command_id = self.next_id
        self._send(json.dumps({"id": command_id, "method": method, "params": params or {}}).encode())
        deadline = time.monotonic() + 12
        while time.monotonic() < deadline:
            message = self._recv()
            if message is None:
                continue
            event = message.get("method")
            if event == "Network.requestWillBeSent":
                self.requests.append(message["params"]["request"]["url"])
            elif event == "Network.responseReceived":
                response = message["params"]["response"]
                if response["status"] >= 400:
                    self.failed_responses.append(
                        f"{int(response['status'])} {response['url']}"
                    )
            elif event == "Network.webSocketCreated":
                self.websockets.append(message["params"]["url"])
            elif event == "Runtime.exceptionThrown":
                details = message["params"]["exceptionDetails"]
                self.exceptions.append(details.get("text", "JS exception"))
            elif event == "Log.entryAdded":
                entry = message["params"]["entry"]
                if entry.get("level") == "error":
                    self.exceptions.append(entry.get("text", "Browser log error"))
            if message.get("id") == command_id:
                if "error" in message:
                    raise RuntimeError(f"{method}: {message['error']}")
                return message.get("result", {})
        raise TimeoutError(f"Timed out on DevTools {method}")

    def evaluate(self, expression: str):
        data = self.call(
            "Runtime.evaluate", {"expression": expression, "returnByValue": True}
        )
        if "exceptionDetails" in data:
            raise RuntimeError(f"JS evaluation error: {data['exceptionDetails']}")
        return data["result"].get("value")

    def close(self) -> None:
        self.sock.close()


def wait_until(predicate, description, seconds=18):
    deadline = time.monotonic() + seconds
    while time.monotonic() < deadline:
        try:
            value = predicate()
            if value:
                return value
        except (ConnectionError, OSError):
            pass
        time.sleep(0.15)
    raise TimeoutError(f"Timed out waiting for {description}")


def main() -> None:
    binary = ROOT / "target/debug/yetlet-todo-server"
    chrome = shutil.which("google-chrome-stable") or shutil.which("google-chrome")
    if not binary.is_file():
        raise RuntimeError("Build the native server first: cargo build -p yetlet-todo-server")
    if not chrome:
        raise RuntimeError("Headless Chrome is required")

    with tempfile.TemporaryDirectory(prefix="yetlet-e2e-") as temp:
        server_port, chrome_port = port_number(), port_number()
        base = f"http://127.0.0.1:{server_port}"
        env = dict(os.environ, YETLET_LISTEN=f"127.0.0.1:{server_port}")
        with (Path(temp) / "server.log").open("w") as server_log, (
            Path(temp) / "chrome.log"
        ).open("w") as chrome_log:
            server = subprocess.Popen(
                [str(binary)], cwd=ROOT, env=env,
                stdout=server_log, stderr=subprocess.STDOUT,
            )
            browser = None
            client = None
            try:
                wait_until(lambda: http(base + "/healthz")[0] == 200, "server")
                status, headers, root = http(base + "/")
                assert status == 200
                ssr = root.decode()
                assert "<script" not in ssr and ".wasm" not in ssr
                assert "unsafe-inline" not in headers["content-security-policy"]
                list_html = re.search(r'<ul class="todo-list">.*?</ul>', ssr)
                assert list_html, "Missing TodoList in SSR response"

                status, headers, shell = http(base + "/app")
                assert status == 200
                assert "'wasm-unsafe-eval'" in headers["content-security-policy"]
                assert "unsafe-inline" not in headers["content-security-policy"]
                assert b'type="module"' in shell

                browser = subprocess.Popen(
                    [
                        chrome,
                        "--headless=new", "--no-sandbox", "--disable-gpu",
                        "--disable-background-networking", "--no-first-run",
                        "--disable-extensions",
                        "--remote-debugging-address=127.0.0.1",
                        "--remote-allow-origins=*",
                        f"--remote-debugging-port={chrome_port}",
                        f"--user-data-dir={Path(temp) / 'chrome-profile'}",
                        "about:blank",
                    ],
                    stdout=chrome_log, stderr=subprocess.STDOUT,
                )
                endpoint = f"http://127.0.0.1:{chrome_port}/json/list"

                def get_target():
                    data = json.loads(http(endpoint)[2])
                    for item in data:
                        if item.get("type") == "page":
                            return item["webSocketDebuggerUrl"]
                    return None

                socket_url = wait_until(get_target, "Chrome debugger")
                client = ChromeDevTools(socket_url)
                for method in ("Runtime.enable", "Page.enable", "Network.enable", "Log.enable"):
                    client.call(method)
                client.call("Page.navigate", {"url": base + "/app"})
                wait_until(
                    lambda: client.evaluate(
                        "document.documentElement.getAttribute('data-yetlet-ready') === 'true'"
                    ),
                    "WASM mount",
                )
                rendered = client.evaluate(
                    "document.querySelector('#todo-list-host ul.todo-list').outerHTML"
                )
                assert rendered == list_html.group(0), (
                    f"SSR/DOM mismatch\nSSR: {list_html.group(0)}\nDOM: {rendered}"
                )
                assert client.evaluate(
                    "document.querySelector('[data-id=\"1\"]').getAttribute('data-completed')"
                ) == "false"
                time.sleep(0.3)
                request_count = len(client.requests)
                client.evaluate("document.getElementById('toggle-first').click()")
                wait_until(
                    lambda: client.evaluate(
                        "document.querySelector('[data-id=\"1\"]').getAttribute('data-completed')"
                    ) == "true",
                    "local click state update",
                )
                assert client.evaluate(
                    "document.querySelector('[data-id=\"1\"]').className"
                ) == "is-complete"
                assert client.evaluate(
                    "document.getElementById('todo-status').textContent"
                ) == "First task: completed"
                client.evaluate("document.getElementById('toggle-first').click()")
                wait_until(
                    lambda: client.evaluate(
                        "document.querySelector('[data-id=\"1\"]').getAttribute('data-completed')"
                    ) == "false",
                    "second click state update",
                )
                time.sleep(0.3)
                assert len(client.requests) == request_count, (
                    f"Click unexpectedly made HTTP requests: {client.requests[request_count:]}"
                )
                assert not client.websockets, f"Unexpected WebSocket: {client.websockets}"
                assert not client.failed_responses, f"HTTP resource failures: {client.failed_responses}"
                assert not client.exceptions, (
                    f"Browser errors: {client.exceptions}; requests: {client.requests}"
                )
                print("PASS: SSR HTML without JS/WASM + strict CSP")
                print("PASS: browser WASM mounts shared TodoList, byte-equal to SSR fixture")
                print("PASS: two local click transitions, class/text updates")
                print("PASS: no click network requests, no WebSocket, no browser errors")
                print(f"Browser asset requests observed: {request_count}")
            finally:
                if client is not None:
                    client.close()
                for process in (browser, server):
                    if process is not None and process.poll() is None:
                        process.terminate()
                        try:
                            process.wait(timeout=5)
                        except subprocess.TimeoutExpired:
                            process.kill()
                            process.wait(timeout=5)
                assert server.poll() is not None
                assert browser is None or browser.poll() is not None


if __name__ == "__main__":
    main()
