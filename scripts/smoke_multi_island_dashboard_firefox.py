#!/usr/bin/env python3
"""Bounded Firefox E2E for separate multi-island bundles and lifecycle."""
from __future__ import annotations

import argparse

import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import time

from smoke_browser import http, port_number, wait_until
from smoke_firefox import FirefoxDriver, await_value

ROOT = Path(__file__).resolve().parent.parent
DEFAULT_CSP = "default-src 'none'; script-src 'self'; style-src 'self'; connect-src 'self'; img-src 'self'; base-uri 'none'; object-src 'none'; form-action 'self'; frame-ancestors 'none'"


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--profile", choices=("debug", "release"), default="debug")
    options = parser.parse_args()
    binary = ROOT / f"target/{options.profile}/yetlet-multi-island-dashboard"
    driver_exe = shutil.which("geckodriver") or (
        "/snap/bin/geckodriver" if Path("/snap/bin/geckodriver").is_file() else None
    )
    if not binary.is_file() or not driver_exe or not shutil.which("firefox"):
        raise RuntimeError("Dashboard binary, Firefox and geckodriver are required")

    with tempfile.TemporaryDirectory(prefix="yetlet-dashboard-firefox-") as folder:
        temp = Path(folder)
        server_port, driver_port = port_number(), port_number()
        base = f"http://127.0.0.1:{server_port}"
        env = dict(
            os.environ,
            YETLET_DASHBOARD_LISTEN=f"127.0.0.1:{server_port}",
            YETLET_DASHBOARD_ASSET_ROOT=str(ROOT / "target/multi-island-dashboard-dist"),
        )
        with (temp / "server.log").open("w") as server_log, (
            temp / "geckodriver.log"
        ).open("w") as driver_log:
            server = subprocess.Popen(
                [str(binary)], cwd=ROOT, env=env,
                stdout=server_log, stderr=subprocess.STDOUT,
            )
            driver_process = None
            driver = FirefoxDriver(driver_port)
            try:
                wait_until(lambda: http(base + "/healthz")[0] == 200, "dashboard server")
                driver_process = subprocess.Popen(
                    [driver_exe, "--host", "127.0.0.1", "--port", str(driver_port)],
                    stdout=driver_log, stderr=subprocess.STDOUT,
                )
                await_value(lambda: driver.call("/status")["ready"] is True, "geckodriver")
                version = driver.create_session()
                driver.navigate(base + "/dashboard")
                await_value(
                    lambda: driver.evaluate(
                        "document.getElementById('counter-island')?.getAttribute('data-yetlet-loader-ready') === 'true' && "
                        "document.getElementById('filter-island')?.getAttribute('data-yetlet-loader-ready') === 'true'"
                    ),
                    "dashboard loaders",
                )
                assert driver.evaluate(
                    "performance.getEntriesByType('resource').every(e=>!e.name.includes('yetlet_dashboard_counter.js') && !e.name.includes('yetlet_dashboard_filter.js'))"
                ) is True
                assert driver.evaluate(
                    "(()=>{window.__counterRoot=document.getElementById('counter-island');"
                    "window.__counterValue=document.getElementById('counter-value');"
                    "window.__filterRoot=document.getElementById('filter-island');"
                    "window.__filterInput=document.getElementById('filter-query');return true;})()"
                ) is True

                driver.evaluate(
                    "document.getElementById('counter-island')"
                    ".dispatchEvent(new Event('yetlet-activate'))"
                )
                await_value(
                    lambda: driver.evaluate(
                        "document.getElementById('counter-island').getAttribute('data-yetlet-activated') === 'true'"
                    ),
                    "counter activation",
                )
                assert driver.evaluate(
                    "performance.getEntriesByType('resource').some(e=>e.name.includes('yetlet_dashboard_counter_bg.wasm')) && "
                    "performance.getEntriesByType('resource').every(e=>!e.name.includes('yetlet_dashboard_filter_bg.wasm'))"
                ) is True
                driver.click("#counter-increment")
                await_value(lambda: driver.evaluate("document.getElementById('counter-value').textContent === '3'"), "counter update")

                driver.evaluate(
                    "document.getElementById('filter-island')"
                    ".dispatchEvent(new Event('yetlet-activate'))"
                )
                await_value(
                    lambda: driver.evaluate(
                        "document.getElementById('filter-island').getAttribute('data-yetlet-activated') === 'true'"
                    ),
                    "filter activation",
                )
                driver.evaluate("document.getElementById('filter-query').focus()")
                driver.evaluate(
                    "(()=>{const input=document.getElementById('filter-query');input.value='web';"
                    "input.dispatchEvent(new Event('input',{bubbles:true}));return true;})()"
                )
                await_value(lambda: driver.evaluate("document.getElementById('filter-count').textContent === '1 matching items'"), "filter update")
                assert driver.evaluate(
                    "document.getElementById('filter-query') === window.__filterInput && "
                    "document.activeElement === window.__filterInput && "
                    "document.querySelector('#filter-results li').textContent === 'WebAssembly' && "
                    "document.getElementById('counter-value').textContent === '3'"
                ) is True

                assert driver.evaluate(
                    "(()=>{window.__beforeLate=document.getElementById('counter-value').textContent;"
                    "document.getElementById('counter-delayed').click();window.__disposed=false;"
                    "window.__yetletDashboard.counter.dispose().then(()=>window.__disposed=true);return true;})()"
                ) is True
                await_value(lambda: driver.evaluate("window.__disposed === true"), "counter disposal")
                time.sleep(0.35)
                assert driver.evaluate(
                    "document.getElementById('counter-value').textContent === window.__beforeLate && "
                    "document.getElementById('filter-count').textContent === '1 matching items'"
                ) is True
                driver.evaluate(
                    "(()=>{window.__remounted=false;"
                    "window.__yetletDashboard.counter.mount().then(()=>window.__remounted=true);"
                    "return true;})()"
                )
                await_value(lambda: driver.evaluate("window.__remounted === true"), "counter remount")
                driver.click("#counter-increment")
                await_value(lambda: driver.evaluate("document.getElementById('counter-value').textContent === '4'"), "single remounted listener")

                driver.evaluate(
                    "(()=>{window.__filterFailed=false;window.__yetletDashboard.filter.dispose().then(()=>{"
                    "const input=document.getElementById('filter-query');input.value='rust';"
                    "input.dispatchEvent(new Event('input',{bubbles:true}));"
                    "document.getElementById('filter-island').setAttribute('data-island-version','999');"
                    "return window.__yetletDashboard.filter.mount();}).catch(()=>window.__filterFailed=true);return true;})()"
                )
                await_value(lambda: driver.evaluate("window.__filterFailed === true"), "invalid filter failure")
                assert driver.evaluate(
                    "document.getElementById('filter-query') === window.__filterInput && "
                    "document.querySelector('#filter-results li').textContent === 'WebAssembly'"
                ) is True
                driver.click("#counter-increment")
                await_value(lambda: driver.evaluate("document.getElementById('counter-value').textContent === '5'"), "counter after filter failure")

                driver.evaluate(
                    "(()=>{document.getElementById('filter-island').setAttribute('data-island-version','1');"
                    "window.__recovered=false;window.__yetletDashboard.filter.mount().then(()=>window.__recovered=true);return true;})()"
                )
                await_value(lambda: driver.evaluate("window.__recovered === true"), "filter recovery")
                assert driver.evaluate(
                    "document.getElementById('filter-query') === window.__filterInput && "
                    "document.querySelector('#filter-results li').textContent === 'Rust handlers' && "
                    "document.getElementById('counter-value').textContent === '5'"
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
                            driver.navigate(fallback_base + "/dashboard")
                            await_value(
                                lambda: driver.evaluate(
                                    "document.getElementById('counter-value')?.textContent === '2' && "
                                    "document.querySelectorAll('#filter-results li').length === 4"
                                ),
                                f"{case} browser SSR fallback",
                            )
                            assert driver.evaluate(
                                "document.querySelectorAll('script').length === 0 && "
                                "globalThis.__yetletDashboard === undefined && "
                                "document.getElementById('counter-island') !== null && "
                                "document.getElementById('filter-island') !== null && "
                                "performance.getEntriesByType('resource').every(e=>!e.name.includes('dashboard-assets') && !e.name.includes('loader.js'))"
                            ) is True
                        finally:
                            if fallback.poll() is None:
                                fallback.terminate()
                                try:
                                    fallback.wait(timeout=5)
                                except subprocess.TimeoutExpired:
                                    fallback.kill(); fallback.wait(timeout=5)

                verify_fallback("missing-manifest", False)
                verify_fallback("malformed-manifest", True)
                print(f"PASS Firefox {version}: separate bundle activation and sibling isolation")
                print("PASS Firefox: input focus/identity, late callback, dispose/remount and recovery")
                print("PASS Firefox: missing and malformed manifests retain strict script-free SSR")
            finally:
                driver.close()
                if driver_process is not None and driver_process.poll() is None:
                    driver_process.terminate()
                    try:
                        driver_process.wait(timeout=5)
                    except subprocess.TimeoutExpired:
                        driver_process.kill(); driver_process.wait(timeout=5)
                if server.poll() is None:
                    server.terminate()
                    try:
                        server.wait(timeout=5)
                    except subprocess.TimeoutExpired:
                        server.kill(); server.wait(timeout=5)


if __name__ == "__main__":
    main()
