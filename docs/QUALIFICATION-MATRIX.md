# Platform, Browser and Accessibility Qualification Matrix

**Task:** YT-065 / M10f. **Current status:** **Done at automated desktop/native/browser qualification scope.** Run `38043565238` tested source `c38837835dac683b09194b475baab2d8eda5a875`. Real IME, assistive-technology speech, WebKit/mobile and zoom remain explicitly Not verified optional coverage. This matrix supplies no supported-device range, accessibility conformance, SLA or production guarantee.

## Coverage and evidence boundaries

| Lane | Defined scope | Current YT-065 evidence | Historical evidence |
|---|---|---|---|
| Ubuntu 24.04 native | Stable Rust check, library tests and doctests for nine reusable crates | Hosted Rust 1.99, x86_64-unknown-linux-gnu: 80 tests/four doctests PASS | [M9j](M9J-VERIFICATION.md) local native workspace freeze; not this hosted lane |
| Windows 2025 native | Same nine crates, native runner host target | Hosted Rust 1.99, x86_64-pc-windows-msvc: 80 tests/four doctests PASS | Separate authorized local Windows GNU Rust 1.88 run: 80/four PASS; not local MSVC |
| macOS 15 native | Same nine crates, native runner host target | Hosted Rust 1.99, aarch64-apple-darwin: 80 tests/four doctests PASS | No x86_64 macOS qualification claimed |
| Ubuntu 24.04 Firefox | Release Todo/Search, keyed bindings, multi-island suites; SSR forms keyboard/semantic checks | Local and hosted Firefox 157.0: all four suites PASS; hosted forms report cleanup PASS | M9j: three release suites PASS on local Firefox 157/geckodriver 0.37.1; forms keyboard suite did not exist |
| Hosted Chrome baseline | Existing required baseline workflow and existing three release suites | Candidate PR run `38042614403`: all three Chrome release suites PASS; not a new local Chrome claim | M9j: three local release suites PASS on Chrome 155 |
| WebKit/Safari desktop | Manual application journeys below | Not verified; no automated WebKit lane | No qualification claimed |
| iOS Safari / Android Chrome | Real device touch, soft keyboard, viewport and zoom | Not verified; no mobile lane | No qualification claimed |
| Real OS IME | Composition, candidate selection, caret and keyed input lifetime | Not verified; manual only | Existing synthetic keyboard/selection checks do not prove OS composition |
| NVDA / VoiceOver | Spoken names, language, errors, focus and navigation | Not verified; manual only | DOM assertions do not prove assistive technology output |
| Zoom/reflow | Desktop 200%/400%, narrow viewport and mobile text scaling | Not verified; manual only | No qualification claimed |

The native package list is exactly `yetlet-core`, `yetlet-reactive`,
`yetlet-html`, `yetlet-dom`, `yetlet-dom-keyed`, `yetlet-actions`,
`yetlet-server-axum`, `yetlet-request-id` and `yetlet-session`. It uses the
actual runner host target and records `rustc -vV`; runner labels do not imply
both x86_64 and ARM coverage. Sample binaries and Unix-specific sample smokes
are outside these native portability lanes. No failing tests are skipped to
manufacture a platform PASS. Browser packaging is qualified separately on
Linux. Stable Rust is explicitly installed and observed; the unchanged
historical baseline workflow supplied the Rust 1.88 MSRV check. On 2026-10-12,
the maintainer selected Rust 1.99 before first publication. The local baseline
now pins 1.99.0; the new toolchain file does not override rolling stable lanes,
which explicitly set `RUSTUP_TOOLCHAIN=stable`. [Integrity preparation](YT-020-INTEGRITY-PREPARATION.md)
records the retained source boundaries. [Hosted rebuild preparation](https://github.com/bohewu/yetlet/blob/849c5894f73d9732db9c2ccaa765385dc9d8a819/docs/YT-020-HOSTED-REBUILD.md)
records the later authorized protected-check migration after actual Rust 1.99 CI PASS.

## Historical performance and capacity evidence (bounded)

The original research cohorts below are **dated, source- and machine-specific**
observations; they do not establish current-version performance, minimum
hardware, production capacity, a cloud SLA, regression thresholds or
cross-platform equivalence. They remain recoverable from local Git source
`1d0eabcb99eeeb681795c1db3e27b2a5b47e0bcb`, including full profiles,
raw measurements, original failures, comparison rules, independent reviews and
cleanup. That local source SHA is **not yet verified as publicly reachable**.
Current build/fixture configuration belongs to the
[SSR measurement example](../examples/ssr-performance-baselines/README.md)
and [Axum/Yetlet workload example](../examples/axum-workload-comparison/README.md).
The retained [YT-141 controlled-arrival reference](YT-141-PERFORMANCE-VALIDATION.md),
[YT-074 Board qualification](YT-074-QUALIFICATION.md) and
[YT-175 admission-policy contract](YT-175-SESSION-BURST-OVERLOAD-POLICY.md)
retain their separate authorities.

| Historical scope | Verified observations and important limits |
|---|---|
| YT-074 desktop and synchronized Board loads | Separate desktop/cross-environment comparison and fsync-stall observations were **not** a portable throughput baseline. Source, storage, platform and synchronized versus staggered conditions matter; failed and noncomparable captures remain at their original identities. The retained [desktop acceptance](YT-074-DESKTOP-ACCEPTANCE.md) and [Board qualification](YT-074-QUALIFICATION.md) remain distinct |
| YT-141–144 API/saturation/router/PostgreSQL | YT-141's read-only controlled-arrival reference had 14,006 valid 200s, no drops and 1/2 ms p50/p95 only for its fixture. YT-142's first **1,000/s point FAILED** with 5 SQL-admission 503s among 10,001 requests; immediate 250/s recovery passed 7,501 valid 200s. YT-143 corrected only an unstarted comparator repetition; its SSR 4,000/s pair was **noncomparable** after a native Axum drop. YT-144's four isolated PostgreSQL pairs had 100,007 measured valid 200s; rejected preload/observer/lookup attempts and scope boundaries remain historical |
| YT-145–160 SSR allocation/latency attribution | Fixed 8/128/1,024-row SSR output baselines and independent escaped-output hashes separate constructing a View, body/document serialization, combined costs and full HTTP p95. Original escaping accounted for 85.70% of measured *document allocation calls* in one YT-146 instrumented profile, **not** of total CPU. Private escape-into and URL-prefix changes reduced selected allocations while held/peak memory did not automatically fall. First-child preallocation (YT-152) was **rejected** on tradeoffs. Changes in timings/frequency/worker CPU cannot uniquely identify the cause of earlier HTTP p95 observations; no general speedup claim |
| YT-161–164 benchmark isolation and transport | The attempted constrained execution profile **was not admitted** in YT-161; controlled-frequency evidence did not result. YT-162 retains 39,977 valid HTTP 200s and **27 dropped arrivals**, with only the first platform pair comparable. YT-163 attributes those 27 drops to a short exposure with full outstanding HTTP brackets but cannot identify a unique origin. YT-164 observed lower p95 under direct private HTTP than SSH forwarding in two bounded pairs; this is a transport/context effect, **not** proof of a faster Yetlet product |
| YT-165–169 finite/sustained capacity | Short isolated 4k and selected 5k/6k points were clean in their specific profiles, while 7k encountered pressure and a prior 8k point was saturated. A **6k/s sustained 300-second run FAILED** with 34,302 dropped arrivals (reported HTTP p95 63.33 ms). Separate **5k/s 300-second** and **5k/s 1,800-second** runs completed with 1,500,001 and 9,000,001 valid 200 responses respectively and zero drops; different allocations and evidence capture prevent a controlled duration-only comparison or production SLO |
| YT-170–174 synthetic Auth/Session/PostgreSQL | The synthetic sign-in is a **preverified fixture identity**, not real credential/OIDC verification; reads and private responses retain app-owned boundaries. YT-170's JSON/SSR fixed exposure had 2,000/2,001 valid 200s at 100/s each. YT-171 retained one Session Busy at JSON 500/s and 19 at SSR 2,000/s, followed by separate passing recoveries. YT-172's 648 read calls split into **552 valid / 96 Busy**. YT-173's bounded HTTP trace correlated 12 Busy outcomes in a 12,550-request cohort; passive host sampling in YT-174 does **not** establish an exact causal timing stage. Cleanup, failure and redaction policies remain source-bound |

**Interpretation:** Counted logical allocation bytes are not whole-process RSS;
nearest-rank HTTP p95 is not necessarily a k6 interpolated percentile; a valid
HTTP status does not establish completed SQL business writes; drops, 503 and
Session Busy must remain failed populations, not disappear into a success-only
denominator. Connection/SSH/frequency, CPU affinity, working sets, tool settings,
VM limits and benchmark fixture are part of the result identity. Unchanged
test/source artifacts are **not rerun** by documentation consolidation.

For application-owned outer HTTP admission and current operational priorities,
use the [workload capacity-selection guide](../examples/axum-workload-comparison/README.md#http-admission-capacity-selection)
and the [bounded cap-four/eight/sixteen results](#opt-in-http-admission-4816-bounded-cohorts),
not a universal default or capacity guarantee from historical throughput. The
[Starter production-readiness guide](PRODUCTION-READINESS-PHASE2.md) keeps the
separately accepted local YT-129 load/recovery and error semantics. Authentication,
authorization, CSRF, strict CSP, durable write ordering and distributed admission
were not changed by these measurements.

## Early provider and HTTP attribution evidence

These dated local research proofs are **not** new hosted qualification,
production capacity, cross-host Session recovery or durable write
guarantees. Complete source/measurement history is in **local Git**
`c3478b7f4e0c929a94ccbf21e05217535553a73d`; remote GitHub availability is unverified.

| Research | Verified bounded observation | Exclusions and historical failures |
|---|---|---|
| Redis async provider | Two real CAS barrier stalls were failed and cleaned; independently corrected test-only barrier followed by five actual Redis integration test passes | Previously accepted cancellation/drain and Cookie/browser tests not rerun; no production persistence or cross-host RPO/RTO |
| Provider-only admission | 648 attempts: 552 validated / 96 Busy; eight recovery reads valid; 72 waves with four active/eight total | No fair queue, exact 1ms scheduling, 80ms expiry or universal capacity ceiling |
| Same-process HTTP Session | 12 SSR Busy/503 rejections, four active/four pending owners; rejected before SQL | Different sixteen-VU arrivals; clock and resource observer interference, no proven unique cause or global SLO |
| Native HTTP/SSR | App-owned streaming can expose first body sooner but finish later; 100ms response-creation timeout can return 504 | No rollback of started work, whole-body deadline, browser paint, TLS/proxy flushing or production latency guarantee |
## Experimental GPU, browser and Physics Lab evidence

These are separate, dated and **bounded** research scopes, not
current universal GPU support or a 60 FPS/SLA criterion.

| Scope | Actual evidence | Explicit exclusions |
|---|---|---|
| Native scene (Oct 12) | Linux RTX4090/Vulkan wgpu27 offscreen 10k/50k/100k static instances with readback proof; 30 frames each, median **0.033760/0.054286/0.083525ms** encode+submit+completion wall | No browser GPU, matched DOM/CPU, GPU timestamp, displayed FPS or actual VRAM consumption |
| Browser Canvas2D/WebGPU (Oct 14) | Firefox158/X11/Intel Vulkan: original 50k screenshot-alignment attempt **FAIL**, separately repaired harness, then **24/24 PASS**, 2160 measured samples, 126 known-pixel screenshots | Different API completion semantics; no useful WebGPU speedup demonstrated, default Firefox/Chrome/Windows/macOS/mobile/NVIDIA not qualified |
| Experimental GPU API | Independently reviewed typed CPU SceneError and logical first-retirement bool, source-specific native/Wasm/Firefox checks | Async Wasm borrows need separate settlement; no stable ABI or platform support promise |
| 3D physics feasibility (Oct 26) | Source/registry comparison of wgpu27/Rapier3D0.36.1 and bounded app-owned design | **No joint Wasm build, browser 3D run or FPS measurement in that study**; later runnable sample has distinct evidence |

Historical source and original failure receipts remain in local Git
dfb27dc816c452e8033ff24e2d853f88a451761a; public SHA reachability is not verified.
## Opt-in HTTP admission 4/8/16 bounded cohorts

These dated **2026-10-25** runs apply to one isolated Auth/Valkey/
PostgreSQL reference, not framework defaults, future CLI versions,
distributed capacity, a physical ceiling or a production SLO.
JSON observe offered 500/s × 5s and SSR 2,000/s × 5s; each
ran a sixteen-request burst and eight serial read recoveries
per JSON and SSR mode. Every checked **503 is a failed request**.

| HTTP cap | Attempted | Valid 200 | Failed checked 503 | Outer HTTP-only / provider Busy / QueueTimeout |
|---:|---:|---:|---:|---|
| 4 | 12,550 | 12,452 | **98** | **98 / 0 / 0** |
| 8 | 12,549 | 12,475 | **74** | **74 / 0 / 0** |
| 16 | 12,552 | 12,540 | **12** | **7 / 5 / 0** |

Cap four rejected 98 before the Session provider, compared
with the separate earlier gate-off cohort's nine Session Busy;
this is a different source/build/oracle, **not an improvement
in successful throughput**. Cap eight had 24 fewer failures
and 23 more successes with **one fewer boundary attempt**;
burst p95 and failed client tails could worsen. Cap sixteen
added three boundary attempts versus cap eight and had seven
outer-only and **five genuine provider Busy** (two JSON burst,
three SSR observe), not twelve identical gate rejections.
None had observed QueueTimeout, SQL errors, denials, generator
drops, invalid 200, incomplete calls or capture overflow.
Both mode-specific eight-read recoveries and cleanup passed
for each run without making its failed requests successes.

Source/feature/startup-cap, selected route/valid GET and exact
private body/header 200/503 oracles plus complete joins are
necessary to attribute outer-only rejections. **No HTTP gate
admit/release/reason event** exists in the trace. Raw
nearest-rank versus native k6 interpolated p95, all/valid/
failed populations, separate Python recovery clocks and
distinct client/server timestamps are noninterchangeable.
These were sequential, not randomized comparisons: arrival,
warming, host/observer changes and caps themselves are confounded.

Original historical negative evidence: cap four needed a new
binary and strict caller oracle; caps eight/sixteen reused the
exact accepted binary/caller. Cap sixteen had one **pre-start
receipt-writing harness FAIL**, referencing `enforcement`
instead of `enforcement_status`; the limited correction
occurred **before startup/measurement**, with no repeated
exposure or runtime test failure. Earlier Session/Auth/CSRF/
browser/live backend proofs were retained, not rerun.
See the [current HTTP gate contract](HTTP-PIPELINE.md#opt-in-application-http-admission-contract)
and [sample's complete p95 and choice table](../examples/axum-workload-comparison/README.md#http-admission-capacity-selection).
Full original studies, failure receipts, raw timing and reviewer
findings survive at **local Git** `619b0d9d1e05c86ea0193efb5fc91a198ec930a8`, but fixed GitHub
historical links are **not yet verified publicly accessible**.
## Workflow source and activation

[Qualification workflow](../.github/workflows/qualification.yml) has only
`schedule` (Wednesday 03:17 UTC) and `workflow_dispatch`. It grants only
`contents: read`, uses full-SHA GitHub-owned checkout/upload
actions, disables persisted checkout credentials, cancels superseded runs,
limits native parallelism to two, and limits native/Firefox jobs to 35/60
minutes. It neither supplies secrets nor requests token write or id-token
permissions. This schedule/manual behavior remains unchanged. Remote protection
now requires `Rust 1.99 MSRV` and `Stable native, samples, and Chrome`, after
both first-candidate jobs passed in run `38193149046`. Historical 1.88 check/run
identities remain accurate. New candidate updates require new actual required CI;
the migration neither weakens protection nor relabels historical qualification.

GitHub requires this workflow on the default branch for activation; schedules
run there and may be delayed or dropped. Manual dispatch also depends on the
default-branch workflow. Local uncommitted YAML cannot establish hosted PASS.
The maintainer authorized that delivery: PR #1 was merged without bypass and run `38043565238` completed successfully. The workflow is now on the default branch; future schedule execution is not preclaimed. No repository visibility or branch-protection setting was changed by this delivery. See
[GitHub trigger documentation](https://docs.github.com/en/actions/reference/workflows-and-actions/events-that-trigger-workflows).

The [official Ubuntu2404 runner image inventory](https://github.com/actions/runner-images/blob/main/images/ubuntu/Ubuntu2404-Readme.md)
documents installed Firefox/geckodriver and the `GECKOWEBDRIVER` directory.
The Firefox job explicitly requires that directory's executable, puts it on
PATH, and checks Firefox plus both versions before builds. Missing tools fail
the job; no floating driver download or silent browser substitution occurs.
Image software can change: each run records `ImageOS`, `ImageVersion`, source
SHA, Rust host/tool versions, Rust xtask and browser/driver versions. Current
observations, rather than this inventory, establish the tested versions.

Artifacts retain only named qualification build/test/identity logs and the
sanitized forms JSON report for seven days, with unique lane/run/attempt names
and `always()` upload. No entire `target` tree, profiles, raw page state,
cookies or arbitrary environment dump is uploaded. Cargo and xtask failures
propagate through Bash `set -euo pipefail` even when stderr/stdout feed `tee`.
Job timeout/cancellation may prevent normal harness cleanup; the hosted runner
is disposable, and an interrupted job is not a successful qualification.

## One bounded local sequence (main session only)

Use prebuilt release assets from `cargo xtask build browser --release`,
`cargo xtask build keyed-bindings --release` and
`cargo xtask build multi-island-dashboard --release`, plus release native binaries
`yetlet-todo-server`, `yetlet-multi-island-dashboard` and
`yetlet-forms-validation`. Require Firefox and geckodriver on PATH and record
their versions together with Rust and source identity. Run each command once:

```sh
cargo xtask smoke browser-firefox --profile release
cargo xtask smoke keyed-bindings-firefox --profile release
cargo xtask smoke multi-island-dashboard-firefox --profile release
cargo xtask smoke forms-accessibility-firefox --profile release
```

Stop at failure, classify it before repair, and retain exact exit statuses.
The executor does not run this sequence. Main-session logs belong under
`target/m10-remaining/` or `target/qualification/`. Existing build helpers and
suites retain their established output locations and behavior. Main session
must check teardown for those existing suites; this task does not rewrite them.

The new forms command defaults to `release` and also accepts `--profile debug`.
It owns one ephemeral loopback server, one headless WebDriver session and its
POSIX process groups; it tracks startup processes before waiting, closes the
session, stops/reaps both groups with bounded existing helpers, and removes its
temporary tree. Its fixed report is
`target/qualification/forms-accessibility-firefox.json`. It uses real W3C
Tab/Enter/text key actions, without assigning focus or input via JavaScript.
DOM reads assert name/email/submit traversal, both label/control associations,
first-invalid-field autofocus, error text/role/descriptions, root `lang=en`
on initial/error/success documents, Unicode input preservation, and no-state
success/refresh/empty-form behavior. A separate no-redirect HTTP observation
checks 303, Location, no-store and no cookie. Unicode key actions exercise text
entry, not a real operating-system IME.

Parse the JSON only after exit: require command exit 0, `status=pass`,
`cleanup=pass`, `stage=complete`, all five declared check groups, a browser
version, and terminated owned process exit codes. Failure reports retain
bounded fixed stage names, exception types, completed groups and process exit
codes; raw process output and exception messages are discarded. `not_verified`
must retain real IME, screen-reader and other untested scopes even after PASS.
Cleanup failure prevents PASS. No screenshots or visible browser are needed.

## Optional future manual qualification runbook (currently Not verified)

Record tester/date, source/build identity, OS/architecture, browser version,
assistive technology or IME version/settings, profile and observed result for
each attempt. Use synthetic text and loopback fixtures; retain only redacted
notes. Record PASS, FAIL or Not verified per journey, with reproduction steps
for failures. A blank record or successful automation is not manual evidence.

1. **Keyboard and forms:** Open `/form` in the forms sample. Traverse with Tab
   and Shift+Tab; confirm name, email, submit order and visible focus. Submit
   `x`/`invalid` with Enter; confirm first-field focus and understandable linked
   errors. Correct only name, submit and confirm email focus. Correct email,
   submit, refresh success, and return to a blank form. Check English language,
   labels and no saved data. Repeat relevant Todo/Search and dashboard controls
   without a pointer; inspect focus after keyed move, remove, disposal/remount.
2. **Real IME and caret:** Enable a real Chinese/Japanese/Korean OS IME on
   Linux, Windows and macOS where available. Compose multi-character input,
   choose candidates, cancel composition and insert in the middle of existing
   text. Observe caret/selection and committed text. In the keyed fixture,
   reorder while editing and verify ownership/focus/text/caret survive; test
   removal/disposal. Record engine and composition behavior explicitly; typed
   Unicode or dispatched synthetic composition events cannot satisfy this cell.
3. **NVDA and VoiceOver:** With Windows NVDA in Firefox/Chrome and macOS
   VoiceOver in Safari, read headings/landmarks and move through controls.
   Confirm spoken field names, English language, required/invalid state,
   linked error text and first-error focus after each server navigation. Confirm
   success content, keyboard operation of islands and absence of duplicate
   announcements after remount. Record actual speech/behavior and settings;
   automated label checks alone cannot satisfy this cell.
4. **WebKit and mobile:** Run SSR forms, Todo local/remote commands, Search
   activation and dashboard independent activation/disposal on desktop Safari,
   real iOS Safari and real Android Chrome. Check touch targets, soft keyboard,
   rotation, text entry, errors, focus and server fallback. Record device/OS
   versions and browser console/CSP outcomes separately. A resized desktop
   Firefox window does not prove a mobile browser.
5. **Zoom/reflow:** At 200% and 400% desktop zoom and a 320 CSS-pixel viewport,
   inspect clipped content, horizontal scrolling, control reachability and
   focus visibility. On mobile increase OS/browser text size and repeat forms
   errors and islands. Record observed defects and exact settings. No general
   WCAG certification follows from these bounded journeys.

## Bounded task acceptance and optional future coverage

The local/browser and hosted run IDs, terminal results, observed versions/architectures and artifact references are recorded in [M10f verification](M10F-VERIFICATION.md). The authorized workflow merge and actual hosted run reached terminal success. YT-065 is therefore Done at the bounded automated desktop/native/browser scope.

The manual cells above remain explicitly Not verified and optional. They do not block this bounded task, do not become supported-platform or accessibility-conformance claims, and must not be inferred from typed Unicode or DOM assertions. Supported platform/version policy, real-device/assistive-technology ownership and any application-specific manual evidence remain future decisions. YT-020 release qualification remains Blocked. YT-066 is a separate backlog item.
