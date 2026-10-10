# Yetlet Prototype Release Readiness

**Date:** 2026-10-10
**Decision:** **M0–M9 are demonstrable and independently reviewed at prototype scope; public package publication and production deployment remain blocked.**

This document separates bounded repository evidence from a support or security commitment. See [M9 final evidence](M9J-VERIFICATION.md), [API contract classification](API-CONTRACTS.md) and the [post-M9 CI/security review](CI-SECURITY-REVIEW.md).

## What is working

- Twelve runnable samples cover native SSR, typed APIs, opt-in Session/CSRF, middleware, files, direct SQLite, SSE, multiple browser islands, synthetic authorization and modular composition.
- Handler-first `App`, `RequestContext`, Tower interoperability and renderer-neutral core values are classified as Stable Candidates, not stable APIs.
- Complete M9 evidence retained 190 Rust tests, five doctests, 11 native smokes, eight wasm32 packages, Chrome and Firefox suites, dependency isolation and byte-identical production assets.
- Strict CSP, escaping, body/media limits, parameterized SQL, transactional rollback, bounded providers and application-level authorization have negative-test evidence.
- The declared MSRV is Rust 1.88. The complete native workspace, tests and doctests pass with that toolchain after upgrading transitive `time` to patched `0.3.47`.
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

| Gate | State | Detail |
|---|---|---|
| M0–M9 prototype contracts | Verified locally | Per-stage evidence and independent reviewers |
| Rust 1.88 MSRV | Verified locally; hosted job added | Native tests/doctests and wasm32 checks are required in CI |
| Current stable Rust | Verified locally | Format, strict Clippy, tests, doctests, smokes and wasm32 |
| Dependency vulnerability audit | Passing after repair | `RUSTSEC-2026-0009` fixed by `time 0.3.47`; Dependabot alerts/security fixes and per-CI `cargo-audit` are enabled, but no scheduled version-update PR configuration or broader continuous monitor exists |
| Hosted GitHub Actions | **Candidate, merged baseline and qualification verified** | PR run `38042614403` passed on `3128d23`; merged baseline `38043547271` passed both required jobs and qualification `38043565238` passed all four platform/Firefox jobs on identical-tree `c388378` |
| Platform/browser compatibility | **Automated lanes verified; manual coverage partial** | Hosted Linux, Windows MSVC, macOS ARM64 and Firefox four-suite execution PASS; local Windows GNU and offline Docker PASS. WebKit/mobile, real IME, screen readers and zoom remain unverified |
| Accessibility | **Partial** | Actual Tab/Enter/Unicode, labels/errors/focus and root-language checks pass; real IME, screen readers, zoom and conformance are not verified |
| Authentication/authorization | **Threat model specified; implementation prototype only** | M10d requires external OIDC plus server-side resource authorization; Auth Notes remains synthetic and is not an IdP/password/passkey/MFA design |
| Credentialed CSRF/Session | **Prototype only** | Bounded in-memory Session/CSRF works; no distributed provider or production topology |
| HTTPS/reverse-proxy trust and rate limiting | **Specified, not implemented** | M10d defines the trusted-edge/header and layered limiting contract; a selected deployment and abuse/load evidence remain required |
| Persistence/transactions | **Local qualification only** | Actual two-process local WAL conflict, online backup, exact isolated restore and corrupt candidate rejection PASS; encryption/retention, off-host durability and distributed production operations remain unqualified |
| Observability/load/cancellation | **Partial** | Lifecycle/cancellation tests exist; no production tracing, SLO, soak or incident evidence |
| Repository security controls | **Partial, private-state configured** | Collaborator-only PRs, protected `master`, required CI, SHA-pinned GitHub-owned Actions and Dependabot enabled; private vulnerability reporting, secret scanning and CodeQL unavailable until Public/current-plan support |
| Package license and vulnerability policy | **Selected and independently reviewed** | `MIT OR Apache-2.0`, `SECURITY.md` and maintainer-directed/no-external-PR policy PASS; GitHub private reporting must be enabled at Public transition |
| Stable API/SemVer/support policy | **Partial** | M10c defines a provisional one-release-train/change-discipline policy and additive document metadata; public activation still requires an explicit release decision and YT-064 supply-chain evidence |
| Artifact integrity/release supply chain | **Design/preflight complete; activation missing** | License/source policy, 41 native SBOMs and checksum tooling PASS; no signed provenance, independent reproducibility or immutable release promotion |
| Codex-assisted internal security assessment | **Done at internal source/fix scope** | Official Standard scan on pre-patch `25bff77` reviewed 227/227 regular files and found one Low issue; local control-character guard, Rust/MSRV/WASM/Chrome regressions and independent reviewer PASS. Original older-source scan incomplete; patched tree not rescanned; no external audit/certification |

## Decisions before publication

1. **Public-transition owner:** Enable and verify GitHub Private Vulnerability Reporting, secret scanning/push protection and CodeQL after changing visibility; preserve collaborator-only PRs.
2. **Maintainer:** Retain or revise the current zero-approval sole-maintainer PR rule when adding another maintainer, and define signed release-tag policy in the release-supply-chain gate.
3. **Maintainer/team:** Activate or revise the provisional SemVer/MSRV policy at public release and declare supported browsers/OS; the bounded title/language document contract remains Experimental pending more consumers.
4. **Security/team:** Implement and test the [M10d Threat Model](THREAT-MODEL.md) against selected external proxy/IdP/Session/limiter/secrets/storage products; YT-066 qualifies provider operations.
5. **Release owners:** Define reproducible artifacts, license/source policy, SBOM, checksums, provenance, signing, approval and rollback.
6. **QA/security owner:** Retain actual scheduled platform/browser evidence, qualify manual accessibility/IME separately, and require a complete internal security-scan report with residual-risk decisions. External assessment is optional and needs separate authorization.

## Scope protection

Yetlet keeps native HTTP, renderer, browser and optional provider dependencies separable. Auth, Session, database, files, SSE, tracing, CORS, rate limiting and deployment capabilities remain explicit opt-ins. The SQLite examples do not introduce an ORM, `DbContext`, repository framework or default data provider.

**Backlog status:** M0–M9 and YT-060–YT-064 are Done at their bounded prototype/policy/design scopes. YT-065 and YT-066 retain hosted/manual and distributed gates despite local reviewer PASS. Revised YT-067 is Done at internal pre-patch source-scan and independently reviewed local Low finding correction scope. The earlier scan of an older snapshot remains incomplete, and no post-fix repository-wide rescan or production certification is claimed. YT-020 remains Blocked; no local review, license, checksum or green CI automatically authorizes production or publication.
