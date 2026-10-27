# YT-114 — Opt-in authentication foundation

**Date:** 2026-10-22. **Base:** `178b273b7568dccf673d392a56f504272b745b2c`.
**Status:** Done at the bounded native foundation scope after independent Reviewer PASS.

After the guarded seven-file YT-113 contract commit, the owner authorized the
smallest YT-114 implementation under the [reviewed contract](YT-113-AUTHENTICATION-CONTRACT.md).
YT-115 remains unselected and Todo. No Session/OIDC adapter, credentials, browser
runtime, application migration, push or release is part of this slice.

## Delivered surface

The new unpublished [yetlet-auth crate](../crates/yetlet-auth/README.md) is an
explicit workspace member, with only Axum in its normal direct dependencies.
Core and the server adapter do not depend on or reexport it. `publish = false`
keeps it outside the original nine-library release candidate; wasm32 use is
explicitly refused. No existing application manifests or runtime files change.

| Selected contract | Implementation |
|---|---|
| Minimal trusted Principal | Private immutable issuer/subject/scheme/optional tenant; verified-input shape constructor, exact issuer+subject comparison, redacted Debug and no Default/serde/View conversion |
| Bounded input | UTF-8 byte limits: issuer 2048, subject 255, scheme 64, tenant 255; reject empty, whitespace-only and control-bearing values, without truncation/normalization. Protocol-specific verification stays with the provider |
| Required/optional typed extraction | `Authenticated` implements FromRequestParts and OptionalFromRequestParts. Only positive anonymous resolution returns optional None; missing private state is generic 500 |
| Native, scoped composition | `Authentication<S>::new` selects scheme, typed state, `Resolver<S>` and anonymous-required response. Native `from_fn_with_state(..., authenticate::<S>)` installs it on selected routes; App::route_layer works unchanged |
| Resolver semantics | A normal borrowed Parts/state function returns `ResolveFuture`, a boxed Send future yielding native `Result<Option<Principal>, Response>`. It runs once; raw Principal extensions are removed, mismatched scheme or duplicate resolution fails closed |
| Explicit failure policy | Resolver errors stop optional and required requests. Required anonymous extraction invokes the explicitly supplied closure. A 401 without a nonempty challenge becomes generic 500; full challenge syntax/applicability and safe local login redirects are application responsibilities |
| Authorization | Ordinary app-owned functions receive Principal plus resource/action and permit only explicit allow. No role schema, policy DSL, filters, DI, Controllers or service locator are added |
| Response/body boundary | Scoped responses, including errors/redirects, are no-store. Ordinary Set-Cookie/Vary/CSP headers survive; original request and response bodies remain streaming |

The Principal constructor asserts **prior verification**, not proof that its
strings are true. Server application/resolver/middleware and selected storage
are trusted. This crate does not sandbox malicious trusted code or make client
headers authoritative. Scheme binding prevents accidental cross-provider
results; tenant and business permissions still need independent checks.

Missing/invalid credentials and provider outages remain distinct native
responses (normally 401 versus 503), supplied by the application. The crate
cannot redact an arbitrary application-created response or make a callback's
challenge truthful. Its own shape/configuration errors contain no identity,
credential or internal type details. It emits no logs or background tasks.

No custom authentication protocol is invented. The compiled example uses an
anonymous resolver and an explicit deny policy. Test fixtures use synthetic
verification outcomes; they do not implement a production credential scheme.
Real verification must come from an explicitly selected maintained provider.

## Focused validation

| Command / evidence | Observed result |
|---|---|
| `cargo test -p yetlet-auth --all-targets --offline` | 15/15 integration tests PASS; library has no unit-test module |
| `cargo test -p yetlet-auth --doc --locked --offline` | README native composition example compiles/runs, 1/1 PASS |
| `cargo clippy -p yetlet-auth --all-targets --locked --offline -- -D warnings` | PASS |
| `cargo fmt --package yetlet-auth -- --check` | PASS |
| `cargo tree --locked --offline --edges normal -p yetlet-core` and `-p yetlet-server-axum` | Before/after outputs byte-identical; no Auth added to baseline native graphs |
| Same normal-graph command for `yetlet-todo-browser` and `yetlet-search-island`, with `--target wasm32-unknown-unknown` | Before/after outputs byte-identical; no Auth/Session/OIDC/Redis provider enters browser graphs |
| `cargo tree --locked --offline --edges normal -p yetlet-auth` | Normal graph excludes Yetlet core/server adapter, Session, Redis and OIDC. Core/server adapter and http-body are test-only dependencies |
| `cargo check -p yetlet-todo-browser -p yetlet-search-island --target wasm32-unknown-unknown --locked --offline` | PASS for both existing browser packages |

The 15 tests cover exact identity and size/control boundaries; redaction;
required/optional anonymous and verified results; invalid/unavailable/malformed
outcomes; missing/duplicate/wrong-scheme resolution; forged public extensions
and identity headers; two sibling schemes and nested policy; matched 404/405
behavior; unmapped/read-only/wrong-tenant denial before mutation; rejection with
zero body polls; original body through RequestContext; explicit login redirect
versus missing challenge; no-store plus existing cookie/Vary/error response;
and deliberate SSR display selection with the unchanged native CSP header and
no serialized principal or scripts. Existing native App, handler State and Page
interoperate without an adapter change.

Two initial test-compilation mistakes (a temporary string borrow and constructing
Page from String instead of View) were corrected in place before the 15-test
PASS. They changed no authentication runtime behavior. No passing prior
Session/OIDC/browser interaction workload was replayed.

**Compile evidence** above includes native and browser-target builds. **Browser
execution/CSP enforcement** was not run: no DOM/loader/script/CSP policy changes
are made. The SSR test checks the native CSP header, not browser enforcement.
There is no real IdP, real Session bridge, Redis service, socket-level deployment,
Windows or cluster qualification in this item.

`cargo xtask check docs` PASS (75 required documents, 123 backlog items;
English prose, required links/structure/whitespace and dependency DAG). Changed
Markdown links/fences/EOF and explicit no-index whitespace checks for all five
new files also pass. The fixed required-document count is not a total Markdown
file count. Independent native Reviewer `/root/yt114_foundation_review_20261022`
returned PASS with zero issues, zero required followups and zero optional notes.
Local trace verified role `reviewer`, model `gpt-6-astra` and effective `high`
effort against the healthy current workspace profile and shared resolver. All
14 target hashes matched through review completion. Only documentation status
and this review record changed afterward; crate source/tests/README/manifests
remain exactly as reviewed.

## Costs and remaining ownership

Only selected routes allocate a boxed resolver future and clone the bounded
principal when extracting it. Configuration and typed state share an Arc;
there is no provider registry, ambient user, installed-by-default middleware,
network call or background thread. No installed-auth latency benchmark or
zero-allocation claim is made. Apps that do not depend on/install Auth retain
the identical normal dependency graphs and execute no Auth layer.

Authentication alone grants no business access. Policy freshness, identity
mapping, CSRF, cookie protections, Session rotation/logout and provider outage
handling stay with applications and the separately authorized YT-115 bridge.
No existing Session sync/async API or YT-094 consumer is altered. YT-116 remains
an adapter study, not an automatic next implementation. YT-096 stays In progress,
YT-020 Blocked, and the immutable release candidate is unchanged.

## Changed files

- `Cargo.toml`, `Cargo.lock`: member registration and its lock entry only; no existing dependency version changes.
- `crates/yetlet-auth/Cargo.toml`, `src/lib.rs`, `tests/authentication.rs`, `README.md`: opt-in native foundation, focused tests and compiled use guide.
- This record plus Backlog, Roadmap, documentation index, Extensions, M21 plan, ADR 0005 and a follow-on link outside the reviewed YT-113 contract body: implementation/status navigation only.

The YT-114 working changes are not committed by this item. Only the preceding
seven-file YT-113 commit was authorized. The next dependency-unblocked item is
YT-115, which awaits separate owner confirmation.

## Subsequent Session slice

The owner subsequently selected [YT-115](YT-115-SESSION-AUTH.md) and the
YT-116/119/120 studies, followed by one Standard security scan. The YT-114
review record above remains bound to its original foundation scope.
