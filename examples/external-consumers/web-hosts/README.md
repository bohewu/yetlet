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

## Start a Web Host on demand

Run the [dynamic Web Hosts example](examples/dynamic_web_hosts.rs) from the
root of the current source checkout.

```sh
cargo run --manifest-path examples/external-consumers/web-hosts/Cargo.toml --example dynamic_web_hosts --locked --offline
```

After the controller answers `GET /status`, the application builds a child
Web Host in its own `HostGroup`. Both bind to `127.0.0.1:0`. Each request
waits for admission and uses the address reported by `WebHandle::local_addr`.
After `GET /job`, the application requests child shutdown and awaits the
child group's original `run_async()` drive. The controller answers one more
status request before its own group shuts down and joins.

## Stop an open SSE stream and a pending TCP receipt

The [SSE shutdown example](examples/sse_shutdown.rs) runs one managed Web Host
and one application-owned Tokio task. The task sends `RECEIPT?\n` to a TCP peer
and waits for the four bytes `ACK\n`. To observe cancellation with the receipt
still outstanding, use three terminals.

In the first terminal, start a loopback peer that prints its address, reads the
request, and keeps the connection open until the application closes it.

```sh
python3 -u - <<'PYTHON'
import socket

with socket.create_server(("127.0.0.1", 0)) as listener:
    print(f"TCP peer: 127.0.0.1:{listener.getsockname()[1]}")
    conn, _ = listener.accept()
    with conn, conn.makefile("rb") as reader:
        assert reader.readline() == b"RECEIPT?\n"
        print("Request received. Withholding ACK.")
        assert reader.read(1) == b""
        print("TCP peer observed EOF.")
PYTHON
```

In the second terminal, run these commands from the repository root. Paste the
peer's printed `127.0.0.1:PORT` at the prompt.

```sh
printf 'TCP peer address: '
read -r peer
cargo run --locked --offline --manifest-path examples/external-consumers/web-hosts/Cargo.toml --example sse_shutdown -- "$peer"
```

In the third terminal, paste the full SSE endpoint URL printed by the
application. Curl prints the first event and continues reading the same response.

```sh
printf 'SSE endpoint URL: '
read -r url
curl --http1.1 --fail --silent --show-error --no-buffer "$url"
```

Once curl shows `data: ready` and the peer shows `Withholding ACK.`, press Ctrl+C
in the **application terminal**. Leave curl and the peer running. The
application signals the SSE body and the pending TCP task, requests HostGroup
shutdown, and waits for both the original task and the group. Curl exits
successfully at the end of the response. The peer observes EOF. The application
prints `TCP task joined: cancelled on shutdown` and `Web Host stopped.`.

After the application exits, run this in the curl terminal to bind the same
Web Host address again. The separate TCP peer owns the other listener.

```sh
python3 - "$url" <<'PYTHON'
import socket
import sys
from urllib.parse import urlsplit

url = urlsplit(sys.argv[1])
with socket.create_server((url.hostname, url.port)) as listener:
    print("Web Host address rebound:", listener.getsockname())
PYTHON
```

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
