# YT-190 — Common-error troubleshooting guide

Date: 2026-10-26 (Asia/Taipei). Status: **Done at source-backed documentation scope after independent Reviewer PASS,
zero issues/required followups.**

## Authority and starting state

The maintainer authorizes an eight-path YT-189 strict/builtin commit without
Push, followed by documentation-only common-error guidance using symptom → cause
→ correction → corresponding guide. Preserve accepted evidence; no old validation
replay. Independent Reviewer acceptance is required before Done. New changes
remain uncommitted/unpushed/unpublished.

YT-189 is committed as `4a4470b0c6696317cc26f796692ad41a4d052112`, tree
`03b20c4630286faf2c272dda1fea6aa7c57f9c3d`. Direct and hook Guard both return 0,
strict/builtin, eight files, no blocking findings; three warning-only generic
Windows example paths in the existing CLI README are retained. Committed blobs
match the eight reviewed hashes and the checkout was clean. No Push occurred.
The original YT-188 failures and subsequent accepted correction remain intact.

## Contract selected before edits

Add [Troubleshooting common problems](TROUBLESHOOTING.md), a lookup catalog
rather than another runnable tutorial. Each row has symptom, possible cause,
correction and exact existing guide links. Cover bounded first-use/CLI, browser,
settings/secrets, middleware/Auth/Session and rate-limit confusion. Current source
and guides own complete procedures; causes remain conditional where a symptom
or HTTP status does not identify a root cause.

Preserve frozen execution/local dependency/cache preparation, profile and target
selection, path protections, secret redaction, CSP/DOM policy, authentication
versus authorization, Session/CSRF/rotation and quota/provider failure boundaries.
Do not recommend bypasses, provider fallback, implicit retries or extra queues.
Settings remain opt-in and application-owned; no SettingsProvider trait, generic
runtime registry, new default feature or new diagnosis tool is introduced.

Add narrow links in the product README, docs reader index, first-app guide and
explicit publication-source table. Register Backlog/Roadmap YT-190 In progress;
only independent PASS permits completion metadata. Preserve old task verdicts,
source identities and release decisions. The guide is a reference catalog under
[the guide-template boundary](GUIDE-TEMPLATE.md), not a forced five-part tutorial.

## Verification boundary and file scope

Eight Markdown paths: new `docs/TROUBLESHOOTING.md` and this record, plus
`README.md`, `docs/README.md`, `docs/FIRST-APP.md`,
`docs/DOCUMENTATION-SCOPE.md`, `docs/BACKLOG.md` and `docs/ROADMAP.md`.
Use source/guide inspection, the repository-required new-document check
`cargo xtask check docs`, whitespace and exact scope/preservation checks.
Freeze reader content for an independent read-only Reviewer; after PASS only
record/Backlog/Roadmap/index completion metadata may change.

Do not execute guide commands, recreate errors, generate an app/lock, fetch
packages, build/test/rustdoc, start a browser/listener, access cloud/services or
replay previous qualification. This is source-backed documentation, not measured
error recovery, novice-user testing, cross-platform or production qualification.
No runtime/API/feature/schema change or release-candidate expansion is selected.
Evidence is retained under `target/yt190-troubleshooting-20261026/evidence/`.
YT-096 remains In progress with D1–D6 Pending; YT-020 remains Blocked.


## Delivered draft and bounded checks

The catalog has 27 four-column symptom rows across six groups: create/start,
CLI settings/execution, browser, runtime settings/secrets, HTTP/Auth/Session and
rate limiting. All remedies point to existing guidance, distinguish possible
causes from a diagnosis, and retain ownership, failure and safety semantics.
The root README stays a product entry; prior first-app instructions remain
unchanged apart from one troubleshooting pointer. Publication remains an exact
source selection, without a website or transitive export.

Initial new-delivery `cargo xtask check docs` returns 0: 75 required documents,
190 backlog items; docs/internal links/Markdown/whitespace/DAG PASS. Tracked
`git diff --check` passes. Both new Markdown files have empty no-index whitespace
diagnostics (exit 1 denotes a content difference). Scope checks confirm eight
Markdown paths, staged zero, and all 1,101 non-target tracked file hashes unchanged.
Inspection also corrects the new inspection link to the existing YT-111 heading.
No reader command/error reproduction or old runtime qualification is executed.

An initial edit assumption that the existing docs index had one closing details
tag failed before changing the index. Its two sections remain intact; the one
bounded editing correction inserts the new record before the final close.
The failed observation is retained in `editing-observation.json`; it is not a
runtime/product failure or an excuse for wider source work. Independent review
is pending; final completion metadata and the new-document gate follow acceptance.


## Independent acceptance and completion

Independent Reviewer `/root/yt190_troubleshooting_guide_independent_review_20261026`
returns PASS, zero issues, required followups and optional notes. Native trace
confirms the registered reviewer, configured model and effective effort match
the saved healthy/current/eligible workspace profile and central resolver.
All eight reviewed source hashes, seven evidence hashes and 1,101 non-target
tracked hashes remain unchanged; HEAD/branch/status and staged-zero state match.
The source-binding map retains 23 existing guide/source references.

Only this record and Backlog/Roadmap/index completion metadata follow acceptance;
the reader catalog and navigation instructions remain frozen. The final
new-document gate/whitespace and closing Git/hash receipts are retained alongside
the initial checks. YT-190 is Done at documentation/source-inspection scope.
No commands were executed to reproduce errors, and no runtime/API/source,
features/settings keys, samples, dependencies, old validation or environment
changed. No new tests, build/rustdoc/browser/service/cloud run or qualification
claim is added. New changes remain unstaged/uncommitted/unpushed/unpublished.
YT-096/D1–D6 and YT-020 decisions remain unchanged; no next implementation
or new unblocked backlog task is selected by this documentation delivery.
