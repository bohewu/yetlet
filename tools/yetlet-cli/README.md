# Yetlet CLI — Unpublished Linux/Windows Developer Preview

The `yetlet` CLI is an **experimental, local-source-only** companion. It is not published to a registry, globally installed, or production-qualified. [YT-087](../../docs/YT-087C-VERIFICATION.md) records historical Linux acceptance; [YT-088](../../docs/YT-088-CROSS-PLATFORM-CLI-PLAN.md) records the later native Windows parity implementation and its bounded validation. YT-020 release gates remain blocked.

## Build locally

From the Yetlet checkout, using its already installed Rust/Cargo 1.99 toolchain and cached dependencies:

```sh
cargo build --offline --locked --manifest-path tools/yetlet-cli/Cargo.toml
tools/yetlet-cli/target/debug/yetlet doctor --framework-path . --json
```

The CLI is in its own unpublished Cargo workspace and has its own Cargo.lock. Its build does not join or change the framework or `cargo xtask` dependency graph.

## Linux — scaffold and run native SSR

Create or select a **protected existing parent directory** (owned by you, non-group/world-writable; `new` also permits trusted sticky parents on Linux). This sample uses a private parent and preserves the current application's local framework dependency paths:

```sh
mkdir -p -m 700 "$HOME/yetlet-apps"
tools/yetlet-cli/target/debug/yetlet new sampleapp --framework-path . --dir "$HOME/yetlet-apps"
cargo generate-lockfile --offline --manifest-path "$HOME/yetlet-apps/sampleapp/Cargo.toml"
tools/yetlet-cli/target/debug/yetlet build --manifest-path "$HOME/yetlet-apps/sampleapp/Cargo.toml"
tools/yetlet-cli/target/debug/yetlet dev --manifest-path "$HOME/yetlet-apps/sampleapp/Cargo.toml"
```

Open `http://127.0.0.1:3000/` and stop with Ctrl-C. The generated application is native, handler-first SSR: no mandatory JS/WASM, Controllers, DI, Session, HTTP event circuit or watcher. `YETLET_PORT` selects another loopback port. `new` never creates an application lockfile or builds/downloads dependencies; **you** run the explicit offline `cargo generate-lockfile` step. If dependencies are absent from the local cache, it fails rather than falling back to the network. Keep the application's own Cargo.lock for future frozen commands.

### Opt-in native SSR cloud profile (YT-089a)

Append `--cloud` to `new NAME --framework-path PATH [--dir DIRECTORY]` to select
the separate app-owned cloud template. It still defaults to `127.0.0.1:3000`.
Only explicit `YETLET_HOST=0.0.0.0` permits external listening; the other accepted
host literal is `127.0.0.1`. `YETLET_PORT` accepts decimal ports 1 through 65535.
Empty, non-Unicode or invalid settings fail startup without echoing their values.
The default template and its generated README are unchanged.

The cloud app alone enables Tokio's signal feature and supplies empty-body
`GET /health/live` (200) and `/health/ready` (200, or 503 after shutdown starts).
These report local-process state, not external DB/Redis health. Unix SIGTERM or
portable Ctrl-C makes readiness unavailable and initiates Axum graceful drain.
After the listener closes, probes may be refused rather than receiving 503.
Long requests/streams have no automatic drain deadline; the application's later
deployment `terminationGracePeriodSeconds` budget and traffic withdrawal policy
must account for them. Run production drain checks against the app binary;
the CLI `dev` supervisor retains its separate bounded cleanup behavior.

The cloud manifest declares `[package.metadata.yetlet] cloud-profile =
"native-ssr-v1"` for future generators; this declaration is not source attestation.
The Docker generator below verifies the parsed source as well as this marker.
See the generated cloud README and [accepted YT-089a evidence](../../docs/YT-089A-VERIFICATION.md)
for its bounded Linux runtime and native Windows compile/build/unit gates.

### Opt-in typed app configuration (YT-097; independently reviewed)

```sh
yetlet new configured --framework-path /absolute/path/to/yetlet --dir /existing/protected/apps --with-config
# Add --cloud for local health and graceful shutdown in the separate cloud app.
```

Both profiles generate root `base.toml`, `development.toml`, `staging.toml`,
`production.toml`, synthetic `.env.example` and app-owned `src/config.rs` with
strongly typed Deserialize settings, SecretString and native Axum State/Arc.
The standalone manifest explicitly selects the unpublished local `yetlet-config`
crate, versioned Serde derive and Secrecy serde. No configuration dependency is
added to the CLI/core/default framework. Existing `create`/`create_cloud` APIs
and files without `--with-config` retain their original bytes. Duplicate or
misplaced flags fail usage parsing.

The generated README covers explicit offline lock/check/test commands, injected
`APP_SERVICE__TOKEN`, finite `APP_ENVIRONMENT` selection (Production default),
`APP_SERVER__HOST`/`PORT`, PowerShell and optional absolute `YETLET_DEV_SECRETS`
selection in Development only. Nothing auto-loads `.env.local`. Root files resolve
from build-time CARGO_MANIFEST_DIR, not runtime cwd; relocation needs deliberate
file provisioning/rebuild. Base and profile TOML are nonsecret. Missing/blank
required token or invalid/unknown config fails before listening with fixed errors.
Strings such as `000123` retain their shape. Only public data enters escaped SSR.

**`--cloud --with-config` is a separate application contract and is rejected by
existing `generate docker/compose/k8s`.** Their exact-source/fixed-dependency
allowlists are unchanged, including the original `new --cloud` contract. A
separate reviewed delivery design is required before container generators can
accept configured apps. [YT-097 evidence](../../docs/YT-097-VERIFICATION.md)
records actual Linux/Windows native verification and independent Reviewer PASS.

### Opt-in Tailwind CSS 4 (YT-091; independently reviewed)

```sh
yetlet new styled --framework-path /absolute/path/to/yetlet --dir /existing/protected/apps --with-tailwind
# --cloud, --with-config, or both can also be selected.
```

All four opt-in combinations generate root `tailwind.css` and fixed
`src/tailwind.rs`, while retaining the selected native/cloud/config contract.
Applications without `--with-tailwind` retain exactly their established source
and configuration bytes. The generator invokes no tools and creates no output
CSS, lockfile or Node dependencies. Duplicate, unknown and misplaced flags fail
closed. The original native safe no-replace writer publishes every file.

Select your own digest-verified official standalone Tailwind **v4.3.3** binary;
the [generated README](templates/tailwind/README.md) pins Linux/Windows x64
digests and gives shell/PowerShell commands. From the generated app root,
explicitly run `TAILWIND_BINARY -i ./tailwind.css -o ./dist/tailwind.css --minify`
before Cargo lock/check/test/run. The CSS input uses `source(none)` and
`@source "./src"`; complete literal Rust utility strings are scanned without
a `tailwind.config.js`. Rebuild CSS and restart after edits. The example uses
system fonts, with no external CSS/JS/font requests, watch mode or HMR.

Startup validates a bounded, fixed app-root compiled CSS file and pinned banner
before listening. App-owned Axum code snapshots its bytes, computes SHA-256 and
serves only the matching same-origin digest URL with CSS MIME, `nosniff` and
immutable caching. Escaped SSR links that asset under unchanged `DEFAULT_CSP`;
no core Page/rendering API or browser dependency changes. The banner is a sanity
check, not a signature or CSS sanitizer; the operator owns trusted build input.
Runtime relocation and old-asset retention limitations are in the generated
README. No CLI or HTTP path implicitly compiles/downloads CSS.

Existing `generate docker/compose/k8s` reject these source variants, including
cloud combinations. [YT-091 evidence](../../docs/YT-091-VERIFICATION.md) separates
generated-app and socketless HTTP tests from actual Linux HTTP/CSP/health/
signal, Windows GNU/MSVC and independent Reviewer PASS evidence; browser
rendering and production qualification remain unverified. The canonical 12-sample catalog
and YT-020 release gate are unchanged.

### Opt-in Vite production assets (YT-099; independently accepted experimental scope)

```sh
yetlet new webapp --framework-path /absolute/path/to/yetlet --dir /existing/protected/apps --assets vite
# Add --with-tailwind for @tailwindcss/vite; --cloud and --with-config compose.
```

Only the exact value `vite` is accepted. Duplicate, unknown and misplaced flags
fail closed. Without `--assets`, all eight native/cloud/config/standalone-Tailwind
combinations preserve their existing parse results, file inventories and bytes.
The existing scaffold APIs are unchanged; `scaffold::create_vite` is additive.
No Node, Vite or frontend dependency enters the CLI, Core or browser defaults.

This production profile writes a finite set of root TypeScript/CSS/SVG, Vite
configuration, tsconfig and pinned `package.json`/`package-lock.json` files, plus
app-owned `src/vite.rs`. No frontend/dist directory is created by scaffolding.
The native Linux/Windows no-replace/no-symlink writer still supports only the
existing single child directory; its source allowlist adds exactly `src/vite.rs`.
The root frontend package pins Vite 8.3.4 and TypeScript 7.0.2. Plain Vite has no
Tailwind dependency. `--assets vite --with-tailwind` selects Tailwind and the
official Vite plugin at 4.3.3, with no standalone compiler or second CSS build.
The CLI never generates/resolves a lock, runs npm, accesses a registry, compiles
assets, loads dotenv files or starts a Node process.

From the generated app root, explicitly run:

```sh
npm ci --ignore-scripts --no-audit --no-fund
npm run typecheck
npm run build
cargo generate-lockfile --offline
cargo test --offline --locked
export YETLET_ASSET_ROOT="$PWD/dist"
# Set APP_SERVICE__TOKEN and any typed config overrides if selected.
cargo run --offline --locked
```

The generated README includes full fmt/check/Clippy and Windows commands.
Startup requires an explicit absolute, trusted and quiescent asset root; missing
or invalid build output fails before listening. The bounded loader snapshots
manifest-whitelisted JS/CSS/SVG once, uses native State/Arc, preserves escaped
SSR and strict DEFAULT_CSP, and serves same-origin `/static/` bytes with fixed
MIME, nosniff and no-store. Static dependency-first CSS is deduplicated; lazy CSS
loads only after interaction. There is no dotenv/public directory copying,
asset inlining, source map or dev-only HMR injection. Keep secrets out of
`VITE_*` and frontend sources. Config retains SecretString validation and cloud
retains local bind/health/graceful shutdown semantics.

The adapter remains application-owned, unpublished and experimental; YT-098 did
not establish a shared stable assets crate. No-store does not retain old build
hashes for rolling deployments. Existing Docker/Compose/Kubernetes generators
reject this source contract until YT-101; HMR belongs to YT-100. See
[YT-099 verification and qualification limits](../../docs/YT-099-VERIFICATION.md).

### Opt-in Docker configuration (YT-089b; independently accepted)

```text
yetlet generate docker --manifest-path APP/Cargo.toml --output NEW_DIRECTORY
```

The output targets Linux OCI/amd64 on Linux or Windows. It requires an existing
app Cargo.lock and the exact generated standalone `--cloud` manifest and parsed
main source contract. Edited/manual apps, extra sources/dependencies/features,
build scripts, unknown framework path packages and unsupported workspace
inheritance fail closed. Output uses the native safe no-replace adapter; its
parent must exist and be protected. Paths support spaces/Unicode within the
platform contract. No Cargo, Docker, network, source copying or source edits
occur during generation. The public three-argument scaffold API is retained.

The eight generated files are Dockerfile, a deny-by-default root `.dockerignore`,
`cargo-overlay/Cargo.toml`, `framework-workspace.toml`, two named-context ignore
templates, a nonsecret SHA-256 inventory and README. Follow that README to
manually prepare and verify two **physically separate sanitized** `app-src` and
`yetlet-src` exports. The output directory is the only default context. Never
send the live app or dirty checkout as a context or assume the output ignore
file sanitizes named sources. No automatic exporter is provided.

The app overlay changes exactly two path values and preserves other supported
manifest semantics. The framework workspace template retains the exact supported
inherited metadata/dependencies and only core/HTML/server-axum members. Docker
uses a Rust 1.99.0 Bookworm GNU builder and Debian Bookworm slim non-root runtime,
explicit `YETLET_HOST=0.0.0.0`, `YETLET_PORT=8080`, exec entrypoint and SIGTERM.
No unprovided healthcheck executable is assumed. The separate operator build
runs `cargo metadata --locked` and `cargo build --locked --release`; registry
crates/base images/frontend can require network. Tags are versioned, mutable,
and require a later verified digest-pin decision. See [accepted YT-089b
evidence](../../docs/YT-089B-VERIFICATION.md). The separately reviewed Kubernetes configuration generator is documented below.

### Opt-in Compose configuration (YT-089c independently accepted)

```text
yetlet generate compose --manifest-path APP/Cargo.toml --output NEW_DIRECTORY --image localhost:5000/team/app:v1
```

Generation creates exactly `compose.yaml` and `README.md` in a fresh directory
under an existing protected parent. It reuses the Docker generator's unchanged
cloud-app/source/framework inspection, revalidation and native no-replace writer.
An explicit Cargo.lock v4 must match the supported app/framework root packages,
direct dependency identities and complete referenced lock nodes; stale/missing
locks fail closed. Static checks do not replace the operator's locked image build.
No Cargo, Docker, Compose, shell, network, registry or kubectl is invoked.

The shared internal image validator requires lowercase ASCII registry/repository
and an explicit non-`latest` tag (ASCII case allowed in tags) or `@sha256:` with
64 lowercase hex digits. A registry must be a dotted DNS/IPv4 host or `localhost`,
optionally with a decimal port 1-65535 without leading zeros. Short/unqualified
names, IPv6, tag-plus-digest, blanks, controls, Unicode, YAML/shell interpolation
and escapes are refused. Missing/duplicate/unknown/invalid args use usage exit 2;
unsafe inputs/output or stale sources use exit 1.

Compose references a separately operator-built Linux/amd64 image, with
`pull_policy: never`, container `0.0.0.0:8080`, host-only `127.0.0.1:8080`,
SIGTERM and a 30-second stop grace period. The app has no automatic drain deadline;
adjust this example budget for measured long requests. No build, volumes,
privileges, credentials or healthcheck tool is assumed. The generated README
describes explicit operator config/up/down actions and local digest availability.
The generator neither reads `.env` nor embeds personal paths or secret values.

[YT-089c verification](../../docs/YT-089C-VERIFICATION.md) records independent Reviewer PASS after native GNU/MSVC byte parity and isolated Linux Docker Compose HTTP/SIGTERM/cleanup; generator invocation itself never builds or deploys.

## Opt-in Kubernetes configuration (YT-089d independently accepted)

```sh
yetlet generate k8s --manifest-path APP/Cargo.toml --output NEW_DIRECTORY --image localhost:5000/team/app:v1 --replicas 1
```

Linux and Windows generate Linux/amd64 Deployment and internal ClusterIP Service
configuration only: `deployment.yaml`, `service.yaml` and `README.md`. The files
use JSON syntax within YAML 1.2, with exact typed scalar/image preservation.
Only the accepted unedited cloud profile and an explicit consistent Cargo.lock
are supported; inspection, source revalidation and native no-replace publication
are shared with the accepted generators. No tool, daemon or cluster is invoked.

`--replicas` is optional, unsigned ASCII decimal 1-100, default 1. Package names
must be unchanged DNS labels of 1-63 lowercase letters/digits/hyphens, with a
letter prefix and letter/digit suffix; underscores and translation are refused.
The generated README covers the separate sanitized BuildKit image build,
operator image import/publication, explicitly selected context/namespace dry-run
and apply commands, local-only probes, example resource limits and the
30-second grace budget without a hard application drain promise.

[YT-089d accepted evidence](../../docs/YT-089D-VERIFICATION.md) records independent Reviewer `wc_job_1SJSI37ZToF_od0U` PASS after Linux manifest schema checks, exact Windows GNU/MSVC native output parity and parent-operated, isolated Multipass K3s Pod/Service/HTTP/health and full namespace/image cleanup. The generator itself never invokes Kubernetes or deploys anything.

## Windows PowerShell (native GNU/MSVC)

Build the isolated CLI using the selected Rust 1.99 Windows toolchain. On Windows, choose an **existing, protected directory on a local fixed NTFS volume** and a trusted, owner-protected `cargo.exe` on PATH before `build` or `dev`. The CLI deliberately refuses unsafe ACLs, junction/reparse traversal and untrusted executables.

```powershell
cargo build --offline --locked --manifest-path .\tools\yetlet-cli\Cargo.toml
$Yetlet = Join-Path (Get-Location) "tools\yetlet-cli\target\debug\yetlet.exe"
$Parent = Join-Path $env:TEMP "yetlet-apps"
New-Item -ItemType Directory -Force -Path $Parent | Out-Null
& $Yetlet doctor --framework-path . --json
& $Yetlet new sampleapp --framework-path . --dir $Parent
$Manifest = Join-Path $Parent "sampleapp\Cargo.toml"
cargo generate-lockfile --offline --manifest-path $Manifest
& $Yetlet build --manifest-path $Manifest
& $Yetlet dev --manifest-path $Manifest
```

If the chosen parent inherited unsafe ACLs, or Cargo's rustup proxy is writable by other principals, the CLI fails closed. Provision a protected NTFS parent or an owner-protected copy of `cargo.exe` and select it through the current session's PATH; do not disable security checks. `dev` requires an attached, processed-input Windows console and uses its own Job Object for Cargo and application descendants. For GNU, some MinGW linkers cannot emit executable files into all Unicode output directories; select a compatible, task-owned ASCII `CARGO_TARGET_DIR` if needed.

## Commands and boundaries

| Command | Behavior |
|---|---|
| `doctor [--framework-path PATH] [--json]` | Sanitized local Rust/Cargo and optional selected framework-source diagnostics; no installation or app launch |
| `new NAME --framework-path PATH [--dir DIRECTORY] [--cloud] [--with-config]` | Linux safe directory-FD/no-replace or Windows local NTFS handle/reparse/ACL-safe no-overwrite scaffold; cloud app profile is explicit opt-in |
| `build --manifest-path PATH [--release]` | Explicit external standalone app, existing lockfile, native `cargo build --frozen`; no implicit resolution |
| `generate docker --manifest-path PATH --output DIRECTORY` | New protected directory containing Linux/amd64 configuration only; operator prepares and verifies separate named source contexts |
| `generate compose --manifest-path PATH --output DIRECTORY --image IMAGE` | Candidate: two deterministic files referencing an explicit operator-built Linux/amd64 image, localhost port only; no build/pull/launch |
| `generate k8s --manifest-path PATH --output DIRECTORY --image IMAGE [--replicas COUNT]` | Candidate: three deterministic configuration files for an operator-built Linux/amd64 image; no cluster discovery, credentials or deployment |
| `dev --manifest-path PATH` | Foreground native `cargo run --frozen` with owned Cargo/app process-group teardown and terminal Ctrl-C handling; **not** HMR/watch mode |

The supported preview OS targets are Linux and Windows x86-64 (GNU/MSVC), each with independently checked native filesystem and process contracts. Other platforms and incompatible/unsafe filesystem or console contexts fail closed. Windows uses a protected local fixed NTFS volume, owner-protected app/Cargo executable and a real console for `dev`; Linux preserves symlink-safe directory FDs and process groups. `dev` does not contain intentionally daemonized/session-escaped children. Path safety, error classification, cleanup and terminal handling are covered by the targeted tests, but this is not an isolation sandbox or deployment agent.

No registry publication, release signature, global installation, hosted operation or release activation is authorized. Prior YT-087a/b tests and actual loopback/PTY receipts are source-bound and separate from YT-020 release readiness.
