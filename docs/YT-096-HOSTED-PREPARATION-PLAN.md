# YT-096 — Minimum source, tag, hosted preparation and custody plan

Current packet (2026-10-28): [first real hosted custody drill plan](YT-096-HOSTED-DRILL-PLAN.md).
Independent plan review PASS (zero issues and required followups); source, settings/Secret and staged hosted
authorizations remain Pending. The dated preparation record below is preserved.

Date: 2026-10-27. **Custody scheme/D2 extension Adopted; local integration/readback PASS; independent review PASS (zero issues and required followups).**
D1–D5 remain Adopted (policy); D6 Pending; YT-096 In progress; YT-020 Blocked.
The owner authorizes bounded custody code integration and local encryption/readback
proof. A dedicated local identity has been created outside Git with private
permissions. Actual GitHub Environment/Secret setup, hosted execution, commit,
Push, PR, Merge, tag, identity issuance, signing and publication remain unapproved.

## Current decisions and remaining authority

| Decision | Recommended minimum | Status / consequence |
|---|---|---|
| Candidate source | Select the actual reviewed post-squash `master` commit as `C_release` only after approved custody changes, review, required CI and reviewed-tree comparison. | Pending selection. `28ea97e035b351cbbc362d834574f9c3a2e9ff03`, tree `6fb019f3873fac152459ed590896e92c639a6bf2`, is the preparation implementation baseline only. |
| Source tag | Retain the existing immutable, unsigned, source-only `v0.1.0` contract; no GitHub Release or crates publication. | Owner must separately approve creation/push. The first hosted test can consume this tag slot; no replacement after a source-changing failure. |
| Hosted verification | One explicitly authorized manual `release.yml` attempt at `refs/tags/v0.1.0`, `expected_commit=C_release`, `cohort=sixteen`; retain the existing 45-minute job cap and runner/tool boundaries. | Pending authority and actual run/attempt. No automatic retry, OIDC, signing or publication. |
| Complete private custody | Before source freeze, integrate pinned age1.3.2 encryption to one source-pinned native X25519 recipient, ciphertext-only Actions transport and verified retention. | Adopted, including D2 ciphertext/safe-metadata disclosure and future isolated, manually approved GitHub Environment verification. Actual settings/Secret installation and hosted execution remain unapproved. |

The [integration guide](YT-096-RELEASE-CI-INTEGRATION.md) describes current code;
the [D1–D5 packet](YT-096-D1-D5-RELEASE-TRUST-PLAN.md) retains adopted policy and
historical evidence. The [custody integration guide](YT-096-AGE-CUSTODY-INTEGRATION.md)
records the public recipient, commands and Environment boundary; no actual
`C_release` or durable hosted custody destination is established.
Read-only repository metadata observed today reports public visibility, default
`master` and squash-only merging. It proves no remote HEAD, tag, protection or CI result.

## Minimum order before the one attempt

1. Local custody integration, targeted fixtures and authenticated age LOCAL-manifest
   readback PASS at synthetic-fixture scope; complete product custody remains false.
   The scheme/disclosure are adopted; no new qualification matrix is authorized.
2. Independently review that delta before selecting source. Workflow, pinned
   recipient or tool changes alter the candidate. Separately authorize any commit,
   Push, PR and Merge; prior baseline commit authority does not carry forward.
3. Preserve required checks `Rust 1.99 MSRV` and `Stable native, samples, and Chrome`.
   After any authorized squash merge, select the actual `master` SHA/tree, record
   equivalence to reviewed PR content and successful required checks on that SHA
   or documented equivalent tree. Ordinary new PR CI remains necessary; accepted
   historical qualification does not substitute for it or require a full replay.
4. Freshly confirm remote `master`, protection/check settings, reviewed workflow,
   helper/tool/recipient bytes and default-branch workflow presence. GitHub requires
   `workflow_dispatch` to exist on the default branch ([manual workflow documentation](https://docs.github.com/en/actions/how-tos/manage-workflow-runs/manually-run-a-workflow)).
   Confirm remote tag availability/object/peeled target before any separately
   authorized creation/push. Local absence of `v0.1.0` proves no remote absence.
5. Require source HEAD = event SHA = workflow SHA = expected commit = peeled tag
   target, clean source and ancestry on `origin/master`; exact fixed workflow/ref
   must agree. The current guard accepts no RC or branch preparation. An RC model
   would need separate contract/guard/identity review; it is only a tradeoff here.
6. With source, tag, custody and one-attempt authority concrete, run the unsigned
   sixteen-package preparation once. Preserve actual URL, run ID/attempt, event,
   ref, source/workflow SHA, times, conclusion and native failures. No hosting now.
7. Independently download and verify public bytes and complete private records;
   record the outcome and actual custody receipt. A successful public artifact
   without verified private custody is incomplete and cannot support promotion.

## Adopted private custody boundary

The implementation adds encrypted capture and isolated verification; no hosted
capture or durable retention has occurred. Public Actions artifacts are downloadable
by logged-in readers with repository read access ([GitHub artifact access](https://docs.github.com/en/actions/how-tos/manage-workflow-runs/download-workflow-artifacts)).
Seven-day artifact retention is a transport cache, not durable private storage.

Build/capture receives only the pinned public recipient. Keep the private identity
outside Git, build jobs, logs and artifacts. The owner's latest choice permits its
transient use only in an isolated, manually approved protected Environment verifier;
this supersedes the earlier local-only CI policy. The existing local identity is
preserved for authorized local proof. Future Environment configuration/Secret
installation needs separate authority; no actual settings have been verified.

The task-scoped Linux amd64 bootstrap authenticated age1.3.2 archive/proof digests
and Sigsum0.13.1 with explicit `sigsum-generic-2025-1` two-of-three witness policy
before first age execution. Go1.27.3 trusts official HTTPS/published checksums and
Go module checksum-database roots; this is neither TUF nor independent binary
reproducibility. There is no global install or hosted CI qualification. The reviewed
[bootstrap script](../scripts/bootstrap-release-age.sh) pins that recipe without
floating versions or mutable root-file overrides.

The implementation must preserve these boundaries:

- Capture only an explicit allowlist of the private assembly and available known
  OpenSSL archive/log/result paths. Never capture all `RUNNER_TEMP`, Cargo home or
  an environment dump. Only the encrypted custody blob may be uploaded from this
  private capture; never upload the raw assembly or use a plaintext fallback.
- Preserve the existing public eighteen-file allowlist: sixteen archives,
  unsigned provenance and public checksum manifest. The adopted D2 extension permits ciphertext and safe
  type/size/digest/source/run/status metadata; complete plaintext private records
  remain private. It does not expand the eighteen-file public package allowlist.
- On success, keep the frozen private manifest and closed provenance unchanged.
  Put the encryption wrapper and receipt outside that assembly; bind source,
  run/attempt, status, private-manifest digest, ciphertext digest and explicitly
  inventoried OpenSSL extras. Avoid a manifest/provenance self-cycle.
- On failure, save available partial records and original exit status as
  `FAILED/incomplete`; do not invent a complete success manifest. Capture,
  encryption or upload failure means failed custody. Cancellation or runner loss
  may yield no archive: report `NOT SAVED`, without claiming guaranteed capture.
- Owner downloads within the planned 24-hour window and before seven-day expiry,
  decrypts locally or in the approved isolated verifier into a private directory
  outside Git (0700; files 0600), verifies
  ciphertext/source/run/attempt and every private-manifest entry plus known extras,
  and independently verifies exact public checksums, archives and closed provenance.
  Record actual download, decryption and full-manifest verification outside the
  frozen assembly; an upload success alone is not a durable-custody receipt.
- Retain failed and successful records until owner final release/candidate
  disposition; artifact TTL grants no local deletion authority. An optional
  backup adds no HA/SLO gate or mandatory Azure/KMS/service qualification.
  Encryption protects confidentiality/byte integrity, not builder authentication
  or release approval ([age documentation](https://github.com/FiloSottile/age)).

## Required outcome and stop rules

Record actual source/export, recipe/tools/OpenSSL and six lock identities, all
33 phase statuses (vendor, sixteen bootstrap collections, sixteen normal offline
locked package verifications), and sixteen final archive names/sizes/fresh digests.
Bind unsigned closed provenance, public exact checksums, complete private manifest
and actual custody receipt. Preserve legacy phase/guard/provenance proof and
original failures; bootstrap collection is never final package verification.
Reuse the original 27 focused checks and unchanged native/browser/CSP/security/
platform evidence at their original SHAs/scopes; no fresh full qualification is claimed.

Wrong source/workflow/tag identity or a remote tag at another target means STOP.
Source-changing, recipe, product, checksum or disclosure failure leaves the
candidate unqualified. Never move/delete/force/overwrite the tag or auto-bump it.
Infrastructure/timeout/cancellation or custody failure is incomplete/failed;
retain available records and stop. No raw fallback, signing promotion or automatic
hosted rerun. A later same-source retry needs explicit bounded authority and its
own attempt record; missing evidence stays missing.

## Open bindings and next step

Independent implementation review PASS, zero issues and required followups.
Pending actual bindings: authorized Environment protection/Secret installation and hosted verifier
qualification; durable retained ciphertext and receipt; post-custody reviewed
`master` SHA/tree/checks/protection; remote tag state/creation authority; one-run
authority; actual run/attempt, final bytes/manifests and verified download receipt.

Next unblockable YT-096 item: owner review/authorization of actual source selection
and protected Environment/Secret setup before hosted execution. Commit, Push, PR, Merge,
tag, settings/Secret operations and hosted dispatch retain separate future
permissions. D6 stays Pending for identity issuance/signing; later publication
remains separate. D6 is not a prerequisite to a separately authorized unsigned
hosted build. This plan completes no release gate and leaves YT-020 Blocked.
