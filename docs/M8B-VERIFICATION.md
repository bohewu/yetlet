# M8b Verification — API-only Typed REST Sample

**Date:** 2026-10-09
**Task:** YT-045 / M8b
**Status:** Locally complete. Source, validation and final independent reviewer PASS are frozen.

## Outcome

`yetlet-rest-api` is a native-only Book CRUD application. It uses ordinary handler functions, Axum extractors, the thin Yetlet `App`, `RequestContext` middleware and the explicitly selected `yetlet-request-id` provider.

The sample provides:

- `GET /healthz`;
- list/create at `GET/POST /api/books`;
- read/versioned-update/delete at `GET/PUT/DELETE /api/books/{id}`;
- typed JSON success and registered-handler/policy error envelopes;
- validation, optimistic version conflict, not-found and malformed input behavior;
- 8 KiB JSON limits, JSON media-type enforcement and a direct-HTTP same-origin policy;
- API version, no-store and nosniff headers on registered API routes, plus global request correlation.

It has no browser companion, DOM dependency, WASM target, browser build script or generated asset.

## Routing and header boundary

Handler validation/resource failures and middleware short circuits return the JSON error envelope. Native Axum method fallback on a registered `/api` route remains an empty `405`, but still crosses the route-scoped middleware and therefore carries `x-yetlet-api-version`, `cache-control: no-store`, `x-content-type-options: nosniff` and the global request id. An unmatched path such as `/api/missing` never enters the nested route layer: it remains an empty `404` with only global request correlation. This distinction is deliberate and documented rather than hidden behind another fallback abstraction.

Origin policy accepts either no `Origin` (non-browser client) or exactly one syntactically valid direct-HTTP origin matching exactly one Host authority. Any supplied Host is validated before the no-Origin path is accepted. A bounded parser accepts DNS/IPv4 or bracketed IPv6, optional numeric `u16` ports and case-insensitive host names; absent ports normalize to 80. Duplicate, conflicting, comma-containing, path/query/fragment/userinfo-bearing, nonnumeric or overflowing-port values fail before mutation. Reverse-proxy scheme/host reconstruction remains outside this direct-listener sample.

## Validation

```bash
cargo fmt --all -- --check
cargo clippy --locked -p yetlet-rest-api --all-targets -- -D warnings
cargo test --locked -p yetlet-rest-api --all-targets
cargo build --locked -p yetlet-rest-api
python3 scripts/smoke_rest_api.py
cargo tree --locked -p yetlet-rest-api
python3 scripts/check_samples.py
python3 scripts/test_check_samples.py
python3 scripts/check_docs.py
git diff --check
```

The dependency tree must exclude `wasm-bindgen`, `wasm-bindgen-futures`, `js-sys`, `web-sys`, `yetlet-dom`, `yetlet-dom-keyed` and `yetlet-reactive`.

Observed on the frozen source:

- `yetlet-rest-api`: **7/7 tests PASS**;
- strict targeted Clippy, format and native build: PASS;
- real localhost CRUD/policy smoke: PASS;
- native dependency isolation: PASS;
- two-sample catalog and focused catalog suite: PASS;
- documentation, Python compilation, tracked/untracked whitespace and process cleanup: PASS.

## Limits

The store is process-local and in-memory. Origin comparison assumes direct HTTP and is not a trusted-proxy scheme policy. The sample has no authentication, authorization, persistence, distributed concurrency, pagination cursor, rate limiting or production observability claim.

## Reviewer gate

First reviewer Job `wc_job_sB-Uqwx9aC_dUh-G` used workspace model `gpt-6-astra`, high reasoning effort and a read-only sandbox. It found two material P2 gaps: duplicate/malformed Origin and Host values could pass raw first-value equality, and the documentation overstated JSON/header coverage across native 404/405 fallbacks. Repeat Job `wc_job_jFyhdEsxxnt0QEiB` confirmed the routing documentation but found that `http::Authority` tolerated comma authorities and invalid explicit ports. Both Jobs were read-only; all material findings were accepted.

Three later reviewer executions—`wc_job_oKEe0tI5mVCprYFd`, `wc_job_dXI7IjmPeuHGqiqj` and `wc_job_R2gkjsrLWo2yCFhh`—reached terminal infrastructure failure before model output with `workspace routing discovery failed`; they produced no reviewer result and made no repository changes. A labeled current-session self-review then found and repaired duplicate `Content-Type` ambiguity, but did not satisfy the independent gate.

After Runner recovery, independent read-only reviewer Job `wc_job_rZzWPo-zeHxLtYev` completed with the configured `gpt-6-astra` model and high effort. It found one material P2: the no-Origin branch returned before validating a supplied malformed or duplicate Host, allowing a valid mutation payload to change state. The repair validates any supplied Host before accepting the no-Origin path and adds focused valid/no-Origin plus malformed/duplicate Host cases that verify rejected requests leave the store unchanged. The complete M8b validation above was rerun successfully.

Final repeat independent reviewer Job `wc_job_d911MsTpbwT6fZY3` used `gpt-6-astra`, high effort and a read-only sandbox. It made no repository changes and returned:

```json
{
  "overall_status": "pass",
  "issues": [],
  "required_followups": [],
  "optional_notes": [
    "Host validation now precedes no-Origin acceptance; focused regressions verify typed 403 responses and unchanged store contents."
  ]
}
```

The reviewer confirmed the same `master` worktree, zero staged files, no M8c implementation and no remaining reviewer process. YT-045 is Done; YT-046 is the next sequential item.
