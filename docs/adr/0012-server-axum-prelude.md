# ADR 0012 — Curated Native Axum Application Prelude

- **Status:** Accepted (contract only; YT-254 implementation pending).
- **Date:** 2026-11-11.
- **Task:** YT-252, M26 Application Startup & Sample Experience.
- **Related:** [M26 plan](../YT-251-258-APPLICATION-STARTUP-SAMPLE-PLAN.md),
  [ADR 0001 — Separation of concerns](0001-separation-of-concerns.md),
  [ADR 0005 — Opt-in capability modules](0005-opt-in-capability-modules.md),
  [ADR 0006 — Handler-first and optional MVC](0006-handler-first-mvc-optional.md),
  [ADR 0011 — Application startup](0011-application-startup-api.md),
  [current API reference](../API-REFERENCE.md),
  [sample catalog](../SAMPLES.md).

## Context

Yetlet's Core crate already re-exports its primary renderer-neutral View vocabulary (`Attribute`, `Element`, `View`) at the crate root. Native server applications additionally import a small set of types from `server_axum`, its `extract` and `response` modules, and selected Axum HTTP types. The current API is explicit and source-compatible, but repeating several module paths in every application adds avoidable import boilerplate.

The native adapter must remain optional and thin. A prelude should improve common application imports without pretending that Yetlet owns all of Axum, hiding advanced transport/middleware APIs, flattening the crate root, or changing any existing public path.

The owner-selected M26 principle is **Simple first, customizable when needed**. For imports this means one curated server-adapter prelude, not a universal `use yetlet::*` or a complete Axum re-export.

## Decision

Add one optional `pub mod prelude` inside `yetlet::server_axum`. It is available only when the existing native `server-axum` feature is selected; the containing module's current native target gate continues to exclude it from `wasm32`.

Proposed implementation shape:

~~~rust
pub mod prelude {
    pub use crate::{Attribute, Element, View};
    pub use super::{
        App, DEFAULT_CSP, DocumentMetadata, HtmlPageError, LanguageTag, LanguageTagError,
        Page, RequestContext, render_document, render_document_with_metadata, render_page,
        render_page_with_metadata,
    };
    pub use super::extract::{DefaultBodyLimit, Extension, Form, Path, Query, State};
    pub use super::http::{HeaderMap, HeaderValue, StatusCode};
    pub use super::response::{IntoResponse, Json, Redirect, Response};
}
~~~

The exact public inventory is:

| Area | Prelude names | Existing source / decision |
|---|---|---|
| Core View values | `Attribute`, `Element`, `View` | `yetlet` crate-root re-exports; no new Core prelude |
| Application and response | `App`, `Page`, `RequestContext` | `server_axum` public re-exports |
| Document metadata | `DocumentMetadata`, `LanguageTag`, `LanguageTagError` | `server_axum` public re-exports |
| Page error and policy | `HtmlPageError`, `DEFAULT_CSP` | `server_axum` public items |
| Common extractors | `DefaultBodyLimit`, `Extension`, `Form`, `Path`, `Query`, `State` | `server_axum::extract`; add `Form` as an additive re-export from Axum's existing `axum::extract::Form` |
| Common HTTP values | `HeaderMap`, `HeaderValue`, `StatusCode` | Selected individual names from the existing `server_axum::http` re-export |
| Common responses | `Json`, `IntoResponse`, `Redirect`, `Response` | `server_axum::response`; export `Json` from this module in the prelude only |
| Rendering helpers | `render_document`, `render_document_with_metadata`, `render_page`, `render_page_with_metadata` | Existing public `server_axum` functions |

`Form` is selected because the official Forms Validation sample uses Axum's native form extractor and Axum 0.8.9's default feature set already includes `form`. Adding it to `server_axum::extract` is an additive convenience re-export; it adds no new dependency or parser behavior. If the Axum dependency later disables default features, the `form` feature must be selected explicitly while this re-export remains available.

`HeaderMap`, `HeaderValue` and `StatusCode` are selected individually because application handlers commonly set response headers or return status codes. The `http` module itself remains available at `server_axum::http`, but the prelude does not re-export that entire namespace.

`Json` remains available at both existing explicit paths (`server_axum::extract::Json` and `server_axum::response::Json`). The prelude imports it exactly once from `response`; the same Axum `Json` type can still be used as an extractor or response.

## Explicitly excluded

- No `yetlet::prelude` is introduced. The Core crate already exposes the three primary View values at its root; another prelude containing only those same names would be a redundant alias.
- No root-level flattening: do not move `App`, `Page`, HTTP types or server APIs to `yetlet::`.
- No `pub use axum::*`, `pub use axum::http::*`, or entire `http` namespace in the prelude.
- No `Router`, `MethodRouter`, `Request`, `Body`, `axum::serve`, `TcpListener`, Tokio, Tower services/layers, `middleware` module or runtime helpers. Advanced transport, routing and middleware imports remain explicit.
- No feature-specific `Multipart`, rejection types such as `FormRejection`/`JsonRejection`, or other opt-in Axum extractors are added merely to make the prelude appear complete. Applications import those from their existing explicit Axum paths and select the required Axum feature.
- No removal, rename or behavior change to `yetlet::view::View`, `yetlet::server_axum::App`, `server_axum::extract::*`, `server_axum::response::*`, `server_axum::http`, or any existing explicit import path.

## Alternatives considered

1. **Add a Core `yetlet::prelude`.** Rejected because the Core root already re-exports `Attribute`, `Element` and `View`; a second path would duplicate rather than simplify the current Core API.
2. **Re-export all Axum symbols.** Rejected because it would make the prelude an unstable mirror of an upstream crate and obscure which native capabilities Yetlet deliberately supports.
3. **Re-export the whole `http` module.** Rejected for the prelude; selected common HTTP types are enough for ordinary handlers, while `server_axum::http` remains available for advanced use.
4. **Export `Json` from both `extract` and `response`.** Rejected because glob imports would become ambiguous. The prelude exports one `Json`; the existing explicit module paths remain intact.
5. **Omit `Form`.** Rejected because it is used by an official sample and is already available through Axum 0.8.9's selected default `form` feature. Re-exporting it is a small, dependency-neutral improvement to the existing extractor facade.
6. **Require explicit imports for every server symbol.** Rejected as the only documented path because the same small set of Yetlet and Axum types is repeated in ordinary application files.

## Consequences

An application can import the common Yetlet server surface with `use yetlet::server_axum::prelude::*;`, while still using ordinary Rust modules and explicit Axum/Tower imports for advanced behavior. Existing code does not need to migrate. Core-only applications do not gain a new module, and browser builds do not gain native server symbols.

The prelude is intentionally curated, not exhaustive. Adding another name later requires a demonstrated common application need and a source/feature audit; a new Axum symbol is not automatically included just because Axum exposes it.

## Review evidence

- Independent saved Reviewer Job: `wc_job_leT3lB8xD2KJSSZS`.
- Terminal state: completed; exit code `0`.
- Reviewer trace: role `reviewer`, model `gpt-6-astra`, effective effort `high`; model/role/effort matches and parent inheritance consistency all `true`.
- Reviewer result: `overall_status = pass`; `issues = []`; `required_followups = []`; `optional_notes = []`.
- Ignored evidence: `.pipeline-output/yt252-application-prelude-contract-review-20261111/reviewer-output.json` and `reviewer-trace.json`.

## Acceptance boundary

This ADR accepts the export contract only. YT-254 implements the module and the additive `Form` re-export, with compile/doctest and explicit-path compatibility coverage. YT-255–257 may adopt the prelude only after the implementation receives its own independent Reviewer PASS. No Core feature, default dependency, runtime, HTTP behavior, published SDK alpha.7, CLI 0.3.1, or Reader Edition is changed by this ADR.

No product code, dependency, feature selection, CLI template, release artifact, Git tag or publication is changed by this ADR.
