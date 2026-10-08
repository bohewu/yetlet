# M2 Browser/WASM verification

**Date:** 2026-10-08
**Status:** Local evidence confirmed. This is neither a remotely executed CI run nor an independent formal review.
**Backlog:** YT-008, YT-009, YT-010

## Code and architecture

- `yetlet-dom` uses `Document.createElement` and `createTextNode` with a restricted safe HTML tag/attribute subset. It does not use `innerHTML`.
- `examples/todo-ui` provides the shared native/WASM `TodoList` source and pure local `toggle_first` state transition.
- `examples/todo-browser` mounts through Rust/WASM and registers a local click listener that can be detached on remount.
- `scripts/build_browser.py` runs `wasm-bindgen --target web` and produces content-addressed JS/WASM/CSS assets plus `dist/manifest.json`.
- `examples/todo-server` serves the `GET /app` shell and `/assets/*` files, while `GET /` remains script-free SSR. No server-side UI event circuit exists.

## Local verification commands

```bash
cargo test --workspace --all-targets --locked
cargo check --locked -p yetlet-todo-browser --target wasm32-unknown-unknown
python3 scripts/build_browser.py
cargo build --locked -p yetlet-todo-server
python3 scripts/smoke_browser.py
```

| Assertion | Observed |
|---|---|
| Native Rust tests | 16/16 PASS at the M2 post-format final validation |
| Browser target compilation | PASS, including shared UI and DOM renderer |
| Build-time asset manifest | PASS; SHA-256 content digest produces versioned URLs |
| WASM mount | Successful in headless Chrome 155 |
| Cross-target TodoList parity | Browser `outerHTML` byte-identical to native SSR `ul.todo-list` |
| Local click behavior | Two clicks switched completed state false → true → false and updated class/status text |
| Network boundary | Click dispatched no HTTP requests; no WebSocket |
| Browser errors | No 4xx/5xx asset responses, JS exceptions, or console errors |
| CSP | SSR ships no script; Browser mode allows `'wasm-unsafe-eval'` but not `'unsafe-eval'` or `'unsafe-inline'` |

The debug build's `yetlet_browser_bg.wasm` was **281,324 bytes uncompressed**. This is a single snapshot, not a production performance baseline; TTFB, interactivity latency, release size, and compatibility in other browsers were not measured.

## Decisions and open gates

M2 chose a minimal shared `View` tree with owned-subtree replacement instead of prematurely committing to fine-grained VDOM diffing or compile-time static-template IR. This is a prototype with a restricted HTML subset. At M2, island hydration/state manifests, typed remote actions, IME/keyboard test matrices, and formal security review were still pending.

The subsequent M3 work is documented separately. See [M3 verification](M3-VERIFICATION.md), [Roadmap](ROADMAP.md), and [ADR 0003](adr/0003-renderers-and-loading.md).
