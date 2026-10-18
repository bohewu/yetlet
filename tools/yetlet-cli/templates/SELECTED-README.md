# @NAME@

Experimental Yetlet SSR application scaffold. The CLI release @VERSION@ emits
exact template crate versions (core/server 0.1.0; config 0.0.0 when selected). These crates are **unpublished and not
usable from a registry until YT-096/YT-020 release qualification**. No registry
availability, production support or clean-source identity is asserted.

For local evaluation, explicitly replace the Yetlet dependencies with compatible
Cargo path declarations (or add Cargo patches), then prepare this application's
own lockfile. Dependency overrides belong to Cargo; the CLI does not locate a
framework checkout. For example, set `path` alongside the matching `version` in
each selected dependency (replace the paths explicitly):

```toml
yetlet-core = { version = "=0.1.0", path = "/absolute/yetlet/crates/yetlet-core" }
yetlet-server-axum = { version = "=0.1.0", path = "/absolute/yetlet/crates/yetlet-server-axum" }
# Only when settings/Vite was selected:
yetlet-config = { version = "=0.0.0", path = "/absolute/yetlet/crates/yetlet-config" }
```

Provision dependencies separately before frozen commands:

```sh
cargo generate-lockfile --offline
cargo check --frozen
yetlet build
yetlet run
```

Creation writes files only: no Cargo lock, package installation, build, download
or listener. `base.toml` and the three profile TOMLs declare public CLI behavior.
Plain SSR leaves Vite disabled and needs no Node runtime. `APP_ENVIRONMENT`
selects one profile and defaults to Production; `--release` selects only Cargo's
build profile. APP_CLI overrides must name keys already declared in base.toml.

The app owns ordinary Rust handlers and escaped HTML with strict CSP. No
Controller, DI container or provider registry is required. Stop foreground
execution with Ctrl-C. Native Windows runtime qualification remains separate.
New shared-settings/release/frontend layouts are outside the deployment
generators' qualified legacy-source representation and are refused.
