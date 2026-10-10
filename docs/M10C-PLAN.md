# M10c Plan — Minimal API Compatibility and Document Metadata

**Date:** 2026-10-10
**Task:** YT-062 / M10c
**Status:** Planning accepted by independent read-only reviewer `wc_job_Ak5u-tHYevPK4wdE`; bounded runtime implementation may proceed.
**Baseline:** `master` at `7c5d1267fd2ea3490738da105b476e271278ae7a`, clean and synchronized with `origin/master`.

## 1. Maintainer decisions and scope

The maintainer explicitly defers YT-060 licensing/security-policy selection and YT-061 GitHub governance changes until Yetlet is formally prepared to become Public. Both items are **Pending by maintainer decision**, not technical failures. This task must not add licenses, publish crates, modify GitHub settings, create releases or deploy anything.

YT-062 is the first technical M10 slice. It must preserve Yetlet's **small, HTML-first, handler-first** shape. The outcome is a minimal additive document metadata contract and a provisional compatibility policy, not a framework-wide head manager.

## 2. Tiny-by-design decision

The current `render_document(title, view)` always emits `<html lang="zh-Hant">`. M9c proved that English content needs an explicit language owner, but that evidence does not justify arbitrary head nodes, a templating DSL or a metadata plugin system.

The bounded design is:

1. Add a small validated `LanguageTag` value type for common HTML language tags such as `en`, `en-US` and `zh-Hant`.
2. Add `DocumentMetadata` containing exactly:
   - escaped document title;
   - validated root language.
3. Keep charset and viewport as framework-owned fixed correctness defaults.
4. Keep the strict CSP as the existing HTTP response policy; document metadata cannot weaken or replace it.
5. Do **not** add arbitrary `<meta>`, `<link>`, `<script>`, `<style>`, raw head HTML, SEO collections, OpenGraph, canonical URLs or per-page CSP.
6. Keep every existing application path source-compatible:
   - `Page::new(title, view)`;
   - `App::page(path, title, handler)`;
   - `render_document(title, view)`;
   - `render_page(title, view)`.
7. Add explicit APIs rather than changing existing signatures:
   - `LanguageTag::new` and `LanguageTag::as_str`;
   - `DocumentMetadata::new` using the legacy `zh-Hant` default;
   - `DocumentMetadata::with_language`;
   - `DocumentMetadata::{title,language}` accessors;
   - `Page::with_metadata`;
   - `render_document_with_metadata`;
   - `render_page_with_metadata`.
8. Custom-language pages use ordinary `.get` handlers returning `Page::with_metadata`; `App::page` remains the smallest default-language convenience.

The language validator is intentionally conservative and local: ASCII alphabetic primary subtag, optional hyphen-separated ASCII alphanumeric subtags, no empty segment, control characters, whitespace or markup delimiters, and a small total-length bound. It is an HTML-safety/application-contract validator, not a complete BCP 47 registry implementation.

## 3. Compatibility policy

YT-062 records a provisional policy without claiming current public stability:

- Yetlet-owned crates continue to share one workspace version/release train; examples are not published contracts.
- While the workspace remains `0.y`, no SemVer guarantee is active. However, changing a **Stable Candidate** requires explicit rationale, migration notes, updated compile fixtures, task evidence and independent review.
- Broad Axum/Tower/http reexports remain support-only and outside Yetlet-owned compatibility promises.
- Experimental browser, renderer, reactive, document and advanced Session APIs may still change; changes must update the API classification.
- The first public release must explicitly activate or revise this policy after YT-060/YT-061 decisions.
- Rust 1.88 remains the declared MSRV. A future MSRV increase requires a documented reason, CI evidence and release notes; no silent increase.

## 4. Compile-level compatibility evidence

Add one normal integration-test fixture under `yetlet-server-axum` that compiles and executes both paths:

- legacy `Page::new`, `App::page`, `render_document` and `render_page` usage;
- explicit `LanguageTag`, `DocumentMetadata`, `Page::with_metadata`, `render_document_with_metadata` and `render_page_with_metadata` usage.

The fixture must verify:

- old calls remain source-compatible;
- the default document remains `zh-Hant`;
- explicit English metadata emits `<html lang="en">` without a nested language workaround;
- title and body escaping remain unchanged;
- invalid language input fails closed before rendering;
- strict CSP remains byte-identical on default and metadata-aware response paths;
- no script/WASM/runtime is injected.

This fixture is deliberately not a new compatibility-testing framework, snapshot engine, procedural macro or API-diff dependency.

## 5. Documentation and classification

Implementation must update:

- `docs/API-CONTRACTS.md` with the provisional compatibility policy and the status of the new metadata types;
- `docs/ARCHITECTURE.md` with the root document ownership boundary;
- `README.md` with the smallest custom-language example;
- `docs/RELEASE-READINESS.md`, while keeping YT-020 Blocked;
- `docs/BACKLOG.md` and `docs/ROADMAP.md` with evidence only after validation;
- `docs/M10C-VERIFICATION.md` with exact tests, reviewer findings and limitations.

A new ADR is unnecessary unless implementation expands beyond the bounded title/language contract. If arbitrary head content, CSP customization or another subsystem becomes necessary, stop and open a separately reviewed architectural task instead.

## 6. Validation gates

Before implementation review:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --all-targets --locked
cargo test --workspace --doc --locked
cargo +1.88.0 test --workspace --all-targets --locked
cargo +1.88.0 test --workspace --doc --locked
python3 scripts/check_docs.py
git diff --check
```

Task-specific assertions must also compile the compatibility fixture and inspect rendered HTML/CSP. Browser E2E is not required unless browser/renderer source or generated assets change; exact production asset isolation remains a required check if any shared dependency graph changes.

## 7. Review sequence

1. Planning-only diff and task boundaries receive an independent role-directed read-only reviewer PASS.
2. Runtime/tests/docs are implemented in the same worktree.
3. Complete validation freezes the source.
4. A new independent read-only reviewer examines the implementation and compatibility evidence.
5. Only after PASS may YT-062 be marked Done. YT-063 must not begin before that gate.

No commit or push is authorized by this plan.
