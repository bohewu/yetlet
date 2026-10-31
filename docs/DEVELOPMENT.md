# Maintainer development workflow

Use this guide for authorized changes to Yetlet itself. Application authors
should start with [your first app](FIRST-APP.md). The
[contribution policy](../CONTRIBUTING.md) still excludes unsolicited external
code PRs; forks and bug reports remain welcome.

## Normal path

`bounded change -> focused checks -> independent review when required ->
authorized commit/push -> PR CI -> authorized merge -> source reconciliation`

The actual default branch is `master`. Use a short-lived `feature/*` or `codex/*`
branch and a PR into protected `master`. A `dev` integration branch is optional,
not a prerequisite or a setting enabled by this guide. If selected separately,
use PRs into `dev` and then a PR into `master`; neither branch replaces review.
Ordinary development does not run release jobs or access release secrets.

| Step | Do | Exit condition |
|---|---|---|
| 1. Bound the work | Read [AGENTS.md](../AGENTS.md), the item/ADR and relevant guide; inspect branch, HEAD, status and existing evidence | One clear change; unrelated work preserved |
| 2. Make the change | Work in the current checkout by default; keep opt-in/package/target boundaries | Behavior and public API changes match the accepted scope |
| 3. Verify the delta | Choose existing checks for changed behavior and affected consumers | Required checks pass, with exact command/source/feature/target recorded |
| 4. Review | Obtain independent Reviewer PASS where the item requires it; separate self-review from independent review | No unresolved material issue; original failures remain recorded |
| 5. Commit and deliver | Obtain Git authority, stage only the intended paths, run strict/builtin Guard, then normal hooks | Reviewed scope and index tree match the actual commit; outgoing range passes before Push |
| 6. PR and merge | Wait for required CI on the exact PR head; merge only under the owner's authority | Expected head checked; actual merged SHA/tree recorded |
| 7. Close the item | Update Backlog/Roadmap from actual evidence | Done only at the verified scope; release/support claims remain separate |

Do not reset, clean, overwrite another change or create a detached worktree to
make a delivery look clean. A separate branch/worktree or history reorganization
needs the authority appropriate to that operation.

## Local checks and CI

Release-tool/document-only changes use a bounded CI lane when the complete Git
delta matches an explicit allowlist and both required checks have already passed
on the exact protected-master base. Both required job names still execute and
record the reused SDK source/run. CI has only `contents: read` and `actions: read`;
the latter reads baseline run/job metadata. This is source correspondence, not a new
Rust/browser qualification. Changed release tools run their focused Python tests,
and documentation runs the existing checker. Any SDK, package/lockfile, sample,
xtask or unknown-path change uses the full original checks. Missing history,
uncertain scope or unavailable/failed baseline CI also selects the full lane.
Release preparation/signing/result/publisher gates and Commit Guard are unchanged.


Run the smallest existing check set that proves the change. Examples of selection:

| Change | Verification to select |
|---|---|
| Documentation only | `cargo xtask check docs`, `git diff --check`, exact new-file content/whitespace and link/snippet review |
| Rust behavior | Owning package's meaningful tests, affected feature/consumer checks, applicable format/lint |
| Browser/loader behavior | Affected WASM build and the relevant browser/CSP/lifecycle flow; report browser evidence separately |
| Dependency/package metadata | Affected lock/feature/target, package and inventory/security checks; reconcile unchanged evidence |
| Release tooling | Focused guard/identity/format tests and review; real hosted/key/signing actions need release authority |

Do not rerun an accepted suite merely because a later document changed. Reuse
evidence only when its source/input/feature/target correspondence is established.
New or affected inputs need new proof; a green result for another source is not
a substitute. A README included in a Cargo archive is a package input.

[rust.yml](../.github/workflows/rust.yml) is the existing unprivileged PR/master
CI lane, with required check names **Rust 1.99 MSRV** and **Stable native,
samples, and Chrome**. Preserve their names and protection requirements. Local
focused checks do not replace required CI. CI success covers its actual matrix,
not every platform or feature. A squash merge has a new commit identity: record
its actual SHA/tree and the resulting master CI without relabeling the PR run.

## Batch small documentation changes

Collect related spelling, wording, navigation and recorded-status corrections
into one coherent delivery at a useful checkpoint. Prepare the complete local
change before opening a PR; update that PR for an affected correction instead of
creating another PR for every small edit. Unchanged accepted checks are evidence
to reuse at their original scope, not work to repeat for a new paragraph.

| Actual effect | Proportional delivery and checking |
|---|---|
| Wording, formatting or navigation only | Batch related edits; use documentation, link and whitespace checks plus the existing bounded CI lane when eligible |
| Commands/examples, API or security meaning, versions/checksums, publication instructions | Review the changed meaning and affected source/evidence; use the relevant existing checks even when the edited file is Markdown |
| Code, dependencies, package inputs or workflow behavior | Complete affected local checks and required PR CI; select full qualification when the existing scope classifier requires it |

This guidance retains protected `master`, Git authorization, strict/builtin Guard
and required check names. It does not select direct-to-master delivery or change
branch protection. A future alternative delivery policy needs a separate owner
decision. Under the existing path, reduce PR frequency by batching work and CI
cost by selecting the bounded lane, while keeping the required checks present.
GitHub documents that skipping a required workflow with path filters or
`[skip ci]` can leave its checks Pending and block Merge; use the existing
[bounded lane](#local-checks-and-ci) rather than that skip mechanism.
See [GitHub's explanation](https://docs.github.com/en/actions/how-tos/manage-workflow-runs/skip-workflow-runs)
and the [release retrospective](RELEASE-RETROSPECTIVE.md).

## Git authority and strict Commit Guard

For agents, implementation authority is not Commit, Push, Merge, tag or release
authority. An owner may approve a concrete batch once, including its exact scope,
CI condition, merge method and subsequent steps. Continue that approved batch
without asking again for each already-authorized step. A passing check grants
no additional authority.

1. Stage only the approved delivery paths; inspect the staged filenames.
2. Invoke the installed trusted Guard directly with `check --staged --profile
   strict --engine builtin --format json`; hook presence is not a replacement.
3. Require exit0, effective strict/builtin and no blocking findings. Immediately
   compare `tree_oid` with `git write-tree` before Commit; rerun after any index
   mutation. Keep normal pre-commit hooks enabled.
4. Before an authorized Push, check the exact accepted remote base to outgoing
   tip with `check --range BASE..TIP --profile strict --engine builtin --format
   json`; keep normal pre-push hooks enabled. Inspect the actual outgoing refs.
5. Preserve Guard outputs, commit/tree and remote readback. Strict is a bounded
   first-screen, not proof that all historical content is clean.

Use the tool path from the local managed installation; never commit personal
installation paths. If a new remote ref would scan unaccepted history or exceed
limits, stop and establish an explicitly accepted existing-remote boundary.
Where the approved batch accepts an existing remote base, a new remote branch
can first be created at that already-existing base, then receive the separately
Guard-checked outgoing commits. Record the live base and actual ref update;
this is not acceptance of newly introduced findings or historical cleanup.
Do not default to an orphan branch, history rewrite, force push, audit profile,
`--no-verify` or hook edits. The existing-remote **tag** path and used-binding
retirement are release-specific controls, not everyday branch steps.

## API, documentation and failures

Follow [naming and migration guidance](NAMING.md), relevant ADRs and
[guide structure](GUIDE-TEMPLATE.md). Update API/setting compatibility notes and
consumers when their contracts change. Keep all repository documentation in
English. Use current source/rustdoc for API contracts; dated reports retain their
own verdicts and limits.

Classify an observed failure before acting. Repair the smallest real defect.
A local wrapper error may need a focused correction, not a new qualification
campaign. Preserve passed evidence and the original failure; no timer, allocation,
key-use or attempt budget is renewed by renaming a run. Before repeating an
uncertain external write, inspect its actual state. A failed hosted job, partial
signing or exhausted authorization does not permit an automatic retry.

For source freezing, tags, protected jobs, signing and publication, switch to
[the release workflow](RELEASING.md). The release freeze is a bounded promotion
window, not a permanent block on development: feature branches may continue,
while master merges wait only when the selected signer gate requires it.
