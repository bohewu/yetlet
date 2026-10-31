# yetlet-browser::dom

Mount a shared Rust View into an explicitly owned browser surface. This optional
renderer replaces a subtree; native SSR works without selecting it.

**Experimental SDK `0.9.0-alpha.5`, published on crates.io.** `yetlet-browser`
selects `dom` by default; actual mounting requires the WASM target and browser
host. The commands below run checkout examples at their documented scope.


## Prerequisites

Use this checkout with the pinned Rust/Cargo toolchain. Native `validate` is
available without a browser. Actual mounting requires WASM and a browser host.
Before the commands below, prepare `wasm32-unknown-unknown` and the Cargo.lock-
matching wasm-bindgen CLI (0.2.129): follow the [root browser setup](../../../README.md#choose-your-next-step),
including tool installation location/PATH. Run commands from the repository root;
stop another server using port 3000 first.

## Minimal integration

### Mount an owned surface

`validate(&view)` is available on native and WASM targets and returns `Result<(), DomPolicyError>`. On WASM, `mount(&web_sys::Element, &view)` validates and constructs the replacement before clearing the old subtree; browser API failures return `JsValue`. There is no native `mount` implementation.

The [Search Island caller](../../../examples/search-island/src/lib.rs) keeps complete SSR readable and replaces only its results host after activation. Follow [Todo setup](../../../examples/todo-server/README.md), or run from the repository root after installing the documented WASM build prerequisites:

```bash
cargo xtask build browser
cargo run --locked -p yetlet-todo-server
```

Open `http://127.0.0.1:3000/search`. The external loader builds/loads assets explicitly; this crate does not load modules or issue HTTP actions.

## Expected result

At `http://127.0.0.1:3000/search`, complete SSR is readable before browser
activation; the selected island updates only its owned results surface. Static
validation alone does not mount anything or prove browser navigation acceptance.

## Configuration and errors

### Supported subset and lifetime

The finite [YT-110 vocabulary and contract](../../../docs/YT-110-DOM-CAPABILITIES.md#dom-vocabulary) supports 38 tags and 42 named attributes: ordinary semantic structure, explicit ARIA/data names, native form controls and bounded navigation. Unsupported names, duplicates, supported void-element children and invalid selected control structures fail before mutation. No arbitrary attribute-prefix expansion, inline style, inline handler or resource URL is admitted. Text uses nodes rather than `innerHTML`.

Initial input/textarea/select/checkbox defaults are declarative; live values, chosen events, reset and submission remain browser/application-owned. Textarea text has LF normalization and preserves an initial newline across SSR and DOM. See the shared [native controls example](../../../examples/todo-ui/src/controls.rs), available at `/app` and the no-JS `/controls` route in Todo.

`href` accepts a root-relative path or fragment; form `action` accepts a root-relative path. Static `validate` checks the narrow spelling. WASM mount additionally uses the browser URL parser and effective document base, rejects non-HTTP(S), cross-origin or credential-bearing destinations, and stores absolute URLs before touching the target. Therefore mount can reject a View that passed static validation. Later base changes do not retarget accepted attributes. The broader SSR URL policy remains separate; see [navigation limits](../../../docs/YT-110-DOM-CAPABILITIES.md#navigation-and-submission-boundary).

The caller owns the target and event listener lifetime. Keep listeners outside the replaced subtree or explicitly release/rebind them. Subtree replacement does not preserve child identity, focus or selection as a general contract, and unexpected DOM API failures are not transactional rollback. Choose the opt-in `dom-keyed` feature for proven identity-sensitive collections. There is no general VDOM, hydration, resumability or GPU acceleration.

## API reference

See [DOM source/rustdoc](../src/dom/mod.rs), the [current API reference](../../../docs/API-REFERENCE.md)
and the linked supported-vocabulary/navigation contract below.

### License and compatibility

Dual-licensed under [MIT](../../../LICENSE-MIT) or [Apache-2.0](../../../LICENSE-APACHE).
SDK `0.9.0-alpha.5` is published; APIs remain Experimental. Read
[compatibility](../../../docs/COMPATIBILITY.md) and the
[current API reference](../../../docs/API-REFERENCE.md) at the matching revision.
The historical evidence below retains its original candidate and scope; it does
not qualify every later checkout or promise production support.

<details>
<summary>Historical delivery context and evidence</summary>

**Experimental `0.9.0-alpha.5`, unpublished.** Requires Rust 1.99 or newer. This is one of Yetlet's sixteen library candidates; use the local checkout rather than assuming a registry release. APIs are pre-public, and bounded prototype verification is not production qualification.

`yetlet_browser::dom` mounts a shared View by constructing real browser nodes and replacing one owned subtree. It depends on `yetlet::view`; `wasm-bindgen` and `web-sys` are added only for `wasm32`. Native SSR does not require this crate.

</details>
