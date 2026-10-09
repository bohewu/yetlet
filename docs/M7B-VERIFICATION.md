# M7b Verification — Keyed Binding and Static-Slot Authoring Prototype

**Date:** 2026-10-09
**Task:** YT-035 / M7b
**Status:** Implemented and locally verified as an **isolated experimental prototype**. It is not a stable Yetlet renderer, component API, VDOM, signal runtime, macro DSL, or production route.

## Scope and architecture

M7a showed that keyed direct DOM and static-slot approaches preserved identity while avoiding whole-subtree reconstruction for one bounded workload. M7b tests whether a small authoring model can express the corresponding semantics without moving benchmark internals into production crates.

The new `examples/keyed-bindings` workspace member is isolated from Todo Browser and Search Island. It provides:

- `ComponentIdentity`: validated component and instance identifiers shared by native SSR and browser binding.
- `KeyedListTemplate<T, K, Message>`: one deliberately fixed list shape with:
  - a typed stable-key function,
  - one dynamic label text slot,
  - allowlisted static/dynamic inert attributes,
  - one optional conditional text slot,
  - one optional local row-click action that produces a typed `Message`.
- Native `ssr_view` rendering through the existing `yetlet-core::View` and `yetlet-html` escaping policy.
- Browser-only `BoundKeyedList<T, K, Message>` for fail-closed SSR binding, keyed reconciliation, and explicit listener disposal.

The prototype does **not** introduce a new production dependency for `yetlet-core`, `yetlet-dom`, Todo Browser, or Search Island. It does not add framework-wide state tracking, automatic effects, remote events, compiler macros, or a general tree diff.

## Compiled authoring example

The following pattern is also a Rust documentation test:

```rust
use yetlet_keyed_bindings::{ComponentIdentity, KeyedListTemplate, TemplateError};

#[derive(Clone)]
struct Item { id: u64, label: String, selected: bool }
#[derive(Clone, Copy)]
enum Message { Select(u64) }

fn template() -> Result<KeyedListTemplate<Item, u64, Message>, TemplateError> {
    KeyedListTemplate::new(
        ComponentIdentity::new("item-list", "example")?,
        |item: &Item| item.id,
        |item: &Item| item.label.clone(),
    )
    .dynamic_attribute("aria-selected", |item| Some(item.selected.to_string()))?
    .conditional_text("status", None, |item| item.selected.then(|| "Selected".into()))?
    .on_row_click("Select", Message::Select)
}
```

Native code calls `template.ssr_view(&items)`. Browser code supplies the same template/items and a local `FnMut(Message)` dispatcher to `BoundKeyedList::bind_existing`. No serialized DOM event or server callback is involved.

## Authoring and trust-boundary rules

- Component/instance/slot identifiers are lowercase token values.
- Keys must be non-empty, control-character-free, and unique within one render.
- Attribute names must use the lowercase safe grammar. User attributes are restricted to `class`, `title`, `data-*`, and `aria-*`; all case variants outside that grammar, every `data-yetlet-*`, inline handlers, `style`, and `srcdoc` are rejected.
- Duplicate static/dynamic attribute declarations are rejected instead of producing ambiguous HTML.
- The built-in `label` slot cannot be shadowed, and only one conditional slot/action exists in this prototype.
- Action labels must contain accessible non-control text.
- Dynamic text/attribute values are serialized through the existing escaping renderer.

## SSR binding behavior

Before any listener is attached or DOM value is updated, browser binding preflights the complete fixed shape:

1. Root tag plus `data-yetlet-component` and `data-yetlet-instance`.
2. Declared list attributes.
3. Exact keyed row set with no missing, unknown, or duplicate key.
4. Row tag/static/dynamic attributes.
5. Exact direct child-node shape: element-only rows plus direct label, optional conditional, and action elements in the expected order, with exact tags, slot/action IDs, values, class, button type, and single text-node content.

Any mismatch returns an error and leaves the SSR nodes unbound. A successful initial bind reuses the original root, keyed rows, label element, and label text node; it does not remount the SSR subtree.

## Browser evidence

`python3 scripts/smoke_keyed_bindings.py --profile release` runs an isolated Chrome 155 profile and local static server. It verified:

- altered component instance/key, unexpected direct row text, and a nested label element with matching text all fail closed without replacing the retained SSR root/row or setting `data-yetlet-bound`;
- successful binding preserves root, row, label, and label-text identity;
- a local typed `DemoMessage::Toggle` updates label text, `data-completed`, `aria-pressed`, and inserts/removes the conditional slot;
- outside input focus remains intact;
- reverse, append, and remove operations preserve an untouched keyed row;
- explicit disposal removes all installed row listeners and the bound marker; clicking afterward has no effect;
- dropping an already-disposed old handle cannot clear a newer binding's marker or permit a concurrent duplicate binding;
- no HTTP request, WebSocket, failed browser resource, or JavaScript exception occurs after module loading;
- external module loading works under strict CSP with `wasm-unsafe-eval`, without `unsafe-inline` or general `unsafe-eval`.

## Build and artifact isolation

Commands:

```bash
cargo test --locked -p yetlet-keyed-bindings --all-targets
cargo test --locked -p yetlet-keyed-bindings --doc
cargo check --locked -p yetlet-keyed-bindings --target wasm32-unknown-unknown
python3 scripts/build_keyed_bindings.py --release
python3 scripts/smoke_keyed_bindings.py --profile release
```

Retained M7b-only release artifact (before M7c):

| Field | Value |
|---|---:|
| M7b content digest | `63da15682051d80d` |
| WASM | 149,542 bytes; 47,264 local gzip bytes |
| wasm-bindgen JS module | 18,325 bytes; 4,093 local gzip bytes |
| External loader | 172 bytes; 150 local gzip bytes |
| Generated fixture HTML | 1,768 bytes |

These are the retained M7b-only artifact sizes from this historical gate, not transfer measurements or a production bundle budget. The same build command now includes M7c and therefore produces the later combined values recorded in [M7c verification](M7C-VERIFICATION.md). Todo Browser and Search Island dependency graphs exclude both M7 experimental crates. Their release assets remained byte-identical at digest `3f3c5f074478c525` before and after the M7b build.

## Final closeout after reviewer repairs

The source was frozen after repairing three reviewer P2 findings: exact child-node/text-slot preflight, lowercase-only reserved attribute protection, and ownership-safe disposal/drop behavior. WebCodex then ran the complete closeout sequence in one terminal Job (`assertion_name=yetlet-m7b-complete-validation`) **after those repairs**.

| Final check | Observed result |
|---|---|
| `cargo fmt --all -- --check` | PASS |
| `python3 scripts/check_docs.py` | PASS — 23 English documents, 38 backlog items, links/Markdown/whitespace/DAG valid |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | PASS |
| `cargo test --workspace --all-targets --locked` | **73/73 unit and integration tests PASS** |
| `cargo test --doc --workspace --locked` | **5/5 doctests PASS** |
| wasm32 checks | Todo Browser, Search Island, M7a render benchmark, and M7b keyed bindings PASS |
| Existing Chrome regression | Chrome 155 Todo/SSR/Search/API/CSP E2E PASS on debug artifacts |
| Existing Firefox regression | Firefox 157 Todo/SSR/Search/API/keyboard E2E PASS on debug artifacts |
| Release Browser/Search rebuild | PASS; digest and every asset hash remained `3f3c5f074478c525` |
| M7a/M7b isolated release builds | PASS |
| M7b isolated Chrome smoke | PASS, including all repaired mismatch and stale-owner paths |
| Production dependency isolation | Todo Browser/Search graphs exclude both M7 experimental crates |
| `git diff --check` and resource teardown | PASS; no benchmark/browser/server Job remained active |

The Chrome/Firefox rows describe the existing regression suites and their actual debug profile. Release compatibility is supported separately by byte-identical Browser/Search release assets; this document does not claim a newly executed release-profile Chrome/Firefox matrix.

## Known limitations and M7c boundary

- The shape is fixed to `ul > li > label span + optional conditional span + optional button`.
- There is one event type, one conditional slot, and no nested component, fragment, arbitrary child topology, async effect, or multi-root support.
- The browser dispatcher is manually owned. It is not yet a reactive graph, scheduler, batcher, dependency tracker, or automatic lifetime system.
- Reconciliation validates keys before mutation, but an unexpected browser DOM API failure after mutation begins has no transactional rollback.
- The demonstration data is deterministically compiled into native and browser targets; this is not generic hydration-state serialization or resumability.
- Chrome validates this M7b slice. The complete Chrome/Firefox lifecycle, selection, listener, and form-state matrix remains YT-037.
- No new performance claim is made. M7a remains the update-strategy measurement; M7b evaluates authoring and correctness boundaries.

M7c/YT-036 now supplies an explicitly owned/disposable browser-local graph around this binding; see [M7c verification](M7C-VERIFICATION.md). The next task is **YT-037 / M7d**, broader Chrome/Firefox keyed lifecycle, focus, selection and form-state coverage.
