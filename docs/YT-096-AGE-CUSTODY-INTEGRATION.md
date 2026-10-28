# YT-096 — Encrypted evidence custody integration

Date: 2026-10-27. **Custody scheme/D2 extension Adopted; local integration/readback PASS;
independent implementation review PASS (zero issues and required followups).** D6 Pending; YT-096 In progress; YT-020 Blocked.
Only code integration and local proof are authorized now. No GitHub settings or
Secret installation, hosted run, tag, commit/Push, OIDC, signing or publication.

## Files and trust boundary

[release.yml](../.github/workflows/release.yml) uses only the public recipient in
[release-evidence-recipient.txt](../.github/release-evidence-recipient.txt), pinned
by SHA-256 `b861a3c7e2abf3caf4c7071c045f6114704db94e8cc39ce010e7fa10711ff506`.
The private identity never enters Git, build jobs, logs or artifacts. Its local
copy is preserved outside Git for authorized proof, with directories0700/files0600.
The latest owner choice permits transient key use only in the isolated, manually
approved Environment job in [evidence-verify.yml](../.github/workflows/evidence-verify.yml).
It supersedes the earlier local-only CI policy; actual Secret setup awaits authority.

[bootstrap-release-age.sh](../scripts/bootstrap-release-age.sh) pins age1.3.2,
Sigsum0.13.1 and Go1.27.3. The local Linux amd64 bootstrap verified archive/proof
SHA-256 and the explicit `sigsum-generic-2025-1` two-of-three witness quorum before
first age execution. It uses official Go HTTPS/published checksum and module
checksum-database trust roots, not TUF or independently reproduced binaries.
No global install or hosted qualification is claimed; no floating latest/root override.

## Recorded local evidence

Run `yt096-age-custody-20261027` retains `generation-result.json` and
`local-roundtrip-proof.json`. Thirteen focused Rust checks are accepted: eight
custody integration and two custody unit checks currently PASS; three unchanged
preparation checks retain their earlier PASS. Cargo fmt and Clippy with warnings
denied PASS. Both workflow YAML files, fifteen embedded Bash blocks, bootstrap
Bash, Python compilation and direct metadata/ZIP fixtures PASS. These checks do
not relabel the original twenty-seven checks or replay product qualification.

The authenticated age local round trip verifies the original/restored LOCAL
manifest byte-for-byte, SHA-256
`bbbea5d051f243d722963e8b8667eca261df5fcd8f1a97edc62c77ce2e7bdf06`.
Native exit17 partial evidence remains incomplete. Ciphertext-hash tampering,
age authentication failure with recomputed receipt hash and original-manifest
mismatch are rejected. All retained fixture directories are0700 and files0600;
local identity metadata is unchanged and no key bytes were uploaded.

**`complete_product_custody=false`:** the synthetic fixture contains no actual
pinned OpenSSL source/build result or sixteen-package build. This is local
capture/readback/full-LOCAL-manifest proof only, not hosted qualification, release
approval or durable production custody. Five bounded implementation repairs and
their failures are preserved in run records; no failure remains outstanding.
Actual Environment/Secret/protection setup and manual execution are unperformed
and unauthorized. Independent implementation review PASS, zero issues and required followups;
all eighteen reviewed source hashes matched before factual review-status updates.

## Minimum local use

Use authenticated `$AGE`, the existing private identity `$IDENTITY`, an approved
source `$SOURCE` and original decimal `$RUN`/`$ATTEMPT`. These variables are operator
placeholders; no concrete release source/run or private filesystem location is
published here. `$TRANSPORT` and `$READBACK` must be new destinations; plaintext
readback and its private parent must be outside the checkout. Preserve private
permissions and avoid shell tracing. The build workflow invokes the same CLI:

```sh
cargo run --locked --manifest-path xtask/Cargo.toml -- release custody capture \
  --age "$AGE" --recipient .github/release-evidence-recipient.txt \
  --source "$SOURCE" --run "$RUN" --attempt "$ATTEMPT" \
  --private "$PRIVATE_ASSEMBLY" --extras "$OPENSSL_EXTRAS" \
  --exit "$ORIGINAL_EXIT" --output "$TRANSPORT"

cargo run --locked --manifest-path xtask/Cargo.toml -- release custody verify \
  --age "$AGE" --recipient .github/release-evidence-recipient.txt \
  --source "$SOURCE" --run "$RUN" --attempt "$ATTEMPT" \
  --transport "$TRANSPORT" --identity "$IDENTITY" --output "$READBACK"
```

Capture accepts only the flat private assembly and known extras
`openssl-3.5.9.tar.gz`, `openssl-build.log`, `openssl-build-result.json`.
It writes only `private-evidence.tar.age` and `transport-receipt.json` to transport;
the encrypted inventory binds source/workflow/run/attempt, recipient hash, exact
members/digests, native exit and completeness. Neither whole runner/Cargo homes
nor environment dumps are captured. Frozen checksums/provenance remain unchanged.

Readback rejects mismatched identity/bindings, ciphertext, unsafe members, invalid
inventory or evidence digests. Successful preparation also requires full frozen
private-manifest verification. It writes private `private/`, `extras/`, inventory
and `local-readback-receipt.json` under the new readback directory. The receipt
records actual local verification; it proves neither builder authenticity nor
permanent retention/release approval. A verified failure inventory remains
`FAILED_INCOMPLETE`, with its original exit; it is not successful preparation.
Generic CLI failure is `custody NOT SAVED or readback rejected; private diagnostics suppressed`.
No overwrite, plaintext fallback or raw diagnostic publication is permitted.

## Future protected Environment verifier

Before any separately authorized setup/run, the owner must configure and review
Environment `release-evidence-verify`, required reviewer `bohewu`, exact custom
**tag** policy `v0.1.0`, and disabled administrator bypass. The single owner may
request and approve a job. Store the matching private identity as Environment
Secret **`AGE_IDENTITY`**, scoped only to this verifier; do not use a repository
Secret or build-job key. GitHub's documented Environment gate releases secrets
only after protection approval ([Environment contract](https://docs.github.com/en/actions/concepts/workflows-and-actions/deployment-environments),
[reviewing deployments](https://docs.github.com/en/actions/how-tos/managing-workflow-runs-and-deployments/managing-deployments/reviewing-deployments)).
Environment administrators remain a trusted policy root. Actual settings, no-bypass
configuration, Secret installation and hosted approval behavior are not verified.

The manual workflow inputs are `expected_commit`, `build_run`, `build_attempt`,
`artifact_id`, `artifact_sha256` (the upload-artifact ZIP digest without `sha256:`).
Dispatch at the approved immutable `refs/tags/v0.1.0` only after separate authority.
The source, workflow SHA, event SHA and peeled tag must agree and belong to
`origin/master`. The preflight runs before scheduling the Environment secret job;
it repeats inside that job before key use. Build permissions are `contents: read`;
verifier permissions are `contents: read`, `actions: read`. No write/id-token path.

[release-evidence-preflight.py](../scripts/release-evidence-preflight.py) reads:

| REST resource | Required evidence |
|---|---|
| `GET /repos/bohewu/yetlet/actions/runs/{run}/attempts/{attempt}` | Original manual release workflow, exact source/tag/run/attempt/repository, completed success or failure |
| `GET /repos/bohewu/yetlet/actions/artifacts/{artifact}` | Exact encrypted artifact name/ID/digest, bounded size, unexpired and original repository/run/source |
| `GET /repos/bohewu/yetlet/environments/release-evidence-verify` | Existing Environment and documented required reviewer/custom policy |
| `GET /repos/bohewu/yetlet/environments/release-evidence-verify/deployment-branch-policies` | Exactly one custom tag policy `v0.1.0` |
| `GET /repos/bohewu/yetlet/actions/artifacts/{artifact}/zip` | Exact ZIP digest and only the two transport files; no token forwarded to storage redirects |

See GitHub's [workflow-run API](https://docs.github.com/en/rest/actions/workflow-runs),
[artifact API](https://docs.github.com/en/rest/actions/artifacts),
[Environment API](https://docs.github.com/en/rest/deployments/environments) and
[branch-policy API](https://docs.github.com/en/rest/deployments/branch-policies).
Missing/misconfigured Environment, unavailable API permission, missing/mismatched
run/artifact, expiry or unsafe download fails closed with
`release evidence metadata/download rejected; no private key accessed`.
Do not silently create an unprotected Environment or invent approval booleans.
The gate does not claim API proof of undocumented bypass/current-attempt approval
fields; manual approval relies on GitHub's protected Environment runtime contract.

The verifier creates ephemeral0700/0600 private files and removes key/plaintext
on exit with a final cleanup step. Only the safe readback receipt is uploaded.
Cancellation/runner loss can prevent execution or cleanup; no capture guarantee
exists. A cancelled/missing capture is `NOT SAVED`. The seven-day encrypted artifact
and seven-day receipt are transport caches. Download within the planned24-hour
window, verify before expiry and retain ciphertext plus actual receipt under owner
custody through final disposition. Upload/readback alone does not prove durability.
Public eighteen-file package disclosure is unchanged; adopted D2 additionally
permits ciphertext and safe source/run/status/size/digest verification metadata.

## Manual key review and replacement

The next authorized local slice is [manual rotation preparation and a synthetic
test-key drill](YT-096-KEY-ROTATION-DRILL.md), based on custody commit `4793620`.
Implementation is delivered; local drill PASS; independent Reviewer acceptance PASS (zero issues and required followups).
Nine focused tests, two offline builds and 25 real commands cover the bounded
LOCAL cases; the first extraction failure and one material repair are retained.
New rotation changes remain uncommitted. This permits isolated LOCAL test keys,
not owner-key rotation or GitHub Secret recovery. D6 Pending, YT-096 In progress
and YT-020 Blocked remain unchanged.

Review custody periodically and replace on an identified leak or access change;
no automatic30/90-day rotation or actual rotation is authorized. Record recipient
fingerprint/version with each ciphertext. Use a newly reviewed source-pinned public
recipient and matching versioned Environment Secret for new encryption; exact future
Secret naming is an operator choice. GitHub does not synchronize custom age rotation.
Retain an old decrypt key while retained ciphertext needs it, or re-encrypt retained
private evidence and verify a new receipt outside the unchanged frozen
manifest/provenance. Never overwrite/delete a still-needed old key. Re-encryption
cannot revoke copied old ciphertext and does not change original build provenance.

Next step: obtain owner approval of the reviewed source and protected Environment/Secret setup. Source-freeze/settings/Secret/hosted execution still requires
separate authority. Hosted execution does not automatically unlock D6 approval,
signing or publication. No actual `C_release` is selected.
