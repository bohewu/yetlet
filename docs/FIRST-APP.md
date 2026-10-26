# Build your first Yetlet app

Create your own standalone native SSR application using the existing CLI and
local Yetlet checkout. This Linux/POSIX walkthrough connects the current setup
steps; it adds no new scaffold or sample. **Experimental and unpublished:**
use compatible local source, not an assumed registry release.

## Prerequisites

- Start in the Yetlet repository root, using the same POSIX terminal for all
  stages. Have Rust/Cargo 1.99.0 installed, a native C/link toolchain and the
  OpenSSL development headers/libraries needed to build the CLI's signing
  dependency. The [CLI prerequisite table](../tools/yetlet-cli/README.md#first-use-prerequisites)
  owns tool details and custom OpenSSL setup. This first app does not select
  signing or inherit the CLI's OpenSSL dependency.
- Use a trusted physical checkout and an owned local application parent with
  no group/world write access. Choose unused directory/application names. The
  CLI refuses overwrite and unsafe paths; do not disable those checks.
- Prepare network access for explicit Cargo dependency preparation, or an
  already populated cache. The CLI does not install tools, refresh locks or
  fetch missing dependencies for frozen execution. No Node.js, WASM target,
  browser build, database, service token or cloud account is needed here.
- Use a terminal without unrelated `APP_CLI__*`, settings-root or port overrides.
  The plain native scaffold defaults to Production CLI settings, Vite off and
  Rust watch off. Stop another owned server using port 3000 before running it.
  Windows has [separate native setup](../tools/yetlet-cli/README.md#windows-powershell-native-gnumsvc);
  these POSIX commands are not a Windows walkthrough.

Stop at the first error. All paths below are explicit; no ancestor search,
global CLI installation or existing-project overwrite is assumed.

## Minimal integration

### 1. Build the local CLI

From the repository root, keep the checkout and CLI paths for later stages.
The toolchain selection is local to this terminal and inherited by child Cargo
commands, including the standalone app's commands:

```sh
export RUSTUP_TOOLCHAIN=1.99.0
yetlet_repo="$(pwd -P)"
yetlet_cli="$yetlet_repo/tools/yetlet-cli/target/debug/yetlet"
cargo build --locked --manifest-path "$yetlet_repo/tools/yetlet-cli/Cargo.toml"
"$yetlet_cli" --version
```

The CLI has its own workspace/lock. `--locked` allows downloading its locked
dependencies; it is not offline mode. Use the explicit binary path throughout
instead of assuming that `yetlet` is installed on PATH. A version or
`doctor` result identifies a tool/environment, not application readiness.

### 2. Create the plain native app

Choose an unused parent for this walkthrough; the command deliberately omits
`-p` so an existing parent does not get silently reused. If it already exists,
stop and choose another owned location, or deliberately select a known protected
parent and skip its creation.

```sh
yetlet_parent="$HOME/yetlet-first-apps"
mkdir -m 700 "$yetlet_parent"
"$yetlet_cli" new firstapp --parent "$yetlet_parent"
yetlet_app="$yetlet_parent/firstapp"
```

Plain `new` selects the existing native template. It writes `Cargo.toml`,
`src/main.rs`, README, gitignore and public base/profile TOMLs, but creates no
Cargo.lock, installs nothing and starts no process. The app declares its own
standalone `[workspace]`; it is not added to Yetlet's workspace.

The public TOMLs configure CLI behavior. They do not mean the app has selected
the optional runtime Settings loader. This path needs no `--settings`,
`--service`, `--vite` or `--tailwind` option.

### 3. Select compatible local Yetlet dependencies

Open `$yetlet_app/Cargo.toml` in your editor. Under its existing `[dependencies]`,
replace the two generated Yetlet dependency lines with the following shape.
Replace `/absolute/yetlet` with the real physical checkout path stored in
`yetlet_repo`; TOML does not expand shell variables or `~`.

```toml
yetlet-core = { version = "=0.1.0", path = "/absolute/yetlet/crates/yetlet-core" }
yetlet-server-axum = { version = "=0.1.0", path = "/absolute/yetlet/crates/yetlet-server-axum" }
```

Keep the existing package metadata, standalone workspace, Axum/Tokio entries
and exact version requirements. Select both crates from the same compatible
checkout. The generated versions are unpublished; leaving version-only entries
does not make them resolvable from a registry. A moved or modified local checkout
also changes the source used by later Cargo commands.

The generated [selected README source](../tools/yetlet-cli/templates/SELECTED-README.md)
describes the same Cargo-owned overrides. Configuration requires an additional
local `yetlet-config` dependency only when that capability is separately selected;
do not add it for this plain app.

### 4. Prepare the app lock/cache, then build and run

Remain in the repository-root terminal; each command selects the app explicitly:

```sh
cargo generate-lockfile --manifest-path "$yetlet_app/Cargo.toml"
cargo fetch --locked --manifest-path "$yetlet_app/Cargo.toml"
"$yetlet_cli" build --manifest-path "$yetlet_app/Cargo.toml"
"$yetlet_cli" run --manifest-path "$yetlet_app/Cargo.toml"
```

The first two commands are explicit dependency preparation: lock generation
resolves this app's graph, and fetch populates its locked dependency cache.
Keep this app's Cargo.lock; do not copy Yetlet's, the CLI's or xtask's lock.
The CLI's native build/run then use frozen Cargo: they cannot repair a stale
lock or fetch a missing dependency. The app's source is still ordinary Rust;
frozen execution is not a sandbox for build scripts or application code.

## Expected result

The foreground app prints its selected loopback address, normally
`Listening on http://127.0.0.1:3000`. Open the printed address:

| Route | Existing scaffold behavior |
|---|---|
| `/` | An HTML page displaying `Hello <Yetlet> & Rust!` as escaped text |
| `/hello/Ada` | A complete page displaying `Hello, Ada!` through the native Path extractor |

These routes come from the existing [native main template](../tools/yetlet-cli/templates/main.rs).
Pages use Yetlet's escaped rendering and default strict CSP; this first path
loads no JS/WASM. It installs no authentication or business authorization.

Press Ctrl+C to stop the foreground CLI. To make the app yours, edit the text
passed to `View::text` in its existing `home` handler in `src/main.rs`, then
repeat the build/run commands. Ordinary `run` does not watch files by default.
It uses native frozen `cargo run`, which can compile changed Rust once; it is
not a binary-only launch or a state-preserving reload promise.

## Configuration and errors

The plain native template reads `YETLET_PORT`, defaulting to 3000, and always
binds IPv4 loopback. For another unused port, use the same explicit app manifest:

```sh
YETLET_PORT=3001 "$yetlet_cli" run --manifest-path "$yetlet_app/Cargo.toml"
```

CLI profile selection is separate: `APP_ENVIRONMENT` accepts `Development`,
`Staging` or `Production`, default Production. `--release` changes Cargo's build
profile, not the settings environment. Runtime Settings-enabled apps have their
own declared variables and validation; follow their guide when you select them.

| Symptom or point of confusion | Meaning and next step |
|---|---|
| CLI build cannot find OpenSSL headers/libraries | Prepare the native CLI prerequisites or your already-selected OpenSSL prefix using the CLI guide; this is separate from app dependencies |
| `new` refuses the destination | Use an unused app name and an explicitly trusted parent; it does not merge or overwrite projects |
| Cargo cannot find the unpublished Yetlet version | Complete both local `path` declarations with actual compatible checkout paths, retaining their exact version requirements |
| CLI refuses a missing/stale lock or missing offline dependency | After an intentional dependency edit, explicitly regenerate this app's lock and fetch its locked dependencies before frozen execution; do not copy another workspace's lock |
| Unsafe application/tool path or permissions are rejected | Use owned local protected inputs and the documented CLI filesystem setup; do not weaken checks or change unrelated system directories |
| Port 3000 is occupied | Stop the owned conflicting listener or select an unused `YETLET_PORT`; keep the loopback boundary |
| Source edit does not appear while the process keeps running | Stop and build/run again; opt into the separate Watchexec path only after its documented setup |
| `doctor` or `inspect` succeeds but startup fails | Those are bounded environment/plan reports, not dependency, secret, asset or execution qualification; inspect the actual failing stage |

For broader CLI, browser, settings and opt-in HTTP failures, use the
[common-error troubleshooting guide](TROUBLESHOOTING.md).

The [CLI execution table](../tools/yetlet-cli/README.md#execution-decision-table)
owns build/run/watch choices. Keep Auth, Session, CSRF, persistence and deployment
policy explicit when expanding the app; this first native scaffold does not
provide those capabilities automatically.

For the next edit, follow [Write Views and reusable components](VIEW-COMPONENTS.md)
to pass data and compose the existing home handler. Use the
[opt-in quick reference](OPT-IN-QUICK-REFERENCE.md) when choosing later capabilities.

## API reference

- [Current CLI prerequisites and creation](../tools/yetlet-cli/README.md#first-use-prerequisites)
  and [selected scaffold source](../tools/yetlet-cli/src/scaffold.rs).
- [Server guide](../crates/yetlet-server-axum/README.md),
  [HTTP pipeline](HTTP-PIPELINE.md) and [current API reference](API-REFERENCE.md).
- [Configuration](CONFIGURATION.md) and [opt-in composition](EXTENSIONS.md) when
  selecting additional capabilities.
- [Existing sample catalog](SAMPLES.md), [Todo browser setup](../README.md#choose-your-next-step)
  and [Admin reference](../examples/admin-dashboard/README.md) for later paths.

This is a source-backed walkthrough of existing behavior. Its commands are
instructions, not a new execution or platform/production qualification receipt.
