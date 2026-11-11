# Todo SSR and Browser Sample

This is Yetlet's integrated full-stack sample. It combines native SSR, a browser/WASM route, a lazy Search Island, typed HTTP actions, request correlation, keyed local DOM updates and explicit runtime disposal.

## Run

From the repository root, build only the native CSS and run without compiling
WASM (requires Rust/Cargo; no Node, Python or browser toolchain):

```bash
cargo xtask build todo-css
cargo run --locked -p yetlet-todo-server
```

For the Browser/WASM and lazy Search routes, build the browser assets instead:

```bash
cargo xtask build browser
cargo run --locked -p yetlet-todo-server
```
The existing twelve-sample catalog retains its exact one-line Todo command:

```sh
cargo xtask build browser && cargo run -p yetlet-todo-server
```

It selects the same Browser/WASM build and native server startup; the separate
commands above show the corresponding locked Cargo form.

Restart the server after rebuilding CSS: native SSR snapshots accepted CSS at
startup. The native-only command leaves any previously built browser output
intact; it does not create browser output on a clean checkout.

Startup wraps the existing Router with `App::from_router(...).serve_on(...)`.
`YETLET_LISTEN` retains the default `127.0.0.1:3000`. Port `0` uses a native
listener so the startup banner reports the actual OS-selected address.

Open:

- `http://127.0.0.1:3000/` for pure SSR;
- `http://127.0.0.1:3000/app` for the browser/WASM Todo route;
- `http://127.0.0.1:3000/search` for SSR with a lazy Search Island;
- `http://127.0.0.1:3000/controls` for the no-JS native draft preview;
- `http://127.0.0.1:3000/api/todos` for the typed Todo JSON resource.

The server never compiles WASM on request. Browser artifacts are built ahead of time into the ignored `dist/` directory. This sample is anonymous and in-memory; it is not a production authentication, persistence or CSRF reference.

## App-owned Rust styles and source-only Tag Builders

The Todo list, shell and interactive layout use `yetlet::view::tags` from this
checkout's YT-234/235 source. These helpers lower into the same `View` tree and
are **not included in the published SDK alpha.6**. Existing keyed list classes,
DOM identity, local events and explicit remote HTTP actions are preserved.

The private, host-only `stylegen/` package copies the finite YT-239 builder.
`styles::stylesheet()` explicitly composes `layout` and `tasks`; both the views
and generator use [shared class names](../todo-ui/src/style_names.rs). The
completed class is registered even though all initial tasks are open. The Flex
toolbar, Grid layout, spacing, typography and custom properties use Rust CSS;
`plain.css` supplies selectors and properties outside that finite vocabulary.
Below 768px the layout has one column; at 768px and above it has two. Both `/`
and `/app` load one external same-origin stylesheet, with no inline styles.

Browser builds retain the existing content-addressed `dist/assets/<digest>/`
graph and link order. Native builds write the private `dist/todo-css.json` CSS
record. On startup, the native adapter validates its fixed asset URL and exact
SHA-256, then serves the selected CSS from memory. Missing or invalid optional
output leaves ordinary no-JS SSR readable. Only `/assets` is a public static
root; the record, manifest and generator sources are not public routes.

The root keeps `Page::new` and strict `DEFAULT_CSP`; browser enhancement alone
uses the existing WASM CSP token. CSS responses have a CSS MIME type, nosniff,
no-store and strict CSP. No request runs Cargo or compiles CSS/WASM. See the
[YT-240 qualification and platform limits](../../docs/YT-240-TODO-STYLE-QUALIFICATION.md).

## Shared native draft controls

The `/app` page includes the same [native control View](../todo-ui/src/controls.rs) used by `/controls`: input, textarea, single select, checkbox, readonly/disabled fields and explicit submit/reset buttons. Edits stay in the native controls through unrelated keyed list updates. Reset restores the current page's defaults; GET Preview shows bounded escaped query values without creating a Todo. It works without JS on `/controls`. Do not place secrets in this read-only preview's URL.

The View also declares root-relative and fragment links. DOM mount resolves them through the browser URL parser with the [YT-110 same-origin contract](../../docs/YT-110-DOM-CAPABILITIES.md); native SSR retains its app-owned URL/CSP policy. No new binding runtime, persistence or CSRF/auth qualification is implied.

## Verify

```bash
cargo test --locked -p yetlet-todo-ui -p yetlet-todo-server --all-targets
cargo build --locked -p yetlet-todo-server
cargo xtask build browser
cargo xtask smoke browser-chrome
cargo xtask smoke browser-firefox --profile debug
cargo xtask check samples
cargo xtask check docs
```

Historical behavior evidence remains in [M7j verification](../../docs/M7J-VERIFICATION.md).
YT-240 adds bounded Linux source qualification; independent acceptance and public
release remain separate gates. No Style Showcase is registered.
