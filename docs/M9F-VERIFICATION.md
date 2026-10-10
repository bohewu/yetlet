# M9f Verification — Bounded SSE Notifications

**Date:** 2026-10-10
**Task:** YT-055 / M9f
**Status:** Done. First reviewer P2 repaired, complete replacement validation PASS, repeat reviewer `wc_job_fa8KJiTxGRuWIEDF` PASS and exact catalog entry promoted.

## Scope

`yetlet-sse-notifications` is a native-only, loopback-only application using ordinary Axum HTTP/SSE primitives. An explicit application-owned Hub provides monotonic finite IDs, a 16-event replay window, eight subscribers, four queued events per subscriber, one-second heartbeat comments and deterministic subscriber cleanup. It introduces no Yetlet event framework, WebSocket, global circuit, browser runtime, Session, persistence, database or ORM.

Routes are `/healthz`, `/metrics`, `/events` and `/notifications`. The mutation endpoint is a command-line/local test helper: it rejects every `Origin`, requires exactly one JSON Content-Type and has a 2 KiB body limit. The stream endpoint returns explicit `text/event-stream`, no-store/nosniff and proxy-buffering-disabled headers.

## Delivery semantics

New subscribers receive future events only. A client supplying `Last-Event-ID` receives retained events with greater IDs. Cursors older than the replay window or newer than the current stream fail closed. A full per-client queue removes that lagging subscriber without delaying publishers; the client drains already queued events, reconnects and uses the replay window. This is bounded at-least-once delivery, not durable messaging or exactly-once processing.

## Acceptance commands

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --all-targets --locked
cargo test --doc --workspace --locked
cargo build --locked -p yetlet-sse-notifications
cargo build --release --locked -p yetlet-sse-notifications
python3 scripts/smoke_sse_notifications.py
cargo tree --locked -p yetlet-sse-notifications --edges normal
python3 scripts/check_samples.py
python3 scripts/test_check_samples.py
python3 scripts/check_docs.py
git diff --check
```

Required evidence includes live and replay order, heartbeat, duplicate/malformed/future/gap cursor behavior, bounded subscriber capacity, lagging-client disconnection, response-body drop cleanup, finite-ID exhaustion, native 404/405, real localhost disconnect/reconnect and absence of SSE-specific dependencies from unrelated server/browser packages. Production Browser/Search assets must remain byte-identical or any difference must be justified.

## Observed validation freeze

- SSE Notifications focused tests: **10/10 PASS**. They cover ordered replay, fresh-subscriber behavior, replay gaps and future cursors, queue overflow/disconnect/replay, eight-client capacity, response-body drop cleanup, finite event-ID exhaustion, publish validation, duplicate/malformed cursor headers, explicit SSE headers, metrics and native 404/405.
- A real localhost smoke observes an actual `: heartbeat`, receives live IDs 1–3, closes the socket and confirms subscriber cleanup, publishes IDs 4–5 while disconnected, reconnects with `Last-Event-ID: 3`, receives ordered replay, and verifies malformed/future cursors and browser-Origin mutation rejection. The owned server exits through SIGINT.
- Complete workspace format and strict Clippy PASS; native workspace tests **163/163 PASS** and doctests **5/5 PASS**.
- Native debug/release builds PASS; repaired release binary is **2,292,888 raw bytes** on this host.
- Six existing wasm32 packages PASS. Production Browser/Search release rebuild retains digest `faebb0709dccd863`; all **136/136** dist files are byte-identical to the pre-build snapshot.
- Existing catalog **8 entries** and checker tests **4/4 PASS**. The exact proposed SSE stanza passes a virtual **9-entry** catalog, but remains outside the real catalog until review.
- All **23 workspace package** normal dependency trees were checked. No unrelated package depends on `yetlet-sse-notifications`; the SSE package excludes browser/WASM, Session, Request ID, DOM/keyed/reactive, SQLite/SQLx and ORM dependencies.
- Documentation/backlog gate reports **47 required documents / 59 task IDs PASS**; tracked/untracked whitespace, original `master` HEAD `292b1524ff2ed0e223aeb2575e5b744cda14dfc5`, one worktree and empty index PASS.
- Retained logs, source/dist hashes, dependency trees, exact candidate catalog stanza and summary are under Git-ignored `target/m9f-evidence/`.

The first Python validation harness invocation stopped before any gate because a non-login `run_script` environment did not include Cargo in `PATH`. The corrected invocation explicitly selected `$HOME/.cargo/bin`, reran the complete matrix above and passed. This was a harness startup issue, not a source/test failure.

Browser E2E and renderer benchmarks were not repeated in this native-only slice because no browser/renderer source changed; wasm32 compilation, production rebuild and exact asset hashes provide the bounded isolation evidence. Full Chrome/Firefox regression remains M9j.

## First independent reviewer finding

Independent read-only reviewer Job `wc_job_LF-i2IsQxFXO-obd` returned one P2. The first parser trimmed the message before checking control characters, so leading newline and trailing tab characters could be removed and the resulting text published. That contradicted the documented fail-closed 1–200 non-control-character contract.

The repair checks the unmodified payload for every control character before trimming ordinary surrounding spaces. Focused endpoint regressions cover internal newline, leading newline and trailing tab, require HTTP 422 `invalid_message`, and assert the prior valid event remains the only retained event with latest ID 1. The complete replacement matrix then passed 163 workspace tests, five doctests, live SSE smoke, all 23 dependency trees, six wasm32 packages, Browser/Search release rebuild and 136 byte-identical assets. Repeat reviewer `wc_job_fa8KJiTxGRuWIEDF` returned PASS with no findings.

## Reviewer gate

First independent reviewer Job `wc_job_LF-i2IsQxFXO-obd` returned the control-character P2 above. After the source/test repair and complete replacement matrix, independent read-only reviewer Job `wc_job_fa8KJiTxGRuWIEDF` completed at terminal exit 0 with explicit `gpt-6-astra`, high reasoning effort and read-only sandbox telemetry. It returned:

```json
{"overall_status":"pass","issues":[],"required_followups":[],"optional_notes":[]}
```

Both were ordinary local `codex exec <reviewer prompt>` role-directed Jobs, not `codex exec review`, native ACP/`agent_type` dispatch or formal assurance. The repeat reviewer made no edits, stages, commits or pushes. Only after PASS was the exact reviewed candidate stanza promoted into `examples/samples.toml`; runtime source was unchanged. YT-055 is locally Done and YT-056 remains Todo.
