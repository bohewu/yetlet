# M28-C2 — Private native GameHost (YT-269)

The independent nested `native/` package selects **winit 0.30.13 (X11)**
and **wgpu 27.0.1 (Vulkan)**. Its opt-in dependency graph is separate from
the existing std-only B1/B2/C1 package, root workspace and pure Web/SSR path.
No public Yetlet Host or Renderer trait is introduced.

`GameDriver` alone owns the unchanged C1 `Simulation::step`. Physical
keyboard events queue bounded validated commands for two square players;
fixed logical 60 Hz ticks run separately from the targeted 30 Hz redraw
cadence. `project(snapshot)` produces two colored quads from immutable
snapshots. Window and surface lifetimes belong to the caller-thread winit
ApplicationHandler; the wgpu renderer owns its Device, Queue and Surface.

WASD = player one; arrow keys = player two. Space = Pause; Enter = Resume;
Escape = exit. Losing focus releases both players. Pause clears held input.
Resume clears tracked keys only when the effective state **after preceding
queued Pause/Resume commands** was Paused. A second Resume queued in the
same Tick preserves physical key tracking when it is a reducer no-op, so
release or focus loss still emits Hold(Still). A nonzero-size Surface
receives draws on RedrawRequested,
never on the simulation tick. A bounded `--smoke-ticks N` CLI option exits
automatically for one actual native startup integration attempt; it is not
C3 visual or resize/minimize/performance certification.

Headless C1 and B1/B2 use no winit/wgpu dependency. The original C2 and
default D2 selections do not own or create a Tokio runtime, HTTP listener,
GPU worker or extra Web thread. D2 explicitly selects one UDP socket owner.
C3 retains the later actual visual/lifecycle qualification.


## Verified bounded Ubuntu native run (C2 only)

On Ubuntu 26.04.1, X11 display `:0`, a 960x640 native winit window and a
wgpu 27 Vulkan Surface were actually created on an NVIDIA GeForce RTX 4090.
The initial 45-control-tick bounded run emitted 24 rendered frames and exited
normally. In one separate owned-app X11 XTest interaction the exact
window/PID were verified, then player-one D, player-two Left, Space Pause,
Enter Resume and corresponding key releases were sent. Its final immutable
Snapshot had control tick 210, active ticks 177, Player One (22,2),
Player Two (0,12), and 106 rendered frames. The test restored original
X11 focus and verified only its own native process ended.

These observations do not replace later C3 actual visual review, resize/
minimize/recovery and performance checks. Do not extrapolate this to other
display servers, GPU backends or native OSes.


## C3 — One bounded native GPU/Window visual and lifecycle check (YT-270)

The independent Native GameHost now accepts an opt-in
`--c3-qa-ticks 480` flag for a **single bounded diagnostic** run.
The unchanged default/`--smoke-ticks` paths do not inject events.
The diagnostic mode injects one 260ms foreground callback delay and
**synthetic** zero-size / Lost / Outdated input to the same guarded
renderer branches. Do not mistake those injections for actual X11
zero-size or genuine GPU device loss.

A separately scoped ignored test driver (see
`.pipeline-output/m28-c3-20261117/visual_lifecycle_x11.py`)
verified exact owned X11 window and process identities, captured real
XWD → PNG from the actual 960×640 Surface, resized the owned native
window to 720×480 and captured a second image, minimized/restored it
through the real Window Manager, captured again, then sent WM_DELETE to
only that window. The original X11 focus was restored, and the owned
WindowId no longer existed after normal exit.

Real screenshots are retained under
`artifacts/verification/m28-c3-20261117/c3-native-{initial,resized,restored}.png`,
with their raw XWD counterparts. Per-image grounded sample colors,
dimensions and SHA256 are in `native-c3-x11-observation.json`; host
runtime logs report Surface configuration, deferred zero size,
synthetic Lost/Outdated reconfigure, occlusion/resize/close and
CPU-side timing. The original failed title-identification harness
attempt is retained separately, not presented as a Native product fail.

During the actual run: 125 control ticks, 61 rendered frames,
two real resize events, four occlusion events, a single WM CloseRequested,
at most four ticks per wake, ~193.583ms dropped debt from one injected
260ms delay. CPU-side render encode/submit/present mean/max
2058.375/7080.547 µs; pure reducer step mean/max 11.116/61.725 µs.
These are single-run CPU timings, not GPU latency or sustained FPS
certification. Three selected C3 clock tests pass. An independent
R1 P2 found that while fully obscured the expired draw deadline could
busy-spin; the bounded repair now waits only for the next Simulation
Tick while hidden and resets the render deadline on restoration.
The one native run emitted only Occluded(false) events, so Occluded(true)
was validated with a focused non-GUI scheduling test, not claimed as
observed live Window behavior. The original GPU run was not repeated.
All previously validated C2/B0/B1/B2/C1 sources/tests remain unchanged.


## D2 — Two real Native GameHost processes, loopback UDP (YT-272)

The opt-in `d2-loopback` binary launches as `host` or
`client 127.0.0.1:<actual-port>`, each one independent OS process
composing a caller-thread winit GameHost with a private NetworkHost or
NetworkClient. Neither is a per-Host process/thread. Host alone owns
D1 HostFixture and C1 Simulation. Client keeps the latest read-only
D1 ReplicaFixture; both GPU renderers display immutable Scene data.
The only network is explicit 127.0.0.1 UDP, with no wildcard bind,
root Yetlet network dependency, firewall changes or LAN exposure.

Host binds port zero, reports its actual endpoint, and accepts only
one loopback peer Source for this bounded demo. D1 canonical Codec
version 1 and Session/Epoch/Peer/Sequence/Tick checks apply before
authority admission. Up to 8 nonblocking 513-byte datagram receives
per wake, one remote assignment, up to three Hello responses and
20 full Snapshot datagrams/s keep processing bounded. The scripted
demo moves Host Player One East and Client Player Two West. This
tests true datagrams and rendered state, not keyboard interaction,
reliable transport or hostile peer authentication.

Actual Ubuntu RTX4090/Vulkan X11 bounded E2E:
Separate Host and Client processes owned their selected loopback UDP sockets.
Both report identical authoritative Revision/Tick
90 with player X positions 22 and 0. Two real 960×640 Native Window
PNG screenshots have identical SHA256. Real Windows are closed via
WM_DELETE; both Exit 0, exact Window IDs disappear, Socket and GPU
ownership released, original X11 focus restored.

Retained exact real screenshots:
`artifacts/verification/m28-d2-20261118/d2-{host,client}-actual-window-r3.png`
and source XWD files; exact SHA256 and process/socket counters in
`d2-real-process-x11-udp-observation-r3.json`. The first two harness
attempts are retained as failed evidence (preflight Git status and a
Client Window positioned 5px beyond the screen), not hidden.

Focused Native-only tests:

    cargo test --manifest-path examples/composable-host-research/native/Cargo.toml --locked --offline --lib d2_

The Host/Client Demo is separately selected and is not a default
Yetlet service or runtime. D3 controlled packet loss/reorder,
reconnect, headless networking and D4 physical LAN remain separate.
Tokens/port matching are **not** cryptographic authentication.

### D2 independent R1 findings and bounded fixes

Original Reviewer R1 `wc_job_YfHgg6cq0ElmcFbT` actual FAIL with three
material P2s is retained. In the corrected Client, rendering waits for
a fully validated D1 Snapshot rather than showing fabricated players.
The 300-control-Tick bounded exit now releases the owned Socket and
Window even if the Window was obscured or zero frames rendered; it
reports a no-frame result separately. The private Native manifest also
restores `default-run = "yetlet-native-gamehost-research"`, so the
previous C2 `cargo run ... --smoke-ticks` guidance remains selectable
without naming a binary; D2 always explicitly uses `--bin d2-loopback`.
Two focused non-GUI regressions, Fmt/Clippy/Native build and Cargo
metadata checks PASS. The original two-PID wgpu/UDP E2E was not rerun
for these post-review repairs; it remains evidence only for the
unchanged successful synchronized path. Independent R2 pending.

### D2 independent R2 bounded Tick repair

Independent R2 `wc_job__nWJv7-x7VPY-qLT` actual FAIL found that
one delayed callback could yield four due ticks when 299 of the
selected 300 had already been processed. Host could then publish
through Tick303 before exiting. The private foreground D2 scheduler
now caps each due batch by the **remaining control-Tick budget before
calling the sole C1 owner step**; selected tests assert 299+4→300,
298+4→300, and zero work after exhaustion. Three/three selected D2
binary regressions PASS with Fmt/Clippy and Native Build. R2 FAIL
and prior R1 FAIL stay immutable; independent R3 pending. The prior
successful two-PID UDP/wgpu run was not relaunched after this source
repair; its original positive-path evidence remains retained.

Run from the repository root:

```sh
cargo test --manifest-path examples/composable-host-research/native/Cargo.toml --locked --offline --lib
cargo run --manifest-path examples/composable-host-research/native/Cargo.toml --locked --offline -- --smoke-ticks 40
```

## F1 / YT-277 - Opt-in same-process native, UDP and Web wiring

The optional private `f1-web` feature adds native Axum/Tokio selection only
to this independent nested research package; it does not change Yetlet's
public API, CLI, default feature graph, or original `d2-loopback host|client`
paths. The `host-web` argument uses the **existing D2 `NetworkHost` as the
single C1 Simulation authority**, not a second `GameDriver` or fixture.

```sh
cargo run --manifest-path examples/composable-host-research/native/Cargo.toml --locked --offline --features f1-web --bin d2-loopback -- host-web
```

The winit main thread still owns the game Window, GPU Surface and bounded
UDP polling. A separately selected, explicitly owned Web service thread has
one caller-created Tokio current-thread runtime, a loopback ephemeral TCP
listener, native Yetlet `App::get(...).into_router()` and Axum graceful serve
future. The root retains the shutdown sender and actual JoinHandle. Web
readiness requires a real `GET /f1/ready` HTTP 200 response; after native
exit the application signals shutdown and joins its Web thread before
reporting completion. If the Web thread terminates while the window runs,
the native app requests exit. The selected cooperative path has no hard join
deadline; timeout/failure recovery is reserved for YT-279.

**YT-277 scope at the time of its review:** the `/f1/ready` route only proves
Web serving, not shared game data. YT-278 adds the private command/snapshot
integration described below. The full required-component failure/drain
contract is YT-279; full F1 acceptance is YT-280. Physical D4 remains
unverified, and this research feature is not a public SDK Host API.

## F1 / YT-278 - Real bounded HTTP command and exact-revision game observation

The selected `host-web` process continues to own only the original D2
`NetworkHost` / D1 `HostFixture` / C1 `Simulation`. Web receives no Rc or
mutable game state. Its dedicated Tokio/Axum thread shares a bounded
8-message typed command queue and copied immutable snapshot records with the
foreground root. The root applies **at most one command per C1 Tick** using
the existing reducer, alongside real native/UDP Hold inputs.

These are deliberately private, loopback-only application endpoints:

| Method/path | Meaning |
| --- | --- |
| GET `/f1/ready` | Actual HTTP service readiness, not game readiness |
| GET `/f1/snapshot` | Latest copied authoritative C1 generation, tick, revision, positions, paused, active_ticks, last_applied_control |
| GET `/f1/snapshot/{revision}` | Exactly that authoritative revision from the bounded 256-publication ring, or 404 if unavailable |
| POST `/f1/control` | A typed Pause or Resume intent, with the C1 generation as a freshness fence; responds **202 queued**, never falsely claims applied |

Example application control request (from a caller authorized to use the
local private research endpoint):

    {"generation":1,"command":"pause"}

A successful POST returns a receipt ID and the latest observed revision.
Only a **later C1 reducer publication** with the matching
`last_applied_control` confirms application. The native root checks the
request generation again before dispatch. Invalid command JSON is rejected
by native Axum; mismatched generation returns 409, a full bounded queue 429,
and closed admission 503. Admission alone is not execution. Web never reads
the C1 Rc directly or creates a parallel GameState.

The original `d2-loopback host|client` paths remain independently selectable.
With `f1-web`, `host-web` has a finite 480-control-Tick budget for the HTTP
and UDP integration; ordinary D2 remains capped at 300. The default
feature-off normal dependency graph still has no Axum, Tokio or Yetlet
Web dependency. The only added direct optional dependency is Serde, selected
with the private `f1-web` feature.

### New actual Ubuntu integration result

One bounded source- and environment-scoped Ubuntu X11/Vulkan experiment used
the compiled same-source private `d2-loopback` binary
`b57663d2335a3f04bed7666280299da92e84c1945e17429a799bd1e9f17bbd01`
(SHA256). The actual Game+UDP+Web Host PID **3250513** and separate Native
UDP Client PID **3250558** both exited **0**. Host created a real wgpu
window, loopback UDP and Axum HTTP endpoint, and the Client admitted
actual wire Snapshots. Source-driven rendering and the read-only Replica
reported pinned matching C1 publications:

| HTTP command applied | Shared Revision / Tick | Both square positions (x,y) | C1 paused |
| --- | --- | --- | --- |
| Pause | 12 / 12 | [[2,2],[20,12]] | true |
| Resume | 18 / 18 | [[7,2],[17,12]] | false |

Each row is a *single exact-revision comparison* across an actual native
drawn scene, an actual UDP Replica, and HTTP GET of the corresponding pinned
publication. The two HTTP POSTs returned **202 queued**; separate actual
C1 apply records and the pinned `last_applied_control` fields distinguish
admission from application. Host recorded 240 native draws and two applied
Web commands; Client recorded 100 real UDP Replica publications. Both
processes completed naturally, with native resources released and Web
completion joined. Original evidence and complete bounded raw logs are in
the ignored `yt278/one-real-native-http-udp-e2e` receipts.

Focused verification covers the changed C1 control path, native Web queue,
latest/exact snapshots, stale/overflow rejects and selected D2 foreground
regressions. This is not a hostile-LAN security test, hard shutdown deadline,
publication API/SDK release or full YT-279/YT-280 acceptance.

The first independent YT-278 Reviewer found one normal-shutdown admission
race: a fresh HTTP control request could return 202 after the C1 owner had
stopped. The foreground exit callback now closes Web command admission
before stopping the authority, and a focused actual HTTP test verifies that a
new valid POST receives 503 and queues no work. That source-only correction
postdates the successful full Native/HTTP/UDP run above; the original positive
run and Reviewer FAIL are retained, not represented as a fresh post-fix E2E.

## F1 / YT-279 - Required component failure, drain and completion

The same selected native concrete root retains all Game/UDP/Web resources.
The Web service supports a selected loopback bind for scoped startup testing:
after a real Native NetworkHost is prepared, an occupied HTTP endpoint yields
the original bind error. The **same Native root startup function** now
explicitly calls NetworkHost.stop() on this failure, then the prepared UDP
socket can be rebound without a test-side stop. The Web startup worker is
joined rather than left running. Ordinary startup still uses ephemeral
loopback HTTP.

The foreground root detects unexpectedly completed required Web work. After
the **original JoinHandle** has been observed, it logs worker success
separately from the fact that a Join occurred. If Web was the first failing
component, the actual worker error replaces the preliminary early-exit
message; if Native/UDP failed first, its original cause remains primary.
The test-only failing worker returns a real error after controlled Web
shutdown, and the focused tests exercise the same cause-preservation logic
used by the Native root.

Before stopping C1 or UDP, the Native root closes Web command admission and
discards at most the finite eight queued but **unapplied** commands without
blocking on the full data queue. The independent Web stop channel does not
share that queue.

The owner's `complete_within` operation returns `Pending` on a deadline
while retaining its original JoinHandle. After a held HTTP request is
released, a later observation completes and joins that same task. The Native
root reports cleanup pending as an error, then retains ownership during a
recovery join instead of detaching and falsely reporting success. If an
in-flight handler never releases, eventual OS process termination is **not
guaranteed**; this is not a general hard-deadline process supervisor.

One scoped test-only HTTP endpoint holds an **actual in-flight Axum request**
through graceful shutdown. The first bounded completion is Pending, then
releasing the handler produces a real HTTP 200 and actual original-task Join.
The route is absent from the selected Native application binary.

### YT-279 evidence and independent R1 corrections

Six initial focused checks (Web 4, foreground 2) passed. Independent
Reviewer R1 then found two material P2s: the Web failure's real Join error
was hidden behind a generic early-exit message and the log conflated Join
with worker success; and the Native root relied on Drop instead of explicitly
stopping UDP on failed Web startup. Both original findings are retained.
The repaired focused set is **nine tests** (Web 5, foreground 4), all passing;
the exact two newly affected tests also passed after the final
Clippy-compatible source-only initialization correction. Rustfmt,
feature-on and feature-off Clippy -D warnings, the private Native build
and the default no-Web dependency graph passed. The earlier negative
Clippy and whitespace diagnostics remain in ignored receipts.

One bounded actual Ubuntu X11/Vulkan normal-close run predates those two
source-only R1 repairs: own `host-web` PID **3329304**, Game/UDP/Web Ready,
authoritative C1 Tick/Revision **480/480**, closed command admission,
stopped native resources, joined Web and exited **0**. The selected
The selected loopback HTTP and UDP listeners were released.
Original logs are under ignored `.pipeline-output/m28-f1-20261119/yt279`.
**That run is not a post-repair full Native/UDP/HTTP retest.**

YT-279 does not establish YT-280's final end-to-end gate, a public Yetlet
Host lifecycle API, Windows/native-driver portability, hostile-network
security, or physical D4.

## F1 / YT-280 - Latest-source three-host acceptance

On 2026-11-19, the latest YT-279 repaired source was built offline with
`f1-web` and ran on Ubuntu X11 / NVIDIA GeForce RTX 4090 / Vulkan.
The binary SHA256 was
`e0e7345296254f9c03f354639651b6b19ddc932c77d3cb608d4b57a6d2ba123a`.
Host PID **3434212** owned GameHost, NetworkHost and the native Web service
inside one application; separate UDP Client PID **3434256** consumed actual
wire Snapshots. There remained one C1 authority and no cross-thread Rc.

Both HTTP controls returned **202 queued**. Each command subsequently
produced an authoritative application record and a matching receipt in the
exact HTTP publication. The application revision and comparison revision
are intentionally separate:

| Command | Applied Revision | Compared Revision / Tick | Both positions (x,y) | Paused |
| --- | --- | --- | --- | --- |
| Pause | 1 | 12 / 12 | [[2,2],[20,12]] | true |
| Resume | 19 | 24 / 24 | [[7,2],[17,12]] | false |

Each comparison joins the Native scene record emitted **after successful
GPU submission and present**, the independent Client's admitted wire Replica,
and HTTP GET of that exact revision. While paused, HTTP observed Revision18
with the same positions and active_ticks=0. After Resume, Revision30 had
active_ticks=12 and positions [[13,2],[11,12]], proving continued Native and
UDP input progress while HTTP remained usable. Host rendered 240 frames;
Client rendered 149 frames. This run did not capture a desktop screenshot:
the Runner lacked computer_observe. Actual Window identities, source-linked
present records and owned process completion are recorded instead; there is
no claim of external desktop Window enumeration or pixel readback.

Both processes exited normally with code **0** after their existing finite
tick budgets. Host shutdown closed admission before authority/socket stop,
released Native Window/GPU owners and joined the original Web thread:
joined=true, worker_ok=true, result=complete, prior_pending=false.
Every observed application thread and both original PIDs were gone.
The selected loopback Host UDP, Client UDP and HTTP endpoints were successfully rebound.

The existing source-unchanged YT-279 focused held-HTTP test supplies the
in-flight drain and Pending/original-Join proof; that test-only route was
not injected into this Native binary. Its recorded failure, full-queue and
startup rollback checks, plus accepted format/lint/default-feature evidence,
are reused at their original scopes. No strict OS-exit deadline is claimed.

An earlier attempt stopped in its run-local script because it parsed the
plain-text /f1/ready response as JSON. Its Host PID3433195 exited0 with
Web joined and ports released; no Client had started. The original failed
receipt/script/logs remain. Only that script parser was corrected; product
source was unchanged. The corrected complete integration passed in 8.434s,
with source hashes unchanged before and after the run.

The ignored evidence directory is
`.pipeline-output/m28-f1-20261119/yt280`; it includes the frozen source,
latest build, original failure and corrected raw logs/receipt.
Independent YT-280 R1 Job `wc_job_57p0UF4VRV_k2Rks` reached terminal
exit0 with actual JSON PASS, issues=[] and required_followups=[]. The frozen
source/evidence hashes matched after review; observed saved role model was
gpt-6.2-astra with high effort and unchanged Runner settings. This is a
role-directed independent Codex Job, not native agent_type/formal assurance.
F1 is complete at this private integration boundary. ADR0013 remains Proposed;
F2 SDK/custom-Host delivery and physical D4/F3 remain open.
