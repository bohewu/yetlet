#!/usr/bin/env python3
"""One YT-080 default-profile Firefox observation; no WebGPU preference override.

Uses the actual source-bound candidate, native server and existing test helpers.
Does not repeat the accepted experimental opt-in/loss/benchmark journeys.
"""
from __future__ import annotations

import base64
import hashlib
import json
import os
from pathlib import Path
import shutil
import signal
import socket
import subprocess
import tempfile

from verify_firefox import ELEMENT_KEY, request
from verify_browser import png_pixels
from smoke_browser import http, port_number, stop_owned_process, wait_until

HERE = Path(__file__).resolve().parent
ROOT = HERE.parent.parent
EVIDENCE = ROOT / "target/m11-gpu/yt080-20261014/default-profile"


def main():
    assert os.environ.get("YT075_FIREFOX_HEADFUL_X11") == "1"
    assert not EVIDENCE.exists(), "Never overwrite or automatically repeat a receipt"
    binary = HERE / "target/debug/yetlet-webgpu-scene-island"
    assets = ROOT / "target/webgpu-scene-island-dist"
    driver = shutil.which("geckodriver")
    snap_common = Path.home() / "snap/firefox/common"
    assert binary.is_file() and driver and snap_common.is_dir()
    manifest = json.loads((assets / "manifest.json").read_text())
    for name, digest in manifest["candidate_sources"].items():
        assert hashlib.sha256((ROOT / name).read_bytes()).hexdigest() == digest, "Stale build"
    paths = [ROOT / p for p in manifest["candidate_sources"]]
    paths += [binary, assets / "manifest.json", Path(__file__), HERE / "scene-loader.js"]
    for entry in ("scene", "counter"):
        paths += sorted((assets / "assets" / manifest[entry]["digest"]).iterdir())
    hashes = {str(p.relative_to(ROOT)): hashlib.sha256(p.read_bytes()).hexdigest()
              for p in paths if p.is_file()}
    EVIDENCE.mkdir(parents=True, exist_ok=False)
    report = {"task_id": "YT-080", "status": "running", "requested_preferences": {},
              "profile_boundary": "Fresh WebDriver profile; automation defaults may apply, no WebGPU preference override",
              "manifest": manifest, "input_hashes": hashes, "actions": [],
              "snapshots": [], "owned": {}, "cleanup": {}}
    server = gecko = session = None
    driver_url = ""
    temporary = None

    def deadline(_signum, _frame):
        raise TimeoutError("YT-080 bounded default-profile observation deadline")

    old_handler = signal.signal(signal.SIGALRM, deadline)
    signal.setitimer(signal.ITIMER_REAL, 90)
    try:
        with tempfile.TemporaryDirectory(prefix="yt080-ff-", dir=snap_common) as tmp:
            temporary = Path(tmp)
            profile_root = temporary / "profiles"
            profile_root.mkdir()
            asset_root = temporary / "assets"
            shutil.copytree(assets, asset_root)
            server_port, driver_port = port_number(), port_number()
            assert server_port != driver_port
            base = f"http://127.0.0.1:{server_port}"
            driver_url = f"http://127.0.0.1:{driver_port}"
            env = dict(os.environ, TMPDIR=str(profile_root), DISPLAY=":0",
                       XAUTHORITY=str(Path.home() / ".Xauthority"))
            report["owned"].update(server_port=server_port, driver_port=driver_port)
            with (EVIDENCE / "server.log").open("w") as slog, (EVIDENCE / "geckodriver.log").open("w") as glog:
                try:
                    server = subprocess.Popen([str(binary)], cwd=ROOT,
                        env=dict(env, YETLET_SCENE_LISTEN=f"127.0.0.1:{server_port}",
                                 YETLET_SCENE_ASSET_ROOT=str(asset_root)),
                        stdout=slog, stderr=subprocess.STDOUT, start_new_session=True)
                    report["owned"]["server_pid"] = server.pid
                    wait_until(lambda: http(base + "/healthz")[0] == 200, "owned server", 15)
                    code, headers, html = http(base + "/scene?q=done&page=1")
                    assert code == 200 and b"40 matches; page 2 of 4" in html
                    assert b'<html lang="en">' in html and "unsafe-inline" not in headers["content-security-policy"]
                    report["ssr"] = {"status": code, "search_pagination": True, "strict_csp": True}
                    gecko = subprocess.Popen([driver, "--host", "127.0.0.1", "--port", str(driver_port), "--log", "info"],
                        cwd=ROOT, env=env, stdout=glog, stderr=subprocess.STDOUT, start_new_session=True)
                    report["owned"]["geckodriver_pid"] = gecko.pid
                    wait_until(lambda: http(driver_url + "/status")[0] == 200, "owned driver", 15)
                    response = request("POST", driver_url, "/session", {"capabilities": {"alwaysMatch": {
                        "browserName": "firefox", "moz:firefoxOptions": {"args": [], "prefs": {}}
                    }}})
                    session = response["sessionId"]
                    caps = response.get("capabilities", {})
                    report["capabilities"] = {k: caps.get(k) for k in
                        ("browserName", "browserVersion", "platformName", "moz:processID", "moz:profile")}
                    assert caps.get("browserVersion", "").startswith("158.")
                    prefix = f"/session/{session}"
                    request("POST", driver_url, prefix + "/timeouts", {"script": 15000, "pageLoad": 20000})
                    request("POST", driver_url, prefix + "/window/rect", {"width": 1100, "height": 1000})

                    def evaluate(code):
                        return request("POST", driver_url, prefix + "/execute/sync", {"script": code, "args": []})

                    def async_eval(code):
                        return request("POST", driver_url, prefix + "/execute/async", {"script": code, "args": []})

                    def observe(selector, label):
                        element = request("POST", driver_url, prefix + "/element", {"using": "css selector", "value": selector})
                        key = element[ELEMENT_KEY]
                        state = evaluate("""return {url:location.href, scene:window.__yt075Scene?.snapshot()??null,
                            records:document.querySelectorAll('#records li').length,
                            counter:document.getElementById('counter-value')?.textContent,
                            selection:window.__yt078Selection?.snapshot()??null,
                            href:document.getElementById('scene-detail-link')?.getAttribute('href')??null};""")
                        report["snapshots"].append({"label": label, "selector": selector, "target": key, "value": state})
                        return key, state

                    def action(selector, label, fn):
                        key, _ = observe(selector, label + " before")
                        fn(key)
                        _, state = observe(selector, label + " after")
                        report["actions"].append(label)
                        return state

                    def click(selector, label):
                        return action(selector, label, lambda key: request("POST", driver_url, prefix + f"/element/{key}/click", {}))

                    def keys(value, label):
                        return action("#scene-navigate", label, lambda key: request("POST", driver_url,
                            prefix + f"/element/{key}/value", {"text": value, "value": list(value)}))

                    action("body", "open actual candidate", lambda _: request("POST", driver_url, prefix + "/url", {"url": base + "/scene"}))
                    wait_until(lambda: evaluate("return !!window.__yt075Scene && document.readyState==='complete'"), "actual loader", 20)
                    _, initial = observe("#scene-canvas", "initial inert candidate")
                    assert initial["records"] == 12 and initial["scene"]["starts"] == 0
                    assert not evaluate("return performance.getEntriesByType('resource').some(x=>x.name.includes('scene_bg.wasm'))")
                    report["capability"] = evaluate("return {secure:window.isSecureContext,navigator_gpu:!!navigator.gpu}")
                    click("#activate-scene", "explicit activation with no preference override")
                    action("#scene-canvas", "restore visible surface", lambda _: evaluate("document.getElementById('scene-canvas').scrollIntoView({block:'center'})"))
                    wait_until(lambda: evaluate("return ['ready','unavailable'].includes(window.__yt075Scene.snapshot().state)"), "observed default capability", 25)
                    _, state = observe("#scene-canvas", "actual default outcome")
                    report["outcome"] = state["scene"]["state"]
                    assert state["records"] == 12 and state["counter"] == "2"
                    if report["outcome"] == "ready":
                        # Only an actually available default backend may be called a GPU PASS.
                        adapter = async_eval("const done=arguments[arguments.length-1];navigator.gpu.requestAdapter().then(a=>done(a?{fallback:a.isFallbackAdapter??a.info?.isFallbackAdapter??null}:null));")
                        assert adapter and adapter["fallback"] is False
                        report["adapter"] = adapter
                        action("#scene-canvas", "fresh default GPU pixels", lambda _: evaluate("window.__yt075Scene.draw()"))
                        async_eval("const done=arguments[arguments.length-1];window.__yt075Scene.completion().then(()=>requestAnimationFrame(()=>requestAnimationFrame(()=>done(null))));")
                        key, _ = observe("#scene-canvas", "capture default GPU")
                        image = base64.b64decode(request("GET", driver_url, prefix + f"/element/{key}/screenshot"))
                        (EVIDENCE / "canvas.png").write_bytes(image)
                        width, height, channels, rows = png_pixels(image)
                        top = int(all(list(rows[0][x*channels:x*channels+3]) == [255,255,255] for x in range(width)))
                        errors = []
                        for i in range(120):
                            x = int((i % 12 + .5)*width/12)
                            y = int((i // 12 + .5)*height/10) + top
                            actual = list(rows[y][x*channels:x*channels+3])
                            expected = ([255,0,0],[0,255,0],[0,0,255])[i % 3]
                            if any(abs(a-b)>25 for a,b in zip(actual,expected)): errors.append(i)
                        assert not errors
                        report["pixels"] = {"verified": 120, "sha256": hashlib.sha256(image).hexdigest()}
                    else:
                        assert state["scene"]["ownedBufferBytes"] == state["scene"]["ownedSurfaces"] == 0
                        assert evaluate("return document.getElementById('scene-status').textContent.includes('HTML')")
                        report["fallback"] = "Actual unavailable state; semantic records and controls remain usable"
                    assert click("#counter-increment", "independent counter")["counter"] == "3"
                    click("#scene-navigate", "focus semantic keyboard alternative")
                    keys("\ue010", "select last record")
                    chosen = keys("\ue007", "commit semantic selection")
                    assert chosen["selection"]["selectedRecord"] == 120 and chosen["href"] == "/scene?record=120"
                    action("#scene-detail-link", "native detail navigation", lambda key: request("POST", driver_url, prefix + f"/element/{key}/click", {}))
                    assert evaluate("return document.getElementById('detail').textContent.includes('Work record 120')")
                    report["status"] = "passed_yt080_default_profile"
                finally:
                    signal.setitimer(signal.ITIMER_REAL, 0)
                    if session is not None:
                        try:
                            request("DELETE", driver_url, f"/session/{session}")
                            report["cleanup"]["session_deleted"] = True
                        except Exception as exc: report["cleanup"]["session_error"] = str(exc)[:180]
                    for name, proc in (("geckodriver", gecko), ("server", server)):
                        try:
                            stop_owned_process(proc, process_group=True)
                            report["cleanup"][name + "_exit"] = proc.returncode if proc else None
                        except Exception as exc: report["cleanup"][name + "_error"] = str(exc)[:180]
    except BaseException as exc:
        report["status"] = "failed"
        report["error"] = type(exc).__name__ + ": " + str(exc)[:450]
    finally:
        signal.setitimer(signal.ITIMER_REAL, 0)
        signal.signal(signal.SIGALRM, old_handler)
        cleanup = report["cleanup"]
        cleanup["profile_removed"] = temporary is None or not temporary.exists()
        cleanup["processes_reaped"] = all(p is None or p.poll() is not None for p in (server, gecko))
        pid = report.get("capabilities", {}).get("moz:processID")
        if pid:
            try: os.kill(pid, 0); cleanup["browser_exited"] = False
            except ProcessLookupError: cleanup["browser_exited"] = True
        else: cleanup["browser_exited"] = session is None
        cleanup["ports_closed"] = {}
        for name in ("server_port", "driver_port"):
            if name not in report["owned"]: continue
            try:
                with socket.create_connection(("127.0.0.1", report["owned"][name]), timeout=.3): cleanup["ports_closed"][name] = False
            except OSError: cleanup["ports_closed"][name] = True
        report["inputs_unchanged"] = all((ROOT/p).is_file() and hashlib.sha256((ROOT/p).read_bytes()).hexdigest() == h for p,h in hashes.items())
        (EVIDENCE / "result.json").write_text(json.dumps(report, indent=2) + "\n")
        print(json.dumps({k:report.get(k) for k in ("status","error","capability","outcome","fallback","pixels","cleanup","inputs_unchanged")}))
    return 0 if (report["status"] == "passed_yt080_default_profile" and report["inputs_unchanged"]
        and cleanup.get("session_deleted") and cleanup["profile_removed"] and cleanup["processes_reaped"]
        and cleanup["browser_exited"] and all(cleanup["ports_closed"].values())
        and not any(k.endswith("_error") for k in cleanup)) else 1


if __name__ == "__main__":
    raise SystemExit(main())
