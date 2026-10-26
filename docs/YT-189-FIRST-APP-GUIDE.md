# YT-189 — Build your first Yetlet application guide

Date: 2026-10-26 (Asia/Taipei). Status: **Done at source-backed documentation
scope after independent Reviewer PASS, zero issues/required followups.**

## Authority and starting state

The maintainer explicitly authorizes resuming the YT-188 blocked commit with a
subshell-local standalone synthetic environment assignment, preserving both
failed Guards, independent rereview and fresh strict/builtin Guard before the
same sixteen-path commit/no Push. That correction is accepted and committed as
`ebce42b8624c99197ce0b1c2c2c0e1bbf19c2320`, tree
`af5000bc91f8ded864de4be61be64c883190df89`: reviewer/directGuard/hook PASS,
zero issues/findings, committed blobs match reviewed inputs and checkout clean.
Both earlier blocked receipts and independent-review history remain unchanged.

The separately authorized next task is a first-application walkthrough joining
existing documentation, documentation only, no accepted-validation replay,
independent Reviewer before Done and no new-change Commit/Push/publication.
No new app/sample, runtime, template, dependency, lock or CLI behavior is selected.

## Selected guide contract

Add [Build your first Yetlet app](FIRST-APP.md), following the existing five-part
[guide template](GUIDE-TEMPLATE.md). Select the supported Linux/POSIX, plain
native SSR path from an existing checkout. Use the exact current CLI `new` grammar,
standalone workspace and explicit compatible local Cargo paths for unpublished
`=0.1.0` core/server packages. Do not point users at the legacy `dev` template or
claim registry availability.

Connect prerequisites → local CLI build/explicit binary → protected unused app
parent → scaffold → manual local dependency selection → app-specific lock and
explicit dependency fetch → frozen CLI build/run → existing routes/results and
first owned text edit. Distinguish source-checkout examples from a user's new
standalone application, CLI settings from opt-in runtime Settings, and the plain
`YETLET_PORT` switch from settings-enabled app variables. Keep a clear same-shell/
working-directory story and refuse overwrite/unsafe-path workarounds.

Commands are reader instructions only. Link current CLI prerequisite/creation/
execution/error material, generated SELECTED-README, exact scaffold/main source,
server/API/configuration references and existing samples. Use source inspection
for expected output/route/escaping statements. Add a compact symptom/cause/next
step table without inventing a new diagnostic tool or weakening trust checks.

Root README gets one create-my-app task link; docs index, CLI guide and existing
publication-source selection get narrow links. Register Backlog/Roadmap YT-189
In progress; Done only after independent acceptance. Preserve all existing task
results/release/candidate decisions. The eight-path target is the new reader guide
and this record plus root README, docs index, CLI README, publication scope,
Backlog and Roadmap. No global documentation restructuring is selected.

## Verification and limits

Use existing-source/manifest/template inspection, the required new-document gate
`cargo xtask check docs`, whitespace, exact file-scope and preservation checks.
Do not run a CLI generator, create a fixture project, install tools, generate or
fetch a lock, run build/check/test/rustdoc, open a listener/browser, scan security
or access a cloud service. Documented commands are not executed acceptance proof.
This is a source-backed walkthrough, not new cross-platform/runtime qualification.

Evidence and baseline hashes are retained under
`target/yt189-first-app-20261026/evidence/`. Current API semantics, the original
candidate and YT-096/D1–D6/YT-020 release decisions remain unchanged; new documents
stay unstaged/uncommitted/unpushed and unpublished.


## Delivered scope and evidence

The five-section reader walkthrough joins the existing plain native CLI/template
path with explicit Linux/POSIX toolchain/binary/parent/manifest variables, local
exact-version dependencies, app-owned lock and dependency fetch, frozen build/run,
expected existing routes and common failure guidance. It labels shell scope,
manual TOML path replacement, default CLI settings versus optional runtime
configuration, and ordinary run versus watch. No legacy `dev` command is used.

Eight changed Markdown paths: new `docs/FIRST-APP.md` and this record;
`README.md`, `docs/README.md`, `tools/yetlet-cli/README.md`,
`docs/DOCUMENTATION-SCOPE.md`, `docs/BACKLOG.md` and `docs/ROADMAP.md`.
The existing API map, ten integration guides, source/templates/manifests/locks
and earlier historical reports are unchanged. Both failed Guards and the later
accepted subshell correction/commit remain in YT-188 evidence.

New-delivery `cargo xtask check docs` exits 0: 75 required documents and
189 backlog items, links/Markdown/whitespace/DAG PASS. `git diff --check` passes;
new-reader/new-record no-index whitespace exits 1 with empty diagnostics, the
expected content-difference status. Static checks confirm the five top-level
sections, the two version/path declaration shapes against the current root and
selected scaffold, exactly eight Markdown paths and 499 protected source/guide/
history hashes unchanged. Source bindings include current CLI grammar, selected
scaffold/README, native main and frozen execution seam.

No tutorial command, CLI generator/help/doctor, app creation, lock generation,
dependency fetch, build/test/rustdoc, browser/listener, service/cloud/security or
old qualification was executed by this task. This is documentation/source review,
not new runtime/platform or novice-user-study qualification. Independent Reviewer
acceptance now returns PASS, zero issues/required followups, from
`/root/yt189_first_app_guide_independent_review_20261026`. Native role/model/
effective effort match the saved profile and central resolver. Before/after
checks match all eight source, 11 evidence and 499 protected-input hashes;
HEAD/branch/status and staged-zero state are unchanged. Only record and
Backlog/Roadmap/index completion metadata follows, with the narrow final
document gate. YT-189 is Done at source-backed documentation scope, with no
new runtime/platform qualification, stage/Commit/Push/publication. No next
implementation is selected; YT-096 remains In progress with D1–D6 Pending,
YT-020 Blocked.
