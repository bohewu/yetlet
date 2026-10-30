# Maintainer release workflow

Use this SOP to prepare a new Yetlet version and promote its verified artifacts.
Development/PR steps live in [DEVELOPMENT.md](DEVELOPMENT.md); actual work and
decisions live in [Backlog](BACKLOG.md) and [Roadmap](ROADMAP.md). The current owner-selected automation policy is in
[Release automation and registry promotion](RELEASE-PUBLICATION.md). This guide
is not itself authority for Git, hosted/key/signing or publication actions.

## Current two-package development boundary — YT-211

Current unreleased source is `0.9.0-alpha.5`: exactly `yetlet` followed by
`yetlet-browser`. Schema2/`consolidated` is the closed two-package record;
schema1/`sixteen` remains for earlier artifacts at their matching source.
The library count is derived only from that validated cohort throughout build,
provenance, signing, result verification, public planning and registry promotion.
There is no arbitrary package-list input. The [alpha.5 qualification record](ALPHA5-RELEASE-PLAN.md#actual-qualification-results--yt-212)
records R and actual C selection; S/tag and release execution remain pending.

Normal packaging verifies the final base archive, stages its exact checksum,
then verifies the Browser archive against it. No reverse development edge,
bootstrap crate or `--no-verify` is used. Old sixteen-package records below
remain historical instructions for their own checkouts, not an instruction to
publish removed crates from this tree. The old one-plus-fifteen alpha.4 registry
continuation remains paused and is explicitly invalid for schema2.

The final publication Environment still requires the owner's approval. This
migration does not execute hosted jobs, signing, a Release or crates.io upload.

## Read this before starting another version

For the current two-package source and remaining gates, use the
[alpha.5 plan](ALPHA5-RELEASE-PLAN.md). The
[alpha.4 hosted record](ALPHA4-HOSTED-RECORD.md) is the latest recorded GitHub
pre-release, with one of sixteen packages published to crates.io and its old
continuation paused. It is not a two-package candidate. The alpha.3 account below
is retained at its original scope.

The historical alpha.3 published baseline is the Experimental
[v0.9.0-alpha.3 GitHub pre-release](https://github.com/bohewu/yetlet/releases/tag/v0.9.0-alpha.3):
sixteen source packages and signed provenance. The
[actual record](ALPHA3-HOSTED-RECORD.md#signing-independent-verification-and-publication--2026-10-30)
binds C `557ab76`, S/V `30baf25`, both verified signatures, all sixteen package
subjects and the private112-entry manifest, owner acceptance and exact20 public
files. It is a prerelease, not Latest. Alpha.1 and failed alpha.2 evidence remain
historical; none of their run IDs or signatures is a new-version template.
At that alpha.3 checkpoint, crates.io was a separate, unexecuted YT-020 gate;
the alpha.4 record above supersedes only that distribution status.

**YT-209 now has positive protected result-verification and public-delivery
evidence.** Its first Python upload failed before creating assets; accepted
recovery used `gh api` for uploads and the same exact-plan/readback/publication
checks. The current maintenance correction makes that successful upload
transport canonical; local checks do not claim another live release.
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

## Baseline release checklist

This is the accepted alpha.3/manual-signing sequence. For the newly selected
combined GitHub/crates.io path, use the [single publication checklist](RELEASE-PUBLICATION.md#each-version).
The candidate, payload, identity and independent-verification requirements below
continue to apply; its per-stage human approvals are historical.


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

For the historical sixteen-package cohort, the maintained source was sixteen libraries;
the original nine-package records and old `0.1.0` artifacts remain historical.
The fixed workflow/cohort is not evidence of uniform support across all sixteen.

## Keep owner decisions concrete and batched

The exact-input manual signing decisions below describe the original policy.
Under the [new policy](RELEASE-PUBLICATION.md), the assistant freezes and verifies
those inputs under standing automation authority; only the final joined
GitHub/crates.io publication request needs the owner at its Environment gate.
Git delivery authority remains separate.


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

## Historical deployed human gates — alpha.3

The owner subsequently selected **one final publication gate**, with automated
build/readback/prepare/sign/independent verification. The local implementation and
one-time rollout are in [the current policy](RELEASE-PUBLICATION.md). Until that
code and Environment change are delivered, the following deployed baseline still
applies; local implementation does not silently remove GitHub protections.


These are the existing GitHub settings, not a new approval policy. The alpha.3
execution had **four** protected-job approval occurrences:

| Phase | GitHub Environment | Reason for the gate |
|---|---|---|
| Original evidence readback | `release-evidence-verify` | Formal age identity decrypts original build evidence |
| Frozen-input preparation | `release-sign` | Formal identity verifies evidence and freezes the complete assembly; no OIDC |
| Exact-payload signing | `release-sign` | Issues public signing identity for approved hashes; the decryption identity is removed first |
| Independent result readback | `release-result-verify` | Separate no-OIDC job verifies both bundles and every frozen entry using isolated decryption |

Ordinary development PR CI and unsigned builds have no Environment approval.
Synthetic/disposable-key checks belong before real-key execution; do not use a
protected production-key job as the first transport test. This does not authorize
a parallel validation framework or replay of unchanged qualification.

Git operations, source selection and execution budgets can be covered by a
concrete bounded owner batch. Exact signing needs two actual frozen hashes;
final publication acceptance needs verified outputs, notes and the public list.
Routine preparation, copies and uploads inside that grant need no additional
conversational permission. Preserve failures and follow the granted recovery
limits. Publication itself has no Environment gate: exact-plan owner acceptance
can authorize staging, readback and publication together.

Reducing or combining real-key gates changes Environment/custody policy and
needs a separate explicit decision after the technical path is stable. This
maintenance work does not remove reviewers or enable bypass.

## Existing workflow map and limits

This is the checked-in implementation baseline, not permission to dispatch it.
[YT-209](YT-209-RELEASE-AUTOMATION.md) defines the versioned input and operator
interfaces; focused local checks and independent implementation review PASS.
Tooling integration, Environment installation and alpha.3 positive protected
verification are recorded in the actual execution record. Historical Pending
statements retain their original checkpoints.

| Carrier | Role and timeout |
|---|---|
| [rust.yml](../.github/workflows/rust.yml) | Unprivileged development PR/master CI; MSRV45 minutes, Stable60 minutes |
| [release.yml](../.github/workflows/release.yml) | Manual unsigned closed-cohort preparation at reviewed C/tag, using a versioned core record;45 minutes |
| [evidence-verify.yml](../.github/workflows/evidence-verify.yml) | Original encrypted build-evidence readback, versioned record plus exact original build identity; preflight5 / protected15 minutes |
| [release-sign.yml](../.github/workflows/release-sign.yml) | Closed prepare/sign modes at approved S on protected master; complete record plus exact request, preflight5 / protected30 minutes |
| [release-result-verify.yml](../.github/workflows/release-result-verify.yml) | Separately protected independent signing-result readback at reviewed V, no OIDC/signing; preflight5 / protected30 minutes |
| [release-publish.yml](../.github/workflows/release-publish.yml) | New combined path: no-secret preflight10 minutes, owner-gated publication60 minutes, GitHub first then exact registry bytes and fresh consumer; rollout Pending |
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
Long-term/off-host retention remains operator-owned. Under the historical deployed policy, Draft creation, key/tool
rotation, source/tag selection, each hosted phase, owner acceptance and publication
still require their applicable explicit authority. No default framework dependency,
automatic signing, mutable tag or registry publication is introduced.

## Public attachments and publication mechanics

The local operator tool requires Python and authenticated GitHub CLI `gh`;
`GH_TOKEN` comes from the existing credential environment, never a command
argument or tracked file. Asset POSTs use only `gh api` with a JSON response and
raw binary stdin (`--input -`). Python keeps the fixed record, closed20-file
allowlist, exact size/SHA checks and authenticated/anonymous full readbacks.
Metadata reads and publication PATCH retain their bounded HTTP path. Do not use
a directory glob, clobber assets or skip downloaded-byte checks.

Upload failure exits once, retains a FAIL receipt and reports only operation,
bounded error class and HTTP status when available. Timeout can leave partial
remote state: inspect it before an authorized continuation. Raw CLI stderr,
tokens, provider bodies and signed URLs are never reported as diagnostics.

The combined publisher retains these bounded details with a fixed phase such as
`github-stage` or `registry-publish` in `PUBLICATION-RESULT.json`. HTTP query
failures preserve their numeric status without the provider body. Raw binary
stdin uploads explicitly set `Content-Length` to the verified byte length;
stdin alone does not establish that header. See the retained
[alpha.4 failure and local correction](ALPHA4-HOSTED-RECORD.md#owner-approved-publication-failure-and-local-repair).

If GitHub publication succeeded but registry delivery is partial, preserve both
the public release and every existing immutable crate version. The explicit
[registry-only continuation](RELEASE-PUBLICATION.md#registry-only-continuation-contract--2026-10-30)
binds the original failed result and unchanged archive plan. Reconcile published
bytes/owner and remaining-version absence before a separately authorized attempt;
do not rerun GitHub publication, resend existing versions or re-sign. The known
one-plus-fifteen path still needs a reviewed new P, green CI, adequate first-name
capacity and its actual owner gate. For the owner-selected conservative path,
the registry-only job waits11minutes before each unique PUT, including the first:
165minutes scheduled waiting for fifteen names, within a210minute job bound.
After every wait it rechecks P/CI, owner approval and unchanged Release/assets,
then checks target-version absence. Keep master=P until completion. Unknown
account limits can still reject a write; HTTP429 stops without retry. The normal
publication job retains its60minute bound. It is not an automatic/general resume.

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

## Publishing to crates.io is a separate operation

GitHub Release attachments are not a registry upload. YT-020 remains Blocked;
this section is an operator reference, not execution authority. A real registry
batch must select the current sixteen-package cohort/version and immutable source,
confirm each name/owner right, derive dependency order, record credentials and
limits, and verify actual registry bytes/resolution. The old nine-package YT-020
records remain historical; do not reuse their scope as a sixteen-package grant.

The [2026-10-30 sixteen-package inventory](YT-020-REGISTRY-READINESS.md) records
not-found name observations, the public account and unverified publishing rights.
Use its normal/build layers together with the Config/Secrets versioned
development-dependency caveat. Five independent workspaces prevent selecting
all sixteen with a root-only `--workspace`; the first-publication verification
strategy remains to be proven. This inventory does not authorize an upload.

The subsequent [Config/Secrets contract and dry-runs](YT-020-FIRST-PUBLISH-CONTRACT.md)
verify the pair normally with accepted local package sources and unchanged
manifests. Actual registry availability/acceptance is not simulated. Their
generated archives differ from the signed set in internal packaged-lock
checksums; resolve the sixteen-package artifact/checksum contract before
uploading or claiming existing signatures cover a repackaged registry set.

The [formal registry contract](YT-020-REGISTRY-PUBLICATION-CONTRACT.md) now selects
the first-name dependency correction and exact-byte promotion of sixteen newly
prepared artifacts. It requires a separately selected immutable R/V, artifact
freeze and reviewed transport. Current alpha.3 archives have a Config/Secrets
development-name cycle and are not approved initial registry inputs. The
isolated prototype and offline envelopes do not activate a publisher.

If registry credentials are not configured, sign in to crates.io with GitHub, verify the account
email and create a token permitted to publish the selected names. Enter it with
`cargo login`; reuse an existing installation rather than creating another token.
Keep token values out of repo files, shell arguments and reports. The
[Cargo publishing guide](https://doc.rust-lang.org/cargo/reference/publishing.html)
documents account setup, names, metadata and ownership.

Cargo source-preflight illustration for the dependency-free foundation crate,
after clean-source selection; this is not frozen-artifact promotion:

```sh
cargo publish --manifest-path crates/yetlet-core/Cargo.toml --registry crates-io --locked --dry-run
```

The [publish command](https://doc.rust-lang.org/cargo/commands/cargo-publish.html)
packages and verifies before upload; `--dry-run` performs checks without an
upload. Do not use a verification bypass. First-publication ordering includes
versioned development dependency names as well as normal/build/optional/target
declarations. Wait for each prerequisite's actual index entry. A downstream registry
dry-run may remain blocked while its selected sibling versions are unpublished;
local directory-source/package proof is not registry proof.

`cargo publish` packages source. The selected frozen-artifact method uses the
official upload protocol after normal verification and separate transport review;
it never regenerates or patches a frozen archive during transmission. Inspect
prepared archives and compare intended payload/digests,
then download every accepted name/version and test a clean consumer resolving
from crates.io, without local path patches. Do not claim the GitHub signatures
attest different registry bytes. An index-wait timeout does not prove upload
failure: inspect the registry before another attempt. A published version cannot
be overwritten; yanking is distinct from replacing its contents.

For subsequent CI publishing, prefer configured crates.io Trusted Publishing
with short-lived OIDC credentials rather than a long-lived registry Secret.
The [official introduction](https://blog.rust-lang.org/2025/07/11/crates-io-development-update-2025-07/)
and [current enhancements](https://blog.rust-lang.org/2026/01/21/crates-io-development-update/)
describe that model. Registry publisher configuration/credentials are separate
from GitHub Release rights and Sigstore identity. No crates.io token, publisher
workflow, registry Environment, name claim or upload is installed by this work.
