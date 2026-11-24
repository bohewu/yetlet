# HostGroup Starter

Run a custom Host by itself, then add an HTTP service to the same
`HostGroup`. This standalone Cargo project requires Rust 1.99 or later and
uses Yetlet SDK `0.9.0-alpha.9` from crates.io. Its lockfile goes with the sample
if you copy this directory out of the repository.

## Run the local Host

From the repository root, run the default configuration. It uses the Rust
standard library and Yetlet Core.

```sh
cargo run --locked --manifest-path examples/hostgroup/Cargo.toml
```

The group starts the Host, admits one local job, then shuts down and joins.

```text
local: started
local: sum = 10
local: stopping
host group: stopped
```

## Add an HTTP service

Enable `web` to add an API-only `WebHost` to the group and drive both
Hosts on the application's Tokio runtime.

```sh
cargo run --locked --manifest-path examples/hostgroup/Cargo.toml --features web
```

The server binds to an available loopback port. Copy the URL printed after
`GET` and request it from another terminal. Replace `PORT` below with the
printed port.

```sh
curl -i http://127.0.0.1:PORT/health
```

The endpoint returns HTTP 200 with the body `ok`. Press Ctrl+C in the
server terminal. The application requests group shutdown and keeps driving it
until both Hosts have joined. The last line is `host group: stopped`.

See the [sample source](src/main.rs), the [composition API guide](../../docs/API-REFERENCE.md#published-application-composition),
or the [source-checkout example with two managed Web services](../external-consumers/web-hosts/README.md).
