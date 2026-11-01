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

## Changed behavior and verification

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

## Bounded load comparison

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
