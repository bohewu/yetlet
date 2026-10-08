# ADR 0001 — Separate execution, rendering, interactivity and transport

- **Status:** Proposed for MVP
- **Date:** 2026-10-08
- **Owners:** project maintainers
- **Related:** [Architecture](../ARCHITECTURE.md), [MVP](../MVP.md)

## Context

傳統 full-stack 框架常將元件 renderer、瀏覽器 hydration、client/server 通訊和 HTTP server 特定選擇綁在一起。Yetlet 需要讓同一套 Rust component model 支援 native SSR 與 browser UI，但使用者不應被迫負擔不需要的 runtime 或 persistent event circuit。

## Proposal

把四個維度分開：execution target（native/wasm）、renderer（HTML/DOM/scene）、interactivity（none/island/full）、action executor（local/HTTP/custom）。真正支援的組合由 target capability 驗證，不能只靠 run-time enum 接受不可能的配置。

HTTP API 經既有 router/middleware；SSR render 直接使用 native domain capability。預設沒有 persistent session/circuit，不建立 framework 專用 event WebSocket。

For MVP, only support three well-tested presets:

1. Native static SSR without a browser runtime.
2. Native SSR with one optional browser island.
3. Browser full UI runtime backed by explicit HTTP data API.

## Expected benefits

- 靜態頁沒有 client-side application cost。
- 以相同 component semantics 組合 SSR/CSR，而不是重複撰寫畫面。
- 事件傳輸可更換而不修改 UI renderer。
- renderer 可延伸至 Canvas/WebGPU，但不污染一般 HTML/DOM。

## Alternatives considered

- **Blazor Server/LiveView-like remote event circuit:** 開發模型直接，但所有互動的網路依賴不符合 local-event-first 原則。
- **All CSR:** 架構簡單，但首屏 HTML 和 minimal runtime 目標難達成。
- **Only static SSR:** 體積小，但 browser stateful app 無法達到共用程式碼目的。
- **One universal runtime binary:** native/browser capabilities/target 區別大，容易把 server-only deps 拉進 browser。

## Risks and tests

模式矩陣可能膨脹。MVP 僅驗證三種 preset；對每種 preset 都提供 smoke test。沒有證據的特徵不宣布支援。當四維分離反而導致大量 adapter boilerplate，再回頭簡化 API。

## Consequences

使用者明確決定互動與 remote boundary；框架內部需維持小而清晰的 trait/module 邊界。此提案不是已確定的 public API contract。
