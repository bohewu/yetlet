# YT-095 — Rust dependency composition and optional DI

Date: 2026-10-20. Source checkout: `649be69`, after the reviewed YT-106
CodeMirror slice. This is the owner-selected research comparison, with no
application, dependency, settings, CLI or framework API implementation.
Independent Reviewer `/root/yt095_composition_research_review_oct20` PASS has
zero issues or required followups. YT-095 is Done at research-only scope.

## Recommendation

Keep ordinary Rust constructors and native Axum `State` as the default. Use
`FromRef` when a handler needs a smaller part of shared state; keep settings
loading and provider selection in application startup. Use a concrete type or
generic parameter unless an actual runtime substitution needs a trait object.
An `Arc` shares an instance; it does not discover, construct or register it.

The existing Starter, signing and Admin consumers do not demonstrate a missing
container capability. Their real differences are service contracts, startup
work and resource lifetimes. Neither shaku nor dill eliminates that work.
No `yetlet-di`, service locator, registration API or new ADR is justified by
this comparison. This retains [ADR 0005](adr/0005-opt-in-capability-modules.md)
and [ADR 0006](adr/0006-handler-first-mvc-optional.md).

## Current consumers and concrete costs

These are current-source observations, not a proposed replacement application:

| Consumer | Actual composition and substitution | Remaining application responsibility |
| --- | --- | --- |
| [YT-092 Starter startup](../examples/production-starter/src/main.rs) and [web state](../examples/production-starter/src/web.rs) | `config::load`, `Store::open`, then `web::app(store, api_token, readiness)`. `WebState` contains the store and two Arc-backed values. Tests construct the router with a temporary store and synthetic token | Configuration validation, scoped anonymous sessions, API authorization, readiness and graceful shutdown. Store is concrete; there is no existing arbitrary repository trait or injected backend factory |
| [YT-093 signing reference](../examples/signing/src/main.rs) and [service](../crates/yetlet/src/signing/mod.rs) | `SigningService<B, P>::new(backend, public, policy)` and `Verifier<P>::new(public, policy)` use generic provider contracts. The reference passes a `LocalProvider` and public-only certificates | Material parsing, explicit key versions, trust and active/retired/revoked policy. Verification-only callers receive no signing backend. The standalone demonstration reads a disposable password from an environment variable; it does not use the settings loader |
| [Admin startup](../examples/admin-dashboard/src/main.rs) and [OIDC setup](../examples/admin-dashboard/src/oidc.rs) | Settings select assets, session backend and authentication. OIDC setup receives an injected `Arc<dyn SecretProvider>` and resolves only its selected secret before discovery. Board owns its router and drain handles | Feature/setting opt-in, asynchronous construction, fixed safe failures and explicit shutdown. Admin currently adapts selected loaded settings through `SettingsSecrets`; it does not automatically use the optional loader-to-provider integration |
| [SecretProvider service example](../crates/yetlet/examples/config/secrets_state.rs) | `DeliveryService::new(Arc<dyn SecretProvider>)`; the same constructor accepts `SettingsSecrets` or a test replacement. Web state owns `Arc<DeliveryService>` and a public message | Lookup names, required/nonempty checks and the decision to fetch again or rebuild a client. A copied secret remains a snapshot |
| [Settings-to-service example](../crates/yetlet/examples/config/secret_provider.rs) | Optional `SecretBinding` resolution produces typed protected settings. The root passes only token/attempts to `delivery::Client`, then passes the client and public message to web state | Construct the source independently, validate settings, and select each module's inputs. No root settings/provider is given to the handler |

Static counts provide a small reproducible baseline for authoring cost:

- Starter `web.rs` lines 32–37 declare its three-field state in six physical
  lines; lines 50–54 construct it in five. That file has ten handler/middleware
  `State(...) : State<...>` parameters.
- Board `lib.rs` lines 115–124 declare seven state fields in ten physical lines;
  lines 290–299 construct them in ten. Its `lib.rs`, `triage.rs` and `oidc.rs`
  contain 11, two and two state parameters respectively.
- Those four files contain zero `FromRef` implementations. Their existing
  handlers receive the complete application state. This is a narrowing
  opportunity, not evidence of a missing injection system.
- The five inspected lockfiles (Starter, Admin, signing reference,
  `yetlet-config`, `yetlet-secrets`) contain neither shaku nor dill.

Counts include braces and attributes within the stated ranges. Parameter counts
use `State\([^)]*\):\s*State<`; they are not service counts, full application
size, a before/after implementation or a performance benchmark. The native
approach adds zero DI dependencies. Container alternatives were not installed,
compiled or timed; binary size, cold/warm build time and runtime lookup costs
remain unmeasured. This study claims no speed or total-line-count winner.

## Smallest practical composition pattern

1. At startup, load the existing typed settings. If selected, construct a
   read-only provider independently and use the existing opt-in
   [SecretBinding integration](SETTINGS-SECRET-PROVIDER.md). Otherwise ordinary
   loader/Development-secret precedence stays unchanged.
2. Validate before serving, choose concrete implementations and construct each
   service with its required settings/secret/capability. Await discovery or
   connection setup explicitly. A factory returning `Result` is enough; it
   need not become a common provider framework.
3. Share long-lived service handles through application state. Prefer existing
   cheaply cloneable handles; add an outer `Arc` when needed, not mechanically
   around every field.
4. Pass a service or small substate to the handler that uses it. Native
   `FromRef<AppState>` can clone that handle without reconstructing the service.
   Import it from `axum::extract`; Yetlet's convenience extractor module does
   not currently re-export `FromRef`. Manual implementation needs no derive
   feature or new Yetlet API.
5. Keep request-derived identity, CSRF/session and other request-local values
   in their existing extractors/extensions. Preserve explicit route scopes and
   the application's stop/drain sequence.

The locked local Axum 0.8.9 `State` implementation requires
`InnerState: FromRef<OuterState>`. Yetlet's `App::get` uses native `Handler<T, S>`
and `with_state` delegates to `Router::with_state`; source inspection finds no
extra container requirement. The official [Axum State guide](https://docs.rs/axum/0.8.8/axum/extract/struct.State.html#substates)
documents the same bound for 0.8.8. The exact 0.8.9 State documentation URL was
unavailable through the research browser; its cached local source was checked.
This study does not add or claim a newly compiled Yetlet substate example.

### Choose the dependency shape from its contract

| Need | Smallest existing Rust mechanism | Limit to preserve |
| --- | --- | --- |
| One implementation, configuration values or a module-private service | Concrete constructor parameters; references during setup | No trait is necessary solely to call this dependency injection |
| Replaceable implementation selected by the caller's type | Generic `T: Trait`, as in signing | Changing the backend changes the instantiated service type; do not claim runtime trait-object interchangeability |
| Runtime replacement behind a shared compatible read interface | `Arc<dyn SecretProvider>` | Only dyn-compatible traits work. The boxed Send future in this contract is deliberate |
| Finite runtime choices with different concrete APIs | App-owned enum, as in Board Memory/Valkey sessions | Session managers are concrete; this checkout has no common pluggable session-store trait. The enum preserves their different errors and fail-closed behavior |
| Handler needs one service from a larger state | Manual native `FromRef` and `State<Substate>` | A local wrapper is useful when two same-typed values have different roles. Narrow extraction is code organization, not a sandbox |

`SigningBackend` and `PublicKeySource` return `impl Future`, so they are not
dyn-compatible in their present form. `Arc<dyn SigningBackend>` is not a drop-in
substitute, and `Arc<T>` alone does not implement those traits by forwarding.
The [Rust trait reference](https://doc.rust-lang.org/reference/items/traits.html#dyn-compatibility)
explains the opaque-return restriction. Keep existing generic constructors;
select an application enum or a separately justified object-safe adapter only
if a real runtime-switching consumer appears. A DI container cannot remove this
type-contract distinction. No such adapter is selected here.

## Optional container comparison

Versions below were read from official published documentation on 2026-10-20.
Neither is a Rust standard-library/official Yetlet component or a qualified
dependency of this checkout. The comparison is a source/design assessment,
not a compiled integration or supply-chain endorsement.

| Criterion | Constructors + State/FromRef | shaku 0.6.3 | dill 0.17.2 |
| --- | --- | --- | --- |
| Assembly/code shape | Existing structs, constructor calls and explicit state fields | Components/interfaces plus module declarations; derive injection annotations replace some manual wiring | Component builders/annotations plus Catalog registration and explicit resolution |
| Type/failure checking | Constructor types and `FromRef` bounds checked by Rust; fallible construction returns application errors | Typed module relationships; runtime settings/material can still fail. Documented missing required component parameters can panic | Rust types describe interfaces, but resolution is fallible at runtime; `CatalogBuilder::validate` checks registered graph dependencies/scopes |
| Lifetime | Caller chooses ownership and handle reuse; request data stays request-local | Components live with the module; providers produce shorter-lived values | Catalog scopes include transient and singleton; chained catalogs can add request-specific values |
| Tests | Pass fake provider/concrete fixture to existing constructors; no global replacement | Builder supports component/provider overrides | Bind test implementations or prebuilt values; preserve error handling for missing/ambiguous bindings |
| Build/maintenance cost | No added DI package; ordinary code remains visible | Default derives add a proc-macro dependency; an application must maintain module declarations | Adds Catalog/builder machinery and `dill-impl` macros; registration and validation become application obligations |
| Async startup/shutdown | Explicit `await`, validation and stop/drain already exist | Can receive prebuilt instances; no evidence that adopting it replaces existing async initialization/drain contracts | Can receive prebuilt values; catalog lookup alone does not qualify resource startup or orderly async shutdown |
| Fit here | Covers the observed consumers | Reasonable optional application choice for a larger stable graph; no demonstrated saving here | Reasonable optional application choice for a dynamic graph; introduces runtime resolution semantics absent from this need |

The [shaku guide](https://docs.rs/shaku/0.6.3/shaku/guide/index.html),
[module overrides](https://docs.rs/shaku/0.6.3/shaku/struct.ModuleBuilder.html),
[provider lifetime guide](https://docs.rs/shaku/0.6.3/shaku/guide/provider/index.html)
and [published manifest](https://docs.rs/crate/shaku/0.6.3/source/Cargo.toml)
support its column. The [dill guide](https://docs.rs/dill/0.17.2/dill/),
[Catalog](https://docs.rs/dill/0.17.2/dill/struct.Catalog.html),
[validation](https://docs.rs/dill/0.17.2/dill/struct.CatalogBuilder.html#method.validate)
and [package dependency metadata](https://docs.rs/crate/dill/0.17.2)
support its column. Build-time impact is inferred only as additional machinery;
no timings, transitive package total or MSRV/platform qualification is claimed.
An application choosing either later should resolve services at its assembly
boundary and pass narrow handles onward, rather than give handlers a Catalog.

## Ownership, secrets and lifecycle

One instance shared by clones of its `Arc` is scoped to the root that created
it. Two test apps can construct independent instances; two replicas still have
separate process memory. Global `static`/`OnceLock` is unnecessary for ordinary
service wiring. Container singleton scope also does not make a service shared
across replicas or make its interior state thread-safe automatically.

`Send + Sync` bounds and synchronization remain the service's responsibility.
Board Database clones share its connection/gate; Starter Store clones share a
path/gate and open connections per operation. Do not infer connection lifetime
from the number of state handles. Dropping an Arc is not an asynchronous drain:
retain Board's explicit session/database shutdown and Starter readiness/drain.

Keep the existing read/administration split. The app reader gets only required
names or selected protected values; operator CLI/deployment controls own secret
and key changes. A substate or `SecretString` reduces accidental exposure; it
does not provide process isolation. Signing and verification capabilities stay
separate, with explicit trust and no private material in verification-only state.
Public SSR/browser data must still be selected explicitly.

Do not make `SettingsSecrets` and the loader bootstrap each other. They are
opposite optional adapters, as [the settings integration](SETTINGS-SECRET-PROVIDER.md)
already specifies. `SettingsSecrets::replace` affects later reads through that
handle, not credentials copied into an existing OIDC/client instance. Injection
does not add reload, source revisions, rotation permissions, or confirmation
that an external service accepts an old credential.

## Gaps, next decision and minimum verification

No blocking framework gap was established. The concrete improvement opportunity
is documentation of native narrow substates, followed only if useful by a
separately selected application trial. Starter/API authorization and Admin
handlers currently receive broader state than they need; this is not a claim
of a demonstrated authorization failure. Board's two session-aware public
constructors have six parameters; an app-local options struct could be compared
if another selection makes call sites confusing. Neither observation justifies
a registration framework or changing provider contracts today.

Reconsider an optional container only when real consumers repeatedly duplicate
a substantial dependency graph or need dynamic registration that constructors
and a finite enum cannot express clearly. Compare that actual graph, failure
behavior, code size and clean/warm builds before selecting a package. A second
example or a longer parameter list alone does not establish the requirement.

If a native substate trial is selected later, use existing seams only:

| Scenario | Minimum focused evidence |
| --- | --- |
| Shared service | Two extracted handles reach the same underlying instance; separately built apps remain independent |
| Substitution and typing | Same consumer accepts a fake SecretProvider; a handler uses only its selected substate. Keep signing generic and session selection finite |
| Startup/errors | Invalid required settings/provider failure returns the existing safe error before serving that service; no fallback or partial success is invented |
| Request boundary | Existing per-request session, permissions, CSRF and route scoping remain intact |
| Lifetime and secrets | Existing drain sequence remains explicit; selected synthetic values stay out of SSR/logs/browser state. No container is required for cleanup |

Existing source tests were inspected, not rerun: Starter temporary-store HTTP
fixtures, SecretProvider's same-service custom-provider/SSR tests, settings'
narrow-module example, and signing's substitute-source/backend rejection tests.
They prove neither a container integration nor a new FromRef consumer. This
documentation-only slice requires documentation/link/whitespace checks and
independent review, not a fresh browser, Windows, cloud or performance matrix.

Validation on 2026-10-20: `cargo xtask check docs` passed with exit 0 (internal
links, Markdown structure, whitespace and backlog DAG); `git diff --check`
passed with exit 0. The independent reviewer inspected the actual research and
consumer sources, verified the static counts and primary documentation, and
independently passed the whitespace check. No runtime or container benchmark was
run or required for this research-only closeout.

The next independently unblockable backlog item is YT-102's remaining DX audit,
if selected: reconcile current CLI/settings/watch behavior against its older
research row before proposing more code. YT-096 release qualification remains a
separate maintainer decision. This research does not authorize implementation,
publication, deployment or a new public DI contract.
