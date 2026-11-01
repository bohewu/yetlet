# YT-224 — Local Yetlet documentation pilot

Date: 2026-11-01. Status: Done at local pilot scope after independent Reviewer PASS0.
Source baseline1802026ea52d541527dbad7cf13f652c416c5d50 is the authorized strict/
builtin ten-document YT-223 closeout commit, treeb8a4cc4b0d68063b47b66af650e1febdf1de21d6.
Guard and normal hook pass, zero blocking findings, no Push. YT-224 work is
uncommitted/unpushed; no Merge, release, tag, hosted job or Pages replacement.

## Selected contract and implementation

The owner selects the [reviewed M25 pilot](YT-219-224-DOM-STATIC-SITE-PLAN.md#yt-224--application-owned-documentation-pilot).
[examples/docs-site](../examples/docs-site/README.md) is a publish=false standalone
application using registry SDK0.9.0-alpha.5 and the accepted source static-export
CLI. No framework, CLI, root dependency/lock, version, site selection or workflow
changes are made. Existing Pages continues to serve the accepted mdBook guide.

Preparation reads the same eight site/pages.toml sources, exclusions and excerpt
markers, reusing the existing link projector. All eight source hashes match the
retained mdBook build; content is not manually forked. Ignored generated input
contains those source/excerpt hashes and the content commit. The exporter maps
maintained [pulldown-cmark0.13.6 events](https://docs.rs/pulldown-cmark/0.13.6/pulldown_cmark/)
to Yetlet Views. The parser stays in the native app graph. It never supplies an
HTML fragment directly: text/code/metadata are escaped by the existing renderer.
Raw HTML, images, selected unsupported features and aligned tables fail explicitly;
ordinary tables own explicit thead/tr/tbody structure. There is no custom Markdown
parser, raw-HTML highlighting, SDK parser feature or generalized document engine.

The application owns a responsive shell, eight chapter links, active-page and
edition/content-source labels, semantic headings/code/tables, skip link and native
mobile details menu. A separately selected Wasm search mounts only its input/
results host, loads after explicit activation and leaves all chapter HTML intact.
It uses ordinary local substring search over eight public texts, not a search
service. The existing startup helper/build algorithm is copied deliberately;
Wasm library/output names are the only build-script changes.

This local pilot fixes /yetlet/, uses no fonts/images/remote resources, and makes
no new source-preview public edition. Pinned external repository links may be
unavailable until this local content commit is pushed; the page explicitly says
so. This is separate from internal chapter/anchor correspondence. There is no
hydration, general browser-only rendering mode or Pages promotion in this item.

## Original acceptance verification (before the startup amendment)

| Evidence | Actual scope/result |
|---|---|
| Canonical content | Same8 sources/excerpts and labels; generated input hash binding. Existing mdBook build/reader proof retained, not replayed |
| Native tests |3 PASS: raw HTML/active URL rejection, escaped code, Unicode/duplicate heading anchors, explicit semantic tables, unsupported content and plain search text |
| Compile/lint | Native exporter check, strict native all-targets Clippy, release wasm build with matching bindgen, strict wasm lib Clippy and format PASS |
| Static exports | Protected copied consumer; HTML-only and selected island outputs PASS using accepted YT-223 CLI; existing CLI tests/platform/browser proof reused |
| Output boundary |8 pages each;219 HTML-only and227 island local links/anchors checked; unique IDs; no source, Cargo, Markdown, manifests or source maps in public output |
| No-JS reading | All8 article texts match parsed canonical excerpts;8 desktop chapter links, current-page/edition labels and no script; keyboard skip/chapter navigation PASS |
| Real browser interactions | Existing Playwright/Chrome,1440×1000 desktop and390×844 touch emulation; no Browser plugin available. Explicit actual-Wasm search, focus, matches/empty/no-hit/result navigation; native mobile menu and no viewport overflow PASS |
| Failure/lifetime | Aborted Wasm preserves article/nav with explicit reload, one attempt/no retry. Actual pagehide callback retires the search listener. Existing startup/bfcache suites not replayed |
| Visual proof | Desktop Overview, activated search results and mobile Views screenshots inspected by the parent; no blank/framework overlay or relevant normal-flow console errors |

The flow is local guide → chapter/anchor reading → explicit search → result
navigation, plus mobile menu/touch and keyboard equivalents. Failure guidance
and owner disposal are verified separately. This is Chromium/viewport-emulation
evidence, not Firefox, MSVC, physical-mobile, assistive-technology or new release
qualification. No unchanged SDK/browser/CLI suite or old docs build is rerun.

## Historical activation-only load comparison

The already-built mdBook artifact has matching eight source hashes and is reused.
A single previously unmeasured cold navigation captures the two local generated
outputs on the same host/browser with fresh contexts; no old mdBook interaction
suite is repeated. Plain HTTP does not enable compression. Values are descriptive
one-sample reader costs, not a throughput benchmark or production speed guarantee.

| Metric | Yetlet pilot | Existing mdBook artifact |
|---|---|---|
| Complete selected-island artifact |403,048 bytes /15 files |1,831,469 bytes /43 files |
| HTML-only pilot artifact |129,506 bytes /10 files |Not a new mdBook configuration |
| Initial encoded response bodies, before activation |19,107 bytes |532,530 bytes |
| Observed DOMContentLoaded / load |21.6 /21.8ms |62.0 /62.7ms |

The initial pilot makes zero Wasm requests; activating search makes one. Full
search assets remain included in the larger total-artifact figure. No repeated
timing campaign, capacity claim, mdBook upgrade or framework optimization follows.

## Retained failures and cleanup

1. First native compile used a String heading name where the published Element
   API requires a static name. Fixed literal h1–h6 mapping; tests had not run.
2. The first checkout export is correctly refused before child/output because
   its ancestor is group-writable. Use a private copied consumer; do not weaken
   admission or change repository/ancestor permissions.
3. The first browser attempt passes all8 no-JS article/label/navigation cases and
   real keyboard actions, then fails a wrapper assertion expecting cargo install
   in First App. That page links to installation instead. One minimal wrapper
   correction checks the actual canonical heading and resumes only the remaining
   work; earlier successes/screenshots are retained without replay.

Full first logs/scripts/results, source/input/output hashes and screenshots are
retained outside tracked source in run yt224-20261101. Generated Cargo/Wasm input
and output are excluded by the app's .gitignore. Final owned browser/server/
temporary-directory cleanup and independent review are recorded at closeout.
No deployment replacement is selected; owner review of this local pilot is the
next decision before any separately authorized live-site proposal.

## Final independent acceptance and cleanup

Independent Reviewer PASS0, zero issues/required followups, verifies all33 review
inputs and the19-path status snapshot. Actual consumer sources/output bindings,
canonical excerpts, local links/CSP/exclusions and retained native/browser evidence
match. Registered reviewer gpt-6-astra/effective high effort are verified. The
reviewer made no edits or test/browser reruns.

All retained temporary consumer files are compared with the external archive
before removal; Cargo object-cache files alone are excluded. The owned temporary
consumer is removed after that proof. Browser and three loopback servers are
closed, with port absence observed. No live Pages, workflow, global tooling or
existing service is changed. All19 new paths stay unstaged/uncommitted; HEAD
remains1802026 and that ten-document commit is not pushed. YT-224 is Done only at
local pilot scope. Inspect this local result before any separately authorized
Pages-replacement decision; no promotion is implied by acceptance.

## Owner-selected automatic search startup — 2026-11-01

The owner requests search ON by default for this reader. Build-time selection
remains explicit through --wasm-islands; within that selected documentation site,
search now initializes automatically on page load. The Enable search button is
removed. The current app loads its Wasm/index as part of startup, so the earlier
activation-only initial-load figures above remain historical and are not current
performance claims. No new timing campaign or old suite replay is performed.

The owned search runtime no longer focuses the input during initialization; it
therefore leaves the reader at the article instead of jumping to the footer.
Loading/error guidance, complete no-JS content, no automatic retry and the
pagehide/late-completion guard remain. The framework defaults, CLI flags, CSP,
settings, dependencies, canonical content and existing Pages are unchanged.

Only affected native export, Wasm build/strict lint/format and startup browser
flows are checked. Original parser, chapter-correspondence and menu/keyboard proof
is reused. A formatting-only check failure is retained and corrected without
changing the already-exported semantics. Current local preview replacement and
independent review of this amendment are recorded after its focused checks.
No Commit/Push or public Pages update is authorized for this amendment.

Focused automatic-startup amendment verification and independent Reviewer PASS0
are complete: zero issues/required followups, five matching review inputs and
fifteen matching public files. Native/Wasm strict lint/build, corrected formatting
and docs pass. Chromium verifies automatic loading/ready without a button, one
Wasm request, unchanged initial focus/scroll, actual search, failed-load static
fallback/no retry, no-JS guidance and disposal during initialization. Original
evidence is reused. Temporary browser/test server/consumer are cleaned up after
verified external retention. The user-requested loopback preview is updated to
this new artifact, while original artifacts and public Pages remain unchanged.
These five UX-amendment paths remain uncommitted/unpushed.

The owner's subsequent compact-search copy request removes visible ready text,
the duplicate field label and the empty-query prompt. The idle panel shows only
Search and its placeholder; the field retains its explicit aria-label. Loading,
failure/reload and no-results messages appear only when relevant. New native/
Wasm builds and focused browser checks pass for visible copy, accessible name,
results/clear/no-hit and failure fallback. Previous artifacts remain retained;
the user-requested loopback preview is updated, with no Commit/Push or Pages
change. Temporary build consumer and test browsers are removed/closed.

## Owner-selected header search — 2026-11-01

The selected island build now places the search field directly in the header.
The native HTML input accepts typing before Wasm finishes loading; activation
preserves the same node, query and focus, then renders only the owned results
container. Matching chapters appear as whole-row links with concise excerpts.
Native Tab/Enter navigation, Escape, outside/blur dismissal and clearing the
query are supported. Mobile layout gives the field its own header row. The
Local pilot badge is removed; source/version disclosure remains in the edition
line. The HTML-only build omits search and keeps all eight readable chapters.

New native export, native/Wasm strict Clippy and the Wasm build pass. Focused
Chromium checks pass at1440x1000 and390x844 for early input preservation,
keyboard/native-link navigation, dropdown dismissal, viewport bounds and the
mobile chapter menu. Failed-load and no-JS guidance preserve readable content.
Desktop and touch-emulated screenshots are inspected. The first browser wrapper
incorrectly expects a result link's accessible name to contain only its title,
although its visible excerpt is also part of the name. That retained harness
failure receives one title-within-link correction; no product change is needed.
Earlier accepted chapter/parser/platform tests and timing evidence are reused.

Independent review and a focused first-Tab observation catch the sticky header
covering the skip link. Raising only the skip link's stacking level restores its
existing keyboard path. The initial Wasm strict Clippy failure for an unnecessary
borrow is retained; removing that borrow passes the same focused check. No SDK
API or search behavior changes as part of these corrections.

Evidence is retained outside the repository in run
yt224-header-search-20261101. This is a local UX amendment; it does not replace
Pages, add dependencies or authorize Commit/Push.

Final independent Reviewer PASS0 confirms both corrections, six source/status
bindings and the focused browser evidence. The final15-file preview is an asset
update of the successful header-search export: rebuilt JS/Wasm are byte-identical,
and only the corrected CSS differs. Two subsequent CLI export attempts stopped
at an absent output-parent admission check; both failures remain retained, with
no third attempt or new export-success claim. The temporary consumer is retained
externally before removal; test browsers are closed. Only the owner-requested
loopback preview remains running. All six UX-amendment paths remain uncommitted.
