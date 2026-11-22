# Managed Web Hosts consumer

This independent workspace uses the current checkout's public `yetlet::web`
and Core Host lifecycle APIs. Its normal Yetlet dependency selects only `web`,
with defaults disabled. HTML, View and Actions remain separate opt-ins.

## Run the finite API-only example

From the repository root:

```sh
cargo run --offline --locked --manifest-path examples/external-consumers/web-hosts/Cargo.toml
```

The example creates an `App::into_host()` at the public default
`127.0.0.1:7878`, and a `WebHost::new(router)` at explicit loopback port `0`.
Both serve API routes without a root GET handler. The application owns a
current-thread Tokio runtime, sends finite requests, requests shutdown, and
drives the same retained HostGroup through completion.

Port 7878 must be available for this particular default-address example.
An occupied default remains a bind error. Applications can choose a different
address with `with_address`; the adapter does not silently select another port.
The handle's `local_addr` is acquired-address metadata. Readiness and normal
admission have separate observations.

## Managed SQLite resource proof

The integration test uses an application-owned SQLite actor through
`ResourceGroup` and `ManagedResource`. SQLite and temporary-file dependencies
are test-only; the SDK exports no database adapter.

```sh
cargo test --offline --locked --manifest-path examples/external-consumers/web-hosts/Cargo.toml --test sqlite
```

The resource keeps its original database thread, readiness receiver and cleanup
state in the retained descriptor. A cancelled borrowing drive can resume that
same owner. Admitted requests retain database access during Web drain; database
release and its original Join precede destruction of the earlier passive value.
Persisted rows are read back after terminal cleanup.

The SDK's `web_hosts` public integration target additionally exercises explicit
HTTP probes, current-thread progress, owned-thread execution, bind rollback and
custom local error/resource adapters. These are bounded current-source consumers,
not a publication or broad platform qualification. The earlier
[private research](../../host-lifecycle-research/README.md) retains its original
Application API, ephemeral default and root-200 readiness contract.
