#!/usr/bin/env python3
"""Firefox E2E for the isolated M7d keyed lifecycle and form-state prototype."""
from __future__ import annotations

import argparse
import json
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import time

from smoke_browser import port_number
from smoke_firefox import FirefoxDriver, await_value
from smoke_keyed_bindings import TARGET, static_server


def run(profile: str) -> str:
    root = TARGET / profile
    manifest = json.loads((root / "manifest.json").read_text(encoding="utf-8"))
    if manifest.get("profile") != profile:
        raise RuntimeError("M7d Firefox manifest/profile mismatch")
    driver_exe = shutil.which("geckodriver")
    if not driver_exe and Path("/snap/bin/geckodriver").is_file():
        driver_exe = "/snap/bin/geckodriver"
    if not driver_exe or not shutil.which("firefox"):
        raise RuntimeError("Firefox and geckodriver are required")

    with tempfile.TemporaryDirectory(prefix="yetlet-m7d-firefox-") as folder, static_server(
        root
    ) as base:
        temp = Path(folder)
        driver_port = port_number()
        process = None
        driver = FirefoxDriver(driver_port)
        with (temp / "geckodriver.log").open("w") as log:
            try:
                process = subprocess.Popen(
                    [driver_exe, "--host", "127.0.0.1", "--port", str(driver_port)],
                    stdout=log,
                    stderr=subprocess.STDOUT,
                )
                await_value(
                    lambda: driver.call("/status")["ready"] is True,
                    "M7d geckodriver readiness",
                )
                version = driver.create_session()
                driver.navigate(base + "/index.html")
                await_value(
                    lambda: driver.evaluate(
                        "document.documentElement.getAttribute('data-yetlet-m7d-ready') === 'true'"
                    ),
                    "M7d module readiness",
                )
                driver.evaluate("window.yetletBindings.mount_demo()")
                await_value(
                    lambda: driver.evaluate(
                        "document.querySelector('[data-yetlet-component]')"
                        ".getAttribute('data-yetlet-bound') === 'true'"
                    ),
                    "M7d keyed binding",
                )

                input_selector = '[data-yetlet-key="2"] [data-yetlet-input="draft"]'
                driver.evaluate(
                    "(()=>{"
                    "window.__m7dFirefoxInput2=document.querySelector('[data-yetlet-key=\"2\"] [data-yetlet-input=\"draft\"]');"
                    "return true"
                    "})()"
                )
                driver.click(input_selector)
                driver.type(input_selector, "\ue009a\ue003")
                driver.type(input_selector, "Firefox draft two")
                await_value(
                    lambda: driver.evaluate(
                        "document.querySelector('[data-yetlet-key=\"2\"]')"
                        ".getAttribute('data-draft') === 'Firefox draft two'"
                    ),
                    "Firefox controlled input update",
                )
                driver.evaluate(
                    "(()=>{window.__m7dFirefoxInput2.setSelectionRange(8,13);return true})()"
                )
                assert driver.evaluate(
                    "document.activeElement === window.__m7dFirefoxInput2"
                )

                driver.evaluate("window.yetletBindings.reverse_demo()")
                await_value(
                    lambda: driver.evaluate(
                        "Array.from(document.querySelectorAll('[data-yetlet-key]'))"
                        ".map(x=>x.getAttribute('data-yetlet-key')).join(',') === '3,2,1'"
                    ),
                    "Firefox keyed reverse",
                )
                assert driver.evaluate(
                    "window.__m7dFirefoxInput2.isSameNode(document.querySelector('[data-yetlet-key=\"2\"] [data-yetlet-input=\"draft\"]'))"
                )
                assert driver.evaluate(
                    "document.activeElement === window.__m7dFirefoxInput2"
                )
                assert driver.evaluate("window.__m7dFirefoxInput2.value") == "Firefox draft two"
                assert driver.evaluate("window.__m7dFirefoxInput2.selectionStart") == 8
                assert driver.evaluate("window.__m7dFirefoxInput2.selectionEnd") == 13

                driver.evaluate("window.yetletBindings.append_demo()")
                driver.evaluate(
                    "(()=>{"
                    "window.__m7dFirefoxRemovedInput=document.querySelector('[data-yetlet-key=\"1\"] [data-yetlet-input=\"draft\"]');"
                    "window.__m7dFirefoxRemovedButton=document.querySelector('[data-yetlet-key=\"1\"] [data-yetlet-action=\"row\"]');"
                    "window.__m7dFirefoxRemovedInput.focus();"
                    "window.__m7dFirefoxRemovedInput.setSelectionRange(1,5);"
                    "window.__m7dFirefoxBeforeRemove=Number(document.documentElement.getAttribute('data-yetlet-m7c-commit-count'));"
                    "return true"
                    "})()"
                )
                assert driver.evaluate(
                    "document.activeElement === window.__m7dFirefoxRemovedInput"
                )
                driver.evaluate("window.yetletBindings.remove_first_demo()")
                await_value(
                    lambda: not driver.evaluate("window.__m7dFirefoxRemovedInput.isConnected"),
                    "Firefox focused keyed row removal",
                )
                assert not driver.evaluate(
                    "document.activeElement === window.__m7dFirefoxRemovedInput"
                )
                assert driver.evaluate(
                    "Number(document.documentElement.getAttribute('data-yetlet-m7c-commit-count'))"
                ) == driver.evaluate("window.__m7dFirefoxBeforeRemove + 1")
                driver.evaluate(
                    "(()=>{"
                    "window.__m7dFirefoxAfterRemove=Number(document.documentElement.getAttribute('data-yetlet-m7c-commit-count'));"
                    "window.__m7dFirefoxRemovedInput.value='detached';"
                    "window.__m7dFirefoxRemovedInput.dispatchEvent(new Event('input',{bubbles:true}));"
                    "window.__m7dFirefoxRemovedButton.click();"
                    "return true"
                    "})()"
                )
                time.sleep(0.15)
                assert driver.evaluate(
                    "Number(document.documentElement.getAttribute('data-yetlet-m7c-commit-count'))"
                ) == driver.evaluate("window.__m7dFirefoxAfterRemove")
                assert driver.evaluate(
                    "document.documentElement.getAttribute('data-yetlet-m7d-event-error')"
                ) is None

                driver.evaluate("window.yetletBindings.reset_demo()")
                driver.evaluate(
                    "(()=>{"
                    "window.__m7dFirefoxDisposedInput=document.querySelector('[data-yetlet-key=\"2\"] [data-yetlet-input=\"draft\"]');"
                    "window.__m7dFirefoxDisposedValue=window.__m7dFirefoxDisposedInput.getAttribute('value');"
                    "return true"
                    "})()"
                )
                driver.evaluate("window.yetletBindings.dispose_demo()")
                assert driver.evaluate(
                    "document.querySelector('[data-yetlet-component]').getAttribute('data-yetlet-bound')"
                ) is None
                driver.evaluate(
                    "(()=>{"
                    "window.__m7dFirefoxDisposedInput.value='after dispose';"
                    "window.__m7dFirefoxDisposedInput.dispatchEvent(new Event('input',{bubbles:true}));"
                    "return true"
                    "})()"
                )
                time.sleep(0.15)
                assert driver.evaluate(
                    "window.__m7dFirefoxDisposedInput.getAttribute('value')"
                ) == driver.evaluate("window.__m7dFirefoxDisposedValue")
                assert driver.evaluate(
                    "document.documentElement.getAttribute('data-yetlet-m7d-event-error')"
                ) is None
                assert driver.evaluate(
                    "performance.getEntriesByType('resource')"
                    ".filter(entry => entry.name.includes('/api/')).length"
                ) == 0

                print("PASS: Firefox controlled input dispatch updates only local keyed state")
                print("PASS: Firefox keyed reorder preserves input node, focus, value and selection")
                print("PASS: Firefox focused removal and disposal detach input/action listeners")
                print("PASS: Firefox M7d fixture issues no API request or remote event circuit")
                print("M7b/M7c/M7d Firefox artifact digest:", manifest["digest"])
                return version
            finally:
                driver.close()
                if process is not None and process.poll() is None:
                    process.terminate()
                    try:
                        process.wait(timeout=6)
                    except subprocess.TimeoutExpired:
                        process.kill()
                        process.wait(timeout=6)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--profile", choices=("debug", "release"), default="release")
    args = parser.parse_args()
    try:
        version = run(args.profile)
    except (AssertionError, OSError, RuntimeError, subprocess.CalledProcessError) as error:
        print(f"M7d Firefox E2E FAILED: {error!r}", file=sys.stderr)
        return 1
    print(f"Firefox {version} M7d E2E: PASS ({args.profile})")
    return 0


if __name__ == "__main__":
    sys.exit(main())
