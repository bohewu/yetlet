# YT-128 — Opt-in Starter observability

Date: 2026-10-23. Status: **Done at bounded local A2 scope after independent Reviewer PASS**.
Independent Reviewer PASS has zero issues and required followups; controller acceptance is complete. This record describes locally verified application behavior, not qualified
deployment behavior. YT-127 remains accepted at
its existing bounded scope; YT-129–YT-131 have subsequently been independently
accepted at their bounded scopes.

## Selected application design

The Starter selects `prometheus` 0.14.0 without default features in its own nested
workspace. `observability::Metrics` owns an independent `Registry`; it does not
install the global default registry, a recorder, subscriber, exporter, collector
thread or second runtime. Core, HTML, DOM and browser manifests are untouched.
Resolved Core/HTML/DOM/Todo-browser normal dependency trees exclude prometheus;
root manifests/lock and browser boundary sources remain unchanged.

Typed `Settings.metrics.enabled` defaults to false, including when an older
settings file omits the entire table. The existing Loader supports
`APP_METRICS__ENABLED=true` or `[metrics] enabled = true` in the selected settings.
The binary constructs collection only when enabled and attaches it to the Store
before cloning into consumers. Library callers can explicitly use
`Store::open(path)?.with_metrics(Arc::new(Metrics::new()?))` before constructing
`web::app` or `web::app_with_admission`. Existing constructors and StoreError
remain source-compatible; pre-existing clones keep their original collection
choice, so attach once at application composition time.

When enabled, `GET /metrics` returns Prometheus text format 0.0.4 on the existing
loopback listener, with no-store and the existing security headers. It performs
no SQL, Session access or new readiness probe. It bypasses HTTP admission so
operators can scrape during saturation. The route has the same local operator
trust boundary as health endpoints; it has no separate authentication and must
not be published through an unrestricted proxy. Disabling collection omits the
route and registry; the route returns the existing 404 behavior.

For a local instance with metrics explicitly enabled:

```bash
curl --fail http://127.0.0.1:3010/metrics
```

Use an operator-owned Prometheus scrape job against that loopback address (or an
explicitly secured deployment boundary). Retention, access controls, scrape
frequency and log rotation belong to the operator. No collector service was
started or qualified by this attempt.

## Signals and bounded labels

All names have the `starter_` prefix. No request URL, path, request ID, actor,
Session ID, body, SQL, raw error message or credential is a metric label.

| Metric suffix | Labels and observation |
|---|---|
| `http_requests_total` | `method`: GET/POST/PATCH/PUT/DELETE/HEAD/OPTIONS/OTHER; `status`: finite numeric HTTP status. Counts responses created, including admission errors, health, 404/405; excludes the enabled metrics path |
| `http_request_duration_seconds` | Same eight method values; histogram boundaries 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1, 2.5, 5, 10, 60 seconds plus infinity; ends at response creation |
| `http_requests_active` | No labels; live observer futures, including briefly rejected requests; RAII drop decrements, even on cancellation; excludes metrics |
| `http_admission_rejections_total` | `reason`: capacity/shutdown/deadline; distinguishes HTTP admission or response deadline failures from SQL errors |
| `sql_admission_total` | `outcome`: acquired/saturated/closed; actual non-waiting permit attempts |
| `sql_admission_wait_seconds` | No labels; records **zero** per attempt, with zero and infinity buckets. There is no permit queue. This does not measure SQLite lock waiting or Tokio blocking-pool scheduling |
| `sql_operations_active` | No labels; admitted closures, including detached or blocking-pool queued work, through connection closure and completion cleanup |
| `sql_operations_total` | `outcome`: ok/not_found/conflict/unavailable/invalid_schema/internal; records returned operation result, including after caller cancellation |
| `readiness_last_probe` | No labels; -1 before first completed probe, 0 not ready, 1 ready. This is a sampled result, not continuously evaluated health |
| `readiness_checks_total` | `outcome`: ready/not_ready; completed probes only |

The method/status counter has at most eight method values times the HTTP library's
finite three-digit status range; labels cannot grow with arbitrary URLs or IDs.
Only observed combinations are emitted. No route label is selected. Registry
snapshots span concurrent updates and are not transactional snapshots.

Throughput is the rate of the response counter; error distributions use its
status label. Histogram quantiles require a collector and adequate samples;
they are estimates and exclude futures dropped before response creation.
Cancelled futures still release their active gauge. Metrics counters reset on
restart. Scrapes exclude themselves; health traffic is included. SQL active
capacity stays four, HTTP default capacity stays 32, and both queues remain zero.
The five-second HTTP, one-second readiness and ten-second shutdown budgets are
unchanged. The metrics route adds an explicit control-plane admission exception;
transport/connection limits and scrape overload protection remain operator work.

## Tracing and lifetime limits

The existing single JSON subscriber remains owned by the binary. The observer
instruments its future with a native `http_request` span containing only the
process-local generated request ID and finite method. No entered span guard
crosses await. Store captures the span and dispatcher before `spawn_blocking`
and enters them only inside synchronous closure execution. `sql_finished`
records one bounded outcome. Incoming correlation/trace headers are untrusted
and are not adopted; request IDs remain non-identity and reset across processes.

Existing startup/shutdown events (`started`, `shutdown_requested`, `stopped`,
`shutdown_incomplete`) provide lifecycle observation. `readiness_checked` logs
a boolean and `http_admission_rejected` a finite reason. INFO filtering can
suppress these events; metric collection does not depend on tracing filters.
No transition counter, persistent lifecycle metric, distributed tracing,
CPU/RSS sampler, SQL statement timing or per-route dashboard is selected.

HTTP timing ends at response creation, not body delivery. A timed-out/dropped
caller cannot cancel an already dispatched blocking SQL closure. Its later
result can still be correlated with the earlier request and its gauge remains
active until completion. Span completion, an HTTP 200, a returned SQL operation,
or a diagnostic event is **not a durable business-write or audit acknowledgment**.
A lost response still requires reconciliation under the YT-127 unknown-write
contract. Operation counters include reads and probes; panics do not produce a
normal result event, although RAII releases active accounting.

Primary references checked against tracing 0.1.44 in the app lock and local
upstream source: [Instrument](https://docs.rs/tracing/latest/tracing/trait.Instrument.html),
[Span](https://docs.rs/tracing/latest/tracing/span/struct.Span.html), and
[Prometheus naming](https://prometheus.io/docs/practices/naming/).
The downloaded prometheus 0.14.0 registry implementation provides the local
`Registry::new/register/gather` API used here. No tracing/Tokio upgrade was made.

## Initial verification and preserved failure history

Source baseline: branch `docs/yt113-123-opt-in-capability-plan`, HEAD
`5db8b8c61b6d319a641814dff4af493ce3e5d965`, with accepted T0/T1 dirty work
preserved. No staging, commit, push, branch or worktree change occurred.
Evidence lives in
`.pipeline-output/prod-readiness-phase2-20261023T020406Z/observations/T2/`.

| Command / observation | Actual result |
|---|---|
| `cargo fetch --manifest-path examples/production-starter/Cargo.toml` | Exit 0; app-only lock adds prometheus and eight dependencies; existing package versions retained |
| `cargo fmt --manifest-path examples/production-starter/Cargo.toml` | Exit 0 during initial implementation; final check pending |
| `cargo test --locked --offline --manifest-path examples/production-starter/Cargo.toml` | Exit 101: 11 unit tests pass; 10 of 11 HTTP tests pass; scoped tracing capture fails under concurrent tests |
| Same test command with `--test http tracing_observes_generated_request_id_without_urls_headers_or_bodies -- --exact --nocapture` | Exit 0: isolated tracing/correlation/privacy test passes |
| Same test command with `--test http` after adding assertion diagnostics only | Exit 101: repeated tracing failure; capture buffer is entirely empty, other 10 HTTP tests pass |

Passing focused assertions cover typed default-off/override/legacy settings,
enabled/disabled endpoints, independent registries, fixed label categories,
metric privacy, readiness schema-failure signal, HTTP saturation/deadline
metrics, and SQL gauges/counters surviving caller cancellation and late commits.
The correlation test runs concurrent SSR/API calls but its scoped subscriber
captures no events alongside the other tests. The same test passes alone. This
is classified as **harness_failure**, with no production failure established;
the repeated-signature stop rule ended the initial attempt.
Only assertion diagnostics changed after failure; no semantic harness repair or
product repair was attempted.

At the end of the initial attempt, actual localhost fault collection, source-bound
runtime diagnostics and shutdown proof, strict Clippy/build, final
format/docs/whitespace checks and resolved Core/browser dependency checks were
**not run**. No listener, exporter or
collector process was started; Cargo processes completed and test temporary
databases were owned by tempfile. Historical qualification and the YT-127
one-shot local proof were not rerun. That initial evidence did not establish the A2 actual-fault gate. Independent
review remains pending.

## Same-task authorized resume and local fault evidence

The user authorized additional budget in the same canonical run. A read-only
diagnosis identified tracing-core's single-registered-dispatcher interest cache:
concurrent uninstrumented callsites could cache `Interest::never`. The existing
scoped test now retains a second newly registered `NoSubscriber` dispatcher for
its lifetime. Assertions, production tracing and runtime behavior are unchanged.
This is one in-place harness correction, not a product repair. The focused
HTTP suite ran once with 11 test threads and all 11 tests passed. Original failed
logs and `task-result.json` are retained; see `debugger-diagnosis-1.json` and
`resume-task-result.json`. The 11 prior passing unit tests are reused only after
hash comparison confirmed all production Rust sources unchanged.

| Command / observation | Actual result |
|---|---|
| `cargo test --locked --offline --manifest-path examples/production-starter/Cargo.toml --test http -- --test-threads=11` | 11 passed, including scoped correlation/privacy, enabled/disabled endpoints, bounded labels and real SQL lifetime after caller timeout |
| `cargo clippy --locked --offline --manifest-path examples/production-starter/Cargo.toml --all-targets -- -D warnings` | Exit 0 |
| `cargo build --locked --offline --manifest-path examples/production-starter/Cargo.toml` | Exit 0; executable SHA-256 recorded below |
| `cargo tree --locked --offline -p yetlet-core -p yetlet-html -p yetlet-dom -p yetlet-todo-browser --edges normal` | Exit 0; no prometheus; unchanged root/browser manifests and sources |
| Direct Python standard-library localhost/SQLite command in `resume-local-command.txt` | PASS; raw JSON logs, four metric snapshots and result retained |

The local command used an ephemeral loopback port, temporary SQLite data,
Production settings, synthetic credentials and `APP_METRICS__ENABLED=true`.
Limits remained HTTP 32, SQL 4, both queues zero, HTTP deadline 5 seconds,
SQLite busy timeout 2 seconds, readiness deadline 1 second and shutdown grace
10 seconds. Binary SHA-256:
`4dc7dec913722dbcf86a8677790e69d20fa27a3676e855e31fd9f238a40fcb29`.

A separate SQLite connection held `BEGIN IMMEDIATE` while four concurrent API
writes ran. Scraping observed SQL active 4 and HTTP active 4. A fifth write and
a readiness probe were rejected by saturated SQL admission; the scrape stayed
available. The four blocked writes returned 503 after SQLite busy timeout.
Their generated request IDs (4–7) each linked `sql_finished outcome=unavailable`
to the HTTP 503 event. The saturated fifth request used ID 10. Fault collection
reported two saturated admissions, four unavailable operation results,
readiness 0, zero permit queue wait, and both active gauges back at zero.
Zero permit wait correctly distinguishes immediate admission from SQLite lock
contention; metrics alone do not identify the lock holder.

After rollback of the injected lock, a new write returned 201, a readiness probe
returned 200 and the isolated database contained exactly the one recovered row.
The four failed writes created no rows. This controlled readback verifies this
scenario; it does not make HTTP success or a span event a durability guarantee.
The existing passing HTTP test separately uses a 100 ms deadline and proves a
SQL gauge remains 1 after HTTP 504, then reaches 0 after lock release and late
commit. That test does not change the binary's five-second deadline.

The live JSON logs and metrics excluded synthetic credential, Session, header,
trace-header, query, body and user markers; incoming request ID 987654321 was
not adopted. Metrics excluded request/user IDs and URL/path labels. The test
suite additionally exercises 20 distinct private paths and an arbitrary method,
which maps to the single `OTHER` label. Restarting with metrics false produced
404 at `/metrics` and 200 readiness. An invalid boolean exited 1 with only the
fixed `configuration_invalid` category before starting a listener.

Both enabled and disabled instances received SIGTERM, exited 0 and were reaped;
enabled logs include `started`, `shutdown_requested`, `stopped`. HTTP clients
closed, the four-worker client pool joined, temporary SQLite/WAL data were
removed and the port refused a final connection. No collector/exporter service
was created. One initial command redirection typo failed before execution;
one operational correction ran the direct command successfully. No product
repair or additional fault run was needed.

Final format/docs/whitespace results are recorded in the resume result and logs.
Independent Reviewer PASS and controller acceptance complete YT-128 at local A2 scope.
YT-129–YT-131 have subsequently been independently accepted at their bounded
scopes; the next remaining decision is **YT-096 D1–D6**. No B1 load
qualification was performed in this A2 exercise. YT-096 remains In progress with D1–D6 Pending;
YT-020 remains Blocked. No broader production-readiness claim is made.

Independent review record: `.pipeline-output/prod-readiness-phase2-20261023T020406Z/observations/T2/independent-review.json`. Reviewed source hashes stayed unchanged; subsequent controller edits only record acceptance.
