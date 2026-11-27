# Yetlet CLI guide

For registry installation, a native quickstart and the selected release, see the
[package README](../tools/yetlet-cli/README.md). This guide covers the command
reference, platform setup, project settings and source maintenance.

Use the [current CLI contract](CLI-CONTRACT.md), [published CLI 0.4.2
record](CLI-042-HOSTED-RECORD.md) and [published SDK alpha.9 record](ALPHA9-HOSTED-RECORD.md)
for version and distribution authority. The CLI build SDK, its immutable
packaged source snapshot and each generated application's selected SDK are
**different identities**. The default latest resolver, exact `--sdk-version`,
offline metadata/cache, generated Cargo.lock/creation receipt and finite
SDK startup capability policy are described below and governed by
[ADR 0010](adr/0010-dynamic-sdk-version-resolution.md), the retained
[YT-247 contract](CLI-GUIDE.md#sdk-resolution-yt-247) and
[YT-259 startup contract](CLI-GUIDE.md#generated-startup-profiles).

### Historical CLI and developer-loop qualification

Original YT-102, YT-104 and YT-111–112 source-only stages were narrower than
the current published CLI. The implementation, doctor, settings-error,
frontend dotenv confinement, generator source closure, Watch and HMR research
and selected Linux/Windows evidence remain recoverable at fixed **local Git**
`ed7a082b1f9f72430f06866f7d550b0ed9ecbae9` (not verified as publicly
reachable on GitHub). The [accepted CLI audit](QUALIFICATION-MATRIX.md#historical-cli-contract-process-and-sdk-resolver-evidence)
retains its historical F-01 finding, not a current unresolved security claim.
[Natural-completion latency](QUALIFICATION-MATRIX.md#historical-cli-contract-process-and-sdk-resolver-evidence) permits skipping
the Linux 500ms natural-exit grace only after a narrow WNOWAIT/single-thread/
retained-child proof; cancellation and descendant cleanup still run.
The [native Windows Vite pair](QUALIFICATION-MATRIX.md#historical-windows-native-browser-and-cli-evidence) separately
qualified its selected GNU process/console cases; compile-only and other
Windows/browser tests cannot be substituted for that evidence.

`run` with Rust watch rebuilds/restarts its owned native process generation;
it does **not** preserve arbitrary live Rust process state. Vite owns
frontend JS/TS/CSS HMR, with application-owned state disposal and bounded
accepted generation changes; not all edits support hot replacement. Compile
failure retains a recoverable previous generation only at the tested seams;
stale assets, failed child cleanup or tool/cache absence must surface as errors.
For an interactive browser-only project use the [Browser app walkthrough](BROWSER-APP.md);
for rolling v1/v2 public assets use [ASSETS](ASSETS.md#cache-and-retention)
and the retained [YT-103 actual qualification](ASSETS.md#historical-single-host-rolling-asset-qualification).
Source-scoped run/watch/asset proof is **not** production HA, universal platform
qualification or automatic dependency installation.

<a id="current-command-and-capability-inventory"></a>
## SDK resolution (YT-247)

This release implements the accepted
[SDK resolution contract](#sdk-resolution-yt-247).
The CLI's build dependency and embedded snapshot are independent of the SDK
chosen for each new project.

```sh
yetlet new demo --sdk-version 0.9.0-alpha.9
yetlet new cached-demo --offline
```

`--sdk-version` accepts one exact canonical SemVer (no ranges, prefix or build
metadata). Without it, select the highest available non-yanked SemVer, including
future releases and prereleases. Browser templates require both SDK packages at
the same version. Policy `sdk-latest-v2` uses `selection: latest-semver`; it has
no SDK approval list, stable-first preference, CLI build-SDK bound or frozen
MSRV ceiling. After selecting latest, required-feature and Browser dependency
checks fail that selected version without falling back to an older SDK.
Cargo owns actual toolchain and dependency feasibility. Successful lock creation
does not prove that a future SDK will compile or retain the template's APIs;
a later compiler, build or API failure remains a failure of the selected version.

Online refreshes required public index metadata; a network failure stops with
no stale fallback. `--offline` or `CARGO_NET_OFFLINE=true` forbids resolver and
Cargo network access. It uses CLI-owned metadata under Cargo home's
`yetlet/sdk-index-v1`, reports validation time/age and cache-limited latest, and
requires Cargo's separate dependency cache for lock generation. Missing/corrupt
metadata or missing Cargo cache fails with no published project. Cargo's own
net.offline configuration can further restrict the lock step; select CLI
`--offline` as well when relying on config to forbid resolver HTTP.

Successful `new` runs bounded Cargo lock generation in private staging,
then publishes one tree with exact SDK pins, Cargo.lock and a creation receipt at
`.yetlet/sdk-resolution.json`. No SDK source snapshot is copied. Browser pins both
SDK packages equally. Retain the app lock and receipt; later build/run/watch stay
frozen and never invoke the resolver. SDK pins fix SDK identity; different creation
dates/caches may produce different transitive locks. Existing destinations are
refused before tool probes, metadata access or lock generation. No tools, packages
or frontend dependencies are installed by new. The dependency-free nested Style
generator lock remains separate and unchanged.

The package README's Build SDK field describes the CLI's own exact dependency. The embedded
snapshot is alpha.8 at its recorded SDK source/digests for the existing
qualified native source exporter; app SDK resolution does not expand Docker or
Compose source support. Policy and snapshot identity resources ship in the CLI
archive and remain bound to that archive's checksum and immutable source.

### Generated startup profiles

For legacy SDK feature maps, CLI `0.4.2` checks the selected exact SDK version against its packaged startup
capability policy. A matching row must also match the registry index SHA-256 of
the core `yetlet` archive. The packaged policy lists SDK `0.9.0-alpha.8`.
An unlisted version, including `0.9.0-alpha.7`, uses the legacy listener for
simple Native and Native Style scaffolds on that same version. A checksum
contradiction for a listed version stops creation with
`SDK_INCOMPATIBLE` and `template-capability`. The check applies to all templates
and also runs offline. If a matched simple template cannot be composed, creation
fails with the same error category. Both failures leave the destination absent.
[Packaged policy](../tools/yetlet-cli/resources/sdk-compatibility.json)

| Generated selection | Unlisted SDK | Exact capability match | Generated startup |
|---|---|---|---|
| Plain Native | `legacy-axum` | `m26-app-serve` | Legacy uses an explicit listener. M26 uses the curated prelude and `App::new().page(...).serve()`. |
| Native Style or standalone Tailwind on Native or Native Style | `legacy-axum` | `m26-app-serve` | Compose the selected asset routes before serving. M26 uses `App::from_router(router).serve()`. |
| Native or Native Style with Settings or Vite, without Service | `configured-axum` | `configured-axum` | Explicit listener and `config::serve` preserve typed configuration, actual bound-address logging and optional development TLS. Vite implies Settings. |
| Native or Native Style with Service | `service-axum` | `service-axum` | Explicit listener, health and readiness routes, and graceful shutdown, with `config::serve` when Settings or Vite is selected. Service takes precedence over those options. |
| Browser or Browser Style | `browser-wasm` | `browser-wasm` | The existing frontend and Wasm startup uses the exact SDK package pair. Browser templates reject Service. |

The first two rows apply when Settings, Vite and Service are absent. Simple M26
scaffolds use fixed HTTP at `127.0.0.1:7878`. The legacy simple listener reads
`YETLET_PORT`. To choose a different address in M26 app code, use
`App::serve_on(address)`. To inspect the actual address after binding port
`0`, coordinate graceful shutdown, or terminate TLS directly, own the native
listener. [SDK startup API](API-REFERENCE.md#published-application-startup--sdk-alpha8)

The top-level `startup_profile` in the schema 1 creation receipt at
`.yetlet/sdk-resolution.json` records the branch that emitted the source.
`sdk.policy_sha256` identifies the packaged policy bytes used at creation, and
`cargo_lock_sha256` identifies the generated lockfile bytes. The receipt records
creation provenance. The manifest and lockfile govern subsequent builds.

<a id="source-only-feature-module-profiles"></a>
### Published feature-module profiles

Published CLI 0.4.2 also recognizes selected core SDK metadata declaring web
or view. It chooses that API profile before validating its feature closure; an
incomplete new profile fails with SDK_INCOMPATIBLE and upgrade/explicit-pin
guidance. It does not fall back to legacy imports or an older SDK. Legacy metadata
uses the startup table above and its checksum-bound capability rule.

| Generated template selection | Explicit core features | Startup receipt |
|---|---|---|
| Simple Native or Native Style, optionally Tailwind | web, html | web-app-serve |
| Native with Settings or Vite | web, html, config | web-configured |
| Native with Service | web, html; config when Settings/Vite is selected | web-service |
| Browser or Browser Style | reactive, view; companion selects dom and dom-keyed | browser-wasm |

All generated SDK dependencies disable defaults. Native uses `yetlet::web` for HTTP/SSR. Winit Hosts use the SDK's separate `native-winit` feature. Style, Tailwind and Vite
composition finish before the shared API-profile conversion. The policy and receipt schemas are unchanged.

The preserved published-contract CLI 0.4.1 rejects the new Native feature map
because server-axum is absent. Its existing diagnostic does not include explicit
upgrade wording. Installed users can retain the published Native contract with:

```sh
yetlet new my-app --template native --sdk-version 0.9.0-alpha.8
```

Use CLI `0.4.2` for SDK alpha.9. The
[public alpha.9 consumer](ALPHA9-HOSTED-RECORD.md#owner-approved-publication-and-registry-consumer--2026-11-23)
passed Native, Service and Chrome/Wasm checks with that pair. See the
[compatibility assessment](COMPATIBILITY.md#current-checkout--cli-and-selected-sdk)
and [bounded evidence](M28-HOST-RESOURCE-LIFECYCLE-DESIGN.md#21-yt-293--cli-consumers-and-minimal-sdk-feature-selection).

## Historical SDK resolver, startup and first CLI release evidence

The present [SDK resolution](#sdk-resolution-yt-247) and
[generated startup profile table](#generated-startup-profiles)
are the actual CLI0.4.0 behavioral contract. The original
YT-247 Phase A/Phase B design had an **obsolete**
`sdk-compat-v1` alpha.6/alpha.7 policy; it is not a current
approval list or default pin. Current `sdk-latest-v2` without an
explicit version selects the greatest valid non-yanked SemVer,
**including prereleases**, using complete registry metadata.
It does not prefer stable, bound latest to the CLI build SDK,
silently fall back when chosen SDK is incompatible or substitute
old cache metadata after an online failure. Effective offline
(`--offline` or `CARGO_NET_OFFLINE=true`) forbids resolver
transport, requires a valid local metadata set *and* separate
Cargo crate cache, and prints an age/cache-scope notice. Missing
or corrupt proof fails closed; latest known offline is not
promised to be globally current.

The CLI build dependency, its immutable version-bound packaged
source snapshot and the exact new-app selected SDK are
**three identities**. A `new` request uses private protected
staging, checks registry origin/SDK checksum and a new generated
`Cargo.lock`, preserves any independent stylegen lock, and
publishes only an absent destination with a schema-1
`.yetlet/sdk-resolution.json` source/selection receipt.
`build`/`run`/`watch` never resolve or auto-upgrade the SDK;
the actual retained lock governs transitive reproducibility.
Original binary `new` refusal on unsafe tools and early
Windows/MSVC correction cases were independent historical
failures, not first-pass success.

For YT-259's source-specific M26 proposal, the five
`startup_profile` values are `legacy-axum`, `m26-app-serve`,
`configured-axum`, `service-axum`, `browser-wasm`;
a **matching exact SDK core archive index checksum** is required
for any explicitly listed capability row, including offline.
An unlisted version remains eligible but emits the legacy
simple native listener on *that same SDK*, not a downgrade.
A listed checksum contradiction reports
`SDK_INCOMPATIBLE/template-capability` with no destination.
Published CLI0.4.1 advertises the verified alpha.8 capability
row; unlisted alpha.7 keeps legacy behavior. The simple
capability-matched Native/Native Style `.serve()` path uses
`127.0.0.1:7878` and ignores `YETLET_PORT`, while a legacy
simple scaffold reads `YETLET_PORT`. Explicit
`serve_on` selects a different address; configured-axum,
Service/genuine shutdown, Vite/Settings TLS and Browser/Wasm
remain distinct. A generated receipt's startup profile is a
creation fact, not a release attestation or change in running
app policy. The old YT-259 contract-only and local MSVC
qualification gates are **not** proof that every later SDK
archive supports the M26 APIs; verify the exact published
archive and locked consumer.

The original CLI0.1.0 GitHub/crates.io publication succeeded,
but its anonymous installed binary printed
`yetlet 0.1.0 (unpublished preview)`; the full registry
consumer therefore **FAILED** at version assertion before
help/new. Earlier custody readback mistakenly bound the SDK tag
and was independently corrected; neither green preliminary
job nor upload success changes the consumer result.
The later separately authorized CLI0.1.1 package, SHA-256
`10a1ffebf42de0baba9604ee2764b2304f601c5119ddcb5fcb988fbe319e1424`,
passed anonymous install/version/help/`new` (original SDK
alpha.5 pin) without rewriting 0.1.0 or claiming that
generated applications compiled. Current distribution and
newer versions are governed only by accepted
[hosted CLI release records](CLI-042-HOSTED-RECORD.md#owner-approved-publication-and-installed-consumer--2026-11-23).
Original failed, consumed and accepted records are retrievable
at fixed **local Git**
`8b3e13004a58380accf16ea440707f84e6271871`;
their task-specific raw CI/ZIP receipts are separately owned,
not guaranteed by Git or publicly reachable at that SHA.

## Historical CLI source evidence and operation limits

Current `new` SDK resolution, app `Cargo.lock`, source receipt, guarded
temporary creation and online/offline selection are defined by the
[SDK version contract](CLI-GUIDE.md#sdk-resolution-yt-247)
and [CLI process contract](CLI-CONTRACT.md#historical-cli-source-qualification-and-process-constraints).
Do not select the old Phase B `sdk-compat-v1` alpha.7 default merely because
the historical source/implementation report used it; the currently selected
policy is `sdk-latest-v2` and each new project is pinned to the result of
its actual resolver decision.

The [historical qualification matrix](QUALIFICATION-MATRIX.md#historical-cli-contract-process-and-sdk-resolver-evidence)
keeps the original linked-dotenv audit failure, Linux guarded natural-exit
fast path and noncausal warm timings, Windows native process/cancellation
statuses, and the separately corrected SDK Resolver initial tests.
Do not substitute a compile check for an executed Windows CLI, a Linux
fixture for registry access or a source-only implementation review for the
published SDK/CLI package consumer.

## CLI command and capability inventory

| Command | Implemented behavior |
|---|---|
| `yetlet --help` / `-h` | Root help; every complete command path also has exclusive scoped help |
| `yetlet --version` / `-V` | Build-time CLI version; no project, tools or settings required |
| `yetlet new NAME [--parent DIRECTORY] [--template native\|browser\|native-style\|browser-style] [--sdk-version VERSION] [--offline] [--vite] [--tailwind] [--service] [--settings]` | Resolve Registry latest SemVer or an explicit exact SDK, write exact pins and generate Cargo.lock with a creation receipt before protected publication; no dependency installation or app build |
| `yetlet doctor [--json]` | Bounded environment-only Rust/Cargo identity and compiled filesystem capability; schema 1 JSON |
| `yetlet inspect [--manifest-path PATH] [--json]` | Read-only public-settings plan for build/run/watch; separate project-execution-plan schema 1; no readiness checks |
| `yetlet key status [KEY] [--provider NAME] [--manifest-path PATH]` | Read-only selected key/version, certificate pins and dates; no password or generation |
| `yetlet key rotate [KEY] [--provider NAME] [--manifest-path PATH] [--yes] [--from PATH]` | Confirmed local rotation and atomic selected-profile settings update; retain old versions |
| `yetlet run [--manifest-path PATH]` | Native frozen Cargo/watch/Vite, or settings-selected browser-only static preview |
| `yetlet watch [--manifest-path PATH]` | Native Rust watch, or explicitly selected browser WASM rebuild/full reload; Vite still uses settings |
| `yetlet build [--manifest-path PATH] [--release]` | Native frontend-then-Cargo, or settings-selected browser-only production build |
| `yetlet build --static --export-bin NAME --output DIRECTORY [--base-path /] [--release] [--wasm-islands]` | Available since CLI `0.2.0`: finite public HTML export; explicit native entry, create-only output and private receipts |
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
See the [native Windows diagnosis and verification](#protected-windows-tool-preparation).

## Static HTML build

This release includes static mode, introduced in CLI `0.2.0` and absent from
CLI `0.1.1`. Install the selected CLI version from the registry or build it from
source, then follow the
[standalone static example](https://github.com/bohewu/yetlet/blob/master/examples/static-site/README.md) for its exact SDK and CLI versions. The example
uses registry dependencies.

The canonical invocation is build --static --export-bin NAME --output DIRECTORY,
with optional --base-path /yetlet/, --release and --wasm-islands. It runs one
explicit trusted native exporter with frozen Cargo, validates finite protocol1
fragments/assets and writes complete public HTML. It does not crawl handlers,
start a backend, install tools, clean/overwrite output or deploy. The native
exporter has a separately bounded execution deadline and explicit child-process
cleanup; uncertain supervision or publication outcomes cannot report success.
Source/input receipts describe observed inputs, not a sandboxed or hermetic
compiler. Existing ordinary CLI commands keep their separate execution rules. HTML-only apps
need no Node/Wasm/server or runtime Settings loader.

Existing public settings are still validated and cli.target must be native.
The explicit static mode does not select watch/Vite auto stages. No new settings
keys are added; --release changes Cargo profile only. Static export differs from
the existing browser-only target, whose statically hosted app mounts after Wasm.
Ordinary build/run/watch and inspect schema1 retain their settings-selected scope;
inspect does not plan static mode.

For static mode, preflight captures CARGO_TARGET_DIR (otherwise
CARGO_BUILD_TARGET_DIR, otherwise app/target) and a plain CARGO_BUILD_BUILD_DIR
(otherwise that target). The child explicitly uses these same target/intermediate
directories, overriding file-config directory selection; Cargo path templates in
the intermediate path are refused. Output cannot overlap either selected tree,
including a physical cache alias. These are existing Cargo variables, not Yetlet
settings; other CLI modes retain their existing Cargo behavior.

Choose an absent output below app-local artifacts/ or a separate protected
existing parent. Another build requires another directory; prior output remains
the last-good artifact. / and /yetlet/ base paths map canonical directory-index
routes without emitting a base element or SPA fallback. The host owns directory
indexes,404 status, MIME and any response headers beyond meta CSP.

--wasm-islands admits already-prepared local JS/Wasm assets and a selected module
entry; it does not invoke npm or create a bundler. The application owns readable
initial content, activation, startup/error text and disposal. Private receipts
under target/yetlet-static/RUN_ID/ bind selected inputs/output hashes and outcomes;
keep them outside uploaded content. A supervision error retains private export
evidence and does not claim child cleanup or success. No automatic retry.

### Static export release and validation scope

`Cli::BuildStatic` and `build_static` are Experimental support APIs of the
independent `yetlet-cli` package, available since `0.2.0`; downstream exhaustive
Rust matches may need a new arm. Static export does not add an SDK module or
require the optional DOM renderer for HTML-only output.

The [original implementation record](https://github.com/bohewu/yetlet/blob/b9873f15bcfd4c36a2274508ef7517e5a83b807b/docs/YT-223-STATIC-HTML-IMPLEMENTATION.md)
retains GNU/MSVC native CLI execution, bounded Chrome/Firefox static/island
checks and their failures. [CLI 0.2.0 release qualification](https://github.com/bohewu/yetlet/blob/master/docs/CLI-020-RELEASE-PREPARATION.md#local-affected-evidence--2026-11-01)
separately verifies the packaged CLI, a generated registry-SDK alpha.6 app and a
six-file static export. The [actual publication and installed consumer](https://github.com/bohewu/yetlet/blob/master/docs/CLI-020-RELEASE-PREPARATION.md#owner-approved-publication-and-installed-consumer--2026-11-02)
prove registry installation, exact version, help and scaffolding; they do not
rerun or broaden the earlier static/browser qualification. SDK registry
resolve/compile evidence alone does not qualify CLI execution or browser behavior.

## First-use prerequisites

For one guided plain-native path, follow [Build your first Yetlet app](https://github.com/bohewu/yetlet/blob/master/docs/FIRST-APP.md).
For the existing browser target, follow [Build a browser-only Yetlet app](https://github.com/bohewu/yetlet/blob/master/docs/BROWSER-APP.md).
This reference retains the broader target, settings and execution choices.

Prepare tools and dependency caches explicitly before running the examples.
Building the CLI from source requires OpenSSL headers/libraries through its
native signing dependency, even if you only plan to create a browser app.
Plain generated native/browser apps do not acquire that CLI dependency.
Building the CLI and building an app are separate steps.

| Selection | Tools and inputs to prepare | Initial workflow |
|---|---|---|
| Build this CLI from the checkout | Rust/Cargo 1.99+, a native C/link toolchain, OpenSSL development files (reference: 3.5 LTS) and cached locked dependencies | [Build the CLI](#build-locally-and-create-an-application), then inspect help/doctor |
| Native `new NAME` | Protected Rust/Cargo 1.99+ and registry access, or prepared resolver/Cargo metadata caches with `--offline` | Retain the generated Cargo.lock, prepare dependency caches explicitly, then `yetlet build` and `yetlet run`; no Node or frontend tool is required |
| Native `new NAME --vite`, optionally `--tailwind` | Native prerequisites plus Node `^20.19.0` or `>=22.12.0`, npm and the generated frontend dependencies | [Vite preparation](https://github.com/bohewu/yetlet/blob/master/tools/yetlet-cli/templates/vite/SELECTED-README.md), initial `yetlet build` with auto_build enabled, then `yetlet run` |
| `new NAME --template browser`, optionally `--tailwind` | Rust/Cargo 1.99+, `wasm32-unknown-unknown`, wasm-bindgen-cli 0.2.129, the same Node/npm prerequisites and the generated dependencies | [Browser preparation](https://github.com/bohewu/yetlet/blob/master/tools/yetlet-cli/templates/browser/README.md), `yetlet build`, then `yetlet run` to preview the existing dist |
| Native `new NAME --tailwind` without Vite | Native prerequisites plus the separately provisioned standalone Tailwind 4.3.3 CLI | [Build `dist/tailwind.css` explicitly](https://github.com/bohewu/yetlet/blob/master/tools/yetlet-cli/templates/tailwind/SELECTED-README.md) before running Rust |

The Vite Tailwind selection uses the official npm plugin, without a standalone
CSS compiler. Native Rust watch additionally needs separately prepared
[Watchexec](#rust-watch); development HTTPS needs separately prepared
[mkcert and certificates](https://github.com/bohewu/yetlet/blob/master/tools/yetlet-cli/templates/config/SELECTED-README.md#development-https).
Settings-enabled native apps, including `--vite`, need their own service token
before startup; use the existing [settings/Development-secret mechanism](https://github.com/bohewu/yetlet/blob/master/tools/yetlet-cli/templates/config/SELECTED-README.md),
keeping credentials out of public TOMLs and frontend files.

Use this order: prepare tools and the required resolver/Cargo caches, build the
CLI, then create the selected app. A successful `yetlet new` resolves and pins
the selected SDK, generates `Cargo.lock` in private staging, records
`.yetlet/sdk-resolution.json`, and publishes both with the app. Fetch the
already-generated lock with `cargo fetch --locked`; do not regenerate it.
Prepare any selected frontend dependencies/assets, then build/run. `new` does
not install tools or packages, build the app, start a listener or produce `dist`;
selected frontend templates include their own pinned package lock. `run` does
not create the initial dist. With frontend `auto_build` disabled, prepare it
separately before startup.
Offline commands require an already populated cache. CLI execution never
installs dependencies or repairs locks automatically.

`yetlet doctor --json` checks the local Rust/Cargo environment and compiled
filesystem capability only. A passing result does not check OpenSSL development
files for a CLI rebuild, app dependencies/settings/secrets, Node/npm, WASM tools,
frontend dist or whether `run` will work. Use the selected setup instructions
and actual build/run for those checks. On Windows, also prepare protected
physical tool paths and an attached processed-input console for native run;
see [native tool preparation](DEVELOPMENT.md#protected-windows-tools-and-historical-exit-125).

## Build locally and create an application

Follow the [registry installation instructions](../tools/yetlet-cli/README.md#install)
for the selected CLI version.

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
registry SDK versions and include Cargo.lock and `.yetlet/sdk-resolution.json`.
Prepare the dependency cache explicitly with `cargo fetch --locked`; CLI
build/run then use frozen Cargo. Offline
preparation requires a populated cache. `--release` changes the Cargo profile,
not APP_ENVIRONMENT. Frozen execution is not a sandbox for package scripts,
build scripts or app code. For the complete walkthrough and expected HTTP
result, see [Build your first Yetlet app](https://github.com/bohewu/yetlet/blob/master/docs/FIRST-APP.md).

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

YT-104 adds an explicitly selected [browser-only Rust/WASM task starter](https://github.com/bohewu/yetlet/blob/ed7a082b1f9f72430f06866f7d550b0ed9ecbae9/docs/YT-104-BROWSER-STARTER.md):
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
auto-discovery is disabled. See [configuration](https://github.com/bohewu/yetlet/blob/master/docs/CONFIGURATION.md).

### Settings selection decision table

For `run`, `watch` and `build`, resolve these choices in order using the
existing [CLI settings projection](https://github.com/bohewu/yetlet/blob/master/tools/yetlet-cli/src/settings.rs) and
[shared Loader](https://github.com/bohewu/yetlet/blob/master/crates/yetlet/src/config/mod.rs):

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

After successful [application preflight and settings selection](https://github.com/bohewu/yetlet/blob/master/tools/yetlet-cli/src/app.rs):

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

See the [signing example](https://github.com/bohewu/yetlet/blob/master/examples/signing/README.md) for the complete
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

Watch covers app-root Rust `*.rs` (including `build.rs`), `Cargo.toml` and
`Cargo.lock`. The CLI built from this checkout also watches authored `*.yetlet`
single-file components under the app root. Saving a correction after a component
compile error triggers another build in the same session. Component saves alone
do not trigger a rebuild in the published CLI 0.4.2.

Watch excludes the configured frontend root, `target`, `node_modules`, `dist`,
the `.yetlet/` metadata directory and `CARGO_TARGET_DIR` when under the app root.
It does not follow symlinks or watch local dependencies outside the app root.
For a special target directory from Cargo config, set `CARGO_TARGET_DIR` to
exclude an in-root location or put the directory outside the watched root.

CLI settings are loaded once per run; rerun the CLI to change watch/Vite policy.
The fresh backend reads its runtime settings on each restart. Browser refresh,
in-process state preservation and graceful request draining are not promised.
Historical Vite lifecycle evidence does not qualify the added watch mode.

## Native CLI platform and generated deployment boundaries

The original M14 Linux and Windows GNU/MSVC work separated only two CLI
OS adapters: owned filesystem paths/no-replace with reparse/junction and
Unicode safety, and foreground child/process ownership. The Windows
source-bound proof used an owned Job Object (including descendant
cleanup) and native Ctrl-C/Break console rules; Windows group signals
and POSIX exit semantics are **not interchangeable**. A source compile
alone was not the real native generated-app/SSR/cancellation proof, and
the original GNU/MSVC checks do not certify every current release
binary, Windows container, terminal or production environment.

The separate M15 opt-in Docker, Compose and K8s CLI commands generate
**deterministic configuration files** targeting Linux workloads. They
neither build/pull/push images nor authenticate to a registry, connect
to a cluster or deploy workloads. The early Docker *design review FAIL*
identified unsafe default build context and missing transitive Cargo
source closure; the corrected independent design required three
physically separated build contexts with deny-by-default source export
and a complete minimal Cargo workspace. Later real Linux Docker and
isolated K3s results and Windows generated-config parity accepted only
their selected source/host slices, not general production HA or
multi-architecture images. An application's host binding, TLS/ingress,
health/readiness truth, Secrets, backups and operating SLOs remain
application/operator decisions. Existing app source is never silently
rewritten just because a deployment manifest is generated.

The [current generator matrix](#current-generator-source-matrix) lists
supported CLI input shapes; see [deployment profiles](PRODUCTION-DEPLOYMENT-PROFILES.md)
for operating responsibility and [CLI contract](CLI-CONTRACT.md) for
command spelling. Full original review failures and source tests are
recoverable from **local Git** `42d8856b9acf5ddddab9a4b998b5bdd4c5e59210`; that SHA is not proven
publicly reachable. These historical checks are not a fresh test of
the published CLI 0.4.0 binary.
## Current generator source matrix

The deployment exporters retain their separately qualified legacy SDK source
representation. Feature-module template support does not expand that source
allowlist. Within the retained representation, generators accept these exact
`new --service` shapes after explicit compatible local Cargo path overrides and
separately prepared locks:

| Current creation choice | Source shape and current acceptance |
|---|---|
| `new NAME --service` | Public CLI settings; no runtime settings dependency or token; accepted by all three generators |
| `new NAME --service --settings` | Public CLI/runtime TOMLs and the exact Config/Secrets source closure; accepted by all three generators |
| `new NAME --service --vite` | Settings implied; accepted with the validated Production Vite asset graph |
| `new NAME --service --vite --tailwind` | Accepted with the pinned official plugin and validated Production Vite asset graph |

The separate [2026-10-22 source-closure repair](https://github.com/bohewu/yetlet/blob/ed7a082b1f9f72430f06866f7d550b0ed9ecbae9/docs/YT-102-GENERATOR-SOURCE-CLOSURE.md)
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
[compatibility evidence](https://github.com/bohewu/yetlet/blob/ed7a082b1f9f72430f06866f7d550b0ed9ecbae9/docs/YT-102-GENERATOR-COMPATIBILITY.md).

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
[YT-101 evidence](https://github.com/bohewu/yetlet/blob/b610acc7e098020b6b59dfa0d1ab2b75b059e656/docs/YT-101-VERIFICATION.md). Standalone Tailwind,
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
[generated Tailwind instructions](https://github.com/bohewu/yetlet/blob/master/tools/yetlet-cli/templates/tailwind/SELECTED-README.md).
[YT-091 evidence](https://github.com/bohewu/yetlet/blob/b610acc7e098020b6b59dfa0d1ab2b75b059e656/docs/YT-091-VERIFICATION.md) retains the historical
source-bound native/browser acceptance; it does not qualify the new settings
representation.

## Windows PowerShell (native GNU/MSVC)

### Windows x64 MSVC first app and native watch

This PowerShell walkthrough keeps the earlier published CLI `0.4.1`
and SDK `0.9.0-alpha.8` pair used in the Windows x64 MSVC reference.
The current Linux/POSIX [First App](FIRST-APP.md) selects the newer pair. Do not paste POSIX shell commands into PowerShell. This is a plain
Native SSR workflow: Node, Vite, WASM and a database are not required.
Keep the CLI installation, app SDK selection, app Cargo cache and optional
Watchexec installation separate. The CLI installs none of those tools.

Use a new **x64 Developer PowerShell for Visual Studio 2022** (or initialize
the same developer environment in an ordinary PowerShell process). The example
below selects the MSVC tools for this process only; it does not alter system
PATH, registry, ACLs or an existing Rust installation:

```powershell
$vswhere = Join-Path ([Environment]::GetEnvironmentVariable('ProgramFiles(x86)')) 'Microsoft Visual Studio\Installer\vswhere.exe'
if (!(Test-Path -LiteralPath $vswhere)) { throw 'Install VS 2022 C++ x64 tools first' }
$vs = & $vswhere -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath |
    Select-Object -First 1
if (!$vs) { throw 'VS 2022 x64 MSVC tools not found' }
Import-Module (Join-Path $vs 'Common7\Tools\Microsoft.VisualStudio.DevShell.dll') -ErrorAction Stop
Enter-VsDevShell -VsInstallPath $vs -DevCmdArguments '-arch=x64 -host_arch=x64' -SkipAutomaticLocation
Get-Command cl.exe, link.exe -ErrorAction Stop | Select-Object Name, Source

$cargo = (& rustup which --toolchain 1.99.0 cargo)
if ($LASTEXITCODE -ne 0 -or !$cargo -or !(Test-Path -LiteralPath $cargo)) {
    throw 'Select an already-installed physical x64 MSVC Rust 1.99 toolchain'
}
$rustBin = Split-Path -Parent $cargo
$env:PATH = "$rustBin;$env:PATH"
& (Join-Path $rustBin 'rustc.exe') -vV
& $cargo -vV
$cargoHome = Join-Path $env:LOCALAPPDATA 'YetletCargo040'
$targetRoot = Join-Path $env:LOCALAPPDATA 'YetletNativeTarget040'
if ((Test-Path -LiteralPath $cargoHome) -or (Test-Path -LiteralPath $targetRoot)) {
    throw 'Choose unused, owner-protected local Cargo cache and build directories'
}
New-Item -ItemType Directory -Path $cargoHome -ErrorAction Stop | Out-Null
New-Item -ItemType Directory -Path $targetRoot -ErrorAction Stop | Out-Null
$env:CARGO_HOME = $cargoHome
$env:CARGO_TARGET_DIR = $targetRoot
```

The selected `rustc -vV` must report `host: x86_64-pc-windows-msvc`,
and Cargo must be version 1.99. Confirm the real toolchain and ancestors
are owner-protected local fixed NTFS paths with no reparse traversal. If
Rustup points to a rejected or redirected tool directory, select an
already-prepared trusted physical toolchain instead; do not disable Yetlet's
path, executable or ACL checks. A compiler build also requires the native
MSVC linker and any source-install OpenSSL headers/libraries. When using a
separately prepared OpenSSL prefix, configure `OPENSSL_DIR` and related
linker variables **only in this PowerShell process**. This prerequisite is
for compiling the CLI; the generated plain Native app does not opt into
signing. Alternatively, reuse an already installed, verified CLI `0.4.1`
from an accepted installation; no prebuilt executable is promised by the
source release.

For a fresh **source installation from the published crates.io CLI
package**, choose an unused, owner-protected local prefix. The two version
numbers are independent:

```powershell
$cliRoot = Join-Path $env:LOCALAPPDATA 'YetletCli041'
if (Test-Path -LiteralPath $cliRoot) { throw 'Choose an unused CLI prefix' }
New-Item -ItemType Directory -Path $cliRoot -ErrorAction Stop | Out-Null
& $cargo install yetlet-cli --version 0.4.1 --locked --root $cliRoot
if ($LASTEXITCODE -ne 0) { throw 'CLI installation failed; retain the Cargo error' }
$yetlet = Join-Path $cliRoot 'bin\yetlet.exe'
& $yetlet --version
& $yetlet --help
& $yetlet doctor --json
if ($LASTEXITCODE -ne 0) { throw 'Doctor failed; inspect its environment report' }
```

`doctor` reports schema 1, scope `environment`: installed Rust/Cargo
identity and compiled filesystem capability. Its `ok` field does **not**
certify OpenSSL for rebuilding the CLI, app locks, Settings, secrets, Vite,
assets, console behavior, or application readiness. Do not treat its pass as
permission to skip the app's actual build/run.

Use a fresh trusted app parent and explicit SDK pin. Keep network access
for the **first** SDK metadata resolution and Cargo lock generation;
`--offline` requires both cached resolver metadata and Cargo inputs and
must fail when either is missing:

```powershell
$appParent = Join-Path $env:LOCALAPPDATA 'YetletFirstApps'
if (Test-Path -LiteralPath $appParent) { throw 'Choose an unused protected app parent' }
New-Item -ItemType Directory -Path $appParent -ErrorAction Stop | Out-Null
& $yetlet new firstapp --parent $appParent --sdk-version 0.9.0-alpha.8
if ($LASTEXITCODE -ne 0) { throw 'Native scaffold creation failed' }
$app = Join-Path $appParent 'firstapp'
$manifest = Join-Path $app 'Cargo.toml'
if (!(Test-Path -LiteralPath (Join-Path $app 'Cargo.lock')) -or
    !(Test-Path -LiteralPath (Join-Path $app '.yetlet\sdk-resolution.json'))) {
    throw 'Generated lock or SDK resolution receipt missing'
}
Push-Location $app
try {
    & $cargo fetch --locked
    if ($LASTEXITCODE -ne 0) { throw 'Locked app dependency fetch failed' }
    & $yetlet inspect --manifest-path $manifest --json
    if ($LASTEXITCODE -ne 0) { throw 'Inspect failed' }
    & $yetlet build --manifest-path $manifest
    if ($LASTEXITCODE -ne 0) { throw 'Native build failed' }
    & $yetlet run --manifest-path $manifest
} finally {
    Pop-Location
}
```

Use a real attached Windows console with **processed Ctrl+C input** for
`run`. Open `http://127.0.0.1:7878/` and `/hello/Ada` (both should return
HTTP 200 with default CSP) and press Ctrl+C to exit before proceeding.
`run` does not turn on Rust watch by default. The generated
`Cargo.toml`, `Cargo.lock` and `.yetlet/sdk-resolution.json` belong to
this app; do not copy another workspace's lock, inject local SDK paths,
or silently re-resolve dependencies. Frozen build/run/watch do not
download missing crates, repair the lock, or install tools.

`inspect --json` is schema 1, scope `project-execution-plan`: a
read-only explanation of the selected settings, Native target and intended
`build`/`run`/`watch` stages. It starts no child and does not validate tools,
TLS, assets, secrets, listener availability or runtime readiness. For this
plain Native app, Vite and default Rust watch are off; explicit `watch`
selects the Rust watcher.

Optionally prepare a **separate, trusted Watchexec 2.8.0** executable on
the current process PATH, then check its identity before starting Native
Watch. Yetlet does not download it. From the same protected app:

```powershell
$watchexec = Get-Command watchexec.exe -ErrorAction Stop
& $watchexec.Source --version  # Expect Watchexec 2.8.0
& $yetlet watch --manifest-path $manifest
```

Edit the existing home-handler text in `src\main.rs` and save: Native Watch
must rebuild and launch a **new process**, not preserve in-process state.
An attached console cancellation must stop the owned Watch/Cargo/app tree
and release port 7878. This is **not** Browser/Vite Watch, not HMR,
and not a guarantee of arbitrary hot reload. Compilation errors can leave
the watcher awaiting the next source change; check its terminal output.

### Windows x64 MSVC command/evidence matrix (M27-C, 2026-11-16)

This is an evidence snapshot, **not a universal Windows support claim**.
The retained local Windows Runner record is
`artifacts/verification/m27-c-windows-msvc-20261116/`, based on
`1312006841ddf883d578da84f8bf38bfcfddeccd`. The exercised
`yetlet.exe` is a previously built **CLI 0.4.0** with SHA-256
`8a8c2b436755a14f0f3a6fe9737f574af71c1b249b4f8fd6486282d91af39a74`;
it is **not** asserted to be byte-identical to the published CLI crate.
The M27-B source difference from the original PR #94 build checkout is
the Service-only README composition in `scaffold.rs`; Doctor, Inspect,
native Run/Watch and plain Native scaffold code are unchanged.

| Lane | Native MSVC result and evidence | Boundary |
|---|---|---|
| CLI/toolchain | `0.4.0`; rustc/Cargo `1.99.0`, `x86_64-pc-windows-msvc`, VS C++ tools `14.44.35207`; `doctor/toolchain-probes.json` | No global toolchain or ACL change |
| Doctor | Exit 0; `doctor/doctor.json`: schema 1, `environment`, Rust/Cargo/Windows filesystem pass; app readiness not checked | Not an app/console/OpenSSL certification |
| Native new/lock | Initial offline metadata cache miss retained (`first-use/yetlet-new.exit` = 1); separate online `new` exit 0, exact SDK alpha.8, `m26-app-serve` and lock/receipt SHA match (`first-use/yetlet-new-online-summary.json`) | Registry metadata is distinct from `cargo fetch`; offline failure was not a successful first pass |
| Fetch and Inspect | `cargo fetch --locked` exit 0; `inspect` exit 0 and schema 1 `project-execution-plan`; `inspect/files-before.json` / `files-after.json` are unchanged | Settings-derived plan only; no runtime readiness proof |
| Build and ordinary Run | Reuse the **separate** accepted published CLI 0.4.0/SDK alpha.8 Windows Native locked first-app result ([M26 Roadmap](ROADMAP.md#m26--application-startup--sample-experience), [CLI release record](CLI-040-HOSTED-RECORD.md)); HTTP 200 at two routes, strict CSP, ordinary Run Ctrl+C and port reuse | This M27-C run did not replay standalone `build`/`run`; their original archive/source identity remains separate from the local source-built executable |
| Native Watch | **New native MSVC runtime**: Watchexec `2.8.0` (SHA-256 `0a362aa3bb944337f9d2e8d37f8a4fdc4aba2f3a6d83e5d69df43a7eedfbfd14`); initial HTTP 200, changed-source HTTP 200, distinct PID, old generation reaped, owned tree and development listener closed (`native-watch-summary.json`, `native-watch-process-tree.json`) | Test sent **Ctrl+Break**, not Ctrl+C; native cancel exit `0xC000013A` is expected and not exit 0. No Browser/Vite/Node runtime |
| Historical comparison | Windows GNU Rust/Vite pair and older MSVC single-Cargo/process or compile qualifications remain in [Qualification Matrix](QUALIFICATION-MATRIX.md#historical-windows-native-browser-and-cli-evidence) | GNU does not establish current MSVC Browser/Vite; current Native Watch does not retroactively pass historical cases |

Keep all failed setup/test attempts alongside successful evidence, including
the original MSVC console-mode test-harness failures and the M27-C offline
metadata miss. The M27-C Watch test performed a **Ctrl+Break** cancellation;
the already accepted M26 ordinary Run evidence covers **Ctrl+C** on its
separate published source. Do not describe the Watch test as a Ctrl+C
test or combine their exit semantics. No new API, command, JSON schema,
dependency, installer, production readiness or Pages release follows.

### Historical Windows source-checkout example

For maintainer-only evaluation, the earlier source-checkout commands remain
separate from the published install path:

```powershell
cargo build --offline --locked --manifest-path .\tools\yetlet-cli\Cargo.toml
$YetletFromCheckout = Join-Path (Get-Location) 'tools\yetlet-cli\target\debug\yetlet.exe'
& $YetletFromCheckout --help
```

The CLI owns native startup and process cleanup. Vite independently owns
frontend compilation and HMR. Frontend JS/TS files may use custom names and
subdirectories while physical root confinement rejects private paths and
outside links. The default entry is `app.ts`.
[Historical integration evidence](https://github.com/bohewu/yetlet/blob/ed7a082b1f9f72430f06866f7d550b0ed9ecbae9/docs/YT-102-STATEFUL-HMR-IMPLEMENTATION.md)
is distinct from this Native-only MSVC result. Historical Windows GNU pair
acceptance covers six separate Rust/Vite cases; the older MSVC pair
qualification is compile-only. Historical single-Cargo MSVC evidence covers
native cancellation and process ownership without proving this full
newly published CLI 0.4.1 workflow.

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
See the [selected contract](https://github.com/bohewu/yetlet/blob/master/docs/CLI-CONTRACT.md#read-only-project-inspection--yt-111).

Exit 0 means settings-derived intentions were explained; unsupported/skipped
stages do not make inspection fail. No tools, frontend directories/assets,
TLS files, secrets, runtime startup or readiness are verified. Inspection starts
no children, writes no output/settings, makes no network requests, reads no
Development secret files and calls no SecretProvider. It never dumps settings,
ENV, tokens or resolved absolute roots. Invalid selection/settings uses exit 1,
empty stdout and safe stderr, including existing source/profile/values hints;
`--json` does not alter error formatting. Usage failures use exit 2.

The [inspection record](https://github.com/bohewu/yetlet/blob/ed7a082b1f9f72430f06866f7d550b0ed9ecbae9/docs/YT-111-IMPLEMENTATION.md) retains its
source-bound acceptance evidence. The separate
[browser-loop record](https://github.com/bohewu/yetlet/blob/ed7a082b1f9f72430f06866f7d550b0ed9ecbae9/docs/YT-112-IMPLEMENTATION.md) records completed Linux
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
[research and bounded implementation evidence](https://github.com/bohewu/yetlet/blob/ed7a082b1f9f72430f06866f7d550b0ed9ecbae9/docs/YT-102-SETTINGS-ERROR-RESEARCH.md#selected-cli-implementation-2026-10-20).

Doctor reports schema 1 environment checks only, never application, asset,
frontend, console, generator or release readiness. Fixed local Rust/Cargo probes
have 3-second/64-KiB combined-output bounds plus 2-second cleanup. Unsafe or
unavailable executables fail; group-writable toolchain paths are refused on
Linux. No installation/update, shell, app settings or Node probes occur.

Linux has current native process/PTY/cleanup tests. Current Windows GNU
[doctor and CLI evidence](https://github.com/bohewu/yetlet/blob/ed7a082b1f9f72430f06866f7d550b0ed9ecbae9/docs/YT-102-DOCTOR-VERIFICATION.md) and
[Rust/Vite pair evidence](QUALIFICATION-MATRIX.md#historical-windows-native-browser-and-cli-evidence) supplement
the separate MSVC compile-only pair checks and
[native single-Cargo followup](QUALIFICATION-MATRIX.md#historical-cli-contract-process-and-sdk-resolver-evidence).
Windows run still requires an attached console
with processed Ctrl-C input and protected app/tool paths; npm uses vetted native
node.exe plus its same-install npm CLI script, without a .cmd shell fallback.
Escaped daemons and unsupported OS/filesystem/console contexts remain unsupported.
Platform evidence applies to the tested configurations. See the
[release records](RELEASE-READINESS.md) for publication and qualification status.

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
are not automatically watched. The [selected amendment](https://github.com/bohewu/yetlet/blob/ed7a082b1f9f72430f06866f7d550b0ed9ecbae9/docs/YT-112-IMPLEMENTATION.md)
records completed Linux browser/lifecycle acceptance and retains the original
failed attempts. New Windows browser-watch execution remains Not verified;
historical YT-104 qualification describes the earlier static starter scope.

<a id="optional-application-owned-rust-css-yt-239-working-tree"></a>
## Application-owned Rust CSS

Application-owned Rust CSS first shipped in CLI `0.3.0` and remains available
in CLI `0.4.2`. The opt-in Style templates generate external CSS at build time
through a private application tool. The current Reader Edition selects SDK
`0.9.0-alpha.9` and CLI `0.4.2`. App SDK selection follows
[SDK resolution](#sdk-resolution-yt-247).

`new --template native-style` and `new --template browser-style` opt into a private
std-only Cargo generator with a complete lock, useful layout/dialog composition,
shared View class names and stylesheet connection. Their `styles::stylesheet()`
entry point already registers the `layout` and `dialog` modules, including a
conditional-only class even when its View is initially absent. To add another
module, explicitly declare it in the private Style registry and compose its
`styles()` output; merely creating a Rust file never discovers or registers it.
Keep target-neutral class constants shared by Views and the private generator.
Module/class ownership is unique, deterministic and checked before CSS is emitted.
Ordinary templates keep ordinary CSS; unsupported selector/value syntax stays
in regular CSS rather than bypassing the finite typed vocabulary. Applicable
Tailwind/Vite options compose with these choices. Creation runs Cargo lock
generation, preserves the separate Style lock and executes no generator.

The optional strict `[cli.styles]` section has `enabled`, `manifest`, `bin` and
`output`; absence means no style effects. The finite manifest is
`stylegen/Cargo.toml`, with a single explicit `main.rs` bin. Standalone output is
`assets/generated/app.generated.css`; Vite output is
`<cli.vite.root>/style-output/app.generated.css`. The existing Loader owns
base/profile/APP_CLI overrides; paths stay app-relative. Generated runtime settings
validate the same optional section. Inspect reports the style stages without tools.

Native build generates before frontend/app stages and accepts a full SHA-256 CSS
asset only after their success. Run generates before startup. Native watch repeats
the serialized CSS/app sequence and discards obsolete candidates. Browser run only
previews prepared dist; browser watch uses the existing gated Node loop and Rust
CSS stages before WASM. Vite owns its normal CSS graph/HMR and bundle hashes.
Standalone Tailwind links precede generated CSS; layers/specificity still govern
cascade precedence. Source/policy edits require the documented watch/restart rules.

The official path adds no Python dependency, public style crate or renderer/CSP API.
Output ownership conflicts and post-CSS metadata failures require explicit operator
reconciliation. Private `owner.pending.json`/`production.pending.json` intents retain
prior accepted metadata; producers and the generated native loader refuse unfinished
state even when hashes match. Marker removal commits acceptance after all fallible
writes/checks. Preserve evidence and explicitly verify rollback or rollforward
before clearing a retained marker; ordinary retry never performs reconciliation.
Prepublication failures preserve accepted data and owned temporaries are cleaned;
initially unknown temporaries remain untouched. No automatic install, retention,
asset cleanup or deployment is supplied.
The selected Cargo cache must satisfy protected filesystem ownership/write policy;
host output uses its own `yetlet-styles/<app-root-sha256>` subtree. See the
[original source implementation and independent review](https://github.com/bohewu/yetlet/blob/e49657922bcdfe238cd1f79061178a32a4622a01/docs/YT-239-STYLE-INTEGRATION.md#independent-review-and-completion).
The earlier [YT-238 local prototype](https://github.com/bohewu/yetlet/blob/e49657922bcdfe238cd1f79061178a32a4622a01/docs/YT-238-STYLE-BUILDER-PROOF.md#independent-review-and-completion)
selected the private builder and default module registration but was not
itself a published CLI or public Style API.

### Style release and platform evidence

| CLI | Linux installed registry consumer | Windows x64 MSVC |
|---|---|---|
| `0.3.0` | Accepted supplement passed Native Style SSR/CSS and Browser Style Vite/Wasm/Chrome with SDK alpha.7. The original publisher's consumer preflight failure is retained. [Publication record](CLI-030-HOSTED-RECORD.md#owner-approved-publication-and-installed-consumer--2026-11-10) | New Style-template revalidation remained unverified. [Publication record](CLI-030-HOSTED-RECORD.md#owner-approved-publication-and-installed-consumer--2026-11-10) |
| `0.3.1` | Hosted consumer passed Native and Browser Style runtime with SDK alpha.7. [Publication record](CLI-031-HOSTED-RECORD.md#owner-approved-publication-and-installed-consumer--2026-11-11) | Corrected source passed Native SSR/CSS, Browser Vite/Wasm/Chrome and immediate port closure after Ctrl-Break for both apps. The reviewed formatting-only delta and package member comparison bind the tested source to the accepted archive. [Windows preparation](CLI-031-RELEASE-PREPARATION.md), [hosted archive record](CLI-031-HOSTED-RECORD.md#windows-corrected-replacement--2026-11-11) |
| `0.4.0` | Hosted consumer passed Native Style HTTP/CSS and Browser Style Vite/Wasm/Chrome with exact SDK alpha.7. The separate alpha.8 first-app runtime check used plain Native. [Publication record](CLI-040-HOSTED-RECORD.md) | The preparation archive passed the locked SDK alpha.8 Native Style compile check, including SDK and startup-profile selection and lock preservation. Windows Style runtime evidence for this release remains unverified. [Qualification record](CLI-040-HOSTED-RECORD.md) |
| `0.4.1` | Published [CLI 0.4.1 record](CLI-041-HOSTED-RECORD.md#owner-approved-publication-and-installed-consumer--2026-11-17): registry-only CLI install, Native Style HTTP/CSS and Browser Style actual Chrome/WASM/CSS PASS, SDK alpha.8; package SHA read back from crates.io and GitHub | Original source-level Windows MSVC Ctrl+C regression and same-run OS-process cleanup accepted in PR #97; no new installed 0.4.1 Windows runtime matrix claimed |

YT-243 records source qualification, including Linux release-profile browser
checks and local Windows MSVC Native Style HTTP/CSS, Browser Style scaffold
creation and wasm32 compile checks. Its Windows browser scope stops at
compilation. [YT-243 source qualification](QUALIFICATION-MATRIX.md#historical-two-package-migration-and-style-source-qualification)

Todo and the native-first Style Showcase have their own bounded Linux
Chrome/Firefox sample results. Showcase runs from a checkout and has an optional
Wasm companion. The [Todo sample's CSS instructions](../examples/todo-server/README.md#app-owned-rust-styles-and-typed-tag-builders)
retain current usage; [its original bounded Linux qualification](https://github.com/bohewu/yetlet/blob/e49657922bcdfe238cd1f79061178a32a4622a01/docs/YT-240-TODO-STYLE-QUALIFICATION.md#platform-matrix-and-acceptance-boundary)
retains observed browser results and limits. [Style Showcase](../examples/style-showcase/README.md)

## Development HTTPS

Use the existing public `server.tls` settings in a settings-enabled app with
Development selected. Prepare loopback certificates and trust explicitly through
[the development guide](DEVELOPMENT.md#opt-in-local-https-preparation-and-lifecycle).
`yetlet build` can compile with certificate directories absent. `yetlet run`
and `watch` require valid prepared material and select optional yetlet-dev-tls.
Automatic Vite startup owns HTTPS/WSS5173 and a private HTTP Rust backend3000.
With automatic Vite disabled Rust serves HTTPS directly. Preserve bridge
Host/Origin/path/CSP and native owned-process cancellation. The
[Linux and MSVC results](QUALIFICATION-MATRIX.md#development-https-evidence-and-original-failures)
separate actual TLS/console checks, failed wrappers and later cleanup.

## Protected Windows tool preparation

The executable trust check can return125 before creating any child. Choose an
owner-protected physical directory, verify the expected tool hash, and select
its absolute directory on process-local PATH. Prepare the corresponding native
Visual Studio and OpenSSL environment, explicit Cargo cache and target roots.
Use an attached console with processed input for run. Preserve the global tool
installation and permission policy.

The historical retained Cargo SHA256 was
`6f4bef66261261fcb43131be8720bab817d403a09edec7455c371974b90bdb7e` and
source-matched CLI SHA256 was
`1cccc6815f63ce8e3318aefa3ce7dbb550b1015694a2725f5b32e326cf0034a9`.
These identify the 2026-10-18 fixture only. The
[original process-local PowerShell setup](https://github.com/bohewu/yetlet/blob/c0a40af80a48932f3cdf7c593f36fe563e4ec669/docs/WINDOWS-CLI-125-VERIFICATION.md#reuse-the-prepared-runner-tools)
is locally recoverable for that owned root. Current installations need their
own identity and permissions checks. A copy under ordinary inherited unsafe
ACLs remains refused. [Native results and original failures](QUALIFICATION-MATRIX.md#windows-exit-125-diagnosis-and-native-results)
retain the MSVC/GNU and console scope of each observation.
