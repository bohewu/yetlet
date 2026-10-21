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

Rust watch is disabled by default for `yetlet run`. Prepare Watchexec 2.8.0 on
PATH separately, then use `yetlet watch` for one invocation. It shares `run`,
keeps Vite selection in settings and leaves settings files unchanged. To make
watch the default for `run`, override the existing base key in the selected profile:

```toml
[cli.rust]
watch = true
```

Run `yetlet run` (or `APP_CLI__RUST__WATCH=true yetlet run`). The CLI restarts only
the Rust tree on app-root Rust/Cargo file changes; Vite stays running. Compile
errors and backend exits wait for the next change. `yetlet build` never watches
and needs no Watchexec. CLI policy changes require rerunning the CLI. External
local dependencies are not watched; browser refresh, backend state preservation
and graceful request draining are not promised.

The app owns ordinary Rust handlers and escaped HTML with strict CSP. No
Controller, DI container or provider registry is required. Stop foreground
execution with Ctrl-C. Native Windows runtime qualification remains separate.

With `--service`, the unedited selected scaffold supports configuration-only
Docker, Compose and Kubernetes generation after compatible explicit local Cargo
path overrides and separate lock preparation. Vite additionally needs an
explicit build in the configured Production frontend root before generation.
Watch and automatic frontend execution settings do not start tools here:

```text
yetlet generate docker --output /work/out/docker
yetlet generate compose --output /work/out/compose --image localhost:5000/team/app:v1
yetlet generate k8s --output /work/out/k8s --image localhost:5000/team/app:v1
```

Select new protected output directories. Follow the generated Docker README's
explicit physical export and hash checks, including `app-src-source-paths` for
frontend assets. Generation does not build, export, publish or deploy an image.
Registry-only dependency delivery and arbitrary edited app layouts require
separate qualification.
