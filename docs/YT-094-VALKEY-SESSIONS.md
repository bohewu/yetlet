# YT-094 — opt-in Valkey shared sessions

Date: 2026-10-19. Status: implementation, native checks and actual two-app
Compose verification pass; independent Reviewer PASS, zero issues and required
followups. Source baseline:
`9a2d68dd98585a6766d09326e126e66491b0f31f` plus the working-tree slice.

## Selected contract

The Admin reference selects the existing session provider through Cargo feature
`valkey` and existing typed settings. Default builds and `session.backend=memory`
retain local Memory sessions. Selecting `valkey` without the compiled capability
fails startup, rather than silently falling back. No core, browser, CLI command,
registry, DI container or new datastore implementation is introduced.

Valkey is the selected shared-session server. The existing unpublished
`yetlet-session-redis` crate and its wire/key schema retain their names for
compatibility; the Board explicitly aliases the dependency as
`yetlet-session-valkey`. Its async manager, Lua/CAS, authoritative reads,
rotation/revocation, bounded errors and owned-work shutdown are reused. The
later owner-requested TTL correction below widens only the lifetime bounds.
The root workspace explicitly excludes that standalone provider, preserving its
separate workspace while allowing an optional application dependency.

The Board uses a small application-local extractor over the existing typed
Memory/remote handles. Remote state stores only an existing identity's name;
the application reconstructs current project/write permissions from its fixed
identity table. Unknown identities or injected permission objects fail decoding.
OIDC pending state can be serialized privately, including an absolute wall-clock
deadline that another process can interpret. A focused round-trip test covers
that state; no external IdP credentials or service were recreated.

Only the native login page may explicitly create missing anonymous remote state.
Protected requests never create a replacement or local session. Missing,
expired, rotated or revoked identities return 401; backend/protocol/codec and
unknown-operation failures return 503; CAS conflicts remain explicit. Login
rotates the session and CSRF token. Logout revokes authoritative state, so all
replicas reject the old cookie. Provider shutdown is part of Board draining.

```toml
[session]
backend = "valkey"

[session.valkey]
endpoint = "redis://127.0.0.1:6379/"
namespace = "isolated_admin_deployment"
ttl_seconds = 1800
```

The existing Loader supplies profile, explicit Development-secret overlay and
`APP_` override precedence. The endpoint is a protected settings string and is
never logged or sent to the browser. All replicas share the namespace, provider
settings and identity rules. Different deployments need distinct namespaces.
The remote cookie has the distinct local-HTTP name `yetlet_lab_valkey`, HttpOnly,
SameSite=Lax and Path=/; the existing Memory cookie is not reused.

The default is **1800 seconds (30 minutes)**. Settings accept 1 second to
30 days, matching the existing Memory provider's upper bound. The underlying
remote provider retains millisecond precision for short expiry tests and accepts
1 ms to 30 days. Four admitted calls and 4096-byte payload bounds are unchanged.
Reads/CAS updates do not refresh expiry; rotation starts a new lifetime and app
restart only preserves unexpired state. No sliding/idle timeout is introduced.

## Initial verification topology and scope (60-second fixture)

The [Compose fixture](../qualification/yt094-valkey/README.md) runs two real
non-root Admin containers and pinned Valkey **9.1.2**, image
`sha256:418652cfb58ef879d4978c33553735d7147016032d5aefaa14c828e611eb9dfd`.
The application image is
`sha256:6fbebdfa15657ec68f5186c4c6d51201019454488ba926078511edd57871acd5`;
its native executable SHA-256 is
`eb2a4a36c282492c0781c8e58c4833b11e4123f02cb04c715c254b8fe43db022`.
Sixteen selected source/lock/fixture hashes accompany the private execution
receipt and matched after that execution. These initial image/hash/results
precede the TTL correction; they are not evidence for the later source bytes.

This is a Linux single-host fixture. Apps use host networking and separate
loopback ports, preserving exact Host/Origin checks without adding public
listeners or proxy trust. The browser shares the host cookie across ports and
visits A/B explicitly; no sticky-session assumption is needed. Valkey publishes
only a loopback port. Its data is deliberately non-persistent. Each replica has
its own SQLite volume: business records are **not** shared by this proof.

Test login uses fresh terminal-only synthetic credentials from A. B accepts the
resulting authoritative session without receiving A's login code. Detailed
logs, source receipts and screenshots stay outside Git; no credentials are
included in repository evidence. Browser checks used system Chrome
155.0.8059.84 with JavaScript disabled at 1366 × 900; Browser plugin was absent.

| Check | Actual result |
| --- | --- |
| Existing Memory Board | 17/17 tests; strict Clippy passed |
| Board with OIDC and Valkey | 25/25 tests, including stored identity rejection and OIDC state round-trip; strict Clippy passed |
| Admin, capability on/off | 17/17 tests and strict Clippy passed in each build |
| Existing provider against Valkey | All four explicitly selected ignored integration tests passed; one service-free test was filtered, not counted |
| Cross-replica login | A login followed by B list returned 200; B did not reissue the shared cookie |
| Cross-replica logout | Incorrect CSRF returned 403; native B sign-out retired the old cookie on A and B, both 401 |
| App restart | While A was stopped, B returned 200. Restarted A returned 200 with the same unexpired browser cookie |
| Valkey pause | Protected reads on A and B returned 503 in 404 ms each; login page also returned 503. None set a cookie. Stateless health stayed 200 |
| Pause recovery | Unpausing the same Valkey dataset restored the original session to 200 |
| Valkey stop/restart | Both apps failed closed with 503. After restarting non-persistent Valkey, old cookies became 401 on both; fresh A login worked on B. Recovery observed an initial 503 before both returned 401 |
| Browser | Expected titles/content, no overlay, zero page errors and zero CSP errors; safe app screenshots retained privately |
| Lifecycle | Both apps emitted database-drain markers and exited 0 on Compose SIGTERM; Valkey exited 0 |
| Cleanup | Zero project containers/volumes, all three ports closed, browser closed/reset, owned app image tag removed; pre-existing containers preserved. The downloaded Valkey image remains only as a local cache |
| Independent review | `/root/yt094_oidc_code_review_oct19`: PASS, zero issues/required followups; implementation, provider seam, native evidence, screenshots, cleanup and all 16 source hashes inspected |

Existing `/healthz` is stateless liveness and `/readyz` checks SQLite. This slice
does not add a combined readiness contract for every optional external service.
No source was committed or pushed as part of this verification.

## Configurable lifetime follow-up

The owner requested a practical default and settings-controlled lifetime rather
than the original 60-second experimental ceiling. The Admin default is now
30 minutes, already used by its Memory demo; settings, provider configuration and
Lua all accept the longer duration. The 30-day upper bound follows the existing
Memory contract, not a universal application security policy. Zero, excessive
and overflowing durations fail configuration. Network/caller deadlines,
operation-result retention and tombstone retention are independent and unchanged.
The Compose default follows the app; a deliberate `YT094_SESSION_TTL_SECONDS=60`
can still select the original short test lifetime.

Focused validation passed against the same pinned disposable Valkey server:

- Provider tests: 7/7 including the owned-work unit test and all six integration
  tests (none ignored). The new test observes actual 30-minute and 30-day PTTL,
  matching cookie Max-Age, unchanged absolute expiry after reads/CAS updates,
  cross-manager rotation and global logout. The existing 120 ms expiry test and
  corruption/uncertain-outcome, concurrency and shutdown tests also pass.
- Admin tests: 17/17 with `valkey` and 17/17 without it. They cover the 1800-second
  default, settings-file/environment precedence, custom and boundary durations,
  and zero/excessive/overflowing rejection before listening.
- Strict all-target Clippy passes for the provider and Valkey-enabled Admin.
- Compose resolution selects 1800 by default on both replicas; explicit 60 and
  3600 overrides select the same requested lifetime on both. Only disposable
  Valkey was started for this follow-up; cleanup removed all owned containers,
  volumes and networks and closed its listener.

Commands use the existing Cargo tests, with the provider's `CARGO_TARGET_DIR`
and `YT_REDIS_URL` pointed at the disposable fixture:

```sh
cargo test --offline --locked --manifest-path crates/yetlet-session-redis/Cargo.toml -- --include-ignored --test-threads=1
cargo test --offline --locked --manifest-path examples/admin-dashboard/Cargo.toml --features valkey
cargo test --offline --locked --manifest-path examples/admin-dashboard/Cargo.toml
cargo clippy --offline --locked --manifest-path crates/yetlet-session-redis/Cargo.toml --all-targets -- -D warnings
cargo clippy --offline --locked --manifest-path examples/admin-dashboard/Cargo.toml --features valkey --all-targets -- -D warnings
```

Independent Reviewer `/root/yt094_session_ttl_review_oct19` returned PASS with
zero issues/required followups after inspecting all ten source/document hashes,
the actual expiry/cookie/configuration paths and the recorded checks/cleanup.
Documentation and whitespace checks also pass. The initial two-replica
browser/outage evidence above remains preserved at its original source and
lifetime; this correction does not claim a fresh full browser matrix or a
wall-clock 30-day soak. No commit or push was performed.

## Boundaries

This proves shared application sessions, not shared business storage, replicated
Valkey, Sentinel/Cluster failover, persistence/restore, TLS/ACL, multi-host or
production HA. A non-persistent Valkey restart intentionally requires a new
login. Restoring older session data and revocation durability remain unqualified.
The existing OIDC proof remains separate; valid real-provider callbacks across
Valkey replicas were not run. Owner-waived Windows OIDC, IME and assistive-tool
matrices are not reopened. YT-094 remains In progress for separately selected
application/deployment qualification; YT-104 remains independently unblockable.

The subsequent [shared-SQLite slice](YT-094-SHARED-SQLITE.md) selects one local
business-data volume through an optional Compose override. Its source/HTTP/native
evidence is separate from the original per-replica-storage results above.
