# Yetlet Backlog

**Status:** Initial backlog established 2026-10-08; updated as evidence becomes available.

**Source of work:** This document records MVP tasks and their dependencies. [Roadmap](ROADMAP.md) tracks milestone gates; [MVP](MVP.md) defines acceptance criteria. `Done` means verified evidence exists, not merely that code was written.

Priority: `P0` is the critical path, `P1` is required for the MVP, and `P2` covers later research. Sizes `S/M/L` indicate relative complexity, not promised delivery time. Update status with test results, commands, or a relevant change reference.

| ID | Pri | Size | Milestone | Work item | Depends on | Acceptance / evidence | State |
|---|---|---|---|---|---|---|---|
| YT-001 | P0 | S | M0 | Repository docs / ADR / roadmap / backlog | — | `python3 scripts/check_docs.py` PASS (2026-10-08) | Done |
| YT-002 | P0 | S | M1 | Cargo workspace + native/WASM targets | YT-001 | Native workspace and wasm32 checks PASS (2026-10-08) | Done |
| YT-003 | P0 | M | M1 | Minimal shared `Component/View` model | YT-002 | `yetlet-core` and `yetlet-todo-ui` native/wasm32 compile PASS | Done |
| YT-004 | P0 | M | M1 | HTML renderer with safe escaping | YT-003 | Four `yetlet-html` escaping/security tests PASS | Done |
| YT-005 | P0 | M | M1 | Axum/Tower server adapter | YT-004 | Native `/` and JSON `/api/ping` routing tests PASS | Done |
| YT-006 | P0 | S | M1 | Static SSR demo at `GET /` | YT-005 | Bounded localhost HTTP smoke: 200, strict CSP, no scripts/WASM | Done |
| YT-007 | P0 | S | M1 | Native checks and CI baseline | YT-002 | Local fmt/clippy/tests/WASM PASS; CI YAML created but not executed remotely | Done |
| YT-008 | P0 | M | M2 | Browser WASM build pipeline | YT-003 | `build_browser.py`, hashed manifest, Chrome asset load PASS | Done |
| YT-009 | P0 | L | M2 | DOM mount and local state updates | YT-008 | Chrome two-click state/class/text update, no HTTP/WS PASS | Done |
| YT-010 | P0 | M | M2 | Shared TodoList across targets | YT-006, YT-009 | Same Rust `todo_list` source, SSR/browser outerHTML byte-identical PASS | Done |
| YT-011 | P1 | S | M3 | Tiny external JS loader and hashed assets | YT-008 | Loader and hashed assets; Chrome confirms no Search WASM before interaction | Done |
| YT-012 | P1 | M | M3 | Island mount boundary | YT-010, YT-011 | Chrome confirms initial SSR results preserved; input/heading nodes remain stable | Done |
| YT-013 | P1 | M | M3 | Search island at `GET /search` | YT-012 | SSR 200; browser case-insensitive search, empty/reset, no input-triggered HTTP/WS | Done |
| YT-014 | P1 | M | M4 | Typed actions and LocalExecutor | YT-003 | `yetlet-actions` unit tests + Browser local toggle, zero network requests PASS | Done |
| YT-015 | P1 | M | M4 | HttpExecutor / explicit remote boundary | YT-014, YT-005 | Chrome one typed Create POST; separate successful/error executor paths; no remote DOM event PASS | Done |
| YT-016 | P1 | M | M4 | `/api/todos` JSON handlers | YT-005, YT-015 | GET/POST tests: state/revisions, 400/403/404/413/415/422 typed errors PASS | Done |
| YT-017 | P1 | M | M3 | Island state schema/version/security | YT-012 | Bounded v1 JSON, HTML escaping, invalid state tests and Chrome fail-closed SSR fallback PASS | Done |
| YT-018 | P1 | M | M4 | Strict CSP and browser security verification | YT-013, YT-016, YT-017 | Headless Chrome/CSP, JSON errors, origin/media/body limits PASS at anonymous prototype scope | Done |
| YT-019 | P1 | L | M5 | Cross-target/E2E tests and performance baseline | YT-006, YT-010, YT-013, YT-016, YT-018 | MVP A01–A10 verification report | Todo |
| YT-020 | P1 | S | M5 | Build/run docs and release readiness | YT-019 | Accurate README, limits, compatibility; maintainer selects license before publishing | Todo |
| YT-021 | P2 | M | Later | Streaming SSR benchmark | YT-006 | Measured TTFB/throughput; ADR if selected | Deferred |
| YT-022 | P2 | L | Later | Serializable handler/resumability spike | YT-017 | Demonstrate stable handler identity and safe capture serialization | Deferred |
| YT-023 | P2 | L | Later | Independent WebGPU scene renderer spike | YT-009 | Reproducible frame/memory results, not a DOM/CSS rewrite | Deferred |
| YT-024 | P2 | M | Later | Custom action executor transport | YT-015 | Opt-in interface without UI/runtime/server coupling | Deferred |
| YT-025 | P2 | M | Later | Static-template IR versus VDOM benchmark | YT-009 | Compare update cost/complexity before committing to macros | Deferred |

## User-approved HTTP core extension

The middleware/RequestContext handoff adds these scoped tasks after M4 without renumbering existing tasks or replacing M5. Its acceptance checks are explicit user requirements, not an unrelated framework expansion.

| ID | Pri | Size | Milestone | Work item | Depends on | Acceptance / evidence | State |
|---|---|---|---|---|---|---|---|
| YT-026 | P0 | M | M4.1 | RequestContext ownership and typed request/app state | YT-005 | Context/state/metadata/query/path/streaming tests PASS; browser graphs unchanged | Done |
| YT-027 | P0 | M | M4.1 | Native Tower global/group/route composition | YT-026 | Nested order, short-circuit, 404/405/Allow, error and third-party layer tests PASS | Done |
| YT-028 | P0 | M | M4.1 | API/SSR integration and request isolation/streaming proof | YT-027, YT-018 | 46/46 workspace tests, Chrome regression, 12 concurrent requests and byte-identical WASM PASS | Done |
| YT-029 | P1 | M | Later | Optional HTTP providers, deadlines and overhead measurements | YT-028 | Opt-in middleware providers and measured native costs; not P0 scope | Deferred |

## Ready queue

**First:** Resume `YT-019` full acceptance/performance baseline. The user-approved HTTP P0 extension (`YT-026`–`YT-028`) is complete at local prototype scope; M0–M4 items retain their recorded evidence. P1 `YT-029` remains optional follow-up.

**Next:** `YT-020` release documentation/compatibility/limitations, after measured M5 evidence. Production authentication, robust CSRF, and real multi-browser testing remain explicit future requirements, not properties of this anonymous demo.

**Do not start before prerequisites:** Generalized hydration, arbitrary event forwarding, WebGPU, and resumability remain out of scope.

## Work-item template

```markdown
### YT-xxx — title
- Outcome:
- In scope:
- Out of scope:
- Dependencies:
- Acceptance commands / assertions:
- Risks / affected ADRs:
- Evidence (filled only after verification):
- State: Todo | In progress | Blocked | Done
```

## Update rules

1. Change status only after checking source changes and task-specific evidence.
2. Mark an item `Blocked` if a required build/toolchain/browser gate cannot run, and name the exact limitation; do not infer a passing result.
3. Record discovered must-have work under a new unique ID rather than silently expanding an existing task.
4. If MVP scope or the dependency graph changes, update this document and the Roadmap together.
