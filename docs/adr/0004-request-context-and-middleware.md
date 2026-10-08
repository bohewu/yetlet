# ADR 0004 — RequestContext and native Tower middleware

- **Status:** P0 implemented and locally verified; optional P1 providers/benchmarks deferred.
- **Date:** 2026-10-08
- **Authority:** User-provided "Yetlet — Middleware Pipeline & RequestContext Implementation Handoff", P0 and acceptance checks.
- **Related:** [Architecture](../ARCHITECTURE.md), [MVP](../MVP.md), [Backlog](../BACKLOG.md).

## Gap and smallest integration path

The M0–M4 checkout already has Axum routing, Tower dependencies, ordinary HTTP responses, bounded JSON body parsing, shared application state and a native HTML renderer. Before this integration it had no Yetlet RequestContext, explicit global pre-routing boundary, documented nested middleware order, or request-isolation tests. The middleware task extends that checkout, not a new framework or replacement phase.

## Decisions

1. Add `RequestContext<S>` inside the existing **server-only** `yetlet-server-axum` crate. It owns the original request head/body and a typed application-state handle. Extraction splits the request without reading or cloning its body. It is neither cloneable nor serializable.
2. Request-local data uses the original `http::Extensions`. Application state is a separate `S`, cloned from Axum State; use `Arc`-backed handles for large state. There is no service locator, shared global request state, ambient context, or general dependency injection container.
3. Query/path decoding delegates to Axum extractors and happens only when requested. `matched_path()` is optional: it is absent before the application router matches. Route parameters are not fabricated in global middleware.
4. Re-export Axum `Next`, `from_fn`, and `from_fn_with_state`. Continue with `next.run(context.into_request()).await`; return a Response to short-circuit. Do not introduce another middleware trait, dynamic registry, future executor, or router.
5. Add `with_global(router, layer)`, applying a Tower Layer around the completed inner Router. An outer fallback-only Router preserves the existing `Router` return type and `axum::serve` call. This incurs a small routing envelope; it is not a claim of zero overhead.
6. Use existing Axum `Router::nest` and `Router::route_layer` for groups, `MethodRouter::layer` for route scope, and `ServiceBuilder` for declared-order composition within a scope. Layers apply to routes already registered; finish a group's routes before layering it.
7. A matched request enters global → outer group → nested group → route → handler and unwinds in reverse. An unmatched path sees global middleware and the inner 404 fallback, but no matched-route group/route layer. A matched path with an unsupported method sees the matching group/route layers (using the specified APIs) around Axum's 405, preserving Allow.
8. Errors remain ordinary responses through the existing IntoResponse/HtmlPageError/typed JSON paths. Outer middleware can inspect and modify 4xx/5xx. Transport errors from fallible Tower services must first be mapped through Axum HandleErrorLayer; panics are not silently caught.
9. Middleware owns the request until it calls Next. Capture any after-processing values before moving the request; request extensions do not magically appear on response extensions. No locks are held across await by the adapter.
10. SSR derives a tiny explicitly allowlisted render value from a request extension. RequestContext, application state, cookies, headers and arbitrary extensions are never automatically copied into HTML/hydration or browser WASM.

## Compatibility and scope

Existing handlers using State/Request and existing raw Axum/Tower layers remain valid. P0 proves native Tower composition with an existing tower-http response layer; no low-level HTTP dependency is replaced. A Yetlet `App::group` facade, arbitrary fallible-Service wrappers, authentication/session providers, tracing backends, deadlines/cancellation and benchmarks remain optional P1 work.

The integration must preserve existing routes, status/error JSON, SSR/Islands, and browser source/dependency graphs. The performance claim is **bounded architecture**, not measured latency: no eager buffering, duplicated type maps, runtime reflection, or client-side payload from this server-only change.

## Validation gate

Focused real-router tests must prove order, before/after, short-circuit, typed state, concurrent isolation, nested scopes, 404/405/Allow, error responses, API and SSR sharing, lazy request/response streaming, on-demand query/path decoding and Tower-layer interoperation. Run native formatting/lint/tests, browser compilation and a bounded regression smoke. Record actual results in the linked implementation note; do not turn this ADR into independent security assurance.

**Observed:** 46/46 workspace tests, format/Clippy, browser target compilation, English documentation checks and one headless regression PASS. Both existing WASM binaries are byte-identical to the pre-integration baseline. See [HTTP pipeline evidence](../HTTP-PIPELINE.md). Native overhead is not benchmarked; no independent reviewer or formal assurance result is claimed.
