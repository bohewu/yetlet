# F0-A — Private Opt-in Composition Root Contract

- **Date:** 2026-11-18.
- **State:** Private design proposal; no runtime implementation or public API promotion.
- **Source baseline:** `845902f941a66738d46eabe7398ebc6a247a41f6` on `docs/m28-composable-architecture`.
- **Authority:** The owner authorized F0-A research, planning and Contract Review only. F0-A/F0-B labels are candidate names, not assigned YT backlog identifiers.
- **Parent:** [M28-E decision](M28-E-PROMOTION-DECISION.md), [M28 plan](M28-COMPOSABLE-APPLICATION-PLAN.md), and [ADR 0013](adr/0013-composable-application-and-hosts.md), which remains **Proposed**.
- **Delivery boundary:** One new design document plus ignored evidence/handoff. No product code, manifests, existing API, ADR status, Roadmap/Backlog, SDK/CLI or publication change. No Stage, Commit or Push.
- **Review boundary:** Independent review evaluates this contract and its source/compatibility claims. Its result is recorded in the ignored F0-A handoff; it cannot qualify an unimplemented F0-B runtime.

## 1. Recommendation and decision to test

**Keep two concrete, application-owned composition roots. Share documented ownership and completion semantics; do not extract a common runtime type or Host trait yet.**

The Headless and Pure Web consumers below are deliberately different workloads. Their common obligations are explicit resource ownership, meaningful readiness, failure retention, stop coordination and observed completion. Their state representation, driving mechanism, readiness event and cleanup operation differ. A documented contract does not require a common Rust trait.

A future private extraction is justified only if both real consumers need the same ownership-sensitive operation and removing the helper duplicates that operation. Repeated names such as Ready or Stopped, two similar constructors, or an artificially added worker do not establish that need. **A two-consumer result of “no shared type needed” is a valid successful design outcome.**

This proposal does not accept all of M28-E or ADR 0013 on the owner's behalf. It does not qualify Game + Network + Axum in one process, physical LAN, native non-loopback UDP, Windows, production security or performance.

## 2. Source inventory and remaining gaps

The references below name inspected source at the baseline, not new test results.

| Existing surface | Actual contract | Consequence for F0-A |
|---|---|---|
| [B1 main](../examples/composable-host-research/src/main.rs) and [local ports/hosts](../examples/composable-host-research/src/lib.rs) | Two concrete caller-thread hosts share one counter owner, bounded admission and immutable Rc publications; no worker | Ordinary Rust already composes capabilities without a listener or runtime |
| [B2 worker](../examples/composable-host-research/src/worker.rs), especially Worker and Group | One explicitly owned std thread, two local attachments, cancellation outside the data queue, retained JoinHandle after an outcome-wait timeout, and a blocking join fallback in Drop | Useful ownership precedent; not a generic hard shutdown deadline or a game-worker adapter |
| [C1 simulation](../examples/composable-host-research/src/simulation.rs) | Simulation contains Rc<Snapshot>; HeadlessRunner drives its single reducer with explicit logical ticks and at most 32 validated commands per tick | Preserve caller-local ownership; do not move an existing Simulation across a thread boundary |
| [C1 demo](../examples/composable-host-research/src/bin/headless-demo.rs) | Two independent simulations compare deterministic replay | Replay comparison is not evidence that two host capabilities share one authoritative state |
| [Research manifest](../examples/composable-host-research/Cargo.toml) | Independent workspace, publish=false, no external normal dependencies | Headless composition can remain std-only; GPU/Web dependencies need a separate selection |
| [Axum App](../crates/yetlet/src/server_axum/app.rs) | Owns Router<S>; S denotes missing state. configure/try_configure/with_state/into_router already provide typed composition. serve/serve_on use the caller's Tokio runtime | Preserve the native path; new lifecycle coordination belongs in an optional consumer, not this public facade |
| [Minimal Web example](../crates/yetlet/examples/server_axum/minimal.rs) and [ADR 0011](adr/0011-application-startup-api.md) | Existing API/SSR application can directly call App::serve; custom listener/address/drain control uses into_router and native Axum/Tokio | A composition root must remain optional. Router construction does not imply a listening server |
| [Yetlet manifest](../crates/yetlet/Cargo.toml) | Default html; server-axum opts into html/Axum/Serde/Tower/Tokio, with Tokio net/rt directly selected | “Pure Web adds no new mandatory dependency” preserves this existing feature selection; it does not mean HTTP has no Tokio or that server-axum omits html |
| [Actions](../crates/yetlet/src/actions/mod.rs) | ActionExecutor returns the command's response; LocalExecutor invokes its handler; no universal Send bound | Do not redefine public command completion as queue admission |
| [ADR 0013](adr/0013-composable-application-and-hosts.md) | Proposed ordinary composition, distinct ownership/topology, optional state policies and native Web preservation | F0-A narrows one private experiment; it does not supersede accepted Axum contracts |

B0 was tested at `30b0625` and recorded at `9b52970`: 55 selected tests and eight doctests. B1/B2/C1 and later milestones retain their own source-bound evidence. F0-A reuses those records without rerunning them. Its source-identity receipt records the inspected files and the selected Web baseline comparison; identical files are not a new runtime or benchmark result.

The remaining gap is **two independently useful consumers of an actual shared operation**. B2's counter worker is not an adapter for the C1 Rc game. Existing separate GPU, UDP and Axum examples do not establish a unified three-host runtime. No such gap is closed by this document.

## 3. Responsibilities and ownership

A composition root is application code at the resource-lifetime boundary. It may be a main function, a concrete struct, or a scoped async function. It is not necessarily a Yetlet object.

~~~mermaid
flowchart TD
    O["Application entry point"] --> H["Concrete Headless root"]
    O --> W["Concrete Web root"]
    H --> C["Input and observation attachments"]
    H --> S["One C1 Simulation"]
    C -->|"validated input / immutable snapshot"| S
    W --> R["Existing App into Router"]
    W --> D["Selected native HTTP driver"]
    D --> R
    B["Caller-owned Tokio runtime"] -. "drives; remains caller-owned" .-> D
~~~

The two roots in this diagram are alternative independent consumers, not an implemented combined application. Headless has no Renderer or Transport. The optional HTTP driver owns its selected listener/serve future; Axum owns HTTP routing and connection behavior. An owned worker, when independently justified, belongs to the selecting root and is not allocated per Host.

| Responsibility | Owner and permitted access | Excluded responsibility |
|---|---|---|
| Headless authoritative state | One HeadlessRunner/Simulation, mutated only through its existing step operation | Neither input attachment nor observer receives unrestricted mutable state |
| Web domain state | Application-selected WebState passed through existing Axum state rules | No compulsory game owner, global actor, tick or command queue |
| Scheduling | Headless caller supplies logical steps; Web caller supplies an executor; selected worker owns only its own loop | No global scheduler, runtime creation or shutdown of borrowed executors |
| Lifecycle coordination | Concrete root knows its selected required resources and failure policy | No registry, dynamic hotplug, automatic restart or implicit dependency injection |
| Domain generation/revision/tick | Existing C1 domain/input/snapshot contract | Root lifecycle does not reuse revision as a readiness or cleanup flag |
| Renderer/Transport/Protocol | Remain separate existing or later explicitly selected components | No shared lifecycle trait doubles as a rendering, byte movement or protocol API |

A caller-owned async operation is not necessarily pinned to one OS thread. Headless caller-local Rc values remain local; native Axum retains its own Send/Sync bounds. Only a genuine cross-thread boundary acquires the corresponding bounds. No unsafe Send implementation, Rc-to-Arc conversion, forced runtime or new type erasure is proposed.

## 4. Alternatives and extraction threshold

| Option | Shape and benefit | Cost / compatibility | Disposition |
|---|---|---|---|
| A. Concrete roots and shared written invariants | Local Headless ownership plus native Web setup/serve; failures remain concrete | Some repeated coordination statements; no new framework bounds, dispatch or dependency | **Recommended first private implementation candidate** |
| B. Small private typed helper after two consumers | A specific ownership-preserving completion result or worker helper used by both callers | Must expose pending ownership; generic errors/lifetimes can obscure different completion semantics; value-only reports do not supervise resources | **Conditional extraction**, only after duplicated operational logic is demonstrated |
| C. Generic Application/HostGroup with uniform async Host | Central registration/start/stop interface | Imposes async/Send or complex local variants, boxes/errors/registry, and risks hidden runtimes or artificial workers | **Reject for F0**; no demonstrated requirement |

For option B, answer all of these with actual future code before extracting:

1. Which ownership-sensitive operation is duplicated in **both** consumers?
2. Can the helper preserve local Rc, caller-owned executors, and native Axum semantics without dummy methods?
3. Does a timeout retain the real pending owner, rather than only return a status enum?
4. Are the call sites and failure paths simpler after extraction, including their type bounds?
5. Can either consumer still run without selecting this helper?

If the commonality is only vocabulary, retain option A. No line-count threshold, invented benchmark target or generic “zero-cost” claim substitutes for that assessment.

## 5. Minimal private semantic contract

These are obligations for the proposed consumers, not new Rust API names or required methods on every Host.

| Operation / observation | Required meaning | Who supplies the concrete mechanism |
|---|---|---|
| Prepare | Validate configuration and establish resource owners before enabling work | Root; fallible setup errors retain their original cause |
| Attach / start | Start only selected capabilities, recording every acquired resource before another fallible step | Root plus the capability's native scheduling rules |
| Ready | A named readiness condition has been observed for this lifetime; it is not perpetual health | Local initialization, explicit bind, or an owned worker acknowledgement, as appropriate |
| Drive | Caller steps, executor polls, or a selected worker performs work | Consumer-specific; not a universal async run method |
| Fail | Latch the first failure observed by the coordinator, with component/stage; keep later cleanup failures separately | Root; no global wall-clock ordering between concurrent failures is claimed |
| Request stop | Idempotently prevent new selected work and signal/wake owned work | Control path independent of a saturated data queue |
| Finish | Dispose attachments and observe all actually owned work ending before claiming complete cleanup | Local disposal, native serve completion, or actual join as applicable |

Use separate observations for **primary outcome** and **cleanup progress**. A failed start can have complete cleanup; a successful stop request can still have incomplete cleanup. “No worker was allocated” is not “worker joined.” A report alone does not own or complete anything.

For the first concrete samples, all selected attachments are required. A required startup/runtime failure closes the affected root's admission and requests stop of its owned work. Optional-component continuation is deferred until a real optional component has a named degraded behavior; no general required/optional registry is needed.

Ordinary HTTP error responses and a client disconnect are request/connection outcomes under Axum, not automatic application-failure signals. C1 invalid input remains an atomic domain rejection; a Headless sample may explicitly choose to stop its finite run on that error. That choice does not change the reducer's semantics.

### Partial startup and readiness races

Record ownership immediately when an allocation/start succeeds. If the next required attachment fails, stop only what actually started, in reverse dependency order. Do not return an ordinary error while implicitly dropping an unjoined worker or an incompletely drained server.

A later implementation must choose one of two honest forms at each fallible startup boundary:

- Cooperatively complete cleanup before returning, documenting any blocking join.
- Retain the partially started root in the caller, or return that owner with the error, so incomplete cleanup can be resumed.

A late Ready acknowledgement cannot reopen a root that has entered Stopping. The receiving root checks its own start/stop phase. F0 has no automatic restart or dynamic membership, so it needs no new general generation allocator. Existing domain generation checks still apply to Headless input.

Do not wait synchronously for a callback that requires the blocked caller loop to run. Headless setup can complete inline; an async Web driver is polled by its caller; an independently selected worker acknowledges after its own required initialization.

## 6. Caller-local and owned-worker lifetimes

### Caller-local case

The Headless root owns its input attachment, one HeadlessRunner, and a latest-snapshot observer on the calling thread. It validates each input batch through the existing reducer, publishes the returned immutable Rc, and never invokes a domain step from the observer.

Stop closes root admission before disposing attachments. Work already applied stays applied; any locally buffered but unapplied input is accounted for explicitly. Once attachment disposal completes, cleanup is complete with **join not applicable**. Retained immutable snapshots may legitimately outlive the root; cleanup is not a claim that every clone has vanished.

Plain scopes and concrete structs are sufficient. Do not add a task, a loopback socket, a NullRenderer, an artificial “join” operation, or a second authoritative state to make this case resemble B2.

### Explicit owned worker, only when justified

The design permits **at most one explicitly selected std worker** in a later bounded variant. The root owns its unique JoinHandle and the cancellation/wakeup mechanism. A worker may create caller-local domain values inside its own thread, but the existing C1 Simulation cannot simply be moved into it. Exporting owned thread-safe projections or designing a C1 worker adapter is a separate implementation choice, not established B2 behavior.

Worker readiness means initialization finished and the selected loop is prepared to receive work. It is not proof that the worker cannot fail immediately afterward. The root must still observe worker failure during running and at join, preserving the primary cause and any secondary cleanup failure.

Cancellation must not wait for a data-queue slot. Bound queue capacity and work per iteration; avoid application callbacks or blocking IO while holding a control lock. A stop request closes admission, wakes the worker and retains ownership until join. Pending work and any in-flight publication race need a consumer-specific applied/discarded policy; F0-A does not silently make admission equivalent to execution.

**A timeout bounds observation, not arbitrary thread termination.** B2 first waits for an outcome and then calls join; its Drop fallback also blocks. An outcome message, is_finished check, stop flag, or returned timeout is not itself a joined thread. std JoinHandle drop detaches, and join may wait for thread-local destruction. Therefore a future timed completion attempt must preserve the owner/handle and report incomplete cleanup on deadline. Subsequent completion attempts may report Joined only after actual join. No hard real-time bound, thread killing or safe cancellation of arbitrary blocked code is promised.

For a deliberately cooperative research worker, an explicit final blocking join can remain a documented fallback. It must not be hidden behind a supposedly bounded destructor or deadline API. An uncooperative worker leaves a pending-cleanup result and requires an explicit application/process-level decision outside this slice; it is never reported as successful teardown.

### Borrowed executor and task case

Borrowing a CallerScheduler token or using an existing Tokio runtime transfers no right to shut it down, cancel unrelated tasks, or join unrelated workers. A root owns only the tasks it explicitly starts, including their handles. If a later variant uses Tokio tasks, cancellation is requested first and their handles are awaited; dropping a JoinHandle detaches. Aborting a task does not synchronously prove its completion, and started spawn_blocking work is not generally abortable. These facts do not justify imposing Tokio on Headless.

## 7. Consumer H — Deterministic Headless composition

**Purpose:** Exercise explicit local composition around the existing C1 domain, independently of HTTP.

Proposed concrete resources:

- One finite typed input tape/source owned by the application.
- One existing HeadlessRunner containing the only authoritative Simulation.
- One caller-local read-only observer retaining only the latest Rc<Snapshot>.
- A root phase and concrete setup/domain error, without a new shared framework type.

The source and observer are capability attachments, not invented OS hosts. Their separate setup/stop obligations give the root something real to coordinate. The source submits a bounded batch for the next logical tick; the root drives the existing HeadlessRunner tick; the observer reads the publication. The existing maximum of 32 commands per tick and generation/tick/sequence checks stay with C1. Input-tape length is finite and chosen by the sample; there is no new wall-clock scheduler.

Proposed readiness: input and observation attachments initialized, owner available, and root admission open. Proposed failure policy: a required attachment setup failure unwinds acquired attachments; an invalid sample input stops that finite run while retaining C1's unchanged prior publication and error. Stop rejects later root submissions and releases attachments. No worker or join is required.

This consumer can prove two capabilities share the same authoritative publication. A separate replay comparison, if useful for the changed wiring, must be identified as a reference run rather than a second host authority.

## 8. Consumer W — Pure Web composition

**Purpose:** Assemble an ordinary SSR/API application using existing App/Router/Tower, with no game domain.

Select concrete application state suitable for native Axum (for example immutable site metadata), one HTML page and one ordinary API route. Two routes are **not** two Hosts. Handler logic accesses its native application service/state directly. The root does not wrap each request in a lifecycle lock, actor, tick or mailbox.

The simplest meaningful setup remains an ordinary function returning a resolved App<()> or Router<()> through existing configure/try_configure/with_state/into_router. If that function suffices, do not introduce a wrapper merely to match Headless's shape. The unchanged minimal App::serve path stays available without any F0 type.

### Distinguish three observations

| Observation | Meaning | What it does not prove |
|---|---|---|
| Router configured | Fallible application setup succeeded; native Router is available | No listener, serving loop or successful HTTP exchange |
| Endpoint bound | Explicit native TcpListener bind succeeded and its actual address is known | The serve future is being polled, application health, or drain completion |
| Driver active / serving observed | Caller has installed and is polling the owned native driver; a future socket acceptance test separately observes an actual response | Perpetual health or a synchronous cross-component readiness guarantee |

For a later lifecycle proof, use the existing into_router escape hatch with an explicitly chosen loopback listener and native axum::serve(...).with_graceful_shutdown(...). The root supplies no runtime. Its caller owns the executor and retains the serve future or an explicitly spawned task handle. This is an opt-in **consumer** of Axum, not a new serve_with_shutdown method on Yetlet.

Validate all other required setup before polling the server, so an early accepting socket cannot bypass the root's required setup. Binding and driver ownership are distinct from reporting a successful request. Native Axum accept/connection behavior remains native; do not promise that its normal serve future surfaces every post-bind IO error.

### Web stop and drain

The application signals native graceful shutdown, stops admitting new server connections through that native path, and continues driving the same serve future to completion. Existing in-flight work may complete according to Axum's drain behavior. State/services needed by those handlers remain alive until their owned work completes.

A finite **wait** may expire while a long-running handler is still active. In that case retain the server future/task handle, required state and caller-runtime lifetime, report cleanup incomplete, and continue observation later. The sample must not drop a serve future or abort only the top-level task and infer that Axum's internally spawned connection work has ended. The future that owns the waiting/drain operation remains accountable.

Native serve completion covers its own connection lifecycle, not arbitrary tasks a handler detached. The proposed minimal routes create no detached jobs. Adding background jobs requires their own ownership and is outside the initial Web consumer.

A Router-only consumer owns no listener/serve task, so there is no server stop or join to fabricate. A full server variant remains a native async lifetime, not an OS worker invented to satisfy a shared interface.

## 9. Dependency, cost and compatibility constraints

No shared support crate or root Cargo feature is proposed in F0-A.

| Candidate location / selection | Permitted dependency direction | Preservation constraint |
|---|---|---|
| Later Headless private binary/module in examples/composable-host-research | Existing std-only C1 package | Keep independent workspace and publish=false; no Tokio/Axum/GPU dependency |
| Later separate examples/composition-web-research workspace | Local Yetlet with explicitly selected server-axum; native Axum/Tokio/Tower only as its own code needs | publish=false; no root workspace member/default-feature addition and no dependency on game/UDP/native-GPU examples |
| Any later private common helper | Only after the two consumers demonstrate the same operation | No new core dependency, mandatory selection, public re-export or registry; assess whether a crate is needed separately |

These are proposed locations, not created packages or exact manifests. Pin any later selected Web versions to the repository's existing compatible versions; inspect the resulting normal dependency/feature graph in that implementation slice. Do not enable rt-multi-thread solely for composition.

Expected costs are qualitative: concrete local calls retain existing Rc cloning and C1 snapshot allocation; an owned worker adds thread-stack, synchronization, bounded buffering and possibly projection copies; Web keeps native request routing and scheduling and may add only explicitly selected startup/shutdown coordination. None is a measured latency, allocation, throughput or zero-overhead result. No benchmark infrastructure is needed for this docs-only task.

Migration is additive and private. Existing App names, prelude, state resolution, Router/Tower behavior, serve/serve_on defaults and error semantics remain unchanged. Public Actions keep their response semantics. Any later change to a published surface requires its own compatibility decision and affected-path validation. Source preservation and opting out of the new example are required; private APIs do not gain stability merely by appearing in a sample.

## 10. Proposed implementation sequence and acceptance

**Every implementation below remains unstarted and requires separate owner authorization.** Candidate labels do not create backlog IDs or authorize a whole chain.

| Order / candidate | Bounded work | Proposed evidence and stop condition |
|---|---|---|
| F0-A, current | Source analysis, options, concrete ownership contract, compatibility and review | New-document checks and independent actual Contract Review; no runtime PASS |
| F0-B1 | Implement only Consumer H with concrete local root and unchanged C1 reducer | Changed-wiring checks: one authority, immutable observation, required setup failure cleanup, late input rejection and join-not-applicable; no display/socket/runtime/worker. Stop after this slice |
| F0-B2, after separate authorization | Add Consumer W in its own unpublished workspace using native App/Router | Router SSR/API response equivalence and setup errors; if listener lifecycle is selected, one bounded Ubuntu loopback serve/drain check with actual completion and borrowed runtime still usable. No game queue/tick or forced worker |
| Extraction decision, after both consumers | Compare actual duplicated coordination and both call sites | Keep concrete roots if commonality is only reporting/vocabulary. Otherwise review the exact private helper, dependencies and error/pending ownership before extraction |
| Optional worker variant, only for a concrete need | At most one cooperative owned worker in the selected consumer; explicit cross-thread data | Focused readiness failure, full-queue cancellation, cause preservation, incomplete cleanup retaining ownership and eventual actual join. Do not retrofit a worker into Web just to reuse the helper |
| Public promotion, deferred | New owner decision, ADR amendment and independent source-scoped review | No public Application/HostGroup/Host/Transport or stable crate follows automatically |

Proposed verification is risk-scoped:

- **Current repository requirement:** run cargo xtask check docs for the new document and a no-index whitespace check for this untracked file. Existing source tests, reviewer results, GPU/UDP/VM/Windows runs remain reused, not repeated.
- **Future changed Headless wiring:** use focused native Ubuntu checks of that wiring; repeat reducer tests only if its implementation/contract changes or a concrete uncovered interaction requires them.
- **Future changed Web consumer:** verify its new native request/lifecycle path. Repeat B0's affected subset and normal dependency comparison only if relevant shared Web/API/features change; unchanged public code does not trigger the entire old matrix.
- **Future owned work:** actual completion is required for every process/thread/task the new check starts. No generic validator, timeout-killing framework or certification machinery is proposed.
- GPU, browser, UDP, Multipass and Windows add no evidence for this unchanged docs-only surface. They become relevant only to separately authorized affected behavior.

These proposed tests directly address the user's ownership, consumer-difference and Web-preservation goals. Exact commands and fixture budgets belong to the separately authorized implementation slice; they are not permission to run it now.

## 11. Risks and rejection conditions

| Risk | Practical consequence | Contract response |
|---|---|---|
| Extracting a type because names match | Local consumers acquire meaningless async/Send/join behavior | Require duplicated operation in both real consumers; option A remains valid |
| Confusing caller ownership with thread affinity | Rc state may be moved incorrectly or native async scheduling is misrepresented | Keep local and executor-driven cases distinct |
| Claiming Ready at Router construction or spawn | Work may be admitted before required initialization, or status overstates serving | Name the exact readiness observation; gate local admission and native polling separately |
| Losing a handle on timeout/startup error | Worker/task survives without an accountable owner | Retain partial root/handle, keep primary failure and cleanup separately |
| Blocking fallback hidden inside a deadline | Caller/UI/executor can stall beyond the advertised wait | No generic termination deadline; blocking fallback must be explicit and limited to cooperative code |
| Aborting/dropping the server and claiming all work ended | Internal connection tasks or handler-owned jobs can outlive the report | Observe native drain and separately owned jobs; incomplete remains incomplete |
| Forcing shared game state into ordinary Web | Changes concurrency, response semantics and dependencies | Native Web handlers keep their own state policy; actor only if explicitly selecting that domain |
| Treating design review as runtime/promotion evidence | Unsupported API, LAN or performance claims | ADR remains Proposed; F0-B and physical D4 stay unverified |

The recommendation is rejected if the private implementation requires a second Router, hidden runtime, per-Host worker, dynamic DI, forced game mailbox, or public core dependency merely to make the two examples uniform.

## 12. Research references and evidence scope

Read on 2026-11-18. These upstream sources support lifecycle constraints, not proof that F0-B exists:

- [Rust std JoinHandle](https://doc.rust-lang.org/std/thread/struct.JoinHandle.html): unique completion handle, detach on drop, and actual join semantics.
- [Tokio 1.53.3 JoinHandle](https://docs.rs/tokio/1.53.3/tokio/task/struct.JoinHandle.html): task ownership, cancellation-safe borrowed awaiting, and spawn_blocking limitations.
- [Axum 0.8.9 serve source](https://docs.rs/axum/0.8.9/src/axum/serve/mod.rs.html): native accept behavior, graceful shutdown and connection-drain waiting.
- [Axum 0.8.9 WithGracefulShutdown](https://docs.rs/axum/0.8.9/axum/serve/struct.WithGracefulShutdown.html): native serving future and its bounds.

Ignored evidence is kept under `.pipeline-output/m28-f0a-20261118/`: source identity, exact review input/output, document checks, original operational failures and final handoff. Prior M28-E R1 actual FAIL3/R2 actual PASS, all B–D receipts and successful-log collision disclosures remain in place. The original physical D4 gate stays **Not verified**; the two-VM relay supplement is distinct. This contract changes none of those results.
