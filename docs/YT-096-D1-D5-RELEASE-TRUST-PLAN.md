# YT-096 — Reduce release supply-chain attacks: adopted D1–D5 policy

Date: 2026-10-27 (Asia/Taipei). Documentation source:
`b834a2ba00e1be9cbe26e5326e52c457e88a039f` (original proposal).
**Current status: D1–D5 Adopted (policy); D6 Pending.** On 2026-10-27 the owner
instructed: "Adopt the recommended D1–D5 choices; keep D6 Pending. Next implement
the build/verify-only Release preparation workflow. Do not Commit/Push the new
changes, sign or publish." This authorizes the bounded workflow implementation,
not hosted execution, OIDC/certificate requests, signing or publication.

The original independent read-only review PASS, zero blocking findings, applies
to the proposal only; it does not approve the new implementation. No actual
`C_release`, final output SHA, authenticated Cosign installation or new build
result is selected or established by this adoption; no tag is created.

## Purpose and consumer outcome

**YT-096 reduces supply-chain attack risk for future official release artifacts.**
A consumer should be able to verify that downloaded package bytes match a reviewed
source and an explicitly approved Yetlet build/signing identity, with traceable
evidence. For example, replacing a `.crate` after approval must fail digest checks;
a valid signature from another repository/workflow must fail identity checks.

| Threat | Proposed control | Limit |
|---|---|---|
| Package bytes replaced after review | Exact archive digests, signed provenance and frozen evidence-manifest signature | Unsigned checksums alone can be replaced together with the files |
| Someone impersonates an official publisher | GitHub workload OIDC, exact certificate issuer/SAN, source/ref checks and approved Sigstore trust roots | A compromised approved identity or CI can still sign malicious output |
| Output claims the wrong source or build | Co-located immutable source/workflow, fresh run-bound provenance and independent output/evidence comparison | Self-reported provenance is not independent proof of every build claim |

This work provides release integrity and traceability. It does not establish
defect-free code, complete compiled/system inventory, a SLSA level or production
certification. Public source-preview use is already available after
[YT-206](YT-206-PUBLIC-PREVIEW-DISCLOSURE.md); this work adds no Yetlet runtime API.

## Adopted choices — 2026-10-27

| Decision | Concrete recommendation | Status |
|---|---|---|
| D1 — Public identity/log disclosure | GitHub CI workload identity for `bohewu/yetlet`, proposed `release.yml@refs/tags/v0.1.0`; accept the certificate/log/TSA disclosure listed below | Adopted (policy) |
| D2 — Public payload/attachments | Publish only the listed sixteen-subject provenance, its verification bundle and an approved release summary; keep the complete detailed evidence assembly private | Adopted (policy) |
| D3 — Source/cohort | Sixteen coordinated experimental `0.1.0` source packages; use118f3d0 as technical reuse baseline and a future co-located `C_release` for the reviewed release workflow and proposed `v0.1.0` tag | Adopted (policy) |
| D4 — Service/verifier | Sigstore Public Good production services; propose Cosign3.1.4 with the exact reference binary/root/config identities below, subject to bootstrap/compatibility verification before execution | Adopted (policy) |
| D5 — Claims/evidence | in-toto Statement v1/SLSA provenance v1 with the existing proposed Yetlet source-package build type; claim the actual sixteen source-package build and bounded evidence scope only | Adopted (policy) |

**D6 remains Pending.** The owner separately authorizes build/verify-only workflow
implementation now. Actual immutable source selection, hosted execution, final
payload/manifest/bundle approval, OIDC/certificate requests, signing and publication
still require their explicit later authority.
No future SHA, timestamp, certificate or output digest is invented here.

## D3 — Candidate source and exact package scope

| Source | Identity | Role in this proposal |
|---|---|---|
| Original nine-package candidate | `ce56e0814e5423782440acd2c959ca777d572cd4` | Preserve its original FAIL/release-ready NO, later bounded dispositions and historical signatures |
| Accepted sixteen-package technical C | `118f3d0f496b487ef20fe243f4e73179f9eadc43`, tree `07faa3e90c0fca5d27a4e55262c75acf851d884a` | Reuse baseline for actual archives, inputs and scoped YT-202–205 evidence; not formal signing approval |
| Public source-preview master | `d7065bbfb5b733c418f65ef55324f4d0ce6ac4d6` | PR12 squash merge; package/runtime/manifests/locks/xtask/workflows match technical C; later Markdown differs |
| Original packet documentation source | `b834a2ba00e1be9cbe26e5326e52c457e88a039f` | Adds accepted YT-206 completion documentation; not a selected formal candidate |
| Proposed formal source | `C_release` — no SHA exists yet | A future reviewed commit containing the release workflow and minimal recipe/identity adaptation; source HEAD, workflow SHA and tag target must agree |

None of the listed historical sources has `.github/workflows/release.yml`. Adopt
the co-located model from the [original formal proposal](YT-096-FORMAL-TRUST-PROPOSAL.md):
implement/review the authorized manual build/verify-only workflow now, compare
its actual delta against technical C, then approve the resulting immutable commit
and proposed `refs/tags/v0.1.0`. Keep the existing private-rebuild guard intact;
do not spoof its environment or relabel its preparation-only output as release.

The sixteen source archives below are the **actual retained118f3d0 baseline**,
not the unbuilt `C_release` subjects. All versions are `0.1.0`, sizes are bytes,
and all16 hosted/local archive digests match. Final release digests must come from
the actual later build. A source/workflow change, repackaging or VCS metadata can
change bytes even when package code is unchanged.

| Archive | Bytes | Baseline SHA-256 |
|---|---:|---|
| yetlet-actions-0.1.0.crate | 11580 | `4c7e7d33569250408269bb2076389b1892b56c13766b0d95a8350fc600702b15` |
| yetlet-auth-0.1.0.crate | 29112 | `6af397f8a8cbe829c050e73d7144bb4a58531dcc6058e1bd9487eb3e8075bcb9` |
| yetlet-config-0.1.0.crate | 23694 | `4e7fbbaf7a33359d0b9bbe226ceae1da7fe3949037f0d6eefa6ea9c8fa4a4d74` |
| yetlet-core-0.1.0.crate | 9660 | `14e504e43f645f8c2c6da0198dce2b3db88ae1b940d865149da149764f46678c` |
| yetlet-dom-0.1.0.crate | 13062 | `ed096b7326b03965289fc7c25271602d9cd211d5b795670e69aab0b63d5ea972` |
| yetlet-dom-keyed-0.1.0.crate | 11511 | `6cadadcdbd94390f576a18ec2c3f2564ca4222d26a4ec07b37883e8861737d91` |
| yetlet-html-0.1.0.crate | 9819 | `bc81e59c7a0557357606577ffd5ed50149b918aed2c097fd5924b53cc2dd846e` |
| yetlet-rate-limit-0.1.0.crate | 29144 | `25bded03607fc6d0252742f74ca6e7ad712f36566e424bd748df77a7dc9f60c6` |
| yetlet-reactive-0.1.0.crate | 14487 | `652d881a49c7bbfa260675c0d22d6249abadb559d958f35b0c8c0cded6b6cdd3` |
| yetlet-request-id-0.1.0.crate | 13133 | `72d45748743d2cd1df54c24ecac503279023054e85bac22e0546a5b4858b0c76` |
| yetlet-secrets-0.1.0.crate | 20999 | `65f6f9e38947cd9c3d15f74700d6b91b2466c4755a81954ed00acbb759365e9b` |
| yetlet-server-axum-0.1.0.crate | 25522 | `737590d701d4ae226bba9384374b8b0e730432ae913b37c4c7151f13ead16ddc` |
| yetlet-session-0.1.0.crate | 18836 | `9569604ab596b6872df0e507650b8158b818d8b7e5f87a422a3e68f5d7a28c41` |
| yetlet-session-redis-0.1.0.crate | 44092 | `74da1335a76a13d91ee47a4bb7ac1969db685f9c454fc3522698447d4633f0ae` |
| yetlet-signing-0.1.0.crate | 18389 | `743ccd822ff7cb54d0228122948e6a10dfd987e37828a504154a46406e0b1a17` |
| yetlet-wgpu-0.1.0.crate | 26729 | `bcb08644d8a55811a4d748883a06b76c2ea4a082a6cc9cd259fb4ce6494b0a21` |

Later publication order uses normal/optional edges: Actions/Core/Rate-limit/
Reactive/Secrets/Signing/wgpu; then Config/DOM/keyed DOM/HTML; then Axum adapter;
then Request-ID/Memory Session/Redis Session; then Auth. Development edges remain
separate. A partial registry upload stops the train for owner disposition;
successful versions/digests and failures stay recorded. No tag movement, silent
repackaging, overwrite or automatic yank/resume is approved.

## D1 — Public identities and service exposure

These are adopted release-policy values, not a certificate requested now.

| Public field/class | Proposed value or constrained runtime value |
|---|---|
| OIDC issuer | `https://token.actions.githubusercontent.com`; workload identity, not a personal browser/email signer |
| Certificate SAN | Exactly `https://github.com/bohewu/yetlet/.github/workflows/release.yml@refs/tags/v0.1.0` |
| Repository/owner URI and identifiers | `https://github.com/bohewu/yetlet`, repository ID1409587171; `https://github.com/bohewu`, owner ID35250002 |
| OIDC token subject, if emitted as certificate metadata | Current read-only settings use the immutable default prefix `repo:bohewu@35250002/yetlet@1409587171`; without an environment claim the proposed tag subject is `repo:bohewu@35250002/yetlet@1409587171:ref:refs/tags/v0.1.0` |
| Source/build-signer/build-config URI and digest | The reviewed release workflow URI, `C_release` and exact tag mapping; no reusable-workflow/source split is selected |
| Workflow display name, if emitted in legacy certificate metadata | Proposed `Yetlet source-package release`, bound to the reviewed workflow bytes |
| Trigger, runner and source visibility | `workflow_dispatch`, `github-hosted`, `public`; proposed Ubuntu24.04 GNU build |
| Invocation and time | Actual `https://github.com/bohewu/yetlet/actions/runs/RUN_ID/attempts/ATTEMPT`, start/finish and certificate validity times |
| Certificate/log-generated material | Ephemeral public key, serial, certificate/chain/SCT, log ID/index/checkpoint/inclusion material, signing digest/signature and TSA timestamp material |

[GitHub's OIDC reference](https://docs.github.com/en/actions/reference/security/oidc)
defines the immutable subject format. Its raw `sub` value differs from the X.509
SAN; neither is a raw bearer JWT. The [Fulcio certificate/OID specification](https://github.com/sigstore/fulcio/blob/main/docs/oid-info.md)
describes build signer/config, runner, source/owner, invocation, visibility and token
subject extensions. Allow those listed identity classes and their documented legacy
aliases; do not assume every JWT claim is copied. An unexpected identity field,
personal-email SAN, custom subject/environment or different workflow requires
review before issuance. The certificate CA issuer is distinct from the OIDC issuer.

GitHub authenticates the workload; Fulcio receives the OIDC authentication and
public-key request; CT records the certificate. Rekor/TSA receive the approved
signing/attestation material or timestamp digest. Treat the complete selected
statement and its DSSE bundle as public, including persistent copies/correlation
through transparency records. Repository deletion cannot retract those records.
Raw JWTs, private keys, tokens and credential/environment dumps must never become
public payload fields. See [Sigstore's security model](https://docs.sigstore.dev/about/security/).

## D2 — Exact statement fields and attachment policy

The following closed field set is the adopted public in-toto statement policy. Runtime
slots must be populated from the real approved execution and compared before
signing; this table is not a runnable or signable attestation.

| Field path | Value/source |
|---|---|
| `_type` | `https://in-toto.io/Statement/v1` |
| `subject[].name`, `subject[].digest.sha256` | Exactly the sixteen archive basenames above and actual `C_release` output SHA-256 values |
| `predicateType` | `https://slsa.dev/provenance/v1` |
| `predicate.buildDefinition.buildType` | `urn:yetlet:build:cargo-source-package:v1`, the existing proposed Yetlet recipe defined below |
| `predicate.buildDefinition.externalParameters` | `workflow.repository`, `workflow.path`, `workflow.ref` as the approved repository/release path/tag; closed manual input `cohort` = `sixteen` |
| `predicate.buildDefinition.internalParameters` | Pinned Rust release1.99.0, GNU target, default-per-package feature policy, OpenSSL3.5.9 and actual source epoch; no raw environment or command dump |
| `predicate.buildDefinition.resolvedDependencies[]` | `uri`/logical `name` and `digest` for actual Git commit, source archive, all six selected locks, recipe/build tools and OpenSSL source/executable identities actually used |
| `predicate.runDetails.builder.id` | `https://github.com/actions/runner/github-hosted`, scoped by the exact certificate/repository/workflow/source checks |
| `predicate.runDetails.metadata` | Actual `invocationId`, `startedOn`, `finishedOn` |
| `predicate.runDetails.byproducts[]` | Logical names and SHA-256 for source-input, build-record, archive-index and inventory/coverage records; private detailed contents are not embedded |

Reject extra public fields unless reviewed. Public logical names must be portable
artifact labels, without filesystem paths, private URLs, mailbox addresses or
raw scan findings. Source/run/tool hashes can still disclose or correlate known
bytes; they are part of the owner's D2 choice.

| Attachment | Recommended disclosure |
|---|---|
| Sixteen approved `.crate` files | Eligible for later explicit release distribution; their exact bytes must match the signed subjects. Registry upload remains separate D6/YT-020 authority |
| Final provenance JSON and DSSE Sigstore bundle | Public, including full approved statement, certificate and log verification material |
| Complete evidence checksum manifest and its detached-signature bundle | Keep under the selected private evidence access control; accept public exposure of the manifest digest/signature/certificate through signing services |
| Detailed source/build commands, environment, SBOMs, coverage, scans, old Windows/operational records | Private by default; no blanket evidence-directory upload. Existing GitHub source/Actions disclosure under YT-206 is separate |
| Final owner approval summary | Publish only approved source/tag/subject and manifest/bundle digests plus support/coverage limits; no raw private record content |

The public consumer can verify the sixteen archives using the public provenance
bundle. An independent technical verifier with private-evidence access also
checks every complete-manifest entry. A public summary never replaces that
complete manifest or implies that an unauthorised reader verified private files.

### Adopted D2 encrypted custody extension — 2026-10-27

The owner adopts age ciphertext transport and safe receipt disclosure: public
recipient/fingerprint, ciphertext type/size/digests, source/workflow/run/attempt,
original exit/status/completeness and inventory/manifest verification metadata.
The eighteen-file public package set and closed provenance fields remain unchanged;
wrappers/receipts stay outside the frozen evidence assembly. Complete plaintext
private records and private identities never enter Git, build-job inputs, logs or
artifacts. The latest owner choice permits the identity transiently only in a
separate isolated, manually approved protected GitHub Environment verifier,
superseding the earlier local-only CI policy. Current authority covers code
integration and local proof, not actual settings/Secret installation or hosted
execution. Independent integration review PASS, zero issues and required followups.
See [custody integration and manual rotation](YT-096-AGE-CUSTODY-INTEGRATION.md).
This encryption extension does not authorize OIDC, signing or publication; D6
remains Pending and durable retention requires actual readback/custody evidence.

## D4 — Concrete signing and verification recommendation

Recommend the public production Sigstore route, with no new KMS/private service.
The upstream [Cosign3.1.4 release](https://github.com/sigstore/cosign/releases/tag/v3.1.4),
published2026-10-20, is the adopted policy pin;3.1.3 remains the historical design/
bootstrap reference. Actual bootstrap, validity and compatibility verification
remain required before execution. No binary is
downloaded, installed or executed here.

| Reference input | Observed/proposed exact identity |
|---|---|
| Cosign3.1.4 source | Commit `0e48502cf5ef34e2e60bbd3617aa19945712f705` |
| Linux amd64 release binary | `cosign-linux-amd64`, upstream release-API SHA-256 `424e2c97d0b45fd7a6b608a87f090f35633f4c23ece5f092d48bba11aaf4bbfa` |
| Production trust/config source snapshot | [sigstore/root-signing commit f39569f](https://github.com/sigstore/root-signing/tree/f39569f398fa958b314611e75c637c8d92e6bd4f/targets) |
| `trusted_root.json` reference SHA-256 | `6494e21ea73fa7ee769f85f57d5a3e6a08725eae1e38c755fc3517c9e6bc0b66` |
| `signing_config_rekor_v2.v0.2.json` reference SHA-256 | `0f5f38554e29e770d4d5d6f0e1b51fcbf84f61dc6934530a09b7a901eaad5bee` |

That published signing profile lists Fulcio `https://fulcio.sigstore.dev`, approved
Rekor endpoints `https://log2025-1.rekor.sigstore.dev` (API2) and
`https://rekor.sigstore.dev` (API1), with `ANY` log selection, and TSA
`https://timestamp.sigstore.dev/api/v1/timestamp`. Its interactive OIDC URL is
`https://oauth2.sigstore.dev/auth`; the proposed CI route uses GitHub workload
OIDC instead of silently switching to personal interactive authentication.
Both log endpoints are explicit adopted policy choices, not an unreviewed fallback.

These SHA-256 values bind read-only upstream reference bytes, not a qualified
installation, TUF bootstrap, root refresh or live-service acceptance. Before
execution, independently authenticate the tool, validate production TUF/root
material and validity, freeze the actual trusted-root/signing-config digests and
check3.1.4 compatibility. A changed service/root/profile or output format needs
review; do not silently use `latest`, staging, artifact-supplied roots or disabled
certificate/log checks. Existing3.1.3 fixture PASS remains historical.

Use Cosign's supplied-statement attestation path for the sixteen-subject statement
and detached blob signing for the complete manifest. Its [pinned implementation](https://github.com/sigstore/cosign/blob/0e48502cf5ef34e2e60bbd3617aa19945712f705/cmd/cosign/cli/attest/attest_blob.go)
accepts statement bytes; it does not establish their truth. Recommend the
[Sigstore v0.3 bundle](https://docs.sigstore.dev/about/bundle/) with a DSSE envelope
for provenance and message signature/digest for the manifest. Require valid
certificate/chain/CT and approved Rekor inclusion/checkpoint verification, plus
any selected timestamp verification; a timestamp alone cannot replace the log.
Actual bundle/log formats and their disclosure must be checked before issuance.

## D5 — Build definition, evidence and acyclic binding

The proposed `urn:yetlet:build:cargo-source-package:v1` retains the existing formal
recipe's meaning and expands only the proposed cohort to sixteen: export the
approved co-located source; preserve six independent manifests/locks and known
license materialization; use fresh Cargo/target state, pinned Rust1.99.0/GNU,
OpenSSL3.5.9 and default package features; vendor the selected source closure;
collect bootstrap inputs separately and verify each final archive with normal
offline locked Cargo packaging. Record all actual inputs, tools, commands,
outputs and errors. Bootstrap collection is never a verification PASS.

The authorized preparation workflow remains manual-only with a contents-read
build/verify job and no OIDC or signing path. A future separately authorized
formal signing extension needs a narrowly scoped OIDC signing job after approved
output/payload review.
It must reject source/workflow/tag disagreement and unreviewed inputs. The
existing private helper remains preparation-only; formal adaptation needs its
own implementation review. No mandatory runtime dependency or custom crypto/
verifier/validation framework is proposed. The [SLSA build-provenance schema](https://slsa.dev/spec/v1.2/build-provenance)
defines the statement's structure; using it does not confer a SLSA level.

| Retained evidence | Exact reference SHA-256 | Claim and limit |
|---|---|---|
| `sixteen-subject-proposal-C.json` | `98af60ad9812fbb3b51d88fa87e3932b519a31b82153aa7f3ead0ab8bf9c2c14` | Actual118f3d0 names/sizes/digests; unsigned baseline |
| `hosted-local-rebuild-comparison.json` | `990cea4b84fb76f4887e8eac3d586ca49172136b826279a743da9cc63acbd3d8` | Both33 phases complete,16 archive bytes equal; source packages, not executable reproducibility |
| Hosted `PROVENANCE-UNSIGNED.json` | `f06b257ae47f58bb8bcfea7c284ed3307b1927ba67bba719769b230e7b90b8b0` | Preparation-only claims from private-rebuild.yml; do not sign/relabel as a fresh formal event |
| `retained-archive-lock-reconciliation.json` | `cc63d747e52974db4740f9ebb8cd1c08b1d45969617e688b2bb31dd0e21ba833` | Accepted archive lock graphs and justified evidence reuse; not registry rights/resolution |
| `reader-source-inventory-C.json` | `d444e88da31b86fc9b1a2057291a9a94632311240e275c6f347a57a87e9f2b6e` |46 source-bound reader manuscripts; not a published/sanitized reader site |

The records reside under `target/yetlet-yt205-execution-20261027-qecriekl/`;
the [accepted YT-205 consolidation](YT-205-RELEASE-EVIDENCE-CONSOLIDATION.md#authorized-current-c-execution--2026-10-27)
maps required CI, current-lock GNU/MSVC, packaged WASM/DOM, scoped GPU,
license/advisory/source-security and rustdoc evidence. Preserve each input scope:
MSVC excludes Signing/Redis Windows and actual GPU; old GPU proof is limited to
its selected Firefox configuration; source-resolution inventories omit complete
compiled/native/system closure. The denied Windows result-ZIP transfer remains
denied; prior accepted original text records are not relabelled as a local ZIP.

For a later formal execution:

1. Review the real `C_release` delta; reuse unchanged product evidence and run only
   changed-path/source/output checks needed for that actual workflow and tool pin.
2. Build the sixteen source packages and derive fresh provenance from that run.
   Bind source export, all six locks, build records and actual final archive bytes.
3. Assemble the complete evidence checksum manifest including final provenance
   and all required detailed records. Freeze it. Keep both signature bundles
   outside its inputs; provenance must not hash this containing manifest or itself.
4. Only after separate D6 authority, obtain workload certificates/log entries and
   produce the provenance and manifest bundles. Preserve actual failures/proofs.
5. Independently verify downloaded bytes, exact identity/source/tag, certificate/
   log proofs, sixteen subjects and complete private-manifest entries. Record final
   owner approval outside the frozen manifest, referencing its and both bundles'
   digests. Promote only those approved bytes; any repackaging stops promotion.

This is the remaining formal integrity work, not a repeat of all accepted product
qualification. Registry account/rights, immutable upload/download comparison,
partial-publication disposition and actual publication remain separate YT-020/D6
operations. No new operational-readiness or business SLO gate is added.

## Review and current authority

**D1–D5 Adopted (policy); D6 Pending; YT-096 In progress and YT-020 Blocked.**
YT-206 remains Done at its source-preview/reporting scope. The owner authorizes
build/verify-only workflow implementation with new changes left uncommitted and
unpushed. This record makes no implementation PASS claim. Actual source/tag,
output/payload and final signature execution remain unapproved; no OIDC,
certificate, signing, publication or hosted run is authorized here.

Earlier nine-package reviews, failures, tool pins and sixteen-package technical
evidence keep their original identities and scope. The original proposal's
new-document checks and independent read-only review do not attest the later
implementation. Current source/workflow/root/output values still need actual
future execution bindings; reference hashes are not authenticated installation
or qualified TUF/root evidence. No old qualification is replayed by this adoption.
