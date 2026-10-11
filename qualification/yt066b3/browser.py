#!/usr/bin/env python3
"""Actual no-JS HTTP Cookie behavior across the three isolated app processes."""
import importlib.util
import json
import os
from pathlib import Path
import shutil
import socket
import subprocess
import sys
import tempfile

ROOT = Path(__file__).resolve().parents[2]
spec = importlib.util.spec_from_file_location("existing_smoke", ROOT / "scripts/smoke_browser.py")
smoke = importlib.util.module_from_spec(spec)
spec.loader.exec_module(smoke)
def run(base, enter_cleanup=lambda: None):
    assert base.startswith("http://127.0.0.1:")
    chrome_bin = shutil.which("google-chrome") or shutil.which("chromium")
    assert chrome_bin
    result = {}; chrome = client = None
    with tempfile.TemporaryDirectory(prefix="yetlet-b3-browser-") as profile:
        port = smoke.port_number()
        with open(Path(profile) / "chrome.log", "w") as log:
            failure = None
            try:
                try:
                    chrome = subprocess.Popen([chrome_bin, "--headless=new", "--no-sandbox", "--disable-gpu",
                        "--disable-background-networking", "--no-first-run", "--disable-extensions",
                        "--remote-debugging-address=127.0.0.1", "--remote-allow-origins=*",
                        f"--remote-debugging-port={port}", f"--user-data-dir={profile}", "about:blank"],
                        stdout=log, stderr=subprocess.STDOUT, start_new_session=True)
                    pages = smoke.wait_until(lambda: json.loads(smoke.http(f"http://127.0.0.1:{port}/json/list")[2]), "owned Chrome")
                    client = smoke.ChromeDevTools(next(p["webSocketDebuggerUrl"] for p in pages if p["type"] == "page"))
                    result["browser"] = client.call("Browser.getVersion")["product"]
                    client.call("Network.enable"); client.call("Page.enable")
                    client.call("Emulation.setScriptExecutionDisabled", {"value": True})
                    def navigate(instance, path, category=None):
                        url = base + f"/__yt066b3/{instance}" + path
                        client.call("Page.navigate", {"url": url})
                        def loaded():
                            if client.call("Page.getFrameTree")["frameTree"]["frame"]["url"] != url: return False
                            try:
                                node = client.call("DOM.getDocument")["root"]["nodeId"]
                                html = client.call("DOM.getOuterHTML", {"nodeId": node})["outerHTML"]
                            except RuntimeError as e:
                                if "Could not find node with given id" in str(e): return False
                                raise
                            return category is None or f'"category":"{category}"' in html
                        smoke.wait_until(loaded, "static SSR category", seconds=5)
                    def cookies(): return client.call("Network.getCookies", {"urls": [base]})["cookies"]
                    navigate("a", "/create", "created")
                    stored = cookies(); assert len(stored) == 1 and stored[0]["name"] == "yt_b3" and stored[0]["httpOnly"]
                    original = stored[0]["value"]
                    navigate("b", "/read", "active"); navigate("c", "/read", "active")
                    navigate("b", "/rotate", "rotated")
                    assert cookies()[0]["value"] != original
                    # A replays its original prepared header; B's replacement remains in Redis.
                    navigate("a", "/control/replay")
                    navigate("c", "/read", "retired_or_missing")
                    assert len(cookies()) == 1 and cookies()[0]["value"] == original
                    navigate("c", "/create", "created"); navigate("a", "/read", "active")
                    navigate("b", "/destroy", "destroyed"); navigate("c", "/read", "retired_or_missing")
                    result.update(js_disabled=True, three_app_shared_session=True,
                        retired_cookie_replay_rejected=True, explicit_create_recovered=True,
                        cross_app_logout_rejected=True, fixed_cookie_count=len(cookies()))
                except BaseException as error:
                    failure = error
                # Transition is covered by the outer recovery boundary.
                enter_cleanup()
            except BaseException as error:
                failure = error
                enter_cleanup()
            try:
                if client is not None: client.close()
            finally:
                smoke.stop_owned_process(chrome, process_group=True)
                result["owned_browser_gone"] = chrome is None or not Path(f"/proc/{chrome.pid}").exists()
                with socket.socket() as sock: result["debug_port_closed"] = sock.connect_ex(("127.0.0.1", port)) != 0
            if failure is not None: raise failure
    result["profile_removed"] = not Path(profile).exists()
    assert result["owned_browser_gone"] and result["debug_port_closed"] and result["profile_removed"]
    return result


if __name__ == "__main__":
    print(json.dumps(run(sys.argv[1]), sort_keys=True))
