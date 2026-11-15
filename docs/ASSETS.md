# Experimental optional asset contract (YT-098)

This is an **application-owned experiment**, locally Chrome-verified and
independently reviewed at its bounded YT-098 scope, under
[ADR 0005](adr/0005-opt-in-capability-modules.md) and
[handler-first ADR 0006](adr/0006-handler-first-mvc-optional.md).
[Sources and run instructions](../experiments/asset-contract/README.md) and
[verification boundaries](https://github.com/bohewu/yetlet/blob/b610acc7e098020b6b59dfa0d1ab2b75b059e656/docs/YT-098-VERIFICATION.md) are separate from the accepted
[Vite feasibility spike](VITE-SPIKE-VERIFICATION.md). No shared or stable Yetlet
asset API is promoted by this task.

### Historical optional asset and Vite proof boundaries

The early YT-098 application-owned producer-neutral graph, YT-099
production Vite build, YT-100 development bridge and YT-101 generated
cloud configuration were separate bounded experiments. Their current
usage follows the [CLI Vite instructions](CLI-GUIDE.md#vite-execution),
[generator matrix](CLI-GUIDE.md#current-generator-source-matrix) and
[deployment profiles](PRODUCTION-DEPLOYMENT-PROFILES.md), not the original
task checklists. They did **not** add a mandatory framework asset/runtime
registry, transparent global HMR, automatic deployment or generalized
production/Windows/cloud qualification. Keep strict prebuilt manifest/source
closure, CSP and app-owned public versus private asset separation. The dated
verification/negative cases are recoverable from Git commit `b610acc7e098020b6b59dfa0d1ab2b75b059e656`
(local original identity; GitHub reachability remains unverified).

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
chunks after a rollout. [YT-101](https://github.com/bohewu/yetlet/blob/b610acc7e098020b6b59dfa0d1ab2b75b059e656/docs/YT-101-VERIFICATION.md) qualifies bounded cloud
delivery of these snapshots. The application/deployment-owned
[YT-103 local qualification](ASSETS.md#historical-single-host-rolling-asset-qualification) actually
validated a separate static service containing the complete two-version graph
union (including late JS/CSS) through update, rollback, one Pod termination
and old-tab lazy imports. All 2,364 measured SSR responses succeeded in that
specific local K3s source/run and an eight-second held request completed after
SIGTERM; missing/private-file handling, image identity and cleanup remained
bounded. The original provider comparison and deployment-plan history are
recoverable at fixed local Git `ed7a082b1f9f72430f06866f7d550b0ed9ecbae9`.
No production HA, node/zone failover, CDN retention guarantee or automatic
snapshot retention is provided by Yetlet's generators or core. The snapshot
caveat also applies to a digest URL produced by the unchanged YT-091 template.

## Historical single-host rolling asset qualification

The **2026-10-18** local `YT-103` verification used three Multipass K3s
VMs on **one physical host** (`v1.36.4+k3s1`), three app replicas and two
separate Nginx gateway/static replicas. Each release had seven reachable
CSS/JS graph files; the union held **11** verified files. The real
application/deployment-owned static service preserved v1/v2 asset bytes
across an upgrade/rollback; publishing different bytes at the **same URL**
was explicitly rejected. Original frozen image **index digests** and
K3s imageID **config digests** were different identities, not an error.
The complete original report (not its raw execution artifacts) is recoverable
from **local Git** `47b0a99ffde069236eaa82ef9fac34c5c6348f3c`
(public historical reachability unverified). The image/source receipts,
raw logs and browser screenshots were recorded separately in Git-ignored
`artifacts/yt103-rolling-20261018/`; Git history alone cannot restore
those task-owned evidence bytes.

| Measured local phase, 10 SSR requests/s | HTTP 200 | Observed p95 (ms) |
|---|---:|---:|
| v1 → v2 | 537 | 2.040 |
| Rollback v2 → v1 | 627 | 2.114 |
| Graceful single Pod deletion | 261 | 2.094 |
| Final v2 rollout and retention | 939 | 1.947 |
| **Combined** | **2,364** | Different phase populations; no pooled p95 asserted |

Within only those finite observation windows there were **no observed SSR
404, 5xx, connection error or timeout**. An eight-second in-flight handler
returned **200** around 1.987s **after observed SIGTERM**; it did not prove
hard kill, PDB enforcement or node failure behavior. Actual scheduling
collapsed from app Pods on three VMs to two after rolling; these VMs shared
one physical fault domain. No multi-host/zone HA, SLA, automatic rollback
or persistent server-side asset retention follows.

Two actually held Chrome155 pages loaded previously *unrequested* v1/v2
lazy JS and CSS **200 after the opposite version had fully replaced app
Pods**, without page reload. A separate favicon **404** remained in the
browser console and is not silently counted as a lazy-load failure.
The conservative old-build eligibility covered **30 seconds plus a 5-second
margin** after the last v1 app Pod retired; **344** dedicated old/new lazy
requests during that window returned 200 and matched SHA-256. After a
separately invoked rollback-unpin and static-image GC, v1-only lazy JS/CSS
returned **expected 404** while all seven v2 graph files survived. There was
**no continuous SSR availability run during static-service GC**; the
35-second lab window is not a production retention recommendation.

Preserved original setup negatives: non-root app could not read exported
public TOML due to inherited permissions; Nginx could not create its
temporary cache directory; a lazy-asset HTML oracle accidentally matched
DOM IDs; the browser wrapper's direct ESM import failed before an existing
CommonJS alternative was used. These were bounded image/fixture/browser
setup corrections, **not product-source upgrades**. The original execution
receipts, failed image archive, verified screenshots, seven/eleven-file graph
hashes and cleanup remain in local source/evidence history. Cleanup removed
only the owned namespace, temporary images/uploads and test Docker tags;
the three K3s nodes, pre-existing resource identities, Valkey/OpenBao and
shared caches were preserved. Actual general serving continues to use
`no-store`; an old hashed URL or in-process snapshot alone does not extend
asset lifetime.

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

## Optional generated application CSS

The [published CLI Style integration](CLI-GUIDE.md#optional-application-owned-rust-css-yt-239-working-tree) supplies a separate small
application-owned CSS consumer in the opt-in Style templates, first published
in CLI `0.3.0` and available in CLI `0.4.0`. The experimental shared asset API
above has its own contract. Development uses a stable selected
CSS URL. Successful standalone builds accept a private logical `app-css` record
and `app.<full-lowercase-sha256>.css`; the consumer verifies that digest against
its exact immutable startup snapshot before linking/serving. Only the selected
same-origin CSS route is exposed, with CSS MIME, nosniff, no-store and unchanged
`DEFAULT_CSP`. Source, private records and generators are never static routes.

Standalone Tailwind composes its document link before generated CSS. Vite imports
the generated stable source after app.css in its normal graph, retaining Vite's
manifest/hash/link ownership with no duplicate standalone link. Cascade layers,
importance and specificity still precede source order. Prior production acceptance
survives failed dependent builds; a post-CSS-replacement metadata failure is
explicitly unaccepted and requires operator reconciliation. Private owner/production
pending intents retain previous accepted metadata; producers and the generated
native loader refuse either unfinished intent even when current hashes match.
Acceptance commits only at marker removal after all fallible writes/checks.
Preserve evidence and explicitly verify rollback or rollforward before clearing a
retained marker; ordinary startup/retry never reconciles it. This supplies no
retention, HA, cleanup or deployment guarantee. The CLI guide records accepted
publication and platform evidence, and Todo's qualification records its bounded
Linux sample results. [CLI Style evidence](CLI-GUIDE.md#style-release-and-platform-evidence),
[Todo CSS example](../examples/todo-server/README.md#app-owned-rust-styles-and-typed-tag-builders)
and [original Linux qualification](https://github.com/bohewu/yetlet/blob/e49657922bcdfe238cd1f79061178a32a4622a01/docs/YT-240-TODO-STYLE-QUALIFICATION.md#platform-matrix-and-acceptance-boundary)
