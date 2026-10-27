# YT-133 — Optional Production Starter single-instance reference

Date: 2026-10-23. Task: **YT-133 / M23 — Done at optional developer reference scope**.
Historical base contract and private operator selection amendment: **Accepted
after independent Reviewer PASS**. Later user-approved framework/responsibility
correction and minimum reusable reference: **Accepted after independent Reviewer
PASS**, zero issues and required followups. Isolated development integration
checks and cleanup pass. No production qualification is claimed.

## Selection and authority

Yetlet supplies optional framework capabilities, reusable integrations and
developer guidance. The user corrected M23 on 2026-10-23: consumer deployment
choices and real operations must not become framework completion gates.
The selected **development reference** is Production Starter, one application
process on one Linux host, native release/systemd lifecycle, private SSH access
through loopback Nginx and local SQLite. It is one possible stack, not a Yetlet
requirement, consumer default or adoption of the current host for production.
Maintenance downtime is acceptable for this reference; HA, zero downtime and
public hosting remain outside it.

The current authorized slice is minimal reusable reference configuration/helpers
and lifecycle guidance, with isolated focused development integration on the
current host, no impact to existing services, cleanup and independent review.
Concrete fixture paths, ports, synthetic credentials, software versions and
resource constraints are for reproducibility. Developers choose their own host,
addresses/accounts/keys, production service users and secrets, operators/on-call,
SLO/RPO/RTO, retention/alerts and production acceptance. These consumer decisions
do not block YT-133 Done. See the [M23 responsibility boundary](PRODUCTION-DEPLOYMENT-PROFILES.md).

Historical planning inputs were branch
`docs/yt113-123-opt-in-capability-plan`, amendment HEAD
`c46c722f92342977facc9daec9350238f819a2b4` and base-contract input
`3cb4e1e04ee0357bdb267dc2457ee832a5351879`. Neither identifies the later
reference executable or a release candidate. The development correction starts
from `0d79ad374e6a216f439f1601ff4fe0654f4609a5`; its changed-source/configuration
identities and executed proof must be recorded separately. A production Profile
is a consumer-owned evidence record, not a settings overlay, Rust API or CLI
feature; see [naming](NAMING.md).

Preserve the [production threat model](THREAT-MODEL.md) and native Axum/Tower,
handler/component-first and opt-in boundaries in
[ADR 0004](adr/0004-request-context-and-middleware.md),
[ADR 0005](adr/0005-opt-in-capability-modules.md) and
[ADR 0006](adr/0006-handler-first-mvc-optional.md).
Optional deployment guidance does not waive security, data-integrity or source
correctness. No Core/crate/API/settings/CLI redesign, generator repair, public
activation or production deployment is selected.

## Source facts: access and edge gaps

[Settings and validation](../examples/production-starter/src/config.rs) accept
`127.0.0.1` only in every settings profile. Existing overrides include
`APP_SERVER__PORT`, `APP_SERVICE__API_TOKEN` and `APP_METRICS__ENABLED`.
Unknown fields are denied; there are no `public_origin`, trusted-proxy or
forwarding-metadata settings to configure.

| Route boundary in current [router/middleware](../examples/production-starter/src/web.rs) | Implemented behavior and limit |
|---|---|
| SSR `/`, `/issues/{id}`, POST `/issues`, POST `/issues/{id}/status` | Anonymous Memory Session `Session<()>`; session-bound hidden CSRF token checked with constant-time comparison before mutation. No operator identity or API Bearer protection. Local HTTP configuration explicitly disables Secure; cookie retains HttpOnly, SameSite=Lax, Path and expiry. TLS termination alone does not qualify access or cookie security |
| `/api` routes, including `/api/v2/issues` | Any Origin header returns 403 before token validation. Exactly one Authorization value with literal `Bearer ` and the configured 64-character token is required; invalid credentials return 401. No Session-based API authentication |
| `/health/live`, `/health/ready`, enabled `/metrics` | Bypass HTTP admission, API authentication and Session middleware; no independent authentication. Protection currently relies on loopback. Disabled metrics returns 404 |
| Response path | Global `Cache-Control: no-store`, `X-Content-Type-Options: nosniff` and strict CSP; Session responses also have cookie/cache variation semantics that an edge must preserve |

The app does not validate Host, Origin or Referer for SSR or consume Forwarded
metadata as trusted authority. Relative actions/redirects do not prove edge
safety. An edge must not strip Origin to bypass the API's 403, replace its Bearer
with edge identity, or treat SSR CSRF as operator authentication.

**Reference access policy:** trusted operator/browser/workstation → SSH key
authentication with verified host identity → local forward bound to
`127.0.0.1` → server-loopback Nginx → Starter's existing `127.0.0.1` listener.
No external HTTP or direct application-port tunnel belongs to this example.
SSH supplies access control, not product login or API Bearer authentication.
Host-local users/processes must be trusted: loopback does not authenticate
against an untrusted local user. See [SSH local forwarding](https://man.openbsd.org/ssh#L).
A local integration fixture must state which parts of this chain it exercises;
unexercised SSH/host/systemd behavior cannot be claimed as verified.

The reference must restrict its canonical Host, preserve API Origin 403/Bearer,
security headers, `no-store`, `Set-Cookie`, `Vary: Cookie` and CSRF behavior, and
deny health/metrics through the business proxy. Keep control access private and
bounded. Never strip Origin to make the API work or invent a trusted-proxy
setting. Local fixture accounts/ports/authorities are examples, not consumer
production identities or defaults.

HTTP/non-Secure cookies are limited to trusted loopback within the encrypted
SSH model. The cookie is **not Secure**; public HTTPS/Secure-cookie qualification
requires a separate application/access decision. Memory Session remains
anonymous CSRF state, not identity; reload forms after restart. No Valkey, IdP
or Auth product is selected. Public TLS/IdP/account-policy/distributed-identity
scenarios do not apply to this reference's private single-process example;
that boundary does not waive direct-access, Host/header/CSRF/Bearer, secret or
storage correctness. Consumers must reassess it before changing exposure.

## Source facts: startup, runtime and probes

[Startup](../examples/production-starter/src/main.rs) loads settings, initializes
JSON logging, validates the address, synchronously opens/migrates/verifies the
Store, optionally attaches metrics, sets readiness false, registers native
signals, constructs the router, binds, then marks readiness true and logs
`started`. Signals are registered before bind but after blocking database
startup. Startup failure exits 1 without a listener; there is no HTTP startup
503 surface. The internal ready marker is not deployment acceptance.

| Probe | Source meaning | Deployment consequence |
|---|---|---|
| Liveness | Reachable `/health/live` returns 200 without SQL/lifecycle inspection | Does not establish business health or data correctness. Shutdown may briefly answer or close the socket; no guaranteed observable 503 interval |
| Readiness | Lifecycle checked before/after SQL and Store closure checked; [Store::ready](../examples/production-starter/src/store.rs) validates exact schema and performs a bounded list query. Caller waits at most 1 s | Shares four SQL permits. Saturation can produce 503; timed-out blocking SQL may continue holding its permit. Not full integrity, backup verification or a write fence |
| Metrics | Explicitly enabled registry/route; no independent authentication and exempt from HTTP admission | Needs a protected caller and separate scrape bounds; no scrape route when disabled |

**Reference runtime policy:** one native release binary with source, locked
`Cargo.lock`, toolchain and binary SHA-256 recorded. Native Linux/systemd is an
optional delivery example. A real deployment should use an independent
least-privilege service user; a disposable developer fixture must document its
actual privilege scope rather than select a consumer production account.
**2 vCPU / 2 GiB RAM / local persistent SSD** is the original reference validation
starting point, not required hardware, actual allocation or capacity evidence.
Record actual fixture resources and any deviations; no hardware qualification
or SLO follows. Docker/Kubernetes and generator delivery are unselected.

Use existing Production settings with explicit absolute configuration/database
paths and a protected unique token through `APP_SERVICE__API_TOKEN`.
A local proof uses synthetic disposable secrets; real secret sources, delivery
and rotation belong to consumers. Never include token values in evidence/logs.

| Reference probe parameter | Required interpretation |
|---|---|
| External startup readiness gate: 60 s | Keep business access closed until success. Type=exec/simple or TimeoutStartSec alone does not prove SQL startup/readiness. Make the bounded gate concrete in the reusable reference |
| Readiness every 5 s, caller timeout 2 s, three consecutive failures | Unavailable without automatic restart. App SQL wait remains 1 s and shares four permits; readiness is not a write fence |
| Liveness every 10 s | Investigation signal, not automatic restart or proof of business correctness |

Avoid overlapping probes; local callers need bounds independent of admission.
App timers need runtime progress.
[systemd ExecStartPost](https://www.freedesktop.org/software/systemd/man/latest/systemd.service.html#ExecStartPost=)
is one possible hook. State the implemented reference gate/probe mechanism and
its actual proof limits. Current Tokio multithread startup sets no explicit
worker/blocking-pool count. Record fixture controls and remaining CPU/RSS/FD/
thread/OS-watchdog limitations without claiming comprehensive resource bounds.
Redacted JSON logs and metrics **off** are the initial reference policy;
optional telemetry integration belongs to YT-136.

## Source facts: application limits versus transport budgets

The current [HTTP admission](../examples/production-starter/src/web.rs) and
[SQL admission](../examples/production-starter/src/store.rs) implement:

| Implemented limit | Exact scope and exclusions |
|---|---|
| 32 HTTP permits, zero queue | Per constructed router and its clones, immediate try-acquire; full capacity or closed Store gives 503. Active handler futures, not TCP connections or a process-global quota across independently built routers |
| 5 s cooperative request deadline | Awaited body parsing and handler/SQL wait through Response creation. Excludes accept/header arrival, response-body transmission and stalled runtime; no SSE/streaming routes currently exist |
| 16 KiB Form/JSON extractor body limit | Not a network upload-time, header, edge buffering or connection-memory bound |
| Four SQL permits, zero queue | Per Store and its clones, not independent Store objects or a cross-instance/process-global quota. Work queued for Tokio blocking execution already holds a permit; the closure retains it through completion/connection close/error/unwind even when the async caller is dropped |
| Per-operation SQLite connections, 2 s busy timeout, startup WAL | Busy timeout covers lock contention, not the whole operation, filesystem I/O or cancellation |

**Reference Nginx parameters:** original selected validation parameters below
are not framework defaults, measured capacity, production requirements or SLO.
Concrete fixture configuration must state any deviations and untested limits.

| Budget | Reference parameter / limit |
|---|---|
| Workers and connections | One worker, 128 worker connections **including upstream connections**, not 128 clients |
| Headers | Arrival timeout 5 s; choose and record bounded header buffer bytes in the reference |
| Request body | 16 KiB, matching the app extractor limit |
| Proxy connection and read | Connect timeout 1 s; read inactivity timeout 7 s |
| Downstream and keepalive | Send inactivity timeout 5 s; keepalive idle timeout 5 s |
| Cache and retries | No proxy cache, automatic upstream retry or mutation replay; no fixed requests-per-second claim |

[Nginx worker connections](https://nginx.org/en/docs/ngx_core_module.html#worker_connections)
include proxy connections. Body/read/send timeouts are inter-I/O inactivity
bounds, not whole-transaction deadlines; the
[proxy read timeout](https://nginx.org/en/docs/http/ngx_http_proxy_module.html#proxy_read_timeout)
is not a 7 s whole-response cap. No 5 s total body-upload cap follows from
[body timeout semantics](https://nginx.org/en/docs/http/ngx_http_core_module.html#client_body_timeout).
The app's deadline excludes TCP/header arrival, body transmission and a wedged
runtime. Document the reference's body-idle/buffering/temp-storage/connection
controls and limits; do not call these parameters comprehensive transport bounds
or require a new timeout/rate-limit framework.

A dispatched write can commit after 504, disconnect or stop. Exit/failure is not
rollback; creates have no idempotency. Preserve **no automatic mutation retry,
upstream retry or replay**, reconciliation of unknown outcomes before manual
retry/re-entry and existing `expected_version` conflicts. An early read can race
unfinished work. A genuine automatic-retry need may separately activate
[YT-137 research](PRODUCTION-DEPLOYMENT-PROFILES.md#conditional-future-work-contracts);
no retry/idempotency implementation is selected here.

## Selected removal, drain and restart policy

The optional systemd reference uses **Restart=no**, manual re-entry after
abnormal exit and a **20 s supervisor stop timeout**. This is a reference
validation parameter separate from traffic-removal propagation; the existing
application grace is **10 s**. Make signal/process scope concrete and allow
that grace plus margin. A wedged runtime can require forced termination with
unknown write outcome; no universal 10 s guarantee or complete OS watchdog
follows. An isolated proof may exercise the equivalent local process lifecycle,
but must identify any systemd/service-scope behavior it did not exercise.

The reusable lifecycle procedure must make these correctness boundaries clear:

1. Preflight the persistent path/data authority and distinguish intentional first
   initialization from restart/restore. [Store::open](../examples/production-starter/src/store.rs)
   creates a missing DB at startup; later launches must not silently accept
   missing/wrong storage through auto-initialization. Document a reference guard
   or explicit refusal procedure and test its selected seam. A green empty DB
   is not successful restore. One SQLite DB/WAL and one writer are assumed.
2. Withdraw business proxy access and existing connections before stop. Readiness
   false alone is insufficient: [YT-131](YT-131-MIXED-VERSION-COMPATIBILITY.md#upgrade-rollback-and-forward-fix-conditions)
   proves live v1 still GET/PATCHes after schema-2 migration while readiness 503.
   The reference must identify the access and old-writer scope it actually closes.
3. Signal and drain HTTP/SQL work; confirm all old writers/processes stopped
   before backup/preflight, migration or cutover. Account for downstream work,
   full 10 s application grace and external stop margin. Document incomplete
   drain/forced-stop outcomes without treating them as successful recovery.
4. Reconcile exit 1, forced kill and unknown write outcomes before retry/re-entry.
   Do not automatically restore older data/binaries. Actual data-loss approval
   belongs to the consumer; the reference must never silently approve loss.
5. Preflight the next binary/schema and data. Startup initializes schema 0,
   transactionally migrates validated schema 1 to 2, accepts exact schema 2 and
   refuses unknown/malformed schema/index DDL. After v2 writes retain schema 2
   and forward-fix: v1 restart refuses schema 2 and restoring schema 1 loses
   later writes. No automatic rollback assumption is safe.

Actual mount identity, production process ownership, cutover propagation,
recovery/data-loss approver and incident responsibility remain consumer guidance.
They are not unfinished YT-133 framework criteria. YT-135 may later supply the
optional backup/restore/upgrade reference with disposable proof; production
RPO/RTO/retention are separate consumer decisions.

## Current optional reference and isolated development proof

The [private Starter reference](../examples/production-starter/deployment/private-operator/README.md)
contains editable Nginx and systemd templates plus a small storage/readiness
helper. It adds no Core, crate, application runtime, Cargo or settings API change.
These files are optional examples; developers can select another deployment stack.

Current checks use Ubuntu 26.04.1, systemd 259.9, OpenSSH 10.2p1 and an extracted
Nginx 1.28.3-2ubuntu1.11 package whose SHA-256 matched distribution metadata.
No package was installed. Disposable state used the host's ext4 filesystem,
not `/tmp` tmpfs. Actual same-UID transient user services, a separate loopback
sshd with disposable keys/known-host verification and an SSH local forward
exercised the reference. This is not dedicated-account or production isolation.
The build used locked/offline release mode, two jobs, nice 10, CPUQuota 200%,
MemoryMax 3 GiB and TasksMax 128. Runtime app limits follow the example unit
(CPUQuota 200%, MemoryMax 2 GiB, TasksMax 128, LimitNOFILE 512 and two Tokio
workers); these are fixture controls, not capacity evidence.

The release binary SHA-256 is
`1ce6aea73a10c4ed15c52af87a842bd7fbc6e83daac5276cb258c3daee9a5796`.
All 345 recorded runtime/source inputs matched before and after execution;
reference/configuration hashes are retained separately with the evidence.

| Focused check | Actual result and limit |
|---|---|
| Startup and storage | Explicit initialization, real readiness hook and guarded ordinary startup passed. Missing, replaced and unexpectedly empty database checks refused startup without automatic reinitialization |
| Private SSH path | Unauthenticated SSH exited 255; disposable key and verified host identity established the loopback forward |
| SSR and API | Native CSRF create/read/status update passed through the tunnel; API returned 401 for missing credentials, 403 with Origin and 200 for the authorized request |
| Proxy boundaries | Wrong Host 421, protected control paths 404, oversized body 413 and oversized header 400; incomplete header connection closed after 5.025 s. Security/no-store/Set-Cookie/Vary headers were preserved |
| Persistence | The exact two synthetic records and schema 2 survived ordinary restart |
| Withdrawal and drain | Gate removal rejected new requests on existing keepalives; an already-admitted blocked write completed with 201 across withdrawal and graceful proxy quit. This synthetic withdrawal/drain took 0.264 s; it is not a workload or SLO measurement |
| Cleanup | Six runtime units and the build unit were inactive with MainPID 0, the SSH client exited, all four owned ports refused connections and disposable credentials were removed. Existing SSH PID/configuration metadata was unchanged |

The fixture explicitly enabled `APP_METRICS__ENABLED=true` to observe the local
control path; direct metrics returned 200 while the business proxy denied it.
This does not change the template/base default-off policy or rerun YT-128's
unchanged disabled-path evidence. Periodic readiness/liveness schedules remain
optional guidance; the implemented and exercised helper is the startup gate.

Evidence is retained at `/tmp/yetlet-yt133-dev-reference-20261023-mft92nl7`:
`result.json`, `cleanup.json`, `commands.json`, `events.json`, build records,
rendered configuration, `final-identities.json` and `failure-history.json`.
One real configuration failure inherited unused gateway temporary paths outside
the fixture; explicit FastCGI/uWSGI/SCGI paths fixed it without host writes.
The initial failure and bounded setup/metadata corrections remain recorded.
No historical load, backup, browser, Windows, IdP or Valkey qualification was
rerun. Current independent Reviewer PASS accepts this bounded reference and
responsibility correction; historical results below retain their own scope.

## Evidence reuse, remaining work and acceptance

| Retained source-bound evidence | Limit retained for this selection |
|---|---|
| [YT-127](YT-127-ADMISSION-OVERLOAD.md), [YT-128](YT-128-OBSERVABILITY.md) | Local HTTP/SQL/deadline/drain and optional telemetry proof. Old Linux 10.054 s failed-drain exit evidence is not a chosen supervisor deadline or Windows qualification |
| [YT-129 B1](YT-129-LOAD-SOAK-RECOVERY.md) | v1/schema-1 debug/tmpfs baseline; eight-client FAIL retained. No v2 release/persistent-storage capacity or soak transfer |
| [YT-130 B2](YT-130-BACKUP-RESTORE.md) | v1/schema-1 local backup/restore, not v2 recovery qualification or production RPO/RTO/cutover |
| [YT-131 C1](YT-131-MIXED-VERSION-COMPATIBILITY.md) | Current local accepted v2/schema-2 status-index, migration/refusal and compatibility proof; not selected-host deployment acceptance |
| [YT-103](YT-103-ROLLING-ASSET-VERIFICATION.md) | Unchanged asset proof reused at its original identity/scope; no recertification |

[Generated-service delivery](PRODUCTION-DEPLOYMENT-PROFILES.md#delivery-mechanism-remains-unselected)
does not deploy this custom Starter. Generated all-interface/token settings and
Pod probes are not compatible by assumption; no generator repair follows.

YT-133 Done requires a minimal reusable optional reference/configuration/helper
and developer guidance, isolated focused integration evidence for the chosen
seams, exact source/build/configuration/fixture identities and limits, cleanup,
and independent Reviewer acceptance. Evidence must distinguish exercised native
process/proxy behavior from unexercised systemd/SSH/host assumptions. Reuse the
accepted SQL/admission/compatibility proofs rather than repeat their full suites.
No actual production host, operator, secret source, SLO/RPO/RTO or production
acceptance is required. The reference implementation, focused local checks and
cleanup are complete. Independent Reviewer
`/root/yt133_optional_reference_review_20261023` returned **PASS**, zero issues
and required followups; all eight reviewed input hashes matched before and after
inspection. Controller acceptance marks **YT-133 Done at optional developer
reference scope**. The next optional item is YT-134 capacity/workload guidance;
it remains Todo and requires explicit activation.
Current evidence is retained separately at
`/tmp/yetlet-yt133-dev-reference-20261023-mft92nl7`.

**Historical private operator selection checkpoint:** amendment Reviewer
`/root/yt133_private_profile_review_20261023` returned PASS with zero issues and
required followups; all four source hashes matched the frozen input before/after
review. Controller acceptance changed review status and next-step records only.
`cargo xtask check docs` passed for 75 required documents and 140 backlog items;
tracked whitespace checks had no diagnostics. Evidence remains at
`/tmp/yetlet-yt133-private-profile-20261023-o2sqmdgf` (`review-input.json`,
`review-input.diff`, `review-initial.json` and check logs). No new runtime/browser/
Valkey/IdP/load/build tests were run at that historical documentation checkpoint;
it authorized no implementation/deployment and does not accept this correction.

**Historical base-contract checkpoint:** `cargo xtask check docs` passed for 75 required
documents and 140 backlog items; tracked/new-file whitespace checks had no
diagnostics. Independent Reviewer `/root/yt133_single_instance_review_20261023`
returned PASS with zero issues and required followups, with all four reviewed
source hashes unchanged. Its controller acceptance updated only review status
and next-step records after that frozen review. Evidence is retained at
`/tmp/yetlet-yt133-contract-20261023-y1rms4z7/` (`review-input.json`,
`review-initial.json`, documentation and whitespace logs).

YT-134 capacity reference, YT-135 recovery reference and YT-136 opt-in telemetry
remain Todo; YT-137–YT-140 remain Todo and conditional. Multi-instance and OIDC
are not activated here. YT-132's historical documentation PASS is unchanged;
YT-096 remains In progress with D1–D6 Pending, YT-020 Blocked and immutable
candidate `ce56e08` with its original nine libraries unchanged.
