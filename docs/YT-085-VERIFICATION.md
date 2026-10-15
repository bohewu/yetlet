# YT-085 — Rust smoke-tool implementation and accepted M12 port

**Task:** YT-085. **Current state (2026-10-15):** Done; final independent Reviewer PASS with zero issues/followups (`target/m12/yt085/reviewer-board-retention-20261015/result.json`). All mapped
source implementations are present; [current Rust runtime evidence](YT-085-RUNTIME-VERIFICATION.md)
records 25 positive suites and one nonzero Chrome GPU-unavailable observation,
with Windows persistent-service and external Redis fixture limits explicit.
Historical partial implementation and sandbox-denied checkpoints below are
superseded by that runtime record, not instructions to rerun them. YT-086 has
not started; the repository toolchain is not yet Python-free.

Checkout: `$HOME/repos/yetlet`, branch `master`, HEAD
`7cfac613f908c5c19fa581438288386809870b56`, same worktree. Existing YT-082–YT-084
inputs were retained. YT-084's independent reviewer receipt at
`target/m12/yt084/reviewer-adoption-fix/result.json` is PASS; no product rebuild
was needed to reconsider that acceptance.

Authority is the owner's YT-085-only implementation assignment and the approved
[M12 plan](M12-RUST-TOOLCHAIN-PLAN.md). The original Python sources in the
[migration ledger](M12-PYTHON-MIGRATION.md) define the scenario contract. Tooling
and its direct tests are the requested product. No generic testing framework,
public CLI, additional agent, installation, global default, worktree, cloud,
driver, desktop, hosted workflow, release, staging, commit or push was performed.

The original terminal implementation and its exhausted counters are retained
below as history. The [remaining-eight checkpoint](#remaining-eight-implementation-checkpoint)
supersedes its implementation-gap list. Parent host runtime receipts cover only
the earlier implemented commands at their exact prior sources.

## Initial terminal checkpoint: actual changes

- `xtask/src/smoke/{mod,http,native}.rs`: ten native implementations, bounded
  loopback readiness, explicit HTTP/cookies/forms/JSON, unique evidence directories
  and disposable application state.
- `xtask/src/smoke/{browser,websocket,png,static_server}.rs`: CDP event recording,
  masked WebSocket framing/handshake, Firefox WebDriver and an initial BiDi
  request/event helper, W3C keys, known PNG filters/point reads and an owned static
  fixture server. BiDi and PNG helpers have no completed scene consumer yet.
- `xtask/src/smoke/{todo,keyed,dashboard,forms,proof,fixtures}.rs`: implementations
  for eight browser commands and two browser fixture commands. Runtime parity is
  not established; remaining source assertions and reporting gaps are below.
- `xtask/src/process.rs` and `xtask/src/process/{unix,windows}.rs`: persistent
  `Service` ownership shares the existing process-group/Job foundation; graceful
  stop request, bounded escalation, reap, log closure and failed-readiness error
  preservation. Existing one-shot process behavior and late-cancellation checks
  remain. Windows execution of the extension is Not run.
- `xtask/src/{lib,main}.rs`: internal dispatch for implemented commands only.
  Unimplemented Board/scene names remain usage errors. No successful stubs exist.
- `xtask/tests/{smoke_browser_harness,smoke_modular_shop,smoke_protocol}.rs`:
  original five direct acceptance cases, two subprocess fixture entry points and
  five focused protocol/negative tests. No new ignored acceptance test was added.
- Isolated `xtask/Cargo.toml` and `xtask/Cargo.lock`: cached `sha1`, `getrandom`,
  `flate2` and `tiny_http` for actual WebSocket, PNG and static-server needs.
- This document, the migration ledger, backlog ready queue and roadmap record
  the partial state. Live README/catalog/CI smoke invocations remain unchanged.

Root `Cargo.toml`, `Cargo.lock` and `rust-toolchain.toml` hashes still equal the
plan's baseline. No framework runtime API or root dependency graph changed. All
30 original YT-085 Python files remain byte-identical to the ledger's baseline.
The separate selected-input snapshot also retains the current, previously
modified YT-084 workflow input. No M11 receipt, port baseline or accepted
YT-082–YT-084 result was overwritten.

## Original assertions mapped to implementation

Paths below are relative to `xtask/src/smoke/`. Presence means implementation,
not a runtime or independent-review PASS. All ten actual native suites remain
Not run to completion.

| Original source | Concrete Rust functions | Assertions represented |
|---|---|---|
| `scripts/smoke_hello_world.py` | `native.rs::hello` | SSR greeting/main, escaped named path, exact CSP/content type, no scripts/Wasm/session/request provider, 404 and POST 405/Allow |
| `scripts/smoke_rest_api.py` | `native.rs::rest` | Health JSON, empty list/version/cache/correlation, typed create/read/query/stale update/update/delete/not-found, foreign Origin 403 and media 415, absent UI routes |
| `scripts/smoke_cart.py` | `native.rs::cart` | Health isolation, SSR/cookie flags/CSRF, add and refresh, invalid-token no mutation, clear rotation/old ID retirement/end removal, absent app route |
| `scripts/smoke_middleware_lab.py` | `native.rs::middleware` | Typed state/Tower header, exact unwind order, route/group/global scope, short circuit, 404/405 and streamed body; broken-stream coverage remains in the existing sample tests, not a new HTTP assertion |
| `scripts/smoke_forms_validation.py` | `native.rs::{forms,document}` | English document root/labels/no scripts/cookies, escaped 422 fields/descriptions/autofocus, both invalid-field subcases, no-store, 303 stateless success and empty fresh form |
| `scripts/smoke_file_server.py` | `native.rs::{files,multipart}` | Static GET/HEAD/Range, token and Origin, repeated opaque downloads/HEAD, independent resource IDs, traversal/name/type/size/method errors, actual incomplete-body socket/file cleanup and graceful storage removal |
| `scripts/smoke_sqlite_crud.py` | `native.rs::sqlite` | Schema health/create, Origin/stale policy, update, three owned process lifetimes on one disposable DB, two concurrent version-2 updates with 200/409, versioned delete and second restart persistence |
| `scripts/smoke_sse_notifications.py` | `native.rs::{sse,Events::open,Events::item}` | Stream headers/heartbeat, subscriber counts, exact one/two/three IDs and JSON, disconnect, four/five replay after ID 3, malformed/future cursor and browser-Origin errors |
| `scripts/smoke_auth_notes.py` | `native.rs::{auth,login}` | Synthetic bad login/no secret HTML/log leakage, local cookie policy, login/logout rotation, old CSRF/ID retirement, escaped private note, Bob list/read/update/delete IDOR, rejected mutation preserving private body |
| `scripts/smoke_modular_shop.py` | `native.rs::{shop,shop_login}` | Login/cart/revisioned checkout/order SSR, SQLite restart persistence, Bob cross-user 403, Orders-disabled routes/navigation/checkout; rollback/idempotency additionally remain existing sample tests |
| `scripts/smoke_browser.py` | `todo.rs::{run,search,lifecycle,exact_post,snapshot}` | SSR/DOM list equality and keyed identity; two local transitions/class/text/no HTTP; semantic POST payload/count and authoritative snapshot; deferred Search assets, input/heading identity, mixed-case/empty/reset, invalid schema SSR fallback; visible 422; queued/disposed listeners, remount and gated late HTTP completion |
| `scripts/smoke_firefox.py` | `todo.rs::{run,lifecycle,search,no_api}` and `browser.rs::Firefox` | Firefox-specific ordering, resource counts/revisions, native W3C clicks/input/Ctrl+A/Backspace, queued and late-response retirement, one remounted listener, typed success/422 and lazy Search reset |
| `scripts/smoke_keyed_bindings.py` | `keyed.rs::{run,mismatch,invoke,removed_inputs,attr,order}` | Identity/key/shape/slot mismatches retain DOM and inert listeners; disposed-handle isolation; label text identity; controlled input/focus/selection and both reversals; conditional slots; append/focused removal/detached listeners; exact batch/reentrant counts; subscription/reducer/commit/graph disposal; no network/circuit/errors |
| `scripts/smoke_keyed_bindings_firefox.py` | `keyed.rs::{firefox_inputs,removed_inputs}` | W3C controlled input, 8–13 selection and keyed reverse identity/focus/value, focused removal one commit, detached input/button inertness, reset/dispose and no API resources |
| `scripts/smoke_multi_island_dashboard.py` | `dashboard.rs::{run,fallback}` | Both deferred loader/bundle request arrays, counter-only activation, sibling DOM/focus, late timer/dispose/remount, invalid filter failure/recovery, joint disposal; missing/malformed manifest strict script-free SSR and no bundle requests; initial HTTP assertion omissions below |
| `scripts/smoke_multi_island_dashboard_firefox.py` | `dashboard.rs::{run,fallback}` | Deferred separate assets, W3C counter actions, local filter identity/focus, late callback/dispose/remount, failed filter/recovery, both manifest fallbacks |
| `scripts/smoke_forms_accessibility_firefox.py` | `forms.rs::{run,exercise,english,labels,focus}` and `browser.rs::Firefox::{keys,replace_text}` | Every original stage/check label: initial semantics, Tab name/email/submit, invalid POST messages/relationships/first focus, Unicode name and email-first focus, native success/303/refresh/no cookie/storage/fresh empty form; sanitized report |
| `examples/form-css-proof/verify_browser.py` | `proof.rs::{run,navigate,click,check,screenshot}` | No-JS native CSP; actual Bootstrap/application styles and labels; 1280/390 screenshots/overflow; external preview/identity/selection/one listener/text sink; synthetic composition labels; timer/listener disposal/reset/live checkbox/no POST; native PRG and no external/resource/JS errors; source-report gaps below |
| `qualification/yt066b/tests/cookie_browser.py` | `fixtures.rs::{run,navigate,cookies}` (`three=false`) | JS-disabled CDP DOM observation, one HttpOnly cookie, all 180 old-response iterations, replacement mismatch, unauthorized/explicit login/logout/old-180 sequence and cleanup |
| `qualification/yt066b3/browser.py` | `fixtures.rs::{run,navigate,cookies}` (`three=true`) | All A/B/C create/read/rotate/replay/retired/create/recovery/destroy/read categories, exact old-cookie replay and fixed count; browser ownership cleanup. Caller transition callback remains YT-086 work |

The five original direct cases map to functions of the **same names** in
`xtask/tests/smoke_browser_harness.rs` (three) and
`xtask/tests/smoke_modular_shop.rs` (two). Tests retain debug/release selection,
owned descendants and ESRCH after reap, injected first ENOTEMPTY removal, preserved
readiness error identity, stream closure, ordinary termination and ignored-TERM
four-second escalation to SIGKILL. The latter two call production
`Service::readiness`; these are real native child processes, not Python mocks.

`smoke_protocol.rs` checks the known WebSocket accept digest and rejection,
masked golden bytes and 125/126/65535/65536 lengths, ping frame reading,
truncated/reserved/oversized negatives, fixed RGB filters 0–4/RGBA and point
bounds, profile/URL/unsupported-command rejection, hidden-field escaping and
contained static paths. Synthetic PNG fixtures test decoding only; they provide
no GPU or screenshot acceptance.

## Validation and receipts

Fresh receipt directory: `target/m12/yt085/implementation-DaoDRt/`.

| Exact command | Result and receipt |
|---|---|
| `cargo build --offline --locked -p yetlet-hello-world -p yetlet-rest-api -p yetlet-cart-server -p yetlet-middleware-lab -p yetlet-forms-validation -p yetlet-file-server -p yetlet-sqlite-crud -p yetlet-sse-notifications -p yetlet-auth-notes -p yetlet-modular-shop` | Exit 0; `native-build.log`. This compiles all ten native binaries; it is not a smoke PASS. |
| `cargo test --offline --locked --manifest-path xtask/Cargo.toml` | Exit 0; 43 passed; `xtask-tests.log`. Two pre-existing YT-083 subprocess fixture entry points are ignored in top-level enumeration and explicitly invoked by their existing tests. No new suite is hidden by an ignore/skip. |
| `cargo test --offline --locked -p yetlet-modular-shop -p yetlet-sqlite-crud -p yetlet-sse-notifications -p yetlet-file-server -p yetlet-auth-notes` | Exit 0; 54 existing tests passed; `sample-tests.log`. Includes existing transaction/idempotency/auth/file/SQLite/SSE assertions, not real server/browser integration. |
| `cargo clippy --offline --locked --manifest-path xtask/Cargo.toml --all-targets -- -D warnings` | Exit 0; `clippy-correction4.log`. |
| `cargo fmt --manifest-path xtask/Cargo.toml --check` | Exit 0; `fmt.log`, `fmt.exit`. |
| `cargo xtask check docs` | Exit 0 before this report; final report check is recorded in `docs-final.log`. |
| `xtask/target/debug/yetlet-xtask smoke hello-world` | Native exit 1 before server spawn; `native-hello-world.log`, `hello-native-result.json`; original run `target/m12/yt085/hello-world-dMx0cR/result.json`. Socket operation returned `PermissionDenied`. |
| `xtask/target/debug/yetlet-xtask smoke browser-chrome --profile debug` | Native exit 1 before server/browser spawn; `browser-acceptance-blocked.log`, `browser-native-result.json`; original run `target/m12/yt085/browser-chrome-AEFAN4/result.json`. Exact diagnostic: `loopback socket unavailable: PermissionDenied`. |

Commands use task-local `$HOME/.cargo/bin` on PATH. No Python or Node was
invoked by these implementations or validation commands. This is a limited
source/execution observation, not YT-086's controlled-PATH invocation-graph proof.
No actual Chrome/Firefox/GPU acceptance batch could start. No prior M11 image,
build, source receipt or browser acceptance substitutes for this missing gate.

`yt085-python-source-sha256.txt` freezes all 30 original YT-085 scripts;
`source-recheck.log` records their final recheck. `root-graph-sha256.txt`,
`original-source-sha256.txt`, `head.txt` and `git-status.txt` retain selected inputs
and checkout identity. `implementation-source-sha256.txt` records this Rust port.

## Initial terminal checkpoint: exact remaining gaps

Eight command implementations remain absent; command parsing does not imply
completion:

| Source / mapped command | Entire uncovered scenario family |
|---|---|
| Board `verify_local.py` / `board-chrome` | Auth/viewer/project denial, persisted browser create/edit/conflict draft and intentional 409 diagnostics, SSE pause/reconnect/status, responsive screenshots and six-record array, restart/retired session, stopped-DB 1000-item/history seeding, three diagnostic 25-client rounds/ten SSE streams/raw samples/metrics/EOF and private cleanup |
| Board `verify_firefox.py` / `board-firefox` | Actual W3C Tab/Unicode/native forms, escaped text and retained validation draft, persisted update, unchanged SSE suppression, message-before-count announcement ordering, pause and owned cleanup |
| Scene `verify_browser.py` / `scene-chrome` | Actual hardware adapter/120 centers/black margin, no-JS search/pagination/detail, explicit profile separation, all injected absent/null/device/validation/Wasm/manifest cases, held real-device retirement/replacement/sibling and cleanup |
| Scene `verify_firefox.py` / `scene-firefox` | Actual temporary WebGPU profile/real pixels/black interior/repeated activation/dispose/remount, all fault arrays and held-device destruction, missing-Wasm and invalid-manifest fallback |
| Scene `verify_lifecycle.py` / `scene-lifecycle-firefox` | Actual resize/DPR/coalescing, zero/detached/offscreen/background-tab suspension, real controlled device loss/recovery, generation/resource counters and held acquisition/validation completion destruction/pixels |
| Scene `verify_picking.py` / `scene-picking-firefox` | Stable selection, stale/generation rejection, portrait viewport input/coordinate mapping, keyboard/semantic HTML alternatives and actual pixel assertions |
| Scene `verify_candidate.py` / `scene-candidate-firefox` | Source-bound extracted candidate's real geometry/pixels, ownership/loss/recovery/lifetime and synthetic input scenarios |
| Scene `verify_support.py` / `scene-support-firefox` | Distinct fresh default-profile observation without WebGPU override, honest ready/unavailable result, actual pixels if ready, semantic counter/selection/detail fallback and cleanup |

Further gaps in the implemented portion remain explicit:

- All native/browser/fixture runtime assertion sequences are unverified. Neither
  the static static-server transport nor live CDP/WebDriver/BiDi event/error
  behavior has runtime parity evidence. PNG tests do not replace live pixels.
- Dashboard's original initial HTTP heading, both loader-markup and explicit
  wasm-CSP/no-inline assertions are not all reproduced by the current initial
  Rust HTTP check. Do not accept that suite as full assertion parity yet.
- CSS-proof's original per-run `head` and recursive `source_sha256` report fields
  are not yet generated in its suite report. Aggregate source hashes below do
  not establish that report-schema parity. Original native PASS summaries are
  currently condensed; reporting compatibility still needs review.
- BiDi has an initial request/event helper but no ported portrait-input or scene
  callers. Windows persistent-service graceful termination/cleanup is Not run.
- No real Redis/cloud provisioning, duration load/performance benchmark, physical
  IME, AT, mobile, public CLI, remaining CI switch or YT-086 caller implementation
  was attempted. The existing qualification callers still invoke Python.

## Initial terminal checkpoint: bounds, cleanup and next action

`repair_attempts_used=4/4`, `operational_retries_used=2/2`. Product correction
rounds addressed initial type/lifetime compilation, a missing exit accessor,
a fixture result type, then strict-Clippy findings. Materiality Gate was read
before each correction. Final code passes strict Clippy and full xtask tests;
implementation edits stopped at the bound. The offline lock-generation command
initially encountered the existing yanked `xml-rs=0.8.19`; preserving the existing
lock entries while resolving added cached dependencies succeeded on one bounded
operational correction. One bounded second runtime attempt reproduced socket
permission denial, ending runtime probes. A zsh exit-capture assignment used its
read-only `status` variable after that command; the command was not rerun. Its
own native `result.json` supplies the exit-1 evidence.

Last unresolved failure signature: `operational_failure: loopback socket
unavailable: PermissionDenied`. Both runtime attempts stopped before spawning a
server/browser/profile or opening a listener. Test-owned process groups were
terminated/escalated/reaped; fixture stream closure and profile removal passed.
All foreground build/test sessions completed. No suite-owned server/browser or
window remains. No desktop or host-network settings were changed to work around
the sandbox.

The next unblockable backlog work is **finishing YT-085**, with a new explicitly
bounded continuation after these exhausted counters are reconciled, in an
execution context that permits its owned loopback listeners. Complete the exact
missing implementations/assertions/report fields, then the full native smokes
and one justified Chrome/Firefox batch, preserve failure evidence, and obtain
parent independent Reviewer PASS. YT-086 remains dependency-blocked. No open
framework architecture decision is introduced by this partial port.

## Remaining-eight implementation checkpoint

This is one newly authorized implementation task inside YT-085, in the same
`master` checkout and HEAD. It is the first implementation of the eight missing
commands, not another attempt at the terminal Job's denied runtime probes. The
owner authorizes the migrated repository toolchain and its direct product tests;
no broader validation framework follows. The original Python scripts were each
read in full and remain unchanged. Evidence is isolated under
`target/m12/yt085/remaining-eight-R5HjpW/`; no M11 receipt was rewritten.

The owner reports native hello-world plus the nine entries in
`target/m12/yt085/parent-native-runtime/result.json` and the eight browser entries
in `target/m12/yt085/parent-browser-first/result.json` all exited 0 through
WebCodex's direct host process tools, with cleanup. Both aggregate receipts were
read in this task. These are positive results for those exact earlier sources.
They do not cover the new eight commands, the added Dashboard/proof assertions,
or changed shared helpers. No previous native/browser command was rerun here.
The prior workspace-write socket denial remains terminal historical evidence;
this task made no socket, browser, server or GUI runtime attempt.

### New source coverage

Paths in this table are relative to `xtask/src/smoke/`. All rows are implemented
and offline-tested where pure contracts permit; actual runtime is **Not run**.

| Command / original | Current Rust scenario coverage |
|---|---|
| `board-chrome` / `verify_local.py` | `board.rs::{run,chrome_journey,seed,load_round,metrics,Stream}`: synthetic operator/viewer/Bob roles, CSRF and listing/item IDOR; literal stored create/history, versioned edit and retained conflict draft, exact intentional 409 filtering; SSE revision/status/pause/reconnect; five original responsive PNGs and six original demonstration records; graceful stop, stopped-DB 1000-item/up-to-version-10 history seed, persistence and retired Memory login; three 25-client, 50-operation-per-client diagnostic rounds after login/four warmup reads, ten open SSE streams, raw nanoseconds/status samples, nearest-rank percentiles, actual Linux CPU/RSS and graceful SSE EOF. This does not implement the YT-086 duration qualification. |
| `board-firefox` / `verify_firefox.py` | `board.rs::firefox_journey`: actual W3C Tab/Unicode/native submission, literal markup/label relationships, retained validation description/error relationship and corrected persisted edit; 2.2-second unchanged-SSE suppression observation, exact message-before-count mutation array, updated revision/status/pause, viewport and private cleanup. |
| `scene-chrome` / `verify_browser.py` | `scene.rs::chrome`: real script-disabled search/page/detail document and PNG, inert GPU assets, hardware/non-software adapter and browser SystemInfo, exact RGB 120-center 600x400 PNG/black margin; separate five original Chrome GPU environment profiles plus default X11, fresh absent/null/device/validation faults, blocked Wasm, missing/malformed manifests, held real-device destruction/replacement/sibling and no mutation/event circuit. Unavailable hardware is nonzero. |
| `scene-firefox` / `verify_firefox.py` | `scene.rs::{baseline,held_device}`: temporary experimental preference, actual adapter/pixels/two black samples, repeated activation, sibling/dispose/remount; held real device and retired completion, all four injected fault cases, missing-Wasm and malformed-manifest strict SSR fallback. |
| `scene-lifecycle-firefox` / `verify_lifecycle.py` | `scene_lifecycle.rs::run`: actual resize/coalescing/same-size behavior, CSS/backing/DPR clamp, zero/detached/offscreen/background-tab suspension/resume and idle draw counts; real controlled device destruction/loss promise, retained CPU/HTML/sibling, explicit recovery/remount/pixels; held acquisition and completed validation scope, exact device/buffer destruction, generation/current-owner and teardown counters. |
| `scene-picking-firefox` / `verify_picking.py` | `scene_picking.rs::run`: fresh WebDriver target/rect/DOM observations around actions; actual mouse/keyboard stable selection, reset, CSS/DPR remap/pixels; explicitly simulated touch/cancel/scroll/move, stale data/generation/disposed rejection; actual BiDi 390x700 viewport, fixed portrait pixels/no overflow, computed names/reduced-motion rule; unavailable semantic selection/sibling, scoped native search/explicit simulated composition, actual scriptless record navigation. |
| `scene-candidate-firefox` / `verify_candidate.py` | `scene_candidate.rs::run`: current source-bound candidate, original RGB and changed replacement/12-record patch pixels, upload metrics/picking/portrait; disposal/loss/recovery; held real queue completion, temporary same-origin page-module free hook under unchanged CSP, resource release versus delayed wrapper free, unaffected replacement; unavailable recovery and scriptless SSR. `YT081_WGPU_API_REFINEMENT=1` still explicitly selects the actual Wasm idempotent-dispose observations. |
| `scene-support-firefox` / `verify_support.py` | `scene.rs::support`: separate fresh default profile with empty requested preferences, secure-context/GPU capability and actual ready/unavailable observation; genuine pixels only when ready, zero-owned-resource/HTML fallback when unavailable; independent counter, native semantic keyboard selection and actual record-120 detail navigation in either outcome. It never enables experimental WebGPU. |

The small shared `Scene` methods cover actual repeated browser APIs, snapshots,
PNG reads and owned cleanup, not a replacement generic GPU scenario. `Browser`
retains its existing ordinary launch defaults; scene-only `Launch` differences
select GPU flags, temporary preferences, Snap-readable owned profile location,
X11 and BiDi. The existing WebSocket framing helper now supplies the origin-less
BiDi handshake actually used by portrait input. Firefox's optional scene deadline
bounds commands; cleanup clears that deadline before deleting the session.
Existing `Service` process-group/Job ownership is reused. Its Windows source was
not changed by this task; actual Windows execution remains the parent's gate.

`dashboard.rs` now checks the original initial HTTP heading, both loader markup
names, explicit `wasm-unsafe-eval` and absence of `unsafe-inline`. `proof.rs`
generates its own per-run `head` and recursive `source_sha256` fields, alongside
additive current Rust-tool source hashes. An aggregate hash is not substituted.

`build.rs::scene_build_sources` adds `rust_build_sources` to the scene manifest:
the actual Rust builder/templates/entry/routing/process sources, isolated
manifest/lock, root manifest/lock/toolchain, counter source and scene loaders and
shader. Original `candidate_sources`, asset bytes and digest algorithm are kept.
New scene commands reject a missing/stale Rust build-source field, so the parent
must use the Rust builder once before their runtime batch. Each scene run also
records/rechecks its own per-file input inventory, including current smoke code,
binary and canonical assets. No generated source receipt is fabricated here.

Actual task files: `xtask/src/smoke/{board,scene,scene_lifecycle,scene_picking,scene_candidate}.rs`
(new), `xtask/src/smoke/{mod,browser,websocket,http,dashboard,proof}.rs`,
`xtask/src/{build,lib,main}.rs`, `xtask/tests/smoke_protocol.rs`, isolated
`xtask/Cargo.toml`/`Cargo.lock`, root `README.md`, this document and the migration
ledger. Cached bundled `rusqlite=0.40.2` is isolated to xtask for stopped-DB
seeding; the root dependency graph is unchanged. No unrelated work was reverted.

### Reporting compatibility and explicit differences

- Original Board `qualification.json`/`firefox.json` report families, checks,
  screenshots, load summaries/raw sample arrays, seed/source/binary identity,
  viewport, sanitized failure observations and cleanup are retained under each
  new per-command directory. Screenshot paths are local to that run, not the
  historical `docs/screenshots/m11-board` directory. `failure_type` is
  `RustToolError` with an exit code rather than a Python exception class.
- Scene commands retain their scenario-specific terminal success strings,
  snapshots/faults/checks/pixels/capabilities/input inventories and cleanup in
  `result.json`. The generic exit envelope is separately `command-result.json`;
  it cannot overwrite the scenario report. Snapshot aliases `scene`/`state` and
  `status`/`text` support the old families; numeric `records` has a separate
  `records_text` field for the former Chrome text observation. Generic native
  PASS stdout remains condensed as previously documented; it is not an exact
  stdout schema replacement. Live consumers are not switched in this task.
- Parent source comparison corrected Support to preserve the original
  `verify_support.py` outcome contract: this particular suite checks default
  profile behavior, so verified semantic fallback may return exit 0 with
  `status=passed_yt080_default_profile`, `outcome=unavailable`,
  `fallback_checked=true` and `gpu_acceptance=false`. This is a successful
  fallback test, **not GPU drawing acceptance or a skipped required test**.
  GPU-only drawing suites still fail for unavailable GPU. Browser/tool absence,
  assertion failure, deadline, changed input and cleanup failure stay nonzero.
  This one minimal parent compatibility correction followed the terminal
  remaining-eight implementation; its original three repairs remain recorded.
- The current scene loader adds global `pendingInitializations`, `lateSettled`
  and retired `resourcesReleased` to snapshots. The old YT-075 whole-snapshot
  equality would fail on a correctly retired held device. `same_current_owner`
  compares all other owner fields and retired identity, while separately
  requiring pending zero, exactly one late settlement and released resources.
  This preserves replacement isolation with current diagnostics. It does not
  weaken the YT-076 acquisition/validation or candidate completion assertions.
- Firefox pixel geometry uses the declared 120 centers, tolerance 25, black
  interior and a one-row origin shift only when the entire first row is white.
  No color searching or tolerance adaptation occurs. Chrome keeps exact RGB and
  black margin. Synthetic fixtures prove mapping/decoding, not actual GPU output.

### Offline evidence, bounds and remaining acceptance

Commands use task-local `$HOME/.cargo/bin` and `RUSTUP_AUTO_INSTALL=0`, without
installations or host changes. The initial offline Cargo check updated only the
isolated lock with seven cached SQLite dependencies and exited 0; all subsequent
checks are locked. Initial full xtask tests passed 48 cases and strict Clippy
passed. A source inspection identified the current-loader snapshot comparison
issue and material reporting/deadline details; two bounded correction rounds
addressed these. A third source-backed correction retains one HTTP agent per
Board client, preserving the original persistent-connection latency boundary.
The final locked results and log hashes are recorded in the
receipt directory, including `checks.json` and `logs.sha256`.

| Final command | Actual result / receipt |
|---|---|
| `cargo test --offline --locked --manifest-path xtask/Cargo.toml` | Exit 0; 49 passed, two pre-existing test-only child entry points ignored in enumeration and explicitly exercised by their owning tests; `closeout-tests.log` |
| `cargo check --offline --locked --manifest-path xtask/Cargo.toml` | Exit 0; `closeout-check.log` |
| `cargo clippy --offline --locked --manifest-path xtask/Cargo.toml --all-targets -- -D warnings` | Exit 0; `closeout-clippy.log` |
| `cargo fmt --manifest-path xtask/Cargo.toml --check` | Exit 0; `closeout-fmt.log` |
| `cargo run --offline --locked --manifest-path xtask/Cargo.toml -- check docs` | Exit 0; 75 documents / 86 backlog items; `final-docs.log` (final content recheck: `docs-closeout.log`) |
| Original-source/root-graph rechecks | Exit 0; all 30 original Python sources and root manifest/lock/toolchain match the saved baselines; `python-source-recheck.log`, `preserved-inputs-recheck.log` |

Pure product tests cover all six distinct scene mappings, selected/default
preferences, source drift rejection/current Rust build sources, all eight command
parsers and invalid flags, exact grid/mouse ID/range mapping, fixed PNG tolerance
and full-white seam, per-file recursive inventories, live-owner versus global
retirement diagnostics, expired deadline before any I/O, SSE chunk/EOF/bounds,
nearest-rank percentile and idempotent stopped-DB seeding that retains existing
records/history. Existing process tests retain readiness-error identity,
termination/escalation, descendants, stream closure and temporary-tree cleanup.
No socket fixture or runtime smoke was executed.

This task's counters are `repair_attempts_used=3/4` and
`operational_retries_used=1/1`. The one tool correction fixed an unmatched context
line in a documentation patch; it launched no command and had no runtime effect.
Round 1 corrects the current snapshot/retirement
compatibility and related required negative/report/deadline behavior; round 2
completes concrete ownership/report fields after the focused rerun. Round 3
preserves each Board client's reusable HTTP connection for diagnostic samples.
These trace to
this owner's explicit parity requirements and the original reports. There is no
unresolved offline failure signature. The original terminal Job remains
`4/4` product repairs and `2/2` operational retries with unresolved historical
`loopback socket unavailable: PermissionDenied`; its counters were not reset.

All test-owned processes have completed/reaped. This task started no suite-owned
browser, driver, application server, listener, profile or database runtime. Pure
SQLite fixtures were closed and removed. No staging, commit, push, worktree,
framework runtime API, public CLI, YT-086 implementation or live CI/smoke
invocation adoption occurred. No Python/Node interpreter was invoked. Existing
M11 results and original Python sources remain unchanged.

No known implementation omission from the eight listed suites or the declared
Dashboard/proof gaps is left intentionally open. Actual Board/scene runtime,
changed Dashboard/proof/shared-helper parity, Windows persistent-service
execution, imported qualification fixture runtime/caller transition and final
independent review remain acceptance boundaries. The owner explicitly reserves
runtime execution and Windows execution for the parent. YT-085 stays In progress;
YT-086 is still dependency-blocked. The next unblockable work is the parent's
bounded new eight-command runtime batch below, then independent review; there is
no new framework architecture decision.

After preparing current artifacts, the parent can run exactly one relevant
batch from `$HOME/repos/yetlet` in its admitted host process context:

```sh
cargo build --offline --locked --manifest-path xtask/Cargo.toml
cargo build --offline --locked --release -p yetlet-lab-operations-board
cargo build --offline --locked --manifest-path examples/webgpu-scene-island/Cargo.toml
xtask/target/debug/yetlet-xtask build scene-island
xtask/target/debug/yetlet-xtask smoke board-chrome
xtask/target/debug/yetlet-xtask smoke board-firefox
xtask/target/debug/yetlet-xtask smoke scene-chrome
YT075_FIREFOX_HEADFUL_X11=1 xtask/target/debug/yetlet-xtask smoke scene-firefox
YT075_FIREFOX_HEADFUL_X11=1 xtask/target/debug/yetlet-xtask smoke scene-lifecycle-firefox
YT075_FIREFOX_HEADFUL_X11=1 xtask/target/debug/yetlet-xtask smoke scene-picking-firefox
YT075_FIREFOX_HEADFUL_X11=1 xtask/target/debug/yetlet-xtask smoke scene-candidate-firefox
YT075_FIREFOX_HEADFUL_X11=1 xtask/target/debug/yetlet-xtask smoke scene-support-firefox
```

These are parent instructions, not executed runtime evidence. Each invocation
creates a fresh directory under `target/m12/yt085/`; a failure remains recorded
and is not automatically retried. Chrome's default X11 hardware path may fail
honestly. Preserve the independent default-profile Support outcome and every
nonzero status; do not enable another GPU diagnostic profile automatically.
## Final YT-085 acceptance checkpoint — 2026-10-15

Original `wc_job_ddy0Xost8yg_9Mv4` was terminal lost/outcome-unknown (no verdict). Fresh independent reviewer cycles first found Browser readiness/native-exit status P2, then forms/scene/Board cleanup-status propagation P2, then Board native exit receipt loss on private directory teardown P2. Bounded repairs under `target/m12/yt085/parent-browser-status-fix/`, `parent-suite-cleanup-classification-fix/`, and `parent-board-status-retention-fix/` repaired these without changing original Python files or prior 26 indexed runtime receipts. The final separate reviewer `wc_job_Bm8KtPx_OBC8blP1` returned `pass`, `issues=[]`, `required_followups=[]` at `target/m12/yt085/reviewer-board-retention-20261015/result.json` (SHA-256 `251c3230e936c89294e15c3f15297eb5cb12363490cd2cd90d3463816df84fce`). 16 final focused Board tests plus fmt/strict Clippy passed; prior Browser tests (7) and owned-process tests (4) also passed. Browser/Server receipts record unknown native status as null, retain Unix signal/native exit evidence and disallow cleanup/receipt-write success. Board retains allowlisted per-process status after securely removing private temporary data, including restarts/failed startup.

Accepted **port scope** is not release or new environment qualification: prior source-bound 25 positive native/browser suites and one explicitly nonzero Chrome GPU-unavailable result remain as recorded; default Firefox Support passes HTML fallback only, `gpu_acceptance=false`. Windows persistent-Service execution and external Redis integration were not run, despite foundation-only Windows MSVC tests from YT-083. The 30 mapped Rust smoke/integration implementations are accepted; Python caller/CI invocation cutover remains YT-086. This checkpoint supersedes earlier `In progress` text only as a **historical status**; previous test failures, limits, and evidence remain immutable. No commit, push or public CLI activation.
