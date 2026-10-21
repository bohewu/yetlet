# YT-102 — Frontend dotenv boundary correction

Date: 2026-10-18. Base HEAD: `f4fa23b0b68e9a346347f0a7611b628e19ac73f8`.
Status: F-01 / AC-14 correction complete; independent Reviewer PASS.

## Scope

This followup addresses F-01 / AC-14 from the
[current CLI contract audit](YT-102-CLI-CONTRACT-AUDIT-20261018.md).
The existing [CLI contract](CLI-CONTRACT.md#4-project-paths-and-frontend-layout)
requires selected frontend dotenv inputs to remain within the configured
frontend root. A linked `.env` previously allowed Vite to read an outside file.

The correction preserves Vite's ordinary frontend-local dotenv support and the
existing settings mechanism. No CLI option, settings key, dotenv parser or
dependency is added. Legacy templates with `envDir: false` keep that behavior.
The scope is the generated Vite integration; package scripts and user code are
not a filesystem sandbox.

## Changes

- [app.rs](../tools/yetlet-cli/src/app.rs) invokes one metadata-only preflight
  after selecting the protected frontend directory and before any child starts.
  This also protects applications created before the template correction.
- [platform/fs/mod.rs](../tools/yetlet-cli/src/platform/fs/mod.rs) rejects
  top-level `.env`/`.env.*` links and nonregular files without reading their
  contents. It verifies the app identity before and after inspection. Windows
  uses case-insensitive names and rejects the native reparse-point attribute.
- [scaffold.rs](../tools/yetlet-cli/src/scaffold.rs) adds the selected-only
  [Vite config fragment](../tools/yetlet-cli/templates/vite/selected-dotenv.mjs)
  to plain and Tailwind templates. Its pre-config hook checks the four active
  dotenv inputs before Vite's loader on each config resolution, including direct
  npm use and development config inheritance.
- [frontend_process.rs](../tools/yetlet-cli/tests/frontend_process.rs) proves
  `run`/`watch`/`build` refuse outside links before child effects, at default and
  custom roots, while a regular local file permits the selected build.
  [selected_scaffold.rs](../tools/yetlet-cli/tests/selected_scaffold.rs) exercises
  real Vite config resolution and preserves legacy scaffold checks.

## Verification and review

Local evidence is retained under `artifacts/cli-dotenv-boundary-20261018/`.
Each named check has a JSON command/exit receipt and a complete log.

| Check | Result |
| --- | --- |
| `cargo test --offline --locked --manifest-path tools/yetlet-cli/Cargo.toml --all-targets` | 111 passed, 0 failed; five explicitly ignored by this command |
| Explicit ignored `selected_vite_dotenv_is_checked_before_load_on_each_build_and_serve_resolution` test, using the retained installed Vite 8.3.4 | 1 passed; plain/Tailwind and build/serve config resolution |
| Focused existing-app dotenv preflight test | 1 passed; refusal has an empty child trace, ordinary local-file build succeeds |
| `cargo clippy --offline --locked --manifest-path tools/yetlet-cli/Cargo.toml --all-targets -- -D warnings` | PASS |
| `cargo fmt --manifest-path tools/yetlet-cli/Cargo.toml --check` | PASS |
| `cargo check --offline --locked --manifest-path tools/yetlet-cli/Cargo.toml --target x86_64-pc-windows-msvc --tests` | PASS; compilation only |

The real Vite test checks all four active dotenv names, ordinary local-file
precedence and inherited public environment precedence, no ancestor discovery,
outside links rejected with zero dotenv reads, dangling links and directory
refusal, and resolution after replacing an earlier accepted regular file. It
uses native config loading without installing dependencies, starting a Vite
server or writing retained dependency caches. Temporary app fixtures are cleaned
up. This is direct coverage of the loader boundary, not browser/HMR qualification.

The first real-Vite fixture attempt failed before Node execution because its
scaffold argument order omitted the frontend. That harness-only failure and the
single in-place correction/rerun are retained separately. It was not a product
failure or a new validation run.

Independent read-only review found no material issues or required followups. Its
native trace matched the registered reviewer role, configured model and effective
effort. `review.json` retains the bounded result. This acceptance covers the
correction above, not the full CLI or every platform/runtime combination.

Native Windows generator execution remains the separate qualification gap from
the audit. This followup does not qualify browser HMR, registry delivery or
production deployment, and does not close the broader research-only YT-102 item.

## Checkout preservation

The work uses the current checkout. Pre-existing YT-103/planning changes and the
historical audit are preserved. No commit, push or remote mutation is included.
Final documentation/whitespace command receipts and file-hash preservation
results are retained as `final-docs.json`, `final-whitespace.json` and
`preservation.json` in the evidence directory. Product source hashes are retained
in `source-tested.json`; HEAD remains the base commit above.

The next CLI qualification action is native Windows execution of the current
generator extension. YT-092 remains the independently dependency-ready application
item; no new backlog item is started or marked Done by this correction.
