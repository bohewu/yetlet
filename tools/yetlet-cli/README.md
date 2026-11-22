# Yetlet CLI

`yetlet-cli` installs the experimental `yetlet` command for scaffolding,
building and running [Yetlet](https://github.com/bohewu/yetlet) applications.

## Install

Prepare Rust/Cargo 1.99 or later, a native C toolchain and OpenSSL development
files, then install the CLI from the registry.

```sh
cargo +1.99.0 install yetlet-cli --version 0.4.2 --locked
yetlet --version
```

`yetlet --version` reports exactly
`yetlet 0.4.2`.

## Create a native app

Work in an owned, protected directory with the installed Cargo and rustc on
PATH. See the [native build prerequisites](https://github.com/bohewu/yetlet/blob/master/docs/CLI-GUIDE.md#first-use-prerequisites)
for toolchain setup.

```sh
yetlet new myapp --sdk-version 0.9.0-alpha.8
cd myapp
cargo fetch --locked
yetlet run
```

Open `http://127.0.0.1:7878/` in your browser. Stop the server with Ctrl+C.

## CLI command and capability inventory

- `yetlet new NAME` creates a native app by default. Use `--template` with
  `browser`, `native-style` or `browser-style` for the other starters.
- By default, online creation selects the latest non-yanked SDK by SemVer,
  including prereleases. Browser templates require the same version of
  `yetlet-browser`. Use `--sdk-version VERSION` for an exact selection, or
  `--offline` with prepared resolver metadata and Cargo caches. Keep the exact
  SDK pins and `Cargo.lock` generated for the app.
- Native and Native Style projects on verified SDK `0.9.0-alpha.8` use
  `App::serve()` at `127.0.0.1:7878` when no Settings, Vite or Service startup
  is requested. Tailwind can use that same startup. SDK alpha.7 retains its
  explicit listener. Settings, Vite and Service keep native Axum lifecycle
  control, and Browser starters continue to emit Wasm entry points. The
  creation receipt records the selected startup profile.
- `yetlet build`, `yetlet run` and `yetlet watch` follow the app's settings.
  `yetlet doctor` checks the Rust/Cargo environment, and `yetlet inspect`
  explains the app's execution plan. Use `yetlet --help` for command syntax.

See the [CLI guide](https://github.com/bohewu/yetlet/blob/master/docs/CLI-GUIDE.md)
for platform setup, settings and the full command reference, or follow the
[browser app guide](https://bohewu.github.io/yetlet/browser/) for a Wasm starter.
[GitHub releases](https://github.com/bohewu/yetlet/releases) record published versions.

**CLI release:** `0.4.2`. **Build SDK:** `=0.9.0-alpha.8`.

The CLI builds against SDK alpha.8 and embeds the matching SDK snapshot for
its source exporters. Applications resolve their SDK independently.

## SDK feature profiles

The CLI recognizes SDKs that declare the optional `web` or `view` feature.

For those SDKs, Native starters explicitly select `web` and `html`, adding
`config` for Settings or Vite. They import `yetlet::web`; simple starters use
`App::serve()`, while Settings, Vite and Service retain their explicit listener
lifecycle. Browser starters explicitly select `reactive` and `view`, with the
companion's `dom` and `dom-keyed` features. Every generated SDK dependency keeps
`default-features = false`.

The existing receipt records `web-app-serve`, `web-configured` or `web-service`
for those Native profiles; Browser remains `browser-wasm`. Older SDKs retain
their established feature selections and startup profiles. Incomplete or
invalid selected feature metadata returns `SDK_INCOMPATIBLE` with upgrade or
explicit-pin guidance; it never silently selects an older SDK.

**SDK policy:** `sdk-latest-v2`.
