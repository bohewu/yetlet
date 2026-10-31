# YT-219 — HTML/DOM coverage and compatibility contract

Date: 2026-10-31. Status: Done at documentation/source-contract scope; independent Reviewer PASS0.
Source: `d1b8ac596888a6fface30e93a84a076f67b7b2e2`.
Scope: documentation/source inspection only. The owner authorizes this contract
after [the reviewed M25 plan](YT-219-224-DOM-STATIC-SITE-PLAN.md); no runtime,
package/dependency/version, browser execution or Git/publication change follows.

## Decision and inventory

**Expand ordinary body HTML broadly, keep resource/navigation behavior explicit,
and preserve the existing HTML-first and opt-in browser boundary.** Support for
an element describes its admitted structure and attributes, not every browser
method, full HTML conformance, arbitrary markup sanitization or automatic state.

The [complete dated inventory](YT-219-HTML-INVENTORY.md) covers every entry in
the WHATWG index observed2026-10-31:113 named HTML elements, two foreign roots
and one custom-element family;180 attribute/applicability rows covering144
distinct names;89 inline event-handler names. It records actual SSR/DOM behavior
separately from selected future work, document ownership, deferral and exclusion.
The reference index header is dated30 October2026; raw bytes/digest and source
inspection receipts are retained under `target/yt219-contract-20261031/`.

| Disposition | Count | Meaning |
|---|---|---|
| Current DOM |38 | Existing tags, with the current42-name attribute policy |
| YT-220 selected additions |51 | Ordinary content/tables/controls and a canvas host; not implemented |
| YT-221 selected addition |1 | `img`, behind explicit image options; not implemented |
| Document writer |7 | Root/head/metadata, outside subtree mounting |
| Rejected executable/embedded tags |5 | Existing ordinary-View boundary remains |
| Deferred entries |14 |11 named HTML elements, two foreign roots and one custom-element family |

The resulting selected target is **90 named HTML elements**, not a claim that
the current renderer supports90. The11 deferred HTML elements are `area`,
`audio`, `map`, `video`, `track`, `picture`, `source`, `noscript`, `template`,
`slot` and `selectedcontent`. Responsive image selection is explicitly deferred
until its candidate grammar is selected; the first YT-221 image capability is
`img` only. This resolves the M25 plan's conditional responsive-image branch.

## Current behavior to preserve and limits to state

- [View/Element](../crates/yetlet/src/view/mod.rs) owns names, attributes and
  children. Construction does not validate/sanitize; it has no namespace/raw-
  HTML variant. No new required field, event model or Core node kind is selected.
- [SSR](../crates/yetlet/src/html/mod.rs) accepts lowercase identifier-shaped
  names broadly, rejects five executable/embedded tags, `on*`, inline style and
  srcdoc, escapes text/attributes and checks duplicate/void/selected controls.
  Its URL denylist covers `href`, `src`, `action`, `formaction` and an unreachable
  colon-qualified legacy name. It is not a general URL/origin policy, and does
  not check `srcset`, `poster`, `ping` or arbitrary HTML applicability.
- [DOM](../crates/yetlet-browser/src/dom/mod.rs) currently admits38 tags and42
  attribute names. Five names (`type`, `for`, `placeholder`, `maxlength`,
  `autocomplete`) are admitted globally for historical compatibility; the
  remaining scope decisions are listed in the inventory. Do not silently remove
  those accepted uses while adding more precisely scoped new attributes.
- Native `validate` checks structure and narrow URL spelling. It cannot prove
  the browser's effective base, origin, CSP, layout or actual mounting. On Wasm,
  `mount` resolves admitted links/forms with the browser URL parser, requires
  HTTP(S), same document origin and no credentials, freezes absolute destinations,
  constructs the replacement and then clears/appends to the owned target.
- Validation/construction failure before replacement preserves the old target.
  An unexpected final DOM operation failure is not transactional rollback.
  Subtree replacement does not preserve arbitrary focus/selection/listeners;
  [explicit keyed ownership](adr/0007-bounded-keyed-dom-and-local-graph.md) remains
  the separate identity-sensitive path. No new hydration/VDOM/scheduler follows.

These are source observations, not rerun product evidence. Historical YT-110,
YT-217/218 and published SDK receipts retain their original sources and limits.

## YT-220 — Selected ordinary vocabulary and structures

Add the51 names marked220 in the inventory, retaining the38 current names.
Use direct HTML-namespace DOM creation, not `innerHTML`; do not enable custom
elements, scripts, resource-bearing attributes or namespace creation by name.
Ordinary CSS remains external, and listeners/state remain application-owned.

### Attribute decisions

The inventory assigns every standard attribute row a disposition.220 means
selected uses on admitted ordinary elements only, not every applicability in
that row. Use these finite additions/scopes alongside preserved current uses:

| Group | Selected names/scope and value rule |
|---|---|
| Global inert/native text hints | `accesskey`, `autocapitalize`, `autocorrect`, `contenteditable`, `draggable`, `enterkeyhint`, `inert`, `inputmode`, `spellcheck`, `translate`, `writingsuggestions`; use standard literal/enumerated semantics, never evaluate values |
| Text semantics | `datetime` on time/del/ins; `value` on data/li; preserve language/direction and existing role/ARIA names; bdo requires explicit valid direction |
| Lists | `start`, `reversed`, `type` on ol; `value` on li; integer values where applicable |
| Tables | `span` on col/colgroup; `colspan`, `rowspan`, `headers` on td/th; `scope`, `abbr` on th; positive spans except standard rowspan zero, scoped tokens and ID references |
| Native form controls | `accept` on input; `accept-charset`, `enctype`, `novalidate` on form; `dirname`, `minlength` on appropriate text controls; `min`, `max`, `step`, `pattern`, `size`, `multiple`, `list` on applicable controls; `form` on admitted form-associated elements, `label` on optgroup/option, `for` on output; preserve existing checked/selected/value/disabled/etc. rules |
| Disclosure/dialog | `open` on details/dialog; `name` on details; native initial state only, no automatic modal/top-layer behavior |
| Meter/progress | Scoped `value`, `min`, `max`, `low`, `high`, `optimum`; finite numeric spelling and standard bounds/ordering for the selected element |
| Canvas host | Nonnegative integer `width`/`height`, fallback content; no context acquisition, GPU initialization or frame loop |
| Textarea | Scoped `wrap`, existing rows/cols/default text semantics; live values and selection remain app-owned |

Apply numeric/enumerated checks only to newly selected uses; do not retrospectively
turn preserved global names into a new schema for all old consumers. Text hints
such as spellcheck/contenteditable retain native behavior; admission does not
implement an editor, IME handling or persistence. Form ID associations, patterns,
labels and submitted values remain application-owned, not authorization.
Native ID references and named details groups can reach outside one mount root;
the application must coordinate those document-wide names. The renderer owns
its direct subtree mutations, not every native effect of author-supplied HTML.

Keep current finite `aria-*`/`data-*` names. No new wildcard admission, reserved
`data-yetlet-*`, `on*`, `style`, `srcdoc` or customized-built-in `is` is selected.
The broader existing `InertAttributeName` slot helper is not a DOM allowlist.
Additional ARIA names, microdata, autofocus, popover/command/closedby behavior and
new heading-offset/input-color extensions stay deferred with inventory reasons.

### Selected structure rules and parser parity

Use canonical valid shapes to prevent known HTML-parser repairs in the selected
new families. This is bounded validation, not an implementation of the full HTML
content-model algorithm:

- A table has at most one first caption, colgroups before row sections, at most
  one thead before tbody groups, and at most one final tfoot. Rows must be inside
  explicit sections; no direct `table > tr` shorthand. A row contains th/td;
  a colgroup contains col or uses span, not both. Permit formatting whitespace,
  not stray non-whitespace table/section/row text. Enforce direct-child rules,
  while cells/caption retain their ordinary valid content responsibilities.
- Details requires one first summary, followed by its body. A summary outside
  details is rejected in this selected subset. Figcaption, when present, is a
  single first or last child of figure. Direct child and duplicate checks belong
  to new DOM validation before mutation, not a global SSR rewrite.
- Select remains the classic option/optgroup form: option is text-only;
  optgroup contains options plus formatting whitespace. Datalist in this slice
  likewise contains options/whitespace; its broader fallback-content model is
  not selected. Nested forms and existing textarea/option checks remain intact.
- Progress/meter and canvas accept the selected fallback content; they do not
  create progress algorithms, drawing APIs or imperative native-control bindings.
  Dialog's `open` is a boolean attribute; showModal/close/focus return belong to
  the owner. Native details events do not acquire a framework scheduler.
- HTML parsing ignores one leading LF after pre/textarea and normalizes CR/CRLF.
  YT-220 must preserve the selected pre text stream across adjacent text nodes,
  including CR/LF split boundaries, by bounded HTML serialization/direct-DOM
  handling. Preserve the existing textarea solution. Pre may contain code/span;
  compensate only when the actual first parsed text token needs it, not blindly
  before a child element. No `innerHTML` or raw-text escape hatch is allowed.

The shared parity claim is scoped to validated canonical shapes, matching host
context, ordinary Unicode text with LF-normalized line endings, and equivalent
combined text/attributes/element structure, not identical text-node segmentation
or exact serializer bytes. The selected pre/textarea normalization is the explicit
exception for CR/CRLF input. U+0000 and arbitrary malformed nesting are outside
the parity claim. The owner supplies a compatible mounting host/ancestor context;
this validator cannot certify an enclosing document it does not own.
See [HTML parsing rules](https://html.spec.whatwg.org/multipage/parsing.html).

SSR's broad existing `render_to_string` admission is retained. Callers can apply
the shared DOM subset's native structural validation before SSR when they want
this parity profile. Do not globally reject pre-existing SSR-only tables or
controls just because DOM's selected shape is narrower. The required pre text
serialization correction is a documented semantic consistency change; snapshot
byte changes need focused migration notes, while `escape_text` and public
View/Element APIs stay intact. No old release is relabeled or requalified.

## YT-221 — Explicit options and resource boundaries

Canonical future names are **`DomOptions`**, **`NavigationPolicy`** and
**`ImagePolicy`**, under `yetlet_browser::dom`. Reserve the additive entry-point
names **`validate_with_options`** and **`mount_with_options`**. They are selected
contract names, not existing exports or executable examples. Exact constructor/
configuration-error signatures must be reviewed with YT-221 before implementation;
they may not weaken the following frozen semantics.

| Setting | Default/legacy entry points | Explicit opt-in |
|---|---|---|
| NavigationPolicy | SameOrigin: current root-relative link/form and link-fragment rules | Additional named HTTPS origins for anchors only; no wildcard/any-scheme option |
| ImagePolicy | Disabled: img remains rejected without image selection | SameOrigin, optionally additional named HTTPS image origins; no automatic fetch during validation |
| Forms | Existing same-origin root-relative action and get/post spelling | No cross-origin action, formaction, new form target or method override in this item |

`validate(view)` and `mount(target, view)` delegate semantically to the defaults.
YT-220's ordinary additions become available without URL relaxation. Native
validation remains structural/spelling-only; successful native options validation
is not browser origin acceptance. Use a standard URL parser at the point of
origin decisions, not string-prefix comparisons or a homegrown parser. On Wasm,
fully resolve/canonicalize both configured origins and destinations with the
browser URL implementation; require true origins (no path/query/fragment or
credentials in configuration). Invalid policy input fails before mounting.

For anchors, keep existing input grammar by default; the explicit external path
adds absolute HTTPS URLs at named origins only. Reject protocol-relative,
credential-bearing, script/data/blob and unselected schemes. Preserve browser-
resolved absolute URLs to resist later base changes. External `target` is limited
to `_self`/`_blank`; `_blank` requires noopener protection. Accept only selected
non-executable rel tokens, never opener; ping/download remain deferred. Selected
referrerpolicy values are no-referrer/same-origin; existing anchors without the
new attribute keep their current behavior. Do not convert link permission into
form, resource, action-transport or authentication authority.

For images, first support `img` with explicit `src`, required `alt` (empty is
valid decoration), dimensions, loading/decoding/fetch-priority enums where
selected. Same-origin resource paths must be root-relative; explicit external
sources use absolute HTTPS at named image origins. No srcset/sizes/picture/source,
crossorigin credentials mode, image-map activation or CSS URL path is selected.
The HTML-first document writer can still include separately trusted public assets;
that does not make those attributes accepted by DOM. SVG-as-an-img resource does
not imply SVG namespace/node support in View.

Preflight every node, structure, option and resolved URL **before assigning any
resource-bearing attribute**, constructing a resource-triggering subtree or
clearing the old target. Validation has no requests. An invalid later child
must not cause an earlier resource's request; accepted resources can start loading
before insertion once complete preflight succeeds. After admission, browser
network errors, cookies/referrers under the selected policy and partial final DOM
operations are not transactional rollback. Use detached construction and existing
ownership; do not add a resource loader/retry runtime. CSP may further restrict
loading and must not be loosened to satisfy a renderer test. SSR's existing broad
URL scheme checks do not establish this mounted-resource policy.

Options are public application configuration, not server secrets. Settings
provider is optional; a host may map validated settings into these typed options.
No mandatory provider/DI, global singleton, automatic environment read or
server-only dependency enters browser builds. The unselected options path has
no resource loading or runtime worker. Reuse existing target-specific web-sys
URL support and retain native/browser feature isolation.

## Consumer and compatibility impact

| Consumer | Preserve | Later changed-behavior evidence |
|---|---|---|
| [Todo controls](../examples/todo-ui/src/controls.rs) | Existing valid forms, default state, reset/submission and root/fragment links | Focused shared table/pre/control additions; no replay of unchanged Todo lifecycle |
| [Browser Tasks](../examples/browser-tasks/src/lib.rs) and [CLI browser template](../tools/yetlet-cli/templates/browser/README.md) | Existing finite task UI, initial shell, explicit keyed/listener lifetime and static dist | Confirm old entry points/defaults compile when touched; no new settings/loader or all-sample campaign |
| [Search Island](../examples/search-island/src/lib.rs) | Readable SSR and owned replacement; no hydration or cross-origin authority | Only changed shared validator/renderer integration, not old Auth/Session proof |
| [Public guide sources](../site/pages.toml) | Same eight sources/excerpts and current mdBook deployment | Future YT-224 exercises code blocks, tables, scoped external links and an image; no site replacement in YT-219 |
| [HTML renderer](../crates/yetlet/src/html/mod.rs) and binding/keyed owners | Existing callable APIs, duplicate/escaping checks and lifetime contracts | New pre parsing semantics and selected canonical structure only; no general VDOM/form API |

Keep current `DomPolicyError` and `RenderError` entry-point behavior for old
inputs. Prefer their existing structural/value errors for new rejected markup;
do not add public enum variants solely for richer messages. If an implementation
truly requires a source-breaking enum/signature change, stop at an explicit
migration decision before making it; the pre-release status is not blanket
permission. Document intentional new admission and any pre output-byte correction.

## Verification and completion boundary

This item uses source inspection, the new standards inventory, documentation
links/structure/whitespace/DAG checks and an independent review. No existing
accepted unit/browser/platform/load/release check is rerun. One initial inventory
extractor assumed explicit optional HTML cell end tags and returned empty tables;
that result was rejected, then a single bounded extraction correction handled
the source's omitted end tags. Both observations remain outside tracked source;
no validator framework, product code or dependency was introduced.

Later YT-220/221 acceptance is limited to changed behavior: focused new names/
scopes/rejection cases, parser-versus-direct-DOM pre/table/disclosure/control
cases, browser URL/base/credential and denied-request ordering, owned target
preservation, relevant native/Wasm/feature checks and independent review. Reuse
old evidence and retain new failures. The few parser-sensitive/native behaviors
need selected Chrome/Firefox checks; this is not full browser/platform
qualification or a replay of every existing sample. YT-222 may proceed from
this reviewed boundary separately from DOM implementation.

YT-219 is not Done until independent Reviewer PASS accepts the inventory,
decisions and source claims. YT-220–224 remain Todo. No implementation, Commit/
Push, Pages update, tag, hosted signing or publication is authorized for the
new work. Next unblockable implementation item after review: **YT-220**.

Independent review: **PASS0**, zero issues/required followups; all seven reviewed
source hashes and ten inspected product-input hashes matched at acceptance.
Registered reviewer model `gpt-6-astra` and effective high effort were verified.
YT-219 is Done at this documentation-only scope. Raw standards/extraction,
reconciliation, review and documentation-check receipts are retained under
`target/yt219-contract-20261031/` and copied outside the repository to its
same-named release-evidence set. No product behavior was executed or changed.
