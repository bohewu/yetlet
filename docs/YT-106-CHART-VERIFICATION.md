# YT-106 — Admin loaded-item chart

Date: 2026-10-20. Base checkout: `ef20ae8`, the committed, independently
reviewed [interoperability study](YT-106-FRONTEND-INTEROP-RESEARCH.md).
The user separately selected its first Chart.js trial. This record covers the
application slice; YT-106's earlier Done status remains research-only evidence.
Independent Reviewer `/root/yt106_chart_impl_review_oct20` PASS has zero issues
or required followups. This bounded application slice is complete; broader
package/platform qualification remains separate.

## Delivered boundary

The Admin triage workspace shows Open, Active, Blocked and Closed totals for
the currently loaded items, including unsaved status drafts. The existing limit
is 12 recently updated items, not the whole project. Filtering and reordering
do not change that scope. Reset restores the saved item's contribution; an
explicit successful Save keeps the existing semantic HTTP boundary.

The server renders a native definition list using the same Rust counting
function as the browser. The native summary stays visible with or without JS.
The decorative, read-only canvas is hidden from accessibility APIs. Existing
native item/editor links remain the fallback; no tooltip or chart action is
needed to obtain the numbers.

The app pins `chart.js` 4.5.1 and resolved `@kurkle/color` 0.3.4, both MIT.
Only the bar controller/element and category/linear scales are registered,
following the [package's bundler guidance](https://www.chartjs.org/docs/latest/getting-started/integration.html).
The private [TS adapter](../examples/admin-dashboard/frontend/status-chart.ts)
owns the canvas and exposes update/destroy. Its fixed four counts are checked
against the loaded-item bound and copied out of the borrowed WASM typed array;
equal values skip redraw. Rust owns this handle with the existing graph lifetime,
updates surrounding native totals, and catches widget failures without faulting
the workspace. Only Chart.js mutates the canvas. The host sits outside keyed rows.

The per-start active token is installed before WASM/package awaits. Retiring a
mount prevents a late import from creating an instance. Page disposal drops the
Rust graph and destroys its chart; partial startup failure removes the canvas.
The adapter uses [update without animation and destroy](https://www.chartjs.org/docs/latest/developers/api.html)+rather than recreating a chart on each graph commit. No framework adapter crate,
registry, callback bus, settings schema or CLI option was added. This optional
enhancement belongs only to the existing interactive triage route.

## Integration finding

The first build's nested dynamic imports made Vite 8.3.4 share its preload
helper through the entry, producing an entry/triage manifest cycle. The existing
asset validator correctly refused the snapshot: two Admin tests failed and the
CLI run exited before READY. The minimal repair keeps both dynamic imports in
the application entry and passes a private loader to triage. The resulting
manifest is acyclic. No asset validator, MIME allowlist, file-size limit or CSP
was relaxed; `vite.config.mjs` is unchanged. This is an application bundling
observation, not a new shared framework API requirement.

## Focused verification

Ubuntu Linux x86_64, system Chrome 155.0.8059.84, existing Playwright.
Browser plugin/skill were unavailable in this session. Temporary evidence:
`/tmp/yetlet-yt106-chart-20261020-xlivefpe/`. No identity-provider configuration,
private secrets, terminal login codes or browser storage state were saved in
repository changes. Synthetic SQLite fixtures and screenshots stay outside Git.

| Command / surface | Result |
| --- | --- |
| `cargo test --offline --locked -p yetlet-board-triage -p yetlet-lab-operations-board` | PASS: 4 triage + 18 Board tests. New checks cover draft/filter/Reset/empty/four-status counts and the server's 12-item summary |
| `cargo check --offline --locked --target wasm32-unknown-unknown -p yetlet-board-triage` | PASS |
| `cargo clippy --offline --locked -p yetlet-board-triage -p yetlet-lab-operations-board --all-targets -- -D warnings` | PASS |
| `cargo clippy --offline --locked --target wasm32-unknown-unknown -p yetlet-board-triage -- -D warnings` | PASS |
| `APP_ENVIRONMENT=Production APP_OIDC__ENABLED=false APP_SESSION__BACKEND=memory tools/yetlet-cli/target/debug/yetlet build --manifest-path examples/admin-dashboard/Cargo.toml` | PASS: existing settings select WASM/bindgen/Vite then native build |
| `npm run typecheck --prefix examples/admin-dashboard/frontend` | PASS after build regenerates bindings |
| `cargo test --offline --locked --manifest-path examples/admin-dashboard/Cargo.toml --all-targets` | PASS: 17 tests, including actual manifest/static-route checks after the cycle repair |
| `cargo clippy --offline --locked --manifest-path examples/admin-dashboard/Cargo.toml --all-targets -- -D warnings` | PASS |
| `cargo xtask check docs` | PASS: 75 documents, 106 backlog items, internal links/Markdown/whitespace/DAG |

`admin-browser.mjs` adapts the existing Admin journey in the temporary directory.
`browser-result.json` records these passing checks:

- Nonblank initial chart and native totals; draft status changes redraw the same
  canvas. Filter/reorder retain it, Reset restores counts, and local actions send
  zero POSTs. One explicit Save returns 200 and sends exactly one POST.
- Desktop 1440x1000 and mobile 390x844 fit after the responsive resize settles.
  Screenshots `desktop-chart.png` and `mobile-chart.png` were visually inspected.
- Pagehide removes the owned canvas while leaving native totals readable. Reload
  creates one chart from a fresh server bootstrap. This is not a new promise to
  rehydrate an arbitrarily mutated same-document DOM.
- JS disabled preserves server totals and links to the native editor. A blocked
  chart chunk or unavailable canvas context still permits Rust edits/native
  count updates; partial canvas construction is cleaned up.
- A delayed chart import resolving after pagehide cannot mount a retired runtime.
- No normal console warning/error or CSP violation. The deliberately aborted
  chart request is the sole expected resource error in its isolated case.
  Existing strict CSP and WASM MIME pass; ordinary SSR pages remain script-free.

The actual app was started with **`yetlet run`**, isolated loopback port, fresh
synthetic SQLite and memory sessions, then stopped with SIGINT through the CLI.
Native outcome was `{ code: null, signal: "SIGINT" }`, not exit 0. The app's
drain message was observed and the port was closed. No preview remains running.
Earlier manifest-cycle failures and test-assumption corrections remain in the
temporary evidence rather than being relabeled as passes.

## Package and delivery cost

Same-host baseline dist was captured before this slice. Final files were measured
with Python `gzip.compress(..., mtime=0)` (default level 9); these are artifact
sizes, not HTTP compression or performance claims. `baseline-assets.json` and
`final-assets.json` retain individual file sizes and SHA-256 identities.

| Asset category | Before raw / gzip bytes | After raw / gzip bytes | Delta raw / gzip bytes |
| --- | --- | --- | --- |
| JS, all chunks | 12,787 / 4,697 | 146,603 / 51,475 | +133,816 / +46,778 |
| CSS | 28,484 / 5,565 | 30,433 / 5,787 | +1,949 / +222 |
| WASM | 312,572 / 105,173 | 311,816 / 105,533 | -756 / +360 |

The new chart chunk is 133,030 raw / 46,509 gzip bytes. It is requested lazily
after the triage WASM initialization; ordinary native pages request no chart or
WASM. All assets pass the existing 2 MiB per-file / 16 MiB snapshot bounds.
There is no worker, external CDN, font, new Rust dependency or core renderer
change. No profiling or speedup conclusion follows from these byte counts.

## Remaining scope

This qualifies the selected Ubuntu/Chrome app integration, not all Chart.js
features, Windows/browser matrices, production deployment or an accessible
interactive chart. CodeMirror remains a separate conditional trial with its
unresolved CSP feasibility decision. A native textarea remains the current
editor. No additional implementation is selected by this record.
