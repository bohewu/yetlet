# Yetlet CLI — Experimental Developer Tool

The CLI creates applications and executes selected tasks. Persistent behavior uses
Yetlet's existing public base/profile settings and scoped environment overrides.
This standalone Cargo binary package installs the `yetlet` command. Generated
apps use the published `yetlet` / `yetlet-browser` SDK `0.9.0-alpha.5`; normal
build/run needs no SDK checkout or Cargo path override. CLI version `0.1.1` and
SDK versions are independent. This is an Experimental developer tool, with
platform evidence limited to the documented qualification scopes.
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
| `yetlet inspect [--manifest-path PATH] [--json]` | Read-only public-settings plan for build/run/watch; separate project-execution-plan schema 1; no readiness checks |
| `yetlet key status [KEY] [--provider NAME] [--manifest-path PATH]` | Read-only selected key/version, certificate pins and dates; no password or generation |
| `yetlet key rotate [KEY] [--provider NAME] [--manifest-path PATH] [--yes] [--from PATH]` | Confirmed local rotation and atomic selected-profile settings update; retain old versions |
| `yetlet run [--manifest-path PATH]` | Native frozen Cargo/watch/Vite, or settings-selected browser-only static preview |
| `yetlet watch [--manifest-path PATH]` | Native Rust watch, or explicitly selected browser WASM rebuild/full reload; Vite still uses settings |
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

## First-use prerequisites

For one guided plain-native path, follow [Build your first Yetlet app](../../docs/FIRST-APP.md).
This reference retains the broader target, settings and execution choices.

Prepare tools and dependency caches explicitly before running the examples.
Building the CLI from source requires OpenSSL headers/libraries through its
native signing dependency, even if you only plan to create a browser app.
Plain generated native/browser apps do not acquire that CLI dependency.
Building the CLI and building an app are separate steps.

| Selection | Tools and inputs to prepare | Initial workflow |
|---|---|---|
| Build this CLI from the checkout | Rust/Cargo 1.99+, a native C/link toolchain, OpenSSL development files (reference: 3.5 LTS) and cached locked dependencies | [Build the CLI](#build-locally-and-create-an-application), then inspect help/doctor |
| Native `new NAME` | Native Rust toolchain, the published SDK and a separately prepared app Cargo.lock | `yetlet build`, then `yetlet run`; no Node or frontend tool is required |
| Native `new NAME --vite`, optionally `--tailwind` | Native prerequisites plus Node `^20.19.0` or `>=22.12.0`, npm and the generated frontend dependencies | [Vite preparation](templates/vite/SELECTED-README.md), initial `yetlet build` with auto_build enabled, then `yetlet run` |
| `new NAME --template browser`, optionally `--tailwind` | Rust/Cargo 1.99+, `wasm32-unknown-unknown`, wasm-bindgen-cli 0.2.129, the same Node/npm prerequisites and the generated dependencies | [Browser preparation](templates/browser/README.md), `yetlet build`, then `yetlet run` to preview the existing dist |
| Native `new NAME --tailwind` without Vite | Native prerequisites plus the separately provisioned standalone Tailwind 4.3.3 CLI | [Build `dist/tailwind.css` explicitly](templates/tailwind/SELECTED-README.md) before running Rust |

The Vite Tailwind selection uses the official npm plugin, without a standalone
CSS compiler. Native Rust watch additionally needs separately prepared
[Watchexec](#rust-watch); development HTTPS needs separately prepared
[mkcert and certificates](templates/config/SELECTED-README.md#development-https).
Settings-enabled native apps, including `--vite`, need their own service token
before startup; use the existing [settings/Development-secret mechanism](templates/config/SELECTED-README.md),
keeping credentials out of public TOMLs and frontend files.

Use this order: prepare tools/cache, build the CLI, create the selected app,
generate its own Cargo lock and fetch its locked dependencies, prepare any
selected frontend dependencies/assets, then build/run. `new` does not install
tools, generate Cargo.lock or produce dist; selected frontend templates include
their pinned package lock. `run` does not create the initial dist;
with frontend auto_build disabled, prepare it separately before startup.
Offline commands require an already populated cache. CLI execution never
installs dependencies or repairs locks automatically.

`yetlet doctor --json` checks the local Rust/Cargo environment and compiled
filesystem capability only. A passing result does not check OpenSSL development
files for a CLI rebuild, app dependencies/settings/secrets, Node/npm, WASM tools,
frontend dist or whether `run` will work. Use the selected setup instructions
and actual build/run for those checks. On Windows, also prepare protected
physical tool paths and an attached processed-input console for native run;
see [native tool preparation](../../docs/WINDOWS-CLI-125-VERIFICATION.md#reuse-the-prepared-runner-tools).

## Build locally and create an application

The selected user installation interface is:

```sh
cargo +1.99.0 install yetlet-cli --version 0.1.1 --locked
yetlet new myapp
```

Cargo downloads and compiles the selected published CLI package; no manual
repository clone or npm is needed. `yetlet --version` reports exactly
`yetlet 0.1.1`. [Release status and scope](https://github.com/bohewu/yetlet/blob/master/docs/YT-216-CLI-011-PATCH-RELEASE.md)
are tracked separately from this installation reference. Prebuilt executables
and `cargo binstall` support are later distribution options.

For maintainer source evaluation, install from a trusted physical
checkout or an unpacked verified CLI archive:

```sh
(
    # Optional, only for a separately prepared static OpenSSL installation:
    # export OPENSSL_DIR=/path/to/prepared/openssl
    # export OPENSSL_STATIC=1
    cargo install --path tools/yetlet-cli --locked
)
```

Cargo installs the `yetlet` executable under its install root's `bin` directory
(normally `~/.cargo/bin`). Add that directory to PATH, or invoke the executable
by its absolute path. This command compiles native Rust; it is not an npm
installation. Registry installation requires a published version of
`yetlet-cli`. No prebuilt CLI downloads are provided by this source release.
For a private install prefix, add `--root /absolute/owned/cli-prefix` and use its
`bin/yetlet`. Compilation requires the native/OpenSSL prerequisites above.

After installation, work outside the Yetlet checkout:

```sh
(
    # Rustup: explicitly select the already-installed physical toolchain.
    yetlet_rust_bin="$(dirname "$(rustup which --toolchain 1.99.0 cargo)")"
    export PATH="$yetlet_rust_bin:$PATH"
    yetlet --version
    yetlet --help
    yetlet doctor --json
    mkdir -m 700 "$HOME/yetlet-apps"
    yetlet new sampleapp --parent "$HOME/yetlet-apps"
    cd "$HOME/yetlet-apps/sampleapp"
    cargo generate-lockfile
    cargo fetch --locked
    yetlet build
    yetlet run
)
```

Use a new owned parent; if it already exists, deliberately select an existing
protected parent or choose another location. Linux supports owner-protected
directories or trusted sticky creation parents; Windows requires protected local
fixed NTFS without reparse traversal. Creation never overwrites or merges an
existing destination. This walkthrough is POSIX/GNU; existing Windows setup and
qualification remain separate.

The CLI checks executable and ancestor ownership/permissions; group/world-
writable tool installations are refused. With Rustup, explicitly select the
installed physical toolchain as above; with another installation, select its
protected real Cargo/rustc directory. Doctor never synchronizes/installs a Rustup
toolchain. Do not disable the checks. The subshell keeps this PATH selection
local to the walkthrough.

The CLI and each app have separate workspaces/locks. New apps declare exact
registry SDK versions and create no Cargo.lock. The two explicit Cargo commands
prepare the app's lock/cache; CLI build/run then use frozen Cargo. Offline
preparation requires a populated cache. `--release` changes the Cargo profile,
not APP_ENVIRONMENT. Frozen execution is not a sandbox for package scripts,
build scripts or app code. For the complete walkthrough and expected HTTP
result, see [Build your first Yetlet app](../../docs/FIRST-APP.md).

### Maintainer resource updates

The CLI archive includes templates, pre-Wasm loading assets, licenses and a
fixed public SDK source snapshot for the existing qualified-source generators.
Compilation does not read files outside the CLI package or fetch source at
build time. Normal Cargo dependency fetching remains explicit.
After deliberately selecting the matching published SDK and updating the exact
CLI dependency, refresh these resources from a trusted repository checkout:

```sh
python3 tools/yetlet-cli/scripts/sync_resources.py
```

Review the snapshot diff and run the CLI `resources` test. The command copies
only Git-tracked public SDK files plus the workspace manifest/lock baseline and two startup
assets; it does not package ignored files, secrets or a user's application.
The snapshot is compatibility data for inspection, not a third public SDK.
Registry-only Docker/Compose/Kubernetes generation is still unselected; those
commands retain their explicit compatible local-source requirements below.

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

The shared `yetlet::config` loader applies base, exactly one selected overlay,
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

### Settings selection decision table

For `run`, `watch` and `build`, resolve these choices in order using the
existing [CLI settings projection](src/settings.rs) and
[shared Loader](../../crates/yetlet/src/config/mod.rs):

| Decision | Existing rule | Practical effect |
|---|---|---|
| Application | Explicit `--manifest-path PATH`, otherwise exactly cwd/Cargo.toml; select a protected standalone Cargo.toml with an existing Cargo.lock | Its parent is the app root; no ancestor/workspace search |
| Public settings directory | Absolute `YETLET_CONFIG_ROOT` when supplied, otherwise the selected app root | Moves public TOML lookup only; source, lock and `cli.vite.root` stay relative to the selected app |
| Settings profile | `APP_ENVIRONMENT` accepts exactly `Development`, `Staging` or `Production`; absent means Production | Selects exactly the matching profile TOML alongside base.toml; `--release` does not select a settings profile |
| Public values | base.toml, then selected profile, then scoped OS overrides such as `APP_CLI__VITE__AUTO_START` | Higher layers replace declared keys with the same value kind; ENV is not a way to create undeclared settings |
| Execution target | Merged `cli.target`, default native when absent; browser must have Vite enabled, Rust watch disabled and native TLS disabled | `--template browser` selects creation; later execution reads settings, including declared `APP_CLI__TARGET` overrides |
| Invocation choice | After settings validation, native `watch` forces Rust watch; browser `watch` requires both auto_start and auto_build; native `build --release` selects Cargo's release profile | Neither writes settings; invalid settings cannot be repaired or skipped by these command choices |

The CLI also admits public `APP_SERVER__TLS__*` overrides for Development TLS;
other backend overrides and Development secret files do not enter tool-policy
loading. Invalid profiles, unknown related keys, wrong types and unsafe selected
paths fail before child startup.

With no recognized base/profile files, legacy defaults are available only when
there is no explicit settings-root or public-value override: native target,
Rust watch off, Vite off and one generated replica. The profile selector must
still be valid. Once files exist, both base.toml and the selected profile are
required. If their validated merged data has no cli section, those defaults
still apply; a present but incomplete cli section is not filled arbitrarily.
An omitted cli.target or cli.rust retains its specific backward-compatible
default. Settings are read for each command invocation; this is not live
tool-policy reload inside an already running watch process.

### Execution decision table

After successful [application preflight and settings selection](src/app.rs):

| Target / command | Settings decision | Result |
|---|---|---|
| Native `run` | `cli.rust.watch`; Vite only when enabled and auto_start | Frozen Cargo run, optionally supervised by Watchexec and/or alongside Vite |
| Native `watch` | Force Rust watch; retain Vite enabled/auto_start/root settings | Watch/rebuild/restart Rust; an owned Vite process stays running across Rust restarts |
| Native `build` | Vite only when enabled and auto_build; Rust watch is not used | Selected frontend build must succeed before frozen Cargo build; when the gate is false, build Cargo only |
| Browser `run` | auto_start | Preview existing static dist, without a native app server; false skips startup with exit 0 and no child |
| Browser `build` | auto_build | Run the frontend-owned WASM/Vite build, without a native app build; false skips it with exit 0 and no child |
| Browser `watch` | Both auto_start and auto_build | Vite dev with serialized frozen WASM builds, immutable publication and full-page reload; either false skips without children; no native watcher or Rust state restoration |

For example, in a generated native Vite app, a Development overlay setting
auto_start=false makes `run` start Rust only. A declared
`APP_CLI__VITE__AUTO_START=true` override enables Vite for that invocation.
Neither changes auto_build: `build` still uses that independent setting.
Use `APP_ENVIRONMENT=Development` explicitly to select that overlay. CLI public
selection does not establish that app runtime secrets or frontend dist are ready;
follow the [first-use preparation order](#first-use-prerequisites).

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

Generators accept the following exact current `new --service` source shapes after
explicit compatible local Cargo path overrides and separately prepared locks:

| Current creation choice | Source shape and current acceptance |
|---|---|
| `new NAME --service` | Public CLI settings; no runtime settings dependency or token; accepted by all three generators |
| `new NAME --service --settings` | Public CLI/runtime TOMLs and the exact Config/Secrets source closure; accepted by all three generators |
| `new NAME --service --vite` | Settings implied; accepted with the validated Production Vite asset graph |
| `new NAME --service --vite --tailwind` | Accepted with the pinned official plugin and validated Production Vite asset graph |

The separate [2026-10-22 source-closure repair](../../docs/YT-102-GENERATOR-SOURCE-CLOSURE.md)
adds only six exact reviewed Config/Secrets source files, including Config's
explicitly declared example. Settings profiles declare 20 framework-context
files; plain service retains 14. These public source files are distinct from
application credentials. The optional secret provider remains disabled and absent
from the selected normal dependency graph. Source/layout comparisons and protected
reads still reject modified, extra or symlinked selected source inputs.

Focused evidence covers all four profiles and 12 outputs, exact inventories,
negative-source cases and an actual sanitized Vite-source offline Cargo build.
Its Vite assets are a structural fixture. The earlier failed five-file attempt
and original config-layout refusal remain recorded; neither metadata success nor
historical qualification is relabeled as a fresh OCI/platform/deployment PASS.

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
selections are historical labels, not current CLI options. The rows below
retain source-bound historical qualification. The matching legacy delivery
fixture passes with the repaired exact source closure; historical platform and
runtime/deployment evidence does not transfer automatically to this checkout.

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
covers six selected real Rust/Vite lifecycle cases; MSVC pair evidence remains
compile-only. Separately, [native MSVC single-Cargo evidence](../../docs/YT-102-COMPLETION-LATENCY.md#windows-native-followup-2026-10-20)
covers build/run, native console cancellation and descendant cleanup, with a
bounded warm-build baseline in the same record.

## Read-only project inspection (YT-111)

Use `yetlet inspect`, or select an explicit project with
`yetlet inspect --manifest-path /work/app/Cargo.toml --json`. The default is
exactly cwd/Cargo.toml, with the same protected standalone manifest and existing
Cargo.lock requirements as build/run/watch. No ancestor search occurs.

Inspection uses the existing public base/profile Loader, YETLET_CONFIG_ROOT,
APP_ENVIRONMENT (default Production) and scoped APP_CLI overrides. It explains
native/browser target, configured Rust watch, Vite enabled/relative root and
auto_build/auto_start, then the intended Rust/watch/Vite stages for each command
and fixed reasons for selection or skipping. Native watch forces Rust watch
after validation; build never watches. Browser build selects frontend build,
run selects static preview, and browser watch requires both auto_start and
auto_build for frontend-owned WASM rebuild/full reload.

Human roots are escaped for terminal safety. JSON is one ASCII-escaped object
with `schema_version: 1`, `scope: "project-execution-plan"`, `environment`,
`target`, `rust_watch`, `vite`, `commands` and `limitations`. Each command has
`supported` and `rust`, `rust_watch`, `vite` stages with `selected`/`reason`.
This schema is separate from the unchanged environment-only doctor schema 1.
See the [selected contract](../../docs/CLI-CONTRACT.md#read-only-project-inspection--yt-111).

Exit 0 means settings-derived intentions were explained; unsupported/skipped
stages do not make inspection fail. No tools, frontend directories/assets,
TLS files, secrets, runtime startup or readiness are verified. Inspection starts
no children, writes no output/settings, makes no network requests, reads no
Development secret files and calls no SecretProvider. It never dumps settings,
ENV, tokens or resolved absolute roots. Invalid selection/settings uses exit 1,
empty stdout and safe stderr, including existing source/profile/values hints;
`--json` does not alter error formatting. Usage failures use exit 2.

The [inspection record](../../docs/YT-111-IMPLEMENTATION.md) retains its
source-bound acceptance evidence. The separate
[browser-loop record](../../docs/YT-112-IMPLEMENTATION.md) records completed Linux
edit/error/recovery and lifecycle acceptance. New Windows browser-watch execution
remains Not verified; the separate earlier Windows CLI evidence does not cover it.

## Diagnostics and platform evidence

CLI-owned statuses are 0 success/help, 1 preflight/report, 2 usage and 125
spawn/supervision/cleanup failure. Native child exits and cancellation retain
identity after verified cleanup. Safe scoped CLI errors use stable codes and
stderr; child-failure summaries identify their stage without guessing lock/cache
causes. Child stdout/stderr remains native and is not guaranteed secret-redacted.

Public CLI settings failures use three fixed human troubleshooting hints:

| Hint | Check |
|---|---|
| `source` | Public settings directory and required base/profile files, including safe ownership/readability |
| `profile` | Exactly Development, Staging or Production; absent selection defaults to Production |
| `values` | Public TOML syntax, base-declared keys/types, ranges and target/TLS requirements |

These are coarse hints, not new machine codes or exact filesystem diagnoses.
They never echo supplied values, unknown keys, configured paths or underlying
parser/IO/Serde errors. The existing Loader, validation order, exit 1/stderr and
caller wrappers remain intact, including generator rejection before output.
Unexpected Loader errors retain the generic hint; existing browser-capability
refusals and later TLS-file checks retain their messages. See the
[research and bounded implementation evidence](../../docs/YT-102-SETTINGS-ERROR-RESEARCH.md#selected-cli-implementation-2026-10-20).

Doctor reports schema 1 environment checks only, never application, asset,
frontend, console, generator or release readiness. Fixed local Rust/Cargo probes
have 3-second/64-KiB combined-output bounds plus 2-second cleanup. Unsafe or
unavailable executables fail; group-writable toolchain paths are refused on
Linux. No installation/update, shell, app settings or Node probes occur.

Linux has current native process/PTY/cleanup tests. Current Windows GNU
[doctor and CLI evidence](../../docs/YT-102-DOCTOR-VERIFICATION.md) and
[Rust/Vite pair evidence](../../docs/YT-102-WINDOWS-VITE-VERIFICATION.md) supplement
the separate MSVC compile-only pair checks and
[native single-Cargo followup](../../docs/YT-102-COMPLETION-LATENCY.md#windows-native-followup-2026-10-20).
Windows run still requires an attached console
with processed Ctrl-C input and protected app/tool paths; npm uses vetted native
node.exe plus its same-install npm CLI script, without a .cmd shell fallback.
Escaped daemons and unsupported OS/filesystem/console contexts remain unsupported.
Public release, production qualification, global installation and deployment
remain outside this preview.

## Browser Rust development loop — YT-112

Use `yetlet watch` for the browser starter's selected edit/rebuild/reload loop.
Both `cli.vite.auto_start` and `auto_build` must be true; either false skips the
whole loop. Keep the native `cli.rust.watch` setting false. No Watchexec/native
backend is needed here: Vite watches app `src/`, Cargo manifest/lock and build.rs.
The initial and subsequent optimized WASM builds are frozen/offline and require
separately prepared wasm-bindgen-cli 0.2.129. Edits are serialized/coalesced;
compiler failures wait for source correction without replacing the last good
JS/WASM pair. Before any successful generation HTTP returns 503.

Successful Rust generation triggers full reload and resets tab-local state.
Vite retains CSS HMR and TS/JS update ownership; no Rust state restoration or
live remount is promised. Strict loopback/port, source confinement, dotenv and
backend-environment filtering remain enforced. A fresh document nonce permits
Vite CSS styles without weakening production CSP. Ctrl-C stops the owned tree
and removes private generations. Browser `run` stays static preview; `build`
packages production `dist/`, with no watch artifacts. External path dependencies
are not automatically watched. The [selected amendment](../../docs/YT-112-IMPLEMENTATION.md)
records completed Linux browser/lifecycle acceptance and retains the original
failed attempts. New Windows browser-watch execution remains Not verified;
historical YT-104 qualification describes the earlier static starter scope.
