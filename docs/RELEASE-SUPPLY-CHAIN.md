# Release Supply-Chain Design

**Task:** YT-064 / M10e
**Original M10e status:** Design and local preflight accepted; independent reviewer `wc_job_CeUNYnHsO1yI_kBz` PASS. Actual experiments are recorded in M10e verification. Release activation, signing and independent rebuild are not performed; YT-020 remains Blocked.

**Private preparation update (2026-10-11):** The maintainer selected experimental `0.1.0` and the original nine library candidates. [YT-020 preparation](YT-020-PRIVATE-RELEASE-PREP.md) records current metadata, source-package preflight, prepared support/release policies and remaining activation gates. It preserves this integrity contract and does not authorize a tag, signing, publication or visibility change. The M10e experiments below retain their original source identity.

**Integrity preparation update (2026-10-12):** [Private integrity preparation](YT-020-INTEGRITY-PREPARATION.md) records Rust/Cargo 1.99.0, byte-identical nine-crate source archives rebuilt on separate Ubuntu/Windows hosts with fresh caches, target-specific inventories, unsigned SLSA v1 drafts, and public upstream signature/provenance verification experiments. Both hosts remain under shared maintainer/agent control and use recorded uncommitted metadata overlays. This does not satisfy independently controlled trusted-release rebuilding, actual Yetlet signing, immutable source/tag approval or publication. The existing release contract below remains mandatory.

**Later Private follow-up (2026-10-12):** The maintainer authorized committing the reviewed preparation and continuing delivery/Private rebuilding. [Hosted rebuild preparation](YT-020-HOSTED-REBUILD.md) records the first-head required CI and completed protected-check migration, a bounded manual builder with a separately operated GitHub execution environment, and actual execution evidence retained separately. It retains unsigned-provenance limits and the existing transparency/signature contract; Private attestation availability or a provider without a transparency log cannot silently waive that release gate. No public activation or signing policy is selected implicitly.

**Explicit Private trust decision (2026-10-12):** The maintainer selected [Cosign public-key preparation and future crates.io distribution](YT-020-PRIVATE-TRUST-POLICY.md). This documentation-only mode authenticates maintainer-approved bytes after separately authorized execution; it does not establish OIDC identity, certificate/chain, transparency/inclusion or authenticated builder evidence and does not replace any formal release guarantee below. Actual keys, signatures, registry activation and visibility changes remain unperformed.

**Later authorized Private execution (2026-10-12):** [Private signing verification](YT-020-PRIVATE-SIGNING-VERIFICATION.md) records a dedicated encrypted key, public fingerprint, three actual preparation signatures, separate public-only verification and six expected negative outcomes. Formal log-required verification rejects the same no-log bundle. This updates Private execution status only; the release contract below, historical evidence identities and all formal guarantees remain mandatory. No push, tag, publication or visibility change occurred.

## Current preparation source boundary (2026-10-12)

The [T08 preparation record](YT-020-PRIVATE-RELEASE-PREP.md#current-preparation-boundary--t08--yt-068-2026-10-12) distinguishes historical signed source `8e557e760407a3d4b3f5a8b8f833748946be1387`, policy HEAD `abb2cce1253afe1d87a97e0a50f3f47efffe7280` and the current dirty research/documentation checkout. The current root manifest/lock and nine newly discovered package READMEs change future package/evidence inputs. T07 passed nine package file listings; those checks do not approve archives or provide current-source reproducibility, inventory, signature or registry assurance. The historical three signatures, 37 artifacts plus manifest and public-only/negative-case PASS results retain their old byte bindings.

A separately approved immutable current-source candidate still needs package preflight, rebuild/provenance, selected target/feature SBOM/license evidence and signing verification under separate authority. This documentation reconciliation performs none of those operations. The complete formal OIDC/certificate/chain/transparency/inclusion/builder/source/approval and immutable promotion contract below is unchanged; YT-020 remains Blocked. A later local commit is not release activation. The independent delivery gate/result is recorded separately in the run; documentation-stage checks do not attest that gate.

## Source and package identity

Use the [coordinated release train](COMPATIBILITY.md): all nine Yetlet library candidates share one workspace version, currently `0.1.0`. No crate is authorized for publication. A future release requires a clean reviewed immutable commit, an approved immutable tag bound to that commit, an unchanged recorded `Cargo.lock` digest, and package archive digests. Reject dirty trees, moved tags, unreviewed source and lockfile drift at the release boundary.

Record exact Rust/Cargo versions (`rustc -vV`, `cargo -V`), tool binary versions and digests, host/build target triples, package/Cargo target, default-feature state, explicit features, profile, relevant build environment and complete commands. Browser artifacts also identify the pinned wasm-bindgen/loader inputs. Hash the reviewed source snapshot and release evidence; HEAD alone cannot identify uncommitted files.

The retained M10e `target/m10-remaining/cargo-metadata.json` contains 131 packages: 28 workspace members and 103 crates.io registry packages, with no Git dependencies. Its root-workspace packages are `0.1.0` and dual licensed `MIT OR Apache-2.0`; the later separately isolated Redis provider is not a publication candidate. The candidate order below is a dependency order, not a publishing script:

| Candidate | Yetlet dependencies | Publication prerequisite |
|---|---|---|
| `yetlet-core` | None | Reviewed archive and release approval |
| `yetlet-actions` | None | Reviewed archive and release approval |
| `yetlet-reactive` | None | Reviewed archive and release approval |
| `yetlet-html` | core | Versioned core dependency |
| `yetlet-dom` | core | Versioned core dependency |
| `yetlet-dom-keyed` | core | Versioned core dependency |
| `yetlet-server-axum` | core, html | Versioned core/html dependencies |
| `yetlet-request-id` | server-axum | Versioned server-axum dependency |
| `yetlet-session` | server-axum | Versioned server-axum dependency |

At the M10e checkpoint, the last six candidates had path-only Yetlet dependencies whose metadata version requirement was `*`. Authorized YT-020 private preparation now adds compatible `0.1.0` requirements while preserving local paths and the original lockfile. Cargo batch-package verification can exercise unpublished dependencies in its temporary local registry; it does not establish name ownership, availability or resolution from the real crates.io registry. Those remain actual activation gates in dependency order. A local build, dirty package archive or `cargo package --list` alone cannot prove registry publication readiness. M10e itself changed no manifest or lockfile.

The 19 packages with `publish = false` are `yetlet-auth-notes`, `yetlet-cart-server`, `yetlet-dashboard-counter-island`, `yetlet-dashboard-filter-island`, `yetlet-file-server`, `yetlet-forms-validation`, `yetlet-hello-world`, `yetlet-keyed-bindings`, `yetlet-middleware-lab`, `yetlet-modular-shop`, `yetlet-multi-island-dashboard`, `yetlet-render-bench`, `yetlet-rest-api`, `yetlet-search-island`, `yetlet-sqlite-crud`, `yetlet-sse-notifications`, `yetlet-todo-browser`, `yetlet-todo-server` and `yetlet-todo-ui`. Their artifacts may be qualification fixtures; they are not publication candidates.

## License and source gate

`deny.toml` targets cargo-deny **0.20.2**. It allows MIT, Apache-2.0, BSD-3-Clause and Unicode-3.0, plus the owner's explicit [YT-204 approval of Zlib, BSL-1.0 and ISC](YT-204-PACKAGE-SECURITY-COVERAGE.md#authorized-license-policy-followup--2026-10-27). It checks development/build dependencies and sets private-package ignore to false. All other licenses remain denied unless the checker can satisfy an alternative expression with an allowed license. At the original preflight, observed LLVM-exception, BSL, LGPL and Unlicense alternatives did not require additional approval when MIT or Apache-2.0 satisfied the expression. Current AND expressions still require every obligation; retain applicable notices in distributed artifacts.

This is a policy-wide SPDX allowlist for the dependency graph checked by this configuration, including future dependencies with these license IDs; it is not a package/version-specific exception. It does not relicense Yetlet or third-party code, authorize publication, or waive copyright, license-text or modified-source notices. `BSL-1.0` means Boost Software License 1.0. Package-specific terms and eventual source/binary/browser-bundle contents still govern distribution obligations; see the [YT-204 scope summary](YT-204-PACKAGE-SECURITY-COVERAGE.md#license-scope-and-distribution-obligations).

Unknown registries and Git sources are denied; only the crates.io registry is accepted. Workspace paths belong to the reviewed source snapshot, not a registry trust exemption. Do not suppress failures, add blanket exceptions or change package licenses to obtain a pass. A future dependency change requires renewed license/source review. Vulnerability audit remains the separate existing RustSec gate.

The main session can run this command against the retained actual metadata (not executed by this slice):

```bash
target/m10-tools/bin/cargo-deny --locked --offline --workspace \
  --metadata-path target/m10-remaining/cargo-metadata.json \
  --config deny.toml check licenses sources
```

For release, regenerate metadata with the same target/features as each build; the unfiltered workspace check is additional coverage, not proof of every feature/target configuration. Preserve exit status, diagnostics, counts, input/policy/tool digests and exact coverage. The installed `--help` confirms the options above. See the upstream [license policy](https://embarkstudios.github.io/cargo-deny/checks/licenses/cfg.html) and [source policy](https://embarkstudios.github.io/cargo-deny/checks/sources/cfg.html).

## Tool-generated SBOM and artifact binding

Use [cargo-cyclonedx 0.5.9](https://github.com/CycloneDX/cyclonedx-rust-cargo/tree/cargo-cyclonedx-0.5.9), not a custom SBOM/SPDX implementation. Generate CycloneDX JSON per package/Cargo target and build platform with the exact build features. Request transitive dependencies with `--all`; retain build dependencies by omitting `--no-build-deps`. Record whether development dependencies occur in generated output, compare coverage with Cargo metadata, and list exclusions explicitly. Rust sysroot, tool binaries, native/system libraries, JS glue and external deployment assets need separate inventories when not represented; do not silently treat the Cargo SBOM as covering them.

The installed CLI writes beside package manifests. For a dirty local experiment, the main session must prepare an isolated copy of the actual workspace inputs under `target/m10-remaining/sbom-input/`, retaining Cargo.lock and a source-digest record. It must not run the generator against the live workspace and write new files outside this slice's allowlist. The following inspected-help command is a proposed all-features native inventory experiment, not a release-equivalent build or an executed result:

```bash
target/m10-tools/bin/cargo-cyclonedx cyclonedx \
  --manifest-path target/m10-remaining/sbom-input/Cargo.toml \
  --format json --spec-version 1.5 --describe all-cargo-targets \
  --all --all-features --target x86_64-unknown-linux-gnu --target-in-filename
```

No `--locked` option is advertised by this generator: compare lockfile digests before/after and reject drift. Repeat separately for selected `wasm32-unknown-unknown` Cargo targets and the actual release feature sets before claiming browser coverage. Native inventory cannot stand in for WASM or other platforms. Retain exact output names/digests, generator identity, source/lock identity, selected dependencies/targets/features and omissions. SBOM generator timestamps/identifiers may vary; the local checksum manifest alone is deterministic.

Bind each distributed artifact's SHA-256 to its SBOM digest and build identity in release provenance. Put the artifact, SBOM and coverage/build-identity record into the same checksum bundle. An SBOM is an inventory, not proof that an artifact is reproducible, trustworthy, or even built from those dependencies.

## Bounded unsigned checksum preflight

```bash
cargo xtask release checksums create target/m10-remaining/artifacts
cargo xtask release checksums verify target/m10-remaining/artifacts
cargo test --locked --manifest-path xtask/Cargo.toml --test release_checksums
```

Supply an existing flat directory of regular files. The CLI fixes the manifest name to `RELEASE-CHECKSUMS.json`, excludes it from artifacts, and requires at least one artifact. Names are 1–128 ASCII characters, start with an alphanumeric character and use only alphanumerics, dot, underscore or hyphen. No absolute/relative nested manifest paths, traversal, backslashes, symlinks, symlink ancestors, directories or other nonregular files are accepted. The explicitly supplied directory itself may be absolute; its ancestors are opened without following symlinks, and `..` directory components are rejected.

Limits are 128 artifacts, 64 MiB per file, 256 MiB total and 64 KiB manifest JSON. Hashing streams bounded chunks. Verification rejects duplicate JSON keys/entries, malformed sizes/hashes, missing/extra files and changed hashes/sizes. Creation uses exclusive creation and never overwrites existing artifacts or a manifest. Canonical sorted JSON contains no time/random fields and explicitly says `unsigned local verification; not trusted provenance`. Errors do not echo filesystem paths or content.

This is a POSIX local preflight, not an archive extractor, provenance verifier or hostile multiwriter filesystem service. Freeze and isolate the supplied directory; do not mutate it concurrently. The manifest cannot authenticate itself: replacing artifacts and manifest together can pass unsigned verification. Trusted external digest/signature verification is a separate release gate.

## Independent rebuild, provenance and approvals

A future reproducibility claim requires a second isolated builder with independently controlled execution to rebuild the same reviewed source/lock/toolchain/target/features and compare every intended distributable digest. Retain both build records and explain mismatches before promotion. Rehashing existing files or two builds sharing a potentially compromised workspace/cache is not independent rebuild evidence. No such rebuild has run for M10e.

Design pin: use mature [Cosign v3.1.3](https://github.com/sigstore/cosign/releases/tag/v3.1.3) for signature/bundle verification; before activation, approve and record its platform binary SHA-256 and trusted-root material. Installation/bootstrap and public-fixture verification have [Private preparation evidence](YT-020-INTEGRITY-PREPARATION.md#verifier-and-public-fixture-experiments); later [maintainer-key preparation signatures](YT-020-PRIVATE-SIGNING-VERIFICATION.md) do not establish formal release signing or release-policy approval, which remain pending. Fail closed if the verifier version/digest or policy identity is absent or mismatched; do not use floating `latest` or a custom verifier. Follow upstream [verification guidance](https://docs.sigstore.dev/cosign/verifying/verify/).

Before promotion, independently verify the signed subject digest against downloaded bytes and require the approved provenance predicate/build type, immutable source commit, tag/ref, toolchain/target/features, inputs and builder identity. For GitHub OIDC, require exact issuer `https://token.actions.githubusercontent.com`, approved repository `bohewu/yetlet`, exact release workflow identity/path, and the approved immutable tag ref/commit association; reject a valid signature from another repository, workflow, ref or issuer. Require certificate/chain and transparency/inclusion verification under the pinned verifier and approved trust roots. A valid signature alone does not prove source, build behavior or reproducibility. The release workflow path, signing identity, tag approval policy, binary digest and trust roots are unresolved activation gates, not broad wildcard permissions.

Human release/source and security approvers must record approval of the concrete commit/tag, artifact/checksum/SBOM/provenance digests, independent rebuild evidence, support matrix, resolved findings and rollback target. Keep independent technical review separate from release authorization; the sole-maintainer merge configuration does not constitute a release approval. No key creation, signing service, registry upload, environment mutation, publication or deployment is performed here.

Promote only already verified immutable digests: do not rebuild, repackage or substitute a mutable filename/tag during promotion. Consumers verify signature identity and provenance policy again, then checksum-bound bytes. Roll back only to a previously approved, still acceptable digest with retained evidence. For compromised source/builder/key/identity, stop promotion, revoke affected trust/identity, mark affected digests rejected, notify through the approved security process and issue a new reviewed version; do not overwrite old release bytes or repoint a released version/tag. Registry yank/revocation and deployment rollback require separate authorization and an incident record.

## Historical M10e and overlay boundary

The M10e dirty snapshot and later YT-020 working overlays are development-only, not an immutable release source, trusted provenance, signed Yetlet artifacts or independently controlled trusted-release evidence. [M10e verification](M10E-VERIFICATION.md), [original Private preparation](YT-020-PRIVATE-RELEASE-PREP.md) and [integrity preparation](YT-020-INTEGRITY-PREPARATION.md) separate their actual source/run identities and experiments from pending activation. Independent reviewer PASS may accept their bounded design/preparation scopes only. Public activation and any application-selected production qualification remain separate; paid external security review is optional, not a framework publication prerequisite.
