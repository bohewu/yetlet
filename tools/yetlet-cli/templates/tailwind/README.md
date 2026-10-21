# @NAME@ — opt-in Tailwind CSS 4 preview

Build the stylesheet **before running Cargo or starting this app**. This is an
explicit, offline build step. `yetlet new`, `build`, `dev` and HTTP requests
never run a CSS compiler, download a tool or create `dist/tailwind.css`.
No Node dependencies, browser compiler, CDN, JS or external font are needed.

## Select and verify the standalone tool

Use your own already-provisioned official standalone **Tailwind CSS v4.3.3**
executable. This scaffold does not fetch/install it. Choose the matching x64
asset from the [tagged official release](https://github.com/tailwindlabs/tailwindcss/releases/tag/v4.3.3)
and verify SHA-256 before execution:

| Asset | SHA-256 |
|---|---|
| `tailwindcss-linux-x64` | `dc61b3ac6b8c9ca874c0cc4c57b2409791a64c5540404ca5f5367360babc313a` |
| `tailwindcss-windows-x64.exe` | `e0e260ce048014e9268f6237ff18f8ccf02cef521cbd0ae04e82c2cdf7aa3955` |

From this app's root, with your selected absolute executable path:

```sh
TAILWIND_BIN=/path/to/verified/tailwindcss-linux-x64
sha256sum "$TAILWIND_BIN"
"$TAILWIND_BIN" --help # must identify v4.3.3
mkdir -p dist
"$TAILWIND_BIN" -i ./tailwind.css -o ./dist/tailwind.css --minify
cargo generate-lockfile --offline
cargo fmt --all -- --check
cargo check --offline --locked --all-targets
cargo test --offline --locked
cargo run --offline --locked
```

PowerShell, from the app root:

```powershell
$TailwindBin = 'C:\tools\tailwindcss-windows-x64.exe'
Get-FileHash -Algorithm SHA256 $TailwindBin
& $TailwindBin --help # must identify v4.3.3
New-Item -ItemType Directory -Force dist | Out-Null
& $TailwindBin -i ./tailwind.css -o ./dist/tailwind.css --minify
if ($LASTEXITCODE -ne 0) { throw 'Tailwind build failed' }
cargo generate-lockfile --offline
cargo check --offline --locked --all-targets
cargo test --offline --locked
cargo run --offline --locked
```

For `--with-config`, supply the required operator-selected configuration before
`cargo run`, following the retained configuration instructions below. No token
is needed to compile/test. All four native/cloud/config combinations support
the same CSS build command. Tailwind 4 does not require `tailwind.config.js`.

## Edit Rust, build CSS, restart

Root `tailwind.css` uses `@import "tailwindcss" source(none);` and
`@source "./src";`: scanning is explicitly restricted to app Rust sources.
Use complete literal strings, for example
`.attribute("class", "p-8 font-sans text-slate-900")`. Select whole literal variants
instead of concatenating fragments such as `text-` and a color.
See [official source detection](https://tailwindcss.com/docs/detecting-classes-in-source-files).
The sample uses system fonts. Do not add external CSS/font imports or runtime
Tailwind. Rebuild the CSS and restart the app after changing utility classes;
there is no watch mode or HMR.

At startup, `src/tailwind.rs` reads only the fixed build-time application-root
`dist/tailwind.css`, at most 1 MiB. Missing, empty, oversized, non-UTF-8 or
wrong-version output fails before binding. The exact pinned compiler banner
is an output sanity check, **not** authentication or a CSS sanitizer; build
assets only from trusted application sources using your digest-verified tool.
The application takes an immutable in-memory snapshot and computes SHA-256
over its actual bytes. It serves only `/assets/tailwind.<sha256>.css` with
`text/css; charset=utf-8`, `nosniff`, one-year immutable caching and strict
`DEFAULT_CSP`. Changing CSS bytes changes the URL after restart. HTML uses
`no-store`, an external same-origin link, escaped title/body and the unchanged
CSP, with no inline styles/scripts. Old digest URLs are not retained; rolling
deployment/asset retention needs a separate production design.

The runtime path is anchored to build-time `CARGO_MANIFEST_DIR`, independently
of cwd. Provision the compiled file at that app root; relocating requires a
rebuild. No output is embedded, fetched, compiled or updated by the server.
Existing Docker/Compose/Kubernetes generators reject this separate source
variant, including `--cloud`. This is an unpublished developer preview;
production, browser rendering and Windows runtime qualification require
separate evidence. YT-020 release remains Blocked.

## Retained application profile instructions
