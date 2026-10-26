# YT-157 — Medium HTTP p95 attribution

Date: 2026-10-24 (Asia/Taipei). Status: **Done at bounded attribution scope; independent final Reviewer PASS.**

## Authority and bounded question

Evidence: `target/yt157-medium-http-tail-iw9_x70a/evidence`.

The user selects contract-first minimum attribution of the medium 1,000 req/s
p95 increase, reusing YT-156, retaining failures, preserving API/output/security,
no optimization and independent final Reviewer PASS before Done. New work must
not be committed/pushed. YT-156 was separately committed with strict/builtin
Guard at `25a0cc6`; the current checkout is this task's clean baseline.

Preserve [ADR 0001](adr/0001-separation-of-concerns.md),
[ADR 0003](adr/0003-renderers-and-loading.md) and
[ADR 0006](adr/0006-handler-first-mvc-optional.md). Product libraries, public APIs,
root/sample dependencies/locks, normal SSR fixture/handler/headers, body release,
validation/escaping/URL rules and browser/default graphs remain unchanged. No
renderer optimization, provider/cache/transport change, dependency, telemetry
package, profiler installation, CI or reusable measurement framework.

[YT-156](YT-156-HTTP-SSR-COMPARISON.md)'s medium high-rate p95 is reference/current
1.208850/1.445209 ms in run 1 and 1.164460/1.390802 ms in run 2: +19.55/+19.44%.
These are accepted historical populations, not results to overwrite or replay.
The question is which observable client/server boundary contains the tail, and
whether application pipeline / construction timing supports a server-work
explanation versus unobserved scheduling/transport/context. A bounded unresolved
cause is valid; do not force unique causality or optimize to obtain a good result.

## First use the retained native components

Read only the four existing medium/1,000 measured raw files, their native receipts,
resource records and source identities. All native timing components already
exist; no original YT-156 script/test/build/live group is rerun. Group native
HTTP duration/sending/waiting/receiving/blocked/connecting by exact native event
time plus available start/anchor/status/method/protocol metadata, validating
uniqueness/counts rather than guessing ambiguous joins. Per response reconcile
sending+waiting+receiving to duration with stated floating tolerance ≤1e-6 ms;
blocked/connecting are separate and not added twice. Report unjoinable groups
explicitly, without excluded-outlier substitution or favorable replay.

For each retained version/run compute component p50/p95/p99, the components on
responses at/above that population's nearest-rank HTTP p95, component dominance
and ten declared one-second start-time bins (boundary starts separate). Native
response time groups allow within-response decomposition; do not add marginal
percentiles. Keep all 40,005 retained measured completions and old unfavorable
values. Analyze old data as old data, without promoting a new diagnostic caller
into a historical denominator. Existing resource sampling is coarse; no per-request
server stage or hardware/frequency cause follows from native waiting alone.

## Minimum new sampled application probe

Only the existing standalone HTTP sample gains default-off `ssr-stage-probe = []`,
private `src/bin/ssr/stage_probe.rs`, cfg-gated hooks in `src/bin/ssr.rs` and
`ssr-stage-probe.js`. Normal code stays text-identical apart from cfg/module /
startup selection and old-test cfg; the old binary/test/script is not requalified.
No crate/public API/runtime feature or dependency is added to Yetlet itself.

Use native Axum middleware/typed State and a private request Extension; keep
actual Page response conversion and its CSP/headers/body unchanged. The diagnostic
handler calls the same byte-identical `page(128)` and returns actual Page.
The normal public route still accepts only the existing three size names; this
probe loads medium only. Its synthetic `X-Yt157-Id` request header carries bounded
`w:<id>` or `m:<id>` (unsigned integer <20,000) solely for local correlation,
never in response headers/body, logs from an actual app or product API. Preload
requests without that header remain valid and produce no stage sample.

Sample exactly IDs divisible by **32**. Selected requests record middleware
`Next::run` wall duration (routing/extraction/handler/actual IntoResponse until
response object ready) and nested fresh metadata/View/Page construction duration.
Capture both only on selected requests using Instant; keep handler output/lifetime
and body validation/escaping unchanged. No copied renderer, timed Page substitute,
response-body wrapper, custom transport or per-request stdout. The unmeasured
remaining pipeline includes extraction/response generation and observer overhead;
it is not pure serializer time. No header exposes timing to clients.

Use a preallocated bounded 1,024-record Vec in private State, with sampled
per-request shared record storage and short locks; unselected requests avoid
per-request record allocation and stage clocks. Retain missing/duplicate IDs and
overflow/drop counters; cap overflow is an evidence stop, not silent truncation.
Dump bounded JSON records/counters only after native server drain, before the
existing stopped event. One new focused test verifies the diagnostic medium
route's exact independent body/content/security headers plus one correlated
selected record with nested timing, without rerunning original route/safety tests
or writing harness self-tests.

These clocks, extractors, request header/State/Extension and sampled locks can
perturb timing. Diagnostic binaries are distinct consumers, not an uninstrumented
repeat or certification of YT-156. Do not infer original request CPU shares,
subtract independent medians or treat probe durations as exact historical costs.

## Sources and new fixed population

Copy accepted YT-156 source-reference/current directories into task-owned sources,
verify accepted product hashes/links, and overlay identical final HTTP sample
bytes. Do not edit the originals or checkout/reset/create worktrees. Their actual
product refs remain `90f83867` and `0580a0a`; current products match this checkout.
Only accepted serializer/document functions and Core test-only code differ.
Reuse literal 31,063-byte medium document and exact strict CSP/header evidence.
Two new offline/locked default-release builds select only ssr-stage-probe, two
jobs, default System allocator/profile and unchanged Rust/Cargo1.99.0. Bind new
source/lock/features/actual library artifacts and binaries. Compile one new
focused test with no-run JSON and execute only emitted `yt157_ --test-threads=1`;
one focused rustfmt check on changed Rust. No counted build or normal/old test.

Use the exact retained official k6 2.3.0 executable SHA-256
`e001bbf39ba67424a85c9c8516440ba5c73ecf7a6021be96c76194380c15f260`,
reusing its acquisition/version proof; no download/install is needed while present.
Adapt the existing script's metadata/correctness/native timing mechanism for the
new request IDs; the independent expected body is prepared outside iterations.
One GET per iteration, native constant-arrival-rate executor, no sleeps, retries,
redirects, cookies/auth, compression, TLS/HTTP2/pipelining or external network.
HTTP/1.1 loopback keep-alive, Accept text/html / Accept-Encoding identity.

Exactly **four new diagnostic phases**: reference/1, current/1, current/2,
reference/2. All are medium 1,000/s for ten scheduled seconds, 64 fixed VUs,
eight-second request timeout/graceful stop. Each fresh server gets one medium
preload, separate two-second125/s warmup, drain/one-second idle, then measured
invocation. That is **40,000 nominal measured +1,000 nominal warmup arrivals**;
ID sequences restart per invocation and phase prefix prevents cross-phase joins.
No small/large/250/s, alternate-rate, old accepted command or repeated point.

Keep YT-156 isolation: server two workers/CPUs12–13/CPUQuota200%/MemoryMax256MiB,
client plus monitor CPUs14–19/600%/1GiB, both TasksMax128/NOFILE512/Nice10/no restart,
server RuntimeMax45s/native stop≤5s and client RuntimeMax600s. Record current
allowed distinct-core placement/ancestors/governor/load/mount/flags before
assertions; no host/governor/allocator/compiler/quota tuning or existing-service
change. Task-owned ext4, ≥8GiB free, ≤128MiB/file /512MiB total evidence;40,000
new measured attempts at8KiB plus warmups leave bounded headroom. Existing raw
files remain outside the new output cap and untouched. Completed owned raw-file
cache advice only, no global/server cache flushing.

Stage2 has one fixed **45-minute** UTC/monotonic deadline before implementation,
never renewed; each build/test ≤600s, runtime/analysis/cleanup ≤600s inside it.
Reuse the existing finite receipt/native unit/monitor/arithmetic seams task-locally.
No validator, proof machinery, new runner framework, startup retry matrix or new
budget after a fault. Preserve failures, classify materiality before any smallest
correction, and never replay passed work or a failed load for favorable results.

## Joins, reporting and limits

Client metadata carries phase/id, start/anchor/completion times and response
validity. Native event-time component groups are joined to custom completion
using the unique phase/id. Selected server samples match exactly one valid client
request by phase/id and the declared modulo32 rule; report absent/duplicates /
extra/overflow rather than silently selecting survivors. Keep warmups separate.
All native/custom durations, counts, schedules and body/header/protocol checks
must reconcile before interpreting the new probe. Native/HTTP/body mismatches,
source drift, observer loss, resource/output/watchdog or missing-required-evidence
conditions stop further execution and retain partial data; ordinary performance /
comparability failures stay in the fixed sequence without tuning/replay.

For each diagnostic version/run retain component, complete HTTP and selected
server pipeline/construction distributions, both repetitions, all one-second
bins and the sampled tail membership / joint observations. Pipeline is wall time,
not CPU; creation is nested, not an additive independent percentile. Compare
selected pipeline times for HTTP-tail requests with non-tail selected requests
without claiming random independent trials or causal subtraction of clocks from
different processes. Report when a 1/32 sample leaves too few tail observations.
No required direction, significant difference, universal overhead or unique
hardware/allocator/transport cause. An unresolved bounded explanation is accepted
only with honest actual observations and explicit remaining uncertainty.

Keep nominal/started/completed/valid200, drops, interruptions, errors/statuses,
boundary extras and window/drain throughput distinct. All-error and valid-200
HTTP p50/p95/p99 remain separate if failures occur. Integer-ms full-call time
has coarse resolution; native HTTP duration excludes blocked/connect. Record
roughly one-second actual app/k6/monitor CPU/RSS/threads/FD/cgroup samples,
inner intervals/gaps and headroom. Measured comparability requires exit0, exact
output, no errors/drops/interruption, ≥95% nominal valid window completions,
consistent joins/counts, ≥2 inner samples/gap≤2s, client peak intervalCPU<480%,
cumulative memory<819.2MiB, zero client throttle/memory/task/FD-limit events.
Warmup success is native priming only; do not hide short-window sampling limits.
Only comparable diagnostic pairs get current/reference ratios; keep historical
and diagnostic populations distinct. No capacity/SLO/production claim.

Cleanup must stop/reap only owned apps/client/monitor, retain native exits and
sampled dump, verify unit inactive/MainPID0, originally captured cgroup paths and
PIDs absent, listener closed. Exited units may clear ControlGroup; do not read
root cgroup as an owned final endpoint. No server/infrastructure remains live.
Retain inert artifacts for independent review and future guarded commit.

## Acceptance and path boundary

Exactly nine repository paths may change: this report, docs/BACKLOG.md,
docs/ROADMAP.md, docs/README.md; examples/axum-workload-comparison/Cargo.toml,
README.md, src/bin/ssr.rs, new src/bin/ssr/stage_probe.rs and new
ssr-stage-probe.js. Contract review precedes code; parent docs/whitespace checks
and independent final Reviewer PASS0 precede Done. Freeze nine source hashes,
all relevant data/history/source/executable bindings. After final PASS, only
status/reviewer references in the five documents then docs/whitespace checks may
change. No optimization, new task, commit/push or YT-096/YT-020 release decision.

## Results and bounded attribution

Contract Reviewer `/root/yt157_medium_tail_contract_review_20261024` initially
returned FAIL with one P2: the planned historical total 40,004 omitted one accepted
boundary observation. The single-token correction to **40,005** received PASS,
zero issues/required followups, before implementation. Original FAIL, hashes,
materiality note and corrected PASS remain separate; no historical test/load or
source was rerun/replaced. All 28 reused history bindings remain unchanged.

### Retained YT-156 components: where the original tail appears

All four native populations reconcile, without ambiguous or duplicate native
event groups: 10,001 / 10,001 / 10,002 / 10,001 completions, all valid 200.
Sending+waiting+receiving matches each response duration to maximum absolute
error 1.78e-15 ms (below the declared tolerance); native/custom duration multisets
match. Exact native event-time groups give per-response components. Historical
custom validity is accepted for the complete population; no ambiguous individual
native-to-custom join is invented from colliding millisecond start times.

The first table is marginal percentiles in ms, not additive CPU fractions.
The last two columns use **the same HTTP-tail responses**, at or above that
population's nearest-rank HTTP p95; each has 501 tail observations.

| Run / product | n | HTTP p95 | Sending p95 | Waiting p95 | Receiving p95 | Blocked p95 | Waiting-largest tail responses | Mean waiting / duration within tail |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 1 / reference | 10,001 | 1.208850 | 0.022885 | 1.036075 | 0.120149 | 0.006039 | 499 / 501 | 87.49% |
| 1 / current | 10,001 | 1.445209 | 0.033284 | 1.193673 | 0.174348 | 0.009024 | 497 / 501 | 82.69% |
| 2 / current | 10,002 | 1.390802 | 0.030235 | 1.135067 | 0.165975 | 0.008433 | 501 / 501 | 81.62% |
| 2 / reference | 10,001 | 1.164460 | 0.022647 | 0.992136 | 0.113306 | 0.006118 | 500 / 501 | 87.62% |

The original positive p95 difference is chiefly observed within **waiting**,
with receiving/sending differences also present. Waiting is the largest component
in 497–501/501 tail responses; this boundary includes application work plus
transport/dispatch/OS scheduling before the first response byte. It is not a
renderer timer. Blocked/connecting are outside native HTTP duration and cannot
explain its increase by being added to it. Do not add these marginal component
p95 values or their differences to derive a percentage attribution.

Ten fixed one-second start bins preserve temporal variation; boundary starts
remain separate (3/2/2/1). Every bin and component's count/mean/p50/p95/p99/min/max
is retained in `historical-component-summary.json`. HTTP p95 per bin is:

| Run / product | Seconds 0 through 9, p95 ms |
| --- | --- |
| 1 / reference | 1.608 / 0.955 / 1.246 / 1.239 / 1.217 / 1.088 / 1.105 / 1.206 / 1.376 / 1.160 |
| 1 / current | 1.678 / 1.317 / 1.385 / 1.908 / 1.304 / 1.480 / 1.369 / 1.389 / 1.342 / 1.300 |
| 2 / current | 1.946 / 1.335 / 1.357 / 1.281 / 1.402 / 1.324 / 1.453 / 1.267 / 1.312 / 1.431 |
| 2 / reference | 1.597 / 1.299 / 1.506 / 1.123 / 0.915 / 0.944 / 0.984 / 0.863 / 0.949 / 1.165 |

The difference is not confined to one omitted extreme: there is substantial
within-phase variation and multiple higher-current bins. These separate-process
bins are serial observations, not paired independent trials; no significance,
stationarity or unique scheduling/frequency cause follows.

### New application probe: four distinct diagnostic populations

The four feature-enabled diagnostic phases all ran once: **40,003 measured**
and **1,002 warmup** valid 200 completions, with zero errors, drops, interruptions
or invalid 200s. All eight k6 invocations exit0 and all four native app stops
exit0. Four medium-only live preload bodies/headers match accepted output.
Both measured pairs meet the reviewed comparability conditions. No normal SSR
binary/test/load or YT-156 accepted group was rerun.

All **1,284** server records correlate exactly to expected modulo32 client IDs:
313 measured +8 warmup per app, 1,252 measured and32 warmup total. There are zero
missing/duplicate/extra IDs, observer overflows or invalid IDs; all nested
construction clocks are positive and bounded by their pipeline durations.
Native component groups and custom HTTP durations agree per probe ID. Warmups,
boundary starts and selected/unselected requests remain distinct populations.

The next table uses all new HTTP observations and their component percentiles
(ms); sampled server stages are shown separately below.

| Run / product | n | HTTP p50 / p95 / p99 | Sending p95 | Waiting p95 | Receiving p95 | App mean CPU % | Sampled RSS MiB |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 / reference | 10,000 | 0.578 / 1.212 / 2.828 | 0.025 | 1.038 | 0.111 | 47.37 | 6.91 |
| 1 / current | 10,001 | 0.605 / 1.337 / 2.942 | 0.030 | 1.107 | 0.160 | 45.73 | 6.93 |
| 2 / current | 10,001 | 0.590 / 1.348 / 3.057 | 0.034 | 1.126 | 0.157 | 46.04 | 6.80 |
| 2 / reference | 10,001 | 0.577 / 1.227 / 2.589 | 0.025 | 1.045 | 0.118 | 47.81 | 6.84 |

HTTP p95 current/reference changes are **+10.30% / +9.91%**, still positive
in both orders but distinct from historical +19.55%/+19.44%. Observer paths,
headers, compilation/caller placement and host/request context differ; this is
not a smaller replacement denominator, proof that the original result was noise,
or an uninstrumented reproduction of the original percentage.

### Sampled server pipeline and fresh construction

Each row has 313 deterministic sampled measured requests. Pipeline clocks span
native middleware Next through actual Page IntoResponse and ready response
object; construction clocks cover fresh metadata/View/Page creation inside it.
The remaining pipeline is computed **within each request**, not by subtracting
independent percentiles. It includes extraction/response generation, renderer
work and observer effects; it is not pure serializer or physical CPU time.

| Run / product | Pipeline p50 / p95 ms | Construction p50 / p95 ms | Remaining pipeline p50 / p95 ms | Selected HTTP-tail n |
| --- | --- | --- | --- | --- |
| 1 / reference | 0.430 / 0.561 | 0.112 / 0.151 | 0.318 / 0.409 | 17 |
| 1 / current | 0.408 / 0.662 | 0.146 / 0.238 | 0.262 / 0.427 | 20 |
| 2 / current | 0.402 / 0.653 | 0.149 / 0.237 | 0.253 / 0.408 | 18 |
| 2 / reference | 0.428 / 0.643 | 0.112 / 0.170 | 0.316 / 0.472 | 14 |

Sampled pipeline p95 changes are **+18.10% / +1.63%**, while its median is
lower in both current probes. Construction source is byte-identical, yet its
sampled median rises from about0.112 ms to0.146/0.149 ms, and construction p95
changes +57.41%/+38.91%. This is evidence that a fixed construction-code change
is not isolated here: cross-request allocation/cache state, compiler/caller
placement and OS/frequency/dispatch context can affect the observed boundary.
It does **not** exonerate the accepted product changes: changed allocation or
lifetime behavior could indirectly affect later requests. This probe does not
distinguish that interaction from host/compiler/observer effects.

Direct ID joins also show HTTP-tail requests do not all have identical server
behavior. The following uses selected requests that meet each complete HTTP
population's p95 threshold, not the p95 of the313-sample subset:

| Run / product | Selected tail n | Tail HTTP p50 ms | Tail waiting p50 ms | Tail pipeline p50 / p95 ms | Non-tail pipeline p50 / p95 ms |
| --- | --- | --- | --- | --- | --- |
| 1 / reference | 17 | 1.823 | 1.532 | 0.500 / 1.139 | 0.428 / 0.526 |
| 1 / current | 20 | 2.100 | 1.440 | 0.526 / 0.797 | 0.396 / 0.618 |
| 2 / current | 18 | 2.008 | 1.506 | 0.437 / 2.532 | 0.401 / 0.608 |
| 2 / reference | 14 | 2.113 | 1.942 | 0.678 / 1.351 | 0.428 / 0.543 |

Only14–20 sampled HTTP-tail requests exist per run/product; the modulo32 sample
is deterministic, not a random representative sample or a confidence interval.
The current run2 selected tail includes a2.006-ms construction outlier within a
2.532-ms pipeline, retained rather than excluded. Wall clocks include possible
preemption or allocator stalls; no instruction-level cause follows. The complete
current run2 HTTP maximum19.715 ms is retained even if not selected by the observer.
Do not subtract clocks across client/server processes to invent an exact transport /
queue budget, or subtract marginal HTTP and pipeline p95 values.

All per-request selected joint values, tail/non-tail component distributions,
ten new one-second bins, boundary populations and warmup records remain in
`diagnostic-component-summary.json`. All native raw values and complete HTTP /
integer-ms full-call populations remain in the raw files and reporting projection.
No row was filtered and no load point was tuned/replayed.

### Scope-supported conclusion and remaining gap

The original increase is localized at the client **waiting boundary**, with
secondary receiving/sending effects. New source-correlated pipeline clocks show
heterogeneous server-tail behavior and changes in an unchanged construction
boundary. These observations do not establish a fixed slower serializer algorithm
as the cause of the full HTTP p95 increase. They also do not establish that the
server or accepted changes are uninvolved. **Unique root cause remains unresolved**
among cross-request allocator/cache behavior, dispatch/scheduling/frequency,
compiler/caller placement, transport and observer/context interaction.

Retain current accepted products and the unfavorable HTTP result. There is no
measured safe optimization target selected in this item. If separately requested,
the next focused question is cross-request construction/allocation-state versus
runtime/OS scheduling context at this same medium workload. Preserve this probe's
limits and source/ID/time boundaries before choosing any profiler or optimization.
No allocator swap, compiler/host tuning, pipeline/API change, new item or further
load is activated here. Done may apply only to bounded attribution research,
not unique-cause resolution or a universal overhead/capacity/production claim.

### Verification, resource limits, failure history and cleanup

New focused verification: one rustfmt check on the changed Rust; one offline /
locked `cargo test --bin ssr --features ssr-stage-probe --no-run
--message-format=json --jobs 2`, then emitted `yt157_ --test-threads=1`:
**1 passed**,0 failed/ignored/filtered (old normal test cfg-excluded). Two new
offline/locked `cargo build --bin ssr --features ssr-stage-probe --release
--message-format=json --jobs 2` commands pass without warnings. Emitted
features are default/ssr-stage-probe; actual Core/HTML/adapter artifacts bind to
the accepted copies. No SQLx/provider/new dependency, old test/build/runtime
qualification, security/browser/platform or normal output proof was rerun.

Products, normal fixture/handler/oracle, dependencies/locks and old workload
scripts remain unchanged; cfg/module/startup selection is the only normal-path
source adjustment. Source review binds those facts, sampled observer overhead /
limits, and exact copies before builds. Exact retained k6 executable identity
is reused with no download/version execution. Resource properties confirm the
selected two-worker server and fixed affinities/quotas/limits. Original root
ancestor context and post-run actual shared-parent limits are separately labeled;
no final cgroup endpoint counters were fabricated after native unit teardown.

Measured client peak interval CPU is148.04% of one core (<480%), cumulative
memory peak114.94MiB (<819.2MiB), with zero client full-invocation throttling or
memory-limit events. Server inner counters likewise show zero throttling.
Inner resource windows span8.200–8.231s, max sample gap1.033s; sampling omits
boundary slivers/possible peaks. This is evidence of the declared sampled
headroom, not proof of absence of short scheduling/host interference. All
measured pairs meet the fixed criteria; all warmups are native-valid priming
operations, with no warmup comparability failure in this population.

New raw JSON totals **225,020,077 bytes**, max file **55,053,708 bytes**,
below512MiB /128MiB bounds. Runtime is56.558s; analysis/cleanup
finish inside its600s allowance and original unrenewed45-minute stage deadline.
Measured nominal40,000 versus actual40,003 is retained: three boundary starts,
nominal-minus-started-minus-dropped−3,39,996 valid window completions and seven
outside-window completions. Each phase's window/drain throughput stays separate;
roughly fixed throughput is not a measured maximum-capacity speedup.

One new arithmetic-artifact harness failure is retained. The first historical
analysis exited0, but the finite capture helper used the same name as its derived
JSON and replaced that summary with its native command receipt. A later display
read raised KeyError. Original raw inputs, exit/stdout/stderr and before-script
remain retained. One smallest output-filename correction regenerated only the
read-only statistics under a distinct receipt label; no test/build/load was
replayed and no clock/run was renewed. `analysis-artifact-failure.json` explains
the gap/correction, `historical-components.json` remains the original receipt,
and `historical-component-summary.json` is the reconstructed derived summary.
The initial contract P2 and old YT-156 observation failures remain separate history.
A report-preparation path variable was shadowed by a table loop and raised
AttributeError before any results write. One distinct-path-variable correction
completed the original report without source/data or measurement changes;
`report-preparation-failure.json` and the before-script retain this separate
preparation fault. There are no new failed test/build/load invocations or runtime errors.

Cleanup confirms **5 inactive owned units /13 absent owned PIDs**, absent original
owned cgroup paths and closed listener; four native stopped events and client
exit0 are retained. The cleared final ControlGroup is treated as unavailable,
not as a root cgroup to read. All inert sources/builds/tools/raw evidence remain
for review; no existing service/host policy or external environment changed.

Evidence under `target/yt157-medium-http-tail-iw9_x70a/evidence/` retains authority,
baseline/history/review/bounds/source/copy/test/library/binary/tool identities,
immutable preexecution, native raw/summary/exit/resource/stage records, both new
component summaries, the arithmetic failure and cleanup. Final independent Reviewer
`/root/yt157_medium_tail_final_review_20261024` returned PASS, zero issues/required
followups, after reconciling all raw components/bins/tail/ID statistics, reviewed
nine hashes, data/history/source/executable bindings and cleanup. Acceptance is
retained in `final-review-result.json`; `closeout.json` records the permitted
five-document status/reviewer-reference transition. All runtime code, binaries
and observations stay frozen. Done is bounded research, with unique cause still
unresolved; no YT-157 commit/push. YT-096 decisions remain Pending,
YT-020 Blocked and the immutable release-candidate scope unchanged.
