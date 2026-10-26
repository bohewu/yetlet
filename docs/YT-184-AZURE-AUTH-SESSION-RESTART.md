# YT-184 — Azure cross-host Auth/Session and persistent restart reference

Date: 2026-10-25 (Asia/Taipei). Status: **YT-184 Done at bounded opt-in cross-host
Auth/Session and orderly persistent-restart reference scope after independent
final Reviewer PASS, zero issues/required followups. Generation4 Auth and
generation5 remaining recovery/VM proof compose acceptance. All five resource
groups/owned credentials are deleted; prior failures and transient503 remain.
No general production/HA/restore or release qualification is implied.**

## Authority, prior commit and retained evidence

The maintainer authorizes YT-183's 20-path strict/builtin commit without Push,
then contract-first minimal isolated Azure cross-host Auth/Session and persistent
restart validation. Reuse sufficient passing evidence, retain failures, delete
all owned test resources, and obtain independent Reviewer PASS before Done.
New changes have no Commit/Push authorization.

YT-183 commit `23cdb869f4fe167d0585045ccb293c6fe0a1da71`, parent `074358d`, tree
`338e7f0ff4e227fd3e6917ec62203d161e6c96ea`, contains exactly the authorized 20
paths. Explicit strict/builtin staged Guard and hook PASS, zero findings, clean
post-commit worktree; no Push. Receipts remain in
`target/yt183-catalog-cache-commit-20261025/`.

Retain [YT-139](YT-139-MULTI-INSTANCE-AUTHORITY-RECOVERY.md) namespace retirement,
[YT-140](YT-140-OIDC-PROXY-REPLICAS.md) canonical-origin/session boundaries,
[YT-170](YT-170-AUTH-SESSION-POSTGRES-WORKLOAD.md) native Auth/Session/SQL consumer,
and [YT-164](YT-164-PRIVATE-HTTP-SSH-COMPARISON.md) /
[YT-169](YT-169-PRIVATE-HTTP-SUSTAINED-1800.md) private network, protected Blob,
Managed Run Command and exact-owned cleanup. Do not replay their load, browser,
IdP, provider matrices or qualify production/HA. Synthetic preverified fixture
login remains synthetic; no external identity protocol is newly tested.

## Actual admission and fixed resource budget

Read-only Azure CLI 2.91.0/AzureCloud/default Enabled subscription is available.
Initial fixed-SKU CLI listing timed out after 55 seconds before allocation;
retain that failure. One bounded ARM REST lookup for the **same** SKU/region
succeeds: Standard_D2s_v5, East Asia zone 1, two cores/eight GiB each, no listed
restrictions; DSv5 and total core quota are 0/20, requiring four new cores.
Selected create/delete/Run Command/storage actions are granted. Fixed Canonical
Ubuntu24.04 image `24.04.202610160` is observed V2. These observations do not
promise dynamic allocation or bypass provider policy.

Azure Retail Prices API reports Linux Standard_D2s_v5 in East Asia at US$0.132
per VM-hour on 2026-10-25; two VMs for 90 minutes estimate US$0.396 compute.
Select **US$5 total experiment ceiling**, including conservative noncompute
allowance; not an invoice/account-discount guarantee. No customer hardware,
production owner or SLO is a completion gate for this isolated reference.

- One fresh random-tagged resource group and allocation generation, two fixed
  VMs, one private VNet/NSG, two NICs, task-only Blob storage and one 32 GiB
  Standard SSD data disk on authority host B, plus bounded OS disks.
- Reuse public IP resources only for outbound connectivity if required by the
  accepted template; NSG denies all public inbound. No public HTTP, SQL, Valkey
  or SSH ingress. Only exact task-private peer addresses access selected ports.
- App A on host A; app B plus one PostgreSQL and one Valkey primary on host B.
  This is one authority failure domain, without failover/HA/distributed SQL.
- One **90-minute absolute lifetime** from first resource creation, stop new
  work at 70 minutes and reserve 20 for cleanup. No clock reset, alternate SKU/
  region, replacement allocation or favorable retry. Failure stops exposure.
- Storage-first authenticated 1 KiB roundtrip before compute. Hash-bind code,
  binaries/images and transferred artifacts; bound new local evidence to
  384 MiB, excluding ordinary compiler cache. No large raw load capture.

## Minimum transport delta and trust

The existing fixture only admits Unix-socket PostgreSQL with TLS disabled,
loopback plaintext Valkey and insecure local cookies. Do not tunnel around or
silently broaden that mode. Add an explicit application `auth-cross-host`
feature and private transport-config-file selector, preserving existing local
features/defaults. Add a separate default-off Session-provider `tls` feature
using the existing Redis client's native Rustls support. No custom TLS/RESP/
HTTP protocol, cryptography, ORM, Core field, AOP or mandatory provider.

| Boundary | Selected configuration |
| --- | --- |
| App ingress | Loopback native app HTTP behind a task-private TLS edge on each host; one canonical `https://auth.yt184.test:8443` origin. Curl selects a concrete private peer with native resolve/SNI/certificate verification |
| Origin trust | Explicit canonical Host and supplied unsafe-method Origin; reject duplicates/mismatch/null. Edge strips forwarding metadata; no request header configures trusted origin or backend. Existing private/CSP headers remain outermost |
| Cookie | Provider's normal Secure/HttpOnly/SameSite=Lax, Path=/, no Domain, `__Host-yt184_session`; no insecure local override in cross-host mode |
| PostgreSQL | Native SQLx verified-full TLS with task CA and expected server hostname, SCRAM application role, no plaintext/fallback. Existing four connections/query/admission bounds and read-only role stay; remote mode selects native pre-acquire health checks for restart recovery, leaving the old local benchmark flag false |
| Valkey | Native `rediss://`, checked server CA/name, authenticated least-scope ACL on the dedicated task keyspace; plaintext listener disabled. No insecure-verification option or raw-URL logging |
| Credentials | Fresh task-private CA/leaf keys and random SQL/Valkey credentials; bounded owner-private files, protected Run Command parameters, never tracked/input archives/logs/results/browser state. Trust roots installed only inside fresh owned guests |

Use the already selected official PostgreSQL18.6 and Valkey9.1.2 digest-bound
images. Dedicated disk directories hold PostgreSQL data and Valkey AOF, with
correct ownership. Select PostgreSQL fsync/full-page-writes/synchronous_commit
on and Valkey append-only persistence with appendfsync always. Native protocols,
server authentication and ACL/role checks provide trust; private IP alone does
not. Native TLS startup with wrong CA/name or absent required configuration
must fail before serving, without secrets or success cookies.

## Authorities, persistence and restart

Account enablement/tenant/permission and documents remain PostgreSQL authority,
queried on every protected request. Valkey remains Session authority, with no
memory fallback, authorization cache or successful response on unavailable
Session/SQL. Existing 303/403/404/503 semantics, CSRF and Session rotation remain.
No business writes or cross-store transaction fencing are added to the web app.

App-process restart with unchanged available authorities may retain a current
Session. Controlled individual service stop/start demonstrates cross-host
503/refusal and same-service recovery, and reconciles acknowledged SQL plus
active/revoked Session outcomes at that limited orderly-restart scope. It does
not establish power-loss/backup-restore/no-revocation-resurrection guarantees.

For the authority-VM persistent restart, stop **all** app consumers before
restarting B. Retain current SQL/disk state. The controller then assigns a new,
never-reused task Session epoch outside the restored authority dataset and
restarts both apps coherently. Do not serve the retired namespace. Require
re-login even if old AOF data contains still-unexpired Sessions. Keep latest
restrictive SQL policy; do not restore old permission merely to match Session
age. The task uses a finite 15-minute Session lifetime in cross-host mode, with
old-record lifetime observed privately if used as epoch-rejection evidence.
This safe re-login policy is deliberately application/deployment-owned, not an
automatic generic provider recovery decision.

## Finite new functional journey

1. Admit private listeners/native TLS identity, transfer hashes and server
   persistence settings. Wrong-peer identity/configuration refuses startup.
   No old workload or throughput point executes.
2. Use native form login on A and the same Secure Session to read complete
   tenant-scoped JSON/escaped SSR on B; other tenant denied. Current SQL revoke/
   disable is observed on both, restoration only by trusted test control.
3. Acknowledged logout on B rejects that credential on A. A fresh current
   Session survives app-process restart while authorities are unchanged.
4. One bounded Valkey and one PostgreSQL service outage each yields redacted
   unavailable responses on both peers; restart each same service and retain
   a fixed two-second initial settle. Following the failed generation4 snapshot
   and actual local reproduction, observe per-client readiness for at most five
   additional seconds at100ms intervals using **read-only** protected requests.
   Retain every503 and full successful JSON oracle before marking each peer
   ready. This controller observation adds no production/mutation retry, fallback
   or larger provider limits; then reconcile persistence/current authority.
5. Write a synthetic durable SQL receipt and restrictive permission, stop all
   consumers, restart authority VM B with its existing disk, adopt fresh epoch,
   reject all retired credentials, require deliberate login and current policy.
   Reconcile complete synthetic SQL receipt before/after; no data rollback or
   stale-policy enable. Fresh login works under explicit current policy.

Use existing native request/SQL/Valkey tools and ordinary bounded test fixtures;
no new benchmark framework, generic cloud orchestrator or validator generation.
Report compile/unit, TLS/HTTP, restart/reconciliation and cleanup separately.
Failure bytes stay failures; no replay of a completed passing step/exposure.

## Cleanup and acceptance

On every outcome stop/reap task processes, retrieve available redacted evidence,
then delete only the exact-owned group after validating ID and random tags.
Poll to actual group absence and zero matching owned resources, including disks,
NICs, public-IP egress resources, network, storage/Blob and private credentials.
Remove local private material; never prune shared caches/pre-existing resources.
An asynchronous delete request alone is not success. The 20-minute reserve
cannot be consumed by new work or result analysis.

Independent contract/implementation mapping review precedes allocation; final
results/failure-retention/cleanup Reviewer PASS precedes Done. YT-184 cannot
claim successful cross-host/persistent restart without its new actual evidence.
YT-181–183 accepted scopes, YT-096 In progress/D1–D6 Pending and YT-020 Blocked
remain unchanged. New changes remain uncommitted/unpushed.

Primary references: [SQLx verified connection options](https://docs.rs/sqlx/0.8.6/sqlx/postgres/struct.PgConnectOptions.html)
and [Azure Retail Prices API](https://learn.microsoft.com/en-us/rest/api/cost-management/retail-prices/azure-retail-prices).
The pinned Redis1.7.1 web page was unavailable; installed source confirms native
Rustls/native roots and default checked hostnames. Selected authority/recovery
policy is application-owned, not an upstream library or Azure guarantee.


## Concrete mapping before allocation

The minimum runtime delta is implemented but cloud acceptance remains pending:
provider default-off native `tls` feature and checked URL selection, explicit
fixture `auth-cross-host`/private transport file, canonical private Host/Origin
with privacy headers on early refusals, Secure Session and native verified SQL.
Standalone TLS explicitly enables mature Ring so it does not depend on another
consumer's hidden backend feature. Five initial new test executions pass; a
sixth new case covers early-refusal private headers. Existing cases are filtered,
not replayed; changed-source Clippy/build evidence is separate. No new runtime
parser, registry, Core API or generic provider interface is introduced.

Run-local execution reuses the accepted `transport.py`/`guest-transport.py`
seams, adapted only for task peers,1200second cleanup reserve,384MiB result cap
and protected private-file names. Fixed bootstrap and finite journey scripts
use native Azure CLI, OpenSSL, curl, SSH control (no application tunnel), Docker,
PostgreSQL/Valkey clients and systemd. Credentials stay outside input archives/
returned ZIPs; complete response oracles and safe synthetic SQL receipts are
retained without cookies/tokens. The TLS edge may add its native Server header;
application bodies/private security headers remain checked, without reusing the
old benchmark's direct-server envelope as an edge claim. Public ingress is
denied; public-IP resources serve outbound connectivity only.

All concrete code/binary/template/public-certificate bindings are reviewed before
the only resource creation. The guest ABI admits maximum required GLIBC at most
2.39 against the fixed Ubuntu24.04 image. Private certificates/credentials have
no retained content or password/key hashes. Source/lock/image/binary inputs,
resource budget and phase mapping must match at execution; review is not a
claim of cloud success. Artifacts live in
`target/yt184-cross-host-20261025/evidence/`; initial CLI timeout remains retained.


## Actual first allocation — bootstrap failure, not functional acceptance

Preallocation independent Reviewer
`/root/yt184_cross_host_mapping_review_20261025` returns PASS, zero issues/required
followups. All 16 source,106 nonsecret evidence,18 execution-input and12 retained
bindings match; native role/model/effort observation and central resolver are
verified/enforced. This historical result accepted the mapping, not cloud success.

One actual resource generation passes authenticated Blob1024byte roundtrip,
ARM template validation and successful provisioning of the fixed two VMs and
32GiB disk. Authority-host bootstrap then exits1 before Nginx certificate install:
`TypeError: unsupported operand type(s) for +: 'PosixPath' and 'str'`.
The author incorrectly combined Path division and filename concatenation in two
certificate-file expressions. This is an experiment-fixture defect, not evidence
of an Auth/Session/provider failure. Static mapping review did not catch it.
No application, SQL/Valkey service or selected Auth/TLS/restart journey starts.
The actual native execution exits1 and remains **failed**.

The original executed bootstrap is preserved byte-for-byte as `bootstrap.failed.py`,
with the preallocation SHA256. One smallest in-place correction parenthesizes
those two filename expressions in canonical `bootstrap.py`; no runtime product,
limit, TLS/authority semantics, hook or Guard changes. Four focused native Python
expression evaluations (both files on each of A/B) pass without filesystem,
sudo, cloud or workload execution. This local correction does not prove guest
bootstrap or any later integration success. No old passing test or cloud step
was replayed and no second allocation was created.

Historical manifests remain unchanged. `harness-correction.json` explicitly maps
the old bootstrap hash to its failed snapshot; eight original reviewed document
snapshots preserve later status/report reconciliation. This is not rewriting
Reviewer acceptance or recertifying failed cloud execution.

| Actual evidence | Outcome |
| --- | --- |
| New native Rust tests | Six successful executions, no old passing-case replay; config/Origin, TLS on/off, standalone crypto backend and early refusal privacy |
| Focused compile/Clippy/fmt/docs | PASS at their recorded changed-source scopes;75 required documents/184 items |
| Current native binary | GLIBC requirement2.34, admitted against fixed Ubuntu24.04/2.39; no cloud app execution claim |
| One cloud allocation | Blob roundtrip/ARM/provisioning PASS; bootstrap-b exits1; all selected functional/TLS/outage/restart steps unexecuted |
| Canonical fixture correction | Two filename-expression parentheses, four local path-expression checks PASS; no cloud rerun |
| Cleanup | PASS: exact group deletion polled to absence, tagged resource inventory empty, local private runtime absent |

Original failure/result/command/native artifacts remain in the evidence directory,
including `bootstrap-b.json`, `session-error.json`, `execution-native.json`,
`bootstrap.failed.py`, `harness-correction.json`, resource inventory/delete polls
and `CLEANUP.json`. Root-owned failure-stop commands attempt only selected units/
containers and retrieve available redacted tails; no successful app drain is
claimed because no app started. The group removes VMs/OS and data disks/NICs/
network/egress IPs/Blob storage, plus guest credential material. The original
resource lifetime is **440.354369879 seconds** (7min20.35s), within5400seconds;
local private credentials are also independently observed absent. No pre-existing
service/resource is changed/deleted; the existing Azure CLI sign-in is retained.

## First-generation remaining acceptance and continuation boundary (historical)

YT-184 remains **In progress**, not Done. Actual checked cross-host TLS exchanges,
complete authenticated JSON/SSR, current policy/logout across apps, same-service
outage recovery, persistent authority-VM restart, unexpired old-epoch rejection
and full SQL reconciliation are all unexecuted. Cleanup and a corrected local
fixture are separate from those missing proofs. No production/HA/restore or
cross-host Session qualification is inferred from unit/compile evidence.

The already selected [scheduling bound](YT-181-184-APPLICATION-FOLLOWUPS.md#yt-184-minimum-useful-azure-experiment)
and this contract admit one allocation generation; it is spent and deleted.
A second resource generation requires explicit maintainer continuation approval
rather than silently replacing this failed one. Keep the same item and canonical
fixture, retain the first failure, do not reset/replay old qualification, and
establish the next generation's concrete cost/lifetime admission before creation.
Failure-retention/correction/cleanup review does not accept the incomplete live
study or authorize resources. No new-change Commit/Push.


## Independent limited closure review

Reviewer `/root/yt184_failure_correction_cleanup_review_20261025` returns PASS,
zero issues/required followups, **only for failure retention, the canonical
fixture correction and cleanup**. It explicitly does not accept full YT-184.
All 16 current source/230 evidence bindings match before/after review; historical
16source/106evidence/18execution/12retained bindings match through the declared
failed-bootstrap and eight-document snapshots. Both returned guest ZIP bootstrap
copies match the original failed input. Native role/model/effort are verified
and the shared resolver enforced without degradation/conflict. Review is read-
only with no tests/builds/cloud queries/allocations.

Only this acceptance/status bookkeeping follows; runtime/test/lock sources stay
reviewed. YT-184 remains In progress awaiting explicit continuation for another
allocation and all missing live proofs. No new Commit/Push.


## Explicitly authorized additional resource generation

After the concrete correction, limited independent review and zero-resource
cleanup, the maintainer explicitly authorizes **one additional generation** through the
confirmation prompt, selecting the same additional
same-two-VM isolated generation, keeping US$5 **total** budget, a new maximum
90minute window including20minute cleanup, retained first failure, reused passing
evidence and deletion on every outcome. This is a human-authorized exception
to the original one-generation/nonresetting-window bound, not an automatic
fixture retry or silent replacement. No new Commit/Push.

The same item/canonical fixture continues. Runtime/test/lock code and accepted
binary are unchanged; no old passing test/build is replayed. Run-local execution
code is byte-identical to the corrected canonical seams except a fresh resource
owner nonce; ARM topology changes only owner tags/public SSH key. Fresh CA/leaf
keys, private credentials, resource/storage names and Session namespace prevent
reuse of deleted authority. Public certificate verification is new input
admission, not a replayed runtime qualification. Both generations' full-lifetime
compute estimate remains under US$0.43 at the retained retail rate, leaving the
remaining total US$5 budget for bounded noncompute use.

The second generation was admitted under those explicit limits. Its actual
outcome is recorded below; first failure and cleanup receipts stay immutable.
YT-184 remains In progress.


## Actual additional generation — SQL observer failure after app startup

Generation 2 uses the explicitly authorized second allocation, unchanged product
sources/locks and accepted binary. Corrected bootstrap passes on both A/B. Both
native app logs reach READY with four SQL pool connections each; their combined
backend count is eight. Authority-host negative checks record that incorrect
SQL and Valkey TLS hostnames fail before serving. The bootstrap also reaches
its private HTTPS health checks. These limited transport/startup observations
are new evidence, separate from still-missing authenticated and restart proof.

The finite journey then exits1 at its first SQL-connection observer assertion,
before any functional mark. The observer converted PostgreSQL `inet` to `text`,
then compared the result to bare private IP strings. PostgreSQL documents that
this cast retains a netmask, including `/32` for host addresses, whereas
[`host(inet)`](https://www.postgresql.org/docs/18/functions-net.html) returns the
address without its netmask. That predicate cannot match this fixture's peer
set. This is a separate run-local observer defect, not a demonstrated product
failure. Raw query rows were written only after the assertion, so they are
**unavailable** for generation 2; no exact returned rows or independent positive
SQL wire observation is reconstructed from the startup logs.

Preserve executed `generation-2/journey.failed.py` byte-for-byte with its original
prepared SHA256. The canonical observer now uses native `host(a.client_addr)`
and saves safe connection rows **before** assertion; the exact eight-connection,
all-TLS and two-private-peer conditions are unchanged. Also close a reachable
control-error privacy gap: native subprocess timeout diagnostics could include
an argv Session-record key. The existing command wrapper now emits a fixed
error with suppressed original context. One new bounded real-child timeout
check passes with a Session sentinel absent from the rendered error; no old
passing test/build or cloud exposure is repeated for these corrections.
`observer-correction.json` records both deltas and their evidence limits.

The actual generation-2 native execution remains **exit1 / failed**. Its group
and every matching owned resource are gone; local private runtime/credentials
are absent. Cleanup PASS records **493.613556862 seconds** (8min13.61s) from
resource creation through absence, within its5400second window. Failure-stop
commands retrieve available redacted app/service tails before exact-owned
deletion. Original failure, prepared bindings, returned ZIPs, negative-TLS logs,
app READY logs and cleanup remain in `evidence/generation-2/`; the earlier
rejected startup preflight is retained and did not allocate any resource.

## After-generation-2 acceptance and resource boundary (historical)

Six new successful Rust test executions and recorded compile/lint/default-graph
proof stay valid and are not replayed. Both failed cloud generations and their
separate cleanup PASS remain failures, not successful integration acceptance.
YT-184 stays **In progress**: complete cross-host login/JSON/SSR, tenant/current
permission/logout decisions, app restart, bounded authority outages/recovery,
persistent authority-VM restart, unexpired retired-epoch refusal and complete
SQL receipt reconciliation have not executed.

The maintainer's original allocation plus explicitly authorized one additional
allocation are both spent and deleted. **No third allocation is authorized or
created.** The concrete observer/privacy correction is prepared locally for
review; another generation needs explicit human approval with concrete
lifetime/cost bounds. Correction/cleanup review alone cannot accept the full
item, authorize resources or promote Done. No new Commit/Push.


## Independent second-correction review

Reviewer `/root/yt184_second_observer_correction_cleanup_review_20261025` returns
PASS, zero issues/required followups, for generation-2 observer/privacy correction,
failed-input retention, cleanup evidence and current status claims only. All
16 source and20 evidence bindings match before/after; the returned guest ZIP
journey matches the failed prepared SHA256 and parent/generation2 corrected
journeys are identical. Native role/model/effective effort match and the central
resolver is enforced, with no degradation/conflict. Review is read-only without
tests/builds/cloud operations. Only this status bookkeeping follows acceptance.
Full YT-184 remains In progress; this PASS does not authorize another allocation.


## Third generation explicitly authorized

The maintainer explicitly approves one more same-two-VM allocation after the
reviewed SQL observer/privacy correction and both previous cleanup results.
The third generation has a new maximum90minute lifetime including20minute
cleanup, within the same US$5 cumulative experiment ceiling. Both original
failures remain failed and retained; no fourth allocation is authorized.
Continue the same YT-184 item and canonical runtime/binary, without old passing
unit/build/workload replay. Fresh task names, keys, certificates and namespace
admit isolated new resources; topology remains identical except ownership and
public SSH key. Reuse generation2's passed negative SQL/Valkey TLS-name checks
instead of repeating that test block; all new functional journey conditions
remain unchanged. Historical preallocation and correction reviews are reused,
not rewritten. Third-generation evidence is under `evidence/generation-3/`.
Actual remaining Auth/restart acceptance is still pending; no new Commit/Push.


## Actual third generation — first auth form failure and deployment ACL gap

The explicitly approved third generation reuses unchanged product/binary,
corrected observer and accepted negative TLS-name evidence. New certificates
are admitted and ARM topology matches except fresh owner/public SSH key.
Both bootstrap commands exit0. The retained SQL observer now contains exactly
eight encrypted connections, four from each expected private peer. The native
HTTPS wrong-name check passes; its functional mark is retained. This is new
positive cross-host TLS observation, distinct from authenticated access.

The first `GET /auth/form` then fails the expected200 assertion; generation3
native execution exits1 and remains **failed**. No successful login, protected
JSON/SSR, permission/logout decision, outage recovery or restart mark follows.
The original response status/body were not saved, so no exact HTTP status or
wire-level causal diagnosis is claimed. Both apps' redacted failure tails reach
READY. All original executed inputs, ZIPs, failure and prepared hashes remain.

Source inspection establishes a concrete deployment defect on that path:
`Store::prepare` directly sends `TIME` before any Session mutation. The fixture's
app ACL admits connection/scripting/read/write categories and removes dangerous
commands, but lacks `+time`. Native `ACL DRYRUN` on the same pinned Valkey9.1.2
image explicitly denies app-user `TIME`; [Valkey's TIME documentation](https://valkey.io/commands/time/)
places it in `@fast`. Thus this ACL cannot support the provider's mutation
preparation. This source/configuration explanation is an inference for the
uncaptured HTTP failure, not an invented retained503 or server error trace.

Canonical bootstrap adds **only `+time`** to that same namespace-scoped app ACL;
no broader category, administrator permission, keyspace or provider algorithm
changes. Canonical auth-form assertion now includes only its numeric status in
future failure output, without response bodies, Session IDs or CSRF. Executed
`generation-3/bootstrap.failed.py` and `journey.failed.py` match prepared hashes;
parent pre-correction snapshots and all previous generations remain intact.
These corrections have not been reexecuted in Azure.

### Local diagnostic limits and stop

The bounded network-none diagnostic uses the already cached identical Valkey
image and no exposed ports, persistent storage or cloud resources. Initial
startup under the image's default root entrypoint/dropped capabilities did not
become ready; it is retained and the container removed. After matching guest
UID999, `ACL DRYRUN` validly reports app-user TIME refusal. However, subsequent
Lua/TIME probes supplied CLI `--user` without password authentication; their
successful commands do not establish the app identity. The expected-denial
assertions fail; both results and cleanup are retained as invalid diagnostics,
not product or corrected-ACL PASS. Repeated diagnostic-identity failure stops
further harness attempts. No new `+time` runtime proof is claimed and no old
passing qualification is replayed. The relevant source and native ACL simulation
support the minimum correction without treating those invalid probes as proof.

Third-generation cleanup PASS: exact owned group deletion is polled to absence,
remaining owned resources are empty and local private runtime is absent.
Resource lifetime is **469.970589638 seconds** (7min49.97s), within5400seconds.
All three cloud groups/VMs/disks/networks/storage/credentials and all local
bounded diagnostic containers are removed; no pre-existing resource is pruned.
Raw failure, `native-sql-tls.json`, `journey-checks.json`, `acl-diagnosis.failed.json`,
`acl-lua-hypothesis.failed.json`, `acl-direct-time.json`, `acl-correction.json` and
`CLEANUP.json` are retained under `evidence/generation-3/`.

All three allocation permissions are spent. No fourth allocation is authorized
or created. YT-184 remains In progress; independent correction/retention/cleanup
review cannot substitute for remaining actual Auth/restart proof or promote
Done. New changes remain unstaged/uncommitted/unpushed.


## Independent third-generation correction/cleanup review

Reviewer `/root/yt184_third_acl_correction_cleanup_review_20261025` returns PASS,
zero issues/required followups, at minimum ACL correction, failure retention,
diagnostic limits, cleanup and honest-reporting scope only. All16source/24evidence
bindings match before/after review; original executed sources match prepared
hashes and inspected guest ZIP members. Native registered role/model/effective
high effort match; the central resolver is enforced without degradation/conflict.
Read-only review performs no tests/builds/cloud operations. Only this acceptance
bookkeeping follows; corrected ACL runtime proof and full YT-184 remain pending.
Both cloud-allocation and repeated-diagnostic stop boundaries remain active.
No Done promotion, fourth allocation, new-change Commit or Push.


## Explicit local continuation — named identity and Session path

After the stopped diagnostic results above, the maintainer requests continuation
of the proposed local named-app/Session confirmation. This reopens only that
bounded local path with a concrete strategy change: password-authenticated app
and control users from server startup, **default user disabled**, explicit native
CLI `AUTH` over stdin, and native Rust-client named authentication. It grants no
fourth Azure allocation, old passing-case replay or new Commit/Push. All earlier
failures and limited reviews remain historical and unchanged.

The [native CLI documentation](https://valkey.io/topics/cli/) requires a password
with `--user` to send named AUTH; the previous flag-only probes never established
that identity. The new isolated container uses the already cached identical
Valkey9.1.2 image, an owner-private ACL mounted read-only and a fresh dynamic
**127.0.0.1-only** TCP port. Credentials are fresh, supplied through private file,
stdin or child environment, never retained as argv/logs/source/hash artifacts.
No shared service/network configuration or cloud resource changes. The native
server entrypoint runs directly with matching local file-owner UID and dropped
capabilities, avoiding the earlier root-entrypoint ownership issue.

Unauthenticated PING returns NOAUTH. Explicit CLI AUTH for `yt184_app` returns OK
and TIME returns NOPERM under the original category/namespace ACL. The single new
ignored-by-default regression test `yt184_named_acl_time_grant_unblocks_session_lifecycle`
then exercises the unchanged actual Session provider, not a hand-written Lua
approximation. Native Rust app authentication succeeds; control-side CLIENT LIST
observes the app username without retaining raw metadata. The original provider
create responds503 with no success cookie. Trusted control grants only `+time`;
the same app connection can read TIME while out-of-namespace GET and CONFIG
remain NOPERM. A fresh explicit create succeeds201 with Secure/HttpOnly/SameSite
host-prefixed cookie. A second independent manager reads the same Session;
CSRF verification, rotation/old-credential refusal, cross-manager destruction
and revoked-credential refusal pass. Both managers explicitly shut down.

This is **one new passing test, six historical integration cases filtered**,
using `cargo test --offline --locked ... --test provider` with the exact new
name and `--ignored --exact`. Requests use the native Axum router's oneshot path;
provider IO is real local TCP. No HTTP listener, browser, TLS, SQL, tenant/business
authorization, authority outage or restart is tested by this local case. It
supports the minimum TIME/Session fix, not completion of YT-184's Azure journey.
No old Rust/provider/load exposure is replayed.

Initial preparation compile fails because the new test tried private Redis1.7.1
ConnectionInfo fields; installed public clone/set_redis_settings builder repairs
only that test, and the formatting discrepancy is corrected. The failure and
successful focused compilation are retained, without runtime exposure before
compilation succeeds. Actual new test native exit0 and all prior failures remain.
Evidence is under `evidence/local-named-auth/`, including compile/test native
receipts, `proof-scope.json` and `result-cleanup.json`. The sole local container
and private ACL directory are absent after **1.034319162 seconds** of resource
lifetime; task data was tmpfs and no persistent volume was created. Existing
Azure groups remain deleted; no fourth allocation is started or authorized.
Strict Clippy initially rejects five `ok().expect()` test panic paths; replacing
them with fixed-message `unwrap_or_else` preserves error redaction and every
successful path/assertion. The passing source snapshot is retained separately
from final strict compile/lint evidence; the runtime case is not replayed for
that reporting-only delta. Final formatting/strict Clippy/docs/whitespace pass.
Independent local-result review now passes at the scope below; YT-184 stays In progress.


## Independent local named-app/Session acceptance

Reviewer `/root/yt184_local_named_acl_session_review_20261025` returns PASS,
zero issues/required followups, for actual local named identity, minimum TIME
ACL correction, Session provider regression, retained failures/privacy and
cleanup evidence. All17source/20evidence hashes match before/after; eight
existing runtime/configuration/lock bindings still match the prior baseline.
The reviewer verifies the passing-source-to-final delta contains only five
redacted panic-report replacements. Native role/model/effective high effort
match; the central resolver is enforced without degradation/conflict. Review
is read-only with no tests/builds/services/cloud operations. Only status/reference
bookkeeping follows acceptance; no further runtime exposure occurs.

This accepts the local Session/ACL fix, not the still-unexecuted Azure business
Auth/outage/persistent-restart journey. All three authorized cloud generations
remain deleted; another allocation requires explicit maintainer approval.
YT-184 remains In progress, YT-096/D1-D6 and YT-020 states remain unchanged,
and new changes remain unstaged/uncommitted/unpushed.


## Fourth Azure generation explicitly authorized after local acceptance

The maintainer explicitly approves one additional same-two-VM Azure generation
after named-app/actual Session correction proof and its independent PASS. The
same US$5 cumulative ceiling and new90minute absolute maximum including20minute
cleanup apply; no fifth allocation is authorized. All three original failed
exposures and invalid local diagnostics stay retained; this is the same YT-184
item, without workflow restart or automatic resource replacement.

Use the same product/binary/ARM topology, fresh owner/key/certificate/namespace
inputs, reviewed +time-only app ACL and numeric-only auth-form failure status.
Reuse generation2's SQL/Valkey negative TLS checks and generation3's negative
HTTPS-hostname/Host-refusal proof; omit those already-passed request blocks.
Fresh positive private TLS/SQL input admission precedes only remaining full
Auth/logout/current-policy/outage/persistent-restart work. No old unit/build/
benchmark case is replayed. Combined full-window compute bound is under US$0.50
at the retained retail rate, with the same conservative noncompute allowance
inside US$5. Evidence lives under `evidence/generation-4/`; actual results remain
pending. YT-184 stays In progress; no new Commit/Push.


## Actual fourth generation — Auth passes, first Valkey recovery read fails

Generation4 executes the explicitly approved same-size resource allocation with
unchanged runtime/binary and the locally verified +time-only app ACL. Native
bootstrap A/B exit0. Fresh checked SQL/HTTPS admission passes, followed by four
accepted functional marks: complete cross-host login/JSON/escaped SSR with
tenant denial; current permission/disable/restoration on both peers; acknowledged
logout rejection across peers; fresh current Session after app-process restart.
The fixture identity remains synthetic, not newly qualified external IdP login.
These exact passed steps will be reused, not replayed as qualification.

The first authority outage stops Valkey and both peers correctly return503.
The same service starts, the controller sleeps2seconds, then its first expected
200 protected read fails. Native execution remains exit1. Original actual
recovery status/body were not retained, so no exact Azure503/303 or wire cause
is invented. PostgreSQL outage and persistent authority-VM restart/epoch/SQL
receipt steps do not execute. App tails retain READY; Valkey tails show orderly
SIGTERM and a new process starting. Those logs do not establish per-client ready.

Exact-owned cleanup PASS after **467.048307419 seconds** (7min47.05s): group
absent, owned-resource list empty, private runtime/credentials absent. All four
resource generations are deleted; fifth allocation is neither authorized nor
created. Original generation4 inputs are retained, with executed journey now
`generation-4/journey.failed.py` matching prepared SHA256 and returned guest ZIP.

## New local persistence/recovery observation and minimum continuation contract

One new opt-in case `yt184_service_restart_records_refusal_and_read_only_recovery`
uses the same cached Valkey9.1.2, default-off/named app ACL including +time, a
fresh fixed127.0.0.1 port and a dedicated owner-private data directory. Native
append-only persistence uses appendfsync always without RDB. The unchanged
provider's two independent managers seed one Session, then stop/restart that
same container and disk data. Both protected reads refuse503 while offline.
After the same2s settle, each first read is503; later readonly observations
return complete200 with unchanged state at **107.495ms / 110.289ms** beyond the
settle, within the bounded5s observation window. Nothing silently converts a
failed response to success; all four timed status rows remain in native output.
The observed times are one local sample, not customer SLOs or a universal bound.

The installed pinned Redis1.7.1 `aio/connection_manager.rs` Behavior section and
`send_packed_command` explain that the selected connection path can return the
previous dropped/failed-reconnect error and trigger background reconnection.
A quiet fixed delay alone is not client readiness. The attempted pinned docs.rs
page is unavailable; installed primary source is inspected instead. This local
reproduction supports that mechanism, but does not prove the uncaptured exact
Azure response or exclude every other cause in that earlier allocation.

One new test passes, **seven older cases filtered**, including the already
accepted named-app ACL case. New strict Clippy/compilation precede exposure.
The test uses real native TCP/provider IO and router oneshot, with no HTTP
listener/TLS/SQL/VM claim. No product algorithm/limits/API/default dependency or
client retry configuration changes. No mutation is replayed: only the controller
samples new read-only requests until each manager first returns valid200.
Both managers shut down; container and all private AOF/credential files are
removed after **3.882395506seconds**. Earlier tests/failures remain untouched.
Evidence is in `evidence/local-service-recovery/`.

The canonical continuation keeps the initial2s settle, then bounded5s/100ms
read-only readiness observations, saving each peer/status/time before proceeding;
existing complete JSON/security-header/no-cookie checks stay. Unexpected status
errors now include numeric actual/expected status and peer, without cookies,
CSRF or bodies. Keep retired-credential refusal after observed recovery. Future
Auth/login requests only seed fresh active/revoked credentials needed by the
remaining outage/epoch work; omit the already accepted generation4 tenant/policy/
logout/app-restart qualification block. No arbitrary delay increase, production
automatic retry, repeated business mutation or favorable-result discard.
After the future VM restart, reuse bootstrap's native pg_isready and protected
Valkey PING admission, bounded60seconds with retained boolean/time observations,
before the first SQL receipt query. VM/agent completion and systemd Type=simple
are not authority readiness; these are input/readiness probes, without mutation
replay or provider change. These source/contract adjustments are prepared locally
and not cloud-executed.
YT-184 stays In progress pending remaining cloud proof; independent local/result
correction review passes below. No fifth allocation or new Commit/Push authorization.


## Independent fourth-results/local-recovery review

Reviewer `/root/yt184_fourth_auth_recovery_observation_review_20261025` returns
PASS, zero issues/required followups, at generation4 actual Auth results, failure
retention, cleanup, new local recovery proof and remaining-work continuation
scope. All17source/28evidence bindings match before/after, and the original
executed journey matches prepared/ZIP hashes. Eight existing runtime/config/
lock bindings match the previous baseline. Native role/model/effective high
match; resolver enforced without degradation/conflict. Read-only review performs
no tests/builds/cloud operations; only status/reference bookkeeping follows.

This accepts the passed Auth subset and minimal recovery-observation contract,
not full Azure authority/VM restart qualification or another allocation. Both
new local cases and all four failed cloud exposures remain retained. All owned
resources/private data are deleted; remaining actual cloud proof requires a new
explicit allocation approval. YT-184 remains In progress, no new Commit/Push.


## Fifth generation explicitly authorized — remaining proof only

The maintainer explicitly approves one additional same-two-VM Azure allocation
for remaining Valkey/PostgreSQL service recovery and persistent authority-VM
restart, using the independently accepted bounded read-only readiness contract.
Maintain the cumulativeUS$5 ceiling and new90minute absolute window including
20minute cleanup; no sixth allocation is authorized. Reuse the accepted runtime,
binary, topology and canonical continuation byte-for-byte except fresh owner
nonce and protected inputs. Existing Auth/policy/logout/app-restart and negative
checks are historical accepted evidence; fresh login/logout only seeds active/
retired credentials for the new recovery/epoch work. No old unit/build/workload
qualification replay, discarded503, production retry or larger limits.

All four earlier cloud exposures and cleanup receipts remain unchanged. Combined
compute full-window estimate remains underUS$0.54 at the retained retail rate,
with the same noncompute allowance insideUS$5. New task keys/certificates,
resource names, Session epoch and ownership bindings admit isolated fresh inputs.
Evidence lives under `evidence/generation-5/`; actual outcome and final independent
acceptance remain pending. YT-184 stays In progress, no new Commit/Push.


## Actual fifth-generation completion and evidence composition

Generation5 native execution exits **0**, with before-reboot, after-reboot and
final-stop commands all exit0. It executes the approved remaining-only canonical
continuation with the same unchanged runtime/binary/topology. Reuse generation4's
successful cross-host synthetic login, complete JSON/escaped SSR, tenant/current
SQL permission/disable/logout and app-process restart proof. Fresh active/retired
Sessions are necessary setup only. The accepted record is composed from two
allocations, not misreported as one uninterrupted successful trial.

Both peers refuse503 during each controlled Valkey/PostgreSQL stop. After the
Valkey start and2s settle, retained additional rows are A503 at11.143ms/B503 at
23.565ms, then A200 at139.902ms/B200 at151.192ms. Successful full JSON/security
headers/no-cookie checks remain. PostgreSQL's corresponding first rows are A200
at35.705ms/B200 at59.305ms. Revoked credentials remain denied after each recovery.
These finite observations are not universal reconnect timings or customer SLOs.
No provider limits, algorithms, retries or business mutation behavior changes.

The controller commits a durable synthetic SQL body and restrictive permission,
records the complete two-account/sixteen-document receipt and stops every app
consumer before restarting the **same** authority VM/disk. Azure restart records
Succeeded; the first native post-restart probe admits both services at0.491s.
The full canonical SQL receipt SHA256 remains
`7a18f9eb72d9fd9a23093f7a7d523495d534d08117737198ac6b09c64db829c9`.
Current restrictive permission stays false, rather than being reset by login.

An old active Session record remains present and unexpired in persisted AOF.
Both stopped consumers deliberately adopt a fresh never-reused namespace; four
old-credential/peer combinations refuse303 and require re-login. Fresh login
under current restrictive SQL policy stays403 on both peers. Only explicit
trusted test control re-enables access; then JSON and escaped SSR match the
complete durable body. Logout again rejects the fresh credential. Both app and
authority/edge services explicitly stop before deletion. This proves selected
coordinated new-epoch re-login, not arbitrary restore, power-loss safety or HA.

### Final native cleanup, retained inputs and limits

Generation5 cleanup PASS after **512.496459007seconds** (8min32.50s), within5400s:
exact group deletion polled to absence, global owned-resource inventory empty,
local private credentials removed. All five groups and both local regression
fixtures/private AOF/ACL data are gone. No pre-existing resources, shared caches
or account sign-in are pruned. The retail-rate full-lifetime compute upper
estimate across actual groups is underUS$0.18, excluding noncompute charges and
not an invoice guarantee; the agreed cumulative experiment ceiling remainsUS$5.
Retained evidence is approximately 40.3MiB, below384MiB; ordinary build caches are excluded.

`generation-5/final-results.json` records composition, binary, receipt, statuses,
resource clocks and native cleanup. Final mirrors hold the last phase's files;
original before-reboot checks remain in its returned ZIP. Initial postprocessing
looked for a pre-reboot marker in the after-reboot mirror and asserted; that
reader failure is retained. The smallest correction reads the original ZIP
and final mirror separately; neither input/result is rewritten and no exposure
is replayed. Original/prepared scripts, transfer/public-certificate hashes,
native commands, guest ZIPs and stopped-service receipts remain retained.

Six earlier changed-source Rust executions plus two new ignored local cases
keep successful evidence. No old passed unit/build/load/browser/IdP/provider
matrix is replayed. Recorded format/Clippy/docs/default-graph checks are scoped
to their revisions. No new crate, Core API, mandatory middleware, AOP, browser
dependency or generic retry/deployment framework is introduced. This isolated
Linux reference adds no production/HA/arbitrary-restore/platform guarantee.

Final independent results/source/retention/cleanup review passes below, allowing
Done only at this bounded scope. YT-096/D1-D6, YT-020 and release qualification
remain unchanged. New changes remain unstaged/uncommitted/unpushed; no sixth
allocation is authorized.


## Final independent acceptance — bounded reference Done

Reviewer `/root/yt184_cross_host_final_results_review_20261025` returns **PASS**,
zero issues/required followups, accepting the full bounded opt-in cross-host
Auth/Session and orderly persistent-restart reference. All17current source and
83evidence bindings match before/after; eight runtime/configuration/lock bindings
match the original preallocation mapping. G4/G5 binaries match SHA256
`e22ec1322fb235792e07f03a55d3594bc24c6a716d804e3f9079657c8378d988`.
Prepared bindings/accepted continuation sources match. Native registered role,
model and effective high effort are verified; shared resolver enforced with no
degradation/conflict. Review is read-only without tests/builds/cloud operations.

The reviewer explicitly accepts composed G4 Auth and G5 remaining recovery/VM
proof, full SQL receipt/current policy, unexpired old-namespace retirement,
retained failures/503 and all five exact-owned cleanups. The missing earlier
recovery status is not invented. This is synthetic identity/orderly isolated
Linux evidence, without browser/external IdP/HA/arbitrary restore/power-loss or
release claims. YT-184 is now **Done at that accepted reference scope**.

Only completion/reference bookkeeping follows PASS; no runtime/test/lock,
execution input or retained result changes. YT-096 remains In progress with
D1-D6 Pending; YT-020 remains Blocked; original release gates stay unchanged.
Current changes remain uncommitted/unpushed, index empty; no new allocation is
authorized. Final reviewer/result/observation receipts remain in generation5.
