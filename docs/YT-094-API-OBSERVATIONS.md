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
evidence allow a separately selected slice. YT-094's Windows browser and real
input/AT gates remain scheduled for final qualification, as requested by the owner.
