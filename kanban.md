# Yetlet Kanban

Derived from [`todo-ledger.json`](todo-ledger.json). The project backlog source is [`docs/BACKLOG.md`](docs/BACKLOG.md).

**Items:** 81 — 68 Done, 1 Blocked, 12 Todo.

M11 is filed for planning; Todo rows require separate implementation authority. Product readiness and optional wgpu rendering are independent. Historical completion scopes and YT-020 are unchanged.

## Blocked

| ID | Priority | Work item | Depends on | Evidence |
|---|---|---|---|---|
| `YT-020` | medium | Build/run docs and release readiness | YT-019 | [Private preparation](docs/YT-020-PRIVATE-RELEASE-PREP.md)<br>[hosted rebuilding](docs/YT-020-HOSTED-REBUILD.md)<br>[trust/registry policy](docs/YT-020-PRIVATE-TRUST-POLICY.md)<br>[actual Private signing](docs/YT-020-PRIVATE-SIGNING-VERIFICATION.md) |

## Todo — separately authorized M11 proposals

| ID | Priority | Work item | Depends on | Plan |
|---|---|---|---|---|
| `YT-070` | medium | Realistic form and existing-CSS proof | YT-069 | [Product research](docs/M11-PRODUCT-READINESS-RESEARCH.md) |
| `YT-071` | medium | Selected safe HTML/live form contracts | YT-070 | [M11 plan](docs/M11-PLAN.md) |
| `YT-072` | medium | Minimal typed stylesheet decision/proof | YT-070 | [M11 plan](docs/M11-PLAN.md) |
| `YT-073` | medium | Complete persisted Lab Operations Board | YT-071, YT-072 | [Product research](docs/M11-PRODUCT-READINESS-RESEARCH.md) |
| `YT-074` | medium | Selected product operating-envelope qualification | YT-073 | [Product research](docs/M11-PRODUCT-READINESS-RESEARCH.md) |
| `YT-075` | medium | Browser WebGPU island and SSR fallback | YT-069, YT-023 | [GPU plan](docs/M11-WGPU-PLAN.md) |
| `YT-076` | medium | GPU lifecycle/resource ownership | YT-075 | [GPU plan](docs/M11-WGPU-PLAN.md) |
| `YT-077` | medium | Matched dynamic renderer measurements | YT-076 | [GPU plan](docs/M11-WGPU-PLAN.md) |
| `YT-078` | medium | Picking and semantic accessibility alternative | YT-077 | [GPU plan](docs/M11-WGPU-PLAN.md) |
| `YT-079` | medium | Approved opt-in yetlet-wgpu candidate | YT-078 | [GPU plan](docs/M11-WGPU-PLAN.md) |
| `YT-080` | medium | Selected renderer target qualification | YT-079 | [GPU plan](docs/M11-WGPU-PLAN.md) |
| `YT-081` | medium | Selected M11 closeout and unmet gates | YT-074, YT-080 | [M11 plan](docs/M11-PLAN.md) |

## Done

| ID | Priority | Work item | Depends on | Evidence |
|---|---|---|---|---|
| `YT-001` | high | Repository docs / ADR / roadmap / backlog | — | — |
| `YT-002` | high | Cargo workspace + native/WASM targets | YT-001 | — |
| `YT-003` | high | Minimal shared `Component/View` model | YT-002 | — |
| `YT-004` | high | HTML renderer with safe escaping | YT-003 | — |
| `YT-005` | high | Axum/Tower server adapter | YT-004 | — |
| `YT-006` | high | Static SSR demo at `GET /` | YT-005 | — |
| `YT-007` | high | Native checks and CI baseline | YT-002 | — |
| `YT-008` | high | Browser WASM build pipeline | YT-003 | — |
| `YT-009` | high | DOM mount and local state updates | YT-008 | — |
| `YT-010` | high | Shared TodoList across targets | YT-006, YT-009 | — |
| `YT-011` | medium | Tiny external JS loader and hashed assets | YT-008 | — |
| `YT-012` | medium | Island mount boundary | YT-010, YT-011 | — |
| `YT-013` | medium | Search island at `GET /search` | YT-012 | — |
| `YT-014` | medium | Typed actions and LocalExecutor | YT-003 | — |
| `YT-015` | medium | HttpExecutor / explicit remote boundary | YT-014, YT-005 | — |
| `YT-016` | medium | `/api/todos` JSON handlers | YT-005, YT-015 | — |
| `YT-017` | medium | Island state schema/version/security | YT-012 | — |
| `YT-018` | medium | Strict CSP and browser security verification | YT-013, YT-016, YT-017 | — |
| `YT-019` | medium | Cross-target/E2E tests and performance baseline | YT-006, YT-010, YT-013, YT-016, YT-018 | [M5 verification](docs/M5-VERIFICATION.md) |
| `YT-021` | low | Streaming SSR benchmark | YT-006 | [Bounded incremental SSR research](docs/YT-021-STREAMING-SSR-RESEARCH.md) |
| `YT-022` | low | Serializable handler/resumability spike | YT-017 | [Bounded resumability research](docs/YT-022-RESUMABILITY-RESEARCH.md) |
| `YT-023` | low | Independent WebGPU scene renderer spike | YT-009 | [Bounded native scene research](docs/YT-023-WEBGPU-RESEARCH.md) |
| `YT-024` | low | Custom action executor transport | YT-015 | [Bounded custom transport research](docs/YT-024-CUSTOM-TRANSPORT-RESEARCH.md) |
| `YT-025` | low | Static-template IR versus VDOM benchmark | YT-009 | [M7a evidence](docs/M7A-VERIFICATION.md) |
| `YT-026` | high | RequestContext ownership and typed request/app state | YT-005 | — |
| `YT-027` | high | Native Tower global/group/route composition | YT-026 | — |
| `YT-028` | high | API/SSR integration and request isolation/streaming proof | YT-027, YT-018 | — |
| `YT-029` | medium | Optional HTTP providers, deadlines and overhead measurements | YT-028 | [Bounded deadline research](docs/YT-029-HTTP-DEADLINE-RESEARCH.md) |
| `YT-030` | high | Minimal App/group/route facade over Axum/Tower | YT-027, YT-016 | [M6a verification](docs/M6A-VERIFICATION.md) |
| `YT-031` | high | Handler/typed result ergonomics with minimal boilerplate | YT-030, YT-004 | [M6 evidence](docs/M6-VERIFICATION.md) |
| `YT-032` | high | Explicit opt-in capability registration and typed access contract | YT-030, YT-026 | — |
| `YT-033` | medium | Verify one independent optional capability in isolation | YT-032 | — |
| `YT-034` | medium | Todo demo migration and baseline regression | YT-030, YT-031, YT-033 | [measured M5 and before/after comparison](docs/M6-VERIFICATION.md) |
| `YT-035` | medium | Keyed binding/static-slot component authoring prototype | YT-025 | [M7b evidence](docs/M7B-VERIFICATION.md) |
| `YT-036` | medium | Browser-local reactive update graph | YT-035 | [M7c evidence](docs/M7C-VERIFICATION.md) |
| `YT-037` | medium | Keyed lifecycle, focus and form-state preservation | YT-036 | [M7d evidence](docs/M7D-VERIFICATION.md) |
| `YT-038` | medium | SSR/WASM renderer regression and production bundle review | YT-036, YT-037 | [M7e evidence](docs/M7E-VERIFICATION.md) |
| `YT-039` | medium | Renderer-neutral identity, key and safe slot contracts | YT-038 | [M7f evidence](docs/M7F-VERIFICATION.md) |
| `YT-040` | medium | Extract synchronous local graph into an optional production crate | YT-039 | [M7g evidence](docs/M7G-VERIFICATION.md) |
| `YT-041` | medium | General keyed DOM reconciliation and binding ownership primitive | YT-039 | [M7h evidence](docs/M7H-VERIFICATION.md) |
| `YT-042` | medium | Migrate production Todo Browser local list to promoted M7 primitives | YT-040, YT-041 | [M7i evidence](docs/M7I-VERIFICATION.md) |
| `YT-043` | medium | Final M7 regression, architecture decision and evidence freeze | YT-042 | [M7j evidence](docs/M7J-VERIFICATION.md) |
| `YT-044` | medium | Standardize runnable sample structure, catalog and documentation | YT-043 | [M8a evidence](docs/M8A-VERIFICATION.md) |
| `YT-045` | medium | API-only REST sample with typed CRUD and HTTP errors | YT-044 | [M8b evidence](docs/M8B-VERIFICATION.md) |
| `YT-046` | medium | SSR shopping cart with explicit opt-in Session provider | YT-045 | [M8c evidence](docs/M8C-VERIFICATION.md) |
| `YT-047` | medium | Developer-experience and boilerplate audit across Todo, REST and Cart | YT-045, YT-046 | [M8d evidence](docs/M8D-VERIFICATION.md) |
| `YT-048` | medium | Full M8 regression, dependency/bundle review and evidence freeze | YT-047 | [M8e evidence](docs/M8E-VERIFICATION.md) |
| `YT-049` | medium | Plan, document and independently review M9 before code | YT-048 | [Planning evidence](docs/M9-PLANNING-VERIFICATION.md) |
| `YT-050` | medium | Promote `hello-world` to verified runnable catalog | YT-049 | [M9a evidence](docs/M9A-VERIFICATION.md) |
| `YT-051` | medium | Build `middleware-lab` | YT-050 | [M9b evidence](docs/M9B-VERIFICATION.md) |
| `YT-052` | medium | Build `forms-validation` | YT-051 | [M9c evidence](docs/M9C-VERIFICATION.md) |
| `YT-053` | medium | Build `file-server` | YT-052 | [M9d evidence](docs/M9D-VERIFICATION.md) |
| `YT-054` | medium | Build SQL-first, opt-in `sqlite-crud` without ORM | YT-053 | [M9e evidence](docs/M9E-VERIFICATION.md) |
| `YT-055` | medium | Build `sse-notifications` | YT-054 | [M9f evidence](docs/M9F-VERIFICATION.md) |
| `YT-056` | medium | Build `multi-island-dashboard` | YT-055 | [M9g evidence](docs/M9G-VERIFICATION.md) |
| `YT-057` | medium | Build security-bounded `auth-notes` | YT-056, YT-046 | [M9h evidence](docs/M9H-VERIFICATION.md) |
| `YT-058` | medium | Build `modular-shop` reference application | YT-057, YT-054, YT-056 | [M9i evidence](docs/M9I-VERIFICATION.md) |
| `YT-059` | medium | Audit API friction, classify contracts and freeze M9 evidence | YT-058 | [M9j evidence](docs/M9J-VERIFICATION.md) |
| `YT-060` | high | Choose repository/crate licenses and vulnerability-reporting policy | YT-059 | [M10a evidence](docs/M10A-VERIFICATION.md) |
| `YT-061` | high | Protect `master` and enable repository security automation | YT-059 | [M10b evidence](docs/M10B-VERIFICATION.md) |
| `YT-062` | medium | Define SemVer/API compatibility and document metadata contract | YT-059 | [M10c evidence](docs/M10C-VERIFICATION.md) |
| `YT-063` | high | Produce production edge/authentication threat model | YT-057, YT-059 | [M10d evidence](docs/M10D-VERIFICATION.md) |
| `YT-064` | medium | Design reproducible release artifacts and supply-chain evidence | YT-060, YT-062 | [M10e evidence](docs/M10E-VERIFICATION.md) |
| `YT-065` | medium | Add scheduled platform/browser/accessibility qualification | YT-059 | [M10f evidence](docs/M10F-VERIFICATION.md) |
| `YT-066` | medium | Qualify distributed providers and operational behavior | YT-046, YT-054, YT-057, YT-059 | [M10g evidence](docs/M10G-VERIFICATION.md)<br>[YT-066b1](docs/YT-066B1-VERIFICATION.md)<br>[YT-066b2](docs/YT-066B2-VERIFICATION.md)<br>[YT-066b3](docs/YT-066B3-VERIFICATION.md) |
| `YT-067` | high | Perform Codex-assisted internal security scan | YT-063, YT-064 | [M10h evidence](docs/M10H-VERIFICATION.md) |
| `YT-068` | medium | Refresh developer entry, candidate documentation and evidence-preserving navigation/status | YT-067 | [Documentation-stage record](docs/README.md#documentation-delivery-status--yt-068)<br>[final Private source/package boundary](docs/YT-020-PRIVATE-RELEASE-PREP.md#current-preparation-boundary--t08--yt-068-2026-10-12) |
| `YT-069` | medium | Integrated M11 documentation and permitted self-review | YT-068, YT-023 | [Plan](docs/M11-PLAN.md), [documentation-only self-review](docs/M11-PLANNING-REVIEW.md) |
