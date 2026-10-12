# YT-029 — Opt-in HTTP deadline and native overhead research

**Task:** T02 / YT-029, delivery run `delivery-20261012-docs-research`.
**Status:** Done at bounded application-owned research scope, 2026-10-12.
**Authority:** Approved YT-029 research delivery; [ADR 0004](adr/0004-request-context-and-middleware.md), [ADR 0006](adr/0006-handler-first-mvc-optional.md), [HTTP pipeline](HTTP-PIPELINE.md) and [backlog](BACKLOG.md).
**Source:** Current checkout HEAD `abb2cce1253afe1d87a97e0a50f3f47efffe7280` plus these uncommitted example/documentation edits and preserved pre-existing signing/T01 documentation. This is new local evidence, not evidence for the HEAD tree alone. Historical hosted qualification at `c38837835dac683b09194b475baab2d8eda5a875` and signed source `8e557e760407a3d4b3f5a8b8f833748946be1387` retain their own identities; neither attests this research.

## Decision and implementation boundary

**Adopt the demonstrated native Tower/Axum composition for explicitly selected application use. Reject adding a generic deadline provider/framework or supported Yetlet core deadline API in this slice.** Existing mature timeout/error composition suffices; deadline budgets, response status and cancellation/transaction policy belong to the application. No architectural decision changes, mandatory middleware, handler scaffolding or new HTTP parser are needed. No numerical acceptance threshold was approved.

[The example module](../examples/middleware-lab/src/deadline.rs) installs `TimeoutLayer` inside `HandleErrorLayer`, around a completed Router using existing `with_global`. An elapsed timeout maps to ordinary HTTP 504; another service error maps to 500. The inner Router is infallible, so the other-error branch is defensive mapping, not a demonstrated new failure path. The normal middleware-lab `app()` and listener install no deadline. `default-run` preserves the existing `cargo run -p yetlet-middleware-lab` entry after adding the separate [measurement binary](../examples/middleware-lab/src/bin/deadline-bench.rs). Only existing Tower/Tokio feature flags are enabled; Cargo.lock is unchanged. No Yetlet library or browser source changes.

## Demonstrated semantics and limits

Six focused tests run alongside all seven existing middleware-lab tests:

| Behavior | Actual evidence |
|---|---|
| Timely completion | Explicitly layered immediate handler returns 200 and `ok` |
| Timeout and default behavior | With a 100 ms budget, an awaited 200 ms handler returns 504 at 100 ms of paused Tokio time; the same unlayered handler returns its original 200/body after 200 ms |
| Cancellation by dropping | Pending handler's owned guard drops once before the 504 is observed; work after its suspended await never executes |
| Earlier effects | Counter increment before the await remains 1 after timeout and further time advancement; no automatic rollback |
| Spawned work | Handler-spawned task survives timeout, increments its counter later and is explicitly joined |
| Started blocking work | An already-started `spawn_blocking` operation remains pending across timeout and explicit abort; releasing its channel lets it finish, increment its counter and be joined |
| Request body | Delayed body read awaited inside the handler expires before producing a response |
| Response body | Response returns 200 immediately, then its delayed body completes after 200 ms despite the 100 ms service budget |

Five tests use paused Tokio time to demonstrate deterministic service/body behavior. The started-blocking test uses real Tokio time and a start/release barrier; its pending request times out and the worker is released/joined before return. These tests do not assert hard real-world latency guarantees.

The timeout is **cooperative**, covering execution of the inner service future until it produces a Response. It does not time socket acceptance, HTTP parsing/header arrival, service readiness or the entire connection. Request upload waits are included only when awaited within that execution. Once a response is returned, the timer is gone: whole-body, idle-stream, downstream-client and transport deadlines require separate application/server policy. Already-sent headers cannot be replaced with a new timeout status.

Dropping an owned Rust future can cancel its remaining local awaits; it does not reverse committed database writes, messages already dispatched, files written or other external effects. The counter test is local evidence for that boundary, not a distributed transaction test. A detached task, remote operation or started blocking closure may continue. Non-yielding/blocking work inside the service future can prevent the timer from being polled. In the actual pinned Tower 0.5.3 source, the response future is polled before the timer; a response becoming ready on that poll can win even after elapsed wall time. This is not a universal wall-clock watchdog or global cancellation promise.

Upstream primary references: [Axum fallible middleware/error mapping](https://docs.rs/axum/0.8.9/axum/error_handling/index.html), [Tower 0.5.3 timeout future source](https://docs.rs/tower/0.5.3/src/tower/timeout/future.rs.html), [Tokio started blocking-task cancellation limits](https://docs.rs/tokio/1.53.2/tokio/task/fn.spawn_blocking.html). Pinned local registry source was also inspected for timeout poll order; online latest versions are not substituted for the lockfile.

## Matched native measurement method

Executed 2026-10-12 around 08:13 UTC on Linux `7.0.0-38-generic`, x86_64 GNU/Linux; Intel Core i5-13600K, 20 logical CPUs, 64,524,128 kB reported RAM, CPU0 governor `powersave`. Rust `1.99.0 (b940084d7 2026-09-28)`, Cargo `1.99.0 (5f94df478 2026-08-27)`, LLVM 23.1.1, native `x86_64-unknown-linux-gnu`; locked Axum 0.8.9, Tower 0.5.3 and Tokio 1.53.2. No governor, affinity, clock, isolation or system settings were changed. Other workstation activity was not controlled.

Release binaries were built before timing. The benchmark uses a current-thread Tokio runtime with real time, four routers constructed once, and a shared static `GET /bench` workload: empty request body, two-byte `ok` response, status 200. Every iteration includes request construction, Router clone, `oneshot` execution, bounded body collection, status/body assertions and a `black_box` result. It excludes router construction, compilation, network listeners/clients, TLS, SQL/Redis, meaningful SSR rendering, logging and production request concurrency. Each variant has 10,000 warmup requests; each of ten samples has 100,000 requests per variant. The starting variant rotates each sample. Total measured requests: 4,000,000, plus 40,000 warmups. Deadline budget is one second and every measured handler completes normally; timeout/error paths are tested separately, not timed here.

| Variant | Composition | Median ns/request | Min–max batch-average ns/request |
|---|---|---:|---:|
| baseline | Direct native Axum Router | 375.590 | 374.889–383.353 |
| global_envelope | Same Router inside existing `with_global`, identity layer | 538.303 | 536.977–540.874 |
| request_context | Same global envelope plus one no-op native middleware extracting `RequestContext<()>` | 619.120 | 616.863–620.720 |
| deadline | Same global envelope plus Axum error mapper and Tower timeout | 622.542 | 621.597–624.568 |

Differences of sample medians: envelope versus direct Router **+162.714 ns/request**; no-op context middleware versus envelope **+80.817 ns/request**; deadline/error mapping versus envelope **+84.239 ns/request**; complete opt-in deadline path versus direct Router **+246.953 ns/request**. These are observed aggregate differences for the exact compositions, not isolated allocation counts or universal per-layer constants. The context variant and deadline variant are alternatives, not stacked. Existing lab trace/header middleware is not benchmarked by this fixture.

Batch averages are not individual-request p95/p99 or network TTFB. There is no production throughput, workload scalability, security certification or improvement claim. Small absolute differences and scheduler/timer/allocator/CPU behavior are environment-dependent. This is sufficient to retain bounded native cost evidence and avoid claiming zero overhead; it does not select a global timeout policy.

## Raw observations and reproduction

The direct raw output is retained in the ignored run artifact `.pipeline-output/delivery-20261012-docs-research/outputs/T02-deadline-bench.csv`; the result record is `.pipeline-output/delivery-20261012-docs-research/outputs/T02-result.json`. Repository-retained raw batch durations below allow reconstruction independently of that local directory. Each cell is total elapsed nanoseconds for exactly 100,000 requests; divide by 100,000 for the batch-average ns/request. Column ordering is for reading; execution rotates as documented above.

| Sample | baseline total ns | global_envelope total ns | request_context total ns | deadline total ns |
|---:|---:|---:|---:|---:|
| 1 | 37571715 | 53945296 | 62071963 | 62423168 |
| 2 | 37546245 | 53902049 | 62039172 | 62268767 |
| 3 | 37582262 | 53701610 | 61786381 | 62177978 |
| 4 | 37730061 | 53809371 | 61686348 | 62162439 |
| 5 | 37488868 | 53751619 | 61824020 | 62159697 |
| 6 | 37606221 | 54087429 | 61923590 | 62456764 |
| 7 | 38335286 | 53941948 | 62063828 | 62239646 |
| 8 | 37529868 | 53723297 | 61891295 | 62323727 |
| 9 | 37521289 | 53851163 | 61900315 | 62175859 |
| 10 | 37531129 | 53697663 | 61985549 | 62286234 |

```bash
cargo test --locked --offline -p yetlet-middleware-lab --all-targets
cargo build --release --locked --offline -p yetlet-middleware-lab --bins
target/release/deadline-bench
cargo fmt -p yetlet-middleware-lab -- --check
cargo clippy --locked --offline -p yetlet-middleware-lab --all-targets -- -D warnings
python3 scripts/check_docs.py
python3 scripts/check_samples.py
git diff --check
```

## Verification, cleanup and retained status

- **Compile/product:** Targeted all-target test run passed 13/13, with zero ignored/filtered tests; both binary test targets contain zero tests. Release build of both native binaries passed. Formatting and targeted Clippy with warnings denied passed.
- **Documentation/catalog:** English/reference/backlog checks and applicable sample catalog checks passed; `git diff --check` passed. These are existing checkers, not newly created validation infrastructure.
- **Fixture history:** Initial compile failed because two test closures captured non-Clone one-shot channels; fixture-local ownership was corrected. A subsequent paused-time blocking test stalled because active blocking work prevented automatic virtual-time progression; that one fixture switched to real Tokio time. Both were harness failures, with no concrete product behavior disproved. The task-owned stalled test and Cargo processes were identity-checked and terminated, then the focused package run passed. No model/capability recovery or product repair was needed.
- **Resources:** No listener, browser, database, watcher or external service was started. Every spawned async/blocking worker in the successful tests was joined; the one-shot benchmark and builds exited. Terminated stalled test PIDs `1442081` / Cargo `1441543` were checked absent. No task-owned helper process remained.
- **Not run:** Browser compilation, browser/CSP smokes, broad workspace Rust suites, Docker and hosted CI qualification. This slice changes no browser/CSP/core source; historical evidence remains attributed to its original runs.

YT-029 is Done only for this deadline/native-overhead research decision. General optional HTTP providers, whole-body policy, production deadline guarantees and core API adoption remain outside the slice. YT-020 stays Blocked; YT-030/YT-065/YT-066/YT-067 retain their established completion bounds and YT-068 stays In progress. Four other research items remain Deferred. The next authorized unblockable research task is **T03 / YT-021**, full versus actual incremental streaming SSR measurements and adoption decision; release activation remains a separate decision.
