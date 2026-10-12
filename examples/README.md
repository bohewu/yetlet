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
python3 scripts/check_samples.py
```

## Getting started

Begin with [Hello World](hello-world/README.md) for native SSR or [REST API](rest-api/README.md) for HTTP-only development. [Todo](todo-server/README.md) adds explicitly built WASM; [Session cart](cart-server/README.md) adds an opt-in server capability. The [developer documentation index](../docs/README.md) separates current guidance from retained milestone evidence.

## Research and internal fixtures

The [five completed research prototypes](../docs/README.md#completed-research-prototypes) have separate bounded decisions and evidence: HTTP deadlines in Middleware Lab, incremental SSR, one custom typed transport, constrained handler/capture descriptors and an independent native scene renderer. They remain application-owned experiments. Browser companions, shared UI and benchmark fixtures also remain outside the twelve user-facing catalog entries. Use [the nine library guides](../docs/README.md#library-guides) for the candidate API boundaries.
