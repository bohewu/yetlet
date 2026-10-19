# YT-105 — HTTP feedback: CSRF and field validation

Date: 2026-10-19. Inspected source: `9d0f735`, after the reviewed
[state-read/initial-render study](YT-105-STATE-READ-INITIAL-RENDER-RESEARCH.md).
This is the third research comparison, not implementation or an accepted new
public API. Independent Reviewer `/root/yt105_http_feedback_review_oct19`
PASS has zero issues or required followups.

## Recommendation

Keep Admin's existing session CSRF token in the JSON command body. A header hook
is not required by either inspected consumer. Improve validation at the
application boundary first: Todo can map its existing `invalid_title` code to a
friendly title error; Admin needs its existing server field errors carried
through to its draft UI. Neither improvement is implemented by this study.

For a separately selected Admin trial, prefer one private HTTP save adapter
with an application-specific error result, a small application-owned validation
payload and field-error state. Keep `HttpExecutor`, `ActionExecutor`,
`ActionError` and `ApiErrorBody` unchanged initially. This costs a private fetch
path, but tests the one demonstrated rich-form requirement without imposing a
generic error contract on every command or transport. Reconsider a shared seam
only if another consumer needs the same details and this trial shows useful
repetition. Ordinary typed HTTP functions remain consistent with
[ADR 0002](adr/0002-events-and-actions.md) and native handler composition in
[ADR 0006](adr/0006-handler-first-mvc-optional.md).

## Current consumers and source cost

| Concern | Admin triage | Todo Browser / Todo server |
| --- | --- | --- |
| Request | `Save { csrf, item }`; item includes ID/version and five editable fields | `TodoCommand::Create { title }` / `Toggle { id }` |
| Authority | Session identity, project/write checks, session CSRF verification and configured Host/Origin guard | Anonymous in-memory demonstration; Origin/content-type checks and 4 KiB body limit; no session authority |
| Server validation | `Fields::errors(project)` supplies five named field rules shared with native forms; JSON save discards the details and returns 422 `validation` | Domain `Validation("invalid_title")` becomes 422 with that code and a fixed generic message |
| Native form baseline | Invalid POST renders the submitted values, summary and field-specific messages, with `aria-invalid` / `aria-describedby` | The compared HTTP click flow is a separate demonstration, not Admin's authenticated form |
| Browser failure feedback | Four fixed status branches, including a generic 422 message; 401/403 share one recovery message | One fixed prefix plus `error.code()` in a text node |
| Draft / lifetime policy | Per-item drafts, one pending save, item/version checks and pre/post-await lifetime guard | Native input value retained; pre/post-await lifetime guard; no rich form-error model |

Source anchors, all relative to the inspected revision:

- [Admin browser](../examples/board-triage/src/browser.rs): CSRF read at 241,
  failure mapper at 219–225 (7 physical lines), save callback at 345–364
  (20 lines, including command construction, pending checks and lifetime work).
  [Save / Workspace](../examples/board-triage/src/lib.rs), lines 56–95 and
  140–213, have no field-error state; `Failed(String)` retains drafts.
- [Admin JSON handler](../examples/lab-operations-board/src/triage.rs):
  error mapping at 167–191 (25 lines), save at 192–228 (37 lines).
  Lines 213–215 discard the nonempty field-error list. This is one loss point;
  adding a transport decoder alone would not restore missing server details.
- [Shared field rules](../examples/lab-operations-board/src/domain.rs),
  lines 121–159, return static field names and messages. The
  [native POST](../examples/lab-operations-board/src/lib.rs), lines 651–714,
  and [native view](../examples/lab-operations-board/src/views.rs), lines
  186–209 and 241–273, already demonstrate useful field feedback.
- [Todo browser](../examples/todo-browser/src/lib.rs), lines 294–337,
  constructs the HTTP action; lines 330–334 (5 lines) display its error code.
  [Todo API](../examples/todo-server/src/api.rs), lines 46–74 (29 lines), builds
  the envelope and status/message mapping. These server mappings are
  application policy, not duplicated fetch infrastructure.
- [Common executor](../crates/yetlet-actions/src/lib.rs), lines 128–161
  (34 lines), serializes, fetches, reads and decodes the response. The
  non-success branch at 155–159 retains only status/code, discarding message;
  the current envelope has no field details at all.

Counts include blank lines and mixed responsibilities; they are source costs,
not estimated net savings or performance measurements. Both applications already
reuse the common fetch path. Only Admin currently demonstrates a need for a
multi-field result, so no repeated rich-error decoder has been established.

## CSRF: preserve the working contract

Admin generates a session token, escapes it into a hidden DOM input, then
explicitly serializes it into `Save.csrf`. The server authenticates/authorizes,
binds the JSON, verifies the session token, validates fields and checks authority
again before storage. Missing/invalid schema produces 400; a supplied invalid
token produces 403. The separate Host/Origin middleware may reject before this
handler and return HTML instead of its JSON envelope.

[OWASP's synchronizer-token guidance](https://cheatsheetseries.owasp.org/cheatsheets/Cross-Site_Request_Forgery_Prevention_Cheat_Sheet.html#transmitting-csrf-tokens-in-synchronized-patterns)
allows a token returned in HTML and submitted in JSON. Keep token validation
on the server, avoid token URLs/logging, and retain existing origin/cookie
defenses. A custom header can be useful for a backend that requires it, but its
absence here is not evidence of broken CSRF protection. This comparison is not
production security qualification.

The executor does not set a credentials option. The
[Fetch standard](https://fetch.spec.whatwg.org/#concept-request-credentials-mode)
defaults credentials to same-origin; same-origin session cookies can accompany
this POST subject to cookie policy. No cross-origin credential option is needed
for these callers. Endpoint validation in `HttpExecutor::new` restricts the
initial endpoint path; it does not establish a general redirect/CORS policy.

Do not label every 403 as "CSRF expired": project permission, token rejection
and origin rejection share this status. Keep a global recovery message and the
draft, with no automatic token refresh or replay. Signing in in another tab can
change the session token; it does not update this mounted workspace's captured
token. Recovery must deliberately obtain a fresh page/token and review the latest
record while preserving/reapplying the local draft. Blindly retrying the same
captured command is not a recovery contract. Automatic draft persistence or
in-place credential refresh is outside this selected recommendation.

Todo's anonymous API is not evidence of session CSRF protection. Do not add a
session module merely for feedback symmetry; adopting cookie-based authority
later requires its own deliberate server-side CSRF policy.

## Minimal proposed field-feedback contract

This is a future application trial, not a new framework schema:

1. After existing authority/CSRF checks, reuse `Fields::errors` and return
   actual field violations with status 422, existing `error.code = "validation"`
   and a fixed summary. Add an application-owned `fields` member, for example
   `{"title":"Enter 1 to 160 characters without control characters."}`.
   Allow only title, description, status, priority and assignee, at most one
   message per field, using the existing static server messages. Do not include
   rejected values, token/session data, database errors or raw HTML.
2. Decode those details only for the expected 422 validation response. Keep
   malformed requests, 401/403, 404, 409, storage failures and network/parse
   failures global. A transition rejected against stored state can remain a
   422 summary without invented field highlights; it is not the same as an
   invalid local field. Missing/unknown details fall back to the fixed summary.
   No validation failure becomes a successful HTTP 200 result.
3. Associate errors with the submitted item ID and its pending save, captured
   before await. The server need not echo an item identifier to select a draft.
   The existing one-pending-save rule prevents overlapping saves, but users can
   select/edit another draft while waiting. Never apply item A's errors to B.
   Clear that field's stale error on edit and the item's errors on Reset or
   successful save; other drafts retain their own errors and values.
4. Render plain text and accessible associations for the selected draft, reusing
   the native form's error pattern. Preserve existing live summary feedback;
   clear obsolete `aria-invalid` / error references when the error clears.
   Keep focus on the user's current control, including when a different draft
   is selected before the reply. No automatic focus move is necessary here.
   [W3C's form notifications tutorial](https://www.w3.org/WAI/tutorials/forms/notifications/)
   explains text summaries and field-message associations; this proposal does
   not claim an accessibility qualification run.
5. Preserve item/version success checks, drafts, one explicit save per click,
   native no-JS routes and current runtime lifetime guards. Non-JSON or invalid
   replies remain failures; never insert a response body as markup. Continue
   treating an unconfirmed save as uncertain and require checking stored state
   before retrying. Rich validation feedback does not add idempotency or cancel
   an already sent mutation when the runtime is disposed.

The private adapter would use ordinary fetch/serde and return an app-owned
`Result<Item, SaveFailure>`; only that app-specific error needs field details.
It can reuse `HttpExecutor::new` / `endpoint()` to validate the configured path,
while keeping same-origin POST/JSON and existing cookie semantics explicit.
Reuse the current fetch behavior as the baseline; this study does not add a
streaming response-limit subsystem, automatic retries or a token service.

Importantly, this is an ordinary application method/function, **not** an
`ActionExecutor<Save>` implementation: the current trait fixes its error to
`ActionError`, whose `Remote` variant contains only status/code. A custom
executor cannot return arbitrary typed field errors through that trait today.
An out-of-band last-error store or encoding field JSON into `code` would hide
state/transport semantics and is not selected. Successful data can stay `Item`;
the semantic command and native Axum handler remain typed.

For Todo, a local mapping of `invalid_title` to its one title control and a fixed
friendly message needs none of the Admin schema or private adapter. Keep unknown
codes as global failures. This lower-cost option is also only a proposal.

## Alternatives and tradeoffs

| Option | Assessment |
| --- | --- |
| Document body-token CSRF and retain generic feedback | Valid current baseline; no implementation required, but Admin keeps its observed native/WASM feedback gap |
| Add only a request-header hook | Does not restore field details; neither inspected server requires a header token. Defer until a concrete caller requires it |
| Custom `ActionExecutor<Save>` alone | Cannot carry typed field errors with today's fixed `ActionError`; changing transport does not remove that limitation |
| One private Admin adapter and payload | Recommended first trial if richer feedback is selected; no core API changes, at the cost of duplicating the small fetch/decode path and its compatibility obligations |
| Extend common `ApiErrorBody` and `ActionError::Remote` | A possible later shared seam; more compact shared fetch use, but field policy becomes public and Rust struct/variant literal callers must migrate. Current code has two server envelope constructors and a custom TCP executor/test constructing Remote, so adding fields is not Rust-source compatible merely because JSON members are optional |
| Generic decoder callbacks, arbitrary error type parameters or a form runtime | No demonstrated second rich-error consumer; larger configuration/type/lifecycle contract than this need |

If a second real consumer justifies common details, compare a small optional
field member with the private adapter using actual code costs before selecting
it. Preserve status/code access and simple callers; do not preselect arbitrary
JSON metadata, a universal result hierarchy, a DI container or a new transport.
The existing [custom TCP example](../examples/custom-action-transport/src/lib.rs)
demonstrates transport injection, not a field-feedback implementation.

## Minimum later verification

Existing [Admin tests](../examples/lab-operations-board/src/tests.rs), especially
`triage_json_reuses_authorization_csrf_validation_and_atomic_conflicts`, already
assert 401/403/400/422/409, a successful versioned write and no rejected writes.
The [Todo API tests](../examples/todo-server/src/api.rs) assert typed codes,
unchanged invalid-command state and origin/content-type/body limits. They were
inspected, not rerun as proof of unimplemented feedback.

If the Admin trial is selected, extend those existing seams only:

| Scenario | Necessary focused evidence |
| --- | --- |
| Authority and token | Valid token saves; invalid/missing token, session/project/write rejection and disallowed origin do not mutate storage or disclose field details. Non-JSON 403 remains a global failure |
| Field / global split | Each known invalid field and a multi-field request yield bounded safe details with 422 and no write; malformed JSON stays 400, state transition stays a summary and conflict stays 409 |
| Browser recovery | Submitted values survive; field messages/ARIA target the submitted draft, clear on edit/Reset/success, and a reply for A never marks B or moves its focus |
| Lifecycle and uncertain save | Existing pending/lifetime/version checks remain; disposed replies do not alter a replacement runtime; network/malformed replies never announce success or auto-replay |
| Simple caller | If Todo mapping changes, invalid_title identifies its title using text; unknown failures retain the global fallback and the native value |

Reuse existing native/browser journeys for any selected UI change; no new
platform/IME matrix, cloud service or validation framework follows from research.

## Verification and research boundary

This slice changes only this report, the documentation index, backlog and roadmap.
Verification comprises current-source inspection, official reference comparison,
scoped local-link/whitespace checks and `cargo xtask check docs`. No executable
API, test, dependency, server, browser session or external environment is changed.

Together with the two earlier reviewed comparisons, this covers YT-105's
research topics. Independent review explicitly approved research-only closure:
YT-105 is Done at this bounded research scope; no proposed API or UI improvement
is implemented. Implementation remains a separate maintainer decision.
The next practical candidate is the already proposed weak-handle snapshot;
YT-106 interoperability research is independently unblockable if selected.
