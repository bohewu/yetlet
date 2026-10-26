# ADR 0005 — Opt-in capability modules and application composition

- **Status:** Implemented at separately bounded scopes: composition/request ID, Memory Session, experimental async Redis provider, native Auth foundation, optional Memory Session/Auth bridge and per-process rate limiter. Application-owned Valkey/Auth adoption is separately qualified. Cache and other unselected providers remain future choices; production deployment is unqualified. See the [current capability map](../CAPABILITIES.md).
- **Date:** 2026-10-08
- **Authority:** Maintainer approval of extensible opt-in Session, Cache, Cookie Authentication, and related features.
- **Related:** [Architecture](../ARCHITECTURE.md), [Backlog](../BACKLOG.md), [Roadmap](../ROADMAP.md), [ADR 0004](0004-request-context-and-middleware.md), [ADR 0006](0006-handler-first-mvc-optional.md).

## Problem and goal

Yetlet's baseline SSR/HTTP and browser WASM must remain useful without sessions, authentication, Redis, application caching, tracing, or other optional capabilities. The priority is **ease of extension with no mandatory feature cost**, not creating a large, dynamic plugin system.

## Approved principles

1. **Dependency and runtime opt-in.** Capabilities should be in independent Rust crates, or narrow opt-in crate features when simpler. Applications without a dependency and explicit registration must not initialize the provider, run its middleware, or pull storage/security dependencies into the base crate.
2. **Native composition.** Integrate HTTP features through existing Tower `Layer`/`Service`, Axum middleware and routing scopes, typed `RequestContext` extensions and explicit application state `S`. Avoid a parallel middleware engine, framework-wide DI container, ambient global state or service locator.
3. **Small RequestContext.** Keep `RequestContext<S>` focused on the original request parts/body, typed request-local extensions and shared application state. Do not add fixed `Session`, `User`, `Cookies`, `Cache`, `TraceId` or `RequestId` fields. Modules may expose their own strongly typed extractors/helpers.
4. **Request ID is not a core default.** M6 moves the Todo correlation example into the independent `yetlet-request-id` crate. The app explicitly installs `RequestIds::new().global(app)`; scoped installation is also available. A per-provider AtomicU64 generates numeric IDs starting at 1, stored as typed RequestId extensions and response headers. The root explicitly selects the number for SSR. This is not globally unique, persistent or authentication identity, and RequestContext does not generate it.
5. **Separate feature boundaries.** Cookies manage HTTP cookies; sessions manage session identity/storage/expiry; authentication establishes verified identity through resolution/extraction. Authorization remains application-owned resource/action policy; a verified identity alone grants no business permission. Data cache stores application data; output cache stores HTTP responses; rate limiting and tracing remain separate. Cookie-auth need not imply server-side sessions, and anonymous sessions need not imply authentication.
6. **Narrow optional providers.** Storage backends may use memory, Redis, SQL or a custom implementation via a small provider contract; a Redis dependency belongs to its provider rather than the core. Do not invent broad provider factories before demonstrating a real consumer.
7. **Target and security isolation.** Optional server-side crates must not enter browser WASM, and request extensions/server handles must not be automatically serialized into SSR/hydration. Cookies and authentication need real Secure/HttpOnly/SameSite, CSRF, key rotation, session fixation/revocation and authorization rules where applicable. Authenticated output caching must handle `Cache-Control`, `Vary` and per-user isolation.
8. **Incremental, handler-first DX.** M6 should prove a thin App/route builder, ordinary function handlers with lightweight View/JSON/redirect results, typed optional capability installation using the existing Tower boundary, and one independently opt-in example. MVC is optional application organization; never require Controllers, an extra model binder, a plugin manager, or full dependency injection. See [ADR 0006](0006-handler-first-mvc-optional.md).

## Historical M6 capability candidates — 2026-10-08

This table records the original proposal vocabulary. Implemented carriers,
selected features and research decisions are maintained in the
[current capability map](../CAPABILITIES.md); a candidate name alone does not
establish an implemented crate, adapter or release inclusion.

| Candidate name | Responsibility | Optional dependency |
|---|---|---|
| `yetlet-cookies` | Read/set/protect cookies | HTTP only |
| `yetlet-session` | Session identity, absolute expiry, bounded memory storage and CSRF token | Implemented in M8c; other stores remain future |
| `yetlet-auth` | Authentication foundation; business authorization is application-owned | Provider integration requires explicit selection |
| `yetlet-cache` | Application data caching | Optional memory/Redis/other store |
| `yetlet-output-cache` | HTTP/SSR response caching | Explicit cache policy; avoid private data leakage |
| `yetlet-rate-limit` | Admission policies | Optional limiter/provider |
| `yetlet-tracing` | Correlation IDs and request tracing | Optional span/ID strategy |

At the M6/M8c baseline, `yetlet-request-id` and then `yetlet-session` supplied the
bounded opt-in proofs. Subsequent Auth and rate-limit implementations are
recorded below and in the current map. Unselected names remain candidates, not
a commitment to build all of them. Simpler features need not each become a crate.

## M21 planned follow-up (2026-10-22; registration only)

The maintainer approved scheduling the [YT-113–YT-123 opt-in capability batch](../YT-113-123-OPT-IN-CAPABILITIES.md),
**not** immediate implementation or a new `yetlet-core` API. YT-113 now records the [documentation-only authentication contract study](../YT-113-AUTHENTICATION-CONTRACT.md),
Done at research scope after independent Reviewer PASS. It selects the smallest server-side
authentication contract and application-owned authorization boundary. The
subsequently selected YT-114 foundation and YT-115 Memory Session bridge are
implemented after their independent reviews, without making Session synonymous
with identity or changing its synchronous API. YT-116 research retained the
existing application-owned OIDC workflow; YT-117 implements independently
opted-in rate limiting with narrowly qualified per-process limits.

The other ADR candidates — `yetlet-cookies`, `yetlet-cache`,
`yetlet-output-cache` and `yetlet-tracing` — have research-only or
consumer-gated YT-120–YT-123 tasks. New proposals `yetlet-audit` (YT-118)
and optional Bearer validation (YT-119) are **not accepted provider crates**.
YT-119/120 are Done at research scope with no new adapter/crate selected;
[YT-118 Audit research](../YT-118-AUDIT-STUDY.md) is Done after independent Reviewer PASS;
[YT-121 Cache research](../YT-121-CACHE-STUDY.md) is Done after independent Reviewer PASS; [YT-122 Output Cache research](../YT-122-OUTPUT-CACHE-STUDY.md) is Done after independent Reviewer PASS; [YT-123 Tracing/Correlation research](../YT-123-TRACING-CORRELATION-STUDY.md) is Done after independent Reviewer PASS. The subsequent owner-selected
[YT-114 foundation](../YT-114-IMPLEMENTATION.md) implements only native identity
resolution/extraction with independent Reviewer PASS. No Audit or Cache crate
is implemented by this note. Previously qualified Session/Request ID/Secret
work remains unchanged. Server-only authentication must never add default
frontend/WASM dependencies, serialize a trusted principal, adopt unsigned
roles, or bypass the existing Axum/Tower route-scope contract.

The later owner-selected [YT-117 rate-limiting slice](../YT-117-RATE-LIMIT.md)
is Done at per-process scope after independent Reviewer PASS. It uses a separate
native Tower layer, validated reusable Policy
settings and explicitly shared Budget state, with bounded per-process Governor
accounting. Application-owned startup configuration defaults off and omits all
limiter construction and request work while disabled. No default dependency,
Auth coupling, runtime registry, hot reload or distributed quota is selected.
The initial slice passed focused new verification without changing accepted
Core/server/browser or default Admin normal dependency graphs.

## M6 acceptance criteria (locally verified)

- SSR-only application builds and runs without any optional capability crate.
- One explicitly installed capability integrates through Tower/Axum and typed extensions/state; scoped registration does not affect unrelated routes.
- No optional module is active by default; demonstrate absence of its dependency, side effects and per-request execution when not installed.
- Browser WASM remains independent of server capability crates; API and SSR pipeline semantics remain intact.
- Show configuration error handling, middleware ordering/short-circuit tests and document security responsibilities before implementing session or authentication.
- Keep API syntax illustrative until a tested App Builder contract exists.

## Rejected alternatives

An always-on session/auth/cache layer, a global service locator, a central dynamic plugin registry, and a mega-crate with every provider enabled by default all impose unnecessary coupling or per-application cost.

**Current implementation:** `App::configure` takes an ordinary typed setup function, and `try_configure` propagates its chosen error type. They execute once during construction, not per request. `yetlet-request-id` proves independent optional correlation. M8c adds `yetlet-session`, with scoped native middleware, typed `Session<T>`, OS-random identity/CSRF, absolute expiry/capacity, rotation/revocation and an SSR cart consumer. Both remain absent from the core/browser dependency graphs unless explicitly selected. The later separately selected [YT-114 Auth foundation](../YT-114-IMPLEMENTATION.md) is Done at bounded scope after independent Reviewer PASS; The separately selected [YT-115 Memory Session bridge](../YT-115-SESSION-AUTH.md) is now Done after independent Reviewer PASS; Cache remains unimplemented. The later separate `yetlet-session-redis` nested workspace is experimental and explicitly selected by applications; [M10g](../M10G-VERIFICATION.md#yt-066-bounded-framework-closeout--2026-10-11) qualifies its single-host/single-primary/no-persistence scope without changing Memory or Core/default/browser graphs. Other distributed providers and production durability/HA remain unqualified. See [Extensions](../EXTENSIONS.md), [M6 evidence](../M6-VERIFICATION.md) and [M8c evidence](../M8C-VERIFICATION.md).
