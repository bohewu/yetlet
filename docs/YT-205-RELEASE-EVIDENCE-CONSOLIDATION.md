# YT-205 — Sixteen-package source and evidence consolidation

Date: 2026-10-27 (Asia/Taipei). Status: **In progress.** Local reconciliation
and the missing checks below have independent delivery Reviewer PASS, zero issues
and zero packet repair requirements.
Current-candidate CI/rebuild, platform dependency deltas and reader-source
disposition remain open. Reviewer acceptance of this checkpoint alone cannot
mark the whole item Done or approve a release.

## Authority and immutable source

The owner authorized strict/builtin Guard and Commit of the preceding 28 paths,
without Push, then YT-205 reconciliation and only missing verification. Commit
`a2d6474a72d5a4f6a56c19195a348c1d648b32ae`, tree
`1c3eab0616ee7a17f41d673eb0421775fa45c4a5`, contains exactly those paths.
The explicit staged Guard and native hook pass with zero findings; the committed
tree matches `tree_oid` and the working tree is clean afterward.

The owner then explicitly selects that immutable commit as technical candidate
**C for all sixteen experimental 0.1.0 libraries**. This does not authorize
Push, signing, registry publication, release activation or any D1–D6 disposition.
New documentation stays uncommitted. The original nine-package ce56e08 candidate,
its reviews, signatures and historical FAIL / release-ready NO remain intact.
Technical C selection is only part of the unresolved formal source/workflow
identity decision; D1–D6 remain Pending and YT-020 stays Blocked.

Follow the [YT-205 acceptance contract](YT-200-SIXTEEN-PACKAGE-RELEASE-PLAN.md#verification-work-registered-for-later-execution).
Distinguish C's immutable archives, retained historical proof, new local checks
and the later reader-documentation draft. Do not silently substitute a dirty
overlay, moving HEAD or newer dependency for an accepted source identity.

## Source, package and lock reconciliation

All sixteen YT-201 archives remain recoverable with their original digests.
Their runtime, test, example, guide and license payloads match C. Three original
manifests differ only in exact Tokio dev/runtime pins: Config, Secrets and Redis
Session. Ten complete archived locks contain Tokio1.53.2; the six other locks
contain no Tokio. No library runtime source or unrelated package record changed.

New local C archives carry the actual clean VCS SHA and version0.1.0. Their
original manifests and distributed payloads match C byte-for-byte; license texts
match the root MIT/Apache files. Ten locks contain Tokio1.53.3, with all other
package records unchanged. The six other locks and normalized manifests match
the accepted old archives exactly. C's root and isolated locks remain unchanged
through all commands. Actual C archive identities are:
| C archive (all0.1.0) | Included Tokio | SHA-256 |
|---|---|---|
| yetlet-actions | 1.53.3 | `70dd918512c8f4ba108e0c6fe86da9fc458a2f0f9e429c16fd7e7974a0044cdb` |
| yetlet-auth | 1.53.3 | `01d6a4d2c2a8a60c3d94780693aa3362d0f960d9817b2e487ec63f4aa8ce72c9` |
| yetlet-config | 1.53.3 | `95c16df1147c272b85a09803e7197846fd94e6941d5ae55852e0a4e6e84ff1a1` |
| yetlet-core | Absent | `bd34de2b3c0634e3145addaa2beadd464fd8ca116910e2f2646497bd1c1aac85` |
| yetlet-dom | Absent | `9fd1f7a9b0e3c90824404d49df3ea5f3315a814bd9bb5db2addd79d414ea6d9f` |
| yetlet-dom-keyed | Absent | `0d208e0e99ea390edca11d3bc43f7e994e67ab0c348ffec6a885ed1a27b784e9` |
| yetlet-html | Absent | `ed969392a0eb7c9f104e02c2327a28f90951e12a8ccef6ed7232a77b07586d94` |
| yetlet-rate-limit | 1.53.3 | `a55d5ba28bdf1dfba0e0e41aae6b72e957be5518e427b44cc2dca0bb2ebbe10d` |
| yetlet-reactive | Absent | `2401c5f3da919e635f09a2672657219523d381abb04b40a6df9f3d0d56a1febd` |
| yetlet-request-id | 1.53.3 | `ac24870d49c512eacc900e693c46b8e69396732f4fb9f973a626540e43bc365d` |
| yetlet-secrets | 1.53.3 | `61e079d03218dc0ab295aff13282d05684a0f2f9d242bfee0a9afcbea3765969` |
| yetlet-server-axum | 1.53.3 | `4aa4e064739ebbefca52b5b4864391a1eaacc9746f51a2860ba9b127bd2ca405` |
| yetlet-session | 1.53.3 | `ebe9c7e57cfa94e533b57a6c1299bdd2ef5de535acd8f50c839c4ae60b63aaeb` |
| yetlet-session-redis | 1.53.3 | `5eed18b3cfbfc1178becf33d8795818feb9897c3187c514b7e8d33777bacfd9c` |
| yetlet-signing | Absent | `66028488624abaef269026fc790254a1b05aca45bbf43c7377295e6e7e044368` |
| yetlet-wgpu | 1.53.3 | `4d8be3bec03310f5d6315995162927566d0991612a58274fd2d39846ef616b1d` |


**Ten new-input normal package checks PASS**, with ordinary verification,
`--offline --locked`, Rust1.99.0 and four jobs. For the other six, only collect
the C-attributed archive using `--no-verify`: retain prior normal verification
of identical normalized manifests, full locks and source payloads. Collection
is not a new verification PASS. Do not replay six accepted compilations merely
because the VCS metadata has a new SHA.

Three `--exclude-lockfile --no-verify` bootstrap collections only populate an
ordinary directory source for unpublished siblings; they are not qualification
or Git/Guard bypasses. The old pool is read-only. A task-owned pool combines
accepted source entries, current locked Cargo vendor output and new normalized
C packages. Null checksums for local unpublished packages do not establish
actual registry identity, name ownership or downloaded consumption.

One new external native integration consumer passes and prints only
`packaged consumer OK`, selecting Auth/session, Config/secret-provider and Redis/tls
from the sixteen C package sources. Its manifest/program retain the accepted
YT-201 public API cases; its lock changes only Tokio to1.53.3. This checks the
changed package/dependency inputs without replaying old functional workloads.
TLS is compilation/API evidence, without a new transport or live-provider claim.

## Selected inventory and security records

The [YT-204 selected matrix](YT-204-PACKAGE-SECURITY-COVERAGE.md) has42 cells.
Fourteen normal/build closures actually contain Tokio: GNU/MSVC Auth default
and session, Rate-limit, Request-ID, Axum adapter and Memory Session, plus GNU
Redis default/tls. Only these fourteen receive new locked metadata, real
CycloneDX1.5 generation and cargo-deny license/source checks. **All14 PASS**;
every library SBOM exactly matches the selected metadata closure with Tokio1.53.3,
no missing/extra name-version pair and no lock drift. Unused-allowance warnings
remain nonblocking; Zlib/BSL-1.0/ISC and all other policy fields stay unchanged.

The remaining28 normal/build inventories are reused:22 accepted YT-204 closures
without Tokio and six earlier exact-input Core/Reactive cells. The archived
normal dependency records and declarations remain identical for those selections;
changed development/inactive-target Tokio records do not make it a selected
normal/build dependency. Historical complete metadata is not relabeled as fresh
C metadata. MSVC inventory on Linux is not Windows execution evidence.

No advisory scan is replayed. Each of the ten new Tokio-bearing archive locks
has a complete third-party name/version/source/checksum set contained in an
already audited current live lock from the accepted Tokio trial. All13 live
audits used DB `3279c99911354c400e1d41e47c7d60e39258b147` and unchanged audit
policy, exited0 and omitted RUSTSEC-2026-0386. Six exact old archive locks retain
their old audit evidence. The existing RSA exception remains disclosed; no new
ignore is added. Old ten-warning results stay historical, not rewritten.

Tokio1.53.3 is in RustSec's patched range, but retains deprecated safe AsyncFd
constructors. This is not a claim that every upstream API enforces fd ownership.
The separately accepted local trial has95 passing tests/eight application checks
and a cleaned real-Valkey cancellation/shutdown case. Those runs are retained,
not executed again here.

The sealed source scan retains120 regular inputs/18 aliases and zero reportable
findings at its selected library scope. Runtime Rust/Lua/WGSL hashes remain
unchanged; dependency metadata and later reader-only changes are disclosed in
the new binding. This is neither a post-upgrade full source scan nor an
application/repository-wide zero-finding claim. Old Low/Auth review/remediation
identities and all prior failures remain distinct.

## Native, browser and documentation coverage

| Evidence | Reuse or new result at this checkpoint | Limit |
|---|---|---|
| YT-201 packaging | Six exact compilation inputs retained; ten changed-lock C package checks/new consumer PASS | Ubuntu26.04 GNU preflight, directory source |
| YT-202 GNU and MSVC | Accepted API/source/platform results retained; the later once-only Ubuntu24/OpenSSL3.5.9 current-lock integration case below also passes | Old locks retain their identities; the current MSVC dependency delta is not called executed |
| Tokio maintenance trial | New1.53.3 GNU tests/checks/audits retained without replay | Does not replace Ubuntu24 or MSVC execution |
| YT-203 WASM/DOM | Same selected runtime sources and browser inputs; new package manifests differ only in native dev pins | No new DOM/Chrome/Firefox flow is required by unrelated native dependency metadata |
| Optional GPU and BFCache | Original YT-081 actual crate GPU and YT-196 lifecycle evidence retained at their exact scopes | Lab success does not qualify wgpu; no new GPU/OS matrix |
| Required CI | Prior run40431263041 at4e5ef64 remains green historical evidence | No current-C hosted CI; no Push or dispatch occurred |
| Native rustdoc | Eight total generation commands cover16 native package pages plus six WASM pages, all PASS/zero warnings | Selected features documented, not every feature combination or runtime certification |

The eight first-time rustdoc commands use pinned Rust/Cargo1.99.0,
`--offline --locked --no-deps`, explicit manifests and targets. Native workspace
selection includes Auth/session; isolated Config uses secret-provider and Redis
uses tls; Signing retains its native prerequisite. WASM selects the original
five packages and separate wgpu/browser. Generated source, static assets and
22 crate entry points are retained in `rustdoc-pages.zip` with a digest.
No docs.rs or website is deployed, and no browser behavior is inferred from doc
generation. Existing product tests/builds/scans/browser/load experiments are not
replayed. Repository docs checks apply only to the new documentation inputs.

## Reader sources and candidate-documentation boundary

Use [repo guides plus rustdoc](DOCUMENTATION-SCOPE.md), never a second API contract.
The exact manuscript inventory has46 files: the18 explicitly selected docs
pages, six application/tool index or guide pages, five root overview/policy/license
files, sixteen package READMEs and wgpu SUPPORT.md. File paths/digests and local
links are recorded individually; selection is not recursive. No image asset is
referenced by this selected manuscript inventory and none is selected for export.

Fourteen manuscripts contain historical/reviewer/maintainer annotations, and350
local link occurrences target unselected files, such as task records, ADRs,
source/examples and the mixed repository index. Their continued existence in
the repository is valid; a standalone reader site must omit maintainer-only
bookkeeping and deliberately map those links, preserving behavioral/security
limits. This inventory is not a sanitized ready-to-host website or a confidentiality
audit. Destination, exact exported edition and repository visibility stay separate.

The draft current API map now describes all sixteen selected packages without
merging opt-in, platform or provider semantics. Compatibility uses a coordinated
version across independent workspaces. The Memory Session README incorrectly
called the prepared remote provider nonpublishable; the draft fixes it. The
remote provider's draft distinguishes earlier no-persistence proof from later
bounded TLS/ACL/cross-host restart results, without broadening support.

**Those two README fixes were authored after C was selected and packaged.**
C's archives remain byte-identical and contain the original guidance. The draft
is an uncommitted reader overlay, not a changed C or a corrected archive claim.
Before delivery, explicitly disposition that known guidance gap or commit/review
the correction and select a successor source. Reuse unaffected runtime and lock
evidence; do not begin another qualification campaign for prose changes.

## Sixteen-subject and partial-publication proposal

`sixteen-subject-proposal.json` records the sixteen actual C archive filenames,
sizes and SHA-256 digests, with normal/optional publication dependencies and
all D1–D6 Pending. It is unsigned local preparation. The original nine-subject
[formal proposal](YT-096-FORMAL-TRUST-PROPOSAL.md) is not silently amended or
accepted for sixteen. An eventual manifest/provenance must bind the separately
approved source/workflow, all selected archive digests and exact inventory scope.

Dependency layers, ignoring dev edges, are:

1. Actions, Core, Rate-limit, Reactive, Secrets, Signing, wgpu.
2. Config, DOM, keyed DOM, HTML.
3. Axum adapter.
4. Request-ID, Memory Session, Redis Session.
5. Auth, after Memory Session for its optional bridge.

Publication is a dependency-ordered series, not an atomic registry transaction.
If a later authorized upload/resolution fails, stop the cohort, retain successful
subject digests and all errors, and do not claim the coordinated release complete.
Do not overwrite a version, repoint a tag, repackage approved bytes or silently
change dependencies. Resolve resume/new-version/yank decisions under separate
owner authority, with fresh bindings where inputs change. Actual downloaded
registry resolution and rights remain YT-020 gates, not this directory-source proof.

## Open acceptance conditions and next step

YT-205 remains **In progress** while these existing conditions are unmet:

- Obtain required hosted CI for the ultimately delivered exact source. Existing
  check names/protection are unchanged; local preflight and old green CI cannot
  satisfy this. No Push is authorized in this task.
- Complete the selected source-bound technical rebuild path. The existing
  private-rebuild workflow/xtask still has nine fixed subjects, not this sixteen
  set. No new builder, second policy, workflow mutation or attestation is invented.
  Its minimal cohort extension and actual hosted execution need a separately
  reviewable authorized slice; formal builder/trust acceptance stays YT-096.
- Close genuinely affected selected native-platform dependency evidence after
  admitting the exact source/lock to the prepared environment. The current1.53.3
  Ubuntu24 integration case below passes; MSVC remains Not verified. Do not replay
  unrelated old suites or infer Windows execution from cross-target metadata.
- Disposition C's two package-guide inconsistencies versus the corrected draft
  before claiming the final reader/package source is aligned.

The next unblockable work is review of this concrete packet and an owner decision
on delivering the source/draft and the minimal existing rebuild extension.
Do not mark this item Done merely because local checks or its delivery review
pass. YT-096/D1–D6 and YT-020 remain separate formal/publication gates.

## Evidence and preserved failures

Evidence: `target/yetlet-yt205-consolidation-20261027-gj5ya_34/evidence/`.
Key records are contract.json, source-before.json, package-commands.json,
candidate-package-inventory.json, reader-commands.json, changed-inventory-results.json,
advisory-and-source-reconciliation.json, rustdoc-output-binding.json,
reader-source-inventory.json and sixteen-subject-proposal.json. C archives remain
under the sibling `archives/`; old evidence is never overwritten.

The first archive collection stops because the accepted old vendor pool lacks
p2560.14.0 required by the later application-owned OIDC patch during workspace
resolution. Ordinary current-root offline locked Cargo vendor corrects only the
owned pool; no passed package command is replayed and no dependency changes.
The first inventory metadata command stops because an extracted package beneath
the checkout inherits its parent workspace. A single harness correction moves
the unchanged extracted package directory outside the checkout; its failed logs
remain and no passed inventory command is replayed. These are preparation
failures, not failing product behavior or reasons for new runtime code.

All new checks use bounded existing Cargo/inventory mechanisms. New changes
remain unstaged; no Commit, Push or Merge follows the already authorized preceding
28-file commit.

## Independent checkpoint acceptance — 2026-10-27

Reviewer `yt205_source_package_evidence_checkpoint_review_20261027` returns
**PASS, zero issues and zero packet repair requirements**. It verifies10 document
and185 evidence bindings,1179 protected tracked inputs, all16 actual C archive
digests/payloads/manifests/locks and clean VCS identities, the ten new versus six
retained package checks, consumer/rustdoc results,14 new plus28 retained
inventories, audit containment, source-scan limits and46 reader-source hashes.
The proposal/order/partial-publication handling and prior failures are accepted.

The reviewer explicitly concludes **item_can_be_done:false** for the remaining
original acceptance conditions above. This PASS accepts a truthful checkpoint;
YT-205 stays In progress. Read-only review replays no commands or external work.
Native trace verifies the registered reviewer/model and effective effort high;
the shared resolver reports enforced with no conflict/degradation. Review input,
filtered runtime observation and result remain separate evidence records.

After review, only this closeout and task-navigation metadata change. The owned
vendor/Cargo-home/build caches and external extracted-package directory are
removed; sixteen C archives, inert bootstrap/consumer inputs, raw logs, SBOMs
and generated rustdoc ZIP remain. At this initial checkpoint no container,
listener, cloud resource or credential had been created; previous artifacts and
services are untouched.

## Once-only current-lock Ubuntu24 continuation

The selected GNU dependency gap is an unmet original YT-205 condition, so the
owner's missing-check authority permits one new current-input case using the
existing YT-202 Ubuntu24/OpenSSL3.5.9 isolation recipe. This is the same candidate
and task, without refreezing, new infrastructure or repeated old suites.
`gnu-delta-continuation.json` records the condition, scope and four-CPU/four-GiB
limit before execution. The earlier checkpoint review remains historical.

The old task image was already removed. The unchanged accepted Dockerfile/base
and pinned OpenSSL source recreate one uniquely labeled task image using the
existing cache. A separate owned directory source contains the sixteen unchanged
C archive payloads and authenticated Tokio1.53.3 material; old pools are untouched.
One network-disabled, read-only-container integration run uses the existing
consumer program and current lock, Auth/session, Config/secret-provider and
Redis/tls. It exits0 and prints the expected `packaged consumer OK` marker.
Rust/Cargo1.99.0, Ubuntu24.04.5, glibc2.39 and OpenSSL3.5.9 are actually observed;
all consumer source/config/lock hashes remain unchanged. No earlier successful
run or feature/workload suite is replayed.

This closes the selected current-lock GNU compile/link/API control, without a
new live provider, TLS transport, physical Ubuntu VM or runtime-capacity claim.
The container shares its host kernel; MSVC remains a separate unexecuted cell.
The owned image/container, vendor, Cargo home and build directory are removed;
shared Docker build cache is not pruned. Raw build/native outputs, exact argv,
image identity, input hashes and successful cleanup remain in gnu-image-build.json,
gnu24-current-lock.json and gnu24-cleanup.json. Directory-source preparation is
supported by the parent-observed extraction/checksum command and its successful
execution; no separate post-copy file snapshot is claimed.

The same independent reviewer returns a separate **GNU delta PASS, zero issues
and zero packet repair requirements**. It verifies10 document/12 additional
evidence bindings, the unchanged original185 records, retained16 C archive
digests, consumer inputs and Tokio checksum, observed environment/compile/native
marker, accepted Docker recipe and cleanup. The parent preparation observation
is retained separately in gnu-preparation-observation.json; this is local
controlled-fixture evidence, not registry or hostile concurrent-writer assurance.
The filtered trace still matches reviewer/model/high and the shared resolver
remains enforced without conflict/degradation. The reviewer again concludes
item_can_be_done:false: exact-source CI/sixteen-package rebuild, current-lock MSVC
evidence and final reader-source disposition remain open. No accepted workload,
earlier review verdict, source candidate or formal decision is replaced.
