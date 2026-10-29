# Experimental Valkey / Redis-protocol Session provider — YT-066b2

**Status: focused integration tests and independent implementation review passed at bounded YT-066b2 scope. Do not treat this crate as production-qualified.** See [provider implementation verification](../../docs/YT-066B2-VERIFICATION.md) and the later [three-application qualification](../../docs/YT-066B3-VERIFICATION.md). It is unpublished and has its own workspace/lockfile, outside the default framework build and dependency graphs. The existing synchronous Memory provider is unchanged.

This is a separate, explicitly selected async Session implementation, not an authentication system. It uses the reviewed single-primary Lua semantics and [selected pure-SSR Cookie contract](../../docs/YT-066B-COOKIE-ORDERING.md). No JavaScript, plugin registry, DI container, implicit Memory fallback or synchronous Redis facade is added.

This package is included in the selected sixteen-package experimental 0.9.0-alpha.2
candidate. It remains unpublished; candidate selection and publication-prepared
metadata do not establish registry availability or expand the support scope.

The later [YT-094 Valkey reference](../../docs/YT-094-VALKEY-SESSIONS.md) selects
Valkey 9.1.2 for actual two-app shared-session verification. It reuses this
implementation and existing protocol/key schema. Its later TTL correction
accepts up to 30 days, matching the Memory provider's upper bound; the other
experimental bounds remain unchanged. The historical crate/API names remain compatible; the
reference's feature and backend setting use `valkey`. This evidence does not
upgrade the provider to a production or HA guarantee.

## API and application responsibilities

- Construct `RedisSessionConfig::new(namespace, cookie_scope, ttl, max_in_flight, payload_limit)` and explicitly connect `RedisSessionManager::<T>::connect(endpoint, config).await`. The endpoint is never included in provider errors/Debug. Default mode accepts `redis://`. The optional `tls` feature additionally admits checked `rediss://` using native Rustls/Ring and native trust roots; insecure URL fragments reject. Deployment/ACL/recovery qualification remains separately scoped.
- `manager.scoped(app)` applies native Axum middleware only to existing routes. `Extension<RedisSession<T>>` or the typed request extension exposes the handle. Routes outside the scope are independent.
- `open` only parses the configured cookie. `read().await` always loads authoritative Redis state; missing/invalid state never silently creates a Session. An application must explicitly call `create(value).await` after its own policy checks. An active identity requires `replace_and_rotate` instead of another create.
- `try_update(|state| Result<R, E>).await` invokes the closure once, commits by CAS and returns its original `R` only after acknowledged success. Application rejection writes nothing. Conflict is explicit; callbacks are never automatically replayed. A synchronous callback must not block the async executor; external side effects are not rolled back.
- `rotate`, `replace_and_rotate` and `destroy` are async, atomic identity operations. `csrf_token` and constant-time `verify_csrf` authoritatively load the record; applications must enforce CSRF and authorization for their own unsafe actions. Session creation alone does not authenticate anyone.
- `OutcomeUnknown` carries a redacted `RedisOperationId`. `resolve_operation` returns historical status only; it neither retries the write, reconstructs arbitrary `R`, nor grants access to a retired identity. Cancellation may lose a ticket before the caller receives it.
- Call `shutdown().await` during application shutdown. It stops admission across clones and drains/reaps owned work. Dropping a request future is not proof that a write was cancelled.

`T` explicitly requires serde serialization/deserialization plus Clone/Send/Sync; the existing Memory provider gains no serialization bound. JSON is an opaque state field under schema 2, not a stable wire format. Unsupported/corrupt data fails closed; there is no migration/reset fallback. The internal issuance metadata is retained from the prototype but does not participate in cookie names or identity selection.

## Cookie behavior

The default cookie has one `__Host-` name, Secure, HttpOnly, SameSite=Lax, Path=/ and no Domain. Only the explicit local HTTP override uses a distinct insecure name. Malformed/duplicate configured-name cookies fail closed and legacy issuance names are ignored.

Only successful create/rotate/replace can leave a pending Set. A successful subsequent update in the same request preserves that creation/rotation directive; an ordinary loaded-session read/update does not mint or refresh a cookie. Error responses and destroy emit no Set/Clear. Finalization resolves the operation and reloads/decodes its current identity before adding the cookie. Middleware finalizer failure returns a redacted 503; malformed Cookie headers return 400. Application handlers choose their own Session-error HTTP mapping.

A late prepared Set may overwrite a newer cookie and require login again. The authoritative store must reject retired/revoked/expired credentials. No old-ID forwarding, alternate-cookie selection or global latest-login/account policy is provided. Logout revokes server state without a clearing response; a dead cookie may remain until overwrite/expiry. The prior Cookie prototype passed its independent review, but that PASS does not certify this new integration.

## Deliberate bounds and unqualified scope

Configuration accepts a 1–64 character isolated namespace, a 1–20 character cookie scope, 1 ms–30 days absolute Session lifetime, 1–4 in-flight calls plus the same number of waiters, and 1–4096 bytes encoded payload. Sub-millisecond lifetime precision is truncated. The caller chooses the lifetime; the Admin reference defaults to 30 minutes through settings. Reads and CAS updates do not renew expiry, while rotation starts a new configured lifetime. Reuse the same namespace/config for workers sharing one application; choose a new namespace when intentionally invalidating all prior state.

The fixed experimental bounds are: 400 ms shared caller deadline across request-lock/queue/Redis phases, 80 ms operation queue wait, 200 ms connect timeout, one reconnect retry with 10–20 ms backoff, immutable one-second server dispatch window, five-second operation-result retention, 64 result records per namespace, 400 ms tombstones and two-second shutdown drain. A synchronous callback cannot be interrupted; if it exhausts the caller deadline, the next phase refuses dispatch. Caller timeout does not cancel a dispatched Redis write; owned work retains its permits until a reply/disconnect or shutdown.

Lua script isolation is not rollback on runtime/OOM failure. Later [YT-066b3 qualification](../../docs/YT-066B3-VERIFICATION.md) passed actual three-application execution, owned cleanup and independent review at single-host/single-primary/no-persistence scope. YT-066 is Done at that recorded scope; this isolated provider remains outside the historical nine-package immutable candidate. The later TLS/ACL and cross-host/persistent-restart evidence below has its own bounded inputs; neither record qualifies arbitrary restore, HA/Cluster or application SLO/RPO/RTO. YT-020 remains Blocked and no registry upload is authorized. Never restore an older namespace and assume revoked credentials remain invalid.

## Local checks

Compile and lint without Redis:

```sh
CARGO_TARGET_DIR="$PWD/target/yt066b2-build" cargo +1.99.0 test --manifest-path crates/yetlet-session-redis/Cargo.toml --locked --no-run
CARGO_TARGET_DIR="$PWD/target/yt066b2-build" cargo clippy --manifest-path crates/yetlet-session-redis/Cargo.toml --locked --all-targets -- -D warnings
```

Remote-service tests are ignored unless explicitly selected; the configuration test needs no service. Use only a disposable [Redis fixture](../../qualification/yt066b/compose.yaml) or the [Valkey fixture](../../qualification/yt094-valkey/README.md), random loopback listener and exact-project cleanup. They deliberately pause writes and corrupt synthetic records. Do not run against a shared store. The CAS test uses `block_in_place` for its synchronous callback barrier. Historical provider tests passed after that test-only correction; the separate cancellation/drain test retains its earlier PASS. The verification record preserves both prior stalled attempts and the explicitly authorized correction. The added TTL integration checks a 30-minute store TTL/cookie, non-sliding reads/updates and cross-manager rotation/logout.


## Separate admission-burst research

[YT-172](../../docs/YT-172-SESSION-ADMISSION-BURSTS.md) adds one ignored,
`cfg(test)`-only provider-read study. Normal builds contain no probe, timer,
capture or public API. It retains the existing four active/eight total slots
and all deadlines/security semantics; it is not an HTTP capacity qualification.
The explicit native test filter is
`store::admission_probe::session_admission_burst_research`, with
`--lib --release --offline --locked` and `-- --ignored --exact --nocapture
--test-threads=1`. Set `YT172_SESSION_URL` only to a dedicated disposable
loopback Valkey and `YT172_OUTPUT` to an excluded private JSON path. Follow the
contract's fixed cases, no-replay and exact-owned cleanup rules. Contract and
corrected mapping pass; the one new probe validates 552 reads and retains 96
barrier-wave busy rejections, with eight successful serial recovery reads. Actual
resource absence is verified; initial startup/mapping and already-unloaded-unit
cleanup failures remain in the report. Final independent Reviewer PASS has zero
issues/required followups; YT-172 is Done at bounded provider-research scope.


## Separate full-HTTP timeline research

[YT-173](../../docs/YT-173-HTTP-SESSION-TIMELINE.md) uses the explicit private
compiler cfg `yetlet_session_admission_probe` to link fixture HTTP scopes and
provider/worker timing on one process-local clock. The hidden linkage module
is an unsupported research-build interface; ordinary builds have no export,
probe state, timer or capture. No Cargo feature/dependency, gate size, deadline,
worker ownership or security semantics changes. Never use its synthetic request
tag for authentication or policy. Follow the fixed study/cleanup contract and
retain timing-boundary/observer limits. Contract and focused checks pass;
corrected mapping and the one new HTTP study pass: twelve busy requests have
eight proven slot owners (four active/four pending), with complete correlated
records and actual cleanup. Original failures/uncertainty remain; final independent
Reviewer PASS has zero issues/required followups, Done at bounded research scope.


## Optional native TLS — YT-184 reference

Select `features = ["tls"]` explicitly and supply `rediss://` through a trusted
private input. The existing connect API, Session/Lua/admission semantics and
errors stay unchanged. No TLS backend is installed for the default graph.
The feature selects the mature native Rustls/Ring backend, standard certificate/
hostname verification and TLS1.2/1.3, with native trust roots; it rejects insecure
URL fragments rather than accepting a verification bypass. Applications own
trust-root selection, server authentication/ACL, secret input and lifecycle. The
application identity needs direct `TIME` permission: `Store::prepare` reads the
server clock before a mutation, separately from Lua execution. A namespace-
scoped deployment ACL using connection/scripting/read/write categories must
explicitly add `+time`; those categories alone do not grant it. The YT-184
fixture initially omitted this permission. The opt-in regression
`yt184_named_acl_time_grant_unblocks_session_lifecycle` now reproduces the refusal
with a password-authenticated app/default user disabled, then verifies the
unchanged provider lifecycle after only `+time`. That local router/TCP test is
separate from pending Azure Auth/restart qualification.
Endpoints/passwords must not be printed or serialized. No custom crypto, parser,
public factory or automatic upgrade of a plaintext endpoint is added.

The [YT-184 isolated cross-host contract](../../docs/YT-184-AZURE-AUTH-SESSION-RESTART.md)
selects task-CA roots only in fresh owned guests, private authenticated Valkey
and SQL, Secure cookies and explicit new-epoch re-login after authority restart.
Its first cloud bootstrap fails at a Path expression; authorized generation2
reaches both apps READY and rejects wrong SQL/Valkey TLS names, then fails the
SQL observer before the Auth/restart journey. Both failed generations are fully
deleted; the observer is corrected locally. These are historical failed attempts; the current composed reference is accepted
at bounded YT-184 scope after final Reviewer PASS, with YT-184 Done. Earlier single-host/
no-persistence qualification remains historical; this optional transport does
not establish general production, HA, arbitrary restore or platform coverage.


Native connection recovery may first surface a dropped or failed-reconnect error
and initiate background reconnection. A fixed sleep or server-process READY
alone is not per-client availability. YT-184's local persistent restart case
records this behavior using read-only observations; it adds no provider automatic
mutation replay or stronger recovery/SLO guarantee. Cross-host Auth/policy/logout
passes in generation4; generation5 completes authority recovery/VM restart,
accepted after final independent Reviewer PASS at bounded reference scope.


YT-184 is Done at bounded reference scope after final independent Reviewer PASS:
generation4 Auth and generation5 remaining authority recovery/persistent VM
restart with native exit0. Full SQL/restrictive policy, unexpired old-epoch
refusal and fresh-login/durable-output/logout checks pass; transient503s and
all earlier failures remain retained. All five groups/credentials are deleted.
This isolated Linux reference does not qualify HA, arbitrary restore or other
platforms; see the contract for exact composition and remaining acceptance.
