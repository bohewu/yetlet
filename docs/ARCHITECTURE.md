# Yetlet Architecture

**Status:** Experimental. **Published SDK:** `0.9.0-alpha.8` (`yetlet` / `yetlet-browser`).
**Published CLI:** `0.4.1` (`yetlet-cli`).
[Release readiness](RELEASE-READINESS.md) links the accepted publication records.
M0–M9 implementation, selected M10 qualification and five application-owned
research prototypes retain their recorded scopes. The typed Tag Builder is part of
the published alpha.7 SDK; private app Style tooling ships in CLI 0.3.0,
while Showcase remains a checkout sample.
Publication does not establish production or universal platform qualification.
**Published scope:** Native Rust server, HTML SSR, browser DOM/WASM, HTTP data APIs, optional islands.
**Related:** [MVP](MVP.md), [Roadmap](ROADMAP.md), [Backlog](BACKLOG.md), [ADRs](adr/0001-separation-of-concerns.md), [application startup contract — ADR 0011](adr/0011-application-startup-api.md), [Axum adapter prelude — ADR 0012](adr/0012-server-axum-prelude.md).

Current usage is documented in the [current module guides](API-REFERENCE.md), [HTTP pipeline](HTTP-PIPELINE.md) and [sample guide](SAMPLES.md). Shared owned Views feed escaped native HTML or explicitly built browser DOM/WASM. App/Page preserve native Axum/Tower handlers; local graph, keyed children, request correlation and Memory Session are explicit opt-ins. SQL/storage stay application-owned. No Controller, DI, plugin registry or ORM is required.

The design rationale and dated evidence below retain the original proposal and M7 decision history. Diagram references to compiler/macros and candidate IR are exploratory, not implemented authoring APIs. Current research decisions are in the [completed studies index](README.md#completed-research-prototypes); historical execution is not transferred to today's dirty source.

## Next-alpha target — YT-287 feature-first composition

The owner selects a small, composable Rust application framework with minimal
SDK publication: keep yetlet and yetlet-browser and expose optional capabilities
through modules/features. The [YT-287 contract](M28-HOST-RESOURCE-LIFECYCLE-DESIGN.md#15-yt-287--feature-first-architecture-and-driver-contract)
and [ADR 0014](adr/0014-layered-host-resource-lifecycle.md#yt-287-target-decision--minimal-packages-optional-capabilities)
define the independently reviewed target. Implementation is staged; the published
pair is unchanged. The built-in Core slice is described below.

| Layer | Next-alpha responsibility |
|---|---|
| Core in yetlet, default=[] | Built-in composition and generic HostGroup/Host/ResourceGroup; std-only lifetime coordination without mandatory UI, runtime, state service or serialization. |
| Optional View / HTML | Shared owned HTML/DOM values and pure serializer; no semantic dependency on lifecycle and no native scene-graph claim. |
| Optional Web | App, WebHost and native HTTP integration; Web API without HTML, SSR with web + html. Caller-runtime and owned-thread drivers use the same lifecycle rules. |
| Optional commands / state helpers | Neutral actions with executor-specific errors, target-neutral actions-http envelopes, local reactive graph and separately optional native command/snapshot bridge. |
| Optional native integrations | Winit, UDP, native wgpu and existing auxiliary services; feature combinations expose capabilities while runtime plans enforce ownership/affinity. |
| yetlet-browser | Existing separate browser adapters; DOM selects view and HTTP actions select actions-http, without enabling native Web. |

App means HTTP service configuration. HostGroup means coordinated Host
lifetimes. ResourceGroup is advanced resource ownership/control, not a DI
container. A Host does not imply a thread, process or renderer.

The consuming App::serve convenience retains ordinary cancellation semantics:
dropping its future is not graceful-completion evidence. Managed resources
require a retained owner, cancellation-safe driver storage and explicit
completion progress. View/HTML-only users still consume the Core-containing
package; feature isolation promises no unwanted adapter dependencies, not a
separate Core-free archive.

YT-287 is Done at contract scope. [YT-288 Core](M28-HOST-RESOURCE-LIFECYCLE-DESIGN.md#16-yt-288--built-in-core-host-lifetimes)
now provides built-in composition and the generic HostGroup/Host/ResourceGroup
surface, with an independent five-Host std-only consumer. Its borrowing poll
and async drive retain original owners through Pending; shutdown uses the same
Core engine. YT-288 is independently reviewed and Done. No runtime, pool, thread
or process is created by Core. [YT-289 feature isolation](M28-HOST-RESOURCE-LIFECYCLE-DESIGN.md#17-yt-289--optional-view-and-browser-feature-isolation)
now selects an empty default and gates View/HTML explicitly; Browser DOM and
keyed-DOM each enable View and their corresponding reexports. Actions/GPU-only
Browser selections remain independent. Focused verification and configured
independent review pass; YT-289 is Done. Published alpha.8 is unchanged.

[YT-290 Web/protocol ownership](M28-HOST-RESOURCE-LIFECYCLE-DESIGN.md#18-yt-290--optional-web-and-http-protocol-ownership)
now separates web from HTML and neutral actions from actions-http. App and the
relocated advanced HttpWorker share one Web-owned serving implementation;
active source consumers use the new module/feature names. Focused native tests,
isolated feature/target graphs and probes, migrated consumers and strict quality
checks and configured independent review pass; YT-290 is Done. The moved worker
retains its existing advanced lifecycle.

[YT-291 managed Web](M28-HOST-RESOURCE-LIFECYCLE-DESIGN.md#19-yt-291--managed-web-lifecycle-and-drivers)
connects App/WebHost to that same Core coordinator. One retained service
implementation supports caller-runtime and owned-thread drivers, with actual
initialization readiness, admission fencing, original completion and dependency
release. Core adds explicit pre-acquisition driver preparation and a std-only
blocking drive; no automatic pool or process policy is introduced.
[YT-292 native integration](M28-HOST-RESOURCE-LIFECYCLE-DESIGN.md#20-yt-292--optional-native-udp-and-custom-host-integration)
adds canonical command-bridge/UDP/Winit/renderer feature modules. A typed root
driver advances the same coordinator inside Winit callbacks through all final
release, with multiple windows under one NativeHost. UDP remains caller-driven;
Web may retain its owned-thread placement. No dependency or renderer is added
to default Core. Remaining consumer/CLI migration and normal archives retain
YT-293/294 gates in the
[ordered plan](M28-HOST-RESOURCE-LIFECYCLE-DESIGN.md#14-replanned-public-delivery--yt-287-through-yt-294).
The research history below retains its original default-feature/API assumptions.

## Proposed composable application research

The owner-directed [M28 study](M28-COMPOSABLE-APPLICATION-PLAN.md) explores
applications without HTTP while preserving pure Web/API/SSR as a first-class
mode. Historical HTML-first positioning does not constrain the study.
[ADR 0013](adr/0013-composable-application-and-hosts.md) has an accepted scoped
F2 amendment for the typed lifecycle core; broader high-level application
contracts remain separately proposed.

Host lifecycle, execution topology and Renderer selection are separate:
a host need not own a thread/process and may select zero/one/multiple compatible
renderers. Existing View projects to HTML or browser DOM; game SceneData has
separate backend paths. Browser is an environment, not itself a renderer.
One writer plus bounded commands/immutable snapshots is the shared-simulation
POC choice, not a mandatory global actor or HTTP request pipeline.

M28-B starts with a focused pure-Web preservation baseline (YT-276), then
same-thread multi-host composition and an explicit worker/lifecycle variation.
Existing Axum App/Router/Tower, default features, direct requests and Web CLI
bootstrap remain; no new supervisor, queue or GPU dependency is compulsory.
The initial M28-A plan made no measured zero-overhead claim and did not
implement GameHost, NetworkHost, NetworkClient, BrowserHost or generic
Application/HostGroup. That initial scope made no public positioning, Core API
or SDK/CLI release change.

The 2026-11-21 [layered Host design](M28-HOST-RESOURCE-LIFECYCLE-DESIGN.md) and
[ADR 0014](adr/0014-layered-host-resource-lifecycle.md) propose default Host
resource management with optional ResourceGroup customization over one evolved
lifecycle engine. At its reviewed checkpoint, the
[YT-286-A private prototype](../examples/host-lifecycle-research/README.md) proved
a compiling Application/Web/custom facade, deferred activity acquisition and
passive RAII retention through original joins using the unchanged core.

The subsequently authorized B1/B2 source proof adds opt-in Composition resources,
managed activities and structural groups. One root admission/local/original-join
barrier precedes final release. Reverse resource/group traversal retains earlier
dependencies on Pending or retryable progress failure; terminal release failure
is recorded once and does not repeat an already completed finalizer. Existing
Local close-before-worker semantics remain unchanged. The private facade adds
inert managed descriptors with deferred acquisition and cooperative release while
preserving supplied passive resources. This introduces no separate coordinator,
automatic executor, general dependency DAG or global Send/Sync/'static bound.

B1/B2 acceptance requires focused evidence and configured independent review.
The separately authorized B3 integration applies this contract to real SQLite
work behind Web handlers in the private research workspace. One SQLite worker
owns its Connection; a typed bounded client uses a shared command sender and
oneshot replies. The registered Web activity drains and joins before SQLite
final release closes that sender and joins the original database thread. A
failed explicit close retains its live Connection for retry. Pending represents
unfinished cleanup/observation and does not prove the Connection is still open.

The [Web/SQLite integration source](../examples/host-lifecycle-research/examples/web_with_sqlite.rs)
and [focused cases](../examples/host-lifecycle-research/tests/sqlite_web.rs) define
the bounded runtime acceptance surface. Actual test and independent review
receipts are required before claiming B3 acceptance. Its SQLite dependencies
are private dev-dependencies; the normal Application API and core/SDK feature
selection are unchanged. Native/shared drivers, full configuration overrides,
live managed-owner adoption, arbitrary graphs, pools/processes and public facade
promotion remain open under YT-286.

## Retained early Core and browser proof boundaries

The M12–M19 source-only verification history predates the published SDK/CLI.
Current Rust `xtask` build/check/smoke paths and the independent CLI replace
the original Python-driven toolchain, but do not claim that every historical
native Windows, browser, build context or deployment case is requalified.
See [developer commands](DEVELOPMENT.md), [CLI guide](CLI-GUIDE.md),
[packaging and assets](ASSETS.md) and [release readiness](RELEASE-READINESS.md).
The original local and Windows delivery evidence remains in Git history
at `b610acc7e098020b6b59dfa0d1ab2b75b059e656`, which is not yet verified reachable on public GitHub.

Early UI/form follow-ups retained handler-first and renderer ownership:
native form/CSRF/validation and explicit save feedback remain app-owned,
while local browser event state is independent from remote HTTP actions.
Server HTML and the finite DOM vocabulary remain separate; `hidden` has
a narrow shared-View SSR/DOM consistency rule but is not a general property
binding or live two-way form framework. Consult the [View guide](VIEW-COMPONENTS.md),
[HTTP pipeline](HTTP-PIPELINE.md), [bounded DOM contract](BROWSER-APP.md#historical-native-dom-controls-and-navigation-boundaries)
and [browser DOM guide](../crates/yetlet-browser/guides/dom.md). Research on
streaming SSR, custom action transport and resumability remains opt-in or
unpromoted; it does not make a general streaming/suspense, TCP transport,
or resumption API available in Core.

## Research-only handler resumption and native composition

**No general resumability API is part of Yetlet Core.** The isolated
[resumability example](../examples/resumability-research/README.md) demonstrated
two statically compiled, literal versioned handlers (`counter.add` and
`counter.subtract`) with an explicitly validated JSON envelope and numeric
captures. The local payload cap was 256 UTF-8 bytes, with unknown/duplicate
fields, unsupported IDs/versions and out-of-range values rejected. That
work demonstrated bounded serialization and native dispatch, **not**
capturing arbitrary Rust closures, resuming a stack, transporting events
remotely, faster hydration or an approved production browser contract.
UI event ownership stays local; remote behavior uses explicit typed
HTTP/actions under [ADR 0002](adr/0002-events-and-actions.md).

**Dependency composition uses ordinary Rust.** Construct services after
typed settings validation, pass concrete constructors or generic traits
by default, and use native Axum `State` or manual `FromRef<AppState>` for
a smaller handler substate. A shared `Arc` retains an instance; it does
not discover or instantiate a service. An application can select a
dyn-compatible interface such as `Arc<dyn SecretProvider>` when runtime
substitution is genuinely needed, but `impl Future` methods in the
existing signing backend traits are not interchangeable
`Arc<dyn SigningBackend>` interfaces. Explicit asynchronous startup,
secret ownership, request identity/CSRF, resource lifetime and shutdown
remain application concerns. The comparison of optional shaku/dill
containers introduced **no** `yetlet-di`, service locator, registration
system or measured performance advantage; see [extension composition](EXTENSIONS.md)
and [ADRs 0005/0006](adr/0005-opt-in-capability-modules.md).

These are conclusions from dated, independently reviewed research; the
unaltered original descriptors, numerical results and negative cases
remain in **local Git** `42d8856b9acf5ddddab9a4b998b5bdd4c5e59210`. The Git SHA has not been verified
publicly reachable, and the studies are not current Core implementation
or new platform qualification.
## Application-owned streaming and custom action transport

**Complete escaped SSR remains the default.** A bounded application-owned
Axum Body::from_stream prototype generated one escaped View row per poll.
Small-page first-body time was 0.257ms streamed versus 0.188ms full; a
10,000-row page exposed first body bytes at 3.011ms versus 6.372ms but
completed later at 12.451ms versus 6.855ms (full). A synthetic delayed
tail demonstrates scheduling, not natural throughput benefit. Once headers
are sent as 200, a later body render error cannot replace the status or
construct a complete error page. No HTTP/2, TLS, proxy flushing, browser
paint or generic Core streaming/suspense implementation was qualified.
See [HTTP policy](HTTP-PIPELINE.md#application-owned-deadlines-streaming-and-admission-timing)
and the [original fixture](../examples/streaming-ssr-research/README.md).

**Action transport remains application-owned.** A source-limited
ActionExecutor<Add> uses a four-byte big-endian length-prefixed TCP
frame (max 1024 payload bytes) over a native loopback socket.
Seven native tests support typed result/error mapping; the synthetic
ActionError::Remote status 422 is **not** an HTTP response.
This experiment adds no Core transport, authentication, CSRF, TLS,
browser, idempotency, retry or production-network guarantees.
See [extension ownership](EXTENSIONS.md#application-owned-action-transport).

## Weak-handle published-state snapshot and explicit first render

The initially proposed GraphHandle::snapshot() was **subsequently
implemented** as an additive experimental read and separately tested.
It temporarily upgrades Weak, checks **Disposed before Faulted**,
borrows the published state reference briefly and ends the RefCell
borrow before running application State::clone. It does not retain a
strong graph owner, render, flush queued commands, schedule callbacks
or replay observers. Reads in reducer/commit see the **previous**
published state, while observers read the newly published state;
queued re-entrant work may still be pending. A commit failure or
exhausted pass is Faulted, and disposed/dropped owners cause Disposed.
A cloned state can outlive the graph but is not a freshness or deep
immutability guarantee after the read.

Later independently selected **Todo Browser** and **Admin triage Save**
consumers replaced application mirror reads with this weak snapshot.
Browser Tasks was **not** migrated. The initial DOM render remains
explicit application setup, not a manufactured initial graph commit
or observer replay; Admin still validates pending/dirty state,
CSRF, item/version and runtime lifetime before Save. Linux/Wasm/Chrome
source-bound checks do not prove Windows/browser universal coverage
or a performance speedup. See the [reactive SDK guide]
(../crates/yetlet/guides/reactive.md) and [View guide](VIEW-COMPONENTS.md).

The complete dated method research, actual Chrome/browser verification,
original errors and full measurements remain in **local Git**
`c3478b7f4e0c929a94ccbf21e05217535553a73d`; remote reachability is not yet verified.
## Experimental scene, browser GPU and Physics Lab boundaries

The first isolated [scene research fixture](../examples/webgpu-scene-research/README.md)
used **native NVIDIA RTX 4090 / Vulkan**, not browser WebGPU.
wgpu 27.0.1 drew deterministic colored 10k/50k/100k static rectangles
into 1024×1024 RGBA with readback color-center checks. Thirty
sequential frames per size measured median **0.033760/0.054286/0.083525ms**
CPU command encoding, submission and GPU completion *wall* time.
These are not GPU timestamp queries, displayed FPS, DOM/CPU speedups,
VRAM accounting or browser support. Core/HTML/default SSR remain GPU-free.

A separate **Firefox 158, X11, Intel Vulkan WebGPU** matched
Canvas2D/Wasm comparison originally **FAILED** at screenshot-coordinate
alignment on one 50k Canvas2D case; after a bounded harness-only
correction, one full run completed **24/24 lanes, 2,160 timed samples
and 126 known-pixel screenshots**. Renderer completion-wait semantics
differ, and the study did **not demonstrate useful WebGPU speedup**,
general browser/OS/device support, 60 FPS or GPU timestamp duration.
See [bounded GPU qualification](QUALIFICATION-MATRIX.md#experimental-gpu-browser-and-physics-lab-evidence)
and the existing [GPU support guide](../crates/yetlet-browser/guides/wgpu-support.md).

An earlier **2026-10-26 Physics Lab feasibility study** only
reviewed wgpu 27 and Rapier3D 0.36.1 metadata; it built **no**
joint physics/Wasm/browser package at that checkpoint. The proposed
isolated educational app owned 1–64 dynamic cubes/spheres and five
static enclosure bodies, a fixed **1/60s physics step**, up to four
steps per callback and at most 100ms admitted wall delta, discarding
excess time rather than accumulating unbounded catch-up. An up-to-60
render submissions/s cap was **not** an FPS guarantee. Pause, single
step, reset, invalid-state freeze and CPU-owned state after GPU loss
required explicit generation-guarded retry/disposal; SSR semantic HTML
controls remain usable without a GPU. No Core 3D engine, persistent
replay, physical accuracy or cross-device determinism was selected.
The [later Physics Lab sample](../examples/physics-lab/README.md)
has separate implemented/qualified source; do not backdate it to
the feasibility-only research.

The complete original source reports, measurements and negative
results remain at **local Git** dfb27dc816c452e8033ff24e2d853f88a451761a; remote URL accessibility
has not been verified.
## Developer tooling boundary — YT-247

[ADR 0010](adr/0010-dynamic-sdk-version-resolution.md) and the
[CLI contract](CLI-GUIDE.md#sdk-resolution-yt-247) define an internal
CLI SDK resolver with separate metadata/cache, compatibility and scaffold
responsibilities. CLI build SDK and qualified embedded source-export snapshot
remain fixed release inputs; generated app SDK selection is a separate registry
consumer decision with exact pins and its own lock. The
[Phase B record](QUALIFICATION-MATRIX.md#historical-cli-contract-process-and-sdk-resolver-evidence) describes its
implementation and bounded verification, now distributed in CLI 0.3.0.
Earlier CLI releases retain their historical behavior. This changes none of the
framework execution/rendering/transport boundaries below.

## 1. Problem statement

Yetlet aims to reduce the conceptual and runtime costs of fat web frameworks. Static SSR pages should not require an entire client framework, and routine browser input/click events should not travel to a backend via a persistent event circuit. Rust component code may be shared between native server and browser targets, while renderers, interactivity policies, and action execution locations remain independently selectable.

This is **not** a proposal to rebuild a browser, HTTP transport stack, WebSocket protocol, or CSS layout engine.

### Invariants

- Native SSR must produce meaningful initial HTML; non-interactive pages require **zero browser JS/WASM**.
- Interactive behavior must be activated through explicit islands or a full browser application.
- The browser owns DOM events; remote commands are semantic operations, not event replication.
- Server and browser may share Rust types and component source, but compile into distinct build artifacts.
- HTTP request handlers may render HTML, serialize bounded state, and reference build-time assets. They must never compile JS/Rust/WASM per request.
- HTTP routing and API middleware integrate with a mature server ecosystem, initially Axum/Tower.

## 2. Separate four dimensions

| Dimension | Options | Coupled to other dimensions? |
|---|---|---|
| Execution | Native server / browser WASM | No |
| Render backend | HTML stream / browser DOM / future GPU scene | No, subject to target capabilities |
| Interactivity | None / island / full browser / future resume | No |
| Remote action | None / plain form / typed HTTP / future custom executor | No |

**Important:** `render(server)` does **not** imply `events(server)`. An SSR route can use native form POST, and a CSR component can operate entirely offline.

## 3. Logical architecture

```text
                       ┌────────────────────────────┐
                       │ Rust component declarations│
                       │ + local state + typed actions│
                       └──────────────┬─────────────┘
                                      │ compiler/macros (future)
                           Component description / IR
                             (shape under evaluation)
                                      │
                    ┌─────────────────┴──────────────────┐
                    │                                    │
             Native server target                  Browser WASM target
                    │                                    │
              HTML renderer                         DOM renderer
                    │                                    │
            markup + asset refs                 client mount / updates
                    │                                    │
                    └──────────┬─────────────────────────┘
                               │
                      Local semantic action
                               │
                      Action executor port
                        │               │
                   Local/state       HTTP/fetch
                                        │
                             Axum/Tower adapter
                               ├─ UI routes
                               └─ API routes
```

Renderers must never implicitly establish a remote session or circuit. Action transport is an optional adapter. API requests follow ordinary server middleware and authorization boundaries.

## 4. Request paths

### 4.1 API route

```text
HTTP request → router → auth/middleware → typed API handler
                                             → JSON/status/body
```

UI SSR and API handlers do not need to share a URL path. Authentication, authorization, input validation, and rate limits belong at the server trust boundary.

Production deployments place the application behind a trusted TLS edge and keep the native listener private. The edge strips caller-supplied forwarding headers and injects one canonical configured set; the application trusts them only from that edge. Authentication uses an external IdP, while opaque Session continuity, CSRF and resource authorization remain distinct server-side checks. Yetlet does not provide a mandatory proxy, IdP, limiter, secrets or authorization subsystem. See [Threat Model](THREAT-MODEL.md).

### 4.1.1 RequestContext and middleware (P0 extension)

The server adapter now exposes a request-only `RequestContext<S>` and native Axum/Tower composition. It owns the original request head/body, retains typed request Extensions, and keeps shared application state separate. `with_global` wraps the completed Router before application route matching; groups/routes use native nested routers and layers. API and SSR pass through the same HTTP middleware, without coupling browser actions to server requests.

Scope order, 404/405 semantics, error responses, body ownership and the safe SSR data boundary are specified in [ADR 0004](adr/0004-request-context-and-middleware.md). The [implementation note](HTTP-PIPELINE.md) includes tested integration points, costs and P1 limits. No browser runtime dependency or general DI/session subsystem is introduced.

### 4.2 Static SSR route

```text
GET / → router → loader (native capability) → component
    → HTML renderer (context-aware escaping) → response
```

Without interactive components, HTML and CSS are sufficient. SSR may call application services directly without localhost HTTP loopback.

### 4.3 SSR + island route

```text
GET /search → native SSR → complete HTML + escaped state + hashed assets
                                  │
                          tiny external JS loader
                                  │ on focus/input
                          lazy Search Island WASM
                                  │
                          local browser DOM events
```

The MVP uses progressive island activation/mounting, not full resumability. Only the island-owned subtree participates in client updates. The loader can be plain JavaScript rather than Wasm. Serialized state requires an explicit schema/version/component ID and may not contain credentials or server capabilities.

**M3 prototype evidence (2026-10-08):** `/search` uses a small external loader that imports Search WASM after a local focus/input event. Native SSR emits an explicitly allowlisted versioned JSON state, escaped as an attribute. Browser WASM validates the payload and updates only the results subtree. Invalid schema/version prevents activation while preserving the readable SSR page. This is **not** generalized component hydration or resumability.

### 4.4 Full browser route

```text
GET /app → shell HTML + compiled client artifacts
    → loader → browser WASM → DOM mount → explicit API if needed
```

Shared component source does not imply inclusion of server-side repositories or other privileged dependencies in browser artifacts. Browser data access is through capabilities that bind to explicit HTTP APIs.

**M7i production evidence (2026-10-09):** `/app` now preflights and binds its local Todo list through optional `yetlet-dom-keyed`, then drives typed `TodoCommand` values through one strongly owned `yetlet-reactive::LocalGraph`. Existing keyed rows are updated in place; the separate Create button remains an explicit `HttpExecutor` path. Chrome and Firefox verify identity, disposal, remount and no local request circuit. The Search Island does not acquire either dependency. See [M7i verification](QUALIFICATION-MATRIX.md#keyed-renderer-and-lifecycle-evidence).

## 5. Implemented and proposed module boundaries

| Candidate module | Responsibility | Timing |
|---|---|---|
| `yetlet-core` | Renderer-neutral component/View and state primitives | M1 |
| `yetlet-reactive` | Optional explicitly owned synchronous local update graph | M7g |
| `yetlet-html` | Deterministic HTML serializer and escaping | M1 |
| `yetlet-server-axum` | HTTP/UI/API routing integration, headers | M1 |
| `yetlet-session` | Optional server-side session identity, memory storage, expiry and CSRF token | M8c |
| `yetlet-dom` | Browser DOM mount and updates | M2 |
| `yetlet-dom-keyed` | Optional keyed child reconciliation and listener ownership | M7h |
| `examples/todo-browser` | Browser WASM bootstrap; production keyed local Todo path | M2 / M7i |
| `examples/search-island` | Search-specific lazy island runtime | M3 |
| `yetlet-actions` | Typed commands and pluggable executor boundaries | M4 |
| `yetlet-macros` | Compile-time `ui!` DSL and template extraction | Future spike |
| `yetlet-gpu` (concept only) | Independent scene renderer; no supported crate selected | Bounded app-owned [YT-023 study](ARCHITECTURE.md#experimental-scene-browser-gpu-and-physics-lab-boundaries) complete |

These are conceptual boundaries, **not** a promise that each one will become a public crate. Use the smallest validated interfaces across the two renderer targets before adding extra abstractions or splitting packages.

### Optional application capabilities (M6 composition; M8c Session proof)

Server features such as sessions, authentication, data caching, output caching, request tracing and rate limiting are **modular, opt-in, and pay-for-what-you-use**. Integrate them through independent Rust crates/providers, native Tower/Axum layers, typed request extensions and explicit application state. Do not add an always-on DI container or plugin manager. Application cache and HTTP output cache have different safety constraints; cookie-based authentication need not require a server session.

`App::configure` and `try_configure` now compose ordinary typed setup functions once, while providers reuse native Tower scopes, explicit state and Extensions. The independent `yetlet-request-id` crate is the M6 opt-in proof: the Todo app installs `RequestIds::new().global(app)`, and a scoped provider can be configured only on selected groups. The core has no dependency on this provider. `RequestContext` itself still generates no ID. No server extension or provider dependency is copied into Browser WASM/hydration. See [Extensions](EXTENSIONS.md), [ADR 0005](adr/0005-opt-in-capability-modules.md) and [M6 evidence](QUALIFICATION-MATRIX.md#early-ssr-and-handler-measurements).

M8c adds the independent `yetlet-session` crate as a second, security-bounded proof. The cart explicitly installs `SessionManager<Cart>` only on its `/cart` group. IDs and CSRF tokens are OS-random, state and expiry remain server-side, unknown IDs are not adopted, mutations use a session-bound token, and clear/end demonstrate rotation and revocation. The server adapter, Todo, REST and Browser packages do not depend on Session. This is an anonymous process-local provider, not authentication or production qualification. See [M8c evidence](M8C-VERIFICATION.md).

M8d's measured three-sample audit retains these boundaries rather than introducing another runtime API: Todo pays for Browser/keyed/reactive capabilities, REST remains a native one-package API, and Cart remains a native one-package SSR application selecting Session. Shared candidates such as listener helpers, JSON errors, origin policy and forms remain application-specific until more examples converge. M8e freezes the complete artifact/dependency regression and final reviewer PASS without changing these boundaries. See [M8d evidence](SAMPLES.md#dx-audit-and-final-freeze) and [M8e evidence](QUALIFICATION-MATRIX.md#sample-and-api-freeze-evidence).

### Handler-first application API (M6 implemented)

**Handler-first, Component-first, MVC optional, minimal boilerplate.** MVC is a code organization convention, not an application requirement. A simple SSR page or typed JSON endpoint uses ordinary Rust handler functions and shared `View`/components, with no Controller superclass, mandatory folder structure, custom model binder, broad `ActionResult` hierarchy or service locator.

M6a now implements `yetlet-server-axum::App<S>`, a thin facade over `axum::Router<S>`. `get`, `post`, `route`, `group`, `route_layer`, `with_state`, `map_router` and conversion to/from native Router preserve Axum/Tower handler, extractor, middleware, 404/405 and response semantics. `RequestContext<S>` remains optional. [M6a verification](QUALIFICATION-MATRIX.md#early-ssr-and-handler-measurements) records the native integration tests and reproducible commands.

M6 now adds `App::page(path, title, handler)` for no-argument async View functions and the small `Page` HTTP result for handlers with native typed extractors or `Result<Page, E>` errors. Json, Redirect and status/header results use native Axum semantics. The Todo server now uses App, Page, native State/Json and explicit provider registration. There is still no Controller runtime, discovery macro, custom model binder or full MVC framework. See [ADR 0006](adr/0006-handler-first-mvc-optional.md).

## 6. Components, templates, and state

### Shared component source

```rust
// Proposed DSL only; not a released Yetlet API.
#[component]
fn TodoList(items: &[Todo]) -> View {
    ui! { <ul>{ for item in items { <li>{item.title}</li> } }</ul> }
}
```

The native target outputs HTML; the browser target creates and updates DOM. Component boundaries and stable identifiers must be deterministic to avoid hydration mismatches.

**M7a measured evidence:** An isolated release benchmark compared current subtree replacement, a positional `View` patch, keyed direct DOM and runtime-cloned static slots over 200 rows × 500 updates × 10 rounds. The retained `2026-10-09T02:02:22+00:00` report recorded median round totals of 213.500, 41.700, 2.000 and 1.900 ms respectively on one local Chrome 155 environment. Whole replacement lost owned input/keyed-row identity; the positional patch preserved the input but not keyed identity after reorder; keyed/slot prototypes preserved both. Production Browser/Search assets remained byte-identical. See [M7a verification](QUALIFICATION-MATRIX.md#keyed-renderer-and-lifecycle-evidence).

**M7b prototype evidence:** The isolated `yetlet-keyed-bindings` crate expresses one fixed keyed-list shape with native SSR, allowlisted typed text/attributes, one conditional slot and one local typed row action. Browser binding preflights component/instance/key/slot/shape identity before attaching listeners, preserves existing SSR nodes, reconciles keyed insert/remove/reorder, and supports explicit disposal. Production Browser/Search assets and dependency graphs remain unchanged. See [M7b verification](QUALIFICATION-MATRIX.md#keyed-renderer-and-lifecycle-evidence).

**M7c prototype evidence:** The same isolated crate now contains one explicitly owned synchronous local graph. Strong ownership stays in `LocalGraph`; dispatch handles and subscriptions are weak. FIFO batches reduce into one draft and one commit, re-entrant messages run in a later bounded pass, reducer errors preserve the previous commit, renderer errors fault the graph, and disposal releases observers/listeners/captures. Chrome verifies the graph driving the M7b binding without HTTP or WebSocket traffic. See [M7c verification](QUALIFICATION-MATRIX.md#keyed-renderer-and-lifecycle-evidence).

**M7d prototype evidence:** The fixed row now optionally owns one controlled text input. Model-aware live-value updates avoid resetting caret selection; successful keyed reconciliation captures/restores a connected focused input and selection, while focused row removal deliberately leaves the detached node unfocused. Chrome 155 and Firefox 157 verify input identity/value/focus/selection across reorder plus listener cleanup after row removal and owner disposal. See [M7d verification](QUALIFICATION-MATRIX.md#keyed-renderer-and-lifecycle-evidence).

**M7e evidence freeze:** The complete native/wasm32/Chrome/Firefox regression, M7a benchmark rerun, production asset comparison and dependency isolation pass without changing production Browser/Search bytes. The evidence identifies small identity, slot, keyed reconciliation, ownership/disposal and weak-handle patterns as candidates for later extraction, but promotes none of the current APIs. See [M7e verification](QUALIFICATION-MATRIX.md#keyed-renderer-and-lifecycle-evidence).

**M7f promoted contract:** `yetlet-core::binding` now owns renderer-neutral `ComponentIdentity`, `BindingKey`, `SlotName`, `InertAttributeName` and `SlotDescriptor` types. The fixed M7b authoring prototype consumes these contracts, eliminating duplicate validation while remaining experimental. This promotion selects only the shared trust-boundary vocabulary; it does not select a template IR, DOM reconciler, scheduler or hydration protocol. See [M7f verification](QUALIFICATION-MATRIX.md#keyed-renderer-and-lifecycle-evidence).

**M7g optional graph:** The exact synchronous graph proven in M7c now lives in dependency-free `yetlet-reactive`. It retains one strong owner, weak dispatch/subscription handles, FIFO batching, bounded later-pass re-entrancy, recoverable reducer errors, faulted commit errors and deterministic disposal. It is an explicit local message/commit primitive, not field-level signals, automatic effects or an async scheduler. See [M7g verification](M7G-VERIFICATION.md).

**M7h optional keyed DOM:** `yetlet-dom-keyed` now owns validated keyed order, element-plus-state entries, insert/remove/reorder reconciliation and disposable event-listener ownership. The M7b–M7d fixture consumes the primitive and retains its Chrome/Firefox identity, controlled-input and cleanup evidence. Keeping it independent from `yetlet-dom` is required for pre-adoption production asset isolation; attempts to place or feature-gate it inside the existing renderer changed compiled Browser/Search bytes. The update/DOM mutation failure boundary remains explicit and non-transactional. See [M7h verification](QUALIFICATION-MATRIX.md#keyed-renderer-and-lifecycle-evidence).

**M7i production adoption:** Todo Browser now binds its local list to the promoted keyed owner and uses a graph-backed adapter at the existing `ActionExecutor<TodoCommand>` boundary. Route runtime ownership is explicit; disposal invalidates the runtime lifetime before dropping local/remote listeners, so queued actions cannot start and in-flight HTTP completions cannot update retired DOM. Readable DOM remains, and remount creates exactly one fresh action path. The remote result list and Search results retain their prior bounded subtree strategy. Separate Cargo builds prevent Todo's expanded browser features from perturbing Search assets. See [M7i verification](QUALIFICATION-MATRIX.md#keyed-renderer-and-lifecycle-evidence).

**M7j architecture freeze:** [ADR 0007](adr/0007-bounded-keyed-dom-and-local-graph.md) accepts the renderer-neutral binding vocabulary, optional explicitly owned local graph, separate optional keyed DOM owner and one production Todo local-list adoption. It retains direct subtree mounting for simpler/independent surfaces and explicitly rejects inferring a general VDOM, template DSL, signals/forms framework or resumability design from the benchmark. The first final reviewer found one queued/in-flight disposal race; explicit lifetime invalidation plus Chrome/Firefox debug/release regressions repair it. Replacement validation and repeat reviewer Job `wc_job_XZzdXEOIx6owj1o5` PASS are recorded in [M7j verification](QUALIFICATION-MATRIX.md#keyed-renderer-and-lifecycle-evidence).

M7a–M7e made the keyed/static-slot/local-ownership direction concrete and ended in a no-decision freeze. M7f–M7h promote only independently proven low-level contracts; M7i adopts them for one production Todo local-list path; M7j freezes the bounded decision, repaired runtime lifetime, complete regression evidence and repeat reviewer PASS. The fixed authoring shape, current text-input API, focus policy and conditional-slot shape remain experimental. No public VDOM, dependency-tracked signals/forms runtime, macro DSL or general template compiler is selected.

### M9 compatibility classification

M9's twelve independently reviewed applications provide enough evidence to classify—but not stabilize—the actual public Rust surface. [Public API Contract Classification](API-CONTRACTS.md) records renderer-neutral `View`/binding values, escaped HTML, handler-first `App`/`RequestContext`/Tower composition and bounded Request ID/Session application APIs as **Stable Candidates**. The complete-document `Page`/language/head contract and browser Actions/DOM/keyed/reactive surfaces remain **Experimental**. Adapter constructors, direct provider middleware and generated implementation details remain **Internal/support-only**.

This classification does not change crate versions or establish SemVer compatibility. M9j froze 190 tests, native/browser/provider/dependency and exact-asset evidence; after reviewer-discovered inventory/reexport corrections, repeat reviewer `wc_job_QkQkovlBlvrhPxCE` accepted the classification. Licensing, historical MSRV, browser/support policy, the root document metadata design, external CI and production security expectations remain release blockers.

### M10c minimal document ownership

The native server adapter owns one bounded complete-document value: escaped title plus a validated root language. `Page::new`, `App::page` and the original render functions retain the `zh-Hant` default; explicit-language handlers use `DocumentMetadata` and `Page::with_metadata`. Charset and viewport remain fixed correctness defaults, while CSP remains response policy rather than page metadata.

This resolves the M9c nested-language workaround without introducing a head tree, SEO collection, raw head markup, scripts/styles, per-page CSP, DI or another rendering framework. The conservative `LanguageTag` shape is serialization-safe but not a full BCP 47 registry implementation. See [Compatibility Policy](COMPATIBILITY.md) and [M10c verification](COMPATIBILITY.md#m10-metadata-compatibility-evidence).

### Data-access capabilities

```rust
// Conceptual interface; the browser target cannot depend on server repositories.
trait TodoSource {
    async fn list(&self) -> Result<Vec<Todo>, TodoError>;
}
```

A native implementation can call domain/repository services directly. The browser implementation uses explicit HTTP JSON endpoints. Do not hide arbitrary RPC generation inside a compiler macro.

### State transfer

Native HTML may carry serializable initial state, subject to all of the following:

- Explicit component and instance identity, schema version, and asset manifest version.
- Context-sensitive HTML/attribute/JSON escaping; avoid `</script>`, ampersand, or Unicode separator injection.
- Payload size limits, browser-side version validation, and a deterministic safe fallback.
- Never serialize server request context, access tokens, passwords, database connections, or privileged closure captures.

M3 demonstrates these constraints for the Search Island's bounded v1 payload. A generalized state protocol remains outside the current proof.

## 7. Events and commands

```text
pointer/keyboard/input DOM event
          │ local browser listener
          ▼
  component-local state transition
          │ when remote work is needed
          ▼
 typed command / explicit HTTP form
          │
          ├─ LocalExecutor / LocalGraph adapter
          ├─ HttpExecutor
          └─ Future CustomExecutor (explicit opt-in)
```

The MVP does not forward raw DOM events. Remote actions need explicit input/output/error schemas; the server must validate permissions again. HTTP mutations must address CSRF, idempotency, replay, and retries. Keep future custom transports separate from renderer interfaces.

**M4 prototype evidence (2026-10-08):** `yetlet-actions` exposes `Command` and injected `ActionExecutor<C>` implementations: `LocalExecutor` runs a shared domain transition; `HttpExecutor` makes an explicit JSON POST to `/api/todos`. `examples/todo-ui` owns `TodoCommand`/`TodoSnapshot`; the server uses the same transition behind an Axum adapter with a per-router in-memory store. Browser E2E confirmed local toggle with no HTTP and a separate semantic Create POST. Typed errors are surfaced without executing untrusted HTML. See [M4 verification](QUALIFICATION-MATRIX.md#early-ssr-and-handler-measurements).

**M7i local adapter evidence (2026-10-09):** Todo Browser's local executor now dispatches the same typed command through a weak graph handle and returns only the last successfully committed snapshot. Graph ownership, rendering and listener lifetime remain local to the route runtime; `HttpExecutor` remains the only remote path. This changes local scheduling/DOM ownership, not the command or transport contract.

**Security boundary:** This prototype is anonymous and listens directly over HTTP. Content-type/Origin restrictions and JSON body limits are demonstrated, but they do not implement user authentication, credentialed CSRF protection, or HTTPS reverse-proxy identity. These remain requirements for any real deployment.

## 8. Security, accessibility, and operations

- Strict CSP as the default: no inline `on*` handlers and no `eval`. Bootstrap scripts are static external assets.
- Context-correct SSR escaping. Treat hydration metadata and client state as hostile.
- Prefer native semantic HTML with labels, keyboard focus, IME, and built-in scrolling.
- Browser builds cannot import server-only secrets/capabilities; invalid target imports should fail at compile time.
- Instrument by mode: SSR render time, HTML bytes, compressed WASM payload, mount time, and request count.
- Distinct error semantics: server renderer errors produce a status code; client actions expose typed errors; failed islands leave usable SSR content.
- Keep CSP, XSS, and dependency checks alongside performance work.

## 9. GPU and parallelism boundary (post-MVP)

The first-party [`yetlet-browser::wgpu` module](../crates/yetlet-browser/guides/wgpu.md) is explicitly **Experimental and opt-in**; it is a bounded flat rectangle renderer, not a DOM/CSS accelerator. Its [support boundary](../crates/yetlet-browser/guides/wgpu-support.md) preserves the selected Firefox/WebGPU, no-device/default browser fallback, real/controlled failure and logical disposal limits. The [separate Physics Lab sample](../examples/physics-lab/README.md) uses app-owned Rapier CPU physics and a private 3D pipeline; it is not a shared framework 3D API or supported simulation engine. The [initial native Vulkan study](ARCHITECTURE.md#experimental-scene-browser-gpu-and-physics-lab-boundaries), [Firefox Canvas2D comparison](QUALIFICATION-MATRIX.md#experimental-gpu-browser-and-physics-lab-evidence) and [cross-crate API audit](API-CONTRACTS.md#cross-crate-api-differences-and-scoped-gpu-refinement) remain distinct original authorities. None justifies production FPS/SLO, hardware portability, WebGPU default enablement or silent lifecycle ownership.

WebGPU cannot accelerate ordinary DOM/CSS layout directly. A future WebGPU target must be a separate **Canvas/scene renderer**, which carries its own scene graph, focus, IME, text, and accessibility responsibilities. Workers may move computation away from the main thread, but DOM commits still occur on the main thread. The completed [YT-023 study](ARCHITECTURE.md#experimental-scene-browser-gpu-and-physics-lab-boundaries) retains native Vulkan draw/readback, completion-wall-time and explicit allocation-byte observations. Browser WebGPU, broad scene/text/accessibility support and matched DOM/CPU speedups remain unverified; no supported core renderer is selected. GPU development is not an MVP gate.

## 10. Risks and experiments

| Risk | Initial test / mitigation |
|---|---|
| Rust UI macro limits and compile complexity | Prove shared View plus dual renderer first; decide on IR/`ui!` later |
| SSR/client state mismatch | Deterministic IDs and SSR/hydration snapshot assertions |
| WASM bundle size and startup | Repaired M7i/M7j records a +82,726 raw / +20,042 local-gzip Browser runtime delta; retain the final baseline and split optional islands where useful |
| Over-abstracted DOM updates | M7i adopts low-level keyed ownership for one Todo list only; remote/Search subtree paths and fixed authoring remain independent |
| Server-only dependencies in browser | Target-specific compilation and browser CI |
| Remote commands accidentally becoming a circuit | Semantic actions only, no default WebSocket |
| Exploding combinations of modes | MVP exposes only three tested route presets |
| GPU renderer not matching HTML semantics | Keep GPU outside MVP and provide a separate scene interface |

See [ADR 0001](adr/0001-separation-of-concerns.md), [ADR 0002](adr/0002-events-and-actions.md), [ADR 0003](adr/0003-renderers-and-loading.md), and [ADR 0007](adr/0007-bounded-keyed-dom-and-local-graph.md).

## SSR, browser and handler architecture boundaries

The M2–M9 prototypes established separable HTML serialization, DOM ownership,
asset loading, local updates and typed HTTP commands. Their original package
names describe the historical layout; use the [current API map](API-REFERENCE.md)
for the published two-package SDK. Neither the historical package count nor a
prototype PASS is a current support or stability promise.

Native SSR returns complete escaped HTML with no mandatory JS/WASM. Browser
assets are compiled and content-addressed at build time. Missing or invalid
island assets leave readable SSR; no code is evaluated or compiled per request.
The original DOM renderer used `createElement`/`createTextNode`, never `innerHTML`.
The Search proof binds only its named/versioned boundary, preserves the input
and initial results, and updates the result subtree/count. Its bounded,
allowlisted `SearchStateV1` rejects unknown fields, duplicate IDs, excess size,
item limits and schema mismatch before binding. It filters server-supplied
initial data; this is not a general hydration registry or resumability.

M6's handler-first facade owns only the native Axum Router. `get`/`post`, nested
`group`, `route`, state and middleware delegate to Axum; `map_router`,
`from_router` and `into_router` retain native escape hatches. MVC/Controllers are
optional application organization. There is no second router, binder, result
class hierarchy, runtime discovery or DI container. `App::page` accepts a
no-argument async View handler; parameterized/fallible handlers use native
extractors and `Page`/`Result<Page, E>`. Rendering failures return a generic 500
with strict CSP; application errors keep their selected response semantics.
[HTTP composition](HTTP-PIPELINE.md#handler-and-typed-action-boundaries) and
[explicit installation](EXTENSIONS.md#construction-and-optional-provider-ownership)
retain the security and ownership details.

The M7e freeze selected no renderer. Later M7f–M7j promoted only safe identity/slot
metadata, an optional synchronous owned graph and optional keyed child/listener
ownership. Todo's local list selects both; remote Todo and Search results retain
owned-subtree replacement. The fixed template/input/focus authoring fixture stays
experimental. [Browser ownership](BROWSER-APP.md#keyed-ownership-and-runtime-retirement)
and [qualification](QUALIFICATION-MATRIX.md#early-ssr-and-handler-measurements)
record the measured costs and failed alternatives. General VDOM, signals/effects,
macros, forms, async scheduling, full resumability and WebGPU-for-DOM were not
adopted. GPU scenes remain an independent optional renderer.

M8's three-shape DX audit and M9's twelve-sample freeze required no new runtime
abstraction. SQL, transactions, synthetic identity, uploads and SSE stay
application-owned; **Yetlet supplies no ORM**. A reusable data API would require
separately reviewed convergent consumers and an ADR. An external ORM may be an
explicit app choice, never a default SDK/browser dependency. See
[sample contracts and dated acceptance](SAMPLES.md#sample-acceptance-and-source-boundaries).

## M11 product and renderer decisions

The source-grounded YT-069 study at c7ecc49eeb667e145cfb7e764c10b72ba104bbee
selected two independent lines under [ADR0008](adr/0008-product-readiness-and-opt-in-scene-renderer.md).
Product completeness does not depend on GPU; renderer qualification does not
establish product readiness or publication. Planning used owner-permitted
**current-session self-review**, not an independent agent/security assurance.
The original three priorities were G1 safe form/live-property semantics,
G2 reusable CSS/document integration and actual theme/CSP proof, and G3 complete
persisted, authorized product failure journeys. Existing CSS and escaping were
already present; no generic hydration/forms/head/UI-kit framework was justified.

The first product was a noncritical Lab Operations Board: persisted work items,
Viewer/Operator and project authorization, SQL history/CAS, validation/PRG,
retained conflict drafts, meaningful status and authoritative refresh. A small
CMS, simulated shop and structured knowledge workspace were alternatives, not
additional activated products. Remote machine control, school/customer data,
payments, arbitrary HTML/editors, collaborative CRDT/OT, ORM and production HA
were excluded. Framework correctness, application domain/auth/SQL/content policy
and operator TLS/IdP/secrets/backup/rollout remain different responsibilities.

On2026-10-14 the owner explicitly approved the bounded first-party
**Experimental opt-in flat-scene renderer despite no demonstrated WebGPU
speedup**. This superseded the original useful-speedup extraction prerequisite
only for that candidate; it did not amend the native Vulkan or matched browser
measurements. Each YT-078/079/080 item still required its own independent PASS.
At extraction the crate stayed publish=false; later package distribution is
separately recorded. Official maintenance did not mean default activation,
stable API, universal browser/HA/production support or FPS guarantees.

One canvas/lifetime owner, independent prebuilt island assets, async generation
guards, bounded dimensions/DPR/allocation, one coalesced RAF, hidden/zero-size
suspension, explicit bounded loss retry and deterministic disposal are the
renderer design boundaries. Validate finite data/IDs/size arithmetic before GPU
work; trusted build-time WGSL is not user-supplied shader text or per-request
Wasm compilation. HTML owns labels/controls/IME/selection detail and a useful
paginated/searchable data alternative; GPU failure must not disable siblings or
essential actions. No default Core/View GPU variant, CSS accelerator, game loop,
3D engine, arbitrary shader plugin, ECS, text shaper or general scene graph was
selected. A general scene abstraction needs a materially different second
consumer; another instance of the same demo is insufficient.

Browser-first proof required one actual selected browser/OS/adapter plus
unavailable/loss fallback. A latest-wgpu upgrade, all native backends, integrated
GPU or Safari/mobile support was never automatically required or proved. Fair
Canvas2D/WebGPU comparisons keep equivalent seed/coverage/viewport/operations,
startup/upload/preparation/completion/presentation separate, raw sample counts
and omitted lanes explicit. GPU completion is not paint/FPS; correctness
readback is outside timing. Pointer coordinates map through CSS/DPR/transforms
to stable IDs, with stale-result rejection, keyboard/touch/reset alternatives
and reduced motion. Small selection state does not justify cloning100k mutable
marks or changing LocalGraph into a global async scheduler.

The exact original M11 plan/research/GPU proposal, self-review and owner decision
are recoverable at local Git `40b66f1ad9bd0eb5666a9e1c8f113310993a3544`.
Remote SHA reachability is unverified. Current API and actual browser evidence
follow [API classification](API-CONTRACTS.md#cross-crate-api-differences-and-scoped-gpu-refinement)
and [GPU qualification](QUALIFICATION-MATRIX.md#experimental-gpu-browser-and-physics-lab-evidence),
not conceptual constructor names or proposed60Hz targets in the original plan.
