# YT-170 — Opt-in Auth, Session and PostgreSQL functional workload

Date: 2026-10-25 (Asia/Taipei). Status: **Done at bounded local functional-reference scope; independent contract/corrected mapping/final Reviewer PASS, zero issues/required followups; new changes uncommitted/unpushed.**

## Authority and bounded question

The user authorizes the separate six-path YT-169 strict/builtin commit without
Push, then contract-first minimal isolated Auth/Session/PostgreSQL functional
workload validation. Preserve opt-in composition, accepted evidence and failed
results; do not replay passed qualification. Delete all test resources and
obtain independent Reviewer PASS before Done. New changes must not be committed
or pushed.

[YT-125](YT-125-VALKEY-AUTH-INTEGRATION.md) supplies the existing application-owned
Auth/Valkey resolver pattern; [YT-144](YT-144-POSTGRES-WORKLOAD-COMPARISON.md)
supplies the optional SQLx/pinned PostgreSQL and isolated Unix-socket pattern.
[YT-169](YT-169-PRIVATE-HTTP-SUSTAINED-1800.md) is retained medium-SSR evidence,
not a throughput target or controlled baseline for this different workload.
Admin/Starter business stores remain SQLite; this item does not migrate them.

Select one new optional binary in the existing standalone
`examples/axum-workload-comparison` package, guarded by the new
`auth-session-postgres` feature. Compose existing `yetlet-auth`, the existing
Valkey-compatible Session provider (dependency alias `yetlet-session-valkey`,
package `yetlet-session-redis`) and existing optional SQLx. Default features
remain empty; existing binaries, public Core/provider APIs, root workspace,
browser graph and dependencies remain unchanged. Preserve shared lock versions;
add only required feature dependencies. This is an unpublished functional
fixture, not a new Auth/ORM/provider framework or default-installed stack.

Preserve ADRs [0001](adr/0001-separation-of-concerns.md),
[0003](adr/0003-renderers-and-loading.md),
[0005](adr/0005-opt-in-capability-modules.md) and
[0006](adr/0006-handler-first-mvc-optional.md). Keep typed State/Extensions,
native Axum/Tower middleware and ordinary handlers. RequestContext gains no
fixed User/Session/Cookie fields.

## Actual application-owned composition

Use one two-worker application process, one dedicated nonpersistent Valkey and
one dedicated PostgreSQL database, all on the existing host with only loopback
HTTP/Valkey and a private PostgreSQL Unix socket. No Azure resources, external
IdP or production deployment is needed for this minimal new question.

The Session DTO stores only explicit anonymous or server-owned account-one
references. Every authenticated request authoritatively reads Valkey, then
PostgreSQL account enablement/tenant before constructing Principal. A trusted
request-local policy snapshot supplies resource permission separately from
identity. Disabled/unmapped accounts deny; no cookie/header/body permission
claim becomes authoritative. No policy or response cache is installed.

PostgreSQL owns two synthetic accounts and sixteen documents across two
tenants. Each allowed data request performs a parameter-bound account lookup
and a parameter-bound document lookup scoped to the verified tenant. The JSON
route returns the selected document; the SSR route constructs a fresh Yetlet
View/Page with its escaped title/body. Identity, permissions, credentials and
Session DTOs are not serialized into those responses or browser state.

Native login/logout use the existing Session-bound CSRF and rotation/revocation
operations. As in YT-125, login is an explicit **synthetic preverified fixture
identity assertion**, not password/OIDC/token verification. Client input contains
only CSRF, never a chosen account/tenant/permission. Do not claim a real IdP or
credential-verifier qualification. Protected native forms may contain their
explicit Session-bound CSRF token; never retain it in logs, metric metadata or
generic hydration state. The measured document SSR contains no form/token.

The application is a form-authenticated consumer with a JSON data endpoint.
Required anonymous/retired credentials receive explicit local303login navigation,
not an invented bearer challenge. A valid identity without resource permission
is403; another tenant's document is404. Malformed cookies are400; unavailable
Session/account/data authority is redacted503, never anonymous, cached success
or login navigation. Session mutation uncertainty cannot acknowledge success,
emit a success cookie or trigger blind replay. Reads do not renew cookies.
Private responses retain no-store, Vary:Cookie, nosniff and strict CSP.

Account policy is fresh per request, not instantaneous in-flight cancellation.
Business requests are read-only; cross-store write fencing, transactions spanning
Valkey/PostgreSQL and stronger YT-138 ordering remain unselected.

## New verification and once-only workload

One new integrated HTTP journey establishes protected access without credentials,
native CSRF login with ID/token rotation, allowed JSON/escaped SSR, cross-tenant
denial, current PostgreSQL permission removal and restoration, account disable
and restoration, acknowledged logout and rejection of the retired credential.
Checks concern the new SQL-authority/data composition; do not replay the old
provider/Auth/IdP/browser/platform matrices or prior performance points.

Eight independently created signed-in Sessions avoid making all virtual users
contend on one credential. Save cookies only in a task-private0600 credential
file, excluded from evidence and removed on every outcome. Every measured
response uses the complete independent JSON/escaped HTML/header oracle and
forbids Set-Cookie, content encoding and credential exposure. No redirects,
proxy, browser, output cache, rate-limit bypass or hidden retry is installed.

Select exactly two new reference points on the same original app PID: JSON
then SSR, each with25arrivals/s for2s admission followed by100arrivals/s for20s.
Use accepted k6 constant-arrival-rate scheduling, eight preallocated=max VUs,
HTTP1.1 identity encoding, connection reuse,3s request/4s graceful bounds and
full native raw JSON retention. Nominal measured count is4000, plus100warmup
arrivals; keep lifecycle, warmup, measurement, boundary extras and drain separate.
This is functional/latency reference evidence, not saturation, maximum capacity,
a production SLO or a percentage overhead comparison with old workloads.

After measurement, one bounded1.5s pause of the dedicated PostgreSQL container
tests the new authenticated SQL-authority outage: protected requests must fail
closed503 without cookies/data/navigation, while public health remains available.
Unpause in a finally path and verify recovery once. The1.5s target includes
control elapsed time before the unpause dispatch; native Docker command latency
is retained separately and each pause/unpause command has a5s bound. Do not inject another
Valkey outage or replay its accepted provider qualification. Preserve any
unexpected result; no load rerun, resizing, rate/connection tuning or favorable
substitution follows a failed point.

## Fixed providers, resources and preservation

Reuse the YT-144 PostgreSQL18.6 Linux/amd64 digest
`sha256:6b38fe68f2c02cafffbbce7496f76d110d8a947c039bd22c067e9a43c2250240`
and YT-094 Valkey9.1.2 digest
`sha256:418652cfb58ef879d4978c33553735d7147016032d5aefaa14c828e611eb9dfd`.
Inspect cached identities before acquisition; pull only missing pinned inputs
after contract review. Retain acquisition failures. No latest tag, version
fallback, global install or shared service/volume is selected.

PostgreSQL runs network-none with no published port, same UID/GID as the local
task, private ext4 data parent mounted at `/var/lib/postgresql` and socket at
`/var/run/postgresql`; PG18 data is under `18/docker`. Check Unix socket path
length before startup and mounts before use, avoiding anonymous volumes.
Use restricted application SELECT privileges on account/document tables, private
generated credentials, statement logging disabled, normal durability settings,
64MB shared buffers/1MB work_mem/16MB maintenance_work_mem and max_connections12.
SQLx fixes four min/max connections,250ms acquire and750ms query bounds,
no idle/lifetime expiry, no per-acquisition health ping and normal prepared
statements. Prove four distinct preconnected backend PIDs and prepare only the
new parameter-bound account/document queries. Snapshot complete sorted rows,
schema/index/privileges and relevant database counters before/after; temporary
owner policy edits must restore the original rows exactly.

Valkey uses its existing dedicated loopback-only publication, no persistence,
128MiB container memory,64MB noeviction store and fresh namespace. Session TTL
is300s, four admitted calls plus existing waiters and256byte payload bound;
provider queue/time/retry/rotation/CSRF contracts are unchanged. No TLS/ACL/HA,
restore, real secrets custody or multi-instance deployment claim is made.

| Resource | Selected limit |
| --- | --- |
| App | CPUs12–13,2CPU quota,512MiB,128tasks/512FDs,Nice10,no restart;480s unit ceiling,5s stop |
| PostgreSQL | CPUs14–15,2CPU quota,512MiB memory/swap,64pids/1024FDs,64MiB shm,no restart; private data at most2GiB |
| Valkey | CPU10,1CPU quota,128MiB memory/swap,64pids/1024FDs,no restart,task-only tmpfs,no persistent volume |
| Load/control | CPUs16–19,4CPU quota,1GiB,128tasks/512FDs,Nice10,no restart; existing per-second host/cgroup observation, including DB/container process context |
| Files |256MiB per raw output,2GiB total new evidence,at least8GiB free; credentials never enter tracked files or evidence |
| Time |180s acquisition,600s resolution/build/focused checks;600s absolute from first new container startup including cleanup, with last120s reserved for cleanup; no reset/replacement allocation |

Admit actual topology, quotas, UID/mount/socket/FD/pid/resource bounds before
load. Preserve matching-UID container observation learned in YT-144; missing
process/context evidence is disclosed or stops admission, not a reason to build
a new observer framework. Record native commands/exits/times, non-secret
configuration, source/lock/binary/image/k6 identities, CPU/charged memory and
provider/pool/HTTP/drop/error populations. Observe query/pool error counters;
do not infer pure Core cost or causation from aggregate process CPU.

Report exact started/completed/valid/error/drop populations, raw all/valid HTTP
and full-call quantiles, half-open scheduled-window throughput and drain. Any
invalid200 aborts; preserve partial raw results. Native/resource/deadline/unsafe
state failures stop new work and clean up. Ordinary pressure remains recorded;
never describe it as clean. Record one smallest canonical harness correction
per distinct seam if necessary, without replaying completed load; consecutive
repeated infrastructure signatures stop. No validation framework is authorized.

## Cleanup and acceptance

On every outcome unpause owned PostgreSQL, stop/drain/reap app/Session manager/
pool/client/observer work, remove only exact-owned containers/transient units,
verify closed listeners/socket and absent owned processes, delete task-private
data/socket/password/cookie material and preserve redacted evidence. Retain
pre-existing cached images; remove a newly acquired test-only image reference
when unused, without force/prune or disturbing another consumer. No active
test resource may be left behind.

Contract Reviewer PASS precedes implementation/acquisition; concrete mapping
review precedes first startup; final independent results/failure-retention/
cleanup Reviewer PASS precedes Done. Only focused new-binary compilation/lint/
format, new default-graph comparison and new integrated execution are needed;
existing passed qualifications remain retained. Update Backlog/Roadmap/docs and
sample usage with actual evidence. New changes remain unstaged/uncommitted,
with no Push. YT-096/D1–D6 and YT-020 remain unchanged.


## Concrete implementation and pre-start mapping

Independent contract Reviewer `/root/yt170_auth_session_postgres_contract_review_20261025`
returns **PASS, zero issues/required followups** before implementation/acquisition.
The selected optional consumer is [the new binary](../examples/axum-workload-comparison/src/bin/auth_postgres.rs),
[fixed caller](../examples/axum-workload-comparison/auth-postgres-workload.js) and
[SQL fixture](../examples/axum-workload-comparison/auth-postgres-fixture.sql).
Existing sample entrypoints and Core/Auth/provider sources remain unchanged.

Offline resolution retains every original name/source/version triple; the
sample's default normal dependency graph remains byte-identical. The new
binary's independent JSON/SSR oracle test, locked offline release build, strict
Clippy and format check pass. The initial compile fails because the shutdown
future borrowed its local signal; one `async move` correction passes the new
checks inside the original build window, without replaying a passed test.
Original source/diagnostic/native exit are retained.

Only the four new caller modes are inspected without HTTP, using explicitly
synthetic placeholder cookies. Pinned image acquisition/version metadata is
retained; no service or live load starts during those checks. Concrete paths,
ports, namespace, units, resource budget and task-only execution adaptation
are recorded in `target/yt170-auth-session-postgres-20261025/evidence`.
Corrected mapping review passes before the only startup; actual results and
cleanup are complete below, with final independent review accepted at closeout.


### Initial mapping review and bounded corrections

The first independent mapping review returns **FAIL with two P2 findings**
before any service startup: controller affinity16–19 could not pass a host
10–19 assertion, and hard-failure fallback omitted newly acquired image cleanup.
Both are execution-glue failures, not product defects. Preserve the original
verdict, sources and manifest. One canonical correction per distinct seam
checks controller/host placements separately and extends only the existing
fallback to remove the exact non-preexisting unused image reference without
force/prune or disturbing another consumer. Static syntax review is sufficient;
no passed compilation, product test, lint, inspection or workload is replayed.
The same task, plan and original budgets remain. Independent re-review by
`/root/yt170_auth_session_postgres_mapping_review_20261025` returns **PASS,
zero issues/required followups** before the first startup.


## Actual new integration and workload evidence

One dedicated PostgreSQL18.6 container and one Valkey9.1.2 container pass the
selected image, UID/mount/socket, quota and matching-UID process admission.
One app PID3000473 preconnects four distinct PostgreSQL backends and uses both
new parameter-bound queries. All eight signed-in Sessions are distinct; private
credential files never enter retained evidence. No existing service or accepted
qualification is restarted or replayed.

The newly composed native HTTP journey passes required anonymous navigation,
Session-bound CSRF login, ID/token rotation, rejection of the retired anonymous
ID, exact JSON and escaped SSR, and404for another tenant's document. PostgreSQL
permission removal keeps identity authenticated but makes both JSON/SSR403;
account disable denies authentication. The owner restores both policy fields.
Eight acknowledged CSRF-protected logouts retire their credentials; a subsequent
protected request with the first retired cookie receives303without a cookie.
This is synthetic fixture login, not credential/IdP verification.

### Fixed new reference points

| Workload | Offered /s | Seconds | Started / completed / valid200 | Drops / HTTP errors | Valid in-window /s | HTTP p50 ms | HTTP p95 ms | HTTP p99 ms | HTTP max ms |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| JSON tenant document |100 |20 |2000 /2000 /2000 |0 /0 |100.000 |1.747749 |2.115684 |2.643780 |3.458653 |
| Private Yetlet SSR |100 |20 |2001 /2001 /2001 |0 /0 |100.000 |1.764104 |2.111892 |2.634366 |5.421291 |

Both complete independent body/header oracles pass on every measured response:
no-store, Vary:Cookie, strict CSP, nosniff, correct UTF-8 length/type, no Set-Cookie,
no encoding/server header and HTTP1.1. Native errors, interrupted requests and
invalid200responses are zero. All/valid completed latency populations coincide
because all measured responses are valid; native interpolated summaries and raw
nearest-rank projections are retained separately. Full-call timing is1ms
quantized: both p50/p95/p99 are2/3/3ms, maxima5msJSON/6msSSR.

JSON has2000starts/completions inside its half-open20s window, no late start or
tail. SSR has2000in-window starts/completions plus one boundary start and one
post-window completion, with3ms quantized drain. Its2001full completions must
not be divided by20and called in-window throughput. Admission is separate:
JSON50valid responses, SSR51with one boundary/tail response; both have zero
drops/errors and25valid in-window responses/s. Measured total4001and warmup
101are separate from lifecycle/fault requests. No favorable replay or additional
load point occurred.

### SQL-authority outage and reconciliation

The one intended PostgreSQL pause targets1.5s; actual pause/control interval is
1.546385s including native command latency. The authenticated data request
returns redacted503in0.753915s with no data, success cookie or navigation;
public health remains200without Session/SQL work. Unpause runs in the finally
path; one subsequent full-oracle read succeeds. This expected fault population
is separate from the zero-error measured groups. No Valkey outage, real IdP,
HA, restore or multi-instance test is replayed or inferred.

All account/document rows, schema, constraints and indexes match before/after;
only the explicit owner policy edits occur and are restored. The app role has
SELECT-only table grants/read-only default transactions; HTTP business handlers
perform no writes. Pool closure and Session drain are acknowledged on shutdown.

Counters reconcile exactly to the new journey:4135account-query attempts,
4106document-query attempts, two business-permission denials, three retired/
missing-Session resolver outcomes and one database error for the intentional
authority outage. The database-error counter aggregates acquire/query/provider
failures; separate pool-wait/query timing or failure-phase attribution was not
instrumented. The source has no silent cache, anonymous fallback or mutation
retry; the successful measured requests require both SQL lookups. Database
statistics and relation snapshots remain context, not per-query causal traces.

### Resource context and finite observation limits

| Measured envelope | App CPU % | PostgreSQL CPU % | Valkey CPU % | Client/control CPU % |
| --- | --- | --- | --- | --- |
| JSON |5.79 |7.33 |4.62 |32.06 |
| SSR |5.77 |7.28 |4.65 |32.43 |

Percentages use one CPU, derived from cgroup usage deltas over same-host
before/after envelopes of20.183/20.186s. They include observer/control/native
finalization activity; process/container components are not perfectly aligned
causal traces. Selected quotas are2/2/1/4CPU respectively. All measured cgroup
CPU-throttle and memory-max/OOM event deltas are zero.

Cumulative charged-memory peaks are3.59/3.75MiB for the app after JSON/SSR,
82.55MiB PostgreSQL,10.59MiB Valkey and77.61MiB client/control. These peaks
span earlier setup/phases and are not per-request allocation, standalone RSS or
heap measurements. Maximum sampled app/client RSS is7128/45612KiB, reported
separately. Forty-nine finite observations have no missing process records and
a largest1.177260s gap. Aggregate sampling wall time is6.750816s; observed
controller CPU grows1.33s, both distinct measures. Stable container-main-PID
context plus whole-container cgroups do not measure every backend FD/RSS peak
or resolve a request's SQL/pool wait.

The sequential JSON-first/SSR-second small warm dataset is not a balanced
router/render comparison, capacity maximum, long soak or production SLO. Do not
infer negligible Auth/SSR overhead or scale headroom linearly from these low
reference points, nor compare percentages against YT-144/YT-169's different
workload/topology. Business writes/fencing and real-provider login remain open
questions requiring separate selection.

### Retention, cleanup and acceptance gate

All four full native raw files total22051938bytes; summaries, projections,
command exits, matching-UID observations, complete logical snapshots, source/
lock/binary/image/k6 identities and failures remain retained. New evidence is
approximately22.65MB against2GiB, observed PGDATA48.43MB against2GiB and no
per-output256MiB limit is reached. The initial Rust compile failure and initial
mapping Reviewer FAIL remain alongside their smallest corrections and later
PASS. Older accepted and failed performance/provider results are unchanged.

Both owned containers and their original main PIDs are absent. The app, all
four native k6 processes, controller and transient units terminate; listener
ports close. Task-private data/socket/password/cookie material is removed.
The newly acquired PostgreSQL image reference is removed without force/prune;
the pre-existing Valkey cache is preserved. Runtime including canonical cleanup
is53.009881s against the original600s, native launch exits0after54.530249s
and the outer launcher finishes54.604167s. No active test resources remain.

Independent final results/privacy/failure-retention/cleanup Reviewer
`/root/yt170_auth_session_postgres_results_final_review_20261025` returns **PASS, zero issues/required
followups**. YT-170 is Done at this bounded local functional-reference scope.
New source/documentation checks apply only to this delivery; no old successful
runtime qualification is replayed.
All ten new paths remain unstaged and uncommitted, with no Push. No further
workload or item is activated; YT-096/D1–D6 and YT-020 stay unchanged.
