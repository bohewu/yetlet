# Yetlet MVP — vertical-slice specification

**Status:** Draft / M0–M2 validated locally; M3–M5 still pending
**Product goal:** 證明「同一份 Rust component source 可在 native SSR 與 browser WASM 使用」，而 SSR 不強制下載 JavaScript、不依賴 server event circuit。

本文件是完整 MVP 的驗收契約。已完成 M1 Native SSR 與 M2 Browser/WASM DOM demo（`/app`，本地事件、共用 TodoList）；`/search` Islands、Todo API/typed actions、hydration state、production metrics 仍未完成。請參閱 [M2 實測](M2-VERIFICATION.md)。

## 1. 使用情境與範圍

Demo 是一個簡單的 Todo / Search app，有四個 routes：

| Route | 初始輸出 | Browser runtime | 必須展示 |
|---|---|---|---|
| `GET /` | SSR HTML | 無 JS、無 WASM | 純 HTML 呈現共用元件資料；導航可用 |
| `GET /search` | SSR HTML | 小 JS loader + 單一 Rust/WASM island | 搜尋輸入、本地 state、HTTP 查詢 |
| `GET /app` | shell HTML | full Rust/WASM DOM runtime | client mount、local updates、共用 TodoList |
| `GET/POST /api/todos` | JSON | 無特定要求 | typed input/error/status；供 browser actions 使用 |

可使用 in-memory store / deterministic fixtures，初期不接資料庫、auth provider、router plugin 或 multi-tenancy。可提供 progressive enhancement 的 native HTML form fallback。

## 2. Normative behavioral requirements

### Functional (F)

- **F01**：共用 Rust `TodoList`/`TodoItem` component source 在 native 與 browser target 都可編譯並正確 render。
- **F02**：`/` 的 response 未包含強制執行的 JS/WASM assets；關閉 JavaScript 仍有可讀的完整頁面。
- **F03**：`/search` SSR 初始 HTML 可讀；互動 island 才載入對應 Wasm，輸入處理在 browser 本地。
- **F04**：`/app` 的 browser runtime 可完成 mount 與至少一個 local state transition（例如切換完成狀態）。
- **F05**：跨網路互動使用明確 `/api/todos` HTTP call，不傳送 raw DOM events，也不建立預設 WebSocket/SignalR-like circuit。
- **F06**：SSR 初始化所需 state 有清楚 schema 與 version；非法/過期資料不得無聲接管 DOM。
- **F07**：API 輸入不合法時提供 4xx + typed error；未預期錯誤不外洩敏感堆疊/設定。

### Nonfunctional (N)

- **N01**：Rust stable + lockfile 可重現 build，並能編譯 native 與 `wasm32-unknown-unknown` target。
- **N02**：HTML text/attribute 可證明 escaped；Hydration state 不含 secret 且不產生可執行注入。
- **N03**：strict CSP 可不依賴 `unsafe-inline` 運作（island/js loader 以 external asset 載入）。
- **N04**：keyboard、focus 與常見 IME/browser native text input 正常；勿攔截一般 HTML 行為。
- **N05**：測試須能在 CI 或有明確記錄的本機命令執行，文件說明所需 toolchain。
- **N06**：保留 baseline measurement：SSR TTFB、HTML bytes、初始必要 JS/WASM compressed size、首個可互動 island 的時間。**不預設未測過的性能結果。**

## 3. Acceptance matrix

| Test ID | Case | Expected |
|---|---|---|
| A01 | native Rust unit test: HTML escaping | HTML/attribute 特殊字元不能跨 context 注入 |
| A02 | integration: `GET /` with JS disabled | 有 TodoList HTML；0 script/wasm dependency |
| A03 | integration: `GET /search` | 有初始 HTML、external asset reference、well-formed island state |
| A04 | browser E2E: `/search` typing | input 在本地處理、需要查詢時只發明確 HTTP request |
| A05 | browser E2E: `/app` click | state 更新；網路沒有 event circuit 傳輸 |
| A06 | SSR/CSR cross-target: common component | 相同 fixture 符合明確 DOM/HTML snapshot contract |
| A07 | integration: API bad input | 4xx + typed error JSON；不洩露內部細節 |
| A08 | CSP E2E | strict policy 下沒被擋的必要資源、不需要 unsafe-inline |
| A09 | build native + browser | 編譯成功；無 server-only capability 進 browser build |
| A10 | toolchain & perf report | 命令、環境、測量值、限制完整；不宣稱未量測的 KPI |

**Definition of Done for MVP:** A01–A09 均有實際可查證的執行結果，A10 有可重現命令與 baseline。僅有 passing unit tests 不等於 browser/Wasm 或 CSP 已驗證。

## 4. Proposed ergonomic API (illustrative only)

```rust
// 提案用 DSL，不是已存在的公開 API：
App::new()
    .page("/", HomePage::ssr())
    .page("/search", SearchPage::island())
    .page("/app", TodoApp::browser())
    .api("/api/todos", todos_api)
    .run()
    .await;
```

```rust
// typed action 的方向；API 形狀尚待 spike：
enum TodoAction {
    Toggle { id: u64 },
    Create { title: String },
}

// Local events decide when to issue an action.
// Only semantic action input may cross the server boundary.
```

初期可以使用顯式 Rust builder/render calls，先確認能力，再引入 `#[component]` 和 `ui!` macro。請勿為追求 API 外觀推遲最早的可執行驗證。

## 5. Build and release envelope

- Native: Rust stable，Cargo workspace，可執行測試與一個 server binary（採既有 HTTP stack）。
- Browser: Rust `wasm32-unknown-unknown` target + 打包工具；輸出版本化 Wasm/JS/assets。
- Loader: 盡可能小的 external JavaScript；MVP **不要求** loader 本身以 Rust/Wasm 實作。
- Public docs: README build/run、route screenshot/smoke instructions、已測試的環境矩陣。
- Default dev mode: server 開發與 browser assets build 可以分開執行，但最終 smoke test 要驗證同一份可重現 artifact set。

## 6. Explicitly out of scope

LiveView / SignalR 類遠端 UI event replication、WebGPU DOM acceleration、完整 CSS/layout 引擎、full resumability、native desktop/mobile、多 backend ORM、自製 HTTP parser、自動 magical RPC、完整生態系 plugin API、server 上 request-time 編譯 Wasm。

## 7. Milestone proof

依 [Roadmap](ROADMAP.md) 的 M1 → M2 → M3 → M4 建立證據。若 dual renderer 共享模型有基本缺陷，先回到 [ADR 0003](adr/0003-renderers-and-loading.md) 重審元件 IR 假設，而不是繼續擴大 crate 清單。
