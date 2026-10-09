# M7e Verification — Renderer Regression and Evidence Freeze

**Date:** 2026-10-09
**Task:** YT-038 / M7e
**Status:** Locally complete as a **no-decision evidence freeze**. No M7 experimental API was promoted into a production crate, and no production renderer winner is declared.

## Scope

M7e freezes the evidence produced by M7a–M7d and revalidates the complete adjacent regression surface before any renderer or public-API decision. The scope includes:

- native SSR, context-correct HTML escaping and strict CSP;
- `App`, `RequestContext`, native Axum/Tower middleware and typed action/API behavior;
- production Todo Browser and Search Island behavior in Chrome and Firefox;
- the isolated M7a render-strategy benchmark;
- the isolated M7b authoring, M7c local graph and M7d keyed lifecycle/form-state prototype;
- wasm32 compilation, dependency-graph isolation and byte-for-byte production asset isolation.

M7e adds no runtime implementation. The M7a–M7d source remains isolated under `examples/`, while this report records what may be worth a later extraction spike and what remains unready for promotion.

## Complete regression result

The required validation commands completed with the following results:

| Validation | Result |
|---|---|
| `cargo fmt --all -- --check` | PASS |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | PASS |
| `cargo test --workspace --all-targets --locked` | **86/86 PASS** |
| `cargo test --doc --workspace --locked` | **5/5 PASS** |
| wasm32 check for Todo Browser, Search Island, M7a and M7b/M7c/M7d | PASS |
| `python3 scripts/check_docs.py` after the M7e documentation update | PASS — 26 English documents, 38 backlog items |
| native Todo server build | PASS |
| production Chrome 155 debug regression | PASS |
| production Firefox 157 debug regression | PASS |
| M7a release build and 200 × 500 × 10 regression benchmark | PASS |
| M7b/M7c/M7d release build and Chrome 155 regression | PASS |
| M7b/M7c/M7d Firefox 157 release regression | PASS |
| production dependency graphs | PASS — exclude both M7 experimental crates |
| production release assets | PASS — exact before/after bytes unchanged |
| `git diff --check` and process teardown | PASS |

The production browser regressions cover native SSR without mandatory JS, HTML escaping, strict CSP, shared Todo output, local-only actions, typed HTTP mutation and errors, Search Island lazy activation, fail-closed state-version handling, and the absence of a browser-to-server event circuit. Workspace tests retain the `App`/handler, `RequestContext`, global/group/route middleware, request-isolation, typed action and API contracts.

## Production asset freeze

The release build retained digest `3f3c5f074478c525`. A snapshot captured before the isolated M7 builds was compared with `dist/` after the benchmark and keyed-binding regressions. All eight files were byte-identical:

| Asset | Bytes | SHA-256 |
|---|---:|---|
| `app.js` | 74 | `c41fe340ae8872cb3e40ffda728b7f8c07a52480079f8080242c86c8f9bf1d1e` |
| `favicon.svg` | 180 | `0beb89b1f412b3829b9dcf1e41b9671468d2cdb6b35e321330f954212d70cd7a` |
| `search-loader.js` | 1,144 | `7f44205cc0a112f5b8725c21195ebc814837920e903cbeddeb307c367cade169` |
| `styles.css` | 354 | `ccda441d1fa719ef17b45966017b79e51d9d39ed80f4baaf94e964571d30232e` |
| `yetlet_browser.js` | 18,199 | `d564f6b0a7ac6ffbf840905f145f34f13369de1fcc952d3467e342fd131a66d0` |
| `yetlet_browser_bg.wasm` | 216,006 | `c6189450b28a1b81c8bc241c72869458e4d862e6d34a8b4d83afe60dfa4c6a77` |
| `yetlet_search.js` | 14,699 | `1aca190b9e125e3e09f79c66be7c249f7be0fb794273575f547e6912f91e4f9b` |
| `yetlet_search_bg.wasm` | 179,716 | `ff46e70c132238db34765e9718692b8246359d2516e344258f464b5d6fc85db9` |

`cargo tree --locked --target wasm32-unknown-unknown -e normal` separately confirmed that `yetlet-todo-browser` and `yetlet-search-island` contain neither `yetlet-render-bench` nor `yetlet-keyed-bindings`.

## M7a evidence freeze

The identified M7a report remains the canonical measurement record. It was not replaced by the M7e rerun:

| Evidence | Timestamp | Subtree replacement | Positional View patch | Keyed direct DOM | Static template slots |
|---|---|---:|---:|---:|---:|
| Retained canonical report | `2026-10-09T02:02:22+00:00` | 213.500 ms | 41.700 ms | 2.000 ms | 1.900 ms |
| M7e regression rerun | `2026-10-09T05:16:46+00:00` | 213.800 ms | 41.700 ms | 1.900 ms | 1.900 ms |

Both runs used 200 keyed rows, 500 updates, 10 measured rounds and Chrome 155. The M7e run retained benchmark artifact digest `71182d03b13e4d13`, equivalent final states and the expected identity/focus classifications. The small timing differences demonstrate normal local-run variation; they do not change the architectural interpretation or select a production renderer.

## M7b–M7d isolated artifact freeze

The combined release artifact remains isolated at digest `c33a92498904a715`:

| Artifact | Bytes | Local gzip bytes |
|---|---:|---:|
| WASM | 241,470 | 69,694 |
| wasm-bindgen JavaScript module | 22,569 | 4,590 |
| external loader | 316 | 161 |
| generated HTML fixture | 2,185 | n/a |

Chrome and Firefox again passed SSR identity preflight, typed local actions, conditional text, FIFO batching, bounded re-entrancy, reducer/commit failure boundaries, keyed append/remove/reorder, controlled input value/focus/selection preservation, removed-listener cleanup and explicit graph/binding disposal. No API request or WebSocket event circuit was introduced.

## Renderer and API classification

### Candidate for a later promotion spike

The following are small enough to consider for extraction, but **none is promoted by M7e**:

| Candidate | Evidence supporting further work | Required before promotion |
|---|---|---|
| Validated component, instance and key identity value types | Native SSR and browser preflight use the same deterministic identity contract and fail closed on invalid or duplicate values | Separate the generic identity vocabulary from fixture-specific attributes and define compatibility/versioning rules |
| Safe typed slot descriptors | Text and inert attribute slots preserve escaping and reject reserved/unsafe attributes | Design a renderer-neutral descriptor set covering more than the current fixed row and prove nested composition |
| Keyed reconciliation primitive | Insert/remove/reorder preserves stable keyed nodes in both browsers | Separate the ordering/lifecycle algorithm from `BoundKeyedList`, define mutation-failure semantics and test general child shapes |
| Binding ownership and disposal primitive | Explicit ownership removes listeners and prevents stale handles from affecting a newer binding | Define a reusable owner token and lifecycle contract independent of the M7 fixture |
| Strong local owner with weak dispatch/subscription handles | Drop, disposal, capture release, batching and bounded re-entrancy are directly tested | Extract only the ownership pattern first; scheduler, async and public error semantics need separate design |

A later task may extract one or more of these as internal primitives. That task must define its own compatibility surface and tests; M7e does not create a new production crate or dependency.

### Must remain experimental

The following remain under `examples/keyed-bindings`:

- the fixed `ul > li` keyed-row authoring shape and current `KeyedListTemplate`/`BoundKeyedList` API;
- the current `text_input` contract and one-input-per-row limitation;
- the current synchronous `LocalGraph` API, pass scheduler and error model;
- the runtime static-template authoring surface;
- the connected-input focus/selection restoration policy;
- the single conditional text-slot shape and fixture-specific identity attributes/preflight schema.

These surfaces lack general nested component trees, arbitrary child topology, async effects, IME/composition, mobile/WebKit, broader form controls, transactional DOM rollback and a stable compatibility/versioning story.

### Explicitly rejected for immediate promotion or deferred

- A **general VDOM** is not selected as Yetlet's default renderer. The bounded positional prototype improved replacement cost but did not preserve semantic keyed identity during reorder, and one benchmark cannot justify a general runtime.
- A **general Signals framework** is not selected. M7c proves ownership and batching for one synchronous graph, not automatic dependency tracking, async resources or a complete scheduler.
- A `ui!` **procedural macro/static-template compiler** remains deferred until a renderer-neutral descriptor model and general composition rules exist.
- A **generic forms runtime** remains deferred beyond the single controlled text-input proof.
- **Full resumability** remains separate research; M7 state/identity evidence does not serialize arbitrary handlers or captures.
- **WebGPU renderer integration** remains a separate scene-renderer track and is not part of the HTML/DOM update decision.

“Rejected” here means rejected for immediate M7 promotion, not permanently forbidden. Any future proposal requires a new bounded task and evidence rather than inheriting a winner from the M7a timing table.

## Architectural outcome

M7e is deliberately a **no-decision evidence freeze**:

1. The current production Todo Browser and Search Island remain unchanged and continue using their existing renderer behavior.
2. No M7 experimental API moves into `yetlet-core`, `yetlet-dom` or another production crate.
3. ADR 0003 is updated with the completed evidence and no-selection outcome.
4. ADR 0007 is not created because M7e makes no new architecture choice.
5. YT-038 may be marked Done because the required regression, artifact isolation and decision classification now have retained evidence.

## Validation execution note

The terminal WebCodex validation Job `wc_job_mRFAbdfyD5Om1i06` passed every required build, Rust test, browser regression, benchmark and dependency-graph check. Its final aggregate helper then exited because importing `scripts/measure_render_strategies.py` from the repository root could not resolve the sibling `smoke_browser` module. This was an evidence-harness import-path error after the product checks, not a product failure.

A bounded recovery used a standalone asset reader to perform the remaining exact byte comparison, parse the 86/5 test totals, verify the retained and rerun benchmark reports, verify the isolated artifact sizes, run `git diff --check`, and confirm process teardown. No covered source changed between the successful required commands and that recovery.

## Remaining limits

This is local prototype evidence, not public release, production security certification, a cross-device performance study or an API stability promise. Chrome and Firefox are covered; WebKit, mobile browsers, real IME composition, assistive-technology breadth, long-lived heap behavior, layout/paint profiling and cross-framework comparison remain open. Public-release item YT-020 remains separately blocked by licensing and support/security policy decisions.

## Reviewer gate

After the source freeze, the WebCodex Runner still reported `coding_agent_providers: []`, so native ACP reviewer dispatch was unavailable. Repository policy therefore permitted an ordinary role-directed local Codex Job, not `codex exec review` and not native `agent_type` dispatch.

WebCodex Job `wc_job_9hT23HL0mwaTto4S` executed the canonical reviewer role with workspace model `gpt-6-astra`, reasoning effort `high`, approval policy `Never` and a read-only sandbox. It reached terminal exit code 0 with:

```json
{
  "overall_status": "pass",
  "issues": [],
  "required_followups": [],
  "optional_notes": []
}
```

The reviewer made no edit, stage, commit or push.
