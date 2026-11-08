# yetlet-browser::dom_keyed

**Experimental SDK `0.9.0-alpha.6`, available on crates.io.** Requires Rust 1.99
or newer. Select the `dom-keyed` feature of `yetlet-browser` explicitly;
actual DOM/listener operations require WASM and a browser host. Native
`validate_keyed_order` is separate from browser execution.

`yetlet_browser::dom_keyed` is an opt-in owner for keyed browser children and event listeners. It depends on `yetlet::view`; browser dependencies (`wasm-bindgen`, `web-sys`) are target-specific. It does not require the reactive graph, HTML renderer or server adapter.

For a standalone app using the published CLI/SDK, follow the
[browser-only walkthrough](https://github.com/bohewu/yetlet/blob/master/docs/BROWSER-APP.md). Its existing starter
selects this feature plus `dom` and a separate local state graph.

## Bind a keyed collection

On all targets, `validate_keyed_order` accepts validated `BindingKey` values and returns the desired order or `KeyedOrderError::DuplicateKey` before DOM mutation. Key syntax is validated separately by core.

On WASM, use `KeyedEntry<State>` for an element plus caller-defined cached slots/listeners. Construct `KeyedChildren` from matching existing direct children with `from_existing`, or from an empty root with `empty`. `reconcile` takes the complete desired item list and explicit key/update/create callbacks. `ReconcileStats` reports retained, inserted, removed and moved entries.

The complete [Todo Browser binding](https://github.com/bohewu/yetlet/blob/master/examples/todo-browser/src/lib.rs) demonstrates preflight, keyed reconciliation and owned listeners. Follow [Todo setup](https://github.com/bohewu/yetlet/blob/master/examples/todo-server/README.md), or run from the repository root after installing its browser prerequisites:

```bash
cargo xtask build browser
cargo run --locked -p yetlet-todo-server
```

Open `http://127.0.0.1:3000/app`. Only the local Todo list adopts keyed updates; server snapshots and Search results retain separate replacement surfaces.

## Ownership and failure boundaries

Retain `DomEventListener` for as long as its callback should be installed. Its handler is `'static`; explicit disposal or Drop removes the listener. `KeyedChildren` owns entries and their state; removal drops that state, and owner disposal/drop releases entries. Browser constructors/reconciliation return `JsValue` for invalid roots, disposed owners, callback errors or browser API failures.

Desired-key validation and missing-entry creation precede structural changes. Update callbacks can mutate retained nodes before an error; final DOM operations are not transactionally rolled back. Callers own callback atomicity, input-property/focus handling and the validity of created markup. This module is not an HTML sanitizer, template compiler or general reconciler, and it does not confer generic hydration support. See [ADR 0007](https://github.com/bohewu/yetlet/blob/master/docs/adr/0007-bounded-keyed-dom-and-local-graph.md) and [M7h evidence](https://github.com/bohewu/yetlet/blob/master/docs/M7H-VERIFICATION.md).

## License and compatibility

Dual-licensed under [MIT](https://github.com/bohewu/yetlet/blob/master/LICENSE-MIT) or
[Apache-2.0](https://github.com/bohewu/yetlet/blob/master/LICENSE-APACHE). SDK `0.9.0-alpha.6` is distributed;
APIs remain Experimental. Read [compatibility](https://github.com/bohewu/yetlet/blob/master/docs/COMPATIBILITY.md),
the [current API reference](https://github.com/bohewu/yetlet/blob/master/docs/API-REFERENCE.md) and the matching
source/rustdoc. Repository example execution and historical acceptance retain
their own platform/feature/source limits.

## Historical delivery context

The following original descriptions belong to earlier preparation checkpoints;
they do not describe current registry availability or package layout.

> **Experimental `0.9.0-alpha.5`, unpublished.** Requires Rust 1.99 or newer. This is one of Yetlet's sixteen library candidates; use the local checkout rather than assuming a registry release. APIs are pre-public, and bounded prototype verification is not production qualification.

> The privately prepared `0.1.x` policy preserves documented behavior and Rust 1.99 in patches; public activation remains blocked. Links to companion packages and records assume this checkout; they do not imply public hosting or distribution.