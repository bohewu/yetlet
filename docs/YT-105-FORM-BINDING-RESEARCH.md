# YT-105 — native form values and event binding comparison

Date: 2026-10-19. Inspected source: `e3c217a` after the YT-104 verification-note
commit. Scope: the first, documentation-only YT-105 comparison requested by the
maintainer. No application, crate API, dependency, template or runtime changes
are selected. Independent Reviewer `/root/yt105_form_binding_research_review_oct19`
PASS has zero issues or required followups.

## Recommendation

Document two existing patterns: read a native draft at an explicit action, or
synchronize a graph-owned draft on explicitly chosen events. Keep application
decisions about messages, reset, validation, focus and HTTP visible. Reuse
`web-sys` and the existing owned `DomEventListener`.

The current comparison does **not** justify a new public form-binding helper.
Admin already consolidates heterogeneous controls in private helpers; Todo's
single read needs no equivalent abstraction. The common ownership primitive is
already shared. A short choice/reference guide is the smallest useful change;
the rules below can serve as that reference without changing executable code.

## Consumers and measured source cost

"Admin" means the WASM workspace in
[`examples/board-triage`](../examples/board-triage/src/browser.rs), hosted by the
Admin dashboard. "Todo" means the existing
[`examples/todo-browser`](../examples/todo-browser/src/lib.rs) route. The newer
[YT-104 task starter](../examples/browser-tasks/src/lib.rs) is a supplementary
check, not a replacement for the two original consumers.

| Concern | Admin triage | Todo Browser |
| --- | --- | --- |
| Native controls in this comparison | Title input, Description textarea, three single selects, filter input; the bootstrap and CSRF hidden values are read separately | One text input for remote creation |
| Value ownership | Five draft fields live in `Workspace`; three text/filter controls use `input`, three selects use `change` | The DOM owns unfinished text; a button `click` reads `.value()` once into `TodoCommand::Create` |
| Reading/type conversion | Private `value` handles input, textarea and select; 11 physical lines, browser.rs 102–112 | One typed cast at lib.rs 294 and one `.value()` read at 303 |
| Writing | Private `set_control`, 22 physical lines, browser.rs 113–134; five-field render loop is 13 lines, 188–200 | No `.set_value` call; local graph updates its separate list and remote completion its separate result host |
| Event mapping | 32-line loop at 298–329 binds six controls and maps their strings into `Msg::Filter` or `Msg::Edit`; event selection and field routing are included in that count | One remote click callback at 301–337 includes the HTTP action, lifetime checks and feedback; these are not all form-binding overhead |
| Listener ownership | Ten listeners at mount: delegated row selection, six controls, Reverse, Reset and Save; retained in `Runtime` | Two listeners: local Toggle and remote Create; retained in `BrowserRuntime` |
| Draft/reset/save policy | Per-item drafts; Reset restores the loaded item; pending-save state disables the selected draft's controls; Save explicitly includes CSRF and item version | Native text remains until the app changes it; click constructs the typed HTTP action; no graph-owned text editor or reset policy |

Counts are static source inspection, not performance or bundle measurements.
Physical line counts use inclusive source ranges and include comments; these
four Admin ranges contain no blank lines. Loop-expanded listener counts differ
from constructor call-site counts: Admin has four `DomEventListener::new` sites
and Todo two. Six controls already share one Admin binding loop. The 32 lines
cannot honestly be presented as 32 lines removable by a generic helper because
they also encode application semantics.

The YT-104 starter reinforces the distinction: Add and Rename read native text
on two `submit` callbacks, reset/prefill are two explicit `.set_value` calls,
and completion reads `.checked()` on delegated `change`. Its six constructor
sites produce eight listeners because three filter buttons share a loop.
Retained editor text is deliberately outside the render loop (lib.rs 182–308).
Replacing this with per-keystroke graph synchronization would add behavior.

## Existing seams and their limits

- [`DomEventListener`](../crates/yetlet-dom-keyed/src/lib.rs), lines 65–120,
  already owns target/callback registration and removes the callback on explicit
  disposal or drop. Callers must retain the returned owner. Its callback receives
  a native `Event`; it does not choose a property, event type or application
  message.
- Admin's private helpers already handle its three control types. The read
  helper is also used for hidden bootstrap/CSRF values; those reads should not
  acquire editing/listener behavior through a broad replacement.
- [`yetlet-dom`](../crates/yetlet-dom/src/lib.rs), lines 27–76, renders a bounded
  View subset, without select/textarea/form support or general live-property
  synchronization. Admin's editor is SSR-created native HTML; its controls do
  not need a renderer expansion to be read or updated through `web-sys`.
- The experimental
  [`KeyedListTemplate::text_input`](../examples/keyed-bindings/src/lib.rs)
  already exists at lines 294–324; `attach_input` at 1094–1117 and `update_row`
  at 1152–1170 bind a keyed text input and compare previous model values before
  rewriting it. This is example-owned, row-specific research, not a public
  crate API for standalone textarea/select editors. Its broader template/focus
  machinery is not a minimal extraction for these consumers. Preserve
  [ADR 0007](adr/0007-bounded-keyed-dom-and-local-graph.md).

## Minimal authoring contract

| Decision | Rule and concrete consequence |
| --- | --- |
| Choose draft ownership first | Use native draft ownership for action-time reads, as Todo does. Choose graph-owned draft synchronization when the UI needs live multi-draft switching, filtering or unsaved-state feedback, as Admin does. These are separate valid patterns. |
| Choose events explicitly | Preserve the consumer's `input`, `change`, `click` or form `submit` boundary. A property helper must not turn typing into HTTP or replace Todo's click flow with a new form workflow. |
| Read the live native property | Read `.value()` for the selected text/select control and `.checked()` for a checkbox. HTML attributes describe a different interface; no attribute-based surrogate for the live property is proposed. |
| Keep raw draft text | App code maps the raw value to a message. Domain parsing, trimming and server validation stay at their selected application boundaries; a common binding must not normalize partially entered or composing text. |
| Write deliberately | Admin's `if current != next` avoids unnecessary value writes. Todo's action-time draft needs no render-time write. An explicit reset, prefill or item switch remains an application decision. |
| Keep state updates explicit | Programmatic `.set_value` is not an implicit `input` dispatch. A graph-owned Reset updates state and then renders; a native-draft clear updates that draft. Neither implies save, validation, HTTP or an extra synthetic command. |
| Retain listener ownership | Use the existing owner vector/row owner and weak graph handles. Drop/dispose removes bindings; existing runtime lifetime checks still guard queued/in-flight HTTP completions. A form convenience must not introduce another owner or scheduler. |
| Keep error policy local | Initialization failures remain `Result` errors; applications retain their dispatch-error/feedback policy. A convenience must not silently introduce retries or hide action failures. |

These are proposed authoring rules over existing APIs, not newly available
methods or an implemented two-way binding layer.

### Focus and composition: precise guarantees

The HTML standard specifies that changing a text input's value can move the
cursor to the end and clear selection; textarea has a corresponding rule.
Comparing the existing value before writing is therefore useful. It does not
prove safety when an incoming value is different, such as a reset or item
switch. See the [input value setter](https://html.spec.whatwg.org/multipage/input.html#dom-input-value)
and [textarea value setter](https://html.spec.whatwg.org/multipage/form-elements.html#dom-textarea-value).

Admin has no composition listener or `InputEvent::is_composing` check. Its raw
input dispatch plus equal-value guard supports its current consumer; it is not
a general composition coordinator. The
[UI Events working draft](https://www.w3.org/TR/uievents/#events-compositionevents)
defines composition events and `isComposing`; this is reference semantics, not
a new claim about platform behavior. A future helper must not silently discard
composition input, force normalization, queue an external replacement for later,
or promise that unequal writes preserve an active composition. Any such policy
requires a separately selected need and focused input validation.

The [maintained input-event reference](https://developer.mozilla.org/en-US/docs/Web/API/Element/input_event)
distinguishes text `input` from committed `change` and explains that programmatic
value changes do not themselves fire `input`. Event selection and explicit
command dispatch therefore remain visible in the application contract.

## Options and decision

| Option | Benefit | Cost / decision |
| --- | --- | --- |
| Document the two existing patterns and link the private helpers | Explains when each fits; no runtime/dependency cost or behavioral migration | **Recommended first result.** Does not reduce source lines by itself; no savings are claimed. |
| Further app-private convenience | Could move repeated casting/listener scaffolding if a concrete second use needs it | Only after comparing the actual callers; keep raw-value/message/error/ownership decisions explicit. Admin already has the main read/write consolidation. |
| Public typed value-listener helper | Could eventually reuse value extraction plus existing listener ownership | Deferred: Todo's current read is simpler, the native-draft starter does not share Admin's continuous synchronization, and type/event coverage has not been validated as one API. |
| Extract the experimental template or build a forms runtime | Broader model-to-control authoring | Does not match this bounded need; carries template, draft, composition and lifecycle policy beyond the measured duplication. |

If later callers demonstrate the same repeated seam, evaluate only a small
value-read/listener adapter first: explicit control and event in, raw value to
an application callback, existing owned listener out, ordinary initialization
errors. This is a candidate shape, not a selected signature or authorization
to implement. Keep render writes, domain validation, reset, graph state and HTTP
outside that candidate. Checkbox/multi-select/radio/file/contenteditable and
composition-policy generalization must not be inferred from string controls.

## Minimum validation for a later selected implementation

No new runtime verification is needed for this documentation-only comparison.
If an implementation is separately chosen, reuse the affected consumer and
existing browser checks rather than building another harness:

| Scenario | Necessary observation |
| --- | --- |
| Native draft / action-time read | Todo reads the current raw value at its existing action; unrelated list changes retain unfinished text; exactly one explicitly selected action occurs. The YT-104 submit variant is a useful additional example only if affected. |
| Admin text and textarea edits | Both map raw values to the intended draft; unrelated commits retain node identity, focus and selection when the value is unchanged. CJK input is preserved without per-keystroke normalization. |
| Select and explicit reset/switch | The selected value reaches the right field; Reset/item switch follows the app's existing state policy; programmatic writes do not create synthetic input commands or HTTP saves. |
| Initialization and error paths | Wrong/missing control fails initialization; existing app feedback and typed validation/permission/conflict behavior are preserved. A read/listener convenience alone does not guarantee existing partial-mount rollback. |
| Lifetime | Disposal/remount leaves one current listener path; retired events/actions cannot mutate a replacement runtime. Reuse existing `DomEventListener` and runtime-lifetime coverage. |
| Composition-sensitive change, if any | Choose a focused Ubuntu input check for the changed behavior. Preserve accepted Windows/operator evidence; do not make routine Windows IME/assistive-technology replay a prerequisite. |

This plan measures only a selected future change. It neither requests a new
browser/platform matrix nor reopens the accepted YT-094/YT-104 journeys.

## Verification and remaining scope

Research verification: source/range/count inspection, official browser-reference
checks, `cargo xtask check docs` and whitespace checks. No server, browser,
toolchain installation, application build, benchmark or runtime test was started
for this study. Existing runtime receipts are background evidence for their
recorded source, not fresh results for a proposed helper.

YT-105 remains **Todo** at its broader backlog scope. This first recommendation
covers form properties/events and necessary ownership/composition boundaries.
Weak-handle committed-state reads/initial rendering and typed HTTP CSRF/per-field
validation ergonomics still need their own bounded comparison if the maintainer
selects them. YT-106 third-party chart/editor interoperability remains separate.
This research document is not included in the earlier YT-104 verification-note
commit and does not authorize its own commit or an implementation.
