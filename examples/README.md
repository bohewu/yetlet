# Yetlet Samples

User-facing samples are catalogued in `examples/samples.toml` and described in [the sample guide](../docs/SAMPLES.md). The catalog is intentionally narrower than the Cargo workspace: internal browser entry crates, shared UI crates and research fixtures remain workspace members without being presented as standalone application samples.

Every catalogued sample must provide:

- one workspace package and one sample-local `README.md`;
- a copyable command executed from the repository root;
- an explicit route list and browser-runtime declaration;
- stated optional Yetlet dependencies;
- retained verification evidence;
- no claim that prototype evidence is production qualification.

Run the catalog gate with:

```bash
cargo xtask check samples
```

## Getting started

Begin with [Hello World](hello-world/README.md) for native SSR or [REST API](rest-api/README.md) for HTTP-only development. [Todo](todo-server/README.md) adds explicitly built WASM; [Session cart](cart-server/README.md) adds an opt-in server capability. The [developer documentation index](../docs/README.md) separates current guidance from retained milestone evidence.

## Source-checkout startup

Use `App::new().page(...).serve().await` for the shortest application startup,
with plain HTTP at `127.0.0.1:7878`. The non-catalog SDK example demonstrates it.

```bash
cargo run --locked -p yetlet --features server-axum --example server-axum-minimal
```

Hello World and the other basic HTTP samples retain their own environment ports
through `App::from_router(...).serve_on(address)`. Todo and Style Showcase use
that same explicit-address path. Port `0` keeps native listener ownership for
actual address reporting. File Server, Auth Notes, Modular Shop, SSE, SQLite CRUD
and Multi-Island Dashboard retain native `TcpListener`, `axum::serve` and graceful
shutdown. `into_router()` and optional Prelude imports preserve native Axum/Tower
composition. The [startup and port table](../docs/SAMPLES.md#startup-paths-in-this-checkout)
and [source API examples](../docs/API-REFERENCE.md#source-only-checkout-additions)
describe these checkout additions after published SDK alpha.7.

Todo and Dashboard build their browser assets explicitly. Style Showcase runs
natively through its catalog command and has an optional browser companion.
The startup helpers use the application's Tokio runtime and provide plain HTTP.
TLS/HTTPS requires explicit edge or native integration, with no inferred TLS or
hidden runtime.

## Research and internal fixtures

The [five completed research prototypes](../docs/README.md#completed-research-prototypes) have separate bounded decisions and evidence: HTTP deadlines in Middleware Lab, incremental SSR, one custom typed transport, constrained handler/capture descriptors and an independent native scene renderer. They remain application-owned experiments. Browser companions, shared UI and benchmark fixtures also remain outside the thirteen user-facing catalog entries. Use [the current module guides](../docs/API-REFERENCE.md) for API and source-only boundaries.
