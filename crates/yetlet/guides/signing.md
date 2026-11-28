# yetlet::signing

Experimental module included in `yetlet` SDK `1.0.0-rc.1`. Requires Rust 1.99+ and
a native OpenSSL development/runtime configuration compatible with the pinned
Rust bindings. Select the `signing` feature explicitly; Core/HTML/browser consumers do
not acquire it automatically. Publication eligibility does not activate release
signing, certificates, public logs or a registry upload.

## Prerequisites

Use source/rustdoc matching your selected SDK version. The selected
reference is GNU/Linux; Windows, macOS and browser Wasm are not qualified by
this package preparation. The bounded native signing evidence uses an isolated static
OpenSSL 3.5.9 build with `openssl 0.10.81` / `openssl-sys 0.9.117`; that is a
recorded recipe, not a promise that every installed OpenSSL is compatible.

## Minimal integration

```rust
use yetlet::signing::Sha256Digest;

let digest = Sha256Digest::of(b"explicit application bytes");
assert_eq!(digest.0.len(), 32);
```

The example hashes exact bytes. It does not select a signing identity, generate
or persist a key, contact a provider or establish trust. For signing, inject a
`SigningBackend` and `PublicKeySource` into the documented service/verifier with
explicit algorithm, versioned key and caller-owned policy.

## Expected result, configuration and errors

The local implementation delegates cryptographic and material parsing work to
OpenSSL. Public key/certificate membership in `PublicKeySet` does not establish
trust; pin and policy decisions remain explicit. Certificate leaf checks are
not general PKI chain/revocation validation or trusted signing-time evidence.
Local state exposes no mandatory network, filesystem service or automatic key
rotation. Protect private material and passwords in the consuming application.

Typed errors distinguish invalid material/signature, unknown key, unsupported
algorithm, rejected policy and unavailable backend, without exposing raw native
diagnostics. Provider adapters and actual production custody are separate choices.

## API reference and license

Read source/rustdoc at the selected revision for signatures and ownership. The
signing service requires explicit backend/public trust and key-policy custody.
Refer to source/rustdoc for the selected SDK build; a floating repository URL
is not a portable package attestation. Dual licensed under [MIT](https://github.com/bohewu/yetlet/blob/master/crates/yetlet-signing/LICENSE-MIT) or
[Apache-2.0](https://github.com/bohewu/yetlet/blob/master/crates/yetlet-signing/LICENSE-APACHE).