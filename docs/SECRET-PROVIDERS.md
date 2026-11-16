# Opt-in secret providers

The native local signing service and operator key administration are described
in the [replaceable signing contract](#local-signing-and-replaceable-provider-contract--yt-093).
Release signing follows the separate [release trust policy](RELEASE-SUPPLY-CHAIN.md#adopted-release-trust-and-disclosure-policy--yt-096).

Inject read-only native secret lookup into application constructors or typed
state. Lookup is asynchronous; loading settings and rotating credentials remain
separate, explicitly selected operations.

**Experimental SDK integration.** These modules are distributed in SDK
`0.9.0-alpha.8`; use the documented features/target and source matching the
version you consume. [Release readiness](RELEASE-READINESS.md) records distribution
and its qualification limits.


## Prerequisites

Use the pinned Rust/Cargo toolchain and the isolated `yetlet::secrets` workspace.
Select a provider explicitly. The existing state example also selects the config
loader and bundled example settings. From the repository root, pass only the
synthetic token below; no cloud account, real credential or listener is needed.

The current composition example is owned by `yetlet::config` as `secrets_state`,
behind its existing `secret-provider` feature. Its four fixture files moved with
it. Dated evidence below preserves the old `yetlet-secrets --example state`
commands and their original results; current navigation points to the new source.

## Minimal integration

Run the existing listener-free composition example:

```sh
(
  export YTSECRETS_SERVICE__TOKEN=example
  cargo run --locked -p yetlet --features config --example config-secrets-state --features secret-provider
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
[crate guide](../crates/yetlet/guides/secrets.md).

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
use yetlet::secrets::{SecretProvider, SettingsSecrets};

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
[State example tests](../crates/yetlet/examples/config/secrets_state.rs) demonstrate
this application-owned pattern; the executable still loads once at startup.

This prevents reload completion inversion within that application entry. It
does not reject an externally rolled-back file, coordinate multiple processes,
make independently read settings files transactional, or refresh existing
clients. Restart builds a new snapshot from current inputs, with no persisted
revision floor. Replacing the same values is allowed; there is no version-based
no-op or stale-update rejection. Source identity/freshness, local snapshot
adoption and an external service's acceptance of old credentials are distinct.

Generic string lookup does not export a non-exportable signing key or replace
the [versioned signing contract](SECRET-PROVIDERS.md#local-signing-and-replaceable-provider-contract--yt-093). Signing rotation,
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

Keep [signing administration](SECRET-PROVIDERS.md#local-signing-and-replaceable-provider-contract--yt-093) separate from normal
signing. Retain old public verification material according to policy; rotating
a key is not revoking its trust. TLS certificate renewal belongs to the chosen
certificate/platform tooling, followed by explicit server adoption. Azure
[certificate renewal](https://learn.microsoft.com/en-us/azure/key-vault/certificates/overview-renew-certificate)
depends on issuer integration, and renewing a certificate can reuse its key.
For local development, keep mkcert CA preparation in explicit developer tooling;
the business app should not require its powerful
[CA private key](https://github.com/FiloSottile/mkcert#installation).

## API reference

Use the [crate guide/source](../crates/yetlet/guides/secrets.md),
[state example](../crates/yetlet/examples/config/secrets_state.rs),
[settings injection guide](SETTINGS-SECRET-PROVIDER.md) and
the [current API reference](API-REFERENCE.md).

<details>
<summary>Historical delivery context and evidence</summary>

Date: 2026-10-18. Scope: separately authorized YT-090 follow-up.
Status: the initial read contract passed focused local verification and
[independent review](SECRET-PROVIDERS.md#native-verification-and-independent-review-boundaries), with zero blocking issues or
required followups. The separately authorized
[rotation/Windows/Azure follow-up](SECRET-ROTATION-VERIFICATION.md) extends that
slice; its current results and review are recorded separately. Local commit is
owner-authorized; publication and production qualification remain separate.

The Experimental `yetlet::secrets` module supplies a
small native string-secret contract. Applications explicitly select its path
dependency and construct a provider. Core, browser, the default configuration loader,
signing, CLI scaffolds and existing applications do not acquire a dependency
or initialize a provider automatically.

The separately authorized [settings integration](SETTINGS-SECRET-PROVIDER.md)
can resolve an injected provider into selected protected settings fields through
the loader's `secret-provider` feature. `SettingsSecrets` retains the reverse,
already-loaded-settings adapter; neither direction implicitly enables the other.

`yetlet::config::Loader` is the settings loader. `SecretProvider` is the distinct
async secret-lookup trait; there is no SettingsProvider interface. The
[naming guide](NAMING.md) records canonical loader entry points and the separate
external settings boundary; secret provider behavior is unchanged.

### Verification

The initial read-contract commands completed with exit 0:

```text
cargo test --offline --locked -p yetlet --features secrets --all-targets
cargo test --offline --locked -p yetlet --features secrets --doc
cargo clippy --offline --locked -p yetlet --features secrets --all-targets -- -D warnings
cargo fmt -p yetlet --features secrets --check
cargo xtask check docs
```

The three provider tests cover exact/owned values, empty/missing distinctions,
redacted Debug and errors, custom implementations and Send execution. Four
listener-free example tests cover base/profile/explicit Development-secret/ENV
precedence, ignored secret-file selection in Staging/Production, ordinary
service and Axum State injection, substitution, escaped SSR/CSP and absence of
synthetic secrets in rendered output. One rustdoc example also passed.

The configured [state example](../crates/yetlet/examples/config/secrets_state.rs) ran
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

## Local signing and replaceable provider contract — YT-093

The 2026-10-18 implementation and independent review accepted an opt-in native
OpenSSL signing service, the console sample and selected key CLI operations.
Its original nonpublishable nested workspace later joined the historical
sixteen-package cohort. The current [two-package SDK](COMPATIBILITY.md#two-package-feature-and-target-contract)
and independently versioned CLI retain their own publication authority.
Cloud adapters remain future selected work. The Azure CLI experiment established
interoperability at its recorded scope. It did not implement an Azure adapter.

Keep signing policy independent of Core, HTTP, Tokio, cloud SDKs and private-key
file formats. Constructor-injected ordinary Rust traits with `Future + Send`
separate exact-version asynchronous signing from exact-version public-material
resolution. Public-only verification needs no signing authority. Key creation,
import, rotation and deletion are provider administration. They are separate
from the common signing capability. No provider registry, DI container or
background rotation service is required.

### Exact bytes, key identity and trust

| Contract | Requirement |
|---|---|
| Key reference | Configured provider scope, logical key ID and immutable opaque version. A versionless `latest` alias cannot identify a completed signature |
| Admission | Capture the exact active version before awaiting signing. Reject a backend response naming another version. No silent provider/key fallback or retry |
| Input | Hash exact input bytes once using the established hash library. Pass a typed 32-byte SHA-256 digest to the backend without rehashing or text normalization |
| Result | Envelope version, explicit algorithm, exact scoped key version and signature bytes. Reference algorithm ECDSA P-256/SHA-256, fixed64-byte `r || s` interchange |
| Adapter encoding | Translate DER/vendor representations through maintained library APIs. Accept valid high-S and low-S signatures. Signature uniqueness, application IDs and replay prevention require separate semantics |
| Authority | Envelope fields only select configured scopes. They never authorize an endpoint, tenant, root, certificate or new key |

Raw signatures authenticate input bytes. Envelope routing/version metadata does
not cryptographically attest which provider or certificate version signed.
Independently revocable identities must use distinct public keys. Reject duplicate
public keys across all scopes, including retained retired/revoked identities,
using library key-equality checks at configuration and policy updates. Certificate
renewal may reuse a key, so certificate fingerprints alone are insufficient.
Retain revoked public identity to prevent its reintroduction under another version.
A future alias must share one trust/revocation identity and needs its own contract.

Authorize the exact key version under explicit trust policy before acceptance.
The local implementation checks leaf-certificate pins, key/certificate matching,
supported curve/algorithm, dates and signing key usage. PFX-supplied certificates
cannot enroll themselves. A bare public key has no invented expiry. The contract
provides no general PKI/OS-store chain construction, revocation service, trusted
timestamp or historical certificate-validity decision. Future KMS metadata need
not use X.509. Detached results carry no JWS/CMS/XMLDSig/PAdES or legal compliance
guarantee.

Offline verification can use configured public material only and fails closed on
missing/disallowed versions, invalid material/password, unsupported algorithms,
invalid signatures, policy rejection and backend unavailability. Return closed
nonsecret errors without library chains, key bytes, passwords or vendor details.
A timeout cannot establish that remote signing produced no effect.

### Local policy lifecycle and operator CLI

Trust v2 public material before activation while v1 can still sign. Activate v2
atomically for new operations. An already admitted v1 operation may finish as v1.
Retire v1 from new signing while preserving its public material for old signatures.
Revoke v1 to reject verification even where signature mathematics remains valid.
Local policy does not delete a remote key, cancel admitted work or synchronize
multiple processes. Rotation, revocation and private-key destruction remain
separate operations. Each logical key rotates independently.

The existing commands are `yetlet key status [KEY] [--provider NAME]
[--manifest-path PATH]` and `yetlet key rotate [KEY] [--provider NAME]
[--manifest-path PATH] [--yes] [--from PATH]`. A provider name is a configured
alias. Omitted names use explicit `default_provider`/`default_key`, without
guessing. Snake_case table keys follow the existing settings loader.
Applications use the service directly. There is no general CLI signing command.

Use a top-level `signing` section separate from `cli`. Base settings declare all
provider/key names and version arrays. A selected profile may replace a declared
array while preserving unknown-key rejection. Passwords come only from explicitly
named process environment variables. Status is read-only and password-free.
It reports certificate dates/availability without conflating renewal and rotation.

Rotation confirmation shows provider, key, current version and selected settings
file. Require terminal approval or explicit `--yes` before effects. Bind the plan
to the inspected snapshot and reject concurrent changes. `self_signed_test`
generates synthetic material with OpenSSL, including empty-key initialization.
Imported identities require replacement PFX via `--from`, with certificate/public
fingerprint in the confirmation. Preserve the actual issuance process. A reused
public key cannot become an independently revocable version.

After confirmation validate material/trust, publish an immutable private version,
then atomically update only the chosen key's public version records and
`active_version` in its selected profile. Preserve comments, array decorations,
unrelated settings, other keys/providers and old public records. Retire old signing
policy without automatically deleting private material. Failed settings writes
preserve the previous active selection and cannot report activation. Masking
environment overrides, unsafe files, unsupported providers and read-only settings
fail explicitly. The shared settings loader has no general write API.
Applications must restart and reload settings to adopt changes. Automatic file
watching, distributed rollout and cloud autorotation remain outside this contract.

### OpenSSL implementation and qualification boundary

Maintained Rust OpenSSL bindings perform hashing, cryptographic arithmetic,
randomness, PEM/PFX decoding, encryption and signature encoding. The recorded
native pin is OpenSSL3.5.9 LTS with `openssl0.10.81` / `openssl-sys0.9.117`,
official source SHA-256
`603f5602e2eef00d77fbd429d34dcd5822bb301757a1bc9cdb24c670f1eb859a`.
Linux used isolated static `OPENSSL_DIR`/`OPENSSL_STATIC=1` because development
headers were unavailable and vendored sources lagged the selected patch.
No cryptographic library was patched. PFX requires a present password-verified
MAC. This does not assert encryption of every bag. Generated private PEM is
encrypted. Lock the application dependencies and review advisories and supported
PEM/PFX profiles. Using the binding supplies no audit or FIPS qualification.

Native headers/libraries or C/Perl/make prerequisites require their own host proof.
Windows SQLite/Zig evidence does not qualify OpenSSL. A pure-Rust/pre-release
PKCS#12 replacement needs a separate maturity decision. Future Azure Key Vault
and OpenBao adapters must demonstrate the same exact-version/digest contract and
native integration. Two independently constructed local providers alone cannot
prove cloud compatibility. See the source-specific
[Linux, Windows and Azure results](QUALIFICATION-MATRIX.md#local-signing-key-cli-and-azure-evidence--2026-10-18).

## Native verification and independent review boundaries

The initial 2026-10-18 read-only provider review at
`b627d291696fe1aaa4a3db5040da07e2ad44da04` plus scoped uncommitted source
passed with zero blocking findings. It checked object-safe Send lookup,
owned protected values, missing/empty distinctions, fixed errors, immutable
selected snapshots and constructor/State replacement without Loader changes.
Three provider/four example tests, one doctest, strict Clippy/fmt, dependency
isolation and synthetic State200 were reviewed. All786 baseline hashes showed
only five intended existing-doc changes, with ten new crate files inspected.
This initial review's scope was local native prototype behavior.

The subsequent rotation review passed separately after inspecting Linux13 tests,
one doctest, native MSVC13 tests/State200 and the Linux-hosted real Azure CLI
experiment. The minimum local-contract follow-up separately passed15 tests,
one doctest, strict Clippy/fmt and independent direct reruns. Its registered
reviewer/high request lacked trace evidence, so effective role/model/effort
remained unverified. That provenance limit stays with its source review result.
The [retained rotation proof](SECRET-ROTATION-VERIFICATION.md) and
[platform/failure reconciliation](QUALIFICATION-MATRIX.md#secret-provider-native-and-review-evidence)
retain exact scope. No production cloud SDK, refresh scheduler,
Azure-on-Windows authentication or browser secret qualification follows.
