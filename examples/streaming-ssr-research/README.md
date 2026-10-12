# Incremental streaming SSR research

This unpublished, native-only YT-021 research fixture compares complete SSR with
an application-owned stream of lazily constructed `View` fragments. It does not
add a supported renderer, suspense API, browser runtime or event circuit.

Run from the repository root:

```bash
cargo test -p yetlet-streaming-ssr-research
cargo run --release -p yetlet-streaming-ssr-research --bin streaming-bench
```

The bounded measurement command starts its own `127.0.0.1:0` Axum listener,
uses Hyper's HTTP/1.1 client, measures three fixed workloads sequentially and
shuts down/join-checks all owned connections and the server before returning.
It checks that reconnecting to the owned listener fails. Requests use `/full`
and `/stream` beneath `/small`, `/large` and `/delayed`.

`src/lib.rs` constructs only the heading before streaming. Each subsequent body
poll constructs and escapes one list-row View through `yetlet-html`; the full
baseline constructs and renders the complete main View. A oneshot barrier test
proves initial-fragment availability before later row construction. The HTML
uses native `main`, `h1` and `ul`/`li` semantics, an English root language and a
viewport declaration; no CSS, script, WASM or external assets are needed.

[Research record](../../docs/YT-021-STREAMING-SSR-RESEARCH.md) describes the
timing definitions, error/header/backpressure limits, decision and verification
boundary. [Raw observations](measurements.csv) retain all 120 samples from the
current dirty-checkout research run; they are local observations without a
production performance guarantee. This fixture stays outside the user-facing
sample catalog and the nine library candidates.
