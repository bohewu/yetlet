#!/usr/bin/env python3
"""Measure reproducible *local* Yetlet MVP latency and asset-size baselines.

The measurements are informational, not a CI performance threshold.
The script uses only Python's standard library and the Chrome E2E helper.
It runs its own bound-localhost server and isolated headless Chrome profile.
"""
from __future__ import annotations

import argparse
from datetime import datetime, timezone
import gzip
import hashlib
import json
import math
import os
from pathlib import Path
import platform
import shutil
import socket
import statistics
import subprocess
import sys
import tempfile
import time

from smoke_browser import ChromeDevTools, http, port_number, wait_until

ROOT = Path(__file__).resolve().parent.parent
DIST = ROOT / "dist"
MARKERS = r"""
(() => {
  // CDP-injected measurement instrumentation, NOT Yetlet runtime code.
  const observer = new MutationObserver(records => {
    for (const record of records) {
      if (record.type !== "attributes") continue;
      if (record.attributeName === "data-yetlet-ready"
          && record.target.getAttribute("data-yetlet-ready") === "true") {
        performance.mark("yetlet:app-ready");
      }
      if (record.attributeName === "data-yetlet-activated"
          && record.target.getAttribute("data-yetlet-activated") === "true") {
        performance.mark("yetlet:island-ready");
      }
    }
  });
  observer.observe(document, {
    subtree: true,
    childList: true,
    attributes: true,
    attributeFilter: ["data-yetlet-ready", "data-yetlet-activated"],
  });
})();
"""


def version(command: list[str]) -> str:
    result = subprocess.run(command, capture_output=True, text=True, timeout=10)
    if result.returncode != 0:
        raise RuntimeError(f"Cannot determine version of {command[0]}")
    return (result.stdout.strip() or result.stderr.strip()).splitlines()[0]


def summary_ms(values: list[float]) -> dict[str, float | int]:
    if not values:
        raise RuntimeError("No measurements taken")
    numbers = sorted(values)
    return {
        "samples": len(numbers),
        "median_ms": round(statistics.median(numbers), 3),
        "p95_ms": round(numbers[math.ceil(len(numbers) * 0.95) - 1], 3),
        "min_ms": round(numbers[0], 3),
        "max_ms": round(numbers[-1], 3),
    }


def bytes_from_manifest(manifest: dict) -> dict[str, dict[str, object]]:
    digest = manifest.get("digest")
    if not isinstance(digest, str) or len(digest) != 16 or not all(
        c in "0123456789abcdef" for c in digest
    ):
        raise RuntimeError("Invalid asset manifest digest")
    assets: dict[str, dict[str, object]] = {}
    expected_prefix = f"/assets/{digest}/"
    for label in ("entry", "module", "wasm", "search_loader", "search_module",
                  "search_wasm", "stylesheet", "favicon"):
        value = manifest.get(label)
        if not isinstance(value, str) or not value.startswith(expected_prefix):
            raise RuntimeError(f"Missing/invalid manifest URL: {label}")
        name = value.removeprefix(expected_prefix)
        if "/" in name or name in (".", ".."):
            raise RuntimeError(f"Unsafe asset name for {label}")
        content = (DIST / "assets" / digest / name).read_bytes()
        assets[label] = {
            "name": name,
            "bytes": len(content),
            "gzip_bytes": len(gzip.compress(content, compresslevel=9, mtime=0)),
            "sha256": hashlib.sha256(content).hexdigest(),
        }
    return assets


def response_first_byte_ms(port: int, resource: str) -> float:
    # TCP connect happens before the clock starts; the timing measures one
    # request from send until first response byte (including HTTP headers).
    with socket.create_connection(("127.0.0.1", port), timeout=5) as conn:
        conn.settimeout(5)
        request = (
            f"GET {resource} HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\n"
            "Connection: close\r\nAccept: text/html\r\n\r\n"
        ).encode("ascii")
        start = time.perf_counter_ns()
        conn.sendall(request)
        head = conn.recv(1)
        elapsed = (time.perf_counter_ns() - start) / 1_000_000
        if head != b"H":
            raise RuntimeError("Server did not respond with an HTTP status line")
        return elapsed


def chrome_measurements(base: str, temp: Path, chrome: str, runs: int, log) -> dict:
    devtools_port = port_number()
    chrome_process = subprocess.Popen(
        [
            chrome, "--headless=new", "--no-sandbox", "--disable-gpu",
            "--disable-background-networking", "--no-first-run",
            "--disable-extensions", "--remote-debugging-address=127.0.0.1",
            "--remote-allow-origins=*",
            f"--remote-debugging-port={devtools_port}",
            f"--user-data-dir={temp / 'chrome-profile'}", "about:blank",
        ],
        stdout=log, stderr=subprocess.STDOUT,
    )
    browser = None
    try:
        def target_url():
            targets = json.loads(http(f"http://127.0.0.1:{devtools_port}/json/list")[2])
            return next((item["webSocketDebuggerUrl"] for item in targets
                         if item.get("type") == "page"), None)

        browser = ChromeDevTools(wait_until(target_url, "Chrome debugger"))
        for method in ("Runtime.enable", "Page.enable", "Network.enable", "Log.enable"):
            browser.call(method)
        browser.call("Network.setCacheDisabled", {"cacheDisabled": True})
        browser.call("Page.addScriptToEvaluateOnNewDocument", {"source": MARKERS})

        app: list[float] = []
        island: list[float] = []
        for _ in range(runs):
            browser.call("Page.navigate", {"url": base + "/app"})
            wait_until(
                lambda: browser.evaluate(
                    "document.documentElement.getAttribute('data-yetlet-ready') === 'true'"
                ),
                "client WASM mount",
            )
            elapsed = browser.evaluate(
                "performance.getEntriesByName('yetlet:app-ready').at(-1)?.startTime"
            )
            if not isinstance(elapsed, (int, float)) or elapsed < 0:
                raise RuntimeError("Browser app-ready performance mark was not observed")
            app.append(elapsed)

            before = len(browser.requests)
            browser.call("Page.navigate", {"url": base + "/search"})
            wait_until(
                lambda: browser.evaluate(
                    "document.querySelector('#search-island')"
                    "?.getAttribute('data-yetlet-loader-ready') === 'true'"
                ),
                "SSR island bootstrap",
            )
            if any("yetlet_search_bg.wasm" in url for url in browser.requests[before:]):
                raise RuntimeError("Search island loaded eagerly, before interaction")
            browser.evaluate(
                "(function(){"
                "performance.mark('yetlet:island-trigger');"
                "document.querySelector('#search-query').dispatchEvent("
                "new Event('input',{bubbles:true}));"
                "return true;"
                "})()"
            )
            wait_until(
                lambda: browser.evaluate(
                    "document.querySelector('#search-island')"
                    "?.getAttribute('data-yetlet-activated') === 'true'"
                ),
                "Search WASM island activation",
            )
            elapsed = browser.evaluate(
                "(function(){"
                "const start=performance.getEntriesByName('yetlet:island-trigger').at(-1);"
                "const ready=performance.getEntriesByName('yetlet:island-ready').at(-1);"
                "return start && ready ? ready.startTime-start.startTime : null;"
                "})()"
            )
            if not isinstance(elapsed, (int, float)) or elapsed < 0:
                raise RuntimeError("Browser island timing marks were missing")
            island.append(elapsed)
        if browser.websockets or browser.failed_responses or browser.exceptions:
            raise RuntimeError(
                "Browser reported WebSocket, failed resource or JavaScript exception"
            )
        return {
            "chrome_app_ready_navigation_ms": summary_ms(app),
            "chrome_search_island_trigger_to_ready_ms": summary_ms(island),
            "cache_disabled": True,
            "test_browser": "headless Chrome",
            "injected_instrumentation": True,
        }
    finally:
        if browser is not None:
            browser.close()
        if chrome_process.poll() is None:
            chrome_process.terminate()
            try:
                chrome_process.wait(timeout=5)
            except subprocess.TimeoutExpired:
                chrome_process.kill()
                chrome_process.wait(timeout=5)


def measure(profile: str, samples: int, browser_runs: int) -> dict:
    binary = ROOT / "target" / profile / "yetlet-todo-server"
    manifest_path = DIST / "manifest.json"
    chrome = shutil.which("google-chrome-stable") or shutil.which("google-chrome")
    if not binary.is_file():
        raise RuntimeError(f"Build {binary.relative_to(ROOT)} first")
    if not manifest_path.is_file():
        raise RuntimeError("Build browser assets first")
    if not chrome:
        raise RuntimeError("Headless Chrome is required")
    manifest = json.loads(manifest_path.read_text("utf-8"))
    if manifest.get("mode") != profile:
        raise RuntimeError(
            f"Manifest profile {manifest.get('mode')!r} differs from {profile!r}"
        )
    assets = bytes_from_manifest(manifest)
    with tempfile.TemporaryDirectory(prefix="yetlet-m5-bench-") as directory:
        temp = Path(directory)
        port = port_number()
        base = f"http://127.0.0.1:{port}"
        environment = dict(os.environ, YETLET_LISTEN=f"127.0.0.1:{port}")
        with (temp / "server.log").open("w") as server_log, (
            temp / "chrome.log"
        ).open("w") as browser_log:
            server = subprocess.Popen(
                [str(binary)], cwd=ROOT, env=environment,
                stdout=server_log, stderr=subprocess.STDOUT,
            )
            try:
                wait_until(lambda: http(base + "/healthz")[0] == 200, "Yetlet server")
                initial: dict[str, dict] = {}
                for route in ("/", "/search", "/app"):
                    status, headers, body = http(base + route)
                    if status != 200:
                        raise RuntimeError(f"{route} returned status {status}")
                    initial[route] = {
                        "html_bytes": len(body),
                        "status": status,
                        "has_browser_script": b'<script' in body,
                        "content_security_policy": headers.get("content-security-policy"),
                    }
                if initial["/"]["has_browser_script"]:
                    raise RuntimeError("Static SSR unexpectedly requires a browser script")
                for _ in range(5):
                    response_first_byte_ms(port, "/")
                first_bytes = [
                    response_first_byte_ms(port, "/") for _ in range(samples)
                ]
                browser = chrome_measurements(base, temp, chrome, browser_runs, browser_log)
            finally:
                if server.poll() is None:
                    server.terminate()
                    try:
                        server.wait(timeout=5)
                    except subprocess.TimeoutExpired:
                        server.kill()
                        server.wait(timeout=5)

    return {
        "schema_version": 1,
        "timestamp_utc": datetime.now(timezone.utc).isoformat(timespec="seconds"),
        "scope": "local_loopback_informational_not_a_performance_gate",
        "profile": profile,
        "environment": {
            "os": platform.system(),
            "architecture": platform.machine(),
            "rustc": version([str(Path.home() / ".cargo/bin/rustc"), "--version"]),
            "chrome": version([chrome, "--version"]),
            "python": platform.python_version(),
        },
        "asset_manifest_digest": manifest["digest"],
        "assets": assets,
        "asset_sets": {
            "full_browser_runtime_gzip_bytes": sum(
                assets[key]["gzip_bytes"] for key in ("entry", "module", "wasm")
            ),
            "island_initial_loader_gzip_bytes": assets["search_loader"]["gzip_bytes"],
            "island_activated_runtime_gzip_bytes": sum(
                assets[key]["gzip_bytes"] for key in ("search_module", "search_wasm")
            ),
        },
        "routes": initial,
        "latency": {
            "ssr_request_to_first_response_byte_ms": summary_ms(first_bytes),
            **browser,
        },
        "limitations": [
            "Loopback only, one host, one headless Chrome version; no WAN/TLS/reverse proxy.",
            "First-byte timing begins after TCP connect and includes HTTP header transmission.",
            "Gzip bytes are local compressed artifact sizes, not actual wire transfer sizes.",
            "Chrome instrumentation is injected via DevTools, not part of shipped runtime.",
            "Browser cache disabled; no cold-process restart or concurrent load benchmark.",
            "No comparison against other frameworks or production SLA implied.",
        ],
    }


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--profile", choices=("debug", "release"), default="release")
    parser.add_argument("--samples", type=int, default=30)
    parser.add_argument("--browser-runs", type=int, default=5)
    parser.add_argument("--out", type=Path, default=ROOT / "target/m5-baseline.json")
    args = parser.parse_args()
    if not 5 <= args.samples <= 500 or not 1 <= args.browser_runs <= 30:
        parser.error("samples must be 5..500 and browser-runs 1..30")
    report = measure(args.profile, args.samples, args.browser_runs)
    output = args.out
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n", "utf-8")
    print("PASS: bounded local M5 performance measurement")
    print("Profile:", report["profile"], "manifest:", report["asset_manifest_digest"])
    print("SSR request-to-first-byte:", report["latency"]["ssr_request_to_first_response_byte_ms"])
    print("Chrome app navigation-to-ready:", report["latency"]["chrome_app_ready_navigation_ms"])
    print("Chrome search trigger-to-ready:", report["latency"]["chrome_search_island_trigger_to_ready_ms"])
    print("Asset set gzip estimates (not wire bytes):", report["asset_sets"])
    print("Report:", output)
    return 0


if __name__ == "__main__":
    try:
        sys.exit(main())
    except (RuntimeError, OSError, subprocess.CalledProcessError, ValueError) as exc:
        print("M5 baseline FAILED:", exc, file=sys.stderr)
        sys.exit(1)
