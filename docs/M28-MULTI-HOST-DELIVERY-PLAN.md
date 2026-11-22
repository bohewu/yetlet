# M28 follow-through - Multi-Host Delivery Plan

- **Date:** 2026-11-19 (Asia/Taipei).
- **Status:** Original delivery plan with separately dated implementation and qualification checkpoints; see section 6 for YT-285 readiness.
- **Baseline:** `b91313dc5b2233125074f52f2bdbde6925781d8a` on `docs/m28-composable-architecture`.
- **Tracking:** [Follow-through backlog](BACKLOG.md#m28-follow-through--multi-host-sdk-delivery); [roadmap](ROADMAP.md#m28--composable-application-and-multi-host-research).
- **Original planning change:** Planning documents and an ignored handoff only. Product source, SDK/CLI, Cargo features, ADR status and historical evidence are unchanged. No Stage, Commit or Push is authorized by this plan.

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

This table records the planning baseline above. File links follow current source
locations; the historical status and evidence remain at their recorded scope.

| Surface | Current grounded status | Missing delivery |
|---|---|---|
| Web | Public [App and native Router integration](../crates/yetlet/src/web/app.rs); the private [F0-B2 root](../examples/composition-web-research/src/lib.rs) configures a Router and leaves serving/drain ownership to its caller | No public type named WebHost or public multi-host application coordinator |
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

**Maintainer scope clarification (2026-11-21):** Yetlet supplies a thin,
opt-in Native Window/EventLoop Host; a future Game Engine would be a separate
SDK. The consumer retains raw Winit Keyboard/Mouse/Focus/Resize/DeviceEvent,
input mapping, game loop, simulation, scenes and application network protocol.
A consumer with its own Window/EventLoop may compose Yetlet's other Hosts
without creating another Window. No new framework Input Manager, engine API,
YTP1/Replica/Reconnect or precise frame-pacing gate is required for YT-283.
YT-284 remains the independent package/compatibility gate.

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

#### YT-285 readiness checkpoint 2026-11-22

**Status: YT-285 Done at native endpoint and selected physical-path readiness scope.**
Configured independent Reviewer `wc_job_njZ3jGWe6WmFO-rC` completed with exit 0
and actual JSON PASS: `issues=[]`, `required_followups=[]`. The saved Reviewer
model was `gpt-6.2-astra`; observed effort was `high`. All 17 selected source
files and 47 evidence inputs remained unchanged during review. This was an
ordinary role-directed review, not formal assurance. Final local Git delivery
uses the scoped document check and exact-index Strict/Builtin Commit Guard.

This checkpoint selects the current public SDK UDP implementation and the original
direct campus path. It does not reuse the old D3 loopback/relay application.

The source baseline is `52b67fa75b88f8bacd9717ba750f5de0a7f0948c`.
Both builds consume the unchanged normal YT-294 Core archive
`yetlet-0.9.0-alpha.8.crate`, SHA-256
`ea267e20bd424aa1c7772140d2269655575f3b0743ac887fa49eabba4c60412b`,
whose recorded Git source is `19993131f02fa4477296542f6bbe0b1774a744b7`.
The intervening commit does not change the SDK, workspace manifest or lockfile.
The selected `src/udp.rs` hash is
`d7d3a04682aa50466bf7e7b7de3dd2e9609164775b197e7b02067274626a4d18`;
`src/udp/host.rs` is
`95f2287bc10efa86778d3c8bae406ce41b53e340c401e097f9083584706be5a7`.

| Selected physical host | Native environment | Explicit endpoint and route |
|---|---|---|
| `physical Linux authority`, ASUS physical host; virtualization detection returned `none` | Linux `7.0.0-44-generic`; Rust `1.99.0`, `x86_64-unknown-linux-gnu` | `<selected-endpoint>`, `enp4s0`; direct on-link route to Windows |
| `physical Windows replica`, ASUS EXPERTCENTER D900TA_M900TA physical host | Windows 11 Pro `10.0.26300`; Rust `1.99.0`, native `x86_64-pc-windows-msvc` | `<selected-endpoint>`, `vEthernet (External)` interface 20 on the physical host; on-link `<selected-gateway>`, next hop `0.0.0.0` |

The existing ZeroTier interfaces were inspected as an initial candidate only.
`SELECTED-PATH-R2.json` explicitly supersedes that candidate in `BASELINE.json`;
neither executed exchange used an overlay, VM or relay. No firewall, routing,
network-profile or ZeroTier setting was changed. Current success does not identify
the cause of the original 2026-11-18 zero-packet results.

**Evidence stages remain distinct:**

| Stage | Ubuntu valid received challenges / acknowledgements | Windows valid received challenges / acknowledgements | Completion |
|---|---|---|---|
| Native Python path preflight, 20 challenges sent per host | 17 / 18 | 18 / 17 | Both exit 0; both original sockets closed and selected endpoints rebound |
| Actual SDK consumer, 20 challenges sent per host | 9 / 10 | 10 / 9 | Both exit 0 after their full 20-second windows; both original endpoints closed and rebound |

The SDK consumer uses public `UdpEndpoint::bind_on`, `send_to`,
`poll_bounded` and `close`. Its shared source SHA-256 is
`8dc72eeaf9076d05e64352cfb87121ac975350f5d0e4655b9b8679057e9fbc98`.
It checks the exact peer tuple before parsing or replying, then validates the
selected run, opposite role and echoed per-process nonce. Its bounds are 256-byte
payload capacity, 16 packets per poll, 20 challenge attempts, 32 acknowledgement
attempts and one 20-second monotonic window. Four focused configuration/frame
fixtures pass; the no-argument invocation binds loopback on an ephemeral port,
sends no packets, closes and successfully rebinds. The SDK's loopback defaults and
caller-owned peer policy are unchanged.

Both native builds compile the std-only Core plus `udp` from that archive directly
with Rust 1.99. This is source-bound native compilation and execution evidence;
normal Cargo/archive consumption remains the separately qualified YT-294 scope.
The Linux binary SHA-256 is
`2a6fdb2185a91cffe5f3666a86c6919637066ac415df6f3b0de3b3dee09f6989`;
the Windows MSVC executable SHA-256 is
`100f57be56ac404f9b3a5ee46b3a5cbe5cb460843fb4a6bb364446736e5d72a3`.
The Windows tracked checkout remains unchanged at
`7fbf918a5a0734f7ed125afd6c860837b1de55d1`; the tested SDK comes from the
SHA-verified archive transfer, not that historical checkout.

SDK Jobs `wc_job_q6B9LvSTVt-z_y0Q` and `wc_job_Pg9TnA_7jgRawICN`
completed with exit 0, running native PIDs 1463055 and 49456 respectively.
Each recorded valid peer traffic, matching cross-peer nonces, no errors,
owned close and successful same-address rebind. Fresh post-run observations
found neither selected process nor selected UDP-port owner. The earlier native
path Jobs `wc_job_bOndpswRTs2zdD2M` and `wc_job_vTzNG2BF-mkAYNVl`
also completed with exit 0. The different receive totals reflect separate
observation windows and are not a packet-loss or reliability measurement.

Ignored evidence is rooted at `.pipeline-output/m28-yt285-20261122/`:
`SDK-PATH-R1.json` binds source/build identities, actual SDK receipts,
cross-peer nonce agreement and cleanup; `SELECTED-PATH-R2.json` binds the
direct-path choice; `UBUNTU-HOST-INVENTORY-R1.json` and
`WINDOWS-HOST-INVENTORY-R1.json` retain the host observations.
`ARTIFACT-TRANSFERS-R1.json` records verified transfers, and
`windows-evidence-r1/` contains the transferred native Windows receipts.
The finite consumer and local build/fixture receipts are under
`artifacts/verification/m28-yt285-20261122/sdk-consumer/`.
These are ordinary task artifacts, not a new supported test framework.

This establishes YT-285 native endpoint and selected physical-path readiness.
The transport remains a byte/socket facility with caller-owned trust and
application protocol policy. YT-274 remains **Not verified**: actual remote
application input, authoritative publication/replica agreement, selected
disconnect/reconnect and owned application shutdown still require their own
source-bound physical execution. The original failed D4 probes and successful
VM supplement retain their historical scopes.

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

#### YT-274 physical application qualification — 2026-11-22

**YT-274: Done at selected private headless physical application scope.** Configured independent Reviewer `wc_job_ZAtDIjA5lHjgElJ4` completed terminal exit 0 with actual JSON PASS (`issues=[]`, `required_followups=[]`); 22 selected sources and 28 evidence inputs remained byte-identical during review. Final local Git delivery uses scoped docs verification and exact-index Strict/Builtin Guard. This dated checkpoint supersedes the earlier *Not verified* status only within its precise application and physical-path scope; it does not retroactively change the 2026-11-18 failed probes or VM supplement.

The exact 17-file source snapshot is preserved as `artifacts/verification/m28-yt274-20261122/d4-physical-source-r1.zip` (SHA-256 `09fbfc2a75ba35c18a6681b0d9bc68dfff7a2f2882967d5f0b374fab37f7e741`). The associated manifest is `D4-SOURCE-MANIFEST-R1.json` (SHA-256 `36316297b208fcac60348472b2793f16b99da6ad4d4141664ca0de25dc76a9ec`). It includes the original one-C1-owner D3 game/protocol and new private `d4-physical` driver with explicit peer addressing. On both machines the source was built natively with Rust 1.99, Linux x86_64 and Windows x64 MSVC; the Windows tracked historical checkout remained unchanged.

| Role | Physical host / selected endpoint | Native executable SHA-256 | Real R4 process |
|---|---|---|---|
| Authority / Host | `physical Linux authority` physical ASUS, `<selected-endpoint>`, `enp4s0` | `8f261e3670bad21df23da69c8a697b399e7491bbb0b27fb41117ad8beb4cc272` | PID `1562193`, exit 0 |
| Replica / Client | `physical Windows replica` physical ASUS Windows 11, `<selected-endpoint>`, physical-host external vSwitch | `5a1bb165952744a02414f87f1a511b4dc82c4d8225d2be16070375b0868448d6` | PID `21876`, exit 0 |

**R4 real application acceptance:** the host accepted 25 actual remote inputs, observed initial westward movement, neutral input expiry after 250 ms, two 3-second peer timeouts, one newer Session/Epoch reconnect (`7101`/`1` → `7102`/`2`) and replacement eastward movement. The single authority retained Generation `1` and progressed to Revision/Tick `1937`. The remote client received 2 Welcome messages and 63 Replica Snapshot frames, observed westward movement, an updated neutral publication, reconnection, replacement movement and the new Still publication. Its final accepted Replica was Revision/Tick `1755` in Session `7102`, Epoch `2`; final values need not match the host's later stop-time revision.

**Raw wire convergence:** the R4 reconciliation compared each client's original canonical Snapshot frame against the host's actually submitted publication bytes: **63/63 exact matches**, comprising **45/45** in Session `7101` and **18/18** in Session `7102`. Both hosts completed their native lifetimes, closed their owned sockets, successfully rebound their exact selected addresses, and left no application process or port owner. This verifies actual remote input and state publication across two physical devices, rather than synthetic fixtures, local loopback, or a virtual relay.

**Prior failure records are preserved:** R1 had no overlapping process lifetime; R2 overlapped but received zero host datagrams; R3 again overlapped with zero host datagrams despite the user's earlier Windows Public program allow. The owner then explicitly added a narrow Ubuntu UFW rule permitting inbound UDP `<physical-linux-port>` from Windows `<physical-windows-address>` via `enp4s0` to `<physical-linux-address>`. R4 succeeded with the identical source and binary identities. This sequence is consistent with an inbound-policy constraint, but the exact earlier packet drop point was not independently measured. The assistant made no firewall, route or profile configuration changes; the user-owned UFW rule remains for the owner to manage.

**Quality and limits:** changed UDP module focused tests **13/13**, driver argument tests **3/3**, Ubuntu build/strict Clippy/format and Windows native MSVC build all passed. Original application receipts, failed attempts, raw logs, SHA checks, post-run cleanup and exact-byte reconciliation are retained at `artifacts/verification/m28-yt274-20261122/`. The root handoff at `.pipeline-output/m28-yt274-20261122/HANDOFF.md` records the review/delivery boundary. This is a private **headless** research application demonstration, not GUI/GameHost GPU verification, arbitrary-device reliability, authenticated untrusted LAN networking, generalized public `yetlet::udp` application protocol, registry publication or release qualification.
## 7. Verification, authority and immediate next work

**Historical planning checkpoint (2026-11-19):** The instructions below record the original planning scope. Later user-authorized delivery is recorded in the dated checkpoints above.

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
