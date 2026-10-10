#!/usr/bin/env python3
"""Bounded Chrome E2E for independent multi-island ownership and lifecycle."""
from __future__ import annotations

import argparse

import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import time

from smoke_browser import ChromeDevTools, http, port_number, wait_until

ROOT = Path(__file__).resolve().parent.parent
DEFAULT_CSP = "default-src 'none'; script-src 'self'; style-src 'self'; connect-src 'self'; img-src 'self'; base-uri 'none'; object-src 'none'; form-action 'self'; frame-ancestors 'none'"


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--profile", choices=("debug", "release"), default="debug")
    options = parser.parse_args()
    binary = ROOT / f"target/{options.profile}/yetlet-multi-island-dashboard"
    chrome = shutil.which("google-chrome-stable") or shutil.which("google-chrome")
    if not binary.is_file():
        raise RuntimeError("Build yetlet-multi-island-dashboard first")
    if not chrome:
        raise RuntimeError("Headless Chrome is required")

    with tempfile.TemporaryDirectory(prefix="yetlet-dashboard-chrome-") as folder:
        temp = Path(folder)
        server_port, chrome_port = port_number(), port_number()
        base = f"http://127.0.0.1:{server_port}"
        env = dict(
            os.environ,
            YETLET_DASHBOARD_LISTEN=f"127.0.0.1:{server_port}",
            YETLET_DASHBOARD_ASSET_ROOT=str(ROOT / "target/multi-island-dashboard-dist"),
        )
        with (temp / "server.log").open("w") as server_log, (
            temp / "chrome.log"
        ).open("w") as chrome_log:
            server = subprocess.Popen(
                [str(binary)], cwd=ROOT, env=env,
                stdout=server_log, stderr=subprocess.STDOUT,
            )
            browser = None
            client = None
            try:
                wait_until(lambda: http(base + "/healthz")[0] == 200, "dashboard server")
                status, headers, body = http(base + "/dashboard")
                assert status == 200
                html = body.decode()
                assert "Yetlet Multi-Island Dashboard" in html
                assert 'id="counter-island"' in html and 'id="filter-island"' in html
                assert "counter-loader.js" in html and "filter-loader.js" in html
                assert "'wasm-unsafe-eval'" in headers["content-security-policy"]
                assert "'unsafe-inline'" not in headers["content-security-policy"]

                browser = subprocess.Popen(
                    [
                        chrome,
                        "--headless=new", "--no-sandbox", "--disable-gpu",
                        "--disable-background-networking", "--no-first-run",
                        "--disable-extensions", "--remote-debugging-address=127.0.0.1",
                        "--remote-allow-origins=*", f"--remote-debugging-port={chrome_port}",
                        f"--user-data-dir={temp / 'chrome-profile'}", "about:blank",
                    ],
                    stdout=chrome_log, stderr=subprocess.STDOUT,
                )
                endpoint = f"http://127.0.0.1:{chrome_port}/json/list"

                def target():
                    for item in json.loads(http(endpoint)[2]):
                        if item.get("type") == "page":
                            return item["webSocketDebuggerUrl"]
                    return None

                client = ChromeDevTools(wait_until(target, "Chrome debugger"))
                for method in ("Runtime.enable", "Page.enable", "Network.enable", "Log.enable"):
                    client.call(method)
                request_start = len(client.requests)
                client.call("Page.navigate", {"url": base + "/dashboard"})
                wait_until(
                    lambda: client.evaluate(
                        "document.getElementById('counter-island')?.getAttribute('data-yetlet-loader-ready') === 'true' && "
                        "document.getElementById('filter-island')?.getAttribute('data-yetlet-loader-ready') === 'true'"
                    ),
                    "dashboard loaders",
                )
                initial = client.requests[request_start:]
                assert any("counter-loader.js" in url for url in initial)
                assert any("filter-loader.js" in url for url in initial)
                assert not any("yetlet_dashboard_counter.js" in url or "yetlet_dashboard_counter_bg.wasm" in url for url in initial)
                assert not any("yetlet_dashboard_filter.js" in url or "yetlet_dashboard_filter_bg.wasm" in url for url in initial)
                assert client.evaluate(
                    "(()=>{window.__counterRoot=document.getElementById('counter-island');"
                    "window.__counterValue=document.getElementById('counter-value');"
                    "window.__filterRoot=document.getElementById('filter-island');"
                    "window.__filterInput=document.getElementById('filter-query');return true;})()"
                ) is True

                counter_requests = len(client.requests)
                client.evaluate(
                    "document.getElementById('counter-island')"
                    ".dispatchEvent(new Event('yetlet-activate'))"
                )
                wait_until(
                    lambda: client.evaluate(
                        "document.getElementById('counter-island').getAttribute('data-yetlet-activated') === 'true'"
                    ),
                    "counter island activation",
                )
                loaded = client.requests[counter_requests:]
                assert any("yetlet_dashboard_counter.js" in url for url in loaded)
                assert any("yetlet_dashboard_counter_bg.wasm" in url for url in loaded)
                assert not any("yetlet_dashboard_filter.js" in url or "yetlet_dashboard_filter_bg.wasm" in url for url in loaded)
                assert client.evaluate(
                    "document.getElementById('filter-island').getAttribute('data-yetlet-activated')"
                ) is None
                client.evaluate("document.getElementById('counter-increment').click()")
                wait_until(lambda: client.evaluate("document.getElementById('counter-value').textContent === '3'"), "counter update")
                assert client.evaluate(
                    "document.getElementById('counter-island') === window.__counterRoot && "
                    "document.getElementById('counter-value') === window.__counterValue && "
                    "document.querySelectorAll('#filter-results li').length === 4"
                ) is True

                filter_requests = len(client.requests)
                client.evaluate(
                    "document.getElementById('filter-island')"
                    ".dispatchEvent(new Event('yetlet-activate'))"
                )
                wait_until(
                    lambda: client.evaluate(
                        "document.getElementById('filter-island').getAttribute('data-yetlet-activated') === 'true'"
                    ),
                    "filter island activation",
                )
                loaded = client.requests[filter_requests:]
                assert any("yetlet_dashboard_filter.js" in url for url in loaded)
                assert any("yetlet_dashboard_filter_bg.wasm" in url for url in loaded)
                client.evaluate("document.getElementById('filter-query').focus()")
                client.evaluate(
                    "(()=>{const input=document.getElementById('filter-query');"
                    "input.value='web';input.dispatchEvent(new Event('input',{bubbles:true}));return true;})()"
                )
                wait_until(
                    lambda: client.evaluate(
                        "document.getElementById('filter-count').textContent === '1 matching items'"
                    ),
                    "filter update",
                )
                assert client.evaluate(
                    "document.getElementById('filter-query') === window.__filterInput && "
                    "document.activeElement === window.__filterInput && "
                    "document.querySelector('#filter-results li').textContent === 'WebAssembly' && "
                    "document.getElementById('counter-value').textContent === '3'"
                ) is True

                client.evaluate(
                    "(()=>{window.__lateCounterBefore=document.getElementById('counter-value').textContent;"
                    "document.getElementById('counter-delayed').click();"
                    "window.__counterDisposed=false;"
                    "window.__yetletDashboard.counter.dispose().then(()=>window.__counterDisposed=true);return true;})()"
                )
                wait_until(lambda: client.evaluate("window.__counterDisposed === true"), "counter disposal")
                time.sleep(0.35)
                assert client.evaluate(
                    "document.getElementById('counter-value').textContent === window.__lateCounterBefore && "
                    "document.getElementById('counter-island').getAttribute('data-yetlet-disposed') === 'true' && "
                    "document.getElementById('filter-count').textContent === '1 matching items'"
                ) is True
                client.evaluate("document.getElementById('counter-increment').click()")
                time.sleep(0.15)
                assert client.evaluate("document.getElementById('counter-value').textContent") == "3"

                client.evaluate(
                    "window.__counterRemounted=false;"
                    "window.__yetletDashboard.counter.mount().then(()=>window.__counterRemounted=true)"
                )
                wait_until(lambda: client.evaluate("window.__counterRemounted === true"), "counter remount")
                assert client.evaluate(
                    "document.getElementById('counter-island') === window.__counterRoot && "
                    "document.getElementById('counter-value') === window.__counterValue"
                ) is True
                client.evaluate("document.getElementById('counter-increment').click()")
                wait_until(lambda: client.evaluate("document.getElementById('counter-value').textContent === '4'"), "single remounted listener")

                exception_count = len(client.exceptions)
                client.evaluate(
                    "(()=>{window.__filterFailed=false;"
                    "window.__yetletDashboard.filter.dispose().then(()=>{"
                    "const input=document.getElementById('filter-query');input.value='rust';"
                    "input.dispatchEvent(new Event('input',{bubbles:true}));"
                    "document.getElementById('filter-island').setAttribute('data-island-version','999');"
                    "return window.__yetletDashboard.filter.mount();"
                    "}).catch(()=>window.__filterFailed=true);return true;})()"
                )
                wait_until(lambda: client.evaluate("window.__filterFailed === true"), "invalid filter failure")
                assert client.evaluate(
                    "document.getElementById('filter-island').getAttribute('data-yetlet-error') === 'mount-failed' && "
                    "document.getElementById('filter-query') === window.__filterInput && "
                    "document.querySelector('#filter-results li').textContent === 'WebAssembly'"
                ) is True
                client.evaluate("document.getElementById('counter-increment').click()")
                wait_until(lambda: client.evaluate("document.getElementById('counter-value').textContent === '5'"), "counter after sibling failure")
                assert len(client.exceptions) >= exception_count

                client.evaluate(
                    "(()=>{const root=document.getElementById('filter-island');"
                    "root.setAttribute('data-island-version','1');window.__filterRecovered=false;"
                    "window.__yetletDashboard.filter.mount().then(()=>window.__filterRecovered=true);return true;})()"
                )
                wait_until(lambda: client.evaluate("window.__filterRecovered === true"), "filter recovery")
                assert client.evaluate(
                    "document.getElementById('filter-query') === window.__filterInput && "
                    "document.getElementById('filter-count').textContent === '1 matching items' && "
                    "document.querySelector('#filter-results li').textContent === 'Rust handlers' && "
                    "document.getElementById('counter-value').textContent === '5'"
                ) is True

                client.evaluate(
                    "window.__dashboardDisposed=false;Promise.all(["
                    "window.__yetletDashboard.counter.dispose(),window.__yetletDashboard.filter.dispose()"
                    "]).then(()=>window.__dashboardDisposed=true)"
                )
                wait_until(lambda: client.evaluate("window.__dashboardDisposed === true"), "dashboard disposal")
                assert client.evaluate(
                    "document.getElementById('counter-island').getAttribute('data-yetlet-disposed') === 'true' && "
                    "document.getElementById('filter-island').getAttribute('data-yetlet-disposed') === 'true'"
                ) is True

                def verify_fallback(case: str, malformed: bool) -> None:
                    asset_root = temp / f"{case}-assets"
                    asset_root.mkdir()
                    if malformed:
                        (asset_root / "manifest.json").write_text("{ malformed")
                    fallback_port = port_number()
                    fallback_base = f"http://127.0.0.1:{fallback_port}"
                    fallback_env = dict(
                        os.environ,
                        YETLET_DASHBOARD_LISTEN=f"127.0.0.1:{fallback_port}",
                        YETLET_DASHBOARD_ASSET_ROOT=str(asset_root),
                    )
                    with (temp / f"{case}-server.log").open("w") as fallback_log:
                        fallback = subprocess.Popen(
                            [str(binary)], cwd=ROOT, env=fallback_env,
                            stdout=fallback_log, stderr=subprocess.STDOUT,
                        )
                        try:
                            wait_until(
                                lambda: http(fallback_base + "/healthz")[0] == 200,
                                f"{case} fallback server",
                            )
                            status, headers, fallback_body = http(fallback_base + "/dashboard")
                            assert status == 200
                            assert headers["content-security-policy"] == DEFAULT_CSP
                            fallback_html = fallback_body.decode()
                            assert 'id="counter-island"' in fallback_html
                            assert 'id="filter-island"' in fallback_html
                            assert "<script" not in fallback_html
                            request_index = len(client.requests)
                            client.call("Page.navigate", {"url": fallback_base + "/dashboard"})
                            wait_until(
                                lambda: client.evaluate(
                                    "document.getElementById('counter-value')?.textContent === '2' && "
                                    "document.querySelectorAll('#filter-results li').length === 4"
                                ),
                                f"{case} browser SSR fallback",
                            )
                            assert client.evaluate(
                                "document.querySelectorAll('script').length === 0 && "
                                "globalThis.__yetletDashboard === undefined && "
                                "document.getElementById('counter-island') !== null && "
                                "document.getElementById('filter-island') !== null"
                            ) is True
                            requests = client.requests[request_index:]
                            assert not any(
                                "dashboard-assets" in url
                                or "dashboard_counter" in url
                                or "dashboard_filter" in url
                                or "loader.js" in url
                                for url in requests
                            ), requests
                        finally:
                            if fallback.poll() is None:
                                fallback.terminate()
                                try:
                                    fallback.wait(timeout=5)
                                except subprocess.TimeoutExpired:
                                    fallback.kill(); fallback.wait(timeout=5)

                verify_fallback("missing-manifest", False)
                verify_fallback("malformed-manifest", True)
                assert not client.websockets
                assert not client.failed_responses
                print("PASS Chrome: separate lazy bundle ownership and sibling request isolation")
                print("PASS Chrome: DOM identity, focus, dispose/remount and guarded late callback")
                print("PASS Chrome: failed island mount preserves sibling and supports recovery")
                print("PASS Chrome: missing and malformed manifests retain strict script-free SSR")
            finally:
                if client is not None:
                    client.close()
                if browser is not None and browser.poll() is None:
                    browser.terminate()
                    try:
                        browser.wait(timeout=5)
                    except subprocess.TimeoutExpired:
                        browser.kill(); browser.wait(timeout=5)
                if server.poll() is None:
                    server.terminate()
                    try:
                        server.wait(timeout=5)
                    except subprocess.TimeoutExpired:
                        server.kill(); server.wait(timeout=5)


if __name__ == "__main__":
    main()
