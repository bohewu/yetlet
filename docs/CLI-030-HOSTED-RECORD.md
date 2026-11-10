# Yetlet CLI 0.3.0 — hosted release record (YT-248)

Date: 2026-11-10. **Checkpoint: original unsigned build and original custody
readback PASS. Signing, independent result verification and Owner-gated
publication remain outstanding. This is not a publication receipt.**

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

## Custody, signer selection and remaining gates

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

This documents product C. A separate independently reviewed docs-only delivery
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
