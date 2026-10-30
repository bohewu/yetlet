# Opt-in secret providers

Inject read-only native secret lookup into application constructors or typed
state. Lookup is asynchronous; loading settings and rotating credentials remain
separate, explicitly selected operations.

**Experimental, unpublished.** Use this source checkout and the documented target scope.


## Prerequisites

Use the pinned Rust/Cargo toolchain and the isolated `yetlet-secrets` workspace.
Select a provider explicitly. The existing state example also selects the config
loader and bundled example settings. From the repository root, pass only the
synthetic token below; no cloud account, real credential or listener is needed.

The current composition example is owned by `yetlet-config` as `secrets_state`,
behind its existing `secret-provider` feature. Its four fixture files moved with
it. Dated evidence below preserves the old `yetlet-secrets --example state`
commands and their original results; current navigation points to the new source.

## Minimal integration

Run the existing listener-free composition example:

```sh
(
  export YTSECRETS_SERVICE__TOKEN=example
  cargo run --locked --manifest-path crates/yetlet-config/Cargo.toml --example secrets_state --features secret-provider
)
```

For your own application, load/validate its settings, select only required fields
into `SettingsSecrets`, and inject `Arc<dyn SecretProvider>` into the consumer.
The detailed existing snippets below assume that already-loaded `settings` value.



## Expected result

The example prints `Example request status: 200 OK`, consumes its in-process
response and exits without a listener or secret dump. `get` returns owned
protected values; absent names, unavailable service and denied access remain
different outcomes. Consumers holding a copied secret keep that value; freshness
of later lookups follows the selected provider.

## Configuration and errors

### Read contract

`SecretProvider: Send + Sync` exposes one asynchronous operation:

```text
get(name) -> Result<Option<SecretString>, SecretError>
```

The object-safe Rust method returns a boxed, Send future so ordinary
`Arc<dyn SecretProvider>` handles can be passed into constructors and application
state. The concrete signature and examples are in the
[crate guide](../crates/yetlet-secrets/README.md).

- `Ok(Some(value))` returns an owned protected string at lookup time.
- `Ok(None)` means that name does not exist. The application decides whether it
  is required. An existing empty string remains `Some`, and validation belongs
  to its consumer.
- `SecretError::Unavailable` means retrieval could not complete.
- `SecretError::AccessDenied` means retrieval was denied.

The error categories have fixed messages and retain no input, credential or
backend response. The local provider's Debug representation omits names and
values. Values use the existing `secrecy::SecretString` representation; reading
their plaintext requires an explicit `ExposeSecret` operation. Applications
remain responsible for excluding plaintext from logs, responses and browser
state. The contract does not assign a business use to a name or value.

### Reuse existing settings

Keep the application's existing typed schema and
[loader](CONFIGURATION.md). For example, deserialize `service.token` as a
`SecretString`, using the same base/profile/explicit Development-secret/scoped
environment precedence. Then select that field explicitly:

```rust
use std::{collections::BTreeMap, sync::Arc};
use yetlet_secrets::{SecretProvider, SettingsSecrets};

let secrets: Arc<dyn SecretProvider> = Arc::new(SettingsSecrets::new(
    BTreeMap::from([("remote-token".into(), settings.service.token)]),
));
```

The lookup name is application-owned and can differ from its configuration
field. Other public or private settings stay outside this map. `SettingsSecrets`
owns a selected snapshot; it does not discover files, read process variables,
parse a second configuration format, or change loader precedence. Development
secret files remain explicit and ignored in Staging/Production as before.

Supported local inputs use the existing composition:

| Input | How it reaches `SettingsSecrets` |
|---|---|
| Application-provided values | Pass an explicit `BTreeMap<String, SecretString>` directly |
| TOML settings | Load app defaults or `base.toml`, then the selected Development/Staging/Production TOML, and select secret fields |
| Development dotenv or TOML secret file | Explicitly select one path and format through the same Loader; ignored in Staging/Production |
| Prefixed OS environment variables | The same Loader applies them last, for example `MYAPP_SERVICE__TOKEN` |

Loader precedence remains defaults/base, selected profile, explicit Development
secret file, then OS environment. Base declares known keys/types; keep real
credentials out of committed settings. A user-private secret-file path is
explicit, not discovered automatically. OS keychain and encrypted-vault adapters
are not supplied by the local provider.

### Constructor injection

A normal service accepts the shared handle through its constructor. Handlers
can receive the service or provider through native Axum State and the existing
`App::with_state`; background tasks and non-HTTP services use the same handle.
The application owns the struct and assembly:

```rust
struct RemoteService {
    secrets: Arc<dyn SecretProvider>,
}

impl RemoteService {
    fn new(secrets: Arc<dyn SecretProvider>) -> Self {
        Self { secrets }
    }
}

let remote = RemoteService::new(secrets.clone());
```

Tests can pass a small custom provider or another `SettingsSecrets` instance.
Startup-only consumers may resolve required values before constructing a client
and need not retain the provider in request state. No global registration or
new App/RequestContext field is required. This follows the explicit composition
in [ADR 0005](adr/0005-opt-in-capability-modules.md) and
[ADR 0006](adr/0006-handler-first-mvc-optional.md).

### Refresh and rotation

`get` reads; it does not generate or rotate a secret. A future provider may
choose caching and current-version resolution internally. The consumer decides
when to look up a value and how retrieval failures affect its operation. The
trait promises neither a freshness interval nor atomic consistency across
separate lookups.

`SettingsSecrets` continues returning its snapshot until the application
explicitly calls `replace` with a complete newly loaded selection. After a
configuration change the application may reload through its existing Loader
and call `replace`, or construct a new provider on restart. A client
that already copied a password keeps that password until the application
reconfigures or rebuilds the client, even if a different provider later returns
a new value. No transparent consumer reload is implied.

#### Minimal local reload contract

No source revision or local generation counter is added. `replace` publishes
the complete supplied map, removing omitted names; it does not establish that
the input is newer. Each lookup sees an old or new complete snapshot, while
separate lookups can cross a replacement boundary.

If an application supports reload, use one application-owned update entry and
serialize the entire sequence:

```text
acquire application reload gate
load current inputs through the existing Loader
validate application requirements
select all secret fields to retain
replace the provider snapshot
release gate
```

Acquire the gate before reading inputs, and route every reload writer through
it. A map loaded before the gate, or a second direct writer, defeats this
ordering rule. Loader or application-validation failure must skip `replace`,
leaving the prior healthy snapshot usable; a later valid reload may retry the
same entry. Provider lock poisoning still makes lookup/replacement Unavailable,
so this is not a promise to recover from every internal failure. The
[State example tests](../crates/yetlet-config/examples/secrets_state.rs) demonstrate
this application-owned pattern; the executable still loads once at startup.

This prevents reload completion inversion within that application entry. It
does not reject an externally rolled-back file, coordinate multiple processes,
make independently read settings files transactional, or refresh existing
clients. Restart builds a new snapshot from current inputs, with no persisted
revision floor. Replacing the same values is allowed; there is no version-based
no-op or stale-update rejection. Source identity/freshness, local snapshot
adoption and an external service's acceptance of old credentials are distinct.

Generic string lookup does not export a non-exportable signing key or replace
the [versioned signing contract](YT-093-SIGNING-CONTRACT.md). Signing rotation,
TLS certificate preparation and cloud-provider implementations remain separate
capabilities. Explicit local snapshot replacement and a bounded Azure CLI
rotation experiment are documented in the separate follow-up; no secret-write
CLI command, cache, watcher, retry policy or provider registry is introduced.

#### Administration and Azure

Applications can create a new Azure Key Vault Secret version through the
[Set Secret API](https://learn.microsoft.com/en-us/rest/api/keyvault/secrets/set-secret/set-secret?view=rest-keyvault-secrets-2025-07-01)
when authorized with `secrets/set`. For an ordinary string secret the caller
supplies the value: publishing a version does not itself generate or change a
database password or third-party API credential. Microsoft's
[rotation example](https://learn.microsoft.com/en-us/azure/key-vault/secrets/tutorial-rotation)
uses a separate Function to generate a password, publish it and update SQL;
those steps can temporarily disagree. Our recorded Azure experiment used
operator-controlled writes around the read-only Rust provider, not an automatic
rotation service.

The recommended ownership here is to keep the business application's identity
limited to the secret reads or cryptographic operations it needs. Separate
creation, rotation, disabling and deletion into an operator command or a
dedicated automation identity. This applies Microsoft's
[least-privilege guidance](https://learn.microsoft.com/en-us/azure/key-vault/general/secure-key-vault)
and [separate read/management roles](https://learn.microsoft.com/en-us/azure/key-vault/general/rbac-guide):
a compromised reader should not gain credential-management authority. Automated
rotation can be application-owned code running as a separate worker; it need
not be manual or part of a deployment, and is not a side effect of `get`.

Keep [signing administration](YT-093-SIGNING-CONTRACT.md) separate from normal
signing. Retain old public verification material according to policy; rotating
a key is not revoking its trust. TLS certificate renewal belongs to the chosen
certificate/platform tooling, followed by explicit server adoption. Azure
[certificate renewal](https://learn.microsoft.com/en-us/azure/key-vault/certificates/overview-renew-certificate)
depends on issuer integration, and renewing a certificate can reuse its key.
For local development, keep mkcert CA preparation in explicit developer tooling;
the business app should not require its powerful
[CA private key](https://github.com/FiloSottile/mkcert#installation).

## API reference

Use the [crate guide/source](../crates/yetlet-secrets/README.md),
[state example](../crates/yetlet-config/examples/secrets_state.rs),
[settings injection guide](SETTINGS-SECRET-PROVIDER.md) and
the [current API reference](API-REFERENCE.md).

<details>
<summary>Historical delivery context and evidence</summary>

Date: 2026-10-18. Scope: separately authorized YT-090 follow-up.
Status: the initial read contract passed focused local verification and
[independent review](SECRET-PROVIDER-REVIEW.md), with zero blocking issues or
required followups. The separately authorized
[rotation/Windows/Azure follow-up](SECRET-ROTATION-VERIFICATION.md) extends that
slice; its current results and review are recorded separately. Local commit is
owner-authorized; publication and production qualification remain separate.

The unpublished, Experimental `yetlet-secrets` nested workspace supplies a
small native string-secret contract. Applications explicitly select its path
dependency and construct a provider. Core, browser, the default configuration loader,
signing, CLI scaffolds and existing applications do not acquire a dependency
or initialize a provider automatically.

The separately authorized [settings integration](SETTINGS-SECRET-PROVIDER.md)
can resolve an injected provider into selected protected settings fields through
the loader's `secret-provider` feature. `SettingsSecrets` retains the reverse,
already-loaded-settings adapter; neither direction implicitly enables the other.

`yetlet_config::Loader` is the settings loader. `SecretProvider` is the distinct
async secret-lookup trait; there is no SettingsProvider interface. The
[naming guide](NAMING.md) records canonical loader entry points and the separate
external settings boundary; secret provider behavior is unchanged.

### Verification

The initial read-contract commands completed with exit 0:

```text
cargo test --offline --locked --manifest-path crates/yetlet-secrets/Cargo.toml --all-targets
cargo test --offline --locked --manifest-path crates/yetlet-secrets/Cargo.toml --doc
cargo clippy --offline --locked --manifest-path crates/yetlet-secrets/Cargo.toml --all-targets -- -D warnings
cargo fmt --manifest-path crates/yetlet-secrets/Cargo.toml --check
cargo xtask check docs
```

The three provider tests cover exact/owned values, empty/missing distinctions,
redacted Debug and errors, custom implementations and Send execution. Four
listener-free example tests cover base/profile/explicit Development-secret/ENV
precedence, ignored secret-file selection in Staging/Production, ordinary
service and Axum State injection, substitution, escaped SSR/CSP and absence of
synthetic secrets in rendered output. One rustdoc example also passed.

The configured [state example](../crates/yetlet-config/examples/secrets_state.rs) ran
successfully with explicit synthetic environment input and reported only
`Example request status: 200 OK`. Normal dependencies are exactly
`yetlet-secrets -> secrecy 0.10.3 -> zeroize 1.9.1`. Root Cargo metadata has 34
packages, none depending on the new crate. Root manifest/lock, existing loader,
starter and Core/browser sources remain unchanged.

Logs and preservation records are retained under
`artifacts/secret-provider-20261018/`. Tests removed their temporary secret
fixtures; no listener, background server or browser was started. All command
sessions terminated. The staged index and HEAD remain unchanged.

Actual changed files are the new crate's manifest/lock, library, provider tests,
README, state example and four settings TOMLs; this guide, the configuration and
extension guides, API classification, documentation index and a separate YT-090
backlog follow-up link. Existing unrelated planning remains intact. No broader
backlog item changes state; YT-094 remains a separate next core discussion.

The initial read-contract slice did not include Windows/cloud execution. The
owner subsequently selected those checks in the linked follow-up. Neither slice
claims browser secret handling or authorizes a commit/push.

</details>
