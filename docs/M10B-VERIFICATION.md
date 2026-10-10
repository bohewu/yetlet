# M10b Verification — Repository Governance

**Date:** 2026-10-10
**Task:** YT-061 / M10b
**Status:** Done. First-reviewer evidence/documentation P2s repaired; repeat independent reviewer `wc_job_lu2WynqvKYemLlmd` PASS.

## Maintainer-directed contribution model

The repository remains **Private**. Pull request creation is set to `collaborators_only`, so the setting also preserves the maintainer-directed model if the repository later becomes Public. External users may report bugs or proposals through Issues, but cannot open upstream pull requests. The only current direct collaborator is the repository owner.

Repository merge policy:

- squash merge only;
- merge commits and rebase merges disabled;
- merged branches deleted automatically;
- no DCO/web sign-off requirement;
- no auto-merge.

## Protected `master`

Branch protection is active and enforced for administrators:

- changes require a pull request, with zero human approvals for the current sole-maintainer phase;
- required checks: `Rust 1.88 MSRV` and `Stable native, samples, and Chrome`;
- conversation resolution and linear history required;
- force pushes and branch deletion disabled;
- repository administrators have no routine bypass through `enforce_admins`.

This keeps maintainer PRs and CI as the merge gate without pretending a second human reviewer exists. Adding another maintainer should trigger a future review of the approval count and bypass policy.

## Actions and dependency security

- workflow token default remains read-only and cannot approve pull requests;
- Actions are restricted to GitHub-owned actions;
- full commit-SHA pinning is required by repository settings;
- checkout credentials remain disabled in the checked-in workflow;
- Dependabot vulnerability alerts and automated security fixes are enabled.

The following settings are unavailable while the repository is Private under its current account/plan and were not bypassed:

- GitHub Private Vulnerability Reporting returned HTTP 404;
- secret scanning/push protection returned HTTP 422 (`not available for this repository`);
- CodeQL default setup returned HTTP 403 because code scanning is not enabled/available.

They must be retried when the maintainer formally changes the repository to Public. Their present unavailability does not weaken the checked-in secret-handling policy or RustSec/CI gates, but it remains a public-transition checklist item.

## Evidence commands

```bash
gh api repos/bohewu/yetlet
gh api repos/bohewu/yetlet/branches/master/protection
gh api repos/bohewu/yetlet/actions/permissions
gh api repos/bohewu/yetlet/actions/permissions/selected-actions
gh api repos/bohewu/yetlet/actions/permissions/workflow
gh api repos/bohewu/yetlet/vulnerability-alerts
gh api repos/bohewu/yetlet/automated-security-fixes
```

The reviewer must verify the live settings rather than relying only on this document. No repository visibility change, release, package publication, CLA/DCO, CODEOWNERS gate or public contribution workflow is part of YT-061.

## First independent-review findings and repair

Independent read-only reviewer Job `wc_job_ISJjq_EUBG8tmQfQ` accepted the live governance payloads but returned two P2 findings:

1. the retained evidence did not include the existing GitHub check-run response proving that the two protected contexts were successful GitHub Actions checks with app ID `15368`;
2. two older readiness sentences still said dependency automation or continuous vulnerability monitoring was entirely absent, contradicting the newly enabled Dependabot alerts/security fixes.

No CI rerun or setting mutation was required. `target/m10ab-evidence/check-runs.json` now records commit `86c30a42114ca74f4197f0389707861308a844a7` with exactly:

- `Rust 1.88 MSRV` — completed/success, GitHub Actions app ID `15368`;
- `Stable native, samples, and Chrome` — completed/success, GitHub Actions app ID `15368`.

The stale documentation now distinguishes enabled Dependabot alerts/security fixes and per-CI `cargo-audit` from the still-missing scheduled version-update configuration, secret/code scanning and broader cross-ecosystem monitoring. Focused documentation and retained-setting checks then passed.

## Reviewer gate

Repeat independent role-directed reviewer Job `wc_job_lu2WynqvKYemLlmd` completed at terminal exit 0 with `gpt-6-astra`, high reasoning effort and a read-only sandbox. It verified the retained GitHub responses, exact successful check runs/app ID, branch protection, collaborator-only PR policy, Actions restrictions, Dependabot state, corrected monitoring wording, checked-in workflow and blocked release status. It returned:

```json
{
  "overall_status": "pass",
  "issues": [],
  "required_followups": [],
  "optional_notes": []
}
```

This was ordinary local `codex exec <reviewer prompt>`, not `codex exec review` or native ACP dispatch. The reviewer changed no files or settings. YT-061 is **Done** for the current Private repository state. Private Vulnerability Reporting, secret scanning/push protection and CodeQL remain explicit Public-transition activation items; green governance evidence does not close YT-020.
