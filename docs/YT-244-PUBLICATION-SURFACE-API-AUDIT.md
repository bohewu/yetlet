# YT-244 — Publication surface and recent API consistency audit

Date: 2026-11-08. Status: **Done at bounded local source/documentation scope after independent Reviewer PASS**.
Scope: one bounded source/documentation release-readiness correction on the current
`feature/style-css-roadmap` checkout after YT-243. Windows validation is
**explicitly deferred** by the owner in this turn. No new version, candidate C,
push, merge, release, hosted CI or production claim is selected.

## Package-carried publication inventory

Existing manifests select exactly three separately published packages: `yetlet`,
`yetlet-browser` and `yetlet-cli`. The following counts come from normal
`cargo package --list --locked --offline --allow-dirty` local inventory of
the dirty source checkout, not a release archive or registry publication.

| Package | Local listed paths | Included reader surfaces |
|---|---:|---|
| `yetlet` | 84 | `README.md`, 13 library guides |
| `yetlet-browser` | 24 | `README.md`, 4 browser guides |
| `yetlet-cli` | 156 | `README.md`, copied templates and private generator resources |

Before this change, 165 inline Markdown references crossed their owning
package boundary: 94 from `yetlet`, 38 from `yetlet-browser`, and 33 from
`yetlet-cli`; all resolved in the full checkout, but not as package-local files.
Those 165 references now select public GitHub source URLs. The three
package READMEs also replace 25 other pre-existing package-local relative
links with public URLs. In all, 190 existing Markdown references were
rewritten and one newly introduced YT-243 evidence link is absolute. The source URL base
`github.com/bohewu/yetlet/blob/master` deliberately identifies **moving
maintainer documentation**, not immutable versioned package evidence.
Versioned SDK and CLI consumers must use the exact release tag/archives/rustdoc.
Package-local links inside guides remain relative only when their target
is carried by that same package.

The SDK/package READMEs now put exact published registry dependency examples
before explicit checkout-only `path` recipes, while retaining Rust 1.99
and native/Browser feature boundaries. The CLI README separates the commands
of published `yetlet-cli` 0.2.0 from subsequent Style variants present only
in this source checkout. Its source-only YT-239/243 paragraph describes
the actual bounded Linux/local Windows evidence, not an unreviewed working tree.

## Targeted release gate and limitations

`scripts/release_docs.py` retains its existing version- and record-derived
release checks and now checks absolute/public links in all three package README
bodies (including the archive README readback). It additionally rejects relative
module-guide links escaping the containing package or naming missing files.
The CLI published-inventory guard rejects source-only Style template names
while the development CLI manifest is still at the already published version.
`scripts/test_release_docs.py` covers these exact failure modes; no independent
documentation validator, new dependency, content rewriting framework, or API
migration has been introduced.

These checks cover inline Markdown links and source-local file membership;
they do not verify every reference-style/HTML link, external HTTP availability,
GitHub Release Notes text, hosted docs.rs rendering, or actual future `.crate`
archive bytes. Existing package/README signing and frozen-source release gates
remain mandatory. GitHub Release Notes are separate selected `notes.md` inputs
under the existing reviewed publication workflow, not inferred from README.

## API consistency disposition

| Current addition | Existing policy | Audit disposition |
|---|---|---|
| `yetlet::view::tags` / typed `TagElement<T>` | Thin consuming authoring facade on original owned `Element`/`View`; typed void children and scoped attributes | No observed divergence from Rust/YT-234/235 authoring contracts; no new renderer or runtime policy |
| `.id`, `.class`, `.add_class` and generic `.attribute` | Named setter replaces, class merge deduplicates, generic attributes append | Intentional and documented distinction; duplicate generic attributes may still be rejected by native/DOM renderers |
| Private Style CSS generator | App-owned explicit registration and external build-time CSS; ordinary CSS default | Does not expose a public style crate, new required feature or implicit View traversal |
| `new --template native-style / browser-style` | Additive CLI template selectors, separate from user-facing settings; not shipped in CLI 0.2.0 | Current checkout works by source-bound evidence; published command inventory no longer falsely advertises them |
| Host/CSP/platform boundary | Native SSR without JS, opt-in WASM, strict external CSS, no request-time compiler | No newly identified contract break in the reviewed surfaces; see YT-243 for bounded verification |

The audit compares these named APIs and their immediate documentation/CLI
contracts. It is **not** a blanket source compatibility certification of every
public symbol in both crates or a full semver check against downloaded crates.
No product/API code was changed in this task.

## Remaining release decisions

Select the next SDK/CLI version(s) and actual included feature cohort separately.
Update the selected package READMEs, SDK examples, CLI pins, template/help output,
per-version reference and release notes only after those selections; keep the
public guide edition independently pinned until accepted. Run normal clean-C
package verification, exact archive README readback, hosted CI, signing and
public registry/doc readback under their existing independent authorization gates.
Do not republish different bytes under SDK alpha.6 or CLI 0.2.0.

Windows follow-up is on hold by owner request; prior YT-243 local MSVC evidence
retains its original scope and has not been replayed. macOS/runtime, hosted
Windows image, Firefox 390, Safari/mobile and any other previously listed
platform gaps are not promoted. Review the original YT-243 acceptance before
deciding which platform claims the next Experimental Alpha will make.

Retained local checks: 96 release-document/publication/signing regression tests
(1 existing skip), 17 focused documentation cases after the corrective change,
release-doc gate, docs/DAG (75 documents, 241 backlog items), 13-sample catalog,
package inclusion counts, and whitespace PASS. The initial independent Reviewer
`wc_job_iR263iW3SNnfeUPT` found one P2: renamed CLI headings dropped two
previously linked fragment IDs. Two explicit GitHub-compatible legacy anchors
now preserve those old links without reverting the more accurate headings.
The independent corrective Reviewer `wc_job_qL8yujng6i4r08m9` returned PASS,
zero issues and required followups; 26 untouched reviewed source hashes matched.

Detailed commands, original failure, the corrective source delta, and review
outputs remain under ignored `artifacts/verification/yt244-publication-surface-20261108/`.
The local source/doc task is accepted at the stated scope; Git delivery still
requires separately authorized strict/builtin Commit Guard and normal hooks.
No Windows requalification, new candidate/version, Push, Merge, hosted check,
release, registry upload, docs.rs live render or cache deletion occurred.
