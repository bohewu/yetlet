# YT-200 — Sixteen-package release feasibility and verification plan

Date: 2026-10-26 (Asia/Taipei). Status: **Done at planning-review scope after
independent Reviewer PASS, zero issues and required followups.** The follow-up
items below are Todo, not executed.

## Authority and outcome

The owner requests another review of releasing the sixteen implemented library
packages together and registration of missing verification work. This authorizes
source/evidence review and planning documents, not manifest/runtime changes,
product qualification, resource allocation, a replacement candidate, Commit,
Push, signing or publication. Independent review gates this planning delivery.

**Recommendation: prepare all sixteen for one coordinated experimental release.**
There is no demonstrated architecture-level reason to exclude a package merely
because it is opt-in or currently nonpublishable. Co-release means a coordinated
versioned set with dependency-ordered package publication, not an atomic upload,
uniform platform support, Stable API promotion or production certification.
Preparation must first close the concrete metadata and evidence gaps below.

The review baseline is `64a6d1c408a0b7a6234620576b4d4820394d28a4`, tree
`fb86350021309e097e99974e0d4797429470f0c0`, on
`docs/yt113-123-opt-in-capability-plan`, plus the four accepted, uncommitted
[YT-199](YT-199-CURRENT-DELIVERY-READINESS.md) documentation paths. Preserve those
changes and their review history. Baseline 64a6d1c cannot identify a later release
with changed manifests. After separately authorized preparation/review and a
guarded authorized commit, explicitly select that later immutable source C.
No moving HEAD, dirty overlay or old signature substitutes for C.

The original nine-package `ce56e0814e5423782440acd2c959ca777d572cd4` candidate
and its archives/reviews remain intact. YT-096 stays In progress, D1–D6 Pending,
and YT-020 Blocked. This plan supersedes only YT-199's recommendation to begin
future preparation with nine; its source observations and accepted evidence
remain unchanged. The seven are being considered for preparation together,
not retrospectively added to any historical release or trusted statement.

## Exact proposed cohort and version boundary

Original nine: `yetlet-core`, `yetlet-html`, `yetlet-server-axum`, `yetlet-dom`,
`yetlet-dom-keyed`, `yetlet-reactive`, `yetlet-actions`, `yetlet-request-id`,
`yetlet-session`.

Additional seven: `yetlet-auth`, `yetlet-rate-limit`, `yetlet-config`,
`yetlet-secrets`, `yetlet-signing`, `yetlet-session-redis`, `yetlet-wgpu`.

Recommend experimental **0.1.0 for all sixteen** under the existing coordinated
train, retaining Rust MSRV 1.99 and toolchain pin 1.99.0. This is a preparation
recommendation; versions/publish fields have not been changed. Config, Secrets
and Redis Session are currently 0.0.0. That is not inherently an invalid Cargo
version; agreeing a deliberate first public version and compatible dependency
requirements is the missing decision. Independent workspace/lock boundaries
can remain independent while their selected release versions agree.

CLI, examples, benchmark fixtures, common loader assets and the physics Lab are
not additional registry packages in this cohort. Their selected source-bound
consumer evidence may support a library without making the application part of
its public support promise. Audit, data/output Cache, OIDC/Bearer/Cookies and
Tracing research do not create unimplemented packages. Existing app-owned
fencing/tracing/cache options remain app-owned; no extraction is required.

## Concrete manifest and archive gaps

The review inspected all sixteen package manifests, feature declarations,
local dependencies and package-local guide/license presence. All seven additions
currently declare `publish = false`; the original nine omit that field. Future
preparation can follow the existing original-package convention when authorized.
Publishing availability and dependency/runtime opt-in remain independent.

| Observed input | Consequence and smallest preparation |
|---|---|
| Auth's optional Session dependency, Config's optional Secrets dependency and Redis Session's Axum adapter dependency have only `path` | Add compatible registry versions while retaining local paths. Optional normal dependencies still need a publishable dependency closure. These three edges are actual registry-packaging gaps |
| Other local-only edges are dev dependencies: Auth/Config/Secrets test consumers and Rate-limit's Settings example | These are not the same normal-dependency publication blocker. Decide what the archive retains for tests/examples and verify that retained consumers are self-contained. Config/Secrets dev references do not establish a normal runtime cycle |
| Three 0.0.0 packages, mixed inherited/literal versions, seven disabled publish fields | Align the separately selected first-release version/requirements and explicit publication eligibility, without merging workspaces or changing default feature/runtime behavior |
| None of the seven has package-local dual-license files; Config/Signing lack a local README; Signing also lacks description/repository metadata | Inspect actual archive inclusion. Follow the original nine's dual-license-text practice and provide bounded public package guidance/metadata. Absence of a local file is not by itself a legal violation or observed Cargo rejection; distributed contents must satisfy the existing release contract |
| Existing consumers use workspace/path inputs; current manifests/locks differ from ce56e08 | Prove archives, normalized manifests and external dependency resolution from selected package inputs. Local builds and package listings alone cannot establish a clean packaged consumer or actual registry consumption |

Cargo documents [path plus registry versions](https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html#multiple-locations)
and [development-dependency publication](https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html#development-dependencies).
Use those distinct rules rather than mechanically treating every path as a
normal dependency. Actual crates.io names/account rights, uploads/downloads and
real registry resolution remain YT-020 operations under separate authorization.

Normal dependency order is acyclic: publish independent Core/Actions/Reactive,
Secrets, Rate-limit, Signing and wgpu roots before their dependents; Core before
HTML/DOM/keyed DOM; Core+HTML before Axum adapter; adapter before Request-ID,
Memory Session and Redis Session; Memory Session before Auth's `session`
selection; Secrets before Config's `secret-provider` selection. This is planning,
not a publish script. YT-201 must verify the final actual normal/optional graph.

## Proposed per-package platform and feature scope

This table proposes the minimum support/qualification claims for the future set.
Every selected cell needs source-bound evidence or an explicit narrower-scope
decision before its claim is made. It is not a report that these cells pass at C.
GNU means Ubuntu 24.04-family x86_64 GNU; MSVC means Windows x64 MSVC. The current
Ubuntu 26.04 host is a preflight environment, not an Ubuntu 24.04 substitute.

| Packages | Proposed selected cells | Important limit / retained adjacent evidence |
|---|---|---|
| Core, Actions, Reactive | GNU/MSVC native; wasm32-unknown-unknown | Actions native construction does not implement native browser fetch; retain candidate/consumer evidence only at matching inputs |
| HTML, Axum adapter, Request-ID, Memory Session | GNU/MSVC native defaults | HTML/adapter YT-110/147/149/154 changes need reconciliation; Session stays synchronous/process-local, no distributed identity or persistence guarantee |
| DOM, keyed DOM | GNU/MSVC static policy; wasm32 and selected Linux desktop Chrome/Firefox DOM workflows | Finite controls/navigation, CSP and lifecycle only. Existing YT-110 evidence and older candidate evidence have different source scopes; exact browser/build inputs matter |
| Auth | GNU/MSVC default and `session` | Native only; required/optional identity and Memory bridge, not business authorization, generic OIDC or all provider combinations |
| Rate-limit | GNU/MSVC default | Per-process Budget; Settings example is dev/application integration, no distributed quota or default installation |
| Config, Secrets | GNU/MSVC; Config default and `secret-provider` | Native only. Retain prior Windows/rotation and later SecretBinding/naming evidence separately. External Azure/provider CLI, filesystem policy and live rotation are not new platform-wide promises |
| Signing | GNU native with explicitly documented OpenSSL recipe | Existing YT-093 uses isolated OpenSSL 3.5.9 LTS and Rust bindings, not arbitrary system OpenSSL. New MSVC/macOS/Wasm support is not selected. Native library build prerequisites must accompany the package |
| Redis-protocol Session | GNU native default and `tls`, selected Valkey reference | Retain TLS/ACL/uncertain-outcome and YT-184 cross-host/restart evidence at its own inputs. MSVC/live Windows provider support, HA/failover and arbitrary restore are not selected |
| wgpu | GNU/MSVC and wasm32 CPU-only defaults; wasm32 explicit `browser` build; existing selected Firefox GPU configuration | CPU validation is separate from GPU execution. YT-081's Firefox158/X11/Intel-Vulkan crate consumer evidence is relevant; YT-198's Firefox154/Weston Lab uses a different private renderer and cannot qualify this crate |

Chrome/Firefox selection for DOM does not imply Chrome WebGPU support. A default
Chrome page opening on this host did not demonstrate a successful 3D Run, and
there is no blanket Chrome-requires-Weston claim. For wgpu, reuse/reconcile its
[recorded support boundary](../crates/yetlet-wgpu/SUPPORT.md) and
[YT-081 refinement](YT-081-WGPU-API-REFINEMENT.md). No new GPU/OS matrix, physical
device, IME/AT/mobile/WebKit qualification is selected here. Existing waivers and
Not verified cells remain visible; public support cannot be inferred from Rust
compile success, a manifest version or a successful different application.

## Verification work registered for later execution

The missing conditions below come from the owner's co-release review request and
existing [supply-chain](RELEASE-SUPPLY-CHAIN.md), [compatibility](COMPATIBILITY.md),
[opt-in](adr/0005-opt-in-capability-modules.md) and
[publication-scope](DOCUMENTATION-SCOPE.md) contracts. A plan cannot grant itself
execution authority. Register YT-201–205 Todo; no product checks run in YT-200.
Each item requires independent Reviewer PASS before Done.

| Item / dependencies | Concrete gap and minimum acceptance | Evidence to reuse / scope stop |
|---|---|---|
| **YT-201 — Publication metadata and packaged consumers**; after YT-200 | Prepare the seven manifests/versioned normal edges, license texts/public guides and final normal/optional publication order; reconcile original-nine inputs. Use existing Cargo packaging and isolated directory-source/consumer patterns to verify exact archive contents, normalized metadata and external native/default+selected-feature consumer resolution. Preserve local paths and independent workspaces. Obtain authorization for any later commit and explicit immutable C selection before final candidate qualification | Old nine-archive proof and wgpu's earlier package evidence stay source-bound. No custom registry, validator framework, runtime redesign or actual publication. Enabling publish eligibility is not upload authority; changing archive bytes is not automatically a reason to replay unchanged product behavior tests |
| **YT-202 — Native, MSRV and opt-in feature qualification gaps**; after YT-201 | For selected GNU/MSVC cells, reconcile final package/lock/features and existing evidence, then execute only uncovered compile/API/negative cases. Include Auth default/session, Config default/secret-provider, Redis default/tls on GNU, Signing's documented native recipe, and wgpu CPU-only MSVC scope. Confirm base SSR and selected browser graphs exclude server/provider/GPU dependencies when unselected | Reuse YT-110, 114/115/117, naming tests, SecretProvider Windows/rotation, YT-184/185 and selected native proofs only where inputs justify reuse. Do not replay live IdP/Azure/provider restart/capacity experiments or force unselected Signing/Redis Windows support. Configuration choices do not install services automatically |
| **YT-203 — Wasm, DOM and optional GPU package evidence gaps**; after YT-201 | Reconcile the five original wasm packages and optional wgpu default/browser build inputs; cover only missing changed DOM controls/navigation/CSP/lifecycle and selected consumer/browser artifact bindings. Keep server dependencies out of the browser. Use the actual wgpu consumer for its GPU cell, with explicit fallback and lifecycle limits | Retain source-bound YT-110, YT-196/Tasks BFCache and YT-081 crate observations. Check reuse first; no new GPU run is mandatory solely because README/license/publish metadata changed. Any genuinely new runtime asset/feature gap needs only its affected selected flow; Lab success is not proxy evidence. No broad browser matrix, new scene or performance campaign |
| **YT-204 — Selected dependency inventory and security gaps**; after YT-201 | Reconcile target/feature-specific metadata/SBOM/license/source/advisory coverage across all sixteen; keep dev over-inclusion and native OpenSSL/sysroot/system limits explicit. Compare actual scan input bindings and assess uncovered selected source/trust-boundary changes using the existing security workflow; retain findings/remediation/coverage and source identities | Old YT-067 pre-patch scan/fix, six-file Auth scan, YT-096 delta review and later item reviews remain distinct. No automatic zero-finding/full-current-scan claim, paid assessment, blanket policy exception or dependency upgrade. Recheck only the missing/stale selected coverage; a current advisory assessment has a new database/time identity, not a replayed historical PASS |
| **YT-205 — Sixteen-package evidence and reader-release consolidation**; after YT-202, YT-203, YT-204 | Join selected C/package/lock/features/platform/browser/security records, reuse versus new runs, remaining unknowns and reviewer outcomes. Close missing exact-source technical rebuild/required-CI evidence using existing mechanisms only under separately authorized execution. Verify versioned public package guidance and appropriate rustdoc target/features; select exact reader pages/assets and inspect publication contents/links. Record updated sixteen-subject proposal and partial-publication handling for later owner decisions | No automatic repeat of prior accepted workloads or CI weakening. Required CI for the eventually delivered candidate retains its own contract; old-source green CI is not current evidence. Actual hosted execution, signing/log disclosure, registry proof and promotion stay in YT-096/YT-020 under explicit authority. No parallel trust policy or second registry-release task is created |

Execution order is YT-201, then the bounded YT-202/203/204 scopes, then YT-205.
Dependencies are not permission to spawn parallel workers or start external
resources. Start each item's source reconciliation before choosing commands.
An affected unsupported tool/target is Not verified, not PASS. If every necessary
cell is satisfied by justified retained evidence, document that rather than run
it again. If source/feature/artifact bindings do not support reuse, the missing
claim requires a new focused check or an explicit support-scope decision.
Use existing commands/fixtures; create no general validation infrastructure.

## Release decisions deliberately kept separate

The existing formal provenance proposal names nine subjects. Co-release would
change the eventual subjects and payload, so its old identity/payload/inventory
dispositions cannot silently approve sixteen. D1–D6 stay Pending, including
exact source/workflow identity and service/verifier choices; the plan is not a
co-located workflow/source decision. Registry name/account rights and real
downloaded consumption, reporting/activation controls and final release approval
remain YT-020 gates. Preserve all historical FAIL/PASS and source-bound signatures.

Same-batch publication does not require a native GPU backend, common provider
semantics, mandatory Settings/DI/AOP, all-feature default installation, or a new
generic Audit/Cache/Tracing crate. Application hardware/SLO/HA/backup/operator
qualification remains deployment-owned. The first useful next item is YT-201,
not additional benchmark or cloud qualification.

## Planning verification and independent review

This delivery changes only this report plus additive Backlog/Roadmap/Release
Readiness entries. Keep the four accepted YT-199 file states recoverable and
the YT-199 report byte-identical. Run the repository documentation check and
direct new-report links/fences/EOF/whitespace checks, not product tests/builds,
rustdoc, scanners, browsers/providers or cloud. Independent review must inspect
the actual package/feature/target distinctions, gap necessity, bounded follow-up
items and unchanged historical decisions. No prerequisite item is marked Done
by registration. Results are recorded at closeout.

Read-only manifest/source observations and review evidence are retained under
`/tmp/yetlet-yt200-candidate-scope-20261026-uh0__arf/`.

Documentation checks passed: `cargo xtask check docs` reports 75 fixed required
documents and 205 backlog items; direct new-report local links, fences, EOF and
whitespace passed separately. Tracked whitespace passed. The untracked report's
no-index whitespace check returned exit 1 for expected content differences
against /dev/null with no diagnostics; it is not a failed whitespace gate.

Independent Reviewer `/root/yt200_sixteen_co_release_plan_review_20261026`
returned PASS with zero issues/required followups. It inspected the four YT-200
documents, sixteen manifests and relevant contracts, verified four target/five
evidence hashes, sixteen manifest hashes and four preserved snapshots, and
confirmed additive navigation changes with YT-199 byte-identical. HEAD/status
matched the review binding; five combined dirty paths, zero staged files.
Native trace matched the registered reviewer/model and effective effort with
the shared resolver; there was no conflict or degradation. Review was read-only,
accepted existing check evidence without replay, and performed no product tests,
builds, scans, services or external operations. Only completion/navigation
metadata is updated after PASS. YT-201–205 stay Todo and execution remains
unperformed; this PASS accepts planning, not publication or product qualification.
