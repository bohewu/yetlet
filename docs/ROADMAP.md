# Yetlet Roadmap — evidence-gated

**Status:** Draft (2026-10-08)
**計畫方式：** 依能力與依賴關係排序，**不虛構交付日期**。以 [MVP spec](MVP.md) 的 acceptance matrix 為完成判準。每個里程碑須保留測試命令、結果與必要的架構決策紀錄。

| Milestone | 目標 | 對應 Backlog | Gate / 交付證據 |
|---|---|---|---|
| **M0 — Design baseline** | 規格、架構、ADR、backlog、repo instructions | YT-001 | 文件互相鏈結，task ID / 依賴清楚；決策不冒稱已驗證 |
| **M1 — Native SSR foundation** | Cargo workspace、共享元件雛形、HTML renderer、HTTP adapter | YT-002–YT-007 | `GET /` 能回 escaped HTML；關閉 JS 可用；native tests pass |
| **M2 — Browser DOM/WASM** | browser build、DOM mount、local signals、共用元件 | YT-008–YT-010 | 同一元件 source 在 server/browser 可編譯，local click 更新 DOM |
| **M3 — Selective islands** | external loader、asset manifest、island activation、初始 state | YT-011–YT-013、YT-017 | `/search` 初始 SSR 完整；browser 僅載 island；無 raw event forwarding |
| **M4 — API/action integration** | typed actions、HTTP executor、JSON API、strict CSP | YT-014–YT-016、YT-018 | local/remote 界線清晰；API/error & CSP E2E pass |
| **M5 — MVP validation / docs** | 跨 target tests、browser smoke、perf baseline、examples | YT-019–YT-020 | [A01–A10](MVP.md#3-acceptance-matrix) 的證據齊備；公開限制 |

## Critical path

```text
YT-001
  └─ YT-002 (workspace/toolchains)
       ├─ YT-003 (shared view)
       │    ├─ YT-004 (HTML renderer) → YT-005 (server adapter) → YT-006 (SSR demo)
       │    └─ YT-008 (browser build) → YT-009 (DOM renderer) → YT-010 (shared component)
       └─ YT-007 (native build/test CI)
YT-010 → YT-011 (loader) → YT-012 (islands) → YT-013 (search demo)
YT-003 → YT-014 (actions) → YT-015 (HTTP executor) → YT-016 (API integration)
YT-012 → YT-017 (state manifest)
YT-013 + YT-016 + YT-017 → YT-018 (CSP test)
YT-006 + YT-010 + YT-013 + YT-016 + YT-018 → YT-019 (E2E/perf)
YT-019 → YT-020 (release documentation)
```

先滿足 native SSR，才為 browser renderer 選 UI IR。避免早期同時對 DOM、SSR、islands、GPU 作過多抽象設計。

## Gate checklist

### M0 — files only

- [x] `README.md` / `AGENTS.md` / `docs/*` exists，所有相對連結有效。
- [x] Backlog ID 唯一、每個 work item 具有驗收標準和依賴。
- [x] ADR 區分 proposal 與已驗證選項。
- [x] `python3 scripts/check_docs.py` 通過（含未追蹤文件的 whitespace/link check）。

**M0 evidence (2026-10-08):** `python3 scripts/check_docs.py` → PASS（9 documents、25 backlog items、dependency DAG）；`git diff --check` → PASS（僅涵蓋 tracked changes）。

### M1 — server proof

- [x] `cargo test --workspace --all-targets --locked` 通過（11 tests）。
- [x] `cargo build -p yetlet-todo-server --locked` 的 native binary 已實際啟動並完成 HTTP smoke。
- [x] `GET /` 包含 escaped HTML、沒有 browser JS/WASM 引用；strict CSP 正常。
- [x] `yetlet-server-axum` 使用 Axum/Tower HTTP stack，`/api/ping` 為獨立 JSON API。

**M1 evidence (2026-10-08):** Rust 1.99 native tests 11/11；`cargo fmt --all -- --check`、`cargo clippy --workspace --all-targets --locked -- -D warnings`、`cargo check --locked -p yetlet-core -p yetlet-todo-ui --target wasm32-unknown-unknown` 均 PASS。一次 bounded localhost smoke：`GET /` 200 HTML、`/healthz` 200，CSP 無 unsafe-inline，SSR 無 script/WASM（當時 423 bytes HTML，非性能基準）。CI workflow 已新增，但尚未在 GitHub 執行；Browser renderer 仍未實作。

### M2 — browser proof

- [x] Browser WASM 實際 mount 並更新 DOM（非只有 cargo check），headless Chrome E2E PASS。
- [x] 同一個 `todo_list` Rust source 在 native SSR 與 browser DOM 渲染出相同初始 HTML。
- [x] 兩次本地 click 狀態切換，沒有額外 HTTP request 或 WebSocket。

**M2 evidence (2026-10-08; 16/16 Rust tests + Clippy PASS):** `python3 scripts/build_browser.py` 生成帶 hash 的 `dist/manifest.json`/JS/Wasm；`cargo check -p yetlet-todo-browser --target wasm32-unknown-unknown --locked` PASS；`python3 scripts/smoke_browser.py` 於 Chrome 155 PASS（SSR/DOM byte-equal、兩次 local click 更新與無 network event bridge）。詳見 [M2 verification](M2-VERIFICATION.md)。GitHub CI workflow 已更新，但未遠端執行。

### M3–M4 — island + actions proof

- [ ] 首屏 SSR 可讀；island 有一致 state schema。
- [ ] 外部 loader / hashed assets 正確載入。
- [ ] Browser local events 和 HTTP command transport 可觀測並區分。
- [ ] API validation + CSP E2E passing。

### M5 — evidence freeze

- [ ] MVP A01–A09 的測試結果可重現。
- [ ] Perf baseline 紀錄環境、方法、數值及限制。
- [ ] docs 與真實實作一致；未完成功能不能寫成已支援。

## Post-MVP research（不阻塞核心）

| Track | Question / experiment |
|---|---|
| Streaming SSR | SSR API 是否需要 first-byte/streamed suspense？ |
| Compile-time template IR | 實際需要 macro static slots 還是簡單 reactive DOM bindings 已足夠？ |
| Resumability | 能否明確限制 serializable handlers + stable IDs？ |
| Custom transport | 可插拔 action executor 的最低必要 interface 是什麼？ |
| WebGPU renderer | 是否能在 10k–100k scene nodes 獲得顯著且可量測的收益？ |
| Worker compute | 哪些 diff/計算可 off-main-thread 而不犧牲 DOM correctness？ |
| Tooling | 何時值得做 HMR、asset bundler integration、devtools？ |

## Decision rule

每個新增功能必須回答：它屬於 kernel、renderer、transport、server adapter，還是 optional tooling？不能回答、或造成同一責任重複時，先新增 ADR/spike，而不是直接寫入 core。
