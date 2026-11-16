# M27-A — Developer Experience Gap Report

**Date:** 2026-11-16
**Baseline:** `master@89e9cc47eac20786b2d2c13a2c02e9bdc06b18dc`
**Published pair:** SDK `0.9.0-alpha.8` / CLI `0.4.0`
**Status:** Audit complete; findings are not implementation authorization

## 1. Executive summary

The current Yetlet experience is materially more complete than the earlier YT-102 audit baseline. CLI 0.4.0 has an SDK-aware `new` path, creates an exact SDK-pinned application lock and provenance receipt, provides separate environment and project-plan reports, and exposes explicit Native, Service, Vite and Browser creation choices. M26 also gives eligible simple Native applications a short `App::serve()` startup while preserving custom listener/lifecycle paths where they are required.

The audit found **two confirmed, bounded documentation gaps**:

| ID | Severity | Surface | Finding | Practical effect |
|---|---|---|---|---|
| DX-01 | P2 | `docs/CLI-GUIDE.md` first-use sequence | The guide says `new` does not generate `Cargo.lock`, contradicting CLI 0.4.0, the CLI contract and the current First App guide | Users may unnecessarily regenerate a lock that `new` already created, changing the resolved dependency set and weakening the intended exact-lock/provenance workflow |
| DX-02 | P2 | Generated `--service` README | The generated README does not document the service scaffold's health routes or bind environment contract, although help advertises bind/health/shutdown and the generated source implements them | Developers integrating probes or configuring a container must inspect generated Rust source to discover the service's primary operational interface |

No current evidence justifies a new Core API, DI container, ORM, generic controller layer, mandatory frontend runtime, new process supervisor, or new public CLI command. The earlier YT-047 no-runtime-API-change decision remains consistent with the current 13-sample catalog. YT-102's earlier setup/diagnostic research must not be repeated as if it were new; current sources and actual workflows were checked against those historical findings.

## 2. Method and evidence boundary

The six journeys in `M27-DX-PLAN.md` were checked against current source, maintained documentation, generated output and focused local execution.

- CLI binary built from the current checkout using locked/offline Cargo; observed version: `yetlet 0.4.0`.
- Existing protected Rust/Cargo 1.99 physical toolchain was selected for the actual CLI workflow. No Runner permissions, toolchain directory modes, global installations or system settings were changed.
- First Native application was created offline with exact SDK `0.9.0-alpha.8`; the CLI generated `Cargo.lock` and `.yetlet/sdk-resolution.json`. Locked dependencies were fetched offline, `yetlet build` succeeded, and `yetlet run` served both routes over HTTP. Ctrl+C exited with 130; the CLI process was reaped and port 7878 was confirmed closed.
- Actual `--service`, `--service --settings`, `--vite` and standalone Vite scaffolds were created with the same exact SDK and offline metadata cache. Their generated source, README and settings were inspected.
- Browser execution-plan behavior was checked through protected standalone fixtures. An actual Browser `new --offline` attempt correctly stopped because this cache did not contain the Browser template's required metadata set. No online fallback was attempted.
- Native Vite and Browser settings combinations were checked with `inspect --json`, including profile overlays and scoped ENV overrides. Invalid profile/undeclared-key errors returned exit 1, empty stdout and fixed safe messages without echoing supplied values.
- No full Vite/npm install, Browser/WASM build, Browser runtime, Windows runtime or performance benchmark was performed in this audit. These are not claimed as PASS.

## 3. Journey findings

### J1 — First Native application

**Result: first-app path works; one documentation contradiction.**

The current CLI help describes the creation contract: exact registry SDK pins, `Cargo.lock`, bounded lock generation, no package/tool installation and no build/listener. `creation.rs` performs SDK resolution, stages the generated tree and lock, and publishes the tree only after lock generation. The generated receipt records the exact selected SDK and startup profile.

Observed first-app sequence:

1. `yetlet doctor --json` passed when the existing protected physical Rust 1.99 toolchain directory was selected.
2. `yetlet new m27-first-app --parent … --sdk-version 0.9.0-alpha.8 --offline` succeeded.
3. The generated tree contained the exact SDK pin, standalone workspace, Cargo.lock, public CLI TOMLs and `.yetlet/sdk-resolution.json`.
4. `cargo fetch --locked --offline` and `yetlet build` succeeded.
5. `yetlet run` served the escaped root content and `/hello/Ada`; Ctrl+C cleaned up the owned server.

The exact generated lock was retained throughout fetch/build/run. This is consistent with `docs/FIRST-APP.md` and the accepted YT-247 CLI contract, but contradicts the maintained first-use paragraph in `docs/CLI-GUIDE.md` (DX-01). The raw shared README template still contains legacy no-lock wording, but `creation.rs` replaces that text and removes the lock-generation command before publishing the generated README; the retained Native and Service output READMEs are correct. That raw template text is not a user-visible finding and is not selected for remediation.

The source-build prerequisite remains explicit: the CLI selects Yetlet's `signing` feature, which pulls OpenSSL; building the CLI from source therefore needs OpenSSL development files. This is a known, accurately documented cost, not a newly discovered defect. Do not restructure the signing dependency without demonstrated consumer impact and a separate compatibility design.

The default Runner Rustup shim/toolchain directories were group-writable, so Doctor correctly refused them as unsafe. Selecting an existing protected physical Rust 1.99 toolchain made both probes pass. This is the intended trust boundary, and the guides already instruct users to select protected physical tools; no permission relaxation is recommended.

### J2 — Settings-backed application and diagnostics

**Result: current separation is coherent; no confirmed gap.**

- `doctor` is an environment-only report (Rust/Cargo identity and compiled filesystem capability). It deliberately does not inspect an app, secrets, frontend assets or runtime readiness.
- `inspect` is a separate read-only project report. It selects exactly a standalone protected `Cargo.toml` and existing `Cargo.lock`, then explains the public settings-derived build/run/watch plan.
- The two versioned JSON schemas remain separate. `inspect` does not claim that tools, assets, TLS files, secrets or runtime startup are ready.
- Profile selection is explicit: Development, Staging or Production; default Production. Scoped `APP_CLI__…` overrides must target keys declared in base settings.
- Focused checks confirmed safe, bounded errors for invalid profiles and undeclared related keys. Error output used empty stdout and did not echo the supplied values.

The current help, CLI guide and contract explain this division and its limitations. Extending Doctor or merging the two reports would weaken a deliberate boundary rather than solve a demonstrated problem.

### J3 — Vite / Style workflow

**Result: settings decisions are discoverable and internally consistent; no confirmed gap.**

Actual native Vite scaffolding sets the Vite root and enables independent `auto_start` and `auto_build` defaults. `inspect` showed:

- Production with both enabled: frontend build is selected for `build`; Vite dev startup is selected for `run`; `watch` additionally forces Rust watch.
- Development overlay with `auto_start = false`: `run` skips Vite while `build` still builds it.
- Scoped `APP_CLI__VITE__AUTO_START=false`: only the run/start stage changes; the independent build decision remains selected.
- Rust watch is a separate opt-in; `watch` forces it for that invocation after settings validation.

The guide explains the frontend root, prerequisites, explicit package preparation, initial dist requirement, manual/two-terminal fallback and the fact that no npm install or tool download is hidden. Style is separately opt-in and remains app-owned. No parallel frontend supervisor or implicit installer is justified.

### J4 — Service lifecycle

**Result: lifecycle architecture is intentional; generated README has a discoverability gap (DX-02).**

The actual `--service` scaffold preserves an app-owned Axum listener rather than using the simple M26 `App::serve()` path. It provides:

- `GET /health/live` returning HTTP 200;
- `GET /health/ready`, driven by an app-owned readiness flag;
- For `--service` without `--settings` or `--vite`, `YETLET_HOST` defaults to `127.0.0.1` and accepts only `127.0.0.1` or explicitly selected `0.0.0.0`; `YETLET_PORT` defaults to `3000` and accepts decimal ports 1–65535.
- For `--service --settings` or `--service --vite`, bind values come from typed `server.host` / `server.port` settings (defaults `127.0.0.1` / `3000`) and `APP_SERVER__HOST` / `APP_SERVER__PORT` overrides. The Service validation allows `127.0.0.1` or `0.0.0.0` and nonzero ports 1–65535. Settings-enabled startup also requires the operator-supplied `APP_SERVICE__TOKEN`.
- SIGINT/SIGTERM handling on Unix and Ctrl+C/Ctrl+Break on Windows, with readiness set false before Axum graceful shutdown.

This is a valid separate startup profile: the simple startup API intentionally does not absorb custom bind, health and shutdown policy.

However, the generated `--service` READMEs only discuss generic build/run and configuration-only Docker/Compose/Kubernetes generation. They do not name the health routes, explain readiness transition or distinguish the two bind-configuration paths. The top-level `new --help` only summarizes this as “bind/health/shutdown”. Since these are the primary capabilities selected by `--service`, the generated README should describe the common health/shutdown contract and both bind paths: `YETLET_HOST` / `YETLET_PORT` for the plain Service profile, and `server.host` / `server.port` plus `APP_SERVER__HOST` / `APP_SERVER__PORT` when Settings/Vite is also selected (DX-02). This is a documentation-only gap; no runtime behavior change is indicated.

### J5 — Browser-first application

**Result: decision model is clear; actual offline creation was blocked by a missing cache entry, not a product failure.**

The Browser target persists `cli.target = "browser"`, uses the frontend-owned WASM build and has no native Cargo build/run stage. The inspected plan confirmed:

- Browser `build` selects the frontend build only when `auto_build` is true.
- Browser `run` selects static preview only when `auto_start` is true.
- Browser `watch` requires both `auto_start` and `auto_build`; it performs serialized WASM builds and full-page reload, without native Rust watch or state restoration.
- Disabling either setting skips Browser watch without starting children.

The actual `new --template browser --offline` attempt stopped with an explicit missing Browser metadata-set error. The CLI did not silently use stale metadata or contact the network. This matches the documented offline contract. The Browser template's real creation/build/runtime remains unverified in this audit.

### J6 — Application authoring and sample composition

**Result: no new framework API candidate is supported.**

The stable catalog contains 13 runnable applications across SSR, API, forms, session, middleware, file, SSE, SQL, modular composition, full-stack islands and Style. Their READMEs and source retain ordinary Rust handlers, Axum extractors/results, explicit capability crates and application-owned policies.

The earlier YT-047/M8d audit already compared Todo, REST and Session Cart with reproducible source/dependency measurements and an independent Reviewer PASS. It found no justified runtime API change: listener bootstrap was intentionally native, JSON error envelopes and origin policies differed, forms/CSRF were Cart-specific, and Browser dependencies belonged only to Browser consumers. Current M26 samples now use the simple startup where appropriate while lifecycle-sensitive samples retain native listeners and graceful shutdown.

The newer sample shapes (for example Modular Shop and Multi-Island Dashboard) continue to own their application-specific database, identity, module, asset and shutdown decisions. No repeated stable need across two independent consumers was found that warrants a new Core abstraction. Do not reopen Controllers, DI, ORM, a generic forms runtime or a mandatory Browser layer.

## 4. Confirmed findings

### DX-01 — Correct the stale Cargo.lock creation statement

**Severity:** P2
**Target:** `docs/CLI-GUIDE.md`, first-use sequence around lines 348–355. The raw `tools/yetlet-cli/templates/SELECTED-README.md` still contains legacy wording, but `creation.rs` normalizes that wording and removes the redundant lock-generation command before publishing the generated README. The retained generated Native and Service READMEs are correct, so no generated-template change is justified.

The maintained CLI guide says `new` does not generate `Cargo.lock` and instructs users to generate a lock after creation. Current CLI 0.4.0 does the opposite: it resolves the exact SDK, generates the app lock in private staging and publishes the app with the lock and provenance receipt. This is also stated in `docs/FIRST-APP.md`, `docs/CLI-CONTRACT.md` and current `new --help`. The raw selected README template contains legacy text, but `creation.rs` normalizes it before publication; all retained generated output READMEs correctly describe the existing lock.

**Impact:** following the stale instruction can unnecessarily regenerate the dependency lock and create a different resolution from the exact lock/provenance emitted by `new`.

**Smallest correction:** update only the CLI guide's first-use sequence to state that `new` creates the lock and provenance receipt before publishing the app. Use the order: create the app; fetch the already-created lock with `cargo fetch --locked`; prepare selected frontend dependencies/assets; then build/run. State that `new` does not install tools/packages, build the app or start a listener. Preserve the existing offline-cache and no-implicit-install boundaries; do not change the already-correct generated README output.

### DX-02 — Document the generated Service operational contract

**Severity:** P2
**Target:** a dedicated Service README fragment at `tools/yetlet-cli/templates/service/SELECTED-README.md`, appended by `scaffold::selected_files_version` only when `--service` is selected.

The current generated Service README does not describe the actual health endpoints, bind variables or readiness/shutdown semantics implemented in the generated `src/main.rs`.

**Impact:** developers integrating a load balancer/container probe or changing the service bind address must inspect source code to discover the interface that `--service` was specifically selected to provide.

**Smallest correction:** add a dedicated Service README fragment and append it from `scaffold::selected_files_version` only when `service == true`. Document the common health routes and local-process readiness transition during graceful shutdown. Distinguish the plain Service `YETLET_HOST` / `YETLET_PORT` path from the Settings/Vite `server.host` / `server.port` plus `APP_SERVER__HOST` / `APP_SERVER__PORT` path. Plain Native output must not contain the section. Do not claim dependency health, HA or production readiness. Keep the custom listener/lifecycle code unchanged.

## 5. Known constraints — not defects

| Constraint | Current treatment | Audit disposition |
|---|---|---|
| OpenSSL development dependency for CLI source build | Explicit in CLI guide and First App prerequisites; follows the selected signing feature | Known cost; revisit only with consumer evidence and a separate dependency/compatibility proposal |
| Protected physical Rust/Cargo paths | Doctor refuses group/world-writable tool paths; protected physical Rust 1.99 path passes | Correct security boundary; do not relax permissions |
| Offline Browser metadata cache miss | Explicit failure; no stale fallback or network access | Expected fail-closed behavior; populate the cache or use an explicitly online creation workflow |
| Inspect is not readiness certification | Explicitly reports settings-derived intentions only | Correct scope; do not expand its schema without a separate requirement |
| Browser full-page reload and no state restoration | Explicitly documented | Accepted limitation |
| Native Service startup differs from M26 simple `App::serve()` | Required for app-owned bind, health and graceful shutdown | Correct separation of concerns |

## 6. M27-A closeout

- Six journeys mapped to current commands, settings, generated source, SDK APIs, docs and samples.
- Two material documentation findings confirmed; no SDK/CLI source defect or new Core API gap demonstrated.
- YT-047 and YT-102 historical findings were reused and reconciled rather than repeated as new work.
- At M27-A audit closeout, no source, manifest, dependency, public API, CLI grammar, schema, Roadmap or Backlog was changed. After M27-B independent Reviewer R4 PASS, the two selected documentation remediations were registered as Todo items YT-260 and YT-261; no implementation was performed.
- No package release, registry publication, Pages deployment, push or merge was performed.
- Actual first Native creation/build/run and cleanup were verified. Service/Vite scaffolds were generated; Browser offline creation was blocked by a missing cache entry and was not retried online.
- The first-app locked fetch/build readback is retained in `.pipeline-output/m27-dx-audit-20261116/first-app-build-verification.log`. The focused HTTP/shutdown readback is retained in `.pipeline-output/m27-dx-audit-20261116/first-app-runtime-verification.log`; the CLI server log and both response bodies are retained beside it. The generated app, exact lock/receipt and build output remain in the Runner-private audit workspace.

M27-A is complete. M27-B selected the two documentation corrections and received independent Reviewer R4 PASS. The owner authorized YT-260/YT-261 implementation.

## 7. M27-B implementation and verification

- **YT-260:** corrected the CLI Guide first-use sequence. It now states that `yetlet new` generates `Cargo.lock` and `.yetlet/sdk-resolution.json` in private staging before publishing the app; `cargo fetch --locked` fetches that existing lock. It no longer instructs users to regenerate the lock.
- **YT-261:** added `tools/yetlet-cli/templates/service/SELECTED-README.md`. `scaffold::selected_files_version` appends this fragment only when `--service` is selected. The README documents liveness/readiness routes, readiness transition during graceful shutdown, plain Service bind variables, and the distinct Settings/Vite bind variables plus token requirement. Plain Native output does not include the section. No generated runtime source or listener behavior changed.
- **Generated output verification:** rebuilt CLI 0.4.0 and created exact SDK alpha.8 projects offline for plain Native, Service-only, Service+Settings and Service+Vite. All four contain Cargo.lock and the SDK-resolution receipt; only the three Service variants contain the Service section. Exact output hashes are in `.pipeline-output/m27-yt260-261-implementation-20261116/final-generated-scaffold-verification.log`.
- **Tests:** full `selected_scaffold` target: 2 passed, 0 failed, 2 intentionally ignored (explicit Vite binary path and optional cached runtime compile). `cargo fmt --all -- --check`, `cargo xtask check docs` (29 documents / 258 Backlog items) and `git diff --check` passed. Detailed evidence: `.pipeline-output/m27-yt260-261-implementation-20261116/final-validation.log`.
- Independent final Reviewer `wc_job_ab4Zl3f_I3Q3_Fu4` completed terminal/exit 0 with overall_status=pass, issues=[], required_followups=[] and optional_notes=[]. Both YT-260 and YT-261 are Done at the bounded documentation scope.
- No commit, push, PR, release or deployment has occurred; repository delivery remains uncommitted pending separate explicit authorization.
