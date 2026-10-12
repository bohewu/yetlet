# YT-020 — Private Hosted Rebuild

**Date:** 2026-10-12
**Scope:** One manual GitHub-hosted rebuild of the original nine experimental `0.1.0` source packages, compared with the original Ubuntu checkout's isolated build of the same immutable source.
**State:** Builder prepared; actual execution and final independent review are retained separately. YT-020 stays **Blocked** for release activation.

## Authority and earlier evidence

The maintainer authorized committing the reviewed Rust 1.99 / Private integrity preparation and finishing the Private follow-up. Delivery uses the existing checkout and normal maintainer PR/required-CI path. The selected minimum remains Rust 1.99; the rolling stable and scheduled/manual qualification contracts stay intact. No visibility change, tag, package publication, Release creation or production deployment is authorized.

The earlier [integrity preparation](YT-020-INTEGRITY-PREPARATION.md) was independently reviewed with no issues or followups: native repository reviewer thread `01a12dab-b825-7ba0-a389-0757a5896306`, verified `gpt-6-astra` / effective `high`. It was committed as `9ec9b7b2e2291f649fd659bcbdb8e9fe17f7ad2f`, tree `4266aa1c229c7290558f93093342c664fd6163fd`, and submitted through maintainer PR #6. That commit preserves the earlier Windows/Ubuntu working-overlay evidence; this later builder does not relabel those attempts as immutable-source or independently controlled execution.

First-candidate run `38193149046` passed `Rust 1.99 MSRV` and `Stable native, samples, and Chrome`. The API head is `9ec9b7b2e2291f649fd659bcbdb8e9fe17f7ad2f`; actual runner checkout is synthetic `cdc67bb6a37d9a1abce70a55f0a770e74853366f`, with the same `4266aa1c229c7290558f93093342c664fd6163fd` tree. Only then was protected `Rust 1.88 MSRV` atomically replaced with `Rust 1.99 MSRV`, retaining the Stable check and every other protection, sole-maintainer governance and Private visibility. Later candidate updates must pass their own actual CI before normal merge; the first-head run is not reused as their CI. Exact checkout/source/tree identities, protection snapshots and final merge evidence are retained under ignored `target/yt020-integrity-prep-20261012/delivery/`.

## Bounded builder

[Private rebuild workflow](../.github/workflows/private-rebuild.yml) is manual-only, uses GitHub's `ubuntu-24.04` runner, a 25-minute limit, full-SHA GitHub-owned actions, `contents: read` and no persisted checkout credentials. It installs Rust/Cargo 1.99.0. No Rust cache, primary build outputs, host Docker daemon, maintainer workstation filesystem or signing credential is mounted/copied into the job.

The workflow checks out the immutable `github.sha` captured at dispatch and requires that exact clean source. It creates a Git archive, extracts regular inputs outside the checkout, and materializes the exact 18 candidate license aliases with root license bytes. It records the archive digest and canonical selected-package input hashes with **no working overlays**. Native Rust/Cargo binaries, target, default workspace feature resolution, development verification profile, source-derived `SOURCE_DATE_EPOCH`, four build jobs, workflow/run/ref/source identities and complete commands are recorded.

Each run starts fresh Cargo home/target directories, fetches locked dependencies, then packages and compiles all nine extracted candidates offline using Cargo's temporary local registry. The native target is `x86_64-unknown-linux-gnu`, matching the primary Ubuntu builder. Only the exact nine source archives are selected. No `.git` is present in the package snapshot; archive/VCS/source identities are bound externally by recorded digests, not an invented Cargo VCS marker.

The existing checksum CLI binds the nine archives, source/build records, two build logs and one **unsigned** standard in-toto Statement v1 / SLSA provenance v1 draft. The workflow retains only that bounded private Actions artifact, uniquely named by run/attempt, for seven days. Download and retain the evidence locally before expiry. Artifact upload into this Private repository is preparation, not a crates.io upload or GitHub Release. A failed build/upload is not PASS; retained failure logs remain distinct.

## Execution and comparison

After protected delivery onto `master`, manually dispatch this workflow once and retain the actual source SHA, workflow SHA/ref, run/attempt, job state, exit status and artifact identity. Rebuild that same immutable source locally from its Git archive with the same Rust/Cargo/target/default-feature/profile settings, fresh Cargo home/target, license materialization and source epoch. Keep these build inputs outside the original checkout's Git state. This is a copied source snapshot, not a new Git worktree.

Compare all nine raw source-archive SHA-256 values and sizes plus selected input/lock/tool identities. Inspect actual package/license metadata and provenance/checksum bindings after download. Native executable digests, other platforms, alternate features and browser assets are outside this comparison. Existing sufficient browser/Docker qualification is retained; the new builds close the concrete immutable-input and separately operated execution-environment gaps.

GitHub operates the secondary ephemeral execution environment independently of the primary workstation. The maintainer still controls the reviewed recipe and dispatch; no independent human approver is invented. An unsigned workflow record is not authenticated provenance, a SLSA-level claim, security certification or automatic release approval. Acceptance of this hosted builder for an actual release and its source/tag/digest policy remains explicit under the [supply-chain contract](RELEASE-SUPPLY-CHAIN.md#independent-rebuild-provenance-and-approvals).

## Signing and remaining activation boundary

The workflow has no `id-token`, attestation write, registry write or signing step. It does not send Yetlet identity/source/digests to public Sigstore/Rekor. The previously qualified Cosign binary/trust material and public-fixture verifier results remain useful preparation; they are not signatures for these Yetlet packages.

[GitHub's current availability policy](https://docs.github.com/en/actions/how-tos/secure-your-work/use-artifact-attestations/use-artifact-attestations) requires Enterprise Cloud for Private/internal artifact attestations. This personal repository's observed account plan field is absent; entitlement is **not established**, rather than inferred from a null value. [Private GitHub Sigstore](https://docs.github.com/en/actions/concepts/security/artifact-attestations#how-github-generates-artifact-attestations) also has no transparency log, so it cannot by itself satisfy the currently retained certificate/transparency/inclusion contract. Do not disable verification requirements to report a signing PASS.

Actual trusted signing remains **Not verified** until an approved identity/trust/signing service and exact source/tag/build/digest policy satisfy the existing contract. Changing that contract, buying an account plan, activating a private signing service, disclosing private provenance publicly or publishing requires a separate maintainer decision. This follow-up does not make any of those decisions implicitly.

YT-020 stays Blocked for actual registry name/ownership/resolution, approved immutable release source/tag, selected release-equivalent inventories/licenses, trusted builder/signature/provenance acceptance, reporting/scanner activation, concrete maintainer/security approval and immutable promotion/rollback digests. YT-065/066/067 retain bounded Done states and original evidence identities. Optional IME/assistive-technology/WebKit/mobile/zoom cells remain Not verified and non-blocking for the completed automated scope. No application production SLA or device/accessibility certification is claimed.
