#!/usr/bin/env python3
"""Build a local mdBook from explicitly selected canonical repository pages."""

import argparse
import hashlib
import json
import re
import shutil
import subprocess
import tomllib
from pathlib import Path
from urllib.parse import quote, unquote, urlsplit

ROOT = Path(__file__).resolve().parents[1]
OUTPUT = ROOT / "target/docs-site"
REPOSITORY = "https://github.com/bohewu/yetlet"
LINK = re.compile(r"(!?\[[^\]\n]*\]\()([^\s)]+)(\))")


def rewrite_links(text, source, destinations, commit):
    """Use local chapters for selected inputs and pinned GitHub links otherwise."""
    def replace(match):
        target = match[2]
        parsed = urlsplit(target)
        if parsed.scheme or parsed.netloc or target.startswith("#"):
            return match[0]
        path = (source.parent / unquote(parsed.path)).resolve()
        relative = path.relative_to(ROOT)
        if not path.is_file():
            raise ValueError(f"Missing link from {source.relative_to(ROOT)}: {target}")
        if path in destinations:
            result = destinations[path]
        else:
            result = f"{REPOSITORY}/blob/{commit}/{quote(relative.as_posix())}"
        if parsed.fragment:
            result += "#" + parsed.fragment
        return match[1] + result + match[3]

    lines = []
    fence = None
    for line in text.splitlines(keepends=True):
        start = re.match(r"^\s*(`{3,}|~{3,})", line)
        if start:
            marker = start[1]
            if fence is None:
                fence = marker
            elif marker[0] == fence[0] and len(marker) >= len(fence):
                fence = None
            lines.append(line)
        else:
            lines.append(line if fence else LINK.sub(replace, line))
    return "".join(lines)


def build(mdbook):
    selection = tomllib.loads((ROOT / "site/pages.toml").read_text())
    version = subprocess.check_output([mdbook, "--version"], text=True).strip()
    if version != f"mdbook v{selection['mdbook-version']}":
        raise ValueError(f"Expected mdBook {selection['mdbook-version']}; got {version}")
    commit = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip()
    dirty = bool(subprocess.check_output(["git", "status", "--porcelain"], cwd=ROOT, text=True))
    sdk = tomllib.loads((ROOT / "Cargo.toml").read_text())["workspace"]["package"]["version"]
    cli = tomllib.loads((ROOT / "tools/yetlet-cli/Cargo.toml").read_text())["package"]["version"]
    pages = selection["pages"]
    destinations = {(ROOT / p["source"]).resolve(): p["output"] for p in pages}
    if len(destinations) != len(pages) or len({p["output"] for p in pages}) != len(pages):
        raise ValueError("Duplicate source or output in page selection")
    for page in pages:
        if Path(page["output"]).name != page["output"] or not page["output"].endswith(".md"):
            raise ValueError("Chapter outputs must be plain Markdown filenames")
    # Generated files live only under this dedicated ignored target directory.
    staging = OUTPUT / "staging"
    if staging.exists():
        shutil.rmtree(staging)
    (staging / "src").mkdir(parents=True)
    shutil.copyfile(ROOT / "site/book.toml", staging / "book.toml")
    manifest = {"mdbook": version, "source_base": commit, "local_changes": dirty,
                "sdk_version": sdk, "cli_version": cli, "pages": []}
    summary = ["# Summary\n\n"]
    for page in pages:
        source = ROOT / page["source"]
        original = source.read_text()
        text = original
        if marker := page.get("until"):
            if marker not in text:
                raise ValueError(f"Missing excerpt marker in {page['source']}")
            text = text.split(marker, 1)[0]
        text = rewrite_links(text, source, destinations, commit)
        label = (f"> Local documentation preview · SDK `{sdk}` · CLI `{cli}`. "
                 f"Source base `{commit[:7]}`" + (" with local edits" if dirty else "") +
                 ". External repository links show that committed base.\n\n")
        (staging / "src" / page["output"]).write_text(label + text)
        summary.append(f"- [{page['title']}]({page['output']})\n")
        manifest["pages"].append({**page,
            "source_sha256": hashlib.sha256(original.encode()).hexdigest(),
            "projected_sha256": hashlib.sha256((label + text).encode()).hexdigest()})
    (staging / "src/SUMMARY.md").write_text("".join(summary))
    subprocess.run([mdbook, "build", str(staging)], check=True)
    destination = OUTPUT / "yetlet"
    if destination.exists():
        shutil.rmtree(destination)
    shutil.copytree(staging / "book", destination)
    (OUTPUT / "EXPORT-MANIFEST.json").write_text(json.dumps(manifest, indent=2) + "\n")
    print(f"Built {len(pages)} explicit chapters: {destination.relative_to(ROOT)}/index.html")
    print("No Pages deployment, workflow dispatch or recursive source export.")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--mdbook", default="mdbook", help="Path to the pinned mdBook executable")
    args = parser.parse_args()
    build(args.mdbook)
