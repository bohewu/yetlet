# Yetlet CLI 0.4.1 Experimental — hosted publication record

**Current distribution (2026-11-17):** Yetlet CLI `0.4.1` is published as
an Experimental pre-release, independently of SDK `0.9.0-alpha.8`.
See the [owner-approved publication and installed consumer](#owner-approved-publication-and-installed-consumer--2026-11-17)
below for the current authority. The earlier signing dossier, including its
Pending states, is **historical pre-publication evidence**, not current status.

**Original signer-dossier checkpoint (before publication):** This was an
unsigned candidate with independent encrypted evidence readback; it was
not yet signed or published. The accepted public CLI was `0.4.0`, with
SDK `0.9.0-alpha.8`. The dossier did not then select signer S, public
Release ID, verifier V or publication P or authorize distribution.

## Immutable source and review, 2026-11-17

| Identity | Actual retained result |
|---|---|
| Cohort | Schema 3, independent one-package CLI, proposed version 0.4.1 |
| Immutable product C and tag | 7fbf918a5a0734f7ed125afd6c860837b1de55d1; lightweight yetlet-cli-v0.4.1 |
| Source tree | 3865d5c5a5f2a7eb7216a0961268553b77ef51c7 |
| Source PR | [PR #98](https://github.com/bohewu/yetlet/pull/98), merged from 08b6fba4a9d35c49f45db0148e4dbbe01f876123 |
| PR GitHub CI | Documentation 44013901447, Rust 44013901173, CodeQL 44013901145; all completed/success |
| Protected C master CI | Documentation 44015423425, Rust 44015423475, CodeQL 44015423494; all completed/success |
| Build SDK | Exact yetlet dependency 0.9.0-alpha.8 and matching embedded SDK source/snapshot |
| SDK resolution | sdk-latest-v2 remains independently scoped to generated apps |
| Historical release binding | Published CLI 0.4.0 built against SDK alpha.7; never relabel its bytes or tag |
| Reader Edition | Remains the verified published SDK alpha.8 / CLI 0.4.0; no premature 0.4.1 promotion |
| Independent source Reviewer | Initial R1 wc_job_UzqSe42-30UPCHTU actual FAIL/P2 retained; corrective R2 wc_job_1IagNb15rq_cBarA terminal/exit0 actual JSON PASS, no issues/followups |
| Tag authorization | Standalone full Ubuntu Git clone; strict/builtin native pre-push PASS, zero findings, exact C/master/tag remote readback; one-time acceptance archived after use |

The R1 P2 identified a stale hardcoded CLI-version list in the publication
workflow's Browser/WASM tool prerequisites. Source PR #98 replaced it with the
schema-3 CLI cohort condition and a version-derived source/workflow test.
The independently reviewed correction and exact-head CodeQL/Rust/Docs passed
without changing the owner publication gate or the schema-2 SDK branch.

The current source includes the Windows Ctrl+C ERROR_ACCESS_DENIED race repair
from [PR #97](https://github.com/bohewu/yetlet/pull/97), retained one-run
signal/exit/cleanup and focused MSVC regression, updated CLI resources for
SDK alpha.8, and other source-only CLI development changes accumulated since
the immutable 0.4.0 tag. Do not call this release an isolated one-file hotfix.
Ordinary Browser UI/WASM/CSP positive-path compatibility uses source-bounded
Ubuntu Chrome evidence; injected inline-handler denial was not run and is not
claimed. No general production, GPU or Windows Chrome qualification is added.

## Original unsigned build and independent readback

| Evidence | Exact observed identity |
|---|---|
| Source-bound [unsigned GitHub Actions build](https://github.com/bohewu/yetlet/actions/runs/44017724892) | Run 44017724892 / attempt 1; completed/success, C exactly as above |
| Original unsigned transport ZIP | Artifact 14298899380, SHA-256 2ac3c9a214e852590d98383c0b25f26f1d445bead83be173d003a33f5d2b06d0, 544103 bytes |
| Original encrypted private transport ZIP | Artifact 14300280017, SHA-256 1ba77dde229169b27176b20573d46a26f4ebac6e7339bc420d2fd4fce0be27e5, 56495311 bytes |
| Unpublished CLI package | yetlet-cli-0.4.1.crate, 541417 bytes, SHA-256 ac449f2dc415abb6c44ca8ee9b2b14a3f7f587e7e6f9b636cf0f5945c30dff0e |
| Original unsigned provenance | PROVENANCE-UNSIGNED.json, 5557 bytes, SHA-256 0044721fd74ca6179017256a63ff7549c1e1d99f78750c24ccc6b9e0eff21a56 |
| Original public checksums | RELEASE-CHECKSUMS.json, 430 bytes, SHA-256 54277992b637c8d9b8e1478611be7d12f5015f502cbea43c958420f8ee2b7796 |
| Package readback | 164 original crate entries; exact package 0.4.1, Build SDK alpha.8 and package README checked without repackage |
| Original ciphertext SHA-256 | e07ae8fe402b5b2ce2bbdd8487a49a5f0f4cf6f683d84c971b8b4c1a05030f4a |
| Original private inventory SHA-256 | 291855e2204ab7c87de9bcc0f9124caed5a5788f8002cdb9bc511a0fd305f2e3 |
| Corrected independent verifier | [Run 44018677389](https://github.com/bohewu/yetlet/actions/runs/44018677389) / attempt 1: Preflight and Verify completed/success |
| Safe readback transport ZIP | Artifact 14300540368, SHA-256 1f652be11f90d8b1a211c65f19f8e9578cfc4eda8099ed0c93621daacb52ff81, 686 bytes |
| Original safe readback receipt | yt096-safe-readback-receipt.json, 878 bytes, SHA-256 32035ac65e7ed661398e956bd0079e8fa705837e696ca8ccd4823d933aa38cf3 |
| Safe receipt result | original_exit=0, status=PREPARATION_SUCCESS, complete=true, local_decryption_and_inventory_verified=true, private_manifest_verified=true |

The initial independent verifier Run 44018194331 failed its read-only
Preflight before accessing private-key material. The operator-selected first
record incorrectly referred to the unsigned artifact rather than the required
encrypted-private artifact, and the protected evidence Environment still had
the old 0.4.0-only tag policy. The original failed run remains unchanged.
The Environment's one allowed tag was moved to 0.4.1 with no admin bypass,
reviewer change or broadened lasting ref set; the corrected encrypted-artifact
record passed the original read-only Preflight before the successful new
verifier execution. This is an input/policy correction, not a replacement
package build or a successful verdict for the failed run.

The safe readback receipt proves local controlled decryption and inventory
readback at C. It does not alone establish durable remote backup, signer
qualification, public publication or future package consumers. Original
seven-day GitHub artifacts expire after 2026-11-24 UTC unless retained outside
transient CI storage.

## Pending independent signer selection and owner gate

Before selecting signer S, independently review this dossier, exact original
artifacts, the external flat qualification selection, source and historical
failure identities, and release notes. Bind each reused old-platform check to
its actual source/snapshot instead of changing its source identity to C.
Deliver this source-neutral evidence record through a guarded documentation PR
and exact PR/protected master CI; S must be a later master commit distinct
from C, while the package archive remains fixed at C.

Only after S is reviewed: create an unpublished Experimental Draft bound to
the immutable C tag, bind its actual Release ID, approved notes SHA, original
Build/readback artifacts and provenance SHA/size into the complete schema-3
release record. Prepare the age-encrypted qualification handoff immediately
before the exact protected signing job. Prepare/sign frozen payloads under the
existing reviewed-automation policy at S and require a separate independent
result verification at V; publication preflight at P must re-derive the
unchanged allowlisted five-file CLI package and registry plan.

**Historical signer-dossier gate (since completed):** At this checkpoint
the final action was pending at the protected release-publish Environment.
No automated agent was authorized to bypass owner approval or retry uncertain
writes. The completed owner-approved publication is recorded below.

## Owner-approved publication and installed consumer — 2026-11-17

CLI `0.4.1` was approved by the owner through the protected
`release-publish` Environment and published as an Experimental pre-release:
[GitHub Release](https://github.com/bohewu/yetlet/releases/tag/yetlet-cli-v0.4.1)
and [crates.io `yetlet-cli` 0.4.1](https://crates.io/crates/yetlet-cli/0.4.1).
SDK `0.9.0-alpha.8` remains independently published and was **not** re-released.
The public Reader Edition is promoted separately through a reviewed Git
change to `site/pages.toml`.

| Proof | Actual result |
|---|---|
| Product C / immutable tag | `7fbf918a5a0734f7ed125afd6c860837b1de55d1` / `yetlet-cli-v0.4.1` |
| Signer / verifier / publication S=V=P | `e6a73d9ce13ba94d640339813e58ee1f110ed4b5` |
| Source / dossier PRs | [#98](https://github.com/bohewu/yetlet/pull/98) and [#99](https://github.com/bohewu/yetlet/pull/99), merged; exact PR/master Rust, Docs and CodeQL CI PASS |
| Original unsigned Build | [Run 44017724892](https://github.com/bohewu/yetlet/actions/runs/44017724892), attempt 1, PASS |
| Independent encrypted Build readback | First Run `44018194331` failed metadata preflight; corrected [Run 44018677389](https://github.com/bohewu/yetlet/actions/runs/44018677389), attempt 1, PASS. Original failure retained |
| Prepare and Sign | [Run 44021862161](https://github.com/bohewu/yetlet/actions/runs/44021862161) and [Run 44024521001](https://github.com/bohewu/yetlet/actions/runs/44024521001), attempt 1, PASS |
| Independent signature Result Verifier | [Run 44024859783](https://github.com/bohewu/yetlet/actions/runs/44024859783), attempt 1, PASS |
| Owner-approved publication | [Run 44025334874](https://github.com/bohewu/yetlet/actions/runs/44025334874), attempt 1, terminal SUCCESS, no automatic retry |
| GitHub Release | ID `448163167`, `draft=false`, `prerelease=true`, exactly five public assets with original approved bytes |
| CLI crate | `yetlet-cli-0.4.1.crate`, 541,417 bytes, SHA-256 `ac449f2dc415abb6c44ca8ee9b2b14a3f7f587e7e6f9b636cf0f5945c30dff0e` |
| crates.io | Version 0.4.1 not yanked; registry index, CDN and GitHub downloads return identical source archive bytes and SHA-256 |
| Installed registry consumer | Actual `CLI-CONSUMER.json` PASS: crates.io CLI 0.4.1, no local path patch or forwarded credentials, six project profiles using SDK alpha.8, real Native Style and Chrome Browser Style/WASM/CSS runtime PASS |
| Original publication receipt | `PUBLICATION-RESULT.json`: `complete`; GitHub and Registry both `published_and_verified`, `automatic_retry=false` |

Published CLI `0.4.0` remains an immutable historical archive built against
SDK alpha.7. The Windows Ctrl+C cleanup correction from
[PR #97](https://github.com/bohewu/yetlet/pull/97) is included in the 0.4.1
source; original native MSVC evidence and earlier failures remain source-scoped.

This Experimental release is not Stable API, broad production/GPU
qualification, or an injected-inline-handler CSP negative test.
The post-publication original receipts and private evidence are preserved
under the owner-controlled
`~/.local/state/yetlet/cli-041-release-guard-20261117/` directory.
