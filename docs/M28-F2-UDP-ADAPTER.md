# M28-F2 / YT-282-C2 — Opt-in UDP byte/socket adapter

- **Date:** 2026-11-19.
- **Status:** Bounded source-only opt-in UDP adapter complete: 8/8 real socket tests, Clippy/Fmt and feature boundary PASS; independent R1 actual PASS. Full YT-282/F2 remain open.
- **Authority:** ADR0013 scoped F2 accepted; YT-282-B public Composition Core and
  YT-282-C1 Web/Axum adapter have independent actual PASS reviews.
- **Package:** Existing source-only, non-published `yetlet-hosts` companion;
  `udp` is a separate default-off **standard-library-only** feature.
  UDP-only has no normal/build dependency on Yetlet, HTTP, Axum, Tokio,
  winit, wgpu, a graphics device, or a background worker. A consumer that
  also needs a composition uses its own explicit `yetlet::composition`
  dependency; the focused test demonstrates that combination.

This dated record retains the original companion dependency graph and tests.
YT-292/293 move the adapter into yetlet behind its std-only `udp` feature;
the source link follows its current location.

## Public behavior

[`udp`](../crates/yetlet/src/udp.rs) exposes a concrete
`UdpEndpoint` (owned `std::net::UdpSocket`), `SendOutcome`,
`PollReport`, `MAX_DATAGRAM_BYTES` and `MAX_PACKETS_PER_POLL`.

- `bind_loopback(capacity)` selects **127.0.0.1:0** by default.
  `bind_on(address, capacity)` requires the caller to explicitly select
  any other interface. Neither method asserts physical LAN connectivity.
  A zero or excessive payload capacity is rejected; its buffer has one
  extra byte to detect an oversized received datagram rather than
  silently delivering truncated data.
- `send_to(destination, raw_bytes)` submits an atomic UDP datagram within
  the configured capacity. It rejects an unspecified destination or
  port 0, distinguishes submitted vs OS WouldBlock, and does **not**
  claim delivery, acknowledgement, reliable order or peer authorization.
  There is no built-in game session/codec/permissions policy.
- `poll_bounded(packet_budget, callback)` performs caller-thread
  **nonblocking** receipt with at most the selected number of datagrams
  (hard capped to 64 per wake). The callback receives raw bytes and the
  actual source endpoint. A zero budget receives nothing. The report
  distinguishes received, callback-delivered, oversized rejected,
  WouldBlock/empty and limit reached. An oversize packet is rejected
  without calling the application. A callback error propagates and the
  socket stays owned; a consumed datagram is not replayed.
- `close()` is explicit and idempotent, actually drops the OS socket and
  permits a subsequent same-address bind when the OS allows it. After
  close, send/poll/address operations return NotConnected rather than
  stale success. Ordinary Drop performs standard Rust socket release;
  it does not perform a custom protocol handshake or remote shutdown.
- Application code controls its own game/server tick, peer acceptance
  and authentication. The public typed `yetlet::composition` can attach
  `UdpEndpoint` as a local part and call `close` during coordinated
  admission-first shutdown; no `Host` or `Transport` trait is required.

## Focused evidence and separate pending gates

New `crates/yetlet-hosts/tests/udp.rs` uses real Linux loopback sockets:

- independent real UDP endpoints and exact binary payload/source readback;
- finite per-wake budget, empty WouldBlock, and rejecting excess budgets;
- UDP oversized packet rejection without truncated callback delivery;
- explicit application-owned allowed/unknown peer policy, not a framework
  permission decision;
- bind collision safety, explicit-close idempotency and actual port rebind;
- nonzero payload/destination bounds and real post-close NotConnected;
- propagating application callback failure while retaining original socket;
- public Composition Core local resource attachment and an application-owned
  domain revision that advances only after the caller explicitly steps it.

Independent Reviewer R1 `wc_job_Y8K4gI5BvnbD-ckB` terminal exit0,
**actual JSON PASS**, `issues=[]`, `required_followups=[]` on the frozen
public source. Real socket tests **8/8 PASS**, Clippy `-D warnings`,
Rustfmt, UDP-only and combined UDP+Web compile/normal graph gates PASS.
Previous C1 actual HTTP and YT-282-B source acceptance are reused at
their original frozen identities and were not rerun solely because this
UDP module is separate. Exact receipts remain in ignored
`.pipeline-output/m28-f2-yt282c2-20261119/`.

## Non-goals and outstanding acceptance

No private `YTP1` codec, Twin Squares snapshots, session/peer token
acceptance, reliable transport, retransmission, remote firewall change,
actual physical LAN, direct GameHost/native renderer/wgpu adapter,
new HTTP host behavior or generic Host trait is added.

YT-282's full Game/Network/Web public delivery, independent external
three-way/custom-Host YT-283, YT-284 package qualification and F2
overall acceptance all remain open. Physical D4 is **Not verified**.
No Stage, Commit, Push, PR, release or registry publication is authorized.
