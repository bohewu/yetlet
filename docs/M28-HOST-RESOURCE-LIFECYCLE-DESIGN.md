# M28 — Layered Hosts and ResourceGroup Lifecycle Design

**Status:** Owner-selected feature-first direction; YT-287–294 independently reviewed and Done at source/package delivery scope.
**Date:** 2026-11-21.
**Tracking:** YT-286 source/package delivery complete through YT-287–294; publication and physical LAN remain separate.
**Decision:** [ADR 0014](adr/0014-layered-host-resource-lifecycle.md).
**Source baseline:** `dec423a5ef6f7100e5f31cb93d0cbbec84652d71`.

The [current delivery sequence](#14-replanned-public-delivery--yt-287-through-yt-294)
supersedes the earlier additive-only packaging and Application naming proposal.
The [YT-287 contract](#15-yt-287--feature-first-architecture-and-driver-contract)
defines the next-alpha feature/module and driver target. Sections 1–13 retain
the original research and private-proof contracts; their historical authorization
statements do not override the owner's later request.

## 1. Problem and owner requirements

The owner wants a short path to create and combine high-level Hosts, with
default resource lifecycle management, and an exposed ResourceGroup for
developers who need custom management. Names must explain intent even in the
advanced API. A Host represents an execution environment or external event
source; its identity does not prescribe a thread, process, renderer or runtime.

This document consolidates the owner's naming, multi-host and lifecycle
questions. It proposes an additive API direction rather than claiming that
the current typed composition API already implements it. The earlier
WindowSlot naming commit is complete and independent of this proposal.

The constraints come from the owner's discussion and the accepted
[ADR 0013 F2 scope](adr/0013-composable-application-and-hosts.md#accepted-scoped-f2-decision--2026-11-19):
preserve caller-local Rust values, explicit resource ownership, admission
fencing, original completion owners and truthful incomplete cleanup. Ordinary
Web/API/SSR and direct Axum/Tower control remain first-class paths.

The initial authorization covered research, documents and independent review.
The owner subsequently authorized YT-286-A, B1 through B2, and B3: the bounded,
unpublished [Host lifecycle prototype](../examples/host-lifecycle-research/README.md),
ordered resource final release, and real Web/SQLite integration. Sections 11
through 13 distinguish those source proofs from the baseline and remaining
proposal. They do not adopt the public
facade or full driver design, or authorize Git commit, push or publication.
ADR 0014 remains Proposed.

## 2. What exists at the source baseline

These are observations of the named baseline, not proposed behavior.

| Source | Existing behavior | Consequence for this design |
|---|---|---|
| [Composition core](../crates/yetlet/src/composition/mod.rs) | The actual entry is `yetlet::composition::Builder::<E>::new()`. `build()` seals a typed chain. There is no implemented `Application`, `ResourceGroup` or `Composition::builder()` entry. | Show new names as proposals, not available API. |
| Same core, `required_local` and `required_foreground` | Both retain a local value. The former starts marked ready; the latter waits for caller attestation. Neither allocates a thread or enforces main-thread affinity. | State, execution constraints and resource ownership need distinct concepts. |
| Same core, `try_required_local` and `try_required_worker` | A factory runs immediately, once, on the calling thread. Failure retains the original root in `StartFailure::into_root()` after requesting stop. | Deferred acquisition in the proposed high-level layer is a new contract, not a mechanical rename. |
| [Lifecycle chain](../crates/yetlet/src/composition/lifecycle.rs) and core | Close all admission gates, close locals in reverse attachment order, signal workers, then observe original workers. `Pending` retains owners; no implicit stop/join in Drop. | General dependency-safe resource release needs an engine extension. Existing sequencing alone cannot ensure that a database outlives a worker using it. |
| [Native adapter](../crates/yetlet/src/native_winit.rs) | `WindowSlot::new` prepares attributes; the caller supplies the active Winit loop when creating a window. The slot is caller-local. | Window preparation, actual native acquisition and event-loop ownership are separate. |
| [Web adapter](../crates/yetlet/src/web/worker.rs) | `HttpWorker::start_on` binds a listener and spawns one owned OS thread; that thread drives a Tokio current-thread runtime. Original thread completion is observable. | The current Web adapter has a concrete topology, although Composition itself has no scheduler. There is no shared framework thread pool. |
| [UDP adapter](../crates/yetlet/src/udp.rs) | `UdpEndpoint` owns a nonblocking socket and exposes bounded caller-driven polling. It creates no worker thread. | Several Hosts can share a driver without each needing a thread. |
| [Three-way consumer](../crates/yetlet/examples/composition_three_way.rs) | App-local Native/Network/Web values compose a caller-driven native/domain/UDP path with the owned HTTP service. The caller explicitly drives readiness and shutdown. | A high-level layer should package this coordination without making its particular topology universal. |

The public-source F2 work and its existing tests support these bounded facts.
They do not validate the new facade, resource hierarchy, placement planner or
new shutdown ordering. The [package compatibility record](M28-F2-PACKAGE-COMPATIBILITY.md)
also distinguishes local source consumption from registry publication.

## 3. External research

Primary documentation was checked on 2026-11-21. Winit references are pinned
to 0.30.13, matching the native research baseline. The Tokio reference pages
resolved to 1.54.1 and Tokio-util to 0.7.20 during research; those external
versions are not claims about the repository lockfiles. The proposed design
is our inference from these sources and the owner's requirements.

| Primary source | Relevant fact | Design implication |
|---|---|---|
| [.NET Generic Host](https://learn.microsoft.com/en-us/dotnet/core/extensions/generic-host) | The application lifetime root groups resources, offers default construction and permits lifecycle customization. | Use default-first layering. Its outer Host corresponds more closely to Yetlet's proposed Application; do not import its DI model or reinterpret Yetlet Host identity. |
| [Tokio graceful shutdown](https://tokio.rs/tokio/topics/shutdown) | Deciding to stop, signalling and waiting for completion are separate stages. | Default lifecycle coordinates stages; a successful stop request is not completed cleanup. |
| [Tokio-util TaskTracker](https://docs.rs/tokio-util/latest/tokio_util/task/task_tracker/struct.TaskTracker.html) | `wait` needs closed and empty; `close` does not prevent new tasks. Dropping a tracker does not abort its tasks. | An admission barrier and task completion tracking are separate responsibilities. A tracker alone is not this lifecycle engine. |
| [Tokio JoinHandle](https://docs.rs/tokio/latest/tokio/task/struct.JoinHandle.html) and [std thread JoinHandle](https://doc.rust-lang.org/std/thread/struct.JoinHandle.html) | Dropping a handle detaches its activity; a stop notification does not recover the result. | Retain the original completion owner through Pending and failure. |
| [Tokio LocalSet](https://docs.rs/tokio/latest/tokio/task/struct.LocalSet.html) | Caller-local tasks can be driven on one thread without requiring Send. | Preserve local ownership; add Send only at a selected cross-thread boundary. |
| [Tokio runtime builder](https://docs.rs/tokio/1.54.1/tokio/runtime/struct.Builder.html) and [runtime ownership](https://docs.rs/tokio/1.54.1/tokio/runtime/struct.Runtime.html) | Scheduler choice and runtime ownership are explicit. A Handle alone does not keep its Runtime alive; shutdown timeout can leave work running. | Sharing an executor requires a lifetime contract. Host count is not a pool-size rule, and a timeout is not proof of release. |
| [Winit EventLoopBuilder](https://docs.rs/winit/0.30.13/winit/event_loop/struct.EventLoopBuilder.html) and [source](https://docs.rs/winit/0.30.13/src/winit/event_loop.rs.html) | Portable native creation belongs on the main thread. The native process-wide creation guard rejects recreation; off-main-thread options are platform-specific. | One selected native loop owns multiple windows. Validate known conflicts early, but keep startup environmental checks and process-wide limits explicit. |
| [Rust TryFrom](https://doc.rust-lang.org/std/convert/trait.TryFrom.html) and [Mutex](https://doc.rust-lang.org/std/sync/struct.Mutex.html) | `try_from` expresses fallible conversion; `try_lock` additionally means nonblocking acquisition. `lock` itself also returns Result. | There is no universal rule that every Result-returning method needs `try_`. Document the operation and the particular distinction. |

These references justify separation of concerns, not a new dependency on
Microsoft hosting, Tokio-util, a global runtime or a generic scheduler.

## 4. Recommended layers

| Layer | Developer intent | Responsibility |
|---|---|---|
| Application and high-level Host | Choose which environments and event sources form the application | Register Host descriptions, validate compatible execution requirements and run a coordinated lifetime with defaults. |
| ResourceGroup | Customize how resources are acquired, used and released together | Describe typed ownership, resource behavior, readiness evidence and completion obligations; permit caller-owned resources. |
| Shared lifecycle engine | Enforce the declared contracts | Track state and primary failure, fence admission, retain dependencies and original owners, and drive retryable cleanup. |

Built-in Hosts construct their default ResourceGroups internally. Custom
Hosts use the same mechanism. The high-level path and the advanced path must
not maintain independent lifecycle algorithms.

ResourceGroup describes a lifetime scope. It does not imply a thread, process,
async runtime, task pool, service locator or untyped registry. The engine can
evolve from Composition; preserving one engine does not mean its existing
release order is already sufficient.

The outer candidate name is `Application`, avoiding confusion with the
existing server-specific `server_axum::App`. `ResourceGroup` is the proposed
advanced concept. Concrete module paths, generic parameters and a statically
composed Host extension protocol require a compiling proof before freezing.

## 5. Normal developer experience

All examples below are illustrative, unimplemented API sketches. They are
not compilation evidence or replacement instructions for current consumers.

~~~rust
let app = Application::new()
    .with_host(
        NativeHost::new()
            .with_window(primary_window)
            .with_window(inspector_window),
    )
    .with_host(WebHost::new(router))
    .with_host(NetworkHost::udp(network_options))
    .build()?;

app.run()?;
~~~

The developer chooses Hosts. They do not classify every resource as local,
foreground or worker, or append `required()` to make explicit registration
meaningful. A selected Host is part of the application's success contract:
startup or fatal runtime failure requests coordinated application shutdown.
Optional participation, restart and isolation policies are separate future
features, not implied by `try_` or an omitted required marker.

### Construction and execution

| Operation | Proposed contract |
|---|---|
| Host `new` and configuration methods | Create inert descriptions. No socket binding, thread spawn, event-loop creation or readiness claim. |
| `with_host` | Transfer a Host description into the application plan; do not start it. |
| `build` | Validate and seal known topology, ownership and execution requirements; return a configured application. Do not acquire deferred resources. |
| `run` | Acquire resources, drive the selected event source(s), establish readiness, admit work and coordinate shutdown. |
| `request_shutdown` | Initiate idempotent shutdown; no claim that resources have finished. |
| `shutdown_until` | Advanced cooperative shutdown progress through a shared deadline, returning Complete or Pending with the original owner still retained. It also requests shutdown if necessary. |

Configuration can itself be invalid. A fallible constructor may return Result
without acquiring live resources. Build-time validation cannot prove a port
is free later, detect every externally created native loop, or determine that
a future readiness probe will succeed.

For the ordinary `run()?` shape, select a **terminal-only convenience contract**:
on normal return, including an operational error return, all managed cleanup
obligations have reached terminal completion. Run drives cleanup after partial
startup failure before returning its diagnostic. There is no hidden finite
shutdown timeout that abandons the group.

This convenience can wait indefinitely for an uncooperative activity or failed
cleanup operation. It must expose progress/diagnostics and cooperate with the
selected driver rather than busy-loop. Applications needing a deadline or
external event-loop ownership use the advanced driving interface, retain the
application owner and handle Pending. Rust cannot promise that abort, process
kill, panic-abort or forgetting a value will run cleanup; Drop is not an
implicit join mechanism.

Native readiness must be progressed inside the selected event loop. Run must
not synchronously wait for a window readiness event before entering that loop.
The initial portable native convenience is a desktop main-thread path;
browser/mobile launch semantics need their own adapter proof.

## 6. ResourceGroup customization

### Defaults and custom behavior

The SDK can supply adapters for resources it understands, default sequencing,
error aggregation and retained ownership. It cannot infer whether an arbitrary
resource is usable or whether a custom task has really stopped. Host authors
either reuse known adapters or describe those operations explicitly.

A proposed customization hook augments the existing group:

~~~rust
let sync = SyncHost::new(sync_options)
    .configure_resources(|resources| {
        resources.with_resource(metrics_resource)
    });

let app = Application::new()
    .with_host(sync)
    .build()?;
~~~

Here `SyncHost` is a hypothetical application-defined Host and
`metrics_resource` is an inert resource description. `configure_resources`
adds resources or uses explicit typed overrides offered by that Host. It
does not silently replace its mandatory admission, driver or completion
contracts. There is no generic replace-everything `.resources(...)` setter in
the recommended first surface.

A custom Host author can begin with `ResourceGroup::new()` and register
resource descriptions through `with_resource`. Its contribution to Application
is the typed resource plan, execution requirements and a compatible driver.
The exact trait/associated-type/closure representation is a proof gate;
a universal object-safe Host trait is not presumed necessary.

The advanced layer must distinguish the following inputs even if their final
method spellings remain subject to that proof:

| Input | Acquisition and cleanup ownership |
|---|---|
| Deferred resource description | Acquire during startup. The group takes ownership as soon as an acquisition side effect needs cleanup. |
| Already acquired owned resource | Ownership transfers explicitly. Dropping a not-yet-started configuration still needs a documented cleanup path; do not allow opaque running work to hide in an inert plan. |
| Caller-owned borrowed resource | Preserve its lifetime and exclusive/shared access constraints; do not shut it down or destroy it as the borrower. |
| Running activity with a completion owner | Use the advanced active-owner path; retain its original handle and dependencies through failure and Pending. It is not an ordinary inert Host description. |

The first high-level path accepts inert descriptions. Adoption of live values
must remain an explicitly advanced path until ownership on configuration
failure and pre-start cancellation is proven. A resource's simple synchronous
RAII drop is adequate only when its contract truly has no pending activity or
fallible managed cleanup.

A resource adapter describes acquisition, availability evidence, admission
closure when applicable, shutdown signalling, completion observation and final
release. Simple resources need only the meaningful subset; a passive value
does not need fake task or readiness callbacks. Avoid an ambiguous catch-all
`on_shutdown` callback that conflates signalling, draining and final release.

### Typed ownership and a bounded dependency model

Start with an Application root and Host child groups. Shared dependencies live
in an owning ancestor; dependent activity receives explicit typed access or
ports. Each resource has exactly one cleanup owner.

A driver shared by Host groups is owned at the Application root so every
child can finish driver-dependent cleanup before it is released. NativeHost
selects and contributes that driver; its window resources belong to its child
group. This is one lifecycle owner for the loop, not a borrowed reference into
a sibling Host that can be destroyed first.

Retain concrete types and borrow lifetimes. Do not require all resources to fit
`Box<dyn Any + Send + Sync + 'static>`. A cross-thread adapter may require Send
for its moved factory, captured values or messages; it must not impose that
bound on unrelated local state.

Keep the first implementation constrained:

1. Acquire a group's passive dependencies in declaration order; an earlier
   resource can support a later resource.
2. Start its activities and child groups only after their declared dependencies
   are available. Readiness that needs an event loop progresses in that driver.
3. Permit lifetime dependencies on earlier resources or ancestors. A resource
   used by an activity must outlive that activity's terminal completion.
4. Keep siblings independent; share dependencies through an ancestor.
   Arbitrary sibling dependencies, cycles and a general dependency DAG are
   outside the first proof and must be rejected rather than guessed.
5. Release resources in reverse dependency order only after their dependents
   have actually completed. Parent resources outlive all child groups using them.

Startup prerequisites, lifetime dependencies, exclusive-resource claims and
thread affinity are different constraints. Registration order alone does not
prove all four. Typed construction or plan validation must enforce the allowed
relations; labels are diagnostics, not lookup keys that manufacture ownership.
The proof must demonstrate access without unsafe self-referential storage.

### Failure, shutdown and completion

The proposed engine advances these responsibilities:

1. Latch the original startup/runtime failure or stop request.
2. Fence new work across affected groups before invalidating their domain
   authority. An adapter must state how already admitted work is drained or
   rejected; closing a producer gate alone does not prove in-flight work ended.
3. Signal owned activities through a separate control path that cannot be
   blocked by a full data queue. Continue to keep all needed dependencies alive.
4. Observe original activity completion and drive callback-local cleanup.
   Observation must still discover an already exited activity if signalling
   failed. Do not spend a new full timeout on each resource.
5. Release completed groups' resources in dependency order, and release shared
   ancestor resources only after every dependent group is complete.
6. Return terminal completion only when all owned obligations are resolved.
   Record the operational result separately from cleanup completion.

If the required admission barrier fails, resources protecting the domain stay
alive and the group remains Pending. Any permitted safe progress must not
bypass that barrier. A resource-close error retains the resource for retry;
successful one-time effects are latched. Adapters declare retry/idempotence
requirements; a deadline cannot preempt an arbitrary blocking callback.

For partial acquisition, a plain `FnOnce() -> Result<T, E>` is sufficient only
when failure leaves no escaped cleanup obligation. A factory that can start
work and then fail must install an owner before that side effect or return the
partial owner as part of its failure. That obligation must not disappear in an
error conversion.

The advanced shape keeps the application owner in scope:

~~~rust
// Fragment within an explicitly driven, already owned application.
// Candidate names and result fields; not compilable current API.
app.request_shutdown();
match app.shutdown_until(deadline) {
    Completion::Complete(report) => inspect_operational_result(report),
    Completion::Pending(progress) => retain_and_continue(app, progress),
}
~~~

`retain_and_continue` is a placeholder for the caller's continued driving, not
an SDK function or a background task spawn. A native callback must use the
driver-compatible nonblocking progress operation; it must not block its own
event loop waiting for a future callback. The public synchronous/async/step
forms remain subject to proof of that driver interaction.

Do not turn an ownership-bearing start failure into a string and propagate it
with `?`, or drop an owner on Pending. A cleanup-complete report may still
contain a startup/runtime error. Success of the application and completion of
cleanup are independent facts.

## 7. Native uniqueness and execution placement

Two `NativeHost::new()` descriptions are harmless preparations. A plan that
requests two independently owned Winit event loops is a conflict, detected
before acquisition when both requests are visible. The recommended native
Host contributes one root-owned loop and manages multiple windows identified
by their WindowId; the root retains that driver through all child cleanup.

The Winit 0.30.13 native guard is process-wide, stronger than one per
Application or one at a time. Do not promise recreation after dropping a
loop, or guaranteed retry after an attempted creation. An external loop and
races between separately built applications can still fail at startup.
Caller-supplied loop integration needs an explicit driver/lifetime contract.

Native `run` must obey the selected platform's entry-thread rules. The proof
must document its caller preconditions and validate supported conflicts;
no portable detection of every wrong calling thread is claimed here.
Platform-specific any-thread support is an explicit adapter capability, not a
universal solution to multiple native Hosts.

| Environment | Initial proposed default | Advanced boundary |
|---|---|---|
| Native desktop | One main-thread loop drives windows and compatible local work. | Explicit supplied driver; no automatic migration of native local resources. |
| Web in a combined native application | Preserve the existing adapter's owned service-thread/current-thread-runtime topology first. | A supplied shared executor requires proven driving, ownership and shutdown semantics. |
| UDP/local work | Bounded polling through the selected driver where supported. | An owned activity is an explicit adapter choice. |
| Pure Web | Existing direct Axum/Tokio path remains available. | High-level composition is optional. |

Multiple Hosts do not establish a need for a new thread pool. Use an
appropriate executor or workload-specific pool only where selected activities
need it. A Tokio current-thread scheduler also does not imply that optional
blocking operations can never create other threads.

Defer a public `ExecutionPreference` until an adapter supports at least two
honest choices. The proposed policy distinction is default placement, a
preference with a declared supported fallback, and a strict requirement that
fails when unsatisfied. Resolve known constraints before acquisition and expose
the chosen placement and fallback reason. A failed spawn must not silently move
a blocking service onto the native callback thread; runtime fallback needs its
own cleanup-safe adapter contract.

Separate-process hosting is a later design with IPC, serialization, readiness,
failure and process supervision. Moving a Rust reference or selecting a
different enum variant does not supply those contracts.

## 8. Naming disposition

These are proposed meanings, not aliases or edits to current Rust APIs.

| Candidate or existing name | Disposition |
|---|---|
| `Application::new()` | Selected private A entry: describe the application and select Hosts. Public promotion remains proposed. |
| `ResourceGroup` / `with_resource` | Candidate advanced lifetime scope and resource registration. |
| `Composition` | May remain implementation vocabulary during migration; do not make it the unexplained quickstart concept. ResourceGroup is not a blind type alias. |
| `required_*` / `.required()` | Omit from normal registration. Participation is the default success contract; policy exceptions require separate explicit semantics. |
| `foreground` | Replace the concept with a concrete execution constraint such as main-thread or caller-driven where relevant, not a momentary readiness label. |
| `attach_ready` / `attach_worker` | Do not adopt as the new primary categories. Distinguish resource ownership, acquisition timing and an activity's completion contract. |
| `request_shutdown` | A request with observable progress, not successful completion. |
| `shutdown_until` | Cooperative shutdown progress to terminal completion or a deadline; owner remains on Pending. |
| `try_*` | Use when a deliberate counterpart has distinct behavior; never imply optional participation, retry, swallowed failure or automatic fallback. |

For the current `try_required_*` methods, failure means error propagation with
the original root available for cleanup. That existing contract must survive
migration. A future fallible acquisition operation may have a descriptive name
without `try_`; its Result and documented cleanup ownership communicate failure.
If a pair is introduced, explicitly define whether the distinction is
fallibility, nonblocking behavior or another operation-specific guarantee.

Follow the [naming guide](NAMING.md): select one canonical name and migrate
affected consumers together when implementation is authorized. Do not retain
a growing family of synonymous methods. Historical evidence keeps its original
names and scopes.

## 9. Alternatives and recommendation

| Alternative | Assessment |
|---|---|
| Rename existing Composition methods only | Useful for a bounded naming change, but does not supply Host defaults, deferred acquisition or dependency-safe release. |
| Expose every lifecycle callback in the quickstart | Preserves control but fails the owner's default-first requirement. |
| Separate built-in Host manager and custom ResourceGroup engine | Duplicates failure and shutdown rules and permits behavioral drift. |
| Default Host facade over a shared ResourceGroup lifecycle engine | Recommended; it meets the requested layers while preserving an advanced ownership contract. Requires explicit engine work. |
| One thread/process per Host or automatic universal pool | Confuses Host identity with topology and conflicts with caller-local/native constraints. |
| Dynamic DI container or arbitrary resource DAG first | Adds unresolved lifetime/type complexity before the bounded consumers justify it. Defer. |

## 10. Delivery and verification gates

Document review does not establish implementation. The separately authorized
A proof below covers a bounded subset; YT-286 public-API promotion stays Todo
until its remaining scope is adopted and evidenced. Physical F3/D4 is independent.

| Gate | Required evidence before the corresponding implementation claim |
|---|---|
| Direction decision | Maintainer disposition of ADR 0014 and bounded implementation authorization. |
| Typed ownership proof | A compile-valid normal facade and a custom Host using the same ResourceGroup engine; local Rc and borrowed resources; no global Send/Sync/'static requirement. |
| Deferred acquisition proof | Multiple descriptions create no sockets, threads or event loops; invalid plans acquire nothing; a later startup failure retains every earlier and partial owner. |
| Dependency-safe engine proof | An activity can use a database through shutdown, reaches actual completion, and only then permits final database release; ancestor dependencies outlive children. |
| Driver proof | Native readiness is progressed by its one active loop; no readiness-before-loop deadlock or blocking completion wait in native callbacks. |
| Completion proof | Failed admission, failed signal with already-exited activity, cleanup failure, shared-deadline Pending and subsequent completion all retain original owners and the primary error. |
| Default-path proof | Terminal-only run returns safely after startup failure and reports failure separately from completed cleanup; uncooperative cleanup remains observable, never false success. |
| Resource customization proof | Added resources preserve default obligations; supported typed overrides cannot remove required ownership/completion behavior. |
| Execution proof | Known duplicate native-loop requests fail before acquisition; external-loop conflict is reported honestly at startup; borrowed executors and unrelated work remain caller-owned. |
| Consumer and compatibility proof | Existing direct Web path, Native plus UDP without Web, and an external custom Host compile through public APIs; selected feature graphs stay isolated. |
| Migration and review | One canonical naming migration, exact consumer updates, behavior-specific checks and independent Reviewer PASS. No registry or platform claims beyond the evidence. |

Resolve exact Rust signatures, live-resource adoption, driver step forms and
the smallest needed engine changes in the bounded proof. Do not add a generic
scheduler, resource registry, process supervisor or exhaustive platform matrix
to satisfy a documentation gate. Native platform checks are needed only for
the native behavior eventually changed.

The original documentation task verified English Markdown, local links and
anchors, whitespace and backlog dependencies, plus independent review. Its
PASS was a document claim only. The subsequent A proof and runtime scope are
recorded below.

## 11. YT-286-A bounded implementation — 2026-11-21

The owner approved the first implementation proof after reviewing the proposed
direction. The independent, unpublished
[research crate](../examples/host-lifecycle-research/README.md) now compiles the
normal path as `Application::new().with_host(...).build()?.run()?`. The same
`Host` protocol and typed `ResourceGroup` support default WebHost and an external
custom Host with local Rc and caller-owned borrows.

At the A snapshot, the prototype was deliberately limited to one to four Hosts
and one activity per Host. Concrete typed slots remained inside the existing
Composition root; a small arity macro built operation tables without copying
the lifecycle state machine or exposing the core's private recursive trait.
A required no SDK file or root feature graph changes. Section 12 records the
later opt-in core extension; the Host count and activity limit remain unchanged.

Built-in descriptions are inert. All validation precedes acquisition. Each
activity stores acquired owners before readiness can fail. Real Web probes
precede work admission; later startup failure skips remaining Hosts, retains
earlier and partial owners, and drives the same rollback/completion path.
Advanced Pending retains the original root. Terminal-only run does not return
an operational error until cleanup completes.

The A resource proof was specifically **passive infallible RAII**: each group's
activity was destroyed before that group's reverse-order passive dependencies,
and the entire root remained intact until all original activities had joined.
A did not prove fallible/asynchronous finalizers, ancestor resource plans or a
general final-release phase. B1/B2 below addresses ordered final release while
preserving legacy Local behavior; broader public promotion remains open.
`WebHost::with_resource` only augments defaults; the broader
`configure_resources` and typed driver override proposals remain deferred.

The [focused tests](../examples/host-lifecycle-research/tests/lifecycle.rs) and
[finite two-Web executable](../examples/host-lifecycle-research/examples/two_web_hosts.rs)
exercise actual listeners and original joins, including second-bind rollback,
self-readiness failure, in-flight HTTP drain, failed admission fencing,
primary/secondary error retention, cancelled startup, inert invalid plans and
borrowed/local custom resources. The package README states the exact contracts,
commands and remaining limits. Independent configured Reviewer acceptance and
source identity receipts belong to `.pipeline-output/m28-yt286a-20261121/`;
a source proof does not establish native, Windows, browser/GPU or public
feature-isolation qualification.

YT-286 still tracks the full public adoption gate. This A proof neither closes
that gate nor changes ADR 0013's accepted F2 baseline.

## 12. YT-286-B1/B2 — ordered final release

The owner authorized B1 through B2 after the A proof. B1 defines resource
acquisition, stop, observed completion and final release as separate obligations.
B2 implements that ordered contract in the existing
[Composition engine](../crates/yetlet/src/composition/resources.rs) and exercises
it from the private Host facade. It does not authorize B3 service/database
integration, native integration, publication or Git commit/push.

### Contract and engine boundary

Existing Local and Worker APIs keep their behavior. In particular, a legacy
Local closes before worker observation. The new resource category supplies the
missing post-Join lifetime: admission must be closed, legacy Local cleanup must
finish, and every original worker must have a terminal Join before any resource
finalizer runs. A failed actual Join is terminal evidence; a failed observation
attempt is not.

| Core source API | Responsibility |
|---|---|
| `Builder::with_resource` | Own a value with an explicit final-release callback |
| `Builder::with_activity` | Preserve Worker readiness/stop/join behavior and add post-Join payload retirement |
| `Builder::with_group` | Nest typed parts inside the same traversal and error ledger |
| `ReleaseCompletion::Pending` | Retain the original owner and cleanup operation |
| `ReleaseCompletion::Released(Result)` | Latch terminal release, including a terminal error |
| Sealed `ResourceParts<E>` | Let external generic resource plans name crate-built parts without implementing the private traversal |

The callback's outer `Err` records a failed progress attempt and remains
Pending. `Released(Err)` instead records a terminal error once and permits
earlier dependencies to advance. The first observed error remains primary,
including an earlier acquisition/start failure. Successful and failed terminal
release outcomes are latched separately from application success.

Later attachments may depend on earlier ones. Final release walks in reverse;
a later Pending or retryable failure blocks earlier resource/group release.
The resource's actual value is dropped on terminal release before an earlier
finalizer advances. Nested groups carry the same absolute deadline, so they
cannot reset the observation/release budget. Zero-budget callbacks may observe
nonblocking progress. This is a cooperative budget, not preemption.

Groups do not create a second coordinator. Existing admission, Local, signal,
original-Join and error bookkeeping remains in Composition; B2 adds one
release traversal there. There is no global `Send`/`Sync`/`'static` bound,
new runtime, dependency, feature or dynamic registry. The source-level opt-in
core additions are real exports under `composition`; the high-level
Application/Host facade remains unpublished research.

### Facade acquisition and destruction

The existing `with_resource(value)` keeps passive RAII ownership semantics.
The advanced `with_managed_resource(label, descriptor)` accepts an inert
`ManagedResource` description with `acquire` and `release_within` methods.
It augments a default WebHost or a custom ResourceGroup.

Build validates all Host descriptions before acquisition and installs resource
descriptors plus activities in the original root. Managed resources acquire
in registration order before their dependent activity starts. A descriptor
must store partial owners before any later fallible operation. Failure skips
later acquisition and drives rollback through the same root. Cancellation is
checked between acquisitions and before activity startup.

A never-attempted descriptor skips its release callback and drops normally.
Therefore pre-start descriptor destruction must not require asynchronous or
fallible cleanup. This prevents a validation error from silently losing a
live managed owner; general adoption of such an already-live resource remains
outside this proof.

Each facade group appends its activity after its resources. Final traversal
first takes and drops that group's `Option<Activity>`, then releases its
resources in reverse order. Core `ManagedWorker` itself retains its
coordination value for status inspection: its callback must retire any payload
whose destructor needs earlier dependencies before reporting Released.
No application-wide claim is made that all activity destructors precede
every group's resource finalizers.

Ordinary Group destruction also drops its contained parts in reverse attachment
order, without invoking lifecycle callbacks. This preserves A's passive RAII
order when a built application is dropped before start: activity, later resource,
earlier dependency. It does not turn Drop into managed shutdown.

Pending retains the same root, unfinished owner and original cleanup operation.
A terminal release can destroy its own value while earlier dependencies stay
owned. Terminal-only `run` waits until release completes before returning
success or failure. Advanced callers must continue driving a Pending owner;
ordinary Drop does not run asynchronous release or recover detached work.

The [managed-resource tests](../examples/host-lifecycle-research/tests/resources.rs)
include one real local Future retained across Pending and completed only once.
The [Rust Future contract](https://doc.rust-lang.org/std/future/trait.Future.html)
requires an actual polling/wake driver and forbids assuming background progress;
the adapter remains responsible for that integration. B2 adds no hidden executor.
Destruction claims follow the
[Rust destructor contract](https://doc.rust-lang.org/reference/destructors.html)
and assume callbacks/destructors do not panic.

### Evidence and remaining work

The core lifecycle suite has 21 cases, including eight new cases for original
joins, group/dependency barriers, transient and terminal release errors,
retained primary evidence, nested deadlines and managed activity finalization.
Six new facade cases cover partial acquisition, never-attempted cleanup,
cancellation, activity destruction and original asynchronous cleanup ownership.
Targeted non-Web compatibility and compilation checks cover changed paths.
Exact results and the configured independent review/source-identity receipts
belong to `.pipeline-output/m28-yt286b12-20261121/`.

A's accepted evidence remains attached to its original snapshot. Its two-Web
and service smoke scenarios are not repeated just to record B1/B2 completion.
A's broad Local limitation is addressed for the new ordered resource category,
not by changing the legacy Local contract.

This proof conservatively serializes reverse group finalization. It does not
prove independent sibling progress, an arbitrary dependency DAG, self-referential
resource wiring, shared ancestor APIs or live activity/resource adoption.
At the B1/B2 checkpoint, real service/database integration remained subsequent
work; section 13 records the authorized B3 consumer. Native/main-thread behavior,
executor placement and public API promotion remain open. ADR 0014 stays Proposed,
and the full YT-286 public gate remains open.

## 13. YT-286-B3 — real Web and managed SQLite integration

The owner authorized B3 after the B1/B2 source proof. The
[finite consumer](../examples/host-lifecycle-research/examples/web_with_sqlite.rs)
and [integration cases](../examples/host-lifecycle-research/tests/sqlite_web.rs)
now use an actual SQLite database from actual loopback HTTP handlers. They use
the existing Application, WebHost and managed-resource interface. No new
lifecycle engine or public SDK database adapter is introduced.

### Concrete ownership and execution

The private [SQLite resource](../examples/host-lifecycle-research/examples/support/sqlite_resource.rs)
is inert during construction and build. Its acquire method starts one owned
database thread and stores the original JoinHandle before waiting for readiness.
The thread opens SQLite and initializes the fixed records table before request
access is published. A database open or schema error remains the original
acquisition cause; the descriptor keeps the acquired thread handle for rollback,
even if setup has already closed its Connection.

The Router receives a typed cloneable client. Client clones share a single
optional sender; they neither clone a Connection nor retain independent senders.
A bounded channel of 32 commands carries parameterized insert and count requests.
Submission uses nonblocking try_send and Tokio oneshot replies, so synchronous
SQLite work executes on the database thread rather than the Web runtime.

This thread is work owned by a resource, distinct from the WebHost's registered
activity. The normal Web adapter still has its existing service thread. These
are concrete consumer choices, not a Host-per-thread rule, a shared pool or a
new application-wide executor.

### Drain, final release and actual completion

Admission closes and already admitted HTTP work drains while the database
remains usable. The original Web worker must join before database final release
begins. Release takes the only sender from the shared client state. That
operation needs no free slot in the data queue, and retained client clones can
no longer enqueue commands. The database thread drains accepted commands,
attempts explicit Connection close and exits.

The first release call can return Pending after requesting this shutdown.
It still retains the original unobserved JoinHandle; Pending does not assert
that SQLite is necessarily still open. Later calls inspect and join that same
handle. An earlier passive dependency remains owned until database release
is terminal.

SQLite can return the live Connection when explicit close fails. The resource
transfers that Connection back into descriptor-owned retry storage before
returning a progress error. Therefore a joined database thread does not alone
prove completed resource release. The independently observable states are
request availability, open Connection ownership, release initiation and
original thread join. Actual close retry occurs on the caller after the worker
has joined; no thread-affinity promise is made for that fallback.

The [rusqlite Connection close contract](https://docs.rs/rusqlite/latest/rusqlite/struct.Connection.html#method.close)
documents the returned owner. The implementation also checked the exact cached
0.41.1 source used by this consumer. The
[Rust channel contract](https://doc.rust-lang.org/std/sync/mpsc/fn.sync_channel.html)
and [JoinHandle contract](https://doc.rust-lang.org/std/thread/struct.JoinHandle.html)
support the bounded queue and retained original completion design. These source
references do not substitute for execution evidence.

### Bounded acceptance and dependency isolation

| Case | Observable acceptance |
|---|---|
| Terminal normal run | Build creates neither database nor listener; actual HTTP writes/counts succeed; a new SQLite Connection reads the persisted rows after original Web/database joins and release. |
| Admitted HTTP drain | A paused admitted handler writes both before and after stop; late work cannot append; post-Web cleanup Pending retains the original database join and earlier dependency. |
| Later bind failure | The original OS bind error stays primary while a clearly labelled controlled cleanup-progress error is secondary; the same database owner completes rollback. |
| SQLite schema/readiness failure | A pre-created non-SQLite file causes an actual acquisition error; Web never starts and the original database worker is joined during rollback. |
| SQLite open failure | A directory database path fails actual open; the acquired thread handle remains owned until observed join. |

The cleanup-progress fault is deliberate lifecycle fault injection; it is not
claimed as an induced native SQLite close failure. Native close failure handling
is grounded in the returned-owner API and source inspection. The private
consumer does not expose raw statements or connections for fault injection.

The research manifest pins rusqlite 0.41.1 with bundled SQLite and tempfile
3.27.0 as dev-dependencies. These versions are already used by repository
consumers and were available offline. They do not change root SDK dependencies,
features or the normal direct Axum path. Previously resolved versions remain unchanged;
the private lock adds the new consumer dependency closure.

B3 acceptance requires focused executed evidence, a finite example result,
configured independent Reviewer PASS and exact reviewed source identities.
Receipts belong to `.pipeline-output/m28-yt286b3-20261121/`. Existing A/B1/B2
results remain attached to their reviewed sources and are reused where unchanged.

This is bounded SQLite/loopback integration, not database throughput,
crash-recovery or production qualification. It does not establish a general
database pool, native driver support, Windows qualification, independent
sibling finalization, live-owner adoption or an arbitrary resource DAG.
The high-level facade remains private, ADR 0014 remains Proposed, and the whole
YT-286 public adoption gate remains open.

## 14. Replanned public delivery — YT-287 through YT-294

The owner selected a breaking next-alpha direction after reviewing the A/B
proofs: composition and resource lifecycle belong in Core, while View, Web and
concrete Host integrations have separate responsibilities. The latest owner
clarification makes **minimal SDK publication and feature switches** the priority.
Keep the existing yetlet and yetlet-browser published cohort as the starting
point; use optional modules/features in yetlet rather than creating published
yetlet-view, yetlet-web or yetlet-hosts packages by default.

Logical layers are not automatically package boundaries. Core composition stays
built in; View/HTML, HTTP and native/network/renderer dependencies remain opt-in.
A View-only consumer still uses the Core-containing package, but the View module
must not call composition or activate its adapters. The exact module, feature,
command/state and driver contract is YT-287's deliverable, not an implemented
or released API. A new package would need concrete independent-consumer evidence
and a separate decision, not symmetry with the architecture diagram.

The owner now authorizes this sequence: replan the YT items, obtain configured
independent Reviewer acceptance, pass the exact staged-index Strict/Builtin
Commit Guard and make a local commit; **only then execute YT-287**. The first
checkpoint includes the existing reviewed A/B1/B2/B3 work plus this plan.
This authority does not select a release, push, deploy or start the remaining
implementation items. Existing published alpha.8 and CLI 0.4.1 retain their
recorded identities and separately selected guide edition.

### Evidence carried forward

The unchanged 22-file B3 review snapshot was compared before this replan.
B1/B2 independent R2 `wc_job_iVK61uyaAJvJP1dE` and B3 independent R2
`wc_job_YXOH7vC7lVW4gYk9` completed with actual PASS and no required followups.
B3's five real SQLite/HTTP cases, finite persisted-readback example, focused
Clippy and formatting receipts remain at their original scope. Earlier failed
reviews, diagnostic calls and checker receipts remain retained. Documentation
changes in this replan need their own review; those runtime receipts do not
prove a new feature boundary or a new execution driver.

### Ordered implementation items

[The backlog](BACKLOG.md#yt-286-replan--core-view-and-web-delivery) is the
authoritative task-state table. The order below is the recommended delivery
sequence; dependencies express the minimum prerequisites.

| Item | Bounded deliverable | Acceptance before Done | Depends on |
|---|---|---|---|
| YT-287 | Architecture, feature/import and execution-driver contract | Source-to-module inventory; minimal published cohort and feature/dependency matrix; Core/View/Web/adapter responsibilities; App versus HostGroup decision; optional command/state and HTTP wire ownership; caller-runtime versus owned-thread drivers; readiness/address/cancellation policy; breaking migration and CLI/cohort ledger. Update this design, ADR 0014, architecture/naming and development intent. Configured independent review; no implemented-feature claim. | YT-284 |
| YT-288 | Built-in Core composition and generic Host defaults | Make composition available without a feature opt-in; promote the bounded generic HostGroup/ResourceGroup surface through one engine after compiling driver/extension contracts. Preserve local and borrowed ownership, original completion owners, global admission/join/final-release barrier and retained Pending. Core module and default-off consumer add no HTTP/Tokio/Winit/wgpu or mandatory serde; the complete View/default-feature cutover is YT-289. No blind promotion of the private io::Error, sleep loop or four-Host proof limit. | YT-287 |
| YT-289 | Optional View/HTML layer and Browser DOM migration | Switch yetlet to default=[] while isolating View, Element, tags, binding and pure HTML serialization behind the selected features. Migrate Browser DOM/keyed-DOM feature selection and reexports in the same slice. Core-only omits View; View/HTML adds no HTTP/runtime dependencies and has no internal composition dependency; DOM/Wasm has no native server dependency. Preserve escaping, validation and component-as-function semantics. | YT-287 |
| YT-290 | Optional Web ownership and transport boundary | Move App, Router/Page/SSR integration and classified HTTP capabilities into the selected yetlet Web module/features; relocate existing HTTP serving ownership without a reverse dependency on the unpublished yetlet-hosts crate. Separate command semantics from HTTP error/envelope types with a Wasm-safe shared wire feature. Migrate affected direct consumers and preserve caller-owned runtime, Router/Tower behavior and explicitly selected defaults. Temporary driver differences remain named until YT-291. | YT-288, YT-289 |
| YT-291 | One Web lifecycle with supported execution drivers | Use the same lifecycle/admission/failure/final-release contracts for direct async serving and multi-Web composition. Prove caller-runtime async progress and explicit owned-thread progress without blocking current-thread Tokio, nested runtime creation, detached original joins or false cancellation completion. Verify API-only routes, multiple Web Hosts, later-start rollback, admitted drain and managed-resource release. | YT-290 |
| YT-292 | Optional Native/Network/custom Host integration | Move concrete adapters onto the public Core contracts through yetlet features, removing the unpublished yetlet-hosts crate from the new SDK delivery path; keep renderer selection independent. One root-owned native loop can manage multiple windows; known duplicate-loop plans fail before acquisition, with external/process-wide constraints checked honestly. Preserve caller-driven UDP and custom local/borrowed resources. Classify the generic command/snapshot bridge separately from HTTP and keep it optional. Focus native checks on changed behavior. | YT-288, YT-291 |
| YT-293 | Consumer, CLI and next-alpha migration | Migrate remaining examples, generated projects, CLI templates/capability metadata, current docs and package README positioning. Verify the retained minimal package cohort and normal packaging against the new feature map; change cohort/schema tooling only if the selected package identities actually change. Preserve historical two-package records. Cover installed old-CLI/new-SDK selection with an explicit supported path or actionable rejection; no silent fallback. Do not promote the pinned reader before actual publication. | YT-289, YT-290, YT-291, YT-292 |
| YT-294 | External consumer and normal package qualification | Compile independent Core-only, View/HTML, pure Web, Browser DOM/actions, Native+UDP without Web and combined/custom-Host consumers against the selected normal archives. Verify target/feature graphs, lifecycle regressions, version/cohort compatibility and required documentation gates. Require configured independent review and exact-index Guard for local delivery. Registry publication, live reader promotion and physical LAN remain separate. | YT-293 |

YT-286 remains the umbrella public-delivery gate and depends on YT-294; it is
not a prerequisite of these children. A/B1/B2/B3 are completed bounded source
proofs, not tasks to redo. YT-285 and the original physical YT-274 gate remain
independent. No new YT-named report files are required for this sequence.

### YT-287 execution boundary

After the planning commit, complete the first item in the existing documents.
Use exact source evidence to decide ownership, feature/import migration and
driver semantics before any module move. Do not create another SDK package merely
to represent a logical layer. In particular:

- Keep generic HostGroup and built-in composition in the Core part of yetlet;
  define optional capability modules and their additive Cargo feature edges.
  Custom Hosts must not require a concrete adapter package.
- Keep HTML/DOM View optional and distinct from native scenes; do not invent a
  universal Component trait, global state service or a new package for every layer.
- Separate typed application command execution, HTTP protocol types and the
  existing command/snapshot bridge. Do not create action/state/resource crates
  merely for symmetry.
- Resolve the current App port 7878 versus private WebHost port 0 distinction,
  and the private root-GET-200 probe versus arbitrary API-only Routers.
- Specify who retains acquired resources when an async drive is cancelled.
  Wrapping the private blocking run loop in async does not satisfy this gate.
- Record migration of all existing optional native capabilities and the
  existing two-package release/CLI selection contracts, not only server_axum.

Exact public generic signatures and new runtime claims still require later
compiling proofs. Arbitrary resource DAGs, live asynchronous owner adoption,
automatic pools, process supervision and broad platform certification remain
deferred unless a concrete later item is separately selected.

## 15. YT-287 — feature-first architecture and driver contract

**Contract baseline:** planning commit `a60e038781ac92f26f8b0e790c9981fbb0f4d29f`.
This is the first item executed after the reviewed, Strict/Builtin-guarded
planning commit. It defines the next-alpha target; it does not move Rust modules,
change Cargo features, implement drivers, select a version or publish packages.
The current manifests and published alpha.8 remain unchanged in this item.

### 15.1 Publication and default boundary

Retain two SDK packages: **yetlet** and **yetlet-browser**. The CLI stays its
independent product/cohort. Use modules and additive Cargo features for capability
selection; do not publish separate Core, View, Web, Host, Action or Resource
packages for this migration. The current unpublished yetlet-hosts is a staging
location whose active implementations/consumers migrate into yetlet; it is not a
new registry dependency or a permanent compatibility facade.

Select `default = []` for the next yetlet API. Built-in `composition`, generic
`HostGroup`, the Host extension contract and `ResourceGroup` form the std-only
foundation, without a `composition` opt-in. This is not a no_std promise.
View, HTML, Web, native windows, UDP, GPU, serialization, reactive state and
auxiliary native services are optional. No default executor, thread pool,
process supervisor, global command bus or application-state owner is installed.

A single archive can contain disabled adapter source and optional dependency
declarations. The guarantee is a minimal enabled dependency/compilation graph,
not a feature-trimmed download or a measured binary-size/zero-overhead claim.
View consumers still compile the built-in Core module in the same package;
View must have no semantic dependency on composition or Host lifecycle.

### 15.2 Current source to target ownership

This table records the inspected baseline responsibilities. Links follow current
file locations after a move; the later implementation checkpoints record which
parts of the target have actually been delivered.

| Current source | Target responsibility / migration | Owning item |
|---|---|---|
| [Core module/exports](../crates/yetlet/src/lib.rs), [composition](../crates/yetlet/src/composition/mod.rs), [resource engine](../crates/yetlet/src/composition/resources.rs) | Built-in std-only lifetime coordination. Generic HostGroup/Host/ResourceGroup belong here; concrete adapters depend on this layer. | YT-288 |
| [Private facade](../examples/host-lifecycle-research/src/lib.rs), [resources](../examples/host-lifecycle-research/src/resources.rs) | Promote the proven ownership semantics, not a blind copy of the io::Error-bound Activity, blocking sleep driver or one-to-four tuple limit. Resolve those public signatures through compiling consumers. | YT-288 |
| [View](../crates/yetlet/src/view/mod.rs), [tags](../crates/yetlet/src/view/tags.rs), [binding](../crates/yetlet/src/view/binding.rs), [HTML](../crates/yetlet/src/html/mod.rs) | Optional yetlet::view and yetlet::html. View is an HTML-like owned tree used by HTML/DOM; components remain ordinary functions. Native GPU scenes are separate. | YT-289 |
| [App](../crates/yetlet/src/web/app.rs), [HTTP/Page/prelude](../crates/yetlet/src/web/mod.rs) | yetlet::web owns App, Router/Tower interoperability, WebHost and serving drivers. Page/rendering/View-dependent prelude entries require HTML as well as Web. Preserve resolved App state before serving/hosting. | YT-290, YT-291 |
| [Current Web adapter](../crates/yetlet/src/web/worker.rs), [command bridge](../crates/yetlet/src/command_bridge.rs) | Move the actual HTTP worker/serving implementation to Web. Move the separate generic command/snapshot bridge to optional yetlet::command_bridge; do not move it into HTTP merely because the current file contains both. | YT-290, YT-292 |
| [Actions](../crates/yetlet/src/actions/mod.rs), [Browser HTTP executor](../crates/yetlet-browser/src/actions/mod.rs) | Optional neutral command/executor semantics plus separately selected target-neutral HTTP errors/envelopes; native serving and browser Fetch stay in their adapters. | YT-290 |
| [Reactive](../crates/yetlet/src/reactive/mod.rs) | Keep the optional synchronous caller-local graph. Application domain state, sharing and synchronization remain application-owned. | YT-289 regression scope |
| [SDK capability manifest](../crates/yetlet/Cargo.toml), [current adapter exports](../crates/yetlet/src/lib.rs) | Move Native/Winit, UDP and native wgpu implementations into separately enabled yetlet modules; keep renderer and Host choices independent. | YT-292 |
| [Browser manifest](../crates/yetlet-browser/Cargo.toml), [Browser exports](../crates/yetlet-browser/src/lib.rs) | Keep the browser package and its DOM/action/GPU adapters. Forward features to only their needed target-neutral yetlet modules; gate current unconditional View reexports. | YT-289, YT-290 |
| [Core capability manifest](../crates/yetlet/Cargo.toml) | Retain independent native configuration, secret/provider and signing opt-ins; classify existing HTTP capabilities explicitly below. Static asset production stays application/CLI-owned rather than inventing a new Core asset API. | YT-290, YT-293 |

The secret/provider classification uses the observed manifest, module gating
and existing configuration contract. It is not a review of secret-handling
implementation or a new security qualification.

### 15.3 Target feature and import matrix

Names below are the selected target vocabulary; complete manifests, Rust
generic signatures and dependency flags must compile in their owning items.
No new feature is available merely because it appears in this table.

| Package feature | API responsibility | Required capability edges / exclusions |
|---|---|---|
| yetlet default [] | Core composition; root HostGroup / ResourceGroup exports | std only; no View, HTTP/runtime, serde, native window or GPU dependency |
| view | yetlet::view; gated View/Element/Attribute reexports | No external dependency and no lifecycle use |
| html | yetlet::html pure escaping/serialization | Enables view; no Web or browser runtime |
| web | yetlet::web::App, WebHost, native HTTP/Router/Tower serving | Native Axum/Tower/Tokio/serde; no automatic html, actions, state graph, GPU or Winit |
| web + html | Page, page/render helpers and View-related Web prelude | The SSR selection is this feature combination; no extra SSR package or required convenience feature |
| actions | yetlet::actions::Command / ActionExecutor / LocalExecutor | No mandatory serde or transport error type; local futures need not be Send |
| actions-http | yetlet::actions_http HTTP errors and serializable envelope | Enables actions and serde; target-neutral, no native Web/Axum/Tokio or browser Fetch |
| reactive | yetlet::reactive local graph | No scheduler, I/O, View or shared-Host state service |
| command-bridge | yetlet::command_bridge bounded command admission and immutable snapshots | Optional std-based native bridge, initially at its proven native scope; independent of HTTP and ActionExecutor |
| udp | yetlet::udp caller-driven socket adapter | Native std networking; no Tokio/Web/window/GPU dependency |
| native-winit | yetlet::native_winit native driver/windows | Winit only; no automatic Web, UDP or renderer |
| renderer-wgpu | yetlet::renderer_wgpu native rendering | Native wgpu only; no automatic Winit, HTTP or View |
| yetlet-browser dom / dom-keyed | Browser DOM renderers and View reexports | Select yetlet/view and their existing browser dependencies; never yetlet/web |
| yetlet-browser actions | Browser HTTP executor | Select yetlet/actions-http plus target-specific Fetch/serialization dependencies; never native Web |
| yetlet-browser wgpu / wgpu-browser | Browser scene contracts / optional GPU backend | Preserve their separate scope without selecting View/DOM solely for package reexports |

Keep the browser package's current DOM convenience default. Its default-off,
actions-only and GPU-only consumers must remain valid with only the features
they need. Keep native dependencies behind native target conditions and browser
bindings behind their actual Wasm target conditions. A native feature's absence
on Wasm must be documented; it is not evidence that native behavior ran there.

Cargo features are additive and can be unified through another dependency.
Features expose capabilities; they cannot choose mutually exclusive execution
owners. Select execution through driver APIs and validate native-loop conflicts
at plan/start time. Verify minimal consumers in separate Cargo invocations, not
only an all-features workspace build. Normal dependency graphs must exclude
test/dev feature leakage. See the primary [Cargo features contract](https://doc.rust-lang.org/cargo/reference/features.html#feature-unification)
and [resolver rules](https://doc.rust-lang.org/cargo/reference/resolver.html#feature-resolver-version-2).

### 15.4 Existing optional native capabilities

Keep these existing feature names unless implementation exposes a concrete
reason to change one. Module residence alone must not broaden its dependency
closure. Existing middleware installation, authentication/authorization and
provider ownership remain explicit.

| Feature family | Target ownership and dependency rule |
|---|---|
| request-id, session | Optional HTTP capabilities; replace their server-axum edge with web and omit HTML. Preserve existing request/cookie/CSRF contracts. |
| auth | Existing optional HTTP authentication adapter; retain its narrow Axum dependency rather than forcing the full Web runtime. No mandatory Core Principal service. |
| auth-session | Explicit auth + session bridge; no automatic installation or application authorization. |
| session-redis, session-redis-tls | Optional native Web Session provider and TLS selection. Preserve the provider's actual runtime requirements; its current multi-thread Tokio feature is not a Core/basic-Web pool policy. |
| rate-limit | Optional HTTP/Tower admission with Governor; keep its existing narrow dependencies, without forcing HTML or an unrelated runtime. |
| config | Independent native startup configuration; no Web dependency. |
| secrets, secret-provider | Independent native secrets and explicit config/secrets bridge, based on the observed feature contracts. |
| signing | Independent native byte signing/verifying and caller-owned trust; OpenSSL remains opt-in. |

### 15.5 Commands, state and components

A Command is an application operation with a typed response. It is not a DOM
event, Winit event, lifecycle stop request or mandatory cross-Host message.
An ActionExecutor chooses how that operation runs. Change the neutral executor
contract to an associated error type so local/custom executors can return their
own domain errors; preserve local non-Send futures and application-owned state.

The current ActionError includes endpoint/network/remote-HTTP variants, and the
same module serializes ApiErrorEnvelope/ApiErrorBody. Assign those transport
types to actions_http, with explicit application-error mapping at HTTP adapters.
Preserve the existing wire envelope unless a separately justified protocol
change is selected. Moving serde derives alone does not make the fixed-error
ActionExecutor transport-neutral. LocalExecutor, BrowserHttpExecutor and the
existing custom-action-transport/Todo/Board consumers need compile coverage for
the associated-error change in YT-290.

The optional command/snapshot bridge is a different contract: queue admission
does not prove command execution, and published snapshots are immutable views of
an application-owned authority. It does not require ActionExecutor and is not
installed for every Host. Reactive remains a local Rc/RefCell graph; lifecycle
composition does not synchronize arbitrary domain state. No universal Component
trait, UI event bus or State container is introduced.

### 15.6 One lifecycle, explicitly different drivers

App describes an HTTP service; HostGroup groups Host lifetimes; ResourceGroup
customizes owned/borrowed resources and their explicit management. They are
different developer intentions. Use HostGroup rather than promoting the private
Application spelling, and use with_host rather than required_* on the quick path.
Keep Composition as advanced engine vocabulary.

Construction and configuration are inert. build validates known topology,
resource/driver compatibility and exclusive claims without binding, spawning or
creating Winit. Acquisition stores each original/partial owner before the next
fallible step. Readiness must be observed, then normal admission opens.
Shutdown fences admission, progresses activity stop and original completion,
then releases dependencies under the existing global barrier. Primary and
secondary failures and terminal error versus Pending stay distinct.

| Entry / driver | Selected execution contract |
|---|---|
| App::serve / serve_on | Caller-owned Tokio async driver; no new runtime or dedicated thread. Ordinary HTTP transport convenience, with the cancellation limit below. |
| HostGroup::run_async with compatible Web/local Hosts | Caller-runtime Web by default. Progress yields to that runtime, including current-thread Tokio; no synchronous probe/sleep/join or nested runtime. Exact borrow/extension signatures require proof. |
| Synchronous HostGroup::run with Web | The Web adapter's documented default is its owned service-thread driver; the group retains original handles. Local/core-only Hosts do not thereby acquire a runtime or thread. |
| A NativeHost in synchronous composition | One root-owned native event-loop driver progresses readiness and local cleanup inside callbacks; Web may use its owned service-thread driver, while UDP remains bounded/caller-driven. |
| Advanced explicit Web placement | Expose only actually supported driver choices. A requested placement must succeed or fail with its owner retained; do not silently fall back to a blocking native/caller thread. No generic best-effort process policy. |

Core owns state and obligations, not a Tokio scheduler. Async adapter code owns
its poll/wake integration and supplies truthful progress to the same coordinator.
The two Web drivers share serving/admission/failure/final-release rules, not two
independent lifecycle implementations. A Tokio Handle does not imply ownership
of or permission to shut down the caller's runtime. No default thread pool or
one-process-per-Host mechanism is added. Pools and process hosting still need
separate concrete consumers, ownership and supervision contracts.

#### Cancellation and retained owners

The convenient consuming App::serve(self).await keeps its ordinary transport
scope: dropping its future does **not** promise graceful drain or framework-managed
resource finalization. App::with_state continues to carry application-owned
state; it does not register an asynchronous finalizer. Do not add managed resource
attachments to this consuming convenience without a proven ownership transfer.

Use a retained HostGroup/managed owner and a borrowing async drive for managed
resources or recoverable cancellation. The owner must outlive the cancelled
future. The driver must store acquired/partial resources, original joins,
in-flight cleanup state and terminal outcomes in retained owner storage before
every suspension point that could otherwise lose an obligation. A borrowing
signature alone is insufficient; cancelling a task that also owns the group can
still drop that group. Do not describe dropping the owner as completed shutdown.

After cancelling a drive, the caller can resume the same owner or request and
drive shutdown. Pending and a deadline retain those same owners/dependencies.
Normal terminal return may wait for uncooperative work; it must not hide a
timeout by detaching. YT-291 must test cancellation during acquisition/readiness,
running, signalling, original join observation and final release. A future
supervisor/cleanup transfer is outside this contract until independently proved.

#### Web address and readiness

Preserve 127.0.0.1:7878 for App convenience and select the same default for the
new public WebHost. This deliberately differs from the private proof's port 0.
Multiple Web Hosts use explicit distinct addresses or port 0, with actual bound
addresses observable after start. No silent port change after failure.
Known identical nonzero exclusive endpoints may be rejected at build; OS/DNS
or external occupancy still requires transactional startup rollback.

Default Web readiness means the listener is acquired and the selected serving
driver has actually initialized sufficiently to service admitted work.
Constructing an unpolled serve future or merely allocating a thread is
insufficient. An API-only Router may return 404 at GET / and still be ready.
An optional explicitly configured HTTP probe keeps its exact route/method/status
and startup-only admission exception; it must not open arbitrary application
work before the group is ready. Probe failure remains primary and retains the
original owner for rollback. This is an intentional next-alpha readiness change
from the private root-200 proof, not evidence that the existing adapter does it.

#### Native uniqueness

One NativeHost/root driver owns one process-level Winit event loop and can own
multiple windows. Multiple inert descriptions do not run loops; incompatible
independent-loop requests fail before acquisition when detectable. Enabling
native-winit twice through Cargo does not create two loops. Actual creation
still enforces the platform entry-thread and process-wide limits, including a
loop created outside Yetlet; a completed/dropped loop does not imply permission
to recreate it. See the pinned [Winit 0.30.13 contract](https://docs.rs/winit/0.30.13/winit/event_loop/struct.EventLoopBuilder.html).
Native callbacks must not wait for a future callback to complete readiness or
cleanup. GPU backend qualification remains separate from feature availability.

### 15.7 Intended developer shape

These are target sketches, **not compiling current API examples**. The feature
names/intent are selected here; YT-288/290/291 must prove the exact signatures.

Simple HTTP uses yetlet with the web feature and an application-owned Tokio
runtime. It does not need View or a separate Host package:

~~~rust
use yetlet::web::App;

App::new()
    .get("/health", health)
    .serve()
    .await?;
~~~

This consuming convenience has the cancellation limit above; no managed resource
finalization is promised if its future is abandoned.

Multiple Web Hosts use the same service/Host foundation and one retained owner:

~~~rust
use yetlet::HostGroup;
use yetlet::web::{App, WebHost};

let mut hosts = HostGroup::new()
    .with_host(
        App::new()
            .get("/api", api)
            .into_host()
            .with_address("127.0.0.1:8080"),
    )
    .with_host(
        WebHost::new(admin_router)
            .with_address("127.0.0.1:8081"),
    )
    .build()?;

hosts.run_async().await?;
~~~

The caller keeps hosts outside a cancellable task/scope when cancellation
recovery is required. No root-GET handler is required merely for readiness.

For advanced ownership, a custom Host supplies the same ResourceGroup plan:

~~~rust
use yetlet::ResourceGroup;

let resources = ResourceGroup::new(activity)
    .with_resource(borrowed_or_owned_passive_value)
    .with_managed_resource("database", database_descriptor);
~~~

A passive value uses its normal Rust lifetime; a managed descriptor defines
acquire/release progress. This is not an untyped resource registry or automatic
DI. The supplied activity must still have its actual original completion
contract. Built-in Host defaults use the same machinery internally.

### 15.8 Breaking migration and compatibility ledger

| Current contract / consumer | Required next-alpha change and evidence |
|---|---|
| yetlet default html; unconditional View exports; optional composition | YT-288/289 make Core built in, default empty and View exports conditional. Migrate required-features on tests/examples and consumer manifests. Core-only and HTML-only normal consumers must compile separately. |
| Private Application and current required_* core vocabulary | Promote HostGroup with canonical intent names; migrate active public consumers without permanent synonym aliases. Legacy local close-before-worker behavior cannot be renamed into post-join resource release; map each callback to its actual lifecycle phase. Historical proof names/receipts remain unchanged. |
| server_axum / server-axum; yetlet_hosts Web/native/UDP imports | YT-290/292 migrate active imports/features to yetlet modules. Keep App's resolved-state and Router/Tower escape hatches. Remove the staging companion from the new release path only after its active consumers move. |
| View/HTML-dependent Web methods and preludes | Gate Page, page methods, document rendering and View imports on Web + HTML. A Web API-only compile must not rely on a dev-dependency activating HTML. |
| Browser DOM/keyed-DOM, no-default and GPU-only modes | Forward DOM features to view and gate reexports; preserve GPU-only/no-default builds. Test normal Wasm consumers, not only native test stubs. |
| ActionExecutor/LocalExecutor/custom transports and HTTP envelopes | Associated executor error; explicit transport mapping; shared actions-http wire module without native dependency. Migrate Todo, Board and custom-action-transport callers together and retain wire compatibility tests. |
| Web/SQLite and external custom/native consumers | Reuse old receipts only for unchanged behavior. New facade/driver imports need current-source startup, admission, drain, original completion, partial rollback, dependency release and cancellation evidence. |
| CLI scaffold/templates/resources | YT-293 updates tools/yetlet-cli templates, capability/version metadata, generated Cargo feature selection and fixtures. Inspect generated projects, not only template substitutions. |
| Installed old CLI selecting the newest SDK | [Current selection code](../tools/yetlet-cli/src/sdk_version.rs) checks server-axum for native templates but reactive/dom/dom-keyed for browser templates. Removing server-axum can make native selection reject; it does not prove old Browser templates reject or compile. Test both old-CLI/new-SDK modes and new-CLI/selected-SDK modes. Preserve actionable SDK_INCOMPATIBLE and no silent older-version fallback; unsupported combinations need an explicit upgrade/pin path and publication disposition. |
| Published SDK/CLI, normal archives and reader edition | Keep the [existing two-package cohort](RELEASING.md#current-two-package-development-boundary--yt-211) and exact dependency-order normal archive verification. Do not invent a new cohort schema when names/count stay unchanged. Current README/module guides/templates update with implementation; the separately pinned public guide changes only after actual accepted publication. |

No permanent old-path aliases are selected. This is an announced experimental
next-alpha source break, not a silent claim that published alpha.8 has changed.
The precise release version, registry upload, CLI publication and live reader
promotion remain outside YT-287. A newly generated project that passes only
metadata selection but fails to compile is not a compatibility PASS.

### 15.9 Acceptance and proof handoff

YT-287's deliverable is this source-backed contract plus the corresponding
architecture, naming, ADR, development-intent and task-state updates. Its
independent Reviewer must check the actual feature/ownership/migration decisions.
Documentation validation checks the changed English Markdown, local links,
anchors, whitespace and the backlog DAG; historical unchanged failed receipts
remain preserved. No Rust behavior or runtime result is claimed by this item.

Configured independent R1 `wc_job_GwI4V4CCnZesLj9w` completed with exit0 and
actual PASS, `issues=[]` and `required_followups=[]`, on the seven-document
contract snapshot. Observed model/effort was gpt-6.2-astra/high. Scoped English,
809 local links/anchors, the 291-item backlog DAG and whitespace checks passed.
Receipts are retained under `.pipeline-output/m28-yt287-20261121/`.
YT-287 is Done at documentation-contract scope; these completion annotations
record that result without changing the reviewed technical contract.

The implementation boundary is deliberately staged: YT-288 establishes built-in
composition and the generic Host/resource contract without claiming final View
exclusion. YT-289 changes the default set and conditional View exports together
with Browser feature wiring, so the latter is not left relying on an unavailable
unconditional export. No intermediate slice selects a release.

The next implementation item is YT-288. Its compiling proof must settle generic
error/Host extension signatures, typed owner storage and driver extension shape,
including cancellation-safe owner placement. Later driver acceptance is YT-291;
Native/UDP/public adapter qualification is YT-292. The private four-Host limit,
live managed-owner adoption, arbitrary resource DAGs, automatic pools,
process placement and broad platform qualification are not silently promoted.

## 16. YT-288 — built-in Core Host lifetimes

**Source baseline:** YT-287 commit `23f8d3df97b082365e6c9970bd7f48ef270d8da7`.
**Scope:** Core implementation and compiling consumer; independently reviewed and complete at this scope.
This section records the implemented Core surface. Section 15 continues to own
the later Web, View, native and publication gates.

### 16.1 Built-in surface and minimal dependencies

Core composition is now available without a Cargo feature. The obsolete
`composition` feature and its active dependency/test/example selections are
removed. Root exports are `yetlet::{HostGroup, Host, ResourceGroup}`; advanced
types remain under `yetlet::composition`. No extra SDK package is introduced.

The current checkout still defaults to HTML and still exports View
unconditionally. YT-289 changes both with Browser wiring. The independent
[normal consumer](../examples/external-consumers/yt288-core-hosts/Cargo.toml)
selects `default-features = false`; its normal/build dependency graph contains
only that consumer and yetlet. This proves no mandatory runtime, HTTP, serde,
Winit or wgpu dependency; it does not yet prove View exclusion or registry
package qualification.

### 16.2 Concrete public extension contract

[HostGroup and Host](../crates/yetlet/src/composition/hosts.rs) use a recursive,
statically typed list. There is no fixed one-to-four limit, erased resource
registry or blanket Send/Sync/'static bound. Each group uses a common
application-selected error type. A custom adapter maps its own concrete errors
into that type when needed; Core does not require io::Error, Error, Clone,
boxing or string conversion.

A Host supplies its diagnostic name, optional inert validation and exact
exclusive claims, then returns `ResourceGroup<Activity, Resources>`. The
activity implements HostActivity; explicit resource descriptors implement
ManagedResource. ResourceGroup::new(activity) uses no extra resource management
configuration. Advanced callers append passive owned/borrowed values with
with_resource and inert acquire/release descriptors with with_managed_resource.
The provided ResourcePlan lists are sealed implementation support, not another
lifecycle-extension engine.

new, with_host and build acquire nothing. build validates the entire plan
before conversion. Equal exclusive-claim kind/key pairs reject the inert plan
with both Host names. Claims do not canonicalize endpoints, reserve OS
resources or detect an event loop created outside the group; actual adapter
acquisition retains that responsibility.

The compiling [external example](../examples/external-consumers/yt288-core-hosts/src/main.rs)
creates five different typed Host plans, a borrowed mutable managed journal,
local Rc values, borrowed configuration and an application error borrowing
a runtime-created string. It also compiles generic builder/driver helpers
against the provided sealed HostList/HostParts bounds. Custom authors implement
Host, HostActivity and ManagedResource, never the private traversal.

### 16.3 Retained owner and caller-owned driving

build returns BuiltHostGroup. Its relevant operations are:

| Operation | Actual Core behavior |
|---|---|
| poll_start(cx) | Progress ordered acquisition and observed readiness; return Pending or StartStatus. Started activities remain observed while a later Host is Pending. Only all-ready plans open normal admission. |
| poll_run(cx) | Progress startup, every running activity and coordinated shutdown once. The first observed activity termination stops the group; its actual outcome still belongs to original Join. |
| run_async(&mut self) | Borrowing std Future drive over poll_run. No runtime, timer, thread or sleep loop. Returns only after terminal cleanup, borrowing the primary HostFailure on failure. |
| request_shutdown() / ShutdownHandle | Request coordinated stop; the shared handle wakes the currently registered drive. No cleanup task or owner transfer is created. |
| poll_shutdown(cx, deadline) | Advance the same Core barrier with a real wake context and cooperative absolute deadline. Pending retains original obligations. |
| shutdown_until(deadline) | One explicitly caller-driven pass with a no-op wake context. The caller arranges another event/poll when necessary. |
| failures() / outcomes() | Borrow original primary/secondary errors and inspect startup-attempt, Join and activity-release outcomes without replacing owners. |

There is no generic synchronous run loop in this Core slice. Concrete native
or owned-thread Web drivers remain YT-291/292 work. The generic borrowing
run_async entry is implemented, but it is not evidence that current Web/native
adapters already implement the new Host trait.

HostActivity stores partial startup and terminal execution outcomes before
returning. poll_finished reports availability of termination only; poll_join
transfers the actual result exactly once. Core stores the error itself, with
Host/resource/operation context. A terminal execution error is distinct from a
retryable observation error.

Admission fences and stop requests are bounded synchronous operations. They
receive the current Context so a retryable error can register notification for
later progress. They must not report success before the fence/signal exists.
Drain and original completion are polled separately. Pending or retryable error
must arrange a wake, or the application must explicitly arrange a future poll;
Core does not install a polling timer or promise progress from an inert waker.
RawWaker clone/drop/wake callbacks run outside the shutdown-handle mutex.

### 16.4 One shutdown engine and cancellation boundaries

The [Composition coordinator](../crates/yetlet/src/composition/mod.rs) remains
authoritative. The private callback adaptation passes Context through existing
admission, signal, original observation and final-release traversal. The public
advanced synchronous callback signatures and their local close-before-worker
semantics remain unchanged; no legacy Local callback is relabelled as a
post-Join resource finalizer. The new default/custom Host path uses its
intent-specific names without required_* markers or compatibility aliases.

All admission must close before any local stop or activity signal. Observation
can join an already-finished original even when signalling failed. Every local
close and original Join across the whole group must complete before final
release. Activities retire before their earlier dependencies; resources and
groups release in reverse order. Pending and retryable release error block
earlier dependencies. Released(Err) is terminal, is recorded once and does not
repeat a completed finalizer.

Acquisition-attempt state is stored before the first descriptor poll, so even
partially acquired descriptors participate in rollback. Later unattempted
descriptors skip managed release. Original activities, managed descriptors,
in-flight state and original errors stay in the retained owner rather than
inside a disposable borrowing drive.

The focused cancellation test drops and recreates only run_async's borrowing
future at pending acquisition, activity start, readiness, running, original
Join, activity release and resource release. It verifies the same owner's
completion without reacquisition. This establishes the Core/custom-extension
storage path; Web/Tokio cancellation during its actual driver operations remains
YT-291's separate acceptance.

Dropping the owner clears Core's waker registration outside its mutex, then
performs ordinary Rust field destruction. It does not signal, join or run
managed finalizers and may detach an application-supplied worker. The caller
must keep that owner outside a cancelled task/scope.
Callbacks must obey their nonblocking, retained-owner and truthful-completion
contracts; Core does not catch panics, preempt callbacks or guarantee that an
uncooperative operation terminates.

### 16.5 Verification and delivery boundary

The changed Core tests cover inert validation, known exclusive claims, pending
readiness, earlier termination during later startup, original non-Clone error
identity, global Join/release ordering, partial-acquisition rollback, seven
cancellation points, current-waker fence retries, failed-signal original Join,
retryable/terminal release, Send/Sync preservation, reentrant waker destruction,
owner-drop waker-cycle release and the ordinary Drop boundary. Existing lifecycle tests retain real original
thread handles and legacy callback regression coverage.

Focused native results: 21 existing composition tests, 14 HostGroup tests,
11 no-default doctests, 8 UDP and 9 Web adapter tests pass. The separate
five-Host executable, its generic driver helpers and minimal normal/build graph
pass. Strict Core/consumer Clippy, changed-source formatting, private/Audit/compat
consumer compilation and Browser wasm32 compilation pass. Wasm is compile-only
evidence, not browser execution or a qualified browser Host driver.

The original Clippy type-complexity and test-Waker lint failures are retained;
an internal type alias and meaningful fixture wake behavior repair them. Strict
rustdoc initially found 92 invalid-HTML diagnostics in the unchanged typed-tag
macro. Backticks in its two generated documentation strings fix that actual
documentation defect without changing View behavior; strict rustdoc now passes.
Independent R1 found a Core waker/ShutdownHandle reference cycle on owner Drop.
The owner now clears its registration without lifecycle callbacks; a Weak-based
regression verifies destruction after one poll, without replacement polling or
explicit shutdown. Focused tests, consumer execution, strict Clippy and doctests
pass after the fix. Configured independent R2 `wc_job_EGI9SDVODJoAu71u`
terminated exit0 with actual PASS, issues=[] and required_followups=[]. Nine-doc
links/anchors, the backlog dependency DAG and whitespace checks pass. YT-288 is
Done at this Core implementation scope; completion-only annotations follow review.
Receipts are retained under `.pipeline-output/m28-yt288-20261121/`.

Published alpha.8, CLI versions, pinned reader, Web/native adapter delivery,
normal archive qualification and physical LAN evidence are unchanged. The next
ordered item after YT-288 is YT-289.

## 17. YT-289 — optional View and Browser feature isolation

**Source baseline:** YT-288 commit `a989c0c9c93b16315fdf0c1665ecaa6f0eed6a77`.
**Scope:** Current-checkout feature isolation and existing consumer migration;
focused verification and configured independent review PASS; YT-289 Done.

### 17.1 Implemented feature boundary

The current yetlet manifest selects `default = []`. Built-in composition and
the root HostGroup, Host and ResourceGroup API remain available. View is an
optional HTML/DOM value layer in the same package; it adds no semantic dependency
on the lifecycle coordinator. No new published package is introduced.

| Selection | Current API / dependency effect |
|---|---|
| yetlet default / no-default | Built-in std-only Core; no View, HTML, runtime, serde, native window or GPU dependency. |
| yetlet view | View/Element/Attribute, typed tags and binding vocabulary; no renderer or external normal dependency. |
| yetlet html | Enables view and the existing pure HTML serializer; no Web/browser runtime. |
| yetlet reactive | Existing independent local graph; no View or renderer selection. |
| Browser default / dom | Retains default DOM convenience and enables yetlet/view; no HTML or native Web. |
| Browser dom-keyed only | Independently enables yetlet/view and keyed bindings; does not depend on the dom feature being selected. |
| Browser no-default | No adapter or View reexports. |
| Browser actions only | Existing portable action/serde plus target-specific Fetch closure; no View, HTML or native Web. |
| Browser wgpu / wgpu-browser only | Existing CPU scene / optional Wasm backend; no View or DOM selection. |

Browser reexports of view, View, Element and Attribute require dom or dom-keyed.
Enabling yetlet/view through some other dependency alone does not enable those
Browser reexports. Set default-features=false for actions-only/GPU-only Browser
selection; keeping Browser defaults deliberately includes DOM.

Features remain additive. An application can enable more capabilities through
another dependency; isolation evidence therefore uses separate Cargo invocations
and a separate workspace, with normal/build edges and resolved features recorded.
This is an enabled-graph guarantee, not a feature-trimmed archive or binary-size
claim. Published alpha.8 retains its original HTML default and View exports.

### 17.2 Existing consumers and documentation

Three path consumers require explicit View after the default change:

- [Todo UI](../examples/todo-ui/Cargo.toml): normal actions + view dependency;
  its dev-only html selection must not mask a missing normal View edge.
- [Style Showcase](../examples/style-showcase/Cargo.toml): target-neutral view
  for its shared library; native server-axum cannot supply that edge on Wasm.
- [Style authoring fixture](../experiments/style-authoring/Cargo.toml): view
  for typed authoring, without adding HTML serialization.

Existing DOM/keyed consumers receive View through their explicitly selected
Browser adapters. Server consumers still receive HTML/View through server-axum
in this slice. Registry-pinned alpha.8 examples remain on that published feature
contract; they must not request the checkout-only view feature.

Package READMEs and View/HTML guides distinguish current source from published
artifacts. Their HTML dependency examples explicitly select html, which works in
both scopes. Root rustdoc examples and links remain valid when optional View or
DOM APIs are absent. No version, public reader or release identity is promoted.

### 17.3 Verification and limits

The View, tags, binding, HTML, reactive, DOM and keyed-DOM implementation sources
are unchanged from the baseline. Focused existing native tests exercise their
value, escaping, output-policy and reconciliation contracts under separate
feature selections. Core default, View-only and HTML-only consumers are compiled
separately; Browser default, keyed-only, actions-only and GPU selections are
likewise isolated. Disabled-surface probes must fail with the intended missing
import/module diagnostics, not merely an arbitrary nonzero Cargo result.

Wasm library compilation is separate evidence from actual browser execution.
This feature migration does not claim new DOM mounting/listener/GPU runtime,
CSP-denial, native platform or normal archive qualification. Existing target-
specific test-only dependencies are not widened to claim Wasm test execution.

The implementation preserves server-axum -> html, current action protocol types
and their serde edge. YT-290 owns Web module/HTTP protocol isolation; YT-291 owns
Web driver/cancellation integration. CLI/general migration and archive acceptance
remain YT-293/294. The next ordered item after YT-289 is YT-290.

Focused native tests pass with separate selections: 21 composition and 14
HostGroup tests under the new default, 12 View-only, 23 HTML, 16 Reactive,
20 default Browser DOM and 1 keyed-only test. The three changed consumer paths
compile, including native/Wasm Style Showcase and its keyed Browser companion.

The isolated consumer passes 21 positive selections: 11 native executions and
10 Wasm compilations, each with a normal/build feature graph. Core/View/HTML
have no external normal dependency; Browser actions/GPU selections omit View/
HTML and native HTTP; normal Todo UI gets View without its dev-only HTML edge.
Seven native disabled-surface probes return the intended missing-import
diagnostics (E0432/E0433), proving the absent API boundary.

Strict Core/Browser Clippy, changed-source formatting, six feature-selected
doctest sets and six strict rustdoc builds pass. Wasm results are compilation
only. Verification and review receipts are retained in
`.pipeline-output/m28-yt289-20261121/`. Configured independent review
`wc_job_THsX4ibX3uyjN_qf` completed with exit0 and actual PASS, issues=[] and
required_followups=[]. YT-289 is Done at this feature-isolation scope;
completion-only annotations follow review.

## 18. YT-290 — optional Web and HTTP protocol ownership

**Implementation baseline:** YT-289 commit `39978af7aadb145d6136047ced07a36b1622e901`.
This checkpoint implements the Web/protocol slice of section 15. It preserves
the two-package SDK cohort and published alpha.8/CLI identities. YT-290 is Done
at the independently reviewed Web/protocol scope; this is not a release record.

### 18.1 Current feature and ownership boundary

| Selection | Current checkout surface | Dependency boundary |
|---|---|---|
| yetlet default [] | Built-in Core composition and generic HostGroup/ResourceGroup | Unchanged std-only foundation. |
| actions | Command, ActionExecutor and LocalExecutor | No serde or HTTP error; the executor selects its associated Error. |
| actions-http | actions_http::ActionError, ApiErrorEnvelope and ApiErrorBody | Enables actions and serde; target-neutral, without native Web or browser Fetch. |
| web | web::App, request/extractor/response/middleware APIs and advanced HttpWorker | Native Axum/Tower/Tokio/serde; no automatic View, HTML, actions, Reactive, Winit or GPU. |
| web + html | Page, document metadata/render helpers, App::page and View-related prelude entries | HTML explicitly enables View; API-only Web omits this surface. |
| yetlet-browser actions, defaults disabled | Explicit same-origin HttpExecutor and its neutral/wire reexports | Selects actions-http; native Web and View remain absent. |

The former server_axum module is physically moved to
[web](../crates/yetlet/src/web/mod.rs); the server-axum feature and old public
module path are removed without aliases. Existing internal test target names
and the server-axum.md guide filename remain identifiers for their retained
tests/navigation, not public compatibility APIs. Current module guides explain
the source/published distinction.

[App](../crates/yetlet/src/web/app.rs) still consumes a resolved App<()> on the
caller's Tokio runtime, with the unchanged 127.0.0.1:7878 convenience default.
Router conversion, native extractors/handlers and Tower middleware stay explicit.
Page/rendering and View exports require HTML; string/JSON routes need only Web.

### 18.2 Serving implementation and lifecycle scope

The actual [HttpWorker](../crates/yetlet/src/web/worker.rs) implementation moves
out of yetlet-hosts. App and that worker call one private
[serving function](../crates/yetlet/src/web/serving.rs). The base package has no
reverse dependency on yetlet-hosts. The former web_axum staging module retains
only its separate std-based command/snapshot bridge; its later relocation is
YT-292, not an HTTP concern.

HttpWorker remains an advanced, explicitly started owned-thread adapter with
its existing loopback/port-0 default, explicit HTTP-200 startup probe, stop
signal and original-join behavior. The move does not redefine those semantics
as the new public WebHost defaults. App::serve retains ordinary future
cancellation; dropping it does not prove graceful drain or managed finalization.
YT-291 still owns the managed WebHost facade, caller-runtime/owned-thread
lifecycle integration, new readiness/address policy and cancellation proof.

### 18.3 Neutral commands, wire compatibility and consumers

[ActionExecutor](../crates/yetlet/src/actions/mod.rs) now has an unconstrained
associated Error. LocalExecutor propagates the supplied handler error and
retains caller-owned, non-Send state/futures. The
[HTTP wire module](../crates/yetlet/src/actions_http/mod.rs) preserves the existing
ActionError variants, error codes and serialized envelope fields.

Todo UI uses its own domain error; the server maps it explicitly at HTTP.
Browser-local execution keeps a local error while the remote executor returns
the HTTP error. The custom TCP consumer uses its own transport error and
preserves its existing TCP reply protocol without inventing an HTTP status.
Board consumers explicitly select HTTP errors/envelopes where their feedback
actually depends on HTTP status. Browser Fetch behavior is unchanged.

Active path consumers move together: API-only examples select web, SSR users
select web + html, and worker consumers import yetlet::web::HttpWorker.
Request ID, Memory Session and Redis Session select web without HTML. Auth
keeps its narrow Axum edge; rate limiting keeps Axum/Tower/Governor; independent
configuration, secret/provider and signing edges remain independent.
This classification does not newly qualify external providers.

Published registry examples, pinned reader content and CLI templates retain
their selected versions. General migration/old-CLI selection is YT-293 and
normal-archive acceptance is YT-294. Obsolete excluded experiments are not
silently promoted as compiling current consumers.

### 18.4 Verification and delivery boundary

The first eight focused checks pass: API-only App serving/configuration,
Web + HTML behavior and Router/Tower/state compatibility, neutral and HTTP
actions, Browser actions, request/session middleware, the moved worker and
the migrated Todo/custom-TCP/Board consumers. The latter five packages pass
47 tests; the moved worker passes nine real HTTP/stop/join tests.

An isolated consumer passes nine native executions and five Wasm compilations,
each with a normal/build graph. Core and neutral actions have no external normal
dependency; Web omits View/HTML/actions and basic Web does not enable Tokio's
multi-thread runtime feature. Actions HTTP and Browser actions omit native Web.
Six additional graph-only checks preserve narrow Auth/rate-limit/configuration/
secret/provider/signing selections. Eight disabled-surface probes produce the
intended E0432/E0599 diagnostics, including removed names and Wasm-native gates.

All 31 workspace packages compile with all targets. Eleven changed standalone
consumers compile or pass their selected tests, including three current-thread
Web composition tests, eleven F1 Web tests and the Web/SQLite facade's 13 lifecycle,
six resource and five integration tests. Auth/Session HTTP adds 22 passing
tests. Actual Todo Browser and Board Wasm libraries compile separately.

The first external-consumer pass retained one lockfile failure for YT283.
Cargo reconciliation removes only obsolete Hosts dependency edges and the
unused Hosts package in the lifecycle consumer; it adds/upgrades no dependency.
Both affected consumers then pass with locked resolution. The original failed
receipt remains. Strict Core/Browser/worker Clippy, 93-file changed-source Fmt,
four feature-selected doctest commands and five strict rustdoc builds pass.
The scoped documentation check's four missing EOF newlines were repaired.

Configured independent R1 `wc_job_OIrnOoBP9snNOTBl` returned FAIL for a concrete
active-consumer omission: the [qualification workflow](../.github/workflows/qualification.yml)
and its [existing feature-selection test](../xtask/tests/release_preparation.rs)
still selected the removed server-axum feature. Both now select web; the
workflow already selects html. The focused existing xtask test and an actual
all-target compile using the workflow's exact feature selection pass.
No hosted workflow or additional platform execution is claimed; R1 is retained.

These are native tests, source checks and Wasm compilations, not new browser
Fetch, native-window/GPU, production-provider or publication qualification.
Receipts are retained in `.pipeline-output/m28-yt290-20261122/`.
Configured independent R2 `wc_job_3w96W4sl3X2sGtmx` completed with exit0 and
actual PASS, issues=[] and required_followups=[]. Scoped R3 checks pass for
27 documents, 1,136 local links, 681 anchors and the 291-item backlog DAG.
YT-290 is Done at this bounded implementation scope. Completion-only annotations
follow review; Strict/Builtin staged Guard remains the final local commit gate.

## 19. YT-291 — managed Web lifecycle and drivers

**Source baseline:** YT-290 commit `ef1c179f70a8b34359595e22d2bf41f73f58010c`.
**Scope:** managed Web implementation and public consumer, independently reviewed
and Done. Native/Network integration and normal package qualification retain
their later gates.

### 19.1 Ordinary entry and advanced customization

`App<()>::into_host()` and `WebHost::new(axum::Router)` create inert HTTP Host
descriptions. The Web feature exposes these without View or HTML. Both direct
App serving and the managed drivers retain the Web-owned Axum serving foundation;
the managed adapter adds admission and retained lifecycle integration through
the existing Core coordinator.

The ordinary path is `HostGroup::new().with_host(web).build()`, followed by a
drive of the retained owner. There is no required-participation marker,
Application alias, second resource registry or new published SDK package.
App retains Router/Tower/state interoperability and its consuming serve
convenience. Cancelling that consuming convenience still does not promise
managed drain or resource finalization.

WebHost defaults to `127.0.0.1:7878`. `with_address` accepts an explicit
address; numeric nonzero SocketAddr values supply exact exclusive claims.
Port 0 permits multiple Hosts. Hostname resolution and external occupancy
remain startup operations, and failures never select a different address.
Capture `handle()` before moving a Host into the group. Its `local_addr()`
reports the actual endpoint, while `is_ready()` and `is_admitting()`
distinguish serving readiness from aggregate admission. An old endpoint remains
metadata after shutdown, not a claim that the listener is live.

A custom Host can obtain the configured ResourceGroup, transform its inert
activity with `map_activity`, and attach passive values or managed descriptors.
The transform retains the dependency plan; it does not remap the errors of
already attached managed descriptors. Each HostGroup still uses one
application-selected error type with no blanket Clone/Send/Sync/'static bound.
WebError retains the original I/O cause; application mapping can remain local
to the retained owner instead of crossing the service-thread boundary.

### 19.2 Explicit driver preparation

Core adds `HostRunMode::{Async, Blocking}` and an inert `prepare_driver`
hook for activities and managed resources. The complete plan is prepared before
the first resource acquisition. A preparation failure becomes the original
typed HostFailure and enters the existing rollback path. The selection is
latched in the retained BuiltHostGroup, not inferred from the presence of a
Tokio handle.

| First drive / explicit selection | Web execution |
|---|---|
| `run_async` or caller polling | Caller-owned Tokio; current-thread runtimes are supported. |
| Blocking `run` | One owned service thread with its current-thread Tokio runtime, plus one dedicated join observer per WebHost. |
| `with_driver(WebDriver::OwnedThread)` | Explicit owned thread; async observation still yields to the calling Tokio runtime. |
| `with_driver(WebDriver::CallerRuntime)` with blocking drive | Rejected during preparation, before any dependency is acquired. |

Cancelling a borrowing drive does not reset placement. Blocking `run` rejects
a still-live Async-selected owner with `HostRunError::ModeMismatch`, leaving
its admission and obligations untouched. Resume its async drive instead.
The existing `run_async -> Result<(), &HostFailure<E>>` contract remains:
terminal lifecycle errors are borrowed from the retained owner. An already
selected blocking placement is not silently replaced if later async polling
observes that same owner.

Core's synchronous loop uses a genuine thread waker and bounded parking.
It starts no runtime, pool, helper thread or process. This loop is not a Winit
event-loop implementation. Native callback ownership and platform affinity
remain YT-292.

### 19.3 One retained Web activity

The caller-runtime driver stores its pinned service future in WebActivity.
The owned-thread driver runs the same service implementation and retains its
original std JoinHandle through an owned join observer. Bind progress,
readiness/probe progress, stop state and terminal outcomes stay in retained
storage across cancellation.

A listener wrapper acknowledges initialization only after the delegated accept
future has actually been polled. A bound listener, allocated thread or
unpolled serve future is insufficient. The adapter's outer admission gate
returns 503 without entering the application Router until Core opens admission.
Default readiness needs no successful GET / route; an API-only Router may
return 404 there.

The advanced activity hook reports `AdmissionOutcome::Opened` or `Closed`.
`Closed` means termination has fenced admission; it is neither temporary
unreadiness nor completed Join. Core stops opening later Hosts, fences any
already opened gates and performs coordinated shutdown through the existing
original-Join barrier. Web may report its shared closure before the owned
thread's TLS and original Join finish. That notification starts rollback without
inventing an admission error; the original execution result remains authoritative.
The same outcome handles closure between the preceding termination poll and the
atomic admission attempt. Real admission-operation errors remain typed failures.

An explicitly configured HttpProbe checks its exact method, path-and-query,
status and retained deadline. Successful exchange observation also checks the
absolute deadline, because Tokio's timeout polls the exchange before its timer.
A matching response buffered after expiry cannot pass readiness when a cancelled
drive resumes. Its method/target is the narrow startup admission
exception; ordinary application work remains fenced. The server continues
progressing while the asynchronous probe waits. Shutdown removes that exception,
fences new work, signals the same service, and waits for admitted work and its
original completion before final dependency release.
The drain guarantee covers Axum-tracked HTTP requests/connections. Detached
handler tasks and upgraded/application-owned work require their own explicit
managed completion obligations; Router interoperability does not register them.

The caller performs no blocking HTTP probe, sleep or original service Join.
R1 found that the service thread's is_finished flag can precede application
thread-local destruction. A timer cannot make that original Join nonblocking.
OwnedThread therefore creates a dedicated join observer before starting its
service thread. The observer waits on a std-only bootstrap latch, consumes the
exact original Join, and returns its raw result. Startup failure closes the
latch and keeps the already-created observer retained until completion.

The observer is a closed implementation surface: while its owner is retained,
it executes no application code, Tokio runtime, waker, panic-payload formatting
or application-result destruction. The service thread sends its existing exit
notification after the serving future ends or unwinds and before owned-runtime
destruction. Runtime teardown can wait for application blocking work; that wait
must not postpone the wake that lets Core fence sibling Hosts. The early guard
still notifies if runtime construction fails. The notification leaves original
Join pending through runtime and TLS destruction. The caller converts the raw
result only after observing the observer's std-only exit tail. Arbitrary panic payloads stay retained in the WebError source rather
than running their custom destructor during a completion poll. Their destruction
follows ordinary error-owner Drop. After the service-body notification, async
observation uses yielding timer rechecks until the observer finishes; blocking
driving uses parked re-polls. This relies on std's documented completion
observation for a thread without application TLS, not a hard OS latency bound.
The [std Join contract](https://doc.rust-lang.org/std/thread/struct.JoinHandle.html#method.join)
includes TLS destruction; [is_finished](https://doc.rust-lang.org/std/thread/struct.JoinHandle.html#method.is_finished)
only observes the thread's main function.

The cost is two dedicated threads for OwnedThread, with no shared pool or new
SDK dependency. CallerRuntime and std-only Core do not gain threads or runtimes.
The retained observer performs the original Join; no detached handle or
replacement completion signal substitutes for it.

The caller must keep the owner outside the cancelled scope and keep its runtime
alive through cleanup. Pending/deadlines preserve original obligations.
Dropping the retained owner still performs ordinary Rust destruction and is
not evidence of graceful completion.

### 19.4 Focused verification and delivery

The public proof lives in the
[Web integration tests](../crates/yetlet/tests/web_hosts.rs) and the
[isolated API-only consumer](../examples/external-consumers/web-hosts/README.md).
They target actual multi-Host HTTP progress, cancellation, drain, original
thread completion and typed passive/managed dependencies. The SQLite proof is
application-owned and adds no mandatory framework database dependency.

The final focused source passes 21 Composition and 19 HostGroup tests, twelve
internal Web tests, ten public managed-Web cases and two real SQLite cases.
The managed tests exercise current-thread serving, resolved App state and a
native response layer, probe cancellation with exact method/query admission,
occupied-bind rollback, explicit owned-thread async Join cancellation, blocking
driver compatibility, and retained local resources through final release.
A controlled owned-thread failure proves that a closed status flag cannot mask
the original completion cause. The post-review regressions retain an observer
and the original error after service-thread spawn failure, hold real handler TLS
behind a gate released by current-thread Tokio after drive cancellation, and
verify borrowed resource release after that original TLS completion. A late
matching probe response stays TimedOut after the same owner resumes, with closed
admission and the same cause through terminal Join. A supplemental source check
also found arbitrary panic-payload destruction during terminal observation.
The repaired error source retains that payload; a real owned-thread panic test
checks formatting, driver-owner destruction and error clones before the final
failure-owner Drop releases it.

Seven direct App/configuration tests and nine existing advanced HTTP-worker
tests pass with the shared serving implementation. The finite external
consumer serves the real default 127.0.0.1:7878 and a distinct OS-selected
ephemeral address, then completes both original services. Its normal/build
graph excludes View/HTML/actions, Browser/staging Hosts, SQLite/tempfile and
Tokio's multi-thread runtime feature. The existing five-Host Core consumer
still runs with only itself and yetlet in its normal graph.

Strict Core/Web/consumer Clippy, all 16 changed Rust files' formatting,
13 positive/compile-fail doctests and strict Web rustdoc pass. Web + HTML
compiles with all targets; default Core compiles for wasm32-unknown-unknown.
Wasm is compile evidence, not a browser or synchronous-thread execution claim.

The initial public probe-failure assertion expected a successful Join. It now
asserts the actual failed terminal service after drain, primary Readiness and
secondary Join sharing the same original cause, and a released listener.
The initial manifest-target timing failure and strict collapsible-if finding
are also retained with their repaired checks. Offline consumer locking initially
selected newer cached transitives; Cargo then aligned only the new lock to
existing workspace version/source/checksum identities. No existing lock or SDK
dependency version is upgraded.

Independent R1 `wc_job_nWr9zedTb14s51sz` completed exit0 with actual FAIL on
the TLS Join and resumed probe deadline findings. Both findings are repaired;
`REPAIR-R2.json` retains that successful focused repair verification.
Independent R2 `wc_job_x1vYy0NDNVKOJPUa` completed exit0 with actual FAIL on
termination between readiness and aggregate admission while original Join
remains pending. The advanced admission outcome and closure notification repair
that interval without replacing the original cause.

`ADMISSION-R1.json` records 13 successful groups for the admission repair's Core/Web/direct/
SQLite tests, strict Core/Web/consumer Clippy, all changed Rust formatting,
the five-Host consumer, Core Wasm, doctests, rustdoc and Web + HTML compilation.
A three-Host Core regression stops later admission after Closed and retains an
original non-Clone failure and every dependency. A two-Web regression holds the
first service's real TLS destruction while the second becomes ready, cancels
and resumes the same drive, and verifies closed gates, original error identity
and resource release only after original Join. The direct closure-after-poll
case also covers the atomic admission boundary. Earlier unchanged worker,
finite Web consumer and feature-graph evidence retain their separate receipts.

Independent R3 `wc_job_gBxAmJtbWljS1V4O` completed exit0 with actual FAIL on
exit notification occurring after owned-runtime destruction. The existing guard
now notifies before teardown can wait for blocking work, while retaining early
notification if runtime construction fails. `WAKE-R1.json` reruns Web/internal/
public/direct and SQLite tests, strict Web Clippy and all 16 changed Rust files'
formatting. Its regression drives the group in an independent local task and
holds real spawn_blocking work: the sibling must be fenced before the blocker
is released, while original Join and both dependencies remain retained. The
controller's timeout cannot repoll the drive and mask a missing wake. Original
error identity and final release are verified after unblocking and joining.
Only the private Web driver and its regression changed after ADMISSION-R1;
unchanged Core and other quality evidence retain their earlier receipts.

Receipts are retained in `.pipeline-output/m28-yt291-20261122/`.
Scoped documentation links/anchors, English, backlog DAG and whitespace checks
pass; the 39 historical global-doc diagnostics remain preserved with unchanged
source identities. Configured independent R4 `wc_job_Ju-yzZ1wX5TXj91c` completed
exit0 with actual PASS, issues=[] and required_followups=[]. Strict/Builtin
staged Guard remains the final ordinary local commit gate. No push, publication,
browser/native-window execution or YT-292 integration is included.

## 20. YT-292 — optional Native, UDP and custom Host integration

**Status:** Done after implementation, focused verification and configured
independent Reviewer approval; the ordinary staged local commit gate follows.

### Optional carriers and ordinary entry points

All four native capabilities now have canonical implementations inside yetlet,
behind explicit default-off features. The SDK default remains [] and introduces
no new published package.

| Feature | Canonical module | Dependency and execution boundary |
|---|---|---|
| command-bridge | yetlet::command_bridge | Std-only bounded command admission and immutable snapshots; independent of HTTP, ActionExecutor and application state authority. |
| udp | yetlet::udp | Std nonblocking UdpEndpoint plus managed UdpHost; bounded caller-driven packet polling, no Tokio, Web, Winit or renderer dependency. |
| native-winit | yetlet::native_winit | One main/entry-thread NativeHost driver and multiple windows; Winit only, no implicit renderer, Web runtime or extra thread. |
| renderer-wgpu | yetlet::renderer_wgpu | The existing Vulkan Surface renderer, independently selected; accepts caller-provided handles without a Winit, View or Web dependency. |

The unpublished yetlet-hosts companion contains forwarding modules while the
remaining historical consumers migrate in YT-293. It no longer owns duplicated
bridge, UDP, Window or renderer implementations. Its direct optional Winit/wgpu
dependencies remain only for old example imports; yetlet never depends on this
companion. Native modules and native dependencies are absent on wasm32.

The [independent consumer](../examples/external-consumers/native-hosts/README.md)
uses ordinary Host descriptions, one retained HostGroup and an optional Web
feature. Its minimal selection is native-winit plus udp. A mixed plan follows
this shape; each constructor is inert:

~~~rust
use yetlet::{HostGroup, native_winit::{NativeHost, winit}, udp::UdpHost};
use yetlet::web::App;

let native = NativeHost::new(winit::window::Window::default_attributes())
    .with_window(winit::window::Window::default_attributes());
let network = UdpHost::new().with_address("127.0.0.1:0".parse()?);
let web = App::new().get("/api", || async { "ready" })
    .into_host().with_address("127.0.0.1:0");
let mut hosts = HostGroup::new()
    .with_host(native).with_host(network).with_host(web).build()?;
hosts.run().map_err(|error| std::io::Error::other(error.to_string()))?;
~~~

Native and UDP infer the group's error type, including WebError in that mixed
plan, through their adapter-local From<io::Error> requirement. They impose no
Clone, Send, Sync or 'static requirement on application callbacks/errors.
Native/UDP-only code selects HostGroup::<std::io::Error>::new() or annotates the
standalone Host error parameter. Web itself remains optional.

### One retained lifecycle and one root native driver

Advanced ResourceGroup::with_blocking_driver stores a concrete BlockingDriver
through typed Driven metadata. Passive resources, managed resources and
map_activity preserve it; NoDriver keeps ordinary Core groups' existing auto
traits. Building rejects multiple selected root drivers before acquisition.
Known duplicate native-loop and nonzero UDP endpoint claims also reject inert
plans; external ownership and OS conflicts still require actual startup checks.

BlockingDriver receives the same ShutdownHandle and lifecycle polling callback.
The whole plan prepares for Blocking before driver entry. Native callbacks
advance the existing Core engine through startup, admission, original completion
and final release. A stop request can interrupt a later pending managed
acquisition without waiting for NativeActivity to start.

NativeHost::with_window adds a window to the same loop. Actual window creation
happens inside Winit's supported callbacks. Every configured window must
observe its own visible, nonzero, unoccluded redraw before aggregate readiness.
After admission, on_event receives original window events and bounded Tick
opportunities on the same caller thread. This proves Window readiness, not a
GPU present, engine frame rate or View renderer. Core retains that startup
observation while sibling Hosts start. Later resize/occlusion may make
NativeHandle::is_ready false temporarily, but only actual stop/release closes
admission permanently; temporary unusability does not shut down the group.

Windows are the first managed Native dependency. Later custom renderer
resources therefore release before windows, while the root driver remains
active through all finalizers. Closing/destroying any managed window, or
suspension, requests coordinated whole-group shutdown. This adapter does not
restart after suspend or create a replacement Winit loop. Original builder,
Window and loop errors remain available through the chosen application error.

A driver error after the last cleanup callback is still a Drive failure and
changes the final phase to Failed. A driver return while obligations remain
instead produces DriverReturnedEarly, retains the concrete driver and original
owners/errors, and requests deferred stop without invoking affine cleanup from
a fallback context. Generic resumable drivers can resume the same instance;
Winit run_app consumes its loop and cannot promise that recovery. A builder
failure before any progress can retire only the still-inert plan on the same
caller thread. No async cleanup thread or silent loop recreation is introduced.
While a selected Blocking driver still owns unfinished obligations, public
poll_start, poll_run, run_async and shutdown polling cannot advance callbacks
outside it. They retain Pending; shutdown requests only defer stop. Only run
through the same retained driver can resume progress. NoDriver polling and
terminal observation retain their ordinary behavior.

### Caller-driven UDP and optional command bridge

UdpHost defaults to loopback port 0, exposes the actual bound address through
a weak metadata handle and opens admission with the same aggregate barrier.
on_datagram borrows the received bytes/socket only for its callback; reply and
send_to retain explicit bounded outcomes. Packet count and datagram size are
finite. Closing admission is permanent; original callback/I/O errors remain
owned until terminal Join and dependency release. Exact bind failure has no
fallback address.

NativeHost and managed UdpHost reject Async preparation before any acquisition.
The std UDP socket has no registered async reactor; ordinary synchronous run
or the selected native callbacks provide bounded progress. Advanced callers may
still drive UdpEndpoint explicitly in their own event environment.

The optional command bridge preserves its existing bounded queue, immutable
snapshot and admission semantics. Enqueuing a command is not execution success.
It neither installs a universal domain state service nor requires ActionExecutor.
Renderer production behavior and shaders are relocated unchanged; package/API
availability and renderer backend qualification remain separate claims.

### Verification and delivery boundary

Focused source verification passes: 48 Core lifecycle tests (21 existing
coordinator and 27 Host-group tests), 16 UDP tests, 14 Native tests, four pure
bridge tests, 24 synthetic renderer tests, 22 managed Web regressions and seven
remaining companion HTTP tests. The 17 positive/compile-fail doctests, strict
Clippy, strict rustdoc, both native consumer feature variants, companion example
compilation and all 24 changed Rust files' formatting pass. Formatting-only
changes do not require replaying the native execution.

Nine independent feature selections each pass a normal/build graph and library
compilation check; two intended disabled-import failures verify export absence.
Default Core, bridge-only and UDP-only native graphs contain only the consumer
and yetlet. Native-only selects Winit without renderer/Web/Tokio; renderer-only
selects wgpu without Winit/Web/Tokio. Combined selections do not add View, HTML,
Actions, Reactive or auxiliary services. Wasm default and all-four-native-feature
graphs contain only Core, with native exports unavailable. These are compilation
and dependency checks, not additional OS/browser/backend execution claims.

One bounded Ubuntu X11 combined consumer run, PID 948749, exits 0. Two actual
windows (60817410 and 60817411) deliver post-admission redraws on ThreadId(1).
The external client receives a 17-byte UDP echo and HTTP 200. The custom borrowed
resource returns Pending once and observes both windows live on both release
passes. Web, Native, UDP and local Hosts all report successful original Join and
final release. The same receipt confirms both window IDs absent afterward and
the process gone; no second UI launch is used as redundant proof. This run
explicitly has gpu_present=false. The later R1 corrections below are verified
by deterministic regressions and compilation; this native run is not replayed.

The initial renderer test-path and companion SyncSender import omissions were
relocation compile defects; focused repairs and checks pass. Consumer-only
Clippy findings and a Native-test formatting failure are likewise preserved
with their corrected checks. Supplemental review also repaired an actual
Windows oversized-datagram path: WSAEMSGSIZE is counted as a consumed/rejected
packet within the same budget. The portable incoming-size regression passes on
Linux; Windows runtime behavior is not claimed. Native builder panic diagnostics
now cache display text and expose nonblocking try_with_payload inspection, with
a no-OS regression for nested inspection/formatting and retained payload Drop.

Cargo generated the new consumer lock and aligned it to existing workspace
version/source/checksum identities. Temporary resolution constraints were removed
before consumer validation. Existing dependency versions were not upgraded.
The feature probe's 225 registry identities also match the baseline and all
31 existing locks remain unchanged by that probe. Original resolution conflicts
and the pre-Cargo PATH setup failure remain recorded with their successful
bounded corrections.

Configured independent R1 wc_job_2cg2c1vQJjQxDYFR returned three material
findings: public polling could bypass an early-returned driver, transient
Native usability loss could masquerade as permanent closure, and the existing
five-Host Core consumer's generic helpers omitted the new driver parameter.
All three are repaired. Regressions exercise every public progression route,
a pending affine finalizer after activity release, and observed Native
readiness followed by usability loss while a sibling starts. The existing
Core consumer now carries H::Drivers and a generic driver parameter; its
ordinary finite execution and strict Clippy pass without changing its lock.
The 12-step focused correction check passes, including immediate Web/UDP
regressions, both Native consumer variants, doctests, strict rustdoc and Wasm
Core. The current total is 135 tests plus 17 doctests; unaffected renderer,
bridge and companion evidence is retained rather than redundantly replayed.

Receipts are retained in .pipeline-output/m28-yt292-20261122/: TEST-R1/R2,
REPAIR-R3, CONSUMER-R4, COMPANION-R3, FEATURE-PROBE-R1, FORMAT-FINAL-R4,
REVIEW-FIX-R1, TEST-COUNTS-R3 and NATIVE-R1 identify their separate scopes. The scoped documentation check
preserves the historical global diagnostics without replaying them.
Configured independent R2 wc_job_JOQWTmAmTSZNi--j reached terminal exit 0
with actual PASS, issues=[] and required_followups=[]. The configured model
and resolver-selected effort were observed, and all reviewed source/receipt
hashes remained unchanged. Only completion annotations follow that review;
Strict/Builtin staged Guard remains the final ordinary local commit gate.
General CLI/consumer migration remains YT-293; normal
archives and selected external package qualification remain YT-294. Published
alpha.8 and the pinned reader retain their original contracts; no publication
or physical LAN qualification is included.

## 21. YT-293 — CLI, consumers and minimal SDK feature selection

Status: Done after focused verification and configured independent R2 actual
PASS. R1 findings and correction evidence remain retained below.
Date: 2026-11-22.
Baseline: 76dfe9099c307705e48c97edad9520b9d13da090 (completed YT-292).

### Scope and packaging

Keep exactly yetlet and yetlet-browser as the SDK publication cohort. The CLI
remains independently packaged. This slice selects no release or version bump:
published SDK alpha.8, CLI 0.4.1, SDK identity, checksum-bound historical startup
capabilities and the pinned reader edition retain their existing authority.

The final source consumers select canonical yetlet features directly. Three
low-level Native/renderer/combined examples move into yetlet/examples, and seven
HTTP/command-bridge integration tests move into yetlet/tests. Their target gates
are explicit. The temporary yetlet-hosts forwarding crate is removed, including
its workspace membership. The revision-audit and public-compat consumers use
canonical imports; render-bench declares its direct View dependency.

Normal package file lists include the moved examples/tests, optional source and
renderer shaders. Disabled capabilities may be present in the archive without
being enabled in its dependency graph. Core default=[] has no normal/build
runtime dependencies; Native+UDP does not select Web, Tokio or the renderer.
Exact selected archives and their external consumers remain YT-294.

### Selected SDK and generated source

The current CLI derives an internal API profile from the selected SDK's feature
metadata. Declaring web or view selects the feature-module profile before its
requirements are checked; incomplete new metadata cannot fall back to legacy
server-axum. Native SSR requires web+html and optional configuration requires
config. Browser requires explicit view+reactive, plus the browser package's
DOM/keyed-DOM capabilities. New generated core dependencies disable defaults.
The CLI's Native template continues to mean native HTTP/SSR, not a Winit Host.

Legacy metadata retains the existing templates and historical startup capability
rules. New templates use canonical yetlet::web paths and truthful web-app-serve,
web-configured or web-service startup receipt labels; Browser keeps browser-wasm.
Profile finalization occurs after Style, Tailwind and Vite composition, so these
choices share the same SDK/import decision. Selection errors keep the selected
SDK identity and actionable upgrade/pin guidance without silently choosing an
older SDK. The release consumer verifier independently checks metadata, manifest
feature opt-ins and generated startup source; no new release schema is needed.

### Verification checkpoint

The preserved baseline CLI executable and all 163 tracked CLI source hashes
identify the old installed-contract input. Its SHA-256 is
d6eb5a8eb498449a505bab555d838b40016092c716055a7d33ef28fb63abf6c1;
the migration capture binary, before the README-only review correction, is
dba99db1ddcd39f76891aa824f6a505a2d43cf423fb5e87994124c8c250bd9c5.
Actual old Native selection rejects the new map with SDK_INCOMPATIBLE before
locking. Actual old Browser selection emits its template; that exact Rust/assets
compile for Wasm against current source because DOM/keyed-DOM forwards View.
Actual current Native and Browser command outputs also compile.

These executable checks use source-derived metadata labeled synthetic 0.10.0.
A bounded recorder captures generated files and deliberately fails the lock step;
no successful creation receipt or registry lock is fabricated. Copies compile with
only the SDK dependency source fields changed to current paths. This is source
compatibility evidence, not normal archive or publication qualification. The old
Native diagnostic lacks upgrade wording. The current guides give an explicit
alpha.8 pin and require a separately qualified compatible CLI for next-SDK
publication. No release/version is selected.

All 32 focused CLI tests pass, including 48 composed template shapes; the CLI
package marker regression and 18 independent release-consumer verifier tests pass.
Nine representative generated Native/Style/Service/Settings/Tailwind/Vite/Browser
projects compile, followed by the three actual command captures above. The
existing two ignored registry-environment tests are not reported as executed.
All generated Rust and assets remain unchanged by source-dependency adaptation.

Current-source consumer checks pass: three moved examples compile, all seven
HTTP tests pass, the revision consumer passes 68 library plus 14 binary tests,
and the compatibility consumer passes three tests. Render-bench compiles. Root
and the two changed standalone locks reconcile without new external package
identities. Ordinary package --list and metadata checks confirm the retained
cohort and target gates; they do not claim built or qualified archives.

Full CLI all-target Clippy and moved SDK-target Clippy pass with warnings denied;
the initial 15 changed Rust files match rustfmt. The existing package marker test keeps
its intentional byte-array fixtures with a narrow explained lint expectation;
its actual case-insensitive marker behavior remains checked. Release-doc checks
pass after restoring current README/guide version shapes, eight path-consumer
version fields and a package-safe Web-guide URL. Historical authority files,
reader edition and both registry-pinned reader manifests remain unchanged.

Initial failures remain retained: the source-capture tool preflight rejected
existing writable tool directories, a retained generation parent was not private,
and release-doc normalization found stale tracked deletion/index and document
shapes. Only task-owned fixture paths were made private. Exact copied tools and
libraries plus a bounded ELF recorder completed the actual executable check;
no product trust rule or existing installation permission was changed.

A supplemental generated-lock assertion initially used an overly narrow workspace
identity set. All nine compiler commands already passed. Configured templates'
existing optional TLS dependencies produce first-time fixture locks; those are
recorded separately. Comparison of all 31 tracked Cargo.lock files confirms only
the three intended lock changes and no added external package identity. No
compiler invocation was repeated for this assessment correction.

Current documentation explains Core, optional View/HTML/Web and Host lifecycle,
while retaining alpha.8 registry recipes and dated execution records. Source
links follow moved files. Evidence is retained in
.pipeline-output/m28-yt293-20261122/: CONSUMERS-R1, CLI-R1/R2,
GENERATED-R1/R2/R3, COMMAND-SOURCE-R1, METADATA-R1 through R6, QUALITY-R1/R2
and old-cli-capture-r2 identify their distinct results and limits. No new UI/GPU
execution or publication is part of this migration.

### Independent review corrections

Configured independent R1 wc_job_OWehrdzPxuUGPYWb completed with exit 0 and
an actual FAIL verdict containing four material P2 findings. Reviewed source and
evidence hashes remained unchanged throughout that review. All four findings
were admitted against the existing migration contract and repaired:

- Feature-module manifest validation now checks explicit required opt-ins and
  disabled defaults for both yetlet and yetlet-browser. Companion-only negative
  cases remove dom-keyed or enable defaults while leaving the core valid.
- Generated Native README guidance describes the retained legacy-only Docker,
  Compose and Kubernetes exporter boundary, including Service. Its historical
  template/resource bytes and Legacy Service instructions remain unchanged.
- The existing cloud scaffold test uses an explicit legacy fixture for historical
  byte assertions. A current-checkout case checks ordinary and Service canonical
  imports, web/html opt-ins, disabled defaults and existing no-replace behavior.
- The current samples command selects web,html with the retained
  server-axum-minimal target; published alpha.8 guidance is separately identified.

Correction verification passes: 18 release-consumer tests, 10 SDK/template tests
and both ordinary cloud-suite tests. The existing generated-runtime cloud test
remains ignored. Full CLI all-target Clippy passes with warnings denied, and
all three Rust correction files match rustfmt. REVIEW-FIX-QUALITY-R1 retains
successful tests/Clippy and two formatting mismatches; R2 closes four test-only
line-wrapping hunks without repeating those successful commands.

All 48 composed profiles were generated again for the changed README behavior.
Of 1128 files, only 32 Native READMEs changed; the remaining 1096 files, including
all Rust, manifests and assets, are byte-identical. The earlier twelve source
compiles and executable captures therefore remain applicable to their stated
source scope without replay. REVIEW-FIX-PYTHON-R1, REVIEW-FIX-CLI-R1,
REVIEW-FIX-QUALITY-R1/R2 and REVIEW-FIX-GENERATED-DELTA-R1 retain these checks.
Configured independent R2 wc_job_DjxGN41kOxQ3Y-SU completed with terminal
exit 0 and actual PASS, issues=[] and required_followups=[]. The saved Reviewer
model and observed effort were gpt-6.2-astra/high under the unchanged operator
launcher. All 61 current source hashes, ten absent paths and 164 evidence inputs
remained unchanged across review. This is ordinary role-directed independent
review, not native agent_type dispatch or formal assurance. YT-293 is Done.
Final local Git delivery uses the Strict/Builtin staged Guard; its final tree and
commit receipts are retained with the local handoff. Normal archives and selected
external-consumer qualification remain YT-294; no release or publication is selected.

## 22. YT-294 — Normal archives and external-consumer qualification

Status: Done after normal archive/consumer verification and configured independent
R1 actual PASS; YT-286 source/package delivery is complete.
Date: 2026-11-22.
Baseline: 19993131f02fa4477296542f6bbe0b1774a744b7 (completed YT-293).

### Exact package inputs and dependency order

This qualification keeps the two-package SDK cohort, empty Core defaults and
the current development version fields. It selects no new release identity.
Both normal archives are built from the clean baseline above; subsequent changes
in this item are repository documentation outside their package inputs.

The existing release-doc sync plan required zero changes, and its check passed.
Cargo vendors the existing locked external dependency set. Normal offline/locked
Core packaging completes Packaging, Verifying, Compiling and Finished. That final
archive is then extracted unchanged into the invocation-local directory source,
with Cargo's package checksum bound to the archive SHA-256. Normal Browser
packaging verifies against that exact Core identity. There is no bootstrap package,
path patch, reverse development edge, archive rewrite or skipped verification.

| Normal archive | Bytes | SHA-256 |
|---|---:|---|
| yetlet-0.9.0-alpha.8.crate | 323654 | ea267e20bd424aa1c7772140d2269655575f3b0743ac887fa49eabba4c60412b |
| yetlet-browser-0.9.0-alpha.8.crate | 53669 | bd7921536aba61d937337be7223ee883b207af59eff0e10c60faa983b674a1eb |

Both archived VCS records name the baseline and report a clean source tree.
Browser's archived Cargo.lock binds Core to the checksum above. Normalized
manifests have no path/git dependency overrides. Package inventories retain
the moved examples/tests, their feature gates and both required WGSL assets.
These local development archives differ from published alpha.8 bytes; their
shared development version is not a new registry publication identity.

### Independent consumers and feature graphs

Six standalone fixture families request exact SDK versions with defaults disabled.
Their Rust comes from the existing accepted consumers/tests without changes.
Only fixture dependency sources, required target declarations and first-time
fixture locks are adapted. Cargo resolves the selected normal archives through
an invocation-local directory source; no fixture uses an SDK path/git dependency
or patch, and no fixture configuration changes the repository's Cargo setup.

| Family | Actual archive-backed verification |
|---|---|
| Core | Finite five-Host execution with caller-local/borrowed resources, inert build, Pending wake and ordered Join/release; native and Wasm normal graphs contain only the consumer and Core |
| View and HTML | Separate View-only and HTML-only finite executions; HTML Wasm compile; explicit feature graphs retain no unrelated normal/build dependencies |
| Browser | Real Wasm compilation for no-default, DOM/keyed-DOM, GPU-only, GPU-backend and actions selections; actions selects actions-http without native Web, and only gpu-backend selects wgpu |
| Pure Web | Finite two-Web-Host execution and two existing SQLite cancellation/drain/original-owner/release tests; normal web graph excludes HTML/View and SQLite development dependencies |
| Native and UDP | Native-only consumer compilation, four headless Native lifecycle tests and seven real-loopback UDP tests; no Web/Tokio/renderer in the selected normal graph |
| Combined and custom Hosts | Native+UDP+Web+caller-local custom-Host consumer compiles; selected graph adds Web/Tokio explicitly, while the Wasm native-feature guard excludes platform-only dependencies |

All 16 run/check/test actions and 15 unique normal/build graph observations pass.
The actual test total is 13 passed, zero failed and zero ignored. Native fixture
locks add no external package identities relative to their accepted seed inputs.
Older View/Browser-action seed locks reconcile to the selected baseline vendor
versions; these are first-time qualification fixture locks, not repository
dependency changes. All copied Rust and prepared manifests retain their hashes.

The initial matrix Job wc_job_mIgK1Gt7kbK0jQBu terminated with exit 1 after
13 successful Cargo actions: a supplemental graph assessment keyed packages only
by name and rejected coexisting rustix versions. Its output remains retained.
The correction keys package identity by name and version, reuses the original
successful command/graph outputs, and completes only the three remaining actions.
Continuation wc_job_TzeymSj71koIgweG terminates with exit 0. This assessment
repair changes no product code and repeats no successful compiler command.

### Native runtime evidence and its limits

The selected archive's 23 relevant Core/Web/Native/UDP runtime files match the
final accepted YT-292 source and current baseline. The Native consumer Rust is
unchanged, and its earlier manifest adaptation only adds the SDK version field.
The current archive compiles and headless tests complement the retained YT-292
bounded X11 run: one application, two windows, loopback UDP, Web and a caller-local
Host, followed by observed final completion and cleanup.

That earlier executable predates YT-292's Reviewer repairs. Its runtime evidence
is carried together with the 12 accepted correction checks and independent R2
PASS; it is not an execution of the current archive binary. This item performs
no new visible Native run. Browser/Wasm and GPU-backend compilation do not claim
browser execution or GPU runtime. Windows, Wayland, physical LAN and broad
platform qualification remain outside these results.

### Version and CLI compatibility disposition

The current source CLI is built once and invoked for Native and Browser against
private metadata projected from these exact archive manifests and checksums.
Both commands reject with SDK_INCOMPATIBLE before lock generation, destination
publication or a successful creation receipt. Their only Cargo invocation is
the tool-version probe. The selected alpha.8 Core checksum contradicts the
published startup capability identity; rejection preserves the existing rule.

YT-293's synthetic source captures and old/current CLI behavior remain separately
identified. The original compatibility ledger permits unsupported combinations
with explicit upgrade/pin guidance and a publication disposition. The additional
YT-293 wording that put successful registry creation into YT-294 is corrected:
these local normal archives are qualified directly, while a successful registry
creation remains a gate for a separately selected unique SDK release and
compatible CLI before publication. No historical checksum, version, startup
capability, release record or pinned reader authority is rewritten to manufacture
a successful scaffold result.

Existing installed users retain the published alpha.8 pin documented in the
[compatibility guide](COMPATIBILITY.md#current-checkout--cli-and-selected-sdk).
A future publication must select the new SDK identity, qualify its matching CLI
and generated registry consumers, and then follow the existing release and reader
promotion gates. Source/package delivery does not authorize those operations.

### Evidence, independent review and local delivery

The paired run m28-yt294-20261122 uses existing cache/evidence layout rules.
Normal archives, immutable consumer inputs and command logs are retained under
artifacts/verification/m28-yt294-20261122/. Receipts under
.pipeline-output/m28-yt294-20261122/ include SOURCE-BASELINE,
PACKAGE-INPUTS-R1, NORMAL-ARCHIVES-R1, CONSUMER-PREPARATION, CONSUMER-LOCKS-R1,
CONSUMERS-R1/R2, CLI-ARCHIVE-IDENTITY-R1 and RUNTIME-CARRYFORWARD-SOURCE-R1.
The failed graph assessment and its passing continuation remain distinct.

Configured independent R1 wc_job_UjwLgXAwgKl0hNz1 reached terminal exit 0
with actual PASS, issues=[] and required_followups=[]. The saved Reviewer model
and observed effort were gpt-6.2-astra/high under the unchanged operator launcher.
All 21 selected source hashes and 109 evidence inputs remained unchanged during
review. This is ordinary role-directed independent review, not native agent_type
dispatch or formal assurance.

Release-document and scoped English/link/anchor/backlog/whitespace checks pass.
The 39 historical global-document diagnostics remain retained without replay;
this is not a new globally clean documentation claim. YT-294 is Done and closes
the dependent YT-286 umbrella at source/package delivery scope. Final ordinary
local Git delivery uses the Strict/Builtin staged Guard; exact tree/commit
receipts are retained with the local handoff. Publication, live reader promotion
and physical YT-285/YT-274 remain separate.
