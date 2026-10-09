# M8a Verification — Sample Contract and Catalog

**Date:** 2026-10-09
**Task:** YT-044 / M8a
**Status:** Locally complete. The bounded catalog and documentation contract passes validation and final independent review.

## Outcome

M8a establishes a documentation-first and machine-checkable contract for user-facing samples without changing Yetlet runtime behavior or promoting a new framework abstraction.

The gate adds:

- `examples/samples.toml`, a versioned catalog of runnable application samples;
- `scripts/check_samples.py`, a standard-library-only validator;
- `docs/SAMPLES.md`, structure, isolation and evidence rules;
- a sample-local README for the existing integrated Todo application;
- M8 backlog and roadmap gates for REST, Session cart, DX audit and final regression.

The catalog intentionally excludes internal browser entry crates, shared UI packages and research fixtures. Those remain workspace members but are not presented as standalone application samples.

The first independent reviewer demonstrated that shape-only validation accepted false command, route, browser/dependency and escaped evidence declarations. The repaired checker now requires repository-contained paths, exact README commands, sample-local route source markers, explicit browser companion/build mappings and direct optional dependencies in the primary or companion manifests. Boolean schema versions and malformed field types fail cleanly.

## Validation

```bash
python3 scripts/check_samples.py
python3 scripts/check_docs.py
python3 -m py_compile scripts/check_samples.py scripts/check_docs.py
cargo metadata --locked --no-deps --format-version 1
cargo test --locked -p yetlet-todo-server --all-targets
git diff --check
python3 scripts/test_check_samples.py
```

No browser/server process is required for this documentation and catalog gate. M8b is the next bounded implementation item.

## Reviewer gate

First reviewer Job `wc_job_LnWNFOae0ITo3c4Q` used the workspace reviewer model `gpt-6-astra`, high effort and a read-only sandbox. It failed with three material P2 findings covering shape-only catalog claims, repository path escape and a non-sequential M8 dependency edge. Repeat Job `wc_job_svcC4YFs7O-H647o` found API-kind isolation, route-prefix and malformed-package gaps; Job `wc_job__5OulkwzrJH87J3Q` then found a route-suffix gap. All material findings were accepted and added to the focused suite before M8b began.

Final read-only reviewer Job `wc_job_vQ6pGDE0slh9gV8c` used `gpt-6-astra` with high effort and returned:

```json
{
  "overall_status": "pass",
  "issues": [],
  "required_followups": [],
  "optional_notes": []
}
```

The final reviewer independently exercised 32 adjacent counterexamples, confirmed the exact 11-file scope and made no edit, stage, commit or push.
