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
