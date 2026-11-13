# Documentation consolidation plan

Status: active maintenance plan. This is a **functional documentation** plan, not another YT task log. Changes to its retention rules or batch scope require independent review.

## Outcome and source of truth

The goal is to **remove redundant numbered YT Markdown files from the current repository**, not merely shorten the Roadmap, rename the files, or move them into an archive directory. The reader should find working capabilities, versions, examples, security limits and compatibility constraints in topic-based guides. Maintainers should find open work and indispensable dependencies in the Backlog, product outcomes and first published versions in the Roadmap, binding architectural decisions in ADRs, and user-visible release changes in existing release notes.

A completed YT ticket is **not** itself a reason to retain a standalone Markdown file. A new or revised batch is incomplete if it only rewrites an index or Roadmap while leaving eligible historical YT documents in place. Do not create a replacement YT-numbered summary or a new directory of relocated historical task files.

## Disposition rules

| Decision | When to use it | Required treatment |
|---|---|---|
| **Merge and delete** | A closed implementation narrative, one-off validation report, execution checkpoint, old Todo, duplicated planning page or status summary | Move only information that is still authoritative into the relevant existing guide, API reference, compatibility document, release note or Backlog dependency. Delete the redundant current file. Use immutable Git history for its original text and anchors |
| **Keep** | A normative ADR, actively referenced technical protocol/contract, standards inventory used for decisions, essential security/compatibility restrictions, actual release record, original validation evidence or still-needed dependency | Record the specific reason; link to the content from the functional guide. A YT-prefixed filename is provisional, not a requirement to keep more task documents forever |
| **Defer** | Removing it would lose a current decision, sever unresolved dependencies, change a public guide's pinned edition, or break historical recovery | State the blocking reference and the later functional batch that owns it. Do not disguise a deferred file as a completed reduction |

Preserve technical meaning and exact support boundaries. In particular, do not equate source availability, package publication, registry resolve/compile, native execution, Wasm compile, browser behavior or live deployment. Keep original failures and unsupported platform/security claims visible via the preserved evidence and authoritative guides. A release note summarizes user-facing changes; it does not duplicate operator journals.

## Reference and deletion gate

Before deleting a candidate, inventory incoming tracked links and anchors, explicitly including Backlog dependencies and later YT records. Consolidate living behavior first. Change current links to functional guides, retained contracts, ADRs and release records. For historical references requiring the old wording or a deep link, use a fixed, existing Git commit **containing the original file**, and verify the path, blob and fragment at that revision. Do not use floating `master` links to a removed path. A historical Git URL is traceability, **not** a replacement current document.

Do not remove formally Done Backlog entries while active dependencies still cite their YT IDs. Retain useful dependencies; first prove the dependency graph is intact. Preserve official releases, source code, CI workflows, original evidence, and the approved Reader Edition manifest. No deletion of caches/evidence or external state is implied by document cleanup.

## Batch contract and measurable acceptance

Every batch must identify:
1. **Baseline number of tracked `docs/YT-*.md` files, candidate set, files explicitly kept/deferred, and target actual deletions.**
2. The surviving functional documentation and the specific decisions/security/compatibility text merged into it; no wholesale copy into a replacement file.
3. Each deleted file, its verified immutable Git recovery identity and repaired incoming links/anchors. Report actual YT file-count reduction and net Markdown file-count change separately.
4. Retained technical contracts, ADRs, inventory, original evidence, release records, Done entries and dependencies; distinguish unchanged historical qualification from new checks.
5. Validation: `cargo xtask check docs`, `python3 scripts/release_docs.py check`, `git diff --check`, tracked link/anchor and dependency checks, source-history recovery, and the existing local Reader build when selected inputs change.
6. **Independent Reviewer PASS on the exact current plan is a blocking gate before any consolidation edits**, including after material changes to its scope, retention decisions or acceptance criteria. A failed or uncertain plan review blocks implementation until the plan is corrected and independently reviewed again to PASS. **A separate independent Reviewer PASS against the final frozen implementation diff** is required before an approved local commit. On approved local commits, read Commit Guard protocol, stage only reviewed files, execute strict/builtin staged Guard, compare its `tree_oid` with `git write-tree`, verify the resulting commit and clean worktree. Never infer Push, publication or deployment approval.

An item counts as **document consolidation** only when redundant YT files are actually deleted; a zero-deletion result is a documented blocker/defer, not a successful cleanup batch. Measure physical files, not just line counts. Any material expansion of the batch requires revising the plan and obtaining a fresh independent PASS before execution.

## M25 corrective batch: YT-219–223 (scope and results)

Baseline: local `master` commit `b9873f15bcfd4c36a2274508ef7517e5a83b807b` has **287 tracked `docs/YT-*.md` files**. The previous pass consolidated Roadmap outcomes and removed the shared `YT-219-224-DOM-STATIC-SITE-PLAN.md`. This corrective batch targets the **six remaining YT-219–223 files**, rather than merely rewriting their index entries.

| Path | Decision | Rationale / destination |
|---|---|---|
| `docs/YT-219-HTML-INVENTORY.md` | **Keep** | Dated WHATWG inventory and Typed Tag Builder selection basis; retain snapshot and original source bounds |
| `docs/YT-222-STATIC-HTML-CONTRACT.md` | **Keep** | Actively referenced, finite static-export protocol, error/ownership/CSP/path contract; keep exact technical authority rather than collapsing it into a superficial CLI synopsis |
| `docs/YT-219-HTML-DOM-CONTRACT.md` | **Merge and delete** | Move still-valid coverage decision, exclusions and parser/DOM compatibility into inventory and the existing browser DOM guide; historical contract stays in Git |
| `docs/YT-220-HTML-DOM-IMPLEMENTATION.md` | **Merge and delete** | Working vocabulary/structure/pre semantics belong in DOM guide; scoped tests and original failures remain recoverable by fixed Git version |
| `docs/YT-221-DOM-URL-IMAGE-CONTRACT.md` | **Merge and delete** | Preserve exact opt-in API, URL preflight, origin isolation, resource and failure/ownership limits in DOM guide; original acceptance evidence stays in Git |
| `docs/YT-223-STATIC-HTML-IMPLEMENTATION.md` | **Merge and delete** | Keep CLI usage, static-export constraints and validation-scope summary in CLI guide and current capability/compatibility references; original execution log and review remain recoverable by Git |

**Physical deletion result:** all four merge-and-delete candidates were removed from the current working tree, leaving the two technical references. This batch reduces tracked YT Markdown files from **287 to 283** when committed, without adding a replacement YT file or moving originals to an archive directory. This durable topic-named plan adds one Markdown file; the net Markdown count falls by **three**. The original four files and their headings are recoverable from fixed source commit `b9873f15bcfd4c36a2274508ef7517e5a83b807b`. Final independent implementation review and guarded Git acceptance remain separate gates.

Use existing `crates/yetlet-browser/guides/dom.md`, `docs/CLI-GUIDE.md`, `docs/API-REFERENCE.md`, `docs/COMPATIBILITY.md`, `docs/CAPABILITIES.md`, `docs/README.md`, `docs/ROADMAP.md`, `docs/BACKLOG.md` and affected example guides as the living destinations. In later historical records such as `docs/YT-227-NEXT-RELEASE-PLAN.md`, replace links to deleted material with fixed Git links while preserving the dated original claims. **Keep all YT-224–226 delivery records and Backlog dependencies, and `site/pages.toml` SDK `0.9.0-alpha.8` / CLI `0.4.0` unchanged.** The first shipped feature outcomes remain SDK `0.9.0-alpha.6` DOM policies and independent CLI `0.2.0` static export; neither is a blanket browser/production qualification.

## Later functional batches (not authorized for implementation by this plan)

| Candidate theme | Inventory / consolidation intent | Boundary |
|---|---|---|
| Typed View / Tag Builder, YT-234–235 | Align shipped SDK alpha.7 with View guides, merge closed API-design/coverage narratives; preserve 101 typed constructors versus default 89 plus opt-in image 90 DOM elements and dated YT-219 inventory | Verify references and delete eligible YT files; no renderer behavior change |
| Reader Edition, YT-224–226 | Consolidate pilot, public build and cutover history into reader/Pages functional guides, live publication records and immutable history | Only after mapping all exact source/edition and hosted evidence references |
| Application-owned CSS, YT-237–241 | Consolidate registration/build/generation/showcase material under CSS/Style usage and limitations | Preserve meaningful contracts, opt-in boundaries and original browser evidence |
| Later CLI/startup and release maintenance | Audit M26 and later YT groups by capability, not simply ascending ticket number | Separate release/CI/evidence constraints; defer without inventing completion |

After each batch, update this plan's **small status table** (file counts, deleted paths, kept exceptions, commit/review outcome and next theme) instead of creating a new YT completion document.

## Completed batches

| Batch | Baseline YT files | Deleted YT files | Result | Evidence |
|---|---:|---:|---|---|
| M25 outcome/index pass, `b9873f1` | 288 | 1 | 287 remained; insufficient physical consolidation | Commit `b9873f15bcfd4c36a2274508ef7517e5a83b807b`; original shared plan recoverable from its fixed pre-delete revision |
| M25 YT-219–223 corrective consolidation | 287 | **4** | **283 remain**; YT-219 inventory and YT-222 static-export protocol kept, four histories restored by fixed links, no archive copy | The Git commit containing this plan; historical files at fixed `b9873f15bcfd4c36a2274508ef7517e5a83b807b`. Final check/reviewer/Guard evidence is retained outside the repository |
