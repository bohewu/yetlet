# yetlet-dom-keyed

**Experimental `0.9.0-alpha.2`, unpublished.** Requires Rust 1.99 or newer. This is one of Yetlet's sixteen library candidates; use the local checkout rather than assuming a registry release. APIs are pre-public, and bounded prototype verification is not production qualification.

`yetlet-dom-keyed` is an opt-in owner for keyed browser children and event listeners. It depends on `yetlet-core`; browser dependencies (`wasm-bindgen`, `web-sys`) are target-specific. It does not require the reactive graph, HTML renderer or server adapter.

## Bind a keyed collection

On all targets, `validate_keyed_order` accepts validated `BindingKey` values and returns the desired order or `KeyedOrderError::DuplicateKey` before DOM mutation. Key syntax is validated separately by core.

On WASM, use `KeyedEntry<State>` for an element plus caller-defined cached slots/listeners. Construct `KeyedChildren` from matching existing direct children with `from_existing`, or from an empty root with `empty`. `reconcile` takes the complete desired item list and explicit key/update/create callbacks. `ReconcileStats` reports retained, inserted, removed and moved entries.

The complete [Todo Browser binding](../../examples/todo-browser/src/lib.rs) demonstrates preflight, keyed reconciliation and owned listeners. Follow [Todo setup](../../examples/todo-server/README.md), or run from the repository root after installing its browser prerequisites:

```bash
cargo xtask build browser
cargo run --locked -p yetlet-todo-server
```

Open `http://127.0.0.1:3000/app`. Only the local Todo list adopts keyed updates; server snapshots and Search results retain separate replacement surfaces.

## Ownership and failure boundaries

Retain `DomEventListener` for as long as its callback should be installed. Its handler is `'static`; explicit disposal or Drop removes the listener. `KeyedChildren` owns entries and their state; removal drops that state, and owner disposal/drop releases entries. Browser constructors/reconciliation return `JsValue` for invalid roots, disposed owners, callback errors or browser API failures.

Desired-key validation and missing-entry creation precede structural changes. Update callbacks can mutate retained nodes before an error; final DOM operations are not transactionally rolled back. Callers own callback atomicity, input-property/focus handling and the validity of created markup. This crate is not an HTML sanitizer, template compiler or general reconciler, and it does not confer generic hydration support. See [ADR 0007](../../docs/adr/0007-bounded-keyed-dom-and-local-graph.md) and [M7h evidence](../../docs/M7H-VERIFICATION.md).

## License and compatibility

Dual-licensed under [MIT](../../LICENSE-MIT) or [Apache-2.0](../../LICENSE-APACHE). The privately prepared `0.1.x` policy preserves documented behavior and Rust 1.99 in patches; public activation remains blocked. See [compatibility](../../docs/COMPATIBILITY.md), [API classifications](../../docs/API-CONTRACTS.md) and [developer guides](../../docs/README.md). Links to companion packages and records assume this checkout; they do not imply public hosting or distribution.
