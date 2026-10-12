# YT-020 — Private Release Preparation

**Date:** 2026-10-11
**Scope:** Preparation for an experimental `0.1.0` release of the original nine library candidates.
**Task state:** YT-020 remains **Blocked** for public activation. Completing this private preparation and its independent review does not authorize publication.

**Later delivery and integrity update (2026-10-12):** This document retains the 2026-10-11 preflight snapshot below. It was subsequently reviewed and delivered through PR #5: candidate `c7f21b4d620a1b6836fb52a94bf19434af059368`, squash merge `633db585f3173f82c512689a09fd9a2065db6ffe`, equal tree `d9d6c4abe08a25ca753647a9149d8e16a80bffa9`. PR run `38144070219` tested synthetic checkout `269da96ea62e9a3b0c1184a0b387373b1d0aad09` with that same tree; master run `38145022739` tested `633db585`. Both original required checks passed. The maintainer then authorized separate Private integrity preparation, another Windows host through WebCodex, and Rust 1.99 unification before first publication. [Integrity preparation](YT-020-INTEGRITY-PREPARATION.md) records those new inputs and results. This follow-up has no commit/push/publication/visibility authorization; it does not relabel the older evidence or activate release trust.

## Authority and boundary

The maintainer authorized preparation in the original checkout, experimental `0.1.0`, the original nine library candidates, and a final independent reviewer. Commit, push, package publication, Release creation and visibility changes are not authorized. The repository remains Private. No tag, release workflow, signing identity, registry credential or external service is activated here.

The baseline is clean `master` at `a61f409627a7b9004338a96b02d0ded91ceb2f51`, the squash merge of maintainer PR #4. Required PR run `38131601428` passed on `dda30de329b0347869a848544696343406e43a78`; automatic post-merge run `38132515711` passed both required checks on `a61f409`. Candidate and merged trees were identical. These runs precede this uncommitted preparation; they are not CI results for the changed manifests.

YT-060–YT-067 retain their recorded bounded completion. Historical hosted qualification `38043565238` still belongs to `c38837835dac683b09194b475baab2d8eda5a875`. It is not relabeled as execution on `a61f409` or this working snapshot. The YT-067 full scan remains bound to its pre-patch source; no new full scan or external audit is claimed.

**Later explicit policy decision (2026-10-12):** [Private trust and future registry policy](YT-020-PRIVATE-TRUST-POLICY.md) selects Cosign maintainer public-key preparation and future crates.io distribution. It defines custody/bootstrap, signed evidence, independent verification, rotation and actual registry gates. Formal release guarantees remain mandatory. This decision authorizes documentation and independent review only; no key/signing/token/publication or repository visibility change occurs.

## Selected artifacts

The proposed distributables are nine source package archives, in dependency order:

| Candidate | Version | Yetlet dependencies |
|---|---|---|
| `yetlet-core` | `0.1.0` | None |
| `yetlet-actions` | `0.1.0` | None |
| `yetlet-reactive` | `0.1.0` | None |
| `yetlet-html` | `0.1.0` | core |
| `yetlet-dom` | `0.1.0` | core |
| `yetlet-dom-keyed` | `0.1.0` | core |
| `yetlet-server-axum` | `0.1.0` | core, html |
| `yetlet-request-id` | `0.1.0` | server-axum |
| `yetlet-session` | `0.1.0` | server-axum |

The six dependent manifests add `version = "0.1.0"` while retaining their local `path`. Cargo interprets that requirement as compatible `0.1.x`, not an exact-version pin. Workspace package version, dependency versions/features and `Cargo.lock` remain unchanged. Rust source and HTTP/browser behavior are unchanged.

The 19 root-workspace examples remain `publish = false`. The separately isolated `yetlet-session-redis` remains experimental `0.0.0`, `publish = false`, outside the default workspace and outside this allowlist. Redis and Docker qualification fixtures, benchmark binaries, generated browser assets and Python harnesses are not distributables. Adding any of them requires a separate scope decision; Redis promotion is not a YT-020 prerequisite.

Every selected archive must contain both MIT and Apache-2.0 license texts and normalized package identity. Package dependencies must retain version requirements after normalization, with no local `path` in normal published dependencies. No blanket licensing exception or extra runtime dependency is added.

## Compatibility and maintenance policy prepared for activation

- All nine candidates use one coordinated `0.1.x` release train. `0.1.0` is experimental, not a 1.0 or production-stability promise.
- The original 2026-10-11 preparation selected Rust 1.88 patch preservation. Before any publication, the maintainer explicitly replaced that baseline with Rust 1.99 on 2026-10-12. A future `0.1.x` patch preserves documented API behavior and that selected minimum. An incompatible API change or later MSRV increase requires a coordinated minor release, explicit rationale, migration notes and the existing compatibility/reviewer/CI gates. MSRV does not rise silently in a published patch.
- Existing Stable Candidate, Experimental and upstream-boundary classifications remain intact; publication alone does not promote an Experimental API. [Compatibility policy](COMPATIBILITY.md) owns the detailed change discipline.
- Once separately activated, the latest approved `0.1.x` patch is the maintained experimental line, on a best-effort basis. Older patches are upgrade targets, not separately maintained branches. No security response deadline, SLA or production support is promised. Before publication there is no released supported version; private `master` remains the reporting target.

The proposed qualification/reporting scope is native Ubuntu x86_64, Windows MSVC x86_64, macOS ARM64 and the separately recorded Windows GNU lane; browser qualification is `wasm32-unknown-unknown` with observed Linux desktop Chrome/Firefox suites. The [qualification matrix](QUALIFICATION-MATRIX.md) retains the actual versions, dates and source identities. This target list accepts best-effort compatibility reports; it is not fresh execution of every lane, a promise for arbitrary OS/browser versions, or supported-device certification. Future changed behavior needs only the affected qualification required by the selected support scope.

Real IME/composition/caret behavior, NVDA/VoiceOver speech, WebKit/Safari, iOS/Android, touch/soft keyboards, 200%/400% zoom and broad accessibility conformance remain **Not verified / optional future qualification**. They do not block this bounded preparation. Applications selecting those environments qualify their own needs.

## Release authority and integrity gates

`bohewu` remains the sole maintainer and proposed release approver. The current zero-human-approval maintainer PR rule, collaborator-only PR creation, squash-only merging and both required checks remain unchanged. A new maintainer would trigger a governance review; no second human approver is invented for this slice. Independent technical review is separate from explicit maintainer release authorization.

The proposed tag is `v0.1.0`, bound to a future clean reviewed immutable commit. It is not created here. Record the tag-signing policy and verifier identity before tag approval; never move an approved release tag or substitute a rebuilt artifact during promotion.

The existing [supply-chain contract](RELEASE-SUPPLY-CHAIN.md) remains mandatory for actual activation: source/lock/toolchain/target/feature identity, archive and SBOM digests, license/source policy, independent rebuild comparisons, trusted provenance, signature verification, explicit digest approval and rollback to a previously approved acceptable digest. A private checksum bundle is unsigned and cannot satisfy those trust gates.

The proposed future GitHub OIDC release identity is repository `bohewu/yetlet`, workflow path `.github/workflows/release.yml`, issuer `https://token.actions.githubusercontent.com`, and the approved immutable tag ref `refs/tags/v0.1.0` bound to its reviewed commit. This is a planned exact allowlist, not an installed workflow or active permission. The provenance predicate/build type, trusted builder, approved Cosign binary digest/trust roots, and tag-signing mechanism still require concrete activation evidence. No wildcard identity, new key, token, OIDC permission or signing request is introduced.

Before publication, the maintainer records release/source and security acceptance of the concrete commit, source delta, support scope, findings/residual risks and artifact/SBOM/checksum/provenance/signature identities. Retained reviews and the old security-scan boundary remain explicit. Green PR CI is not that approval. Do not replace the independently controlled rebuild or signed verification with two builds using this checkout/cache.

## Public-transition checklist, not a settings change

The transition owner is `bohewu`. While Private, vulnerability reports use the existing private collaborator channel. Before public activation, verify a reachable private reporting route, including a concrete fallback if GitHub reporting is unavailable; do not invent a contact address.

After separately authorized visibility transition, enable and verify GitHub Private Vulnerability Reporting, secret scanning/push protection and CodeQL under the current account's supported features. Retain collaborator-only PRs, protected `master`, both required check names, read-only baseline workflow permissions and Dependabot alerts/security fixes. Record actual scanner coverage and findings disposition rather than treating enabled settings as a security assessment.

Observed private-state controls are recorded in the local verification receipt. No public-transition endpoint is enabled, no scheduled version-update configuration is added, and no existing workflow is changed by this preparation.

Application proxy/IdP/TLS/ACL configuration, backup protection, business RPO/RTO, operational staffing, durable Redis recovery, HA/cross-host/load/SLO and production deployment belong to separately selected application work. They remain unverified where applicable and are not prerequisites for these source packages or bounded YT-066 completion.

## Historical local preflight evidence (2026-10-11)

The working snapshot and generated artifacts are development evidence, not an immutable release candidate. Full command results, archive inventories/digests, metadata, tool identity and preservation checks are retained under ignored `target/yt020-private-prep-20261011/`. A separate final independent reviewer inspects this document, the actual diff and those receipts; acceptance is reported separately without modifying the reviewed source afterward.

| Actual command/check | Result and boundary |
|---|---|
| Cargo 1.88 batch `package --allow-dirty --locked --offline`, nine selected candidates | Exit 101 at unpublished `yetlet-core` registry resolution after creating actions/core archives; no successful package verification is attributed to this attempt |
| Cargo 1.99 batch `package --allow-dirty --locked --offline`, with `RUSTC` selected by `rustup which --toolchain 1.88.0 rustc` | Exit 0; all nine archives packaged and compiled from extracted contents with Rust 1.88.0, default features, native `x86_64-unknown-linux-gnu`, using Cargo's temporary local registry |
| Archive inspection against actual inputs | Nine normalized `0.1.0` identities/MSRV/license expressions, both license texts, retained compatible dependencies without published path entries, and byte-identical packaged Rust source PASS |
| `cargo +1.88.0 metadata --locked --offline --format-version 1` | Exit 0; 131 packages, 28 root-workspace members, original `Cargo.lock` unchanged |
| Cargo-deny 0.20.2, original `deny.toml`, locked/offline actual metadata, `check licenses sources` | Exit 0; licenses ok, sources ok; no policy change/exception |
| Cargo-cyclonedx 0.5.9, CycloneDX 1.5 JSON, `--describe crate --all --target all --target-in-filename` | Exit 0 in a disposable copy of actual tracked working inputs; generator traversed 28 workspace members, with only the nine candidate inventories retained in the bundle. Copied inputs and both original/copied lockfiles stayed unchanged |
| Candidate inventory comparison with actual Cargo metadata | Transitive normal/build dependency closures, including each root, contain 1/40/1/2/22/22/62/63/72 components in the table's candidate order; each inventory matches with no missing/extra component |
| Existing checksum CLI `create` then `verify` on the private bundle | Both exit 0; nine `.crate` files, nine candidate SBOMs and `BUILD-IDENTITY.json` bound by unsigned `RELEASE-CHECKSUMS.json` |
| Read-only crates.io API lookup on 2026-10-11 | All nine candidate names returned HTTP 404. This is an observed lookup, not reservation, ownership, a future availability guarantee or registry dependency resolution |

Live private-state observation retained Private visibility, collaborator-only PRs, squash-only merging, the unchanged protected branch and both exact required checks. Dependabot alerts/security fixes remain enabled. PVR lookup returned 404, secret scanning reported disabled (404), and code scanning reported not enabled (403). No tags or Releases were present; these GET-only observations changed no settings.

At this checkpoint, current Cargo's native multi-package staging avoided Cargo 1.88's registry-resolution limitation without weakening dependencies, adding a custom registry or increasing the then-selected consumer baseline. The later explicit Rust 1.99 decision is separate. The successful historical command packages all nine candidates together. It is not nine real registry uploads or proof that unpublished dependencies can already resolve from crates.io.

The package bundle records nine manifest digests, the original lock digest `7260eef21ce05abbe90ebccc4b255b3da07cdf5cd0acac51fdbb90ee7b3d4b57`, Rust source hashes, normalized dependency entries, archive/SBOM digests and actual Cargo/compiler/generator identity. The local receipts also record tool binary, policy and metadata digests. Cargo marks the six changed packages dirty; its clean VCS markers for the other three packages do not make the overall working snapshot immutable.

The SBOM scope is default workspace feature resolution and the union of Cargo dependency targets, not nine independently built platform/feature variants. Development-only dependencies, Rust sysroot, tool binaries, native/system libraries and JS/generated browser assets are not silently claimed as build inventory. No packaged WASM build, independent builder, signed provenance or signature verification ran; selected-target release-equivalent inventory remains an activation gate.

Existing runtime/browser/Docker suites are retained; they are not rerun merely to refresh a date or SHA. Packaging and inventory checks address the changed publication metadata. Final document/whitespace, source-preservation and independent-review evidence is reported separately. All changes remain unstaged and uncommitted.

## Remaining YT-020 blockers

Private preparation can be accepted while YT-020 stays Blocked. Actual activation still requires a clean reviewed immutable source/tag; registry name/ownership and real registry resolution in dependency order; selected-target release inventory and license checks; independent rebuild, provenance and signed verification; a reachable public private-reporting route and verified scanner controls; and explicit maintainer approval of the concrete digests and public transition. No success in this document authorizes those external actions.
