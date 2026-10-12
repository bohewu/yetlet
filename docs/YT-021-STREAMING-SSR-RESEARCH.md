# YT-021 — Full versus incremental streaming SSR research

**Task:** T03 / YT-021, delivery run `delivery-20261012-docs-research`.

**Authority:** Activated bounded [backlog](BACKLOG.md) research; [ADR 0003](adr/0003-renderers-and-loading.md) preserves complete native SSR and separate renderer/loading ownership.

**Evidence date:** 2026-10-12. This record applies to the local dirty checkout based on policy HEAD `abb2cce1253afe1d87a97e0a50f3f47efffe7280`, including this research implementation. It does not qualify or relabel historical source/artifact evidence.

## Decision and implementation boundary

**Retain the application-owned incremental prototype; reject promotion into a supported core streaming/suspense API on this evidence.** The large CPU-only page shows earlier first body bytes, but the small page is slower to its first body byte and all three workloads have lower sequential throughput and slower completion with streaming. The delayed-data case establishes scheduling semantics only: its explicit synthetic 20 ms wait is not evidence of a natural application advantage.

The unpublished [research example](../examples/streaming-ssr-research/README.md) uses mature Axum `Body::from_stream` and Hyper HTTP/1.1 rather than replacing HTTP parsing. It constructs and renders the heading before returning the response. On subsequent body polls, a `try_unfold` state machine constructs **one new row View** and serializes it through existing `yetlet-html::render_to_string`. It ends with fixed document closing tags. No full page View or fully rendered page String exists before the first fragment; this is not buffered HTML split into chunks. The baseline constructs one complete `main` View containing the same heading/list, serializes it once and uses the same static document envelope. Dynamic row text and attributes are escaped by the unchanged renderer.

The document shape, row generation and fragment boundaries are application-owned and fixed. No generic SSR helper, global suspense, template framework, browser assets, remote event circuit or supported crate API is introduced. The default HTML/server renderer remains complete SSR. No fundamental architectural selection is made, so ADR 0003 needs no change. Whether a realistic application warrants an optional adapter remains open; generic suspense, nesting and render-tree traversal were not investigated.

## Proven semantics and HTTP limits

Four focused product tests pass:

- Zero-, one- and 100-row outputs are byte-equivalent to full View rendering, including escaped text and attributes containing `<`, `>`, `&`, quotes and apostrophes.
- A bounded oneshot barrier blocks later row construction. The first fragment is available while that barrier is closed; polling the next fragment remains pending, and releasing the barrier completes the identical document.
- A forbidden `onclick` attribute in a later row makes full SSR return `500`. Streaming has already returned `200` with its CSP/content type/cache headers, emits the initial fragment and valid first row, then fails the body with the renderer error. That error cannot retrospectively replace the status or already emitted HTML.
- Both Axum routes return equivalent escaped HTML with `Content-Security-Policy: default-src 'none'; base-uri 'none'; form-action 'none'; frame-ancestors 'none'`, `Cache-Control: no-store` and `text/html; charset=utf-8`, without script tags.

Authorization, redirects, status, cookies, cache policy and document metadata must be decided before committing a streaming response. The fixed heading is validated before response creation, but later rows cannot be validated without giving up lazy generation. A body error after headers means an incomplete document/transport failure, not a replacement error page. This prototype has no error-boundary markup or late status protocol. The error test exercises the Axum body locally; it does not assert a browser's rendering or a wire client's treatment of a truncated response.

Full bodies have an exact size hint; this stream has no known content length. HTTP/1.1 generally frames unknown-length response bodies as chunked transfer, while HTTP/2 has different framing. Application fragments are not promises of one network packet, immediate flushing or one client body frame. Axum/Hyper, socket buffers, runtime scheduling, proxies and compression can coalesce or buffer data. Measured first body bytes arrived after multiple rows could already have been generated. There is no buffering-proxy, compressed, TLS or HTTP/2 coverage here.

The stream has no detached producer or unbounded application queue. Each poll generates at most one row; if polling stops, generation pauses and dropping the body drops its state. Hyper can still poll ahead until its own flow control applies. Slow-client/cancellation behavior and peak memory were not measured. CPU rendering occurs synchronously within each poll; one large row can still monopolize the runtime. Per-row allocations, serializer invocations and stream/HTTP framing contribute overhead; their individual costs have not been isolated. Streaming allows a complete early heading while later data is pending, but does not reduce the total required rendering work.

## Matched HTTP measurement method

Executed command:

```bash
cargo run --release -p yetlet-streaming-ssr-research --bin streaming-bench
```

Environment: Rust `1.99.0 (b940084d7 2026-09-28)`, Cargo `1.99.0 (5f94df478 2026-08-27)`, release optimization; Ubuntu host `bohe-ubuntu`, Linux `7.0.0-38-generic`, x86_64, Intel Core i5-13600K. Server and client share a Tokio `current_thread` runtime. The listener binds only `127.0.0.1:0`; Hyper opens a fresh TCP connection for every request, sets client TCP_NODELAY and uses HTTP/1.1 without TLS or compression. Concurrency is one. This is a localhost comparison with no affinity/governor or background-load control and no independent repeated-run confidence interval.

Each mode/workload receives three warmups followed by 20 samples (120 retained measured requests). Mode order alternates within each matched pair. Row text is exactly `<report> & "quoted" 'item'`, repeated four times; the same text is also the `title` attribute. Each row has a decimal index prefix. The full reference is generated outside the measured request; every HTTP response is checked byte-for-byte against it. The delayed variant puts the same single 20 ms readiness wait before rows in both modes; the streaming heading precedes that wait. CPU-only workloads have no sleep, including no zero-duration timer.

| Workload | Rows | Text repetitions | Explicit tail wait | Decoded HTML bytes |
|---|---:|---:|---:|---:|
| small | 100 | 4 | 0 ms | 29,434 |
| large | 10,000 | 4 | 0 ms | 2,939,134 |
| delayed | 1,000 | 4 | 20 ms, synthetic | 293,134 |

The metric named **first body byte** starts immediately before TCP connect and stops when the client receives its first nonempty decoded HTTP body data frame. It includes connection setup, client/server scheduling and response headers, but it is not the first HTTP response/header byte and not browser paint/FCP. The requested TTFB comparison uses this precise body metric. **Completion** uses the same start and stops at body EOF, including client body accumulation. **Lifecycle** additionally includes byte comparison and joining the connection driver. Sequential throughput is `20 × 1e9 / sum(lifecycle_ns)`; decoded MiB/s is that request rate multiplied by body bytes and divided by `2^20`. CSV printing and between-sample gaps are excluded, so this is the measured sequential request-lifecycle rate, not a concurrent load-test capacity claim.

## Results and raw observations

| Workload | Mode | Median first body byte (ms) | First body range (ms) | Median completion (ms) | Mean completion (ms) | Sequential requests/s | Decoded MiB/s |
|---|---|---:|---:|---:|---:|---:|---:|
| small | full | 0.188 | 0.184–0.210 | 0.196 | 0.199 | 4,694.11 | 131.77 |
| small | stream | 0.257 | 0.248–0.270 | 0.339 | 0.340 | 2,833.95 | 79.55 |
| large | full | 6.372 | 6.217–6.781 | 6.855 | 6.868 | 143.15 | 401.25 |
| large | stream | 3.011 | 2.958–3.076 | 12.451 | 12.453 | 79.66 | 223.28 |
| delayed | full | 21.630 | 20.696–21.710 | 21.665 | 21.580 | 46.31 | 12.95 |
| delayed | stream | 0.065 | 0.047–0.073 | 22.481 | 22.427 | 44.56 | 12.46 |

[Raw CSV](../examples/streaming-ssr-research/measurements.csv) retains all input conditions, sample IDs, mode order, exact byte sizes and integer nanosecond first-body/completion/lifecycle observations, plus the cleanup record. The same output is retained run-locally as `.pipeline-output/delivery-20261012-docs-research/outputs/T03-streaming-bench.csv`. Reproduction executes the command above; summarize each `(workload, mode)` group's 20 rows with median/min/max for `first_body_ns`, median/mean for `complete_ns`, and the stated throughput formula. Observations are rounded in this table only; raw durations remain intact. No numerical pass threshold was approved.

For the large page, lazy serialization plus protocol scheduling exposes body bytes sooner, with roughly doubled completion time in this run. The experiment does not isolate View-tree construction, serializer allocation or HTTP framing costs; it also does not demonstrate browser usability before the entire list arrives. For the small page, initial streaming overhead outweighs any first-body opportunity. The delayed workload's first fragment escapes its artificial wait, while completion still pays that wait plus streaming overhead. These results support retaining complete SSR as the default and keeping any future streaming choice explicit and application-specific.

## Verification and cleanup boundary

Executed focused checks:

```bash
cargo test -p yetlet-streaming-ssr-research
cargo build --release -p yetlet-streaming-ssr-research --bin streaming-bench
cargo fmt -p yetlet-streaming-ssr-research -- --check
cargo clippy -p yetlet-streaming-ssr-research --all-targets -- -D warnings
python3 scripts/check_docs.py
python3 scripts/check_samples.py
git diff --check
```

Native tests: **4 passed, 0 failed, 0 ignored, 0 filtered out**; binary and doctest targets contain no tests. The release HTTP measurement finished successfully, and all 120 measured responses plus 18 warmups matched the baseline with strict CSP headers. Compile/build, formatting, Clippy, documentation and catalog checks pass. The existing catalog remains 12 user-facing samples; this research fixture needs only workspace registration, with `publish = false`. The nine library candidates are unchanged.

The first strict Clippy check flagged one nested nonempty-data condition in the measurement client (`collapsible_if`). A single equivalent conditional correction resolved it; the focused rerun passed. The sample gate also established the catalog count as 12, correcting the initial draft's count. Neither correction changes the workloads or recorded timing semantics.

The final command output records all connection drivers joined, server joined and reconnect refused for the owned ephemeral listener `127.0.0.1:43931`. The command exited; it starts no browser, watcher or external service. Client/body errors are returned through the cleanup path, with bounded driver/server joins and abort/join fallback. No task-owned listener or background process remains.

**Browser/visual evidence:** not run; this native research comparison does not qualify layout, progressive paint, accessibility behavior, hydration or browser error recovery. The fixed semantic HTML is not a new polished application surface. **CSP evidence:** native response-header and HTML escaping tests, plus actual measured HTTP CSP headers; no new browser enforcement claim. Historical broad Rust/browser/Docker/CI qualification was not rerun or transferred.

YT-021 is Done only for this bounded incremental SSR comparison and rejection of core promotion. YT-029 stays Done at its own deadline research scope; YT-020 stays Blocked, YT-068 stays In progress, and YT-030/YT-065/YT-066/YT-067 retain their established bounds. The other three authorized research tracks remain Deferred until their tasks start. The next unblockable research task is **T04 / YT-024**, one opt-in application-owned custom action transport. No release activation is implied.
