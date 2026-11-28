# Yetlet Prototype Release Readiness

**Current distribution (2026-11-23):** **Published SDK:** `0.9.0-alpha.9` (`yetlet` / `yetlet-browser`).
**Published CLI:** `0.4.2` (`yetlet-cli`). The accepted
[SDK publication record](ALPHA9-HOSTED-RECORD.md#owner-approved-publication-and-registry-consumer--2026-11-23)
and [CLI publication record](CLI-042-HOSTED-RECORD.md#owner-approved-publication-and-installed-consumer--2026-11-23)
record actual GitHub/crates.io delivery, exact archives, signatures and consumer limits.
SDK alpha.9 includes empty-default Core composition, optional View/HTML
and the Web namespace. The independent CLI resolves each app's SDK and records
exact pins and locks. It builds against alpha.8 and generates scaffolds for
alpha.9. Both published distributions passed their registry consumers. The
Reader Edition source selects SDK alpha.9 and CLI 0.4.2. The
[live Pages readback](GITHUB-PAGES-PUBLICATION.md#sdk-alpha9-and-cli-042-reader-rollover--2026-11-23) matched the eight chapters and six assets. Development version equality
does not establish archive/source equality. This is not Stable API, production, SLSA
level or universal platform qualification.

## SDK 1.0 release candidate acceptance

**Status:** Local source freeze for SDK `1.0.0-rc.1`. The rehearsal started from
verified Git commit `f3920d45b0c071976163aa8fa09c5719643262dd`. The two SDK
packages inherit `0.9.0-alpha.9` at that commit. The release source adds the SDK
version synchronization, including the exact Browser-to-Core pin, affected
lock entries and release documentation required by the version change. The
rehearsal record identifies the tested delta and rehearsal archive digests. The
owner approved the 1.0 compatibility scope below and the local release commit.
Final archive qualification and publication remain open.

The published alpha.9 pair is the migration baseline. Use the independent CLI
0.4.2 release to verify the consumer path.

### Approved 1.0 compatibility scope

Apply SemVer to the documented Rust APIs and behavior below for the `yetlet`
and `yetlet-browser` package pair. Retain Rust 1.99 as the minimum supported
version. The initial target is `x86_64-unknown-linux-gnu` for native
Core, View/HTML, Reactive and Web, and `wasm32-unknown-unknown` in Chrome on
Linux for portable Core, View/HTML, Reactive and Browser DOM. Other target and
browser support requires separate qualification. The approved scope follows the
current Core/View/Web sections of [Compatibility](COMPATIBILITY.md) and the
implementation checkpoints in [ADR 0014](adr/0014-layered-host-resource-lifecycle.md).

| Package and feature | Commitment |
|---|---|
| `yetlet` Core (`default = []`) | Built-in `HostGroup`, `Host` and `ResourceGroup` composition with caller-owned drive. Retain startup and observed readiness, rollback, stop/drain, completion and final release, including the retained owner's ability to resume cleanup after a cancelled borrowing drive. Core starts no runtime. |
| `yetlet/view` and `yetlet/html` | Shared `View`, `Element`, `Attribute`, typed tags and binding APIs. `html` enables `view` and adds escaped HTML serialization. View and HTML remain independent of HTTP and Browser. |
| `yetlet/web` | Native API-only HTTP and managed `App`/`WebHost` startup and shutdown. Retain `App::into_router()` and the documented Axum/Tower interoperability boundary. Axum, Tower and Tokio own their upstream APIs and runtime policies. `web` works independently of HTML. Adding `html` enables page and document rendering with usable server-rendered output when JavaScript or Wasm is unavailable. |
| `yetlet/reactive` | The Browser starter uses the optional `LocalGraph` API. Retain synchronous reducer/commit ordering, snapshots, weak handles and owned subscriptions for local updates. |
| `yetlet-browser/dom` (default) and `yetlet-browser/dom-keyed` | Retain shared View reexports, subtree mount and DOM admission/URL policies with `dom`, and keyed reconciliation, retained entry ownership and listener cleanup with `dom-keyed`. Both independently enable `yetlet/view`. Reordering uses `moveBefore()` where available and retains the `insertBefore()` fallback, which can interrupt focus or selection. |

Other opt-ins remain experimental under this scope. For `yetlet`, these are
`sfc`, `actions`, `actions-http`, `command-bridge`, `udp`, `native-winit`,
`renderer-wgpu`, `web-tls`, `acme`, `request-id`, `session`, `auth`,
`auth-session`, `session-redis`, `session-redis-tls`, `config`, `secrets`,
`secret-provider`, `signing` and `rate-limit`. For `yetlet-browser`, they are
`actions`, `wgpu` and `wgpu-browser`.

| Area | Required acceptance evidence |
|---|---|
| Public contract | Compare the candidate manifests and documented API paths with the approved feature boundaries and minimum Rust version above. |
| Lifecycle and service | Verify readiness, rollback, draining and resource release for the documented Host paths. The [external Web Host consumer](../examples/external-consumers/web-hosts/README.md) covers shutdown with an active SSE stream and a task awaiting a TCP receipt. Use the [Production Starter controlled-arrival scenario](../examples/production-starter/deployment/performance/README.md) for bounded load and recovery evidence, with the workload, limits and observed errors recorded. |
| Clean consumers and upgrade | Build the `yetlet` archive first, then `yetlet-browser` against that exact Core archive and checksum. Build clean Native, Service and Browser consumers from the pair. Migrate an existing alpha.9 app, recording changed imports, features and lock entries. Check CLI 0.4.2 with explicit `--sdk-version` and its default resolver against the candidate metadata. A higher non-yanked prerelease can become the default selection, so verify the chosen version and generated locks. |
| Browser and native targets | On a supported browser, verify the generated Wasm app, the no-JS SSR path, CSP and keyed reordering. Qualify native behavior on the platforms in the selected support scope. Reuse exact-source results where they cover the same contract and record unverified limits in the [qualification matrix](QUALIFICATION-MATRIX.md). |

Before selecting the candidate commit or packaging, run the
[release documentation gate](RELEASING.md#release-documentation-gate) and review
the package guides, capability map and upgrade instructions against the selected
scope. Use [the per-version publication procedure](RELEASE-PUBLICATION.md#each-version)
for exact-source CI, artifact verification and owner approval. Record the commit,
toolchain, archive digests and consumer outcomes together. Public registry
consumers and the Reader Edition update follow verified publication.

## M28 release preparation — 2026-11-22

At the 2026-11-22 checkpoint, the local SDK `0.9.0-alpha.9` and CLI `0.4.2`
had passed scoped archive checks. The rollout planned CLI first, then SDK
Core and Browser. The accepted records above document the subsequent
publication and registry verification.
[Release handoff and notes](ALPHA9-RELEASE-PREPARATION.md).

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

The 2026-10-26 YT-199 documentation-only review compared the fixed nine-library
`0.1.0` candidate with seven additional source-only opt-in workspaces. All sixteen
were unpublished then, YT-020 was Blocked and YT-096 trust gates incomplete.
Exact local Git `42d8856b9acf5ddddab9a4b998b5bdd4c5e59210` retains that dated
comparison, with remote reachability unverified. It selected no archives,
registry consumer or new candidate. Current publication is the two-package
SDK alpha.9 / independent CLI 0.4.2 above. Optional app-owned Auth/Session/cache/
renderer source proof and earlier signing cannot qualify another checkout,
browser/platform or production deployment.
[Compatibility](COMPATIBILITY.md) and [release SOP](RELEASING.md).

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

The dated progression below preserves unresolved and accepted states separately.
Exact source/test/platform evidence belongs to the linked canonical guides.
The [alpha.1 frozen inputs and signatures](#alpha1-frozen-inputs-signatures-and-acceptance--2026-10-29)
record the later D6 acceptance. Current publication remains the pair above.

| Checkpoint | State and remaining source-bound gates |
|---|---|
| 2026-10-28, alpha.1 selection | Owner selected 59-path5d7b91842f8188d45e5af22367245aebec7ca7d1/tree6adff26fc4fec65e9aa8f40d7172ff266b7e500d for sixteen alpha.1 packages. Fixed-binding patch, consumed Guard disposition, execution S/tag/payload hashes and separate authorization were still pending. Earlier C3eacfe3/v0.1.0 custody stayed distinct; D6 Pending/YT-096 In progress/YT-020 Blocked. [Original version preflight](https://github.com/bohewu/yetlet/blob/849c5894f73d9732db9c2ccaa765385dc9d8a819/docs/YT-096-ALPHA-VERSION-PREFLIGHT.md) |
| 2026-10-27, YT-205 a2d6474 | Guarded28-path local commit selected sixteen0.1.0 libraries,10 new changed-lock package checks/six compilation reuses, external integration consumer, eight rustdoc commands and14 inventory cells.28 inventories and prior proof retained. Current-source CI/rebuild, affected MSVC evidence and C-versus-draft guidance stayed open. [Historical consolidation](RELEASE-SUPPLY-CHAIN.md#historical-candidate-inventory-and-technical-consolidation) |
| 2026-10-27, YT-205118f3d0 | Guarded10-path source/tree equality selected new C118f3d0f496b487ef20fe243f4e73179f9eadc43. Exact-source CI, explicit sixteen-cohort hosted/local comparison, affected14-package MSVC and final review were ordered gates. Their later PASS closed only technical/reader scope, preserving earlier item_can_be_done:false and D1–D6 Pending. [Actual CI/rebuild/acceptance](#historical-release-and-hosted-source-checkpoints) |
| 2026-10-26, YT-202 | Four source-bound GNU default graphs/two Ubuntu24/OpenSSL3.5.9 consumers passed. Windows lacked toolchain/VC Tools/SDK then; YT-202 Blocked, independent review accepted GNU/documentation only. Local8fd8751/same-tree74f53fe were stopped by both full-history Push Guard routes. [Native source/transfer/failure history](QUALIFICATION-MATRIX.md#historical-sixteen-package-qualification-and-wasm-proof) |
| 2026-10-26, YT-201 | Publication metadata/versioned dependencies/license/guides, sixteen normal packages and two isolated native consumers passed local preflight/review on Ubuntu26.04.1. Hash-bound uncommitted inputs selected no C or registry activation; ce56e08/YT-096/D1–D6/YT-020 gates remained. [Sixteen-package proof](QUALIFICATION-MATRIX.md#historical-sixteen-package-qualification-and-wasm-proof) |
| 2026-10-26, YT-199/200 | Source64a6d1c versus unchanged nine-packagece56e08 and sixteen-library co-release planning had independent PASS only at documentation scope. YT-201–205 were registered Todo, with version/source/qualification/signing/public activation unselected. [Original199 comparison](https://github.com/bohewu/yetlet/blob/42d8856b9acf5ddddab9a4b998b5bdd4c5e59210/docs/YT-199-CURRENT-DELIVERY-READINESS.md), [original200 plan](https://github.com/bohewu/yetlet/blob/849c5894f73d9732db9c2ccaa765385dc9d8a819/docs/YT-200-SIXTEEN-PACKAGE-RELEASE-PLAN.md) |

Intermediate selection, no-execution/source-freeze and independent-review
boundaries remain at the locally verified
[original readiness checkpoint heading](https://github.com/bohewu/yetlet/blob/8d6fad7e95ee25849872b974f922d10c71b70a95/docs/RELEASE-READINESS.md#historical-readiness-checkpoints).
Remote reachability and raw ignored/external evidence availability are unverified.

**Date:** 2026-10-10
**Decision:** **M0–M9 are demonstrable and independently reviewed at prototype scope; public package publication and production deployment remain blocked.**

**Private preparation update (2026-10-11):** The maintainer selected experimental `0.1.0` and the original nine library candidates. [YT-020 preparation](QUALIFICATION-MATRIX.md#private-nine-package-preparation-and-verifier-fixtures--2026-10-11-and-12) records the prepared compatibility/maintenance scope, package metadata, actual local preflight and remaining activation gates. YT-020 remains Blocked; this is not publication approval or a visibility change.

**Integrity update (2026-10-12):** The maintainer selected Rust 1.99 before any publication. [Private integrity preparation](QUALIFICATION-MATRIX.md#private-nine-package-preparation-and-verifier-fixtures--2026-10-11-and-12) records two-host source-package rebuilding, inventories, unsigned provenance and public-fixture signature verification. This is still preparation under shared maintainer/agent control, not trusted release activation. No commit, push, tag, signing request, publication or visibility change is authorized for this follow-up.

**Explicit trust/registry decision (2026-10-12):** [Private trust policy](RELEASE-SUPPLY-CHAIN.md#private-maintainer-key-preparation-policy-and-historical-signatures) selects Cosign public-key preparation and future crates.io distribution. Key custody/fingerprint and actual signing/registry verification remain pending. Public-key preparation cannot satisfy or weaken the formal OIDC, certificate/chain, transparency/inclusion and authenticated-source/builder gates. This follow-up authorizes documentation and independent review only; YT-020 remains Blocked.

**Later authorized execution (2026-10-12):** [Private signing evidence](RELEASE-SUPPLY-CHAIN.md#private-maintainer-key-preparation-policy-and-historical-signatures) now records key/fingerprint enrollment, three preparation signatures and separate verification of 37 artifacts and six negative cases. Formal log-required verification correctly rejects this bundle. Earlier pending-key/signing statements describe their policy checkpoints; formal release guarantees and registry activation remain open. The policy was locally committed; subsequent execution documentation remains unstaged and no push/publication/visibility change occurred.

This document separates bounded repository evidence from a support or security commitment. See [M9 final evidence](API-CONTRACTS.md#original-inventory-acceptance-and-review-limits), [API contract classification](API-CONTRACTS.md) and the [post-M9 CI/security review](CI-SECURITY-REVIEW.md).

## Current candidate selection and local preflights (2026-10-20)

The owner selected C `ce56e0814e5423782440acd2c959ca777d572cd4`, nine experimental
`0.1.0` libraries and Rust1.99/1.99.0. Local Ubuntu26.04.1 normal archive and
four-test directory-source consumer checks passed. Twenty-three scoped SBOMs
with dev-feature over-inclusion,23 license/source checks, nine fresh-database
RustSec scans and the36-entry unsigned bundle retained their original inputs.
[Candidate inventory and technical consolidation](RELEASE-SUPPLY-CHAIN.md#historical-candidate-inventory-and-technical-consolidation).

Distinct platform lanes later passed 83 native tests/four doctests in
Ubuntu24.04.5 userspace on a shared kernel, five packaged wasm builds, the
source-reconciled Linux Firefox workflow and resumed Windows MSVC standalone
packages. Initial Windows cache failures remained. Windows/Linux source
archives matched, with shared controller/staged-dependency trust limits.
Azure SSH attempts failed before transfer; the separately authorized private
Blob/Run Command lane passed native/wasm/empty-cache rebuild and removed all
resources. GitHub39418279958 then rebuilt nine byte-identical archives from
63 source/lock/epoch/tool bindings. Source-package equality did not establish
compiled-binary/system inventory or authenticated builder trust.
[Exact environments, failed lanes and builder evidence](QUALIFICATION-MATRIX.md#historical-nine-package-candidate-platform-and-builder-evidence).

Reconciliation retained 133 browser inputs/eight assets, known dev-only and
non-Cargo inventory omissions, and exact-candidate Chrome **Not verified**.
Final independent review was **FAIL / release-ready NO**, with no new material
source defect found in the inspected delta. Five unmet gates were formal trust,
support/inventory disposition, registry proof, public activation/reporting and
concrete maintainer release approval. Owner acceptance after report2118495
resolved only support/inventory disposition. Missing Chrome/complete compiled
inventory stayed Not verified, and four gates remained open. Update0bd7596
recorded that decision. The later GitHub OIDC/Sigstore proposal was research,
with future source/disclosure/workflow approval still pending. YT-096 stayed
In progress and YT-020 Blocked at this date. The later alpha.1 acceptance and
current publications have distinct identities above.
[Original final verdict](#historical-release-and-hosted-source-checkpoints) ·
[Historical trust proposal](https://github.com/bohewu/yetlet/blob/66de5b5794ee9587dfbfe5e5c629c572e1af7b8a/docs/YT-096-FORMAL-TRUST-PROPOSAL.md#historical-nine-package-proposal--2026-10-20).

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

**YT-202 dated native-route progression (2026-10-27).** Reconnection established
Rust1.99MSVC/VC14.44/SDK26100 prerequisites while both14-package consumers were
still Not run. The earlier source-read policy rejection and unavailable admin
trace remained real blockers. PR12/4e5ef64 Linux CI PASS did not replace Windows
execution or select C. The owner then selected a task-owned Git clone fixed at
74f53fe/tree2faa, matching YT-201, without transferring the rejected artifact.
Both default and integrated consumers subsequently passed, covering Auth/session,
Config/secret-provider, missing-Memory refusal and CPU-only wgpu. Forty package
hashes reconciled to CRLF checkout and100 external lock identities stayed bound,
without Linux archive-byte equality. Fixture dependency-preparation failure and
environment-observation correction remained, with no passing-suite replay.
Six owned directories were removed. Separate final independent Reviewer PASS,
zero issues/followups, verified 4docs/30 new/32 retained records and24 ZIP members,
closing only selected native/MSRV/feature scope. Generic non-Git freshness and
directory-source/CRLF limits remained. At completion YT-203 was next, with
YT-204/205/YT-096/D1–D6/YT-020 gates separate.
[Native sources, refusals, execution and acceptance](QUALIFICATION-MATRIX.md#historical-sixteen-package-qualification-and-wasm-proof) ·
[Original dated scope history](https://github.com/bohewu/yetlet/blob/8d6fad7e95ee25849872b974f922d10c71b70a95/docs/RELEASE-READINESS.md#scope-protection).

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

This historical sixteen-source-package acceptance precedes the later published
SDK alpha.8 / CLI 0.4.1 pair. The adopted
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

## Public reader acceptance boundaries

YT-217's selected API comments and local mdBook0.5.4 preview passed at
Chromium/emulated-mobile scope. YT-218 separately accepted the first Pages
publication at 64d1074, and YT-226 accepted the Yetlet-built/Wasm-search cutover
at 0ae6b93 with run41289520993/1 and actual public-byte/MIME/reader observations.
Original failed/stopped harness and later passing proof retain their own
sources in [Pages history](GITHUB-PAGES-PUBLICATION.md#public-guide-architecture-and-original-local-acceptance).
They qualify neither physical mobile, other browsers, hosted rustdoc nor a new
product release. The current reader pair is selected in protected
site/pages.toml; live Pages require their own readback. Local documentation consolidation does not
advance the edition or replace live Pages.
