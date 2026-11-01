# YT-226 — Controlled switch to the Yetlet-built Pages guide

Date: 2026-11-01. Status: Local integration reviewed; hosted/publication acceptance pending.

## Current boundary

The owner authorizes Push of f73439b and squash Merge of PR44 after green CI,
then this plan only. The existing public website must remain in place. The
existing PAGES_DEPLOY_ENABLED variable was observed true; it is now false to
prevent the authorized Merge from automatically deploying the old mdBook flow.
This holds future updates, not availability of the already published website.
The switch stays false until a later deployment authorization.

[PR44](https://github.com/bohewu/yetlet/pull/44) has now squash-merged as
99902bbb0ecaa07a14c1133f5d863d211632135c, tree
6015c8d8c3ee4a412924b72c8c1ef278dfe00585, identical to the qualified f73439b
tree. Required Rust/source checks, both document builds and all four CodeQL
analyses passed before Merge, without retries. The new master mdBook build
41277712999 passes and its deployment job skips. Subsequent master readback
confirms Rust MSRV/Stable41277713000, guide artifact41277713008, mdBook41277712999
and all four CodeQL analyses41277712950 succeed on attempt1 at99902bb. No manual
rebuild, retry or deployment is dispatched. This completes the owner's master-CI
gate before independent review of these six planning documents.

Original local master9b46c30 and the PR branch atf73439b remain intact; a verified
external Git bundle preserves the PR-source range. This reviewed plan is on a
new named branch from99902bb in the same checkout, without reset or a detached
worktree. These planning edits are uncommitted at the review checkpoint. The
owner separately authorizes a strict/builtin local commit only after independent
Reviewer PASS; Push and deployment remain unauthorized.

The last accepted mdBook publication is source64d1074 and run41198529190/1.
YT-225 has an accepted clean public-mode guide artifact at run41265011134/1,
15 exact files,8 source/excerpt bindings and independent Reviewer PASS0. Its
source is a PR merge-ref, not the future production documentation-source commit.
Neither this plan nor the product Merge promotes that PR artifact into Pages.

## Recommended scope

- Keep https://bohewu.github.io/yetlet/ and /yetlet/ as the project base. Publish
  the same eight selections/excerpts from site/pages.toml, SDK0.9.0-alpha.5 and
  CLI0.1.1, readable HTML and the current owned Wasm search.
- The owner declines old mdBook .html aliases. Adopt /chapter/ directory URLs
  deliberately; old bookmarks are allowed to break at the eventual cutover.
- Do not add Wiki, custom domains, hosted rustdoc, Markdown inputs, task records,
  screenshots, private export receipts, release tooling or new SDK features.
- Preserve github-pages Environment/master restriction, HTTPS, existing action
  pins, same-run artifact deployment and the accepted routine-update policy.
  No Secret, release Environment or product-signing identity is needed.

Keep the current plain escaped code and literal chapter search. Complete mdBook
UI parity, themes, print controls and syntax highlighting remain outside the
first cutover scope.

## Future implementation, with the deployment hold retained

Use tools/build_yetlet_docs.py as the single build entry point. Replace only the
mdBook-specific build steps in .github/workflows/docs.yml with the already
proven tool/dependency preparation and clean --public build from docs-preview.yml.
Select the complete immutable event commit, expand source path filters and retain
the private manifest/CLI receipts outside the site root. Do not upgrade tools or
dependencies during the switch.

Upload only the generated15-file site root through the existing pinned Pages
artifact action. Keep .nojekyll and exclude its parent directory/records. The
deploy job still consumes its own successful run's artifact, not a PR artifact,
arbitrary URL or another run. PR builds have no deployment permissions.

Retire docs-preview.yml when its build path is absorbed by docs.yml so ordinary
document changes do not launch duplicate site builds. Keep the current independent
Rust/source checks and release workflows; this plan does not weaken CI policy.

Implement and independently review this bounded workflow change while
PAGES_DEPLOY_ENABLED remains false. Later Git/CI/Merge authority is needed for
those new changes. Verify the merged workflow's actual build and skipped deploy
before considering activation. Proposed build/deploy limits remain30/10 minutes;
observed hosted timing, not these starting limits, determines later tuning.

## Evidence reuse and the missing checks

Reuse accepted YT-223 GNU/MSVC static export and YT-224/YT-225 canonical content,
parser/escaping, Chrome/no-JS/mobile/keyboard/search/failure/lifetime and artifact
proof. Do not replay whole product/platform/browser suites or benchmarks.

Only new workflow wiring requires focused syntax/mode/artifact-root/permission
checks and the actual clean master build. Compare its manifest with the selected
commit/edition/eight source hashes and retain output hashes. The new master SHA
will differ from the accepted PR artifact; that source binding must be observed,
not inferred from identical guide bodies.

A narrow Firefox consumer search/keyboard check is recommended before activation;
the existing Firefox framework proof is retained and does not assert this reader
journey. This recommendation adds no test execution to the current plan-only work.

## One later activation decision

Recommend authorizing one concrete batch after reviewed workflow/master readiness:

1. Identify the immutable new master source and the last-good mdBook output,
   source/manifest/deployment record, with retained fallback bytes available
   outside expiring Actions storage. Confirm actual current-site correspondence
   before choosing that artifact as rollback input.
2. Enable the switch and dispatch docs.yml once on that master, within the
   proposed30-minute build/10-minute deploy limits. Keep the existing same-run,
   project-base, HTTPS and master-only Environment checks. No new per-deployment
   manual Environment gate is proposed.
3. Read back public HTTPS home, First app and API pages, a nested URL/refresh and
   selected CSS/JS/Wasm assets. Compare hashes/source/edition, MIME/CSP and one
   focused search/menu journey. These are new hosting-boundary observations,
   not a replay of local content or throughput validation.
4. If that readback fails, keep the failure and disable further automatic
   updates. The batch may explicitly include one rollback to the identified
   last-good artifact through the protected Pages path, with its own readback.
   If selected, its restore path must first be implemented/reviewed to verify
   that fixed output and upload it within the deployment's own master run;
   an arbitrary cross-run artifact cannot be passed directly to the deploy job.
   Rollback is a deployment and is not authorized now. Do not automatically
   retry the failed new build/deployment or silently switch source identities.

After accepted publication, the existing policy of automatic relevant protected-
master document updates can resume. A rollout approval must explicitly cover the
cutover and any bounded rollback; no release tag, signing or crates.io operation
follows from it. Current settings hold and this plan alone authorize neither.

## Independent planning acceptance — 2026-11-01

After all selected merged-master checks pass, independent Reviewer PASS0 accepts
these six planning documents with zero issues/required followups. All six frozen
source hashes/copies, worktree status, branch, HEAD and tree match. The reviewer
reconciles actual retained CI/hold evidence and existing workflow boundaries
without rerunning tests, opening browsers, modifying sources or changing settings.
Registered reviewer gpt-6-astra/effective high are verified.

Documentation/internal-link/Markdown/whitespace/DAG checks pass:75 documents,
226 backlog items. The owner authorizes a strict/builtin local commit of these
six files after this PASS. This accepts the plan only: YT-226 implementation and
activation stay Todo, the deployment switch stays false, and no Push or deployment
is selected. Existing publication and failed-check history remain unchanged.

## Owner-selected minimal workflow integration — 2026-11-01

The owner commits the reviewed six-file plan as c5b8608 with strict/builtin
Guard PASS0 and no Push. The next authorized scope is local integration and
independent review only: no Commit/Push, hosted dispatch or deployment. The
repository switch is read back false and is not changed.

The checkout now has one documentation workflow, docs.yml. It absorbs the
previously accepted docs-preview.yml's exact Rust1.99.0/wasm-bindgen0.2.129
preparation and tools/build_yetlet_docs.py --public invocation. The duplicate
workflow is removed. Source path filters cover the selected manuscripts, app,
selection, shared helper, build/test tools and source CLI. Build timeout is30
minutes; deployment remains10. Tool/dependency versions, action pins and
SDK/browser/exporter/CLI runtime code are unchanged.

Clean public-format output is independent of permission to publish. The private
job output is now named deploy; the existing master/event/switch checks and
deploy job otherwise remain identical. Build permissions stay contents:read;
Pages/OIDC permissions and the github-pages Environment stay in the guarded
deploy job. No global cancellation is added to the deployment path.

The upload action receives only the generated15-file site root under
RUNNER_TEMP/yetlet-guide/public/yetlet. Its explicit include-hidden-files:true
preserves .nojekyll: the [pinned official action](https://github.com/actions/upload-pages-artifact/blob/fc324d3547104276b827a68afc52ff2a11cc49c9/action.yml)
otherwise excludes hidden files by default. The same run's named github-pages
artifact is the sole deployment input. Manifest/CLI receipts upload separately,
including on failure, never as part of the site. Retention stays seven days.

New focused verification:

- Four PagesWorkflow tests PASS, executing the actual Bash deployment selector
  and checking the new output connection, public-format/eligibility separation,
  single workflow, upload root/hidden-file/private-record and permission gates.
  Only this new class runs; prior publication/parser/product cases are reused.
- Existing actionlint1.7.12 passes the changed docs.yml. Tool preparation is
  byte-identical to the accepted preview workflow. The deploy job differs only
  in the private output-key reference; existing branch/event/concurrency and
  protected Environment/action behavior are reused.
- The official pinned action's Linux tar script, with hidden-file inclusion,
  packages the accepted YT-22515-file fixture. All15 file bytes match; .nojekyll
  and root index.html remain present. Sentinel manifest/receipt files beside the
  upload root are excluded. This is new archive-transport proof, not a rebuilt
  app or a new public-source candidate. The archive is retained externally and
  the owned temporary fixture removed; no upload/server/browser is started.

Evidence is retained outside the repository in the implementation subdirectory
of the YT-226 evidence set. No old application build, native/Wasm/browser/MSVC
suite, benchmark or hosted job is rerun. Local Reviewer acceptance does not
verify the unpushed workflow on GitHub or replace the public website. Git delivery,
clean hosted workflow/build/skip evidence and explicit activation/public readback
remain open; YT-226 stays In progress until its full acceptance boundary is met.
The optional rollback path is not implemented by this minimal integration.

## Independent local integration acceptance — 2026-11-01

Independent Reviewer PASS0 has zero issues/required followups. All11 changed
paths match the frozen sources, deleted-file baseline, diff and worktree status
at c5b8608. The reviewer reconciles unchanged tool preparation/deployment policy,
the four new workflow cases, actionlint/docs checks and the actual15-file Pages
archive against its accepted fixture/manifest. No test/build/browser run,
extraction, source mutation or external action is performed by the reviewer.
Registered reviewer gpt-6-astra/effective high are verified.

The repository switch is independently read back false after review, without a
settings write. All11 paths remain unstaged/uncommitted; no Commit/Push, hosted
dispatch, upload or deployment occurs. YT-226 remains In progress, with the
clean hosted new workflow and actual authorized activation/readback still open.
