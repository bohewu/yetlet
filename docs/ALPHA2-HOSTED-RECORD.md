# Alpha.2 — Hosted build and rejected custody binding

Date: 2026-10-29 (Asia/Taipei). Status: Sixteen-package build PASS;
custody qualification FAIL. No evidence verifier or private-key operation was
started. The original artifacts and incorrect binding remain unchanged.

## Authority and immutable inputs

The owner approved candidate `af7bbe5d44c7fbfa87a294203f55183d1fd92d05`,
unsigned immutable `v0.9.0-alpha.2`, one sixteen-package build (45 minutes),
then exact-tag Environment rollover/readback only after acceptable build evidence.
Master CI40892880090/1 PASS preceded execution. No signing, publication or
automatic retry was authorized.

The existing-remote-tag assessment and armed binding passed strict/builtin;
normal native pre-push consumed that binding once. Remote tag/source match C,
tree `9527a1e592e1e5435978a5d8bf317859d15a735d`. Original alpha.1 stays at
`bf3363801132f4481263ceb411d9bba3349df86d`. No tag was moved, signed or forced.
The used Guard binding remained selected at the failed build's closeout.
Its subsequent owner-authorized [alpha.3 retirement](ALPHA3-VERSION-PREFLIGHT.md)
preserves the binding bytes and tag; no rearming is inferred.

## Actual build and evidence discrepancy

One [release build40895514392/1](https://github.com/bohewu/yetlet/actions/runs/40895514392)
completed successfully at C/tag alpha.2. Both original ZIP downloads match their
Actions size/digest. The public allowlist contains sixteen alpha.2 `.crate`
files, unsigned provenance and public checksums; all public checksums/subjects
and source/tag/run identities match.

| Binding | Expected / public provenance | Actual encrypted-transport receipt |
|---|---|---|
| Source/workflow SHA | `af7bbe5d44c7fbfa87a294203f55183d1fd92d05` | Matches |
| Run/attempt | `40895514392` / `1` | Matches |
| Workflow ref | `release.yml@refs/tags/v0.9.0-alpha.2` | `release.yml@refs/tags/v0.9.0-alpha.1` |
| Git ref | `refs/tags/v0.9.0-alpha.2` | `refs/tags/v0.9.0-alpha.1` |

The receipt's ciphertext size/SHA also matches the retained encrypted bytes.
That does not repair the wrong tag claim. Build success is not custody or release
acceptance; original receipt/inventory bytes must not be relabeled or rebuilt.

Root cause: `xtask/src/release/custody.rs` still constructed and validated its
binding with fixed alpha.1 constants. Its CLI synthesized those fields rather
than deriving the selected version. The earlier migration inventory incorrectly
treated these active capture/verify constants as historical pins. Versioned
workflow/API checks did not cover the helper's serialized binding.

The discrepancy was detected from source and then confirmed in the actual safe
receipt before any Environment mutation or protected verifier dispatch. The
evidence Environment still has exactly the alpha.1 tag policy; sign/result
Environments, Secrets and the age identity remain untouched. No local private
plaintext or private key was accessed. No knowingly incompatible decrypt job
was dispatched to obtain another failure.

## Retention and minimum correction

Original ZIPs, public files, ciphertext, safe receipt and API/Guard/core-record
metadata are retained in an owner-only directory outside the repository.
`target/yt096-alpha2-hosted-20261029/retention-plan.json` identifies that location;
`artifact-correspondence.json` records the actual mismatch and artifact hashes.
The custody index explicitly marks qualification FAIL, not verified preservation.
This owner-host retention is not an offsite-backup or indefinite-availability claim.

The minimum local correction derives custody workflow/ref from the same compiled
workspace manifest as packaging. Existing source/workflow-SHA/run/attempt,
recipient, archive, checksum and no-clobber rules remain. Existing positive CLI
fixtures follow the manifest, and capture/verify negative cases retain legacy
and mismatched-version rejection. The rotation drill's descriptive binding is
copied from its actual synthetic capture receipt instead of a duplicate version
literal; no real drill or key operation is replayed. No historical receipt, public API, runtime,
dependency, key, tag, workflow permission or accepted signature changes.

Three affected existing cases PASS offline/locked: current-version closed CLI,
capture/verify identity refusal before tools, and inventory validation. Owning
library/test Clippy with `-D warnings` and format PASS. No old package, consumer,
browser, crypto, platform or hosted check was replayed. Independent correction
Reviewer `/root/alpha2_version_independent_review_20261029` PASS0, all six
source hashes unchanged. The review also verified 42 retained-index entries,
original artifacts and ten immutable source inputs. Native reviewer/model/actual
high effort are verified; only completion metadata follows. Subsequent delivery
authority belongs to the separately approved alpha.3 batch.

The immutable alpha.2 tag cannot select corrected code. The subsequent owner
decision authorizes the bounded [alpha.3 recovery batch](ALPHA3-VERSION-PREFLIGHT.md):
version synchronization, affected package checks, reviewed Git delivery,
used-binding retirement and one new tag/build/readback. This alpha.2 record does
not select corrected C. Preserve
alpha.2 as an unpromoted candidate and all evidence; do not move its tag, edit
its receipt, add a permissive legacy fallback or waive the mismatch.

Alpha.1 publication/D6 remain Complete; YT-020 remains Blocked. Alpha.2 custody
is not qualified, positive schema-1 hosted qualification remains Pending, and
no new signing or publication approval follows from the build's green status.
