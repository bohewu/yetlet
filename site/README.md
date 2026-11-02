# Guide sources and legacy mdBook preview

This checkout's [YT-226 integration](../docs/YT-226-PAGES-CUTOVER-PLAN.md#owner-selected-minimal-workflow-integration--2026-11-01)
uses one .github/workflows/docs.yml to build the Yetlet reader from the exact
selection in pages.toml. Follow the [Yetlet site build guide](../examples/docs-site/README.md#formal-artifact-preparation-no-deployment)
for current Rust/Node/Wasm prerequisites and artifact commands. Build records stay
outside the upload root. The owner-authorized
[public cutover](../docs/YT-226-PAGES-CUTOVER-PLAN.md#authorized-public-cutover--2026-11-01)
now serves this Yetlet reader at https://bohewu.github.io/yetlet/, source0ae6b93.
PAGES_DEPLOY_ENABLED is true; relevant protected-master updates publish through
the single workflow. Local build commands below do not enable or deploy Pages.

The mdBook command below remains an explicit reference/fallback preview and
does not run in the new Pages build. Original YT-217/218 evidence is retained.

YT-217 uses existing English Markdown as the sole guide source and Rust doc
comments as the API source. This directory contains only build configuration
and the explicit chapter selection; it contains no second copy of the guides.
The original Pages workflow built this mdBook preview before the YT-226 change.
Publication requires the owner-adopted [Pages settings and policy](../docs/GITHUB-PAGES-PUBLICATION.md).

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

The legacy builder also supports `--public` with the same command. This only builds
files: it never enables Pages or deploys. It requires a clean checkout and the
explicit SDK/CLI edition in `pages.toml` to match the reviewed published edition,
independently of future workspace versions. The selected source edition is
SDKalpha.6/CLI0.2.0; this local update does not replace the live reader. Its
public banner links the full source commit; local/PR builds retain preview labels.
Both modes check the rendered upload root, selected chapters and local links.

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

This reference preview's navigation, search and assets use mdBook's built-in HTML renderer with project
base `/yetlet/`; Rust playground execution is disabled. Initial Pages enablement
and public publication follow the separately owner-authorized sequence under the
[publication scope](../docs/DOCUMENTATION-SCOPE.md) and
[public documentation plan](../docs/PUBLIC-DOCS-PLAN.md).
