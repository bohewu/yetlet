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
| Dependency vulnerability audit | Passing after repair | `RUSTSEC-2026-0009` fixed by `time 0.3.47`; continuous service still absent |
| Hosted GitHub Actions | **Pending new pushed run** | Earlier run failed only in Browser E2E harness; repaired workflow must reach terminal success |
| Browser compatibility | **Partial** | Chrome 155 and Firefox 157 retained; no WebKit/mobile/IME matrix |
| Accessibility | **Partial** | Semantic/error/focus tests exist; no complete screen-reader or IME audit |
| Authentication/authorization | **Prototype only** | Synthetic Auth Notes is not an IdP/password/passkey/MFA design |
| Credentialed CSRF/Session | **Prototype only** | Bounded in-memory Session/CSRF works; no distributed provider or production topology |
| HTTPS/reverse-proxy trust and rate limiting | **Missing** | No trusted-header, TLS termination or brute-force policy |
| Persistence/transactions | **Prototype only** | SQLite samples prove rollback/restart; no backup, encryption, retention or multi-node operations |
| Observability/load/cancellation | **Partial** | Lifecycle/cancellation tests exist; no production tracing, SLO, soak or incident evidence |
| Repository security controls | **Missing** | No protected branch; Dependabot, secret scanning and code scanning are disabled |
| Package license and vulnerability policy | **BLOCKED** | Maintainer must choose licenses and reporting channel |
| Stable API/SemVer/support policy | **BLOCKED** | Stable Candidate classification is not a compatibility commitment |
| Artifact integrity/release supply chain | **Missing** | No SBOM, provenance, signing or immutable promotion pipeline |
| Independent production security review | **Not performed** | Local role reviewers are not penetration testing or formal assurance |

## Decisions before publication

1. **Maintainer:** Choose repository/crate licensing and a supported vulnerability-reporting policy.
2. **Repository owner:** Protect `master`, require CI and decide dependency/secret/code scanning plus action restrictions.
3. **Maintainer/team:** Define SemVer scope, Rust upgrade policy, supported browsers/OS and the document language/head metadata contract.
4. **Security/team:** Define production authentication, authorization, TLS proxy trust, rate limiting, secret management, Session/storage topology and incident response.
5. **Release owners:** Define reproducible artifacts, license/source policy, SBOM, checksums, provenance, signing, approval and rollback.
6. **QA:** Add scheduled Firefox/additional-platform/accessibility/IME coverage and external security review after the production topology is fixed.

## Scope protection

Yetlet keeps native HTTP, renderer, browser and optional provider dependencies separable. Auth, Session, database, files, SSE, tracing, CORS, rate limiting and deployment capabilities remain explicit opt-ins. The SQLite examples do not introduce an ORM, `DbContext`, repository framework or default data provider.

**Backlog status:** M0–M9 are Done at prototype scope. `YT-020` remains **Blocked** and must not be closed by a green CI run alone.
