# M27 — Real-world Developer Experience: Audit and Planning Proposal

**Date:** 2026-11-16
**Status:** Approved planning baseline; M27-A/B audit and prioritization only; no implementation authorized
**Baseline:** `master@89e9cc47eac20786b2d2c13a2c02e9bdc06b18dc`
**Published inputs:** SDK `0.9.0-alpha.8`, CLI `0.4.0`

## 1. Purpose

M27 evaluates whether the currently published Yetlet SDK and CLI provide a coherent, low-friction path for building and maintaining real applications. It is not a feature-count milestone, an ASP.NET Core parity exercise, or a proposal to expand Core by default.

The governing product constraints remain:

- Small, composable, HTML-first Rust framework.
- Native SSR and ordinary Axum/Tower interoperability remain first-class.
- Browser/WASM, Style, Settings, Service, Session, Auth and other capabilities remain explicit and opt-in.
- Handler-first and component-first application composition; no mandatory Controllers, DI container, ORM, frontend runtime, or framework-owned service locator.
- Simple first, customizable when needed. Existing CLI settings and Cargo own persistent behavior and dependency resolution.

M27 must identify concrete user-visible friction from actual current source and representative workflows, then recommend only the smallest changes that address demonstrated problems.

## 2. Current baseline and completed work to reuse

### 2.1 Current published experience

- SDK `0.9.0-alpha.8`: `App::serve()`, `App::serve_on(address)`, `into_router()`, and optional `server_axum::prelude`.
- CLI `0.4.0`: `new`, `doctor`, `inspect`, `build`, `run`, `watch`, `key`, and opt-in generators; SDK-aware scaffolding selects the simple startup only for the exact verified SDK capability/archive match.
- The generated simple Native and Native Style paths use the M26 startup where eligible. Settings/Vite and Service retain their distinct configured/graceful-shutdown startup paths.
- The current stable sample catalog contains 13 applications: Hello World, Todo, REST API, Session Cart, Auth Notes, Modular Shop, Middleware Lab, File Server, SSE Notifications, Forms Validation, SQLite CRUD, Multi-Island Dashboard, and Style Showcase.
- GitHub Pages Reader Edition now selects SDK alpha.8 / CLI 0.4.0 and has been deployed from the merged M26/docs consolidation baseline.

### 2.2 Existing decisions and research to preserve

- ADR 0001: separation of concerns.
- ADR 0005: opt-in capability modules.
- ADR 0006: handler-first; MVC/Controllers remain optional.
- ADR 0010: dynamic SDK version resolution and exact app lock/provenance.
- ADR 0011: application startup API.
- ADR 0012: curated Axum prelude.
- YT-102 DX audit: historical measurements and findings remain historical. It already studied warm CLI completion delay, first-use OpenSSL prerequisites, project/settings diagnostics, and the extension boundary. Do not repeat that research without checking the changed source and current consumer impact.
- YT-047: earlier sample DX/boilerplate audit; reuse its accepted conclusions rather than re-auditing unchanged surfaces.

## 3. Evidence collected for this proposal

This is a bounded repository/source/documentation audit with focused current CLI and first-app runtime checks, not a full cross-platform or full-feature qualification.

1. `AGENTS.md` confirms the HTML-first, handler-first, opt-in and evidence-gated product constraints.
2. `docs/CLI-GUIDE.md` and `docs/CLI-CONTRACT.md` define the published CLI grammar, SDK resolution, settings projection, generated startup profiles, process ownership, frontend root/dotenv confinement, and documented platform limits.
3. `docs/FIRST-APP.md` and `docs/SAMPLES.md` provide the first-app path and 13-entry runnable sample catalog.
4. `docs/ROADMAP.md` and `docs/BACKLOG.md` show M26/YT-251–259 completed. The formal Backlog marks YT-020 and YT-075–081 Done at their recorded bounded scopes. The compact `todo-ledger.json` still labels those same IDs blocked/todo, so it is stale and must not be treated as the current task authority; M27 should not reopen those completed items.
5. `tools/yetlet-cli/src` has distinct modules for creation, settings, doctor, inspect, SDK resolution, generators, filesystem safety and platform process supervision. This structure is evidence to inspect, not proof of friction.
6. The old YT-102 DX report identifies OpenSSL as a first-use CLI build prerequisite because of its then-current signing dependency graph, and describes the doctor/inspect distinction. These are **historical observations**. Reconfirm current dependency graph and installed-consumer behavior before recommending any change.
7. Current docs state that Doctor is environment-only and Inspect is project-oriented. M27 should assess whether that division is discoverable and sufficient, not silently expand either JSON contract.
8. The Runner shell initially omitted Cargo from PATH; the audit located the existing protected physical Rust/Cargo 1.99 toolchain without changing permissions. The current CLI 0.4.0 was built offline, a Native SDK alpha.8 app was created/fetched/built/run, two HTTP routes and owned shutdown were verified, and Service/Vite scaffolds plus Inspect settings plans were checked. Browser offline creation stopped on a missing metadata set; no Browser/WASM runtime, Windows runtime, full feature matrix or performance measurement is claimed. Existing M26 Windows and CI evidence remains source-bound to its recorded revisions.

## 4. M27 objectives

- **O1 — First successful application:** measure the actual path from published CLI installation to a running Native SSR app, including prerequisites, expected output, errors, and the first edit/restart loop.
- **O2 — Progressive complexity:** verify that simple Native remains simple, while Settings, Vite, Style, Service and Browser selections compose predictably without hidden installs or unexpected startup behavior.
- **O3 — Discoverability and diagnosis:** determine whether a new user can understand command selection, project readiness, settings errors, generated startup profile, and next action without reading implementation source.
- **O4 — Application composition:** inspect representative minimal, API, forms/session, and browser/island samples for repeated ceremony, confusing ownership, and places where Yetlet abstractions genuinely reduce application code.
- **O5 — Cross-platform parity:** distinguish Linux and native Windows x64 MSVC behavior from cross-compilation or CI-only evidence. Do not infer Windows runtime parity from MSVC compilation.
- **O6 — Preserve architecture:** every recommendation must state why it belongs in Core, an optional crate, the CLI, a sample/application, or documentation.

## 5. Audit journeys and scope

The audit uses a small set of representative journeys rather than reviewing every file or every historical task.

| Journey | Representative surface | Questions |
|---|---|---|
| J1: first native app | CLI install, `yetlet new`, generated README, `run` | Are prerequisites visible before failure? Is the default path genuinely short? |
| J2: settings-backed app | `--settings`, profiles, typed settings, `doctor` / `inspect` | Can a developer understand which settings apply and why? Are diagnostics actionable and redacted? |
| J3: Vite / Style | `--vite`, `--tailwind`, Style templates, `run` / `watch` | Are frontend root, tool prerequisites, build/watch behavior and failure states discoverable? |
| J4: Service lifecycle | `--service`, health/readiness, graceful shutdown | Is the extra lifecycle ceremony justified and explicit? |
| J5: browser-first app | Browser template, WASM build, static hosting | Are build prerequisites, SSR/fallback and deployment boundaries clear? |
| J6: application authoring | Hello World, REST, Forms, Cart, Todo, Modular Shop | Which patterns repeat, and are they application policy rather than missing framework features? |

Audit output must identify the exact current source paths and relevant docs for each finding. Historical measurements may provide context but must not be presented as current measurements.

## 6. Candidate problem areas — hypotheses, not accepted defects

Prioritize investigation in this order:

1. **First-use prerequisites and actionable failure guidance.** Reconcile the published CLI install/build prerequisites, including whether OpenSSL remains required by the current CLI dependency graph. Determine whether a clearer preflight/message/documentation change is sufficient before considering dependency-feature restructuring. No dependency change is preselected.
2. **Command and readiness discoverability.** Compare `doctor` (environment-only) with `inspect` (project-oriented) and the actual `new` / `build` / `run` / `watch` transitions. Determine whether current help and reports explain scope and next action. Do not change the versioned Doctor JSON schema without a separately justified compatibility proposal.
3. **Generated startup profile comprehension.** Check whether users can predict why simple `App::serve()`, configured listener, Service graceful shutdown, or Browser startup was selected. Prefer a decision table or generated README improvement over another flag.
4. **Frontend workflow transparency.** Validate the discoverability of Vite root, dotenv boundary, auto-build/auto-start, Rust watch, static dist and browser fallback. Do not create a parallel frontend supervisor or implicit package installer.
5. **Application-level composition friction.** Compare the representative samples for repeated setup and API friction. Only promote a framework API candidate if at least two independent real consumers demonstrate the same stable need and the abstraction respects existing ownership boundaries.
6. **Native Windows first-use and lifecycle parity.** Identify gaps in current native MSVC evidence versus Linux. Keep compile-only, cross-target and native runtime/browser evidence distinct.

Do not reopen WebGPU/3D, DI, ORM, generic Controllers, generic SPA/router, mandatory frontend tooling, or production HA as M27 work absent new consumer evidence.

## 7. Proposed M27 work slices

These are planning candidates, not registered Backlog IDs or implementation authorization.

| Order | Candidate slice | Expected decision / output | Entry gate |
|---|---|---|---|
| A | Journey and source audit | DX Gap Report with severity, reproducibility, current source identity and existing evidence reuse | This plan independently reviewed |
| B | Prioritize and classify gaps | Each gap assigned to docs, CLI, SDK Core, optional crate, sample, or defer | Gap report complete |
| C | Select the smallest high-value change | Exact contract, acceptance criteria, non-goals, dependencies and platform matrix | Independent review of proposal |
| D | Implement one vertical slice | One user-visible outcome with focused tests and source-bound evidence | Explicitly selected Backlog item and reviewed implementation plan |
| E | Integration and qualification | Relevant Linux, native Windows MSVC, Browser/Chrome, docs, and CI checks; only applicable lanes | Implementation Reviewer PASS |
| F | Reconcile and close | Backlog/Roadmap evidence, independent final review, Guarded local commit and later PR/CI as separately authorized | Exact frozen diff accepted |

M27 may conclude that a documentation or diagnostic correction is the highest-value first change. It is not required to add a new SDK API.

## 8. Required deliverables and acceptance

### M27-A — DX Gap Report

- Six journeys above are mapped to exact current CLI commands, settings, generated files, SDK APIs and sample references.
- Each finding is classified as confirmed current friction, historical-only observation, accepted limitation, documentation/discoverability issue, or unverified hypothesis.
- Every confirmed finding includes a reproduction path or direct source evidence, user impact, affected platform, severity and existing evidence that can be reused.
- At least two independent sample/application consumers are required before proposing a generalized Core API.
- No product source, public contract, dependencies, manifest, or release identity is changed.

### M27-B — prioritized product plan

- A small ranked set of candidates with value, cost, risk, ownership layer, dependencies, and acceptance evidence.
- Explicitly list deferred areas and explain why they do not belong in the selected scope.
- Assign formal Backlog IDs only after the plan is independently reviewed and accepted.
- Preserve M26 completion and all existing Done/Blocked states until new evidence supports a status change.

### Independent review gate

The complete plan (M27-A findings and M27-B prioritization) must receive an independent Reviewer result with terminal execution, exit code 0, actual structured `overall_status=pass`, `issues=[]`, and `required_followups=[]`. A self-review or a static diff inspection is not a substitute. If the configured independent reviewer dispatch cannot be established, report that gate as blocked; do not claim a PASS or emulate independence.

## 9. Non-goals and safety boundaries

- No SDK/CLI implementation, dependency changes, public API changes, CLI grammar changes, schema changes or sample rewrites in the audit/plan stage.
- No SDK/CLI package release, signing, registry publication, Pages deployment or changes to the published Reader Edition.
- No push, merge, remote branch deletion, archival tag mutation, or cleanup of unrelated worktrees/caches.
- No broad rerun of accepted tests. Reuse evidence with exact source identity and run only focused checks when an observed gap requires it.
- Do not claim production readiness, universal Windows/browser support, performance improvements, or accessibility conformance without matching evidence.

## 10. Current repository state after PR #94 closeout

- Local Ubuntu `master` is synchronized to `origin/master` at `89e9cc47eac20786b2d2c13a2c02e9bdc06b18dc`.
- The prior local head is preserved as `archive/pre-squash-master-0fd3f84`.
- The permanent annotated tag `archive/docs-consolidation-z18-20261115` still targets `e4d17e8ba6791e1c1fc25c7d4e7498b941b76812`.
- Product runtime source, manifests and dependencies remain unchanged. M27 registration is in `ROADMAP.md` and `BACKLOG.md`; implementation includes the CLI Guide correction, a conditional Service README fragment, and its narrowly scoped README composition/test changes. The M27 audit/plan documents remain untracked, with ignored evidence under `.pipeline-output/`. Other worktrees and Windows validation checkouts were left untouched.
- After independent Reviewer R4 PASS, YT-260 and YT-261 were registered and implemented. Independent final Reviewer `wc_job_ab4Zl3f_I3Q3_Fu4` PASS closed both items at the accepted documentation scope; evidence is under `.pipeline-output/m27-yt260-261-implementation-20261116/`.
- Git delivery remains uncommitted because the owner authorized implementation, not Commit/Push/PR.
