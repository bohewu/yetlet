# Yetlet CLI 0.3.0 — hosted release record (YT-248)

Date: 2026-11-10. **Published Experimental GitHub pre-release and crates.io
CLI 0.3.0; anonymous readback and Owner-authorized standalone registry consumer
PASS. The original publisher remains FAIL at consumer preflight; its receipts
and two local setup failures are preserved without a publication retry.**

## Authority and immutable product source

After accepting the [candidate](YT-248-CLI030-CANDIDATE.md), the Owner authorized
merging CodeQL PR #72 and publishing the independent CLI 0.3.0 release under the
[existing single-publication policy](RELEASE-PUBLICATION.md#each-version).
The closed cohort is schema 3 / `cli`, exactly `yetlet-cli`; the already published
SDK alpha.7 is only a consumed dependency. No SDK archive/tag is rebuilt,
replaced or uploaded. Reader Edition remains SDK alpha.6 / CLI 0.2.0.

| Binding | Selected value |
|---|---|
| Product source C | `6bfa8c4bc14bda6a5b466c776d4ccb8916f34bb6` |
| Immutable tag | `yetlet-cli-v0.3.0`, lightweight, remote readback exactly C |
| Source delivery | Candidate PR #71, followed only by the four CodeQL input-scope files in PR #72 |
| Exact C master CI | [Rust 42767458815](https://github.com/bohewu/yetlet/actions/runs/42767458815) and [CodeQL 42767459099](https://github.com/bohewu/yetlet/actions/runs/42767459099), both SUCCESS |
| Release documentation gate | `release_docs.py sync`: zero changes; `check`, workspace version check and `cargo +1.99.0 xtask check docs`: PASS |
| Tag Guard | Explicit strict/builtin plan and one-use acceptance; normal native pre-push PASS; used binding retained and retired through the canonical helper |

The main checkout and GPUI research worktree were preserved. The tag procedure
used a temporary standalone clone because the Guard does not support shared
linked-worktree metadata for that operation. No hook bypass, Runner permission
or branch-protection change was made.

## Original hosted build and source correspondence

| Binding | Actual value |
|---|---|
| Build / attempt | [42769438585 / 1](https://github.com/bohewu/yetlet/actions/runs/42769438585), SUCCESS including cleanup |
| Public artifact | `unsigned-preparation-42769438585-1`, ID `13741004048` |
| Public artifact ZIP SHA-256 | `d903453316dccdcdd61d3ad3c1e293d9777d2f0af34b990609adbdeac298de54` |
| Encrypted private artifact | ID `13741336486`, 56505553 bytes |
| Encrypted artifact ZIP SHA-256 | `99c545fd373b44083a4e2995ccf4f4b37e7ce2e83b690bf2e973d11e0394004e` |
| CLI archive | `yetlet-cli-0.3.0.crate`, 544259 bytes |
| CLI archive SHA-256 | `7ca1f84eec6a9c5c7b96536c8e241e273f36035148fc8b2f3603d6d4c228add0` |
| Artifact expiry | 2026-11-17; owner-host retention is not offsite backup |

The public artifact has exactly one CLI archive, original unsigned provenance
and public checksums. Its provenance has exactly that archive as its subject.
The normal hosted package contains 162 files, all byte-equal to the common files
in the accepted 163-file local candidate. Only the local candidate's Git-aware
`.cargo_vcs_info.json` is absent from the non-Git hosted package staging tree.
The dirty local archive's checksum is not relabeled as the formal archive.

This exact correspondence reuses the candidate's archive installation, six
registry-only SDK consumer variants, Native Style HTTP/CSS and Browser Style
Vite/WASM/Chrome computed-style/CSP evidence at their original scope. Independent
corrective implementation Reviewer PASS has zero issues/followups. The initial
credential-isolation finding, its focused fix and actual Chrome delta remain
retained. Third-party CLI lock entries are unchanged; the alpha.7 SDK checksum
matches its immutable published archive. This is not a new dependency audit.

The build SDK and embedded export snapshot are alpha.7. Generated apps instead
use the separate `sdk-compat-v1` policy: ordinary native/browser alpha.6 or
alpha.7, Style alpha.7, exact pins and per-project locks/receipts. Historical
CLI 0.1.0/0.1.1 → alpha.5 and CLI 0.2.0 → alpha.6 bindings remain immutable.
New Windows Style-template revalidation remains paused/unverified; earlier
Windows resolver MSVC evidence is not new Style/runtime qualification.

## Original custody and historical prepublication gates

Original protected custody readback
[42770085305 / 1](https://github.com/bohewu/yetlet/actions/runs/42770085305)
completed SUCCESS, including cleanup. Safe receipt artifact ID `13741915677`
has ZIP SHA-256 `41a3eefc21b92c9effec584ebc24cafb075baba6e87c96ace06f63cfc6c6b656`.
Its `complete`, private-manifest and local-decryption/inventory checks are true,
with original exit 0 and status `PREPARATION_SUCCESS`. It binds original C/tag,
build/attempt, recipient and ciphertext. This safe receipt is not itself builder
authentication, durable custody or release approval.

The existing `release-evidence-verify` Environment's single tag
policy selects exactly `yetlet-cli-v0.3.0`; its ID, automation policy, disabled
admin bypass and existing secret remain unchanged. Original encrypted evidence,
safe receipts, source/qualification/review records and failures are retained in
the owner-local external evidence run `yt248-cli030-publication-20261110`.
No formal private key or decrypted original evidence is downloaded locally.

The following gate list preserves the original prepublication checkpoint;
actual later outcomes are recorded below. This documents product C. A separate
independently reviewed docs-only delivery
will select S, distinct from C, after exact-head PR/master CI PASS. Freeze the
complete record using the actual successful build/readback, unsigned provenance,
S and private Draft Release ID `440021677`. Keep master at S through prepare/sign;
V/P may equal S if its source and live master remain unchanged. Do not rebuild
or repack the accepted archive.

Then use the existing protected no-OIDC prepare, exact frozen-input signing and
separate independent result verifier. Only a successful verifier can seed the
five-file public plan and one-package registry plan. Final GitHub Experimental
pre-release (not Latest), then crates.io publication and a fresh registry-only
installed CLI/SDK/Style consumer remain behind the Owner's exact
`release-publish` Environment approval. No failed write/signing job is retried
automatically. Current CLI publication authority remains 0.2.0 until actual
successful publication receipts exist; YT-248 remains In progress.


## Fixed signer and independent verification — 2026-11-10

PR [#73](https://github.com/bohewu/yetlet/pull/73) delivers separately reviewed
S=V=P `98dff7c7524238c4bc8a322a195b00194e85a83d`, tree
`3774c6a5467a30980c0639f02950d838b169e5d1`, distinct from immutable C.
Independent Reviewer PASS has zero issues/followups; strict/builtin staged,
exact-outgoing and normal hooks PASS. PR exact-head CI and actual master Rust
[42771178423](https://github.com/bohewu/yetlet/actions/runs/42771178423) / CodeQL
[42771178527](https://github.com/bohewu/yetlet/actions/runs/42771178527) PASS.
No package source, immutable tag or archive changes for S.

| Stage | Actual result |
|---|---|
| Prepare/freeze | [42771467801/1](https://github.com/bohewu/yetlet/actions/runs/42771467801), SUCCESS; 35 selected qualification entries, all six roles and original build evidence; original frozen ZIP/receipt retained |
| Signing | [42773167750/1](https://github.com/bohewu/yetlet/actions/runs/42773167750), SUCCESS; standing `reviewed-automation` policy, unchanged exact payloads, both real bundles and cleanup |
| Independent V | [42773396049/1](https://github.com/bohewu/yetlet/actions/runs/42773396049), SUCCESS; separate no-OIDC worker verifies one CLI subject and all 52 final private-manifest entries, then cleans plaintext/key |

Original unsigned provenance SHA-256 is `54f80bd828123c50515d691343ebab133b5bc63b69564af841edc14601c5e347`;
final private manifest SHA-256 is `d5bc88d3f168c313e2042a21a9c6afffcb472be0be1cf32367e25dbb692b5d60`.
The final Owner packet passes independent Reviewer with zero issues/followups;
its closed public/registry plans match the hosted no-secret preflight. The
short-lived encrypted Draft qualification asset was removed after verified
intake/frozen retention. No formal private key or full private plaintext was
read back to the owner workstation.

## Owner-approved publication and installed consumer — 2026-11-10

Owner `bohewu` approved `release-publish` Environment ID `25761919341` for
[publisher 42774193500/1](https://github.com/bohewu/yetlet/actions/runs/42774193500)
at the fixed P above. The
[CLI 0.3.0 release](https://github.com/bohewu/yetlet/releases/tag/yetlet-cli-v0.3.0)
is public at Release ID `440021677`, Experimental pre-release and not Latest,
with the exact original tag/C, notes and five approved files: CLI archive,
original unsigned provenance, public checksums, public Sigstore bundle and
reduced signing/publication approval. All asset names/sizes/digests and anonymous
full downloads match the frozen plan. No private evidence/manifest/bundle or
qualification asset is attached.

crates.io accepted exactly `yetlet-cli 0.3.0`, not yanked. Its index checksum and
anonymous archive download equal the frozen 544259-byte archive SHA-256
`7ca1f84eec6a9c5c7b96536c8e241e273f36035148fc8b2f3603d6d4c228add0`.
No SDK package was uploaded, archive repacked, signature repeated or tag moved.

The original publisher concludes **FAIL**, with `PUBLICATION-RESULT.json`
`status=incomplete`, `phase=registry-consumer`, and both GitHub/registry
`published_and_verified`. `REGISTRY-RESULT.json` retains
`registry_verified_consumer_pending`; its `consumer=not_attempted` is the
unchanged initial marker, not a claim that no consumer command ran. The actual
logs prove a successful fresh locked registry install, exact `yetlet 0.3.0`,
help/new-help, followed by offline-miss exit 125 before registry resolution.
Rust/Cargo probes reject the shared Runner tool path. This is an environment
preflight failure, not an archive or publication failure; the original workflow
is never rerun.

The original result artifact ID `13743193094` has ZIP SHA-256
`50c0ffaff7737ed3deac83355096e6406dd1afee5262e006295ff8d769f7f8de`. Its 17 original receipt/plan/log/lock
files and the failed job log are retained unchanged.

The Owner subsequently authorizes actual consumer verification and documentation
closeout. The bounded correction uses an owned private HOME-rooted Rust 1.99
copy and direct compiler binaries, without changing shared Runner permissions or
CLI probes. Consumer child processes use private umask 077. Two local supplements
pass install/default/exact native/browser resolution and atomic miss/repeated-new
checks, then encounter the same protected-directory refusal during Style build.
The child-only umask correction cannot protect a Cargo target already created by
the parent installation. Both failures and automatic private-workspace cleanup
are retained. After the required stop, the Owner explicitly authorizes creating
the task-owned target root with mode 0700 before install and one bounded
read-only supplement. No published CLI source or archive changes.

That existing canonical consumer now **PASS** from a fresh credential-free Cargo
home using `cargo install yetlet-cli --version 0.3.0 --locked`. It verifies the
actual executable/version/help, atomic offline miss, all six default/native/
browser/native-style/browser-style/offline cases, exact SDK pins, captured
metadata/provenance, app locks and repeated-new refusal. Both Style apps build
and run through the registry-installed CLI: Native SSR/CSS and Browser
Vite/WASM/Chrome computed styles, CSP/console and asset checks PASS. SDK alpha.7
is selected in this captured registry observation; dynamic defaults remain
policy/cache-dependent rather than a permanently fixed latest-version claim.
All root/stylegen locks are preserved, owned processes exit and ports close;
Chrome profile and the complete private compiler/build/consumer workspace are
removed. The separate supplement does not relabel the original workflow PASS.

Installed CLI lock SHA-256: `f5550c3e56756dc2fb91b3767efa30ea0ced4be21bd48c3d13237a4d7ca4f75f`.
Supplement result SHA-256: `f893d6b5e383df5adbb7e9a16ad7d91aa3730680dffb3638aabbe4a031bb1fa5`.
The result, original source/helper hashes, exact commands, locks, runtime/Chrome
receipts and cleanup live under `postpublication/registry-consumer-supplement/`,
with the successful bounded supplement in `private-target/`. Earlier failures
remain in their original parent and `corrected/` locations. Linux consumer
acceptance is complete; new Windows Style-template revalidation remains
paused/unverified. Reader Edition stays SDK alpha.6 / CLI 0.2.0, including the
site manifest, first-app guide and docs-site SDK manifest/lock. Current published
version labels and publication links derive from this accepted record; Reader
Edition labels continue to derive separately from `site/pages.toml`.

All raw records, original failures, immutable inputs, real signatures, public
readbacks, consumer output and reviews are retained in the existing external
`yt248-cli030-publication-20261110` evidence run and its `postpublication/`
subdirectory. GitHub artifacts expire beginning 2026-11-17; owner-host retention
is not offsite backup. The original 917360450-byte cache/temporary-clone cleanup
is preserved separately; this closeout's new cache cleanup follows verification.
