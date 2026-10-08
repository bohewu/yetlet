# ADR 0003 — Dual-target renderers and opt-in browser loading

- **Status:** M2 dual-renderer baseline validated; Islands and template IR remain proposed
- **Date:** 2026-10-08
- **Related:** [Architecture](../ARCHITECTURE.md), [MVP](../MVP.md)

## Context

Yetlet 想用 Rust 維持 SSR/CSR 共用 UI 邏輯，同時讓 SSR 的 browser payload 最低。但 server native HTML serializer、browser DOM API 和 WebGPU scene renderer 是不同能力；把它們塞進單一底層 rendering path 容易引入過度抽象。

## Proposal

- Shared **component semantics/source**；build outputs 為 native binary、browser Wasm、外部 bootstrap/manifest 等獨立 artifacts。
- Server：native HTML renderer，對沒有互動的頁面輸出 HTML，不載任何 Wasm/JS。
- SSR + Islands：保留已 render 的 HTML；小型 external JS loader 從 build-time manifest 定位需要的 browser module，再進行 island mount/hydration。MVP 只需做到 per-island ownership，無需 full resumability。
- Browser：較完整的 Rust/Wasm DOM runtime，直接建立與更新 DOM。
- Browser loader 優先最小有效實作，允許純 JS；**不把「每一 byte 都是 Rust/Wasm」當作指標**。
- WASM/code 均在 build/deploy time 產生並 hash；request path 不執行編譯器。
- GPU：未來可加獨立 scene renderer，但不承諾可替 DOM/CSS 直接使用 WebGPU 加速。

## Key unresolved design: representation

### Option A — Minimal shared View tree

以少量 nodes/value types 驗證 native HTML 與 browser DOM。起步快，但可能有 tree allocation/update 成本。

### Option B — Compile-time static template + dynamic slots

`ui!` macro 提取固定 HTML/template，native serialize static/dynamic slots，browser clone template + update bound slots，減少 VDOM diff 與額外 runtime。但宏、事件 capture、hydration identity 的設計成本較高。

**Decision gate:** M1/M2 先用最小能測的 representation，完成同元件跨 target demo，再測量必要的 macro/slot optimizations。不要在還沒有 baseline 的時候宣稱 no-VDOM 有性能優勢。

## Hydration and state

- SSR HTML and browser code 需要可驗證的 stable component/instance ID，且 hydration 不可盲目覆寫不屬於 island 的 DOM。
- serialized state 用安全 escaping，包含 schema / asset version；不相容時顯式 fail/fallback。
- browser hydration state 不能攜帶 secret / repository / server handle。
- 避免在 script tag 內直接插入未處理 JSON；方案需通過 CSP 與 XSS 測試。

## Alternatives considered

- Full VDOM everywhere：設計統一但易增加 SSR/browser runtime cost，仍需測量。
- Server-only UI streaming updates：與 local-first/events-optional 目標衝突。
- Full resumability first：Rust closure capture/serialization 需要對編譯器/handler 設限，屬後續研究。
- Canvas for all elements：會涉及 CSS layout、text selection、IME、accessibility 等 browser 重造，不屬初期範圍。

## Verification

**Observed at M2 (2026-10-08):** Shared View source has been rendered through both native HTML serialization and a browser WASM direct-DOM renderer; a headless Chrome test confirmed byte-equal initial TodoList markup, local event updates, and no WebSocket. The Browser shell uses a tiny external JavaScript bootstrap and content-hashed assets. The implementation uses subtree replacement, not fine-grained subscriptions or resumability. SSR Islands, state-transfer/compatibility and optional transport have NOT been implemented; see [M2 verification](../M2-VERIFICATION.md).


- `/` zero runtime、可完全禁 JS。
- `/search` 初始 HTML 完整，僅載入 island assets。
- `/app` mount/update 成功；同一 Component 可用於 server/browser。
- 記錄 Wasm compressed size、SSR response bytes、browser mount time，不以未量測數據作比較。
