# YT-191 — View authoring, opt-in lookup and current CLI status

Date: 2026-10-26 (Asia/Taipei). Status: **Done at bounded documentation scope after independent Reviewer PASS,
zero issues/required followups.**

## Authority and baseline

The maintainer explicitly asks to fix all three DX gaps identified after YT-190:
stale CLI status, a bridge from the first app to composing Views/components,
and concentrated opt-in selection information. Continue the document-only scope,
preserve accepted evidence, use existing APIs, and perform independent review
before Done. No new sample/runtime/tests, accepted-verification replay,
Commit/Push or publication is selected.

Starting branch is `docs/yt113-123-opt-in-capability-plan`, HEAD
`57a56d1c8d446ce769dbe68660d037fc566587b0`. The eight-path YT-190 commit passed
explicit strict/builtin and native hook Guard, exit 0 and zero findings; reviewed
source hashes and tree matched, checkout clean, no Push. Earlier results/failures
remain retained at their original source boundaries.

## Contract selected before changes

Correct the two present-tense YT-112 In progress claims in the current CLI guide
using its accepted implementation record. Describe completed bounded Linux
browser/lifecycle acceptance, retain first failed attempts and keep new Windows
browser-watch execution Not verified. Do not rewrite its historical report,
expand platform support or rerun that qualification.

Add [View/component authoring](VIEW-COMPONENTS.md) using the existing five-part
[guide template](GUIDE-TEMPLATE.md). Continue the existing plain first app, keep
its imports/listener/other route, and label helper/handler replacement code.
Use only existing View/Element constructors and ordinary Rust functions for
typed input, owned values, lists, empty-state conditionals and nested composition.
Explain expected escaped SSR text, no automatic styling/events/state and distinct
HTML/DOM policies. No Component trait, macro, DI or new application is added.
The reader fragment is not an executed runtime or compile qualification claim.

Add [the opt-in quick reference](OPT-IN-QUICK-REFERENCE.md) as a lookup catalog,
not a duplicate API contract or new configuration surface. Concentrate package/
feature/target, explicit startup entry or typed options, exact existing external
keys/defaults, named consumer and corresponding guide. Distinguish framework
packages from app-owned integrations; compilation from installation; Settings
loader from SecretProvider and from consumers that do not choose Settings.
Maintain source/security, authorization, CSRF, fresh-v2 authority, cache display
and safe tracing limits. Research Done does not imply a delivered generic crate.

Add narrow links in root README, docs index, first-app/API map, publication
source table and core guide. Register one bounded YT-191 item, In progress until
independent Reviewer PASS. Do not restructure all guides, duplicate every pub
signature, select new features or reopen release decisions.

## Delivery scope and verification boundary

Twelve Markdown files: new `docs/VIEW-COMPONENTS.md`,
`docs/OPT-IN-QUICK-REFERENCE.md` and this record; existing `README.md`,
`docs/README.md`, `docs/FIRST-APP.md`, `docs/API-REFERENCE.md`,
`docs/DOCUMENTATION-SCOPE.md`, `docs/BACKLOG.md`, `docs/ROADMAP.md`,
`tools/yetlet-cli/README.md` and `crates/yetlet-core/README.md`.

Evidence and pre-edit contract/baseline are retained under
`target/yt191-dx-guides-20261026/evidence/`. Use source/guide/manifest inspection,
repository-required new-delivery `cargo xtask check docs`, whitespace and bounded
scope/source preservation checks. Independent review is read-only; after PASS
only record/index/Backlog/Roadmap completion metadata follows, plus the final
document gate. No reader commands, generator, sample/lock creation, fetch,
build/test/rustdoc/browser, cloud/service/security scan or old validation runs.
No source/API/feature/settings-schema/lock/template/test file changes.

New documentation remains unstaged/uncommitted/unpushed/unpublished. YT-096 with
D1–D6 Pending and YT-020 Blocked remain unchanged; no release scope expansion.


## Delivered draft and bounded verification

The View authoring guide follows the five required integration sections and
continues the existing scaffold's home handler/imports. It uses only existing
constructors, passing borrowed application records into ordinary functions that
return owned Views. Expected text/escaping and empty-state/class behavior are
source-derived instructions, not a claim that the new fragment was executed.
The quick reference has eleven framework-selection rows and four application
integration rows, with full external keys and named consumer/default boundaries.
Current CLI paragraphs now describe accepted bounded Linux execution while
retaining unverified Windows browser-watch limits and links to original failures.
All pre-existing CLI fenced code blocks are unchanged.

Initial new-delivery `cargo xtask check docs` returns 0: 75 required documents,
191 backlog items; docs/links/Markdown/whitespace/DAG PASS. Tracked whitespace
passes; three new Markdown files have empty no-index whitespace diagnostics
(exit 1 is the expected content difference). Exactly twelve Markdown paths
change, staged zero, and all 1,100 non-target tracked hashes remain unchanged.
Seventeen manifest/source/history bindings support API, feature, settings and
existing acceptance facts. No runtime/source/API/manifest/lock/template/test or
earlier record changes, and no reader command, old validation or environment
execution. At this draft checkpoint independent acceptance was pending; the
completed result follows below.


## Independent acceptance and completion

Independent Reviewer `/root/yt191_dx_guides_independent_review_20261026` returns
PASS, zero issues, required followups and optional notes. Native trace confirms
the registered role, configured model and effective effort match the saved
healthy/current/eligible workspace profile and central resolver decision.
Twelve reviewed source hashes, six evidence hashes and all 1,100 non-target
tracked hashes match before/after review; HEAD/branch/status and staged-zero
state are unchanged. No source correction or new verification was requested.

Only this record and index/Backlog/Roadmap completion metadata follow acceptance;
reader guides, API/navigation additions and CLI fixes remain frozen. Final
new-document/whitespace and source-preservation checks are retained with the
review and initial document-check receipts. YT-191 is Done at source-backed
documentation scope, without executing the authoring fragment or guide commands,
adding tests or replaying earlier qualification. This does not establish a
new platform/runtime or measured novice-DX qualification.

Twelve document changes remain unstaged/uncommitted/unpushed/unpublished.
Existing API, feature, wire/settings schema, trust/security and release boundaries
remain unchanged. The selected three gaps are closed; no further implementation
is selected. YT-096/D1–D6 owner decisions and YT-020 remain unchanged.
