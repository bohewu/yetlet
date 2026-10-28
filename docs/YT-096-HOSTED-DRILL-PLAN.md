# YT-096 — First real hosted custody drill plan

Date: 2026-10-28. **Hosted build/readback and repository-external retention completed; original plan review PASS (zero issues and required followups).**
D1–D5 Adopted (policy); D6 Pending; YT-096 In progress; YT-020 Blocked.

**Initial reviewed plan snapshot (retained):**
No source is selected as `C_release`. No Commit, index/ref change, Push, PR,
Merge, tag, settings/Secret write, private-key read, hosted execution/approval,
OIDC, signing or publication is authorized by this packet.

**Subsequent signer design decision (2026-10-28):** The owner adopts the
[independent signer contract](YT-096-SIGNER-CONTRACT.md) and
[ADR 0009](adr/0009-release-build-and-signer-identities.md), amending D1/D3 while
preserving this completed build/readback/custody checkpoint and product tag.
Future signer `S`, final evidence scope/SHA, implementation/setup and execution
remain pending. No new payload preparation or key access is performed; D6 Pending.

## Actual hosted and custody checkpoint — 2026-10-28

This dated checkpoint supersedes the original Pending observations below. It
records completed, separately authorized operations; it grants no new operation.

| Binding / result | Actual evidence |
|---|---|
| Selected source | PR13 squash `C_release` `3eacfe31f5e3281411d3bc4653a6baccab668b30`, tree `86385d60cb4c3e595c670dbacce117ebeeb1a993`; [master CI accepted](https://github.com/bohewu/yetlet/actions/runs/40612890110). |
| Guard and source tag | Installed `agents_pipeline` 0.44.0 managed hook upgrade and strict/builtin doctor PASS; preparation and actual native pre-push PASS with zero outgoing commits. One-use existing-remote binding consumed. Unsigned lightweight `v0.1.0` pushed at `C_release`; observed remote `master` unchanged. Historical ancestry is accepted-existing debt, not clean history. |
| Hosted build | [40628943583](https://github.com/bohewu/yetlet/actions/runs/40628943583), attempt 1, manual `release.yml`, exact `refs/tags/v0.1.0` / `C_release`, `cohort=sixteen`; PASS, native exit 0, 259-second job, 2026-10-28 11:50:25–11:54:44 UTC+08. |
| Encrypted transport | Artifact `12791140773`, `encrypted-private-3eacfe31f5e3281411d3bc4653a6baccab668b30-40628943583-1`; ZIP SHA-256 `da4697ddb6df167dbda24355d40b7be1dd658be1be1dec4a3cbc5181d50ad7c9`. |
| Public transport | Artifact `12791125794`, `unsigned-preparation-40628943583-1`; ZIP SHA-256 `42aabfb4034767dba921e25c08cda6877f1f946c666e020789a6c2783b2ae63c`. |
| Protected verifier | [40631789238](https://github.com/bohewu/yetlet/actions/runs/40631789238), attempt 1, manual `evidence-verify.yml`, same immutable tag/source and exact original build/artifact inputs; PASS. Preflight 6 seconds; protected verify 61 seconds, 2026-10-28 12:21:57–12:22:58 UTC+08. |
| Environment approval | Actual `bohewu` approval retained in API observation; `release-evidence-verify`, exact `v0.1.0` tag policy and `can_admins_bypass=false` observed. Matching Environment-only `AGE_IDENTITY` used transiently by the approved verifier. |
| Safe receipt transport | Artifact `12791094206`, `evidence-readback-40631789238-1`; ZIP SHA-256 `14fc44399e875e5e1a1b7d958c413aee6cf50f0bb324ff2e2668c8fc8378aa08`. |
| Hosted readback | `PREPARATION_SUCCESS`, `complete=true`, original exit 0; decryption, inventory and full private-manifest verification true; isolated cleanup PASS. |
| Independent local correspondence | ZIP/ciphertext/safe-receipt, exact eighteen public-file checksums, sixteen provenance subjects, source/workflow/run bindings PASS. `source.tar` SHA-256 `e1469c4d0caffab779be86132b0da0d7565b837ec29e902d18ff9ca81852bc31` matches provenance. No local private-key or private-plaintext access. |
| Repository-external custody | Recorded 2026-10-28 12:25:12 UTC+08 at owner-host `$CUSTODY_ROOT`: 51 files, 129,846,094 bytes, directories0700/files0600, flushed and reread, retention index and receipt retained. Persistent owner-host storage through explicit owner disposition, no automatic expiry. Offsite/replicated backup and indefinite availability are not proved. |

GitHub artifacts expire on 2026-11-04 and remain seven-day transport caches. The
actual owner-host copy has no scheduled expiry; its existence and reread were
recorded separately from hosted upload/readback. This checkpoint completes the
bounded first hosted custody drill, not universal release qualification, signing,
registry publication, hosted rotation or lost-Secret recovery.

Evidence labels: `target/yt096-hosted-build-20261028-v044/{completion,hosted-build-summary,hosted-final-results,native-push-result,post-push-observation-correction}.json`
and `target/yt096-hosted-verifier-20261028/{completion,verifier-final}.json` plus
`HANDOFF`. Under `$CUSTODY_ROOT`, retain `retention-receipt.json`,
`retention-index.json`, `receipts/hosted-readback-receipt.json`,
`receipts/correspondence-verification.json` and
`metadata/environment-approval-observation.json`. These are record labels, not
tracked private evidence or machine-specific paths.

| Original stage | Completed checkpoint |
|---|---|
| A — Source delivery | Authorized integration delivery and PR13 completed. |
| B — CI, Merge, freeze | Actual squash source/tree selected; master CI accepted. |
| C — Environment/Secret | Reviewed settings and matching Secret installed; actual protected approval observed. |
| D — Real tag/build | One-use guarded source-only tag Push and build attempt 1 completed. |
| E — Later verifier | Separately authorized attempt 1 and actual owner approval completed; full readback PASS. |
| F — Retention | Download, independent correspondence verification and persistent repository-external owner-host retention completed within 24 hours and before transport expiry. |

### Preserved stops and review history

Before the successful operation, the direct strict/builtin planned-input
`check --pre-push` assessment failed with exit 1: 13 findings in two historical
commits across six files, 23 commits and 1,760 file operations inspected. It was
not a native hook execution; no tag or hosted attempt existed at that stop.
The original findings remain unresolved accepted-existing debt. The
[existing-remote baseline design](YT-096-EXISTING-REMOTE-TAG-BASELINE.md)
retains the original design Reviewer P2 FAIL and focused followup PASS, zero
issues/required followups, correcting that distinction. Original plan and design
reviews retain their own scopes and do not accept this new checkpoint.

After the actual successful Push, the observation wrapper failed because it
expected hook output in stderr while Git returned it in stdout. The correction
inspected both streams of that same Push and observed remote refs. Push exit was
0; no repush, binding rearm or replacement hosted attempt occurred. The original
wrapper failure and correction remain retained.

The next unblockable YT-096 work is D6 identity/signing approval and the remaining
trust, registry and final-release gates. Reuse these accepted build/readback
records; no repeat build, verifier or earlier accepted checks is required by this
document update. D6 Pending; YT-096 In progress; YT-020 Blocked. No new Commit,
Push, signing or publication is authorized.

## Original reviewed plan snapshot (historical)

The remaining planning sections preserve the initial proposal and its original
Pending authorization packet. Their source/settings/authority observations apply
to that snapshot; completed stages above supersede them for the dated drill.

## Original facts and proposed source

These are read-only observations, not fresh CI or permanent remote guarantees.
Record labels: `source-findings.md`, `remote-summary.json` (observed
2026-10-28 00:06 UTC), `source-delivery-relationships.json`, `authority.json`.

| Binding | Observed value / consequence |
|---|---|
| Local implementation baseline | Clean `5753502b46a41bae96ccd3b9b0fa44c086e983ca`, tree `aca89e50ad9b4e0dda52d160ab09897e8aeb5eb2`; pre-merge source proposal only. |
| Remote `master` | `d7065bbfb5b733c418f65ef55324f4d0ce6ac4d6`, tree `cca0153c3e56bc5e13e651a4a4bb1469dc042cfe`; not an ancestor of local HEAD. |
| Original remote branch | `codex/yt201-package-preparation` at `3d63f0bcaa12d23ab075bfd63e784c8d50f39cc4`; exactly the same tree as `master` after PR12 squash. Preserve this branch/history. |
| Reviewed delivery scope | Local baseline is eight commits ahead of that branch; 28 changed paths cover YT-206 closeout and YT-096 tooling/docs, with no public framework/runtime/lock changes in that delta. |
| Repository/protection | Public, default `master`, squash-only; required `Rust 1.99 MSRV` and `Stable native, samples, and Chrome`, administrator enforcement enabled, force pushes disabled. Observed approving-review count 0 and strict status false are facts, not permission to waive review or change protection. |
| Hosted prerequisites | Environment list empty; target Environment/Secret-list and `v0.1.0` ref return 404. Default branch exposes only `private-rebuild.yml`, `qualification.yml`, `rust.yml`; release/verifier workflows are not yet there. |

Recommend a new integration branch `codex/yt096-hosted-custody` from the latest
`origin/master`, carrying only the reviewed local tree delta. Do not replay the
old squash history or rewrite the original branch. The 28-path count describes
the observed baseline; later approved documentation/source commits change it.
Refresh exact source/tree/delta bindings after any such commit or remote movement.

For the first Push, explicitly seek owner acceptance of the observed existing
public `master` as the historical scan boundary and authority to create the
remote integration ref at that existing commit. That GitHub write is itself
pending authorization. Then scan the staged index before any integration commit
and the exact outgoing `BASE..TIP` before its update Push using Commit Guard
`strict`/`builtin`, and run the native hook. Recheck final tip/branch/range after
any approved plan commit. No full-history bypass, ignore, force or history rewrite.

After PR review, ordinary required CI and a separately authorized squash Merge,
explicitly select the actual resulting `master` SHA/tree as `C_release`. Compare
it with reviewed content and operational inputs: both workflows, helpers, tool
pins, recipient/pins and six locks. Record required checks on that SHA or the
documented equivalent tree. No current tip or later commit is selected automatically.

## Proposed protected settings

At the original review, all settings and Secret installation below were **Pending / not granted**.

| Setting | Exact proposed value / review |
|---|---|
| Environment | `release-evidence-verify` |
| Required reviewer | Exactly User `bohewu`; `prevent_self_review=false` so the single owner can dispatch and approve. |
| Administrator bypass | Disabled; administrators remain a trusted policy root. |
| Deployment selection | `protected_branches=false`, `custom_branch_policies=true`; exactly one policy `{name: v0.1.0, type: tag}`. No branch or wildcard rule. |
| Secret | Environment-only `AGE_IDENTITY`, matching the existing enrolled identity and current source-pinned public recipient. Never a repository/build Secret. |
| Installation | After configuration review and separate authority, upload from the existing protected file via stdin; no chat/log/private argv, new key, deletion or rotation. Current work does not read that file. |

The API preflight checks documented reviewer/tag-policy fields; it does not prove
self-review or bypass behavior. Capture actual UI/manual configuration evidence
before Secret installation and before job approval. An unavailable/ambiguous API
or setting is a stop, not a reason to weaken the gate. GitHub documents the
[Environment configuration](https://docs.github.com/en/actions/how-tos/deploy/configure-and-manage-deployments/manage-environments)
and [protected Environment runtime boundary](https://docs.github.com/en/actions/concepts/workflows-and-actions/deployment-environments).

Build permissions remain `contents: read`; verifier permissions remain
`contents: read`, `actions: read`. No `id-token` or write permission. Unprivileged
preflight has no Environment/key. Compile and authenticated tool bootstrap occur
before the Secret step; repeat metadata checks before key use. The protected job
uses ephemeral directories0700/files0600 and cleanup of identity/plaintext on exit
and in the final step; cancellation/runner loss can prevent cleanup or capture.

## Sequential future authorization packet

At the original review, every row was **Pending / not granted**. Approval of one row grants no successor.
Freshly recheck remote source/protection, tag and workflow/settings before action.

| Stage | Concrete operation requiring owner authorization and evidence |
|---|---|
| A — Source delivery | Approve exact reviewed delta, latest `BASE`, integration branch and historical boundary; create remote ref at that existing `BASE`; separately authorized guarded local commit/update Push and PR. Preserve original branch/history. No action now. |
| B — CI, Merge, freeze | Let ordinary required PR CI run after authorized delivery; review actual results/content, then separately authorize squash Merge. Select and record actual post-merge `C_release` SHA/tree, checks, reviewed-tree/operational-input comparison and default-branch workflow presence. |
| C — Environment/Secret | Approve exact settings above, record API plus UI/manual review, then separately authorize installation of the matching existing `AGE_IDENTITY` by protected-file stdin. No key read/upload before authority. |
| D — Real tag/build | Freshly confirm remote tag absence; authorize immutable unsigned source-only `v0.1.0` creation/push at `C_release` and exactly one `release.yml` build attempt: ref `refs/tags/v0.1.0`, `expected_commit=C_release`, `cohort=sixteen`, 45-minute job cap. |
| E — Later verifier | Inspect D's actual outcome/artifact first. Separately authorize at most one `evidence-verify.yml` attempt at the same immutable ref with the recorded inputs below; preflight cap 5 minutes, protected job cap 15 minutes. Owner manually approves the Environment job after inspecting its identity/settings. |
| F — Retention | Authorize download, independent verification and retention outside Git of actual ciphertext and receipts, plus public bytes, within 24 hours and before seven-day expiry; record durable custody through final disposition. Hosted readback alone is insufficient. |

Current guards accept only the real sixteen-package train on
`refs/tags/v0.1.0`: no smoke/test tag, RC or branch path exists. **The first hosted
test consumes the real immutable tag slot.** A source-changing failure permits no
move, deletion or recreation of that tag, silent RC support or automatic tag bump.
No GitHub Release or package publication accompanies this source-only tag.

Workflow dispatch must exist on the default branch; see GitHub's
[manual dispatch requirements](https://docs.github.com/en/actions/how-tos/manage-workflow-runs/manually-run-a-workflow).
Require source HEAD = event SHA = workflow SHA = `expected_commit` = peeled tag
target, clean source and ancestry on `origin/master`, with exact workflow/ref.
The 65-minute total is configured runner caps only, excluding queue and approval
wait. No automatic successor, rerun or retry is authorized.

Record D's actual URL, run ID/attempt, ref/event, source/workflow SHA, times,
conclusion/native exit, artifact ID/name and upload ZIP digest. E inputs are
`expected_commit`, `build_run`, `build_attempt`, `artifact_id`, `artifact_sha256`;
the last value is the ZIP SHA-256 **without** the `sha256:` prefix. Bind the
original completed manual build and exact repository/source/run/attempt/artifact,
name, size, digest, expiry and safe ZIP members before any key use. Record E's
actual run/attempt, inputs, human approval and safe receipt. A failed-build
ciphertext readback can be separately authorized but remains incomplete.

## Success, disclosure and stop rules

Complete product custody requires `PREPARATION_SUCCESS`, original native exit 0,
`complete=true`, all 33 phases, sixteen fresh archives, source/export, recipe,
tools/OpenSSL and six-lock bindings, full manifest/inventory verification, closed
unsigned provenance and independently verified exact public checksums. Retain the
actual ciphertext, transport receipt and actual readback/retention receipt outside
the frozen assembly. A green verifier with `FAILED_INCOMPLETE` is failure-evidence
readback, never preparation success. Missing/cancelled capture may be `NOT SAVED`.

Public output remains eighteen package files: sixteen archives, unsigned provenance
and checksum manifest. Adopted D2 additionally permits ciphertext and safe
source/run/status/size/digest metadata/receipts. No raw private logs, plaintext,
identity, environment dump or whole-runner capture. Seven-day transport and hosted
readback prove no durable retention; keep successful and failed records outside
Git through owner disposition, including tooling/bindings needed for old ciphertext.

STOP on source/workflow/tag identity, protection/API, artifact/expiry/ZIP, key,
authentication, inventory or manifest failure. Preserve original failure and
available records; no plaintext fallback, automatic retry or changed protection.
Any later attempt needs fresh bounded owner authority and its own record.

This is first hosted readback with the current pin, not hosted rotation or
lost-Secret recovery. Reuse the accepted thirteen custody checks, nine rotation
tests and actual LOCAL rotation drill at their original sources/scopes; no test or
qualification replay is part of this plan. See the [custody guide](YT-096-AGE-CUSTODY-INTEGRATION.md),
[rotation record](YT-096-KEY-ROTATION-DRILL.md) and [CI integration guide](YT-096-RELEASE-CI-INTEGRATION.md).

Original next unblockable YT-096 work: owner decision on the independently reviewed
staged authorization packet after refreshed bindings. No source,
settings or execution approval is inferred. D6 is not a prerequisite to a separately
authorized unsigned build; hosted green grants neither D6 nor release approval.
Identity issuance, OIDC, signing and publication remain separate and unauthorized.
