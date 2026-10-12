# yetlet-dom

**Experimental `0.1.0`, unpublished.** Requires Rust 1.99 or newer. This is one of Yetlet's nine library candidates; use the local checkout rather than assuming a registry release. APIs are pre-public, and bounded prototype verification is not production qualification.

`yetlet-dom` mounts a shared View by constructing real browser nodes and replacing one owned subtree. It depends on `yetlet-core`; `wasm-bindgen` and `web-sys` are added only for `wasm32`. Native SSR does not require this crate.

## Mount an owned surface

`validate(&view)` is available on native and WASM targets and returns `Result<(), DomPolicyError>`. On WASM, `mount(&web_sys::Element, &view)` validates and constructs the replacement before clearing the old subtree; browser API failures return `JsValue`. There is no native `mount` implementation.

The [Search Island caller](../../examples/search-island/src/lib.rs) keeps complete SSR readable and replaces only its results host after activation. Follow [Todo setup](../../examples/todo-server/README.md), or run from the repository root after installing the documented WASM build prerequisites:

```bash
python3 scripts/build_browser.py
cargo run --locked -p yetlet-todo-server
```

Open `http://127.0.0.1:3000/search`. The external loader builds/loads assets explicitly; this crate does not load modules or issue HTTP actions.

## Supported subset and lifetime

The allowlisted tags are `main`, `section`, `div`, `article`, `header`, `h1`, `h2`, `p`, `span`, `ul`, `li`, `button`, `strong`, `input` and `label`. Attributes are limited to `id`, `class`, `type`, `data-id`, `data-completed`, `aria-live`, `aria-pressed`, `for`, `placeholder`, `maxlength` and `autocomplete`. Unsupported tags/attributes return `DomPolicyError` before mutation. Text uses DOM text nodes rather than `innerHTML`; URL, inline style and inline event attributes are excluded.

The caller owns the target and event listener lifetime. Keep listeners outside the replaced subtree or explicitly release/rebind them. Subtree replacement does not preserve child identity, focus or selection as a general contract, and unexpected DOM API failures are not transactional rollback. Choose the separate opt-in keyed crate for proven identity-sensitive collections. There is no general VDOM, hydration, resumability or GPU acceleration.

## License and compatibility

Dual-licensed under [MIT](../../LICENSE-MIT) or [Apache-2.0](../../LICENSE-APACHE). The privately prepared `0.1.x` policy preserves documented behavior and Rust 1.99 in patches; public activation remains blocked. See [compatibility](../../docs/COMPATIBILITY.md), [API classifications](../../docs/API-CONTRACTS.md) and [developer guides](../../docs/README.md). Links to companion packages and records assume this checkout; they do not imply public hosting or distribution.
