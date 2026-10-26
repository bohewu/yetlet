# YT-156 — Full HTTP SSR before/current comparison

Date: 2026-10-24 (Asia/Taipei). Status: **Done at bounded comparison scope; independent final Reviewer PASS.**

## Authority and question

The user selects a contract-first minimum full HTTP SSR comparison, small /
medium / large pages at two fixed load points, before/current product sources,
reused evidence, no accepted-proof replay, retained failures and independent
Reviewer PASS before Done. New changes must not be committed or pushed. Evidence is under
`target/yt156-http-ssr-15_kpxn8/evidence`. YT-155
was committed separately with strict/builtin Guard at `0580a0a`; that clean HEAD
is this task's baseline. This request does not authorize an optimization,
capacity sweep, provider/database benchmark or public release.

Preserve [ADR 0001](adr/0001-separation-of-concerns.md),
[ADR 0003](adr/0003-renderers-and-loading.md) and
[ADR 0006](adr/0006-handler-first-mvc-optional.md). The question is whether the
accepted serializer/document changes have observable effects through fresh View
construction, Page response generation, Axum/Hyper and HTTP transfer under these
fixed conditions. No speedup target or requirement to outperform the reference
exists. Shared-host short paired observations do not prove universal performance,
maximum capacity, an SLO or unique instruction-level causality.

## Reused evidence and actual products

Reuse [YT-143](YT-143-AXUM-WORKLOAD-COMPARISON.md)'s pinned k6, native arrival
executor, isolation, corrected ext4 placement and resource/reporting seams.
Its 428-byte SSR, plaintext/JSON and [YT-144](YT-144-POSTGRES-WORKLOAD-COMPARISON.md)
results remain historical, not denominators for this new experiment. Reuse
[YT-145](YT-145-SSR-PERFORMANCE-BASELINES.md)'s exact 8/128/1,024-row fixture and
independent literal oracle, [YT-147](YT-147-PRIVATE-ESCAPE-INTO.md) /
[YT-149](YT-149-PRIVATE-URL-PREFIX.md) /
[YT-154](YT-154-DOCUMENT-CAPACITY-COMPARISON.md)'s accepted output/security evidence
and [YT-155](YT-155-SMALL-MEDIUM-LATENCY-STABILITY.md)'s timing limitations.
Retain old negative points, tmpfs/observation/setup failures and review history.
Do not run old suites, microbench/counts, plaintext/JSON/PostgreSQL, saturation,
browser/CSP enforcement, provider/platform or release qualification again.

Reference is immutable `90f83867c77307c9aab1b4ca72afe7f20afb4dc1`, before private
escape-into, URL streaming and document capacity changes. Current product source
is `0580a0ac86eab0dcc83b4b24ca330da595fc81fc`. Use actual separately compiled
libraries, not shadow renderers. Read-only Git archives into task-owned source
directories are allowed; do not checkout/reset or create a worktree. Overlay the
same final standalone HTTP sample bytes on both archives. Freeze hashes of all
relevant product/manifests/locks and emitted library artifacts/executables.
Inspected product differences are HTML serializer and adapter document functions;
Core's only intervening source difference is test-only code. Its rejected
first-child capacity policy is absent. Root dependencies/locks/default graphs,
public APIs and actual Core runtime are unchanged. Verify these facts before
interpreting the pair. Both variants must use one identical dependency lock,
compiler/profile, fixture, handler and runtime; only accepted product versions differ.

## Minimum consumer and exact response

Extend only standalone `examples/axum-workload-comparison`: add `ssr` binary
`src/bin/ssr.rs`, explicit binary declaration and `ssr-workload.js`, plus README.
Use existing Axum/Tokio/Core/adapter dependencies and lock without adding packages,
features or root membership. PostgreSQL remains unselected. No product/Core,
renderer, middleware, transport, dependency, CI or reusable runner changes.

`App::new().get("/ssr/{size}", handler).into_router()` runs the same handler in
both binaries. The only sizes are small=8, medium=128, large=1,024. Every valid
GET constructs a fresh metadata/View using byte-identical YT-145 fixture constants
and builder call order, then returns actual `Page::with_metadata`. No retained
View, HTML/response/escaped-string cache, body fusion, custom serializer or
per-request log. All routing/extraction/SSR/response and network work is inside
the HTTP workload. Startup variant label affects logs only. Bind disposable
loopback/high port, two Tokio workers, native graceful SIGTERM/Ctrl-C stop.

Expected complete document UTF-8 lengths are **2,383 / 31,063 / 245,207 bytes**.
The fixed independent already-escaped literal oracle is copied unchanged from
YT-145, bound to retained YT-154 current document files, not computed by either
renderer. Title/text/attributes include all five escape characters and café;
relative URL attributes include an ampersand. The body is constant for a size
but genuinely rebuilt per request; this synthetic workload makes no user-data
or arbitrary-template claim. Client expected Strings are prepared once outside
iterations and are not server caches.

Every load response must be HTTP 200, HTTP/1.1, exact literal body, matching
Content-Length, `text/html; charset=utf-8` and the existing strict CSP verbatim:

```text
default-src 'none'; script-src 'self'; style-src 'self'; connect-src 'self'; img-src 'self'; base-uri 'none'; object-src 'none'; form-action 'self'; frame-ancestors 'none'
```

No Content-Encoding, Set-Cookie or Server header. Compare all non-Date headers
across versions/phases, retaining actual headers and body SHA/length in preload
probes. The default Date value is observed but time-dependent. Requests use GET,
Accept text/html, Accept-Encoding identity, no body, cookies, auth, redirects,
retries, sleeps or compression. Plain loopback HTTP/1.1 keep-alive only; no TLS,
HTTP/2, pipelining, proxy, browser runtime or remote network. Exact HTTP/CSP-header
checks do not requalify browser CSP enforcement/security policy.

## Fixed sequence, new measurements and limits

Use the accepted official k6 **2.3.0**, archive SHA-256
`39c3117b6af817592dcd0ce4242105c0a7af10948c2a425306f0be8f7a8a8ab1`,
executable SHA-256
`e001bbf39ba67424a85c9c8516440ba5c73ecf7a6021be96c76194380c15f260`.
Reuse official acquisition/version/checksum receipts; if the old binary is gone,
restore only that exact task-local artifact with bounded curl/tar and hash checks.
No installation, new tool comparison, cloud usage or telemetry. Use the existing
k6 native constant-arrival-rate executor and response timings, not a new scheduler.

Two repetitions: repetition 1 sizes small/medium/large, repetition 2 reverses
large/medium/small. Each size visits **250/s then 1,000/s**, ten scheduled seconds
per point. Each size/rate pair is adjacent reference/current in repetition 1,
current/reference in repetition 2. Thus 3 sizes × 2 rates × 2 versions × 2
repetitions = **24 measured phases / 150,000 nominal measured arrivals**.
Each phase starts a fresh matching server on the same owned high port, probes
all three exact response sizes, runs a separate **2-second 125/s warmup** of its
selected size, waits for completion/drain and idles one second, then measures.
That is 6,000 nominal warmup arrivals, excluded from measured counts.
64 preallocated=max VUs, request timeout/gracefulStop eight seconds; no changes
based on results. Warmup primes server work; measured invocation connections
start fresh. Stop/reap server and confirm closed listener before the next variant.
No failed point replay or rate/duration/resource tuning. Ordinary performance /
comparability failures remain in the fixed sequence; correctness/resource/source
or missing-evidence failures stop further phases and preserve partial evidence.

| Resource | Fixed bound |
| --- | --- |
| Server | CPUs 12–13, two workers, CPUQuota 200%, MemoryMax 256 MiB, TasksMax 128, LimitNOFILE 512, Nice 10, no restart; RuntimeMax 45 s, native stop ≤5 s |
| Client plus monitor | CPUs 14–19, CPUQuota 600%, MemoryMax 1 GiB, TasksMax 128, LimitNOFILE 512, Nice 10, no restart; outer RuntimeMax 1,200 s |
| Raw/evidence | Task-owned ext4, per-file ≤512 MiB, total evidence ≤2 GiB, ≥8 GiB free initially; no response bodies in raw per-request metrics |
| Time | Fixed unrenewed stage-2 45-minute UTC/monotonic deadline; new builds/tests ≤600 s each, tool restore ≤120 s, runtime including analysis/stops/cleanup ≤1,200 s, all inside stage deadline |

Record current topology/affinity, mount/space, cgroup ancestors, limits, compiler /
Cargo 1.99.0, nonsecret flags, load/kernel/RAM/governor and tools before assertions.
The inspected CPUs 12–19 are distinct physical cores; confirm exact placement
still available or stop, rather than substitute after results. No host/quota /
governor/compiler/allocator adjustment, service interruption or external resource.
Use default release/System allocator, two build jobs, offline locked builds,
no RUSTFLAGS/LTO/PGO/profile override. Retain sources/builds/raw files inert.

At 8 KiB/attempt allowance, 156,000 nominal attempts produce about 1.19 GiB of
raw points, below 2 GiB; actual bytes remain enforced. Each completed owned raw
file is streamed, fsynced, then advised POSIX_FADV_DONTNEED to avoid prior output
cache accumulating in the client cgroup. This is not global cache dropping or
server-input flushing. Adapt only existing one-off YT-143 receipt/monitor/native
unit lifecycle/arithmetic seams; keep them task-local. Do not introduce a runner,
validator framework, certification matrix or harness self-tests.

## Reporting and comparable pairs

Persist exact argv/env, UTC/monotonic boundaries, native exits/timeouts, immutable
source/lock/script/tool/executable identities and all k6 raw JSON/summaries/stdout /
stderr immediately. Keep nominal, started, completed, valid 200, drops,
interruptions, transport/timeouts, all statuses including 429/503/504 and invalid
200 separately. Record scheduled-window completion rate and through-final-drain
rate, metadata anchor/start/completion timestamps, boundary extras and residuals.
Configured arrival rate is not achieved throughput.

Primary per-response metric is native k6 HTTP sending+waiting+receiving duration;
report nearest-rank p50/p95/p99 for all completions, HTTP responses and valid 200
separately. Retain native/custom timing equality and the distinct integer-ms
full-call wall-clock population (Date.now; limited resolution/clock adjustment).
Drops/interrupted attempts have no completed time. Never infer submillisecond
full-call overhead from integer values or exclude unfavorable rows.

Retain roughly 1-second actual post-exec app/k6/monitor PID CPU/RSS/thread/FD
samples, inner scheduled-window sample bounds/gaps, and cgroup CPU/throttling,
memory/events/pids/output counts. One core=100%; process RSS is neither logical
allocation counts nor cgroup memory (which includes file cache). No allocation
instrumentation or telemetry endpoint. Cleanup records native app stop, k6 wait,
unit inactive/MainPID 0/cgroup absence, owned PIDs absent and listener closed.

A comparable phase requires exact output, native exit0, no errors/drops /
interruptions, ≥95% of nominal valid completions inside its window, consistent
counts/timings, ≥2 inner resource samples with gaps ≤2 s and no limit/evidence
failure. Client headroom requires peak interval k6 CPU <480% (80% of allocation),
client cgroup cumulative peak <819.2 MiB, zero client throttling/OOM/memory-limit
or task/FD-limit events. Report server CPU/throttling distinctly; failed high
points do not automatically establish server capacity. Missing evidence is
unverified, not zero. Do not stop merely for a slow but otherwise safe point.

Calculate current/reference ratios only when both same-size/rate/repetition
phases are comparable with identical output/build/runtime/limits. Latency >1
means slower; achieved-throughput >1 means more completed work. Fixed offered
rates tend to fix throughput, so no capacity-speedup claim. Show every point,
both orders, CPU/RSS and errors; no pooled-only result, significance claim,
unique micro-optimization attribution, universal winner or production claim.
If neither/both comparisons are conclusive, report that honestly; retain failed
points and do not tune for a favorable result. Existing security semantics and
release gates are unchanged.

## Minimum verification and acceptance

After independent contract PASS, freeze new sample Rust/Cargo/JS bytes. One new
sample-local `yt156_` test exercises all three fresh HTTP routes in process,
checking the independent literal body, byte lengths, content/CSP headers and
absent forbidden headers. Compile once offline locked `cargo test --bin ssr
--no-run --message-format=json --jobs 2` against the current copy and run only
its emitted `yt156_ --test-threads=1` filter. Do not run old sample tests or
root/product suites. One new-file rustfmt check, two ordinary new `--bin ssr`
offline locked release builds and the fixed live probes/populations suffice.
Reuse accepted invalid-tree/escaping/URL/body-lifetime proofs unchanged.

Exactly eight repository paths may change: four documents (this report,
BACKLOG.md, ROADMAP.md, docs/README.md) and four existing HTTP sample paths
(Cargo.toml, README.md, new src/bin/ssr.rs, new ssr-workload.js). Lock/root/sample
old binaries/scripts/fixtures and all product sources remain unchanged. Freeze
all eight hashes, data/source/history/executable bindings for independent final
review. Only after final PASS0 mark Done with status/reviewer-reference edits in
the five documents, then docs/whitespace checks; Rust/Cargo/JS, binaries and
observations stay frozen. No optimization, additional item, new commit/push or
YT-096/YT-020 release decision follows.

## Results and scope of acceptance

Independent contract Reviewer `/root/yt156_http_ssr_contract_review_20261024`
returned PASS, zero issues/required followups, before implementation. Exact four
reviewed hashes, native role/model/effort trace and enforced resolver decision are
retained. All 24 new fixed phases ran once, in the declared order, with 48 native
k6 exits 0 and 24 successful native app stops. No point was replayed or removed.

Measured nominal arrivals were **150,000**; actual started/completed/exact valid
200 responses were **150,019**. Warmup nominal was 6,000; actual started/completed /
valid 200 were **6,014**. Both populations have zero drops, interruptions, transport
errors, unexpected statuses or invalid 200s. All native/custom HTTP distributions
match. There are 72 exact live preload probes (all three sizes per fresh server),
with equal non-Date headers and accepted literal body lengths/hashes.

Actual starts are not forcibly rounded to nominal: measured starts include 24
outside the half-open scheduled windows, nominal-minus-started-minus-dropped is
−19, and 149,984 valid completions fall inside those windows; 35 finish outside.
Each point's anchors/start/completion boundaries and residuals remain in raw data.
Scheduled valid rates are 250.0/s at the low points and 999.7–1,000.0/s at the
high points. Drain-based denominators are separate. Neither the extra boundary
arrivals nor roughly equal throughput demonstrate capacity improvement.

### All measured HTTP points

Nearest-rank HTTP p50/p95/p99 are milliseconds, all exact valid responses (also
all completions here). CPU is app mean percent of one core over the recorded
inner interval; RSS is sampled app peak MiB, not logical allocations or an exact
peak. Every row is comparable under this contract; no failed point is omitted.

| Run | Size | Offered /s | Product | Valid 200 | Window valid /s | p50 ms | p95 ms | p99 ms | App CPU % | App RSS MiB |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | small | 250 | reference | 2,500 | 250.0 | 0.494 | 0.622 | 0.771 | 7.14 | 6.95 |
| 1 | small | 250 | current | 2,501 | 250.0 | 0.464 | 0.597 | 0.767 | 6.42 | 6.73 |
| 1 | small | 1,000 | reference | 10,001 | 999.9 | 0.350 | 0.542 | 2.167 | 13.13 | 7.06 |
| 1 | small | 1,000 | current | 10,001 | 999.9 | 0.332 | 0.516 | 1.902 | 12.24 | 7.36 |
| 1 | medium | 250 | reference | 2,500 | 250.0 | 1.553 | 1.683 | 2.181 | 31.41 | 7.32 |
| 1 | medium | 250 | current | 2,501 | 250.0 | 1.252 | 1.394 | 1.726 | 24.24 | 7.19 |
| 1 | medium | 1,000 | reference | 10,001 | 999.7 | 0.582 | 1.209 | 2.606 | 48.07 | 7.43 |
| 1 | medium | 1,000 | current | 10,001 | 999.8 | 0.609 | 1.445 | 2.878 | 47.04 | 7.35 |
| 1 | large | 250 | reference | 2,501 | 250.0 | 2.033 | 2.210 | 3.220 | 48.27 | 10.18 |
| 1 | large | 250 | current | 2,500 | 250.0 | 2.057 | 2.241 | 3.548 | 48.24 | 10.32 |
| 1 | large | 1,000 | reference | 10,001 | 999.8 | 1.909 | 2.449 | 3.722 | 180.71 | 10.55 |
| 1 | large | 1,000 | current | 10,001 | 999.9 | 1.441 | 2.607 | 2.899 | 133.88 | 10.61 |
| 2 | large | 250 | current | 2,500 | 250.0 | 2.049 | 2.251 | 3.570 | 47.79 | 10.36 |
| 2 | large | 250 | reference | 2,501 | 250.0 | 2.015 | 2.157 | 2.492 | 48.46 | 9.93 |
| 2 | large | 1,000 | current | 10,001 | 999.8 | 1.438 | 2.588 | 2.810 | 133.52 | 10.62 |
| 2 | large | 1,000 | reference | 10,000 | 999.8 | 1.877 | 2.466 | 10.642 | 178.84 | 10.95 |
| 2 | medium | 250 | current | 2,501 | 250.0 | 1.256 | 1.424 | 1.720 | 24.47 | 7.45 |
| 2 | medium | 250 | reference | 2,501 | 250.0 | 1.536 | 1.665 | 2.115 | 30.52 | 6.69 |
| 2 | medium | 1,000 | current | 10,002 | 999.8 | 0.602 | 1.391 | 3.056 | 45.80 | 7.17 |
| 2 | medium | 1,000 | reference | 10,001 | 1,000.0 | 0.575 | 1.164 | 2.606 | 46.85 | 7.11 |
| 2 | small | 250 | current | 2,501 | 250.0 | 0.466 | 0.620 | 0.973 | 6.54 | 7.11 |
| 2 | small | 250 | reference | 2,501 | 250.0 | 0.492 | 0.634 | 0.827 | 7.52 | 6.84 |
| 2 | small | 1,000 | current | 10,001 | 1,000.0 | 0.338 | 0.515 | 1.973 | 12.49 | 6.93 |
| 2 | small | 1,000 | reference | 10,000 | 1,000.0 | 0.354 | 0.539 | 1.791 | 13.81 | 7.00 |

### Paired changes, including unfavorable tails

Changes below are current/reference minus one, shown descriptively at each
size/rate/run. The two orders are not independent trials or confidence intervals.
CPU compares the recorded inner sample intervals, which differ slightly between
processes; approximately one-second sampling and CPU ticks limit precision.

| Size | Offered /s | Run | Order | p50 change | p95 change | p99 change | App mean CPU change |
| --- | --- | --- | --- | --- | --- | --- | --- |
| small | 250 | 1 | reference → current | -6.2% | -4.0% | -0.5% | -10.1% |
| small | 1,000 | 1 | reference → current | -5.2% | -4.7% | -12.2% | -6.8% |
| medium | 250 | 1 | reference → current | -19.3% | -17.2% | -20.9% | -22.8% |
| medium | 1,000 | 1 | reference → current | +4.8% | +19.6% | +10.4% | -2.1% |
| large | 250 | 1 | reference → current | +1.2% | +1.4% | +10.2% | -0.1% |
| large | 1,000 | 1 | reference → current | -24.5% | +6.5% | -22.1% | -25.9% |
| large | 250 | 2 | current → reference | +1.7% | +4.3% | +43.3% | -1.4% |
| large | 1,000 | 2 | current → reference | -23.4% | +5.0% | -73.6% | -25.3% |
| medium | 250 | 2 | current → reference | -18.2% | -14.5% | -18.7% | -19.8% |
| medium | 1,000 | 2 | current → reference | +4.6% | +19.4% | +17.3% | -2.2% |
| small | 250 | 2 | current → reference | -5.4% | -2.3% | +17.7% | -13.0% |
| small | 1,000 | 2 | current → reference | -4.5% | -4.4% | +10.2% | -9.6% |

The accepted optimization bundle has observable but mixed full-HTTP effects:

- Small pages have lower p50/p95 in both orders at both points; p99 changes
  direction between repetitions. These submillisecond differences in a short
  shared-host loopback test are not a general tiny-page speed guarantee.
- Medium at 250/s has about 18–19% lower p50, 14–17% lower p95 and lower app CPU.
  At 1,000/s, p50 is about 5% higher and p95 about **19% higher in both orders**,
  while app CPU is slightly lower. This unfavorable result remains explicit.
- Large at 250/s has roughly unchanged CPU and slightly higher p50/p95; current
  p99 is higher in both runs. At 1,000/s, current p50 is about **23–25% lower**
  and app CPU about **25% lower**, but p95 is **5–6% higher**. The reference's
  second-run p99 10.642 ms is retained; its large reduction in that one pair
  is not promoted into a stable tail-speedup claim.
- RSS does not show a consistent reduction across these points. Retained logical
  String-capacity improvements from YT-154 do not imply reduced process RSS.

Differences between 250/s and 1,000/s include lower medians at the higher offered
rate in several groups. The host's recorded powersave governor, load/runtime /
connection/allocator state and scheduling are contextual possibilities, not
measured causes. No per-request frequency/cycle/queue attribution exists here.
Do not subtract YT-145/150/155 microbench times from HTTP latency, sum component
medians or assign these bundle effects to one accepted optimization. This is
before/current Yetlet, not native-Axum/template-engine ranking or an official
TechEmpower result. Current/reference achieved-throughput ratios remain near
1 because arrivals are fixed; no larger maximum req/s follows.

### Resources, bounds and raw populations

All 24 **measured** phases meet the declared comparability rules and all twelve
pairs are comparable. Peak measured client interval CPU is 147.87% of one core,
below 480%; client cgroup cumulative memory peak is 178.67 MiB, below 819.2 MiB.
Client full-invocation cgroup CPU/memory-event deltas are retained in addition to
inner-window samples. No sampled/client invocation throttling, OOM, memory/task /
FD-limit event is present. Server inner intervals likewise show no throttling.
Server mean CPU spans 6.42–180.71%, sampled RSS 6.69–10.95 MiB. These are under
the fixed two-worker/200% server envelope, not maximum capacity measurements.

Inner measured resource intervals span 8.218–9.230 s, maximum sample gap 1.037 s;
sampling misses window-boundary slivers and possible peaks. Actual interval bounds,
process/cgroup populations, CPU counters, RSS/threads/FDs/memory and start/stop
properties remain in `samples.jsonl`, runtime and reporting projection. Native
unit properties confirm the selected affinity, quotas, memory/FD/task/file bounds,
Nice, restart/stop/runtime limits; common ancestors and eight distinct physical
cores are recorded. No host frequency/governor/cache policy was changed.

Warmups are native-successful priming operations, not timing-comparison points.
The projection conservatively flags insufficient inner resource samples for the
eight two-second small-page warmups, with dependent headroom fields unverified;
these flags are not observed CPU/memory/task/FD cap breaches. They are retained
without replay or longer warmup. All measured phases have sufficient samples;
no warmup resource-comparability claim is made. Initial connection/VM preparation
is outside the scheduled measured-window latency populations.

Raw k6 files total **816,917,315 bytes**, largest **52,666,658 bytes**,
below 2 GiB total / 512 MiB per file. The 24-phase runtime was
355.951 s; analysis and owned cleanup also finish inside
the original 1,200-second runtime allowance and unrenewed 45-minute stage clock.
All per-invocation JSON/summaries/stdout/stderr/exits and full-call integer-ms
populations remain; no filtering, pooling-only result or repeated load occurred.
The corrected ext4 placement and completed-owned-file cache advice were selected
before startup; the earlier YT-143 tmpfs failure was not recreated.

### Verification, failures and cleanup

New verification only:

- One focused `rustfmt --edition 2024 --check` on `src/bin/ssr.rs`: exit 0.
- One offline/locked `cargo test --bin ssr --no-run --message-format=json --jobs 2`
  compile against the current source copy, then emitted `yt156_ --test-threads=1`:
  **1 passed**, 0 failed/ignored/filtered. The single test exercises all three
  new routes, exact independent literal bodies and content/security headers.
- Two offline/locked default-release `cargo build --bin ssr --release
  --message-format=json --jobs 2` commands with separate source/target paths:
  exits 0, no compiler warnings. Actual library/executable artifacts select
  default dependencies only; PostgreSQL/SQLx was not compiled. Root/sample locks
  and earlier HTTP/SSR consumers are unchanged.
- Exact accepted k6 archive/executable identities restored task-locally; previous
  official provenance/version evidence reused. No tool version/workload proof
  requalification or old accepted suite was run.
- All 48 new k6 native exits, 24 native app stops and 72 live HTTP probes pass;
  browser/CSP enforcement, providers/platforms, databases, sustained capacity,
  remote/TLS/HTTP2 and production qualification remain unrun at this scope.

Two task-local observation/preparation failures are preserved:

1. Before compilation, source binding compared resolved LICENSE contents with
   Git symlink blobs and failed one assertion. One smallest binding correction
   separately checks link text and resolved content SHA; ordinary Git-blob
   checks stay unchanged. No product/source/archive change or test/load replay.
2. After all 24 phases and native client exit0, an optional final cgroup read
   used the exited unit's cleared ControlGroup and hit FileNotFoundError at the
   cgroup root. Required live resource records and exits were already saved.
   One smallest cleanup observation correction omitted the unavailable endpoint
   read and checked absence using the originally captured owned cgroup path.
   It only finished native stop/absence checks, with no runtime/source change
   or measurement replay. Missing final endpoint counters are not invented.

`source-binding-failure.json` and `cleanup-observation-failure.json` retain the
classifications, observed exit/error, unmet condition and bounded corrections.
Neither is a product failure, candidate restart or renewed budget. Historical
negative observations/failures remain intact. The frozen runtime reports no
errors and matching source hashes at native completion. Subsequent README/report
updates only document results; Rust/Cargo/JS/products/copies/binaries stay frozen.

Cleanup confirms **25 inactive owned units, 73 absent owned PIDs**, absent owned
cgroup paths and a closed loopback listener. All 24 apps report the native
`stopped` event and zero exit status; the client/monitor native exit is 0. Inert
sources/builds/tools/evidence remain task-local. No existing service/data, host
policy, external resource, commit or push was changed by YT-156.

### Decision and remaining diagnostic question

Retain the already accepted product implementations and report the workload /
percentile tradeoffs. This comparison does not justify another optimization or
claim universal page acceleration. A useful next separately selected question
is **medium-page HTTP p95 at 1,000/s**, followed by the large-page p95 tradeoff:
distinguish request scheduling/response transfer and full pipeline behavior
before proposing a product change. These repeated unfavorable tails are recorded
as an open diagnostic gap, not a diagnosed renderer defect, new backlog item or
activated implementation. No extra load/tool/profiling scope is started.

Artifacts under `target/yt156-http-ssr-15_kpxn8/evidence/` include authority,
baseline/history bindings, contract review, stage clock, source snapshots/review,
measurement source hashes, emitted test/library/binary artifacts, tool receipts,
immutable `preexecution.json`, all raw/native/property/sample data, runtime and
`reporting-projection.json`, both failures and `cleanup.json`. Final independent Reviewer
`/root/yt156_http_ssr_final_review_20261024` returned PASS, zero issues/required
followups, after independently reconciling all raw populations, paired ratios,
source/data/history/executable bindings and cleanup. `final-review-result.json`
retains acceptance; `closeout.json` records the permitted five-document
status/reviewer-reference transition. Runtime code, binaries and observations
remain frozen. Done applies only to the bounded comparison with its mixed results. YT-096 D1–D6 remain Pending and YT-020 Blocked;
original immutable release-candidate scope is unchanged. No new commit or push.
