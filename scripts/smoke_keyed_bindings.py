#!/usr/bin/env python3
"""Bounded Chrome E2E for the isolated M7b/M7c keyed-reactivity prototype."""
from __future__ import annotations

import argparse
from contextlib import contextmanager
from functools import partial
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer
import json
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import threading
import time

from smoke_browser import ChromeDevTools, http, port_number, wait_until

ROOT = Path(__file__).resolve().parent.parent
TARGET = ROOT / "target/m7b-keyed-bindings"


class QuietHandler(SimpleHTTPRequestHandler):
    def log_message(self, format, *args):  # noqa: A002
        pass


@contextmanager
def static_server(root: Path):
    port = port_number()
    handler = partial(QuietHandler, directory=str(root))
    server = ThreadingHTTPServer(("127.0.0.1", port), handler)
    thread = threading.Thread(target=server.serve_forever, daemon=True)
    thread.start()
    try:
        yield f"http://127.0.0.1:{port}"
    finally:
        server.shutdown()
        server.server_close()
        thread.join(timeout=5)
        if thread.is_alive():
            raise RuntimeError("M7b static server did not stop")


def evaluate_error(client: ChromeDevTools, expression: str):
    return client.evaluate(
        "(()=>{try{" + expression + ";return null}catch(error){return String(error)}})()"
    )


def run(profile: str) -> None:
    root = TARGET / profile
    manifest = json.loads((root / "manifest.json").read_text(encoding="utf-8"))
    if manifest.get("profile") != profile:
        raise RuntimeError("M7b manifest/profile mismatch")
    chrome = shutil.which("google-chrome-stable") or shutil.which("google-chrome")
    if not chrome:
        raise RuntimeError("Headless Chrome is required")

    with tempfile.TemporaryDirectory(prefix="yetlet-m7b-") as folder, static_server(root) as base:
        temp = Path(folder)
        chrome_port = port_number()
        with (temp / "chrome.log").open("w") as log:
            process = subprocess.Popen(
                [
                    chrome,
                    "--headless=new",
                    "--no-sandbox",
                    "--disable-gpu",
                    "--disable-background-networking",
                    "--no-first-run",
                    "--disable-extensions",
                    "--remote-debugging-address=127.0.0.1",
                    "--remote-allow-origins=*",
                    f"--remote-debugging-port={chrome_port}",
                    f"--user-data-dir={temp / 'chrome-profile'}",
                    "about:blank",
                ],
                stdout=log,
                stderr=subprocess.STDOUT,
            )
            client = None
            try:
                def target_url():
                    targets = json.loads(http(f"http://127.0.0.1:{chrome_port}/json/list")[2])
                    return next(
                        (
                            item["webSocketDebuggerUrl"]
                            for item in targets
                            if item.get("type") == "page"
                        ),
                        None,
                    )

                client = ChromeDevTools(wait_until(target_url, "Chrome debugger"))
                for method in ("Runtime.enable", "Page.enable", "Network.enable", "Log.enable"):
                    client.call(method)
                client.call("Network.setCacheDisabled", {"cacheDisabled": True})
                client.call("Page.navigate", {"url": base + "/index.html"})
                wait_until(
                    lambda: client.evaluate(
                        "document.documentElement.getAttribute('data-yetlet-m7b-ready') === 'true'"
                    ),
                    "M7b module readiness",
                )
                assert client.evaluate(
                    "document.documentElement.getAttribute('data-yetlet-m7c-ready') === 'true'"
                )
                policy = client.evaluate(
                    "document.querySelector('meta[http-equiv=\"Content-Security-Policy\"]').content"
                )
                assert "'wasm-unsafe-eval'" in policy
                assert "'unsafe-inline'" not in policy and "'unsafe-eval'" not in policy
                assert client.evaluate(
                    "document.querySelectorAll('[data-yetlet-key]').length"
                ) == 3
                assert client.evaluate(
                    "document.querySelector('[data-yetlet-component]').getAttribute('data-yetlet-bound')"
                ) is None
                client.evaluate(
                    "window.__m7bRoot=document.querySelector('[data-yetlet-component]');"
                    "window.__m7bRow2=document.querySelector('[data-yetlet-key=\"2\"]');"
                    "window.__m7bLabel2=window.__m7bRow2.querySelector('[data-yetlet-slot=\"label\"]');"
                    "window.__m7bLabelText2=window.__m7bLabel2.firstChild;"
                    "window.__m7bRoot.setAttribute('data-yetlet-instance','wrong-instance')"
                )
                mismatch = evaluate_error(client, "window.yetletBindings.mount_demo()")
                assert mismatch and "identity mismatch" in mismatch, mismatch
                assert client.evaluate(
                    "window.__m7bRoot.isSameNode(document.querySelector('[data-yetlet-component]'))"
                )
                assert client.evaluate(
                    "window.__m7bRow2.isSameNode(document.querySelector('[data-yetlet-key=\"2\"]'))"
                )
                assert client.evaluate(
                    "document.querySelector('[data-yetlet-component]').getAttribute('data-yetlet-bound')"
                ) is None
                client.evaluate(
                    "window.__m7bRoot.setAttribute('data-yetlet-instance','m7b-demo')"
                )
                client.evaluate(
                    "window.__m7bRow2.setAttribute('data-yetlet-key','99')"
                )
                keyed_mismatch = evaluate_error(client, "window.yetletBindings.mount_demo()")
                assert keyed_mismatch and "unknown row key" in keyed_mismatch, keyed_mismatch
                assert client.evaluate(
                    "window.__m7bRow2.isSameNode(document.querySelector('[data-yetlet-key=\"99\"]'))"
                )
                client.evaluate(
                    "window.__m7bRow2.setAttribute('data-yetlet-key','2')"
                )
                client.evaluate(
                    "window.__m7bExtraText=document.createTextNode('unexpected');"
                    "window.__m7bRow2.appendChild(window.__m7bExtraText)"
                )
                row_shape_mismatch = evaluate_error(
                    client, "window.yetletBindings.mount_demo()"
                )
                assert row_shape_mismatch and "unexpected child node" in row_shape_mismatch
                assert client.evaluate(
                    "document.querySelector('[data-yetlet-component]').getAttribute('data-yetlet-bound')"
                ) is None
                client.evaluate(
                    "document.querySelector('[data-yetlet-key=\"2\"] [data-yetlet-action=\"row\"]').click()"
                )
                time.sleep(0.05)
                assert client.evaluate(
                    "window.__m7bRow2.getAttribute('data-completed')"
                ) == "false"
                client.evaluate("window.__m7bExtraText.remove()")
                client.evaluate(
                    "window.__m7bLabel2.innerHTML='<b>Update typed slots locally: active</b>'"
                )
                slot_mismatch = evaluate_error(client, "window.yetletBindings.mount_demo()")
                assert slot_mismatch and "label slot mismatch" in slot_mismatch, slot_mismatch
                assert client.evaluate(
                    "window.__m7bRow2.isSameNode(document.querySelector('[data-yetlet-key=\"2\"]'))"
                )
                assert client.evaluate(
                    "document.querySelector('[data-yetlet-component]').getAttribute('data-yetlet-bound')"
                ) is None
                client.evaluate(
                    "window.__m7bLabel2.textContent='Update typed slots locally: active';"
                    "window.__m7bLabelText2=window.__m7bLabel2.firstChild"
                )
                assert evaluate_error(
                    client, "window.yetletBindings.verify_disposed_handle_isolation()"
                ) is None
                assert client.evaluate(
                    "document.querySelector('[data-yetlet-component]').getAttribute('data-yetlet-bound')"
                ) is None
                assert client.evaluate(
                    "window.__m7bRow2.isSameNode(document.querySelector('[data-yetlet-key=\"2\"]'))"
                )
                assert evaluate_error(client, "window.yetletBindings.mount_demo()") is None
                assert client.evaluate(
                    "window.__m7bRoot.isSameNode(document.querySelector('[data-yetlet-component]'))"
                )
                assert client.evaluate(
                    "window.__m7bRow2.isSameNode(document.querySelector('[data-yetlet-key=\"2\"]'))"
                )
                assert client.evaluate(
                    "document.querySelector('[data-yetlet-component]').getAttribute('data-yetlet-bound')"
                ) == "true"
                assert client.evaluate(
                    "window.__m7bLabel2.isSameNode(document.querySelector('[data-yetlet-key=\"2\"] [data-yetlet-slot=\"label\"]'))"
                )
                assert client.evaluate(
                    "window.__m7bLabelText2.isSameNode(document.querySelector('[data-yetlet-key=\"2\"] [data-yetlet-slot=\"label\"]').firstChild)"
                )

                lifecycle_requests = len(client.requests)
                client.evaluate(
                    "window.__m7dInput2=document.querySelector('[data-yetlet-key=\"2\"] [data-yetlet-input=\"draft\"]');"
                    "window.__m7dInput2.focus();"
                    "window.__m7dInput2.value='Edited draft two';"
                    "window.__m7dInput2.setSelectionRange(7,12);"
                    "window.__m7dInput2.dispatchEvent(new Event('input',{bubbles:true}))"
                )
                wait_until(
                    lambda: client.evaluate(
                        "document.querySelector('[data-yetlet-key=\"2\"]').getAttribute('data-draft') === 'Edited draft two'"
                    ),
                    "controlled row input commit",
                )
                assert client.evaluate(
                    "window.__m7dInput2.isSameNode(document.querySelector('[data-yetlet-key=\"2\"] [data-yetlet-input=\"draft\"]'))"
                )
                assert client.evaluate("document.activeElement === window.__m7dInput2")
                assert client.evaluate("window.__m7dInput2.selectionStart") == 7
                assert client.evaluate("window.__m7dInput2.selectionEnd") == 12
                assert evaluate_error(client, "window.yetletBindings.reverse_demo()") is None
                assert client.evaluate(
                    "Array.from(document.querySelectorAll('[data-yetlet-key]')).map(x=>x.getAttribute('data-yetlet-key')).join(',')"
                ) == "3,2,1"
                assert client.evaluate(
                    "window.__m7dInput2.isSameNode(document.querySelector('[data-yetlet-key=\"2\"] [data-yetlet-input=\"draft\"]'))"
                )
                assert client.evaluate("document.activeElement === window.__m7dInput2")
                assert client.evaluate("window.__m7dInput2.value") == "Edited draft two"
                assert client.evaluate("window.__m7dInput2.selectionStart") == 7
                assert client.evaluate("window.__m7dInput2.selectionEnd") == 12
                assert evaluate_error(client, "window.yetletBindings.reverse_demo()") is None
                assert client.evaluate(
                    "Array.from(document.querySelectorAll('[data-yetlet-key]')).map(x=>x.getAttribute('data-yetlet-key')).join(',')"
                ) == "1,2,3"
                assert client.evaluate("document.activeElement === window.__m7dInput2")
                assert client.evaluate("window.__m7dInput2.selectionStart") == 7
                assert client.evaluate("window.__m7dInput2.selectionEnd") == 12
                assert len(client.requests) == lifecycle_requests
                assert client.evaluate(
                    "document.documentElement.getAttribute('data-yetlet-m7d-event-error')"
                ) is None

                requests_before = len(client.requests)
                client.evaluate(
                    "document.getElementById('outside-focus').focus();"
                    "document.querySelector('[data-yetlet-key=\"2\"] [data-yetlet-action=\"row\"]').click()"
                )
                wait_until(
                    lambda: client.evaluate(
                        "document.querySelector('[data-yetlet-key=\"2\"]').getAttribute('data-completed') === 'true'"
                    ),
                    "typed local row event",
                )
                assert client.evaluate(
                    "window.__m7bRow2.isSameNode(document.querySelector('[data-yetlet-key=\"2\"]'))"
                )
                assert client.evaluate(
                    "document.querySelector('[data-yetlet-key=\"2\"] [data-yetlet-slot=\"details\"]') !== null"
                )
                assert client.evaluate("document.activeElement.id") == "outside-focus"
                assert len(client.requests) == requests_before
                client.evaluate(
                    "document.querySelector('[data-yetlet-key=\"2\"] [data-yetlet-action=\"row\"]').click()"
                )
                wait_until(
                    lambda: client.evaluate(
                        "document.querySelector('[data-yetlet-key=\"2\"]').getAttribute('data-completed') === 'false'"
                    ),
                    "conditional slot removal",
                )
                assert client.evaluate(
                    "document.querySelector('[data-yetlet-key=\"2\"] [data-yetlet-slot=\"details\"]') === null"
                )
                assert client.evaluate(
                    "window.__m7bRow2.isSameNode(document.querySelector('[data-yetlet-key=\"2\"]'))"
                )
                assert client.evaluate("document.activeElement.id") == "outside-focus"
                assert len(client.requests) == requests_before

                assert evaluate_error(client, "window.yetletBindings.reverse_demo()") is None
                assert client.evaluate(
                    "Array.from(document.querySelectorAll('[data-yetlet-key]')).map(x=>x.getAttribute('data-yetlet-key')).join(',')"
                ) == "3,2,1"
                assert client.evaluate(
                    "window.__m7bRow2.isSameNode(document.querySelector('[data-yetlet-key=\"2\"]'))"
                )
                assert evaluate_error(client, "window.yetletBindings.append_demo()") is None
                assert client.evaluate(
                    "document.querySelector('[data-yetlet-key=\"4\"]') !== null"
                )
                client.evaluate(
                    "window.__m7dRemovedInput=document.querySelector('[data-yetlet-key=\"1\"] [data-yetlet-input=\"draft\"]');"
                    "window.__m7dRemovedButton=document.querySelector('[data-yetlet-key=\"1\"] [data-yetlet-action=\"row\"]');"
                    "window.__m7dRemovedInput.focus();"
                    "window.__m7dRemovedInput.setSelectionRange(2,6)"
                )
                assert client.evaluate("document.activeElement === window.__m7dRemovedInput")
                assert client.evaluate("window.__m7dRemovedInput.selectionStart") == 2
                assert client.evaluate("window.__m7dRemovedInput.selectionEnd") == 6
                removed_commit_before = int(
                    client.evaluate(
                        "document.documentElement.getAttribute('data-yetlet-m7c-commit-count')"
                    )
                )
                assert evaluate_error(client, "window.yetletBindings.remove_first_demo()") is None
                assert client.evaluate(
                    "Array.from(document.querySelectorAll('[data-yetlet-key]')).map(x=>x.getAttribute('data-yetlet-key')).join(',')"
                ) == "3,2,4"
                assert not client.evaluate("window.__m7dRemovedInput.isConnected")
                assert not client.evaluate("document.activeElement === window.__m7dRemovedInput")
                removed_commit_after = int(
                    client.evaluate(
                        "document.documentElement.getAttribute('data-yetlet-m7c-commit-count')"
                    )
                )
                assert removed_commit_after == removed_commit_before + 1
                client.evaluate(
                    "window.__m7dRemovedInput.value='detached edit';"
                    "window.__m7dRemovedInput.dispatchEvent(new Event('input',{bubbles:true}));"
                    "window.__m7dRemovedButton.click()"
                )
                time.sleep(0.1)
                assert int(
                    client.evaluate(
                        "document.documentElement.getAttribute('data-yetlet-m7c-commit-count')"
                    )
                ) == removed_commit_after
                assert client.evaluate(
                    "document.documentElement.getAttribute('data-yetlet-m7d-event-error')"
                ) is None
                assert evaluate_error(client, "window.yetletBindings.reset_demo()") is None
                assert client.evaluate(
                    "Array.from(document.querySelectorAll('[data-yetlet-key]')).map(x=>x.getAttribute('data-yetlet-key')).join(',')"
                ) == "1,2,3"
                # Remount a fresh owner before exercising the M7c graph contract.
                assert evaluate_error(client, "window.yetletBindings.dispose_demo()") is None
                assert evaluate_error(client, "window.yetletBindings.mount_demo()") is None
                client.evaluate(
                    "window.__m7cRow2=document.querySelector('[data-yetlet-key=\"2\"]')"
                )
                assert client.evaluate(
                    "document.documentElement.getAttribute('data-yetlet-m7c-owned')"
                ) == "true"
                assert client.evaluate(
                    "document.documentElement.getAttribute('data-yetlet-m7c-commit-count')"
                ) == "0"
                assert client.evaluate(
                    "document.documentElement.getAttribute('data-yetlet-m7c-notification-count')"
                ) == "0"

                assert evaluate_error(client, "window.yetletBindings.batch_demo()") is None
                assert client.evaluate(
                    "document.documentElement.getAttribute('data-yetlet-m7c-batch-messages')"
                ) == "3"
                assert client.evaluate(
                    "document.documentElement.getAttribute('data-yetlet-m7c-batch-passes')"
                ) == "1"
                assert client.evaluate(
                    "document.documentElement.getAttribute('data-yetlet-m7c-batch-commits')"
                ) == "1"
                assert client.evaluate(
                    "document.documentElement.getAttribute('data-yetlet-m7c-commit-count')"
                ) == "1"
                assert client.evaluate(
                    "document.documentElement.getAttribute('data-yetlet-m7c-notification-count')"
                ) == "1"
                assert client.evaluate(
                    "Array.from(document.querySelectorAll('[data-yetlet-key]')).map(x=>x.getAttribute('data-yetlet-key')).join(',')"
                ) == "3,2,1"
                assert client.evaluate(
                    "window.__m7cRow2.isSameNode(document.querySelector('[data-yetlet-key=\"2\"]'))"
                )

                assert evaluate_error(client, "window.yetletBindings.reentrant_demo()") is None
                assert client.evaluate(
                    "document.documentElement.getAttribute('data-yetlet-m7c-reentrant-messages')"
                ) == "2"
                assert client.evaluate(
                    "document.documentElement.getAttribute('data-yetlet-m7c-reentrant-passes')"
                ) == "2"
                assert client.evaluate(
                    "document.documentElement.getAttribute('data-yetlet-m7c-reentrant-commits')"
                ) == "2"
                assert client.evaluate(
                    "document.documentElement.getAttribute('data-yetlet-m7c-commit-count')"
                ) == "3"
                assert client.evaluate(
                    "document.documentElement.getAttribute('data-yetlet-m7c-notification-count')"
                ) == "3"

                assert evaluate_error(
                    client, "window.yetletBindings.verify_subscription_disposal()"
                ) is None
                assert client.evaluate(
                    "document.documentElement.getAttribute('data-yetlet-m7c-commit-count')"
                ) == "4"
                assert client.evaluate(
                    "document.documentElement.getAttribute('data-yetlet-m7c-notification-count')"
                ) == "4"
                assert evaluate_error(
                    client, "window.yetletBindings.verify_reducer_failure()"
                ) is None
                assert client.evaluate(
                    "document.documentElement.getAttribute('data-yetlet-m7c-commit-count')"
                ) == "5"
                assert client.evaluate(
                    "document.documentElement.getAttribute('data-yetlet-m7c-notification-count')"
                ) == "5"
                assert evaluate_error(
                    client, "window.yetletBindings.verify_commit_failure_isolation()"
                ) is None
                assert client.evaluate(
                    "window.__m7cRow2.isSameNode(document.querySelector('[data-yetlet-key=\"2\"]'))"
                )

                disposed_input = client.evaluate(
                    "window.__m7dDisposedInput=document.querySelector('[data-yetlet-key=\"2\"] [data-yetlet-input=\"draft\"]');"
                    "window.__m7dDisposedInput.getAttribute('value')"
                )
                before_dispose = client.evaluate(
                    "document.querySelector('[data-yetlet-key=\"2\"]').getAttribute('data-completed')"
                )
                assert evaluate_error(
                    client, "window.yetletBindings.verify_graph_disposal()"
                ) is None
                assert client.evaluate(
                    "document.querySelector('[data-yetlet-component]').getAttribute('data-yetlet-bound')"
                ) is None
                assert client.evaluate(
                    "document.documentElement.getAttribute('data-yetlet-m7c-owned')"
                ) is None
                client.evaluate(
                    "window.__m7dDisposedInput.value='after dispose';"
                    "window.__m7dDisposedInput.dispatchEvent(new Event('input',{bubbles:true}))"
                )
                time.sleep(0.1)
                assert client.evaluate(
                    "window.__m7dDisposedInput.getAttribute('value')"
                ) == disposed_input
                assert client.evaluate(
                    "document.querySelector('[data-yetlet-key=\"2\"]').getAttribute('data-draft')"
                ) == disposed_input
                assert client.evaluate(
                    "document.documentElement.getAttribute('data-yetlet-m7d-event-error')"
                ) is None
                client.evaluate(
                    "document.querySelector('[data-yetlet-key=\"2\"] [data-yetlet-action=\"row\"]').click()"
                )
                time.sleep(0.15)
                assert client.evaluate(
                    "document.querySelector('[data-yetlet-key=\"2\"]').getAttribute('data-completed')"
                ) == before_dispose
                assert len(client.requests) == requests_before
                assert not client.websockets, client.websockets
                assert not client.failed_responses, client.failed_responses
                assert not client.exceptions, client.exceptions
                print("PASS: M7b rejects mismatched SSR identity without replacing markup")
                print("PASS: typed slots, conditional node and local event update in place")
                print("PASS: keyed reorder/append/remove preserve existing row identity")
                print("PASS: explicit dispose removes event behavior; no network or WebSocket")
                print("PASS: disposed handle cannot clear a newer binding marker")
                print("PASS: M7c batch coalesces to one commit and one notification")
                print("PASS: re-entrant dispatch is deferred into a bounded second pass")
                print("PASS: reducer/commit failures and graph disposal fail closed")
                print("PASS: subscription and owner disposal stop later propagation")
                print("PASS: M7d controlled input preserves node, value, focus and selection across keyed reorder")
                print("PASS: removed/disposed row input and action listeners stop local propagation")
                print("M7b/M7c/M7d artifact digest:", manifest["digest"])
            finally:
                if client is not None:
                    client.close()
                if process.poll() is None:
                    process.terminate()
                    try:
                        process.wait(timeout=5)
                    except subprocess.TimeoutExpired:
                        process.kill()
                        process.wait(timeout=5)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--profile", choices=("debug", "release"), default="release")
    args = parser.parse_args()
    run(args.profile)
    return 0


if __name__ == "__main__":
    try:
        sys.exit(main())
    except (AssertionError, OSError, RuntimeError, subprocess.CalledProcessError) as error:
        print(f"M7b/M7c/M7d smoke FAILED: {error!r}", file=sys.stderr)
        sys.exit(1)
