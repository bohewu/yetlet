# YT-222 — Static HTML export and CLI contract

Date: 2026-10-31. Status: Done at documentation-only contract scope; independent
Reviewer PASS0, zero issues/required followups. Implementation belongs to YT-223. No command, protocol or new
API described as selected below exists in the published CLI0.1.1 or SDK0.9.0-alpha.5.
Source HEAD:4b88f3fec2325170fa4b1109cb08d2a53518c55c, with accepted YT-221
source changes still uncommitted. This item preserves those changes and their
evidence; it does not authorize Git delivery, hosted execution or deployment.

## Purpose and existing seams

Implement the [M25 static-build boundary](YT-219-224-DOM-STATIC-SITE-PLAN.md#yt-222--static-html-build-contract)
in a later slice: finite public pages readable before/without JavaScript, with
separately selected local Wasm islands. Static hosting of the existing browser
starter waits for Wasm to mount; it is a different output mode.

| Existing source | Reuse and concrete gap |
|---|---|
| [View/HTML](../crates/yetlet/src/html/mod.rs) | render_to_string escapes/validates fragments; HTML-only requires no server/browser dependency |
| [Server document writer](../crates/yetlet/src/server_axum/mod.rs) and [metadata](../crates/yetlet/src/server_axum/document.rs) | Bounded title/language semantics are useful, but current types require server-axum and the writer supplies no stylesheet/island head. Do not import that graph or relocate its public types for this slice |
| [CLI parser](../tools/yetlet-cli/src/lib.rs), [app execution](../tools/yetlet-cli/src/app.rs) | Existing build chooses native/frontend from validated public settings; no static export command/protocol |
| [Settings](../tools/yetlet-cli/src/settings.rs), [owned processes](../tools/yetlet-cli/src/platform/process/mod.rs), [protected filesystem](../tools/yetlet-cli/src/platform/fs/mod.rs) | Reuse standalone app/lock/profile checks, literal frozen Cargo, process-tree ownership and no-replace publication. Current finite filesystem allowlists do not yet implement nested export trees |
| [Wasm startup](WASM-STARTUP.md), [browser build](../tools/yetlet-cli/templates/browser/build-wasm.mjs) | Existing build-time assets and explicitly owned startup/disposal; no automatic hydration or second bundler |

The CLI is an orchestrator and local output writer. The application owns the
finite page Views, public content and export binary. No handler crawl, router
introspection, request server, database query or Auth/Session export is selected.
No new SDK crate/module, template engine, DI, mandatory Settings loader, Wasm or
Node dependency is needed by the HTML-only application.

## Selected CLI and typed request

Proposed invocation, **not executable today**:

```text
yetlet build --static --export-bin site-export --output artifacts/site-v1
yetlet build --static --export-bin site-export --output artifacts/site-v2 --base-path /yetlet/ --release
```

| Choice | Contract for YT-223 |
|---|---|
| Mode | Only build accepts --static. Ordinary build/run/watch/new/generate retain their behavior; no generate static alias |
| Manifest | Existing --manifest-path, default exactly cwd/Cargo.toml; protected standalone package with its own prepared Cargo.lock, no ancestor/workspace inference |
| Export entry | Required --export-bin NAME, a single ASCII Cargo binary name matching [a-zA-Z0-9_-]+ and not starting with a hyphen; no arbitrary command, shell string or extra argv forwarding |
| Output | Required --output DIRECTORY; absolute or app-root-relative native path under an existing protected parent. Entire destination must be absent, including an empty directory or prior generated output |
| Base | --base-path defaults to /; accepts / or an origin-root path of the same lowercase component grammar as page routes, ending in /. No scheme/authority/query/fragment, percent encoding or base element |
| Profile | --release only selects Cargo release; absent selects dev. Neither changes APP_ENVIRONMENT or installs tools/dependencies |
| Islands | --wasm-islands, default false, permits explicit prepared JS/Wasm assets and per-page module entry. It does not trigger frontend builds or install tools |
| Settings | Existing public base/profile/scoped override validation runs first. Static export requires cli.target=native. No new static settings keys/environment aliases; a project with no settings files uses existing defaults |
| Stage selection | After validation, --static selects the export pipeline explicitly. cli.rust.watch and cli.vite auto flags do not start/skip stages; no watcher, TLS startup or implicit npm/Vite execution. Invalid existing settings still fail |
| Errors/help | Unknown, duplicate, missing or misplaced options fail before children/output. Exclusive help/version remain read-only. Explain trusted native execution, create-only output and static versus browser-hosted Wasm in help |

Follow the existing typed parser style with a selected new variant:
Cli::BuildStatic { manifest: PathBuf, export_bin: String, output: PathBuf,
base_path: String, release: bool, wasm_islands: bool }. Existing Cli::Build
fields/signatures stay unchanged. Parsed strings/paths are not proof of safe
filesystem state. Adding an enum variant can require downstream exhaustive
matches to be updated; record that Experimental CLI source migration if YT-223
is implemented. No public SDK options type is added merely to carry CLI settings.
Existing inspect schema1 continues to describe ordinary settings-selected
build/run/watch only; static inspect is deferred, not silently added to schema1.

## Native exporter protocol 1

The selected binary is trusted application code. Reuse the pinned app cwd and
owned process tree to run literal argv equivalent to:

```text
cargo run --frozen --manifest-path Cargo.toml --bin site-export [--release] -- --yetlet-static-export 1 --output PRIVATE_EXPORT_DIRECTORY --base-path /yetlet/ [--wasm-islands]
```

The CLI allocates PRIVATE_EXPORT_DIRECTORY itself with protected ownership,
outside the public output. No user-supplied process shell is used. Existing
offline/no-rustup-auto-install settings apply; [Cargo --frozen](https://doc.rust-lang.org/cargo/commands/cargo-run.html#manifest-options)
locks dependency resolution and disables Cargo network access. It does not
sandbox build scripts or exporter code. Source/dependency fetching and lock
preparation remain separate explicit operations.

The exporter receives only these protocol arguments; stdout/stderr are ordinary
child diagnostics, not the manifest transport. It writes UTF-8 export.json and
declared regular files inside the private export directory, then exits0. Example
descriptor (shown as schema data, not an existing public Rust API):

```json
{
  "schema_version": 1,
  "pages": [
    {"route": "/", "title": "Home", "language": "en", "fragment": "fragments/home.html", "stylesheets": ["assets/site.css"], "island_entry": null},
    {"route": "/guide/", "title": "Guide", "language": "en", "fragment": "fragments/guide.html", "stylesheets": ["assets/site.css"], "island_entry": null},
    {"route": "/404.html", "title": "Not found", "language": "en", "fragment": "fragments/not-found.html", "stylesheets": ["assets/site.css"], "island_entry": null}
  ],
  "assets": ["assets/site.css", "assets/diagram.png"]
}
```

All fields are required, including nullable island_entry; reject unknown/duplicate
JSON keys, wrong types or another schema version. Exactly one / page is required;
/404.html is optional and unique. Title is literal text, language is explicit
with the existing conservative ASCII shape (1–35 bytes; alphabetic primary
subtag of1–8, subsequent alphanumeric subtags of1–8). No server default language
is silently reused. Stylesheets name declared CSS assets, island_entry names a
declared JS/MJS asset or null; arrays contain no duplicates. Fragments are unique
regular UTF-8 files below fragments/, produced from application-owned Views by
render_to_string. Assets are already-prepared, explicit regular files below
assets/; the exporter owns their deliberate selection/copying.

The CLI builds the complete document using a small private static-document
writer: doctype, validated language, charset, early fixed CSP meta, viewport,
escaped title, escaped local stylesheet URLs and selected external module script,
then the trusted rendered fragment and closing tags. Reuse escape_text and
existing language rules without moving server types or adding a raw-HTML View
variant. Fragments are trusted build output, not sanitized third-party HTML:
the CLI cannot prove that an arbitrary executable called the renderer. Raw
Markdown/HTML ingestion and a general head API are outside this contract.

Initial finite limits:256 pages,1024 assets,1MiB descriptor,2MiB per fragment,
16MiB per asset and64MiB aggregate exported bytes; reject oversized inputs before
unbounded allocation or public output. These are implementation bounds, not
throughput/hosting guarantees; larger sites need a separately selected change.

## Routes, links and project prefixes

| Canonical route | Output path within the selected directory |
|---|---|
| / | index.html |
| /guide/ | guide/index.html |
| /guide/install/ | guide/install/index.html |
| /404.html |404.html, when explicitly supplied |

Except the special404 route, components use [a-z0-9][a-z0-9_-]{0,62}, at most
8 components/240 bytes total. Reject missing trailing slash, uppercase aliases,
empty/dot components, encoded separators/dots, backslashes, queries/fragments,
and the reserved top-level assets directory. Reject Windows device stems con,
prn, aux, nul, com1–com9 and lpt1–lpt9 case-insensitively. Do not normalize unsafe
input into an accepted route. The output mapping is fixed, not a path supplied
by the exporter. Detect duplicate routes/files, file-versus-directory prefixes
and ASCII-case-folded collisions before writing public files.

Artifact path components use lowercase ASCII letters/digits/dot/underscore/hyphen,
start with a letter/digit, do not end with dot, and reject dot components/device
stems (including before an extension). Apply the same depth/length bounds and
collision checks to fragments/assets. No absolute path, drive/UNC spelling,
percent escape, symlink, hard-linked input, device, socket or Windows reparse
point is accepted by CLI reads/writes. Reject unlisted private-export files;
only export.json, declared fragments/assets and their necessary directories
belong there. The CLI generates an empty .nojekyll file; it is the sole special
public hidden file. No .env, settings, Cargo files, manifests or source maps are
copied to the public tree.

Base /yetlet/ changes document links/assets to /yetlet/...; it does not add a
yetlet directory inside the artifact. The host mounts the artifact at that URL
prefix. Application-owned URL construction joins the selected base with declared
canonical routes/assets; anchors remain #id. No HTML/CSS/JS textual rewriting or
general link crawler is promised. The application owns fragment IDs and local
link correctness; YT-223 verifies its concrete routes/anchors and nested refresh.
Root-prefixed asset URLs also work when404.html is served for an unknown nested
URL. No SPA index fallback or redirect rules are generated. Hosts must support
directory indexes; [GitHub Pages documents a custom404.html](https://docs.github.com/en/pages/getting-started-with-github-pages/creating-a-custom-404-page-for-your-github-pages-site),
but the CLI does not itself select HTTP status codes or publish Pages.

## Public assets, CSP and optional islands

HTML-only assets are CSS, PNG, JPG/JPEG, WEBP or ICO. Selected island mode also
admits JS/MJS and WASM. SVG, fonts, responsive image manifests, arbitrary
downloads and other active content are deferred. Extensions are a finite file
selection, not proof that untrusted bytes are safe. MIME correctness is a host
requirement; YT-223's isolated server checks HTML/CSS/image/JS/Wasm MIME as used.

The static writer emits an early meta policy with default-src 'none',
style-src 'self', img-src 'self', base-uri 'none', object-src 'none' and
form-action 'none'. HTML-only adds script-src 'none' and connect-src 'none';
selected island mode uses script-src 'self' 'wasm-unsafe-eval' and connect-src
'self'. There is no unsafe-inline, unsafe-eval, remote script/image origin or
inline state bootstrap. This static-local policy does not widen existing server
CSP or YT-221 policies. Host headers may further restrict it.

Meta CSP is not equivalent to response-header CSP: it applies after parsing the
meta element, and [does not support frame-ancestors, sandbox or report-uri](https://w3c.github.io/webappsec-csp/#meta-element).
Do not claim framing protection from the artifact. A capable host may add
frame-ancestors 'none' and other response headers; configuring a host is separate.
No general _headers file, nonce service, CSP report collector or TLS setup is added.

Island selection requires already-built local module/glue/Wasm assets from
existing tooling; failures do not trigger a second bundler or implicit install.
The application supplies the external bootstrap and owned mount roots, readable
initial HTML, selected startup shell, failure text and explicit cleanup. Ready
means mount succeeded; failed/disabled JS or missing Wasm leaves public content
readable. No general hydration, request server, automatic fetch/retry or full-page
spinner is required. Imports/load URLs and styles must honor the chosen base;
the CLI copies the declared asset graph exactly and does not rewrite modules.

## Ownership, frozen inputs, receipts and failure

1. Validate invocation/settings, app/lock, output parent and absent destination
   before a child or public write. Reject output intersecting the app source,
   Cargo build directory or private work directory; a new app-root artifacts/
   descendant or a separate protected parent is suitable.
2. Record manifest/lock bytes and bounded regular src/ plus optional build.rs
   input hashes before execution; compare them again before success. Use the
   same file/depth/aggregate limits as the export inventory. Changes abort.
   This is a receipt of selected files, not an immutable source snapshot or
   complete hermetic closure of proc macros, includes, path dependencies,
   toolchains, environment and arbitrary build-script IO.
3. Create owned private staging; supervise the single Cargo/export tree with a
   10-minute wall budget and existing platform cancellation/termination ownership,
   allowing at most10 additional seconds for forced cleanup. Nonzero exit,
   timeout, cancellation or cleanup uncertainty cannot publish success. No
   automatic retry, daemonization or process-group escape is supported.
4. After a zero exit and verified descendant cleanup, validate the complete
   descriptor/file inventory. Build a fresh private public tree, bounded-check
   and hash every final file, and prepare the private receipt before committing
   the absent destination using protected no-replace publication. Recheck pinned
   directory/file identities; refuse unsupported filesystem semantics/platforms.
5. Only owned temporary paths may be removed. No --force, overwrite, recursive
   clean or replacement of a previous output, even one created by this CLI.
   For another build select a new output directory. Existing outputs remain the
   last-good artifact; deleting/promoting/deploying them is application-owned.
   Abrupt crash can leave a private stage, but never justifies deleting an
   unknown directory. A final receipt/cleanup failure after publication reports
   failure with output-present status and never retries or removes that output.

Private receipts live outside the public tree under the app's owned
target/yetlet-static/RUN_ID/. Record schema1, CLI/export protocol versions,
selected base/profile/options, package/lock/input hashes, declared page mapping,
asset/output sizes and SHA256, child exit/cancellation and cleanup/publication
status. Store failure receipts where safely possible; receipt-write failure is
explicit, not a false success. No secret values, inherited environment dump or
private export.json/fragments enter the public artifact. Receipt schemas are
distinct from doctor/inspect/browser manifests and do not imply signatures.

No application runtime settings/secrets are automatically deserialized into
Views. Strip APP_* and existing development/config/secret selector variables
from exporter children; pass only the public protocol values explicitly. Normal
toolchain environment and access to the user's files remain: trusted Cargo/app
execution can read credentials or open a network socket. It is not a sandbox,
credential isolation or proof that arbitrary application output is public-safe.
The application must deliberately classify its content; use an unprivileged
build environment for untrusted sources. Child diagnostics are application-owned;
CLI policy errors must not echo raw content, environment or credentials.

## YT-223 minimum consumer and affected verification

Deliver a standalone application with explicit [workspace], prepared lock and
native site-export binary depending only on the HTML capability. Use a finite
home, nested guide, optional404, local stylesheet/image, working links/anchor
and Unicode/escaped text. No repository clone or server/browser dependency is
inherent in that application. A separate explicitly selected island uses the
existing prepared toolchain/assets; HTML-only never needs those tools.

Only verify changed behavior: CLI/help/typed request and refusal ordering;
schema/route/path/collision/size limits; metadata/escaping/meta-CSP; unsafe input,
no-replace output and failed-export/last-good/cancellation cleanup; actual
standalone build with no backend and dist-only hosting, no-JS reading/nested
refresh/assets/anchors at / and /yetlet/; selected island startup/failure; touched
GNU/MSVC process/filesystem behavior and affected CLI packaging/external consumer.
Reuse old HTML/DOM/browser/release evidence instead of replaying all samples.
No new harness framework or certification machinery is selected. Retain first
failures and obtain independent Reviewer PASS before YT-223 Done.

## YT-222 acceptance and limits

This item changes documentation only: source inspection and the finite contract,
current navigation and Backlog/Roadmap synchronization. Run docs/link/structure/
DAG/whitespace checks; no implementation, package build or browser replay.
Independent Reviewer PASS is required before YT-222 Done. YT-223/224 stay Todo;
live mdBook/Pages and published SDK/CLI are unchanged. New work remains
uncommitted/unpushed.

Independent acceptance: **PASS0**, zero issues/required followups. The reviewer
inspected all six documentation changes against existing contracts and source,
the docs/whitespace receipts and21 matching input hashes. The15-path unstaged
worktree status matches; all nine non-shared accepted YT-221 inputs are unchanged,
and the five shared-document baseline/delta bindings match. Registered reviewer
model gpt-6-astra and effective high effort are verified. No runtime checks were
replayed or source edits made by the reviewer. Only acceptance/Done updates
follow this verdict; it grants no implementation or Git/deployment authority.

Evidence is retained at target/yt222-20261031/ and outside the repository under
${XDG_DATA_HOME:-$HOME/.local/share}/yetlet-release-evidence/runs/yt222-20261031/.
YT-222 is Done at contract scope. Next is YT-223's separately selected minimal
implementation; YT-223/224 remain Todo. Existing YT-221 failures/evidence remain.
