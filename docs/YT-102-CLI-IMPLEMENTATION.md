# YT-102 — Local CLI implementation

Date: 2026-10-18 (Asia/Taipei).

## Authority and scope

The owner explicitly authorized implementation of the independently reviewed
[CLI contract](CLI-CONTRACT.md), followed by independent Reviewer review. The
latest instruction makes `new --vite` imply the runtime settings module. Plain
new retains `--settings` opt-in; every new app declares public CLI settings.
This implementation authority is separate from YT-102's research-only backlog
registration. The [prior design record](YT-102-CLI-CONTRACT-VERIFICATION.md)
retains its original evidence and failures.

Checkout: `$REPO`, branch `feat/yt090-configuration`, HEAD
`a883469862c5be3d88b7fe22262e10be8b53b92a` plus the dirty working tree. The task
started with uncommitted YT-101 and M19/M20 work; HEAD alone is not source identity.
No commit, push, publication, deployment or host-permission change was performed.

## Implemented behavior

- Revised creation/execution options, literal separate/equal values, default
  cwd manifest, scoped exclusive help/version and safe migration refusals.
- Existing `yetlet-config` base/profile loading projected into strict CLI types,
  scoped APP_CLI overrides, protected configuration roots and no Development
  secret-file reads for CLI behavior.
- Public CLI settings in all scaffolds, app settings opt-in, implicit settings
  for Vite, complete frontend/ sources, configurable project-relative root and
  confined frontend dotenv loading. Cargo owns explicit local dependency overrides.
- Independent auto_start/auto_build selection, ordered frontend-before-Rust
  builds, explicit skipped stages, frozen Cargo and no implicit installs.
- Native two-tree process ownership/readiness, stage-attributed native exits,
  backend ENV exclusion from frontend, bounded cancellation/cleanup and terminal
  restoration. Windows implementation has compile-only evidence for this revision.
- Doctor schema 1, environment-only semantics, protected fixed local probes,
  3-second/64-KiB combined-output limits and a further 2-second cleanup budget.
- K8s replicas from shared settings. Generator source/lock/export qualification
  remains the legacy YT-101 matrix, with no automatic new-representation acceptance.

The implementation is in `tools/yetlet-cli` source, native filesystem/process
adapters, selected scaffold templates, dependency manifest/lock and focused tests.
Root/CLI guides, contract and backlog/roadmap notes distinguish current behavior
from historical evidence and remaining work. No framework runtime/core/browser
API was changed.

## Validation

All Cargo validation uses the installed toolchain, offline resolution and
`--locked`, with rustup auto-install/update disabled. Evidence directory:
`artifacts/cli-implementation-20261018/`.

| Check | Result and boundary |
|---|---|
| `cargo test --manifest-path tools/yetlet-cli/Cargo.toml --offline --locked --all-targets` | PASS: 100 tests, four explicitly ignored generated-app/tool-dependent tests; Linux native evidence |
| Selected ignored generated Vite/settings runtime test | PASS: one generated service/Vite/plugin app, explicit compatible Cargo path overrides, separately prepared offline lock, compiled runtime settings/custom-root tests |
| `cargo clippy` for CLI all targets, `-D warnings` | PASS on Linux and Windows MSVC cross-target; no native Windows execution |
| Windows MSVC `cargo check --tests` | PASS; compile-only, no Windows startup/cancellation/probe claim |
| All 16 creation combinations | PASS with tools/cache unavailable, no app lock/install/build, exact settings/layout choices and no-overwrite preservation |
| Linux frontend process tests | PASS 9/9: independent settings/skip branches, literal Unicode root, ENV filtering, build order, native exit/signal attribution, both-tree descendant cleanup, occupied-port refusal and real PTY restoration |
| Native probe tests | PASS 5/5: fixed allowlist, success/failure, combined output bound, deadline and resistant-descendant cleanup |
| Real Vite/Tailwind pipeline | PASS: explicitly prepared cached dependencies with `npm ci --offline --ignore-scripts`, then CLI frontend production build before frozen Rust build |
| Frontend dotenv boundary | PASS: built JS contains the frontend-local public marker and excludes the repository-root marker |
| Real `yetlet run` HTTP smoke | PASS: backend3000 and frontend5173 return200, development client/nonce CSP present, .env/traversal requests403; SIGINT preserved as native -2; all five observed owned processes gone and both listeners closed |
| Documentation, formatting and diff | PASS: docs check (75 documents/103 backlog items), CLI fmt and git diff --check; final record updated after review |

Real smoke inputs are generated task-owned files with explicit local Cargo
overrides and synthetic token/public markers. No existing app or runtime secret
was used. The dependency preparation is an explicit verification action, never
a hidden CLI install step. HTTP smoke is not a browser/HMR qualification claim.

The live build/run check exposed npm output directories inheriting umask002 under
a private frontend root. A CLI-only manifest read now recognizes protection by
that private ancestor, retains owner/no-follow/size/identity checks, and leaves
the legacy generator input path unchanged. The process fixture covers this
permission shape. The original failed preflight is retained in
`live-run-initial.stderr`. A subsequent smoke wrapper's raw bind check confused
TCP TIME_WAIT with a surviving listener; the corrected check verifies refused
connections after owned-process cleanup. No product change followed that wrapper
failure. The generated-app fixture's manifest serializer was corrected in place;
its final single-app check passes.

## Independent review and remaining gates

Initial independent implementation review failed with one P2: selected Tailwind
CSS still scanned the former flat Rust source path, so the real build omitted
SSR utilities. The original failure is retained in review-initial.json.
The selected scaffold now uses a corrected default relative source plus a
pre-transform driven by the CLI-owned YETLET_PROJECT_ROOT child context. This
context follows the selected manifest, including nested roots, without another
settings format or CLI option. Legacy template source bytes remain unchanged.

Real production builds now emit bg-blue-100, p-4 and font-bold at the default
root, then those three plus a newly added Rust mt-8 utility after moving the
frontend to a nested path containing spaces and Unicode. The result files are tailwind-default-result.json and
tailwind-nested-result.json. Selected creation/process tests cover the generated
adapter and correct context overriding. Final focused independent Reviewer reinspection passed with zero issues and
zero required followups. The terminal result is review-final.json; native trace
review-final-trace.json verifies registered reviewer role, gpt-6-astra model and
high effective effort. Shared resolver evidence records adaptive deep/high;
selector causality remains indeterminate and is not claimed. The initial failed
review is preserved. Final documentation/whitespace checks passed after this
record update. No design review substitutes for implementation review.

This initial implementation run did not execute the new Windows native paths.
The later [doctor/native CLI follow-up](YT-102-DOCTOR-VERIFICATION.md) and
[Windows Vite pair verification](YT-102-WINDOWS-VITE-VERIFICATION.md) record
source-matched Windows GNU execution separately from the original compile-only
evidence, including six selected real Vite/Rust lifecycle cases.
The original live doctor failed because the host Rustup paths were group-writable; that failure and its original permissions are retained as historical evidence. The owner later authorized the [permission repair and cross-platform doctor verification](YT-102-DOCTOR-VERIFICATION.md). Current results are recorded there; the protected-executable policy remains unchanged.

New registry-version/shared-settings/full-Vite layouts remain outside the
qualified generator source representation. The crate declarations are
unpublished and require explicit local Cargo overrides for current evaluation.
No release/production, arbitrary proxy/auth/actions, Rust watcher or deployment
support is claimed. YT-102 remains Todo for its full journey audit, measurements,
DX comparison and Extension Contract. The next unblockable backlog item is the
broader YT-102 DX research; additional platform/runtime qualification remains
bounded by the accepted evidence. YT-020 remains Blocked.

## Source preservation and changed paths

The exact implementation path list, source hashes, original-file comparison,
branch/HEAD and unchanged-index result are retained in
`artifacts/cli-implementation-20261018/final-state.json`. The original YT-101
compose/docker implementation and app config template remain byte-identical to
the pre-implementation snapshot. Native filesystem changes preserve the original
work while adding the explicitly selected frontend layout. No reset, clean,
stage or commit was performed. Task-owned servers/probes and native child trees
were stopped and reaped; no listener or runtime process remains.
