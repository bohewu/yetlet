# Automatic certificates with ACME

The experimental source-checkout feature `acme` enables `web-tls` and
manages one DNS identity through HTTP-01. An optional external account binding
(EAB) supplies an account key identifier and decoded MAC key bytes during
account registration.

## Compose the site

Pass explicit HTTP and HTTPS routers. `AcmeHost::http01` reserves
`/.well-known/acme-challenge/{token}` on the plaintext router and returns the
plaintext Web Host and the ACME Host. Add both to the same async `HostGroup`
on a Tokio runtime with I/O and time enabled.

```rust,no_run
use axum::{Router, routing::get};
use yetlet::{HostGroup, acme::{AcmeConfig, AcmeHost}};

async fn serve() -> Result<(), Box<dyn std::error::Error>> {
    let config = AcmeConfig::new(
        "service.example.test",
        "https://ca.example.test/directory",
        "private-acme-state",
    )?;
    let plaintext = Router::new().route("/health", get(|| async { "ready" }));
    let secure = Router::new().route("/", get(|| async { "hello over HTTPS" }));
    let (http, acme) = AcmeHost::http01(
        config,
        plaintext, "127.0.0.1:8080".parse()?,
        secure, "127.0.0.1:8443".parse()?,
    );
    let status = acme.handle();
    let mut group = HostGroup::new().with_host(http).with_host(acme).build()?;
    assert!(status.status().https_address.is_none());
    group.run_async().await.map_err(|error| std::io::Error::other(error.to_string()))?;
    Ok(())
}
```

The CA must be able to fetch the challenge from the requested name. Point
public HTTP port 80 at the plaintext listener when using a public CA.
See the [HTTP-01 specification](https://www.rfc-editor.org/rfc/rfc8555.html#section-8.3).

The state directory contains the account credential, certificate private key,
and durable retry state. Give the application a dedicated, private directory
with an existing parent. Unix directories are created with mode 0700. A
directory is exclusive to one running ACME Host, DNS name and directory URL.
Changing the CA or DNS name requires a different state directory.

The plaintext Host starts serving before issuance. HTTPS starts after a stored
or newly issued identity verifies against the configured name and trust roots.
Renewal replaces the identity for new TLS handshakes. Active connections keep
their established sessions. If an automatic attempt fails, the last failure
category and next retry deadline persist, including before the first successful
issuance. A still-valid certificate can load while the Host waits to retry.
On restart, an expired certificate remains ineligible for HTTPS.

## Run the example

Build `cargo build -p yetlet --features acme --example acme_http01 --locked`.
The example reads `ACME_DOMAIN`, `ACME_DIRECTORY` and `ACME_STATE_DIR` and
prints public status observations. Set `ACME_HTTP_BIND` and `ACME_HTTPS_BIND`
to select the listen addresses. The defaults are loopback ports 8080 and 8443.
For a private CA, set `ACME_DIRECTORY_ROOT_PEM` and
`ACME_ISSUANCE_ROOT_PEM` to the respective transport and issuance roots.
The platform's trusted roots are the defaults. Set `ACME_EAB_KEY_ID` and
`ACME_EAB_KEY_FILE` together when the CA requires EAB. The file contains
decoded MAC key bytes. For a CA with subscriber terms, set
`ACME_AGREE_TO_TERMS=1` after accepting them.

Run the example with
`cargo run -p yetlet --features acme --example acme_http01 --locked`.
Use `AcmeConfig::with_renew_before` and `with_retry_policy` to select the
renewal lead and retry delays independently. The default backoff starts at
one minute and doubles up to 24 hours.
