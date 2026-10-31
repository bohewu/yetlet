# Release retrospective — through CLI 0.1.1

Date: 2026-10-31. Maintainer lessons from YT-096, YT-020 and YT-209–YT-216.
This synthesis reuses retained records; it does not replace their original
FAIL/PASS verdicts, run attempts, source identities or qualification limits.
The operational entry points remain [Releasing](RELEASING.md),
[publication automation](RELEASE-PUBLICATION.md) and
[development](DEVELOPMENT.md#batch-small-documentation-changes).

## Successful reference execution

Owner-approved [CLI publication 41174511818/1](https://github.com/bohewu/yetlet/actions/runs/41174511818)
completed successfully. The existing `yetlet-cli-v0.1.1`
[GitHub pre-release](https://github.com/bohewu/yetlet/releases/tag/yetlet-cli-v0.1.1)
has its five planned public assets; registry metadata and downloaded archive
checksums agree. The actual fresh, credential-free registry consumer installed
`yetlet-cli 0.1.1` with its packaged lock, printed exactly `yetlet 0.1.1` plus
one newline, displayed help and created `consumerapp`. Generated metadata pins
SDK `=0.9.0-alpha.5`, without path, patch or replace overrides. The SDK was not
republished. Project creation is not a new generated-project compilation or
browser qualification claim.

Candidate C is `ae5dd4c55e1124f56fcc3b07a9fc9e5bf083b25f`; S/V/P is
`1207ba5c7dbb7611df27d971a856c69538feeb0b`. The 471926-byte CLI archive has
SHA-256 `10a1ffebf42de0baba9604ee2764b2304f601c5119ddcb5fcb988fbe319e1424`.
Publication artifact `13031589501` has ZIP SHA-256
`fa35871ed0d02d68929ad9a3bd1f05dceaa8ec2989e991299c796d679bb5a394`.
Its plans, receipts, consumer logs/lockfile, actual owner approval and anonymous
public reconciliation are retained in the repository-external release-evidence
store. All new-version release stages used attempt 1. The earlier, separately
authorized source-CI retry below remains part of this version's history.

## Confirmed failures and prevention

| Incident and retained record | What failed or was missed | Smallest lesson for the next change |
|---|---|---|
| [YT-209 PR20 integration](YT-209-RELEASE-AUTOMATION.md#subsequent-integration-ci--2026-10-29) | Direct helper checks missed an existing `importlib` caller (`ModuleNotFoundError: release_record`) and old custody/preparation fixtures. | Before delivery, find callers of the changed interface and run their affected existing cases locally. Passing new-helper tests alone does not cover old callers. |
| [Alpha.2 fixture migration](ALPHA2-VERSION-PREFLIGHT.md#subsequent-pr22-integration-failure-and-local-correction) | Synthetic package versions/filenames remained alpha.1 after the recipe began deriving alpha.2. | Distinguish current-version fixtures from deliberate historical/negative pins; derive the former from the same version authority. |
| [Alpha.2 custody binding](ALPHA2-HOSTED-RECORD.md#actual-build-and-evidence-discrepancy) | Build/archive checks passed, but the actual custody receipt named the alpha.1 tag. Active helper constants had been mistaken for history. | Inspect serialized output identities before creating an immutable candidate/tag. A green build does not validate every downstream receipt. Keep the wrong receipt unchanged. |
| [Real-layout qualification collision](YT-096-SIGNER-CONTRACT.md#pre-hosted-real-layout-correction--2026-10-28) | Two legitimate input groups used the same exclusive destination names for OpenSSL build logs. | Exercise the real input layout in the existing assembly case. Retain both byte sets with an explicit name map; do not overwrite or discard evidence. This was found before dispatch. |
| [Prepare input delivery](YT-096-SIGNER-CONTRACT.md#actual-blocked-delivery) and [alpha.5 capability lifetime](YT-096-SIGNER-CONTRACT.md#alpha5-short-lived-capability-correction--2026-10-31) | One hosted Draft endpoint returned 403; later, an actual five-minute download capability contradicted the assumed longer lifetime. | Prove the supported permission/data path and actual deadline. The reviewed encrypted handoff and bounded early intake separate time-sensitive retrieval from slow tool preparation. Never expose the capability or extend its lifetime by assumption. |
| [Prepare cleanup](YT-096-SIGNER-CONTRACT.md#actual-frozen-output-and-go-cache-cleanup-failure--2026-10-29) | Data verification/freeze/upload passed, but task-private read-only Go module-cache entries prevented cleanup. The overall job failed. | Verify cleanup of resources actually created. The scoped `-modcacherw` bootstrap correction preserves the cleanup gate; an uploaded artifact alone is not overall PASS. |
| [Alpha.3 upload](YT-209-RELEASE-AUTOMATION.md#publication-upload-maintenance--2026-10-30) and [alpha.4 upload](ALPHA4-HOSTED-RECORD.md#owner-approved-publication-failure-and-local-repair) | Generic errors hid the failing operation/status. A separate local loopback reproduced missing `Content-Length` with the observed `gh api --input -` path. | Use the canonical byte-exact upload path, explicit length, safe phase/class/status diagnostics and the existing loopback case. Preserve unknown historical HTTP causes as unknown; local reproduction does not identify an earlier unrecorded status. |
| [Alpha.4 registry response](ALPHA4-HOSTED-RECORD.md#actual-public-release-and-one-package-registry-outcome) | `yetlet-actions` was accepted, but the parser required `krate.name`; its mocks shared that false assumption. The other fifteen uploads never started. | Test against the provider's documented response contract, including warnings-only success. Resolve actual identity/content using index and download readback before considering any continuation. A parser failure after PUT is not proof of no write. |
| [CLI 0.1.0 consumer](YT-215-CLI-FIRST-RELEASE.md#actual-first-publication-and-consumer-failure--2026-10-31) | Publication and install succeeded, but the real executable printed a stale unpublished suffix. Help/new were not reached. | Check the real binary's exact version, help and project creation before delivery. Derive the CLI version from its own manifest. Published 0.1.0 remains historical; 0.1.1 is a distinct patch release. |

### Limits that must not be rewritten as known root causes

- The earlier generic prepare rejection is not retroactively explained by the
  later observed 403. Provider bodies/headers were not retained; the
  [blocked-delivery record](YT-096-SIGNER-CONTRACT.md#actual-blocked-delivery)
  states that limit.
- The [alpha.4 sixteen-package continuation](ALPHA4-HOSTED-RECORD.md#owner-selected-conservative-continuation-cadence)
  planned conservative pacing; it did not establish actual account quota or
  demonstrate a 429. The owner later selected two SDK packages and a separately
  versioned CLI. Package boundaries follow API, target, opt-in and installation
  needs; quota alone is not an architectural reason to combine everything.
- [Source CI 41172082238/1](YT-216-CLI-011-PATCH-RELEASE.md#selected-candidate-and-source-delivery--2026-10-31)
  failed an unchanged Lab SQLite concurrency assertion: expected 303/409,
  observed 303/503 Busy. Only the owner-authorized failed Stable job was retried;
  attempt 2 passed with the original MSRV execution reused. The observation does
  not prove a scheduler/SQLite root cause, fix the test or authorize future retries.
- The retained CLI Clippy findings and offline-cache/private-directory fixture
  failures have their own [YT-216 scope](YT-216-CLI-011-PATCH-RELEASE.md#selected-candidate-and-source-delivery--2026-10-31).
  Do not convert an inherited diagnostic or local harness failure into a new
  release implementation project without an unmet requirement.

## Reduce handoffs and repeated CI

Complete the changed interface and affected local caller checks before opening
a delivery PR. Use remote CI for the required source/platform checks and genuine
hosted integration; a local PASS never establishes remote permissions, provider
capability duration or a protected execution's actual output.

Batch small wording/navigation/status updates at a useful checkpoint. One PR
can deliver that complete documentation batch; each sentence does not need its
own PR, Merge or full SDK qualification. The existing bounded documentation/tool
CI lane still reports the required check names and reuses only a proven unchanged
SDK baseline. Commands, API/security contracts, versions/checksums, packaging and
workflow/tool behavior need the checks appropriate to their actual effect.
See [documentation batching](DEVELOPMENT.md#batch-small-documentation-changes).

For source-bound release work, establish C, later S and their input sets in the
right order. Accumulate tracking notes without changing frozen payloads; deliver
them with the next appropriate batch. A wording correction is not, by itself,
authority to start a new release chain. Preserve strict/builtin Git checks,
protected-master rules and the final owner publication approval.

## Small pre-delivery reminder

1. Identify the cohort, version authority, target/feature scope and changed
   callers. Reuse unchanged evidence at its recorded source/input scope.
2. Run the affected existing local checks, including actual binary behavior for
   CLI changes and actual serialized identities for release-tool changes.
3. Review the complete intended delivery once, then use an authorized Git batch;
   consolidate incidental documentation followups.
4. Bind actual C/S/V/P, tag, run/attempt, artifact IDs and digests from outputs.
   Check custody, expiry, cleanup and public/private boundaries at their stages.
5. Present the exact final publication set for the owner's gate. Record GitHub,
   registry and consumer results separately because publication is not atomic.
6. On failure, preserve the first receipt/log and inspect actual remote state.
   Continue only under the applicable authority; never automatically repeat a
   write, move a tag, replace accepted bytes or relabel a partial run PASS.

This retrospective adds no runtime, workflow, branch-protection or retry-policy
change. It records the successful publication without changing Backlog verdicts
or claiming a new independent review of the historical executions.
