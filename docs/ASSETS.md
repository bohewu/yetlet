# Experimental optional asset contract (YT-098)

This is an **application-owned experiment**, locally Chrome-verified and
independently reviewed at its bounded YT-098 scope, under
[ADR 0005](adr/0005-opt-in-capability-modules.md) and
[handler-first ADR 0006](adr/0006-handler-first-mvc-optional.md).
[Sources and run instructions](../experiments/asset-contract/README.md) and
[verification boundaries](YT-098-VERIFICATION.md) are separate from the accepted
[Vite feasibility spike](VITE-SPIKE-VERIFICATION.md). No shared or stable Yetlet
asset API is promoted by this task.

## Two actual producers before extraction

`vite.rs` translates real Vite 8.3.4 JSON manifests into an application-owned
`Graph` of `Node` records. Both plain Vite and official Tailwind-Vite 4.3.3
outputs are consumed, including JS, split CSS, SVG and dynamic imports.
`standalone.rs` declares a single CSS entry for actual standalone Tailwind 4.3.3
CLI output, without a Vite manifest. Its build uses the already pinned local
Tailwind theme/utilities CSS input and an explicit Rust source scanner target.
No Node process participates in that compilation or in serving either producer.

Both adapters call the same `Snapshot::load(absolute_root, graph, entry)`.
A node has a validated relative `file`, ordered `imports`, ordered `dynamic`
references, ordered `styles`, and referenced `assets`. The producer-neutral
module knows nothing about Vite schema, Tailwind classes, CLI flags or HTTP
routers. A selected entry is CSS or JavaScript. Its immutable-in-memory snapshot
contains only its static/dynamic reachable file closure. An unrelated entry's
exclusive files are not served. The whole bounded graph is validated, including
references in unselected records; unselected files are not opened. Multiple
entries can load separate snapshots and select their own initial links.

`asset(relative_path)` returns verified bytes/MIME or absence; `links(mount)`
returns initial stylesheet links plus an optional module script. `images()`
exposes validated static SVG paths for ordinary escaped component attributes.
The application composes ordinary Axum handlers/state and Yetlet HTML rendering.
No Controller, plugin registry, framework service locator or default asset loader
is introduced. HTML, useful SSR and strict CSP remain independent of JS.

## CSS cascade and graph policy

The initial CSS order is static dependency-first DFS, preserving each imports
array's order, then each node's CSS array order. Each CSS path appears at its
first occurrence only, stably across shared dependencies and repeated paths.
The selected standalone CSS file is itself an initial stylesheet. No sorting by
filename, map key or asynchronous completion changes the cascade.

Dynamic references are validated and snapshotted, but excluded from initial
links, images and traversal. Their JavaScript requests them later. The actual
Vite runtime owns its dynamic stylesheet insertion/dedup; the server neither
adds lazy CSS eagerly nor reinserts shared CSS on requests. Native tests prove
stable shared-path dedup across a diamond and dynamic branch. The Chrome product
test asserts initial shared-before-entry order, no lazy requests before click,
unchanged initial stylesheet prefix, no duplicate shared path and one lazy CSS
addition after two clicks. **Those assertions passed in the local Chrome
continuation**, with source identity and cleanup recorded in the verification.

Missing static/dynamic references, non-JS import targets and any static/dynamic
cycle fail startup. This deliberately conservative cycle refusal is narrower
than full JavaScript bundler semantics; it is not universal Vite compatibility.
CSS files and SVG/JS source contents are trusted build output, not parsed or
sanitized. This contract does not rewrite CSS imports/URLs, compute an AST or
sanitize arbitrary hostile SVG. Strict CSP restricts actual browser loads.

## Paths, MIME, bounds and filesystem trust

Relative names are nonempty ASCII alphanumeric plus `/`, `.`, `-`, `_`, at most
240 bytes, with no empty, dot or dot-dot segments. Absolute URLs/paths,
backslashes, percent encodings, query/fragment syntax, quotes and whitespace are
rejected. Extensions are exactly lowercase `.css`, `.js`, `.svg`, mapped to
`text/css; charset=utf-8`, `text/javascript; charset=utf-8`, `image/svg+xml`.
No raw manifest string is accepted as HTML. Graph keys and Vite descriptive
metadata also pass the safe alphabet/segment rules; unsupported Vite fields
fail schema parsing. Mounted URLs stay same-origin. HTML text and attributes
use the existing escaped renderer; initial tags use only validated filenames
and mounts. No environment values, runtime state or server secrets are serialized.
A synthetic backend-only sentinel is used in native producer tests.

Limits are fixed for this experiment: 128 KiB Vite manifest, 128 graph nodes,
512 aggregate dependency/CSS/asset references, 256 distinct file paths, 2 MiB per
file and 16 MiB total snapshot bytes per entry. DFS depth is bounded by nodes.
This server explicitly loads three snapshots, at most 48 MiB of retained asset
bytes, plus bounded rendered HTML/metadata. No unbounded per-request build or
filesystem operation occurs; request paths only select in-memory whitelist keys.
Manifest files, source maps, absent files and query variants are never served.
All responses retain exact `DEFAULT_CSP`, `nosniff` and conservative cache headers.

The operator supplies explicit absolute asset roots, separate from cwd. Roots
must be real directories; after canonicalizing the trusted root, every requested
build-path component is checked with `symlink_metadata`. All symlink components
are rejected, including internal aliases and manifest/directory escapes. Stable
nonregular files are rejected **before opening**, and open-handle metadata and
bounded reads recheck regular-file/size requirements. No request rereads disk,
so changing/removing files after startup cannot change snapshot bytes.

These std filesystem checks are **not atomic hostile-filesystem isolation**.
An attacker concurrently swapping directories/files between checks and open can
race them, including the pre-open nonregular-file check. Ancestors of the
explicit root and the tree must be trusted, operator-controlled and frozen while
startup reads occur. Content is not authenticated by this contract. Run against
an immutable deployment build tree; stronger descriptor-relative no-follow/race
isolation is not claimed or implemented here.

## Cache and retention

Every profile, including standalone CSS, returns `Cache-Control: no-store`.
A Vite-looking hashed filename alone proves neither content identity nor retained
old-build availability. This experiment does not enable immutable caching,
validate a filename digest or persist prior snapshots across server restarts.

Any future immutable policy must verify a content digest against its URL, and
operators must retain **all** old digest assets needed by old HTML and lazy JS
through the full cache/rollout lifetime. An in-process snapshot protects current
requests only until that process exits. Cache expiry cannot repair missing lazy
chunks after a rollout. [YT-101](YT-101-VERIFICATION.md) qualifies bounded cloud
delivery of these snapshots. The optional application/deployment-owned
[YT-103 plan](YT-103-ROLLING-ASSET-PLAN.md) compares retention options, and its
[local evidence](YT-103-ROLLING-ASSET-VERIFICATION.md) records an independent
static service retaining two complete build graphs through rollout and rollback.
That arrangement does not add retention to these templates or core. The snapshot
caveat also applies to a digest URL produced by the unchanged YT-091 template.

## Extraction decision and next boundary

Both producers now use one narrow application-owned module, so producer-neutral
loading is grounded in real outputs rather than a Vite-only abstraction. That
supports a future extraction boundary around validated graph input, selected
entry links and bounded verified bytes. It does **not** yet justify publishing
or stabilizing `crates/yetlet-assets`: both adapters still live in one
experimental application, without a demonstrated shared-framework consumer.
The local Chrome gate and subsequent independent Reviewer both passed; the
verification retains the earlier missing-evidence failure as historical evidence.
The smallest current delivery keeps this contract app-owned, without an ADR
change, new dependency in core, or duplicate shared implementation in CLI.

YT-098's real-browser gate and independent review have passed. The next scoped
item is YT-099, optional production Vite CLI/profile integration, under separate
implementation authorization. HMR, Windows Vite
qualification, cloud delivery/retention, default/browser changes and public
release are not implemented here. YT-092 through YT-096 remain untouched Todo.
