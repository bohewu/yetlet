# M10c Verification — Compatibility and Document Metadata

**Date:** 2026-10-10
**Task:** YT-062 / M10c
**Status:** Done. Planning review PASS, implementation-review P2 repaired, complete post-repair validation and repeat reviewer `wc_job_rz9Ci3gWOLfLKdqN` PASS.

## Bounded implementation

The native server adapter now owns a minimal typed complete-document contract:

- `LanguageTag` and `LanguageTagError`;
- `DocumentMetadata` with escaped title plus validated root language;
- additive `Page::with_metadata`;
- additive `render_document_with_metadata` and `render_page_with_metadata`.

Legacy `Page::new`, `App::page`, `render_document` and `render_page` remain source-compatible and retain the existing `zh-Hant` default. Charset, viewport and strict CSP remain framework-owned. No arbitrary head content, raw HTML, scripts/styles, SEO collection, DI, MVC, plugin registry, ORM or metadata subsystem was introduced.

The Forms Validation sample now renders root `<html lang="en">` through the typed metadata API and removes its previous nested `<main lang="en">` workaround.

## Compatibility evidence

`crates/yetlet-server-axum/tests/public_api_compat.rs` compiles and executes old and new entry points. Unit/integration tests cover:

- default and explicit language rendering;
- title/body escaping;
- conservative invalid-language rejection;
- unchanged strict CSP;
- no browser runtime injection;
- Forms GET/error/success responses using one root language owner.

The provisional version/change rules are recorded in [Compatibility Policy](COMPATIBILITY.md). All crates remain private `0.1.0` prototype packages; this task does not activate a public SemVer guarantee.

## Required validation

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --all-targets --locked
cargo test --workspace --doc --locked
cargo +1.88.0 test --workspace --all-targets --locked
cargo +1.88.0 test --workspace --doc --locked
python3 scripts/check_docs.py
python3 scripts/check_samples.py
python3 scripts/test_check_samples.py
git diff --check
```

Because no browser/renderer code or dependency graph changes, browser E2E is not a task-specific requirement. Production asset and normal-dependency isolation should still be compared before source freeze.

## Observed validation

Initial source-freeze Job `wc_job_EJNR6JoCuEDoy6u-` completed at terminal exit 0; after the reviewer compatibility repair, replacement Job `wc_job_4kyDLYWfyvUtPgQV` reran the complete matrix and also exited 0:

| Gate | Result |
|---|---:|
| Current-stable workspace tests | **196 / 196 PASS** |
| Current-stable doctests | **5 / 5 PASS** |
| Rust 1.88 workspace tests | **196 / 196 PASS** |
| Rust 1.88 doctests | **5 / 5 PASS** |
| Format and strict Clippy | PASS |
| Rustdoc with `-D warnings` | PASS |
| Current-stable wasm32 packages | **8 PASS** |
| Rust 1.88 wasm32 packages | **8 PASS** |
| Forms Validation real localhost smoke | PASS |
| Documentation / Backlog | **57 documents / 67 tasks PASS** |
| Sample catalog / focused checker | **12 / 4 PASS** |
| Normal dependency trees | **28 PASS; no new dependency** |
| Production `dist/` | **136 / 136 byte-identical** |
| Production digest | `faebb0709dccd863` |
| Git/worktree/index | original `master` HEAD, one worktree, empty index |

The first complete pass exposed a pre-existing rustdoc warning where `App<S> -> App<S>` was interpreted as HTML. The comment now uses code formatting, and the entire current-stable/MSRV/wasm/docs/smoke/asset/dependency matrix was rerun after that source change. No runtime behavior or dependency changed.

Planning reviewer `wc_job_Ak5u-tHYevPK4wdE` passed before implementation. No YT-063 source was started.

## First implementation-review finding and repair

Independent read-only reviewer Job `wc_job_VWOJoR-lV3PAyS_-` found one material P2 source-compatibility regression. The initial implementation changed `Page::new` from the established `impl Into<Cow<'static, str>>` title bound to `impl Into<String>`. Literal callers still compiled, but a downstream generic wrapper bounded only by `Into<Cow<'static, str>>` no longer type-checked.

The original signature is restored exactly. `Page::new` converts the resulting `Cow` to an owned string only at the internal `DocumentMetadata` boundary. The compile fixture now includes a generic legacy wrapper and exercises both borrowed and owned title inputs, so future narrowing of the original bound fails at compile time.

This repair does not change rendered HTML, CSP, metadata validation, dependencies or browser assets. Replacement validation Job `wc_job_4kyDLYWfyvUtPgQV` reran the focused generic fixture and the complete current-stable/MSRV/wasm/smoke/docs/asset/dependency matrix, all PASS.

## Independent implementation reviewers

First read-only reviewer Job `wc_job_VWOJoR-lV3PAyS_-` returned one P2: the first implementation accidentally narrowed the public `Page::new` generic bound. After the exact signature and compile fixture repair, a new role-directed reviewer Job `wc_job_rz9Ci3gWOLfLKdqN` completed with terminal exit 0 using `gpt-6-astra`, high reasoning effort and read-only sandbox. It was ordinary `codex exec <reviewer prompt>`, not `codex exec review`, native ACP dispatch or formal assurance. It made no workspace mutations and returned:

```json
{
  "overall_status": "pass",
  "issues": [],
  "required_followups": [],
  "optional_notes": []
}
```

YT-062 / M10c is locally Done at prototype scope. YT-060 and YT-061 remain Pending by maintainer decision, YT-063 remains Todo, YT-020 remains Blocked, and no commit or push was performed.
