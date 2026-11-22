# YT-286-A / B1 / B2 / B3 — private Host lifecycle prototype

This unpublished, independent workspace tests the layered direction in
[ADR 0014](../../docs/adr/0014-layered-host-resource-lifecycle.md).
A established the default Host path. B1/B2 adds an ordered final-release
contract to the existing Composition engine and uses it here for managed
resources. The separately authorized B3 integration applies that contract to
real Web/SQLite work. The facade remains private research; ADR 0014 and the
broader public adoption gate remain open.

B1/B2 adds opt-in source API under the existing `composition` feature, including
`ReleaseCompletion`, typed resources/groups and a sealed `ResourceParts`
support bound. That core extension adds no dependency or feature and preserves
legacy Local/Worker behavior and the direct Axum API. B3 adds SQLite and temporary
file support only as this private workspace's dev-dependencies. The normal
Application API and SDK/core dependency selection remain unchanged. Git commit,
push and release are outside this authorization.

## Normal entry

The compiling spelling selected for this proof is `Application::new()`:

~~~rust
use axum::{Router, routing::get};
use yetlet_host_lifecycle_research::{Application, WebHost};

let app = Application::new()
    .with_host(WebHost::new(
        Router::new().route("/", get(|| async { "public" })),
    ).named("public"))
    .with_host(WebHost::new(
        Router::new().route("/", get(|| async { "admin" })),
    ).named("admin"));

let shutdown = app.shutdown_handle();
// Connect this handle to the application's existing shutdown trigger.
let report = app.build()?.run()?;
~~~

The two defaults request independent loopback listeners on OS-selected ports.
Use `bind(address)` to choose a listener and `status().endpoint()` to inspect
the acquired address. An address is not proof of readiness or liveness.
`named` only improves error/report attribution.

Constructors, registration and `build` do not bind or spawn. Build validates
all Host descriptors before acquisition. `run` owns startup and terminal
cleanup; an explicit shutdown request, observed activity completion or startup
failure initiates coordinated stop. It returns a `RunReport` or `RunFailure`
only after the original activities have joined and ordered resource release
has finished. A terminal cleanup error is still an application failure.

There is no hidden Ctrl-C handler, forced termination, thread pool or process
supervisor. Integrate `ShutdownHandle` with a caller-owned trigger.
The [two-Web example](examples/two_web_hosts.rs) performs real requests against
both listeners, requests shutdown, and joins its client.

## Advanced resource customization

Custom Hosts implement `Host` and return the same
`ResourceGroup::new(label, activity)` used by WebHost.
Both resource operations augment default Web lifecycle management:

~~~rust
let host = WebHost::new(router)
    .with_resource(passive_value)
    .with_managed_resource("state", inert_descriptor);
~~~

| Operation | Ownership and acquisition | Final release |
|---|---|---|
| `with_resource(value)` | Retains an already supplied value; a borrow remains caller-owned | Ordinary infallible synchronous RAII |
| `with_managed_resource(label, descriptor)` | Retains an inert descriptor; calls `acquire` only after installation in the lifecycle root | Explicit cooperative, possibly fallible release |

A caller may construct an already acquired passive value; that caller work is
not deferred. An unstarted or rejected plan drops supplied passive values
normally. A managed descriptor must be inert before acquisition: its pre-start
Drop cannot require fallible or asynchronous cleanup. General adoption of an
already-live managed resource is not implemented.

The private managed protocol is deliberately small:

~~~rust
pub trait ManagedResource {
    fn acquire(&mut self) -> std::io::Result<()>;

    fn release_within(
        &mut self,
        budget: std::time::Duration,
    ) -> std::io::Result<ReleaseCompletion<std::io::Error>>;
}
~~~

Acquire resources in registration order, then start their dependent activity.
Every successful or partial acquisition must be stored in the descriptor
before another fallible step. Acquisition failure stops later acquisition and
retains the attempted descriptor for rollback. A descriptor whose acquisition
was never attempted skips its release callback and undergoes ordinary Drop.
Cancellation is checked between acquisitions and before activity startup.

The activity descriptor also enters its owned core slot before startup.
`Activity::start` must retain acquired handles before readiness can fail.
Actual readiness follows acquisition; all Hosts must pass readiness before
business admission opens. A factory remains responsible for anything it fails
to transfer. Callbacks and destructors must not panic.

### Release outcomes

| Callback result | Meaning | Owner and earlier dependencies |
|---|---|---|
| `Ok(ReleaseCompletion::Pending)` | Cleanup is unfinished | Retain the same owner and retry later |
| `Err(error)` | This progress attempt failed; cleanup is still unfinished | Record the error, retain ownership and retry |
| `Ok(ReleaseCompletion::Released(Ok(())))` | Cleanup finished successfully | Drop the resource value and advance |
| `Ok(ReleaseCompletion::Released(Err(error)))` | Cleanup finished with a terminal error | Record once, drop the value and advance; never retry it |

`Released` promises that no outstanding cleanup operation can still use the
resource. A stop request or a failed polling attempt cannot make that promise.
Completion and application success remain separate. The first observed error
stays primary; later errors are retained as secondary evidence. This does not
infer causal order between simultaneous failures. Original `io::Error` values
remain behind local `Rc`, including after terminal root destruction.

### Ordering and retained ownership

The same Composition root coordinates admission closure, legacy Local
shutdown, worker stop requests and original joins. Only after every admission
gate is closed, every legacy Local is stopped and all original workers have a
terminal Join does it enter final release.

Final release traverses attachments in reverse order. A later unfinished
resource or group blocks earlier dependencies. Within each facade group, the
activity value is destroyed first, then resources are finalized in reverse
registration order. Each resource value is dropped immediately after its
terminal release, before an earlier finalizer may run. All application joins
precede this phase; there is no claim that every group's activity destructor
runs before any other group's resources.

Core `with_group` is structural nesting, sharing the root's state and error
ledger. It is not another scheduler. The ordered contract permits later
resources/groups to depend on earlier ones; it conservatively serializes their
release. It does not construct arbitrary references between owned fields,
provide a dependency-injection registry, schedule a DAG, or progress an earlier
sibling past a later Pending finalizer.

Ordinary Group Drop destroys its contained parts in reverse attachment order,
without running lifecycle callbacks. A built-but-never-started facade therefore
keeps the passive order: activity, last dependency, earlier dependencies. This
ordinary destruction is not a substitute for driving live managed cleanup.

Core `ManagedWorker` retains its coordination value for inspection. Its final
callback must retire any payload whose destructor needs earlier resources.
This facade stores the activity in `Option<A>` and takes it in that callback.
Core `Resource` physically drops its value on terminal release.

Local `Rc` and non-static borrows remain supported. No root-wide `Send`,
`Sync` or `'static` bound is introduced. Web's actual Router keeps its existing
Axum/thread requirements.

### Advanced driving

~~~rust
let mut app = Application::new().with_host(host).build()?;
match app.start() {
    StartStatus::Started => { /* drive caller work */ }
    StartStatus::Stopping => { /* inspect app.primary_failure() */ }
}
app.request_shutdown();
match app.shutdown_until(deadline) {
    Completion::Pending => { /* retain this SAME app and continue later */ }
    Completion::Complete => { /* app.report() is now available */ }
}
~~~

`start` retains errors in the original owner and requests rollback. It does
not return a cleanup-bearing error that `?` could discard. Pending retains
every unfinished owner and its earlier dependencies; already terminal values
may have been released. The terminal report is exposed only after the remaining
coordination root is destroyed.

Dropping a live/Pending advanced owner does not run managed release or join.
Continue driving that owner or use terminal-only `run`. An uncooperative
activity or finalizer can keep `run` waiting indefinitely. Deadlines are
cooperative: one absolute deadline is shared across nested observation and
release, each callback receives the current remaining budget, and a zero-budget
callback may perform a nonblocking progress check. Callbacks can overrun their
budget; no hard timeout or panic containment is provided.

The protocol permits an adapter to retain and drive an original asynchronous
cleanup operation; it does not create an executor or automatically poll arbitrary
futures. The [managed resource tests](tests/resources.rs) retain one local,
borrowing cleanup Future across Pending, then finish and drop it once. The
adapter must supply the actual polling/wake or runtime integration and must not
poll a completed Future again. See the [Rust Future contract](https://doc.rust-lang.org/std/future/trait.Future.html).

## Web readiness, admission and execution

Each WebHost uses the existing `HttpWorker`: one owned service thread with a
Tokio current-thread runtime. Two WebHosts therefore own two service threads
in this prototype. This is an adapter default, not the definition of Host.
No additional shared pool or process placement is implemented.

Default readiness is a real HTTP GET / returning 200 within two seconds.
Select an existing read-only route and cooperative budget with `readiness_probe`.
The prototype never inserts a hidden health route.

During a Web readiness probe, only GET requests to that exact route may bypass
its closed admission gate. After that probe, the exception closes. Business
admission opens after all Hosts pass readiness. Failed startup skips later
acquisition and closes all gates before shutdown signals. Observed cancellation
at startup boundaries also initiates stop.

Requests that passed the atomic gate before closure are admitted work.
Graceful shutdown lets them finish while dependencies remain owned. New
requests receive 503 if a listener is still serving during a failed peer
admission fence; connection refusal is possible after listener shutdown.

Readiness failure retains the acquired HttpWorker. A second bind failure keeps
the first original worker for rollback and does not close an external listener.
`WebStatus::joined` becomes true only for an actually acquired worker's Join.

## B3 — real Web and SQLite resource integration

The bounded integration uses the existing managed-resource path without changing
the normal Application API. Its source is the [finite Web/SQLite example](examples/web_with_sqlite.rs),
the [SQLite resource adapter](examples/support/sqlite_resource.rs), and the
[HTTP/SQLite integration tests](tests/sqlite_web.rs). SQLite support is confined
to this unpublished research workspace's [dev-dependencies](Cargo.toml), using
bundled SQLite. It adds no mandatory database dependency or SDK/core feature.

The descriptor remains inert through construction and build. Acquisition starts
one SQLite worker and immediately stores its original JoinHandle, before any
subsequent readiness operation can fail. The worker owns the Connection; the
Router receives a cloneable typed client backed by one shared command sender.
A bounded 32-entry channel carries insert/count operations, with Tokio oneshot
replies. Insert values are parameterized. The client exposes no arbitrary SQL
interface, Connection clone or independently retained command sender.

The lifecycle separates registered Host activity from resource-owned work:

1. Web admission closes and already admitted HTTP work drains while SQLite is
   still available. The original registered Web worker joins.
2. SQLite final release closes the shared command sender. It does not need to
   enqueue a shutdown message into the bounded data queue.
3. The descriptor observes and joins its original SQLite worker. Until that
   join is observed, Pending retains the original handle and prevents earlier
   dependencies from being released.
4. If explicit Connection close returns a live Connection with an error, the
   descriptor retains it for retry. Terminal release requires both the original
   thread join and completion of any retained connection cleanup.

The adapter reports distinct observations:

| Observation | Meaning |
|---|---|
| `is_open` | The adapter currently retains a live SQLite Connection. |
| `release_started` | The resource's release protocol has begun. |
| `joined` | The original SQLite thread handle has actually been joined. |

These observations are independent. Pending can mean that shutdown was requested
and completion has not yet been observed, even if the worker already exited.
It does not establish that SQLite is still open. A close retry can retain a live
Connection after the thread has joined. Similarly, failed acquisition can leave
the original thread handle pending even if setup already closed the Connection.

B3 acceptance covers these bounded cases:

| Case | Required evidence |
|---|---|
| Normal terminal run | Inert build, real HTTP insert/count, original joins and persisted readback through a newly opened Connection. |
| Admitted HTTP drain | SQLite remains usable by the admitted handler; post-Web cleanup Pending retains the original cleanup owner and earlier passive dependencies. |
| Later listener bind failure | Preserve the actual bind error as primary while an explicitly injected retryable cleanup fault is secondary; complete original cleanup on retry. |
| SQLite schema/readiness failure | A pre-created non-SQLite file causes an actual setup error; retain the original worker for rollback before Web starts. |
| SQLite open failure | A directory used as the database path causes an actual open error; preserve the original thread until join. |

The injected cleanup fault is a controlled test condition. The corrupt-file
schema/readiness and directory-path open cases exercise actual SQLite errors.
B3 acceptance requires executed focused tests, the finite example and independent
configured Reviewer evidence for the exact source. This section records the
integration contract; it does not itself claim a successful run or review.

## Deliberate proof boundaries

| Implemented source proof | Still open before broader adoption |
|---|---|
| One to four typed Hosts; one activity per Host | A broader normal facade and public release decision |
| Default WebHost and custom Hosts share Composition | Native/main-thread integration and Winit process-wide constraints |
| Passive values plus inert managed descriptors | Live managed-resource adoption and richer acquisition protocols |
| Post-Join, ordered, fallible/cooperative final release | Arbitrary dependency graphs and generalized live owner adoption |
| Bounded private Web/SQLite integration source | Public database adapters and qualification of other database/runtime choices |
| Structural nested core groups with one error ledger | Shared ancestor resource APIs and independent sibling progress |
| Default augmentation with both resource operations | `configure_resources` and typed driver overrides |
| Real HTTP readiness and original-worker ownership from A | Pool/process preferences, restart and optional-Host policy |

The arity macro supplies concrete Host tuples. Generic resource lists use the
sealed core `ResourceParts` proof bound; external code does not implement the
private traversal. There is no second admission, retry, join or release engine.
Legacy Local close-before-Join behavior is preserved. Resources that must
outlive workers use the new final-release category.

## Verification and evidence

The B1/B2 focused commands, from the repository root, are:

~~~sh
CARGO_TARGET_DIR="$PWD/target" cargo test --offline --locked -p yetlet --no-default-features --features composition --test composition
CARGO_TARGET_DIR="$PWD/target" cargo test --offline --locked --manifest-path examples/host-lifecycle-research/Cargo.toml --test resources
~~~

The core suite includes the original 13 cases and eight final-release cases.
The six new facade cases cover join/destruction order, reverse dependency
retention, partial-acquisition rollback, primary/secondary errors, terminal
failure, inert rejected plans, cancellation during acquisition, and one retained
local cleanup Future. Non-Web compatibility tests and compilation checks cover
the directly changed facade/core paths; exact executed commands and outcomes
belong to `.pipeline-output/m28-yt286b12-20261121/`.

The B3 focused commands are:

~~~sh
CARGO_TARGET_DIR="$PWD/target" cargo test --offline --locked --manifest-path examples/host-lifecycle-research/Cargo.toml --test sqlite_web
CARGO_TARGET_DIR="$PWD/target" cargo run --offline --locked --manifest-path examples/host-lifecycle-research/Cargo.toml --example web_with_sqlite
~~~

Their exact results and source identities belong to the B3 task receipts. The
commands and acceptance cases listed here do not assert that execution or
independent review has passed.

A's listener, second-bind, readiness, real in-flight HTTP drain and finite
two-Web evidence remains in `.pipeline-output/m28-yt286a-20261121/`. It is
historical evidence for that reviewed snapshot; B1/B2 did not claim to rerun
or newly qualify those service scenarios. B3 separately exercises the changed
Web/SQLite ownership boundary through its focused integration cases.

Final acceptance requires the configured independent Reviewer's actual PASS
and unchanged reviewed source identities. No native, browser/GPU, Windows,
published-package or broad feature-matrix qualification is claimed. The broader
YT-286 public gate and ADR 0014 proposal remain open.
