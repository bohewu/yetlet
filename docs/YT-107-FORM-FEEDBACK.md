# YT-107 — Admin and Todo form/save feedback

Date: 2026-10-21. Status: In progress. Owner-approved implementation follows
the [YT-105 HTTP-feedback research](YT-105-HTTP-FEEDBACK-RESEARCH.md), keeping
the existing application boundaries, typed settings, CLI and server authority.
YT-105's completed research and earlier implementation remain Done.

## Selected scope and slices

| Slice | Delivery | State |
| --- | --- | --- |
| 1. Server validation to field messages | Carry Admin's five existing server field rules through its application-owned JSON save boundary into the correct draft/control; map Todo's existing invalid_title code to its title control. Preserve entered values, safe text/ARIA associations, current authorization/CSRF/status behavior, native routes and lifetime checks | Verified; independent Reviewer PASS |
| 2. Draft/save state and recovery | Improve the complete editing/pending/success/failure workflow and deliberate recovery for session/permission changes, conflicts and uncertain network outcomes; preserve other drafts and prevent duplicate submissions | Planned; not implemented by slice 1 |
| 3. Integration and developer experience | Complete editor/native/Todo consistency, document actual application patterns and repeated costs, assess demonstrated API gaps, and qualify the complete selected journey | Planned; only compatibility work required by slice 1 is included now |

The current authorization selects slice 1 and its focused verification/review.
No general forms runtime, DI container, generic error hierarchy, new public
transport API, automatic retry/credential refresh, signing or publication is
selected. The full item remains In progress after a passing first slice.

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

## Remaining scope and observed authoring costs

Slice 1 does not complete the pending/success/failure recovery workflow or the
full editor/native/Todo comparison. Windows, Firefox, real IME and assistive
technology were not rerun for this bounded slice. No performance claim is made.
YT-107 stays In progress; slice 2 is the next implementation slice.

Two concrete costs remain for later assessment: rich application field details
currently require a small app-owned fetch path because HttpExecutor exposes
status/code, and the shared DOM View's attribute subset excludes hidden even
though an application DOM binding can set it. Neither observation changes a
core/public API in this slice.

## Independent review

Registered reviewer /root/yt107_field_feedback_review_oct21 returned PASS with
zero issues or required followups. Native child trace verified the reviewer
role, gpt-6-astra model and high effort against the healthy/current/eligible
workspace profile; the observed shared-resolver decision is enforced without
conflict. The reviewer inspected all 13 changed files, relevant contracts,
actual browser scripts/receipts, all three screenshots and the standalone Todo
CLI build log. The reviewer independently checked checkout identity, diff
whitespace and Todo fixture restoration. Native/WASM/Clippy/TypeScript/docs
results were accepted from parent-session command evidence and were not
independently rerun. Acceptance is for slice 1 only. No commit or push was made.
