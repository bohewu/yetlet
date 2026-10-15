# Native form and CSS proof — YT-070

An unpublished, loopback-only SSR proof for the M11 Lab Operations Board. It validates a draft but saves nothing and authenticates nobody. This is not the Board application or a generic forms API. Default pages require no JavaScript or WASM.

```sh
cargo xtask build form-css-proof --check
cargo run --locked -p yetlet-form-css-proof
```

The listener defaults to `127.0.0.1:3011`; `YETLET_FORM_PROOF_LISTEN` may select another loopback socket. Read the exact startup URL. Routes: `/` redirects to `/proof/native`; `/healthz` reports asset readiness; `/proof/native` is script-free; `/proof/enhanced` selects a local preview; POST `/proof/submit` validates with 422 errors or redirects to `/proof/accepted`; `/form-proof-assets/{name}` serves only the finite embedded inventory; `/favicon.ico` returns 204.

## Contracts

Text input, textarea, select and checkbox are native controls. False booleans are omitted; initial attributes/defaults differ from live values. A leading textarea sentinel preserves an initial newline during HTML parsing. Invalid drafts remain plain escaped text, with field descriptions; malformed/duplicate/unknown fields fail. The 64 KiB form-body bound accommodates percent-encoded Unicode within the smaller domain limits (title 160, notes 4,000 characters). Successful submission uses stateless POST/Redirect/GET.

Only `/proof/enhanced` includes the external preview module. Explicit activation reads live properties, never replaces controls or writes HTML, defers preview during composition and reads reset values after default handling. A weak owner map, abortable listeners, cancelled timers and current-owner checks protect disposal/remount. The two page-loader controls live for the document, separately from the disposable form owner. Synthetic composition tests do not establish real Chinese OS IME or spoken assistive technology.

Bootstrap 5.3.8 CSS is MIT; retain `assets/upstream/LICENSE.bootstrap`. `assets/inventory.json` records the exact source URL, upstream SHA-384/SHA-256 and all output hashes. Build-time deterministic processing externalizes 23 trusted-distribution SVG occurrences into same-origin files; no runtime CDN, data URL, remote font or Bootstrap JS is used. This fixed trusted-asset transformation is not a user-content sanitizer. `--fetch` is an explicit acquisition step; normal builds/checks are offline. The finite asset table validates paths and MIME, orders/deduplicates styles, and embeds exact bytes. An absent/invalid manifest preserves readable native HTML and gives 503 readiness. Security middleware covers success/error/fallback responses without unsafe-inline/eval. Assets alone use immutable caching; other responses use no-store.

```sh
cargo test --locked --offline -p yetlet-form-css-proof --all-targets
cargo clippy --locked --offline -p yetlet-form-css-proof --all-targets -- -D warnings
cargo build --locked --offline -p yetlet-form-css-proof
cargo xtask smoke form-css-proof-chrome
```

The browser command uses the native xtask DevTools helper, opens one isolated headless Chrome and one ephemeral server, and closes only those resources. Actual reports and screenshots go to fresh ignored `target/m12/yt085/`. No new library dependencies or generic DOM policy are introduced. The proof stays outside the verified sample catalog until its recorded acceptance is complete.
