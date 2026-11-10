# Yetlet CLI 0.3.1 — hosted preparation record (YT-250)

Date: 2026-11-10. **Phase 1 PASS: original unsigned build and protected
original-evidence custody readback. T5 remains In progress at the separate
signer-source S gate. No signing, independent V or publication has executed.**

## Authority, source and delivery gates

The Owner-authorized preparation follows the
[CLI 0.3.1 preparation record](CLI-031-RELEASE-PREPARATION.md) and the existing
[single-publication SOP](RELEASE-PUBLICATION.md#each-version). This checkpoint
covers exactly schema 3 / `cli`, one `yetlet-cli` package. SDK alpha.7 remains a
consumed dependency and immutable embedded snapshot; no SDK is republished.
Formal GitHub/crates.io publication is excluded. Published distribution and
Reader remain SDK alpha.7 / CLI 0.3.0.

| Binding | Actual accepted input |
|---|---|
| Product source C | `f489a089ec2b814e830942318f84b0e45b0a67bd` |
| Reviewed tree | `413cfdb7ddac0c8f1cf32e92a54f38b83fc5ad35` |
| Immutable lightweight tag | `yetlet-cli-v0.3.1`; parent live readback exactly C |
| Source delivery | [PR #77](https://github.com/bohewu/yetlet/pull/77), exact head `556be2695837035c9e4693615552ac49bbf3a86e`; all three workflows PASS on attempt 1, merge preserves the reviewed tree |
| Exact-C master CI | [Rust 42828348950](https://github.com/bohewu/yetlet/actions/runs/42828348950), [CodeQL 42828348985](https://github.com/bohewu/yetlet/actions/runs/42828348985), [Docs/Pages 42828349071](https://github.com/bohewu/yetlet/actions/runs/42828349071), all PASS on attempt 1 |
| Independent source/release-automation review | Re-review PASS, zero issues/required followups; 28 frozen file bindings retained |
| Guarded delivery | Parent records strict/builtin staged and outgoing checks and native hooks; immutable tag uses one-use accepted existing remote history, without a historical-cleanliness claim |
| Reader readback | Parent verifies all 15 live Pages files byte-equal to the C export; eight HTML pages have one matching edition row and full C source URL; remote root README remains published SDK alpha.7 / CLI 0.3.0 |

The two reviewed docs-site masthead files remain included exactly. The main
checkout and research worktree are preserved. This phase performs no Git
operation or permission change; the parent owns delivery and tag operations.

## Original hosted build and package correspondence

| Binding | Actual result |
|---|---|
| Build / attempt | [42830865959 / 1](https://github.com/bohewu/yetlet/actions/runs/42830865959), SUCCESS |
| Public artifact | `unsigned-preparation-42830865959-1`, ID `13771622095`, 547901 bytes |
| Public artifact ZIP SHA256 | `53b23623bc7908c3f8c320e546d9e7b3d9665f1a862e44359935f899370bba43` |
| Encrypted private artifact | ID `13770498928`, 56505551 bytes |
| Encrypted artifact ZIP SHA256 | `c31f60c6caefd6828f9cb43f836230e650e8751f40fc2f19726d6cf42739f444` |
| Hosted release archive | `yetlet-cli-0.3.1.crate`, 545352 bytes |
| Hosted archive SHA256 | `305f67eac6c302ff9efeddc6d0bb20dce2508c4ddf899fe7727545f5e6168f9d` |
| Original unsigned provenance | 5557 bytes, SHA256 `77df62321946f38dee5ce3dcf1e4a707158b56322721c523727b5f3dd3ec5f8f` |
| Artifact expiry | 2026-11-17; owner-host retention is not offsite backup |

One `release.yml` invocation used the exact core record at the immutable tag.
Normal source/version/documentation checks, verified packaging and encrypted
transport passed. The downloaded public ZIP contains exactly the CLI archive,
original unsigned provenance and public checksums; the existing checksum verifier
passes. Provenance identifies exactly the hosted archive above, C, the original
build/attempt and pinned inputs. Rust 1.99.0, static OpenSSL 3.5.9 and its archive
SHA256 `603f5602e2eef00d77fbd429d34dcd5822bb301757a1bc9cdb24c670f1eb859a`
remain the trusted build inputs. The public recipient file SHA256 is
`b861a3c7e2abf3caf4c7071c045f6114704db94e8cc39ce010e7fa10711ff506`;
authenticated age 1.3.2 binary SHA256 is
`eb7dd1b518f0a307c99cd97782623c5321da049154b04acd2d98d21aa7bc9b2c`.
Pinned workflow/bootstrap/source hashes and original logs are retained.

The hosted archive is the selected future release/signing/public-plan subject.
The separate clean local T4 archive SHA256 remains
`15803c419cc5c9d6ea7be3239cf09a47adb336ee8100d39da17fedb199099493`.
All 162 hosted files are byte-equal to the common files in the 163-file local
archive. Only the local Git-aware `.cargo_vcs_info.json` is absent from the
non-Git hosted staging representation. Full per-member comparisons are retained;
the two archive digests are not interchangeable. No archive was repacked or
rebuilt to force digest equality.

Identical compiled source, dependency lock, resources and template bytes support
reuse of T4's actual archive-installed binary/version, six registry-only SDK
consumer creations, exact pins/checksums/locks/receipt provenance, Native Style
SSR/CSS and Browser Style Vite/Wasm/Chrome/CSP results. This is source/behavior
correspondence, not a claim that the hosted archive itself was installed locally
or that unpublished CLI 0.3.1 was installed from Registry. Refreshed default SDK
was actually alpha.7; future SDK API/compiler compatibility is not qualified.
Windows Style remains paused/unverified, and unrelated Cloud/Config/standalone
Tailwind opt-ins were not replayed.

## Original protected custody and durable retention

Original custody readback
[42831991715 / 1](https://github.com/bohewu/yetlet/actions/runs/42831991715)
completed SUCCESS, including isolated cleanup. Safe receipt artifact ID
`13771298253`, 687 bytes, has ZIP SHA256
`7a567cd295ca5010899c7e9e61f4aa79af9c7d6ec5f57347368f2177c45ca8a8`.
Its complete, decryption/inventory and private-manifest checks are true;
original exit is 0 and status is `PREPARATION_SUCCESS`. It binds C/tag,
build/attempt, recipient and actual ciphertext. Ciphertext SHA256 is
`6bf026263388f556d8f2893bfe95396079b1dfa84ddd6fe8e98a4030fbefe54b`;
original custody inventory SHA256 is
`36a104dd5498511a42be87d253c7568b1fd640ecca59f3b06231514adceaf8e5`.
The safe receipt is not builder authentication, durable custody or release approval.

The parent rolled the existing Environment's single tag policy from CLI 0.3.0
to exactly `yetlet-cli-v0.3.1`, after confirming no active original verifier.
Environment ID `25525950662` and policy ID `64604698` are unchanged. Existing
automation rules, disabled admin bypass, tag-only ref mode and Secret remain
unchanged; no wildcard, additional policy or Runner permission is introduced.
The canonical metadata check passed before the single custody dispatch.
No formal private key or decrypted original private evidence was downloaded locally.

Raw input snapshots, actual run/artifact metadata, safe receipts, downloaded
public outputs, comparison and logs are retained under
`artifacts/verification/cli031-latest-sdk-20261110/T5/`. The original encrypted
ZIP/transport and retained public/input/receipt copies are in the owner-local
external evidence run `cli031-latest-sdk-20261110/T5-original-build`, below
`~/.local/share/yetlet-release-evidence/runs/`. Preserve the seven-day expiry and
the lack of an off-host backup guarantee. One direct local observation initially
asserted terminal success from an in-progress post-checkout snapshot; that snapshot
and diagnostic remain retained. A fresh terminal snapshot passed, without a
workflow retry, product repair or altered archive.

## Separate S dossier and remaining runtime inputs

This document and its linked preparation evidence form the proposed docs-only
S dossier. S is **unselected**, must differ from C under
[ADR 0009](adr/0009-release-build-and-signer-identities.md), and requires the
parent's independent review, strict/builtin guarded delivery, exact-head PR CI
and post-merge master CI. No future commit hash is guessed or embedded in itself.
Keep master at the accepted S through prepare/sign; V may equal S only when its
reviewed verifier source and live master remain unchanged.

The retained `signer-record-known-fields.json` is a partial snapshot, not a
complete dispatchable record. It preserves the accepted core and exact encrypted
build identity, adds custody verifier `42831991715/1` / Environment `25525950662`,
and binds the original public provenance SHA256/size above. After S acceptance,
complete the existing record with actual `signer_commit` and the exact new private
Draft `release_id`; validate it at stage `complete` with the existing
`release_record.py`. Do not change any accepted earlier snapshot or reuse the
CLI 0.3.0 Draft/record.

The retained `phase2-input-templates.json` mirrors the existing complete-record,
prepare, sign and result request fields. Unknown identities/output hashes are
explicitly null; these templates are not valid dispatch inputs or selected S/V.
Populate them only from accepted S and actual subsequent receipts, then validate
with existing scripts before any authorized dispatch.

The next phase uses only existing input contracts:

1. Select an explicit bounded flat qualification closure with original sources,
   hashes and scope for all six existing roles: qualification, security,
   inventory, support, review and failures. Include the source review/CI gates,
   T4 actual consumers/limits/cleanup, the original build/custody inputs and
   retained failures; do not reclassify reused results as a new platform run.
   Pack/encrypt with existing `release-sign.py pack` and the pinned public
   recipient. A private Draft staging asset is a handoff only, not publication.
2. Generate the existing encrypted short-lived qualification handoff immediately
   before prepare. Bind actual qualification asset ID, ZIP SHA256, manifest
   SHA256, encrypted handoff SHA256 and expiry, with mode `prepare` and exact S.
   Validate the complete release record and request, then perform the normal
   no-OIDC prepare at S. No prepare or intake job has run at this checkpoint.
3. Retain actual frozen run/attempt/artifact/digest/receipt and the two raw payload
   hashes. Sign once under unchanged `reviewed-automation` policy using those
   exact inputs, then run the separate no-OIDC independent V verifier. Neither
   signature nor verifier success is inferred from this original custody receipt.
4. Only successful independent verification can supply the five-file CLI public
   allowlist, exact notes/identity and one-package read-only registry plan. Every
   archive subject must use hosted SHA256 `305f67eac6c302ff9efeddc6d0bb20dce2508c4ddf899fe7727545f5e6168f9d`.
   Stop before `release-publish`, formal GitHub publication or crates.io upload.

At this dated checkpoint S review/delivery/CI, Draft/qualification handoff,
prepare/freeze, signing, independent V and final prepublication plans remain
pending. T5 and YT-250 remain In progress. No owned local server, Chrome session,
watcher or build remains. Both hosted jobs report terminal success with cleanup;
retained caches/evidence were not deleted. Later verified publication would
require switching the accepted CLI publication authority, followed by README,
Reader/Pages synchronization and actual consumer/native/Wasm/deployment/live
readback before closeout; this preparation grants no promotion.
