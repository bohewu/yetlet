
## Application settings

The selected settings module loads typed server/service and known public CLI
settings, rejecting unknown fields. Set `APP_SERVICE__TOKEN` to your own value
before startup; no usable token is generated. The token never enters frontend
source or SSR. Keep credentials out of public TOMLs.

`YETLET_CONFIG_ROOT` can select an explicit absolute common settings directory.
The default remains this app's build-time `CARGO_MANIFEST_DIR`. Development
secret loading requires both `APP_ENVIRONMENT=Development` and an explicit
absolute `YETLET_DEV_SECRETS` path; no backend dotenv discovery occurs. CLI
public-settings selection never reads that secret file.

### Development HTTPS

Prepare mkcert separately. Installing its CA with `mkcert -install` is an
explicit local trust action; the CLI never installs tools or trust. Create a
private `.dev/certs/` directory outside frontend sources, then run:

```text
mkcert -cert-file .dev/certs/localhost.pem -key-file .dev/certs/localhost-key.pem localhost 127.0.0.1 ::1
```

Set these existing declarations in `development.toml`:

```toml
[server.tls]
enabled = true
certificate = ".dev/certs/localhost.pem"
private_key = ".dev/certs/localhost-key.pem"
```

With `APP_ENVIRONMENT=Development`, `yetlet run` selects the optional
`yetlet-dev-tls` Cargo feature and serves HTTPS. With automatic Vite startup,
use `https://127.0.0.1:5173`; Vite owns HTTPS/WSS and Rust remains private HTTP
on `127.0.0.1:3000`. Otherwise Rust serves HTTPS on its configured port.
`yetlet build` never opens development certificate/key files. Prepare the lock
and dependency cache separately; `.dev/` stays outside version control.
