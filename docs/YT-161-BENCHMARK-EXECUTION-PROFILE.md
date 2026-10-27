# YT-161 — Controlled benchmark profile and isolation admission

Date: 2026-10-24 (Asia/Taipei). Status: **Done at profile/admission scope after independent final Reviewer /root/yt161_benchmark_profile_final_review_20261024 PASS, zero issues/required followups. Controlled performance: Not admitted; suitable target Pending.**

## Authority, scope and explicit stop condition

Evidence: `target/yt161-benchmark-profile-90o1gler/evidence`.

The user asks to continue the isolated-benchmark next step after
[YT-160](YT-160-FREQUENCY-WORKER-DUTY.md). Establish a concrete execution profile,
evaluate actual control boundaries, perform the smallest isolation admission
probe and obtain independent review. Preserve all accepted evidence/failures,
fourteen uncommitted paths, APIs/output/security and product optimizations.
No shared-host setting change, privilege escalation, installation, cloud resource,
commit/push or new product/runtime/sample code is activated here.

**Profile/admission completion is distinct from controlled performance execution.**
If available isolation cannot control the relevant physical CPU policy and
observe its effective behavior, record Not admitted and stop before another SSR
load. Do not manufacture a new timing result on the same uncontrolled host,
rename resource limits as hardware isolation, or claim the historical p95 cause
has been resolved. A suitable actual target remains an external input.

Preserve [ADR 0001](adr/0001-separation-of-concerns.md),
[ADR 0003](adr/0003-renderers-and-loading.md) and
[ADR 0006](adr/0006-handler-first-mvc-optional.md). This is an opt-in benchmark /
application execution profile, not a Yetlet dependency, consumer hardware/SLO,
production operator requirement or framework completion gate.

## What isolation must mean for this question

YT-160 found lower reported frequency accompanying longer construction and
fixed empty CPU durations. Its cached/fallback readings are observational; an
ordinary isolated process is insufficient to establish frequency causality.

| Candidate | What it separates | What must not be assumed | Present decision |
| --- | --- | --- | --- |
| Existing native transient units | Owned processes, affinity, CPU/memory/tasks/FD limits and lifecycle | Exclusive physical CPUs, shared-package power or frequency control | Accepted diagnostics remain valid; controlled-frequency admission pending actual controls |
| Local unprivileged container | Process/filesystem/network/resource boundary | Separate kernel, independent physical P-state policy or a writable governor | One metadata admission probe; no SSR load if physical control is unavailable |
| Ordinary cloud VM | Guest OS and infrastructure boundary | Guest root controls physical host frequency, fixed CPU model, or a provider-wide causal guarantee | Possible separately selected platform comparison; no resource provisioned here |
| Suitable dedicated test host / controlled physical allocation | Declared exclusive benchmark resources and explicitly available host controls | A policy label alone makes real frequency constant | Recommended target class for the controlled question, subject to actual admission |

[Docker's container model](https://docs.docker.com/get-started/docker-concepts/the-basics/what-is-a-container/)
uses a shared kernel; [resource flags](https://docs.docker.com/engine/containers/resource_constraints/)
bound CPU scheduling/memory, not an independent physical frequency policy.
This does not reduce the value of containers for reproducible packaging.

[intel_pstate](https://docs.kernel.org/admin-guide/pm/intel_pstate.html) can let
hardware choose P-states; even performance policy is not a guarantee of identical
instantaneous frequency under power/thermal conditions. A fixed requested limit,
governor label or matching cached sysfs number is not sufficient evidence.

[Azure benchmark documentation](https://learn.microsoft.com/en-us/azure/virtual-machines/linux/compute-benchmark-scores)
reports hardware-dependent scores and variation. [Dedicated Host](https://learn.microsoft.com/en-us/azure/virtual-machines/dedicated-hosts)
is a physical-tenancy service; its name alone does not prove that this task can
set and verify the needed hardware policy. The accepted
[YT-134 Azure reference](YT-134-AZURE-REFERENCE.md) remains a separate platform /
storage result, not frequency-control qualification. Azure CLI availability is
not a reason to create resources that cannot answer this question.

## Proposed execution profile

These are reference choices for a future explicitly selected target, not commands
or host changes performed by YT-161. Record actual values before activation.

| Field | Minimum proposal | Required target evidence |
| --- | --- | --- |
| Purpose | Compare reference/current medium HTTP SSR under matched execution conditions | Claim is bounded to that workload, target and observed controls |
| Isolation | A test-only host or verified physical allocation with benchmark-exclusive CPUs and known SMT/policy siblings | Actual topology/placement and background-work limits; no production-service migration |
| Controls | An explicitly approved, reversible frequency-policy setup on that isolated allocation | Writable/control capability, original settings, proposed settings, authorization scope and restoration plan |
| Frequency observation | Effective active frequency/counter evidence adequate for the comparison | State observer semantics/overhead/availability; cached sysfs alone cannot prove matching frequency |
| Policy interpretation | Select supported policy for that hardware and observe its behavior | Do not equate performance or min=max requests with proven constant hardware frequency |
| Artifacts | Reuse exact YT-159 reference/current executables, locks/libraries/caller/k6 and accepted verification | SHA/ABI/loader compatibility; only target-incompatible artifacts need separately scoped rebuilding |
| Workload | Keep medium128 rows,1,000/s10s,64 fixed VUs, two-worker app; reference1/current1/current2/reference2 | Per-target preexecution manifest, fixed ordering/warmup/baseline and unchanged output/CSP |
| Context | Declare frequency comparability tolerance, warmup and observer limitations before loading | Values are experiment criteria, not Yetlet SLOs; no threshold tuning after seeing results |
| Results | Retain all timing/resource/error/clock/boundary populations and failed admission | No favorable replay, exclusions, normalized cycles from unrelated clocks, or universal capacity claim |
| Lifecycle | Task-private resources, finite deadline/output and exact cleanup | Stop/reap owned processes, restore approved isolated settings, remove only owned resources |

The first controlled comparison should preserve the accepted workload dimensions;
do not add small/large/rate/allocator matrices. YT-160's baseline/warmup/idle
sequence and observation limits are reusable proposals, with per-target review
before execution. Hardware-specific policy values, counter access and tolerance
remain **Pending actual target**, not invented fields to be silently filled by
the framework. A standard VM result may be useful under an explicitly different
observational claim; it must not inherit controlled-frequency acceptance.

## Actual local capability inventory

The current task has no effective Linux capabilities. CPU12/13 policy metadata
is readable, but governor, min/max frequency and EPP are not writable; current
values are powersave,800,000–3,900,000kHz,balance_performance. This is a shared
host, not a benchmark-exclusive physical allocation. No new setting is attempted.

Docker daemon29.9.2 is reachable. A pre-existing immutable local Linux amd64
image is available:
`sha256:418652cfb58ef879d4978c33553735d7147016032d5aefaa14c828e611eb9dfd`.
Its normal Valkey entrypoint must be overridden with /bin/sh for metadata only;
no Valkey server, benchmark binary or HTTP workload will run. Do not pull or
modify/delete that existing image. A Docker metadata template initially assumed
an optional User key and failed; retain exit1 and one structured-JSON correction.
No container existed at that failure.

Read-only ELF inspection of the accepted current executable finds required
GLIBC symbol versions through2.34. That is necessary, not sufficient, for an
unseen target: loader/library/architecture compatibility still needs admission.
No accepted executable or test was executed to obtain that metadata.

## Smallest new isolation admission probe

After independent contract PASS, create exactly one task-owned metadata container
using that exact local image ID, --pull=never, overridden /bin/sh entrypoint,
non-root task UID/GID, read-only rootfs, network none, no published ports,
no bind mounts/volumes, all capabilities dropped and no-new-privileges.
Use CPU set12–13/CPU quota2/64MiB memory and swap-total/16 tasks. No host tuning,
privileged mode, runtime service, new image/build/test or package/network action.

Print only bounded metadata: /proc status allowed CPU list / effective
capabilities, CPU12/13 driver/governor/min/max/EPP and whether those controls
appear writable. Snapshot host policy before/after. Keep the process alive only
long enough for native inspect, then exit normally within10s. Retain exact argv,
immutable image/container IDs, logs, native start/wait/inspect exits and actual
limits. Do not attempt a control write to prove that it fails.

The admission result must separate functioning resource isolation from physical
frequency authority. Matching policy values and unreadable/unwritable controls
are not proof of a separate physical-frequency domain. If the controlled profile
is Not admitted, no SSR/k6 load follows. Do not create cloud resources or grant
extra privileges to repair a failed scientific prerequisite.

## Bounds, reporting and acceptance

Before the probe set one fixed **25-minute** UTC/monotonic deadline, never
renewed. Native Docker commands≤60s, metadata process≤10s and total probe /
analysis/cleanup≤120s. Evidence≤8MiB and task-private ext4 with≥1GiB free.
Use finite native receipts/direct reads only; no admission framework, validator
generator, self-test matrix, recertification or provider bootstrapper.
Classify failures; one smallest correction per distinct seam, same consecutive
signature stops. Preserve all failures, accepted evidence and the original clock.

Stop only the exact owned container if it does not exit; collect logs/status,
remove that container and verify it absent plus its original PID absent. Preserve
the existing image, unrelated containers/services and all inert evidence. Confirm
no ports, volumes or cloud resources were created and host policy snapshots match.
Do not claim transient snapshots prove absence of every external host change.

Repository scope is this report, docs/BACKLOG.md, docs/ROADMAP.md and docs/README.md:
four paths; combined with prior work fifteen paths, staged0. All prior runtime /
sample/source/lock/qualification and historical reports stay unchanged. Run
cargo xtask check docs and git diff --check after documentation changes. No old
Rust/test/build/HTTP/provider/browser/security qualification is replayed.

Independent final Reviewer PASS0 may mark **profile/admission scope** Done.
It must not mark controlled comparison performed, qualified performance or
historical p95 causality resolved. Report actual Not admitted / Pending target
states prominently. Only status/reviewer references in these four documents and
docs/whitespace checks may change after PASS. No product optimization, next live
item, commit/push, YT-096 decision or immutable release-scope expansion.

## Actual admission results

Independent contract Reviewer
/root/yt161_benchmark_profile_contract_review_20261024 returned **PASS**, zero
issues/required followups, before the probe. The exact accepted executable was
inspected as ELF metadata only; no Rust build/test, SSR/k6 population or accepted
qualification was replayed.

| Observation | Actual result | Interpretation |
| --- | --- | --- |
| Metadata process | One container; native exit0;2.165369s from Docker timestamps | Admission metadata only, no Valkey or Yetlet service |
| CPU boundary | Allowed CPUs12–13; quota2 CPUs | Scheduling/resource limits verified; physical exclusivity not established |
| Other limits | Read-only rootfs;64MiB memory/swap-total;16 tasks; UID/GID1000; capabilities0; no-new-privileges | Bounded task-owned container configuration |
| Connectivity/storage | Network none; no mounts/volumes or published ports | No external service, persistent data or network workload |
| Physical policy | Both CPUs expose intel_pstate,powersave,800,000–3,900,000kHz,balance_performance; all inspected controls unwritable | No admitted physical-frequency control; cached policy is not effective-frequency proof |
| Host snapshot | Before/after readable policy equal | No task setting change; not proof that every external host event was absent |
| Cleanup | Exact container removed; original PID2375655 absent; existing image retained | No task-owned live container/process remains |
| Controlled comparison | **Not admitted; not executed** | Actual suitable target, authorized controls and adequate observation remain Pending |

Native create/start/running-inspect/wait/exited-inspect/logs/remove all return0;
wait reports process0, and exited inspect has Running=false,ExitCode=0,Pid=0.
Container ID:
`16af5762f2a81bfe5993f73197d706797198cfc025e364dd130e5fcf1c1c4600`.
The original image entrypoint is overridden; its exposed-port metadata does not
create a port binding. Inspect verifies the actual configured limits, empty
mounts and absent published ports. Resource admission succeeds at this limited
scope; the controlled-frequency prerequisite fails. No additional privileges,
host control writes, cloud provisioning or further load follows that decision.

### Retained preparation and closeout failures

The initial image-inspect template returns1 because an optional User key is
absent. One structured-JSON inspection corrects that metadata assumption before
any container exists. The initial failure remains in
evidence/image-inspect-format-failure.json.

After successful native removal, the wrapper returns1: its absence diagnostic
matcher expects uppercase “No such”, while Docker29 emits lowercase “no such
object”. The original absence receipt has exit1 and structured empty result[];
its false wrapper verdict and traceback are preserved. A single case-insensitive
matcher correction is made without executing the probe again. A read-only
container-list query reconfirms absence; original PID absence, unchanged policy
and retained image are also recorded. The original executed source is preserved
as probe-before-absence-correction.py with SHA256
`c13707d075999c6c576c98502a37982fad689d7c26642547cc4894d59bcfb1d5`;
the corrected probe.py is not a claim of executed source.

The native lifecycle/wrapper completes in2.872989069s, with removal already
successful. Recovering its interpretation and reconfirming cleanup takes the
aggregate to126.935921123s, **exceeding the120s probe/analysis/cleanup bound by
6.935921123s**. Record that timing miss; do not renew the clock, replay the
container, hide the failed wrapper or claim all operational bounds passed.
The deadline miss does not create a new performance population or make the
controlled profile admitted. The separate25-minute stage deadline is unchanged.

A later path-inventory assertion initially receives Git's collapsed untracked
directory listing. Preserve that preparation failure and use
git status --short --untracked-files=all for the exact file inventory; no source,
probe or workload correction follows it.

## Review boundary and next executable step

Native receipts, logs/inspect, immutable preexecution manifest, failure/correction
records and admission-result.json are retained under the evidence directory.
The original executed script, corrected unexecuted script and capture.py remain
task-local artifacts. Repository changes are limited to this report and the
three indexes; prior runtime/sample/history/accepted evidence are preserved.

Independent final review must assess the truthful negative admission and retained
timing limitation, rather than require another uncontrolled load. No tuning or
optimization is selected by this study, and historical HTTP p95 causality remains
unresolved. The next live step is selection/admission of a suitable isolated test
target with explicit control authority and effective-frequency observation.
Apply the proposed profile there before a separately reviewed controlled
comparison; do not expand the Yetlet framework or impose consumer hardware/SLOs.
