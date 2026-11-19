# M28 follow-through - Multi-Host Delivery Plan

- **Date:** 2026-11-19 (Asia/Taipei).
- **Status:** Planned work, not an implementation or qualification claim.
- **Baseline:** `b91313dc5b2233125074f52f2bdbde6925781d8a` on `docs/m28-composable-architecture`.
- **Tracking:** [Follow-through backlog](BACKLOG.md#m28-follow-through--multi-host-sdk-delivery); [roadmap](ROADMAP.md#m28--composable-application-and-multi-host-research).
- **This change:** Planning documents and an ignored handoff only. Product source, SDK/CLI, Cargo features, ADR status and historical evidence are unchanged. No Stage, Commit or Push is authorized by this plan.

## 1. Delivery objective and F0 boundary

The owner reaffirmed that the objective is usable multi-host application development:
GameHost, NetworkHost and native Web integration in one process; an opt-in public
composition API that also admits compatible user-defined Hosts; and actual
application communication between two physical computers.

A private integration demonstration is an intermediate deliverable. The SDK goal
is incomplete while users must copy private research implementations or their
startup, failure and shutdown machinery.

[F0-C](M28-F0C-TWO-CONSUMER-DECISION.md) remains valid: the reviewed Headless and
Pure Web roots do not contain a common ownership-sensitive operation that
justifies extracting a shared helper or generic lifecycle trait. That bounded
finding does not close the multi-host SDK objective. Keep those Concrete Roots.
Do not start F0-B3 or implement an abstraction merely to standardize names.
Use the new integrated consumer to establish actual operations, then deliver the
smallest useful public composition contract. A universal Host trait is a design
option requiring evidence, not the definition of success.

This continuation records the owner's clarified delivery direction. It does not
retroactively accept every recommendation in the [M28-E proposal](M28-E-PROMOTION-DECISION.md),
rewrite reviewed evidence, accept [ADR 0013](adr/0013-composable-application-and-hosts.md),
or authorize runtime implementation in this planning turn.

## 2. What exists and what remains

| Surface | Current grounded status | Missing delivery |
|---|---|---|
| Web | Public [App and native Router integration](../crates/yetlet/src/server_axum/app.rs); the private [F0-B2 root](../examples/composition-web-research/src/lib.rs) configures a Router and leaves serving/drain ownership to its caller | No public type named WebHost or public multi-host application coordinator |
| Game and UDP | Private [D2 native application](../examples/composable-host-research/native/src/bin/d2-loopback.rs) already combines foreground rendering and [NetworkHost](../examples/composable-host-research/native/src/udp_d2.rs) in one application, with a separate client application | No SDK GameHost/NetworkHost and no three-way Web integration |
| Local/headless composition | Private [F0-B1 concrete owner](../examples/composable-host-research/src/headless_composition_f0b1.rs), plus earlier B/C/D evidence | No reusable public HeadlessHost or compatible custom-Host composition API |
| Physical LAN | [D4 / YT-274](M28-E-PROMOTION-DECISION.md#d4-physical-device-gate-versus-supplementary-virtual-network-evidence) remains Not verified after blocked application UDP ingress | Native selected-NIC operation and real two-physical-computer application evidence |
| Virtual supplement | D4VM crossed two VM kernels through test-only relays on one physical host | Does not establish physical D4, native non-loopback bind or Windows qualification |

These are source- and environment-scoped historical facts. No old tests,
reviewers or HTTP/GPU/UDP E2E runs are repeated by this planning change.

## 3. Milestones and order

| Milestone | Work items | Deliverable | Completion boundary |
|---|---|---|---|
| M28-F1 | YT-277 through YT-280 | One real, application-owned Game + UDP + native Web integration in one OS process | Private integration only; public SDK is still owed |
| M28-F2 | YT-281 through YT-284 | Opt-in SDK capabilities, public composition and extension contract, external consumers and usage documentation | Public source/package consumption proven; hosted publication is separate |
| M28-F3 / existing D4 | YT-285 and existing YT-274 | Selected native endpoints and qualified two-physical-computer application communication | D4 closes only on real physical application evidence |

F1 precedes final public API design. F2 is a planned delivery obligation, not an
optional question about whether to continue after another demo. The physical
network preparation can progress independently when authorized and available;
a blocked campus network does not block F1 or F2. Any LAN capability claim remains
blocked until its matching physical evidence exists.

All new work items start at **Todo**. No implementation, new Reviewer PASS or
runtime completion is asserted by assigning IDs.

## 4. F1 - Same-process integration

### YT-277 - Concrete ownership and execution wiring

Create one bounded integration entry point using the existing native, UDP and
Web capabilities. Prefer a small addition under the private research workspaces;
leave old baseline consumers intact unless a narrowly identified seam must change.
Freeze the exact touched-file allowlist before implementation.

Use this initial topology unless a concrete implementation constraint requires
an explicitly recorded revision:

- The main thread owns winit/window integration, the authoritative simulation
  and bounded nonblocking UDP polling. Rendering consumes that same owner's
  immutable publication.
- One application-owned service thread runs one current-thread Tokio runtime
  for Axum. The composition root owns its startup result, stop capability and
  completion/join handle. This is one chosen integration topology, not one
  thread per Host or a mandatory SDK worker.
- Native caller-local Rc values remain local. Only a bounded command port,
  immutable sendable snapshot data, lifecycle notifications and completion
  results cross to the Web side.
- Keep explicit handles for every owned task, thread, socket and window.
  No process-per-Host launcher, detached worker or hidden global runtime.

The existing D2 NetworkHost contains a HostFixture authority, while the standalone
native GameDriver contains a Simulation authority. Composing both constructors
unchanged would risk two independent authorities. Select exactly one owner
for this integration, record its actual ownership path and route every producer
to it. Do not duplicate a fixture to make HTTP responses look correct.

**Done when:** a concrete root starts the three components in one process with
one authority, a written resource/handle map, observable readiness and a working
normal stop path. No generic helper/Host trait is needed for this item.

### YT-278 - Shared domain data and real Web operation

Add a small native Web surface to the integration, using existing App/Router
APIs. Proposed integration routes are a read-only status/snapshot endpoint and
one bounded typed domain command endpoint. They are application-specific test
routes, not new Yetlet SDK or CLI API promises.

Choose one existing domain operation, such as Pause/Resume, for HTTP control.
Native input and real UDP input must still reach the same authority. A read-only
snapshot contains generation/revision/tick and the state needed to compare the
native scene, HTTP response and UDP replica.

- HTTP command acceptance means admitted, not applied. A later authoritative
  publication must demonstrate application of the command.
- Keep queue capacity, per-wake work budget, full/closed handling and stale
  publication policy explicit. Stop uses an independent control path.
- Snapshot conversion must not send Rc or mutable game-state aliases across
  threads. Do not force the whole application into Arc<Mutex<GameState>>.
- The Web endpoint must stay responsive while native ticks/rendering and UDP
  polling continue. Do not block the foreground loop waiting for HTTP.
- A future publication may be newer than an earlier observer read. Compare an
  identified publication or a controlled quiescent checkpoint; do not claim
  simultaneous equality from unrelated latest reads.

**Done when:** real HTTP observes the actual game state and one HTTP domain
command produces an authoritative change visible to the native consumer and
UDP replica; native/UDP inputs are likewise visible through HTTP.

### YT-279 - Required-component lifecycle and failure behavior

Treat all three selected components as required for this first integration.
Readiness requires actual native initialization, UDP binding and HTTP serving
readiness, not only task creation. Record startup order and partial-start rollback.

Required startup/runtime failure and native close request coordinated stop:
close admission, retain the primary failure, stop owned producers, drain selected
HTTP work, release sockets/window resources and observe actual task/thread
completion. Separate stop requested, failed, cleanup pending and completed.
A timeout retains ownership and a resumable incomplete result; detach is not
successful cleanup.

Add focused cases for the new risks:

1. HTTP bind conflict after another owned component has been acquired.
2. Required HTTP task/worker failure reaching the foreground root.
3. A full data queue that cannot prevent stop.
4. An in-flight HTTP request during native-close shutdown, with explicit drain
   behavior and observed completion.
5. A bounded completion timeout preserving the original failure and handle.

Use deterministic focused checks for failure paths; do not launch a new visible
native window for every negative case.

**Done when:** normal exit and selected failures have concrete cleanup evidence,
correct cause retention and no untracked application-owned execution.

### YT-280 - New integration acceptance and independent review

Run one bounded real integration attempt for the new source after focused fixes:

- Record the source identity and one application PID owning the Game, UDP and
  HTTP integration. A separate UDP client is an external peer, not a substitute
  for co-locating the three server components.
- Use the actual native window, real loopback UDP and real HTTP. Capture a
  coherent authoritative publication shared by the scene, HTTP and replica.
- Demonstrate progress while HTTP is in use and observe the new command path.
- Close the owned native application, observe graceful HTTP/task completion,
  and verify selected socket/port release and application-owned thread exit.
- Run only tests, formatting/lint checks and dependency observations needed
  for the new integration and genuinely changed seams.
- Obtain independent Reviewer actual JSON PASS for this new integration scope.
  Keep the same dispatched Job through terminal exit and preserve failures.

Source-unchanged F0, C2/C3, D2/D3 and B0 evidence is reused. The new three-way
E2E is necessary because its source and behavior did not previously exist; it
is not a rerun of the old standalone F0-B2 HTTP test. If review changes an
observed behavior, repeat only the affected proof and label other evidence by
its actual source. No blanket replay is required.

**F1 acceptance gate:** YT-277 through YT-280 all satisfy their criteria. Mere
coexistence, three fixtures with matching constants, a shutdown signal without
completion, or a review of a different snapshot cannot pass. This closes F1
only, not the SDK or physical LAN objective.

## 5. F2 - SDK and public composition delivery

### YT-281 - Minimal public contract from actual consumers

Use the F1 call sites to specify a consistent public composition entry point,
typed attachment/extension points and observable startup/failure/stop/completion.
Make foreground affinity, caller-local values, application-owned work and borrowed
runtime ownership expressible without blanket Send/Sync requirements.

Compare a small concrete adapter API with any justified static typed contract.
Do not reopen an unbounded framework study. Every proposed public operation
must be exercised by the integrated consumer or the custom consumer below.
No requirement exists for dynamic registration, a universal async Host trait,
DI, boxed futures, hotplug or a new scheduler.

Select exact package/module/opt-in feature names in this item; none are invented
as existing APIs by this plan. Resolve any required ADR amendment explicitly
before API implementation. ADR0013 stays Proposed until separately accepted;
planning approval does not silently accept it.

**Done when:** a bounded public contract, ownership/type rules, migration map and
consumer-shaped usage are reviewable; the implementation scope is explicit.

### YT-282 - Implement reusable composition and opt-in adapters

Deliver the selected public composition operations and usable non-Web Host
capabilities. Implement in reviewable increments for the concrete Game/native,
Network and Web integration boundaries.

- Game/native support owns execution/window lifecycle and connects an explicitly
  selected renderer and application data. Twin Squares rules and fixtures stay
  in consumer code.
- Network support owns the selected transport/lifecycle and typed application
  boundary. Separate byte transport from the example's two-player permissions,
  codec/session policy and game-specific commands. Do not promise a general
  reliable or authenticated network protocol.
- Web composition adapts the existing App/Router serving path when requested.
  Pure Web callers keep their direct native path without composition setup.
- Composition coordinates real owned resources and completion. Compatible
  custom adapters use the same documented public extension boundary.
- Optional dependencies remain opt-in. Pure Web gains no mandatory winit, GPU,
  game tick, UDP Host or shared simulation. Headless/local-only usage need not
  construct a window, socket or Tokio runtime.

**Done when:** SDK consumers can use the promised capabilities through public
package APIs, without importing research modules or copying orchestration.
Renaming or moving the entire research example into a public crate is insufficient.

### YT-283 - External consumers and custom Host proof

Create an independent consumer crate using only the candidate's public package
surface. Migrate the F1 three-way application to that surface. Add a small custom
Host from outside Yetlet, such as a caller-driven finite input/status component,
and attach it without changing Yetlet source or adding a built-in registration
branch. Give it an actual application operation and lifecycle, not a no-op marker.

Include a caller-local Rc consumer. Send/Sync bounds may appear only at its real
thread/async adapter boundary. An external consumer must not use private modules,
include! of repository sources, a research-crate dependency or copied supervisor
code to simulate the promised API.

**Done when:** Game + Network + native Web and an independently defined compatible
Host compose via public APIs with observable state and lifecycle behavior.
Document restrictions for incompatible foreground schedulers instead of promising
that every arbitrary runtime can be combined without an adapter.

### YT-284 - Public consumer, compatibility and documentation gate

Check the exact selected candidate package contents with independent consumers;
distinguish local source/package proof from registry publication. Provide concise
working guidance for direct Web, non-Web local/headless, three-way composition and
custom extension, including failure/completion semantics and opt-in dependencies.

Verify the affected feature combinations and normal dependency graph: existing
pure Web does not acquire native-game/GPU dependencies; non-Web composition does
not require HTTP/Axum; caller-local use does not acquire a global Send bound.
Select platform verification by changed behavior. Reuse exact-source older
evidence only within its scope; do not impose unrelated Windows/browser reruns.

Obtain independent review of the new public API, consumer evidence and truthful
documentation. Product publication, CLI templates, release selection, tags,
signing, push and hosted release remain separately authorized actions.

**F2 acceptance gate:** usable opt-in SDK capabilities, external three-way consumer,
external custom Host, caller-local evidence, lifecycle evidence and documented
compatibility all pass. An example-only result does not close F2.

## 6. F3 - Existing physical D4, without evidence substitution

### YT-285 - Native endpoints and physical-path readiness

Select the exact application/transport source to qualify and provide explicitly
configured bind/peer endpoints suitable for the chosen physical machines. Current
D3 loopback plus a relay is not native non-loopback support. Preserve the safe
local default; bound peer acceptance and expose only the selected test route.

Inventory two real physical hosts, their OS/toolchains and selected interfaces.
Resolve permitted application UDP ingress before attempting the full application.
Diagnose an unavailable route once and retain its exact blocker. Firewall/ACL
changes require their own authorization; none follows from this plan.

**Done when:** source-bound native endpoint handling and a narrowly authorized
application UDP path between the two selected physical hosts are ready.
Unresolved ingress is a blocked gate, not permission to substitute VM evidence.

### Existing YT-274 - Physical application qualification

Continue the original D4 item. Record both physical host identities, exact source,
binaries/builds, selected endpoints and actual application processes. Demonstrate
real remote input, authoritative publication/replica agreement, the selected
disconnect/reconnect behavior and owned shutdown. If Windows is part of the
selected claim, obtain the necessary native Windows evidence for that source.

Physical acceptance remains **Not verified** until this succeeds. A VM relay,
ping, ARP, localhost packet, matching fixture or plan cannot satisfy it. If the
qualified source is an older research application, report only that scope; do
not promote it into a claim about an untested SDK transport.

## 7. Verification, authority and immediate next work

This turn adds documentation only. A scoped document check can establish valid
links, English text, backlog dependencies and whitespace; it cannot establish
a runtime or API PASS. The prior independent F0-C PASS applies only to the reviewed
F0-C document, not to this new plan.

The preserved repository-wide Docs FAIL18 originates in the ignored historical
F0-B2 handoff's Chinese prompt. Retain that handoff and the original failure.
Do not relabel the global docs gate PASS, repair the checker as incidental work,
or rerun unchanged old product tests/reviewers/HTTP E2E to clear bookkeeping.

**Next implementation milestone:** M28-F1 / YT-277 through YT-280, starting with
YT-277's exact ownership and touched-file boundary. Deliver the real integrated
consumer and its new evidence before moving to F2. The next request should
authorize that bounded implementation; this planning turn does not start it.

Until such a request, leave these tasks Todo, ADR0013 Proposed, physical D4
Not verified and the current SDK/CLI/features unchanged. No Stage, Commit or
Push; no M27-C edits; no old evidence edits. Preserve any new planning diff for
the next session instead of resetting it.
