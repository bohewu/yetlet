# M7g Verification — Optional Synchronous Local Graph

**Date:** 2026-10-09
**Task:** YT-040 / M7g
**Status:** Locally complete. The proven strong-owner/weak-handle synchronous graph now lives in the optional `yetlet-reactive` crate; this does not establish a general signals framework or async scheduler.

## Outcome

M7c and M7d exercised one explicitly owned local graph inside the isolated keyed-binding fixture. M7g moves that exact implementation and its thirteen ownership, batching, re-entrancy, failure and teardown tests into `crates/yetlet-reactive`.

The production crate provides:

- one non-cloneable `LocalGraph<State, Message, Error>` strong owner;
- weak `GraphHandle` dispatch capabilities;
- weak `Subscription` ownership with explicit unsubscribe and drop cleanup;
- FIFO `dispatch` and `batch` processing;
- one reducer draft and one commit/notification per successful pass;
- deferred, bounded re-entrant messages;
- recoverable reducer errors that preserve the previous committed state;
- faulted commit/pass-limit behavior;
- explicit disposal and deterministic capture release outside active `RefCell` borrows.

`examples/keyed-bindings::reactive` is now only a compatibility re-export of `yetlet-reactive`; there is no duplicate graph implementation.

## Boundary

`yetlet-reactive` has no dependencies and contains no browser, server, renderer, HTTP, WebSocket, task runtime or persistence code. It is synchronous and single-threaded by design. It does not provide field-level dependency tracking, automatic effects, resources, async scheduling, workers, server synchronization or a `Signal` API.

The crate is optional: neither `yetlet-core`, `yetlet-html`, `yetlet-dom`, Search Island nor the current production Todo Browser depends on it at this gate. Production adoption is deferred to YT-042 after the independent keyed DOM primitive exists.

## Validation

Cargo.lock was refreshed once for the new path-only workspace member, followed by locked validation:

```bash
cargo fmt --all -- --check
cargo clippy --locked -p yetlet-reactive -p yetlet-keyed-bindings --all-targets -- -D warnings
cargo test --locked -p yetlet-reactive -p yetlet-keyed-bindings --all-targets
cargo test --locked -p yetlet-reactive -p yetlet-keyed-bindings --doc
cargo check --locked -p yetlet-reactive -p yetlet-keyed-bindings --target wasm32-unknown-unknown
python3 scripts/check_docs.py
git diff --check
```

Observed before reviewer dispatch:

- `yetlet-reactive`: **13/13 tests PASS**;
- experimental keyed authoring fixture: **4/4 tests PASS**;
- keyed-bindings rustdoc: **1/1 PASS**;
- strict targeted Clippy, native/wasm32 compile, formatting and whitespace checks: PASS;
- no browser/server process was launched.

## Remaining M7 sequence

- `YT-041 / M7h`: general keyed DOM reconciliation and explicit binding/listener ownership;
- `YT-042 / M7i`: production Todo Browser adoption with Chrome/Firefox identity and lifecycle evidence;
- `YT-043 / M7j`: complete regression, bundle/performance comparison and final architecture freeze.

The fixed `KeyedListTemplate`, current controlled-input API, focus policy, conditional slot shape, general VDOM, macro DSL, generic forms and full resumability remain outside this promotion.

## Reviewer gate

The Runner still reported `coding_agent_providers: []`; native ACP reviewer dispatch was unavailable. WebCodex launched an ordinary role-directed local Codex Job using the canonical reviewer definition, not `codex exec review` and not native `agent_type` dispatch.

Reviewer Job `wc_job_GnG93jQMHK4dTQv7` used workspace model `gpt-6-astra`, reasoning effort `high`, approval policy `Never` and a read-only sandbox. It reached terminal exit code 0 with:

```json
{
  "overall_status": "pass",
  "issues": [],
  "required_followups": [],
  "optional_notes": []
}
```

The reviewer made no edit, stage, commit or push.
