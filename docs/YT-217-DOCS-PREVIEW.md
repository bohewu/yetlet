# YT-217 — API doc comments and local reader preview

Date: 2026-10-31. Status: **Done at selected API-documentation and local
reader-preview scope after final independent Reviewer PASS, zero issues or
required followups.** Earlier Pending/stopped checkpoints below retain their
original meaning; the final acceptance section governs current status. No Pages
activation, broader platform qualification or publication is claimed.

## Authority and boundary

The owner authorizes committing the preceding twelve documentation files with
strict/builtin and no Push, then implementing the selected
[YT-217 scope](PUBLIC-DOCS-PLAN.md#implementation-order--yt-217) and obtaining an
independent Reviewer. Commit `9ff77b0c2f193a399a1694cca418173db1ae8296`, tree
`2c4a102225a54d84435efb293f1260afadbfa1d2`, contains exactly those twelve files.
Direct staged Guard and native hook exit0 with strict/builtin, zero blocking
findings and three non-blocking generic Windows-drive-path warnings in the
existing CLI guide. No Push occurs. New YT-217 files remain uncommitted.

The new Rust changes contain documentation comments only. No signatures,
executable code, feature graph, versions, lockfiles, settings or runtime policy
change. No prior product/platform/browser qualification is replayed. Published
SDK0.9.0-alpha.5 and CLI0.1.1 archives and signatures remain unchanged.

## Implemented slice

- Both SDK crate homepages now explain first use, features/targets, explicit
  startup ownership and guide links. Browser docs distinguish static validation
  from actual WASM mounting; GPU remains a separate opt-in renderer.
- View variants, Attribute fields and Element builders/accessors explain owned
  values, append order, conversions and the renderer's validation/escaping role.
- SessionConfig and its errors document actual token-name, truncated whole-second
  TTL and capacity checks, Secure default and explicit installation. Session is
  not authenticated identity or business authorization.
- [The local preview builder](../site/README.md) uses mdBook0.5.4 and eight exact
  canonical source selections. Root/HTTP/DOM excerpts omit their maintainer or
  historical tails. Selected chapters link locally; other source links use the
  committed base on GitHub. Labels distinguish source base/local edits from
  published product evidence. Search is built in; playground execution is off.
- Current selected guides correct stale unpublished/package wording. A retained
  View-guide link to first-app step4 was genuinely broken because that heading
  is step3; only that link changes. This is not a renderer/API change.

The builder exports generated HTML/assets only under ignored `target/docs-site`.
It does not copy arbitrary linked reports, source files, manifests or Markdown.
There is no Pages workflow, public hosting, new runtime dependency or Wiki copy.
Other guide sources not selected for this preview retain their own scope.

## New verification

All Cargo commands use `--offline --locked`. An isolated target directory keeps
the selected native/WASM rustdoc output separate from earlier generated docs.

| Verification | Result and limit |
|---|---|
| `cargo test -p yetlet --features session --doc 'src/lib.rs'` | One new crate example PASS; old unrelated examples filtered out |
| Same command with `view::Element` | One new owned-tree example PASS |
| Same command with `session::SessionConfig::new` | One new configuration/error example PASS |
| `cargo test -p yetlet-browser --doc 'src/lib.rs'` | One new static DOM-validation example PASS; not a real mount |
| `cargo doc --no-deps -p yetlet --features session` | PASS with `RUSTDOCFLAGS='-D rustdoc::broken_intra_doc_links'` |
| `cargo doc --no-deps -p yetlet-browser --target wasm32-unknown-unknown` | PASS with the same link lint; default DOM only, no new GPU execution |
| Explicit preview build | PASS, eight selected source chapters, project base `/yetlet/` |
| Generated internal links/assets/anchors | 454 checked, zero remaining failures after the step-number correction |
| Non-documentation Rust source comparison | All four changed Rust files match HEAD after removing doc-comment lines and blank lines |
| Export inspection | No repository Markdown, Rust source or TOML copied into the book output |

mdBook's official Linux-musl0.5.4 archive is downloaded only into ignored target
tooling, with its GitHub release digest verified as
`5222beabd3e37dc5be0d18ff99b79058469354db5c220153a1b92db5ba12be89`.
The builder itself checks the version and never downloads tools.

Run-local evidence is under `target/yt217-docs-20261031/`; the actual export
manifest is `target/docs-site/EXPORT-MANIFEST.json`. Native and WASM rustdoc
entry pages are in that run's `rustdoc/doc/yetlet/` and
`rustdoc/wasm32-unknown-unknown/doc/yetlet_browser/` respectively. These outputs
are local previews, not hosted docs.rs availability or distribution evidence.

## Retained failures and remaining acceptance

The in-app browser reports that its access host is unavailable. No Browser
plugin is listed. Existing Playwright/installed headless Chrome is used for
local preview inspection, without installing browser dependencies or touching
the owner's existing browser profile.

The parent browser check twice stops at the same harness assertion: expected
eight chapter links, observed twelve DOM matches. Both the original sidebar
selector and a narrower wrapper selector include more nodes than the eight
explicit source chapters. The generated chapter manifest/TOC still lists eight;
the DOM count alone is not evidence of recursive export or a product defect.
Page identity, meaningful content and source/version labels were reached, with
no captured console/HTTP errors before the assertion. Navigation, search,
mobile interaction and screenshots were not reached. The failed harness is
not repaired/retried further after the repeated signature; no full browser
PASS is claimed. Parent command observations and the last failure receipt are
retained; the second receipt uses the same run-local browser evidence location.

The independent review must evaluate implementation/evidence and this explicit
remaining boundary. Keep YT-217 In progress until adequate rendered-reader
acceptance exists. Initial Pages activation/publication is a later separately
authorized step, not a reason to run release or signing workflows now.

## Independent review and retained disposition

Independent Reviewer `/root/yt217_api_docs_preview_review_20261031` first finds
one P2: the selected View guide still claims unpublished/local-dependency setup.
The two contradictory statements are corrected to published SDK0.9.0-alpha.5
and the existing registry-first dependency; only that guide changes in the
reviewed implementation boundary. Its preview is rebuilt and inspected, with
another owning docs check PASS75/217. The initial FAIL and original hashes remain.

Follow-up independent review PASS: zero issues/required followups. All nineteen
source hashes match the updated review manifest. It accepts the source/build
implementation, original four new doctests, selected rustdoc and explicit export
boundary; an independent static inspection checks465 href/src references with
zero failures. The parent's distinct static check counted454 references; these
are different enumerations, not another product validation or a coverage score.
Parent Markdown inspection also checks1,375 local links/anchors with no failures.
Rustfmt on the four selected files, Python syntax and whitespace checks PASS.

The workspace profile is healthy/current/eligible and requests the registered
reviewer with strong-model routing and adaptive high effort. The mandatory
child trace lookup returns `trace_found:false`; model/effort therefore remain
unverified/requested, not runtime-attested. This is ordinary scoped review, not
formal assurance. No reviewer recovery or browser-harness retry is attempted.
Only review/status metadata is updated after that acceptance; no implementation
change follows it. New source/build work is not committed or pushed.

YT-217 remains **In progress**. Complete the still-unverified navigation/search/
mobile reader acceptance before claiming the full item Done. No public Pages
site, hosted workflow, signing or release action is needed for that remaining
local acceptance. The owned loopback preview server is terminated and its port
confirmed closed; both browser scripts close their isolated browser in `finally`.
The generated book remains available for later local inspection using the
commands in `site/README.md`.

## Owner-authorized browser completion — 2026-10-31

After the earlier two stopped assertions, the owner explicitly authorizes the
sidebar-check correction and remaining navigation/search/mobile/screenshot
acceptance. Continue the same QA script and evidence location; keep both old
failures and the script before correction. No fresh run, product rebuild or
replay of the four passing doctests/rustdoc/static checks occurs. All nineteen
prior source hashes match before this continuation.

### Correction and scope

mdBook adds current-page heading links to its sidebar dynamically. At Overview
there are eight chapter links and four same-page fragment links. The old generic
sidebar selectors counted all twelve nodes. Compare the non-fragment same-origin
chapter URL paths against the eight outputs in the existing export manifest;
do not filter to only expected paths or deduplicate an incorrect count. The
actual comparison finds exactly all eight expected paths and excludes four
fragment links. No website or renderer behavior changes to satisfy the check.

Use the existing Playwright/installed headless Chrome and a temporary browser
profile; no Browser plugin is listed and the prior CUA access-host failure is
retained. The already passing initial page-identity/content/version assertions
are reused rather than executed again. This continuation covers only the
previously unreached reader interactions and screenshots:

| New check | Actual result |
|---|---|
| Sidebar chapter selection | PASS: eight exact chapter paths; four page-heading fragments excluded |
| Desktop navigation | PASS: Overview to Create your first app; installed CLI0.1.1 instructions rendered |
| Search | PASS: search SessionConfig, receive an API-chapter result and open that chapter |
| Mobile interaction | PASS at390x844 touch emulation: API heading/code content visible; tap menu, tap Views, verify target heading and closed menu |
| Console and assets | No captured page errors, console errors or HTTP4xx/5xx during these journeys |
| Screenshots | Desktop Overview, desktop API search destination and mobile Views captured; parent visually inspected all three |

Desktop viewport is1440x1000. Mobile is headless Chromium touch/viewport emulation,
not a physical handset, Safari, Firefox, accessibility certification or broader
browser-support qualification. Dense API tables use mdBook's normal presentation;
no theme redesign or new framework feature is selected.

### Evidence and cleanup

The original local QA script exits0 and closes its isolated browser in `finally`;
the separate mobile context closes explicitly. The owned loopback-only server is
terminated after inspection, with closed listener confirmed. No owner's browser
profile, existing service, external hosting or release environment changes.

The run-local receipts are `validation/browser-authorized-sidebar-fix.log`,
`validation/browser-acceptance-evidence.json` and
`validation/cleanup-browser-acceptance.json` under the existing YT-217 evidence
root. Screenshots, corrected/prior script, original failure receipts and the new
result JSON are also archived outside the repository as evidence set
`yt217-docs-preview-20261031`; copy hashes match. They are not site exports.
Original STOPPED/FAIL records and scoped source/build review remain intact.

Final independent review is Pending at this checkpoint. Mark the item Done
only after accepting this bounded reader evidence; initial Pages publication
remains separately authorized future work. No new Commit/Push is authorized.

## Final independent acceptance — 2026-10-31

Independent Reviewer `/root/yt217_api_docs_preview_review_20261031` accepts the
owner-authorized browser continuation with PASS, zero issues/required followups,
and explicit `can_mark_done:true` disposition. It inspects the corrected/prior
scripts, successful log/result, all three screenshots, eight matching evidence
file hashes in temporary/durable stores, all nineteen source hashes and eight
unchanged chapter-source bindings. It confirms the direct URL comparison,
actual navigation/search/touch interactions and cleanup. No browser or old
source/build tests are repeated by the review.

The current canonical task-path trace lookup succeeds: registered reviewer and
configured `gpt-6-astra` match; effective effort is `high` and matches the adaptive
resolver decision. The earlier unavailable trace receipt is retained as history;
this acceptance does not relabel that earlier lookup. The observed-effort
resolver remains non-degraded and conflict-free. This is scoped independent
review, not formal assurance or expanded platform support.

YT-217 is now **Done** for the selected doc comments, explicit local static guide
and bounded Chromium desktop/touch-emulated reader acceptance. Initial failures,
first source-review P2/correction, earlier partial reviews and consumed attempts
remain unchanged. Only completion/review metadata changes after final review.
No runtime, API, package version or feature changes follow. New files remain
uncommitted/unpushed; Pages/Wiki, signing and release workflows are not activated.
The next possible step is a separately authorized public-site preview acceptance
and Pages publication decision; this completion does not authorize it.
