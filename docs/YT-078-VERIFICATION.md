# YT-078 — Picking Implementation and Prepared Browser Product Test

**Date:** 2026-10-14. **Backlog: Done at selected experimental Firefox scope.**
Implementation/native/WASM checks and selected Firefox browser picking checks
**PASS**. Fresh independent reviewer **PASS**, zero issues and required followups. The original implementation-only
checkpoint below remains historical; no YT-079 code has started.

## Current parent-owned browser result

The selected Firefox 158 X11 test completed with native exit 0 and status
`passed_yt078_picking`, using the actual server and current Rust/wgpu Wasm.
Evidence is retained under `target/m11-gpu/yt078-20261014/browser/viewport/`;
`result.json` SHA-256 is
`a265e7feb6aa72191253532d206848dda33687c74bd2e94872fb7ada3894f92d`.
All 54 bracketed action records completed. Four actual canvas captures each
verified 120/120 centers and black interior: initial, CSS resize/injected DPR,
replacement and portrait. Stable IDs, actual WebDriver mouse/keyboard,
deferred stale version/generation rejection, labeled navigation, no-GPU
selection/counter, and scriptless real SSR record navigation passed.
Touch tap/cancel/move/scroll and composition are explicitly synthetic.
The actual reduced-motion media match was false; animations and transitions
were absent, with the reduced-motion rule retained. No physical mobile,
Chinese IME session, assistive speech or true reduced-motion OS setting
qualification is claimed.

The first test failed its narrow-window setup assertion: the X11 window
manager imposed a larger native Firefox window than the requested 390 pixels.
Its other executed assertions and successful cleanup remain in the original
`browser/result.json`; that failure is not relabeled as a product failure.
A single focused harness correction uses standard WebDriver BiDi
`browsingContext.setViewport`, reusing the existing JSON WebSocket transport,
with no browser/OS preference or product-code change. The returned unique
owned context was selected by its current URL. Actual final layout was
390 x 700 CSS pixels, document scroll width 390 and 12 semantic list rows.
Desktop outer width remained 1100; this is viewport emulation, not a phone.
Pixel tolerance stayed 25 and earlier evidence was not overwritten.

The WebDriver session, Firefox, driver/server processes, temporary profile
and both loopback ports were cleanly removed. Fresh PID checks found the
reported processes absent. All source/binary/asset input hashes still match.
The builder used no product repairs; parent used one harness setup correction.
The separate independent reviewer passed, completing the YT-078 gate.

## Independent reviewer acceptance

The independent read-only role-directed Job `wc_job_p0kklYOATDXlWvRq`
completed with native exit 0 and verdict PASS, zero issues and zero required
followups. Exact Codex thread `01a13a6f-9ae4-7322-85f7-a74d63b98c5c`
reported saved model `gpt-6-astra` and resolver-selected actual effort `high`.
This is not native managed `agent_type` dispatch or formal certification.
The reviewer checked source/diff, 54 bracketed action records, real mouse/
keyboard and stale token behavior, semantic fallback, safe sinks, all four
PNGs independently (120/120 each), capture setup, asset hashes and teardown.
Only the browser fixture changed between attempts; no product failure was
concealed. All reviewed source hashes stayed unchanged during review.

The receipt is `target/m11-gpu/yt078-20261014/reviewer/result.json`, SHA-256
`491fa16a0900934fe77036745c47e96840a5a1a1cbc7e7793841372a4fee7675`.
The selected viewport/touch/composition qualifications remain explicit; no
physical-mobile or assistive-speech support is inferred. YT-079 can follow
under the owner's separate Experimental opt-in decision, with its own review.
No new commit or push occurred after the initial guarded YT-075–077 commit.

## Authority and preservation

The owner assigned only YT-078 implementation, focused native/WASM checks and a
direct browser product fixture for parent execution, followed by a different
independent reviewer. Requirements are [M11-G sections 6 and 8](M11-WGPU-PLAN.md),
[YT-078 backlog](BACKLOG.md), [YT-076 lifecycle evidence](YT-076-VERIFICATION.md),
[YT-077 measurements](YT-077-MEASUREMENT.md) and its
[independent review](YT-077-INDEPENDENT-REVIEW.md). Repository AGENTS.md, ADR 0008,
the canonical exported generalist leaf role and Materiality Gate were read.
No workflow, child dispatch or runtime model identity is claimed.

The current checkout stays at committed baseline
`8c524520256e5fd9884d4d023dbdfbd76181c561`. The three existing owner decision edits
in M11-WGPU-PLAN.md, ADR 0008 and M11-G-EXPERIMENTAL-PROMOTION-DECISION.md are
byte-identical to their pre-work snapshot. The parent-created implementation
evidence directory and previous receipts were retained. No server/browser,
installation, host setting, detached worktree, fetch, commit, push or YT-079+
code was executed. The [Experimental promotion decision](M11-G-EXPERIMENTAL-PROMOTION-DECISION.md)
is preserved without turning future 3D motivation into this task's scope.

## Changed files and contract

| File | Required behavior |
| --- | --- |
| `examples/webgpu-scene-island/src/lib.rs` | Finite CSS/backing/scene coordinate mapping, bounded reverse flat picking, typed generation/version token, monotonically checked data versions and CPU tests |
| `examples/webgpu-scene-island/src/browser.rs` | Wasm owner pick/version/acceptance methods sharing CPU rules and lifetime guards |
| `examples/webgpu-scene-island/scene-loader.js` | Local tap/cancel/scroll handling, deferred token checks, labeled keyboard selection/reset, safe text and validated detail URL |
| `examples/webgpu-scene-island/src/server.rs` | Semantic controls/help/detail, portrait/reduced-motion CSS and focused SSR tests; original search, pagination and counter remain |
| `examples/webgpu-scene-island/verify_picking.py` | Direct selected Firefox product test, prepared only, reusing existing helpers |
| `examples/webgpu-scene-island/README.md`, `docs/BACKLOG.md`, this record | Invocation, actual checks and pending parent browser/reviewer gates |

The existing 32-byte layout, finite geometry/color validation, maximum 100,000
records, typed update ranges and allocation bounds are retained. wgpu
`=27.0.1`, wasm-bindgen `=0.2.129`, web-sys `=0.3.106`, manifests and both locks
are unchanged. No root/default GPU dependency, public renderer API, speculative
scene graph, new dependency or generic testing framework is introduced. Existing
YT-075–YT-077 APIs, tests, shader and comparison workload remain intact.

Picking is an O(n) CPU validation/scan on explicit selection, bounded by the
existing maximum. It creates no spatial index or GPU readback. Client coordinates
subtract the current CSS canvas origin, map proportionally through rounded,
bounded physical backing dimensions and then normalized scene coordinates.
DPR cancels in the ratio rather than scaling input twice. Nonfinite/negative
viewport or pointer inputs fail; zero-sized targets and exterior points miss.
Canvas left/top edges are included, right/bottom exterior boundaries excluded.
Rectangle edges inside the canvas are inclusive. Reverse draw order defines the
last-drawn/topmost overlap winner. Misses leave selection unchanged.

Record slot + 1 is the stable application semantic ID, independent of color or
viewport, across status updates and row-major replacements. Replacements preserve
that application ordering contract; arbitrary record reordering is not an API.
Every successful replace/update advances a checked u32 data version; failed
validation or exhaustion cannot mutate data. Pick tokens bind generation,
version and validated ID to the same lifetime at the text sink. Updates invalidate
pending taps; replacement/disposal resets selection; old generations cannot
commit to a replacement. These are small synchronous/deferred local selections,
not an asynchronous scene-query framework.

All labels and detail use Rust Views or JS textContent. Selected product IDs
1–120 alone produce `/scene?record=<validated integer>` URLs; larger research
batches have no invented SSR record URL. The product's default 120-record HTML
alternative retains real server search, 12-record pagination, record detail and
text status meanings without JavaScript or GPU. Large comparison batches remain
research data; no 100k hidden nodes or expanded SSR dataset is claimed.

The labeled selection button handles Left/Right, Up/Down, Home/End and native
Enter/Space. Reset is a separate labeled action. Handlers are scoped to that
button and ignore composing/modifier keys; ordinary search/IME remains native.
Touch uses pointer tap with a six-CSS-pixel movement threshold and cancels on
movement, scroll, pointercancel or lost capture. There is no capture,
preventDefault on touch, drag dependency or hover action. CSS allows pan/zoom.
Portrait keeps the canvas aspect ratio and data/actions without horizontal
page overflow. Static demand drawing has no animations in either motion mode;
reduced-motion CSS keeps scrolling immediate. Text/links carry meaning alongside
RGB colors. Accessible names/roles are prepared for inspection, with no
screenreader speech or physical mobile qualification claimed.

## Actual focused checks

Logs are retained in `target/m11-gpu/yt078-20261014/implementation/`. Commands
used the installed Cargo and Node via task-local PATH; nothing was installed.

| Command | Result / evidence |
| --- | --- |
| `cargo test --offline --locked --manifest-path examples/webgpu-scene-island/Cargo.toml` | PASS: 12 tests, 0 failed, ignored or filtered; `yt078-cpu-tests.log` |
| `cargo fmt --manifest-path examples/webgpu-scene-island/Cargo.toml`, then `--check` | PASS; `yt078-fmt-check.log` |
| `cargo clippy --offline --locked --manifest-path examples/webgpu-scene-island/Cargo.toml --all-targets -- -D warnings` | PASS; `yt078-native-clippy.log` |
| `cargo clippy --offline --locked --manifest-path examples/webgpu-scene-island/Cargo.toml --target wasm32-unknown-unknown --lib -- -D warnings` | PASS; `yt078-wasm-clippy.log` |
| `cargo build --offline --locked --manifest-path examples/webgpu-scene-island/Cargo.toml` | PASS, server never launched; `yt078-native-build.log` |
| `python3 examples/webgpu-scene-island/build.py` | PASS, current-source release Wasm and matching bindgen; `yt078-release-build.log` |
| Installed Node `--input-type=module --check` on three loaders; Python `ast.parse` on all six fixture/build scripts | PASS; `yt078-syntax.log`; no runtime execution |
| `python3 scripts/check_docs.py` | PASS; `yt078-docs-check.log` |
| `git diff --check` and whitespace check of new files | PASS; `yt078-whitespace.log` |
| SHA-256 comparison of eight current Todo/Search assets, root/nested locks and three owner edits | All unchanged; `yt078-isolation-before.json`, `yt078-isolation-after.json` |

The nine earlier native tests remain, with two focused CPU picking/token tests
and one semantic detail test added. CPU evidence demonstrates altered CSS size
and portrait aspect, DPR 1/2/3, stable IDs after status changes, invalid numeric
input/payload, exterior/zero bounds, inclusive rectangle edges, topmost overlap,
maximum-size picking, stale versions after update/replacement, stale generation,
retired lifetime and version-exhaustion atomic rejection. SSR retains escaping,
strict CSP, search/page content and sibling fallback; CPU/build results do not
establish browser pixels, input dispatch or hardware accessibility.

Current scene digest: `ced431f7a73b1017`; scene Wasm: 284,585 bytes. Counter digest
remains `81c388455a88d98a`; counter Wasm: 151,613 bytes. Current bundle sizes and
SHA-256 values are in `yt078-bundle.json`, not an HTTP transfer or performance
measurement. No old browser suite or benchmark was rerun.

## Prepared parent run and pending review

Run exactly once in the selected environment:

```sh
YT075_FIREFOX_HEADFUL_X11=1 /usr/bin/python3 examples/webgpu-scene-island/verify_picking.py
```

`target/m11-gpu/yt078-20261014/browser` is reserved and currently unused. The
fixture refuses existing evidence, performs no build/install, starts one exact
native server and one Firefox 158 headful X11 process with an isolated temporary
`dom.webgpu.enabled=true` profile, and hashes sources/binary/current assets.
It uses a 180-second focused deadline, reserving cleanup within a 240-second
operational bound. Fresh WebDriver element/rect and live state snapshots bracket
every input action. Real mouse and keyboard changes must produce expected DOM
text/validated URLs. Actual non-fallback hardware and four 120-center canvas
captures are required; the established full-white top capture row adds a fixed
one-row coordinate offset and tolerance remains 25 per channel.

Prepared checks include initial/resized/portrait/replaced pixels, CSS and injected
DPR mapping, simulated touch tap/cancel/move/scroll, stale replacement/disposal/
generation tokens, accessible labels, static rendering/media rule, no-GPU
keyboard fallback, independent counter, native text entry and simulated composing
key noninterception. Scriptless real SSR search/page/detail and link navigation
use only a temporary asset-manifest removal; this is no-script HTML evidence,
not a browser JS-disabled preference claim. Actual reduced-motion match is
recorded; the fixture does not change OS settings. Synthetic touch is not
physical input, simulated composition is not a Chinese IME qualification,
and accessible-name checks are not screenreader speech evidence.

Cleanup always deletes the WebDriver session, reaps the owned process groups,
removes the disposable profile/asset tree, checks the browser PID and closed
ports, and rechecks input hashes. Incomplete cleanup cannot pass. The leaf
started no server/browser/watcher or background runtime; all build commands
completed. Browser pixels and input/accessibility outcomes remain **NOT RUN**.
A **different independent reviewer** remains **NOT RUN**. Parent execution and
that review must pass before Done; YT-079 is the next dependent item only after
these gates, under the preserved Experimental opt-in approval. No implementation
of that item is included. There are no additional open architectural decisions
within this slice.

Product repairs used: **0/2**. Operational retries used: **0/2**. Harness-only
repairs used: **0/1**. Last failure signature: empty; no check failed. Pending
browser/review gates are handoff work, not failed-check repairs.
