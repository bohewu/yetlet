# ADR 0013 — Composable Application and Explicit Host Ownership

- **Status:** Proposed.
- **Date:** 2026-11-17.
- **Tracking:** YT-265, M28-A.
- **Decision owner:** Project maintainers after M28-B through M28-E evidence.
- **Related:** [M28 plan and source inventory](../M28-COMPOSABLE-APPLICATION-PLAN.md), [ADR 0001](0001-separation-of-concerns.md), [ADR 0003](0003-renderers-and-loading.md), [ADR 0005](0005-opt-in-capability-modules.md), [ADR 0011](0011-application-startup-api.md), [ADR 0012](0012-server-axum-prelude.md).

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

The [M28 plan](../M28-COMPOSABLE-APPLICATION-PLAN.md) contains the phased backlog
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
