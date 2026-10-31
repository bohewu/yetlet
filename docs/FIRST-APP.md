# Build your first Yetlet app

Create a standalone native SSR app with the published source-installed CLI and
`yetlet` SDK `0.9.0-alpha.5`. This Linux/POSIX walkthrough uses the existing
scaffold; the app needs no local SDK source or Cargo path overrides. APIs remain
Experimental. CLI `0.1.1` and SDK `0.9.0-alpha.5` have independent versions.

## Prerequisites

- Use the same POSIX terminal for all stages; no repository clone is needed.
  Have Rust/Cargo 1.99.0 installed, a native C/link toolchain and the
  OpenSSL development headers/libraries needed to build the CLI's signing
  dependency. The [CLI prerequisite table](../tools/yetlet-cli/README.md#first-use-prerequisites)
  owns tool details and custom OpenSSL setup. This first app does not select
  signing or inherit the CLI's OpenSSL dependency.
- Use an owned local application parent with
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
existing-project overwrite is assumed. The installation below places the CLI
in Cargo's user install directory.

## Minimal integration

### 1. Install the CLI

Install the independently published CLI from crates.io:

```sh
cargo +1.99.0 install yetlet-cli --version 0.1.1 --locked
yetlet --version
```

This downloads the CLI source archive from crates.io and compiles the `yetlet`
executable. Expected version output is exactly `yetlet 0.1.1`. This is a native
source installation, so the prerequisites above apply. Maintainers evaluating a
checkout can use the separate [source-install instructions](../tools/yetlet-cli/README.md#build-locally-and-create-an-application).

Cargo places `yetlet` under its install root's `bin` directory (normally
`~/.cargo/bin`). Ensure it is on PATH, then check the installed command:

```sh
yetlet --version
yetlet --help
yetlet_rust_bin="$(dirname "$(rustup which --toolchain 1.99.0 cargo)")"
env PATH="$yetlet_rust_bin:$PATH" yetlet doctor --json
```

For an isolated prefix, add `--root /absolute/owned/cli-prefix` to the install
command and invoke `/absolute/owned/cli-prefix/bin/yetlet`. The CLI version is
independent of SDK `0.9.0-alpha.5`.
`--locked` allows downloading locked dependencies. A version or doctor result
identifies a tool/environment, not application readiness. For a Rustup setup,
`rustup which` selects the actual already-installed toolchain directory. The
CLI checks executable and ancestor ownership/permissions; group/world-writable
tool installations are refused. With another Rust installation, select its real
protected Cargo/rustc directory instead. This does not install tools or relax
CLI checks. Doctor does not synchronize/install Rustup toolchains.

### 2. Create the plain native app

Choose an unused parent for this walkthrough; the command deliberately omits
`-p` so an existing parent does not get silently reused. If it already exists,
stop and choose another owned location, or deliberately select a known protected
parent and skip its creation.

```sh
yetlet_parent="$HOME/yetlet-first-apps"
mkdir -m 700 "$yetlet_parent"
yetlet new firstapp --parent "$yetlet_parent"
yetlet_app="$yetlet_parent/firstapp"
```

Plain `new` selects the existing native template. It writes `Cargo.toml`,
`src/main.rs`, README, gitignore and public base/profile TOMLs, but creates no
Cargo.lock, installs nothing and starts no process. The app declares its own
standalone `[workspace]`; it is not added to Yetlet's workspace.

The public TOMLs configure CLI behavior. They do not mean the app has selected
the optional runtime Settings loader. This path needs no `--settings`,
`--service`, `--vite` or `--tailwind` option.

### 3. Prepare the app lock/cache, then build and run

The generated manifest already selects `yetlet = "=0.9.0-alpha.5"` from crates.io
with `server-axum`; keep that dependency and its selected features. No manifest
editing, SDK checkout, `[patch]` or path override is required. Work in the new
app directory, outside the Yetlet checkout:

```sh
cd "$yetlet_app"
(
    export PATH="$yetlet_rust_bin:$PATH"
    cargo generate-lockfile
    cargo fetch --locked
    yetlet build
    yetlet run
)
```

The first two commands explicitly prepare this app's dependency graph and cache.
Keep its own Cargo.lock; do not copy Yetlet's, the CLI's or xtask's lock. CLI
build/run use frozen Cargo: they cannot repair a stale lock or download a missing
dependency. Frozen execution is not a sandbox for build scripts or application
code. Node/npm is unnecessary for this plain native app.

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
(
    export PATH="$yetlet_rust_bin:$PATH"
    export YETLET_PORT=3001
    yetlet run --manifest-path "$yetlet_app/Cargo.toml"
)
```

CLI profile selection is separate: `APP_ENVIRONMENT` accepts `Development`,
`Staging` or `Production`, default Production. `--release` changes Cargo's build
profile, not the settings environment. Runtime Settings-enabled apps have their
own declared variables and validation; follow their guide when you select them.

| Symptom or point of confusion | Meaning and next step |
|---|---|
| CLI build cannot find OpenSSL headers/libraries | Prepare the native CLI prerequisites or your already-selected OpenSSL prefix using the CLI guide; this is separate from app dependencies |
| `new` refuses the destination | Use an unused app name and an explicitly trusted parent; it does not merge or overwrite projects |
| Cargo cannot resolve the SDK or download a dependency | Confirm the exact published version and registry/network access; explicitly prepare the app lock/cache before frozen CLI commands |
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
- [Server guide](../crates/yetlet/guides/server-axum.md),
  [HTTP pipeline](HTTP-PIPELINE.md) and [current API reference](API-REFERENCE.md).
- [Configuration](CONFIGURATION.md) and [opt-in composition](EXTENSIONS.md) when
  selecting additional capabilities.
- [Existing sample catalog](SAMPLES.md), [Todo browser setup](../README.md#choose-your-next-step)
  and [Admin reference](../examples/admin-dashboard/README.md) for later paths.

The isolated GNU source-package install and registry-only first-app path are
recorded in [YT-214](YT-214-CLI-INSTALLATION.md). That evidence does not extend
Windows, browser/GPU or production qualification.
