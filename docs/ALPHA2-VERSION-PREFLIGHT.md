# Alpha.2 — Coordinated version and package preflight

Date: 2026-10-29 (Asia/Taipei). Status: Local preflight and independent Reviewer
PASS, zero issues/required followups. This is the YT-096/YT-209 new-version follow-up, not a new release
approval or a reopening of the accepted alpha.1 D6 result.
Subsequent PR22 integration CI failed at a synthetic rebuild fixture; the
original local acceptance remains scoped below. Merge is blocked pending repair
delivery and successful required CI.

## Authority and source

The owner approved synchronizing all sixteen libraries to `0.9.0-alpha.2`,
internal dependency requirements, affected locks and outdated current documents,
with no third-party upgrade. Only affected package/external-consumer checks are
selected; accepted behavior/platform/security evidence is retained.

The clean baseline is merged master
`c2d4bd085682cf86748bdd3ff18d6ca9f26534fc`, tree
`32ad5a974f8de1cd405bbc96fa37c731bbb7d9fd`. Both required jobs in automatic
[master CI40884588178/1](https://github.com/bohewu/yetlet/actions/runs/40884588178)
PASS before this migration. PR21's original CI40883138343/1 and squash/tree
readback remain recorded. This uncommitted preflight does not select a new C,
S or V, create a tag, execute hosted release work, sign or publish.

The published baseline remains Experimental
[v0.9.0-alpha.1](https://github.com/bohewu/yetlet/releases/tag/v0.9.0-alpha.1)
at `bf3363801132f4481263ceb411d9bba3349df86d`, with its accepted signatures,
D6 decision and public assets unchanged. YT-020 remains Blocked.

## Package and compatibility contract

Eleven library versions inherit the root; five retain explicit isolated-workspace
versions. All sixteen and eighteen versioned normal/optional/dev sibling edges
move to alpha.2. Paths, optional features, targets, API/runtime behavior,
Rust1.99 and the coordinated Experimental compatibility policy are unchanged.
Inherited versions also move twenty-six unpublished workspace examples; those
examples do not join the sixteen-library distribution cohort. Independent
tool/example/vendor versions remain unchanged.

Thirteen current package READMEs update usage requirements. Nineteen locks change
only 112 unsourced local version fields. Reversing alpha.2 to alpha.1 reproduces
the original bytes of every changed manifest, package README and lock. All 2,372
sourced package blocks across the twenty-one tracked locks retain exact bytes,
including third-party names, versions, sources, checksums and dependency records.
No dependency upgrade, ignore, feature expansion or registry claim is introduced.

The current compatibility/release guides distinguish the published alpha.1 from
the alpha.2 checkout and record completed YT-209 Environment installation versus
Pending positive schema-1 hosted qualification. The hosted plan gains a current
disposition while retaining its dated pre-installation inventory/proposals.
Historical candidate decisions, failures, original qualification, old archive
names and signed payloads retain their original version/source identities.

## Affected verification and actual results

The existing alpha.1 local package recipe and external consumer are reused with
new output/version inputs. Rust/Cargo1.99.0, four build jobs and the local GNU
host/system OpenSSL are used. This is local packaging/linkage preflight, not an
Ubuntu24/OpenSSL3.5.9 hosted build or new Windows/GPU qualification.

| Check | Result and limit |
|---|---|
| Nineteen affected manifest roots | `cargo +1.99.0 metadata --offline --locked --format-version 1` PASS; no lock drift |
| Version/dependency correspondence | Sixteen versions, eighteen internal requirements and 112 local lock fields match; all third-party blocks unchanged |
| Sixteen new-version archives | Normal `cargo package --offline --locked --allow-dirty` verification PASS; no verification-bypass flag on final commands |
| Archive input correspondence | All sixteen normalized manifests and packaged locks differ from retained alpha.1 archives only by the local version; 132 current source/test/example/README/license/original-manifest payload files match |
| External packaged consumer | `cargo run --offline --locked --features integrations` PASS, prints `packaged consumer OK`; Auth/session, Config/secret-provider and Redis/tls compile/integrate through verified directory sources |
| Consumer dependency resolution | Locked metadata resolves all sixteen alpha.2 libraries from final verified directory-source packages, with no checkout paths/patches |

Sixteen `--no-verify --exclude-lockfile` bootstrap collections populate ordinary
directory sources for unpublished siblings; they are not verification PASS.
Final verified archives replace those bootstrap sources before the external
consumer executes. `--allow-dirty` applies only to this uncommitted local
preflight; it does not waive the clean-source release gate. The consumer's
program is byte-identical to the accepted alpha.1 consumer. No live Redis,
Session service, cloud resource, network provider or registry is exercised.

Evidence is retained at `target/yt096-alpha2-version-20261029/` (local, ignored):
master CI receipt, baseline hashes, migration/metadata records,
`manifest-lock-verification.json`, `package-command-results.json`,
`package-inventory.json`, `archive-delta.json`, `package-completion.json` and
original command outputs. The thirty-five package workflow commands include
collection, normal verification, consumer execution and metadata; they are not
thirty-five behavioral tests. No affected package/consumer check failed.

Accepted YT-202/203/204/205, alpha.1 version preflight and YT-209 implementation
evidence retain their exact original sources/inputs/limits. Runtime and release
tool implementation files are unchanged. No native functional suite, browser,
WASM/GPU, CSP, MSVC, security scan, load test, hosted build, custody drill or
unchanged release-tool test is replayed. Old advisory evidence retains its DB
date; this migration does not call it a fresh scan.

## Review, remaining gates and next step

`cargo xtask check docs` PASS: 75 required documents, 209 backlog items, links,
Markdown structure, whitespace and dependency DAG. Changed/new Markdown link
and whitespace inspection also PASS. Independent Reviewer
`/root/alpha2_version_independent_review_20261029` PASS, zero issues/required
followups. All 53 source and 125 evidence hashes matched before/after review;
sixteen archive hashes, 132 current payload and 181 final vendor correspondences
were verified read-only. Native reviewer/model/actual high effort are verified.
Only review-completion metadata follows this frozen source boundary.
Review targets are the actual version/document diff,
original commands/archives, unchanged-dependency evidence and preserved source
boundary; a local PASS does not qualify new hosted output.

The next Git action requires separate authority. After delivery/required CI,
select exact C before any separately approved tag/build/readback. The new
schema-1 prepare/sign/result/public sequence still needs actual new record,
run/artifact/payload hashes, concrete owner approvals, cleanup and retained
outputs. It cannot reuse/edit old receipts lacking `record_sha256`.

The three existing Environments and age identity are not changed or read here.
Future exact evidence-tag policy rollover is separate from current version
editing. Alpha.1 D6 stays Complete; future signing/publication is not authorized,
and no local private-key exception is renewed. No Commit/Push, tag, hosted
dispatch, signing or publication occurs in this slice.

## Subsequent PR22 integration failure and local correction

The owner authorized strict/builtin Commit of the reviewed 53 paths as
`dbe1fbbb139fe3d7814ec8188a119a365286464c`, then Push and PR22. The staged and
exact outgoing range/native hook checks PASS. Original
[CI40888941346/1](https://github.com/bohewu/yetlet/actions/runs/40888941346)
passed MSRV but failed Stable's Rust-tool tests before audit/browser execution.
No Merge or automatic retry followed. Earlier archive/consumer PASS results
remain valid at their local preflight scope; they are not a current CI PASS.

The failed existing case is
`private_rebuild::real_cargo_sixteen_alpha_packages_and_normal_verification_refusal`.
Its synthetic manifests, internal requirements and expected archive names were
still literal alpha.1; the reviewed recipe derives alpha.2 from its compiled
root manifest. Cargo collected an alpha.1 fixture archive while staging correctly
looked for alpha.2. This version-sensitive test consumer was missed in the
initial migration/verification selection. It is a fixture mismatch, not a failed
real sixteen-library package check.

The minimal correction changes only `xtask/tests/private_rebuild.rs`: synthetic
versions, dependency requirements, filenames and manifest assertions read the
same compiled workspace-version authority. No public helper, runtime, recipe,
historical trust pin, feature or dependency is changed. Normal verification and
the intentional compile-error refusal remain required. An inventory of the other
alpha.1 literals identifies retained legacy custody pins and deliberate negative
or self-contained synthetic inputs, not additional current-version fixtures.

The one failed case passes locally with `--offline --locked`, including all
sixteen synthetic archives and its expected negative verification refusal.
Owning-test Clippy with `-D warnings` and format PASS. No already accepted package,
consumer, browser or platform check is replayed. Original CI metadata/full log
and focused repair output are retained at `target/yt096-alpha2-pr22-20261029/`.
The expected compile-error output is the passing negative assertion, not a new
product failure. Independent repair Reviewer
`/root/alpha2_version_independent_review_20261029` PASS0; all four source hashes
match before/after read-only review, and the negative block remains byte-identical.
Native reviewer/model/actual high effort are verified; only completion metadata
follows the review. New Commit/Push/CI
delivery requires the applicable authority; conditional Merge remains blocked.
