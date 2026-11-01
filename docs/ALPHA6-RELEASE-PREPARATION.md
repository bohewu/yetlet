# SDKalpha.6 release preparation — YT-228

Date: 2026-11-01. YT-228 remains Done at preparation scope; the later owner-approved
GitHub/crates.io publication and registry consumer also PASS.
Earlier Pending/waiting statements below are dated stage checkpoints; the final
section records the actual publication without replacing their history.

## Selected scope and authority

After accepting [YT-227's plan](YT-227-NEXT-RELEASE-PLAN.md), the owner authorizes
committing the six planning documents and proceeding through preparation to the
last publication approval. Planning commit `a106c2f` passes strict/builtin Guard.
Prepare SDK `0.9.0-alpha.6` first, then independent CLI `0.2.0` after the new SDK
is public. Keep the existing registry-token route; optional registry OIDC is
deferred, with no new permissions or credential exchange.

This batch may deliver reviewed changes via strict/builtin Commit/Push, PR/CI
and squash Merge, select immutable source identities, and execute the existing
bounded build/readback/prepare/sign/independent-verification stages once each.
It stops at the owner's `release-publish` gate. GitHub publication and crates.io
upload remain unauthorized. Preserve failures, with no automatic retry, moved
tag, historical signature reuse or third-party upgrade.

## Product and guide boundary

Exactly `yetlet` then `yetlet-browser`, schema2/`consolidated`. Include the
already reviewed YT-217/220/221 API docs, ordinary DOM vocabulary, initial-pre
newline preservation and typed per-mount URL/image options. Existing HTML/DOM
defaults and optional feature/security boundaries remain unchanged. This is
Experimental source distribution, not a production/platform expansion.

Advance workspace/path dependency versions and nine path-based lockfiles,
including sample packages that inherit the workspace version.
Keep CLI0.1.1, its bundled SDK resources and the registry-only docs/static-site
consumers on SDKalpha.5. The public guide keeps its approved alpha.5/CLI0.1.1
edition while development versions advance. The minimal label check now admits
that fixed published edition independently of workspace versions; it still
rejects dirty public inputs and unreviewed editions. The actual guide consumer
retains its existing pinned SDK and hardcoded admission check. No automatic
edition promotion, preview fallback, workflow or deployment-policy change.

CLI0.2.0's actual registry pin/resource migration and final consumer qualification
wait for SDKalpha.6 publication. They cannot be completed before this SDK gate
without introducing a registry override or prematurely publishing the SDK.

## Local affected verification

Evidence root outside the repository: run `yt227-next-release-plan-20261101`,
subdirectory `sdk-alpha6`.

| Check | Actual result |
|---|---|
| Published-guide metadata | 21 Python tests PASS, including new workspace versions retaining the old published edition, candidate-edition refusal and unchanged dirty-source/deployment separation |
| Dependency correspondence | All nine modified locks change only inherited local package versions; no upstream version/source/checksum/dependency changes. Full locked root/dashboard dependency resolution PASS; CLI and registry-only consumer locks unchanged |
| Normal packaging | Existing explicitly selected consolidated-archive test PASS: base verified before Browser, exact predecessor checksum staged, no bootstrap or verification bypass |
| Archive/source | Two normalized versions, licenses and all99 included product/source files correspond; Yetlet163499 bytes and Browser51602 bytes |
| External consumer | Exact-version archive-backed native execution and wasm32 compile PASS: shared View, canonical table, default image refusal, explicit options and pre newline/escaping |
| Reused behavior | Eight selected SDK product-input hashes match accepted YT-220/221 records. Prior browser/native security evidence remains at its original scope; no browser matrix replay |
| Reused upstream closure | Final Yetlet192 and Browser86 upstream lock entries exactly match their accepted alpha.5 archives; retain existing inventory/security/license decisions for those unchanged entries |
| Independent preparation review | First P1 retained; followup PASS0,54 hashes/status/diff match, registered reviewer/model/effective high verified. Local preparation may be delivered |
| Delivery/platform | PR47 all checks PASS, squash Merge Cc7e6372/treec3137b4 matches the reviewed tree. Actual master Rust CI41305920477/1 both jobs PASS. Real MSVC normal archives and changed-options/SSR native consumer PASS; both archive digests match GNU; owned checkout/cache/build cleaned |
| Independent platform acceptance | PASS0,55 source/15 evidence hashes match, original P1 preserved and all28 inherited entries corrected; registered reviewer/model/effective high verified |
| Missing execution proof | Fresh hosted build/readback/prepare/sign/result and final publication plan remain Pending |

The first independent preparation review is FAIL with one P1: root and dashboard
locks retained26/two sample versions inherited from the old workspace version.
The minimal correction updates those local entries; complete `cargo metadata
--offline --locked` resolution now passes for both roots without replaying
product tests. Original review/source snapshots remain retained. The corrected
entries are absent from the isolated SDK archives; their verified source,
normalized manifests and packaged locks remain unchanged.

Local package hashes (new bytes; not published):

| Archive | SHA-256 |
|---|---|
| yetlet-0.9.0-alpha.6.crate | 0726e0fbe8f9ed0fcf9163fd64f88859c0a5e69a34ea73f4cd531e4820df09cc |
| yetlet-browser-0.9.0-alpha.6.crate | 8bb3dd2c8481a9ef405a41e5a28cee33b55042f1921bfe874568f1d8509cc482 |

The local packaging test deliberately isolates the two SDK manifests. Its archive
bytes must be reconciled against the later immutable-source hosted build; a local
hash does not substitute for new C/build/readback evidence. Do not replay unrelated
provider/cloud/GPU/benchmark qualification or relabel old candidate PASS.

## Remaining execution and final stop

1. Obtain independent preparation acceptance and deliver the complete bounded
   change, including affected local caller checks, in one PR. Require exact-head
   CI, squash Merge and actual-master CI. Supplement only the missing MSVC
   archive/consumer proof through a Git-sourced isolated Windows checkout.
2. Select C after qualification; create/read back the immutable unsigned
   `v0.9.0-alpha.6` using strict/builtin existing-remote tag support. Preserve and
   explicitly retire any consumed Guard binding under the existing procedure.
   Run one source-bound build (45 minutes) and one original readback
   (5-minute preflight/15-minute protected phase); preserve complete ciphertext
   and verified receipts outside transient Actions artifacts.
3. Select reviewed S distinct from C; keep V/P identities and actual master CI
   explicit. Use the existing protected master workers, unchanged age custody,
   short-lived encrypted handoff and no local formal-key decryption. Execute one
   prepare, one signing and one separate result verification within their
   committed timeouts. Retain payload hashes, both real bundles, complete private
   manifest and cleanup proof. Only already accepted identity/transparency fields
   may become public during signing.
4. Derive six public files: two archives, unsigned provenance, public checksum,
   provenance Sigstore bundle and reduced signing-acceptance summary. Prepare
   exact notes, public allowlist and joined registry plan. Dispatch publication
   preflight and stop at `release-publish`; the owner decides whether to publish.
   No private manifest, encrypted private bundle, credential or qualification
   directory is a public attachment.

C/S/V/P, new tag/run/artifact IDs, Draft and signing outputs are Pending until
actual execution records establish them. Existing releases, signatures and
failure records stay unchanged. YT-228 is not Done until its independent
acceptance and stated pre-publication proof exist; publication remains separate.

## Selected C and first hosted execution

The owner-authorized PR47 delivery selects immutable C
`c7e63724b2fc303abcf85ff503cc8f230794cb41`, tree
`c3137b46001bdf41d5cff5138b9452f11457d18e`, after actual master CI and independent
platform acceptance. The original planning/local-preparation branches remain
preserved. Full required CI runs once on PR and once on the actual merged source;
those are distinct source events, not a failed-job retry. Windows Rust1.99,
VC14.44.35207 and SDK10.0.26100 proof covers the selected default/archive consumer;
the existing xtask dead-code warning is retained without a new feature claim.

The unsigned lightweight `v0.9.0-alpha.6` tag is pushed/read back at C through
strict/builtin accepted-remote-history Guard and its native hook. The used binding
is canonically retired without moving any tag or changing hooks. Exactly one
[build41306591044/attempt1](https://github.com/bohewu/yetlet/actions/runs/41306591044)
starts at that C/tag, with the committed45minute limit. It remains In progress at
this checkpoint. Original readback will use only this build's actual encrypted
artifact/digest and the exact selected-tag Environment. The first Environment
policy update rejects an immutable type field with422 before mutation; the
supported name-only update passes after unchanged-state readback. Both records
are retained, with no hosted attempt consumed. No Secret/key rotation occurs.
Prepare/sign/result, S/V/P and publication stay Pending.

This tracking-only followup is intended to establish the later reviewed signer S
distinct from C, with unchanged package/source inputs. Pending values above are
historical stage boundaries; actual execution records supersede them individually.

## Verified outputs and final publication stop

Product C remains `c7e63724b2fc303abcf85ff503cc8f230794cb41`, with the immutable
unsigned `v0.9.0-alpha.6` tag. Tracking PR48 passes both existing required light-CI
jobs and squash-merges as S/V/P
`65310013dab38dca2ffa82bd5cdb0fde1806f31c`. Only three maintainer documents differ
from C; package inputs remain unchanged. Its actual required master CI
41307133072/1 passes. Optional Rust CodeQL continues independently at Merge and
subsequently passes for both PR48 and the merged master; it was not skipped or
relabeled. No extra CI, failed-job or release-stage rerun occurs.

| Stage | Actual run / attempt | Result |
|---|---|---|
| Normal source build at C | [41306591044/1](https://github.com/bohewu/yetlet/actions/runs/41306591044) | PASS; both archives equal the accepted GNU/MSVC bytes |
| Original encrypted-evidence readback | [41307044066/1](https://github.com/bohewu/yetlet/actions/runs/41307044066) | PASS; complete original manifest/inventory and cleanup |
| Intake and frozen preparation at S | [41307620413/1](https://github.com/bohewu/yetlet/actions/runs/41307620413) | PASS;25 bounded qualification entries; temporary Draft ciphertext asset removed after retained freeze |
| Actual signing at S | [41307961476/1](https://github.com/bohewu/yetlet/actions/runs/41307961476) | PASS; two frozen payloads, real bundles and same-job checks |
| Independent no-OIDC result verification at V | [41308106470/1](https://github.com/bohewu/yetlet/actions/runs/41308106470) | PASS; two package subjects and all44 private manifest entries; key/plaintext cleanup precedes safe receipt |

Frozen provenance SHA-256:
`e21e9144d67861e1109d2dfe2993fc85efd9f6d37f1af9123dd66c999da08379`.
Frozen full-manifest SHA-256:
`d37d38eed7dd38406b9357d9bdf95499aaa209c1fedae1062b5295151cf44f4e`.
These hashes remain unchanged through actual signing and independent verification.
Full ciphertext/artifact ZIPs, receipts, both bundles and23 selected retention
bindings are in the repository-external evidence store. No formal private key or
decrypted original evidence is accessed locally; this is owner-host retention,
not an offsite recovery claim.

Draft release430574118 stays unpublished and Experimental, with fixed C/tag,
no Latest designation and zero assets before the owner gate. Final notes and
prospective public/registry plans pass the existing closed-cohort helper checks
and actual Draft metadata readback. The owner-approved publisher will upload and
read back exactly six public files before making the GitHub pre-release public,
then upload the same two archives to crates.io in dependency order. The preview
acceptance summary is prospective; it is not a claim of owner approval.

| Public file | Scope |
|---|---|
| `yetlet-0.9.0-alpha.6.crate` | Verified base archive, unchanged digest above |
| `yetlet-browser-0.9.0-alpha.6.crate` | Verified matching Browser archive |
| `PROVENANCE-UNSIGNED.json` | Original C/build provenance with both subjects |
| `RELEASE-CHECKSUMS.json` | Closed three-entry public archive/provenance checksums |
| `provenance.sigstore.json` | Actual public provenance signature/identity/log bundle |
| `SIGNING-APPROVAL.json` | Reduced summary activated only in the owner-approved publishing job |

Private manifest contents, encrypted private signature bundle, qualification
files, temporary capabilities and credentials are excluded from public files.
The actual registry-only post-publication consumer is not run before upload.
CLI0.2.0 still waits for the SDK's actual registry publication.

Retain all preparatory failures separately from successful hosted attempts:
the initial inherited-lock P1,422 Environment type-field rejection, prefixed
temporary-asset name rejected by local handoff, omitted local automation-mode
context, and a notes/name-only private Draft update that returned an untagged
metadata name. Corrections use the supported tag-policy name update, the same
ciphertext asset renamed to the canonical name, the already committed automation
mode and an explicit complete Draft identity update. The Git tag, C/S, archive
bytes and signatures never change. No new PR/runtime fix, re-upload, re-sign or
second hosted attempt is needed for these local/settings corrections.

Final independent output review and one publication preflight remain Pending at
this checkpoint. Stop at the owner's `release-publish` gate; no approval or
GitHub/crates.io publication is inferred from these successful worker stages.
New closeout documents remain local while the fixed P/master is held stable.

## Independent final acceptance and observed owner gate

Final independent Reviewer PASS0, zero issues/required followups, accepts all
three closeout-document hashes,32 evidence hashes plus the supplementary CodeQL
record, all23 retention bindings and the actual25/44-entry qualification/verification
correspondence. Registered reviewer/model and effective high are verified. The
acceptance permits Done only after actual preflight PASS and the observed gate.

[Publication preflight41308935654/1](https://github.com/bohewu/yetlet/actions/runs/41308935654)
then passes once. Its retained public and registry plans exactly match the
reviewed local plans. The `publish` job is waiting for owner approval in
`release-publish`; GitHub/registry receipt fields remain `not_attempted`.
The assistant performs no approval. YT-228 is Done at preparation scope only;
the owner may approve this exact run to publish the six-file GitHub Experimental
pre-release (not Latest), then the two identical registry archives. Actual upload
and the registry-only consumer remain unexecuted. CLI0.2 still follows successful
SDK publication. The three new closing documents are not committed/pushed so
that the fixed publisher source stays unchanged during this gate.

## Owner-approved publication and registry consumer — 2026-11-01

The owner approves `release-publish` and asks for publication monitoring, exact
checksums and registry-consumer verification, preserving failures without retries.
The same [publisher41308935654/attempt1](https://github.com/bohewu/yetlet/actions/runs/41308935654)
completes successfully at fixed P `65310013dab38dca2ffa82bd5cdb0fde1806f31c`.
The actual approval by owner `bohewu` for the selected Environment is retained.
There is no second dispatch, retry, re-signing or tag/source change.

[GitHub SDKalpha.6](https://github.com/bohewu/yetlet/releases/tag/v0.9.0-alpha.6)
is public as an Experimental pre-release, not Latest, with exactly the six reviewed
files. Actual metadata and publisher receipts match the frozen public/registry
plans, final notes and fixed C/tag. The two crates.io versions are available and
not yanked:
[yetlet](https://crates.io/crates/yetlet/0.9.0-alpha.6) and
[yetlet-browser](https://crates.io/crates/yetlet-browser/0.9.0-alpha.6).
Anonymous owner-host downloads from both GitHub and the registry CDN match each
other and the exact archive SHA-256/size table above. Registry metadata, the
publisher's index readback and the consumer lock agree on both checksums.

The actual hosted consumer uses a fresh credential-free Cargo home, exact
`=0.9.0-alpha.6` registry dependencies and no path/patch/source replacement.
Version resolution and `cargo +1.99.0 check --locked` PASS. Its retained lock
SHA-256 is `ac833fa4b0b7992a47861db005f94a4a4c944254d333c249988397825078b92e`;
both SDK lock entries name the public registry, exact version and approved crate
checksums. This is compile/resolve proof, not a new runtime or browser test.
Passed consumer/signature/product checks are not replayed locally.

Publisher result artifact13085121451 has ZIP SHA-256
`dbd0e1ab8c04b51b1899fafdadccd6f5f6db64ce599e0834dbecc4ec8f65910b`.
Its complete GitHub/registry receipts, both plans, consumer logs/lock, approval,
anonymous release/version responses and downloaded archive bytes are retained
outside the repository in the same evidence run's `publication-monitor` directory.
Earlier review/settings/operator failures and immutable build/signature artifacts
remain intact. Publication/registry receipt states are complete with automatic
retry disabled; no new independent-review or broader platform claim is added.

This monitoring followup changes only the three existing local closeout documents;
it does not Commit/Push, change the live guide edition, republish SDK/CLI or start
CLI0.2. The planned CLI0.2 SDK-publication prerequisite is now satisfied; its
version/resource/binding preparation and affected checks remain separate work.
