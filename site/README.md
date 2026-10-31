# Local reader documentation preview

YT-217 uses existing English Markdown as the sole guide source and Rust doc
comments as the API source. This directory contains only build configuration
and the explicit chapter selection; it contains no second copy of the guides.
No GitHub Pages workflow or deployment is installed.

## Prerequisites

Use Python 3.11 or newer, Git and mdBook **0.5.4**. Install mdBook separately
if needed; for example, from the repository root:

```sh
cargo install mdbook --version 0.5.4 --locked --root target/docs-tools
```

This is optional maintainer tooling, not a dependency of Yetlet applications.
The builder checks the exact mdBook version and never downloads tools itself.

## Build and inspect

```sh
python3 tools/build_docs.py --mdbook target/docs-tools/bin/mdbook
python3 -m http.server 8765 --bind 127.0.0.1 --directory target/docs-site
```

Open `http://127.0.0.1:8765/yetlet/`. Ctrl+C stops the loopback-only preview.
The generated site lives under ignored `target/docs-site/yetlet`; no public
hosting, keys or release services are used. Rebuilding replaces only this
builder's generated staging/site directories.

## Selection, versions and API docs

[`pages.toml`](pages.toml) selects eight exact chapter sources. The root README
ends before its project/maintainer footer; HTTP/DOM guide excerpts end before
their historical details. Links to selected chapters stay local. Other repository
links point to the source-base commit on GitHub; they do not copy linked files
or assets. Code blocks are preserved. Review full new inputs before changing
the selection; never point the book at the whole repository or `docs/` tree.
`target/docs-site/EXPORT-MANIFEST.json` records inputs, excerpts and hashes.

Each page labels SDK/CLI versions, the source base and local edits. Local edits
are visible in this preview but not in its external committed-source links.
The preview is not a signed product release or proof of docs.rs availability.
For locally generated native/WASM API documentation, follow the separate
[rustdoc commands](../docs/API-REFERENCE.md#local-rustdoc). Open their stated
entry pages; the guide does not substitute native rustdoc for browser-only APIs.

Navigation, search and assets use mdBook's built-in HTML renderer with project
base `/yetlet/`; Rust playground execution is disabled. Initial Pages enablement
and public publication remain separately authorized future work under the
[publication scope](../docs/DOCUMENTATION-SCOPE.md) and
[public documentation plan](../docs/PUBLIC-DOCS-PLAN.md).
