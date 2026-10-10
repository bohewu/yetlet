# M9i Verification — Modular Shop Reference Application

**Date:** 2026-10-10
**Task:** YT-058 / M9i
**Status:** Done. Complete validation, first-reviewer P2 repair and repeat independent reviewer `wc_job_zbWS7OSbUtaKzYcn` PASS; exact catalog stanza promoted.

## Scope

`yetlet-modular-shop` composes distinct `identity`, `catalog`, `cart` and `orders` Rust modules through explicit application setup. It deliberately introduces no plugin registry, service locator, DI container, Controller hierarchy, MVC folder contract, ORM, generic repository or default database/session/auth capability.

- Identity uses terminal-only synthetic Alice/Bob codes, constant-time comparison, Session fixation rotation and CSRF.
- Catalog/inventory/orders use application-local direct parameterized `rusqlite` with a sample-owned migration.
- Cart uses the existing opt-in `yetlet-session` provider.
- Orders use `BEGIN IMMEDIATE`, atomic inventory/line/order writes and per-request owner authorization.
- Checkout uses a Session-CSRF plus cart-revision idempotency key, preventing duplicate inventory decrement when a commit succeeds before Session cleanup.
- `ShopModules` is a typed construction option. Disabling Orders omits its routes, link and checkout form; invalid dependency combinations fail during setup.

This application does not implement a payment processor or claim production commerce, identity, tax, shipping, backup, encryption, multi-process inventory or security qualification.

## Focused acceptance

```bash
cargo fmt --all -- --check
cargo clippy --locked -p yetlet-modular-shop --all-targets -- -D warnings
cargo test --locked -p yetlet-modular-shop --all-targets
cargo build --release --locked -p yetlet-modular-shop
python3 scripts/test_smoke_modular_shop.py
python3 scripts/smoke_modular_shop.py
cargo tree --locked -p yetlet-modular-shop --edges normal
python3 scripts/check_samples.py
python3 scripts/test_check_samples.py
python3 scripts/check_docs.py
git diff --check
```

Focused tests must cover explicit module dependencies and disabled-module ownership; synthetic-code redaction and Session rotation; CSRF and stale-cart rejection; transactional order success/persistence; line-insert rollback preserving inventory and Session cart; idempotent checkout; exactly one winner for last-stock concurrency; cross-user 403/list isolation; schema/listener fail-closed behavior and script-free strict-CSP SSR.

The real localhost smoke must use three owned server processes against one temporary DB: place an order, restart and verify persistence/cross-user authorization, then restart with Orders disabled and verify route/navigation/checkout removal. Every process must be reaped.

## Complete validation freeze

The complete matrix passed before review. Retained command logs, package trees, exact candidate catalog stanza, source/dist hashes and summary are under Git-ignored `target/m9i-evidence/`.

| Check | Observed result |
|---|---|
| Modular Shop focused tests | 9/9 PASS |
| Workspace native tests | 190/190 PASS |
| Workspace doctests | 5/5 PASS |
| Workspace format and strict Clippy | PASS |
| Modular Shop debug/release build | PASS; release binary 5,062,464 raw bytes |
| Three-process localhost integration smoke | PASS; checkout, restart persistence, Bob 403, disabled Orders and process cleanup |
| Eight existing wasm32 packages | PASS |
| Production Browser/Search release rebuild | digest `faebb0709dccd863` |
| Production `dist/` | 136/136 files byte-identical |
| Pre-review catalog/checker | 11 entries and 4/4 tests PASS |
| Modular Shop catalog | Exact virtual 12-entry catalog PASS before review; same stanza promoted after PASS and real 12-entry catalog rechecked |
| Dependency isolation | 28 workspace package normal trees PASS; SQLite only in `sqlite-crud` and `modular-shop`; no SQLx/Diesel/SeaORM |
| Documentation/backlog | 50 required documents / 59 task IDs PASS |
| Git/worktree/whitespace | Original pushed M9e HEAD, one worktree, empty index and untracked whitespace PASS |

The first evidence script reached every product gate and all dependency-tree commands, then failed only because its assertion incorrectly allowed `rusqlite` exclusively in Modular Shop while the separately reviewed `sqlite-crud` sample is also an intended application-local consumer. A recovery assertion corrected the policy to allow exactly those two applications, completed asset/Git/source freeze and reran docs/catalog/diff checks. No runtime assertion, source code or test expectation was changed.

The sample remained outside `examples/samples.toml` during review. The exact stanza retained in `target/m9i-evidence/catalog-entry.toml` passed the 12-entry virtual catalog and was promoted only after repeat reviewer PASS; the real catalog was then rechecked. Browser E2E and renderer benchmarks were not repeated because Modular Shop is native SSR and does not alter browser/renderer code; wasm compilation, production release rebuild and exact asset hashes provide bounded isolation. The complete cross-product browser matrix remains M9j.

## First independent reviewer finding and repair

Independent read-only reviewer Job `wc_job_a_WnhFSftzNoYYgf` returned one material P2 in the Python evidence harness. `start()` launched a child and opened its log before calling `wait_ready()`, but readiness failure occurred before the tuple reached the caller's `try/finally`; the child and stream could therefore leak. The reviewer reproduced this path with mocked process creation.

`start()` now catches every readiness exception, calls a dedicated `cleanup_failed_start`, terminates and reaps the exact child, falls back to kill/reap after a bounded wait, closes the log stream in `finally`, and re-raises the original readiness error. `scripts/test_smoke_modular_shop.py` has two standard-library regressions covering normal terminate/reap and timeout-to-kill fallback, including stream closure and original-exception identity. Both tests, Python compilation, the real three-process localhost smoke, docs/catalog checks, `git diff --check` and an explicit no-owned-process check pass. This repair changes no Rust runtime, database/session behavior or catalog stanza, so the reviewer-requested scoped validation is sufficient; the complete 190-test matrix remains retained from the frozen source before this smoke-only correction.

The original P2 is closed by repeat independent reviewer `wc_job_zbWS7OSbUtaKzYcn` PASS after the focused cleanup regression and real smoke.

## Reviewer gate

Repeat independent role-directed reviewer Job `wc_job_zbWS7OSbUtaKzYcn` completed at terminal exit 0 using the saved `gpt-6-astra` model, high reasoning effort and read-only sandbox. This was ordinary `codex exec <reviewer prompt>`, not `codex exec review`, native ACP/`agent_type` dispatch or formal assurance. It inspected the first finding, cleanup implementation, both mocked failure paths, scoped evidence and retained complete product freeze, and returned:

```json
{
  "overall_status": "pass",
  "issues": [],
  "required_followups": [],
  "optional_notes": []
}
```

The reviewer made no edits, stages, commits or pushes. Only after PASS was the exact reviewed stanza promoted to `examples/samples.toml`; the real **12-entry** catalog and 4/4 catalog checker tests were rerun. Runtime Rust source did not change after the complete freeze. YT-058 is locally Done, and YT-059/M9j is the next eligible final audit/freeze stage.
