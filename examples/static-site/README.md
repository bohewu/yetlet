# Static HTML site with an optional Wasm island

**Unreleased CLI source capability (YT-223).** Published CLI0.1.1 does not
implement build --static. This application uses registry SDK0.9.0-alpha.5 and
can be copied outside the repository without SDK source/path overrides.
See the [static export contract](../../docs/YT-222-STATIC-HTML-CONTRACT.md) and
[actual evidence and remaining platform gate](../../docs/YT-223-STATIC-HTML-IMPLEMENTATION.md).

## Prerequisites

Prepare Rust/Cargo1.99, a native linker and the app's explicit Cargo.lock/cache.
The provided lock is for this standalone [workspace]. Fetch missing dependencies
as a separate operation; the CLI itself uses frozen/offline Cargo and never
installs tools. HTML-only needs neither a server dependency nor Node/Wasm tools.
Build the unreleased CLI from this checkout using its existing native/OpenSSL
prerequisites:

```bash
cargo build --offline --locked --manifest-path tools/yetlet-cli/Cargo.toml --bin yetlet
```

Project/input directories must satisfy the existing protected-owner rules. On
POSIX use a private/non-group/world-writable project; on Windows use the current
user's protected ACL on a local fixed NTFS volume. The existing output parent
must be protected too. The commands below run from the repository root and set
demonstration choices inside a subshell.

## Minimal integration

```bash
(
    umask 077
    cd examples/static-site
    mkdir -p artifacts
    chmod go-w . src frontend public artifacts
    RUSTUP_TOOLCHAIN=1.99.0 ../../tools/yetlet-cli/target/debug/yetlet build \
        --static --export-bin site-export --output artifacts/site-v1 \
        --base-path /yetlet/ --release
)
```

The native exporter creates finite Views, renders escaped fragments and copies
explicit public assets to the CLI-owned private export directory. The CLI
validates protocol1 and wraps complete documents with bounded metadata and CSP.
It writes a new artifact directory; subsequent builds require another name.
There is no overwrite, clean, route crawler, request server or deployment.

## Expected result

Only public files appear in artifacts/site-v1/: index.html,
guide/install/index.html,404.html, assets/site.css, assets/diagram.png and an
empty .nojekyll. Host this directory at /yetlet/ on an HTTP(S) static server
supporting directory indexes. Opening /yetlet/guide/install/#steps directly or
refreshing it needs no SPA rewrite or application backend. Using --base-path /
instead prepares an origin-root deployment. No base element is emitted.

Main text/navigation remain readable with JavaScript disabled. The literal
`<safe>` and Unicode text demonstrate escaping; the local image/CSS remain normal
browser assets. The host chooses real404 behavior and MIME types; file:// is
not the verification target. Meta CSP does not supply frame-ancestors protection.
No output is uploaded and the existing public documentation site is unchanged.

## Configuration and errors

Existing public CLI settings validate first and cli.target must remain native;
this app needs no settings files or runtime Settings loader. Explicit --static
does not consult watch/Vite auto flags to select or skip its export stage.
--release selects Cargo's profile, not APP_ENVIRONMENT. No new settings keys are
introduced. See the [CLI guide](../../tools/yetlet-cli/README.md#unreleased-static-html-build).

Exit/error handling preserves owned child exit/cancellation after cleanup.
Invalid descriptors, unsafe paths/resources, source changes or an existing
destination cannot publish success. Private receipts under
target/yetlet-static/RUN_ID/ record selected input/output hashes and outcomes;
they never belong in the public artifact. Unknown/unsafe temporary contents can
be retained with a cleanup failure for inspection. Never remove another output
merely to make a build pass.

The exporter is trusted native code, not a sandbox. Classify build content as
public deliberately; do not load Auth/Session/business secrets into Views.
Source hashes cover selected src/manifest/lock/build.rs inputs, not every include,
path dependency, toolchain or external input that arbitrary Rust code can read.

### Separately selected island

After preparing wasm32-unknown-unknown and lock-matching wasm-bindgen-cli0.2.129,
use the existing browser build algorithm copied into frontend/build-wasm.mjs
(only the library/output name changes). Node orchestrates this explicitly
selected preparation; no npm package install, Vite build or second bundler is
performed by static export.

```bash
(
    cd examples/static-site
    # WASM_BINDGEN may explicitly select an already installed matching tool.
    RUSTUP_TOOLCHAIN=1.99.0 node frontend/build-wasm.mjs
    RUSTUP_TOOLCHAIN=1.99.0 ../../tools/yetlet-cli/target/debug/yetlet build \
        --static --export-bin site-export --output artifacts/site-island-v1 \
        --base-path /yetlet/ --wasm-islands
)
```

Serve that new directory at /yetlet/. Enable counter first loads the prepared
module/Wasm and mounts only its owned placeholder. Surrounding public HTML stays
readable, including when bootstrap/Wasm fails. Counter ready follows actual mount;
Increment changes local state without an HTTP action. Failure offers explicit
reload, with no automatic retry. dispose removes the listener; pagehide retires
the owner except when preserved by bfcache. No new bfcache qualification is claimed.

frontend/wasm-startup.mjs is a deliberate standalone copy of the canonical
[startup helper](../../assets/wasm-startup.mjs). Refresh it deliberately when
changing this consumer. The application owns the small bootstrap, styles and
wording; no SDK-wide loader or hydration behavior is added.

## API reference and limits

Use [View/HTML](../../crates/yetlet/guides/html.md), [DOM](../../crates/yetlet-browser/guides/dom.md)
and the [protocol/CLI contract](../../docs/YT-222-STATIC-HTML-CONTRACT.md).
New Cli::BuildStatic and build_static are Experimental support APIs; exhaustive
Rust enum matches may need another arm. Existing Build fields and commands stay
unchanged. No SDK version is republished by this sample. GNU/local-browser
evidence and the pending MSVC gate are reported separately in the implementation
record; current source is not a new platform/release qualification.
