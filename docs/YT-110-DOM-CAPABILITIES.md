# YT-110 — Bounded shared-View DOM capabilities

Date: 2026-10-21. Status: Done at the selected bounded DOM capability scope.
Baseline: 096f8db1e2c5b0f0fd1e664e776503cc21a74727 plus accepted uncommitted
YT-108/YT-109 work on feat/yt108-safe-attribute-consistency.

## Authority and delivered scope

After the [YT-109 comparison](YT-109-DOM-CAPABILITY-REVIEW.md) passed independent
review, the owner selected completing its bounded implementation sequence in one
delivery. YT-110 joins A1, A2, B and the concrete navigation portion of C. It does
not reinterpret research Done as implementation authority: this later owner
request is the implementation authorization. No commit or push was authorized.

The existing [repository instructions](../AGENTS.md), renderer ADRs and
[YT-107 authoring contract](YT-107-FORM-FEEDBACK.md#application-authoring-guide)
remain the requirements for safe rendering, explicit ownership, focused evidence
and no unrelated framework expansion. [YT-108](YT-108-HIDDEN-ATTRIBUTE.md) and
YT-109 historical documents remain unchanged. The separate YT-096 ce56e08
candidate and YT-020 release gates are not requalified.

| Selected slice | Delivered behavior |
| --- | --- |
| A1: common View consistency | Both renderers reject duplicate attribute names; DOM rejects children on its supported void elements before mutation. Selected native controls reject inappropriate child shapes and nested forms within the View. |
| A2: finite semantic/accessibility vocabulary | DOM supports 38 explicit tags and 42 explicit attribute names, with control/navigation attributes scoped to appropriate elements. No arbitrary data-/aria- prefix expansion. |
| B: native control initialization | Shared initial input/textarea/select/checkbox defaults, native reset and GET submission, with live edits owned by the browser/application rather than a new binding runtime. |
| C: bounded navigation | Root-relative anchor/action and anchor fragment admission, browser URL parsing with an HTTP(S) same-origin/no-credentials check, and absolute destination freezing before replacing the target. |

Image/resource URLs, srcset, SVG/MathML, custom elements, arbitrary external
navigation, a VDOM, hydration, automatic two-way binding, effects, general forms,
new event ownership and a validation framework are not selected. Keyed/reactive
crates and direct web-sys access remain separate and unchanged.

## Shared structure and compatibility

[DOM validation](../crates/yetlet-dom/src/lib.rs) rejects repeated attribute names
with DuplicateAttribute; [HTML serialization](../crates/yetlet-html/src/lib.rs)
uses the corresponding RenderError. Element::attribute still appends; there is no
implicit first-wins/last-wins rewrite in core. Callers must choose one value.
This deliberately removes the ambiguous pre-public input for which HTML parsing
and repeated DOM set_attribute previously chose different winners.

DOM input/br/hr cannot have children. Textarea and option accept text children;
DOM select accepts option children and whitespace. SSR retains its broader
optgroup vocabulary. Both renderers reject nested forms within the supplied View.
These are selected structural checks, not full HTML content-model validation or
a guarantee that arbitrary nesting, target ancestry or parser repair is identical.

New DOM errors are DuplicateAttribute, VoidElementHasChildren, InvalidChildren and
InvalidAttributeValue; HTML adds DuplicateAttribute and InvalidChildren.
Exhaustive callers matching these pre-public enums must account for the additions.
The old immutable release candidate is not changed by this source contract.

Inspection and real sample tests identified one duplicate class builder in
[Admin triage](../examples/lab-operations-board/src/triage.rs): the Save helper
already supplied btn-outline-secondary, then appended btn-primary. The appended
class was ignored by the HTML parser. Removing only that second attribute preserves
the effective previous appearance and restores the page under duplicate rejection.
No generic builder mutation API or whole-Admin migration was needed.

## DOM vocabulary

The 38 tags are:

```text
main section div article header footer nav aside
h1 h2 h3 h4 h5 h6 p span ul ol li dl dt dd
strong em small code br hr button input label
form fieldset legend textarea select option a
```

The 30 global names are:

```text
id class hidden title lang dir role tabindex
data-id data-completed data-select data-title data-meta data-selected
aria-live aria-pressed aria-label aria-labelledby aria-describedby
aria-invalid aria-busy aria-hidden aria-current aria-expanded aria-controls
type for placeholder maxlength autocomplete
```

The last five retain the original finite allowlist's admission; this is not a
complete HTML applicability/value validator. ARIA admission does not certify role
combinations, reference targets or assistive-technology behavior. The remaining
12 names have element constraints:

| Names | Allowed DOM elements |
| --- | --- |
| href | a |
| action, method | form |
| name | input, button, textarea, select, form |
| value | input, button, option |
| checked | input |
| selected | option |
| disabled | input, button, textarea, select, fieldset, option |
| required | input, textarea, select |
| readonly | input, textarea |
| rows, cols | textarea |

Unknown names, data-yetlet-* ownership markers, arbitrary data-/aria- names,
inline handlers/styles, srcdoc and resource attributes remain excluded. The
hidden enumerated-value contract from YT-108 is retained without coercion.

## Native defaults, editing and reset

[Shared native_controls](../examples/todo-ui/src/controls.rs) is used inside the
existing browser Todo page and by the native [GET /controls route](../examples/todo-server/src/lib.rs).
It includes labelled text/textarea/single-select/checkbox controls, required,
readonly and disabled states, a fieldset/legend, explicit submit/reset buttons,
a hint association and navigation. The browser page keeps this editing surface
outside the keyed Todo list. No listener or local graph was added for these edits.

Input value and checked, and option selected, initialize browser defaults. Live
value/checked/selectedness remain native properties. Boolean attributes retain
presence semantics, including nonempty strings such as false. The application
must explicitly choose subsequent property writes, events, reset policy and HTTP.
The chosen form uses GET only to preview escaped, bounded query values; it does
not create a Todo or persist data. Readonly fields submit, disabled/unchecked
fields do not. Reset on the response page restores that page's submitted defaults.
This is not a production validation/authentication/CSRF sample; sensitive values
must not be placed in this preview's URL.

Textarea children are joined and CRLF/CR normalized to LF in both renderers.
SSR supplies an extra leading LF only when needed to compensate for the HTML
parser removing the first LF after the start tag. Thus a supplied initial newline
is retained in the parsed defaultValue and live value. The old newline sentinels
in [form-css-proof](../examples/form-css-proof/src/views.rs) and
[Board native forms](../examples/lab-operations-board/src/views.rs) are removed;
the serializer now owns that behavior. Native caller regressions passed.

No new guarantee covers radio groups, multi-select, file inputs, unequal
render-time property writes, composition coordination or focus restoration after
subtree replacement. Existing app-owned IME/editor policies remain unchanged.

## Navigation and submission boundary

Static validate admits href values beginning with one slash or a hash, while action
requires one slash. It rejects empty, absolute, protocol-relative, path-relative,
whitespace/control-containing or backslash-containing spellings. Form method is
limited to get/post. This small input grammar is not an origin check or URL parser.

At mount, the native browser Url parser resolves each href/action against the
owner document's effective base URI. The owner and destination must use HTTP(S),
the resolved destination must share the document origin, and credentials must be
absent. Accepted destinations are written as absolute URLs while the replacement
is detached; later base changes cannot redirect those stored attributes. Invalid
structure or rejected navigation leaves the existing target subtree untouched.
Static validate can therefore pass while mount rejects a document-dependent URL.

Fragments resolve against the effective base, which may be a different same-origin
path. Absolute attribute serialization need not match SSR's relative spelling.
The broader HTML serializer keeps its pre-existing URL policy; SSR applications
still own base-uri/CSP, navigation and submission constraints. This is not a new
universal SSR URL sanitizer, redirect policy, server authorization check or promise
against later application mutation. The selected native route retains strict CSP.
Unexpected failure in the final clear/append stage is still not rollback-safe.

Only the existing web-sys Url feature is enabled. No package version, lockfile,
core dependency or new runtime library was added.

## Verification and retained failures

Validation used the current Ubuntu checkout, Rust 1.99.0, cached wasm-bindgen-cli
0.2.129 and existing headless Chrome. No Windows/IME/AT/release suite was repeated.
The temporary bindgen installation from YT-108 was reused through command-local
PATH; this task performed no installation.

| Evidence | Actual result |
| --- | --- |
| Native library regression across the shared boundary | 235 passing tests across 32 workspace library targets, reconciled from the initial traversal and the affected/remaining-package continuation; no current failures/ignored tests |
| Todo server integration | 2 tests PASS |
| Standalone Admin all-targets | 17 tests PASS; total distinct native tests above is 254 |
| DOM/HTML/Todo native strict Clippy | PASS with -D warnings |
| DOM/Todo/Search wasm32 checks; DOM/Todo wasm32 strict Clippy | PASS |
| Board/form-css-proof strict Clippy; xtask all-targets strict Clippy | PASS |
| Existing release browser builder and Todo server build | PASS; browser digest 75c43ef2f6e916df, reported aggregate WASM bytes 356925; not a performance claim |
| Existing Chrome Todo/Search smoke | PASS once after implementation: original HTTP/revision/keyed/lifetime behavior plus native defaults/edit/reset/GET and actual fragment/root navigation |
| Focused temporary actual-renderer browser fixture | 17 behavior checks PASS for structural rejection/target preservation, unsafe spelling, cross-origin/credential/non-HTTP bases, same-origin resolution and frozen destinations |

The first formatting attempt exposed an unmatched delimiter in a new test; it was
corrected. The existing title-feedback assertion incorrectly required no
aria-describedby anywhere on the page; it is now scoped to the original title
input rather than rejecting the new control's valid hint. The initial shared
regression exposed the duplicate Admin class described above, then its focused
repair and remaining sample tests passed. A standalone Admin --lib invocation
failed because that package has only a binary target; the corrected --all-targets
command passed. Original failed logs remain alongside the passing evidence. Do
not describe this as a single initially clean whole-workspace test invocation.

The primary browser receipt is
`target/m12/yt085/browser-chrome-1dOkaX/result.json`. Adjacent exit receipts verify
closed listeners/streams, reaped owned trees and deleted browser profiles. The
separate temporary product fixture also verifies both servers' closed listeners,
Chrome exit 0, no remaining Chrome group and profile removal. No preview remains.
The negative fixture is repo-external product verification, not a new maintained
harness/framework. It does not change repository manifests or lockfiles.

Desktop/mobile sample screenshots were captured at 1280x900 and 390x844. The
artifact image surface refused the ignored target path, so the images were not
visually inspected in this session and no visual-layout certification is claimed.
No alternate export was used to bypass that refusal. Behavioral checks and their
cleanup receipts passed independently of screenshot display.

Evidence and original logs are retained at
`/tmp/yetlet-yt110-20261021-rr5lb8ba`. Final documentation checks and review are
recorded below; the existing docs checker covers its fixed document set and the
backlog DAG, not every new report's complete content.

## Execution and review status

The requested executor dispatch was rejected by the platform safety check before
execution. Fresh inspection found no executor process, prompt, events, result,
terminal receipt or Job, and no active/uncertain Runner work. The blocked dispatch
was not retried. Under the owner's prior self-work fallback authorization, this
ChatGPT Web session performed the bounded implementation as current-session role
emulation using the canonical executor contract. No saved executor model, selected
effort, native role identity or Job execution is claimed for that work.

A separate read-only Reviewer role-directed Codex Job
`wc_job_v5BMe9GdWQmvY03d` completed with process exit 0 and overall_status pass,
zero issues, required_followups and optional_notes. It inspected the 17 changed/new
YT-110 targets, adjacent contracts and retained native/browser/cleanup evidence.
All tracked source and delivered new-file bytes matched the review-start snapshot
before this closeout record; the reviewer did not edit or repeat accepted tests.

The healthy/current/eligible/trusted workspace selected reviewer gpt-6-astra.
The shared resolver requested adaptive high for ordinary ad-hoc review. Runtime
telemetry observed that model and effort; re-resolution found no conflict or
degradation. This is an independent role-directed Job result, not native agent_type
identity or formal assurance. Reviewer self-review fallback was not needed. The
separate earlier executor rejection and current-session implementation are not
relabelled as a successful agent implementation.

Root/xtask format checks, cargo xtask check docs and tracked whitespace passed.
The docs check reports 75 established documents and a 110-item backlog DAG;
new report/source whitespace, final newline and local file links were checked
separately. The no-index check's content-difference exit 1 is accepted only with
no whitespace diagnostics. No checker framework expansion was made.

The 4,280 pre-existing untracked paths and both accepted YT-108/YT-109 documents
are preserved. New delivery files are this report and examples/todo-ui/src/controls.rs;
all other YT-110 changes are scoped tracked edits. No lockfile changed. The existing
branch and HEAD remain unchanged with an empty index. Task-local browser/server
cleanup passed; no preview remains running.

YT-110 is Done for A1/A2/B and selected same-origin C navigation. The explicit
unsupported capabilities, URL/application boundaries and unverified platform/visual
limits above remain. No commit, push, release or deployment operation was performed.
The next repository action can be an explicitly authorized strict-guarded commit;
this completion does not authorize it or a release qualification change.
