# YT-090 follow-up — settings from an injected SecretProvider

Date: 2026-10-19. Scope: owner-authorized opt-in settings integration, followed
by separate YT-094 shared-SQLite work only after this slice passes review.
Implementation and focused checks pass; independent Reviewer PASS with zero
issues and required followups.

## Contract

The existing `yetlet-config` loader adds the optional `secret-provider` feature.
Its ordinary `load`/`load_from_env` methods, file/environment precedence and
Development-only secret-file rules are unchanged. Default builds have no normal
dependency on `yetlet-secrets`. Core/browser and application defaults are unchanged.

Enable the feature and inject an already-constructed read-only provider:

```toml
yetlet-config = { path = "../yetlet-config", features = ["secret-provider"] }
```

```rust
use yetlet_config::SecretBinding;

let settings: Settings = loader.load_with_secrets(
    provider.as_ref(),
    &[SecretBinding::new("delivery-token", |s: &mut Settings| &mut s.delivery.token)],
).await?;
settings.validate()?;
let delivery = DeliveryClient::new(settings.delivery)?;
```

`SecretBinding` contains an application-owned lookup name and a function selecting
one `&mut SecretString` field. The typed selector cannot target an ordinary
`String`. Bindings are explicit code, not a new configuration syntax, discovery
mechanism, URL/command evaluator or provider registry. The provider is built
first by the application, independently of the settings it will resolve.

Normal settings are loaded once before any provider calls. Only explicitly
bound fields are looked up; for those fields the selected provider is the sole
final source, overriding normal literals even when an environment value exists.
There is no fallback on lookup failure. Unbound fields retain existing loader
precedence. Empty binding lists and ordinary loads perform no provider calls.

Values move directly into protected typed fields; no new plaintext TOML/string
intermediate is created for provider results. Each value is limited to the
existing 65536-byte input bound. `Some("")` stays an empty protected value;
application-required/nonempty/business validation must run before opening a
listener or constructing clients. `None`, Unavailable and AccessDenied return
distinct closed loader errors without names, values or backend diagnostic chains.
A failure returns no partially resolved settings; separately held prior settings
are not mutated. Separate lookups are not an atomic external-vault snapshot.

## Module and security boundaries

The application owns assembly. Give each module its needed typed settings or
specific secret, rather than the entire root settings/provider. The
[listener-free example](../crates/yetlet-config/examples/secret_provider.rs)
passes only a protected token and attempts to its delivery module; web state
receives presentation data and the constructed service. Public SSR uses only
the selected message. No automatic Serialize, browser export or settings dump
is added. Names and values are omitted from binding Debug/errors.

`SettingsSecrets` still adapts selected existing settings into a provider. The
new integration consumes a separately supplied provider; applications must not
create a circular bootstrap between them. No new `SettingsProvider` trait,
global access, DI container, cache, watcher, scheduler, hot reload or write/rotate
operation is introduced. Existing clients retain copied values until explicitly
reconfigured. The provider's `get` contract and local replacement semantics stay
unchanged.

Types and redaction reduce accidental disclosure; they are not process-memory
encryption or an isolation boundary against malicious code in the same process.
Applications/operators still own vault identity, transport and least-privilege
authorization. The reader identity should only access required secrets, with
administrative writes/rotation assigned separately. Ordinary source loading may
still parse literal secrets as before; this feature makes no process-wide
zeroization guarantee.

## Validation and review

Focused checks passed with exit 0:

```sh
cargo test --offline --locked --manifest-path crates/yetlet-config/Cargo.toml --features secret-provider --all-targets
cargo test --offline --locked --manifest-path crates/yetlet-config/Cargo.toml --all-targets
cargo test --offline --locked --manifest-path crates/yetlet-config/Cargo.toml --features secret-provider --doc
cargo clippy --offline --locked --manifest-path crates/yetlet-config/Cargo.toml --features secret-provider --all-targets -- -D warnings
cargo fmt --manifest-path crates/yetlet-config/Cargo.toml --check
cargo test --offline --manifest-path crates/yetlet-secrets/Cargo.toml --all-targets
cargo run --offline --locked --manifest-path crates/yetlet-config/Cargo.toml --features secret-provider --example secret_provider
cargo xtask check docs
```

The loader has 21 passing tests with the feature and 16 without. Existing typed
example tests remain 2/2; the new narrow-module example adds 2/2, and both
doctests pass, including rejection of an ordinary String selector. Secret-provider
regressions pass 15/15 across library/provider and existing example targets.
The listener-free example returns 200 without printing a secret. Normal Cargo
dependency trees confirm no default secret-provider dependency; opting in adds
only the existing `yetlet-secrets`/`secrecy`/`zeroize` chain. Root locked Cargo
metadata succeeds. New-file whitespace and documentation checks pass.

The new cases cover file/env precedence plus authoritative provider selection,
missing/denied/unavailable errors without fallback, exact literal values and
redaction, no lookup for ordinary/empty selection or invalid normal input,
empty-value/size bounds, and no publication/mutation on partial lookup failure.
The example covers module assembly and absence of secrets in SSR/CSP output.
An initial test compile required an explicit closure parameter type; it was
corrected before these passing results, without changing the implementation.

Independent Reviewer `/root/settings_secret_provider_review_oct19` returned
PASS with zero issues/required followups after inspecting the actual eleven
source/document targets and focused evidence. Only synthetic values and
listener-free requests were used; no listener, background service, cloud
resource or browser was started. Existing accepted OIDC, Valkey, Windows and
cloud evidence remains at its recorded scope. No commit or push was performed.
