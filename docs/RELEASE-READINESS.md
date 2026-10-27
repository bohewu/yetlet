# Yetlet Prototype Release Readiness

**YT-202 native continuation (2026-10-26):** [New GNU evidence and Windows
prerequisite gap](YT-202-NATIVE-FEATURE-GAPS.md) records four source-bound default
graphs and two Ubuntu24 userland/OpenSSL3.5.9 native consumer PASS results.
Selected Windows MSVC remains Not verified because the configured host lacks
its toolchain/VC Tools/SDK; YT-202 is Blocked. Independent Reviewer PASS with zero
issues/required followups accepts the bounded GNU evidence/documentation only.
Commit8fd8751 and same-tree integration74f53fe are local; strict full-history
Push guards stopped both routes before any Push. New history/setup decisions
are pending, with no change to formal candidate/trust/registry gates.

**YT-201 package preparation (2026-10-26):** [Sixteen-package metadata and local
consumer preflight](YT-201-PACKAGE-PREPARATION.md) aligns publication metadata,
versioned local dependencies and license/guide inputs without runtime/feature
changes. Sixteen normal Cargo package checks and two isolated native consumer
runs pass; independent Reviewer PASS with zero issues/required followups accepts
metadata/local preflight only. These are hash-bound uncommitted
preflight inputs on Ubuntu 26.04.1, not final candidate C, actual registry or
public activation evidence. Historical ce56e08 records and YT-096/D1–D6/YT-020
gates remain unchanged.

**Sixteen-package planning (2026-10-26):** [YT-200 co-release review and follow-up
items](YT-200-SIXTEEN-PACKAGE-RELEASE-PLAN.md) considers all sixteen implemented
libraries together, with distinct feature/platform limits and missing validation
registered as YT-201–205 Todo. Independent Reviewer PASS, zero issues/required
followups, accepts planning only; it does not change manifests, select a
replacement source or authorize qualification,
signing/registry/public activation. The original ce56e08 candidate and all
YT-096/D1–D6/YT-020 gates remain intact.

**Current-source planning (2026-10-26):** [YT-199 delivery scope and evidence
gaps](YT-199-CURRENT-DELIVERY-READINESS.md) compares source snapshot `64a6d1c`
with the unchanged nine-package `ce56e08` candidate. Sixteen current API entries
do not select sixteen releases or transfer historical qualification. This is
documentation planning accepted after independent Reviewer PASS with zero
issues/required followups, not a new source/support selection. YT-096/D1–D6 and
YT-020 decisions remain unchanged. The dated
checkpoints below retain their original evidence and verdicts.

**Date:** 2026-10-10
**Decision:** **M0–M9 are demonstrable and independently reviewed at prototype scope; public package publication and production deployment remain blocked.**

**Private preparation update (2026-10-11):** The maintainer selected experimental `0.1.0` and the original nine library candidates. [YT-020 preparation](YT-020-PRIVATE-RELEASE-PREP.md) records the prepared compatibility/maintenance scope, package metadata, actual local preflight and remaining activation gates. YT-020 remains Blocked; this is not publication approval or a visibility change.

**Integrity update (2026-10-12):** The maintainer selected Rust 1.99 before any publication. [Private integrity preparation](YT-020-INTEGRITY-PREPARATION.md) records two-host source-package rebuilding, inventories, unsigned provenance and public-fixture signature verification. This is still preparation under shared maintainer/agent control, not trusted release activation. No commit, push, tag, signing request, publication or visibility change is authorized for this follow-up.

**Explicit trust/registry decision (2026-10-12):** [Private trust policy](YT-020-PRIVATE-TRUST-POLICY.md) selects Cosign public-key preparation and future crates.io distribution. Key custody/fingerprint and actual signing/registry verification remain pending. Public-key preparation cannot satisfy or weaken the formal OIDC, certificate/chain, transparency/inclusion and authenticated-source/builder gates. This follow-up authorizes documentation and independent review only; YT-020 remains Blocked.

**Later authorized execution (2026-10-12):** [Private signing evidence](YT-020-PRIVATE-SIGNING-VERIFICATION.md) now records key/fingerprint enrollment, three preparation signatures and separate verification of 37 artifacts and six negative cases. Formal log-required verification correctly rejects this bundle. Earlier pending-key/signing statements describe their policy checkpoints; formal release guarantees and registry activation remain open. The policy was locally committed; subsequent execution documentation remains unstaged and no push/publication/visibility change occurred.

This document separates bounded repository evidence from a support or security commitment. See [M9 final evidence](M9J-VERIFICATION.md), [API contract classification](API-CONTRACTS.md) and the [post-M9 CI/security review](CI-SECURITY-REVIEW.md).

## Current candidate selection and local preflights (2026-10-20)

The owner selected immutable `ce56e0814e5423782440acd2c959ca777d572cd4`, the original nine experimental `0.1.0` libraries and Rust 1.99/1.99.0. [YT-096 candidate qualification](YT-096-CANDIDATE-QUALIFICATION.md) records the approved native/browser targets, exact new archive digests and remaining gates. Nine-package Cargo verification, archive/license/normalized-manifest checks and four clean packaged-input consumer tests pass on the actual Ubuntu 26.04.1 GNU host. The [selected dependency/API delta](YT-096-SUPPLY-CHAIN-DELTA.md) now records 23 target-specific SBOMs with documented dev-feature over-inclusion, 23 passing license/source checks, nine passing fresh-database RustSec scans, source/API/threat reconciliation and a verified unsigned checksum bundle. These initial local preflights alone do not establish the selected platform/browser matrix, exact compiled-artifact inventory, independent rebuild, final release/security review or formal trust/registry evidence. YT-096 is In progress; YT-020 stays Blocked. No old signature or historical PASS is reassigned to these new artifacts. The dated preparation checkpoints below remain historical evidence.

The [partial platform continuation](YT-096-PLATFORM-QUALIFICATION.md) now records
83 native tests/four doctests in Ubuntu 24.04.5 container userspace, five packaged
wasm builds and a fresh source-reconciled Firefox workflow on Ubuntu 26.04.1.
Its shared host-kernel limitation and historical Chrome boundary remain
explicit. The resumed Windows MSVC standalone archive lane now passes all
nine package commands, 83 native tests and four doctests, with matched
inputs/tools and unchanged locks. Historical cache failures are retained.
These platform results do not complete final platform acceptance or subsequent
release gates. The separately executed [two-host archive rebuild](YT-096-TWO-HOST-REBUILD.md)
now matches all nine Windows source archives exactly to the Linux originals,
with normal Cargo verification. Shared control/staged dependencies do not
satisfy independent builder evidence. Build-bound inventory and formal
trust/release/security/approval gates remain open. The
[earlier Azure SSH attempts](YT-096-AZURE-CLEAN-BUILDER.md) remain historical
failures. The [Run Command/private Blob continuation](YT-096-AZURE-RUN-COMMAND.md)
now passes actual Ubuntu 24.04.5 native execution, empty-cache nine-archive
rebuilding, 83 native tests/four doctests and five wasm libraries. Actual Cargo
observations retain known dev-only SBOM differences and inventory limits.
All temporary resources are deleted; formal independent trust is not established.

The subsequently authorized [GitHub-hosted candidate rebuild](YT-096-GITHUB-HOSTED-REBUILD.md)
uses the existing manual workflow at immutable ce56e08. Run 39418279958 passes
with all nine archives byte-identical, 63 selected source inputs and exact
lock/epoch/tool matches; checksum and unsigned provenance bindings pass. This
adds separately operated builder execution evidence, while authenticated
builder/provenance/trust-policy acceptance and final security/release approval
remain open. No signing or publication occurred; API classifications remain
unchanged and YT-020 remains Blocked.

The [final documentary reconciliation](YT-096-FINAL-RECONCILIATION.md) now
joins nine source archives/package locks to 23 SBOMs and the retained 36-entry
unsigned bundle; actual GNU/wasm Cargo observations have only known dev-only
differences. Existing native/wasm/Firefox and historical Chrome results are
retained without runtime reruns. Historical Chrome has no original independent
candidate-wide manifest, and complete compiled/per-library/system inventory is
Not verified. At that reconciliation checkpoint, support/inventory-limit
acceptance and remaining formal trust/registry/final review/maintainer gates
were open; the subsequent decisions below retain those evidence boundaries.

The subsequent [independent final candidate review](YT-096-SECURITY-RELEASE-REVIEW.md)
is complete with overall FAIL and release-ready NO. It accepts bounded
technical/security evidence and identifies no new material source defect in the
inspected delta. At that review checkpoint, formal integrity/trust, final
support/inventory acceptance, actual registry proof, public activation controls
and concrete maintainer release approval were unmet. These are existing release gates;
the review authorized no repairs, reruns, signing or publication. YT-096 stays
In progress and YT-020 stays Blocked.

After the review report was committed as `2118495`, the owner explicitly
accepted the [bounded support scope and known inventory differences](YT-096-FINAL-RECONCILIATION.md#owner-support-and-inventory-acceptance-2026-10-20).
Missing evidence remains Not verified, including exact-candidate Chrome
qualification and complete compiled/binary/system inventory. This resolves
only the missing support/inventory disposition. The original final verdict
stays FAIL / release-ready NO; formal integrity/trust, actual registry proof,
public activation/reporting controls and concrete final maintainer approval
remain open. No policy, API classification, required CI or release authority
changes, and no runtime rerun, signing or publication occurs.

The accepted support/inventory update was committed as `0bd7596`.
[Formal trust/provenance research](YT-096-FORMAL-TRUST-PROPOSAL.md) proposes
a conditional GitHub OIDC/public Sigstore route with exact identities,
approved roots, source-package provenance, a complete evidence manifest and
independent verification. Disclosure and a future workflow/source candidate
remain owner decisions; this research does not implement or satisfy the
formal trust gate. All four release conditions and existing evidence limits
remain unchanged, with no signing or publication.

## Current documentation and package boundary (2026-10-12)

YT-068 completes the developer entry, exactly nine package guides, evidence-preserving navigation and [T08 Private preparation reconciliation](YT-020-PRIVATE-RELEASE-PREP.md#current-preparation-boundary--t08--yt-068-2026-10-12) at bounded documentation scope. The five research items YT-029/YT-021/YT-024/YT-022/YT-023 are Done with their linked [actual evidence and decisions](README.md#completed-research-prototypes), without supported-core promotion. T07's nine package-list checks passed; they establish README inclusion only. The independent delivery gate/result is recorded separately in the run; documentation-stage checks do not attest that gate.

Signed source `8e557e760407a3d4b3f5a8b8f833748946be1387`, policy HEAD `abb2cce1253afe1d87a97e0a50f3f47efffe7280` and the current HEAD-plus-dirty preparation checkout are distinct. The historical three signatures and 37-artifact set remain valid historical Private evidence; they do not attest new candidate READMEs, root manifest/lock changes or research. Historical compiler/browser/CSP/CI and audit results below retain their original identities; they are not fresh qualification or an audit of the current research dependency graph. New-source package preflight, rebuild/provenance, selected SBOM/license evidence and signing verification require a separately approved immutable candidate and execution authority. No such rebuild/signing or broad qualification is run for this documentation phase. YT-020 remains Blocked and a later local commit alone does not activate release.

## M11 planning boundary (2026-10-12)

[M11](M11-PLAN.md) separately proposes product-readiness and an official opt-in Experimental `yetlet-wgpu` line. This documentation filing does not implement a product/renderer, expand the original nine release candidates, change YT-020 or qualify an operating environment. Current source inspection starts from clean `c7ecc49eeb667e145cfb7e764c10b72ba104bbee`; historical dirty/signing/CI statements retain their original checkpoints. Lack of public publication does not itself prevent an explicitly approved immutable-source private deployment, and publication/signatures do not establish runtime correctness. [Product research](M11-PRODUCT-READINESS-RESEARCH.md) defines the finite application evidence and owner responsibilities required before such a scoped claim.

## Implemented behavior and retained validation

- Twelve runnable samples cover native SSR, typed APIs, opt-in Session/CSRF, middleware, files, direct SQLite, SSE, multiple browser islands, synthetic authorization and modular composition.
- Handler-first `App`, `RequestContext`, Tower interoperability and renderer-neutral core values are classified as Stable Candidates, not stable APIs.
- Complete M9 evidence retained 190 Rust tests, five doctests, 11 native smokes, eight wasm32 packages, Chrome and Firefox suites, dependency isolation and byte-identical production assets.
- Strict CSP, escaping, body/media limits, parameterized SQL, transactional rollback, bounded providers and application-level authorization have negative-test evidence.
- The declared minimum is now Rust 1.99, with development pinned to 1.99.0. Historical Rust 1.88 tests remain source-bound evidence. Retained stable CI on `633db585` already used Rust 1.99; new local compiler/package checks cover the metadata/toolchain update without repeating sufficient runtime/browser/Docker suites.
- `cargo-audit 0.22.2` reports zero vulnerabilities and warnings for the patched lockfile.
- The updated GitHub Actions workflow uses read-only permissions, full-SHA action pins, an MSRV job, RustSec audit, native sample smokes and three Chrome release suites.

## Standard development and validation commands

```bash
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version 0.2.129 --locked

cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --all-targets --locked
cargo test --workspace --doc --locked
cargo xtask check docs
cargo xtask check samples

cargo xtask build browser --release
cargo build --release --locked -p yetlet-todo-server
cargo xtask smoke browser-chrome --profile release
```

The complete sample/browser matrix is recorded in M9 evidence and the checked-in workflow. Firefox needs geckodriver. Informational latency and renderer benchmarks are not CI performance thresholds.

## Readiness and unresolved gates

The table records both framework evidence and application deployment concerns.
TLS/ACL deployment policy, backup protection, business RPO/RTO, on-call ownership
and real production rollout are application/operator responsibilities, not
prerequisites for bounded YT-066 framework completion. See the [responsibility
boundary](YT-066-PERSISTENCE-RECOVERY-PLAN.md). YT-020 remains Blocked for its
separate framework publication/activation gates; no release is authorized here.

| Gate | State | Detail |
|---|---|---|
| M0–M9 prototype contracts | Verified locally | Per-stage evidence and independent reviewers |
| Rust 1.99 minimum | Selected pre-release; local and first-candidate hosted checks pass | Development/packaging/compiler pins agree on 1.99.0. First-candidate run `38193149046` passes both jobs; protection now requires `Rust 1.99 MSRV` plus the unchanged Stable check. Historical 1.88 identities remain accurate; later candidate updates require their own actual CI |
| Retained stable Rust checks | Verified locally at recorded source | Format, strict Clippy, tests, doctests, smokes and wasm32 |
| Historical dependency vulnerability audit | Passing after repair at recorded lockfile | `RUSTSEC-2026-0009` fixed by `time 0.3.47`; Dependabot alerts/security fixes and per-CI `cargo-audit` are enabled, but no scheduled version-update PR configuration or broader continuous monitor exists |
| Hosted GitHub Actions | **Delivered baseline and first Rust 1.99 candidate verified** | PR #5 and master `633db585` retain runs `38144070219` / `38145022739`. PR #6 first-head run `38193149046` passes on actual synthetic `cdc67bb6`, tree-equal to candidate `9ec9b7b`. Later candidate/workflow updates are not claimed as execution of that first-head run; exact final delivery evidence is retained separately |
| Platform/browser compatibility | **YT-065 automated scope complete; optional coverage unverified** | Hosted Linux, Windows MSVC, macOS ARM64 and Firefox four-suite execution PASS; local Windows GNU and offline Docker PASS. WebKit/mobile, real IME, screen readers and zoom remain explicitly Not verified and are not supported-platform claims |
| Accessibility | **Partial** | Actual Tab/Enter/Unicode, labels/errors/focus and root-language checks pass; real IME, screen readers, zoom and conformance are not verified |
| Authentication/authorization | **Threat model specified; implementation prototype only** | M10d requires external OIDC plus server-side resource authorization; Auth Notes remains synthetic and is not an IdP/password/passkey/MFA design |
| Credentialed CSRF/Session | **Prototype; single-host limitation observed** | Bounded in-memory Session/CSRF works. YT-066a three-instance Docker execution proves stateless routing but also proves Memory Session is rejected cross-instance and lost on issuer restart; final independent review passed. The separate experimental Redis provider has reviewed b2 implementation and [b3](YT-066B3-VERIFICATION.md) passing single-host/no-persistence execution, cleanup and final independent review; broader deployment durability/recovery is unverified and application-specific, not a bounded YT-066 completion prerequisite |
| HTTPS/reverse-proxy trust and rate limiting | **Specified, not implemented** | M10d defines the trusted-edge/header and layered limiting contract; a selected deployment and abuse/load evidence remain required |
| Persistence/transactions | **Local qualification only** | Actual two-process local WAL conflict, online backup, exact isolated restore and corrupt candidate rejection PASS; encryption/retention, off-host durability and distributed production operations remain unqualified |
| Observability/load/cancellation | **Partial** | Lifecycle/cancellation tests exist; no production tracing, SLO, soak or incident evidence |
| Repository security controls | **Partial, private-state configured** | Collaborator-only PRs, protected `master`, required CI, SHA-pinned GitHub-owned Actions and Dependabot enabled; private vulnerability reporting, secret scanning and CodeQL unavailable until Public/current-plan support |
| Package license and vulnerability policy | **Selected and independently reviewed** | `MIT OR Apache-2.0`, `SECURITY.md` and maintainer-directed/no-external-PR policy PASS; GitHub private reporting must be enabled at Public transition |
| Stable API/SemVer/support policy | **Experimental policy prepared; activation missing** | Original nine candidates, coordinated `0.1.x`, patch API/MSRV preservation and best-effort maintenance are selected privately. API classifications stay intact; no released supported version, SLA or production stability is claimed |
| Source-package metadata | **Private preflight verified; crates.io selected for future activation** | Compatible `0.1.0` dependencies retain paths. Retained native package builds pass; future destination is crates.io, original nine names, intended owner `bohewu`. Real account/name rights, exact uploaded/downloaded digests and clean-consumer registry resolution remain unverified. Package disclosure and repository visibility require separate approvals |
| Artifact integrity/release supply chain | **Private maintainer-key signing/verification complete; formal trust missing** | Hosted run `38195517029` and primary archives retain immutable `8e557e7` identities. Three preparation signatures and their 37-artifact set pass separate public-only verification; six negative cases pass, including rejection with log verification required. This does not authenticate an OIDC builder or provide certificate/log proofs; formal release trust, approvals and immutable promotion remain gates |
| Codex-assisted internal security assessment | **Done at internal source/fix scope** | Official Standard scan on pre-patch `25bff77` reviewed 227/227 regular files and found one Low issue; local control-character guard, Rust/MSRV/WASM/Chrome regressions and independent reviewer PASS. Original older-source scan incomplete; patched tree not rescanned; no external audit/certification |

## Decisions before publication

1. **Maintainer `bohewu`:** Activate only the selected experimental `0.1.0`/nine-library scope after its concrete gates pass. The prepared [compatibility policy](COMPATIBILITY.md) preserves API/MSRV in patches, limits maintenance to the latest approved experimental patch and retains source-aware qualification. No stable API or production support is declared.
2. **Release approver `bohewu`:** Approve a clean reviewed immutable source/tag, registry name/ownership and resolution, selected-target inventory/license evidence, independent rebuild, trusted provenance/signatures, exact digests and rollback target. The existing [supply-chain requirements](RELEASE-SUPPLY-CHAIN.md) are not replaced by local checksums or green CI.
3. **Public-transition owner `bohewu`:** After separate authorization, verify a reachable private reporting route/fallback and enable/verify GitHub Private Vulnerability Reporting, secret scanning/push protection and CodeQL. Preserve collaborator-only PRs, protected `master` and the current sole-maintainer approval rule. Migrate the selected MSRV check name without bypass or weaker protection and retain `Stable native, samples, and Chrome`; revisit governance only if another maintainer is added.
4. **Security acceptance owner `bohewu`:** Record acceptance of the concrete release source delta and findings/residual risks while retaining the YT-067 scan/fix identity boundary. Retain scheduled evidence; optional IME/AT/device/WebKit/mobile/zoom work is required only by a separately selected support/application scope. No automatic full rescan or paid external assessment is required by this preparation.

Application deployment owners apply the [M10d Threat Model](THREAT-MODEL.md) to their chosen proxy/IdP/Session/limiter/secrets/storage products. Production TLS/ACL, backups, RPO/RTO, staffing, durability/HA and rollout are not framework source-package publication prerequisites.

## Scope protection

Yetlet keeps native HTTP, renderer, browser and optional provider dependencies separable. Auth, Session, database, files, SSE, tracing, CORS, rate limiting and deployment capabilities remain explicit opt-ins. The SQLite examples do not introduce an ORM, `DbContext`, repository framework or default data provider.

**Backlog status:** M0–M9 and YT-060–YT-065 are Done at their bounded prototype/policy/design/automated-qualification scopes. YT-065's real-device/IME/assistive-technology/WebKit/mobile/zoom cells remain explicit optional unknowns; YT-066 is Done at single-host Docker multi-instance and selected Redis-provider scope; broader deployment durability/recovery remains Not verified. Application TLS/ACL policy, backup protection, RPO/RTO and operational staffing are not framework completion prerequisites. Revised YT-067 is Done at internal pre-patch source-scan and independently reviewed local Low finding correction scope. The earlier scan of an older snapshot remains incomplete, and no post-fix repository-wide rescan or production certification is claimed. YT-068 and the five authorized research studies are Done at their bounded documentation/prototype scopes: at the pre-M11 checkpoint, 67 of 68 backlog items were Done and YT-020 alone remained Blocked. M11 now files additional work without changing those historical completion results; current counts belong to Backlog/Kanban. The independent delivery gate/result is recorded separately in the run; documentation-stage checks do not attest that gate. No historical signature, local commit, review, license, checksum or green CI automatically authorizes current-artifact assurance, production or publication.

**YT-202 readiness correction (2026-10-27):** [Remaining native cells](YT-202-NATIVE-FEATURE-GAPS.md#current-native-coverage--2026-10-27) confirms the reconnected Windows MSVC toolchain/VC Tools/SDK are now ready. Accepted GNU/package results remain unchanged and are not replayed; fourteen-package default and selected-feature MSVC consumer executions are still Not run. The earlier artifact source-read policy rejection has not been resolved; an admin-only diagnostic is unavailable to the current connection, and no exact policy change is proposed. YT-202 stays Blocked until admitted source delivery, both executions and final independent review. PR #12 now contains4e5ef64 with green RustSec/MSRV/native/browser CI and remains unmerged; that hosted Linux result does not replace these Windows cells or select a new final release candidate. This continuation performs no Commit/Push/Merge.

**YT-202 authorized source-route update (2026-10-27):** The owner explicitly selects Git clone for the remaining Windows checks. [The new task-owned clone](YT-202-NATIVE-FEATURE-GAPS.md#owner-authorized-git-source-continuation--2026-10-27) is admitted and fixed at74f53fe/tree2faa, matching accepted YT-201 source; no rejected artifact is relocated or transferred. Windows prerequisites are ready; preparation uses the existing Cargo bootstrap/directory-source convention and a newly bound equivalent fourteen-package consumer. The earlier artifact/admin diagnostic blockers are historical. YT-202 is In progress, with both MSVC executions and final independent review pending; accepted GNU/package/provider/browser evidence is retained without replay. No new Commit/Push/Merge or final release selection.

**YT-202 MSVC execution (2026-10-27):** [Two new native consumers](YT-202-NATIVE-FEATURE-GAPS.md#new-msvc-results-and-preserved-preparation-failure--2026-10-27) PASS on prepared Rust1.99MSVC/VC14.44/SDK26100: fourteen default package dependencies, then Auth/session and Config/secret-provider with missing-Memory-layer refusal and CPU-only wgpu. Inputs/lock remain unchanged; all40 package-file hashes reconcile to committed CRLF checkout and100 external lock identities are retained, without a Linux archive-byte-identity claim. The fresh fixture dependency-preparation failure and environment-observation correction remain recorded; no passing native run or old suite was replayed. Six task-owned Windows directories are removed and evidence retained. YT-202 awaits final independent review before Done; no Commit/Push/Merge or release activation.

**YT-202 final native acceptance (2026-10-27):** [Final independent Reviewer PASS](YT-202-NATIVE-FEATURE-GAPS.md#final-independent-acceptance--2026-10-27), zero issues/required followups, closes YT-202 at selected native/MSRV/feature scope. Both fourteen-package MSVC consumers pass with exact source/feature/lock/tool/result/cleanup bindings;4documentation/30new/32retained evidence hashes and24ZIP members match. Original request records Rust1.99MSVC selection; generic non-Git source freshness and directory-source/CRLF limits remain explicit. Prior failures/reviews and accepted GNU evidence stay intact without replay. Four task documents remain uncommitted; no Commit/Push/Merge or release activation. YT-203 is next; YT-204/205, YT-096/D1–D6 and YT-020 gates remain separate.

**YT-203 source-bound browser continuation (2026-10-27):** [WASM/DOM/optional GPU inventory](YT-203-WASM-DOM-GPU-EVIDENCE.md) reuses accepted CI4e5ef64 original-five/MSRV/Chrome evidence, unchanged DOM contracts and YT-081 actual crate GPU inputs; Lab and Tasks remain separate application evidence. Only missing normalized six-package WASM default/browser compile and current-package Firefox154 DOM cells execute: both Cargo checks and23new controls/navigation/CSP/atomic-refusal observations PASS. Source/lock/assets,34-node default isolation and owned cleanup are retained; directory-source/old-GPU-lock/compile-only and unavailable historical tmp limits remain explicit. Final independent Reviewer is pending before YT-203 Done. No new runtime/sample, old suite replay, new-change Commit/Push/Merge or release activation.

**YT-203 final browser acceptance (2026-10-27):** [Independent Reviewer PASS](YT-203-WASM-DOM-GPU-EVIDENCE.md#final-independent-acceptance--2026-10-27), zero issues/required followups, closes the selected packaged WASM/DOM/optional GPU evidence gaps. All4 candidate docs,37 evidence/fixture files,17 protected inputs and6 packaged-source comparisons verify; original GPU result/review and28 retained inputs reconcile to the six disclosed changed wrappers. New default Wasm build, browser-feature compile and23 Firefox observations pass with owned cleanup; accepted Chrome/GPU/BFCache proof is reused without replay or scope expansion. Completion metadata alone follows acceptance. Four YT-203 documents remain uncommitted; no Push/Merge, runtime feature or release activation. YT-204 remains Todo and is next; YT-205, YT-096/D1–D6 and YT-020 gates remain separate.
