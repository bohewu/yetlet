# YT-117 — Opt-in endpoint and group rate limiting

**Date:** 2026-10-22. **Status:** Done at per-process memory and Settings-example scopes after separate independent Reviewer PASS.
**Source baseline:** `428fa43d118ee9f311cf123d51a0552102f9ef28`.
**Authority:** The owner selected a general-purpose optional middleware with
endpoint/group policies, explicit configuration reuse versus shared budgets,
and per-process memory state. Establish the contract first, then implement the
minimum slice; independent Reviewer PASS gates Done. No commit or push.

## Selected boundary

Add one small unpublished native `yetlet-rate-limit` crate, explicitly selected
and installed through ordinary Tower/Axum composition. No Auth, Session, Core,
configuration-file service or browser dependency is required by the limiter.
Existing applications do not install it automatically. No RequestContext field,
new routing engine, DI container, Controller, mandatory middleware or default
application migration is selected.

An explicit application startup setting controls installation and defaults to
disabled. The owner subsequently selected integration of the native example with
the existing opt-in `yetlet-config` loader. This is an example-only dependency:
application-owned `enabled` and `policy` values load once from TOML/profile/env,
then determine native middleware composition. The limiter runtime does not read
files or environment variables and has no configuration dependency. See the
[Settings follow-up](#settings-loader-follow-up--2026-10-22) for current wiring
and separate acceptance. Disabled routing constructs no Policy/Budget/layer and
performs no extraction, accounting or pruning; the generic loader still reads
and type-checks configuration. Enabled policy validation precedes listener bind.
Restart is required; live toggling/hot reload and benchmark infrastructure remain
outside this item.

The algorithm is the existing Governor 0.10.4 GCRA implementation, selected with
only the required standard-library feature. Rate and burst are distinct: a
one-cell refill interval of two seconds with burst five initially admits five
requests and then replenishes one cell per two seconds. This is a sustained
rate with a bounded burst, not a strict rolling-window request count.

Primary sources consulted for the contract:
[Governor algorithm](https://docs.rs/governor/0.10.4/governor/),
[quota semantics](https://docs.rs/governor/0.10.4/governor/struct.Quota.html),
[HTTP 429](https://www.rfc-editor.org/rfc/rfc6585.html#section-4), and
[Axum method-scoped layers](https://docs.rs/axum/0.8.9/axum/routing/method_routing/struct.MethodRouter.html#method.route_layer).
Published API/source inspection informed this selection; dependency and runtime
verification are separate implementation evidence.

## Policy reuse and budget identity

The initial public concepts are deliberately separate:

| Concept | Meaning |
|---|---|
| `Policy` | Immutable validated refill interval, burst and retained-key capacity; safe to copy/reuse as configuration |
| `Budget::new(policy)` | New independent in-memory accounting state, constructed at application startup |
| `Budget::clone()` | Shares the exact same accounting state through Arc, including across Layer/Service clones |
| `RateLimitLayer` | Binds one Budget and an explicit key extractor to selected native methods/routes |
| `Key` | Opaque, bounded, redacted client/accounting key |

Two endpoints can reuse one Policy while creating separate Budgets. Two
endpoints or methods share consumption only when they use the same cloned
Budget **and the same extracted key namespace**. Adding a method or route to
that key intentionally partitions the budget. A fixed key makes the selected
Budget a common endpoint/group total. Policy names and settings-file parsing
remain application-owned startup configuration; no implicit name-to-state
registry, policy inheritance, override engine or URI lookup is added.

Bind methods explicitly using `MethodRouter::route_layer`; combine native
method routers or nest/merge groups as appropriate. For a shared group total,
each selected method uses the same cloned Budget. A selected all-application
budget is the same explicit composition over those routes, not a pre-routing
catch-all that silently changes fallback behavior. Preserve native 404/405;
native GET normally includes HEAD. Policies apply to routes registered before
the layer is attached. There is no automatic hot reload or automatic migration
of Admin endpoints into these policies.

Each installed layer charges once. If an application intentionally stacks two
budgets, the request must pass both; a charge from an earlier admitted layer is
not refunded when a later layer rejects. This is sequential composition, not an
atomic multi-budget transaction or an implicit endpoint override.

## Trusted extraction and ordering

The extractor is synchronous and reads immutable HTTP request Parts. It returns
a bounded Key or an explicit native rejection response. It does not read,
buffer, parse or replay the body, and it runs outside the storage lock.

Applications explicitly choose one of these sources:

- A fixed endpoint/group key, intentionally sharing one total.
- Transport-provided TCP peer/IP metadata. Arbitrary `Forwarded` or
  `X-Forwarded-For` headers are not parsed or trusted by the limiter. A trusted
  proxy identity contract requires separate explicit application selection.
- An application-verified typed account/tenant extension, established by prior
  authentication. The application's key encoding must distinguish its issuer,
  account, scheme and tenant dimensions as needed; Session rotation must not
  accidentally reset an intended account budget.

There is no built-in header identity parser, Auth lookup or fallback key. If
required trusted metadata is absent, the application extractor rejects with a
generic no-store response; it must never bypass accounting or silently switch
to a different identity. A verified-account limiter runs after authentication;
it does not protect the cost of the preceding authentication lookup.

## Bounded accounting, clock and cleanup

The selected limits are explicit: keys contain 1–512 bytes, key capacity is
1–4096 per Budget, refill interval and burst are nonzero, and the checked full
refill duration (`interval × burst`) is at most one day. Reject invalid or
unrepresentable configuration before constructing the algorithm. Rejected key
construction must not copy an oversized input. Key Debug output is redacted;
responses and normal diagnostics contain no key or account value.

Use one private mutex-protected map of bounded keys and direct Governor state.
Lookup, eligible cleanup, capacity checking, insertion, admission and successful
timestamp recording share one atomic critical section. Do not let an entry be
evicted/replaced while an independent handle still admits against it. Release
the lock before invoking downstream service code.

Evict only after a record is indistinguishable from fully replenished state.
A conservative implementation records the same monotonic clock **after** a
successful Governor check, then requires at least the complete validated refill
duration before reclamation. Denied requests do not extend that retention time.
Use elapsed comparisons and checked arithmetic; do not wrap time values or
grant credit on clock/representation failure. Fail closed with 503 before a
Governor operation whose clock arithmetic cannot safely be represented.

Existing keys remain usable at capacity. New keys may reclaim only fully
replenished records; otherwise admission returns generic 503. Never evict an
indebted record merely to admit a different key, and never permit a concurrent
capacity check to overfill the map. The bound concerns retained limiter state;
application extraction allocations, in-flight requests and the number of
explicitly constructed Budgets remain application-owned.

Cleanup is opportunistic and bounded by configured capacity, without an
unowned background worker. State is released when the last Budget/layer/service
handle is dropped. A new process or deliberately new Budget starts fresh;
cloning a service must not do so.

## HTTP and cancellation semantics

| Outcome | Selected response/behavior |
|---|---|
| Available cell | Charge one and invoke the existing handler/service |
| Same-key deficit | Generic 429, `Cache-Control: no-store`, finite `Retry-After` delta-seconds rounded upward with a minimum of one |
| New-key capacity or internal/clock state unavailable | Generic 503, no-store, no invented Retry-After |
| Extraction failure | Preserve the application's explicit native rejection and make it no-store; no budget or downstream work |
| Invalid Policy/Key construction | Fallible startup/caller error; no silent disable or weaker fallback |

Retry-After is the earliest observed opportunity, not a reservation: another
request sharing the key can consume replenished credit first. There is no
refill sleep, limiter queue, automatic retry or refund after admission. Handler
errors, rejected bodies and cancellation retain their admitted charge. Ordinary
Tower readiness/backpressure and error propagation remain intact; call the same
service instance whose readiness was obtained. The limiter does not override
downstream readiness or buffer request/response bodies.

Native responses remain native responses; no general result hierarchy, public
error schema, credential value, rate-key echo or automatic cookie is added.
Application-specific HTML/JSON presentation and concurrency/body/connection
limits remain separate choices. An SSE request counts at connection admission,
not once per event or as an active-connection limit.

## Qualification and exclusions

Implement only this per-process memory contract. There is no Valkey/shared
counter, cluster quota, WAF/DDoS guarantee, live administration UI, hot reload,
weighted request-cost model or distributed policy service. Two processes each
have their own Budget even if they use identical settings. This is not a new
release candidate or production capacity claim; initial example quotas are
illustrations rather than measured deployment recommendations.

Focused new product checks must cover configuration/numeric bounds, deterministic
refill/burst and safe reclamation, concurrent key/capacity accounting, key privacy,
same-Policy independent budgets, shared clones across routes/methods, scope and
404/405 isolation, trusted extraction and spoofed-header rejection, no body
polling/downstream work on rejection, cancellation without refunds, correct
429/503 headers, and native Tower readiness. Use the existing Governor test
clock internally rather than new validation infrastructure.

Configuration checks must also prove default-off and explicit-off native
behavior beyond the enabled quota, explicit-on throttling, no extractor or
limiter work while disabled, and rejection of invalid enabled settings. No
benchmark throughput or disabled-mode performance measurement is claimed.

Add one small compiled native composition example and focused actual loopback
HTTP tests with explicit shutdown/owned cleanup. The new capability's package
tests, strict Clippy, format/rustdoc/docs checks and normal dependency comparisons
are sufficient; do not repeat accepted Auth/Session/IdP/browser/deployment suites.
Verify unchanged Core/server/browser and default Admin graphs. Keep server-only
limiter dependencies out of browser builds and preserve the original release
catalog's historical counts. Record actual results and independent Reviewer
PASS before changing YT-117 to Done.

## Initial implementation and local evidence — 2026-10-22

The selected crate is implemented in [the native package](../crates/yetlet/guides/rate-limit.md).
The public surface is `Policy::new`, `Budget::new`/Clone, `Key::new` and
`RateLimitLayer::new`, plus its native Tower service and value-free construction
error. Governor is pinned to `=0.10.4` with default features disabled and only
`std` enabled. No existing locked package version or checksum changed.

The private engine uses a validated monotonic snapshot clock shared by its direct
Governor states. Samples beyond 100 × 365 days of Budget age, backward or
unavailable samples and poisoned storage fail closed; no unchecked dependency
clock addition can approach its u64 nanosecond boundary. A successful admission
records a subsequent real monotonic sample under the same mutex. This preserves
safe full-refill reclamation even when a thread is suspended during admission.

The originally reviewed [native example](../crates/yetlet/examples/rate_limit/native.rs)
used `RATE_LIMIT_ENABLED` (absent/`false` means disabled),
`RATE_LIMIT_INTERVAL_MS` (2000), `RATE_LIMIT_BURST` (5) and
`RATE_LIMIT_MAX_KEYS` (64). Numeric settings are read and validated only when
enabled. Disabled routing returns before constructing any Policy/Budget/layer;
no quota parsing or extractor/accounting/pruning runs. The library never reads
ambient environment variables. Restart is required to change the setting. This
initial manual environment parser is superseded by the owner-selected Settings
follow-up below; the original tests and review remain historical evidence.

Focused results, without repeating accepted Auth/Valkey/browser workloads:

| Verification | Actual result |
|---|---|
| `cargo test -p yetlet-rate-limit --all-targets --offline --locked` | PASS: seven private-state tests, seven middleware tests including two actual loopback HTTP tests, three compiled-example startup-setting tests; none ignored |
| `cargo test -p yetlet-rate-limit --doc --offline --locked` | PASS: one doctest |
| `cargo clippy -p yetlet-rate-limit --all-targets --offline --locked -- -D warnings` | PASS |
| `RUSTDOCFLAGS='-D warnings' cargo doc -p yetlet-rate-limit --no-deps --offline --locked` | PASS |
| `cargo fmt -p yetlet-rate-limit -- --check` | PASS |
| Normal dependency graph comparisons | Core, server facade, two browser roots at wasm32 target and default Admin byte-identical to accepted HEAD baseline |
| New-file whitespace/fences/links/final newline | PASS, checked directly in addition to the repository docs checker |

State tests cover policy/key bounds and privacy, exact refill/burst and safe
reclamation, non-refreshing denial, same-policy independent versus cloned shared
budgets, concurrent same-key admission/capacity, post-success time and guarded
clock/storage failures. Middleware tests cover shared methods/routes/HEAD,
404/405 and public-scope isolation, trusted peer/typed-extension extraction,
spoofed forwarding headers, preserved explicit rejection, no body polling or
downstream work on denial, native readiness/errors, cancellation/stacked charges,
and 429/503 cache/retry headers. Example tests prove both off paths beyond the
enabled quota, enabled shared throttling and invalid enabled settings.

Both actual HTTP fixtures own ephemeral loopback listeners, send shutdown and
await their server with a three-second bound; timeout cleanup aborts and awaits
the task, and Drop also signals/aborts on unwinding. Passing tests assert listener
connection refusal after shutdown. All 16 concurrent test threads per case are
joined. No external service or unowned background worker remains.

The original failed compile (Governor's default middleware clock type) and first
strict-Clippy failure (`result_large_err` for native Response) are retained along
with their passing corrections. The latter uses the established narrow native
response lint exemption; final lint attributes do not change tested behavior.
Two bounded verification corrections were used, with no runtime behavior failure,
operational retry, harness correction or capability recovery. Passing product
tests were not repeated to regenerate evidence.

Raw command/stdout/stderr/exit evidence is retained at
`/tmp/yetlet-yt117-rate-limit-20261022-tmp2roy7/`, including
`worker-commands.txt`, `test-repair1.*`, `clippy-repair2.*`, `doctest.*`,
`rustdoc.*`, `fmt-final.*`, `normal-graph-comparison.json` and `lock-delta.json`.
This is new capability verification, not a benchmark, dependency advisory scan,
browser runtime qualification or production/multi-instance quota claim.

`cargo xtask check docs` PASS: 75 required documents and 126 backlog items;
internal links, Markdown structure, whitespace and dependency DAG pass. The new
contract and crate README also received direct relative-link, fence, final-newline
and whitespace checks because the checker has a fixed required-document list.

## Initial independent acceptance and changed files

Independent Reviewer `/root/yt117_rate_limit_independent_review_20261022`
returned **PASS**, with zero issues and zero required followups. Its registered
reviewer role, configured `gpt-6-astra` model and effective high effort were
verified through native trace and the shared resolver. All 15 source hashes
matched before and after review and again before documentary closure. The
reviewer inspected the implementation, selected contract, Governor arithmetic,
focused evidence, cleanup and isolation; no tests or services were rerun.

YT-117 was marked Done at the selected native per-process memory scope. At that
initial closeout, only status/evidence documents changed; reviewed runtime and
Cargo files remained byte-identical. No commit or push was made. YT-118 Audit research is
the next independent Todo candidate and needs separate owner selection.

Changed files are the workspace [Cargo.toml](../Cargo.toml) and
[Cargo.lock](../Cargo.lock); new crate [manifest](../crates/yetlet/Cargo.toml),
[README](../crates/yetlet/guides/rate-limit.md),
[implementation](../crates/yetlet/src/rate_limit/mod.rs),
[state tests](../crates/yetlet/src/rate_limit/tests.rs),
[middleware/HTTP tests](../crates/yetlet-rate-limit/tests/middleware.rs) and
[native example](../crates/yetlet/examples/rate_limit/native.rs); this contract,
[Backlog](BACKLOG.md), [Roadmap](ROADMAP.md), [documentation index](README.md),
[Extensions](EXTENSIONS.md), [M21 plan](YT-113-123-OPT-IN-CAPABILITIES.md) and
[ADR 0005](adr/0005-opt-in-capability-modules.md).

The raw review report is retained as `independent-review.json` alongside the
pre-review hashes and final closure checks in the evidence directory above.

## Settings-loader follow-up — 2026-10-22

**Authority:** The owner explicitly selected connecting the example's enabled and
policy values to the existing settings loader, default disabled, with focused
verification and independent Reviewer PASS before reporting completion. No commit
or push. The original capability and its independent review remain accepted;
this follow-up changes startup configuration/example wiring only.

The existing provider is the opt-in [yetlet-config typed loader](CONFIGURATION.md),
not a Core service. The example selects it and Serde in development dependencies.
No loader/provider changes, Core API, default runtime dependency, hot reload or
application migration are selected. The existing limiter algorithm and middleware
runtime stay unchanged. Cargo requires explicitly excluding the existing nested
`crates/yetlet-config` and `crates/yetlet-secrets` workspaces from the parent
workspace when this path dependency is selected; their own manifests, locks
and APIs are not modified.

The application owns `Settings { enabled, policy }`, where `policy` holds
`interval_ms`, `burst` and `max_keys`. Loader reads the explicit
[example settings directory](../crates/yetlet/examples/rate_limit/settings/base.toml)
at startup, applying base TOML, exactly one selected finite profile, then OS
snapshot overrides. The prefix `RATE_LIMIT` preserves `RATE_LIMIT_ENABLED`;
policy overrides use `RATE_LIMIT_POLICY__INTERVAL_MS`,
`RATE_LIMIT_POLICY__BURST` and `RATE_LIMIT_POLICY__MAX_KEYS`. The former flat
policy variables are now unknown keys and fail loading. All checked-in profiles
default to disabled; absent profile selection remains Production.

Both settings structs deny unknown fields. Loading/type/known-key errors retain
the provider's fail-closed startup behavior even when disabled; domain validation
and Policy/Budget/layer construction are enabled-only. Disabled native routing
performs no extraction/accounting/pruning. Enabled values must pass Policy's
existing bounds before serving. A fixed `--check` success path loads and validates
settings without binding a listener. Restart is required for setting changes.

The [crate README](../crates/yetlet/guides/rate-limit.md#opt-in-settings-loader-in-the-native-example)
shows actual TOML, provider environment names and runnable commands. Focused
example tests will use the existing Loader's environment snapshot API and actual
settings files; accepted library/Governor/HTTP/Auth/Valkey/browser tests will not
be rerun. This is configuration adoption, not benchmark qualification.

**Follow-up status:** Done after focused verification and separate independent
Reviewer PASS; original acceptance and failures remain unchanged.

### Settings follow-up verification

`cargo test -p yetlet-rate-limit --example native --offline --locked` PASS:
seven focused tests cover real base/profile/environment precedence, default and
explicit off with unmetered native methods beyond the enabled quota, enabled
custom policy/shared Budget and public isolation, every checked-in profile off,
missing/malformed/unknown/wrong-type inputs, and enabled policy-bound failures.
Disabled accepts type-valid unused domain values without constructing Policy;
provider input/type failures still reject startup. Settings fixtures clean up
through Drop; no residue remains.

Strict example Clippy, package formatting and the compiled native example build
PASS. Five actual compiled `--check` processes, launched from `/tmp` with isolated
`RATE_LIMIT_` environment snapshots, cover default/off, enabled/custom policy,
invalid enabled policy, disabled unused invalid-domain value and disabled invalid
type. Their expected success/failure exits and fixed output pass; all processes
completed, and this path starts no listener. No original limiter state/HTTP,
Auth/Valkey, browser or deployment test was repeated.

The shared root lock adds the config/TOML graph and follows the existing loader's
exact libc pin: **libc 0.2.190 → 0.2.186**. No other existing package version changes
or upgrades occur. Do not claim every graph is byte-identical: Core, both browser
roots at wasm32 and default Admin remain byte-identical, while the server graph
has exactly the libc version change with otherwise identical edges/content. The
limiter's normal graph contains neither config nor secrets; both added crate
manifest dependencies are development-only. A focused
`cargo check -p yetlet-server-axum --offline --locked` checks compatibility with
this changed shared native dependency; prior passing runtime suites are reused.

The initial Cargo `multiple workspace roots found` failure is retained with the
minimal root exclusions and passing correction. This used one compatibility
repair, with no operational retry or capability recovery. Raw command outputs,
exit codes, compiled-process receipts, graph comparisons and lock delta are in
`/tmp/yetlet-yt117-settings-20261022-qw8izz1v/`. The original implementation and
review directory above remains intact.

### Settings follow-up independent acceptance and wiring

Independent Reviewer `/root/yt117_settings_independent_review_20261022` returned
**PASS**, zero issues and zero required followups. Registered role, configured
`gpt-6-astra` and effective high effort were verified through native trace and
the shared resolver. All 19 workspace source hashes matched before/after review;
all 14 incremental targets were inspected. The reviewer specifically confirmed
example-only opt-in dependencies, startup provider wiring/default off, native
route preservation, the only existing-version change (libc), focused evidence
and completed-process/fixture cleanup. No tests or services were rerun by review.

The actual example loads and composes using:

```rust
let settings: Settings = loader().load()?;
let rate_limit_policy = settings.build_rate_limit_policy()?; // None when disabled.
// The --check branch returns here, before routes/Budget/listener construction.
let app = routes(rate_limit_policy, |_: &Parts| {
    Ok(Key::new("example-group").expect("bounded constant"))
});
```

`loader()` is an ordinary example function calling the existing
`Loader::new(manifest_dir.join("examples/settings"), "RATE_LIMIT")`. Settings is
application-owned, not a new provider trait or Core field. The application can
supply the same typed values through an explicitly selected existing settings
source; the limiter itself receives only a validated Policy and trusted key
extractor.

Incremental files for this follow-up are the root Cargo.toml/Cargo.lock, crate
Cargo.toml/README/native example, four new settings TOMLs, and the Backlog,
Roadmap, Extensions, ADR 0005 and this evidence document. Limiter library,
state tests and HTTP tests remain byte-identical to their original acceptance.
The final closure changes status/evidence documents only; all reviewed Cargo,
example and TOML bytes remain unchanged. The complete working tree has 19
uncommitted paths, with zero staged files and unchanged HEAD. No commit or push.

Follow-up review and closure receipts are retained as `independent-review.json`
and `closure-checks.json` under its evidence directory. YT-118 remains the next
independent Todo candidate, requiring separate owner selection.

### Owner-selected naming clarification before commit

The owner requested capability-specific names following Rust community naming
conventions. The private example constructor is `build_rate_limit_policy()`;
it validates and constructs an optional value, rather than merely reading the
`policy` settings field. The example aliases the public
`yetlet_rate_limit::Policy` as `RateLimitPolicy`, uses `RateLimitPolicySettings`
for its typed configuration, and carries `rate_limit_policy` at composition.
The public crate API and TOML/environment schema are unchanged. This selection
uses method snake_case and type UpperCamelCase from the
[Rust API Guidelines](https://rust-lang.github.io/api-guidelines/naming.html);
the precise capability-specific names are application choices, not mandated by
the guideline. Focused compile/tests and independent delta review precede a new
strict/builtin staged guard; the earlier guard applies only to its old tree.

The naming delta passed seven example tests, one changed README doctest, strict
example Clippy, formatting and docs checks. Independent Reviewer
`/root/yt117_naming_delta_review_20261022` returned **PASS**, zero issues/required
followups; role/model/effective high effort were verified. All 19 hashes matched
before/after its read-only review. It confirmed unchanged behavior/public API and
configuration schema. Naming logs and `naming-independent-review.json` are retained
in the Settings follow-up evidence directory. No wider runtime suite was rerun.

## Subsequent naming compatibility follow-up

Historical checkpoint: the following paragraph describes the separately accepted
slice committed as `83f417e`. The later owner-selected
[pre-public naming migration](NAMING.md#pre-public-change-discipline) removes
the former Rust aliases; external settings keys and the original evidence remain
unchanged. See the [audit response](NAMING-AUDIT-20261022.md) for current results.

The owner selected [five compatibility-preserving naming decisions](NAMING-COMPATIBILITY.md)
after commit `9189b23`. New Rust names use `refill_interval` and
`refill_interval_ms`; the latter explicitly maps to existing TOML `interval_ms`
and unchanged `RATE_LIMIT_POLICY__INTERVAL_MS`. The application constructor remains
`build_rate_limit_policy()`. Example snapshots use `Loader::load_with_env`;
`load_from_env` remains supported. The canonical rate-limit construction error is
`InvalidRateLimitInput`, with `InvalidConfiguration` retained as the same unit
error under its former name. Original validation/review history remains intact;
this follow-up has separate focused compatibility checks and independent review.
