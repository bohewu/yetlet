# YT-240 — Todo app-owned Rust CSS qualification

**Date:** 2026-11-08. **Status:** **Done at bounded Todo/Linux sample scope
after independent Reviewer PASS.** Job `wc_job_7bfgn4kA_CNHLN22`, terminal
completed, exit 0, zero issues/followups. Public release, Windows qualification
and the separate Style Showcase remain outside this acceptance.

## Scope and source contracts

The existing `todo` sample remains in the twelve-entry catalog, with its existing
packages, HTTP routes, keyed local runtime, native controls, Search Island and
typed remote action boundary. No Showcase or new stable framework package is
added. This follows the accepted [YT-237 contract](YT-237-STYLE-REGISTRATION-CONTRACT.md),
[YT-239 integration](YT-239-STYLE-INTEGRATION.md) and ADRs
[0003](adr/0003-renderers-and-loading.md),
[0005](adr/0005-opt-in-capability-modules.md) and
[0007](adr/0007-bounded-keyed-dom-and-local-graph.md).

The affected Todo list, shell, headings, layout and interactive containers use
`yetlet::view::tags` from the YT-234/235 checkout source. These constructors lower
to the existing View tree; they are not in published SDK alpha.6. Search controls
and the separate native draft implementation retain their existing authoring.
`ul` still has class exactly `todo-list`; each keyed `li` retains `data-id`,
`data-completed`, and exactly `is-open` or `is-complete`. Event IDs are stable.

Shared target-neutral `todo-ui/src/style_names.rs` constants connect the Views
to a private, std-only, host Cargo generator under `todo-server/stylegen/`.
The generator reuses the finite YT-239 builder without expanding its vocabulary.
The root `styles::stylesheet()` registers two app modules, `layout` and `tasks`.
`is-complete` is registered before any initially open task is toggled. The
complete output appends trusted `plain.css` for syntax outside the finite DSL;
there is no View execution, scanning, global registry or runtime style injection.

The compact task workspace uses a Flex toolbar, Grid panels, shared spacing and
color variables, system typography and visible keyboard focus. Native headings,
list semantics, labels and live status remain HTML. The CSS has one responsive
threshold: below 768px one column, at or above 768px two columns. Verification
uses 390px and 1280px viewport states. Completed tasks gain green text/background
and line-through, so their state is not communicated by color alone.

## Copyable native and browser commands

Run from the repository root:

```sh
cargo xtask build todo-css
cargo run --locked -p yetlet-todo-server
```

Open `http://127.0.0.1:3000/`. The narrow `todo-css` builder needs only Rust/Cargo;
it invokes a frozen host-target generator and writes versioned CSS plus the
existing favicon. It never probes wasm-bindgen or builds WASM. Existing browser
output is retained; on a clean output tree `/app` remains an explicit 503 until
the browser build runs. Missing optional CSS output leaves ordinary SSR usable.

For the existing browser and Search enhancement:

```sh
cargo xtask build browser
cargo build --locked -p yetlet-todo-server
cargo run --locked -p yetlet-todo-server
```

Open `/app` or `/search`. Browser tool preflight still precedes generation;
generation completes before any WASM build, and failed generation publishes no
new CSS or native record. The existing builder hashes the completed asset graph
into `dist/assets/<digest>/`, writes `manifest.json` and `app.html`, then selects
the same CSS URL/bytes for native SSR through private `dist/todo-css.json`.
Document order remains stylesheet, favicon, then browser module, with one CSS
link. A standalone `todo-css` rebuild can independently select a CSS-only digest
for native SSR while leaving the previous browser graph intact.

Native startup validates the record's fixed lowercase digest URL and the full
SHA-256 of the exact CSS bytes, then serves that selected CSS from memory. Restart
the native server after rebuilding CSS. The root decorates the existing
`Page::new` response with app-owned external links and retains `DEFAULT_CSP`.
The CSS route returns `text/css; charset=utf-8`, nosniff, no-store and exact
`DEFAULT_CSP`. The Browser/Search routes retain their existing optional WASM
policy; no unsafe-inline, unsafe-eval or remote asset is added. Only the existing
`/assets` tree is static public content; private records and generator sources
are not routed. Requests never invoke a compiler or read a CSS record.

The repository builder remains its existing local, sequential build path, not
the CLI's protected pending-record publisher or a deployment transaction. The
new record is startup input; no crash-safe multi-file acceptance, hostile
filesystem isolation, automatic watcher or production packaging claim is made.

## Focused results and retained evidence

Evidence root: `artifacts/verification/yt240-todo-style-20261108/`. Initial source
hashes and snapshots remain in `initial-source-state.json` and `initial-sources/`.
All 61 pre-existing dirty source files are preserved. Root `Cargo.lock` changes
only the Todo server's SHA-256 dependency edge; it resolves an already locked
version. The xtask help change documents the narrow native-only command.

Existing native/WASM root targets and `xtask/target` are reused because the
existing smoke/build consumers address them directly. No global Cargo target
redirection, tool installation, cache deletion, permission change or new
validation framework is introduced. The prepared isolated cache remains intact.

| Check | Result and retained evidence |
|---|---|
| `cargo test --locked --offline -p yetlet-todo-ui -p yetlet-todo-server --all-targets` | PASS, 26 tests; `targeted-tests-final.log` |
| `cargo test --locked --offline --manifest-path xtask/Cargo.toml --test builders` | PASS, 7 existing builder regression tests; `xtask-builders-tests.log` |
| `cargo xtask build todo-css` | PASS, native-only generator; `native-css-baseline.log` and `native-css-corrected.log` |
| CSS source edit, invalid value, correction | PASS: changed CSS digest, expected exit 1 on negative spacing, prior accepted bytes/record preserved, original source and output restored; `style-edit-recovery.json` and original failure log |
| Native executable in CSS-only and absent-output working directories | PASS: no browser files required, useful no-JS SSR, `/app` 503, private paths 404, CSS digest/MIME/nosniff/no-store/CSP; `native-http.json` |
| `cargo build --locked --offline -p yetlet-todo-server` | PASS; `native-build-final.log` |
| `cargo xtask build browser` | PASS; `browser-build-final.log` |
| `cargo xtask smoke browser-chrome` | PASS; `smoke-chrome-final/` and command log |
| `cargo xtask smoke browser-firefox --profile debug` | PASS within supported viewport; `smoke-firefox-final/` and command log |
| `cargo xtask check docs` | PASS; `check-docs.log` |
| Root, xtask and generator formatting; `git diff --check` | PASS; `fmt-check.log`, `diff-check.log` |
| `cargo xtask check samples` | PASS after the parent restored the exact catalog command in Todo README; `check-samples-parent-corrected.log`. Original exit-1 check retained as `check-samples.log` |

Both browser journeys retain byte-identical SSR/list HTML, keyed node identity,
local updates without remote requests, typed remote HTTP, lifecycle disposal,
lazy Search and native draft behavior. Both check one stylesheet link/request,
Flex display and the pre-registered completed class's computed color, background
and line-through. Chrome verifies one/two computed Grid columns at 390/1280,
no horizontal overflow and native content while page scripts are disabled.
Existing Chrome network/runtime/log checks observe no resource/CSP/console errors
on those initial/native/local and successful remote journeys. The deliberately
invalid remote command still produces its expected typed 422. Firefox's existing
WebDriver path does not collect full browser console/CSP logs.

`asset-identity.json` independently recomputes the existing browser asset digest,
checks full CSS SHA-256, shared native/browser URL and stable link order. The
accepted CSS, browser/native records and browser HTML are retained beside it.
Final screenshots were actually inspected: Chrome native/browser at 390/1280
and Firefox native/browser at 1280. The affected task panels wrap and align as
intended. No formal accessibility audit, spoken screen-reader or physical mobile
device result is claimed.

Initial failures remain retained: a CSS route state-type compile error, then a
Chrome native favicon 404. Two bounded product repairs fix router state and add
the existing favicon to native output/links. One diagnostic rerun preserves the
exact 404 evidence. Initial Firefox default selection used the older release
binary; its command was corrected to the newly built debug profile. Firefox then
exposed an unsupported window width; the existing assertion records that limit
instead of claiming a 390px result. The deliberate generator error is a successful
failure-path exercise, not an unresolved product failure.

The existing YT-239 accepted native/browser watch and recovery evidence is reused;
no Todo watcher is invented or rerun. This task adds the Todo-specific bounded
build/edit/failure/correction proof. Original xtask smoke evidence is retained in
its existing location and copied into this task's evidence root. Browser/server
exit receipts confirm listeners closed, streams closed and owned process trees
reaped; browser profiles are removed. The two additional native HTTP server PIDs
are separately waited and observed absent.

## Platform matrix and acceptance boundary

| Platform / mode | Observed scope |
|---|---|
| Linux x86_64, Rust 1.99.0 | Native host generation/build/run, CSS-only and absent-output SSR PASS |
| Linux Chrome, debug WASM | 390px and 1280px native/browser layout, computed conditional style, CSP/resource checks and existing full Todo journey PASS |
| Linux Firefox, debug WASM | 1280px layout, computed conditional style and existing full Todo journey PASS; requested 390px yields actual 500px, so narrow layout is unverified on this engine |
| Windows, macOS, Safari, physical mobile | Not run; no platform qualification claim |
| Release binaries / production deployment | Not qualified by this debug sample run |

Post-prototype recommendation: keep the finite helper private to app/tooling,
consistent with YT-238 SIMPLIFY and YT-239. This Todo consumer does not justify an
optional public `yetlet-style` crate, an expanded DSL or automatic promotion.
Keep ordinary CSS as the default. The first Executor Job completed with a
partial result after two product repairs; its sole remaining material gap was
the exact catalog command omitted from the updated Todo README. The parent then
performed **bounded current-session role emulation** limited to restoring that
existing one-line command. The focused `cargo xtask check samples` rerun passed
for all twelve entries. Both the original failure and corrected log are retained;
this parent correction is not evidence of an independently dispatched Executor
role/model. No additional runtime or CSS changes followed the browser evidence.

## Independent review and completion

The independent read-only Reviewer returned **PASS** in Job
`wc_job_7bfgn4kA_CNHLN22`, terminal completed, exit 0, with zero issues,
required followups and optional notes. The exact saved role `gpt-6-astra` and
the shared reasoning resolver's `high` matched observed process telemetry.
This was a separate role-directed ad hoc review, not current-session self-review
or native managed-role/formal assurance. All **81 frozen source hashes** matched
during review, including the 61 preserved prior dirty files and 20 YT-240
review targets. No new source implementation was introduced after review;
subsequent edits update completion records only.

`artifacts/verification/yt240-todo-style-20261108/review-input.json`,
`reviewed-inputs/`, `reviewer-result.json` and `review-acceptance.json`
preserve the scope, source identities and independent result. The initial
Executor Job ended partial under its fixed repair budget; its source work and
the parent's narrowly corrected README/catalog command were both included in
the reviewed input. The original sample check failure remains in its evidence.

The accepted post-prototype choice is to **retain the finite Rust CSS builder
application/tooling-local** under SIMPLIFY, rather than add a public style crate.
Chrome 390/1280px, Firefox desktop, native no-JS SSR, CSS/assets, recovery and
cleanup are bounded Linux evidence only. Firefox 390px, Windows/macOS/Safari,
physical mobile and production deployment remain unverified. The owner intends
to consider a dedicated Style Showcase only **after this Todo work**; it is not
part of this delivery. Git delivery and public release remain unapproved.
