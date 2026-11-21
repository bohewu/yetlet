# Yetlet naming guide

Date: 2026-10-22. Scope: Experimental Rust APIs, CLI support vocabulary and
application settings. This guide records the owner's single-canonical-name
decision; [the audit response](#dated-naming-audit-dispositions) records its implementation.

## Rust conventions and semantic precision

Follow the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/naming.html)
for casing, conversions, getters and feature names. Use `UpperCamelCase` for
types, traits and enum variants, `snake_case` for functions, fields and modules,
and `SCREAMING_SNAKE_CASE` for constants. These conventions guide spelling;
the exact domain vocabulary below is a Yetlet decision.

Use ordinary constructors such as `new` and descriptive operations. Accessors
normally omit `get_`; distinguish borrowing (`principal`) from consumption
(`into_principal`). Use `as_`, `to_` and `into_` according to ownership and work
performed. Preserve domain-specific sync/async semantics: Memory operations are
synchronous, Redis operations perform asynchronous IO. A universal `Async`
suffix would add no useful distinction to the current contracts.

Use the crate, module or receiver as context. `yetlet::rate_limit::Policy`,
`Budget` and `Key` are valid concise names; applications can import
`Policy as RateLimitPolicy` where several policies coexist. A `Policy` is
reusable configuration; a `Budget` owns accounting state and its clones share
that state. Do not rename both to the same concept or add a framework-wide
policy registry. `build_rate_limit_policy` is appropriate in an application
that builds several kinds of policy.

Names must cover their actual inputs. `yetlet::config::Error::Key` applies across
configuration sources, while TOML syntax and application deserialization retain
their separate error categories. `yetlet::signing::PublicKeySet` can contain
certificate-backed or certificate-free `PublicMaterial`; membership alone does
not establish trust. `InvalidAuthenticationInput` and `InvalidRateLimitInput`
cover their existing construction boundaries without implying authentication
failure or startup-only configuration failure.

## Shared vocabulary

| Term | Current meaning |
|---|---|
| Settings loader | Opt-in `yetlet::config::Loader`, loading typed startup settings from base/profile TOML and explicit overrides; no `SettingsProvider` service or trait |
| Secret provider | Separately selected async `yetlet::secrets::SecretProvider` lookup contract, explicitly injected by the application |
| `load_with_env` | Load ordinary sources and apply a supplied environment snapshot; it is not environment-only loading |
| `authentication_name` | Application-configured resolver/registration label, compared exactly; not an HTTP challenge scheme, issuer, storage backend or permission |
| Authentication | Establish verified identity through an explicit resolver and native extractor |
| Authorization | Application-owned decision permitting a resource/action using current authoritative policy; identity extraction alone never grants it |
| `refill_interval` | Time to replenish one rate-limit cell, not a full burst or a counting window |
| Native watch | Rust rebuild and process restart through the selected Watchexec path |
| Browser watch | Browser-target Rust/WASM rebuild and full reload, requiring frontend `auto_start` and `auto_build` |
| Vite HMR | The separately documented frontend update path; not a promise of state-preserving Rust hot reload |
| Redis / Valkey | Redis names the existing protocol/provider crate; Valkey names the server selected by the Admin deployment. Do not rename a protocol API merely to match one deployment |

Keep meaningful differences: Session is usable anonymously, cookie primitives
do not establish identity, local account mapping is not OIDC protocol
verification, and an application integration is not a generic framework adapter.
Use the [current capability map](CAPABILITIES.md) to identify the actual carrier.

## CLI and settings vocabulary

Keep user-facing argv separate from Rust support fields and persisted formats:

| CLI option | `Cli::New` field / support concept |
|---|---|
| `--service` | `service`; the scaffold operation is `create_service` |
| `--settings` | `runtime_settings`; selects the application's runtime settings loader |
| `--tailwind` | `tailwind` |
| `--vite` | `vite`; implies runtime settings |

All templates already have public CLI settings, so `runtime_settings` is more
precise than an unqualified `settings` flag inside the support enum. Existing
removed-option diagnostics may name `--cloud` and `--with-config` to explain
migration. Historical fixture directories and the existing `cloud-profile`
metadata contract are distinct from Rust field names and remain explicit.
The [CLI guide](CLI-GUIDE.md#current-command-and-capability-inventory)
owns current execution and generator limits.

External settings are a separate compatibility boundary. Rust
`refill_interval_ms` deliberately maps through Serde to TOML
`policy.interval_ms` and environment `RATE_LIMIT_POLICY__INTERVAL_MS`. Those
remain the sole interval wire names. No duplicate key, alias map, precedence
fallback or hidden environment switch is introduced. Any future wire rename
needs its own explicit migration decision.

## Pre-public change discipline

Before publication, select one canonical Rust name and migrate repository
consumers, examples, templates and current documentation in the same bounded
change. Do not keep stale wrappers or re-exports merely to accumulate synonyms.
Record intentional source breaks and preserve behavior, validation, redaction,
feature isolation and external formats unless separately selected. Public named
enum fields and variants are source API too; parameter/private-field renames
have a different impact.

The owner selected these intentional Rust source migrations after checkpoint
`83f417e`; no compatibility aliases remain for these entries:

| Former Rust entry | Canonical entry |
|---|---|
| `Loader::load_from_env` | `Loader::load_with_env` |
| `Loader::load_from_env_with_secrets` | `Loader::load_with_env_and_secrets` |
| `Error::EnvironmentKey` | `yetlet::config::Error::Key` |
| `Principal::scheme` | `Principal::authentication_name` |
| `MAX_SCHEME_BYTES` | `MAX_AUTHENTICATION_NAME_BYTES` |
| `InvalidPrincipal` | `InvalidAuthenticationInput` |
| Rate-limit `InvalidConfiguration` | `InvalidRateLimitInput` |
| Signing `PublicCertificates` | `PublicKeySet` |
| `Cli::New::{cloud, with_config, with_tailwind}` | `Cli::New::{service, runtime_settings, tailwind}` |
| `scaffold::create_cloud` | `scaffold::create_service` |

An enum variant cannot be transparently preserved by a unit-struct re-export
technique or an associated constant: variant imports and exhaustive patterns
must be considered. Here repository consumers migrate to `Error::Key`; no
second variant is introduced. Fixed Display diagnostics remain redacted and
unchanged. Debug/type-name text is not a persisted wire format.

Retain dated plans, verification commands, failures and Reviewer results as
historical evidence. Label superseded choices and link the current guide;
do not rewrite an old PASS to claim it covers a later migration. The earlier
[compatibility checkpoint](NAMING-COMPATIBILITY.md) remains such a record.
Future public stability promises still require the separate
[compatibility](COMPATIBILITY.md) and release decisions.


## Application opt-in options — YT-185

Use application-owned typed instance options for composing the YT-18x opt-ins:
`BoardOptions` with `Board::open_with_options` / `open_with_oidc_and_options`,
and `DatabaseOptions` with Shop `Database::open_with_options`. Ordinary `open`
and the existing pre-18x frontend/Session conveniences remain default-off.
Use descriptive fields `write_authorization`, `request_tracing`, `catalog_cache`;
avoid positional flags for another capability or a constructor per combination.
Do not add a framework-wide options trait/registry or force identical IO shape.
Board remains async; Shop opening remains synchronous.

[The contract and migration table](https://github.com/bohewu/yetlet/blob/4792565e6cff7a94c8c2c860d1858f840f122cbe/docs/YT-185-OPT-IN-OPTIONS.md) record five removed
capability-specific constructors, without stale aliases. Dated YT-181/182/183
contracts and verification retain their former entry points and accepted
results as history; current sample guides describe the options interface.
Cargo features select compiled availability; startup values select instances.
Options are independent of their settings source. The existing Admin Settings
loader and Shop environment keys are unchanged, and the loader remains optional
for other consumers. Transport URI/TLS and YT-184's experiment fixture inputs
keep their distinct native roles.

## Two-package module paths — YT-211

The unreleased alpha.5 source replaces the former sixteen library crates with
`yetlet` and `yetlet-browser`. Import shared types from `yetlet::view` (or root
`yetlet::View`), native capabilities from qualified modules such as
`yetlet::auth`, and browser APIs from `yetlet_browser::dom`. HTTP Action execution
is `yetlet_browser::actions::HttpExecutor`; portable contracts remain
`yetlet::actions`. There are no old-crate compatibility facades.

Feature names use Cargo kebab-case: `server-axum`, `auth-session`,
`session-redis-tls`, `rate-limit`, `dom-keyed`, `wgpu-browser`. Rust modules use
snake_case. Existing type/method names, application feature names, settings keys
and wire formats remain unchanged. Historical release documentation keeps its
original names. See the [migration map](COMPATIBILITY.md#two-package-feature-and-target-contract).

## Native Window API semantic names — YT-282 naming follow-up

The optional `yetlet_hosts::native_winit::WindowSlot` exposes
`window()` as a borrowed `&Window` and `shared_window()` as a cloned
`Arc<Window>` for an independently selected renderer. Avoid the former
`window_handle()` spelling, which can be mistaken for a raw OS Window handle.
`redraw_event_count()` is a monotonically increasing count of accepted
Window redraw events, **not** a collection of events or the number of GPU
Presents. These two pre-release Rust names intentionally replace the previous
`window_handle()` and `redraw_events()` methods with **no aliases**; the
selected Native examples and external Consumer use the canonical spellings.
Original runtime diagnostic output labels are historical observation formats,
not Rust public method names, and remain unchanged. This does not change
Window ownership, EventLoop affinity, readiness or renderer lifecycle.

## Proposed Host lifecycle vocabulary — YT-286

The [layered Host design](M28-HOST-RESOURCE-LIFECYCLE-DESIGN.md) and
[ADR 0014](adr/0014-layered-host-resource-lifecycle.md) originally proposed
Application/Host defaults with ResourceGroup customization over one lifecycle
engine. The next-alpha vocabulary below supersedes that outer name.
The authorized [YT-286-A private prototype](../examples/host-lifecycle-research/README.md)
uses `Application::new`, `with_host`, `ResourceGroup`, `with_resource`,
`request_shutdown` and `shutdown_until`. The earlier `Application::builder`
spelling was illustrative and is not retained as an alias. These names are not
yet an accepted public replacement for the current Composition API.

B1/B2 adds bounded source interfaces with distinct responsibilities:

| Layer and spelling | Meaning |
|---|---|
| Private `ResourceGroup::with_resource(value)` / `WebHost::with_resource(value)` | Retain a supplied passive owned or borrowed value; the A ownership contract remains. |
| Private `with_managed_resource(label, descriptor)` | Register an inert resource whose `acquire` and `release_within` have explicit ownership contracts. |
| Core `Builder::with_resource(label, value, release)` | Install a typed final-release dependency; registration does not acquire it. |
| Core `Builder::with_activity` | Attach an original activity with a post-join final-release callback. |
| Core `Builder::with_group` | Nest typed parts under the same coordinator and failure ledger. |
| `ReleaseCompletion::Pending` | Cleanup is incomplete and its original owner remains retained. |
| `ReleaseCompletion::Released(result)` | Cleanup is terminal; an error in `result` records failure without repeating release. |
| `ReleaseStatus` | Read the latched Pending/Succeeded/Failed release outcome. |

An outer error from `release_within` means retryable progress failure and retains
ownership. It differs from terminal `Released(Err(...))`. The sealed
`ResourceParts` bound is generic storage support, not a custom coordinator
extension interface. These additions do not rename the accepted Local/Worker
API or promote the private Host facade.

The proposed rule is to name resource ownership, acquisition and shutdown
operations by intent. Do not use `required`, `foreground` or `ready/worker`
as unexplained quickstart categories. A `try_` pair needs a documented
operation-specific distinction; Result alone does not require that prefix.
Deferred acquisition and dependency-safe release are behavioral additions
requiring implementation proof, not consequences of renaming.

## Next-alpha Host vocabulary — YT-287

The [feature-first contract](M28-HOST-RESOURCE-LIFECYCLE-DESIGN.md#15-yt-287--feature-first-architecture-and-driver-contract)
selects these intentions for the next API. The [YT-288 Core surface](M28-HOST-RESOURCE-LIFECYCLE-DESIGN.md#16-yt-288--built-in-core-host-lifetimes)
implements HostGroup, Host and ResourceGroup; remaining adapter signatures are
proof-gated. The private Application spelling above is historical.

| Target name / operation | Developer intent and lifecycle meaning |
|---|---|
| HostGroup | Group high-level Host lifetimes using the built-in Core coordinator. |
| yetlet::web::App | Configure an HTTP service; retain Router/Tower interoperability. |
| yetlet::web::WebHost | Run an HTTP service as a participant in a HostGroup lifetime. App::into_host is the target bridge after missing state is resolved. |
| ResourceGroup | Customize typed owned/borrowed resources and explicit acquisition/release behavior. |
| new / with_host / build | Describe, register and validate an inert plan; none starts a listener, thread or native loop. |
| with_address | Configure a future Web bind address; actual acquisition belongs to the driver. |
| serve / serve_on | Execute ordinary HTTP serving on caller-owned Tokio; a cancelled consuming future does not promise managed cleanup. |
| run / run_async | Drive a compatible HostGroup through its documented synchronous/native or async driver. Managed async driving borrows a retained owner; the adapter must prove cancellation-safe storage. |
| request_shutdown / shutdown_until | Request stop or advance shutdown toward terminal completion; Pending retains the original obligations. |
| with_resource / with_managed_resource | Retain a passive Rust value or register an inert descriptor with explicit acquire/release semantics; do not conflate the two. |

The current Core owner is BuiltHostGroup. poll_start reports readiness progress;
poll_run advances the lifetime; poll_shutdown advances original completion and
final release. run_async borrows the owner and uses the caller's executor.
HostActivity names the custom activity contract, ManagedResource names explicit
acquire/release management, and HostFailure retains the original typed error.
No generic synchronous run loop is provided by this slice. Existing advanced
Builder callbacks retain their established phase-specific behavior while the
new Host path avoids the legacy required_* vocabulary.

Ordinary participation needs no required marker. try_* is reserved for a
documented operation-specific distinction and never means optional success,
swallowed failure or automatic fallback. Do not substitute ready/worker or
foreground for a precise resource/driver contract.

Migrate the legacy required_* callbacks by their actual lifecycle phase;
legacy local close-before-worker behavior is not equivalent to post-join
resource release. Keep Composition as advanced engine vocabulary, not the
unexplained quickstart entry. No permanent Application or server_axum aliases
are selected for the breaking target; do not rewrite historical receipts.

## Dated naming audit dispositions

The 2026-10-22 audit attachment SHA256
`2928a7923ad63ad42441ae2374272953cb36a102f78dbd8fecaec720af843229`
matched 30 reported source hashes at checkpoint
`83f417ea12dfc4cc2aa5c05e0f712706ba78565a`. The owner selected canonical
pre-public Rust names over the attachment's alias suggestions. Delivery became
`8c8fba3ae2e84c7493d56b74d0ccc8d8f26920ed` with separate subsequent generator
repair. These historical dispositions preserve the accepted scope.

| Finding | Accepted disposition |
|---|---|
| N01 | Help distinguishes native Watchexec rebuild/restart, browser Rust/Wasm full reload requiring both frontend auto flags, and Vite HMR. Inspect schema/runtime behavior was unchanged |
| N02 | Root summary links source/platform matrices and discloses the settings-profile generator refusal discovered by the focused probe |
| N03 | Error::Key replaces EnvironmentKey with source-neutral rustdoc and unchanged Display. Duplicate TOML syntax and app deserialization retain Toml/Settings |
| N04 | PublicKeySet replaces PublicCertificates. Certificate-backed/free PublicMaterial and explicit trust/pins retain crypto behavior |
| N05 | Authentication resolves/extracts verified identity. Resource/action authorization stays application-owned under ADR0005 |
| N06 | Capability map distinguishes implemented carriers, app-owned OIDC/Valkey/Auth, research decisions and unselected candidates. Cookie primitives, Memory bridge and generic Cookie Auth differ |
| N07 | M9 API inventory stays dated, with supplements for Config/Secrets/Signing/Auth/rate-limit and the original nine release candidates preserved |
| N08 | Cli::New service/runtime_settings/tailwind/vite and create_service align support fields. Argv, removed-option diagnostics and real cloud-profile metadata retain their meanings |

Stale Loader/Auth/rate-limit wrappers and re-exports were removed. Public-naming
tests replaced compatibility-only test targets. These intentional enum/field
source breaks preserved settings TOMLs, dependencies, locks, Display text,
middleware and cryptography. Rust refill_interval_ms still uses the sole
policy.interval_ms / RATE_LIMIT_POLICY__INTERVAL_MS wire spelling.
[Focused results and the original generator/reviewer failures](QUALIFICATION-MATRIX.md#naming-audit-results-and-generator-failure)
retain their dated source and OpenSSL/platform boundaries.
