# Current API reference

Updated: 2026-11-12. This map distinguishes the published Experimental two-package
SDK (`0.9.0-alpha.8`) from independent CLI tooling and checkout samples. `yetlet` contains the shared View and opt-in
native capabilities; `yetlet-browser` contains browser integrations and depends
one-way on the same View. Both packages are available from crates.io at this version.
Earlier release tags retain their original layouts and evidence.

Use guides for behavior and rustdoc from the same revision for signatures.
[The migration contract](COMPATIBILITY.md#historical-two-package-pre-publication-source--2026-10-30) records the complete
source break, feature names and preserved boundaries.

| Capability | First published version | Current usage and original release evidence |
|---|---|---|
| Broader DOM coverage and per-mount link/image policies | SDK `0.9.0-alpha.6` | [DOM guide](../crates/yetlet-browser/guides/dom.md#explicit-links-and-images); [SDK publication and registry consumer](ALPHA6-RELEASE-PREPARATION.md#owner-approved-publication-and-registry-consumer--2026-11-01) |
| Static HTML export with `yetlet build --static` | Independent CLI `0.2.0` | [Static build guide](CLI-GUIDE.md#static-html-build); [CLI publication and installed consumer](CLI-020-RELEASE-PREPARATION.md#owner-approved-publication-and-installed-consumer--2026-11-02) |

The DOM APIs include `DomOptions`, `NavigationPolicy`, `ImagePolicy`,
`validate_with_options` and Wasm `mount_with_options`. Native validation is
provisional; actual mounting enforces browser URL policy. Static export is CLI
orchestration around an application-owned exporter, not an SDK static-site module.
Source/browser qualification and later registry-consumer checks retain their
separate scopes in the linked records. The [Reader Edition](../site/pages.toml)
selects its SDK/CLI pair independently of these first-release versions.

For first authoring, follow [Views/components](VIEW-COMPONENTS.md). For package
features, startup wiring and consumer settings/defaults in one place, use the
[opt-in quick reference](OPT-IN-QUICK-REFERENCE.md); the tables below remain the
API-carrier map, with full contracts in their linked guides.

For source loading, startup validation, typed options and application-owned
diagnostics together, follow [Settings, options and logging](SETTINGS-OPTIONS-LOGGING.md).

## Published typed Tag Builder — SDK alpha.7

The SDK has included `yetlet::view::tags` since `0.9.0-alpha.7`, with **101 typed
ordinary tag constructors**, element-scoped attribute setters and the existing
owned `View`.
See the [typed View guide](VIEW-COMPONENTS.md#typed-tag-builders),
[actual constructors](../crates/yetlet/src/view/tags.rs) and
[dated HTML inventory](API-CONTRACTS.md#historical-html-element-and-attribute-inventory) for the 101 constructor
selection, nine typed void tags and renderer admission boundaries.
The registry-first `Element::new` tutorial remains valid and intentionally uses
the reviewed Reader Edition selected in [the site manifest](../site/pages.toml).
[SDK alpha.7 publication and registry consumer](ALPHA7-HOSTED-RECORD.md#owner-approved-publication-and-registry-consumer--2026-11-09)


## Published application startup — SDK alpha.8

### Application startup and optional Prelude — YT-258

**Simple first, customizable when needed.** These startup methods and the
optional native `server_axum::prelude` ship in SDK `0.9.0-alpha.8`.
Select `server-axum` and declare the application's Tokio runtime explicitly.

Published CLI `0.4.0` generates this simple startup for Native and Native
Style with verified SDK alpha.8 and compatible options. The creation receipt
records `startup_profile` as `m26-app-serve`. SDK alpha.7 still uses the legacy
native listener. [CLI startup profiles](CLI-GUIDE.md#sdk-resolution-yt-247)

```rust
use yetlet::server_axum::prelude::*;

async fn home() -> View {
    View::text("Hello from Yetlet")
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    App::new().page("/", "Yetlet", home).serve().await
}
```

`serve()` consumes a resolved `App<()>` and binds plain HTTP at
`127.0.0.1:7878`. Address-resolution and bind failures propagate as I/O errors.
Axum handles post-bind accept errors internally. The application owns its Tokio
runtime. The default is fixed, with no environment lookup or port fallback.
For an existing resolved-state Axum Router and an explicit nondefault address,
use the following inside the application's async entry point.

```rust
let router: axum::Router = App::new().page("/", "Yetlet", home).into_router();
let address = "127.0.0.1:8080";
App::from_router(router).serve_on(address).await?;
```

The prelude is optional. Explicit imports such as `yetlet::View` and
`yetlet::server_axum::App` remain supported. Native `into_router()`,
`tokio::net::TcpListener` and `axum::serve(...).with_graceful_shutdown(...)`
remain first-class for listener ownership, actual ephemeral-port reporting and
shutdown coordination. The capability samples and Dashboard demonstrate this
native lifecycle.

Both convenience methods accept TCP addresses and provide HTTP only. TLS and
HTTPS require an explicitly configured secure edge or native TLS integration.
They never infer TLS from ports, hostnames or environment, provision certificates,
or create a hidden runtime. Binding all interfaces with `0.0.0.0` or `::`
explicitly exposes the listener beyond loopback.

## Source-only checkout additions

The current checkout includes built-in yetlet::composition and the root
HostGroup, Host and ResourceGroup exports; no composition feature is required.
The [independent Core consumer](../examples/external-consumers/yt288-core-hosts/src/main.rs)
demonstrates custom Hosts, typed owned/borrowed resources and caller-driven
polling without normal runtime dependencies. [YT-288](M28-HOST-RESOURCE-LIFECYCLE-DESIGN.md#16-yt-288--built-in-core-host-lifetimes)
records the Core contract and bounded verification. [YT-289](M28-HOST-RESOURCE-LIFECYCLE-DESIGN.md#17-yt-289--optional-view-and-browser-feature-isolation)
sets the checkout default to []; select view for View/Element/Attribute, typed
tags and binding, or html for serialization (which enables view). Browser dom
and dom-keyed select view and expose the same types; default-off actions/GPU
selections do not. Published alpha.8 retains its original feature contract.

[YT-290](M28-HOST-RESOURCE-LIFECYCLE-DESIGN.md#18-yt-290--optional-web-and-http-protocol-ownership) moves native HTTP to `yetlet::web`. Select `web` for
routing, middleware and HTTP serving; select `web` plus `html` for `Page`,
document metadata and View rendering. Web alone does not select View or HTML.
`App` configures an HTTP service and serves on the caller's Tokio runtime.
[YT-291](M28-HOST-RESOURCE-LIFECYCLE-DESIGN.md#19-yt-291--managed-web-lifecycle-and-drivers)
adds `App::into_host()` / `WebHost::new(router)` for retained HostGroup lifetimes.
Fresh async driving uses caller-owned Tokio; synchronous `run()` uses an owned
Web service thread plus its dedicated join observer. Explicit `WebDriver`
selection is optional. Multiple Hosts,
API-only readiness, admission/drain and managed final release use the same Core
coordinator. See the [Web lifecycle guide](../crates/yetlet/guides/server-axum.md#execution-ownership)
and [finite consumer](../examples/external-consumers/web-hosts/README.md).
The existing `HttpWorker` retains its advanced low-level contract.

[YT-292](M28-HOST-RESOURCE-LIFECYCLE-DESIGN.md#20-yt-292--optional-native-udp-and-custom-host-integration) adds explicit native `command-bridge`, `udp`, `native-winit` and
`renderer-wgpu` features. Their matching modules contain the canonical bridge,
`UdpHost`/`UdpEndpoint`, `NativeHost`/`WindowSlot` and independent renderer.
One NativeHost uses `.with_window(...)` for multiple windows under the same
caller-thread loop. Native/UDP ordinary hosting uses synchronous `run()`;
`run_async()` rejects them before acquisition. Advanced custom resources may
supply a typed `BlockingDriver` through `ResourceGroup::with_blocking_driver`.
See the [finite native consumer](../examples/external-consumers/native-hosts/README.md)
for Native/UDP and optional Web with borrowed local resources. These APIs remain
checkout-only; feature availability is not renderer/backend qualification.

The `actions` feature keeps `Command`, `ActionExecutor` and `LocalExecutor`
transport-neutral. Executors choose an associated `Error`; local execution
accepts the caller's error type without requiring serialization. Select
`actions-http` for `yetlet::actions_http::{ActionError, ApiErrorEnvelope, ApiErrorBody}`.
Browser `actions` forwards this HTTP contract and retains its Wasm HTTP executor
without selecting native Web. The checkout removes the old `server-axum`
feature and `server_axum` module without aliases; see the
[checkout migration](COMPATIBILITY.md#current-checkout--web-and-http-protocol-ownership).
Published alpha.8 retains the startup API shown above.

The source CLI now recognizes the selected SDK's feature-module vocabulary.
Native HTTP/SSR starters choose web+html and import yetlet::web; Browser starters
choose view+reactive, with DOM/keyed-DOM in yetlet-browser. Settings and Vite add
config. An incomplete new profile is rejected rather than using legacy imports.
Older SDK metadata retains its established templates and startup capability rules.
See [CLI source profiles](../tools/yetlet-cli/README.md#current-source-sdk-feature-profiles)
and the [migration checkpoint](M28-HOST-RESOURCE-LIFECYCLE-DESIGN.md#21-yt-293--cli-consumers-and-minimal-sdk-feature-selection).
This source support does not select a new CLI/SDK release or change the pinned
reader's published recipes.

The native-first Style Showcase is a checkout sample with an optional Wasm
companion. Both packages are built from the repository.
[Style Showcase](../examples/style-showcase/README.md)

## CLI Style authoring and checkout Showcase

Published SDKalpha.8/CLI0.4.1 artifacts serve separate roles. The Reader
Edition uses the pair selected in the site manifest.
Application-owned Rust CSS first shipped in CLI `0.3.0`. The CLI ships Native
and Browser Style templates with a private build-time generator, build/watch
integration and exact SDK resolution and locking. By default, `new` selects
the highest available non-yanked SemVer and checks the selected SDK for the
template's required packages and features.
[CLI Style release and platform evidence](CLI-GUIDE.md#style-release-and-platform-evidence)

| Source surface | Entry point and contract | Availability and limits |
|---|---|---|
| Optional Style authoring | [CLI Style guide](CLI-GUIDE.md#optional-application-owned-rust-css-yt-239-working-tree), [retained registration contract](CLI-CONTRACT.md#application-owned-rust-style-registration-and-css-publication) | Published CLI templates and private app generator, finite typed vocabulary and explicit registration. Build-time external CSS; ordinary CSS handles unsupported syntax. No public style crate, runtime style injection or renderer change |
| Style Showcase | [Native-first gallery](../examples/style-showcase/README.md) | Unpublished checkout sample with actual View/Style/CSS disclosures and optional local Wasm class switching. Bounded Linux Chrome/Firefox sample evidence |

## HTML, HTTP and browser modules

This map describes the current checkout and its explicit feature selections.
For the published alpha.8 startup contract, use the section above and the
[first-app guide](FIRST-APP.md). Renderer validation and native Axum/Tower
semantics remain separate responsibilities.

| Checkout module and guide | Main entry points | Selection, ownership and failure boundary |
|---|---|---|
| [`yetlet::view`](../crates/yetlet/guides/core.md) | `View`, `Element`, `binding` validated vocabulary | Owned renderer-neutral values; construction does not sanitize HTML. Binding validation can return `BindingContractError`; the renderer owns output policy |
| [`yetlet::html`](../crates/yetlet/guides/html.md) | `render_to_string`, `escape_text`, `RenderError` | Borrow a View, return an owned escaped fragment or policy error. Complete buffering; no raw HTML escape hatch or HTTP-header ownership |
| [`yetlet::web`](../crates/yetlet/guides/server-axum.md) | `App`, `RequestContext`, `middleware`, advanced `HttpWorker`; with `html`: `Page`, `DocumentMetadata`, `LanguageTag` | `web` selects native routing/extractors/State/Extensions and caller-runtime serving. `web` plus `html` adds View rendering; rendering failures map to generic HTTP 500. Metadata and strict CSP retain their bounds |
| [`yetlet_browser::dom`](../crates/yetlet-browser/guides/dom.md) | `validate`, `validate_with_options`, `DomOptions`, `NavigationPolicy`, `ImagePolicy`; WASM `mount`, `mount_with_options` | Optional browser subtree replacement, finite DOM policy with explicit per-mount options. Static validation returns `DomPolicyError`; browser failures use `JsValue`. No general identity preservation; mount can reject navigation that static validation accepted |
| [`yetlet_browser::dom_keyed`](../crates/yetlet-browser/guides/dom-keyed.md) | `validate_keyed_order`; WASM `KeyedChildren`, `KeyedEntry`, `DomEventListener` | Explicit keyed/listener lifetime; duplicate-key preflight and browser failures. Callbacks and DOM failures do not provide transactional rollback or a generic sanitizer |
| [`yetlet::reactive`](../crates/yetlet/guides/reactive.md) | `LocalGraph`, `GraphHandle`, `Subscription`, `ReactiveError` | Synchronous thread-local reducer/commit ownership; weak handles and explicit disposal. Snapshot clones published state; faults and queued re-entrant work retain their documented semantics |
| [`yetlet::actions`](../crates/yetlet/guides/actions.md) | `Command`, `ActionExecutor`, `LocalExecutor` | `actions` selects transport-neutral execution with an associated executor `Error`; local commands and errors require no serialization contract |
| [`yetlet::actions_http`](../crates/yetlet/guides/actions.md) | `ActionError`, `ApiErrorEnvelope`, `ApiErrorBody` | `actions-http` selects the existing HTTP error variants and serialized envelope shape; it enables neutral Actions without selecting native Web or HTML |
| [`yetlet_browser::actions`](../crates/yetlet/guides/actions.md) | `HttpExecutor`; shared neutral traits and HTTP error/envelope reexports | Browser `actions` forwards `yetlet/actions-http`. HTTP execution is Wasm-only; native construction does not provide native fetch. No implicit Auth/CSRF, retries or remote DOM-event circuit |
| [`yetlet::request_id`](../crates/yetlet/guides/request-id.md) | `RequestIds`, `RequestId`, `HEADER`, `InvalidStart` | Explicit scoped/global installation. Clones share a bounded numeric counter; independent providers/restarts do not. Generated IDs replace untrusted inbound values; overflow refuses with 503, not distributed trace identity |
| [`yetlet::session`](../crates/yetlet/guides/session.md) | `SessionConfig`, `SessionManager`, `Session`, typed Session errors | Explicit bounded Memory store, synchronous operations, cookie/CSRF and rotation/revocation. Process-local state disappears on restart; anonymous Session is not authentication or durable business storage |

## Additional opt-in modules

Each native capability below is a feature of `yetlet`; WebGPU belongs to
`yetlet-browser`. Selecting a feature compiles an API, while startup installation
and application-owned configuration remain explicit. Native modules are absent
on wasm32 even in a broad feature union.

| Package and current guide/source | Main entry points | Selection, ownership and failure boundary |
|---|---|---|
| [`yetlet::auth`](../crates/yetlet/guides/auth.md) | `Principal`, `Authentication`, `authenticate`, `Authenticated`, resolver types | Native route-scoped identity resolution; required versus optional extraction, missing-layer refusal and responses follow the guide. Verified identity does not grant business authorization. Optional `auth-session` explicitly bridges Memory Session only |
| [`yetlet::rate_limit`](../crates/yetlet/guides/rate-limit.md) | `Policy`, `Budget`, `Key`, `RateLimitLayer`, `InvalidRateLimitInput` | Native per-process memory admission. Policy reuse shares configuration; Budget/layer clones share accounting. Trusted keys, bounded capacity and rejection semantics are explicit; no distributed quota |
| [`yetlet::config`](../crates/yetlet/guides/config.md) / [full guide](CONFIGURATION.md) | `Loader`, `Environment`, `Error::Key`; optional `SecretBinding` | Opt-in startup loading with finite precedence. `load_with_env` applies a supplied snapshot after ordinary sources; optional `secret-provider` resolves selected protected fields. No ambient Settings service, live reload or browser secrets |
| [`yetlet::secrets`](../crates/yetlet/guides/secrets.md) | `SecretProvider`, `SecretFuture`, `SecretError`, `SettingsSecrets` | Injected native async lookup: missing `None` differs from access/unavailable errors. Settings adapter owns an explicitly replaceable snapshot; no automatic rotation, registry or browser dependency |
| [`yetlet::signing`](../crates/yetlet/guides/signing.md) / [source](../crates/yetlet/src/signing/mod.rs) | `SigningBackend`, `PublicKeySource`, `PublicKeySet`, `Policy`, `Verifier`, `SigningService` | Explicit exact-byte operations, versioned keys and caller-owned trust. Public material membership does not establish trust; local OpenSSL support does not activate release signing or imply general PKI validation |
| [`yetlet::session_redis`](../crates/yetlet/guides/session-redis.md) | `RedisSessionConfig`, `RedisSessionManager`, `RedisSession`, typed operation/errors | Separately selected async provider; configured namespace, limits, lifecycle and uncertain-outcome handling remain explicit. Optional native `session-redis-tls` selects checked `rediss://`; Redis names the protocol API and Valkey the selected reference server. No generic HA/failover/restore promise |
| [`yetlet_browser::wgpu`](../crates/yetlet-browser/guides/wgpu.md) / [support](../crates/yetlet-browser/guides/wgpu-support.md) | Bounded scene/batch/lifetime and picking APIs | Separate optional flat-scene renderer. CPU-only defaults; explicit `wgpu-browser` plus WASM target selects WebGPU. Scene/device/lifecycle errors remain explicit; no DOM acceleration, stable API, 3D or performance guarantee |

Native Auth, configuration, secrets, signing and remote Session capabilities
must not enter browser builds or serialized hydration state. Package selection
and Cargo features determine compiled availability; application startup decides
whether an available capability is installed for an instance. Workspace-wide
builds are not the default dependency graph of a small SSR application.

## Application options and tools

| Existing consumer | Current integration API | Contract boundary |
|---|---|---|
| [Board](../examples/lab-operations-board/README.md) / [Admin](../examples/admin-dashboard/README.md) | `BoardOptions`; `Board::open_with_options` and `open_with_oidc_and_options` | Independent default-false `write_authorization` / `request_tracing`, plus frontend/Session inputs. Uncompiled selections fail before storage; existing simpler constructors stay off. Admin maps its existing typed Settings; Board loads no ambient settings or global subscriber |
| [Shop](../examples/modular-shop/README.md) | `DatabaseOptions`; `Database::open_with_options` | Default-false `catalog_cache`; the binary maps its existing environment key. Display freshness is bounded; Session, authorization and checkout SQL stay authoritative. No mandatory Settings loader or framework cache crate |
| [CLI](CLI-GUIDE.md#cli-command-and-capability-inventory) | Existing parser, commands, static HTML export and configured generators | Independently published `yetlet-cli`, executable `yetlet`; the resolver selects a compatible registry SDK and preserves exact app pins/locks. Settings, supported source layouts and platform limits belong to its current guide. Generators do not silently install or deploy services |

These options are public Rust interfaces of example applications, not methods
on a framework-wide capability builder. Their public fields do not grant trust
or remove the existing fresh-v2 write-authority/reopen restrictions. Feature
names and external settings keys remain as documented in each current guide.
The [naming guide](NAMING.md#application-opt-in-options--yt-185) records migration
from the former capability-specific constructors without aliases.

OIDC, opaque Bearer checks, native tracing and catalog caching have existing
application-owned integrations. Audit history and output-cache studies do not
imply `yetlet-audit`, `yetlet-output-cache` or other extracted packages. Do not
infer an implemented API from a research item marked Done.

## Reading Rust API documentation

Start with [the first-app walkthrough](FIRST-APP.md) for a working application
and [Views/components](VIEW-COMPONENTS.md) for UI authoring. Use rustdoc when you
need an exact function, type or error. It is the API reference generated from
Rust source comments, also shown by Rust-aware editor hover/help. The
[Rust documentation guide](https://doc.rust-lang.org/rustdoc/how-to-write-documentation.html)
explains `///` comments on an item and `//!` comments on its containing module
or crate. Ordinary `//` comments are implementation notes, not API pages.

| What you see | How to read it in Yetlet |
|---|---|
| Crate and module | A crate is a compiled Rust library/package target; a module groups APIs. Cargo package `yetlet-browser` is imported as `yetlet_browser` |
| Struct / enum | A struct represents a value such as `Element`; an enum lists alternatives such as `View::Text` and `View::Element` |
| `Type::new(...)` / `value.method(...)` | An associated constructor versus an operation on a value; open the item's page for its contract |
| `&View`, `self`, `&self` | A shared borrow, consuming the value, or borrowing the receiver. A builder taking `self` returns the value used for the next chained call |
| `impl Into<String>` | An accepted conversion, for example from a string literal or owned `String`; you do not need a custom type |
| `Result<T, E>` | Success returns `T`; failure returns `E`. Read error variants and the integration guide before deciding how the application responds |
| `async fn` | Produces a future; ordinary asynchronous application code awaits it. Async does not select a provider or install middleware |
| Feature / target gate | The item exists only for the documented Cargo features and compilation target. Feature selection and runtime installation are separate |

For example, [`Element::child`](../crates/yetlet/src/view/mod.rs) takes the
current element and a value convertible into `View`, then returns an element
for chaining. Constructing a View does not validate or escape HTML; rendering
owns that policy. Follow the [View guide](VIEW-COMPONENTS.md) for complete usage.
An API missing from one generated page can be feature-gated: check the selected
features/target before assuming it was removed. For a published dependency,
read its version's docs; checkout rustdoc can describe later source changes.

### Current doc-comment coverage and gaps

YT-217 improves the selected entry points below in the current working source.
Four new examples and selected native/WASM rustdoc builds PASS. This is a focused
inventory, not a complete public-item coverage measurement or new browser/runtime
qualification. Published archives retain their original documentation:

| Inspected entry | Current documentation | Remaining boundary |
|---|---|---|
| [`yetlet` crate root](../crates/yetlet/src/lib.rs) and [`yetlet-browser` crate root](../crates/yetlet-browser/src/lib.rs) | First example, feature/target map, startup ownership and guide links | Other feature-specific examples remain in their existing guides; no all-API coverage claim |
| [`web::App`](../crates/yetlet/src/web/app.rs) | Existing module context, application example, state/ownership explanation and method comments | Unchanged in YT-217; native Axum semantics remain authoritative. YT-290 supplies the current module path |
| [`View` / `Element`](../crates/yetlet/src/view/mod.rs) | Variant/field docs, builder/accessor semantics, an owned-tree example and renderer trust boundaries | Constructors still perform no sanitization or renderer validation |
| [`session::SessionConfig`](../crates/yetlet/src/session/mod.rs) | Constructor/example/error-variant docs, exact TTL/capacity/name validation, Secure default and guide link | No provider installation, authenticated identity, cookie-prefix validation or remote-provider qualification |

The selected work follows [YT-217](https://github.com/bohewu/yetlet/blob/c0a40af80a48932f3cdf7c593f36fe563e4ec669/docs/PUBLIC-DOCS-PLAN.md#implementation-order--yt-217);
other modules retain their existing documentation and can be improved when needed.
Guides should explain how to integrate a capability; doc comments should explain
the callable API's behavior, ownership, errors and applicable security limits.
Add a runnable example where it clarifies usage. Use `Errors`, `Panics` or
`Safety` sections only when applicable; document existing behavior rather than
adding guarantees. The YT-217 source changes contain doc comments only, with unchanged executable
Rust tokens and public API.

## Local rustdoc

From the repository root, select features and target explicitly:

```sh
cargo doc --offline --locked --no-deps -p yetlet --features web,html,actions-http,auth-session,rate-limit,config,secret-provider
cargo doc --offline --locked --no-deps -p yetlet-browser --target wasm32-unknown-unknown --all-features
```

The native entry page is `target/doc/yetlet/index.html`; browser output is under
`target/wasm32-unknown-unknown/doc/yetlet_browser/index.html`. Signing separately
requires its documented native OpenSSL dependency. Local rustdoc instructions
are not fresh platform qualification or a docs.rs publication claim.

## Historical evidence and current API ownership

M6's thin App/Page and M7's metadata, optional keyed ownership and local graph
established the handler/component boundaries used by the current SDK. Follow the
module guides above for current signatures; original multi-crate names in the
[historical API classification](API-CONTRACTS.md) describe a dated inventory.
Stable Candidate is an evidence category, not a stability or SemVer guarantee.
The [sample acceptance guide](SAMPLES.md#catalog-contract)
and [qualification matrix](QUALIFICATION-MATRIX.md#coverage-and-evidence-boundaries)
retain the original failed reviews, repaired scope and native/browser freeze.
No sample introduces a Yetlet ORM, mandatory Controller/DI, generic forms,
implicit browser circuit or production-security promise.
