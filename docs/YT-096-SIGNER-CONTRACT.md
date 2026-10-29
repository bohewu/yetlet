# YT-096 — Separate signer and frozen payload contract

**Subsequent owner-approved GitHub pre-release (2026-10-29):**
[Yetlet v0.9.0-alpha.1](https://github.com/bohewu/yetlet/releases/tag/v0.9.0-alpha.1)
is public as an Experimental pre-release, not Latest, at unchanged C `bf33638`.
The owner separately approves publication of the exact notes and 20 public
attachments; anonymous API and bundle-download readback PASS. No private evidence,
private bundle, registry upload, new signing or Git operation is included.
D6 remains Complete, YT-096 Done at accepted source-package signing/integrity
scope, YT-020 Blocked. Earlier unpublished/Pending records retain their dated scope.

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

Date: 2026-10-29 (Asia/Taipei). **Current state: actual signing and independent
public/private verification and final Reviewer PASS; owner conditional acceptance activated, D6 Complete.**
PR19 squash Merge selects S `d65e3180ad3076659f03c069fc71cd0e7e3f0518`, with
reviewed tree `5812dcfbc9e50a76c4d230bd5d32a336568db12a` and PR/master CI PASS.
Owner-approved prepare `40806988578/1` and sign `40811275645/1` both pass every
step, including cleanup. The owner adopts the fixed confirmation packet and
personally approves signing, then authorizes one local private-verification
exception and conditional acceptance of the actual fixed bundle hashes.
[Actual execution and acceptance](#actual-signing-independent-verification-and-owner-acceptance--2026-10-29)
record both independent cryptographic checks, all 117 complete-manifest entries,
16 subjects, cleanup and completed Reviewer condition. Original C/tags, frozen
bytes and all previous failures stay unchanged. No new source selection, signing
retry, Git operation or publication is authorized. D6 Complete;
YT-096 Done at source-package signing/integrity scope; YT-020 Blocked. Earlier checkpoints retain their
original source, result, review and acceptance scope.

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
| Selected signer source S | `d65e3180ad3076659f03c069fc71cd0e7e3f0518`, after PR19 squash Merge |
| Signer tree | `5812dcfbc9e50a76c4d230bd5d32a336568db12a`, identical to reviewed correction commit `a29796b66fb19686474ce7f120ae3eecf694ce5b` |
| Proposed signer ref | Protected `refs/heads/master` |
| Signer execution equality | Checked-out code SHA = event SHA = workflow SHA = `expected_signer_commit` = S |
| Exact certificate SAN | `https://github.com/bohewu/yetlet/.github/workflows/release-sign.yml@refs/heads/master` |
| OIDC issuer | `https://token.actions.githubusercontent.com` |
| Repository/owner | `bohewu/yetlet`, repository ID1409587171; `bohewu`, owner ID35250002 |

S is not C. It is the actual owner-authorized PR19 squash commit, with the
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
age and Cosign3.1.4 and pin valid approved trust roots/configuration before use.
Authenticated bootstrap and local supplied-statement compatibility evidence are
retained; actual OIDC issuance/service acceptance remains Pending. Never use floating
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
| Private complete evidence manifest | Final frozen assembly's `RELEASE-CHECKSUMS.json`; SHA-256 `e0921075e47a5c2f16948dde94bf006ec8afac14c8730ba68b22d28d0533d93f`; detached blob signature over its actual bytes |

The public payload is copied byte-for-byte from the verified repository-external
alpha custody into ignored preparation storage, flushed and reread with that
size/hash. That earlier public-only copying checkpoint fixed the public bytes;
the successful protected preparation below subsequently froze the full assembly.
Neither operation grants D6 or signing approval. The old
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
the final manifest SHA can differ. The successful protected preparation exports
the original private build-manifest SHA as
`9585635674d073b740b83ccd4d1e7edf1e60aab52e1b0a7efff8102a911cc4ec`;
this is not the final signing target above. Public `RELEASE-CHECKSUMS.json`, retention-index,
inventory SHA and ciphertext SHA cannot substitute for either private manifest.

## D6 confirmation packet — 2026-10-29

This is the historical packet presented before owner signing approval. Its
original Pending language records that checkpoint; the actual owner decisions,
signatures and conditional final acceptance are recorded below.

**Recommendation: accept one later signing execution for the exact two raw
payloads above, subject to the bindings and disclosure below. D6 is still
Pending; this packet requests no execution now.** Sixteen Experimental
`0.9.0-alpha.1` source-package subjects are covered by the unchanged provenance;
this is two payload signatures, not sixteen separate package-signing operations.
It adds no binary, platform, production-readiness or SLSA-level claim.

### Frozen input binding to accept together

| Input | Exact value |
|---|---|
| Original product/build | C `bf3363801132f4481263ceb411d9bba3349df86d`, unsigned immutable `v0.9.0-alpha.1`, build `40669449040/1`, verifier `40688772221/1` |
| Signer | S `d65e3180ad3076659f03c069fc71cd0e7e3f0518`; workflow `.github/workflows/release-sign.yml` at protected `refs/heads/master` |
| Successful preparation | Run `40806988578`, attempt `1`; original artifact `12867774698`, `signing-inputs-40806988578-1`, 74,815,047 bytes |
| Artifact ZIP SHA-256 (`digest`) | `bee5cb493b35ea681a20054e7a095b13989d523c3f7a8dd17a4c3cbd9c8d23c8` |
| Safe `SIGNING-INPUTS.json` SHA-256 (`receipt_sha256`) | `f7a0cc5df0f471f3d47526fb8e41822a647e51519124ec43fddd338f5e9570a9` |
| Frozen ciphertext SHA-256 | `89cbc583a9c27f4cd4c1c3350ae3c1cd47ca241458b1920adac237adc2143e52` |
| Assembly `EVIDENCE-CLOSURE.json` SHA-256 | `a13ccb85554d4ab68f7aaf95bd151758841a03392496a59685a4adf1bce2760c` |
| Public raw payload SHA-256 (`provenance_sha256`) | `4d4de1a8ce9cc077c0aa62a1aafb3ab9799eed312deddb66d9e6f8fc6fef2d68` |
| Complete private raw manifest SHA-256 (`manifest_sha256`) | `e0921075e47a5c2f16948dde94bf006ec8afac14c8730ba68b22d28d0533d93f` |

The ZIP, ciphertext, receipt, closure and retention-index hashes are bindings,
not additional signing targets. The full private manifest is the final frozen
assembly manifest, not the old private build manifest or a public summary.
The preparation receipt's `D6: Pending` remains immutable after approval; a later
sign request/execution receipt records the new authority outside that assembly.

### Public metadata and controlled material

| Disclosure class | Exact identity or accepted field scope |
|---|---|
| Full public statement and DSSE bundle | The unchanged 9,158-byte provenance: `_type`, exactly sixteen archive names/SHA-256 values, `predicateType`, build type `urn:yetlet:build:cargo-source-package:v1`; original repository/release workflow/alpha tag and `cohort=sixteen`; Rust1.99.0, GNU target, default-feature policy, OpenSSL3.5.9 and source epoch; source/lock/recipe/tool dependency names and digests; builder ID, original invocation/times and four byproduct names/digests. No new field or replacement builder S is inserted |
| Certificate identity | OIDC issuer `https://token.actions.githubusercontent.com`; exact SAN `https://github.com/bohewu/yetlet/.github/workflows/release-sign.yml@refs/heads/master`; authenticated workflow/config/signer digest S above |
| Repository and workload claims | `https://github.com/bohewu/yetlet`, ID1409587171; `https://github.com/bohewu`, ID35250002; ref `refs/heads/master`, `workflow_dispatch`, `github-hosted`, `public`, Environment `release-sign`; expected subject `repo:bohewu@35250002/yetlet@1409587171:environment:release-sign`, distinct from the certificate SAN |
| Runtime signing metadata | Actual future signing run/attempt URI, certificate validity/serial/public key/chain/SCT, signatures and approved log ID/index/inclusion/checkpoint/time and TSA material. Future run, certificate and bundle hashes do not exist yet and must not be invented |
| Private manifest disclosure | Its digest, detached signature and certificate/log material may be public through signing services. The manifest body, filenames/entries, detailed evidence and raw findings remain controlled; the manifest bundle is encrypted before upload |

GitHub workload OIDC authenticates to Fulcio; the adopted production Sigstore
profile uses Fulcio `https://fulcio.sigstore.dev`, approved Rekor API2
`https://log2025-1.rekor.sigstore.dev` / API1 `https://rekor.sigstore.dev` with
the adopted `ANY` selection, and TSA
`https://timestamp.sigstore.dev/api/v1/timestamp`. Cosign3.1.4 and root/config
digests remain those bound by the preparation receipt and
[D4](YT-096-D1-D5-RELEASE-TRUST-PLAN.md#d4--concrete-signing-and-verification-recommendation).
Actual service/certificate acceptance still needs the later authorized signing
run. Treat the full statement, certificate and transparency material as public
and durable; deleting a GitHub asset/release cannot retract public log copies.
Raw JWTs authenticate only to their intended service; tokens, private keys,
short-lived handoff URLs and decrypted evidence are never public payload fields.
Existing source/Actions publicity is separate from disclosure of private evidence.

### Recommended single owner decision and execution limit

A later explicit owner decision should accept this whole packet: exact C/S,
both payload SHA-256 values, receipt/artifact binding, workload identity and
public disclosure; authorize **one `sign` dispatch**, five-minute preflight and
thirty-minute protected job, with the owner personally approving `release-sign`.
Retain failures/partial issuance and stop without automatic retry. Reuse the
frozen assembly, verify both raw hashes and every required binding before OIDC,
remove the decryption identity before signing, then verify and retain the actual
bundles. No new build, reassembly, payload edit, tag, GitHub Release publication
or crates.io upload is included. Signing success alone does not close YT-096:
independent verification and final owner acceptance of actual output digests
remain required. Publication and YT-020 retain separate authority.

Keep protected master at S for that proposed dispatch. These result documents
may be committed on a separate documentation branch under later authority, but
must not be merged into master before signing at S. The existing gate rejects
a moved master rather than silently selecting new code. If it advances, stop
for owner disposition; this packet authorizes no replacement S or re-prepare.
No Commit/Push, issuance, signing or publication is authorized now.

## D6 completion and the GitHub draft boundary

D6 is concrete authority for identity issuance/signing and later promotion; it
is not satisfied by a green unsigned build or an evidence-decryption receipt.
Keep the following conditions separate:

| Gate | Actual alpha status / required outcome |
|---|---|
| Source, unsigned build, verifier and retained bytes | PASS at the existing bounded scope; reuse without replay |
| Public signing bytes | Fixed original bytes; owner-approved supplied-statement DSSE issued in `40811275645/1`, independently verified |
| Complete private evidence assembly | PASS: allowlisted closure, protected checksum verification, freeze/encryption and final manifest SHA exported in `40806988578/1` |
| Trusted signer | Reviewed S `d65e318`; actual issuer/SAN/S/repository/owner/Environment/run claims and normal trust/CT/log/timestamp verification PASS for both bundles |
| Protected preparation | `40806988578/1` SUCCESS including cleanup and retained readback; no OIDC permission. Earlier failed runs retain FAIL and are ineligible |
| Exact D6 signing approval | Owner accepts the reviewed exact packet by reference and personally approves `release-sign`; exact original C/S/payload/artifact/receipt preserved |
| Actual signatures and verification | Sign `40811275645/1` and independent public/private checks PASS; 16 subjects and all 117 complete-manifest entries verified; wrapper errors retained, no cryptographic check/signing retry |
| Final disposition | All owner conditions PASS, including final independent Reviewer PASS0; acceptance of actual fixed public/encrypted-private bundles is activated, D6 Complete and YT-096 Done at accepted source-package signing/integrity scope |

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

## Actual frozen output and Go cache cleanup failure — 2026-10-29

The owner approves guarded delivery of the eight reviewed handoff paths and one
new prepare. Commit `4292754`, tree `07404415d79443c83109a1205b0ccb6a25584cd7`,
passes strict/builtin staged/outgoing checks and native hooks.
[PR18](https://github.com/bohewu/yetlet/pull/18) CI `40794539309/1` passes;
squash Merge selects S `e2d4a9f64925f2a20c35c7e7aeae6e352e4938ef` with that same
tree. Automatic master CI `40795652719/1` passes. Original C/tags remain intact.

The owner-local helper creates a fresh capsule using the original public
recipient, proves unauthenticated original ZIP retrieval/hash/receipt, and emits
handoff SHA `13ee340c421a5384cbe87f296bc569367860035c36fbcf0ead0a04f5041f199f`.
The owner then personally approves Environment `25580949441`; approvals identify
user `35250002`. [Prepare run 40795794183](https://github.com/bohewu/yetlet/actions/runs/40795794183),
attempt 1, passes all data/tool steps through upload. Both the one-shot dispatch
and the real formal-key phase occurred at this fixed S; sign is skipped.

| Actual stage | Result |
|---|---|
| Preflight `123191847380` | PASS |
| Tool/root authentication and trusted helper build | PASS |
| Original build ciphertext/Environment recheck | PASS |
| Isolated capability retrieval, custody/qualification decryption, full manifest, sixteen subjects and encrypted freeze | PASS |
| Encrypted frozen output/receipt upload | PASS |
| Final owned-work cleanup | FAIL: 200 permission-denied entries, all under the task-private Go module cache |
| Overall prepare job `123191897780` / workflow run | FAIL; never relabeled PASS |
| Sign job `123191900050` | Skipped; no OIDC issuance or signatures |

Full logs show no `rm` errors outside `sign-tools/age-tools/gomodcache`; the
isolated phase and its own work cleanup complete successfully. This does not
constitute an independent post-job filesystem inspection or full cleanup PASS.
The failure is the bootstrap's default read-only Go cache combined with ordinary
recursive removal, not a failure of the observed data/manifest/upload steps.

### Preserved candidate readback

Artifact `12863583130`, `signing-inputs-40795794183-1`, has size 74,815,048 and
ZIP SHA-256 `561058586354c54c935af000f7b5fac3d1084d93ae12f606dd257262cfc49b4b`.
The original ZIP and exact two regular payload files are downloaded and retained
outside Git. ZIP, ciphertext and all safe receipt bindings match source S/C,
run/attempts, original build/verifier/artifact, qualification data, handoff,
recipient and authenticated tools. No local private decryption occurs.

- Safe receipt SHA-256: `b6b79e3b98a21c1caa226ea27a004fab367ac7d2eb951e6785340f06025e19a5`.
- Original public provenance SHA-256: `4d4de1a8ce9cc077c0aa62a1aafb3ab9799eed312deddb66d9e6f8fc6fef2d68`.
- Observed frozen candidate manifest SHA-256: `e0921075e47a5c2f16948dde94bf006ec8afac14c8730ba68b22d28d0533d93f`.
- Original private build manifest SHA-256: `9585635674d073b740b83ccd4d1e7edf1e60aab52e1b0a7efff8102a911cc4ec`.
- Frozen ciphertext SHA-256: `e47f1d0544acc240f31310eec7f1c79d169187312f5e1a4bec716086b83a4f27`.

These are actual candidate bytes, not signing authority or accepted final
inputs. The existing run gate still rejects the failed workflow; no
`continue-on-error`, exception or relaxed status acceptance is added. The
qualification Draft asset is retained because the successful eligible-run
condition for deletion was not reached. D6 Pending; final accepted manifest
remains Pending, despite the now-known candidate hash.

### Minimum local cleanup repair

Set `GOFLAGS=-modcacherw` only in the existing task-private age bootstrap, whose
Go cache/path/toolchain environment already points exclusively at its empty
owned directory. [Go documents this flag](https://go.dev/ref/mod#build-commands)
for creating removable module-cache directories. No global `go env` settings,
sudo, hook, scope expansion, crypto version/hash, sumdb policy, key or permission
change occurs. The normal cleanup command and strict gate are retained.

One focused native Go case uses the existing authenticated Go1.27.3 and pinned
Sigsum module ZIP through an offline file proxy. It first reproduces ordinary
`rm -rf` failure with the old default, then proves full removal using the flag
read from the actual bootstrap script. The original test failure and passing
repair are retained. Offline fixture mode is solely a permission/removal test;
it does not replace production sumdb/signature checks. Temporary fixture caches
are removed with scoped `go clean -modcache`, including on assertion failure.
No prior qualification, crypto bootstrap, private preparation or hosted job is
replayed. Shell/Python syntax and docs PASS (75 documents / 207 backlog items).
Independent followup Reviewer PASS0 accepts the six-file local repair/status
delta, with zero issues or required followups; it does not certify a repaired
hosted run. Only completion metadata changes after that review; the two code
files retain their reviewed hashes.
New repair/status changes remain uncommitted; no new S or hosted retry is
implied. The effective release status remains YT-096 In progress / D6 Pending,
with YT-020 Blocked and no signing/publication.

## Successful protected prepare and retention — 2026-10-29

The owner authorizes strict/builtin delivery of the six independently reviewed
cache-cleanup correction/status paths, CI-gated squash Merge, actual replacement
S selection and one additional prepare. Commit `a29796b`, tree
`5812dcfbc9e50a76c4d230bd5d32a336568db12a`, passes staged/outgoing strict/builtin
checks and native hooks. [PR19](https://github.com/bohewu/yetlet/pull/19) CI
`40805140129/1` PASS; squash Merge selects S
`d65e3180ad3076659f03c069fc71cd0e7e3f0518` with that exact tree and parent
`e2d4a9f64925f2a20c35c7e7aeae6e352e4938ef`. Master CI `40806189605/1` PASS.
Original product C, both immutable tags and accepted build/verifier stay unchanged.

The owner-local helper verifies original encrypted qualification ZIP/receipt and
creates a fresh age capability handoff without reading the local formal key.
Exactly one dispatch creates
[prepare 40806988578](https://github.com/bohewu/yetlet/actions/runs/40806988578),
attempt1. Read-only preflight `123228194489` PASS. The owner personally approves
Environment ID25580949441; GitHub's approval record identifies user ID35250002.
Protected prepare `123228240593` passes every step: authenticated tools/helper,
Environment/original input recheck, isolated handoff/decryption/full checksum
verification/freeze, encrypted upload and final owned-work cleanup. Sign job
`123228242222` is skipped; no OIDC issuance, signature or publication occurs.
No failed job or hosted attempt is automatically retried.

Artifact `12867774698` and its exact two regular payload files are downloaded
once for owner readback. ZIP size/digest, ciphertext digest and every safe receipt
binding match the fixed C/S, original build/verifier, qualification, preparation,
recipient and authenticated tools. The complete private manifest SHA is exported
by the authorized protected verification, not obtained through local plaintext
decryption. The original provenance bytes are also matched to their retained
9,158-byte public copy. The two raw signing hashes are in the packet above.
This confirms preparation eligibility; it is not D6 signing authority or proof
of a certificate/signature that has not been issued.

The owner-local repository-external custody set retains the original artifact
ZIP, frozen ciphertext, safe receipt, a copy of the original encrypted
qualification ZIP, source/Guard/CI/approval/run/job/log records, prior focused
cleanup-regression/review evidence and actual readback/cleanup receipts. Its
final index covers 42 files / 168,101,480 payload bytes, SHA-256
`46eb08c7cbccac1f569d0505c6fad7749e5166cb3b20958622c40c27370f0140`;
the index itself is excluded to avoid a self-hash. Files are private, flushed,
reread and indexed; this is local owner custody, not an off-host backup claim.
The frozen GitHub artifact's recorded expiry is 2026-11-05 10:35:06 Asia/Taipei.
The future signing gate still requires the original successful run/artifact;
local custody does not authorize reupload or reconstruction if it expires.

After successful readback and durable retention, the previously authorized
conditional cleanup deletes only temporary qualification asset `685300857`:
HTTP204, then owner-authenticated Release readback confirms zero assets and
`draft=true`, `published_at=null`. Draft `426670086` remains unpublished at C.
Both original/local encrypted qualification copies, both product tags, original
build artifacts and all failed-run custody records remain. In particular,
`40795794183/1` stays FAIL/ineligible, even though the final manifest bytes match
this successful preparation. Its prior index SHA remains
`1b374f38414cd7dcf770c24deef2e6a1c977645e560e48820324e059e24687fc`.

Execution/readback evidence is retained under ignored
`target/yt096-owned-cache-cleanup-delivery-20261029/` and the owner-local
repository-external run custody set. This documentation update reuses it without
replaying tests, builds, bootstrap, decryption or hosted jobs. Documentation
checks PASS (75 documents / 207 backlog items), and whitespace PASS. Independent
Reviewer `/root/yt096_d6_confirmation_review_20261029` PASS, zero issues or
required followups, accepts the six-file documentation delta against retained
evidence without replaying validation. Native role/model/effort verification
matches the requested reviewer configuration; this is ordinary independent
review, not formal signing qualification. Only completion metadata changes after
review. Six documentation files remain uncommitted and unstaged; no Commit/Push,
signing or publication occurs. D6 Pending, YT-096 In progress, YT-020 Blocked.

## Actual signing, independent verification and owner acceptance — 2026-10-29

The owner instructs continuation to complete D6 immediately after the reviewed
exact confirmation packet. This accepts that packet by reference: unchanged
C/S, both raw payload hashes, exact frozen artifact/receipt and adopted public
statement/workload/certificate/log/TSA disclosure. The single `sign` request uses
`d6=approve-exact-signing-inputs` and consumes successful prepare `40806988578/1`,
artifact `12867774698`; no source change, product rebuild or payload reassembly.

[Sign run 40811275645](https://github.com/bohewu/yetlet/actions/runs/40811275645),
attempt1, succeeds at S `d65e3180ad3076659f03c069fc71cd0e7e3f0518`. Preflight
`123241908097` PASS; the owner personally approves Environment25580949441,
with user ID35250002 confirmed by GitHub. Protected sign job `123241955416`
passes authenticated-tool bootstrap, frozen input download/verification,
identity removal before OIDC, both signing operations, normal same-job
verification, encrypted upload and cleanup. Prepare job `123241956871` is skipped.
There is one sign dispatch/attempt, no automatic hosted or signing retry.

### Actual immutable signed outputs

| Output | SHA-256 |
|---|---|
| Artifact `12870551335`, `signing-results-40811275645-1`, 20,484-byte ZIP | `27ea5d5315f17ca9728943c944db653b6f56e2297a4efded76ded4b7c170eef7` |
| Immutable `SIGNING-EXECUTION.json` | `8a7648ed0fd13c8fa065289514b84a10299c9394bc0b27e5d1c4b92d94e6f2ac` |
| Public `provenance.sigstore.json` | `524c4f2aa6c611959d3c3b6acf6e27dce3e0a55cc58c68a80513345d20434a21` |
| Controlled `manifest.sigstore.json.age` | `37608522d8c8162d62052b7d3fe918ae1d1ac5014c148564fabf36923d4700b8` |
| Raw private manifest bundle, transiently decrypted for verification only | `8a365ad5c7f530e98b08b5a26aa70c0d1d0db764bbc701d571f525b70c6618e0` |
| Signed original provenance | `4d4de1a8ce9cc077c0aa62a1aafb3ab9799eed312deddb66d9e6f8fc6fef2d68` |
| Signed complete private manifest | `e0921075e47a5c2f16948dde94bf006ec8afac14c8730ba68b22d28d0533d93f` |

Original ZIP and its exact three regular members are downloaded, hashed and
bound to successful run/attempt/S and the exact approved input request. The
execution receipt and original preparation receipt are immutable: their
`independent_verification` / `final_owner_acceptance` Pending values describe
issuance time. Later checks/acceptance are recorded separately, not patched
into those original bytes or into the signed assembly.

### Independent technical verification and bounded custody exception

On the owner host, retained authenticated Cosign/root independently verify the
public DSSE signature, chain/CT/log/timestamp and exact issuer/SAN. Exact
certificate claims match S, source/config digest, repository/owner IDs, actual
signing invocation and `release-sign`. The decoded statement is exactly the
original 9,158 bytes; its 16 subject digests match all retained public archives.
This is a new verification of actual signatures, not a rebuild or qualification
rerun. A missing local interpreter path interrupted only the not-started
certificate wrapper; system cryptography completes that check without replaying
the successful Cosign/subject checks. The operational error is retained.

For the private result, the owner explicitly answers: permit local verification
and accept/close after all checks pass. The approved one-time exception uses the
existing owner-local identity directly with authenticated age in private temporary
storage outside Git, with a 300-second total limit. No key copy, move, rotation,
upload or export occurs; the default future D2 protected-Environment rule remains.

The existing bounded restore/checksum verifier passes all 117 complete-manifest
entries and the actual manifest SHA matches the approved target. A local
bookkeeping wrapper then uses `name` instead of the manifest's `path` field and
raises KeyError; actual complete-entry verification has already passed. Its
temporary plaintext is removed. One canonical harness correction continues only
the not-started checks within the original phase/deadline: reacquire metadata
from identical authenticated ciphertext, bind all 16 subjects to the already
verified entries and closure, decrypt the private bundle, and run independent
Cosign `verify-blob` with exact issuer/SAN, pinned root, signed timestamps and
normal CT/log checks. Every private certificate S/repository/owner/Environment/run
claim matches, as does its message digest to the actual complete manifest.
No passed full-entry check, cryptographic check or hosted job is replayed.

The complete phase takes 146.12 seconds, including correction. Both temporary
plaintext directories are removed; existing identity metadata is unchanged.
Private bundle/body/evidence plaintext is not retained or uploaded. Only safe
verification/cleanup records, public certificate/log metadata, raw bundle digest
and original encrypted bytes are retained. The wrapper failure and both cleanup
records remain available; it is not relabeled as a signing/cryptographic failure
or silently dropped from history.

Public certificate SHA-256 is
`b490d4b3a134c3191a573b7ad386dd326826f8b48b54d172c5b151a610f6c12e`;
private-signature certificate SHA-256 is
`5fce264e6b9f2a66437d10fc572a5fa4c334d5213a429e825d7f425744dd249a`.
Verified Rekor log indices are `174117699` and `174117716`; each bundle has one
RFC3161 timestamp. These are actual independent-check observations, not future
identities inferred from a green build. They add no support, SLSA or production
qualification claim.

### Conditional final acceptance and retained scope

The owner approves the one-time local exception and accepts the **already
existing** public bundle SHA `524c4f2a…` and encrypted private bundle SHA
`37608522…`, with the fixed raw provenance/manifest, S and original C, conditional
on independent private verification, plaintext cleanup and independent Reviewer
all passing. The raw private-bundle SHA above is subsequently learned as a
binding to that approved ciphertext; no unknown replacement output is approved.
Verification/cleanup and final independent Reviewer
`/root/yt096_d6_confirmation_review_20261029` PASS, zero issues/required followups.
Native reviewer/model/effort evidence matches the resolved configuration. The
owner's existing conditional acceptance is therefore activated against these
exact output identities; D6 Complete, YT-096 Done at the accepted source-package
signing/integrity scope. This ordinary independent review is not a new formal
assurance or product/platform qualification claim.

Signed output, immutable receipts, exact source/run/job/approval metadata,
independent verification results, original wrapper failures, cleanup and owner
authority are retained outside Git. The pre-final-review custody index covers
30 files / 123,052 payload bytes, SHA-256
`704c41a65f8db02012b798530486cd4180a4c9fc2572f9d8d480d5ba595dd439`.
It references the unchanged original frozen preparation custody; no private
plaintext is included. This is owner-local retention, not indefinite/off-host
backup. That pre-review index remains immutable. Final repository-external
retention adds final review/authority/acceptance records: 36 files / 134,686
payload bytes, final index SHA-256
`e940be86c4d121696c93a4465593624c01208306e336cedfc3bb6a2946952d9d`.
The index excludes itself; document completion metadata stays outside its hash
closure. A separate immutable `FINAL-OWNER-ACCEPTANCE.json` records the condition
activation; original issuance receipts, signed bytes and failure records remain
unchanged. The one local identity exception is spent and grants no further use.

This closes only the accepted sixteen Experimental `0.9.0-alpha.1` source-package
signing/integrity scope. All original failures,
qualification limits and evidence dispositions stay intact. No GitHub Release
publication, crates.io upload, tag change, Commit or Push occurs; YT-020 remains
Blocked. The six-document result update, docs (75 documents / 207 backlog items)
and whitespace checks PASS; final independent Reviewer PASS0. Only completion,
status and final custody metadata change after review. All six documents remain
unstaged/uncommitted; no old qualification is rerun.

## Subsequent GitHub pre-release publication — 2026-10-29

After D6 closure, the owner separately approves publishing the prepared existing
GitHub Draft as Experimental `v0.9.0-alpha.1`. Release ID `426670086` is published
at 2026-10-29 12:09:39 Asia/Taipei, with `draft=false`, `prerelease=true` and
`make_latest=false`. Existing unsigned lightweight tag and C `bf33638` stay
unchanged; signer S `d65e318` and both signed payload/bundle identities remain
those in the accepted record above. No new tag, build, signature or Git action.

The exact 20 public attachments are the original sixteen `.crate` subjects,
original `PROVENANCE-UNSIGNED.json` and public `RELEASE-CHECKSUMS.json`, unchanged
public `provenance.sigstore.json`, and a reduced `SIGNING-APPROVAL.json` containing
only adopted source/tag/subject/manifest/bundle identities and support limits.
They total 358,948 bytes. Complete private evidence, manifest body, private bundle
and raw operational records are not Release attachments. The original public
checksum manifest is distinguished from the signed complete private manifest;
GitHub-generated source ZIP/tar archives are not signed subjects.

The owner approves the exact staged notes, SHA-256
`f16a974b7136e27202db42c7eca7b6c4f4b201069c5bfc9f85caac2bcd04c6ce`.
The notes reflect actual signing/verification completion and retain Experimental,
source-package-only, compatibility and no-registry limits. Owner-authenticated
and anonymous API readbacks both confirm all 20 names/sizes/digests and published
state. Anonymous public bundle download matches the already verified
`524c4f2aa6c611959d3c3b6acf6e27dce3e0a55cc58c68a80513345d20434a21`;
no passed signature/qualification check is replayed. The published pre-release
is not the latest stable release.

A draft-update wrapper initially omits `tag_name`; GitHub returns its internal
untagged placeholder, and tag-based CLI asset lookup fails before uploading any
file. Explicit full tag/C binding corrects that draft-interface error, then each
of the 20 assets is uploaded once using the same existing Release ID, without
clobber or changed credentials/permissions. The lookup, observed temporary
binding and correction records are preserved. There is exactly one publication
PATCH; no publication or failed binary-upload write is automatically repeated.

Publication receipts, exact public copies, notes/allowlist, owner authority,
readbacks and retained interface errors are stored outside Git. Publication index
covers 56 files / 572686 bytes, SHA-256 `677f68d2eb14e0b23b432d1025b6f3b8a33927848f66c9c73adb20c4d1aba63e` (index excluded).
It binds the unchanged signed-output/frozen-input custody indexes rather than
copying private material into the public set. This is local owner custody, not
an indefinite/off-host-backup claim.

GitHub publication does not include crates.io upload or change YT-020's Blocked
state. No new support/production/SLSA claim follows. D6 Complete and YT-096 Done
retain their accepted source-package signing/integrity scope. The six local result
documents remain uncommitted/unstaged. Their prior independent acceptance covers
D6; this later publication checkpoint records the owner's separate approval and
actual API/asset readback only.
