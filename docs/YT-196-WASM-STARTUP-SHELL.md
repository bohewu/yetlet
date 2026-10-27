# YT-196 — Customizable pre-Wasm startup shell

Date:2026-10-26. Status:**Done after independent whole-item acceptance and
bounded P2 re-review PASS,0issues/required followups.**

## Authority and selected minimum

The owner authorizes the reviewed [common contract](WASM-STARTUP.md), minimum
shared shell/template adoption, full-page and SSR-island use, opt-in/safety,
no accepted qualification replay and independent Reviewer before Done. New
changes noCommit/Push. YT-195 nine-path strict/builtin commit8a53b34 passes direct
and hook checks,0findings, tree580c341faca13e49570314ac44cd425f563072d4;
postcommit tree/path/clean checks match, noPush. Old receipts stay unchanged.

Canonical optional assets: assets/wasm-startup.mjs and wasm-startup.css. No crate,
Core/DOM/GPU API, dependency/lock or mandatory native inclusion. CLI browser
creation copies the assets; browser-tasks carries identical published copies.
The Lab hashes/serves the same assets explicitly for its existing optional UI.
Native defaults and unrelated applications stay outside this adoption.

createStartupShell receives explicit root/status/indicator and optional owned
fieldset/messages. Initial HTML/CSS exists before the helper or Wasm; constructor
does not start an import, probe a device, hide initial content or move focus.
begin returns an opaque ticket, refusing another pending begin. ready/fail
updates only the live ticket; ready cannot overwrite an error. cancel/dispose
invalidate tickets; one live controller owns a root. Messages use textContent;
custom HTML/brand/layout remain caller-authored trusted initial markup/CSS.
CSS custom properties and optional classes style the reduced-motion indicator.
There is no percentage, network fetching, universal deadline or auto-retry.

This is a presentation/state helper, not a Wasm resource manager. Existing
loaders own import/initialization, mounting, abort/borrow settlement and disposal.
They call ready only after mount succeeds and gate retry until retirement settles.
The full-page Tasks entry catches dynamic import/init/mount failures, checks its
live page owner after async boundaries and disposes a late initialized runtime
without mounting it. Static recovery/noJS text remains if the entry never runs.
Stylesheets link from initial HTML. Browser watch rewrites the new dynamic import
seam while retaining existing source/host/origin/CSP/private-generation policy.
The SSR Lab keeps its GPU lifetime, explicit Run, pendingPause, visibility,
invalid-world and Retry controls, delegating only startup presentation.

## Scope and focused evidence plan

Selected surfaces: canonical assets/product tests; CLI browser asset publisher,
bootstrap/HTML/watch/template and its focused test; matching browser-tasks
consumer; Lab loader/asset builder/SSR/CSS/README; shared guide/Backlog/Roadmap/
index and this record. App-owned Tasks disposal export only, no framework API.
No new sample, FPS/scenarios, environment/backend search or release expansion.

Use new helper state/token/ownership tests, one new targeted CLI emission test,
changed consumer typecheck/builds and actual narrow full-page/SSR loading proof.
The new browser flow observes pre-entry HTML+CSS, successful mount readiness,
controlled import failure/late-init retirement, customized independent roots and
SSR interaction gating; old Tasks business/benchmark/GPU/Run controls are not
replayed. Reuse existing scoped browser/process pattern, retain failures and
clean owned profiles/servers. New-delivery docs/format/whitespace and source/
asset bindings close the review. Only independent PASS may set Done.


## New verification and retained failures

Evidence root: `target/yt196-wasm-startup-shell-20261026/evidence/` (local,
ignored). `evidence-index.json` identifies the accepted cases across the raw
receipts and corrects a completion-receipt carried-forward filename typo without
rewriting any receipt. No earlier accepted business/GPU/benchmark qualification
was rerun.

| New scoped verification | Result |
|---|---|
| `node --test assets/wasm-startup.test.mjs` | 4 passed: scoped presentation, tokens/error ordering, retirement/replacement and text-only messages |
| `cargo test --offline --locked --manifest-path tools/yetlet-cli/Cargo.toml --lib browser_startup_assets_and_pre_wasm_links_are_emitted` | 1 passed,87 filtered; canonical asset emission and initial HTML/bootstrap seam |
| Tasks `npm run build` with existing exact bindgen and `npm run typecheck` | PASS; actual new disposal export/Wasm production build and TypeScript |
| Lab build-assets and native server build, `--offline --locked` | PASS; canonical JS/CSS included in five-file content hash and explicit native serving |
| Actual Firefox154/private Wayland startup checks | 8 accepted new cases across retained receipts; detailed below |

Browser proof covers initial styled HTML with the entry held, actual Tasks mount
without WebGPU, caught runtime-import503, missing-entry static recovery, late
actual init after synthetic pagehide, independently styled/scoped DOM regions
with literal custom text, existing watch initial-generation dynamic loading, and
actual SSR Lab idle/no-Wasm then shared ready/failure handoff. An injected Lab
fault tests the UI bridge; it is not a new real-device-loss qualification.
Screenshots were inspected for initial loading, independent regions and SSR ready.
All owned watch/server/WebDriver/Firefox/compositor processes, private profile and
socket were cleaned after every attempt. No environment install/config change.

Two unsuccessful observations remain in full. Attempt1 completed its first four
checks, then a WebDriver-sandbox WebAssembly observer could not hold page init.
The one focused correction bound the observer to window.WebAssembly; attempt2
accepted late-init retirement and custom regions without replaying the first four.
Attempt2 then exposed a product failure: Vite still resolves a literal import
marked vite-ignore. The watch transform now binds a nonliteral URL before import,
retaining private-generation/host/origin rules. The final attempt accepted only
watch and SSR checks; it did not replay the six prior accepted cases.

This demonstrates the startup adoption on the recorded local browser/toolchain,
not all browsers, OSes, GPU devices, network rates or performance. No production
support claim, deterministic loading deadline, progress percentage or automatic
retry is introduced. The runtime loader still owns actual async resource safety.

New-delivery documentation check: PASS,75documents/196backlog items; internal
links, Markdown structure, whitespace and dependency DAG. Changed Rust format,
JS syntax, tracked/untracked whitespace and final newlines pass. Exact canonical
copies and Lab five-file bundle `c81e5acec2ae936a679f8df4d90b850fb059067ac79d1d53b59106f9d8526c53` match source.
The26-path scope preserves1110 non-target tracked baseline hashes and
HEAD8a53b34, staged0. The one local bundle-path observation typo was corrected
at its existing source path only; preceding successful source checks were retained.
Independent acceptance was pending at this source check; final closure follows.


## Independent review and bounded lifecycle repair

Initial independent Reviewer: FAIL,oneP2, no other material blocker. All26source,
36evidence and1,110protected tracked hash bindings matched during review. Its
actual-source in-memory probe demonstrated that persisted pagehide disposed the
Tasks model/shell, with no entry rerun on cached restore. This was a new Tasks
regression; existing Lab pagehide behavior predates this item and was not expanded.

Both matching Tasks/template entries now ignore persisted pagehide and retain
the teardown listener for the next non-persisted hide. The page owner/runtime and
tab-local model survive cached navigation; actual disposal still invalidates the
owner, disables dependent controls and drops the runtime. No shared helper or
Lab change was needed. `bfcache-sequence.mjs` runs a new focused event sequence
against the actual two handler sources and real shared helper:2passed,0failed.
It checks cached hide/restore preserves runtime identity/model/usability, then
real hide retires exactly once and rejects stale readiness. This is an in-memory
lifecycle proof, not a new actual-browser BFCache qualification. TypeScript
recheck passes for the changed branch. Accepted browser late-init retirement,
other startup cases, helper/CLI tests and Rust/Wasm builds are reused.
The first review and its evidence remain unchanged; the final re-review follows.


## Final acceptance

Independent re-review PASS,0issues/required followups: the firstP2 is closed.
The reviewer accepted the original whole-item review plus the narrow repaired
lifecycle delta. All26source/47evidence bindings,36unchanged original evidence
bindings and1,110protected tracked hashes matched. Native role/model/effort
telemetry matches reviewer/gpt-6-astra/high for both reviews; observed shared
resolver decisions satisfy the selected profile. No accepted tests were replayed.

Actual changed paths (26):

- Shared assets: `assets/wasm-startup.mjs`, `assets/wasm-startup.css`,
  `assets/wasm-startup.test.mjs`.
- Tasks: `examples/browser-tasks/README.md`, `src/lib.rs`,
  `frontend/app.ts`, `frontend/index.html`, `frontend/serve.mjs`,
  `frontend/wasm-startup.mjs`, `frontend/wasm-startup.css` (paths relative to
  `examples/browser-tasks/` after the README).
- CLI: `tools/yetlet-cli/src/browser.rs`, `templates/browser/README.md`,
  `templates/browser/app.ts`, `templates/browser/index.html`,
  `templates/browser/lib.rs`, `templates/browser/serve.mjs` (template paths
  relative to `tools/yetlet-cli/`).
- Lab: `examples/physics-lab/README.md`, `loader.js`, `style.css`,
  `src/bin/build-assets.rs`, `src/server.rs` (relative to that example).
- Documentation: `docs/BACKLOG.md`, `docs/README.md`, `docs/ROADMAP.md`,
  `docs/WASM-STARTUP.md` and this record.

Only completion metadata follows the accepted source binding. Final documentation,
whitespace and source/evidence preservation checks are retained in local closure
receipts. YT-196 changes remain unstaged/uncommitted; noPush. The earlier YT-195
commit is separate. Outside YT-096/020 release decisions, no other Todo item
remains; more Lab scenarios/FPS are still owner feedback, not an authorized
implementation or an expanded benchmark/support claim.


## Actual browser Back/Forward cache followup

On2026-10-26 the owner explicitly requested real BFCache proof and then supplied
additional budget after the first two cache misses. This is a bounded YT-196
verification followup, not a new implementation or qualification matrix. The
original independent acceptance, in-memory lifecycle proof and failure receipts
above remain historical and unchanged. Actual browser observations and
independent evidence review PASS,0issues/required followups.

Source: committed25f5018567f7eca3893550ec7a10a1096cc1bc5d. Only the current
frontend JS/CSS was rebuilt through Vite; the four existing generated Wasm/glue/
type files retained exact hashes. No Rust build, old accepted business/startup/
GPU/benchmark qualification or source mutation was repeated. The CLI template
and Tasks entry remain identical. Original1,136tracked hashes matched before
these documentation updates.

### Environment and observation method

Actual Firefox154.0 on Linux/private packaged Weston,1200x864content viewport,
a new disposable profile, no forced BFCache/WebGPU preference and no installation
or permanent configuration change. The static fixture used an ephemeral127.0.0.1
listener and Cache-Control private,max-age60. Original built JS/CSS/Wasm bytes
were served unchanged. Fixture HTML added only a same-origin external lifecycle
observer and navigation link under the unchanged HTML CSP; no inline script/eval
or synthetic pagehide/pageshow. DOM properties/attributes and native W3C pointer
input supplied observation/interaction; no classic executeScript in Tasks proof.

Flow: native page before -> Tasks -> add unique task -> native page after ->
actual Back to Tasks -> complete that task -> Back to preceding native page ->
actual Forward to Tasks -> add a second unique task.

| Scoped check | Actual result |
|---|---|
| Native control with pointer actions | Real cached restore,matching document token and persisted pageshow |
| Tasks Back | pageshow.persisted=true,unchanged document instance/timeOrigin/DOM row,ready shell and enabled controls; original added task retained; checkbox changes Rust model and active count |
| Tasks Forward | Second persisted pageshow,same instance/timeOrigin/row and completed task; one submit adds exactly one task |
| Observable UI and errors | Counts4tasks/3active ->4tasks/2active ->5tasks/3active; expected title/content, no overlay, zero page-observed runtime/rejection/CSP errors or failed asset responses; screenshots inspected |

This establishes actual ready-page cache restoration on both history directions,
not merely HTTP cache reuse or a synthetic transition. It does not establish
startup-pending cache behavior, eviction/long retention, mobile/other browsers,
Lab GPU resource restoration or an obligation for a browser to cache every page.
Browser cache misses still reload and reset this intentionally tab-local model.

### Retained misses and diagnosed test interference

Five browser allocations were bounded and fully cleaned. The first two observed
fresh documents on Back; the second captured pagehide.persisted=false and a
native HTML miss. After explicit additional budget, a page-owned observer without
classic executeScript still saw a native miss, so an executeScript-only cause
was not established. The planned no-op diagnostic did not run because its
preceding native control failed.

A native-only diagnostic allocation enabled process-local engine logging. It
recorded UNLOAD_LISTENER on the high-level Element Click navigation path and
explicitly rejected that native page from BFCache; another native page was
accepted with persisted=1. No related own-profile user override was recorded.
The final allocation replaced the high-level click seam with native W3C pointer
input. Both the native control and actual Tasks Back/Forward then restored from
BFCache. The observed evidence supports navigation-tool interference on this
recorded path; it is not a Firefox-wide incompatibility or an application defect.
No application patch or cache-forcing preference was needed.

[Mozilla's navigation implementation](https://github.com/mozilla-firefox/firefox/blob/main/remote/marionette/navigate.sys.mjs)
and [script evaluation](https://github.com/mozilla-firefox/firefox/blob/main/remote/marionette/evaluate.sys.mjs)
show unload handling in automation. These current upstream references support
the diagnostic interpretation; they do not attest the exact installed binary's
source. The local engine logs and observed native-action restores are primary
receipts for this environment.

Local retained artifacts outside the checkout:
`/tmp/yetlet-yt196-bfcache-20261026-yqrevktt/`. The canonical focused command is
`python3 <evidence-directory>/bfcache.py`; result-actions.json and
validation-final-summary.json identify accepted observations, while all earlier
FAIL/diagnostic receipts remain intact. Owned server/profile/Firefox/WebDriver/
Weston processes and socket were removed after every allocation. No new sample,
Lab scene/FPS, crate/runtime change, Commit or Push.


Independent evidence Reviewer verified both document bindings,41evidence hashes,
1,134protected tracked hashes, all four generated files and recorded built assets.
The actual ready-page Back/Forward proof is accepted at its recorded scope.
Native role/model/effort telemetry matches reviewer/gpt-6-astra/high. Only this
completion wording and the final documentation/whitespace check follow review;
no new browser run or accepted verification replay. These two documentation
changes remain unstaged/uncommitted, with noPush.
