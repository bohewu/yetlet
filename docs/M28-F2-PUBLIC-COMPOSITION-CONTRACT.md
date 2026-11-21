# M28-F2 / YT-281 — Public composition and extension contract candidate

- **Date:** 2026-11-19.
- **Status:** Independently reviewed **YT-281 design-only contract candidate** (R4 actual PASS). ADR0013 separately remains Proposed. No public Rust item, feature, crate, or consumer specified here has been implemented or compiled.
- **Inspected baseline:** `363dde2f83c248a5c099bc6a1fff67c656039145`, branch `docs/m28-composable-architecture`.
- **Authority and next gates:** [YT-281 backlog](BACKLOG.md#m28-follow-through--multi-host-sdk-delivery), [F2 delivery plan](M28-MULTI-HOST-DELIVERY-PLAN.md#5-f2---sdk-and-public-composition-delivery), [ADR 0013](adr/0013-composable-application-and-hosts.md) (still **Proposed**).
- **Scope:** Public contract selection and migration specification only. No change to existing Yetlet SDK/CLI, Cargo graph, implementation, or platform acceptance. Physical D4 remains **Not verified**.

## Subsequent bounded adoption — 2026-11-19

The status bullets above describe the reviewed YT-281 checkpoint. The maintainer
has since explicitly [accepted ADR0013's scoped F2 amendment](adr/0013-composable-application-and-hosts.md#accepted-scoped-f2-decision--2026-11-19)
and authorized YT-282-B public std-only core implementation. The illustrative
signatures below remain historical design sketches: use the actual public module
and its compiling consumers for implemented signatures. Native/Web/UDP adapters,
external YT-283 consumers and YT-284 package qualification remain later work.

## 1. Decision: a static, application-owned composition root

**Choose a small, statically typed composition root with concrete adapters and caller-supplied lifecycle callbacks.** Reject a universal `Host` trait, dynamic registry, a mandatory scheduler, and a per-Host process/thread. An external custom Host participates by supplying the same typed ownership and lifecycle callbacks as a built-in adapter, without a Yetlet source change or a special registration branch.

The root coordinates **owned resources** rather than claiming to own every event loop, runtime, renderer, or domain state. It must retain the original worker/task handles and first failure; a status-only coordinator that merely prints `Stopped` does not satisfy F2. Hosts remain ordinary Rust values. The concrete application chooses its domain/ports and how to invoke the foreground event loop.

The F0-C no-extraction conclusion remains correct for its *two* Headless/Pure-Web consumers. F1 supplies a new real ownership-sensitive case: required UDP and Web startup/rollback, admission closure before authority stop, original Web worker completion after native exit, and a shared failure precedence rule. That justifies an opt-in coordination **operation**, not a mandatory Host type for all applications.

### Grounding: actual current call sites

| Existing site | Demonstrated responsibility | Public candidate operation |
| --- | --- | --- |
| [D2 Native root](../examples/composable-host-research/native/src/bin/d2-loopback.rs), `D2App::start_required_web` | NetworkHost owns the sole C1 authority; Web start after UDP; on HTTP bind error explicitly stop prepared UDP | `Composition::required_*` setup with rollback of previously owned resources |
| D2 `schedule` / `about_to_wait` | One foreground owner polls nonblocking UDP, applies at most one queued Web command per tick and publishes a copied C1 state | Caller-driven `parts_mut` and explicit typed domain ports; **no SDK simulation tick** |
| D2 `resumed` / `window_event` | Winit Window is created and GPU initialized in the event loop, not by construction | `mark_ready` after actual callback readiness; no artificial background foreground loop |
| D2 `exiting` / `main` | Admission closes before authority/socket stop; Native drops Window/GPU; Web completes/joins after `run_app` | `request_stop` + `complete_until` with preserved primary cause and pending handle |
| [Web service](../examples/composable-host-research/native/src/f1_web.rs), `F1WebService` | Bounded command admission, immutable Snapshot, own Tokio thread and original JoinHandle, held-handler drain | `web_axum::HttpWorker`, explicit owned-Web bridge and deadline observation |
| [UDP adapter](../examples/composable-host-research/native/src/udp_d2.rs), `NetworkHost` | Loopback native socket and finite polling; game policy and C1 HostFixture are currently mixed in private research | `udp::UdpEndpoint` transports bytes; consumer owns session/codec/player permissions |
| [Headless root](../examples/composable-host-research/src/headless_composition_f0b1.rs) | Synchronous local `Rc<Snapshot>` state, no thread/socket/runtime | `required_local` when desired; ordinary Headless usage remains valid without composition |
| [Pure-Web root](../examples/composition-web-research/src/lib.rs) | Configures existing `server_axum::App` / `Router`; caller owns binding/runtime/drain | **No migration required** for normal Web; optional `web_axum` adapter only for composed apps |

This is a source map, not an assertion that the proposed types already exist.

## 2. Selected public package/module and opt-in names

| Candidate package | Feature | Public path / owner | Selected dependency boundary |
| --- | --- | --- | --- |
| Existing `yetlet` | `composition` (new, off by default) | `yetlet::composition` | Rust standard library only; no required Axum/Tokio/UDP/winit/wgpu; callable without HTML rendering |
| New companion `yetlet-hosts` | `native-winit` | `yetlet_hosts::native_winit` | Optional native foreground/window integration; caller owns the event loop and thread affinity |
| `yetlet-hosts` | `renderer-wgpu` | `yetlet_hosts::renderer_wgpu` | Optional compatible native GPU renderer; renderer selection is not Host identity |
| `yetlet-hosts` | `udp` | `yetlet_hosts::udp` | Optional standard UDP socket/lifecycle adapter; no game rules or universal Transport trait |
| `yetlet-hosts` | `web-axum` | `yetlet_hosts::web_axum` | Opt-in existing `yetlet/server-axum`, native Axum/Tokio; pure Router path is unchanged |

`yetlet-hosts` has **no default features**. `yetlet` retains its current default `html` and existing `server-axum` contract. No normal import, feature graph, CLI template, or SSR request path can gain a game tick, GPU/socket dependency, worker, or composition state merely because this proposal exists. The package and feature names are selected for implementation review here, **not added to Cargo**. Avoid a new crate for every host; use one companion with independently selected feature modules.

### Minimum candidate public API shape

`yetlet::composition` selects the following names and semantics for YT-282. This is a **signature-level design sketch**, not executable Rust documentation:

| API | Ownership/behavior contract |
| --- | --- |
| `Composition<E, Parts>::new()` | Empty local composition; `E` is a caller-selected error type; no global `Send` bound or implicit runtime |
| `required_local(label, value, close)` / `try_required_local(label, prepare, close)` | Move an already constructed local value or **fallibly prepare it inside the root**; the root owns the returned typed slot and synchronously closes earlier acquired slots if a later required start fails |
| `try_required_worker(label, start, close_admission, signal_stop, observe_completion)` | Start and retain a real worker handle **inside the already-owning root**. A failed HTTP bind/readiness stops previously prepared UDP and keeps the original Web error primary. Distinct callbacks close new domain commands, send the Web worker's graceful-shutdown signal, and observe the exact owned JoinHandle; Pending never detaches |
| `required_foreground(label, value, close) -> (root, PartKey<T>)` | Attach a **required, caller-thread owned native Window/Renderer slot** before entering winit. It stays not-ready until an actual `resumed` callback creates both Window and Renderer; failure is reported on this same typed part. It owns no invented worker thread |
| `borrowed(label, &mut value) -> (root, PartKey<T>)` | Attach a lifetime-bound **caller-owned** mutable value via a typed key. The root may use its operations while borrowed and closes its logical participation on stop, but never claims to own/drop that value; after the root releases the borrow, the caller can inspect/reuse it. Borrowed runtime variants cannot shut down an unrelated Tokio runtime or scheduler |
| `with_part_mut(part, f)` / `parts_mut()` | Borrow concrete parts by typed key/known position to drive native callbacks/UDP work; no dynamic lookup or unrestricted mutable domain alias |
| `mark_ready(part)` / `fail_required(part, cause)` | Mark observed readiness for **all required parts, including Native** (not task creation); retain first typed failure and later cleanup errors separately |
| `request_stop(cause)` | Idempotent control path independent of command queue; close Web command admission before stopping C1/UDP, **send worker shutdown** before observing completion, then dispose actual owned resources in application-declared order |
| `complete_until(&mut self, deadline) -> StopReport<E>` | Observe actual owned completion after foreground return **without consuming the root**; truthful `Complete` / `Pending` and joined-vs-worker-succeeded distinction; if Pending, **the same root still owns the original handle** for later observation |

The static builder's return types retain each concrete part and callback type in `Parts` (tuple/linked typed storage, not `Vec<Box<dyn Host>>`). Callers may name their application root with a type alias if generated types are verbose. The exact generics, receiver types and callback error conversions must be compiler-checked in YT-282 *before* the surface is stabilized; a sketch is not a declaration of compiled API compatibility. No universal `Host` / `Renderer` / `Transport` trait is introduced.

**Explicit limits:** No arbitrary foreground loop starts as a `Send` future. Winit callback readiness and draw failure stay with the caller-thread native adapter and its **required** part. No generic `run_all()` or magically reordered async stop. Shutdown order is selected at composition setup: Web **close_admission** fences new typed commands before C1 stops; the separate Web **request_stop** signal (a one-shot control independent of the bounded data queue) makes Axum stop accepting and gracefully drain already-running handlers; only after that can the root truthfully observe the original Join. A worker bound to a borrowed runtime exposes completion of **its own** work only. `Pending` never means success, and a permanently held HTTP handler can prevent eventual OS termination.

### Required failure and completion state model

1. `Prepared` -> `Starting` -> `Running` only after each required component's actual readiness. A prepared UDP socket is not a ready native Window; a spawned HTTP task is not HTTP 200 readiness.
2. Any required startup or runtime failure preserves the **first** typed cause. A `try_required_*` start failure returns that error **after rolling back already-owned preceding slots**, with secondary cleanup results recorded separately. For an HTTP bind/readiness failure after UDP acquisition, the UDP socket is explicitly stopped by the root, not left to a caller-side `?` or mere Drop. A failed Native `resumed` is likewise a required-component failure, not an optional rendering failure.
3. `request_stop` first closes new Web command admission (including a concurrent submit fence) before stopping domain authority; then sends the separately owned HTTP graceful-shutdown signal and observes its serving task. Queued/admitted is distinct from applied; pending unapplied commands may be discarded, while already-running HTTP handlers may drain. Closing domain-command admission alone never counts as stopping the HTTP worker.
4. `Stopping` can yield `CleanupPending`; a later `complete_until` observes the *same original* owned handle. `Stopped` requires actual completion; per-component reports distinguish local `JoinNotApplicable`, worker `JoinedSuccess`, and `JoinedFailure`.
5. A full data queue cannot block shutdown control. A timeout does not cancel the borrowed runtime, detach the handle, assert a hard OS-exit deadline, or silently rewrite the initiating error.

The root does **not** impose a universal actor, simulation cadence, command count, Snapshot schema, authentication, or cross-process state sharing. If a domain chooses one writer, all its producers use its explicit typed ingress; its immutable published generation/revision/tick may be compared by one pinned identity.

## 3. Data/transport and Rust type boundaries

| Value or operation | Type/ownership rule |
| --- | --- |
| Local domain authority, local observers, custom caller-thread Host | Can hold `Rc` / `RefCell` and `&mut`; no global `Send`/`Sync` constraint |
| Cross-thread HTTP command and read-only publication | The **selected adapter edge** requires command `C: Send + 'static` and copied publication `S: Clone + Send + Sync + 'static`; no `Rc<GameState>` crosses |
| HTTP handler state | Retain existing Axum `S: Clone + Send + Sync + 'static`, handler and native `Router` rules, without imposing them on local composition |
| Web runtime | Owned worker starts/joins its selected Tokio thread; borrowed-runtime variant does not stop/cancel the runtime itself |
| UDP | Raw bytes/endpoints with finite nonblocking read budget; application-specific codec/session/peer authorization/replica policy remains consumer-owned |
| Native Window/render | Winit event loop stays caller-thread, GPU renderer is explicitly selected; drawing immutable projection never advances the authoritative simulation |
| Error result | The app chooses its typed `E` and maps each concrete adapter error; no string-only failure identity or blanket `dyn Error + Send` bound on a local root |

A candidate `yetlet_hosts::web_axum::CommandBridge<C, S>` exposes **bounded typed** enqueue/receive, closed/full/stale admission results, immutable read-only publication and latest/exact-revision read where the application's `S` exposes such identity. Only that explicit cross-thread bridge has the edge bounds above; it is not inserted into Pure Web requests automatically. The bridge does not execute commands: the domain consumer records application separately from an admission receipt. A candidate `yetlet_hosts::udp::UdpEndpoint` owns socket operations, not the F1 `HostFixture`/`YTP1` rules or token authentication.

## 4. Consumer-shaped use (design sketches, not compiled examples)

### Direct Pure Web (existing public API; unchanged)

```rust
// This code path already exists: no Composition or yetlet-hosts required.
let router = yetlet::server_axum::App::new()
    .get("/health", health)
    .into_router();
// The caller selects its existing serve(), serve_on() or native Router path.
```

### Caller-local Headless/custom Host (proposed)

```rust
// Candidate calls; NOT IMPLEMENTED in this commit.
let local = Rc::new(LocalDomain::new());
let mut caller_owned_log = RevisionLog::default(); // remains owned by caller
let root = Composition::<AppError, _>::new()
    .required_local("input", LocalInput::new(Rc::clone(&local)), close_input)
    .required_local("audit", RevisionAudit::new(), close_audit);
let (mut root, log_part) = root.borrowed("revision-log", &mut caller_owned_log);
let applied_revision = drive_one_local_frame(&local); // one C1 owner step
root.with_part_mut(log_part, |log| log.record(applied_revision));
root.parts_mut().audit.record(applied_revision); // meaningful custom Host work
root.request_stop(StopCause::Finished);
let local_report = root.complete_until(deadline); // local: JoinNotApplicable
assert!(local_report.cleanup_complete());
drop(root); // releases the borrow; does NOT destroy caller-owned log
assert_eq!(caller_owned_log.last(), Some(applied_revision));
// No worker, HTTP, winit or global Send bound.
```

The external `RevisionAudit` has an actual operation: observe and record each applied revision and expose a local count/status. It lives outside Yetlet, uses the same `required_local` attachment rather than a built-in extension branch, and is stopped by the root. The borrowed `RevisionLog` is a separate concrete consumer-owned resource: its `record` operation occurs through the typed borrow, the root ends its participation on stop, and the caller still owns and can read that exact log **after** `drop(root)` releases the borrow. A borrowed Tokio runtime handle uses the same ownership rule (only the application's explicitly owned task may be stopped, never the caller's runtime), but this Headless case proves the proposed borrowed operation without needing Tokio. These expressions are illustrative; YT-283 must compile/run an independent external consumer before they are called working public examples.

### F1 Game + UDP + Web plus external custom Host (proposed)

```rust
// Candidate calls; API NOT IMPLEMENTED. One C1 owner stays inside the
// application-owned Composition for the entire stop/drain sequence.
let ports = app.typed_web_ports(8); // copied bridge edge, NOT a second C1
let root = Composition::<AppError, _>::new()
    .required_local("domain", app, stop_c1_authority)
    .try_required_local("udp", || udp::UdpEndpoint::bind_loopback(config), close_udp)?
    .try_required_worker(
        "web", || web_axum::HttpWorker::start(app_router, ports.web()),
        close_web_admission, signal_web_graceful_shutdown, observe_original_join,
    )?;
// If Web bind/readiness fails, the root has already acquired C1 and UDP.
// try_required_worker closes those preceding owned slots, retains Web's error
// as PRIMARY, reports any cleanup error separately and returns Err. No
// unowned UDP socket can bypass rollback through '?'.

let (root, native_part) = root.required_foreground(
    "native", NativeWindowSlot::new(), close_native_window_and_renderer);
let root = root.required_local("audit", RevisionAudit::new(), close_audit);
let mut native = NativeApp::new(root, native_part); // NativeApp OWNS root+C1

// NativeApp implements winit::ApplicationHandler on the caller thread:
// resumed(active_loop):
//   result = root.with_part_mut(native_part, |slot| {
//       slot.create_window_and_renderer(active_loop)
//   });
//   match result {
//       Ok(()) => root.mark_ready(native_part), // actual Winit+GPU ready
//       Err(err) => { root.fail_required(native_part, err); active_loop.exit(); }
//   }
// Only actual UDP bind, HTTP 200 and Native Window+Renderer may mark the
// three required Host capabilities Running; the root also owns the C1 part.
// Per wake: finite UDP, bounded typed Web input, single C1 step, immutable
// Snapshot publication, custom RevisionAudit observation.
// redraw_requested(active_loop):
//   if let Err(err) = root.with_part_mut(native_part, |slot| slot.draw(snapshot)) {
//       root.fail_required(native_part, AppError::Draw(err));
//       active_loop.exit();
//   }
// The actual initiating draw error must remain primary even when run_app
// itself later returns Ok after the requested event-loop exit.
// exiting(): native.request_stop_while_authority_owned(WindowClosed), using
// the same idempotent stop sequence shown below before C1 can be released.
let loop_result = event_loop.run_app(&mut native);
if let Err(error) = loop_result {
    native.fail_required(native_part, AppError::EventLoop(error));
}

// This MUST run while NativeApp still owns its root and the single C1.
// It also covers a failed run_app return that skipped the exiting callback.
// Internally the same root.request_stop() first fences new HTTP command
// admission, then stops the C1 owner/UDP and releases Window+Renderer in
// configured dependency order, and sends Web's independent graceful-shutdown
// one-shot. Neither owner nor pending JoinHandle is dropped early. Preserve
// a prior Native failure instead of replacing it with WindowClosed.
native.request_stop_while_authority_owned(StopCause::WindowClosed);

// Only AFTER admission is closed and C1/UDP/Native cleanup has proceeded
// may NativeApp transfer the still-owned Web task/JoinHandle in the root.
// HTTP stops accepting new requests and drains already-running handlers;
// complete_until observes the original worker, not an artificial replacement.
let mut root = native.into_composition_after_stop();
let report = root.complete_until(deadline); // &mut self, not consumed
if report.cleanup_pending() {
    // Root still owns the SAME original JoinHandle for later observation.
    // No hard process-exit deadline or silent detach is claimed.
}
```

The sketch has three required **Host capabilities** (Game, Network and Web). Game's caller-thread Native slot and its sole application-owned C1/domain slot remain two distinct typed resources under one root, alongside UDP, Web and the meaningful external `RevisionAudit` custom part. Initialization, redraw and event-loop failures all latch their original cause **before** requesting shutdown, so a normal `run_app` return cannot falsely replace an earlier draw error with `WindowClosed`. The `NativeApp` holds that root through `run_app`, and even a failed return requests stop **before** extracting or dropping C1. It still omits Rust generic/callback error conversion details and is **not** proof that these public types compile today. `try_required_worker` must own and preserve the worker through failed startup/rollback and later Pending observations; treating `close_admission` as the shutdown signal would violate the accepted F1 behavior. YT-282 must replace the sketch with a compiling consumer-driven signature; YT-283 must demonstrate the external audit Host through that exact public boundary. One host application PID and a separate UDP client process remain distinct, as they were in F1.

## 5. Migration and implementation map

| Existing F1/private behavior | YT-282 implementation owner | Non-negotiable acceptance |
| --- | --- | --- |
| `D2App` owns network, optional Web, Window/Renderer and primary error | `yetlet::composition` typed required attachments + `yetlet_hosts::native_winit` | Keep exactly one C1/domain local part in the application-owned root, distinct from UDP and Native slot. `NativeApp` retains the root through `run_app`; `exiting` or fallback after a failed return requests stop with C1 still owned. Mark native ready only from actual `resumed` Window+Renderer success; no second C1 |
| `NetworkHost::bind` / `poll` / `stop` | `yetlet_hosts::udp::UdpEndpoint` | Socket lifecycle and finite poll separate from F1 game rules; default loopback only |
| `F1WebService::start_game` / `publish_snapshot` / `pop_control` | `yetlet_hosts::web_axum::HttpWorker` / typed CommandBridge | Start inside root after UDP acquisition with explicit rollback on HTTP bind/readiness failure; distinguish close-admission, separate graceful HTTP shutdown signal and actual Join; copied immutable publication, no cross-thread `Rc` |
| `D2App::exiting` + `complete_within` | `Composition::request_stop` / `complete_until` | Call `request_stop` **before extracting the owning root or releasing C1**; close Web ingress, stop C1/UDP and release Native resources before observing HTTP worker completion; preserve first cause, true original Join and retained Pending |
| Actual winit/window/wgpu drawing | `native_winit` + optional `renderer_wgpu` | Preserve caller-thread event-loop ownership, selectable projection/renderer and truthful GPU cleanup |
| F0-B1 Headless root | Optional `composition` for new callers | Keep original Rc local usage and std-only graph; no forced wrapper/migration |
| F0-B2 existing Router/App | **No migration**; optional HttpWorker only for composed use | No changed public `App` behavior or mandatory composition |
| External audit Host | `required_local` custom attachment in independent consumer | No private include, no Yetlet source change, actual audit operation and lifecycle |

Implementation order for YT-282: (A) std-only static composition and fake owned/local attachment tests; (B) one real native Web owner/drain and typed bridge; (C) reusable UDP adapter with app-owned protocol; (D) caller-driven native window plus opt-in wgpu renderer; (E) migrate the F1 consumer using the same ports. **YT-283** separately tests a genuinely external custom Host and `Rc` usage; **YT-284** qualifies exact feature/dependency combinations and package consumption. No new test framework, scheduler, game engine, DI, runtime auto-start, generic Host trait, or public game-specific Snapshot/Command is required.

## 6. Alternatives, decision risks and ADR gate

| Design | Benefits | Material costs | Decision |
| --- | --- | --- | --- |
| **Typed application-owned slots + concrete adapters** | First-class external Host attachment; real stop/complete ownership; no global Send or dynamic registry; one opt-in package | Static nested types and typed callback wiring; compile validation required | **Select for YT-282** |
| Static generic `Host` trait for everything | Common method vocabulary and generic traversal | Foreground winit / local Headless / owned Web have different required execution and Join contracts; a trait would need extra capability methods or fake N/A methods | Do not select absent a proven simpler call site |
| `Vec<Box<dyn Host>>` / async HostGroup | Easy dynamic registration | Erasure/Send/futures/runtime scheduling, type/lifetime friction; unsupported runtime hotplug and misleading uniform cleanup | Reject |
| Promote existing `D2App`/`F1WebService` as SDK | Small move-only diff | Copies Twin Squares, YTP1 session policy and winit app-specific orchestration into SDK; no real external extension | Reject |
| Keep private F1 only | Zero public churn | Users must copy its orchestration and adapters; fails the owner's usable SDK goal | Reject for F2 |

**ADR boundary:** ADR0013 still says no public generic Application/Host/HostGroup for the original private POC. The proposed opt-in `Composition<E, Parts>` is a *new consumer-grounded public coordinator*, not a universal Host trait. Before implementing/promoting it in YT-282, the owner must explicitly decide the ADR amendment/acceptance, recording this scoped changed contract without retroactively accepting the older Proposed ADR. This document and YT-281 review do not themselves amend or accept ADR0013.

## 7. YT-281 acceptance and truthful limits

- Exact candidate module, companion package, feature names, typed extension operations, first-failure/cleanup semantics and Rust bounds selected above.
- All candidate operations map to real F1, F0-B1/F0-B2 or required external custom-Host consumer actions. External custom usage remains an implementation obligation, **not** a current working API.
- Preserve successful F1 evidence at its source and platform scope; do **not** re-run old GPU/UDP/HTTP E2E, Reviewer, native/Cargo or browser tests for documentation-only design.
- Only this new English contract, linked Backlog and Roadmap receive scoped document/link/dependency/whitespace checks and new independent review. Global Docs inherited FAIL18 in ignored F0-B2 Chinese handoff remains a separate non-PASS; retain that evidence.
- No Stage/Commit/Push, no public API or feature mutation, no implementation of YT-282/283/284, no physical D4 qualification, no new Host Trait.
## 8. Independent review and YT-281 closeout

Independent R1 `wc_job_Gg6DaWF8HyV8jtyZ` returned actual FAIL3/P2 (startup rollback, required Native readiness, worker shutdown signal); R2 `wc_job_XvwGVrxGgrGieztL` actual FAIL1/P2 (C1 destruction before HTTP admission closure); R3 `wc_job_mWWOvRV4oBT-4cC-` actual FAIL2/P2 (unreported redraw error, unexercised borrowed attachment). All failures, focused source fences, reviewer JSON and raw transcripts are preserved under ignored `.pipeline-output/m28-f2-yt281-20261119/`. Their repairs are integrated into this document; the implementation source was unchanged.

**Independent R4 `wc_job_IRqBb3AB_nBtRySb` terminal completed, exit 0, actual JSON PASS, `issues=[]`, `required_followups=[]`.** Its exact reviewed contract-source SHA256 was `38f90ec5ba5d0de8bffa437c82042b057f21f6532c5d08c25fbc3bfa6fd6f107`. The later edit of this section and the status label records that result, not a newly modified API design. The saved role was `gpt-6.2-astra`, observed `high`, without changing original Runner permissions; this was a role-directed Codex Job, not native `agent_type` formal assurance.

Scoped document/link/whitespace and unchanged-implementation-source checks PASS. The repository-wide Docs run still returned inherited **FAIL18**, all in the unchanged ignored F0-B2 Chinese handoff; it was not upgraded to PASS or hidden. YT-281 is Done **only as the reviewed public contract design**. YT-282 implementation, YT-283 external custom-Host compilation, YT-284 compatibility/public usage, ADR0013 acceptance, physical D4 and Git delivery remain separate; none is claimed complete.
