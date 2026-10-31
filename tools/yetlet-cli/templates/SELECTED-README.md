# @NAME@

Experimental Yetlet SSR application scaffold. CLI @VERSION@ emits the selected
published SDK version @SDK_VERSION@. The generated manifest selects the registry
package and its required features (including `config` for Settings/Vite).
No framework checkout or Cargo path override is needed for normal build/run.

Provision dependencies separately before frozen commands:

```sh
cargo generate-lockfile
cargo fetch --locked
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
