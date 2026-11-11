# YT-253 — Application Startup Implementation Verification (Port 7878)

**Date:** 2026-11-11 (Asia/Taipei)  
**Status:** Implementation tests PASS; independent YT-253 Reviewer PASS at Linux/Windows cross-target scope; native Windows execution still pending.  
**Project:** Yetlet / M26, YT-253  
**Branch:** `codex/m26-application-startup-dx`  
**Base commit:** `68336d8eede7e3a9f1e240327208eabc7432aa11` (YT-252 contract).  
**Reviewed candidate:** current uncommitted `crates/yetlet/src/server_axum/app.rs`, `crates/yetlet/Cargo.toml`, and `crates/yetlet/tests/server_axum_app_serve.rs` with related YT-251–253 documentation edits.

## Scope and change

The owner changed the fixed loopback HTTP development port from `127.0.0.1:3000` to **`127.0.0.1:7878`**, because an unrelated local service already occupies port `3000`. No attempt was made to stop, reconfigure, or probe that service beyond checking the listener.

Source changes are limited to additive `App<()>::serve()` and `serve_on(address)`, Tokio optional feature ownership (native adapter versus Redis Session), focused Rustdoc and existing source/feature compatibility tests. `serve()` still binds loopback HTTP only; `serve_on` retains explicit TCP binding and `into_router()` remains the native escape hatch. No TLS policy or release identity changes.

## Preserved original Reviewer finding

- Initial YT-253 independent Reviewer Job: `wc_job_CdqciKQzlAcfXH-o`, terminal `completed`, exit code `0`, JSON `overall_status = fail` with **one P2 evidence-only** finding.
- Original JSON: `.pipeline-output/yt253-application-startup-implementation-review-20261111/reviewer-output.json` (retained, not overwritten).
- The Reviewer did not identify a source correctness defect. It requested the original Cargo output, exit codes and feature trees to be retained and indexed.
- The earlier default-port HTTP branch on `127.0.0.1:3000` was skipped because a different listener already owned that address. This is preserved as historical evidence; it is not evidence for the new default.

## New, reproducible verification evidence

Evidence directory: **`artifacts/verification/yt253-port7878-20261111-a/`**.

The directory contains:

- `metadata.txt`: branch, baseline commit, exact source/test SHA-256 hashes, execution timestamps, overall status.
- `results.tsv`: one exit code and log path for each of 15 verification commands.
- `*.argv.txt`: the literal argv tokens, one per line, for each command (no shell interpolation or inferred flags).
- `*.log`: complete captured command stdout/stderr plus exit code.
- `log-hashes.sha256`: immutable-content digest checks for all 15 logs.

All 15 commands below exited `0`:

| Command group | Actual command | Result |
|---|---|---|
| Focused startup | `cargo test --offline --locked -p yetlet --features server-axum --test server_axum_app_serve -- --nocapture` | PASS, **4/4** |
| Full adapter scope | `cargo test --offline --locked -p yetlet --features server-axum` | PASS (unit/integration/doctests) |
| Clippy | `cargo clippy --offline --locked -p yetlet --features server-axum --all-targets -- -D warnings` | PASS |
| Native adapter | `cargo check --offline --locked -p yetlet --no-default-features --features server-axum` | PASS |
| Redis Session | `cargo check --offline --locked -p yetlet --no-default-features --features session-redis` | PASS |
| Core-only | `cargo check --offline --locked -p yetlet --no-default-features --features html` | PASS |
| Windows MSVC target | `cargo check --offline --locked -p yetlet --no-default-features --features server-axum --target x86_64-pc-windows-msvc` | PASS (Linux-host **cross-compile**, not native Windows execution) |
| Browser target | `cargo check --offline --locked -p yetlet --no-default-features --features html --target wasm32-unknown-unknown` | PASS |
| Core Tokio graph | `cargo tree --offline --locked --edges normal -e features -p yetlet --no-default-features --features html -i tokio` | PASS, no Tokio in Core graph |
| Server Tokio graph | Same Cargo tree arguments using `--features server-axum` | PASS; `net`/`rt` present, `rt-multi-thread` absent |
| Redis Tokio graph | Same Cargo tree arguments using `--features session-redis` | PASS; `rt-multi-thread` present |
| Rust formatting | `cargo fmt --all -- --check` | PASS |
| Documentation | `cargo xtask check docs` | PASS, 75 docs / 255 items at verification time |
| Sample catalog | `cargo xtask check samples` | PASS, 13 catalog samples |
| Whitespace | `git diff --check` | PASS |

Verification started `2026-11-11T17:52:58+08:00` and ended `2026-11-11T17:53:02+08:00`. The source SHA-256 for the startup method file is `e12e2891617d0519cca5bf18c8131b22bd1acc260610425ac977e0b397f8aeea`; for the startup test file it is `675e193735c088d024d954894a78fa01ddb6096c90c559083004b7e50df98097`.

**Default HTTP test was not skipped.** The targeted test log reports `serve_uses_loopback_default_and_supports_new_and_existing_routers ... ok`, in addition to explicit-port HTTP, binding-error and Router escape-hatch assertions. The test exercised live loopback HTTP on `127.0.0.1:7878`, then aborted its own spawned server tasks. No server process was left running.

## Independent Reviewer closeout

- Independent implementation follow-up Reviewer Job `wc_job_vX_D7m0O9q5on1hg`: terminal `completed`, exit code `0`, JSON `overall_status = pass`, `issues = []`, `required_followups = []` (five non-blocking optional notes).
- Saved `reviewer` role; model `gpt-6-astra`, effective effort `high`. Final child trace verifies role, model, effort and parent inheritance.
- Reviewer independently inspected all 15 command argv/log files, digests and both current source/test SHA-256 values; confirmed actual default `7878` live HTTP 4/4 without skip, feature-tree isolation and prior P2 evidence gap closure.
- Preserve initial Reviewer FAIL JSON at `.pipeline-output/yt253-application-startup-implementation-review-20261111/reviewer-output.json` and successful follow-up at `.pipeline-output/yt253-port7878-implementation-review-20261111/reviewer-output.json`, with role telemetry in `reviewer-trace.json`.
- Owner port-amendment contract Reviewer `wc_job_eJ9LTvWN2G-Yt7Oh`: PASS, terminal exit `0`, `gpt-6-astra`/high; separate from the YT-253 implementation review.

## Limits and release boundary

- Native Windows Runner execution remains pending; Windows MSVC *cross-target compilation on Linux* is not proof of native Windows runtime behavior.
- Tests are local-development and compatibility checks; they do not establish HTTPS/TLS, deployment, production readiness, high availability or performance guarantees.
- This is a source-only change after SDK `0.9.0-alpha.7` / CLI `0.3.1`; the published Reader Edition, CLI templates, tags and release artifacts remain unchanged.
- The previous evidence-only Reviewer FAIL remains preserved; both owner-amended YT-251 and YT-253 implementation follow-up reviews passed independently. YT-253 is Done at the documented Linux/Windows cross-target scope; native Windows runtime testing remains an explicit unverified limitation. Sample migration still requires YT-254 implementation and its own Reviewer PASS.
