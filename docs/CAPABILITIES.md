# Current capability and API carrier map

Date: 2026-10-26. This is the current package/integration inventory, not a
promotion to Stable Candidate or an expansion of the immutable release set.
Every package below is unpublished and experimental. The original nine
candidates retain their [M9 API classification](API-CONTRACTS.md) and
[release gates](RELEASE-READINESS.md); later additions are outside that inventory.

[YT-201](YT-201-PACKAGE-PREPARATION.md) prepares all sixteen library manifests
for experimental 0.1.0 packaging, without selecting a new immutable candidate
or changing runtime installation and platform guarantees.

Start at [Current API reference](API-REFERENCE.md) for current entry points,
ownership/error boundaries and local rustdoc. [Documentation scope](DOCUMENTATION-SCOPE.md)
classifies reader inputs and retained evidence; this inventory does not select
publication of its linked execution records.

## Framework packages and selected features

| Carrier | Implemented scope and maturity | Original nine release candidates? | Authoritative contract |
|---|---|---|---|
| `yetlet-core` | Renderer-neutral Views/components and bounded binding vocabulary; M9 classification applies | Yes | [Core guide](../crates/yetlet-core/README.md) |
| `yetlet-html` | Escaped HTML serialization and allowlisted elements/attributes; M9 classification applies | Yes | [HTML guide](../crates/yetlet-html/README.md) |
| `yetlet-dom` | Restricted native browser DOM mount; M9 classification applies | Yes | [DOM guide](../crates/yetlet-dom/README.md) |
| `yetlet-dom-keyed` | Optional keyed children and disposable listener/entry ownership | Yes | [Keyed DOM guide](../crates/yetlet-dom-keyed/README.md) |
| `yetlet-reactive` | Optional synchronous local reducer/commit graph with explicit lifetime | Yes | [Reactive guide](../crates/yetlet-reactive/README.md) |
| `yetlet-actions` | Typed commands and explicit local/HTTP execution boundaries | Yes | [Actions guide](../crates/yetlet-actions/README.md) |
| `yetlet-server-axum` | App/Page, request context and native Axum/Tower routing/composition | Yes | [Server guide](../crates/yetlet-server-axum/README.md) |
| `yetlet-request-id` | Explicit scoped/global process-local numeric correlation | Yes | [Request ID guide](../crates/yetlet-request-id/README.md) |
| `yetlet-session` | Opt-in bounded Memory Session, anonymous state, expiry, CSRF, cookie lifecycle, rotation/revocation; synchronous API | Yes | [Session guide](../crates/yetlet-session/README.md) |
| `yetlet-session-redis` | Isolated, publication-prepared/unpublished, async provider. Retained qualification is single-host, single-primary, no persistence; compatible Valkey use is separately evidenced | No | [Provider guide](../crates/yetlet-session-redis/README.md), [Valkey evidence](YT-094-VALKEY-SESSIONS.md) |
| `yetlet-session-redis/tls` | Default-off native Rustls/Ring transport for checked `rediss://`; unchanged Session/Lua/limits, no insecure fragments. YT-184 bounded evidence complete from generation4 Auth plus generation5 authority recovery/persistent restart/new-epoch re-login; transient503/failures retained, all five groups deleted. Done at bounded isolated Linux reference scope after final independent Reviewer PASS0; no general HA/restore/platform claim | No | [Provider TLS guide](../crates/yetlet-session-redis/README.md#optional-native-tls--yt-184-reference), [YT-184](YT-184-AZURE-AUTH-SESSION-RESTART.md) |
| `yetlet-config` | Isolated, publication-prepared/unpublished, opt-in typed startup loader and finite source precedence; optional `secret-provider` integration | No | [Configuration](CONFIGURATION.md), [settings/secret integration](SETTINGS-SECRET-PROVIDER.md) |
| `yetlet-secrets` | Isolated, publication-prepared/unpublished, native async secret lookup and explicit settings adapter; later local rotation/Azure evidence remains separately bounded | No | [Secret providers](SECRET-PROVIDERS.md), [rotation evidence](SECRET-ROTATION-VERIFICATION.md) |
| `yetlet-signing` | Isolated, publication-prepared/unpublished, injected signer/public-key source and explicit trust/version policy; local OpenSSL-backed sample/CLI consumers | No | [Signing contract](YT-093-SIGNING-CONTRACT.md), [verification](YT-093-SIGNING-VERIFICATION.md) |
| `yetlet-auth` | Root workspace, publication-prepared/unpublished, native identity foundation/resolver/extractors. YT-114 Done after independent review; business authorization remains application-owned | No | [Auth guide](../crates/yetlet-auth/README.md), [contract](YT-113-AUTHENTICATION-CONTRACT.md) |
| `yetlet-auth/session` | Explicit optional feature bridging the existing Memory Session to application identity mapping. YT-115 Done after independent review; not a generic Redis bridge or complete auth product | No | [Memory bridge](YT-115-SESSION-AUTH.md) |
| `yetlet-rate-limit` | Root workspace, publication-prepared/unpublished, native opt-in Tower middleware, endpoint/group Policy and explicitly shared Budget. YT-117 Done at per-process memory scope; configured example defaults off | No | [Rate-limit guide](../crates/yetlet-rate-limit/README.md), [contract/evidence](YT-117-RATE-LIMIT.md) |
| `yetlet-wgpu` | Isolated, publication-prepared/unpublished, optional flat-scene renderer, CPU-only defaults and explicit wasm32 browser feature; selected configuration only | No | [GPU guide](../crates/yetlet-wgpu/README.md), [support](../crates/yetlet-wgpu/SUPPORT.md) |

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
| Yetlet CLI | Separate unpublished tool with typed parser/support API, explicit scaffolding and configuration-only generators; per-command/source/platform limits apply | [Current CLI guide](../tools/yetlet-cli/README.md#current-command-and-capability-inventory) |

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
