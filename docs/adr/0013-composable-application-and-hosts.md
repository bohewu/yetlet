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

The POC needs five observable capabilities:

1. **Construct:** create explicit state/resources and bounded ports without automatically starting IO.
2. **Submit:** admit a typed command or return a clear Full, Closed, Invalid or Stale outcome.
3. **Observe:** obtain an immutable owned snapshot with its generation/revision and, for simulation, tick.
4. **Coordinate:** report ready/failed/exited, request stop through a non-starvable control path, and observe terminal cleanup.
5. **Own:** keep each resource with one responsible owner until it is released or reported as still active.

These are semantic requirements, not final Rust signatures. Use concrete structs,
functions and generics before dynamic dispatch. Existing `Command` and
`ActionExecutor<C>` can supply a typed command boundary when appropriate.
No transport serialization or universal Send requirement is added to local
commands. Queued admission never masquerades as completed execution.

A public Host trait becomes a candidate only after B–D show a genuinely shared
contract. If an interface is later extracted, it must represent foreground and
background ownership honestly instead of imposing a universal executor.
Dynamic host addition/removal and a DI/container/service registry are deferred.
Peer joins/reconnects remain inside one running network host.

## State and thread ownership

One simulation owner is the only writer of authoritative GameState. Local input,
network input and any later Web commands enter controlled bounded ingress.
Renderer and networking read immutable snapshots; neither receives mutable
GameState access.

The host process has one authority. A network client keeps a display replica,
not another authoritative simulation. Local UI state and a later predicted
display state are separate concepts.

For the native POC, the main thread owns winit/window callbacks, a dedicated
worker owns simulation, and an optional worker owns UDP. A caller may explicitly
own a Tokio runtime for Axum or another async adapter. Neither Tokio nor Send
bounds on every host/state become Core requirements.

A short-held mutex around an `Arc<Snapshot>` slot is acceptable for B–D.
It protects pointer publication, not permission for every host to mutate the
domain. Published payloads contain no shared mutable domain aliases. Rendering
and serialization happen after releasing the slot lock. The existing Rc/RefCell
LocalGraph remains a local reactive primitive, not the cross-thread actor.

## Lifecycle and failure

The composition root owns startup readiness, error propagation and shutdown.
All POC components are required unless an explicitly selected optional policy
has its own test.

- Validate and construct first; begin startup with domain ingress closed.
- Background components report readiness without taking foreground ownership.
- A window/GPU host reports readiness from the required event-loop callbacks; do not block the main thread waiting for callbacks before entering the loop.
- Open domain admission when all required components are ready.
- On required startup/runtime failure or foreground exit, close admission, invalidate the active generation and signal cancellation.
- Stop IO/publication, join owned workers, and release resources in their dependency order. Keep primary and cleanup errors.
- A full data queue cannot suppress shutdown. A timeout remains an incomplete/failed cleanup result, never evidence of a joined thread.

A peer timeout removes one peer. A NetworkHost failure ends the LAN group by
default; silently continuing as a standalone game would misrepresent the selected
mode. Optional observer degradation and hot-plug hosts are not initial features.

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

Application state, scene projection, backend, window/surface, UI and GPU resources
are separate. Existing HTML-like View remains useful for UI; it is not stretched
into a mesh/camera/material language. Reuse appropriate identity, commands,
subscriptions and lifecycle ideas only where their contracts fit.

GameHost owns the foreground/window lifecycle. WgpuRenderer owns rendering
resources and coordinates surface lifetime with that window. Headless execution
constructs neither. A no-rendering path does not require a public NullRenderer.
GPU device/surface errors and zero-size windows have explicit behavior and
bounded verification; compiling is not a visual test.

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

No change to existing `server_axum::App`, its state bounds, routes, prelude,
default address, serve methods or `into_router()` occurs in A. No second HTTP
router is introduced. A future Axum composition example should use caller-owned
listener/runtime and native graceful shutdown around the current Router.

B–D start in private research examples with explicit features and locks. Core
must not gain compulsory Axum, Tokio, winit or wgpu dependencies; browser/native
target gates remain explicit. A public crate/module, game template or package
description change requires the E decision and separate implementation scope.

## Acceptance and rejection criteria

- A passes only at documentation scope after independent review and required document checks.
- B must prove two hosts, one owner, no listener requirement, readiness/failure/stop and compatibility.
- C must prove the same simulation headlessly and in a real native window with actual GPU/backend evidence.
- D must prove separate-process native UDP, validated inputs, bounded faults and safe session/stop behavior.
- Two-process loopback cannot qualify two physical LAN devices.
- E may promote, simplify, retain private or reject the approach. Keep this ADR Proposed until maintainers accept the evidence-based outcome.
- If shared abstractions require extensive conditionals, hidden runtimes, unsafe ownership, or compatibility changes without a demonstrated benefit, prefer retaining application-owned composition.

The detailed phased backlog, costs, source evidence and per-stage gates are in
the [M28 plan](../M28-COMPOSABLE-APPLICATION-PLAN.md). No runtime POC has been run
by this ADR.

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
