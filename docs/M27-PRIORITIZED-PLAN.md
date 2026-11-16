# M27-B — Prioritized Developer Experience Work Plan

**Date:** 2026-11-16
**Status:** Draft for independent review; no remediation implementation authorized
**Baseline:** `master@89e9cc47eac20786b2d2c13a2c02e9bdc06b18dc`
**Input:** `docs/M27-DX-GAP-REPORT.md`

## 1. Decision

M27-B selects a small documentation-first remediation slice. The audit did not demonstrate a need for new SDK Core APIs, CLI commands, dependency layers or process infrastructure. The highest-value work is to make the existing CLI contract easier to use correctly.

The selected scope is limited to two generated/maintained documentation surfaces:

1. Correct the first-use CLI guide's stale statement about Cargo.lock generation.
2. Explain the generated `--service` scaffold's actual operational interface.

No product source or runtime behavior changes are selected.

## 2. Ranked candidates

| Rank | Finding | Priority | Owner layer | Change | Value | Cost / risk |
|---|---|---|---|---|---|---|
| 1 | DX-01: stale `new` / Cargo.lock sequence | P2 | Maintained CLI documentation | Correct the first-use sequence in `docs/CLI-GUIDE.md` to match CLI 0.4.0, `FIRST-APP.md` and `CLI-CONTRACT.md` | Prevents unnecessary lock regeneration and protects exact SDK/dependency provenance | Low; factual documentation correction only |
| 2 | DX-02: Service README omits health/bind contract | P2 | Conditional generated README fragment | Add `tools/yetlet-cli/templates/service/SELECTED-README.md` and append it from `scaffold::selected_files_version` only for `--service`; include routes, bind defaults/allowed values and shutdown readiness behavior | Makes the selected Service capability discoverable without source inspection | Low; generated documentation plus narrowly scoped README composition logic |

### Candidate 1 — DX-01 acceptance

- State that `yetlet new` resolves the selected SDK and creates `Cargo.lock` plus `.yetlet/sdk-resolution.json` before publishing the app.
- State that creation does not install tools/packages, build the app or start a listener.
- Do not change `templates/SELECTED-README.md`: although its raw source contains legacy no-lock text, `creation.rs` normalizes it before publication and retained generated READMEs are correct.
- Use the correct sequence: create app (which publishes its lock and receipt); `cargo fetch --locked`; prepare selected frontend dependencies/assets if applicable; then `yetlet build` / `yetlet run`.
- Correct only the maintained CLI guide; do not edit `templates/SELECTED-README.md` because its lock wording is normalized before generated output is published, and do not edit the unused legacy `templates/README.md`.
- Preserve the explicit offline metadata/Cargo-cache requirements and the rule that later build/run/watch use the frozen app lock.
- Do not change the CLI contract, resolver, generated lock, package identity or Reader Edition.

### Candidate 2 — DX-02 acceptance

- Document `GET /health/live` and `GET /health/ready`.
- For `--service` without `--settings` / `--vite`, document `YETLET_HOST` default `127.0.0.1`, allowed values `127.0.0.1` or explicitly selected `0.0.0.0`, and `YETLET_PORT` default `3000`, valid range 1–65535.
- For `--service --settings` or `--service --vite`, document `server.host` / `server.port` defaults and `APP_SERVER__HOST` / `APP_SERVER__PORT` overrides; service host validation allows only `127.0.0.1` or `0.0.0.0`, and port range is 1–65535. The settings-enabled path also requires operator-supplied `APP_SERVICE__TOKEN`.
- Explain that readiness starts true and is set false on the handled shutdown signal before graceful server shutdown.
- Do not describe these routes as dependency checks, add authentication/metrics, or promise HA, zero downtime or production readiness.
- Put the Service guidance behind a clearly labeled “Service profile (`--service`)” heading in the dedicated Service README fragment; append it only for Service selections and do not imply that plain Native scaffolds expose these routes.
- Explicitly distinguish the plain Service environment variables from the Settings/Vite APP-prefixed settings path so users do not set variables that their selected scaffold does not read.
- Preserve existing custom listener and Axum shutdown implementation.

## 3. Explicitly deferred

| Area | Decision | Reason |
|---|---|---|
| OpenSSL / signing dependency split | Defer | Current prerequisite is accurately documented; no consumer-impact evidence justifies a package/feature split |
| Doctor / Inspect merge or schema expansion | Defer | Separate environment and project-plan contracts are intentional and clearly documented |
| New CLI command, flag or settings schema | Defer | No current journey requires another command or configuration language |
| SDK Core startup/DI/Controller/ORM APIs | Defer | M26 and YT-047 evidence supports current separation; no repeated consumer need demonstrated |
| Vite supervisor / package auto-install | Defer | Existing ownership and explicit-tool boundaries are coherent |
| Browser template cache/online behavior | Defer | Offline cache miss fails clearly and closed; no defect demonstrated |
| WebGPU / 3D / HA | Out of M27 scope | Separate experimental or operational tracks; no relation to the confirmed DX findings |

## 4. Execution and verification gates

1. Independent plan Reviewer R4 PASS; YT-260 and YT-261 registered in the formal Roadmap/Backlog.
2. Owner authorized the bounded documentation change. YT-260 corrects the CLI first-use sequence. YT-261 adds a Service-only README fragment and narrowly scoped selection logic; generated runtime code and listener behavior are unchanged.
3. Verified exact generated README output for plain Native, Service-only, Service+Settings and Service+Vite with the existing scaffold integration test and fresh CLI 0.4.0 registry scaffolds. The Service fragment is absent from plain Native and documents both bind modes. `cargo fmt --all -- --check`, the complete `selected_scaffold` integration target (2 passed, 2 intentionally ignored), `cargo xtask check docs` (29 documents / 258 Backlog items) and `git diff --check` passed.
4. Run the repository's existing strict Commit Guard only if a commit is separately authorized. No new validator or validation infrastructure.
5. Obtain an independent final Reviewer PASS on the exact changed documentation and generated output evidence.
6. Commit/PR/merge/Pages actions remain separate authorization gates. Do not publish packages.

## 5. Non-goals

- No SDK runtime behavior, dependencies, feature flags, public APIs, argv grammar, JSON schema or lock behavior. The only CLI source change permitted is conditional README-fragment composition for generated Service documentation.
- No network fallback for offline workflows, npm install, Browser build, engine installation or system permission changes.
- No broad test reruns, cache cleanup, unrelated worktree cleanup, branch deletion, release, signing, registry publication, Pages deployment, push or merge.
- No implementation of OpenSSL optionalization or other deferred architecture proposals.

## 6. Current status

M27-A identified exactly two bounded P2 documentation findings. Independent plan Reviewer R4 and independent implementation Reviewer `wc_job_ab4Zl3f_I3Q3_Fu4` both returned PASS with empty issues and required_followups. YT-260 and YT-261 are Done at the accepted documentation scope. No runtime behavior or public contract changed.
