# Experimental native secret providers

`yetlet-secrets` is an unpublished, independent opt-in crate and nested Cargo
workspace. Its only runtime dependency is `secrecy`. It does not register with
Yetlet Core, App, configuration, the CLI, or browser code. Browser WASM is rejected.

`SecretProvider: Send + Sync` exposes object-safe asynchronous
`get(name) -> Result<Option<SecretString>, SecretError>` through a boxed Send
future. Applications inject `Arc<dyn SecretProvider>` into their own ordinary
service constructors or Axum/Yetlet state. Replace that value to select a custom
implementation; no provider registry or DI container is required.

`Ok(None)` means a genuinely missing name. `Unavailable` and `AccessDenied` are
distinct failures with fixed public messages and no backend error sources.
Names are exact and case-sensitive. Empty and whitespace-only strings remain
present values; required-value validation belongs to the consumer.

```rust
use std::{collections::BTreeMap, sync::Arc};
use yetlet_secrets::{SecretProvider, SettingsSecrets};

// `settings` was already loaded by the application's existing Loader.
let settings_provider = Arc::new(SettingsSecrets::new(
    BTreeMap::from([("delivery-token".into(), settings.service.token)]),
));
let provider: Arc<dyn SecretProvider> = settings_provider.clone();
// Pass `provider` into an ordinary service constructor or app-owned state.

// After explicitly reloading settings with the same application Loader:
settings_provider.replace(BTreeMap::from([
    ("delivery-token".into(), reloaded.service.token),
]))?;
```

`SettingsSecrets` owns a snapshot of only the selected fields. Its explicit
`replace(values)` replaces the whole selected map through `&self`, so existing
shared handles observe new values on subsequent lookups. Include every name to
retain; omitted names become missing. Each lookup sees a complete old or new
snapshot, but separate lookups are not an atomic transaction. The adapter uses a
short synchronous lock, holds no lock across await or I/O, and returns the fixed
`Unavailable` error if that lock is poisoned.
There is no revision check: replacing identical values is allowed, and any
supplied map can replace the previous one. If an application offers reload,
serialize the full load, app validation, selection and replacement through one
app-owned entry, acquiring its gate before any input read. Loading/validation
failure skips replacement and retains the healthy snapshot. Every writer must
use that entry; the provider's own lock only serializes publication. This does
not prevent external source rollback or coordinate other processes. Restart
loads current inputs with no persisted version history.
It reads no files or environment variables, does not reflect over all settings,
and does not change existing settings or development-secret precedence. Its Debug
output omits both keys and values. Each successful lookup returns an owned
protected clone; exposing it requires an explicit `ExposeSecret` call. Do not put
secret values in logs, errors, HTML, JSON, or serialized hydration state. Protected
strings do not prevent an application from leaking an explicitly exposed value,
and the existing loader still handles intermediate configuration text normally.

Lookup timing is application policy. `get` never generates, rotates or reloads a
credential. To adopt a local file/environment change, the application explicitly
reruns its existing Loader, selects secret fields and calls `replace`. Generic
credential rotation also requires updating the external source and its consumers;
replacement is only the local adoption step. An already returned `SecretString`
or a client that copied its credential keeps the old value. Applications must
explicitly reconfigure or rebuild those clients. This slice adds no cloud SDK,
remote provider, secret writer, cache, watcher, retry, or automatic rotation.

The listener-free [state example](../yetlet-config/examples/secrets_state.rs) loads typed app-owned
`service.token`, explicitly maps it to `delivery-token`, constructs an ordinary
service with the provider, and installs that service in app-owned Axum/Yetlet
state. The public page uses only a selected public message. It makes no external
request and implements no authentication. The Development fixture contains only
a synthetic placeholder. The default Production fixture inherits an empty token
and the example reports HTTP 503; a runtime override supplies a configured value:

```sh
YTSECRETS_SERVICE__TOKEN="${YTSECRETS_SERVICE__TOKEN:?Set a local test value first}" cargo run --offline --locked \
  --manifest-path crates/yetlet-config/Cargo.toml --example secrets_state --features secret-provider
```

For an explicit local development secret file, application startup can retain the
existing `Loader::development_secrets(path, SecretFormat::Toml)` or Dotenv API
before `.load()`. Select Development with `YTSECRETS_ENVIRONMENT=Development`.
Staging and Production ignore that development file selection. Base settings,
the selected profile, the explicit Development secret file, then environment
overrides retain their existing order. No new secret namespace or parser exists.

The example's tests exercise those layers with synthetic values, both ignored
development-file profiles, explicit v1-to-v2 file reload into the same service's
provider, custom provider substitution, and escaped SSR with CSP and no secret
output. They also demonstrate a single application-owned reload gate around
loading, validation and publication, concurrent reload ordering, and preservation
of the old snapshot on failed reload followed by recovery. The executable still
loads once at startup; no reload loop or provider administration is installed.
The provider tests cover whole-map replacement through shared handles,
retained owned values, empty/missing distinctions, Send execution, fixed poison
errors, protected ownership, and redacted Debug/errors.

The State example and its four fixtures now belong to `yetlet-config`, behind
its existing `secret-provider` feature. Run that target with `cargo test --offline
--locked --manifest-path crates/yetlet-config/Cargo.toml --example secrets_state
--features secret-provider`. The provider-only commands below do not run the
moved composition target. This removes the reverse Config development dependency
for first publication without changing provider APIs or runtime dependencies.

```sh
cargo test --offline --locked --manifest-path crates/yetlet-secrets/Cargo.toml --all-targets
cargo test --offline --locked --manifest-path crates/yetlet-secrets/Cargo.toml --doc
cargo clippy --offline --locked --manifest-path crates/yetlet-secrets/Cargo.toml --all-targets -- -D warnings
```

The original read-contract evidence was local native only. The separately
selected [rotation, Windows and Azure record](../../docs/SECRET-ROTATION-VERIFICATION.md)
covers explicit replacement and the read-only Azure CLI experiment in
`examples/azure_cli.rs`. It does not establish production cloud-provider or
secret-management guarantees.

See the [local inputs and administration contract](../../docs/SECRET-PROVIDERS.md)
for the existing settings sources and the separation between application reads,
operator/automation rotation, signing and TLS certificate renewal.
