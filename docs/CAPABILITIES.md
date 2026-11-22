# Current capability and API carrier map

Updated: 2026-11-12. The published Experimental SDK has two implementation packages:
`yetlet` and `yetlet-browser`, version `0.9.0-alpha.8`, available from crates.io.
For those published packages, base defaults select HTML and Browser defaults
select DOM. Native capabilities and GPU/action integrations are additive opt-ins
with explicit startup wiring. The checkout map below describes capabilities,
not additional installable crates.
Earlier sixteen-package releases and qualification records keep their original
sources. Opt-in static HTML export through the [independent CLI](CLI-GUIDE.md#static-html-build)
is distributed in [CLI `0.2.0`](CLI-020-RELEASE-PREPARATION.md#owner-approved-publication-and-installed-consumer--2026-11-02);
it adds no SDK module/default. SDKalpha.6 includes the YT-220/221 DOM vocabulary
and options, absent from SDKalpha.5. Earlier records keep their original
source, API names and scope. See [the migration](COMPATIBILITY.md#two-package-feature-and-target-contract).

Start at [Current API reference](API-REFERENCE.md) for current entry points,
ownership/error boundaries and local rustdoc. [Documentation scope](DOCUMENTATION-SCOPE.md)
classifies reader inputs and retained evidence; this inventory does not select
publication of its linked execution records.

SDK alpha.8 includes [simple application startup and the optional Axum Prelude](API-REFERENCE.md#published-application-startup--sdk-alpha8)
with the opt-in `server-axum` feature. Published CLI `0.4.0` generates
`App::serve()` for Native and Native Style with verified SDK alpha.8 and
compatible options. It records the selected startup profile in the scaffold
receipt. [CLI startup profiles](CLI-GUIDE.md#sdk-resolution-yt-247)

## Published typed authoring and CLI Style tooling

SDK alpha.7 **includes** `yetlet::view::tags` with 101 typed ordinary tag
constructors, the same owned View and existing renderer restrictions. See the
[typed View guide](VIEW-COMPONENTS.md#typed-tag-builders)
and the [SDK alpha.7 release record](ALPHA7-HOSTED-RECORD.md#owner-approved-publication-and-registry-consumer--2026-11-09).
The original YT-245 preparation selects local SDK `0.9.0-alpha.7` contents;
these typed constructors are now distributed rather than checkout-only.


Published SDKalpha.8/CLI0.4.1 artifacts serve separate roles. The independent CLI
ships native/browser Style templates and a private app generator. The Style
walkthroughs verify SDK alpha.7 with exact app pins and locks. The
[Style Showcase](../examples/style-showcase/README.md) remains a checkout sample.
See the [API carrier map](API-REFERENCE.md#source-only-checkout-additions)
and [Style contract](CLI-CONTRACT.md#application-owned-rust-style-registration-and-css-publication).
Style authoring has a finite private app vocabulary, explicit registration and
external build-time CSS; ordinary CSS remains the default and supplies unsupported
syntax. There is no public style crate, renderer change or runtime style injection.

The current checkout additionally provides built-in Core composition with
HostGroup, custom Host extensions and ResourceGroup. YT-288 removes the
composition feature opt-in. [YT-289](M28-HOST-RESOURCE-LIFECYCLE-DESIGN.md#17-yt-289--optional-view-and-browser-feature-isolation)
selects `default = []`, optional `view` and `html -> view`; Browser DOM/keyed
features select View and gate its reexports. Default-off actions/GPU selections
do not add View, HTML or native Web. See the [Core lifecycle contract](M28-HOST-RESOURCE-LIFECYCLE-DESIGN.md#16-yt-288--built-in-core-host-lifetimes)
and [independent consumer](../examples/external-consumers/yt288-core-hosts/src/main.rs).

[YT-290](M28-HOST-RESOURCE-LIFECYCLE-DESIGN.md#18-yt-290--optional-web-and-http-protocol-ownership) supplies native HTTP through `web`, with `html` additionally
required for Page/document/View rendering. Neutral `actions` has an associated
executor `Error`; `actions-http` separately owns `ActionError` and the serialized
API error envelopes. Browser `actions` forwards `yetlet/actions-http` without
selecting native Web. There is no checkout `server-axum` feature or
`server_axum` compatibility module.

`App` is HTTP service configuration and direct serving uses caller-owned Tokio.
[YT-291](M28-HOST-RESOURCE-LIFECYCLE-DESIGN.md#19-yt-291--managed-web-lifecycle-and-drivers)
adds managed `WebHost` / `App::into_host`, caller-runtime and owned-thread drivers,
API-only readiness and cancellation recovery through the retained HostGroup.
Custom ResourceGroup management uses the same coordinator. The existing
`HttpWorker` retains its advanced low-level contract. These source changes do
not change published alpha.8 package contents or its startup examples.

[YT-292](M28-HOST-RESOURCE-LIFECYCLE-DESIGN.md#20-yt-292--optional-native-udp-and-custom-host-integration)
adds canonical optional native modules without changing the empty default.
NativeHost contributes one caller-thread root loop for multiple windows; UDP
remains bounded and caller-driven. Web, renderer and command bridge selections
are independent. The unpublished companion only forwards remaining historical
consumers; normal package qualification stays a later gate.

## Framework packages and selected features

| Checkout carrier | Implemented scope and maturity | Original nine release candidates? | Authoritative contract |
|---|---|---|---|
| `yetlet::view` | Renderer-neutral Views/components and bounded binding vocabulary; M9 classification applies | Yes | [Core guide](../crates/yetlet/guides/core.md) |
| `yetlet::html` | Escaped HTML serialization and allowlisted elements/attributes; M9 classification applies | Yes | [HTML guide](../crates/yetlet/guides/html.md) |
| `yetlet_browser::dom` | Native validation/Wasm-only owned mount. Since SDKalpha.6:89 default tags,90 with explicit ImagePolicy; SDKalpha.5 has38. Independent per-mount HTTPS link/image policies retain strict defaults and same-origin forms. Not full HTML conformance | Yes | [DOM guide](../crates/yetlet-browser/guides/dom.md), [dated coverage inventory](API-CONTRACTS.md#historical-html-element-and-attribute-inventory) |
| `yetlet_browser::dom_keyed` | Optional keyed children and disposable listener/entry ownership | Yes | [Keyed DOM guide](../crates/yetlet-browser/guides/dom-keyed.md) |
| `yetlet::reactive` | Optional synchronous local reducer/commit graph with explicit lifetime | Yes | [Reactive guide](../crates/yetlet/guides/reactive.md) |
| `yetlet::actions` | `actions`: transport-neutral Command/ActionExecutor/LocalExecutor; the executor selects its associated Error | Yes | [Actions guide](../crates/yetlet/guides/actions.md) |
| `yetlet::actions_http` | `actions-http`: HTTP ActionError and serialized API error envelopes; Browser `actions` forwards this contract | No; separated from the former Actions carrier | [Actions guide](../crates/yetlet/guides/actions.md), [YT-290](M28-HOST-RESOURCE-LIFECYCLE-DESIGN.md#18-yt-290--optional-web-and-http-protocol-ownership) |
| `yetlet::web` | `web`: native App, managed WebHost, request context, Axum/Tower routing and advanced HttpWorker; add `html` for Page/document/View rendering | Yes; formerly `server_axum` | [Web guide](../crates/yetlet/guides/server-axum.md) |
| `yetlet::command_bridge` | `command-bridge`: native bounded command admission and immutable snapshots; independent of HTTP and ActionExecutor | No | [Native integration contract](M28-HOST-RESOURCE-LIFECYCLE-DESIGN.md#20-yt-292--optional-native-udp-and-custom-host-integration) |
| `yetlet::udp` | `udp`: managed UdpHost and explicit low-level UdpEndpoint; std-only bounded nonblocking progress | No | [Native consumer](../examples/external-consumers/native-hosts/README.md) |
| `yetlet::native_winit` | `native-winit`: one caller-thread NativeHost loop with multiple windows; no renderer or Web runtime | No | [Native integration contract](M28-HOST-RESOURCE-LIFECYCLE-DESIGN.md#20-yt-292--optional-native-udp-and-custom-host-integration) |
| `yetlet::renderer_wgpu` | `renderer-wgpu`: independently selected native Vulkan Surface renderer; backend qualification stays separate | No | [Native integration contract](M28-HOST-RESOURCE-LIFECYCLE-DESIGN.md#20-yt-292--optional-native-udp-and-custom-host-integration) |
| `yetlet::request_id` | Explicit scoped/global process-local numeric correlation | Yes | [Request ID guide](../crates/yetlet/guides/request-id.md) |
| `yetlet::session` | Opt-in bounded Memory Session, anonymous state, expiry, CSRF, cookie lifecycle, rotation/revocation; synchronous API | Yes | [Session guide](../crates/yetlet/guides/session.md) |
| `yetlet::session_redis` | Opt-in native module in the published SDK. Original qualification is single-host, single-primary, no persistence; later bounded TLS/restart evidence is listed separately below | No | [Provider guide](../crates/yetlet/guides/session-redis.md), [Valkey evidence](PROVIDER-OPERATIONS.md#redis-session-cas-operation-outcome-and-cookie-ordering) |
| `yetlet/session-redis-tls` | Default-off native Rustls/Ring transport for checked `rediss://`; unchanged Session/Lua/limits, no insecure fragments. YT-184 bounded evidence complete from generation4 Auth plus generation5 authority recovery/persistent restart/new-epoch re-login; transient503/failures retained, all five groups deleted. Done at bounded isolated Linux reference scope after final independent Reviewer PASS0; no general HA/restore/platform claim | No | [Provider TLS guide](../crates/yetlet/guides/session-redis.md#optional-native-tls--isolated-cross-host-reference), [YT-184](PRODUCTION-DEPLOYMENT-PROFILES.md#historical-shared-authority-and-azure-persistence-qualification) |
| `yetlet::config` | Opt-in native module providing typed startup loader and finite source precedence; optional `secret-provider` integration | No | [Configuration](CONFIGURATION.md), [settings/secret integration](SETTINGS-SECRET-PROVIDER.md) |
| `yetlet::secrets` | Opt-in native async secret lookup and explicit settings adapter; later local rotation/Azure evidence remains separately bounded | No | [Secret providers](SECRET-PROVIDERS.md), [rotation evidence](SECRET-ROTATION-VERIFICATION.md) |
| `yetlet::signing` | Opt-in native injected signer/public-key source and explicit trust/version policy; local OpenSSL-backed sample/CLI consumers | No | [Signing contract](SECRET-PROVIDERS.md#local-signing-and-replaceable-provider-contract--yt-093), [verification](QUALIFICATION-MATRIX.md#local-signing-key-cli-and-azure-evidence--2026-10-18) |
| `yetlet::auth` | Opt-in native identity foundation/resolver/extractors in the published SDK. YT-114 Done after independent review; business authorization remains application-owned | No | [Auth guide](../crates/yetlet/guides/auth.md), [contract](PROVIDER-OPERATIONS.md#local-auth-permissions-and-multi-instance-authority) |
| `yetlet/auth-session` | Explicit optional feature bridging the existing Memory Session to application identity mapping. YT-115 Done after independent review; not a generic Redis bridge or complete auth product | No | [Memory bridge](EXTENSIONS.md#historical-opt-in-auth-session-and-rate-limiter-selection) |
| `yetlet::rate_limit` | Native opt-in Tower middleware in the published SDK, endpoint/group Policy and explicitly shared Budget. YT-117 Done at per-process memory scope; configured example defaults off | No | [Rate-limit guide](../crates/yetlet/guides/rate-limit.md), [contract/evidence](EXTENSIONS.md#historical-opt-in-auth-session-and-rate-limiter-selection) |
| `yetlet_browser::wgpu` | Optional flat-scene renderer in the published browser SDK, CPU-only defaults and explicit wasm32 browser feature; selected configuration only | No | [GPU guide](../crates/yetlet-browser/guides/wgpu.md), [support](../crates/yetlet-browser/guides/wgpu-support.md) |

Root workspace membership is a development/build choice, not automatic
installation in an application's dependency graph or inclusion in the release
candidate. There is no mandatory Settings/Auth/Session/Cache/Tracing service,
DI container or provider registry. Server-only capability dependencies and
trusted request extensions stay out of browser builds and hydration state.

## Application-owned integrations and tools

| Carrier | Current status and boundary | Authority |
|---|---|---|
| Board/Admin OIDC | Existing application-owned maintained-client integration and explicit issuer/subject to local-account mapping. YT-116 research Done; no extracted generic OIDC adapter | [OIDC study](https://github.com/bohewu/yetlet/blob/4ad0978f8a70b36c307f5e538aa1a3aa496a7403/docs/YT-116-OIDC-ADAPTER-STUDY.md), [accepted provider evidence](../examples/admin-dashboard/README.md#historical-auth-session-oidc-and-local-policy-evidence) |
| YT-125 Valkey/Auth consumer | Isolated application-owned async Auth resolver and two-process qualification; fresh shared local authority, explicit lifecycle/failure semantics | [YT-125](../examples/admin-dashboard/README.md#historical-auth-session-oidc-and-local-policy-evidence) |
| Admin Valkey/Auth adoption | YT-126 Done after focused verification and independent review; explicit feature/runtime selection, fresh policy and documented pre-SQL race limit. No generic crate bridge or cross-store commit fencing | [YT-126](../examples/admin-dashboard/README.md#historical-auth-session-oidc-and-local-policy-evidence), [contract study](PROVIDER-OPERATIONS.md#local-auth-permissions-and-multi-instance-authority) |
| Shop optional catalog display cache | YT-183 Done after independent Reviewer PASS, zero issues/followups: app-only default-off `catalog-cache`, bounded age/admission and generation invalidation; Session/checkout remain authoritative, no framework cache crate or output cache | [Contract](../examples/modular-shop/README.md#optional-catalog-display-cache--yt-183), [implementation/new proof](../examples/modular-shop/README.md#optional-catalog-display-cache--yt-183) |
| Opaque Bearer capabilities | Existing File Server/Production Starter app-owned route guards; no generic identity-aware JWT/introspection provider | [Bearer study](https://github.com/bohewu/yetlet/blob/4ad0978f8a70b36c307f5e538aa1a3aa496a7403/docs/YT-119-BEARER-STUDY.md) |
| Yetlet CLI | Independently published `yetlet-cli`, executable `yetlet`, with typed parser/support API, explicit scaffolding, static HTML export and configuration-only generators. Scaffolding pins the SDK selected by the resolver exactly and writes the app lock and receipt. Per-command and platform limits apply. | [Current CLI guide](CLI-GUIDE.md#current-command-and-capability-inventory) |

Application accounts, permissions, project policy, sensitive-write rechecks and
deployment durability/HA remain application responsibilities. Authentication
success does not grant a resource/action. Historical multi-instance proof does
not qualify every later source revision or a production deployment.

## Research decisions and unselected candidates

| Item | Current result | Runtime selection |
|---|---|---|
| YT-116 OIDC adapter | Research Done after independent Reviewer PASS; reuse existing app composition | No generic adapter selected |
| YT-119 Bearer | Research Done after independent Reviewer PASS; retain app-owned opaque capabilities | No Bearer/JWT provider or Auth feature selected |
| YT-120 Cookie primitives | Research Done after independent Reviewer PASS; reuse existing Session providers and mature Rust cookie primitives | No `yetlet-cookies`, ambient jar or duplicate CSRF/store subsystem selected |
| YT-118 Audit | [Research](https://github.com/bohewu/yetlet/blob/4ad0978f8a70b36c307f5e538aa1a3aa496a7403/docs/YT-118-AUDIT-STUDY.md) Done after independent Reviewer PASS; preserve app-owned history/event policy | No `yetlet-audit` implementation |
| YT-121 Data cache | [Research](https://github.com/bohewu/yetlet/blob/4ad0978f8a70b36c307f5e538aa1a3aa496a7403/docs/YT-121-CACHE-STUDY.md) Done after independent Reviewer PASS; current reads/snapshots remain app-owned | No `yetlet-cache` implementation |
| YT-122 Output cache | [Research](https://github.com/bohewu/yetlet/blob/4ad0978f8a70b36c307f5e538aa1a3aa496a7403/docs/YT-122-OUTPUT-CACHE-STUDY.md) Done after independent Reviewer PASS; retain app-owned header/static policy | No `yetlet-output-cache` implementation; private output remains uncached by default |
| YT-123 Tracing | [Research](https://github.com/bohewu/yetlet/blob/4ad0978f8a70b36c307f5e538aa1a3aa496a7403/docs/YT-123-TRACING-CORRELATION-STUDY.md) Done after independent Reviewer PASS; retain optional RequestIds and app-owned native composition | No `yetlet-tracing` implementation; request IDs alone are not distributed tracing |

See the [M21 plan](EXTENSIONS.md#historical-opt-in-auth-session-and-rate-limiter-selection), [Backlog](BACKLOG.md) and
[Extensions](EXTENSIONS.md). Research Done records an accepted decision, not a
delivered provider. The [naming guide](NAMING.md) fixes the current vocabulary;
historical plans and verification retain their original scopes and results.


[YT-185 application startup options](https://github.com/bohewu/yetlet/blob/4792565e6cff7a94c8c2c860d1858f840f122cbe/docs/YT-185-OPT-IN-OPTIONS.md) describe current
Board/Admin and Shop instance selection. Cargo availability and startup enablement
remain separate, default-off choices; Settings loading stays optional. Native
provider TLS and private experiment configuration retain their distinct scopes.
Earlier YT-181/182/183 verification describes the former constructors; the linked
migration and current sample guides own the new integration surface.
