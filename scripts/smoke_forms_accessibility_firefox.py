#!/usr/bin/env python3
"""Bounded headless POSIX Firefox forms keyboard checks; no IME/AT certification.

Requires a prebuilt forms binary, Firefox and geckodriver on PATH. Writes only
target/qualification/forms-accessibility-firefox.json and owned temporary files
beneath target/qualification. Raw process output and exception messages are not
retained; failure diagnostics contain only fixed stages, types and exit codes.
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

from smoke_browser import port_number, remove_owned_tree, stop_owned_process
from smoke_firefox import FirefoxDriver, await_value
from smoke_forms_validation import call

ROOT = Path(__file__).resolve().parent.parent
REPORT = ROOT / "target/qualification/forms-accessibility-firefox.json"
TAB, ENTER, CONTROL, BACKSPACE = "\ue004", "\ue007", "\ue009", "\ue003"


def keys(driver: FirefoxDriver, text: str) -> None:
    """Use standard W3C key actions against the currently focused element."""
    actions = []
    for key in text:
        actions.extend(({"type": "keyDown", "value": key},
                        {"type": "keyUp", "value": key}))
    driver.command("/actions", "POST", {"actions": [
        {"type": "key", "id": "forms-keyboard", "actions": actions}
    ]})


def replace_text(driver: FirefoxDriver, text: str) -> None:
    driver.command("/actions", "POST", {"actions": [{
        "type": "key", "id": "forms-keyboard", "actions": [
            {"type": "keyDown", "value": CONTROL},
            {"type": "keyDown", "value": "a"},
            {"type": "keyUp", "value": "a"},
            {"type": "keyUp", "value": CONTROL},
        ]
    }]})
    keys(driver, BACKSPACE + text)


def focus_is(driver: FirefoxDriver, selector: str) -> None:
    await_value(lambda: driver.evaluate(
        f"document.activeElement === document.querySelector({json.dumps(selector)})"
    ), "keyboard focus")


def english_document(driver: FirefoxDriver) -> None:
    assert driver.evaluate("document.documentElement.lang") == "en"
    assert driver.evaluate("document.querySelectorAll('script').length") == 0


def labels(driver: FirefoxDriver) -> None:
    for field, label in (("name", "Your name"), ("email", "Email address")):
        assert driver.evaluate(
            f"document.querySelector('label[for={field}]').textContent"
        ) == label
        assert driver.evaluate(
            f"document.querySelector('label[for={field}]').control === "
            f"document.getElementById('{field}')"
        ) is True


def exercise(driver: FirefoxDriver, base: str, report: dict) -> None:
    report["stage"] = "initial_document"
    driver.navigate(base + "/form")
    english_document(driver)
    labels(driver)
    assert driver.evaluate("document.querySelector('form').noValidate") is True
    report["checks"].append("initial_language_labels_no_script")

    report["stage"] = "tab_order"
    keys(driver, TAB)
    focus_is(driver, "#name")
    keys(driver, "x" + TAB)
    focus_is(driver, "#email")
    keys(driver, "invalid" + TAB)
    focus_is(driver, "button[type=submit]")
    report["checks"].append("tab_name_email_submit")

    report["stage"] = "invalid_post"
    keys(driver, ENTER)
    await_value(lambda: driver.command("/url").endswith("/form/submit"), "invalid POST")
    focus_is(driver, "#name")
    english_document(driver)
    labels(driver)
    for field, message in (
        ("name", "Enter a name between 2 and 40 characters."),
        ("email", "Enter a valid example email address."),
    ):
        assert driver.evaluate(
            f"document.getElementById('{field}').getAttribute('aria-invalid')"
        ) == "true"
        assert driver.evaluate(
            f"document.getElementById('{field}').getAttribute('aria-describedby')"
        ) == f"{field}-error"
        assert driver.evaluate(
            f"document.getElementById('{field}-error').textContent"
        ) == message
        assert driver.evaluate(
            f"document.getElementById('{field}-error').getAttribute('role')"
        ) == "alert"
    assert driver.evaluate("document.querySelectorAll('[autofocus]').length") == 1
    report["checks"].append("invalid_first_error_focus_language_labels_descriptions")

    report["stage"] = "unicode_and_email_focus"
    name = "Zoé 漢字"
    replace_text(driver, name)
    assert driver.evaluate("document.getElementById('name').value") == name
    keys(driver, TAB + TAB + ENTER)
    focus_is(driver, "#email")
    english_document(driver)
    assert driver.evaluate("document.getElementById('name').value") == name
    assert driver.evaluate("document.getElementById('name').hasAttribute('aria-invalid')") is False
    assert driver.evaluate("document.querySelectorAll('[autofocus]').length") == 1
    report["checks"].append("unicode_key_input_preserved_email_first_error_focus")

    report["stage"] = "success_prg"
    replace_text(driver, "unicode@example.test")
    keys(driver, TAB + ENTER)
    await_value(lambda: driver.command("/url") == base + "/form/success", "success GET")
    english_document(driver)
    assert driver.evaluate("document.querySelector('h1').textContent") == "Form accepted"
    assert driver.evaluate("document.querySelector('main p').textContent") == "No data was saved."
    assert name not in driver.evaluate("document.body.textContent")
    assert driver.command("/cookie") == []
    assert driver.evaluate("localStorage.length + sessionStorage.length") == 0
    driver.command("/refresh", "POST", {})
    assert driver.command("/url") == base + "/form/success"
    english_document(driver)
    # Observe the HTTP 303 contract separately; WebDriver follows redirects.
    status, headers, _ = call(base, "/form/submit", {
        "name": name, "email": "unicode@example.test"
    })
    assert status == 303 and headers["location"] == "/form/success"
    assert headers["cache-control"] == "no-store" and "set-cookie" not in headers
    driver.navigate(base + "/form")
    assert driver.evaluate("document.getElementById('name').value") == ""
    assert driver.evaluate("document.getElementById('email').value") == ""
    report["checks"].append("success_language_no_state_303_refresh_empty_form")


def run(profile: str) -> dict:
    report = {
        "task_id": "YT-065", "status": "fail", "profile": profile,
        "stage": "prerequisites", "checks": [], "browser_version": None,
        "cleanup": "pending", "process_exit_codes": {},
        "not_verified": ["real_os_ime", "screen_reader", "webkit", "mobile", "zoom",
                         "accessibility_conformance", "sla", "production_qualification"],
    }
    server = gecko = driver = temp = None
    failure = None
    cleanup_errors = []
    try:
        if os.name != "posix":
            raise RuntimeError("Owned process-group cleanup requires POSIX")
        binary = ROOT / "target" / profile / "yetlet-forms-validation"
        executable = shutil.which("geckodriver")
        if not executable or not shutil.which("firefox") or not binary.is_file():
            raise RuntimeError("Prebuilt forms binary, Firefox and geckodriver required")
        REPORT.parent.mkdir(parents=True, exist_ok=True)
        temp = Path(tempfile.mkdtemp(prefix="forms-firefox-", dir=REPORT.parent))
        server_port, driver_port = port_number(), port_number()
        while driver_port == server_port:
            driver_port = port_number()
        base = f"http://127.0.0.1:{server_port}"
        report["stage"] = "server_startup"
        server = subprocess.Popen(
            [str(binary)], cwd=ROOT,
            env=dict(os.environ, YETLET_FORMS_LISTEN=f"127.0.0.1:{server_port}"),
            stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, start_new_session=True,
        )
        await_value(lambda: call(base, "/form")[0] == 200, "forms server", seconds=8)
        report["stage"] = "driver_startup"
        gecko = subprocess.Popen(
            [executable, "--host", "127.0.0.1", "--port", str(driver_port)],
            env=dict(os.environ, TMPDIR=str(temp)),
            stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, start_new_session=True,
        )
        driver = FirefoxDriver(driver_port)
        await_value(lambda: driver.call("/status")["ready"] is True, "geckodriver", seconds=8)
        report["stage"] = "session_startup"
        report["browser_version"] = driver.create_session()[:80]
        exercise(driver, base, report)
    except Exception as error:
        # Never serialize potentially sensitive driver/page/process exception text.
        failure = type(error).__name__[:80]
    finally:
        if driver is not None:
            try:
                driver.close()
            except Exception as error:
                cleanup_errors.append(type(error).__name__[:80])
        for name, process in (("geckodriver", gecko), ("forms_server", server)):
            try:
                stop_owned_process(process, process_group=True)
            except Exception as error:
                cleanup_errors.append(type(error).__name__[:80])
            if process is not None:
                report["process_exit_codes"][name] = process.poll()
        if temp is not None:
            try:
                remove_owned_tree(temp)
            except Exception as error:
                cleanup_errors.append(type(error).__name__[:80])
        report["cleanup"] = "fail" if cleanup_errors else "pass"
        report["cleanup_errors"] = cleanup_errors
    report["failure_type"] = failure
    if failure is None and not cleanup_errors:
        report["status"] = "pass"
        report["stage"] = "complete"
    return report


def main(argv=None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--profile", choices=("debug", "release"), default="release")
    options = parser.parse_args(argv)
    report = run(options.profile)
    REPORT.parent.mkdir(parents=True, exist_ok=True)
    REPORT.write_text(json.dumps(report, indent=2, ensure_ascii=True) + "\n", encoding="utf-8")
    print(f"Forms Firefox: {report['status'].upper()}; stage={report['stage']}; "
          f"cleanup={report['cleanup']}; report=target/qualification/forms-accessibility-firefox.json")
    return 0 if report["status"] == "pass" else 1


if __name__ == "__main__":
    sys.exit(main())
