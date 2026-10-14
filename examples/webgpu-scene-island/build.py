#!/usr/bin/env python3
"""Build the Experimental scene consumer and independent counter; no browser execution."""
import hashlib
import json
from pathlib import Path
import subprocess
import sys
import tempfile

HERE = Path(__file__).resolve().parent
ROOT = HERE.parent.parent
sys.path.insert(0, str(ROOT / "scripts"))
from build_browser import command

def main():
    cargo = command("cargo", Path.home() / ".cargo/bin/cargo")
    bindgen = command("wasm-bindgen", Path.home() / ".local/share/yetlet-tools/bin/wasm-bindgen")
    if subprocess.check_output([bindgen, "--version"], text=True).strip() != "wasm-bindgen 0.2.129":
        raise RuntimeError("YT-075 requires the matching cached wasm-bindgen 0.2.129 CLI")
    dist = ROOT / "target/webgpu-scene-island-dist"
    manifest = {"schema_version": 1}
    for key, module, package, source in [
        ("scene", "scene", "yetlet-webgpu-scene-island", HERE / "target/wasm32-unknown-unknown/release/yetlet_webgpu_scene_island.wasm"),
        ("counter", "yetlet_dashboard_counter", "yetlet-dashboard-counter-island", ROOT / "target/wasm32-unknown-unknown/release/yetlet_dashboard_counter_island.wasm"),
    ]:
        args = [cargo, "build", "--offline", "--locked", "--release", "--target", "wasm32-unknown-unknown", "--lib", "-p", package]
        if key == "scene": args += ["--manifest-path", str(HERE / "Cargo.toml")]
        subprocess.run(args, cwd=ROOT, check=True)
        with tempfile.TemporaryDirectory(prefix="yt075-bindgen-") as tmp:
            subprocess.run([bindgen, "--target", "web", "--out-dir", tmp, "--out-name", module, str(source)], check=True)
            files = {f"{key}-loader.js": (HERE / f"{key}-loader.js").read_bytes()}
            if key == "scene": files["comparison-loader.js"] = (HERE / "comparison-loader.js").read_bytes()
            for name in (f"{module}.js", f"{module}_bg.wasm"):
                files[name] = (Path(tmp) / name).read_bytes()
            digest = hashlib.sha256(b"yt075-v1\0" + b"".join(name.encode() + b"\0" + data + b"\0" for name, data in sorted(files.items()))).hexdigest()[:16]
            directory = dist / "assets" / digest
            directory.mkdir(parents=True, exist_ok=True)
            for name, data in files.items(): (directory / name).write_bytes(data)
            manifest[key] = {"digest": digest, "loader": f"/scene-assets/{digest}/{key}-loader.js", "module": f"/scene-assets/{digest}/{module}.js", "wasm": f"/scene-assets/{digest}/{module}_bg.wasm"}
            print(f"{key}: {digest}; WASM {len(files[f'{module}_bg.wasm'])} bytes", flush=True)
    candidate = ROOT / "crates/yetlet-wgpu"
    sources = [HERE / "Cargo.toml", HERE / "Cargo.lock", *sorted((HERE / "src").glob("*.rs")),
               candidate / "Cargo.toml", candidate / "Cargo.lock", *sorted((candidate / "src").glob("*"))]
    manifest["candidate_sources"] = {str(path.relative_to(ROOT)): hashlib.sha256(path.read_bytes()).hexdigest()
                                     for path in sources if path.is_file()}
    (dist / "manifest.json").write_text(json.dumps(manifest, indent=2) + "\n")

if __name__ == "__main__": main()
