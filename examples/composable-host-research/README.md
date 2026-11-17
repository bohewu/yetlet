# YT-266 — Private caller-thread composition research

**Scope:** M28-B1 only. This is an independent, unpublished Rust research
package, **not** a Yetlet SDK API, game, Host trait or process supervisor.

`PanelHost` and `RuleHost` are two concrete, caller-driven hosts with distinct
Prepared → Running → Stopped lifecycles. Both operate on the same OS thread,
use `Rc<RefCell<_>>`/`Rc<Cell<_>>` for their own non-`Send` local state,
and submit typed commands to one bounded mailbox. Only `StateOwner` applies
those commands to its private authoritative state. Both Hosts obtain the
same immutable `Rc<Snapshot>` publication with generation/revision;
old snapshots remain unchanged. Admission does not imply application.

The composition root explicitly starts both Hosts before opening domain
ingress, drains a bounded batch, and stops each Host separately. One Host
can stop without terminating the other or creating a worker. If closed
with pending admitted commands, the owner reports a discard count; full
readiness, failure propagation, deterministic shutdown completion and late
generation recovery are **not** proven here and belong to YT-267.

Run from the repository root with the pinned Rust toolchain:

```sh
cargo test --manifest-path examples/composable-host-research/Cargo.toml --locked --offline
cargo run --manifest-path examples/composable-host-research/Cargo.toml --locked --offline
cargo clippy --manifest-path examples/composable-host-research/Cargo.toml --all-targets --locked --offline -- -D warnings
```

The package declares its own `[workspace]` and `Cargo.lock`, with
**no normal dependencies**. It does not modify the root workspace, CLI
templates or SDK, and creates no HTTP listener, socket, window, GPU,
Tokio runtime, background thread or second process. A `println!` in
the demonstration shows the observed snapshot; it is not a UI renderer.

This POC deliberately does not implement Yetlet `ActionExecutor` or an
async runtime: a local bounded queue accepts commands, while application
is a separate explicit step. Integration/extraction into SDK abstractions
is an evidence-gated M28-E choice, not implied by this research example.
Pure API/SSR remains on the existing direct Axum path (YT-276).
