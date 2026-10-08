#!/usr/bin/env python3
"""Build versioned Yetlet browser Wasm/JS assets without a Node bundler."""
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
DIST = ROOT / "dist"
JS = 'import init, { mount } from "./yetlet_browser.js";\nawait init();\nmount();\n'
CSS = """
:root { color-scheme: light dark; font-family: system-ui, sans-serif; }
body { max-width: 48rem; margin: 0 auto; padding: 1.5rem; line-height: 1.6; }
.is-complete { text-decoration: line-through; opacity: .65; }
button { font: inherit; padding: .5rem .9rem; cursor: pointer; }
button:focus-visible { outline: 3px solid Highlight; outline-offset: 2px; }
"""


# Avoid Chrome's default missing /favicon.ico request during E2E testing.
ICON = (
    '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 16 16">'
    '<rect width="16" height="16" fill="#263238"/>'
    '<path d="M3 4l5 8 5-8" stroke="white" stroke-width="2" fill="none"/>'
    '</svg>\n'
)

def command(name, fallback):
    tool = shutil.which(name) or (str(fallback) if fallback.exists() else None)
    if not tool:
        raise RuntimeError(f"Missing {name}; see README.md")
    return tool


def execute(*argv):
    print("+", " ".join(map(str, argv)), flush=True)
    subprocess.run(list(map(str, argv)), check=True, cwd=ROOT)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--release", action="store_true")
    options = parser.parse_args()
    cargo = command("cargo", Path.home() / ".cargo/bin/cargo")
    bindgen = command(
        "wasm-bindgen",
        Path.home() / ".local/share/yetlet-tools/bin/wasm-bindgen",
    )
    packages = tomllib.loads((ROOT / "Cargo.lock").read_text())["package"]
    versions = {p["version"] for p in packages if p["name"] == "wasm-bindgen"}
    if len(versions) != 1:
        raise RuntimeError("Expected exactly one wasm-bindgen version in Cargo.lock")
    version = versions.pop()
    cli_version = subprocess.check_output([bindgen, "--version"], text=True).strip()
    if not re.search(rf"\b{re.escape(version)}\b", cli_version):
        raise RuntimeError(f"CLI {cli_version} does not match crate {version}")

    release = ["--release"] if options.release else []
    execute(
        cargo, "build", "--locked", "--target", "wasm32-unknown-unknown",
        "-p", "yetlet-todo-browser", *release
    )
    profile = "release" if options.release else "debug"
    source = ROOT / "target/wasm32-unknown-unknown" / profile / "yetlet_todo_browser.wasm"
    if not source.is_file():
        raise RuntimeError(f"Missing {source}")
    DIST.mkdir(exist_ok=True)

    with tempfile.TemporaryDirectory(prefix=".stage-", dir=DIST) as temp:
        execute(
            bindgen, "--target", "web", "--out-dir", temp,
            "--out-name", "yetlet_browser", source
        )
        stage = Path(temp)
        files = {
            "app.js": JS.encode(),
            "styles.css": CSS.encode(),
            "favicon.svg": ICON.encode(),
            "yetlet_browser.js": (stage / "yetlet_browser.js").read_bytes(),
            "yetlet_browser_bg.wasm": (stage / "yetlet_browser_bg.wasm").read_bytes(),
        }
        digest = hashlib.sha256(b"yetlet-v1\0" + b"".join(
            k.encode() + b"\0" + v + b"\0" for k, v in sorted(files.items())
        )).hexdigest()[:16]
        asset_dir = DIST / "assets" / digest
        asset_dir.mkdir(parents=True, exist_ok=True)
        for name, content in files.items():
            (asset_dir / name).write_bytes(content)

    urls = {name: f"/assets/{digest}/{name}" for name in files}
    page = f"""<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>Yetlet | Browser Todo</title>
<link rel="stylesheet" href="{urls['styles.css']}">
<link rel="icon" href="{urls['favicon.svg']}" type="image/svg+xml">
<script type="module" src="{urls['app.js']}"></script>
</head>
<body>
<div id="app"><p>Loading browser runtime...</p></div>
<noscript>JavaScript is required. <a href="/">Open the SSR page</a>.</noscript>
</body>
</html>
"""
    manifest = {
        "schema_version": 1,
        "digest": digest,
        "mode": profile,
        "entry": urls["app.js"],
        "module": urls["yetlet_browser.js"],
        "wasm": urls["yetlet_browser_bg.wasm"],
        "stylesheet": urls["styles.css"],
        "favicon": urls["favicon.svg"],
    }
    (DIST / "app.html").write_text(page, encoding="utf-8")
    (DIST / "manifest.json").write_text(
        json.dumps(manifest, indent=2) + "\n", encoding="utf-8"
    )
    print("PASS browser assets:", digest, "Wasm bytes:", len(files["yetlet_browser_bg.wasm"]))


if __name__ == "__main__":
    try:
        main()
    except (OSError, RuntimeError, subprocess.CalledProcessError) as error:
        print("Browser build failed:", error, file=sys.stderr)
        sys.exit(1)
