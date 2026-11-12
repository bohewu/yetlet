# Yetlet documentation site

Application-owned, nonpublishable YT-224 package using registry SDK0.9.0-alpha.8,
with [YT-225 public-mode artifact preparation](../../docs/YT-225-DOCS-PUBLIC-BUILD.md).
It renders the eight existing selections/excerpts in site/pages.toml with Yetlet
HTML, plus separately selected browser search. The later owner-authorized
[YT-226 cutover](../../docs/YT-226-PAGES-CUTOVER-PLAN.md#authorized-public-cutover--2026-11-01)
publishes this reader at https://bohewu.github.io/yetlet/ without adding an SDK/CLI
API. Local export commands do not change Pages. Static export is available in
published CLI `0.2.0` and remains available in later releases; the reproducible
source-bound build below uses the reviewed CLI from this checkout. The reader
source edition/consumer is explicitly pinned to SDKalpha.8/CLI0.4.0 independently
of future builder versions. [YT-249](../../docs/YT-249-READER-RELEASE-ROLLOVER.md)
records this promotion and the mandatory update after each verified publication;
[YT-232](../../docs/YT-232-PUBLIC-GUIDE-EDITION.md) retains the prior edition's proof.

## Prerequisites

Use Rust/Cargo1.99 and the existing native CLI/link/OpenSSL prerequisites in the
[CLI guide](../../docs/CLI-GUIDE.md#first-use-prerequisites). Prepare the
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

The masthead reads two independent version pairs. `Published` reads the accepted
publication records through `scripts/release_version_authority.py`, shared with
the root README's published status. `Reader Edition` reads `site/pages.toml` and
keeps the guide's exact consumer pins and content source visible. A new published
release updates the first label on the next site build without promoting the
reader edition or republishing a CLI archive. The private export manifest records
both version pairs. Matching pairs share one compact version line with the source
link. If either SDK or CLI differs, separate `Published` and `Reader Edition`
lines identify that difference. This presentation does not change either authority.

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
users can skip to the main content. The selected island build adds a native search
field to the header; HTML-only output omits search. This initial pilot fixes
/yetlet/ and does not add arbitrary routes,
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
Initial content is complete HTML. In the selected island build, search starts
automatically when the page loads; there is no Enable search button. Loading the
prepared local Wasm/index updates only the owned results host. The header input
is usable before Wasm is ready, and initialization preserves its value, native
node and focus. Results appear below the field as chapter links with excerpts.
Tab/Enter follow native links; Escape, leaving the search region or clearing the
query closes the dropdown. On narrow screens the field occupies its own header
row. The HTML-only build still has no Wasm. No request API,
general hydration, remote search or mandatory search service is introduced.
Failure preserves the article/navigation and offers explicit reload; no automatic
retry. pagehide retires the listener except for a persisted bfcache page. No new
bfcache qualification is claimed.

Existing CLI settings/native target validation, CSP and protected filesystem/
process boundaries are reused. There are no new settings keys. Refused ownership,
raw/unsupported Markdown, missing preparation or invalid output stops export;
private target/yetlet-static receipts and inputs never enter public output.

## API reference and evidence

### Formal artifact preparation (no deployment)

After separately preparing the matching tools, locked dependencies and source
CLI described above, run from the repository root with a fresh protected output
parent. The example variables stay within a subshell:

```bash
(
    umask 077
    guide_artifact_root="$(mktemp -d)"
    python3 tools/build_yetlet_docs.py --public \
        --cli tools/yetlet-cli/target/debug/yetlet \
        --output "$guide_artifact_root/yetlet"
    echo "Guide artifact and private build record: $guide_artifact_root"
)
```

--public requires a clean checkout and the reviewed SDK/CLI edition. It binds
the source link to the full commit and omits local-preview footer copy. Omit the
flag for a local build with edits. The command builds Wasm and exports all15
selected site files; it neither installs tools nor deploys. Use only the yetlet
subdirectory as a future upload root. EXPORT-MANIFEST.json and STATIC-RECEIPTS
stay outside it. Output/record destinations are create-only. Remove the printed
temporary root only when its artifact/evidence is no longer needed.

This checkout's [single documentation workflow](../../.github/workflows/docs.yml)
builds the formal artifact through this same command; the duplicate docs-preview
workflow is retired. The build has read-only repository access. A separate
master-only deployment job remains gated by PAGES_DEPLOY_ENABLED and the build's
deploy output. That switch was held false during integration and is now true
under the separately authorized YT-226 publication, source0ae6b93/run41289520993/1.
The owner
chooses the directory URLs without old mdBook .html aliases; the early site's
old bookmarks are not preserved by the cutover.

### API and accepted evidence

Use [HTML](../../crates/yetlet/guides/html.md), [DOM](../../crates/yetlet-browser/guides/dom.md),
the [export contract](../../docs/YT-222-STATIC-HTML-CONTRACT.md) and
[pilot evidence/limits](../../docs/YT-224-DOCS-PILOT.md). SDK/CLI versions, provider
defaults and renderer APIs are unchanged. The initial local evidence is
Linux/Chromium with desktop and emulated touch. The cutover adds exact public
HTTPS bytes/MIME, directory navigation, real search and a narrow Firefox
reader observation; temporary-check failures remain explicit. This does not
extend the SDK platform matrix or claim physical-mobile, general Markdown
conformance or application production-deployment qualification.
