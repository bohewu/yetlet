# Custom action transport research — YT-024

This unpublished, native-only research fixture explicitly implements the existing
`yetlet_actions::ActionExecutor<Add>` boundary using one TCP connection per command.
It is excluded from the user-facing sample catalog and adds no supported crate API.

From the repository root:

```bash
cargo test -p yetlet-custom-action-transport --offline -- --nocapture --test-threads=1
```

The direct product tests bind only `127.0.0.1:0`, use synthetic read-only arithmetic,
and join every peer task before checking that reconnect is refused. There is no
long-lived server command, browser runtime, UI, HTTP route, credential or external
service. The adapter and server handler depend only on `yetlet-actions`, Tokio,
`tokio-util`'s mature length-delimited codec, futures and serde; no renderer,
reactive runtime, server adapter, controllers, DI or plugin registry is installed.

Application selection is ordinary Rust construction: use `LocalExecutor` for local
commands, or explicitly pass a `TcpExecutor::new(loopback_address, deadline)` to a
caller using `ActionExecutor<Add>`. Neither construction changes framework defaults.
The server application owns its listener and calls `serve_connection` for each
accepted connection. This fixture is not a deployment server or a generic provider.

The single protocol is a four-byte big-endian payload length followed by at most
1,024 JSON bytes. A request such as `{"left":20,"right":22}` returns
`{"result":"ok","value":42}`; overflow returns an explicit error envelope.
Malformed command fields are rejected; malformed/oversized/truncated frames are
closed. One connection handles one request/reply and then closes. There is no
retry, reconnect, DOM event serialization or persistent remote event circuit.

Loopback restriction is a research containment boundary, not authentication.
Any local process can connect. Authentication, per-command authorization, TLS,
rate limits, deployment policy, and any future cookie/browser CSRF design remain
application responsibilities and are not implemented here. Deadlines drop owned
sockets but cannot roll back effects already performed by a future real handler.

See [the research decision and verification record](../../docs/YT-024-CUSTOM-TRANSPORT-RESEARCH.md)
and [retained direct proof output](proof.log). Native compile and wire tests passed;
browser/WASM and CSP enforcement were not run. This is bounded prototype evidence,
not production security, portability or release qualification.
