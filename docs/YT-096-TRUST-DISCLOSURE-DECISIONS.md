# YT-096 — Release supply-chain integrity: trust and disclosure decisions

**Subsequent owner-approved GitHub pre-release (2026-10-29):**
[Yetlet v0.9.0-alpha.1](https://github.com/bohewu/yetlet/releases/tag/v0.9.0-alpha.1)
is public as an Experimental pre-release, not Latest, at unchanged C `bf33638`.
The owner separately approves publication of the exact notes and 20 public
attachments; anonymous API and bundle-download readback PASS. No private evidence,
private bundle, registry upload, new signing or Git operation is included.
D6 remains Complete, YT-096 Done at accepted source-package signing/integrity
scope, YT-020 Blocked. Earlier unpublished/Pending records retain their dated scope.

**Purpose:** Reduce package substitution, official-signer impersonation and
source/build mismatch risks for future Yetlet releases. Consumers need verifiable
artifact bytes and an approved source/workflow identity; signatures alone do not
prove defect-free code or an uncompromised CI system.

**Actual signing and final condition (2026-10-29):** Owner-approved sign
`40811275645/1` at S `d65e318` SUCCESS; independent public/private verification,
all 16 subjects, all 117 complete-manifest entries and plaintext cleanup PASS.
The owner explicitly grants one bounded local private-key readback exception
and conditional acceptance of the existing public/encrypted-private bundle
digests in the [actual acceptance record](YT-096-SIGNER-CONTRACT.md#actual-signing-independent-verification-and-owner-acceptance--2026-10-29).
Final independent Reviewer PASS0 activates the existing owner acceptance;
D6 Complete and YT-096 Done at accepted source-package signing/integrity scope.
The default D2 rule remains unchanged for future work; actual disclosure stays
within adopted D1/D2 scope. No registry/GitHub Release publication or Git mutation;
YT-020 Blocked. Earlier decisions below retain their original dated statuses.

**Current execution and owner decision (2026-10-29):** D1–D5 remain Adopted
(policy); D6 Pending. The [single D6 confirmation packet](YT-096-SIGNER-CONTRACT.md#d6-confirmation-packet--2026-10-29)
binds actual alpha C `bf33638`, unchanged unsigned `v0.9.0-alpha.1`, signer
S `d65e318`, successful owner-approved prepare `40806988578/1`, both frozen raw
payload hashes and exact artifact/receipt/custody. The
[result record](YT-096-SIGNER-CONTRACT.md#successful-protected-prepare-and-retention--2026-10-29)
preserves failures and confirms cleanup/retention without signing or publication.
The proposed owner decision accepts public statement, workload identity and
durable certificate/log metadata for one later protected signing run; private
manifest/evidence bodies remain controlled. No such decision is taken by this
documentation. Signing/verification/final acceptance and publication remain
separate gates. Earlier adoption/proposal values below retain their dated scope;
they are not current alpha signing inputs. YT-096 In progress; YT-020 Blocked.

**Current owner adoption (2026-10-27): D1–D5 Adopted (policy); D6 Pending.**
The owner adopts the [concrete D1–D5 plan](YT-096-D1-D5-RELEASE-TRUST-PLAN.md):
GitHub workload OIDC/public Sigstore metadata and log disclosure; the closed
sixteen-subject public provenance/DSSE bundle with complete detailed evidence
private; sixteen experimental `0.1.0` source packages using `118f3d0` as the
technical baseline and a future co-located `C_release`/`release.yml` with proposed
`v0.1.0`; Cosign3.1.4 and the exact reference root/config identities subject to
actual bootstrap, validity and compatibility checks; and source-package-only
provenance/evidence scope with acyclic bindings. Reference hashes are not qualified
TUF/bootstrap evidence.

The owner also authorizes implementing the build/verify-only Release preparation
workflow, leaving new changes uncommitted/unpushed. No real `C_release`, tag, final
output/payload, authenticated Cosign installation or new build result is approved
by adoption. Hosted build/verify execution still needs separate explicit
authority and may be authorized while D6 remains Pending. D6 holds identity
issuance/signing; publication needs separate explicit approval. Policy adoption
grants none of those execution authorities. YT-096 stays In progress, YT-020 Blocked
and YT-206 Done. Original review PASS does not approve new implementation.

## Historical decision brief — 2026-10-23

The remaining text records the original nine-package recommendation and Pending
decisions as of 2026-10-23. Its source, failures, pins, results and review remain
unchanged; the adoption above supersedes those Pending policy statuses.

**Date:** 2026-10-23. **Documentation source:** `a31f9ceed1579e8c6277d1d03102b2a12bd1e93f`.
**Status:** Independent Reviewer PASS, zero issues and required followups. All
owner decisions below remain Pending. This is not an approved signing policy.

## Recommendation and unchanged candidate

Retain the [existing proposal](YT-096-FORMAL-TRUST-PROPOSAL.md): public Sigstore
with GitHub Actions workload identity is the preferred conditional route if the
owner accepts its public identity and payload disclosure. First decide the
disclosure boundary, then the source/workflow model. Approve no certificate
request, signing, tag or publication through this document. If the disclosure
is unacceptable, keep the formal gate open rather than weaken log checks or
introduce a private signing service without a separate need/decision.

The selected candidate remains
`ce56e0814e5423782440acd2c959ca777d572cd4`, original nine experimental unpublished
0.1.0 source packages, Rust MSRV 1.99 / qualification pin 1.99.0. This brief does
not select current documentation HEAD, M21 capabilities or any future commit C
as release source. Existing archives, SBOMs, builder/platform evidence, owner
waivers and historical failures retain their exact scope; none is rerun,
regenerated, re-signed or relabelled. No new release guarantee is inferred.

The [original candidate review](YT-096-SECURITY-RELEASE-REVIEW.md) remains
**FAIL / release-ready NO**. The [owner's support/inventory disposition](YT-096-FINAL-RECONCILIATION.md#owner-support-and-inventory-acceptance-2026-10-20)
resolved that disposition only: exact-candidate Chrome qualification and
complete compiled/system inventory stay Not verified. YT-096 stays In progress;
YT-020 stays Blocked. The original nine-package release scope is unchanged.

## What would become public

These are exposure classes for the proposed future route, not metadata emitted
by this task. No certificate or bundle has been requested/generated here.
Distinguish public CA/log submission from separately publishing a bundle,
evidence attachment, repository or source package.

| Surface | Metadata to treat as public | Point of disclosure and impact |
|---|---|---|
| Fulcio certificate and certificate transparency | Workload identity/SAN, OIDC issuer, public signing key, serial/validity and certificate/chain verification data; additional workflow OIDC extensions may also be present | Certificate issuance can disclose identity before any artifact release. The proposal's SAN exposes `bohewu/yetlet`, `.github/workflows/release.yml` and the proposed `refs/tags/v0.1.0`; no such tag/identity is approved or created now |
| Workflow/source identity in certificate extensions or signed provenance | Approved repository/source URI, commit/ref/workflow identity; selected profile may also expose owner/repository identifiers, trigger, runner/environment or workflow invocation metadata | Exact fields depend on the approved certificate/signing profile and payload. Do not assume every JWT claim is copied, or that private repository names and refs remain private. Links do not themselves grant access to private source |
| Rekor signature record and proof material | Signature/certificate or verifier material, signed artifact/payload digest, log identity/index/time and inclusion/checkpoint material as applicable to the chosen format | Public append-only evidence can be copied and correlated across attempts; deleting a GitHub artifact or repository does not retract that public record. A digest can link to already-known bytes; it is not an anonymization mechanism |
| Provenance statement and attestation bundle | Assume the full approved statement can be disclosed: nine subject names/digests, predicate/build type, source/input/tool digests, builder and selected run/attempt/timing/byproduct references | Bundle access can reveal statement content, not just an archive hash. Actual public-log entry content depends on the selected Rekor/service format; do not promise that every log stores the full statement or that only digests can escape |
| Complete evidence manifest and detailed evidence | If separately published: its logical artifact names, sizes/digests and evidence relationships; publishing referenced reports would disclose their contents too | Signing a manifest digest does not by itself publish every referenced file. Keep the complete manifest/evidence under selected access control unless its disclosure is separately approved; do not silently upload the raw evidence directory |
| Source archives, source tree and repository visibility | Actual `.crate` contents or repository access only if separately distributed/opened | Public signing metadata does not automatically publish source packages, create a GitHub Release, grant crates.io rights or change repository visibility. Those remain separate operations/approvals |

The proposed exact identity from the existing proposal is:

`https://github.com/bohewu/yetlet/.github/workflows/release.yml@refs/tags/v0.1.0`

It is a future expected identity, not the identity of a current signature.
The OIDC issuer is the proposed `https://token.actions.githubusercontent.com`;
it is distinct from the certificate CA issuer and the application's OIDC IdP.
Use the CI workload route rather than silently substituting a personal browser
login/email identity. Raw OIDC JWTs, private keys and credentials are never
approved public metadata.

[Sigstore's security model](https://docs.sigstore.dev/about/security/) describes
public signature/certificate transparency and the persistence of log entries.
[Fulcio's certificate specification](https://github.com/sigstore/fulcio/blob/main/docs/certificate-specification.md)
defines SAN/OIDC identity and permits additional OIDC extensions. These support
the identity exposure above; the repository/run/payload examples are this
brief's application of the existing Yetlet proposal, not a promise of one
universally fixed certificate extension set.

At the existing Cosign 3.1.3 design pin, the
[attest-blob source](https://github.com/sigstore/cosign/blob/v3.1.3/cmd/cosign/cli/attest/attest_blob.go)
passes supplied statement bytes into the
[attestation bundle path](https://github.com/sigstore/cosign/blob/v3.1.3/cmd/cosign/cli/signcommon/common.go).
The [CLI reference](https://github.com/sigstore/cosign/blob/v3.1.3/doc/cosign_attest-blob.md)
also exposes signing-configuration selection. Therefore exact service/log entry,
bundle format and payload exposure must be reviewed together before activation;
this brief does not infer them from a floating default or execute the tool.

## Pending owner decisions

Each row is a separate decision; accepting the research or a recommendation does
not silently approve the others. All statuses are Pending, with no default yes.

| Decision | Choice and impact | Recommendation and required boundary |
|---|---|---|
| D1 — Public identity and durable disclosure | Accept the public workload/repository/workflow/ref and signature-log exposure, or defer formal public signing | Conditionally prefer public Sigstore if that exposure is acceptable. Obtain explicit disclosure approval before the first OIDC/certificate request; an unpublished/Private repository does not make the proposed flow private |
| D2 — Public payload versus private evidence | Approve the exact statement field list and which manifest/bundle/report attachments may be public; private detailed evidence can remain access-controlled | Treat the whole selected statement as potentially public. Review its exact data-only representation before signing, with no raw reports/environment/logs or blanket artifact-directory upload. Keep complete evidence bindings; a public summary must not replace the required complete frozen manifest |
| D3 — Source/workflow identity model | Keep the current candidate frozen now; later choose the proposal's co-located C/workflow/tag or separately research a source/workflow split | Defer any replacement candidate under this task. When separately authorized, co-location is the simpler existing proposal. A split retaining ce56e08 needs its own reviewed identity design; do not relabel old executions or spoof existing helper claims |
| D4 — Service, verifier, roots and exact identity | Approve pinned verifier/configuration, current service/root/log material, exact issuer/SAN/repository/workflow/ref/source and builder policy | Retain Cosign 3.1.3 as the existing design pin, not a refreshed/verified service configuration. Exact root/configuration validity, service acceptance and entry/bundle formats remain Not verified until separately authorized. Reject wildcard identity and disabled certificate/log checks |
| D5 — Claims and evidence binding | Approve provenance recipe/build type, subjects, inventory limits and the source-to-evidence mapping for the eventually selected execution | Preserve the existing in-toto/SLSA-v1 proposal and independently check claims against that actual build. Old PASS evidence remains source-bound; signing old self-reported provenance now would not authenticate its historical builder. Reuse evidence only where the future source/input/output bindings justify it |
| D6 — Authority for later operations | Separately select workflow implementation/candidate, hosted execution, public issuance/signing, downloaded-output verification and eventual promotion | Authorize none now. A docs Reviewer PASS is not source/tag/signing/release approval. Registry proof, public activation/reporting controls and concrete final maintainer approval remain separate open gates; publication is not implied by metadata consent |

The evidence to approve D1/D2 later must be a concrete field/payload and
attachment listing, with an explanation of which services receive it. A general
approval of "metadata" is insufficient for an unreviewed predicate or log kind.
No consent prompt bypass, unattended confirmation option or client default can
substitute for that owner decision. This is a future authorization requirement,
not permission to generate signed material or contact signing services now.

## Minimum disclosure recommendation

The public provenance may carry only the necessary reviewed facts: selected
source/repository/ref/workflow identity, the nine source-package names and
SHA-256 subjects, approved predicate/build type and builder, selected toolchain
and build parameters, invocation/time references and source/lock/build-record
digests. Preserve truth and verifiability rather than hide a required identity
claim. Exact final values await the source/execution decisions; current HEAD
must not be substituted for the selected candidate.

Keep raw build commands/environment values, private filesystem paths, complete
scan findings/logs, internal URLs, personal contacts, source contents and all
tokens/secrets out of the public statement. Required detailed commands,
inventory/coverage, reports and source-input/build records still belong in the
complete private evidence set with their required digest bindings. Review
logical names and digests too: they can disclose structure or support matching
known reports. Do not delete required evidence, sign a partial manifest as
complete, or edit a payload/manifest after its approved bytes are frozen.

For the existing acyclic design: final provenance is included in the complete
evidence manifest; the signature bundles remain outside that manifest's inputs.
Provenance does not hash its containing manifest, and the manifest does not hash
itself or its own signatures. Final maintainer approval references frozen
manifest/bundle digests externally. Disclosure minimization does not change
these bindings or turn preparation-only records into formal provenance.

## Candidate/workflow constraint and alternatives

The original candidate has no proposed `release.yml`. The existing
[private rebuild workflow](../.github/workflows/private-rebuild.yml) is read-only;
its [helper](../xtask/src/release/private.rs) requires source HEAD/GITHUB_SHA,
workflow SHA and its exact private-rebuild path to agree. It emits preparation-only
provenance. The documented proposal cannot be activated unchanged while
pretending that another workflow commit is ce56e08. This task changes no helper,
workflow, ref/tag or identity guard, and selects no future C.

If public identity/payload disclosure is rejected, preserve current private
preparation and defer the formal gate. GitHub's
[managed private-repository attestation service](https://docs.github.com/en/actions/concepts/security/artifact-attestations)
has no transparency log; entitlement alone does not close Yetlet's existing
log requirement. The existing maintainer-key preparation similarly does not
supply formal OIDC/certificate/log/builder guarantees. A privately operated
formal service is a separate operational/security commitment, not the minimum
next step recommended here. Do not disable log verification to obtain PASS.

## Evidence, review and next step

This is source/documentation and primary-reference research only. Following the
owner's no-validation-rerun instruction, no `cargo xtask check docs`, tests,
builds, rebuilds, scans, browser/platform suites or prior qualification checks
were rerun. References/source were read; no signed payload, certificate, key,
OIDC request, tag, root refresh, hosted workflow or publication was produced.
No private settings/credentials were read or services started. Existing
candidate evidence/results/waivers are reused without recertification.

Independent Reviewer
`/root/yt096_trust_disclosure_decisions_review_20261023` returned **PASS**, zero
issues and zero required followups. All five document hashes, HEAD and Git
status were unchanged during review; the reviewer role/model/effective effort
was verified before acceptance. PASS applies to this documentation brief only.
After review, edits only record acceptance and evidence, with no decision or
contract changes. The repo-external directory
`/tmp/yetlet-yt096-trust-decisions-20261023-toqeud__/` retains `baseline.json`,
`primary-references.json`, `review-source.json`, `independent-review.json`,
`review-acceptance.json` and `closure-observations.json`. The last record lists
final document hashes and static Git observations, not validation reruns.
Changes are this brief, a dated navigation addendum to the original proposal,
and Backlog/Roadmap/docs-index links. No original review verdict, policy,
workflow or candidate input is changed.

The next step is the owner's D1/D2 disclosure decision, followed by D3 identity
scope if continuation is selected. This report asks for no execution authority
and makes no owner decision. YT-096 remains In progress, YT-020 remains Blocked;
HEAD remains `a31f9ce`, staged files zero, no commit or push.
