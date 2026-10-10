# M10e Verification — Release Supply Chain

**Date:** 2026-10-10
**Task:** YT-064 / M10e
**Status:** Done at design/preflight scope. Main-session validation and independent read-only reviewer `wc_job_CeUNYnHsO1yI_kBz` PASS (terminal exit 0; `gpt-6-astra`, high requested). This is not publication, signed provenance or reproducibility certification.

## Scope and written files

This slice writes only:

- `deny.toml`: cargo-deny default-deny license/source policy.
- `scripts/release_checksums.py`: bounded stdlib local checksum CLI.
- `scripts/test_release_checksums.py`: direct checksum acceptance tests.
- `docs/RELEASE-SUPPLY-CHAIN.md`: release design, inventory and pending gates.
- `docs/M10E-VERIFICATION.md`: this evidence record.
- Ignored `target/m10-remaining/YT-064-executor.json`: canonical executor handoff, plus owned temporary test fixtures removed at test completion.

No Rust/Cargo dependency, runtime abstraction, browser/server/container, settings, branch/worktree, signing, publication, deployment, commit, push or staging action is part of this slice. README, backlog and catalog updates belong to the orchestrator.

## Observed input identity

Initial observed branch: `master`; HEAD: `7c5d1267fd2ea3490738da105b476e271278ae7a`. The initial status contained 78 existing dirty paths, including the remaining-plan document (the handoff described 77 pre-existing reviewed changes). This is a dirty development snapshot, not the contents of HEAD or a release source/tag.

| Input | Observed SHA-256 |
|---|---|
| `target/m10-remaining/cargo-metadata.json` | `aa09f00fc74284246ef132c937f108f79c8df748da4c9b805ae9eb0181dce90b` |
| `Cargo.lock` | `7260eef21ce05abbe90ebccc4b255b3da07cdf5cd0acac51fdbb90ee7b3d4b57` |
| `target/m10-tools/bin/cargo-deny` | `e76c96220653e171ba219603bbf2a8f6274551eb0ff002ac700df8c98d9e6429` |
| `target/m10-tools/bin/cargo-cyclonedx` | `e0b06abbe7fb6d957769ffd725bc71ed024409a9f1c804a244e4824667d68b56` |

A direct stdlib read of the supplied metadata confirmed 131 packages, 28 workspace members, 103 registry dependencies, nine candidates, 19 `publish = false` samples and the candidate path-only dependency blockers. Registry sources are exclusively `registry+https://github.com/rust-lang/crates.io-index`. The design lists all candidates and nonpublishable package names. The minimal policy accepts required MIT/Apache/BSD/Unicode terms without adding unused licenses solely because they occur in OR alternatives.

## Executed focused evidence

| Command | Actual result |
|---|---|
| `target/m10-tools/bin/cargo-deny --version` | Exit 0; `cargo-deny 0.20.2` |
| `target/m10-tools/bin/cargo-deny --help` | Exit 0; global metadata/config/workspace/locked/offline flags inspected |
| `target/m10-tools/bin/cargo-deny check --help` | Exit 0; `licenses` and `sources` checks confirmed |
| `target/m10-tools/bin/cargo-cyclonedx cyclonedx --version` | Exit 0; `cargo-cyclonedx-cyclonedx 0.5.9` |
| `target/m10-tools/bin/cargo-cyclonedx cyclonedx --help` | Exit 0; JSON/spec/target/features/transitive/Cargo-target options inspected; no locked flag advertised |
| `PYTHONDONTWRITEBYTECODE=1 python3 scripts/test_release_checksums.py` | Exit 0; 16 tests PASS, including subprocess CLI create/verify |
| `python3 scripts/check_docs.py` | Exit 0; 65 documents, 67 backlog items; docs, internal links, Markdown structure, whitespace and DAG PASS |
| `git diff --check` | Exit 0; no whitespace errors |

The tests exercise identical-input canonical bytes, manifest exclusion, no overwrite, artifact/hash tampering, missing/extra files, unsafe manifest paths, self-reference, duplicate entries/keys, malformed manifest fields, file/manifest/directory/ancestor symlinks, traversal, directories/FIFO rejection, empty input, count/size/manifest limits and nondisclosing failure output. Tests create only owned temporary directories beneath `target/m10-remaining/` and remove them. No background resources were started.

The final read-only status/hash comparison found all 77 pre-existing reviewed dirty files byte-identical to the executor baseline. A concurrent change was observed only in the separate remaining-plan document; this executor did not write that file. The only five new nonignored paths are the five deliverables listed above. Branch/HEAD remained unchanged, no staged paths were present, and no owned `checksum-test-*` fixtures remained.

## Main-session executed validation

After the executor Job `wc_job_Y9RTho2cXqI4Nu0o` (requested `gpt-6.1-sol`, high, workspace-write) completed, the main session retained these actual results:

- Cargo-deny 0.20.2, actual unfiltered 131-package workspace metadata, locked/offline policy: **licenses ok, sources ok**, exit 0. Log: `target/m10-remaining/license-source-check.log`. The 16 checksum tests also independently passed.
- Cargo-cyclonedx 0.5.9 generated **41 CycloneDX 1.5 JSON SBOMs**, `--describe all-cargo-targets --all --all-features --target x86_64-unknown-linux-gnu --target-in-filename`, from a disposable source artifact copy under `target/m10-remaining/sbom-input/`. This is not a Git worktree. Offline Cargo resolution was enforced; all copied source hashes and both copied/original Cargo.lock were unchanged.
- `target/m10-remaining/sbom-result.json` binds generator paths/output hashes to the source snapshot, original lock, baseline HEAD and native/all-feature scope. The generated BOM files and `BUILD-IDENTITY.json` form a 42-file unsigned local artifact bundle. Actual checksum `create` and `verify` both exited 0.
- No WASM, other-platform, Rust sysroot, native/system-library contents or artifact-provenance coverage is claimed by this native SBOM experiment. No independent rebuild/signature experiment ran. All results are explicitly development-only for a dirty source snapshot.
- The initial install command used the wrong placement of `--version` for cargo-cyclonedx after both installs had already succeeded; only the diagnostic command was corrected to `cargo-cyclonedx cyclonedx --version`. No package reinstall or source workaround was needed.
- Current runtime baseline: 196 workspace Rust tests, five doctests, format and strict Clippy passed. No runtime or Cargo input was changed by YT-064.

## Pending main-session validation and acceptance

The [design](RELEASE-SUPPLY-CHAIN.md) contains proposed commands derived from the actual installed help output. This executor did **not** run cargo-deny license/source checks, generate an SBOM, perform release builds or independent rebuilds, install a verifier, verify signatures/provenance, sign, or publish. Main-session results must be added with actual exit status/logs, snapshot/lock/tool identity, output digests, target/features/dependency coverage and exclusions. An SBOM is inventory, not proof of artifact origin, trust or reproducibility.

The focused docs/whitespace and preservation checks above passed; broader main-session catalog/baseline closeout remains with the orchestrator. Independent implementation reviewer Job/result is Pending. Do not mark YT-064 Done before its main validation and reviewer gate. YT-020 remains Blocked regardless of a future design/preflight PASS.

Open release decisions: coordinated publishable dependency versions, supported artifact/target/feature matrix, clean tag/source identity, independent rebuild infrastructure, signing workflow/ref/identity, pinned verifier binary digest/trust roots, named human approvers and incident promotion/revocation controls. These are release activation gates; this slice adds no mandatory runtime dependencies or custom trust framework.

**Independent implementation review:** `wc_job_CeUNYnHsO1yI_kBz` returned `overall_status=pass`, zero issues and zero required followups. Main validation and review close YT-064 as the release-design/preflight task. Earlier executor pending statements are historical handoff observations only. Signing, clean release artifacts, independent rebuild and external audit remain separate gates.
