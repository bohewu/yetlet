# Constrained resumability research — YT-022

This unpublished, native-only study encodes an explicit versioned handler ID and
two bounded numeric capture fields, then decodes and dispatches through an ordinary
Rust match over two precompiled local counter handlers. It serializes data only;
it does not serialize closures, arbitrary Rust captures, code or execution stacks.
No browser, HTTP server, remote event circuit or supported Yetlet API is added.
The research fixture is a workspace member outside the user-facing sample catalog.

From the repository root:

```bash
cargo test -p yetlet-resumability-research --offline
cargo run --release -p yetlet-resumability-research --offline --bin resumability-bench
```

Protocol v1 accepts only an object containing `protocol_version`, `handler_id`,
`handler_version` and `capture`. Known IDs are `counter.add` and
`counter.subtract`, each at handler version 1. Capture is an object containing
only integer `count` (0–1,000,000) and `step` (1–1,000). The entire JSON document
is limited to 256 bytes before parsing; missing/extra/duplicate fields,
positional arrays, invalid types/ranges, unknown handlers and version mismatches
fail closed. Dispatch also checks its result stays within the count range.

Snapshots are untrusted application data, not authentication, authorization or
integrity proof. The capture type has no secret, credential or server capability
field. The application must deliberately select client-safe values; this is not
automatic secret detection or redaction. Valid numeric tampering is deliberately
accepted and tested. Remote mutations would still require the existing typed
action boundary and server-side authorization/input checks.

[Research record](../../docs/YT-022-RESUMABILITY-RESEARCH.md) describes the measured
workload, version compatibility, security limits and adoption decision.
[Raw observations](measurements.csv) retain actual encoded JSON/bytes and all
100 native timing samples. Each phase gets three warmups and 20 measured batches
of 40,000 operations on the same four descriptors. The command exits without
creating listeners, threads, watchers or browser sessions. Native costs do not
demonstrate hydration, startup, payload or browser performance savings.
