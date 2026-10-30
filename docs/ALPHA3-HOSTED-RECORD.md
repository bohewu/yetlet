# Alpha.3 — Hosted build, readback and signing preparation

Date: 2026-10-30 (Asia/Taipei). Status: build/readback and owner-local retention
PASS at the original checkpoint. The later signing/publication completion is
recorded below; the original prepare grant retains its historical scope.
This is the execution record after the [local preflight](ALPHA3-VERSION-PREFLIGHT.md).
Original local acceptance, optional harness failure and alpha.2 failed
qualification remain unchanged. Alpha.1 D6/publication stay Complete.

## Actual immutable source and executions

| Binding | Actual value / result |
|---|---|
| Product source C | `557ab76e03b619e959eae7529dbe03317fe1f952` |
| Product tree | `c890b4c9215b0dcc2741caf97d27770062dde6e1` |
| Version / cohort | `0.9.0-alpha.3`; the same sixteen Experimental libraries |
| Delivery | [PR23](https://github.com/bohewu/yetlet/pull/23) squash Merge; tree matches the reviewed delivery head |
| Required PR CI | [40904870059/1](https://github.com/bohewu/yetlet/actions/runs/40904870059): MSRV and Stable PASS |
| Automatic master CI | [40906704945/1](https://github.com/bohewu/yetlet/actions/runs/40906704945): both required jobs PASS |
| Immutable unsigned tag | `v0.9.0-alpha.3`, exactly at C; strict/builtin and native pre-push PASS |
| Sixteen-package build | [40906989456/1](https://github.com/bohewu/yetlet/actions/runs/40906989456): SUCCESS, original attempt only |
| Protected evidence readback | [40908551139/1](https://github.com/bohewu/yetlet/actions/runs/40908551139): preflight and verify SUCCESS |
| Actual manual gate | `bohewu` approved `release-evidence-verify`, Environment ID25525950662; no pending deployment remains |
| Cleanup | Isolated readback cleanup SUCCESS; no formal private key or private plaintext read locally |

Provenance and the actual serialized transport receipt agree on C, workflow SHA,
`refs/tags/v0.9.0-alpha.3`, build40906989456 and attempt1. This closes the alpha.2
version-binding defect; green build alone was insufficient. Protected readback
verifies the complete private manifest, inventory and all entries, then removes
temporary plaintext. It does not sign or approve release.

## Original artifact and retention identities

| Retained bytes | Identity |
|---|---|
| Encrypted build ZIP | artifact12912576748; 56,454,319 bytes; SHA-256 `fbe484e072232b7a4627b20a6524f009da609e56ceba0f178c6f2251b78adec3` |
| Unsigned-public ZIP | artifact12912471850; 326,976 bytes; SHA-256 `6a57baaadef48bf53d3dc194f3e9a3686e35be18c971696def03aef3002ffde7` |
| Safe readback ZIP | artifact12937882753; 686 bytes; SHA-256 `2f81380d2ecbc8e7f5076902f081fde1c36981b84a3a9478845c7d0aa891011b` |
| Encrypted private evidence | SHA-256 `9db0a17057d06263c15de46c6096013eabdd43dc8fed616080173532abc0b3ae` |
| Custody inventory | SHA-256 `2720dfbb0b3c1438b6ef4d4ee8147f993c3eb51b40fd69260a6ef81712a9968c` |
| Safe hosted readback receipt | SHA-256 `211d5d3d791f95dadbb3682e6b23b9813f6f5e60ab08f50e9502636afc0e2561` |
| Original raw public provenance | 9,158 bytes; SHA-256 `4192b2f57fbe44080dd967b9d58cc53273948cb4620d623417ae92182a666434` |
| Owner-local retention index | 70 files / 130,668,783 indexed bytes; SHA-256 `d35aa34a1b855277cd2c48fbcb6e5ed4a9745eea5b26653d413b16f13d3cc170` |

Repository-external custody retains original ZIPs/ciphertext, sixteen packages,
provenance/checksum, source/recipe/lock correspondence, actual runs/jobs/approval
and safe receipt. Index and separate receipt were flushed and reread. No offsite
backup or indefinite availability is claimed. Build artifacts expire 2026-11-05
UTC; local retention does not authorize resurrecting an expired Actions artifact.

The evidence Environment has exactly the alpha.3 tag policy; original reviewer,
no-admin-bypass and `AGE_IDENTITY` name/timestamps remain. Other Environments were
not modified. Alpha.1/alpha.2 tags remain at original sources. The alpha.2 failed
retention index still binds 42 original files, SHA-256
`e1284ac006287accda9816d9e2a77e64e5e4ac825eb4425751a059bc963d106b`.

Optional disposable-key local readback remains FAILED/UNVERIFIED: its harness
omitted the encryption recipient literal. Actual formal hosted PASS is separate
evidence and does not rewrite that failure. Accepted product/platform/browser/
security/load suites were not manually replayed.

## Authorized next batch: stop after prepare

The owner authorizes this record and linked status documents for independent
review, then strict/builtin Commit/Push/PR. Required CI must pass on the exact head
before squash Merge. Verify actual merge SHA/tree; select that real follow-up
commit as S, distinct from C under [ADR0009](adr/0009-release-build-and-signer-identities.md).
Keep C/tag/product bytes fixed. No empty commit, retag, rebuild, third-party
upgrade or runtime/workflow change is proposed.

Before normal authorized branch Push, archive the consumed alpha.3 tag Guard
binding through supported fingerprint-bound retirement. Preserve exact used
bytes/tag/history; no rearming, hook change or strict/builtin weakening. Use
current remote master C as the existing-history boundary of this follow-up PR.

After Merge, verify protected master equals actual S; hold further master merges
through prepare and any later separately approved signing. Create one unpublished
alpha.3 prerelease Draft targeting C, preserving the published alpha.1 Release.
Capture its real ID; unknown or floating identities are not inputs.

Complete the existing strict schema-1 record from actual values only:

- Core: schema1, version `0.9.0-alpha.3`, C and cohort `sixteen`.
- Build: run40906989456, attempt1, artifact12912576748 and original ZIP SHA above.
- Custody verifier: run40908551139, attempt1, Environment ID25525950662.
- Signer: actual S; provenance: unchanged raw public SHA/size above.
- Release: actual new unpublished Draft ID. No invented fields or approval flags.

The qualification selection reuses original native/browser/security/inventory
records with each source/limitation, plus actual alpha.3 package/consumer/review/
CI/build/readback/retention records and failures. This inventories the selected
set, not every global raw log or complete compiled/system inventory. Remote
Windows originals remain remote; their index/denial does not claim a local ZIP
or bypass a transfer refusal. The protected helper verifies and restores the
original encrypted build evidence, complete private files/extras and OpenSSL source.

Retain this document review separately and add its immutable acceptance receipt
to qualification. Use existing checksums/qualification/pack interfaces to create
the complete flat manifest and encrypt to the unchanged public recipient. Keep
the formal private key in the protected job. Final containing manifest, future
prepare receipts and later signing approval cannot include themselves.

Upload only encrypted `qualification-inputs.zip` to the new Draft. Immediately
before dispatch, generate the existing encrypted short-lived capability bound
to actual asset ID/digests, S and complete record. Never put owner tokens,
plaintext capability URLs or private evidence into Actions inputs, tracked
documents or public attachments.

One `release-sign.yml` **prepare** is authorized: preflight5/protected30 minutes;
owner personally approves `release-sign`; no OIDC/signing permission. Recheck
build/readback, protected master/S, Environment/Secret metadata, expiry and
encrypted transfer before key access. Preserve failure/uncertainty and stop;
no automatic retry or capability renewal.

Completion requires overall prepare and cleanup SUCCESS, exact frozen artifact/
receipt correspondence, complete-record digest and both raw payload SHA-256
values, then checksummed repository-external retention. Delete the new temporary
encrypted Draft asset only after confirmed successful prepare and durable
retention; keep original metadata and local ciphertext. On failure retain
evidence for explicit disposition. Draft stays unpublished; no public assets
are staged in this batch.

Actual S/Draft/run/artifact IDs, final manifest and both signing hashes remain
Pending until operations produce them; retain each snapshot outside the source.
Do not invent a self-referential S in this commit. Document review accepts
recorded results and prepare readiness, not future execution. Signing, positive
independent result verification, output acceptance, publication and crates.io
remain outside this grant. Alpha.1 D6 stays Complete; no alpha.3 signature exists.

## Signing, independent verification and publication — 2026-10-30

The owner subsequently authorizes the exact prepared inputs and then accepts
the actual verified outputs/public plan. The prior Pending statements above
remain historical; no original receipt or failed result is rewritten.

| Binding | Actual result |
|---|---|
| Product C / unchanged tag | `557ab76e03b619e959eae7529dbe03317fe1f952` / `v0.9.0-alpha.3` |
| Signer S / verifier V | `30baf2504a14859223416bccdc2d05755ae000f6`; PR24 squash Merge, reviewed tree `7a46e5dd0710fde1b1bf83e33ecacbdc8e8390fa` |
| Prepare | [40981466538/1](https://github.com/bohewu/yetlet/actions/runs/40981466538), SUCCESS including cleanup |
| Public raw payload | SHA-256 `4192b2f57fbe44080dd967b9d58cc53273948cb4620d623417ae92182a666434` |
| Full private raw manifest | SHA-256 `d5374f1bf0e34ebe0728a8b127398d8b83f4332a21877aaf79d148bad9319069` |
| Formal signing | [40982672108/1](https://github.com/bohewu/yetlet/actions/runs/40982672108), both signatures/verification/cleanup PASS |
| Protected independent result verification | [40983028507/1](https://github.com/bohewu/yetlet/actions/runs/40983028507), both bundles, all16 subjects/all112 private entries and cleanup PASS |
| Actual manual approvals | Owner `bohewu`, `release-sign` and `release-result-verify`; original approvals retained |
| Owner-local signing-output index | 68 files; SHA-256 `1debb47e1b463e689d099998670d31697bf0910becc456f006011fcba96242a3` |
| Final publication | Existing Release428536609, [Experimental alpha.3 prerelease](https://github.com/bohewu/yetlet/releases/tag/v0.9.0-alpha.3), published 2026-10-30 09:17:18 +08:00; not Latest |
| Public delivery | Exact20 approved files, authenticated and anonymous complete-byte readback PASS; no private attachment or crates.io upload |
| Publication custody index | 97 files; SHA-256 `d4ebc76e961cc5b0ca940f7ca7b77072f3c4c45b73a4b6a50f63081514104a90` |

The first native Python upload stage failed before any asset was created; exact
Draft/tag and zero-asset reads passed. Its generic error did not retain provider
HTTP details, so no specific HTTP status or root cause is claimed. After the
owner authorizes autonomous transport recovery, `gh api` uploads the same20 files
once each. Original closed-plan, digest, readback and publication checks remain;
one publication PATCH completes the existing Draft. The initial FAIL and later
success receipts remain separate. Neither signing nor publication was repeated.

The owner accepted the actual results and activated the exact acceptance summary
before publication. Original `SIGNING-INPUTS` and result receipts retain their
then-Pending fields; new authority/outcome receipts live outside the frozen
assembly. No formal private key or private plaintext was accessed locally.
Earlier alpha.1/alpha.2 and optional local failures remain unchanged. Owner-local
retention is not offsite or indefinite-availability proof.
