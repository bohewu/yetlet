#!/usr/bin/env python3
"""One YT-076 real-Wasm Firefox X11 lifecycle test; never reruns YT-075 receipts.

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
import socket
import subprocess
import tempfile

from verify_firefox import ELEMENT_KEY, request
from verify_browser import png_pixels
from smoke_browser import http, port_number, stop_owned_process, wait_until

HERE = Path(__file__).resolve().parent
ROOT = HERE.parent.parent
EVIDENCE = ROOT / "target/m11-gpu/yt076-20261014/browser"


def main():
    assert os.environ.get("YT075_FIREFOX_HEADFUL_X11") == "1", "Use the established headful X11 recipe"
    assert not EVIDENCE.exists(), "Existing YT-076 evidence: never overwrite or retry automatically"
    firefox, driver = shutil.which("firefox"), shutil.which("geckodriver")
    binary = HERE / "target/debug/yetlet-webgpu-scene-island"
    assets = ROOT / "target/webgpu-scene-island-dist"
    snap_common = Path.home() / "snap/firefox/common"
    assert firefox and driver and binary.is_file() and snap_common.is_dir(), "Existing Firefox fixture unavailable"
    assert (assets / "manifest.json").is_file(), "Build first; this test never rebuilds"
    EVIDENCE.mkdir(parents=True, exist_ok=False)
    source_paths = [HERE / name for name in (
        "Cargo.toml", "Cargo.lock", "src/lib.rs", "src/browser.rs", "src/server.rs",
        "src/scene.wgsl", "scene-loader.js", "counter-loader.js", "build.py", "verify_lifecycle.py",
        "verify_firefox.py", "verify_browser.py")]
    manifest = json.loads((assets / "manifest.json").read_text())
    asset_paths = [assets / "manifest.json"]
    for key, module in (("scene", "scene"), ("counter", "yetlet_dashboard_counter")):
        directory = assets / "assets" / manifest[key]["digest"]
        asset_paths.extend(directory / name for name in (f"{key}-loader.js", f"{module}.js", f"{module}_bg.wasm"))
    inputs = source_paths + asset_paths + [binary, ROOT / "Cargo.lock"]
    hashes = {str(p.relative_to(ROOT)): hashlib.sha256(p.read_bytes()).hexdigest() for p in inputs}
    (EVIDENCE / "input-hashes.json").write_text(json.dumps(hashes, indent=2) + "\n")
    report = {"task_id": "YT-076", "status": "running", "checks": [], "snapshots": [],
              "pixels": [], "manifest": manifest, "input_hashes": hashes,
              "profile_changes": ["dom.webgpu.enabled=true (temporary only)"],
              "fault_policy": "Controlled real device.destroy() triggers the real loss promise; injected loss, not hardware failure",
              "counters_policy": "Application counters/explicit buffer bytes, not measured VRAM",
              "owned": {}, "cleanup": {}}
    server = gecko = session = None
    driver_url = ""
    tmp_path = None
    try:
        with tempfile.TemporaryDirectory(prefix="yt076-ff-", dir=snap_common) as tmp:
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
                        "browserName": "firefox", "moz:firefoxOptions": {"args": [], "prefs": {"dom.webgpu.enabled": True}}
                    }}})
                    session = response["sessionId"]
                    capabilities = response.get("capabilities", {})
                    report["capabilities"] = {k: capabilities.get(k) for k in ("browserName", "browserVersion", "platformName", "moz:processID", "moz:profile")}
                    prefix = f"/session/{session}"
                    request("POST", driver_url, prefix + "/timeouts", {"script": 25000, "pageLoad": 30000})

                    def evaluate(script):
                        return request("POST", driver_url, prefix + "/execute/sync", {"script": script, "args": []})

                    def async_eval(script):
                        return request("POST", driver_url, prefix + "/execute/async", {"script": script, "args": []})

                    def snap(label):
                        data = evaluate("""return {scene:window.__yt075Scene.snapshot(),
                            records:document.querySelectorAll('#records li').length,
                            counter:document.getElementById('counter-value').textContent,
                            text:document.getElementById('scene-status').textContent}""")
                        report["snapshots"].append({"label": label, "value": data})
                        return data["scene"]

                    def frames():
                        async_eval("const done=arguments[arguments.length-1];requestAnimationFrame(()=>requestAnimationFrame(()=>requestAnimationFrame(()=>requestAnimationFrame(()=>done(null)))));")

                    def state(name):
                        wait_until(lambda: evaluate(f"return window.__yt075Scene.snapshot().state==={json.dumps(name)}"), name + " state", 40)
                        frames()
                        assert snap(name)["state"] == name

                    def activate():
                        evaluate("document.getElementById('activate-scene').click()")
                        state("ready")

                    def check(name, **extra):
                        report["checks"].append({"name": name, "status": "passed", **extra})

                    def pixels(label):
                        # Demand a new real frame, then wait through presentation opportunities.
                        async_eval("""const done=arguments[arguments.length-1];window.__yt075Scene.draw();
                            requestAnimationFrame(()=>requestAnimationFrame(()=>done(null)));""")
                        element = request("POST", driver_url, prefix + "/element", {"using": "css selector", "value": "#scene-canvas"})
                        png = base64.b64decode(request("GET", driver_url, prefix + f"/element/{element[ELEMENT_KEY]}/screenshot"))
                        (EVIDENCE / f"{label}.png").write_bytes(png)
                        width, height, channels, rows = png_pixels(png)
                        css = evaluate("const b=document.getElementById('scene-canvas').getBoundingClientRect();return {width:b.width,height:b.height}")
                        assert (width, height) == (round(css["width"]), round(css["height"])), "Unexpected capture shape"
                        mismatches = []
                        for index in range(120):
                            x, y = int((index % 12 + .5) * width / 12), int((index // 12 + .5) * height / 10)
                            actual = list(rows[y][x * channels:x * channels + 3])
                            expected = ([255, 0, 0], [0, 255, 0], [0, 0, 255])[index % 3]
                            if any(abs(a-b) > 25 for a, b in zip(actual, expected)):
                                mismatches.append({"id": index + 1, "pixel": actual})
                        background = [list(rows[2][2 * channels:2 * channels + 3]),
                                      list(rows[int(height / 20)][int(width / 12) * channels:int(width / 12) * channels + 3])]
                        proof = {"label": label, "width": width, "height": height, "centers_verified": 120-len(mismatches),
                                 "mismatches": mismatches[:8], "interior_black": background,
                                 "capture_top_edge": list(rows[0][:3]), "sha256": hashlib.sha256(png).hexdigest()}
                        report["pixels"].append(proof)
                        assert not mismatches and background == [[0, 0, 0], [0, 0, 0]], "Real canvas pixel proof failed"

                    def fresh(label):
                        request("POST", driver_url, prefix + "/url", {"url": base + "/scene?yt076=" + label})
                        wait_until(lambda: evaluate("return !!window.__yt075Scene && document.readyState==='complete'"), "scene loader", 20)
                        evaluate("document.getElementById('scene-canvas').scrollIntoView({block:'center'})")
                        frames()

                    fresh("lifecycle")
                    report["adapter"] = async_eval("""const done=arguments[arguments.length-1];
                        navigator.gpu.requestAdapter().then(a=>done(a?{available:true,fallback:a.isFallbackAdapter??a.info?.isFallbackAdapter??null}:null));""")
                    assert report["adapter"] and not report["adapter"]["fallback"], "Real non-fallback adapter required"
                    # Persist on window: WebDriver's globalThis sandbox does not survive eval calls.
                    evaluate("""window.__yt076Devices=[];window.__yt076Losses=0;
                        const request=GPUAdapter.prototype.requestDevice;
                        GPUAdapter.prototype.requestDevice=async function(...args){
                            const d=await request.apply(this,args);window.__yt076Devices.push(d);
                            d.lost.then(()=>window.__yt076Losses++);return d;};""")
                    activate()
                    before = snap("initial owner")
                    assert before["draws"] >= 1 and before["configurations"] == 1 and not before["rafPending"]
                    assert before["ownedBufferBytes"] == 3840 and before["observers"] == 3 and before["listeners"] == 3
                    evaluate("document.getElementById('activate-scene').click()")
                    frames()
                    assert snap("static idle")["draws"] == before["draws"]
                    assert snap("single owner")["starts"] == 1
                    check("single owner and demand-only static idle")
                    evaluate("""const c=document.getElementById('scene-canvas');c.style.width='450px';c.style.height='300px';
                        c.style.width='480px';c.style.height='320px';
                        for(let i=0;i<12;i++)window.__yt075Scene.draw();""")
                    frames()
                    resized = snap("coalesced resize")
                    assert resized["configurations"] == before["configurations"] + 1
                    assert resized["coalesced"] >= 11 and not resized["rafPending"]
                    evaluate("const c=document.getElementById('scene-canvas');for(let i=0;i<12;i++){c.style.width='480px';window.__yt075Scene.draw()}")
                    frames()
                    assert snap("same size")["configurations"] == resized["configurations"]
                    check("coalesced latest resize and repeated same-size configuration")
                    evaluate("Object.defineProperty(window,'devicePixelRatio',{value:3,configurable:true});window.dispatchEvent(new Event('resize'))")
                    frames()
                    dpr = snap("injected DPR 3 bounded to 2")
                    assert dpr["dpr"] == 2 and dpr["backing"] == {"width": min(960, dpr["deviceLimit"]), "height": min(640, dpr["deviceLimit"])}
                    pixels("resized-dpr")
                    check("CSS/backing/DPR distinction", injected_dpr=True)
                    evaluate("document.getElementById('scene-canvas').style.height='0px'")
                    state("suspended")
                    zero = snap("zero-sized target")
                    frames()
                    assert snap("zero idle")["draws"] == zero["draws"] and not zero["rafPending"]
                    evaluate("document.getElementById('scene-canvas').style.height='320px'")
                    state("ready")
                    check("zero-size suspension and resume")
                    evaluate("""window.__yt076Parent=document.getElementById('scene-canvas').parentNode;
                        window.__yt076Canvas=document.getElementById('scene-canvas');window.__yt076Canvas.remove();""")
                    state("suspended")
                    detached = snap("detached")
                    frames()
                    assert snap("detached idle")["draws"] == detached["draws"]
                    evaluate("window.__yt076Parent.append(window.__yt076Canvas);window.__yt076Canvas.scrollIntoView({block:'center'})")
                    state("ready")
                    check("detached target suspension and reattachment")
                    evaluate("document.getElementById('scene-canvas').style.marginTop='2000px';window.scrollTo(0,0)")
                    state("suspended")
                    offscreen = snap("offscreen")
                    frames()
                    assert snap("offscreen idle")["draws"] == offscreen["draws"]
                    evaluate("document.getElementById('scene-canvas').style.marginTop='0px';document.getElementById('scene-canvas').scrollIntoView({block:'center'})")
                    state("ready")
                    check("real intersection offscreen pause and resume")
                    # Real background-tab visibility event; retain the original page proof on window.
                    evaluate("""window.__yt076Hidden=[];window.__yt076Visibility=()=>{
                        if(document.hidden){const first=window.__yt075Scene.snapshot();
                            setTimeout(()=>window.__yt076Hidden.push({first,later:window.__yt075Scene.snapshot()}),150);}};
                        document.addEventListener('visibilitychange',window.__yt076Visibility);""")
                    original_window = request("GET", driver_url, prefix + "/window")
                    new_window = request("POST", driver_url, prefix + "/window/new", {"type": "tab"})["handle"]
                    request("POST", driver_url, prefix + "/window", {"handle": new_window})
                    async_eval("const done=arguments[arguments.length-1];setTimeout(()=>done(null),450)")
                    request("DELETE", driver_url, prefix + "/window")
                    request("POST", driver_url, prefix + "/window", {"handle": original_window})
                    state("ready")
                    hidden = evaluate("document.removeEventListener('visibilitychange',window.__yt076Visibility);return window.__yt076Hidden")
                    assert hidden and hidden[-1]["first"]["state"] == "suspended"
                    assert hidden[-1]["later"]["draws"] == hidden[-1]["first"]["draws"] and not hidden[-1]["later"]["rafPending"]
                    check("real background-tab visibility pause and resume", proof=hidden)
                    lost_before = snap("before controlled real device destruction")
                    evaluate("window.__yt076Devices[0].destroy()")
                    state("lost")
                    lost = snap("real loss promise")
                    assert lost["fault"] == "device-lost" and lost["ownedBufferBytes"] == lost["ownedSurfaces"] == 0
                    assert lost["cpuRecordBytes"] == lost_before["cpuRecordBytes"] == 3840
                    assert lost["observers"] == lost["listeners"] == 0 and not lost["rafPending"]
                    frames()
                    assert snap("loss idle")["starts"] == lost_before["starts"]
                    assert evaluate("return window.__yt076Losses") == 1
                    evaluate("document.getElementById('counter-increment').click()")
                    assert evaluate("return document.getElementById('counter-value').textContent") == "3"
                    assert evaluate("return document.querySelectorAll('#records li').length") == 12
                    activate()
                    recovered = snap("explicit recovery")
                    assert recovered["generation"] > lost_before["generation"] and recovered["starts"] == lost_before["starts"] + 1
                    assert recovered["ownedBufferBytes"] == 3840
                    pixels("recovered")
                    check("injected real device loss, retained HTML/CPU records, sibling and one explicit recovery", injected_loss=True, hardware_failure=False)
                    evaluate("document.getElementById('dispose-scene').click()")
                    disposed = snap("dispose")
                    assert disposed["state"] == "disposed" and disposed["ownedBufferBytes"] == disposed["ownedSurfaces"] == 0
                    assert disposed["listeners"] == disposed["observers"] == 0 and not disposed["rafPending"]
                    activate()
                    assert snap("remount")["generation"] > recovered["generation"]
                    evaluate("document.getElementById('dispose-scene').click()")
                    check("owned disposal and remount")
                    # Hold a real acquired device or real completed validation scope, then retire it.
                    for phase in ("acquisition", "validation"):
                        fresh("late_" + phase)
                        evaluate("""window.__yt076Held={calls:0,held:false,destroyed:0,buffersDestroyed:0};
                            const request=GPUAdapter.prototype.requestDevice;
                            GPUAdapter.prototype.requestDevice=async function(...args){
                                const first=++window.__yt076Held.calls===1;const d=await request.apply(this,args);
                                if(!first)return d;
                                const destroy=d.destroy.bind(d);d.destroy=()=>{window.__yt076Held.destroyed++;destroy()};
                                const create=d.createBuffer.bind(d);d.createBuffer=(...args)=>{
                                    const b=create(...args);const destroy=b.destroy.bind(b);
                                    b.destroy=()=>{window.__yt076Held.buffersDestroyed++;destroy()};return b;};
                                if(PHASE==='acquisition')return await new Promise(resolve=>{
                                    window.__yt076Held.held=true;window.__yt076Held.release=()=>resolve(d)});
                                const pop=d.popErrorScope.bind(d);d.popErrorScope=async()=>{
                                    const result=await pop();return await new Promise(resolve=>{
                                        window.__yt076Held.held=true;window.__yt076Held.release=()=>resolve(result)});};
                                return d;};""".replace("PHASE", json.dumps(phase)))
                        evaluate("document.getElementById('activate-scene').click()")
                        wait_until(lambda: evaluate("return window.__yt076Held.held"), "held real " + phase, 35)
                        evaluate("document.getElementById('activate-scene').click()")
                        assert snap("pending duplicate " + phase)["starts"] == 1
                        evaluate("document.getElementById('dispose-scene').click();document.getElementById('activate-scene').click()")
                        state("ready")
                        replacement = snap("replacement " + phase)
                        evaluate("window.__yt076Held.release()")
                        wait_until(lambda: evaluate("return window.__yt076Held.destroyed===1 && window.__yt075Scene.snapshot().pendingInitializations===0"), "late resource destruction", 25)
                        after = snap("late completion " + phase)
                        assert after["generation"] == replacement["generation"] and after["starts"] == 2
                        assert after["configurations"] == replacement["configurations"] and after["state"] == "ready"
                        assert after["lateSettled"] == 1
                        assert evaluate("return window.__yt076Held.buffersDestroyed") == (1 if phase == "validation" else 0)
                        evaluate("document.getElementById('counter-increment').click()")
                        assert evaluate("return document.getElementById('counter-value').textContent") == "3"
                        evaluate("document.getElementById('dispose-scene').click()")
                        final = snap("final teardown " + phase)
                        assert final["lastRetired"]["resourcesReleased"] and final["pendingInitializations"] == 0
                        assert final["ownedBufferBytes"] == final["listeners"] == final["observers"] == 0
                        check("late real " + phase + " result destroyed without replacing current owner", controlled_hold=True)
                    report["status"] = "passed_yt076_lifecycle"
                finally:
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
    return 0 if (report["status"] == "passed_yt076_lifecycle" and report["inputs_unchanged"]
                 and report["cleanup"].get("session_deleted") and report["cleanup"]["tmp_profile_removed"]
                 and report["cleanup"]["processes_reaped"] and report["cleanup"]["browser_exited"]
                 and all(report["cleanup"]["ports_closed"].values()) and not any(k.endswith("_error") for k in report["cleanup"])) else 1


if __name__ == "__main__":
    raise SystemExit(main())
