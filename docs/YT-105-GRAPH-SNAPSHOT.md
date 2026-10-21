# YT-105 — weak-handle published-state snapshot

Date: 2026-10-19. Implementation follows the separately authorized minimal
contract from the reviewed
[state-read study](YT-105-STATE-READ-INITIAL-RENDER-RESEARCH.md).
Base: `0ce0ac7`, which commits the reviewed HTTP-feedback research.
Independent Reviewer `/root/yt105_graph_snapshot_impl_review_oct19` PASS has
zero issues or required followups and approves this bounded implementation as
verified and complete.

## Implemented contract

[`GraphHandle::snapshot`](../crates/yetlet-reactive/src/lib.rs) now exposes:

```rust
pub fn snapshot(&self) -> Result<State, ReactiveError<Error>>;
```

It uses the same existing State/Message/Error bounds and the same private read
function as `LocalGraph::snapshot`. The read temporarily upgrades the weak
handle, checks Disposed before Faulted, retains the published state reference
under a short borrow, ends that borrow, then calls State::clone. A failed weak
upgrade returns Disposed. No new error, owner, reader type or dependency is added.

| Read timing / state | Result |
| --- | --- |
| Before dispatch | Initial state; no reducer, commit or observer call |
| Reducer or commit callback | Previous published state, never the current draft |
| Observer | Newly published state; a just-queued re-entrant message may still be pending |
| After an outer successful dispatch | Final published state after its completed passes |
| Recoverable reducer error | Previous valid published state remains readable |
| Commit failure / exhausted pass limit | Faulted, matching the owner |
| Explicit disposal / owner drop | Disposed; retained weak handles do not retain callback captures |

Reading does not initiate a flush, observer replay, rendering, I/O or scheduling.
State::clone remains user code: it may re-enter the graph, have side effects or
share interior-mutable data. Snapshot observes the state reference acquired for
that read; it is not a freshness promise after cloning, return or await. Existing
owned data snapshots can outlive the graph without granting graph ownership.

The change is additive and remains Experimental under the existing
[API classification](API-CONTRACTS.md). It preserves
[ADR 0007](adr/0007-bounded-keyed-dom-and-local-graph.md); the graph's publication,
queue, callback and failure order has not changed. The
[crate guide](../crates/yetlet-reactive/README.md) documents callback use and limits.

## Focused verification

Native tests extend the existing crate's seams rather than adding infrastructure:

- Three new tests cover initial/updated owner-handle parity without callback
  side effects, retained ordinary-data snapshots, reducer/commit/observer reads
  including a queued later pass, and re-entrant custom State::clone from both
  read entry points with no active graph borrow.
- Existing reducer, commit-fault, pass-limit, explicit-disposal and weak-lifetime
  tests now assert handle read behavior. Fault-then-dispose verifies Disposed
  takes precedence; capture-release checks remain in place.

| Command | Result |
| --- | --- |
| `cargo test --offline --locked -p yetlet-reactive` | PASS: 16 unit tests; 0 doctests |
| `cargo clippy --offline --locked -p yetlet-reactive --all-targets -- -D warnings` | PASS |
| `cargo check --offline --locked -p yetlet-reactive --target wasm32-unknown-unknown` | PASS |
| `RUSTDOCFLAGS="-D warnings" cargo doc --offline --locked -p yetlet-reactive --no-deps` | PASS |
| `rustfmt --edition 2024 --check crates/yetlet-reactive/src/lib.rs` | PASS |
| `cargo xtask check docs`, scoped local links and `git diff --check` | PASS |

## Scope of the initial method slice

The 2026-10-19 implementation added the read method and its focused
tests/documentation. It left Admin, Todo and browser Tasks mirrors and
initial-render setup unchanged. The separately selected consumer adoption below
supersedes that migration limit for Admin/Todo only. The original method checks
did not claim browser, Windows or input-device qualification.

YT-105's three research comparisons stay Done at their recorded research scope.
The method's independent PASS above is retained separately from the consumer
review below. YT-106 research is independently unblockable if selected.

## Admin / Todo adoption — 2026-10-20

The owner subsequently selected the two real consumers:

- [Todo Browser](../examples/todo-browser/src/lib.rs): the local executor
  dispatches, then reads `handle.snapshot()` using its existing
  `graph_action_error` mapping. Its mirror field, initialization, capture and
  per-commit publication clone are removed. The renderer only reconciles DOM.
- [Admin triage](../examples/board-triage/src/browser.rs): Save selected reads
  `save_handle.snapshot()` before checking pending/dirty state and constructing
  the versioned command. A read error returns before Begin/HTTP. Mirror storage,
  capture and publication are removed; the initial Workspace moves directly
  into its owner. The existing initial Filter/render step remains unchanged.

Both inspected call sites read outside an active flush. This adoption does not
create a general dispatch-and-read transaction or change queued-dispatch
semantics. Save still checks Begin, CSRF, item/version and both runtime-lifetime
guards. The current HTTP executor, error messages and native no-JS routes are
unchanged. Browser Tasks is not migrated by this slice.

The two source changes add 3 and remove 11 physical lines in total. Each commit
callback loses one explicit application mirror clone; the graph's own clones
and the owned clone on snapshot read remain. This is source-level simplification,
not measured allocation, CPU, latency or bundle-size savings.

### Profiling decision

Do not add a profiling subsystem or performance gate for this migration.
Correctness/lifetime checks cover the selected behavior; they do not demonstrate
a speedup or prove the absence of performance issues. If a concrete slow action
or retention symptom is selected later, profile that bounded journey with
representative data and comparable release builds, separating local update work
from network time. The existing
[Chrome Performance panel](https://developer.chrome.com/docs/devtools/performance)
can record and analyze runtime bottlenecks. No CPU/heap trace, benchmark or
performance claim is part of this slice.

### Consumer verification

Linux, system Chrome 155.0.8059.84. Browser plugin/skill is not available;
the existing Playwright/system-Chrome Admin journey and repository Todo Chrome
smoke are used. Temporary script, synthetic SQLite fixture, screenshots and
Admin outcome JSON live outside the repository. Startup demo codes stay in
process memory and are not retained in evidence. No external IdP is used.

| Check | Result and boundary |
| --- | --- |
| `cargo check --offline --locked --target wasm32-unknown-unknown -p yetlet-board-triage -p yetlet-todo-browser` | PASS |
| `cargo test --offline --locked -p yetlet-board-triage -p yetlet-todo-ui` | PASS: 3 triage and 10 Todo shared-state/command tests |
| Wasm-target Clippy on both consumers, all targets, `-D warnings`; rustfmt on both source files | PASS |
| CLI `yetlet build --manifest-path examples/admin-dashboard/Cargo.toml`; frontend `npm run typecheck` | PASS; rebuilt the selected Rust/WASM, bindings, Vite assets and native host |
| `cargo xtask build browser` then `cargo xtask smoke browser-chrome` | PASS; fresh Todo assets, real local/remote actions, retained keyed identity, disposal/remount and late/queued action checks |
| Admin real Chrome through CLI run | PASS; per-item edits/filter/reorder without HTTP; one save with latest draft values; another draft editable while pending; real 409/422 retain drafts; Reset; reload/remount saves once |
| Admin lifetime / fallback | PASS; disposed listeners inert, late completion does not alter retired DOM, same-stack Save/dispose starts zero POSTs; no-JS links still reach native editor |
| Admin visible UI / assets | PASS at 1440×1000 and 390×844; correct page/nonblank/no overlay, no horizontal overflow, retained row/editor identity, ordinary SSR CSP and WASM MIME; screenshots inspected |
| Admin console | PASS; zero captured application exceptions/errors or warnings, with expected 409/422 resource diagnostics excluded |
| Cleanup | PASS; Admin CLI received SIGINT, printed database-drain marker and closed its port. Todo's existing harness recorded native SIGTERM, Chrome exit 0, closed listeners/streams, reaped owned processes and removed browser profile |

Admin's CLI result remains the Linux signal exit `{code: null, signal: SIGINT}`;
do not relabel it native exit 0. Todo's browser smoke result is exit 0 while its
owned server cleanup retains native signal 15. The existing Todo receipt is
under `target/m12/yt085/browser-chrome-dSWXtV/`; the Admin run-local evidence is
under the task's temporary directory. There is no new Windows, OS IME/assistive
technology, production or performance qualification, and no test infrastructure
or dependency was added. Earlier passing qualification is retained.

Independent Reviewer `/root/yt105_consumer_snapshot_review_oct20` PASS has zero
issues or required followups and approves the bounded Admin/Todo adoption as
complete. The original method source remains byte-identical to its earlier
review. No further profiling or verification is required for this scope; no
performance improvement is established. Initial-render/HTTP/form proposals
and any further consumer migration remain separate selections.
