# Alpha.5 hosted release record

Date: 2026-10-31 (Asia/Taipei). Status: GitHub pre-release and both crates.io
packages published successfully. Earlier Pending statements below belong to
their historical execution checkpoints; failures are preserved.

## Completed publication and subsequent cleanup

The owner-approved [publisher41104391525/1](https://github.com/bohewu/yetlet/actions/runs/41104391525)
succeeds at P `bb414d173ad960356347bb9aaa1ddd0a27f0ef03`. It retains product C
`0a9fa8a594e3dd2ce753422dd848df10cd8c98df`, S/V
`864f2abc7ca076db13f18ddd5fbcc1aa0145fe0b` and the existing unsigned tag.
[GitHub alpha.5](https://github.com/bohewu/yetlet/releases/tag/v0.9.0-alpha.5) is a
public Experimental pre-release, not Latest, with the six checked public assets.
Both registry versions are available, not yanked, and their downloaded archives
match the frozen public checksums:

| Package | Version | SHA-256 |
|---|---|---|
| yetlet | 0.9.0-alpha.5 | 662d67180cec2fd6e50c3757bdc910d057cd37428785ab695b0d275a9e4e5091 |
| yetlet-browser | 0.9.0-alpha.5 | 2fbeec5d4324367cf682d722fc25a35f851fc5293fc6e353aedc35ca6771c9a6 |

An isolated registry-only consumer passes without Cargo path overrides or
forwarded credentials. Its lock SHA-256 is
`4116b7b8342c048b52ea1f71d5a5c612b32ccb8575e6ea27f1590d5d5c939c1b`.
These are retained publication receipts, not newly replayed qualification.
Local receipts: `target/yt096-alpha5-execution-20261031/PUBLICATION-COMPLETE.json`
and its `publication-result/` directory. Durable copies are in the repo-external
release-evidence store under the source/build-bound alpha.5 run.

After this publication, the owner separately deletes the old `yetlet-actions`
crates.io package; it must no longer be described as an available dependency.
The old signed artifacts/tags and first-publication result remain historical
evidence. PR29 is closed unmerged and its legacy continuation is retired.
Later release-note status corrections do not alter those signed attachments or
move any tag. CLI installation followup is [YT-214](YT-214-CLI-INSTALLATION.md),
not a third SDK publication or a new version of either released package.

## Source and selected scope

PR31's accepted Windows xtask correction and YT-212 closeout are squash-merged
at `0a9fa8a594e3dd2ce753422dd848df10cd8c98df`, tree
`7de1f613f6bd9263d5aaf5c83af495bfaad25217`. This is the alpha.5 C after its actual
master CI passes both required checks. Old `0f0c84a` remains an accepted historical qualification
checkpoint; no old tag or artifact is changed. Both required PR31 checks pass
at `04c2d177ed8d5586690938a8ad3659319d89f57d`. Both checks pass at the actual master run
[41086232892/1](https://github.com/bohewu/yetlet/actions/runs/41086232892).

Product record: schema2, version `0.9.0-alpha.5`, cohort `consolidated`, exactly
`yetlet` followed by `yetlet-browser`.101 SDK/package inputs and the two
MSVC-tested xtask files match accepted YT-212/213 proof. Existing qualification,
security, inventory, browser and failure records retain their original source
and feature/target limits; see the [qualification record](ALPHA5-RELEASE-PLAN.md).
No SDK/browser requalification, third-party upgrade or production claim.

## Execution and freeze boundary

The lightweight unsigned `v0.9.0-alpha.5` tag is pushed and read back at C with
strict/builtin accepted-remote-history Guard/native hook PASS. The consumed
binding is preserved/retired without moving the tag or changing hooks.
[Build41087985092/1](https://github.com/bohewu/yetlet/actions/runs/41087985092)
passes with the existing45minute normal base-to-Browser recipe. Original readback
[41089439159/1](https://github.com/bohewu/yetlet/actions/runs/41089439159) passes
both5/15minute preflight/protected phases. Reconcile the exact two archives,
provenance and complete encrypted private evidence before retaining receipts
outside transient Actions artifacts. No local formal private key/decryption.
The original readback Environment retains its identity/no-reviewer/admin-bypass
policy and now permits exactly the alpha.5 tag; sign/result/publish policies and
Secrets are unchanged.

The original tracking-only source S is `035033ea4c3ea8e1792738df6b2af25fe486fe09`
after PR32 and exact master CI41089812359/1 pass. Its unexecuted complete record
and failed local handoff remain historical evidence. The small encrypted Draft
asset returns a provider capability with299seconds remaining, incompatible with
the old1200second minimum and delayed retrieval. No prepare/sign workflow has
started and no payload is frozen. The required signer-only correction gets a
new reviewed S after guarded PR/CI/Merge and exact master CI; select that observed
SHA in a separately named record. C/tag/build/readback,65 qualification entries,
original manifests and encrypted asset692720632 stay unchanged. No re-upload,
SDK qualification replay, private-key access locally or silent payload refreeze.
Buffer master after the replacement S selection; V/P may equal S.

An unpublished Draft carries only temporary encrypted qualification inputs until
verified retention/cleanup. One prepare freezes the complete selected assembly
and both raw payload hashes. One `reviewed-automation` signing attempt uses the
standing D1/D2 disclosure authority; independent result verification has no
OIDC/signing permission. Prepare splits its existing protected30minute budget
into a3minute encrypted intake and27minute fresh-runner preparation; the5minute
preflight and sign/result worker bounds remain unchanged. Preserve the
complete private manifest, actual certificates/logs, original failures and
cleanup receipts. No automatic failed-job/write retry or silent refreeze.

## Final owner gate

Derive the six public files from the actual independently verified two-package
cohort: two crates, unsigned provenance, public checksum, Sigstore provenance
bundle and reduced acceptance summary. Prepare exact notes and joined registry
plan before the final `release-publish` gate. Do not add private evidence or a
directory glob. GitHub Experimental pre-release precedes registry writes, uses
no Latest designation, and both destinations require the owner gate.

Public absence of the two new registry names does not prove token/create rights.
The owner explicitly confirms the existing dedicated90day publication token
permits both names, including first creation; Secret presence alone was not
treated as proof. Never import local cargo
credentials or print a token. Keep old alpha.4 and its published actions crate
immutable; its fifteen-package continuation stays paused.

Actual tag/build/readback/S/payload/run/digest/Draft/publication identities belong
to the frozen data-only execution records and later result entry. Pending values
are not accepted outputs. Historical D6 acceptance is preserved and does not
automatically attest to alpha.5 payloads.


## Actual frozen inputs and verified signatures — 2026-10-31

PR33 and exact master CI41095951188/1 select signer/verifier S/V
`864f2abc7ca076db13f18ddd5fbcc1aa0145fe0b`. The short-lived intake, prepare and
cleanup pass in [41096828780/1](https://github.com/bohewu/yetlet/actions/runs/41096828780).
The same frozen bytes are signed in
[41097671841/1](https://github.com/bohewu/yetlet/actions/runs/41097671841), then
independently verified in
[41097893900/1](https://github.com/bohewu/yetlet/actions/runs/41097893900):
**two subjects and84 private manifest entries PASS**, with plaintext cleanup.
Original encrypted build, intake, frozen assembly, both bundles and safe receipts
are retained outside transient Actions storage. No local formal key was accessed.

Before dispatching publication preflight, a local audit finds two remaining
sixteen-package counts in the public allowlist validator:20 assets and17 checksum
entries. These would reject the actual six public files/three checksum entries.
Use the validated record's closed package names to derive the counts, retaining
exact names, duplicate rejection, hashes and private-file exclusion. Legacy
sixteen-package continuation remains unchanged. Exercise the entire consolidated
stage/readback/publish boundary with synthetic transport and the actual verified
projection with read-only metadata before delivery. No hosted publisher attempt
or publication write has occurred. Deliver one reviewed publisher correction;
select actual P after guarded PR/CI/Merge. C/tag/S/V and signatures stay fixed,
with no rebuild, re-signing or independent-result replay. Final publication
continues to require the owner's `release-publish` approval.
