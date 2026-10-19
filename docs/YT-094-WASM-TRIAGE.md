# YT-094 — Rust/WASM Admin triage slice

Date: 2026-10-19. Base: `7657f13` (reviewed Windows native evidence).
This is an application-owned follow-up to the CLI-created Admin reference. It
uses the existing local graph, keyed DOM/listener ownership and typed HTTP action
contracts from ADRs [0002](adr/0002-events-and-actions.md),
[0003](adr/0003-renderers-and-loading.md) and
[0007](adr/0007-bounded-keyed-dom-and-local-graph.md). No framework API is added.
The earlier Windows native result qualifies its earlier source, not this slice.

## Selected workflow

An operator opens `/board/projects/{project}/triage` from the Admin navigation.
The server supplies at most the 12 most recently updated, project-authorized
items. The page remains readable without JavaScript; each item links to its
native detail/edit flow. An empty project still has the existing New work item
navigation. The full paginated list remains a separate native page.

After WASM mounts, the user can filter loaded titles, reverse their local order,
select items and edit several drafts. These operations send no HTTP request.
Selection and filtering retain each draft; keyed moves preserve retained row
identity. The editor uses persistent native input/textarea/select elements,
updating their value properties only when different. Filtering an item out may
remove its row; its draft remains in application state.

Save selected sends exactly one typed `Save` command to the same-origin JSON
endpoint. It contains CSRF and one item's loaded version, ID and fields. The
server reuses session/project/write authorization, CSRF, field validation,
transition rules and transactional SQLite/history writes. A stale version returns
409 without overwriting the newer record; the local draft remains visible.
Validation and permission/session errors retain drafts too. A storage/network
failure can have an uncertain outcome; the UI tells the user to check the latest
record rather than silently repeating a write.

Only one save may be pending. Its item is temporarily read-only; another selected
item remains editable. Completion updates the original item by ID/version, not
the currently selected draft. The owned runtime invalidates its lifetime and
releases listeners on disposal. A queued action checks that lifetime before
starting HTTP; a late HTTP completion cannot mutate retired DOM.
A request already admitted by the server may still save. Disposal does not cancel
or roll back that server operation. Unsaved drafts are tab-local and disappear
on navigation/reload; no offline persistence or conflict merge is claimed.

## Source and build boundaries

- [Shared browser/state crate](../examples/board-triage/src/lib.rs): app-owned
  DTOs, bounded bootstrap validation, draft reducer and native tests. The
  [browser module](../examples/board-triage/src/browser.rs) owns graph, native
  control bindings, keyed rows, HTTP executor and runtime lifetime.
- [Board route](../examples/lab-operations-board/src/triage.rs): SSR view and
  authenticated semantic save endpoint. Existing constructors retain their
  defaults; `open_with_frontend` selects a validated module path for this page.
- [Admin build step](../examples/admin-dashboard/frontend/build-wasm.mjs):
  locked independent browser Cargo build, matching wasm-bindgen, then Vite.
  [Bootstrap glue](../examples/admin-dashboard/frontend/triage.ts) only loads,
  mounts and disposes Rust code; interaction logic is Rust.
- [Asset adapter](../examples/admin-dashboard/src/vite.rs): the existing startup
  byte snapshot now permits `application/wasm`. Safe paths, regular-file checks,
  graph/file/byte bounds and no request-time filesystem dependency are retained.

Prerequisites: the `wasm32-unknown-unknown` Rust target and `wasm-bindgen-cli`
0.2.129 (matching the locked bindings). `WASM_BINDGEN` can explicitly select the
binary; otherwise the existing user-local Yetlet tools directory or PATH is
used. Generated bindings and build output are ignored. The app's existing
settings-selected `yetlet build` runs this frontend build; `yetlet run` serves its
built assets and `Ctrl-C` drains the application. No new CLI flag or compiler
service is introduced. Browser Cargo compilation contains no Board/SQLite,
settings loader, native session or secret-provider dependency.

The bootstrap is serialized into an escaped hidden input, not an inline script.
It contains authorized records and a CSRF token, never login codes or server-only
settings. Only the enhanced triage response selects `script-src 'self'
'wasm-unsafe-eval'`; it does not permit inline scripts or JavaScript `unsafe-eval`.
Ordinary pages retain the default CSP and contain no script. Missing/blocked
WASM leaves the native links usable. Runtime assets remain no-store startup
snapshots; this slice does not qualify rolling deployment or old asset retention.

## Local verification

Linux, Rust/Cargo 1.99.0, Node 24.21.0, system Chrome 155.0.8059.84. The Browser
plugin/skill was unavailable in this session, so the existing bundled Playwright
controlled system Chrome. No replacement browser infrastructure was added.

| Check | Result |
| --- | --- |
| `cargo test --offline --locked -p yetlet-board-triage -p yetlet-lab-operations-board` | 3 reducer tests and 17 Board tests passed |
| `cargo test --offline --locked --manifest-path examples/admin-dashboard/Cargo.toml` | 13 passed, including actual Vite/WASM asset graph and ordinary SSR regression |
| Focused native and wasm32 Clippy with `-D warnings`; package formatting | Passed |
| Frontend `npm run typecheck` and CLI `yetlet build --manifest-path examples/admin-dashboard/Cargo.toml` | Passed; locked Rust/WASM, bindgen, Vite and native app |
| Real Chrome local interaction | No HTTP during editing/filter/reorder/selection; per-item drafts and retained row/editor identity preserved |
| Real HTTP | Single save, edit another item while pending, original-item completion, real 409 and 422 with draft retention |
| Lifetime | Same-stack Save/dispose starts zero POSTs; listener disposal and in-flight late-response suppression after pagehide passed |
| SSR/CSP/assets | No-JS native editor reachable; normal pages have no module or WASM permission; WASM MIME correct; no console/CSP errors |
| Layout/input | 1440×1000 and 390×844, no horizontal overflow, 44px save control; CJK browser-protocol composition passed |
| CLI run/stop | Isolated disposable database/port, SIGINT drain marker and closed port; CLI terminated with SIGINT on Linux |

Native tests also reject unauthorized, viewer, cross-project, forged-CSRF,
unknown-field and invalid-field saves without writes, and prove one history row
per accepted version. SSR tests bound the list, escape script-shaped text and
reject unsafe module paths before storage is opened. The local result summary is
[retained here](../artifacts/yt094-wasm-20261019/result.json). Detailed run-local
commands, screenshots and logs are under the task's temporary work directory;
startup codes were retained only in process memory, never in the report.

An obsolete asset test initially classified WASM as unsupported; it now asserts
the intentionally selected MIME while retaining the other rejection cases.
Initial build/test compile errors were corrected before the checks above.
A shell wrapper initially used zsh's read-only `status` variable after the CLI
build completed; the corrected wrapper captured a subsequent CLI build exit 0.
These failures do not constitute Windows qualification evidence.

The first independent review found one P2 lifetime defect: a queued save checked
its owner only after awaiting HTTP. The reviewer reproduced a POST starting after
same-stack disposal. The correction adds the same pre-execution lifetime check
used by Todo Browser while retaining the completion check. A focused browser
regression covers zero POSTs for queued disposal alongside the in-flight case.
Independent Reviewer `/root/yt094_wasm_review` follow-up PASS: zero remaining
issues and required followups. It independently repeated the same-stack
reproduction against the rebuilt WASM and observed zero POSTs, then closed its
browser. This does not qualify the deferred platform/input gates.

## Follow-up and remaining scope

[API observations](YT-094-API-OBSERVATIONS.md) register YT-105 and connect the
app-owned packaging work to YT-104. They do not authorize an immediate framework
redesign. This slice leaves YT-094 In progress. Windows execution of the changed
slice, Windows browser, real OS keyboard/IME, actual spoken assistive technology,
external OIDC, production hosting/durability and authenticated frontend HMR remain
unqualified. The owner's existing Linux demonstration was preserved separately.
