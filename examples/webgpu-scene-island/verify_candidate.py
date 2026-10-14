#!/usr/bin/env python3
"""One YT-079 extracted-candidate Firefox X11 integration product test; never reruns YT-075 receipts.

Uses the established WebDriver/PNG/process helpers, a temporary profile with only
experimental dom.webgpu.enabled=true, and existing application build outputs.
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
from verify_picking import portrait_viewport

HERE = Path(__file__).resolve().parent
ROOT = HERE.parent.parent
# Same candidate test; preserve the first offscreen-setup failure.
EVIDENCE = ROOT / "target/m11-gpu/yt079-20261014/browser/page-module-hook"


def main():
    assert os.environ.get("YT075_FIREFOX_HEADFUL_X11") == "1", "Use the established headful X11 recipe"
    assert not EVIDENCE.exists(), "Existing YT-079 evidence: never overwrite or retry automatically"
    firefox, driver = shutil.which("firefox"), shutil.which("geckodriver")
    binary = HERE / "target/debug/yetlet-webgpu-scene-island"
    assets = ROOT / "target/webgpu-scene-island-dist"
    snap_common = Path.home() / "snap/firefox/common"
    assert firefox and driver and binary.is_file() and snap_common.is_dir(), "Existing Firefox fixture unavailable"
    assert (assets / "manifest.json").is_file(), "Build first; this test never rebuilds"
    EVIDENCE.mkdir(parents=True, exist_ok=False)
    source_paths = [HERE / name for name in (
        "Cargo.toml", "Cargo.lock", "src/lib.rs", "src/browser.rs", "src/server.rs", "src/main.rs",
        "src/scene.wgsl", "scene-loader.js", "counter-loader.js", "build.py", "verify_lifecycle.py",
        "verify_firefox.py", "verify_browser.py", "verify_picking.py", "comparison-loader.js")]
    candidate = ROOT / "crates/yetlet-wgpu"
    source_paths += [candidate / "Cargo.toml", candidate / "Cargo.lock", candidate / "README.md",
                     *sorted((candidate / "src").glob("*")), HERE / "verify_candidate.py", ROOT / "Cargo.toml"]
    manifest = json.loads((assets / "manifest.json").read_text())
    assert manifest.get("candidate_sources"), "Source-bound extracted candidate manifest required"
    for path, expected in manifest["candidate_sources"].items():
        assert hashlib.sha256((ROOT / path).read_bytes()).hexdigest() == expected, "Stale candidate source/build"
    assert "crates/yetlet-wgpu/src/browser.rs" in manifest["candidate_sources"]
    asset_paths = [assets / "manifest.json", assets / "assets" / manifest["scene"]["digest"] / "comparison-loader.js"]
    for key, module in (("scene", "scene"), ("counter", "yetlet_dashboard_counter")):
        directory = assets / "assets" / manifest[key]["digest"]
        asset_paths.extend(directory / name for name in (f"{key}-loader.js", f"{module}.js", f"{module}_bg.wasm"))
    inputs = source_paths + asset_paths + [binary, ROOT / "Cargo.lock"]
    hashes = {str(p.relative_to(ROOT)): hashlib.sha256(p.read_bytes()).hexdigest() for p in inputs}
    (EVIDENCE / "input-hashes.json").write_text(json.dumps(hashes, indent=2) + "\n")
    report = {"task_id": "YT-079", "status": "running", "checks": [], "snapshots": [],
              "pixels": [], "manifest": manifest, "input_hashes": hashes,
              "profile_changes": ["dom.webgpu.enabled=true (temporary only)"],
              "input_policy": "WebDriver mouse/keyboard; touch/cancel/scroll and DPR overrides are explicitly simulated; no physical mobile or screenreader speech claim",
              "browser_process_bound_seconds": 240, "test_deadline_seconds": 180,
              "pixel_tolerance_per_channel": 25,
              "counters_policy": "Application counters/explicit buffer bytes, not measured VRAM",
              "owned": {}, "cleanup": {}}
    server = gecko = session = None
    driver_url = ""
    tmp_path = None
    def timeout(_signal, _frame):
        raise TimeoutError("YT-079 180-second focused test deadline")

    previous_handler = signal.signal(signal.SIGALRM, timeout)
    signal.setitimer(signal.ITIMER_REAL, 180)
    try:
        with tempfile.TemporaryDirectory(prefix="yt079-ff-", dir=snap_common) as tmp:
            tmp_path = Path(tmp)
            profile_root = tmp_path / "driver-profiles"
            profile_root.mkdir()
            asset_root = tmp_path / "assets-root"
            shutil.copytree(assets, asset_root)
            server_port, driver_port = port_number(), port_number()
            assert server_port != driver_port, "Distinct owned ports required"
            base, driver_url = f"http://127.0.0.1:{server_port}", f"http://127.0.0.1:{driver_port}"
            env = dict(os.environ, TMPDIR=str(profile_root))
            env.setdefault("DISPLAY", ":0")
            env.setdefault("XAUTHORITY", str(Path.home() / ".Xauthority"))
            report["owned"].update(server_port=server_port, driver_port=driver_port, profile_root=str(profile_root))
            with (EVIDENCE / "server.log").open("w") as slog, (EVIDENCE / "geckodriver.log").open("w") as glog:
                try:
                    server = subprocess.Popen([str(binary)], cwd=ROOT,
                        env=dict(env, YETLET_SCENE_LISTEN=f"127.0.0.1:{server_port}", YETLET_SCENE_ASSET_ROOT=str(asset_root)),
                        stdout=slog, stderr=subprocess.STDOUT, start_new_session=True)
                    report["owned"]["server_pid"] = server.pid
                    wait_until(lambda: http(base + "/healthz")[0] == 200, "owned scene server", 20)
                    code, headers, html = http(base + "/scene?q=done&page=1")
                    assert code == 200 and b"40 matches; page 2 of 4" in html
                    assert b'<html lang="en">' in html and "unsafe-inline" not in headers["content-security-policy"]
                    gecko = subprocess.Popen([driver, "--host", "127.0.0.1", "--port", str(driver_port), "--log", "info"],
                        cwd=ROOT, env=env, stdout=glog, stderr=subprocess.STDOUT, start_new_session=True)
                    report["owned"]["geckodriver_pid"] = gecko.pid
                    wait_until(lambda: http(driver_url + "/status")[0] == 200, "owned geckodriver", 20)
                    response = request("POST", driver_url, "/session", {"capabilities": {"alwaysMatch": {
                        "browserName": "firefox", "webSocketUrl": True, "moz:firefoxOptions": {"args": [], "prefs": {"dom.webgpu.enabled": True}}
                    }}})
                    session = response["sessionId"]
                    capabilities = response.get("capabilities", {})
                    report["capabilities"] = {k: capabilities.get(k) for k in ("browserName", "browserVersion", "platformName", "moz:processID", "moz:profile")}
                    assert capabilities.get("browserVersion", "").startswith("158."), "Selected Firefox 158 required"
                    prefix = f"/session/{session}"
                    request("POST", driver_url, prefix + "/timeouts", {"script": 25000, "pageLoad": 30000})

                    def evaluate(script):
                        return request("POST", driver_url, prefix + "/execute/sync", {"script": script, "args": []})

                    def async_eval(script):
                        return request("POST", driver_url, prefix + "/execute/async", {"script": script, "args": []})

                    def frames():
                        async_eval("const done=arguments[arguments.length-1];requestAnimationFrame(()=>requestAnimationFrame(()=>requestAnimationFrame(()=>done(null))));")

                    def observe(selector, label):
                        # Resolve a fresh WebDriver target and live DOM/state every time.
                        element = request("POST", driver_url, prefix + "/element", {"using": "css selector", "value": selector})
                        key = element[ELEMENT_KEY]
                        rect = request("GET", driver_url, prefix + f"/element/{key}/rect")
                        data = evaluate("""const t=document.querySelector(SELECTOR), c=document.getElementById('scene-canvas');
                            return {url:location.href, connected:t.isConnected, targetText:t.textContent,
                            label:t.getAttribute('aria-label'), describedBy:t.getAttribute('aria-describedby'),
                            disabled:t.disabled??false, scene:window.__yt075Scene?.snapshot()??null,
                            selection:window.__yt078Selection?.snapshot()??null,
                            detail:document.getElementById('scene-selection')?.textContent??null,
                            href:document.getElementById('scene-detail-link')?.getAttribute('href')??null,
                            records:document.querySelectorAll('#records li').length,
                            counter:document.getElementById('counter-value')?.textContent??null,
                            css:c?{left:c.getBoundingClientRect().left,top:c.getBoundingClientRect().top,width:c.getBoundingClientRect().width,height:c.getBoundingClientRect().height}:null};""".replace("SELECTOR", json.dumps(selector)))
                        assert data["connected"] and not data["disabled"], "Fresh target must be connected/enabled"
                        report["snapshots"].append({"label": label, "selector": selector, "webdriver_target": key, "rect": rect, "value": data})
                        return key, data

                    def action(selector, label, operation):
                        key, before = observe(selector, label + " before")
                        value = operation(key, before)
                        frames()
                        _, after = observe(selector, label + " after")
                        report["checks"].append({"name": label, "action_result": value, "status": "executed"})
                        return after

                    def click(selector, label):
                        return action(selector, label, lambda key, _: request("POST", driver_url, prefix + f"/element/{key}/click", {}))

                    def script(selector, label, code):
                        return action(selector, label, lambda _key, _before: evaluate(code))

                    def keys(selector, label, value):
                        return action(selector, label, lambda key, _: request("POST", driver_url, prefix + f"/element/{key}/value", {"text": value, "value": list(value)}))

                    def fresh(label):
                        action("body", "fresh document " + label, lambda _key, _before: request("POST", driver_url, prefix + "/url", {"url": base + "/scene?yt079=" + label}))
                        wait_until(lambda: evaluate("return !!window.__yt078Selection && document.readyState==='complete'"), "selection loader", 20)
                        script("#scene-canvas", "scroll canvas into view " + label, "document.getElementById('scene-canvas').scrollIntoView({block:'center'})")

                    def ready():
                        # Native click can scroll the activation button above a narrow canvas.
                        # Restore actual visibility before expecting the demand renderer to draw.
                        script("#scene-canvas", "restore visible canvas before readiness", "document.getElementById('scene-canvas').scrollIntoView({block:'center'})")
                        wait_until(lambda: evaluate("return window.__yt075Scene.snapshot().state==='ready' && window.__yt075Scene.snapshot().draws>=1"), "drawn GPU ready", 35)
                        frames()

                    def pointer(id, label):
                        script("#scene-canvas", "visible input target " + label, "document.getElementById('scene-canvas').scrollIntoView({block:'center'})")
                        def send(key, before):
                            css = before["css"]
                            x = round(css["left"] + ((id - 1) % 12 + .5) * css["width"] / 12)
                            y = round(css["top"] + ((id - 1) // 12 + .5) * css["height"] / 10)
                            return request("POST", driver_url, prefix + "/actions", {"actions": [{"type": "pointer", "id": "yt079-mouse", "parameters": {"pointerType": "mouse"}, "actions": [
                                {"type": "pointerMove", "duration": 0, "origin": "viewport", "x": x, "y": y},
                                {"type": "pointerDown", "button": 0}, {"type": "pointerUp", "button": 0}]}]})
                        result = action("#scene-canvas", label, send)
                        assert result["selection"]["selectedRecord"] == id and f"{id:03}" in result["detail"]
                        assert result["href"] == f"/scene?record={id}"

                    def pixels(label, colors=None):
                        script("#scene-canvas", "fresh hardware draw " + label, "window.__yt075Scene.draw()")
                        async_eval("const done=arguments[arguments.length-1];window.__yt075Scene.completion().then(()=>requestAnimationFrame(()=>requestAnimationFrame(()=>done(null))));")
                        key, before = observe("#scene-canvas", "hardware pixels " + label + " before")
                        png = base64.b64decode(request("GET", driver_url, prefix + f"/element/{key}/screenshot"))
                        (EVIDENCE / f"{label}.png").write_bytes(png)
                        width, height, channels, rows = png_pixels(png)
                        assert abs(width - before["css"]["width"]) <= 1 and abs(height - before["css"]["height"]) <= 1
                        # Established mapping: a full-white top row is a capture seam,
                        # never search colors or change the predeclared tolerance 25.
                        top = int(all(list(rows[0][x * channels:x * channels + 3]) == [255, 255, 255] for x in range(width)))
                        mismatches = []
                        for index in range(120):
                            x = int((index % 12 + .5) * width / 12)
                            y = int((index // 12 + .5) * height / 10) + top
                            actual = list(rows[y][x * channels:x * channels + 3])
                            expected = ([255, 0, 0], [0, 255, 0], [0, 0, 255])[index % 3 if colors is None else colors[index]]
                            if any(abs(a-b) > 25 for a, b in zip(actual, expected)):
                                mismatches.append({"id": index + 1, "capture": [x, y], "actual": actual, "expected": expected})
                        black = list(rows[top + 2][2 * channels:2 * channels + 3])
                        report["pixels"].append({"label": label, "width": width, "height": height, "capture_canvas_origin": [0, top],
                            "verified": 120 - len(mismatches), "mismatches": mismatches, "interior_black": black, "sha256": hashlib.sha256(png).hexdigest()})
                        observe("#scene-canvas", "hardware pixels " + label + " after")
                        assert not mismatches and black == [0, 0, 0], "Actual hardware canvas proof failed"

                    fresh("candidate")
                    action("body", "desktop viewport", lambda _key, _before: request("POST", driver_url, prefix + "/window/rect", {"width": 1100, "height": 1000}))
                    adapter = async_eval("const done=arguments[arguments.length-1];navigator.gpu.requestAdapter().then(a=>done(a?{fallback:a.isFallbackAdapter??a.info?.isFallbackAdapter??null,info:a.info?{vendor:a.info.vendor,architecture:a.info.architecture,device:a.info.device,description:a.info.description}:null}:null));")
                    report["adapter"] = adapter
                    assert adapter and not adapter["fallback"], "Actual non-fallback hardware adapter required"
                    click("#activate-scene", "explicit extracted renderer activation")
                    ready()
                    pixels("initial")
                    pointer(1, "actual mouse known slot 1")
                    click("#scene-navigate", "focus keyboard alternative")
                    keys("#scene-navigate", "keyboard Home", "\ue011")
                    keys("#scene-navigate", "keyboard Right", "\ue014")
                    chosen = keys("#scene-navigate", "keyboard Enter selects slot 2", "\ue007")
                    assert chosen["selection"]["selectedRecord"] == 2 and chosen["href"] == "/scene?record=2"
                    assert chosen["describedBy"] == "selection-help"
                    action("#scene-canvas", "prepare app-owned replacement", lambda _key, _before: async_eval("const done=arguments[arguments.length-1];window.__yt075Scene.records(120,1).then(data=>{window.__yt079Data=data;done(null)});"))
                    script("#scene-canvas", "capture old-data pick", "const b=document.getElementById('scene-canvas').getBoundingClientRect();window.__yt079Pick=window.__yt078Selection.capturePick(b.left+b.width/24,b.top+b.height/20)")
                    replaced = script("#scene-canvas", "replace and reject stale pick", "window.__yt075Scene.replace(window.__yt079Data);return window.__yt078Selection.commitPick(window.__yt079Pick)")
                    assert replaced["selection"]["selectedRecord"] is None
                    assert replaced["selection"]["rejectedPicks"] > chosen["selection"]["rejectedPicks"]
                    pixels("replacement", [(i+1)%3 for i in range(120)])
                    before_metrics = evaluate("return window.__yt075Scene.snapshot().metrics")
                    action("#scene-canvas", "prepare 12-record status update", lambda _key, _before: async_eval("const done=arguments[arguments.length-1];window.__yt075Scene.patch(window.__yt079Data,0,12,1,1).then(data=>{window.__yt079Patch=data;done(null)});"))
                    updated = script("#scene-canvas", "upload bounded status patch", "window.__yt075Scene.update(0,window.__yt079Patch)")
                    pixels("updated", [(i+(2 if i<12 else 1))%3 for i in range(120)])
                    after_metrics = evaluate("return window.__yt075Scene.snapshot().metrics")
                    assert after_metrics[0] - before_metrics[0] == 12*32
                    assert after_metrics[1] - before_metrics[1] == 1
                    assert after_metrics[9] == 120*32 and after_metrics[11] == 1
                    pointer(120, "known pick after update")
                    action("#scene-canvas", "390 CSS-pixel narrow viewport via proven BiDi seam", lambda _key, before: portrait_viewport(capabilities["webSocketUrl"], before["url"]))
                    script("#scene-canvas", "narrow responsive target", "document.getElementById('scene-canvas').scrollIntoView({block:'center'})")
                    ready()
                    report["narrow_layout"] = evaluate("return {innerWidth,innerHeight,scrollWidth:document.documentElement.scrollWidth,records:document.querySelectorAll('#records li').length}")
                    assert report["narrow_layout"]["innerWidth"] == 390 and report["narrow_layout"]["scrollWidth"] <= 390 and report["narrow_layout"]["records"] == 12
                    pointer(3, "narrow known pick")
                    click("#dispose-scene", "candidate teardown")
                    _, disposed = observe("#scene-canvas", "disposed counters")
                    assert disposed["scene"]["lastRetired"]["resourcesReleased"]
                    assert disposed["scene"]["ownedSurfaces"] == disposed["scene"]["ownedBufferBytes"] == disposed["scene"]["observers"] == disposed["scene"]["listeners"] == 0
                    assert not disposed["scene"]["rafPending"]

                    fresh("loss")
                    script("#activate-scene", "capture actual acquired devices for controlled loss", """window.__yt079Devices=[];
                        const request=GPUAdapter.prototype.requestDevice;
                        GPUAdapter.prototype.requestDevice=async function(...args){const d=await request.apply(this,args);window.__yt079Devices.push(d);return d;};""")
                    click("#activate-scene", "activate loss sample")
                    ready()
                    _, before_loss = observe("#scene-canvas", "before controlled loss")
                    script("#scene-canvas", "CONTROLLED real device destroy", "window.__yt079Devices[0].destroy()")
                    wait_until(lambda: evaluate("return window.__yt075Scene.snapshot().state==='lost'"), "controlled loss fallback", 20)
                    _, lost = observe("#scene-canvas", "lost fallback counters")
                    assert lost["scene"]["fault"] == "device-lost" and lost["scene"]["ownedSurfaces"] == lost["scene"]["ownedBufferBytes"] == 0
                    assert lost["scene"]["starts"] == before_loss["scene"]["starts"] and lost["records"] == 12
                    assert lost["scene"]["listeners"] == lost["scene"]["observers"] == 0 and not lost["scene"]["rafPending"]
                    count = click("#counter-increment", "counter survives lost renderer")
                    assert count["counter"] == "3"
                    click("#activate-scene", "one explicit loss recovery")
                    ready()
                    _, recovered = observe("#scene-canvas", "fresh recovery generation")
                    assert recovered["scene"]["generation"] > before_loss["scene"]["generation"] and recovered["scene"]["starts"] == before_loss["scene"]["starts"]+1
                    pixels("recovered")
                    click("#dispose-scene", "recovery teardown")
                    report["loss_policy"] = "Controlled real device.destroy and real loss promise; not physical hardware failure"

                    fresh("completion-retirement")
                    # Hold a real settled queue completion at its browser boundary.
                    script("#activate-scene", "install bounded pending-completion/resource sample", """window.__yt079Held={held:false,destroyed:0,buffers:0,unconfigured:0,frees:0};
                        const request=GPUAdapter.prototype.requestDevice;
                        GPUAdapter.prototype.requestDevice=async function(...args){const first=!window.__yt079Held.deviceSeen;
                            const d=await request.apply(this,args);if(!first)return d;window.__yt079Held.deviceSeen=true;
                            const destroy=d.destroy.bind(d);d.destroy=()=>{window.__yt079Held.destroyed++;destroy()};
                            const create=d.createBuffer.bind(d);d.createBuffer=(...args)=>{const b=create(...args),destroy=b.destroy.bind(b);
                                b.destroy=()=>{window.__yt079Held.buffers++;destroy()};return b;};
                            const complete=d.queue.onSubmittedWorkDone.bind(d.queue);d.queue.onSubmittedWorkDone=async()=>{
                                await complete();return new Promise(resolve=>{window.__yt079Held.held=true;window.__yt079Held.release=resolve;});};return d;};
                        const unconfigure=GPUCanvasContext.prototype.unconfigure;GPUCanvasContext.prototype.unconfigure=function(){window.__yt079Held.unconfigured++;return unconfigure.call(this)};""")
                    click("#activate-scene", "activate completion sample")
                    ready()
                    # The WebDriver script world may have a distinct module cache. Install
                    # this single-purpose hook in the PAGE module world through a temporary
                    # same-origin external script. No inline script/CSP bypass or product edit.
                    hook_text = "import {SceneOwner} from './scene.js'; const free=SceneOwner.prototype.free; SceneOwner.prototype.free=function(){window.__yt079Held.frees++;return free.call(this)}; window.__yt079Held.hookInstalled=true;\n"
                    hook_path = asset_root / "assets" / manifest["scene"]["digest"] / "yt079-free-hook.js"
                    hook_path.write_text(hook_text)
                    hook_url = "/scene-assets/" + manifest["scene"]["digest"] + "/yt079-free-hook.js"
                    script("#scene-canvas", "observe page-world generated wrapper freeing", "const s=document.createElement('script');s.type='module';s.src=" + json.dumps(hook_url) + ";document.head.append(s)")
                    wait_until(lambda: evaluate("return window.__yt079Held.hookInstalled===true"), "owned page-world hook loaded", 15)
                    report["wrapper_hook"] = {"scope": "temporary same-origin page module, not production asset", "sha256": hashlib.sha256(hook_text.encode()).hexdigest(), "strict_csp_unchanged": True}
                    script("#scene-canvas", "start retained completion future", "window.__yt075Scene.completion().then(()=>window.__yt079Completion='resolved',error=>window.__yt079Completion=String(error))")
                    wait_until(lambda: evaluate("return window.__yt079Held.held"), "held real queue completion", 20)
                    _, pending_owner = observe("#scene-canvas", "pending completion owner")
                    assert pending_owner["scene"]["pendingCompletions"] == 1
                    click("#dispose-scene", "retire owner during completion")
                    assert evaluate("return window.__yt079Held.frees===0 && window.__yt079Held.destroyed===1 && window.__yt079Held.buffers===1 && window.__yt079Held.unconfigured===1 && !window.__yt075Scene.snapshot().lastRetired.resourcesReleased")
                    click("#activate-scene", "replacement while old completion pending")
                    ready()
                    _, replacement = observe("#scene-canvas", "replacement generation ready")
                    assert replacement["scene"]["generation"] > pending_owner["scene"]["generation"]
                    script("#scene-canvas", "settle retired completion", "window.__yt079Held.release()")
                    wait_until(lambda: evaluate("return window.__yt079Held.frees===1 && !!window.__yt079Completion"), "old wrapper freed once", 15)
                    assert evaluate("return window.__yt079Completion.includes('retired') && window.__yt079Held.destroyed===1 && window.__yt079Held.buffers===1 && window.__yt079Held.unconfigured===1 && window.__yt075Scene.snapshot().lastRetired.resourcesReleased")
                    _, settled = observe("#scene-canvas", "replacement unaffected by stale completion")
                    assert settled["scene"]["generation"] == replacement["scene"]["generation"] and settled["scene"]["state"] == "ready"
                    pixels("after-retired-completion")
                    report["completion_retirement"] = evaluate("const h=window.__yt079Held;return {held:h.held,destroyed:h.destroyed,buffers:h.buffers,unconfigured:h.unconfigured,frees:h.frees,settlement:window.__yt079Completion}")
                    click("#dispose-scene", "completion replacement teardown")
                    assert evaluate("return window.__yt079Held.frees===2 && window.__yt075Scene.snapshot().lastRetired.resourcesReleased")

                    fresh("unavailable")
                    script("#activate-scene", "INJECT unavailable GPU capability", "window.__yt079GPU=navigator.gpu;Object.defineProperty(navigator,'gpu',{value:undefined,configurable:true})")
                    click("#activate-scene", "unavailable fallback")
                    wait_until(lambda: evaluate("return window.__yt075Scene.snapshot().state==='unavailable'"), "unavailable state", 15)
                    click("#scene-navigate", "fallback keyboard")
                    keys("#scene-navigate", "fallback End", "\ue010")
                    fallback = keys("#scene-navigate", "fallback Enter", "\ue007")
                    assert fallback["selection"]["selectedRecord"] == 120 and fallback["href"] == "/scene?record=120" and fallback["records"] == 12
                    count = click("#counter-increment", "independent counter without GPU")
                    assert count["counter"] == "3"
                    script("#activate-scene", "restore injected capability", "Object.defineProperty(navigator,'gpu',{value:window.__yt079GPU,configurable:true})")
                    click("#activate-scene", "explicit unavailable recovery")
                    ready()
                    click("#dispose-scene", "unavailable recovery teardown")
                    saved_manifest = (asset_root / "manifest.json").read_bytes()
                    (asset_root / "manifest.json").unlink()
                    try:
                        nojs = action("#query", "scriptless actual SSR search/page/detail", lambda _key, _before: request("POST", driver_url, prefix + "/url", {"url": base + "/scene?q=done&page=1&record=38"}))
                        assert nojs["records"] == 12 and nojs["counter"] == "2"
                        assert evaluate("return !document.querySelector('script') && document.getElementById('matches').textContent.includes('40 matches; page 2 of 4') && document.getElementById('detail').textContent.includes('Work record 038')")
                        opened = click("#records a", "scriptless real record navigation")
                        assert "record=38" in opened["url"] and evaluate("return document.getElementById('detail').textContent.includes('Work record 038')")
                        report["scriptless_ssr"] = "Actual no-script HTML search/page/detail navigation, not a JS-disabled browser preference"
                    finally:
                        (asset_root / "manifest.json").write_bytes(saved_manifest)
                    report["status"] = "passed_yt079_candidate"
                finally:
                    signal.setitimer(signal.ITIMER_REAL, 0)
                    if session is not None:
                        try:
                            request("DELETE", driver_url, f"/session/{session}")
                            report["cleanup"]["session_deleted"] = True
                        except Exception as exc:
                            report["cleanup"]["session_delete_error"] = str(exc)[:250]
                    for label, proc in (("geckodriver", gecko), ("server", server)):
                        try:
                            stop_owned_process(proc, process_group=True)
                            report["cleanup"][label + "_exit"] = proc.returncode if proc else None
                        except Exception as exc:
                            report["cleanup"][label + "_error"] = str(exc)[:250]
    except BaseException as error:
        report["status"] = "failed"
        report["error"] = type(error).__name__ + ": " + str(error)[:600]
    finally:
        signal.setitimer(signal.ITIMER_REAL, 0)
        signal.signal(signal.SIGALRM, previous_handler)
        report["cleanup"]["tmp_profile_removed"] = tmp_path is None or not tmp_path.exists()
        report["cleanup"]["processes_reaped"] = all(p is None or p.poll() is not None for p in (server, gecko))
        browser_pid = report.get("capabilities", {}).get("moz:processID")
        if browser_pid:
            try:
                os.kill(browser_pid, 0)
                report["cleanup"]["browser_exited"] = False
            except ProcessLookupError:
                report["cleanup"]["browser_exited"] = True
        else:
            report["cleanup"]["browser_exited"] = session is None
        report["cleanup"]["ports_closed"] = {}
        for key in ("server_port", "driver_port"):
            if key in report["owned"]:
                try:
                    with socket.create_connection(("127.0.0.1", report["owned"][key]), timeout=.3):
                        report["cleanup"]["ports_closed"][key] = False
                except OSError:
                    report["cleanup"]["ports_closed"][key] = True
        report["inputs_unchanged"] = all(p.is_file() and hashlib.sha256(p.read_bytes()).hexdigest() == hashes[str(p.relative_to(ROOT))] for p in inputs)
        for name in ("geckodriver.log", "server.log"):
            path = EVIDENCE / name
            if path.exists(): report[name + "_sha256"] = hashlib.sha256(path.read_bytes()).hexdigest()
        (EVIDENCE / "result.json").write_text(json.dumps(report, indent=2) + "\n")
        print(json.dumps({k: report.get(k) for k in ("status", "error", "checks", "pixels", "cleanup", "inputs_unchanged")}))
    return 0 if (report["status"] == "passed_yt079_candidate" and report["inputs_unchanged"]
                 and report["cleanup"].get("session_deleted") and report["cleanup"]["tmp_profile_removed"]
                 and report["cleanup"]["processes_reaped"] and report["cleanup"]["browser_exited"]
                 and all(report["cleanup"]["ports_closed"].values()) and not any(k.endswith("_error") for k in report["cleanup"])) else 1


if __name__ == "__main__":
    raise SystemExit(main())
