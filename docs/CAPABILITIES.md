# Current capability and API carrier map

Updated: 2026-11-12. The published Experimental SDK has two implementation packages:
`yetlet` and `yetlet-browser`, version `0.9.0-alpha.8`, available from crates.io.
The module map below describes capabilities, not additional installable crates.
Base defaults select HTML; Browser defaults select DOM. Native capabilities and
GPU/action integrations are additive opt-ins with explicit startup wiring.
Earlier sixteen-package releases and qualification records keep their original
sources. Opt-in static HTML export from [YT-223](YT-223-STATIC-HTML-IMPLEMENTATION.md)
is distributed in [CLI `0.2.0`](CLI-020-RELEASE-PREPARATION.md#owner-approved-publication-and-installed-consumer--2026-11-02);
it adds no SDK module/default. SDKalpha.6 includes the YT-220/221 DOM vocabulary
and options, absent from SDKalpha.5. Earlier records keep their original
source, API names and scope. See [the migration](YT-211-TWO-PACKAGE-MIGRATION.md).

Start at [Current API reference](API-REFERENCE.md) for current entry points,
ownership/error boundaries and local rustdoc. [Documentation scope](DOCUMENTATION-SCOPE.md)
classifies reader inputs and retained evidence; this inventory does not select
publication of its linked execution records.

SDK alpha.8 includes [simple application startup and the optional Axum Prelude](API-REFERENCE.md#published-application-startup--sdk-alpha8)
with the opt-in `server-axum` feature. The published CLI `0.3.1` continues to
generate its existing explicit-listener native scaffold.

## Published typed authoring and CLI Style tooling

SDK alpha.7 **includes** `yetlet::view::tags` with 101 typed ordinary tag
constructors, the same owned View and existing renderer restrictions. See the
[typed View guide](VIEW-COMPONENTS.md#optional-typed-tag-builders-yt-234)
and the [SDK alpha.7 release record](ALPHA7-HOSTED-RECORD.md#owner-approved-publication-and-registry-consumer--2026-11-09).
The original YT-245 preparation selects local SDK `0.9.0-alpha.7` contents;
these typed constructors are now distributed rather than checkout-only.


Published SDKalpha.8/CLI0.3.1 artifacts serve separate roles. The independent CLI
ships native/browser Style templates and a private app generator. The Style
walkthroughs verify SDK alpha.7 with exact app pins and locks. The
[Style Showcase](../examples/style-showcase/README.md) remains a checkout sample.
See the [API carrier map](API-REFERENCE.md#source-only-checkout-additions)
and [Style contract](YT-237-STYLE-REGISTRATION-CONTRACT.md).
Style authoring has a finite private app vocabulary, explicit registration and
external build-time CSS; ordinary CSS remains the default and supplies unsupported
syntax. There is no public style crate, renderer change or runtime style injection.

## Framework packages and selected features

| Carrier | Implemented scope and maturity | Original nine release candidates? | Authoritative contract |
|---|---|---|---|
| `yetlet::view` | Renderer-neutral Views/components and bounded binding vocabulary; M9 classification applies | Yes | [Core guide](../crates/yetlet/guides/core.md) |
| `yetlet::html` | Escaped HTML serialization and allowlisted elements/attributes; M9 classification applies | Yes | [HTML guide](../crates/yetlet/guides/html.md) |
| `yetlet_browser::dom` | Native validation/Wasm-only owned mount. Since SDKalpha.6:89 default tags,90 with explicit ImagePolicy; SDKalpha.5 has38. Independent per-mount HTTPS link/image policies retain strict defaults and same-origin forms. Not full HTML conformance | Yes | [DOM guide](../crates/yetlet-browser/guides/dom.md), [vocabulary](YT-220-HTML-DOM-IMPLEMENTATION.md), [URL/image contract](YT-221-DOM-URL-IMAGE-CONTRACT.md) |
| `yetlet_browser::dom_keyed` | Optional keyed children and disposable listener/entry ownership | Yes | [Keyed DOM guide](../crates/yetlet-browser/guides/dom-keyed.md) |
| `yetlet::reactive` | Optional synchronous local reducer/commit graph with explicit lifetime | Yes | [Reactive guide](../crates/yetlet/guides/reactive.md) |
| `yetlet::actions` | Typed commands and explicit local/HTTP execution boundaries | Yes | [Actions guide](../crates/yetlet/guides/actions.md) |
| `yetlet::server_axum` | App/Page, request context and native Axum/Tower routing/composition | Yes | [Server guide](../crates/yetlet/guides/server-axum.md) |
| `yetlet::request_id` | Explicit scoped/global process-local numeric correlation | Yes | [Request ID guide](../crates/yetlet/guides/request-id.md) |
| `yetlet::session` | Opt-in bounded Memory Session, anonymous state, expiry, CSRF, cookie lifecycle, rotation/revocation; synchronous API | Yes | [Session guide](../crates/yetlet/guides/session.md) |
| `yetlet::session_redis` | Opt-in native module in the published SDK. Original qualification is single-host, single-primary, no persistence; later bounded TLS/restart evidence is listed separately below | No | [Provider guide](../crates/yetlet/guides/session-redis.md), [Valkey evidence](YT-094-VALKEY-SESSIONS.md) |
| `yetlet/session-redis-tls` | Default-off native Rustls/Ring transport for checked `rediss://`; unchanged Session/Lua/limits, no insecure fragments. YT-184 bounded evidence complete from generation4 Auth plus generation5 authority recovery/persistent restart/new-epoch re-login; transient503/failures retained, all five groups deleted. Done at bounded isolated Linux reference scope after final independent Reviewer PASS0; no general HA/restore/platform claim | No | [Provider TLS guide](../crates/yetlet/guides/session-redis.md#optional-native-tls--yt-184-reference), [YT-184](YT-184-AZURE-AUTH-SESSION-RESTART.md) |
| `yetlet::config` | Opt-in native module providing typed startup loader and finite source precedence; optional `secret-provider` integration | No | [Configuration](CONFIGURATION.md), [settings/secret integration](SETTINGS-SECRET-PROVIDER.md) |
| `yetlet::secrets` | Opt-in native async secret lookup and explicit settings adapter; later local rotation/Azure evidence remains separately bounded | No | [Secret providers](SECRET-PROVIDERS.md), [rotation evidence](SECRET-ROTATION-VERIFICATION.md) |
| `yetlet::signing` | Opt-in native injected signer/public-key source and explicit trust/version policy; local OpenSSL-backed sample/CLI consumers | No | [Signing contract](YT-093-SIGNING-CONTRACT.md), [verification](YT-093-SIGNING-VERIFICATION.md) |
| `yetlet::auth` | Opt-in native identity foundation/resolver/extractors in the published SDK. YT-114 Done after independent review; business authorization remains application-owned | No | [Auth guide](../crates/yetlet/guides/auth.md), [contract](YT-113-AUTHENTICATION-CONTRACT.md) |
| `yetlet/auth-session` | Explicit optional feature bridging the existing Memory Session to application identity mapping. YT-115 Done after independent review; not a generic Redis bridge or complete auth product | No | [Memory bridge](YT-115-SESSION-AUTH.md) |
| `yetlet::rate_limit` | Native opt-in Tower middleware in the published SDK, endpoint/group Policy and explicitly shared Budget. YT-117 Done at per-process memory scope; configured example defaults off | No | [Rate-limit guide](../crates/yetlet/guides/rate-limit.md), [contract/evidence](YT-117-RATE-LIMIT.md) |
| `yetlet_browser::wgpu` | Optional flat-scene renderer in the published browser SDK, CPU-only defaults and explicit wasm32 browser feature; selected configuration only | No | [GPU guide](../crates/yetlet-browser/guides/wgpu.md), [support](../crates/yetlet-browser/guides/wgpu-support.md) |

Root workspace membership is a development/build choice, not automatic
installation in an application's dependency graph or inclusion in the release
candidate. There is no mandatory Settings/Auth/Session/Cache/Tracing service,
DI container or provider registry. Server-only capability dependencies and
trusted request extensions stay out of browser builds and hydration state.

## Application-owned integrations and tools

| Carrier | Current status and boundary | Authority |
|---|---|---|
| Board/Admin OIDC | Existing application-owned maintained-client integration and explicit issuer/subject to local-account mapping. YT-116 research Done; no extracted generic OIDC adapter | [OIDC study](YT-116-OIDC-ADAPTER-STUDY.md), [accepted provider evidence](YT-094-OIDC.md) |
| YT-125 Valkey/Auth consumer | Isolated application-owned async Auth resolver and two-process qualification; fresh shared local authority, explicit lifecycle/failure semantics | [YT-125](YT-125-VALKEY-AUTH-INTEGRATION.md) |
| Admin Valkey/Auth adoption | YT-126 Done after focused verification and independent review; explicit feature/runtime selection, fresh policy and documented pre-SQL race limit. No generic crate bridge or cross-store commit fencing | [YT-126](YT-126-ADMIN-VALKEY-AUTH-ADOPTION.md), [contract study](YT-124-MULTI-INSTANCE-AUTH-SESSION-CONTRACT.md) |
| Shop optional catalog display cache | YT-183 Done after independent Reviewer PASS, zero issues/followups: app-only default-off `catalog-cache`, bounded age/admission and generation invalidation; Session/checkout remain authoritative, no framework cache crate or output cache | [Contract](YT-183-CATALOG-CACHE-CONTRACT.md), [implementation/new proof](YT-183-CATALOG-CACHE-IMPLEMENTATION.md) |
| Opaque Bearer capabilities | Existing File Server/Production Starter app-owned route guards; no generic identity-aware JWT/introspection provider | [Bearer study](YT-119-BEARER-STUDY.md) |
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
| YT-118 Audit | [Research](YT-118-AUDIT-STUDY.md) Done after independent Reviewer PASS; preserve app-owned history/event policy | No `yetlet-audit` implementation |
| YT-121 Data cache | [Research](YT-121-CACHE-STUDY.md) Done after independent Reviewer PASS; current reads/snapshots remain app-owned | No `yetlet-cache` implementation |
| YT-122 Output cache | [Research](YT-122-OUTPUT-CACHE-STUDY.md) Done after independent Reviewer PASS; retain app-owned header/static policy | No `yetlet-output-cache` implementation; private output remains uncached by default |
| YT-123 Tracing | [Research](YT-123-TRACING-CORRELATION-STUDY.md) Done after independent Reviewer PASS; retain optional RequestIds and app-owned native composition | No `yetlet-tracing` implementation; request IDs alone are not distributed tracing |

See the [M21 plan](YT-113-123-OPT-IN-CAPABILITIES.md), [Backlog](BACKLOG.md) and
[Extensions](EXTENSIONS.md). Research Done records an accepted decision, not a
delivered provider. The [naming guide](NAMING.md) fixes the current vocabulary;
historical plans and verification retain their original scopes and results.


[YT-185 application startup options](YT-185-OPT-IN-OPTIONS.md) describe current
Board/Admin and Shop instance selection. Cargo availability and startup enablement
remain separate, default-off choices; Settings loading stays optional. Native
provider TLS and private experiment configuration retain their distinct scopes.
Earlier YT-181/182/183 verification describes the former constructors; the linked
migration and current sample guides own the new integration surface.
