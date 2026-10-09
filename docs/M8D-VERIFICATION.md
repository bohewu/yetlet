# M8d Verification — Sample Developer Experience Audit

**Date:** 2026-10-09
**Task:** YT-047 / M8d
**Status:** Locally complete. Reproducible audit, no-runtime-API-change decision and independent reviewer PASS are frozen.

## Outcome

M8d compares the three runnable applications after M8a–M8c instead of adding an abstraction in advance. The result is a **no-runtime-API-change evidence freeze**:

- the existing handler-first `App`, native Axum extractors/results, `Page`, middleware scopes and explicit optional crates cover the three application shapes;
- no Controller hierarchy, dependency-injection container, procedural macro, generic forms runtime, shared client circuit or new application layout convention is justified;
- repeated code is either small native process bootstrap, intentionally application-specific policy, or already owned by an optional crate;
- no ADR is added because M8d makes no new architecture selection.

`scripts/measure_sample_dx.py` is a standard-library-only evidence tool. It reads the checked sample catalog and workspace manifests, follows application-owned dependencies under `examples/`, and measures non-blank physical Rust lines, test lines, async functions, routes and direct non-dev dependencies. It does not claim that line count is complexity, quality or performance.

## Reproducible snapshot

Run:

```bash
python3 scripts/measure_sample_dx.py --out target/m8d-sample-dx.json
```

| Sample | Application packages | Rust files | Production non-blank lines | Test non-blank lines | Production async functions | Catalog routes | Direct dependency names | Browser runtime |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| Todo | 4 | 8 | 1,102 | 536 | 10 | 4 | 16 | Yes |
| REST API | 1 | 2 | 551 | 417 | 8 | 3 | 6 | No |
| Session cart | 1 | 2 | 424 | 200 | 7 | 7 | 6 | No |

The Todo scope honestly includes its transitive application-owned `yetlet-todo-ui` package in addition to the catalog primary and two browser companions. REST and Cart remain one-package applications. Their six direct dependency names reflect the capabilities they actually select; neither pulls a browser runtime or generated assets.

Optional capability implementation is counted separately from application authoring:

| Capability | Production non-blank lines | Test non-blank lines | Selected by |
|---|---:|---:|---|
| `yetlet-request-id` | 124 | 149 | Todo, REST |
| `yetlet-reactive` | 474 | 399 | Todo Browser |
| `yetlet-dom-keyed` | 318 | 25 | Todo Browser |
| `yetlet-session` | 559 | 283 | Session Cart |

These figures demonstrate isolation, not bundle cost. M8e retains the existing artifact/dependency checks for actual Browser output.

## Boilerplate audit

### Already solved by current APIs

All applications use ordinary async functions. `App::new`, `get`, `post`, `route`, `group`, `configure`, `route_layer`, `with_state` and native Axum method routers cover setup without parallel routing or a framework-specific handler trait. Typed extractors and `IntoResponse` remain native. SSR uses `Page`/`View`; optional capabilities arrive through explicit crates and typed extensions.

No sample requires a Controller base class, service locator, reflection/scanning, generated registration, macro DSL or prescribed folder layout.

### Repeated but not promotion-worthy

1. **Listener bootstrap:** the three `main.rs` files are 11–14 lines and intentionally retain native `TcpListener`/`axum::serve`. Environment names, default ports, actual-address reporting and Cart's loopback security gate differ. A Yetlet `run()` helper would hide shutdown, TLS and listener policy before those contracts exist.
2. **JSON response/error helpers:** Todo and REST both serialize JSON errors, but their envelopes, status mapping, state transitions and header boundaries differ. Two samples do not establish a stable public error framework.
3. **Media-type and direct-origin checks:** Todo and REST share small policy names, while M8b evidence shows materially different strictness and deployment assumptions. Security policy must not be generalized from inconsistent examples.
4. **Request correlation:** Todo and REST already reuse `yetlet-request-id`; no new core field or module system is needed.
5. **Forms/CSRF helpers:** hidden inputs and POST forms occur only in Cart. A generic forms runtime would outgrow the evidence and blur Session/application ownership.
6. **Browser build and DOM ownership:** only Todo pays for WASM, keyed DOM and local reactivity. Moving these into a default application surface would violate the measured API-only/SSR isolation.

### Candidate only after more evidence

- a shared strict HTTP-origin policy after another deployment mode defines proxy/scheme authority;
- a typed JSON error helper after at least one more API converges on the same wire envelope and fallback boundary;
- reusable SSR form descriptors after another application repeats validation, accessibility and CSRF composition;
- listener/shutdown helpers after TLS and graceful-shutdown requirements are explicit.

These are not M8d deliverables or current public API commitments.

## Decision

No runtime/source API is promoted or removed in M8d. The sample catalog, local READMEs and retained evidence are the appropriate developer-experience improvement at this stage. The higher-level authoring proposals deferred by M7 remain deferred:

- general VDOM or signals framework;
- `ui!` procedural macro;
- generic forms runtime;
- Controller/MVC requirement or DI container;
- full resumability or WebGPU integration.

This decision is based on repeated-shape evidence, not benchmark rank or raw line counts.

## Validation

```bash
python3 scripts/measure_sample_dx.py --out target/m8d-sample-dx.json
python3 -m py_compile scripts/measure_sample_dx.py
cargo metadata --locked --no-deps --format-version 1
cargo test --locked \
  -p yetlet-todo-server \
  -p yetlet-rest-api \
  -p yetlet-cart-server \
  -p yetlet-session \
  --all-targets
python3 scripts/check_samples.py
python3 scripts/test_check_samples.py
python3 scripts/check_docs.py
git diff --check
```

The retained JSON must reproduce the table above and list the exact package/dependency sets used to derive it. M8d changes no Rust framework or application behavior.

Frozen validation reproduces all three rows exactly, passes **32/32 sample/provider tests**, Cargo metadata, the three-sample catalog, **4/4 focused catalog checks**, **37 documents / 48 backlog items**, Python compilation, tracked/untracked whitespace and an empty index. Twenty-three untracked files have no whitespace diagnostics.

## Reviewer gate

Independent read-only reviewer Job `wc_job_lZSDSBgMKWu1X5X7` used `gpt-6-astra`, high effort and made no repository changes. It returned:

```json
{
  "overall_status": "pass",
  "issues": [],
  "required_followups": [],
  "optional_notes": [
    "Source, retained JSON, documented metrics and dependency sets agree; the no-runtime-API-change decision is supported."
  ]
}
```

The reviewer confirmed the unchanged `master` HEAD, empty index, cleanup and absence of M8e implementation. YT-047 is Done; YT-048 is the next sequential item.
