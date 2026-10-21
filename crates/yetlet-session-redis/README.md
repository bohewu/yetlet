# Experimental Valkey / Redis-protocol Session provider — YT-066b2

**Status: focused integration tests and independent implementation review passed at bounded YT-066b2 scope. Do not treat this crate as production-qualified.** See [provider implementation verification](../../docs/YT-066B2-VERIFICATION.md) and the later [three-application qualification](../../docs/YT-066B3-VERIFICATION.md). It is unpublished and has its own workspace/lockfile, outside the default framework build and dependency graphs. The existing synchronous Memory provider is unchanged.

This is a separate, explicitly selected async Session implementation, not an authentication system. It uses the reviewed single-primary Lua semantics and [selected pure-SSR Cookie contract](../../docs/YT-066B-COOKIE-ORDERING.md). No JavaScript, plugin registry, DI container, implicit Memory fallback or synchronous Redis facade is added.

The later [YT-094 Valkey reference](../../docs/YT-094-VALKEY-SESSIONS.md) selects
Valkey 9.1.2 for actual two-app shared-session verification. It reuses this
implementation and existing protocol/key schema. Its later TTL correction
accepts up to 30 days, matching the Memory provider's upper bound; the other
experimental bounds remain unchanged. The historical crate/API names remain compatible; the
reference's feature and backend setting use `valkey`. This evidence does not
upgrade the provider to a production or HA guarantee.

## API and application responsibilities

- Construct `RedisSessionConfig::new(namespace, cookie_scope, ttl, max_in_flight, payload_limit)` and explicitly connect `RedisSessionManager::<T>::connect(endpoint, config).await`. The endpoint is never included in provider errors/Debug. This private stage accepts `redis://`; TLS/ACL, off-host operation and HA are not qualified.
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

Lua script isolation is not rollback on runtime/OOM failure. Later [YT-066b3 qualification](../../docs/YT-066B3-VERIFICATION.md) passed actual three-application execution, owned cleanup and independent review at single-host/single-primary/no-persistence scope. YT-066 is Done at the selected single-host Docker/Redis-provider scope; YT-020 remains Blocked and this isolated provider remains outside the nine library candidates and nonpublishable. Session capacity beyond the selected bounds, persistence/restart/restore, production TLS/ACL/secrets, HA/Cluster and SLO/RPO/RTO remain unqualified. Never restore an older namespace and assume revoked credentials remain invalid.

## Local checks

Compile and lint without Redis:

```sh
CARGO_TARGET_DIR="$PWD/target/yt066b2-build" cargo +1.99.0 test --manifest-path crates/yetlet-session-redis/Cargo.toml --locked --no-run
CARGO_TARGET_DIR="$PWD/target/yt066b2-build" cargo clippy --manifest-path crates/yetlet-session-redis/Cargo.toml --locked --all-targets -- -D warnings
```

Remote-service tests are ignored unless explicitly selected; the configuration test needs no service. Use only a disposable [Redis fixture](../../qualification/yt066b/compose.yaml) or the [Valkey fixture](../../qualification/yt094-valkey/README.md), random loopback listener and exact-project cleanup. They deliberately pause writes and corrupt synthetic records. Do not run against a shared store. The CAS test uses `block_in_place` for its synchronous callback barrier. Historical provider tests passed after that test-only correction; the separate cancellation/drain test retains its earlier PASS. The verification record preserves both prior stalled attempts and the explicitly authorized correction. The added TTL integration checks a 30-minute store TTL/cookie, non-sliding reads/updates and cross-manager rotation/logout.
