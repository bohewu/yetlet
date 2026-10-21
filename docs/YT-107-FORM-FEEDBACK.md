# YT-107 — Admin and Todo form/save feedback

Date: 2026-10-21. Status: Done at the selected application scope. Owner-approved implementation follows
the [YT-105 HTTP-feedback research](YT-105-HTTP-FEEDBACK-RESEARCH.md), keeping
the existing application boundaries, typed settings, CLI and server authority.
YT-105's completed research and earlier implementation remain Done.

## Selected scope and slices

| Slice | Delivery | State |
| --- | --- | --- |
| 1. Server validation to field messages | Carry Admin's five existing server field rules through its application-owned JSON save boundary into the correct draft/control; map Todo's existing invalid_title code to its title control. Preserve entered values, safe text/ARIA associations, current authorization/CSRF/status behavior, native routes and lifetime checks | Verified; independent Reviewer PASS |
| 2. Draft/save state and recovery | Improve the complete editing/pending/success/failure workflow and deliberate recovery for session/permission changes, conflicts and uncertain network outcomes; preserve other drafts and prevent duplicate submissions | Verified; independent Reviewer PASS |
| 3. Integration and developer experience | Complete editor/native/Todo consistency, document actual application patterns and repeated costs, assess demonstrated API gaps, and qualify the complete selected journey | Verified; independent Reviewer PASS |

The owner selected all three slices sequentially, committing the reviewed first
and second slices before work on their successors. The
[application authoring guide](#application-authoring-guide) consolidates the
resulting practical patterns.
No general forms runtime, DI container, generic error hierarchy, new public
transport API, automatic retry/credential refresh, signing or publication is
selected. Whole-item closure requires all selected slices and independent review.

## Slice 1 contract

- Admin reuses Fields::errors after existing authentication, project/write and
  CSRF checks. Return HTTP 422 with the existing validation code and a bounded
  application-owned field map for title, description, status, priority and
  assignee. Messages come from static server rules, never rejected values,
  credentials, database diagnostics or raw HTML.
- Only the expected 422 validation payload becomes field messages. Malformed
  requests, authentication/permission failures, not-found, concurrency/state
  transitions, storage/network/invalid responses remain global failures.
  Unknown/malformed field details fall back to the fixed global summary.
- Capture the submitted item identity before await. Validation messages belong
  to that pending draft even if selection changes. Retain every entered value;
  clear a field's obsolete message on edit and that item's messages on Reset
  or successful save. Keep existing pending/version/lifetime guards.
- Render plain text with visible field messages and aria-invalid/describedby
  associations. CodeMirror and its native textarea fallback must expose the
  same Description error without replacing the editor or moving focus.
- Todo keeps its existing anonymous server contract and HttpExecutor. Map only
  the existing invalid_title response to its native title input; retain text,
  clear stale feedback on edit/success, and keep unknown failures global.
- Native no-JS forms, CSP/escaping, session authority, explicit save boundaries,
  and the existing CLI build/run/stop flow remain usable. No new default secret,
  configuration option or dependency is required.

## Focused verification

Extend the existing Board HTTP and triage reducer tests for bounded field
details, authority/token rejection without detail leakage or writes, all five
field rules, global transition/conflict responses, submitted-draft identity and
error clearing. Exercise the real Admin and Todo browser flows, including
safe text/ARIA, correction, selection while awaiting a response, CodeMirror
compatibility and native fallback. Use the existing CLI for build/run/stop,
strict Clippy, WASM compilation, TypeScript and documentation checks.

Browser plugin not available; use the existing Playwright/system Chrome path.
Keep temporary scripts, screenshots and synthetic databases outside the
repository. Reuse historical platform evidence; do not repeat Windows OIDC,
routine OS IME/assistive-technology work, release or cloud qualification.
Independent reviewer review follows implementation and actual verification.

## Slice 1 verification record

Verified on Ubuntu 26.04.1 x64, Rust 1.99.0, Node 24.21.0 and system Chrome
155.0.8059.84 against the current checkout based on
e02332c60b3eb117d3840a922e2f9111a78f5003. This is application-slice evidence,
not an expansion of release/platform qualification.

| Check | Actual result |
| --- | --- |
| cargo test --offline --locked -p yetlet-board-triage -p yetlet-lab-operations-board -p yetlet-todo-ui | PASS: 7 triage, 20 Board and 10 shared Todo tests; the added transition/global-error assertions also pass the focused triage_ selection |
| cargo test --offline --locked -p yetlet-todo-server --all-targets | PASS: 8 unit and 2 integration tests |
| cargo test --offline --locked --manifest-path examples/admin-dashboard/Cargo.toml --all-targets | PASS: 17 tests |
| cargo test --offline --locked -p yetlet-todo-browser -p yetlet-todo-ui | PASS: 1 browser-page DOM-policy regression and 10 shared Todo tests; total distinct native tests across this slice: 65 |
| cargo check --offline --locked --target wasm32-unknown-unknown -p yetlet-board-triage -p yetlet-todo-browser | PASS |
| Strict Clippy for changed native packages, standalone Admin and both WASM entrypoints | PASS with -D warnings; Todo native/WASM checks repeated after the DOM-policy fix |
| npm run typecheck --prefix examples/admin-dashboard/frontend | PASS |
| yetlet build --manifest-path examples/admin-dashboard/Cargo.toml | PASS: native app, WASM/bindgen and Vite assets |
| cargo xtask build browser --release | PASS after the Todo fix; asset digest 6debc0f0fb8ae762 |
| Todo standalone-consumer yetlet build/run and Admin yetlet run | PASS: real HTTP readiness, application interactions and SIGINT cleanup; both owned listeners closed; Admin drain marker observed |
| cargo xtask check docs; git diff --check | PASS before independent review |

The real Admin HTTP flow returns all five existing static field rules without
echoing rejected values. Unauthorized, read-only, cross-project and forged-CSRF
requests expose no field details and do not mutate records/history. Invalid
state transitions retain the global 422 response. Reducer tests bind delayed
validation to the submitted draft and check field/edit/reset/save clearing.

Chrome exercised actual title/Description 422 errors, ARIA associations on the
native input and CodeMirror, retained values, correction and successful save.
A delayed actual response stayed with its original draft while another draft
was selected/edited. Reset kept the other draft. Actual CSRF denial remained
global; simulated unknown details fell back to the fixed summary and simulated
markup-bearing messages rendered as text. Package-import failure retained a
working textarea; JS-off links still reached the native editor. Desktop
1440x1000 and mobile 390x844 screenshots were visually inspected. Todo's real
invalid_title response displayed inline/ARIA feedback, editing cleared it,
and explicit correction/create succeeded with the entered value retained
(1280x720). No unexpected console warning/error or CSP violation occurred;
deliberately exercised HTTP 403/422 resource messages were expected.

Temporary scripts, JSON receipts, screenshots and the synthetic SQLite database
are outside the repository at /tmp/yetlet-yt107-20261021-gFTgDe. The Admin
receipt preserves its passed journey and the subsequent Todo fixture failure;
todo-browser-result.json separately records the completed Todo journey. The
initial immediate mobile-width assertion was corrected to await layout settling.
The first Todo render exposed unsupported hidden markup: the app now hides the
empty error node through its existing DOM binding after mount, and a native
regression validates the complete shared browser View against the DOM policy.
The failed receipts are retained alongside the passing evidence.

The older Todo workspace fixture is outside the CLI's standalone-app contract.
Its original directory permissions were restored and no fixture dist link
remains. A private temporary standalone consumer uses the unchanged Todo main,
the checkout's Todo-server path dependency and existing root dist assets. The
CLI guard and repository manifests were not weakened to run the check. Both
applications and the browser were stopped; no live preview remains.

## Slice 1 observations

Slice 1 does not complete the pending/success/failure recovery workflow or the
full editor/native/Todo comparison. Windows, Firefox, real IME and assistive
technology were not rerun for this bounded slice. No performance claim is made.
Those observations led to the separately selected slice 2 and slice 3 below.

Two concrete costs remain for later assessment: rich application field details
currently require a small app-owned fetch path because HttpExecutor exposes
status/code, and the shared DOM View's attribute subset excludes hidden even
though an application DOM binding can set it. Neither observation changes a
core/public API in this slice.

## Slice 1 independent review

Registered reviewer /root/yt107_field_feedback_review_oct21 returned PASS with
zero issues or required followups. Native child trace verified the reviewer
role, gpt-6-astra model and high effort against the healthy/current/eligible
workspace profile; the observed shared-resolver decision is enforced without
conflict. The reviewer inspected all 13 changed files, relevant contracts,
actual browser scripts/receipts, all three screenshots and the standalone Todo
CLI build log. The reviewer independently checked checkout identity, diff
whitespace and Todo fixture restoration. Native/WASM/Clippy/TypeScript/docs
results were accepted from parent-session command evidence and were not
independently rerun. Acceptance is for slice 1 only. The owner subsequently
authorized local commit 739e2b3 after strict/builtin staged Commit Guard PASS
and matching tree verification. A credential-named synthetic unknown-field
fixture was replaced with a neutral unknown field; its focused rejection test
passed without changing the assertion. No push was made.

## Slice 2 contract

The owner selected practical save state and failure recovery using existing
mechanisms. State belongs to each example; no common forms/transport/session
framework or server endpoint is introduced.

| Concern | Selected behavior |
| --- | --- |
| Admin state | Derive loaded/unsaved/saving/saved/correction/recovery labels from the existing draft, pending item and a small per-draft result. A reply names and updates the submitted item even if another draft is selected |
| Known rejection | Expected validation details and global 422 stay correctable. Editing clears obsolete field feedback; explicit save retries existing rules; Reset restores the loaded baseline |
| Access/conflict/uncertainty | 401/403 require access/session review; 409 requires latest-record comparison; network, malformed or unexpected success replies and other unconfirmed outcomes require checking storage. Keep edits and block save/Reset on that draft so the old token/version or uncertain command is not directly replayed. Other drafts retain their own state |
| Deliberate Admin recovery | Provide selectable draft values containing only Item data, plus latest-record, sign-in and fresh-workspace links in separate tabs. The original draft remains available for comparison/reapplication. A fresh page obtains the current session token and stored version through the existing route. No automatic credential refresh, mutation retry, imported backup or browser persistence |
| Todo request ownership | Claim one create synchronously before queuing its future. Keep input text; disable unchanged-title resubmission after a confirmed create. An edited title expresses a new create intent |
| Todo uncertainty | Block creates, retain the title and offer a read-only GET through the existing /api/todos endpoint. Failed/invalid reads keep the block. A usable server list remains separate from confirmation of the earlier create. After comparing, the user explicitly chooses Start another create and then separately submits; reading/resuming sends no POST |
| Lifecycle/security | Retain pre/post-await lifetime checks, safe text, existing CSP, native routes and authority/CSRF enforcement. No response body, credentials or session token enters feedback/backup |

The Todo app adds a direct dependency on the already locked serde_json version
for its one read-only snapshot decoder and explicitly selects the existing
web-sys Response feature. Cargo.lock changes only that package's dependency
list; no version or framework API changes. This is application control over
submission, not server idempotency, exactly-once delivery or cancellation of a
sent mutation. Page reloads discard these transient guards and drafts.

## Slice 2 verification

Source: 739e2b3 plus the current slice changes, on the same Ubuntu 26.04.1 /
Rust 1.99.0 / Chrome 155 host. Browser plugin not available; existing Playwright
with system Chrome was used. No new browser dependency or cloud resource.

| Command or flow | Actual result |
| --- | --- |
| cargo test --offline -p yetlet-board-triage -p yetlet-todo-browser -p yetlet-todo-ui | PASS: 10 triage, 4 Todo-browser and 10 shared UI tests; offline updates only the selected Cargo.lock dependency edge |
| cargo test --offline --locked -p yetlet-lab-operations-board -p yetlet-todo-server --all-targets | PASS: 20 Board, 8 Todo-server unit and 2 integration tests |
| cargo test --offline --locked --manifest-path examples/admin-dashboard/Cargo.toml --all-targets | PASS: 17 tests; total distinct native tests: 71 |
| cargo check --offline --locked --target wasm32-unknown-unknown -p yetlet-board-triage -p yetlet-todo-browser | PASS |
| cargo clippy --offline --locked -p yetlet-board-triage -p yetlet-todo-browser -p yetlet-todo-ui -p yetlet-lab-operations-board --all-targets -- -D warnings | PASS |
| cargo clippy --offline --locked --target wasm32-unknown-unknown -p yetlet-board-triage -p yetlet-todo-browser -- -D warnings | PASS |
| npm run typecheck --prefix examples/admin-dashboard/frontend | PASS |
| yetlet build --manifest-path examples/admin-dashboard/Cargo.toml | PASS: native app, WASM/bindgen and Vite assets |
| cargo xtask build browser --release; standalone Todo-consumer yetlet build | PASS: browser digest 366f98a576eba04f and native consumer |
| CLI run / SIGINT stop for Admin and Todo | PASS: HTTP ready, both owned listeners closed, Admin drain marker observed |
| cargo xtask check docs; git diff --check | PASS before independent review |

The focused Chrome journey verified loaded/editing/saving/saved labels, one
pending Admin save despite repeated click dispatch, and a delayed success for
item A preserving item B's text/focus. A real competing write produced 409;
the original draft remained selectable/copyable, replay/Reset stayed blocked,
and manual comparison/reapplication from a fresh workspace saved successfully.
A real sign-out/sign-in rotated session CSRF: the old mounted workspace got
403, retained its draft and stayed blocked; a fresh workspace could save the
reapplied values with its current token.

For uncertainty, the test let the server commit, then interrupted the browser
reply. Admin retained the unconfirmed draft, prevented replay and opened a
fresh read that observed the committed value. A wrong-version success payload
also remained unconfirmed. Todo likewise committed before a lost reply, kept
its title and blocked further creates. A simulated GET 503 left recovery
blocked; a real successful GET displayed the committed task but sent no POST.
Explicit resume still sent no POST; a subsequent Create sent exactly once.
A pending response after Todo disposal/remount did not change the replacement
runtime's title/status. No unexpected console error/warning or CSP violation;
the deliberate 403/409/422/503 and interrupted-resource errors were expected.

Admin screenshots were visually inspected at 1440x1000 and 390x844; Todo at
1280x720. Evidence is outside the repository in the slice2 subdirectory of
/tmp/yetlet-yt107-20261021-gFTgDe: browser.mjs, browser-result.json, three build
logs and recovery screenshots. The first operational startup attempt used the
wrong Cargo target directory for Admin, triggering a readiness timeout during
unnecessary recompilation. The canonical fixture now keeps each app's existing
target directory, and cleanup requires a drain marker only after app readiness.
Its failed receipt/script remain preserved beside the passing run. No product
change was made for that environment setup failure. Both apps and Chrome were
stopped. The existing Todo standalone consumer was reused without modifying the
repository fixture's permissions or adding a dist link.

Changed files are Cargo.lock; this report, BACKLOG.md and ROADMAP.md; Admin
frontend/app.css; Board-triage src/lib.rs and src/browser.rs; Board-server
src/triage.rs; Todo-browser Cargo.toml, src/lib.rs and new src/remote.rs; and
Todo-ui src/lib.rs.

## Slice 2 independent review

Registered reviewer /root/yt107_save_recovery_review_oct21 returned PASS with
zero issues and required followups. Native child trace verified reviewer /
gpt-6-astra / high against the healthy/current/eligible workspace profile; the
observed shared-resolver decision is enforced without conflict. The read-only
review covered all 12 changed files, relevant contracts, adjacent server and
executor behavior, focused test code, browser script/receipt, all four
screenshots, three build logs and the preserved operational failure. Checkout
identity, diff whitespace and Todo fixture preservation were independently
checked. Native/WASM/Clippy/TypeScript/docs results were accepted from parent
session evidence rather than rerun. Acceptance covers slice 2 only. The owner
subsequently authorized local commit 15c0e7d after strict/builtin staged Guard
PASS and exact tree verification. No push was made.

## Remaining limitations

Recovery stays intentionally manual: copying/reapplying values does not merge
concurrent changes or guarantee exactly-once delivery. Todo allows another
create only after a successful review plus explicit user intent; that is not
proof whether the earlier request committed, nor automatic deduplication of
business content. A fresh/reloaded page has no memory of transient submission
guards. There is no browser draft persistence, import, automatic retry or
in-place token refresh. A sent mutation may finish after runtime disposal.

Windows, Firefox, real IME/assistive-technology and performance qualification
were not rerun. The final comparison follows; these accepted limits apply to
the whole selected application slice.

## Application authoring guide

Choose an existing owner for the interaction. This guide describes working app
code, not a stable framework forms API; the earlier
[form-binding comparison](YT-105-FORM-BINDING-RESEARCH.md) and
[HTTP-feedback study](YT-105-HTTP-FEEDBACK-RESEARCH.md) retain their research scope.

| Concern | Native Admin form | WASM triage: textarea or CodeMirror | Todo remote Create |
| --- | --- | --- | --- |
| Draft owner | Current HTML form, bound once through native Axum Form | Workspace owns per-item values/version/errors/result; GraphHandle::snapshot reads committed state | Native input owns text; RemoteForm owns submission/review state |
| Server rule | Shared Fields::errors for five fields; title 160, Description 4,000 characters | The same Fields::errors through the existing JSON save route | Existing TodoCommand rule: one title, 100 characters; its own domain |
| Field feedback | HTTP 422 document with submitted values, summary and named errors | Expected 422 details attach to the submitted draft, even after another item is selected | Only existing invalid_title maps to the title control |
| Accessibility | Native label, visible message, aria-invalid and error reference | Same server message; Description also keeps its hint; CodeMirror's actual contentDOM receives the association | Native title input/error and live result summary |
| Correction | Inputs change locally; server-rendered errors remain until another POST returns a document | Explicit input/change clears only that field's stale error; other drafts remain | input clears the title's stale feedback; explicit click submits |
| Success | POST/303 to the stored detail page | Matching ID/version updates that saved draft; another selected draft/focus stays intact | Snapshot updates the separate result host; text stays; unchanged confirmed title cannot be submitted twice immediately |
| Reset | Restore page values uses current HTML defaults, including rejected values after a 422 page | Reset draft restores loaded/saved values and clears errors; item switch/Reset starts fresh editor undo history; recovery states block Reset | No Reset control; text remains available |
| Recovery | 409/422 retain bound form values; latest-record link opens separately. Authority/malformed/global failures remain ordinary HTTP failure pages | Copy Item data, compare latest record, reapply in a fresh page with current token/version; no direct replay from a blocked draft | Read existing /api/todos after uncertainty, compare, explicitly resume, then separately create |
| JavaScript | Native routes work without it | WASM only on triage; failed package enhancement leaves the textarea; without WASM use native item links | The optional browser route requires JS/WASM; the separate SSR root stays script-free |

Keep shared domain rules and safe messages consistent. Preserve intentional
ownership differences: native error clearing requires a document response,
Reset baselines differ, and Todo does not gain Admin's session capabilities or
field limits. Native maxlength is a browser guard; the server remains the
validation authority. Exact Unicode entry-limit parity is not qualified here.

### Apply the existing mechanisms

1. **Choose the owner first.** Use native Form for SSR, a .value() read at the
   action for one DOM-owned draft, or Workspace/local graph for live multi-draft
   behavior. Programmatic Reset is separate from an input event and HTTP save.
2. **Keep authority on the server.** Authenticate, authorize and verify CSRF
   before feedback. Reuse the domain rules and actual HTTP status. Select only
   necessary bootstrap values; never serialize settings/session/provider objects.
3. **Carry only needed detail.** Todo uses HttpExecutor status/code. Admin's
   private save returns SaveFailure::Validation(FieldErrors) because that executor
   does not carry a field map. Reuse its endpoint validation, bound/whitelist the
   five known fields, and render text. Do not hide field JSON inside code. See
   [save/bindings](../examples/board-triage/src/browser.rs) and
   [payload/reducer](../examples/board-triage/src/lib.rs).
4. **Keep submission identity and lifetime.** Claim pending before queuing the
   future, capture the submitted ID/title and retain pre/post-await active checks.
   Never attach A's errors to B. Failed/unexpected confirmation retains the draft;
   read-only review does not confirm or repeat the earlier mutation. See
   [Todo submission state](../examples/todo-browser/src/remote.rs).
5. **Give a widget one subtree owner.** Rust owns draft data and the native
   fallback; the app-owned TS instance owns the CodeMirror subtree. Use its
   connect/update/destroy boundary. Equal-value/error-only updates preserve the
   document/view; use the current contentDOM for ARIA. Keep the textarea usable
   if enhancement fails. See the
   [Description integration](../examples/admin-dashboard/frontend/description-editor.ts).
6. **Validate the actual renderer and changed journey.** SSR support does not
   prove a shared View is accepted by the bounded DOM renderer. Todo's existing
   complete-page validate regression catches unsupported markup. Retain listener
   owners, avoid replacing foreign editor subtrees, and reuse focused tests plus
   CLI build/run/stop. Check actual correction and the changed failure path;
   use existing evidence for unchanged paths.

### Concrete costs and API decisions

| Demonstrated cost | Current decision | Condition for a future shared change |
| --- | --- | --- |
| Admin's rich errors need private fetch/decode glue | Keep it app-owned; Todo still uses status/code | A second real rich-error consumer and measured repeated code before changing ActionError or transport contracts |
| Shared DOM View rejects hidden, while a live DOM binding can set it | Retain post-mount binding and complete-page validation | Separately select narrow safe-attribute support, starting with the demonstrated hidden case and preserving executable-markup rejection; no arbitrary attribute bag |
| Error/disabled/listener/lifetime glue remains explicit | Keep existing native Form, local graph, RemoteForm and TS instance | Demonstrated policy repetition in equivalent consumers; multi-draft/versioned edit differs from anonymous create |
| Native and WASM Reset had the same ambiguous label | Native now names/explains page values; latest-record recovery opens separately | No custom native reset runtime is needed |

No common forms runtime, provider registry, validation scheduler, general
frontend adapter or public API change is selected. Feedback adds no settings;
existing settings and CLI remain the setup/execution boundary. The safe hidden
attribute is a concrete future candidate, recorded here without registering or
implementing a new item.

## Slice 3 verification and closeout

Source: 15c0e7d, followed only by the native form label/help and safe latest-record
link changes in [views.rs](../examples/lab-operations-board/src/views.rs), plus
these documentation updates. The earlier 71-test/WASM/TypeScript/build and
browser evidence remains applicable to unchanged implementation paths.

| New focused verification | Actual result |
| --- | --- |
| Enhanced Chrome journey, 1440x1000 | PASS: actual title/Description 422, same editor node/undo history, selection across a graph commit without focus change, per-field clearing and explicit successful save |
| Package-import failure, Chrome 1440x1000 | PASS: textarea fallback uses identical server messages, hint/error ARIA, retained raw values, per-field correction and successful save |
| Native JS-off form, Chrome 390x844 | PASS: identical JSON/server field messages and ARIA, rejected values retained, 422 writes nothing, HTML Reset returns page defaults, correction POST303 persists |
| Native-only follow-up after the copy/link change | PASS: Restore page values label, latest record opens separately while failed form remains, same validation/correction behavior, no mobile overflow |
| APP_CLI__VITE__AUTO_BUILD=false yetlet build --manifest-path examples/admin-dashboard/Cargo.toml | PASS: settings select a native-only build; unchanged slice-2 frontend assets reused |
| cargo test --offline --locked -p yetlet-lab-operations-board persisted_text_history_and_conflict_keep_safe_drafts | PASS: 1 existing focused test, 19 filtered |
| cargo clippy --offline --locked -p yetlet-lab-operations-board --all-targets -- -D warnings | PASS |
| CLI run / SIGINT stop | PASS for both focused browser invocations; listeners closed and Admin drain marker observed |
| cargo xtask check docs; git diff --check | PASS: English docs, links, 107-item DAG and whitespace |
| Visual inspection | PASS: editor/fallback field panels at desktop size and the corrected JS-off native form at 390x844; no clipping/overlap |

Browser plugin not available; existing Playwright/system Chrome 155 on Ubuntu
26.04.1 / Rust 1.99.0 was used. No unexpected console error/warning or CSP
violation; deliberate 422 responses and the aborted package import were expected.
Evidence is outside the repository in the slice3 subdirectory of
/tmp/yetlet-yt107-20261021-gFTgDe: browser.mjs, browser-result.json,
native-result.json, native-cli-build.log and editor/fallback/native screenshots.
The first assertion conflated selection with an ordinary Save/focus/read-only
transition; its failed script/receipt is retained. The corrected check isolates
selection across a non-focus-changing equal graph commit, and checks undo after
actual rejection. Selection through focus-changing Save is not claimed.

Changed files: this report, BACKLOG.md, ROADMAP.md, docs/README.md,
YT-094-API-OBSERVATIONS.md, examples/admin-dashboard/README.md and the native
Board views.rs. No dependency, public API, settings or browser implementation
changes are introduced by the final slice.

## Selected acceptance reconciliation

| Selected condition | Evidence/disposition |
| --- | --- |
| Server validation reaches the correct fields safely | Slice 1: all five existing Admin server rules, fixed bounded messages, authority/CSRF rejection without detail leakage or writes; Todo existing code mapping |
| Submitted draft identity, values and lifetime survive | Slice 1 reducer/browser proof and slice 2 pending/late-response flows; no reply for A alters B or a replacement runtime |
| Clear save state and deliberate failure recovery | Slice 2 actual success, 409 competing write, changed-session 403, committed/lost reply, wrong-version reply, Todo failed/successful read-only review; no automatic replay |
| Editor/native/fallback consistency is practical | Slice 3 actual matching messages/ARIA, field correction/save, retained editor view/undo, ordinary native POST422/303 and explicit Reset baselines/new-tab recovery |
| Existing settings/CLI/CSP boundaries remain usable | Slice 1/2 build/run/stop and slice 3 native-only CLI build/run/stop; strict CSP and package-failure fallback; no extra configuration/framework |
| Developer guidance and observed costs are actionable | Authoring guide links actual owners, transport and widget seams; rich-error and hidden-attribute costs recorded with prerequisites for future changes |
| Independent final review | PASS: all three selected slices accepted at bounded application scope, with zero issues/required followups; reviewer explicitly permits YT-107 Done |

YT-107 is Done at the selected application form/save-feedback scope. Acceptance
does not imply broader platform/production/release qualification. The final slice is
not committed or pushed. At this reconciliation the other remaining registered
items are release work: YT-096 stays In progress and YT-020 stays Blocked; signing
and publication remain deferred to an explicitly selected release.


## Slice 3 independent review and final status

Registered reviewer /root/yt107_consistency_closeout_review_oct21 returned PASS
with zero issues/required followups and explicitly accepted marking YT-107 Done
for all three selected slices. Native child trace verified reviewer / gpt-6-astra /
high against the healthy/current/eligible workspace profile; the observed
shared-resolver decision is enforced without conflict. The reviewer directly
inspected the seven-file diff, relevant ADRs/adjacent source and tests, current
and prior browser/reviewer records, three screenshots and the native build log,
and independently checked whitespace. Focused native test, Clippy and docs
results were accepted from parent-session evidence without repeating accepted
workloads. Cleanup is supported by the receipts and script teardown. Manual
recovery, transient drafts, native HTTP failure semantics, selection limits and
the stated platform/performance/release exclusions remain accepted boundaries.
The final slice is uncommitted. This review grants no delivery/release authority.
