# YT-094 — API observations from the Admin triage consumer

Date: 2026-10-19. These are concrete authoring observations from the
[Rust/WASM slice](YT-094-WASM-TRIAGE.md), recorded at the owner's request.
They are follow-up candidates, not blockers for the selected application or an
approval to expand framework APIs. Existing primitives were sufficient.

| Observation | Concrete application cost and current workaround | Follow-up |
| --- | --- | --- |
| A weak `GraphHandle` dispatches but cannot inspect committed state | The save callback needs the selected draft before starting HTTP. Like Todo Browser, Admin maintains a committed `Rc<RefCell<Workspace>>` alongside the graph, updated by the commit callback. `LocalGraph::new` also requires an explicit initial render/dispatch. | YT-105: compare an explicit, lifetime-safe state read pattern with the current bridge using both consumers. Do not add async effects or keep owners alive through event handles. |
| Native control bindings are app-owned | The sample manually maps input/change events, value properties, select/textarea casts, disabled state and conditional writes. Keyed rows and their listeners have useful ownership APIs, but control synchronization requires substantial glue. | YT-105: document or propose the smallest reusable binding seam with focus/selection/composition and disposal tests. This does not select a general forms runtime or VDOM. |
| Typed HTTP is small but does not carry per-field UX | `HttpExecutor` exposes status/code for failures and no caller header hook. The sample carries CSRF in the serialized command body and displays explicit fixed status messages; server field rules remain authoritative. Native SSR already has field-specific feedback. | YT-105: compare existing custom executors versus a narrow documented extension for CSRF/validation. The body-token contract is valid; a new generic response hierarchy is not required. |
| A browser-capable crate does not yet provide a complete authoring/delivery path | Admin owns Cargo/bindgen/Vite wiring, selected script insertion, WASM MIME and page-scoped CSP. Generated JS only bootstraps Rust. The existing CLI can execute this app-owned build through its settings contract. | Existing YT-104: reuse this evidence when defining browser-only creation/build/static delivery. Do not add a second packaging backlog item or silently implement new CLI options here. |

YT-105 should begin with a short comparison of the existing Todo Browser and
Admin code. Select only repeated, measurable boilerplate with a clear safety
contract. Retain explicit ownership, app-selected HTTP boundaries, native SSR
fallback and constructor/trait injection. A result that recommends documentation
alone is valid. Any implementation needs its own selected bounded slice and
independent review; no DI container, provider registry, scheduler or broad
reactivity/validation framework follows from this record.

YT-105 records the later authoring comparison after YT-094. Independently, YT-104
is an unblockable delivery item whose completed dependencies and this packaging
evidence allow a separately selected slice. YT-094's subsequent Windows browser
checks and operator-attested native input/AT results are reconciled in the
[Windows verification](YT-094-WINDOWS-WASM-VERIFICATION.md); they do not expand
these API proposals or establish production qualification.

## Third-party frontend components — YT-106

After trying the triage workspace, the owner requested a separate research item
for existing chart/editor packages. Vite/npm already supplies asset bundling;
it does not provide a general Rust component adapter. The proposed investigation
compares small app-owned JS/TS glue, existing wasm-bindgen interop and Web
Components. A component that requires React may need its own React mount.

The practical contract should give each DOM subtree one owner and specify
mount/update/dispose, typed data and events, stale callbacks, meaningful SSR
fallback, CSP/static assets and package cost. Select one chart and one editor
candidate for a later bounded trial. The research may recommend documentation
or application glue alone; it does not preselect a universal adapter. YT-106
remains Todo. Registration adds no implementation or dependencies and does not
expand YT-094's pending platform/input qualification or its reviewed code.

## YT-107 form/save follow-up (2026-10-21)

The separately selected [YT-107 guide](YT-107-FORM-FEEDBACK.md#application-authoring-guide)
connects these observations to actual Admin/Todo code: application-owned field
feedback, per-draft save results, deliberate recovery, and native/editor/reset
comparison. One rich-error Admin adapter and one code-only Todo mapping do not
yet demonstrate repeated rich-error transport policy. Keep the current private
seams and existing owner/lifetime primitives. The shared DOM View's unsupported
hidden attribute is a concrete, bounded future candidate; it is recorded for
later selection without a new framework or automatic renderer expansion.
The original dated observations and their source scope above remain historical.
