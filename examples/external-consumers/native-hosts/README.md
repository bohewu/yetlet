# Native Host composition consumer

This independent consumer uses one ordinary HostGroup with a NativeHost that
owns two windows, a loopback UdpHost, and a custom local Host with an explicitly
managed resource. Its optional web feature adds one API-only WebHost.

The only direct dependency is yetlet with default features disabled. The
consumer's default feature set is empty. Enabling native selects only
yetlet/native-winit and yetlet/udp; it does not select Web, Tokio, View or a
renderer. Enabling web additionally selects yetlet/web. No command bridge is
needed because this example does not cross an application command boundary.

Run the finite native and UDP example from the repository root:

```sh
cargo run --manifest-path examples/external-consumers/native-hosts/Cargo.toml --features native
```

Include the optional API-only HTTP Host:

```sh
cargo run --manifest-path examples/external-consumers/native-hosts/Cargo.toml --features native,web
```

A graphical native desktop and its platform entry thread are required. Run
one process for each attempt: Winit's process-wide event-loop restrictions
still apply. These commands create real windows. They close automatically
after the loopback client completes or its absolute deadline expires.

To inspect feature selection without running windows:

```sh
cargo tree --manifest-path examples/external-consumers/native-hosts/Cargo.toml --no-default-features --edges normal
cargo tree --manifest-path examples/external-consumers/native-hosts/Cargo.toml --features native --edges normal
cargo tree --manifest-path examples/external-consumers/native-hosts/Cargo.toml --features native,web --edges normal
```

## What the program checks

Construction and build are inert. The root's native driver creates two windows
on the calling thread and continues driving UDP, optional Web observations, and
cleanup in the same native event-loop lifetime. Native callbacks, UDP callbacks,
the custom Host and the managed resource borrow application data or use Rc;
they check the calling-thread identity.

The custom Host publishes actual bound endpoints only after all Hosts admit
work and both windows have delivered real post-admission redraw callbacks.
One scoped std client thread, explicitly outside HostGroup, receives those
endpoints, checks a real UDP echo, and optionally checks GET /health with a
plain std TCP client. Channel and socket operations use an absolute deadline.
The client requests the group's same ShutdownHandle on success or failure;
main joins that client, including after early startup failure. No synchronous
network operation runs in a native callback.

The custom resource returns Pending once during final release, requests
another wake, and checks that both native windows are still present on both
release passes. This demonstrates that the root keeps its event loop and
windows alive while a later resource completes cleanup. The program then
checks successful original joins/final release for every Host, closed native
windows, cleared UDP socket metadata, and closed HTTP readiness/admission.
Web retains its bound address as metadata after shutdown. Live Window clones
are never retained outside native ownership.

stdout records actual window IDs, redraw observations, caller-thread identity,
bound endpoints, the Pending release, and final lifecycle outcomes. The final
gpu_present=false field is intentional: this consumer proves window and
transport behavior without selecting or qualifying a GPU renderer.
