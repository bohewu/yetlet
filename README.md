# Yetlet

**Yetlet** 是實驗中的 HTML-first Rust universal UI runtime：以共用 Rust `View`/元件來源支援 native SSR 與 Browser WASM DOM renderer，預設不需要遠端 event circuit。

> **狀態：M2 Browser prototype 已驗證，完整 MVP 尚未完成。** Islands、typed HTTP actions、resumability 和 WebGPU 均未實作；public API 尚不穩定。

## 設計原則

- HTML-first：純 SSR 無 JS/WASM；server 與 browser 使用獨立 build outputs。
- Local-event-first：DOM events 只在 browser 處理；如需 remote work，未來由 typed HTTP action 完成。
- Render/transport 分離：沒有 SignalR/LiveView-like persistent circuit，也不自行實作 HTTP/TLS/CSS engine。
- Strict CSP：SSR 安全 escaping，DOM renderer 限定安全 tag/attribute subset；`/app` 的 CSP 只額外允許 `'wasm-unsafe-eval'`（不允許 `'unsafe-eval'` 或 `'unsafe-inline'`）。

## 已實作與未實作

| Mode / path | 狀態 | 行為 |
|---|---|---|
| `GET /` | 已實作 | Native SSR TodoList，沒有 browser runtime |
| `GET /app` | M2 已實作 | 外部 JS loader + Rust/WASM，Browser DOM mount / local toggle |
| `GET /healthz` | 已實作 | 健康探針 |
| `GET /api/ping` | 已實作 | 獨立 JSON API 路徑 |
| `GET /search` | M3 規劃 | SSR + lazy island |
| `GET/POST /api/todos` | M4 規劃 | Typed data API / actions |
| Canvas/WebGPU | 後續研究 | 與 HTML DOM 獨立的 scene renderer |

同一份 `examples/todo-ui` 的 `TodoList` Rust 原始碼會在 server 與 browser targets 被編譯；browser 透過 `yetlet-dom` 直接建立 DOM、透過本地 listener 改變第一筆 Todo 狀態（不發出 HTTP 請求）。

## 執行範例

需 Rust stable（宣告 MSRV 1.85，**尚未實際測試 MSRV**）、Python 3.11+ 與 wasm32 target。已於 Rust 1.99、wasm-bindgen 0.2.129、Chrome 155 測試。

```bash
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version 0.2.129 --locked \
  --root "$HOME/.local/share/yetlet-tools"

python3 scripts/build_browser.py
cargo run -p yetlet-todo-server
```

可開啟 `http://127.0.0.1:3000/`（SSR）及 `http://127.0.0.1:3000/app`（Browser/WASM）。`dist/` 是 build-time 生成的 Git-ignored 資產與 hash manifest；server 不會在 request time 編譯 Wasm。未先執行 browser build 時，`/app` 明確回傳 503。

`python3 scripts/build_browser.py --release` 可產生 optimized browser bundle；不要用 debug Wasm bytes 代表 production 性能。

## 驗證

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --all-targets --locked
cargo check --locked -p yetlet-todo-browser --target wasm32-unknown-unknown

python3 scripts/check_docs.py
cargo build --locked -p yetlet-todo-server
python3 scripts/smoke_browser.py
```

Browser smoke 會使用獨立的 headless Chrome profile 和暫時 localhost server，只停止自己啟動的程序。驗證 SSR/DOM TodoList 完全一致、兩次 click 本地更新 class/data/text、無 click HTTP/WS、無 browser errors。執行命令與結果整理於 [M2 驗證紀錄](docs/M2-VERIFICATION.md)。

## Workspace

- `yetlet-core`：Renderer-neutral View
- `yetlet-html`：Safe native HTML serializer
- `yetlet-dom`：Browser direct DOM renderer（有限安全 HTML subset）
- `yetlet-server-axum`：HTTP response / CSP adapter
- `examples/todo-ui`：共用 TodoList + state transition
- `examples/todo-browser`：Rust/WASM entrypoint + local event binding
- `examples/todo-server`：SSR / Browser / API routes
- `scripts/build_browser.py`：Hashed browser asset pipeline

**目前限制：** Browser 更新是 owned-subtree replacement，尚無 fine-grained signal graph、hydration/resumability 或跨網路 typed actions；也沒有跨框架性能比較。此版本為 prototype，不宣稱 production-ready。

## 文件

- [架構與模組邊界](docs/ARCHITECTURE.md)
- [完整 MVP 規格](docs/MVP.md)
- [Roadmap](docs/ROADMAP.md)
- [Backlog](docs/BACKLOG.md)
- [M2 驗證紀錄](docs/M2-VERIFICATION.md)
- [ADR 0001](docs/adr/0001-separation-of-concerns.md)
- [ADR 0002](docs/adr/0002-events-and-actions.md)
- [ADR 0003](docs/adr/0003-renderers-and-loading.md)

下一條主路徑為 `YT-011` 的 SSR island loader → `YT-012`/`YT-013`；`YT-014` typed LocalExecutor 可獨立推進。授權由維護者後續決定。
