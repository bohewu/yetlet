# YT-096 — minimum formal trust and provenance proposal

Date: 2026-10-20. **Research only; proposed, not an approved signing policy.**
The reviewed support/inventory decision was committed as `0bd7596` after a
strict/builtin staged Commit Guard PASS (seven files, zero findings, matching
tree). It was not pushed. The selected product candidate remains
`ce56e0814e5423782440acd2c959ca777d572cd4`; no replacement candidate is selected.

The owner requested this research and independent reviewer review, excluding
signing and publication. No workflow/code change, key access, OIDC request,
tag creation, signing, transparency upload, hosted run, registry operation or
visibility change is performed. [Supply-chain policy](RELEASE-SUPPLY-CHAIN.md#independent-rebuild-provenance-and-approvals)
and [Private trust policy](YT-020-PRIVATE-TRUST-POLICY.md) remain unchanged.

## Recommendation and service choice

Use a future manually dispatched GitHub-hosted `release.yml`, GitHub Actions
OIDC and the existing Cosign 3.1.3 design pin with public Sigstore
Fulcio/Rekor. Produce one source-package provenance attestation plus one
signature over a complete evidence checksum manifest; verify them separately
before any promotion. No new KMS, Azure vault, registry or signing service is
needed. This is a proposed use of mature tooling, not a new Yetlet API or
cryptographic verifier.

The recommendation is conditional on explicit approval of public signing
metadata disclosure. Certificates identify the repository/workflow/ref;
public log entries are persistent and may expose digests and attestation data.
Review the exact signing format and payload before the first OIDC/certificate
request, not after signing. Do not upload secrets, raw environment dumps,
private scan details or unreviewed logs. A private GitHub repository does not
make public Sigstore operations private. Repository visibility and package
publication are separate decisions.

| Route | Fit with the existing formal contract | Recommendation |
| --- | --- | --- |
| Existing maintainer-key Private preparation | Authenticates approved historical bytes, but lacks GitHub OIDC/certificate/transparency/builder proof | Preserve the old evidence; do not repeat it or call it formal release trust |
| GitHub private-repository artifact attestations | GitHub documents Enterprise Cloud entitlement and a private Sigstore instance without a transparency log | Buying entitlement alone cannot close the required log/inclusion gate; not the minimum route |
| GitHub OIDC plus public Sigstore/Cosign | Can supply workload identity, certificate and log evidence without a long-lived project signing key | Preferred after disclosure, identity and source/tag approvals; actual Yetlet execution remains Not verified |
| Public-repository GitHub artifact attestations | Managed public Sigstore integration is an alternative after a separately approved Public transition | No visibility change is implied; avoid coupling the first trust experiment to publication |
| A private Sigstore deployment with approved roots/logs | Could retain confidentiality but adds service operation, roots, log custody and qualification | Defer unless public disclosure is unacceptable and private formal signing is required now |

The entitlement/log distinction is documented by
[GitHub's attestation overview](https://docs.github.com/en/actions/concepts/security/artifact-attestations)
and [availability guide](https://docs.github.com/en/actions/how-tos/secure-your-work/use-artifact-attestations/use-artifact-attestations).
No current account entitlement or service acceptance was tested.
[Sigstore's security model](https://docs.sigstore.dev/about/security/)
explains its OIDC, certificate and public-log trust chain. If disclosure is
not approved, keep the formal gate open; do not disable log checks.

## Source and workflow identity must agree

The current candidate has no `release.yml`. Its
[private rebuild workflow](../.github/workflows/private-rebuild.yml) is
read-only, and `xtask/src/release/private.rs` explicitly requires source HEAD,
GITHUB_SHA and workflow SHA to agree and the exact private-rebuild workflow
path. Its provenance is marked preparation-only. It cannot already execute the
proposed release flow. Do not spoof environment claims or remove these guards.

The smallest identity model is a **future, separately approved commit C** that
contains the reviewed release workflow and its minimal source-package recipe.
The proposed `refs/tags/v0.1.0` would point exactly to C; workflow, certificate
source claims and packaged-source identity must agree. This is a future
candidate decision, not permission to move an existing tag or relabel ce56e08
evidence. Review the workflow/helper delta, compare package inputs and final
archive bytes, and reuse previous evidence only where those identities match.
Changed inputs or bytes need their own bounded assessment.

Keeping ce56e08 while signing from another workflow commit requires an
explicitly reviewed source/workflow split and different exact identity policy;
the existing helper does not support that model. It adds more identity rules
than the co-located proposal, so it is not the first recommendation. No new
source-package build runs merely to complete this research.

## Required identities and trust material

All entries below are proposed activation inputs; actual enrollment/approval
is still pending unless historical evidence is explicitly identified.

| Identity | Minimum exact value or decision | Evidence to retain |
| --- | --- | --- |
| Release/source approver | Existing maintainer authority; concrete C, tag, scope, evidence digests and rollback target | Explicit approval record; candidate selection or a technical review alone is insufficient |
| Workload issuer | `https://token.actions.githubusercontent.com` | Verified certificate issuer; do not reuse the application's OIDC IdP or client secret |
| Repository and workflow | `bohewu/yetlet`, `.github/workflows/release.yml` | Approved workflow bytes/action pins, workflow commit C, actual event/run/attempt/job identities |
| Certificate identity | Proposed exact SAN `https://github.com/bohewu/yetlet/.github/workflows/release.yml@refs/tags/v0.1.0` | Exact SAN plus repository/ref/source-SHA claims and independently checked tag-to-C mapping; no wildcard identity |
| Builder | Proposed `https://github.com/actions/runner/github-hosted`, actual Ubuntu 24.04 hosted run | Reviewed recipe and successful build job; no self-hosted substitution or independent-human claim |
| Provenance semantics | in-toto Statement v1, predicate `https://slsa.dev/provenance/v1`; proposed build type `urn:yetlet:build:cargo-source-package:v1` | Approve the recipe definition below; do not silently promote an old preparation build type |
| Verifier and service roots | Existing Cosign 3.1.3 Linux amd64 pin; approved Sigstore Public Good trust material and signing-service configuration | Independently bootstrapped tool/root provenance and SHA-256, root/version validity and service/log identities; never trust roots merely shipped beside an artifact |
| Independent technical verifier | A separate review/verification execution receiving downloaded bytes and approved public trust inputs | Commands, native exit codes, policy comparisons and verdict; no private signing material or primary build cache |

The retained Cosign binary SHA-256 is
`4629c757b7618056f8ddd7e2625ae9fdd94c0372a65049520bc7d9df9efc7f71`;
[earlier bootstrap/public-fixture evidence](YT-020-INTEGRITY-PREPARATION.md#verifier-and-public-fixture-experiments)
is reusable at its original scope. Actual activation still approves the
binary, currently valid roots and signing configuration; a tool change needs a
reviewed pin decision. This research neither refreshes roots nor executes tools.

For the proposed build type, the recipe accepts only C/tag, the nine selected
0.1.0 packages, locked source dependencies, Rust/Cargo 1.99.0, native GNU Cargo
package verification, default features, recorded profile/jobs/environment and
source epoch. It exports reviewed source without overlays, materializes only
the known license aliases, uses fresh Cargo/target directories, and records
complete source-archive/lock/tool/command/output identities. Reuse the existing
recipe where applicable; the formal identity/path adaptation needs separate
implementation review. The build type covers source archives, not compiled
executables or complete system inventories. SLSA's
[provenance v1 definition](https://slsa.dev/spec/v1.1/provenance)
separates build definition, builder identity, run details and output subjects;
using that schema does not itself earn a SLSA level.

## Minimum evidence set and acyclic binding

| Evidence | Binding needed before signing |
| --- | --- |
| Approved source | C/tree/tag mapping, source export, package-input map, Cargo.lock, tool/recipe digests, target/defaults/profile/environment and commands |
| Nine `.crate` subjects | Exact names, sizes and SHA-256; compare with approved final bytes and retained rebuild outputs. Source-archive comparison does not prove binary reproducibility |
| Inventory and accepted limits | All 23 source-resolution SBOMs and archive/lock/target crosswalk, coverage differences, source-bound license/audit snapshots and the accepted support/Not verified disposition |
| Fresh build provenance | Nine archive subjects, source/lock/tool inputs, proposed build type, actual hosted builder/run/attempt/times, build-record/source-input byproduct digests and explicit inventory scope |
| Complete release evidence manifest | A new separately named assembly using the existing checksum command, covering the selected archives, SBOMs, coverage, fresh provenance/build records, retained rebuild comparisons and approved policy/support records |
| Sigstore material | DSSE provenance bundle and detached manifest-signature bundle, leaf certificate/chain material, certificate transparency evidence, Rekor inclusion proof/checkpoint and approved trusted roots; preserve actual service responses and verifier results |

Generate fresh provenance from the new successful build. Include it in the
evidence manifest, then freeze the manifest. Keep both signature bundles
outside that manifest's input directory so neither manifest nor provenance
hashes its own signature. Provenance references inputs/byproducts, not the
manifest that contains it. The separately approved manifest digest is the
root binding; old unsigned bundles/provenance remain unchanged historical
inputs, never newly authenticated records of their old execution.

Cosign 3.1.3 provides
[attest-blob with a supplied statement](https://github.com/sigstore/cosign/blob/v3.1.3/doc/cosign_attest-blob.md)
and [blob-attestation verification](https://github.com/sigstore/cosign/blob/v3.1.3/doc/cosign_verify-blob-attestation.md).
Its [statement implementation](https://github.com/sigstore/cosign/blob/v3.1.3/cmd/cosign/cli/attest/attest_blob.go)
signs supplied statement bytes; it does not establish that their claims are
true. Therefore derive and compare subjects/claims against the actual build
before signing. Use its existing blob-signature path for the frozen manifest;
no OCI registry or custom DSSE/crypto implementation is necessary.

## Future operations and authority boundaries

These are proposed operations, **not commands executed or authorized now**.

| Step | Minimum operation | Authority/evidence boundary |
| --- | --- | --- |
| 1. Decide | Approve public metadata disclosure, service/roots, exact identities, proposed build type and candidate/tag model | Research/reviewer PASS cannot grant those approvals; no OIDC request before the disclosure decision |
| 2. Prepare | Implement/review one manual release workflow and bounded recipe adaptation; pin actions/tools; select C and review applicable source/evidence delta | Separate implementation/candidate authority. No generic policy engine, custom verifier or application change |
| 3. Approve source | Approve exact source/tag association, expected package/evidence inputs and execution scope; create the tag only when authorized | No automatic tag creation, moved tags, floating release source or old signatures reassigned to C; new output digests are frozen after the successful build and before signing |
| 4. Build and attest | Run the fresh source-package build; prepare/compare provenance and manifest; obtain GitHub OIDC certificate and create the two Sigstore bundles | Separately authorize workflow execution, signing and persistent public disclosure; no registry token, package upload or Release creation |
| 5. Verify independently | Download frozen outputs, use approved Cosign/roots, check certificates/log inclusion and exact claims, verify all subjects and manifest entries | Verification uses public material only; record native success/failure and exact downloaded digests before final approval |
| 6. Decide promotion | Record security/release review and final maintainer decision, including retained limits and rollback digest | Registry qualification, public activation controls and publication remain separately authorized YT-020 work; promotion never rebuilds approved bytes |

The final release-approval record references the frozen manifest and signature
bundle digests externally; do not insert that approval back into the already
signed manifest and create a circular or changing approval target.

Propose a contents-read build job with no OIDC permission, followed by a
small attestation job with `contents: read` and `id-token: write`. The latter
consumes only successful same-run build outputs by exact run/attempt/artifact
identity, checks their digests and approved claims, and does not execute code
from downloaded artifacts. It has no contents/packages-write or registry
credentials. This separates workload execution from OIDC access while keeping
one reviewed workflow. [Sigstore's CI guidance](https://docs.sigstore.dev/quickstart/quickstart-ci/)
documents the GitHub OIDC permission and workload identity flow. Actual private
repository service acceptance and exact pinned invocations remain Not verified.

The independent verifier must check exact issuer/SAN/repository/workflow/ref/C,
subject bytes, approved predicate/build type/builder and source/tool/inventory
bindings. Compare the verified DSSE statement with the manifest-bound provenance
bytes; a valid signature alone is insufficient. Require both certificate-chain
and certificate-transparency checks, plus a verified Rekor inclusion proof and
checkpoint, not just an entry ID, inclusion promise or timestamp. A timestamp
authority is not a substitute for the existing log requirement. The
[bundle format](https://docs.sigstore.dev/about/bundle/)
distinguishes these verification materials. Cosign handles cryptography;
ordinary explicit JSON/digest comparisons handle the release's exact policy.
No new validator framework is proposed.

Reuse historical public-fixture rejection evidence where unchanged; future
actual Yetlet qualification must still demonstrate its positive identity and
bounded rejection of wrong identity/ref/source, altered artifact/manifest or
provenance, wrong predicate/builder/root and absent log proof. Never disable
claim, certificate or log checks to obtain PASS. Exact claim comparison is
required even when the upstream signature-verification command succeeds.

## Guarantees, limits and next decision

If later approved and successfully executed, this scheme can authenticate who
signed the exact provenance/manifest, establish their logged existence and bind
the approved source/build recipe and inventory claims to downloaded archive
bytes. Existing independently operated rebuild comparisons remain separate
evidence. It does not prove defect-free code, an uncompromised GitHub account,
runner or dependency, a SLSA Build Level 3 boundary, independent human approval,
production support, actual crates.io bytes/rights or completed public controls.

The accepted nine-source-package scope and known differences stay intact;
exact-candidate Chrome and complete compiled/system inventory remain Not verified.
The original final release verdict stays FAIL / release-ready NO. Formal trust
remains open alongside registry qualification, public activation controls and
concrete maintainer release approval. YT-096 stays In progress; YT-020 stays
Blocked. The next decision is whether to approve this conditional route and
public metadata disclosure, then separately authorize a bounded workflow/policy
implementation and new-candidate proposal. No activation follows automatically.

## Independent research review and checks

Reviewer `/root/yt096_formal_trust_research_review_oct20` returned PASS with
zero issues, required followups or optional notes for this research and the
six related status/index updates. Native trace verification matched the
registered reviewer role, gpt-6-astra model and effective high effort; the
shared resolver observation is enforced. The
[exact result](../artifacts/yt096-formal-trust-research-20261020/REVIEW-RESULT.json)
and [bounded receipt](../artifacts/yt096-formal-trust-research-20261020/REVIEW-ACCEPTANCE.json)
accept the proposal's research scope, not policy adoption or release readiness.
These closing lines record the returned review; they do not replace the
original final candidate verdict.

`cargo xtask check docs` and tracked whitespace checks passed. The new-file
no-index whitespace check returned the expected content-difference exit 1
with no diagnostics; changed-document local links and final newlines were also
checked. [Focused receipts](../artifacts/yt096-formal-trust-research-20261020/CHECKS.json)
retain the exact scopes. All 772 prior YT-096 evidence files, 63 selected source
inputs and nine policy/builder/accepted-scope files retain their hashes.
Only the separately authorized acceptance commit was made. The research
documents remain uncommitted/unpushed; no runtime qualification was repeated,
no resource started and no signing, OIDC request or publication occurred.
