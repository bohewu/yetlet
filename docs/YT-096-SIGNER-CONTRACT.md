# YT-096 — Separate signer and frozen payload contract

**Owner-selected version update (2026-10-28):** The next sixteen-package
Experimental preview is `0.9.0-alpha.1`; see the [alpha version preflight](YT-096-ALPHA-VERSION-PREFLIGHT.md).
The owner now selects guarded 59-path commit `5d7b91842f8188d45e5af22367245aebec7ca7d1`
(tree `6adff26fc4fec65e9aa8f40d7172ff266b7e500d`) as the new immutable alpha candidate C.
The [minimum hosted/readback plan](YT-096-ALPHA-HOSTED-PLAN.md) records the
remaining fixed-binding patch, consumed Guard-state disposition and separate
execution-source selection/authorization gates. Existing `v0.1.0`, source
`3eacfe3`, hosted/readback runs and retained bytes stay historical and unchanged;
no alpha output or signing payload inherits their identity. No new execution C,
alpha tag, hosted run or payload SHA is selected by the proposed future patch. The separate fixed-SHA signer design still applies. D6 Pending;
YT-096 In progress; YT-020 Blocked. No tag, hosted execution, signing or publication
is authorized by this migration.

Date: 2026-10-28 (Asia/Taipei). **Design adopted; implementation and setup Pending.**
D1/D3 amendment only: D6 Pending; YT-096 In progress; YT-020 Blocked.
This contract and [ADR 0009](adr/0009-release-build-and-signer-identities.md)
preserve the [D1–D5 policy](YT-096-D1-D5-RELEASE-TRUST-PLAN.md), including D2
disclosure and custody limits. They authorize no execution, key access, signing,
publication, Commit or Push. No signer workflow or Environment is configured here.

## Immutable build and separate signer identities

| Binding | Required value |
|---|---|
| Build source C | `3eacfe31f5e3281411d3bc4653a6baccab668b30` |
| Build tree | `86385d60cb4c3e595c670dbacce117ebeeb1a993` |
| Product tag | Existing immutable unsigned `v0.1.0`; never move it |
| Build workflow/run | `.github/workflows/release.yml@refs/tags/v0.1.0`; `40628943583` / attempt1 |
| Protected verifier | `40631789238` / attempt1 |
| Signer source S | Pending: a different, future reviewed lowercase full forty-hex commit containing proposed `.github/workflows/release-sign.yml` and trusted helpers |
| Proposed signer ref | Protected `refs/heads/master` |
| Signer execution equality | Checked-out code SHA = event SHA = workflow SHA = `expected_signer_commit` = S |
| Exact certificate SAN | `https://github.com/bohewu/yetlet/.github/workflows/release-sign.yml@refs/heads/master` |
| OIDC issuer | `https://token.actions.githubusercontent.com` |
| Repository/owner | `bohewu/yetlet`, repository ID1409587171; `bohewu`, owner ID35250002 |

S is neither C nor documentation HEAD `5535a75`; no current commit is selected
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
| Public provenance | Original build `public/PROVENANCE-UNSIGNED.json`, 9022 bytes, SHA-256 `b3a1e66a34d316c6513ff4e0b9c8055c37b8f81daa598a8110f867e911a400fc`; supplied-statement DSSE whose decoded payload is byte-identical |
| Private complete evidence manifest | Final frozen assembly's `RELEASE-CHECKSUMS.json`; SHA-256 Pending; detached blob signature over its actual bytes |

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
