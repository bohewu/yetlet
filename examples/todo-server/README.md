# Todo SSR and Browser Sample

This is Yetlet's integrated full-stack sample. It combines native SSR, a browser/WASM route, a lazy Search Island, typed HTTP actions, request correlation, keyed local DOM updates and explicit runtime disposal.

## Run

From the repository root:

```bash
cargo xtask build browser && cargo run -p yetlet-todo-server
```

Open:

- `http://127.0.0.1:3000/` for pure SSR;
- `http://127.0.0.1:3000/app` for the browser/WASM Todo route;
- `http://127.0.0.1:3000/search` for SSR with a lazy Search Island;
- `http://127.0.0.1:3000/controls` for the no-JS native draft preview;
- `http://127.0.0.1:3000/api/todos` for the typed Todo JSON resource.

The server never compiles WASM on request. Browser artifacts are built ahead of time into the ignored `dist/` directory. This sample is anonymous and in-memory; it is not a production authentication, persistence or CSRF reference.

## Shared native draft controls

The `/app` page includes the same [native control View](../todo-ui/src/controls.rs) used by `/controls`: input, textarea, single select, checkbox, readonly/disabled fields and explicit submit/reset buttons. Edits stay in the native controls through unrelated keyed list updates. Reset restores the current page's defaults; GET Preview shows bounded escaped query values without creating a Todo. It works without JS on `/controls`. Do not place secrets in this read-only preview's URL.

The View also declares root-relative and fragment links. DOM mount resolves them through the browser URL parser with the [YT-110 same-origin contract](../../docs/YT-110-DOM-CAPABILITIES.md); native SSR retains its app-owned URL/CSP policy. No new binding runtime, persistence or CSRF/auth qualification is implied.

## Verify

```bash
cargo test --locked -p yetlet-todo-server --all-targets
cargo xtask build browser
cargo xtask smoke browser-chrome
```

Complete retained evidence is in [M7j verification](../../docs/M7J-VERIFICATION.md).
