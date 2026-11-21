# ADR 0014 — Layered Hosts with Default Resource Lifecycle Management

- **Status:** Owner-selected target direction; YT-287–290 independently reviewed and Done; remaining capabilities pending.
- **Date:** 2026-11-21.
- **Tracking:** YT-286 umbrella remains Todo; YT-287–294 define the remaining delivery gates.
- **Decision owner:** Project maintainer.
- **Related:** [Research and API design](../M28-HOST-RESOURCE-LIFECYCLE-DESIGN.md), [ADR 0013](0013-composable-application-and-hosts.md), [ADR 0011](0011-application-startup-api.md), [naming guide](../NAMING.md).

## Current owner direction and ordering

After A/B3, the owner selected breaking next-alpha layering: Core includes
composition/lifecycle, with optional View, Web and concrete Host capabilities.
The latest clarification prioritizes minimal published SDK packages and feature
switches. Keep the existing yetlet and yetlet-browser cohort as the starting point;
logical layers use modules/features rather than new published packages by default. The earlier
Application name and additive-only packaging are superseded proposals, not
constraints on the new work. Existing private proof names remain historical.

The [replanned sequence](../M28-HOST-RESOURCE-LIFECYCLE-DESIGN.md#14-replanned-public-delivery--yt-287-through-yt-294)
is reviewed and locally committed before YT-287 executes. YT-287 resolves the
module/feature/import, App/HostGroup, command/state/wire and driver contracts in this
ADR and the existing design. Public implementation and release claims wait for
their own later gates; the recorded alpha.8 contracts are not rewritten.

## YT-287 target decision — minimal packages, optional capabilities

This section records the next-alpha target and supersedes conflicting
Application naming, additive-only packaging, opt-in-Core and private Web
default assumptions below. Historical A/B names and receipts remain unchanged.
The [detailed contract and source inventory](../M28-HOST-RESOURCE-LIFECYCLE-DESIGN.md#15-yt-287--feature-first-architecture-and-driver-contract)
define the feature/import, driver and migration acceptance for YT-288–294.

1. Keep the existing yetlet + yetlet-browser SDK cohort. Use modules and features
   rather than new published Core/View/Web/Hosts packages. The unpublished
   yetlet-hosts staging code migrates into optional yetlet modules.
2. Select next-alpha yetlet default=[] with built-in std-only composition,
   generic HostGroup/Host and ResourceGroup. There is no default runtime,
   serialization, View, global state owner, command bus, pool or process policy.
3. View is an optional HTML/DOM value layer; html enables view. Native web can
   serve an API without HTML, while web + html enables SSR/Page helpers.
   Browser DOM selects view; Browser actions select a target-neutral actions-http
   feature without native Web. Keep independent native/network/renderer and
   existing auxiliary native features.
4. Name the generic lifetime root HostGroup; keep App for HTTP service
   configuration in yetlet::web. Core composition is the one lifecycle engine
   for built-in/custom Host defaults and advanced ResourceGroup management.
   Construction/build are inert; original owner, admission, failure, join and
   final-release rules remain explicit.
5. Keep actions optional and make its executor error associated with the
   implementation. HTTP errors/envelopes belong to actions_http; moving serde
   alone is insufficient. Reactive is a local graph, and the optional native
   command/snapshot bridge is separate from ActionExecutor.
6. Direct App serving uses caller-owned Tokio. Managed async Web uses a yielding
   caller-runtime driver; synchronous composition can select the documented
   owned-thread Web driver. Native composition uses one root-owned event loop
   with multiple windows. Features expose these APIs but cannot select or prove
   valid execution topology.
7. Preserve 127.0.0.1:7878 for App and select it for the new public WebHost;
   multiple Web Hosts use explicit distinct addresses or port 0 and expose
   their actual addresses. Default readiness observes listener plus functioning
   serving-driver initialization, without requiring GET / to return 200.
   Optional HTTP probes retain their explicit startup admission contract.
8. Preserve the published alpha.8/CLI identities. Migrate active imports,
   feature/default assumptions, Browser reexports, generated templates and
   archive consumers coherently. No permanent old-name aliases or new package
   cohort schema are selected merely for this restructuring.

### Cancellation is an ownership contract

App::serve(self).await remains the ordinary transport convenience. Dropping
its future is not guaranteed graceful drain or framework-managed resource
finalization; App::with_state does not register an async finalizer. Do not add
managed attachments to that consuming path without proven owner transfer.

Managed resources use a retained owner with a borrowing drive. Before each
cancellation point, acquired/partial resources, original joins, in-flight cleanup
state and terminal outcomes must be retained in that owner rather than solely
inside a disposable future. The caller must also retain the owner outside the
cancelled task/scope. Cancelling only the drive permits later progress with the
same owner; dropping both does not constitute completed shutdown.

One lifecycle foundation does not imply that every drive is cancellation-safe,
or that cancelling a future has completed its obligations. YT-291 must prove
these suspension-point invariants, current-thread runtime progress, rollback,
admitted drain and final dependency release. No blocking sleep/probe/join in an
async driver, replacement detached cleanup task, or automatic runtime shutdown
can substitute for that proof.

### Decision scope and alternatives

The owner selected the composable framework direction and the minimal-package,
feature-first constraint. YT-287 supplies the independently reviewed design
contract (R1 `wc_job_GwI4V4CCnZesLj9w`, actual PASS with no issues/followups);
later compiling proofs own exact generic signatures,
extension traits and runtime implementation. Architecture acceptance does not
claim that new features exist or qualify a registry release.

| Alternative | Disposition |
|---|---|
| Publish one crate per logical layer | Reject for this migration; it conflicts with the owner's publication-minimization preference without a demonstrated need. |
| Put generic HostGroup in the optional concrete-adapter package | Reject; a custom/local Host should use the Core lifetime foundation without concrete integrations. |
| Make View/State/Actions universal Core services | Reject; each has a narrower optional responsibility. |
| Put both Web modes over one blocking run loop | Reject; it cannot support caller-owned current-thread async progress. |
| Separate lifecycle state machines for standalone and hosted Web | Reject; keep one owner/admission/failure/final-release contract with distinct drivers. |
| Keep all capability modules in yetlet with explicit features | Select; independently verify feature closures and native/Wasm target isolation. |

The remaining exact feature names/dependency edges for existing optional
capabilities, the App/HostGroup sketches, installed old-CLI/native versus
Browser risks and the package proof matrix are in the linked contract.
The earlier proposal and proof records below remain evidence at their original
scope; this section governs the selected next-alpha direction.

## YT-288 implementation checkpoint — built-in Core

The [Core implementation](../M28-HOST-RESOURCE-LIFECYCLE-DESIGN.md#16-yt-288--built-in-core-host-lifetimes) now exposes HostGroup, Host and
ResourceGroup without a composition feature opt-in. A recursive typed Host list
replaces the private tuple limit, with one application error type and no mandatory
Error/Clone/Send/Sync/'static bound. Custom HostActivity/ManagedResource polling
stores partial owners and cleanup state in the retained group.

Context-aware callback adapters use the existing Composition admission,
local-stop, original-Join and reverse-final-release coordinator. Bounded fence
and signal requests receive wake context for retry. Earlier started activities
remain observed while later startup is Pending. Build only validates exact known
exclusive claims; actual platform availability still belongs to acquisition.

BuiltHostGroup offers caller polling and a borrowing, executor-neutral run_async.
Core creates no runtime, thread, timer or sleep driver. Cancelling that borrowing
future preserves the root; dropping the owner does not complete shutdown.
The five-Host independent consumer and targeted Core cancellation/ownership
tests are implementation evidence, not qualification of Web or native drivers.

YT-288 is Done at the independently reviewed Core implementation scope; the
[completion record](../M28-HOST-RESOURCE-LIFECYCLE-DESIGN.md#165-verification-and-delivery-boundary)
retains the R1 finding, repair and R2 PASS. That checkpoint retained HTML
defaults/View exports; the YT-289 transition is recorded below. The existing two-package SDK cohort and all published identities
remain unchanged; no new adapter or release claim is made.

## YT-289 implementation checkpoint — optional View

The [feature implementation](../M28-HOST-RESOURCE-LIFECYCLE-DESIGN.md#17-yt-289--optional-view-and-browser-feature-isolation)
sets yetlet default=[] and gates View, Element, Attribute, tags and binding
behind view. HTML enables view. Browser dom and dom-keyed independently enable
view and gate the corresponding reexports; default-off actions/GPU consumers
remain independent. The existing DOM convenience default is retained.

Only feature declarations, root exports/rustdoc and three necessary View
consumer manifests change. Renderer, binding and reactive behavior remains
unchanged. Separate normal graphs and actual disabled-import checks prevent
workspace/dev feature unification from hiding a missing edge. Wasm evidence is
library compilation, not new browser/GPU runtime qualification.

YT-289 is Done at the independently reviewed feature-isolation scope.
Focused verification and configured independent review `wc_job_THsX4ibX3uyjN_qf` PASS;
issues=[] and required_followups=[]. Published alpha.8/CLI identities remain unchanged.
Web/protocol isolation, driver integration and general migration retain their
later task gates; no additional SDK package or compatibility alias is added.

## YT-290 implementation checkpoint — optional Web and HTTP protocol

The [Web/protocol implementation](../M28-HOST-RESOURCE-LIFECYCLE-DESIGN.md#18-yt-290--optional-web-and-http-protocol-ownership)
now exposes web independently of HTML. App and the relocated advanced
HttpWorker use one private serving function; yetlet has no reverse dependency
on the staging package. Page, document rendering and View-related prelude
entries require web + html. App keeps resolved-state serving, Router/Tower
interoperability and its caller-owned runtime.

Actions now choose an associated executor Error without serde or an HTTP
error dependency. The target-neutral actions-http module preserves the wire
contract and is selected by Browser actions. Todo, Board and custom TCP
consumers move together with explicit domain/transport error boundaries.
HTTP auxiliary features omit HTML while independent narrow features keep
their existing dependency scope. Old public module/feature aliases are absent.

Focused native tests, 14 isolated native/Wasm consumers and six narrow normal
graphs, eight disabled-surface probes, migrated consumers and strict quality
checks pass. The stale-lock failure is repaired without dependency upgrades.
Independent R1 found an active qualification workflow/test still selecting the
removed feature. Both now select web; the existing xtask test and actual workflow
feature compilation pass. Original failed evidence remains; configured independent
R2 `wc_job_3w96W4sl3X2sGtmx` completed with exit0 and actual PASS,
issues=[] and required_followups=[]. Scoped documentation checks pass;
YT-290 is Done at this Web/protocol implementation scope.
This slice does not implement managed WebHost or new readiness/cancellation
semantics: those remain YT-291. Published alpha.8, CLI, reader and later
normal-archive gates retain their existing identities and scopes.

## Context

The owner requests layered exposure: ordinary developers create high-level
Hosts using default resource lifecycle management; advanced developers can
customize that management through ResourceGroup. Names should communicate
intent without first teaching the internal composition mechanism.

The accepted F2 baseline supplies a standard-library-only typed lifecycle core
and opt-in adapters, with caller-owned coordination. Its constructor is
`yetlet::composition::Builder::<E>::new()`. That baseline did not supply the
proposed Application facade, ResourceGroup or final-release phase. A added a
private facade proof; the subsequently authorized B1/B2 source proof extends
the opt-in core as described below. B3 exercises that managed-resource contract
with private Web/SQLite integration. Public facade adoption remains proposed.

Terms such as `required_foreground`, `attach_ready` and `attach_worker` mix
success policy, lifecycle state and execution mechanism. The owner also
questions an unexplained `attach` / `try_attach` pair and the intent of
`Composition::builder()`. That last expression was an illustrative suggestion,
not an existing entry point.

The [source inventory and primary-source research](../M28-HOST-RESOURCE-LIFECYCLE-DESIGN.md#2-what-exists-at-the-source-baseline)
distinguish actual behavior from proposals. Microsoft Generic Host provides
a useful default-first lifetime pattern, while Tokio and Winit illustrate
the need to separate signalling, completion, executor ownership and native
thread affinity. These are design references, not new dependency selections.

## Proposed decision

### 1. Expose defaults and advanced control through one engine

Use three layers:

| Layer | Proposed responsibility |
|---|---|
| Application / high-level Host | Select environments and event sources, validate the application plan and run its coordinated lifetime using defaults. |
| ResourceGroup | Describe and customize typed resource ownership, acquisition, readiness and completion contracts. |
| Shared lifecycle engine | Enforce state, admission barriers, failure retention, dependency lifetime and cleanup progress for both paths. |

A built-in Host creates its default ResourceGroup. A custom Host can use that
same mechanism. Host identity remains independent of thread, process, renderer
and runtime identity. ResourceGroup is a lifetime scope, not a thread pool or
service registry.

Candidate public vocabulary is `Application`, `with_host`, `ResourceGroup`,
`with_resource`, `request_shutdown` and `shutdown_until`. Exact module paths,
Rust signatures and a static Host extension protocol are not frozen by this
ADR. The existing server-specific `server_axum::App` keeps its established role.

### 2. Keep the ordinary path short and defer resource acquisition

The full multi-environment API below remains illustrative. The bounded A/B
research crate implements the Application/Web/custom subset and resource
contracts described in its [README](../../examples/host-lifecycle-research/README.md):

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

Host constructors describe configuration. `with_host` registers descriptions.
`build` validates known ownership, topology and execution constraints without
starting those resources. `run` acquires and drives them. Fallible configuration
can return Result, but construction does not itself bind sockets, spawn threads
or create an event loop.

All selected Hosts participate in application success by default. Startup or
fatal runtime failure requests coordinated shutdown; ordinary registration
does not require `required()`. Optional-host participation, restarts and failure
isolation need separate explicit contracts and are deferred.

The existing immediate `try_required_*` factory semantics are preserved until
an authorized migration. The new deferred acquisition model is an additive
behavioral contract, not a completed naming-only change.

### 3. Make customization explicit without losing defaults

`configure_resources` is a candidate hook to extend a Host's default group or
use a typed override that the Host explicitly supports. It must not silently
replace mandatory admission, driver or completion behavior. A custom Host
author can build a ResourceGroup directly and provide its resource plan,
execution requirements and compatible driver through the proven extension
protocol.

In the current private proof, `with_resource(value)` preserves supplied passive
RAII ownership, while `with_managed_resource(label, descriptor)` adds deferred
acquisition and explicit release. Both ResourceGroup and WebHost provide the
latter operation. These additions preserve the Host's default obligations;
`configure_resources`, typed driver overrides and general live asynchronous
owner adoption remain outside B1/B2.

Distinguish inert acquisition descriptions, already owned values, borrowed
values and running activities with original completion owners. Only the first
belongs in the ordinary inert plan by default. Advanced live-resource adoption
must prove cleanup ownership on configuration failure and pre-start cancellation.

Default adapters implement known resource behavior. Arbitrary custom resources
still need a truthful contract for availability, stopping and terminal
completion. A cancellation token, a generic shutdown callback or dropping a
JoinHandle cannot establish that contract by itself.

### 4. Preserve typed local ownership and use a bounded hierarchy

Start with an Application root and Host child groups. Share resources through
an owning ancestor with explicit typed access; assign exactly one cleanup owner.
Allow a dependency on an earlier resource or an ancestor. Defer arbitrary
sibling dependencies, cycles and a general dependency DAG.

A driver used by several Host groups is an Application-root resource.
NativeHost contributes the native driver and manages its window group; the
root retains the loop until every child needing it has completed. Do not
represent that lifetime as an unchecked borrow into a sibling Host.

Preserve local Rc, borrowed values and native thread-affine resources. No
application-wide Send, Sync or 'static bound and no universal
`Box<dyn Any + Send + Sync + 'static>` storage requirement is adopted.
A selected cross-thread adapter supplies its own boundary-specific bounds.

Startup prerequisites, resource lifetime, execution affinity and exclusive
resource claims are distinct. Validate permitted relations and retain a
dependency until all activity using it has actually completed. Reverse
registration order alone does not establish this guarantee.

### 5. Preserve ownership through failures and incomplete shutdown

The engine coordinates admission fencing, shutdown signalling, observation
of original completion owners and safe final release. Existing admitted work
must follow the adapter's explicit drain/reject policy before its domain
authority can be invalidated. Control signalling cannot depend on capacity in
the data queue.

Primary failure remains identifiable; cleanup failures are additional
diagnostics. Failed admission or retryable release progress does not discard
the responsible owner. Terminal release with an error is a separate outcome,
described below. A failed signal does not prevent observing that an activity already
exited. Successful one-time operations are latched and unfinished effects can
be retried under the resource's documented retry contract.

Factories must either clean up everything that did not transfer or retain
a partial owner in the group/failure result. Error conversion must not drop
that obligation. A completed cleanup can accompany an operational failure.

`request_shutdown` requests progress; `shutdown_until` cooperatively advances
shutdown through one shared deadline and returns Complete or Pending. Pending
retains the original owners and dependencies. Neither the deadline nor Drop
implies preemption or completed cleanup.

Select terminal-only semantics for the convenient `run()?` form: it returns,
including on an operational error, only after managed cleanup is terminal.
This can wait indefinitely for uncooperative work. Deadline-sensitive callers
use the advanced retained-owner driving interface; no hidden timeout detaches
their resources. Abort/process termination is outside this cleanup guarantee.

The native driver must progress readiness and local cleanup inside its event
loop. It must neither wait for readiness before entering the loop nor block
a callback waiting for future callbacks. Synchronous, async and step-driven
advanced forms require a compiling driver proof.

### 6. Evolve the current engine where its ordering is insufficient

At the baseline, Composition closes admission, closes locals, signals workers
and then observes completion. That supports its existing bounded consumers.
It cannot automatically guarantee that a database stays open until a dependent
worker joins.

The authorized B1/B2 source proof adds that bounded extension through
`Builder::with_resource`, `with_activity` and `with_group`. Existing Local
close-before-worker behavior remains unchanged. The new final-release traversal
starts only after admission is closed, legacy local cleanup is complete and
every original worker in the root has joined.

Groups contain typed parts under the same coordinator and failure ledger.
Reverse registration and structural group traversal retain earlier dependencies
when a later release is incomplete. This is a conservative ordering contract,
not an arbitrary dependency DAG or a guarantee of independent sibling progress.
The sealed `ResourceParts` bound lets generic Host code name crate-built typed
storage without implementing a second lifecycle traversal.

Resource release distinguishes progress from its terminal outcome:

| Callback result | Ownership and retry contract |
|---|---|
| `Ok(ReleaseCompletion::Pending)` | Retain the original resource, cleanup operation and earlier dependencies. |
| Outer `Err(error)` | Record a retryable progress failure and retain those owners for a later attempt. |
| `Ok(ReleaseCompletion::Released(Ok(())))` | Latch successful release and drop the resource value before advancing earlier dependencies. |
| `Ok(ReleaseCompletion::Released(Err(error)))` | Record terminal failure once, drop the resource value and never repeat its finalizer. |

`ReleaseStatus` exposes the latched Pending/Succeeded/Failed outcome. A
`ManagedWorker` delegates original worker bookkeeping and retains its
coordination storage; its final callback must retire the activity payload before
earlier dependencies release. The private facade does so with `Option<A>`.

The private `ManagedResource` descriptor is inert before installation.
`acquire` stores partial owners before another fallible step; failed acquisition
still requires their release. A descriptor whose acquisition was never attempted
skips its release callback. `release_within` cooperatively progresses the
original cleanup operation. Nested traversal shares one absolute deadline,
passing only remaining duration to callbacks. It does not create an executor,
automatically poll arbitrary futures or promise a hard timeout.

### 7. Keep execution policy separate and native uniqueness honest

The initial native proposal uses one root-owned Winit driver, contributed by
NativeHost, with multiple windows. Two inert NativeHost descriptions do not run two loops, but a plan
requesting two independent native loops must reject that known conflict before
acquisition.

Winit 0.30.13's native creation guard is process-wide. Application-local
validation alone cannot detect every external loop or make a previously created
loop recreatable. Startup checks and documented platform entry-thread
preconditions remain necessary. Platform-specific any-thread support does not
become a portable default.

Do not add a universal pool or one thread/process per Host. Preserve the
current Web service-thread and caller-driven native/UDP choices initially.
An explicitly shared executor needs a lifetime and driving contract; a borrowed
executor is not shut down by its borrower.

Defer public `ExecutionPreference` until an adapter has multiple supported
placements. A future preference must declare its fallback and report the chosen
placement; a strict requirement must fail if unsatisfied. Do not silently put a
blocking service onto the native callback thread after spawn failure.
Separate-process hosting requires a later IPC and supervision design.

### 8. Name operations by intent and define any try distinction

Avoid `required_*`, `foreground` and `ready/worker` as the ordinary resource
categories. Use specific execution constraints and resource contracts where
advanced developers need them.

A Result return does not universally require a `try_` prefix. Introduce a pair
only when its distinction is deliberate and documented. Fallible acquisition,
nonblocking acquisition, optional participation and fallback are different
policies; `try_` must not ambiguously stand for all of them.

During an authorized source migration, choose canonical names and update
affected consumers together following the naming guide. Preserve historical
evidence rather than rewriting it to use future vocabulary.

## Alternatives considered

| Alternative | Disposition |
|---|---|
| Only rename the existing methods | Insufficient for the requested default lifecycle layer, though useful as a separately scoped naming change. |
| Expose the complete low-level callback surface first | Fails the owner's simple entry-point requirement. |
| Separate built-in and custom lifecycle implementations | Reject; it creates two sets of ownership and failure semantics. |
| Default Host facade over ResourceGroup and one evolved engine | Recommended for owner adoption. |
| Universal DI, arbitrary dependency graph or automatic process/thread pool | Defer; current consumer needs do not justify their additional lifetime and execution contracts. |

## Consequences

Ordinary application code focuses on Host intent. Advanced users can customize
resource management without rewriting the coordinator. Local ownership,
direct Web APIs and optional adapter dependencies remain viable.

The cost is substantive implementation work: deferred acquisition, dependency
retention and driver integration require proof. Terminal-only run can wait
without a fixed time limit; the advanced interface must remain available for
explicit deadlines. Static typing may constrain builder ergonomics, and those
constraints should be resolved through real consumers before fixing signatures.

## Adoption and verification

This ADR stays Proposed for public adoption. After the documentation review,
the owner authorized the unpublished YT-286-A proof, B1 through B2, and then
the bounded B3 application integration.
The [implementation records](../M28-HOST-RESOURCE-LIFECYCLE-DESIGN.md) distinguish
those source proofs from the remaining proposal. Review PASS does not itself
adopt the full public facade or authorize commit, push or publication.

At its reviewed checkpoint, A compiled the `Application::new()` entry, default
WebHost and external custom Host using the unchanged Composition core. It
proved passive RAII retention through original joins. B1/B2 subsequently adds
the opt-in final-release core and private managed-resource descriptors described
above, preserving existing Local semantics and the supplied-value facade path.
Focused core and resource checks cover the new contract; final B1/B2 acceptance
requires configured independent review of the exact source and its evidence.

The authorized B3 source integration uses one resource-owned SQLite worker and
a typed bounded client from real HTTP handlers. Web admission/drain and its
original join precede SQLite final release; closing the shared command sender
is independent of queue capacity. Before join, Pending preserves the original
database thread handle. A failed explicit Connection close retains the live
Connection for retry, including after that join. Thread join, release initiation
and an open Connection are distinct
observations; Pending alone does not prove the Connection remains open.

The [finite example](../../examples/host-lifecycle-research/examples/web_with_sqlite.rs)
and [focused integration cases](../../examples/host-lifecycle-research/tests/sqlite_web.rs)
cover the bounded acceptance contract: persisted readback, admitted HTTP drain,
later bind failure with an explicitly controlled cleanup fault, actual SQLite
schema/readiness failure and actual open failure. Final B3 acceptance requires
executed evidence and configured independent review; no PASS is asserted by
this design text. SQLite remains a private research dev-dependency, with no
mandatory core/SDK database feature.

Native/shared driver proof, general live asynchronous owner adoption, the full
configuration hook, arbitrary dependency graphs, pools and process placement
remain outside B3. YT-286 remains Todo for wider public adoption and compatibility.
The [delivery matrix](../M28-HOST-RESOURCE-LIFECYCLE-DESIGN.md#10-delivery-and-verification-gates)
requires local/borrowed ownership, partial-start rollback, actual dependency-safe
completion, native driver progress, customization safety, external consumer
usage and preserved direct Web/feature isolation before promotion.

ADR 0013's accepted F2 scope remains the implemented baseline. This proposal
does not retroactively alter that acceptance, erase its evidence or close
physical F3/D4 qualification.
