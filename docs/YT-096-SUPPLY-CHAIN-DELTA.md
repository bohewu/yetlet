# YT-096 selected dependency, supply-chain and API delta

**Date:** 2026-10-20

**State:** Local inventory, license/source, RustSec and source-delta preflight complete with the SBOM coverage limits below. Ordinary slice review is recorded separately in the session evidence. YT-096 remains **In progress**; YT-020 remains **Blocked**.

## Candidate and authority

The owner-approved candidate remains
`ce56e0814e5423782440acd2c959ca777d572cd4`, experimental unpublished
`0.1.0`, Rust MSRV `1.99` and qualification toolchain `1.99.0`.
The [candidate decision](YT-096-CANDIDATE-QUALIFICATION.md) selects the original
nine libraries, GNU/Linux and Windows MSVC native targets, and five wasm
packages. This slice changes no product source, manifest, lockfile, tool pin,
license policy, API classification or support commitment.

The actual analysis host is Ubuntu **26.04.1 LTS**, not the selected Ubuntu
24.04 family execution baseline. Cross-target metadata and SBOM generation
do not compile or execute Windows or wasm code. Accepted first-slice packages,
consumer tests, historical application/platform results and owner waivers
retain their original source and environment bindings.

Work remains in the same checkout. This continuation authorizes local
qualification records, not commit, push, a tag, signing, publication, hosted
workflow activation, visibility changes or deployment.

## Inputs and scope isolation

[Input identities](../artifacts/yt096-supply-chain-delta-20261020/INPUTS.json)
bind the exact source, nine retained archive hashes, nine included package
lock hashes, policy and tool environment. Before reuse, all 63 selected
source inputs matched both the immutable source export and checkout; all nine
archives matched the accepted first-slice inventory. The 48 files in the
previous artifact directory were hashed for preservation.

Each unchanged archive was extracted into its own standalone package directory
under `/tmp/yt096-supply-ce56e08-o7bnx7oe/packages/`. Its normalized
`Cargo.toml` and included `Cargo.lock` were used without overlays.
Every metadata workspace has exactly one root. Sibling unpublished Yetlet
dependencies use the existing standard Cargo directory source. A fresh Cargo
home records that explicit source replacement; no new registry implementation,
consumer fixture or validation framework was introduced.

The root research workspace, excluded packages and previous consumer's extra
test dependencies are not roots of this analysis. Package-owned development
dependencies remain in the broader metadata, license and lockfile checks.
The directory source's registry-style package identifiers do not prove real
registry downloads or package rights.

All 23 explicit `cargo metadata --locked --offline --filter-platform` runs
passed. Lock hashes remained unchanged after metadata, SBOM generation and
license/source analysis. [Vendor integrity](../artifacts/yt096-supply-chain-delta-20261020/VENDOR-INTEGRITY.json)
checked the standard Cargo file checksums for all active dependency directories:
75 directories, including three sibling Yetlet packages and 72 third-party
packages, covering 5,415 files. The larger staged vendor directory is not the
release dependency inventory.

## Tool identity and commands

[Tool inspection](../artifacts/yt096-supply-chain-delta-20261020/TOOL-INSPECTION.json)
retains installed CLI help, versions and binary SHA-256 values. Existing tools
were reused; none was installed or upgraded.

| Tool | Version | Binary SHA-256 |
|---|---|---|
| cargo-cyclonedx | 0.5.9 | `e0b06abbe7fb6d957769ffd725bc71ed024409a9f1c804a244e4824667d68b56` |
| cargo-deny | 0.20.2 | `e76c96220653e171ba219603bbf2a8f6274551eb0ff002ac700df8c98d9e6429` |
| cargo-audit | 0.22.2 | `bc5027e6d3c25d5e15af67b9f0674cf0790b76f16657e0ff2e793845f10e679f` |

Every package uses default features with no explicit feature additions.
Exact argv, cwd, outputs and exit codes are retained in
[metadata results](../artifacts/yt096-supply-chain-delta-20261020/METADATA-RESULTS.json),
[SBOM results](../artifacts/yt096-supply-chain-delta-20261020/SBOM-RESULTS.json),
[license/source results](../artifacts/yt096-supply-chain-delta-20261020/LICENSE-SOURCE-RESULTS.json)
and [audit results](../artifacts/yt096-supply-chain-delta-20261020/AUDIT-RESULTS.json).

Representative forms, run against each standalone extracted package:

```sh
cargo metadata --format-version 1 --locked --offline \
  --manifest-path <package>/Cargo.toml --filter-platform <target>

cargo cyclonedx --manifest-path <package>/Cargo.toml \
  --format json --spec-version 1.5 --describe all-cargo-targets \
  --all --target <target> --target-in-filename

cargo tree --locked --offline --manifest-path <package>/Cargo.toml \
  --target <target> --edges normal,build --prefix none \
  --format '{p}|features={f}|license={l}'

cargo deny --format json --locked --offline \
  --manifest-path <package>/Cargo.toml --metadata-path <target-metadata.json> \
  --config <repository>/deny.toml --target <target> \
  check --show-stats licenses sources

cargo audit --file <package-lock> --db <owned-advisory-database> \
  --url https://github.com/RustSec/advisory-db.git --json
```

A second Cargo tree with `--edges normal,build,dev` records package development
coverage. SBOM generation used `CARGO_NET_OFFLINE=true`; cargo-cyclonedx has no
`--locked` flag, so the explicit locked preflight and before/after lock hashes
supply the preservation check. The RustSec audits used a separate initially
empty Cargo home and requested database/registry refresh; they were not run
under the offline package-analysis configuration.

## Target inventory and SBOM coverage

All **23 CycloneDX 1.5 JSON files** were generated successfully, one library
Cargo target for each selected package/platform pair. Counts below exclude
the root package and include transitive Yetlet and third-party components.
A dash means that pair is outside the selected scope.

| Package | GNU SBOM components | MSVC SBOM components | wasm SBOM components | Complete package lock records |
|---|---:|---:|---:|---:|
| yetlet-core | 0 | 0 | 0 | 1 |
| yetlet-actions | 11 | 11 | 27 | 36 |
| yetlet-reactive | 0 | 0 | 0 | 1 |
| yetlet-html | 1 | 1 | — | 2 |
| yetlet-dom | 1 | 1 | 20 | 21 |
| yetlet-dom-keyed | 1 | 1 | 20 | 21 |
| yetlet-server-axum | 50* | 51* | — | 56 |
| yetlet-request-id | 50 | 51 | — | 55 |
| yetlet-session | 61 | 62 | — | 66 |

GNU means `x86_64-unknown-linux-gnu`; MSVC means
`x86_64-pc-windows-msvc`; wasm means `wasm32-unknown-unknown`.

[Coverage comparison](../artifacts/yt096-supply-chain-delta-20261020/SBOM-COVERAGE.json)
records component names/versions, metadata feature sets, both Cargo trees,
development-only differences and exact commands. All 23 SBOM component sets
match the normal/build reachability in their filtered Cargo metadata. All 23
trees including development dependencies match the active metadata nodes.

**Known conservative over-inclusion (`*`):** the two native
`yetlet-server-axum` SBOMs include `serde_derive 1.0.229` beyond
`cargo tree --edges normal,build`. Those trees contain 49 and 50 dependencies,
respectively. The root's development dependency enables Serde's `derive`
feature; metadata feature unification retains that edge after the generator
removes dev-only edges. No SBOM component is missing compared with these
recorded trees; the other 21 package/target component sets match exactly.

The generated SBOMs are retained unchanged. The extra component is documented,
not silently deleted or represented as a normal-library-only build fact.
The feature-bearing Cargo trees are part of the coverage evidence. This is
a conservative source-package inventory, not an exact compiled-binary graph.

The [0.5.9 generator source](https://docs.rs/crate/cargo-cyclonedx/0.5.9/source/src/generator.rs)
excludes dev-only edges and test, example, bench and custom-build Cargo targets.
`--all` requests transitive dependencies, not all workspace members or all
dependency kinds. Build dependencies are retained; the generator may label a
build-only component with `scope: excluded`, which does not mean it is absent
from the JSON. No `--no-build-deps` option was used.

[Cargo metadata](https://doc.rust-lang.org/cargo/commands/cargo-metadata.html)
filters the resolved graph by platform; manifest dependency declarations are
not themselves proof of target activity.
[Cargo tree](https://doc.rust-lang.org/cargo/commands/cargo-tree.html) describes
its normal/build view as an approximation of build dependencies. Cargo
metadata/features, tool-generated SBOMs and actual compiled artifacts must
therefore remain distinct evidence.

Rust sysroot, compiler/tool binaries, native/system libraries, JS loader/glue
and deployment assets are not represented by these Cargo component inventories.
The tool binary identities above do not make them SBOM components. Actual
platform builds and any distributed browser/deployment assets need their
matching build/input records before release claims.

## License and source results

**PASS: 23/23 checks, zero license errors and zero source errors/warnings.**
The existing `deny.toml` was unchanged:
SHA-256 `a223b86141be9273e2a772b3636a308236f674eb14d90920fcfb240700c0ef4d`.

The policy includes development and build dependencies, does not ignore private
packages, allows MIT, Apache-2.0, BSD-3-Clause and Unicode-3.0, and denies unknown
registries/Git sources. No exception, suppression or `--exclude-dev` was added.

[License/source summary](../artifacts/yt096-supply-chain-delta-20261020/LICENSE-SOURCE-SUMMARY.json)
records **29 warnings**, all `license-not-encountered`: BSD-3-Clause 17 times
and Unicode-3.0 12 times across the individual checks. They mean that a policy
allowance was unused in that package/target graph, not that a dependency has a
rejected license. The original policy was retained.

These checks establish the configured license/source policy result for the
recorded local resolution. Registry-style source identifiers under a directory
replacement do not authenticate upstream publishers, prove registry rights or
waive notice obligations.

## RustSec audit results

**PASS: all nine unmodified package lockfiles; zero reported vulnerabilities
and zero warnings.** Every command exited 0 and produced empty stderr.
The complete locks cover 73 distinct third-party name/version pairs across all
package/target records, including package development/build dependencies and
platform entries beyond the selected execution matrix. This is deliberately
broader than an individual SBOM.

[Audit summary](../artifacts/yt096-supply-chain-delta-20261020/AUDIT-SUMMARY.json)
and raw JSON retain:

- database: `https://github.com/RustSec/advisory-db.git`;
- observed database commit: `999ec89d5d3f10f20be14750ec8c2d8b44e0ac9b`;
- database commit time: `2026-10-17T17:23:59+02:00`;
- 1,338 advisories; same database commit in every package result;
- refresh/check window: 2026-10-20 07:52–07:53 UTC;
- empty ignore list, no target filters, no stale override and yank checking enabled.

The completed Session result from the initial refresh was reused. No duplicate
target-by-target audit was needed for identical package lock bytes.
`cargo-audit` inspects complete lockfiles; its OS/architecture filters would
filter advisory applicability, not reconstruct a selected dependency graph.
An empty report describes known advisories in this database snapshot, not
absence of all vulnerabilities. It does not prove unpublished Yetlet registry
availability, ownership, upload/download bytes or real-registry consumption.

## API, MSRV and threat-model delta

[Source delta](../artifacts/yt096-supply-chain-delta-20261020/SOURCE-DELTA.json)
compares historical signed source
`8e557e760407a3d4b3f5a8b8f833748946be1387` with the selected `ce56e08`.
All nine package manifests are byte-identical. All 73 current third-party
archive-lock name/version pairs already occurred in the old root lock.
That last comparison is a version-identity baseline, not a reconstructed old
target graph or a refresh of historical signatures.

The nine package READMEs were added. The only selected library Rust change is
`crates/yetlet-reactive/src/lib.rs`: the already accepted
[YT-105 weak-handle snapshot](YT-105-GRAPH-SNAPSHOT.md). The other eight
libraries' Rust sources are unchanged. Existing Rust 1.99 metadata, first-slice
nine-package compilation and packaged consumer evidence are retained; no lower
MSRV or new cross-platform runtime result is inferred.

| Boundary | Assessment for this candidate |
|---|---|
| Public API | Additive `GraphHandle::snapshot(&self) -> Result<State, ReactiveError<Error>>`; existing owner snapshot delegates to the same helper. No selected manifest/dependency or Stable Candidate promotion. Reactive remains Experimental. |
| Read capability | A weak handle now permits reading published State as well as dispatch. Applications must treat possession of a handle as that explicit local capability; it is not an authorization credential or a read-restricted dispatch token. |
| Ownership and disposal | The handle remains weak; the call temporarily upgrades it. Disposed precedes Faulted. A returned cloned value can outlive graph disposal and is not revoked by it. |
| Reentrancy and freshness | The graph borrow ends before user-defined `State::clone`. Clone may re-enter, have side effects or share interior-mutable data. Snapshot is the state reference acquired for that read, not a deep copy or freshness promise after return/await. |
| Network, credentials and browser trust | This change adds no network I/O, serialization, credential provider, authentication, Session or server authorization path. Application-owned State and all browser/WASM data remain subject to existing confidentiality and server-validation rules. |
| Supply-chain threat T15 | Source/lock/tool identities, selected inventories, license checks and the fresh RustSec snapshot improve local candidate evidence. Independent rebuilding, authenticated provenance/signing, immutable promotion and registry trust remain separate unresolved release gates. |

The [security policy](../SECURITY.md) and [threat model](THREAT-MODEL.md) are
byte-identical to the historical baseline. Their requirements remain in force;
this delta record does not convert their historical sample descriptions into
new deployment claims. Accepted Admin/Starter/cloud/rolling evidence keeps its
own scope. The Windows OIDC waiver and deleted disposable IdP account/client
remain as recorded; this work recreates neither. Unrelated Windows IME,
assistive-technology or performance repeats are not required by this source
delta.

No product fix was indicated by this bounded inspection. The existing YT-105
implementation/consumer results were reused, not relabeled as new tests.
This is source-bound delta analysis and an ordinary slice review, not the final
independent security/release gate or production certification.

## Artifact binding and preservation

The [candidate identity](../artifacts/yt096-supply-chain-delta-20261020/CANDIDATE-IDENTITY.json)
maps every archive SHA-256 and package lock to its target-specific SBOM SHA-256,
Cargo target, selected features, tools and known coverage differences.

The flat checksum bundle contains **36 files**: nine unchanged archives,
23 generated SBOMs, candidate identity, coverage comparison, license/source
summary and audit summary. Existing `cargo xtask release checksums create`
and `verify` both passed.
[Checksum result](../artifacts/yt096-supply-chain-delta-20261020/CHECKSUM-RESULT.json)
binds the manifest
[RELEASE-CHECKSUMS.json](../artifacts/yt096-supply-chain-delta-20261020/bundle/RELEASE-CHECKSUMS.json),
SHA-256 `9e49da9da4991c97dc2c48d1a6f8937ff1cd426265cd255f509164c19bc4908a`.

This is **unsigned local verification, not trusted provenance**. No old
signature is reassigned to the new bundle. No background server, browser,
watcher or hosted job was started. The owned analysis inputs and fetched
advisory database remain available for continuation; all foreground commands
terminated. Final documentation/review and preservation evidence is recorded
separately with this slice.

## Next qualification work

| Order | Bounded work and evidence |
|---|---|
| 1 | Use an actual Ubuntu 24.04 family environment with pinned Rust/Cargo 1.99.0, the same candidate inputs and default features. Record OS/toolchain/source hashes; run the selected native package/API tests and required wasm checks. The current 26.04.1 host cannot fill that support-baseline cell. |
| 2 | Qualify the same nine packages on the selected Windows MSVC target. Bind exact source/archive inputs, toolchain, native checks/tests/doctests and cleanup. A Windows GNU historical PASS is preserved but does not fill the MSVC cell. |
| 3 | Reconcile the five packaged wasm libraries and relevant Chrome/Firefox behavior, recording actual browser/wasm-bindgen/loader versions and source matches. Reuse accepted unrelated application, IME/AT and performance evidence under the owner waivers. |
| 4 | Rebuild the exact intended distributables on the selected second host and satisfy the independently controlled builder requirement. Carry this inventory's dev-feature over-inclusion and non-Cargo omissions into the build-bound coverage review. |
| 5 | Complete current-artifact provenance/signature, formal trust/registry and independent release/security review, followed by explicit maintainer approval. External activation needs its own authority; local preflight never supplies it. |

Preserve existing required and scheduled CI. This slice neither runs hosted
workflows nor weakens their native, browser, MSRV or audit gates.

The subsequent [partial platform followup](YT-096-PLATFORM-QUALIFICATION.md)
records passing Ubuntu 24.04.5 container userspace tests/doctests, five packaged
wasm builds and a Linux Firefox workflow. The resumed Windows MSVC standalone
archive lane now passes 83 native tests and four doctests across nine package
commands. Historical root-workspace/cache failures remain preserved. The
331-directory offline staging includes inactive-target resolution inputs; it
is not a replacement for the selected dependency inventories. The original
inventory and checksum evidence above remains unchanged. Consuming those
archives on Windows was not a second-host archive rebuild. The separately
executed [two-host rebuild](YT-096-TWO-HOST-REBUILD.md) now produces nine
byte-identical Windows archives with normal Cargo verification. Original
archive/SBOM/checksum bytes remain unchanged. Shared builder control and
staged sources mean independent rebuilding, build-bound inventory and formal
trust/approval gates remain open.

## Historical Azure SSH clean-builder attempts

The [Azure attempt](YT-096-AZURE-CLEAN-BUILDER.md) created two temporary VMs
but stopped before source transfer/build after consecutive SSH timeouts.
No Azure toolchain installation, Cargo fetch, package verification, native
test, wasm build or build-bound inventory result is claimed. Separate cleanup
records preserve both attempts. The accepted platform/archive/SBOM/checksum
evidence and all remaining trust/approval boundaries stay unchanged.

## Azure Run Command/private Blob continuation

The [owner-resumed clean builder](YT-096-AZURE-RUN-COMMAND.md) now passes on
an actual Ubuntu 24.04.5 guest, kernel 7.0.0-1017-azure, with upstream tool and
dependency downloads into an empty cache. Nine archives are byte-identical
to the original Linux/Windows outputs; 83 native tests/four doctests and five
wasm libraries pass. All temporary resources and private Blob storage are
deleted. The earlier failed SSH attempts remain unchanged.

There are 140 actual Cargo compiler-artifact events. Native 70 observed package
versions comprise 68 in retained target SBOMs and 2 known dev-only dependencies,
tower-http 0.6.11 / bitflags 2.13.2, already covered by retained dev metadata.
All 32 wasm package versions occur in retained wasm SBOMs. This aggregate
observation does not prove per-library feature equivalence, output-file
digests, Rust sysroot/native-linker inventory or formal builder trust.
Existing source inventory bytes, coverage differences, checksums and owner
waivers remain intact; final inventory/trust/release approvals remain open.
