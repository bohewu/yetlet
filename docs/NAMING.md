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
