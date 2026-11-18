# M28-E / YT-275 — Evidence-Based Promotion Decision

- **Date:** 2026-11-18.
- **Source baseline:** isolated M28 worktree `docs/m28-composable-architecture` at `cde0a03f712e01ffb48811a2f1121afc1e976f7b`.
- **Decision state:** **Proposed for owner acceptance**. This is a documentation and architecture recommendation, not an accepted public API or an implementation authorization.
- **Parent:** [M28 plan](M28-COMPOSABLE-APPLICATION-PLAN.md), particularly [M28-E promotion gate](M28-COMPOSABLE-APPLICATION-PLAN.md#m28-e-promotion-gate).
- **Related:** [ADR 0013](adr/0013-composable-application-and-hosts.md) remains **Proposed**; [M28 roadmap](ROADMAP.md#m28--composable-application-and-multi-host-research), [YT-275 and conditional YT-274](BACKLOG.md#m28--composable-application-and-multi-host-research).
- **Review:** Original independent R1 `wc_job_tHV6cHY7qckBm8pG` terminal exit0 actual FAIL3 (wire frame-size wording, public ActionExecutor semantics and B0 source provenance); all three factual/documentation defects corrected. Independent R2 `wc_job_yKyyePqAbLGP4xaz` terminal exit0 actual JSON **PASS**, `issues=[]`, `required_followups=[]`; original R1 FAIL remains retained. No product tests were rerun. **Owner acceptance and any public implementation remain separate.**

## 1. Decision proposed

**SIMPLIFY the architecture contract, RETAIN the M28 host implementations as private research, and defer public runtime promotion.** The useful commonality is *explicit application-owned composition and resource lifetime*, not one universal Host trait, actor, scheduler, renderer, transport or worker.

This decision is deliberately narrower than the name "Composable Application Framework": the direction is supported by multiple concrete examples, but a stable public `Application`, `Host`, `HostGroup`, `Renderer`, `Transport` or `NetworkHost` interface is **not yet justified**. No new crate, default feature, SDK API, CLI template, code path, release or branding change is approved by this document. The owner may separately authorize a small opt-in adapter after checking actual consumer contracts.

**Preserve first-class pure Web/API/SSR.** Existing Yetlet `server_axum::App`, `into_router()`, native Axum/Tower integration, current View/HTML/DOM and CLI templates continue without mandatory host composition, actor, command mailbox, tick, worker, GPU, network runtime or new dependencies. The B0 source-bound baseline is the preservation guard; it is **not** a measured claim of zero overhead or a benchmark.

The case for promotion is **selective, not all-or-nothing**. In particular, "one authoritative StateOwner" is an application-selected shared-game-state policy, **not** a rule imposed on ordinary concurrent Web request handlers. Hosts need not imply threads or processes, and render data is not necessarily Yetlet HTML View.

## 2. Actual source- and environment-scoped evidence

The IDs below identify **separate reviewed source milestones**, not a single simultaneously revalidated release matrix. Counts and environments must retain their original test selection and exclusions. See the linked plan sections and ignored detailed verification receipts; unavailable ignored receipts must not be simulated from this summary.

| Stage / local source | Grounded accepted capability and selected evidence | Independent Reviewer | Explicit limit |
|---|---|---|---|
| [A / YT-265](M28-COMPOSABLE-APPLICATION-PLAN.md#9-documentation-verification-and-review-boundary), `4e5bdc1` plus owner clarification | Compared concrete composition, static adapters and dynamic HostGroup; clarified independent Host, renderer, execution topology and B0 Web preservation | Clarification `wc_job__Fg38dxSv31AqFr5` actual PASS | Architecture/docs only, not runtime approval |
| [B0 / YT-276](M28-COMPOSABLE-APPLICATION-PLAN.md#10-b0-existing-web-baseline--yt-276), **tested at `30b0625`; documented/committed at `9b52970`** | Existing Axum/API/SSR/View source baseline: 55 selected tests plus eight doctests, executed against the `30b0625` checkout; locked/offline normal feature/dependency observations recorded subsequently without changing the tested Web source | `wc_job_gxUREPbXwflV_qFr` actual PASS | No new Host code, measured latency or subsequent changed-Web-path proof |
| [B1 / YT-266](M28-COMPOSABLE-APPLICATION-PLAN.md#11-b1-private-same-thread-multi-host-proof--yt-266), `77fd396` | Two concrete caller-thread hosts, independent lifecycle, `Rc` local state, bounded command ingress and one immutable shared snapshot; 7/7 focused PASS | `wc_job_q8KjqtfssJDRTlml` actual PASS | Private same-thread proof, no generic public Host or worker requirement |
| [B2 / YT-267](M28-COMPOSABLE-APPLICATION-PLAN.md#12-b2-owned-worker-and-required-host-lifecycle-proof--yt-267), `f2894d7` | One explicitly owned worker alongside caller-thread hosts; startup/failure/stop/join behavior and bounded ingress; 15/15 focused PASS | R1 P2 retained; R2 `wc_job_UqJr-AVptjqv0L0D` actual PASS | Does not prescribe a worker for each Host or transfer scheduler ownership |
| [C1 / YT-268](M28-COMPOSABLE-APPLICATION-PLAN.md#13-c1-deterministic-headless-twin-squares--yt-268), `f405d4f` | One C1 fixed-tick integer reducer, checked control sequence and authoritative immutable snapshots; 13/13 focused PASS, no display/network/dependencies | `wc_job_9AM-fykfMMzURQb4` actual PASS | Toy two-player game, not a general ECS, prediction, physics or global Web actor |
| [C2 / YT-269](M28-COMPOSABLE-APPLICATION-PLAN.md#14-c2-opt-in-native-gamehost--yt-269), `9be98e7` | Actual Ubuntu X11/winit + NVIDIA RTX 4090 Vulkan/wgpu 27 Native GameHost, keyboard/Pause and independent scene projection; selected 11/11 Native tests | R1/R2 failures retained; R3 `wc_job_Pi934pMwws_L1XwD` actual PASS | One Native backend/environment, not browser/Windows universal renderer |
| [C3 / YT-270](M28-COMPOSABLE-APPLICATION-PLAN.md#15-c3-bounded-ubuntu-native-visual-and-lifecycle-qualification--yt-270), `f203709` | Actual 960×640 and resized 720×480 X11 window/XWD pixel evidence, minimize/restore/WM close, bounded catch-up; 3/3 focused PASS | R1 P2 retained; R2 `wc_job_oQRL1tjhmefNiWii` actual PASS | Zero-size and Lost/Outdated were **injected**; no real device-loss, independent Windows GPU or FPS guarantee |
| [D1 / YT-271](M28-COMPOSABLE-APPLICATION-PLAN.md#16-d1-private-bounded-protocol-and-inmemory-session-proof--yt-271), `545153e` | Private fixed version/length bounded codec, Endpoint/Peer/Session/Epoch/Sequence/Tick admission and in-memory Host/Replica; 16/16 focused PASS, zero external normal deps | `wc_job_CjE2M88h6SNMhH7D` actual PASS | No real network, cryptographic identity or public stable wire protocol |
| [D2 / YT-272](M28-COMPOSABLE-APPLICATION-PLAN.md#17-d2-two-independent-native-application-processes-over-real-loopback-udp--yt-272), `7fc70f2` | Separate Ubuntu Native Host/Client PIDs and real `127.0.0.1` UDP; source/replica Revision90/Tick90 and two genuine matching Vulkan window captures; 4/4 UDP tests plus 3/3 postreview regressions | R1 FAIL3/R2 FAIL1 preserved; R3 `wc_job_yVREJyi-XD7-Ax2p` actual PASS | Positive GPU/UDP E2E predates the source-only negative-path corrections; no physical LAN or hostile-peer security |
| [D3 / YT-273](M28-COMPOSABLE-APPLICATION-PLAN.md#18-d3-bounded-real-loopback-faults-reconnect-and-headless-composition--yt-273), `cde0a03` | Actual two Headless application PIDs, one authoritative C1 owner; real loopback UDP fixed-seed drop/duplicate/reorder, 250ms Lost Release, 3s Timeout, new Session/Epoch and source/Replica Revision432/Tick432 convergence; 14/14 D3 tests | R1 FAIL3 retained; R2 `wc_job_-kuV4Ftw7m4omNAI` actual PASS | User-space faults on one Ubuntu host; E2E predates final source-only R1 fixes; no Windows or physical LAN claim |

**Evidence integrity:** The original failing Reviewer/test runs are retained; a terminal Reviewer JSON PASS does not erase earlier failures. The D2 earlier successful Build log and D3 earlier successful Clippy log each had one filename collision during later successful checks, explicitly documented in ignored handoffs. Do not imply those original successful log bytes were recovered. For every source-only postreview fix, distinguish targeted focused checks from a fresh full E2E.

### D4 physical-device gate versus supplementary virtual-network evidence

| Experiment | Real result | What it establishes | What it does **not** establish |
|---|---|---|---|
| D4 campus LAN preflight (ignored receipt: `.pipeline-output/m28-d4-20261118/NETWORK-PREFLIGHT-BLOCKED-HANDOFF.md`) | Three distinct physical hosts found; exact campus IPv4 interfaces chosen. Native UDP probes between Ubuntu↔Windows and Windows↔Windows **received zero valid application packets**, despite some ICMP/ARP reachability. All probes terminated and test sockets closed | Physical-host inventory, actual routing and a **blocking UDP ingress condition** on selected ports | Windows MSVC same-source D3 app, functional cross-host game/Session/Replica, or blanket proof of which firewall/ACL caused the block |
| D4VM Multipass supplement (ignored receipt: `.pipeline-output/m28-d4vm-20261118/VM-UDP-PASS-PHYSICAL-D4-NOT-VERIFIED-HANDOFF.md`) | Two separate Ubuntu 26.04 VM kernels on **one physical Ubuntu host** exchanged actual `<isolated-vm-network>` UDP via test-only per-VM relays with identical binary SHA256 `f9085aebcdd2a4f085ce241dffc4178fdb0f04d582deabc9fe00fc6b4e910ba9`. After fault injection and reconnect the owner/Replica matched **Revision462/Tick462, X=(22,0)**; all four process commands exited0; all guest UDP ports closed and both disposable VMs purged | Actual canonical D3 bytes crossed two virtual kernels; the source/replica protocol recovered via relay on the virtual LAN | Two **physical** devices; D3 **native** non-loopback bind (D3 still bound `127.0.0.1` inside each VM), Windows/MSVC, untrusted LAN authentication, hardware-independent fault realism or original D4 qualification |

The original **YT-274 / D4 physical LAN acceptance remains `Not verified — selected UDP ingress blocked`**. D4VM is useful *additional* confidence but cannot relabel D4. Absence of verified physical LAN is not a blocker for a **reviewed decision proposal**; it is a blocker for any feature/release statement that requires physical LAN support.

## 3. Requested workload and combination coverage

| Target scenario | Evidence status | Required scope qualifier |
|---|---|---|
| Existing pure Web/API/SSR, Axum `App` and `into_router()` | **Existing source path accepted at B0** | No new composition owner or measured before/after performance result |
| Two caller-driven Hosts without OS workers | **B1 proven, private** | One thread and concrete hosts, no generic Host contract |
| Two caller Hosts plus one explicitly owned worker | **B2 proven, private** | One selected worker, not per-Host allocation |
| Headless deterministic two-player simulation | **C1 proven, private** | One fixed-tick toy reducer, no GPU/HTTP requirement |
| Native GameHost with selected wgpu renderer | **C2/C3 proven for selected Ubuntu X11/Vulkan** | Main-thread Window ownership; actual GPU loss/other OS unverified |
| GameHost + NetworkHost, independent client GameHost + NetworkClient | **D2 proven on Ubuntu loopback** | Two OS processes, not physical devices; client read-only replica |
| HeadlessHost + NetworkHost and separate Headless Client | **D3 proven on Ubuntu loopback** | Real UDP, one C1 owner, injected fault scope |
| Same D3 protocol across two Ubuntu VM kernels | **D4VM proven through test-only relays** | Single physical machine and VM relays; no native non-loopback D3 bind |
| Physical Linux↔Windows or Windows↔Windows UDP | **Not verified, D4 blocked** | Actual campus network ingress failed before app transport |
| GameHost + NetworkHost + Axum in the **same application process** | **Not implemented / not verified** | Can be proposed but does not follow from separate C/D proofs |
| CustomHost adapting a third-party scheduler or main-loop | **Not implemented / not verified** | No independent custom Host consumer |
| Browser DOM + WebGPU renderers combined in one app | **Not M28-qualified** | Existing separate browser capabilities are not proof of integrated multi-renderer composition |
| Direct D3 binding to a non-loopback NIC, hostile LAN peers | **Not supported or qualified** | Current D3 implementation explicitly selects localhost; VM relays do not remove that limitation |

## 4. Promotion dispositions by responsibility

| Concept / candidate | Decision recommendation | Reasons and constraints |
|---|---|---|
| **Application composition** | **Promote the documented principle; keep concrete composition root in examples**. **Candidate for later opt-in adapter only** | B1/B2 and C/D show resource coordination without HTTP, but no public minimal signature is supported by a Web+Native+Network same-process consumer. Do not rename `server_axum::App<S>` or impose a universal `Application` object |
| **Host lifecycle / capability** | **Retain concrete Hosts; optional static typed ports only if a second production consumer emerges** | Foreground winit readiness/affinity, caller-thread `Rc`, owned worker and TCP/UDP readiness differ. A universal `async Host: Send` or process-per-Host contract would contradict observed behavior |
| **HostGroup / dynamic registry / DI** | **Reject initial core/public promotion** | No dynamic hotplug, registration or automatic restart need is proven; adds registry, identity, boxed futures and object-safety costs |
| **StateOwner / bounded Submit–Observe** | **Keep as opt-in domain pattern/private sample, not Yetlet core** | C1/D3 establish one writer for this game, snapshot publication and capacity errors. Generic Web services do not share that single-writer requirement. **Private queued ingress** acknowledges admission separately from application; existing public `ActionExecutor::execute` returns the command-specific response and `LocalExecutor` invokes its handler directly, not merely a queue-admission receipt |
| **Scheduler / execution topology** | **Application owned; do not promote a universal scheduler** | Fixed 60Hz simulation, native winit callbacks, Web requests and Headless step loops have different timing and failure modes. Enforce explicit worker/thread ownership only when selected |
| **Renderer / render data** | **Keep existing View/HTML/DOM; retain native wgpu game scene as separate opt-in research** | View is HTML data, Scene is not. Surface/device/window cleanup is backend-specific; no mandatory `NullRenderer`, renderer chosen by Host identity, or public `Renderer` trait |
| **NetworkHost / Transport** | **Keep private protocol-specific UDP adapter, no public general Transport trait/RPC layer** | D1–D4VM validate finite complete-message semantics under selected conditions, but TCP/QUIC/WebSocket reliability and security differ, physical ingress is blocked, and nonce/session are not cryptographic identities |
| **Protocol / session codec** | **Private versioned research contract only; no wire compatibility promise** | The five message kinds each have an **exact encoded length** (20/53/53/63/44 bytes including header), while **512 bytes including header is only the maximum datagram bound**, not a fixed frame size; extra trailing bytes are rejected. One peer/two game players and private rate/sequence/expiry are scoped to the prototype; threat model, authorization, compatibility and cross-OS conformance remain required before any shared SDK surface |
| **Axum integration** | **Retain current `App` and native `Router/Tower` path; consider one opt-in `into_router()` consumer later** | Preserve B0 direct Web path. No compulsory HostGroup or request mailbox; three-host Game+Network+Axum process remains untested |
| **CLI game template / SDK packaging** | **Defer; retain existing Web templates equally supported** | No stable public Host API, same-source Windows MSVC Native+UDP evidence, installation/cache/driver matrix or release qualification. A standalone private sample guide may be maintained without making a new template |
| **Product language** | **Describe composable application support as research direction only** | Do not reinstate a rigid HTML-first limitation, but do not claim an accepted universal Game/Network framework or physical LAN production support |

This is a **disposition matrix**, not permission to move any private code into `crates/yetlet`, create an independent published crate, change `Cargo.toml` default features, amend public docs/README positioning or accept ADR 0013. All such actions need a subsequent owner decision and independent source-scoped validation.

## 5. Costs, risks, and explicit preservation gates

1. **Baseline and compatibility:** Keep the B0 source/feature test selection and normal dependency graph as a regression fixture. The existing Web `App` direct Router/Tower path must remain callable without composing Hosts, creating queues/ticks/workers, or requiring winit/wgpu/UDP. Any later code change requires a focused changed-path comparison, and API names must not conflate Web `App` with a hypothetical general Application.
2. **Lifecycle and topology:** A Host lifecycle is not an OS thread. Respect foreground winit main-thread callbacks, borrowed runtimes, bounded stop independent of saturated data queues, actual owned worker joins and failure evidence. An "exit requested" or timeout is not proof of process termination; only actual observed terminal state qualifies.
3. **State and command semantics:** Domain invariants, ordered application, bounded admission, one authoritative C1 writer and immutable snapshots are meaningful for selected shared-state workloads. Do not export `&mut GameState` to network or renderer. Do not promise that accepted ingress is completed domain execution or impose the game actor model on Web services.
4. **Renderer compatibility:** HTML View/DOM and native SceneData use distinct projections; there is no proven one-model-for-all RenderTree, BrowserHost or multi-renderer service. Memory ownership, Window/Surface lifetime and real GPU error paths need their own qualification.
5. **Transport and trust:** Direct D3 binds loopback. Source address, peer ID, session, epoch and Hello nonce are **freshness/routing markers**, not strong authentication or confidentiality. An on-path or unauthorized LAN sender, replay/flood under source spoofing, network segmentation, MTU/fragmentation and OS firewall policies are not qualified. If Internet/untrusted LAN is required, independently evaluate a mature authenticated transport, not custom crypto or a universal UDP/TCP `send` facade.
6. **Fixed budgets and scaling:** The private two-player capacity (message-kind-specific exact wire lengths, **up to 512 bytes per datagram including the header**, finite queues, held-input expiry, rate limits and fixed game ticks) is **prototype-specific**. Decoder admission rejects trailing/padded bytes rather than accepting a universal 512-byte frame. Neither D2/D3 nor the virtual VM relay establishes Internet congestion control, production fairness, latency SLAs, GPU FPS, hotplug or many-client performance. Any promotion needs explicit measured costs and resource ownership.
7. **Evidence classes:** Distinguish B0 baseline, unit/in-memory proof, native GPU screenshot, two-process same-host UDP, seeded user-space fault injection, two-kernel VM relay, and real physical LAN. Negative-path repairs with targeted tests must not be described as a new full E2E run. Preserve original failed Reviewer findings and documented successful-log pathname collisions.

## 6. Minimum next opt-in contract experiment, if authorized later

The smallest useful follow-on is **one private, feature-neutral composition example** (not a new public crate) with a single process combining a concrete selected domain service, two caller-managed host capabilities and, only if needed, one owned worker. A later independently authorized experiment may add the existing `App<S>::into_router()` to this root without changing its direct HTTP pipeline. Its acceptance must explicitly include:

- Ownership table for each concrete resource, readiness/failure/stop and no process-per-Host shortcut; no universal `Send` or `async` requirement.
- Per-Host and per-state policy capability selection. The shared-game-state use case gets bounded latest input/snapshot ports, while a pure Web request still calls its services directly.
- At least **two independent consumers** demonstrating that a proposed generic signature reduces duplication without imposing winit, UDP or Tokio on the other consumer.
- Exact B0 Web tests/dependency comparison **only if** the relevant Web source/feature surface changes; real Native or network tests only for newly affected paths.
- An independently reviewed compatibility/naming ADR amendment, including a migration plan and concrete opt-in dependencies, **before** any stable public SDK symbols.
- A separate operational/transport/security review before non-loopback UDP, physical LAN, Windows MSVC, game template or networking claims.

Do not assign a new backlog identifier implicitly. Obtain owner scope/authorization, review workload and dependency-order before creating or executing any follow-on implementation.

## 7. Explicit owner decision requested

The proposed choice is **A — accept M28-E as a reviewed evidence-based disposition, keeping ADR 0013 Proposed and all Host/Transport code private pending another authorization**.

Alternative **B — authorize a separate narrow opt-in composition adapter experiment** only after a concrete shared Host consumer, public signature options and preservation gates are specified and reviewed. This is **not** authorized by selecting A or by merely completing M28-E.

**Reject C — immediately publish a universal Application/HostGroup/Renderer/Transport runtime**, because B0, C, D and D4 evidence cannot justify its mandatory costs, cross-platform/physical networking guarantees or lifecycle semantics.

In parallel, D4/YT-274 stays **Not verified** until a separately authorized physical-device UDP ingress fix and same-source cross-device application test. The successful D4VM supplement may be cited as a distinct virtual-kernel transport proof but cannot close D4. Any future owner acceptance of the architectural principle should explicitly state whether ADR 0013 remains Proposed or is amended in a *separate* reviewed ADR decision.

**No source code, root manifest, public product description, CLI template, ADR status or release is modified by this M28-E proposal.**
