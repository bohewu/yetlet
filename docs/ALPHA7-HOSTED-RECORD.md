# Yetlet SDK 0.9.0-alpha.7 — hosted source/build/readback record

Date: 2026-11-09. Status: **C/tag/build and original encrypted-evidence readback PASS; S/V/P, signing, independent signing-result verification and publication remain pending**.

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

## Remaining release gates

1. Select a separately reviewed signer commit S, distinct from C, through a normal PR, independent Reviewer, Strict Commit Guard, exact-head CI, Merge and post-merge master CI. Keep master at S through prepare/sign.
2. Create the unpublished prerelease Draft and bind its actual Release ID. Complete the schema-2 record with actual build, custody verifier, S, unsigned provenance digest/size and Release ID.
3. Run the protected no-OIDC prepare, freeze the exact payloads, then the authorized automated signing workflow. Retain actual public/private bundle hashes, certificate/transparency identities and cleanup receipts.
4. Independently verify the actual signing outputs and all two package subjects/private-manifest entries at V. Select P (V/P may equal S if source and policy remain unchanged).
5. Finalize English Release Notes and the exact six-file GitHub / two-crate publication plan from actual verified outputs.
6. Stop before dispatching the `release-publish` workflow into its owner-protected Environment. No GitHub Release is published and no crates.io version is uploaded until explicit owner approval.

No signature, signed provenance bundle, independent signing-result receipt, final publication plan or public release exists at this checkpoint.
