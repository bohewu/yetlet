#!/usr/bin/env python3
"""YT-075 Firefox Linux experimental WebGPU browser acceptance.

Runs the unchanged YT-075 native SSR server and actual scene Wasm asset bundle,
using a disposable Firefox profile with only dom.webgpu.enabled=true.
No existing user profile, host preference, GPU blocklist, or sandbox is modified.
"""
from __future__ import annotations

import base64
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile
from urllib.error import HTTPError
from urllib.request import ProxyHandler, Request, build_opener

from verify_browser import png_pixels
from smoke_browser import http, port_number, stop_owned_process, wait_until

HERE = Path(__file__).resolve().parent
ROOT = HERE.parent.parent
HEADFUL_X11_DIAGNOSTIC = os.environ.get("YT075_FIREFOX_HEADFUL_X11") == "1"
EVIDENCE = ROOT / "target/m11-gpu/yt075-20261014" / (
    "firefox-158-x11-window-owner" if HEADFUL_X11_DIAGNOSTIC else "firefox-158-optin-harnessfix1"
)
ELEMENT_KEY = "element-6066-11e4-a52e-4f735466cecf"
opener = build_opener(ProxyHandler({}))


def request(method: str, base: str, path: str, payload: object | None = None):
    body = json.dumps(payload).encode() if payload is not None else None
    req = Request(
        base + path,
        data=body,
        headers={"Content-Type": "application/json"},
        method=method,
    )
    try:
        with opener.open(req, timeout=20) as response:
            data = json.loads(response.read())
    except HTTPError as error:
        data = json.loads(error.read())
        failure = data.get("value") or {}
        raise RuntimeError(
            f"WebDriver {method} {path}: {failure.get('error')}: "
            f"{failure.get('message', '')[:350]}"
        ) from error
    value = data.get("value")
    if isinstance(value, dict) and "error" in value:
        raise RuntimeError(f"WebDriver {method} {path}: {value.get('error')}: {value.get('message', '')[:250]}")
    return value


def main():
    assert not EVIDENCE.exists(), "Existing Firefox evidence: never overwrite/retry automatically"
    firefox = shutil.which("firefox")
    driver = shutil.which("geckodriver")
    binary = HERE / "target/debug/yetlet-webgpu-scene-island"
    canonical_assets = ROOT / "target/webgpu-scene-island-dist"
    assert firefox and driver and binary.is_file(), "Firefox/geckodriver/native fixture unavailable"
    assert (canonical_assets / "manifest.json").is_file(), "YT-075 original build assets absent"
    # The firefox Snap can only read profiles in its permitted home directories.
    snap_common = Path.home() / "snap/firefox/common"
    assert snap_common.is_dir(), "Firefox Snap data directory unavailable"
    EVIDENCE.mkdir(parents=True, exist_ok=False)
    report = {
        "task_id": "YT-075",
        "mode": "Firefox 158 Stable Linux, disposable dom.webgpu.enabled=true; "
                + ("headful X11 hardware compositor" if HEADFUL_X11_DIAGNOSTIC else "headless"),
        "status": "not_run",
        "actual_original_wasm_bundle": True,
        "snapshots": [],
        "faults": [],
        "profile_changes": ["dom.webgpu.enabled=true (temporary only)"],
        "cleanup": {},
    }
    server = gecko = None
    session = None
    driver_url = ""
    with tempfile.TemporaryDirectory(prefix="yt075-ff-", dir=snap_common) as tmp:
        tmp = Path(tmp)
        driver_profile_root = tmp / "driver-profiles"
        driver_profile_root.mkdir()
        # Isolate negative manifest/Wasm checks from immutable generated assets.
        asset_root = tmp / "assets-root"
        shutil.copytree(canonical_assets, asset_root)
        server_port, driver_port = port_number(), port_number()
        base = f"http://127.0.0.1:{server_port}"
        driver_url = f"http://127.0.0.1:{driver_port}"
        env = dict(os.environ, TMPDIR=str(driver_profile_root))
        env.setdefault("DISPLAY", ":0")
        env.setdefault("XAUTHORITY", str(Path.home() / ".Xauthority"))
        with (EVIDENCE / "server.log").open("w") as slog, (EVIDENCE / "geckodriver.log").open("w") as glog:
            try:
                server = subprocess.Popen(
                    [str(binary)], cwd=ROOT,
                    env=dict(env, YETLET_SCENE_LISTEN=f"127.0.0.1:{server_port}",
                             YETLET_SCENE_ASSET_ROOT=str(asset_root)),
                    stdout=slog, stderr=subprocess.STDOUT, start_new_session=True,
                )
                wait_until(lambda: http(base + "/healthz")[0] == 200, "Firefox baseline fixture server", 20)
                code, headers, document = http(base + "/scene?q=Done&page=1")
                assert code == 200 and b"40 matches; page 2 of 4" in document
                assert "unsafe-inline" not in headers["content-security-policy"]
                assert b'<html lang="en">' in document
                report["ssr"] = {"http": code, "english": True, "strict_csp": True, "search_and_pagination": True}

                gecko = subprocess.Popen(
                    [driver, "--host", "127.0.0.1", "--port", str(driver_port), "--log", "info"],
                    cwd=ROOT, env=env, stdout=glog, stderr=subprocess.STDOUT, start_new_session=True,
                )
                wait_until(lambda: http(driver_url + "/status")[0] == 200, "owned geckodriver", 20)
                capabilities = {
                    "capabilities": {
                        "alwaysMatch": {
                            "browserName": "firefox",
                            "moz:firefoxOptions": {
                                "args": [] if HEADFUL_X11_DIAGNOSTIC else ["-headless"],
                                "prefs": {"dom.webgpu.enabled": True}
                            },
                        }
                    }
                }
                response = request("POST", driver_url, "/session", capabilities)
                session = response["sessionId"]
                report["capabilities"] = {
                    k: response.get("capabilities", {}).get(k) for k in
                    ("browserName", "browserVersion", "platformName")
                }
                prefix = f"/session/{session}"
                request("POST", driver_url, prefix + "/timeouts", {"script": 25000, "pageLoad": 30000})
                def evaluate(script, args=None):
                    return request("POST", driver_url, prefix + "/execute/sync", {"script": script, "args": args or []})
                def evaluate_async(script, args=None):
                    return request("POST", driver_url, prefix + "/execute/async", {"script": script, "args": args or []})
                def navigate(path):
                    request("POST", driver_url, prefix + "/url", {"url": base + path})
                    wait_until(lambda: evaluate("return document.readyState==='complete'"), "Firefox DOM", 15)
                def snapshot(label):
                    data = evaluate("""return ({
                      state: globalThis.__yt075Scene?.snapshot?.() ?? null,
                      text: document.querySelector('#scene-status')?.textContent,
                      counter: document.querySelector('#counter-value')?.textContent,
                      records: document.querySelectorAll('#records li').length,
                      canvas: {
                        width: document.querySelector('#scene-canvas')?.width,
                        height: document.querySelector('#scene-canvas')?.height
                      }
                    })""")
                    report["snapshots"].append({"label": label, "value": data})
                    return data
                navigate("/scene")
                wait_until(lambda: evaluate("return !!globalThis.__yt075Scene"), "actual scene loader", 22)
                initial=snapshot("before activation")
                assert initial["records"] == 12 and initial["canvas"] == {"width":600,"height":400}
                assert initial["state"]["starts"] == 0
                assert not evaluate("""return performance.getEntriesByType('resource').some(x =>
                    x.name.includes('scene_bg.wasm') || x.name.endsWith('/scene.js'))"""), "GPU Wasm fetched before activation"
                report["navigator_adapter"] = evaluate_async("""
                    const done=arguments[arguments.length-1];
                    (async()=>{const a=await navigator.gpu?.requestAdapter();
                        done(a?{
                            available:true,
                            vendor:a.info?.vendor??"",
                            architecture:a.info?.architecture??"",
                            fallback:a.isFallbackAdapter??a.info?.isFallbackAdapter??null
                        }:{available:!!navigator.gpu,adapter:null});
                    })().catch(e=>done({error:String(e)}));""")
                evaluate("document.getElementById('activate-scene').click()")
                wait_until(
                    lambda: evaluate("return ['ready','unavailable'].includes(globalThis.__yt075Scene?.snapshot?.().state)"),
                    "YT-075 Firefox GPU initialization", 45
                )
                state = snapshot("GPU activation")
                if state["state"]["state"] != "ready":
                    report["status"] = "gpu_unavailable"
                else:
                    assert not report["navigator_adapter"].get("fallback"), "software adapter does not qualify"
                    # The browser may retire a one-shot swapchain image after compositing.
                    # Draw again immediately before taking the actual canvas screenshot.
                    redrawn = evaluate_async("""
                        const done=arguments[arguments.length-1];
                        requestAnimationFrame(()=>{
                            try { globalThis.__yt075Scene.draw();
                                requestAnimationFrame(()=>done("draw_submitted"));
                            } catch (e) { done("draw_error: "+String(e)); }
                        });
                    """)
                    assert redrawn == "draw_submitted", redrawn
                    report["pre_capture_redraw"] = redrawn
                    element = request("POST", driver_url, prefix + "/element", {
                        "using":"css selector","value":"#scene-canvas"
                    })
                    element_id = element[ELEMENT_KEY]
                    png = base64.b64decode(request(
                        "GET", driver_url, prefix + f"/element/{element_id}/screenshot"))
                    (EVIDENCE / "actual-canvas.png").write_bytes(png)
                    width,height,channels,rows=png_pixels(png)
                    mismatches=[]
                    if (width,height)==(600,400):
                        for index in range(120):
                            x=(index%12)*50+25
                            y=(index//12)*40+20
                            actual=list(rows[y][x*channels:x*channels+3])
                            expected=([255,0,0],[0,255,0],[0,0,255])[index%3]
                            if any(abs(a-b)>25 for a,b in zip(actual,expected)):
                                mismatches.append({"id":index+1,"pixel":actual})
                    else:
                        mismatches.append({"unexpected_image_shape":[width,height]})
                    report["actual_pixels"]={
                        "width":width,"height":height,
                        "centers_verified":120-len(mismatches),
                        "mismatches":mismatches[:8],
                        # Firefox element screenshots include a one-pixel white top crop seam;
                        # sample clear color strictly inside the canvas bounds.
                        "capture_top_edge":list(rows[0][:3]),
                        "interior_black":[list(rows[1][5*channels:5*channels+3]),
                                          list(rows[20][45*channels:45*channels+3])],
                        "sha256":hashlib.sha256(png).hexdigest()
                    }
                    assert not mismatches, "Original Wasm GPU 120-center draw failed"
                    assert report["actual_pixels"]["interior_black"] == [[0,0,0],[0,0,0]]
                    before=snapshot("repeated activation before")
                    evaluate("document.getElementById('activate-scene').click()")
                    after=snapshot("repeated activation after")
                    assert after["state"]["starts"]==before["state"]["starts"], "duplicate mount"
                    evaluate("document.getElementById('counter-increment').click()")
                    counter=snapshot("sibling counter")
                    assert counter["counter"]=="3", "sibling failed"
                    evaluate("document.getElementById('dispose-scene').click()")
                    disposed=snapshot("after dispose")
                    assert disposed["state"]["state"]=="disposed"
                    report["completed_browser_checks"]=[
                        "120 real canvas pixel centers", "interior black clear", "single pending initialization",
                        "independent sibling counter", "explicit resource dispose"
                    ]
                    # The original app's canvas must permit a second owner after explicit disposal.
                    evaluate("document.getElementById('activate-scene').click()")
                    wait_until(lambda: evaluate("return ['ready','unavailable'].includes(__yt075Scene.snapshot().state)"),
                               "GPU remount result", 35)
                    remount = snapshot("after remount")
                    assert remount["state"]["state"] == "ready", "GPU remount failed"
                    assert remount["state"]["starts"] == 2, "remount owner count wrong"
                    evaluate("document.getElementById('dispose-scene').click()")
                    assert snapshot("after second dispose")["state"]["state"] == "disposed"
                    report["completed_browser_checks"].append("dispose and remount")

                    def fresh(case):
                        navigate("/scene?yt075="+case)
                        wait_until(lambda: evaluate("return !!globalThis.__yt075Scene"),
                                   "fresh loader for "+case, 22)
                        first=snapshot("fresh "+case)
                        assert first["state"]["starts"]==0
                        assert first["counter"]=="2"

                    def check_unavailable(name, injected):
                        fresh(name)
                        evaluate(injected)
                        evaluate("document.getElementById('activate-scene').click()")
                        wait_until(lambda: evaluate("return __yt075Scene.snapshot().state === 'unavailable'"),
                                   name+" GPU fallback", 35)
                        result=snapshot(name+" fallback")
                        assert result["records"]==12
                        assert "HTML list" in result["text"], result["text"]
                        evaluate("document.getElementById('counter-increment').click()")
                        assert snapshot(name+" sibling")["counter"]=="3"
                        report["faults"].append({"name":name,"injected":True,
                                                 "fallback_and_sibling":"passed"})


                    # Hold a real acquired first device while its owner is disposed.
                    # Only the second owner is allowed to update the current scene status.
                    fresh("late_completion")
                    evaluate("""
                        window.__yt075Held={calls:0,acquired:false,destroyed:0};
                        const oldRequest=GPUAdapter.prototype.requestDevice;
                        GPUAdapter.prototype.requestDevice=async function(...args){
                            const first=++window.__yt075Held.calls === 1;
                            let device;
                            try { device=await oldRequest.apply(this,args); }
                            catch(error){ window.__yt075Held.requestError=String(error); throw error; }
                            if (!first) return device;
                            const destroy=device.destroy.bind(device);
                            device.destroy=()=>{
                                window.__yt075Held.destroyed++;
                                return destroy();
                            };
                            window.__yt075Held.firstDevice=device;
                            window.__yt075Held.acquired=true;
                            return await new Promise(resolve=>{
                                window.__yt075Held.release=()=>resolve(device);
                            });
                        };
                    """)
                    assert evaluate("return !!window.__yt075Held && window.__yt075Held.calls === 0"), "WebDriver page state was not installed"
                    evaluate("document.getElementById('activate-scene').click()")
                    try:
                        wait_until(lambda: evaluate("return !!window.__yt075Held?.acquired"),
                                   "first device acquired, pending", 35)
                    except TimeoutError:
                        report["held_diagnostics"] = evaluate("""return ({
                            calls: window.__yt075Held?.calls ?? null,
                            acquired: window.__yt075Held?.acquired ?? false,
                            requestError: window.__yt075Held?.requestError || null,
                            scene: __yt075Scene.snapshot(),
                            status: document.getElementById("scene-status").textContent
                        })""")
                        raise
                    evaluate("document.getElementById('activate-scene').click()")
                    assert snapshot("duplicate while pending")["state"]["starts"]==1
                    assert evaluate("return window.__yt075Held.calls") == 1
                    evaluate("document.getElementById('dispose-scene').click()")
                    evaluate("document.getElementById('activate-scene').click()")
                    wait_until(lambda: evaluate("return __yt075Scene.snapshot().state === 'ready'"),
                               "replacement ready while first delayed", 35)
                    replacement=snapshot("replacement before late completion")
                    evaluate("window.__yt075Held.release()")
                    wait_until(lambda: evaluate("return window.__yt075Held.destroyed===1"),
                               "retired device actually destroyed", 25)
                    after=snapshot("replacement after late completion")
                    assert after["state"]==replacement["state"]
                    assert after["text"]==replacement["text"]
                    evaluate("document.getElementById('counter-increment').click()")
                    assert snapshot("sibling after retirement")["counter"]=="3"
                    evaluate("document.getElementById('dispose-scene').click()")
                    report["faults"].append({"name":"held_real_device_retirement",
                        "injected":True,"stale_completion_destroyed":True,
                        "replacement_and_sibling":"passed"})

                    # Probe unavailable paths after the real-device lifetime test.
                    # Injected GPU validation failures can affect Firefox device scheduling.
                    check_unavailable("absent_webgpu",
                        "Object.defineProperty(navigator,\'gpu\',{get:()=>undefined,configurable:true});")
                    check_unavailable("null_adapter",
                        "Object.defineProperty(navigator.gpu,\'requestAdapter\',{value:async()=>null,configurable:true});")
                    check_unavailable("device_rejection",
                        "Object.defineProperty(GPUAdapter.prototype,\'requestDevice\',{value:async()=>{throw new Error(\'injected device failure\')},configurable:true});")
                    check_unavailable("initialization_validation",
                        "Object.defineProperty(GPUDevice.prototype,\'popErrorScope\',{value:async()=>new GPUValidationError({message:\'injected validation error\'}),configurable:true});")

                    # Validate server-side fail-closed loader checks on a temporary copy.
                    manifest_path=asset_root/"manifest.json"
                    original_manifest=manifest_path.read_bytes()
                    meta=json.loads(original_manifest)
                    wasm_path=asset_root/"assets"/meta["scene"]["digest"]/"scene_bg.wasm"
                    original_wasm=wasm_path.read_bytes()
                    try:
                        wasm_path.unlink()
                        navigate("/scene?yt075=missing_wasm")
                        assert evaluate("return !globalThis.__yt075Scene")
                        assert evaluate("return document.querySelectorAll('#records li').length")==12
                        report["faults"].append({"name":"missing_wasm","semantic_ssr":"passed"})
                    finally:
                        wasm_path.write_bytes(original_wasm)
                    try:
                        manifest_path.write_bytes(b"invalid")
                        code, csp_headers, html=http(base+"/scene?record=38&q=done&page=1")
                        assert code==200 and b"Work record 038" in html
                        assert b"<script" not in html
                        assert "unsafe-inline" not in csp_headers["content-security-policy"]
                        report["faults"].append({"name":"invalid_manifest","semantic_ssr":"passed"})
                    finally:
                        manifest_path.write_bytes(original_manifest)
                    report["completed_browser_checks"].append(
                        "missing Wasm and malformed manifest fail closed")
                    report["status"]="passed_y075_browser_qualification"
            except BaseException as error:
                report["error"]=type(error).__name__+": "+str(error)[:600]
                report["status"]="failed"
            finally:
                if session is not None:
                    try:
                        request("DELETE", driver_url, f"/session/{session}")
                        report["cleanup"]["session_deleted"]=True
                    except Exception as exc:
                        report["cleanup"]["session_delete_error"]=str(exc)[:250]
                for label, proc in (("geckodriver",gecko),("server",server)):
                    try:
                        stop_owned_process(proc,process_group=True)
                        report["cleanup"][label+"_exit"]=proc.returncode if proc is not None else None
                    except Exception as error:
                        report["cleanup"][label+"_error"]=str(error)[:250]
                report["cleanup"]["tmp_profile_removed"] = not driver_profile_root.exists()
                glog.flush()
                report["geckodriver_log_sha256"]=hashlib.sha256(
                    (EVIDENCE/"geckodriver.log").read_bytes()).hexdigest()
        # TemporaryDirectory removes the disposable profile directory.
    report["cleanup"]["tmp_profile_removed"]=not Path(tmp).exists()
    (EVIDENCE/"result.json").write_text(json.dumps(report,indent=2,ensure_ascii=False)+"\n")
    print(json.dumps({
        "status":report["status"],
        "error":report.get("error"),
        "capabilities":report.get("capabilities"),
        "adapter":report.get("navigator_adapter"),
        "snapshots":report["snapshots"][-2:],
        "actual_pixels":report.get("actual_pixels"),
        "cleanup":report["cleanup"],
    },ensure_ascii=False)[:4200])
    return 0 if report["status"]=="passed_y075_browser_qualification" and report["cleanup"]["tmp_profile_removed"] else 1


if __name__=="__main__":
    raise SystemExit(main())
