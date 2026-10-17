# @NAME@ — opt-in typed configuration preview

This standalone, unpublished native SSR application uses local Yetlet source
paths and the finite YT-090 loader. No JS, DI container, secret discovery,
authentication implementation or production qualification is included.
`new` only wrote synthetic files. It did not read environment values or secret
files, install tools, create Cargo.lock, compile, contact a network or deploy.

## Explicit offline build

Use an installed Rust/Cargo 1.99 toolchain and a populated local dependency cache.
From this application directory:

```sh
cargo generate-lockfile --offline
cargo check --offline --locked --all-targets
cargo test --offline --locked
```

Keep this application's own Cargo.lock. Subsequent `yetlet build/dev` commands
require that lock and use frozen Cargo. Cache misses fail without network
fallback. Local path dependencies are unpublished and not registry releases.

## Operator-selected configuration

The app reads `base.toml` and exactly one of `development.toml`, `staging.toml`,
`production.toml` from the absolute build-time `CARGO_MANIFEST_DIR` application
root, independent of runtime cwd. Move/rebuild and provision those files
explicitly when relocating the app; the binary does not embed their contents.
`APP_ENVIRONMENT` accepts exactly `Development`, `Staging`, `Production` and
defaults to Production. Precedence is base, selected overlay, explicitly selected
development file, then real `APP_*` environment overrides. Unknown/malformed
keys and invalid values fail with bounded, fixed errors before listener creation.

No usable secret is generated. The required `service.token` starts empty;
startup fails until the operator injects `APP_SERVICE__TOKEN` or selects a
private Development file. No key is generated and `.env.example` is an empty,
synthetic declaration, not a working credential. Supply values using your
operator-controlled environment or secret delivery tooling. For a disposable
local smoke test only, a synthetic value can be used:

```sh
APP_ENVIRONMENT=Development APP_SERVICE__TOKEN=000123 cargo run --offline --locked
```

`000123` remains a string, including leading zeros. The token is a SecretString;
Settings has no Serialize/Debug implementation. Only `service.public_message`
enters escaped SSR with strict CSP. This example does not authenticate requests.
Do not add secrets to public messages, checked-in TOML, logs, HTML or browser state.

The non-cloud app accepts only `APP_SERVER__HOST=127.0.0.1`. The cloud app also
accepts an explicit `0.0.0.0`; neither profile accepts arbitrary hostnames/addresses.
`APP_SERVER__PORT` is a typed integer in 1..=65535 (default 3000). Configuration
profiles do not change the selected native/cloud source profile. `YETLET_HOST`
and `YETLET_PORT` belong to the original no-config template; use `APP_SERVER__*`
for this application.

## Optional Development file

There is no automatic `.env`/`.env.local` lookup or shell sourcing. To use a
private local file, create it yourself outside version control, with an explicit
absolute path and owner-protected file/parent permissions (Unix 0600/0700 or
Windows ACLs). The loader does not audit those permissions. Set
`YETLET_DEV_SECRETS` to that path **and** select `APP_ENVIRONMENT=Development`.
The selected file is required and must contain known APP overrides, for example
`APP_SERVICE__TOKEN=your-operator-supplied-value`. Do not copy that illustrative
text as a real credential. OS overrides have higher priority. Dollar signs are
rejected by the finite dotenv adapter; no interpolation occurs.

Staging and Production do not inspect the development path, even if the path
variable is set. A missing Development file, relative selected path or missing
required token fails startup. `.env.local` remains ignored; `.env.example` alone
is safe to track. Never use the example filename for real secrets.

PowerShell, using a synthetic local smoke value:

```powershell
cargo generate-lockfile --offline
$env:APP_ENVIRONMENT = 'Development'
$env:APP_SERVICE__TOKEN = '000123'
# Optional instead of the token variable: select your own protected absolute file.
# $env:YETLET_DEV_SECRETS = 'C:\private\myapp.env'
cargo run --offline --locked
Remove-Item Env:APP_SERVICE__TOKEN
Remove-Item Env:APP_ENVIRONMENT
```

Use a protected local fixed NTFS application directory and trusted Cargo for
Windows CLI build/dev. Stop the foreground application with Ctrl-C.

## Cloud boundary

When created with `--cloud --with-config`, the app retains empty-body local
`/health/live` and `/health/ready`, bounded bind selection and pre-registered
SIGTERM/SIGINT (Unix) or Ctrl-C/Ctrl-Break (Windows) handlers. Readiness becomes
unavailable when graceful shutdown begins; closed listeners may refuse probes.
Health does not qualify external dependencies. Axum drain has no automatic hard
deadline; deployment withdrawal/termination budgets require separate design.
The non-cloud profile supplies SSR only, without these cloud health/signal hooks.

**The configuration cloud app is a separate application contract.** Existing
`generate docker`, `generate compose` and `generate k8s` accept only the exact
original unmodified cloud template and fixed dependencies. They reject this
configuration application. The retained cloud-profile marker does not widen
that acceptance. Container/configuration delivery, sanitized settings/secret
injection, images and deployment require a separate reviewed design. No
publication, deployment or YT-020 release activation follows from scaffolding.
