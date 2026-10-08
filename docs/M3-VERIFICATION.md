# M3 SSR Islands — verification

**Date:** 2026-10-08
**Status:** Local implementation and Chrome E2E evidence. No push, no GitHub Actions execution, no independent formal review.
**Backlog:** YT-011, YT-012, YT-013, YT-017

## Implementation

- `GET /search` is rendered by `yetlet-todo-ui::search_page` through the native HTML renderer. SSR markup already contains a labeled search input, two initial TodoList items, an aria-live count, and a distinct island boundary.
- `assets/search-loader.js` is the small **external** JavaScript bootstrap. It registers local `focusin` and `input` listeners after the SSR DOM exists, without importing the search WASM until interaction.
- `examples/search-island` exports a `mount_search` WASM function. Mount checks its named island boundary/version, preserves the server-rendered input and initial results, and owns only the `#search-results` subtree plus the count text.
- `scripts/build_browser.py` compiles both Browser app and Search island ahead of time, runs matching wasm-bindgen, and references content-addressed assets from `dist/manifest.json`.
- `examples/todo-server` validates the generated manifest's hash-based URLs before inserting script/style links, applying CSP with `'wasm-unsafe-eval'` (not `'unsafe-eval'` / `'unsafe-inline'`). Missing/invalid assets leave the SSR result page intact without client enhancement.
- SSR state uses an explicit, bounded `SearchStateV1` JSON DTO (component, schema_version, Todo id/title/completed only). The HTML renderer escapes the attribute; browser Wasm validates size, schema, item count, duplicate IDs and unknown fields before any event binding. It filters this **server-supplied initial data**, not an independently compiled copy. This phase does **not** implement HTTP data loading, event forwarding, persistent sessions, or general-purpose resumability.

## Verification commands

```bash
cargo test --workspace --all-targets --locked
cargo check --locked -p yetlet-search-island --target wasm32-unknown-unknown
python3 scripts/build_browser.py
cargo build --locked -p yetlet-todo-server
python3 scripts/smoke_browser.py
```

The extended Chrome 155 E2E checks:

1. `/` remains zero-script SSR and `/app` still mounts and locally toggles shared TodoList.
2. `/search` returns meaningful SSR results, safe external script references and strict CSP.
3. On load the tiny loader is present, but no `yetlet_search.js` / `yetlet_search_bg.wasm` request has occurred.
4. A simulated bubbling `input` event activates the island and dynamically loads its WASM. The initial SSR result HTML remains unchanged, as do the input node and page heading.
5. Search is case-insensitive; a no-match query shows an empty state; clearing restores exactly the original SSR TodoList markup.
6. Subsequent input events make no HTTP requests and do not create WebSockets. No unexpected 4xx/5xx resources or browser JS errors occurred in the positive flow.
7. Unit tests verify state roundtrip, unknown/oversized payload rejection, duplicate IDs, version mismatch and HTML escaping. A negative Chrome scenario corrupts `schema_version`: the island refuses activation and leaves the readable SSR result markup intact (the expected enhancement error is reported).

**Testing nuance:** Headless Chrome may retain input focus across navigations, so calling `focus()` again is not proof of a new `focusin` event. The E2E deliberately dispatches an `input` event to model typing, then checks local UI behavior and network boundaries. The loader also supports real `focusin` as an additional activation trigger.

## Boundaries still open

- **YT-017 delivered for Search Island only:** versioned JSON state, explicit field allowlist, bounded parsing and fail-closed SSR fallback are verified. There is **no generic component state transfer registry**, general hydration reconciliation or resumability yet.
- **YT-014–YT-016:** semantic typed actions and HTTP data API are pending; the search demo uses a deterministic in-memory fixture on both targets.
- **YT-018:** complete CSP/security/browser matrix is pending. Current validation is headless Chrome on one Runner.
- **YT-019:** bundle, startup time, memory and production performance comparisons are not certified.
- Renderer still uses owned-subtree replacement rather than fine-grained reactive subscriptions.

See [Roadmap](ROADMAP.md), [Backlog](BACKLOG.md), and [ADR 0003](adr/0003-renderers-and-loading.md).
