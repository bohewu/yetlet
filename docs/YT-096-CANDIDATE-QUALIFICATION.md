# YT-096 candidate selection and package preflight

**Date:** 2026-10-20

**State:** Candidate selected; local package/consumer and dependency/API delta preflights complete with recorded limits. Platform followups pass an actual Ubuntu 24.04.5 Azure guest, five wasm libraries and a Linux Firefox workflow; the resumed Windows MSVC standalone-package lane passes 83 native tests and four doctests. The two-host archive rebuild matches all nine packages exactly; the subsequent empty-cache Azure build also matches all nine and passes 83 native tests/four doctests. YT-096 remains In progress and YT-020 remains Blocked.

## Owner decision and immutable source

The owner approved the inventory/support proposal and instructed continuation on
2026-10-20. The selected source is
`ce56e0814e5423782440acd2c959ca777d572cd4`, tree
`0c5fa71179741f7b52dead078ff6d8a9efc30241`, on
`feat/yt090-configuration`. The selected coordinated Cargo version is
**experimental, unpublished 0.1.0**, with **Rust 1.99 MSRV** and **1.99.0**
as the development and qualification toolchain pin.

This selection authorizes the bounded qualification work described here. It
does not authorize commit, push, tagging, signing, registry publication,
repository visibility changes, hosted workflow activation or deployment.
The earlier [private trust policy](YT-020-PRIVATE-TRUST-POLICY.md) and
[release supply-chain gates](RELEASE-SUPPLY-CHAIN.md) remain in force.

The checkout initially had no tracked/index changes and 48 existing untracked
status entries. Work continues in that checkout, without a detached worktree.
Package inputs come from an immutable Git archive, not those untracked files.
The new documentation and consumer fixture are separate from the selected
candidate source. A later candidate change must receive an explicit source
identity and corresponding evidence; historical signatures cannot follow
changed bytes automatically.

## Selected package and API scope

All nine packages use their default features. Their current manifests declare
no package feature table; browser-only dependencies are target-conditioned.

| Package | Selected boundary |
|---|---|
| `yetlet-core` | Renderer-neutral values and validated binding vocabulary |
| `yetlet-actions` | Explicit local/HTTP typed action boundary |
| `yetlet-reactive` | Optional synchronous local graph, including weak-handle snapshot |
| `yetlet-html` | Complete-string HTML rendering and escaping |
| `yetlet-dom` | Bounded browser DOM renderer and native policy validation |
| `yetlet-dom-keyed` | Explicit keyed ownership/reconciliation primitives |
| `yetlet-server-axum` | Native Axum/Tower application and request boundary |
| `yetlet-request-id` | Optional process-local request correlation |
| `yetlet-session` | Optional bounded process-local Memory Session/CSRF |

Selection does not promote the classifications in [API contracts](API-CONTRACTS.md)
or change the experimental [compatibility policy](COMPATIBILITY.md).
Stable Candidates remain pre-public candidates; actions, reactive and browser
surfaces retain their documented Experimental limits. Session is neither
authentication nor durable/distributed storage.

CLI, config, secrets, signing, Redis/Valkey, wgpu, samples, xtask and third-party
frontend glue are outside the distributed nine-package scope. Existing
applications may supply source-bound compatibility evidence without becoming
registry-only consumers or supported packages. Native constructor,
trait/Arc and Axum State/FromRef composition remains the
[YT-095 decision](YT-095-COMPOSITION-RESEARCH.md).

## Selected qualification matrix

These are the owner-selected qualification targets, not a declaration that
every fresh-candidate cell has already passed.

| Target or environment | Selected use | Current evidence |
|---|---|---|
| Ubuntu 24.04 family, `x86_64-unknown-linux-gnu` | Native nine-package support baseline | [Actual Azure Ubuntu 24.04.5 guest](YT-096-AZURE-RUN-COMMAND.md), kernel 7.0.0-1017-azure: 83 native tests/four doctests PASS; earlier container shared-kernel evidence retained |
| Windows x64, `x86_64-pc-windows-msvc` | Native nine-package support baseline | [Standalone archived inputs](YT-096-PLATFORM-QUALIFICATION.md#windows-msvc-standalone-package-continuation-pass) PASS: nine commands, 83 native tests and four doctests on Rust 1.99.0 MSVC; historical offline workspace failures retained |
| `wasm32-unknown-unknown` | core/actions/reactive/dom/dom-keyed | Five unchanged standalone packaged roots build successfully with Rust 1.99.0 |
| Linux desktop Chrome and Firefox | Selected browser workflows, with exact versions recorded | Fresh candidate-source Firefox PASS on Ubuntu 26.04.1; accepted Chrome consumer evidence retained without claiming a new candidate run |
| Local bohe-ubuntu, Ubuntu 26.04.1 LTS, GNU x86_64 | Package and native consumer preflight host | Passed below; this is not Ubuntu 24.04 evidence |
| Historical macOS ARM64 and Windows GNU | Retained historical qualification | No fresh support expansion in this slice |

Required and scheduled CI remain unchanged. The selected package list does not
remove existing full-workspace, sample, browser, audit or other required checks.
The host's actual Ubuntu 26.04.1 identity was discovered during execution and is
recorded without silently changing the selected Ubuntu 24.04 baseline.

## Inventory and reusable evidence

At the selected commit there are 52 tracked Cargo manifests, 51 package
manifests, 17 workspace roots and 35 explicit root members. Forty-two packages
are nonpublishable. The root lock has 358 package records: 322 registry and 36
local records. These workspace counts include research and examples; they are
not the selected release dependency inventory.

Relative to the previously signed source
`8e557e760407a3d4b3f5a8b8f833748946be1387`, the nine candidate manifests are
unchanged and all nine package READMEs have been added. The only selected-library
Rust source change is in `yetlet-reactive/src/lib.rs`: the reviewed additive
weak-handle snapshot and shared snapshot implementation. Its source digest is
`8aca69b38154384a7739d602e9ec6094caff882d0b0e66d6f1bb6ac91333a62b`.
The [YT-105 contract and accepted tests/consumer adoption](YT-105-GRAPH-SNAPSHOT.md)
remain applicable at their recorded source and platform bounds; no new graph
abstraction or dependency was introduced by this slice.

Retain [Starter](YT-092-STARTER-VERIFICATION.md),
[Admin closeout](YT-094-CLOSEOUT.md),
[cloud delivery](YT-101-VERIFICATION.md), and
[bounded rolling/asset retention](YT-103-ROLLING-ASSET-VERIFICATION.md)
evidence. Those references do not establish production authentication,
certificate custody, durability, SLA or HA guarantees for the nine packages.
The owner-waived Windows OIDC and routine repeated Windows IME/assistive
technology work remain waived. Do not recreate the removed disposable OIDC
client/account or repeat unrelated passing workloads.

The three historical signatures and 37-artifact set remain bound to their
original source. The old audit is not a fresh audit of the current root lock.
The local package preflight below does not replace a selected-target SBOM,
license policy check, vulnerability audit or independent release/security review.

## Immutable source and package evidence

Retained local evidence is in
`artifacts/yt096-candidate-preflight-20261020/`; the records and archives are
not a signed release bundle. The [source input record](../artifacts/yt096-candidate-preflight-20261020/SOURCE-INPUTS.json)
contains exact tool hashes, OS identity, 63 selected input hashes and source
locations. Important identities are:

| Input | SHA-256 |
|---|---|
| Git source archive | `9096cabe9a7b6d8b809c5bd337c2a8ab88f33ff19b6b4c8b59f31267c7335546` |
| Canonical selected input map | `c10830cd2c843c9e9e0f887b37ab0b985c9e396198263610306599bf03910d4d` |
| Root Cargo.lock | `924333885e8d65ed9f9e107b5bb00fbe47748771624198877b00edd1cc42bc3f` |
| Clean consumer Cargo.lock | `5bba384706961564d5de47f0819eb887b2d72b18ca0ea4e72661cb0569d4fcd9` |

The source export has no Git metadata. Exactly the existing 18 package license
symlinks were materialized with the corresponding root license bytes, matching
the established private-rebuild method. No product/source overlays were used.
Consequently the archives omit `.cargo_vcs_info.json`; source identity is bound
by the retained external input record. An eventual publish operation that
produces different bytes must be assessed and approved separately.

Actual tools: `rustc 1.99.0 (b940084d7 2026-09-28)`,
`cargo 1.99.0 (5f94df478 2026-08-27)`, host
`x86_64-unknown-linux-gnu`. Packaging used an empty target directory and the
existing dependency cache, offline. It was not an independent clean-cache or
second-host rebuild.

The [package build record](../artifacts/yt096-candidate-preflight-20261020/PACKAGE-BUILD.json)
and [Cargo log](../artifacts/yt096-candidate-preflight-20261020/package.log)
record this command, with the absolute pinned Cargo binary, exported-source cwd,
controlled environment and nine explicit selections:

```bash
cargo package --locked --offline --manifest-path <export>/Cargo.toml \
  --target-dir <owned-package-target> \
  -p yetlet-core -p yetlet-actions -p yetlet-reactive \
  -p yetlet-html -p yetlet-dom -p yetlet-dom-keyed \
  -p yetlet-server-axum -p yetlet-request-id -p yetlet-session
```

**PASS:** all nine archives and Cargo's extracted-package compilation completed
with exit 0, without `--no-verify` or warnings. All 63 selected source inputs,
including the root lock, remained unchanged.

The [package inventory](../artifacts/yt096-candidate-preflight-20261020/PACKAGE-INVENTORY.json)
records every archive/file hash, normalized manifest and exact byte size.
All nine READMEs and both license files per package are included. Normalized
manifests have the selected name/version/edition/MSRV/license and registry
version dependencies, with no path/workspace/Git dependency leakage or
workspace/patch/replace table. Original source files and license bytes match.
Only the expected normalized manifest, original manifest copy and package lock
are Cargo-generated packaging representations. Archive SHA-256 values match
Cargo's temporary-registry index.

| Package | Archive bytes | SHA-256 |
|---|---:|---|
| yetlet-core | 9,103 | `664505f181f28e502ae02cdd7cb757044780c3059399cbc73863417082cd184e` |
| yetlet-actions | 11,580 | `dbd7f3d37ae977bdc03537bbfaa569040c8d9eb2ae9288f3d258b209c9ea1cd0` |
| yetlet-reactive | 14,487 | `652d881a49c7bbfa260675c0d22d6249abadb559d958f35b0c8c0cded6b6cdd3` |
| yetlet-html | 8,673 | `09d306cf4e9b2cc85811e215e10bdbbfeb874db9e3a770678b19703d5a61cb23` |
| yetlet-dom | 9,651 | `22826cb54e7ec3ae41a83240a76877d11e425c5da8894c6df0ca4172fb045668` |
| yetlet-dom-keyed | 11,561 | `1bc2c891286713c0552e827e0092213252c57099d3e4aa886cf487a090d4a05b` |
| yetlet-server-axum | 25,173 | `bdfe803d8e4a67b94bbf6ee3a237a972740cbaf8ec0cbcff3383ecfb952ab380` |
| yetlet-request-id | 13,274 | `3d0d547ef1d93f5e492b0e2975ce26ff3a06ddc710a5923b91eae48fdb958f60` |
| yetlet-session | 18,520 | `927525c423dcf77f9046f648c7de02d82f802c0979cac396dd225a67e1274534` |

Package READMEs still explicitly describe an unpublished checkout workflow.
Their companion-document/example links assume that checkout. Packaging them
does not prove a public documentation site or registry release exists.

## Clean packaged-input consumer

The [consumer manifest](../artifacts/yt096-candidate-preflight-20261020/consumer/Cargo.toml)
declares the nine exact `=0.1.0` version dependencies, no path dependencies or
patches, and its own workspace. A disposable copy was run outside the repository
with an empty Cargo home and target directory. Only fixture development
dependencies add the existing Axum/Tokio/Tower versions.

Cargo's standard directory source was used for a **local prepublication
simulation**. Cargo vendor staged the cached 322 remote workspace packages
offline; each of the nine candidates was then extracted unchanged from its
retained archive, with standard file/package checksum metadata. Extra vendored
packages are staging data, not additional release candidates or a claim that
all 322 dependencies are in this consumer.

The consumer initialized its own lock once with `cargo generate-lockfile
--offline`, then ran `cargo metadata --format-version 1 --locked --offline`
and `cargo test --locked --offline`. The
[result record](../artifacts/yt096-candidate-preflight-20261020/CONSUMER-RESULT.json)
and [test log](../artifacts/yt096-candidate-preflight-20261020/consumer-test.log)
record exact commands, environment and hashes.

**PASS:** 81 resolved packages (consumer, nine Yetlet packages and 71 third-party
packages); all dependency manifests resolve inside the disposable directory
source, with no original-checkout package path. Each selected lock checksum
equals its retained archive SHA-256. The consumer lock is unchanged after
metadata and tests. Four executed native tests passed:

1. Shared View renders escaped HTML and passes native DOM policy validation.
2. Keyed order accepts shared BindingKey values and rejects duplicates.
3. A local typed action uses weak-handle snapshot; dropping the strong graph
   owner leaves the handle disposed.
4. A native router composes Session, Request ID, strict CSP and script-free SSR.

These tests do not execute a real browser, network listener or deployment.
There are no background server/browser resources from this slice.
Cargo's source replacement retains registry-style identifiers; those identifiers
are not proof of registry downloads. Actual crates.io names, account rights,
upload/download bytes and registry consumer resolution remain unverified.

The first metadata inspection stopped before tests because its one-off parser
assumed a `path` key always existed. Cargo omits that optional key for registry
dependencies. The terminal harness failure is retained in the result record;
using an optional-key read resolved it. Successful packaging, vendoring, lock
generation and metadata were reused, and no product source was changed.

## Selected dependency and API delta

The [supply-chain and API delta](YT-096-SUPPLY-CHAIN-DELTA.md) records
23 standalone package/target metadata and CycloneDX inventories, 23 passing
license/source checks, nine passing fresh-database RustSec package-lock scans,
and an unsigned 36-file checksum bundle. The two native server-axum SBOMs
conservatively include one dev-enabled Serde derive component beyond their
normal/build Cargo trees; the generated outputs and coverage differences are
retained. No source, package lock, dependency, tool pin or API classification
changed. The source/API and threat-boundary reconciliation is complete at this
bounded preflight scope; final independent release/security review remains open.

## Remaining qualification and review

The package/consumer and selected dependency/API delta preflights are complete
at their documented local scopes. Documentation checks and ordinary slice
reviews are recorded with their final session evidence; no independent
release/security PASS is claimed here.

| Remaining gate | Required evidence |
|---|---|
| Candidate MSRV/platform execution | Rust 1.99.0 native GNU/MSVC and five wasm libraries PASS; preserve source/API reconciliation and final supported-matrix review |
| Selected native/browser targets | Actual Ubuntu 24.04 guest, Windows MSVC and selected wasm execution PASS; fresh Firefox and historical Chrome scope require final matrix reconciliation |
| Build-bound inventory | Carry completed source-package SBOM/license/audit and threat delta into actual artifact/build review, including recorded dev-feature over-inclusion and non-Cargo omissions |
| Independent rebuild | [Two-host archive comparison](YT-096-TWO-HOST-REBUILD.md) PASS; [separately operated GitHub-hosted candidate rebuild](YT-096-GITHUB-HOSTED-REBUILD.md) matches all nine archives with exact source/tool/lock inputs. Final authenticated builder/source/tag/trust-policy acceptance remains required |
| Trust and registry | Current exact artifact provenance/signatures, formal trust requirements and separately authorized real registry rights/download/consumer checks |
| Independent release/security review | Review the complete candidate evidence and residual risks; this local slice does not satisfy that gate |
| Maintainer decision | Explicit final release approval and separate external activation authorization |

The [partial platform followup](YT-096-PLATFORM-QUALIFICATION.md) records
Ubuntu userspace/wasm and Firefox results, the resumed Windows MSVC PASS for
all nine standalone archived packages, retained cache-resolution failures and
exact preservation/cleanup. The [Windows archive rebuild](YT-096-TWO-HOST-REBUILD.md)
now produces nine byte-identical archives with normal Cargo verification.
Resolve final platform evidence scope and continue build-bound inventory and
independently controlled builder evidence before the later trust/approval gates.
The earlier [Azure SSH attempts](YT-096-AZURE-CLEAN-BUILDER.md) stopped before
transfer/build and remain preserved. The [Run Command/private Blob continuation](YT-096-AZURE-RUN-COMMAND.md)
now passes actual Ubuntu 24.04.5 execution, an empty-cache nine-archive rebuild,
83 native tests/four doctests and five wasm builds. All temporary resources
are deleted. Aggregate compiler observations and known dev-only differences
are recorded; they do not complete all feature/binary/system inventory or
independent builder trust. Preserve accepted application/platform evidence
and owner waivers.
YT-096 stays **In progress** and YT-020 stays **Blocked**.

## Subsequently authorized GitHub-hosted rebuild

The owner later authorized necessary push and dispatch of the existing private
workflow, comparison and independent review, while prohibiting signing and
publication. [Run 39418279958](YT-096-GITHUB-HOSTED-REBUILD.md) now passes at
the same immutable candidate: nine byte-identical archives, matched 63 source
inputs, lock/epoch/tools and unsigned checksum/provenance bindings. The native
GitHub-hosted Ubuntu 24.04.5 job is terminal and its private artifact/logs were
saved locally. This supplies separately operated execution evidence, not formal
authenticated provenance or final release authorization. Historical results and
API classifications remain unchanged.
