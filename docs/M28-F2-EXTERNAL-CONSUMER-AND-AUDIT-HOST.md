# M28-F2 — Independent consumer and revision audit Host (YT-283)

## Status and source boundary

**Latest acceptance (2026-11-21): YT-283 Thin Native Window Host external
consumer is Done at public-source + real Native integration scope.** Independent
R1 `wc_job_trL0VLA-wKRTVtPM` terminal exit0, actual JSON PASS,
`issues=[]`, `required_followups=[]`. YT-284 package compatibility and F2
final delivery remain open. Older dated entries retain their former scope.

Current position (2026-11-21): the overall YT-282 public Source-only gate is
Done. YT-283-M1 and M2-A through M2-E are independently reviewed source
slices; M2-E passed independent R3 wc_job_xx_xLnOdmeet0MOa, terminal exit0,
actual JSON PASS, issues=[], required_followups=[]. Original M2-E R1/R2
FAIL2/P2 results remain preserved. Historical checkpoints below retain their
then-open gates. Complete YT-283/F1 migration and YT-284 remain open.

Independent R6 `wc_job_RzpUkYN_WzE5OZhT` returned actual PASS for the earlier
bounded external source proof. R1 FAIL5, R2 FAIL1, R3 FAIL2, R4 FAIL1 and R5
FAIL1 remain in the original records. The bounded YT-282-F application-data source-only increment passed independent
Reviewer R3 `wc_job_m8Cmu3fcywPg-6P_` (terminal exit0, actual JSON PASS,
`issues=[]`, `required_followups=[]`). Its own R1 FAIL2 and R2 FAIL1 remain
preserved as original failures. Overall YT-282, complete YT-283/F1 and YT-284
remain Todo.

The independent [consumer manifest](../examples/external-consumers/yt283-revision-audit/Cargo.toml)
has its own empty `[workspace]`, lockfile and feature selection. It is outside
Yetlet's workspace membership. It depends on the public path packages `yetlet`
and optional `yetlet-hosts`, plus public Axum/Winit. It uses no private modules,
research dependency, source inclusion or copied supervisor. It uses the reviewed
Composition Core and Web, UDP and Winit adapters with the YT-282-F rectangle
entry point in the separately selected WGPU renderer.

## Build and use

From the repository root, use the independent manifest explicitly:

```sh
cargo test --offline --locked --manifest-path examples/external-consumers/yt283-revision-audit/Cargo.toml --no-default-features
cargo check --offline --locked --manifest-path examples/external-consumers/yt283-revision-audit/Cargo.toml --all-targets --features three-way
cargo test --offline --locked --manifest-path examples/external-consumers/yt283-revision-audit/Cargo.toml --features three-way
cargo clippy --offline --locked --manifest-path examples/external-consumers/yt283-revision-audit/Cargo.toml --all-targets --features three-way -- -D warnings
cargo fmt --manifest-path examples/external-consumers/yt283-revision-audit/Cargo.toml --check
```

The current-checkout default consumer uses built-in `yetlet::composition`
with `default-features = false`; it needs no HTTP, GPU, window or worker. The optional
`three-way` feature selects `yetlet-hosts` Web/Axum, UDP, Native/Winit and
renderer-wgpu. One explicit smoke invocation briefly opens a visible 640×420
window with two squares and exits after the finite client verifies both moves.

```sh
DISPLAY=:0 WINIT_UNIX_BACKEND=x11 cargo run --offline --locked --manifest-path examples/external-consumers/yt283-revision-audit/Cargo.toml --features three-way --bin revision-audit-three-way -- --smoke
```

This invocation needs an accessible X11 display and compatible Vulkan GPU.
Offline builds require cached dependencies. For the YT-282-F verification,
the isolated Cargo target was `.cache/yetlet/builds/yt282-f-20261120/` and
the logs were retained under `.pipeline-output/m28-f2-yt282f-20261120/`.
The sample binds both network endpoints to loopback.

## Useful custom Host and ownership

[`RevisionAuditHost`](../examples/external-consumers/yt283-revision-audit/src/lib.rs)
is defined entirely by the consumer. It validates exact previous identity,
unchanged generation, next revision, both player positions, paused state and
the exact pinned readback.
It rejects stale, cross-generation, out-of-order, inconsistent or closed
publications without changing its ledger. The sample deliberately bounds its
ledger to 16 entries and refuses further application when full. Status, pinned
identity and audit records remain readable after truthful close.

One caller-local `Rc<C1Domain>` is the authority. Its apply operation preflights
the next state and audit contract without mutation, publishes the candidate
through the real `CommandBridge`, reads `SnapshotReader::at_revision`, and checks
both publication and snapshot generation/revision plus the actual state. Only
then does it commit C1 and append exactly one audit entry on the same caller
thread. A rejected publication changes neither C1 nor audit. A post-publication
readback/commit failure latches its first failure, closes admission and leaves
C1 uncommitted with completion Pending; immutable published history remains
visible and is never described as rolled back. A command never executes in HTTP or in the test client.
The custom Host attaches through ordinary `Builder::required_local` callbacks;
there is no Host trait, registration branch, DI container or new framework API.
Local tests compose real Rc values with successful close, invalid publications,
first-failure retention and deliberately blocked cleanup/Pending.

The [three-way wiring](../examples/external-consumers/yt283-revision-audit/src/three_way.rs)
uses `CommandBridge` with four admitted commands and 16 immutable snapshots.
`GET /ready` must return 200 before Web readiness. A request frame is exactly
20 bytes. It starts with `RA`, version byte 1 and a tag, followed by big-endian
64-bit generation and revision. Tags 1 and 2 are Pause and Resume. Tag 3 is a
UDP read. Tags 4–7 are Player One moves in north, east, south and west order.
Tags 8–11 are Player Two moves in the same order. HTTP POST admits commands
only and returns 202 for admission. `GET /snapshot/{generation}/{revision}`
returns the exact retained snapshot. The seven snapshot fields are generation, revision, paused, Player One x and y,
and Player Two x and y.

UDP uses the same C1 apply path with at most four packets per foreground
wake and one explicitly trusted loopback peer. UDP read replies are gated
on the caller-local C1 having committed the requested revision. Each poll gets
the trusted address from the live socket reservation and uses that address for
both sender validation and replies. The application endpoint is privately
owned. Its address and the reservation address are available through read-only
accessors. The socket clone retains the peer port after the finite client
returns. Closing first releases the application endpoint, then the reservation.
The peer lookup returns `NotConnected` after close.

The [binary](../examples/external-consumers/yt283-revision-audit/src/main.rs)
owns Winit's caller-thread EventLoop, WindowSlot, separately selected
SurfaceRenderer, the original Web/Tokio worker and one bounded joined test
client. Initial Native readiness needs a matching Window redraw and a
returned Present. An eligible redraw reconfigures an Unconfigured renderer
using the live Window size, including after Lost or Outdated. Polling, redraw,
startup, client I/O and stop have finite cooperative budgets. A synchronous
driver call can outlast those budgets. Other foreground schedulers cannot
automatically share Winit's thread.

## YT-282-F application-data scene path

The 24×16 arena uses 2×2 blue and orange squares. C1 applies a cardinal Move
for one player by one cell, clamped so the entire square stays inside the
arena. A move at an arena limit can advance the revision with the position
unchanged. A move while paused is rejected with the current revision intact.

The finite client submits Player One east by HTTP at revision 0, receives
202, then verifies revision 1 over HTTP and reads the same committed
revision over UDP. It submits Player Two north by UDP at revision 1, checks
the application reply and a separate UDP read, then checks HTTP revision 2.
Native projects the copied C1 snapshot to two NDC rectangles on redraw. It
records the snapshot only after `draw_rects` returns Presented. Successful
closeout requires the exact revision-2 positions and paused state to agree
across that receipt, the client, the C1 snapshot, the published history and
Audit. The initial Present can start the client and occurs before that pin.

Independent YT-282-F R1 `wc_job_PUpAGDMfgwwB1KlL` returned actual FAIL2/P2.
The repair binds UDP trust to the retained socket and rejects overflowing
rectangle edges before surface acquisition or upload. A two-socket loopback
test verified that the other socket's valid Player Two move leaves C1 at
revision 0 and elicits no reply. Only the reserved socket applies that same
move, advances C1 once and receives the pinned snapshot. Separate compile-fail
doctests cover attempts to replace the endpoint and peer. Independent R2
`wc_job_htdyrvWrmo15r0kb` actual FAIL1/P2 then identified a potentially failing
startup peer lookup after C1 creation. The last lookup now runs before any
C1/Audit/admission acquisition, and its read-only address is passed to the
startup log. New binary focused 7/7, Clippy, Fmt and Build PASS; independent
R3 `wc_job_m8Cmu3fcywPg-6P_` actual PASS with no issues/followups.

The earlier R2-source run used Ubuntu 26.04.1 LTS, X11 `DISPLAY=:0` and an
NVIDIA GeForce RTX 4090 with Vulkan driver 595.99.02. PID 715103 exited 0
after 97 returned Presents. The third Present pinned generation
1795156271637764642, revision 2, unpaused, with players at (3, 2) and (20, 11).
The original Web worker and finite client joined and Composition reached
Complete/Stopped. The window and process disappeared, TCP refused connections,
and both UDP ports rebound. These observations cover the returned API calls and
captured X11 client area. GPU completion, physical scanout and induced driver
faults were outside the run. The earlier R1-source run, PID 669351 with 98
returned Presents, remains historical evidence.

| F1 migration area | YT-282-F evidence and remaining scope |
|---|---|
| Game state and Native projection | Two discrete moves and a revision-pinned rectangle frame through public APIs. Full Native keyboard/held-input behavior, the 60 Hz authority tick and tick-bearing snapshots remain open. |
| Network | Bounded RA1 loopback commands and exact committed reads. YTP1 session and epoch handling, player binding, reconnect and replica convergence remain open. |
| Web | Typed command admission and exact snapshot history verified over loopback. Full F1 status and tick integration remain open. |
| Composition and custom Host | One caller-thread EventLoop, one C1, one owned Web worker and consumer Audit exercised through Complete/Stopped. This bounded source-only path passed independent R3 review; the external R6 source proof remains historical, and the full F1 migration is still open. |
| Package compatibility | YT-284 exact-package qualification remains open. |

## Stop and failure contract

Composition fences HTTP command admission first. Reverse local cleanup closes
GPU Surface before Window, then UDP and its peer reservation, then the audit,
then C1. A separate graceful shutdown signal reaches the independent Web worker;
completion observes its exact original JoinHandle. The finite client also joins.
Fallible UDP client bind, endpoint creation, peer clone and address lookup all
precede acquisition of C1, audit and admission. A real UDP bind-collision test
proves the domain/audit factory was never called. A Web bind collision after
acquiring C1/audit/UDP closes the bridge gate before rolling back those resources.

Draw and resize failure records retain stage, phase, category and the original
public Device error/loss or fatal diagnostic Arc before renderer disposal; a
frame-cleanup diagnostic stays secondary to an earlier cause. Even when
`SurfaceRenderer::close` returns FrameCleanupFailed, its Surface has been
released, so Native still closes the Window and retains the original error.
Completion remains Pending with upstream UDP/audit/C1 owners held.
First failure and secondary cleanup failures remain separate. Pending retains
the original root/worker; a paced completion loop stops retrying a persistent
native blocker after terminal Web Join and successful Window closure. If its finite deadline expires, the
executable reports failure and retains the root until process exit; it never
labels that path Complete. Process exit is not successful application cleanup.
Synthetic audit-close blockers in tests are not induced GPU/driver failures.

The previously reviewed E1 full-queue/in-flight HTTP drain evidence is reused
only for the byte-identical HttpWorker/CommandBridge implementation. The new
external audit-close seam is directly tested with real Web shutdown and retained
original root/Join. Broad old adapter/native/GPU suites were not rerun.

## Initial R1 checkpoint — Ubuntu, 2026-11-20

The following earlier-source receipts are preserved. Independent Reviewer R1
`wc_job_n9F2y1IeAsGzridr` is terminal exit 0 with actual **FAIL5**, not source
acceptance: publication/audit ordering (P1), skipped Window close, missing redraw
reconfiguration, lost driver diagnostic and UDP acquisition order (four P2).
Its original JSON/logs and all earlier run evidence remain unchanged. The
following first-pass results do not certify the repaired source.

- Independent local tests **3/3 PASS**; feature-enabled external tests **7/7
  PASS**, including real HTTP admission versus application, same-C1 UDP/audit,
  strict peer/codec/generation rejection, startup bind rollback and Pending
  audit cleanup. Check/build/Clippy `-D warnings`/Fmt passed offline and locked
  where supported. The original compile failure and its correction are retained.
- Cargo metadata confirms that the external workspace contains only this
  consumer, and the parent workspace excludes it. Its default normal/build graph
  contains only the consumer and `yetlet`. Exact earlier default/no-default
  Yetlet receipts are reused against unchanged package/Core/adapter hashes.
- **One actual external binary run:** Ubuntu X11 `DISPLAY=:0`, NVIDIA RTX4090,
  Vulkan driver 595.99.02; **PID 302442**, Window **0x600002**, exit 0 in about
  2.12 seconds. The HTTP listener, application UDP socket and reserved peer UDP
  socket were independently observed as owned by that PID. Real HTTP 200/202,
  real loopback UDP and Native/audit matched generation **1795142559439681731**,
  revision **2**, paused **false**. There were **107 returned Present calls**.
- The original Web Join succeeded, client joined, root became Complete/Stopped,
  C1 had one Rc owner and no primary/secondary failure remained. Independent
  read-only OS probes observed the Window/process gone, TCP listener absent and
  ECONNREFUSED; both UDP ports rebound. TCP TIME_WAIT was observed separately;
  no immediate raw TCP rebind claim is made. The prior E1 EADDRINUSE observer
  remains preserved as PARTIAL/FAIL at its original identity.
- Executable SHA256:
  `35fa80b648d1f5901f0e8e609d891c5104da83c3d5f28a0f6f2d6ee2ae685ae5`.
  Executed source-map SHA256:
  `e0c4ccfcf9f02c8a418ea90bd6aabff9e17f192c5a88c078d4c84b08a663892b`.
  `smoke-source.json`, `smoke.json`, raw stdout/stderr/exit and a separate
  deterministic `EVALUATION.json` retain the exact observations. The evaluator
  reports PASS; it is not an independent code Reviewer.

## R2 implementation checkpoint — Ubuntu, 2026-11-20

- Local tests **3/3 pass**; three-way library **9/9** and binary **4/4 pass**.
  New focused coverage uses real bridge collision/cross-generation rejection,
  accepted per-revision pinned history, synthetic unavailable/wrong post-publish
  readback with retained immutable history/Pending, real WindowSlot close after
  synthetic Surface cleanup failure, production redraw recovery decision,
  original driver-diagnostic Arc retention and a real early UDP bind collision.
  The existing later Web bind-conflict rollback remains covered.
- The first new feature run exited 101: library 9/9 passed, binary 3/4 passed;
  one assertion wrongly expected Composition to deduplicate a failed close
  retry. Its raw output, source and executable SHA remain preserved. One test
  assertion correction verifies both recorded failures retain the exact first
  GPU Arc. The corrected run passes all 13 tests. No driver fault was induced.
- Changed-target offline/locked all-target check, binary build, Clippy
  `-D warnings` and Rustfmt check exit 0. Test executable hashes and actual test
  source hashes are recorded separately in `repair-r2-test-*-executables.json`
  and the command receipts. Public SDK/adapter sources, feature declarations,
  manifest and own/parent locks remain unchanged.
- Exactly **one new current-source smoke**: Ubuntu X11 `DISPLAY=:0`, NVIDIA
  RTX4090 Vulkan 595.99.02, PID **344341**, Window **0x800002**, exit 0 in
  **2.1067 seconds**. Real HTTP 200/202, loopback UDP, Native and audit agree on
  generation **1795143934573510802**, revision **2**, paused **false**. Pause
  revision 1 and Resume revision 2 each have exactly one audit entry. There are
  **103 returned Present calls**. The HTTP listener, application UDP socket and
  reserved peer socket were observed as owned by this process.
- Original Web Join succeeded and finite client joined. Admission, Surface,
  Window, UDP/reservation, audit and C1 closed in order; root Complete/Stopped,
  primary None, secondary empty, C1 Rc owners 1. Independent OS probes observe
  process/Window gone, TCP listener absent with ECONNREFUSED and both UDP ports
  rebound. Observer TCP TIME_WAIT rows remain separately recorded.
- Executable SHA256:
  `c6980de0b4d2cc39cc36069e54f8d78c83d80375b9bc066ab08032907d98d43f`.
  Executed source-map SHA256:
  `2826633e038212f0925b7224d80139ca976dadc9a0fe7f2a830654afd2715524`.
  New raw `repair-r2-*` receipts, smoke-source map, smoke JSON and deterministic
  evaluator (13/13 observations pass) are separate from all first-run evidence.
  The evaluator is not an independent code Reviewer.

Returned Present proves API return, not GPU completion, scanout or frame rate.
No actual driver loss/OOM, Windows, browser/CSP, physical LAN or F3 qualification
is claimed. This bounded Pause/Resume application does not migrate the full F1
Twin Squares game. It is not full YT-283/F2 closure. The next unblockable action
is the orchestrator's independent R3 source review against the newly sealed
public mutation boundary; formal YT-282 prerequisite and YT-284 package gates
remain open.

## R3 bounded direct-mutation boundary repair — 2026-11-20

Independent R2 `wc_job_ZqOtqD9ZDpP2pYzS` was terminal exit0 but actual **FAIL1/P2**:
public `C1Domain::preflight` and `commit_published` accepted a caller-supplied
candidate as its own readback without ever publishing through `CommandBridge`.
The reviewed R2 result and prior evidence are preserved unchanged.

The external crate now keeps `PreparedApply`, `preflight`, `candidate`, and
`commit_published` crate-private; its audit append-forged-readback test helper
is test-only and private. Public C1/audit **read** methods remain accessible;
`C1Domain::close` was public at the R3 checkpoint; the subsequent R4 repair
makes it crate-private and uses the owning `Ports::close` callback for the
external binary. Neither operation advances a revision. Only `Ports::apply` performs publish, actual pinned
readback and C1/audit commit. A new external `compile_fail` doctest proves the
unpublished direct-commit calls are rejected by Rust visibility.

Latest scoped checks: local **3/3** plus compile-fail doctest **1/1**, optional
three-way **9/9** library + **4/4** binary plus doctest **1/1**, Clippy
`-D warnings`, Build, Fmt PASS. The first Cargo PATH launcher check and the
first visibility-restricted binary compile fail are preserved with their
original error receipts; only the required public `close` was restored. This
is a Rust visibility-only change, not a new renderer or runtime behavior; the
previous source-fenced PID344341 / 103 returned Present integration evidence
is reused with its exact earlier product SHA and does **not** certify a new
runtime binary identity. No additional GUI run or historical tests were
performed for accounting. Independent R3 then returned actual FAIL2/P2; the
bounded R4 repair below is awaiting independent review. Whole YT-283/F2
remain Todo.

## R4 bound Ports ownership and closure — 2026-11-20

Independent R3 `wc_job_nqSx475lm-5uYiDx` terminal exit0 returned actual
**FAIL2/P2**: public mutable `Ports.c1`/`Ports.audit` allowed a second
`Ports` instance to borrow/replace a different bridge's authority and ledger,
while public `C1Domain::close` accepted a closed forged audit with matching
snapshot, bypassing the original admission gate.

The external `Ports` fields are now private; public read-only handle getters
still support the real caller-local `Rc` attachments and status inspection.
The only live domain close entry is its bound `Ports::close` callback, which
requires that same bridge's command admission closed, the attached real audit
closed at the exact snapshot, and outstanding queued commands discarded. The
external binary uses `Ports` as its required C1 local instead of calling
C1Domain::close on a separate Rc. `preflight` and `commit_published` remain
crate-private, so no external caller can advance C1/audit without a successful
real CommandBridge publication and pinned readback.

Two focused tests confirm independent `Ports` cannot advance each other's
history, and a separate closed audit with identical identity cannot stop an
active `Ports` even if that instance's admission is subsequently closed.
Two external compile-fail doctests reject both direct commit and public
reassignment of bound Ports handles. Current tests: local **3/3 + 1 compile-fail**,
three-way **11/11 library + 4/4 binary + 2 compile-fail**, targeted Clippy
`-D warnings`, Build and Fmt exit0. Source and docs changed only in the
external crate and scoped YT-283 status metadata; main Yetlet Core/adapters
remain byte-identical. The previous PID344341 /103 returned Present calls
remain valid as earlier-source runtime behavior, **not** an exact-current-binary
execution. No unrelated older tests or a new GUI run were requested.

**Independent R4 returned actual FAIL1/P2**; R5 remains the next independent
source review. Full YT-282/YT-283/YT-284 and F2 remain Todo. This bounded
external source proof is not a release or physical LAN qualification.

## R5 audit-close admission fence — 2026-11-20

Independent R4 `wc_job_QpcwD_U8wl8yYoqZ` was terminal, exit0, but actual
**FAIL1/P2**. The public `close_audit(&mut Rc<RevisionAuditHost>)` accepted
a clone of the *real* attached audit and closed its ledger while the owning
HTTP admission still accepted valid POST commands (202). C1 then rejected
drained commands as ordinary InvalidData. This was not a healthy shutdown.

The external crate now defines a concrete `AuditAttachment` with **private**
`Rc<RevisionAuditHost>` and the cloned `CommandIngress` of the exact `Ports`
which created it. Only `Ports::audit_attachment()` can construct this bound
attachment, and `close_audit(&mut AuditAttachment)` rejects closure until its
owning `CommandIngress::is_closed()` returns true. Public read-only
`Ports::audit()`/`AuditAttachment::audit()` remain available, but a plain
cloned Audit Rc is no longer accepted by the public close callback. The
standalone local-only library tests remain internal; all actual three-way
Composition required-local callbacks (including startup rollback and
synthetic Pending) use the bound attachment. No Yetlet Core or adapter change.

New focused **real loopback HTTP negative test** attempts close with the
attached Audit Rc while admission remains open, observes the rejection and
`AuditStatus::Open`, then confirms HTTP 202 only admitted the Pause command.
The sole C1 applies Revision1, exact pinned readback and Audit record match;
an additional early close still fails. Closing the original admission
permits Audit close, and then the original `Ports::close` stops C1 at the
matching identity. Existing full-queue/Pending tests remain unchanged in
scope, not mistaken for induced GPU faults.

Changed-source offline locked validation: local **3/3** and doctest **1/1**;
three-way library **12/12**, binary **4/4**, and external compile-fail
doctests **2/2** PASS. Cargo Clippy `-D warnings`, binary Build, Rustfmt,
scoped English/links/fences/whitespace and Git diff check PASS.

Because this repair changes actual shutdown/admission behavior, one bounded
current-source external X11/RTX4090/Vulkan three-way run was performed:
**PID481951 exit0**, matching viewable X11 Window **0x800002**, actual
**95 returned Draw/Submit/Present** calls, exact HTTP/UDP/Native/Audit
generation **1795148457409499440**, revision **2**, paused **false**.
Original Web Join and finite client Join completed; root Complete/Stopped,
process/window/listeners absent, HTTP ECONNREFUSED, UDP endpoint/peer released.
Independent evidence evaluator 13/13 PASS is **not** a code Reviewer.
Run identity: binary SHA256
`63469f236fa11ad0a29f535a3abbd102cfcf2ffa02d397fa05a914950d114368`;
source map SHA256
`a0aea1b3eadd196a43d11f8400b04f3835587ab78cd57eca9075837226caa12f`.
All earlier source/GPU/Reviewer failures remain preserved. Returned Present
does not certify physical scanout or recovery from real driver OOM.

**Independent R5 actual FAIL1/P2 retained; R6 Reviewer pending**.
This is still source-only proof:
whole YT-282/YT-283/YT-284/F2 remain Todo; no Stage/Commit/Push,
F3/physical LAN or release is authorized.

## R6 sealed Web route and shutdown admission — 2026-11-20

Independent R5 `wc_job_8NBMTfRXdgBnDeIG` was terminal exit0 but actual
**FAIL1/P2**: the public Web construction accepted an independently provided
Router A and shutdown CommandIngress B. `close_admission` then closed B
successfully while A's HTTP handler still accepted new commands (202),
leaving A's real audit unable to close and Composition Pending.

External `Ports::web_launch()` now constructs one sealed `WebLaunch` with
private Router and CommandIngress fields sourced from **the same Ports**.
`start_web(WebLaunch, SocketAddr)` cannot accept a mixed router/ingress pair;
the resulting `Web` keeps both the original HttpWorker and ingress private,
with public read-only endpoint/readiness/Join observation accessors. A
`compile_fail` doctest proves externally constructing a mixed WebLaunch is
rejected. No main Yetlet SDK/adapter or Cargo dependency changed.

A new **two-live-worker real loopback HTTP negative test** starts Web A
and Web B from different Ports/generations. After `close_admission(A)`,
POST to A returns **503**, while POST to B still returns **202**.
B's own C1 then applies Revision1, publishes exact pinned history and
records the corresponding Audit entry; A remains Revision0. Each true
admission then closes before its own Audit and Ports C1, and both original
workers Join. Earlier clone-Audit early-close and cross-Ports tests remain.

Latest external standalone Cargo test results: no-default library **3/3**
plus compile-fail doctest **1/1**, optional three-way library **13/13**,
binary **4/4**, external compile-fail doctests **3/3**, Clippy
`-D warnings`, Build and Rustfmt exit0.

Because the normal Web worker construction path changed, one bounded
source-fenced Ubuntu X11/RTX4090/Vulkan run was repeated (not the old
E1/standalone test): application **PID506716**, exit0, X11 Window
**0x800002**, **94 returned Draw/Submit/Present** calls. HTTP/UDP/Native/Audit
all observed generation **1795149091452176037**, revision **2**,
`paused=false`. Original Web worker and finite client Joined; root
Complete/Stopped, no lost first failure, process/window/listener gone,
HTTP ECONNREFUSED and both UDP ports released. Independent deterministic
OS/source evaluator **13/13 PASS** is evidence, not a code Reviewer.
Exact binary SHA256:
`a5e0705fad14324cc9e93d83a84e319d24ba325a62d32eecc9dff262a331a67a`.
Executed product-source map SHA256:
`c9aaeff48c552981eb4dc620c3960c5a3015012b4613cbe9d54bffba00b04ac3`.

Original R1–R5 reviewer FAIL evidence and all previous independent GPU
receipts are preserved. The known historical ignored-only global Docs FAIL19
was not rerun or relabeled. Independent Reviewer R6 actual PASS closes only
this bounded external **source-only proof**, not the full backlog item.
Full YT-282/283/284/F2 and physical LAN/F3 remain open. No Stage/Commit/Push.

## Independent R6 reviewed closeout — 2026-11-20

The saved independent Reviewer `wc_job_RzpUkYN_WzE5OZhT` reached terminal
exit0 with actual `overall_status=pass`, `issues=[]`,
`required_followups=[]`, `optional_notes=[]` on the frozen eight-file source
map canonical SHA256
`17482ad3e59b4a96702331cd7cade6ffc3a218b11fb55b37c679cbd86d128653`.
This was a role-directed Job, not a native managed-agent or formal assurance
trace. Original R1–R5 failures and all distinct GPU/validation receipts stay
available in the ignored evidence directory. No product source, Cargo manifest,
lock, reviewer or GPU test was modified/replayed after R6; only this accepted
source-proof documentation and Backlog/Roadmap metadata were updated.

**Source-only external proof Done** means an independently compiled outside-
workspace Cargo consumer and meaningful `RevisionAuditHost` combine existing
public opt-in Web/Axum, UDP, Winit and WGPU with a single caller-local C1,
correct pinned readback, admission-first shutdown and owned Worker Join.
It does **not** satisfy the full F1 Twin Squares migration, formal YT-282
prerequisite, full YT-283, YT-284 package/public compatibility, F2 acceptance,
release or physical LAN/F3. These remain Todo/open until separately qualified.

## YT-282-G R3 review finding — external aggregate startup gating (2026-11-20)

The independent YT-282-G overall-gate Reviewer R3
`wc_job_EYHdVDKF0L9huUr5` reached terminal exit0 but actual **FAIL1/P2**:
this external consumer's real HTTP command route could previously respond 202
before its required Native Host was Ready. The earlier E1 example had already
fixed its own route; the external router remained a separate ingress. The
original YT-282-G R1 FAIL3, R2 FAIL2 and R3 FAIL1 JSON/source/job evidence
are retained without rewriting or claiming PASS.

The **consumer-owned** `Ports` now binds a narrow `Arc<AtomicBool>` startup
gate to its exact sealed `WebLaunch` and original Web worker/admission.
`GET /ready` still returns 200 for Web readiness. A valid pre-ready
`POST /command` returns 503 without joining the queue or changing the sole
C1/RevisionAuditHost. The actual caller-thread application opens admission
only after `Composition::all_required_ready()` (including the Window/real
rectangle Present); test-only synthetic foreground readiness is explicitly
identified as a fixture, not OS evidence. The same gate denies trusted UDP
application commands until readiness without blocking read-only pinned state.
`close_admission`, startup failure and normal stop revoke the gate. Direct
`Ports::apply` remains a caller-owned domain operation usable by local/headless
tests; it is not an independently accepting transport or a second Simulation.

A new **real loopback HTTP Router** focused regression constructs one typed
Composition with original Web worker and a synthetic required foreground
part: before Native readiness GET `/ready` is 200, the exact valid Move POST
is 503, and a C1 drain leaves revision 0 and Audit empty; after all required
parts are explicitly marked Ready, the identical POST is 202, with revision
still 0 until C1's actual drain commits revision 1 and the Audit record.
The original Web handle Joins after normal stop. Other scoped tests retained
the cross-Ports A-closed/B-ready policy and reserved UDP peer rejection.

Current changed-source qualification: external three-way **18/18 library**,
**7/7 binary**, **5/5 compile-fail doctests**, no-default **5/5 units +1
doctest**, focused corrected-router regression **1/1**, Rustfmt, strict
Clippy `-D warnings`, and binary Build exit0. The first strict Clippy exit101
(a test-only boolean comparison) remains retained as a corrected formatting/
lint failure; the subsequent narrow regression and strict lint are PASS.
No new GPU/Window/physical LAN run was performed. Earlier native Vulkan
PID715103 /97 returned Presents and the former external R6 proof retain
their **earlier-source** identity; they are not byte-identical hardware
receipts for this repair. No Yetlet SDK/adapter, feature manifest, lockfile
or public Host trait was changed.

The bounded YT-282-G R4 independent overall-gate review subsequently
completed: Reviewer `wc_job_iebUf3KFJs9dqYlS` terminal exit0, **actual JSON
PASS**, `issues=[]`, `required_followups=[]`. Whole **YT-282 Source-only
public API gate is Done**. Complete original F1/YTP1 game migration remains
YT-283 Todo, exact package/compatibility is YT-284 Todo, and F2 overall plus
physical D4 remain open. No new GPU run or registry publication is claimed.
All raw receipts and original failed reviews remain under ignored
`.pipeline-output/m28-f2-yt282g-20261120/`.
## YT-283-M1 — Consumer-local deterministic logical Tick/Input source slice (2026-11-20)

Following whole YT-282-G public source-only acceptance, the independent
outside-workspace Cargo consumer adds its first **local/headless** original
F1 simulation migration vertical slice, not a new Yetlet framework feature.
Reference: private `examples/composable-host-research/src/simulation.rs`
`Simulation::step` and native `GameDriver` (both read-only, byte-identical).
Full source-to-public gap matrix is retained in ignored
`.pipeline-output/m28-f2-yt283m1-20261120/MIGRATION-GAP-MATRIX.md`.

The consumer adds `src/tick_m1.rs` and extends its **existing**
`C1Domain`, which still owns the only authoritative `Cell<Snapshot>` of
generation, revision, paused state and two positions. Added copy-only Tick
metadata holds logical `tick`, `active_ticks`, `last_sequence`, and two
`HeldDirection` values, never duplicate game positions. The explicit
`step_tick(tick, &[TickInput])` is one **logical 1/60-second step**, not
a wall-clock runner. It sorts at most 32 stamped inputs by strict contiguous
sequence, rejects wrong generation/tick/batch/overflow atomically, and
advances control tick and revision per accepted step. On active ticks,
two held cardinal directions move the squares by at most one whole grid
cell; Still releases, Pause clears Held, paused Holds are discarded, and
effective Resume clears Held without catching up paused time. All positions
remain clamped to the 24×16 arena's 2×2 footprint.

**Mode safety:** the selected `three_way::Ports::new` binds the **same C1**
to its existing RA1 CommandBridge and RevisionAuditHost before returning.
At RA1 revision 0, public `Ports::c1().step_tick` and
`close_tick_m1` still reject `ModeConflict`; there is no bypass of
existing pinned publication or audit. Once a standalone local Tick C1
has advanced, its RA1 preflight/close are disabled until M2 formally
integrates Tick publication through that existing bridge/audit.
`close_tick_m1` can also stop an **unbound standalone C1 at Tick0**
or after a rejected first input, so actual public `Builder::required_local`
completes without reporting false Pending. The sole local C1 stops and
rejects further Tick after closure. No second C1/Simulation or new SDK
Host/Renderer/Transport trait was introduced.

Original independent M1 Reviewer R1 `wc_job_a1yWjx-YLCxeax7x`
terminal exit0 returned **actual FAIL1/P2**: a pre-first-Tick standalone
C1 could not close. The minimal repair removed only that wrong Tick0
close predicate while retaining RA1-bound and revision-mismatch guards.
New focused tests cover both zero-Tick normal stop and rejected-input stop.
Independent M1 Reviewer R2 `wc_job_b-rok_zrMYYsa0a5` terminal exit0
returned **actual JSON PASS**, `issues=[]`, `required_followups=[]`,
`optional_notes=[]`; original R1 failure and every raw receipt stay
unchanged in ignored evidence.

Final source-bound verification: standalone no-default Cargo unit **12/12**
+ compile-fail doctest **1/1**; selected external three-way library
**26/26** + original binary **7/7** + compile-fail doctests **5/5**;
both Clippy `-D warnings`, Rustfmt and external binary Build exit0.
The first formatting-only nonzero is preserved as an earlier source
receipt; exact-source logs are in
`.pipeline-output/m28-f2-yt283m1-20261120/`.

**Explicit limits:** Tick observations are local-only and are **not yet**
published into the original RA1 Bridge/Audit/HTTP/UDP history. Real
foreground Winit physical W/A/S/D and arrow-key priority/repeat/release,
monotonic 60Hz wake/4-tick catch-up/debt cap, full F1 Tick-bearing status,
YTP1 Session/Epoch/sequence and UDP replica/reconnect are **future
YT-283-M2+**. The existing RA1 20-byte frame, original `RevisionAuditHost`,
Web startup/readiness and Native `draw_rects` still compile and pass,
without any new current-source GPU/Window execution. Whole YT-283,
YT-284 package compatibility, F2 and physical D4 remain open.

## YT-283-M2-A — Atomic Tick publication through the original Bridge/Audit (2026-11-21)

**Bounded source-only slice Done**, independently reviewed. This section
supersedes only the M1 limitation that bound Tick snapshots cannot be
published through the existing Bridge/Audit. Whole YT-283 remains Todo.

The same external `C1Domain` still owns all mutable game positions and
revision. `Ports::step_tick` now publishes a deterministic Tick through its
original `CommandBridge` and `RevisionAuditHost`. The bridge carries one
consumer-owned immutable envelope containing the original Snapshot and an
optional shared `TickAuditRecord`. Its before/applied observations include
the complete TickClock: logical Tick, active Tick, sequence and both Held
directions. The validated input batch is stored in canonical sequence
order. The bridge and audit retain the **same immutable Arc proof**;
`Ports::pinned_tick` returns that exact revision's proof.

### Atomic commit and authority

All input reduction, original audit identity/open/capacity checks, mutable
ledger borrowing, slot reservation and proof allocation happen before
publication. The original Bridge's single write is the external
linearization point. If it rejects, C1 Snapshot, TickClock, audit entries
and immutable history remain unchanged. After successful publication,
the private prepared commit performs only a reserved Vec push and Cell
copies; it has no fallible readback, callback, logging or allocation.

Concurrent SnapshotReader workers see the complete immutable envelope.
C1 and Audit remain caller-local Rc/Cell/RefCell owners, so those workers
cannot inspect a half-updated local authority. No SDK API, second Bridge,
C1/Simulation, Supervisor copy or generic Host/Renderer trait was added.

Public aliases to a bound C1 still reject local `step_tick` and
`close_tick_m1`; only its owning Ports may publish. The first **accepted**
RA1 action or Tick selects the exclusive mode; a failed first Tick does
not latch it. Existing RA1 commands are not translated into Held inputs.
RA1 `records()` remains the original audit projection;
`tick_records()` and `pinned_tick()` expose accepted Tick evidence.
The same admission-first, original audit-before-C1 close now checks both
Snapshot and TickClock identity, including zero/rejected-first Tick close.

### Verification and preserved behavior

Independent saved Reviewer R1 `wc_job_vN1Yd0c1Kq-mXG0_` completed
with terminal exit0 and **actual JSON PASS**, `issues=[]`,
`required_followups=[]`, `optional_notes=[]`. The reviewed canonical
product SHA256 is
`68eec0aa1ba81be0090f5f9359813cac940e9d9871c93fe1dc7086acddb25d0d`.
Review is a role-directed Codex Job, not native managed-role or formal
assurance trace. Product source did not change after that review.

The current external default-off target passes **12 unit +1 compile-fail
doctest**; three-way passes **34 library +7 binary +5 compile-fail doctests**.
Both strict Clippy configurations, Fmt and actual selected binary Build
exit0. Eight added tests cover canonical same-proof publication, prior
revision immutability, input/borrow failures and retry, real bridge
stale/cross-generation rejection, finite ledger capacity, mode isolation,
concurrent copied readers, original Composition close, and real loopback
HTTP/UDP reads with original Web stop/Join. Original feature-off Clippy
exit101 and its minimal cfg repair remain preserved.

The RA1 wire format remains **20 bytes**, and its HTTP/UDP Snapshot text
is unchanged for both publication modes. Original Native main.rs,
manifests/lock, public SDK/adapters and private F1 source are unchanged.
Prior RA1 postpublication readback failure/Pending semantics remain their
existing contract; the Tick path does not use that fallible finish step.
Detailed commands, source identities, raw logs and independent result are
in ignored `.pipeline-output/m28-f2-yt283m2a-20261121/`.

### Remaining bounds and next slice

The original audit still accepts only **16 entries**: Tick17 is rejected
atomically and consumes no input sequence. The Bridge retains the latest
16 revisions. Before an ongoing Native scheduler, the next bounded
YT-283-M2-B should define fixed-capacity Tick audit retention/eviction,
expired pinned lookup semantics and no-partial-eviction failure behavior,
while preserving RA1's existing finite-ledger contract.

Actual Native keys/60Hz scheduling, tick-bearing network protocol,
YTP1 Session/Epoch, Replica/Reconnect, whole YT-283 gameplay migration,
YT-284 package/compatibility and F2 remain open. No new GPU/window or
physical LAN verification occurred; older GPU receipts remain valid only
for their historical sources. No Stage/Commit/Push.

## YT-283-M2-B — bounded recent Tick evidence (2026-11-21)

**Bounded source-only slice Done**, independently reviewed. This section
supersedes only M2-A's finite Tick17-stop limitation. Whole YT-283 remains Todo.

The existing `RevisionAuditHost` retains the most recent **16 accepted Tick
proofs**, in oldest-to-newest order. The same consumer-owned
`TICK_EVIDENCE_CAPACITY` selects the original Bridge's history capacity.
At successful Tick N >=16, both windows contain revisions N-15 through N;
the initial non-Tick Revision0 leaves Bridge history after Tick16.
Logical Tick/Revision, active Tick and lifetime input sequence keep their
existing meanings and never reset when evidence is evicted.

### Publication and eviction remain one transaction

The existing private `PreparedTick` still holds the exact audit ledger's
mutable borrow from preparation through publication and commit. Below
capacity it reserves one slot before publishing. At capacity preparation
leaves all16 old proofs untouched and does not reserve a seventeenth slot.

Only after the original Bridge successfully publishes the complete immutable
envelope does the local commit remove the oldest proof, reuse that Vec slot,
and update the same Audit/C1 Snapshot and clock. This bounded shift moves at
most15 entries. No new ledger abstraction, second Bridge/C1, SDK method,
private Supervisor copy or Host/Renderer trait is introduced.

Input, authority, audit borrow/lifecycle/identity and real Bridge publication
rejections preserve the original evidence and counters. The successful
postpublication commit has no recoverable failure, allocation, callback or
readback. The original M2-A reader argument still applies: concurrent
Bridge readers see the complete immutable proof; local C1/Audit cannot be
observed concurrently between the Bridge write and local completion.

### Exact query and latest Audit semantics

| Read | Current contract |
|---|---|
| `tick_records()` | Chronological latest min(accepted Ticks,16) immutable proofs; its length is not a lifetime counter. |
| `audit.pinned()` / `audit.pinned_tick()` | Latest accepted Snapshot/full Tick identity, preserved through eviction and close; Tick identity is absent before the first accepted Tick and in RA1 mode. |
| `Ports::pinned(generation, revision)` | Exact retained Snapshot; expired, absent or wrong-generation identities return NotFound. |
| `Ports::pinned_tick(generation, revision)` | Exact retained Tick proof; expired, absent, wrong-generation or non-Tick identities return NotFound, never the latest substitute. |
| Already returned Arc/copied proof | Remains valid and unchanged after internal eviction. Holding it does not prevent eviction or keep its revision queryable. Caller-owned handles may retain old evidence outside the bounded histories. |

RA1's original finite16-entry ledger and20-byte protocol are unchanged.
The first accepted mode remains exclusive even after repeated Tick eviction;
direct bound-C1 mutation still cannot bypass Ports. Original
admission -> Audit -> C1 close preserves the final identity and retained proofs.

### Focused verification and independent review

Seven focused `three_way::tests::tick_retention_` cases pass on the reviewed
source, with29 existing tests filtered out. They cover49 accepted Ticks with
two-player Held, empty inputs, Pause/Resume and sequence continuity; exact
window/Arc identities; old proof immutability and expired queries; full-window
input/borrow and genuine Bridge rejection without premature eviction; retry
of the same next input; and RA1's unchanged finite limit and mode isolation.

A real reader thread crosses33 publications and joins. Original Composition
close completes at zero, rejected-first, one accepted and33 accepted Ticks.
Real loopback HTTP and RA1 UDP return the exact Revision33 Snapshot, then
original admission/network/audit/C1 close and Web Join complete. No GPU,
visible window, physical LAN or realtime60Hz scheduler was run.

Strict default-off and three-way Clippy for all targets, plus Fmt, exit0.
Unchanged M1 reducer, public SDK/adapters, private F1, binary and prior GPU
evidence are reused only within their original source scope; unrelated
test suites were not rerun. The first six-case development receipt remains
preserved; the final seven-case receipt includes explicit RA1 regression.

Independent saved Reviewer R1 `wc_job_NQ0a8oXFyKSHF-j3` completed
with terminal exit0 and **actual JSON PASS**, `issues=[]`,
`required_followups=[]`, `optional_notes=[]`. The reviewed canonical
product SHA256 is
`86063c2692c171f02cdfaae59fb5c6886374daac43ca5d1c1a505ea1b61fe2e6`.
This is a role-directed Codex Job, not native managed-role or formal assurance
trace. No product source changed after review.

Evidence lives in ignored `.pipeline-output/m28-f2-yt283m2b-20261121/`.
All2718 prior ignored files, including original failures, remain byte-identical.
Only the existing external lib.rs/three_way.rs and three status/evidence docs
changed; HEAD363dde2,42 dirty,staged0,uncommitted/unpushed.

### Next bounded candidate

YT-283-M2-C should prepare consumer-owned physical key bookkeeping and a
bounded pending Tick input batch by comparing private F1's map_key,
GameDriver::key, directions and lost_focus. Explicit caller-driven next-Tick
publication through the same Ports can validate that mapping headlessly.
The adapter must not own another Simulation or copy the private Supervisor.

Actual Window keyboard delivery and monotonic60Hz scheduling remain separate
future acceptance. YTP1 Session/Epoch, Replica/Reconnect, whole YT-283,
YT-284 package/compatibility and F2 stay open. M2-C is a recommendation only;
no next-slice implementation or Git delivery is authorized by this completion.

## YT-283-M2-C — Native key preparation through original Tick Ports (2026-11-21)

**Bounded source-only slice Done**, independently reviewed. This increment
adds consumer-owned input preparation only. Actual Window event delivery and
realtime60Hz scheduling remain separate.

### Consumer API and owner boundary

The existing optional three-way feature now exposes the external consumer's
`native_input::{NativeInput, Key, KeyResult, map_key}`. Mapping a Winit KeyCode
is pure and opens no Window. The default-off dependency graph, original
binary/--smoke, public SDK and adapter implementations are unchanged.

`NativeInput::new(&ports)` attaches to the exact live, nonzero-generation
Ports only at Revision0/default TickClock, before the first accepted RA1
action or Tick. It stores eight physical key bits, at most32 pending
TickCommand values, a Weak identity to the original C1, and an accepted
revision validation token. It owns no Snapshot, positions, clock, Simulation,
worker, renderer or scheduler.

Each key/focus/submission call takes that same Ports. Another Ports with the
same generation is rejected by identity, and an intervening successful
advance by another caller makes this adapter stale. Rejection keeps pending
input intact; no implicit rebase, rebind or physical-state reconstruction is
offered. Preparation itself does not select RA1/Tick mode. Only the existing
first successful Ports operation selects the mode.

An explicit caller can prepare and publish headlessly:

```rust
use yt283_revision_audit::{
    native_input::{Key, NativeInput},
    three_way::Ports,
};

fn main() -> std::io::Result<()> {
    let ports = Ports::new(91)?;
    let mut input = NativeInput::new(&ports)?;
    input.key(&ports, Key::W, true, false)?;
    input.key(&ports, Key::Up, true, false)?;
    let tick1 = input.step_tick(&ports)?;
    let tick2 = input.step_tick(&ports)?; // held movement, no new physical event
    assert_eq!(tick2.clock.tick, tick1.clock.tick + 1);
    Ok(())
}
```

This is the existing headless Ports path. It does not open external Web/UDP
startup admission or substitute for required Host readiness.

### Private F1 semantics retained

| Event or state | Prepared behavior |
|---|---|
| WASD / arrows | Player One / Player Two; independent fixed N > S > W > E priority. |
| Repeat flag | Unchanged for every key, including Pause/Resume/Exit; no additional command. |
| Duplicate movement state | Unchanged; a real shadowed-key change is remembered even when it adds no command. |
| Space / Enter / Escape | Pause / Resume / exit request; control releases are ignored. Exit is only a caller result and does not close resources or enqueue a Tick command. |
| Pause press | Admit Pause before clearing physical keys. |
| Resume press | Fold published paused state with already pending Pause/Resume. Clear keys only for an effective Paused -> Running transition; another non-repeat Resume still queues its command. |
| Held input while paused | Existing reducer consumes it without movement; effective Resume requires a fresh press. |
| Focus loss | Preflight all needed Still commands, append the complete batch, then clear both players' keys. |
| Full32 queue | WouldBlock leaves pending contents and keys unchanged. A physical change requiring zero new commands may still succeed. Rejected focus loss needs caller retry after capacity becomes available. |

Only input bookkeeping semantics are migrated from private F1 map_key,
directions, GameDriver::key/lost_focus/step. Its owned Simulation/Supervisor
is not imported or duplicated.

### Publication remains authoritative and atomic

Explicit step_tick checks the exact owner, accepted revision, open admission,
live C1/Audit and retained publication-failure gate. It derives the checked
next Tick and contiguous lifetime sequences from the original C1 observation,
stamps the pending commands, then calls only the existing
`Ports::step_tick`.

The pending queue is cleared and its revision token advanced only after that
original atomic publication/commit returns success. There is no new direct
C1 mutation, injected publisher callback, second bridge or fallible work after
successful publication. Empty batches can advance held movement. Capacity,
identity, lifecycle, stamp overflow, audit-borrow and real bridge publication
failures leave local input and authoritative state unchanged for that attempt.

M2-B's latest16 Audit/Bridge retention, exact expired pinned-query NotFound,
old immutable proof handles and latest Audit identity stay intact. RA1 keeps
its original finite16 ledger,20-byte protocol and exclusive mode. Original
three-host behavior and --smoke are preserved through unchanged source.

### Focused source evidence

Twelve new `native_input`-filtered library tests pass;36 existing tests are
filtered out. They exercise all mapped keys and both players' priority/
release/repeat behavior, shadowed keys at full capacity, pending control
projection, effective Resume rejection, paused Hold, atomic two-player
focus loss/retry, exact owner and stale revision rejection, RA1 mode selection,
original audit-borrow rejection/retry and checked Tick/sequence exhaustion.

A genuine original CommandBridge rejection fixture after16 successful Ticks
preserves pending input and all old Audit identities. Restoring the original
bridge allows the same batch to publish exactly once as Tick17/sequence1,2;
only success consumes it and evicts the oldest proof. Separate key-driven
execution reaches33 explicit Ticks with monotonic counters, old proof
immutability, expired queries and original typed Composition close. Every
accepted helper step checks the same C1, pinned Snapshot, Audit observation
and exact shared immutable proof.

Strict default-off and three-way all-target Clippy plus Fmt exit0. No
unrelated tests, doctests, binary smoke, GPU, Window, realtime scheduling or
physical LAN were rerun. The unchanged M1 reducer/M2-A publication/M2-B
retention, RA1, SDK/adapters and three-host evidence retain their prior scope.

Independent saved Reviewer R1 `wc_job_EwcffKLCqR7HA64E` completed with
terminal exit0 and **actual JSON PASS**, `issues=[]`,
`required_followups=[]`, `optional_notes=[]`. It inspected the actual source,
private reference, exact M2-B delta and raw receipts in13 read-only commands.
The reviewed canonical product SHA256 is
`faeeeea6311df5ff2d8c980909c7f1498de33a7ecabdc3b2bc37ee8ec52ef925`.
No product source changed after review; only completion documentation follows.

Raw receipts and the private-reference mapping are in ignored
`.pipeline-output/m28-f2-yt283m2c-20261121/`. All2769 prior ignored evidence
files remain byte-identical. Only the new native_input.rs, existing lib.rs
module export, one added three_way.rs test and three status/docs files changed.
HEAD363dde2,43 dirty,staged0,uncommitted/unpushed. Whole YT-283 remains Todo.

### Next bounded candidate

M2-D can prepare consumer-owned monotonic60Hz scheduling policy headlessly,
with explicit time inputs, bounded catch-up and no Pause/Resume catch-up,
driving this same NativeInput/Ports path. Actual Window keyboard delivery
and a running native timer remain a later integration slice. YTP1
Session/Epoch, Replica/Reconnect, full YT-283 gameplay migration, YT-284
package/compatibility and F2 overall stay open. No next implementation or Git
delivery is authorized by this status record.

## YT-283-M2-D — monotonic scheduling preparation through original Tick Ports (2026-11-21)

**Bounded source-only slice Done**, independently reviewed. Saved Reviewer R1
wc_job_c-kDYbb4TcI5_no0 reached terminal exit0 and actual JSON PASS:
issues=[], required_followups=[]; three optional notes explain the accepted
scope and fixture evidence limits. The canonical reviewed product SHA256 is
64b58f0955790fcb2cf899046ba15a0ff9c36e6ca92a5c89ce08282fa57dcdff.

The Reviewer independently inspected current source, exact delta, raw receipts,
hashes and prior-history preservation. This was a role-directed Job, not native
managed-role/formal-assurance tracing; configured model/requested effort are
not claimed as structured runtime telemetry. Product source is unchanged after
review. All2828 prior ignored files and37 untouched dirty sources are preserved.
HEAD363dde2,44 dirty,staged0,uncommitted/unpushed. A prior document-only EOF
whitespace failure and its corrected check remain retained. Whole YT-283 stays
Todo; this slice does not run a timer or receive Window events.

### Private F1 comparison and explicit consumer policy

The private native TickClock uses an integer step of16,666,666ns (1e9/60),
MAX_ELAPSED100ms and MAX_TICKS_PER_WAKE4. Its advance method debits due work
before the caller steps the game. Its overload branch drops the fractional
remainder without including that fraction in dropped_ns. Its backwards-time
saturating subtraction still moves last backwards. Actual F1 clock reset
callers are Window resumed/suspended; GameDriver Pause/Resume does not reset
the native clock. These are reference facts, not copied implementations.

The consumer keeps the same step and bounds, rejects backwards observations,
counts every discarded nanosecond, and debits work only after successful
original publication. Game-control reset is an explicit consumer policy:
an admitted Pause, or effective Paused -> Running Resume anywhere in the
pending batch, requests debt reset after that control Tick succeeds.
Pause then Resume within one batch still requests reset. Running repeated
Resume, key repeat and control-key release do not reset scheduling.

The detailed mapping is in ignored
.pipeline-output/m28-f2-yt283m2d-20261121/MIGRATION-GAP-MATRIX.md.

### API, bounded time and atomic publication

The optional consumer module native_clock exposes NativeScheduler, copied
ScheduleState diagnostics, PumpReport/PumpError and the three constants.
Construction binds one existing NativeInput to the original initial Ports.
The scheduler owns only that input adapter and timing metadata: observed_at,
at most4 pending_ticks, a fraction smaller than FIXED_STEP, lifetime overload
dropped_ns and separately counted successful-control reset_ns. It stores no
Snapshot, game positions, independent simulation clock, mutable authority,
new Bridge or Audit. No SDK API, dependency or manifest changes.

Callers explicitly pass an Instant to pump. key/lost_focus forward to the
same existing input preparation. No mutable input handle, public reset,
rebase or replacement API can bypass the policy. Before sampling, pump
checks the exact original C1 owner, accepted revision and lifecycle even
when no Tick is due. Backwards time rejects without moving the watermark;
same-Instant retry is legal. next_tick_in is zero for owed work and otherwise
FIXED_STEP minus the fractional remainder; it is a scheduling hint, not a
timer or proof of liveness.

Sampling clamps each newly observed interval to100ms. Previously admitted
unpublished Ticks have priority; new elapsed time can fill only the remaining
slots up to4. Excess newly due work and the fraction in that overload branch
are discarded and counted exactly. For260ms from an empty scheduler, four
Ticks publish and193,333,336ns are discarded; this intentionally includes
the4ns the private reference's counter omits.

Each pending Tick calls only NativeInput::step_tick -> original Ports.
The first successful Tick consumes the original pending input batch once;
later catch-up Ticks use empty batches and the same authoritative Held state.
Publication failure retains failed/unperformed admitted work and pending
input. Sampling already admitted time and overload discard remain recorded;
they are not successful Tick consumption. A retry at the same Instant adds
no time and does not double-count discard.

Atomicity remains **per Tick**. If one Tick publishes and a later Tick fails,
PumpError.published_ticks reports the earlier success; it is never rolled
back or replayed. All fallible first-Tick/reset accounting is preflighted
before publication. Only infallible copies/debits follow each successful
original Bridge/Audit commit.

After a successful reset-requesting control Tick, remaining owed work and
fraction are cleared into reset_ns. A failed control publication clears
neither. Paused logical Ticks may still publish while movement and active_ticks
remain frozen; effective Resume publishes one control Tick and discards
remaining debt, preventing that pump's movement catch-up. For an initially
empty scheduler, elapsed time is accounted as published Tick durations plus
outstanding Tick durations, fraction, dropped_ns and reset_ns.

### Focused verification and evidence limits

New native_clock-filtered tests pass **15/15**, with48 unrelated library
tests filtered out. Two existing Resume tests directly affected by the shared
pending-control projection each pass1/1. Strict default-off and three-way
Clippy for all targets, plus Fmt, exit0. All six validation commands passed
on the first run; source hashes were unchanged during validation.

Coverage includes exact thresholds/fractions, cap/exact discard, backwards and
same-time observations, old owed work under new overload, Pause/Resume and
net-zero control pairs, repeated running Resume, key/release/focus forwarding,
overflow preflight, exact owner/stale/RA1/closed rejection before zero-work
success,33 scheduled Ticks, immutable old proofs, expiration/latest16 evidence
and original Composition close.

A real rejecting CommandBridge test begins with the full16-proof window.
Both ordinary input and Pause/Resume batches preserve C1/Audit, all old Arc
proofs, pending input and four owed Ticks on failure; restoring the exact
original bridge allows same-time retry without premature eviction or duplicate
input sequence. A separate close-after-failure test preserves unperformed work
while closing the original authority normally.

The later-Tick partial-progress test uses a **private initial active-counter
boundary fixture** (active_ticks=MAX-1 at Tick0), not a coherent public gameplay
history. Its second Tick fails in reduction before publication. It demonstrates
reported earlier success, retained owed work and a legitimate Pause retry,
not a genuine second Bridge write failure. No production fault injection
callback, generic publisher or second Simulation was added.

### Preserved boundary and next candidate

RA1 remains20-byte with its finite16-entry ledger; accepted RA1/Tick modes
remain exclusive. The Native --smoke binary, all three Host implementations,
SDK/adapters, manifests/locks, M1 reducer and private F1 sources are unchanged.
M2-B retention and M2-C input acceptance evidence are reused within their
original scope; no unrelated old tests or GPU/Window/physical LAN runs occurred.

A bounded M2-E candidate can wire an explicit opt-in
Tick mode into the existing Native application's key/focus and wake callbacks,
define resumed/suspended reset and failure/stop handling, and verify the wiring
headlessly without launching a Window or realtime loop. This is a proposal,
not authorization for another implementation. YTP1 Session/Epoch,
Replica/Reconnect, full YT-283, YT-284 package/compatibility and F2 remain open.
No Stage/Commit/Push.

## YT-283-M2-E — opt-in Native Tick callbacks and lifecycle (2026-11-21)

**Bounded Source-only slice Done, independently reviewed.** Saved Reviewer R3
wc_job_xx_xLnOdmeet0MOa reached terminal exit0 with actual JSON PASS,
issues=[] and required_followups=[]. The three optional notes preserve scope,
verification and history facts. Reviewed product SHA256:
2baf16f36a1c16ca71629e2c2cc2fe2d40155c554710b162901a73635af7cefa.
Product source is unchanged after review. Only completion documentation follows
that reviewed source fence. Original R1
wc_job_hCaNqUWPzIgj0Xvx and R2 wc_job_H2P6OspMyvOCcYuX each reached
terminal exit0 with actual FAIL2/P2. Minimal repairs restore the original
post-EventLoop smoke and startup-failure shutdown timing boundaries and correct
the Tick-mode UDP availability claims in both the guide and source comment.
Both original results, sources, raw events and receipts remain preserved.
This bounded slice connects
the existing Native Application to the original consumer NativeScheduler,
NativeInput, Ports, sole C1, CommandBridge and RevisionAuditHost. It introduces
no second Simulation, generic Host/Renderer trait, copied private Supervisor,
SDK API, dependency or manifest change.

### Explicit mode and callback ownership

The binary now requires exactly one explicit mode, `--smoke` or `--tick`;
missing, unknown, duplicate or conflicting arguments fail before acquiring
resources. The existing finite RA1 `--smoke` proof remains its own mode.
The opt-in Tick mode is an interactive Native source path, not another smoke
test: its normal close does not require RA1 revision2 or a finite test client.
No `--tick` Window or realtime execution was performed for this slice.

The same typed Composition root still owns the original Web worker, foreground
UDP Host, Native WindowSlot/SurfaceRenderer, C1 and Audit. Tick mode does not
start the finite RA1 client. It leaves the existing RA1 command-readiness gate
closed, while keeping the original command ingress open for Tick publication.
HTTP command POST therefore returns503 in Tick mode; HTTP is the available
snapshot read path. The original UDP Host still polls its reserved trusted
peer, but Tick mode starts no client for that peer and drops the unspawned
client socket. Arbitrary UDP senders remain rejected and cannot read snapshots.
Smoke mode retains Ready-gated RA1 admission, the original foreground drain
path and its two-second minimum/ten-second proof budget starting after Host
preparation and successful EventLoop creation. An already-stopping startup
also receives its original full three-second completion budget at this loop
entry boundary; initial completion/redraw timing is reanchored there. RA1 is still20-byte with a finite16-entry
ledger; Tick uses the separately reviewed latest16-proof retention policy.

Only events matching the currently owned WindowId can prepare input or request
an interactive close. Physical key code, pressed/released state and repeat
are forwarded to the existing NativeInput mapping: WASD/arrows, N>S>W>E,
Space Pause, Enter Resume and atomic focus-loss release. Escape press without
repeat requests normal Tick-mode stop, including before the first Ready.
Window close/destroy also requests normal Tick-mode stop. Other keys are
ignored until the first scheduler exists and while it is suspended.
Callbacks do not own or mutate a second game state.

The application creates its one scheduler lazily at the first fresh aggregate
Ready, after a returned Native Present, using that wake's Instant. Startup
work is excluded from elapsed scheduling time. Existing wake callbacks call
that same scheduler's pump. ControlFlow uses its next-Tick hint, bounded by
the existing12ms maintenance interval; shutdown uses a paced maintenance wake
even if unpublished Tick debt remains. This is source wiring, not an observed
timer accuracy or liveness result.

### Native lifecycle, game control and failure policy

Native suspension and game Pause are distinct. The scheduler's suspend/resume
API checks the exact original owner, accepted revision and lifecycle before
changing state. Backwards time and checked-counter overflow reject without
moving timing or input. Successful Native suspend discards only the unadmitted
interval and fractional remainder into separate `lifecycle_ns` accounting;
already admitted unpublished Ticks remain owed. It atomically appends necessary
Still releases through the existing bounded32 input queue. If both releases
cannot fit, neither physical-key state nor timing is rewritten.

The renderer is closed/dropped before the existing WindowSlot is suspended,
and the prior Present receipt is cleared. Duplicate suspension is inert.
A duplicate active resumed callback does not replace a live renderer or reset
the clock. A true resumed callback recreates Native resources through the
existing path, but timing resumes only after a fresh matching Present and
aggregate Ready. Resume reanchors the same scheduler and retains its owed
Ticks, pending input and accepted-revision fence; it does not unpause the game.
Duplicate active scheduler resume does not move its watermark.

`lifecycle_ns` records excluded Native-inactive time and discarded fractions,
separately from overload `dropped_ns` and successful game-control `reset_ns`.
Unpublished debt is never charged to lifecycle discard. Existing M2-D
successful Pause/effective Resume reset semantics remain unchanged.

The scheduler retains explicit same-time retry after publication failure,
including pending input, admitted work and the complete PumpError progress
count. The **Application policy is fail-and-stop**: it retains the complete
PumpError as the primary cause and starts the original bounded Composition
shutdown; it does not automatically retry callbacks after a failed pump.
Earlier successful Ticks in that pump remain committed. Queue-full input or
suspend release, lifecycle validation, Native/GPU or worker failures use the
same stop path. Pending input and unpublished Tick debt remain inspectable;
neither is cleared merely to make shutdown appear successful.

Normal Escape/close and error stop use the original root's admission-first
request_stop/fail_required path before further C1 work. Once stopping begins,
new key/focus/wake/suspend/resume work is ignored and Native resources are not
recreated. The existing original Web Join, Surface-before-Window cleanup,
Audit close, endpoint release, C1 stop and retained Pending ownership rules
remain in force. Tick0 and long-running Tick mode can close normally without
applying an extra Tick or satisfying the unrelated finite RA1 proof.

### Focused verification and limits

Ubuntu source/headless validation passed all seven command receipts:

| Check | Actual result |
|---|---|
| New Native lifecycle tests, `native_lifecycle_` | 5/5 PASS;63 unrelated library tests filtered |
| New binary mode/callback tests, `native_tick_` | 7/7 PASS;7 unrelated binary tests filtered |
| Existing partial-progress/retry regression | 1/1 PASS; exact test selected |
| Existing overload/exact-discard regression | 1/1 PASS; exact test selected |
| Strict Clippy, no default features | exit0 |
| Strict Clippy, selected three-way feature | exit0 |
| Consumer Rustfmt check | exit0 |

These are the initial seven R1 validation commands. After each review repair,
the changed binary's seven selected tests and three-way Clippy passed again.
The post-R2 Fmt check first exited1 for one chained assignment's line wrapping;
that raw result is retained. The exact formatter-requested whitespace-only
correction was applied and Fmt then exited0. Passing tests/Clippy are reused
for the same Rust tokens; the unchanged library/default-off checks were not
repeated. The restored post-EventLoop timing boundary is source-inspected and
compiled, with no runtime smoke claim.

The five lifecycle tests include real Audit-borrow publication rejection,
same-time retry with owed work preserved through suspension, exact inactive
time accounting, full31-command/two-release rejection, backwards/counter
overflow preflight, owner/stale/closed fences and game Pause versus Native
resume. The binary tests exercise the production mode helpers and Application
advance/stop/suspend paths with controllable time, actual original Web workers
and owned loopback UDP resources, HTTP503/202 mode isolation, original Join and
Composition Complete/Stopped. Tick33 confirms latest16 retention, expired
pinned reads and immutable old Arc proofs.

Native readiness and WindowId in these tests are explicitly **synthetic
fixtures**. No EventLoop, OS Window, Surface or GPU was created. Raw Winit
callback delivery, actual device lifecycle faults, visual Present, realtime
60Hz accuracy and physical LAN were not tested. The application failure test
closes the real ingress before the failing wake; its failure is before time
admission. The separate lifecycle test fails publication under an actual Audit
borrow after debt admission. The reused M2-D partial-progress test still has
its documented private active-counter boundary fixture and is not a second
Bridge-write fault test.

Receipts, exact changed-source identity and preservation records are retained
under `.pipeline-output/m28-f2-yt283m2e-20261121/`. M2-B/M2-C/M2-D and earlier
RA1/three-Host evidence are reused only within their unchanged contract scope.
No unrelated old test suite or known historical global Docs failure was rerun.
Whole YT-283 gameplay migration, YTP1 Session/Epoch, Replica/Reconnect, YT-284
package/compatibility, F2 overall and F3/physical LAN remain open.
No Stage/Commit/Push.

## YT-283 final Native Window Host integration (2026-11-21)

**Bounded YT-283 public-source + real Native integration Done, independently reviewed.**
The maintainer clarified that Yetlet is a thin Window/Host composition SDK;
a possible Game Engine would be separately developed. The existing external
`--tick` consumer uses only public Composition/Native/WGPU/UDP/Web adapters,
one caller-local C1 and its real `RevisionAuditHost`. A small opt-in
`YT283_NATIVE_EVENT_PROBE` logs original Winit input in the consumer only;
no Yetlet Input/EventLoop API, mandatory scheduler or engine abstraction changed.

One identified Ubuntu X11 `:0` RTX4090 Vulkan process PID3054131 (Window
`0x3a00002`, initial size640×420) ran current binary SHA256
`2edd222e870014f991cb25638840bdc30a0af9fca178b8950292ddc2917d5ff2`
and exited0 through actual Escape. Winit observed both players' physical
key Down/Up, mouse motion/left button/wheel, focus loss/gain and resize to
720×460. Nine Window-cropped PNGs show the original two colored rectangles;
pixel readback confirms blue center (80,78)→(373,78) after D and orange
(560,341)→(560,52) after Up. Paused+D leaves both boxes unchanged; Enter
resumes. Mouse inputs left both scene positions unchanged.

The owned Web `/ready` returned200; RA1 POST returned503 in Tick mode,
as required. The same process held its HTTP listener and two loopback UDP
sockets. On close the actual Renderer reported **11,631 returned
Presents**, Surface-before-Window close, UDP reservation release, and original
Web Join `Succeeded`/pending_handle=false. Audit and C1 both closed at exact
generation1795242741520676058/revision11630/paused=false/positions
[(22,2),(20,1)]. `EXTERNAL_FINAL complete=true phase=Stopped`, primary=None,
secondary=[]. Independent OS probes found the PID, target XID and three
listeners/ports gone. Original Job `wc_job_ONqHb_9EyfNGziJH` terminal exit0.

Current 7/7 selected binary tests, strict Clippy, Fmt and offline locked build
passed. Original M2-E focused owner/revision/immutable Bridge evidence and
previous real RA1 UDP/HTTP proof are reused only for unchanged contracts.
Latest-run HTTP `/snapshot/{generation}/{revision}` was **not** directly
queried; it is not falsely claimed. RA120-byte and Tick latest16 evidence
retention are unchanged. All actual Job/OS/input/cropped image evidence is
retained under `.pipeline-output/m28-f2-yt283-final-20261121/`.

Independent Reviewer R1 `wc_job_trL0VLA-wKRTVtPM` inspected the exact
consumer source, preserved 44 prior dirty paths and 3,076 ignored artifacts,
original X11/Pillow/job evidence and focused tests. Its terminal exit0 and
actual JSON `PASS`, `issues=[]`, `required_followups=[]`, accept YT-283 at
this thin-Host scope. Reviewed canonical product SHA256 is
`53472daf003939d8fe59fca1979f5a559c6a5f6ad7335b591c44ffcfc7158532`;
product source is unchanged after review. Whole YT-283 is Done; only completion
status documentation changes follow the review.

This qualifies functional Window/input/renderer/Host lifecycle, **not**
precise60Hz/jitter, actual GPU scanout, new DPI scaling, cursor grab,
relative DeviceEvent, OS suspend/resume, GPU fault injection, YTP1,
Replica/Reconnect or physical LAN. These game/transport-specialized concerns
are not the YT-283 thin-Host gate. YT-284 exact package compatibility and
F2 final delivery remain separate; no Stage/Commit/Push.
