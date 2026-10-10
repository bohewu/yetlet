# Public API Contract Classification

**Date:** 2026-10-10
**Scope:** M9j/YT-059 compatibility inventory for the current `0.1.0` prototype.
**Status:** Complete regression and repeat independent final reviewer `wc_job_QkQkovlBlvrhPxCE` support this classification. It remains pre-SemVer evidence, not a stability guarantee.

This document inventories the actual public Rust surface exercised by M0–M9. It separates **Stable Candidate**, **Experimental**, and **Internal/support-only** contracts. These labels are evidence classifications, not SemVer guarantees, production approval or a promise that package names and signatures will never change.

Public release remains blocked by licensing, browser/support and SemVer policy, document metadata, repository governance and production security expectations. A post-M9 security round raises the declared MSRV to Rust 1.88 and validates that toolchain, but a future release decision must still adopt an MSRV upgrade policy before any Stable Candidate becomes stable.

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
| `Page`, `Page::new`, `App::page`, `render_document`, `render_page`, `HtmlPageError` | Extensively used, but the complete-document contract still fixes root language/head metadata. M9c had to add sample-owned `lang="en"` because the current document root is `zh-Hant`. A typed document metadata/language contract may require a breaking signature change before stability. |
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

1. choose and publish repository/crate licensing;
2. retain CI for the declared Rust 1.88 MSRV and define the future upgrade policy;
3. define SemVer scope across crates and upstream Axum/Tower compatibility;
4. replace or deliberately freeze the complete-document language/head metadata contract;
5. declare supported browsers/OS/toolchains and CI coverage;
6. decide which optional provider security claims require independent audit;
7. add migration notes and compile-level compatibility fixtures for adopted Stable Candidates.
