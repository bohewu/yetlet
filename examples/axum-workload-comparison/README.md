# Native Axum / Yetlet workload comparison — YT-143

The optional [YT-176 fail-fast HTTP admission](../../docs/YT-176-HTTP-ADMISSION.md)
uses `--features auth-workload-admission --bin auth-postgres`. With that feature,
set `YT176_HTTP_MAX_IN_FLIGHT` to an explicit application-selected cap; for example,
`YT176_HTTP_MAX_IN_FLIGHT=4` selects four concurrent HTTP futures. Unset means
no gate. See the [capacity selection guide](#http-admission-capacity-selection)
for the retained 4/8/16 choices; none is a universal default.
Values must be ASCII digits in 1–1024; invalid input fails startup
before backend connection. The setting is read once, with no hot reload or new
Settings-provider API. Without the feature the variable is not read. Existing
backend/fixture configuration is still required; use only its isolated topology.

One application-owned gate covers all Session-scoped API/SSR/login/logout routes
before Session/Auth lookup and through Session finalization. Capacity exhaustion
returns generic private/CSP-protected 503, without a cookie, login redirect or
Retry-After; no extra queue, request deadline or automatic retry is installed.
Health and optional observation endpoints stay outside. Session remains four
active/eight total with unchanged deadlines and worker retention. The HTTP permit
ends at response construction or cancellation, not body delivery; outstanding
backend work can outlive it. This protects bounded work, not burst acceptance,
distributed fairness or a higher capacity/SLO.

New focused composition tests use
`cargo test --offline --locked --manifest-path examples/axum-workload-comparison/Cargo.toml --features auth-workload-admission --bin auth-postgres admission::tests::`.
They use in-process synthetic Session/Auth/finalizer boundaries, not live backend
qualification. Reuse prior Auth/Valkey/SQL/security evidence without replay.

The [YT-173 HTTP/Session timeline study](../../docs/YT-173-HTTP-SESSION-TIMELINE.md)
requires `RUSTFLAGS='--cfg yetlet_session_admission_probe'` with the existing
`--features auth-workload-observation --bin auth-postgres` research build.
Only that build scopes a sanitized `X-Yetlet-Probe` synthetic ID and writes
bounded non-secret trace records to excluded `YT173_TRACE_PATH` after drain.
Missing/invalid tags keep normal response behavior, and ordinary builds contain
no diagnostic linkage. New `YT173_STEP=observe` selects only JSON500/s5s or
SSR2000/s5s; `burst` selects sixteen VUs with one iteration each. Keep the
contract's original limits/full oracles, unique ID ranges, finite exposures,
no-replay and exact-owned cleanup rules. Old selections retain their settings.
Contract/corrected mapping and focused checks pass; the sole new study records
12538 valid responses and twelve correlated busy rejections, with all burst/
recovery reads valid and exact-owned cleanup complete. Normal limits/security
remain unchanged, failures/observer limits retained; final independent Reviewer
PASS has zero issues/required followups, Done at bounded research scope.

The optional [YT-171 staged Auth workload](../../docs/YT-171-AUTH-CAPACITY-RECOVERY.md)
requires `--features auth-workload-observation --bin auth-postgres`. This fixture-
only feature enables bounded pool/query/Session observations and the loopback
`/fixture/observations` aggregate endpoint; feature-off has no observer work or
endpoint. New `YT171_STEP` selections admit only `admission`, `load-250`,
`load-500`, `load-1000`, `load-2000` and `recovery`, with fixed128VUs. Existing
YT-170 selections retain their settings/full oracles. Use the contract
first-pressure stop/recovery/resource/privacy rules; these are configured-profile
observations, not a universal capacity claim. Contract/mapping pass; the sole
allocation records one Session-busy503 at JSON500/s and nineteen at SSR2000/s,
with zero SQL errors or generator drops. Both same-PID100/s recoveries and
exact-owned cleanup pass. Failed points remain failed; independent final Reviewer
PASS has zero issues/required followups. YT-171 is Done at bounded research scope. Existing accepted YT-170 evidence is preserved. No commit/push.

The optional [YT-170 functional fixture](../../docs/YT-170-AUTH-SESSION-POSTGRES-WORKLOAD.md)
requires `--features auth-session-postgres --bin auth-postgres`. It composes
existing Auth/Valkey Sessions with PostgreSQL account/tenant authority and fresh
JSON/Yetlet SSR. Its explicit synthetic sign-in asserts a preverified fixture
identity; it does not implement credential/OIDC verification. Business workload
is read-only, private responses are no-store, and existing/default binaries and
Core/browser graphs are unchanged. Use only the dedicated loopback/Unix-socket
resources and fixed workload described in the contract; never shared stores.
Contract/corrected mapping/final independent reviews and new lifecycle/workload/
cleanup checks pass, with zero reviewer issues/required followups. JSON2000/SSR2001valid responses at100in-windowRPS each have
zero measured drops/errors, HTTP p95 2.116/2.112ms. All owned services/private
material and newly acquired PG image are removed; original failures remain.
See the report for synthetic-login/read-only/reference limits. No commit/push.

The optional [YT-169 thirty-minute study](../../docs/YT-169-PRIVATE-HTTP-SUSTAINED-1800.md)
adds `warmup-soak-5000` and `soak-5000` to `YT165_STEP` for one5000/s1800s
exposure. Only the new measured plan uses16384request sparse diagnostics,
keeping the accepted server bounds; all old plans retain4096. Every response
keeps the same complete body/header/CSP check. Full native metrics use lossless
gzip under predeclared capture/resource/time bounds. Gzip and a new VM allocation
prevent a controlled duration-only comparison with older results. Contract
mapping and final independent reviews pass, with zero issues/required followups. The once-only exposure
retains9000001valid200,0drops/errors,4999.996667in-windowvalidRPS and0.954ms
HTTP p95. All test resources are deleted; full gzip results, localized tails
and one finite observer read gap remain in the report.
New changes remain uncommitted and unpushed.

The optional [YT-168 sustained5k study](../../docs/YT-168-PRIVATE-HTTP-SUSTAINED-5000.md)
adds `warmup-sustained-5000` and `sustained-5000` to `YT165_STEP` for one
5000/s300s exposure. It keeps the accepted workload/oracle,256VUs and YT-167
resource/capture limits. Old5k short and6k long observations remain unchanged;
new-target results are finite profile evidence, not a controlled causal
comparison across allocations or a production SLO. The actual5k/300s exposure
retains1500001valid200 with0drops/errors,4999.983valid in-window responses/s
and0.894ms HTTP p95. All test resources are deleted; full raw results and the
old failed6k population remain. See the report for finite-observation limits.

The optional [YT-167 sustained capacity study](../../docs/YT-167-PRIVATE-HTTP-SUSTAINED-CAPACITY.md)
adds only `warmup-sustained-6000` and `sustained-6000` to `YT165_STEP` for one
300s exposure at6000arrivals/s. It preserves the accepted app/workload/oracle,
256VUs and sparse sampling. Client capture headroom explicitly increases for
full-duration raw evidence; app limits remain unchanged. This finite isolated
test does not establish a production SLO or replay the accepted short points.
The actual300s offered load fails the clean marker:34302drops and63.33ms HTTP
p95, with5884.83valid in-window responses/s. All started requests validate,
all test resources are deleted and the complete failed population is retained.
See the report for different-allocation and finite-observation limits.

The optional [YT-166 capacity refinement](../../docs/YT-166-PRIVATE-HTTP-CAPACITY-REFINEMENT.md)
extends the same caller with `warmup-5000` / `refine-5000`, corresponding6000
and7000 labels in `YT165_STEP`. Only these new points execute in that study,
with a fresh app instance for each rate. The existing oracle, request/timing
code,256VUs and bounded sparse probes remain unchanged; accepted old workload
points and recovery evidence are reused without replay. The once-only new-host
study observes5k/6k clean and7k pressure, retaining9585drops and approximately
6016valid in-window responses/s at7k. These finite results are not a sustained
capacity or production SLO; full evidence and cleanup are in the report.

The optional [YT-165 private capacity study](../../docs/YT-165-PRIVATE-HTTP-CAPACITY-RECOVERY.md)
uses `ssr-capacity-probe.js` with explicit `YT165_STEP`, `YT164_TARGET_IPV4` and
`YT157_PORT`. It admits only the declared finite staircase/recovery plan and
private IPv4/39262, preserves the full SSR response oracle, and fixes256VUs.
It reuses the accepted diagnostic server with sparse bounded probe headers;
its capacity/recovery results apply to the declared isolated profile. It does
not change the path-study caller, default server behavior or Core APIs.

The optional [YT-164 private transport study](../../docs/YT-164-PRIVATE-HTTP-SSH-COMPARISON.md)
uses the diagnostic SSR executable with `YT164_BIND_IPV4` and its existing k6
caller with `YT164_TARGET_IPV4`. Unset values retain `127.0.0.1`; explicit values
accept only that loopback address or canonical RFC1918 IPv4. The binding override
requires `ssr-stage-probe` and does not affect the default executable. A private
binding requires a separately isolated network allowing only the declared load
client; it does not enable authentication or expose a public benchmark service.
Both paths share the same diagnostic executable, workload and full response
checks. This is a transport observation, not a new product-version comparison.

This standalone public synthetic fixture compares the same Axum HTTP engine
through native route registration and Yetlet's App facade. Read the
[source-bound contract](../../docs/YT-143-AXUM-WORKLOAD-COMPARISON.md) before
running it. There is no database, Auth/Session provider, browser runtime or
production-qualification claim. The existing protected Starter is unchanged.

The same binary selects `axum` or `yetlet` at startup, with two Tokio workers.
The modes share plaintext and freshly serialized JSON handlers. SSR compares
fresh Yetlet View/Page rendering with independent handwritten escaped HTML;
it does not isolate router overhead or compare template engines. Every request
renders a dynamic ID in title, text and attributes, including all five HTML
escaping characters and UTF-8. Exact response/CSP tests cover all 20 IDs.

```bash
cargo test --locked --manifest-path examples/axum-workload-comparison/Cargo.toml
cargo fmt --manifest-path examples/axum-workload-comparison/Cargo.toml -- --check
cargo build --release --locked --manifest-path examples/axum-workload-comparison/Cargo.toml
examples/axum-workload-comparison/target/release/yetlet-axum-workload-comparison axum 45379
```

Use a task-private `CARGO_TARGET_DIR` for the controlled comparison; its binary
path differs from this default interactive example. GET `/plaintext`, `/json`
and `/ssr/1` through `/ssr/20` on loopback. Stop with SIGTERM/Ctrl-C; a clean
native shutdown prints `stopped`. Binding to a public address is unsupported.

`workload.js` uses official k6 2.3.0's native constant-arrival-rate executor.
Use its contract-verified task-local executable, never a cloud backend or global
installation. For one measured phase, supply the recorded port and private
output directory (placeholders below):

```bash
env -i PATH=/usr/bin:/bin HOME="$COMPARISON_PRIVATE_HOME" \
  YT143_PORT="$COMPARISON_PORT" YT143_MODE=axum YT143_REPETITION=1 \
  YT143_WORKLOAD=plaintext YT143_PHASE=measured YT143_RATE=1000 YT143_SECONDS=10 \
  "$K6_EXECUTABLE" --no-usage-report --address=127.0.0.1:0 run \
  --out "json=$COMPARISON_OUTPUT/raw.jsonl" \
  --summary-export "$COMPARISON_OUTPUT/summary.json" \
  examples/axum-workload-comparison/workload.js
```

Warmup uses phase `warmup`, rate 250 and duration two seconds. Follow the
contract's exact paired order, fresh processes, separate 64-VU invocations,
one-second idle gap, fixed CPU/memory/time/output limits and cleanup. Do not
add rates, retune, repeat failed points or treat offered rate as capacity.

The script records native metrics plus `yt143_started`, `yt143_http_ms` and
`yt143_full_call_ms`. Bounded tags identify mode/workload/repetition/phase/rate,
status/error/protocol/validity and fixture ID. Per-point metadata carries actual
start/completion and scenario anchor milliseconds without creating timestamp
metric series. HTTP timing is floating milliseconds excluding blocked/connect;
full-call wall time has only millisecond resolution. Retain all attempts and
errors, report all-response and exact-valid latency separately, and stream raw
JSON during analysis. Drops/interruptions have no completed latency sample.

These local workloads are TechEmpower-inspired; HTTP pipelining, database and
Fortunes tests and official certification are absent. One binary cannot compare
dependency/binary footprint. Shared-host short measurements cannot establish
maximum throughput, production SLOs or browser CSP enforcement. Independent
final review is required before YT-143 is Done; PostgreSQL is a separate next task.

The recorded run completed the fixed phases once, with an original tmpfs-output/client-memory failure retained. One independently accepted canonical output-path correction ran only the unstarted second repetition on ext4, under unchanged bounds. Five corrected pairs are comparable; SSR 4,000/s has one native-Axum dropped arrival and is noncomparable. No two valid repetitions or stable overhead was established. Source/checks and owned cleanup are complete; final independent Reviewer `/root/yt143_axum_final_review_20261023` PASS, zero issues and required followups. YT-143 is Done at this bounded scope. See the contract for all results, deviations and evidence locations.

## Optional PostgreSQL comparison (YT-144)

The separate `postgres` binary is enabled only by `--features postgres`; default
builds and the original main/workload remain unchanged. It uses application-local
SQLx 0.8.6 with only Tokio/PostgreSQL features, ordinary Axum State, one explicit
parameterized SELECT, a 16-connection asynchronous pool and fresh JSON responses.
No Yetlet ORM or default database/provider dependency is introduced.

Follow [the reviewed YT-144 contract](../../docs/YT-144-POSTGRES-WORKLOAD-COMPARISON.md)
for the pinned disposable PostgreSQL image, private Unix socket/credentials,
read-only `postgres-fixture.sql`, finite resources and fixed paired load. The
sample must use an isolated synthetic database; do not run the fixture against
existing data. After creating the restricted `yt144_app` role and fixture:

```sh
cargo build --release --locked --manifest-path examples/axum-workload-comparison/Cargo.toml --features postgres --bin postgres
# Arguments: axum|yetlet, loopback port, private host socket directory, private app password file.
/path/to/task-build/release/postgres axum 39000 /path/to/private/socket /path/to/private/app-password
```

`GET /db/1` returns `{"id":1,"randomNumber":7920}`. IDs outside 1–10000 or malformed
IDs receive 400. Pool acquisition waits at most 250 ms; query time is bounded to
750 ms and overall handling to 1,250 ms. Redacted 503/504 responses disclose no
provider details. Startup proves 16 distinct backends and prepares the exact
query on each connection. Shutdown closes the pool and emits bounded error
category totals; there is no per-request log or telemetry endpoint.

The focused new binary test requires `YT144_SOCKET`, `YT144_APP_PASSWORD_FILE`
and `YT144_OWNER_PASSWORD_FILE` pointing only to this disposable fixture. It
checks both routes, read-only grants, preconnection, bounded acquisition/query
timeouts, redaction and close. It temporarily locks the table using the owner
connection outside measurement, then rolls back. `postgres-workload.js` is the
separate k6 script; retain failures and exact tool/source/resource receipts.
Results remain local experimental observations; the final review status follows below.

The separately authorized YT-144 measurement continuation completed the eight
fixed groups once: 100,007 measured exact valid 200s, zero errors/drops/
interruptions and four comparable pairs. Full logical/schema equality and owned
cleanup pass; original observer failures and a corrected case-sensitive
preload-header assertion remain retained. The reconstructed binary matches the
accepted original byte-for-byte; prior passing tests were reused. YT-144 is
Done at this bounded comparison scope after independent final Reviewer PASS,
zero issues and required followups, with no stable-overhead or capacity claim. See the contract for populations, resources, ratios and evidence.

## YT-156 full HTTP SSR product-version comparison

Status: Done at bounded comparison scope after independent contract Reviewer
`/root/yt156_http_ssr_contract_review_20261024` and final Reviewer
`/root/yt156_http_ssr_final_review_20261024` PASS, zero issues/required followups.
[Contract](../../docs/YT-156-HTTP-SSR-COMPARISON.md) selects the separate `ssr`
binary and `ssr-workload.js`; old binaries/scripts and the lock are unchanged.
Two new offline/locked release builds use actual products at `90f83867` and
`0580a0a`, identical sample overlays and default dependencies. Each request to
`/ssr/small`, `/ssr/medium`, `/ssr/large` constructs the retained YT-145 fixture
and returns an actual Page: 8/128/1024 rows, 2383/31063/245207 UTF-8 bytes.
No provider, database, cache, custom renderer or product optimization is installed.

Use task-local pinned k6 and the reviewed isolation/commands, not the old scripts:
`YT156_PORT`, `YT156_RATE`, `YT156_SECONDS`, `YT156_SIZE`, `YT156_MODE`,
`YT156_REPETITION`, `YT156_PHASE` identify the fixed workload. Mode is reference
or current, phase warmup (125/s,2s) or measured (250/1000 per second,10s), repetition
1/2. Exact literal bodies, content/security headers and HTTP/1.1 are checked.
24 order-balanced measured phases retain 150000 nominal arrivals plus separate
6000 nominal warmups, all failures/context/resources with finite cleanup.
No replay of older workload proofs or claim of maximum capacity, universal
speedup, browser CSP enforcement or production readiness. Final independent
Reviewer PASS precedes Done; no commit or push.

The fixed24-phase population completed once: 150019 measured and 6014 warmup
valid200 responses, zero errors/drops/interruptions, 72 exact live probes and all
48 k6 exits0. All24 measured points / twelve pairs meet comparability; eight
short small-page warmup resource comparisons remain unverified, as documented.
One new focused test, focused rustfmt and two ordinary release builds pass.

At1000/s, large p50 falls23–25 percent and app CPU about25 percent, while p95
rises5–6 percent. Medium p95 at1000/s rises about19 percent in both orders;
small/medium-low points improve at several percentiles, and RSS has no consistent
reduction. Full tables/raw data/limits are in the linked report. Fixed arrival
throughput is not maximum capacity and the optimization bundle is not a universal
latency improvement. Source-binding and optional post-exit cgroup observation
failures/corrections are retained, with no test/load replay or deadline renewal.
All25 owned units are inactive,73 PIDs absent, cgroups/listener closed; inert
artifacts remain. Final independent Reviewer `/root/yt156_http_ssr_final_review_20261024` PASS,
zero issues/required followups, verifies all eight file/data/history/source/
executable/cleanup bindings. Only five document status/reviewer references changed
after acceptance; no commit/push.

## YT-157 medium HTTP p95 attribution

Status: Done at bounded attribution scope after final independent Reviewer
`/root/yt157_medium_tail_final_review_20261024` PASS0 and corrected contract Reviewer
`/root/yt157_medium_tail_contract_review_20261024` PASS0; original denominator
FAIL/correction retained. [Contract](../../docs/YT-157-MEDIUM-HTTP-TAIL-ATTRIBUTION.md)
selects default-off `ssr-stage-probe = []` for the existing ssr binary, private
stage_probe module and separate ssr-stage-probe.js. Normal handler/fixture/
Page/body/headers, products, dependencies and locks remain unchanged.

Sampled IDs divisible by32 record native middleware Next pipeline and nested
fresh construction wall times; bounded request-only X-Yt157-Id joins client and
server without response timing headers. Records dump only after drain, not per
request. These observer paths can perturb latency and are not uninstrumented
historical costs or pure renderer CPU fractions. Retained native components are
analyzed first; four new diagnostic phases only cover medium1000/s, with fixed
source/order/warmup/isolation/caps and correlation checks. No prior proof/load
replay, optimization or product API. Final independent PASS precedes Done; no commit/push.

YT-157's fixed four diagnostic phases completed once:40003 measured and1002
warmup valid200s,zero errors/drops/interruptions,eight k6 exits0. All1284 stage
records join exactly (313 measured+8warmup perapp),zero missing/duplicate/extra/
overflow/invalid IDs. One new focused test,rustfmt,two release builds pass;
all measured pairs meet headroom/measurement conditions. Five owned units are
inactive,13 PIDs absent,cgroups/listener closed; inert evidence retained.

Historical40005 responses localize most tail duration to waiting, which also
includes dispatch/transport/scheduling. New diagnostic HTTPp95 rises10.30/9.91
percent; sampled pipelinep95 rises18.10/1.63percent. Unchanged construction code
has higher sampled times, so unique renderer/allocator/host cause remains
unresolved; no optimization or exoneration of indirect product effects follows.
Only14–20 selected tail requests perpopulation; observer/caller/context effects
and all unfavorable extrema remain explicit. Initial contract countFAIL and one
arithmetic artifact-name correction are preserved without test/load replay or
new clocks. Full raw/distributions/IDs/limits are in the linked report. Final
independent Reviewer `/root/yt157_medium_tail_final_review_20261024` PASS, zero
issues/required followups, confirms all nine reviewed file/data/history/source/
executable/cleanup bindings. Only five document status/reviewer references changed
after acceptance; unique cause remains unresolved, no commit/push.
# YT-158 synchronous render-history diagnostic

`ssr-context-study` is a separate, default-off, Linux x86_64 GNU-only diagnostic
mode of the `ssr` sample. Select it without `ssr-stage-probe`. It uses the same
fresh medium fixture and actual Page response conversion, without an HTTP server
or runtime during measurements. It does not change Yetlet APIs or products.

The fixed experiment compares construction after a construct/drop predecessor
with construction after a Page/render/response-drop predecessor. It keeps eight
counterbalanced blocks, 256 warmup and 256 measured cycles per context/group,
wall/thread CPU clocks, 128 empty brackets and coarse read-only allocator /
scheduler context. Output is JSON lines after measurement. The task runs four
new processes in reference1/current1/current2/reference2 order on logical CPU12;
the process verifies its single-thread CPU affinity. For this host's task-owned
binaries, an equivalent invocation is `taskset -c 12 /path/to/ssr current 1`.
Build using `cargo build --bin ssr --features ssr-context-study --release
--offline --locked --jobs 2` and preserve the exact source/artifact identity.

This is an intervention on render history, including allocator/cache/instruction
and lifetime context. It cannot identify allocator bins, pure off-CPU time or
the historical HTTP p95 cause. See
[YT-158](../../docs/YT-158-RENDER-HISTORY-CONTEXT.md) for the contract, bounds and
independent review. Status: Done at bounded attribution scope; independent final Reviewer
`/root/yt158_render_context_final_review_20261024` PASS0. No optimization,
commit or push.


YT-158 measured all four processes once:16,384 construction times,16,384 warmup
cycles,512 empty brackets and64 groups. Current/reference CPU p50 is0.55–3.10%
lower after construct-only history and0.50–1.76% lower after render/drop history.
History CPU p50 effects range+0.44–2.10%; no unique allocator/HTTP cause or
optimization target is established. All data/tails, initial direct-Page header
assertion failure and corrected feature artifacts remain retained. One new test,
format and corrected builds pass; owned cleanup complete. Final independent
Reviewer `/root/yt158_render_context_final_review_20261024` PASS0.

## YT-159 actual HTTP worker thread CPU

`ssr-thread-cpu-probe` is a separate default-off extension of `ssr-stage-probe`,
requiring Linux x86_64 GNU. Select it without `ssr-context-study`. It adds
same-thread CPU/wall clocks only around synchronous fresh Page construction on
selected actual HTTP requests, plus one separate empty bracket and local thread /
logical CPU identities. Middleware pipeline timing remains wall-only across await.
Actual Page response conversion, body/CSP and normal behavior stay unchanged.

Build with `cargo build --bin ssr --features ssr-thread-cpu-probe --release
--offline --locked --jobs 2` in a task-private target directory. Use the existing
`ssr-stage-probe.js` caller and private request IDs; metric names remain yt157,
while this task has distinct binary/source/population identities. Follow the
four fixed new-feature phases and resource/cleanup contract in
[YT-159](../../docs/YT-159-HTTP-WORKER-THREAD-CPU.md). Do not rerun accepted groups.

Signed wall-minus-CPU includes observer overhead and possible descheduling; it
is not pure off-CPU time. Thread CPU is never measured across an async await.
The extra clocks/queries and larger selected-record storage perturb worker /
allocator/cache context, so these results cannot replace historical HTTP data.
Status: Done at bounded attribution scope; independent final Reviewer
`/root/yt159_http_worker_cpu_final_review_20261024` PASS0. No product optimization,
commit or push.

YT-159 completed all four new-feature phases once:40,002 measured/1,003 warmup
valid200,zero errors/drops/interruptions;1,284 samples and2,568 construction /
empty clocks join with valid same-thread identities. Construction CPU p50
increases27.83%/36.80% current/reference like wall; fixed empty CPU work also
increases28.65%/33.03%. This supports CPU-visible worker context without proving
an intrinsic algorithm/frequency/allocator cause or resolving historical HTTP p95.
One new test, format, two builds and owned cleanup pass. All raw tails and one
report-format-only preparation correction remain; no load replay or optimization.
Final independent Reviewer
`/root/yt159_http_worker_cpu_final_review_20261024` PASS0.


## HTTP admission capacity selection

YT-180, 2026-10-25. Status: **Done at documentation-only scope after independent
Reviewer PASS, zero issues and required followups.**

Independent Reviewer `/root/yt180_http_admission_capacity_guide_review_20261025`
verified all four document, 12 evidence and 19 retained/source hashes before/
after review; native reviewer role/model/effort match the saved configuration.
New document structure/link/whitespace checks pass (75 documents/180 backlog
items). Only completion/review metadata changes after PASS. No runtime tests,
builds, workload or analysis programs are added or replayed; no resources start.
New changes remain uncommitted/unpushed. YT-096 D1–D6 remain Pending and YT-020
remains Blocked.

This guide helps an application choose among the existing reference caps **4,
8 and 16**. The gate, setting and choice remain application-owned and opt-in,
under ADRs [0005](../../docs/adr/0005-opt-in-capability-modules.md) and
[0006](../../docs/adr/0006-handler-first-mvc-optional.md). No framework default,
new API, Settings-provider contract, dependency, queue, retry or limit change is
selected. This is a document-only synthesis of accepted evidence, with no new
or replayed tests, builds, workloads or analysis programs.

### Activation and the meaning of the number

- Select the existing `auth-workload-admission` feature for `auth-postgres`, then
  explicitly supply `YT176_HTTP_MAX_IN_FLIGHT=4`, `=8` or `=16` at application
  startup. These are alternative setting values, not three commands to run.
  Existing isolated backend/fixture configuration is still required.
- Unset the variable to omit the gate when the feature is present. Zero is
  invalid, not a disable value. Without the feature, the module and environment
  lookup are absent. Startup reads the value once; changing it requires an app
  restart. The existing parser accepts ASCII digits representing 1–1024; that
  parser range does not qualify the performance of every accepted value.
- The value bounds concurrent HTTP futures in one process. All existing
  Session-scoped API/SSR/login/logout routes share one gate and its slots; it is
  not a fresh allowance per endpoint or a requests-per-second rate. Health and
  optional fixture observations remain outside. Separate processes have
  independent gates: there is no shared cluster quota or linear scaling claim.
- HTTP admission is separate from Session **four active/eight total**, 80ms
  active wait/400ms per-API deadline, and the four-connection SQL pool. The HTTP
  permit spans response construction and Session finalization, not body delivery;
  cancellation may release it while started backend work is still owned by its
  worker. A small HTTP cap therefore cannot guarantee zero downstream pressure.
- A full gate fails immediately with the existing private generic **503**, not
  rate-quota **429**. There is zero added HTTP waiting or automatic retry;
  downstream Session/SQL waiting remains bounded by existing contracts. Preserve
  Auth/authorization/CSRF, no-store/CSP, worker ownership and drain semantics.
  No retry behavior for reads or uncertain writes is changed by this guide.

### Retained outcomes and suitable application priorities

These are the accepted isolated [cap-four](../../docs/YT-177-HTTP-ADMISSION-WORKLOAD.md),
[cap-eight](../../docs/YT-178-HTTP-ADMISSION-EIGHT.md) and
[cap-sixteen](../../docs/YT-179-HTTP-ADMISSION-SIXTEEN.md) cohorts. They use the
same release binary/caller and fixed JSON 500/s × 5s, SSR 2000/s × 5s,
sixteen-request bursts and eight serial recovery reads per mode. The Auth/
Valkey/PostgreSQL fixture and Session/SQL limits stay fixed.

| HTTP cap | Selected / valid 200 | Checked 503 / selected | Outer-only / provider Busy / QueueTimeout | Application priority for considering this reference |
| --- | --- | --- | --- | --- |
| 4 | 12550 / 12452 | 98 / 12550 (0.781%) | 98 / 0 / 0 | Favor earlier rejection and a smaller admitted HTTP population when the application accepts more outer failures. Both bursts still fail; the clean provider result is limited to this cohort. |
| 8 | 12549 / 12475 | 74 / 12549 (0.590%) | 74 / 0 / 0 | Consider the observed compromise of fewer outer failures than four with no measured provider Busy. It is a protection-oriented candidate, not a zero-pressure guarantee or selected default. |
| 16 | 12552 / 12540 | 12 / 12552 (0.096%) | 7 / 5 / 0 | Consider when the application prioritizes the observed lower total failure rate and explicitly tolerates provider Busy. A larger gate admits pressure downstream; it does not enlarge Session or SQL capacity. |

Every checked 503 remains a failed request. In cap sixteen, two Busy outcomes
occur in the JSON burst and three in SSR observe; seven outer-only failures
also occur in SSR observe. Cap-eight/four failures are outer-only in their
accepted captures. All three cohorts have zero observed QueueTimeout, SQL
error, denial, drops, incomplete calls and capture overflow; both eight-read
recoveries and owned cleanup pass in each. These facts do not make the failed
load points error-free or establish long-duration capacity.

Outer-only attribution is conditional on the research source graph, actual
feature/cap, valid fixed GETs, full response oracles, complete trace and reconciled
provider/SQL counters. The trace has no HTTP gate reason/admit/release marker;
the generic 503 body alone cannot distinguish an outer rejection from provider
Busy. Do not infer exact gate occupancy or permit lifetime from these totals.

### Latency and failure tradeoffs

The following HTTP-duration p95 values use **raw nearest-rank, all completed
responses including failures**, in milliseconds. Native k6 interpolated p(95),
valid-only/failed populations, full-call timing, tails and server spans remain
separate in the linked reports and retained evidence. Recovery uses a different
Python-client population and is not included in these percentiles.

| Phase | Cap 4 | Cap 8 | Cap 16 |
| --- | --- | --- | --- |
| JSON observe | 1.316030 | 1.244038 | 1.259762 |
| JSON sixteen-request burst | 1.041342 | 1.885702 | 1.224517 |
| SSR observe | 0.966173 | 0.902209 | 0.908322 |
| SSR sixteen-request burst | 1.493250 | 2.140189 | 1.988075 |

Eight has fewer failures than four but longer burst p95 in these cohorts.
Sixteen has fewer total failures than eight, slightly higher observe p95 and
lower burst p95, while exposing provider Busy. Faster rejected responses can
also affect an all-response percentile; compare valid and failed populations
before choosing a cap. Increasing the number is not a monotonic latency rule.

Immediate server rejection does not guarantee a short client tail. Retained
SSR outer-failure full-call maxima are 27ms/54ms/5ms at caps 4/8/16, from
81/66/7 failed SSR responses respectively. Cap-sixteen provider-Busy full-call
maxima are 1ms for two JSON failures and 4ms for three SSR failures. Keep these
small, different populations and integer full-call resolution visible; never
subtract client and server clocks to assign a unique scheduler/network cause.

The observe executors record JSON 2501/2500/2503 and SSR 10001 each at caps
4/8/16. Retain those boundary differences. Valid in-window JSON/SSR rates are
499.0/1985.0, 500.0/1987.6 and 499.8/1997.8 per second respectively; successful
end-boundary/tail completions remain separate in the reports. Cap sixteen's
extra JSON iterations do not establish throughput above the configured rate.
Fresh sequential allocation, arrival spacing, warming, connection and observer
variation remain. These short, unrandomized cohorts provide no statistical,
physical-capacity, production-SLO or universally optimal cap conclusion.

### Application decision without a universal default

Choose the application's acceptable **total 503 rate**, downstream Busy/
QueueTimeout budget and valid-response p95/p99/tail requirements together.
Consider four for earlier rejection, eight for the observed provider-protection
compromise, or sixteen for the observed acceptance-rate priority with explicit
Busy tolerance. Keep startup selection explicit; this guide chooses none for
the caller and imposes no hardware, operator or SLO completion requirement.

For this reference, the next useful decision is the application's failure/
latency tradeoff between eight and sixteen, with four retained as the smaller
admission alternative. Preserve Session and SQL limits while making that
choice. Any additional cap, sustained workload, implementation or provider
change needs a separate question and authorization; none is activated here.
Existing quota-based rate limiting remains a separately opted-in capability.


## Optional cross-host Auth/Session reference — YT-184

The [contract and current evidence](../../docs/YT-184-AZURE-AUTH-SESSION-RESTART.md)
select `auth-cross-host` on the existing `auth-postgres` fixture, without
installing Auth/SQL/Session/TLS in default builds. Existing `YT170_*` local
Unix-socket/loopback mode remains unchanged. Cross-host mode is selected only
with `YT184_TRANSPORT_CONFIG_FILE` and the compiled feature; absent feature
plus that selector fails startup. This is a fixed isolated reference, not
general deployment discovery or a framework server API.

An owner-private file of at most4096bytes supplies the following shape:

```json
{
  "pg_host": "sql.yt184.test",
  "ca_file": "/run/yetlet-reference/ca.pem",
  "pg_password_file": "/run/yetlet-reference/sql-credential",
  "valkey_url_file": "/run/yetlet-reference/valkey-endpoint",
  "namespace": "yt184_example_e1",
  "port": 39270,
  "origin": "https://auth.yt184.test:8443"
}
```

Secrets are bounded owner-private regular files, not inline JSON/argv/logs.
The fixture admits only its explicit SQL/Valkey reference peer names and
checked native TLS; wrong-name peers are reserved for negative startup proof.
The Valkey endpoint must be authenticated `rediss://` on the fixed peer/port,
without fragments; PostgreSQL uses verified-full TLS. Cross-host Session uses
the provider's normal Secure `__Host-yt184_session`,15minute absolute lifetime,
and unchanged four-operation/payload limits. The native app stays loopback
behind a private TLS edge preserving canonical Host/Origin; forwarded metadata
does not configure trust. Early refusals also carry private no-store/Vary/CSP.
SQL uses four connections and the existing query/acquire limits; only remote
mode enables native pre-acquire health checks for server-restart recovery.

Current account/tenant/permission and Session authority are reread on protected
requests. This synthetic-login reference adds no real IdP or business writes.
Persistent services do not make old Session namespaces safe after lost/rolled
back authority. The selected recovery stops all consumers, preserves current
SQL and switches every consumer to a fresh never-reused namespace, requiring
re-login. The first cloud attempt fails at a bootstrap Path expression. The
authorized second generation reaches both apps READY/wrong-name TLS refusal,
then fails a SQL observer IP-format assertion before the Auth/restart journey.
Both failed generations are fully deleted; the observer is corrected locally.
At that stage complete Auth/restart proof was unexecuted. Authorized
generation3 passes private SQL/HTTPS checks but fails at the initial auth form;
the deployment ACL lacks direct `TIME`, which the provider calls when preparing
Session mutations. Canonical bootstrap adds only `+time`; a new named-user local
regression now reproduces refusal and validates the actual provider lifecycle
with that grant. It uses local native TCP/router oneshot, without SQL/TLS or
Azure restart qualification; all three resource generations remain deleted.
No new benchmark, HA, power-loss/restore or production guarantee.


Generation4 now passes cross-host login/JSON/SSR/tenant/current policy/logout/app
restart, then fails the initial post-Valkey-restart expected200 snapshot. A local
native AOF/provider test records initial503 followed by read-only recovery; the
reference continuation observes bounded per-client readiness rather than assuming
a quiet delay guarantees it. No production retry/limit changes. All four cloud
generations were deleted; authority-outage/VM restart was pending at that
closeout, before the subsequent generation5 result below.


YT-184 is Done at bounded reference scope after final independent Reviewer PASS:
generation4 Auth and generation5 remaining authority recovery/persistent VM
restart with native exit0. Full SQL/restrictive policy, unexpired old-epoch
refusal and fresh-login/durable-output/logout checks pass; transient503s and
all earlier failures remain retained. All five groups/credentials are deleted.
This isolated Linux reference does not qualify HA, arbitrary restore or other
platforms; see the contract for exact composition and accepted reference scope.
