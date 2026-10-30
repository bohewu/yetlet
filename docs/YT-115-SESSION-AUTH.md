# YT-115 — Optional Memory Session authentication bridge

**Date:** 2026-10-22. **Status:** Done at the selected Memory bridge scope after
independent Reviewer PASS. The owner selected YT-115, YT-116, YT-119 and YT-120, followed
by one scoped Standard security scan after their individual completion gates.
No commit/push, new IdP, distributed provider or release was authorized.

## Selected slice and concrete API

The [reviewed authentication contract](YT-113-AUTHENTICATION-CONTRACT.md) and
[YT-114 foundation](YT-114-IMPLEMENTATION.md) remain authoritative. The smallest
new surface is [session::memory](../crates/yetlet/src/auth/session.rs), behind
the explicitly enabled `yetlet-auth` **session** Cargo feature. Defaults remain
empty; `yetlet-session` is an optional dependency, not a core/server default.

`memory<T>(parts, identify, retired) -> Result<Option<Principal>, Response>`
reads the existing typed `Session<T>` synchronously and calls an app-owned
mapping function. `T` is the protected server-store identity reference, not a
serialized Principal or caller-supplied role. Install existing Session middleware
outside `authenticate` using ordinary Tower ServiceBuilder; no store registry,
new cookie jar, fixed RequestContext field or mandatory service interface exists.

| Input / event | Selected behavior |
|---|---|
| Successfully read anonymous reference | Mapping returns None; optional extractor is anonymous, required extractor invokes its explicit login/failure policy |
| Successfully read authenticated reference | Mapping checks current local account/issuer/subject/scheme/tenant validity before constructing Principal; business permissions remain separate |
| Session absent from request extensions | Generic no-store 500; fail closed even for optional routes |
| Record expired/destroyed after opening | Invoke the explicitly supplied retired-identity response; never restore authentication |
| Session store/clock unavailable | Generic no-store 503; no anonymous downgrade, login redirect or alternate store |
| Mapping rejects disabled/unmapped account or cannot reach policy service | Preserve its explicit native denial/error; callback owns freshness, redaction and exact account mapping |

Memory's existing middleware can replace a well-formed unknown/expired cookie
with a fresh anonymous Session before the bridge reads it. That behavior is
unchanged: required routes still deny and old CSRF cannot mutate authenticated
state. Malformed/duplicate designated cookies retain provider 400 responses.
No new broad error hierarchy or async rewrite is added. Narrow Clippy
`result_large_err` expectations document the deliberately retained native
Response contract rather than boxing it or weakening project-wide checks.

## Identity, permissions and lifecycle ownership

The HTTP product fixture stores only an optional stable local account reference
under an explicit application identity namespace. Its login decision is
synthetic and already selected by trusted server fixture state; no client field
chooses a user or grants a role. This tests Session/Auth composition, not a
deployable credential verifier. Real login verification stays with the selected
application/provider; this item introduces no password/MFA/OIDC protocol.

The application owns explicit local mapping and current permissions. The
fixture reads live process-local account policy on each request; write and
logout acquire the same application lock, and writes re-read Session/CSRF and
permissions at their commit boundary. There is no stale permission cache in
this selected consumer. Already-running read responses are request snapshots,
not instant cancellation promises. No cross-node freshness interval is claimed.

Successful login uses existing `replace_and_rotate`, changing Session ID and
CSRF. Logout verifies CSRF and destroys the record. Stale in-flight writes
recheck before mutation, and stale responses cannot clear a replacement cookie.
Missing/invalid CSRF is not authentication or business permission. Protected
native hidden CSRF fields remain escaped, no-store and usable without scripts;
Session IDs and Principal are not rendered or put into hydration/logs.

The form consumer explicitly selects local 303 login redirects for anonymous
or retired required identity; definitive policy denial stays 403, provider
outage stays 503, and neither becomes a login redirect. No HTTP challenge scheme
is invented. Normal Session Set-Cookie/Vary/no-store behavior and the default
Secure/HttpOnly/SameSite=Lax/Path=/ settings are preserved. Local HTTP fixtures
explicitly opt out of Secure except the default-cookie attribute test.

## Evidence

| Command / check | Result |
|---|---|
| `cargo test -p yetlet-auth --features session --test session_http --offline` | 7/7 PASS: six actual loopback HTTP journeys and one missing-layer/router check |
| `cargo test -p yetlet-auth --features session --doc --locked --offline` | 2/2 compiled README examples PASS |
| `cargo clippy -p yetlet-auth --features session --all-targets --locked --offline -- -D warnings` | PASS with the documented narrow native-Response lint expectations |
| `cargo fmt --package yetlet-auth -- --check` | PASS |
| Four baseline normal dependency graphs (core/server and both browser packages) | Byte-identical to YT-114's retained before snapshots |
| `cargo tree -p yetlet-auth --no-default-features --edges normal --locked --offline` | No Session, Redis or reqwest in default normal graph; HTTP client is test-only |

HTTP journeys cover native login/form CSRF, identity/CSRF rotation, old-cookie
fixation/replay denial, protected write, CSRF-protected logout and revocation;
expiry/forged/duplicate/malformed cookies; optional-route provider outage; live
permission/account-disable denial; a paused write resumed after logout without
mutation or replacement-cookie deletion; a paused write resumed during provider
outage returning 503 without redirect/mutation; default cookie attributes and
scope isolation for health. Servers use ephemeral loopback listeners, bounded
timeouts and graceful shutdown; Drop aborts owned tasks on test failure.

Source review found the HTTP fixture initially mapped a mid-request store
failure to login navigation. Its application guard mapping was corrected to
preserve 503 and a new in-flight outage regression passed. The library bridge
already distinguished that error. Initial Clippy large-Response diagnostics
were resolved by narrow, explained expectations; runtime tests were not widened
into a new verification framework. Original YT-114 and Session/OIDC passing
workloads were not replayed.

Compile, native HTTP and HTML/CSP-header evidence are distinct from browser
execution. No browser interaction or CSP-enforcement run is claimed, and no
DOM/script/CSP policy changed. This is direct local HTTP, not TLS or production
cookie enforcement, Windows, cluster or live IdP qualification.

`cargo xtask check docs` and tracked/new-file whitespace, changed Markdown
link/fence/EOF checks PASS. Independent Reviewer `/root/yt115_memory_bridge_review_20261022` returned
PASS with zero issues/required followups. Native trace verified reviewer role,
`gpt-6-astra` and effective `high` effort; all 17 working-target hashes matched
through review completion. Only status/review documentation changed afterward. Redis's existing optional async API and single-primary qualified
scope are unchanged; an async bridge is not selected. This does not complete
YT-116 or convert its adapter study into implementation authority.

## Changed files and follow-on

Changes are limited to Auth's optional feature/dependency and lock entry, one
bridge module, its real-HTTP product test, compiled README guidance, this record,
and current backlog/roadmap/navigation. Previously accepted YT-114 changes stay
in the same uncommitted checkout. Existing Session/Redis and application runtime
sources are unchanged. No commit/push occurred.

YT-116, YT-119 and YT-120 are now Done at independently reviewed study scopes;
none selected new adapter runtime. YT-096 remains In progress and YT-020
Blocked; the original immutable release candidate is unchanged.

## Auth group security scan — 2026-10-22

One Codex Security Standard scan, `6b4f365a-e700-4b41-9235-43b998ab788a`,
completed and sealed after the four owner-selected Auth items passed their
independent review gates. Its scope is all six current files under
`crates/yetlet-auth`, including the optional Memory bridge, package guidance and
HTTP fixtures. All six were fully reviewed, with zero reportable findings and
no deferred work. A fresh-context baseline, independent architecture mapping,
focused lifecycle/response investigation and parent source verification retained
source-backed controls and limitations in the generated report.

This is offline static review of the uncommitted Auth source, not HEAD alone,
a whole-repository scan, dependency-advisory rescan, live IdP/Redis test or
production qualification. Supporting Session source explains immediate bridge
controls without claiming a full provider audit. No tests or application code
were executed during the scan. The generated report and canonical model,
findings and coverage are retained in the local Codex Security workbench under
that scan ID; no retained scan artifact was authored or changed by this
documentation closeout. The six audited crate files remain unchanged.
