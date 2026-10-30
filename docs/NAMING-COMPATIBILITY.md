# Naming and source-compatibility decisions — 2026-10-22

**Historical checkpoint:** This record describes the accepted 20-file slice
subsequently committed as `83f417ea12dfc4cc2aa5c05e0f712706ba78565a`.
Its compatibility aliases, test targets and no-commit state below describe that
slice only. The owner then selected one canonical name before publication;
the [current naming guide](NAMING.md) and [audit response](NAMING-AUDIT-20261022.md)
supersede the alias-retention decision. Original validation and Reviewer results
below remain historical evidence, not acceptance of the later removal.

**Status:** Done after focused verification and independent Reviewer PASS.
**Source:** `9189b234c01c1951cf88fc0121bafe17be5ddb48`.
**Related:** YT-090 settings loader, YT-114 Auth and YT-117 rate limiting.
**Authority:** Owner-selected five naming clarifications, minimal changes after
checking public API/settings compatibility, updated examples/docs, focused
verification and independent Reviewer PASS. No commit or push.

## Selected vocabulary

| Concern | Selected primary name | Compatibility contract |
|---|---|---|
| Settings provider terminology | Configuration/settings loader: `yetlet_config::Loader` | No `SettingsProvider` trait exists or is added; `SecretProvider` remains the separate optional secret-lookup trait |
| Rate-limit interval | Rust `refill_interval` and example `refill_interval_ms` | `Policy::new` remains positional; Serde maps the new Rust field to existing TOML `policy.interval_ms`; `RATE_LIMIT_POLICY__INTERVAL_MS` stays unchanged |
| Explicit environment snapshot loading | `load_with_env`; optional secret resolution uses `load_with_env_and_secrets` | Existing `load_from_env` and `load_from_env_with_secrets` delegate unchanged; ordinary loading still reads base/profile TOML before applying the supplied environment snapshot |
| Auth registration label | `authentication_name()` and `MAX_AUTHENTICATION_NAME_BYTES` | Existing `scheme()` and `MAX_SCHEME_BYTES` remain equivalent compatibility entry points; constructor argument positions and exact values do not change |
| Shape/construction error types | `InvalidRateLimitInput` and `InvalidAuthenticationInput` | Old `InvalidConfiguration` and `InvalidPrincipal` remain public re-exports of the same unit structs, preserving type/value construction and patterns; Display messages remain unchanged |

These choices follow function/field snake_case, type UpperCamelCase and ordinary
accessor conventions in the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/naming.html).
Exact semantic names are project decisions, not mandated spellings. Short names
with adequate crate/receiver context (`Policy`, `Budget`, `Key`, `Authenticated`)
remain valid. Async signatures/Future return types retain existing Rust semantics;
no universal Async suffix, builder framework or result hierarchy is introduced.

## Public and settings compatibility

Compatibility entry points are documented without deprecation attributes in this
slice. Existing consumers using strict warnings continue to compile; no removal
schedule is selected. Shared unit-error names use public re-exports rather than
type aliases, preserving the constructor/value namespace as well as type names.
Error shape/size checks, redaction and Display text remain unchanged. Debug's
canonical type/private-field labels can reflect the clearer Rust names; exact
Debug text and `type_name` spellings are not a persistence or wire contract.

`authentication_name` identifies the application-configured authentication
registration/resolver. It is not an HTTP WWW-Authenticate protocol token, an
OIDC issuer, a storage-backend type or a permission. Authentication and business
authorization remain separate; matching still compares the same exact string.
Old `scheme()` returns that same string. No normalization, serialization or
Session identity change occurs.

The existing typed configuration loader declares allowed keys in base TOML
before Serde deserialization. Preserve its strict key/type and precedence rules.
Use [Serde rename](https://serde.rs/field-attrs.html) to expose
`refill_interval_ms` in Rust while retaining `interval_ms` externally. No second
wire spelling, alias map, precedence rule, hidden fallback or legacy flat
RATE_LIMIT_INTERVAL_MS variable is added. Existing default-off profiles and all
RATE_LIMIT_POLICY__ overrides keep their meanings. Documentation explicitly
states that interval_ms replenishes one cell, rather than a full burst or a
counting window. A future configuration-schema rename is a separate decision.

## Bounded implementation and verification

Three independent slices cover config loader wrappers, Auth label/error aliases,
and rate-limit interval/error aliases plus its typed example. Existing source
loading, provider lookup, Auth matching and Governor algorithm behavior stay
unchanged. The optional secret-provider wrapper retains the existing feature
boundary. Do not change dependencies, locks, settings TOMLs, release scope,
Core/browser defaults, Auth/Session protocols or business policy.

Focused checks prove old/new API equivalence (including unit-error construction),
old key/OS override behavior, default-off and enabled policy behavior, and
unchanged invalid-input/fail-closed semantics. Use direct product tests and
existing compiler/lint/docs commands; no validation infrastructure or old
Auth/Valkey/IdP/browser/deployment qualification rerun. Original review and scan
history remains unchanged. Independent review of actual sources/evidence must
pass before this follow-up is complete.

## Implemented changes and focused results

Config adds `load_with_env` and feature-gated `load_with_env_and_secrets`, with
former names delegating to those same implementations. Direct typed and secret
provider examples use the primary names. No source precedence, file handling,
error category or secret-provider authority changed.

Auth stores the same string under the clearer private name and uses
`authentication_name()` for the same exact matching operation. Old accessor,
constant and error spelling remain usable. Its compiled README examples use the
canonical error. Rate-limit changes only Rust interval vocabulary and its
construction error spelling. The typed example uses Serde's explicit wire-name
mapping and the primary loader method. Existing native routes and default-off
behavior are retained.

| Focused command | Actual result |
|---|---|
| Config `cargo test --offline --locked --manifest-path crates/yetlet-config/Cargo.toml --test naming_compatibility` | PASS: two source/precedence/error compatibility tests |
| Same config test with `--features secret-provider` | PASS: four tests, including provider authority and closed-error equivalence |
| Config strict Clippy with secret-provider/all-targets, examples compile and format | PASS; existing suites compiled, not executed |
| `cargo test -p yetlet-auth --all-features --test naming_compatibility` | PASS: four tests for old/new constructors/types/accessors/constants, bounds/redaction and exact fail-closed matching |
| Auth README doctests with all features | PASS: two compiling examples |
| Auth strict Clippy/all-targets/all-features and format | PASS |
| `cargo test -p yetlet-rate-limit --test naming_compatibility --offline --locked` | PASS: two tests for old/new unit-error values/patterns/types and positional constructor bounds |
| Rate-limit native example tests | PASS: seven tests for unchanged settings keys, default-off/enabled behavior and input rejection |
| Rate-limit README doctest | PASS: one compiling example |
| Rate-limit strict Clippy/all-targets and format | PASS |

Config and rate-limit Cargo test/lint/check records explicitly use offline and
locked flags. Auth command records do not include those flags; no such invocation
property is claimed for Auth, while its passing results and unchanged Cargo
manifests/lockfiles are verified. No manifest/lock/settings-TOML changes were
selected; the original Auth/Session,
limiter-state/HTTP, Valkey/IdP/browser and deployment runtime suites were reused,
not repeated. No listener/server/browser was started. Config/example temporary
fixtures were removed and command processes completed. No repairs or operational
retries were needed in any of the three implementation slices.

Raw commands, stdout/stderr, exit codes and cleanup records are retained in
`/tmp/yetlet-naming-compat-20261022-rumxsq4d/` under `config-*`, `auth-*` and
`rate-*`. Cross-file docs verification and independent Reviewer acceptance are
recorded below when complete. The original review/security-scan history remains
bound to its historical sources; this naming change is not a fresh security scan
or production qualification.

## Independent acceptance and final scope

Independent Reviewer `/root/naming_compat_independent_review_20261022` returned
**PASS**, zero issues and zero required followups. Registered reviewer role,
configured `gpt-6-astra` model and effective high effort were verified through
native trace and the shared resolver. All 20 scoped source hashes matched before
and after review. It confirmed old/new API equivalence, unchanged settings wire
keys, strict feature boundaries, fail-closed Auth matching, redaction and focused
evidence/cleanup. It ran no tests or services and changed no files.

The reviewer identified an evidence-wording limitation: retained Auth commands do
not prove offline/locked invocation flags. Documentary closure corrects the Auth
command row and blanket claim to match the actual logs. The original reviewed
document and this observation remain in the evidence directory; no passing test
was rerun to manufacture different evidence. There were no code repair rounds;
this was one documentation accuracy correction at closure.

Changed files by area:

| Area | Actual files |
|---|---|
| Auth | [library](../crates/yetlet/src/auth/mod.rs), [README](../crates/yetlet/guides/auth.md), historical `crates/yetlet-auth/tests/naming_compatibility.rs` at checkpoint `83f417e` |
| Configuration | [loader](../crates/yetlet/src/config/mod.rs), [optional secret methods](../crates/yetlet/src/config/secrets.rs), [typed example](../crates/yetlet/examples/config/typed.rs), [secret example](../crates/yetlet/examples/config/secret_provider.rs), historical `crates/yetlet-config/tests/naming_compatibility.rs` at checkpoint `83f417e` |
| Rate-limit | [library](../crates/yetlet/src/rate_limit/mod.rs), [README](../crates/yetlet/guides/rate-limit.md), [native example](../crates/yetlet/examples/rate_limit/native.rs), historical `crates/yetlet-rate-limit/tests/naming_compatibility.rs` at checkpoint `83f417e` |
| Documentation | This record, [configuration guide](CONFIGURATION.md), [Extensions](EXTENSIONS.md), [index](README.md), [Roadmap](ROADMAP.md), [secret providers](SECRET-PROVIDERS.md), [settings/secret integration](SETTINGS-SECRET-PROVIDER.md), [YT-117 evidence](YT-117-RATE-LIMIT.md) |

The closure changes only this evidence document and its index status; reviewed
crate sources/examples/tests/READMEs remain byte-identical. Final docs and direct
file checks are recorded as `docs-final.*` and `closure-checks.json` alongside
`independent-review.json`. HEAD and branch remain unchanged; staged files zero,
no commit/push. No new security scan, browser/platform/production claim or M20
release change follows. YT-118 is the next independent Todo candidate and needs
separate owner selection.
