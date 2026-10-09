# M8e Verification — Final Sample Architecture and Evidence Freeze

**Date:** 2026-10-09
**Task:** YT-048 / M8e
**Status:** Locally complete. Final source/evidence freeze and repeat independent reviewer PASS are recorded.

## Final M8 decision

M8 closes the sample-driven application-experience track around three deliberately different applications:

- Todo proves SSR plus an explicit Browser/WASM runtime, lazy Search Island, typed local/HTTP actions and optional keyed/local-reactive capabilities;
- REST proves a one-package native JSON API with ordinary handlers, extractors, middleware, explicit errors and no browser dependency;
- Session Cart proves a one-package native SSR/forms application that explicitly selects the independent server-side Session provider.

The architecture remains **handler-first, component-first, middleware-native and pay-for-what-you-use**. M8 promotes only the bounded `yetlet-session` provider already recorded by ADR 0005. It does not add Controllers, DI, a procedural authoring macro, generic forms, a shared browser circuit, default Session state or a new application layout convention. M8d's measured audit found no repeated authoring friction sufficient to change the runtime API, so no new ADR is required.

This is an experimental prototype evidence freeze, not a stable/public release decision. YT-020 remains independently blocked by licensing, support policy and production security/compatibility qualification.

## Complete validation matrix

```bash
cargo fmt --all -- --check
python3 -m py_compile scripts/*.py
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --all-targets --locked
cargo test --doc --workspace --locked
cargo check --locked \
  -p yetlet-todo-browser \
  -p yetlet-search-island \
  -p yetlet-render-bench \
  -p yetlet-keyed-bindings \
  -p yetlet-dom-keyed \
  -p yetlet-reactive \
  --target wasm32-unknown-unknown

python3 scripts/build_browser.py
cargo build --locked -p yetlet-todo-server
python3 scripts/smoke_browser.py
python3 scripts/smoke_firefox.py --profile debug
cargo build --locked -p yetlet-rest-api -p yetlet-cart-server
python3 scripts/smoke_rest_api.py
python3 scripts/smoke_cart.py

python3 scripts/build_browser.py --release
cargo build --release --locked -p yetlet-todo-server
python3 scripts/smoke_browser.py
python3 scripts/smoke_firefox.py --profile release
python3 scripts/measure_baseline.py --profile release --samples 30 \
  --browser-runs 5 --out target/m8e-baseline-release.json

python3 scripts/measure_render_strategies.py \
  --capture-production-before target/m8e-production-before.json
python3 scripts/build_render_bench.py --release
python3 scripts/measure_render_strategies.py --profile release \
  --rows 200 --updates 500 --rounds 10 --seed 7 \
  --production-before target/m8e-production-before.json \
  --out target/m8e-render-strategies.json

python3 scripts/build_keyed_bindings.py --release
python3 scripts/smoke_keyed_bindings.py --profile release
python3 scripts/smoke_keyed_bindings_firefox.py --profile release

python3 scripts/measure_sample_dx.py --out target/m8e-sample-dx.json
python3 scripts/check_samples.py
python3 scripts/test_check_samples.py
python3 scripts/check_docs.py
git diff --check
```

## Frozen validation result

Complete validation Job `wc_job_9CsJkTO3rMLY7Z5Q` ran the matrix above as one bounded execution and reached terminal exit code 0 with 113 workspace tests. After the final reviewer found one Session-only stale-destroy path, the focused Session/Cart checks, Cart smoke, complete workspace tests/doctests, native dependency isolation and production-asset hashes were rerun. The final source now passes 114 workspace tests; the browser matrix was not duplicated because the changed crate is excluded from Browser/Search graphs and all frozen production bytes remain identical. Neither execution edited, staged, committed, pushed, reset or switched worktrees.

| Gate | Result |
|---|---|
| Workspace format / strict Clippy | PASS |
| Workspace unit/integration tests | **114/114 PASS** |
| Workspace doctests | **5/5 PASS** |
| Six-package wasm32 check | PASS |
| Production Chrome 155 debug / release | PASS / PASS |
| Production Firefox 157 debug / release | PASS / PASS |
| REST localhost CRUD/policy smoke | PASS |
| Cart localhost Session/CSRF smoke | PASS |
| M7a 200 × 500 × 10 release benchmark | PASS |
| Isolated keyed lifecycle Chrome / Firefox release | PASS / PASS |
| Three-sample catalog / focused checker | **3 samples / 4 tests PASS** |
| Documentation / backlog graph | **38 documents / 48 items PASS** |
| Dependency and production-asset isolation | PASS |
| Tracked / untracked whitespace | **PASS / 24 files PASS** |
| Owned server/browser/driver cleanup | PASS |
| Staged files / HEAD / worktree | None / unchanged / original worktree |

Production regressions retain native SSR/escaping, strict CSP, handler-first App/RequestContext/middleware, typed local and HTTP actions, Search lazy activation/fail-closed state, keyed identity, queued/in-flight lifetime retirement, explicit disposal and no WebSocket/raw-event circuit. REST and Cart add independent native-only CRUD, form, Session, CSRF, rotation/revocation and bounded-state coverage.

## Production bundle freeze

The release digest remains byte-identical to M7j before native sample builds, after the final Browser rebuild, and after the isolated benchmark/keyed builds:

```text
faebb0709dccd863
```

The comparison includes every referenced asset plus `dist/manifest.json` and `dist/app.html`.

| Todo Browser runtime asset | Raw bytes | Local gzip bytes | SHA-256 |
|---|---:|---:|---|
| `app.js` | 151 | 138 | `dc88e7c082b7fbd8670a9d4adba1d3fc11018ee47f19f59cbba37139fec22f9a` |
| `yetlet_browser.js` | 22,299 | 4,792 | `1b3602c227214b5d4b9cca505e94876ab2d66367b2c96fb613e28fb47b940ae8` |
| `yetlet_browser_bg.wasm` | 294,555 | 96,419 | `ca595a6fa978f8eb3c6a4d10d002b96316f7f4419addf44708b0b290296c8cb6` |
| **Runtime total** | **317,005** | **101,349** | — |

| Search asset | Raw bytes | Local gzip bytes | SHA-256 |
|---|---:|---:|---|
| `search-loader.js` | 1,144 | 551 | `7f44205cc0a112f5b8725c21195ebc814837920e903cbeddeb307c367cade169` |
| `yetlet_search.js` | 14,699 | 3,675 | `1aca190b9e125e3e09f79c66be7c249f7be0fb794273575f547e6912f91e4f9b` |
| `yetlet_search_bg.wasm` | 179,716 | 70,806 | `ff46e70c132238db34765e9718692b8246359d2516e344258f464b5d6fc85db9` |

The activated Search runtime remains 74,481 local-gzip bytes, with a separate 551-byte initial loader estimate. These are deterministic local gzip estimates, not wire-transfer sizes or a bundle budget.

## Dependency isolation

The final Cargo-tree assertions establish:

- Todo Browser includes `yetlet-reactive` and `yetlet-dom-keyed`, but excludes Session, REST and Cart;
- Search excludes keyed/reactive, Session and request correlation;
- Todo Server and REST select `yetlet-request-id` but exclude Session;
- Cart selects `yetlet-session` but excludes request correlation and every browser/WASM dependency;
- `yetlet-session` is native-only;
- `yetlet-server-axum` and `yetlet-core` depend on neither optional provider.

The native-only M8 packages therefore do not perturb Browser/Search feature resolution or bytes.

## Repeated local baseline

Environment: Linux x86_64, rustc 1.99.0, Python 3.14.4 and Chrome 155.0.8059.39. Cache was disabled. These are bounded local-loopback observations, not Web Vitals, statistical equivalence, a production SLA or a cross-framework comparison.

| Measurement | Samples | Median | p95 | Range |
|---|---:|---:|---:|---:|
| SSR request to first response byte | 30 | **0.034 ms** | 0.072 ms | 0.032–0.082 ms |
| Chrome `/app` navigation to ready | 5 | **48.8 ms** | 51.4 ms | 48.3–51.4 ms |
| Search trigger to ready | 5 | **3.4 ms** | 4.2 ms | 3.3–4.2 ms |

The M7j medians were 0.037, 49.0 and 3.5 ms respectively. The small sequential rerun shows no material local change, but does not prove equivalence.

## Retained renderer and authoring fixtures

The full isolated M7a workload again produced equivalent final state and preserved production assets:

| Strategy | M8e median round total |
|---|---:|
| Subtree replacement | 215.9 ms |
| Positional View patch | 41.5 ms |
| Keyed direct DOM | 1.9 ms |
| Static template slots | 1.75 ms |

Benchmark digest remains `71182d03b13e4d13`; release WASM is 165,287 bytes. The result does not select a general renderer.

The isolated keyed authoring/reactivity/lifecycle artifact remains digest `6754025153168694` with 272,728-byte release WASM. Chrome 155 and Firefox 157 release suites pass controlled-input identity/focus/selection, batching, failure, removal and listener-disposal behavior.

## Sample/DX and repository freeze

The M8d measurement reproduces the retained three rows exactly: Todo 4 application packages / 1,102 production non-blank lines / 16 direct dependency names; REST 1 / 551 / 6; Cart 1 / 424 / 6. These figures remain evidence of shape and isolation, not quality or performance.

No new owned browser/server/driver process remained relative to the pre-run process baseline. The index is empty, branch is `master`, HEAD is still `828d8a7fa484e1e08486850ba8159777d8667385`, and the original worktree is unchanged.

## Reviewer gate

First final independent reviewer Job `wc_job_dospP8AM0wUmvHFd` used `gpt-6-astra`, high effort and a read-only sandbox. It verified the complete retained matrix, hashes, dependency graphs and repository state, but found one material P2: a stale request could call `destroy()` after another request rotated the same identity, queue a removal cookie despite removing no record, and delete the replacement browser cookie if its response arrived last.

The repair makes `destroy()` clear the browser cookie only after successful removal of the exact server record. An already-retired identity now returns `SessionError::Destroyed`, sets no cookie directive and cannot disturb the replacement record. A deterministic two-request regression verifies CSRF on the stale request, rotates the peer request, executes stale destroy, confirms no `Set-Cookie`, and reads the preserved replacement value. Final focused validation passes **16/16 Session/Cart tests**, workspace **114/114**, doctests **5/5**, Cart smoke, native dependency isolation and production digest `faebb0709dccd863`.

Repeat final independent reviewer Job `wc_job_J_MVNzFaVgTH1Gr1` used `gpt-6-astra`, high effort and a read-only sandbox. It made no repository changes and returned:

```json
{
  "overall_status": "pass",
  "issues": [],
  "required_followups": [],
  "optional_notes": [
    "YT-048/M8e and complete M8 review pass; stale destroy preserves the replacement identity while successful explicit destroy still expires the cookie.",
    "Session/Cart tests, retained regression evidence and all frozen production hashes agree; no duplicate browser matrix is required."
  ]
}
```

The reviewer independently confirmed the 114/5 test counts, all ten frozen production files, dependency trees, M8d metrics, 24 untracked whitespace checks, no owned process, one original worktree, empty index and unchanged `master` HEAD. YT-048 and the complete M8 milestone are Done. YT-020 remains separately Blocked.
