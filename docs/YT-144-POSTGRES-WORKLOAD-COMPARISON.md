# YT-144 — Optional PostgreSQL workload comparison

Date: 2026-10-23. Status: **Done at the bounded comparison scope. Corrected independent contract review
PASS; optional implementation/build/focused PostgreSQL checks PASS. Separately
human-authorized measurement continuation completed all eight groups with
100,007 exact measured valid responses and four comparable pairs. Original
failed review retained. Independent final Reviewer
`/root/yt144_postgres_final_review_20261023` PASS, zero issues and required
followups.**

## Authority, question and limits

The user requested PostgreSQL as the separate next step after accepted
[YT-143](YT-143-AXUM-WORKLOAD-COMPARISON.md): contract first, minimum isolated
new verification, opt-in integration, retained failures and independent final
Reviewer PASS before Done. Baseline HEAD is
`2fbf1b4a0a7adfe8515dc61dd27037ff2c823c8e`, clean before these four documentation
edits. YT-143 and all earlier accepted proofs remain source-bound historical
evidence; do not replay them. No new commit, push, deployment or cloud work is
authorized here.

The new question is how native Axum route registration and Yetlet's `App::get`
behave when the same handler acquires from the same asynchronous PostgreSQL
pool, executes the same parameterized single-row SELECT and serializes the same
fresh JSON object. This measures that complete local path at two fixed offered
rates. It does not isolate router overhead, compare PostgreSQL with SQLite,
establish PostgreSQL capacity, qualify a production deployment or promise a
resolved Yetlet overhead. Multi-query, writes, Fortunes, pgbench, soaks and
framework optimization are outside this slice.

Preserve [ADR 0001](adr/0001-separation-of-concerns.md),
[ADR 0006](adr/0006-handler-first-mvc-optional.md) and the
[M9 SQL-first policy](M9-PLAN.md#3-sql-first-data-access-policy-no-yetlet-orm).
The database is a native application-owned optional capability. No Yetlet ORM,
Core provider, query DSL, repository/DbContext, migration framework, DI layer or
browser dependency follows. The public synthetic read-only fixture needs no
Settings, Auth, Session or rate-limit provider and must not inherit the
Starter's four-slot immediate SQL rejection policy. No protected application
is changed or bypassed.

This single-row fixture is inspired by the
[archived TechEmpower database description](https://github.com/TechEmpower/FrameworkBenchmarks/wiki/Project-Information-Framework-Tests-Overview).
Deterministic ID selection, Unix sockets, k6 and this local resource envelope
are declared differences; no official, certified or complete suite result is
claimed.

## Read-only capability findings and selected source surface

Inspection found Docker available and responding, zero existing containers,
and no cached PostgreSQL image. `psql`, `initdb`, `postgres` and `pg_ctl` are
absent from PATH; no versioned binaries were found under `/usr/lib/postgresql`,
`/usr/local/pgsql` or `/opt`. Only Valkey and hello-world images were cached.
Nothing was pulled, started or reconfigured. Raw discovery receipts are under
`target/yt144-postgres-workload-c9a61dwf/evidence` on verified ext4; `/tmp/yetlet-yt144-current-evidence`
is a pointer file, never the raw-output destination.

Cached SQLx wrappers include 0.8.6 and 0.9.0; 0.8.6 Core/SQLite support is
cached, but its PostgreSQL driver is not. No cached `tokio-postgres` or
`deadpool-postgres` was found and no repository Cargo consumer uses PostgreSQL.
The [M9e comparison](M9E-DATA-ACCESS-COMPARISON.md) already evaluated SQLx 0.8.6
as a direct asynchronous pool/query option. Select exact `sqlx = "=0.8.6"`,
`default-features = false`, only `runtime-tokio` and `postgres`; use runtime
`query_as` into an explicit tuple and map into the response struct. No query
macros, offline schema cache, derive macro, TLS stack or SQLx migration feature
is needed. A deadpool/tokio-postgres pair would add a second independently
resolved integration without an existing consumer; SQLx supplies this one
pool/query boundary directly. This is an application selection, not a general
library ranking or a new dependency comparison benchmark.

After independent contract PASS, extend only the existing isolated
`examples/axum-workload-comparison` package with optional feature `postgres`,
an explicit `[[bin]]` for `src/bin/postgres.rs` with `required-features =
["postgres"]`, sample-owned `postgres-fixture.sql`, `postgres-workload.js`,
README instructions and its standalone Cargo.lock. Preserve `src/main.rs` and
`workload.js` bytes. Keep its empty `[workspace]`, root membership/manifests/lock,
Core/HTML/adapter and other applications unchanged. Separate binary and script
avoid touching the accepted three-workload fixture; a new standalone package
would duplicate its pinned HTTP dependencies and manifest.

Default features remain empty. PostgreSQL dependencies must be absent from the
resolved default graph as well as Core/browser graphs; lockfile entries for an
optional feature are not active dependencies. Preserve all existing shared
lock versions (Axum 0.8.9, Tokio 1.53.2, Serde 1.0.229, Serde JSON 1.0.151,
Tower 0.5.3, Hyper 1.12.0 and Hyper-util 0.1.21). Add only required PostgreSQL
transitives; do not solve resolution by upgrading shared dependencies. Freeze
new resolved versions/checksums/features before build; unavailable resolution
stops for a concrete amendment.

Both modes share one compiled executable, handler, pool setup, SQL, response
mapping, runtime and HTTP serving path. Mode selection occurs only at startup:
native `Router::route(..., get(handler))` versus
`App::new().get(..., handler).into_router()`, with the same explicit state.
Use Rust/Cargo 1.99.0, native host, two Tokio workers, default release profile,
no RUSTFLAGS/profile overrides/CPU tuning/PGO/alternate allocator. Build once
with `cargo build --release --locked --manifest-path examples/axum-workload-comparison/Cargo.toml --features postgres --bin postgres`
and task-private ext4 CARGO_TARGET_DIR. Record exact source/lock/compiler/
feature/binary hashes and argv/environment/times/exits; this shared executable
cannot establish binary-size or dependency-footprint differences between modes.

## Database identity, fixture, protocol and pool

Select official `postgres:18.6-bookworm`, Linux/amd64, pinned for acquisition as
`docker.io/library/postgres@sha256:6b38fe68f2c02cafffbbce7496f76d110d8a947c039bd22c067e9a43c2250240`.
Read-only official-images and Docker registry metadata agree on that platform
manifest and config/image ID
`sha256:63d4ad6f3de5e288238b3044970a677dcb0e77004d8b6050732aa0cbbeedf5d7`;
compressed layer sizes total 157,268,716 bytes. The
[pinned official Dockerfile](https://raw.githubusercontent.com/docker-library/postgres/e00e1bd34ec5c8a8e7ad89b273b3d42efaf6d5bc/18/bookworm/Dockerfile)
and [official image instructions](https://github.com/docker-library/docs/blob/master/postgres/README.md)
identify PostgreSQL 18.6, Debian Bookworm and arbitrary-user NSS-wrapper support.
Metadata provenance is retained as `official-postgres-image-identity.json`.
Pull only this digest after contract acceptance; retain actual acquisition
argv/time/exit/stdout/stderr, pulled manifest/config identity and
`SELECT version()` / `SHOW server_version_num`. No image was pulled or server
started during contract drafting; metadata selection is not runtime evidence.

Run Docker with `--network none`, no published ports, restart disabled, a unique
owned name/label and no existing volumes. Bind the task-private ext4 data parent
to `/var/lib/postgresql`, with PG18 PGDATA `/var/lib/postgresql/18/docker`, so
the image's declared parent volume does not create an anonymous volume. Share
only that parent and a task-private Unix-socket directory. The socket directory may permit socket access
inside its mode-0700 task parent; do not expose it outside the task. Use the
current host UID/GID 1000:1000 with the official Debian entrypoint's arbitrary
user support; precreate owned writable mount directories and assert actual
server UID/GID. Never chown outside the private fixture. Select socket host path
`target/yt144-postgres-workload-c9a61dwf/socket`, mount it
at `/var/run/postgresql`, set
`unix_socket_directories=/var/run/postgresql`, port 5432 and
`listen_addresses=''`. Verify the host path plus `/.s.PGSQL.5432` fits Linux's
108-byte Unix-socket address bound before startup. Inspect actual
container mounts and host `findmnt -T` for data, socket, build and every output
path before startup; checking the build mount alone is insufficient. No host
network, existing data, service or global PostgreSQL setting is touched. No
TLS is selected for this private same-host Unix socket; record that limit and
actual PostgreSQL extended-query protocol/prepared statement cache behavior.
The initial independent contract review returned FAIL with one P2 before any
startup: the pinned official entrypoint clears PGHOST/PGHOSTADDR for bootstrap
psql, so a custom internal `/socket` directory would prevent initialization.
The corrected internal directory above matches that bootstrap client; the
private host path, UID/GID and disabled network are unchanged. The original
verdict is retained in `contract-review-failed.json`; no custom bootstrap or
runtime proof was needed for this compatibility correction. Independent
re-review returned PASS with zero issues and required followups before stage 2.
The corrected frozen source is retained as `contract-reviewed-document.md` and
the verdict as `contract-review-final.json`.

No PgBouncer or other proxy is installed. SQLx pool/query and timeout semantics
follow the pinned [SQLx 0.8.6 pool options](https://docs.rs/sqlx/0.8.6/sqlx/postgres/struct.PgPoolOptions.html)
and [prepared-query API](https://docs.rs/sqlx/0.8.6/sqlx/fn.query.html);
PostgreSQL settings and observations follow its
[connection settings](https://www.postgresql.org/docs/18/runtime-config-connection.html),
[statement timeout](https://www.postgresql.org/docs/18/runtime-config-client.html),
[EXPLAIN](https://www.postgresql.org/docs/18/using-explain.html) and
[statistics](https://www.postgresql.org/docs/18/monitoring-stats.html) documentation.

Initialize one database, a bootstrap owner and a restricted app role with only
CONNECT, schema USAGE and SELECT. Generate synthetic disposable passwords into
mode-0600 local files; do not put values into commands, receipts, SQL logs,
URLs, source or stdout. Supply them through private files/process environment
without retaining secret values. Startup and errors must redact credentials,
connection strings, paths and SQL details. These credentials have no production
identity. No browser state is produced.

The sample-owned SQL creates `public.world(id integer PRIMARY KEY, random_number
integer NOT NULL CHECK (random_number BETWEEN 1 AND 10000))`, seeds exactly IDs
1 through 10000 with `random_number = 1 + ((id * 7919) % 10000)`, and runs
ANALYZE once. No sequence, random seed, migration engine or measured write is
needed. Record complete sorted logical rows/hash, row count, schema/constraints
and index definition before/after the experiment; enforce default read-only
transactions for the app role and verify its lack of write privileges. A
bytewise PGDATA comparison is inappropriate because statistics and control
files change during reads.

Every measured request runs exactly:

```sql
SELECT id, random_number FROM public.world WHERE id = $1
```

Bind the integer parameter; map two i32 values to a fresh
`World { id, randomNumber }` response. No response/row cache, server-side random
number query, per-request connection opening or alternate query between modes.
Use SQLx's normal per-connection persistent prepared statements with explicit
statement-cache capacity 100; the measured query is identical on all physical
connections. Preconnect all 16 pool connections before listening, holding all
16 acquisitions concurrently, verify 16 distinct backend PIDs and the matching
16 app-role sessions in pg_stat_activity, then run the exact query once on each
held connection and return them. A min/max setting alone is not proof of actual
preconnection. `min_connections = max_connections = 16`,
acquire timeout 250 ms, no lifetime/idle expiry during the finite run,
`test_before_acquire(false)` explicitly disables SQLx's otherwise default
per-acquisition health ping after successful startup. Record that setting and
actual per-connection preparation/preconnect evidence; no hidden per-request
connect or health-check SELECT is selected. Connection loss is a failure, not a reason to tune/replay. This small explicit budget permits waiting
for a connection rather than immediate admission rejection. PostgreSQL
`max_connections = 24` reserves room for bootstrap/observation without a second
app pool. `shared_buffers = 64MB`, `work_mem = 1MB`, `maintenance_work_mem =
16MB`, normal fsync/full_page_writes/synchronous_commit, no parallel query
workers, no JIT, and no statement logging. Record actual settings, role
read-only/statement timeout and connection counts, including non-app sessions.

Apply a 750 ms query deadline after acquisition, PostgreSQL role
`statement_timeout = 750ms`, and a 1,250 ms overall handler deadline. Pool wait
expiry maps to HTTP 503 with literal `database unavailable`; query deadline or
PostgreSQL query-cancel timeout maps to HTTP 504 with literal `database timeout`;
other provider/row-mapping failures map to generic HTTP 503. Overall expiry
maps to the same redacted 504. Preserve bounded category counts (acquire,
query-timeout, overall-timeout, provider), without per-request logs. No retry.
Return/close cancelled connections through the driver's supported lifecycle;
no fire-and-forget query tasks. Graceful shutdown closes/drains the pool.

Before load retain `EXPLAIN (ANALYZE, BUFFERS, FORMAT JSON)` of the exact bound
query for boundary/interior IDs, pg_class tuple/page estimates, actual relation/
index sizes, pg_stat_database / table/index counters and cache-hit context.
Record planner choice rather than force an index. Read all 10,000 rows in sorted
order once before the first phase; separate each phase's exact-query warmup.
This is intentionally a warm small dataset, with OS cache uncontrolled and
shared-host interference disclosed. Do not flush global caches, change plans
or settings after observing load, or call this a storage-I/O/cold-cache test.

## Exact HTTP behavior and fixed new experiment

The only measured route is GET `/db/{id}`. Parse and bound IDs to 1–10000 before
SQL. Malformed/out-of-range IDs receive 400; missing/wrong-method paths retain
native 404/405. Successful responses are HTTP 200, `application/json`, exact
compact UTF-8 `{"id":N,"randomNumber":R}` with no newline and the formula above;
for ID 1 the literal body is `{"id":1,"randomNumber":7920}`. Match Content-Length
and all non-Date headers across modes. No Content-Encoding, Set-Cookie, Server,
redirect, authorization, cookie, Origin or request body; no CSP is needed for
this JSON-only endpoint. Record full preload headers/body lengths and HTTP/1.1.
No browser/CSP-enforcement or WASM verification is claimed.

Use plain loopback HTTP/1.1 TCP and keep-alive, identical `Accept` and
`Accept-Encoding: identity`, no HTTP/2, TLS, pipelining, proxy, retry or redirect.
Each k6 iteration makes one GET; select `id = 1 + (scenario.iterationInTest %
10000)` and check the exact response formula independently of the server. Keep
per-ID counts. Use the retained official k6 2.3.0 provenance and checksums:
archive SHA-256 `39c3117b6af817592dcd0ce4242105c0a7af10948c2a425306f0be8f7a8a8ab1`,
executable `e001bbf39ba67424a85c9c8516440ba5c73ecf7a6021be96c76194380c15f260`.
The disposable YT-143 binary was removed; reacquire only that exact release
with raw acquisition/version/hash receipts, no global install/cloud output and
usage reporting disabled. Use k6's native constant-arrival-rate executor, not
a new scheduler, benchmark runner or validator product.

Run two repetitions, visiting **500/s then 2,000/s**, ten scheduled seconds per
mode/point. Adjacent paired modes are Axum then Yetlet in repetition 1, Yetlet
then Axum in repetition 2. This is eight measured phases and
`2 repetitions × 2 modes × (500 + 2000)/s × 10 s = 100,000` nominal measured
arrivals. Each phase starts one fresh app/pool on the same owned loopback port;
only one app is alive. Probe IDs 1, 2, 9999 and 10000 plus invalid bounds before
load. Run separate two-second 250/s warmup, wait for drain and one second idle,
then measured load; nominal warmup total is 4,000. Keep correctness, warmup and
measurement populations separate. The same DB remains alive and logically
unchanged throughout; no reset, reseed or database restart between modes.

Both warmup and measurement use preallocated=max 64 VUs, `timeUnit=1s`, two-second
HTTP timeout and three-second graceful stop, no sleeps. App native stop is at
most five seconds with connection drain/closure observed before its pair.
Freeze all eight phase commands and inputs before startup; no added rate,
favorable replay, duration change, connection/resource tuning or accepted old
proof replay follows from a failed point.

| Owned resource | Fixed bound |
| --- | --- |
| App | CPUs 12–13, two Tokio workers, CPUQuota 200%, MemoryMax 256 MiB, TasksMax 128, LimitNOFILE 512, Nice 10, no restart |
| PostgreSQL container including backends | CPUs 14–15, CPU quota 200%, memory and memory+swap both 512 MiB, pids limit 64, nofile 1024, shared-memory cap 64 MiB, no restart |
| k6 plus monitor/analysis | CPUs 16–19, CPUQuota 400%, MemoryMax 1 GiB, TasksMax 128, LimitNOFILE 512, Nice 10, no restart; sequential raw parsing |
| Actual ext4 files | 256 MiB per output file, 2 GiB aggregate evidence/output, 2 GiB PGDATA aggregate, at least 8 GiB free before startup |
| Finite time | Acquisition at most 180 s, new resolution/build/checks 600 s, DB initialization/correctness/warmup/load/drain/cleanup 600 s from the actual first startup; total at most 1,380 s excluding human review |

Confirm affinity/topology and actual cgroup limits before execution; CPUs
12–19 were distinct cores in retained YT-143 topology, but the new mapping must
be observed. Missing placements/limits stop instead of selecting better ones
after results. Check `memory.max`, memory+swap, CPU quota and FD/pid bounds on
the actual DB/app/client cgroups. The output estimate uses twice the retained
roughly 4 KiB per attempt: 104,000 × 8 KiB is about 813 MiB, below 2 GiB;
20,000 measured arrivals yield about 156 MiB below the 256 MiB file cap. Actual
native boundary extras and other receipts still count toward caps.

Set an absolute deadline at first owned DB startup and preserve it through any
interruption/correction; cleanup stays inside the runtime budget. Acquisition
and build have separate original-start clocks, not resettable per command.
Bound commands and keep an outer watchdog; owned lifecycle glue is disposable
run-local evidence, not a new repository runner/harness framework.

## Populations, attribution, failure retention and acceptance

Retain raw per-attempt native k6 JSON, summaries, stdout/stderr, exact argv,
non-secret environment, UTC/epoch/monotonic intervals and exits, including
failures. Save invocation exit receipts before parsing. Bind reviewed contract,
new sources/fixture/lock/script/tool/image/binary and existing protected source
hashes before startup and recheck afterward. Never reconstruct stdout or infer
an unavailable exit as zero.

Report planned, started, completed, exact valid-200, invalid-200, HTTP-status,
transport/timeout, dropped and interrupted counts separately; distinguish
native endpoint boundary extras and half-open scheduled start/completion
windows. Reconcile nominal minus actual starts minus drops without inventing
missing requests. Give both scheduled-window valid throughput and throughput
through final completion, with explicit denominators; offered load is not
achieved throughput. Read-only logical rows/schema/indexes must reconcile with
zero acknowledged mutations/unknown writes.

Retain nearest-rank p50/p95/p99 for floating-millisecond native
`http_req_duration`/`response.timings.duration` (send + wait + receive, excluding
blocked/DNS/connect), separately for all completed attempts, HTTP responses
and exact valid 200s. Also retain full-call `Date.now()` elapsed milliseconds
for all completions/valid 200s, with integer resolution/wall-clock limits.
Dropped/interrupted requests have no completed latency sample; failed transport
zero/incomplete HTTP timings are explicit. No submillisecond claim from integer
full-call time or false precision from native decimals.

Sample actual app PID, all DB container processes/backends, post-exec k6 PID
and monitor CPU/RSS/threads/FDs roughly once per second. Retain cgroup CPU/
throttling, memory.current/peak/max, memory.stat anon/file/slab, memory.events
including max/oom/oom_kill/socket pressure, pids events/current/max and output/
data size/space. DB process RSS sums may double-count shared buffers; report
DB cgroup memory separately. One CPU core is 100%. Publish actual inner measured
sample windows/gaps rather than dilute CPU with startup/warmup. Snapshot DB
connections/counters outside measured windows to avoid concurrent observer
queries; plans/DB wait/resource evidence and redacted handler category counters
help distinguish pool queueing, provider/DB time, HTTP/client failure and shared
host/resource pressure. No new telemetry provider/metrics endpoint is needed.

Stop on correctness mismatch, source drift, absent required evidence, deadline,
file/aggregate/disk/pid/FD cap, unexpected process exit, memory.events max/OOM/
kill/socket-limit event or unsafe state. Retain partial evidence and finish owned
cleanup. CPU throttling/saturation and ordinary HTTP failures/drops remain
visible failed comparison points; continue the fixed remaining plan only when
no safety/correctness/evidence stop applies. Do not attribute queue/503/504 or
latency to Yetlet alone when PostgreSQL/client/resources are constrained.
Apply the existing materiality/repair limits before any correction; do not
create a validator/certification framework, new run or favorable replay.

A comparable phase needs correct output, zero HTTP/transport/timeout/error/
drop/interruption, at least 95% of nominal arrivals completed validly inside
the ten-second window, complete raw/resource evidence and no resource/output
cap. Client headroom requires zero client CPU throttling/OOM, no FD/pid cap,
sampled CPU below 80% of 400% and peak cgroup memory below 80% of 1 GiB.
Missing evidence means unverified. Record DB/app saturation and throttle data;
no consumer SLO or requirement that Yetlet beat Axum exists.

Show all eight phases and failures. Calculate Yetlet/Axum ratios only within a
matching comparable rate/repetition and timing definition. Latency above 1
means slower; valid-throughput above 1 means more completed work at that fixed
offered load, not a capacity speedup. Do not pool reversals, divide by zero or
claim statistical significance from two repetitions. Small/noisy/inconclusive
results and unresolved failure attribution are valid bounded outcomes. No
maximum/sustained throughput, SQLite speed comparison, remote-network, TLS,
cold-cache, production-security or universal framework-performance claim follows.

## Minimum new verification, cleanup and review gates

Contract review must PASS before dependency acquisition, implementation or DB
startup. Then run only the new PG binary's focused checks: both route modes'
exact boundary/interior JSON/headers from seeded rows, bounded/malformed IDs
and native 404/405; verify parameter binding, app read-only permissions, full
16-connection preconnect and no response cache. Focused finite pool-exhaustion
and query-timeout checks exercise redacted 503/504 categories outside load;
no fault-injection HTTP endpoint or alternate measured SQL is added. Verify
provider-error redaction and bounded native close without changing old tests.

Check sample formatting, the new locked release build and new binary behavior.
Use focused dependency-tree/lock comparison to show the default sample graph
has no PG activation, only the optional binary selects SQLx, shared versions
stay fixed and root/Core/browser sources/manifests/lock are unchanged. No old
three-workload test/build/load, full workspace suite or browser matrix replay
is selected. Run `cargo xtask check docs`, tracked `git diff --check` and
`git diff --no-index --check /dev/null docs/YT-144-POSTGRES-WORKLOAD-COMPARISON.md`;
for the untracked file accept only exit 0/1 with empty diagnostic streams.
Preserve real command stdout/stderr, argv, start/end and exit receipts.

After proof, close the app pool, natively stop/reap app/client/monitor/watchdog,
stop PostgreSQL within ten seconds, capture exit/cgroup/OOM/mount identity and
logical reconciliation, and remove only the owned container and task-private
data/socket/credential/tool/build files. Verify owned PIDs, listeners, Unix
socket, container and transient units are absent; retain all raw evidence and
failed points on ext4. Track newly pulled image identity; remove its owned
reference only if newly acquired, unused and safe without touching prior
images. Missing cleanup proof prevents Done.

Independent final Reviewer PASS over changed source, exact frozen new workload,
retained failures, claims and cleanup is required before YT-144 becomes Done.
The original checkpoint ended before implementation/startup; actual subsequent
work, the terminal observation blocker and independent final review are recorded
below. The original failed review remains retained. The separately human-authorized
measurement continuation below addresses its missing workload evidence.
YT-096 owner decisions and YT-020 release gates retain their existing status.


## Original implementation, checks and retained terminal blocker

The corrected independent contract Reviewer PASS had zero issues and required
followups. Stage 2 added only the optional SQLx 0.8.6 feature and new `postgres`
binary, SQL fixture and k6 script within the existing standalone sample, plus
its lock/README and these four task documents. Internal Rust names use
`random_number`; Serde explicitly preserves external JSON `randomNumber`.
Original `src/main.rs` and `workload.js`, root manifests/lock and Core/HTML/
adapter sources remain byte-identical. Default dependency-tree inspection has
no SQLx/PostgreSQL activation; every pre-existing standalone lock version is
preserved. No previous test, build or workload was replayed.

Pinned image/dependency/k6 acquisitions succeeded. The official archive and
executable matched their declared hashes; the actual image uses the pinned
Linux/amd64 manifest. Docker's containerd store reports that manifest digest
as image-inspect `Id`, while registry metadata separately identifies the config
digest. An initial setup assertion expecting the config digest in `Id` failed;
one smallest identity-check correction accepted the matching Descriptor,
RepoDigests/platform and retained both identities. This was a harness failure,
not a product result; no image was repulled or product input changed.

The new locked release build and compilation of the new binary's focused test
passed with Rust/Cargo 1.99.0. One focused test then passed against actual
PostgreSQL **18.6 / server_version_num 180006**, using the private Unix socket,
UID/GID 1000:1000 and the declared network/resource configuration. It checks
20 in-process HTTP route/status/body/header responses across both registration
modes; actual 16 distinct backend PIDs and 16 app sessions; read-only role/grants;
finite exhausted-pool acquisition and locked-table query timeouts; generic
503/504 failure bodies; and pool close. The lock was rolled back and a later
read succeeded. No fault-injection route was added, and the passing test was
not replayed after the observation failure. These are compile, database and
in-process HTTP checks; no live HTTP workload, browser or CSP-enforcement proof
is claimed.

The database contained the exact 10,000 deterministic rows. Retained schema
and plans show the public.world primary-key btree and index scans for four
boundary/interior IDs. Heap/index relation sizes were 368,640 / 245,760 bytes,
well within the 64 MiB shared-buffer allocation. Plans reported shared hits,
not cold storage reads. Actual settings retained fsync/full_page_writes/
synchronous_commit on, no TCP listen address, SSL off, 24 max connections,
64 MiB shared buffers and the app's read-only/750 ms statement-timeout role
configuration. These setup observations are not workload performance results.

The one runtime began at epoch **1792761097.506**, with original absolute
runtime deadline **1792761697.506**. After the focused test and before any app
phase, the initial monitor failed with `PermissionError` reading an owned
PostgreSQL backend's host `/proc/<pid>/fd`. It stopped and natively stopped/
removed the owned database container; no application or k6 process was started.
This operational permission failure disproved no product behavior.

One bounded parent-authorized operational continuation used the same retained
PGDATA and original deadline, with no reseed, test replay, product/tool/workload
change or new run. The original monitor/result/preexecution were preserved;
the declared observer delta attempted a read-only Docker-local user-0 proc scan
while retaining host CPU/RSS/status and cgroup observation. All 10,000 sorted
logical rows matched the pre-test snapshot after the restart. This restart
happened before any paired mode; cache history includes initialization, plans,
focused reads, restart and the logical scan, with OS cache uncontrolled.
The fallback returned no required database process records and stopped on
`in-container DB process evidence missing`. That is a terminal observation
blocker, not evidence that all possible local observation methods are
unavailable. No additional correction, startup or workload attempt followed.

**Measured phases: 0/8. Warmup phases: 0/8. Live app phases: 0/8.** There are no
new latency, throughput, resource-headroom or comparable-pair results. The
nominal 100,000-arrival plan remains unexecuted. Full post-workload logical/
schema reconciliation and app/native HTTP stop proof remain unrun because
those phases never started; the completed post-focused-test logical equality
must not be promoted into workload acceptance.

Raw acquisition/build/test/setup/failure stdout/stderr, command receipts,
source identities, plans, logical rows and the correction history remain in
`target/yt144-postgres-workload-c9a61dwf/evidence`. Both operational failures
and the original independent contract FAIL remain retained. Some same-named
setup/stop command receipts were replaced during the continuation; original
failure result/script/preexecution and focused-check receipts remain preserved.
No missing original receipt is reconstructed or inferred successful.

Owned cleanup completed at epoch **1792761264.884**, inside the original
runtime deadline. Both database stops exited 0; the retained latest container
state is exited with code 0 and no OOM. Owned monitor/database PIDs, listener,
socket, container and transient units are absent. Private data/socket/password,
tool and build directories were removed; the newly acquired pinned image
reference was removed, leaving the original Valkey/hello-world images.
`cleanup-proof.json` and raw cleanup commands retain the actual observations.
No existing service/data, host policy, privilege setting, commit or push was
changed. Final independent Reviewer `/root/yt144_postgres_final_review_20261023`
returned **FAIL with one P2**: the original requested workload comparison has no
actual load evidence. The review supports the partial implementation, focused
checks and cleanup, with no material product defect established. Its full
verdict and reviewed source bindings are retained in `final-review-result.json`
and `final-reviewed-files.json` in the evidence directory. YT-144 remains
**In progress** at that closeout; no performance or Done claim was made and no
further workload retry was selected without separate human authorization.

## Human-authorized measurement continuation — 2026-10-23

The user subsequently authorized the smallest owned-container observation fix
and only the eight never-started comparison groups, with the same source, SQL,
fixture, tool/image identities, rates, order, limits and acceptance criteria.
This continues YT-144 in the same evidence directory; it is not a replacement
experiment. The original expired deadline, raw failures, review FAIL and prior
counters remain unchanged. New receipts are under
`target/yt144-postgres-workload-c9a61dwf/evidence/measurement-continuation`.

Decoded original argv contained no literal backslash-doublequotes. Its UID-0
scan suppressed failed executable readlinks before filtering executable names.
The bounded owned-postmaster readlink as UID 0 exited 1; the matching
UID/GID 1000:1000 view successfully read executable and FD records for every
host-cgroup PID mapped through `NSpid`. The root readlink emitted no errno, so
only this UID-dependent access result is claimed. Each subsequent sample used
the exact owned namespace PID list, with failures unsuppressed; no host policy,
sudo, capability, sysctl or existing service changed. Complete DB records were
observed before load.

Prior cleanup had deleted the binary, tools, image and PGDATA. Exact k6 archive/
executable and pinned Linux/amd64 image identities were restored in 9.38 s.
The release executable was necessarily reconstructed at its original path in
10.29 s; SHA-256
`a1c7583665cddd204e54df149f48763651d4bbc3642aeda33f192ca7b822d431`
is byte-identical to the original accepted binary. This reconstructed the
measurement artifact; no focused test was compiled/run and no old accepted
workload/build validation was replayed. Credentials were newly generated in
the same owned paths. The identical fixture initialized fresh PGDATA; sorted
10,000-row and schema/constraint/index snapshots matched the retained original.

The renewed, explicitly authorized runtime began at epoch **1792761976.854**,
with absolute deadline **1792762576.854**. The first live preload returned the
correct HTTP 200 JSON, but an existing Python helper looked up lowercase wire
header names case-sensitively and failed. This is a retained **harness failure**,
not a product mismatch. Its single live response, app PID, zero error counters
and native pool-close/DB-stop evidence remain in the continuation root and are
excluded from warmup/measurement. One smallest lookup normalization corrected
the helper. The same PGDATA restarted before any warmup/measured group, within
the unchanged deadline. Focused preload checks then passed. Corrected receipts
are in `measurement-continuation/header-correction/`; earlier receipts were not
overwritten. No repeated load point or additional rate/repetition was run.

Cache history is fresh initialization, settings and sorted logical scans, a
pre-load restart after the failed preload, then per-app exact-query warmup.
No database restart/reseed occurred between paired modes. Retained old plans
and focused proof were reused, with OS cache uncontrolled. Automatic PostgreSQL
vacuum/analyze activity is visible in the table counters; settings were unchanged.

### Actual populations and latency

All eight two-second warmups and ten-second measured groups completed once.
Measured starts/completions/native requests/exact valid 200s are **100,007**,
against 100,000 nominal arrivals; warmup starts/completions/valid 200s are
**4,003**, against 4,000 nominal. Invalid 200s, HTTP errors, transport/timeout
errors, drops and interruptions are all zero, and every native k6 exit is 0.
The measured nominal-minus-starts-minus-drops residual is **−7**, and warmup
is **−3**, retaining native boundary extras. Half-open scheduled windows contain
99,994 measured starts and **99,993 valid completions**; 13 starts and 14
completions occur outside them. Warmup has 4,000 in-window starts, 3,998 valid
in-window completions, three outside starts and five outside completions.
Boundary extras and delayed boundary starts remain distinct; no absent request
is invented or classified as a drop. Per-ID counts, status/error-code
populations, raw JSON, summaries and native exit receipts are retained.

The following floating-millisecond native HTTP timings use nearest-rank
percentiles, excluding blocked/DNS/connect. All-completed, HTTP-response and
exact-valid-200 latency populations coincide here because all attempts succeeded;
`comparison-projection.json` and each raw projection retain them separately.

| Rep | Rate/s | Mode | Valid 200s | Valid/s in 10 s | HTTP p50/p95/p99 ms | Full-call p50/p95/p99 ms |
| --- | --- | --- | --- | --- | --- | --- |
| 1 | 500 | axum | 5,001 | 500.0 | 0.441 / 0.582 / 1.001 | 1 / 1 / 2 |
| 1 | 500 | yetlet | 5,001 | 499.9 | 0.466 / 0.597 / 0.986 | 1 / 1 / 2 |
| 1 | 2000 | axum | 20,001 | 1999.9 | 0.208 / 0.532 / 2.480 | 0 / 1 / 3 |
| 1 | 2000 | yetlet | 20,001 | 1999.9 | 0.202 / 0.533 / 2.215 | 0 / 1 / 3 |
| 2 | 500 | yetlet | 5,001 | 500.0 | 0.444 / 0.586 / 0.943 | 1 / 1 / 2 |
| 2 | 500 | axum | 5,000 | 500.0 | 0.443 / 0.592 / 0.954 | 1 / 1 / 2 |
| 2 | 2000 | yetlet | 20,001 | 1999.8 | 0.209 / 0.532 / 2.487 | 0 / 1 / 3 |
| 2 | 2000 | axum | 20,001 | 1999.8 | 0.210 / 0.496 / 2.145 | 0 / 1 / 3 |

Full-call elapsed time is integer `Date.now()` milliseconds: zero medians
cannot support submillisecond full-call claims. Through-final-completion valid
rates, in table order, are 500.050, 500.000, 2000.100, 2000.100, 500.050,
500.050, 2000.100 and 2000.100/s. Their explicit denominator is last completion
minus scenario anchor (9.999–10.002 s), rather than the scheduled 10 s. Neither
denominator estimates maximum capacity.

### Resources and pair interpretation

All groups meet the declared comparability and client-headroom criteria.
Each measured inner sample window has nine samples over approximately 9.2 s;
actual per-phase endpoints and min/max gaps are in `comparison-projection.json`.
Maximum gaps are 1.161–1.216 s. CPU below uses actual inner cgroup counter
deltas, with one core equal to 100%; it excludes startup/warmup/drain. The DB
cgroup includes the bounded Docker-local observer. DB process RSS sums may
double-count shared buffers; separate cgroup memory is retained.

| Rep | Rate/s | Mode | Mean app / DB / client CPU % | App / DB / client cgroup peak MiB |
| --- | --- | --- | --- | --- |
| 1 | 500 | axum | 9.20 / 13.54 / 77.77 | 4.47 / 72.06 / 83.92 |
| 1 | 500 | yetlet | 9.58 / 14.43 / 77.62 | 4.45 / 74.38 / 87.13 |
| 1 | 2000 | axum | 19.10 / 17.54 / 144.66 | 4.66 / 74.38 / 175.79 |
| 1 | 2000 | yetlet | 18.88 / 17.23 / 144.53 | 4.48 / 76.36 / 178.93 |
| 2 | 500 | yetlet | 9.16 / 13.47 / 77.66 | 4.48 / 76.58 / 178.93 |
| 2 | 500 | axum | 9.34 / 13.74 / 78.09 | 4.58 / 76.58 / 178.93 |
| 2 | 2000 | yetlet | 18.94 / 17.11 / 145.85 | 4.58 / 76.58 / 182.55 |
| 2 | 2000 | axum | 19.36 / 17.47 / 146.10 | 4.61 / 76.58 / 182.63 |

Peak client CPU across inner intervals is 150.34%, below the 320% headroom
ceiling; peak measured-lifetime client cgroup memory is 182.63 MiB, below
819.2 MiB. Client memory peaks are cumulative for its process lifetime, not
fresh allocations per phase. App/DB/client measured CPU-throttling deltas and
memory max/OOM/kill/socket-pressure/pid-limit events are zero. App/k6 FD maxima
are 90/72; observed DB FD maximum is 37. Required PID, executable, CPU, RSS,
FD and cgroup anon/file/slab records are retained. Largest sampled output is
119,695,229 bytes, aggregate sampled evidence 626,376,899 bytes and PGDATA
66,004,208 bytes; disk free remains above 342 GB. All declared caps hold.

Yetlet/Axum ratios are calculated only for the four matching comparable pairs:

| Rep | Rate/s | HTTP p50 ratio | HTTP p95 ratio | HTTP p99 ratio | Scheduled valid-throughput ratio |
| --- | --- | --- | --- | --- | --- |
| 1 | 500 | 1.058 | 1.025 | 0.986 | 0.9998 |
| 1 | 2000 | 0.972 | 1.002 | 0.893 | 1.0000 |
| 2 | 500 | 1.002 | 0.990 | 0.989 | 1.0000 |
| 2 | 2000 | 0.996 | 1.071 | 1.159 | 1.0000 |

Latency ratios above 1 mean slower; throughput ratios compare completed work
at this fixed offered load. Small differences and opposite tail directions
across reversals do not establish stable framework overhead or significance.
No capacity, storage-I/O/cold-cache, remote/TLS, production or universal claim
follows. Historical observation failures and the failed preload remain failed
evidence points; no measured group failed the declared comparison criteria.

### Reconciliation, cleanup and review status

Final sorted logical rows and schema/constraints/index definitions exactly
match both the new pre-load snapshots and retained originals. The world table
has 10,000 fixture inserts, zero updates/deletes and 10,000 live rows; catalog
activity in database-wide statistics is not an acknowledged application write.
All eight app stops report zero acquire/provider/query/overall timeout counts
and `pool_closed: true`, with native stop at most 0.068 s. The failed-preload
app also stopped natively. PostgreSQL stopped with exit 0 and no OOM.

Cleanup completed at epoch **1792762215.089**, within the renewed runtime
deadline. All 173 captured owned DB/app/k6/monitor PIDs, including the failed
preload app, are absent; units, container, TCP listener and Unix socket are
absent. Owned PGDATA, socket, credential, home, build and tool paths and the newly
reacquired unused image were removed. Original Valkey/hello-world images and
existing services remain. One final observation encountered the already-removed
client cgroup (empty `ControlGroup`); that operational observation failure is
retained separately, with no resource mutation before it. Actual empty-group
state, retained measured cgroup samples and native unit cleanup resolved it.
The bounded one-shot static projection command also exited 0 and was collected;
its PID/unit absence is recorded in `analysis-cleanup-proof.json`. No running
server, monitor, watcher or analysis process remains.

The continuation preserves prior operational retries (1), the original image-
identity harness correction, the one renewed observer correction, one new
header-lookup harness correction and one cleanup-observation correction;
product repairs remain zero. Source hashes before/after execution match the
accepted runtime inputs. All native k6 exits were saved before parsing.
`continuation-summary.json`, `comparison-projection.json`, runtime results,
cleanup proofs and raw commands bind this result. The original parent evidence
and failed independent verdict are preserved. Independent final Reviewer
`/root/yt144_postgres_final_review_20261023` returned **PASS with zero issues
and required followups** for this completed continuation. The verdict and
reviewed source bindings are retained in `measurement-continuation/` as
`final-review-result.json` and `reviewed-files.json`. YT-144 is **Done** at
this bounded PostgreSQL comparison scope.
YT-096/YT-020 status, old passing proofs and the no-commit/no-push instruction
remain unchanged.
