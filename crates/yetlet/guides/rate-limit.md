# yetlet::rate_limit

Apply native endpoint/group rate limits using explicit per-process memory
budgets. Configuration reuse and shared accounting are separate choices.

**Experimental, unpublished.** Use this source checkout and the documented target scope.


## Prerequisites

Use Rust 1.99+ and this checkout. Select the limiter in a native Tower/Axum
application; choose a trusted key and intended route scope. The Rust wiring is
a setup fragment inside a fallible function, not a complete server. The existing
`native` example additionally selects the optional Settings loader as an example
dependency; its external settings are documented below.

## Minimal integration

For a complete listener-free configuration check, from the repository root:

```sh
cargo run --locked -p yetlet --features rate-limit,config --example rate-limit-native -- --check
```

The following existing fragment instead installs a limiter directly; compiling
or loading settings does not select that installation automatically.

### Shared budget wiring

```rust
use axum::{Router, http::request::Parts, routing::get};
use std::time::Duration;
use yetlet::rate_limit::{Budget, InvalidRateLimitInput, Key, Policy as RateLimitPolicy, RateLimitLayer};

let refill_interval = Duration::from_secs(2);
let policy = RateLimitPolicy::new(refill_interval, 5, 64)?;
let shared = Budget::new(policy);
let layer = RateLimitLayer::new(shared, |_: &Parts| {
    Ok(Key::new("group-total").expect("bounded constant"))
});
let app: Router = Router::new()
    .route("/first", get(|| async { "first" }).route_layer(layer.clone()))
    .route("/second", get(|| async { "second" }).route_layer(layer));
# Ok::<(), InvalidRateLimitInput>(())
```

## Expected result

The wiring installs one shared Budget/key on `/first` and `/second`: the initial
burst is five and one cell refills every two seconds. Deficits return 429;
capacity/internal failures return 503 as detailed below. Separately, the runnable
native example defaults off. Its `--check` prints `Configuration valid; listener not started.`
and exits without binding; ordinary native mode listens on port 3000.

## Configuration and errors

### Admission, ownership and errors

`Policy` is immutable reusable configuration. Each `Budget::new(policy)` starts
independent accounting. Cloning a Budget, layer or service shares accounting.
The same Budget and same key across routes/methods create a group total; adding
a route/method to the key intentionally partitions it. Attach native
`MethodRouter::route_layer` after registering methods to preserve 404/405;
native GET includes HEAD. Later-added methods are not automatically covered.
Intentional stacked layers each charge once, without cross-layer refunds.

Governor 0.10.4 implements GCRA: the example policy initially admits five
requests, then refills one cell every two seconds. It is not a rolling-window
request count. Keys contain 1–512 bytes; retained key capacity is 1–4096.
Refill interval/burst must be nonzero and their checked product at most one day.
Invalid Policy or Key construction returns the value-free unit error
`InvalidRateLimitInput`. Key Debug output is redacted.

A synchronous extractor reads immutable request Parts and returns a Key or a
native Response. It never receives the request body. Choose an explicit fixed
key, actual transport peer metadata, or a verified typed account extension
installed by earlier authentication. Account encodings must distinguish the
relevant issuer, tenant and account, independently of session rotation. Missing
required metadata must reject. Arbitrary Forwarded/X-Forwarded-For headers are
not trusted or parsed; there is no fallback identity or built-in Auth lookup.
An account limiter after authentication does not protect authentication cost.

A private mutex covers lookup, cleanup, capacity, admission and post-success
clock sampling. Entries are reclaimed only after a full refill duration since
the last successful check's subsequent monotonic timestamp. Rejection never
refreshes retention. Existing keys remain serviceable at capacity; new keys
return 503 when no fully replenished record can be reclaimed. Cleanup is bounded
and opportunistic, with no background task. Dropping all handles releases state.

Validated time snapshots protect Governor's unchecked nanosecond arithmetic.
A Budget fails closed after 100 × 365 days of age, on backward/unavailable clock
samples, or on poisoned state. A post-admission clock failure also fails closed;
it does not refund. The horizon is deliberately far below the dependency's
approximately 584-year representation limit and leaves room for refill debt.

Deficits return generic 429 with `Cache-Control: no-store` and a finite
`Retry-After` rounded up to seconds, at least one. The hint is an opportunity,
not reserved credit. Capacity/internal failures return generic no-store 503
without Retry-After. Extractor rejections retain their status, body and other
headers and receive no-store. Native downstream responses/errors, readiness and
backpressure are preserved. There is no body buffering, waiting for refill or
refund after errors/cancellation. SSE is charged at connection admission.

### Opt-in Settings loader in the native example

The example explicitly selects the existing [yetlet-config loader](../../../docs/CONFIGURATION.md)
as a **development/example dependency**. Configuration is not a Core service or
limiter runtime dependency. The application calls `Loader::load` once at startup,
deserializes its own `Settings` type, and composes the native routes. No framework
provider registry, per-request configuration lookup or ambient library setting
is introduced. The root workspace excludes the nested config/secrets workspaces
so they retain their isolation. In the shared root lockfile, config's existing
exact libc pin resolves libc from 0.2.190 to 0.2.186; it adds no normal Settings
dependency to the limiter or server. See the YT-117 follow-up evidence for the
explicit version delta and focused compile check.

The [application-owned base TOML](../examples/rate_limit/settings/base.toml) declares:

```toml
enabled = false

[policy]
interval_ms = 2000
burst = 5
max_keys = 64
```

The loader reads this explicit manifest-relative settings directory, overlays the
selected finite profile and applies environment overrides. The prefix is
`RATE_LIMIT`; absent `RATE_LIMIT_ENVIRONMENT` selects Production. Development,
Staging and Production all keep rate limiting disabled unless explicitly enabled.
The example names its startup constructor `build_rate_limit_policy()` and aliases
the public `yetlet::rate_limit::Policy` as `RateLimitPolicy` at the application
boundary. This makes construction and capability explicit without renaming the
public crate API or changing the settings schema.

The Rust settings field is `refill_interval_ms`, with
`#[serde(rename = "interval_ms")]` preserving `policy.interval_ms` and
`RATE_LIMIT_POLICY__INTERVAL_MS` as the external names. They specify
the refill interval for **one cell**, not a counting window or the full burst
refill duration. There is no `policy.refill_interval_ms` wire key or environment
alias. Explicit environment snapshots use `Loader::load_with_env`.

The application settings structs reject unknown fields. Missing files, unknown
keys, malformed TOML and invalid scalar types fail startup according to the
existing loader contract, including when rate limiting is disabled.

| TOML field | Environment override | Base default |
|---|---|---|
| `enabled` | `RATE_LIMIT_ENABLED` | `false` |
| `policy.interval_ms` | `RATE_LIMIT_POLICY__INTERVAL_MS` | `2000` |
| `policy.burst` | `RATE_LIMIT_POLICY__BURST` | `5` |
| `policy.max_keys` | `RATE_LIMIT_POLICY__MAX_KEYS` | `64` |

The earlier flat example keys `RATE_LIMIT_INTERVAL_MS`, `RATE_LIMIT_BURST` and
`RATE_LIMIT_MAX_KEYS` are replaced by the nested policy keys above. The existing
loader rejects these old, now-unknown prefixed keys rather than silently ignoring
them. Boolean overrides accept exactly `true` or `false`.

Enabled mode validates positive refill interval/burst, retained capacity 1–4096
and checked refill interval × burst at most one day before binding. Disabled mode still
loads and type-checks settings, then returns the original native routes without
constructing Policy/Budget/layer, invoking extraction, accounting or pruning.
Type-valid unused policy values do not activate or construct a limiter.

Run from the repository root:

```sh
cargo run -p yetlet --features rate-limit,config --example rate-limit-native -- --check
RATE_LIMIT_ENABLED=true RATE_LIMIT_POLICY__BURST=2 \
  cargo run -p yetlet --features rate-limit,config --example rate-limit-native -- --check
RATE_LIMIT_ENABLED=true cargo run -p yetlet --features rate-limit,config --example rate-limit-native
```

`--check` validates startup settings and exits before opening a listener. Without
it, the example listens on `127.0.0.1:3000`; Ctrl-C shuts it down. `/first`
(GET/HEAD/POST) and `/second` (GET/HEAD) share the loaded policy and the same fixed
Budget/key; `/public` is unmetered. A Policy reused with a separately constructed
Budget would instead retain independent accounting.

The limiter library never reads files or environment variables. Changing settings
requires restart. No hot reload, throughput measurement or benchmark is included;
a later benchmark can select `enabled = false` for native unmetered routing.

Budgets are per process and reset on restart or deliberate reconstruction.
There is no shared/Valkey counter, cluster quota, WAF/DDoS guarantee or production
capacity claim. Body, concurrency and connection limits remain separate
application choices. The example quotas are illustrations, not deployment
recommendations.

## API reference

Use [limiter source/rustdoc](../src/rate_limit/mod.rs), the [native example](../examples/rate_limit/native.rs)
and the [current API reference](../../../docs/API-REFERENCE.md). The `# Ok` line is
rustdoc context; place wiring in your own fallible setup function.

<details>
<summary>Historical delivery context and evidence</summary>

An experimental, unpublished, native-only Tower/Axum admission layer for
[YT-117](../../../docs/YT-117-RATE-LIMIT.md). Explicit installation is required;
Core, the server facade, Auth and browser applications do not enable it.

</details>
