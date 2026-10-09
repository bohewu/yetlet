#!/usr/bin/env python3
"""Build the isolated M7a render benchmark without touching production dist/."""
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
TARGET = ROOT / "target" / "m7a-render-bench"
PACKAGE = "yetlet-render-bench"
WASM_TARGET = "wasm32-unknown-unknown"

BOOTSTRAP = """\
import init, { run_benchmark } from "./yetlet_render_bench.js";
await init();
window.runYetletBenchmark = (rows, updates, rounds, seed) =>
  JSON.parse(run_benchmark(rows, updates, rounds, seed));
document.documentElement.dataset.benchmarkReady = "true";
document.getElementById("run-default").addEventListener("click", () => {
  const output = document.getElementById("output");
  output.textContent = "Running...";
  setTimeout(() => {
    try {
      output.textContent = JSON.stringify(
        window.runYetletBenchmark(200, 500, 10, 7), null, 2
      );
    } catch (error) {
      output.textContent = String(error);
    }
  }, 0);
});
"""

STYLES = """\
:root { color-scheme: light dark; font-family: system-ui, sans-serif; }
body { max-width: 68rem; margin: 0 auto; padding: 1.5rem; line-height: 1.5; }
button { font: inherit; padding: .5rem .9rem; }
#bench-root { position: absolute; left: -100000px; top: 0; }
pre { max-height: 36rem; overflow: auto; padding: 1rem; border: 1px solid; }
"""


def tool(name: str, fallback: Path) -> str:
    found = shutil.which(name)
    if found:
        return found
    if fallback.is_file():
        return str(fallback)
    raise RuntimeError(f"Missing {name}; see README.md")


def execute(arguments: list[str]) -> None:
    print("+", " ".join(arguments), flush=True)
    subprocess.run(arguments, check=True, cwd=ROOT)


def wasm_bindgen_version() -> str:
    packages = tomllib.loads((ROOT / "Cargo.lock").read_text(encoding="utf-8"))["package"]
    versions = {item["version"] for item in packages if item["name"] == "wasm-bindgen"}
    if len(versions) != 1:
        raise RuntimeError(f"Expected one wasm-bindgen version, got {versions}")
    return versions.pop()


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--release", action="store_true")
    args = parser.parse_args()

    cargo = tool("cargo", Path.home() / ".cargo/bin/cargo")
    bindgen = tool(
        "wasm-bindgen", Path.home() / ".local/share/yetlet-tools/bin/wasm-bindgen"
    )
    crate_version = wasm_bindgen_version()
    cli_version = subprocess.check_output([bindgen, "--version"], text=True).strip()
    if not re.search(rf"\b{re.escape(crate_version)}\b", cli_version):
        raise RuntimeError(
            f"wasm-bindgen CLI ({cli_version}) must match crate ({crate_version})"
        )

    profile = "release" if args.release else "debug"
    command = [
        cargo,
        "build",
        "--locked",
        "--target",
        WASM_TARGET,
        "-p",
        PACKAGE,
    ]
    if args.release:
        command.append("--release")
    execute(command)

    source = ROOT / "target" / WASM_TARGET / profile / "yetlet_render_bench.wasm"
    if not source.is_file():
        raise RuntimeError(f"Missing benchmark Wasm: {source}")

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
                "yetlet_render_bench",
                str(source),
            ]
        )
        payloads = {
            "bench.js": BOOTSTRAP.encode(),
            "styles.css": STYLES.encode(),
            "yetlet_render_bench.js": (stage / "yetlet_render_bench.js").read_bytes(),
            "yetlet_render_bench_bg.wasm": (
                stage / "yetlet_render_bench_bg.wasm"
            ).read_bytes(),
        }

    digest = hashlib.sha256(
        b"yetlet-m7a-render-bench-v1\0"
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
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>Yetlet M7a Render Benchmark</title>
  <link rel="icon" href="data:,">
  <link rel="stylesheet" href="{urls['styles.css']}">
  <script type="module" src="{urls['bench.js']}"></script>
</head>
<body>
  <h1>Yetlet M7a Render Benchmark</h1>
  <p>This isolated fixture is not a production route or public renderer API.</p>
  <button type="button" id="run-default">Run default benchmark</button>
  <pre id="output">Ready after the benchmark module loads.</pre>
  <div id="bench-root" aria-hidden="true"></div>
</body>
</html>
"""
    manifest = {
        "schema_version": 1,
        "scope": "isolated_m7a_render_benchmark",
        "profile": profile,
        "digest": digest,
        "entry": urls["bench.js"],
        "module": urls["yetlet_render_bench.js"],
        "wasm": urls["yetlet_render_bench_bg.wasm"],
        "stylesheet": urls["styles.css"],
    }
    (output / "index.html").write_text(page, encoding="utf-8")
    (output / "manifest.json").write_text(
        json.dumps(manifest, indent=2, sort_keys=True) + "\n", encoding="utf-8"
    )
    print(
        "PASS: isolated render benchmark",
        profile,
        digest,
        len(payloads["yetlet_render_bench_bg.wasm"]),
        "Wasm bytes",
    )
    return 0


if __name__ == "__main__":
    try:
        sys.exit(main())
    except (OSError, RuntimeError, subprocess.CalledProcessError) as error:
        print(f"Render benchmark build FAILED: {error}", file=sys.stderr)
        sys.exit(1)
