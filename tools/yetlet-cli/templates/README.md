# @NAME@

Unpublished local Yetlet SSR preview. Requires stable Rust/Cargo 1.99 or newer
and the explicitly selected local framework checkout. No JS/WASM, browser tools,
Controllers, DI or optional providers are needed. This is not production qualification.

From this project's parent directory, explicitly create this application's own
lockfile using already cached dependencies, then verify and run:

```sh
cargo generate-lockfile --offline --manifest-path @NAME@/Cargo.toml
cargo check --frozen --manifest-path @NAME@/Cargo.toml
cargo run --frozen --manifest-path @NAME@/Cargo.toml
```

`new` creates no Cargo.lock and runs no build or dependency resolution. The offline
lock step fails if the cache is insufficient; provision dependencies separately.
Keep the generated application Cargo.lock for subsequent frozen commands. Do not
copy the framework, xtask or CLI lockfile into this application.

Visit http://127.0.0.1:3000/ or http://127.0.0.1:3000/hello/World and stop with
Ctrl-C. `YETLET_PORT` optionally selects another port; the listener stays on
loopback. Axum serves native HTML via App/View/Page with Yetlet's default strict
CSP and text escaping. No JavaScript or WASM is loaded.

Cargo.toml contains explicit absolute local path dependencies on `yetlet-core`
and `yetlet-server-axum`, with independent package metadata. If moving the
framework checkout, edit both paths and explicitly refresh this app's lockfile
offline before the next frozen build. These crates are not fetched from a
registry or Git. With an already-built **unpublished Linux or Windows** Yetlet CLI binary,
you may instead run the frozen commands below after the explicit lockfile step:

```sh
YETLET_CLI=/path/to/your/yetlet-checkout/tools/yetlet-cli/target/debug/yetlet
"$YETLET_CLI" build --manifest-path @NAME@/Cargo.toml
"$YETLET_CLI" dev --manifest-path @NAME@/Cargo.toml
```

On Windows use an already-built native `yetlet.exe` with a protected local NTFS project and a trusted owner-protected `cargo.exe` on PATH:

```powershell
$YetletCli = "C:\path\to\your\yetlet-checkout\tools\yetlet-cli\target\debug\yetlet.exe"
& $YetletCli build --manifest-path "@NAME@\Cargo.toml"
& $YetletCli dev --manifest-path "@NAME@\Cargo.toml"
```

Windows `dev` requires an attached console with processed Ctrl-C input; its owned Job Object handles bounded application/descendant cleanup. On GNU a compatible ASCII `CARGO_TARGET_DIR` may be needed if the linker cannot emit an executable at a Unicode output location. No file watching or hot reload is provided.



The CLI does not install itself or download missing application dependencies.
`dev` is an owned foreground Cargo run, not file watching or hot reload; it
requires supported Linux terminal/process ownership or Windows protected native Job/console ownership. Unsupported operating systems and unsafe filesystems/executables fail closed.
