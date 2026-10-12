# YT-066b1 private Redis semantics spike

This unpublished experiment tests one Redis primary. It is a separate Cargo workspace, not a framework crate, Session middleware or a dependency of the existing Memory provider. See [design](../../docs/YT-066B-REDIS-SESSION-DESIGN.md) and [execution evidence](../../docs/YT-066B1-VERIFICATION.md).

`redis = 1.7.1` uses only `tokio-comp`, `connection-manager` and `script` (default features disabled). State is one typed counter/note encoded as JSON in an opaque Redis Hash field. Lua handles version/revision, primary-clock absolute expiry, CAS, atomic rotation/revocation and independent operation results. The Rust wrapper demonstrates once-only callbacks and explicit unknown outcomes. `command` and raw keys are fixture fault-injection seams, not proposed provider APIs.

The eleven ignored integration tests require an explicitly selected disposable fixture; the focused browser case also requires local Chrome and Python 3. One additional cookie-policy test runs without Redis:

```sh
task_project="yt066b1-$(date +%s)-$$"
docker compose -f qualification/yt066b/compose.yaml -p "$task_project" up -d --wait --wait-timeout 20
redis_listener="$(docker compose -f qualification/yt066b/compose.yaml -p "$task_project" port redis 6379)"
YT_REDIS_URL="redis://$redis_listener/" CARGO_TARGET_DIR="$PWD/target/yt066b1-build" \
  cargo +1.99.0 test --manifest-path qualification/yt066b/Cargo.toml --locked -- --include-ignored --test-threads=1
docker compose -f qualification/yt066b/compose.yaml -p "$task_project" down --volumes --remove-orphans --timeout 3
```

Run the `down` command even after test failure or interruption. Verify that the exact project's container, network and volume label queries return no resources before calling the run complete. Do not use global prune, run against a shared Redis, or reuse an uncertain live project. Tests deliberately use `CLIENT PAUSE`, `CLIENT REPLY OFF`, `CLIENT KILL` and corrupt synthetic records. `Store::connect` accepts only an unauthenticated `127.0.0.1` fixture URL. The fixture exposes one random loopback port, disables persistence, uses tmpfs, 128 MiB container memory, Redis `maxmemory 64mb`/`noeviction`, one CPU and 64 PIDs. No production TLS, ACL or durability claim follows.

Run compile/lint checks without starting Redis:

```sh
CARGO_TARGET_DIR="$PWD/target/yt066b1-build" cargo +1.99.0 test --manifest-path qualification/yt066b/Cargo.toml --locked --no-run
CARGO_TARGET_DIR="$PWD/target/yt066b1-build" cargo clippy --manifest-path qualification/yt066b/Cargo.toml --locked --all-targets -- -D warnings
cargo fmt --manifest-path qualification/yt066b/Cargo.toml --check
```

The ordinary unqualified `cargo test` command skips the real-Redis cases. A compile-only or ignored-only result is not execution evidence.

## Bounds and interpretation

- Fixture state payload: 1024 bytes; schema version 2 retains the prior experiment's monotonic issuance metadata (not part of cookie names or selection); schema 1 and unknown/corrupt records fail closed. Session lifetime is at most 60 seconds and ordinary CAS preserves its absolute expiry.
- Operation dispatch window: 1000 ms from Redis time. Results live for 5000 ms, independently of later mutations; tombstones live for 400 ms. Missing result evidence stays unresolved. At most 64 unexpired result keys are admitted per namespace. No exact global Session-capacity or persistence guarantee is claimed.
- Each tested manager allows one or two active operations, at most the same number of waiters, an 80 ms queue deadline and a 400 ms overall call deadline. Connect attempts have a 200 ms timeout, one reconnect retry and 10–20 ms bounded backoff.
- redis-rs response timeout is deliberately disabled inside owned workers: a caller timeout/cancellation must not release capacity while a reply remains outstanding. Workers retain both permits, and late results do not rerun user code. Explicit shutdown closes admission, drains for at most two seconds, aborts/joins remaining workers and drops the canonical driver's handles. A stuck manager can exhaust its bounded capacity until shutdown; automatic manager replacement is outside this spike.
- `try_update` returns the original generic `R` only after acknowledged commit. Application errors do not write. Unknown outcomes carry a redacted operation ticket and drop `R`; resolution returns status only. Cancellation may lose a ticket not yet returned. No callback replay or arbitrary `R` recovery occurs.
- Redis state and operation outcomes are written in one script, with pre-write validation. Script isolation is not rollback after a runtime/OOM error; such errors remain unknown. Restart, eviction, restore and partial-failure recovery are not qualified by this no-persistence fixture.

## Original single-name cookie counterexample

The HTTP test uses Axum on a random loopback socket and reqwest's real shared cookie jar. It is a candidate finalizer experiment, not production middleware. It verifies:

1. A final check that already sees rotation suppresses an old directive and preserves the new cookie.
2. Rotation after the final check but before header construction lets an old `Set-Cookie` overwrite the new cookie.
3. An older prepared response delivered after a new response also overwrites the new cookie.
4. A successful logout's delayed clear response can clear a later new login's cookie.

The latter three are **expected counterexamples** for the original single-name policy. Their tests passing does not establish the requested replacement-cookie guarantee. Redis rejects retired identities, but that policy can lose the client's usable Session. This test is retained as a negative control.

## Selected pure-SSR policy

The later unique-name candidate failed independent review: delayed lower cookies could evict the replacement or logout marker and expose another still-live Session. Its tests and actual Chrome storage failure remain [historical evidence](../../docs/YT-066B-COOKIE-ORDERING.md).

On 2026-10-11 the maintainer explicitly accepted no mandatory JS, preserved authorization safety, and possible login loss under extreme response reordering. The current `cookies` module uses one exact cookie name with Secure/HttpOnly/host-only/root-path/Lax defaults. Only a positively resolved create/rotate/replace may set it after a current Redis read. Reads, updates, errors and logout emit no Set/Clear; logout revokes in Redis. Missing, duplicate, malformed, expired, retired, corrupt or unavailable credentials fail closed. Loading never creates a Session, follows an old ID or chooses another cookie.

A prepared old Set may overwrite a replacement; the next request must be unauthorized, even if the replacement's Redis record remains alive. Explicit login can recover. The fixed name prevents per-issuance cookie accumulation; independent browser deletion/eviction is not prevented. Separate still-live Sessions are not globally ordered or revoked by this Session layer. The Memory API remains unchanged.

`cookie_fencing.rs` exercises emission eligibility, HTTP races before/after final checks, delayed logout, competing rotations, exact-name selection, no fallback and no automatic creation. Its browser case uses actual HTTP responses and Redis with Chrome script execution disabled and CSP `default-src 'none'`. It repeats one prepared retired cookie 180 times, checks a single stored cookie, observes unauthorized access, then explicit login and logout. It is not a broad browser/device qualification or the earlier 180-independent-cookie eviction experiment. The small Python driver reuses the existing repository DevTools helper and owns its temporary browser/profile.

Focused command (inside the disposable fixture):

```sh
YT_REDIS_URL="redis://$redis_listener/" CARGO_TARGET_DIR="$PWD/target/yt066b1-build" \
  cargo +1.99.0 test --manifest-path qualification/yt066b/Cargo.toml --locked --test cookie_fencing -- --include-ignored --test-threads=1 --nocapture
```

The selected contract passed independent read-only review (thread `01a128b4-dbc6-7922-87e7-ab67c171665a`, exit 0, observed `gpt-6-astra` / `high`, unchanged inputs). This opens the separately authorized YT-066b2 stage; YT-066 remains In progress. This remains a private semantics experiment, not public provider qualification.
