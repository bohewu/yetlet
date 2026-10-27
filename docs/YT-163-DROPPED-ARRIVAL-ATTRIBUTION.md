# YT-163 — Dropped-arrival attribution from retained platform evidence

Date: 2026-10-24 (Asia/Taipei). Status: **Done at bounded retained-data attribution scope after Independent final Reviewer /root/yt163_drop_results_final_review_20261024 PASS, zero issues/required followups. Unique underlying cause unresolved; failed second pair preserved.**

## Purpose and authority

The user activates YT-163 after committing the sixteen accepted YT-157–162
paths. Investigate the twenty-seven unstarted arrivals in YT-162 reference2
using retained evidence; separate load-generator scheduling, VM/SSH transport
and server waiting before suggesting a change. This is optional benchmark
research, not a consumer hardware/SLO or release gate. New work must not be
committed or pushed.

Preserve [YT-162](YT-162-ISOLATED-PLATFORM-COMPARISON.md), including its failed
second pair, native controller exit1, cleanup-only final app stop, earlier
provisioning failures and unresolved historical Blob404. Reuse all accepted
qualifications and exact binaries/caller without executing them. Preserve
[ADR 0001](adr/0001-separation-of-concerns.md),
[ADR 0003](adr/0003-renderers-and-loading.md) and
[ADR 0006](adr/0006-handler-first-mvc-optional.md): no Core, API, output,
security, runtime, dependency, feature-default or application behavior changes.

## Retained inputs and semantic boundaries

The accepted evidence root is
`target/yt162-target-selection-3l3dnyvk/evidence/azure-ready`.
Use its canonical returned ZIP, measured JSONL, native invocation snapshots,
logs, existing component analysis and sampled server records. New private
analysis/output belongs in `target/yt163-drop-attribution-20261024`;
the YT-162 inputs remain byte-for-byte intact.

The unchanged caller uses medium128-row SSR,1,000 arrivals/s for10s and64 fixed
VUs. All39977 started measured requests completed with valid200, with27 drops
only in reference2. The four nominal end-boundary starts remain included;
unstarted arrivals have no response latency. The second pair remains unqualified
even if this study explains its scheduling failure.

Grafana's [dropped-iteration definition](https://grafana.com/docs/k6/latest/using-k6/scenarios/concepts/dropped-iterations/)
states that arrival-rate iterations can drop when no VU is available. That
identifies the executor's immediate condition, not which downstream or local
component occupied it. [Built-in metric definitions](https://grafana.com/docs/k6/latest/using-k6/metrics/reference/)
distinguish iteration duration, sampled active VUs and HTTP components.
HTTP waiting includes the transport path and cannot be equated with server CPU.

## Bounded offline contract

1. Read the four retained measured populations once for this new question.
   Reconcile twenty-seven native dropped-iteration points with the preserved
   warning, start/completion timeline and any burst or recovery. Keep zero-drop
   populations as existing-data context; do not rerun their load or qualifications.
2. Work on the client clock. Preserve native timestamp precision and the
   caller's integer-millisecond start/completion resolution separately. Use
   request IDs to join sampled server records; do not subtract unrelated VM
   wall clocks or pair untagged iteration metrics with invented request IDs.
3. Examine native HTTP sending/waiting/receiving, blocked/connecting and full-call
   intervals around the drops. Reconstructed iteration intervals may describe
   measured occupancy, but are not a continuous executor/VU-state trace. Account
   for JS validation, bookkeeping and scheduling outside the HTTP bracket.
4. Compare the existing boundary cgroup/proc snapshots with their coverage:
   invocation means/events cannot exclude a short CPU, memory, guest scheduling
   or tunnel stall. Sparse modulo32 server probes cannot describe every request;
   synchronous construction CPU must not be taken across await or interpreted
   as complete server/transport service time. The final app's missing normal
   stop snapshot stays missing.
5. State each supported observation, inference and unresolved alternative.
   Stop at the strongest attribution the retained data supports. If unique
   causality is unidentifiable, report that limit and propose the smallest
   future measurement needed; do not silently increase VUs, memory or hardware,
   tune the server, change the caller, or manufacture a passing point.

This contract authorizes local read-only analysis and four documentation paths
only: this report, BACKLOG.md, ROADMAP.md and README.md under docs. It does not
authorize new live requests, builds/tests, benchmarks, VM/cloud resources, SSH
sessions, host configuration changes, a benchmark framework or a validator.
Stream the existing ZIP without retaining a second raw copy; retain every
failure and keep new local analysis/output below32MiB. No local server or
background load process is started, so no new external cleanup is necessary.

## Acceptance

Independent contract review precedes the dependent offline analysis. Run
`cargo xtask check docs` and `git diff --check` for the changed documentation;
these are checks of new work, not replays of accepted runtime qualifications.
Keep the analysis command/native exit, input/output hashes, all drop points
and the attribution limits available for independent final review.

YT-163 remains In progress until independent final Reviewer PASS. Done means
the bounded retained-data investigation is complete, with an honest unresolved
cause if the evidence cannot separate alternatives. It does not mean the failed
point passed, an optimization was implemented, or future live work was approved.

## Actual offline investigation

Independent contract Reviewer /root/yt163_drop_contract_review_20261024 returned
**PASS, zero issues/required followups**, before analysis. One local streaming
pass read the four measured JSONL members of the accepted ZIP; no raw copy was
extracted and no executable/caller, workload or cloud resource was started.
The native analysis exits0. Its script, output and hashes are retained under the
declared YT-163 directory. A small derivative calculation reads only that output.
All57 retained input bindings remain unchanged; earlier failures and the failed
pair's disposition remain intact.

### Timeline and immediate scheduling mechanism

The reference2 client scenario anchor is08:44:28.700UTC on2026-10-24. Native
drop points are emitted at08:44:31.945899214–08:44:31.973651361UTC: all27 occur
within27.752147ms, at relative3245.899214–3273.651361ms. These are actual
processing/emission times, not exact nominal arrival-slot timestamps. The
[pinned k6 v2.3.0 executor](https://github.com/grafana/k6/blob/v2.3.0/lib/executor/constant_arrival_rate.go)
records a drop when its pool cannot start the iteration and emits the64-VU
warning once. Dropped attempts have no probe ID; do not assign their slots to
gaps in the sequence of successfully started request IDs.

Native HTTP points mark response completion. Reconstruct their elapsed brackets
as [completion minus duration, completion), excluding blocked/connecting.
For iteration duration, the
[pinned runner](https://github.com/grafana/k6/blob/v2.3.0/internal/js/runner.go)
emits its runFn elapsed time at its end; reconstruct that bracket separately.
This includes the JS function/event loop but excludes some executor and metric
delivery work. Untagged iteration brackets are never joined to individual
request IDs. Native timestamps retain nanoseconds; elapsed metrics are converted
to integer nanoseconds with rounding, not a claim of nanosecond clock accuracy.
The caller's Date.now bracket stays a separate1ms-resolution observation.

| Retained measured population | Completed valid200 | Drops | Peak reconstructed native HTTP overlap | Peak reconstructed runFn overlap |
| --- | --- | --- | --- | --- |
| reference1 | 10001 | 0 | 18 | 19 |
| current1 | 10001 | 0 | 18 | 19 |
| current2 | 10001 | 0 | 15 | 17 |
| reference2 | 9974 | 27 | 64 | 64 |

At **every one of the27 drop timestamps**, both reconstructed native HTTP and
runFn overlap are64. At the first drop all64 HTTP brackets are in waiting;
at the last,63 are waiting and one receiving. This supports a transient
occupied-VU mechanism involving outstanding HTTP, rather than an explanation
solely in post-HTTP body validation/bookkeeping. It does not establish which
machine or transport component delayed those HTTP completions.

The following bins are client-clock half-open intervals. Open HTTP is measured
at the bin end and includes requests started before that bin.

| Relative interval ms | Starts | HTTP completions | Drops | Open native HTTP at end |
| --- | --- | --- | --- | --- |
| 3180–3190 | 9 | 2 | 0 | 8 |
| 3190–3240 | 50 | 0 | 0 | 58 |
| 3240–3250 | 6 | 0 | 4 | 64 |
| 3250–3260 | 0 | 0 | 11 | 64 |
| 3260–3270 | 0 | 0 | 9 | 64 |
| 3270–3280 | 8 | 14 | 3 | 58 |
| 3280–3300 | 20 | 43 | 0 | 34 |
| 3300–3310 | 10 | 44 | 0 | 1 |

The native HTTP completion gap is89.036876ms, from relative3181.538225 to
3270.575101ms. The largest caller-start gap is26.171678ms during the full-VU
period; it must not be relabeled as a26ms total VM freeze because the executor
continues emitting drops during it. This one burst accounts for all27 drops;
later responses drain without another drop. The sampled vus gauge reads1 on
either side of the burst, roughly1s apart, and misses it; its maximum2 over the
whole invocation is not a continuous peak or evidence contradicting the burst.

### Waiting path versus instrumented server work

The union of native HTTP brackets covering at least one drop contains66
completed requests. Its HTTP median is77.006964ms and maximum121.683587ms;
waiting accounts for98.869% of summed HTTP durations. Sending/receiving means
are0.008077/0.849859ms. Connecting is zero throughout this cohort and blocked
is at most0.017173ms: the measured local connection-setup bracket does not
explain these long durations. HTTP waiting still covers the existing
client/SSH/network/server path, including time before the sampled middleware.

Only two of those66 requests have modulo32 server probes:

| Request ID | HTTP ms | HTTP waiting ms | Server pipeline ms | Synchronous construction CPU microseconds |
| --- | --- | --- | --- | --- |
| m:3200 | 97.978046 | 97.929442 | 0.397084 | 119.886 |
| m:3232 | 58.328725 | 58.277287 | 0.390973 | 118.248 |

Their long HTTP waits are outside their short instrumented Next/run/response
construction bracket. That bracket ends before response-body transmission and
does not cover server accept/runtime scheduling before middleware. Small
sampled pipeline/CPU durations therefore narrow those two requests' location;
they do not exonerate all server queues or describe all66 requests. No server
timestamp is subtracted from a client timestamp, and no thread CPU is measured
across await. The worst HTTP request, m:3184, has no server probe.

### Responsibility and remaining uncertainty

| Candidate | Supported by retained data | What remains unidentifiable |
| --- | --- | --- |
| Load-generator executor |64 outstanding native HTTP/runFn brackets coincide with every drop; the fixed pool has no available iteration slot | Continuous VU state, Go/JS GC and scheduler traces are absent; exclude neither local receive/event-processing delay nor other generator overhead |
| Client resource limits | Reference2 group mean CPU about109.33% of one CPU within its two-CPU quota; no recorded throttling or memory max/OOM event; cumulative memory peak275.24MiB | Boundary averages/counters are not instantaneous headroom; no per-drop run queue, GC or per-process tunnel CPU trace |
| VM scheduling | Client proc/stat boundary deltas report zero steal ticks | Guest accounting is coarse and not a provider-host pause trace; last app normal end snapshot is still missing |
| SSH/network | The unchanged internal SSH path is included in HTTP waiting; response completions pause and recover | No per-channel/socket/tunnel timing or direct-path counterfactual separates transport from endpoint scheduling |
| Server waiting/SSR | Two correlated pipeline brackets remain about0.4ms despite58–98ms HTTP; construction CPU is about118–120microseconds | Sparse probes exclude pre-middleware scheduling and post-response transmission; unprobed requests and server-wide stalls remain possible |

The supported conclusion is **a transient HTTP-completion backlog fills the
fixed64-VU pool, causing27 scheduled arrivals to remain unstarted**. The
underlying cause cannot be uniquely divided among generator/VM scheduling,
SSH/network and server work outside the instrumented bracket. It is not a
measured sustained Yetlet capacity ceiling, a proven GC/SSH/VM defect, or a
reason to change SSR serialization. Reference2 and pair two remain failed and
unqualified; no new speed ratio is computed.

## Minimal next measurement proposal

Do not raise maxVUs or apply a product optimization from this evidence. A future
separately activated bounded transport study should keep the accepted workload
and artifacts, and first declare an isolated private direct-HTTP versus SSH-path
comparison with the same response checks and fixed limits. Include finite
subsecond client/tunnel/server scheduling observations and request-ID ingress /
response-ready boundaries; report observer cost and clock limits. Private
binding/caller adaptation would need its own opt-in contract before any change
or live request. A path effect would narrow the transport question; it would
not by itself prove a particular SSH, VM or kernel mechanism. This proposal
does not activate new execution, an implementation, a hardware purchase or a
consumer SLO requirement.

Documentation checks and final independent review cover this report and the
three indexes. Unique causal attribution remains unresolved at the stated
coverage; bounded retained-data investigation can be accepted without relabeling
the original failure. No new runtime qualification, workload, cloud action,
commit or push is part of this YT-163 delivery.
