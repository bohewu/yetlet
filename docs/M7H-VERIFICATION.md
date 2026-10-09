# M7h Verification — Optional Keyed DOM Ownership

**Date:** 2026-10-09
**Task:** YT-041 / M7h
**Status:** Locally complete. General keyed child ordering, node-state ownership and browser listener disposal now live in the optional `yetlet-dom-keyed` crate; the fixed M7b authoring surface remains experimental.

## Outcome

M7h extracts the independently useful DOM mechanisms demonstrated by M7a–M7d without promoting `KeyedListTemplate`, the current text-input API or a template compiler. The optional crate provides:

- `validate_keyed_order`, using the renderer-neutral `BindingKey` contract and rejecting duplicates before browser work;
- `KeyedEntry<State>`, which pairs one element with caller-owned cached slots, listener owners or other row state;
- `KeyedChildren<State>`, which owns the complete key-to-entry set under one direct-child root;
- `DomEventListener`, an explicit listener owner that removes its callback on `dispose` or drop;
- `ReconcileStats` for retained, inserted, removed and moved entries.

The existing M7b–M7d browser fixture now drives its fixed row shape through `KeyedChildren` and stores input/click listener owners inside each `KeyedEntry`. There is no duplicate reconciliation or listener-lifetime implementation in the fixture.

## Reconciliation contract

Before structural mutation, `KeyedChildren::reconcile`:

1. derives and validates every desired key;
2. rejects duplicate keys;
3. creates every missing entry as a detached element;
4. rejects a create callback that returns an already attached element;
5. runs caller updates for retained entries.

It then removes stale nodes, inserts new nodes and moves retained nodes into the requested order. Semantic keys preserve element and caller-state identity across reorder. Removing an entry drops its cached state and listener owners after the node is disconnected. Explicit owner disposal drops all entry state/listeners but deliberately leaves the caller-owned DOM subtree in place.

The mutation boundary is explicit rather than overstated: key/create failures occur before structural changes, but caller update callbacks may mutate retained nodes before returning an error. An unexpected browser DOM API error during the final remove/insert phase has no transactional rollback. Callers that require atomic slot updates must supply fail-before-mutate update logic.

## Isolation decision

The first implementation placed the primitive directly in `yetlet-dom`. Even without a production call site, broadening that existing crate changed compiled production asset bytes:

- direct integration with additional `web-sys` features produced digest `6895d575d9c6a494`;
- feature-gating the API still changed dependency crate metadata and produced digest `fdb6f0049bde5083`.

M7h therefore moved the implementation into independent `yetlet-dom-keyed`. Todo Browser and Search Island do not depend on it yet. Rebuilding production assets then restored the complete M7e digest and hashes:

```text
3f3c5f074478c525
```

This is an evidence-based pay-for-what-you-use crate boundary, not a claim that the primitive is zero-cost after an application opts in. Production adoption and its actual bundle delta belong to YT-042.

## Cross-browser evidence

The release-profile isolated fixture passed in Chrome 155 and Firefox 157. It verifies:

- complete SSR identity/shape preflight before binding;
- retained root and row identity;
- keyed reverse, append and removal;
- controlled text input value, focus and selection preservation through reorder;
- focused keyed-row removal;
- detached input and click listeners no longer dispatch;
- explicit graph/binding disposal removes later propagation;
- reducer and commit failures remain fail-closed at the documented boundary;
- no API request, WebSocket or remote event circuit.

## Isolated release artifact

| Field | M7h observed value | M7d retained value | Delta |
|---|---:|---:|---:|
| Content digest | `6754025153168694` | `c33a92498904a715` | changed as expected |
| WASM | 272,728 bytes | 241,470 bytes | +31,258 |
| WASM local gzip | 75,818 bytes | 69,694 bytes | +6,124 |
| wasm-bindgen JavaScript module | 22,749 bytes | 22,569 bytes | +180 |
| JavaScript local gzip | 4,616 bytes | 4,590 bytes | +26 |
| External loader | 316 bytes | 316 bytes | 0 |
| Generated fixture HTML | 2,185 bytes | 2,185 bytes | 0 |

These are isolated fixture measurements, not production transfer sizes or a public budget. The increase includes the promoted generic owner/reconciler used by the fixture. Todo Browser adoption is measured separately in M7i.

## Validation

```bash
cargo fmt --all -- --check
cargo check --locked -p yetlet-dom
cargo clippy --locked -p yetlet-dom-keyed -p yetlet-keyed-bindings --all-targets -- -D warnings
cargo test --locked -p yetlet-dom-keyed -p yetlet-keyed-bindings --all-targets
cargo test --locked -p yetlet-dom-keyed -p yetlet-keyed-bindings --doc
cargo check --locked -p yetlet-dom-keyed -p yetlet-keyed-bindings --target wasm32-unknown-unknown
python3 scripts/build_browser.py --release
python3 scripts/build_keyed_bindings.py --release
python3 scripts/smoke_keyed_bindings.py --profile release
python3 scripts/smoke_keyed_bindings_firefox.py --profile release
python3 scripts/check_docs.py
git diff --check
```

Observed before reviewer dispatch:

- `yetlet-dom-keyed`: **1/1 native order-contract test PASS**;
- experimental keyed authoring fixture: **4/4 native tests PASS** and **1/1 doctest PASS**;
- strict targeted Clippy and native/wasm32 compile: PASS;
- Chrome 155 and Firefox 157 release lifecycle suites: PASS;
- production release digest: `3f3c5f074478c525`, byte-isolated from the optional crate;
- no browser/server process remained active after the scripts completed.

## Remaining M7 sequence

- `YT-042 / M7i`: adopt `yetlet-dom-keyed` and `yetlet-reactive` in the production Todo Browser local list path, then measure the real bundle delta;
- `YT-043 / M7j`: complete regression, performance/bundle comparison and final architecture decision freeze.

General VDOM, compile-time templates, dependency-tracked signals, generic forms, full resumability and WebGPU remain outside this primitive.

## Reviewer gate

The Runner reported `coding_agent_providers: []`, so native ACP reviewer dispatch remained unavailable. WebCodex launched an ordinary role-directed local Codex Job using the canonical reviewer definition, not `codex exec review` and not native `agent_type` dispatch.

Reviewer Job `wc_job_IpxKPD3wwMQto2FU` used workspace model `gpt-6-astra`, reasoning effort `high`, approval policy `Never` and a read-only sandbox. It reached terminal exit code 0 with:

```json
{
  "overall_status": "pass",
  "issues": [],
  "required_followups": [],
  "optional_notes": []
}
```

The reviewer independently rechecked production asset hashes, locked dependency isolation, the restored `yetlet-dom` files and the current diff. It made no edit, stage, commit or push.
