# YT-143 — Native Axum and Yetlet standard-workload comparison

Date: 2026-10-23. Status: **Done at bounded three-workload comparison scope after
independent contract and final Reviewer PASS, zero issues and required followups.
The initial tmpfs deviation and all failed points remain retained.**

## Authority and declared question

The user authorized a new native Axum-versus-Yetlet comparison after the YT-142
commit: contract first, minimum new verification, independent Reviewer PASS
before Done. This document records that bounded request; its implementation
details do not authorize a broader benchmark product or new qualification gates.
The question is how the two route-registration paths behave under equivalent
plaintext, per-request JSON serialization and small dynamic SSR workloads at
two fixed offered rates, with matched output and resources. Record latency,
achieved throughput, failures and CPU/RSS for both repetitions. A difference
in this local fixture is not a universal framework ranking or maximum capacity.

Preserve [ADR 0001](adr/0001-separation-of-concerns.md) and
[ADR 0006](adr/0006-handler-first-mvc-optional.md). `App::get` delegates to native
Axum routing; plaintext/JSON use identical handlers. SSR compares Yetlet's
component/render/response pipeline with independent handwritten escaped HTML.
Axum does not supply a native template engine. Label this **render pipeline
versus direct rendering**, not pure router overhead or a template-engine contest.
The native baseline must not call Yetlet's HTML renderer or escape function.

These workloads are inspired by the [TechEmpower test overview](https://github.com/TechEmpower/FrameworkBenchmarks/wiki/Project-Information-Framework-Tests-Overview).
Its JSON test requires per-request object creation/serialization; its plaintext
test includes HTTP pipelining. The [original repository](https://github.com/TechEmpower/FrameworkBenchmarks)
was archived on 2026-03-24. This k6 HTTP/1.1 keep-alive comparison does not
reproduce pipelining, database/Fortunes tests, official infrastructure or full
rules: **no certified, compliant or complete TechEmpower result** is claimed.

[YT-141](YT-141-PERFORMANCE-VALIDATION.md) and
[YT-142](YT-142-SATURATION-RECOVERY.md) remain accepted at their existing scopes;
their passing/failed measurements are not matched baselines for this synthetic
app. Do not repeat their workloads or the older production-profile proofs.
PostgreSQL is the requested next step after this comparison, outside YT-143;
no database, cloud execution, product optimization, Core/runtime change,
dependency upgrade, public release, commit or push is selected here.

## Smallest source and dependency surface

Bind baseline HEAD `bee5565012604d5ae6792c37e0cdc343d2bb8498` and the recorded
clean checkout before these documentation edits. The inspected seams are
`crates/yetlet-server-axum/src/{app,response,document}.rs`, its `src/lib.rs`,
`crates/yetlet-html/src/lib.rs`, root manifests/lock/toolchain and the existing
standalone `examples/production-starter/Cargo.toml` pattern.
`examples/render-bench` measures browser rendering strategies and is not an
HTTP comparison fixture; leave it unchanged.

After independent contract PASS, add only standalone
`examples/axum-workload-comparison/{Cargo.toml,Cargo.lock,README.md,src/main.rs,workload.js}`.
An empty `[workspace]` with resolver 3 isolates resolution from the root
workspace; preserve root membership and `Cargo.lock`. One binary selects
`axum` or `yetlet` at startup and exposes the same three routes; no dispatch
branch is added inside a measured handler. Native registration uses
`Router::new().route(path, get(handler))`; Yetlet uses
`App::new().get(path, handler).into_router()`. Both finish with the same
`axum::serve`, listener, runtime and response handling. The shared executable
links the same dependencies in both modes: this design cannot compare binary
size or dependency footprint.

Use only Axum, Tokio, Serde/Serde JSON, Yetlet Core and the Axum adapter; a
test-only Tower utility may exercise the new routes in process. No new
templating library, middleware/provider, custom transport, scheduler, xtask
integration or reusable runner/validator framework is needed. Route tests stay
in the sample source. The public synthetic app intentionally installs no
Auth/Session/rate-limit/database capability; bind it to disposable loopback
only. It has no private data or mutation route. Do not modify the protected
Starter or add a benchmark bypass to any product.

Pin direct registry versions to the inspected root-lock resolutions: Axum
0.8.9, Tokio 1.53.2, Serde 1.0.229 and Serde JSON 1.0.151; Tower 0.5.3 if used
for tests. Freeze the standalone lock and resolved shared HTTP dependencies
before build. The inspected root has Hyper 1.12.0 / Hyper-util 0.1.21; use
those versions in the sample's lock as well, without adding runtime libraries
just to pin transitives. Any unavailable resolution stops this stage for a
contract amendment, rather than an implicit upgrade or alternate baseline.
Both modes share exactly one compiled dependency graph and feature set.

Build once with repository Rust/Cargo 1.99.0, native host target,
`cargo build --release --locked --manifest-path examples/axum-workload-comparison/Cargo.toml`
and a task-private `CARGO_TARGET_DIR`. Use Cargo's default release profile
(opt-level 3, no LTO, 16 codegen units, unwind); unset `RUSTFLAGS`, encoded
flags, Cargo profile overrides and inherited app/proxy settings. No native CPU
tuning, PGO, alternate allocator, stripping or per-mode build is selected.
Record compiler/Cargo versions, host target, environment, exact argv, source/
lock/executable SHA-256, resolved version/feature identities and build result.
Recheck the relevant inputs after execution. Resolution/build evidence is new
sample evidence; do not rerun old workspace metadata or unrelated tests.

## Exact workload and response contract

Every attempt is one GET, with no request body, cookies, authorization, Origin,
redirects, retries, compression or application logging per request. Use
HTTP/1.1 over plain loopback TCP, k6 connection reuse within each invocation,
no HTTP/2/TLS/pipelining/proxy. Verify `response.proto` is HTTP/1.1. Match request
headers between modes, including `Accept` and `Accept-Encoding: identity`.

| Route | Per-request work | Exact success contract |
| --- | --- | --- |
| `/plaintext` | Same ordinary handler returns native `&'static str` in both modes | HTTP 200; `text/plain; charset=utf-8`; UTF-8 `Hello, World!`, 13 bytes, no newline |
| `/json` | Construct a fresh `Message { message: "Hello, World!" }` and return native `Json<Message>`; Serde serializes on each response | HTTP 200; `application/json`; UTF-8 `{"message":"Hello, World!"}`, 27 bytes, no newline |
| `/ssr/{id}` | Same native `Path<u8>` extraction and fixture creation; IDs 1–20 formatted as two decimal digits `NN`; construct/render the entire document on every call | HTTP 200; `text/html; charset=utf-8`; exact 428-byte UTF-8 document below and strict CSP |

For SSR, derive these raw values anew from the parsed request ID:

- Title: `YT143 NN <&> "quoted" 'title' café`.
- Main `title` attribute: `Item NN <&> "quoted" 'attribute' café`.
- Heading text: `Item NN <&> "quoted" 'text' café`.
- Main `data-id`: `NN`; paragraph text: `Dynamic request NN`.

The Yetlet handler builds a fresh `Element`/`View` tree and returns
`Page::with_metadata` with a fresh title and `LanguageTag::new("en")` through
the existing metadata API. The independent Axum handler builds a fresh String
using fixed structural HTML and its own small character-wise escape function:
`&` -> `&amp;`, `<` -> `&lt;`, `>` -> `&gt;`, `"` -> `&quot;`, `'` -> `&#39;`.
Apply it to every dynamic title/text/quoted-attribute value. Tags, attribute
names and language are fixed, so no untrusted structural interpolation is
needed. Do not cache full HTML, a pre-rendered fixture, escaped dynamic strings,
JSON buffers, response objects or a prebuilt View tree in either mode. Shared
raw constants and a client-side expected-body fixture are permitted.

For ID 1, the complete body is this single line, without a trailing newline;
substitute only the five `01` occurrences for other IDs:

```html
<!doctype html><html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width, initial-scale=1"><title>YT143 01 &lt;&amp;&gt; &quot;quoted&quot; &#39;title&#39; café</title></head><body><main data-id="01" title="Item 01 &lt;&amp;&gt; &quot;quoted&quot; &#39;attribute&#39; café"><h1>Item 01 &lt;&amp;&gt; &quot;quoted&quot; &#39;text&#39; café</h1><p>Dynamic request 01</p></main></body></html>
```

Use the existing strict policy verbatim on SSR responses in both modes:

```text
default-src 'none'; script-src 'self'; style-src 'self'; connect-src 'self'; img-src 'self'; base-uri 'none'; object-src 'none'; form-action 'self'; frame-ancestors 'none'
```

Require equal status, body bytes, Content-Type, Content-Length and SSR CSP;
no Content-Encoding, Set-Cookie or added Server header. The default HTTP Date
header may differ with time; record it without requiring equal values. No CSP
is added to plaintext/JSON in either mode. Record the actual complete header
set and body lengths before load; preserve default keep-alive behavior and
match any additional generated headers. Do not silently normalize unequal
payloads. These checks prove HTTP output/escaping/CSP-header equality; they do
not establish browser CSP enforcement, accessibility qualification or Wasm
behavior, which remain unmeasured.

## Fixed paired experiment and finite resources

Use the same official k6 2.3.0 Linux-amd64 release accepted in YT-141/142,
archive SHA-256
`39c3117b6af817592dcd0ce4242105c0a7af10948c2a425306f0be8f7a8a8ab1`.
Reuse its retained official release/checksum evidence; if fetching the removed
archive again, verify that digest and executable identity/version and retain
the acquisition command/time/exit/stdout/stderr. Keep it task-local, disable
usage reporting, use no cloud output and make no global installation.
The new small workload-specific script uses k6's native
[`constant-arrival-rate`](https://grafana.com/docs/k6/latest/using-k6/scenarios/executors/constant-arrival-rate/)
executor, not a new scheduler or an extension of the Starter's authenticated
script. One iteration makes one complete GET and checks its expected output.

Run two repetitions. Within each repetition, visit plaintext, JSON, then SSR;
for each workload visit **1,000/s then 4,000/s**, ten scheduled seconds each.
Every workload/rate pair is adjacent between modes: repetition 1 runs Axum
then Yetlet; repetition 2 runs Yetlet then Axum. This is twelve paired blocks,
24 measured phases and **600,000 nominal measured arrivals**. It is a fixed-rate
comparison, not a capacity sweep. Complete the planned blocks even if an
investigation point fails, unless a correctness, safety, evidence or resource
stop condition applies; retain every failed point. No new rate, favorable
replay, tuning, larger resource pool or post-result duration change is allowed.

Each of the 24 phases gets a fresh process in the selected mode on the same
owned high loopback port, with no other comparison server alive. Before its
measured invocation, validate the three response types including all 20 SSR
IDs, then run a separate two-second 250/s warmup of that phase's workload;
exclude these probes and warmup from measured counts. Wait for warmup exit/
drain and one second idle. Use 64 preallocated=max VUs for both warmup and load,
`timeUnit=1s`, eight-second request timeout and eight-second graceful stop,
no sleeps/retries/redirects. Separate invocations have fresh connections;
warmup primes application/cache work, not the later connection pool. Cycle SSR
IDs deterministically as `1 + scenario.iterationInTest % 20`, independently in
each invocation; retain actual per-ID counts. After load drain, stop the app
natively and verify exit/listener closure before starting its paired mode.

| Owned resource | Fixed bound |
| --- | --- |
| Server | Tokio multithread runtime with exactly two workers; logical CPUs 12–13; CPUQuota 200%; MemoryMax 256 MiB; TasksMax 128; LimitNOFILE 512; Nice 10; Restart=no; graceful stop at most five seconds |
| k6 plus monitor | Different logical CPUs 14–19; CPUQuota 600%; MemoryMax 1 GiB; TasksMax 128; LimitNOFILE 512; Nice 10; Restart=no; 64 fixed VUs |
| Run files | Private task directory, local ext4; 512 MiB per file and 4 GiB total retained output; at least 8 GiB free before startup |
| Time | At most 1,200 s for all correctness/warmup/measurement/stop/cleanup work; separate acquisition 120 s and new build/checks 600 s; total at most 1,920 s; no automatic restart |

The retained YT-142 paired raw file is 71,315,952 bytes for 17,502 completed
attempts, about 4 KiB per attempt. Allowing twice that for new tags/HTTP timing
points gives about 4.7 GiB for 600,000 measured plus 12,000 warmup arrivals.
The original contract incorrectly stated 300,000 measured / about 2.4 GiB and
therefore overstated the 4 GiB cap's planning headroom. The frozen cap was not
increased; actual output size and this arithmetic erratum are recorded below. Stream raw JSON
analysis sequentially and retain only needed latency/count populations in
memory; never load all raw files into the 1 GiB client/monitor allocation.

Use owned transient units/process limits and bounded command timeouts with an
outer watchdog; no new lifecycle framework is a deliverable. Record the actual
CPU model, logical/sibling/cache topology, allowed affinity, governor if readable,
host load, RAM, kernel, mount/space and cgroup limits before execution. The
initial host allows CPUs 0–19. The selected server CPUs 12–13 map to distinct
physical cores 6–7; client CPUs 14–19 map to distinct cores 8–13. Confirm this
topology and allowed placement before load; record any hybrid core classes. If
unavailable, stop rather than choose placement after results. Distinct cores
can still share caches and host resources; disclose shared-host, monitor and
output interference. No existing service is stopped or reconfigured.
One active comparison server avoids competing server modes; this does not
isolate the host from unrelated users. Do not infer server capacity when the
generator, host, output or resource limits constrain the experiment.

## Measurement, comparison and stop rules

Freeze the reviewed contract, source/lock/binary/tool/script identities, fixture,
full commands, non-secret environment, phase order and limits before startup.
Retain per-attempt raw k6 native JSON, summaries and stdout/stderr, including
nonzero exits. Record phase/workload/mode/repetition/rate/ID, actual start and
completion times, status, protocol, error code and validity without emitting
response bodies. Keep planned, started, completed, valid-200, dropped and
interrupted counts separate, including native endpoint boundary extras.
Report scheduled-window completions/rate separately from throughput through
the final completion; configured arrival rate is not achieved throughput.

Primary HTTP latency is k6's floating-millisecond `response.timings.duration`
and native `http_req_duration`: sending + waiting + receiving, excluding
DNS/connect/blocked time. See [Response timings](https://grafana.com/docs/k6/latest/javascript-api/k6-http/response/)
and [metric definitions](https://grafana.com/docs/k6/latest/using-k6/metrics/reference/).
Retain nearest-rank p50/p95/p99 for all completed attempts, all HTTP responses
and exact valid-200 responses separately; transport failures can carry zero/
incomplete HTTP timings and must remain separately counted. Also retain elapsed
full-call wall time around GET via `Date.now()` for all completions and valid
200s, explicitly limited to millisecond resolution and wall-clock adjustments.
Never derive submillisecond overhead from those integer full-call values.
HTTP timings have different scope and uncalibrated instrument precision; extra
decimal digits alone do not establish a meaningful difference. Dropped or
interrupted attempts have no completed latency sample. Native HTTP and custom
full-call populations must not be substituted for one another.

Report timeout/transport errors, 429/503/504, other unexpected statuses,
invalid-200 bodies/headers/protocol, drops and interruptions rather than hide
them in successful-only percentiles. Retain approximately one-second timestamped
app and actual post-exec client PID CPU/RSS/thread/FD samples plus cgroup CPU,
throttling, memory/events and output size. Publish the actual inner measured
sample intervals/gaps; do not dilute measured CPU with warmup/startup or use
fork-inherited child high-water memory as isolated k6 RSS. One core is 100%;
report process and cgroup populations distinctly. No app telemetry middleware
or new metrics endpoint is required for this synthetic comparison.

A comparable phase needs correct output throughout, zero error/drop/interruption,
at least 95% of nominal arrivals completed validly inside the scheduled window,
complete generation/samples and no resource/output limit. Demonstrate client
headroom with zero client cgroup throttling/OOM, no task/FD cap, sampled client
CPU below 80% of its 600% allocation and peak client-cgroup memory below 80% of
1 GiB. Report server throttling and CPU saturation; do not turn a failed point
into an application-only limit without evidence. Missing client/sample evidence
makes comparability unverified, not a passing zero. There is no consumer latency
SLO or requirement that Yetlet beat Axum.

Show every point and both repetitions. Only calculate a Yetlet/Axum ratio for
the same workload/rate/repetition with both sides comparable, matching payload,
build/runtime/resources and timing definition. Label ratios explicitly: latency
above 1 means slower, valid throughput above 1 means more completed work. Fixed
offered rates tend to fix throughput; such ratios are not capacity speedups.
Report CPU/memory separately and do not divide CPU samples from unequal windows
without disclosing the interval. Do not pool repetitions to conceal reversal,
claim statistical significance from two pairs, manufacture an exact percentage
when resolution/noise dominates or divide by zero. Inconclusive is a valid
answer. This short shared-host experiment does not measure sustained/maximum
capacity, cold starts, TLS/HTTP2/pipelining, remote-network cost, full apps,
browser interactivity, security middleware, storage or production performance.

Correctness mismatch, source drift, missing required evidence, watchdog,
resource/output cap or unsafe state stops further execution and retains partial
evidence; finish owned cleanup. Ordinary failed performance/comparability points
remain in the planned sequence, without optimization or replay. A tool/setup
failure is recorded separately from product behavior; apply the existing
materiality/repair limits before any correction. Do not create a new validator,
certification wrapper or proof framework in response to an evidence failure.

## Minimum verification, acceptance and next step

After contract PASS, run only sample-local focused tests: both registration
paths' exact plaintext/JSON response; all 20 independently specified SSR bodies,
UTF-8 byte lengths and security/content headers; distinct ID output and all five
escaping characters in title/text/attribute contexts. Expected output is literal
fixture data, not generated by either server renderer. Source inspection must
confirm fresh JSON/SSR construction, independent native escaping, no cache and
no measured-mode dispatch. Verify sample formatting and its locked release
build. Run `cargo xtask check docs` and `git diff --check` for changed docs.
No old full workspace tests, browser matrix or historical workload rerun is
required. Report compile, HTTP/CSP-header checks and unrun browser enforcement
separately. No changes to Yetlet Core/HTML/adapter or existing applications are
authorized to make this sample pass.

Retain exact new commands/exits/timestamps, correctness results, all raw measured
and failed runs, source/build/tool receipts, per-point calculations, resource
samples and limitations in
`/tmp/yetlet-yt143-axum-comparison-20261023-tpmphwa7`.
After drain, reap owned app/client/monitor/watchdog children, observe actual unit
state, verify owned descriptors/listener gone, stop/remove only owned units and
disposable executable/tool files, and retain evidence. No existing service or
data is touched. Missing cleanup proof prevents Done.

Independent contract review precedes implementation; a separate independent
final Reviewer must PASS the actual changed source, fixed experiment, retained
failures, conclusions and cleanup before YT-143 becomes Done. A valid comparison
may conclude no resolved difference or client limitation; workload failure is
not silently converted to PASS. The actual checkpoint and limits follow below.
After this bounded comparison and final review, the next user-requested work is
a separate PostgreSQL integration contract; do not start it under YT-143.
YT-096 owner decisions and YT-020 release gates remain unchanged.


## Actual implementation, deviations and observations

Initial contract Reviewer `/root/yt143_axum_contract_review_20261023` returned
PASS, zero issues and required followups. The immutable original reviewed
contract and `preexecution.json` are retained separately from this updated
record. The standalone sample implements the one-binary startup selection,
identical native plaintext/JSON handlers and independent escaped direct-HTML
baseline alongside fresh Yetlet View/Page SSR. Root manifests/lock, existing
applications and Core/HTML/adapter source are unchanged.

Two sample-local tests PASS, covering 44 exact responses, including all 20 SSR
IDs in both modes. Sample formatting and the offline locked release build PASS.
Rust 1.99.0 and the declared dependency pins, including Hyper 1.12.0 /
Hyper-util 0.1.21, were used. The acquired official k6 archive matched the
accepted digest; executable SHA-256 was
`e001bbf39ba67424a85c9c8516440ba5c73ecf7a6021be96c76194380c15f260`.
Commands, environments, UTC/epoch times, exits and separate stdout/stderr files
are retained. No old test/workload or browser matrix was rerun. Compile and
HTTP/CSP-header checks passed; browser CSP enforcement remains unverified.

**Arithmetic erratum:** the explicit fixed 24-phase plan is
2 repetitions × 3 workloads × 2 modes × (1,000 + 4,000)/s × 10 seconds =
**600,000 nominal measured arrivals**, plus 12,000 warmup arrivals. The initial
contract incorrectly stated 300,000; no phase was added or replayed to correct
that text. `nominal-arithmetic-erratum.json` preserves the error, corrected
formula and consequence for the original output estimate. The original 4 GiB
aggregate / 512 MiB-file bounds stayed fixed.

**One canonical resource correction:** the initial setup checked the build
path's ext4 mount, while actual raw output went to `/tmp`, a tmpfs mount.
Retained tmpfs bytes accumulated in the client cgroup despite file-cache advice.
The original sampler omitted `memory.stat`; its exact historic anon/file/slab
breakdown cannot be reconstructed. The mount receipt, growing retained bytes
and low process RSS establish the output-placement error without attributing
that pressure to the Rust server. All repetition-1 points are retained as
**descriptive observations under a storage deviation**, not comparable accepted
points.

Client headroom first fell below the declared 80% at phase 7, 12:26:50.590 UTC.
The first captured memory-limit event was phase 8 at 12:27:07.459 UTC: `max=54`,
`sock_throttled=1,734`, zero OOM/kill. The initial sampler checked OOM but omitted
stopping on `memory.events.max`; phases 9–12 therefore ran after the cap.
External SIGINT at 12:28:09.621 UTC stopped that execution. These late points,
56 dropped arrivals and the stop delay remain failures, not product capacity
results. Final original-client events were `max=1,985`,
`sock_throttled=10,056`, zero OOM/kill and zero CPU throttling. Its sampled
cgroup peak reached 1 GiB while k6/monitor RSS stayed much smaller.

The twelfth measured k6 invocation completed and its native raw/summary and
post-wait sample exist, but SIGINT interrupted subsequent parsing before its
command exit receipt was saved. That exit is **unavailable**, not inferred zero.
Its 40,000 native/custom completions agree; no HTTP attempt was interrupted.
The original monitor's recorded status was 2 after SIGINT. Original script,
results, projection and command history were preserved before correction.

The parent authorized one smallest in-place harness correction, independently
accepted in `correction-delta-review.json`: only the never-started phases 13–24
wrote output under verified ext4
`target/yt143-axum-comparison-tpmphwa7/retained-evidence`.
The corrected seam checks that actual output mount, captures `memory.stat`,
stops on memory-limit/socket/FD events and persists invocation exit receipts
before parsing. It preserves original identities/history and the absolute
12:45:12.818 UTC deadline. No source, tool, binary, rate, VU, CPU/memory limit or
output cap changed; phases 1–12 were not replayed. This is one harness failure
and correction, zero product repairs or operational retries, not a new run or
benchmark platform. No test or build was repeated after it.

The full observed interval was 12:25:12.818–12:36:55.219 UTC; the corrected
continuation began 12:33:54.987 UTC. Both sets contain each predeclared phase
once. The corrected second repetition has 300,006 valid measured HTTP 200s,
one dropped arrival and zero interrupted, transport, timeout, unexpected-status
or invalid-200 outcome. The original repetition has 299,963 valid measured
200s and 56 drops. Combined measured counts are 599,969 started/native/custom
completed/valid-200, 57 drops and zero interruptions; **do not pool the storage
populations for a performance conclusion**. Warmups contain 12,015 valid 200s;
528 separate preload responses matched exact bodies, UTF-8 lengths and headers.
Non-Date headers match across all fixture paths and modes; HTTP/1.1 and strict
SSR CSP were retained.

The following nearest-rank HTTP timings cover all completed attempts, all HTTP
responses and valid-200 responses equally because every completed attempt was
an exact 200. Separate populations and full-call wall-clock timings remain in
`reporting-projection.json`; dropped arrivals have no latency sample. Ratios
are Yetlet/Axum p95 for comparable second-repetition points only; values above
1 mean higher observed latency. They are descriptive paired observations,
not a significant percentage overhead or maximum-throughput claim.

| Rep | Workload / offered per second | Axum valid / drops | Yetlet valid / drops | Axum p50/p95/p99 ms | Yetlet p50/p95/p99 ms | Y/A p95 | Scope |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | plaintext / 1,000 | 10,001 / 0 | 10,001 / 0 | 0.257/0.406/1.733 | 0.259/0.404/1.861 | unavailable | tmpfs deviation |
| 1 | plaintext / 4,000 | 40,000 / 0 | 40,002 / 0 | 0.102/0.379/2.068 | 0.103/0.386/2.089 | unavailable | tmpfs deviation |
| 1 | json / 1,000 | 10,000 / 0 | 10,000 / 0 | 0.261/0.393/1.552 | 0.259/0.398/1.838 | unavailable | tmpfs deviation |
| 1 | json / 4,000 | 40,010 / 0 | 39,989 / 13 | 0.103/0.404/2.478 | 0.104/0.373/2.165 | unavailable | tmpfs deviation |
| 1 | ssr / 1,000 | 10,001 / 0 | 10,001 / 0 | 0.285/0.603/2.722 | 0.288/0.631/2.710 | unavailable | tmpfs deviation |
| 1 | ssr / 4,000 | 39,958 / 43 | 40,000 / 0 | 0.099/0.627/2.904 | 0.100/0.480/2.486 | unavailable | tmpfs deviation |
| 2 | plaintext / 1,000 | 10,000 / 0 | 10,001 / 0 | 0.260/0.382/1.489 | 0.262/0.406/1.556 | 1.061 | comparable |
| 2 | plaintext / 4,000 | 39,996 / 0 | 40,004 / 0 | 0.101/0.333/1.901 | 0.102/0.364/2.099 | 1.095 | comparable |
| 2 | json / 1,000 | 10,001 / 0 | 10,001 / 0 | 0.260/0.398/1.617 | 0.261/0.407/1.811 | 1.022 | comparable |
| 2 | json / 4,000 | 40,001 / 0 | 40,002 / 0 | 0.102/0.381/2.324 | 0.102/0.369/2.120 | 0.969 | comparable |
| 2 | ssr / 1,000 | 10,001 / 0 | 10,001 / 0 | 0.285/0.616/2.585 | 0.288/0.627/2.447 | 1.018 | comparable |
| 2 | ssr / 4,000 | 39,995 / 1 | 40,003 / 0 | 0.099/0.458/2.561 | 0.100/0.488/2.637 | unavailable | FAIL: one Axum drop |

The second-repetition SSR 4,000/s pair is **noncomparable** because native Axum
had one dropped arrival; it was not replayed. Its 39,995 starts plus one drop
also differ from nominal 40,000, a retained scheduling residual. More generally,
nominal, actual, dropped and boundary counts are distinct: original/corrected
repetitions had 51/28 actual starts outside their half-open scheduled windows,
and nominal minus started minus dropped totals of -19/-7. These are not
invented responses or interruptions. Per-point start/completion windows,
counts, native endpoint extras and both scheduled-window and through-final-
completion rate denominators remain in the raw/reporting projection.

Corrected valid completions inside the ten-second window were about 1,000/s or
3,999/s at the respective offered points; they do not establish capacity.
Scheduled-window throughput ratios at comparable points were approximately
1.000, as expected with fixed arrivals. Corrected app mean CPU was 6.67–11.55%
of one core and k6 mean CPU 126.17–234.58%; process RSS peaks were 6.09/86.01 MiB.
Client-cgroup cumulative memory peak was 274.12 MiB, with zero memory/task/
socket-limit events, OOM or CPU throttling. Inner measured resource intervals
spanned 8.187–8.260 seconds, with maximum sample gap 1.037 seconds; sampling
misses boundary slivers and possible peaks. Per-point CPU, RSS, FD/thread,
cgroup and `memory.stat` observations are retained, without warmup dilution.

The host is a shared Intel Core i5-13600K with the recorded powersave governor.
Selected logical CPUs 12–19 are distinct single-threaded cores in the observed
topology (lscpu compact core IDs 6–13; sysfs IDs 24–31), max 3.9 GHz; no core
class is inferred from an unavailable `core_type`. No host setting changed.
Only the second repetition has the declared output placement, so two valid
order-balanced repetitions were **not** obtained. The small observed latency
ratios, mixed direction and short shared-host fixture do not resolve a stable
Yetlet overhead. SSR compares component/render/response work with direct
escaped rendering. Integer full-call p50 values include zero milliseconds;
those values cannot support submillisecond claims.

Raw native files total 3,192,004,841 bytes across both populations, below the
unchanged 4 GiB aggregate cap; all files stayed below 512 MiB. There was no
output-cap expansion or favorable replay. All 24 native app stops recorded
`stopped`, closed listeners and absent app PIDs; final client exited with
code/status 1/0 (normal exit). At 12:37:45.013 UTC, all 74 app/k6/monitor PIDs
were absent, all 25 owned units inactive/MainPID 0 with no cgroup, and the
loopback listener closed, within the original 20-minute budget. Owned build,
executables, tool archive/extraction and private home were removed; the ext4
raw output directory and evidence receipts remain intentionally retained.
Existing services/data were untouched. Before/after product input hashes match;
only the reviewed one-off harness seam changed during execution.

`checks.jsonl`, separate check logs, `preexecution.json`, tool receipts,
`output-mount-cause.json`, correction/review artifacts, original and current
`runtime-result.json`/reporting snapshots, native raw/summary/stdout/stderr,
`samples.jsonl`, `partial-record-recovery.json`, `resource-stop.json`,
`reporting-projection.json`, command histories and `cleanup.json` retain the
bounded evidence. Final independent Reviewer
`/root/yt143_axum_final_review_20261023` returned **PASS**, zero issues and
required followups; all nine reviewed file hashes matched. YT-143 is **Done**
at this bounded comparison scope. The next separately requested task is a PostgreSQL contract;
no PostgreSQL, cloud, commit, push or release work was performed.
