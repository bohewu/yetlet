# M9j Verification — API Compatibility and Final M9 Evidence Freeze

**Date:** 2026-10-10
**Task:** YT-059 / M9j
**Status:** Done. Complete regression PASS; API inventory P2 findings repaired; repeat independent final reviewer `wc_job_QkQkovlBlvrhPxCE` PASS.

## Scope

M9j freezes the complete 12-sample expansion and classifies the actual public Rust contracts in [Public API Contract Classification](API-CONTRACTS.md). Stable Candidate is an evidence category only; it does not mark Yetlet or any crate stable, released or production-qualified.

The final gate must repeat native workspace tests/doctests, strict Clippy/format, all relevant localhost samples, wasm32 packages, Chrome/Firefox production regressions for Todo/keyed/multi-island behavior, retained render strategy checks, every workspace normal dependency tree, exact production asset hashes and documentation/catalog/Git hygiene.

## Required validation

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --all-targets --locked
cargo test --doc --workspace --locked
cargo check --locked --target wasm32-unknown-unknown \
  -p yetlet-todo-browser -p yetlet-search-island \
  -p yetlet-reactive -p yetlet-dom-keyed \
  -p yetlet-render-bench -p yetlet-keyed-bindings \
  -p yetlet-dashboard-counter-island -p yetlet-dashboard-filter-island

python3 scripts/smoke_hello_world.py
python3 scripts/smoke_rest_api.py
python3 scripts/smoke_cart.py
python3 scripts/smoke_middleware_lab.py
python3 scripts/smoke_forms_validation.py
python3 scripts/smoke_file_server.py
python3 scripts/smoke_sqlite_crud.py
python3 scripts/smoke_sse_notifications.py
python3 scripts/smoke_auth_notes.py
python3 scripts/test_smoke_modular_shop.py
python3 scripts/smoke_modular_shop.py

python3 scripts/build_browser.py --release
python3 scripts/smoke_browser.py --profile release
python3 scripts/smoke_firefox.py --profile release
python3 scripts/build_keyed_bindings.py --release
python3 scripts/smoke_keyed_bindings.py --profile release
python3 scripts/smoke_keyed_bindings_firefox.py --profile release
python3 scripts/build_multi_island_dashboard.py --release
python3 scripts/smoke_multi_island_dashboard.py --profile release
python3 scripts/smoke_multi_island_dashboard_firefox.py --profile release
python3 scripts/build_render_bench.py --release
python3 scripts/measure_render_strategies.py --profile release

python3 scripts/check_samples.py
python3 scripts/test_check_samples.py
python3 scripts/check_docs.py
git diff --check
```

Retain exact commands, test counts, browser versions, bundle/binary sizes, production digest and per-file asset comparison. Validate that provider/database/browser dependencies remain confined to declared consumers. No catalog or backlog completion claim may precede a final independent read-only reviewer PASS.

## Complete observed regression freeze

The final matrix completed successfully. Retained command stdout/stderr, dependency trees, API declarations, exact asset/source hashes, bundle/binary sizes, browser/tool versions, local baseline and benchmark JSON are under Git-ignored `target/m9j-evidence/`.

| Gate | Result |
|---|---|
| Workspace Rust tests | **190/190 PASS** |
| Workspace doctests | **5/5 PASS** |
| Workspace strict Clippy / format / rustdoc | PASS |
| Native release builds | 12 runnable server applications PASS |
| Native sample smokes | **11 PASS**: Hello, REST, Cart, Middleware, Forms, Files, SQLite, SSE, Auth, Modular cleanup regression and Modular integration |
| wasm32 packages | **8 PASS** |
| Chrome 155 release suites | **3 PASS**: Todo/Search, Keyed lifecycle, Multi-Island |
| Firefox 157 release suites | **3 PASS**: Todo/Search, Keyed lifecycle, Multi-Island |
| Catalog / checker | **12 entries / 4 of 4 tests PASS** |
| Documentation / backlog | **52 required documents / 59 task IDs PASS** |
| Workspace normal dependency trees | **28 PASS** |
| Public API inventory | **170 public declarations/constants across 14 source files** classified across all owned crates |
| Production Browser/Search digest | `faebb0709dccd863` |
| Production `dist/` isolation | **136/136 files byte-identical** before/after every build/fixture |
| Git/process hygiene | One original worktree, pushed M9e HEAD, empty index, whitespace PASS, no owned Yetlet process |

Observed tools at the M9j freeze: Rust/Cargo 1.99.0, Chrome 155.0.8059.39, Firefox 157.0 and geckodriver 0.37.1. That freeze did not verify the then-declared Rust 1.85 baseline or other browser/OS versions. A later CI/security round raises the workspace baseline to Rust 1.88 for patched `time 0.3.47` and separately validates the native workspace with that toolchain; this does not alter the retained M9j browser evidence.

### Native release executable sizes

Raw local unstripped bytes; these are not memory, cold-start or transfer measurements.

| Application | Raw bytes |
|---|---:|
| Hello World | 1,886,560 |
| Middleware Lab | 2,130,528 |
| Forms Validation | 2,047,936 |
| REST API | 2,472,408 |
| Session Cart | 2,363,888 |
| File Server | 2,863,160 |
| SQLite CRUD | 4,691,512 |
| SSE Notifications | 2,307,000 |
| Multi-Island server | 2,641,576 |
| Auth Notes | 2,562,480 |
| Modular Shop | 5,077,728 |
| Todo server | 3,002,408 |

### Production Browser/Search bundle freeze

The M7j/M8/M9 production digest remains unchanged, so M9 introduces **zero byte delta** to the existing Todo/Search production bundle.

| Asset | Raw | Deterministic local gzip |
|---|---:|---:|
| Todo loader `app.js` | 151 | 138 |
| Todo JS module | 22,299 | 4,792 |
| Todo WASM | 294,555 | 96,419 |
| **Todo activated total** | **317,005** | **101,349** |
| Search loader | 1,144 | 551 |
| Search JS module | 14,699 | 3,675 |
| Search WASM | 179,716 | 70,806 |
| **Search activated total, excluding loader** | **194,415** | **74,481** |

The separately generated Multi-Island release remains isolated under `target/`: Counter 166,999 raw / 63,710 local-gzip bytes, Filter 177,694 / 68,753, shared CSS 416 / 279, digest `1eccb326477b59ae`. Neither island changes production `dist/`.

### Local informational baseline

Loopback/local isolated Chrome observations, not SLA or cross-framework comparisons:

| Measurement | Median | p95 |
|---|---:|---:|
| SSR request to first response byte | 0.034 ms | 0.037 ms |
| Chrome navigation to Todo ready | 48.7 ms | 89.8 ms |
| Search trigger to island ready | 3.6 ms | 4.7 ms |

### Retained renderer benchmark

The isolated M7a release fixture retained digest `71182d03b13e4d13`, 200 keyed rows, 500 updates and 10 rounds. All strategies produced equivalent final state.

| Strategy | Median round total |
|---|---:|
| Subtree replacement | 215.75 ms |
| Positional View patch | 41.75 ms |
| Keyed direct DOM | 1.90 ms |
| Static template slots | 1.80 ms |

These values continue to support bounded keyed adoption only; they do not select a general production renderer, VDOM or template compiler.

### Dependency and target isolation

- `rusqlite`/`libsqlite3-sys` occur only in `yetlet-sqlite-crud` and `yetlet-modular-shop`.
- `yetlet-session` occurs only in its own crate plus Cart, Auth Notes and Modular Shop consumers.
- SQLx, Diesel and SeaORM occur nowhere in normal workspace dependency trees.
- Browser/WASM packages exclude SQLite and Session dependencies.
- File multipart/tempfile, SSE hub, synthetic identity and dashboard browser dependencies remain application-local.

Two evidence-harness assertions required correction after all product commands had already passed: the first forgot the `yetlet-session` crate must appear in its own tree; the second required an exact standalone crate label even though the draft wrote `yetlet-actions::{...}`. The policy was corrected to the intended allowlist and the document added explicit crate headings. No runtime source, product test or acceptance threshold changed; all retained commands, trees, assets and Git/source hashes were then verified.

## Public API inventory gate

The inventory must cover every crate's public types/functions and method families, explain evidence and known break risks, and avoid treating upstream reexports or test hooks as stable merely because they are public. In particular:

- handler-first `App`, `RequestContext`, renderer-neutral View/binding values and bounded provider application APIs may be Stable Candidates;
- the fixed document language/head contract, browser actions, DOM/keyed/reactive APIs and advanced Session hooks remain Experimental;
- adapter constructors, direct middleware functions and generated/browser implementation details remain Internal/support-only.

## First independent reviewer findings and repair

Independent final reviewer Job `wc_job_BzkzTPcKr4FKATTr` accepted the complete regression evidence but returned two documentation/inventory P2 findings:

1. the generated declaration inventory omitted public constants and the classification did not explicitly cover `SessionConfigError`, `InvalidStart`, `yetlet_request_id::HEADER`, `yetlet_server_axum::DEFAULT_CSP` or `LocalGraph::DEFAULT_MAX_PASSES`;
2. the first draft placed broad Axum `http`/`extract` reexports under Stable Candidates even though sample evidence covers selected upstream types rather than the complete upstream modules.

The declaration extractor now includes public constants, producing **170** entries across the same 14 source files. Request ID's header/typed start error and Session's typed configuration error are explicitly bounded Stable Candidates; `DEFAULT_CSP` and `DEFAULT_MAX_PASSES` are Experimental policy defaults with stated break risks. Broad Axum/http/extractor/response reexports are explicitly Internal/support-only. No runtime source or product evidence changed, so documentation/inventory checks and a new independent reviewer were the required replacement gate; repeat reviewer `wc_job_QkQkovlBlvrhPxCE` returned PASS and accepted the repaired 170-entry classification.

## Reviewer gate

Repeat independent role-directed reviewer Job `wc_job_QkQkovlBlvrhPxCE` completed at terminal exit 0 using the saved `gpt-6-astra` model, high reasoning effort and read-only sandbox. This was ordinary `codex exec <reviewer prompt>`, not `codex exec review`, native ACP/`agent_type` dispatch or formal assurance. It inspected the repaired constant/error inventory, broad reexport classification, retained complete matrix and release blockers, and returned:

```json
{
  "overall_status": "pass",
  "issues": [],
  "required_followups": [],
  "optional_notes": []
}
```

The reviewer made no edits, stages, commits or pushes. YT-059 and M9 are locally Done at prototype scope. Stable Candidate remains an evidence classification, not a SemVer guarantee. Public-release YT-020 remains **Blocked** for licensing, verified historical MSRV, support/compatibility policy, document metadata design, external CI and production-security decisions.
