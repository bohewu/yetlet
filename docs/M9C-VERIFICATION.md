# M9c Verification — Stateless SSR Forms Validation

**Date:** 2026-10-09
**Task:** YT-052 / M9c
**Status:** Done. Reviewer P2 repaired, complete validation and repeat read-only reviewer `wc_job_lGtk_ZIoAvK4F6PW` PASS.

## Intent

A runnable `yetlet-forms-validation` native-only SSR application exercises ordinary handler-first `App` routes, native Axum typed form extraction and `Page` rendering with field-level errors, input escaping, label associations, bounded body size and no browser runtime.

Routes: `GET /form`, `POST /form/submit`, `GET /form/success`.

Validation failures return SSR 422 with escaped input values and explicit `aria-describedby`/`aria-invalid` plus focus on the first invalid field. A valid request redirects 303 to a generic GET success page. All state is **ephemeral per request**; no Session, DB, auth, workflow or persistent mutation is installed. The sample **does not claim CSRF protection**: adding credentials or a durable mutation requires explicit security design and a new reviewer gate. Email format validation is illustrative, not a fully compliant email validator.

## Validation

```bash
cargo fmt --all -- --check
cargo clippy --locked -p yetlet-forms-validation --all-targets -- -D warnings
cargo test --locked -p yetlet-forms-validation --all-targets
cargo build --locked -p yetlet-forms-validation
python3 scripts/smoke_forms_validation.py
cargo tree --locked -p yetlet-forms-validation
python3 scripts/check_samples.py
python3 scripts/test_check_samples.py
python3 scripts/check_docs.py
git diff --check
```

The independent reviewer must verify native SSR without JS/WASM, correctness/escaping of both attribute and text content, safe response headers, malformed/oversized/unknown input behavior and 303 PRG with no state. Core/server adapter, Todo, REST, Cart and browser packages must not inherit dependencies introduced by this app.

## Observed results

- All **6/6** Forms Validation tests PASS: SSR form structure/strict CSP/no cookie, 422 invalid field/error associations and first-error focus, escaped reflected hostile values, stateless 303 PRG, 415 unsupported media, native 422 unknown extra field, 413 oversized input and 404/405 fallbacks.
- Native `cargo fmt` and targeted Clippy `-D warnings` PASS, debug/release binary builds PASS.
- Full workspace strict Clippy and **130/130 tests** PASS; doctests **5/5** PASS.
- Real localhost `scripts/smoke_forms_validation.py` exercises live SSR, invalid field output and escaped markup, 303 success and absence of state/cookies; owned server is terminated.
- Sample catalog **6** entries PASS; focused checker **4/4** PASS; English docs/internal links/backlog dependency DAG **43 documents / 59 tasks** PASS.
- Cargo `tree --edges normal` confirms no `yetlet-session`, `yetlet-request-id`, browser/WASM/reactive crates, SQLite/SQLx/ORM, or other optional dependencies in this sample. Application-only serde is the existing mature `Form<T>` extractor dependency; no core or server-adapter source changed.
- Binaries: debug **27,859,472 raw bytes**, release **2,022,248 raw bytes**; local builds, not browser transfer or memory metrics.
- Git remains on the one original `master` worktree at `742e0d4909cc9b132d81b78e12bbb66d5bf60f57`, empty index and no commit/push. All untracked files were whitespace checked.
- The first targeted test assertion expected 400 for unknown fields, while Axum's native `Form<T>` rejection returns **422**; the assertion and README now truthfully retain native 422 rather than adding a Yetlet translation layer. The first HTTP smoke check incorrectly expected a name error for a name valid under the 2–40-character contract; coverage was corrected to verify both hostile-value escaping and a separate invalid-name/error-focus case. Subsequent focused and full validation passed.

No authentication, storage, persistent side effect, CSRF mechanism, browser renderer or database/ORM code was introduced.

## First independent reviewer finding and repair

The independent high/read-only role-directed reviewer `wc_job_7D2rtHdGEfwW1Aet` returned `overall_status=fail` with one concrete P2 accessibility finding: Yetlet's existing SSR `render_document` gives the root document `lang="zh-Hant"`, while the Forms sample's headings, labels, validation messages and success text are English. Unoverridden English content therefore inherited the wrong language for assistive technology. The reviewer requested a **sample-only** `lang="en"` declaration on the two sample-owned `<main>` elements, not a change to the renderer's global locale policy.

The two forms/success `<main lang="en">` declarations now cover GET form, invalid-form SSR and success SSR; native Rust focused tests and real localhost smoke assert the override on all three response paths. This has no impact on other app languages, core APIs, HTTP semantics, persistence or Session. The original P2 was not ignored; the repaired source passed 6 focused tests, live localhost smoke and final full-workspace validation (130 tests, 5 doctests).

## Reviewer gate

Repeat independent role-directed review Job `wc_job_lGtk_ZIoAvK4F6PW` ran to terminal exit 0, using an explicit `gpt-6-astra` model selector, high effort request and read-only sandbox, not `codex exec review`. The Job returned:

```json
{
  "overall_status": "pass",
  "issues": [],
  "required_followups": [],
  "optional_notes": [
    "The language override covers all three SSR paths; existing targeted tests passed 6/6.",
    "Native reviewer model/effort trace was unavailable; the requested selectors are not independently verified."
  ]
}
```

This reviewer PASS is an ordinary independent source/validation gate, **not** formal assurance, an authenticated production security review or proof of the effective model/effort beyond the explicit invocation arguments. No code edits, staging, commit, push or worktree changes were performed by the reviewer. YT-052 is locally **Done**; YT-053 File Server is the next eligible bounded slice.
