# YT-076 — GPU Lifecycle Implementation and Build Verification

**Date:** 2026-10-14. **Backlog: Done at the selected Ubuntu Firefox scope.**
Implementation, focused CPU/native/WASM checks and the parent-owned real
Firefox lifecycle run have **PASS** evidence. The browser completed all
11 selected checks and two 120/120 pixel samples with verified teardown.
**Independent reviewer: PASS**, zero issues and zero required followups.
The original implementation-only leaf checkpoint is retained below; no
YT-077 or later work, host change, commit, push or release has started.

## Authority and preserved source

The assignment explicitly states:

> User explicitly requests continuing from Done YT075 all the way to YT080 if feasible, independently reviewing EACH item before next.

> THIS assignment is ONLY YT076 implementation + focused CPU/WASM verification and a ready-to-execute focused Firefox lifecycle test.

> Do not work on YT077+ or mark YT076 Done. The parent will execute one bounded Firefox run then dispatch a separate reviewer.

Requirements come from [M11-G sections 3–4 and 8](M11-WGPU-PLAN.md),
[backlog YT-076](BACKLOG.md) and this explicit bounded assignment. Earlier
[YT-075 activation](YT-075-ACTIVATION.md), [successful Ubuntu Firefox receipt](YT-075-UBUNTU-BROWSER-VERIFICATION.md)
and [independent review](YT-075-INDEPENDENT-REVIEW.md) retain their historical
bytes and scope; they are not transferred to the new lifecycle implementation.

Work stayed in the original Ubuntu Yetlet checkout, `master`, HEAD`76be7aa1107e08e3c7850abc8c68f53301cf0f7c`. The existing 20-path uncommitted
YT-075 delta and empty index were preserved. The supplied pre-change source
snapshot remains at
`target/m11-gpu/yt076-20261014/implementation/yt075-source-snapshot.zip`.
The exported generalist leaf contract and Materiality Gate were read; no
child dispatch, runtime model identity or independent self-review is claimed.

## Changed application surface

Only the existing application-owned, unpublished nested workspace is extended.
wgpu remains `=27.0.1` with `std`, `webgpu`, `wgsl`, wasm-bindgen remains
`=0.2.129`, and both lockfiles and manifest pins are unchanged. Core, HTML,
default server and unrelated DOM entry graphs acquire no GPU dependency.
The trusted shader and initial 120 synthetic row-major red/green/blue records
are unchanged. No public crate or scene graph is extracted.

| Path | Bounded change |
| --- | --- |
| `examples/webgpu-scene-island/src/lib.rs` | Finite viewport helper, DPR/dimension bounds and single-commit lifetime transitions; CPU tests |
| `examples/webgpu-scene-island/src/browser.rs` | Device-limit resize/reuse, demand draw counters, generation-bound device-loss/error callbacks, retained immutable CPU bytes and owned destruction |
| `examples/webgpu-scene-island/scene-loader.js` | Coalesced owner RAF, viewport/pause observers, named failure, explicit recovery and teardown |
| `examples/webgpu-scene-island/src/server.rs` | Same-origin external CSS route/link reserves CSS dimensions separately from backing dimensions under strict CSP |
| `examples/webgpu-scene-island/verify_lifecycle.py` | One focused real-Wasm Firefox lifecycle product test using existing WebDriver/PNG/process helpers |
| `examples/webgpu-scene-island/README.md` | Current lifecycle behavior, pending gates and parent command |
| `docs/BACKLOG.md`, `docs/ROADMAP.md`, this record | YT-076 In progress and pending delivery gates |

## Lifecycle choices and limits

- CSS dimensions come from the actual canvas rectangle; physical dimensions use
  finite positive DPR clamped to 1–2 and are rounded/clamped to the acquired
  device's `max_texture_dimension_2d` before canvas/surface configuration.
  Scene coordinates remain normalized and independent. Invalid numeric inputs
  fail before configuration. Zero-sized or detached targets suspend and retain
  the previous bounded backing size; they do not configure zero dimensions.
- Only changed physical dimensions reconfigure the surface. Resize and explicit
  invalidations coalesce into at most one owner RAF. A static scene has no
  perpetual RAF loop. ResizeObserver, IntersectionObserver, detach observation,
  document visibility, window resize and a rearmed resolution media query drive
  demand. Hidden, offscreen and detached canvases pause; zero size also suspends.
  There is no animation clock or resumed simulation delta to accumulate.
- Rust lifetime guards permit one initialization commit. Disposal/failure retires
  the generation; acquired or validated late results are dropped/destroyed. The
  pending Rust object is freed only after its async method settles. A replacement
  cannot receive the old generation's status, device callback or surface update.
- Real device loss enters `lost`; surface lost/outdated enters `lost`;
  validation, exhaustion, timeout and other device/surface errors enter named
  `error`. Capability/acquisition/asset failure retains `unavailable`. No raw
  diagnostic is inserted as HTML. There is no automatic retry loop. Activating
  again after failure explicitly starts one new generation.
- Immutable 3,840-byte CPU record data survives device generations. The semantic
  SSR list/search/detail remains independent of rendering and faults. GPU teardown
  destroys the owned record buffer/device, drops renderer handles, explicitly
  unconfigures the current canvas context (wgpu 27 WebSurface Drop is a no-op),
  cancels RAF, disconnects all three owner observers and removes three owner
  listeners. Pending GPU promises cannot be cancelled by the browser API; their
  lifetime guards discard results. Page activation/disposal controls remain for
  remount and are removed on pagehide; the sibling counter is independent.
- Diagnostics distinguish CSS/backing/DPR, configuration/draw/coalesced counts,
  pending/late initialization, generation, owner observer/listener/surface counts
  and explicit CPU/buffer byte lengths. These are application counters, **not
  measured VRAM**, browser/OS memory or immediate driver reclamation evidence.

## Actual checks

All final commands exited 0. Logs are under
`target/m11-gpu/yt076-20261014/implementation/`.

| Command | Actual result and log |
| --- | --- |
| `cargo test --offline --locked --manifest-path examples/webgpu-scene-island/Cargo.toml` | 7 passed; 0 failed, ignored or filtered; `cpu-test-final.log` |
| `cargo fmt --manifest-path examples/webgpu-scene-island/Cargo.toml --check` | PASS; format applied once when stable; `fmt-repair2.log` |
| `cargo clippy --offline --locked --manifest-path examples/webgpu-scene-island/Cargo.toml --all-targets -- -D warnings` | PASS; `native-clippy-final.log` |
| `cargo clippy --offline --locked --manifest-path examples/webgpu-scene-island/Cargo.toml --target wasm32-unknown-unknown --lib -- -D warnings` | PASS; `wasm-clippy-final.log` |
| `cargo build --offline --locked --manifest-path examples/webgpu-scene-island/Cargo.toml` | Native server binary built, not launched; `native-build-final.log` |
| `python3 examples/webgpu-scene-island/build.py` | Release wasm32/build-bindgen PASS; scene `b0c42a3466125a53`, 259,231 raw WASM bytes; counter unchanged `81c388455a88d98a`, 151,613 bytes; `release-build-final.log` |
| `node --input-type=module --check < examples/webgpu-scene-island/scene-loader.js` and counter loader | Syntax PASS, no execution; `syntax-final.log` |
| Python `ast.parse` for all four fixture Python files | Syntax PASS, browser tests not executed; `syntax-final.log` |
| `python3 scripts/check_docs.py` | PASS including this record; `docs-check.log` |
| `git diff --check` and focused no-index whitespace checks | PASS; `diff-check.log` |
| SHA-256 comparison of eight existing Todo/Search assets and both lockfiles | All unchanged; `isolation-before.json`, `isolation-after.json` |
| Exact source snapshot comparison | Only the nine listed paths differ/add; historical YT-075 source/evidence preserved; `source-delta.json` |

CPU tests cover finite/nonfinite/negative values, invalid DPR/limits, bounded
DPR, extremely large CSS dimensions, fractional minimum backing, zero/detached
suspension and init/fail/dispose/replacement transition isolation. SSR tests
retain search/pagination/escaping, language, strict CSP and asset/sibling fallback.
Build/lint results establish no browser GPU, screenshot or CSP runtime outcome.

## Failure classification and bounds

The first native/WASM strict Clippy checks failed on deprecated
`AtomicU8::fetch_update` under the installed Rust 1.99 toolchain. This was a
`product_failure` against the required lint gate; repair 1 changed only that
call to `try_update`, and focused checks passed. The initial test/logs are
retained as `cpu-test.log`, `native-clippy.log` and `wasm-clippy.log`.

Source inspection then confirmed the wgpu 27 WebSurface drop is a no-op and
initial CPU bytes were temporary. Those were concrete gaps in the existing
teardown/CPU-retention contract. Repair 2 added explicit current-generation
context unconfiguration and immutable CPU-byte retention. Final focused checks
passed. Product repairs: **2/2**, no further product repair authorized within
this leaf attempt. There were no harness execution failures or harness repairs.
One documentation-edit command had a Python quoting SyntaxError before any
mutation; one minimal command correction succeeded. Operational retries: **1/2**.
No browser, server, watcher or other background runtime was started by this
slice; bounded build commands are terminal. There is no cleanup blocker.

## Completed parent-owned browser verification

After the generalist implementation/build Job completed (native exit 0),
the current session ran the prepared command once, with no product or harness
repair and no repeat of the old YT-075 proof. The real Firefox 158 X11 test
exited 0, using the current native server and scene digest `b0c42a3466125a53`.
The receipt is `target/m11-gpu/yt076-20261014/browser/result.json`, SHA-256
`20dcc18211812764333082ba583455d40d304f5f54c76ca708896fd0f30de2e0`.
Its input hashes still match the current sources/binary/assets.

All 11 checks passed: one owner/static idle; latest-size resize coalescing
and unchanged-size reuse; CSS/backing/DPR separation with controlled DPR 3
clamped to 2; zero-size, detach, real offscreen intersection and real
background-tab visibility pause/resume; controlled real device destruction
and its loss promise, retained CPU/HTML and independent counter, exactly
one explicit recovery; disposal/remount; and late real acquisition and
validation completions destroyed without displacing the replacement owner.
Both resized-DPR and recovered screenshots verified all 120 centers and
black interior samples. Their identical PNG SHA-256 is
`347ca920fa268d7bc812c7e33ca429c52ed430775803bf2a66ab6129875885f1`.
The one-pixel Firefox capture edge remains recorded; it was not treated as
an interior GPU pixel. This is controlled loss, not a hardware failure claim.

WebDriver session deletion, geckodriver/server reaping, Firefox exit,
private profile removal and both loopback port closures passed. A fresh
check found each reported owned PID absent. No source changed during the
browser run and no browser configuration/host setting was altered outside
the disposable test profile. The subsequent independent review passed.

## Independent reviewer acceptance

The separate read-only reviewer Job `wc_job_F-A3JSzK9jd6Qb1y` completed
with native exit 0 and verdict **PASS**, zero issues and zero required
followups. Its exact Codex thread was
`01a13a07-67bf-7a90-a00b-072d28442da0`; runtime `turn_context`
confirmed saved model `gpt-6-astra` and resolver-selected effective
`high`. Profile/current policy verification and re-resolution with the
observed effort passed without conflict or degradation. This is a local
role-directed Job, not native `agent_type` identity or formal assurance.
All reviewed source hashes remained unchanged during the review.

The local result is
`target/m11-gpu/yt076-20261014/reviewer/result.json`, SHA-256
`7575382a5554ab575db4c3028ef592989e3246bc8ff4cc699596af09e1bd46bf`.
Related reasoning input/decision, process/exit, source-before and observed
telemetry records are retained. No additional browser run was made by the
reviewer. YT-076 is Done for the selected Firefox experimental-opt-in
configuration only; YT-077 dynamic comparison is the next separate task.
No Chrome/NVIDIA, broader platform support, public crate, commit or push
claim follows from this result.

## Historical prepared parent-owned browser and review gates

**Browser Not run.** The prepared, single-use focused command is:

```sh
YT075_FIREFOX_HEADFUL_X11=1 /usr/bin/python3 examples/webgpu-scene-island/verify_lifecycle.py
```

It requires the already installed Firefox/geckodriver, existing X11 session
(`DISPLAY=:0` and existing Xauthority by default), built native binary and release
manifest. It uses the successful YT-075 headful-X11 recipe, only temporary
`dom.webgpu.enabled=true`, and no blocklist/security bypass or installation.
The test refuses an existing `target/m11-gpu/yt076-20261014/browser` directory;
that directory does not yet exist. It must not overwrite a failed or completed
receipt, rerun the original YT-075 test or change the source during the attempt.

Prepared assertions cover demand-only idle, repeated activation, coalesced/latest
resize and unchanged-size reuse; injected DPR 3 clamped to 2 over actual resources;
zero/detach/intersection/background-tab pause and resume; real `GPUDevice.destroy()`
with its real loss promise and exactly one explicit recovery; retained CPU/HTML
and sibling controls; disposal/remount; and held real device acquisition and
validation completions destroyed after retirement without replacing the owner.
**Controlled destruction is injected loss, not hardware failure.** The held
promises and DPR override are explicitly labeled controls, not simulated GPU
renderers. Pixels are rendered by the actual Rust/wgpu WASM.

The browser test retains `input-hashes.json`, `result.json`, `server.log`,
`geckodriver.log`, `resized-dpr.png` and `recovered.png`. Screenshot checks request
an actual demand draw and wait through RAF presentation opportunities; all 120
centers and interior black points must match. Firefox's possible one-row white
capture edge is retained/reported, with clear-color assertions inside the canvas.
Ready state alone never passes the pixel gate. Results contain actual typed
snapshots, per-check status, owned PIDs/ports/profile, WebDriver DELETE, process
reap/profile removal/port closure and exact input hashes, including on failure.

After one bounded browser run and verified cleanup, the parent dispatches a
separate independent read-only reviewer. These two gates remain unmet, so
YT-076 remains In progress. YT-077 is the next dependent item only after those
gates and any separately required workload authority; no later implementation
is unblockable in this leaf assignment. Promotion/support/performance decisions
remain open in their original tasks.
