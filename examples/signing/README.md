# Local signing reference — YT-093

This opt-in, nonpublishable sample demonstrates exact-byte signing and public-only
verification using the experimental `yetlet::signing` crate and OpenSSL. It is a
console application, with no HTTP listener, UI, cloud SDK or real certificate.
The sample does not implement CMS, XMLDSig, PDF signing, PKI chain validation or
trusted historical signing time.

## Prepare dependencies explicitly

Prepare Rust/Cargo 1.99 and supported patched OpenSSL headers/libraries using the
[Rust OpenSSL build instructions](https://docs.rs/openssl/latest/openssl/).
Use `OPENSSL_DIR` and `OPENSSL_STATIC=1` if selecting a private static installation.
The repository's local verification uses OpenSSL 3.5.9 LTS; a system executable
alone does not supply the development headers/libraries. Native Windows needs
its own supported OpenSSL installation and compiler setup.

Build the unpublished CLI explicitly from the repository root:

```sh
cargo build --locked --manifest-path tools/yetlet-cli/Cargo.toml
```

From this example directory, explicitly prepare the existing lock/dependencies:

```sh
cargo fetch --locked
```

Make the built `yetlet` binary available on PATH, or substitute its explicit
`../../tools/yetlet-cli/target/debug/yetlet` path (`yetlet.exe` on Windows).
The CLI performs no implicit dependency installation or lock refresh.

## Run the disposable application workflow

Select `APP_ENVIRONMENT=Development` and set `YETLET_SIGNING_PASSWORD` to a nonempty
disposable password through your shell or secret tooling. Do not put it in public
TOML, command arguments, source control or logs.

```sh
yetlet build
yetlet run
```

The application creates disposable identities in memory and reports encrypted
PEM/PFX signing, tamper rejection, documents v1-to-v2 rotation, independent
invoices signing, public-only verification and revoked-v1 rejection. It writes
no private files. This console demonstration is separate from the persistent
key-management workflow below.

## Manage persistent local test keys

The base settings explicitly select `local_dev` and `documents`. `invoices` is a
second independent logical key with its own versions and certificate. The two
tracked key directories contain only ignore rules; no private key or certificate
is committed. `mode = "self_signed_test"` explicitly permits generated synthetic
identities. It does not issue or renew a CA-signed certificate.

```sh
yetlet key status
yetlet key rotate
yetlet key status documents --provider local_dev
yetlet key rotate invoices --provider local_dev
yetlet key rotate documents --provider local_dev
```

`status` is read-only and does not require the private-key password. An empty key
is uninitialized. Its first explicit `rotate` initializes a new version; later
rotations create fresh key material. The command shows the exact target, current
version and selected settings file, then requires terminal confirmation. Use
`--yes` only when intentionally approving the operation in automation. A piped
`yes` is not terminal confirmation.

After validation, `rotate` publishes a new protected version and automatically
updates that key's `active_version` and `versions` array in the selected profile,
preserving other keys, providers, unrelated settings and comments. Previous
records remain available with signing retired. No old private key is deleted.
Restart a consuming application to load the new settings; a successful file
update does not claim an already-running process reloaded.

Keep private keys/passwords outside public asset trees and browser state. The
reference creates private version directories and encrypted private-key files.
`APP_SIGNING*` overrides are refused for rotation so a file update cannot be
masked by a different effective environment selection. `APP_ENVIRONMENT` and the
explicit absolute `YETLET_CONFIG_ROOT` keep their existing meaning. Base must
declare every provider/key and its `versions` array; the selected profile can
replace the array under the existing strict loader rules.

## Import a replacement identity

An operator-managed/CA-issued provider uses `mode = "imported"`. Obtain the new
PFX through the existing issuance process, then use a protected replacement file:

```sh
yetlet key rotate documents --provider local_dev --from replacement.pfx
```

The configured `password_env` supplies the archive password and protects its
exported encrypted PEM. Inspect and approve the displayed certificate fingerprint.
Wrong passwords, unsupported material and duplicate public keys are rejected.
Renewing a certificate with the same key cannot create an independently revocable
key version. No OS trust store, chain-building or automatic issuer integration is
claimed; configured leaf pins express the reference's trust policy.

Old signatures remain acceptable only while their version is trusted and their
certificate satisfies current validity policy. Retiring signing is different
from revoking trust. Long-term verification after certificate expiry requires a
separate timestamp/historical-validation design.

Azure Key Vault/OpenBao adapters and automatic following of approved remote
versions are future opt-in work. Unsupported providers fail explicitly. Reading
or signing does not trigger key creation or administrative rotation.
