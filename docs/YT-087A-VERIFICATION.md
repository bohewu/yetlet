# YT-087a — Unpublished CLI Native SSR Preview Verification

**Task:** YT-087a. **Latest state (2026-10-16):** Original first independent Reviewer FAIL and its three later test-fixture failures are retained as history. The resumed local CLI verification now passes 15 tests plus check/fmt/strict Clippy; **087a independently accepted** after a separate read-only Reviewer PASS (see final addendum). YT-087 is In progress, not Done. No release, public CLI installation, registry publication, Git push or deployment is authorized. Baseline: `master` HEAD `94cf8f555b554a26c2416842eab06110b229904d`, existing checkout `$HOME/repos/yetlet`. YT-082–YT-086 remain committed and independently accepted.

## Design and authority

The unpublished local-preview design is in [YT-087 plan](YT-087-CLI-PLAN.md). Initial independent design reviewer `wc_job_YVAgSOYdrlfPO5qz` returned FAIL for a missing first-app Cargo.lock workflow and unsafe replacing publication. Corrected contracts explicitly require the user's separate offline lockfile generation and a directory-descriptor-anchored no-replace publish. Independent focused design re-review `wc_job_-ZsSRbXwI3fDdoBQ` returned PASS, zero issues/followups, for that design only. Neither review accepts CLI implementation.

## Interrupted role execution and bounded continuation

Generalist implementation was initially dispatched through a WebCodex-launched, role-directed `codex exec` Job `wc_job__3DiWnfRkzAjXaJ-` on the existing checkout. The Runner instance was replaced while the Job was active, leaving `status=lost`, `command_execution_state=outcome_unknown`; **no successful terminal Generalist verdict exists**. Its known process IDs were inspected after recovery and were absent; no matching Codex/Cargo process was running, allowing no concurrent or uncertain work to remain.

The parent Web session then applied **current-session role emulation**, after reading the canonical Generalist role and reasoning/materiality policies. This is not managed dispatch, not completion by the original role-directed Job, and **not an independent review**. It preserved the original partial source files, formatted the CLI, added focused parser/template/diagnostic tests, completed local verification and this document. The independently reviewed YT-086 implementation and receipts were not reopened. No replacement Generalist Job was started.

## Actual deliverable

- `tools/yetlet-cli/` is its own unpublished Rust 1.99 Cargo workspace, `publish = false`, with separate `Cargo.lock`, binary `yetlet` and no root workspace/xtask dependency graph changes.
- Supported preview commands: `yetlet doctor [--framework-path PATH] [--json]` and `yetlet new NAME --framework-path PATH [--dir DIRECTORY]`. `build` and `dev` remain unimplemented YT-087b commands; calling them fails as a usage error, rather than silently succeeding.
- `doctor` reports sanitized local Cargo/rustc 1.99-or-newer and optional framework-source capability without installation/download. `new` validates a local framework source and generates an independent Axum/Tower + Yetlet `App`/native `View`/Page SSR scaffold without JS/WASM, Controller, DI or optional providers.
- The standalone manifest has explicit, correctly escaped local `yetlet-core` and `yetlet-server-axum` path dependencies. The user must explicitly create the **application's own** Cargo.lock using `cargo generate-lockfile --offline --manifest-path APP/Cargo.toml`; subsequent native build uses `--frozen`. Neither `new` nor `doctor` implicitly builds, fetches or installs.
- On Linux, `new` uses directory-FD-anchored `openat` without symlink following, a uniquely owned temporary sibling, `renameat2(RENAME_NOREPLACE)`, identity-aware cleanup and fail-closed ambiguous post-move states. Windows `new` explicitly fails closed pending a separately verified no-replace/reparse implementation. **No Windows native execution or support PASS is claimed.**

## Focused Rust validation (2026-10-15)

| Actual command | Result |
|---|---|
| `cargo check --offline --manifest-path tools/yetlet-cli/Cargo.toml --all-targets` | Exit 0; first offline resolution produced only the isolated CLI Cargo.lock |
| `cargo fmt --manifest-path tools/yetlet-cli/Cargo.toml --check` | Exit 0 after formatting the interrupted implementation |
| `cargo test --offline --locked --manifest-path tools/yetlet-cli/Cargo.toml` | Exit 0; **14 passed, 0 failed, 0 ignored** across safe-directory race, CLI parsing, name validation, diagnostic redaction and scaffold/manifest tests; no binary unit or doctest cases |
| `cargo check --offline --locked --manifest-path tools/yetlet-cli/Cargo.toml --all-targets` | Exit 0 |
| `cargo clippy --offline --locked --manifest-path tools/yetlet-cli/Cargo.toml --all-targets -- -D warnings` | Exit 0 |

The test cases explicitly exercise late creation of a competing destination file, directory or symlink, ancestor/parent substitution and uncertain cleanup without deleting foreign entries. They are focused filesystem tests, not Windows guarantees or exhaustive adversarial assurance.

## One bounded real external-project and HTTP check

Evidence directory: `target/m13/yt087a/local-vH7Tot/`, retained in the ignored local `target/` area.

One bounded Linux check performed these steps on a unique external temporary project:

1. Ran CLI `doctor --framework-path . --json` (report `ok=true`).
2. Ran CLI `new yt087probe --framework-path . --dir <owned-external-temp-parent>`. Verified the generated manifest and source exist and **no application Cargo.lock** was created by `new`.
3. Explicitly ran `cargo generate-lockfile --offline --manifest-path <app>/Cargo.toml`, `cargo check --frozen`, and `cargo build --frozen`: all native exit 0 with already cached dependencies. Root, xtask and isolated CLI lockfile hashes remained unchanged.
4. Launched the generated binary on loopback `127.0.0.1:45671` (`YETLET_PORT=0`), fetched `GET /` and verified HTTP 200, Content-Security-Policy header, escaped literal text `Hello &lt;Yetlet&gt; &amp; Rust!`, and no `<script` tag. The application was not deployed.
5. Sent SIGTERM and waited for the task-owned server to exit. Fresh inspection found no remaining app process. The external temporary project was removed and verified absent. No other applications or windows were closed.

Actual outputs are retained in `doctor.json`, `new.log`, `generate-lockfile.log`, `app-check.log`, `app-build.log`, `server.log` and `response.http` in the evidence directory. Generated application runtime verification is source-bound to this candidate; it is not release qualification.

## Acceptance boundary and handoff

Current status is **087a implementation candidate, pending its own independent read-only Reviewer PASS**. This parent continuation is self-review and cannot satisfy that gate. 087b (`build`/`dev`) and 087c (final closeout) remain Todo within YT-087; YT-020 remains Blocked. Retain the historical Runner-loss condition and accepted M12 reviews without relabeling them as new evidence. No source staging, commit, push, remote installation, hosted workflow, publish or deployment occurred.
## Post-Reviewer source corrections — blocked validation (2026-10-15)

The first independent implementation Reviewer Job `wc_job_svbKz5pUuUI1cBVh` returned **FAIL**, identifying two material conditions that prevent 087a acceptance: (P1) on a shared group-writable non-sticky parent, another user could substitute an unrelated 0700 staging directory in the `mkdirat`/ `openat` gap; (P2) new application names matching the local 0.1.0 dependencies `yetlet-core`, `yetlet-html` or `yetlet-server-axum` could lead to Cargo package identity conflicts.

A **current-session role emulation** correction, not a Generalist Job result, added `Anchor::require_protected_create_parent` to refuse untrusted non-sticky parent configurations before creating temporary entries, and added the three local dependency names to `scaffold::validate_name` with direct negative tests. The security refusal remains intact. The new group-writable parent negative test passes.

**Current-source validation is NOT PASS.** The first post-correction CLI suite reported 7 passed / 8 failed. Diagnostics determined that `tempfile::tempdir()` on this host honors umask 0002 and creates mode 775 directories. A minimal canonical-test adjustment explicitly set test root directories to 700; the next suite reached **12 passed / 3 failed**. The remaining failures (`parent_directory_or_symlink_substitution_fails_closed`, `ancestor_swap_is_detected_even_when_final_parent_is_held`, `swap_after_parent_check_never_publishes_into_substituted_parent`) create nested `parent` directories with `fs::create_dir` / `fs::create_dir_all`, which also acquire group-writable mode 775 and are rejected before reaching their intended race hooks. Under the Generalist role's repeated harness-failure stop rule, work stopped rather than making another fixture edit or repeating tests. No product acceptance or independent Reviewer PASS is inferred from the earlier 14-pass source baseline.

**Resumable next gate:** inspect the three named test functions in `tools/yetlet-cli/src/safe_dir.rs`, establish mode 700 for their intended trusted test-parent directories while keeping the separate 777 refusal fixture, then run one bounded focused CLI test/check/fmt/strict Clippy and request a distinct independent read-only Reviewer re-review of both original P1/P2 issues. Do not weaken `require_protected_create_parent`, skip tests, or repeat the already accepted HTTP smoke solely for redundant evidence. Windows support remains unverified and fails closed. YT-087 remains In progress, 087a has **not** passed independent implementation review, and 087b/087c have not started.

The latest failed `cargo test --offline --locked --manifest-path tools/yetlet-cli/Cargo.toml` had native exit 101; later Cargo check/Clippy in that chained command were not reached. The original successful generated-app SSR evidence at `target/m13/yt087a/local-vH7Tot/` remains historical to the pre-correction source, not a post-correction full-suite PASS. The original `wc_job__3DiWnfRkzAjXaJ-` remains lost/outcome_unknown (Runner replaced); original PIDs were absent before this role emulation. No new Generalist Job, staged files, commit, push, deployment, release, or hosted trigger occurred.
## Resumed YT-087a fixture verification (2026-10-16)

Following the maintainer's explicit continuation, the original checkout at `94cf8f555b554a26c2416842eab06110b229904d` and WebCodex Job inventory were checked: no competing execution was active. **Current-session role emulation** changed only three existing `safe_dir.rs` tests: the trusted nested `parent` directories created by `fs::create_dir` / `fs::create_dir_all` now explicitly use mode `0700`, independent of host umask `0002`. The production `Anchor::require_protected_create_parent` rule was not loosened; the separate group-writable non-sticky (`0777`) refusal test remains active, as do no-replace/race, cleanup and ownership tests.

Fresh source-bound validation results: `cargo fmt --manifest-path tools/yetlet-cli/Cargo.toml --check`, `cargo test --offline --locked --manifest-path tools/yetlet-cli/Cargo.toml` (**15 passed, zero failed/ignored**), `cargo check --offline --locked --manifest-path tools/yetlet-cli/Cargo.toml --all-targets`, `cargo clippy --offline --locked --manifest-path tools/yetlet-cli/Cargo.toml --all-targets -- -D warnings`, and `git diff --check` all returned exit 0. Earlier 7/8 and 12/3 failure receipts above remain historical and are not rewritten. No new browser/SSR run was necessary for these test-fixture-only corrections; the previously accepted local HTTP smoke remains source-scoped to its original execution.

**Acceptance remains pending an independent read-only implementation re-review** of the original P1/P2 issues and the corrected fixtures. Windows native `new` remains fail-closed and unverified. The original lost Generalist Job still has `outcome_unknown`, not a fabricated success. No worktree, staging, commit, push, installation, publication or hosted trigger was performed by this continuation.

## Final independent YT-087a acceptance (2026-10-16)

Independent read-only Reviewer Job `wc_job_eWTKCMqqc5Ox5Bzn` completed with `overall_status=pass`, `issues=[]`, and `required_followups=[]`. It confirmed original P1 and P2 closed in current source and that the three nested-parent fixture permissions preserve the exact adversarial parent-replacement tests, while the separate 0777 refusal fixture stays active. The role read actual CLI code, source-bound test evidence and original M13 plan; its verdict permits closing **087a at Linux-only unpublished CLI preview scope** and starting 087b. The Reviewer did not execute filesystem-mutating tests or claim Windows support. The initial Generalist Job remains terminal lost/outcome-unknown; current-session role emulation and the independent review are distinct, not managed dispatch. YT-087 as a whole stays In progress; no staging, commit, push, global installation, publication or deployment occurred.
