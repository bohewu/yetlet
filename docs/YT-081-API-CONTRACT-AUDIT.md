# YT-081 — Cross-Crate API Contract Consistency Audit

**Date:** 2026-10-14. **Source:** original Ubuntu master checkout `e499d2438d9cf5f5c6e1095d2fcff4b1428e15ef`.
**Scope:** nine original unpublished `0.1.0` library candidates, the separate experimental Redis provider, and the approved Experimental `yetlet-wgpu` crate.
**Assessment:** no demonstrated cross-crate P0/P1 contract failure on the selected exercised consumer paths. The owner then authorized a targeted Experimental `yetlet-wgpu` API refinement. Its CPU errors and logical retirement now have precise typed/boolean contracts, while unrelated P2 design boundaries remain intentionally independent. [Refinement evidence](YT-081-WGPU-API-REFINEMENT.md) records the new candidate and the need for a fresh source-bound reviewer.

The original nine candidates remain the only YT-020 release inventory. The Redis and GPU crates are unpublished, independent nested workspaces, and no broad platform, stability or compatibility promise is implied. See [API Contracts](API-CONTRACTS.md), [Compatibility](COMPATIBILITY.md) and the GPU [support boundary](../crates/yetlet-wgpu/SUPPORT.md).

## 1. Implemented contract matrix

| Area | Source-grounded public shape | Assessment |
| --- | --- | --- |
| Core/HTML | `yetlet-core::View` and validated binding IDs; `yetlet-html::{render_to_string,RenderError}` | Consistent renderer-neutral values, safe HTML and explicit fallibility |
| Server / middleware | `App<S>`, `get/post/page/route/group/configure/try_configure/into_router`, `RequestContext<S>`, and Axum/Tower `Next` and `from_fn` | Handler-first without Controller/DI or parallel router; native request/body ownership and return types retained |
| Actions | `Command`, `ActionExecutor<C>::execute`, `LocalExecutor`, `HttpExecutor`, and `ActionError` | Typed command stays separate from DOM event; HTTP boundary explicit and same-origin validated; caller `execute().await` is uniform |
| Browser DOM | `yetlet-dom::{validate,mount}`, opt-in `KeyedChildren`, `DomEventListener` | Browser `JsValue` errors at JS boundaries; keyed ownership explicit; SSR remains independent |
| Local graph | `LocalGraph/GraphHandle/Subscription`, synchronous `dispatch/batch/snapshot` | Strong owner, weak handles, no hidden async scheduler or mandatory graph |
| Request correlation | `RequestIds::{new,try_new,scoped,global}` and `RequestId` | Optional numeric process-local ID; `global` deliberately finalizes Axum Router, not a builder |
| Memory session | `SessionManager::new`; sync `Session::{read,try_update,rotate,replace_and_rotate,destroy}` | In-process mutex and clone-then-commit semantics fit sync Rust methods |
| Redis session | `RedisSessionManager::connect().await`; sync `open`/`scoped`; async `RedisSession::{read,try_update,rotate,destroy}`; `OutcomeUnknown`/CAS | Native Rust `async fn` is the differentiation; remote status/cancellation is **not** memory-provider behavioral substitutability |
| GPU | `yetlet-wgpu::Batch`, `SceneError`, `PickViewport`, explicit wasm32 `browser::SceneOwner`, `initialize().await`, `completion().await`, `dispose() -> bool` | Typed CPU validation, first-retirement boolean, CPU-only `default = []`, opt-in browser feature, no implicit GPU/SSR dependency or public wgpu type; Experimental only |
| Error ownership | `RenderError`, `HtmlPageError`, `ActionError`, `SessionError`, `RedisSessionError`, `ReactiveError<E>`, `DomPolicyError` and bounded GPU categories | Domain-local typed errors are intentional; no universal `YetletError` is selected |

Rust naming is generally idiomatic: `async fn` communicates asynchronous
work without an `Async` suffix, and a fallible `new` is valid Rust.
`ActionExecutor::execute` stays async even when the `LocalExecutor`
closure itself is synchronous, so consumer code can change transport
without changing `.await` sites.

## 2. Concrete differences requiring an explicit contract decision

| Priority | Verified source/API difference | Risk and disposition |
| --- | --- | --- |
| **Resolved under owner-authorized Experimental refinement: CPU error shape** | `crates/yetlet-wgpu/src/lib.rs` now returns `Result<_, SceneError>` with six non-exhaustive named variants for batch, viewport and picking failures; browser `SceneOwner` still has a Wasm-facing `Result<_, JsValue>` boundary | Callers can match typed CPU variants and receive the former bounded diagnostic messages via `as_str`/`Display`. Wasm translates errors at one explicit helper; browser device/validation errors remain `JsValue`. Actual consumer native/Wasm builds and a separate source-bound Firefox WebGPU test passed. No public stable API is promised |
| **Resolved for Experimental GPU first-retirement signal; other domain shapes intentionally differ** | `Lifetime::retire(&self) -> bool` and `browser::SceneOwner::dispose(&self) -> bool` now report `true` only once, `false` thereafter. `LocalGraph`/`KeyedChildren` also return `bool`, but DOM listeners use `Result<bool, JsValue>` and sessions use `destroy() -> Result` | First-retirement status is explicit without promising that async borrows are settled or the Wasm wrapper can be immediately freed. Do not mechanically normalize unrelated cleanup operations or alter their actual error models |
| **P2 — provider semantics** | Memory `Session::try_update` is synchronous clone/commit; Redis is async CAS, can yield `OutcomeUnknown`, has explicit `create`/`resolve_operation` | Similar names express intent, **not** identical atomicity/cancellation/availability behavior. Redis `open` is cookie parsing only. Do not force Memory into async or add sync Redis wrappers to imitate it. An actual cross-provider compatibility abstraction needs a separate proposal |
| **P2 — documentation classification drift** | Original M9/M10 API classification did not yet list the approved M11 Experimental `yetlet-wgpu`; the independent Redis provider also needs explicit classification. Some M11 title/status text still represented a historical planning/implementation stage | **Documentation fix in YT-081**, no source signature change or false Stable Candidate promotion |
| **P2 — missing public-API breadth** | `crates/yetlet-server-axum/tests/public_api_compat.rs` covers legacy `Page` and additive metadata; Redis runtime tests are fixture-dependent; selected GPU tests cover Firefox but not all platform consumers | Future *public* stability or second GPU consumer should add compile-level compatibility cases and migration notes. This is **not** a present YT-081 or YT-020 authorization and does not replace separate browser qualification |

No P0/P1 compatibility defect was established by the original source audit.
The two owner-selected Experimental GPU debts above now have **actual
source changes** and separately tested evidence: six CPU tests, one
doctest, twelve consumer tests, strict native/Wasm Clippy and a
current-source Firefox real-Wasm PASS (inert disposal `[true,false]`,
configured disposal `[true,true]`, 5 × 120/120 pixels). The
[receipt](YT-081-WGPU-API-REFINEMENT.md) preserves source hashes
and owned cleanup. A [fresh independent **new-source review**](YT-081-WGPU-API-INDEPENDENT-REVIEW.md)
subsequently returned **PASS** (zero issues, zero required followups),
closing the two selected Experimental API debts at this bounded scope.
Other P2 labels remain future design limits, not current failing callers.

## 3. Rules consumers must not infer from similar method names

1. **SSR:** `Page::new` and `App::page` preserve the historical `zh-Hant` default; a different language requires validated `LanguageTag` and `DocumentMetadata`. `render_document*` returns a `Result`; `Page` renders on Axum response conversion.
2. **Axum:** `RequestContext::path_params(&mut self).await` follows extractor internals, while ordinary borrowed request accessors remain synchronous. `route_layer` is registered-route scope; `with_global` and `RequestIds::global` wrap a completed Router.
3. **Session:** Memory `try_update` commits a synchronous cloned draft on success. Redis `try_update(...).await` uses CAS and may require explicit resolution after `OutcomeUnknown`. Never automatically rerun application callbacks after an uncertain write.
4. **Actions:** `execute().await` does not mean local commands require serialization. `HttpExecutor` alone adds the browser JSON/HTTP boundary. Actions do not acquire an implicit event circuit.
5. **GPU:** `Batch` owns checked CPU records/versioning and reports typed `SceneError`; `dispose()` reports the first logical retirement, not async settlement. One `SceneOwner` owns one canvas and device. The application owns semantic record IDs, events, RAF, visibility, HTML fallback and generation bridging. The Wasm wrapper must stay alive until asynchronous borrows settle. `completion().await` does not prove physical paint/FPS. There is no 3D engine/API, stable ABI or speedup guarantee.
6. **Release:** Root original-nine `0.1.x` train does not silently include the nested unpublished Redis or GPU workspaces; default HTML/DOM/HTTP graphs do not acquire GPU or Redis.

## 4. Actual regression evidence for this API survey

All commands ran offline with locked manifests using Rust/Cargo 1.99 in
the original checkout. Logs are retained at
`target/m11-closeout/yt081-20261014/`.

| Command family | Actual result |
| --- | --- |
| Root `cargo test --offline --locked --workspace` | **PASS**, 250 passing test assertions across unit/integration/doctest logs; zero failures |
| Redis `cargo test --offline --locked --manifest-path crates/yetlet-session-redis/Cargo.toml --lib` | **Exit 0, 0 executed; one Redis-fixture-dependent test ignored**. This is a compile/runner check **not fresh Redis integration evidence** |
| Initial `yetlet-wgpu` isolated `cargo test` | Historical PASS, four CPU tests plus one doctest (pre-refinement) |
| Current-source isolated `yetlet-wgpu` tests | **PASS**, six CPU tests plus one doctest, including error variants and repeat retirement |
| Current-source real Firefox/Wasm GPU test | **PASS**, 65 actions, five 120/120 pixel proofs, retirement booleans and actual device resource cleanup; [unique receipt](YT-081-WGPU-API-REFINEMENT.md) |
| Scene consumer isolated `cargo test` | **PASS**, twelve native tests |
| Server `public_api_compat` test | **PASS**, two additive source-compatibility tests |

Root workspace metadata currently lists 34 members; the Redis and GPU
isolated crates are **not** members. Previous Redis real three-application
single-primary proof is retained in [YT-066b3](YT-066B3-VERIFICATION.md),
and Firefox GPU support comes from [YT-079](YT-079-VERIFICATION.md) and
[YT-080](YT-080-VERIFICATION.md); this audit **does not repeat** either
external Redis or GUI execution. A passing native compile/test run is
not a statement of broad public API stability.

## 5. Recommendation

Retain the idiomatic, scoped APIs. The requested small Experimental GPU
typed-error and first-retirement return improvement is implemented as a
separate reviewed change, with source identity and separate reviewer PASS
tracked in [YT-081 GPU API refinement](YT-081-WGPU-API-REFINEMENT.md)
and [independent review](YT-081-WGPU-API-INDEPENDENT-REVIEW.md). Do not
invent a common `YetletError`, force Memory sessions to async, or
normalize unrelated domain-specific `dispose` return types for symmetry. Future changes
to a Stable Candidate still require the compatibility rationale,
migration note, compile fixtures, focused tests and independent review
described in [Compatibility](COMPATIBILITY.md). Public release
YT-020 remains **Blocked**.
