#!/usr/bin/env python3
"""One owned YT-075 Chrome attempt. Explicit fault injection is never GPU proof.

Requires the existing build.py output and debug native binary. No builds, unsafe
GPU flags, third-party modules, retries, benchmarks or unrelated product probes.
"""
import base64
import json
import os
from pathlib import Path
import shutil
import struct
import subprocess
import sys
import tempfile
import zlib

HERE = Path(__file__).resolve().parent
ROOT = HERE.parent.parent
sys.path.insert(0, str(ROOT / "scripts"))
from smoke_browser import ChromeDevTools, http, port_number, remove_owned_tree, stop_owned_process, wait_until

NVIDIA_ICD_DIAGNOSTIC = os.environ.get("YT075_NVIDIA_ICD_DIAGNOSTIC") == "1"
PRIME_OFFLOAD_DIAGNOSTIC = os.environ.get("YT075_PRIME_OFFLOAD_DIAGNOSTIC") == "1"
INTEL_GEN12_DIAGNOSTIC = os.environ.get("YT075_INTEL_GEN12_DIAGNOSTIC") == "1"
NESTED_WESTON_DIAGNOSTIC = os.environ.get("YT075_NESTED_WESTON_DIAGNOSTIC") == "1"
ANGLE_VULKAN_SHARING_DIAGNOSTIC = os.environ.get("YT075_ANGLE_VULKAN_SHARING_DIAGNOSTIC") == "1"
if sum((NVIDIA_ICD_DIAGNOSTIC, PRIME_OFFLOAD_DIAGNOSTIC, INTEL_GEN12_DIAGNOSTIC,
        NESTED_WESTON_DIAGNOSTIC, ANGLE_VULKAN_SHARING_DIAGNOSTIC)) > 1:
    raise RuntimeError("Select only one YT-075 GPU diagnostic profile")
EVIDENCE = ROOT / "target/m11-gpu/yt075-20261014" / (
    "browser-angle-vulkan-sharing" if ANGLE_VULKAN_SHARING_DIAGNOSTIC else
    "browser-weston-x11" if NESTED_WESTON_DIAGNOSTIC else
    "browser-intel-gen12" if INTEL_GEN12_DIAGNOSTIC else
    "browser-prime-offload" if PRIME_OFFLOAD_DIAGNOSTIC else
    "browser-nvidia-icd" if NVIDIA_ICD_DIAGNOSTIC else "browser-x11"
)

def png_pixels(data):
    """Decode Chrome's noninterlaced RGB/RGBA PNG screenshot for actual center checks."""
    assert data[:8] == b"\x89PNG\r\n\x1a\n"
    offset, compressed = 8, bytearray()
    while offset < len(data):
        size = struct.unpack(">I", data[offset:offset + 4])[0]
        kind, chunk = data[offset + 4:offset + 8], data[offset + 8:offset + 8 + size]
        if kind == b"IHDR":
            width, height, depth, color, compression, filtering, interlace = struct.unpack(">IIBBBBB", chunk)
            assert depth == 8 and color in (2, 6) and (compression, filtering, interlace) == (0, 0, 0)
        if kind == b"IDAT": compressed.extend(chunk)
        offset += size + 12
    channels = 3 if color == 2 else 4
    stride = width * channels
    raw, rows, previous = zlib.decompress(compressed), [], bytearray(stride)
    for y in range(height):
        start = y * (stride + 1)
        mode = raw[start]
        row = bytearray(raw[start + 1:start + 1 + stride])
        for x in range(stride):
            left = row[x - channels] if x >= channels else 0
            above = previous[x]
            corner = previous[x - channels] if x >= channels else 0
            if mode == 0: value = 0
            elif mode == 1: value = left
            elif mode == 2: value = above
            elif mode == 3: value = (left + above) // 2
            elif mode == 4:
                p = left + above - corner
                distances = (abs(p - left), abs(p - above), abs(p - corner))
                value = (left, above, corner)[distances.index(min(distances))]
            else: raise AssertionError(f"Unsupported PNG filter {mode}")
            row[x] = (row[x] + value) & 255
        rows.append(row)
        previous = row
    return width, height, channels, rows

def main():
    chrome = shutil.which("google-chrome-stable") or shutil.which("google-chrome")
    binary = HERE / "target/debug/yetlet-webgpu-scene-island"
    assert chrome and binary.is_file(), "Build native fixture and provide installed Chrome first"
    assert (ROOT / "target/webgpu-scene-island-dist/manifest.json").is_file(), "Run build.py first"
    EVIDENCE.mkdir(parents=True, exist_ok=True)
    report = {"task_id": "YT-075", "status": "in_progress", "snapshots": [], "faults": [], "cleanup": False}
    server = browser = client = browser_client = None
    target_id = None
    with tempfile.TemporaryDirectory(prefix="yt075-chrome-") as profile:
        asset_root = Path(profile) / "assets-root"
        shutil.copytree(ROOT / "target/webgpu-scene-island-dist", asset_root)
        manifest_bytes = (asset_root / "manifest.json").read_bytes()
        server_port, chrome_port = port_number(), port_number()
        base = f"http://127.0.0.1:{server_port}"
        with (EVIDENCE / "server.log").open("w") as server_log, (EVIDENCE / "chrome.log").open("w") as chrome_log:
            try:
                server = subprocess.Popen([str(binary)], cwd=ROOT, env=dict(os.environ, YETLET_SCENE_LISTEN=f"127.0.0.1:{server_port}", YETLET_SCENE_ASSET_ROOT=str(asset_root)), stdout=server_log, stderr=subprocess.STDOUT)
                wait_until(lambda: http(base + "/healthz")[0] == 200, "owned server")
                _, headers, html = http(base + "/scene?q=Done&page=1")
                assert b"40 matches; page 2 of 4" in html
                assert "unsafe-inline" not in headers["content-security-policy"]
                # Existing owner X11 session; no host changes or sandbox/GPU-policy bypass.
                browser_env = dict(os.environ, DISPLAY=":0", XAUTHORITY=str(Path.home() / ".Xauthority"))
                browser_args = [chrome, "--headless=new", "--enable-gpu", "--ozone-platform=x11"]
                if NVIDIA_ICD_DIAGNOSTIC:
                    # This profile changes only the owned Chrome process. Keep prior X11 results intact.
                    icd = Path("/usr/share/vulkan/icd.d/nvidia_icd.json")
                    assert icd.is_file(), "NVIDIA Vulkan ICD manifest unavailable"
                    browser_env.update(VK_DRIVER_FILES=str(icd), __NV_PRIME_RENDER_OFFLOAD="1", __VK_LAYER_NV_optimus="NVIDIA_only")
                    browser_args.extend(["--use-angle=vulkan", "--enable-features=Vulkan"])
                elif PRIME_OFFLOAD_DIAGNOSTIC:
                    # Preserve normal Vulkan loader discovery. Limit offload selection to owned Chrome.
                    browser_env.pop("VK_DRIVER_FILES", None)
                    browser_env.pop("VK_ICD_FILENAMES", None)
                    browser_env.update(__NV_PRIME_RENDER_OFFLOAD="1", __VK_LAYER_NV_optimus="NVIDIA_only", __GLX_VENDOR_LIBRARY_NAME="nvidia")
                    browser_args.extend(["--use-angle=vulkan", "--enable-features=Vulkan"])
                elif ANGLE_VULKAN_SHARING_DIAGNOSTIC:
                    # Based on gpuweb/gpuweb issue 5022. Share the ANGLE Vulkan device;
                    # this is an owned Chrome launch profile, not a system-wide flag.
                    for name in ("VK_DRIVER_FILES", "VK_ICD_FILENAMES", "DRI_PRIME", "MESA_VK_DEVICE_SELECT"):
                        browser_env.pop(name, None)
                    browser_env.update(__NV_PRIME_RENDER_OFFLOAD="1", __VK_LAYER_NV_optimus="NVIDIA_only",
                                       __GLX_VENDOR_LIBRARY_NAME="nvidia")
                    browser_args.extend(["--use-angle=vulkan",
                                         "--enable-features=Vulkan,VulkanFromANGLE,DefaultANGLEVulkan"])
                elif INTEL_GEN12_DIAGNOSTIC:
                    # Chrome 144+ supports Linux Intel Gen12+: WebGPU uses Vulkan; UI uses GL.
                    # Filter selected Vulkan physical devices, not the loader ICD list.
                    for name in ("VK_DRIVER_FILES", "VK_ICD_FILENAMES", "__NV_PRIME_RENDER_OFFLOAD",
                                 "__VK_LAYER_NV_optimus", "__GLX_VENDOR_LIBRARY_NAME"):
                        browser_env.pop(name, None)
                    browser_env.update(DRI_PRIME="0", MESA_VK_DEVICE_SELECT="8086:a780!")
                    browser_args.append("--use-angle=gl")
                elif NESTED_WESTON_DIAGNOSTIC:
                    # An independent owner must already hold this nested Wayland socket.
                    # Never modify the active X11 session, Vulkan ICD discovery or sandbox.
                    socket_name = "wayland-yt075-weston-browser"
                    runtime_dir = Path(browser_env.get("XDG_RUNTIME_DIR", "/run/user/1000"))
                    assert (runtime_dir / socket_name).is_socket(), "Owned Weston Wayland socket is missing"
                    for name in ("VK_DRIVER_FILES", "VK_ICD_FILENAMES", "DRI_PRIME", "MESA_VK_DEVICE_SELECT"):
                        browser_env.pop(name, None)
                    browser_env.update(WAYLAND_DISPLAY=socket_name, XDG_SESSION_TYPE="wayland",
                                       __NV_PRIME_RENDER_OFFLOAD="1", __VK_LAYER_NV_optimus="NVIDIA_only",
                                       __GLX_VENDOR_LIBRARY_NAME="nvidia")
                    browser_args = [chrome, "--headless=new", "--enable-gpu", "--ozone-platform=wayland",
                                    "--use-angle=vulkan", "--enable-features=Vulkan"]
                    report["nested_wayland_socket"] = socket_name
                report["gpu_profile"] = ("angle-vulkan-sharing-x11-nvidia" if ANGLE_VULKAN_SHARING_DIAGNOSTIC
                                          else "nested-weston-x11-nvidia-wayland" if NESTED_WESTON_DIAGNOSTIC
                                          else "intel-gen12-default-icds" if INTEL_GEN12_DIAGNOSTIC
                                          else "prime-offload-default-icds" if PRIME_OFFLOAD_DIAGNOSTIC
                                          else "nvidia-icd-only" if NVIDIA_ICD_DIAGNOSTIC else "x11-default")
                browser_args.extend(["--no-first-run", "--disable-extensions", "--disable-background-networking", "--remote-debugging-address=127.0.0.1", "--remote-allow-origins=http://127.0.0.1", f"--remote-debugging-port={chrome_port}", f"--user-data-dir={profile}", "about:blank"])
                browser = subprocess.Popen(browser_args, env=browser_env, stdout=chrome_log, stderr=subprocess.STDOUT, start_new_session=True)
                version = wait_until(lambda: json.loads(http(f"http://127.0.0.1:{chrome_port}/json/version")[2]), "Chrome debugger")
                report["chrome"] = version
                browser_client = ChromeDevTools(version["webSocketDebuggerUrl"])
                target_id = browser_client.call("Target.createTarget", {"url": "about:blank"})["targetId"]
                report["owned_target"] = target_id
                def target_url():
                    matches = [p for p in json.loads(http(f"http://127.0.0.1:{chrome_port}/json/list")[2]) if p["id"] == target_id and p["type"] == "page"]
                    assert len(matches) <= 1
                    return matches[0]["webSocketDebuggerUrl"] if matches else None
                client = ChromeDevTools(wait_until(target_url, "unique returned page target"))
                for method in ("Page.enable", "Runtime.enable", "Network.enable", "Log.enable"): client.call(method)
                def evaluate_async(code):
                    result = client.call("Runtime.evaluate", {"expression": code, "awaitPromise": True, "returnByValue": True})
                    assert "exceptionDetails" not in result, result
                    return result["result"].get("value")
                def snapshot(label):
                    state = client.evaluate("({url:location.href,ready:document.readyState,scene:globalThis.__yt075Scene?.snapshot(),status:document.querySelector('#scene-status')?.textContent,counter:document.querySelector('#counter-value')?.textContent,records:document.querySelector('#records')?.textContent,adapter:document.querySelector('#scene-island')?.dataset.adapter})")
                    report["snapshots"].append({"label": label, "value": state})
                    return state
                def click(identifier):
                    before = snapshot("before " + identifier)
                    assert client.evaluate(f"document.getElementById({json.dumps(identifier)}) !== null"), before
                    client.evaluate(f"document.getElementById({json.dumps(identifier)}).click()")
                    return snapshot("after " + identifier)
                visit = 0
                def navigate(enhanced=True):
                    nonlocal visit
                    visit += 1
                    query = f"?visit={visit}"
                    client.call("Page.navigate", {"url": base + "/scene" + query})
                    wait_until(lambda: client.evaluate(f"location.search === {json.dumps(query)} && document.readyState === 'complete'"), "fresh navigation document")
                    if enhanced:
                        wait_until(lambda: client.evaluate("!!globalThis.__yt075Scene && document.querySelector('#counter-island')?.dataset.yetletActivated === 'true'"), "fresh fixture and sibling")
                    snapshot("fresh navigation")
                # JS-disabled semantic alternative on the same owned page.
                client.call("Emulation.setScriptExecutionDisabled", {"value": True})
                client.call("Page.navigate", {"url": base + "/scene?q=Done&page=1&record=38"})
                def no_js_document():
                    node = client.call("DOM.getDocument")["root"]["nodeId"]
                    html = client.call("DOM.getOuterHTML", {"nodeId": node})["outerHTML"]
                    return html if "40 matches; page 2 of 4" in html and "Work record 038" in html and 'method="get"' in html else None
                report["no_js_html"] = wait_until(no_js_document, "no-JS semantic document")
                (EVIDENCE / "no-js.png").write_bytes(base64.b64decode(client.call("Page.captureScreenshot")["data"]))
                client.call("Emulation.setScriptExecutionDisabled", {"value": False})
                navigate()
                assert not any("/scene.js" in u or "scene_bg.wasm" in u for u in client.requests), "GPU files loaded before activation"
                report["navigator_adapter"] = evaluate_async("(async()=>{if(!navigator.gpu)return {available:false};const a=await navigator.gpu.requestAdapter();return a?{available:true,info:{vendor:a.info.vendor,architecture:a.info.architecture,device:a.info.device,description:a.info.description},fallback:a.isFallbackAdapter ?? a.info.isFallbackAdapter ?? null}: {available:true,adapter:null};})()")
                report["system_gpu_observation"] = browser_client.call("SystemInfo.getInfo").get("gpu")
                click("activate-scene")
                wait_until(lambda: client.evaluate("['ready','unavailable'].includes(__yt075Scene.snapshot().state)"), "fallible GPU initialization")
                state = snapshot("actual GPU result")
                assert state["scene"]["state"] == "ready", "Required GPU capability unavailable; stop without GPU acceptance"
                adapter_text = json.dumps(report["navigator_adapter"]) + state.get("adapter", "")
                assert report["navigator_adapter"].get("fallback") is not True and not any(s in adapter_text.lower() for s in ("swiftshader", "llvmpipe", "lavapipe", "software")), "Software adapter is not hardware proof"
                # Read real screenshot pixels: all 120 known centers, plus a black margin.
                client.evaluate("document.getElementById('scene-canvas').scrollIntoView(); __yt075Scene.draw()")
                evaluate_async("new Promise(resolve=>requestAnimationFrame(()=>requestAnimationFrame(resolve)))")
                rect = client.evaluate("(()=>{const r=document.getElementById('scene-canvas').getBoundingClientRect();return {x:r.x+scrollX,y:r.y+scrollY,width:r.width,height:r.height,scale:1};})()")
                image = base64.b64decode(client.call("Page.captureScreenshot", {"clip": rect, "captureBeyondViewport": True})["data"])
                (EVIDENCE / "actual-canvas.png").write_bytes(image)
                width, height, channels, rows = png_pixels(image)
                assert (width, height) == (600, 400)
                for index in range(120):
                    x, y = (index % 12) * 50 + 25, (index // 12) * 40 + 20
                    expected = ([255, 0, 0], [0, 255, 0], [0, 0, 255])[index % 3]
                    assert list(rows[y][x * channels:x * channels + 3]) == expected, f"Actual center mismatch record {index + 1}"
                assert list(rows[0][:3]) == [0, 0, 0]
                report["actual_pixels"] = {"centers": 120, "margin": "black", "screenshot": "actual-canvas.png"}
                # Explicit injected faults, each on a fresh fixture; no fabricated drawing.
                for name, source in [
                    ("absent_webgpu", "Object.defineProperty(Navigator.prototype,'gpu',{get:()=>undefined,configurable:true});"),
                    ("null_adapter", "navigator.gpu.requestAdapter=async()=>null;"),
                    ("device_rejection", "GPUAdapter.prototype.requestDevice=async()=>{throw new Error('YT-075 injected device failure')};"),
                    ("initialization_validation", "GPUDevice.prototype.popErrorScope=async()=>new GPUValidationError('YT-075 injected initialization validation');"),
                ]:
                    script = client.call("Page.addScriptToEvaluateOnNewDocument", {"source": source})["identifier"]
                    navigate()
                    click("activate-scene")
                    wait_until(lambda: client.evaluate("__yt075Scene.snapshot().state === 'unavailable'"), name)
                    click("counter-increment")
                    assert snapshot(name)["counter"] == "3"
                    assert client.evaluate("document.querySelectorAll('#records li').length === 12")
                    report["faults"].append({"name": name, "injected": True, "fallback_and_sibling": "passed"})
                    client.call("Page.removeScriptToEvaluateOnNewDocument", {"identifier": script})
                client.call("Network.setBlockedURLs", {"urls": ["*scene_bg.wasm"]})
                navigate(); click("activate-scene")
                wait_until(lambda: client.evaluate("__yt075Scene.snapshot().state === 'unavailable'"), "injected missing WASM")
                click("counter-increment"); assert snapshot("missing WASM")["counter"] == "3"
                report["faults"].append({"name": "missing_wasm_network_block", "injected": True, "fallback_and_sibling": "passed"})
                client.call("Network.setBlockedURLs", {"urls": []})
                for name, data in [("missing_manifest", None), ("invalid_manifest", b"invalid")]:
                    if data is None: (asset_root / "manifest.json").unlink()
                    else: (asset_root / "manifest.json").write_bytes(data)
                    navigate(enhanced=False)
                    assert client.evaluate("!globalThis.__yt075Scene && document.querySelectorAll('#records li').length === 12 && !!document.querySelector('form[method=get]')")
                    report["faults"].append({"name": name, "injected": True, "semantic_ssr": "passed"})
                    (asset_root / "manifest.json").write_bytes(manifest_bytes)
                # Hold the first real acquired device until after retirement/replacement.
                source = """globalThis.__yt075Held={calls:0,destroyed:0,acquired:false};
const request=GPUAdapter.prototype.requestDevice;
GPUAdapter.prototype.requestDevice=async function(...args){
 const first=++__yt075Held.calls===1; const device=await request.apply(this,args);
 if(!first)return device;
 const destroy=device.destroy.bind(device); device.destroy=()=>{__yt075Held.destroyed++;destroy()};
 __yt075Held.acquired=true;
 return await new Promise(resolve=>{__yt075Held.release=()=>resolve(device)});
};"""
                script = client.call("Page.addScriptToEvaluateOnNewDocument", {"source": source})["identifier"]
                navigate(); click("activate-scene")
                wait_until(lambda: client.evaluate("__yt075Held.acquired"), "injected pending real device")
                click("activate-scene"); assert snapshot("repeated pending activation")["scene"]["starts"] == 1
                assert client.evaluate("__yt075Held.calls === 1")
                click("dispose-scene"); click("activate-scene")
                wait_until(lambda: client.evaluate("__yt075Scene.snapshot().state === 'ready'"), "replacement owner")
                replacement = snapshot("replacement before release")
                snapshot("before injected completion"); client.evaluate("__yt075Held.release()")
                wait_until(lambda: client.evaluate("__yt075Held.destroyed === 1"), "late device destroyed")
                after = snapshot("replacement after late completion")
                assert after["scene"] == replacement["scene"] and after["status"] == replacement["status"]
                click("counter-increment"); assert snapshot("sibling after retirement")["counter"] == "3"
                click("dispose-scene")
                client.call("Page.removeScriptToEvaluateOnNewDocument", {"identifier": script})
                report["faults"].append({"name": "held_real_device_completion", "injected": True, "single_initialization_late_destroy_replacement_sibling": "passed"})
                assert not client.api_posts and not client.websockets, "Unexpected remote event forwarding"
                report["browser_exceptions"] = client.exceptions
                report["network_failures"] = client.failed_responses
                report["status"] = "passed"
            except BaseException as error:
                report["status"] = "blocked_or_failed"
                report["error"] = str(error)
                raise
            finally:
                cleanup_errors = []
                for label, operation in [
                    ("target", lambda: browser_client.call("Target.closeTarget", {"targetId": target_id}) if target_id and browser_client else None),
                    ("page socket", lambda: client.close() if client else None),
                    ("browser socket", lambda: browser_client.close() if browser_client else None),
                    ("browser process", lambda: stop_owned_process(browser, process_group=True)),
                    ("server process", lambda: stop_owned_process(server)),
                    ("profile", lambda: remove_owned_tree(Path(profile))),
                ]:
                    try: operation()
                    except Exception as error: cleanup_errors.append(f"{label}: {error}")
                report["browser_exceptions"] = client.exceptions if client else []
                report["network_failures"] = client.failed_responses if client else []
                report["cleanup"] = {"browser_exit": browser.returncode if browser else None, "server_exit": server.returncode if server else None, "profile_removed": not Path(profile).exists(), "errors": cleanup_errors}
                if cleanup_errors or Path(profile).exists(): report["status"] = "blocked_cleanup"
                (EVIDENCE / "result.json").write_text(json.dumps(report, indent=2) + "\n")
    assert report["status"] == "passed" and not Path(profile).exists(), "Owned cleanup incomplete"
    report["cleanup"]["profile_removed"] = True
    (EVIDENCE / "result.json").write_text(json.dumps(report, indent=2) + "\n")
    print("PASS YT-075 bounded browser assertions; see", EVIDENCE)

if __name__ == "__main__": main()
