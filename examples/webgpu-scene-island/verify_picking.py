#!/usr/bin/env python3
"""One YT-078 real-Wasm Firefox X11 picking product test; never reruns YT-075 receipts.

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
from smoke_browser import ChromeDevTools, http, port_number, stop_owned_process, wait_until
from urllib.parse import urlsplit

HERE = Path(__file__).resolve().parent
ROOT = HERE.parent.parent
# Same product test after the desktop-window minimum-size setup failure.
EVIDENCE = ROOT / "target/m11-gpu/yt078-20261014/browser/viewport"


def portrait_viewport(websocket_url, page_url):
    """Use the browser's standard BiDi viewport, not the desktop window-size hint.

    Reuse the existing JSON WebSocket transport after the origin-less Firefox
    handshake. This one-purpose fixture connection never changes CSP or profiles.
    """
    parsed = urlsplit(websocket_url)
    assert parsed.scheme == "ws" and parsed.hostname in ("127.0.0.1", "localhost", "::1")
    connection = ChromeDevTools.__new__(ChromeDevTools)
    connection.sock = socket.create_connection((parsed.hostname, parsed.port), timeout=8)
    connection.sock.settimeout(12)
    connection.buffer = b""
    connection.next_id = 0
    for name in ("requests", "api_posts", "websockets", "exceptions", "failed_responses"):
        setattr(connection, name, [])
    try:
        key = base64.b64encode(os.urandom(16)).decode()
        target = parsed.path + ("?" + parsed.query if parsed.query else "")
        handshake = (f"GET {target} HTTP/1.1\r\nHost: {parsed.netloc}\r\n"
                     "Upgrade: websocket\r\nConnection: Upgrade\r\n"
                     f"Sec-WebSocket-Key: {key}\r\nSec-WebSocket-Version: 13\r\n\r\n")
        connection.sock.sendall(handshake.encode())
        while b"\r\n\r\n" not in connection.buffer:
            part = connection.sock.recv(4096)
            assert part and len(connection.buffer) < 65536, "BiDi handshake closed or oversized"
            connection.buffer += part
        headers, connection.buffer = connection.buffer.split(b"\r\n\r\n", 1)
        assert b" 101 " in headers.split(b"\r\n", 1)[0], "BiDi handshake rejected"
        fields = dict(line.split(b":", 1) for line in headers.split(b"\r\n")[1:] if b":" in line)
        fields = {name.lower(): value.strip() for name, value in fields.items()}
        expected = base64.b64encode(hashlib.sha1((key + "258EAFA5-E914-47DA-95CA-C5AB0DC85B11").encode()).digest())
        assert fields.get(b"sec-websocket-accept") == expected, "BiDi handshake mismatch"
        contexts = connection.call("browsingContext.getTree", {"maxDepth": 0})["contexts"]
        matches = [item for item in contexts if item["url"] == page_url]
        assert len(matches) == 1, "Expected one returned owned browser context"
        context = matches[0]["context"]
        connection.call("browsingContext.setViewport", {
            "context": context, "viewport": {"width": 390, "height": 700}
        })
        return {"context": context, "width": 390, "height": 700, "kind": "BiDi viewport emulation; not physical mobile"}
    finally:
        connection.close()


def main():
    assert os.environ.get("YT075_FIREFOX_HEADFUL_X11") == "1", "Use the established headful X11 recipe"
    assert not EVIDENCE.exists(), "Existing YT-078 evidence: never overwrite or retry automatically"
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
    manifest = json.loads((assets / "manifest.json").read_text())
    asset_paths = [assets / "manifest.json"]
    for key, module in (("scene", "scene"), ("counter", "yetlet_dashboard_counter")):
        directory = assets / "assets" / manifest[key]["digest"]
        asset_paths.extend(directory / name for name in (f"{key}-loader.js", f"{module}.js", f"{module}_bg.wasm"))
    inputs = source_paths + asset_paths + [binary, ROOT / "Cargo.lock"]
    hashes = {str(p.relative_to(ROOT)): hashlib.sha256(p.read_bytes()).hexdigest() for p in inputs}
    (EVIDENCE / "input-hashes.json").write_text(json.dumps(hashes, indent=2) + "\n")
    report = {"task_id": "YT-078", "status": "running", "checks": [], "snapshots": [],
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
        raise TimeoutError("YT-078 180-second focused test deadline")

    previous_handler = signal.signal(signal.SIGALRM, timeout)
    signal.setitimer(signal.ITIMER_REAL, 180)
    try:
        with tempfile.TemporaryDirectory(prefix="yt078-ff-", dir=snap_common) as tmp:
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
                        action("body", "fresh document " + label, lambda _key, _before: request("POST", driver_url, prefix + "/url", {"url": base + "/scene?yt078=" + label}))
                        wait_until(lambda: evaluate("return !!window.__yt078Selection && document.readyState==='complete'"), "selection loader", 20)
                        script("#scene-canvas", "scroll canvas into view " + label, "document.getElementById('scene-canvas').scrollIntoView({block:'center'})")

                    def ready():
                        wait_until(lambda: evaluate("return window.__yt075Scene.snapshot().state==='ready' && window.__yt075Scene.snapshot().draws>=1"), "drawn GPU ready", 35)
                        frames()

                    def pointer(id, label):
                        script("#scene-canvas", "visible input target " + label, "document.getElementById('scene-canvas').scrollIntoView({block:'center'})")
                        def send(key, before):
                            css = before["css"]
                            x = round(css["left"] + ((id - 1) % 12 + .5) * css["width"] / 12)
                            y = round(css["top"] + ((id - 1) // 12 + .5) * css["height"] / 10)
                            return request("POST", driver_url, prefix + "/actions", {"actions": [{"type": "pointer", "id": "yt078-mouse", "parameters": {"pointerType": "mouse"}, "actions": [
                                {"type": "pointerMove", "duration": 0, "origin": "viewport", "x": x, "y": y},
                                {"type": "pointerDown", "button": 0}, {"type": "pointerUp", "button": 0}]}]})
                        result = action("#scene-canvas", label, send)
                        assert result["selection"]["selectedRecord"] == id and f"{id:03}" in result["detail"]
                        assert result["href"] == f"/scene?record={id}"

                    def pixels(label):
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
                            expected = ([255, 0, 0], [0, 255, 0], [0, 0, 255])[index % 3]
                            if any(abs(a-b) > 25 for a, b in zip(actual, expected)):
                                mismatches.append({"id": index + 1, "capture": [x, y], "actual": actual, "expected": expected})
                        black = list(rows[top + 2][2 * channels:2 * channels + 3])
                        report["pixels"].append({"label": label, "width": width, "height": height, "capture_canvas_origin": [0, top],
                            "verified": 120 - len(mismatches), "mismatches": mismatches, "interior_black": black, "sha256": hashlib.sha256(png).hexdigest()})
                        observe("#scene-canvas", "hardware pixels " + label + " after")
                        assert not mismatches and black == [0, 0, 0], "Actual hardware canvas proof failed"

                    # First navigation starts from the browser-owned blank document.
                    fresh("picking")
                    action("body", "desktop browser viewport", lambda _key, _before: request("POST", driver_url, prefix + "/window/rect", {"width": 1100, "height": 1000}))
                    script("#scene-canvas", "visible desktop canvas", "document.getElementById('scene-canvas').scrollIntoView({block:'center'})")
                    adapter = async_eval("const done=arguments[arguments.length-1];navigator.gpu.requestAdapter().then(a=>done(a?{fallback:a.isFallbackAdapter??a.info?.isFallbackAdapter??null,info:a.info?{vendor:a.info.vendor,architecture:a.info.architecture,device:a.info.device,description:a.info.description}:null}:null));")
                    report["adapter"] = adapter
                    assert adapter and not adapter["fallback"], "Actual non-fallback hardware adapter required"
                    click("#activate-scene", "explicit GPU activation")
                    ready()
                    pixels("initial")
                    pointer(1, "actual mouse selects ID 1")
                    pointer(120, "actual mouse selects ID 120")
                    click("#scene-navigate", "focus labeled keyboard selection")
                    keys("#scene-navigate", "keyboard Home chooses ID 1", "\ue011")
                    keys("#scene-navigate", "keyboard Right chooses ID 2", "\ue014")
                    keys("#scene-navigate", "keyboard Down chooses ID 14", "\ue015")
                    selected = keys("#scene-navigate", "keyboard Enter selects ID 14", "\ue007")
                    assert selected["selection"]["selectedRecord"] == 14 and selected["href"] == "/scene?record=14"
                    keys("#scene-navigate", "keyboard End chooses final ID", "\ue010")
                    selected = keys("#scene-navigate", "keyboard Space selects final ID", " ")
                    assert selected["selection"]["selectedRecord"] == 120
                    reset = click("#scene-reset", "keyboard alternative reset")
                    assert reset["selection"]["selectedRecord"] is None and reset["href"] is None
                    script("#scene-canvas", "resize CSS with simulated DPR 3 clamped to 2", """const c=document.getElementById('scene-canvas');c.style.width='300px';c.style.height='200px';
                        Object.defineProperty(window,'devicePixelRatio',{value:3,configurable:true});window.dispatchEvent(new Event('resize'));""")
                    ready()
                    _, mapped = observe("#scene-canvas", "DPR mapping assertion")
                    assert mapped["scene"]["dpr"] == 2 and mapped["scene"]["backing"] == {"width": 600, "height": 400}
                    pixels("resized-injected-dpr")
                    pointer(13, "actual mouse after CSS/DPR remapping selects ID 13")
                    # Synthetic PointerEvents are explicitly not physical mobile evidence.
                    touch = """const c=document.getElementById('scene-canvas'),b=c.getBoundingClientRect();
                        const init={bubbles:true,pointerId:78,pointerType:'touch',isPrimary:true,button:0,clientX:b.left+b.width*1.5/12,clientY:b.top+b.height*.5/10};
                        c.dispatchEvent(new PointerEvent('pointerdown',init));MID;c.dispatchEvent(new PointerEvent('pointerup',init));"""
                    tapped = script("#scene-canvas", "SIMULATED touch tap selects ID 2", touch.replace("MID", ""))
                    assert tapped["selection"]["selectedRecord"] == 2
                    cancelled = script("#scene-canvas", "SIMULATED touch pointercancel preserves ID 2", touch.replace("1.5/12", "2.5/12").replace("MID", "c.dispatchEvent(new PointerEvent('pointercancel',init))"))
                    assert cancelled["selection"]["selectedRecord"] == 2 and not cancelled["selection"]["tapPending"]
                    scrolled = script("#scene-canvas", "SIMULATED touch scroll cancellation preserves ID 2", touch.replace("1.5/12", "2.5/12").replace("MID", "window.dispatchEvent(new Event('scroll'))"))
                    assert scrolled["selection"]["selectedRecord"] == 2
                    moved = script("#scene-canvas", "SIMULATED touch move cancellation preserves ID 2", touch.replace("1.5/12", "2.5/12").replace("MID", "c.dispatchEvent(new PointerEvent('pointermove',{...init,clientX:init.clientX+20}))"))
                    assert moved["selection"]["selectedRecord"] == 2
                    assert evaluate("return getComputedStyle(document.getElementById('scene-canvas')).touchAction.includes('pan-y')")
                    script("#scene-canvas", "capture deferred old-data pick", "const b=document.getElementById('scene-canvas').getBoundingClientRect();window.__yt078Old=window.__yt078Selection.capturePick(b.left+b.width/24,b.top+b.height/20)")
                    action("#scene-canvas", "prepare same-ID replacement records", lambda _key, _before: async_eval("const done=arguments[arguments.length-1];window.__yt075Scene.records(120,0).then(data=>{window.__yt078Data=data;done(null)});"))
                    replaced = script("#scene-canvas", "replace data then reject deferred stale-version pick", "window.__yt075Scene.replace(window.__yt078Data);return window.__yt078Selection.commitPick(window.__yt078Old)")
                    assert replaced["selection"]["selectedRecord"] is None and replaced["selection"]["rejectedPicks"] > scrolled["selection"]["rejectedPicks"]
                    pixels("replaced")
                    script("#scene-canvas", "capture old-generation pick", "const b=document.getElementById('scene-canvas').getBoundingClientRect();window.__yt078Old=window.__yt078Selection.capturePick(b.left+b.width/24,b.top+b.height/20)")
                    disposed = click("#dispose-scene", "owned disposal")
                    assert disposed["scene"]["ownedSurfaces"] == disposed["scene"]["ownedBufferBytes"] == 0
                    rejected = script("#scene-canvas", "reject deferred pick after disposal", "return window.__yt078Selection.commitPick(window.__yt078Old)")
                    assert rejected["selection"]["selectedRecord"] is None
                    click("#activate-scene", "explicit replacement generation")
                    ready()
                    rejected = script("#scene-canvas", "reject old generation after replacement", "return window.__yt078Selection.commitPick(window.__yt078Old)")
                    assert rejected["selection"]["selectedRecord"] is None
                    # Actual portrait browser viewport; no physical phone qualification.
                    action("#scene-canvas", "390 CSS-pixel portrait viewport via BiDi", lambda _key, before: portrait_viewport(capabilities["webSocketUrl"], before["url"]))
                    script("#scene-canvas", "portrait responsive aspect layout", "const c=document.getElementById('scene-canvas');c.style.width='';c.style.height='';c.scrollIntoView({block:'center'})")
                    ready()
                    pixels("portrait")
                    pointer(3, "portrait actual mouse selects ID 3")
                    _, portrait = observe("#scene-navigate", "portrait labels/data/actions")
                    assert portrait["records"] == 12 and portrait["describedBy"] == "selection-help"
                    report["portrait_layout"] = evaluate("return {innerWidth,innerHeight,outerWidth,scrollWidth:document.documentElement.scrollWidth,records:document.querySelectorAll('#records li').length}")
                    assert report["portrait_layout"]["innerWidth"] == 390 and report["portrait_layout"]["innerHeight"] > 390 and report["portrait_layout"]["scrollWidth"] <= 390 and report["portrait_layout"]["records"] == 12, "390-pixel portrait viewport must preserve data without overflow"
                    key, _ = observe("#scene-navigate", "accessible keyboard name")
                    report["accessible_navigation_name"] = request("GET", driver_url, prefix + f"/element/{key}/computedlabel")
                    assert "Select record" in report["accessible_navigation_name"]
                    key, _ = observe("#scene-canvas", "accessible canvas name")
                    assert "HTML record list" in request("GET", driver_url, prefix + f"/element/{key}/computedlabel")
                    motion = evaluate("""const c=document.getElementById('scene-canvas'),s=getComputedStyle(c);return {matches:matchMedia('(prefers-reduced-motion: reduce)').matches,
                        animation:s.animationName,transition:s.transitionDuration,mediaRule:[...document.styleSheets].some(sheet=>[...sheet.cssRules].some(rule=>rule.conditionText==='(prefers-reduced-motion: reduce)'))};""")
                    assert motion["animation"] == "none" and motion["transition"] == "0s" and motion["mediaRule"]
                    report["reduced_motion"] = {**motion, "policy": "Static demand rendering in both modes; actual match recorded, no OS preference injection or animation"}
                    click("#dispose-scene", "portrait final GPU teardown")
                    _, teardown = observe("#scene-canvas", "teardown counters")
                    assert teardown["scene"]["lastRetired"]["resourcesReleased"] and teardown["scene"]["observers"] == teardown["scene"]["listeners"] == 0
                    assert not teardown["scene"]["rafPending"]
                    fresh("unavailable")
                    script("#activate-scene", "INJECT unavailable GPU capability", "Object.defineProperty(navigator,'gpu',{value:undefined,configurable:true})")
                    click("#activate-scene", "activate unavailable fallback")
                    wait_until(lambda: evaluate("return window.__yt075Scene.snapshot().state==='unavailable'"), "unavailable GPU state", 15)
                    click("#scene-navigate", "fallback keyboard selection")
                    keys("#scene-navigate", "fallback End selection", "\ue010")
                    fallback = keys("#scene-navigate", "fallback Enter detail", "\ue007")
                    assert fallback["selection"]["selectedRecord"] == 120 and fallback["href"] == "/scene?record=120"
                    count = click("#counter-increment", "independent counter without GPU")
                    assert count["counter"] == "3"
                    # Native text entry stays scoped to the search input, including IME events.
                    before = fallback["selection"]
                    keys("#query", "native text search input", "done")
                    unchanged = keys("#query", "search ArrowLeft does not navigate scene", "\ue012")
                    assert unchanged["selection"]["focusedRecord"] == before["focusedRecord"]
                    ime = script("#query", "SIMULATED composing key is not intercepted", """const q=document.getElementById('query');const e=new KeyboardEvent('keydown',{key:'ArrowDown',bubbles:true,cancelable:true,isComposing:true});
                        q.dispatchEvent(new CompositionEvent('compositionstart',{data:'中'}));q.dispatchEvent(e);q.dispatchEvent(new CompositionEvent('compositionend',{data:'中'}));return e.defaultPrevented;""")
                    assert ime["selection"]["focusedRecord"] == before["focusedRecord"]
                    # Scriptless real SSR page: remove only the owned temporary manifest.
                    # This proves no-JS HTML data/navigation; it is not an OS JS preference change.
                    saved_manifest = (asset_root / "manifest.json").read_bytes()
                    (asset_root / "manifest.json").unlink()
                    try:
                        nojs = action("#query", "scriptless SSR search/pagination/detail", lambda _key, _before: request("POST", driver_url, prefix + "/url", {"url": base + "/scene?q=done&page=1&record=38"}))
                        assert nojs["records"] == 12 and nojs["counter"] == "2"
                        assert evaluate("return !document.querySelector('script') && document.getElementById('matches').textContent.includes('40 matches; page 2 of 4') && document.getElementById('detail').textContent.includes('Work record 038')")
                        opened = click("#records a", "scriptless real record navigation")
                        assert "record=38" in opened["url"] and evaluate("return document.getElementById('detail').textContent.includes('Work record 038')")
                        report["scriptless_ssr"] = "No script tags/assets; actual SSR search, page and record link navigation; not a browser JS-disabled preference claim"
                    finally:
                        (asset_root / "manifest.json").write_bytes(saved_manifest)
                    report["status"] = "passed_yt078_picking"
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
    return 0 if (report["status"] == "passed_yt078_picking" and report["inputs_unchanged"]
                 and report["cleanup"].get("session_deleted") and report["cleanup"]["tmp_profile_removed"]
                 and report["cleanup"]["processes_reaped"] and report["cleanup"]["browser_exited"]
                 and all(report["cleanup"]["ports_closed"].values()) and not any(k.endswith("_error") for k in report["cleanup"])) else 1


if __name__ == "__main__":
    raise SystemExit(main())
