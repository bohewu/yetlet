# YT-022 — Constrained handler and capture resumability research

## Scope and decision

**2026-10-12 / T05:** retain the explicit application-owned, finite data-descriptor
pattern for this bounded experiment; **reject promotion to generic resumability
or a supported core API**. Two known precompiled local handlers can reconstruct
numeric state and dispatch after a JSON round trip. This demonstrates stable
literal identities and constrained data capture, not arbitrary closure or
execution-stack resumption. The boilerplate, deployment/version policy and
trust review remain application responsibilities. There is no evidence that this
improves hydration or browser startup, and no numerical pass threshold was approved.

The isolated [source](../examples/resumability-research/src/lib.rs),
[tests](../examples/resumability-research/src/tests.rs) and
[direct benchmark](../examples/resumability-research/src/bin/resumability-bench.rs)
belong to the unpublished `yetlet-resumability-research` example (`publish = false`).
Its only dependencies are the already locked serde/serde_json packages; necessary
workspace registration and its lockfile package entry do not change the nine
library candidates or the 12 user-facing catalog samples. No renderer, UI runtime,
HTTP/action interface or production browser asset is modified. No architectural
selection changes [ADR 0002](adr/0002-events-and-actions.md) or
[ADR 0003](adr/0003-renderers-and-loading.md); default complete SSR remains usable
without JS/WASM and local events remain local.

## Data and dispatch contract

The canonical 101-byte Add fixture is:

```json
{"protocol_version":1,"handler_id":"counter.add","handler_version":1,"capture":{"count":40,"step":2}}
```

`counter.add` and `counter.subtract` are literal identities resolved with a finite
Rust match, independently of function addresses or serialized function names.
Both require handler version 1 and protocol version 1. There is no plugin registry,
dynamic code loading, reflection, automatic capture derivation or remote event
forwarding. The selected code is compiled before execution.

Capture permits exactly two numeric fields: integer `count` in 0–1,000,000 and
integer `step` in 1–1,000. The envelope and capture reject unknown, missing and
duplicate fields. Object-shape checks explicitly reject positional arrays (serde
structs otherwise permit them). Wrong types, nulls, nested capture objects,
overflowing integers, invalid UTF-8 and trailing documents are rejected. The
complete UTF-8 JSON input, including whitespace, must fit **256 bytes**, checked
before serde parsing/ID allocation. Unknown ID strings within that bound are
rejected; longer documents fail the size check. There is no arbitrary JSON payload.

Decoding validates schema, handler identity/version and capture ranges. Encoding
and dispatch independently recheck validity, including when a caller bypasses the
checked decoder with raw serde. Dispatch computes `count + step` or `count - step`
and rejects results outside the count range. Round-trip tests produce 42 and 38
from count 40/step 2, then construct explicit next-event snapshots producing 44
and 36. This reconstructs a value and runs known code; no suspended stack,
subscription graph, browser node, closure environment or request continuation is
resumed. Boundary cases produce 1,000,000 and 0, and out-of-range results fail.

### Compatibility and trust

All protocol/handler versions other than 1 fail closed, with no migration or
automatic fallback. A future application must couple descriptors to compatible
build-time assets, never reuse a handler ID/version for changed semantics, and
define deploy/cache skew, retirement, migration or a safe SSR fallback explicitly.
This experiment does not validate cross-build compatibility, hashed asset identity,
DOM instance ownership, browser event binding or delayed asset loading.

The capture type deliberately contains no arbitrary string, credential, repository
object, server handle or capability. Synthetic extra `token`, `server_handle`,
`code` and `closure` fields are rejected. This is a type/schema boundary, **not a
token-content redactor or automatic secret detector**. The application must choose
client-safe values, even for numeric fields; adding an allowlisted string in a
future application would not make its contents automatically safe.

Browser-provided snapshots are untrusted, public and tamperable. A test changes
count 40 to 100 and obtains 102: a valid descriptor proves neither integrity nor
authentication/authorization. Replaying the same snapshot repeats the same local
calculation; there is no freshness, replay protection or durable state authority.
These handlers expose only local arithmetic. Any future remote mutation must use
the separate typed action boundary and revalidate authorization, inputs and
CSRF/idempotency policy on the server. The JSON is ordinary data, not HTML-safe
hydration markup; future embedding needs escaping and CSP/browser enforcement
qualification. This study emits no HTML or inline script and evaluates no code.

## Matched native cost measurement

Executed:

```bash
cargo run --release -p yetlet-resumability-research --offline --bin resumability-bench
```

Environment: Rust `1.99.0 (b940084d7 2026-09-28)`, Cargo
`1.99.0 (5f94df478 2026-08-27)`, native release optimization; Ubuntu host
`bohe-ubuntu`, Linux `7.0.0-38-generic`, x86_64, Intel Core i5-13600K. Locked serde
is 1.0.229 and serde_json is 1.0.151. This is one native foreground process, with
no concurrency, CPU affinity/governor control or independent repeated-run
confidence interval. Measurements are from this dirty delivery checkout based on
HEAD `abb2cce1253afe1d87a97e0a50f3f47efffe7280`, after T01–T04 work; they are not
evidence for historical signed source `8e557e760407a3d4b3f5a8b8f833748946be1387`
or a public release. Run-local source hashes identify the measured code.

All five phases use the same ordered workload:

| Handler | Count | Step | Result | Actual encoded UTF-8 bytes |
|---|---:|---:|---:|---:|
| counter.add v1 | 40 | 2 | 42 | 101 |
| counter.subtract v1 | 40 | 2 | 38 | 106 |
| counter.add v1 | 999,000 | 1,000 | 1,000,000 | 108 |
| counter.subtract v1 | 1,000 | 1,000 | 0 | 111 |

The four encoded documents total **426 bytes** with no compression/HTML framing.
The benchmark checks their decoded dispatch results before timing. Each phase gets
three discarded warmup batches and 20 retained batches of 10,000 repetitions of
the four-descriptor workload: **40,000 operations per batch**, 100 measured rows.
Phase order rotates each round. `Instant` measures entire batches; compiler
`black_box` consumes inputs/results. Preparation, correctness checks and CSV
printing are outside timing. Per-operation cost is `elapsed_ns / operations`;
the table summarizes each phase's 20 such values, rounded only here.

| Phase | Median ns/operation | Minimum | Maximum |
|---|---:|---:|---:|
| Direct typed handler | 1.868 | 1.738 | 1.893 |
| Descriptor dispatch | 2.110 | 1.967 | 2.213 |
| Encode | 66.472 | 65.255 | 67.414 |
| Decode | 144.483 | 142.839 | 149.826 |
| Encode + decode + dispatch | 215.526 | 214.097 | 218.361 |

Direct typed calls include capture/result validation. Descriptor dispatch adds
version/ID resolution and validation; encode includes validation, JSON allocation
and vector drop. Decode includes input bounds, parsing, ID allocation, validation
and descriptor drop. The combined phase includes the whole data round trip.
Repeated tiny inputs are cache-warm; nanosecond-scale dispatch differences are
especially sensitive to compiler/loop/timer effects and are not an application
latency forecast. Parsing malformed inputs, allocator peak memory, network,
browser/WASM payload, loader overhead, DOM binding, startup and hydration were not
measured. Native JSON costs and bytes cannot establish browser savings.

[Raw CSV](../examples/resumability-research/measurements.csv) retains the exact
encoded documents, byte lengths, phase/order/sample IDs and integer elapsed
nanoseconds, plus the cleanup record. The same output is retained as
`.pipeline-output/delivery-20261012-docs-research/outputs/T05-resumability-bench.csv`.
Reproduce with the command above and group rows by `phase` for median/min/max of
`elapsed_ns / operations`; fresh results may differ.

## Verification and delivery boundary

Executed focused checks:

```bash
cargo test -p yetlet-resumability-research --offline
cargo build --release -p yetlet-resumability-research --offline
cargo fmt -p yetlet-resumability-research -- --check
cargo clippy -p yetlet-resumability-research --offline --all-targets -- -D warnings
python3 scripts/check_docs.py
python3 scripts/check_samples.py
git diff --check
```

Native product tests: **6 passed, 0 failed, 0 ignored, 0 filtered out**; binary
and doctest targets contain zero tests. Release build, formatting and strict
Clippy pass. Documentation/reference/backlog and sample gates pass. The benchmark
completed successfully with all declared raw samples; no performance threshold
or production qualification was inferred. Run-local logs and the T05 result
record retain the actual commands and exits. No code repair was needed; one
draft environment-version correction followed direct lockfile inspection.

**Browser/WASM evidence:** not run. **CSP/browser enforcement:** not run; this
native data-only study has no HTML/HTTP/browser surface. Historical broad
Rust/browser/Docker/CI verification was not rerun or transferred to this source.
Every task command completed; no task-owned listener, thread, watcher, server or
browser session was created. The benchmark's final cleanup line records this
foreground-only boundary; the release-build command session also exited.

YT-022 is Done only for this constrained semantics/cost study and its bounded
decision. YT-029/YT-021/YT-024 remain Done at their own research bounds;
YT-023 remains Deferred until the next task. YT-068 stays In progress, YT-020
stays Blocked, and YT-030/YT-065/YT-066/YT-067 retain their established completion
bounds. The next unblockable task is **T06 / YT-023**, the independent optional
WebGPU scene-renderer study. No stage, commit, push, signing, publication or
external environment change is part of T05.
