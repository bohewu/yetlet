# Public API Contract Classification

**Date:** 2026-10-10
**Scope:** M9j/YT-059 compatibility inventory for the current `0.1.0` prototype.
**Status:** Complete regression and repeat independent final reviewer `wc_job_QkQkovlBlvrhPxCE` support this classification. It remains pre-SemVer evidence, not a stability guarantee.

This document inventories the actual public Rust surface exercised by M0–M10c. It separates **Stable Candidate**, **Experimental**, and **Internal/support-only** contracts. These labels are evidence classifications, not SemVer guarantees, production approval or a promise that package names and signatures will never change. The provisional change discipline is recorded in [Compatibility Policy](COMPATIBILITY.md).

The original post-M9 security round validated Rust 1.88; that evidence remains historical. Licensing, document metadata, bounded qualification and private governance are now complete at their recorded scopes. The maintainer selected experimental `0.1.0` and Rust 1.99 before publication; [Compatibility Policy](COMPATIBILITY.md) defines patch preservation and future increases. Public activation remains blocked by [concrete release gates](YT-020-INTEGRITY-PREPARATION.md#remaining-activation-gates), not by a requirement to certify every platform or application deployment. These API classifications remain unchanged.

## Classification meanings

- **Stable Candidate:** Repeatedly exercised by materially different applications or renderers, with negative/lifecycle evidence and no known required architectural replacement. Breaking changes should require explicit rationale and migration notes, but compatibility is not yet promised.
- **Experimental:** Useful and tested, but the shape is supported by too few consumers, exposes a known design gap, or may change as authoring, browser, storage or security requirements expand.
- **Internal/support-only:** Public for adapter wiring, testing, escape hatches or upstream interoperability. It is not the recommended application contract and receives no compatibility expectation yet.

## Stable Candidates

### `yetlet-core`

| Public surface | Evidence | Known break risks |
|---|---|---|
| `View`, `View::text` | Shared SSR and browser component source across Todo, Hello, Forms, Cart, Auth and Modular Shop | Future richer node kinds may be added; exhaustive matching should remain discouraged |
| `Element`, `Attribute`; `Element::{new,attribute,child,children,tag,attributes,child_views}` | Every SSR sample, HTML renderer, Browser renderer and M7/M9 regressions | Static tag/attribute names intentionally keep the trust boundary narrow; namespace/head metadata remain separate questions |
| `binding::{ComponentIdentity,BindingKey,SlotName,InertAttributeName,SlotDescriptor,BindingContractError}` and their validated constructors/accessors | Promoted in M7f, consumed by keyed prototype and production Todo keyed path | General template IR, hydration and authoring macros are explicitly not selected |

### `yetlet-html`

| Public surface | Evidence | Known break risks |
|---|---|---|
| `RenderError`, `escape_text`, `render_to_string` | Native SSR escaping, hostile text/attribute samples, browser/server parity and complete regression | Streaming/suspense rendering is deferred; current function is complete-string rendering |

### `yetlet-server-axum` handler and middleware boundary

| Public surface | Evidence | Known break risks |
|---|---|---|
| `App<S>` with `new`, `from_router`, `into_router`, `map_router`, `configure`, `try_configure`, `route`, `get`, `post`, `group`, `route_layer`, `with_state` | REST, Session Cart, Middleware Lab, Auth Notes, Modular Shop and optional providers use ordinary Axum/Tower composition | Additional HTTP verbs remain available through `route`; no second router or DI contract is implied |
| `RequestContext<S>` accessors for method/URI/path/headers/extensions/state/body/matched path/original URI/query/path params and request/body ownership transfer | M4/M6 middleware, REST policy, Request ID and Middleware Lab ordering/body tests | Constructor remains support-only below; extractor behavior follows Axum routing semantics |
| `middleware::{Next,from_fn,from_fn_with_state,with_global}` | Global/group/route ordering, short circuit, streaming failures, 404/405 and Tower interoperability | `with_global` requires a completed router and intentionally exposes native Tower constraints |

### `yetlet-request-id` and `yetlet-session` application boundaries

| Public surface | Evidence | Known break risks |
|---|---|---|
| `yetlet-request-id::{HEADER,RequestId,RequestIds,InvalidStart}` and `RequestIds::{new,try_new,scoped,global}` | Todo and REST use the exact public header name, scoped/global correlation, typed invalid-start result, dependency isolation and overflow/config tests | Numeric process-local IDs are not distributed tracing identity; future distributed tracing uses a different capability |
| `yetlet-session::{SessionConfig,SessionConfigError,SessionManager,Session,SessionError,SessionUpdateError}` with `SessionManager::{new,scoped}` and `Session::{read,try_update,csrf_token,verify_csrf,rotate,replace_and_rotate,destroy}` | Session Cart, Auth Notes and Modular Shop; typed configuration rejection, expiry/capacity, fixation rotation, stale response, concurrent update/destroy and CSRF evidence | Error enums may gain variants before SemVer adoption; Stable Candidate is limited to bounded in-memory opaque-session semantics, not persistent/distributed storage, authentication or production security certification |

## Experimental

### Document/page convenience

| Public surface | Why experimental |
|---|---|
| `Page`, `Page::new`, `App::page`, `render_document`, `render_page`, `HtmlPageError` | Extensively used and retained source-compatible, but the convenience/default-language policy remains experimental until the metadata contract has more independent consumers. |
| `LanguageTag`, `LanguageTagError`, `DocumentMetadata`, `Page::with_metadata`, `render_document_with_metadata`, `render_page_with_metadata` | M10c resolves the root-language ownership gap additively and removes the Forms sample's nested-language workaround. The contract is intentionally title + language only; it remains Experimental while consumer and migration evidence is limited. |
| `yetlet_server_axum::DEFAULT_CSP` | Every SSR/browser regression verifies the current default, but the exact directive string is security policy that may be tightened or made configurable with document metadata; callers should not rely on byte-for-byte permanence yet. |

### `yetlet-actions`, `yetlet-dom`, `yetlet-dom-keyed`, and `yetlet-reactive`

| Public surface | Why experimental |
|---|---|
| `yetlet-actions::{Command,ActionExecutor,ActionError,LocalExecutor,HttpExecutor,ApiErrorEnvelope,ApiErrorBody}` | Typed local/HTTP actions work in Todo, but only one production browser transport/consumer exists; cancellation, retry/auth/error-envelope policy may evolve |
| `yetlet-dom::{DomPolicyError,validate,mount}` | One simple positional renderer and strict policy are verified, but general component composition and hydration contracts remain deferred |
| `yetlet-dom-keyed::{KeyedOrderError,validate_keyed_order,DomEventListener,KeyedEntry,ReconcileStats,KeyedChildren}` | Bounded keyed direct-DOM primitive is used by Todo and lifecycle fixtures, but arbitrary shapes, transactional rollback and focus policy are not generalized |
| `yetlet-reactive::{FlushStats,ReactiveError,LocalGraph,GraphHandle,Subscription}`, `LocalGraph::DEFAULT_MAX_PASSES` and graph/handle/subscription methods | Explicit ownership, weak handles, batching, bounded pass limit and disposal are verified; this is not a general Signals/Effects/Resources framework, and scheduler/error/default-pass policy may still change |

### Session advanced/test hooks

| Public surface | Why experimental |
|---|---|
| `SessionConfig::insecure_for_local_http` | Explicitly restricted by each sample to loopback; easy to misuse without deployment admission policy |
| `SessionClock`, `SystemClock`, `SessionClockError`, `SessionManager::with_clock` | Valuable deterministic expiry hook, but clock/provider extensibility has only test consumers |
| `session_middleware` | Low-level wiring entry used by `SessionManager::scoped`; direct installation is not the preferred application contract |

## Internal/support-only

| Public surface | Reason |
|---|---|
| `RequestContext::new` | Adapter/extractor construction hook; applications receive `RequestContext` from Axum |
| `RequestIds::assign_request_id` | Public middleware function underlying `RequestIds::{scoped,global}`; direct calls are support wiring |
| Broad `yetlet_server_axum::http`, `extract` and response-type reexports | Convenience/interoperability surface inherited from the pinned Axum/http versions. Samples exercise selected upstream types, not the entire upstream modules; they are explicitly **not** Stable Candidates owned by Yetlet. |
| Public browser implementation structs' exact fields and generated wasm-bindgen exports | Build/runtime implementation details; only documented Rust constructors/methods above are reviewed contracts |

## Explicitly not adopted

M9 does not adopt a general VDOM, Signals framework, `ui!` procedural macro, generic forms runtime, ORM, `DbContext`, repository/unit-of-work framework, dynamic plugin registry, framework DI container, full resumability, default live event circuit, WebSocket abstraction, WebGPU DOM renderer or payment/commerce framework.

Application dependencies such as `rusqlite`, `tower-http`, `tempfile`, synthetic identity fixtures and browser build scripts remain opt-in application concerns. Repetition alone does not promote them to Yetlet core.

## Compatibility work still required

Before a public stability declaration:

1. retain the selected dual-license texts and verify the actual release inventories;
2. deliver the declared Rust 1.99 baseline and migrate its protected required CI check under separate authorization;
3. activate or revise the provisional cross-crate SemVer policy at the first public release;
4. gather additional consumers before promoting the bounded document metadata API;
5. declare supported browsers/OS/toolchains and CI coverage;
6. decide which optional provider security claims require independent audit;
7. add migration notes and compile-level compatibility fixtures for adopted Stable Candidates.
