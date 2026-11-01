# Local Yetlet documentation pilot

Application-owned, unpublished YT-224 consumer of registry SDK0.9.0-alpha.5.
It renders the eight existing selections/excerpts in site/pages.toml with Yetlet
HTML, plus separately selected browser search. It does not replace the current
mdBook site, change Pages, or add an SDK/CLI API. Published CLI0.1.1 does not have
build --static; use the reviewed source CLI from this checkout.

## Prerequisites

Use Rust/Cargo1.99 and the existing native CLI/link/OpenSSL prerequisites in the
[CLI guide](../../tools/yetlet-cli/README.md#first-use-prerequisites). Prepare the
source CLI and dependencies explicitly; export itself uses frozen/offline Cargo.
Python3.11+ stages canonical excerpts. HTML-only export needs no Node, Wasm,
browser or server dependency. The sample's publish=false workspace and lockfile
are independent; neither root lockfile nor package versions change.

The maintained [pulldown-cmark parser](https://docs.rs/pulldown-cmark/0.13.6/pulldown_cmark/)
is native-build-only. Its events become escaped Yetlet Views, never raw HTML.
CommonMark text/headings/lists/code/links, plain tables and strikethrough are
mapped explicitly. Raw HTML, images, aligned tables and unsupported events fail
export. Code blocks stay escaped plain code; no syntax highlighter or HTML sink
is installed. This is a selected repository reader, not a general Markdown service.

## Minimal integration

From the repository root, stage the selected inputs once:

```bash
python3 examples/docs-site/prepare.py
cargo fetch --locked --manifest-path examples/docs-site/Cargo.toml
```

prepare.py reuses the existing site selection/excerpt/link projection; it writes
ignored src/generated/chapters.json with source/excerpt hashes. Do not edit that
generated copy. External repository links use the recorded content commit; an
unpublished local commit can be unavailable remotely. The page labels this
local-preview limitation; it is not a public edition selection.

Use a disposable private consumer so group-writable checkout ancestors do not
weaken CLI protection. The following demonstration variables stay in a subshell.
Only the generated public directory is hosted:

```bash
(
    umask 077
    docs_pilot_repo="$PWD"
    docs_pilot_root="$(mktemp -d)"
    mkdir "$docs_pilot_root/app" "$docs_pilot_root/public"
    cp examples/docs-site/Cargo.toml examples/docs-site/Cargo.lock "$docs_pilot_root/app/"
    cp -R examples/docs-site/src examples/docs-site/frontend examples/docs-site/public "$docs_pilot_root/app/"
    chmod -R go-w "$docs_pilot_root/app"
    "$docs_pilot_repo/tools/yetlet-cli/target/debug/yetlet" build \
        --manifest-path "$docs_pilot_root/app/Cargo.toml" --static \
        --export-bin docs-export --output "$docs_pilot_root/public/yetlet" \
        --base-path /yetlet/
    echo "Disposable pilot: $docs_pilot_root"
    python3 -m http.server 8766 --bind 127.0.0.1 --directory "$docs_pilot_root/public"
)
```

Open http://127.0.0.1:8766/yetlet/. Stop with Ctrl-C; remove only the printed
disposable root when finished. Do not serve the repository, application inputs,
private receipts or the Cargo target directory. Select an unused port first.

## Expected result

Eight readable HTML chapters, local CSS and .nojekyll: ten public files. Direct
nested chapter navigation, headings/anchors, code, tables and the native mobile
menu work without JavaScript. Desktop navigation has eight entries; keyboard
users can skip to the main content. The header Search link leads to the optional
search panel. This initial pilot fixes /yetlet/ and does not add arbitrary routes,
remote content ingestion, SPA rewriting or a web application server.

## Configuration and errors

For real opt-in search, prepare the existing wasm32 target, Node and matching
wasm-bindgen0.2.129 explicitly, then build assets before copying the consumer:

```bash
(
    # WASM_BINDGEN may select an already installed matching executable.
    node examples/docs-site/frontend/build-wasm.mjs
)
```

Append --wasm-islands to the shown static build. Use a new disposable root/output;
the CLI never replaces an existing artifact. This mode exports fifteen files.
Initial content is complete HTML; only the small external bootstrap loads
automatically. Enable search explicitly downloads the prepared local Wasm/index,
mounts its own input/results host and searches eight chapter texts. No request
API, general hydration, remote search or mandatory search service is introduced.
Failure preserves the article/navigation and offers explicit reload; no automatic
retry. pagehide retires the listener except for a persisted bfcache page. No new
bfcache qualification is claimed.

Existing CLI settings/native target validation, CSP and protected filesystem/
process boundaries are reused. There are no new settings keys. Refused ownership,
raw/unsupported Markdown, missing preparation or invalid output stops export;
private target/yetlet-static receipts and inputs never enter public output.

## API reference and evidence

Use [HTML](../../crates/yetlet/guides/html.md), [DOM](../../crates/yetlet-browser/guides/dom.md),
the [export contract](../../docs/YT-222-STATIC-HTML-CONTRACT.md) and
[pilot evidence/limits](../../docs/YT-224-DOCS-PILOT.md). SDK/CLI versions, provider
defaults and renderer APIs are unchanged. New evidence is Linux/Chromium with
desktop and emulated touch; it is not Windows, Firefox, physical-mobile,
general Markdown conformance or production deployment qualification.
