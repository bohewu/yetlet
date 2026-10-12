# YT-024 — Application-owned custom action transport research

**Task:** T04 / YT-024, delivery run `delivery-20261012-docs-research`.

**Authority:** The activated [backlog](BACKLOG.md) research item and
[ADRs 0001](adr/0001-separation-of-concerns.md),
[0002](adr/0002-events-and-actions.md) and
[0006](adr/0006-handler-first-mvc-optional.md).

**Evidence date:** 2026-10-12. Evidence applies to the local dirty checkout based on
policy HEAD `abb2cce1253afe1d87a97e0a50f3f47efffe7280`, including this research
implementation and the preceding documentation/deadline/streaming work. Historical
signed source `8e557e760407a3d4b3f5a8b8f833748946be1387` remains a separate identity;
this evidence does not qualify, sign or relabel historical artifacts.

## Decision and ownership

**Adopt the application-owned adapter pattern for this bounded prototype; reject
promotion into a supported generic transport framework or core TCP executor.**
The existing `Command` and `ActionExecutor<C>` traits suffice without modification.
The [unpublished research fixture](../examples/custom-action-transport/README.md)
implements `ActionExecutor<Add>` directly. Its synthetic command owns its serde
representation and `Sum` response, while the native adapter owns codec, endpoint,
deadline and error mapping. No generic provider registry, routing facade or new
core abstraction is justified by one command and one wire protocol.

The application's explicit executor construction chooses local or remote execution.
A `LocalExecutor` arithmetic call while a test listener is waiting does not accept
a connection; a later explicitly selected `TcpExecutor` produces a real typed
network result. This native test establishes executor selection, not a new browser
interaction result. Ordinary UI events continue to belong to the existing local
runtime: this adapter imports no DOM/event types and has no browser integration.
Default no-JS/no-WASM SSR and existing HTTP execution remain unchanged.

The dependency boundary is `yetlet-actions` plus Tokio, `tokio-util`, futures and
serde. No core, renderer, reactive runtime or server adapter depends on this
research package; no HTTP routing or Controller/DI scaffolding is added. The
existing nine library candidates remain unchanged. `publish = false` and workspace
registration enable focused native checks only; the user-facing catalog remains
12 samples. No fundamental architectural decision changes, so the ADRs retain
their existing status and supported-core custom transport remains proposed.

## Wire contract and error behavior

One native TCP connection carries one explicit semantic command and one result.
The mature `tokio-util::codec::LengthDelimitedCodec` handles framing: four-byte
big-endian length, then at most 1,024 payload bytes, on both client and server.
This is a custom non-HTTP protocol, not a second HTTP parser/server stack. The
[codec documentation](https://docs.rs/tokio-util/0.7.19/tokio_util/codec/length_delimited/index.html)
describes the configured framing and maximum-frame control; the installed 0.7.19
primary source was also inspected. No manual framing parser is implemented.

Successful actual wire exchange:

```text
request (22 payload bytes): {"left":20,"right":22}
reply   (26 payload bytes): {"result":"ok","value":42}
```

Each frame adds four header bytes, so this exchange carries 56 application wire
bytes excluding TCP/IP headers. `Add` accepts exactly two unsigned 32-bit integers;
unknown fields, missing/wrong fields and invalid JSON fail serde decoding. Server
arithmetic uses checked addition. Overflow returns `{"result":"error","code":"overflow"}`;
malformed commands return `{"result":"error","code":"invalid_command"}`. The server
then closes the connection. Oversized and truncated frames fail the codec before
command decoding and close without a reply. There is no retry, session, multiplexing,
idempotency mechanism, event replication or reconnect loop.

The client maps error envelopes to existing `ActionError::Remote { status: 422,
code }`. The number 422 is explicitly an application compatibility mapping, not an
HTTP status received over TCP. Invalid JSON, wrong result shape, unexpected EOF or
response framing failures map to `InvalidResponse`. Connect/send failures and the
whole-request deadline map to `Network`; the existing error type does not expose a
distinct timeout variant. No failed request is treated as a successful result.

The client owns its socket inside the execution future and starts no helper task.
Dropping that future, whether through timeout or caller cancellation, drops the
socket. The [Tokio timeout contract](https://docs.rs/tokio/1.53.2/tokio/time/fn.timeout.html)
is cancellation by future drop, with no preemption of work that never yields.
The application handler similarly owns one socket under a one-second deadline.
The test fixture owns and joins the accept/connection task and closes its ephemeral
listener after one accept, with bounded abort/join fallback on a missed cleanup
deadline. Cancellation does not roll back domain effects already committed; this
command is deliberately stateless and read-only. No side-effect/retry guarantees
are inferred for future commands or spawned/blocking work.

## Security and deployment limits

Both adapter construction and server connection handling require loopback; tests
bind `127.0.0.1:0`. This confines synthetic proof traffic but authenticates nobody:
another local process could send commands. No identity, credential, private data,
mutating domain capability, TLS or external listener is present. Authentication
and authorization must be revalidated by a real application for every command;
typed input and loopback addresses cannot replace those controls.

This native socket protocol has no browser HTTP endpoint, cookies or ambient
browser identity. The proof therefore neither implements nor qualifies CSRF.
Any browser bridge or cookie-authenticated mutation would need its own deliberate
Origin/CSRF/SameSite policy and authorization at the server boundary. Browser
sandbox portability was not investigated. Exposing this unauthenticated protocol
to a network or real domain mutations is outside its evidence and scope.

Frame limits and per-connection deadlines bound this fixture's input buffering and
waiting. They are not rate limiting, admission control, concurrent-load protection,
process-wide memory bounds or protection from hostile clients. The single sequential
proof listener is not a deployment service. Protocol version negotiation, TLS,
structured timeout reporting, idempotent mutations, multi-command routing and
cross-platform/browser transport choice remain uninvestigated application decisions.
No production transport, security or portability claim is made.

## Executed evidence and reproduction

Environment: Rust `1.99.0 (b940084d7 2026-09-28)`, Cargo
`1.99.0 (5f94df478 2026-08-27)`, Linux `7.0.0-38-generic`, x86_64,
Intel Core i5-13600K. Tokio `1.53.2`, `tokio-util` `0.7.19`, futures-util `0.3.34`;
no dependency version upgrade was required. Tests use the debug test profile and
Tokio current-thread runtime, with `--test-threads=1` ensuring one owned listener
at a time. All traffic uses fresh native loopback sockets without TLS.

Commands from the repository root:

```bash
cargo test -p yetlet-custom-action-transport --offline -- --nocapture --test-threads=1
cargo build --release -p yetlet-custom-action-transport --offline
cargo fmt -p yetlet-custom-action-transport -- --check
cargo clippy -p yetlet-custom-action-transport --offline --all-targets -- -D warnings
python3 scripts/check_docs.py
python3 scripts/check_samples.py
git diff --check
```

[Retained direct proof output](../examples/custom-action-transport/proof.log) records
**7 passed, 0 failed, 0 ignored, 0 filtered out**, and zero doctests. It includes
the successful typed executor path; actual request/reply bytes; malformed JSON and
unknown event-field rejection; overflow error mapping; oversized/truncated frames;
malformed/wrong-shaped/missing replies; refused connection; 50 ms deadline and
caller cancellation with peer-observed EOF. Every peer task was joined, each client
was dropped and all 12 peer listener addresses refused reconnect. The separate
refused-connect fixture also dropped its thirteenth listener. No client driver, browser,
watcher, persistent server or external resource was started.

The typed successful request's single observed connect-to-decoded-result duration
was **194,230 ns** for 22 request and 26 reply payload bytes. This raw diagnostic
observation excludes cleanup, is not a benchmark or throughput comparison, and has
no warmup, repeated-run confidence interval or numerical acceptance threshold.
Malformed request payloads were 8 and 36 bytes; their error replies were 43 bytes.
The oversized frame declares 1,025 bytes, and the truncated frame declares 10 but
sends one. Raw output retains these exact inputs and outcomes.

Native compile/release build, format, strict Clippy, documentation/reference/backlog
and sample catalog checks pass. Run-local logs and implementation source hashes
are retained under `.pipeline-output/delivery-20261012-docs-research/outputs/T04-*`.
**Browser/WASM validation:** not run; native TCP is not a browser adapter.
**CSP validation:** not run/not applicable to this non-HTTP native protocol; no
HTML or response headers are generated. Historical broad Rust/browser/Docker/CI
evidence was not rerun or transferred to this source.

YT-024 is Done only for this bounded optional network adapter proof and adoption
decision. YT-029/YT-021 remain Done at their bounded research scopes, YT-020 remains
Blocked, YT-068 remains In progress and established YT-030/YT-065/YT-066/YT-067
bounds are preserved. YT-022/YT-023 remain Deferred until their tasks start. The
next unblockable research task is **T05 / YT-022**, constrained handler identity
and capture resumability. No release activation is implied.
