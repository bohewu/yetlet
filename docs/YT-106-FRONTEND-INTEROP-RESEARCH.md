# YT-106 — existing frontend packages from Rust/WASM

Date: 2026-10-20. Inspected checkout: `ebea759`, which commits the independently
reviewed weak snapshot method and Admin/Todo adoption. This is research only:
no package installation, widget implementation, adapter crate or runtime change.
Independent Reviewer `/root/yt106_frontend_interop_research_review_oct20` PASS
has zero issues or required followups and approves research-only closure.

Subsequent, separately authorized implementation:
[Admin Chart.js slice](YT-106-CHART-VERIFICATION.md). The dated observations and
proposals below describe the research checkout, not that later implementation.
The separately selected [CodeMirror CSP study](YT-106-CODEMIRROR-CSP-RESEARCH.md)
inspects published packages and isolated Chrome style mechanisms. It recommends
a response-local style nonce for a later trial; actual editor qualification
remains separate.

## Recommendation

Start with an application-owned TS adapter around an existing package, mounting
into its own ordinary `div`. Reuse the existing Vite build and wasm-bindgen
boundary when Rust owns the local state. A JS-only island is sufficient when
the widget does not need Rust state; do not load WASM solely to host a JS chart.
Keep one owner for each widget's descendant DOM and an explicit update/destroy
boundary. No new core renderer or public widget framework is justified yet.

Select **Chart.js** for the first proposed trial: four bars summarizing the
currently loaded Admin items. Select **CodeMirror 6** as the editor candidate,
but make a focused CSP feasibility check a prerequisite to adopting it. Its
runtime styles are not automatically compatible with the current policy.
Keep the existing textarea if that check fails. These are separate later
selections, not implementation authorized by this report.

## What the checkout already supplies

| Source | Existing capability and relevant limit |
| --- | --- |
| [Admin app entry](../examples/admin-dashboard/frontend/app.ts) and [triage bootstrap](../examples/admin-dashboard/frontend/triage.ts) | An external module dynamically selects the Rust/WASM page and owns pagehide disposal; this is startup glue, not a package adapter. A future asynchronous package load must install its lifetime guard before its first await |
| [Vite config](../examples/admin-dashboard/frontend/vite.config.mjs) and [build step](../examples/admin-dashboard/frontend/build-wasm.mjs) | Existing TypeScript/npm bundling, build-time WASM/bindgen, manifest, external assets and same-origin `/static/`; packages need no second delivery mechanism |
| [Admin browser runtime](../examples/board-triage/src/browser.rs) | Explicit graph/listener owner, weak handles and per-mount lifetime checks; render currently writes native controls and keyed rows. It has no foreign-widget owner |
| [Simple DOM mount](../crates/yetlet-browser/src/dom/mod.rs) | Replaces all target descendants. Its allowlist excludes canvas, custom-element tags and many editor nodes; it cannot render an arbitrary third-party subtree |
| [Keyed ownership](../crates/yetlet-browser/src/dom_keyed/mod.rs) | Retains keyed element state and reorders with insert_before; removal detaches the node before dropping its entry state. It does not promise a foreign library's teardown callback before DOM removal |
| [Triage route](../examples/lab-operations-board/src/triage.rs) | Escaped bounded bootstrap, selected-page module/WASM policy, native item links; `style-src 'self'` has no runtime stylesheet nonce |
| [Asset snapshot](../examples/admin-dashboard/src/vite.rs) | JS/CSS/SVG/WASM allowlist, 2 MiB per file and 16 MiB total; arbitrary worker/font/image assets are not already supported |

Do not pass library-created nodes back through `yetlet-dom::mount`, reconcile
their internals or loosen its allowlist to admit arbitrary widget markup.
For the first trial, keep widget hosts outside the keyed item list. The parent
runtime can then destroy them explicitly before removal; this avoids needing
a new keyed teardown hook. These are app integration constraints, not defects in
the current bounded renderers. Preserve [ADR 0003](adr/0003-renderers-and-loading.md)
and [ADR 0007](adr/0007-bounded-keyed-dom-and-local-graph.md).

## Compare the integration paths

TS glue and wasm-bindgen are complementary, not competing runtimes:

| Path | Fit and cost | Decision |
| --- | --- | --- |
| Plain JS/TS island | Package owns its DOM; inputs can come from escaped SSR data. No Rust bridge needed for an independent read-only widget | Smallest option when Rust state is unnecessary |
| Small TS adapter plus existing wasm-bindgen | TS handles package types/configuration; Rust passes bounded values/callbacks and retains the instance lifetime | Recommended when the widget follows the Rust graph |
| Bind the entire package API directly in Rust | Possible, but exposes many JS options/types/exceptions to Rust and still needs JS assets/build wiring | No broad Rust binding crate selected for one chart/editor |
| Web Component | Useful for an existing custom element or a widget consumed across applications; adds properties/events, registration and lifecycle handling | Optional later boundary, not necessary wrapping for this first trial |
| Separate React root | Appropriate only when the chosen component requires React; adds React/runtime/component assets and root disposal | Neither selected package requires this layer |

The [wasm-bindgen structural-method facility](https://wasm-bindgen.github.io/wasm-bindgen/reference/attributes/on-js-imports/structural.html)
can call methods on passed JS objects. A private TS factory/object passed into
the app's Rust mount avoids global service lookup and hard-coded cross-crate TS
module paths. The [catch facility](https://wasm-bindgen.github.io/wasm-bindgen/reference/attributes/on-js-imports/catch.html)
can represent thrown JS exceptions as Results. The exact private bindings belong
to a later app trial; these facilities do not constitute an implemented adapter.

[Custom-element lifecycle documentation](https://developer.mozilla.org/en-US/docs/Web/API/Web_components/Using_custom_elements#lifecycle_callbacks_and_state-preserving_moves)
explains disconnect/connect callbacks during insertBefore moves. Existing keyed
reordering therefore must not be assumed to preserve a custom element's internal
state solely because the outer DOM node survives. Shadow DOM also does not remove
CSP, teardown or data-boundary obligations. Prefer properties for structured data
and explicit events if an actual Web Component is later selected.

[React's root API](https://react.dev/reference/react-dom/client/createRoot)
provides render/unmount for a separate owned container. It is an option for a
React-only component, not a way to treat React components as Rust components or
hydrate arbitrary Yetlet SSR markup. Keep native fallback content outside such
a root, whose initial render clears its contents.

## Proposed minimum app contract

Use two specific widget instances, not a generic registration/protocol system:

| Boundary | Minimum proposed values/operations |
| --- | --- |
| Chart input | Four fixed status labels and four finite integer counts, each within the existing 12-item loaded-workspace bound. Update from the Rust draft state; label the scope as loaded items, not all project records |
| Chart handle | Update counts without replacing the host; destroy idempotently. Keep the first chart read-only; no new status-filter/action API |
| Editor input | Selected item ID, plain description text under the existing 4,000-character field contract, and read-only state for the pending item |
| Editor handle | Set the selected document/read-only state and destroy; do not expose the library's whole API or internal DOM to Rust |
| Editor callback | Item ID and plain text into the existing app edit boundary. Reject callbacks for a retired runtime or no-longer-bound selection; never apply item A's late edit to B |
| Transport | Explicit primitive values or small app DTOs via existing serde/wasm-bindgen. No raw DOM events, server secrets, function-valued data from untrusted input, or remote event circuit |

TypeScript types do not validate a runtime boundary. Check expected shapes and
bounds, retain server field validation and never silently truncate a draft.
Preserve existing semantic HTTP Save, CSRF and version checks; local editor or
chart updates must not cause requests by themselves.

On a user edit, Rust stores the text and the renderer's equal-value update must
be a no-op. Programmatic Reset/item switch must not echo another user edit.
Keep editor transactions/history inside the library: preserve ordinary typing,
selection and undo in the current document; explicitly start fresh history on
Reset/item switch for this minimal trial. Do not rebuild the view on every graph
commit or introduce a scheduler/automatic effect to synchronize the two sides.

Lifetime sequence for a later implementation:

1. Create a per-mount active token before awaiting package/WASM imports. A late
   import from a retired mount cannot create an instance in its replacement.
2. Mount synchronously into stable reserved hosts, retaining instance handles
   and Rust callback owners in the existing app runtime. If construction fails,
   clean up any partial instance and leave the native fallback usable.
3. Mutate descendants only through the package. Rust may update surrounding
   native controls/labels but must not replace a live widget's host contents.
4. On disposal, invalidate the token and JS callback entry points first, detach
   callbacks/cancel owned delayed work, destroy instances, then release Rust
   closures and finally remove hosts. A queued JS callback must check its token
   before invoking a Rust closure that may already have been freed.

The existing [wasm-bindgen closure guide](https://wasm-bindgen.github.io/wasm-bindgen/reference/passing-rust-closures-to-js.html)
documents explicit callback ownership. Reuse the project's current Closure
pattern, not leaked/forgotten callbacks. Cleanup must remain idempotent and
best-effort across individual JS destruction errors. Disposal does not undo
an already admitted HTTP Save or guarantee transactional graph/DOM rollback.

## Candidates, CSP and package cost

Chart.js supports [plain JS and selective bundler imports](https://www.chartjs.org/docs/latest/getting-started/integration.html).
For a bar chart, import/register the bar controller/element and category/linear
scales; omit the all-features auto entry, time adapters and unused plugins.
Its [update/destroy API](https://www.chartjs.org/docs/latest/developers/api.html)
supports updating an instance and releasing its resources before canvas reuse.
Keep a native text/table equivalent: [canvas accessibility is application-owned](https://www.chartjs.org/docs/latest/general/accessibility.html).

CodeMirror's project [EditorView source](https://github.com/codemirror/view/blob/main/src/editorview.ts)
supplies parent mounting, transactions, update listeners, destroy and cspNonce.
Use the modular state/view packages with explicit commands/history and keymaps
for a practical plain-text editor, without syntax grammars, completion or workers.
Its [history module](https://github.com/codemirror/commands/blob/main/src/history.ts)
supplies undo/redo; ordinary text changes should use transactions, not writes
into the editable DOM. No claim is made that every callback phase permits a
nested update; equal-value and programmatic-echo guards remain necessary.

**CSP feasibility is unresolved for the selected editor.** The current route
does not authorize CodeMirror's generated stylesheets. cspNonce can attach a
nonce to those sheets, but that does not authorize style attributes. The inspected
view also emits styles through its [attribute helper](https://github.com/codemirror/view/blob/main/src/attributes.ts).
[CSP style rules](https://developer.mozilla.org/en-US/docs/Web/HTTP/Reference/Headers/Content-Security-Policy/style-src)
distinguish stylesheet nonces, inline attributes/cssText and individual style
property assignments. Do not claim a nonce alone proves compatibility.

A later editor feasibility check may try a fresh, unpredictable response-local
stylesheet nonce and EditorView.cspNonce on this selected page only, keeping the
existing script policy and inline-attribute restrictions. Inspect actual emitted
styles/violations before enabling the editor. Do not default to unsafe-inline,
reuse a static nonce/CSRF token, fork the editor, or add a core nonce service.
If it cannot qualify under the chosen policy, retain the textarea and reconsider
the editor candidate. No CSP change is implemented or approved here.

Registry snapshot, retrieved read-only on 2026-10-20; all four declare MIT:

| Proposed package | Version observed | Published unpacked bytes | Direct runtime dependencies relevant to cost |
| --- | --- | --- | --- |
| [chart.js](https://registry.npmjs.org/chart.js/4.5.1) | 4.5.1 | 6,178,899 | @kurkle/color |
| [@codemirror/state](https://registry.npmjs.org/@codemirror%2fstate/6.7.6) | 6.7.6 | 440,367 | @marijn/find-cluster-break |
| [@codemirror/view](https://registry.npmjs.org/@codemirror%2fview/6.43.15) | 6.43.15 | 1,264,544 | state, crelt, style-mod, w3c-keyname |
| [@codemirror/commands](https://registry.npmjs.org/@codemirror%2fcommands/6.11.1) | 6.11.1 | 246,640 | state/view, @codemirror/language, @lezer/common |

These are proposed direct inputs, not the complete transitive graph or a lock.
Unpacked package sizes are not transferred bundle sizes; do not compare them
directly with the asset adapter's per-file limit. No package tarball was installed
or bundle built. A selected trial must pin resolved versions, inspect the lock
and measure added raw/gzip JS/CSS/WASM and actual lazy requests. Existing asset
limits must still pass. Package licenses/maintainer sources are selection inputs,
not a dependency security audit or compatibility guarantee.

CodeMirror's rendered guide/reference returned HTTP 403 in this research
environment; accessible project source was used instead. Source links above
track the project's main branch; the versioned registry observations do not
prove that exact published packages have been built or qualified here.

## Proposed trials and focused acceptance

**First: Chart.js loaded-item summary.** One opt-in Admin triage panel, fixed
four-status bars, app-owned TS module and stable host outside the keyed list.
Rust data changes update it; a native count table remains readable. Keep all
other pages script-free. Use the existing settings-selected CLI build/run/stop.

**Second, conditional: CodeMirror description editor.** First settle the CSP
feasibility above. Then enhance only the selected description, keeping its native
textarea/fallback links outside the widget host and hiding the textarea only
after successful mount. Rust draft/Save rules remain authoritative. This trial
is plain text, not a language IDE or rich-text/HTML editor.

| Future check | Smallest useful evidence |
| --- | --- |
| Data / update | Four bounded chart counts match the loaded Rust drafts; editor changes reach the correct item; equal-value updates do not echo or lose selection |
| Document boundary | Typing/undo/redo work within one item; Reset/switch starts the stated history boundary; pending item read-only and item B's draft are preserved |
| Lifetime | Destroy/remount once; no duplicate instance/callback, late import or old-item event; partial failure retains fallback; JS callback guards run before any freed Rust closure |
| Native fallback | JS off, package load failure or editor CSP failure leaves useful table/textarea or native item-editor links; keep keyboard labels and text alternatives |
| CSP / assets | No unexplained CSP or console error; module/CSS/WASM served by existing manifest/MIME rules; nonce, if selected, is response-local and not confused with inline-attribute authorization |
| Local / HTTP | Local edits/count updates cause no API calls; explicit Save still uses existing authorization/CSRF/version boundary and error behavior |
| Delivery / cost | Existing CLI build/run/stop and process cleanup, installed lock plus raw/gzip/lazy-request deltas; no new browser/platform matrix, benchmark framework or deployment |

This report changes documents only. Current verification is source/API and
registry inspection, scoped local-link/whitespace checks and
`cargo xtask check docs`; implementation/browser/CSP/package-cost checks above
are future acceptance, not results. Independent review passed; YT-106 is Done
at research scope. The next practical selection is the Chart.js trial; the
editor's CSP decision remains explicit rather than silently enlarging it.
