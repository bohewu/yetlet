# Yetlet Architecture — v0 design baseline

**Status:** Proposal / pre-MVP (2026-10-08)
**Scope:** Rust native server, HTML SSR, browser DOM/WASM, HTTP data API, optional islands.
**Related:** [MVP](MVP.md) · [Roadmap](ROADMAP.md) · [Backlog](BACKLOG.md) · [ADRs](adr/0001-separation-of-concerns.md)

## 1. Problem statement

Yetlet 要降低 web framework 的概念與 runtime 成本：不要求開發者為一般 SSR 頁面負擔整個 client framework，也不把每個 browser input/click 透過持久連線轉送後端。Rust component code 應能共用於 native server 與 browser target，但 renderer、互動策略及動作執行位置可獨立選擇。

這不等於重新實作瀏覽器、HTTP stack、WebSocket protocol 或 CSS layout engine。

### 不變原則

- SSR 初始 HTML 必須可獨立呈現；沒有互動需求時 **0 browser JS / WASM**。
- 有互動需求時，以可明確識別的 island 或 browser app 載入 client code。
- DOM event 的 ownership 在 browser；remote command 並非 event replication。
- server 和 browser 可能共用 Rust 型別與 component code，但一定是不同 build artifact。
- 在 request path 僅做 rendering、state serialization 與 build-time asset lookup；不即時編譯 Rust、Wasm 或 JS。
- HTTP routing 與 API middleware 接現有生態系的 server adapter，不強制自造 webserver。

## 2. 分離四個維度

| 維度 | 候選值 | 是否受其他維度綁定 |
|---|---|---|
| Execution | native server / browser WASM | 否 |
| Render backend | HTML stream / browser DOM / future GPU scene | 否（但須檢查 target capability） |
| Interactivity | none / island / full browser / future resume | 否 |
| Remote action | none / plain form / typed HTTP / future custom executor | 否 |

**重要：** `render(server)` 並不代表 `events(server)`。一條 SSR route 可使用 native form POST；一個 CSR component 可以完全離線工作。

## 3. Logical architecture

```text
                       ┌────────────────────────────┐
                       │ Rust component declarations│
                       │  + signals + typed actions │
                       └──────────────┬─────────────┘
                                      │ compiler / macros
                           Component description / IR
                             (shape under evaluation)
                                      │
                    ┌─────────────────┴──────────────────┐
                    │                                    │
             Native server target                  Browser WASM target
                    │                                    │
              HTML renderer                         DOM renderer
                    │                                    │
            markup + asset refs                 client mount / updates
                    │                                    │
                    └──────────┬─────────────────────────┘
                               │
                     Local semantic action
                               │
                      Action executor port
                        │               │
                   Local/state       HTTP/fetch
                                        │
                             Axum/Tower adapter
                               ├─ UI routes
                               └─ API routes
```

Renderer 永遠不應偷偷建立 remote session/circuit。Transport 是可選 adapter；API request 僅由正規 middleware/pipeline 處理。

## 4. Request paths

### 4.1 API route

```text
HTTP request → router → auth/middleware → typed API handler
                                             → JSON/status/body
```

API handler 與 UI SSR 不必共享 HTTP path。跨網路驗證、授權、限流在 server boundary 執行。

### 4.2 Static SSR route

```text
GET / → router → loader (native capability) → component
    → HTML renderer (context-aware escaping) → response
```

沒有互動 component 時，回傳 HTML+CSS 即可。後端可以直接呼叫 application service，**不經 localhost HTTP loopback**。

### 4.3 SSR + island route

```text
GET /search → native SSR → HTML + data-state + hashed asset references
                                 │
                         tiny external JS loader
                                 │ on visible / interaction
                         lazy load island WASM
                                 │
                         browser-local DOM events
```

MVP 採「island mount/hydration」而非 full resumability；只由該 island 擁有的 DOM 子樹參與 client updates。Loader 是 bootstrap，不必強制使用 Wasm。State 應安全序列化、含 version/component identity，且不能含 credentials。

### 4.4 Full browser route

```text
GET /app → shell HTML + compiled client artifacts
    → loader → browser WASM → DOM mount → API when needed
```

共用 component source 不代表每個 server-side domain dependency 都會進入 browser bundle；browser 透過 capability interface 綁定 HTTP implementation。

## 5. Proposed module boundaries

| Candidate module | 單一職責 | MVP 時機 |
|---|---|---|
| `yetlet-core` | Component/View、state/signal、renderer-neutral types | M1 |
| `yetlet-html` | deterministic escaping + HTML renderer | M1 |
| `yetlet-server-axum` | UI/API routing、HTTP adapter、response headers | M1 |
| `yetlet-dom` | browser DOM mount / dynamic updates | M2 |
| `yetlet-browser` | Wasm bootstrapping + browser capabilities | M2 |
| `yetlet-islands` | island identity/state/loader manifest | M3 |
| `yetlet-actions` | typed command + local/HTTP executor interfaces | M4 |
| `yetlet-macros` | compile-time `ui!` DSL & static template extraction | M2+ spike |
| `yetlet-gpu` | independent retained scene renderer | 後續研究 |

These are **candidate crates**, not committed public package boundaries. 優先確認兩個 render targets 的最小可行介面，再決定是否拆 crate，避免在 MVP 前為假想 extensibility 增加 complexity。

## 6. Components, templates and state

### Shared source

```rust
// 介面示意，尚非 Yetlet API：
#[component]
fn TodoList(items: &[Todo]) -> View {
    ui! { <ul>{ for item in items { <li>{item.title}</li> } }</ul> }
}
```

Native target 輸出 HTML；browser target 建立及更新 DOM。元件要有 deterministic boundaries 和 stable IDs，否則 hydration 容易 mismatch。

**Design hypothesis：** macro 在編譯時擷取 static template 與 dynamic slots。這可能避開 full VDOM diff；但初期允許簡化 `View`/node representation，以建立可驗證的基準。是否必須做 template IR 是 spike decision，而非已落實的設計。

### Data access

```rust
// 概念介面；瀏覽器 target 不能依賴 server-only repository。
trait TodoSource {
    async fn list(&self) -> Result<Vec<Todo>, TodoError>;
}
```

Native implementation 對 application/domain service 直呼；browser implementation 對 HTTP JSON API 呼叫。這不應變成隱藏於 compiler 的 arbitrary RPC。

### State transfer

Native HTML 輸出時可以送出可序列化的初始 state，但要：

- 控制 component/instance identity、schema version、asset manifest version。
- 使用上下文正確的 HTML/attribute/JSON escaping；避免 `</script>`、`&` 或 Unicode separator 注入問題。
- 對不可信 state 做大小限制，及在 browser hydration 時執行 version validation。
- 不將 server context、token、密碼、資料庫連線或具特權的 closure capture 序列化。

## 7. Events and commands

```text
pointer/keyboard/input DOM event
          │ local browser listener
          ▼
  component-local state transition
          │ when remote work needed
          ▼
 typed command / explicit HTTP form
          │
          ├─ LocalExecutor
          ├─ HttpExecutor
          └─ Future CustomExecutor (explicit opt-in)
```

MVP 不提供 raw DOM event forwarding。遠端 action 需要明確的 input/output/error schema，server 必須重新授權驗證。HTTP mutation 需考慮 CSRF、idempotency、replay 和 retry policy。較晚才決定 custom transport API。

## 8. Security, accessibility and operations

- 以 strict CSP 為基準；無 inline `on*`、無 `eval`；loader 使用 external static assets。
- SSR output 使用正確 context escaping；hydration metadata 視為 hostile input。
- DOM renderer 優先原生 semantic HTML、label/input、focus、keyboard navigation、IME、native scroll。
- browser bundle 不可 import server-only code/secrets；feature boundary 應在 compile time fail。
- 每種模式都需可辨識 metrics（SSR render time、HTML bytes、Wasm compressed bytes、mount latency、requests count）。
- 錯誤語意應清楚：server render fail 回 status；client action fail 回 typed error；browser mount error 可回退到 HTML/form。
- 對 browser bundle 與 HTML 注入作 CSP、XSS 及 dependency audit，避免為性能犧牲安全。

## 9. GPU / parallelization boundary (post-MVP)

WebGPU 不會直接加速一般 DOM/CSS layout。若要支援 WebGPU，應新增 **Canvas/scene renderer**；其 scene graph、focus/IME/ARIA/accessibility 職責與 HTML renderer 不同。DOM 情境中的 Worker 可做 background compute，但 DOM commit 仍由 main thread 執行。GPU 將以可重現的節點量、frame time、memory 指標作為獨立 spike，而非 MVP 阻塞條件。

## 10. Architecture risks and decisions awaiting experiment

| Risk | First test / mitigation |
|---|---|
| Rust UI macro 限制/編譯複雜度 | 先做 shared view + dual renderer spike，再決定 IR / `ui!` |
| SSR/client state mismatch | deterministic identity + SSR fixture / hydrate snapshot assertions |
| WASM bundle 體積或啟動過慢 | 記錄 gzip/brotli bytes & mount timing，loader 分拆 islands |
| DOM update 過度抽象 | 先驗證 direct node updates，再比較 VDOM / static slots |
| server-only code 意外編入瀏覽器 | target-specific features + browser CI compile |
| 遠端 command 越界成 circuit | typed actions only，無預設 WebSocket 連線 |
| 模式組合爆炸 | MVP 固定三種 route，traits 範圍最小化 |
| GPU renderer 與 HTML semantics 不一致 | GPU 完全排除 MVP，另建 scene interface |

參閱 [ADR 0001](adr/0001-separation-of-concerns.md)、[ADR 0002](adr/0002-events-and-actions.md)、[ADR 0003](adr/0003-renderers-and-loading.md)。
