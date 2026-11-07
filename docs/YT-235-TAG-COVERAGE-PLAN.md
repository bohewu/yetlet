# YT-235 — Complete standard HTML tag authoring coverage (YT-234 follow-up)

**Status:** Done at local source and bounded verification scope; two independent Reviewer PASS.
**Date:** 2026-11-07.
**Branch:** `feature/view-authoring-api` on base `889c7f7977e897a3bdf2ee2052bbfe01c2ad5ffd`.
**Dependency:** YT-234 typed `TagElement<T>`, uncommitted on the same checkout, local review PASS.

## User decision and source of truth

The user approved extending YT-234's tag constructors to *all standard tags accepted by Yetlet's ordinary HTML View serializer*, retaining the original owned `Element`/`View` tree and the independent renderer policies. The first slice already supplies 13 names. Add this as a bounded source-only extension, not a runtime or HTML/DOM policy expansion.

Authoritative inventory is [YT-219](YT-219-HTML-INVENTORY.md), cross-checked against actual [HTML serializer](../crates/yetlet/src/html/mod.rs) and [Browser DOM vocabulary](../crates/yetlet-browser/src/dom/vocabulary.rs). The 2026-10-31 dated table inventories 113 standard HTML elements, plus 2 foreign namespace roots and 1 custom-element family. Its disposition partitions the 113 standard names into:
- 90 ordinary DOM-capable tags (89 default-admitted and opt-in `img`);
- 11 SSR-accepted **DOM-deferred** standard tags (`area`, `audio`, `map`, `video`, `track`, `picture`, `source`, `noscript`, `template`, `slot`, `selectedcontent`);
- 7 document/head-owned tags (`base`, `body`, `head`, `html`, `link`, `meta`, `title`), excluded from **ordinary View** authoring conveniences: use the existing document metadata/writer pathway;
- 5 forbidden executable/embedded tags (`script`, `style`, `iframe`, `object`, `embed`), excluded;
- non-HTML SVG/MathML roots and custom elements remain outside scope.

**Target is exactly 101 named constructors.** This is a pinned dated inventory, not a claim of automatic future WHATWG coverage. Keep documenting `Element::new` as the generic escape hatch, which may accept arbitrary lowercase names on native SSR but never implies browser DOM admission.

## Proposed public API additions

Keep `TagElement<T>` and existing 13 constructors; extend the existing private `define_tag!` macro with a single declarative coverage list defining all 101 constructors. Rust names use `snake_case`, static marker names use `UpperCamelCase`; avoid proc macros and new dependencies. Expose no JSX or new View representation.

All conventional void elements in this authoring subset (`area`, `br`, `col`, `hr`, `img`, `input`, `source`, `track`, `wbr`) must use the void marker with no `.child`/`.children`/`.text` method. Non-void tags retain inherited existing builder semantics. Add compile-fail doctests for representative newly-added void tags.

Provide **only selected common, properly scoped fluent setters**; other attributes stay available through `.attribute(name, value)`, which preserves append-only semantics:
- `.href(...)` on `a`, `area`;
- `.src(...)` on `img`, `audio`, `video`, `source`, `track`;
- `.alt(...)` on `img`, `area`;
- `.type_(...)` on `input`, `button`, `ol`, `source`;
- `.for_(...)` on `label`, `output`;
- `.name(...)` on supported form-associated tags `input`, `button`, `textarea`, `select`, `form`, `fieldset`, `output`, `details`;
- `.value(...)` on `input`, `button`, `option`, `data`, `li`, `meter`, `progress`.

These are examples of supported usage, not claims of full HTML attribute typing:

```rust
use yetlet::view::{View, tags::{div, a, img, table, tbody, tr, td}};

let page: View = div()
    .child(a().href("/about").text("About"))
    .child(img().src("/logo.png").alt("Logo"))
    .child(table().child(tbody().child(tr().child(td().text("Hello")))))
    .into();
```

All typed setters **set/replace** their own attribute, consolidating any earlier duplicates of that name in the typed builder; a later generic `.attribute` remains append-only and may cause a duplicate renderer error. Do not introduce side-effectful URL parsing or HTML attribute normalization at construction time. Existing `.class` / `.add_class`, `.placeholder`, `.id`, tree conversion and generic attribute APIs remain compatible.

## Renderer and security boundaries

- **No modifications to HTML serializer or DOM allowed tag/attribute sets, options or output policy.** The existence of a constructor alone must not promise that a given Browser DOM View is admissible. `img()` is only browser-admitted through explicit `DomOptions::with_image_policy`; default `validate` still rejects it.
- Deferred tags may be serialized by SSR under its existing bounded validation but fail browser DOM validation; explicitly document this and do **not** add their DOM capabilities.
- No `.style`, `on*` attribute sugar, raw HTML, hydration, WebGPU changes, JavaScript, new macros for consumers, or WASM runtime changes. Browser/CSP semantics remain unchanged.
- Unknown/unimplemented standard tags are not automatically exported; the contract is the dated 101-name inventory.
- Preserve Rust 1.99 and absence of additional crates or Cargo features. This is a library API extension independent of target; native and wasm32 compile.

## Acceptance checks and handoff

1. Add tests to exercise **every** constructor at compile-time and verify `Into<Element>` produces the intended tag name, with a single source of definitions. Verify the 101 total and each emitted tag/attribute text through existing SSR validation; do not overclaim content-model conformance for empty structural elements.
2. Existing 13 constructors and methods remain source compatible. Assert representative nested elements, void type-check failures, selected fluent setter availability and mixed-setter/raw-`attribute` precedence. Add compile-fail doctests for `div().href` and `br().child` as well as earlier `div().placeholder` / `input().child` examples.
3. Browser DOM tests: `img` is rejected with default `validate`, admitted with existing `ImagePolicy::SameOrigin` and a valid root-relative `src`; `video` (deferred) is not admitted. Selected `a().href` and nested ordinary table/semantic content pass the existing policies. Tests must not alter the renderer.
4. Verify same tree by old `Element::new` equality, native `render_to_string`, wasm32 compile of Yetlet and Yetlet-browser, relevant package tests/doctests/Clippy/format/docs and whitespace. No new validation infrastructure, release signing/publication or hosted/browser interaction is required.
5. Update [View guide](VIEW-COMPONENTS.md), [compatibility](COMPATIBILITY.md), [backlog](BACKLOG.md) and this plan with actual source and limitations. Preserve release-version claims: SDK 0.9.0-alpha.6 remains unchanged/published without this API.
6. **Independent ad hoc Reviewer reviews the plan before implementation**; after scoped code/tests, obtain independent implementation Reviewer PASS. Resolve only sourced blocking issues. No commit, push, PR, merge or cache deletion without separately explicit owner authority.

## YT-235 final local acceptance — 2026-11-07

This follow-up is Done at **source-only implementation and scoped verification**.
No published artifact contains the new API.

- Independent design Reviewer Job `wc_job_CxGZR0jVxunw3KqE`: terminal exit 0,
  `overall_status=pass`, zero issues and required followups.
- Independent implementation Reviewer Job `wc_job_N3kCuLn2vjLD-PkC`:
  terminal exit 0, `overall_status=pass`, zero issues and required followups.
  These are two distinct role-directed read-only Codex Jobs using the verified
  saved `gpt-6-astra` Reviewer role, not a claim of native managed dispatch.
  Effort is inherited according to the existing reasoning policy; there is no
  additional model/effort telemetry assertion.
- `cargo test --locked -p yetlet --lib`: PASS, 23 tests including all
  101 generated tag constructors, nested rendering and scoped attribute cases.
- `cargo test --locked -p yetlet --doc`: PASS, 3 ordinary and 4 compile-fail
  doctests (typed placeholder, child/void, link-only helpers).
- `cargo test --locked -p yetlet-browser --lib`: PASS, 20 tests including
  default-image denial, explicit image opt-in and deferred video denial.
- `cargo check --locked -p yetlet --target wasm32-unknown-unknown`: PASS.
- `cargo check --locked -p yetlet-browser --target wasm32-unknown-unknown`:
  PASS.
- `cargo clippy --locked -p yetlet -p yetlet-browser --all-targets -- -D warnings`:
  PASS.
- `cargo fmt --all -- --check`, `cargo xtask check docs` and
  `git diff --check`: PASS.
- Changed shared source: `crates/yetlet/src/view/tags.rs`, with the
  YT-234 shared `view/mod.rs` exposure unchanged; a bounded DOM test addition
  is in `crates/yetlet-browser/src/dom/mod.rs`. Documentation:
  this record, `docs/VIEW-COMPONENTS.md`, `docs/COMPATIBILITY.md`,
  `docs/BACKLOG.md` and `docs/ROADMAP.md`.

This does **not** requalify Browser visual/lifecycle behavior, expand the DOM
allowlist, promote package support, update package versions or publish SDK/CLI.
No Commit, Push, PR, Merge, release, deployment, or cache deletion is authorized
or performed. Branch remains `feature/view-authoring-api` on base `889c7f7`,
with the combined YT-234/YT-235 source/document delta uncommitted.
