# YT-123 — Tracing, correlation and propagation decision

**Date:** 2026-10-23. **Source:** `5316859e116993b84b47685ea2b6e8cea100faf2`.
**Status:** Done at research scope after independent Reviewer PASS, zero issues
and required followups.
**Decision:** Retain the optional `yetlet-request-id` provider and application-owned
native tracing/Tower composition. No `yetlet-tracing` crate, propagation API,
subscriber, exporter, background agent or runtime is selected.

The owner selected documentation-only research under the
[M21 plan](YT-113-123-OPT-IN-CAPABILITIES.md#yt-123--tracingcorrelation-adapter-study-research-only).
Compare real SSR/API, structured diagnostics and asynchronous work before
extracting tracing glue. A shared numeric request label is useful correlation;
it does not deliver distributed tracing, trustworthy remote identity or an
audit record. Research completion does not qualify instrumentation that has
not been implemented or expand the original nine release candidates.

## Observed consumers and actual propagation

| Inspected consumer | Implemented behavior | Propagation and evidence limit |
|---|---|---|
| [Request ID provider](../crates/yetlet-request-id/src/lib.rs), `RequestIds`, `assign_request_id` | Explicit native global/scoped installation creates a typed request extension and `x-yetlet-request-id` response header without reading/buffering the body | Clones share one allocator; separate providers/restarts/replicas can reuse numbers. No span, logging, inbound trace parser, exporter or automatic response extension/HTML serialization |
| [Todo SSR/API](../examples/todo-server/src/lib.rs), `root`, `app`; [API middleware](../examples/todo-server/src/pipeline.rs) | One explicitly installed global provider covers the assembled router. Root deliberately selects only the number into inert `data-request-id`; API/error paths receive the response header | Native SSR selection is not copying RequestContext/Extensions into hydration. The selected router's single provider aligns handler-visible and response ID; no browser trace propagation is established |
| [REST API](../examples/rest-api/src/lib.rs), `app`, `api_policy`, `api_error` | Global provider adds correlation on health/API/global errors; DTOs/error JSON do not carry the ID | Header correlation, not a span or downstream trace. Matched API policy and global ID scopes differ; do not infer universal matched-route metadata/header coverage |
| [Production Starter](../examples/production-starter/src/web.rs), `app`, `observe`; [binary](../examples/production-starter/src/main.rs) | Outer RequestIds runs before the observer. One JSON INFO event records `event="http_request"`, numeric request ID, finite method category, numeric status and duration in milliseconds | Event excludes URL/path/query/headers/body/actor/error text. Timing ends when the Response returns, not at body completion, client delivery or business commit. Dropping the observer before return emits no completion event |
| Starter [SQL store](../examples/production-starter/src/store.rs), `Store::run`; [deadline fixtures](../examples/middleware-lab/src/deadline.rs) | Store captures an owned permit, path and operation into bounded blocking work. Deadline tests model surviving spawned/blocking work and a body that outlives service timeout | No RequestId/Span/Instrument capture in the inspected Store path. The timeout cases are source-inspected test fixtures, not a durable application job system or newly executed proof |
| [SSE Notifications](../examples/sse-notifications/src/lib.rs), `events`, `LiveStream`; [Redis command workers](../crates/yetlet-session-redis/src/store.rs), `Store::call` | SSE body polling/drop owns bounded subscription/replay lifetime. Provider command tasks own admitted IO and reply through oneshot | SSE event IDs are replay positions; per-command workers are provider IO. Neither demonstrates persistent job correlation, automatic tracing inheritance or a second request-span adapter |

The bounded Rust/manifest/name search found no durable application-job consumer.
It did find spawned IO and timeout fixtures; do not manufacture queue/retry
semantics or label them as production background jobs. Middleware Lab's local
`Trace` is a vector of static ordering markers, not Rust tracing or W3C context.
Only Starter demonstrates application HTTP-to-tracing events in these inspected
paths. There is no repeated shared span/propagation implementation to extract.
Existing tests and historical [YT-092 evidence](YT-092-STARTER-VERIFICATION.md)
were inspected/reused at their original scope, not rerun or refreshed.

## Identity, namespace and source trust

| Correlation concept | Meaning and authority |
|---|---|
| `RequestId(pub u64)` | Predictable provider-local label. Public construction and typed storage confer no authenticated provenance. Read from the deliberately selected trusted middleware, not an arbitrary incoming header; never use it for authorization, Session continuity or idempotency |
| Tracing span/event | Diagnostic execution context and selected facts. A span ID is local to its tracing machinery, not automatically a network trace identifier, durable job identity or business result |
| Remote trace context | Optional cross-service correlation under a separately selected validation/trust policy. Neither remote parent IDs nor sampling flags authenticate a caller, tenant or operation |
| Operation/job/attempt reference | Application-owned lifecycle identity if a real job/retry consumer needs one. Correlation with an originating request is separate from durable intake, retries, deduplication and transaction acknowledgment |
| Actor/security audit | Current trusted local identity and explicit domain outcome; protected retention/delivery remains [YT-118](YT-118-AUDIT-STUDY.md), not a property of request IDs or tracing |

`RequestIds::new()` starts at 1; `try_new` accepts 1 through `u64::MAX - 1`.
The allocator refuses exhaustion instead of wrapping; its early 503 skips
downstream and has no provider-added ID header. It is provider-local even within
one process. Cross-instance interpretation would need an explicitly selected
instance/lifetime namespace or a different correlation strategy; none is
provided by this research.

Incoming correlation headers remain untouched. The middleware replaces the
typed request extension and, on unwind, the response header; it does not sanitize
or adopt the raw request header. Downstream custom logging can still encounter
untrusted input. Duplicate/nested installation has no rejection guard: source
inspection shows an inner provider can replace the handler-visible extension
while the outer provider later writes its captured ID to the response. This is
a composition hazard inferred from source, not a newly run test or selected
repair. Install one chosen generator per request path and preserve the existing
[scope/order contract](EXTENSIONS.md#the-independent-opt-in-proof-request-correlation).

[W3C Trace Context](https://www.w3.org/TR/trace-context/) describes network
propagation and its privacy/abuse risks. Any later application integration must
use a maintained propagator, bound/validate remote context and choose trust
boundaries, sampling and outbound destinations explicitly. Untrusted sampling
is not permission for unlimited telemetry work; trace state can carry sensitive
metadata. Do not forward arbitrary headers or baggage, encode personal/secret
data into trace fields, or automatically echo remote context to SSR/browser
responses. No W3C parser, HTTP client adapter or distributed propagation is
implemented/selected here.

## Native tracing and Tower comparison

| Existing mechanism or native option | Decision for this study |
|---|---|
| Existing RequestIds plus selected SSR/header projection | Retain for local request correlation. It can supply an allowlisted field to later application instrumentation; it is not replaced or renamed |
| [Rust tracing](https://docs.rs/tracing/latest/tracing/) events/spans and application subscriber | Reuse directly when a real consumer needs structured diagnostics or execution context. Starter already selects events and its own JSON subscriber; no framework subscriber default is needed |
| [Tower HTTP trace](https://docs.rs/tower-http/0.6.11/tower_http/trace/index.html) | Optional native `trace` feature with configurable span/callback behavior. Response-return, body and failure callbacks have distinct observation points; this is a candidate mechanism, not an installed Yetlet tracing layer |
| A common `yetlet-tracing` module | Deferred. Repeated ID headers, one HTTP event projection, static ordering markers and task APIs do not demonstrate duplicated replaceable tracing glue |

The consulted tracing documentation is 0.1.44. Tower HTTP references deliberately
use 0.6.11, the repository lock's existing version, rather than substituting
latest documentation for a dependency update. No feature, dependency/version
pin, upgrade, new setting or compile/performance claim is selected.

Native defaults still need application field selection. Tower's
[default span source](https://docs.rs/tower-http/0.6.11/src/tower_http/trace/make_span.rs.html)
records URI even when headers are disabled; blindly installing it would not
preserve Starter's URL/query exclusion. The
[`instrument` attribute](https://docs.rs/tracing/latest/tracing/attr.instrument.html)
records arguments by default, including Debug for non-value types. Select
`skip`/`skip_all` and explicit safe fields before adopting it around requests,
State, principals, errors or credentials. Filtering a level or enabling JSON
formatting is not redaction of fields already emitted.

## SSR, API, task and stream lifetime boundaries

| Boundary | Minimum decision for later instrumentation; current behavior stays unchanged |
|---|---|
| HTTP entry and route scope | Generate/select correlation before the observer needs it. The outer pre-routing envelope has no application matched-route metadata; use a fixed safe category there or an intentionally selected post-routing template. Capture allowlisted request fields before moving the request; response Extensions do not automatically inherit them |
| SSR and API response | Keep correlation server-side except deliberately selected inert fields/headers. Preserve status/security/Session policy and per-request metadata; [YT-122](YT-122-OUTPUT-CACHE-STUDY.md) must not replay an earlier response's request ID. DTOs, View state and errors do not gain automatic trace data |
| Async future or spawned task | Explicitly attach a selected span/context for the task lifetime using native mechanisms. Do not assume request extensions, thread-local entry or caller correlation follows a task automatically. Copy only bounded owned safe metadata, not the whole request/Session/provider secrets for logging |
| Blocking work | Capture the chosen context before crossing into the closure, and scope it inside synchronous work if selected. Record caller timeout/cancellation separately from worker completion/committed effect; correlation neither cancels started blocking work nor rolls it back |
| Detached/durable job or retry | Requires a real application consumer and explicit job/operation/attempt identity, originating-request linkage and parent/link policy, terminal/unknown outcomes and shutdown ownership. Do not stretch an HTTP completion measurement into a job result or treat its local counter as persistent identity |
| SSE/streamed body | Admission/response return, later body progress, termination/drop and any failure are separate facts. Preserve bounded queues, disconnect cleanup and current access checks. A starting 200 or replay cursor is not successful complete delivery; no whole-body buffering or per-chunk payload logging |

[`Instrument`](https://docs.rs/tracing/latest/tracing/trait.Instrument.html)
attaches a span while a future is polled. The
[`Span` documentation](https://docs.rs/tracing/latest/tracing/struct.Span.html#in-asynchronous-code)
warns against holding entered guards across `.await`; `in_scope` is for
synchronous work. These are native integration options, not new Yetlet wrappers
or evidence that current SQL/SSE tasks already propagate spans. App-selected
subscriber/filter ownership remains separate from copying a request label.

[Tokio's blocking-task contract](https://docs.rs/tokio/1.53.2/tokio/task/fn.spawn_blocking.html)
and [YT-029](YT-029-HTTP-DEADLINE-RESEARCH.md) retain the started-work cancellation
limit. Future observation must distinguish timed-out waiting, dropped response,
completed work and committed effect. Even body completion is not a domain
transaction or durable audit acknowledgment. Do not infer success from a span
drop, HTTP status, log enqueue or subscriber delivery.

## Sensitive-data projection and operational ownership

Preserve the [threat-model exclusions](THREAT-MODEL.md#13-logging-audit-metrics-and-privacy).
Default to an allowlist selected before formatting: finite event/operation name,
trusted local correlation in its namespace, finite method/status/failure class
and explicitly defined duration/outcome. Starter's missing-extension sentinel 0
is not a generated trusted identity. Its `duration_ms` ends at Response return;
formatter metadata is separate from these application event fields.

Never dump RequestContext, State, Extensions, Principal Debug, raw URI/query,
Cookie/Authorization/Set-Cookie, Session IDs, CSRF, OIDC code/state/nonce/tokens,
passwords/secret-provider values, business bodies, SQL parameters, file contents
or uncontrolled error chains. Optional diagnostics must not collect them even
when a client supplies a correlation header. Bound any intentionally selected
text/category and encode for the output format; a structured event does not
make arbitrary business content safe. Request IDs/user IDs/raw paths/error text
are not metric labels. Correlation IDs can themselves be linkable metadata;
access/retention/export policy remains explicit.

Starter's binary initializes the subscriber once and accepts typed
Error/Warn/Info/Debug settings, default Info; its INFO request/lifecycle events
are filtered at Error/Warn. Its code does not install EnvFilter/RUST_LOG. The
library app constructor does not initialize a subscriber. No resolved-graph
claim of zero transitive tracing through Axum is made: inspected manifests show
no new framework subscriber/exporter selection, and no dependency build was run.

Applications/operators own subscriber composition, field policy, sampling,
bounded output/queues/retries, failure/drop visibility, shutdown drain, storage
access, retention/deletion and optional exporter destinations. Do not turn a
diagnostic outage into an authorization fallback or pretend a missing event is
an audit delivery guarantee. No telemetry worker, cloud account, exporter or
durability contract is selected through this study.

## Extraction threshold and completion gate

Reconsider only when independent real consumers repeat the same useful bounded
span construction, safe projection and propagation glue across their actual
HTTP/task lifetimes. Identify repeated functions and the replaceable mechanism
native tracing/Tower cannot already compose simply. Select provenance, privacy,
namespaces, sampling, cancellation/body phases and output-failure ownership
first; any minimal implementation/adoption requires separate owner selection,
focused evidence and independent review. Do not add a job consumer or exporter
just to justify a candidate crate.

Retain explicit native Axum/Tower scopes, typed State/Extensions and ordinary
constructors. No fixed RequestId/TraceId field, DI/registry, automatic SSR/state
serialization or server-only browser/WASM dependency is introduced. Primary
references were checked on 2026-10-23. This is source/documentation research;
no runtime, compile, browser/CSP, Windows, Valkey/IdP, benchmark or security scan
reruns. No private settings/credentials were read or services started.

Independent Reviewer `/root/yt123_tracing_study_independent_review_20261023`
PASSed with zero issues and required followups. It inspected the actual
eight-document patch, critical consumer/provider source and primary references,
including exact local Tower HTTP source when its web source URL was unavailable.
All eight reviewed hashes, HEAD and index were unchanged during review; role,
model and effective effort were verified before acceptance. Only status/evidence
bookkeeping follows that result. `cargo xtask check docs` PASSed (75 required
documents, 126 backlog items), as did changed/new-file links, anchors, English,
fences, EOF and whitespace checks. No runtime validation was performed or
transferred from earlier work.

The repo-external `yetlet-yt123-tracing-study-20261023-pvdkblkm` directory retains
two source memos, `primary-references.json`, `documentation-checks.json`,
`review-source.json`, `independent-review.json` and final `closure-checks.json`.
All eight changed files are documents: this study, Backlog, Roadmap, M21 plan,
documentation index, capability map, Extensions and ADR 0005. No source, manifest,
lock, test or runtime input changes.

YT-123 is Done at research scope. All eleven original M21 entries are now Done
at their individually selected implementation/research scopes; this does not
deliver all candidate providers. No next implementation is automatically selected.
YT-096 stays In progress and YT-020 stays Blocked. HEAD remains `5316859`, staged
files zero, no commit or push.
