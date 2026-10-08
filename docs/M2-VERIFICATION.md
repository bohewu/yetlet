# M2 Browser/WASM verification

**Date:** 2026-10-08
**Status:** Local evidence confirmed; NOT a remotely executed CI run or independent formal review.
**Backlog:** YT-008, YT-009, YT-010

## Code and architecture

- `yetlet-dom`：Browser `Document.createElement` / `createTextNode` DOM renderer；無 `innerHTML`，僅安全的 tag/attribute subset。
- `examples/todo-ui`：Native/Wasm 同源 `TodoList`、本地純函數 `toggle_first`。
- `examples/todo-browser`：Rust/WASM mount + local click event listener（重掛載時會移除舊 listener）。
- `scripts/build_browser.py`：`wasm-bindgen --target web`；`dist/manifest.json` + 內容 hash 版本化的 JS/Wasm/CSS assets。
- `examples/todo-server`：`GET /app` shell 與 `/assets/*`；`GET /` 維持零 JS/Wasm SSR。無 server-side UI event circuit。

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
| Native Rust tests | 16/16 PASS (post-format final validation) |
| Browser target compilation | PASS；包含 shared UI 與 DOM renderer |
| Build-time asset manifest | PASS，使用 SHA-256 內容 digest 生成版本化 URL |
| WASM mount | Headless Chrome 155 成功載入並 mount |
| Cross-target TodoList parity | Browser `outerHTML` 與 native SSR `ul.todo-list` byte-equal |
| Local click behavior | 兩次 click：完成狀態 false → true → false，更新 class 與 status text |
| Network boundary | Click 沒有發送 HTTP；無 WebSocket |
| Browser errors | 無資源 4xx/5xx、JavaScript exceptions 或 console errors |
| CSP | SSR 無 script；Browser `'wasm-unsafe-eval'`、無 `'unsafe-eval'`/`'unsafe-inline'` |

測得 debug build `yetlet_browser_bg.wasm` **281,324 bytes（未壓縮）**。這不是正式效能基準，尚未測 TTFB、interactivity latency、release size 或其他瀏覽器。

## Decisions / open gates

M2 選擇最小的 shared `View` tree + owned-subtree replacement，而不是提早承諾 fine-grained VDOM diff 或 compile-time static-template IR。此版仍是 prototype：有限 HTML 子集、尚無 island hydration/state manifest、typed remote action、IME/keyboard 測試矩陣、資安正式審查。

下一階段：YT-011–YT-013 SSR + Islands；YT-014 typed local action abstraction 可並行獨立做。參閱 [ADR 0003](adr/0003-renderers-and-loading.md)。
