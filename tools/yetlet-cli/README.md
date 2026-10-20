# Yetlet CLI — Unpublished Linux/Windows Developer Preview

The CLI creates applications and executes selected tasks. Persistent behavior uses
Yetlet's existing public base/profile settings and scoped environment overrides.
This experimental preview is unpublished. Its emitted version declarations are
not usable from a registry before YT-096/YT-020 qualification. Local evaluation
uses explicit Cargo dependency overrides; no framework source flag is needed.
See the [CLI contract](../../docs/CLI-CONTRACT.md) and
[implementation evidence](../../docs/YT-102-CLI-IMPLEMENTATION.md).
The subsequent [DX audit](../../docs/YT-102-DX-AUDIT.md) compares actual setup,
build/run/watch, diagnostics, testing and deployment-preview journeys and records
bounded costs and proposed improvements. Independent Reviewer PASS completes
YT-102 at research-only scope, with zero issues or required followups. CLI
behavior and wider qualification remain unchanged.
The separately selected [Linux completion-latency slice](../../docs/YT-102-COMPLETION-LATENCY.md)
avoids the natural-exit grace only after a conservative single-thread/pinned-child
check; cancellation and descendant cleanup retain their existing behavior.
Independent implementation Reviewer PASS has zero issues or required followups.

## Current command and capability inventory

| Command | Implemented behavior |
|---|---|
| `yetlet --help` / `-h` | Root help; every complete command path also has exclusive scoped help |
| `yetlet --version` / `-V` | Build-time CLI version; no project, tools or settings required |
| `yetlet new NAME [--parent DIRECTORY] [--template native\|browser] [--vite] [--tailwind] [--service] [--settings]` | Publish a new scaffold; no tools, installs, builds or lock generation |
| `yetlet doctor [--json]` | Bounded environment-only Rust/Cargo identity and compiled filesystem capability; schema 1 JSON |
| `yetlet key status [KEY] [--provider NAME] [--manifest-path PATH]` | Read-only selected key/version, certificate pins and dates; no password or generation |
| `yetlet key rotate [KEY] [--provider NAME] [--manifest-path PATH] [--yes] [--from PATH]` | Confirmed local rotation and atomic selected-profile settings update; retain old versions |
| `yetlet run [--manifest-path PATH]` | Native frozen Cargo/watch/Vite, or settings-selected browser-only static preview |
| `yetlet watch [--manifest-path PATH]` | Same run pipeline with Rust watch enabled for this invocation; Vite still uses settings |
| `yetlet build [--manifest-path PATH] [--release]` | Native frontend-then-Cargo, or settings-selected browser-only production build |
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

On Windows, executable selection requires absolute PATH entries without reparse
traversal, a trusted executable owner and no untrusted write access. A refusal
prints `CLI_PROCESS stage=executable` with the error kind and optional numeric OS
code before returning exit 125. Prepare a protected tool installation and select
its physical directory on PATH; tool permissions are managed separately from the CLI.
See the [native Windows diagnosis and verification](../../docs/WINDOWS-CLI-125-VERIFICATION.md).

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

The CLI has its own Cargo workspace and lock. Its local signing dependency requires
OpenSSL headers/libraries (the YT-093 reference targets OpenSSL 3.5 LTS); use
`OPENSSL_DIR` and `OPENSSL_STATIC=1` for an explicitly prepared static prefix.
No native tools or OpenSSL installation is performed by the CLI. New apps have their own standalone
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

YT-104 adds an explicitly selected [browser-only Rust/WASM task starter](../../docs/YT-104-BROWSER-STARTER.md):
`yetlet new tasks --template browser --tailwind`. This persists
`cli.target = "browser"` and includes Vite/public settings, with no native
server or app-runtime settings/secret crate. Existing `build`/`run` then manage
only the frontend; their automatic-stage booleans still apply.
`--template native|browser` selects creation only and defaults to `native`. `new` ignores
`APP_CLI__TARGET`; execution retains that normal settings override. The same
creation command works in POSIX shells and PowerShell. Plain creation keeps
the native behavior below.

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

## Local signing keys

See the [signing example](../../examples/signing/README.md) for the complete
public settings schema and disposable setup. Provider and key names are configured
snake_case aliases. Omitted names require explicit `signing.default_provider`
and `signing.default_key`; there is no guessed provider or key.

`key status` reads public certificate material only and reports certificate dates
separately from policy expiry. `key rotate` requires a protected existing
`<provider.root>/<key>` directory. This public parent may be owner-writable
0755 on Linux; newly created version directories remain 0700 and private files
0600. Windows versions use a protected owner-only DACL. Provider roots are
project-relative even when
`YETLET_CONFIG_ROOT` selects another settings directory. The configured
`password_env` names a nonempty separate environment source; passwords never
appear in CLI arguments, public TOML or diagnostics.

Rotation prints provider, key, old/new versions and the selected settings file.
Without `--yes`, both stdin and stdout must be terminals and the exact answer
`yes` approves the displayed plan. Noninteractive input and declines cause no
changes. `self_signed_test` mode generates a fresh 30-day test certificate;
`imported` mode requires `--from` with a protected replacement PFX encrypted under
the configured password. An imported certificate pin is displayed for approval.
This is replacement, not CA renewal. There is no `yetlet sign` or cloud client.

Rotation validates material, key matching, certificate usability and distinct
public keys across every configured historical identity before publishing an
immutable encrypted PEM version. It then atomically updates only the selected
profile's key version array and active version, retaining unrelated settings and
comments. Base declarations and other keys/profiles are preserved. Signing ENV
overrides are refused for rotation because they could mask the saved result;
status applies them through the normal shared loader. Retired/revoked public
identities remain available for duplicate detection and explicit trust policy.

A per-settings-directory exclusive lock and byte/identity revalidation reject
concurrent or stale plans. Failed activation retains the old settings when the
atomic replacement has not occurred, and retains any published new material for
inspection; it never deletes private keys. A failure after replacement reports
uncertain durability, not a rollback or success. The `.yetlet-key.lock` inode is
retained for safe cross-process locking. These are local filesystem guarantees,
not distributed policy synchronization or certificate custody qualification.

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

## Current generator source matrix

Current unedited `new --service` output now supports all three generators after
explicit compatible local Cargo path overrides and separately prepared locks:

| Current creation choice | Delivery requirement |
|---|---|
| `new NAME --service` | Public CLI settings; no runtime settings dependency or token |
| `new NAME --service --settings` | Public CLI/runtime TOMLs; runtime token supplied separately |
| `new NAME --service --vite` | Settings implied; complete explicitly built production Vite graph |
| `new NAME --service --vite --tailwind` | Same graph contract with the pinned official plugin |

Keep the generated pinned versions when adding the qualified framework's local
path dependencies. Registry-only images still require separate release/source
qualification. The generator validates all public CLI profiles; its asset input
uses the Production Vite root, which may differ from Development/Staging.
Watch, auto_start and auto_build never execute during generation. Docker's
inventory adds `app-src-source-paths` so a configured `frontend/dist` or custom
root can be explicitly exported to canonical `dist/`. Follow the generated
Docker README's physical allowlist/hash instructions. See the
[compatibility evidence](../../docs/YT-102-GENERATOR-COMPATIBILITY.md).

Generators retain their separately qualified, **unedited legacy local-source**
representation, consistent app lock and source revalidation. Retained internal
legacy scaffold functions support its acceptance fixtures. These source
selections are historical labels, not current CLI options:

| Qualified legacy source | Delivery requirement |
|---|---|
| Native service SSR | Established source/lock closure |
| Service with typed settings | Exact public TOMLs; runtime token supplied separately |
| Service with production Vite, optional settings/plugin Tailwind | Explicitly prebuilt and validated flat dist |

The retained legacy matrix is separately qualified by
[YT-101 evidence](../../docs/YT-101-VERIFICATION.md). Standalone Tailwind,
legacy dev bridge, edited/general apps and unknown dependencies/layouts remain
refused; adding a marker or CLI namespace alone cannot qualify arbitrary code.

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
