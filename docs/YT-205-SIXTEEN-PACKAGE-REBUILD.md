# YT-205 — Minimal sixteen-package rebuild extension

Date: 2026-10-27. Status: **Implemented and accepted at local preparation scope after independent Reviewer PASS.**

**Later commit and candidate selection (2026-10-27):** The owner authorizes
strict/builtin Commit of the ten accepted paths, without Push, and selects the
resulting `118f3d0f496b487ef20fe243f4e73179f9eadc43` as new technical C. Explicit
Guard/native hook pass with zero findings and the committed tree matches
`07faa3e90c0fca5d27a4e55262c75acf851d884a`/`tree_oid`. The
[remaining verification order](YT-205-RELEASE-EVIDENCE-CONSOLIDATION.md#new-candidate-and-remaining-verification-order--2026-10-27)
supersedes only the earlier source-selection/uncommitted status below. The local
review and historical results stay intact. The [later authorized execution](YT-205-RELEASE-EVIDENCE-CONSOLIDATION.md#authorized-current-c-execution--2026-10-27)
now records required CI, sixteen byte-identical hosted/local archives and affected
MSVC PASS with final independent Reviewer PASS, zero issues/required followups.
Whole YT-205 is Done at selected technical/reader scope. Source C was Pushed; no Merge
or signing/publication occurs and D1–D6 stay Pending.

## Authority and preserved source boundary

The owner authorizes strict/builtin Commit of the ten accepted YT-205 documents,
without Push, then a minimal extension of the existing rebuild to sixteen
packages, only missing verification and independent Reviewer acceptance.
The documentation commit is `012ecb3521fb7f0bd2b4d33d9798765cb1b9f281`, tree
`3f66942f31ed8caa508cfd5d3e98306880f8781c`: ten files, explicit Guard/native hook
PASS, zero findings, matching `tree_oid` and clean checkout afterward. No Push.

The separately selected technical candidate remains a2d6474. The new builder and
documentation are an uncommitted overlay, not that candidate or a new immutable
source selection. Keep the [earlier YT-205 packet](YT-205-RELEASE-EVIDENCE-CONSOLIDATION.md),
all historical PASS/FAIL results and the original nine-package recipe recoverable.
D1–D6 remain Pending; no signing, registry, release activation, new-change
Commit/Push/Merge or hosted workflow dispatch is authorized.

## Minimum behavior

- Keep `cargo xtask release private-rebuild` on the original nine packages.
  Add the explicit closed selection `--cohort original-nine|sixteen`; invalid,
  missing or extra arguments fail before execution. A typed Rust cohort carries
  the selection. No arbitrary package list, general builder or new dependency.
- Add the same two-choice manual workflow input, defaulting to original-nine.
  Pass the input through a quoted environment variable; retain manual-only
  triggering, read-only contents permission, pinned actions, immutable clean
  source/workflow equality, exclusive output creation and failure propagation.
- Preserve independent workspaces. Sixteen mode covers the root workspace and
  Config, Secrets, Signing, Redis Session and wgpu manifests/locks. Record every
  selected lock and all sixteen package inputs; the root lock alone is insufficient.
- Use ordinary Cargo vendor with all six manifests and fresh Cargo home/target
  directories. Collect each package's normalized bootstrap archive with
  `--no-verify --exclude-lockfile` solely for the existing directory-source
  convention. Materialize standard per-file checksums with null unpublished
  registry checksum. Bootstrap collection is never a verification PASS.
- Verify all sixteen through ordinary offline locked `cargo package`, without
  `--no-verify` or lock exclusion, at default package features. Keep bootstrap
  output separate from final archives. Reject missing/extra archive names and
  selected source/lock changes; include isolated roots without merging workspaces.
- Retain the original eighteen exact license aliases for the original nine.
  The additional seven have regular license files; do not loosen the source
  symlink/type checks or manufacture more aliases.
- Sixteen mode needs Signing's existing OpenSSL3.5.9 native prerequisite. The
  manual workflow may prepare that checksum-pinned static recipe only when
  sixteen is selected, under its owned temporary directory. Record the actual
  version/binary identity. Original-nine mode does not install this prerequisite.
- Record selected cohort, manifest/lock identities, commands and native outcomes;
  retain failed steps. Extend the existing preparation-only unsigned provenance
  to exactly sixteen subjects and all selected lock dependencies. No new trust
  policy, authenticated-provenance claim, signer, public metadata disclosure,
  package upload or source/workflow approval is introduced.

## Bounded verification and completion

The existing tool is the product surface in this task. Add focused tests for
closed CLI selection, original-nine compatibility, independent-manifest/input
coverage, directory-source staging and normal verification of unpublished
cross-workspace packages. Use a tiny dependency-free sixteen-package fixture
with relevant optional/dev cross-workspace edges, not copies or reruns of the
already accepted real framework suites. Do not impersonate GitHub or weaken the
manual clean-source guards to run a dirty overlay locally.

Run the new focused tests and affected legacy alias control, relevant xtask
compile/lint/format checks, workflow structure/permission/input checks and
repository documentation/whitespace checks. Preserve failures and classify
product, harness or operational causes before any bounded correction.
Independent read-only review must verify the actual patch, tests and retained
evidence before reporting this extension accepted.

Actual GitHub execution and exact-source sixteen-archive rebuild comparison
remain Not run until the reviewed tool is committed, delivered and authorized
at an appropriate immutable source. Fixture success is mechanism evidence,
not current-candidate CI, package/platform qualification or builder trust.
The remaining [YT-205 acceptance conditions](YT-205-RELEASE-EVIDENCE-CONSOLIDATION.md#open-acceptance-conditions-and-next-step)
therefore keep the whole item In progress. Record results and exact-owned
temporary-resource cleanup at closeout; new changes remain unstaged.

## Implemented scope and actual checks

The change is confined to the existing xtask parser/entry point/private rebuild,
its focused product tests, manual workflow and task documentation. No framework
runtime source, crate manifest, dependency, lockfile, trust policy or hook changes.

The original-nine branch retains its two Cargo commands, eighteen alias check,
source/workflow/manual guards, fresh homes/targets, selected inputs and unsigned
record shape. The new typed `Cohort::Sixteen` selects all sixteen names and six
manifest roots. Its fixed33 Cargo steps are one locked vendor operation with
five sync manifests, sixteen unverified bootstrap collections and sixteen normal
offline locked verifications. GNU tar extracts only the Cargo-produced bootstrap
archives into the owned directory source; regular-file checks and standard file
checksums preserve the existing source convention. Bootstrap staging receipts
explicitly say verification:false; failed native commands/staging stop the build.

Sixteen-mode records include all six lock hashes and default_package_features,
with sixteen exact final subjects; the provenance includes the five additional
lock dependencies. It stays preparation_only:true. The workflow's conditional
OpenSSL source/Configure/make recipe and checksum match retained3.5.9 inputs;
the builder records the actual version and binary hash and requires that recipe.

| New or affected verification | Actual outcome |
|---|---|
| Native xtask lib/bin check, Rust1.99 offline/locked | PASS |
| New private_rebuild product tests | 4 PASS,0 failed/ignored |
| Affected existing alias/path control only | 1 PASS; unrelated tests filtered |
| Focused xtask Clippy with warnings denied | PASS |
| xtask formatting | PASS |
| Workflow YAML/choice/permissions/action pins/timeout | PASS; existing permissions/actions/25minute bound retained |
| All workflow run blocks, bash syntax only | PASS |
| OpenSSL pin versus retained source bytes | PASS; no new download/build |

Commands are retained with exact argv/native exits in the evidence directory:

```text
cargo +1.99.0 check --offline --locked --manifest-path xtask/Cargo.toml --lib --bin yetlet-xtask
cargo +1.99.0 test --offline --locked --manifest-path xtask/Cargo.toml --test private_rebuild -- --nocapture
cargo +1.99.0 test --offline --locked --manifest-path xtask/Cargo.toml --test measurements private_exact_package_alias_allowlist_and_paths -- --exact
cargo +1.99.0 clippy --offline --locked --manifest-path xtask/Cargo.toml --lib --bin yetlet-xtask --test private_rebuild -- -D warnings
cargo fmt --manifest-path xtask/Cargo.toml -- --check
```

The Linux fixture uses the actual pinned Cargo, eleven workspace members and
five independent workspaces, unpublished normal/optional/dev edges including
the Config/Secrets relation, fresh owned Cargo state, and a path with spaces.
It executes all33 new steps, verifies the exact sixteen archive names and all
normal verification outcomes, and proves source/lock hashes stay unchanged.
It then changes only one fixture to contain compile_error! and runs that single
normal package step: expected exit101 and `failed to verify package tarball` are
retained in raw test stderr. This is an intentional negative control, not an
unexpected product failure. The successful full pipeline is not replayed.

All temporary fixtures are TempDir-owned; the named full-pipeline directory is
checked absent after the test. No container, service, cloud resource or signing
credential is created. Actual framework package/workload/browser/GPU/native
platform tests and accepted OpenSSL qualification are not replayed.

The complete manual workflow, its OpenSSL preparation step and hosted builder
envelope have **not run**. No GitHub identity was forged and no local dirty-tree
execution mode was added. Real six-workspace dependency fetching, current-source
hosted rebuild/comparison and CI remain future admitted execution; fixture and
static workflow checks do not claim those results.

Evidence: `target/yetlet-yt205-rebuild-20261027-ew2fymd5/evidence/` contains
commit-paths/Guard/receipt, focused-tests, verification and workflow-checks
records. The previous YT-205 source/evidence bundles and all historical verdicts
remain untouched.

## Independent acceptance and remaining gate

Reviewer `yt205_sixteen_rebuild_extension_independent_review_20261027` returns
**PASS, zero issues and zero required followups** for this bounded extension.
It verifies all10 candidate/28 evidence/1181 protected content hashes and the
unchanged18 symlink targets, actual patch/CLI/source guards, six manifest/lock
roots, bootstrap versus normal verification, archive/provenance handling,
workflow/OpenSSL pin and permissions, five passing focused controls and cleanup.
Read-only review performs no additional builds, tests, network or workflow runs.

Native trace verifies the registered reviewer model gpt-6-astra and effective
effort high; the shared resolver reports enforced without conflict/degradation.
Reviewer input/result and filtered runtime observation remain in the evidence
directory. Reviewed code/tests/workflow and evidence stay unchanged; only this
closeout and task-navigation metadata are amended after acceptance.

The reviewer explicitly confirms whole_YT205_can_be_done:false. Current-source
hosted CI, actual sixteen-package rebuild/comparison, MSVC dependency evidence
and final reader/package source disposition remain open. This extension prepares
the existing mechanism, not a fresh candidate or release approval. YT-205 stays
In progress, D1–D6 remain Pending and YT-020 stays Blocked. New implementation
and documentation remain unstaged/uncommitted; no Push, Merge or dispatch occurs.
