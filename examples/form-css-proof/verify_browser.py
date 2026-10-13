#!/usr/bin/env python3
"""One bounded form-proof journey using the repository's existing Chrome driver.

Synthetic composition events are not real OS IME evidence. No user browser is
attached. Only owned temporary processes, profile and listeners are stopped.
"""
from __future__ import annotations
import base64
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import time

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "scripts"))
from smoke_browser import ChromeDevTools, http, port_number, wait_until, stop_owned_process, remove_owned_tree

OUT = ROOT / "target/m11-form-proof"

def run():
    OUT.mkdir(parents=True, exist_ok=True)
    report = {"task_ids": ["YT-070", "YT-071"], "status": "fail", "checks": [], "not_verified": ["real OS IME", "spoken assistive technology", "Firefox", "mobile device", "generic DOM form mounting"], "cleanup": "pending"}
    server = chrome = client = None
    temp = Path(tempfile.mkdtemp(prefix="form-proof-", dir=OUT))
    ports = []
    try:
        server_port, chrome_port = port_number(), port_number()
        assert server_port != chrome_port
        ports = [server_port, chrome_port]
        base = f"http://127.0.0.1:{server_port}"
        binary = ROOT / "target/debug/yetlet-form-css-proof"
        assert binary.is_file()
        report["binary_sha256"] = hashlib.sha256(binary.read_bytes()).hexdigest()
        report["head"] = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip()
        report["source_sha256"] = {str(p.relative_to(ROOT)): hashlib.sha256(p.read_bytes()).hexdigest() for p in (ROOT / "examples/form-css-proof").rglob("*") if p.is_file()}
        server = subprocess.Popen([str(binary)], cwd=ROOT, env=dict(os.environ, YETLET_FORM_PROOF_LISTEN=f"127.0.0.1:{server_port}"), stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, start_new_session=True)
        wait_until(lambda: http(base + "/healthz")[0] == 200, "form proof readiness", seconds=8)
        executable = shutil.which("google-chrome")
        assert executable
        chrome = subprocess.Popen([executable, "--headless=new", "--disable-gpu", "--no-first-run", "--no-default-browser-check", "--remote-allow-origins=http://127.0.0.1", f"--remote-debugging-port={chrome_port}", f"--user-data-dir={temp / 'profile'}", "about:blank"], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, start_new_session=True)
        targets = wait_until(lambda: json.loads(http(f"http://127.0.0.1:{chrome_port}/json")[2]), "owned Chrome target", seconds=10)
        pages = [target for target in targets if target.get("type") == "page"]
        assert len(pages) == 1
        client = ChromeDevTools(pages[0]["webSocketDebuggerUrl"])
        report["browser"] = client.call("Browser.getVersion")["product"]
        for method in ["Page.enable", "Runtime.enable", "Network.enable", "Log.enable"]:
            client.call(method)
        def observe(expr):
            return client.evaluate(expr)
        def check(expr, label):
            assert observe(expr) is True, label
            report["checks"].append(label)
        def navigate(path):
            client.call("Page.navigate", {"url": base + path})
            wait_until(lambda: observe("document.readyState === 'complete' && location.pathname === " + json.dumps(path)), path)
        def click(selector):
            check(f"document.querySelectorAll({json.dumps(selector)}).length === 1", "unique control " + selector)
            observe(f"document.querySelector({json.dumps(selector)}).click()")
            observe("document.readyState")
        client.call("Emulation.setScriptExecutionDisabled", {"value": True})
        navigate("/proof/native")
        status, headers, html = http(base + "/proof/native")
        assert status == 200 and "unsafe-inline" not in headers["content-security-policy"] and b"<script" not in html
        report["checks"].append("native response has no scripts and strict CSP")
        client.call("Emulation.setScriptExecutionDisabled", {"value": False})
        check("document.querySelector('#notify').checked && document.querySelector('#priority').value === 'normal' && document.querySelector('#notes').value === 'Check the latest test report.'", "native initial checkbox/select/textarea properties")
        check("getComputedStyle(document.querySelector('.form-control')).paddingLeft !== '0px' && [...document.styleSheets].length === 2", "actual Bootstrap and application styles loaded")
        check("['title','notes','priority','notify'].every(id => document.querySelector('label[for='+id+']').control === document.getElementById(id))", "labels associated with actual controls")
        client.call("Emulation.setDeviceMetricsOverride", {"width": 1280, "height": 960, "deviceScaleFactor": 1, "mobile": False})
        (OUT / "native-desktop.png").write_bytes(base64.b64decode(client.call("Page.captureScreenshot", {"captureBeyondViewport": True})["data"]))
        client.call("Emulation.setDeviceMetricsOverride", {"width": 390, "height": 844, "deviceScaleFactor": 1, "mobile": False})
        check("document.documentElement.scrollWidth <= 390", "390px responsive page has no horizontal overflow")
        (OUT / "native-narrow.png").write_bytes(base64.b64decode(client.call("Page.captureScreenshot")["data"]))
        navigate("/proof/enhanced")
        check("document.querySelectorAll('script').length === 1", "preview is separately selected external JS")
        click("[data-preview-activate]")
        check("document.querySelector('[data-form-proof]').dataset.previewState === 'mounted'", "preview activation")
        observe("globalThis.proofOriginal = document.getElementById('title'); proofOriginal.focus(); proofOriginal.setSelectionRange(2, 5)")
        click("[data-preview-activate]")
        check("document.getElementById('title') === proofOriginal && proofOriginal.selectionStart === 2 && proofOriginal.selectionEnd === 5", "remount retains control identity and selection")
        observe("globalThis.beforeCount = Number(document.querySelector('[data-form-proof]').dataset.previewUpdates); proofOriginal.value = '<b>literal</b>'; proofOriginal.dispatchEvent(new Event('input', {bubbles:true}))")
        check("Number(document.querySelector('[data-form-proof]').dataset.previewUpdates) === beforeCount + 1 && document.querySelector('[data-preview-summary]').textContent.includes('<b>literal</b>') && !document.querySelector('[data-preview-summary] b')", "one listener after remount and text-only preview sink")
        observe("globalThis.beforeComposition = document.querySelector('[data-preview-summary]').textContent; proofOriginal.dispatchEvent(new CompositionEvent('compositionstart', {bubbles:true})); proofOriginal.value = 'synthetic composition'; proofOriginal.dispatchEvent(new InputEvent('input', {bubbles:true,isComposing:true}))")
        check("document.querySelector('[data-preview-summary]').textContent === beforeComposition", "synthetic composition suppresses premature preview")
        observe("proofOriginal.dispatchEvent(new CompositionEvent('compositionend', {bubbles:true}))")
        check("document.querySelector('[data-preview-summary]').textContent.includes('synthetic composition')", "synthetic compositionend uses live value, not OS IME proof")
        click("[data-preview-later]")
        click("[data-preview-dispose]")
        observe("globalThis.disposedCount = document.querySelector('[data-form-proof]').dataset.previewUpdates; proofOriginal.value = 'retired'; proofOriginal.dispatchEvent(new Event('input', {bubbles:true}))")
        time.sleep(0.25)
        check("document.querySelector('[data-form-proof]').dataset.previewUpdates === disposedCount", "disposed listeners and timer cannot update preview")
        click("[data-preview-activate]")
        click("button[type=reset]")
        check("proofOriginal.value === 'Inspect lab backup' && document.querySelector('[data-preview-summary]').textContent.includes('Inspect lab backup')", "native reset and preview read restored defaults")
        before = len(client.api_posts)
        click("#notify")
        check("!document.getElementById('notify').checked && document.querySelector('[data-preview-summary]').textContent.includes('notifications: off')", "live checkbox is not initial checked attribute")
        assert len(client.api_posts) == before and not client.websockets
        report["checks"].append("local preview has no mutation POST or event circuit")
        click("button[type=submit]")
        wait_until(lambda: observe("location.pathname === '/proof/accepted' && document.readyState === 'complete'"), "native form PRG")
        check("document.querySelector('h1').textContent === 'Form validated'", "real native submit and redirect complete")
        assert not client.exceptions, client.exceptions
        assert not client.failed_responses, client.failed_responses
        assert all(url.startswith(base) or url == "about:blank" for url in client.requests), "unexpected external request"
        report["checks"].append("no CSP/JS/network errors or external assets")
        report["status"] = "pass"
    finally:
        if client is not None:
            client.close()
        for process in (chrome, server):
            stop_owned_process(process, process_group=True)
        remove_owned_tree(temp)
        import socket
        for port in ports:
            with socket.socket() as sock:
                assert sock.connect_ex(("127.0.0.1", port)) != 0, "owned listener survived"
        report["cleanup"] = "owned processes reaped, listeners closed, temporary profile removed"
        (OUT / "browser.json").write_text(json.dumps(report, indent=2) + "\n")
        print(json.dumps({k: v for k, v in report.items() if k != "source_sha256"}, indent=2))
    assert report["status"] == "pass"

if __name__ == "__main__":
    run()
