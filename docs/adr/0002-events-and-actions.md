# ADR 0002 — Local DOM events, typed actions across boundaries

- **Status:** Proposed for MVP
- **Date:** 2026-10-08
- **Related:** [Architecture](../ARCHITECTURE.md), [Backlog](../BACKLOG.md)

## Context

將 DOM click/input/keyboard 等 raw event 序列化送到 server，使每次互動依賴網路、產生 circuit state、跨序列化界線和 reconnect protocol。Yetlet 不以這類 remote-event 模型作為 default。

## Proposal

1. DOM event listener 屬於 browser component/local runtime。
2. UI state transition 優先在 browser local executor 完成。
3. 若需 server 資料或 mutation，component 產生**語意明確、可驗證、可序列化的 typed command/action**。
4. ActionExecutor 決定該 command 在 local 或 HTTP endpoint 執行；custom transport 為未來 opt-in。
5. SSR 不載入 runtime 時，普通 `<form method="post">` 仍可以使用；progressive enhancement 不應破壞 HTML semantics。

```rust
// 概念草案；traits 與 error type 仍待驗證：
trait ActionExecutor<C: Command> {
    async fn execute(&self, command: C) -> Result<C::Output, C::Error>;
}
```

實作順序先是 `LocalExecutor` 與 `HttpExecutor`。一開始**不提供**「序列化 Browser Event 並在 server 執行 listener」的自動方案。

## Security requirements

- 每個 remote action 在 server **重新**驗證認證、授權與輸入；client state 非可信。
- Cookie-based mutation 必須有明確 CSRF 防護 / SameSite policy；不是「typed」就自動安全。
- 針對可重試 action 設計 idempotency；不得盲目重送非冪等請求。
- 避免在 serialization、logging、HTML hydration state 中洩露 private fields。
- 明確定義 timeout、network error、validation error、user-facing error 的處理。

## Tradeoffs

此設計將不同於透過 compiler 自動實作所有 server functions 的魔法 RPC，開發者需明確管理 service/action 邊界。但代價是更好的可觀測性、HTTP semantics、testing 與 security review。

## Verification

- browser local click 無 remote request。
- 遠端 mutation 僅送出 semantic input，不送 `MouseEvent` 等 raw event。
- API 4xx、timeout、server 5xx 的 client state 不可靜默視為成功。
- 無 framework 專用 persistent socket，custom executor 啟用後才允許 custom transport。
