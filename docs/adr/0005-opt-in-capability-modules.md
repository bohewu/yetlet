# ADR 0005 — Opt-in capability modules and application composition

- **Status:** M6 composition/request-ID and bounded M8c Memory Session are implemented. M10g adds a separately opt-in experimental Redis provider qualified only at single-host/single-primary/no-persistence scope; Auth/Cache, other providers and production deployment remain future choices.
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
5. **Separate feature boundaries.** Cookies manage HTTP cookies; sessions manage session identity/storage/expiry; authentication manages identity/authorization; data cache stores application data; output cache stores HTTP responses; rate limiting and tracing remain separate. Cookie-auth need not imply server-side sessions, and anonymous sessions need not imply authentication.
6. **Narrow optional providers.** Storage backends may use memory, Redis, SQL or a custom implementation via a small provider contract; a Redis dependency belongs to its provider rather than the core. Do not invent broad provider factories before demonstrating a real consumer.
7. **Target and security isolation.** Optional server-side crates must not enter browser WASM, and request extensions/server handles must not be automatically serialized into SSR/hydration. Cookies and authentication need real Secure/HttpOnly/SameSite, CSRF, key rotation, session fixation/revocation and authorization rules where applicable. Authenticated output caching must handle `Cache-Control`, `Vary` and per-user isolation.
8. **Incremental, handler-first DX.** M6 should prove a thin App/route builder, ordinary function handlers with lightweight View/JSON/redirect results, typed optional capability installation using the existing Tower boundary, and one independently opt-in example. MVC is optional application organization; never require Controllers, an extra model binder, a plugin manager, or full dependency injection. See [ADR 0006](0006-handler-first-mvc-optional.md).

## Capability modules

| Candidate name | Responsibility | Optional dependency |
|---|---|---|
| `yetlet-cookies` | Read/set/protect cookies | HTTP only |
| `yetlet-session` | Session identity, absolute expiry, bounded memory storage and CSRF token | Implemented in M8c; other stores remain future |
| `yetlet-auth` | Authentication and authorization | Optional cookie, bearer or OIDC scheme |
| `yetlet-cache` | Application data caching | Optional memory/Redis/other store |
| `yetlet-output-cache` | HTTP/SSR response caching | Explicit cache policy; avoid private data leakage |
| `yetlet-rate-limit` | Admission policies | Optional limiter/provider |
| `yetlet-tracing` | Correlation IDs and request tracing | Optional span/ID strategy |

`yetlet-session` and `yetlet-request-id` now exist as bounded opt-in crates. The other names remain candidates, not a commitment to build all of them. Simpler features need not each become a crate.

## M6 acceptance criteria (locally verified)

- SSR-only application builds and runs without any optional capability crate.
- One explicitly installed capability integrates through Tower/Axum and typed extensions/state; scoped registration does not affect unrelated routes.
- No optional module is active by default; demonstrate absence of its dependency, side effects and per-request execution when not installed.
- Browser WASM remains independent of server capability crates; API and SSR pipeline semantics remain intact.
- Show configuration error handling, middleware ordering/short-circuit tests and document security responsibilities before implementing session or authentication.
- Keep API syntax illustrative until a tested App Builder contract exists.

## Rejected alternatives

An always-on session/auth/cache layer, a global service locator, a central dynamic plugin registry, and a mega-crate with every provider enabled by default all impose unnecessary coupling or per-application cost.

**Current implementation:** `App::configure` takes an ordinary typed setup function, and `try_configure` propagates its chosen error type. They execute once during construction, not per request. `yetlet-request-id` proves independent optional correlation. M8c adds `yetlet-session`, with scoped native middleware, typed `Session<T>`, OS-random identity/CSRF, absolute expiry/capacity, rotation/revocation and an SSR cart consumer. Both remain absent from the core/browser dependency graphs unless explicitly selected. Auth and Cache remain unimplemented. The later separate `yetlet-session-redis` nested workspace is experimental and explicitly selected by applications; [M10g](../M10G-VERIFICATION.md#yt-066-bounded-framework-closeout--2026-10-11) qualifies its single-host/single-primary/no-persistence scope without changing Memory or Core/default/browser graphs. Other distributed providers and production durability/HA remain unqualified. See [Extensions](../EXTENSIONS.md), [M6 evidence](../M6-VERIFICATION.md) and [M8c evidence](../M8C-VERIFICATION.md).
