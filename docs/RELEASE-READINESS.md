# Yetlet Prototype Release Readiness

**Date:** 2026-10-10
**Decision:** **M0–M9 are demonstrable and independently reviewed at prototype scope; public package publication and production deployment remain blocked.**

**Private preparation update (2026-10-11):** The maintainer selected experimental `0.1.0` and the original nine library candidates. [YT-020 preparation](YT-020-PRIVATE-RELEASE-PREP.md) records the prepared compatibility/maintenance scope, package metadata, actual local preflight and remaining activation gates. YT-020 remains Blocked; this is not publication approval or a visibility change.

**Integrity update (2026-10-12):** The maintainer selected Rust 1.99 before any publication. [Private integrity preparation](YT-020-INTEGRITY-PREPARATION.md) records two-host source-package rebuilding, inventories, unsigned provenance and public-fixture signature verification. This is still preparation under shared maintainer/agent control, not trusted release activation. No commit, push, tag, signing request, publication or visibility change is authorized for this follow-up.

**Explicit trust/registry decision (2026-10-12):** [Private trust policy](YT-020-PRIVATE-TRUST-POLICY.md) selects Cosign public-key preparation and future crates.io distribution. Key custody/fingerprint and actual signing/registry verification remain pending. Public-key preparation cannot satisfy or weaken the formal OIDC, certificate/chain, transparency/inclusion and authenticated-source/builder gates. This follow-up authorizes documentation and independent review only; YT-020 remains Blocked.

**Later authorized execution (2026-10-12):** [Private signing evidence](YT-020-PRIVATE-SIGNING-VERIFICATION.md) now records key/fingerprint enrollment, three preparation signatures and separate verification of 37 artifacts and six negative cases. Formal log-required verification correctly rejects this bundle. Earlier pending-key/signing statements describe their policy checkpoints; formal release guarantees and registry activation remain open. The policy was locally committed; subsequent execution documentation remains unstaged and no push/publication/visibility change occurred.

This document separates bounded repository evidence from a support or security commitment. See [M9 final evidence](M9J-VERIFICATION.md), [API contract classification](API-CONTRACTS.md) and the [post-M9 CI/security review](CI-SECURITY-REVIEW.md).

## Current documentation and package boundary (2026-10-12)

YT-068 completes the developer entry, exactly nine package guides, evidence-preserving navigation and [T08 Private preparation reconciliation](YT-020-PRIVATE-RELEASE-PREP.md#current-preparation-boundary--t08--yt-068-2026-10-12) at bounded documentation scope. The five research items YT-029/YT-021/YT-024/YT-022/YT-023 are Done with their linked [actual evidence and decisions](README.md#completed-research-prototypes), without supported-core promotion. T07's nine package-list checks passed; they establish README inclusion only. The independent delivery gate/result is recorded separately in the run; documentation-stage checks do not attest that gate.

Signed source `8e557e760407a3d4b3f5a8b8f833748946be1387`, policy HEAD `abb2cce1253afe1d87a97e0a50f3f47efffe7280` and the current HEAD-plus-dirty preparation checkout are distinct. The historical three signatures and 37-artifact set remain valid historical Private evidence; they do not attest new candidate READMEs, root manifest/lock changes or research. Historical compiler/browser/CSP/CI and audit results below retain their original identities; they are not fresh qualification or an audit of the current research dependency graph. New-source package preflight, rebuild/provenance, selected SBOM/license evidence and signing verification require a separately approved immutable candidate and execution authority. No such rebuild/signing or broad qualification is run for this documentation phase. YT-020 remains Blocked and a later local commit alone does not activate release.

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
python3 scripts/check_docs.py
python3 scripts/check_samples.py

python3 scripts/build_browser.py --release
cargo build --release --locked -p yetlet-todo-server
python3 scripts/smoke_browser.py --profile release
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

**Backlog status:** M0–M9 and YT-060–YT-065 are Done at their bounded prototype/policy/design/automated-qualification scopes. YT-065's real-device/IME/assistive-technology/WebKit/mobile/zoom cells remain explicit optional unknowns; YT-066 is Done at single-host Docker multi-instance and selected Redis-provider scope; broader deployment durability/recovery remains Not verified. Application TLS/ACL policy, backup protection, RPO/RTO and operational staffing are not framework completion prerequisites. Revised YT-067 is Done at internal pre-patch source-scan and independently reviewed local Low finding correction scope. The earlier scan of an older snapshot remains incomplete, and no post-fix repository-wide rescan or production certification is claimed. YT-068 and the five authorized research studies are Done at their bounded documentation/prototype scopes: 67 of 68 backlog items are Done and YT-020 alone remains Blocked. The independent delivery gate/result is recorded separately in the run; documentation-stage checks do not attest that gate. No historical signature, local commit, review, license, checksum or green CI automatically authorizes current-artifact assurance, production or publication.
