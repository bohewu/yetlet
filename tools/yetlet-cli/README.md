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
| `new NAME --framework-path PATH [--dir DIRECTORY]` | Linux safe directory-FD/no-replace or Windows local NTFS handle/reparse/ACL-safe no-overwrite scaffold |
| `build --manifest-path PATH [--release]` | Explicit external standalone app, existing lockfile, native `cargo build --frozen`; no implicit resolution |
| `dev --manifest-path PATH` | Foreground native `cargo run --frozen` with owned Cargo/app process-group teardown and terminal Ctrl-C handling; **not** HMR/watch mode |

The supported preview OS targets are Linux and Windows x86-64 (GNU/MSVC), each with independently checked native filesystem and process contracts. Other platforms and incompatible/unsafe filesystem or console contexts fail closed. Windows uses a protected local fixed NTFS volume, owner-protected app/Cargo executable and a real console for `dev`; Linux preserves symlink-safe directory FDs and process groups. `dev` does not contain intentionally daemonized/session-escaped children. Path safety, error classification, cleanup and terminal handling are covered by the targeted tests, but this is not an isolation sandbox or deployment agent.

No registry publication, release signature, global installation, hosted operation or release activation is authorized. Prior YT-087a/b tests and actual loopback/PTY receipts are source-bound and separate from YT-020 release readiness.
