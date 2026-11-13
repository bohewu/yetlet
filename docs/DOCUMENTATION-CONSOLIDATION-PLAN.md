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

## Current batch: Typed Tag Builder authoring (YT-234–235)

Authorization: user requested continuation after the completed M25 commit. Baseline: local `master` at `9a6f954dc38f3b9e854fce5a08c9b843182657f8` with a clean index and **283 tracked `docs/YT-*.md` files**. This is documentation consolidation only, not a new Rust authoring or renderer feature. An independent Reviewer must **PASS this exact batch plan before editing any other file**. If the plan fails or the scope changes materially, correct and re-review it first.

| Current YT file | Decision | Living destination / historical retention |
|---|---|---|
| `docs/YT-234-VIEW-AUTHORING-API-PLAN.md` | **Merge and delete** | Fold selected builder/class/id/child/void and compatibility semantics into `docs/VIEW-COMPONENTS.md`. Keep `Element::new` and append-only `Element::attribute` behavior; preserve the original 2026-11-07 source-only design/implementation reviewer evidence via immutable Git history |
| `docs/YT-235-TAG-COVERAGE-PLAN.md` | **Merge and delete** | Put the 101-constructor coverage and exact typed setters, void restrictions, SSR-only exceptions and renderer policies in `docs/VIEW-COMPONENTS.md` and the existing API/compatibility maps. Keep the original scoped tests/reviews at their immutable Git identity rather than copying the operational log |
| `docs/YT-219-HTML-INVENTORY.md` | **Keep unchanged** | Dated standards inventory and current reference for Typed Tag Builder inclusion/exclusion, already retained by M25 |
| `docs/ALPHA7-HOSTED-RECORD.md` and later official release records | **Keep unchanged** | SDK `0.9.0-alpha.7` is the first accepted publication of `yetlet::view::tags` (101 constructors); registry resolve/compile proof is not new browser/platform certification. CLI is a separate product |

**Physical deletion result:** both redundant YT task documents were removed, not relocated or re-created. Tracked `docs/YT-*.md` count **283 → 281**, and repository Markdown-file count decreases by **two** without new Markdown files. Update `docs/ROADMAP.md` and `docs/BACKLOG.md` for the now-published outcome while preserving formal Done entries and all original dependency edges. Repair every incoming local or floating `master` reference in `docs/API-REFERENCE.md`, `docs/VIEW-COMPONENTS.md`, `docs/COMPATIBILITY.md`, `docs/YT-242-DOCUMENTATION-TRUTH.md`, Roadmap and Backlog. Current usage must point to the View guide, exact Rust source, retained inventory or actual release record; historically dated review/selection links may point to the exact existing Git commit `9a6f954dc38f3b9e854fce5a08c9b843182657f8` with original paths/anchors verified. Do not remove or rewrite original release/verification records, ADRs, source/CI, or YT-224–226 Reader delivery; keep `site/pages.toml` SDK `0.9.0-alpha.8` / CLI `0.4.0` unchanged.

Checks: `cargo xtask check docs`, `python3 scripts/release_docs.py check`, `git diff --check`, missing-link/anchor and Backlog DAG scan, historical Git object/fragment recovery and the existing local Reader build if changed inputs require it. Independent **final frozen-diff Reviewer PASS**, then the prescribed strict/builtin staged Commit Guard with identical tree OID before one owner-authorized **local commit only**; no Push, Release, Deploy or cache cleanup. Report the exact deleted paths and remaining file count, not just line reduction.

## Current batch: Reader Edition and Pages cutover (YT-224–226)

Authorization: owner-directed continuation after the completed Typed Tag Builder batch, with local `master` at `f03e8886bc1dbe28f135e05e527b69e01eb3deba` and **281 tracked `docs/YT-*.md` files**. Only this plan may be revised before the required independent **plan Reviewer PASS**. No Reader/export/deployment source or live Pages action is authorized.

| Candidate YT document | Decision | Preserved functional source and recovery |
|---|---|---|
| `docs/YT-224-DOCS-PILOT.md` | **Merge and delete** | Preserve the selected eight-chapter parser/escaping/explicit Wasm search, no-JS/keyboard/mobile boundaries and original local-only acceptance in the [application-owned Reader guide](../examples/docs-site/README.md); the original pilot and failure/verification log remain at the fixed pre-deletion Git commit |
| `docs/YT-225-DOCS-PUBLIC-BUILD.md` | **Merge and delete** | Preserve protected clean-source/edition, `--public`, fixed 15-file artifact, private receipts and build-only CI versus deployment distinction in the same Reader guide and [formal Pages publication record](GITHUB-PAGES-PUBLICATION.md). Original hosted PR/build and rejected-script evidence remain in fixed Git history |
| `docs/YT-226-PAGES-CUTOVER-PLAN.md` | **Merge and delete** | Preserve the actual published deployment/source, exact artifact and HTTPS readback, master-only switch, absence of old `.html` aliases, original failures (including final Firefox stale-heading FAIL) and bounded independent acceptance in `docs/GITHUB-PAGES-PUBLICATION.md`. Fixed Git history retains the complete YT-226 review and original section anchors |
| `docs/GITHUB-PAGES-PUBLICATION.md`, `site/pages.toml` and official release/evidence records | **Keep** | Formal Pages source/deployment history and the still-selected SDK alpha.8 / CLI 0.4.0 Reader Edition; source-only pilot/CI, actual Pages, SDK and CLI publication are separate scopes |
| Formal YT-224, YT-225 and YT-226 Done Backlog rows and all useful dependencies | **Keep** | Preserve IDs, Done states and dependency DAG; shorten repeated operational prose only after current guide/publication evidence links exist |

**Physical deletion result:** all three redundant YT files were deleted, without renaming or archiving copies, reducing tracked `docs/YT-*.md` **281 → 278** and net repository Markdown count by three. If a material technical-contract or formal publication gap blocks one deletion, amend this table and reobtain independent plan PASS before editing. Retain the original complete source at fixed `f03e8886bc1dbe28f135e05e527b69e01eb3deba`, checking Git blob identity and historical anchors before deletion. Current links must point to the Reader/Pages functional guides; historical references in `docs/YT-232-PUBLIC-GUIDE-EDITION.md` or other dated evidence may point to those exact immutable Git paths/anchors. Rewrite no actual evidence, release record or ADR; preserve all original failures. Do not modify `site/pages.toml` or product/CI code.

Expected living documentation changes: `examples/docs-site/README.md`, `docs/GITHUB-PAGES-PUBLICATION.md`, `docs/README.md`, `docs/ROADMAP.md`, `docs/BACKLOG.md`, `docs/DOCUMENTATION-SCOPE.md`, `docs/PUBLIC-DOCS-PLAN.md`, `docs/YT-232-PUBLIC-GUIDE-EDITION.md` and `site/README.md` only as needed to preserve current usage and repair references. Keep the existing Reader Edition input pair and the original static-export security bounds.

Acceptance: verified historical Git paths/anchors, all tracked link repair, Backlog dependency/Dones unchanged, `cargo xtask check docs`, `python3 scripts/release_docs.py check`, `git diff --check`, and existing local Reader build if inputs are touched. Require an **independent final frozen-diff Reviewer PASS** after implementation, then stage only reviewed paths, run the prescribed **strict/builtin staged Commit Guard**, match its `tree_oid` to `git write-tree`, and allow **one local commit only**. No Push, Release, Deploy, Pages setting change or cache cleanup.

## Current batch: Application-owned Rust CSS / Style (YT-237–241)

Authorization: owner-directed continuation after the Reader Edition commit. Starting local clean `master`: `e49657922bcdfe238cd1f79061178a32a4622a01`, with **278 tracked `docs/YT-*.md` files**. Inspect and independently **PASS this exact plan before changing any other document**. This batch edits documentation only; no product code, CI, CLI flags, sample execution, publication or live service mutation.

| YT source | Decision | Functional owner / essential retention |
|---|---|---|
| `docs/YT-237-STYLE-REGISTRATION-CONTRACT.md` | **Keep** | Still-cited normative registration, deterministic CSS vocabulary, ownership, process boundaries and compatibility contract. Do not delete a technical contract merely to hit a count; an independently reviewed future contract extraction may revisit the file's long historical appendices |
| `docs/YT-238-STYLE-BUILDER-PROOF.md` | **Merge and delete** | Completed local prototype, default module registration and SIMPLIFY decision. Current app-owned generator workflow belongs in `docs/CLI-GUIDE.md`; prototype-specific outcomes/failures remain recoverable via fixed Git revision, not copied into another numbered file |
| `docs/YT-239-STYLE-INTEGRATION.md` | **Merge and delete** | Actual `native-style`/`browser-style`, exact `[cli.styles]` configuration, default module registration, build/run/watch/Vite/Tailwind/publisher/CSP rules live in `docs/CLI-GUIDE.md`, `docs/CLI-CONTRACT.md`, `docs/ASSETS.md` and existing sample READMEs. Keep the dated source-only Linux validation and implementation review at immutable Git identity |
| `docs/YT-240-TODO-STYLE-QUALIFICATION.md` | **Merge and delete** | Retain copyable local Todo commands, CSS-only versus browser/Wasm generation, protected assets, `DEFAULT_CSP`, native/browser state and bounded Chrome/Firefox qualifications in `examples/todo-server/README.md`, `docs/ASSETS.md` and `docs/SAMPLES.md`. Original platform/failure/evidence table stays at fixed Git revision |
| YT-241 Style Showcase | **Keep functional sample** | There is **no standalone `docs/YT-241-*.md` file** to delete. Keep `examples/style-showcase/README.md` as the native-first sample guide and its bounded actual browser evidence |
| Official CLI/SDK release records, ADRs and Backlog dependencies | **Keep unchanged** | The CLI first shipped Style templates/generation in **CLI 0.3.0**; CLI 0.4.0 remains independently published, while Typed Tag Builder was introduced in **SDK 0.9.0-alpha.7**. No public Style SDK crate, no runtime style injection, no broader platform claim, and no Reader Edition promotion |

**Physical deletion result:** all three redundant closed YT Markdown records were deleted, with no renamed/archive copies: **278 → 275 tracked `docs/YT-*.md` files** and net Markdown-file change **−3** (no new task documents). Retain `YT-237` contract for its current technical authority; preserve the original three deleted files and exact historical anchors via fixed pre-deletion Git revision `e49657922bcdfe238cd1f79061178a32a4622a01`. Repair every local/floating `master` link to removed documents; current users should follow functional guides and the preserved technical contract. Historical evidence-only references in the dated YT-237, YT-242/243 or similar files may use full fixed Git URLs. Avoid spreading obsolete `unreleased`/Todo/Windows-not-verified statements into current guidance: mark historical qualification distinctly from actual shipped releases, bounded Linux/browser/Windows source evidence and any still-unverified modes.

Expected scope: `docs/DOCUMENTATION-CONSOLIDATION-PLAN.md`, `docs/ROADMAP.md`, `docs/BACKLOG.md`, `docs/README.md`, `docs/API-REFERENCE.md`, `docs/CLI-GUIDE.md`, `docs/CLI-CONTRACT.md`, `docs/ASSETS.md`, `docs/COMPATIBILITY.md`, `docs/SAMPLES.md`, `docs/YT-237-STYLE-REGISTRATION-CONTRACT.md` (reference-only), `docs/YT-242-DOCUMENTATION-TRUTH.md`, `docs/YT-243-RELEASE-QUALIFICATION.md`, `examples/todo-server/README.md`, and exact three deletions; touch fewer files if sufficient. Preserve `site/pages.toml` SDK alpha.8 / CLI 0.4.0 unchanged, all formal Done items and dependencies, YT-219 inventory, YT-222 protocol, original release/evidence, product source and CI.

Validation and commit gates: `cargo xtask check docs`, `python3 scripts/release_docs.py check`, `git diff --check`, exact historical blobs/anchors and incoming-link checks, 256 Backlog states/dependencies, and existing local Reader build **only if a selected input changes**. Freeze exact final diff and obtain **independent implementation Reviewer PASS** distinct from this plan review. Only then stage reviewed files, run the operator-installed Commit Guard with `check --staged --profile strict --engine builtin --format json`, verify Guard `tree_oid` equals `git write-tree`, and make one local commit. No Push, Release, Deploy, permissions adjustment or cache cleanup.

## Later functional batches (not authorized for implementation by this plan)

| Candidate theme | Inventory / consolidation intent | Boundary |
|---|---|---|
| Later CLI/startup and release maintenance | Audit M26 and later YT groups by capability, not simply ascending ticket number | Separate release/CI/evidence constraints; defer without inventing completion |

After each batch, update this plan's **small status table** (file counts, deleted paths, kept exceptions, commit/review outcome and next theme) instead of creating a new YT completion document.

## Completed batches

| Batch | Baseline YT files | Deleted YT files | Result | Evidence |
|---|---:|---:|---|---|
| M25 outcome/index pass, `b9873f1` | 288 | 1 | 287 remained; insufficient physical consolidation | Commit `b9873f15bcfd4c36a2274508ef7517e5a83b807b`; original shared plan recoverable from its fixed pre-delete revision |
| M25 YT-219–223 corrective consolidation | 287 | **4** | **283 remain**; YT-219 inventory and YT-222 static-export protocol kept, four histories restored by fixed links, no archive copy | The Git commit containing this plan; historical files at fixed `b9873f15bcfd4c36a2274508ef7517e5a83b807b`. Final check/reviewer/Guard evidence is retained outside the repository |
| Typed View / Tag Builder YT-234–235 | 283 | **2** | **281 remain**; published SDK alpha.7 usage and limits consolidated into View/API/compatibility guides; original source-only reviews available from fixed Git, no archive copy | The local commit containing this result; original files at `9a6f954dc38f3b9e854fce5a08c9b843182657f8`. Final validation, reviewer and Commit Guard evidence retained outside the repository |
| Reader Edition and Pages YT-224–226 | 281 | **3** | **278 remain**; current Reader/Pages functional guides preserve actual deployment, security/reader bounds and original failed checks; no archive copy | The local commit containing this result; original pilot/build/cutover evidence at fixed `f03e8886bc1dbe28f135e05e527b69e01eb3deba`. Final validation, review and Guard records retained outside the repository |
| Application-owned Rust CSS/Style YT-237–241 | 278 | **3** | **275 remain**; retained YT-237 normative registration contract and YT-241 Showcase sample; merged 238 prototype/239 CLI integration/240 Todo qualification into existing functional guides | Original three documents and headings at fixed `e49657922bcdfe238cd1f79061178a32a4622a01`; independently reviewed final diff, validation and Guard evidence retained outside the repository |
