# ADR 0011 — Simple Application Startup with Explicit Native Control

- **Status:** Accepted (contract only; YT-253 implementation pending).
- **Date:** 2026-11-11.
- **Task:** YT-251, M26 Application Startup & Sample Experience.
- **Related:** [M26 plan](../YT-251-258-APPLICATION-STARTUP-SAMPLE-PLAN.md),
  [ADR 0001 — Separation of concerns](0001-separation-of-concerns.md),
  [ADR 0005 — Opt-in capability modules](0005-opt-in-capability-modules.md),
  [ADR 0006 — Handler-first and optional MVC](0006-handler-first-mvc-optional.md),
  [current API reference](../API-REFERENCE.md),
  [sample catalog](../SAMPLES.md).

## Context

`yetlet::server_axum::App` is intentionally a thin facade over native Axum `Router`. It composes ordinary Rust handlers, `Page`, route groups and Tower middleware; it does not own a parallel router, Controller hierarchy, dependency-injection container or application runtime. `App::into_router()` is the existing boundary for callers that need ordinary Axum/Tower APIs.

The current minimal application still has to convert `App` into a `Router`, construct a Tokio `TcpListener`, and call `axum::serve`. That is unnecessary ceremony for the common local-development case. At the same time, applications that need a custom listener, actual bound address, graceful shutdown, TLS, multiple listeners or custom socket options must retain full native control.

The owner-selected product principle is **Simple first, customizable when needed**. The default must also avoid exposing a new application to the local network unintentionally, and it must not imply that a plain HTTP listener is a production TLS deployment.

## Decision

Add two additive convenience methods to the native Axum adapter, restricted to a fully resolved `App<()>`:

~~~rust
impl App<()> {
    pub async fn serve(self) -> std::io::Result<()>;

    pub async fn serve_on<A>(self, address: A) -> std::io::Result<()>
    where
        A: tokio::net::ToSocketAddrs;
}
~~~

The intended implementation is equivalent to:

~~~rust
const DEFAULT_LISTEN_ADDRESS: &str = "127.0.0.1:3000";

impl App<()> {
    pub async fn serve(self) -> std::io::Result<()> {
        self.serve_on(DEFAULT_LISTEN_ADDRESS).await
    }

    pub async fn serve_on<A>(self, address: A) -> std::io::Result<()>
    where
        A: tokio::net::ToSocketAddrs,
    {
        let listener = tokio::net::TcpListener::bind(address).await?;
        axum::serve(listener, self.into_router()).await
    }
}
~~~

`DEFAULT_LISTEN_ADDRESS` is an implementation detail, not a new public configuration API.

### Default address and exposure

- `App::serve()` binds exactly `127.0.0.1:3000`.
- The default is loopback-only. It is a safe local-development convenience and does not expose the listener to other machines on the LAN.
- The port is fixed. If another process already owns it, the bind error is returned; Yetlet does not silently choose another port.
- `serve()` does not read environment variables, config files, DNS names or deployment metadata. This avoids hidden precedence and keeps the default deterministic.
- `App::serve_on(address)` is the explicit override. It accepts Tokio `ToSocketAddrs`, including a string such as `"127.0.0.1:8080"` or a `SocketAddr`.
- Binding to `0.0.0.0` or `::` is an explicit all-interface exposure decision. Rustdoc and the server guide must warn that this is not a safe default for an untrusted network.
- Existing samples with environment-configured addresses retain those variables and call `serve_on(configured_address)`; they must not silently discard their current configuration contract.

### HTTP and HTTPS

`serve()` and `serve_on()` provide plain HTTP only. Their input is a TCP socket address, not an `http://` or `https://` URL.

HTTPS is not enabled by default. These methods do not infer TLS from a port, hostname, environment variable or deployment mode; they do not create self-signed certificates, discover certificate files, load private keys or manage certificate renewal. A self-signed development certificate is not a substitute for an explicit trust and certificate-lifecycle decision.

For a common production topology, an explicit reverse proxy or load balancer terminates HTTPS and forwards to the Yetlet application over a protected internal HTTP connection. If an application must terminate TLS itself, it uses an explicit TLS-capable native Axum/Tower listener/accept loop around the Router. That advanced transport is outside these convenience methods and does not add a TLS dependency to Yetlet Core or the default server feature.

The loopback HTTP default is not production deployment guidance. Applications must not expose plain HTTP on an untrusted network without an explicitly selected secure edge or direct-TLS integration.

### Ownership and failure behavior

- Both methods consume `App<()>`; an unresolved `App<S>` cannot be served until its Axum state has been supplied and the router becomes `App<()>`.
- `App::from_router(router).serve()` and `.serve_on(address)` are supported when the supplied Router has no unresolved state.
- Both methods reuse `into_router()` and Axum's existing `serve`; they do not create a second router or server runtime.
- Both methods are async. The application owns and starts its Tokio runtime. Yetlet does not create, hide, spawn or shut down a runtime.
- Address resolution and `TcpListener::bind` errors propagate as `std::io::Error`. After a successful bind, Axum 0.8.9's `Serve` future has an `io::Result<()>` output type but its internal run loop is non-returning; it does not return post-bind accept errors or `Ok(())` during normal operation.
- Axum's TCP `Listener` handles accept errors internally: connection errors are retried immediately; non-connection accept errors are logged by Axum and followed by a one-second delay before retry. These are not application-visible errors. Yetlet adds no startup output or custom logging/error policy; it preserves native Axum behavior.
- Yetlet adds no bind-address/port fallback, detached task, or alternate post-bind error policy.
- The methods do not return the bound listener or its actual local address. An address using port `0` can bind, but the caller cannot discover the selected port through these methods.
- Graceful shutdown is not added to the convenience API in this ADR. Applications needing Ctrl+C/SIGTERM coordination, drain ordering, listener ownership, port `0` plus `local_addr()`, custom socket options, multiple listeners, or direct TLS use `into_router()` and native Tokio/Axum.
- No `serve_with_shutdown`, `listen`, `run`, `bind_and_serve`, configuration builder or overload family is introduced.

## Dependency and feature boundary

`server-axum` remains an optional, native-only feature. The adapter needs a direct Tokio dependency to name `tokio::net::TcpListener` and `tokio::net::ToSocketAddrs`; that dependency must remain optional and target-gated.

Source inspection of the locked current graph (`axum 0.8.9`, `tokio 1.53.3`) shows that Axum's default `tokio` feature already enables Tokio `net`, `rt` and `macros`; Hyper/Axum also contribute other shared features. The current normal `server-axum` graph does **not** require `rt-multi-thread`. The optional `session-redis` capability is the path that requires the multi-thread runtime.

The baseline was checked with three separate commands:

- `cargo tree --offline --locked --edges normal -e features -p yetlet --no-default-features --features html -i tokio`: no Tokio dependency.
- The same command with `--features server-axum`: `macros`, `net`, `rt`, `sync` and `time`, but not `rt-multi-thread`.
- The same command with `--features session-redis`: adds `rt-multi-thread`.

These are observations of the current locked graph; YT-253 must repeat the checks after editing Cargo features.

Implementation direction for YT-253:

- Add the direct optional Tokio dependency to `server-axum` with only the adapter's required `net` and `rt` features.
- Keep `rt-multi-thread` out of `server-axum`; keep it selected only by `session-redis`.
- Keep any additional Tokio features used by `session-redis` explicit on that feature rather than enabling them merely to support the simple listener API.
- Preserve the `html`-only default graph: no Axum/Tokio dependency in the default Core build.
- Preserve the browser/WASM boundary: native server modules and Tokio remain unavailable to `wasm32` builds.
- Do not add a new server, TLS, configuration or runtime crate.

The current graph is evidence for the contract, not proof that a future Cargo edit is correct. YT-253 must inspect `cargo tree -e features` after the change and compile the relevant feature combinations.

## Alternatives considered

1. **Keep requiring `serve(address)`.** Rejected as the only simple path because every first application must know and specify a bind address before it can run.
2. **Provide only `serve()` with a fixed address.** Rejected because applications need a simple, explicit way to override the default without dropping to raw Axum.
3. **Use `serve()` plus `serve_on(address)`.** Selected. Rust has no default method parameters or overloads; this pair makes the default obvious and the override explicit without introducing a configuration object.
4. **Read a bind address from an environment variable automatically.** Rejected. It introduces hidden configuration precedence and makes the same source behave differently based on ambient process state. Callers that already own environment/configuration continue to pass the resolved value to `serve_on`.
5. **Default to `0.0.0.0`.** Rejected because a first run would expose the application to other network interfaces without an explicit decision.
6. **Default to HTTPS or generate a self-signed certificate.** Rejected because TLS requires caller-owned identity, private-key custody, trust, renewal and deployment policy. An automatically generated certificate would not be generally trusted.
7. **Add a TLS stack or graceful-shutdown abstraction to `App`.** Rejected for this task. These are transport/lifecycle policies that belong to explicit native integration, not the default application facade.
8. **Remove `into_router()` and standardize every sample on the convenience method.** Rejected because it would make listener/lifecycle ownership harder and would erase useful native Axum examples.

## Consequences

New applications can start with `App::new().page(...).serve().await` and receive a deterministic, loopback-only HTTP listener. Existing applications can use `serve_on` to retain explicit configured addresses. The facade stays small and the native Router remains available.

The fixed port can conflict with another local process; this is reported rather than silently hidden. The default is not remotely accessible. HTTP is not encrypted, so production deployments must choose an explicit TLS edge or direct TLS integration. The convenience API intentionally does not expose the bound listener, ephemeral port, shutdown future or custom accept loop.

## Review evidence

- Initial independent Reviewer Job: `wc_job_zaxfmBinaf26QRTV`; terminal exit code `0`; result `FAIL` with one P2 finding on post-bind Axum accept-error propagation.
- Corrective follow-up Reviewer Job: `wc_job_Nad2HOpWhHREJOty`; terminal exit code `0`; result `PASS` with `issues = []` and `required_followups = []`.
- Follow-up Reviewer trace: saved `reviewer` role, model `gpt-6-astra`, effective effort `high`; role/model/effort matches and parent inheritance all `true`.
- The P2 was closed by documenting that only address-resolution/bind errors propagate; Axum 0.8.9 handles post-bind accept errors internally and its serve loop does not return during normal operation.
- Ignored review evidence is retained under `.pipeline-output/yt251-application-startup-contract-review-20261111/` (`reviewer-output.json`, `reviewer-output-v2.json`, `reviewer-trace-v1.json`, `reviewer-trace-v2.json`).

## Acceptance boundary

This ADR accepts the startup API contract only. YT-253 implements the two methods and performs the targeted native tests and dependency-graph verification. YT-255–257 migrate samples only after the implementation receives its own independent Reviewer PASS. The SDK alpha.7, CLI 0.3.1, published Reader Edition, CLI templates and published First App guide remain unchanged until a separate release/reader decision.

No product code, Tokio dependency, TLS dependency, CLI template, release artifact, Git tag or publication is changed by this ADR.
