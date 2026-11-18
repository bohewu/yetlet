# Private composable-host research — B1, B2 and C1

**Scope:** Independent, unpublished Rust research package, **not** a Yetlet
SDK API, public Host trait or process supervisor. B1 caller-thread composition,
B2 worker lifecycle and C1 headless game simulation are separately selected.

## B1 — Caller-thread hosts (YT-266)

PanelHost and RuleHost are two concrete, caller-driven hosts with distinct
Prepared → Running → Stopped lifecycles. Both operate on the same OS thread,
use non-Send Rc local state, and submit commands to one bounded mailbox.
Only StateOwner changes the authoritative value. Both Hosts obtain the same
immutable Rc Snapshot generation/revision, and old snapshots stay unchanged.
Successful admission does not imply command application.

The caller starts both Hosts before opening ingress, explicitly applies a
batch, and stops them independently. B1 creates no owned worker, socket,
HTTP listener, GPU or runtime. Its close method reports the number of
admitted but unapplied commands discarded, not a delivery guarantee.

## B2 — Explicitly owned worker (YT-267)

The worker module adds a **separate opt-in variation**, not a replacement for
B1. The composition root spawns and owns exactly one std worker holding the
authoritative numeric state; two concrete foreground host attachments remain
on the caller thread and use Rc-backed local callback state. A borrowed
CallerScheduler token remains usable after both Host attachments stop.

Worker input is typed and bounded; Full, Closed and StaleGeneration admission
are distinct from successfully applied domain changes. Immutable snapshots
with generation/revision are published independently. Required startup failure
of the second Host stops the first and joins the worker before returning the
original cause. Required runtime Host failure or an injected worker fault
closes ingress, coordinates stop, and keeps the primary failure distinct from
cleanup status.

The cancellation signal is **not** enqueued into the bounded data queue. A
saturated or intentionally paused worker can therefore be stopped without
unblocking the data queue. A zero-timeout cleanup returns TimedOut while the
Group still retains the JoinHandle; a subsequent completion attempt can report
Joined. Dropping an unjoined Worker is deliberately a blocking join fallback,
not a misleading detach-as-cleanup success. No generic deadline enforcement or
arbitrary blocked native IO termination is claimed.

The negative fixture old_generation checks fencing without pretending a
generation restart/reconnect implementation exists. The artificially paused
worker and injected fault/delay options only exercise bounded test cases.
Neither B1 nor B2 provides production-grade scheduling, browser/native UI,
Window, GPU, game ticks, socket/protocol, multi-process or network support.

## C1 — Twin Squares headless simulation (YT-268)

Twin Squares is an original, deliberately small 24 by 16 integer-grid arena.
Two 2 by 2 squares start at (2,2) and (20,12), move one cardinal unit per
active tick and clamp their entire footprint to the arena. Overlap is allowed;
there is no collision engine, scoring system, ECS or floating-point physics.

`HeadlessRunner` owns one `Simulation`. Each successful call advances one
logical 1/60-second control tick through the single private `reduce` function.
There is **no wall-clock pacing, elapsed-time accumulator or background worker**
in C1. A future GameHost may call the same `Simulation::step`; no real GameHost
or B2 worker-to-game adapter has been implemented by this slice.

Inputs bind generation, tick and sequence to a typed Hold/Pause/Resume command.
At most 32 commands are accepted per tick. The owner sorts by sequence, rejects
duplicates/gaps/replays and requires contiguous numbering across its lifetime.
The next control tick must be exact; wrong generation/tick, over-budget input
or counter exhaustion rejects the **entire tick**, preserving the previous
publication and consuming no sequence. The source queue/order comes from the
caller, not a claim of deterministic live network arrival.

Pause clears held movement; Hold commands while paused are consumed but ignored.
Control tick/revision continue so Resume can be applied, while active game ticks
and positions remain frozen. Resume clears held movement on transition and does
not replay paused ticks; movement needs fresh input. Repeated Resume when already
running is a no-op. Still releases a held direction. Tick, active-tick, revision
and input sequence increments are checked; generation is a fixed nonzero lifetime
ID, not a wrapping counter or a restart mechanism.

Only the owner publishes immutable `Rc<Snapshot>` values; fields are private and
accessors return copies. Retained snapshots never change. Tests compare the
**complete state** at each tick, not an unspecified hash or only final positions.
Read-only observation cadence does not advance simulation. GPU/Window, sockets,
HTTP, display variables, real-time scheduling and C2 remain out of scope.

## C2 — Optional native GameHost (YT-269)

[Native X11/Vulkan GameHost research](native/README.md) is a **different,
explicitly selected nested workspace** with winit/wgpu dependencies. It
calls the same C1 Simulation::step and read-only Scene projection, but
does not add graphics dependencies or extra Runtime costs to this std-only
B1/B2/C1 package, root Web server, SDK or CLI. Actual Ubuntu X11/Vulkan
startup and a bounded keyboard interaction path were exercised; C3 reserves
full visual, window lifecycle and performance qualification.


## D1 — Private bounded Codec and InMemory session (YT-271)

The std-only private protocol module is NOT a public Yetlet
Protocol/Transport/RPC API. It has no Socket, worker, Tokio or networking
dependency. Canonical wire messages use ASCII YTP1, version 1, a strict
12-byte header, zero reserved bits and a payload length in little-endian
u16. The maximum frame is 512 bytes including header. The five exact
kinds and sizes are: Hello=1 (20 bytes), Welcome=2 (53), HeldInput=3
(53), Snapshot=4 (63), Leave=5 (44). Every invalid version/kind/bit,
length, field or zero identity is rejected with no partial decode.

Trusted in-memory Endpoint provenance is mapped by the Host to a
server-issued peer/session/epoch/Player binding. A wire peer ID alone
is not authority. Two peers may share a game session; peer and endpoint
IDs remain distinct. Each remote input needs a fresh per-peer sequence
(gaps allowed), never MAX, and exactly the next C1 owner Tick. Two
latest directional slots are bounded; a local command and remote
command use only the unchanged single C1 Simulation::step with its
own consecutive authoritative input sequence. Admission is not
application. Invalid inputs preserve the immutable Rc<Snapshot>.
A finite InMemoryMailbox holds at most four complete packets.

The read-only replica checks trusted host Endpoint plus expected
Session/Epoch/Generation; duplicate, stale revision/tick and old
session snapshots do not overwrite the latest immutable projection.
Hello/Welcome/Leave are only Codec shapes, not network handshake or
reconnect support. Session/epoch IDs are NOT authentication tokens.

Focused D1-only tests and a runnable zero-Socket demo:

    cargo test --manifest-path examples/composable-host-research/Cargo.toml --locked --offline --lib protocol::
    cargo run --manifest-path examples/composable-host-research/Cargo.toml --locked --offline --bin protocol-demo

Real UDP, independent process IDs and packet-fault qualification are
reserved for D2/D3; D1 does not establish them.

## Run the examples

From the repository root using the pinned Rust toolchain:

```sh
cargo test --manifest-path examples/composable-host-research/Cargo.toml --locked --offline
cargo run --manifest-path examples/composable-host-research/Cargo.toml --locked --offline
cargo run --manifest-path examples/composable-host-research/Cargo.toml --bin worker-demo --locked --offline
env -u DISPLAY -u WAYLAND_DISPLAY cargo run --manifest-path examples/composable-host-research/Cargo.toml --bin headless-demo --locked --offline
cargo test --manifest-path examples/composable-host-research/Cargo.toml --lib simulation:: --locked --offline
cargo clippy --manifest-path examples/composable-host-research/Cargo.toml --all-targets --locked --offline -- -D warnings
```

The original B1 binary remains the default; worker-demo and headless-demo are
explicitly selected. The package declares its own nested workspace and Cargo.lock,
publish=false and **zero normal dependencies**. It does not change the root
workspace, SDK, CLI, templates or base Web server feature graph.

This is ordinary Rust composition, not integration with Yetlet ActionExecutor
or a universal async runtime. Any future public Host/worker contract remains
an evidence-gated M28-E decision. Pure Web/API/SSR continues along the
unchanged native Axum path established at YT-276.
