# Alpha.3 — Custody correction and coordinated package preflight

Date: 2026-10-29 (Asia/Taipei). Status: Required local affected checks and final
independent Reviewer PASS0; optional synthetic readback FAILED/UNVERIFIED.
No new immutable C is selected by this record.

## Approved bounded batch

After the [retained alpha.2 custody failure](ALPHA2-HOSTED-RECORD.md), the owner
approved one recovery batch: retire its used Guard binding without deleting the
tag/evidence, synchronize sixteen libraries to `0.9.0-alpha.3`, perform affected
checks and independent review, then strict/builtin Commit/Push/PR. Both required
checks must pass on the exact head before squash Merge/source selection. One
new unsigned immutable tag/build (45 minutes) and one protected evidence readback
(preflight5/protected15 minutes) are authorized only at valid source/output
bindings. The owner personally approves the Environment gate. Failures remain;
no automatic retries, signing, Release/public writes or registry upload.

Baseline master is `af7bbe5d44c7fbfa87a294203f55183d1fd92d05`, tree
`9527a1e592e1e5435978a5d8bf317859d15a735d`. Original alpha.1 publication/D6
stay Complete, alpha.2 stays an immutable unpromoted candidate, and YT-020 stays
Blocked. None of their signatures, receipts or verdicts is reassigned to alpha.3.

The approved Guard retirement PASS archives the original used binding unchanged
under fingerprint `0ab3f761e3eb7831750ae6ffba768e1a3d7695811d1153b26d04d410c91b750d`.
This is metadata cleanup, not rearming, history remediation or tag deletion;
strict/builtin and native hooks remain enforced.

## Minimum correction and version scope

Custody capture construction and capture/verify validation now derive workflow
and tag refs from the same compiled workspace manifest as package preparation.
Source/workflow-SHA/run/attempt, recipient, checksum/archive, no-clobber and
receipt equality before decryption remain. Current positive fixtures follow
that authority; negative cases reject independently wrong versions, legacy refs
and source mismatches before tools. The existing real roundtrip additionally
asserts the actual serialized workflow/ref. Rotation drill reporting takes its
descriptive binding from the actual synthetic receipt instead of a second literal.

The version flow inventory covers the root manifest, release record/workflow,
Rust preparation/custody, Python build-evidence preflight, signer/result/public
readers and rotation reporting. Production version refs are source/record-derived;
the remaining alpha.1 string in preparation's negative unit case is deliberately
invalid input, not a live pin. No legacy receipt adapter, wildcard identity,
old-output rewrite, new public API, framework runtime or key rotation is added.

All sixteen versions and eighteen normal/optional/dev internal requirements move
together. Eleven libraries inherit the root and five retain isolated manifests.
Nineteen locks change only 112 local version fields; all 2,372 sourced package
blocks across twenty-one tracked locks are byte-identical to baseline. Current
usage in thirteen package READMEs changes. Independent tool/example/vendor
versions, features, targets, MSRV and third-party dependency versions stay fixed;
twenty-six unpublished workspace examples inherit the root as before without
joining the release cohort. Historical alpha.1/alpha.2 documents remain dated.

## Affected evidence

Evidence is retained under `target/yt096-alpha3-release-20261029/` (local, ignored).
The existing alpha.2 package recipe/consumer are reused with alpha.3 inputs and
fresh output paths. Rust/Cargo1.99.0, four build jobs and local GNU/system OpenSSL
are local preflight, not a new Ubuntu24/Windows/GPU support qualification.

| Check | Actual result / boundary |
|---|---|
| Nineteen affected roots | Offline/locked metadata PASS; no lock drift |
| Third-party preservation | 2,372 exact sourced blocks unchanged; 112 local version fields only |
| Sixteen final archives | Normal offline/locked Cargo package verification PASS; bootstrap collection is not verification |
| Archive correspondence | Normalized manifests and packaged locks differ from retained alpha.2 only by version; 132 current payload files match |
| External packaged consumer | PASS with Auth/session, Config/secret-provider, Redis/tls; all sixteen resolve from verified directory sources without checkout patches |
| Current-version custody CLI/inventory/refusal | Three affected existing cases PASS under compiled alpha.3 |
| Existing rebuild-version fixture | PASS under alpha.3, retaining normal verification and the expected compile-error refusal |
| Optional synthetic custody roundtrip | Not verified: isolated harness changed the recipient digest but omitted the encryption recipient literal; capture/version assertions reached, disposable-key readback correctly failed |

Normal package commands use `--offline --locked --allow-dirty`; final checks do
not use a verification-bypass flag. Sixteen `--no-verify --exclude-lockfile`
bootstrap collections only stage ordinary directory sources for unpublished
siblings. Final verified archives replace bootstrap sources before the consumer.
Dirty local preflight does not waive the later clean-source/tag gate.

The additional existing roundtrip used disposable test identities and an isolated
source copy. The harness updated the recipient SHA/file but omitted capture's
fixed encryption recipient, so it encrypted LOCAL fixture bytes to the unchanged
formal public recipient. Its disposable private key could not decrypt them.
No formal private key was accessed. First failure and diagnostic output remain;
the additional harness is stopped without another attempt. Both test identities
and source snapshots were removed. This is a harness failure, not a new production
crypto defect or a passing roundtrip. The four required changed-version cases,
actual reached serialization assertions and unchanged crypto/recipient code are
separate evidence; formal hosted readback remains mandatory before qualification.
A public pinned-tool download interruption/resumption is also retained; its
archive/binary hashes matched the reviewed pins before any fixture execution.

## Review and execution boundaries

Owning library/test Clippy, format and docs/whitespace PASS. Independent Reviewer
`/root/alpha2_version_independent_review_20261029` PASS0: all 56 source hashes
and 126 evidence files match before/after review, including sixteen archive
hashes, 132 current payloads and 181 final vendor correspondences. Native
reviewer/model/actual high effort are verified. Only completion metadata follows;
acceptance is local delivery readiness, not CI/hosted/crypto qualification. All changed paths,
raw outputs, archive identities and source correspondence are review targets.
Accepted unchanged application/browser/platform/security/load evidence retains
its original sources/limits and is not replayed or relabeled as current proof.

After authorized delivery/CI/Merge, select the actual merged SHA/tree as C and
freeze the core schema-1 record. Before protected readback, actual build
provenance, transport receipt and tag must all agree on source/version/run.
Any mismatch stops before Environment changes/private-key access. The exact-tag
policy rollover and existing Secret are separately rechecked; actual decryption,
complete manifest/member verification and cleanup must succeed before retention
is qualified. Signing and public Release acceptance remain outside this batch.
