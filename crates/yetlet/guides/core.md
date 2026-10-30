# yetlet::view

**Experimental `0.9.0-alpha.5`, unpublished.** Requires Rust 1.99 or newer. This is one of Yetlet's sixteen library candidates; use the local checkout rather than assuming a registry release. APIs are pre-public, and bounded prototype verification is not production qualification.

`yetlet::view` describes UI as owned Rust values shared by the native HTML and browser DOM renderers. With `yetlet` default features disabled it has no external normal dependencies, browser runtime, server transport or implicit state scheduler.

For a first-app continuation covering typed data, reusable functions, lists and
conditions, follow [Write Views and reusable components](../../../docs/VIEW-COMPONENTS.md).

## Build a View

`View` is either `Text(String)` or `Element`. Construct elements with a static tag, inert attributes and child Views. This fragment is copied verbatim from the crate's existing `view_tree_is_renderer_neutral` test:

```rust
let tree: View = Element::new("ul")
    .attribute("class", "todos")
    .child(Element::new("li").child(View::text("first")))
    .into();
```

Import `yetlet::view::{Element, View}` in the caller. To see the shared component in an application, read [Todo UI](../../../examples/todo-ui/src/lib.rs) and run native SSR from the repository root:

```bash
cargo run --locked -p yetlet --features server-axum --example server-axum-minimal
```

## Contracts and limits

`Element` stores tag/attribute names and values without validating or escaping them. `View::text` also stores untrusted text as-is: the selected renderer must enforce its own policy. Select `yetlet::html` for HTML serialization or explicitly select a browser renderer for WASM; core alone does not render. Tags and attribute names have static string lifetimes, while dynamic values and children are owned.

The `binding` module supplies `ComponentIdentity`, `BindingKey`, `SlotName`, `InertAttributeName` and `SlotDescriptor`. Validated identity/key/slot types reject values outside their bounded syntax through `BindingContractError`; this vocabulary does not implement event handlers, a template compiler, hydration or resumability. See [the binding source](../src/view/binding.rs) and [ADR 0007](../../../docs/adr/0007-bounded-keyed-dom-and-local-graph.md).

## License and compatibility

Dual-licensed under [MIT](../../../LICENSE-MIT) or [Apache-2.0](../../../LICENSE-APACHE). The privately prepared `0.1.x` policy preserves documented behavior and Rust 1.99 in patches; public activation remains blocked. See [compatibility](../../../docs/COMPATIBILITY.md), [API classifications](../../../docs/API-CONTRACTS.md) and [developer guides](../../../docs/README.md). Links to companion packages and records assume this checkout; they do not imply public hosting or distribution.
