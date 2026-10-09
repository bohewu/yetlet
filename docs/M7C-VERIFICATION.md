# M7c Verification — Explicitly Owned Browser-Local Reactive Graph

**Date:** 2026-10-09
**Task:** YT-036 / M7c
**Status:** Implemented and locally verified in the isolated `yetlet-keyed-bindings` prototype. It is not a stable signals API, scheduler, production renderer, or remote UI transport.

## Scope

M7c adds a small synchronous local update graph to the M7b keyed/static-slot fixture. The graph exists to validate ownership, batching, re-entrant dispatch, cleanup, and error boundaries before Yetlet considers a public reactive API.

The implementation remains in `examples/keyed-bindings/src/reactive.rs`; Todo Browser and Search Island do not depend on it.

## Ownership model

`LocalGraph<State, Message, Error>` is the single strong owner of:

- the last committed state;
- one reducer;
- one renderer/commit callback;
- the FIFO message queue;
- observer callbacks;
- fault/disposal state.

`GraphHandle` and `Subscription` contain only weak references. They cannot keep the graph, reducer, renderer, state, or observer captures alive. Dropping or explicitly disposing the owner clears queued messages, observers, reducer and commit captures. Dropping or unsubscribing a `Subscription` removes that observer without affecting the graph.

A native drop-probe test verifies that state, reducer, renderer and observer captures are all released when the owner is dropped while weak handles/subscriptions remain.

## Update and batching semantics

- `dispatch(message)` enqueues one typed local message.
- `batch(messages)` enqueues messages in deterministic FIFO order.
- One outer batch reduces all queued messages against one cloned state draft.
- The renderer/commit callback runs once for the final draft.
- State is published and observers are notified once only after a successful commit.
- An empty batch performs no commit or notification.

The M7c browser fixture verifies that a three-message batch produces:

```text
messages = 3
passes  = 1
commits = 1
observer notifications = 1
```

No DOM event, state snapshot, or callback crosses a network boundary.

## Re-entrant dispatch

A reducer, commit callback, or observer may dispatch another local message. The graph does not recurse into another render while the current pass is active. New messages remain queued and execute in the next FIFO pass.

The browser fixture verifies one observer-triggered message produces two messages, two passes and two commits. A configurable pass limit defaults to 64; exceeding it clears the queue, faults the graph and rejects later dispatches. The native test uses a limit of three to prove bounded termination.

## Failure behavior

| Failure | State behavior | Later dispatch |
|---|---|---|
| Reducer returns an error | Previous state and DOM remain committed; queued messages are cleared | Graph remains usable |
| Renderer/commit returns an error | Draft is not published; queue is cleared; existing keyed DOM identity remains | Graph becomes faulted |
| Re-entrant pass limit exceeded | Already completed passes remain committed; pending queue is cleared | Graph becomes faulted |
| Owner disposed/dropped | Captures and observers are released | Weak handles return `Disposed` |

The detached browser failure fixture intentionally creates a duplicate-key draft. M7b reconciliation rejects it before mutation, the existing keyed row remains the same DOM node with the same key, and the graph remains faulted afterward. This demonstrates the intended fail-closed integration path; arbitrary third-party commit callbacks are still responsible for their own atomicity.

## Browser integration

The isolated keyed-list demo now uses the graph as follows:

1. Native SSR produces the fixed M7b list.
2. `BoundKeyedList` binds the existing SSR nodes.
3. Row listeners dispatch typed `DemoMessage` values through a weak `GraphHandle`.
4. The reducer updates an owned state draft.
5. One commit callback reconciles the keyed binding.
6. Explicit subscriptions observe only successfully committed state.
7. `dispose_demo` disposes the graph and binding, removes listeners and releases captures.

The fixture stores one owner in a `thread_local!` slot solely to keep the WASM demo alive between exported calls. It is not framework-global state: remount first disposes the previous owner, and explicit disposal removes the owner and all propagation.

## Targeted evidence

Commands:

```bash
cargo clippy --locked -p yetlet-keyed-bindings --all-targets -- -D warnings
cargo test --locked -p yetlet-keyed-bindings --all-targets
cargo test --locked -p yetlet-keyed-bindings --doc
cargo check --locked -p yetlet-keyed-bindings --target wasm32-unknown-unknown
python3 scripts/build_keyed_bindings.py --release
python3 scripts/smoke_keyed_bindings.py --profile release
```

Observed targeted results:

- 17/17 native M7b/M7c tests passed, including thirteen reactive ownership/batching/error/teardown/drop-safety tests.
- One compiled authoring doctest passed.
- Strict package Clippy and wasm32 check passed.
- Chrome 155 verified batching, re-entrant scheduling, explicit observer removal, reducer recovery, faulted commit isolation, owner disposal and zero post-load HTTP/WebSocket traffic.
- Existing M7b SSR identity, keyed reconciliation, CSP and listener-disposal scenarios remained passing.

## Release artifact and isolation

| Field | Observed value |
|---|---:|
| M7b/M7c content digest | `150d17ce892eaebd` |
| WASM | 221,449 bytes; 64,483 local gzip bytes |
| wasm-bindgen JS module | 19,915 bytes; 4,256 local gzip bytes |
| External loader | 244 bytes; 157 local gzip bytes |
| Generated fixture HTML | 1,818 bytes |

These values describe the isolated prototype, not production transfer size or a bundle budget. Todo Browser and Search Island dependency graphs exclude `yetlet-keyed-bindings` and `yetlet-render-bench`. Production assets remain separately verified during final closeout.

## Final closeout evidence

The M7c source was frozen before one complete local validation sequence. A final artifact comparison initially used the wrong `dist/` path shape; that harness-only path assumption was corrected and only the comparison was rerun. All product builds, tests and browser regressions in the original sequence had already completed successfully.

| Final check | Observed result |
|---|---|
| `cargo fmt --all -- --check` | PASS |
| `python3 scripts/check_docs.py` | PASS — 24 English documents, 38 backlog items, links/Markdown/whitespace/DAG valid |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | PASS |
| `cargo test --workspace --all-targets --locked` | **86/86 unit and integration tests PASS** |
| `cargo test --doc --workspace --locked` | **5/5 doctests PASS** |
| wasm32 checks | Todo Browser, Search Island, M7a render benchmark and M7b/M7c prototype PASS |
| Existing Chrome regression | Chrome 155 Todo/SSR/Search/API/CSP E2E PASS on debug artifacts |
| Existing Firefox regression | Firefox 157 Todo/SSR/Search/API/keyboard E2E PASS on debug artifacts |
| Release Browser/Search rebuild | PASS; digest and every captured asset hash remain `3f3c5f074478c525` |
| M7a and M7b/M7c release builds | PASS |
| Isolated M7b/M7c Chrome smoke | PASS, including batching, re-entrancy, failure and disposal paths |
| Reviewer repair regressions | Nested-subscription cleanup, reducer-triggered disposal, iterator/state drops and reducer/commit failure cleanup all PASS without retained queue work |
| Production dependency isolation | Todo Browser/Search graphs exclude both M7 experimental crates |
| `git diff --check` and resource teardown | PASS; no browser/server/reviewer Job remained active before review dispatch |

The Chrome/Firefox rows describe the existing regression suites and their actual debug profile. Release compatibility is supported separately by byte-identical Browser/Search release assets; this report does not claim a newly executed release-profile cross-browser matrix.

## Limitations and M7d boundary

- The graph is synchronous and single-threaded; it has no async effects, tasks, priorities, microtask scheduler or worker integration.
- It coalesces explicit message batches, not automatic field-level dependency tracking.
- Observers are infallible callbacks; panics are not caught.
- A custom commit callback can still partially mutate external state before returning an error. M7c only proves the fail-before-mutate path through the bounded keyed renderer.
- The component shape remains the fixed M7b keyed list.
- No heap profiler or browser-GC metric is claimed. Lifetime evidence is structural: weak handles, deterministic capture drops, listener removal and post-disposal behavior.
- M7c uses Chrome for the isolated reactive fixture. Full Chrome/Firefox focus, selection, form-state and listener-lifecycle coverage remains YT-037.
- No stable `Signal`, `Effect`, `Resource`, macro or renderer API is selected.

The next task is **YT-037 / M7d**: expand keyed lifecycle and form-state verification across Chrome and Firefox before M7e considers any public API decision.
