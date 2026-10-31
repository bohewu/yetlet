# YT-215 — First independent CLI release

Date: 2026-10-31. Status: In progress. Contract and implementation independent
Reviewer PASS0; actual release preparation/output review remain required. Stop at
the final owner publication gate; no CLI registry upload is authorized yet.

## Selected scope

Publish only the developer tool `yetlet-cli` version `0.1.0`, executable `yetlet`.
It depends on the already-published `yetlet = "=0.9.0-alpha.5"`; neither SDK
package is a release subject, a staged predecessor or an upload in this cohort.
SDK versions, existing tags/releases/signatures and historical failures remain
unchanged. No dependency upgrade, runtime change, prebuilt binary or new sample.

Use closed release-record **schema3 / cli**, exactly one allowed package. Read
its version from `tools/yetlet-cli/Cargo.toml`, independently of the SDK workspace
version. Use **`cli-v0.1.0`**; the existing SDK `v0.1.0` tag is unrelated and must
not move. Cargo publication does not choose or increment a version.

The CLI README describes the installation interface without a permanently stale
unpublished label. Registry availability and actual execution are tracked here.
Before the first upload, `cargo install yetlet-cli --locked` is the selected
interface, not a claim that the version already exists.

## Existing release path and authority

Extend the existing source-bound `release.yml` preparation, age custody,
`release-sign.yml`, independent result verification and `release-publish.yml`.
Keep schema1/sixteen and schema2/consolidated behavior and all closed asset sets.
The CLI public set is **five files**: its one `.crate`, unsigned provenance,
`RELEASE-CHECKSUMS.json`, `provenance.sigstore.json` and reduced
`SIGNING-APPROVAL.json`. Complete private evidence stays encrypted and outside
the public release. GitHub publication is an independent Experimental pre-release,
not Latest, followed by the same exact CLI archive on crates.io.

The owner authorizes preparation through the final gate: necessary reviewed
source delivery with strict/builtin Guard, PR/required CI/Merge, immutable CLI tag,
one hosted build/readback, automated exact signing and separate verification.
The final `release-publish` approval remains with the owner. Do not approve that
gate or perform a GitHub/public registry publication during preparation.

Preserve the existing signer certificate identity, issuer, public disclosure
fields, trust roots, Environment separation, age recipient/key custody and
disabled admin bypass. C is the green merged support source; S must differ from
C. A narrow result-document delivery can provide S after actual build/readback.
The existing evidence Environment's exact-tag restriction can advance from the
completed SDK cycle to this CLI tag without changing its key or protection rules.
Keep the old execution receipts. No private key is exported or read locally.

Bounds: one45minute unsigned build; original readback5/15minutes;
prepare/sign5/30minutes each; separate result5/30minutes; final no-secret
publication preflight10minutes and owner-gated job60minutes. Preserve failures;
no automatic write or job retry. A failed attempt does not renew its budget.

## Missing proof and reuse

Reuse [YT-214](YT-214-CLI-INSTALLATION.md)'s twelve focused CLI tests and actual
isolated source install/new/build/HTTP cleanup at their recorded input/scope
limits. Current CLI runtime, templates, resources, manifest and lock stay the
same; only its distribution README changes. Retain the HTTP byte-evidence limit
and earlier tool/cache/harness failures. Reuse SDK qualification and accepted
master CI; do not replay Windows/browser/GPU/runtime or external-service suites.

Supplement only these changed or previously absent boundaries:

- Closed CLI cohort, own version/manifest/tag, one-package vendor/normal verified
  packaging, one provenance subject and exact five-file public set. Reject SDK
  tag/version/cohort substitutions and preserve existing SDK semantics.
- The new CLI registry envelope contains only its archive and references the
  published SDK version. It never uploads SDK dependencies. Read-only absence
  checks reject an existing CLI version before GitHub or registry writes.
- The post-publication consumer anonymously installs the exact CLI using its
  packaged lock, checks version/help and generated SDK registry pins. It has no
  publication token, Cargo path patch or source replacement; it does not compile
  the CLI as though it were a View library.
- Newly public CLI dependency coverage: cargo-audit0.22.2 checks its own61-entry
  lock against database `2295466c4e9df26647fd67d8fa1c705ac2fc7f81`,1395 advisories:
  zero vulnerabilities/warnings. It inherits the repository's existing
  `RUSTSEC-2023-0071` RSA exception from `.cargo/audit.toml`; the CLI lock has no
  `rsa` package, so this exception conceals no applicable CLI finding. No new
  ignore, target filter or stale override is added. Existing cargo-deny policy
  passes licenses/sources including dev/build inputs;
  unused license-allowance warnings remain nonblocking. No policy is weakened.
- Normal135-file package verification after the README change, followed by actual
  clean-tag hosted archive/checksum binding, original readback, frozen payloads,
  signatures and independent verification. Old archive/signature identities do
  not attest to this new source.

Local fixture correction is retained: the first CLI public-set assertion used
the wrong checksum filename; the product used the existing correct filename.
One focused fixture correction passed without replaying the other19 passing
checks. This was a harness failure, not an archive or signing defect.

## Final review and execution record

Run-local evidence is under `target/yt215-cli-first-release-20261031/`.
Candidate C, S/V/P, immutable tag, actual run/artifact IDs, output hashes, durable
custody and exact publication request remain unselected until their respective
stages succeed. Only complete actual results may be promoted into these fields.
The final owner summary must identify the one CLI archive/checksum, five public
attachments, release notes, destinations and exact gate/run; SDK re-publication
must be absent. The owner reports token scope `yetlet-*` plus `yetlet`; this is
not a token readback or authenticated upload-rights receipt.
