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
| `new NAME --framework-path PATH [--dir DIRECTORY] [--cloud]` | Linux safe directory-FD/no-replace or Windows local NTFS handle/reparse/ACL-safe no-overwrite scaffold; cloud app profile is explicit opt-in |
| `build --manifest-path PATH [--release]` | Explicit external standalone app, existing lockfile, native `cargo build --frozen`; no implicit resolution |
| `generate docker --manifest-path PATH --output DIRECTORY` | New protected directory containing Linux/amd64 configuration only; operator prepares and verifies separate named source contexts |
| `generate compose --manifest-path PATH --output DIRECTORY --image IMAGE` | Candidate: two deterministic files referencing an explicit operator-built Linux/amd64 image, localhost port only; no build/pull/launch |
| `generate k8s --manifest-path PATH --output DIRECTORY --image IMAGE [--replicas COUNT]` | Candidate: three deterministic configuration files for an operator-built Linux/amd64 image; no cluster discovery, credentials or deployment |
| `dev --manifest-path PATH` | Foreground native `cargo run --frozen` with owned Cargo/app process-group teardown and terminal Ctrl-C handling; **not** HMR/watch mode |

The supported preview OS targets are Linux and Windows x86-64 (GNU/MSVC), each with independently checked native filesystem and process contracts. Other platforms and incompatible/unsafe filesystem or console contexts fail closed. Windows uses a protected local fixed NTFS volume, owner-protected app/Cargo executable and a real console for `dev`; Linux preserves symlink-safe directory FDs and process groups. `dev` does not contain intentionally daemonized/session-escaped children. Path safety, error classification, cleanup and terminal handling are covered by the targeted tests, but this is not an isolation sandbox or deployment agent.

No registry publication, release signature, global installation, hosted operation or release activation is authorized. Prior YT-087a/b tests and actual loopback/PTY receipts are source-bound and separate from YT-020 release readiness.
