# YT-187 — Product README and developer entry paths

Date: 2026-10-26 (Asia/Taipei). Status: **Done at product/DX documentation scope
after independent Reviewer follow-up PASS, zero issues/required followups.**

## User direction and scope

The maintainer asks for a DX view of documentation access/planning, explicitly
requests a README rewrite, and selects a product-oriented presentation: explain
what Yetlet does, who it serves, value, first use and next documentation paths.
Unfinished work belongs in Todo/Backlog/Roadmap, not a running README task log.
Preserve the preceding YT-186 document work and historical evidence. No runtime,
sample, old validation replay, publication, Commit or Push is selected.

## Selected entry contract

- The root README introduces an HTML-first Rust UI framework for Rust/Axum
  application authors. Product positioning must describe implemented capabilities;
  no invented adoption, price, benchmark advantage, release or support claims.
- Give one first-run route from the current checkout, with its prerequisite,
  copyable command, expected URLs and exact existing minimal source. Preserve
  the existing `start-with-native-ssr` anchor used by other guides.
- Organize next links by developer task (HTTP pages, browser interaction,
  forms/Session, identity, rate limiting, configuration and existing applications),
  not milestone sequence or original-versus-later release candidates.
- Keep preview availability/compatibility clear in one short note. Detailed
  package/platform/safety limits stay in the current guides. No task IDs,
  Reviewer results, cloud instructions, pending-feature list or release-progress
  diary remains in the new root body. Root links to Backlog/Roadmap own planning.
- Retain the previous root README below as an explicitly dated snapshot. Only
  relative links and heading depth are adapted to this archive location; original
  content, source IDs, verdicts and limits stay intact. This archive is outside
  the reader publication selection under the YT-186 scope.

## Static DX findings and bounded plan

The prior root interleaves first use, CLI bootstrap, container/Kubernetes output,
package release groups and private qualification. A reader must pass operational
and milestone material to find later API navigation. Current API entry and package
READMEs already exist; this slice promotes those routes instead of creating a
new tutorial/sample or another signature inventory.

Four static walkthroughs define the minimum document result: run native SSR from
README; find current API/rustdoc in one direct link; choose an existing opt-in or
application through task links; find planned work in Backlog/Roadmap. This is
source/navigation inspection, not a measured novice study or a usability score.
The larger guide bodies and site destination are separate future choices; this
slice does not silently expand into a full documentation rewrite or new tooling.

Only README, Backlog, Roadmap, one index link and this record change for YT-187.
The baseline is the reviewed, uncommitted ten-file YT-186 delivery at HEAD
`332462b31f6d549b46977c885d71036cb7acc715`. Preserve its six non-shared document
paths byte-for-byte; changes to root/index/planning are this explicitly requested
follow-up, not evidence that the old review covers the new README.

Run only the required new-document gate `cargo xtask check docs`, whitespace
and bounded source/link/snapshot comparison; no runtime tests, rustdoc generation,
sample gate or browser. Independent read-only Reviewer PASS is required before
Done. Raw old README/baseline/commands stay in
`target/yt187-readme-dx-20261026/evidence/`.

## Minimum delivery evidence

Root README decreases from 196 to 108 lines. Its complete Rust block matches
`crates/yetlet-server-axum/examples/minimal.rs` exactly, excluding only the source's
module-doc comment. Existing root-anchor callers still target the retained native
SSR heading. Current API, sample and opt-in links point to existing guides; no
new sample, signature, feature or package support is claimed.

The new-delivery `cargo xtask check docs` exits 0 (75 required documents,
187 backlog items; links/structure/whitespace/DAG PASS); `git diff --check` passes.
New-record no-index whitespace exits 1 with no diagnostics, the expected
content-difference result. Six non-shared YT-186 document hashes and 492 earlier
protected source/guide/history hashes match. Exactly five paths change for
YT-187; the combined YT-186/187 checkout contains eleven paths, staged zero.

One ad hoc no-task-tracking observation initially matched the legitimate Todo
sample name and failed. The original failure is retained; only that expression
was narrowed and the failed source observation repeated. No product edit or
passing document/runtime proof replay followed. No server, browser, rustdoc,
test/build/benchmark, external service, publication, Commit or Push ran.
The separate final acceptance below closes review; this is static DX/source review,
not a measured user study or a production-readiness claim.

The initial independent review returns FAIL with one P2: browser task links
lost the WASM target and matching wasm-bindgen CLI prerequisites. That verdict
is preserved, not relabeled. This violates the original usable next-step path,
so a minimal root-only fix now states the required target/locked CLI version,
installation location and PATH selection. `xtask/src/build.rs`, `tooling.rs` and
`Cargo.lock` confirm those instructions by source inspection. No tool install,
browser build or passing runtime qualification is repeated. Follow-up independent
review now returns PASS; all other scope and preservation requirements stay unchanged.

## Final acceptance

Independent Reviewer `/root/yt187_product_readme_dx_review_20261026` returns
follow-up PASS, zero issues/required followups. Preserve its original one-P2 FAIL
and the minimal browser-prerequisite correction separately. Native role/model/
effective effort match the saved profile and central resolver. All five source,
18 evidence and six non-shared YT-186 source bindings match before/after final
review; 492 earlier source/guide/history inputs remain unchanged. Only record,
Backlog/Roadmap/index completion metadata follows. Root README is frozen at the
reviewed 108-line product entry; the full prior snapshot is intact below.

YT-187 is Done at bounded product/DX documentation scope. No new unblocked
implementation is selected; YT-096 remains In progress with D1–D6 Pending,
YT-020 remains Blocked. Publication/Commit/Push and runtime qualification were
not performed. Combined YT-186/187 changes remain eleven unstaged paths.

## Prior root README — historical snapshot

The following is the root README immediately before this product-entry rewrite,
including the earlier YT-186 fixes. It is retained context, not current usage or
status instructions. Relative links are rebased from the repository root to
`docs/`, and headings are nested two levels deeper. Code blocks are unchanged.

<details>
<summary>Root README snapshot — 2026-10-26</summary>

### Yetlet

Yetlet is a small, HTML-first Rust universal UI framework. Write shared Rust Views for native server-rendered pages and opt-in browser WASM/DOM updates; use ordinary Axum handlers for HTTP and typed actions for explicit remote work.

**Experimental `0.1.0`, unpublished, Private.** M0–M9 implementation, selected M10 qualification and M11-P/M11-G product/GPU work are complete at their recorded prototype scopes. [M11 closeout](../docs/YT-081-M11-CLOSEOUT.md) reconciles those limits; APIs are pre-public and may change. Public release (`YT-020`) remains **Blocked**; this repository is not production-qualified.

#### Start with native SSR

From this checkout, install the Rust/Cargo 1.99.0 toolchain pinned in `rust-toolchain.toml`. Native SSR needs no JavaScript, WASM target or browser build tools.

```bash
cargo run --locked -p yetlet-server-axum --example minimal
```

Open `http://127.0.0.1:3000/` for a script-free HTML page and `/api/status` for JSON. The complete [minimal example](../crates/yetlet/examples/server_axum/minimal.rs) is:

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

`App` is a thin facade that returns a native `axum::Router`. `.page` accepts a no-argument async View handler; parameterized or fallible handlers use `.get`, native Axum extractors and `Page` or `Result<Page, E>`. `Page` provides escaped HTML, strict CSP and generic rendering-error responses. See [HTTP usage](../docs/HTTP-PIPELINE.md) and [compatibility/API boundaries](../docs/COMPATIBILITY.md).

For a small application with named-route input and a sample-local guide:

```bash
cargo run --locked -p yetlet-hello-world
```

Open `http://127.0.0.1:3003/` or `/hello/World`. See [Hello World](../examples/hello-world/README.md).

#### Optional local Yetlet CLI preview

The separate [unpublished CLI](../tools/yetlet-cli/README.md) creates native SSR applications and executes `run`/`build`. Persistent CLI/Vite behavior uses the existing settings mechanism. `new --vite` implies the optional runtime settings module; plain new keeps it opt-in.

```bash
cargo build --offline --locked --manifest-path tools/yetlet-cli/Cargo.toml
tools/yetlet-cli/target/debug/yetlet --help
tools/yetlet-cli/target/debug/yetlet doctor --json
mkdir -p -m 700 "$HOME/yetlet-apps"
tools/yetlet-cli/target/debug/yetlet new sampleapp --parent "$HOME/yetlet-apps"
# Follow the generated README: unpublished crates need explicit Cargo-owned local overrides.
cargo generate-lockfile --offline --manifest-path "$HOME/yetlet-apps/sampleapp/Cargo.toml"
tools/yetlet-cli/target/debug/yetlet build --manifest-path "$HOME/yetlet-apps/sampleapp/Cargo.toml"
tools/yetlet-cli/target/debug/yetlet run --manifest-path "$HOME/yetlet-apps/sampleapp/Cargo.toml"
```

Creation runs no tools or installation and generates no app lock. Native `watch` forces Rust rebuild/restart through Watchexec. Browser-target `watch` requires frontend `auto_start` and `auto_build` and performs WASM rebuild/full-page reload; Vite HMR is a separate frontend path. The [current command/capability inventory](../tools/yetlet-cli/README.md#current-command-and-capability-inventory) owns the target-specific requirements. Linux and Windows evidence remains bound to each recorded source and workflow; the latest naming/help checks do not requalify process, browser or platform behavior. Windows run requires a protected local NTFS app/tools and an attached processed-input console. See [CLI implementation evidence](../docs/YT-102-CLI-IMPLEMENTATION.md) and [browser-watch evidence](../docs/YT-112-IMPLEMENTATION.md). Global installation, publication and production qualification remain separate.

#### Optional Linux container and Kubernetes configuration

The unpublished CLI can scaffold a [service SSR app](../docs/YT-089A-VERIFICATION.md) with local-process liveness/readiness, configurable bind address, and graceful SIGTERM/Ctrl-C handling. The separate [configuration-only generators](../tools/yetlet-cli/README.md) emit Dockerfile, Compose and generic Linux/amd64 Kubernetes Deployment/Service files with an explicit, consistent Cargo.lock and compatible local Cargo path overrides retaining pinned versions. The [current generator source matrix](../tools/yetlet-cli/README.md#current-generator-source-matrix) covers the four exact current service profiles, including settings/Vite/Tailwind combinations. A separate [source-closure repair](../docs/YT-102-GENERATOR-SOURCE-CLOSURE.md) restores the fixed Config/Secrets export without enabling a secret provider: all 12 generator outputs and one sanitized Vite-source offline Cargo build passed focused checks. This is source/configuration evidence; historical platform and deployment qualification remains source-bound. Registry-only, standalone Tailwind and unsupported/edited layouts remain refused:

```sh
YETLET=tools/yetlet-cli/target/debug/yetlet
# Select an existing qualified legacy local-source app with its prepared lock.
"$YETLET" generate docker --manifest-path "$HOME/yetlet-apps/cloudapp/Cargo.toml" --output "$HOME/yetlet-apps/docker-output"
"$YETLET" generate compose --manifest-path "$HOME/yetlet-apps/cloudapp/Cargo.toml" --output "$HOME/yetlet-apps/compose-output" --image localhost:5000/team/cloudapp:v1
"$YETLET" generate k8s --manifest-path "$HOME/yetlet-apps/cloudapp/Cargo.toml" --output "$HOME/yetlet-apps/k8s-output" --image localhost:5000/team/cloudapp:v1
```

Generation **does not** run Docker or Kubernetes, fetch dependencies, export private source, publish images, or deploy a cluster. Docker builds require separate audited `app-src` and `yetlet-src` named BuildKit contexts; deployment images/credentials, ports and namespaces remain operator responsibilities. Linux OCI/K3s output and Windows GNU/MSVC generator parity have [separate source-bound evidence](../docs/YT-089E-VERIFICATION.md). M15 is an unpublished preview, not production or YT-020 release qualification.

#### Optional typed application configuration

The separate unpublished [configuration guide](../docs/CONFIGURATION.md) shows typed
Serde settings, explicit Development/Staging/Production layers, development-only
secret files and native Axum State. The isolated workspace leaves default/core/
browser dependencies unchanged. Current CLI templates opt into this same mechanism. [YT-090 verification](../docs/YT-090-VERIFICATION.md)
records focused Linux checks and independent Reviewer PASS at the unpublished scope.

#### Add browser behavior when needed

The Todo application shares its Rust View between SSR and the browser. `/app` owns local keyed updates and a separate typed HTTP Create action; `/search` renders complete HTML before lazily loading its local Search Island.

Browser builds additionally require `wasm32-unknown-unknown` and wasm-bindgen CLI 0.2.129. The unpublished Rust `cargo xtask` provides build, check, smoke, measure, qualification and release-preparation commands. Original Python files remain frozen reference oracles. YT-085 is accepted after its separate Reviewer PASS; [YT-086 verification](../docs/YT-086-VERIFICATION.md) records the migration candidate and remaining acceptance boundaries. No Python interpreter is required by the migrated command implementations.

```bash
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version 0.2.129 --locked \
  --root "$HOME/.local/share/yetlet-tools"
cargo xtask build browser
cargo run --locked -p yetlet-todo-server
```

Open `http://127.0.0.1:3000/`, `/app` and `/search`. Stop the minimal server first because both use port 3000. Assets are built ahead of time into ignored `dist/` with a content-hash manifest; requests never compile WASM. `/app` returns 503 if browser assets are absent. Use `cargo xtask build browser --release` for optimized assets; debug and release measurements are separate.

#### Choose an application example

The [sample guide](../docs/SAMPLES.md) lists twelve runnable applications with commands, routes, dependencies, security limits and retained evidence. Start with native Hello World or REST; explore Session cart, middleware, forms, files, SQLite, SSE, islands, synthetic authentication and the Modular Shop as needed. Each [sample README](../examples/README.md) owns its setup details. SQL and storage remain application choices; Yetlet has no built-in ORM or mandatory database layer.

#### Nine library candidates

These are the original nine unpublished `0.1.0` candidates. Use them from the checkout; no registry installation or stable API is claimed. Each package guide below describes its implemented usage, dependencies, ownership and limits.

| Crate | Responsibility |
|---|---|
| [`yetlet-core`](../crates/yetlet/guides/core.md) | Renderer-neutral View/component model and validated identity, key and binding-slot vocabulary |
| [`yetlet-html`](../crates/yetlet/guides/html.md) | Escaped HTML serialization with safe element/attribute policy |
| [`yetlet-dom`](../crates/yetlet-browser/guides/dom.md) | Direct browser DOM mount over a restricted HTML subset |
| [`yetlet-dom-keyed`](../crates/yetlet-browser/guides/dom-keyed.md) | Optional keyed child identity and disposable listener/entry ownership |
| [`yetlet-reactive`](../crates/yetlet/guides/reactive.md) | Optional synchronous local message/reducer/commit graph with explicit lifetime |
| [`yetlet-actions`](../crates/yetlet/guides/actions.md) | Typed command and local/HTTP executor boundaries |
| [`yetlet-server-axum`](../crates/yetlet/guides/server-axum.md) | App/Page facade, request context and native Axum/Tower composition |
| [`yetlet-request-id`](../crates/yetlet/guides/request-id.md) | Opt-in process-local numeric request correlation |
| [`yetlet-session`](../crates/yetlet/guides/session.md) | Opt-in process-local server-side Session, expiry, CSRF and cookie lifecycle |

The separate experimental Redis provider is outside these nine candidates and remains nonpublishable. Its retained qualification is limited to the selected single-host/single-primary/no-persistence scope.

The [current capability and API carrier map](../docs/CAPABILITIES.md) also lists
later opt-in Config, Secrets, Signing, Auth and rate-limit packages, selected
features, application integrations and research decisions. Their existence does
not expand these nine release candidates or imply production qualification.

#### Experimental opt-in scene renderer

[`yetlet-wgpu`](../crates/yetlet-browser/guides/wgpu.md) is a separate first-party
**Experimental, unpublished, opt-in** flat-scene renderer. CPU defaults have no
GPU dependency or initialization; only explicit wasm32 `browser` activation
adds WebGPU. It does not accelerate ordinary DOM/CSS or alter the nine release
candidates. The [support boundary](../crates/yetlet-browser/guides/wgpu-support.md) separates the
qualified Firefox/X11/Intel opt-in configuration from unverified platforms.
There is no performance, FPS, stable-API or production guarantee. Future 3D
is motivation, not a shipped 3D API. See the [real consumer](../examples/webgpu-scene-island/README.md).

#### Design boundaries

- Default SSR works without JS/WASM or a persistent server-side event circuit.
- Browser events stay local. Remote operations cross explicit typed HTTP/action boundaries; the server validates inputs, authentication and authorization.
- Views, renderers, HTTP routing, loaders and transport remain separable. HTTP uses Axum/Tower rather than a custom parser/server stack.
- Escaping, semantic HTML, accessibility and strict CSP are correctness requirements. Browser state must contain no server secrets or capabilities. WASM pages may allow `'wasm-unsafe-eval'`, without `'unsafe-eval'` or `'unsafe-inline'`.
- Optional capabilities use explicit Rust setup functions and scoped middleware. The minimal app needs no Session or request-ID provider.
- Keyed DOM/local reactivity are opt-in per owned surface. Only the Todo local list adopts them; remote Todo and Search results retain bounded subtree replacement.

Generic hydration, resumability, general VDOM/template macros and dependency-tracked signals/forms remain unselected. The five original research tracks retain their bounded application-owned [decisions and measurements](../docs/README.md#completed-research-prototypes), not supported core APIs. Separately approved M11 work now provides the Experimental opt-in scene candidate above; the old native Vulkan study does not establish its browser support or accelerate DOM/CSS. See [architecture decisions](../docs/README.md#architecture-and-contracts).

#### Developer documentation and checks

Start with the [current API reference](../docs/API-REFERENCE.md) for package entry
points, opt-in selection and local rustdoc. The [documentation scope](../docs/DOCUMENTATION-SCOPE.md)
separates reader sources from maintainer/evidence records; the mixed
[repository index](../docs/README.md) retains architecture and delivery history.
The [naming guide](../docs/NAMING.md) owns current Rust/CLI/settings vocabulary and
pre-public migrations. [Roadmap](../docs/ROADMAP.md) and [Backlog](../docs/BACKLOG.md)
own milestone and task status. Documentation and sample navigation checks are:

```bash
cargo xtask check docs
cargo xtask check samples
```

For implementation validation, use the changed-path checks required by [repository instructions](../AGENTS.md) and the relevant sample/verification record. Historical compiler, browser, CSP, benchmark and CI results retain their original dates and source identities; navigation changes do not rerun or transfer those results. [Retained history](../docs/README.md#historical-evidence-and-maintainer-notes) includes the former detailed command and milestone notes.

#### Qualification and release limits

The original nine candidates remain experimental `0.1.0`, unpublished and Private. [Release readiness](../docs/RELEASE-READINESS.md) and [Private preparation](../docs/YT-020-PRIVATE-RELEASE-PREP.md#current-preparation-boundary--t08--yt-068-2026-10-12) explain the Rust 1.99 minimum, best-effort maintenance and remaining approval gates. YT-068 is Done at documentation scope; the independent delivery gate/result is recorded separately in the run; documentation-stage checks do not attest that gate. YT-020 remains **Blocked** for formal activation.

Historical [Private signing](../docs/YT-020-PRIVATE-SIGNING-VERIFICATION.md) authenticates the earlier `8e557e760407a3d4b3f5a8b8f833748946be1387` evidence set. It does not attest this later committed source tree, evolved root manifest/lock, nine package guides or Experimental GPU candidate. A future release needs a separately approved immutable candidate and current-source package, rebuild, provenance, SBOM/license and signing evidence; a local commit alone cannot activate release.

[Retained qualification](../docs/QUALIFICATION-MATRIX.md) keeps its original
native/browser/CI identities; later [desktop acceptance](../docs/YT-074-DESKTOP-ACCEPTANCE.md)
records selected real OS IME and spoken assistive-technology observations.
Those observations are source/device-bound, not current-checkout or universal
device/conformance/production/SLA qualification. Optional unverified coverage
remains explicit. The Redis Session provider remains nonpublishable; its original
single-host/no-persistence evidence and later bounded
[Valkey/TLS cross-host and orderly-restart reference](../docs/YT-184-AZURE-AUTH-SESSION-RESTART.md)
retain separate source/configuration limits, without a general HA, failover or
arbitrary-restore guarantee. Application TLS/ACL policy, backups, business RPO/RTO
and operational ownership remain application responsibilities.
[Detailed history](../docs/README.md#historical-evidence-and-maintainer-notes)
preserves earlier commands, PASS/FAIL results and limitations.

#### Governance and license

Yetlet is a maintainer-directed project. External pull requests are not accepted; repository settings restrict pull-request creation to collaborators so maintainers can still use PRs and required CI internally. Reproducible bugs, compatibility reports, documentation feedback, and focused design proposals are welcome through Issues. Security reports must follow [SECURITY.md](../SECURITY.md) and must not be posted publicly. See [CONTRIBUTING.md](../CONTRIBUTING.md) for the complete policy.

Yetlet is dual-licensed under your choice of [MIT](../LICENSE-MIT) or [Apache License 2.0](../LICENSE-APACHE). Workspace packages carry the SPDX expression `MIT OR Apache-2.0`; example and application packages remain `publish = false`. The license permits forks and independent modifications but does not require the upstream project to accept external code contributions.

</details>
