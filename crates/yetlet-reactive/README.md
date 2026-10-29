# yetlet-reactive

**Experimental `0.9.0-alpha.2`, unpublished.** Requires Rust 1.99 or newer. This is one of Yetlet's sixteen library candidates; use the local checkout rather than assuming a registry release. APIs are pre-public, and bounded prototype verification is not production qualification.

`yetlet-reactive` provides an optional synchronous local message/reducer/commit graph. It is renderer-neutral and has no dependencies, I/O or server/browser runtime. Add it only to surfaces that need explicit local update ownership.

## Use the implemented graph

Construct `LocalGraph::new(state, reducer, commit)`, obtain a weak read/dispatch handle with `handle()`, and call `dispatch(message)` or `batch(messages)`. Both `graph.snapshot()` and `handle.snapshot()` return an owned clone of the published state. `subscribe(observer)` returns an owned subscription: retain it while notifications are needed.

Read through the weak handle when a callback needs state without capturing the
strong owner. Before any dispatch, snapshot reads the initial state without
rendering or notifying. Inside a reducer/commit callback it reads the previous
published state; inside an observer it reads the newly published state. A
re-entrant dispatch queues work, so an immediate read does not imply that queued
work has completed. Snapshot does not flush or run callbacks itself.

The internal borrow ends before `State::clone`; arbitrary clone code can
re-enter the graph and controls its own side effects. Clone may share
interior-mutable application data, so this is not a deep-copy or freshness
guarantee after return/await. The handle stays weak; a returned value adds no
graph ownership beyond its own application data. The
[YT-105 snapshot slice](../../docs/YT-105-GRAPH-SNAPSHOT.md) records the bounded
contract and verification, including Todo/Admin adoption. Todo's local executor
reads the handle after dispatch; Admin reads it before constructing a save.
Their separate committed-state mirrors have been removed. Initial rendering
and the supplementary browser Tasks caller remain unchanged.

The complete [Todo Browser caller](../../examples/todo-browser/src/lib.rs) constructs the graph, reduces typed Todo commands, commits keyed DOM changes and owns its disposal lifetime. Follow [Todo's browser setup](../../examples/todo-server/README.md) or, from the repository root:

```bash
cargo xtask build browser
cargo run --locked -p yetlet-todo-server
```

Open `http://127.0.0.1:3000/app`; the local toggle uses the graph and the server Create action uses a separate HTTP executor. Browser build prerequisites are in the sample guide. A native caller can use the graph without any browser dependency.

## Ownership and failures

`State` must be `Clone + 'static`; messages, error values and captured callbacks also require the documented static lifetimes. The graph uses `Rc`/`RefCell` and is local to one thread. One non-cloneable strong owner controls its lifetime; `GraphHandle` and `Subscription` are weak. Dropping the owner or calling `dispose()` releases owned captures; dispatch and snapshot afterwards return `ReactiveError::Disposed`. Snapshot returns `ReactiveError::Faulted` for a live faulted graph, with disposal taking precedence. Dropping or unsubscribing a subscription removes its observer.

Messages are FIFO, with one successful commit/notification per pass. Dispatch during commit/observation queues a later pass. The default limit is 64 passes; `with_pass_limit` requires a positive limit and panics on zero. Reducer errors keep the prior state and remain recoverable. Commit errors and pass exhaustion fault the graph; dispose and rebuild it before reuse. External effects performed by callbacks are not transactionally undone.

This is not dependency-tracked signals, automatic effects, async scheduling, generic forms or resumability. Retained [M7g evidence](../../docs/M7G-VERIFICATION.md) and [the lifecycle decision](../../docs/adr/0007-bounded-keyed-dom-and-local-graph.md) describe the bounded selection.

## License and compatibility

Dual-licensed under [MIT](../../LICENSE-MIT) or [Apache-2.0](../../LICENSE-APACHE). The privately prepared `0.1.x` policy preserves documented behavior and Rust 1.99 in patches; public activation remains blocked. See [compatibility](../../docs/COMPATIBILITY.md), [API classifications](../../docs/API-CONTRACTS.md) and [developer guides](../../docs/README.md). Links to companion packages and records assume this checkout; they do not imply public hosting or distribution.
