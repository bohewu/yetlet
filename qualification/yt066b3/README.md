# YT-066b3 — Three-application Redis qualification

This private, unpublished fixture explicitly composes the experimental async
Redis provider with three separate Yetlet/Axum processes and one Redis primary,
behind the accepted YT-066a Nginx topology. See the
[design](../../docs/YT-066B-REDIS-SESSION-DESIGN.md) and
[provider evidence](../../docs/YT-066B2-VERIFICATION.md).

Run from the repository root:

```sh
python3 qualification/yt066b3/run.py
```

The fixture builds with pinned Rust 1.88 using locked, locally vendored inputs,
an offline container and read-only source. It creates one unique application
image and one unique Compose project. The only published port is a random
host-loopback Nginx listener. Applications and Redis share an internal backend;
Redis has no host port. All services are nonroot with read-only roots, dropped
capabilities and resource limits. Redis is pinned at the same digest as b1/b2,
with `appendonly no`, `save ""`, tmpfs and no persistent volume.

`YT_ISOLATED_LAB=1` and explicit namespace/endpoint/instance settings are required.
The `/control/*` routes, synthetic counter state, GET mutations for the no-JS
browser probe, Redis record observation and timing gates are qualification-only.
They are not authentication, production APIs or deployment examples. No auth
secret, Cookie, identity, CSRF, operation ticket or serialized state is logged
in reports. Raw synthetic credentials remain only in request/test memory.

The test gates delay a callback before CAS, a handler before Session finalization,
or an outer HTTP response after finalization. They have finite timeouts and retain
the provider's once-only callback, conflict and deadline assertions. A standard
HTTP Cookie jar observes a genuinely delayed response. Actual Chrome with script
execution disabled separately checks cross-app reads/rotation, a replayed prepared
old Cookie header, explicit recovery and logout rejection. These are distinct
observations; neither implies unconditional Cookie retention.

The selected pure-SSR policy permits login loss after reordered prepared responses
while requiring authoritative rejection of retired identities. No automatic
authenticated create, Memory fallback, JS coordinator or old-ID forwarding exists.
Direct instance routes disable proxy retry; pooled qualification requests are
stateless health reads. The fixture is outside the default Cargo workspace and
does not change the Memory provider or browser dependencies.

The focused matrix covers create/read, cross-process CAS, rotation/revocation,
pre-finalizer suppression, prepared-response reordering, delayed logout,
absolute TTL, unknown outcome and result expiry, caller cancellation, queue/admission
bounds, explicit drain, application stop/restart, outage and controlled Redis
restart. A Redis restart under this selected configuration must lose Session
state and reject old credentials. Recovery deliberately restarts the applications
and creates a new Session; automatic reconnect recovery is not asserted.

Each execution retains its own sanitized report and build logs under ignored
`target/yt066b3-20261011`. The runner reuses the existing reviewed Docker cleanup
method for exact-owner removal and fail-closed rechecks. Cleanup includes the
five containers, two networks, generated application image, build container,
vendor/build work, and owned Chrome process/debug port/profile. Shared pinned
images remain cached. Passing behavior alone is not PASS until cleanup succeeds.
The browser probe runs inside the runner process; cancellation unwinds its
existing owned-browser cleanup before Docker teardown. The runner catches a
first cancellation at the teardown transition and never reports it as PASS.

This proves only a bounded single-host, single-primary, no-persistence topology.
Production TLS/ACL, durable persistence/restore, HA/Cluster, cross-host operation,
load/SLO/RPO/RTO, broad browsers and public release remain unqualified. YT-066
stays In progress and YT-020 stays Blocked.
