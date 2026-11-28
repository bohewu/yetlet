# Build a browser-only Yetlet app

Create a standalone interactive app with published CLI `0.4.2` and SDK
`1.0.0-rc.1`, without cloning Yetlet. Rust owns local state and interaction;
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
    cargo +1.99.0 install yetlet-cli --version 0.4.2 --locked
    rustup target add --toolchain 1.99.0 wasm32-unknown-unknown
    cargo +1.99.0 install wasm-bindgen-cli --version 0.2.129 --locked \
        --root "$HOME/.local/share/yetlet-tools"
    export PATH="$(dirname "$(rustup which --toolchain 1.99.0 cargo)"):$HOME/.cargo/bin:$HOME/.local/share/yetlet-tools/bin:$PATH"
    yetlet --version
    wasm-bindgen --version
)
```

Expected identities are `yetlet 0.4.2` and `wasm-bindgen 0.2.129`. The latter
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
    yetlet new firstbrowser --template browser --parent "$yetlet_parent" --sdk-version 1.0.0-rc.1
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

## Windows x64 MSVC PowerShell first use

This companion uses published CLI `0.4.2` and the earlier SDK
`0.9.0-alpha.9` Browser template. The Windows evidence below applies to its
recorded release pair.
Prepare the native Rust 1.99 x64 MSVC toolchain, its
`wasm32-unknown-unknown` target, `wasm-bindgen-cli 0.2.129`, Node.js
22.12+ and npm before building. An installation of the CLI from source
additionally requires Visual Studio 2022 x64 C++ tools and OpenSSL development
libraries; follow the [Windows CLI installation guide](CLI-GUIDE.md#windows-x64-msvc-first-app-and-native-watch).
An existing trusted, correctly versioned CLI installation can be reused.

In PowerShell, select **physical, trusted executable paths**. In particular,
fnm's `aliases/default` may be a junction: Node and its companion
`node_modules/npm/bin/npm-cli.js` must be read through the actual physical
Node version directory. The Windows CLI intentionally rejects untrusted/reparse
executable paths. Do not disable that validation or change global PATH/ACLs
to make it pass.

```powershell
$ErrorActionPreference = 'Stop'
$cargo = (& rustup which --toolchain 1.99.0 cargo)
if ($LASTEXITCODE -ne 0 -or !$cargo -or !(Test-Path -LiteralPath $cargo)) {
    throw 'Prepare a physical Rust 1.99 x64 MSVC toolchain'
}
$rustBin = Split-Path -Parent $cargo
& (Join-Path $rustBin 'rustc.exe') -vV
& $cargo --version
$targets = @(& rustup target list --installed --toolchain 1.99.0)
if ($targets -notcontains 'wasm32-unknown-unknown') {
    throw 'Install the wasm32-unknown-unknown target for this Rust 1.99 toolchain first'
}
$nodeDir = Split-Path -Parent (Get-Command node.exe -ErrorAction Stop).Source
$nodeEntry = Get-Item -LiteralPath $nodeDir -Force
if ($nodeEntry.Attributes -band [IO.FileAttributes]::ReparsePoint) {
    if ($nodeEntry.LinkType -ne 'Junction' -or !$nodeEntry.Target) {
        throw 'Select the owner-verified physical Node version directory'
    }
    $nodeDir = [string]$nodeEntry.Target
}
if ((Get-Item -LiteralPath $nodeDir -Force).Attributes -band [IO.FileAttributes]::ReparsePoint) {
    throw 'Node version directory is not a physical path'
}
if (!(Test-Path -LiteralPath (Join-Path $nodeDir 'node_modules/npm/bin/npm-cli.js'))) {
    throw 'Node/npm installation is incomplete'
}
$env:PATH = "$nodeDir;$rustBin;$env:PATH"
& node.exe --version
& npm.cmd --version
& wasm-bindgen.exe --version

# Select a previously installed, trusted CLI 0.4.2.
$yetlet = (Get-Command yetlet.exe -ErrorAction Stop).Source
& $yetlet --version
& $yetlet doctor --json
if ($LASTEXITCODE -ne 0) { throw 'CLI Doctor environment check failed' }
```

Confirm `rustc -vV` reports `host: x86_64-pc-windows-msvc`,
`yetlet 0.4.2`, and `wasm-bindgen 0.2.129`; stop if any version
differs. Run this creation block **once**, with an unused owner-protected
local NTFS parent outside a Yetlet checkout. Do not use it to overwrite a
previously created project or its locks.

```powershell
$parent = Join-Path $env:LOCALAPPDATA 'YetletBrowserApps'
if (Test-Path -LiteralPath $parent) {
    throw 'Choose another unused, owner-protected Browser app parent'
}
New-Item -ItemType Directory -Path $parent -ErrorAction Stop | Out-Null
& $yetlet new firstbrowser --template browser --parent $parent --sdk-version 0.9.0-alpha.9
if ($LASTEXITCODE -ne 0) { throw 'Browser creation failed; preserve diagnostic output' }
$app = Join-Path $parent 'firstbrowser'
$manifest = Join-Path $app 'Cargo.toml'
foreach ($path in @('Cargo.lock', '.yetlet/sdk-resolution.json', 'frontend/package-lock.json')) {
    if (!(Test-Path -LiteralPath (Join-Path $app $path))) {
        throw "Created app is missing $path"
    }
}
& $yetlet inspect --manifest-path $manifest --json
if ($LASTEXITCODE -ne 0) { throw 'Project plan inspection failed' }
Push-Location $app
try {
    & $cargo fetch --locked
    if ($LASTEXITCODE -ne 0) { throw 'Locked Cargo fetch failed' }
    & npm.cmd ci --prefix frontend --ignore-scripts --no-audit --no-fund
    if ($LASTEXITCODE -ne 0) { throw 'Locked frontend installation failed' }
    & $yetlet build --manifest-path $manifest
    if ($LASTEXITCODE -ne 0) { throw 'Frozen WASM/Vite build failed' }
    & npm.cmd run typecheck --prefix frontend
    if ($LASTEXITCODE -ne 0) { throw 'TypeScript typecheck failed' }
    & $yetlet run --manifest-path $manifest
} finally {
    Pop-Location
}
```

Open `http://127.0.0.1:5173/` in a supported browser and stop the
preview with **Ctrl+C in the owning PowerShell console**. `build`
produces `frontend/dist/`; `run` serves that existing production
output, without building or starting a native Rust HTTP backend.
Verify source lock/receipt stability and that the owned child processes
exit and port 5173 can be rebound. The reviewed Windows console cleanup repair
first shipped in CLI 0.4.1. A Ctrl+C stop can return native `0xC000013A`. If an installed
CLI returns 125 after Ctrl+C, preserve its logs and confirm actual
owned-process cleanup instead of reporting cancellation PASS.

### Qualification scope and cross-platform evidence

The Windows x64 MSVC Browser first-use lane has accepted isolated
`new`/SDK receipts, locked Rust WASM/Vite build, TypeScript,
HTTP 200, seven asset byte/MIME matches (including
`application/wasm`) and an unchanged strict CSP **declaration**.
Ubuntu's existing real Chrome evidence covers the matching Browser
task model (add, complete and filter locally under SDK alpha.7) and
SDK alpha.8 Browser Style WASM startup/normal CSP with zero recorded
console/network errors. The template's Browser model and runtime
implementation are unchanged across the relevant source cohorts.
These are **distinct source-scoped observations**, not a claim of
Windows Chrome execution, identical build bytes, Browser Watch/HMR,
or an injected-inline-handler CSP denial test. Browser/CSP-specific
negative enforcement remains a separate risk-driven security check
when its contract changes; see the platform policy in
[repository instructions](../AGENTS.md).

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

## Historical native DOM controls and navigation boundaries

The original YT-110 source-bound browser/SSR exercise extended the finite
DOM vocabulary with native text/textarea/select/checkbox controls,
fieldset/legend, explicit reset/submit and labelled hints. An initial
`value`, `checked`, or option `selected` attribute supplies a
**browser default**, not a continuously synchronized live property.
`readonly` values submit; **disabled/unchecked** fields do not.
Boolean attributes use **presence** semantics, so the literal string
`false` still means present/true. On reset, the browser restores the
current page's captured defaults; stateful live edits/IME/focus restoration
after View replacement remain application-owned.

Native textarea content normalizes CRLF/CR to LF. The SSR serializer
preserves a requested *leading LF* that the HTML parser ordinarily
removes immediately after an opening textarea; do not add a second
manual leading-newline sentinel in application views. The sample's
`GET /controls` route is an escaped/bounded **preview only**, with
no Todo mutation, authorization, file upload, or CSRF assurance.
Do not put secrets in the GET query.

The selected **static input grammar** admits a hash or a single
root-relative slash for `href`, one root-relative slash for form
`action`, and `get`/`post` for form `method`. Empty, absolute,
protocol-relative, path-relative, backslash and control/whitespace
values are rejected by the scoped DOM policy. On actual mount, URL
resolution uses the effective document base and native URL parser:
only same-origin HTTP(S) targets without credentials survive;
the resolved URL is fixed on the detached replacement before
publication, so a later base change cannot redirect it. Invalid
structure, attribute or navigation **must leave the prior subtree
untouched**. The SSR serializer has a distinct broader URL policy,
and this small DOM restriction is **not** a universal sanitizer,
redirect check, HTTP authorization, focus preservation or Content
Security Policy.

For current released features follow the
[versioned DOM guide](../crates/yetlet-browser/guides/dom.md);
for the dated finite-element/attribute matrix and negative cases see
[HTML inventory history](API-CONTRACTS.md#historical-html-element-and-attribute-inventory)
and the full original report at local Git
`8b3e13004a58380accf16ea440707f84e6271871`.
Original rejected shapes, browser fixtures and review results remain
historical; this consolidation performs no new browser execution.

## API reference

- [CLI browser template and prerequisites](../tools/yetlet-cli/templates/browser/README.md)
  and [creation source](../tools/yetlet-cli/src/browser.rs).
- [DOM integration](../crates/yetlet-browser/guides/dom.md) and
  [keyed collections/listeners](../crates/yetlet-browser/guides/dom-keyed.md).
- [Reactive API source](../crates/yetlet/src/reactive/mod.rs) and
  [API documentation entry](API-REFERENCE.md).

This guide connects existing capabilities. It adds no runtime/API, platform
qualification or documentation-site deployment, and selects no desktop host.

## Keyed ownership and runtime retirement

The renderer-neutral identity/key/slot vocabulary validates deterministic
lowercase component/instance/slot tokens, nonempty control-free keys and inert
attribute names. It does not authorize raw HTML, URL attributes, inline handlers,
`style`, `srcdoc` or the reserved `data-yetlet-*` namespace. Renderers still own
escaping and DOM admission. See [View ownership](VIEW-COMPONENTS.md#binding-metadata-and-explicit-lifetimes)
and the [keyed module guide](../crates/yetlet-browser/guides/dom-keyed.md).

Keyed children own the complete direct-child key/entry set and caller row state.
Desired-key validation and creation of detached missing entries precede structural
mutation; an attached create result is rejected. Retained entries are updated,
then stale nodes removed and nodes inserted/moved. Removal disconnects the node
and drops its state/listeners; disposal removes propagation while leaving DOM
readable. Caller updates can mutate before returning an error and final browser
DOM errors have no transactional rollback. Applications own callback atomicity.

The M7b–M7d experimental fixture additionally preflights the exact SSR root,
instance, keyed rows, attributes, direct child/text slots and input live/serialized
values before installing listeners. It supports one controlled text input,
one conditional text and one click path per fixed row, not arbitrary nested
components/forms. It writes a live input value only when different, capturing
focus/selection before successful keyed moves and restoring only a still-connected
input. Removing the focused row deliberately does not focus a detached node.
Chrome 155 and Firefox 157 verified input identity/value/focus/selection, keyed
reorder/removal, detached input/click inertness and disposal; this is no real IME,
WebKit/mobile, screen-reader or general focus-manager qualification.

The synchronous graph owns committed state, reducer, commit callback, FIFO queue
and observers. Handles/subscriptions are weak; unsubscribe/drop releases captures.
An explicit outer batch reduces one cloned draft and commits/notifies once;
empty batches do nothing. Re-entrant work runs in later FIFO passes, bounded by
the experimental default 64. Reducer failure clears work and preserves usability;
commit failure faults the graph without publishing the draft; pass exhaustion
faults after any already-completed commits. Disposal clears owners/captures and
weak dispatch returns Disposed. Arbitrary commit callbacks are responsible for
external atomicity; observer panics are not caught. There is no automatic field
dependency tracking, async effect scheduler, worker or heap/GC guarantee.
The unchanged [M7g record](M7G-VERIFICATION.md) retains extraction evidence.

Todo's production local list opts into these primitives; remote result/Search
updates retain subtree replacement. Its route owner holds graph, listeners and a
lifetime token. Retirement invalidates the token first, including tasks queued by
`spawn_local`; HTTP completion checks it again after await before any DOM/status
write. Remount owns a fresh token and one listener path. The first final M7
review found this queued/in-flight race despite an earlier narrower PASS. Repaired
Chrome/Firefox debug/release tests proved click-then-dispose starts neither local
work nor HTTP, held old responses cannot mutate after remount, and retired nodes
stay inert. [Dated validation and costs](QUALIFICATION-MATRIX.md#keyed-renderer-and-lifecycle-evidence)
keep that FAIL separate from repeat final PASS.

## M11 native form and safe-content decision

The historical YT-071 outcome retained native SSR controls and an explicitly
enabled application-owned **read-only text preview**. It did not widen the then
generic DOM/URL allowlist or introduce controlled inputs, rich HTML, namespaces,
keyed control reorder or arbitrary user destinations. Later released DOM
coverage above is a separate source/qualification, not backdated M11 evidence.

False booleans are omitted; defaults differ from dirty live properties. Preview
reads current textarea/select/checkbox state, never rewrites user controls,
preserves identity/selection and uses owned abortable listeners/queued work.
Native reset restores defaults; synthetic composition suppression/end is not OS
IME proof. Chrome155 tested remount's single listener, disposal cancellation,
reset, native submit/303 PRG, no local mutation POST/event circuit and CSP.
The proof was stateless/unauthenticated, without credentialed-CSRF assurance;
Board supplied separate Session/authorization/CSRF/storage behavior.

Safe View escaping is contextual encoding, not a rich-HTML sanitizer, universal
URL/origin policy or protection for raw Axum Html/keyed callbacks. Store bounded
plain text, not pre-escaped strings, and test edit/save/read/preview/history as
one path. Page reapplies default CSP on its own error response; successful
render-page helpers attach CSP, directly converted renderer errors differ, and
string renderers cannot attach HTTP headers. Applications can overwrite headers.
Do not infer namespace support from a URL-policy name or assume compound URL
values/redirects are validated. Structured Markdown must reject raw HTML and
validate destinations; imported rich HTML/editor integration would require a
separately selected vetted sanitizer and mutation/parity/security review, not a
regex or new unconstrained RawHtml escape hatch.

Current-session self-review accepted this bounded application decision; it was
not independent security assurance. A broadened HTML/URL/live-control security
boundary or externally reachable authenticated product retains its separate
independent gate. Exact research and form-proof records remain at local Git
`40b66f1ad9bd0eb5666a9e1c8f113310993a3544`; remote access is unverified.

## Browser DX research and selected follow-up

The 2026-11-07 inspection used source
`bebdd03660ca7dc33fdd1ed4c7e56b5680e055c1`, SDK alpha.6 and CLI0.2.0.
It selected this connected walkthrough over adding a generator, event registry,
state abstraction or desktop host. Existing template preparation, matching
bindgen, frozen build, startup shell, listener disposal and frontend/dist
ownership supplied the workflow. Future SDK work still requires a concrete
separately selected limitation. This study selected no new API or platform test.

The browser template owns its local graph, typed commands, keyed rows and
listener handles. A caller-owned Rust closure, as in documentation search, is
sufficient for smaller interactions. Subtree replacement requires rebinding and
disposal, while keyed identity leaves live input, focus and selection to the app.
The [View lifetime contract](VIEW-COMPONENTS.md#binding-metadata-and-explicit-lifetimes)
and [asset contract](ASSETS.md#vite-feasibility-decisions-and-limits) retain those
boundaries. The earlier YT-104 starter and YT-112 Linux Rust/Wasm loop, including
its initial module-routing Reviewer FAIL, remain at local Git
`ed7a082b1f9f72430f06866f7d550b0ed9ecbae9`. The study added no selected Reader
chapter and replayed no accepted build/browser/watch/platform qualification.
