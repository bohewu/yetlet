#!/usr/bin/env python3
"""Stage the existing eight canonical excerpts; no fetch, build or deployment."""
import argparse
import hashlib
import importlib.util
import json
from pathlib import Path
import subprocess
import tomllib

APP = Path(__file__).resolve().parent
ROOT = APP.parents[1]


def prepare(public=False):
    selection = tomllib.loads((ROOT / "site/pages.toml").read_text())
    pages = selection["pages"]
    expected = ["index.md", "first-app.md", "views.md", "http.md", "browser.md",
                "features.md", "api.md", "troubleshooting.md"]
    if [p["output"] for p in pages] != expected:
        raise ValueError("Pilot requires the reviewed eight-page selection")
    spec = importlib.util.spec_from_file_location("existing_docs", ROOT / "tools/build_docs.py")
    existing = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(existing)
    commit = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip()
    dirty = bool(subprocess.check_output(
        ["git", "status", "--porcelain", "--untracked-files=all"], cwd=ROOT, text=True))
    sdk = tomllib.loads((ROOT / "Cargo.toml").read_text())["workspace"]["package"]["version"]
    cli = tomllib.loads((ROOT / "tools/yetlet-cli/Cargo.toml").read_text())["package"]["version"]
    # Preserve the published consumer edition while workspace versions advance.
    existing.page_label(commit, dirty, sdk, cli, selection.get("edition"), public)
    base = "/yetlet/"
    routes = {p["output"]: "/" if p["output"] == "index.md" else "/" + p["output"][:-3] + "/"
              for p in pages}
    destinations = {(ROOT / p["source"]).resolve(): base + routes[p["output"]].lstrip("/") for p in pages}
    chapters = []
    for page in pages:
        source = ROOT / page["source"]
        original = source.read_text()
        excerpt = original
        if marker := page.get("until"):
            if marker not in excerpt:
                raise ValueError(f"Missing excerpt marker: {page['source']}")
            excerpt = excerpt.split(marker, 1)[0]
        chapters.append({"source": page["source"], "title": page["title"],
                         "route": routes[page["output"]],
                         "source_sha256": hashlib.sha256(original.encode()).hexdigest(),
                         "excerpt_sha256": hashlib.sha256(excerpt.encode()).hexdigest(),
                         "markdown": existing.rewrite_links(excerpt, source, destinations, commit)})
    prepared = {"commit": commit, "public": public, "sdk": selection["edition"]["sdk"],
                "cli": selection["edition"]["cli"], "base": base, "chapters": chapters}
    consumer = tomllib.loads((APP / "Cargo.toml").read_text())
    dependencies = [consumer["dependencies"],
                    *(target.get("dependencies", {}) for target in consumer.get("target", {}).values())]
    requirements = {name: [table[name]["version"] for table in dependencies if name in table]
                    for name in ("yetlet", "yetlet-browser")}
    if any(not versions or any(version != "=" + prepared["sdk"] for version in versions)
           for versions in requirements.values()):
        raise ValueError("Guide consumer must pin both selected registry SDK packages exactly")
    destination = APP / "src/generated"
    destination.mkdir(exist_ok=True)
    (destination / "chapters.json").write_text(json.dumps(prepared, ensure_ascii=True, indent=2) + "\n")
    print("Prepared eight canonical excerpts; parser/rendering remains a native build step.")
    return prepared


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--public", action="store_true",
                        help="Require clean sources and the reviewed published edition; does not deploy")
    prepare(parser.parse_args().public)
