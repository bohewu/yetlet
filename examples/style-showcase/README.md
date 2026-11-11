# Yetlet Style Showcase — YT-241

**Status: Done at source and bounded Linux sample qualification scope.**
Independent implementation Reviewer `wc_job_SjBc9esAHAREdWS7` returned PASS
with zero material issues or required followups. The parent subsequently selected
this gallery as the thirteenth formal sample: native SSR requires no browser
runtime; the separate WASM companion remains an explicit optional enhancement.
It uses checkout-only typed Tag Builders and the private finite style helper,
not a published CSS API.

Run these exact commands from the repository root:

```sh
cargo build --locked -p yetlet-style-showcase
cargo run -p yetlet-style-showcase --locked
```

Open `http://127.0.0.1:3011/showcase`. Native generation, build and run need only
Rust/Cargo. The app-owned `build.rs` runs on the host and explicitly composes
`styles/mod.rs` using the existing tooling-local builder. It embeds one complete
stylesheet and its source identity into the executable. It does not invoke Cargo
recursively, probe browser tools, use Python/Node/Vite, or compile on requests.
Restart the server after rebuilding. `YETLET_SHOWCASE_LISTEN` overrides loopback
port 3011; `YETLET_SHOWCASE_DIST` selects optional browser output at startup.

Native startup wraps the existing Router with
`App::from_router(...).serve_on(...)`. Port `0` uses a native listener so the
startup banner reports the actual OS-selected address with the `/showcase` path.

The four areas demonstrate Layout (Flex, responsive Grid, spacing), Tokens
(complete light/dark colors and comfortable/compact density), Components (button,
card, badge and labeled native controls), and States (keyboard focus, native
disabled, text/semantic selection and native details). Theme, density and selected
example links use allowlisted GET values and work with scripts disabled. Form
fields are local demonstrations; Reset fields works natively, submission is
disabled, and there is no backend action or persistence.

Every preview discloses its real compiled Rust View and registered Style module
through `include_str!` and escaped text nodes. The build section also discloses
registration, class names, selection, gallery composition, host generation,
ordinary app CSS and the complete produced stylesheet. Snippets are not separately
maintained examples. Ordinary CSS supplies selectors/properties outside the finite
builder, including backgrounds, borders, focus, wrapping and gallery chrome.
The Grid changes from one to three columns at 768 CSS pixels; compact category
navigation replaces the desktop rail below 960 pixels.

## Explicit optional browser enhancement

With the repository's existing wasm32 target and matching wasm-bindgen CLI:

```sh
cargo xtask build showcase-browser
cargo build --locked -p yetlet-style-showcase
cargo run -p yetlet-style-showcase --locked
```

Open `http://127.0.0.1:3011/showcase/interactive`. Both routes render the **same
shared View and CSS**. The small Rust/WASM enhancement preserves all SSR nodes,
changes only predefined classes/semantic attributes/text, and owns disposable
listeners. It never inserts styles, writes inline CSS, opens a runtime circuit,
or performs HTTP commands. Modified-click links retain native navigation.

If browser output is absent, malformed or stale, the interactive route still
returns the useful native gallery with an explicit unavailable message and no
script. Native `/showcase` always omits browser assets, even when they exist.
Startup snapshots only the three fixed browser files after checking source/CSS
identity and asset digest. Output lives in `target/style-showcase-dist`; it is
optional, rebuildable local output, not a crash-safe deployment publisher.

Native HTML and CSS retain exact `DEFAULT_CSP`, MIME and nosniff. Only an available
interactive route selects the existing Todo policy with `wasm-unsafe-eval`.
No inline styles/scripts, runtime style elements, external network assets, general
hydration, renderer changes, public DSL, theme engine or server capability is added.

## Targeted verification

```sh
cargo test --locked -p yetlet-style-showcase --all-targets
cargo clippy --locked -p yetlet-style-showcase --all-targets -- -D warnings
cargo xtask smoke style-showcase-chrome --profile debug
cargo xtask smoke style-showcase-firefox --profile debug
cargo xtask check docs
cargo xtask check samples
```

Browser smoke requires the native and explicit browser builds above. Record actual
viewport width: headless Firefox may clamp a requested 390px window to 500px.
Linux debug verification does not establish Windows, macOS, Safari, physical
mobile, production or public-release support. The independent acceptance above
is limited to this recorded source and Linux sample scope.

## Observed YT-241 verification — 2026-11-08

Linux debug native build, four native tests, native/browser strict Clippy, the
seven existing builder tests and ten smoke-protocol tests pass. Chrome 157 smoke
passes at observed 390/768/1280 CSS pixels. Firefox 159 required gallery smoke
passes at observed 500/768/1280; the requested 390px window is clamped to 500px.
Native script-disabled navigation, disclosure, focus/disabled semantics, one CSS
request, computed theme/density/selected styles, source containment, exact CSP,
CSS identity and task-owned process teardown were checked. Desktop and narrow
first-view/source/selected screenshots were captured and actually inspected.

Chrome explicit listener disposal passes. Firefox's additional WebDriver dynamic
module-import disposal probe encounters an uninitialized module instance; explicit
listener disposal through that probe remains unverified. Required Firefox page
behavior and browser/server process teardown pass separately. This is not a
Firefox console/CSP-log audit or 390px qualification. Original compile, lint and
fixture failures are retained in the ignored YT-241 verification directory.
The independent implementation review accepts this bounded evidence; production,
other-platform qualification and public package publication remain separate.
