#!/usr/bin/env python3
"""Bounded Firefox WebDriver smoke using Python stdlib, without Selenium.

Requires geckodriver and Firefox. Starts an isolated browser session, an
ephemeral localhost server, and stops only the processes owned by this script.
"""
from __future__ import annotations

import argparse
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import time
from urllib.error import HTTPError, URLError
from urllib.request import ProxyHandler, Request, build_opener

from smoke_browser import http, port_number, wait_until

ROOT = Path(__file__).resolve().parent.parent
W3C_ELEMENT = "element-6066-11e4-a52e-4f735466cecf"


class FirefoxDriver:
    def __init__(self, port: int):
        self.base = f"http://127.0.0.1:{port}"
        self.opener = build_opener(ProxyHandler({}))
        self.session_id: str | None = None

    def call(self, path: str, method: str = "GET", body: dict | None = None):
        payload = json.dumps(body).encode("utf-8") if body is not None else None
        request = Request(
            self.base + path,
            data=payload,
            method=method,
            headers={"Content-Type": "application/json"},
        )
        try:
            with self.opener.open(request, timeout=15) as response:
                result = json.loads(response.read().decode("utf-8"))
        except HTTPError as error:
            data = json.loads(error.read().decode("utf-8"))
            detail = data.get("value", {})
            raise RuntimeError(
                f"Firefox WebDriver {method} failed: "
                f"{detail.get('error', 'HTTP error')}"
            ) from error
        if isinstance(result.get("value"), dict) and "error" in result["value"]:
            raise RuntimeError(f"Firefox WebDriver returned {result['value']['error']}")
        return result.get("value")

    def create_session(self) -> str:
        answer = self.call(
            "/session",
            "POST",
            {
                "capabilities": {
                    "alwaysMatch": {
                        "browserName": "firefox",
                        "moz:firefoxOptions": {"args": ["-headless"]},
                    }
                }
            },
        )
        self.session_id = answer["sessionId"]
        return str(answer["capabilities"]["browserVersion"])

    def close(self) -> None:
        if self.session_id is not None:
            try:
                self.call(f"/session/{self.session_id}", "DELETE")
            except (OSError, RuntimeError):
                pass
            self.session_id = None

    def command(self, suffix: str, method: str = "GET", body: dict | None = None):
        if self.session_id is None:
            raise RuntimeError("Firefox WebDriver has no active session")
        return self.call(f"/session/{self.session_id}{suffix}", method, body)

    def navigate(self, url: str) -> None:
        self.command("/url", "POST", {"url": url})

    def evaluate(self, expression: str):
        return self.command(
            "/execute/sync",
            "POST",
            {"script": f"return ({expression});", "args": []},
        )

    def element(self, selector: str) -> str:
        value = self.command(
            "/element", "POST", {"using": "css selector", "value": selector}
        )
        return value[W3C_ELEMENT]

    def click(self, selector: str) -> None:
        self.command(f"/element/{self.element(selector)}/click", "POST", {})

    def type(self, selector: str, value: str) -> None:
        self.command(
            f"/element/{self.element(selector)}/value",
            "POST",
            {"text": value},
        )

    def clear(self, selector: str) -> None:
        self.command(f"/element/{self.element(selector)}/clear", "POST", {})


def await_value(predicate, description: str, seconds: float = 15) -> None:
    end = time.monotonic() + seconds
    while time.monotonic() < end:
        try:
            if predicate():
                return
        except (URLError, RuntimeError):
            pass
        time.sleep(0.12)
    raise RuntimeError(f"Firefox failed to reach state: {description}")


def smoke(profile: str) -> str:
    binary = ROOT / "target" / profile / "yetlet-todo-server"
    driver_exe = shutil.which("geckodriver")
    if not driver_exe and Path("/snap/bin/geckodriver").is_file():
        driver_exe = "/snap/bin/geckodriver"
    if not driver_exe or not shutil.which("firefox"):
        raise RuntimeError("Firefox and geckodriver are required")
    if not binary.is_file():
        raise RuntimeError(f"Build {binary.relative_to(ROOT)} first")

    with tempfile.TemporaryDirectory(prefix="yetlet-firefox-") as folder:
        temp = Path(folder)
        server_port, driver_port = port_number(), port_number()
        base = f"http://127.0.0.1:{server_port}"
        environment = dict(os.environ, YETLET_LISTEN=f"127.0.0.1:{server_port}")
        with (temp / "server.log").open("w") as server_log, (
            temp / "geckodriver.log"
        ).open("w") as driver_log:
            server = subprocess.Popen(
                [str(binary)], cwd=ROOT, env=environment,
                stdout=server_log, stderr=subprocess.STDOUT,
            )
            driver_process = None
            driver = FirefoxDriver(driver_port)
            try:
                wait_until(lambda: http(base + "/healthz")[0] == 200, "Yetlet server")
                response, headers, html = http(base + "/")
                assert response == 200
                assert b"<script" not in html and b".wasm" not in html
                assert "unsafe-inline" not in headers["content-security-policy"]

                driver_process = subprocess.Popen(
                    [driver_exe, "--host", "127.0.0.1", "--port", str(driver_port)],
                    stdout=driver_log, stderr=subprocess.STDOUT,
                )
                await_value(
                    lambda: driver.call("/status")["ready"] is True,
                    "geckodriver readiness",
                )
                version = driver.create_session()

                driver.navigate(base + "/app")
                await_value(
                    lambda: driver.evaluate(
                        "document.documentElement.getAttribute('data-yetlet-ready') === 'true'"
                    ),
                    "app WASM mount",
                )
                assert driver.evaluate(
                    "document.querySelectorAll('#todo-list-host li').length"
                ) == 2
                driver.click("#toggle-first")
                await_value(
                    lambda: driver.evaluate(
                        "document.querySelector('#todo-list-host [data-id=\"1\"]')"
                        ".getAttribute('data-completed') === 'true'"
                    ),
                    "first local action",
                )
                driver.click("#toggle-first")
                await_value(
                    lambda: driver.evaluate(
                        "document.querySelector('#todo-list-host [data-id=\"1\"]')"
                        ".getAttribute('data-completed') === 'false'"
                    ),
                    "second local action",
                )
                _, _, response = http(base + "/api/todos")
                assert json.loads(response)["revision"] == 0
                assert driver.evaluate(
                    "performance.getEntriesByType('resource')"
                    ".filter(e => e.name.endsWith('/api/todos')).length"
                ) == 0

                driver.evaluate(
                    "(document.getElementById('remote-title').value='Firefox saved task')"
                )
                driver.click("#create-remote")
                await_value(
                    lambda: driver.evaluate(
                        "document.getElementById('remote-status').textContent"
                    ) == "Saved on server (revision 1)",
                    "remote typed POST",
                )
                _, _, response = http(base + "/api/todos")
                assert json.loads(response)["items"][-1]["title"] == "Firefox saved task"
                driver.evaluate("(document.getElementById('remote-title').value='  ')")
                driver.click("#create-remote")
                await_value(
                    lambda: driver.evaluate(
                        "document.getElementById('remote-status').textContent"
                    ) == "Remote error: invalid_title",
                    "typed 422 error",
                )
                _, _, response = http(base + "/api/todos")
                assert json.loads(response)["revision"] == 1

                driver.navigate(base + "/search")
                await_value(
                    lambda: driver.evaluate(
                        "document.querySelector('#search-island')"
                        ".getAttribute('data-yetlet-loader-ready') === 'true'"
                    ),
                    "lazy search loader",
                )
                assert driver.evaluate(
                    "document.querySelector('#search-island')"
                    ".getAttribute('data-yetlet-activated')"
                ) is None
                assert driver.evaluate(
                    "performance.getEntriesByType('resource')"
                    ".filter(e => e.name.includes('yetlet_search_bg.wasm')).length"
                ) == 0
                driver.type("#search-query", "ReNdErErS")
                await_value(
                    lambda: driver.evaluate(
                        "document.querySelector('#search-island')"
                        ".getAttribute('data-yetlet-activated') === 'true'"
                        " && document.querySelectorAll('#search-results li').length === 1"
                    ),
                    "physical keyboard input to lazy WASM island",
                )
                assert driver.evaluate(
                    "document.querySelector('#search-results [data-id=\"1\"]').textContent"
                ) == "Share one Rust View across renderers"
                assert driver.evaluate(
                    "performance.getEntriesByType('resource')"
                    ".filter(e => e.name.includes('yetlet_search_bg.wasm')).length"
                ) >= 1
                assert driver.evaluate(
                    "performance.getEntriesByType('resource')"
                    ".filter(e => e.name.endsWith('/api/todos')).length"
                ) == 0
                # WebDriver Element.clear() does not always emit an input event.
                # Use physical Ctrl+A and Backspace to test real user editing.
                driver.type("#search-query", "\ue009a\ue003")
                await_value(
                    lambda: driver.evaluate(
                        "document.getElementById('search-query').value === ''"
                    ),
                    "native keyboard clearing",
                )
                await_value(
                    lambda: driver.evaluate(
                        "document.querySelectorAll('#search-results li').length === 2"
                    ),
                    "search clear restores SSR list",
                )
                print("PASS: Firefox SSR, Browser WASM, two native local clicks")
                print("PASS: typed HTTP mutation + typed 422 error, local actions do not use API")
                print("PASS: real keyboard input activates lazy Search Island only on demand")
                print("PASS: search results reset; no input-triggered API requests")
                return version
            finally:
                driver.close()
                for child in (driver_process, server):
                    if child is not None and child.poll() is None:
                        child.terminate()
                        try:
                            child.wait(timeout=6)
                        except subprocess.TimeoutExpired:
                            child.kill()
                            child.wait(timeout=6)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--profile", choices=("debug", "release"), default="release")
    args = parser.parse_args()
    try:
        version = smoke(args.profile)
    except (OSError, RuntimeError, AssertionError, subprocess.CalledProcessError) as error:
        print("Firefox E2E FAILED:", type(error).__name__, str(error)[:280], file=sys.stderr)
        return 1
    print(f"Firefox {version} E2E: PASS ({args.profile})")
    return 0


if __name__ == "__main__":
    sys.exit(main())
