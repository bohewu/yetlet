# YT-020 — Private Public-Key Signing Verification

**Date:** 2026-10-12
**Scope:** Actual maintainer-key Private preparation signing; no formal release activation.
**State:** Three frozen preparation objects signed and independently verified at Private public-key scope. The final repository reviewer result is reported separately against the final diff. YT-020 remains **Blocked**.

## Current preparation applicability (2026-10-12)

All signing, public-only PASS results and negative-case diagnostics below retain their historical `8e557e760407a3d4b3f5a8b8f833748946be1387` source and frozen object identities. The [T08 boundary](YT-020-PRIVATE-RELEASE-PREP.md#current-preparation-boundary--t08--yt-068-2026-10-12) records later policy HEAD `abb2cce1253afe1d87a97e0a50f3f47efffe7280` plus dirty research/documentation, changed root manifest/lock and nine new candidate READMEs. The old signatures, 37 artifacts and manifest do not attest those new bytes. No signed object, public trust anchor, historical SBOM or receipt is regenerated, relabeled or replaced here. Current-source preflight/rebuild/provenance/SBOM/license/signing evidence requires a separately approved immutable candidate and execution authority. YT-020 remains Blocked for formal activation; the independent delivery gate/result for this documentation is recorded separately in the run; documentation-stage checks do not attest that gate.

## Authority and policy commit

After independent policy reviewer PASS, the maintainer authorized peon to commit that policy, create a dedicated Cosign key, record its fingerprint, sign the Private evidence and independently verify it, then obtain a separate repository review. The original checkout is retained. Push, publication and visibility changes are not authorized.

Peon committed exactly seven policy documents as `abb2cce1253afe1d87a97e0a50f3f47efffe7280`, parent `8e557e760407a3d4b3f5a8b8f833748946be1387`, tree `59b556fac98f6268299ef5d75550bdc13f2d9d6f`. Strict/builtin staged Commit Guard passed with zero findings and the accepted tree matched immediately before commit. The reviewed source bytes were preserved. Native peon thread `01a12e3c-f5d0-7de3-b0ec-bbac96979c37` observed `gpt-6-luna` / effective `high`; no push occurred.

[Private trust policy](YT-020-PRIVATE-TRUST-POLICY.md) remains the preparation contract. The user delegated this enrollment and bounded signing operation; no separate human fingerprint ceremony, formal release approval or independently operated signing authority is claimed.

## Actual public trust anchor and custody

| Identity | Actual value |
|---|---|
| Owner and purpose | `bohewu`; Private preparation only |
| Algorithm | ECDSA P-256 / SHA-256 |
| Fingerprint convention | SHA-256 of DER SubjectPublicKeyInfo |
| Public-key fingerprint | `6e995c563c322fe14912c8660932bb8b8c803bb837007b596c883bd9b4da5fef` |
| Public PEM file SHA-256 | `d6f71a4e794368f425c2546d267bc5c293b89a04a9a7a53e6fe93316622a7fea` |
| Cosign | `3.1.3`, Linux amd64 |
| Cosign binary SHA-256 | `4629c757b7618056f8ddd7e2625ae9fdd94c0372a65049520bc7d9df9efc7f71` |

Cosign generated one password-encrypted key in a network-disabled container. Private key, unlock material, separately protected encrypted recovery copy and enrolled public anchor reside outside the repository. Owned directories use mode `0700`; key, credential, recovery and anchor files use `0600`. The recovery copy matches the encrypted key bytes and lives in another protected directory on the same workstation; this is not off-host disaster recovery. Private material is absent from artifacts, Git, browser state and CI caches. The unlock value is delivered privately through stdin, never in command arguments or retained logs.

The enrolled public key/fingerprint is supplied to verification independently of the artifact set. An artifact's included public-key identity record cannot enroll its own trust. Local custody locations are retained only in a restricted, ignored receipt for the maintainer.

## Frozen inputs and actual signatures

The signed artifact source is immutable `8e557e760407a3d4b3f5a8b8f833748946be1387`, not the later policy commit. The retained primary/GitHub run `38195517029` rebuild comparison has nine identical archive digests and input digest `8e5799c319d626812daca22c90feba41a41d23d07ecd642af4d6d3506dc7afb7`. Lockfile SHA-256 remains `7260eef21ce05abbe90ebccc4b255b3da07cdf5cd0acac51fdbb90ee7b3d4b57`. No package/build/browser/Docker qualification suite or SBOM generation was repeated to refresh identities.

The new flat Private evidence set contains **37 artifacts plus its manifest**: nine archives, 18 retained native-target inventories, primary/hosted build and provenance records, source inputs, comparison, inventory coverage/boundary and public-key/signing-input identities. The existing checksum CLI verified original input bundles and the newly assembled set. The inventory boundary preserves earlier overlay-source generation, platform execution identities and omissions; signing does not expand inventory coverage.

Cosign signed three exact objects: `RELEASE-CHECKSUMS.json`, `PRIMARY-PROVENANCE.json` and `HOSTED-PROVENANCE.json`. All three signing commands exited 0. Manifest SHA-256 is `a3e2aa936e673cd5ef5a5f9b3cf24d295dddf5a4d5338a2d9d240592f123abb3`. Inputs are frozen; detached bundles live separately to avoid a checksum/signature cycle. Original provenance bytes are preserved. Their original `BUILD-RECORD.json` byproduct names map to the separately named primary/hosted copies through the signed input-identity record.

The pinned signing invocation uses an explicit key, `--use-signing-config=false`, `--tlog-upload=false`, `--oidc-disable-ambient-providers=true` and a local bundle output. Containers use `--network=none`, read-only root/input/key mounts, dropped capabilities and no-new-privileges; only signature output and ephemeral scratch are writable. No public Rekor/Fulcio/TSA, OIDC call or registry upload is performed.

For Cosign 3.1.3, the explicitly selected public-key/no-log verification flow needs `--insecure-ignore-tlog`. This is confined to Private preparation; it cannot be used in the [formal release gate](RELEASE-SUPPLY-CHAIN.md#independent-rebuild-provenance-and-approvals). A separate negative verification keeps log verification enabled and must reject these no-log bundles. No certificate/SCT verification is disabled to report formal release success.

## Independent verification boundary

A separate native generalist agent runs verification with only public inputs in read-only, network-disabled containers. No private key, password, primary build cache, Git checkout or Docker daemon socket is mounted. This is separate technical verification under the same maintainer/workstation control, not an independent human release approval or independently controlled signing service. Expected fingerprint and manifest/source identities come from the coordinator's enrolled trust/snapshot record, rather than trusting bundle-provided claims.

Native verifier thread `01a12e47-9f6c-7ee0-bf6d-9ee58f79214b` observed `gpt-6.1-sol` / effective `high` and returned terminal `done`. Its final container exited 0 and was removed. Three canonical signatures verified; the existing checksum CLI accepted all 37 artifacts. All 54 approved Git source inputs, including 18 materialized license aliases, matched; 36 retained source-to-copy identities, both statement/build-record bindings and 18 historical SBOM hashes matched. Expected fingerprint and manifest identity also matched. Frozen inputs remained unchanged.

| Actual verification | Expected and observed result |
|---|---|
| Three canonical signatures | Each Cosign invocation exited 0 |
| Complete checksum set | Existing CLI exited 0 for 37 artifacts |
| Wrong existing public key | Cosign exited 1 |
| Modified manifest bytes | Cosign exited 1 |
| Modified provenance bytes | Cosign exited 1 |
| Modified subject copy | Original manifest signature remains valid; checksum CLI rejects modified archive |
| Correctly signed historical snapshot | Historical signature verifies with exit 0; approved-current-source comparison rejects `633db585` instead of `8e557e7` with exit 1 |
| Log-required verification without the Private skip | Exited 1: `failed to verify log inclusion: not enough verified log entries from transparency log: 0 < 1` |

All six negative cases passed their expected rejection checks. The older overlay-source provenance was signed separately only as an authentic historical negative fixture; it was not approved as the current snapshot. No canonical inputs or historical records were mutated. The log-required rejection is evidence that the formal gate remains unmet, not a formal verification PASS.

Retained evidence is under ignored `target/yt020-private-signing-20261012/`: public identity, restricted custody receipt, input/signature snapshots, exact nonsecret signing commands, independent verification outputs and role/effort traces. No new reusable validator or cryptographic implementation is added.

## Historical invocation diagnostics and formal limits

An initial post-keygen label assertion expected `ENCRYPTED COSIGN PRIVATE KEY`; the actual pinned tool emits `ENCRYPTED SIGSTORE PRIVATE KEY`. The check was corrected against the actual encrypted header without regenerating or replacing the key. Key generation itself exited 0; later signing used that same key. The peon resolver initially rejected an explicit null context; removing the absent context corrected the pre-dispatch input before any peon spawn. These are setup/assertion diagnostics, not failed cryptographic signatures or omitted product failures.

Independent verification retained one Git-symlink input-check correction and two terminal container setup failures: capability-dropped root could not read the owner-only public anchor, corrected by running as its owner; a tmpfs copy retained read-only archive mode, corrected only on that disposable copy. The final bounded attempt passed. Host permissions and frozen artifacts were not changed; every failed/final container was removed. Full diagnostics and retry accounting remain in the verification receipt. No product/content repair or broader rerun occurred.

Private public-key signing authenticates maintainer-approved statement bytes. GitHub OIDC identity, certificate/chain, transparency/inclusion, trusted timestamp, authenticated builder provenance and SLSA-level claims remain **Not verified**. All formal source/tag/tool/target/inventory/license/rebuild/approval/promotion guarantees are preserved. There is no approved release tag, registry ownership/resolution or publication; future [crates.io policy](YT-020-PRIVATE-TRUST-POLICY.md#future-cratesio-distribution) is unchanged. YT-065/066/067 retain their bounded Done states; YT-020 remains Blocked for formal activation.
