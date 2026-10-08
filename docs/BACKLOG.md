# Yetlet Backlog

**Status:** initial backlog; created 2026-10-08.
**唯一工作來源：** 此檔記錄 MVP 實作與依賴；[ROADMAP](ROADMAP.md) 管理 milestone gates，[MVP](MVP.md) 管理功能驗收。`Done` 需要驗證證據，不代表僅寫了程式碼。

Priority：`P0` critical path；`P1` MVP must-have；`P2` later / research。Size 為相對複雜度 `S/M/L`，非交付時間。初始狀態 `Todo`；完成後須同步連結實測命令或 PR/commit。

| ID | Pri | Size | Milestone | Work item | Depends on | Acceptance / evidence | State |
|---|---|---|---|---|---|---|---|
| YT-001 | P0 | S | M0 | Repo docs / ADR / roadmap / backlog | — | `python3 scripts/check_docs.py` → PASS（2026-10-08） | Done |
| YT-002 | P0 | S | M1 | Cargo workspace + native/wasm targets | YT-001 | native workspace + wasm32 check PASS (2026-10-08) | Done |
| YT-003 | P0 | M | M1 | Tiny shared `Component/View` model | YT-002 | `yetlet-core` + `yetlet-todo-ui` native/wasm32 compile PASS | Done |
| YT-004 | P0 | M | M1 | HTML renderer + safe escaping | YT-003 | `yetlet-html` 4 security/escaping tests PASS | Done |
| YT-005 | P0 | M | M1 | Axum/Tower server adapter | YT-004 | native `/` + JSON `/api/ping` routing tests PASS | Done |
| YT-006 | P0 | S | M1 | Demo `GET /` static SSR | YT-005 | bounded localhost HTTP smoke 200, strict CSP, no script/WASM | Done |
| YT-007 | P0 | S | M1 | Native checks + CI baseline | YT-002 | fmt/clippy/tests/wasm check PASS locally; CI YAML present (not run remotely) | Done |
| YT-008 | P0 | M | M2 | Browser WASM build pipeline | YT-003 | `build_browser.py` + hashed manifest + Chrome asset load PASS | Done |
| YT-009 | P0 | L | M2 | DOM mount + local state updates | YT-008 | Chrome two-click state/class/text update; no HTTP/WS PASS | Done |
| YT-010 | P0 | M | M2 | Shared TodoList across targets | YT-006, YT-009 | Same `todo_list` Rust source, SSR/Browser outerHTML byte-equal PASS | Done |
| YT-011 | P1 | S | M3 | Tiny external JS loader + hashed assets | YT-008 | no eval or inline JS; lazy module loading verified | Todo |
| YT-012 | P1 | M | M3 | Island mount boundary | YT-010, YT-011 | SSR DOM preserved; only island updated by browser runtime | Todo |
| YT-013 | P1 | M | M3 | Demo `GET /search` island | YT-012 | SSR content and browser typing smoke evidence | Todo |
| YT-014 | P1 | M | M4 | Typed actions + LocalExecutor | YT-003 | typed input/output/errors, local action test | Todo |
| YT-015 | P1 | M | M4 | HttpExecutor / explicit remote boundary | YT-014, YT-005 | typed HTTP request; no DOM-event forwarding | Todo |
| YT-016 | P1 | M | M4 | `/api/todos` JSON handlers | YT-005, YT-015 | GET/POST happy/error tests; status/error schema | Todo |
| YT-017 | P1 | M | M3 | Island state schema/version/security | YT-012 | safe escaping; invalid version fails closed or fallback | Todo |
| YT-018 | P1 | M | M4 | Strict CSP / browser security verification | YT-013, YT-016, YT-017 | no unsafe-inline; browser E2E + server headers proof | Todo |
| YT-019 | P1 | L | M5 | Cross-target + E2E tests + perf baseline | YT-006, YT-010, YT-013, YT-016, YT-018 | MVP A01–A10 verification report | Todo |
| YT-020 | P1 | S | M5 | Build/run docs + release readiness | YT-019 | README, limitations, compatibility; maintainer selects license before publication | Todo |
| YT-021 | P2 | M | Later | Streaming SSR benchmark | YT-006 | measurable TTFB/throughput experiment; ADR if chosen | Deferred |
| YT-022 | P2 | L | Later | Serializable handler/resumability spike | YT-017 | demonstrate handler identity & safe capture serialization | Deferred |
| YT-023 | P2 | L | Later | Independent WebGPU scene renderer spike | YT-009 | reproducible frame/memory benchmark, not DOM/CSS rewrite | Deferred |
| YT-024 | P2 | M | Later | Custom action executor transport | YT-015 | opt-in interface without runtime/server coupling | Deferred |
| YT-025 | P2 | M | Later | Static template IR vs VDOM benchmark | YT-009 | compare update cost and complexity before macro commitment | Deferred |

## Ready queue

**Do first:** `YT-011` 建立 SSR Islands 專用的 lazy loader；`YT-001`–`YT-010` 已完成 M0–M2 本機驗證。
**Next:** `YT-012` island boundary → `YT-013` search island；`YT-014` typed LocalExecutor 亦可獨立啟動。
**Do not start before capability exists:** island hydration、typed remote bridge、WebGPU、resumability。

## Work-item template

When opening a task, use:

```markdown
### YT-xxx — title
- Outcome:
- In scope:
- Out of scope:
- Dependencies:
- Acceptance commands / assertions:
- Risks / changed ADRs:
- Evidence (filled after verification):
- State: Todo | In progress | Blocked | Done
```

## Update rules

1. Update state only after observing a changed source snapshot and task-specific evidence.
2. Mark `Blocked` if toolchain/build/browser gate cannot run; include exact missing capability, do not substitute a speculative pass.
3. Record newly discovered must-have work with a new unique ID rather than silently expanding an existing ticket.
4. If the MVP spec or dependency graph changes, update both this file and ROADMAP.
