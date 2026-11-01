#!/usr/bin/env python3
"""Stage the existing eight canonical excerpts; no fetch, build or deployment."""
import hashlib
import importlib.util
import json
from pathlib import Path
import subprocess
import tomllib

APP = Path(__file__).resolve().parent
ROOT = APP.parents[1]


def prepare():
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
    prepared = {"commit": commit, "sdk": selection["edition"]["sdk"],
                "cli": selection["edition"]["cli"], "base": base, "chapters": chapters}
    if prepared["sdk"] != "0.9.0-alpha.5" or prepared["cli"] != "0.1.1":
        raise ValueError("Pilot edition changed; review the consumer first")
    destination = APP / "src/generated"
    destination.mkdir(exist_ok=True)
    (destination / "chapters.json").write_text(json.dumps(prepared, ensure_ascii=True, indent=2) + "\n")
    print("Prepared eight canonical excerpts; parser/rendering remains a native build step.")


if __name__ == "__main__":
    prepare()
