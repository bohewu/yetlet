# YT-266 / YT-267 — Private composable-host research

**Scope:** Independent, unpublished Rust research package, **not** a Yetlet
SDK API, game runtime, public Host trait or process supervisor. The B1
caller-thread example and B2 explicitly owned-worker variation are separate.

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

## Run the examples

From the repository root using the pinned Rust toolchain:

```sh
cargo test --manifest-path examples/composable-host-research/Cargo.toml --locked --offline
cargo run --manifest-path examples/composable-host-research/Cargo.toml --locked --offline
cargo run --manifest-path examples/composable-host-research/Cargo.toml --bin worker-demo --locked --offline
cargo clippy --manifest-path examples/composable-host-research/Cargo.toml --all-targets --locked --offline -- -D warnings
```

The original B1 binary remains the default; worker-demo is explicitly
selected. The package declares its own nested workspace and Cargo.lock,
publish=false and **zero normal dependencies**. It does not change the root
workspace, SDK, CLI, templates or base Web server feature graph.

This is ordinary Rust composition, not integration with Yetlet ActionExecutor
or a universal async runtime. Any future public Host/worker contract remains
an evidence-gated M28-E decision. Pure Web/API/SSR continues along the
unchanged native Axum path established at YT-276.
