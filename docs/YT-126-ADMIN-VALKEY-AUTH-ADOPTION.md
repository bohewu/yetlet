# YT-126 — Opt-in Admin adoption of Valkey/Auth resolution

**Date:** 2026-10-22. **Status:** Done at the selected application-adoption scope after independent Reviewer PASS.
**Source baseline:** `67e7ffb59c855bac193367647d1e2e72dd44d956`.
**Authority:** The owner selected Admin adoption after the separately reviewed
[YT-124 contract](YT-124-MULTI-INSTANCE-AUTH-SESSION-CONTRACT.md) and
[YT-125 local integration](YT-125-VALKEY-AUTH-INTEGRATION.md). Commit and push
are explicitly unselected.

## Selected application scope

Admin already delegates its routes to the Lab Operations Board. This slice
adopts the existing native `yetlet-auth` resolver/extractor contract there,
using the existing async Valkey Session provider. The implementation remains
application-owned. It adds no framework, Auth/provider library API, new crate,
provider registry, mandatory dependency or Core API.

Compilation and runtime selection are both explicit. Existing Memory and
Valkey-only constructors/settings retain their previous behavior. Selecting
the new resolver requires the compiled capability, Valkey and a trusted policy
file; incompatible selection fails startup. Unselected applications do not
open that file or install the new resolver. Server-only dependencies remain
outside browser builds.

Admin's `auth-valkey` feature forwards the native selection to the Board.
Runtime settings default to `[auth] enabled = false`; selecting
`APP_AUTH__ENABLED=true` requires `session.backend=valkey` and an absolute
`APP_AUTH__POLICY_FILE` on Linux. The application composition seam is
`SessionBackend::AuthValkey`; existing constructors and the previous Valkey
variant are retained. See the [Admin usage](../examples/admin-dashboard/README.md)
for the explicit build/run and policy configuration.

## Identity and current local policy

Existing login verification establishes a protected server-owned local account
reference. Generic Auth identity uses an explicit application-local namespace,
local account subject, Session scheme and validated project binding. It does
not invent an external OIDC issuer/subject from a local name. Generic Principal
is never serialized or included in hydration.

The canonical identity namespace is `urn:yetlet:admin:local-accounts`, scheme
`admin-session`, subject `alice`, `bob` or `viewer`, and tenant `project-1` or
`project-2` from the compiled account binding. Existing stored local names stay
compatible. Explicit `user: null` denotes an anonymous record; a missing user
field is malformed rather than an anonymous identity.

OIDC verification, state/nonce/PKCE, atomic pending-state consumption, CSRF and
Session rotation stay with the existing maintained client and application.
The OIDC subject map remains startup login-admission configuration. Its existing
restart/freshness limits remain; local policy changes do not promise upstream
IdP disable, live subject remapping or upstream logout.

Both instances read the same bounded, trusted single-host JSON policy file
afresh on each selected decision. Operator updates atomically replace the
file. Account enablement and read/write restrictions intersect the compiled
local account/project limits: policy can revoke permission, but cannot promote
Viewer to writer or move an account to another project. Missing accounts deny;
malformed, oversized, missing or unavailable authority returns a redacted 503.
There is no policy cache, public policy-edit endpoint, SQLite account migration
or new authorization service.

The strict JSON shape is `{"accounts":{"alice":{"enabled":true,"read":true,"write":true}}}`,
with the same optional entries for `bob` and `viewer`. Missing/null entries
deny that account; unknown fields/accounts and incomplete flag objects are
invalid authority. Reads have a 4096-byte limit and a 250 ms deadline, check the
opened handle is a regular file, and use `O_NONBLOCK | O_NOFOLLOW` on the
selected Linux platform. Permission is a fresh decision snapshot; operator
replacement is not an acknowledgement that all in-flight work has stopped.

## Request, lifecycle and authorization boundaries

Session middleware runs outside Auth on selected protected routes. Resolution
clones the exact typed remote handle, performs an authoritative read and checks
current policy without consuming the request body. Protected handlers consume
the resolved Auth identity, then independently enforce current account,
project and read/write permission.

Only a valid explicit anonymous record is an anonymous resolution. Missing,
expired, rotated or revoked protected state uses the selected fixed local
login navigation. Invalid cookies remain 400, denied policy 403, unavailable
policy/provider 503 and missing/misordered Auth configuration 500. Unknown
mutations never acknowledge success, emit a success cookie or replay a callback.

Login-page anonymous creation, existing demo/OIDC verification and native
CSRF-protected logout remain explicit lifecycle routes. Disabled accounts can
still revoke their Session through logout. Public health and assets remain
outside the new Auth scope. Preserve route 404/405 behavior, exact Host/Origin
checks, strict CSP, native SSR, safe JSON feedback and provider-owned cookie,
no-store and Vary semantics.

Policy governs protected identity/resource decisions. Existing credential-proof
admission and the OIDC callback remain lifecycle operations: a rotation can
succeed while the separate policy authority is unavailable or denies the
account. That Session record does not grant Auth identity or resource access;
the next protected decision returns 503 or 403 as applicable. No policy-aware
login-admission or live external subject-map guarantee is added. Early Auth
rejections use native responses; triage retains its existing JSON envelope for
resource/write denials reached inside its handler.

Fresh account/resource checks cover native pages, triage, both save paths and
the existing SSE revalidation loop. The final pre-SQL check supplies the
principal passed to the business save, rather than an earlier permission
snapshot. Already-rendered documents are not automatically replaced by policy
updates; an already-admitted response is not instantaneously cancelled.

## Existing write consistency limit

Native and triage saves retain their existing Session/policy-check-then-SQL
behavior. A final check can succeed before logout or policy replacement while
its SQL operation remains queued or running. Testing allowed saves, denied
requests and optimistic conflicts proves compatibility and fresh decisions;
it does not qualify strict prevention of commits after revocation.

No shared transaction, fencing rule, second Session ledger or distributed lock
is selected here. The stronger sensitive-write ordering gate in YT-124 remains
open and needs a separate application decision. This adoption does not extend
the SQLite transaction to Valkey or to the policy file.

## Focused verification and review gate

Verify only newly changed behavior: configuration/DTO and application glue
tests, one opt-in actual two-Admin-process HTTP journey against a task-owned
disposable Valkey, strict lint/format, dependency comparisons and documentation
checks. The live journey must execute rather than count an ignored test as
success. It covers cross-instance login/logout, current policy denial,
native/triage compatibility, restart and fail-closed outage/recovery. Bound and
clean up owned app processes, temporary files and the isolated service.

The prior [YT-094 Valkey/TTL](YT-094-VALKEY-SESSIONS.md),
[shared SQLite](YT-094-SHARED-SQLITE.md), [OIDC](YT-094-OIDC.md),
[YT-103 rolling-assets](YT-103-ROLLING-ASSET-VERIFICATION.md) and YT-125
results retain their recorded source/topology limits. Do not repeat their
passing browser, IdP or deployment workloads. No frontend layout, IME,
assistive-technology, Windows, real-provider, persistence, HA or multi-host
qualification is selected. The earlier Auth security scan does not certify
this new consumer change.

Actual focused verification, cleanup and independent review are recorded below.
The initial failures and corrections remain part of the evidence history.

## Actual focused results

| Check | Actual result |
|---|---|
| New `auth::tests` in Board with `oidc,auth-valkey` | PASS: 2 tests, 28 filtered; account/project/write bounds, identity binding, fresh/bounded policy and missing-layer failures |
| Expanded `sessions::tests::stored_identity_selects_local_permissions_and_rejects_forged_state` | PASS: 1 test, 29 filtered; existing valid references plus forged identity/permission and missing-field rejection |
| New Admin `config::tests::auth_selection_requires_valkey_feature_and_absolute_policy_path` | PASS with `auth-valkey` and without it: 1 test, 17 filtered in each mode; selected/unselected configuration only |
| `cargo clippy --offline --locked --manifest-path examples/admin-dashboard/Cargo.toml --features auth-valkey --all-targets -- -D warnings` | PASS after final method-layer repair |
| `cargo clippy --offline --locked -p yetlet-lab-operations-board --features oidc,auth-valkey --all-targets -- -D warnings` | PASS after final method-layer repair |
| `cargo check --offline --locked --manifest-path examples/admin-dashboard/Cargo.toml` | PASS for the default unselected application after that repair |
| Focused assigned-source formatting | PASS; only changed application Rust files selected |
| `cargo test --manifest-path examples/admin-dashboard/Cargo.toml --features auth-valkey --offline --locked --test auth_valkey -- --ignored --exact two_admin_processes_use_fresh_auth_policy --nocapture` | Parent actual execution PASS: 1 passed, 0 failed/ignored/filtered; test 4.59 seconds, complete command 6.77 seconds, with explicit task-owned `YT126_VALKEY_URL` |
| Normal dependency metadata | Admin default byte-identical to its pre-edit snapshot; Core/server/Todo Browser/Search Island byte-identical to retained YT-125 graphs. Browser metadata targets `wasm32-unknown-unknown`; no repeated browser compile/workload |

The live test compiles and launches the real Admin binary, rather than the
YT-125 synthetic consumer. A and B have different generated credential proofs
and share the isolated Valkey namespace, atomically replaced policy and existing
local SQLite database. No credential value is printed or retained in reports;
the child stdout reader holds demo codes only in memory. Existing built frontend
assets are reused; no frontend source/build/browser or real-IdP workload ran.

Actual observations include:

- Health, native 404/405 and malformed/duplicate cookies; missing and explicit
  anonymous protected access uses fixed 303 navigation.
- Valid demo login on A rotates Session/CSRF, rejects the old anonymous Session/CSRF and
  authenticates on B. Wrong project and Viewer writes deny.
- Native create/update and optimistic conflict, plus JSON triage update/conflict,
  operate through real Auth extraction. Removing write permission denies both
  native and JSON saves on both instances, preserving item/history output;
  handler-level JSON denial keeps its existing safe `forbidden` envelope.
- Removing read permission closes the existing SSE stream on its next check
  and denies the next resource reads. Disabling/removing an account denies both
  instances. Missing/malformed policy gives 503 for protected signed-in and
  explicit anonymous requests; health remains available.
- CSRF logout remains possible during policy unavailability and account disable.
  Acknowledged logout on B prevents subsequent protected access on A and B.
- B restarts while the unexpired Session survives. A finite 1300 ms
  `CLIENT PAUSE ALL` yields 503 on both instances without Set-Cookie or login
  navigation; reads recover after the pause. A separate 750 ms write pause makes
  login rotation uncertain: 503, no success cookie and no automatic replay.
- Scoped responses retain native CSP/private-response headers. These are HTTP
  assertions, not browser CSP enforcement or cross-store atomicity proof.

The selected server reported Valkey 9.1.2 and used the existing pinned digest
`valkey/valkey@sha256:418652cfb58ef879d4978c33553735d7147016032d5aefaa14c828e611eb9dfd`.
The service was fresh, loopback-published, task-owned and non-persistent; finite
pauses affected no shared service. The successful journey asserted SIGTERM exit
0 and the drain marker for A, original B and restarted B. Parent observation
found no remaining Admin executable processes; Valkey exited 0, its owned
container was removed and the published port closed. No persistent volume was
created; the existing cached image remains.

## Preserved failure and correction history

1. First live attempt stopped before readiness because the temporary settings
   omitted the required default Production overlay. This was a harness failure,
   not Auth evidence. One minimal correction changed the temporary profile
   filename to `production.toml`; no runtime change was made for that failure.
2. The next attempt started both Admin instances and exposed a product routing
   regression: unsupported `DELETE` became a 303 Auth redirect instead of 405.
   The parent moved Auth onto native `MethodRouter::route_layer` for registered
   methods, leaving the 405 fallback outside Auth. The assertion was retained.
   Both packages' strict lint and default compilation passed afterward.
3. The subsequent complete focused journey passed as recorded above. The two
   failures remain alongside the final PASS; no previous passing workload was
   replayed. One harness correction and one product repair were used, without
   reasoning/model recovery or new validation infrastructure.

Executor role/model (`gpt-6-astra`) and effective high effort were verified by
native trace and the shared resolver. Its focused unit/lint/format stdout and
exit records were retained. The method-layer repair and actual live execution
were parent-owned. Evidence is retained outside Git in
`/tmp/yetlet-yt126-admin-auth-20261022-ubfgya19/`, including both initial failures,
their classifications, final live stdout/stderr, cleanup and dependency graphs.
No commit or push was performed.

## Independent review and closeout

Independent Reviewer `/root/yt126_admin_valkey_auth_review_20261022` returned
PASS with zero issues, required followups and optional notes. Native trace
verified the reviewer role, `gpt-6-astra` model and effective high effort; the
shared resolver accepted the observed effort. The reviewer inspected the patch,
adjacent contracts, raw successful/failed execution records, cleanup and graphs
without rerunning tests or starting services.

All 19 source SHA256 values matched through review completion: 16 tracked
modifications and three new files, HEAD unchanged and staged empty. The reviewer
independently compared the default Admin graph and four Core/server/browser
graphs against their retained original baselines. Only documentation status and
review bookkeeping changed after PASS; application sources, tests and lockfiles
remain exactly as reviewed.

`cargo xtask check docs` PASS: 75 fixed required documents, 126 backlog items,
links, Markdown structure, whitespace and dependency DAG. Focused Rust formatting,
tracked diff whitespace and direct new-file whitespace/EOF checks PASS. No
previously passing unchanged runtime workload was rerun. This is bounded local
identity/lifecycle/current-policy and CRUD-compatibility completion; the stronger
sensitive-write ordering, real-provider, production and multi-host limits above
remain open. YT-096/YT-020 and the original release-candidate scope are unchanged.
