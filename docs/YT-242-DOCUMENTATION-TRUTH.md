# YT-242 — Documentation truth audit and bounded corrections

Date: 2026-11-08. State: **Done at bounded documentation scope after independent review**.
[Backlog](BACKLOG.md) and [Roadmap](ROADMAP.md) retain the ordered YT-242 → YT-243
gates. Neither task ID existed at baseline. YT-243 starts only after the separately
authorized YT-242 local strict commit; this acceptance is not release qualification.

## Inventory and exact audit boundary

Baseline `877b6ead49c47d7e02f4f5360522c47453fdf737` has 516 tracked Markdown files,
413 under `docs/` including ADRs and screenshot indexes. The per-path inventory
assigns 101 current-guidance, 315 historical-evidence and 100 proposal-decision
roles. Completed, rejected and deferred proposals remain decisions; Done does
not imply a distributed API. The new report is additional maintainer evidence.
[Documentation Scope](DOCUMENTATION-SCOPE.md#yt-242-inventory-and-semantic-audit-boundary--2026-11-08)
extends the existing classification with exact inspected surface groups.

Semantic inspection covers 54 entry surfaces (53 current guidance plus the MVP
specification), bounded to availability, navigation, edition and source meaning.
It includes root reader/project policies, the index/API/capability maps,
Architecture/Compatibility/Readiness, the two package READMEs and all 18 current
module guides, first-app/browser/View/settings/HTTP guides and the linked
Style/Showcase tool entries. The local handoff retains exact per-path scope,
dispositions, baseline/final hashes, original failed checks and command logs.

The remaining 48 current-guidance files are inventoried, without a full semantic
audit claim. The other 462 baseline files outside the 54-entry set are classified;
selected release-record sections, ADRs and source contracts are additionally read
as comparison facts. No line-by-line audit of all history, every module signature,
code snippet, application/template README or operational guide is claimed.

## Corrections and comparison facts

| Boundary | Authority | Correction / disposition |
|---|---|---|
| Actual distribution | Accepted [SDK publication](ALPHA6-RELEASE-PREPARATION.md#owner-approved-publication-and-registry-consumer--2026-11-01) and [CLI publication](CLI-020-RELEASE-PREPARATION.md#owner-approved-publication-and-installed-consumer--2026-11-02) | Architecture/Readiness identify SDKalpha.6 and CLI0.2.0. AGENTS, security/MVP/threat-model entries, index and affected module/integration guides no longer present old unpublished/nine/sixteen-package preparation as current distribution |
| Selected reader edition | [site/pages.toml](../site/pages.toml) | Compatibility uses SDKalpha.6/CLI0.2.0. Separately pinned reference consumers and dated preparation prose keep their original versions |
| Later typed Views | [tags source](../crates/yetlet/src/view/tags.rs), [YT-234](YT-234-VIEW-AUTHORING-API-PLAN.md), [YT-235](YT-235-TAG-COVERAGE-PLAN.md) | Current checkout wording replaces obsolete feature-branch guidance; 101 typed ordinary tags remain source-only. Published first-app `Element::new` examples stay valid |
| Optional Style authoring | [CLI coordinator](../tools/yetlet-cli/src/styles.rs), [private builder](../tools/yetlet-cli/templates/styles/builder.rs), [contract](YT-237-STYLE-REGISTRATION-CONTRACT.md), [integration](YT-239-STYLE-INTEGRATION.md) | API/Capability/root/index entries make private app generation discoverable and explicitly absent from published CLI0.2.0. Finite vocabulary, ordinary CSS fallback, explicit registration and external build-time CSS; no public style crate or renderer change |
| Showcase | [Sample guide](../examples/style-showcase/README.md), [build source](../examples/style-showcase/build.rs) | Linked as a checkout sample with optional WASM class switching, preserving original bounded Linux/browser evidence and limitations |
| Retained history | Original dated records and historical disclosures | Source IDs, original prose/failures/verdicts remain at their original scope. Ambiguous old policy sections are labeled Historical; no mass version substitution or qualification replay |

Matching development manifest versions do not establish equality with distributed
archives. The API/capability maps now separate published modules from later
checkout authoring/tools. Current guide-only boilerplate and package-carrier
corrections do not certify every remaining example or change package contents.

## Bounded release-document gate

Only `scripts/release_docs.py` and its existing focused tests are extended.
Architecture/Readiness distribution fields derive expectations from the explicit
accepted publication sections, independently of development manifests. Reader
edition prose derives expectations from `site/pages.toml`. Maintained module
introductions reject evidenced obsolete unpublished/candidate boilerplate;
explicit historical headings and delivery disclosures remain permitted. The old
typed-authoring branch reference is rejected in the two current authoring guides.

Existing package-README, installation, expected-binary, status, archive and
pre-packaging gates remain intact. Regression cases exercise mismatches, changed
expected versions, source-only additions and retained historical text. This is
not a new taxonomy engine or proof framework, and cannot establish arbitrary
sentence truth or future registry availability.

## Verification and remaining gates

Exact final command results are retained in the implementation handoff. Required
checks are the focused release-document Python suite, the existing publication
and automation suites affected by the shared source gate, release-doc check,
`cargo xtask check docs`, `cargo xtask check samples` and `git diff --check`.
Original failed checks and bounded repair classifications remain retained.
No fresh SDK compilation, browser/server/CSP run or platform qualification is
claimed; earlier qualification stays at its linked original source/scope.

The focused Python suites pass 14/39/35 tests; release-doc, docs/DAG, thirteen
sample entries and whitespace checks pass. The parent normalized the existing
YT-237–241 prerequisite chain into Backlog table rows without weakening the DAG
checker. The first independent review found one stale WebGPU quickstart rationale;
only that paragraph was corrected. Independent corrective Reviewer
`wc_job_m2SOL6psGzaIGPjm` returned PASS with zero issues or required followups,
verifying all 43 current file hashes and all 42 unchanged prior-review inputs.
The earlier worker partial result and first review failure remain retained.

The parent accepts this bounded documentation/checker work. Local delivery is
separately authorized and must pass strict/builtin Commit Guard with normal hooks.
No source freeze, version bump, candidate selection, Push, hosted action, signing,
publication, Pages activation, installation or cache cleanup is part of YT-242.
YT-243 admission requires the successful local commit; its verification remains
separate.
