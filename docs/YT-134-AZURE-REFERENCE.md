# YT-134 — Optional ephemeral Azure reference

Date: 2026-10-23. Status: **Accepted at bounded optional Azure reference scope after independent Reviewer PASS**.
The accepted [local release/ext4 reference](YT-134-RELEASE-STORAGE-CAPACITY.md)
remains Done at its original scope; its PASS does not accept this supplement.

## Authorization and purpose

The user explicitly authorized the installed Azure CLI to create short-lived
test resources and delete **all task-created resources** when testing finishes.
This adds one optional cloud application reference under the
[M23 responsibility boundary](PRODUCTION-DEPLOYMENT-PROFILES.md), not mandatory
Azure infrastructure, consumer hardware/SLO requirements or production adoption.
No Commit or Push is authorized. Existing local YT-134 work and all historical
PASS/FAIL records remain preserved.

The new questions are cross-VM workload behavior, the actual managed-data-disk
boundary and data readback after an ordinary app-VM reboot. No old unit/browser/
CSP/Windows/IdP/Valkey/fault/backup matrix is repeated. Reuse the accepted release
binary only after its SHA-256 and all 345 build inputs match; no rebuild is
planned. Required native libraries are checked on the selected image first.

## Declaration before provisioning and workload

The ordinary evidence directory retains `azure-plan-before-provision.json`
before resource creation, and the exact workload/script/settings identities
before workload execution. Resource IDs, subscription and operator network
addresses stay in private task evidence rather than repository examples.

| Selection | Bounded reference choice |
|---|---|
| Lifecycle | One newly named/tagged Resource Group; only its resources may be changed or deleted; maximum 60-minute resource lifetime, cleanup on success and failure |
| Region / zone | East Asia / zone 1, subject to actual allocation success |
| Compute | Two `Standard_D2s_v5` VMs, each 2 vCPU / 8 GiB; separate app and load generator |
| Image | Pinned x64 Ubuntu 24.04 image `Canonical:ubuntu-24_04-lts:server:24.04.202610160` |
| OS disks | Standard SSD managed OS disks; application data does not use temporary storage |
| Data disk | App-only 16 GiB `PremiumV2_LRS`, 3,000 IOPS / 125 MB/s, caching None, ext4; record actual disk ID/LUN/device/UUID/mount and Azure properties |
| Control transport | Reuse accepted Azure Managed Run Command plus private authenticated Blob staging; protected short-lived per-blob SAS, no operator SSH/SCP or custom public inbound rule; public IPs provide outbound connectivity |
| Workload path | Private VNet between owned VMs; verified internal SSH local forward from load VM to the app's unchanged loopback listener |
| Application | Existing Production configuration, isolated port/data, ephemeral token, metrics explicitly enabled and two Tokio workers; existing Auth/CSRF/admission/SQL/deadline semantics |
| Resource bounds | App CPU quota 200%, memory 2 GiB, 128 tasks, 512 FDs, Nice 10, no restart policy and 20-second supervisor stop bound; client/monitor at most two CPUs, 512 MiB and 64 tasks |
| Workload | The declared local reference's 20 seeds, fixed short synthetic data and ten-operation 40/30/20/10 mix; six phases totaling 66 scheduled seconds; no automatic retries |

VM family and disk choice describe this fixture only. B-series CPU credits would
add a sustained-load confounder, so this fixture selects a non-B family. Premium
SSD v2 supports independently provisioned disk performance and does not support
host caching; VM-level disk limits and actual observed contention still apply.
See [VM specifications](https://learn.microsoft.com/en-us/azure/virtual-machines/sizes/general-purpose/dsv5-series),
[CPU credits](https://learn.microsoft.com/en-us/azure/virtual-machines/sizes/b-series-cpu-credit-model)
and [managed disk types](https://learn.microsoft.com/en-us/azure/virtual-machines/disks-types#premium-ssd-v2).
These infrastructure specifications are not application throughput guarantees.

Retain the [local workload declaration and investigation thresholds](YT-134-RELEASE-STORAGE-CAPACITY.md#workload-declaration)
unchanged for interpretation on this new environment. A failed load point is
retained and excluded from passing points; it does not authorize tuning, a
replay or a higher-cost configuration. The endpoint remains private and the
application remains loopback-only; HTTP traverses an authenticated SSH forward.
Results therefore include network/tunnel overhead and do not measure Nginx or
bare application maximum throughput.

Capture all/successful full-body latency and throughput, statuses and write
quantiles, app and client CPU/throttling, process/memory/storage observations,
private readiness and actual schema/data invariants. Reconcile acknowledged and
failed/ambiguous writes before one ordinary app-VM reboot. Verify the selected
data disk's mount and existing database before restarting the application and
read back the same rows/schema/index/sequence afterward. No migration, restore,
automatic initialization fallback, HA or power-loss claim follows.

## Preserved public-SSH bootstrap failure

The first YT-134 cloud attempt allocated both declared VMs and attached the data
disk. Authenticated Azure control-plane reads obtained host keys. App-VM public
SSH worked, but load-VM public SSH timed out; one explicit read-only operational
retry repeated the failure. No binary was transferred, no filesystem formatted,
and no app, workload or reboot verification ran. No product behavior was
disproved and no cloud performance result exists for that attempt. The network
failure's complete cause was not established.

The exact task-owned Resource Group and all 11 resources were deleted, including
OS/data disks and public IPs. Group absence and zero owned tagged resources were
verified at 07:39:56 UTC on 2026-10-23, after a 363.77-second resource lifetime.
All ephemeral local keys/token files were removed and local processes completed.
The original plan, blocked execution report and cleanup remain in the evidence
root; they are not overwritten by the continuation below.

## User-requested Run Command/private Blob continuation

The user proposed Blob transfer and requested inspection of previous Azure
connections. The repository already records the same class of public SSH/SCP
failures and a separately accepted [YT-096 Run Command/private Blob
continuation](YT-096-AZURE-RUN-COMMAND.md). Its transport driver and credential
handling are reused as the bounded strategy change; none of its candidate
build/tests or release decisions is rerun or expanded.

Control-plane provisioning/execution and authenticated Blob HTTPS transfer
replace operator SSH/SCP entirely. The Blob container disables anonymous access;
this uses the authenticated public endpoint, **not a Private Endpoint**.
Per-blob SAS grants only input read or separate result/status create/write,
expires within two hours and is passed only as protected Run Command parameters.
The account key stays in a local subprocess environment; credentials are not
retained in evidence or the input bundle and are removed from child workload
environments where applicable. Only synthetic fixtures use protected task-local
credentials. Retained records show credential names/scope/expiry and payload
hashes, not values.

The same two-VM/image/disk/workload bounds above apply to a new uniquely owned
Resource Group after verified deletion of the previous one. No failed public
SSH route is retried. Guest-to-guest workload traffic still uses the private
VNet and verified internal SSH forward; no public app/control listener or
Nginx throughput claim is introduced. Async Managed Run Command must reach a
terminal execution state, native exit 0 and a verified returned guest result;
provisioning success alone is insufficient. Inputs/results use the existing
minimal transport pattern, not a new benchmark framework.

Additive evidence: `run-command-blob/continuation-scope.json` under the evidence
root records user authority, the concrete strategy change, preserved failures,
carried operational history and remaining conditions. New resource ownership,
exact execution/declaration, returned bundle hashes and cleanup are recorded
there separately.

Before guest bootstrap, the first Blob provisioning generation encountered an
Azure CLI response-field mismatch: the local adapter expected `diskSizeGb`,
while the actual response uses `diskSizeGB` (and `diskIOPSReadWrite` /
`diskMBpsReadWrite`). No filesystem formatting, app or workload ran. Cleanup
confirmed group absence, zero case-insensitively matched owned resources and
removed local credentials after 260.85 seconds. The original script, error and
cleanup remain under `attempt1-prebootstrap-harness-failure/`.

One smallest in-place response-map correction was checked against the saved
actual JSON. Only after terminal cleanup did the same owned scope resume with
the same task, model, resource/workload bounds, thresholds and original
60-minute deadline; the clock and failure history were not reset. There was no
second correction, product repair, workload replay or threshold tuning.

## Actual environment and measurements

Both declared VMs, image and disk properties were verified. The exact accepted
release executable and all 345 build inputs matched before and after; required
native libraries resolved in the guest and no rebuild occurred. The app used
Production configuration, schema 2 and the owned ext4 data disk, listening only
on `127.0.0.1:31340`. The actual LUN/device/UUID/mount identity is retained in
`data-disk-binding.json` and the format/mount records.

The app retained the declared 200% CPU / 2 GiB / 128 tasks / 512 FDs / Nice 10,
Restart=no and 20-second stop bounds. The client/monitor budget was explicitly
split before execution: client 190% CPU / 448 MiB / 48 tasks on the load VM;
app-local sampler 10% CPU / 64 MiB / 16 tasks on the app VM. Their combined caps
stay within the declared two CPUs / 512 MiB / 64 tasks. VM agents, SSH and the
kernel are outside these application unit caps.

`cloud-preexecution.json` binds both document hashes, script/config hashes,
resource settings, the six phases and unchanged thresholds before app startup.
The cloud contract hash at execution was
`8475c02a0413113d0abf8f68cb401e6fb27dd1bcedfd3c69856cc9749b505e4a`;
only its hash was captured before execution, not a separate full document copy.
This results section is additive. The 20 authenticated seed creates and warmed
SSR cookie completed before the single 66-scheduled-second workload.

The table preserves the load-side phase classifier, including its readiness
samples. Rates use actual phase duration through last in-flight completion;
quantiles use nearest rank and include full response bodies. CPU is a process
delta estimate with one core equal to 100%; RSS is sampled, not a proven peak.

| Phase | Actual s | Requests | All / successful req/s | All p95 / p99 ms | Workload 503 | App CPU % / RSS MiB | Load-side phase result |
|---|---:|---:|---|---|---:|---|---|
| Baseline, 1 worker | 8.0000 | 351 | 43.88 / 43.88 | 9.48 / 10.61 | 0 | 4.13 / 7.55 | PASS |
| Step 2, 2 workers | 10.0004 | 1,623 | 162.29 / 162.29 | 8.48 / 9.77 | 0 | 11.40 / 7.76 | PASS |
| Step 4, 4 workers | 10.0050 | 5,200 | 519.74 / 519.74 | 8.86 / 13.94 | 0 | 33.88 / 8.51 | **FAIL: readiness** |
| Step 8, 8 workers | 10.0423 | 23,581 | 2,348.16 / 1,712.75 | 6.36 / 15.86 | 6,381 | 91.11 / 9.67 | **FAIL: 503/readiness** |
| Sustained, 2 workers | 20.0000 | 3,223 | 161.15 / 161.15 | 8.45 / 9.25 | 0 | 11.80 / 9.69 | PASS |
| Recovery, 1 worker | 8.0000 | 351 | 43.88 / 43.88 | 9.82 / 11.37 | 0 | 4.13 / 9.66 | PASS |

There were **34,329 workload requests**: 25,059 HTTP 200, 2,889 HTTP 201 and
6,381 HTTP 503, with zero transport/acknowledgement, unexpected-status or monitor
errors. Step 8 rejected 27.0599% of its workload requests; successful-only
p95/p99 were 7.64/24.10 ms. Successful-only quantiles equal all-response
quantiles in every other phase. Lower-load sustained and recovery phases
returned to zero workload rejections. Neither failed point is a passing
capacity claim; a short paced two-worker observation is not a hard limit or SLO.

Creates have different latency from reads. All-create p95/p99 ms were
10.73/11.20, 10.21/12.69, 15.17/25.20, 38.38/109.77, 9.59/11.25 and
11.62/11.88 in phase order. Successful-only Step 8 create p95/p99 were
41.26/133.63 ms; all other create populations coincide. Exact operation/status
counts and p50/p95/p99 remain in `load/result.json`, with per-request/write logs.

### Probe populations and measurement limits

The load-side observer retained 83 snapshots, including 79 assigned to measured
phases: 10/12/12/12/23/10. Phase readiness was all 200 except one 503 each in
Step 4 and Step 8. A separate app-local approximately one-second sampler
retained 149 observations across a wider startup/workload/stop window: liveness
and metrics were 200 in all 149, readiness was 200 in 146 and 503 in three.
These three failures are additional observations, not duplicate workload
responses or automatically identical to the load-side probes.

Client elapsed times and phase boundaries use the load VM's own monotonic
clock. App-local samples use their own monotonic clock and UTC; inter-VM clock
offset was not measured. Their UTC placement is a coarse diagnostic and does
not prove exact phase attribution. The table's original load-side PASS labels
are limited to that observer's phase population; they do not assert every
app-local probe passed in each phase or promote this cloud experiment to an
overall all-probe capacity qualification. All five observed readiness 503s
remain retained, and thresholds were not relaxed.

Across the first-to-last load-metrics window, SQL saturated admissions increased
by 6,386: 6,381 workload rejections plus the two load-side and three app-local
readiness rejections. Acquired admissions and successful SQL operations each
increased by 28,097; these global counters include both probe populations and
their boundary window differs from workload-only counts. This is existing SQL
admission, not opt-in rate-limit middleware or an HTTP 429 policy.

Load-side samples observed 4–10 app threads, 10–26 FDs and at most 9.69 MiB RSS;
the broader app-local window observed 3–10 threads and 10–25 FDs. Sampled database,
WAL and shared-memory maxima were 282,624 / 1,339,032 / 32,768 bytes in the
load-side population; sampling may miss peaks. Per-phase app and client cgroup
`nr_throttled` / `throttled_usec` deltas were zero. The app-local sampler's own
throttling was not captured. Raw process I/O and memory observations remain in
the samples, without equating OS write counters with stable-media flushes.

Remote process snapshots, readiness/metrics and private SSH have measurement
overhead, non-atomic boundaries and possible phase-edge interference. Both
probe populations contend for existing application admission/SQL resources.
The SSH forward/network, VM scheduling, warm page cache and filesystem caching
are part of this fixture; no comparison with local direct-loopback throughput
establishes an Azure speedup or a physical-disk/fsync result. The app's process
CPU/RSS and cgroup observations do not measure all guest/host resources.

## Write reconciliation and ordinary reboot

After 5.05 idle seconds and a clean native app stop in 0.0501 seconds, exact reconciliation found
**2,909 rows = 20 seeds + 2,889 acknowledged creates**. IDs and markers were
unique; complete rows, schema-2 DDL/index and sequence matched. All 540 rejected
create markers were absent and there were no ambiguous transport writes.
The retained inspector returned integrity success; its SQLite connection does
not measure the server's runtime version or per-connection synchronous state.

One ordinary app-VM restart completed. Before explicitly restarting the app,
the guest verified the same mounted ext4 UUID and existing database, changed
boot ID, identical database bytes/SHA-256/inode and the full row/schema/index/
sequence receipt. There was no automatic initialization fallback. After app
restart, liveness/readiness and authenticated first/last issue HTTP reads all
returned 200 with the expected records. A final native stop again returned 0.
This proves ordinary reboot/readback for this fixture, not power-loss durability,
backup/restore, storage loss, failover, HA or multi-instance consistency.

## Cleanup and acceptance

Retain redacted evidence, stop owned activity and remove ephemeral credentials.
Delete the exact task-owned Resource Group after verifying its ownership, then
observe that same deletion until both the group and its tagged resources are
absent. Never delete pre-existing resources or retry an uncertain creation as a
new deployment. Provisioning/test failure still requires cleanup; an incomplete
deletion remains an explicit unresolved condition.

All 20 Managed Run Commands reached Succeeded with native exit 0 and returned
ZIP hashes verified against status records. Both final guest cleanup commands confirmed
inactive owned units with MainPID 0 and removed task API/tunnel/monitor secrets.
The owned Resource Group's 32-resource pre-deletion inventory included two VMs,
three disks, NICs, public IPs, VNet, NSG, Storage Account and 20 Run Commands;
the input/result/status blobs were inside the deleted account.

At **08:12:41 UTC on 2026-10-23**, the exact owned group was absent and a
case-insensitive inventory found zero owned resources. The initial public-SSH
attempt's group also remained absent. Local credential/runtime files were
removed, controller session 51004 terminated with exit 0 and all 345 build
inputs and release executable still matched. The original Blob-scope clock
ran 1,379.90 seconds through terminal cleanup, including the first cleaned
generation and continuation; it stayed below 60 minutes without a reset.
No task cloud resource or running task process is retained.

Evidence: `/tmp/yetlet-yt134-azure-20261023-t5nj6gc1`, especially the additive
`run-command-blob/` directory. The pre-bootstrap failure archive, corrected
response-map record, declarations, redacted Azure/MRC observations, verified
returned bundles, raw workload/sampler data, reconciliation, reboot receipts,
`stage-success.json`, `CLEANUP.json` and unchanged-input records remain there.
Independent Reviewer `/root/yt134_azure_reference_independent_review_20261023`
returned **PASS**, zero issues and required followups. All seven reviewed file
hashes matched before and after inspection; 991 other tracked paths, HEAD,
branch and staged-zero status were unchanged. The reviewer inspected existing
raw evidence without tests, workload reruns or cloud actions. Controller
acceptance covers this bounded optional reference, retained failed points and
measurement limits; it does not declare all load points passing or production
capacity. The local YT-134 PASS remains scoped to the original local fixture. YT-135–YT-140 stay Todo and
require separate activation; this ordinary reboot/readback does not activate
backup/restore research or deployment qualification. YT-096/D1–D6, YT-020 and
the immutable release candidate remain unchanged.
