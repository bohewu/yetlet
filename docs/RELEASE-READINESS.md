# Yetlet Prototype Release Readiness

**Current distribution (2026-11-12):** **Published SDK:** `0.9.0-alpha.8` (`yetlet` / `yetlet-browser`).
**Published CLI:** `0.4.0` (`yetlet-cli`). The accepted
[SDK publication record](ALPHA8-HOSTED-RECORD.md#owner-approved-publication-and-registry-consumer--2026-11-12)
and [CLI publication record](CLI-040-HOSTED-RECORD.md#owner-approved-publication-and-installed-consumer--2026-11-12)
record actual GitHub/crates.io delivery, exact archives, signatures and consumer limits.
The typed Tag Builder first shipped in SDK alpha.7. SDK alpha.8 includes M26
application startup methods and the optional Axum Prelude. The independent CLI
ships private app Style generation/templates, SDK resolution with exact app
pins and locks, and SDK-aware native startup scaffolds. Both published
distributions passed their registry consumers. The Reader Edition source
selects SDK alpha.8 and CLI 0.4.0. Development version equality does not
establish archive/source equality. This is not Stable API, production, SLSA
level or universal platform qualification.

For a new release, use [RELEASING.md](RELEASING.md), including its current
version-binding/tooling gaps. This page retains historical readiness decisions,
not a runnable release sequence. The dated Pending/FAIL/unpublished states below
remain true for their original checkpoints; they do not describe current
distribution or transfer qualification to another source.

**CLI 0.3.1 release (YT-250):** The
[preparation record](CLI-031-RELEASE-PREPARATION.md) tracks the Owner-corrected
`sdk-latest-v2` default and retained exact pins/locks/provenance.

Independent semantic/source-freeze review and local consumers passed for the
original candidate. Windows Browser Style shutdown has since been corrected and
verified on the same MSVC runner. The replacement completed fresh hosted build,
custody, frozen preparation, signing and independent verification under the
approved unpublished-candidate tag transition.
The [hosted record](CLI-031-HOSTED-RECORD.md) retains both candidates and the
Owner-approved publication of the replacement. Anonymous GitHub and crates.io
readback confirmed the same archive, and the installed consumer passed Native
and Browser Style runtime checks.

### Historical source-only package inventory versus current publication

The 2026-10-26 YT-199 delivery-scope review was **documentation-only**:
it compared a fixed nine-library `0.1.0` candidate against a later
source checkout with seven additional experimental opt-in workspaces.
At that **original** checkpoint none of the sixteen packages was
published, YT-020 public signing remained Blocked and YT-096 source/trust
gates were incomplete. Source comparisons were not package archives,
registry consumers, a new release candidate or production qualification.
Those dated findings remain recoverable at exact **local Git**
`42d8856b9acf5ddddab9a4b998b5bdd4c5e59210` (remote reachability not verified). They must not be
carried forward as current unavailability.

The current published distribution is independently established above
by the exact hosted **SDK alpha.8 two-package** and **CLI 0.4.0** records.
Optional app-owned Auth/Session/cache/renderer integrations are not
automatically included in those releases, and a source-only proof or
an older certificate/signing result cannot qualify a newer checkout,
browser, platform or production deployment. Follow the canonical
[API/reference and compatibility guides](COMPATIBILITY.md) and the
[actual release SOP](RELEASING.md) for current decisions.
## Historical release and hosted source checkpoints

These are dated results, not the current published pair above or renewed
execution authority. The eighteen original Z11 Markdown records and heading/blob
identities are locally recoverable at Git
`c440c4542c888671750738a5908a667d2124db15`; their GitHub SHA reachability is
unverified. Original logs, signatures and receipts remain in their recorded
ignored/external stores, whose continued byte availability is not guaranteed by
Git. Current operating rules belong to [RELEASING](RELEASING.md),
[publication](RELEASE-PUBLICATION.md) and the retained normative trust contracts.

| Date / immutable scope | Actual result and surviving limit |
|---|---|
| 2026-10-20, nine unpublished experimental 0.1.0 packages, C `ce56e0814e5423782440acd2c959ca777d572cd4`, Rust1.99.0 | Nine normally verified archives and four directory-source consumer tests PASS; this was a local prepublication simulation, not registry ownership/download proof. Native/wasm/browser/inventory results are separated in the [qualification matrix](QUALIFICATION-MATRIX.md#historical-nine-package-candidate-platform-and-builder-evidence) |
| Same C, final security/release review | **FAIL / release-ready NO** despite accepted bounded technical/security evidence and no new material defect found in the inspected delta. Five conditions were unmet: formal integrity/trust, support/inventory disposition, actual registry proof, public activation/reporting and concrete final maintainer approval. After report commit `2118495`, owner acceptance resolved only support/inventory disposition; four conditions remained open. Chrome exact-candidate binding and complete compiled/system inventory stayed Not verified; later releases do not retrospectively pass this candidate |
| 2026-10-27, sixteen unpublished 0.1.0 packages | At `a2d6474a72d5a4f6a56c19195a348c1d648b32ae`, ten changed-Tokio-lock normal package checks plus six exact compilation reuses, one consumer, eight rustdoc commands/22 entries and fourteen affected inventory cells PASS. Twenty-eight inventories were retained. Initial checkpoint and GNU-delta reviews both PASS but explicitly `item_can_be_done:false`; MSVC/current-source CI/rebuild and two package-guide corrections were still open |
| 2026-10-27, successor C `118f3d0f496b487ef20fe243f4e73179f9eadc43` | CI40473956480 PASS on synthetic merge `d53a0f6830e3c31ed8a00a9278dd8577f9e859b6` with complete tree equal C; hosted40475829522/1 and matching local Git-archive recipe each pass33 phases, all16 archives byte-identical. Two affected MSVC consumers and46 reader manuscripts reconcile. Independent PASS closes YT-205 technical/reader scope; D1–D6 remained Pending at that checkpoint, with no signing/publication approval |
| 2026-10-28, historical unsigned `v0.1.0`, C `3eacfe31f5e3281411d3bc4653a6baccab668b30` | Build40628943583/1, protected verifier40631789238/1 and local correspondence PASS. Actual owner-approved readback reports complete preparation/full manifest; owner-host custody51 files/129,846,094 bytes recorded12:25:12 UTC+08. Seven-day artifacts expiring11-04 were transport only; no offsite/indefinite availability, signing or registry claim |
| 2026-10-28, alpha.1 C `bf3363801132f4481263ceb411d9bba3349df86d` | PR14 tree equality/source selection and master CI40654332287/1 PASS after retained format failure40651836854. Build40669449040/1 PASS initially left readback Pending/Not verified. Separately authorized verifier40688772221/1 and owner approval later PASS full manifest; distinct custody57 files/130,178,085 bytes, index `426c4d0252a92cb5a39a658c7d3d95734d802dcbb431554b7954b64c41b62405`. Original0.1.0 custody is unchanged |
| 2026-10-29, alpha.1 signer failures then distinct accepted S | Prepare40717870186/1 at S87d24e9 failed read-only preflight, protected work skipped; prepare40789535074/1 at S0e8fb60 failed Draft metadata HTTP403. At Se2d4a9f,40795794183/1 froze/uploaded data but final Go-cache cleanup failed: overall **FAIL**, ineligible for signing. Local handoff/diagnostic/cleanup fixes did not convert those runs to PASS |
| 2026-10-29, alpha.1 S `d65e3180ad3076659f03c069fc71cd0e7e3f0518` | New authorized prepare40806988578/1 and sign40811275645/1 PASS. Independent public/private verification binds16 subjects/all117 entries; single-use local custody exception and final Reviewer PASS0 activate exact-output owner acceptance. D6 Complete/YT-096 Done at accepted source-package signing/integrity scope. Separate owner approval then publishes alpha.1 GitHub Experimental pre-release with20 public files, not Latest, anonymous readback PASS; no registry upload at that checkpoint |

The original direct planned-input tag Guard screen failed with13 historical
findings across two commits/six files (23 commits/1,760 scan operations).
It was not native-hook execution. Design Reviewer P2 FAIL then focused PASS
corrected that distinction. Later authorized Guard0.44 native one-use tag Push
passed with zero outgoing commits; findings remained accepted-existing debt.
An after-Push stdout/stderr wrapper failure was corrected by observing the same
successful Push, without repush/rearm. Guard0.45 used-binding retirement was
metadata-only and preserved the exact archived used record, not a new grant.
These dated results do not authorize Guard installation, remote access or work
in another repository. Preserve [current source/tag and freeze rules](RELEASING.md#release-identities-and-freeze-order).

Detailed original alpha failure/custody identities remain in the retained
[signer contract](RELEASE-SUPPLY-CHAIN.md#two-payloads-complete-assembly-and-independent-verification); the original
[alpha checkpoint](https://github.com/bohewu/yetlet/blob/c440c4542c888671750738a5908a667d2124db15/docs/YT-096-ALPHA-HOSTED-PLAN.md#actual-alpha-protected-verifier-and-retention--pass)
is locally verified history, with remote reachability unverified. Failure
stores include `target/yt096-alpha-hosted-*20261028/`,
`target/yt209-delivery-20261029/` and the recorded repository-external custody
indexes; no raw store is moved or claimed newly verified by consolidation.

## Historical readiness checkpoints

The [alpha.1 frozen input and signed output identities](#alpha1-frozen-inputs-signatures-and-acceptance--2026-10-29)
preserve the later D6 acceptance independently of these earlier checkpoints.

**Owner-selected version update (2026-10-28):** The next sixteen-package
Experimental preview is `0.9.0-alpha.1`; see the [alpha version preflight](https://github.com/bohewu/yetlet/blob/849c5894f73d9732db9c2ccaa765385dc9d8a819/docs/YT-096-ALPHA-VERSION-PREFLIGHT.md).
The owner now selects guarded 59-path commit `5d7b91842f8188d45e5af22367245aebec7ca7d1`
(tree `6adff26fc4fec65e9aa8f40d7172ff266b7e500d`) as the new immutable alpha candidate C.
The [minimum hosted/readback plan](RELEASE-READINESS.md#historical-release-and-hosted-source-checkpoints) records the
remaining fixed-binding patch, consumed Guard-state disposition and separate
execution-source selection/authorization gates. Existing `v0.1.0`, source
`3eacfe3`, hosted/readback runs and retained bytes stay historical and unchanged;
no alpha output or signing payload inherits their identity. No new execution C,
alpha tag, hosted run or payload SHA is selected by the proposed future patch. The separate fixed-SHA signer design still applies. D6 Pending;
YT-096 In progress; YT-020 Blocked. No tag, hosted execution, signing or publication
is authorized by this migration.

**Latest YT-205 execution:** Candidate118f3d0 has required CI PASS, sixteen
byte-identical hosted/local package archives and both affected MSVC consumers
PASS. [Current evidence and limits](RELEASE-READINESS.md#historical-release-and-hosted-source-checkpoints)
have final independent Reviewer PASS0 at selected technical/reader scope. The
earlier dated checkpoints below retain their
point-in-time identities; no Merge/signing/publication and D1–D6 remain Pending.

**YT-205 new candidate and remaining order (2026-10-27):** Strict/builtin Guard
and the native hook pass with zero findings for the ten-path rebuild-extension
commit `118f3d0f496b487ef20fe243f4e73179f9eadc43`; its tree matches `tree_oid`.
The owner selects that exact commit as new technical C. The
[remaining plan](https://github.com/bohewu/yetlet/blob/c440c4542c888671750738a5908a667d2124db15/docs/YT-205-RELEASE-EVIDENCE-CONSOLIDATION.md#new-candidate-and-remaining-verification-order--2026-10-27)
orders separately authorized source delivery, required PR CI, explicit sixteen
cohort hosted/local archive comparison, affected fourteen-package MSVC scope
and final evidence reconciliation/independent review. Old a2d6474 archives/results
retain their identities; no old passing suite is replayed here. These post-commit
selection notes remain uncommitted, with no Push/Merge/dispatch or new platform
validation. YT-205 remains In progress, YT-020 Blocked and D1–D6 Pending.

**YT-205 selected-source consolidation (2026-10-27):** After guarded28-path
commita2d6474 without Push, the owner selects that immutable source for all16
experimental0.1.0 libraries. [The evidence packet](RELEASE-READINESS.md#historical-release-and-hosted-source-checkpoints)
binds16 clean C archives,10 new changed-lock package checks/six exact compilation
reuses, one new integration consumer, eight first-time rustdoc commands and14
affected inventory cells, all new commands PASS after preserved preparation
failures.28 inventories and prior source/platform/browser/audit results remain
source-bound without replay. Exact-source hosted CI/rebuild, affected native
dependency evidence and C-versus-draft reader guidance remain open; YT-205 stays
In progress and delivery review is pending. New documentation is uncommitted;
no Push/Merge, signing or publication, and D1–D6 remain Pending.

**YT-202 native continuation (2026-10-26):** [New GNU evidence and Windows
prerequisite gap](QUALIFICATION-MATRIX.md#historical-sixteen-package-qualification-and-wasm-proof) records four source-bound default
graphs and two Ubuntu24 userland/OpenSSL3.5.9 native consumer PASS results.
Selected Windows MSVC remains Not verified because the configured host lacks
its toolchain/VC Tools/SDK; YT-202 is Blocked. Independent Reviewer PASS with zero
issues/required followups accepts the bounded GNU evidence/documentation only.
Commit8fd8751 and same-tree integration74f53fe are local; strict full-history
Push guards stopped both routes before any Push. New history/setup decisions
are pending, with no change to formal candidate/trust/registry gates.

**YT-201 package preparation (2026-10-26):** [Sixteen-package metadata and local
consumer preflight](QUALIFICATION-MATRIX.md#historical-sixteen-package-qualification-and-wasm-proof) aligns publication metadata,
versioned local dependencies and license/guide inputs without runtime/feature
changes. Sixteen normal Cargo package checks and two isolated native consumer
runs pass; independent Reviewer PASS with zero issues/required followups accepts
metadata/local preflight only. These are hash-bound uncommitted
preflight inputs on Ubuntu 26.04.1, not final candidate C, actual registry or
public activation evidence. Historical ce56e08 records and YT-096/D1–D6/YT-020
gates remain unchanged.

**Sixteen-package planning (2026-10-26):** [YT-200 co-release review and follow-up
items](https://github.com/bohewu/yetlet/blob/849c5894f73d9732db9c2ccaa765385dc9d8a819/docs/YT-200-SIXTEEN-PACKAGE-RELEASE-PLAN.md) considers all sixteen implemented
libraries together, with distinct feature/platform limits and missing validation
registered as YT-201–205 Todo. Independent Reviewer PASS, zero issues/required
followups, accepts planning only; it does not change manifests, select a
replacement source or authorize qualification,
signing/registry/public activation. The original ce56e08 candidate and all
YT-096/D1–D6/YT-020 gates remain intact.

**Current-source planning (2026-10-26):** [YT-199 delivery scope and evidence
gaps](https://github.com/bohewu/yetlet/blob/42d8856b9acf5ddddab9a4b998b5bdd4c5e59210/docs/YT-199-CURRENT-DELIVERY-READINESS.md) compares source snapshot `64a6d1c`
with the unchanged nine-package `ce56e08` candidate. Sixteen current API entries
do not select sixteen releases or transfer historical qualification. This is
documentation planning accepted after independent Reviewer PASS with zero
issues/required followups, not a new source/support selection. YT-096/D1–D6 and
YT-020 decisions remain unchanged. The dated
checkpoints below retain their original evidence and verdicts.

**Date:** 2026-10-10
**Decision:** **M0–M9 are demonstrable and independently reviewed at prototype scope; public package publication and production deployment remain blocked.**

**Private preparation update (2026-10-11):** The maintainer selected experimental `0.1.0` and the original nine library candidates. [YT-020 preparation](QUALIFICATION-MATRIX.md#private-nine-package-preparation-and-verifier-fixtures--2026-10-11-and-12) records the prepared compatibility/maintenance scope, package metadata, actual local preflight and remaining activation gates. YT-020 remains Blocked; this is not publication approval or a visibility change.

**Integrity update (2026-10-12):** The maintainer selected Rust 1.99 before any publication. [Private integrity preparation](QUALIFICATION-MATRIX.md#private-nine-package-preparation-and-verifier-fixtures--2026-10-11-and-12) records two-host source-package rebuilding, inventories, unsigned provenance and public-fixture signature verification. This is still preparation under shared maintainer/agent control, not trusted release activation. No commit, push, tag, signing request, publication or visibility change is authorized for this follow-up.

**Explicit trust/registry decision (2026-10-12):** [Private trust policy](RELEASE-SUPPLY-CHAIN.md#private-maintainer-key-preparation-policy-and-historical-signatures) selects Cosign public-key preparation and future crates.io distribution. Key custody/fingerprint and actual signing/registry verification remain pending. Public-key preparation cannot satisfy or weaken the formal OIDC, certificate/chain, transparency/inclusion and authenticated-source/builder gates. This follow-up authorizes documentation and independent review only; YT-020 remains Blocked.

**Later authorized execution (2026-10-12):** [Private signing evidence](RELEASE-SUPPLY-CHAIN.md#private-maintainer-key-preparation-policy-and-historical-signatures) now records key/fingerprint enrollment, three preparation signatures and separate verification of 37 artifacts and six negative cases. Formal log-required verification correctly rejects this bundle. Earlier pending-key/signing statements describe their policy checkpoints; formal release guarantees and registry activation remain open. The policy was locally committed; subsequent execution documentation remains unstaged and no push/publication/visibility change occurred.

This document separates bounded repository evidence from a support or security commitment. See [M9 final evidence](API-CONTRACTS.md#original-inventory-acceptance-and-review-limits), [API contract classification](API-CONTRACTS.md) and the [post-M9 CI/security review](CI-SECURITY-REVIEW.md).

## Current candidate selection and local preflights (2026-10-20)

The owner selected immutable `ce56e0814e5423782440acd2c959ca777d572cd4`, the original nine experimental `0.1.0` libraries and Rust 1.99/1.99.0. [YT-096 candidate qualification](QUALIFICATION-MATRIX.md#historical-nine-package-candidate-platform-and-builder-evidence) records the approved native/browser targets, exact new archive digests and remaining gates. Nine-package Cargo verification, archive/license/normalized-manifest checks and four clean packaged-input consumer tests pass on the actual Ubuntu 26.04.1 GNU host. The [selected dependency/API delta](RELEASE-SUPPLY-CHAIN.md#historical-candidate-inventory-and-technical-consolidation) now records 23 target-specific SBOMs with documented dev-feature over-inclusion, 23 passing license/source checks, nine passing fresh-database RustSec scans, source/API/threat reconciliation and a verified unsigned checksum bundle. These initial local preflights alone do not establish the selected platform/browser matrix, exact compiled-artifact inventory, independent rebuild, final release/security review or formal trust/registry evidence. YT-096 is In progress; YT-020 stays Blocked. No old signature or historical PASS is reassigned to these new artifacts. The dated preparation checkpoints below remain historical evidence.

The [partial platform continuation](QUALIFICATION-MATRIX.md#historical-nine-package-candidate-platform-and-builder-evidence) now records
83 native tests/four doctests in Ubuntu 24.04.5 container userspace, five packaged
wasm builds and a fresh source-reconciled Firefox workflow on Ubuntu 26.04.1.
Its shared host-kernel limitation and historical Chrome boundary remain
explicit. The resumed Windows MSVC standalone archive lane now passes all
nine package commands, 83 native tests and four doctests, with matched
inputs/tools and unchanged locks. Historical cache failures are retained.
These platform results do not complete final platform acceptance or subsequent
release gates. The separately executed [two-host archive rebuild](https://github.com/bohewu/yetlet/blob/849c5894f73d9732db9c2ccaa765385dc9d8a819/docs/YT-096-TWO-HOST-REBUILD.md)
now matches all nine Windows source archives exactly to the Linux originals,
with normal Cargo verification. Shared control/staged dependencies do not
satisfy independent builder evidence. Build-bound inventory and formal
trust/release/security/approval gates remain open. The
[earlier Azure SSH attempts](QUALIFICATION-MATRIX.md#historical-nine-package-candidate-platform-and-builder-evidence) remain historical
failures. The [Run Command/private Blob continuation](QUALIFICATION-MATRIX.md#historical-nine-package-candidate-platform-and-builder-evidence)
now passes actual Ubuntu 24.04.5 native execution, empty-cache nine-archive
rebuilding, 83 native tests/four doctests and five wasm libraries. Actual Cargo
observations retain known dev-only SBOM differences and inventory limits.
All temporary resources are deleted; formal independent trust is not established.

The subsequently authorized [GitHub-hosted candidate rebuild](QUALIFICATION-MATRIX.md#historical-nine-package-candidate-platform-and-builder-evidence)
uses the existing manual workflow at immutable ce56e08. Run 39418279958 passes
with all nine archives byte-identical, 63 selected source inputs and exact
lock/epoch/tool matches; checksum and unsigned provenance bindings pass. This
adds separately operated builder execution evidence, while authenticated
builder/provenance/trust-policy acceptance and final security/release approval
remain open. No signing or publication occurred; API classifications remain
unchanged and YT-020 remains Blocked.

The [final documentary reconciliation](QUALIFICATION-MATRIX.md#historical-nine-package-candidate-platform-and-builder-evidence) now
joins nine source archives/package locks to 23 SBOMs and the retained 36-entry
unsigned bundle; actual GNU/wasm Cargo observations have only known dev-only
differences. Existing native/wasm/Firefox and historical Chrome results are
retained without runtime reruns. Historical Chrome has no original independent
candidate-wide manifest, and complete compiled/per-library/system inventory is
Not verified. At that reconciliation checkpoint, support/inventory-limit
acceptance and remaining formal trust/registry/final review/maintainer gates
were open; the subsequent decisions below retain those evidence boundaries.

The subsequent [independent final candidate review](RELEASE-READINESS.md#historical-release-and-hosted-source-checkpoints)
is complete with overall FAIL and release-ready NO. It accepts bounded
technical/security evidence and identifies no new material source defect in the
inspected delta. At that review checkpoint, formal integrity/trust, final
support/inventory acceptance, actual registry proof, public activation controls
and concrete maintainer release approval were unmet. These are existing release gates;
the review authorized no repairs, reruns, signing or publication. YT-096 stays
In progress and YT-020 stays Blocked.

After the review report was committed as `2118495`, the owner explicitly
accepted the [bounded support scope and known inventory differences](QUALIFICATION-MATRIX.md#historical-nine-package-candidate-platform-and-builder-evidence).
Missing evidence remains Not verified, including exact-candidate Chrome
qualification and complete compiled/binary/system inventory. This resolves
only the missing support/inventory disposition. The original final verdict
stays FAIL / release-ready NO; formal integrity/trust, actual registry proof,
public activation/reporting controls and concrete final maintainer approval
remain open. No policy, API classification, required CI or release authority
changes, and no runtime rerun, signing or publication occurs.

The accepted support/inventory update was committed as `0bd7596`.
[Formal trust/provenance research](https://github.com/bohewu/yetlet/blob/66de5b5794ee9587dfbfe5e5c629c572e1af7b8a/docs/YT-096-FORMAL-TRUST-PROPOSAL.md#historical-nine-package-proposal--2026-10-20) proposes
a conditional GitHub OIDC/public Sigstore route with exact identities,
approved roots, source-package provenance, a complete evidence manifest and
independent verification. Disclosure and a future workflow/source candidate
remain owner decisions; this research does not implement or satisfy the
formal trust gate. All four release conditions and existing evidence limits
remain unchanged, with no signing or publication.

## Current documentation and package boundary (2026-10-12)

YT-068 completes the developer entry, exactly nine package guides, evidence-preserving navigation and [T08 Private preparation reconciliation](RELEASE-SUPPLY-CHAIN.md#current-preparation-source-boundary-2026-10-12) at bounded documentation scope. The five research items YT-029/YT-021/YT-024/YT-022/YT-023 are Done with their linked [actual evidence and decisions](README.md#completed-research-prototypes), without supported-core promotion. T07's nine package-list checks passed; they establish README inclusion only. The independent delivery gate/result is recorded separately in the run; documentation-stage checks do not attest that gate.

Signed source `8e557e760407a3d4b3f5a8b8f833748946be1387`, policy HEAD `abb2cce1253afe1d87a97e0a50f3f47efffe7280` and the current HEAD-plus-dirty preparation checkout are distinct. The historical three signatures and 37-artifact set remain valid historical Private evidence; they do not attest new candidate READMEs, root manifest/lock changes or research. Historical compiler/browser/CSP/CI and audit results below retain their original identities; they are not fresh qualification or an audit of the current research dependency graph. New-source package preflight, rebuild/provenance, selected SBOM/license evidence and signing verification require a separately approved immutable candidate and execution authority. No such rebuild/signing or broad qualification is run for this documentation phase. YT-020 remains Blocked and a later local commit alone does not activate release.

## M11 planning boundary (2026-10-12)

[M11](ARCHITECTURE.md#m11-product-and-renderer-decisions) separately proposes product-readiness and an official opt-in Experimental `yetlet-wgpu` line. This documentation filing does not implement a product/renderer, expand the original nine release candidates, change YT-020 or qualify an operating environment. Current source inspection starts from clean `c7ecc49eeb667e145cfb7e764c10b72ba104bbee`; historical dirty/signing/CI statements retain their original checkpoints. Lack of public publication does not itself prevent an explicitly approved immutable-source private deployment, and publication/signatures do not establish runtime correctness. [Product research](ARCHITECTURE.md#m11-product-and-renderer-decisions) defines the finite application evidence and owner responsibilities required before such a scoped claim.

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
boundary](https://github.com/bohewu/yetlet/blob/4ad0978f8a70b36c307f5e538aa1a3aa496a7403/docs/YT-066-PERSISTENCE-RECOVERY-PLAN.md). YT-020 remains Blocked for its
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
| Credentialed CSRF/Session | **Prototype; single-host limitation observed** | Bounded in-memory Session/CSRF works. YT-066a three-instance Docker execution proves stateless routing but also proves Memory Session is rejected cross-instance and lost on issuer restart; final independent review passed. The separate experimental Redis provider has reviewed b2 implementation and [b3](QUALIFICATION-MATRIX.md#historical-session-auth-oidc-and-cross-host-evidence) passing single-host/no-persistence execution, cleanup and final independent review; broader deployment durability/recovery is unverified and application-specific, not a bounded YT-066 completion prerequisite |
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

**YT-202 readiness correction (2026-10-27):** [Remaining native cells](QUALIFICATION-MATRIX.md#historical-sixteen-package-qualification-and-wasm-proof) confirms the reconnected Windows MSVC toolchain/VC Tools/SDK are now ready. Accepted GNU/package results remain unchanged and are not replayed; fourteen-package default and selected-feature MSVC consumer executions are still Not run. The earlier artifact source-read policy rejection has not been resolved; an admin-only diagnostic is unavailable to the current connection, and no exact policy change is proposed. YT-202 stays Blocked until admitted source delivery, both executions and final independent review. PR #12 now contains4e5ef64 with green RustSec/MSRV/native/browser CI and remains unmerged; that hosted Linux result does not replace these Windows cells or select a new final release candidate. This continuation performs no Commit/Push/Merge.

**YT-202 authorized source-route update (2026-10-27):** The owner explicitly selects Git clone for the remaining Windows checks. [The new task-owned clone](QUALIFICATION-MATRIX.md#historical-sixteen-package-qualification-and-wasm-proof) is admitted and fixed at74f53fe/tree2faa, matching accepted YT-201 source; no rejected artifact is relocated or transferred. Windows prerequisites are ready; preparation uses the existing Cargo bootstrap/directory-source convention and a newly bound equivalent fourteen-package consumer. The earlier artifact/admin diagnostic blockers are historical. YT-202 is In progress, with both MSVC executions and final independent review pending; accepted GNU/package/provider/browser evidence is retained without replay. No new Commit/Push/Merge or final release selection.

**YT-202 MSVC execution (2026-10-27):** [Two new native consumers](QUALIFICATION-MATRIX.md#historical-sixteen-package-qualification-and-wasm-proof) PASS on prepared Rust1.99MSVC/VC14.44/SDK26100: fourteen default package dependencies, then Auth/session and Config/secret-provider with missing-Memory-layer refusal and CPU-only wgpu. Inputs/lock remain unchanged; all40 package-file hashes reconcile to committed CRLF checkout and100 external lock identities are retained, without a Linux archive-byte-identity claim. The fresh fixture dependency-preparation failure and environment-observation correction remain recorded; no passing native run or old suite was replayed. Six task-owned Windows directories are removed and evidence retained. YT-202 awaits final independent review before Done; no Commit/Push/Merge or release activation.

**YT-202 final native acceptance (2026-10-27):** [Final independent Reviewer PASS](QUALIFICATION-MATRIX.md#historical-sixteen-package-qualification-and-wasm-proof), zero issues/required followups, closes YT-202 at selected native/MSRV/feature scope. Both fourteen-package MSVC consumers pass with exact source/feature/lock/tool/result/cleanup bindings;4documentation/30new/32retained evidence hashes and24ZIP members match. Original request records Rust1.99MSVC selection; generic non-Git source freshness and directory-source/CRLF limits remain explicit. Prior failures/reviews and accepted GNU evidence stay intact without replay. Four task documents remain uncommitted; no Commit/Push/Merge or release activation. YT-203 is next; YT-204/205, YT-096/D1–D6 and YT-020 gates remain separate.

**YT-203 source-bound browser continuation (2026-10-27):** [WASM/DOM/optional GPU inventory](QUALIFICATION-MATRIX.md#historical-sixteen-package-qualification-and-wasm-proof) reuses accepted CI4e5ef64 original-five/MSRV/Chrome evidence, unchanged DOM contracts and YT-081 actual crate GPU inputs; Lab and Tasks remain separate application evidence. Only missing normalized six-package WASM default/browser compile and current-package Firefox154 DOM cells execute: both Cargo checks and23new controls/navigation/CSP/atomic-refusal observations PASS. Source/lock/assets,34-node default isolation and owned cleanup are retained; directory-source/old-GPU-lock/compile-only and unavailable historical tmp limits remain explicit. Final independent Reviewer is pending before YT-203 Done. No new runtime/sample, old suite replay, new-change Commit/Push/Merge or release activation.

**YT-203 final browser acceptance (2026-10-27):** [Independent Reviewer PASS](QUALIFICATION-MATRIX.md#historical-sixteen-package-qualification-and-wasm-proof), zero issues/required followups, closes the selected packaged WASM/DOM/optional GPU evidence gaps. All4 candidate docs,37 evidence/fixture files,17 protected inputs and6 packaged-source comparisons verify; original GPU result/review and28 retained inputs reconcile to the six disclosed changed wrappers. New default Wasm build, browser-feature compile and23 Firefox observations pass with owned cleanup; accepted Chrome/GPU/BFCache proof is reused without replay or scope expansion. Completion metadata alone follows acceptance. Four YT-203 documents remain uncommitted; no Push/Merge, runtime feature or release activation. YT-204 remains Todo and is next; YT-205, YT-096/D1–D6 and YT-020 gates remain separate.

**YT-204 inventory/security continuation (2026-10-27):** After strict/builtin guarded YT-203 commitf31eed7 without Push, [current selected coverage](RELEASE-SUPPLY-CHAIN.md#historical-license-and-advisory-qualification) records42 cells:6 exact-input inventories retained without replay and36 new library SBOM closures matching metadata.31 new license/source cells pass;5 fail unchanged policy for Zlib/BSL-1.0/ISC.16 current-db package-lock audits report0 vulnerabilities/exit0 while10 retain Tokio informational warning; no ignore/upgrade. Scoped Standard source scan63212d73 completes120 regular files/18 aliases with0 reportable findings, preserving old Low/remediation/Auth identities and application/system limits. Delivery review is pending; YT-204 stays In progress for an authorized narrow license decision and successful affected checks. No runtime/lock/policy change, old qualification replay or new-change Commit/Push/Merge; YT-205 is unopened and formal release gates remain separate.

**YT-204 bounded delivery acceptance (2026-10-27):** [Independent Reviewer PASS](RELEASE-SUPPLY-CHAIN.md#historical-license-and-advisory-qualification), zero issues/required followups, verifies4docs/5canonical/6supplemental/138source bindings, sixteen archives, six retained cells and all36 new closures. It accepts truthful inventory/gap reporting and explicitly keeps YT-204 In progress for the five failed license-policy checks; no policy amendment or release approval is implied. Four owned extraction/cache/database directories are removed after review, raw inert evidence and sealed bindings retained. Four task documents remain uncommitted, staged0; no Push/Merge or old qualification replay. YT-205 and formal release gates remain unchanged.

**YT-204 authorized policy followup (2026-10-27):** The owner approves only Zlib/BSL-1.0/ISC additions to the license allowlist, notice preservation and the five failed checks. [All five reruns PASS](RELEASE-SUPPLY-CHAIN.md#historical-license-and-advisory-qualification), zero license/source errors,39 retained input and10 notice hashes unchanged; six unused-allowance warnings remain. All other policy fields, manifests/locks/runtime and prior failures/scan/advisory evidence stay intact.42/42 selected coverage combines prior passes and these five outcomes; no other successful check is replayed. Policy-wide versus per-component distribution scope is documented without relicensing Yetlet. YT-204 remains In progress pending final independent review; no ignore, upgrade or Commit/Push/Merge.

**YT-204 final policy acceptance (2026-10-27):** [Independent Reviewer PASS](RELEASE-SUPPLY-CHAIN.md#historical-license-and-advisory-qualification), zero issues/required followups, closes the authorized license blocker and marks YT-204 Done at selected inventory/security scope. Six candidates/five followup evidence bindings,39 inputs/10 notices/138 source-and-alias bindings and prior canonical/supplemental records verify; all five affected checks pass under the sole three-ID allowlist delta. Other policy, dependency/runtime and historical warning/failure identities remain intact. Restored package trees and owned Cargo home are removed, raw evidence preserved. Six paths stay uncommitted/staged0; no other passed-check replay, Commit/Push/Merge or release activation. YT-205 is next but unopened; formal release gates remain separate.

**YT-205 independent checkpoint acceptance (2026-10-27):** [Read-only Reviewer PASS](https://github.com/bohewu/yetlet/blob/c440c4542c888671750738a5908a667d2124db15/docs/YT-205-RELEASE-EVIDENCE-CONSOLIDATION.md#independent-checkpoint-acceptance--2026-10-27), zero issues/packet repair requirements, verifies10document/185evidence/1179protected bindings,16clean C archives and their retained/new proof distinctions. The reviewer explicitly keeps item_can_be_done:false for current-source CI/rebuild, affected native dependency evidence and reader-source disposition; YT-205 remains In progress. Owned caches/extracted packages are removed; archives, failure logs, SBOMs and rustdoc remain. No product replay or new-change Commit/Push/Merge, and D1-D6 remain Pending.

**YT-205 current-lock GNU delta (2026-10-27):** [One additional Ubuntu24/OpenSSL3.5.9 integration run](https://github.com/bohewu/yetlet/blob/c440c4542c888671750738a5908a667d2124db15/docs/YT-205-RELEASE-EVIDENCE-CONSOLIDATION.md#once-only-current-lock-ubuntu24-continuation) PASS at C/Tokio1.53.3 with unchanged source/lock and observed Rust1.99/glibc2.39. It closes only this selected native dependency gap; no old suite is replayed. Owned image/container/vendor/build inputs are removed and all raw evidence retained. The earlier checkpoint PASS remains intact; separate independent GNU delta review PASS0 verifies10docs/12additional records, original185records unchanged, and explicitly keeps item_can_be_done:false. Current-source hosted CI/sixteen-package rebuild, MSVC dependency evidence and reader-source disposition keep YT-205 In progress; no Commit/Push/Merge or formal decision.

**YT-205 rebuild preparation (2026-10-27):** After strict/builtin guarded ten-document commit012ecb3 without Push, [the bounded builder extension](https://github.com/bohewu/yetlet/blob/849c5894f73d9732db9c2ccaa765385dc9d8a819/docs/YT-205-SIXTEEN-PACKAGE-REBUILD.md) preserves original-nine mode and adds explicit sixteen selection, six manifest/lock roots, separate bootstrap/normal verification and conditional pinned OpenSSL3.5.9 preparation. Five focused tests, native compile/lint/format and static manual-workflow checks PASS; expected fixture compile rejection retained and old product proofs not replayed. Independent review is pending. No hosted workflow/OpenSSL setup execution, successor candidate, new-change Commit/Push/Merge or D1-D6 decision; whole YT-205 remains In progress.

**YT-205 rebuild extension acceptance (2026-10-27):** [Independent Reviewer PASS](https://github.com/bohewu/yetlet/blob/849c5894f73d9732db9c2ccaa765385dc9d8a819/docs/YT-205-SIXTEEN-PACKAGE-REBUILD.md#independent-acceptance-and-remaining-gate), zero issues/required followups, verifies10candidate/28evidence/1181protected hashes and18 unchanged symlink targets. Five focused controls and compile/lint/format/workflow/docs checks pass; fixtures are removed and intentional rejection logs retained. Acceptance is local preparation only: actual hosted CI/rebuild, MSVC and final reader/source conditions remain open, so wholeYT205_can_be_done:false. New code/docs remain uncommitted/staged0; no Push/Merge/dispatch, candidate change or D1-D6 disposition.

**YT-205 current-C technical execution (2026-10-27):** Authorized strict/builtin
Push updates PR12 to118f3d0 after5commits/56file operations/0findings and native
hook PASS. [New execution evidence](RELEASE-READINESS.md#historical-release-and-hosted-source-checkpoints)
records both required CI jobs PASS with a C-equivalent synthetic merge tree,
explicit sixteen hosted/local rebuilds with all33 phases PASS each and all16
archives byte-identical, and two affected current-lock MSVC consumer PASS.
Forty-six reader sources now match the accepted draft at C; prior input-bound
proof is retained without manual replay. Owned scratch is removed. Windows
original text evidence remains readable at its source; a denied local ZIP transfer
is preserved without alternate transfer or a local-receipt claim. Final independent
Reviewer PASS0 accepts the actual local/remote evidence and closes YT-205 at
selected technical/reader scope. Seven document paths stay
uncommitted/staged0; no Merge/signing/publication and D1–D6 Pending.

## Alpha.1 frozen inputs, signatures and acceptance — 2026-10-29

This historical sixteen-source-package acceptance precedes the current published
SDKalpha.8/CLI0.4.0 pair. The adopted
[two-payload contract](RELEASE-SUPPLY-CHAIN.md#two-payloads-complete-assembly-and-independent-verification)
preserves independent builder/signer identities and all earlier failures.

| Immutable input | Actual identity |
|---|---|
| Build C/tree | `bf3363801132f4481263ceb411d9bba3349df86d` / `c4ee13aa558489ba3962874dbf5e0a0a1bc99605` |
| Product/build | Unsigned immutable `v0.9.0-alpha.1`, release.yml,40669449040/1, protected verifier40688772221/1 |
| Signer S/tree | `d65e3180ad3076659f03c069fc71cd0e7e3f0518` / `5812dcfbc9e50a76c4d230bd5d32a336568db12a`, matching reviewed a29796b |
| Successful prepare | 40806988578/1, preflight123228194489, protected123228240593, sign123228242222 skipped |
| Preparation artifact | 12867774698, `signing-inputs-40806988578-1`,74,815,047bytes, ZIP `bee5cb493b35ea681a20054e7a095b13989d523c3f7a8dd17a4c3cbd9c8d23c8` |
| Raw safe preparation receipt | `f7a0cc5df0f471f3d47526fb8e41822a647e51519124ec43fddd338f5e9570a9` |
| Frozen ciphertext | `89cbc583a9c27f4cd4c1c3350ae3c1cd47ca241458b1920adac237adc2143e52` |
| Closure map | `a13ccb85554d4ab68f7aaf95bd151758841a03392496a59685a4adf1bce2760c` |
| Public raw provenance | 9,158bytes, `4d4de1a8ce9cc077c0aa62a1aafb3ab9799eed312deddb66d9e6f8fc6fef2d68` |
| Final raw private manifest | `e0921075e47a5c2f16948dde94bf006ec8afac14c8730ba68b22d28d0533d93f` |
| Original private build manifest | `9585635674d073b740b83ccd4d1e7edf1e60aab52e1b0a7efff8102a911cc4ec`, separately authenticated input |

Old0.1.0 C `3eacfe31f5e3281411d3bc4653a6baccab668b30`, tree
`86385d60cb4c3e595c670dbacce117ebeeb1a993`, build40628943583/1 and verifier40631789238/1
retain distinct unsigned provenance9,022bytes,
`b3a1e66a34d316c6513ff4e0b9c8055c37b8f81daa598a8110f867e911a400fc`.
The earlier118f3d0 baseline archive table cannot identify either later output set.
Original sixteen archive sizes/digests and proposal evidence hashes remain in the
[fixed original D1–D5 source](https://github.com/bohewu/yetlet/blob/66de5b5794ee9587dfbfe5e5c629c572e1af7b8a/docs/YT-096-D1-D5-RELEASE-TRUST-PLAN.md#d3--candidate-source-and-exact-package-scope).
This Git object/heading is locally verified. Remote GitHub reachability is
unverified. Git cannot recover separately ignored/external raw signatures.

The successful prepare passed authenticated tools, original bindings, owner
Environment25580949441 approval by user35250002, isolated handoff/decryption,
full checksum/subject checks, encrypted freeze/upload and final cleanup.
Owner-local readback matched both transport files and every safe binding without
local decryption. Custody42files/168,101,480bytes had index
`46eb08c7cbccac1f569d0505c6fad7749e5166cb3b20958622c40c27370f0140`.
The artifact's recorded expiry was2026-11-05 10:35:06 Asia/Taipei. Retention proved
that owner-local copy at observation time. It granted no later resurrection or
indefinite/off-host availability. Only after successful retention was temporary
qualification asset685300857 deleted with204/zero-assets Draft readback.
Original encrypted copies and failed-run records remained.

The owner accepted the exact D6 packet by reference and personally approved one
sign dispatch40811275645/1 at unchanged S. Preflight123241908097 and protected
job123241955416 passed tool authentication, exact frozen inputs, identity removal
before OIDC, both signatures, normal verification, encrypted upload and cleanup.
Prepare123241956871 was skipped. There was no signing retry or new product build.

| Actual signed output | SHA-256 |
|---|---|
| Artifact12870551335, `signing-results-40811275645-1`,20,484-byte ZIP | `27ea5d5315f17ca9728943c944db653b6f56e2297a4efded76ded4b7c170eef7` |
| Raw `SIGNING-EXECUTION.json` | `8a7648ed0fd13c8fa065289514b84a10299c9394bc0b27e5d1c4b92d94e6f2ac` |
| Public provenance bundle | `524c4f2aa6c611959d3c3b6acf6e27dce3e0a55cc58c68a80513345d20434a21` |
| Encrypted private manifest bundle | `37608522d8c8162d62052b7d3fe918ae1d1ac5014c148564fabf36923d4700b8` |
| Transient raw private bundle | `8a365ad5c7f530e98b08b5a26aa70c0d1d0db764bbc701d571f525b70c6618e0` |
| Public certificate | `b490d4b3a134c3191a573b7ad386dd326826f8b48b54d172c5b151a610f6c12e` |
| Private-signature certificate | `5fce264e6b9f2a66437d10fc572a5fa4c334d5213a429e825d7f425744dd249a` |

Independent public verification checked chain/CT/log/timestamp, exact
issuer/SAN/S/source-config/repository/owner/Environment/invocation, original DSSE
bytes and all16archives. A missing interpreter interrupted only a not-started
certificate wrapper, completed with system cryptography without replaying passes.
Private verification used one explicitly authorized owner-local identity read,
300seconds maximum. It passed all117complete-manifest entries. A bookkeeping
`name` versus `path` KeyError occurred afterward. One correction continued
not-started checks within the same146.12-second phase, checking closure/subjects
and independent private Cosign signature/claims. Both temporary plaintext
directories were removed, identity metadata unchanged. No key copy/export/upload,
retained private plaintext or repeated successful cryptographic check occurred.
Rekor indices174117699/174117716 and one RFC3161 timestamp per bundle were verified.

Final Reviewer PASS0 activated the owner's conditional acceptance of the already
fixed public/encrypted-private output hashes. D6 Complete/YT-096 Done covered
only these Experimental source-package signing/integrity outputs. The local-key
exception was spent. Future D2 custody uses the approved protected-job policy.
Original issuance receipts' Pending fields stayed immutable. Final acceptance
and review were external records. Final custody36files/134,686bytes had index
`e940be86c4d121696c93a4465593624c01208306e336cedfc3bb6a2946952d9d`,
preserving earlier30-file index
`704c41a65f8db02012b798530486cd4180a4c9fc2572f9d8d480d5ba595dd439`.

Separate owner approval published Release426670086 at2026-10-29 12:09:39
Asia/Taipei, draft=false/prerelease=true/make_latest=false, unchanged C/tag/S.
Exactly20public attachments totaled358,948bytes, comprising16archives, original
public provenance/checksum, public bundle and reduced `SIGNING-APPROVAL.json`.
Private manifest/body/bundle and raw receipts were excluded. Generated GitHub
source archives were outside signed subjects. Approved note hash was
`f16a974b7136e27202db42c7eca7b6c4f4b201069c5bfc9f85caac2bcd04c6ce`.
Authenticated/anonymous API and actual bundle download matched all names/sizes/
digests. Initial draft update omitted tag_name and asset lookup failed before
uploads. Exact tag/C correction reused the same Release, uploaded each asset
once and made one publication PATCH. No credential/permission change or re-signing
occurred. Publication custody56files/572,686bytes had index
`677f68d2eb14e0b23b432d1025b6f3b8a33927848f66c9c73adb20c4d1aba63e`.
No crates.io upload occurred at this checkpoint. YT-020 remained Blocked then.
This history cannot undo later independently accepted registry publications.

Full dated preflight, approval, failure and custody provenance remains in the
[original signer source](https://github.com/bohewu/yetlet/blob/66de5b5794ee9587dfbfe5e5c629c572e1af7b8a/docs/YT-096-SIGNER-CONTRACT.md#actual-signing-independent-verification-and-owner-acceptance--2026-10-29),
locally verified with unverified remote reachability. Raw logs/signatures remain
under recorded `target/yt096-*` and repository-external owner custody. Markdown
recovery provides their references and historical claims, without new raw-byte
availability verification or renewed execution authority.
