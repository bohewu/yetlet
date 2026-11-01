# SDKalpha.6 release preparation — YT-228

Date: 2026-11-01. In progress; no publication yet.

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
