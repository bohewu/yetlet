# Issue Tracker production starter (YT-092)

A small, copyable native Rust SSR/API reference application. Create an issue,
read its list/detail, change its status, then restart the process: SQLite retains
committed issues. All pages/forms work without JavaScript, WASM, Vite or a browser
server circuit. This application explicitly selects SQLite, settings, anonymous
CSRF sessions and request IDs; none becomes a framework default.

This is an experimental reference, not supported hosting, HA, OIDC or security
qualification. The anonymous operator pages bind **127.0.0.1 only** in every
profile. Run locally or behind an operator-managed access-controlled boundary.
The command API has a separate shared Bearer token; it does not authenticate the
anonymous SSR pages and is not a user/role system. Do not expose this board as a
public service. Sessions are process-local and expire on restart; issue data is
persistent.

## Run with existing tools

Requires Rust 1.99+ and a native C compiler for bundled SQLite. The repository
CLI is unpublished; use its locally built executable directly. From this
directory, prepare the CLI and app's dependency cache once. On Linux, the app directory
and CLI inputs must be owned by you and not group/world-writable, as required
by the existing [CLI filesystem contract](../../tools/yetlet-cli/README.md):

```bash
cargo build --locked
cargo build --locked --manifest-path ../../tools/yetlet-cli/Cargo.toml
```

Create a private `.env.development` containing `APP_SERVICE__API_TOKEN=` followed
by your unique 64-character lowercase hexadecimal secret. For example, with an
existing OpenSSL executable on Unix, create the file once without overwriting:

```bash
(
  umask 077
  set -C
  {
    printf '%s' 'APP_SERVICE__API_TOKEN='
    openssl rand -hex 32
  } > .env.development
)
```

Select the existing Development secret source, then invoke only the Yetlet CLI:

```bash
export APP_ENVIRONMENT=Development
export YETLET_DEV_SECRETS="$PWD/.env.development"
../../tools/yetlet-cli/target/debug/yetlet run
```

On Windows PowerShell, prepare the same private secret file, protect it with
an operator-owned ACL, then set `$env:APP_ENVIRONMENT = 'Development'` and
`$env:YETLET_DEV_SECRETS = (Join-Path $PWD '.env.development')`. Run the same
locally built CLI with `& ..\..\tools\yetlet-cli\target\debug\yetlet.exe run`.
Python and an outer `cargo run` are not required to launch the application.
If the CLI is already on PATH, the launch command is simply `yetlet run`.

Open <http://127.0.0.1:3010/>. Ctrl-C closes admission and drains work for up to ten seconds.
`yetlet watch --manifest-path <absolute-app-directory>/Cargo.toml` selects the existing Watchexec
watch/rebuild/restart path; Watchexec must already be installed. `yetlet build`
uses the app-owned lockfile and offline Cargo cache:

```bash
../../tools/yetlet-cli/target/debug/yetlet build
```

The existing CLI reads `[cli]` in the same base/profile settings; Vite is disabled.
There are no new CLI options. CLI generators still accept only their declared
scaffolds: this edited reference app does not extend `new --service`, Docker,
Compose or Kubernetes generation. See [CLI contract](../../docs/CLI-CONTRACT.md).

## Manual acceptance

Use this short journey to check the application yourself. Keep the CLI terminal
open so you can stop and restart the same app with the same settings/database.

| Step | Action | Expected result |
|---|---|---|
| 1 | Create an issue with a recognizable title and description | The detail page shows your exact text, status `open` and version 1 |
| 2 | Follow "All issues", then reopen your issue | The list and detail show the same saved issue |
| 3 | Select "In progress" and submit "Update status" | The detail shows `in_progress` and version 2 |
| 4 | Refresh the browser page | The same issue, status and version remain |
| 5 | Press Ctrl-C in the CLI terminal, run the same `yetlet run` command, then refresh | The committed issue remains after restart |

The automated tests separately cover stale-write conflicts, invalid input/body
limits, CSRF rejection, escaping/CSP, configuration precedence and secret-safe
errors/logs. This journey checks visible functionality; formal visual,
accessibility and cross-browser qualification remain separate work.

## Settings and secrets

`yetlet-config` loads `base.toml` → selected `development.toml`, `staging.toml` or
`production.toml` → explicitly selected Development secret file → `APP_*` OS
overrides. `APP_ENVIRONMENT` accepts exactly `Development`, `Staging`,
`Production`; omission selects Production. Every application settings structure
rejects unknown fields. There is no automatic `.env` discovery.

- `YETLET_CONFIG_ROOT`: optional absolute settings directory; parent traversal is
  rejected. Copy all four TOML files there for runtime deployment. It also anchors
  relative database paths, independently of the process working directory.
- `YETLET_DEV_SECRETS`: absolute dotenv file, read only in Development. Staging
  and Production do not inspect even a selected missing secret file.
- `APP_SERVICE__API_TOKEN`: required 64-character lowercase hexadecimal value.
  No usable credential is shipped. Use a unique secret for each installation.
- `APP_DATABASE__PATH`: persistent database path. Default `data/issues.sqlite3`
  is relative to the selected configuration root; an absolute operator-owned
  volume path is supported. Parent traversal, `:memory:` and Windows drive-relative/root-only paths are rejected.
- `APP_SERVER__PORT`: integer 1–65535 (default 3010). Host remains loopback-only.
- `APP_LOGGING__LEVEL`: `error`, `warn`, `info` (default), or `debug`.
- `APP_METRICS__ENABLED`: boolean, default `false`; opt-in app-owned Prometheus
  collection at `GET /metrics`. Older settings without this table remain valid.
  See [YT-128 implementation status and limits](../../docs/YT-128-OBSERVABILITY.md).

For Staging/Production, supply the API secret through the process environment
from the operator's secret manager. Never copy `.env.development` into images,
public assets or version control. The server does not serialize settings, tokens,
paths or database diagnostics. Configure the selected settings root explicitly
when moving the built executable: its default is the original manifest directory.

## HTTP contract

| Route | Behavior |
|---|---|
| `GET /` | Latest 100 issues and create form |
| `POST /issues` | CSRF-protected create; 303 redirect to detail |
| `GET /issues/{id}` | Detail and status form |
| `POST /issues/{id}/status` | CSRF-protected versioned status update; 303 redirect |
| `GET /api/issues` | Latest 100 issues as JSON |
| `GET /api/v2/issues?status=open` | Latest 100 matching issues; required status is `open`, `in_progress` or `closed` |
| `POST /api/issues` | JSON `{title, body}`; 201 with issue/location |
| `GET /api/issues/{id}` | JSON issue |
| `PATCH /api/issues/{id}` | JSON `{status, expected_version}` |
| `GET /health/live` | Process liveness independent of database |
| `GET /health/ready` | Lifecycle readiness and real schema/connection probe |

All `/api` routes require `Authorization: Bearer <token>` and reject browser
`Origin` headers. They are command-oriented; there is no CORS or browser token
injection. Status values are `open`, `in_progress`, `closed`. An issue contains
`id`, `title`, `body`, `status`, `version`. Both forms and JSON cap request bodies
at 16 KiB, title at 120 characters and body at 4,000. Concurrent updates use the
expected version: a stale write returns 409 without changing stored data.
Invalid requests return bounded errors; unknown database failures return generic
500, busy/unavailable storage returns 503, missing issues return 404.

Pages use the existing escaped renderer, English document metadata and strict
CSP. Anonymous sessions use HttpOnly/SameSite cookies, explicit local HTTP mode
and hidden CSRF tokens. API credentials never enter HTML or session state.

## Operate, copy and extend

The app initializes and checks its versioned SQL schema before binding. SQLite
uses WAL, a two-second busy timeout, and at most four dispatched blocking
operations, with **zero permit waiters**. Excess SQL work immediately returns
503. Each operation opens/closes a connection on Tokio's blocking pool; the
closure owns its permit until the connection closes, even if its caller leaves.
Startup rejects unsupported or malformed schemas.

V2 adds an application-owned status index and upgrades schema 1 to 2 in one
startup transaction; issue fields and existing routes remain compatible.
Retained v1 binaries refuse schema 2 at startup. Stop/drain v1 before upgrade;
an already-running v1 can still serve legacy requests despite failing readiness.
Reload held pages to obtain fresh process-local CSRF sessions. Stale expected
versions return 409. The [YT-131 compatibility record](../../docs/YT-131-MIXED-VERSION-COMPATIBILITY.md)
contains actual old/new HTTP/data evidence and the upgrade/rollback procedure.
Rollback requires a verified schema-1 backup and no unaccounted later writes;
preserve schema-2 data and forward-fix after new writes. The B2 runbook's schema-1
commands and measurements describe v1; current backups require schema-2/index
checks and a matching binary. Bounded local C1 verification received independent reviewer PASS and controller acceptance.

Application routes also admit at most 32 active HTTP handler futures with zero
waiters and a five-second request deadline. Capacity rejection is 503, separate
from rate-limit quotas/429. The deadline returns 504 with an explicit warning
that a submitted write may still complete. It covers awaited body parsing and
handler work until Response creation, not socket/header arrival or response
transmission. Both API JSON and browser text errors retain security headers and
request IDs. Health routes and the explicitly enabled metrics endpoint bypass HTTP admission: liveness
performs no SQL; readiness uses the same four SQL permits and a one-second probe
budget, so saturation or shutdown returns 503. There are no streaming/SSE routes.
`web::app` keeps its signature and selects these defaults; consumers can select
`web::app_with_admission` with `Admission` (1–1024 requests, positive deadline up
to 60 seconds). Existing settings files need no new keys.

The SQLite busy timeout bounds lock waits, not the whole operation or filesystem
IO. Dropping a request before SQL dispatch starts no work; once dispatched,
blocking work may run or commit after a 504, disconnect or shutdown signal.
**Do not blindly retry a timed-out create:** it has no idempotency key. Inspect
current persisted issues and reconcile uncertain outcomes; a list read racing
an unfinished write is not proof that it failed. Versioned status updates retain
their expected-version check; reload before deciding whether to retry. A lost
response, 503 or 504 is not evidence of rollback.

The separately activated [YT-137 contract research](../../docs/YT-137-WRITE-OUTCOME-IDEMPOTENCY.md)
reviews this unknown-outcome risk alongside other consumers' retry/replay
behavior, operation identity, authorization, atomicity, retention and recovery.
The accepted decision retains no automatic mutation retry and makes unresolved
identity/late-write ambiguity explicit; independent Reviewer PASS, zero issues
and required followups. This research is Done at documentation-only scope and
does not add an idempotency key, automatic mutation retry or runtime middleware.

Shutdown registers native signals before bind, marks the process unready, closes
SQL/new HTTP admission, then drains Axum handlers **and detached SQL work**.
Only complete draining emits `stopped` and exits successfully. A ten-second grace
expiry emits `shutdown_incomplete` and exits with status 1 without waiting for
Tokio runtime destruction. Remaining writes have unknown outcomes; process exit
is not an application rollback. Reopen/check the database before reconciliation.
The grace is a cooperative runtime deadline, not an OS watchdog against a stalled
runtime. Linux SIGINT/SIGTERM and Windows Ctrl-C/Ctrl-Break are handled. The new
shutdown behavior has local Linux evidence; Windows execution is not rerun here.

See [YT-127 admission evidence](../../docs/YT-127-ADMISSION-OVERLOAD.md) for normal
load, overload/recovery, late writes, shutdown and operating limits. TCP connection,
header/transport and downstream body limits remain deployment responsibilities;
this application policy is not network DoS protection or a capacity benchmark.

Logs are JSON tracing events containing a fixed event name, process-local numeric
request ID, method, status and duration. No request URI/query, bodies, headers,
secret values or raw error chains are logged. `x-yetlet-request-id` is generated
by the server, overwrites inbound IDs, and resets on process restart. Configure
log collection/rotation outside this application. Native request spans now carry
local correlation into blocking SQL diagnostics. Optional app-owned Prometheus
collection exposes finite method/status, admission, SQL and sampled readiness
signals without a global recorder or background exporter. `GET /metrics` uses
the same loopback-only operator boundary as health, excludes scrapes from HTTP
counts and stays available during application saturation. SQL permit queue wait
is explicitly zero, not SQLite lock or blocking-pool wait. HTTP timing ends at
response creation; detached SQL remains active until its closure finishes.
Neither measurement proves durable writes. Startup/shutdown remain JSON events.

The [YT-128 record](../../docs/YT-128-OBSERVABILITY.md) lists exact signals,
configuration, privacy, collection and lifetime limits. This implementation is
accepted at bounded local A2 scope after concurrent tests, actual localhost fault
collection and independent Reviewer PASS with zero issues or required followups. No distributed
tracing or persistent sessions are provided.

The separately activated [YT-136 bounded collection reference](../../docs/YT-136-TELEMETRY-COLLECTION.md)
adds [optional CLI snapshot/log guidance](deployment/telemetry/README.md), reusing
the accepted metrics/logging and private-edge evidence. Focused enabled, stopped
and disabled collection, last-good preservation, correlated log projection and
oversize refusal passed; owned cleanup is complete with independent Reviewer
PASS, zero issues and required followups, completing YT-136 at bounded optional
reference scope. Collection must distinguish a fresh
valid sample from disabled/down/failed collection and a retained last-good
snapshot. No collector service, sink, alert, log rotation or new runtime is
installed by default; production retention and operations remain consumer choices.

Keep the database and its WAL/SHM siblings on writable persistent local storage.
Use SQLite's supported backup workflow or stop the process before copying the
complete database; do not copy only a live main file or delete WAL files. Restore
into a separate local directory and verify the application can open it before
switching an instance. This example does not qualify network filesystems,
multiple replicas, migration rollback, long-load performance or HA.

Copy this directory and retain its `Cargo.lock`. Until Yetlet is published, adjust
the five Yetlet dependency paths in `Cargo.toml` to your framework checkout;
`yetlet-server-axum` resolves its companion local crates there. No registry
release is assumed. Application-specific SQL lives in `migrations/`, settings in
`src/config.rs`, handlers/views in `src/web.rs`, persistence in `src/store.rs`,
and startup/signal ownership in `src/main.rs`. There is no ORM, DI container or
required Controller. Authentication, pagination and deletion are later app choices.

```bash
cargo test --locked --offline
cargo clippy --locked --offline --all-targets -- -D warnings
cargo fmt --check
```

See [YT-092 verification](../../docs/YT-092-STARTER-VERIFICATION.md) for actual
platform evidence, limitations and the lessons that should inform later
scaffolding work.

The [YT-129 local load/soak/recovery record](../../docs/YT-129-LOAD-SOAK-RECOVERY.md)
reports one predeclared synthetic run on the retained v1 debug binary and temporary
**tmpfs** SQLite data. Passing paced normal points span 43.64–467.01 requests/second;
two clients sustained 153.08 requests/second for 45 seconds with zero errors or
rejections and p99 5.80 ms. The eight-client step failed the declared rejection
and CPU thresholds. Forced SQLite-lock overload recovered and owned resources
were cleaned. YT-129 is Done after independent Reviewer PASS at this bounded local scope;
these measurements do not qualify persistent storage, release builds, long soak
or deployment capacity.


The [YT-134 release/storage capacity reference](../../docs/YT-134-RELEASE-STORAGE-CAPACITY.md)
records 23,549 predeclared direct-loopback requests against the accepted release
executable and disposable schema-2/ext4 data. The highest tested passing point
was 452.48 req/s for about ten seconds; two workers sustained 153.65 req/s for
20 seconds. Step 8 failed with 27.5568% 503 and two readiness 503 samples; lower
loads recovered, acknowledged writes reconciled and owned resources were cleaned.
YT-134 is Done at bounded optional reference scope after independent Reviewer
PASS, zero issues and required followups. These measurements do not
qualify SSH/proxy throughput,
consumer hardware/SLO, long soak, crash durability or production deployment.
The separately authorized [ephemeral Azure supplement](../../docs/YT-134-AZURE-REFERENCE.md)
uses separate app/load VMs and a managed data disk. Its 34,329-request workload,
write reconciliation, ordinary reboot/readback and full deletion are complete,
with separate independent Reviewer PASS, zero issues and required followups.
Failed cloud load points and probe/clock limits remain explicit; local PASS
keeps its own scope.

The separately activated [YT-135 current-schema recovery reference](../../docs/YT-135-CURRENT-SCHEMA-RECOVERY.md)
targets the accepted release binary and schema 2, including the status index,
complete business rows/sequence, safe fresh restore and explicit loss accounting.
The three-row snapshot restored exactly; two deliberately excluded source
acknowledgments were reconciled, zero unknown outcomes remained, a missing-index
candidate was refused, and an accounted postrestore create plus same-release
restart preserved all four rows. Owned cleanup is complete; independent Reviewer
PASS has zero issues and required followups, completing YT-135 at bounded optional reference scope. Historical YT-130/131 results below retain their original binary/schema scopes. The separately activated
[YT-141 performance reference](../../docs/YT-141-PERFORMANCE-VALIDATION.md) compares
community tools before a bounded new controlled-arrival read-only API workload.
Independent contract review passed; the [optional k6 script and guidance](deployment/performance/README.md)
produced 14,006 measured valid-200 responses plus 501 warmup at fixed 100/500/s,
zero drops/errors, p50/p95 1/2 ms and p99 2–4 ms. Both repetitions/recovery
met the investigation bounds; no saturation maximum is established. Twenty
acknowledged rows/schema/index/sequence were unchanged, and native stop/owned
cleanup completed. YT-141 is Done at bounded optional local reference scope after final independent
Reviewer PASS, zero issues and required followups; eight hashes matched.
No benchmark dependency or bypass is added to the app.
The separately activated [YT-142 saturation/recovery follow-up](../../docs/YT-142-SATURATION-RECOVERY.md)
has initial contract PASS and new isolated measurement/cleanup. The first
1,000/s point failed with five SQL-admission 503 responses among 10,001 requests
(~0.05%), without CPU or generator saturation. Immediate 250/s recovery completed
7,501 valid HTTP 200 responsess, all 30 bins healthy; full 20-row/schema/index/sequence unchanged.
The ladder stopped before 2,000/s and higher; no new matched passing bracket or maximum is
established. YT-142 is Done at bounded optional observation scope after final independent
Reviewer PASS, zero issues/required followups; eight hashes matched. The load
point remains FAIL. No app
optimization or old passing-proof replay is selected.

The [YT-130 backup/restore runbook](../../docs/YT-130-BACKUP-RESTORE.md) records
a supported SQLite backup, fresh local restore with exact schema/business-data
checks, measured RPO/RTO, process-local Session loss, and actual startup refusal
for an unsupported-schema candidate. Local recovery took 52.36 ms; backup-start
recovery-point age at outage was 41.30 ms, with two deliberately later acknowledged
mutations excluded. These are historical v1/schema-1 synthetic debug/ext4 observations; external
cutover, storage-loss durability and deployment recovery objectives are unqualified.
YT-130 is Done at bounded local B2 scope after independent Reviewer PASS, zero issues and required followups;
YT-131 is Done at bounded local C1 scope after independent Reviewer PASS and controller acceptance. Follow
the version-specific preflight before startup: a missing path
can initialize an empty database, and readiness alone cannot prove recovery.
