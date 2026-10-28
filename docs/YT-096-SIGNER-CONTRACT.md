# YT-096 — Separate signer and frozen payload contract

**Current alpha preparation checkpoint — 2026-10-28:** The owner accepts
preparing the two signing inputs and a separate signer workflow. Original alpha
build `40669449040`/attempt1 and manually approved verifier `40688772221`/attempt1
PASS at the [recorded custody scope](YT-096-ALPHA-HOSTED-PLAN.md#actual-alpha-protected-verifier-and-retention--pass).
The execution source is `bf3363801132f4481263ceb411d9bba3349df86d`, immutable unsigned
`v0.9.0-alpha.1`, with sixteen Experimental `0.9.0-alpha.1` libraries. The earlier
`5d7b918` was version-preflight C, not this hosted execution source. Historical
`v0.1.0`, source `3eacfe3`, build/readback and retained payloads remain unchanged.

Date: 2026-10-28 (Asia/Taipei). **Design adopted; signer implementation and setup
Pending.** Public provenance bytes are now locally fixed and identified below;
the complete private-manifest freeze, signer S and signing authority remain
Pending. The source/payload readback is not a new independent implementation
review. D6 Pending; YT-096 In progress; YT-020 Blocked.

This contract and [ADR 0009](adr/0009-release-build-and-signer-identities.md)
preserve the [D1–D5 policy](YT-096-D1-D5-RELEASE-TRUST-PLAN.md), including D2
disclosure and custody limits. Local preparation does not authorize hosted jobs,
new Environment/Secret placement or private-key access, OIDC, signing,
publication, Commit or Push. No signer workflow or Environment is configured yet.

## Immutable build and separate signer identities

| Binding | Required value |
|---|---|
| Build source C | `bf3363801132f4481263ceb411d9bba3349df86d` |
| Build tree | `c4ee13aa558489ba3962874dbf5e0a0a1bc99605` |
| Product tag | Existing immutable unsigned `v0.9.0-alpha.1`; never move it |
| Build workflow/run | `.github/workflows/release.yml@refs/tags/v0.9.0-alpha.1`; `40669449040` / attempt1 |
| Protected verifier | `40688772221` / attempt1 |
| Signer source S | Pending: a different, future reviewed lowercase full forty-hex commit containing proposed `.github/workflows/release-sign.yml` and trusted helpers |
| Proposed signer ref | Protected `refs/heads/master` |
| Signer execution equality | Checked-out code SHA = event SHA = workflow SHA = `expected_signer_commit` = S |
| Exact certificate SAN | `https://github.com/bohewu/yetlet/.github/workflows/release-sign.yml@refs/heads/master` |
| OIDC issuer | `https://token.actions.githubusercontent.com` |
| Repository/owner | `bohewu/yetlet`, repository ID1409587171; `bohewu`, owner ID35250002 |

S is neither C nor current documentation HEAD `b271af0`; no current commit is selected
implicitly. Verify the certificate's authenticated workflow/build-signer digest
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
administrator bypass `false`. These are design values, not observed settings.
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
| Trusted signer | Pending implementation/review, fixed S, tool/root bootstrap and supplied-statement compatibility |
| Protected preparation | Pending separate reviewed Environment/Secret setup and explicit hosted execution authority; no OIDC permission |
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
not product code execution or a new product build. This checkpoint is a concrete
input/authority plan; it does not claim that `release-sign.yml` is implemented.

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

Next unblockable work is separately authorized signer implementation and review;
setup/custody, payload preparation, exact D6 issuance/signing and publication each
need their own authority. Cosign/root bootstrap and compatibility remain open.
This document creates no certificate, signature, payload assembly or hosted result.
