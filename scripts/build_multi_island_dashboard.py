#!/usr/bin/env python3
"""Build two independently owned dashboard WASM islands outside production dist/."""
from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
import re
import shutil
import subprocess
import sys
import tempfile
import tomllib

ROOT = Path(__file__).resolve().parent.parent
OUTPUT = ROOT / "target/multi-island-dashboard-dist"

# Package names are intentionally explicit for the repository sample checker.
COUNTER_PACKAGE = "yetlet-dashboard-counter-island"
FILTER_PACKAGE = "yetlet-dashboard-filter-island"

COUNTER_LOADER = r'''function installCounterIsland() {
  const root = document.querySelector('[data-yetlet-island="counter"]');
  if (!root) return;
  const dashboard = globalThis.__yetletDashboard ??= {};
  let modulePromise;
  let active = false;
  const load = () => modulePromise ??= import("./yetlet_dashboard_counter.js")
    .then(async module => { await module.default(); return module; });
  const activate = async () => {
    if (active) return;
    try {
      const module = await load();
      module.mount_counter();
      active = true;
      root.removeAttribute("data-yetlet-error");
    } catch (error) {
      active = false;
      root.setAttribute("data-yetlet-error", "mount-failed");
      console.error("Yetlet counter island failed", error);
      throw error;
    }
  };
  const dispose = async () => {
    if (modulePromise) {
      try { (await modulePromise).dispose_counter(); } catch (error) {
        root.setAttribute("data-yetlet-error", "dispose-failed");
        throw error;
      }
    }
    active = false;
  };
  dashboard.counter = Object.freeze({ activate, mount: activate, dispose });
  root.addEventListener("focusin", () => { activate().catch(() => {}); });
  root.addEventListener("yetlet-activate", () => { activate().catch(() => {}); });
  root.setAttribute("data-yetlet-loader-ready", "true");
}
if (document.readyState === "loading") {
  document.addEventListener("DOMContentLoaded", installCounterIsland, { once: true });
} else {
  installCounterIsland();
}
'''

FILTER_LOADER = r'''function installFilterIsland() {
  const root = document.querySelector('[data-yetlet-island="filter"]');
  if (!root) return;
  const dashboard = globalThis.__yetletDashboard ??= {};
  let modulePromise;
  let active = false;
  const load = () => modulePromise ??= import("./yetlet_dashboard_filter.js")
    .then(async module => { await module.default(); return module; });
  const activate = async () => {
    if (active) return;
    try {
      const module = await load();
      module.mount_filter();
      active = true;
      root.removeAttribute("data-yetlet-error");
    } catch (error) {
      active = false;
      root.setAttribute("data-yetlet-error", "mount-failed");
      console.error("Yetlet filter island failed", error);
      throw error;
    }
  };
  const dispose = async () => {
    if (modulePromise) {
      try { (await modulePromise).dispose_filter(); } catch (error) {
        root.setAttribute("data-yetlet-error", "dispose-failed");
        throw error;
      }
    }
    active = false;
  };
  dashboard.filter = Object.freeze({ activate, mount: activate, dispose });
  root.addEventListener("focusin", () => { activate().catch(() => {}); });
  root.addEventListener("yetlet-activate", () => { activate().catch(() => {}); });
  root.setAttribute("data-yetlet-loader-ready", "true");
}
if (document.readyState === "loading") {
  document.addEventListener("DOMContentLoaded", installFilterIsland, { once: true });
} else {
  installFilterIsland();
}
'''

CSS = r''':root { color-scheme: light dark; font-family: system-ui, sans-serif; }
body { max-width: 64rem; margin: 0 auto; padding: 1.5rem; line-height: 1.5; }
#dashboard-page { display: grid; gap: 1rem; }
section { border: 1px solid CanvasText; border-radius: .5rem; padding: 1rem; }
button, input { font: inherit; margin: .25rem; padding: .4rem .7rem; }
:focus-visible { outline: 3px solid Highlight; outline-offset: 2px; }
'''


def command(name: str, fallback: Path) -> str:
    tool = shutil.which(name) or (str(fallback) if fallback.exists() else None)
    if not tool:
        raise RuntimeError(f"Missing {name}")
    return tool


def execute(*argv: object) -> None:
    print("+", " ".join(map(str, argv)), flush=True)
    subprocess.run(list(map(str, argv)), cwd=ROOT, check=True)


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--release", action="store_true")
    options = parser.parse_args()
    cargo = command("cargo", Path.home() / ".cargo/bin/cargo")
    bindgen = command(
        "wasm-bindgen", Path.home() / ".local/share/yetlet-tools/bin/wasm-bindgen"
    )
    packages = tomllib.loads((ROOT / "Cargo.lock").read_text())["package"]
    versions = {package["version"] for package in packages if package["name"] == "wasm-bindgen"}
    if len(versions) != 1:
        raise RuntimeError("Expected exactly one wasm-bindgen version")
    version = versions.pop()
    if not re.search(rf"\b{re.escape(version)}\b", subprocess.check_output([bindgen, "--version"], text=True)):
        raise RuntimeError("wasm-bindgen CLI does not match Cargo.lock")

    release = ["--release"] if options.release else []
    for package in (COUNTER_PACKAGE, FILTER_PACKAGE):
        execute(
            cargo,
            "build",
            "--locked",
            "--target",
            "wasm32-unknown-unknown",
            "-p",
            package,
            *release,
        )
    profile = "release" if options.release else "debug"
    counter_wasm = ROOT / f"target/wasm32-unknown-unknown/{profile}/yetlet_dashboard_counter_island.wasm"
    filter_wasm = ROOT / f"target/wasm32-unknown-unknown/{profile}/yetlet_dashboard_filter_island.wasm"
    if not counter_wasm.is_file() or not filter_wasm.is_file():
        raise RuntimeError("Missing dashboard island Wasm artifacts")

    if OUTPUT.exists():
        shutil.rmtree(OUTPUT)
    OUTPUT.mkdir(parents=True)
    with tempfile.TemporaryDirectory(prefix="dashboard-islands-") as folder:
        stage = Path(folder)
        execute(
            bindgen,
            "--target",
            "web",
            "--out-dir",
            stage,
            "--out-name",
            "yetlet_dashboard_counter",
            counter_wasm,
        )
        execute(
            bindgen,
            "--target",
            "web",
            "--out-dir",
            stage,
            "--out-name",
            "yetlet_dashboard_filter",
            filter_wasm,
        )
        files = {
            "counter-loader.js": COUNTER_LOADER.encode(),
            "filter-loader.js": FILTER_LOADER.encode(),
            "dashboard.css": CSS.encode(),
            "yetlet_dashboard_counter.js": (stage / "yetlet_dashboard_counter.js").read_bytes(),
            "yetlet_dashboard_counter_bg.wasm": (stage / "yetlet_dashboard_counter_bg.wasm").read_bytes(),
            "yetlet_dashboard_filter.js": (stage / "yetlet_dashboard_filter.js").read_bytes(),
            "yetlet_dashboard_filter_bg.wasm": (stage / "yetlet_dashboard_filter_bg.wasm").read_bytes(),
        }
    digest = hashlib.sha256(
        b"yetlet-dashboard-v1\0"
        + b"".join(name.encode() + b"\0" + content + b"\0" for name, content in sorted(files.items()))
    ).hexdigest()[:16]
    asset_dir = OUTPUT / "assets" / digest
    asset_dir.mkdir(parents=True)
    for name, content in files.items():
        (asset_dir / name).write_bytes(content)
    prefix = f"/dashboard-assets/{digest}"
    manifest = {
        "schema_version": 1,
        "digest": digest,
        "mode": profile,
        "stylesheet": f"{prefix}/dashboard.css",
        "counter": {
            "loader": f"{prefix}/counter-loader.js",
            "module": f"{prefix}/yetlet_dashboard_counter.js",
            "wasm": f"{prefix}/yetlet_dashboard_counter_bg.wasm",
        },
        "filter": {
            "loader": f"{prefix}/filter-loader.js",
            "module": f"{prefix}/yetlet_dashboard_filter.js",
            "wasm": f"{prefix}/yetlet_dashboard_filter_bg.wasm",
        },
    }
    (OUTPUT / "manifest.json").write_text(json.dumps(manifest, indent=2) + "\n")
    print(
        "PASS dashboard assets:",
        digest,
        "counter Wasm bytes:",
        len(files["yetlet_dashboard_counter_bg.wasm"]),
        "filter Wasm bytes:",
        len(files["yetlet_dashboard_filter_bg.wasm"]),
    )


if __name__ == "__main__":
    try:
        main()
    except (OSError, RuntimeError, subprocess.CalledProcessError) as error:
        print("Dashboard build failed:", error, file=sys.stderr)
        sys.exit(1)
