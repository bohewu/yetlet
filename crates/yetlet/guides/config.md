# yetlet::config

Experimental, unpublished `0.9.0-alpha.5` release preparation. Requires Rust 1.99+.
This native-only package is an explicit opt-in; it is not a global Settings
service and is rejected in browser Wasm. Publication eligibility does not
authorize a registry upload or establish a supported public release.

## Prerequisites

Use this source checkout until a release is explicitly published. Select a
trusted application-owned configuration directory and environment prefix.
The package remains a module of the shared SDK; Settings loading is optional
for Yetlet applications.

## Minimal integration

```rust
use yetlet::config::{Environment, Loader};

let loader = Loader::new("/explicit/app/settings", "MYAPP");
assert_eq!(Environment::default(), Environment::Production);
assert_eq!(format!("{loader:?}"), "Loader { .. }");
```

This constructs a loader; it does not read files, bind a listener, install
middleware or load secrets. Call the documented typed loading methods at
application startup with your settings type and selected inputs.

## Expected result, configuration and errors

Startup loading uses bounded source precedence and typed validation. Missing,
invalid or unavailable protected values fail without a less-trusted fallback.
Select `secret-provider` only when explicitly injecting `yetlet::secrets` and
typed `SecretBinding` fields. The normal default path calls no provider.
There is no automatic reload or provider registry. Applications offering reload
own serialization of load, validation and whole-snapshot publication.

The listener-free `secrets_state` example owns the Settings/SecretProvider
composition. From the repository root, use a synthetic value:

```sh
(
  export YTSECRETS_SERVICE__TOKEN=example
  cargo run --offline --locked -p yetlet --features config \
    --example config-secrets-state --features secret-provider,server-axum
)
```

Expected output: `Example request status: 200 OK`. This example moved from
`yetlet::secrets`; the provider package no longer depends on Config in tests.
The public Settings/provider APIs and default feature selection are unchanged.
Its focused test target is `cargo test --offline --locked --manifest-path
crates/yetlet/Cargo.toml --example config-secrets-state --features secret-provider,server-axum`.

`Error` exposes bounded categories, including `Error::Key` for key selection,
without raw parser/IO diagnostics or secret values. Read the OS-specific file
ownership/permission limits before choosing a deployment configuration. Do not
serialize settings secrets or native handles into browser state.

## API reference and license

Rustdoc/source at the same revision owns signatures and feature availability.
The repository [configuration guide](https://github.com/bohewu/yetlet/blob/master/docs/CONFIGURATION.md)
and [protected-field guide](https://github.com/bohewu/yetlet/blob/master/docs/SETTINGS-SECRET-PROVIDER.md)
provide fuller integration context when repository access is available; those
branch links are navigation, not version-pinned qualification evidence.
Dual licensed under [MIT](../../yetlet-config/LICENSE-MIT) or [Apache-2.0](../../yetlet-config/LICENSE-APACHE).
