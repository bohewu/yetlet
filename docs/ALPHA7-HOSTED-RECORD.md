# Yetlet SDK 0.9.0-alpha.7 — hosted release record

Date: 2026-11-09. **Current status: Published Experimental GitHub pre-release and two crates.io packages; owner-approved publication and exact registry-only consumer PASS.** Original build/readback checkpoint and prepublication gates below retain their historical state; see [the final publication receipt](#owner-approved-publication-and-registry-consumer--2026-11-09).

This is the two-package SDK cohort only: `yetlet`, then `yetlet-browser` (`schema=2`, `cohort=consolidated`). It does not include `yetlet-cli`. CLI remains `0.2.0`, its SDK registry pin remains `=0.9.0-alpha.6`, and the public reader edition remains SDK alpha.6 / CLI 0.2.0.

## Immutable product source C

| Binding | Actual value |
|---|---|
| C | `a9ec562eed5a0a08ab894a76a04fe859416a8ba1` |
| Tree | `7df26e82cd2fe00e7f92902dac8b5f834877012c` |
| Tag | `v0.9.0-alpha.7`, lightweight and exactly at C |
| Delivery | YT-246 PR #64 merged to `master`; all three merge-head master workflows passed |
| Strict tag Guard | `strict` / `builtin` PASS; native pre-push consumed the exact one-use acceptance binding; remote tag readback equals C |

The original checkout and its separate GPUI research worktree were preserved. Because the shared linked-worktree topology is not supported by the remote-tag Guard, tag creation used a temporary standalone clone on the same Ubuntu Runner with the same strict/builtin Guard and native hooks. No bypass or Runner permission change was used. The used Guard binding and remote readback are retained under the ignored verification directory.

## Source and release-document gate

At final master C:

- `python3 scripts/release_docs.py sync`: zero files changed.
- `python3 scripts/release_docs.py check`: PASS.
- `python3 scripts/sync_workspace_versions.py check`: PASS (52 manifests; 34 SDK path-dependency manifests).
- `cargo xtask check docs`: PASS (75 documents; 243 backlog items).
- `cargo xtask check samples`: PASS (13 samples).
- `git diff --check`: PASS; original master checkout remained clean.

The closed build record is schema 2 / consolidated, version `0.9.0-alpha.7`, source C. Its canonical build-stage digest is retained in `release-record-build.json`.

## Unsigned hosted build

| Binding | Actual value |
|---|---|
| Workflow | `Yetlet source-package release` |
| Run / attempt | `42589716426` / `1` |
| Result | SUCCESS; all steps and cleanup completed |
| Build artifact | `unsigned-preparation-42589716426-1`, ID `13660827643`, 225080 bytes |
| Build artifact ZIP SHA-256 | `3a0b4f721f1856257e0d15c068ed012ad055fd6de6462f599b2b506714bea07e` |
| Encrypted private artifact | `encrypted-private-a9ec562eed5a0a08ab894a76a04fe859416a8ba1-42589716426-1`, ID `13660413770`, 56249410 bytes |
| Encrypted artifact ZIP SHA-256 | `b23e3c109d620fa7779b569b0bbb170e57a8daef0b4122e7277789f37fcd4d97` |
| Retention | Both original artifacts were unexpired at readback; GitHub expiry was 2026-11-16 |

The unsigned allowlist contains exactly `PROVENANCE-UNSIGNED.json`, `RELEASE-CHECKSUMS.json`, `yetlet-0.9.0-alpha.7.crate` and `yetlet-browser-0.9.0-alpha.7.crate`. The in-toto/SLSA unsigned statement has exactly two subjects, and both subject SHA-256 values match the actual `.crate` files. `cargo xtask release checksums verify` passed with its explicit notice that this is unsigned local verification, not trusted provenance.

| Package archive | Bytes | SHA-256 |
|---|---:|---|
| `yetlet-0.9.0-alpha.7.crate` | 169595 | `7f63ac7ddd10dfba42704b54c2a2310b8251440cede638e51bbd5054abf6f1ea` |
| `yetlet-browser-0.9.0-alpha.7.crate` | 52559 | `5892542ba76e1d001903c3a745bfa70eb2a4cf756c70d7bf6cd9f0a110475154` |

Both package archives contain the exact selected package/version and their expected source files. Their bytes also match the previously inspected YT-245 local preparation archives; the YT-246 source delta does not modify either SDK crate tree. This identity reuse does not replace the fresh hosted C-bound build and custody readback above.

### Local byte-integrity and remote-run reconciliation — 2026-11-09

The retained files were freshly SHA-256-checked against the original hosted ZIP digests above. The exact original unsigned *raw* files (not their transport ZIP) also have these independent readback identities:

| File | Bytes | SHA-256 |
|---|---:|---|
| `PROVENANCE-UNSIGNED.json` | 5732 | `0326365e75162c516f1d6dcf091a97509364e5dc2430e4fcd67aa348d7a42d37` |
| `RELEASE-CHECKSUMS.json` | 603 | `fc9e548d7f27fee4b23ddcb9c95bbaf916edee111fa7cf371264adcde5af0bef` |

The unchanged original GitHub Actions run records both report `status=completed`, `conclusion=success`, `run_attempt=1` and `head_sha=a9ec562eed5a0a08ab894a76a04fe859416a8ba1`. The build and custody readback are two different run IDs; the readback alone does not authenticate the unsigned statement or prove a formal signature. These raw input digests may feed the later closed S-bound record only after its actual signer source and Draft Release ID are established.

## Original encrypted-evidence readback

| Binding | Actual value |
|---|---|
| Workflow | `Verify encrypted release evidence` |
| Run / attempt | `42591012250` / `1` |
| Result | SUCCESS; preflight, protected readback, upload and cleanup passed |
| Environment | `release-evidence-verify`, ID `25525950662` |
| Safe receipt artifact | `evidence-readback-42591012250-1`, ID `13661490996`, 684 bytes |
| Safe receipt artifact ZIP SHA-256 | `96c5e77c0fc789781f610a8299ba85d35e5d51930c78b1ccfaffcbd633910d51` |

The safe receipt confirms `complete=true`, `private_manifest_verified=true`, `local_decryption_and_inventory_verified=true`, `original_exit=0`, and `status=PREPARATION_SUCCESS`. It binds the original run/attempt, tag, C, workflow SHA, encrypted ciphertext digest, complete inventory digest and unchanged recipient digest. Its notice correctly limits the claim to local readback; it is not builder authentication, durable custody or release approval. No private plaintext or formal private key was downloaded to the Ubuntu workspace.

The `release-evidence-verify` Environment previously allowed only the completed CLI tag `yetlet-cli-v0.2.0`. No queued, in-progress or waiting evidence-verifier runs existed during rollover. Its single existing custom tag policy was updated in place to exactly `v0.9.0-alpha.7`; the Environment ID, no-reviewer automation policy, `can_admins_bypass=false`, custom-policy mode and `AGE_IDENTITY` secret name were preserved. `release-sign` and `release-result-verify` remain master-only. `release-publish` remains master-only with the sole required reviewer `bohewu` and its `CRATES_IO_TOKEN` secret; no token value was read.

## Historical pre-publication gates — 2026-11-09
The following were genuine outstanding gates at the original C/readback checkpoint. All are now completed in the separately recorded owner-approved publication stage below; they are not currently pending.

1. Select a separately reviewed signer commit S, distinct from C, through a normal PR, independent Reviewer, Strict Commit Guard, exact-head CI, Merge and post-merge master CI. Keep master at S through prepare/sign.
2. Create the unpublished prerelease Draft and bind its actual Release ID. Complete the schema-2 record with actual build, custody verifier, S, unsigned provenance digest/size and Release ID.
3. Run the protected no-OIDC prepare, freeze the exact payloads, then the authorized automated signing workflow. Retain actual public/private bundle hashes, certificate/transparency identities and cleanup receipts.
4. Independently verify the actual signing outputs and all two package subjects/private-manifest entries at V. Select P (V/P may equal S if source and policy remain unchanged).
5. Finalize English Release Notes and the exact six-file GitHub / two-crate publication plan from actual verified outputs.
6. Stop before dispatching the `release-publish` workflow into its owner-protected Environment. No GitHub Release is published and no crates.io version is uploaded until explicit owner approval.

No signature, signed provenance bundle, independent signing-result receipt, final publication plan or public release exists at this checkpoint.

## Owner-approved publication and registry consumer — 2026-11-09

Owner `bohewu` approved the protected `release-publish` Environment (ID `25761919341`) for the single existing publisher [Run 42625356684/attempt 1](https://github.com/bohewu/yetlet/actions/runs/42625356684). Its exact source P was the independently reviewed and PR-merged signer/verifier commit **S=V=P `4e358144e00f287d34725b8e118db6debb53c69e`** (tree `a915a66d051de2acec7dfb8f8a8201021979fc7f`), distinct from immutable product C. PR [#65](https://github.com/bohewu/yetlet/pull/65) passed independent Reviewer `pass`, strict/builtin staged/outgoing and normal hooks, exact-head PR Rust/CodeQL CI, squash merge with the identical reviewed tree, and actual post-merge master Rust/CodeQL CI. No SDK product inputs, tag or original build archives were changed for S.

The original source-bound builder [42589716426/1](https://github.com/bohewu/yetlet/actions/runs/42589716426) and isolated original encrypted-evidence readback [42591012250/1](https://github.com/bohewu/yetlet/actions/runs/42591012250) passed at C. The separately reviewed 33-entry supplemental qualification selection passed protected preparation [42610509040/1](https://github.com/bohewu/yetlet/actions/runs/42610509040); its `signing-inputs` artifact ID `13669622834` was frozen with an unchanged original public provenance SHA `0326365e75162c516f1d6dcf091a97509364e5dc2430e4fcd67aa348d7a42d37` and final full private manifest SHA `a947cc86335c703d7823745e59df1b1d4d53d143ee2fd641ef702a1e96b4066c`. Formal signing [42622287058/1](https://github.com/bohewu/yetlet/actions/runs/42622287058) passed with unchanged frozen payloads and OIDC/certificate bindings; the separate no-OIDC independent verification [42622622478/1](https://github.com/bohewu/yetlet/actions/runs/42622622478) returned `PASS`, verifying **both archive subjects and all 52 final private-manifest entries**. Signing and independent result workspaces completed required secret/plaintext cleanup. Prior Windows MSVC/native SSR and wasm32 compile evidence was reused only at its accepted original scope; no new platform certification was invented.

The owner-approved publisher's exact [Experimental GitHub SDK alpha.7 pre-release](https://github.com/bohewu/yetlet/releases/tag/v0.9.0-alpha.7) is public (`draft=false`, `prerelease=true`; not Latest) at Release ID `439046998`, targeted at immutable C, with exactly **six** approved assets: `yetlet-0.9.0-alpha.7.crate`, `yetlet-browser-0.9.0-alpha.7.crate`, `PROVENANCE-UNSIGNED.json`, `RELEASE-CHECKSUMS.json`, `provenance.sigstore.json` and reduced `SIGNING-APPROVAL.json`. No encrypted private manifest/bundle, private qualification input, identity or raw evidence was made public. The temporary encrypted Draft qualification attachment was removed before publication. Owner approval identity is present in the original publisher's GitHub Environment approvals; preflight and publish jobs both completed `success` without an automatic retry.

Both original frozen packages are now public, not yanked, and accepted by crates.io in dependency order: [yetlet 0.9.0-alpha.7](https://crates.io/crates/yetlet/0.9.0-alpha.7) (169595 bytes, SHA-256 `7f63ac7ddd10dfba42704b54c2a2310b8251440cede638e51bbd5054abf6f1ea`) then [yetlet-browser 0.9.0-alpha.7](https://crates.io/crates/yetlet-browser/0.9.0-alpha.7) (52559 bytes, SHA-256 `5892542ba76e1d001903c3a745bfa70eb2a4cf756c70d7bf6cd9f0a110475154`). Each registry index digest and downloaded archive SHA-256 matched the original frozen archive; no package regeneration was used. The publisher's fresh credential-free registry-only consumer resolved both exact alpha.7 versions with no path patches and ran `cargo +1.99.0 check --locked` successfully. This proves the bounded package/consumer release, not general runtime/browser/platform certification.

The original publisher result artifact ID `13679391761` (ZIP SHA-256 `8240f3eaafdd2dabac77d9d6441de4549311fc98263190587d0fd48533c6fb50`) contains `PUBLICATION-RESULT.json` (`status=complete`, `github=published_and_verified`, `registry=published_and_verified`, `automatic_retry=false`), `GITHUB-PUBLICATION.json`, `REGISTRY-RESULT.json` (`status=complete`, both archives `verified`, consumer `PASS`), both final plans and the registry consumer lock (SHA-256 `6beb63e6f8c879781e97d049f9cf439e6dfc058f5e43abae454bf8b1c37e53aa`). The publication request digest is `e3e90cc38dca0c508d487590f46958bcd62200739c2c67495d5e6ba47ee040c2`; registry plan digest `2b97034928900b483f138a22c4abe0db6da3134ece01001ce64fa5afdffe9843`; public plan digest `e2d4931bfd18953d45132a9c69f90af07de1b329a99743e5f326f1c8f0e7a420`. Original and final encrypted evidence, receipts and failure histories remain retained in the owner-host evidence store. Actions artifacts expire beginning 2026-11-16; owner-host retention is not offsite backup.

**Public documentation timing:** alpha.7 is now the accepted published SDK. The independently published CLI is **still 0.2.0** with its deliberate registry SDK pin `=0.9.0-alpha.6`; it is not a new CLI 0.3.0 release. The separately pinned site/first-app reader edition in `site/pages.toml` remains SDK alpha.6 / CLI 0.2.0 until an explicit reader promotion. Publish-version documentation may now synchronize from this accepted receipt; that documentation delivery is a separate reviewed Guard/PR/CI/Merge step, not a change to C/S/V/P or published package bytes.
