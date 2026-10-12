# YT-020 — Private Integrity Preparation

**Date:** 2026-10-12
**Scope:** Original nine experimental `0.1.0` source-package candidates; Rust/Cargo 1.99.0, separate-host rebuilding, provenance drafts and mature verifier experiments.
**Task state:** YT-020 remains **Blocked** for activation. The final independent reviewer result is reported separately against the frozen diff. No release or production assurance is claimed.

**Later authorized delivery (2026-10-12):** The maintainer subsequently authorized committing this reviewed snapshot and continuing the Private follow-up. Native reviewer thread `01a12dab-b825-7ba0-a389-0757a5896306` returned PASS with no issues/followups, observed `gpt-6-astra` / effective `high`. Commit `9ec9b7b2e2291f649fd659bcbdb8e9fe17f7ad2f` retains the historical working-overlay evidence below. [Hosted rebuilding](YT-020-HOSTED-REBUILD.md) records the separately prepared immutable-source builder and exact delivery/activation boundary. Earlier unstaged/no-delivery statements below describe the original reviewed checkpoint; they do not cancel that later authorization or relabel its older evidence.

## Authority and source boundary

The maintainer authorized Private preparation, use of another Windows runner through WebCodex, Rust-version unification before first publication, and independent repository review. No staging, commit, push, tag, Yetlet signing request, package/Release publication or visibility/settings change is authorized here. The original Ubuntu checkout is retained. A pre-existing detached checkout was observed and left untouched; none was created or used by this task.

The baseline is `master` / local `origin/master` at `633db585f3173f82c512689a09fd9a2065db6ffe`, tree `d9d6c4abe08a25ca753647a9149d8e16a80bffa9`. PR #5 candidate `c7f21b4d620a1b6836fb52a94bf19434af059368` and the merge have equal trees. Required PR run `38144070219` used synthetic checkout `269da96ea62e9a3b0c1184a0b387373b1d0aad09`, also tree-equal; master run `38145022739` used `633db585`. Both original required jobs passed. Their stable lane observed Rust 1.99.0 (`b940084d7`), with native/WASM/Chrome regression coverage. These are retained baseline results, not hosted execution of this working diff.

Historical hosted qualification `38043565238` remains tied to `c38837835dac683b09194b475baab2d8eda5a875`. Older Rust 1.88, offline Docker, Windows GNU and Firefox results are not relabeled. YT-067's complete source scan remains pre-patch; its small remediation was independently reviewed. No new full scan, external audit, penetration test or certification is claimed.

## Unified Rust baseline and delivery gate

The maintainer selected Rust 1.99 before the first release to remove an unnecessary old-compiler constraint and align consumers, development, packaging and qualification. The official [Rust 1.99.0 release](https://blog.rust-lang.org/2026/10/01/Rust-1.99.0/) is dated 2026-10-01. Both builders observe Rust 1.99.0 (`b940084d7`, LLVM 23.1.1) and Cargo 1.99.0 (`5f94df478`).

- Root and isolated Redis/qualification manifests declare `rust-version = "1.99"`; `rust-toolchain.toml` pins `1.99.0`, minimal profile, Clippy and rustfmt. WASM remains an application opt-in target.
- Local MSRV YAML installs and runs `1.99.0`, names its check `Rust 1.99 MSRV`, and retains the existing test/doctest/eight-WASM checks. Stable and weekly/manual qualification lanes explicitly select `RUSTUP_TOOLCHAIN=stable` so the new development pin cannot freeze those lanes. Triggers, permissions, action pins, coverage and the stable required-check name remain unchanged.
- The compiler and two qualification Dockerfiles agree on Linux amd64 `rust@sha256:b36c246742b4d323472588f789601be34eb4af053255e2af53969078ee8dcec7` (Rust/Cargo 1.99.0, Debian 12). Nginx/Redis pins and application behavior are unchanged.
- Remote branch protection still requires `Rust 1.88 MSRV` and `Stable native, samples, and Chrome`. No remote setting changed. Future authorized delivery must migrate the former to `Rust 1.99 MSRV`, retain the stable check, and obtain actual required CI PASS without bypass or weaker protection. Local YAML is not proof of an active hosted gate.

This explicit pre-release decision changes no published patch: no version has been released. Users of earlier private preparation install 1.99 or newer. After publication, the selected `0.1.x` patch line preserves Rust 1.99; later increases follow [Compatibility Policy](COMPATIBILITY.md).

## Two-host source-package rebuild

Retained evidence is under ignored `target/yt020-integrity-prep-20261012/`; it is local development evidence, not published metadata. Both hosts independently obtain the same base Git archive and apply the two recorded working metadata overlays. Build/cache directories are fresh and host-local; archives are not copied from Ubuntu to Windows.

| Input | Exact identity |
|---|---|
| Base commit | `633db585f3173f82c512689a09fd9a2065db6ffe` |
| Base Git archive SHA-256 | `cbcfe22c8008ac66fb0374c5aee566a436d7a1241ab143cb3ee9cc966cb11b00` |
| Working `Cargo.toml` SHA-256 | `10857511488c65b515d4953360b101b628df94d07b9dd131507ad6e4b3ea62df` |
| Working `rust-toolchain.toml` SHA-256 | `b2c19d86958ed6810741488db5cd352856f303c53bcf0ea774e72f94ee4d201e` |
| Original `Cargo.lock` SHA-256 | `7260eef21ce05abbe90ebccc4b255b3da07cdf5cd0acac51fdbb90ee7b3d4b57` |
| Canonical mapping of 54 selected package-input hashes | `8e5799c319d626812daca22c90feba41a41d23d07ecd642af4d6d3506dc7afb7` |

Each owned archive extraction materializes the 18 candidate license aliases with the exact root license bytes, avoiding a Windows symlink privilege dependency. There is no `.git` in the packaging input and no `.cargo_vcs_info.json` in these archives; external input digests provide the explicit source binding. The two working overlays mean the input is **not an immutable release commit**. Adjacent documentation, workflow and isolated-provider changes are outside this nine-package snapshot and are not falsely represented by the base commit alone.

Each builder runs `cargo fetch --locked` into its own fresh Cargo home, then native-host `cargo package --locked --offline --target-dir <owned directory>` with repeated `-p` selectors for the original nine candidates. Cargo's temporary local registry verifies their unpublished interdependencies. Default workspace feature resolution, native host target, development verification profile, `SOURCE_DATE_EPOCH=1791726849`, four build jobs, commands and tool hashes are retained in each build record. Windows uses its task-local GNU toolchain plus retained w64devkit and `-C link-self-contained=yes`; omitting `--target` lets that flag reach host build scripts too. Linux and Windows native executables are not compared as distributables.

| Builder | Host target | Result |
|---|---|---|
| Ubuntu primary | `x86_64-unknown-linux-gnu` | Package exit 0; all nine extracted candidate archives compile |
| Windows secondary, Runner `ncut-bohe-c420c5de1cac4bd4` | `x86_64-pc-windows-gnu` | WebCodex Job `wc_job_hW6_8gK6yQeL2dPx`, terminal completed, exit 0; all nine compile |
| Raw source archives | Same selected input digest on both hosts | All nine SHA-256 values and byte sizes identical; license texts, normalized `0.1.0` identity and Rust 1.99 metadata inspected |

The Windows project remains on its original `maint/m10-qualification-20261010` / `3128d23591feee15ce976d7b83a56dcd587048d7`, clean with empty index. Fetching the base commit used `FETCH_HEAD` because that single-branch clone has no `origin/master`; no checkout/reset was needed. New toolchains/caches/evidence are task-local and Git-ignored; the existing user-default Rust 1.88 toolchain and global PATH were not changed.

These are physically separate hosts, caches and native package-verification builds, but both remain under the same maintainer/agent/WebCodex control. This is useful cross-host source-archive reproducibility preparation. It does **not** satisfy the [independently controlled trusted-release builder contract](RELEASE-SUPPLY-CHAIN.md#independent-rebuild-provenance-and-approvals), independent human approval or a trusted immutable release. GitHub-hosted rebuilding is a possible future execution surface under an approved release workflow; it is not performed here.

WebCodex closeout observes the original Windows checkout clean, no active/pending Jobs and no unresolved actionable failures, but returns a non-blocking `validation_source_unproven` warning: its generic ledger does not bind this external archive/overlay build to the checked-out branch. The retained build record explicitly checks all 54 input hashes and the archive/overlay identities. This warning is disclosed; the generic closeout is not claimed as source certification and does not justify a duplicate rebuild.

The initial Windows Rust 1.88 attempt `wc_job_7LkjxkmPZbcBLYaE` ended exit 101 at `-lgcc_eh` in host build scripts. Its explicit target flag prevented target-only linker flags reaching those scripts; it is not a Yetlet source failure or PASS. Task-local Rust 1.99 installation Job `wc_job_r7qfbREmB0pL4ee0` installed the toolchain, then ended exit 1 at Rustup's self-update check under the alternate Cargo home. The final build verified and used those installed compiler binaries directly; no replacement Job overlapped either terminal attempt. Initial Ubuntu Rust 1.88 packaging is retained separately, not relabeled as the final unified lane.

## Inventory, checksums and provenance drafts

Cargo-cyclonedx 0.5.9 generates 18 CycloneDX 1.5 JSON inventories in a disposable source copy: nine candidates each for Linux GNU and Windows GNU, default workspace feature resolution, `--describe crate --all --target <triple> --target-in-filename`. Both targets' transitive normal/build dependency closures match target-filtered locked Cargo metadata, including each root. In dependency order core/actions/reactive/html/dom/dom-keyed/server-axum/request-id/session, component counts are `1/12/1/2/2/2/59/60/69` for Linux and `1/12/1/2/2/2/58/59/68` for Windows. The inventory runs execute on Ubuntu, including the Windows-target metadata; they are not falsely claimed as Windows generator execution.

Cargo-deny 0.20.2 `check licenses sources` passes both target-filtered metadata inputs under unchanged `deny.toml`; copied/original lockfiles remain unchanged. Development-only dependencies, Rust sysroot/compiler internals, GNU/system/native libraries, JS/generated browser assets, WASM and other platform/feature variants are explicit omissions. No new blanket exception or framework dependency is introduced.

Two **unsigned** in-toto Statement v1 / [SLSA provenance v1](https://slsa.dev/provenance/v1) drafts bind the actual nine subject hashes, source/archive/lock/overlay/tool digests, native target, feature/profile/environment choices, build commands/records and inventory byproducts. Their preparation build type is `urn:yetlet:build:private-cargo-source-package:v1`; builder IDs are local preparation URNs. These are self-reported run records, not authenticated attestations, approved production build types or a SLSA-level claim.

The existing bounded checksum CLI creates and verifies the final flat `bundle/`: **34 artifacts** (nine archives, 18 inventories, two build records, source-input and comparison records, inventory coverage, two unsigned provenance drafts), plus `RELEASE-CHECKSUMS.json`. Both commands exit 0. This binds bytes but cannot authenticate itself. The earlier partial bundle and failed version-query attempt are retained separately as `bundle-incomplete/`; their checksum PASS is not the final bundle's provenance/completeness result. No validator framework, custom SBOM or cryptographic verifier is added.

## Verifier and public-fixture experiments

The design-pinned Cosign **v3.1.3** Linux amd64 binary has SHA-256 `4629c757b7618056f8ddd7e2625ae9fdd94c0372a65049520bc7d9df9efc7f71`. Bootstrap follows [official installation guidance](https://docs.sigstore.dev/cosign/system_config/installation/): mature Python TUF 7.0.1 / securesystemslib 1.5.1 refresh from the official pinned bootstrap root, authenticated `artifact.pub` SHA-256 `59ebf97a9850aecec4bc39c1f5c1dc46e6490a6b5fd2a6cacdcac0c3a6fc4cbf`, and OpenSSL verify of the official artifact-key signature (`Verified OK`). The tool remains task-local. No private signing key is created.

GitHub CLI 2.102.0 obtains trusted roots through its TUF client; retained original `trusted_root.jsonl` SHA-256 is `65ca537f6ed8a47fd0e560c421baa1f6c1efb8b25fc200d8c5c02c0e92eb2b9c`. Root files, fetched TUF metadata, tool versions/digests, upstream release asset hashes, exact commands and diagnostics are retained under `tools/`. Those observed preparation identities still require release approver acceptance before actual Yetlet activation; they are not floating trust inputs or silently approved signing policy.

Only already public upstream artifacts/bundles are used:

- Cosign's release binary keyless bundle verifies with exact official identity `keyless@projectsigstore.iam.gserviceaccount.com`, issuer `https://accounts.google.com`, pinned public roots and default certificate/transparency checks. This upstream identity is not GitHub OIDC.
- Public GitHub CLI `v2.102.0` Linux archive SHA-256 `bb766f710eef8ede859c18578c72c327597cd4c8a85b06001b1f3843c6019386` verifies under Cosign and GitHub CLI's local-bundle verifier. Exact issuer is `https://token.actions.githubusercontent.com`, repository `cli/cli`, workflow `.github/workflows/deployment.yml`, ref `refs/heads/trunk`, source `fc4b137cdef0a6bd28fd461b7cf9c84a5812a8cd`, predicate `https://slsa.dev/provenance/v1`. GitHub CLI additionally checks source/ref and rejects self-hosted runners. The upstream immutable workflow and tag source are retained alongside the fixture; expected policy is not accepted merely because the certificate claims it.
- Eleven Cosign negative cases return exit 1: wrong issuer, repository, workflow, ref, source SHA, predicate, trust root, changed subject bytes, damaged signature, missing transparency, and this valid foreign fixture under the planned Yetlet identity. No insecure ignore flag, wildcard identity or disabled claim check is used.

There are 14 successful expectation checks: three positive verifier cases and 11 expected rejections. Bootstrap/API/CLI harness errors are retained: missing local `ensurepip` was handled with a pinned task-local pip wheel; TUF 7 required explicit bootstrap bytes; GitHub CLI selectors were mutually exclusive and its custom-root input required original JSONL. Corrections address those observed invocation contracts, not Yetlet source or weaker verifier policy. Individual terminal failures are not represented as successful cryptographic verification.

No Yetlet artifact/digest was uploaded to an attestation/signing/transparency service. No Yetlet OIDC call, signing operation or release workflow was activated. These experiments qualify verifier invocation and rejection behavior on public fixtures; actual signed Yetlet provenance, issuer/repository/workflow/tag/source binding and trusted builder acceptance remain **Not verified**.

## Bounded validation and preserved state

Current Rust 1.99 workspace strict Clippy, formatting and the eight existing wasm32 package checks pass. Strict Clippy also passes for the isolated Redis provider and both b/b3 fixture manifests. The existing `test_qualify_multi_instance.py` passes all 18 checks; a read-only, network-disabled, automatically removed compiler container observes the new pinned image's Rust/Cargo/OS identity. That container is not a repeated multi-instance qualification run.

Retained master CI already executed the unchanged runtime source with Rust 1.99, including workspace tests/doctests, application/browser suites and WASM. New package builds and compiler/configuration checks cover this metadata/toolchain change. No broad native/browser/Docker suite or CI is rerun just to refresh source/date. Rust/application/browser source, all lockfiles, generated assets and nginx/Redis/application semantics are unchanged.

Final `python3 scripts/check_docs.py`, `git diff --check`, exact changed-file/index/source checks and resource observations are recorded with the independent review receipt. All working changes remain unstaged. Windows rebuilding is terminal; no owned server/browser/container or active build Job is left running. The reviewer uses the repository role without WebCodex, inspects actual final files/evidence and has no edit/stage/commit/push authority.

## Remaining activation gates

YT-020 remains **Blocked** for a clean reviewed immutable source/tag, actual registry name/ownership and resolution in dependency order, authorized protected-check migration/new hosted CI, approved release-equivalent target/feature inventories/licenses, independently controlled trusted-release rebuilding, actual signed Yetlet provenance/signature verification with approved tools/roots/builder/build type/exact identity/source policy, reachable private reporting/fallback and scanner activation, concrete maintainer/security acceptance and immutable promotion/rollback digests. Unsigned drafts and public verifier fixtures do not replace any gate.

YT-065 remains Done at automated native/desktop browser qualification scope; YT-066 remains Done at single-host Docker multi-instance and selected Redis-provider scope; YT-067 remains Done at internal pre-patch source/fix scope. Real IME/composition/caret, NVDA/VoiceOver speech, WebKit/Safari, iOS/Android, touch/soft keyboard, 200%/400% zoom and broad accessibility conformance remain **Not verified / optional future qualification**. Application TLS/ACL, backup protection, RPO/RTO, operational staffing, production support and SLA are outside this framework preparation. No devices, broad certification or new deployment work are required here.
