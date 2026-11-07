# YT-234 — View Authoring API: typed fluent builder plan

**Status:** Done at local source and verification scope; independent design and implementation Reviewer PASS (2026-11-07).**Source baseline:** `master` at `889c7f7977e897a3bdf2ee2052bbfe01c2ad5ffd`.
**Scope:** Ergonomic Rust UI authoring only; no renderer, browser transport, runtime, or output-policy rewrite.

## Motivation and requested behavior

The user wants shorter, composable Rust authoring syntax while **retaining** the existing renderer-neutral `View` / `Element` tree and existing `Element::new(...).attribute(...).child(...)` API. They explicitly accepted a thin typed builder layer, chose `class` replacement/addition semantics, and deferred inline `style`. They do not request JSX, a proc macro, nested mutable builders, a new runtime, or a new WASM rendering model.

Current `Element` is already a consuming builder. It stores an owned tree; `Element::attribute` appends without deduplication, and existing native HTML and browser DOM renderers enforce different admission/security policies. The new API must lower to the same `Element` / `View` values **before** those existing renderers run.

## Selected authoring surface
Imports should make convenience constructors explicit and avoid conflicts with `yetlet::html` rendering, e.g. `use yetlet::view::tags::{div, h1, input, textarea};`.

```rust
let page: View = div()
    .id("app")
    .class("container")
    .add_class("active")
    .child(h1().text("Hello <Yetlet>"))
    .child(input().id("query").placeholder("Search..."))
    .child(textarea().placeholder("Notes..."))
    .into();
```

Proposed `TagElement<Tag>` (name is provisional) owns one `Element`, with zero-sized tag marker(s) and concrete tag constructors. Keep the public surface small. First slice: `div`, `span`, `p`, `h1`, `h2`, `section`, `form`, `label`, `button`, `input`, `textarea`, `ul`, `li`. If implementation mechanics justify a smaller starter set, preserve the quoted example and document deferred constructors. No additional crates/features; shared API remains compilable on native and `wasm32`.

- `.id(value)`: global attribute; **set/replace** id on this typed builder, not append another id.
- `.class(value)`: **replace** the effective class token sequence on this typed builder. Subsequent calls replace prior class values. No required boolean flag or alternate overload.
- `.add_class(value)`: append one or more HTML ASCII-whitespace-separated class tokens, stable-first-occurrence order, deduplicate tokens, emit at most one class attribute. Works with preceding `.class` / `.add_class` calls. Empty input does not introduce a duplicate or malformed class.
- `.placeholder(value)`: available **only** on `input()` and `textarea()` typed builders; `div().placeholder(...)` should fail to compile. Both elements are permitted by HTML; do not claim input-only.
- `.child(impl Into<View>)`, `.children(IntoIterator<Item = View>)`, and `.text(impl Into<String>)`: available only on non-void typed elements, with `.text` appending an ordinary `View::text`, **not** raw HTML; `input().child(...)` / `input().text(...)` should fail at compile time. Children order is preserved.
- `.attribute(name, value)` remains the general escape valve on a typed builder with the **same append semantics as** `Element::attribute`; it does not bypass either renderer's validation. Explicit convenience `.id`/`.class`/`.add_class` handle their targeted names without emitting duplicate attributes, including when a previous generic attribute on the same typed builder used that name. An unconditional duplicate produced solely by repeated generic `.attribute` calls still receives the existing renderer error. Document the mixed-call order and precedence explicitly.
- `From<TagElement<Tag>> for Element` and `From<TagElement<Tag>> for View` must preserve ownership and make existing `Element::child` / `View` call sites interoperable. No required `.build()` or fallback raw HTML conversion.
- `Element::new(...)` and `Element::attribute` keep their old semantics; **no renaming or breaking change** to the old API.
- Typed convenience methods are compile-time affordances only. They must not silently weaken the current native HTML renderer or the optional browser DOM validator. Existing broad legacy allowlists are a separate policy topic; this task must not change them.
- Do not support `.style()` (inline style is rejected by the present output policy). No JSX, proc macro, `div(children)` function overloading, stateful `.div().end()` navigation, VDOM, WASM renderer modifications, new features, or dependencies.

## Architecture and implementation constraints

1. Keep `View::Element(Element)` and all existing fields and serialization behavior unchanged. Place typed builder conveniences in an explicit `yetlet::view::tags` module (or equally narrow exported module), not `yetlet::html`. Avoid moving renderers or introducing new render intermediate representation.
2. Prefer one generic typed wrapper and small marker definitions, rather than a duplicated hand-written stateful builder per HTML tag. If `Element` needs a private/internal attribute upsert helper, keep raw `Element::attribute` append-only; justify any newly public helper separately.
3. Do not add new implicit URL trust, event bindings, CSS sanitization, hydration, resource loading or text escaping rules. Native HTML and DOM continue to validate independently.
4. Do not promise compile-time complete HTML validity: only selected convenience methods and void-child restriction are typed. Existing generic `.attribute` remains intentionally permissive; renderer policy remains authoritative.
5. Retain Rust 1.99 MSRV, no new dependency, no new public Cargo feature, no build tool changes, and no requirement for browser-only code to author Views.

## Acceptance and verification

- Native tests assert **structural equality** to an old-API-constructed `View`, including owned text escaping through existing `render_to_string`; confirm attribute/child order and direct conversion to `View`/`Element`.
- Tests cover `class` replacement, `add_class` deduplication (including multiple tokens/ASCII whitespace), call ordering and mixed helper/raw `.attribute` interactions, no emitted duplicate class/id, and renderer rejecting raw duplicate attributes where appropriate.
- Tests cover `input` and `textarea` placeholder, and that the resulting trees satisfy current applicable validation. If compile-fail testing is practical using built-in doctests, assert `div().placeholder(...)` and `input().child(...)` cannot compile; no new validation harness framework.
- Confirm the same `View` tree renders through existing native HTML and optional DOM's existing `validate` path where supported; no browser lifecycle or repaint changes asserted. A wasm32 **compile check** may be used if the target is installed, but a browser test is not demanded for an unchanged renderer.
- Update the View authoring guide, rustdoc examples and compatibility notes with the actual API and the fact that old callers work unchanged. Test docs with `cargo xtask check docs` and check whitespace.
- Run focused Rust package tests, docs, rustfmt, and affected consumer compile checks; record exact commands and which were unavailable. Do not broaden to release qualification or CI certification solely for an additive authoring API.

## Delivery gates

1. **First gate: independent ad hoc Reviewer reviews this plan** against the actual `View`, HTML, DOM source, ADR 0001/0003 and user intent. Resolve blocking material design issues before coding.
2. On Reviewer PASS, implement the bounded slice on `feature/view-authoring-api`; preserve unrelated work.
3. Run focused checks and request an **independent implementation review** on actual files/tests. Only claim PASS if that reviewer actually verifies them; no self-review presented as independent acceptance.
4. Keep commits, pushes, PR, merges, releases, hosted/browser execution and any cache deletion out of scope without specific owner authorization. Do not mark the item Done until implementation evidence and reviewer acceptance are recorded.

## Source-only acceptance record — 2026-11-07

The implementation lives only in the local `feature/view-authoring-api` branch
checkout, based on unchanged HEAD `889c7f7977e897a3bdf2ee2052bbfe01c2ad5ffd`.This is **not** part of the published SDK `0.9.0-alpha.6`.

- Independent design Reviewer Job `wc_job_szDmqHta7Rkkahtp`: terminal exit 0,
  `overall_status=pass`, zero issues and required followups.
- Independent implementation Reviewer Job `wc_job_a6P71YWNG8Z1vQs5`: terminal
  exit 0, `overall_status=pass`, zero issues and required followups. Both ran
  as read-only role-directed Codex Jobs using the verified reviewer role
  (`gpt-6-astra`); neither is a native managed agent dispatch claim.
- `cargo test --locked -p yetlet --lib`: PASS, 20 tests.
- `cargo test --locked -p yetlet --doc`: PASS, three doctests plus two
  compile-fail doctests.
- `cargo test --locked -p yetlet-browser --lib`: PASS, 19 tests including
  the new typed-tree DOM policy admission test.
- `cargo check --locked -p yetlet --target wasm32-unknown-unknown`: PASS.
- `cargo check --locked -p yetlet-browser --target wasm32-unknown-unknown`: PASS.
- `cargo clippy --locked -p yetlet -p yetlet-browser --all-targets -- -D warnings`:
  PASS.
- `cargo fmt --all -- --check`, `cargo xtask check docs`, and
  `git diff --check`: PASS. Source files were formatted by the installed
  `rustfmt` after an unsupported structured formatting recipe; no unrelated
  source files changed.
- Changed implementation paths:
  `crates/yetlet/src/view/tags.rs`, `crates/yetlet/src/view/mod.rs`,
  `crates/yetlet-browser/src/dom/mod.rs`. Documentation: this plan,
  `docs/VIEW-COMPONENTS.md`, `docs/COMPATIBILITY.md`, `docs/BACKLOG.md`.

This acceptance is limited to additive authoring API source and bounded tests.
No physical browser interaction, release/registry publication, Git commit,
push, PR, merge, deployment or cache deletion is claimed or authorized.
