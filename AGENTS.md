# Yetlet — repository agent instructions

## Intent and scope

Build a **small, HTML-first Rust universal UI framework**. The guiding contract is in `README.md` and `docs/MVP.md`; architecture, proposal status, milestone gates and task dependencies are in `docs/ARCHITECTURE.md`, `docs/adr/`, `docs/ROADMAP.md` and `docs/BACKLOG.md`.

This repository contains an Experimental SDK and independent CLI with accepted public releases. Use the publication records linked from `docs/RELEASE-READINESS.md` for distribution identity and `site/pages.toml` for the selected guide edition. Later checkout additions can be source-only even when development manifests retain a published version. Production qualification remains open. Do not assume undocumented or deferred interfaces exist.

## Documentation language

Write **all repository documentation in English**, including the README, ADRs, architecture, roadmap, backlog, validation records, and future Markdown files. Preserve the original technical meaning, acceptance evidence, links, IDs, and decision status when editing. Run `cargo xtask check docs` to verify language, references, and backlog structure.

### Known ignored evidence diagnostics

Historical, intentionally retained `.pipeline-output/` handoffs, review
requests, failed-run logs and receipts can be Git-ignored yet still be scanned
by `cargo xtask check docs`. Git ignore status does **not** imply checker PASS.

- Preserve original ignored evidence and the actual nonzero checker result;
  never delete/edit evidence, bypass checks or call a failing command PASS to
  clear historical diagnostics.
- If a failure is wholly attributable to previously identified, unchanged
  ignored evidence, record its source paths/count and classification in the
  ignored task handoff once. **Do not repeatedly foreground or re-run the same
  known non-actionable failure** in later progress reports or handoffs.
- Continue scoped validation of new or touched source/doc files. Newly
  introduced findings, changed identities/counts, tracked-file failures,
  dependencies, CI/Commit Guard/release gates or product defects remain
  actionable. Investigate and report them; an ignored-path error can still
  matter if it actually blocks a required gate.
- If a required broad command still exits nonzero, preserve its true result
  in detailed evidence and qualify any completion claim. Reuse unchanged-source
  receipts instead of rerunning a known failing global check for bookkeeping.
  Keep routine summaries focused on changed tests, reviewers, Git state and
  open product gates; mention historical-only findings again only if they
  materially change a decision.

## Work sequence

1. Read this file, the relevant ADR(s), the targeted backlog item(s), and any higher-priority workspace policies before making changes.
2. Implement one bounded vertical slice in the **current checkout** by default; do not create a detached worktree unless expressly requested.
3. Keep server HTTP routing, UI component semantics, HTML/DOM renderers, browser loader, and command transport separable.
4. Add tests that demonstrate the acceptance criteria for the item; report compile / browser / CSP verification evidence separately.
5. Only mark backlog items Done when evidence exists. Update roadmap gates based on completed work, not intentions.
6. Do not commit, push, deploy or alter external environments without explicit instruction. Preserve unrelated work.
7. When changing a fundamental architectural decision, update the relevant ADR and explain tradeoffs.

## Evidence-scoped platform verification

Choose verification environments by the behavior at risk, not a blanket
Ubuntu-and-Windows matrix. State the exact platform, product version, source
identity, tested actions and unverified limits for every reused result.

- Native CLI installation/compilation, process supervision, console signals,
  filesystem ownership and other OS integrations need focused checks on the
  affected native platform (including Windows x64 MSVC when Windows behavior
  changes). Reuse already accepted exact-source builds and runtime receipts;
  unrelated web changes do not force another Windows CLI run.
- Shared Browser DOM, JavaScript/WASM, CSS/Vite and normal CSP behavior can
  qualify in one real, supported browser environment, ordinarily Ubuntu
  Chrome. Windows Chrome is **not** a default release gate. Reuse older browser
  evidence only after comparing SDK/CLI versions, generated template/source
  identity and the specific changed contract; a difference needs a focused
  check for that difference, not an entire second OS/browser matrix.
- GPU/graphics checks require an actual supported device, driver/backend and
  visual or engine evidence. Ubuntu GPU qualification is sufficient for a
  platform-neutral renderer; add Windows or other hosts only for a named
  platform-specific graphics backend or reproducible platform defect.
- Preserve distinctions between a strict CSP declaration, successful normal
  browser execution with zero violations, and deliberate negative enforcement
  tests. Never label an unperformed inline-handler/blocking probe PASS. Security
  changes that touch CSP or browser trust boundaries still need risk-appropriate
  negative tests, but not automatic cross-OS duplication.
- A platform or Browser control denial is **not** permission to launch through
  an alternate route. Report the unsupported gate or accept independently
  grounded cross-platform evidence within its proven scope.

## Development artifacts

Follow `docs/DEV-ARTIFACTS.md` for new isolated verification runs, including
WebCodex runners. Separate rebuildable Cargo output from logs, original failures,
manifests and accepted artifacts; retain existing historical evidence in place.
Reuse stable daily caches. Use `scripts/dev_artifacts.py` for layout, space
inventory and cleanup preview only. Actual deletion requires separate authority;
never run wholesale cleanup on a mixed historical `target/` tree.

## Version-control guard

Before committing or pushing, use Commit Guard with the explicit `strict` profile and `builtin` engine: scan the staged index before a commit and the exact outgoing commit range before a push. Any nonzero result stops the operation; do not fall back to audit, bypass hooks, or weaken the guard. A passing scan does not grant authorization to commit or push; preserve the user's explicit authorization requirement.

After a successful merge, delete the same-repository remote PR head branch once
its live tip matches the merged PR head. Preserve protected branches, branches
used by an open PR or active worktree, and archival or research refs. Use an
exact-tip lease for deletion and verify that the remote ref is gone.

## Release documentation gate

Before selecting candidate C or packaging a release, follow
`docs/RELEASING.md#release-documentation-gate` and run
`python3 scripts/release_docs.py check`. Review the package READMEs, installation
and expected-version output, CLI/API guides, capability map and documentation
index against the selected manifests and release scope. Update affected content;
a touched file alone is not proof of correctness. Any failed check stops source
freezing or release execution. Never replace version-derived checks with literals.

Package READMEs must describe what the selected release includes without claiming
an upload already happened or retaining current-version pending/unreleased text.
Preserve explicitly historical checkpoints and the separately pinned public guide
edition. After every verified SDK or CLI publication, update the Reader Edition, README
and Pages to the accepted published pair before closing the release item. Follow
`docs/RELEASING.md#required-reader-update-after-every-sdk-or-cli-publication`;
require consumer/native/Wasm verification and live Pages readback. Development
versions alone never promote the reader. Independent review covers semantic
claims outside the automated check's documented scope. No check grants Git,
hosted, signing, publication or Pages authority.

## Technical guardrails

- Default SSR must work without JS/WASM and without a persistent server-side event circuit.
- Client events stay local; remote behavior crosses a typed HTTP/action boundary unless an optional custom transport is installed.
- Do not evaluate code or compile Wasm on each request; serve build-time versioned assets.
- Strict CSP, HTML/state escaping, semantic HTML and accessibility are correctness requirements.
- Do not put server-only secrets or capabilities into browser builds or serialized hydration state.
- WebGPU is a separate optional renderer, **not** an accelerator for normal DOM/CSS.
- Do not replace a mature HTTP parser/server implementation with an in-house transport stack.
- M6 application ergonomics are **handler-first and component-first**; MVC and Controllers are optional. Do not introduce mandatory Controller scaffolding, duplicate model binding/filters, a broad result hierarchy, or a DI container. Preserve native Axum/Tower interoperability (see ADR 0006).

## Work reporting

Include: task IDs, actual changed files, validation commands/results, known limitations, open decisions, and the next unblockable backlog item. If a toolchain or dispatch mechanism is unavailable, explicitly separate completed repository work from validation that was not run.
