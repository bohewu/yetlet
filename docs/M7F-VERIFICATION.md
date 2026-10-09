# M7f Verification — Renderer-Neutral Binding Contracts

**Date:** 2026-10-09
**Task:** YT-039 / M7f
**Status:** Locally complete. The validated identity, key, slot-name and inert-attribute vocabulary is now a renderer-neutral `yetlet-core` contract; no production renderer or authoring DSL is selected by this extraction.

## Outcome

M7e identified a small identity and slot vocabulary as a candidate for promotion, while explicitly keeping the fixed keyed-list authoring surface experimental. M7f performs only that bounded extraction:

- `ComponentIdentity` validates deterministic lowercase component and instance tokens;
- `BindingKey` rejects empty and control-character values while remaining renderer-neutral;
- `SlotName` validates the same deterministic token grammar;
- `InertAttributeName` allowlists `class`, `title`, `data-*` and `aria-*`, while rejecting executable/parsing-sensitive attributes and the reserved `data-yetlet-*` namespace;
- `SlotDescriptor` distinguishes text slots from inert-attribute slots without selecting a DOM implementation, template compiler or scheduler.

The implementation is in `crates/yetlet-core/src/binding.rs`. The M7b–M7d prototype now imports these production contract types instead of maintaining duplicate identity, key, slot and attribute validation logic. Its fixed `KeyedListTemplate`, input, focus and conditional-slot APIs remain experimental.

## Compatibility and security boundary

The extracted types contain no browser, server, transport, macro or async dependency. They compile for native and wasm32 targets. Values remain ordinary untrusted data: HTML escaping is still the renderer's responsibility, and these contracts do not authorize raw HTML, URL attributes, inline handlers, style, `srcdoc` or user-defined `data-yetlet-*` metadata.

M7f does not change Todo Browser or Search Island dependencies/assets and does not claim hydration, resumability, a general template IR, a VDOM or a forms runtime.

## Validation

```bash
cargo fmt --all -- --check
cargo clippy --locked -p yetlet-core -p yetlet-keyed-bindings --all-targets -- -D warnings
cargo test --locked -p yetlet-core -p yetlet-keyed-bindings --all-targets
cargo test --locked -p yetlet-core -p yetlet-keyed-bindings --doc
cargo check --locked -p yetlet-keyed-bindings --target wasm32-unknown-unknown
python3 scripts/check_docs.py
git diff --check
```

Observed before reviewer dispatch:

- `yetlet-core`: **5/5 tests PASS**, including four binding-contract tests;
- `yetlet-keyed-bindings`: **17/17 tests PASS**;
- keyed-bindings rustdoc: **1/1 PASS**;
- targeted strict Clippy, wasm32 compile, formatting and whitespace checks: PASS.

## Remaining M7 sequence

M7f deliberately promotes only metadata contracts. The remaining bounded tasks are:

1. `YT-040 / M7g`: extract the proven strong-owner/weak-handle synchronous local graph into an optional production crate;
2. `YT-041 / M7h`: add a general keyed DOM reconciliation and binding-ownership primitive without promoting the fixed authoring API;
3. `YT-042 / M7i`: migrate the production Todo Browser local list path to the promoted primitives and prove cross-browser identity/lifecycle behavior;
4. `YT-043 / M7j`: run the complete regression, record bundle/performance changes and freeze the final M7 architecture decision.

General VDOM, signals dependency tracking, `ui!`, generic forms, full resumability and WebGPU remain outside these tasks.

## Reviewer gate

The Runner reported `coding_agent_providers: []`, so native ACP reviewer dispatch was unavailable. WebCodex therefore launched an ordinary role-directed local Codex Job using the canonical reviewer definition, not `codex exec review` and not native `agent_type` dispatch.

Reviewer Job `wc_job_DUHpN6tii7xSFCnD` used workspace model `gpt-6-astra`, reasoning effort `high`, approval policy `Never` and a read-only sandbox. It reached terminal exit code 0 with:

```json
{
  "overall_status": "pass",
  "issues": [],
  "required_followups": [],
  "optional_notes": []
}
```

The reviewer made no edit, stage, commit or push.
