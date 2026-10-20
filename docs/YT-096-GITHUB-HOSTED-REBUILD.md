# YT-096 — GitHub-hosted candidate rebuild

Date: 2026-10-20. Candidate remains
`ce56e0814e5423782440acd2c959ca777d572cd4`, the original nine experimental,
unpublished 0.1.0 libraries, Rust MSRV 1.99 and qualification pin 1.99.0.

**PASS at the separately operated hosted source-archive rebuild scope:** the
existing manual workflow completed successfully and all nine returned archives
are byte-identical to the accepted Linux/Windows/Azure outputs. Independent
ordinary slice review passed with zero issues or required followups. YT-096 remains In progress; YT-020 remains
Blocked. Formal authenticated provenance, trust and final release approval
remain separate gates.

## Authority and exact candidate delivery

The owner explicitly authorized necessary pushes, dispatch of the existing
private-rebuild workflow, same-candidate comparison and independent reviewer
review. Signing and publication were prohibited. This later authority extends
the earlier candidate-selection checkpoint without changing its source.

The current local checkout stays on `feat/yt090-configuration`, HEAD
`a82c4205b7f268b6391fbadc9137b5d94c54d7c6`. No branch switch, detached worktree,
source edit or new commit was needed. The dedicated remote branch is
`codex/yt096-candidate-rebuild-20261020`, pointing exactly to the selected
candidate, not the later qualification-documentation commit.

The initial strict/builtin EMPTY-to-candidate scan blocked on personal/machine
paths in already-published historical commits. That new-ref push was not
attempted. Read-only comparison established that remote master
`25d6a465428c22b15f7f566ad6ddcf266fb52bfe` is an ancestor and the actual 41 new
commits pass strict/builtin scanning: 626 file operations, zero blockers and
78 generic drive/UNC warnings. The dedicated remote ref was first created at
that existing remote master object, without uploading new Git objects, then
normally fast-forwarded to the candidate. The explicit pre-push scan used that
actual remote-old-OID-to-candidate range and passed. No force push, hook bypass,
policy change, audit fallback or history rewrite occurred. The managed hook
remained executable and unchanged; the retained stderr-only label probe does
not independently prove hook execution. The required explicit scan passed.
Master was not updated and no PR or merge was created.

The workflow and builder code are byte-identical to the candidate's existing
[private-rebuild workflow](../.github/workflows/private-rebuild.yml) and
`xtask/src/release/private.rs`. No validation framework was added.

## Actual hosted execution and returned evidence

| Identity or result | Observed value |
| --- | --- |
| Workflow | [Run 39418279958, attempt 1](https://github.com/bohewu/yetlet/actions/runs/39418279958), manual workflow_dispatch |
| Native job | 118612562396; completed, success; approximately 74 seconds |
| Source and workflow SHA | `ce56e0814e5423782440acd2c959ca777d572cd4` |
| Runner | GitHub-hosted Ubuntu 24.04.5 x64; image `ubuntu24`, version `20261018.339.1` |
| Toolchain | Rust/Cargo/rustdoc 1.99.0 GNU; all three binary digests match the accepted primary build |
| Recipe | Fresh private Cargo home/target, upstream locked fetch, normal offline Cargo package verification, default workspace features, dev verification profile, four jobs |
| Source epoch | `1792475772`, unchanged |
| Cargo results | fetch exit 0, package exit 0; native child status 0, no signal, cleanup_ok true |
| Artifact | `private-rebuild-39418279958-1`, ID 12262540945; 15 flat files |
| Artifact ZIP SHA-256 | `afec916ec8545f9c2f45321bbd4267fdcdade11c3e24c97add4deadce4526126`, matches GitHub artifact digest |

The source archive SHA-256 remains
`9096cabe9a7b6d8b809c5bd337c2a8ab88f33ff19b6b4c8b59f31267c7335546`.
All 63 selected input hashes match the primary record; no overlays exist.
The selected-input digest remains
`c10830cd2c843c9e9e0f887b37ab0b985c9e396198263610306599bf03910d4d`.
The root lock, source epoch, GNU tool digests and recorded defaults/profile
match. Raw package logs contain all nine Packaging and Verifying steps with
normal verification enabled. The returned complete bytes, sizes, SHA-256 values
and tar member names match every original archive in the
[nine-archive comparison table](YT-096-TWO-HOST-REBUILD.md#archive-comparison).

The existing command
`cargo xtask release checksums verify artifacts/yt096-github-hosted-rebuild-20261020/returned`
passed. The unsigned in-toto Statement v1 / SLSA provenance v1 draft binds the
nine actual archive subjects, source commit/archive/lock, workflow/run/attempt
and source/build-record byproducts. These bindings were checked against the
downloaded bytes; no signature, authenticated attestation or SLSA-level claim
is implied. The GitHub artifact digest verifies transfer integrity, not formal
release trust. This slice adds no compiled-binary reproducibility claim or
new SBOM, native test, wasm or browser run.

Raw API run/job/artifact records, downloaded artifact/log ZIPs, extracted files,
comparison and prior-evidence baseline are retained under
`artifacts/yt096-github-hosted-rebuild-20261020/`. All 659 preexisting evidence
files and 63 selected source inputs remain unchanged. The private Actions
artifact expires on 2026-10-27; its bytes and logs were saved locally before
expiry. The hosted job and post-checkout cleanup completed; no application,
listener, self-hosted runner or new Azure resource was started. The candidate
branch remains as an evidence reference. Existing required CI and repository
visibility/protection settings were not changed.

The run retains a nonblocking upload-artifact Node.js 20 deprecation annotation;
GitHub ran that pinned action with Node.js 24 and upload succeeded. This slice
does not change the reviewed workflow to address that advisory.

## Acceptance boundary and API stability

This supplies the selected candidate's archive comparison from a GitHub-operated
ephemeral execution environment, separately operated from the primary desktop
and manually controlled Azure VM. The maintainer still controls the recipe and
dispatch; no independent human approver or builder-authenticated statement is
invented. Final builder/source/tag/trust-policy acceptance, formal provenance,
signature/registry verification, remaining inventory/platform reconciliation,
independent final security/release review and maintainer release approval remain
open under the existing [supply-chain policy](RELEASE-SUPPLY-CHAIN.md#independent-rebuild-provenance-and-approvals).
No signing, tag, Release, registry upload or production deployment occurred.

This qualification does not promote API classifications. The existing
[API inventory](API-CONTRACTS.md) labels core View/bindings, HTML rendering,
selected Axum application/middleware and bounded memory-session boundaries as
Stable Candidates, not downstream compatibility guarantees. Actions, DOM/keyed
DOM, reactive including GraphHandle::snapshot, page convenience and subsequent
opt-in providers retain their Experimental classifications. There is no
released supported version. The selected [experimental compatibility policy](COMPATIBILITY.md)
requires future 0.1.x patches to preserve documented API behavior and MSRV;
incompatible changes require a coordinated minor release with migration notes.
Neither successful rebuilding nor publication alone promises 1.0 stability.

## Independent ordinary slice review

Reviewer `/root/yt096_github_hosted_rebuild_review_oct20` returned PASS with
zero issues, required followups or optional notes. Native trace verification
matched registered role reviewer, model gpt-6-astra and effective high effort;
the shared reasoning resolver observation is enforced. The
[exact verdict](../artifacts/yt096-github-hosted-rebuild-20261020/REVIEW-RESULT.json)
and [bounded acceptance](../artifacts/yt096-github-hosted-rebuild-20261020/REVIEW-ACCEPTANCE.json)
accept this hosted qualification slice. Final edits record the verdict and
final documentation/preservation checks only. Formal trust and final
security/release approval remain open.
