# Yetlet Prototype Release Readiness

**Date:** 2026-10-08
**Decision:** **Locally demonstrable MVP prototype, not approved for production or public package publication.**

This document separates completed local tests from genuine release certification. Refer to [M5 evidence](M5-VERIFICATION.md) and [HTTP pipeline](HTTP-PIPELINE.md).

## What is working

**M6 update (2026-10-09):** The full handler-first App/Page API, typed setup composition, optional request-ID proof and migrated Todo are locally complete; see [M6 evidence](M6-VERIFICATION.md). Existing licensing, MSRV, support and production/security blockers below remain unchanged.

- Native Rust/Axum HTTP server, ordinary JSON APIs, and script-free `GET /` SSR.
- `GET /app` Browser WASM with independent local and explicit typed HTTP actions.
- `GET /search` SSR with a small external loader, lazy WASM, bounded SearchStateV1 and fail-closed fallback.
- Shared Rust Todo components rendered natively and in browser DOM; no default remote event circuit.
- Native Tower middleware and `RequestContext` with request-local typed extensions and global/group/route scope tests.
- Local strict CSP, escaping, API validation, Chrome 155 and Firefox 157 E2E tests.
- Release-mode informational performance baseline with the exact environment and limitations.

## Development and release commands

```bash
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version 0.2.129 --locked

# Normal local development (debug mode):
python3 scripts/build_browser.py
cargo run -p yetlet-todo-server

# Release-mode baseline (not a public release):
cargo build --release --locked -p yetlet-todo-server
python3 scripts/build_browser.py --release
python3 scripts/smoke_firefox.py --profile release
python3 scripts/measure_baseline.py --profile release \
  --samples 30 --browser-runs 5 --out target/m5-baseline-release.json

# The existing Chrome script uses a debug server:
cargo build --locked -p yetlet-todo-server
python3 scripts/build_browser.py
python3 scripts/smoke_browser.py
```

Run the Cargo format/Clippy/native tests, wasm32 checks and `python3 scripts/check_docs.py` as the standard verification chain. Rebuild matching artifacts when switching build profiles. The Firefox smoke needs geckodriver.

## Readiness and unresolved gates

| Gate | State | Detail |
|---|---|---|
| M0–M4 prototype contracts | Verified locally | Native/Browser/Middleware tests |
| M5 A01–A10 acceptance matrix | Verified at prototype scope | [M5](M5-VERIFICATION.md) |
| Reproducible performance tooling | Verified locally | Debug and release baselines, no performance thresholds |
| Rust stable + WASM | Verified with rustc 1.99.0 | Native/wasm32 builds |
| Declared MSRV 1.85.0 | **Unverified** | Separate historical toolchain was not installed/tested |
| Browser compatibility | **Partial** | Chrome 155, Firefox 157; no WebKit/mobile/IME matrix |
| Accessibility/keyboard | **Partial** | Firefox physical keyboard input works; no complete screen reader/IME audit |
| Authentication/authorization | **Missing** | Anonymous in-memory Todo demo only |
| Credentialed CSRF defense | **Missing** | Origin/JSON constraints are not production session/CSRF policy |
| HTTPS reverse-proxy trust | **Missing** | Demo direct-HTTP Origin matching |
| Persistence and transactional semantics | **Missing** | Only in-memory store; no migrations or durable state |
| Native performance/observability | **Partial** | First-byte sample; no concurrency/throughput, tracing stack or cancellation system |
| Package license | **BLOCKED** | Maintainer must choose and commit a license |
| Stable public API/version policy | **BLOCKED** | Current component/renderer/action APIs are experimental |
| Independent security review | **Not performed** | Self-review and local tests are not independent certification |
| GitHub Actions for current stage | **Not run** | Remote CI artifact needs an authorized push/PR |

## Decisions before publication

1. **Maintainer:** Select project license and commit LICENSE before distribution.
2. **Maintainer/team:** Agree on public API scope, semantic versioning, supported Rust MSRV and browsers.
3. **Security/team:** Design authentication, session/CSRF defense, trusted TLS proxy policy and durable storage before any real user-data deployment.
4. **QA:** Complete real IME/accessibility coverage, additional browser environments and independent review.
5. **CI owners:** Run the updated workflow through an authorized PR/branch, inspect results and retain the baseline artifact.

## Scope protection

Yetlet keeps RequestContext in the native server adapter; browser WASM is independent of Axum/Tower. Auth, sessions, tracing, CORS, rate limiting, GPU and other providers should remain opt-in rather than enlarging the runtime core. The state sent to the browser is explicitly selected and does not serialize server request context, credentials or arbitrary extensions.

**Backlog status:** YT-019 can be completed using local evidence. YT-020 documentation is prepared, but **public-release readiness remains blocked** by the maintainer license decision and separate production-support gates. Do not mark the project production-ready.
