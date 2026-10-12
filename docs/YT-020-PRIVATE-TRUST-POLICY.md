# YT-020 — Private Trust and Future Registry Policy

**Date:** 2026-10-12
**Decision:** Select Cosign public-key trust for bounded Private preparation and crates.io for a future, separately authorized public source-package release.
**Original policy execution state:** Documentation only. No key, fingerprint, signature, tag, registry ownership or publication is activated. YT-020 remains **Blocked** for formal release activation.

**Later explicit execution authorization (2026-10-12):** The maintainer subsequently authorized committing the reviewed policy, dedicated key creation/fingerprint enrollment, bounded Private signing and independent verification/review, with no push/publication/visibility change. [Actual signing evidence](YT-020-PRIVATE-SIGNING-VERIFICATION.md) records this later execution separately. The original documentation-only and Pending/Not verified statements below describe the policy checkpoint; the later evidence updates only Private execution status and never waives formal release guarantees.

**Historical bounded execution at signed source `8e557e7`:** The dedicated ECDSA P-256 key is enrolled with SHA-256 DER SubjectPublicKeyInfo fingerprint `6e995c563c322fe14912c8660932bb8b8c803bb837007b596c883bd9b4da5fef`. Three canonical signed objects, their 37-artifact set and six negative cases passed separate public-only verification. Cosign's no-log flag is confined to this explicitly selected preparation mode; requiring the original log proof rejects the bundle. Formal OIDC/certificate/log/builder approval and registry activation remain Not verified. The next activation decision is the formal release service/identity/root/builder/tag policy, not duplicate preparation verification.

## Current preparation applicability (2026-10-12)

The [T08 source/package boundary](YT-020-PRIVATE-RELEASE-PREP.md#current-preparation-boundary--t08--yt-068-2026-10-12) separates signed source `8e557e760407a3d4b3f5a8b8f833748946be1387`, policy HEAD `abb2cce1253afe1d87a97e0a50f3f47efffe7280` and the current dirty research/documentation checkout. The enrolled key and three signatures/37-artifact verification remain historical Private preparation successes. They do not attest the new READMEs, changed root manifest/lock or research. A separately approved immutable candidate needs its own package preflight, rebuild/provenance, selected SBOM/license evidence and signing verification under separate execution authority. The independent delivery gate/result for this documentation is recorded separately in the run; documentation-stage checks do not attest that gate; no key access or signing occurs here.

The original authority and Pending statements below describe the policy checkpoint, superseded only at historical Private execution scope by the signing record. Formal service/identity/roots/builder/tag approval, certificate/chain and transparency/inclusion guarantees, real registry/disclosure/reporting controls and concrete digest approval remain open. YT-020 stays Blocked; a future local commit does not activate release.

## Original policy authority and scope

The maintainer authorized this policy and independent repository review in the original checkout. Key creation, signing, staging, commit, push, publication, visibility changes and external configuration changes are not authorized for this task. Document approval does not prove unperformed execution.

The scope remains experimental `0.1.0`, Rust 1.99, `MIT OR Apache-2.0`, and the [original nine libraries](YT-020-PRIVATE-RELEASE-PREP.md#selected-artifacts). Samples, browser assets, Docker fixtures and isolated `yetlet-session-redis` remain nonpublishable. [Compatibility and best-effort maintenance](COMPATIBILITY.md) are unchanged.

## Preparation trust does not replace release trust

| Property | Private public-key preparation | Formal release activation |
|---|---|---|
| Trust anchor | Exact maintainer public key and independently approved fingerprint | Approved verifier, trust roots, signing identity and builder policy |
| Signature meaning | The approved maintainer key signed these exact bytes | Required release signature, identity and provenance verification all pass |
| Source/build claims | Signed claims are checked against retained evidence; maintainer signing does not authenticate a builder's identity | Independently verify approved source/tag, inputs, build policy and builder identity |
| GitHub OIDC identity | Not established by a public-key signature | Exact issuer, repository, release workflow and ref/commit binding remain required for the planned GitHub flow |
| Certificate and transparency | Not provided or claimed by this mode | Certificate/chain and transparency/inclusion verification remain mandatory |
| Publication authority | None | Explicit approval of concrete release identities and digests after all gates pass |

This is an additional preparation mode, not an exception to [the supply-chain contract](RELEASE-SUPPLY-CHAIN.md#independent-rebuild-provenance-and-approvals). Missing certificates, transparency proof and authenticated builder identity remain **Not verified** after a future Private public-key PASS. Signing a self-reported statement does not make it builder-authenticated provenance or establish a SLSA level.

The planned formal GitHub OIDC identity remains issuer `https://token.actions.githubusercontent.com`, repository `bohewu/yetlet`, workflow `.github/workflows/release.yml`, and proposed ref `refs/tags/v0.1.0` bound to its separately approved immutable commit. This policy installs no workflow. The actual service, approved roots/builder/build type and tag-signing mechanism remain activation decisions. Wildcard identity, disabled certificate/log verification and valid foreign signatures cannot satisfy the release gate.

## Private key and verification policy

`bohewu` is the selected key owner and approval authority. The minimum future custody choice is a dedicated password-encrypted Cosign key outside the repository and build environment, with private file access and a separately protected recovery copy. Supply its password privately, never in commands, logs or evidence. No private key/password belongs in Git, archives, browser state, CI caches or artifact bundles. An existing KMS or hardware key may be selected later; buying or operating one is not required for this preparation.

Before trusting a key, record its public-key bytes, SHA-256 of DER SubjectPublicKeyInfo, algorithm, owner, approval date and preparation-only purpose. The fingerprint and custody location are **Pending**. Authenticate the expected fingerprint through the existing private maintainer channel independently of the artifact download. An artifact's accompanying key is verification material, never its own trust authority. Missing or mismatched key approval fails closed.

Retain the bootstrapped Cosign **3.1.3** Linux amd64 pin, SHA-256 `4629c757b7618056f8ddd7e2625ae9fdd94c0372a65049520bc7d9df9efc7f71`, and [bootstrap evidence](YT-020-INTEGRITY-PREPARATION.md#verifier-and-public-fixture-experiments). Other platforms need their own approved binary identity. Tool changes require a reviewed pin decision; no floating `latest` or custom cryptographic verifier.

Cosign supports [explicit-key blob signing](https://docs.sigstore.dev/cosign/signing/signing_with_blobs/) and [self-managed keys](https://docs.sigstore.dev/cosign/key_management/signing_with_self-managed_keys/). The exact no-network signing/public-key verification invocation still needs qualification against the pinned binary after separate execution authorization. Default Cosign service behavior is not assumed private or offline. No executable signing recipe or verifier bypass is approved here. Report an unsupported invocation before changing tools, adding services or relaxing verification.

Future preparation signs frozen checksum-manifest and selected provenance bytes. The manifest must bind all selected archives, SBOMs/coverage records, source/lock/tool identities, primary/hosted build records, comparison and provenance. Each provenance statement must bind its exact subjects and source/build inputs. Preserve original unsigned records and assemble a separately identified evidence set. Freeze inputs before signing; keep detached signature outputs separately to avoid a manifest/signature hash cycle.

The retained immutable-source experiment uses commit `8e557e760407a3d4b3f5a8b8f833748946be1387` and GitHub run `38195517029`; its nine raw archive digests equal the primary build's. [Hosted rebuilding](YT-020-HOSTED-REBUILD.md) and ignored `target/yt020-integrity-prep-20261012/hosted-rebuild/final-closeout.json` retain actual execution and independent reviewer PASS. This is not a release-tag approval or signature. Earlier Windows/shared-control builds and inventories retain their original identities and omissions; reuse requires the recorded input/digest comparison, not relabeled execution.

A separately run verifier receives frozen artifacts and public material without the private key, password or primary cache. Verify the approved fingerprint and signatures over downloaded bytes, then manifest completeness/hashes and provenance subject/source/lock/tool/target/features/build-record bindings against the independently approved snapshot. Reject a valid signature with a wrong approved source/digest or an unapproved older snapshot. Retain exact commands, tool/key identities, exit codes and diagnostics.

Bounded future qualification covers successful verification and rejection of wrong keys, altered manifest/provenance, altered subject bytes and a correctly signed different snapshot. These tests are **Not verified**, not permission to execute them now or build a validator framework. No duplicate package/browser/Docker test is needed merely to refresh a date or SHA.

For rotation, independently approve a new key and record when the old key stops being accepted. For suspected compromise, stop accepting affected signatures and record rejected key/snapshot identities. Re-signing alone does not clear source/build concerns; recovery requires reviewed evidence and explicit approval, without replacing old artifacts.

This mode provides authentication and integrity under the approved maintainer key. It provides no public transparency, trusted signing timestamp, OIDC certificate identity, authenticated builder attestation, publication approval or production assurance. Actual Private signatures and independent public-key verification remain **Not verified** until separately authorized and executed.

## Future crates.io distribution

Select crates.io, the original nine names and intended named owner `bohewu`. This selects a destination, not an upload date, ownership claim or permission to disclose source. The read-only 2026-10-12 API lookup returned HTTP 404 for all nine names; they are unreserved and ownership is **Not verified**. Recheck at activation because [names are allocated on first publication](https://doc.rust-lang.org/cargo/reference/publishing.html#before-publishing-a-new-crate).

A future upload makes packaged source/metadata public even if GitHub stays Private. Package disclosure and repository visibility require separate approvals. Existing Public-transition reporting/scanner gates remain required for that separately authorized transition; this policy neither performs nor waives them. Resolve reachable reporting and applicable security acceptance for the selected distribution before release approval. No private registry is required for preparation.

Establish the intended crates.io account, verified email and actual publish rights; GitHub ownership alone proves none of these. Adding a team/other owner is a separate decision, not a first-release prerequisite.

For initial publication of each new crate, select separately authorized, short-lived API credentials restricted to publishing the exact candidate names, without yank or owner-management rights. [The official first-release flow](https://blog.rust-lang.org/2025/07/11/crates-io-development-update-2025-07/#trusted-publishing) requires initial manual publication before Trusted Publishing setup. Confirm current capability/scopes before execution; no token creation or placeholder uploads to reserve names.

After ownership exists, select crates.io Trusted Publishing for repository `bohewu/yetlet` and planned `.github/workflows/release.yml`; use an environment constraint if that workflow selects one. Verify actual allowed ref/trigger behavior rather than assuming the registry checks arbitrary OIDC claims. Enable [Trusted Publishing-only mode](https://blog.rust-lang.org/2026/01/21/crates-io-development-update/#trusted-publishing-only-mode) only after verified publisher setup. Registry authentication does not replace signatures, provenance or release approval. No token, OIDC permission or publisher setting is activated here.

## Real registry and immutable promotion gates

| Phase | Candidates | Required registry boundary |
|---|---|---|
| Roots | `yetlet-core`, `yetlet-actions`, `yetlet-reactive` | Real account/name rights; all formal source/tag/integrity/approval gates pass before any upload |
| Core dependents | `yetlet-html`, `yetlet-dom`, `yetlet-dom-keyed` | Published core version is visible and resolves |
| Server | `yetlet-server-axum` | Published core/html versions resolve |
| Optional server capabilities | `yetlet-request-id`, `yetlet-session` | Published server-axum version resolves |

Prepare exact release archives before signature/digest approval. Upload those approved bytes; if Cargo would repackage, add VCS metadata or change an archive, stop and verify/approve that actual archive before uploading. Private preflight archives are not registry uploads. Download each published version and compare its digest with approval before dependent publication; verify a clean consumer without workspace path/patch overrides. The temporary local registry cannot prove real ownership, visibility or resolution.

Partial publication stops the remaining train until assessed. Retain actual versions/digests and outcomes. Do not overwrite versions, move approved tags or publish empty name reservations. Yank, replacement versions and incident actions need explicit authority under the existing release contract.

## Original policy pending inputs and next bounded task

Selected: preparation trust mode, future destination, intended owner and candidate names. Pending: actual key/algorithm/fingerprint/custody approval, qualified offline invocation, permission for key/signing operations, crates.io account/email/rights and first-publish credentials, final source/tag/archives, formal service/roots/builder/build type/tag mechanism, concrete security/release acceptance, reporting/applicable transition controls and actual registry/consumer evidence. Document approval fabricates none of these.

The next bounded task is qualifying Private public-key preparation against a concrete approved trust anchor after separate key/signing authorization. Reuse retained immutable-source evidence when inputs and subjects match. YT-020 remains Blocked; YT-065/066/067 retain bounded Done states. No public disclosure, device lab, application deployment or broad rerun is required to complete this documentation task.

## Current next preparation action

The dedicated algorithm/fingerprint and bounded Private signing/verification were subsequently qualified for the historical source; repeating that old preparation is not the next task. Prepare the source/package and formal release-policy decision record for a separately approved immutable new candidate. Its actual preflight, rebuild, provenance, SBOM/license and signing verification remain separately authorized future work. Real crates.io rights/resolution, reporting/scanner controls and concrete release/security approval remain open. YT-020 stays Blocked; YT-030/YT-065/YT-066/YT-067 retain their bounded Done scopes.
