# YT-109 — DOM capability and authoring gap review

Date: 2026-10-21. Status: Done at research-only scope; independent Reviewer PASS.
Source: 096f8db1e2c5b0f0fd1e664e776503cc21a74727 plus the accepted, uncommitted
[YT-108 hidden-attribute slice](YT-108-HIDDEN-ATTRIBUTE.md). Inspection uses the
current checkout, not the earlier release candidate.

## Authority and recommendation

The owner asked whether yetlet-dom should become more capable, then selected
this capability/authoring review. This authorizes research and documentation,
not implementation of every gap discussed. Preserve YT-108 and YT-107 acceptance,
all existing evidence, the current branch and the separate YT-096/YT-020 gates.
No commit, push, dependency installation, browser launch or runtime change is
part of this review.

**Improve the supported shared-View vocabulary and its predictable behavior;
do not grow a second application runtime.** The useful next step is a bounded
common-subset contract, followed by deliberately selected native-control and
URL capabilities. Existing direct web-sys bindings, keyed ownership and the
local graph remain valid ways to build richer applications.

The recommendations below are ordered candidates, not newly approved APIs or
registered implementation tasks. Research Done would close this comparison
only. It does not certify the proposed behavior or select a release source.

## Current capability and responsibility map

The exact current DOM allowlist has **15 tags and 12 attributes**:

- Tags: main, section, div, article, header, h1, h2, p, span, ul, li, button,
  strong, input and label.
- Attributes: id, class, hidden, type, data-id, data-completed, aria-live,
  aria-pressed, for, placeholder, maxlength and autocomplete.

These are source counts, not browser-compatibility or performance measurements.
See [DOM validation and construction](../crates/yetlet-dom/src/lib.rs).

| Surface | Implemented responsibility | Boundary to retain |
| --- | --- | --- |
| [yetlet-core View/Element](../crates/yetlet-core/src/lib.rs) | Text/element tree; static tag/attribute names, owned string values and children | No event handlers, live-control properties, renderer scheduler or DOM namespace model |
| [Core binding vocabulary](../crates/yetlet-core/src/binding.rs) | Validated keys/slots; InertAttributeName accepts class/title and bounded lowercase data-/aria- names, excluding data-yetlet-* | This is a dynamic-slot contract, not the existing View renderer's allowlist or proof of accessibility/third-party-script safety |
| [yetlet-html](../crates/yetlet-html/src/lib.rs) | Escaped text/attributes, identifier checks, forbidden markup checks, limited URL-scheme rejection and void-child rejection | Broader SSR vocabulary; not an arbitrary-HTML sanitizer or complete HTML-conformance engine |
| [yetlet-dom](../crates/yetlet-dom/src/lib.rs) | Recursively validate the finite View subset, build detached nodes, then replace an owned target's contents | No keyed diff, hydration, live-property synchronization or automatic event/listener ownership |
| [yetlet-dom-keyed](../crates/yetlet-dom-keyed/README.md) | Keyed child identity, explicit create/update callbacks and owned event listeners | Caller owns created markup, property updates and callback atomicity; final DOM operations do not roll back |
| [yetlet-reactive](../crates/yetlet-reactive/README.md) | Optional synchronous message/reducer/commit graph and weak state-read/dispatch handles | No implicit DOM dependency, effects, HTTP or form state machine |
| Application and JS component bindings | Draft policy, live properties, focus, HTTP/CSRF, feedback, chart/editor subtrees | One owner per subtree; no migration into yetlet-dom solely to shorten application code |

This preserves [ADR 0003](adr/0003-renderers-and-loading.md) and
[ADR 0007](adr/0007-bounded-keyed-dom-and-local-graph.md). SSR and DOM need not
accept every same element. Their intentionally overlapping subset should have
explicit, testable semantics.

## Real consumer evidence

| Consumer and inspected seam | What the application actually needs | What it does not prove |
| --- | --- | --- |
| [Todo Browser](../examples/todo-browser/src/lib.rs), complete_browser_page_obeys_dom_policy, RemoteControls and title_feedback | A full shared View, static initial hidden feedback, then explicit disabled/ARIA/visibility mutations; typed remote action and lifetime checks | That those mutations should be replaced with automatic binding or a generic forms runtime |
| [Admin native triage View](../examples/lab-operations-board/src/triage.rs), control/select/status_summary; [browser binding](../examples/board-triage/src/browser.rs), row_create/set_control/Dom::render | SSR textarea/select/options, aria-describedby/label/busy/invalid, data-select/title/meta and fixed app-generated links; guarded value writes and per-draft policy | A current failure to render Admin: its editor already exists as SSR HTML and is updated through web-sys, not mounted through yetlet-dom |
| [Browser Tasks](../examples/browser-tasks/src/lib.rs), create_row/update_row and submit/change callbacks | Uses yetlet-dom for each row, sets checkbox checkedness and edit labels through direct properties/attributes, reads text on submit and checkedness on change | A need to synchronize every keystroke or replace its native-draft ownership |
| [Search Island](../examples/search-island/src/lib.rs) | Bounded result-subtree replacement while the input remains outside that surface | A need to remount the input or add a framework-wide renderer |
| [Browser CLI template](../tools/yetlet-cli/templates/browser/lib.rs) | Mirrors the Tasks row/binding approach | Independent evidence for a second different application policy; a later shared change must keep template and example aligned |

The render-bench example also calls the DOM renderer; it is a research consumer,
not evidence of a user-facing performance problem. No fresh timings, bundle
savings or removed-line estimates are claimed.

The [YT-105 form comparison](YT-105-FORM-BINDING-RESEARCH.md) already rejected a
new public form-binding abstraction for these consumers. Its dated source counts
are historical; this review confirms the present responsibility split rather
than treating old counts or old Todo statuses as current. The
[YT-107 authoring guide](YT-107-FORM-FEEDBACK.md#application-authoring-guide)
remains the accepted application contract.

## Gaps, significance and disposition

### 1. Declarative semantics and accessibility: a useful expansion

The current View cannot declare many ordinary semantic elements or attributes
that the existing SSR/direct-DOM consumers already use. The most concrete
attribute cases are aria-label, aria-labelledby, aria-describedby, aria-invalid,
aria-busy, role and disabled. The current DOM policy only admits aria-live and
aria-pressed. Title, language/direction and app data markers are other bounded
candidates; nav/footer/aside/headings and ordinary text/list elements are useful
semantic groups, not a request to enable the complete HTML catalogue.

Prefer a finite first addition based on an actual selected View. Native semantics
come first; admitting role or aria-* names is not validation of their values,
allowed role/element combinations or referenced IDs. [ARIA in HTML][aria]
requires those combinations to remain meaningful. Browser accessibility checks
must follow a changed user-visible behavior, not merely an expanded name list.

Do not assume all data-* is harmless to the application: the browser treats these
as application data, but page libraries can interpret it. [HTML data attributes][data]
also recommend library namespacing. Compare the existing InertAttributeName
rules before inventing another name validator, and preserve data-yetlet-* for
framework ownership. The initial recommendation is explicit required names;
opening an entire prefix requires a separately documented admission decision.
Do not automatically couple the slot contract to the renderer's capabilities.

### 2. Two existing cross-renderer inconsistencies to settle before widening

These are constructible source-level cases, not observed failures in the accepted
Todo/Admin journeys. No browser reproduction or regression repair is claimed here.

**Duplicate names.** Element::attribute appends without deduplication. SSR emits
both entries, whereas DOM construction calls set_attribute sequentially. For
Element::new("div").attribute("id", "first").attribute("id", "last"), the current
serializer emits two id attributes. The [HTML parser][parsing] discards the later
duplicate, while [DOM setAttribute][dom] updates the existing attribute. Thus the
same View can identify a different node value across the two rendering paths.

Select one explicit policy before extending the shared subset. Rejecting duplicate
names during renderer preflight is the recommended candidate because it avoids a
silent winner, but first inspect builder/helper callers for intentional overrides
and document any compatibility change. Neither a core builder rewrite nor a new
public error variant is selected by this research. First-wins/last-wins would be
alternatives only if consistently specified and verified for both renderers.

**Void-element children.** A shared input with a text child passes DOM validate,
which currently checks names recursively without a void-child rule. The same View
returns RenderError::VoidElementHasChildren from SSR. Align the supported subset's
preflight before mutation; this is a small contract candidate, not a mandate to
write a complete HTML content-model validator.

HTML parsing and direct node construction also differ for invalid nesting. Later
table support should use explicit row groups and valid structure; textarea needs
its actual text/default-value model. Generic parser repair, arbitrary fragments,
SVG/MathML namespaces and custom elements are outside this proposal. Do not claim
arbitrary View-to-parsed-HTML identity from passing a finite tag allowlist.

### 3. Form controls: initialization is not ongoing binding

[HTML form state][input] distinguishes defaults from current value/checkedness.
[Textarea and option semantics][controls] add their own rules. A later selected
control slice should specify these separately:

| Concern | Initial shared View | Updates and owner |
| --- | --- | --- |
| Text input | A selected value attribute supplies the default; avoid inventing coercion | Read/write the current value property explicitly |
| Checkbox/radio | checked is a presence-based default, not checked="false" | Current checked property; group behavior remains native; do not infer radio coverage from one checkbox |
| Textarea | Text children supply defaultValue, not a value attribute | Current value property; preserve raw draft text |
| Select/option | Option values and selected defaults must be defined with valid children | Current selectedness/value, with single/multiple modes explicitly scoped |
| disabled/required/readonly | [Boolean attributes][boolean] use presence/absence | Use appropriate native properties; ARIA state is not a substitute for native disabling |
| hidden/ARIA | hidden keeps YT-108's enumerated-value contract; ARIA has its own value vocabulary | Do not apply one string-to-boolean conversion to all attributes |

Admin's set_control already checks current value before writing and keeps disabled
state explicit. Tasks already uses checked/set_checked and native submit drafts.
Retain those working seams. Changed text values can move selection under the
HTML value-setter rules; an equal-value guard is not a guarantee for unequal
external writes, Reset, item switching or active composition. The
[UI Events composition model][composition] is a reference, not evidence that a
new binding layer has been tested with real IME.

A form element also changes submission behavior. It must not be added as merely
another container: specify implicit submit, button types, action/method, native
fallback and the application-owned HTTP/CSRF boundary together. No automatic
save, retry, form-validation engine or per-keystroke normalization follows.

### 4. Links and resources: a separate policy decision

Admin's row_create constructs a root-relative link from numeric app identifiers.
That is evidence for a narrow navigation use case, not arbitrary user URL input.
DOM currently rejects href/src; SSR's unsafe_scheme is a conservative blacklist,
not a complete origin/navigation/resource policy.

Before admitting a and href, decide allowed navigation targets, parsing against
a trusted document/application base, same-origin expectations and the treatment
of fragments, protocol-relative URLs and disallowed schemes. Relative URLs are
resolved using a [document base URL][urls], so a leading slash alone is not an
independent trust proof. Use established URL parsing, not ad hoc substring tests.
Images add network and alternative-text requirements; form actions add submission
requirements. They should not inherit one blanket URL permission. Keep srcset,
SVG URLs, embedded browsing contexts and executable/custom elements out until a
separately selected consumer demonstrates the need. Strict CSP remains defense
in depth, not a substitute for a defined renderer boundary.

### 5. Ownership and replacement: preserve, do not redesign

DOM mount already validates and builds before clearing its target. It then clears
and appends in separate operations; it is not a general atomic rollback contract.
Replacing an editing subtree also does not promise focus, selection or third-party
editor-instance preservation. These are documented scope limits, not evidence
that the whole renderer needs a patch engine.

Keep stable editing controls outside replacement surfaces; use keyed owners when
identity matters. Preserve DomEventListener disposal and application lifetime
checks. Capture/passive/once listener options, automatic delegation, generic
hydration, a VDOM, signals/effects, a new forms package and public adapter registries
have no selected requirement in this review. Retain direct web-sys access.

## Proposed implementation sequence, not yet authorized

| Order | Bounded proposal | Acceptance basis and minimum later verification |
| --- | --- | --- |
| A1 | Settle duplicate-name and supported void-child contracts | Existing shared-View semantics plus the concrete cases above. Inspect real builders; native positive/negative tests, SSR outputs and one existing browser check if construction behavior changes. Preserve the target on validation failure. No complete HTML parser/conformance engine. |
| A2 | Add one finite semantic/accessibility subset with a real declarative consumer | Select exact tags/attributes, namespacing and value responsibilities first. Native rejection/security tests, relevant WASM build, the selected visible/ARIA interaction and existing source/template regression seams. Do not migrate Admin wholesale. |
| B | Add a deliberately bounded native-control initialization slice | Define initial defaults versus live values and native submit/reset behavior. Use the chosen text/textarea/select/checkbox consumer only; relevant browser edits/reset/identity checks. Any composition-sensitive write change needs focused input evidence, not routine replay of waived platform suites. |
| C | Optional navigation, then resource attributes only if selected | Explicit URL/base/origin and side-effect policy; positive/negative URL cases plus one real navigation or resource interaction. Keep links, images and form submission distinct. |

A1 is the recommended first implementation selection. A2 is the first visible
capability expansion. B does not create a generalized binding API; C is conditional.
All are proposals inside this research item, not YT-110 or later registrations.
Each separately authorized implementation needs focused evidence and independent
review. Existing CLI build/run/stop and smoke seams remain the execution path;
no new validation framework, benchmark or broad browser/platform matrix is needed.

## Research verification and limits

Source inspection covered the files linked above, the existing renderer ADRs and
the YT-105/YT-107/YT-108 records. Official references below were consulted on
2026-10-21; they define platform behavior, not Yetlet compatibility certification.
No new native/WASM/browser workload is required for this documentation-only change.
YT-108's accepted runtime evidence remains historical evidence for that slice.

The existing cargo xtask check docs passed: 75 established documents and the
109-item backlog DAG. Tracked whitespace checks also passed; the new report's
no-index whitespace check emitted no diagnostics (its content-difference exit 1
was initially misclassified by a temporary wrapper, then correctly interpreted
without changing product files or rerunning accepted workloads).
The docs command uses the fixed REQUIRED list in [xtask/src/docs.rs](../xtask/src/docs.rs):
it checks that established set, linked-file existence there and the backlog DAG,
not every newly added study's full content. Review this study and its local links
directly; do not expand the checker as an unrelated task.

Byte comparisons against the starting checkout confirm that all tracked files
outside the three permitted documentation indices/records are unchanged, including
the accepted YT-108 runtime changes. Its new document and all 4,279 pre-existing
untracked paths are preserved. Independent review was pending at the initial
handoff and is completed by the continuation below. Only research
documentation, its index and the backlog/roadmap registration may change.
No performance improvement, browser conformance or implementation completion is
claimed by this report.

## Independent review and closeout

### Initial dispatch rejection (historical)

The initial independent reviewer dispatch was rejected by the platform safety
check before execution. Follow-up inspection found no reviewer process receipt,
prompt, event log, result or terminal record in this task's evidence directory,
no matching Codex reviewer process, and zero active/running/queued/recovering Jobs
on the focused Runner. No replacement dispatch or current-session role emulation
was attempted in that turn. This was a dispatch rejection, not a reviewer failure
finding or PASS. The initial handoff correctly left YT-109 In progress.

The initial preflight selected the saved reviewer model gpt-6-astra and requested
adaptive high, but neither ran in that attempt. Retain that missing-execution
record separately from the successful continuation below.

### Owner-selected continuation — 2026-10-21

The owner requested continuation of independent review and authorized self-review
only as a fallback after dispatch failure. Fresh inspection confirmed that no
prior reviewer process or uncertain Job remained. A single read-only Codex review
then ran through the existing process surface, without a replacement dispatch.

Independent Reviewer role-directed Job `wc_job_QTvHEtko7LzZWmbB` completed with
process exit 0 and overall_status pass, with no issues, required_followups or
optional_notes. It inspected this report, the YT-109 additions to BACKLOG,
ROADMAP and the documentation index, immediately related renderer/control source,
prior form-binding decisions, local links and retained documentation evidence.
It did not modify the workspace or rerun accepted application tests.

Fresh workspace preflight was healthy/current/eligible/trusted with current
configuration compatibility. The saved reviewer model was gpt-6-astra; the
shared resolver selected adaptive high for ordinary ad-hoc review. Runtime
telemetry observed that model and effort, and re-resolution found no conflict
or degradation. This is an independent role-directed Codex Job result, not
native agent_type identity, formal assurance or self-review. The authorized
self-review fallback was not needed.

All tracked source bytes and both new YT-108/YT-109 documents matched the
review-start snapshot before the parent recorded this closeout. The 4,280
pre-existing untracked paths at continuation start were unchanged. Direct
inspection found all 19 local file links in this report present. No runtime,
dependency, template or release change is part of the continuation.

YT-109 is Done at research-only scope. A1 remains the recommended next selection;
A1, A2, B and C still require separate implementation authorization. YT-108's
accepted results, the exact YT-096 candidate and YT-020 release gates are unchanged.
No commit or push was performed.

The retained evidence directory is `/tmp/yetlet-yt109-20261021-n0te3j8c`.
It contains the original inventory and failure record plus the continuation
profile/resolver decisions, source snapshot, process/events/terminal result,
bounded runtime metadata and `review-continuation-acceptance.json`. The report
and backlog/roadmap closeout remain in the existing checkout.

## Official references

[aria]: https://www.w3.org/TR/html-aria/
[data]: https://html.spec.whatwg.org/multipage/dom.html#embedding-custom-non-visible-data-with-the-data-*-attributes
[parsing]: https://html.spec.whatwg.org/multipage/parsing.html#attribute-name-state
[dom]: https://dom.spec.whatwg.org/#dom-element-setattribute
[input]: https://html.spec.whatwg.org/multipage/input.html#the-input-element
[controls]: https://html.spec.whatwg.org/multipage/form-elements.html#the-textarea-element
[boolean]: https://html.spec.whatwg.org/multipage/common-microsyntaxes.html#boolean-attributes
[composition]: https://www.w3.org/TR/uievents/#events-compositionevents
[urls]: https://html.spec.whatwg.org/multipage/urls-and-fetching.html#document-base-urls
