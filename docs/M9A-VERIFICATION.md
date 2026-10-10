# M9a Verification — Hello World Sample

**Date:** 2026-10-09
**Task:** YT-050 / M9a
**Status:** Done. Native validation and independent read-only reviewer `wc_job_Htk-Quo_aFExMJs3` PASS.

## Outcome

A distinct `yetlet-hello-world` application is promoted to the verified user-facing catalog. It reuses the existing handler-first `App`, `Page` and native Axum server; it does not replace the internal `yetlet-server-axum/examples/minimal.rs` example and adds no framework abstraction.

The only application routes are `GET /` (native SSR greeting) and `GET /hello/{name}` (escaped untrusted path text). Native 404 and 405 remain unchanged. Strict CSP and no JavaScript/WASM are required.

## Isolation contract

- Direct application dependencies: `axum`, `tokio`, `yetlet-core`, `yetlet-server-axum`. `tower` is test-only.
- Excluded from `yetlet-hello-world` runtime dependency graph: `yetlet-session`, `yetlet-request-id`, `cookie`, `getrandom`, `wasm-bindgen`, `wasm-bindgen-futures`, `js-sys`, `web-sys`, `yetlet-dom`, `yetlet-dom-keyed`, `yetlet-reactive`, and any database/ORM dependency.
- No generated browser assets, server-side event circuit, provider registrations or custom route/response abstractions.
- The app defaults to loopback and is an anonymous prototype, not a public/exposed hardened service.

## Acceptance validation

```bash
cargo fmt --all -- --check
cargo clippy --locked -p yetlet-hello-world --all-targets -- -D warnings
cargo test --locked -p yetlet-hello-world --all-targets
cargo build --locked -p yetlet-hello-world
python3 scripts/smoke_hello_world.py
cargo tree --locked -p yetlet-hello-world
python3 scripts/check_samples.py
python3 scripts/test_check_samples.py
python3 scripts/check_docs.py
git diff --check
```

**Observed source freeze:** Native sample 3/3 Rust tests PASS; complete workspace **117/117** and doctests **5/5** PASS; workspace strict Clippy, format, native debug and release builds PASS. The actual isolated localhost process smoke checks both routes, escaped HTML and CSP, zero browser assets or Session headers, native 404/405 and process cleanup. Catalog now has **four runnable samples**; four catalog checker tests PASS; docs check reports **41 documents / 59 backlog items** PASS. The existing catalog checker negative tests previously assumed Todo was always item zero; this stage repairs the fixture to select the Todo by its stable ID, preserving all original negative assertions without changing the checker API.

The Cargo normal-dependency tree for `yetlet-hello-world` includes only the expected Axum/Tokio/core/server/HTML infrastructure and excludes all named Session, request ID, browser/WASM, DB/ORM and related provider dependencies. Captured local binary sizes: **26,737,520 raw bytes** debug and **1,857,904 raw bytes** release. These are unstripped local binary sizes, not memory/transfer measurements. No Todo/REST/Cart, Browser, renderer or core runtime source changed. The independent reviewer inspected code, catalog entry, exact README command, source evidence, negative tests, dependency isolation and Git hygiene, and returned PASS.

## Reviewer gate

Independent role-directed `codex exec` reviewer Job `wc_job_Htk-Quo_aFExMJs3` completed with terminal exit 0 using `gpt-6-astra`, high reasoning effort and a read-only sandbox; it was **not** `codex exec review` or a native coding-agent dispatch. It reviewed the M9a source, catalog negative-test repair, full M9a diff and retained native/workspace verification, without writing files. Final result:

```json
{
  "overall_status": "pass",
  "issues": [],
  "required_followups": [],
  "optional_notes": []
}
```

YT-050 / M9a is locally **Done** at prototype scope. YT-051 / M9b is next and must receive its own independent reviewer. No commit, push or worktree change occurred.
