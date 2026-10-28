# YT-096 — 0.9.0-alpha.1 version and package preflight

Date: 2026-10-28 (Asia/Taipei). Status: Local version/package preflight and independent Reviewer PASS, zero
issues/required followups.
D6 Pending; YT-096 In progress; YT-020 Blocked.

## Subsequent guarded commit and source selection

After the local preflight/review, the owner authorizes the exact 59-path commit
and selects resulting `5d7b91842f8188d45e5af22367245aebec7ca7d1`, tree
`6adff26fc4fec65e9aa8f40d7172ff266b7e500d`, as the new immutable alpha candidate C.
Strict/builtin staged Guard and normal native hook PASS, zero findings; exact
committed paths/tree match the reviewed delivery, and the worktree is clean
immediately after commit. No Push occurs. The preflight statements below retain
their original dirty-input/source scope; they are not a new clean-C archive run.

The [minimum hosted/readback plan](YT-096-ALPHA-HOSTED-PLAN.md) separates current
candidate selection from the needed fixed-binding patch and a later explicitly
selected execution source. Existing `v0.1.0` and evidence stay unchanged; D6
Pending. The selection/plan notes are post-commit documentation, not new authority
to Commit/Push, create tags, execute hosted, sign or publish.

## Authority and bounded contract

The owner selects `0.9.0-alpha.1` for all sixteen libraries, synchronized internal
dependency requirements and lockfiles. Only affected packaging/build verification
is authorized. No Commit/Push, real tag creation, hosted execution, certificate
issuance, signing or publication. Existing tags, accepted evidence and failures
must remain intact. This implements the coordinated version discipline in
[Compatibility](COMPATIBILITY.md) and the sixteen-package scope in
[YT-200](YT-200-SIXTEEN-PACKAGE-RELEASE-PLAN.md).

The baseline checkout is `edfb0d68b947b5c571aef94d0af6ac7b942683fb` on
`codex/yt096-hosted-custody`. No new immutable source C is selected by this dirty
preflight. Preserve actual unsigned `v0.1.0` at
`3eacfe31f5e3281411d3bc4653a6baccab668b30`, hosted build `40628943583`/attempt1,
protected verifier `40631789238`/attempt1 and repository-external retained bytes.
Their original package/provenance digests remain valid for those inputs only.

## Version and dependency boundary

All sixteen library versions move together; eleven inherit the root workspace
version and five retain explicit isolated-workspace versions. Internal normal,
optional and dev path dependencies keep their paths, features and target scopes,
with prerelease-aware version requirements `0.9.0-alpha.1`. No third-party version,
source, checksum or dependency record may change. Unpublished workspace examples
inherit the workspace number as before; independently versioned tools, examples
and vendored dependencies keep their own versions. No API, runtime, defaults,
MSRV, dependency algorithm or security contract changes.

Current archive-local README version text follows the new metadata. Dated task
reports, old archives, old manifests inside evidence and old signature proposals
retain their original numbers. Version `0.9` does not claim 90% completeness,
production qualification or stable public APIs; `alpha.1` is a prerelease.

## Minimal affected verification

Use the existing offline/locked Cargo package workflow, with bootstrap archive
collection separated from normal verification. All sixteen final archives need
normal `cargo package` verification because every package identity changes.
Check normalized manifests and sibling requirements, archive-local locks,
licenses/readmes and unchanged source payloads. Exercise one external packaged
consumer with the already selected integrations (Auth session, Config secret
provider and Redis TLS), using version requirements and an ordinary directory
source rather than checkout paths. This is alpha dependency/linkage verification,
not replay of old behavioral tests.

Use focused existing tests only for changed prerelease archive-staging/provenance
seams. Run metadata reads for affected lock roots and documentation/whitespace
checks. Do not replay native functional suites, browser/Wasm/GPU, CSP, Windows,
security scans, load tests, hosted rebuild or custody drills. Those accepted
results retain their original source/scope; this preflight does not relabel them
as exact-alpha candidate qualification.

The new package names in local rebuild/archive checks must use the selected
version. Actual hosted tag/workflow/custody identity guards remain bound to the
previous `v0.1.0` drill. They must refuse a new alpha checkout; do not broaden them
to arbitrary tags or manufacture a new source SHA. Before any later alpha hosted
execution, explicitly select a reviewed immutable C, update and review the
version-specific workflow/readback bindings and obtain the separate execution
and Environment/custody authority. No existing tag or policy is moved here.

## Signing and next gates

The [separate signer contract](YT-096-SIGNER-CONTRACT.md) and
[ADR0009](adr/0009-release-build-and-signer-identities.md) retain their identity,
permission, two-payload and disclosure rules. The previous `0.1.0` provenance SHA
is historical, not an alpha signing target. New alpha archive bytes require new
provenance and a separately frozen complete evidence manifest after an approved
build. New product C, tag, build/readback run, both raw payload hashes and signer
S remain Pending. D1-D5 retain their adopted policies with this explicit change
of intended product version; D6 remains Pending.

## Actual results and review

The local GNU host uses Rust/Cargo 1.99.0 and the existing system OpenSSL input;
this is local package/linkage preflight, not the isolated Ubuntu24/OpenSSL3.5.9
hosted recipe or a new platform qualification.

- All sixteen library identities and eighteen versioned internal normal/optional/
  dev edges resolve to `0.9.0-alpha.1`. Root inheritance also updates twenty-six
  unpublished example identities; sixteen independent non-cohort manifests,
  including standalone tool/example/vendor versions, remain byte-identical.
- All nineteen affected lock roots pass `cargo metadata --offline --locked`.
  Across twenty-one tracked locks, exactly 112 local version fields change in
  nineteen locks. Reversing those fields reproduces baseline bytes. All 2,372
  sourced package records retain exact bytes, including versions, checksums,
  dependency records and sources; no third-party upgrade occurs.
- Offline locked Cargo vendor succeeds. Sixteen bootstrap archives only populate
  ordinary directory sources; `--no-verify --exclude-lockfile` counts as collection,
  never verification. All sixteen subsequent normal `cargo package --offline
  --locked --allow-dirty` commands PASS without either bypass flag. Dirty-tree
  permission is limited to this recorded local preflight, not a release gate.
- Each verified archive has the alpha version, prerelease-aware sibling requirements
  with paths removed, alpha local identities in its packaged lock, required
  licenses/readme and byte-matched current source/test/example/original-manifest
  payloads. Final verified archives replace bootstrap copies in the directory source.
- One external `cargo run --offline --locked --features integrations` PASS prints
  `packaged consumer OK`. Its requirements select all sixteen alpha packages
  without checkout paths/patches, plus Auth session, Config secret-provider and
  Redis TLS. Locked metadata confirms all sixteen resolve from the verified
  directory source. No network/provider runtime or actual registry is exercised.
- Two affected existing tests PASS: real sixteen-package alpha staging/normal
  verification refusal, and exact archive-subject unsigned statement/public
  allowlist generation. No other accepted product test is replayed.

Evidence: `target/yt096-alpha-version-20261028/manifest-verification.json`,
`manifest-metadata-results.json`, `package-command-results.json`,
`package-inventory.json`, `package-completion.json`, `focused-tests.json` and
individual raw command outputs. They preserve source, actual argv/exits, archive
hashes, matched payload paths and external-resolution observations. No evidence
directory is a publication allowlist. No product check failed in this slice.
The initial native child trace lookup used short task names and returned no match;
the canonical returned task paths resolved matching role/model/effort without
redispatch. This observation correction is not a product or verification replay.

Documentation and whitespace checks PASS: `cargo xtask check docs` reports
75 required documents / 207 backlog items; internal links, Markdown structure,
whitespace and dependency DAG PASS. Direct changed-guide links/fences/EOF and
the new-file no-index whitespace check also PASS. No-index exit 1 is the expected
content difference only when whitespace diagnostics are empty. Final immutable source selection, alpha hosted/readback identities,
exact signing payloads, registry consumption and release approval remain open.
No listener, browser, service, cloud allocation, Secret or private-key operation
was started. Inert local archives/logs/caches are retained; no running resource
requires cleanup. D6 remains Pending.


Independent Reviewer `/root/yt096_alpha_version_independent_review_20261028`
returns **PASS**, zero issues/required followups. It verifies all 59 changed-path
hashes and 39 evidence hashes before/after review, all sixteen archive hashes,
132 packaged payload files, eighteen normalized sibling requirements, twenty-one
locks, nineteen metadata roots and all 66 protected files. HEAD/branch/status,
staged-zero, original `v0.1.0` and the safe external retention receipt match.
Native trace verifies the registered reviewer/model and actual high effort;
shared resolver observation is enforced without conflict or degradation.
Review is read-only and replays no build, test, scan, hosted or custody check.

Acceptance is only the local alpha version/package preflight. It grants no
immutable candidate selection, platform requalification, registry proof, new
hosted authority, signature or publication approval. D6 remains Pending. Only
this bounded review-closeout metadata changes after PASS; reviewed manifests,
locks, package payloads, recipes and historical evidence remain unchanged.
