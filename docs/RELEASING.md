# Maintainer release workflow

Use this SOP to prepare a new Yetlet version and promote its verified artifacts.
Development/PR steps live in [DEVELOPMENT.md](DEVELOPMENT.md); actual work and
decisions live in [Backlog](BACKLOG.md) and [Roadmap](ROADMAP.md). This guide is
not authority to execute Git, hosted/key/signing or publication actions.

## Read this before starting another version

The published baseline is the Experimental
[v0.9.0-alpha.1 GitHub pre-release](https://github.com/bohewu/yetlet/releases/tag/v0.9.0-alpha.1):
sixteen source packages, signed provenance and a controlled complete evidence
manifest. [Its actual signing/publication record](YT-096-SIGNER-CONTRACT.md#subsequent-github-pre-release-publication--2026-10-29)
keeps exact historical C/S, hashes and owner decisions. It is not a template of
run IDs to reuse. crates.io remains a separate, unexecuted YT-020 gate.

**YT-209 tooling delivery, required CI and protected result Environment setup
are complete. Positive schema-1 hosted result verification remains Pending.**
The owner-approved [alpha.2 version preflight](ALPHA2-VERSION-PREFLIGHT.md)
updates package identities without selecting a new C or authorizing execution.
Before another release, review its
source/record, applicable version changes and missing qualification, then use the
[versioned interfaces](YT-209-RELEASE-AUTOMATION.md). No version change alone
authorizes a dispatch. Historical tagged workflows retain their original guarded
interfaces; never pass invented new inputs to those old sources. Local tooling
acceptance does not qualify a new candidate or grant key/signing/publication
permission.

## Separate one-time setup from each release

| When | Required work |
|---|---|
| Initial setup, or an explicitly reviewed policy change | Protected branches/check names, trusted workflow permissions, Environment reviewers/ref restrictions, age recipient/identity custody, authenticated tool/root configuration, operator Guard installation and artifact-retention procedure |
| Each version | Version/package/feature/platform scope, affected verification and evidence reconciliation, immutable source/tag, new artifact/run/hash bindings, frozen signing inputs, actual signatures/independent verification, concrete result/publication approval and distribution readback |
| Each protected execution | Recheck source, Environment, identity and input bindings; authenticate the pinned tools/trust material as the implemented workflow requires; honor that execution's approval and time/attempt limit |
| Only when evidence calls for it | Key rotation, environment repair, failed-job disposition, changed trust/disclosure policy or additional platform coverage; none is a routine version-bump prerequisite |

Do not repeat the first release's policy research, host provisioning or completed
unaffected product qualification on every version. Do not reuse old signatures,
source-bound PASS labels or expired artifacts for new bytes. Tool/root configuration
is reusable setup; its current authentication/validity checks are not waived.

## Release identities and freeze order

| Name | Meaning |
|---|---|
| C | Immutable product/build source, selected after the intended version/package inputs and necessary verification are ready |
| S | Separately approved signer workflow/code commit, distinct from C under [ADR0009](adr/0009-release-build-and-signer-identities.md); not an alias for a mutable branch |
| Release dossier | Reviewed scope, source/evidence/approval documents and receipts: version/cohort/C/tag, feature/target limits, S/identity/tool pins, original runs/digests, closure, both raw signing hashes, approvals and destinations |
| Schema-1 release record | Strict data-only input implemented by YT-209: core version/C/cohort, then exact build, custody verifier, S, raw public provenance identity and Release ID; extra fields are rejected |

The [implemented record schema](YT-209-RELEASE-AUTOMATION.md#integration-reference)
is the workflow input, not the complete release dossier. Keep additional scope,
trust pins, later results and approvals in their defined external records/trusted
code. Populate inputs from actual outputs and retain each snapshot; never overwrite
an earlier record or place approval/result receipts inside its frozen assembly.

Prepare the build workflow's version/ref bindings **before selecting C and
creating its immutable tag**. Obtain build/readback IDs and reconcile the
evidence next. Prepare/review the signer input bindings before selecting S.
Freeze the complete assembly before asking for exact signing approval. After
that approval, do not rebuild, reassemble or edit either payload.

The current signer checks require checkout/event/workflow SHA = S and protected
master = S. From selecting S through prepare/sign execution, buffer master
merges; feature branches may continue. Keep result-documentation commits on a
separate branch until signing finishes. After successful signing, later master
commits do not invalidate its immutable certificate/source identity. If master
advances before a pending gated execution, stop for source disposition rather
than silently choosing another S or refreezing outputs.

For a new lightweight tag at an already-remote C, inspect the actual planned
ref update and installed Guard protocol before creation. Its explicit
existing-remote-tag path binds the live accepted branch tip, canonical push
endpoint, C, exact new tag and strict/builtin policy under owner history
acceptance. A read-only plan does not arm it. The normal native pre-push check
consumes an armed binding once; verify the remote tag, preserve the record and
retire a used binding only under the approved metadata authority. Never rearm
a consumed binding, force a tag, treat history acceptance as full-history
cleanliness, or reinstall hooks as routine release work. Tool/platform support
and a passing Guard do not themselves authorize a tag or Push.

## Normal release checklist

| Stage | Action | Required evidence before continuing |
|---|---|---|
| 1. Scope/version | Choose Experimental/stable intent, exact library cohort, semantic version and feature/platform limits; update all affected manifests, internal dependency requirements and independent locks | Reviewed version/package diff and migration notes; examples/CLI do not silently become released libraries |
| 2. Qualify/select C | Complete affected package/security/consumer/target checks and exact-source reconciliation; deliver via the normal PR/CI lane | Exact C and tree, CI and accepted reused/new evidence; explicit source selection, not floating HEAD |
| 3. Tag/build | Authorize the new immutable tag and one bounded build; use strict/builtin Guard and native hooks for Git operations | Live tag points exactly at C; original build run/attempt and actual archive/inventory/provenance identities |
| 4. Readback/retain | Use the separately approved protected verifier; verify input ZIP/ciphertext, complete manifest and public allowlist; retain outside transient CI storage | Actual verifier/owner gate, checksum/receipt correspondence, durable owner-local index and recorded expiry/backup limits |
| 5. Select S/prepare | Review code/source/input bindings, select exact S, run no-OIDC preparation on original inputs; freeze complete evidence and encrypt it | Successful overall preparation including cleanup, exact frozen artifact/receipt, closure and both raw payload SHA-256 values |
| 6. Approve/sign | Present one concrete exact-input packet; owner authorizes one sign attempt and personally approves its Environment | Same frozen bytes verified before OIDC, key removed from signing phase, exact C/S/issuer/SAN/run/certificate/log bindings, retained real bundles/failures |
| 7. Independently verify/accept | Verify both bundles, all source-package subjects and complete private-manifest entries; obtain required independent review and actual-output owner acceptance | Actual bundle/manifest digests, normal chain/CT/log/timestamp checks, plaintext cleanup and scoped final acceptance; same-job PASS alone is not independent verification |
| 8. Publish/read back | Prepare exact public attachments/notes, authorize the selected destination, publish once and verify public delivery | Exact names/sizes/hashes, unchanged tag/C, publication flags and anonymous reader/download correspondence; no private evidence attachment |
| 9. Close out | Retain immutable outputs, approvals, failed attempts and final custody; update item/roadmap and current entry points | Done only at accepted scope; commit result documents only with Git authority |

For the present coordinated cohort, the maintained source is sixteen libraries;
the original nine-package records and old `0.1.0` artifacts remain historical.
The fixed workflow/cohort is not evidence of uniform support across all sixteen.

## Keep owner decisions concrete and batched

Avoid asking for permission for every tool call. Before requesting an action,
finish the already-authorized work needed to make its scope reviewable. A normal
owner decision can cover a bounded sequence, its CI/merge condition, per-stage
attempt/time limits and cleanup. Reuse it throughout that exact sequence.

| Decision | What the owner must be able to review |
|---|---|
| Source/build batch | Version/cohort/targets, actual candidate when selected, Git/ref operations, intended hosted stages and budgets; no automatic retries |
| Exact signing-input approval | C/S, both raw payload hashes, frozen artifact/receipt and identity/tool/disclosure policy; one signing attempt, with actual Environment gate still required |
| Final result and distribution | Actual verified bundle/output hashes, review/cleanup outcome, exact public notes/attachments and destination. Final acceptance and GitHub publication may be approved together when those results already exist |

An earlier approved D1–D5 policy need not be researched/adopted again if unchanged.
Changed identity/disclosure/roots/custody/scope needs review. Approval of preparation
does not permit signing; a green signature job does not permit publication. GitHub
Release publication does not permit crates.io upload. For the latter, resolve
YT-020's explicit registry/credential/order/readback requirements separately.

## Existing workflow map and limits

This is the checked-in implementation baseline, not permission to dispatch it.
[YT-209](YT-209-RELEASE-AUTOMATION.md) defines the versioned input and operator
interfaces; focused local checks and independent implementation review PASS.
Tooling integration and Environment installation are accepted; positive hosted
verification of new-format signing results remains Pending.

| Carrier | Role and timeout |
|---|---|
| [rust.yml](../.github/workflows/rust.yml) | Unprivileged development PR/master CI; MSRV45 minutes, Stable60 minutes |
| [release.yml](../.github/workflows/release.yml) | Manual unsigned sixteen-package preparation at reviewed C/tag, using a versioned core record;45 minutes |
| [evidence-verify.yml](../.github/workflows/evidence-verify.yml) | Original encrypted build-evidence readback, versioned record plus exact original build identity; preflight5 / protected15 minutes |
| [release-sign.yml](../.github/workflows/release-sign.yml) | Closed prepare/sign modes at approved S on protected master; complete record plus exact request, preflight5 / protected30 minutes |
| [release-result-verify.yml](../.github/workflows/release-result-verify.yml) | Separately protected independent signing-result readback at reviewed V, no OIDC/signing; preflight5 / protected30 minutes |
| [release-public.py](../scripts/release-public.py) | Explicit local stage/readback/publish by Release ID and closed twenty-file plan; no automatic trigger or registry support |
| [private-rebuild.yml](../.github/workflows/private-rebuild.yml), [qualification.yml](../.github/workflows/qualification.yml) | Historical rebuilding/qualification scopes; not substitutes for current cohort/source approval |

The source-manifest version and a strict [release record](../scripts/release_record.py)
replace scattered alpha C/tag/run/artifact constants in new workflow execution.
Keep core, build-readback and complete snapshots separately; freeze the complete
record before prepare and preserve its digest through signing and independent
readback. Original tagged workflows and accepted alpha results retain their old
interfaces and identities. Do not dispatch a new interface against an old source.

The [minimum hosted/Environment plan](YT-209-HOSTED-VERIFICATION-PLAN.md) lists
the observed configuration, legacy receipt limits and ready-to-authorize steps.
The `release-result-verify` Environment and existing `AGE_IDENTITY` provisioning
are [installed and read back](YT-209-RELEASE-AUTOMATION.md#owner-authorized-recovery-delivery-and-setup--2026-10-29):
required owner reviewer, master only, no admin bypass. Installation/name readback
does not yet qualify actual decryption; each approved execution must recheck its
settings and original input bindings. It verifies actual original signing bundles
and all frozen entries without signing permission. The
first release's one-time local-key exception remains spent.

Local implementation checks do not qualify a hosted job or authorize a release.
Long-term/off-host retention remains operator-owned. Draft creation, key/tool
rotation, source/tag selection, each hosted phase, owner acceptance and publication
still require their applicable explicit authority. No default framework dependency,
automatic signing, mutable tag or registry publication is introduced.

## Public attachments and publication mechanics

Use the adopted [D1/D2 field/attachment policy](YT-096-D1-D5-RELEASE-TRUST-PLAN.md)
and a closed exact list. The first alpha has twenty public files: sixteen `.crate`
archives, original provenance/public checksum, public DSSE bundle and a reduced
owner-acceptance summary. This is a historical count, not a future directory glob.
Never attach private evidence, full private manifest/body, private bundle or raw
logs/findings merely because they are in the same working directory. Public
identity/log disclosure can occur at signing before Release publication.

Identify a Draft by its returned **Release ID**, not a published-tag lookup.
When updating it, explicitly preserve `tag_name`, `target_commitish` = C, name,
body, draft and pre-release selection. Record notes SHA and every asset's
name/size/digest/ID. Upload each intended file without clobber; inspect partial
state instead of automatically repeating writes. Before publication, verify the
existing tag still resolves to C and the Draft exactly matches the approved list.

Publish with explicit full bindings and the approved pre-release/Latest choice;
do not let an omitted field select a default or create/move a tag. For an
Experimental alpha, retain `prerelease=true`, `make_latest=false`. Read back
published state and asset digests through owner and anonymous access, and verify
an actual public bundle download. GitHub-generated source ZIP/tar downloads are
not signed subjects unless explicitly included in the signed statement.

The public package-set checksum is not the signed complete private manifest.
Retain original signed payloads and immutable issuance receipts; record later
verification/owner acceptance separately. Keep signatures, approval and retention
indexes outside the frozen manifest to avoid hash cycles. Indexes exclude themselves.

## Failure and recovery

- Inspect an uncertain run/ref/upload before any repeat; observation failure is
  not evidence that an action never happened. Never redispatch a run merely because
  its list entry has not propagated yet.
- Preserve a real failed/partial run and any public issuance. Overall cleanup
  failure remains FAIL; do not relax the signing input gate or use `continue-on-error`.
- Correct a demonstrated wrapper/interface error at its smallest seam, retaining
  the original error and completed proof. Continue only authorized, not-started
  steps within the original phase/deadline. Reacquiring data is not permission
  to replay accepted qualification, use a new key scope or reset a budget.
- If the original phase is closed/exhausted, a real check fails or a signature/
  allocation limit is spent, stop for a concrete remaining-scope decision. No
  automatic retry, re-signing, source replacement or artifact resurrection follows.
- Do not rotate the age key just to bump a version. Record recipient/custody and
  retention; rotate under its existing incident/loss/change policy. CI expiry is
  not durable custody, and local custody is not an off-host-backup guarantee.

Consult the dated records only for diagnosis and evidence. Do not copy an old
run ID, local key exception, candidate or successful result into a new release.
