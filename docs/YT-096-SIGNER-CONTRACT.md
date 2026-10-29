# YT-096 — Separate signer and frozen payload contract

**Historical real-input preparation blocker (2026-10-28):** Owner-authorized PR15 is squash
merged as selected signer S `d2562e14af75d08c177c01e976cbd8f13124b8ff`, with the
reviewed tree unchanged. The protected `release-sign` Environment and existing
`AGE_IDENTITY` are installed at the approved scope. Before any hosted prepare,
static comparison against the actual original builder/custody layout reveals
duplicate OpenSSL log/result basenames in private and extras. The initial
synthetic fixture had no extras and missed this reachable failure. No hosted
attempt or official identity is used. A minimal label-preservation correction
and matching real-layout regression are now local, with independent followup
review PASS and source delivery Pending. S is immutable; no corrected code is executed as if it
were the selected S, and replacement source selection requires explicit authority.

**Historical alpha preparation checkpoint — 2026-10-28:** The owner accepts
preparing the two signing inputs and a separate signer workflow. Original alpha
build `40669449040`/attempt1 and manually approved verifier `40688772221`/attempt1
PASS at the [recorded custody scope](YT-096-ALPHA-HOSTED-PLAN.md#actual-alpha-protected-verifier-and-retention--pass).
The execution source is `bf3363801132f4481263ceb411d9bba3349df86d`, immutable unsigned
`v0.9.0-alpha.1`, with sixteen Experimental `0.9.0-alpha.1` libraries. The earlier
`5d7b918` was version-preflight C, not this hosted execution source. Historical
`v0.1.0`, source `3eacfe3`, build/readback and retained payloads remain unchanged.

Date: 2026-10-29 (Asia/Taipei). **Current state: PR17 delivered; additional
preflight identifies Draft API HTTP403; local encrypted handoff implemented.**
Selected S is `0e8fb600af7aaebd4641a78c477035afe637c67e`, with PR/master CI PASS.
Prepare `40789535074/1` fails before protected work. The new uncommitted handoff
code is not S and has not run hosted. Real ciphertext transfer and nine focused
local cases and independent Reviewer PASS0 accept the bounded implementation. See the
[current handoff contract](#encrypted-ciphertext-capability-handoff--2026-10-29).
D6 Pending; YT-096 In progress; YT-020 Blocked. All dated earlier checkpoints
keep their original source, scope and acceptance results.

This contract and [ADR 0009](adr/0009-release-build-and-signer-identities.md)
preserve the [D1–D5 policy](YT-096-D1-D5-RELEASE-TRUST-PLAN.md), including D2
disclosure and custody limits. Local preparation does not authorize hosted jobs,
new Environment/Secret placement or private-key access, OIDC, signing,
publication, Commit or Push. The protected Environment was subsequently configured
under separate owner authority; this local diagnostic continuation changes no settings.

## Immutable build and separate signer identities

| Binding | Required value |
|---|---|
| Build source C | `bf3363801132f4481263ceb411d9bba3349df86d` |
| Build tree | `c4ee13aa558489ba3962874dbf5e0a0a1bc99605` |
| Product tag | Existing immutable unsigned `v0.9.0-alpha.1`; never move it |
| Build workflow/run | `.github/workflows/release.yml@refs/tags/v0.9.0-alpha.1`; `40669449040` / attempt1 |
| Protected verifier | `40688772221` / attempt1 |
| Selected signer source S | `0e8fb600af7aaebd4641a78c477035afe637c67e`, after PR17 squash Merge; uncommitted handoff code is not selected |
| Proposed signer ref | Protected `refs/heads/master` |
| Signer execution equality | Checked-out code SHA = event SHA = workflow SHA = `expected_signer_commit` = S |
| Exact certificate SAN | `https://github.com/bohewu/yetlet/.github/workflows/release-sign.yml@refs/heads/master` |
| OIDC issuer | `https://token.actions.githubusercontent.com` |
| Repository/owner | `bohewu/yetlet`, repository ID1409587171; `bohewu`, owner ID35250002 |

S is not C. It is the actual owner-authorized PR17 squash commit, with the
reviewed tree unchanged. A later local edit never implicitly replaces S. Verify the certificate's authenticated workflow/build-signer digest
cryptographically against S, alongside the exact issuer/SAN and repository/owner
identities. SAN or branch name alone is insufficient. A mismatch fails closed.
Check the actual default OIDC subject for the proposed Environment separately;
`sub` is not the certificate SAN. Do not assume the original tag subject applies.

Product provenance continues to identify C, the original build workflow/tag/run
and sixteen actual archives. Signing at S never rewrites that history or claims
S built the products. Keep original archive and provenance bytes unchanged.

## Proposed execution and custody boundary

Propose protected Environment `release-sign`: exact `master` **branch** policy,
reviewer `bohewu`, prevent self-review `false` for the single-owner arrangement,
administrator bypass `false`. These adopted values were installed and read back
under separate owner setup authority; the original proposal alone granted none.
Existing `release-evidence-verify` tag policy and Secret stay unchanged.
Installing a Secret or accessing `AGE_IDENTITY` in the new Environment needs
explicit future custody/setup authority; this amendment does not broaden D2.

The proposed manual workflow has only two closed modes:

| Mode | Permission and approval boundary |
|---|---|
| `prepare` | Separately authorized later; protected job has `contents: read`, `actions: read`, no OIDC permission, issuance or signatures |
| `sign` | Requires D6 approval of exact S, identities and both payload SHA-256 values, plus actual Environment approval; protected signing job alone adds `id-token: write` |

Read-only preflight precedes protected work. Neither mode receives contents or
packages write permission. Privileged jobs execute only reviewed trusted helpers
at S: no PR/untrusted code, unreviewed remote scripts or product code execution. Authenticate
age and Cosign3.1.4 and pin valid approved trust roots/configuration before use;
bootstrap and supplied-statement compatibility remain Pending. Never use floating
tools, artifact-supplied roots or disabled certificate/log checks.

The age private identity is available only in an isolated, manually approved
verification/decryption phase, using per-step environment scope. Remove identity
files and unset key variables before any OIDC signing step; clean up on success
and failure. No private key enters builds or logs; no private plaintext is uploaded.
Environment approval does not substitute for D6 or secret-placement authority.

## Two raw payloads

| Target | Exact bytes and signature contract |
|---|---|
| Public provenance | Original alpha build `public/PROVENANCE-UNSIGNED.json`, 9158 bytes, SHA-256 `4d4de1a8ce9cc077c0aa62a1aafb3ab9799eed312deddb66d9e6f8fc6fef2d68`; supplied-statement DSSE whose decoded payload is byte-identical |
| Private complete evidence manifest | Final frozen assembly's `RELEASE-CHECKSUMS.json`; SHA-256 Pending; detached blob signature over its actual bytes |

The public payload is copied byte-for-byte from the verified repository-external
alpha custody into ignored preparation storage, flushed and reread with that
size/hash. This fixes the proposed public signing bytes only: it creates no
signature, D6 approval, final private assembly or preparation receipt. The old
`0.1.0` provenance remains 9022 bytes, SHA-256
`b3a1e66a34d316c6513ff4e0b9c8055c37b8f81daa598a8110f867e911a400fc`,
with its original `3eacfe3`/build `40628943583` identity; never sign or relabel it
as the alpha provenance.

The sixteen subjects cover the sixteen actual archives; this does not require
sixteen individual signatures. Compatibility must prove preservation of the
supplied statement before issuance authority; add no fields or new source claims.

The existing private preparation manifest inside age covers its build assembly
only. Safe transport/readback receipts omit its SHA. Obtain that original SHA
only during later authorized isolated decryption: compare authenticated
`inventory.private_manifest_sha256` with the actual restored manifest file hash,
then export the digest, never plaintext. This is a historical input identity;
the final manifest SHA can differ. Public `RELEASE-CHECKSUMS.json`, retention-index,
inventory SHA and ciphertext SHA cannot substitute for either private manifest.

## D6 completion and the GitHub draft boundary

D6 is concrete authority for identity issuance/signing and later promotion; it
is not satisfied by a green unsigned build or an evidence-decryption receipt.
Keep the following conditions separate:

| Gate | Actual alpha status / required outcome |
|---|---|
| Source, unsigned build, verifier and retained bytes | PASS at the existing bounded scope; reuse without replay |
| Public signing bytes | Fixed above; no signature or owner signing approval |
| Complete private evidence assembly | Pending reviewed exact closure, protected assembly/verification, encryption and actual final manifest SHA |
| Trusted signer | Local implementation/review and tool compatibility retained; fixed S selected; hosted preflight failure remains unresolved |
| Protected preparation | Owner-authorized setup complete; `40717870186/1` preflight FAIL, protected job skipped; no OIDC permission |
| Exact D6 signing approval | Pending owner acceptance of S, identities, both raw payload hashes and the frozen preparation receipt/artifact binding |
| Actual signatures and verification | Pending protected signing, independently verified bundles/certificate/log/S, all subjects and all private-manifest entries; retain failures without automatic retry |
| Final disposition | Record the owner's acceptance against actual manifest/bundle digests; do not mark YT-096 Done from draft creation or technical preparation alone |

After those formal integrity conditions pass and the owner accepts the actual
result, the signing work can close at that recorded scope. GitHub Release
publication and crates.io upload still need their own explicit approvals; the
separately blocked YT-020 registry gate does not disappear. This adds no business
SLO, hardware or general operations requirement.

The owner also asks to create a Release. A single **unpublished Draft
pre-release** is created using existing `v0.9.0-alpha.1`, source C, release ID
`426670086`: [alpha Release draft](https://github.com/bohewu/yetlet/releases/tag/untagged-cbcde6fe4a28746e82cc).
Fresh readback confirms `draft=true`, `prerelease=true`, `published_at=null`,
no uploaded assets and unchanged release-note bytes. The notes introduce native
SSR, opt-in capabilities, source usage, library scope and documented limits;
they explicitly disclose pending signing/registry publication. No new tag,
GitHub Release publication, certificate, signature or package upload is created.
The draft is a review surface, not evidence that D6 has passed; publication must
reconcile its pending status wording against the actual verified outcome.

The minimal workflow preparation remains the two closed modes above, with a
read-only preflight before any protected job. `prepare` validates the exact
alpha artifact/receipt/source inputs and selected evidence closure, emits only
encrypted frozen input plus safe `SIGNING-INPUTS`, and has no `id-token: write`.
`sign` later consumes those same frozen bytes under exact D6 approval and adds
OIDC permission only to the protected signing job. Both use reviewed code at S,
not product code execution or a new product build. That earlier checkpoint prepared the input/authority plan only. The subsequent
local implementation below does not select S or grant execution authority.

Ignored `target/yt096-alpha-signer-preparation-20261028/` retains public payload
bytes/readiness, exact Release request/response/readback, release notes and
source/tag/disclosure checks. It contains no private key or decrypted evidence.

## Final assembly scope and freeze order

Before assembling, review an exact allowlist mapping every required original
input identity to a unique flat logical name, size, digest and evidence scope.
Resolve required members and their digest relationships before freeze. Include:

- All sixteen actual C archives and the original provenance bytes.
- Required source export, six selected locks, recipe/tool identities, retained
  OpenSSL source archive, recorded executable digest and original build records.
  Do not claim an unretained tool binary is present merely because its digest exists.
- Explicitly selected reused qualification, security, inventory, support and
  review evidence, retaining original source/run identities, limits and failures.
- A closure map recording that selection and its scope; it cannot include the
  future containing manifest hash or later signing outputs.

Do not upload a whole repository/evidence tree or include secrets. Preserve the
old preparation manifest bytes under a distinct flat label if required by the
closure map; never overwrite the original frozen assembly. Apply existing flat
checksum limits: 128 artifacts, 64 MiB/file, 256 MiB total, 64 KiB manifest. If
complete required closure does not fit, stop for owner disposition; do not drop
proof, raise bounds or archive unverified content to evade them.

Future authorized preparation must follow this order:

1. Verify original artifact ZIP digests/IDs, build source/run/attempt, owner
   approval and transport receipt; decrypt with the existing bounded helper,
   verify its authenticated inventory and original private manifest.
2. Materialize the reviewed allowlist once in a new isolated directory outside
   Git. Retain all original evidence scopes; do not rebuild or relabel products.
3. Use the existing `cargo xtask release checksums create <assembly>` and
   `cargo xtask release checksums verify <assembly>` CLI on that complete flat
   assembly. These are proposed future operations, not commands executed here.
   Hash the actual final manifest bytes and freeze the verified assembly.
4. Encrypt it with the adopted recipient. Emit only encrypted assembly and a safe
   `SIGNING-INPUTS` receipt, outside the assembly, for later owner review.
5. Obtain separate owner approval of the exact two payload SHA-256 values, S,
   signer identities and receipt/freeze binding. Preparation grants no D6 authority.

The immutable preparation receipt binds C/S, both payload SHA-256 values,
actual input artifact IDs and ZIP digests, original build/verifier and preparation
run/attempts, source/signer identities, authenticated tool/root/config identities
and the closure-map/frozen-assembly binding. Its own digest and artifact identity
become part of the later owner-approved signing input. Emit it after freeze,
outside the assembly, without inventing IDs, hashes, certificates or bundle IDs.

Record the later actual signing invocation in a separate execution receipt bound
to that approved preparation receipt; never rewrite the preparation receipt to
fill a future run ID after approval. The actual signing run/attempt is checked
before issuance and is visible for the human Environment approval. A pending S
or either payload SHA is never a valid signing input.

Authorized `sign` retrieves the **same** frozen encrypted assembly, verifies
approved receipt/bindings and both raw payload hashes before requesting OIDC,
then signs those bytes. It never reassembles, relabels, rebuilds or edits payloads
after approval. Retain both future Sigstore bundles and final owner approval
outside the manifest; final approval references actual manifest/bundle digests.

## Verification, disclosure and remaining authority

Independent verification checks exact raw payloads, sixteen archive subjects,
complete private manifest entries, certificate chain/CT, approved issuer/SAN/S,
repository/owner identities and approved log inclusion/checkpoint and timestamp
material. A timestamp alone is insufficient. Preserve original build identity C
separately from authenticated signer identity S.

Full public provenance and certificate/log metadata remain public. Private
manifest body stays controlled; its digest, signature and certificate may be
public through signing services. Never expose raw tokens, keys, private findings
or plaintext evidence. Existing D1/D2 disclosure classes are unchanged.

Preserve CE56's original FAIL/release-ready NO, thirteen historical path findings,
old guard/design-reviewer/wrapper/rotation failures and later bounded dispositions.
Accepted unsigned custody PASS establishes its recorded scope, not release
authorization. No earlier review PASS applies to this contract automatically.

Next unblockable work is acceptance of the local implementation review, followed
by separately authorized delivery/source-S selection and protected custody setup.
Payload preparation, exact D6 issuance/signing and publication each need their own
authority. Bounded new tool/statement compatibility evidence is recorded below;
actual hosted execution and certificate/service acceptance remain unverified.

## Local workflow implementation — 2026-10-28

The owner authorizes strict/builtin commit of the five existing result documents,
then the minimal local signer workflow and new verification with independent
Reviewer acceptance. Commit `765757d833ae432d9f72050e176b8af485e9fe66`, tree
`d9a3f9448c44a76811485cdbb913e4e25024567f`, contains only those five documents;
explicit strict/builtin and the normal hook PASS, no Push. The new integration
remains uncommitted. Independent implementation review is **PASS**, zero issues and required followups.

### Entry points and permissions

- `.github/workflows/release-sign.yml` is manual only. A single closed JSON
  `request` selects `prepare` or `sign`; unknown fields/modes and implicit S are
  rejected. Read-only preflight checks exact event/workflow/checkout S, protected
  current `master`, repository/owner IDs and the existing `release-sign` rules.
- `prepare` has only contents/actions read, followed by real Environment approval.
  No OIDC permission is present. It verifies the fixed original build/verifier,
  actual historical approval, original artifact ZIP and reviewed supplemental
  input, then uses existing custody/xtask checksum code to create a new flat
  assembly without rebuilding product code.
- `sign` alone has `id-token: write`. Its explicit request binds S, preparation
  run/attempt/artifact ZIP, immutable `SIGNING-INPUTS` receipt and both raw payload
  hashes, with `d6=approve-exact-signing-inputs`. A real Environment gate is still
  required. That input text alone cannot stand in for recorded owner D6 authority.
- The age identity is step-scoped, removed before signing, and never passed to
  tools/bootstrap. The signer rechecks the frozen bytes and API rules before
  requesting GitHub OIDC; TLS-origin JWT claim preflight checks the exact subject,
  S and invocation before Fulcio issuance. Fulcio validates the token signature;
  Cosign validates certificate/SCT/log/timestamp material. Additional authenticated
  certificate extensions bind signer S separately from product build C.
- Supplied-statement DSSE signs the original alpha provenance bytes. Detached blob
  signing covers the complete frozen private manifest; neither containing manifest
  nor original preparation receipt is rewritten. All sixteen subjects/entries are
  checked. Same-job verification is not the required later independent verification.
- Upload paths are explicit: encrypted assembly plus safe preparation receipt, or
  public provenance bundle, encrypted private bundle and safe execution receipt.
  Failures seal available unchecked partial bundles, record FAIL without a retry,
  and do not upload plaintext. Cleanup runs after success/failure/cancellation;
  runner loss cannot be promised recoverable by a cleanup step.

Preparation jobs are bounded to thirty minutes including authenticated tool
bootstrap, with a five-minute read-only preflight. Neither mode can write
repository contents/packages or publish a GitHub Release. No new settings,
Environment, Secret, remote asset or hosted execution is created by this slice.

### Explicit supplemental input transport

Required reused evidence is not automatically copied from the checkout or a
whole local tree. Before any hosted prepare, select and review one **flat**
qualification directory outside Git. It contains the chosen immutable records,
`QUALIFICATION-CLOSURE.json` and a normally created/verified checksum manifest.
The closure lists unique logical names, actual sizes/hashes, original source,
recorded limits and roles for qualification/security/inventory/support/review and
preserved failures. Those fields preserve scopes; they do not automatically prove
that the chosen records satisfy every qualification requirement. Review the exact
selection separately. Existing 128-file/64-MiB-file/256-MiB-total/64-KiB-manifest
limits stay unchanged, with failure on collisions or an incomplete selection.

`release-sign.py pack <absolute-flat-input> <absolute-new-output>` encrypts that
verified selection with the pinned recipient using an authenticated age binary
supplied through `SIGN_AGE`. It produces `qualification-inputs.zip` with only age
ciphertext and a safe receipt; temporary plaintext TAR is removed on failure too.
There is no upload command or recursive discovery in this helper.

A later separately authorized upload can attach **only** that ciphertext ZIP to
existing Draft release ID `426670086`, without publishing it or replacing source
C/tag. Prepare checks the exact asset ID, API digest, ZIP bytes, recipient and
qualification-manifest hash supplied in its reviewed request. It decrypts only
inside the isolated job. No such real selection, packing, asset upload or secret
access occurred here. The draft is still unpublished and has no uploaded assets.

The prepare request uses `mode`, `signer_commit`, `qualification_asset`,
`qualification_sha256` (raw ZIP) and `qualification_manifest_sha256`. The sign
request uses `mode`, `signer_commit`, `run`, `attempt`, `artifact`, `digest`
(raw ZIP), `receipt_sha256`, `provenance_sha256`, `manifest_sha256` and exact `d6`
value above. Numeric IDs are positive decimal strings; SHAs are full lowercase
hex. Actual S, supplemental asset and frozen-manifest values remain Pending.

### New bounded evidence and remaining gates

Local new verification authenticates the pinned Cosign3.1.4 binary before first
execution: production TUF refresh from pinned Sigstore root10, authenticated
artifact key/root/signing configuration, OpenSSL KMS-signature verification,
then exact upstream Google keyless certificate verification. The adopted D4
binary/root/config hashes are unchanged. Bootstrap dependencies are isolated
hash-pinned Python tools, not Yetlet runtime dependencies. Existing accepted age
bootstrap is reused without a new local replay.

A LOCAL synthetic ephemeral-key Cosign3.1.4 supplied-statement case passes and
preserves every original statement byte. It uses empty local service endpoints,
no ambient OIDC, no Yetlet payload/identity and no public log submission; fixture
keys are removed with the temporary directory. New focused Python tests exercise
closed requests, source/Environment/OIDC checks, bounded hostile ZIP/TAR input,
retained qualification scopes/failures, freeze/decrypt handoff, identity removal,
partial failure sealing and permissions. New Python/Bash/YAML syntax and
changed-document checks are separate from product/platform qualification.

Preserve the initial missing TUF `bootstrap` argument failure, its one canonical
API correction/continuation, and the misplaced synthetic test-block failure with
focused correction. Evidence resides in ignored
`target/yt096-alpha-signer-preparation-20261028/implementation/`. No old passing
product/platform/security scan, build or hosted verification is replayed.

This proves local integration only. Pending: source-S delivery/selection; reviewed supplemental closure and encrypted upload;
new Environment/Secret setup; actual protected preparation/final manifest freeze;
exact D6 owner approval; real signing and independent downloaded-output/certificate
verification; final owner disposition. Publication remains separate. D6 Pending,
YT-096 In progress and YT-020 Blocked; no official identity/signature is issued.

### Independent local implementation acceptance

Reviewer `/root/yt096_signer_workflow_independent_review_20261028` returns
**PASS**, zero issues and required followups. All twelve frozen target hashes,
HEAD `765757d` and empty staged index match before/after review. The review
inspects the actual targets and necessary custody/checksum dependencies; it
performs no mutation, live operation or replay of passing verification.

The dispatch profile is healthy/current/eligible/trusted; the shared resolver
requests `high` for configured reviewer `gpt-6-astra`. The installed native trace
helper cannot locate the child/parent trace, so runtime role/model/effective
effort remain **unverified**, not enforced or formal-assurance evidence. The
non-strict ordinary independent content review is retained with that limitation.
The requested routing/result and missing-trace receipts are kept in the ignored
implementation evidence directory; no runtime identity is inferred from the
reviewer's own output. Subsequent document edits record acceptance only.

Thirteen unique new focused cases have passing observations across the retained
iterations; corrected/new paths are checked without replaying accepted product
validation. This acceptance does not select S or qualify hosted service behavior.
No Environment/Secret/asset change, hosted execution, official signing, new-change
Commit/Push or publication follows. D6 remains Pending.

## Pre-hosted real-layout correction — 2026-10-28

The owner approves one batch: exact PR15 squash Merge/source selection, protected
Environment/Secret setup, independently reviewed encrypted evidence selection,
one prepare run and repository-external retention, without signing/publication.
The merged S `d2562e1` has tree `74dd75a7f132f9080885190e1cecdc99245d57af`, identical
to reviewed PR head `146a449`; original build C `bf33638`, tags and both source
branches remain unchanged. The new Environment has ID `25580949441`, required
reviewer `bohewu`, administrator bypass disabled, self-review permitted under the
existing single-owner policy and sole `master` branch policy ID `64671016`.
`AGE_IDENTITY` enrollment confirms the pinned recipient and unchanged owner key;
no key is written into Git/logs, rotated or copied to a repository Secret. Old
verifier Environment, policy and Secret metadata remain byte-identical.

Before dispatch, `preparation::record_inputs` is observed to copy
`openssl-build.log`/`openssl-build-result.json` into the private build assembly;
`custody::capture` also retains both as extras alongside OpenSSL source. The new
prepare helper previously attempted the same exclusive destination names for
both sets and would always raise `FileExistsError` on successful actual inputs.
The original implementation Reviewer PASS keeps its historical scope; it did
not qualify real hosted execution, and this missed case is recorded explicitly.

The minimum correction maps original extras to `BUILD-EXTRA-<original-name>` and
records that exact map in `EVIDENCE-CLOSURE.json`. Original private files and
preparation manifest stay intact; every extra byte remains retained under a
unique reviewed logical label. There is no overwrite, discarded proof, bound
increase, source rebuild, provenance edit, changed public payload, key change or
OIDC permission change. Extending the existing synthetic assembly case with both
real duplicate names reproduces the old failure; that focused case then PASSes
with corrected code, preserved duplicate bytes and same frozen-input handoff.
No previously accepted qualification is replayed.

The separately authorized evidence preparation selects 53 existing local records
outside Git: original C source/locks/recipes, alpha packaging/review, accepted
native/browser/GPU/dependency/security/inventory summaries and proofs, actual
alpha build/readback/custody and retained failures. The original verified
370-member security inventory ZIP is retained byte-identically, not repackaged
or used to hide unverified content. Windows denied ZIP transfer is not bypassed:
its original remote evidence is represented only at the admitted result/index/
reference scope. The later Tokio audit ZIP is referenced by accepted immutable
reconciliation, not falsely asserted to be present in this selected local set.
`REUSE-SCOPE.json` records these limits; the full actual C source export retains
the support/qualification documents. Selection is not a new all-source security
scan, complete compiled/native/system inventory or blanket platform claim.

The selection proposal contains exact names/sizes/hashes, original sources and
qualification/security/inventory/support/review/failure roles. All required
original private build records/logs and extras will be independently authenticated
by the existing bounded custody helper during future prepare. Adding the followup
review receipt remains acyclic: that receipt reviews the fixed base selection
and code, without containing the future outer manifest hash or signing outputs.
Final assembly retains the existing 128-artifact/64-MiB-file/256-MiB-total and
64-KiB-manifest limits; this selection leaves count/size margin. No real capsule
is uploaded and no hosted attempt is started at this checkpoint.

Independent followup review is **PASS**, zero issues and required followups. Evidence is kept in ignored
`target/yt096-signer-prepare-batch-20261028/`: owner authority, actual S/tree,
Environment/enrollment readbacks, original blocker, before/after focused case,
exact selection and source-path bindings. Any Git delivery/replacement-S change
is separate from the original exact-PR15 approval. D6 Pending, YT-096 In progress,
YT-020 Blocked; no official certificate/signature or publication occurs.

### Followup acceptance and prepared supplemental bytes

Reviewer `/root/yt096_real_layout_and_evidence_selection_review_20261028`
returns PASS0 for the six frozen source files and exact 53-record proposal,
SHA-256 `91dc6c988e2d3452a1614b157ad1e71bfef1694cd65f6ccfe55cbe464683e999`.
All file/source-copy hashes, sizes and 0700/0600 modes match. Actual C export,
locks/recipes, support document membership and original 370-member security ZIP
are checked; Windows/Tokio references keep their stated limits. The native trace
helper remains unavailable, so configured reviewer model/effort are requested,
unverified runtime metadata, not formal-assurance proof. No test or scan is replayed
by review. Only acceptance/status metadata changes afterward; corrected code
hashes remain identical to the reviewed bytes.

Adding that non-self-referential review receipt yields 54 selected records plus
closure and manifest (56 flat files, 18,267,605 bytes). Normal xtask checksum
create/verify and the existing pack helper succeed. The immutable qualification
manifest SHA-256 is
`20757af3039310399145a93acf9bab06439d88235a55aa7e776aa90280bece3c`;
the ciphertext ZIP SHA-256 is
`c90d98d20d579ed76c9ba8e705a6b204e1e43186eb3e02ec0121e91d15596168`.
These are supplementary input identities, **not** the final complete private
signing-manifest hash. The temporary plaintext TAR is removed; encryption uses
only the approved public recipient. No private identity is accessed for packing.
The original C artifacts and source S `d2562e1` remain unchanged.

The owner adds `+3` allowance without specifying a unit; this is retained as a
budget instruction, not interpreted as signing/publication authority or a new
source/ref authorization. The six-file correction still needs explicit guarded
Git delivery and replacement-S selection before the already authorized prepare
can execute. The original one-shot hosted run has not started.

## Hosted preflight failure and safe local diagnostics — 2026-10-29

The owner authorizes delivery of the six-file real-layout correction and
replacement S. Strict/builtin screening and native Commit/Push hooks pass;
[PR16](https://github.com/bohewu/yetlet/pull/16) head `7acb517` has tree
`977b82db0323e57f8692b8c22172f609a7c8c008`. Its initial
[CI attempt](https://github.com/bohewu/yetlet/actions/runs/40710158856) fails at
Chrome-debugger startup, before browser assertions; diagnostics and owned cleanup
are retained. A separately authorized retry runs only the failed Stable job;
MSRV keeps its original execution timestamps. Attempt 2 passes. Squash Merge
selects S `87d24e9e63b4c9486d63a4b0141c20368ceaf540`, preserving that exact tree;
[automatic master CI](https://github.com/bohewu/yetlet/actions/runs/40716085116)
attempt 1 also passes. All original branches, C, tags and evidence remain intact.

The one authorized [prepare run](https://github.com/bohewu/yetlet/actions/runs/40717870186),
attempt 1 at S/master, fails in its read-only metadata preflight. Both `prepare`
and `sign` are skipped, no pending Environment approval or output artifact
exists, and no private-key step, decryption, OIDC issuance or signature occurs.
This consumes the one dispatch, not a successful freeze. Final private-manifest
SHA and D6 remain Pending. No automatic retry, permission change or replacement
source follows this failure.

The old source emits only a generic rejection, so its exact failing checkpoint
is unknown. Owner-context cached metadata meets the same pure conditions; that
does not prove access by the hosted read-only job token. Draft-carrier visibility
is only a hypothesis. Do not report an observed HTTP status or raise permissions
on that inference. The encrypted supplemental asset `685300857` remains on
unpublished draft `426670086`; removal was conditional on successful frozen
readback/retention, which did not occur. Original ciphertext stays retained.

Failure/operational receipts are copied outside Git and reread at `0700`/`0600`:
23 files, 200,037 indexed payload bytes; retention-index SHA-256
`351cf237ce2010d36ca262f370da50d11ab8d1684f5288b7563aef8f6082d160`.
This is failure custody, not the final signing manifest or a frozen assembly.
Ignored `target/yt096-signer-prepare-batch-20261028/` retains the original
records, including the first Chrome failure and both CI identities.

The owner then asks to continue with the proposed safe diagnostic minimum:

- Existing gate conditions and order stay unchanged. Fixed internal stage names
  distinguish source/branch, Environment/policy, build/artifact, verifier/approval,
  qualification Release/assets and prior prepare run/artifact checks.
- Rejections emit only a stage, `http`, `network`, `metadata` or `internal` class,
  and an integer HTTP error status in 400–599 when available. Never print a raw
  exception, reason, URL, response body/header, token or private path. Close HTTP
  error responses without reading their bodies so finalizer warnings cannot
  accidentally expose reason text. Failures outside the gate stay generic.
- Both success and rejection keep existing exit behavior, and rejection does
  not reach native tools or OIDC. The helper's checks are also reused before
  protected download/signing; their diagnostic classification grants no new rights.
- Three new focused local tests use synthetic API objects only. They cover a
  valid metadata sequence followed by a draft-HTTP rejection, blocked bypass,
  network/internal failures and generic CLI redaction. The original missing-entry
  failure and initial HTTP-resource warning are retained; the affected HTTP case
  passes after explicit response closure. No accepted product test is replayed.

Evidence is under `safe-preflight-diagnostics-20261029/` within the same ignored
batch directory. Local focused cases, Python syntax and whitespace checks pass;
`cargo xtask check docs` passes (75 documents, 207 backlog items). Independent
review PASS0 accepts the bounded local change. No hosted diagnostic execution
has occurred, so the original failure cause is still unknown. Diagnostic changes
remain uncommitted/unpushed; changing S or executing another hosted attempt needs
separate owner authority. D6 Pending; YT-096 In progress; YT-020 Blocked.

### Safe diagnostic independent acceptance

Reviewer `/root/yt096_safe_preflight_diagnostics_review_20261029` returns PASS,
zero issues and required followups, after inspecting the six-file diff, relevant
caller/helper/workflow/ADR, all six frozen hashes and retained focused results.
The Reviewer reruns no tests and performs no network, key, hosted or Git action.
This dispatch has matching native role/model/effort evidence: `reviewer`,
`gpt-6-astra`, effective `high`. It is ordinary content review, not formal
assurance or a diagnosis of the hosted failure. Earlier unavailable-trace
limitations and accepted historical scopes are preserved.

Only completion metadata follows review; the two reviewed code hashes are
unchanged. The source remains selected S `87d24e9`, with six uncommitted paths
and nothing staged. Next is owner-authorized strict/builtin Git delivery and,
separately, selection of any replacement S/extra hosted attempt; no such action
is implied by this PASS. D6 remains Pending and the final manifest is unfrozen.

## Encrypted ciphertext capability handoff — 2026-10-29

### Actual blocked delivery

Owner-authorized strict/builtin/native-hook delivery of six paths creates
`e6bf25e` and [PR17](https://github.com/bohewu/yetlet/pull/17). PR CI
`40788520807/1` and master CI `40789389079/1` PASS, including Chrome. Squash Merge
selects S `0e8fb600af7aaebd4641a78c477035afe637c67e`, tree
`a4abae728a957991611c2f362ba5b005522b1abd`. The one additional
[prepare](https://github.com/bohewu/yetlet/actions/runs/40789535074), attempt 1,
reports `gate=qualification-release class=http status=403`. Earlier source,
Environment/policy, build/artifact and verifier/approval checks pass; protected
prepare/sign are skipped. There is no key step, pending approval or output
artifact. This newly observed endpoint/status does not retroactively explain
previous generic rejection `40717870186/1`. The exact provider error body and
rate-limit/accepted-permission headers were deliberately not retained.

Failure custody retains 41 files outside Git, 224,168 indexed payload bytes,
index SHA-256 `517c584090f91c6318275263dbb51f0b770a9755d229c29b35e0fb7ea258824d`.
The encrypted asset `685300857` remains on unpublished draft `426670086`, with
original ZIP SHA-256 `c90d98d20d579ed76c9ba8e705a6b204e1e43186eb3e02ec0121e91d15596168`.
Conditional post-freeze deletion was not reached. C, both tags and original
proof remain unchanged. No automatic retry follows.

### Selected local correction and boundaries

The owner asks to implement the complete data path. Use the existing encrypted
asset through an owner-resolved, age-encrypted download capability. This requires
no new upload workflow, repository write permission, PAT or Secret. Actions
artifacts remain suitable for existing build and frozen outputs, but creating a
new artifact needs a runner-side upload path; an operator REST read is not an
artifact upload. The selected minimum reuses the current stored ciphertext.

1. The owner-local `handoff` command reads the selected template, authenticated
   age binary and pinned public recipient. Existing `gh` authentication is read
   for exactly `github.com` and used only against its repository API. The command
   verifies protected master/S, Draft/tag/C and selected asset ID/name/digest/size.
   It resolves the provider's temporary binary redirect, then actually retrieves
   the ZIP **without GitHub authentication** and verifies its SHA, two-file shape
   and safe qualification receipt before emitting a request.
2. Only exact HTTPS `release-assets.githubusercontent.com` URLs under this
   repository's release-asset prefix are accepted, without userinfo, port,
   fragment or redirects. Provider capability permissions must be read-only.
   The earlier of SAS expiry and the download JWT's unverified `exp` bounds the
   deadline. This claim only shortens validity; it is not JWT authentication or
   a replacement for TLS/provider authorization and immutable ZIP/manifest hashes.
   Direct-stream API responses and unsupported destinations fail closed.
3. The short URL is encrypted with authenticated age and the existing pinned
   public recipient. Output is `request.json` and `HANDOFF-RECEIPT.json`, outside
   Git at `0700`/`0600`; temporary capability plaintext is removed. Neither the
   owner credential nor the plaintext URL enters CI inputs, logs, receipts or
   final provenance. The owner key is not accessed to make a handoff.
4. CI preflight retains source/S, Environment, original build/artifact and
   verifier/approval checks; it validates the bounded armored capsule/hash and
   expiry instead of attempting the unavailable Draft read. It requires at least
   20 minutes remaining. Original build ciphertext is fetched by the existing
   read-only step. No owner token or new credential is installed in CI.
5. Only the existing owner-approved isolated decryption phase opens the URL
   capsule with `AGE_IDENTITY`. It verifies every binding, performs a bounded
   unauthenticated download with no redirects, and checks ZIP/hash/receipt before
   any original private-build or qualification plaintext decryption. Download
   phases require five minutes remaining; a delayed approval cannot reuse the
   preflight's freshness assumption. Capability files are removed on rejection
   and success, with the existing whole-work cleanup retained. This places one
   tightly scoped ciphertext HTTPS transfer inside the protected phase; the
   credential is still restricted to that phase, with no OIDC or write permission.
6. Existing original-custody checks, qualification decrypt/full manifest, final
   assembly, sixteen subjects, output encryption and later frozen-input signing
   remain unchanged. The safe frozen-input receipt adds only the handoff hash
   and expiry, never its URL. Signing consumes frozen bytes; expiry does not cause
   a later sign job to retrieve or rebuild qualification data.

Prepare requests now require `qualification_handoff`, its SHA-256 and
`qualification_expires` in addition to the original five fields. The old bare
prepare JSON is an **operator template only**, rejected as a hosted request;
there is no silent old-carrier fallback. The two hosted modes remain prepare
and sign. Sign requests are unchanged. The handoff action is owner-local only.

### Operator use after separately authorized source delivery

Create a new repository-external `0700` evidence directory. The template has
exactly `mode=prepare`, explicitly selected replacement `signer_commit`,
`qualification_asset`, `qualification_sha256` and
`qualification_manifest_sha256`. Preserve the reviewed original data hashes.
Set `EVIDENCE_ROOT` to that directory; use the already authenticated age1.3.2:

```sh
(
  SIGN_AGE=/path/to/authenticated/age \
    python3 scripts/release-sign.py handoff \
      "$EVIDENCE_ROOT/prepare-template.json" "$EVIDENCE_ROOT/new-handoff"
)
```

Generate immediately before dispatch, **after** the reviewed S is selected;
require an unused output directory. Review the safe receipt and use the generated
request unchanged. The observed provider capability lasts about 30 minutes;
actual parsed expiry controls. Approve the real Environment gate promptly, with
sufficient time for tool/bootstrap work. Expiry stops the attempt; no automatic
refresh, retry, source replacement, signature or publication occurs. This is not
permission to commit, push or dispatch another run.

### Bounded local evidence

The real encrypted ZIP was retrieved through the new owner-to-receiver path
using a disposable **test** age identity. Both SHA-256 values match the existing
ZIP and internal ciphertext; the safe receipt matches C and the qualification
manifest. The receiver sends only `Accept`, even with a synthetic ambient token;
no GitHub credential is forwarded. Formal owner identity and qualification
plaintext were not read. All test-key/capability plaintext files were removed.
This is actual local data-transfer proof, not hosted source-S qualification.

Nine affected/new focused cases pass: request migration, preflight without Draft
access, diagnostic rejection, unchanged downstream assembly, capability
expiry/tampering/destination limits, unauthenticated hash-bound download,
receiver cleanup/bindings, owner generation and early receipt mismatch rejection.
The real transfer preceded the added explicit CLI-host binding and earlier
receipt/phase-expiry guards; these tighten checks on the same observed bytes and
are covered by focused cases rather than replaying the accepted network transfer.
Evidence is in ignored `target/yt096-signer-prepare-batch-20261028/ciphertext-handoff-20261029/`.
Syntax/docs/whitespace checks and independent Reviewer PASS0 accept this local
implementation (75 documents, 207 backlog items).
No new hosted run, formal key use, signing or publication is claimed. New code
remains uncommitted; replacement S, final frozen manifest and D6 remain Pending.

### Encrypted handoff independent acceptance

Reviewer `/root/yt096_encrypted_capability_handoff_review_20261029` returns
PASS, zero issues and required followups, for the actual eight-file diff and
retained focused/real-data proof. All eight review-input hashes match. The
Reviewer checks owner credential confinement, exact capability/data bindings,
phase-specific freshness, restricted unauthenticated retrieval, early safe
receipt checks, cleanup, redaction and unchanged signing/permission boundaries.
No product/network test is replayed and no source is changed during review.
Native role/model/effective effort match `reviewer`, `gpt-6-astra`, `max`;
this is ordinary local content review, not formal or hosted qualification.

Completion metadata alone follows review; both scripts and workflow retain
their reviewed hashes. Eight changes remain uncommitted, none staged; the
selected S stays `0e8fb60`. Guarded delivery, explicit replacement-S selection
and any additional hosted run still need owner authority. D6 remains Pending,
YT-096 In progress and YT-020 Blocked; the final manifest is not frozen.
