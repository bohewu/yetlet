# Alpha.4 — Hosted source packages and signing checkpoint

Date: 2026-10-30 (Asia/Taipei). **Original build/readback and owner-local retention
PASS. Signing preparation/result Pending.** The owner authorizes continuation
through preparation, signing and independent verification, stopping for the
final combined GitHub/crates.io publication confirmation.

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

## Signer checkpoint and selected evidence

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

## Final owner boundary

The [single final publication gate](RELEASE-PUBLICATION.md) covers an exact
GitHub Experimental pre-release followed by crates.io for the same sixteen
archives. A read-only publication preflight may verify frozen record/notes/plans
and original verified public artifacts. The owner must review the real result,
public attachment/metadata scope and request/plan hashes before approving
`release-publish`. Do not approve that gate on the owner's behalf. No Release
publication, Latest selection, registry write, yank or credential rotation is
performed by this checkpoint. YT-020 remains **Blocked** until actual registry
readback and registry-only consumer proof succeed.
