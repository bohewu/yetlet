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
- `http://127.0.0.1:3000/api/todos` for the typed Todo JSON resource.

The server never compiles WASM on request. Browser artifacts are built ahead of time into the ignored `dist/` directory. This sample is anonymous and in-memory; it is not a production authentication, persistence or CSRF reference.

## Verify

```bash
cargo test --locked -p yetlet-todo-server --all-targets
cargo xtask build browser
cargo xtask smoke browser-chrome
```

Complete retained evidence is in [M7j verification](../../docs/M7J-VERIFICATION.md).
