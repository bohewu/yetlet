#!/usr/bin/env python3
"""ONE focused YT-077 actual Rust/wgpu versus Canvas2D Firefox measurement.
Prepared for parent execution only. No installation, persistent preferences or
shared smoke infrastructure changes. Refuses existing output. Samples are real;
a deadline omission is explicit and cannot qualify as an inferior renderer.
"""
from __future__ import annotations

import base64
import gzip
import hashlib
import json
import math
import os
from pathlib import Path
import shutil
import signal
import socket
import subprocess
import tempfile
import time

from verify_firefox import ELEMENT_KEY, request
from verify_browser import png_pixels
from smoke_browser import http, port_number, stop_owned_process, wait_until

HERE = Path(__file__).resolve().parent
ROOT = HERE.parent.parent
# Same YT-077 measurement task; preserve the completed first-attempt failure.
EVIDENCE = ROOT / "target/m11-gpu/yt077-20261014/browser/capture-alignment-fix"
SIZES = [1000, 10000, 50000, 100000]
WORKLOADS = ["sparse", "full", "burst"]
# Fixed before execution: inherited YT-076 screenshot tolerance, no post-result adjustment.
PIXEL_TOLERANCE = 25


def distributions(samples):
    """Nearest-rank p50/p95/p99 across 30 measured update-to-result samples.
    Canvas completionWaitMs is absent, never a zero-time GPU comparison.
    """
    output = {}
    for key in ("prepareMs", "applyMs", "uploadCPUms", "encodeSubmitCPUms",
                "drawObservedMs", "completionWaitMs", "completionWallMs",
                "rafOpportunity1Ms", "rafOpportunity2Ms"):
        values = sorted(s[key] for s in samples if s.get(key) is not None)
        if values:
            output[key] = {f"p{p}": values[math.ceil(p / 100 * len(values)) - 1] for p in (50, 95, 99)}
            output[key]["n"] = len(values)
    return output


def main():
    assert os.environ.get("YT075_FIREFOX_HEADFUL_X11") == "1", "Use headful X11 only"
    assert not EVIDENCE.exists(), "Existing YT-077 browser evidence; do not overwrite or retry"
    firefox, driver = shutil.which("firefox"), shutil.which("geckodriver")
    binary = HERE / "target/debug/yetlet-webgpu-scene-island"
    assets = ROOT / "target/webgpu-scene-island-dist"
    snap_common = Path.home() / "snap/firefox/common"
    assert firefox and driver and binary.is_file() and snap_common.is_dir(), "Existing Firefox fixture unavailable"
    manifest = json.loads((assets / "manifest.json").read_text())
    sources = [p for p in HERE.iterdir() if p.suffix in (".js", ".py") or p.name in ("Cargo.toml", "Cargo.lock")]
    sources += list((HERE / "src").glob("*"))
    paths = sources + [binary, ROOT / "Cargo.lock", assets / "manifest.json"]
    bundle = []
    for key, module in (("scene", "scene"), ("counter", "yetlet_dashboard_counter")):
        names = [f"{key}-loader.js", f"{module}.js", f"{module}_bg.wasm"]
        if key == "scene": names += ["comparison-loader.js"]
        for name in names:
            path = assets / "assets" / manifest[key]["digest"] / name
            paths.append(path)
            content = path.read_bytes()
            bundle.append({"entry": key, "name": name, "rawBytes": len(content),
                           "gzipBytes": len(gzip.compress(content, mtime=0)),
                           "sha256": hashlib.sha256(content).hexdigest()})
    hashes = {str(p.relative_to(ROOT)): hashlib.sha256(p.read_bytes()).hexdigest() for p in paths}
    EVIDENCE.mkdir(parents=True, exist_ok=False)
    (EVIDENCE / "input-hashes.json").write_text(json.dumps(hashes, indent=2) + "\n")
    report = {"task_id": "YT-077", "status": "running", "lanes": [], "omitted": [],
              "pixels": [], "bundle": bundle, "manifest": manifest, "input_hashes": hashes,
              "method": {"sizes": SIZES, "rounds": 3, "warmups": 5, "samples": 30,
                         "workloads": WORKLOADS, "burst_operations": 8, "seed": 7,
                         "css": [600, 400], "dpr": 1, "sustained_seconds": 10,
                         "percentile": "nearest rank; one final-state demand draw per sample",
                         "pixel_tolerance_per_channel": PIXEL_TOLERANCE,
                         "observational_budget_ms": 16.67, "gpu_timestamps": "not requested",
                         "browser_process_bound_seconds": 480, "measurement_cutoff_seconds": 420},
              "profile_changes": ["dom.webgpu.enabled=true (temporary only)"],
              "owned": {}, "cleanup": {}, "os_rss": "not observed", "vram": "not observed"}
    server = gecko = session = None
    driver_url = ""
    tmp_path = None
    started = time.monotonic()
    canonical_order = [(round_, size, kind) for round_ in range(3) for size in SIZES
                       for kind in (["canvas2d", "gpu"] if round_ % 2 == 0 else ["gpu", "canvas2d"])]
    proof_by_key = {}

    def timeout(_signal, _frame):
        raise TimeoutError("YT-077 420-second measurement deadline; remaining lanes bounded out")

    previous_handler = signal.signal(signal.SIGALRM, timeout)
    signal.setitimer(signal.ITIMER_REAL, 420)
    try:
        with tempfile.TemporaryDirectory(prefix="yt077-ff-", dir=snap_common) as tmp:
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
                    wait_until(lambda: http(base + "/healthz")[0] == 200, "owned server", 20)
                    code, headers, html = http(base + "/scene?q=done&page=1")
                    assert code == 200 and b"40 matches; page 2 of 4" in html
                    assert b'<html lang="en">' in html and "unsafe-inline" not in headers["content-security-policy"]
                    assert b"comparison-loader.js" not in html
                    report["default_ssr"] = "120 records, search/pagination, independent counter, strict CSP"
                    gecko = subprocess.Popen([driver, "--host", "127.0.0.1", "--port", str(driver_port), "--log", "info"],
                        cwd=ROOT, env=env, stdout=glog, stderr=subprocess.STDOUT, start_new_session=True)
                    report["owned"]["geckodriver_pid"] = gecko.pid
                    wait_until(lambda: http(driver_url + "/status")[0] == 200, "owned geckodriver", 20)
                    browser_started = time.monotonic()
                    response = request("POST", driver_url, "/session", {"capabilities": {"alwaysMatch": {
                        "browserName": "firefox", "moz:firefoxOptions": {"args": [], "prefs": {"dom.webgpu.enabled": True}}
                    }}})
                    session = response["sessionId"]
                    caps = response.get("capabilities", {})
                    report["capabilities"] = {k: caps.get(k) for k in ("browserName", "browserVersion", "platformName", "moz:processID", "moz:profile")}
                    assert caps.get("browserVersion", "").startswith("158."), "Selected Firefox 158 required"
                    prefix = f"/session/{session}"
                    request("POST", driver_url, prefix + "/timeouts", {"script": 17000, "pageLoad": 20000})
                    request("POST", driver_url, prefix + "/window/rect", {"width": 1100, "height": 1000})

                    def evaluate(script, args=None):
                        return request("POST", driver_url, prefix + "/execute/sync", {"script": script, "args": args or []})

                    def async_eval(body, args=None):
                        return request("POST", driver_url, prefix + "/execute/async", {"script":
                            "const done=arguments[arguments.length-1];(async()=>{" + body +
                            "})().then(value=>done({ok:true,value})).catch(e=>done({ok:false,error:String(e)}));", "args": args or []})

                    def actual(body, args=None):
                        result = async_eval(body, args)
                        assert result["ok"], result.get("error")
                        return result.get("value")  # null is a normal WebDriver value

                    def pixels(label):
                        proof = actual("return await window.__yt077.capture();")
                        element = request("POST", driver_url, prefix + "/element", {"using": "css selector", "value": proof["captureSelector"]})
                        png = base64.b64decode(request("GET", driver_url, prefix + f"/element/{element[ELEMENT_KEY]}/screenshot"))
                        (EVIDENCE / f"{label}.png").write_bytes(png)
                        width, height, channels, rows = png_pixels(png)
                        assert (width, height) == (600, 400), "Matched capture shape required"
                        # Firefox's known all-white top crop row is outside this RGB/black
                        # canvas. Map canvas coordinates into the screenshot; never search
                        # for a matching expected color or relax the fixed tolerance.
                        top_crop_rows = int(all(list(rows[0][x * channels:x * channels + 3]) == [255, 255, 255]
                                                for x in range(width)))
                        mismatches = []
                        for point in proof["samples"]:
                            x, y = point["x"], point["y"] + top_crop_rows
                            assert 0 <= x < width and 0 <= y < height, "Sample outside visible canvas crop"
                            color = list(rows[y][x * channels:x * channels + 3])
                            if any(abs(a-b) > PIXEL_TOLERANCE for a, b in zip(color, point["rgb"])):
                                mismatches.append({**point, "captureY": y, "actual": color})
                        x, canvas_y = proof["interiorBlack"]
                        canvas_y = min(canvas_y, height - 1 - top_crop_rows)
                        y = canvas_y + top_crop_rows
                        black = list(rows[y][x * channels:x * channels + 3])
                        item = {**proof, "label": label, "verified": len(proof["samples"]) - len(mismatches),
                                "mismatches": mismatches[:12], "black": black,
                                "capture_canvas_origin": [0, top_crop_rows],
                                "black_canvas_sample": [x, canvas_y], "black_capture_sample": [x, y],
                                "capture_top_edge": list(rows[0][:3]), "sha256": hashlib.sha256(png).hexdigest()}
                        report["pixels"].append(item)
                        assert not mismatches and black == [0, 0, 0], "Predeclared bounded pixel correctness failed"
                        return proof

                    for order, (round_, size, kind) in enumerate(canonical_order):
                        if time.monotonic() - started > 400:
                            report["omitted"] += [{"round": r, "size": s, "renderer": k, "reason": "bounded out before lane; not measured"}
                                                  for r, s, k in canonical_order[order:]]
                            break
                        label = f"r{round_}-{size}-{kind}"
                        request("POST", driver_url, prefix + "/url", {"url": base + "/scene-comparison?lane=" + label})
                        wait_until(lambda: evaluate("return !!window.__yt077 && document.readyState==='complete'"), "comparison entry", 20)
                        assert evaluate("return window.__yt075Scene.snapshot().starts===0"), "Implicit GPU activation"
                        assert evaluate("return !performance.getEntriesByType('resource').some(x=>x.name.includes('scene_bg.wasm'))"), "Preactivation GPU Wasm fetch"
                        if kind == "gpu":
                            adapter = actual("const a=await navigator.gpu.requestAdapter();return a?{vendor:a.info?.vendor,architecture:a.info?.architecture,fallback:a.isFallbackAdapter??a.info?.isFallbackAdapter??null}:null;")
                            assert adapter and adapter["fallback"] is False, "Actual non-fallback GPU required"
                            report["adapter"] = adapter
                        startup = actual("return await window.__yt077.setup(arguments[0],arguments[1]);", [kind, size])
                        lane = {"round": round_, "size": size, "renderer": kind, "order": order,
                                "startup": startup, "workloads": [], "status": "running"}
                        report["lanes"].append(lane)
                        pixels(label + "-initial")
                        for workload in WORKLOADS:
                            raw = []
                            # Three small chunks preserve partial real samples even on deadline/error.
                            for start_index in (0, 10, 20, 30):
                                number = min(10, 35 - start_index)
                                samples = actual("const out=[];for(let i=0;i<arguments[1];i++)out.push(await window.__yt077.sample(arguments[0],arguments[2]+i<5));return out;", [workload, number, start_index])
                                raw.extend(samples)
                                (EVIDENCE / f"{label}-{workload}-samples.json").write_text(json.dumps(raw, indent=2) + "\n")
                            measured = [s for s in raw if not s["warmup"]]
                            assert len(measured) == 30 and len(raw) == 35
                            proof = pixels(label + "-" + workload)
                            key = (round_, size, workload)
                            identity = {k: proof[k] for k in ("sequence", "checksum", "samples")}
                            if key in proof_by_key: assert proof_by_key[key] == identity, "Unequal renderer final state"
                            else: proof_by_key[key] = identity
                            lane["workloads"].append({"name": workload, "samples": len(measured), "warmups": 5,
                                                      "distributions": distributions(measured), "state": identity})
                        if size == 10000:
                            # Backpressure: each sample completes before the next; no queued backlog.
                            sustained = actual("const out=[],begin=performance.now();while(performance.now()-begin<10000)out.push(await window.__yt077.sample('sparse'));return {elapsedMs:performance.now()-begin,samples:out};")
                            (EVIDENCE / f"{label}-sustained-samples.json").write_text(json.dumps(sustained, indent=2) + "\n")
                            lane["sustained"] = {"elapsedMs": sustained["elapsedMs"], "samples": len(sustained["samples"]),
                                                  "distributions": distributions(sustained["samples"]),
                                                  "policy": "10-second sequential sparse lane, per-sample counters; completion is not FPS"}
                            pixels(label + "-sustained")
                        # Full replacement is separately checked, outside measured status workloads.
                        actual("return await window.__yt077.replace(9);")
                        replacement = pixels(label + "-replacement")
                        key = (round_, size, "replacement")
                        identity = {k: replacement[k] for k in ("sequence", "checksum", "samples")}
                        if key in proof_by_key: assert proof_by_key[key] == identity, "Unequal replacement state"
                        else: proof_by_key[key] = identity
                        evaluate("document.getElementById('counter-increment').click()")
                        assert evaluate("return document.getElementById('counter-value').textContent") == "3"
                        disposed = evaluate("return window.__yt077.dispose()")
                        assert disposed["appCPUBytes"] == 0 and not disposed["canvasRafPending"]
                        if kind == "gpu":
                            assert disposed["gpu"]["ownedBufferBytes"] == disposed["gpu"]["listeners"] == disposed["gpu"]["observers"] == 0
                            assert disposed["gpu"]["retainedJSBytes"] == 0
                        lane["cleanup"] = disposed
                        lane["status"] = "completed"
                        (EVIDENCE / "result.json").write_text(json.dumps(report, indent=2) + "\n")
                    report["status"] = "bounded_partial" if report["omitted"] else "passed_yt077_measurement"
                finally:
                    # Reserve 60 seconds for owned cleanup before the 8-minute process bound.
                    signal.setitimer(signal.ITIMER_REAL, 0)
                    if session is not None:
                        try:
                            request("DELETE", driver_url, f"/session/{session}")
                            report["cleanup"]["session_deleted"] = True
                        except Exception as exc:
                            report["cleanup"]["session_delete_error"] = str(exc)[:250]
                    for name, proc in (("geckodriver", gecko), ("server", server)):
                        try:
                            stop_owned_process(proc, process_group=True)
                            report["cleanup"][name + "_exit"] = proc.returncode if proc else None
                        except Exception as exc:
                            report["cleanup"][name + "_error"] = str(exc)[:250]
                    if "browser_started" in locals(): report["browser_process_seconds"] = time.monotonic() - browser_started
    except BaseException as exc:
        report["status"] = "bounded_partial" if isinstance(exc, TimeoutError) else "failed"
        report["error"] = type(exc).__name__ + ": " + str(exc)[:600]
        completed = {(lane["round"], lane["size"], lane["renderer"]) for lane in report["lanes"] if lane["status"] == "completed"}
        report["omitted"] = [{"round": r, "size": s, "renderer": k, "reason": "incomplete/not measured; no losing-renderer claim"}
                             for r, s, k in canonical_order if (r, s, k) not in completed]
    finally:
        signal.setitimer(signal.ITIMER_REAL, 0)
        signal.signal(signal.SIGALRM, previous_handler)
        cleanup = report["cleanup"]
        cleanup["tmp_profile_removed"] = tmp_path is None or not tmp_path.exists()
        cleanup["processes_reaped"] = all(p is None or p.poll() is not None for p in (server, gecko))
        pid = report.get("capabilities", {}).get("moz:processID")
        if pid:
            try: os.kill(pid, 0); cleanup["browser_exited"] = False
            except ProcessLookupError: cleanup["browser_exited"] = True
        else: cleanup["browser_exited"] = session is None
        cleanup["ports_closed"] = {}
        for name in ("server_port", "driver_port"):
            if name in report["owned"]:
                try:
                    with socket.create_connection(("127.0.0.1", report["owned"][name]), timeout=.3): cleanup["ports_closed"][name] = False
                except OSError: cleanup["ports_closed"][name] = True
        report["inputs_unchanged"] = all(p.is_file() and hashlib.sha256(p.read_bytes()).hexdigest() == hashes[str(p.relative_to(ROOT))] for p in paths)
        report["elapsed_seconds"] = time.monotonic() - started
        (EVIDENCE / "result.json").write_text(json.dumps(report, indent=2) + "\n")
        print(json.dumps({k: report.get(k) for k in ("status", "error", "omitted", "browser_process_seconds", "cleanup", "inputs_unchanged")}))
    return 0 if (report["status"] == "passed_yt077_measurement" and report["inputs_unchanged"]
                 and report.get("browser_process_seconds", 481) <= 480
                 and cleanup.get("session_deleted") and cleanup["tmp_profile_removed"]
                 and cleanup["processes_reaped"] and cleanup["browser_exited"]
                 and all(cleanup["ports_closed"].values()) and not any(k.endswith("_error") for k in cleanup)) else 1


if __name__ == "__main__":
    raise SystemExit(main())
