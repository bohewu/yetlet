# YT-096 — Development and release CI integration

Date: 2026-10-27. **D1–D5 Adopted (policy); D6 Pending.**
YT-096 In progress; YT-020 Blocked; YT-206 Done.
Original build/verify-only implementation independently reviewed PASS, zero issues
and required followups. Added age custody integration/local fixture readback PASS;
its independent review PASS, zero issues and required followups. Historical verification retains its original scope. No actual `C_release`, tag, hosted run or fresh sixteen-package output
is selected or claimed by this implementation.

For pending source/tag and one-attempt hosted authority, see the [minimum preparation plan](YT-096-HOSTED-PREPARATION-PLAN.md).

## Development path and separate authority

Use `feature/*` or `codex/*` → PR → protected `master`; optional integration can
use `feature/*` → PR → `dev` → PR → `master`. The actual default branch remains
`master`. Branch membership does not prove owner review or release approval.
Preserve required checks `Rust 1.99 MSRV` and `Stable native, samples, and Chrome`.
No branch, setting, protection or required-check change accompanies this slice.

| Lane | Checked-in boundary |
|---|---|
| Development/PR | [rust.yml](../.github/workflows/rust.yml), unprivileged PR checks and `master` pushes |
| Historical preparation | [private-rebuild.yml](../.github/workflows/private-rebuild.yml), unchanged identity guard, default original-nine or explicit sixteen |
| New release preparation | [release.yml](../.github/workflows/release.yml), manual-only fixed sixteen, read-only contents, unsigned public allowlist |
| Private readback | [evidence-verify.yml](../.github/workflows/evidence-verify.yml), separate manual protected Environment job; read-only contents/actions, safe receipt only |
| Signing/publication | No implemented path; separately authorized D6 and YT-020 remain required |

[qualification.yml](../.github/workflows/qualification.yml) retains its original
scheduled/manual nine-library qualification. This source inventory is not a live
settings attestation, hosted PASS or replay of accepted qualification.

## Fixed preparation invocation

After independent implementation review and separate approval of a real future
co-located source, dispatch `release.yml` at `refs/tags/v0.1.0` with `cohort=sixteen`
and `expected_commit` equal to that reviewed lowercase forty-character commit.
The input is an explicit operator assertion, not proof that the owner approved it.
No current HEAD is silently selected, and this guide authorizes no tag creation or
hosted invocation. The command executed by the workflow is:

```sh
cargo run --locked --manifest-path xtask/Cargo.toml -- \
  release preparation --expected-commit "$EXPECTED_COMMIT"
```

The shell checks identity before tool installation or repository Cargo execution.
Rust repeats the authoritative gate before build/tool actions: exact repository,
manual event, fixed workflow/ref/tag, clean source, HEAD = event SHA = workflow SHA
= expected commit = peeled tag target, and source ancestry on `origin/master`.
History/tags come from immutable event-SHA checkout with credentials not persisted.
Run ID/attempt must be decimal. Branch/PR dispatches and disagreements fail closed.
The existing private helper retains its original guard and provenance semantics;
the release command never impersonates `private-rebuild.yml`.

## Actual recipe and unsigned evidence

The shared recipe exports immutable source, materializes only the eighteen known
license aliases, preserves six independent locks, and uses Rust1.99.0/GNU,
OpenSSL3.5.9, default package features and fresh Cargo/target directories. Its
33 phases keep bootstrap collection separate from normal offline locked final
package verification. Fresh state is not a claim of a hermetic environment.
The workflow checks downloaded OpenSSL bytes before extraction and retains the
archive; Rust independently checks its actual bytes against the same reviewed pin.
Actual OpenSSL executable, recipe, tool and lock hashes enter the run records.
Digest pins constrain bytes; independent input authentication remains a trust limit.

The new unsigned Statement v1/SLSA v1 uses
`urn:yetlet:build:cargo-source-package:v1` and the adopted closed D2 fields in the
[D1–D5 packet](YT-096-D1-D5-RELEASE-TRUST-PLAN.md). It binds exactly sixteen actual
archive basenames/digests, source Git/export, six portable logical lock names,
recipe/tools/OpenSSL inputs, actual run/times and four byproduct digests. It embeds
no raw commands, environment, private paths or preparation-only extension fields.
This statement confers neither a SLSA level nor release approval.

Finalize raw source/build records, archive index and bounded inventory/coverage
first; derive provenance second; create/verify the complete private checksum
manifest last. Provenance never hashes itself or its containing manifest. Both
future signature bundles and final owner approval remain outside the assembly.
Exact-set checksum verification rejects missing, extra, modified, nonregular or
oversized files; existing bounds and no-overwrite behavior remain unchanged.

## Disclosure, failure retention and custody limit

| Runner-local sibling path | Contents and handling |
|---|---|
| `target/release-preparation/private/` | Sixteen archives, raw phase logs/native exit records, source/build records, archive index, coverage, final provenance and complete checksum manifest; never uploaded |
| `target/release-preparation/public/` | Only sixteen archives, `PROVENANCE-UNSIGNED.json` and its public `RELEASE-CHECKSUMS.json`; staged/verified before a success-only explicit-path upload |
| `RUNNER_TEMP` | Retained OpenSSL source and raw OpenSSL success/failure log/result; isolated shared build workspace is removed by the helper |

The adopted [age custody integration](YT-096-AGE-CUSTODY-INTEGRATION.md) captures
only the flat private assembly and three known OpenSSL extras. Ciphertext and its
safe transport receipt are separate siblings outside the frozen assembly;
`preparation::finish`, shared private recipe, frozen checksums and closed provenance
retain their existing semantics. The public eighteen-file set is unchanged.

Build/capture receives only the source-pinned public recipient. The private identity
stays outside Git, build jobs, logs and artifacts; only the separate manually
approved Environment verification job may receive it transiently. This owner-adopted
route supersedes the earlier local-only CI policy. Actual Environment protection,
Secret installation and hosted execution remain unapproved and unverified.

On success or failure, trusted preparation can encrypt available records and upload
only `private-evidence.tar.age` and `transport-receipt.json`. Failure records preserve
the native exit and `FAILED_INCOMPLETE` status, without claiming a complete success
manifest. Capture/encryption/helper/upload failure or cancellation can mean
`NOT SAVED`; there is no plaintext fallback or guaranteed cancellation capture.
The public package upload requires successful preparation, complete capture and
successful encrypted transport. Seven-day artifact transport is not permanent
archival storage. Actual download/decryption/inventory and complete-manifest
verification plus durable retention still require an owner custody receipt.

## Verification and next work

Current custody evidence accepts thirteen focused Rust checks (eight integration,
two unit current PASS and three unchanged preparation checks with retained PASS),
fmt/Clippy with warnings denied, both workflow YAML files, fifteen Bash blocks,
bootstrap Bash, Python compilation and metadata/ZIP fixtures PASS. The authenticated
age LOCAL roundtrip preserves the original/restored manifest bytes; native exit17
partial evidence and tamper rejection remain explicit. See the [custody evidence
record](YT-096-AGE-CUSTODY-INTEGRATION.md#recorded-local-evidence).
`complete_product_custody=false`: no actual OpenSSL/sixteen-package build or hosted,
release or durable production custody qualification is claimed. The five bounded
implementation repairs remain recorded; no failure is outstanding. Independent
review of this custody delta PASS, zero issues and required followups.


Focused fixtures cover identity disagreements and valid isolated Git source/tag,
closed CLI/workflow, actual digest bindings/closed fields, exact public disclosure,
failure retention and unchanged private identity refusal. Related selected private
recipe tests and the existing checksum suite protect the shared boundary.
No hosted run, actual sixteen-product train, native/browser/CSP/security/platform
qualification, external service, OIDC/certificate, signing, publication, commit or
push is performed in this slice. Original preparation implementation review PASS,
zero issues and required followups; earlier planning review PASS covers planning
only. Both precede the current custody delta and do not attest its acceptance.

Next unblockable YT-096 work is separate owner selection/approval of actual
source/tag, protected Environment/Secret setup and hosted preparation. Age tool bootstrap is locally authenticated at Linux amd64 scope;
Cosign/D6 bootstrap, exact final payloads/bundles, durable custody and immutable
registry publication remain separate later work. Docker
image delivery is outside this sixteen-source-package slice.
