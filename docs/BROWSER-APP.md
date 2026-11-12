# Build a browser-only Yetlet app

Create a standalone interactive app with published CLI `0.4.0` and SDK
`0.9.0-alpha.8`, without cloning Yetlet. Rust owns local state and interaction;
JavaScript starts the WASM module. The finished app needs an HTTP(S) static host.
These instructions use the pinned reader edition and existing APIs; commands
below are a walkthrough, not evidence of a new execution.

| Choose this path when you need | Guide |
|---|---|
| A local interactive UI that starts after JS/WASM loads | This browser-only walkthrough |
| Request-time HTML or HTTP handlers, with optional browser islands | [Native first app](FIRST-APP.md) |
| Readable build-time HTML, optionally with selected WASM islands | [Static HTML export](FIRST-APP.md#next-export-a-static-site) |

## Prerequisites

- Use a POSIX terminal with Rust/Cargo `1.99.0`, its native linker/C tools,
  Node.js `22.12` or newer and npm. The source-installed CLI also needs the
  [native OpenSSL prerequisites](CLI-GUIDE.md#first-use-prerequisites);
  this is separate from the browser application's dependencies. The CLI guide
  owns [Windows setup](CLI-GUIDE.md#windows-powershell-native-gnumsvc);
  the commands here are not a PowerShell walkthrough.
- Prepare registry/network access, or the corresponding populated Cargo/npm
  caches. Install tools and prepare dependencies explicitly before frozen builds.
- Choose an unused app name and an owned, protected physical parent. The CLI
  rejects existing destinations and unsafe paths. Use a terminal without
  unrelated `APP_CLI__*`, profile or Cargo target overrides, and reserve loopback
  port `5173` for this app.
- Keep frontend inputs public. Browser configuration, JS and WASM are readable
  by visitors; server settings and secrets belong in a separately selected backend.

Install the two native tools once and add the WASM target:

```sh
(
    set -eu
    cargo +1.99.0 install yetlet-cli --version 0.4.0 --locked
    rustup target add --toolchain 1.99.0 wasm32-unknown-unknown
    cargo +1.99.0 install wasm-bindgen-cli --version 0.2.129 --locked \
        --root "$HOME/.local/share/yetlet-tools"
    export PATH="$(dirname "$(rustup which --toolchain 1.99.0 cargo)"):$HOME/.cargo/bin:$HOME/.local/share/yetlet-tools/bin:$PATH"
    yetlet --version
    wasm-bindgen --version
)
```

Expected identities are `yetlet 0.4.0` and `wasm-bindgen 0.2.129`. The latter
matches the generated app's exact bindgen dependency. With a custom Cargo install
root or tool installation, select its actual protected executable directory on
PATH; do not weaken the CLI's filesystem checks.

## Minimal integration

### 1. Create, prepare, build and preview

Run this block from outside the Yetlet checkout. The parent example deliberately
uses `mkdir` without `-p`: if it exists, stop and choose a new owned location, or
deliberately use an existing protected parent and omit only its creation.

```sh
(
    set -eu
    export PATH="$(dirname "$(rustup which --toolchain 1.99.0 cargo)"):$HOME/.cargo/bin:$HOME/.local/share/yetlet-tools/bin:$PATH"
    yetlet_parent="$HOME/yetlet-browser-apps"
    mkdir -m 700 "$yetlet_parent"
    yetlet new firstbrowser --template browser --parent "$yetlet_parent" --sdk-version 0.9.0-alpha.8
    cd "$yetlet_parent/firstbrowser"
    cargo fetch --locked
    npm ci --prefix frontend --ignore-scripts --no-audit --no-fund
    yetlet build
    npm run typecheck --prefix frontend
    yetlet run
)
```

`new` resolves the explicitly selected reader SDK, writes exact pins for both SDK
packages, Cargo.lock, `.yetlet/sdk-resolution.json` and the public CLI settings.
It installs no tools and starts no application server. Initial metadata/lock
resolution requires registry access, or `--offline` with populated resolver and
Cargo metadata caches. It selects `cli.target = "browser"` and Vite.
Tailwind is optional through the existing `--tailwind`
creation flag; it is not needed for this path.

The app owns its generated Cargo.lock, resolution receipt and frontend package
lock. Preserve them; do not regenerate or copy Yetlet's workspace lock.
`build` uses frozen Cargo/WASM inputs and creates the
production frontend; it does not prepare missing dependencies. `run` previews
that existing output at `http://127.0.0.1:5173/`, without building it or starting
a native Yetlet HTTP backend. Ctrl+C stops the owned preview process tree.

### 2. Find the code to change

| Generated file | Responsibility |
|---|---|
| `src/model.rs` | `Tasks`, typed `Command` values and `Tasks::apply`; change the three titles in `Tasks::default` for a first edit |
| `src/lib.rs` | Rust Views, local graph, event bindings and keyed row updates |
| `frontend/index.html` and `app.css` | Initial HTML/loading surface, owned controls and styling |
| `frontend/app.ts` | Module startup, readiness/error handling and page disposal |
| `frontend/build-wasm.mjs` and `vite.config.mjs` | Existing frozen WASM/bindgen and frontend build stages |
| `base.toml` and profile overlays | CLI target and automatic stage settings; public tool configuration, not app-runtime secrets |

After editing the sample titles, stop the preview, enter the app directory with
the same tool PATH, and repeat `yetlet build` and `yetlet run`.
For the already implemented edit/rebuild loop, run `yetlet watch` instead of
`run`. It requires both `cli.vite.auto_build` and `auto_start` to be true. A Rust
edit rebuilds WASM and fully reloads the page, resetting tab-local state; CSS HMR
is Vite-owned. See the [watch contract](CLI-GUIDE.md#browser-rust-development-loop--yt-112).

### 3. Understand the existing local interaction path

The [model](../tools/yetlet-cli/templates/browser/model.rs) accepts typed commands.
The [Rust entry](../tools/yetlet-cli/templates/browser/lib.rs) owns a `LocalGraph`
and `DomEventListener` handles: an event dispatches a command, the reducer updates
state, and the render callback reconciles visible rows. These sample actions
make no application API requests. Loading the initial page and JS/WASM still
uses the static host.

Keep the listener owner alive while its handlers are installed. Dispose it when
the owned UI retires. Ordinary `dom::mount` replaces a subtree, so listeners on
removed children need explicit rebinding/disposal; use [keyed ownership](../crates/yetlet-browser/guides/dom-keyed.md)
for identity-sensitive rows. Keyed identity does not automatically manage live
input values, focus or selection: the application writes those properties.
The [search caller](../examples/docs-site/src/browser.rs) shows a smaller
caller-owned Rust closure when a local graph/keyed collection is unnecessary.

## Expected result

The page shows a loading status before WASM is ready, then three sample tasks.
Add, rename, complete and filter tasks locally. The sample holds at most 200
tasks in tab memory; reload restores its sample data. It provides no persistence,
login or backend API. Its initial HTML remains useful for loading/no-JS recovery;
the interactive app requires both JavaScript and WebAssembly.

After `yetlet build`, the ship directory is **only `frontend/dist/`**. Upload its
complete contents to the static host you select, preserving relative paths,
JS/CSS/WASM MIME types and the HTML CSP. Relative assets support a directory
prefix ending in `/`. Keep source, settings, dotenv, locks, Cargo `target/`, npm
dependencies and verification records outside uploaded content. The host owns TLS
and response headers; no Node or native Rust process is required at runtime.
Vite preview is a local check, not production hosting. `file://`, a history
router and automatic deep-route fallback are not part of this starter.

## Configuration and errors

The scaffold selects `yetlet/reactive` and `yetlet-browser` features `dom` and
`dom-keyed` for WASM. The SDK's server integrations and the optional action
transport are not required for this sample. Cargo features make APIs available;
the Rust/JS entry owns startup. Public `base.toml` configures the native CLI;
only selected public `VITE_*` values belong in frontend dotenv files.

| Symptom | Existing correction |
|---|---|
| Missing tool, WASM target or bindgen mismatch | Prepare the documented tool/target; select bindgen `0.2.129` on the scoped PATH. Keep the app's exact dependency pin. |
| Frozen build cannot find a lock or dependency | Intentionally prepare this app's Cargo.lock/cache and run npm's locked install before building. Stop at the first failed stage. |
| Build/run reports a successful skip | Inspect `cli.target`, Vite selection and `auto_build`/`auto_start`; exit 0 for a disabled stage is not an executed build or usable preview. |
| An edit is absent in `run` | Stop, rebuild and run again, or explicitly select `watch`. Rust reload does not preserve tab state. |
| Startup stays on loading or reports an error | Inspect the first failed JS/WASM request, MIME type, CSP and bindgen identity. Preserve the startup shell; use its explicit reload recovery. |
| A control loses its listener or focus after a render | Check replaced versus keyed ownership and application-owned input/focus handling. Follow the DOM/keyed guides rather than relying on generic state preservation. |
| Port 5173 is occupied | Stop the owned conflicting process; the existing preview uses a strict loopback port. |

Use [read-only inspection](CLI-GUIDE.md#read-only-project-inspection-yt-111)
to understand settings-selected plans. It does not install dependencies or prove
app readiness. The [startup guide](WASM-STARTUP.md) owns shell customization and
lifetime details; [troubleshooting](TROUBLESHOOTING.md#browser-builds-and-interaction)
links the broader failure paths. Keep accepted failures and evidence separate
from rebuildable output using the [artifact guide](DEV-ARTIFACTS.md).

## API reference

- [CLI browser template and prerequisites](../tools/yetlet-cli/templates/browser/README.md)
  and [creation source](../tools/yetlet-cli/src/browser.rs).
- [DOM integration](../crates/yetlet-browser/guides/dom.md) and
  [keyed collections/listeners](../crates/yetlet-browser/guides/dom-keyed.md).
- [Reactive API source](../crates/yetlet/src/reactive/mod.rs) and
  [API documentation entry](API-REFERENCE.md).

This guide connects existing capabilities. It adds no runtime/API, platform
qualification or documentation-site deployment, and selects no desktop host.
