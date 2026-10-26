# YT-136 — Opt-in bounded telemetry collection reference

Date: 2026-10-23. Status: **Done at bounded optional collection/logging reference scope**.
Independent Reviewer `/root/yt136_telemetry_collection_review_20261023`: **PASS**,
zero issues and required followups. Seven reviewed file hashes remained unchanged
through review; controller acceptance records only this bounded integration.

## Pre-runtime contract and scope

This optional application/deployment reference reuses accepted
[YT-128 observability](YT-128-OBSERVABILITY.md) and
[YT-133 private access](YT-133-STARTER-SINGLE-INSTANCE-CONTRACT.md).
The reusable [collection commands](../examples/production-starter/deployment/telemetry/README.md)
use installed curl and standard Python only. No collector service, telemetry
crate, mandatory dependency, tracing platform or audit platform is selected.
Consumer retention, on-call, real sink/alert delivery and SLO acceptance are not
repository Done gates. Independent review and controller acceptance are complete
at the bounded reference scope; production operations remain separate.

Before starting an application, copy and hash this contract, the reusable README,
its exact command blocks, copied Production configuration, release binary and
recorded 345 build inputs. Stop on an input/binary mismatch; do not rebuild or
silently substitute a binary. Freeze concrete paths, port, configuration,
resource controls and exact execution command in the private evidence directory.
Preserve failed commands and actual exit codes.

Selected fixture: current checkout at HEAD
`01460938b737fbc1e5399e745de1af4c6584c7d3`; existing YT-133 release
SHA-256 `1ce6aea73a10c4ed15c52af87a842bd7fbc6e83daac5276cb258c3daee9a5796`.
Use a new private directory under ignored `target/` on local ext4, a high unused
loopback port, copied `base.toml`/`production.toml`, Production mode, synthetic
64-hex token in child environment only and two Tokio workers. Native children
use two allowed CPU affinities, nice 10, address-space limit 2 GiB, 512 FDs,
30 CPU seconds and no core dumps. These are fixture controls, not dedicated
CPU/RAM allocation, cgroup isolation, comprehensive transport bounds or capacity
qualification. Startup gate is 60 seconds, stop watchdog 20 seconds; application
grace remains 10 seconds. File output is bounded separately below. No existing
service, cloud resource, global configuration or policy is changed.

Run one enabled instance, one synthetic API create/read, readiness, a bounded
fresh metrics scrape and bounded correlated JSON capture. Stop/reap that child,
verify its listener closed, then collect a failed/down observation without
changing the last-good pointer. Verify the existing schema-2 database before a
sequential explicitly disabled run; require readiness 200 and metrics 404 with
no new valid sample. No missing-storage fallback is allowed on restart.
Record exact acknowledged/stored rows, source/config/binary identities before
and after, stop results, closed descriptors, removed disposable DB/config/token
state and listener closure on every exit. No old SQL fault, overload, browser,
CSP, Windows, proxy/SSH, identity, build or migration matrix is rerun.

## Collection and privacy boundaries

Collection uses only `http://127.0.0.1:PORT/metrics`. It has no authentication;
trusted same-host operators are assumed. The accepted Nginx business reference
blocks health/metrics, and its SSH business forward targets that edge. Reuse
that proof/configuration identity; this exercise adds no proxy/SSH lab. Loopback
and same-UID private files are neither user authentication nor a DoS firewall.
The route bypasses HTTP admission; use one non-overlapping scrape per chosen
interval, with bounded callers even when business traffic is saturated.

The README command requires 1-second connect, 3-second total and 5-second caller
watchdog, no curl rc/proxy/redirect/retry, 64 KiB body and per-file OS output
limits. Publish only successful HTTP 200, expected content type and bounded
source-specific signature; this is not a full Prometheus parser. New observation
records distinguish fresh, 404/disabled-or-missing and other failures. Failed
observations never advance the last-good pointer. A previous sample is historical
even if its values look healthy. Its timestamp/hash/path must travel with it;
no old sample is evidence of current health.

Logs come only from the owned Starter JSON output file. The command reads at
most 128 KiB plus one detection byte, at most 256 lines of at most 4096 bytes,
and emits a fixed event/field subset under a 128 KiB output cap. Oversize input
is rejected with an explicit gap; it is never presented as complete history.
This is a bounded snapshot, not a tailer, rotator or forwarder. The current
source's accepted sensitive-field exclusion proof is reused. New synthetic
correlation demonstrates the actual file-to-collection integration. No token,
Bearer, cookie, headers, body, query, raw SQL or raw error messages are copied
into collected material. Request IDs are generated process-local numbers that
reset at restart, not identities or distributed trace IDs. Original private
application output and filtered evidence have separate scopes.

## Diagnosis and optional operations

| Observation | Operator interpretation and next bounded check |
|---|---|
| Scrape connection failure/timeout | Current observation failed; last-good is stale. Check owned process/listener and private target configuration; do not substitute cached health |
| Metrics 404 with readiness 200 | Check the known typed metrics setting and route/version identity. It can be intentionally disabled; a 404 alone does not establish that |
| Readiness last probe -1/0/1 | No completed probe / sampled not ready / sampled ready. A scrape does not execute a new SQL readiness check |
| SQL admission saturated with active four | Immediate permit exhaustion, no queue. YT-128 already proved this signal under an actual lock; inspect bounded finite outcomes and correlated request IDs |
| SQL unavailable after busy timeout | Existing YT-128 lock evidence explains one cause. Metrics alone cannot identify a lock holder or distinguish every storage failure |
| HTTP deadline plus later SQL result | A dispatched write may finish later. Withdraw/reconcile actual data before manual retry; log/span completion is not durability or audit acknowledgment |
| Collector/log output cap or sink error | Record an explicit collection gap, preserve last-good separately, repair operator storage/access; never report successful delivery |

HTTP timing ends at response creation, excluding body delivery and durable
commit. SQL permit-wait is zero because admission does not queue; it is not
SQLite lock wait or blocking-pool wait. Counters reset on restart. Readiness and
business probes affect HTTP/SQL populations; scrapes exclude themselves but use
CPU, memory and connections. Concurrent registry snapshots are not transactional.
A small sample does not establish latency quantiles or SLO compliance.

An operator may separately select Prometheus scraping against the protected
loopback boundary, with explicit interval/timeout, resource budgets, retention,
access controls and a maintained parser. Alert suggestions are collector-target
unavailable, sample age, readiness failure, finite admission/unavailable/deadline
rates, and missing/truncated log observations. Require minimum samples and
restart/reset handling before rate/quantile decisions. No Prometheus/promtool
binary is installed or service exercised here; this is guidance only. Sink
credentials stay in a protected operator secret source, never command arguments
or log fields. A sink/rotation/journal policy needs its own bounded storage,
loss/backpressure and failure handling. Do not add synchronous remote log writes
to request processing or treat redaction as an audit acknowledgment.

## Execution evidence

The exact direct command `python3 exact-execution-command.py`, retained in the
private evidence directory with its stdout/stderr and executable shell invocation,
exited **0**. Evidence root:
`/tmp/yetlet-yt136-telemetry-20261023-0q5fbm3d`.
`preexecution.json` binds the pre-runtime copies, command/config hashes and
fixture: `target/yt136-telemetry-av8psuuj`, ext4 `/dev/nvme0n1p2`, port 34373,
CPU affinity 0/1. Native children used the declared limits plus a 1 MiB per-file
output cap on application stdout. These are observed fixture limits, not a
systemd/cgroup exercise or an ongoing log-storage policy.

| Focused new integration check | Actual result |
|---|---|
| Release/source reuse | All 345 inputs and release SHA matched before/after; no rebuild, tests or old fault proof rerun |
| Enabled collection | HTTP 200, curl exit 0, collection command exit 0; 3,998 bytes, 0.008882 s curl-call elapsed; snapshot SHA-256 `26f3b65548df8b7238f0e6944a1c17eb46ac033e5175999ae35121f97b69abdf` |
| Owned JSON collection/correlation | 3,098 input bytes became 920 projected bytes / 14 events; no observed gap. Generated ID 2 links POST 201 and `sql_finished outcome=ok`. Input/output privacy assertions exclude the synthetic token and business-body marker; accepted YT-128 supplies the wider exclusion proof |
| Down collection after clean stop | Collection exit 1, curl exit 7, status 000, zero body bytes, 0.008532 s; last-good pointer byte-identical and no valid snapshot published |
| Explicit disabled run | Same verified existing schema-2 DB, `APP_METRICS__ENABLED=false`, readiness/liveness 200; collection exit 1, curl exit 0, HTTP 404, 15-byte candidate, 0.008323 s. This known setting establishes disabled behavior; the command alone reports disabled-or-missing |
| Disabled JSON snapshot | 1,603 input bytes became 466 projected bytes / eight events; no observed gap |
| Oversize log input | Exactly 131,073 bytes read, collection exit 1, explicit failed/gap observation and no projected events file. No complete-history claim |
| Data reconciliation | One 201 acknowledgment, exactly row `[1, "YT136 synthetic collection", "bounded local diagnostic fixture", "open", 1]`, schema/index/sequence unchanged across disabled restart; zero unknown writes |
| Stop/cleanup | Enabled/disabled SIGTERM exits 0 in 0.001118 / 0.001090 s; children reaped, `/proc` entries absent, app log handles closed and listener refused connections. DB/config/collection fixture removed after evidence copy; no token file, child environments gone and synthetic parent token reference cleared |

The fresh snapshot and each failed observation remain separate in `collection/`;
`last-good-before-failure.json`, `last-good-after-down.json` and
`last-good-after-disabled.json` are identical. Recorded paths retain the original
removed fixture prefix; `cleanup.json` maps its collection directory to the
retained evidence copy. `create-correlation.json`, `requests.json`, raw private
`enabled.jsonl`/`disabled.jsonl`, projected JSON, database manifests, exact command
blocks, `reuse-before.json`/`reuse-after.json` and process results support the table.
`private-reference-reuse.json` binds unchanged accepted business-edge inputs;
that proof was reused without starting SSH/Nginx. `tool-availability.json` records
curl/timeout/journalctl available and Prometheus/promtool absent.

No failure/repair cycle was needed: zero operational retries, zero harness fixes,
zero product repairs. The deliberate down/disabled/oversize observations are
expected command behavior, not failed verification. No sink failure, network
stall, metric oversize, full-disk, journal, rotation, remote collector/alert or
sustained-load fault was injected. Size limits rely on the shown curl/OS controls;
the oversize-input check directly exercised only the log rejection boundary.
Compile, browser, CSP, SQL contention, overload and historical private-access
results retain their accepted original scope. No new production or performance
qualification follows. Next optional backlog item is YT-137 research, requiring
separate activation; YT-141 remains Todo and unactivated.
Aggregate documentation checks and independent review belong to the controller.
No commit, push, production deployment or performance activation is authorized.
