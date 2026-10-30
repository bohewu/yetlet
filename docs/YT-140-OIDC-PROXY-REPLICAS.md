# YT-140 — OIDC origin and cross-instance callback reference

Date: 2026-10-23. Status: **Done at bounded optional local integration scope
after independent contract and final Reviewer PASS, zero issues and required
followups**.
Source baseline: `7b23e5ff11cd5ab6669eac13bb80b0dd524e04b4`.
The owner activated contract-first guidance and the smallest concrete integration.
Initial independent Reviewer `/root/yt140_oidc_contract_review_20261023` returned
PASS, zero issues and required followups; all six contract-review hashes matched
before implementation. `contract-review-input.json`, `contract-review.patch` and
`contract-review-final.json` retain the original snapshot and result in the
evidence directory below. Implementation, focused proof and cleanup are complete
under that accepted contract. Final independent Reviewer
`/root/yt140_oidc_origin_final_review_20261023` returned PASS, zero issues and
required followups; all twelve final-review hashes matched before acceptance. New changes have no
commit or push authorization.

## Original authority and selected delta

The original YT-140 row in [Backlog](BACKLOG.md) and the
[deployment-profile contract](PRODUCTION-DEPLOYMENT-PROFILES.md#conditional-future-work-contracts)
select maintained-client issuer/origin/callback/trusted-hop and identity-freshness
guidance, reuse of source-bound IdP evidence, and focused proof of changed
integration gaps. The original row was Todo; the owner's subsequent activation
selects this bounded reference. The accepted
[YT-139 authority contract](YT-139-MULTI-INSTANCE-AUTHORITY-RECOVERY.md) supplies
the multi-instance prerequisite. [ADR 0004](adr/0004-request-context-and-middleware.md),
[ADR 0005](adr/0005-opt-in-capability-modules.md) and
[ADR 0006](adr/0006-handler-first-mvc-optional.md) retain native Axum/Tower,
application-owned authorization and optional capabilities.

The concrete gap is that Admin derives its OIDC callback from its listener and
Board separately derives its security origin from that listener. Two backends
cannot currently use one explicitly selected proxy origin consistently. Add one
optional Admin setting and let Board use its already-configured OIDC callback's
origin. Keep the existing loopback HTTP restriction. No new public HTTPS/proxy
deployment API, provider adapter, protocol, cryptography, dependency, crate,
registry, Controller, DI container or mandatory Core feature is selected.

The new live proof uses **two independent Board listeners and Session managers
inside one Rust test process**, behind one task-only loopback proxy. Initiation
on A and callback rejection on B prove shared pending-transaction ownership.
It does **not** perform a successful token exchange, new real-provider verification
or cross-process Admin OIDC login. These limits are part of acceptance.

## Bound source findings and reused evidence

Source findings bind to `git show <baseline>:<path>`, including the original
backlog/profile text, rather than mutable activation edits. Ordinary SHA256
receipts and findings are retained outside Git in
`/tmp/yetlet-yt140-oidc-proxy-20261023-wpji3irv/source-findings.json` and
`contract-summary.json`. These are notes, not a validation framework.

| Source at the bound baseline | Existing behavior and resulting minimum change |
| --- | --- |
| [Admin settings](../examples/admin-dashboard/src/config.rs), [startup credential composition](../examples/admin-dashboard/src/oidc.rs) and [main](../examples/admin-dashboard/src/main.rs) | Existing Loader/Settings and injected SecretProvider resolve one confidential-client credential at startup. Callback is `http://<listener>/board/oidc/callback`. Add an explicit optional origin in these settings and use it for callback construction; keep provider composition and main's existing constructor path |
| [Configuration Loader](../crates/yetlet/src/config/mod.rs), `merge` and `override_value` | Overlay/OS keys must already exist in base TOML. Declare the empty new key in the shipped Admin base file; a Serde default alone cannot make an undeclared OS override selectable |
| [Board constructors/security](../examples/lab-operations-board/src/lib.rs) | Actual listener must remain nonzero loopback. Host must occur once and equal listener authority. Unsafe methods reject duplicate or nonmatching Origin; safe callback GET currently ignores Origin. For OIDC only, derive canonical security origin from the configured callback and check a supplied callback Origin too |
| [Board OIDC](../examples/lab-operations-board/src/oidc.rs) | `openidconnect` 4.0.1 handles protocol/verification. Discovery and all provider endpoints require HTTPS; client backchannel is HTTPS-only, redirects disabled, connect/request budgets 5/15 seconds. Callback accepts only explicit-port `http://127.0.0.1/...` at the exact callback path. No relaxation is needed |
| [Session composition](../examples/lab-operations-board/src/sessions.rs), [DTO](../examples/lab-operations-board/src/domain.rs), [provider CAS](../crates/yetlet/src/session_redis/mod.rs) | Private pending state/nonce/PKCE verifier and absolute deadline already serialize in the shared record. `take_oidc` delegates to acknowledged CAS; the callback never retries consumption. Existing DTO and provider require no change |
| [Local policy](../examples/lab-operations-board/src/auth.rs) | Verified login stores a protected local account reference. Generic Auth uses the application-local issuer/scheme, not an invented external issuer/subject. Protected decisions freshly read the bounded shared policy and independently authorize resources |
| [Existing OIDC tests](../examples/lab-operations-board/src/oidc/tests.rs), [Admin integration helper](../examples/admin-dashboard/tests/auth_valkey.rs) | A private test-only maintained-client factory already avoids discovery. Existing ignored tests demonstrate explicit task-service selection, owned shutdown and private Cookie/CSRF handling. Extend the private Board test scope; do not add a public test factory or duplicate the historical Admin journeys |

The pinned local client source confirms `Client::redirect_uri()` returns the
configured redirect URL (`openidconnect-4.0.1/src/client.rs`, lines 498–505).
Its existing authorization builder and the pinned OAuth2 PKCE helper are enough
for the proposed test; no custom verifier, token signer or protocol peer is
needed. Relevant pinned source hashes are included in the findings receipt.

Reuse [YT-094 OIDC](YT-094-OIDC.md),
[YT-116](YT-116-OIDC-ADAPTER-STUDY.md),
[YT-094 Valkey](YT-094-VALKEY-SESSIONS.md),
[shared SQLite](YT-094-SHARED-SQLITE.md),
[YT-124](YT-124-MULTI-INSTANCE-AUTH-SESSION-CONTRACT.md) and
[YT-126](YT-126-ADMIN-VALKEY-AUTH-ADOPTION.md) only at their recorded scopes.
YT-094 retains its initial failed callback/WAF observation, later source-matched
Linux real-provider verified-token/explicit-access/local-logout PASS, separate
review results and owner-waived Windows OIDC check. Existing browser/Windows
records and their historical failures are not rerun or relabeled. The
[dependency-audit applicability decision](YT-094-OIDC.md#dependency-audit-applicability--2026-10-21)
remains unchanged: the bounded RSA verification-only exception does not permit
private-key use or new algorithms. No dependency or cryptographic consumer change
is selected here.

## Canonical origin, trust and cookie contract

1. Introduce optional `[oidc] application_origin` with an empty Serde default,
   preserving existing configuration files and direct-listener behavior. Declare
   `application_origin = ""` in Admin's shipped `base.toml` so the existing Loader
   permits `APP_OIDC__APPLICATION_ORIGIN` and private/profile overrides.
   A nonempty selection is accepted only when OIDC is enabled and must be exactly
   `http://127.0.0.1:<port>` with a canonical decimal nonzero explicit port other
   than 80. Port 80 is already incompatible with the client's explicit-port
   callback check after URL normalization. Reject whitespace, controls, userinfo,
   other hosts/schemes, paths (including a trailing slash), query, fragment and
   noncanonical authority spellings. While OIDC is disabled, leave these unused
   OIDC values unvalidated and unused as today: no discovery, secret resolution
   or origin change occurs, and demo security still follows its listener.
   Configuration errors use fixed redacted categories and never reflect input.
2. Admin constructs exactly `<application_origin>/board/oidc/callback`, or uses
   `http://<listener>/board/oidc/callback` when empty. Both instances must select
   the identical issuer, client ID, credential source, callback, subject mapping,
   Session namespace/DTO/cookie settings and local policy authority. No arbitrary
   request header supplies issuer, scheme, authority or callback. Discovery and
   callback validation remain before serving; bad configuration fails startup.
3. Board retains separate validation of the actual loopback listener. Its OIDC
   security origin comes from that client's configured redirect URL, using the
   maintained URL type's normalized origin. The callback path stays exactly
   `/board/oidc/callback`; route matching and existing 404/405 behavior remain
   native. Demo constructors keep deriving their origin from the listener.
4. A single static trusted local proxy preserves the original canonical Host
   and Origin when forwarding to A/B. Backend listeners and proxy bind only
   `127.0.0.1`. The proxy strips inbound `Forwarded` and `X-Forwarded-*`; Board
   ignores those headers even if sent directly. No trusted-header parser, hop
   count, dynamic discovery or forwarded peer-address authorization is selected.
   The host's processes are within the fixture trust boundary: loopback and Host
   checks are not peer authentication or isolation from a hostile local process.
5. Require one exact canonical Host on Board requests. Retain existing unsafe
   method Origin checks. On the OIDC callback, an absent Origin is permitted for
   normal top-level authorization redirects; a supplied Origin must occur once
   and match exactly. Reject `null`, duplicates and mismatches before consuming
   pending state. Forwarded host/proto values cannot rescue an invalid Host or
   Origin or override the configured redirect. Other safe GET behavior is unchanged.
6. Keep existing local cookies: opaque `yetlet_lab_valkey` (or Memory's distinct
   `yetlet_lab`), HttpOnly, SameSite=Lax, Path=/, no Domain, explicit insecure
   local-HTTP override. Cookies are host scoped and are shared across loopback
   ports; ports are not cookie isolation. No new cookie or CSRF format is needed.
   The provider's Secure/default `__Host-` behavior remains available independently;
   it does not make this application a public HTTPS reference. A future public
   TLS edge requires separately selected origin, secure-cookie, hop/peer/access
   controls and deployment evidence; this setting rejects that configuration.

The existing login-only CSP authorization origin, callback `no-referrer`, private
`no-store`/Vary behavior and secret isolation remain authoritative. No credential,
code, state, nonce, verifier, Cookie, CSRF or provider error description belongs
in reports, generic errors, hydration or normal logs.

## Shared transaction, identity freshness and recovery

Login requires native CSRF, generates fresh state/nonce/S256 PKCE, and rotates
Session/CSRF while installing a five-minute pending record. B must use the same
remote Session namespace and read the pending data created on A. A decoded
callback atomically takes that record before state/error/issuer/code checks or
token exchange. Wrong state, provider error, mismatched returned issuer, expiry
or missing code retires that transaction. A malformed Query or rejected
Host/Origin can fail before the handler takes it. Replay never retries exchange.
Concurrent CAS can return 409; an unavailable or unknown consume returns 503,
with no successful login or success cookie. A timeout is not proof of rollback.

After successful maintained-client verification, the exact configured issuer and
verified subject select an explicitly existing local identity. Email, display
name and provider roles do not grant permissions or provision accounts. Local
Session state stores the protected local binding, not a generic trusted Principal
or an external-identity assertion. Current policy checks can deny even when the
callback lifecycle successfully rotated a Session; policy-aware login admission
is not added by this slice.

| Authority/freshness boundary | Selected rule and limit |
| --- | --- |
| Discovery/JWKS/client credentials | Startup snapshots. No automatic refresh, retry refresh, introspection or hot reload. Discovery failure prevents startup; token verification failure denies login. Restart all participating consumers coherently to adopt changed snapshots; temporary old/new key overlap is an operator/provider decision, not a qualified rolling protocol |
| Subject map and issuer/client/callback | Identical trusted startup configuration on all replicas. A map change does not rewrite existing local Session references. For a security-significant remapping or compromised trust configuration, stop/drain all consumers, adopt coherent settings and a new never-reused Session namespace, then require fresh login; do not mix mappings or restore old namespaces on rollback |
| Local account disable/read/write policy | Existing shared local file, fresh bounded lookup on every selected protected decision; missing account/disable denies, unavailable/malformed authority is 503. No policy cache or IdP account-disable feed is added. Lifecycle login/logout remains separate |
| Upstream account/session | No immediate IdP-disable propagation, token revocation, refresh, upstream logout or cancellation guarantee. Local continuity ends at its configured absolute Session expiry, logout, policy denial or namespace cutover; ID Token expiry is not local Session expiry. The 1800-second default and allowed 1-second–30-day TTL range are existing local policy. A new login depends on the provider's then-current issuance rules; no universal disable-latency guarantee follows |
| SQL and write ordering | One same-host SQLite file and current policy authority, as in YT-139. Shared Valkey does not share per-replica business files. Retain [YT-138's pre-SQL race](YT-138-AUTHORIZATION-COMMIT-ORDERING.md): policy/Session checks do not atomically fence business commit. No stronger revocation-to-commit guarantee is selected |
| Unknown outcomes and recovery | Retain [YT-137](YT-137-WRITE-OUTCOME-IDEMPOTENCY.md): no blind mutation/callback replay. Reconcile unknown work and stop/drain all old writers before cutover. Restored pending or authenticated Session records can resurrect old authority; retire the old namespace under [YT-139](YT-139-MULTI-INSTANCE-AUTHORITY-RECOVERY.md), never reuse generations on rollback, retain current SQL/policy, and require re-login. No persistent epoch allocator or automatic fencing is added |

## Minimum implementation and acceptance after contract PASS

Only these implementation paths are selected:

| Path | Exact bounded change and acceptance |
| --- | --- |
| `examples/admin-dashboard/base.toml` | Declare `application_origin = ""` under existing `[oidc]`, enabling ordinary Loader overrides without selecting a proxy or changing the default |
| `examples/admin-dashboard/src/config.rs` | Optional origin field, exact local-origin validation/default selection and focused test `config::tests::oidc_application_origin_is_explicit_and_loopback_only`; absent/empty defaults remain direct, valid enabled proxy selection resolves consistently, invalid enabled selections reject, disabled OIDC keeps ignoring its unused values |
| `examples/admin-dashboard/src/oidc.rs` | Construct the callback from the validated origin through existing Settings/SecretProvider composition. No alternate secret or discovery route |
| `examples/lab-operations-board/src/oidc.rs` | Private origin accessor from the existing client's redirect URI; no new public constructor or transport/verifier seam |
| `examples/lab-operations-board/src/lib.rs` | Select configured OIDC origin independently of listener; enforce supplied callback Origin before Session consumption; retain demo/direct defaults |
| `examples/lab-operations-board/src/oidc/tests.rs` | Reuse private client factory and helpers. Add `configured_oidc_origin_is_distinct_from_listener` for direct compatibility and explicit canonical origin/Host/Origin boundaries. Add ignored Linux `auth-valkey` test `proxy_callback_consumes_shared_pending_across_board_instances` for the exact live sequence below |

The parent owns this contract's index links/status and Admin usage documentation.
No main, Session/DTO/provider/Core, manifest, lockfile, migration,
generator, browser or old integration-test source change is needed. If this
minimum proves infeasible, return the concrete blocker for contract reassessment
before expanding scope.

1. Require explicit `YT140_VALKEY_URL=redis://127.0.0.1:<port>/` for a task-only
   disposable Valkey. No default service selection. The parent owns the pinned
   `valkey/valkey@sha256:418652cfb58ef879d4978c33553735d7147016032d5aefaa14c828e611eb9dfd`
   container and its cleanup; no persistent volume. Bind one proxy and two backend
   listeners on distinct nonzero ephemeral loopback ports. Use one TempDir for
   one SQLite file, one atomically written policy file and a unique Session
   namespace. Construct two separate Board objects/managers with `AuthValkey`.
2. Reuse the private OIDC factory's synthetic HTTPS issuer/client metadata and
   empty signing-key set; set both clients to the same exact proxy callback.
   Do not perform discovery or token HTTP. The proxy uses native Axum and the
   existing reqwest client, with redirects disabled, bounded body/response sizes
   and request deadlines; fixed routing sends login/initiation to A and callback
   to B. A test-owned selector may choose A/B for the race; inbound headers never
   choose a backend. This is a tiny fixed fixture, not a general proxy product.
3. Through the proxy, fetch login, POST valid CSRF and observe rotation plus an
   authorization Location with the shared callback, S256, state and nonce. In
   memory only, read B's independent manager and confirm the same state/nonce,
   verifier-derived S256 challenge and unexpired deadline. No value is printed.
   Old anonymous continuity must fail. Assert local cookie scope/attributes.
4. Invalid canonical Host or supplied Origin fails 403 before consumption;
   spoofed Forwarded/X-Forwarded host/proto cannot change that result. Through
   the proxy, spoofed forwarding is stripped. Canonical Host with absent or exact
   Origin still follows the selected application origin. Then send correct state
   **without code** to B: 400 proves B found and consumed A's pending transaction
   before the missing-code check. Replay on A/B yields 401, no Set-Cookie, no
   authenticated protected access and no code/state in the error body. Require
   callback no-referrer/no-store/CSP headers. Never send an exchangeable code.
5. Start one fresh transaction and race a correct-state missing-code callback on
   A against B through the controlled proxy. Require exactly one 400 and the
   other 401 or CAS 409, never two 400s or a success response; later replay is
   401. This proves the wiring's single acknowledged consumer, not token or
   business exactly-once behavior. Check shared pending is absent afterward.
6. Check the shared business list remains empty using the existing database
   read API. This fixture submits no business writes. Stop/await proxy and both
   HTTP tasks, call both Boards' existing drain (including provider shutdown),
   remove TempDir data and confirm all three listeners closed. Own task handles
   with abort-on-drop fallback so failures cannot leave listeners/tasks running;
   record successful normal cleanup separately. The parent removes its Valkey
   container and confirms port/resource absence.

Executed focused commands (raw command/exit/output receipts retained below):

```sh
cargo test --offline --locked --manifest-path examples/admin-dashboard/Cargo.toml config::tests::oidc_application_origin_is_explicit_and_loopback_only -- --exact
cargo test --offline --locked -p yetlet-lab-operations-board --features oidc oidc::tests::configured_oidc_origin_is_distinct_from_listener -- --exact
cargo test --offline --locked -p yetlet-lab-operations-board --features oidc,auth-valkey oidc::tests::proxy_callback_consumes_shared_pending_across_board_instances -- --ignored --exact --nocapture
cargo check --offline --locked -p yetlet-lab-operations-board
cargo clippy --offline --locked -p yetlet-lab-operations-board --features oidc,auth-valkey --all-targets -- -D warnings
cargo clippy --offline --locked --manifest-path examples/admin-dashboard/Cargo.toml --features auth-valkey --all-targets -- -D warnings
cargo xtask check docs
git diff --check
```

Run focused `rustfmt --check --edition 2024` only on changed Rust files. Exact new
tests execute once after implementation; ignored-test compilation alone is not
PASS. Formatting, compilation/lint, actual HTTP/Session observations, documentation
and cleanup are separate evidence categories. No old IdP/browser/Windows,
benchmark, graph, provider-matrix or accepted workload rerun is selected. No
new verification framework, generator or validator is authorized.

## Current checkpoint and remaining decisions

Initial contract review PASS preceded implementation and service execution.
The six selected configuration/application/test paths are implemented, with no
main, Core, provider, Session/DTO, manifest, lockfile, migration, browser or old
integration-test changes. Focused verification and cleanup are complete below;
final independent Reviewer PASS completes the bounded YT-140 acceptance. No external IdP,
browser or production operation was selected.
Real customer origins, TLS custody, cross-host topology, actual IdP/accounts,
operators and SLOs are unselected consumer decisions, not repository reference
Done gates. No public deployment, HA or production authentication is qualified.
YT-141 remains Todo/inactive; YT-096 D1–D6 remain Pending, YT-020 remains Blocked
and immutable M20 evidence remains unchanged. The next registered item is YT-141 performance work, which requires separate
activation. No other backlog item is activated by this note.

## Actual focused results and evidence

All seven worker-owned test/compile/lint/format commands completed with exit 0. Source SHA256 values for the
six implementation paths, literal argv, elapsed times and stdout/stderr hashes
are retained in `implementation-summary.json` in the evidence directory above.
No failing runtime attempt, repair or retry was used. Historical test filters
were retained, not counted as new passing tests.

| Verification | Actual result and scope |
| --- | --- |
| New Admin settings exact test | PASS: 1 passed, 0 failed/ignored, 18 filtered. Empty/absent/direct, enabled canonical selection, invalid redacted configuration and disabled unused-value compatibility |
| New Board origin exact test | PASS: 1 passed, 0 failed/ignored, 25 filtered. Separate configured/listener origin, direct/demo compatibility, exact Host and absent/exact/mismatched/null/duplicate callback Origin, forwarding spoof refusal and listener bounds |
| New ignored live proxy test, explicitly executed | PASS: 1 passed, 0 failed/ignored, 31 filtered. Test 0.04 s; complete Cargo command 9.261 s. Two independent Board listeners/managers plus one fixed proxy in one test process, using task-only Valkey 9.1.2 |
| Shared pending transaction and callback rejection | A initiation used the common callback; B's independent manager observed matching state/nonce/verifier-derived S256 challenge and future deadline privately. B's correct-state missing-code callback returned 400, then replay on A/B returned 401. Invalid Host/Origin failed before consumption; forwarding spoof could not rescue it |
| Concurrent consume and authority | A/B race had one 400 and one 401 or CAS 409, pending absent afterward, later replay 401. No successful authentication/Set-Cookie or business effect; both protected scopes stayed unauthenticated and both project lists were empty on both Boards |
| HTTP response/privacy boundaries | Callback errors retained no-referrer, no-store and existing CSP; errors did not contain state. Protocol values, cookies, credentials and full callback/authorization URLs remained in memory, outside reports |
| Default Board compilation | `cargo check --offline --locked -p yetlet-lab-operations-board` PASS; default-feature compile compatibility, not a repeated default test matrix |
| Formatting and strict Clippy | Changed-source rustfmt PASS; Board `oidc,auth-valkey` and Admin `auth-valkey` all-target Clippy with `-D warnings` PASS. Compilation/lint does not mean every target executed |
| Owned cleanup | Three HTTP tasks stopped and joined, both Boards/provider managers drained, all three listeners closed and TempDir data removed. Parent independently observed closed listeners/root absence; its pinned Valkey container stopped exit 0, was removed and its loopback port closed. No persistent volume or remaining owned resource |

The live fixture reuses the existing private maintained-client test factory;
there was no discovery or token HTTP, fake provider, token signer, cryptographic
verifier or TLS bypass. Token finish/verification and HTTPS-only/no-redirect
backchannel code were unchanged. The previous real-provider success remains
source-bound historical evidence, not this new test's result. This new proof is
not a successful token exchange, cross-process Admin OIDC, browser/Windows,
public HTTPS, HA or production authentication qualification.

Raw `config-origin`, `board-origin`, `proxy-live`, `board-default`, `board-clippy`,
`admin-clippy` and `rustfmt` JSON/stdout/stderr records, plus `valkey-resource.json`
and independent parent `cleanup.json`, preserve commands, results and ownership.
The implementation source remains unchanged after the passing checks. Parent
documentation/whitespace checks PASS: 75 required documents, 141 backlog items,
internal links, Markdown structure, whitespace and dependency DAG. Independent
final review PASS, zero issues, required followups and optional notes; all twelve
reviewed hashes matched before and after review. `review-input.json`,
`review.patch` and `review-final.json` preserve the snapshot and result. Only
completion status/evidence bookkeeping changed afterward; all six tested and
reviewed implementation paths remain byte-identical.
