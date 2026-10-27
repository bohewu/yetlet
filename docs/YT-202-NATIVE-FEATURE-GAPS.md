# YT-202 — Native platform and opt-in feature evidence

Date: 2026-10-27 (Asia/Taipei). Status: **Done at the selected native/MSRV/feature
scope after final independent Reviewer PASS, zero issues/required followups.** The historical
GNU and inventory reviews retain their bounded acceptance. No Commit, Push,
Merge, publication or final release-candidate selection occurs in this continuation.

## Current native coverage — 2026-10-27

| Selected coverage | Accepted execution / current review state |
|---|---|
| Sixteen normal package checks and Ubuntu 26 consumers | Retained YT-201 evidence; not replayed |
| Four default graphs and Ubuntu 24/OpenSSL 3.5.9 consumers | Retained GNU execution and independent partial review; 32 evidence hashes unchanged |
| Windows MSVC default consumer | PASS, fourteen packaged-source dependencies, native exit 0, `packaged consumer OK` |
| Windows MSVC integrated consumer | PASS, Auth `session` and Config `secret-provider`, missing Memory-layer refusal and CPU-only wgpu; exit 0, same marker |
| Owned Windows cleanup | PASS, clone/build/vendor/cache/bootstrap/consumer directories absent; inert evidence retained |
| Whole YT-202 acceptance | Final independent Reviewer PASS, zero issues/required followups, at the selected native/MSRV/feature scope |

The new MSVC execution is detailed below. The earlier checkpoints preserve
their point-in-time missing-toolchain, artifact and Git-delivery failures.

## Inventory checkpoint before Git-source authorization — 2026-10-27

The owner requests only the remaining YT-202 inventory and MSVC checks, reusing
accepted evidence, with independent review before completion and no Commit,
Push or Merge. The delivery branch now ends at
`4e5ef64ad0b754856200b63455a347cf29b15c01`; PR #12 is open and unmerged.
Its OIDC application dependency patch does not change any path under `crates/`
relative to the accepted YT-201 source. The fixed package/vendor payload remains
bound to that original source; current CI does not relabel it as a different
candidate's Windows execution.

| Coverage | Current disposition |
|---|---|
| Sixteen package checks and Ubuntu 26 default/integrated consumers | Retain YT-201; no replay |
| Four default graphs and Ubuntu 24/OpenSSL 3.5.9 consumers | Retain accepted YT-202 GNU evidence; all 32 retained evidence hashes match |
| Windows MSVC prerequisites | Read-only observation confirms Rust/Cargo 1.99.0 MSVC, VC Tools 14.29/14.44 and SDK 19041/22621/26100; no installation |
| Windows default external consumer | Not run; fourteen selected package dependencies, native MSVC |
| Windows integrated external consumer | Not run; Auth `session`, Config `secret-provider`, missing Memory-layer refusal and CPU-only wgpu |
| Signing/Redis on Windows, browser/GPU/provider/cloud workloads | Not selected; no added platform promise or replay |

The pre-existing payload and consumer hashes match their retained bindings.
Its `windows-continuation-contract.json` removes only the unselected Signing/
Redis Windows dependencies and their references from the accepted consumer.
No package runtime, API, lock, archive or consumer source was changed here.

The previous official `transfer_project_artifact` source read returned
`source_read_failed / policy_rejected` for the recorded `target/...` payload.
No unchanged retry, relocation, encoding or alternate transport is attempted.
Current Runner health/config validation succeeds, but does not establish artifact
admission. The official `read_tool_trace` diagnostic requires `admin`; the current
connection returns `UNAUTHORIZED` and requests reauthentication. No exact policy
field or safe configuration change has yet been established. Administrator
diagnostic access or the actual refusal detail is required before selecting an
admitted route. Toolchain readiness alone cannot close either execution cell.

Current observations and retained-input checks are recorded in
`evidence/windows-continuation-20261027.json` under the existing evidence root.
The historical prerequisite, Git-delivery failures and accepted GNU review below
remain intact. No native MSVC product execution or whole-item PASS is claimed.

## Owner-authorized Git-source continuation — 2026-10-27

After the inventory above, the owner explicitly authorized `git clone`, with
mint-game available as an alternative Runner. The prepared MSVC Runner can
read the already-pushed GitHub repository. Its new task-owned clone selects
`74f53fe454c26a5ea425e66589082a344a90c381` on a local attached test branch;
the observed complete tree is `2faa5d55cb5b251bac6242d9d77fafb26835eb45`, exactly
the accepted YT-201 source. The rejected generated ZIP is not read or transferred
through this route. The earlier transfer and admin-diagnostic failures remain
historical; admin reauthentication is not required for this authorized Git read.

Preparation follows the existing YT-201 Cargo archive/directory-source pattern:
collect fourteen bootstrap archives without verification, vendor locked upstream
sources and use a separate native consumer. Bootstrap `--no-verify` and
`--exclude-lockfile` collect normalized source only; they are not counted as
passing package checks or used for Git/hook bypass. The new consumer derives the
same selected public-API cases from committed source using fresh synthetic data.
It does not claim byte identity to the old blocked consumer. Record its generated
lock, package-source identity and actual Windows tools before the two missing
offline/locked MSVC executions. No prior package/GNU/provider/browser suite is
replayed. Source acquisition and input preparation alone do not close YT-202.

The strategy delta, authority and cleanup boundary are in
`evidence/windows-git-source-contract-20261027.json`. Native product outcomes
were pending at that preparation checkpoint; the completed execution follows.

## New MSVC results and preserved preparation failure — 2026-10-27

The same prepared Runner, `ncut-bohe-244143825ae3488e`, executes native
`x86_64-pc-windows-msvc` Rust/Cargo 1.99.0 under Windows 11 version 10.0.26300.0.
Actual developer-environment observation selects VC Tools 14.44.35207 and SDK
10.0.26100.0. No Rust, SDK or system installation/configuration change is made.
The existing `localtasks` workspace/services and unrelated Yetlet checkout stay
unchanged; only the new task-owned clone and verification files are used.

Fourteen Cargo bootstrap archives supply normalized manifests to the existing
directory-source convention; they are source collection, not repeated package
verification. The Windows consumer uses registry version requirements and that
directory source, with no checkout path dependencies or patches. Its 100 external
lock identities all come from the committed source's existing locks; none is
upgraded. Signing and Redis Windows support remain unselected.

Initial offline consumer preparation exits 101 because the fresh fixture's
`futures` meta-crate is absent from this committed lock/vendor pool. No native
product test has started. One revision-fenced fixture correction uses the already
locked Tokio 1.53.2 current-thread runtime for the same Secrets async API check.
Preparation then succeeds; the original error remains in
`windows-git/preparation-first-failure.stderr`. This is an input-preparation
failure, not a framework regression. The developer-environment observation's
initial literal-variable output is also retained; a corrected environment-only
read captures actual values without replaying either passing native run.

Both commands use `cargo run --offline --locked -j 4`, explicit consumer manifest,
directory-source config, task-local Cargo home/build paths, Rust 1.99 MSVC and
`--target x86_64-pc-windows-msvc`; the second adds `--features integrations`.
Exact argv/shell commands and native results are in `windows-git/msvc-results.json`.
Default compiles all fourteen selected libraries and exits 0 in 74.542 seconds.
Integrated exits 0 in 5.254 seconds. Each prints only `packaged consumer OK`.
The integrated cases preserve the missing-Memory-layer 500 refusal, protected
`SecretString`/`SecretBinding` type compatibility and redacted binding Debug.
The wgpu dependency remains CPU-only; no adapter/browser/provider network starts.

All four consumer/config/lock/source input hashes match before and after both
runs. The 26 packaged runtime files and fourteen original manifests are rehashed
after execution and remain unchanged. Windows Git has `core.autocrlf=true`:
all forty hashes exactly match the corresponding committed blobs after CRLF
checkout conversion, with zero unexplained differences. This is Git-equivalent
Windows source evidence, not byte-identical Linux archives or a new final release
archive identity. The generic WebCodex source-freshness label for the non-Git host
project remains `unproven`; explicit package/consumer hash records provide the
task-specific input binding, without relabeling that automatic observation.

The 24-file evidence ZIP is transferred through the admitted artifact route with
matching SHA-256 `5c5c784cfcffb4de4a8ba9830853184fee44228b8da52eefcc068bceddab93f7`.
After its receipt, all six owned source/build/vendor/cache/bootstrap/consumer
directories are removed. `windows-git/cleanup.json` confirms their absence and
forty post-run source matches; only inert logs/evidence remain. No old product
suite, package verification, GPU/IdP/Valkey/Azure/benchmark/security scan or CI is
rerun. Final independent acceptance is recorded below.

## Final independent acceptance — 2026-10-27

Reviewer `/root/yt202_msvc_final_evidence_review_20261027` returns PASS with zero
issues and zero required followups. It verifies four documentation hashes,
thirty new evidence hashes, thirty-two retained GNU evidence hashes, twenty-four
ZIP members, forty committed-to-CRLF package-file comparisons and all one hundred
external lock identities. The original submitted execution request in
`evidence/windows-git-execution-request.json` explicitly selects the installed
Rust 1.99 MSVC toolchain for both child executions. This request evidence remains
distinct from the recorded runtime output and developer-tool observations.
Source, selected features, positive/negative cases, native results and cleanup
support the selected YT-202 acceptance; no remaining selected execution gap is
found. The generic source-freshness limitation and all scope limits above remain.

`evidence/review-msvc-final-binding.json` and
`evidence/review-msvc-final-result.json` retain the reviewed inputs/result and
verified native role/model/effort evidence. Review is read-only and reruns no
check. Only completion metadata, navigation summaries and this acceptance record
change afterward; runtime/fixture/evidence inputs stay fixed. Documentation and
whitespace checks pass for the final text. YT-203 is the next registered item;
YT-204/205, YT-096/D1–D6 and YT-020 keep their separate gates. No Commit, Push,
Merge, signing or publication is performed.

## Authority and fixed inputs

The owner authorized YT-201 Commit/Push followed by YT-202 under the
[reviewed per-package plan](YT-200-SIXTEEN-PACKAGE-RELEASE-PLAN.md#proposed-per-package-platform-and-feature-scope).
YT-201 was committed as `8fd8751c3a505ad4811e0d3c83419e468565473a`, tree
`2faa5d55cb5b251bac6242d9d77fafb26835eb45`, after strict/builtin staged Guard
exit 0 with zero findings. This immutable reviewed source snapshot supplies the
qualification payloads; no new runtime or manifest change belongs to YT-202.

The separately owner-approved integration commit
`74f53fe454c26a5ea425e66589082a344a90c381` has the same complete tree, parent
`71b5f084fdeedd3feade202740077fc199f950ae` from origin/master. The original
branch/tip remain preserved. Commit/source equality is explicit; future delivery
history decisions cannot relabel a different run as this snapshot's execution.
No final signing/workflow identity is selected. The original ce56e08 candidate,
YT-096/D1–D6 Pending and YT-020 Blocked retain their historical boundaries.

Before new product execution, the task's excluded contract selected: retain
YT-201 archive/consumer proof, reconcile source/locks/features, run only missing
GNU userland and configured MSVC cells, and inspect default dependency isolation.
No browser/Wasm compilation, GPU runtime, provider/IdP/Azure, capacity/benchmark,
advisory/security scan or hosted CI. Independent review is required before any
completion claim; new documentation remains uncommitted/unpushed.

## Retained versus new coverage

- [YT-201](YT-201-PACKAGE-PREPARATION.md) retains sixteen normal package checks,
  source/manifest/license/archive bindings and two native external-consumer runs
  on Ubuntu 26.04.1 with host OpenSSL 3.5.5. Those are not replayed locally.
- [YT-110](YT-110-DOM-CAPABILITIES.md), [naming](NAMING.md), Auth/Session/rate-limit,
  SecretProvider rotation/Windows and YT-184/185 retain their exact source and
  environment bounds. Metadata-only version/publish changes do not erase prior
  functional evidence or create a reason to rerun live Session/IdP/Valkey work.
- The new GNU checks reuse the unchanged sixteen-package consumer and verified
  directory sources from YT-201, with native default and selected integration
  features, in a previously untested Ubuntu 24.04/OpenSSL 3.5.9 environment.
  This is new platform evidence, not another Ubuntu 26 preflight run.
- Windows GNU tool availability is only a diagnostic. It cannot fill selected
  `x86_64-pc-windows-msvc` cells or transfer old host qualification to this source.

## New GNU and dependency-graph results

Four exact-source-export `cargo tree --offline --locked` normal/build graphs
PASS: Core has one unique node, Axum adapter 50, Todo browser 34 and Search island
32. Native targets are GNU x86_64; browser graph selection uses wasm32 but does
not invoke a Wasm compiler/browser. No graph contains unselected Yetlet Auth,
Session, remote Session, Config, Secrets, Signing, Rate-limit or wgpu, or external
wgpu/Redis/Moka. Upstream HTTP tracing internals are not a mandatory application
subscriber or the app-owned tracing option; do not claim all tracing-related
upstream crates are absent. The source export is an immutable Git archive, not
a detached worktree or edited consumer workspace.

The GNU environment uses the recorded Ubuntu 24.04 amd64 base digest, existing
Rust/Cargo 1.99.0 toolchain mounted read-only, glibc 2.39 and a task-local static
OpenSSL 3.5.9 build. The source archive's SHA-256 matches the accepted YT-093
recipe: `603f5602e2eef00d77fbd429d34dcd5822bb301757a1bc9cdb24c670f1eb859a`.
Compilation dependencies are installed only inside the owned test image. No
host toolchain/library, service or configuration changes. A container shares
the Docker host kernel; this is Ubuntu 24.04 userland evidence, not a newly
booted Ubuntu 24.04 VM or arbitrary-kernel qualification.

| New check | Result and boundary |
|---|---|
| Native GNU default external consumer | PASS, native exit 0; all sixteen 0.1.0 package dependencies compile/link and the API smoke exits with only `packaged consumer OK` |
| Native GNU integrated-feature consumer | PASS, native exit 0; Auth `session`, Config `secret-provider`, Redis provider `tls`, Memory missing-layer refusal and shared protected-field types; no provider network operation |
| Input/tool/environment observation | Rust/Cargo 1.99.0, OpenSSL 3.5.9 and Ubuntu 24.04/glibc 2.39 observed; readonly archived consumer/vendor inputs, source replacements and exact commands retained |
| Owned cleanup | All labeled containers absent, named test image and newly pulled base tag removed; inert source/build evidence retained, shared builder cache not pruned |

Tests use four jobs/CPUs, memory 4 GiB, PID limit 512, read-only container root,
read-only toolchain/consumer/vendor mounts, task-owned output/tmp and no network
or published ports. Both are ordinary native Cargo commands, not old suites or
new test infrastructure. TLS is compilation/API evidence only; no new transport
certificate/ACL/restart semantics are inferred. Package directory-source null
registry checksums and VCS metadata limits from YT-201 remain explicit; these
are not actual crates.io downloads or a final sixteen-archive reproducible release.

## Windows MSVC prerequisite gap — historical 2026-10-26

The historical Windows Runner/project identity is no longer registered. A
read-only fleet lookup found the current online native Windows Runner and its
existing Yetlet project. Its unrelated checkout stays at
`3128d23591feee15ce976d7b83a56dcd587048d7`, with the preexisting untracked
artifacts directory preserved. No reset/checkout/source modification occurred.

Observed Windows 11 version 10.0.26300.0 / PowerShell 7.6.7 has configured
Rust 1.88 GNU and 1.99 GNU toolchains. The configured MSVC toolchain is absent,
vswhere reports no instance with the VC Tools x86/x64 component, and the known
historical tool paths and standard Windows SDK library path are absent. A GNU
Cargo 1.99 version observation is not native MSVC compilation. No Windows product
test, build, installation, SDK/workload modification or environment retargeting
was performed. Old SecretProvider/native MSVC results remain historical proof.

Required next input: provide a configured Rust 1.99 MSVC plus VC++/Windows SDK
environment, or explicitly authorize its setup, then execute only the uncovered
selected native Windows cells. Signing/Redis Windows runtime was never selected;
wgpu on Windows is CPU-only. Owner setup/defer decision is pending. This is an
execution prerequisite, not a demonstrated source defect or a new framework
feature/operations item. Keep YT-202 Blocked until the selected MSVC gate closes
or the owner explicitly revises support scope; no silent GNU substitution.

## Separate Git delivery blocker and preserved failures — historical 2026-10-26

No Push has executed. The original new branch's full-history strict/builtin
scan stopped at the 2,000 changed-file-operation cap (103 commits). The approved
single integration commit preserves the tree and passes staged strict/builtin
with nine nonblocking generic Windows-path warnings. Its full new-branch range
scan covers 22 commits / 1,680 file operations and stops with 13 blocking personal/
current-machine paths in two commits already inherited from origin/master.
The new integration commit itself has no blocking finding. No smaller range,
audit downgrade, hook change or force push was used. A separately proposed
independent-root delivery branch needs the owner's new decision; this task does
not authorize it or rewrite history. Git delivery failure is not a product test
failure and does not justify replaying passing native checks.

Initial `tool_manifest` discovery used a prose intent instead of its closed enum;
one corrected discovery call selected the documented value. No process/Job was
created by the rejected read. The known SSH alias could not resolve; it was not
retried. Authorized Runner discovery then located the current Windows project.
These operational observations are retained, not relabeled as tests or source
failures. GNU environment creation and both product runs succeeded without a
product repair or test replay.

## Evidence and acceptance state — historical 2026-10-26

Evidence is retained under `target/yetlet-yt202-native-20261026-s931pywk/evidence/`:
fixed contract/source export, four graph outputs, source/image/tool identities,
GNU commands/native exits and cleanup, plus the Windows prerequisite summary.
Prior YT-201 package and Git Guard artifacts remain unchanged in its evidence
directory. No new release/security result replaces earlier PASS/FAIL records.

Run documentation/link/whitespace checks for this new report and its three
Backlog/Roadmap/Release Readiness summaries. Independent evidence review may
accept this bounded GNU delivery while the whole item remains Blocked. It must
not turn Windows Not verified, Push blocked or deferred YT-203–205 into PASS.

Documentation checks PASS: 75 fixed required documents / 205 backlog items,
tracked whitespace, direct four-document local links/fences/EOF and the new
report's no-index whitespace check (expected difference exit 1, no diagnostics).
Independent Reviewer `/root/yt202_gnu_partial_evidence_review_20261026` returns
PASS at bounded GNU/evidence-documentation scope, zero issues/required followups.
It verified four source and 32 evidence hashes, unchanged consumer inputs,
sixteen archive bindings, graphs, native results and cleanup. HEAD/branch/status
and staged-zero state stayed stable. Native trace matches the registered role,
saved model and resolver-selected effective effort; no conflict/degradation.
No review-time product check, scan, service or mutation was performed. Only
this closeout and three task/navigation summaries change after that review.
The selected MSVC prerequisite/execution gate remains unmet and YT-202 stays
Blocked. Owner Git-delivery and Windows setup/support decisions are still pending;
no new Commit/Push or whole-item completion is inferred.
