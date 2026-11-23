# ADR 0013 — Composable Application and Explicit Host Ownership

- **Status:** Accepted for the scoped F2 amendment on 2026-11-19; remaining implementation gates stay open.
- **Date:** 2026-11-17.
- **Tracking:** YT-265, M28-A.
- **Decision owner:** Project maintainer, explicit scoped F2 adoption on 2026-11-19 after private F1 and YT-281/282-A evidence.
- **Related:** [M28 plan and source inventory](https://github.com/bohewu/yetlet/blob/5d57fce472f4ec01ce9bf17b3a387a43d3e38a5e/docs/M28-COMPOSABLE-APPLICATION-PLAN.md), [ADR 0001](0001-separation-of-concerns.md), [ADR 0003](0003-renderers-and-loading.md), [ADR 0005](0005-opt-in-capability-modules.md), [ADR 0011](0011-application-startup-api.md), [ADR 0012](0012-server-axum-prelude.md).

## Layered Host follow-up — 2026-11-21

[ADR 0014](0014-layered-host-resource-lifecycle.md) and its
[research/design document](../M28-HOST-RESOURCE-LIFECYCLE-DESIGN.md) now record
owner-selected Core Host defaults, optional capabilities and ResourceGroup
customization. Their YT-287–294 checkpoints distinguish implemented checkout
slices from remaining consumer/archive qualification. The historical accepted
F2 boundary and evidence below retain their original scope.

## Context

The owner wants to investigate Yetlet as a Composable Application Framework:
one application can run without HTTP and can compose foreground, headless,
network and optional Web hosts. Historical HTML-first descriptions do not
restrict this research. Public positioning and published SDK/CLI contracts do
not change in this ADR.

Current Yetlet already separates owned View values, local commands, a local
reactive graph and opt-in adapters. The current `server_axum::App<S>` is a
thin wrapper around Axum Router, not a runtime-neutral application owner.
Its server feature is optional. Existing browser GPU and native offscreen
research do not provide a native game host.

The main problem is execution and resource ownership, not replacing Axum.
A foreground event loop cannot universally be started as a Send background
future. Multiple producers must not acquire independent authoritative game
states or bypass one shared domain state through unrestricted mutable access.

## Owner clarification — 2026-11-17

Preserve pure Web/API/SSR and existing View/HTML/DOM as first-class modes.
Separate Host identity, Renderer selection, state policy and execution topology.
A Host retains lifecycle without requiring its own thread/process. The amended
plan inserts YT-276 / B0 Web preservation before multi-host implementation.
This remains a Proposed design, not public API or measured performance evidence.

## Accepted scoped F2 decision — 2026-11-19

The project maintainer explicitly accepted the bounded F2 amendment in the
original WebCodex conversation on 2026-11-19 and authorized YT-282-B public
std-only core implementation, focused validation and independent review.
This satisfies the distinct owner-decision gate; it does not authorize a
commit, push, publication, version change or another adapter increment.

The accepted decision is the independently reviewed proposal at ADR SHA256
`cc98ae4d768af7f8b9848cdc4e9e8f5ed9549aed7ba5b1e5054d46dc1af3f8cd`
and promotion-packet SHA256
`108d2f19696df5b6bfb6b5e398d4fbf205be297d3b1682a6ca9f20a88e6ca6d3`.
Independent proposal Reviewer R1 `wc_job_b4AiE5FZQ86zwJB6` completed with
exit 0 and actual JSON PASS, `issues=[]`. The YT-281 illustrative signatures
remain design evidence, not a frozen public API.

Accepted scope: default-off `yetlet::composition` through the additive
`composition` feature; statically typed application-owned resources and
custom Host callbacks; caller-local `Rc`, borrowed resources and foreground
execution; required readiness, first-failure retention, admission-before-domain
stop, separate shutdown signal, and truthful original-worker Join/Pending.
No generic public Host/Transport/Renderer trait, dynamic registry, DI,
mandatory scheduler or automatic per-Host thread/process is adopted.

Separate optional `yetlet-hosts` adapter directions remain later increments.
The present implementation authority is **YT-282-B std-only core only**.
Real Web/UDP/Native/GPU adapters, F1 migration, external YT-283 consumers,
YT-284 package qualification, F3 and physical D4 are deferred. Existing
default HTML, direct Axum/Tower APIs, browser separation and CLI stay intact.
Physical D4 remains **Not verified**; historical Docs FAIL18 and current
FAIL19 evidence remain retained. F2 overall is still open.

The historical proposal below records the pre-adoption reasoning and gates.
Its then-pending wording is superseded only by this bounded decision.

## 2026-11-19 — F2 scoped promotion amendment candidate (decision pending)

**Status: Proposed only; this section is not an accepted/public API decision.**
The earlier B–D instruction not to publish a generic Application/Host/HostGroup
remains applicable to those private proofs. After the bounded [F1 three-host
acceptance](../M28-MULTI-HOST-DELIVERY-PLAN.md) and [YT-281 source-grounded
contract](../M28-F2-PUBLIC-COMPOSITION-CONTRACT.md), F2 has one newly justified
coordination responsibility: an *opt-in, application-owned, statically typed*
composition of resources with required readiness, explicit admission-first stop,
rollback, and truthful owned completion. This is an additive **candidate
amendment** to the historical POC direction, not a universal `Host` trait.

The [promotion decision packet](https://github.com/bohewu/yetlet/blob/5d57fce472f4ec01ce9bf17b3a387a43d3e38a5e/docs/M28-F2-ADR-PROMOTION-BOUNDARY.md) records the
exact pending maintainer choice and acceptance gates. The [YT-282-A private
test-only proof](../../examples/composable-host-research/src/composition_f2a.rs)
compiles a minimal static-chain ownership algorithm in an unpublished research
crate (12 focused tests); it is not an implemented public `yetlet` module or
real native/Web adapter.

### Proposed public scope, contingent on owner acceptance

- **Core opt-in:** add an off-by-default `composition` feature under the
  existing `yetlet` package, exposing `yetlet::composition`. The core remains
  standard-library-only, with no compulsory Axum/Tokio/winit/wgpu/UDP, new
  scheduler, app-wide `Send`/`Sync` bound, or HTML-centric framework assumption.
  The existing default `html`, browser separation, and direct
  `server_axum::App`/`Router`/`serve`/`serve_on` stay unchanged.
- **Optional native adapters:** one candidate companion `yetlet-hosts` with
  no default features and independently selected `native-winit`,
  `renderer-wgpu`, `udp`, and `web-axum` features. Its name, package layout and
  individual features are *proposed*, not present in Cargo or promised as
  released products. `server-axum` integration reuses native Axum/Tower.
- **Typed attachments, no universal Host Trait:** proposed local, fallibly
  prepared, owned-worker, required foreground, and borrowed-part operations;
  statically preserved concrete types and caller callbacks for stop/join. A
  user-defined Host supplies the same typed attachment operations with real
  behavior and cleanup rather than registering into a global trait object,
  service locator, DI container or runtime registry.
- **Resource lifetime:** the caller retains winit foreground execution and
  any borrowed Tokio/runtime resources. The application owns its domain and
  selects any single-authority policy, including the F1 one-C1-owner case.
  Only actual cross-thread adapter messages/publications require appropriate
  `Send`/`Sync`; local `Rc` and mutable borrows remain caller-thread-local.
- **Required lifecycle invariant:** actual readiness (not worker creation);
  on failure, preserve the first cause, close previously owned resources and
  separately report secondary cleanup issues. Stop new Web command admission
  **before** stopping its C1 authority; if admission closure fails, keep C1 and
  owned workers alive and return Pending until safe retry. A distinct control
  signal stops/drains HTTP, not the bounded data queue. Stop/cleanup/signal
  outcomes and original JoinHandle ownership remain observable; timeouts
  return `Pending`, not successful completion or detached workers.
- **Domain/transport separation:** byte-oriented UDP and Native renderer
  lifecycle may become optional adapters, but F1 Twin Squares, `YTP1`
  codec/session policy, player permissions, game Snapshot and fixed tick
  remain consumer-specific. This decision does not introduce a general
  Transport/Renderer trait, reliable/authenticated UDP, physical LAN proof
  or a second authoritative Simulation.

### Decision and implementation gates

The maintainer must **explicitly accept or reject this amendment** before
mutating the published `yetlet` API/feature graph or adding `yetlet-hosts`.
Design review, a subsequent "continue" instruction, or a private proof alone
does not convey that approval. If accepted, update this ADR's status through
a distinct, recorded maintainer decision; acceptance chooses *direction*, not
automatic source/registry publication or version promotion.

Implement F2 in bounded increments **after** that decision: (1) compile and
test the small public std-only typed core with consumer-driven signatures;
(2) real owned Web bridge/drain; (3) UDP byte/socket adapter separated from
game/session policy; (4) caller-driven Native/Winit and optional wgpu;
(5) migrate the actual F1 three-way application while retaining the sole
authority. Each changed adapter gets behavior-specific focused tests and
independent review. An actual external custom Host and local `Rc` consumer
must compile via only public package APIs in YT-283; exact package/dependency
and compatibility tests belong to YT-284. F2 is **not Done** until all its
public consumer/feature/lifecycle gates are satisfied. Physical D4 remains
**Not verified** independently of F2.

**Not yet frozen:** the exact Rust generic/receiver shapes in the YT-281
illustrative examples. YT-282's first compiling public consumer must resolve
them without introducing a blanket Send bound or hiding an unfinished
original worker behind `Drop`. Reject a candidate if actual consumer code
cannot preserve those invariants with reasonable static types.

## Proposed direction

Use **ordinary Rust composition with application-owned ports** for the POC.
Do not yet publish a generic Application, Host, HostGroup, Transport or Renderer
trait. Name concrete internal components by their actual responsibility.
A concept can have a documented contract without requiring a new public type.

Keep these boundaries distinct:

| Concept | Responsibility | Boundary |
|---|---|---|
| Application | Domain state, commands, invariants and resource ownership | No compulsory HTTP, socket, window, GPU or async runtime |
| Host | Selected execution environment and external event source | Knows required scheduling/lifetime; talks to application ports |
| HostGroup | Coordination of required components | Initially ordinary composition-root code, not a registry |
| Renderer | Converts appropriate render data into output | Does not own domain mutation or transport |
| Transport | Byte movement and IO/endpoint results | Does not define player permissions, commands or simulation |
| Protocol | Message version, structure and semantics | Bounded codec/validation independent of socket and renderer |

This extends the separation already explored by ADRs 0001/0003/0005. It does not
supersede the accepted server-specific contracts in ADRs 0011/0012.

## Minimum application and host contract

General application composition needs explicit construction, resource ownership
and host lifetime coordination. It does not mandate a concrete Application type,
command queue, snapshot publisher, actor or simulation clock for every workload.
Hosts can invoke typed application services directly when that fits the domain.

The B–D shared-state POC additionally selects bounded Submit/Observe ports:
validated typed command admission with Full/Closed/Invalid/Stale outcomes,
and immutable generation/revision/tick snapshots. Queued admission is not
completed execution. These are private POC semantics, not compulsory HTTP
middleware or new public Rust signatures.

Use concrete structs/functions and generics first. Existing Command and
ActionExecutor<C> remain usable without adding transport serialization or
universal Send bounds. A Host trait or HostGroup becomes a candidate only when
real consumers justify extraction and foreground/background ownership remains
explicit. Dynamic host registration, a DI/container and process-per-host
execution are not required. Peer reconnect is internal to one network host.

## State and execution ownership

The shared game/simulation POC selects one writer of authoritative GameState.
Its local/network inputs and any explicitly connected Web commands use bounded
validated ingress; render/network consumers receive immutable snapshots.
This is not a rule that all Yetlet domains or HTTP requests use one global actor.
Ordinary Web services retain their existing concurrent handler/state design.

Host is not Thread, Task or Process. Multiple hosts normally compose inside one
application process. Two caller-driven hosts can share one thread; several hosts
can share an explicitly owned async runtime, and one host may own multiple tasks.
An explicitly selected simulation/socket worker is a POC topology choice, not a
framework allocation rule. Only cross-thread values acquire applicable Send/Sync
bounds; caller-thread Rc-owned values remain valid.

For C/D, main-thread winit callbacks, a simulation worker and a network worker
remain one possible topology. Do not assign a new event loop to every Host.
Borrowed runtime ownership stays with the caller. Across process boundaries,
ordinary Rust references/Arc are not shared state: use explicit messages and
preserve one authoritative simulation plus client replicas. D's two processes
are two composed application instances, not one process per Host. A generic
IPC/process supervisor and a new BrowserHost implementation are deferred.

Short-lived locking around Arc<Snapshot> publication is acceptable for the POC,
not unrestricted mutable domain access. Clone the handle and release the lock
before rendering/serialization; payloads have no mutable GameState alias.
Rc/RefCell LocalGraph remains a local reactive primitive.
## Lifecycle and failure

Every Host has a logical lifecycle whether it uses a caller loop, shared tasks,
its own worker or a selected process. The concrete host owns its resources;
the composition root coordinates readiness, required/optional failure and stop.
Prepared, Starting, Running, Stopping and Stopped describe normal progress.
Failure retains its cause and separate cleanup pending/complete/timeout status:
a Failed label alone is not proof that tasks or threads have ended.

- Construct/validate first; shared-state POC ingress stays closed until required components are ready.
- Callback-driven hosts report readiness from their own loop. Do not block before entering the required foreground loop.
- Required failure or foreground exit requests group stop, closes admission and invalidates late work.
- Cancel/wake owned work and dispose callbacks/listeners/subscriptions. Wait/join only actual owned tasks/workers, not a nonexistent per-host thread.
- Release resources in dependency order. A host borrowing a runtime/scheduler never shuts it down or cancels unrelated work; only its actual owner coordinates final shutdown.
- Stop cannot depend on an already-full data queue. Completion timeout remains failed/incomplete cleanup; detach is not success.

Single-Web lifecycle reuses current native Axum/Tokio control, without adding a
supervisor or new serve() lifecycle methods. Pause/Suspend/Resume are optional
capability semantics, not required for every Host. Required POC hosts fail the
group by default; a peer timeout removes only that peer. Dynamic host membership,
automatic restart and generic process supervision remain out of scope.
## Simulation and render scheduling

Simulation consumes a stable ordered batch at a fixed tick. Input sampling and
render frames are independent. Headless tests call the same reducer with explicit
ticks and input; they require no display, GPU or socket.

The initial game is a small original two-square arena with integer movement,
bounds and Pause/Resume. Runtime scheduling uses monotonic time and bounded
catch-up; discarded timing debt is observable. Pause does not stop transport
freshness timers. Renderers consume the latest snapshot and do not advance
simulation.

The POC does not introduce prediction, rollback, interpolation, ECS, physics
engine or a general scene graph. Source-bound deterministic tests do not promise
cross-device floating-point determinism.

## Network composition

Prefer NetworkHost plus Transport plus Protocol. RPC may later use these
facilities at a higher layer but is not mandatory.

Start with Local/InMemory and then a bounded native UDP adapter. TCP streams,
UDP datagrams, QUIC streams/datagrams and WebSocket messages have materially
different semantics; a public universal Transport trait is premature.

D uses server-issued peer/session bindings, an epoch, protocol version,
sequence and tick metadata, finite packet sizes, bounded input/snapshot storage,
rate limits and timeouts. Complete held-input messages and complete small
snapshots tolerate loss without a new reliable-UDP layer. Fresh sessions fence
old input, Leave, Welcome and snapshots after reconnect. Decode and session
admission precede the owner's domain validation.

Default binding is loopback. Physical LAN mode explicitly selects an interface.
No discovery broadcast, wildcard fallback, firewall change, Internet support or
raw browser UDP is implied. Endpoint/session tokens provide routing and
freshness checks, not cryptographic authentication on an untrusted LAN.
The implementation plan defines the initial test budgets and exact acceptance
matrix; numeric choices remain proposed until fixed in their implementation slice.

## Renderer boundary

Host, Renderer and execution model are orthogonal composition dimensions with
capability constraints. View is an output model; browser is an environment.
A host may select zero, one or multiple compatible renderers, not a fixed backend
chosen solely from its Host type. Do not create a universal Renderer trait.

Application data can project to existing View (HTML serialization or browser
DOM), game SceneData (native or browser GPU where implemented), or an API response
(no UI renderer). A browser may combine DOM UI and a GPU canvas; a native window
does not provide DOM without a separately selected browser/WebView environment.
HTML serialization does not require Axum or a listener. Headless needs no GPU,
window or mandatory NullRenderer. These combinations describe capability
boundaries, not newly verified support for all combinations.

Existing View and Actions are preserved; mesh/camera/materials do not become
HTML nodes. Stateless HTML rendering needs no artificial lifecycle loop.
DOM owners dispose listeners/subscriptions; GPU backends own device/pipeline
resources and coordinate surface lifetime with the window owner. GameHost
coordinates foreground/window callbacks but does not write simulation through
the renderer. Actual GPU/lifecycle proof remains C, not compilation.
## Alternatives and consequences

| Choice | Advantage | Cost | Disposition |
|---|---|---|---|
| Concrete composition and typed ports | Smallest starting surface; thread and resource owners remain visible | Some repeated startup/stop code in examples; channel and snapshot costs remain | Recommended for POC |
| Static generic lifecycle adapters | Can consolidate demonstrated repetition without a registry | More bounds/types; foreground and background need distinct contracts | Reconsider at E |
| Dynamic HostGroup and boxed hosts | Runtime registration and replacement | Object-safety/future boxing, registry policy, lifecycle state and executor assumptions | Defer until a concrete requirement exists |
| Keep only Axum App as the application model | No new work | Cannot represent an application without HTTP ownership | Retain for Web use, insufficient as the general research model |
| Expose shared mutable GameState to all hosts | Minimal initial synchronization code | No enforced command order, validation or authority; IO can stall simulation | Do not use as the multi-host contract |

The proposed actor/ports boundary adds queue admission, snapshot publication and
thread coordination costs. It avoids claiming zero-cost abstraction or unmeasured
performance improvement. The two-player bounds make these costs observable before
any public commitment.

## Compatibility and migration

Pure Web/API/SSR is a first-class mode, not a special case of a game runtime.
Existing server_axum::App, prelude, native Router/Tower, state, handlers,
middleware, responses, serve/serve_on and into_router remain source-compatible.
The caller retains existing listener/runtime/graceful-shutdown ownership.
Do not require a new Application wrapper, HostGroup or changed Web CLI template.

Direct HTTP requests do not automatically enter a global actor, bounded queue,
fixed tick or Snapshot pipeline. Only routes explicitly operating the shared
simulation select its controlled boundary. API responses need no renderer;
SSR retains View-to-HTML. Current server-axum enabling html remains unchanged:
no renderer invocation is not a claim of no HTML compile-time feature.

The principle is no mandatory composition overhead: no new compulsory worker,
process, scheduler, lock, allocation layer or GPU/network-host dependency on the
unchanged Web path. This is a design and test constraint, not a zero-cost or
performance claim. B0 records a focused baseline using existing Web tests and
normal graphs; later affected paths require comparison, not broad requalification.

B–D remain private opt-in examples. Base/default features stay free of compulsory
Axum/Tokio/winit/wgpu; existing native/browser gates and the public Web surface
remain. A new crate/module/game template or public positioning needs E and
separate implementation authority.
## Acceptance and rejection criteria

- A accepts documentation only after independent review and document checks; each substantive amendment needs its own delta review.
- B0 / YT-276 records the existing pure API/SSR, source API and optional-feature baseline before B1. It creates no mandatory host wrapper and is not a performance certification.
- B1 proves two hosts with distinct lifecycles on the same caller thread, one shared owner and no listener/GPU/Tokio requirement. B2 adds an explicitly owned worker and scoped failure/stop checks without assuming ownership of a shared scheduler.
- C proves the same simulation headlessly and in a real native window with actual GPU/backend evidence.
- D proves two application processes with native UDP, validated inputs, bounded faults and safe sessions/stop; it does not require one process per Host.
- Two-process loopback cannot qualify two physical LAN devices.
- E may promote, simplify, retain private or reject. Pure-Web preservation and capability-compatible render selection are mandatory before promotion; keep this ADR Proposed until owner acceptance.
- Hidden runtimes, compulsory game machinery on HTTP paths, unsafe ownership or unjustified compatibility changes are reasons to simplify or retain private composition.

The [M28 plan](https://github.com/bohewu/yetlet/blob/5d57fce472f4ec01ce9bf17b3a387a43d3e38a5e/docs/M28-COMPOSABLE-APPLICATION-PLAN.md) contains the phased backlog
and bounded evidence requirements. This ADR neither executes a runtime POC nor
establishes measured zero overhead.

## Research references

Read on 2026-11-17. Upstream documentation is evidence for constraints, not proof
of repository support or a dependency upgrade.

- [winit 0.30.13 EventLoopBuilder](https://docs.rs/winit/0.30.13/winit/event_loop/struct.EventLoopBuilder.html): main-thread/event-loop ownership.
- [winit ApplicationHandler](https://docs.rs/winit/latest/winit/application/trait.ApplicationHandler.html): foreground lifecycle and redraw responsibilities.
- [wgpu 30.0.2 Surface](https://docs.rs/wgpu/30.0.2/wgpu/struct.Surface.html): research reference for surface lifecycle; existing Yetlet examples use 27.0.1.
- [Tokio sync/async bridging](https://tokio.rs/tokio/topics/bridging): explicitly owned runtime integration.
- [RFC 8085](https://www.rfc-editor.org/rfc/rfc8085.html): UDP usage, bounded applicability, reliability and packet-size considerations.
- [Rust UdpSocket](https://doc.rust-lang.org/std/net/struct.UdpSocket.html): native socket semantics and receive limits.
- [RFC 1982](https://www.rfc-editor.org/rfc/rfc1982.html): serial arithmetic considerations if wrapping counters are later selected.
- [RFC 9221](https://www.rfc-editor.org/rfc/rfc9221.html): QUIC unreliable datagrams are distinct from reliable streams.
- [WebSockets Standard](https://websockets.spec.whatwg.org/): browser message API, not raw UDP access.
