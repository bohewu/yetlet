# M7i Verification — Production Todo Keyed Graph Adoption

**Date:** 2026-10-09
**Task:** YT-042 / M7i
**Status:** Locally complete, including the later M7j reviewer repair. The production Todo Browser local-list path uses the promoted optional graph and keyed DOM primitives; the Search Island and explicit HTTP action path remain separate.

## Production integration

`examples/todo-browser` now opts into `yetlet-reactive` and `yetlet-dom-keyed` for the local Todo list only.

At mount time the route still renders the complete shared `browser_page` through the existing direct DOM renderer. The runtime then preflights the local list before ownership:

- exactly one `ul.todo-list` under `#todo-list-host`;
- the complete expected keyed row set and order;
- each row's `data-id`, `data-completed`, class and exact text-node shape;
- validated `BindingKey` values with no duplicates.

A successful bind stores each existing row and text node in `KeyedChildren<TodoRow>`. The original SSR-equivalent nodes are reused. Local `TodoCommand` values dispatch through one strongly owned `LocalGraph<TodoSnapshot, TodoCommand, ActionError>`; its commit callback reconciles the keyed rows and publishes the committed snapshot only after a successful DOM commit.

A small `LocalGraphExecutor` continues to implement the existing `ActionExecutor<TodoCommand>` boundary. It owns only a weak graph handle plus the last successful committed snapshot. The route-level `BrowserRuntime` is the explicit strong owner of the graph, all `DomEventListener` instances and one shared runtime-lifetime token.

The remote Create path remains an explicit `HttpExecutor` POST to `/api/todos`. Its returned server snapshot still renders into the separate remote-list host through the existing subtree mount. No DOM event, graph message or state snapshot becomes a remote event circuit.

## Lifecycle contract

The generated external `app.js` initializes the wasm-bindgen module once and exposes a frozen internal lifecycle bridge at `globalThis.__yetletTodoRuntime`. This bridge contains only the already exported `mount` and `dispose` functions and is not declared a stable framework API.

`dispose` first invalidates the runtime-lifetime token, then drops the strong graph owner, weak-dispatch target, keyed entry state and local/remote listener owners. It deliberately leaves the current DOM readable. A queued local or remote task checks the token before starting; an in-flight HTTP task checks it again after `await` before rendering or changing status. Chrome and Firefox prove click→dispose in one JavaScript task issues no request or mutation, a delayed HTTP completion cannot update retired DOM after remount, clicks after disposal remain inert, and a later `mount` creates a fresh graph/token with exactly one local and one remote listener path.

## Browser and transport evidence

Chrome 155 and Firefox 157 verify all of the following against the production `/app` route:

- initial Browser DOM remains byte-equal to the shared native TodoList HTML;
- the local list root and keyed row 1 remain the same JavaScript objects across two toggles;
- local toggles update text/class/attributes through typed graph messages and issue no HTTP request;
- `dispose` removes both button listeners while keeping the current row connected and readable;
- local and remote clicks queued immediately before `dispose` do not start after retirement;
- an HTTP response held until after disposal/remount cannot update either retired or current DOM;
- disposed local and remote clicks produce no state, status or request change;
- remount creates fresh route nodes and one local click performs exactly one transition;
- one valid remote click after remount performs exactly one POST and advances the server from revision 1 to revision 2;
- strict CSP, escaped SSR, typed 422 errors, Search Island lazy activation and no WebSocket behavior continue to pass.

The Chrome run uses release browser assets with the ordinary debug native smoke server. The Firefox run uses the release server/profile path.

## Build isolation

Building Todo Browser and Search Island in one Cargo invocation unified the Todo route's expanded `web-sys` features into the Search build. Product behavior was unchanged, but the generated Search JavaScript changed by three bytes. `scripts/build_browser.py` now builds each browser entry in a separate locked Cargo invocation before running wasm-bindgen.

After that correction, Search remains byte-identical to the M7e baseline:

| Search asset | Bytes | Local gzip | SHA-256 |
|---|---:|---:|---|
| `yetlet_search.js` | 14,699 | 3,675 | `1aca190b9e125e3e09f79c66be7c249f7be0fb794273575f547e6912f91e4f9b` |
| `yetlet_search_bg.wasm` | 179,716 | 70,806 | `ff46e70c132238db34765e9718692b8246359d2516e344258f464b5d6fc85db9` |

The Search dependency graph excludes both `yetlet-reactive` and `yetlet-dom-keyed`.

## Production Browser artifact delta

The current repaired content digest changes from the pre-adoption M7e baseline `3f3c5f074478c525` to:

```text
faebb0709dccd863
```

| Browser runtime asset | M7e raw / gzip | Repaired M7i/M7j raw / gzip | Delta raw / gzip |
|---|---:|---:|---:|
| `app.js` | 74 / 84 | 151 / 138 | +77 / +54 |
| `yetlet_browser.js` | 18,199 / 4,220 | 22,299 / 4,792 | +4,100 / +572 |
| `yetlet_browser_bg.wasm` | 216,006 / 77,003 | 294,555 / 96,419 | +78,549 / +19,416 |
| **Browser runtime total** | **234,279 / 81,307** | **317,005 / 101,349** | **+82,726 / +20,042** |

These are deterministic local gzip estimates, not network transfer measurements or a production budget. The increase is accepted for this bounded evidence slice because the production route now contains the generic keyed owner, synchronous graph, explicit listener ownership and lifecycle exports. M7j retains the full baseline/performance comparison and final decision boundary.

## Validation

```bash
cargo fmt --all -- --check
cargo clippy --locked -p yetlet-todo-browser --all-targets -- -D warnings
cargo check --locked -p yetlet-todo-browser --target wasm32-unknown-unknown
cargo test --locked -p yetlet-todo-browser --all-targets
python3 -m py_compile scripts/build_browser.py scripts/smoke_browser.py scripts/smoke_firefox.py
python3 scripts/build_browser.py --release
cargo build --locked -p yetlet-todo-server
cargo build --release --locked -p yetlet-todo-server
python3 scripts/smoke_browser.py
python3 scripts/smoke_firefox.py --profile release
cargo tree --locked -p yetlet-todo-browser
cargo tree --locked -p yetlet-search-island
python3 scripts/check_docs.py
git diff --check
```

Observed before reviewer dispatch:

- strict targeted Clippy, native target and wasm32 compile: PASS;
- Chrome 155 release-asset production regression: PASS;
- Firefox 157 release production regression: PASS;
- Todo Browser dependency graph includes `yetlet-reactive` and `yetlet-dom-keyed`;
- Search Island dependency graph excludes both optional crates;
- Search JS/WASM and all common static assets remain byte-identical;
- production Browser release digest and exact byte/gzip deltas match the table above;
- no browser, driver or server validation process remained active.

## Remaining boundary

M7i adopts a low-level keyed list path, not the fixed experimental `KeyedListTemplate` API. The remote result list and Search result updates still use bounded owned-subtree replacement. General nested component trees, compile-time static templates, dependency-tracked signals/effects, generic forms, IME/mobile/WebKit qualification, full resumability and WebGPU remain deferred.

`YT-043 / M7j` must run the complete workspace/native/wasm32/CSP/API/cross-browser/benchmark/bundle regression and record the final M7 architecture decision before the milestone closes.

## Reviewer gate

The Runner reported `coding_agent_providers: []`, so native ACP reviewer dispatch remained unavailable. WebCodex launched an ordinary role-directed local Codex Job using the canonical reviewer definition, not `codex exec review` and not native `agent_type` dispatch.

Reviewer Job `wc_job_h8nwZIFRImFvMYhG` used workspace model `gpt-6-astra`, reasoning effort `high`, approval policy `Never` and a read-only sandbox. It reached terminal exit code 0 with:

```json
{
  "overall_status": "pass",
  "issues": [],
  "required_followups": [],
  "optional_notes": []
}
```

The reviewer independently checked the production dependency trees, source diff, unstaged/untracked hygiene and absence of validation processes. It made no edit, stage, commit or push.

A later final M7j reviewer Job, `wc_job_orP7QpVjLsh07HWI`, found one P2 that the narrower M7i gate had missed: `spawn_local` tasks queued before disposal and HTTP responses completing after disposal could still act on retired state/DOM. The current lifetime token and focused Chrome/Firefox debug/release regressions repair that finding. Final acceptance and the repeat reviewer are recorded in [M7j verification](M7J-VERIFICATION.md).
