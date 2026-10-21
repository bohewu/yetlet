# YT-093 — Local signing and replaceable provider contract

Status: implemented and independently reviewed at the bounded scope in the
[verification record](YT-093-SIGNING-VERIFICATION.md). Date: 2026-10-18.

The owner selected local signing first, deferred OpenBao to future opt-in work,
then required a contract for replacement by Azure Key Vault/OpenBao providers
and careful use of established cryptographic implementations. The owner then
included the key CLI in the first version, required confirmed rotation to update
settings automatically, and selected multiple named keys/certificates. Cloud
adapters remain future opt-in work. Existing YT-092 preview/data and unrelated
planning remain outside this slice.

## Scope and ownership

The signing core is a small, nonpublishable, opt-in experimental crate shared by
the reference example and the explicitly selected key-management CLI operations.
These two consumers justify sharing the contract types and provider policy.
It remains independent of Yetlet Core, HTTP, Tokio, cloud SDKs and private-key
file formats. Constructor injection is sufficient. This slice creates no provider
registry, DI container, background rotation scheduler or mandatory service.

This stage defines the contract and a local OpenSSL-backed implementation target.
Azure Key Vault and OpenBao are future separately selected adapters. Their SDKs,
credentials, endpoints, provisioning and administration are absent now.

## Minimal boundaries

| Part | Responsibility |
|---|---|
| Core types/policy | Immutable scoped key identity, explicit algorithm/digest/signature representation, active-key snapshot, trusted/retired/revoked version decisions, bounded errors |
| Signing backend | Asynchronously sign using one exact key version; return the version actually used; never require private-key export |
| Public-key source | Resolve public verification material for an exact configured key version; supports a public-only verifier with no signing authority |
| Local implementation | Load configured PEM/PFX material and use maintained library APIs for key parsing, signing, verification and certificate operations |

Consumers use a small signing service built from the two backend interfaces and
explicit policy. Ordinary Rust traits with `Future + Send` permit local and
network implementations. Generic constructor injection is the first choice;
object-safe boxing is only needed if a later consumer requires runtime selection.
These are semantic contracts, not a published or stabilized Rust trait API.

The two backend operations are conceptually:

```text
sign_digest(exact_key_version, sha256_digest) -> signature + exact_key_version
public_key(exact_key_version) -> public verification material + exact_key_version
```

Key creation/import/remote rotation/deletion are provider administration, outside
the common signing interface. A future administration capability must not become
a method every signing-only backend has to implement.

## Identity and signed result

- A key reference includes a configured provider scope, logical key identifier
  and immutable version. Values are opaque to core; versions need not be integers.
  A versionless alias such as `latest` cannot identify a completed signature.
- Capture the exact active version before awaiting the backend. Reject a response
  reporting a different version. No retry against another key or provider.
- A detached result records envelope version, algorithm, exact scoped key version
  and signature bytes. The reference algorithm is ECDSA P-256 with SHA-256.
- Hash the exact input bytes once through an established hash library. The backend
  receives a typed 32-byte digest and must not hash it again. No text normalization.
- The proposed interchange signature representation is fixed 64-byte `r || s`.
  Adapters translate DER/vendor representations using library APIs. High-S and
  low-S valid signatures are accepted; uniqueness/canonicalization is not promised
  and signatures are not application identifiers or replay-prevention tokens.
- A key reference selects only an operator-configured provider/key scope. Envelope
  fields never authorize a new URL, tenant, trust root, certificate or key.
- The envelope is a raw detached-signature reference format. It is not JWS, CMS,
  XMLDSig, PAdES or a legal/business document-signing compliance claim.

Raw signatures authenticate the input bytes, not the envelope metadata. The exact
version records the backend response and routes verification; it is not a
cryptographic attestation of which provider or certificate version performed an
operation. Independently revocable identities must therefore use distinct key
material. The initial policy rejects the same public key under different
identities across all configured scopes, including retained retired/revoked
identities. Check public-key equality using maintained library APIs, not just
certificate fingerprints: certificate renewal can reuse a key. Check this on
initial configuration and policy updates, retaining revoked public-key identity
to prevent reintroduction under another version. A future alias feature would
have to share one trust/revocation identity; it is outside this slice.

## Rotation, retirement and revocation

| Event | Signing | Verification |
|---|---|---|
| Publish new version v2 | Continue with v1 until v2 public material is trusted and usable | Accept configured v1 and v2 |
| Activate v2 | New operations capture v2 atomically; an admitted v1 operation may complete as v1 | Existing v1 signatures remain accepted under policy |
| Retire v1 signing | No new v1 signing through the core policy; private material can be removed separately | Retain v1 public material for old signatures |
| Revoke v1 trust | Block operations admitted after the local policy update | Reject v1 even if the mathematics of an old signature are valid |

Rotation is not revocation or key destruction. The core controls local policy;
it does not claim to disable/delete a remote key or cancel already admitted
provider work. A revoked operation might already have produced a signature.
Multi-process policy rollout, remote administrative enforcement, trusted signing
time and long-term historical validation require separate contracts.

The first local rotation proof uses disposable identities and explicit policy
transitions. Multiple logical keys have independent versions and certificates;
rotating one key must preserve all other keys and providers. Settings select
the provider/identity through the existing settings mechanism. Automatic file
watching, cloud autorotation and distributed policy synchronization are outside
this slice.

## First-version key CLI

```text
yetlet key status [KEY] [--provider NAME] [--manifest-path PATH]
yetlet key rotate [KEY] [--provider NAME] [--manifest-path PATH] [--yes] [--from PATH]
```

The provider name selects a configured alias, not a provider type or arbitrary
endpoint. Omitted targets use explicit `default_provider` and `default_key`
settings; never guess among multiple keys. Names follow the existing settings
loader's snake_case table-key rules. No `yetlet sign`, plugin registry or cloud
SDK is introduced. Ordinary applications call the signing service directly.

Use the existing base/profile settings loader with a top-level `signing` section,
separate from `cli`. Base declares every configured provider/key and a `versions`
array; the selected profile can replace that declared array without weakening
the loader's unknown-key rules. Passwords come from an explicitly named process
environment variable, never public TOML, argv or diagnostic output.

`status` is read-only and requires no private-key password. Report certificate
dates and availability without treating certificate renewal as key rotation or
inventing an expiry date for a bare public key. Signing does not create keys.

`rotate` shows the exact provider, logical key, current version and selected
settings file. Interactive approval is required unless `--yes` supplies explicit
automation approval. Without a terminal or explicit approval, fail before
effects. Bind the operation to its inspected input snapshot; concurrent changes
must invalidate the plan rather than overwrite another operator's settings.

An explicit `self_signed_test` local mode can generate fresh synthetic material
through OpenSSL, including first initialization of an empty key. An imported
identity requires replacement PFX supplied with `--from`; its certificate and
public-key fingerprint are part of the confirmation plan. It cannot silently
become a locally self-signed identity or acquire a CA-issued replacement without
the existing issuance process. Certificate renewal reusing a key is not a new,
independently revocable key version.

After confirmation, validate new material and trust, publish an immutable private
version, then atomically update only the selected key's public version records
and `active_version` in the selected profile TOML. Preserve comments, other keys,
other providers and unrelated settings. Retain old public material and mark its
signing policy retired. Never automatically delete an old private key. A failed
settings update leaves the previous active selection intact and must not report
successful activation. Environment overrides masking the writable selection,
unsafe files, unsupported providers and read-only configuration fail explicitly.
The shared loader gains no general-purpose write API.

Updating settings affects the next application settings load; the CLI does not
claim a running application has reloaded. The first version requires an explicit
application restart. Creation/import/rotation and private-key destruction remain
separate from the normal signing-only backend capability.

## Trust and failure behavior

Resolving a public key is not authorization to trust it. Verification authorizes
the exact key version under configured trust policy before reporting acceptance.
The local example uses explicit leaf-certificate pins, checks key/certificate
matching, the supported curve/algorithm, certificate dates and signing key usage.
It makes no OS trust-store, general chain-building, revocation-service, timestamp
or historical certificate-validity claim. Certificates bundled in PFX are not
implicitly trusted. Future KMS key metadata need not have an X.509 certificate.

Use closed nonsecret failure categories for invalid material/password, unknown
or disallowed key version, unsupported algorithm, invalid signature, policy
rejection and backend unavailability. Never expose library error chains, private
key data, passwords or vendor credential details. A timeout does not prove the
remote signer performed no operation. Core does not silently retry or fall back.

## Established cryptographic implementation

Use a local implementation using the maintained third-party
[Rust OpenSSL bindings](https://github.com/rust-openssl/rust-openssl), targeting a
supported patched [OpenSSL release](https://openssl-library.org/source/).
The local Linux verification uses an isolated static build of OpenSSL 3.5.9 LTS
and explicit `OPENSSL_DIR`, because published vendored source packages lag the
current LTS patch. No upstream crate or cryptographic implementation is patched.
Record the resolved binding/source versions and native build prerequisites in
implementation evidence.
The binding exposes [PKCS#12 parsing](https://docs.rs/openssl/latest/openssl/pkcs12/struct.Pkcs12.html)
and [sign/verify APIs](https://docs.rs/openssl/latest/openssl/sign/index.html).
It is not an official OpenSSL Rust SDK, and using it is not by itself an audit,
FIPS qualification or application-security guarantee.

The local implementation must delegate cryptographic arithmetic, hashing,
randomness, key encryption, PEM/PFX decoding and signature encoding to maintained
libraries. Pin the resolved app lockfile, review current advisories, document the
supported PFX/PEM profiles and test positive/negative interoperability. No custom
curve, ASN.1 parser, padding or certificate-chain algorithm is proposed.

RustCrypto is a candidate for a justified narrower implementation, not selected
merely to avoid native compilation. A new/pre-release PKCS#12 parsing stack needs
its own maturity assessment; pure Rust alone does not establish assurance.

The [binding build requirements](https://docs.rs/openssl/latest/openssl/) include
OpenSSL headers/libraries, or C/Perl/make for a vendored build. Native Windows
packaging must be verified separately; YT-092's Zig/SQLite proof does not qualify
OpenSSL. No global tooling installation is authorized by this document.

## Required implementation evidence

Use disposable synthetic identities created at runtime. Test PEM/PFX loading,
wrong passwords, key/certificate mismatch, unsupported/malformed material,
changed input/signature, algorithm/version rejection, certificate dates, explicit
trust, rotation retaining old verification material and rejection after revocation.
Reject duplicate public keys under different versions, including different
certificates containing the same key and reintroduction of a revoked key.
Two independently constructed local providers must satisfy the same contract.
Provider replacement is not proven for a cloud service until that adapter has
its own native/integration qualification.

The manual sample must run/build through the existing Yetlet CLI and show clear
sign/verify/tamper/rotation outcomes. Linux and Windows native checks plus an
independent Reviewer PASS remain required before YT-093 is Done. No cloud
resource, real credential, certificate custody or release signing is included.

## Official provider references

- [Azure key identifiers and versions](https://learn.microsoft.com/en-us/azure/key-vault/general/about-keys-secrets-certificates): immutable versions differ from versionless aliases.
- [Azure sign/verify semantics](https://learn.microsoft.com/en-us/azure/key-vault/keys/about-keys-details): sign operates on a digest; private key export is not required.
- [OpenBao Transit API](https://openbao.org/api-docs/secret/transit/): future adapter reference only; no client, service or compatibility claim is implemented here.
