# Yetlet CLI 0.3.1 — hosted preparation record (YT-250)

Date: 2026-11-10. **Prepublication preparation PASS: original build/custody,
separate reviewed S, frozen-input signing and independent V completed. The
five-file public allowlist and single-package registry plan pass read-only gates.
Formal GitHub/crates.io publication remains held.**

The phase-1 checkpoint below retains its original pending S/signing state; the
later phase-2 section records actual completion without changing C or its bytes.

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

## Separate S dossier and remaining runtime inputs — phase-1 checkpoint

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


## Phase 2 — fixed S, signing and independent V

Parent-delivered [PR #78](https://github.com/bohewu/yetlet/pull/78) selects
S=V `40e2c9cd4dda831e60fca145a04e4ef748d0c048`, tree
`6abe4cbcc5197356b53d4492d64132a16e8dc846`, distinct from C. Independent
S-dossier review PASS has zero issues/required followups. Strict/builtin staged,
outgoing and native-hook checks passed. Exact PR head
`6faead5841ea1e5830b973a2dc4db5007ff34b34` passed all three workflows on attempt 1;
master [Rust 42834256035](https://github.com/bohewu/yetlet/actions/runs/42834256035),
[CodeQL 42834256017](https://github.com/bohewu/yetlet/actions/runs/42834256017) and
[Docs/Pages 42834256037](https://github.com/bohewu/yetlet/actions/runs/42834256037)
all passed on attempt 1. Parent readback matches all 15 live files to the S
export; all eight HTML pages keep one edition row, full S source link and
published/Reader SDK alpha.7 / CLI 0.3.0. The earlier C readback remains retained.
No source/workflow or package bytes changed for the five-document S delivery.

The actual private Draft is Release ID `440475335`, tag `yetlet-cli-v0.3.1`,
target C, Experimental pre-release metadata, and remains unpublished. The complete
schema3/cli record fixes that ID and S with the unchanged accepted phase-1
core/build/custody/provenance inputs. Its canonical SHA256 is
`8592f9eebca25f687e4f62901871d63ef34083bdeae5918a3703ba650d9f0021`.
The explicit qualification closure has 35 evidence entries covering the six
existing roles, with original source/scope/hash bindings; no new agent review,
platform run, dependency audit or validation framework is inferred.

| Stage | Actual result and retained identity |
|---|---|
| Prepare/freeze | [42835822093/1](https://github.com/bohewu/yetlet/actions/runs/42835822093), SUCCESS; no-OIDC intake/prepare and cleanup; frozen artifact `13774830204`, ZIP SHA256 `3ba6e601d16029697b775abd28bb9a663c86a28c9636121598fa1e399e5905da` |
| Qualification intake | Artifact `13773073754`, ZIP SHA256 `9cb6572eab6bd24aba3e9016cb3b73b2b60ad50d428f0009235adf6fada1f94a`; canonical encrypted short-lived capability and ciphertext checks passed |
| Signing | [42836461667/1](https://github.com/bohewu/yetlet/actions/runs/42836461667), SUCCESS under standing `reviewed-automation`; actual results artifact `13774850862`, ZIP SHA256 `80e60f1d7ad4bd4922088bea12263b2fa6c4ff21cc4f265874c106c862d3f5c0` |
| Independent V | [42836870230/1](https://github.com/bohewu/yetlet/actions/runs/42836870230), SUCCESS at V=S; separate no-OIDC worker verifies one actual CLI subject and all 52 complete private-manifest entries, including both real bundles and cleanup |
| V safe receipt | Artifact `13773312679`, ZIP SHA256 `b78f1f87aa24e101e524752ecfa65501158751b40daa009f0015f86f9a7cb659` |
| Verified public export | Artifact `13772833854`, ZIP SHA256 `00e75cf092b0b9b9cefe66915cc0a302c95b0673b14b233b9a8a7c66fc3b0735` |

The exact original provenance payload remains SHA256
`77df62321946f38dee5ce3dcf1e4a707158b56322721c523727b5f3dd3ec5f8f`.
The final private signing manifest is SHA256
`28f8c2ba8fd8503a9f448e5441bdf6550b1d061c2d1197ef92eb71d6cfc23e04`.
Frozen receipt SHA256 is
`99bf67ed658355f5bc7c034525a045dad761ca92e8a72949cbf7c89a4950c613`;
actual signing execution SHA256 is
`f47b83faaf51a0343dc594b4d621bcc74410e08ca69a0723a095ee14676e489b`.
V's original receipt SHA256 is
`ecaa98151a662097639170f127b1ba7c577ed8049e3c0547024b1486d2e04ec1`.
The detached private manifest bundle remains encrypted in owner custody; no formal
private identity or original decrypted private evidence was downloaded locally.
Actual certificate/issuer/SAN/workflow-S and complete signature checks are supplied
by the successful independent verifier, not inferred from same-job signing.

## Read-only publication packet and retained boundary

The existing registry planner and public-input/metadata gates pass against the
actual V export and original hosted archive. The five-file preview allowlist is:

| File | Bytes | SHA256 |
|---|---:|---|
| `yetlet-cli-0.3.1.crate` | 545352 | `305f67eac6c302ff9efeddc6d0bb20dce2508c4ddf899fe7727545f5e6168f9d` |
| `PROVENANCE-UNSIGNED.json` | 5557 | `77df62321946f38dee5ce3dcf1e4a707158b56322721c523727b5f3dd3ec5f8f` |
| `RELEASE-CHECKSUMS.json` | 430 | `3bd6bbcd1c36dd3b5079105c9d58dc658cd820a278e5799adfb3479977795e34` |
| `provenance.sigstore.json` | 17139 | `4ea6b39256aee05439cd9abbfd0202240630fe3de8e318ef4e93593d38512d0e` |
| `SIGNING-APPROVAL.json` | 839 | `53ca09e684aa942e96952863a1ec51d1c2e9b408a4c3d19bb293367cc4aedefe` |

The single-package registry plan SHA256 is
`97be2e7494121d7ec2973ad583943fde71b897ebf7357946f25246a767ae7b51`;
its package is exactly `yetlet-cli 0.3.1`, using the unchanged hosted archive.
Preview name is `Yetlet CLI 0.3.1 Experimental`; selected notes SHA256 is
`8da23b56902e5e1739bf1fc281592c69af4307aa1c1c6263dca35052ffa56edf`.
The retained preview request at planning P=S has SHA256
`afba4aaa4fc444902cfc7aba130ae2bbe5d13585406ac914b4ee0063a5a6c299`.
The reduced approval file is the existing private preview calculation, not an
Owner publication grant. No `release-publish` workflow, including preflight,
was dispatched. No public attachment was staged, GitHub Release published or
crate uploaded; the private Draft has zero assets after qualification cleanup.

The notes-only Draft metadata PATCH succeeded but changed GitHub's Draft tag to
an `untagged-*` value. The read-only exact-ID gate caught the mismatch before
any public asset or publication operation. Original response/failure and partial
plan remain retained. The parent admitted one task-local metadata artifact repair
and applied an explicit tag/C/private/pre-release/name/body PATCH; exact readback
passed with zero assets and the immutable Git tag unchanged. Only the existing
read-only metadata/public-input gate was repeated afterward, PASS. No failed
write, hosted job, build, signing or V was retried; no source/signature/archive
changed. The separate pending-job metadata display observation is also retained.

The proposed final documentation may be delivered through the parent's normal
whole-Pipeline review and guarded docs-only PR/CI. A later reviewed publication
source P may be that actual final commit F; this does not select a new S or V.
Preserve the original P=S preview unchanged, then derive a new publication
request and joined public plan for actual P after its CI. C, S, V, complete
record, original builds, frozen payloads and signatures stay immutable. No
self-referential commit hash is required in this dossier.

All original phase-2 inputs, raw logs, safe receipts, actual bundles, ciphertext,
verified exports and preview plans are retained under the T5 `phase2` evidence
subdirectory and owner-local external `T5-phase2` evidence directory in the same
`cli031-latest-sdk-20261110` run. Artifacts begin expiring on 2026-11-17; owner-host
retention does not prove off-host backup. All applicable hosted jobs completed
successfully with cleanup; mode-excluded jobs are skipped. The local qualification
plaintext root and short-lived qualification Draft asset were removed only after
verified frozen retention. No owned local server/browser/watcher/build remains.
The parent removed the round-owned Rust/tool caches after final tests. The
retained `artifacts/verification/cli031-latest-sdk-20261110/cleanup-result.json`
records the exact scope and preservation of accepted artifact hashes.

T5 and YT-250 are complete at the requested prepublication endpoint following
independent whole-run Pipeline acceptance. Guarded documentation delivery is
recorded separately. Formal publication remains a
separate Owner boundary. Published authority and Reader remain SDK alpha.7 /
CLI 0.3.0. After any later verified CLI publication, switch the accepted CLI
publication authority, synchronize README/Reader/Pages and perform actual
consumer/native/Wasm/deployment/live readback before closeout. The Owner's
additional rule to update `AGENTS.md` for successful PR merge source-branch
deletion and clean accumulated merged remote branches is explicitly deferred
until formal publication; no branch deletion or `AGENTS.md` edit occurs here.
