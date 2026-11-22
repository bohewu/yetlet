# M28-F2 / YT-282-E1 — Public three-way Composition consumer

Status: **YT-282-E1 source-only Done**, after independent Reviewer R2
`wc_job_A-G7FIs1j2jWPuyW` terminal exit0 and actual JSON **PASS** with
`issues=[]` and `required_followups=[]`. The original R1 **actual FAIL2**
(`wc_job_3X3Dtuan09eLH0tP`) and all historical source/evidence are retained.
The external raw TCP observers remain **PARTIAL/FAIL** as qualified below;
full YT-282/F2, YT-283/284 and physical D4 remain open.

The current [example](../crates/yetlet/examples/composition_three_way.rs) consumes
public `yetlet::{composition,native_winit,renderer_wgpu,udp,web,command_bridge}`,
Winit, standard Rust and native Axum APIs. YT-293 relocates the consumer after
the canonical modules moved to yetlet. The execution evidence below retains its
original source, package names and bounded qualification; this import migration
does not claim another graphics run.
It follows the [accepted scoped ADR0013 direction](adr/0013-composable-application-and-hosts.md#accepted-scoped-f2-decision--2026-11-19).
The historical YT-281 signatures and private F1 code are not compiled consumer
interfaces. No private implementation is imported, included or copied.

## Build and run

From this checkout, with the existing Rust toolchain and supported X11/Vulkan
hardware available:

```sh
cargo build --offline --locked -p yetlet --no-default-features \
  --features native-winit,renderer-wgpu,udp,command-bridge,web \
  --example composition_three_way
DISPLAY=:0 XDG_RUNTIME_DIR=/run/user/1000 WINIT_UNIX_BACKEND=x11 \
  target/debug/examples/composition_three_way --smoke
```

This finite demonstration briefly opens one visible 640 by 420 Window. It
requires a discrete Vulkan adapter for its bounded hardware proof. It does not
provide an interactive application or a new public protocol. All five example
`required-features` are explicit in the
[manifest](../crates/yetlet/Cargo.toml). Defaults remain empty. YT-293 retires
the staging companion; SDK publication retains only yetlet and yetlet-browser,
with no package version or published guide edition change.

## One authority and explicit ownership

| Attachment order | Concrete owned value | Execution and completion |
|---|---|---|
| Required local C1 | One `Rc<C1Domain>` allocation with `Cell<Snapshot>` and domain-owned `CommandBridge<Command, Snapshot>` | Creator foreground thread alone validates/applies commands and publishes immutable copies; close verifies admission closure and discards unapplied queued commands |
| Required local UDP | One `UdpEndpoint::bind_loopback(128)`, exact trusted peer address and `Option<UdpSocket>` OS clone of that bound peer | Foreground polls at most eight packets per wake; explicit close closes the application endpoint before releasing the peer reservation |
| Required foreground Native | `WindowSlot` plus independently selected `SurfaceRenderer` | Caller owns the sole `EventLoop`/`ApplicationHandler`; Surface closes before Window and C1 |
| Required worker Web | Original `HttpWorker` and clone of the exact bridge ingress | Exactly one original Tokio service thread/JoinHandle; separate admission fence, one-shot graceful stop and `observe_within` |

A temporary caller-local Rc alias permits typed navigation during UDP polling;
it references the same C1 allocation and is dropped before the wake ends. No
second domain, actor, scheduler, generic Host/Renderer/Transport trait, registry
or DI container is introduced. Final normal teardown verifies Rc strong count 1.

The finite `e1-finite-client-only` thread owns real TCP clients and one UDP peer.
It has no C1, command drain, snapshot publication capability, EventLoop or
renderer. Its original JoinHandle is retained and joined on success and error.
Network retains an OS `try_clone` of that same bound UDP socket after the client
returns or errors. The address remains reserved until the application endpoint
has actually closed, including while Native cleanup leaves the root Pending.
Remaining client handles must also close before another socket can bind the
peer address. This client is an explicit smoke peer, not an implicit Host thread.
Driver-owned threads are outside the application's count of owned Host workers.

## Commands, immutable snapshots and exact comparison

`Snapshot { generation, revision, paused }` is copied and immutable outside C1.
A process-selected generation identifies this bridge; revisions start at zero
and increase through checked addition only in C1's `apply`. The consumer-specific
typed `Pause` and `Resume` commands carry generation and expected revision.
Wrong generation, stale expected revision, closed admission and stopped C1 deny
application without changing state. HTTP queue admission is separate from this
validation and application step. The sole domain-owned bridge publishes each
accepted revision; HTTP receives only `CommandIngress` and `SnapshotReader`.

The local byte format is exactly 17 bytes: one tag byte followed by big-endian
`u64` generation and revision. Tags 1 and 2 select Pause and Resume. UDP tag 3
reads an exact revision; it is rejected as a Web command. Other tags, trailing
bytes and short datagrams are invalid. This format contains no private game
protocol, session token, simulation state or reliability machinery.

| Real HTTP route | Consumer behavior |
|---|---|
| `GET /ready` | HTTP 200 must actually return before worker `mark_ready`; bind/spawn alone is insufficient |
| `POST /command` | Exactly one typed byte command; 202 means admitted, 429 means bounded queue full, 503 means closed/disconnected/poisoned; malformed input is 400 and the 17-byte body limit rejects larger bodies |
| `GET /snapshot` | Latest copied `generation revision paused` text |
| `GET /snapshot?revision=N` | Exact retained immutable revision; invalid query is 400 and absent/evicted revision is 404 |

The queue capacity is four, domain drain is at most four commands per wake and
history retains eight immutable revisions. UDP accepts only the caller-selected
exact loopback IP/port. Malformed or wrong-sender packets cannot mutate C1 and
receive no success acknowledgement. A valid UDP command invokes the same C1
`apply` used by the HTTP drain; its acknowledgement contains the resulting
published snapshot. An exact UDP read also verifies generation. This local peer
reservation prevents consumer-local loopback address reuse; it does not establish
spoof-proof authentication, authenticated remote sessions or physical LAN behavior.

The smoke client checks HTTP initial state, submits HTTP Pause with real 202,
waits for exact revision 1, reads that same revision over UDP, applies UDP Resume,
and compares its revision 2 acknowledgement with exact HTTP and UDP readback.
After that finite client is joined, Native reads that exact retained revision
and records it only with an eligible visible OS redraw and a returned GPU
Present. No latest/latest timing assumption substitutes for an exact identity.

The existing renderer draws its original fixed neutral triangle. Logging an
immutable C1 observation does **not** mean the triangle visually projects
paused state or other domain data. Window Created, eligible OS redraw and real
Draw/Submit/returned Present are distinct receipts. Required Native readiness
is marked only after the latter two occur together; Window-only readiness
cannot qualify this composition.

## Failure, rollback and truthful completion

Every stop first closes the exact HTTP command ingress. Reverse local teardown
then releases Surface, Window, UDP and C1, followed by the worker's separate
graceful shutdown signal. Queue saturation cannot consume that one-shot.
Accepted but unapplied commands may be discarded after admission closes; they
are never described as executed. An already in-flight request cannot admit a
command after closure. The original worker must still drain and actually Join.

The Web startup factory closes issued ingress before returning a bind failure,
so immediate Composition rollback can stop previously acquired C1/UDP resources.
The bind error stays primary, with any fence/cleanup errors separate. Tests use
a prepared Window slot and a real TCP bind conflict; no negative GUI launches
are needed. Runtime failures call `fail_required` before loop exit; the
`run_app` return path also records missing proof/errors and drives cleanup even
if the exiting callback was absent.

A retained GPU discard/cleanup error allows actual Surface and Window release
but returns an error from Native close, preserving earlier UDP/C1 owners and
`Completion::Pending`. First required failure and secondary cleanup diagnostics
remain separate. A small completion driver observes the same root and original
worker within a cooperative three-second budget. Once the retained cleanup
error remains, Surface/Window are released and the original Web Join is terminal,
it returns Pending immediately; later driver calls do no further close work or
append duplicate diagnostics. Other Pending retries are paced. C1 remains
unstopped and the UDP endpoint and peer reservation remain owned. If incomplete,
the executable records that fact, retains the exact root storage until process
exit and returns an error. It does not drop/detach a pending JoinHandle or claim
successful cleanup. Driver FFI and user callbacks are not preemptible; this is
not a hard cancellation guarantee.

## Source-focused verification and integrated runs

On Ubuntu 26.04.1, Rust/Cargo 1.99.0, the repaired example's **7/7 tests pass**:

- Real HTTP 202 admission versus explicit C1 application, exact history,
  malformed queries/commands, stale generation/revision and publication rejection.
- Real UDP malformed and wrong-sender rejection without mutation, valid C1
  application/ack and matching HTTP immutable readback.
- Real loopback peer address reservation after client return and error, rejected
  unknown-peer command without C1 mutation, endpoint-before-reservation close,
  and successful peer rebind only after all remaining OS handles close.
- Actual Web bind conflict after C1/UDP acquisition, admission-first rollback
  and application/peer socket release without creating an OS Window.
- Actual full queue and held in-flight HTTP request, closed-gate 503, separate
  stop signal, deterministic Pending with the same worker storage, then actual
  original Join and TCP rebind.
- Synthetic retained GPU cleanup failure, preserved first required cause,
  Pending C1/UDP/reservation and independent actual Web Join. After Join, 1,024
  repeated driver calls leave secondary diagnostics at two. The exact Pending
  root is retained until test-process exit.
- Strict byte lengths/tags, typed authority checks and monotonic revisions.

Combined example `cargo check`, build, tests, targeted Clippy `-D warnings` and
rustfmt pass on repaired source. Earlier default/no-default library checks and
normal/build graphs remain applicable: manifests, features, dependencies and
adapters are unchanged by this repair. The example is correctly omitted with
features off. The initial Clippy exit101
(two collapsible conditionals) and subsequent correction are retained. No old
adapter test suites or Reviewers were replayed. Changed-document English,
links, fences and whitespace are checked separately from the known historical
global Docs failure.

The original pre-repair `DISPLAY=:0` run executed PID **169769**, exit **0**
in about **2.2 seconds**, with real viewable X11 Window **0x600002**, 640 by 420,
NVIDIA GeForce **RTX 4090**, Vulkan driver **595.99.02**. Real endpoints were
HTTP `<selected-endpoint>`, application UDP `<selected-endpoint>` and trusted client
UDP `<selected-endpoint>`. The foreground was ThreadId(1), client-only ThreadId(3),
and both command applications occurred on ThreadId(1).

Exact generation **1795138106814624176**, revision **2**, paused **false** matched
HTTP, actual UDP acknowledgement/readback and Native's immutable observation.
There were **38** actual render passes, draws, submissions and returned Present
calls, two configurations, and zero recorded recovery requests/timeouts. Normal
teardown was Complete/Stopped, Surface before Window before UDP before C1,
HTTP admission closed first, original Web worker Joined successfully, client
joined, Rc owners 1 and no secondary errors. The application itself successfully
rebound the TCP and UDP endpoints after Join/socket close. The same XID was
observed viewable while running and absent after exit; the process is gone.

**Observer qualification remains partial:** the first external observer exited1
because a raw TCP bind after the run returned address-in-use, despite the
application's successful rebind. Closing TCP connection state is a possible
cause, not an observation captured at that instant. Its original partial JSON
and raw logs remain unchanged. A read-only reconciliation attempt hit its
existing-run assertion before launch; that failed receipt is retained too.
Direct later readback exited0: connection refused on that exact HTTP endpoint,
no matching TCP table entry/listener, process absent, and original UDP/client
socket rebinds successful. No second application/GUI run was made at that
original checkpoint. The combined
records support actual application behavior and teardown; they do not relabel
the original observer as PASS. Independent review must assess this evidence.

### Current repair runtime and review gate

Exactly one new repaired-source `DISPLAY=:0` run executed PID **206510**, exit
**0** in **2.235 seconds**, without forced termination. The real viewable X11
Window **0x600002** was 640 by 420 and was absent after exit. RTX4090 Vulkan
driver **595.99.02** returned **46** real Draw/Submit/Present calls. HTTP
`<selected-endpoint>`, application UDP `<selected-endpoint>` and trusted peer
`<selected-endpoint>` agreed on generation **1795139176261158488**, revision **2**,
paused **false**, with real HTTP200 readiness and HTTP202 admission.

Final state was Complete/Stopped, exact original Web Join Succeeded, finite
client joined, sole C1 owner, no secondary failures, and admission-first
Surface/Window/UDP/C1 teardown. Network released the peer reservation after
actual endpoint close. The application successfully rebound HTTP, application
UDP and the trusted peer port. Both external UDP rebinds succeeded.

The new external observer is still **PARTIAL/FAIL**, exit1, because its raw TCP
bind returned address-in-use. Its immutable receipt remains unchanged. A
separate read-only check exited0: process absent, HTTP ECONNREFUSED, no LISTEN,
and closing TIME_WAIT entries present at readback. This supports listener
teardown; it does not turn the raw bind result into PASS or prove the TCP state
at the earlier bind instant. No additional GPU launch followed this result.

Current fourteen-file executed product-source map SHA256:
`eb4cbe0271d95aaa68bb42b80133616949f1f93c85756506dd401f1ba9575339`.
Current binary SHA256:
`c1f0033a7f0d063bdacab7bc27d1e263449d0f6f517950ba8eb5b3d5f65a1ac3`.
Original independent R1 FAIL2, its JSON/transcript/streams and the old GPU
receipts remain immutable. Independent Reviewer R2 `wc_job_A-G7FIs1j2jWPuyW`
returned actual JSON PASS (`issues=[]`, `required_followups=[]`) on the
frozen E1 source; only post-review documentation/status metadata changed.

Ignored full argv/stdout/stderr/exit receipts, original failures, source maps
and readback live under `.pipeline-output/m28-f2-yt282e1-20261120/`. Repair
records use unique `repair-r2-*` names, `REPAIR-R1.md` and `SOURCE-FENCE-R2.json`.
The historical `SOURCE-FENCE-R1.json` executed product-source map SHA256 is:
`989dd58310e59bf0198ebf1f7d1fe830ab02ac53ef28ad83da1678744a828dee`.
The reviewed adapters remain byte-identical to the E1 starting baseline.

No GPU completion fence, physical scanout, induced driver failure, Windows,
OS zero-size Window, F3/physical LAN, game-ready renderer, publication or external
YT-283 useful custom Host qualification is claimed. The next unblocked action
is the next bounded YT-282 public composition qualification increment, followed
by independent YT-283 external useful custom Host and YT-284 package gates.
The E1 source-only review does not close the full YT-282/F2 milestone.

## YT-282-G startup-order correction — 2026-11-20

The independent overall-gate Reviewer R1 `wc_job_YCMclXRrlLFOxokV`
terminated exit0 with **actual FAIL3/P2**, including a concrete E1 startup
rollback issue: a fallible UDP bind/address/peer clone followed C1 and open
command-admission construction but preceded Composition ownership. This could
return without the coordinated domain stop. The earlier E1 R2 actual PASS,
its GPU source and complete original receipts remain historical, unchanged.

The caller now runs its private `prepare_network_then_domain` helper before
Router/Ingress/Composition construction. It resolves the application UDP bind,
local address and trusted peer reservation before invoking the sole C1 factory.
A real UDP bind-collision negative confirms the domain factory never runs on
that early error. All later Web startup errors still follow the original
admission-first Composition rollback and retained original Web Join path.

On the changed source, E1 focused tests **8/8**, Build, Rustfmt and combined
Clippy `-D warnings` exit0; affected Web real HTTP **9/9** and local Web unit
**1/1** also pass. No second physical C1, generic Host/Renderer Trait, extra
worker, protocol promotion or GPU rerun was introduced. The original PID
206510 / 46 returned Presents is **earlier-source** evidence; the current
example has been rebuilt/tested, not GPU-executed. Independent whole YT-282
R2 review was pending at that source checkpoint; its later material
finding and correction are recorded below. YT-283 full game / YT-284
package qualification remain separate.
## YT-282-G R2 aggregate-startup readiness repair — 2026-11-20

Independent overall-gate Reviewer R2 `wc_job_xv_go3S7UYT4jyZM` reached
terminal exit0 with actual **FAIL2/P2**. The E1 route formerly accepted HTTP
202 before the required Native Window/GPU was Ready. HTTP `/ready` is real
Web worker readiness, not aggregate application readiness.

E1 now shares a narrow `Arc<AtomicBool>` command-admission gate between its
existing one C1 and its Web route. It starts closed and the caller-thread
Native callback opens it only after the actual GPU Present, Native required
part readiness and aggregate `Composition::all_required_ready()`. A valid
pre-ready HTTP POST returns 503 without queuing a command; `/ready` can still
return 200. The finite client retries only this pre-ready 503 under a fixed
deadline. HTTP202 remains admission, not an applied reducer receipt. The same
C1 denies trusted UDP commands while aggregate startup is incomplete, so the
UDP transport cannot bypass the required-Host startup boundary. Required
failure, domain stop and Winit exit revoke the flag without a second C1,
Simulation, Host abstraction or worker.

The focused fixture explicitly uses **synthetic** foreground Ready (not
an OS Window claim) to check pre-ready HTTP200 `/ready`, valid POST503, empty
queue and Revision0, then post-ready valid POST202 and sole C1 execution. It
also checks UDP rejection before readiness. E1 focused **8/8**, external
three-way **17 library / 7 binary / 5 doctests**, Web unit **2/2** and real
HTTP **9/9** PASS. Clippy `-D warnings`, Fmt and Build exit0.

The original E1 GPU run PID206510 / 46 returned Presents is **earlier-source**
evidence, not a current GPU execution; no GPU/GUI test was rerun. R1/R2
Reviewer FAIL results and all historical source receipts remain preserved.
Independent YT-282-G **R3 review is pending**; whole YT-282, full F1 YT-283,
package YT-284, F2 and physical LAN/D4 remain open.

## YT-282-G whole public source-only acceptance — 2026-11-20

After its R2 actual FAIL2/P2 and the bounded Web/E1 source repairs described
above, independent R3 `wc_job_EYHdVDKF0L9huUr5` returned actual FAIL1/P2:
the **separate external consumer** still admitted HTTP commands before Native
was ready. That consumer now binds its own sealed WebLaunch and sole C1 to
aggregate required-Host admission, with real HTTP 200 readiness, valid
pre-ready POST 503, post-ready POST 202 and C1/Audit revision readback.
The original R1 FAIL3, R2 FAIL2 and R3 FAIL1 evidence remains unchanged.
Independent R4 `wc_job_iebUf3KFJs9dqYlS` terminal exit0 returned actual
JSON PASS, `issues=[]`, `required_followups=[]`, after focused E1 8/8,
real HTTP 9/9 and external 18/18+7/7+5/5 verification.
The **whole YT-282 source-only public-API gate is Done**; the original full
F1 game migration YT-283 and YT-284 package/compatibility remain Todo.
Prior E1 Vulkan GPU receipts are source-specific historical evidence; the
current example was not GUI/GPU executed. Full F2 and physical D4 stay open.
