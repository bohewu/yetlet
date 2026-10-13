#!/usr/bin/env python3
"""Reproduce only this proof's pinned, same-origin assets; no compiler or package install."""
from __future__ import annotations
import argparse
import base64
import hashlib
import json
from pathlib import Path
import re
import urllib.parse
import urllib.request
import xml.etree.ElementTree as ET

ROOT = Path(__file__).resolve().parent
UPSTREAM = "https://raw.githubusercontent.com/twbs/bootstrap/v5.3.8/"
CSS_SRI = "sRIl4kxILFvY47J16cr9ZwB07vP4J8+LH7qKQnuqkuIAvNWLzeN8tE5YBujZqJLB"
PREFIX = "/board-assets/"

def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()

def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--fetch", action="store_true", help="Acquire only the pinned CSS and MIT license")
    parser.add_argument("--check", action="store_true", help="Compare generated files without writes")
    args = parser.parse_args()
    if args.fetch and args.check:
        parser.error("fetch and check are separate operations")
    source = ROOT / "assets/upstream"
    inputs = {"bootstrap.min.css": "dist/css/bootstrap.min.css", "LICENSE.bootstrap": "LICENSE"}
    if args.fetch:
        source.mkdir(parents=True, exist_ok=True)
        for name, relative in inputs.items():
            path = source / name
            if path.exists():
                continue
            with urllib.request.urlopen(UPSTREAM + relative, timeout=30) as response:
                data = response.read(1024 * 1024 + 1)
            if len(data) > 1024 * 1024:
                raise ValueError("upstream asset exceeds bound")
            if name.endswith(".css") and base64.b64encode(hashlib.sha384(data).digest()).decode() != CSS_SRI:
                raise ValueError("upstream CSS does not match the official version's SHA-384")
            if name.startswith("LICENSE") and b"Permission is hereby granted" not in data:
                raise ValueError("unexpected license content")
            path.write_bytes(data)
    originals = {name: (source / name).read_bytes() for name in inputs}
    css_bytes = originals["bootstrap.min.css"]
    if base64.b64encode(hashlib.sha384(css_bytes).digest()).decode() != CSS_SRI:
        raise ValueError("CSS source digest mismatch")
    css = css_bytes.decode("utf-8")
    outputs: dict[str, tuple[bytes, str]] = {}
    svg_count = 0
    def replace_svg(match: re.Match[str]) -> str:
        nonlocal svg_count
        data = urllib.parse.unquote_to_bytes(match.group(1))
        tree = ET.fromstring(data)
        allowed = {"svg", "g", "path", "polyline", "polygon", "line", "circle", "ellipse", "rect"}
        for node in tree.iter():
            if node.tag.rsplit("}", 1)[-1] not in allowed:
                raise ValueError("unexpected trusted-distribution SVG element")
            for key, value in node.attrib.items():
                if key.lower().startswith("on") or key.rsplit("}", 1)[-1] in {"href", "style"} or "url(" in value:
                    raise ValueError("unexpected SVG active/external reference")
        name = f"icon-{sha(data)}.svg"
        outputs[name] = (data, "image/svg+xml")
        svg_count += 1
        return f'url("{PREFIX}{name}")'
    css = re.sub(r'url\("data:image/svg\+xml,([^\"]*)"\)', replace_svg, css)
    css = re.sub(r'/\*# sourceMappingURL=.*?\*/', '', css)
    if "data:" in css or "@import" in css:
        raise ValueError("unconverted embedded/external CSS source")
    for url in re.findall(r'url\(([^)]+)\)', css):
        value = url.strip('"\'')
        if not value.startswith(PREFIX) or value[len(PREFIX):] not in outputs:
            raise ValueError("CSS references an unregistered asset")
    css_data = css.encode()
    css_name = f"bootstrap-{sha(css_data)}.css"
    outputs[css_name] = (css_data, "text/css; charset=utf-8")
    local_names = {}
    for name, mime in [("app.css", "text/css; charset=utf-8"), ("enhance.js", "text/javascript; charset=utf-8")]:
        data = (ROOT / "assets" / name).read_bytes()
        stem, suffix = name.rsplit('.', 1)
        hashed = f"{stem}-{sha(data)}.{suffix}"
        outputs[hashed] = (data, mime)
        local_names[name] = hashed
    manifest = {"schema_version": 1, "stylesheets": [PREFIX + css_name, PREFIX + local_names["app.css"]], "enhancement": PREFIX + local_names["enhance.js"]}
    inventory = {"bootstrap_version": "5.3.8", "license": "MIT", "source_css_sri": "sha384-" + CSS_SRI, "source": [{"file": "upstream/" + name, "url": UPSTREAM + relative, "sha256": sha(originals[name])} for name, relative in inputs.items()], "externalized_svg_occurrences": svg_count, "outputs": [{"file": "generated/" + name, "sha256": sha(data), "bytes": len(data), "content_type": mime} for name, (data, mime) in sorted(outputs.items())]}
    expected = {ROOT / "assets/generated" / name: data for name, (data, _) in outputs.items()}
    expected[ROOT / "assets/manifest.json"] = (json.dumps(manifest, indent=2) + "\n").encode()
    expected[ROOT / "assets/inventory.json"] = (json.dumps(inventory, indent=2) + "\n").encode()
    rust = '// Generated by build_assets.py; never edit by hand.\n'
    rust += '#[rustfmt::skip]\npub struct EmbeddedAsset { pub name: &\'static str, pub content_type: &\'static str, pub bytes: &\'static [u8] }\n'
    rust += '#[rustfmt::skip]\npub static ASSETS: &[EmbeddedAsset] = &[\n'
    for name, (_, mime) in sorted(outputs.items()):
        rust += f'    EmbeddedAsset {{ name: "{name}", content_type: "{mime}", bytes: include_bytes!("../assets/generated/{name}") }},\n'
    rust += '];\n'
    expected[ROOT / "src/asset_data.rs"] = rust.encode()
    if args.check:
        mismatches = [str(p.relative_to(ROOT)) for p, data in expected.items() if not p.is_file() or p.read_bytes() != data]
        if mismatches:
            raise ValueError("generated artifact mismatch: " + ", ".join(mismatches))
        actual = set((ROOT / "assets/generated").iterdir())
        if actual != {p for p in expected if p.parent.name == "generated"}:
            raise ValueError("unexpected generated asset")
    else:
        generated = ROOT / "assets/generated"
        generated.mkdir(parents=True, exist_ok=True)
        for p in generated.iterdir():
            if p not in expected:
                raise ValueError("stale asset: remove deliberately after inspecting " + p.name)
        for p, data in expected.items():
            p.parent.mkdir(parents=True, exist_ok=True)
            p.write_bytes(data)
    print(f"PASS pinned CSS, license record, {svg_count} SVG occurrences; {len(outputs)} hashed outputs; mode={'check' if args.check else 'build'}")

if __name__ == "__main__":
    main()
