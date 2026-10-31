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
version. Use **`yetlet-cli-v0.1.0`**; the existing SDK `v0.1.0` tag is unrelated and must
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
source-bound hosted build/readback, automated exact signing and separate verification.
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
At the initial implementation checkpoint, C, S/V/P, tag and run/output identities
were unselected. The dated checkpoint below records actual progress; remaining
identities and the exact publication request are selected only after their
respective stages succeed. Only complete actual results may be promoted into these fields.
The final owner summary must identify the one CLI archive/checksum, five public
attachments, release notes, destinations and exact gate/run; SDK re-publication
must be absent. The owner reports token scope `yetlet-*` plus `yetlet`; this is
not a token readback or authenticated upload-rights receipt.

## Source and tag checkpoint — 2026-10-31

[PR36](https://github.com/bohewu/yetlet/pull/36) passes both required checks in
41164540635/1 and is squash-merged at selected C
`6f26961b7b0871b3262e36ff0c52f382453a949f`; exact-source master CI41165089029/1
also passes. The merged tree matches the reviewed strict/builtin index tree.
Sixteen support files are delivered; the only CLI-package input change relative
to accepted4f8e919 is its distribution README. Runtime, templates, resources,
manifest/lock and published SDK inputs remain unchanged. Original focused
proof, audit and the corrected independent review retain their actual scope.

The new lightweight unsigned tag `cli-v0.1.0` is pushed and read back exactly at C
through strict/builtin accepted-remote-history/native Guard PASS. The consumed
private Guard binding is explicitly retired byte-preservingly; no tag/history or
hook changes. Old `v0.1.0` and `v0.9.0-alpha.5` stay fixed. The source-support
branch is removed after merged-tree correspondence and a verified external bundle.

One [CLI build41165581554/1](https://github.com/bohewu/yetlet/actions/runs/41165581554)
is dispatched at that tag with a45minute bound. At this checkpoint it is running;
no archive hash, successful custody, frozen payload or signature result is claimed.
The existing original-evidence Environment's one exact tag policy advances to
`cli-v0.1.0`; the key, disabled admin bypass and worker/final reviewer protections
are unchanged. Secret values are never read locally.

This narrow documentation delivery provides a separately reviewable later S,
without changing builder/signer/runtime inputs. Its actual green merge SHA will
be bound only after CI. Build/readback may complete independently while that
source is delivered; prepare/sign cannot start before the complete actual record
and required live-master CI pass. Remaining actual results live in the run record
until output review and closeout. No write/job retry, SDK upload, public CLI
release or crates.io upload is authorized by this checkpoint.

### Subsequent build completion

The same build41165581554/1 subsequently PASS without retry. Original public and
encrypted-private ZIP digests match API metadata and durable copies are retained
outside the repo. Public checksums and the sole CLI provenance subject verify.
The CLI archive SHA-256 is `e53a1389a667b153425901602cf18bc084c54b192330d74c25f2ae95fafe46ae`,
size 471525 bytes. No SDK archive is present. Original protected readback is
now dispatched once at the same immutable CLI tag; decryption/custody completion
is still Pending. This status update uses actual retained receipts and does not
claim a signature, final verification or publication result.

## Custody identity correction — 2026-10-31

The original build41165581554/1 and readback41166121653/1 report GitHub success.
The public CLI archive/provenance and encrypted ZIP digest checks remain valid at
their recorded scope. However, inspection of the actual readback receipt finds
`git_ref` and builder workflow bound to the SDK `v0.9.0-alpha.5` tag. The helper
used the SDK workspace version during both capture and verification, so a green
job did not establish the intended CLI custody identity. Reject that receipt as
CLI identity proof; preserve the original jobs, artifact bytes and wrong-ref
receipt. No CLI payload has been signed or published.

The material fix derives capture and verification context from the validated
record's schema/cohort, own manifest version and source SHA. A CLI record rejects
SDK refs, wrong sources and incompatible schema/version combinations; existing
SDK and standalone legacy context retain their behavior. Focused identity tests
cover these affected boundaries. No dependency/runtime/platform suite is replayed.

Keep `cli-v0.1.0` immutable as the original preparation attempt. Before any first
publication, adopt the full package-name namespace `yetlet-cli-v0.1.0` for the
corrected source; CLI0.1.0 and SDK0.9.0-alpha.5 remain unchanged. PR37 now delivers
this material fix rather than selecting its earlier documentation-only head as
S. Select a new C only after its required CI and merged-master CI pass; new-source
build/readback must prove the corrected actual receipt fields. The original
one-build/one-readback attempts remain consumed and recorded; this changed-source
qualification is not a retry of either old job. Continue the same task/evidence
ledger and final owner gate, without moving old tags, resetting counters or
claiming future outputs. The later S, payloads, signatures and result proof
remain Pending.
