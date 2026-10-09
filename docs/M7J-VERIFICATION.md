# M7j Verification — Final M7 Architecture and Evidence Freeze

**Date:** 2026-10-09
**Task:** YT-043 / M7j
**Status:** Done. Source, reviewer-found lifetime repair, replacement validation, architecture decision and repeat read-only reviewer PASS are frozen.

## Final decision

M7 closes with the bounded architecture recorded in [ADR 0007](adr/0007-bounded-keyed-dom-and-local-graph.md):

- renderer-neutral identity/key/slot safety contracts live in `yetlet-core`;
- the synchronous strong-owner/weak-handle graph is optional `yetlet-reactive`;
- keyed child/state/listener ownership is optional `yetlet-dom-keyed`;
- the production Todo local list opts into both primitives;
- remote Todo results and Search results retain bounded subtree replacement;
- browser applications are built separately to preserve asset isolation.

This is a deliberate per-surface renderer choice. It is not a benchmark-winner declaration or selection of a general VDOM, static-template DSL, signals/forms framework, resumability protocol or WebGPU renderer.

## M7 delivery map

| Item | Outcome | Reviewer Job |
|---|---|---|
| M7a / YT-025 | Four-strategy isolated benchmark and identity evidence | Retained earlier evidence |
| M7b / YT-035 | Fixed keyed/static-slot authoring and SSR identity prototype | Retained earlier evidence |
| M7c / YT-036 | Explicit local graph ownership, batching and failure boundaries | Retained earlier evidence |
| M7d / YT-037 | Chrome/Firefox controlled-input lifecycle and cleanup | `wc_job__eL686DqLofTLyVR` |
| M7e / YT-038 | Complete no-decision regression/evidence freeze | `wc_job_9hT23HL0mwaTto4S` |
| M7f / YT-039 | Renderer-neutral binding contracts promoted | `wc_job_DUHpN6tii7xSFCnD` |
| M7g / YT-040 | Optional synchronous local graph extracted | `wc_job_GnG93jQMHK4dTQv7` |
| M7h / YT-041 | Optional keyed DOM/listener ownership extracted | `wc_job_IpxKPD3wwMQto2FU` |
| M7i / YT-042 | Production Todo local list adoption | `wc_job_h8nwZIFRImFvMYhG` |
| M7j / YT-043 | Final full regression, decision, lifetime repair and evidence freeze | First final review `wc_job_orP7QpVjLsh07HWI` found one P2; repeat reviewer `wc_job_XZzdXEOIx6owj1o5` PASS |

Every M7f–M7i continuation reviewer used the workspace reviewer role, `gpt-6-astra`, high reasoning effort and a read-only sandbox. The first M7j final reviewer used the same configuration and found the lifetime P2 described below. The Runner exposed no native coding-agent provider, so these were ordinary role-directed local `codex exec <reviewer prompt>` Jobs—not `codex exec review` and not native `agent_type` dispatch. None edited, staged, committed or pushed.

## Complete validation Job

The initial pre-review frozen-source sequence was Job `wc_job_KoFrkmsaBZPbjngI` (terminal exit 0). After the first final reviewer found the lifetime race, replacement execution `wc_job_FykOaiYPhzHBeaiJ` reran every command below and completed all product, documentation, toolchain, browser, benchmark, artifact, dependency and whitespace gates. Its final cleanup helper then matched the parent shell command containing its own search strings and exited 1 after writing `target/m7j-final-summary.json`. A bounded post-terminal recovery excluded the ancestor chain and passed exact process cleanup, HEAD/branch/worktree, no-staged-files and `git diff --check` verification. No product gate failed or was retried selectively.

```bash
cargo fmt --all -- --check
python3 scripts/check_docs.py
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

python3 scripts/build_browser.py --release
cargo build --release --locked -p yetlet-todo-server
python3 scripts/smoke_browser.py
python3 scripts/smoke_firefox.py --profile release
python3 scripts/measure_baseline.py --profile release --samples 30 \
  --browser-runs 5 --out target/m7j-baseline-release.json

python3 scripts/measure_render_strategies.py \
  --capture-production-before target/m7j-production-before.json
python3 scripts/build_render_bench.py --release
python3 scripts/measure_render_strategies.py --profile release \
  --rows 200 --updates 500 --rounds 10 --seed 7 \
  --production-before target/m7j-production-before.json \
  --out target/m7j-render-strategies.json

python3 scripts/build_keyed_bindings.py --release
python3 scripts/smoke_keyed_bindings.py --profile release
python3 scripts/smoke_keyed_bindings_firefox.py --profile release
```

## Validation results

| Gate | Result |
|---|---|
| Workspace format | PASS |
| Workspace strict Clippy | PASS |
| Workspace unit/integration tests | **91/91 PASS** |
| Workspace doctests | **5/5 PASS** |
| Six-package wasm32 check | PASS |
| Production Chrome 155 debug regression | PASS |
| Production Firefox 157 debug regression | PASS |
| Production Chrome 155 release-asset regression | PASS |
| Production Firefox 157 release regression | PASS |
| Queued click→dispose and delayed HTTP completion races | PASS in Chrome/Firefox debug and release |
| M7a full 200 × 500 × 10 benchmark | PASS |
| Isolated keyed authoring/graph/lifecycle Chrome release | PASS |
| Isolated keyed authoring/graph/lifecycle Firefox release | PASS |
| Todo/Search dependency boundary | PASS |
| Tracked and all untracked whitespace checks | PASS |
| Browser/server/driver/benchmark cleanup | PASS |
| Staged files | None |

Production regressions include native SSR/escaping, strict CSP, App/RequestContext/middleware, typed local and HTTP actions, Todo API errors, Search lazy activation/fail-closed state, keyed local identity, queued-task retirement, delayed HTTP completion suppression, disposal, detached-listener behavior and duplicate-free remount. No WebSocket or raw-event transport was observed.

## Final production artifacts

The repaired M7i/M7j production digest is:

```text
faebb0709dccd863
```

Relative to the M7e pre-adoption digest `3f3c5f074478c525`:

| Browser runtime asset | M7e raw / gzip | M7j raw / gzip | Delta raw / gzip |
|---|---:|---:|---:|
| `app.js` | 74 / 84 | 151 / 138 | +77 / +54 |
| `yetlet_browser.js` | 18,199 / 4,220 | 22,299 / 4,792 | +4,100 / +572 |
| `yetlet_browser_bg.wasm` | 216,006 / 77,003 | 294,555 / 96,419 | +78,549 / +19,416 |
| **Browser runtime total** | **234,279 / 81,307** | **317,005 / 101,349** | **+82,726 / +20,042** |

Search remains byte-identical to M7e after separate package builds:

| Search asset | Raw / gzip | SHA-256 |
|---|---:|---|
| `yetlet_search.js` | 14,699 / 3,675 | `1aca190b9e125e3e09f79c66be7c249f7be0fb794273575f547e6912f91e4f9b` |
| `yetlet_search_bg.wasm` | 179,716 / 70,806 | `ff46e70c132238db34765e9718692b8246359d2516e344258f464b5d6fc85db9` |

The Todo Browser dependency graph includes `yetlet-reactive` and `yetlet-dom-keyed`; Search Island excludes both.

## Repeated release baseline

Environment: Linux x86_64, rustc 1.99.0, Python 3.14.4 and Chrome 155.0.8059.39. Browser cache was disabled. Measurements are local loopback observations, not Web Vitals, transfer measurements or an SLA.

| Median | M5 historical | M6 after | M7j |
|---|---:|---:|---:|
| SSR request to first response byte | 0.034 ms | 0.033 ms | **0.037 ms** |
| Chrome `/app` navigation to ready | 48.5 ms | 49.0 ms | **49.0 ms** |
| Search trigger to ready | 3.7 ms | 3.8 ms | **3.5 ms** |

M7j sample details:

- SSR: 30 samples, median 0.037 ms, p95 0.073 ms, range 0.015–0.076 ms;
- Chrome app ready: 5 samples, median 49.0 ms, p95 54.8 ms, range 48.1–54.8 ms;
- Search ready: 5 samples, median 3.5 ms, p95 5.9 ms, range 3.3–5.9 ms.

These small sequential samples do not establish statistical equivalence, superiority or a performance budget. They disclose the observed bundle/runtime tradeoff while showing no material local regression in this environment.

## Render-strategy evidence retained

The canonical M7a report remains the identified `2026-10-09T02:02:22+00:00` record. The repaired M7j source reran the complete workload at `2026-10-09T07:08:21+00:00` without replacing that canonical evidence.

| Strategy | Canonical median | M7j median |
|---|---:|---:|
| Subtree replacement | 213.500 ms | 219.000 ms |
| Positional View patch | 41.700 ms | 42.250 ms |
| Keyed direct DOM | 2.000 ms | 2.050 ms |
| Static template slots | 1.900 ms | 1.800 ms |

All four strategies again produced equivalent final state. The benchmark digest remains `71182d03b13e4d13`, and production assets remained unchanged throughout the isolated run. The result supports the bounded keyed choice but does not make either direct keyed DOM or static slots a general production winner.

## Isolated authoring/lifecycle artifact

The M7h-era fixed authoring fixture remains separately validated:

- digest: `6754025153168694`;
- WASM: 272,728 bytes;
- local-gzip WASM: 75,818 bytes;
- Chrome 155 release lifecycle suite: PASS;
- Firefox 157 release lifecycle suite: PASS.

This fixture continues to test current text-input/focus/conditional-slot behavior, but those authoring APIs are not promoted by ADR 0007.

## First final reviewer finding and repair

Role-directed reviewer Job `wc_job_orP7QpVjLsh07HWI` reached terminal exit code 0 but returned `overall_status = fail` with one P2: `spawn_local` always starts on a later microtask, so click→dispose in one JavaScript task could leave local/remote work queued, and a remote response already awaiting fetch could later update retired DOM.

The repair adds one reference-counted runtime-lifetime token. `BrowserRuntime::drop` invalidates it before graph/listener fields are destroyed. Each queued task checks the token before starting; the HTTP path checks it again after `await` and before any render/status mutation. Remount owns a distinct fresh token.

Focused Chrome 155 and Firefox 157 debug/release regressions now prove:

- local and remote clicks immediately followed by disposal start no graph transition or HTTP request;
- an invalid HTTP response deliberately held until after disposal/remount cannot update the retained old nodes or the new route DOM;
- ordinary disposal, detached-listener and duplicate-free remount behavior remains passing.

## Final reviewer gate

Runner status still reported `coding_agent_providers: []`, so native ACP reviewer dispatch was unavailable. WebCodex launched ordinary role-directed Job `wc_job_XZzdXEOIx6owj1o5` with the canonical reviewer definition, workspace model `gpt-6-astra`, high reasoning effort, approval policy `Never` and a read-only sandbox. It was not `codex exec review` and not native `agent_type` dispatch.

The Job reached terminal exit code 0 and returned:

```json
{
  "overall_status": "pass",
  "issues": [],
  "required_followups": [],
  "optional_notes": []
}
```

The reviewer inspected the repaired lifetime implementation, focused Chrome/Firefox race coverage, current M7 diff, retained test/browser/benchmark artifacts, exact asset hashes, dependency isolation, documentation and Git hygiene. It independently recomputed Search byte identity and the isolated keyed digest. It made no edit, stage, commit or push.

`YT-043 / M7j` and the M7 milestone are locally Done at experimental prototype scope.

## Limits and release boundary

M7 is still experimental prototype work. It does not close public release ticket YT-020 or provide:

- a stable API or support/MSRV/browser policy;
- authenticated deployment, credentialed CSRF controls or independent security qualification;
- generic nested component reconciliation or transactional DOM rollback;
- async effects, workers or dependency-tracked signals;
- generic forms, broad IME/mobile/WebKit/accessibility qualification;
- a template macro, full hydration/resumability or WebGPU renderer;
- production network/throughput/memory/heap guarantees or cross-framework results.

No work was staged, committed or pushed. HEAD and the single existing worktree remain unchanged.
