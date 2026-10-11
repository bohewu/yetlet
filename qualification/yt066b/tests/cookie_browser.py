#!/usr/bin/env python3
"""Focused real HTTP Cookie check; reuses the existing Chrome DevTools helper."""
import importlib.util
import json
import os
from pathlib import Path
import shutil
import signal
import socket
import subprocess
import sys
import tempfile
import time

ROOT = Path(__file__).resolve().parents[3]
spec = importlib.util.spec_from_file_location("existing_smoke", ROOT / "scripts/smoke_browser.py")
smoke = importlib.util.module_from_spec(spec)
spec.loader.exec_module(smoke)
base = sys.argv[1]
assert base.startswith("http://127.0.0.1:")
chrome_bin = shutil.which("google-chrome") or shutil.which("chromium")
assert chrome_bin, "local Chrome is required for this explicitly selected test"
result = {}
chrome = client = None
with tempfile.TemporaryDirectory(prefix="yetlet-ssr-cookie-") as profile:
    port = smoke.port_number()
    with open(Path(profile) / "chrome.log", "w") as log:
        try:
            chrome = subprocess.Popen([
                chrome_bin, "--headless=new", "--no-sandbox", "--disable-gpu",
                "--disable-background-networking", "--no-first-run", "--disable-extensions",
                "--remote-debugging-address=127.0.0.1", "--remote-allow-origins=*",
                f"--remote-debugging-port={port}", f"--user-data-dir={profile}", "about:blank",
            ], stdout=log, stderr=subprocess.STDOUT, start_new_session=True)
            pages = smoke.wait_until(lambda: json.loads(smoke.http(f"http://127.0.0.1:{port}/json/list")[2]), "owned Chrome")
            client = smoke.ChromeDevTools(next(p["webSocketDebuggerUrl"] for p in pages if p["type"] == "page"))
            result["browser"] = client.call("Browser.getVersion")["product"]
            client.call("Network.enable")
            client.call("Page.enable")
            client.call("Emulation.setScriptExecutionDisabled", {"value": True})
            deadline = time.monotonic() + 45

            def navigate(path, marker):
                assert time.monotonic() < deadline, "focused browser time bound"
                client.call("Page.navigate", {"url": base + path})
                def loaded():
                    try:
                        doc = client.call("DOM.getDocument")["root"]["nodeId"]
                        html = client.call("DOM.getOuterHTML", {"nodeId": doc})["outerHTML"]
                    except RuntimeError as error:
                        # A navigation can replace the document between the two
                        # DevTools reads. Poll the new document, not a stale node.
                        if "Could not find node with given id" in str(error):
                            return False
                        raise
                    return f'<p id="marker">{marker}</p>' in html
                smoke.wait_until(loaded, "static SSR response", seconds=3)

            def cookies():
                return client.call("Network.getCookies", {"urls": [base]})["cookies"]

            navigate("fresh", "FRESH")
            stored = cookies()
            assert len(stored) == 1 and stored[0]["name"] == "yt_ssr" and stored[0]["httpOnly"]
            replacement = stored[0]["value"]
            navigate("auth", "AUTHORIZED")
            for i in range(180):
                navigate(f"old/{i}", f"OLD-{i}")
            stored = cookies()
            assert len(stored) == 1 and stored[0]["value"] != replacement
            navigate("auth", "UNAUTHORIZED")
            navigate("login", "LOGIN")
            navigate("auth", "AUTHORIZED")
            navigate("logout", "LOGOUT")
            navigate("auth", "UNAUTHORIZED")
            navigate("old/180", "OLD-180")
            navigate("auth", "UNAUTHORIZED")
            assert len(cookies()) == 1
            result.update(js_disabled=True, delayed_http_responses=180, cookie_count=1,
                          reordered_login_loss_rejected=True, explicit_login_succeeded=True,
                          logout_and_late_retired_cookie_rejected=True)
        finally:
            if client is not None:
                client.close()
            if chrome is not None:
                if chrome.poll() is None:
                    os.killpg(chrome.pid, signal.SIGTERM)
                    try:
                        chrome.wait(timeout=5)
                    except subprocess.TimeoutExpired:
                        os.killpg(chrome.pid, signal.SIGKILL)
                        chrome.wait(timeout=5)
                result["owned_browser_gone"] = not Path(f"/proc/{chrome.pid}").exists()
            with socket.socket() as sock:
                result["debug_port_closed"] = sock.connect_ex(("127.0.0.1", port)) != 0
result["profile_removed"] = not Path(profile).exists()
assert result["owned_browser_gone"] and result["debug_port_closed"] and result["profile_removed"]
print(json.dumps(result, sort_keys=True))
