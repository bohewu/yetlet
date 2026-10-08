# M5 MVP Acceptance and Performance Evidence

**Date:** 2026-10-08
**Status:** Local prototype acceptance and release-build measurements recorded; not a production release, GitHub Actions execution, comparative benchmark, or independent security review.
**Backlog:** YT-019. YT-020 publication requirements are tracked in [Release readiness](RELEASE-READINESS.md).

## A01–A10 acceptance matrix

The normative criteria remain in [MVP](MVP.md).

| ID | Local status | Observed evidence and scope |
|---|---|---|
| A01 | PASS | `yetlet-html` escaping/policy tests and native SSR island-attribute escaping tests |
| A02 | PASS | `GET /` includes usable HTML with no script/WASM; Rust integration and Chrome/Firefox |
| A03 | PASS | `GET /search` includes SSR results, external versioned loader and bounded SearchStateV1 |
| A04 | PASS | Chrome local search/empty/reset; Firefox physical keyboard input and deletion; no input-triggered Todo API requests |
| A05 | PASS | Chrome and Firefox Browser Todo perform two local clicks without remote API requests |
| A06 | PASS | Chrome initial TodoList `outerHTML` is byte-identical between native SSR and browser DOM |
| A07 | PASS | Typed JSON API errors 400/403/404/413/415/422; failure preserves revision; both browsers show typed 422 |
| A08 | PASS at prototype scope | External scripts, strict CSP without `unsafe-inline`/`unsafe-eval`, Chrome resource/console checks; WASM explicitly requires `wasm-unsafe-eval` |
| A09 | PASS | Cargo native and wasm32 checks, browser dependency graph excludes Axum, Hyper, Tower and RequestContext |
| A10 | PASS for a local baseline | Reproducible tool records environment, SSR timing, HTML sizes, gzip artifact estimates, browser activation timing and limitations |

Passing A01–A10 **at this local prototype scope** is not an independent production, security, IME/accessibility, or cross-platform certification.

## Reproduction

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --all-targets --locked
cargo check --locked -p yetlet-todo-browser -p yetlet-search-island --target wasm32-unknown-unknown
python3 scripts/check_docs.py

# Chrome E2E currently selects the DEBUG native server:
cargo build --locked -p yetlet-todo-server
python3 scripts/build_browser.py
python3 scripts/smoke_browser.py

# Release binary and release browser asset manifest must match:
cargo build --release --locked -p yetlet-todo-server
python3 scripts/build_browser.py --release
python3 scripts/smoke_firefox.py --profile release
python3 scripts/measure_baseline.py --profile release --samples 30 \
  --browser-runs 5 --out target/m5-baseline-release.json
```

Browser/WASM packaging requires a `wasm-bindgen-cli` version matching `Cargo.lock` (0.2.129). The Firefox test also needs Firefox and geckodriver. The measurement JSON is intentionally in the Git-ignored `target/` directory. Do not mix debug and release assets in one measurement run.

## Measured release environment

| Dimension | Observed |
|---|---|
| OS | Linux x86_64 |
| rustc | 1.99.0 (2026-09-28) |
| Chrome | 155.0.8059.39 (headless, cache disabled during timing) |
| Firefox | 157.0, geckodriver 0.37.1 (headless isolated WebDriver smoke) |
| Python | 3.14.4 |
| Browser manifest digest | `3f3c5f074478c525` |
| Build | Native Rust release + browser WASM release, wasm-bindgen 0.2.129 |

## Release latency measurements

| Metric | Samples | Median | P95 | Range |
|---|---:|---:|---:|---|
| Local `GET /` request to first HTTP response byte (TCP already connected) | 30 | **0.034 ms** | 0.039 ms | 0.032–0.052 ms |
| Chrome `/app` navigation to WASM-ready DOM marker | 5 | **48.5 ms** | 50.6 ms | 47.9–50.6 ms |
| Chrome `/search` interaction trigger to island-ready marker | 5 | **3.7 ms** | 5.3 ms | 3.0–5.3 ms |

The first-byte clock starts immediately before the HTTP request is written and stops at the first response byte, which may be part of the headers; it excludes TCP connection establishment. Browser measurements use DevTools-injected MutationObserver marks without modifying Yetlet runtime code. These numbers are neither Web Vitals (LCP/INP) nor real-world WAN/TLS measurements.

## Release HTML and asset sizes

| Output | Size |
|---|---:|
| `GET /` static SSR HTML | 532 bytes |
| `GET /search` initial SSR HTML | 1,431 bytes |
| `GET /app` shell HTML | 554 bytes |
| Browser WASM | 216,006 bytes, uncompressed |
| Search Island WASM | 179,716 bytes, uncompressed |
| Full Browser JS + WASM | **81,307 bytes gzip**, local estimate |
| Search initial JavaScript loader | **551 bytes gzip**, local estimate |
| Search dynamically loaded JS + WASM | **74,481 bytes gzip**, local estimate |

Gzip values are sums of locally compressed artifacts, **not actual wire transfer bytes**. CSS and favicon are excluded from the three combined JS/WASM totals. HTTP compression is not claimed or verified.

## Limitations and interpretation

- One local host, loopback, single headless Chrome timing environment; browser cache was disabled. No TLS, remote network, concurrent throughput, cold-process baseline or framework comparison.
- Chrome E2E covers WASM/SSR, API actions, lazy islands, CSP/resource and console observations. Firefox E2E covers keyboard input/clearing, WASM, local/remote actions and typed 422 but not complete console/CSP telemetry.
- Safari/WebKit/mobile, real IME composition, comprehensive accessibility, authentication/authorization, credentialed CSRF, reverse-proxy TLS, persistence and independent security review remain unverified.
- The declared Rust MSRV 1.85.0 is **not** tested: the Runner has rustc 1.99.0.
- GitHub Actions has not executed these M5 changes remotely. Local results do not establish CI success.

See [Release readiness](RELEASE-READINESS.md) for explicit blockers. These figures provide a repeatable starting point, not a performance budget or evidence that Yetlet is faster than other frameworks.
