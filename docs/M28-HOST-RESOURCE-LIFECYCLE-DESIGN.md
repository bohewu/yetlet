# M28 — Layered Hosts and ResourceGroup Lifecycle Design

**Status:** Reviewed A/B source proofs retained; owner-selected Core/View/Web replan, with detailed public contracts pending YT-287.
**Date:** 2026-11-21.
**Tracking:** YT-286 public-delivery umbrella; ordered remaining items YT-287–294.
**Decision:** [ADR 0014](adr/0014-layered-host-resource-lifecycle.md).
**Source baseline:** `dec423a5ef6f7100e5f31cb93d0cbbec84652d71`.

The [current delivery sequence](#14-replanned-public-delivery--yt-287-through-yt-294)
supersedes the earlier additive-only packaging and Application naming proposal.
Sections 1–13 retain the original research and private-proof contracts; their
historical authorization statements do not override the owner's later request.

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
| [Native adapter](../crates/yetlet-hosts/src/native_winit.rs) | `WindowSlot::new` prepares attributes; the caller supplies the active Winit loop when creating a window. The slot is caller-local. | Window preparation, actual native acquisition and event-loop ownership are separate. |
| [Web adapter](../crates/yetlet-hosts/src/web_axum/mod.rs) | `HttpWorker::start_on` binds a listener and spawns one owned OS thread; that thread drives a Tokio current-thread runtime. Original thread completion is observable. | The current Web adapter has a concrete topology, although Composition itself has no scheduler. There is no shared framework thread pool. |
| [UDP adapter](../crates/yetlet-hosts/src/udp.rs) | `UdpEndpoint` owns a nonblocking socket and exposes bounded caller-driven polling. It creates no worker thread. | Several Hosts can share a driver without each needing a thread. |
| [Three-way consumer](../crates/yetlet-hosts/examples/composition_three_way.rs) | App-local Native/Network/Web values compose a caller-driven native/domain/UDP path with the owned HTTP service. The caller explicitly drives readiness and shutdown. | A high-level layer should package this coordination without making its particular topology universal. |

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
| YT-288 | Built-in Core composition and generic Host defaults | Make composition available without a feature opt-in; promote the bounded generic HostGroup/ResourceGroup surface through one engine after compiling driver/extension contracts. Preserve local and borrowed ownership, original completion owners, global admission/join/final-release barrier and retained Pending. Core-only consumer has no View/HTTP/Tokio/Winit/wgpu or mandatory serde. No blind promotion of the private io::Error, sleep loop or four-Host proof limit. | YT-287 |
| YT-289 | Optional View/HTML layer and Browser DOM migration | Isolate View, Element, tags, binding and pure HTML serialization behind the selected yetlet features. Migrate Browser DOM/keyed-DOM feature selection and reexports in the same slice. Core-only omits View; View/HTML adds no HTTP/runtime dependencies and has no internal composition dependency; DOM/Wasm has no native server dependency. Preserve escaping, validation and component-as-function semantics. | YT-287 |
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


