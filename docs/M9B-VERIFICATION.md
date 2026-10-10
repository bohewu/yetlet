# M9b Verification — Middleware Lab

**Date:** 2026-10-09
**Task:** YT-051 / M9b
**Status:** Done. Full native validation and independent reviewer `wc_job_UnKflDaA8ZQHXCce` PASS.

## Purpose and scope

A runnable native-only `yetlet-middleware-lab` package exercises the existing handler-first `App`, request-owned `RequestContext`, native Axum middleware `Next` and Tower `ServiceBuilder`, plus a third-party `tower-http` response-header layer.

The routes `/outside`, `/lab/plain`, `/lab/guarded`, `/lab/stream` and `/lab/broken-stream` demonstrate:
- pre-routing global middleware around 404/405;
- matched group scope and an individual method-route guard, entry/exit and reverse unwind;
- short-circuit before a handler and without polling the owned HTTP body;
- separation of typed request-local extensions from shared application state;
- normal streaming chunks and an intentionally failing stream whose error must not be swallowed after the response headers are returned.

No new Yetlet framework abstractions, default providers, mandatory DI or browser runtime are part of this sample.

## Acceptance

```bash
cargo fmt --all -- --check
cargo clippy --locked -p yetlet-middleware-lab --all-targets -- -D warnings
cargo test --locked -p yetlet-middleware-lab --all-targets
cargo build --locked -p yetlet-middleware-lab
python3 scripts/smoke_middleware_lab.py
cargo tree --locked -p yetlet-middleware-lab
python3 scripts/check_samples.py
python3 scripts/test_check_samples.py
python3 scripts/check_docs.py
git diff --check
```

**Observed results:** All **7/7** Middleware Lab tests pass: exact global/group/method scope order and reverse unwind, short-circuit skipping the handler, no request body polling during early return, native 404/405 with Allow, request-local header/extension isolation from Arc-backed application state, bounded label handling, Tower HTTP response headers, complete chunk stream and unmodified mid-body failure. Real ephemeral localhost smoke passes global/route labels, 403, 404, 405 and streaming; the sample process is terminated by the smoke harness.

Complete workspace Clippy `-D warnings`, formatting, **124/124 native workspace tests**, **5/5 doctests**, native debug and release builds, Python compile, **5 verified samples**, **4/4 catalog checker tests** and **42 documents/59 backlog items** all pass. Release binary is **2,101,760 raw bytes** and debug binary **28,809,352 raw bytes** (local, unstripped, not wire size). Native Cargo normal-dependency tree includes Axum/Tokio/Tower/tower-http/futures-util/server adapter but excludes Session, Request ID, DOM/keyed/reactive, WASM, SQLite, SQLx and ORMs. The only new transitive dependency added to Cargo.lock is `futures-macro` for the explicitly selected application dependency. No framework runtime source was changed.

A complete validation Job `wc_job_aXTYXNi0ncprylRQ` reached terminal exit 0 with all full-workspace/normal-tree, docs/catalog and Git hygiene checks passed. The original `master` HEAD `742e0d4909cc9b132d81b78e12bbb66d5bf60f57` and one worktree were preserved, the index remains empty, and no samples after M9b were started. The independent reviewer has independently inspected the actual code, task validation, dependency boundaries and status claims. No material findings were returned; M9b can close.

## Independent reviewer gate

Role-directed local `codex exec` reviewer Job `wc_job_UnKflDaA8ZQHXCce` completed terminal exit 0 with model `gpt-6-astra`, high reasoning effort and read-only sandbox. This was **not** `codex exec review` or native ACP dispatch; it did not edit, stage, commit or push. It reviewed the full M9b diff, native middleware and body semantics, retained source/test and catalog evidence and returned:

```json
{
  "overall_status": "pass",
  "issues": [],
  "required_followups": [],
  "optional_notes": []
}
```

YT-051 is locally **Done** at prototype scope. YT-052 / M9c Forms Validation is the next eligible item, with its own reviewer gate.
