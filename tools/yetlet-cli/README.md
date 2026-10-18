# Yetlet CLI — Unpublished Linux/Windows Developer Preview

The CLI creates applications and executes selected tasks. Persistent behavior uses
Yetlet's existing public base/profile settings and scoped environment overrides.
This experimental preview is unpublished. Its emitted version declarations are
not usable from a registry before YT-096/YT-020 qualification. Local evaluation
uses explicit Cargo dependency overrides; no framework source flag is needed.
See the [CLI contract](../../docs/CLI-CONTRACT.md) and
[implementation evidence](../../docs/YT-102-CLI-IMPLEMENTATION.md).

## Current command and capability inventory

| Command | Implemented behavior |
|---|---|
| `yetlet --help` / `-h` | Root help; every complete command path also has exclusive scoped help |
| `yetlet --version` / `-V` | Build-time CLI version; no project, tools or settings required |
| `yetlet new NAME [--parent DIRECTORY] [--vite] [--tailwind] [--service] [--settings]` | Publish a new scaffold; no tools, installs, builds or lock generation |
| `yetlet doctor [--json]` | Bounded environment-only Rust/Cargo identity and compiled filesystem capability; schema 1 JSON |
| `yetlet run [--manifest-path PATH]` | Frozen Cargo run, with independent settings-controlled Rust watch and Vite supervision |
| `yetlet watch [--manifest-path PATH]` | Same run pipeline with Rust watch enabled for this invocation; Vite still uses settings |
| `yetlet build [--manifest-path PATH] [--release]` | Optional frontend production build followed by frozen Cargo build |
| `yetlet generate docker [--manifest-path PATH] --output DIRECTORY` | Qualified-source configuration/inventory only |
| `yetlet generate compose [--manifest-path PATH] --output DIRECTORY --image IMAGE` | Configuration for a separately built Linux/amd64 image |
| `yetlet generate k8s [--manifest-path PATH] --output DIRECTORY --image IMAGE` | Deployment/Service configuration; replicas use settings, default 1, range 1–100 |

Value options accept separate or equals forms; NAME may appear anywhere among
new's options. Booleans are presence-only. Unknown, duplicate, empty, misplaced
or removed options fail before effects. Help/version are exclusive. There is no
argument forwarding, shell expansion, ancestor project search or implicit install.
Manifest defaults to exactly cwd/Cargo.toml. `--parent` selects an existing
creation parent; `--output` selects the complete new generator destination.
Paths remain literal, including supported native Unicode and spaces.

## Build locally and create an application

```sh
cargo build --offline --locked --manifest-path tools/yetlet-cli/Cargo.toml
tools/yetlet-cli/target/debug/yetlet --help
tools/yetlet-cli/target/debug/yetlet doctor --json
mkdir -p -m 700 "$HOME/yetlet-apps"
tools/yetlet-cli/target/debug/yetlet new sampleapp --parent "$HOME/yetlet-apps"
```

Use an existing protected parent: Linux owner-protected directories or trusted
sticky creation parents; Windows protected local fixed NTFS without reparse
traversal. Creation never overwrites or merges an existing destination.

The CLI has its own Cargo workspace and lock. New apps have their own standalone
workspace and no Cargo.lock. Current Yetlet crate versions are unpublished:
explicitly replace generated dependencies with compatible local Cargo path
entries or patches, using the generated README's example. Core/server use
0.1.0 and config uses 0.0.0. Then prepare the app's lock separately:

```sh
cd "$HOME/yetlet-apps/sampleapp"
cargo generate-lockfile --offline
cargo check --frozen
yetlet build
yetlet run
```

The example assumes the locally built CLI is explicitly available as `yetlet`.
It does not install it globally. Missing cached dependencies fail offline.
`--release` changes only the Cargo compilation profile, not APP_ENVIRONMENT.
Frozen execution is not a sandbox for package scripts, build scripts or app code.

## Creation choices and settings

Plain `new` remains HTML-first SSR with no mandatory JS/WASM or app settings
module. `--settings` adds typed runtime configuration. **`--vite` implies
`--settings`**, and generates the complete frontend production/development
integration. `--tailwind` selects the official Vite plugin with Vite, or the
existing separately prepared standalone Tailwind 4.3.3 path otherwise.
`--service` adds app-owned health/readiness and graceful shutdown. Controllers,
DI, sessions and other capability modules remain optional.

Every new app has public base.toml and development/staging/production overlays.
Every scaffold declares `cli.rust.watch = false`. A plain scaffold also
declares disabled Vite settings without a runtime config crate. `new webapp --vite` declares:

```toml
[cli.rust]
watch = false

[cli.vite]
enabled = true
root = "frontend/"
auto_start = true
auto_build = true

[cli.generate.k8s]
replicas = 1
```

The shared `yetlet-config` loader applies base, exactly one selected overlay,
and real scoped ENV overrides. APP_ENVIRONMENT selects Development, Staging or
Production and defaults to Production. Examples include
APP_CLI__RUST__WATCH=true, APP_CLI__VITE__AUTO_START=false, APP_CLI__VITE__AUTO_BUILD=false,
APP_CLI__VITE__ROOT=web and APP_CLI__GENERATE__K8S__REPLICAS=3. Values override
existing declared keys; invalid keys/types and partial settings fail safely.
CLI behavior is not stored in Cargo metadata.

CLI public settings default to the selected app root. Explicit absolute
YETLET_CONFIG_ROOT can relocate the public settings root. Vite root always
resolves relative to the selected Cargo.toml, including when settings move.
Legacy apps without CLI declarations retain disabled-watch/disabled-Vite/default-generator
behavior; an absent cli.rust section also defaults to watch disabled; their source and settings are not rewritten automatically.

App runtime settings retain APP_SERVER__HOST/PORT, APP_SERVICE__TOKEN and the
existing explicit Development-only YETLET_DEV_SECRETS mechanism. No usable token
is generated; settings-enabled applications require an operator-provided token
before startup. CLI tool decisions never open that secret file. Backend `.env`
auto-discovery is disabled. See [configuration](../../docs/CONFIGURATION.md).

## Vite execution

Frontend files live under frontend/ by default: package.json, pinned package
lock, production/dev Vite configs, source assets and a public .env.example.
Change `cli.vite.root` after moving that directory to a protected app-relative
location. Traversal, unsafe paths, symlinks and reparse escapes are refused.
Frontend `.env*` discovery is confined to this root; VITE_* values imported into
the browser are public. Backend APP_* variables and secret/config selectors are
removed from the frontend child environment. The CLI supplies its selected app
root as YETLET_PROJECT_ROOT for Tailwind Rust-source discovery. Direct npm at a
custom nested root needs this explicit absolute context; the default frontend/
location has a relative source fallback.

Prepare Node, npm and dependencies separately, for example an explicit
`npm ci --offline` in frontend/ when the cache is already complete. CLI actions
never invoke install/ci or refresh either package lock.

`build` selects only `enabled && auto_build`. When selected, it runs
`npm run build` in the configured frontend root and waits for success before
frozen Cargo compilation. Frontend failure preserves its status and skips Rust.
When disabled, it reports that the frontend stage was skipped; this does not
claim missing/stale assets are ready.

`run` and `watch` select Vite through `enabled && auto_start`, independently of Rust watch. When selected, they start the strict
loopback Vite server on 127.0.0.1:5173, waits for bounded readiness, and supervises
it together with frozen Cargo run. The generated read-only bridge expects the
backend on 127.0.0.1:3000; keep the corresponding app settings when using it.
Build initial frontend dist separately before startup. Rust snapshots and
validates dist; auto-start does not silently perform a production frontend build.
With Rust watch disabled, both owned trees are stopped/reaped when either child
exits or the CLI is cancelled. Browser launch is not added.

Disable auto_start for the manual two-terminal workflow: set YETLET_VITE_PORT=5173
for the Rust process and start `npm run dev` separately in the frontend root.
The bridge remains root-page/read-only, with strict origin/path/CSP/nonce bounds;
no authentication, cookies, POST/actions, arbitrary proxy or multi-page support.
Production uses versioned static assets and no Node runtime.

## Rust watch

Rust watch is disabled by default for `run` and independent of Vite. Prepare
Watchexec on PATH separately (integration targets version 2.8.0); the CLI never
installs it. Use `yetlet watch` to enable watch for this invocation, even when
`cli.rust.watch` is false or absent. It shares `run` and validates settings
strictly before applying the override; settings files are unchanged.

To make watch the default for `yetlet run`, enable the existing base declaration
through the selected profile:

```toml
[cli.rust]
watch = true
```

Then run `yetlet run`, or use `APP_CLI__RUST__WATCH=true yetlet run` with the
base key declared. `--settings` still only selects generated runtime settings;
watch uses public CLI settings even for plain SSR apps. There is no watch flag
or Cargo metadata configuration. `build` never starts watch and requires no
Watchexec, even when this setting is true.

The CLI starts frozen/offline Cargo immediately. Watchexec emits JSON file events;
the CLI stops/reaps the old Rust tree and starts a fresh generation on a relevant
change. Compile errors and Rust runtime exits are reported, then wait for the
next change. Vite keeps the same process through Rust restarts. Cancellation,
Vite exit or watcher death ends the session and cleans up all owned trees.

Watch covers app-root Rust `*.rs` (including build.rs), Cargo.toml and Cargo.lock.
It excludes the configured frontend root, target, node_modules and dist, plus
CARGO_TARGET_DIR when under the app root, and does not follow symlinks. Outside-app
local dependencies are not watched. Special target directories from Cargo config
need separate exclusion if required; select CARGO_TARGET_DIR for an excluded
in-root location or put the directory outside the watched root.

CLI settings are loaded once per run; rerun the CLI to change watch/Vite policy.
The fresh backend reads its runtime settings on each restart. Browser refresh,
in-process state preservation and graceful request draining are not promised.
Historical Vite lifecycle evidence does not qualify the added watch mode.

## Current generator source matrix (YT-101)

Generators retain their separately qualified, **unedited legacy local-source**
representation, consistent app lock and source revalidation. Retained internal
legacy scaffold functions support its acceptance fixtures. These source
selections are historical labels, not current CLI options:

| Qualified legacy source | Delivery requirement |
|---|---|
| Native service SSR | Established source/lock closure |
| Service with typed settings | Exact public TOMLs; runtime token supplied separately |
| Service with production Vite, optional settings/plugin Tailwind | Explicitly prebuilt and validated flat dist |

New shared CLI settings, complete nested frontend output and registry version
representation **do not inherit generator acceptance**. Standalone Tailwind,
legacy dev bridge, edited/general apps and unknown dependencies/layouts remain
refused. See [YT-101 evidence](../../docs/YT-101-VERIFICATION.md).

Generators run no npm, Cargo, Docker, kubectl, image pull/build, source export or
deployment. Runtime tokens are not inspected. Configuration publication is
protected and never replaces an existing output. Docker image preparation still
requires three separate contexts: generated config, sanitized app-src and
sanitized yetlet-src; live checkout/app trees are not build contexts. Public
config/assets have the recorded bounded validation, not authenticated provenance.
Compose uses separately built local images; K8s creates no runtime Secret.
Image references require a validated explicit registry/repository and non-latest
tag or sha256 digest. K8s package names retain unchanged DNS-label restrictions.

Replicas use the same public settings loader. A qualified legacy app can select
an explicit separate public YETLET_CONFIG_ROOT with declared cli settings;
adding new namespace/layout fields to legacy app sources does not bypass exact
source inspection. Existing token interpolation/Secret references, Linux/amd64
security defaults and operator-owned drain/retention limits remain unchanged.

## Opt-in Tailwind CSS 4 (YT-091 independently reviewed)

Use `new NAME --tailwind`; add `--vite` for the official Vite plugin. Without
Vite, prepare the pinned standalone tool and CSS build explicitly using the
[generated Tailwind instructions](templates/tailwind/SELECTED-README.md).
[YT-091 evidence](../../docs/YT-091-VERIFICATION.md) retains the historical
source-bound native/browser acceptance; it does not qualify the new settings
representation.

## Windows PowerShell (native GNU/MSVC)

The CLI owns configured startup, stop/cleanup and build orchestration. Vite owns
module compilation, dependency tracking and HMR. Frontend JS/TS files may use
custom names and subdirectories without registration; physical root confinement
still refuses private/tooling files and outside links. The default entry remains
`app.ts`. [Integration evidence](../../docs/YT-102-STATEFUL-HMR-IMPLEMENTATION.md)
separates the earlier Windows browser checks from the current Linux follow-up.

```powershell
cargo build --offline --locked --manifest-path .\tools\yetlet-cli\Cargo.toml
$Yetlet = Join-Path (Get-Location) "tools\yetlet-cli\target\debug\yetlet.exe"
& $Yetlet --help
& $Yetlet new sampleapp --parent C:\ProtectedApps
# Prepare compatible local Cargo overrides and the app lock as its README describes.
& $Yetlet build --manifest-path C:\ProtectedApps\sampleapp\Cargo.toml
& $Yetlet run --manifest-path C:\ProtectedApps\sampleapp\Cargo.toml
```

Choose an existing owner-protected local fixed NTFS parent. Do not disable ACL,
reparse or executable checks. The [native Windows GNU pair record](../../docs/YT-102-WINDOWS-VITE-VERIFICATION.md)
covers six selected real Rust/Vite lifecycle cases; MSVC evidence remains compile-only.

## Diagnostics and platform evidence

CLI-owned statuses are 0 success/help, 1 preflight/report, 2 usage and 125
spawn/supervision/cleanup failure. Native child exits and cancellation retain
identity after verified cleanup. Safe scoped CLI errors use stable codes and
stderr; child-failure summaries identify their stage without guessing lock/cache
causes. Child stdout/stderr remains native and is not guaranteed secret-redacted.

Doctor reports schema 1 environment checks only, never application, asset,
frontend, console, generator or release readiness. Fixed local Rust/Cargo probes
have 3-second/64-KiB combined-output bounds plus 2-second cleanup. Unsafe or
unavailable executables fail; group-writable toolchain paths are refused on
Linux. No installation/update, shell, app settings or Node probes occur.

Linux has current native process/PTY/cleanup tests. Current Windows GNU
[doctor and CLI evidence](../../docs/YT-102-DOCTOR-VERIFICATION.md) and
[Rust/Vite pair evidence](../../docs/YT-102-WINDOWS-VITE-VERIFICATION.md) supplement
the separate MSVC compile-only checks. Windows run still requires an attached console
with processed Ctrl-C input and protected app/tool paths; npm uses vetted native
node.exe plus its same-install npm CLI script, without a .cmd shell fallback.
Escaped daemons and unsupported OS/filesystem/console contexts remain unsupported.
Public release, production qualification, global installation and deployment
remain outside this preview.
