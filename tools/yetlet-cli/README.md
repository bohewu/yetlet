# Yetlet CLI

`yetlet-cli` installs the experimental `yetlet` command for scaffolding,
building and running [Yetlet](https://github.com/bohewu/yetlet) applications.

## Install

Prepare Rust/Cargo 1.99 or later, a native C toolchain and OpenSSL development
files. Run the registry install once the selected version is available.

```sh
cargo +1.99.0 install yetlet-cli --version 0.3.1 --locked
yetlet --version
```

`yetlet --version` reports exactly
`yetlet 0.3.1`.

## Create a native app

Work in an owned, protected directory with the installed Cargo and rustc on
PATH. The [first-app guide](https://bohewu.github.io/yetlet/first-app/) shows how
to select a physical toolchain and verify the HTTP response.

```sh
yetlet new myapp
cd myapp
cargo fetch --locked
yetlet run
```

## CLI command and capability inventory

- `yetlet new NAME` creates a native app by default. Use `--template` with
  `browser`, `native-style` or `browser-style` for the other starters.
- By default, online creation selects the latest non-yanked SDK by SemVer,
  including prereleases. Browser templates require the same version of
  `yetlet-browser`. Use `--sdk-version VERSION` for an exact selection, or
  `--offline` with prepared resolver metadata and Cargo caches. Keep the exact
  SDK pins and `Cargo.lock` generated for the app.
- `yetlet build`, `yetlet run` and `yetlet watch` follow the app's settings.
  `yetlet doctor` checks the Rust/Cargo environment, and `yetlet inspect`
  explains the app's execution plan. Use `yetlet --help` for command syntax.

See the [CLI guide](https://github.com/bohewu/yetlet/blob/master/docs/CLI-GUIDE.md)
for platform setup, settings and the full command reference, or follow the
[browser app guide](https://bohewu.github.io/yetlet/browser/) for a Wasm starter.
[GitHub releases](https://github.com/bohewu/yetlet/releases) record published versions.

**CLI release:** `0.3.1`. **Build SDK:** `=0.9.0-alpha.7`.

**SDK policy:** `sdk-latest-v2`.
