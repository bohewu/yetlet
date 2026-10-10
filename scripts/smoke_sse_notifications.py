#!/usr/bin/env python3
"""Bounded localhost SSE heartbeat, publish, disconnect and replay smoke."""
from __future__ import annotations

from contextlib import closing
import http.client
import json
import os
from pathlib import Path
import signal
import socket
import subprocess
import tempfile
import time
from urllib.error import HTTPError
from urllib.request import ProxyHandler, Request, build_opener

ROOT = Path(__file__).resolve().parent.parent
BINARY = ROOT / "target/debug/yetlet-sse-notifications"


def free_port() -> int:
    with closing(socket.socket()) as listener:
        listener.bind(("127.0.0.1", 0))
        return int(listener.getsockname()[1])


def call(base: str, method: str, path: str, body: object | None = None,
         headers: dict[str, str] | None = None) -> tuple[int, dict[str, str], object]:
    request_headers = dict(headers or {})
    payload = None
    if body is not None:
        payload = json.dumps(body).encode()
        request_headers.setdefault("content-type", "application/json")
    request = Request(base + path, method=method, data=payload, headers=request_headers)
    opener = build_opener(ProxyHandler({}))
    try:
        with opener.open(request, timeout=3) as response:
            raw = response.read()
            parsed = json.loads(raw) if raw else None
            return response.status, {k.lower(): v for k, v in response.headers.items()}, parsed
    except HTTPError as error:
        raw = error.read()
        parsed = json.loads(raw) if raw else None
        return error.code, {k.lower(): v for k, v in error.headers.items()}, parsed


def open_stream(port: int, last_event_id: int | None = None):
    connection = http.client.HTTPConnection("127.0.0.1", port, timeout=3)
    headers = {}
    if last_event_id is not None:
        headers["Last-Event-ID"] = str(last_event_id)
    connection.request("GET", "/events", headers=headers)
    response = connection.getresponse()
    assert response.status == 200, response.status
    assert response.headers["content-type"].startswith("text/event-stream")
    assert response.headers["cache-control"] == "no-store"
    assert response.headers["x-accel-buffering"] == "no"
    return connection, response


def read_item(response: http.client.HTTPResponse) -> dict[str, str]:
    item: dict[str, str] = {}
    data: list[str] = []
    while True:
        raw = response.fp.readline()  # type: ignore[union-attr]
        if raw == b"":
            raise RuntimeError("SSE stream ended unexpectedly")
        line = raw.decode("utf-8").rstrip("\r\n")
        if not line:
            if data:
                item["data"] = "\n".join(data)
            if item:
                return item
            continue
        if line.startswith(":"):
            return {"comment": line[1:].strip()}
        field, separator, value = line.partition(":")
        if not separator:
            continue
        value = value[1:] if value.startswith(" ") else value
        if field == "data":
            data.append(value)
        elif field in {"id", "event", "retry"}:
            item[field] = value


def wait_ready(base: str, process: subprocess.Popen) -> None:
    deadline = time.monotonic() + 10
    while time.monotonic() < deadline:
        if process.poll() is not None:
            raise RuntimeError(f"SSE server exited early: {process.returncode}")
        try:
            status, _, body = call(base, "GET", "/healthz")
            if status == 200 and body == {"status": "ok"}:
                return
        except OSError:
            pass
        time.sleep(0.05)
    raise RuntimeError("SSE server did not become ready")


def wait_subscribers(base: str, expected: int) -> None:
    deadline = time.monotonic() + 3
    while time.monotonic() < deadline:
        status, _, body = call(base, "GET", "/metrics")
        if status == 200 and body["active_subscribers"] == expected:
            return
        time.sleep(0.05)
    raise RuntimeError(f"subscriber count did not become {expected}")


def main() -> None:
    if not BINARY.is_file():
        raise RuntimeError("Build yetlet-sse-notifications before the smoke test")
    port = free_port()
    base = f"http://127.0.0.1:{port}"
    env = os.environ.copy()
    env["YETLET_SSE_LISTEN"] = f"127.0.0.1:{port}"

    with tempfile.TemporaryDirectory(prefix="yetlet-sse-") as directory:
        log_path = Path(directory) / "server.log"
        with log_path.open("w") as log:
            process = subprocess.Popen(
                [str(BINARY)], cwd=ROOT, env=env, stdout=log, stderr=subprocess.STDOUT
            )
            try:
                wait_ready(base, process)
                connection, response = open_stream(port)
                heartbeat = read_item(response)
                assert heartbeat == {"comment": "heartbeat"}, heartbeat
                wait_subscribers(base, 1)

                for message in ("one", "two", "three"):
                    status, _, body = call(base, "POST", "/notifications", {"message": message})
                    assert status == 201 and body["message"] == message
                live = [read_item(response) for _ in range(3)]
                assert [int(item["id"]) for item in live] == [1, 2, 3]
                assert all(item["event"] == "notification" for item in live)
                assert [json.loads(item["data"])["message"] for item in live] == ["one", "two", "three"]

                response.close()
                connection.close()
                wait_subscribers(base, 0)

                for message in ("four", "five"):
                    status, _, _ = call(base, "POST", "/notifications", {"message": message})
                    assert status == 201
                connection, response = open_stream(port, last_event_id=3)
                replay = [read_item(response) for _ in range(2)]
                assert [int(item["id"]) for item in replay] == [4, 5]
                assert [json.loads(item["data"])["message"] for item in replay] == ["four", "five"]
                response.close()
                connection.close()
                wait_subscribers(base, 0)

                status, _, body = call(base, "GET", "/events", headers={"Last-Event-ID": "bad"})
                assert status == 400 and body["error"]["code"] == "invalid_last_event_id"
                status, _, body = call(base, "GET", "/events", headers={"Last-Event-ID": "99"})
                assert status == 409 and body["error"]["code"] == "future_cursor"
                status, _, body = call(
                    base,
                    "POST",
                    "/notifications",
                    {"message": "browser blocked"},
                    {"Origin": "https://example.test"},
                )
                assert status == 403 and body["error"]["code"] == "browser_origin_forbidden"
                print("PASS: explicit SSE headers, heartbeat, ordered live notifications and JSON events")
                print("PASS: disconnect cleanup, Last-Event-ID replay, malformed/future cursor rejection")
                print("PASS: loopback-only command mutation policy and owned process cleanup")
            finally:
                if process.poll() is None:
                    process.send_signal(signal.SIGINT)
                    try:
                        process.wait(timeout=5)
                    except subprocess.TimeoutExpired:
                        process.kill()
                        process.wait(timeout=5)
                if process.returncode not in (0, -signal.SIGINT):
                    raise RuntimeError(log_path.read_text(errors="replace"))


if __name__ == "__main__":
    main()
