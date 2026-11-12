# M26 — Application Startup & Sample Experience

**Status:** YT-251–258 independently accepted. PR #86 merged, exact-master Rust, Docs and CodeQL passed, and native Windows MSVC passed 148 tests.
**Date:** 2026-11-11
**Repository:** `bohewu/yetlet`
**Base:** `master` at `83d10df6c2a219c1f7271419ad4b1b04c78741bb` (PR #85, CLI package README follow-up)
**Working branch:** `codex/m26-application-startup-dx`
**Release baseline:** SDK `0.9.0-alpha.7`; CLI `0.3.1`; Reader Edition selects the same published pair
**Owner revision (2026-11-11):** Local HTTP startup port changed from `3000` to `7878` because another service already occupies `127.0.0.1:3000` on the Ubuntu Runner. All original reviewed contract decisions besides the port stay intact. The port amendment received independent Reviewer PASS in Job `wc_job_eJ9LTvWN2G-Yt7Oh` (terminal exit 0, `gpt-6-astra`/high, zero issues), separate from the earlier YT-251 contract review. Preserve both sets of evidence.

## 1. Purpose

M26 improves Yetlet's application-startup and sample-authoring experience. It is not a new server architecture, a second HTTP runtime, or a CLI release task.

The governing principle is:

> **Simple first, customizable when needed.**

A developer should be able to create and serve a small Yetlet application without manually constructing a TCP listener or learning Axum's server-startup ceremony. When the application needs custom middleware, listener ownership, graceful shutdown, TLS, multiple listeners, or another lifecycle policy, it must be able to use the existing Axum/Tower APIs directly.

M26 has three outcomes:

1. A concise `App::serve()` path with a safe loopback development default, plus `App::serve_on(address)` for explicit bind addresses, while preserving `into_router()` as the full-control escape hatch.
2. A small optional `server_axum::prelude` that reduces repeated import paths without flattening the crate namespace or introducing a redundant Core prelude.
3. A coherent rewrite of all 13 official catalog samples so their primary examples teach the recommended API, while samples whose purpose includes custom server lifecycle continue to demonstrate native Axum.

## 2. Repository baseline and numbering

- The latest `master` at planning time is `83d10df6c2a219c1f7271419ad4b1b04c78741bb` (PR #85).
- YT-249 is Reader Edition publication follow-through and is Done.
- YT-250 is CLI 0.3.1 release preparation/publication and is Done.
- M25 is already occupied by YT-219–YT-226: broader HTML DOM support and static-site delivery. It is historical and must not be reused.
- The next available milestone is **M26**; the next available work-item range is **YT-251–YT-258**.
- The official catalog in `examples/samples.toml` contains exactly 13 applications.

All new backlog items are initially `Todo`. A plan review is not an implementation review. Each YT item requires its own independent Reviewer PASS before it can be marked Done or its dependent item is admitted.

## 3. User-facing API direction

### 3.1 Simple startup

Before YT-255, the minimal SDK example manually converted the App to a Router, bound a Tokio listener, and called `axum::serve`. The accepted checkout example now uses the following local-development path:

~~~rust
use yetlet::server_axum::prelude::*;

async fn home() -> View {
    View::text("Hello from Yetlet")
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    App::new()
        .page("/", "Yetlet", home)
        .serve()
        .await
}
~~~

Applications that need a non-default bind address use the explicit variant:

~~~rust
App::new()
    .page("/", "Yetlet", home)
    .serve_on("0.0.0.0:8080")
    .await?;
~~~

YT-251 must confirm the exact contract. The current proposal is:

~~~rust
const DEFAULT_LISTEN_ADDRESS: std::net::SocketAddr =
    std::net::SocketAddr::from(([127, 0, 0, 1], 7878));

impl App<()> {
    pub async fn serve(self) -> std::io::Result<()> {
        self.serve_on(DEFAULT_LISTEN_ADDRESS).await
    }

    pub async fn serve_on<A>(self, address: A) -> std::io::Result<()>
    where
        A: tokio::net::ToSocketAddrs,
    { /* bind and serve */ }
}
~~~

The default is deliberately a loopback-only development listener at `127.0.0.1:7878`. The implementation constant need not become another public API unless a real consumer demonstrates that it needs to inspect it.

Intended semantics:

1. Both methods consume a configured `App<()>`; the router state must already be resolved.
2. `serve()` binds the fixed default loopback address and delegates to `serve_on`.
3. `serve_on(address)` binds the caller-supplied TCP socket address, accepting normal Tokio `ToSocketAddrs` inputs such as `&str` and `SocketAddr`.
4. Both reuse the existing `into_router()` conversion and call native `axum::serve`.
5. Address-resolution and listener-bind errors are returned unchanged. After a successful bind, Axum 0.8.9's serving future is non-returning; its TCP listener handles accept errors internally rather than returning them to the application.

Contract constraints:

- `serve()` and `serve_on()` are available only when Axum Router state is fully resolved (`App<()>`). An unresolved `App<S>` must not be served accidentally.
- The default is exactly `127.0.0.1:7878` over plain HTTP. It is intentionally loopback-only so a first run does not expose a development server to the LAN.
- `serve()` does not inspect environment variables, configuration files, hostnames, certificates or deployment mode. A port conflict is returned as an ordinary bind error; there is no silent fallback to another port.
- `serve_on` requires an explicit socket address including a port. `0.0.0.0` is an explicit request to listen on all IPv4 interfaces and must be documented as network exposure, not a safe default.
- These convenience methods provide HTTP only. They do not accept URL schemes, infer HTTPS from a port/host/environment, create self-signed certificates, load private keys, or enable TLS implicitly.
- HTTPS is not the default because useful TLS requires explicit certificate/private-key provisioning, renewal and trust decisions. For common production deployments, terminate TLS at a reverse proxy/load balancer (for example Nginx or BunkerWeb) and connect to the app over a protected internal HTTP path. Direct TLS termination remains possible through an explicitly configured native Axum/Tower accept loop; it is outside these convenience methods and M26.
- Applications must not treat the loopback HTTP default as production deployment guidance. Do not expose plain HTTP on an untrusted network; configure an HTTPS edge or an explicit direct-TLS integration.
- Both methods are async and do not create, own, or hide a Tokio runtime. The application still declares its own Tokio dependency and runtime features (for example `#[tokio::main]`).
- Both methods consume the App and do not spawn a detached task. Address-resolution and bind errors propagate; after binding, Axum 0.8.9 handles accept errors internally (connection errors retry; non-connection errors are logged by Axum and followed by a one-second delay). Yetlet adds no startup output or custom post-bind error policy.
- `App::from_router(router).serve()` and `App::from_router(router).serve_on(address)` are supported when the supplied Router has no unresolved state.
- Neither method returns the bound listener or its actual local address. Port `0`, querying `local_addr()`, custom socket options, multiple listeners, graceful shutdown, TLS, and other listener ownership require `into_router()` plus native Tokio/Axum.
- No `serve_with_shutdown` convenience method is added in M26. Applications requiring graceful shutdown keep the native `TcpListener` + `axum::serve(...).with_graceful_shutdown(...)` path.
- Do not add aliases such as `listen`, `run`, or multiple competing startup names.
### 3.2 Dependency and feature boundary

`server-axum` is optional and native-only. Adding `App::serve` may require the existing optional Tokio dependency to become available under `server-axum`, but must not leak unrelated Tokio capabilities into ordinary server applications.

YT-251 must define and YT-253 must verify the exact feature arrangement:

- Keep Tokio optional and native-target-only.
- Enable the minimum Tokio feature set required for `TcpListener` and Axum serving when `server-axum` is selected.
- Keep Tokio macro/runtime/sync/time features explicitly associated with the optional consumers that need them, including `session-redis`; do not activate them accidentally merely because `server-axum` gains listener support.
- Preserve the default `html`-only dependency graph and the browser/WASM dependency boundary.
- Do not add a new crate or another HTTP server.

Use `cargo tree -e features` and targeted feature-matrix checks to prove the result. The exact transitive Tokio features required by the selected Tokio/Axum versions must be reported rather than guessed.

### 3.3 Optional Axum adapter prelude

The Core crate already re-exports `Attribute`, `Element`, and `View` at its root, so `use yetlet::View` and `use yetlet::{Element, View}` are already valid. A separate `yetlet::prelude` containing only those same names would add little value.

M26 therefore proposes one curated, optional `yetlet::server_axum::prelude` for application code:

~~~rust
use yetlet::server_axum::prelude::*;
~~~

Proposed exports, subject to YT-252's source/API audit:

| Area | Names |
|---|---|
| Core View values | `Attribute`, `Element`, `View` |
| Application | `App`, `Page`, `RequestContext` |
| Document metadata | `DocumentMetadata`, `LanguageTag`, `LanguageTagError` |
| Page errors and server policy | `HtmlPageError`, `DEFAULT_CSP` |
| Common extractors | `DefaultBodyLimit`, `Extension`, `Form`, `Path`, `Query`, `State` |
| Common HTTP values | `HeaderMap`, `HeaderValue`, `StatusCode` |
| Common responses | `Json`, `IntoResponse`, `Redirect`, `Response` |
| Rendering helpers | `render_document`, `render_document_with_metadata`, `render_page`, `render_page_with_metadata` |

`Form` is an additive re-export from Axum's existing `axum::extract::Form` into `server_axum::extract`; Axum 0.8.9's default `form` feature already supplies it. `HeaderMap`, `HeaderValue` and `StatusCode` are selected individual exports from the existing `server_axum::http` namespace. Do not re-export the entire `http` namespace.

Export `Json` from the existing `response` module only; do not import the same name through both `extract` and `response`. Do not re-export `Router`, `axum::serve`, `TcpListener`, Tokio, Tower middleware, `FormRejection`, `Multipart`, or the entire Axum namespace. Advanced infrastructure and feature-specific extractors remain explicit.
Prelude rules:

- It exists only when `server-axum` is enabled and remains unavailable in Core-only/browser builds.
- Existing explicit paths such as `yetlet::view::View`, `yetlet::server_axum::App`, `yetlet::server_axum::response::Json`, and `yetlet::server_axum::extract::Path` remain source-compatible.
- The prelude is optional; examples may use explicit imports where they teach a useful boundary.
- Do not flatten all public types into `yetlet::` or change Rust's normal `::` path syntax. The prelude is a convenience import, not a namespace redesign.

## 4. M26 backlog and dependency order

| Item | Priority / size | Title | Depends on |
|---|---|---|---|
| YT-251 | P1 / M | Axum Application Startup Contract (ADR-0011) | — |
| YT-252 | P1 / S | Axum Adapter Prelude Contract (ADR-0012) | — |
| YT-253 | P1 / M | Implement `App::serve()` and `App::serve_on()` | YT-251 |
| YT-254 | P1 / S | Implement `server_axum::prelude` | YT-252 |
| YT-255 | P1 / M | Migrate minimal and basic HTTP samples | YT-253, YT-254 |
| YT-256 | P1 / M | Migrate lifecycle-sensitive capability samples | YT-253, YT-254 |
| YT-257 | P1 / M | Migrate browser and visual samples | YT-253, YT-254 |
| YT-258 | P1 / S | Synchronize sample catalog and source documentation | YT-255, YT-256, YT-257 |

~~~text
YT-251 ──> YT-253 ──┐
                    ├──> YT-255 ──┐
YT-252 ──> YT-254 ──┼──> YT-256 ──┼──> YT-258
                    └──> YT-257 ──┘
~~~

YT-251 and YT-252 are independent contract tasks. YT-253 and YT-254 may proceed after their respective contract reviews pass. The three sample-migration items require both APIs to have passed their implementation reviews. YT-258 is the final documentation/catalog synchronization item.

### YT-251 — Axum Application Startup Contract

Deliverable: `docs/adr/0011-application-startup-api.md` plus this plan's source-backed contract detail.

Acceptance:

- Decide and document the exact `App::serve()` default and `App::serve_on(address)` signatures, `App<()>` state boundary, return type, address-resolution/bind error propagation, native Axum post-bind accept-error behavior, and listener ownership.
- Define the default loopback IP/port, explicit all-interface binding, port-zero behavior, HTTP-only/HTTPS/TLS boundary, and the no-hidden-runtime rule.
- Explicitly preserve `into_router()` and identify advanced cases that must continue using native Axum.
- Define the minimum Tokio feature/dependency boundary and a targeted test plan.
- Confirm that the API is additive and does not alter the default Core/browser dependency graphs.
- Independent Reviewer PASS on ADR and contract before YT-253 implementation begins.

### YT-252 — Axum Adapter Prelude Contract

Deliverable: `docs/adr/0012-server-axum-prelude.md` plus a source-verified export inventory.

Acceptance:

- Explain why the initial scope adds `server_axum::prelude` but not a redundant Core prelude.
- List every export and its exact source path; check names against the actual public modules and locked Axum 0.8.9 feature gates.
- Decide the additive `Form` re-export in `server_axum::extract` and verify Axum's `form` feature is selected without adding a new dependency.
- Include only selected common HTTP types (`HeaderMap`, `HeaderValue`, `StatusCode`), not the whole `http` namespace.
- Avoid duplicate/ambiguous `Json` exports and broad upstream namespace re-exports.
- Preserve all explicit import paths and the optional `server-axum` feature boundary.
- Include simple and advanced import examples.
- Independent Reviewer PASS before YT-254 implementation begins. **PASS:** `wc_job_leT3lB8xD2KJSSZS` (terminal exit 0; saved `reviewer` role, `gpt-6-astra`/high; zero issues and required followups).

### YT-253 — Implement `App::serve()` and `App::serve_on()`

Scope:

- Implement only the YT-251-approved `serve()` and `serve_on(address)` methods in the Axum adapter.
- Add the minimum Tokio dependency/features required by the method.
- Reuse `into_router()`; do not add a second router or server abstraction.
- Add Rustdoc for `App::new().serve()`, `App::new().serve_on(address)`, and both variants on `App::from_router(...)`.

Acceptance and validation:

- Successful HTTP response through the real `App::serve()` default path and explicit `App::serve_on(address)` path.
- Bind failure is returned to the caller with its I/O error intact.
- `App::from_router(router).serve()` and `.serve_on(address)` work for a resolved-state Router.
- Unresolved state cannot call `serve`; supplying state and resolving to `App<()>` can.
- Existing `into_router()` and Axum/Tower compatibility tests remain green.
- `cargo check --locked -p yetlet --no-default-features --features server-axum` passes.
- `cargo test --locked -p yetlet --features server-axum` passes.
- `cargo tree -e features` confirms no unrelated Tokio feature leakage; `session-redis` still receives the features it needs.
- The default/Core-only dependency graph remains free of Axum/Tokio.
- Run the relevant native compile/test scope on Linux and the existing Windows MSVC Runner when available; report each platform separately.
- Independent implementation Reviewer PASS before sample migration. **YT-253 PASS:** `wc_job_vX_D7m0O9q5on1hg` (terminal exit 0; saved Reviewer `gpt-6-astra`/high; zero blocking issues and required followups). Actual verification: [YT-253 record](YT-253-STARTUP-VERIFICATION-20261111.md). Native Windows execution remains pending.

### YT-254 — Implement `server_axum::prelude`

Scope:

- Add the curated `pub mod prelude` under the existing `server_axum` feature gate.
- Add `Form` to the existing `server_axum::extract` re-export module and include it in the prelude.
- Add compile/doctest coverage for the approved exports and a minimal server application.
- Keep explicit imports supported and document that the prelude is optional.

Acceptance:

- A minimal application compiles using `use yetlet::server_axum::prelude::*;`.
- Each export resolves to an existing Yetlet or Axum public API; `Form` is an additive re-export backed by Axum's already selected `form` feature, with no new dependency.
- `StatusCode`, `HeaderMap`, `HeaderValue` and `DEFAULT_CSP` resolve from their existing public paths.
- `Json` is exported once; no broad Axum/HTTP/Tower/Tokio re-export is added.
- Core-only and browser/WASM builds do not gain server-only symbols.
- Existing explicit import compatibility tests remain green.
- Independent implementation Reviewer PASS before sample migration. **YT-254 PASS:** `wc_job_t0nfWyjaZtjOd5Gx` (terminal exit 0; saved Reviewer `gpt-6-astra`/high; 0 issues/required followups). [Verification record](YT-254-PRELUDE-VERIFICATION-20261111.md) contains 12 retained commands and source/test hashes. Native Windows runtime remains unverified.

### YT-255 — Migrate minimal and basic HTTP samples

Catalog entries:

- `hello-world` (`examples/hello-world`)
- `rest-api` (`examples/rest-api`)
- `session-cart` (`examples/cart-server`)
- `middleware-lab` (`examples/middleware-lab`)
- `forms-validation` (`examples/forms-validation`)

Also migrate the SDK's non-catalog minimal example: `crates/yetlet/examples/server_axum/minimal.rs`.

Scope and acceptance:

- Use the prelude in the primary examples where it improves readability.
- Use `.serve()` only when a sample intentionally adopts the default `127.0.0.1:7878`; use `.serve_on(configured_address)` when preserving an existing environment/configuration-controlled address, including through `App::from_router(...)`.
- Keep the existing Tokio runtime declaration explicit in application manifests and code.
- Preserve environment variable names, configured bind addresses, routes, HTTP methods, response bodies, CSP, middleware ordering, Session/CSRF policy and sample commands.
- Preserve the 13 stable catalog IDs and README command lines.
- Where the old main printed `listener.local_addr()`, either keep native listener ownership if the actual bound address is a requirement or print the configured address only when that is semantically equivalent; do not silently lose port-zero support.
- Run affected package tests/builds and real HTTP smoke through both `App::serve()` and `App::serve_on(...)`; retain configured addresses and existing environment variable names.
- Independent Reviewer PASS.

### YT-256 — Migrate lifecycle-sensitive capability samples

Catalog entries:

- `file-server` (`examples/file-server`)
- `auth-notes` (`examples/auth-notes`)
- `modular-shop` (`examples/modular-shop`)
- `sse-notifications` (`examples/sse-notifications`)
- `sqlite-crud` (`examples/sqlite-crud`)

These current samples use explicit graceful shutdown in their server entry points. Their direct Axum lifecycle is intentional.

Scope and acceptance:

- Use the prelude for Yetlet values/responses where useful, but keep `TcpListener`, `axum::serve`, and `with_graceful_shutdown` visible.
- Do not force these samples through `App::serve` and do not introduce `serve_with_shutdown` as an M26 workaround.
- Preserve file/upload token boundaries, synthetic identity/Session/CSRF behavior, SQL transaction/idempotency semantics, SSE replay/disconnect behavior, and shutdown/drain ordering.
- Preserve sample commands, package identities and route declarations.
- Add a short README explanation that these are intentional advanced lifecycle examples.
- Run affected package tests and bounded startup/shutdown smoke checks, including listener closure and sample-owned cleanup.
- Independent Reviewer PASS.

### YT-257 — Migrate browser and visual samples

Catalog entries:

- `todo` (`examples/todo-server`)
- `multi-island-dashboard` (`examples/multi-island-dashboard`)
- `style-showcase` (`examples/style-showcase`)

Scope and acceptance:

- Use `App::from_router(...).serve_on(configured_address)` for Todo and native Style Showcase where their existing environment-configured addresses must remain; use `.serve()` only for an intentional default-address example.
- Keep `multi-island-dashboard` on native Axum because its current entry point owns graceful shutdown; explain this intentional exception.
- Preserve SSR-first behavior, browser build commands, WASM assets, island independence, lazy loading, local interaction, CSP, generated CSS and Style Showcase's optional-WASM status.
- Keep the catalog's `style-showcase` entry native-first (`browser_runtime = false`); do not create another sample or promote the optional browser companion into a required catalog dependency.
- Run affected native and browser tests/builds plus the existing bounded browser smoke relevant to changed startup/import code.
- Independent Reviewer PASS.

### YT-258 — Synchronize sample catalog and source documentation

Scope:

- Update `docs/SAMPLES.md` and `examples/README.md` to describe the simple startup path and the advanced native-Axum lifecycle path.
- Update the source-only section of `docs/API-REFERENCE.md` with `App::serve` and the optional prelude, explicitly stating that these additions are not part of the published alpha.7 SDK.
- Synchronize affected sample-local READMEs and Rustdoc examples after YT-255–257.
- Keep `examples/samples.toml` at exactly 13 entries with stable IDs, package names, routes, browser declarations and commands.

Acceptance:

- `cargo xtask check samples` passes all 13 catalog entries.
- `cargo xtask check docs` passes language, links, Markdown structure, whitespace and dependency DAG checks.
- `cargo fmt --all -- --check` and `git diff --check` pass.
- Every changed sample README command matches the catalog and remains copyable from the repository root.
- The API reference distinguishes source-only checkout additions from published SDK APIs.
- Independent final Reviewer PASS.

## 5. Official sample coverage

| Catalog ID | Package/directory | M26 item | Startup direction |
|---|---|---|---|
| `hello-world` | `yetlet-hello-world` / `examples/hello-world` | YT-255 | `App::from_router(...).serve_on(configured_address)`; preserve `YETLET_HELLO_LISTEN` |
| `rest-api` | `yetlet-rest-api` / `examples/rest-api` | YT-255 | `App::from_router(...).serve_on(configured_address)`; preserve `YETLET_REST_LISTEN` |
| `session-cart` | `yetlet-cart-server` / `examples/cart-server` | YT-255 | `App::from_router(...).serve_on(configured_address)`; preserve `YETLET_CART_LISTEN` |
| `middleware-lab` | `yetlet-middleware-lab` / `examples/middleware-lab` | YT-255 | `App::from_router(...).serve_on(configured_address)`; preserve `YETLET_MW_LISTEN` and Tower composition |
| `forms-validation` | `yetlet-forms-validation` / `examples/forms-validation` | YT-255 | `App::from_router(...).serve_on(configured_address)`; preserve `YETLET_FORMS_LISTEN` |
| `file-server` | `yetlet-file-server` / `examples/file-server` | YT-256 | Native Axum; graceful shutdown |
| `auth-notes` | `yetlet-auth-notes` / `examples/auth-notes` | YT-256 | Native Axum; graceful shutdown |
| `modular-shop` | `yetlet-modular-shop` / `examples/modular-shop` | YT-256 | Native Axum; graceful shutdown |
| `sse-notifications` | `yetlet-sse-notifications` / `examples/sse-notifications` | YT-256 | Native Axum; graceful shutdown |
| `sqlite-crud` | `yetlet-sqlite-crud` / `examples/sqlite-crud` | YT-256 | Native Axum; graceful shutdown |
| `todo` | `yetlet-todo-server` / `examples/todo-server` | YT-257 | `App::from_router(...).serve_on(configured_address)`; preserve `YETLET_LISTEN` |
| `multi-island-dashboard` | `yetlet-multi-island-dashboard` / `examples/multi-island-dashboard` | YT-257 | Native Axum; graceful shutdown |
| `style-showcase` | `yetlet-style-showcase` / `examples/style-showcase` | YT-257 | `App::from_router(...).serve_on(configured_address)`; preserve `YETLET_SHOWCASE_LISTEN`; optional WASM remains optional |

The table covers all 13 IDs from the current catalog. The SDK minimal example is an additional non-catalog example and is included in YT-255.

## 6. Release and documentation boundary

The published SDK `0.9.0-alpha.7` and CLI `0.3.1` are immutable release identities. M26 adds source APIs and migrates checkout samples; it does not publish a new SDK or CLI.

Do not modify as part of M26:

- `tools/yetlet-cli/templates/` or the published CLI 0.3.1 package/readme.
- `docs/FIRST-APP.md` where it represents the published Reader Edition's selected installation/API journey.
- `examples/docs-site/src/generated/chapters.json` or its pinned Reader Edition source snapshot.
- `site/pages.toml`, published version authorities, release tags, archives, checksums or publication records.
- SDK/CLI version numbers, registry publication workflows or release identities.

`docs/API-REFERENCE.md` may document the new API only under its explicit source-only checkout section. It must not imply that alpha.7 contains the new methods or prelude. After a future SDK release, a separate owner-approved release/Reader follow-through item can update the First App guide, CLI templates if needed, and Reader Edition against the accepted publication record.

## 7. Global constraints and non-goals

- No changes to Yetlet Core's dependency boundary or default feature set.
- No new runtime, controller hierarchy, MVC requirement, DI container, service locator, plugin registry, or second router.
- No hidden Tokio runtime, environment-dependent bind address, detached task, Yetlet-added startup logging, or implicit shutdown behavior; the documented loopback default remains explicit and fixed, while Axum retains its native accept-error logging policy.
- No removal or renaming of existing public module paths or `into_router()`.
- No mandatory prelude use; explicit imports remain first-class.
- No route/business-logic, persistence, security-policy, browser-rendering or asset-lifecycle redesign during sample migration.
- No new catalog entry; keep all 13 stable sample IDs.
- No production-readiness, performance, HA, TLS, or all-platform guarantee inferred from this DX batch.
- No Commit, Push, PR, Merge, release, publication, or Pages deployment without separate explicit authorization.

## 8. Review and execution gates

1. The complete M26 plan, new Backlog rows and Roadmap summary receive an independent read-only Reviewer review. Fix every blocking finding and obtain a follow-up PASS before treating planning as accepted.
2. YT-251 and YT-252 each receive an independent contract Reviewer PASS before their implementation items begin.
3. YT-253 and YT-254 each receive an independent implementation Reviewer PASS before any sample migration begins.
4. YT-255, YT-256 and YT-257 each receive their own independent sample Reviewer PASS.
5. YT-258 receives the final independent documentation/catalog Reviewer PASS.
6. Preserve original failures and exact evidence. Do not replay unrelated historical qualification or expand the validation harness.
7. Mark an item Done only after its scoped validation and independent Reviewer PASS are recorded. No item inherits a parent's PASS.

## 9. Planning review and validation evidence

- Independent saved Reviewer subagent Job: `wc_job_aW6Lqgq5TYz6DWi0`.
- Terminal state: completed; exit code `0`.
- Reviewer trace: role `reviewer`, model `gpt-6-astra`, effective effort `high`; model/role/effort matches and inheritance consistency all `true`.
- Reviewer result: `overall_status = pass`; `issues = []`; `required_followups = []`; `optional_notes = []`.
- Ignored evidence: `.pipeline-output/m26-application-startup-sample-review-20261111/reviewer-output.json` and `reviewer-trace.json`.
- Owner-clarification amendment Reviewer Job: `wc_job_IguwE8skhKa3EkkU` (re-reviewed the default bind and HTTP/TLS decisions).
- Amendment Reviewer terminal state: completed; exit code `0`.
- Amendment Reviewer trace: role `reviewer`, model `gpt-6-astra`, effective effort `high`; model/role/effort matches and inheritance consistency all `true`.
- Amendment Reviewer result: `overall_status = pass`; `issues = []`; `required_followups = []`; `optional_notes = []`.
- Ignored amendment evidence: `.pipeline-output/m26-application-startup-sample-review-20261111-amendment/reviewer-output.json` and `reviewer-trace.json`.
- `cargo xtask check docs`: PASS (75 documents, 255 backlog items).
- `cargo xtask check samples`: PASS (13 samples).
- `git diff --check`: PASS.
- Review scope is the M26 plan, Backlog rows, Roadmap summary and the source/compatibility surfaces listed in the review instruction. No product code was changed.
- YT-251 initial contract Reviewer Job `wc_job_zaxfmBinaf26QRTV`: terminal exit `0`, result `FAIL` with one P2 on Axum post-bind accept-error behavior.
- YT-251 corrective follow-up Reviewer Job `wc_job_Nad2HOpWhHREJOty`: terminal exit `0`, result `PASS`, zero issues and required followups.
- YT-251 follow-up trace: saved `reviewer` role, model `gpt-6-astra`, effective effort `high`; model/role/effort and parent inheritance checks all `true`.
- Ignored YT-251 evidence: `.pipeline-output/yt251-application-startup-contract-review-20261111/reviewer-output.json`, `reviewer-output-v2.json`, `reviewer-trace-v1.json`, and `reviewer-trace-v2.json`.
- YT-252 independent Reviewer Job `wc_job_leT3lB8xD2KJSSZS`: terminal exit `0`, result `PASS`, zero issues and required followups.
- YT-252 Reviewer trace: saved `reviewer` role, model `gpt-6-astra`, effective effort `high`; model/role/effort and parent inheritance checks all `true`.
- Ignored YT-252 evidence: `.pipeline-output/yt252-application-prelude-contract-review-20261111/reviewer-output.json` and `reviewer-trace.json`.

- YT-251 owner port-amendment Reviewer Job `wc_job_eJ9LTvWN2G-Yt7Oh`: terminal exit `0`, independent saved `reviewer` model `gpt-6-astra`, effective effort `high`, `overall_status = pass`, zero issues/required followups; trace matches role/model/effort.
- Ignored port-amendment evidence: `.pipeline-output/yt251-port7878-amendment-review-20261111/reviewer-output.json` and `reviewer-trace.json`.

### YT-253 implementation evidence (owner default-port revision)

The initial YT-253 Reviewer `wc_job_CdqciKQzlAcfXH-o` completed with exit code `0` but returned **FAIL** with one evidence-only P2 because prior Cargo command summaries lacked retained raw outputs. The original JSON remains at `.pipeline-output/yt253-application-startup-implementation-review-20261111/reviewer-output.json`.

[YT-253 verification record](YT-253-STARTUP-VERIFICATION-20261111.md) now points to `artifacts/verification/yt253-port7878-20261111-a/`: 15 exact argv manifests, full stdout/stderr logs, command exit-code table, SHA-256 checksums and source identity. All 15 checks exited `0`. Live HTTP on the **new** `127.0.0.1:7878` default passed (4/4 startup tests; **no skip**). The original `127.0.0.1:3000` port remains occupied by an unrelated service and was not modified. Windows MSVC was cross-compiled on Linux, not executed on a native Windows Runner.

Owner-amended YT-251 port review `wc_job_eJ9LTvWN2G-Yt7Oh` PASS and YT-253 implementation follow-up `wc_job_vX_D7m0O9q5on1hg` PASS (both terminal exit 0; saved `gpt-6-astra`/high). Both Reviewer JSON outputs, role traces, and the original YT-253 evidence-only FAIL remain intact. The YT-253 Reviewer inspected all 15 retained command logs and both source/test SHA-256 hashes independently. Native Windows runtime remains unverified; Windows MSVC cross-target checking is documented accurately.

### YT-254 source verification evidence

[YT-254 verification record](YT-254-PRELUDE-VERIFICATION-20261111.md) points to `artifacts/verification/yt254-prelude-20261111-a/`: 12 exact argv manifests, complete logs, exit codes, SHA-256 checksums and source/test identity. All 12 checks exited `0`; the two new Prelude tests and two explicit-path compatibility tests passed, as did the complete Axum adapter, Clippy, Core/WASM feature boundaries, and Windows MSVC **cross-target** build. Native Windows runtime is unverified. Independent YT-254 implementation Reviewer Job `wc_job_t0nfWyjaZtjOd5Gx` completed exit 0, saved `gpt-6-astra`/high, JSON PASS with zero issues/followups; role/model/effort trace PASS. The accepted source and evidence are tracked in the YT-254 verification record.

## 10. Next handoff

YT-251–254 retain their independent acceptance. YT-255–257 are now Done after
separate independent role-directed Reviewer Jobs, each terminal exit 0 and JSON
PASS. Their accepted commits are recorded below.

| Task | Reviewer Job | Commit |
|---|---|---|
| YT-255 | `wc_job_BZbrOzBZvM6IlT-B` | `40ca677887e30c4895112bd003963aa31329b70c` |
| YT-256 | `wc_job_DFU2zIIEJzJBYxkF` | `083804c17217f3f65c79d1368def531f9e55ef81` |
| YT-257 | `wc_job_Xsa68FvOKhLXEQJD` | `caa3758c94b47d09f273cb3d3743398210c78da6` |

YT-258 source documentation/catalog synchronization passed independent Reviewer
`wc_job_BujoBeCZVwUl_ZGG`, terminal exit 0, JSON `overall_status=pass`, zero
issues/followups. The catalog retains thirteen stable IDs, package/command
identities and browser metadata, including Style Showcase's optional companion.
PR #86 passed exact-head CI and merged at
`9d1075ab6442a9d8cb58606c170327aa32799bf7`. Rust, Docs and CodeQL passed
on that exact master commit. Native Windows MSVC qualification ran on the second
Windows Runner in a separate clean checkout and passed 148 tests across the
SDK Axum adapter and all 13 official samples. The final 2026-11-11 M26 handoff
records the test scope and preserves the original Windows failure evidence.
[PR #86](https://github.com/bohewu/yetlet/pull/86)

The work used branch `codex/m26-application-startup-dx`. Published SDK alpha.7, CLI
0.3.1, CLI templates, release records, Reader Edition and published First App
remain at their accepted release boundaries.
