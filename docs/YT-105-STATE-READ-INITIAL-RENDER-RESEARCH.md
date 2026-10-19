# YT-105 — committed-state reads and initial rendering

Date: 2026-10-19. Inspected source: `6e782bf`, after the independently reviewed
[form-binding comparison](YT-105-FORM-BINDING-RESEARCH.md) was committed.
This is the second selected YT-105 research slice, not implementation or an
accepted public API. Independent Reviewer `/root/yt105_state_read_review_oct19`
PASS has zero issues or required followups.

## Recommendation

Propose one small addition for a separately authorized implementation:
`GraphHandle::snapshot()`, with the same owned-clone result and error policy as
the existing `LocalGraph::snapshot()`. Both Admin and Todo maintain a separate
state mirror because callbacks receive only a weak dispatch handle. A weak
read capability would let those callbacks query the graph's published state
without maintaining that second copy or capturing the strong owner.

For initial rendering, recommend an explicit application render/setup step,
sharing its rendering function with later commits where practical. Keep
`LocalGraph::new` free of rendering and observer side effects. Do not add an
automatic initial commit, constructor option, scheduler or second public
initialization method for these consumers.

These are independent decisions: a weak snapshot does not render the first
screen, and an explicit first render does not solve callback state reads.

## Actual consumers and repeated work

Primary consumers are the Admin-hosted
[board triage workspace](../examples/board-triage/src/browser.rs) and the
existing [Todo Browser](../examples/todo-browser/src/lib.rs). The YT-104
[browser task starter](../examples/browser-tasks/src/lib.rs) corroborates both
patterns; it is not substituted for either primary consumer.

| Concern | Admin triage | Todo Browser | Supplementary browser Tasks |
| --- | --- | --- | --- |
| Strong owner | Runtime retains the one `LocalGraph` | BrowserRuntime retains the one `LocalGraph` | Runtime retains the one `LocalGraph` |
| Why a callback reads state | Save reads selected dirty draft and pending status before constructing one versioned HTTP command, lines 345–350 | LocalGraphExecutor dispatches a command and returns the committed snapshot, lines 33–43 | Edit reads the selected task before prefilling its native editor, lines 252–259 |
| Mirror setup | `committed` plus `current` clone, lines 264–265 | Mirror seed at 230, closure capture at 236 and executor field at 36/255 | Mirror seed/alias at 147–148 |
| Mirror publication | Commit callback clones Workspace into the mirror after rendering, line 274 | Commit callback clones TodoSnapshot into the mirror after successful reconcile, line 248 | Render closure clones Tasks into the mirror, line 169 |
| Mirror read | Clone at line 346 | Clone at line 42 | Read at line 256 |
| Initial rendering | Adopt/preflight SSR rows, then dispatch `Msg::Filter(String::new())` after binding setup, lines 365–368 | Mount `browser_page` and bind its list before constructing the graph, lines 223–232 | Call the same render closure once before moving it into `LocalGraph::new`, lines 150–173 |

Each of the three consumer commit/render callbacks contains one explicit
application-state clone for its mirror. Todo additionally exposes that mirror
as an executor field. These are source-level duplication observations, not
measurements of allocation bytes, latency or bundle savings. Snapshot reads
would still clone `State`, and the graph's own draft/observer copies remain.

The Admin initial Filter is valid current behavior, not a demonstrated defect.
Its [Workspace constructor/reducer](../examples/board-triage/src/lib.rs), lines
97–112 and 140, already starts with an empty query. That dispatch currently
creates a reduction/commit pass even though the desired purpose is initial DOM
synchronization. No consumer in this comparison installs a graph observer.

## Existing graph semantics that constrain the proposal

The source of truth is
[`crates/yetlet-reactive/src/lib.rs`](../crates/yetlet-reactive/src/lib.rs):

| Existing surface | Observed behavior |
| --- | --- |
| Owner and handle, lines 98–109 | LocalGraph is the one non-Clone strong owner; GraphHandle stores Weak and currently exposes dispatch/batch only |
| `new` / `with_pass_limit`, lines 159–188 | Store the initial state, callbacks and empty queues; do not invoke the commit callback or notify observers |
| Owner `snapshot`, lines 196–213 | Reject disposed/faulted graphs, retain the published state Rc under a short borrow, then call State::clone after ending that borrow |
| `subscribe`, lines 215–247 | Register an owned subscription; no immediate replay of the initial state |
| `batch`, lines 284–322 | Empty batches do nothing; dispatch during an active flush queues work and returns without completing a nested flush |
| Flush, lines 382–504 | Reduce a draft; call commit with the draft; after success and a disposal check, publish it; only then notify observers. Queued work proceeds in a later bounded pass |
| Errors and disposal | Reducer error preserves the prior published state and remains recoverable; commit failure or pass exhaustion faults the graph; explicit disposal/owner drop releases captures and invalidates future weak-handle work |

Do not infer that the word "commit" makes the renderer callback's argument
already published. Inside that callback, graph storage still contains the
previous state. This ordering must remain unchanged by a read convenience.

## Proposed minimal read contract

Illustrative signature only; **this method does not exist in this checkout**:

```rust
// Proposed method on GraphHandle<State, Message, Error>.
pub fn snapshot(&self) -> Result<State, ReactiveError<Error>>;
```

Use the existing State: Clone bound and existing errors. The method would:

1. Temporarily upgrade the existing Weak reference; return Disposed if the
   owner is gone, and preserve disposed-before-faulted checks on a retained
   allocation, matching existing owner semantics.
2. Read only the published state. Retain its internal state Rc under a short
   RefCell borrow and end that borrow before calling user-defined State::clone,
   matching the existing owner method. Do not expose Inner, a Ref, the internal
   Rc, a mutable reference or a strong LocalGraph owner.
3. Return the owned clone. Do not flush queued messages, invoke the reducer or
   renderer, register an observer, start I/O, or change counters/state.

The [standard Weak contract](https://doc.rust-lang.org/std/rc/struct.Weak.html#method.upgrade)
supports a temporary upgrade without changing what the stored handle owns.
Do not retain a new strong graph capability in callbacks. Releasing the graph
borrow before State::clone matters because arbitrary clone code can re-enter
the graph; conflicting runtime borrows can panic under
[RefCell's contract](https://doc.rust-lang.org/std/cell/struct.RefCell.html#method.borrow).

### Publication, failure and lifetime

Timing rows assume an active, healthy graph; the failure/disposal rows take
precedence when those conditions apply.

| Read timing | Required result of the proposed method |
| --- | --- |
| After construction, before any dispatch | Clone the stored initial state; render/commit/observer counts remain zero |
| Outside a flush after an outer dispatch returns Ok | Clone the final published state after that outer flush, including any completed later passes |
| Inside a reducer or commit callback | Clone the previous published state, never the in-progress draft |
| Inside an observer | Clone the state published before notification; work just queued by a nested dispatch may not be applied yet |
| Recoverable reducer error | Clone the previous valid published state |
| Commit failure or re-entrant pass exhaustion | Return Faulted, matching owner snapshot; do not return an apparently healthy mirror |
| After explicit disposal or owner drop | Return Disposed, even if weak handles or earlier snapshots still exist |

A snapshot is a read at the point the published state reference is acquired.
It is not a promise that the state is still current at return or after an
await, and it does not guarantee that a particular queued command has executed.
Existing re-entrant dispatch semantics remain: enqueueing from an observer and
immediately reading may see the current pass rather than the queued update.

An earlier returned snapshot may outlive the graph. Its data cloning follows
[State's Clone implementation](https://doc.rust-lang.org/std/clone/trait.Clone.html),
which need not be a deep copy of arbitrary interior-mutable application data.
The proposal adds no immutable-state, rollback, remote-consistency or automatic
lifetime guarantee beyond the existing owner snapshot contract.

### Bounded future adoption

- Admin could replace its mirror read with a weak snapshot before checking
  pending/dirty state. A read failure must stop that save path rather than
  starting HTTP from stale cached data. Keep explicit Begin, item ID/version,
  CSRF, existing feedback and queued/in-flight runtime-lifetime checks.
- Todo's local executor could dispatch, then return the weak snapshot with its
  existing graph-to-ActionError mapping. Its current callback is outside an
  active flush; no general "dispatch-and-read my result" transaction is added.
- Tasks could read through the weak handle before opening its native editor.
  Its separate native editing ID/value and focus behavior remain app-owned.

These are migration opportunities, not changes made by this study. They would
remove the mirror setup/publication/capture plumbing at the named sites. Exact
line-count or performance reductions require an actual separately selected
implementation and are not claimed here.

## Initial render: explicit application ownership

Prefer the existing Tasks pattern: keep the initial value available, explicitly
render/synchronize the initial screen, then move the value and render callback
into the graph. Register listeners and expose readiness only when application
setup succeeds. A shared application render function or closure is sufficient;
there is no requirement to create a new public helper.

For Admin, a later application change could call its existing Dom::render on the
initial Workspace before moving Dom into the commit callback, replacing the
synthetic initialization Filter pass. Its typed SSR-row preflight and native
fallback remain; this would be explicit DOM synchronization, not a new hydration
protocol. Todo already creates its page before constructing its graph and need
not acquire a redundant initial commit. Rendering failure should fail mount
without publishing a ready runtime; this does not promise transactional rollback
of partially modified DOM.

| Alternative | Decision |
| --- | --- |
| Document explicit setup/render and retain the existing constructor | Recommended; accommodates SSR adoption and client-created pages without hidden reduction/notification |
| Make new automatically render | Not selected; changes existing construction side effects and adds failure semantics to a currently infallible constructor |
| Add an auto-render option or another constructor | Not justified by these callers; still must define callback readiness and error/publication order |
| Add a refresh/initial-commit method or dispatch an empty batch | No new method selected; empty batch currently does nothing, and a commit method would need new queue/error/re-entrancy semantics |
| Keep Admin's explicit initial Filter | Valid compatibility baseline if no app change is chosen; the new snapshot proposal does not require changing startup |

An initial render is not an initial graph commit or observer notification.
Do not silently change subscribe to replay initial state. Retain the explicit
ownership and synchronous FIFO model in
[ADR 0007](adr/0007-bounded-keyed-dom-and-local-graph.md).

## Read alternatives and tradeoffs

| Alternative | Assessment |
| --- | --- |
| Keep app mirrors | Valid current implementation; duplicates publication/capture bookkeeping at two primary consumers and the supplementary starter |
| Update mirrors through subscribe | Still needs a mirror plus retained subscription and explicit initial seeding; not a simplification of this read need |
| Capture Rc<RefCell<LocalGraph>> in every event | Changes strong ownership and introduces borrow/lifetime coupling; unnecessary when a weak clone read can reuse owner semantics |
| Add a borrowed with_state callback or expose Rc/Ref | Saves some cloning in theory but introduces callback re-entrancy/borrow and resource-lifetime policy; no demonstrated need for that larger contract |
| Add weak snapshot using existing errors | Recommended candidate: one method, existing naming/clone/error behavior, no registry or scheduling machinery |

For a later implementation, share the read logic internally between owner and
handle if that is the smallest way to keep their semantics identical. This is
not a request for another reader type, provider system, DI container or cache.

## Minimum future verification

Existing tests in yetlet-reactive already cover successful batches, later-pass
re-entrancy, reducer recovery, commit faulting, capture release and weak-handle
disposal (lib.rs 520–643, 722–744 and 883–943). They were inspected, not rerun as
proof of a method that does not exist. A later selected method needs only
focused native tests at this existing seam:

| Scenario | Required evidence |
| --- | --- |
| Owner/handle read parity | Equal initial/post-dispatch clones, no reducer/commit/notification from reads, and an old ordinary-data snapshot stays unchanged after a later update |
| Phase visibility | Reads in reducer/commit see prior state; observer sees published state; queued observer dispatch is not falsely reported as already applied |
| Failure parity | Recoverable reducer error keeps the prior state readable; commit failure/pass exhaustion return Faulted from both entry points |
| Lifetime | Explicit disposal/drop rejects later reads; retained weak handles do not preserve reducer/renderer captures; already returned data remains usable |
| Borrow boundary | A custom State::clone that re-enters a harmless graph operation does not inherit an active snapshot RefCell borrow; do not assume all Clone implementations are pure |

If consumer migration is separately selected, reuse the current Admin save and
Todo local-action checks to verify exactly one command/save, retained drafts and
existing disposal/error feedback. If initial rendering changes, check one first
render, no manufactured business action/observer replay, correct mount failure
and repeated disposal/remount. Existing browser journeys suffice; do not build
new validation infrastructure or a platform/input matrix for this research.

## Verification and remaining scope

This slice changes documents only. Verification is source/control-flow inspection,
comparison with the Rust standard-library reference, scoped local-link/whitespace
checks and `cargo xtask check docs`. No new test, API, renderer, dependency,
application migration, browser run or runtime benchmark was introduced.

Independent review passed; implementation remains subject to maintainer selection.
YT-105 remains Todo at broader scope: the separate HTTP CSRF/per-field feedback
comparison is still open. Neither this document nor its review authorizes
implementation, a research commit, release or deployment.
