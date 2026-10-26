# YT-173 — Full HTTP and Session provider timeline attribution

Date: 2026-10-25 (Asia/Taipei). Status: **Done at bounded correlation-research scope; contract, corrected mapping
and final independent Reviewer PASS, zero issues/required followups; actual cleanup
PASS. Failed requests and all initial CLI/mapping results remain retained.**

## Authority and preserved baseline

The user authorizes alignment of complete HTTP and Session provider timing to
research the earlier `busy` errors: contract first, minimal measurements, current
limits/security unchanged, no replay of passed validation, preserve failures,
remove all test resources and obtain independent Reviewer PASS before Done.
New changes remain uncommitted; no Push is authorized.

The separately authorized seven [YT-172](YT-172-SESSION-ADMISSION-BURSTS.md)
paths are committed as `51c4a49` after strict/builtin Commit Guard PASS, zero
findings, exact tree `b097851ceb18ae35ea31d4fdd796f3b6d7faffc8`. The worktree
is clean before this study. Preserve its 552 valid/96 rejected provider reads,
eight successful recovery reads, original mapping/startup/glue FAIL history
and actual cleanup confirmation. Preserve [YT-171](YT-171-AUTH-CAPACITY-RECOVERY.md)
and its twenty HTTP `busy` failures without rewriting reviews or rerunning its
passing HTTP, lifecycle, provider, browser or security qualification.

Keep ADRs [0001](adr/0001-separation-of-concerns.md),
[0003](adr/0003-renderers-and-loading.md),
[0005](adr/0005-opt-in-capability-modules.md) and
[0006](adr/0006-handler-first-mvc-optional.md). No Core API, normal consumer API,
business authorization, SQL/schema, response/cookie/CSRF, browser/default graph,
Admin/deployment or provider limit change is selected.

## Minimal research build and same-process linkage

Use the existing opt-in Auth/Valkey/PostgreSQL fixture, complete JSON/private SSR
responses and existing full body/header/CSP oracle. Add a private research-only
compiler configuration `yetlet_session_admission_probe`; declare its check-cfg
name in the two affected Cargo manifests. No Cargo feature/dependency or
normal-build public export, timer, capture or environment lookup is added.

Only that explicit compiler configuration exposes a hidden diagnostic linkage
module in the provider for this fixture. It supplies one process-local monotonic
clock, a request-ID task-local scope and bounded record flush. This is an
unsupported research-build interface, not a selected consumer feature or Core
API. Ordinary builds exclude the module and calls. Record its observer cost
and changed artifact identity; do not claim production performance equivalence.

The fixture's outermost application middleware reads a bounded synthetic
`X-Yetlet-Probe` integer. Missing/invalid IDs leave application response behavior
unchanged; tags never establish identity, alter policy, select a tenant or bypass
Session work. No tag is echoed, logged with credentials or put into state.
The scope covers middleware/auth/handler/response construction; it starts after
HTTP parsing and ends before socket transmission completes, not at TCP accept
or final wire delivery. Client spans remain separately measured and joined by
the synthetic ID, not falsely placed on the server's monotonic clock.

For each tagged request retain server entry/response-ready/status, provider
attempt/admitted/active-acquired/returned/outcome, worker reply and a post-permit
drop marker. Explicitly carry request/call IDs into the existing owned worker;
Tokio task locals are not assumed to propagate through spawn. Preserve original
awaits, worker-owned permit lifetime, cancellation/reconnect/drain and error/HTTP
mapping. No lock, delay or hold is added to admission itself except bounded
synchronous observation in this explicit research build.

The post-drop marker is an upper bound on actual release: a different task may
acquire the slot before the marker is written. Reply-before-send is a lower
release boundary. For `busy`, the failed atomic admission attempt proves the
gate was full at its instant; reconstruct individual owners only when timestamp
bounds support them. Report ambiguous boundary overlaps, scheduling and observer
perturbation rather than invent exact release times or unique causation.

Capture at most 65536 fixed-format records per app, aggregate append cost and
overflow count. Only synthetic IDs, monotonic timings, closed outcomes/status
and bounded numeric context are permitted. Never record Session IDs, cookies,
CSRF, state, account/tenant data, SQL text/values or raw errors. Flush only after
HTTP drain and original Store shutdown, to excluded private evidence files.

## Fixed new exposure and stop rules

These are new diagnostic observations, not repeat certification of YT-171's
20s points or YT-172's passed direct-provider cases. Do not compare their
throughput/latency as a controlled optimization result.

| Stage per workload | Once-only selection |
| --- | --- |
| Workloads | JSON then private SSR, one fresh app/pool/eight Sessions per mode, one dedicated PostgreSQL/Valkey allocation shared sequentially |
| Short observation | JSON 500/s for 5s; SSR 2000/s for 5s, fixed 128 VUs, existing k6 constant-arrival engine and full response oracle |
| Small full-HTTP burst | Sixteen VUs with exactly one iteration each; retain actual start spread, no claim of an exact barrier or target req/s |
| Recovery | Eight sequential full authenticated HTTP reads on the same original app/Store/pool after that mode's new exposures |
| Identity | Request IDs unique within the app; separate ranges for observation, burst and recovery, no ID reuse across stages |

Bootstrap only the credentials/dataset required for these requests; reuse old
negative/lifecycle/security proofs instead of replaying them. At most 12500
nominal scheduled reads, 32 burst reads and 16 recovery reads are selected,
plus finite boundary iterations/bootstrap/control. No duration extension,
extra load ladder, permit tuning, retry or favorable replay if `busy` is absent.

Ordinary `busy`/queue/availability pressure remains failed requests within the
finite invocation; retain and correlate every one. Body/header mismatches,
native/capture/coverage/resource/deadline failure stop new exposure and trigger
cleanup, leaving later stages unqualified. Failed recovery stops the other
workload. Missing IDs, ambiguous joins, lost records or inconsistent populations
cannot be silently synthesized as successful correlation. Reconcile native
client counts, all/valid statuses/timings, server spans, provider calls/workers,
SQL/error counters and private logical fixture equality.

If no new `busy` is observed, report that finite outcome and unresolved
historical causation; do not repeat until an error appears. Even a newly
correlated rejection does not retroactively prove the precise sequence of the
old twenty requests, whose internal traces do not exist. Separate observed
new mechanisms, supported historical hypotheses and still-open alternatives.

## Limits, resources and cleanup

Keep YT-171's four active/eight total Session slots, 80ms queue/400ms caller
deadline, 300s absolute TTL/256-byte payload; SQL pool min/max four,
250ms acquire/750ms query bounds, same prepared statements and read-only fixture.
No fault injection, Valkey pause, active-slot hold, production configuration or
network topology change. Use the same accepted Valkey 9.1.2/PostgreSQL 18.6
pinned digests and private loopback/Unix socket setup.

| Resource | Fixed research bound |
| --- | --- |
| Each app | CPUs 12–13, two CPU quota, 512MiB/zero swap, 128 tasks/512 FDs, Nice 10, no restart, 180s unit bound |
| PostgreSQL | CPUs 14–15, two CPU quota, 512MiB/zero swap, 64 PIDs/1024 FDs, 64MiB shm, network none, private ext4 data at most 2GiB |
| Valkey | CPU 10, one CPU, 128MiB/zero swap, 64 PIDs/1024 FDs, 64MB noeviction, no persistence/restart, UID 10001 and owned read-only/tmpfs setup |
| Load/control | CPUs 16–19, four CPU quota, 1GiB/zero swap, 128 tasks/512 FDs, Nice 10, no restart |
| Data | 128MiB per raw file, 256MiB total new evidence, at least 8GiB initial free disk |
| Time | 180s missing pinned-input acquisition; 600s new build/check allowance; one 600s absolute service clock with 180s cleanup reserve, no reset/replacement run |

Reuse the corrected timeout/ownership/cleanup seams: cap each command to the
remaining work budget; preserve stop failures while still attempting bounded
owner-verified removal; treat an already-unloaded unit as absent only after
explicit state/PID verification. Preserve all command exits and startup failures.
One smallest canonical glue correction per distinct seam is bounded; repeated
signatures stop. Never replay completed measurements or reset their clock.

On every outcome stop/drain/reap app, HTTP handlers, Session manager/driver,
SQL pool, k6 and controller; remove exact-owned containers/units/listeners,
private data/socket/password/cookie files and newly acquired unused image
references. Retain cached inputs. Verify actual absence independently of a
single wrapper exit flag. Preserve all raw failures; no active test resource
may remain. No Azure/shared service or host configuration change is needed.

## Acceptance and scope closure

Independent contract review precedes implementation/acquisition; concrete
mapping review precedes startup. Use only new diagnostic scope/capture checks,
locked/offline research build/lint, changed normal-branch compile/format and
new caller-plan inspection. Reuse all passed old tests and qualifications.
No new observability framework, validator framework or general metrics product.

Final independent Reviewer PASS precedes Done. Update this report, Backlog,
Roadmap, index and affected sample/provider usage with actual evidence and
limits. Completion means bounded correlation research, not that a single
historical cause or higher capacity is guaranteed. No new-change Commit/Push,
release-gate expansion, provider tuning or further item is activated;
YT-096/D1–D6 and YT-020 remain unchanged.

## Concrete implementation before startup

Independent contract Reviewer
`/root/yt173_http_session_contract_review_20261025` returns **PASS, zero
issues/required followups** before implementation/acquisition. The explicit
compiler cfg adds one hidden provider linkage module; ordinary exports/timers/
capture/env lookups are absent. No Cargo feature or dependency/version changes.
The two manifest edits declare only the custom check-cfg name.

The new scope/worker test passes (one new test, two old tests filtered). It
checks independent concurrent request scopes, explicit identity transfer into
spawned workers and no ambient scope or sensitive fields. Locked/offline
research build/strict Clippy and cfg-off ordinary application/provider compile
pass. Existing API/security/provider tests and YT-172's probe are not replayed.
Worker trace guards are declared before the original permit tuple, so their
post-drop marker follows original running-count/permit destruction. All original
awaits, error mapping and owned worker lifetime remain intact.

Four new caller-plan inspections pass without HTTP: JSON/SSR observe and burst.
Two initialization failures remain: an unsupported inspect CLI flag, then the
inspect command's opt-in environment forwarding. Local CLI help identifies the
required `--include-system-env-vars` flag; only the failing new initialization
is corrected, with no workload or accepted test replay and no product change.
The complete body/header/CSP oracle remains unchanged; new conditional tags,
metadata IDs and fixed scenarios affect only explicit `YT173_STEP` selections.

The pinned PostgreSQL image is acquired once; the pre-existing Valkey image is
retained. No service has started. The concrete plan uses new exact-owned names,
ports, namespace and private files. Canonical execution reuses YT-171's fixture,
bootstrap, cgroup/resource and stop/drain seams, applies command deadline caps
and ownership-verified force-removal fallback, and verifies already-unloaded
unit absence. Full trace/raw analysis follows service removal. Mapping/live
outcomes/actual cleanup and final results review remain pending.

## Sole execution, coverage and retained failures

The corrected mapping Reviewer
`/root/yt173_http_session_mapping_review_20261025` returns **PASS, zero
issues/required followups** before the only service startup. The initial review
finds two P2 exception paths in task-local glue: outer fallback cleanup could
skip later resources on a timeout; caller sampling failure could skip the native
exit/reap receipt. The same canonical seams are corrected once before startup;
initial FAIL/source/manifests remain. No product code, model, budget or passing
check changes during that correction, and no load replay is used to verify it.

One dedicated Valkey/PostgreSQL allocation and two sequential app instances
complete. Four k6 invocations exit 0 and are waited/reaped; all selected IDs join
exactly one server HTTP span and one provider call. Every successful call joins
one worker; every failed call has the expected busy/no-worker/503 chain. Across
12550 selected HTTP requests there are 12538 exact-oracle valid 200 and twelve
503s, zero generator drops, interrupted/incomplete populations, invalid 200,
missing/duplicate joins or record overflow. The two trace files contain 37638
records. Bootstrap/control requests are untagged and excluded from this cohort.

The native execution exits 0, cleanup completes in 18.200s from the original
600s service clock, and the outer launcher finishes in 18.779s. No service,
workload or old passing proof is restarted/replayed; no exposure is extended
until an error appears. The initial two caller-inspect initialization failures
and initial mapping FAIL remain alongside accepted results.

| New phase | Started / completed | Valid 200 / 503 | Client raw HTTP p95 / p99 ms, all | Client raw HTTP p95 / p99 ms, valid |
| --- | --- | --- | --- | --- |
| JSON observe | 2501 / 2501 | 2501 / 0 | 1.284469 / 2.509755 | 1.284469 / 2.509755 |
| JSON burst | 16 / 16 | 16 / 0 | 3.874155 / 3.874155 | 3.874155 / 3.874155 |
| SSR observe | 10001 / 10001 | 9989 / 12 | 0.941433 / 2.782430 | 0.939260 / 2.785456 |
| SSR burst | 16 / 16 | 16 / 0 | 3.313396 / 3.313396 | 3.313396 / 3.313396 |

JSON observe has 2500 valid completions in its half-open 5s client window
(500/s), plus one tail. SSR observe has 9986 in-window (1997.2/s), three tails
and twelve 503s out of 10001 completed (0.119988%). These are short instrumented
observations, not qualification of the old 20s points. Raw nearest-rank timings
and native interpolated summaries remain separate. Full-call all/valid timings,
window/drain counts and raw hashes are retained in `timeline-analysis.json`.
Do not divide the burst's sixteen requests by its maximum-duration bound to
invent a req/s result.

Each mode's eight sequential recovery reads validates the full authenticated
JSON/private SSR body and headers on the original PID/Store/pool, with zero
Session/SQL/denial errors. JSON PID 3110929 and SSR PID 3111773 are unchanged
within their mode. Response-ready-to-next-entry orchestration gaps are
322.010/310.937ms from observe to burst and 393.345/385.272ms from burst to
recovery for JSON/SSR. No instantaneous recovery or long-window SLO is claimed.

## What the new rejection timelines establish

For every one of the twelve SSR rejections, the failed acquisition interval
is 107–654ns wide. Eight distinct other requests' admission-after-acquire to
reply-before-send intervals cover that entire interval. The conservative
possible-owner set also contains exactly those eight IDs. Four of the eight
already hold active permits throughout; because active capacity is four, the
other four are admitted but pending active-slot acquisition. This is stronger
than inferring concurrency from client millisecond buckets.

| Rejected synthetic ID | Server failed-attempt start ms | Guaranteed admission owners | Guaranteed active owners | Possible admission-owner candidates |
| --- | --- | --- | --- | --- |
| 59 | 40.621508 | 8 | 4 | 8 |
| 62 | 40.663107 | 8 | 4 | 8 |
| 66 | 40.201753 | 8 | 4 | 8 |
| 67 | 40.485536 | 8 | 4 | 8 |
| 69 | 40.102158 | 8 | 4 | 8 |
| 70 | 40.053562 | 8 | 4 | 8 |
| 78 | 40.804334 | 8 | 4 | 8 |
| 80 | 40.705134 | 8 | 4 | 8 |
| 81 | 40.577755 | 8 | 4 | 8 |
| 82 | 40.760588 | 8 | 4 | 8 |
| 83 | 40.533982 | 8 | 4 | 8 |
| 2108 | 1052.367097 | 8 | 4 | 8 |

The direct observed mechanism is **full bounded admission: four active plus
four pending active acquisition**, followed by immediate `Busy`/503 without a
provider dispatch or SQL lookup for the rejected request. All eight owners'
replies are still in the future across each failure interval, so these events
are not explained by retaining permits solely after replies have arrived.
The original worker-owned lifetime is preserved. This does not infer precise
release instants from the later post-drop markers.

Eleven rejections occupy the server-clock band 40.053562–40.804471ms. Client
starts are spread over offsets 29–41ms, illustrating why client start order is
not provider arrival order. Request 59 has a 13ms quantized full call, native
connecting 12.642657ms and native HTTP duration 0.344387ms. This supports a
connection/setup and arrival-coalescing context; it does not assign every
startup delay to one client, kernel or server scheduler.

The remaining rejection, request 2108, occurs at server time 1052.367097ms on
an existing connection (connecting 0; blocked 0.004907ms). The four active
owners 2100–2103 have acquired-to-reply spans 2.853–4.014ms. Four admitted
requests 2104–2107 are still waiting for active capacity at the failed instant;
their eventual acquisition spans reach 1.730213ms, below the unchanged 80ms
queue bound. This event shows a reply-path/service-rhythm component beyond
initial connection setup. The span combines worker scheduling, redis-rs,
transport and Valkey; it is not Valkey CPU execution time or a unique diagnosis.

k6's HTTP duration omits initial connection setup; connecting/blocking fields
provide separate client context. Native metric point timestamps are emitted
at request completion, not provider entry. Interpret these fields using the
[official k6 metrics reference](https://grafana.com/docs/k6/latest/using-k6/metrics/reference/),
not as additive independent buckets or a shared server clock.

## Observer interference remains an explicit alternative

An advisory reduction of the retained data derives a feasible 48.549us-wide
server-origin/epoch interval from all 10001 SSR native client duration endpoints
and causal server-span containment. It assumes stable wall-versus-monotonic
rate over these five seconds, includes sub-microsecond parsing/rounding allowance
and checks data consistency; it is not an exact synchronization measurement.
The conservative server-only owner proof above does not depend on this offset.

Under that explicit assumption, the whole feasible failed-attempt intervals of
the first eleven requests fall inside the recorded Valkey container-proc
`docker exec` command interval of the first resource sample. Request 2108 is
outside such intervals. The sample/command timestamps include dispatch and
control overhead, not only execution inside Valkey. This is evidence of a
possible observer interaction, not proof that sampling causes those eleven
errors or that every error is an observer artifact. Keep the full bounds and
association rows in `advisory-client-clock-bounds.json`; do not promote this
advisory clock assumption into a blocking qualification or historical fact.

The 26 resource samples cost 3.182s in aggregate, maximum 0.143s each, with a
largest sample-start gap 1.140s. These costs and the process-local trace can
perturb the selected profile. Trace append/lock costs total 3.928ms for JSON
(mean 519ns/event, maximum 46758ns) and 7.901ms for SSR (mean 263ns/event,
maximum 29870ns). They exclude clocks/atomics/branches. No zero-overhead,
production-equivalent latency or pure hardware-capacity claim follows.

## Burst, SQL and timing context

Both sixteen-VU HTTP bursts validate all sixteen responses without busy.
Their server-entry spreads are 2.207ms JSON and 3.984ms SSR, unlike YT-172's
sub-millisecond direct-provider barrier waves. Sixteen client VUs therefore do
not imply sixteen simultaneous provider calls. Do not overwrite YT-172's
retained direct-provider rejections with this different-layer observation.

| Cohort | Server HTTP all / valid p95 us | Admitted acquisition p95 us | Acquired-to-reply p95 us | Reply-to-post-drop marker p95 us |
| --- | --- | --- | --- | --- |
| JSON observe | 948.741 / 948.741 | 1.191 | 551.939 | 1.276 |
| JSON burst | 505.502 / 505.502 | 0.835 | 247.748 | 0.641 |
| JSON recovery | 531.511 / 531.511 | 0.536 | 279.280 | 0.651 |
| SSR observe | 539.940 / 539.964 | 0.687 | 239.367 | 0.735 |
| SSR burst | 498.027 / 498.027 | 0.495 | 243.668 | 0.624 |
| SSR recovery | 543.775 / 543.775 | 0.496 | 298.635 | 0.797 |

All SQL errors and business denials remain zero. The rejected calls stop before
SQL. Across measured/burst/recovery cohorts there are 12538 successful account
and 12538 document lookups; final stopped account totals add sixteen bootstrap
lookups per app, yielding 12570 accounts/12538 documents overall. Phase histogram
count/bucket/error deltas reconcile; pool acquisition p95 is bounded by 100us.
Accounts/documents, schema/indexes/constraints remain equal; no business writes
or owner policy edits occur. This supplies no evidence for enlarging the SQL pool.

Observed cgroup CPU-seconds per interval second during JSON/SSR observe are
0.229/0.401 app, 0.879/1.511 client/control, 0.131/0.205 PostgreSQL and
0.071/0.106 Valkey, under fixed quotas. Sparse samples have zero throttle and
memory high/max/OOM counters. Largest sampled cumulative memory peaks are
13.305MiB app, 156.730MiB client/control, 81.969MiB PostgreSQL and 11.426MiB
Valkey. These are sampled charged peaks, not allocations or guaranteed final
trace-flush peaks; they cannot explain sub-millisecond scheduling alone.

The four raw metric files total 71056007 bytes; all evidence including joined
rows is below 90MiB and every file below 128MiB. PGDATA peaks at 48427266 bytes,
within its original bound. Full analysis runs only after services are removed.
All raw per-request joins and separate all/valid/client/server/provider timing
populations remain in the evidence directory.

## Cleanup, conclusion and remaining boundary

Cleanup PASS: both app instances stop with Session manager drained and SQL pool
closed; k6/controller/launcher are waited/reaped. Both exact-owned containers
and main PIDs are absent, both transient units inactive with MainPID zero,
listeners closed, private data/socket/password/cookie files removed. The newly
acquired unused PostgreSQL image reference is removed without force/prune;
pre-existing Valkey cache is retained. No cloud/shared service/host setting is
changed and no active test resource remains.

Keep four active/eight total admission slots. The new data establishes the
immediate full-gate mechanism and separates a startup arrival cluster from a
later outstanding-reply episode; it does not show a permit leak, queue timeout,
SQL failure or authorization fallback. Normal builds contain no new diagnostic
API/capture/timer or dependency. Old YT-171/172 sources/evidence, failures and
reviews remain intact; these short differently instrumented results do not
prove the exact cause of each historical rejection or qualify a higher rate.

If further selected, the next useful bounded question is a non-intrusive
host-side resource-observer comparison, avoiding container-proc `exec` during
load while preserving all limits/workload/oracles. That is a new control
experiment, not implemented or activated here. It would test observer influence
rather than prematurely tune the provider.

Final independent Reviewer
`/root/yt173_http_session_results_final_review_20261025` returns **PASS, zero
issues/required followups**. Thirteen source paths, 549 evidence files, eighteen
mapped inputs and thirty-three accepted source/evidence bindings remain
hash-identical throughout review; native role/model/effort match the resolved
reviewer selection. Only documentation status/review references change afterward.
The backlog is Done at bounded correlation-research scope; failed calls and
initial CLI/mapping results retain their original statuses. All thirteen new
source paths remain unstaged/uncommitted; no Push or release-gate expansion is
authorized. YT-096/D1–D6 and YT-020 remain unchanged.
