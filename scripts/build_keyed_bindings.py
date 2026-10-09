#!/usr/bin/env python3
"""Build the isolated M7b keyed/static-slot authoring prototype."""
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
TARGET = ROOT / "target/m7b-keyed-bindings"
PACKAGE = "yetlet-keyed-bindings"
WASM_TARGET = "wasm32-unknown-unknown"
BOOTSTRAP = """import init, * as api from "./yetlet_keyed_bindings.js";
await init();
window.yetletBindings = api;
document.documentElement.setAttribute("data-yetlet-m7b-ready", "true");
document.documentElement.setAttribute("data-yetlet-m7c-ready", "true");
document.documentElement.setAttribute("data-yetlet-m7d-ready", "true");
"""
STYLES = """
:root { color-scheme: light dark; font-family: system-ui, sans-serif; }
body { max-width: 52rem; margin: 0 auto; padding: 1.5rem; line-height: 1.5; }
.m7b-list { display: grid; gap: .5rem; padding: 0; list-style: none; }
.m7b-row { display: grid; grid-template-columns: minmax(12rem, 1fr) minmax(10rem, 1fr) auto; gap: .5rem; align-items: center; }
.m7b-details { grid-column: 1 / -1; font-size: .9rem; opacity: .8; }
button { font: inherit; }
input { font: inherit; min-width: 0; }
"""
ICON = (
    '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 16 16">'
    '<rect width="16" height="16" fill="#263238"/>'
    '<path d="M3 4l5 8 5-8" stroke="white" stroke-width="2" fill="none"/>'
    '</svg>\n'
)


def command(name: str, fallback: Path) -> str:
    resolved = shutil.which(name) or (str(fallback) if fallback.exists() else None)
    if not resolved:
        raise RuntimeError(f"Missing {name}; see README.md")
    return resolved


def execute(argv: list[str]) -> None:
    print("+", " ".join(argv), flush=True)
    subprocess.run(argv, check=True, cwd=ROOT)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--release", action="store_true")
    args = parser.parse_args()
    profile = "release" if args.release else "debug"
    cargo = command("cargo", Path.home() / ".cargo/bin/cargo")
    bindgen = command(
        "wasm-bindgen", Path.home() / ".local/share/yetlet-tools/bin/wasm-bindgen"
    )
    packages = tomllib.loads((ROOT / "Cargo.lock").read_text(encoding="utf-8"))["package"]
    versions = {item["version"] for item in packages if item["name"] == "wasm-bindgen"}
    if len(versions) != 1:
        raise RuntimeError("Expected exactly one wasm-bindgen version in Cargo.lock")
    version = versions.pop()
    cli_version = subprocess.check_output([bindgen, "--version"], text=True).strip()
    if not re.search(rf"\b{re.escape(version)}\b", cli_version):
        raise RuntimeError(f"CLI {cli_version} does not match crate {version}")

    build = [
        cargo,
        "build",
        "--locked",
        "--target",
        WASM_TARGET,
        "-p",
        PACKAGE,
    ]
    render = [
        cargo,
        "run",
        "--quiet",
        "--locked",
        "-p",
        PACKAGE,
        "--bin",
        "render-ssr",
    ]
    if args.release:
        build.append("--release")
        render.append("--release")
    execute(build)
    ssr = subprocess.check_output(render, text=True, cwd=ROOT)
    source = ROOT / "target" / WASM_TARGET / profile / "yetlet_keyed_bindings.wasm"
    if not source.is_file():
        raise RuntimeError(f"Missing M7b Wasm build artifact: {source}")

    output = TARGET / profile
    output.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix=".stage-", dir=output) as folder:
        stage = Path(folder)
        execute(
            [
                bindgen,
                "--target",
                "web",
                "--out-dir",
                str(stage),
                "--out-name",
                "yetlet_keyed_bindings",
                str(source),
            ]
        )
        payloads = {
            "prototype.js": BOOTSTRAP.encode(),
            "styles.css": STYLES.encode(),
            "favicon.svg": ICON.encode(),
            "yetlet_keyed_bindings.js": (stage / "yetlet_keyed_bindings.js").read_bytes(),
            "yetlet_keyed_bindings_bg.wasm": (
                stage / "yetlet_keyed_bindings_bg.wasm"
            ).read_bytes(),
        }

    digest = hashlib.sha256(
        b"yetlet-m7b-keyed-bindings-v1\0"
        + b"".join(
            name.encode() + b"\0" + content + b"\0"
            for name, content in sorted(payloads.items())
        )
    ).hexdigest()[:16]
    asset_dir = output / "assets" / digest
    asset_dir.mkdir(parents=True, exist_ok=True)
    for name, content in payloads.items():
        (asset_dir / name).write_bytes(content)
    urls = {name: f"assets/{digest}/{name}" for name in payloads}

    page = f"""<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta http-equiv="Content-Security-Policy" content="default-src 'self'; script-src 'self' 'wasm-unsafe-eval'; style-src 'self'; img-src 'self'; connect-src 'self'; object-src 'none'; base-uri 'none'">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>Yetlet M7b/M7c/M7d Keyed Bindings, Reactivity and Lifecycle</title>
  <link rel="stylesheet" href="{urls['styles.css']}">
  <link rel="icon" href="{urls['favicon.svg']}" type="image/svg+xml">
  <script type="module" src="{urls['prototype.js']}"></script>
</head>
<body>
  <h1>Yetlet M7b/M7c/M7d Keyed Binding, Reactivity and Lifecycle Prototype</h1>
  <p>This fixture is isolated from production Browser and Search bundles.</p>
  <label for="outside-focus">Outside focus sentinel</label>
  <input id="outside-focus" type="text" value="outside">
  <div id="prototype-host">{ssr}</div>
</body>
</html>
"""
    manifest = {
        "schema_version": 1,
        "scope": "isolated_m7b_m7c_m7d_keyed_reactivity_lifecycle_prototype",
        "profile": profile,
        "digest": digest,
        "page": "index.html",
        "entry": urls["prototype.js"],
        "module": urls["yetlet_keyed_bindings.js"],
        "wasm": urls["yetlet_keyed_bindings_bg.wasm"],
        "stylesheet": urls["styles.css"],
        "favicon": urls["favicon.svg"],
    }
    (output / "index.html").write_text(page, encoding="utf-8")
    (output / "manifest.json").write_text(
        json.dumps(manifest, indent=2, sort_keys=True) + "\n", encoding="utf-8"
    )
    print(
        "PASS: isolated M7b/M7c/M7d keyed-reactivity-lifecycle build",
        profile,
        digest,
        len(payloads["yetlet_keyed_bindings_bg.wasm"]),
        "Wasm bytes",
    )
    return 0


if __name__ == "__main__":
    try:
        sys.exit(main())
    except (OSError, RuntimeError, subprocess.CalledProcessError) as error:
        print(f"M7b/M7c/M7d build FAILED: {error}", file=sys.stderr)
        sys.exit(1)
