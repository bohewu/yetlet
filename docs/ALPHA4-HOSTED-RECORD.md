# Alpha.4 — Hosted source packages and signing checkpoint

Date: 2026-10-30 (Asia/Taipei). **Build/readback, preparation, signing and
independent verification PASS. GitHub pre-release is public with twenty verified
assets; registry completion is one of sixteen.** Both failed attempts remain
preserved. Current local work corrects response parsing and prepares an explicit
registry-only continuation, without Git delivery, upload or re-signing.

## Immutable product source and actual evidence

| Binding | Actual value / scope |
|---|---|
| Product C | `c45e10c277ef8d418feb2fd6025b0c7b045640dc` |
| Tree | `4e61155382b9d53a4b7a0b998cdc4a9e6c583c25` |
| Version/cohort | `0.9.0-alpha.4`, sixteen Experimental source libraries |
| Delivery | [PR26](https://github.com/bohewu/yetlet/pull/26), squash Merge; reviewed tree preserved |
| PR/master CI | [41012346643/1](https://github.com/bohewu/yetlet/actions/runs/41012346643), [41013333532/1](https://github.com/bohewu/yetlet/actions/runs/41013333532), both required jobs PASS |
| Unsigned immutable tag | `v0.9.0-alpha.4` points exactly at C; strict/builtin/native hook PASS |
| Sixteen-package build | [41015091274/1](https://github.com/bohewu/yetlet/actions/runs/41015091274), SUCCESS on its original attempt |
| Original evidence readback | [41015782685/1](https://github.com/bohewu/yetlet/actions/runs/41015782685), preflight/verify/cleanup SUCCESS |
| Actual policy | Automatic workers under the reviewed single-publication policy; no owner click is claimed for these executions |
| Public packages | All sixteen subjects verified; all sixteen hosted archives exactly equal the accepted local alpha.4 archive bytes |
| Original provenance | 9158 bytes; SHA-256 `094e23c14a8eb6cfd76fe449f0382f434a51ba83e34ea63cc9ca03f14e2719dd` |
| Owner-local retention | 40 indexed files, 113,541,467 bytes; index SHA-256 `ab8dd686c1b2392e8af796c76866507531fbecb38bd9efb417391dbc56d78c4b` |

The original encrypted artifact12958749746 has ZIP SHA-256
`83c7d35405e5bdf77a8b8d674945bdacaae8a94d27c07313191cebcc2ffafacd`;
public artifact12959158180 has ZIP SHA-256
`7a36d91851a56751df63a1b3ad3ac1816a239691dfa7f14403ae17daafa260df`.
The hosted safe receipt binds the actual C/workflow/tag/build/attempt, ciphertext
and inventory, complete original manifest verification, and private cleanup.
No formal private key or private evidence plaintext was read locally.

This closes the new normally verified package recipe and automatic original
readback at their actual scope. It does not qualify HA, production, every feature
or a live registry consumer. The [local preflight](ALPHA4-VERSION-PREFLIGHT.md)
retains the first stale-consumer-lock failure and its one temporary correction.
Existing releases, tags, signatures and failed observations remain immutable.

## Retained operational outcomes

The first Environment tag-policy update supplied an unsupported `type` field.
Remote readback showed the old rule unchanged; one schema-correct `name` update
completed the alpha.4 rollover, retaining tag type, one exact rule, no required
worker reviewers, disabled admin bypass and the existing age Secret. No hosted
job was retried for that operator payload error.

The local encrypted ZIP transfer timed out at180 seconds after36,782,080 bytes.
A bounded HTTP206 read resumed the same original artifact. Full56,423,591-byte
ZIP SHA matches the original GitHub digest. The timeout/offset/hash receipts
remain; duplicate transfer parts were removed only after full verification.
Local wrapper ordering/receipt-schema mistakes occurred before acceptance and
were corrected without another hosted run, private decryption or product edit.
Original encrypted/public/readback artifacts and a checksummed index are retained
outside the repository. This is owner-local custody, not an offsite backup or a
claim that expiring Actions artifacts remain available indefinitely.

The consumed tag Guard binding is retired through the supported exact-digest
metadata transition. Original used bytes, tag/history and strict/builtin hooks
are preserved; no rearming or history rewrite occurred.

## Historical signer checkpoint and selected evidence

The following is the accepted pre-signer checkpoint. Its Pending fields are
superseded only by the actual dated execution below; its evidence scope and
independent acceptance remain unchanged.

Use a distinct reviewed full-SHA signer S under
[ADR0009](adr/0009-release-build-and-signer-identities.md). The real build/readback
record and these status/index changes are the next meaningful documentation
commit; no empty commit or runtime/workflow edit is needed. Independent review,
strict/builtin delivery, required PR/master CI and actual merge/tree readback
precede selecting S. Its own SHA is recorded externally after Merge, avoiding a
self-reference in this document. Keep C/tag/product bytes fixed throughout.

Complete the existing versioned record only from actual results: core C/version,
build41015091274/1 and artifact12958749746/digest, original verifier41015782685/1
and Environment ID, actual S, unchanged raw provenance identity above, and the
actual unpublished alpha.4 Draft Release ID. S/Draft/future run/artifact/payload
identities remain Pending until produced; do not substitute old alpha.3 values.

The closed qualification selection retains original native/MSVC/browser/optional
GPU/security/inventory sources and limitations, original failures/support scope,
current alpha.4 package/consumer/review/CI evidence, and the actual hosted
build/readback/custody records. It is a selected evidence set, not every raw log,
complete compiled/system inventory or a recovered denied Windows archive.
Only reviewed allowlisted files are encrypted to the unchanged public recipient;
private keys stay in the isolated GitHub jobs. Historical support acceptance is
not silently expanded by a new signature.

After this record's independent acceptance and S selection, create the unpublished
Draft, upload only encrypted qualification inputs, generate the existing bounded
short-lived encrypted handoff, and execute one prepare. Freeze its actual
receipt and both raw payload SHA values. Recheck actual source/input/tool/custody
bindings before automatic signing with `d6="reviewed-automation"`. Preserve
failures without automatic write/job retries. Verify both bundles, all16 subjects
and the complete private manifest in the separate no-OIDC verifier, including
cleanup and repo-external custody, before presenting the final publication plan.

Independent Reviewer `/root/alpha4_hosted_checkpoint_review_20261030` accepts
this four-document checkpoint and its forty-nine-file selected evidence set with
**PASS**, zero issues/required followups. All four source hashes and selected
file sizes/digests match before/after review; original artifacts and all sixteen
package correspondences were independently checked. Native reviewer/model/actual
high effort are verified. Only this acceptance metadata follows the reviewed
boundary; future S/prepare/sign/results remain separately verified stages.

## Actual preparation, signing and independent verification

[PR27](https://github.com/bohewu/yetlet/pull/27) delivered the reviewed checkpoint
with strict/builtin Guard. Both PR CI41017621833/1 and master CI41018603003/1
passed. Actual S/V is `6ed70c3edb2f122b9a40da1aca1d038e189892f6`, tree
`a7c7518ca709c60cd4caed3a66d013bc945ac8a4`; product C/tag/archives did not change.

| Stage | Original run / outcome |
|---|---|
| Prepare | [41019469360/1](https://github.com/bohewu/yetlet/actions/runs/41019469360), SUCCESS including private cleanup |
| Actual signing | [41020727145/1](https://github.com/bohewu/yetlet/actions/runs/41020727145), SUCCESS; two fixed payloads signed and verified in the signing job |
| Independent verification | [41020913657/1](https://github.com/bohewu/yetlet/actions/runs/41020913657), SUCCESS; both bundles, sixteen subjects and all98 manifest entries verified; plaintext cleanup completed |
| Final independent review | PASS, zero issues/required followups; actual receipts, retained originals, closed20 public projection and exact-byte registry plan accepted |

Complete record SHA-256 is
`ad5524f314d39648d191d0e0a58ca443221123f33fa59ba27b288695dc1f9dd3`.
Original prepare artifact12960503201 has ZIP SHA-256
`3f23d0c51a385b871c8ec1d510d2bfaac27566eb36adf69bdfec2b4c88d45fb9`.
The independent public projection artifact12961677010 has ZIP SHA-256
`61be9005f26486fb4af960862c32a84e910a3c09ab048a8840ccdf4106b98bff`.
Original ZIPs, safe receipts, encrypted private bundles and indexed copies remain
outside the repository. No formal private key or original private plaintext was
read locally. The temporary qualification asset was deleted after retention.

The first Draft metadata update omitted tag/source fields and read back an
unexpected `untagged-*` Draft label. One same-Draft update explicitly bound all
fields; exact name/body/tag/C readback then passed. Product tag C remained
unchanged. Both the first observation and successful correction are retained.

## Owner-approved publication failure and local repair

[Publication41022282283/1](https://github.com/bohewu/yetlet/actions/runs/41022282283)
passed its read-only preflight; its plans matched the local frozen plans. The
owner approved `release-publish`. The protected job failed with
`github: stage_attempted`, `registry: not_attempted`, `status: incomplete`.
Actual Release428745799 remains a Draft with zero assets and exact reviewed
metadata. Approval was recorded; no registry write or consumer check was reached.
This is an incomplete publication, not a release-success claim.

The old combined publisher discarded exception details, so that run's exact
failing HTTP status/suboperation cannot be recovered. Owner-login production
read-only Draft/assets checks pass; they do not prove hosted-token behavior.
Do not retroactively label the original failure as a known HTTP411.

A separate reachable uploader defect is reproduced: `gh api --input -` emits a
chunked request without `Content-Length` with the observed local CLI2.46.0.
[GitHub requires that header](https://docs.github.com/en/rest/releases/assets#upload-a-release-asset).
An isolated loopback endpoint rejects the old command with411. The local repair
sets exact byte length while preserving raw binary stdin; the real CLI loopback
check accepts the exact bytes without chunked transfer. No real token or
external write is involved. Upload/HTTP errors now retain bounded operation,
class/status and publisher phase, excluding raw messages, URLs and bodies.
Local checks and independent repair review are recorded separately from any
future hosted qualification. No publication retry follows from this repair.

## Final owner boundary

The [single final publication gate](RELEASE-PUBLICATION.md) covers an exact
GitHub Experimental pre-release followed by crates.io for the same sixteen
archives. A read-only publication preflight may verify frozen record/notes/plans
and original verified public artifacts. The owner must review the real result,
public attachment/metadata scope and request/plan hashes before approving
`release-publish`. Do not approve that gate on the owner's behalf. No Release
publication, Latest selection, registry write, yank or credential rotation is
performed by this local repair. A later reviewed publisher P may reuse the
unchanged C/S/V/artifacts and signatures; select P only after normal delivery and
green master CI, then refreeze the publication request/summary/plan hashes.
Another hosted attempt needs explicit continuation authority and its actual
owner gate. YT-020 remains **Blocked** until actual registry
readback and registry-only consumer proof succeed.

## Local repair acceptance

Seven affected checks PASS, including the real CLI loopback binary/length check
and safe HTTP/upload/combined-publisher failure cases. Docs75/209, YAML, all16
shell syntax blocks and whitespace PASS. Independent Reviewer
`/root/alpha4_hosted_checkpoint_review_20261030` returns **PASS**, zero issues or
required followups; all ten reviewed source hashes match. Native reviewer/model
and actual high effort are verified. This acceptance-only metadata follows the
reviewed boundary. Actionlint is unavailable; hosted cache-hit/performance and
corrected live publisher acceptance remain unverified. No Commit/Push, new
hosted execution, retry or publication occurred in this local slice.

## Actual public release and one-package registry outcome

[PR28](https://github.com/bohewu/yetlet/pull/28) delivered the upload-length and
bounded-diagnostic repair with strict/builtin Guard. Both PR CI41026724358/1 and
master CI41027664663/1 passed. Publisher P is
`1c0bb3157e27db026b8f8c050c658e7c48973749`; its reviewed tree is
`9fc1936369ce95963ca5db2a9f8443a84680f981`. C/tag/S/V, all original archives and
signatures remain fixed. CI tool caching completed cold restore/save; no warm
speedup is claimed.

The owner approved [41028646006/1](https://github.com/bohewu/yetlet/actions/runs/41028646006).
GitHub publication and authenticated/anonymous readback succeeded for all twenty
exact planned assets. [Release428745799](https://github.com/bohewu/yetlet/releases/tag/v0.9.0-alpha.4)
is a public Experimental prerelease, not Latest. No private evidence was attached.
The job subsequently failed in registry response validation, and cleanup passed.
Its original receipt records `github: published_and_verified`, registry attempted
and overall incomplete; it is not rewritten as an overall success.

Only `yetlet-actions0.9.0-alpha.4` was sent. Independent observation confirms its
owner is bohewu, it is not yanked, and API/index/download all match the exact
11,597-byte archive SHA-256
`ca5decf8a93c213fe60bfad981d75e52ac01f9b3ca56c49e8f2d8eddbad411e3`.
The other fifteen packages and registry consumer were not attempted by that run.
The original registry receipt remains incomplete with one attempted entry; a
separate read-only reconciliation records the observed successful write.

The parser incorrectly required `krate.name`. Cargo's published success contract
does not require that field, and crates.io's API type renames its Rust `krate`
member to JSON `crate`. The old mocks encoded the same wrong assumption. The
original PUT body was not retained; its exact fields are not fabricated.

## Local registry-only continuation

The [continuation contract](RELEASE-PUBLICATION.md#registry-only-continuation-contract--2026-10-30)
binds original result artifact12964529138, ZIP SHA-256
`d264113a13efb9ee6bef900d78a9dfe3d8c5db01a8cf53d1b7d1cc3b3fabedb7`.
The unchanged sixteen-package registry plan SHA-256 is
`83841e02ce97c22cc41902de67af7f042a75ca95c65eba26ea72916dc8d162a7`.
An explicit known one-plus-fifteen selection preserves the public acceptance
summary and all twenty asset IDs/bytes. The new protected job has GitHub read
permissions and the existing final owner gate; it has no GitHub mutation,
signing or age-key path. Existing-package readback/owner and all target-version
absence checks precede writes; any detected target version stops rather than
being silently skipped or resent.

Local affected checks use official warnings-only and additional-`crate` success
formats, fifteen original body/dependency ordering, unchanged published metadata,
bound failed artifacts/receipts, owner/job gates and zero-write refusal cases.
Read-only preparation with actual artifacts matches the original request/plans,
confirms all twenty public assets, verifies the existing package/owner and
observes all fifteen remaining versions absent. No registry token/private key or
mutation is used. A non-executable request template leaves the future P Pending.

Independent local review is recorded separately. New P delivery/CI, actual
account new-crate capacity, a separately authorized hosted attempt and its owner
gate remain open. No complete registry consumer, Commit/Push, upload, re-signing,
GitHub asset change or YT-020 completion is claimed by this local preparation.

Local continuation acceptance: **fifteen distinct focused checks PASS**, with
only the two extended cases rerun; actual original-artifact/read-only state
preparation PASS. Docs75/209, workflow YAML/nine shell blocks/Python syntax and
whitespace PASS. Independent Reviewer
`/root/alpha4_hosted_checkpoint_review_20261030` returns **PASS**, zero issues or
required followups; all nine reviewed source hashes match. Native reviewer/model
and actual high effort are verified. Only this acceptance metadata follows the
reviewed boundary. New P/CI, hosted acceptance, account capacity and full registry
consumer proof remain Pending; this verdict grants no upload or Git authority.
