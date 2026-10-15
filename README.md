# Yetlet

Yetlet is a small, HTML-first Rust universal UI framework. Write shared Rust Views for native server-rendered pages and opt-in browser WASM/DOM updates; use ordinary Axum handlers for HTTP and typed actions for explicit remote work.

**Experimental `0.1.0`, unpublished, Private.** M0–M9 implementation, selected M10 qualification and M11-P/M11-G product/GPU work are complete at their recorded prototype scopes. [M11 closeout](docs/YT-081-M11-CLOSEOUT.md) reconciles those limits; APIs are pre-public and may change. Public release (`YT-020`) remains **Blocked**; this repository is not production-qualified.

## Start with native SSR

From this checkout, install the Rust/Cargo 1.99.0 toolchain pinned in `rust-toolchain.toml`. Native SSR needs no JavaScript, WASM target or browser build tools.

```bash
cargo run --locked -p yetlet-server-axum --example minimal
```

Open `http://127.0.0.1:3000/` for a script-free HTML page and `/api/status` for JSON. The complete [minimal example](crates/yetlet-server-axum/examples/minimal.rs) is:

```rust
use yetlet_core::View;
use yetlet_server_axum::{App, response::Json};

async fn home() -> View {
    View::text("Hello from Yetlet")
}
async fn status() -> Json<&'static str> {
    Json("ready")
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app: axum::Router = App::new()
        .page("/", "Yetlet", home)
        .group("/api", |api| api.get("/status", status))
        .into_router();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
    axum::serve(listener, app).await?;
    Ok(())
}
```

`App` is a thin facade that returns a native `axum::Router`. `.page` accepts a no-argument async View handler; parameterized or fallible handlers use `.get`, native Axum extractors and `Page` or `Result<Page, E>`. `Page` provides escaped HTML, strict CSP and generic rendering-error responses. See [HTTP usage](docs/HTTP-PIPELINE.md) and [compatibility/API boundaries](docs/COMPATIBILITY.md).

For a small application with named-route input and a sample-local guide:

```bash
cargo run --locked -p yetlet-hello-world
```

Open `http://127.0.0.1:3003/` or `/hello/World`. See [Hello World](examples/hello-world/README.md).

## Add browser behavior when needed

The Todo application shares its Rust View between SSR and the browser. `/app` owns local keyed updates and a separate typed HTTP Create action; `/search` renders complete HTML before lazily loading its local Search Island.

Browser builds additionally require Python 3.11+, `wasm32-unknown-unknown` and wasm-bindgen CLI 0.2.129:

```bash
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version 0.2.129 --locked \
  --root "$HOME/.local/share/yetlet-tools"
python3 scripts/build_browser.py
cargo run --locked -p yetlet-todo-server
```

Open `http://127.0.0.1:3000/`, `/app` and `/search`. Stop the minimal server first because both use port 3000. Assets are built ahead of time into ignored `dist/` with a content-hash manifest; requests never compile WASM. `/app` returns 503 if browser assets are absent. Use `python3 scripts/build_browser.py --release` for optimized assets; debug and release measurements are separate.

## Choose an application example

The [sample guide](docs/SAMPLES.md) lists twelve runnable applications with commands, routes, dependencies, security limits and retained evidence. Start with native Hello World or REST; explore Session cart, middleware, forms, files, SQLite, SSE, islands, synthetic authentication and the Modular Shop as needed. Each [sample README](examples/README.md) owns its setup details. SQL and storage remain application choices; Yetlet has no built-in ORM or mandatory database layer.

## Nine library candidates

These are the original nine unpublished `0.1.0` candidates. Use them from the checkout; no registry installation or stable API is claimed. Each package guide below describes its implemented usage, dependencies, ownership and limits.

| Crate | Responsibility |
|---|---|
| [`yetlet-core`](crates/yetlet-core/README.md) | Renderer-neutral View/component model and validated identity, key and binding-slot vocabulary |
| [`yetlet-html`](crates/yetlet-html/README.md) | Escaped HTML serialization with safe element/attribute policy |
| [`yetlet-dom`](crates/yetlet-dom/README.md) | Direct browser DOM mount over a restricted HTML subset |
| [`yetlet-dom-keyed`](crates/yetlet-dom-keyed/README.md) | Optional keyed child identity and disposable listener/entry ownership |
| [`yetlet-reactive`](crates/yetlet-reactive/README.md) | Optional synchronous local message/reducer/commit graph with explicit lifetime |
| [`yetlet-actions`](crates/yetlet-actions/README.md) | Typed command and local/HTTP executor boundaries |
| [`yetlet-server-axum`](crates/yetlet-server-axum/README.md) | App/Page facade, request context and native Axum/Tower composition |
| [`yetlet-request-id`](crates/yetlet-request-id/README.md) | Opt-in process-local numeric request correlation |
| [`yetlet-session`](crates/yetlet-session/README.md) | Opt-in process-local server-side Session, expiry, CSRF and cookie lifecycle |

The separate experimental Redis provider is outside these nine candidates and remains nonpublishable. Its retained qualification is limited to the selected single-host/single-primary/no-persistence scope.

## Experimental opt-in scene renderer

[`yetlet-wgpu`](crates/yetlet-wgpu/README.md) is a separate first-party
**Experimental, unpublished, opt-in** flat-scene renderer. CPU defaults have no
GPU dependency or initialization; only explicit wasm32 `browser` activation
adds WebGPU. It does not accelerate ordinary DOM/CSS or alter the nine release
candidates. The [support boundary](crates/yetlet-wgpu/SUPPORT.md) separates the
qualified Firefox/X11/Intel opt-in configuration from unverified platforms.
There is no performance, FPS, stable-API or production guarantee. Future 3D
is motivation, not a shipped 3D API. See the [real consumer](examples/webgpu-scene-island/README.md).

## Design boundaries

- Default SSR works without JS/WASM or a persistent server-side event circuit.
- Browser events stay local. Remote operations cross explicit typed HTTP/action boundaries; the server validates inputs, authentication and authorization.
- Views, renderers, HTTP routing, loaders and transport remain separable. HTTP uses Axum/Tower rather than a custom parser/server stack.
- Escaping, semantic HTML, accessibility and strict CSP are correctness requirements. Browser state must contain no server secrets or capabilities. WASM pages may allow `'wasm-unsafe-eval'`, without `'unsafe-eval'` or `'unsafe-inline'`.
- Optional capabilities use explicit Rust setup functions and scoped middleware. The minimal app needs no Session or request-ID provider.
- Keyed DOM/local reactivity are opt-in per owned surface. Only the Todo local list adopts them; remote Todo and Search results retain bounded subtree replacement.

Generic hydration, resumability, general VDOM/template macros and dependency-tracked signals/forms remain unselected. The five original research tracks retain their bounded application-owned [decisions and measurements](docs/README.md#completed-research-prototypes), not supported core APIs. Separately approved M11 work now provides the Experimental opt-in scene candidate above; the old native Vulkan study does not establish its browser support or accelerate DOM/CSS. See [architecture decisions](docs/README.md#architecture-and-contracts).

## Developer documentation and checks

Use the [documentation index](docs/README.md) for API guidance, architecture, qualification and historical evidence. [Roadmap](docs/ROADMAP.md) and [Backlog](docs/BACKLOG.md) own milestone and task status. Documentation and sample navigation checks are:

```bash
python3 scripts/check_docs.py
python3 scripts/check_samples.py
```

For implementation validation, use the changed-path checks required by [repository instructions](AGENTS.md) and the relevant sample/verification record. Historical compiler, browser, CSP, benchmark and CI results retain their original dates and source identities; navigation changes do not rerun or transfer those results. [Retained history](docs/README.md#historical-evidence-and-maintainer-notes) includes the former detailed command and milestone notes.

## Qualification and release limits

The original nine candidates remain experimental `0.1.0`, unpublished and Private. [Release readiness](docs/RELEASE-READINESS.md) and [Private preparation](docs/YT-020-PRIVATE-RELEASE-PREP.md#current-preparation-boundary--t08--yt-068-2026-10-12) explain the Rust 1.99 minimum, best-effort maintenance and remaining approval gates. YT-068 is Done at documentation scope; the independent delivery gate/result is recorded separately in the run; documentation-stage checks do not attest that gate. YT-020 remains **Blocked** for formal activation.

Historical [Private signing](docs/YT-020-PRIVATE-SIGNING-VERIFICATION.md) authenticates the earlier `8e557e760407a3d4b3f5a8b8f833748946be1387` evidence set. It does not attest this later committed source tree, evolved root manifest/lock, nine package guides or Experimental GPU candidate. A future release needs a separately approved immutable candidate and current-source package, rebuild, provenance, SBOM/license and signing evidence; a local commit alone cannot activate release.

[Retained qualification](docs/QUALIFICATION-MATRIX.md) keeps its original native/browser/CI identities. YT-030/YT-065/YT-066/YT-067 remain Done at their stated bounded scopes. Real IME, spoken assistive technology, WebKit/mobile and zoom remain optional unverified coverage; no supported-device, conformance, production or SLA claim follows. The Redis provider remains nonpublishable and qualified only at selected single-host/single-primary/no-persistence scope. Application TLS/ACL, backups, business RPO/RTO and operational ownership remain application responsibilities. [Detailed history](docs/README.md#historical-evidence-and-maintainer-notes) preserves earlier commands, PASS results and limitations.

## Governance and license

Yetlet is a maintainer-directed project. External pull requests are not accepted; repository settings restrict pull-request creation to collaborators so maintainers can still use PRs and required CI internally. Reproducible bugs, compatibility reports, documentation feedback, and focused design proposals are welcome through Issues. Security reports must follow [SECURITY.md](SECURITY.md) and must not be posted publicly. See [CONTRIBUTING.md](CONTRIBUTING.md) for the complete policy.

Yetlet is dual-licensed under your choice of [MIT](LICENSE-MIT) or [Apache License 2.0](LICENSE-APACHE). Workspace packages carry the SPDX expression `MIT OR Apache-2.0`; example and application packages remain `publish = false`. The license permits forks and independent modifications but does not require the upstream project to accept external code contributions.
