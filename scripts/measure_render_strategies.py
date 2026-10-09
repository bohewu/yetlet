#!/usr/bin/env python3
"""Run the isolated M7a benchmark in headless Chrome and retain JSON evidence."""
from __future__ import annotations

import argparse
from contextlib import contextmanager
from datetime import datetime, timezone
import gzip
import hashlib
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer
import json
import os
from pathlib import Path
import platform
import shutil
import subprocess
import sys
import tempfile
import threading
import time

from smoke_browser import ChromeDevTools, http, port_number, wait_until

ROOT = Path(__file__).resolve().parent.parent
TARGET = ROOT / "target" / "m7a-render-bench"


class QuietStaticHandler(SimpleHTTPRequestHandler):
    def log_message(self, _format: str, *_args) -> None:
        pass


@contextmanager
def static_server(directory: Path):
    server = ThreadingHTTPServer(
        ("127.0.0.1", 0),
        lambda *args, **kwargs: QuietStaticHandler(
            *args, directory=str(directory), **kwargs
        ),
    )
    thread = threading.Thread(target=server.serve_forever, daemon=True)
    thread.start()
    try:
        yield f"http://127.0.0.1:{server.server_port}"
    finally:
        server.shutdown()
        server.server_close()
        thread.join(timeout=5)


def version(command: list[str]) -> str:
    result = subprocess.run(command, capture_output=True, text=True, timeout=10)
    if result.returncode != 0:
        raise RuntimeError(f"Cannot determine version: {command[0]}")
    return (result.stdout.strip() or result.stderr.strip()).splitlines()[0]


def benchmark_assets(profile_root: Path, manifest: dict) -> dict[str, dict[str, object]]:
    digest = manifest.get("digest")
    prefix = f"assets/{digest}/"
    result = {}
    for name in ("entry", "module", "wasm", "stylesheet"):
        value = manifest.get(name)
        if not isinstance(value, str) or not value.startswith(prefix):
            raise RuntimeError(f"Invalid benchmark manifest field: {name}")
        file_path = profile_root / value
        content = file_path.read_bytes()
        result[name] = {
            "path": value,
            "bytes": len(content),
            "gzip_bytes": len(gzip.compress(content, compresslevel=9, mtime=0)),
            "sha256": hashlib.sha256(content).hexdigest(),
        }
    return result


def production_assets(path: Path) -> dict:
    manifest = json.loads((path / "manifest.json").read_text(encoding="utf-8"))
    digest = manifest["digest"]
    asset_root = path / "assets" / digest
    return {
        "digest": digest,
        "mode": manifest["mode"],
        "files": {
            item.name: {
                "bytes": item.stat().st_size,
                "sha256": hashlib.sha256(item.read_bytes()).hexdigest(),
            }
            for item in sorted(asset_root.iterdir())
            if item.is_file()
        },
    }


def validate_report(report: dict, rows: int, updates: int, rounds: int) -> None:
    config = report["config"]
    assert config["rows"] == rows
    assert config["updates_per_round"] == updates
    assert config["rounds"] == rounds
    strategies = {item["strategy"]: item for item in report["strategies"]}
    expected = {
        "subtree_replacement",
        "view_tree_patch",
        "keyed_direct_dom",
        "static_template_slots",
    }
    assert set(strategies) == expected
    for item in strategies.values():
        assert item["final_state_equivalent_all_rounds"] is True
        assert item["outside_focus_preserved_all_rounds"] is True
        assert item["setup"]["samples"] == rounds
        assert item["update"]["samples"] == rounds * updates
        assert item["structural"]["samples"] == rounds * 3
        assert len(item["final_state_checksums"]) == rounds
    checksum_sets = {
        tuple(item["final_state_checksums"]) for item in strategies.values()
    }
    assert len(checksum_sets) == 1, "strategies produced different final states"
    assert strategies["subtree_replacement"][
        "inside_input_identity_preserved_all_rounds"
    ] is False
    assert strategies["subtree_replacement"][
        "inside_focus_preserved_all_rounds"
    ] is False
    assert strategies["subtree_replacement"][
        "untouched_row_identity_preserved_all_rounds"
    ] is False
    assert strategies["view_tree_patch"][
        "inside_input_identity_preserved_all_rounds"
    ] is True
    assert strategies["view_tree_patch"][
        "inside_focus_preserved_all_rounds"
    ] is True
    assert strategies["view_tree_patch"][
        "untouched_row_identity_preserved_all_rounds"
    ] is False
    for name in ("keyed_direct_dom", "static_template_slots"):
        assert strategies[name]["inside_input_identity_preserved_all_rounds"] is True
        assert strategies[name]["inside_focus_preserved_all_rounds"] is True
        assert strategies[name]["untouched_row_identity_preserved_all_rounds"] is True


def run_browser(
    base: str,
    chrome: str,
    rows: int,
    updates: int,
    rounds: int,
    seed: int,
    log,
) -> dict:
    chrome_port = port_number()
    process = subprocess.Popen(
        [
            chrome,
            "--headless=new",
            "--no-sandbox",
            "--disable-gpu",
            "--disable-background-networking",
            "--disable-extensions",
            "--no-first-run",
            "--remote-debugging-address=127.0.0.1",
            "--remote-allow-origins=*",
            f"--remote-debugging-port={chrome_port}",
            f"--user-data-dir={Path(log.name).parent / 'chrome-profile'}",
            "about:blank",
        ],
        stdout=log,
        stderr=subprocess.STDOUT,
    )
    client = None
    try:
        endpoint = f"http://127.0.0.1:{chrome_port}/json/list"

        def target_url():
            targets = json.loads(http(endpoint)[2])
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
                "document.documentElement.dataset.benchmarkReady === 'true'"
            ),
            "benchmark Wasm readiness",
        )
        client.evaluate(
            "window.__yetletBenchResult=null;window.__yetletBenchError=null;"
            "setTimeout(()=>{try{window.__yetletBenchResult="
            f"window.runYetletBenchmark({rows},{updates},{rounds},{seed});"
            "}catch(error){window.__yetletBenchError=String(error);}},0);true"
        )
        wait_until(
            lambda: client.evaluate(
                "window.__yetletBenchResult !== null || "
                "window.__yetletBenchError !== null"
            ),
            "render benchmark completion",
            seconds=120,
        )
        error = client.evaluate("window.__yetletBenchError")
        if error:
            raise RuntimeError(f"Benchmark runtime failed: {error}")
        report = client.evaluate("window.__yetletBenchResult")
        if not isinstance(report, dict):
            raise RuntimeError("Benchmark did not return a JSON object")
        if client.websockets:
            raise RuntimeError(f"Unexpected WebSocket: {client.websockets}")
        if client.failed_responses:
            raise RuntimeError(f"Failed resources: {client.failed_responses}")
        if client.exceptions:
            raise RuntimeError(f"Browser errors: {client.exceptions}")
        return report
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
    parser.add_argument("--rows", type=int, default=200)
    parser.add_argument("--updates", type=int, default=500)
    parser.add_argument("--rounds", type=int, default=10)
    parser.add_argument("--seed", type=int, default=7)
    parser.add_argument(
        "--capture-production-before",
        type=Path,
        help=(
            "write the current production dist asset snapshot and exit; "
            "run this before building benchmark assets"
        ),
    )
    parser.add_argument(
        "--production-before",
        type=Path,
        default=ROOT / "target/m7a-production-before.json",
    )
    parser.add_argument(
        "--out", type=Path, default=ROOT / "target/m7a-render-strategies.json"
    )
    args = parser.parse_args()
    if args.capture_production_before is not None:
        snapshot = production_assets(ROOT / "dist")
        args.capture_production_before.parent.mkdir(parents=True, exist_ok=True)
        args.capture_production_before.write_text(
            json.dumps(snapshot, indent=2, sort_keys=True) + "\n", encoding="utf-8"
        )
        print(
            "PASS: captured production Browser/Search asset baseline",
            snapshot["digest"],
            args.capture_production_before,
        )
        return 0

    if not args.production_before.is_file():
        raise RuntimeError(
            "Missing required production baseline. First build production browser assets, "
            "then run this script with --capture-production-before PATH before building "
            "the M7a benchmark."
        )
    before = json.loads(args.production_before.read_text(encoding="utf-8"))
    current_production = production_assets(ROOT / "dist")
    if before != current_production:
        raise RuntimeError(
            "Production Browser/Search assets no longer match the captured pre-benchmark baseline"
        )
    if not 3 <= args.rows <= 1_000:
        parser.error("rows must be 3..1000")
    if not 1 <= args.updates <= 10_000:
        parser.error("updates must be 1..10000")
    if not 1 <= args.rounds <= 30:
        parser.error("rounds must be 1..30")

    profile_root = TARGET / args.profile
    manifest = json.loads((profile_root / "manifest.json").read_text(encoding="utf-8"))
    if manifest.get("profile") != args.profile:
        raise RuntimeError("Benchmark profile/manifest mismatch")
    assets = benchmark_assets(profile_root, manifest)
    chrome = shutil.which("google-chrome-stable") or shutil.which("google-chrome")
    if not chrome:
        raise RuntimeError("Headless Chrome is required")

    with tempfile.TemporaryDirectory(prefix="yetlet-m7a-bench-") as folder:
        with (Path(folder) / "chrome.log").open("w") as chrome_log:
            with static_server(profile_root) as base:
                report = run_browser(
                    base,
                    chrome,
                    args.rows,
                    args.updates,
                    args.rounds,
                    args.seed,
                    chrome_log,
                )
    validate_report(report, args.rows, args.updates, args.rounds)

    after = production_assets(ROOT / "dist")
    production_check = {
        "before_digest": before["digest"],
        "after_digest": after["digest"],
        "unchanged": before == after,
    }
    if before != after:
        raise RuntimeError("Production Browser/Search assets changed during M7a")

    output = {
        "schema_version": 1,
        "timestamp_utc": datetime.now(timezone.utc).isoformat(timespec="seconds"),
        "environment": {
            "os": platform.system(),
            "architecture": platform.machine(),
            "rustc": version([str(Path.home() / ".cargo/bin/rustc"), "--version"]),
            "chrome": version([chrome, "--version"]),
            "python": platform.python_version(),
        },
        "benchmark_manifest": manifest,
        "benchmark_assets": assets,
        "production_assets": production_check,
        "report": report,
    }
    args.out.parent.mkdir(parents=True, exist_ok=True)
    args.out.write_text(json.dumps(output, indent=2, sort_keys=True) + "\n", encoding="utf-8")

    print("PASS: M7a isolated render-strategy benchmark")
    print("Environment:", output["environment"])
    print("Benchmark digest:", manifest["digest"])
    print("Benchmark Wasm bytes:", assets["wasm"]["bytes"])
    print("Production assets unchanged:", production_check)
    for strategy in report["strategies"]:
        print(
            strategy["strategy"],
            "setup median", strategy["setup"]["median_ms"],
            "update median", strategy["update"]["median_ms"],
            "update p95", strategy["update"]["p95_ms"],
            "round median", strategy["total_round"]["median_ms"],
        )
    print("Report:", args.out)
    return 0


if __name__ == "__main__":
    try:
        sys.exit(main())
    except (AssertionError, OSError, RuntimeError, subprocess.CalledProcessError) as error:
        print(f"M7a measurement FAILED: {error!r}", file=sys.stderr)
        sys.exit(1)
