# YT-236 — CodeQL PR concurrency and Rust analysis diagnosis

**Date:** 2026-11-07 (Asia/Taipei).
**Status:** Done at local source and static verification scope; independent Reviewer PASS.**Baseline:** `master` squash commit `f37f4e4f5e6e457553b18079f517ee71ff7d06b6`.
**Scope:** Concurrency scheduling and evidence-based performance diagnosis only.
No CodeQL rule, language, query-suite, event-trigger, security permission,
branch-protection, dependency, or build-mode change is authorized by this item.

## Why the change is bounded

PR #60 required roughly 14m44s for the Rust CodeQL job, even though its
other language scans and mandatory Rust CI had already completed. The matching
`master` push scanned the exact same source tree at a new squash commit.
Those are independent and intended analysis events: the PR checks a change
before merge, and the `master` scan updates the default-branch security baseline.

The existing workflow triggers on `pull_request`, protected `master` pushes,
weekly schedule and `workflow_dispatch`. It scans all four selected language
families unless its existing scope job establishes a Markdown-only change.
The only new behavior is that an updated PR supersedes an older unfinished
**CodeQL workflow run for that same PR**.

## Concurrency contract

The workflow-level key is:

```yaml
concurrency:
  group: yetlet-codeql-${{ github.event_name }}-${{ github.event.pull_request.number || github.run_id }}
  cancel-in-progress: ${{ github.event_name == 'pull_request' }}
```

- For `pull_request`, the event name and PR number form a stable key
  across new commits to that one PR. Only the old **same-PR CodeQL** run is
  canceled on a newer run, without affecting a different PR or other workflows.
- For `push`, `schedule` and `workflow_dispatch`, `github.run_id` is a
  unique key. Non-PR runs are not canceled, **including pending runs**: GitHub's
  default concurrency implementation otherwise replaces an older pending run
  when the group key repeats, even when `cancel-in-progress` is false.
- The `yetlet-codeql` prefix is specific to this workflow, preventing
  cross-workflow cancellation. No job is started or canceled by the file edit
  itself; activation happens only after separately authorized Git delivery.

GitHub concurrency behavior:
https://docs.github.com/en/actions/how-tos/write-workflows/choose-when-workflows-run/control-workflow-concurrency

## Rust CodeQL diagnostics (existing immutable GitHub run logs)

Sources (actual CI job logs, not a synthetic timing estimate):

- PR #60, Rust CodeQL:
  https://github.com/bohewu/yetlet/actions/runs/42369692279/job/128395088693
- Merged `master` commit `f37f4e4`, Rust CodeQL:
  https://github.com/bohewu/yetlet/actions/runs/42370814842/job/128398398509

Observed from the PR job:
- `No overlay-base databases found in Actions cache.`
- `Using overlay database mode: none without caching.`
- The `run-queries` command supplied a PR diff-range pack, evidence of a
  diff-informed PR analysis path; this is **not** a cached overlay database.
- Extractor also reported some files without a loadable manifest or module,
  which limits semantic macro expansion for those files; this task does not
  suppress those diagnostics or narrow the scan.

Observed from the `master` job:
- `Using overlay database mode: none without caching.`
- `Not performing diff-informed analysis because we are not analyzing a pull request.`
- Its query command used `--expect-discarded-cache` and database cleanup used
  `--cache-cleanup=clear`; no reusable overlay-base was demonstrated.

The existing action uses pinned `github/codeql-action@...` v4,
`build-mode: none`, the default query suite and the four-language matrix.
GitHub's 2025 diff-informed rollout covers supported languages, including
Rust, but the published 2026 improved overlay-cache rollout specifically
lists C#, Java, JavaScript/TypeScript, Python and Ruby (and later C/C++).
It does **not** establish Rust overlay caching support. Therefore no
`actions/cache` for CodeQL databases, artificial overlay flags, custom
CLI invocation, `trap-caching` modification, query-suite relaxation or
language exclusion is introduced.

References:
- https://github.blog/changelog/2025-09-23-incremental-security-analysis-with-codeql-is-now-available-for-all-languages/
- https://github.blog/changelog/2026-03-24-faster-incremental-analysis-with-codeql-in-pull-requests/
- https://docs.github.com/en/code-security/how-tos/find-and-fix-code-vulnerabilities/scan-from-the-command-line/incremental-analysis

## Verification and delivery boundary

Verify this exact YAML source using an actual YAML parser, the four-event
concurrency truth table (same PR/new commit, different PR, default-branch
pushes, weekly/manual runs), preserved `on` event map and analyzer matrix,
`cargo xtask check docs`, `git diff --check` and an independent read-only
Reviewer. No re-run of the 15-minute hosted CodeQL job is needed to prove the
static YAML expression; a future PR CI run remains the live gate.

Do not claim the change improves the latency of a single untouched PR or that
Rust overlay caching is available. The win is avoiding superseded PR analysis.
Do not Commit, Push, open a PR or merge without explicit separate Git
authorization, and do not change external GitHub settings.

## Local acceptance — 2026-11-07

- Changed workflow: `.github/workflows/codeql.yml`; the existing
  `pull_request`, `master` push, weekly schedule, and manual dispatch
  triggers, language matrix, permission scopes, skip behavior and CodeQL Action
  pin are unchanged.
- A PyYAML `BaseLoader` parsed the actual workflow (without YAML 1.1's
  `on`-as-boolean coercion); asserted the complete event map, `master`
  branch filter, cron, four languages, permissions, build mode and both
  concurrency expressions.
- Nine local event cases PASS: a new commit to one PR shares a group and
  cancels its previous run; a different PR has a separate group; two distinct
  `master` push runs, two weekly runs and two manual runs retain separate
  groups with cancellation disabled.
- `cargo xtask check docs` PASS: 75 indexed documents, 233 backlog entries;
  links, Markdown structure, whitespace and DAG accepted.
- `git diff --check` PASS.
- Independent read-only Reviewer Job `wc_job_q-3UNuOljhTITncN` completed
  with process exit 0, `overall_status=pass`, `issues=[]` and
  `required_followups=[]`. This was an ad hoc role-directed Codex Job
  using the saved Reviewer contract, not native managed role dispatch.
- No real GitHub concurrency-cancel event has been executed or observed yet.
  The first separately authorized PR is the actual hosted acceptance gate.

At the initial local-review checkpoint, the branch had not been committed
or pushed. The owner subsequently authorized completing Git delivery through
the PR. That delivery remains subject to strict/builtin Commit Guard and the
protected-`master` CI/merge gates; the review alone is not remote acceptance.
No CodeQL reruns, new caches, branch-protection changes, or GitHub security
settings were performed as part of the local implementation.