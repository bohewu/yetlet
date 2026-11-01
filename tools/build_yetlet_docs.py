#!/usr/bin/env python3
"""Build the selected Yetlet guide artifact; never deploy or download tools."""
import argparse
import hashlib
import importlib.util
import json
from pathlib import Path
import shutil
import subprocess
import tempfile

import build_docs

ROOT = Path(__file__).resolve().parents[1]
APP = ROOT / "examples/docs-site"


def validate_output(output, prepared):
    pages = [{"output": "index.md" if p["route"] == "/" else
              p["route"].strip("/") + "/index.md"} for p in prepared["chapters"]]
    expected = {p["output"].removesuffix(".md") + ".html" for p in pages}
    expected.update({".nojekyll", *("assets/" + name for name in (
        "guide.css", "search.mjs", "search-data.mjs", "wasm-startup.mjs",
        "docs_search.js", "docs_search_bg.wasm"))})
    actual = {p.relative_to(output).as_posix() for p in output.rglob("*") if p.is_file()}
    if actual != expected:
        raise ValueError("Guide artifact must contain exactly eight chapters and selected assets")
    checked = build_docs.validate_output(output, pages)
    if prepared["public"]:
        source = "https://github.com/bohewu/yetlet/tree/" + prepared["commit"]
        for p in pages:
            html = (output / (p["output"].removesuffix(".md") + ".html")).read_text()
            if source not in html or "Local, unpublished checkout" in html or "No live Pages replacement" in html:
                raise ValueError("Public guide source disclosure is missing or still a local preview")
    return checked


def build(cli, output, public=False):
    output = output.absolute()
    manifest = output.parent / "EXPORT-MANIFEST.json"
    if output.exists() or output.is_symlink() or manifest.exists():
        raise ValueError("Use a fresh output and manifest destination; never replace accepted artifacts")
    spec = importlib.util.spec_from_file_location("yetlet_guide_prepare", APP / "prepare.py")
    prepare = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(prepare)
    prepared = prepare.prepare(public)
    # Tool/target and locked dependency preparation are separate caller actions.
    subprocess.run(["node", str(APP / "frontend/build-wasm.mjs")], cwd=ROOT, check=True)
    output.parent.mkdir(parents=True, exist_ok=True, mode=0o700)
    with tempfile.TemporaryDirectory(prefix=".yetlet-guide-", dir=output.parent) as temp:
        consumer = Path(temp) / "app"
        consumer.mkdir(mode=0o700)
        for name in ("Cargo.toml", "Cargo.lock"):
            shutil.copyfile(APP / name, consumer / name)
        for name in ("src", "frontend", "public"):
            shutil.copytree(APP / name, consumer / name)
        for path in consumer.rglob("*"):
            path.chmod(path.stat().st_mode & ~0o022)
        inputs = {p.relative_to(consumer).as_posix(): hashlib.sha256(p.read_bytes()).hexdigest()
                  for p in consumer.rglob("*") if p.is_file()}
        try:
            subprocess.run([str(cli.resolve()), "build", "--static", "--export-bin", "docs-export",
                            "--output", str(output), "--base-path", "/yetlet/", "--wasm-islands"],
                           cwd=consumer, check=True)
        finally:
            receipts = consumer / "target/yetlet-static"
            if receipts.exists():
                shutil.copytree(receipts, output.parent / "STATIC-RECEIPTS")
    checked = validate_output(output, prepared)
    if public:
        commit = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip()
        dirty = subprocess.check_output(["git", "status", "--porcelain", "--untracked-files=all"],
                                        cwd=ROOT, text=True)
        if dirty or commit != prepared["commit"]:
            raise ValueError("Public source changed during the build; artifact is not accepted")
    record = {"mode": "public" if public else "preview", "source": prepared["commit"],
              "sdk": prepared["sdk"], "cli": prepared["cli"], "base": prepared["base"],
              "local_links_checked": checked, "consumer_inputs": inputs,
              "chapters": [{k: p[k] for k in ("source", "route", "source_sha256", "excerpt_sha256")}
                           for p in prepared["chapters"]],
              "outputs": {p.relative_to(output).as_posix(): {
                  "sha256": hashlib.sha256(p.read_bytes()).hexdigest(), "size": p.stat().st_size}
                  for p in sorted(output.rglob("*")) if p.is_file()}}
    manifest.write_text(json.dumps(record, indent=2) + "\n")
    print(f"Built {len(record['outputs'])} public files; private manifest/receipts stay outside upload root.")
    return record


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--cli", type=Path, required=True, help="Already-built source CLI with static mode")
    parser.add_argument("--output", type=Path, required=True, help="Fresh site root below an owned parent")
    parser.add_argument("--public", action="store_true", help="Clean edition-bound output, not deployment")
    args = parser.parse_args()
    build(args.cli, args.output, args.public)
