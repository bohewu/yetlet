# M25 — Broader HTML DOM support and static-site delivery

Date: 2026-10-31. Status: reviewed plan; independent Reviewer PASS0.
Source baseline: `c92c56af0a4dd7fd34a9bf809d6c6919d017b4ad`.
At this reviewed planning checkpoint all six items were **Todo**. This batch
plans work only; it does not
implement a renderer, CLI command, new package, sample or website replacement.

## Purpose and authority

The owner requests several independently reviewable items, prioritizing broader
DOM support before a Yetlet-built documentation site. Aim to cover ordinary
modern HTML comprehensively; classify unsuitable or specialized elements
explicitly instead of accepting arbitrary tag strings. A static site is also a
useful real consumer of Yetlet's HTML-first architecture and optional Wasm.

Preserve [renderer separation](adr/0003-renderers-and-loading.md),
[owned keyed/event lifetimes](adr/0007-bounded-keyed-dom-and-local-graph.md),
[current DOM guarantees](YT-110-DOM-CAPABILITIES.md),
[naming](NAMING.md) and [opt-in composition](EXTENSIONS.md). This plan selects
neither a general VDOM/hydration runtime nor a Markdown/CMS subsystem in Core.
Existing SDK0.9.0-alpha.5, CLI0.1.1, release evidence and the live
[YT-218 guide](GITHUB-PAGES-PUBLICATION.md) remain at their accepted scope.
Implementation, Git delivery and public-site replacement are later steps, not
consequences of accepting this planning review.

## Actual starting point

| Surface | Current behavior | Gap relevant to this plan |
|---|---|---|
| [Shared View](../crates/yetlet/src/view/mod.rs) | Owned text/elements, static names, renderer-owned validation | No namespace or trusted raw-HTML variant; keep these boundaries |
| [HTML renderer](../crates/yetlet/src/html/mod.rs) | Escaped serialization, forbidden executable tags/attributes, URL/void/control checks | Broader than DOM; not a full HTML validator or arbitrary-markup sanitizer |
| [DOM renderer](../crates/yetlet-browser/src/dom/mod.rs) |38 explicit tags/42 attribute names; owned subtree replacement; prevalidation; same-origin root-relative/fragment navigation with frozen browser-resolved URLs | Missing normal content/table elements and controlled external links/images |
| [Browser CLI starter](../tools/yetlet-cli/templates/browser/README.md) | `new --template browser`, settings-selected `build`/`run`/`watch`; ship only `frontend/dist/` | Already supports static hosting of a browser-only Wasm app; not build-time HTML generation |
| [CLI command inventory](../tools/yetlet-cli/README.md#current-command-and-capability-inventory) | `generate docker/compose/k8s` writes deployment configuration | No `generate static`, SSG mode or finite multi-page export contract |
| [Guide selection](../site/pages.toml) and [builder](../tools/build_docs.py) | Eight selected Markdown sources/excerpts; mdBook navigation/search; source/version labels | Reuse content/disclosure boundaries; a Yetlet replacement is not implemented |

Do not confuse static hosting with static HTML generation: the existing browser
app mounts after Wasm loads. SSG renders public Views during the build and ships
readable HTML. SSG can reuse the HTML renderer without the browser renderer;
DOM completion is a priority for the eventual interactive consumer, not a
technical prerequisite for serializing HTML.

## DOM coverage policy

Use the [WHATWG conforming-element index](https://html.spec.whatwg.org/multipage/indices.html#elements-3)
and [HTML element/content-model definitions](https://html.spec.whatwg.org/multipage/dom.html#elements)
as the inventory reference, observed2026-10-31. YT-219 must classify **every
entry in its recorded standards snapshot**, including entries not illustrated
below. Record source date/identity; future standards additions are not silently
accepted. A grouped target list here is not a completed conformance matrix.

| Class | Planned disposition | Examples and ownership |
|---|---|---|
| Ordinary semantic/text/list content | Broad support in YT-220, including currently missing families | `address`, `hgroup`, `search`, `pre`, `blockquote`, `figure`, `figcaption`, `abbr`, `b`, `i`, `u`, `s`, `mark`, `sub`, `sup`, `kbd`, `samp`, `var`, `cite`, `q`, `dfn`, `data`, `time`, `bdi`, `bdo`, `ruby`, `rt`, `rp`, `ins`, `del`, `wbr`, `menu` |
| Tables | Support in YT-220 with explicit structure/attributes | `table`, `caption`, `colgroup`, `col`, `thead`, `tbody`, `tfoot`, `tr`, `th`, `td`; do not rely on HTML parser repair |
| Native forms and disclosure | Broaden in YT-220 | Existing controls plus `datalist`, `optgroup`, `output`, `progress`, `meter`, `details`, `summary`, `dialog`; native initial state and app-owned live methods/events |
| Canvas host | Element/fallback content only in YT-220 | `canvas` creation does not acquire a context, start an animation or select WebGPU |
| Navigation and images | Explicit controlled support in YT-221 | Existing `a`/form navigation, external HTTPS links, `img` and `picture`/`source` image selection; resources need a URL contract |
| Audio/video, tracks and image maps | Deferred with explicit matrix reasons | `audio`, `video`, media `source`, `track`, `map`, `area`: separate loading/URL/accessibility behavior and a concrete consumer are needed; not permanently forbidden |
| Document metadata/root | Outside owned-subtree mounting | `html`, `head`, `body`, `title`, `base`, `meta`, `link`: initial document/asset/head writer owns them; do not mutate global base/CSP through a mounted View |
| Executable/embedded content | Remain rejected in the general View DOM path | `script`, `style`, `iframe`, `object`, `embed`, inline handlers/styles and `srcdoc`; trusted bootstrap/CSS stay separate |
| Special parsing/composition or non-HTML namespaces | Deferred or explicitly unsupported after YT-219 analysis | `template`, `slot`, `noscript`, specialized selectable-content elements, custom elements, SVG/MathML roots; names alone cannot implement their behavior |
| Obsolete/nonconforming elements | Excluded | No compatibility aliases solely to imitate legacy markup |

Element admission does not imply every attribute, browser API, nesting or native
behavior is implemented. The matrix must distinguish tag, applicable attribute,
value/URL policy, initial state, live property/method, event ownership and
renderer/target availability. Keep ordinary CSS in external stylesheets and
application behavior in explicit listeners/native browser APIs.

## Items and dependency order

| Item | Deliverable | Dependencies | Completion boundary |
|---|---|---|---|
| YT-219 | HTML/DOM coverage and compatibility contract | YT-110 | Reviewed inventory, selected batches and explicit unsupported/deferred reasons; docs only |
| YT-220 | Broad semantic, table and native-control DOM vocabulary | YT-219 | Selected vocabulary/attributes/structure implemented with focused native and real DOM evidence |
| YT-221 | Controlled navigation and image-resource DOM support | YT-219, YT-220 | Reviewed typed policy, browser-resolved URLs and minimal external-link/image consumer |
| YT-222 | Static HTML export and CLI contract | YT-219 | Reviewed finite public page/asset/build contract and canonical CLI naming; docs only |
| YT-223 | Minimal opt-in static HTML build and CLI integration | YT-222 | Small multi-page no-Wasm output, optional island packaging and external consumer proof |
| YT-224 | Yetlet-built documentation-site pilot | YT-218, YT-220, YT-221, YT-223 | Local isolated pilot using the existing selected sources; no automatic live-site replacement |

Start YT-219 next. YT-222 may follow its reviewed renderer boundary independently
of DOM implementation; YT-224 integrates both paths. Each later implementation
starts with its own selected contract and ends with independent Reviewer PASS
before Done. Completion of this plan does not mark any of these items Done.

### YT-219 — Coverage, attributes and compatibility contract

Inventory every standard element against both actual renderers and current
consumers. Assign existing/selected/deferred/rejected/document-only status and
reason; inventory scoped attributes separately. Prioritize comprehensive normal
body content rather than a documentation-only allowlist. Resolve minimum
structure checks for tables, disclosure, select/datalist/optgroup and other
browser-normalized shapes. Address `pre` leading-newline parsing, adjacent text
nodes, boolean/enumerated attributes and serialization versus DOM construction.

Decide public typed options/policy names and default behavior before adding API.
Keep existing `validate`/`mount` entry points usable and existing strict URL
defaults intact; newly admitted inert tags are an intentional additive capability.
Any unavoidable source-breaking change needs an explicit migration decision.
Do not make broader SSR admission imply identical DOM policy or tighten existing
SSR globally without a specific compatibility requirement. State supported
structural parity precisely; this is not full HTML content-model conformance.

Acceptance: source-backed element/attribute/status matrix, named batches, default
and compatibility decisions, touched-consumer list, focused future checks,
documentation checks and independent review. No browser or old suite replay is
needed merely to inventory source.

### YT-220 — Ordinary HTML, tables and native controls

Implement the YT-219 selected ordinary content families, applicable inert
attributes and bounded structure validation, reusing the existing View model
and browser DOM creation. Define scoped semantic attributes such as table
headers/spans, date/time values, disclosure/control state and numeric ranges
where selected. Do not admit URL-bearing attributes through a generic attribute
expansion. Reuse existing binding-name rules where applicable; reserved ownership
markers, executable attributes and unreviewed `data-*`/`aria-*` expansion remain
excluded. Preserve accepted attributes and their existing presence semantics.

Check the selected shared subset through HTML serialization/browser parsing and
direct DOM construction, including table sections and `pre` text handling.
Reject selected invalid shapes before replacing the owned target. Keep focus,
keyed identity, live input properties, modal dialog methods and listener lifetimes
at their existing explicit ownership boundaries; no automatic modal/focus/form
runtime follows from admitting an element.

Acceptance: focused changed-validator/serializer cases; native/Wasm compilation;
real browser evidence for the new content, table and disclosure/control cases;
unchanged-target rejection and text escaping; selected consumer integration and
updated public capability docs. Reuse unrelated historical suites; cross-browser
checks cover only changed parser/native-element behavior, not all old samples.

### YT-221 — Navigation and image URL policies

Select an explicit typed opt-in policy for external navigation and image
resources. Preserve strict behavior through existing entry points. Distinguish
link destination, form submission and resource fetch; accepting external links
must not silently permit external form actions or arbitrary resource origins.
Use the browser's [URL parser/origin model](https://url.spec.whatwg.org/), resolve
against the effective document base and freeze accepted destinations.

Define scheme/credential/origin rules, root/subpath/fragment handling, external
HTTPS origins, `_blank`/`rel` behavior and omitted/default values. Keep `ping`,
script/data URLs and unselected schemes excluded. Images need `alt` (empty for
decoration is valid), selected dimensions/loading attributes and an explicit
same-origin default or named origin allowlist. `srcset` candidates and `sizes`
need their own grammar; if the minimal implementation cannot validate them,
defer `picture`/responsive source admission explicitly rather than trusting raw
strings. Media `source` is not enabled by admitting an image source.

Preflight all selected URL values before assigning resource attributes or
replacing the target, so a later invalid node cannot trigger an earlier denied
fetch. Pure validation performs no network request. Document that an admitted
resource may fetch before final insertion and that browser/network failures
are not transactional rollback. CSP remains an additional host restriction,
not a substitute for the renderer's policy. Keep server secrets outside Views.

Acceptance: typed defaults and compatibility docs; focused hostile URL/base/
credential/attribute cases; actual same-origin and explicitly allowed external
links, image load/failure and rejection-without-denied-request evidence; preserved
form policy, owned target and escaped text; independent review. No live third-
party service is needed: use isolated test origins for changed behavior.

### YT-222 — Static HTML build contract

Prefer **`yetlet build --static`** as a proposed build mode; it is **not an
existing command**. Confirm the final spelling, help/settings precedence and
error behavior in this item. `generate` retains its configuration-only meaning.
The distinction between initialization and building is also used by
[Zola](https://www.getzola.org/documentation/getting-started/cli-usage/#build);
reuse Yetlet's existing `new`/`build` vocabulary instead of adding aliases.

Define an explicit application-owned native export entry and finite public
page list, using existing View/HTML serialization. The CLI builds/invokes that
entry through the existing frozen process/ownership seams; it does not crawl
Axum handlers, infer arbitrary routes, run a request server or export sessions,
authenticated output or request-time business data. No mandatory HTTP server,
Settings provider, DI, browser dependency or Node tooling for HTML-only export.
Decide the smallest page/head/CSP metadata shape and reuse existing suitable
types without importing the server dependency graph just for metadata.

The contract must settle:

- Canonical public routes, output-file mapping, index/404 semantics, base prefix
  (including `/yetlet/`), local links/anchors and direct refresh without SPA rewrites.
- Explicit public assets, output ownership, duplicate/case-folded route collisions,
  path traversal, symlinks, stale files and failure/last-good publication behavior.
  Never delete or overwrite an unrelated directory to obtain a clean build.
- Frozen Rust/build inputs, `--release` semantics, bounded owned subprocess
  cleanup and source/version/input/output manifest receipts outside public content.
- Build-only public configuration and secret exclusion. Executing an app's
  export code is explicit trusted-code execution, not a sandbox or proof that
  arbitrary code cannot read credentials; no server environment is serialized.
- HTML-only output and separately selected Wasm islands. Island mount roots and
  readiness/failure use the [existing shell](WASM-STARTUP.md); preserve readable
  surrounding HTML, without inventing hydration or claiming zero loader JS.

Acceptance: reviewed finite protocol, proposed CLI/typed options, asset/CSP and
error contract, no-backend output example and bounded new verification list.
No runtime implementation or package/version change in this contract item.

### YT-223 — Minimal static build and CLI delivery

Implement only the reviewed export entry/protocol and CLI mode, using existing
HTML rendering and process/filesystem safeguards. Exercise a small independent
consumer with home, nested page, links/anchors and one local stylesheet/image.
Keep its HTML-only build free of required Wasm/Node/server dependencies. Add one
separately selected interactive island only after the static result is sound;
reuse existing browser build/manifest integration instead of a second bundler.

Acceptance: focused CLI/help/options/error and output-boundary tests; actual
standalone consumer build, independently hosted dist-only HTML with JavaScript
disabled, nested direct navigation at root/project prefix, CSP/MIME/assets and
owned cleanup. Check only touched GNU/MSVC process/filesystem behavior, optional
island lifecycle and package/external-consumer inputs; retain prior passes and
failures. No implicit install, network data fetch, deploy, tag or registry upload.

### YT-224 — Application-owned documentation pilot

Use the existing eight selected Markdown inputs/excerpts as the initial content
boundary, keeping one canonical source. Parse Markdown at build time with an
existing maintained parser if needed; do not implement a parser or put a mandatory
Markdown dependency into the SDK. Disable/reject raw HTML by default and handle
unsupported constructs explicitly rather than silently dropping content. Map
code, tables and links through reviewed rendering rules; syntax highlighting
must not introduce a raw-HTML bypass.

Build the shell/navigation/styles with Yetlet and retain source/SDK/CLI labels.
Keep complete static content readable before or without Wasm. Add optional
Wasm search, menu or interactive documentation examples only where they provide
an actual consumer. Mount into explicitly owned surfaces; do not claim general
hydration or take over unrelated static content. A comparison browser-only mode
can be considered later, but is not required for this static-first pilot.

Acceptance: chapter/excerpt/link/disclosure correspondence; source/edition and
artifact exclusion checks; no-JS reading and real search/menu/touch/keyboard
journeys; bounded cold-load/asset-size comparison with existing accepted mdBook
evidence, not a new framework-wide benchmark. Preserve stopped/failed results
and reuse unchanged content proof. Propose deployment replacement only after
independent review and explicit owner acceptance. YT-218's current workflow and
public website continue to serve the accepted guide during the pilot.

## Planning acceptance and execution limits

For this batch, change only this plan, Backlog/Roadmap and maintainer navigation.
Run documentation/link/structure/DAG and whitespace checks, then obtain an
independent Reviewer verdict on scope, dependencies, safety, compatibility and
honest current capability claims. Do not run old SDK/browser/load/release suites.
Keep all six items Todo after planning acceptance. No Commit, Push, new API,
package/dependency, hosted job or Pages setting/public-content change is selected.

Independent planning review: **PASS0**, zero issues/required followups. All five
planning-document hashes matched at acceptance; registered reviewer model
`gpt-6-astra` and effective high effort were verified. Evidence is retained under
`target/yt219224-plan-20261031/`. This accepts planning only; all six items remain
Todo at that checkpoint. Next selected item was **YT-219**, contract only.

## Subsequent YT-219 contract checkpoint — 2026-10-31

The owner separately authorizes committing these five planning documents with
strict/builtin (local d1b8ac5, zero findings, no Push), then source/standards
inspection for [YT-219](YT-219-HTML-DOM-CONTRACT.md). Its
[complete inventory](YT-219-HTML-INVENTORY.md) selects90 target named HTML
elements:38 current,51 ordinary additions and an opt-in img. Picture/source
remain deferred until responsive/media grammar is selected. The contract freezes
boundaries, attribute scope, typed option names/defaults and limited SSR/DOM
parity; no implementation follows from this documentation-only item.
YT-219 is Done at documentation/source-contract scope after independent Reviewer
PASS0, zero issues/followups and verified role/model/effective high effort.
YT-220–224 remain Todo; later implementation needs its own authority.
The original planning PASS and source baseline above remain historical evidence.

## Subsequent YT-220 implementation checkpoint — 2026-10-31

After strict/builtin commit c52ba85 of the accepted YT-219 documents (no Push),
the owner selects [YT-220](YT-220-HTML-DOM-IMPLEMENTATION.md). The unreleased
checkout implements all51 ordinary additions and selected attributes/structures,
pre parity and a shared Todo consumer. Focused new native/Wasm/Chrome/Firefox
checks pass with original lint/fixture failures retained and owned cleanup.
YT-220 is Done at selected unreleased-source scope after independent Reviewer
PASS0, zero issues/followups and16 stable inputs with registered model/effective
high effort verified. YT-221–224 remain Todo. No package/dependency/version, external navigation/image
policy or live-site replacement is included; earlier checkpoints retain their
own states and evidence.

## Subsequent YT-221 implementation checkpoint — 2026-10-31

The owner selects strict/builtin commit4b88f3f of YT-220's16 paths without Push,
then contract-first [controlled links/images](YT-221-DOM-URL-IMAGE-CONTRACT.md).
Contract review PASS0 precedes minimal per-mount typed options and the separate
Todo sample. Defaults stay strict; named HTTPS link/image origins are independent,
forms remain same-origin, and browser preflight freezes all destinations before
resource assignment. Focused new native/Wasm/Chrome/Firefox evidence passes;
first contract P2 and Firefox popup-navigation assertion failure are retained.
Final independent Reviewer PASS0,0 issues/followups and23 stable input hashes
complete YT-221 at selected unreleased-source scope; registered model/effective
high verified. YT-222–224 remain Todo. No new package/dependency/version, Git
delivery or live-site change; original checkpoints and failures remain historical.

## Subsequent YT-222 contract checkpoint — 2026-10-31

The owner continues with [YT-222's finite contract](YT-222-STATIC-HTML-CONTRACT.md),
preserving accepted YT-221 code/contract/evidence without Commit/Push. This selects
the future build --static spelling, typed CLI request, native app-owned export
protocol1, public fragment/asset inventory, small private document writer and
separate prepared islands. HTML-only apps need no server/browser/Settings/Node;
output is protected and create-only, with prefix/index/404/CSP and private receipt
boundaries. Existing settings validation remains and no new settings wire is
required. Documentation-only inspection and checks PASS; independent Reviewer
PASS0,0 issues/followups,21 matching inputs and verified model/effective high.
YT-222 Done at contract scope, YT-223/224 Todo; no implementation, old verification replay or live
site/release change. Earlier checkpoints retain their own dates/status/evidence.
