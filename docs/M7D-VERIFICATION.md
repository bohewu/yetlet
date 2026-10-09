# M7d Verification — Keyed Lifecycle and Form-State Preservation

**Date:** 2026-10-09
**Task:** YT-037 / M7d
**Status:** Implemented and locally verified in the isolated `yetlet-keyed-bindings` prototype. This is not a stable form/component API, a general renderer, or production browser certification.

## Scope

M7d extends the fixed M7b keyed-list shape with one optional controlled text input per row and verifies lifecycle behavior in both Chrome and Firefox. The production Todo Browser and Search Island remain unchanged and do not depend on this prototype.

The bounded row shape is now:

```text
ul
└─ li[data-yetlet-key]
   ├─ label text slot
   ├─ optional input[type=text]
   ├─ optional conditional text slot
   └─ optional local-action button
```

## Authoring and SSR contract

`KeyedListTemplate::text_input` declares:

- a unique static slot name;
- an accessible non-empty `aria-label`;
- a model-to-string value function;
- a typed `(key, value) -> Message` constructor.

Native SSR writes `type="text"`, `data-yetlet-input`, `aria-label`, and an escaped `value` attribute. Duplicate inputs, ambiguous slot names, empty/control-character labels, reserved attributes and duplicate keys fail before rendering.

Browser preflight checks the exact input element shape, static attributes, escaped initial attribute, live value, and absence of unexpected children before installing any listener. A mismatch fails closed without replacing or partially binding the SSR tree.

## Controlled input and selection semantics

The browser binding stores the last committed model value for each input. On local `input`:

1. the existing DOM input produces a typed `DemoMessage::Edit(key, value)`;
2. the M7c graph reduces and commits the new model once;
3. the keyed row updates `data-draft` and the input's serialized value;
4. the live `value` property is written only when the committed model actually differs from the value already in the input.

Avoiding a redundant live-value write preserves the current caret/selection during controlled input. Before structural reconciliation, the binding captures a focused descendant text input and its selection start/end/direction. After successful keyed moves, it restores focus and selection only when the same input remains connected. Removing the focused keyed row intentionally does not restore focus to a detached node.

## Listener ownership and cleanup

Input and click listeners are owned by the corresponding `BoundRow` in a vector of event bindings. Removing a keyed row drops that row and removes both listener types. Explicit binding/graph disposal removes all remaining row listeners and ownership markers.

Chrome and Firefox retain detached input/button references during tests. Dispatching events on those detached nodes after removal, or on retained nodes after disposal, does not change graph commit counts, does not set the event-error marker, and does not produce an API request or WebSocket.

## Cross-browser evidence

### Chrome 155

The isolated Chrome E2E verifies:

- controlled input updates only the local graph;
- input node identity, live value, focus, and selection range survive keyed reverse and restoration;
- append and removal retain unrelated keyed rows;
- removing the focused row disconnects it and clears active focus;
- detached input/click events do not propagate;
- graph/binding disposal removes later input and click behavior;
- no HTTP request, WebSocket, failed browser resource, or JavaScript exception occurs after loading.

### Firefox 157

A separate WebDriver E2E uses real key input to replace the row draft, then verifies the same node/focus/value/selection behavior across keyed reorder. It also verifies focused-row removal, detached listener cleanup, disposal cleanup, and no `/api/` resource request.

This does not constitute an IME composition, mobile browser, screen-reader, or broad accessibility matrix.

## Commands

```bash
cargo fmt --all -- --check
cargo clippy --locked -p yetlet-keyed-bindings --all-targets -- -D warnings
cargo test --locked -p yetlet-keyed-bindings --all-targets
cargo test --locked -p yetlet-keyed-bindings --doc
cargo check --locked -p yetlet-keyed-bindings --target wasm32-unknown-unknown
python3 scripts/build_keyed_bindings.py --release
python3 scripts/smoke_keyed_bindings.py --profile release
python3 scripts/smoke_keyed_bindings_firefox.py --profile release
```

## Isolated release artifact

| Field | Observed value |
|---|---:|
| M7b/M7c/M7d content digest | `c33a92498904a715` |
| WASM | 241,470 bytes; 69,694 local gzip bytes |
| wasm-bindgen JavaScript module | 22,569 bytes; 4,590 local gzip bytes |
| External loader | 316 bytes; 161 local gzip bytes |
| Generated fixture HTML | 2,185 bytes |

These are isolated local artifact sizes, not transfer measurements or a production budget. The production Browser/Search release digest remains separately checked at `3f3c5f074478c525`; their dependency graphs exclude both M7 experimental crates.

## Final closeout evidence

After the M7d source and documentation were frozen, WebCodex ran one complete validation Job (`wc_job_7ZxB5yxZJF5Zpn9e`) to terminal success.

| Final check | Observed result |
|---|---|
| `cargo fmt --all -- --check` | PASS |
| `python3 scripts/check_docs.py` | PASS — 25 English documents, 38 backlog items, links/Markdown/whitespace/DAG valid |
| strict workspace Clippy | PASS with `-D warnings` |
| workspace unit/integration tests | **86/86 PASS** |
| workspace doctests | **5/5 PASS** |
| wasm32 checks | Todo Browser, Search Island, M7a benchmark, and M7b/M7c/M7d prototype PASS |
| existing production Chrome regression | Chrome 155 Todo/SSR/Search/API/CSP E2E PASS on debug artifacts |
| existing production Firefox regression | Firefox 157 Todo/SSR/Search/API/keyboard E2E PASS on debug artifacts |
| isolated lifecycle regressions | M7d Chrome 155 and Firefox 157 release-profile E2E PASS |
| production release assets | Digest and every captured asset remain byte-identical at `3f3c5f074478c525` |
| dependency isolation | Todo Browser and Search Island graphs exclude both M7 experimental crates |
| artifact/document consistency | M7d digest, byte counts and gzip estimates match generated release output |
| `git diff --check` and teardown | PASS; no browser/server/benchmark Job remained active at validation completion |

The production Chrome/Firefox rows are the pre-existing debug-profile regression suites. Release-profile cross-browser evidence applies to the isolated M7d fixture; production release compatibility is separately supported by byte-identical Browser/Search assets.

## Limitations and M7e boundary

- Only one optional text input is supported per fixed row. Checkbox/radio/select/textarea, validation UI, nested forms, composition events and arbitrary children are not covered.
- Focus restoration is a bounded successful-reconcile policy for a connected text input. It is not a general focus manager.
- A DOM API error after structural mutation still has no transactional rollback.
- Chrome and Firefox headless behavior is covered; WebKit/mobile and real IME composition remain unverified.
- The authoring, reactivity and lifecycle APIs remain experimental and isolated from production routes.

The next task is **YT-038 / M7e**: freeze M7 evidence, rerun complete native/SSR/WASM/CSP/bundle/performance contracts, and decide whether any bounded API is ready to move out of `examples/`.
