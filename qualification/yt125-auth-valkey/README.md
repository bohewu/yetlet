# YT-125: application-owned Auth/Valkey qualification fixture

This unpublished, standalone Linux/native workspace implements the owner-selected
first candidate from the reviewed [YT-124 contract](../../docs/YT-124-MULTI-INSTANCE-AUTH-SESSION-CONTRACT.md).
It is a **test-only application with a synthetic trusted identity**. The login
button does not verify a password, OIDC token, or other credential. Never deploy
it as an authentication application. It is not adopted into Admin or the example
catalog, and it adds no Auth/provider feature, bridge crate, or Core API.

The direct dependencies explicitly select `yetlet-auth` without its Memory
feature and alias the existing `yetlet-session-redis` package as
`yetlet-session-valkey`. Existing package/type names and `redis://` wire endpoint
names do not change the selected Valkey server. Nothing modifies the parent
workspace manifests, lockfiles, existing consumers, or browser graph.

## Selected application contract

Two actual child processes bind separate ephemeral `127.0.0.1` listeners. Each
owns its existing Valkey Session manager. They share one isolated `yt125_*`
namespace, the `yt125_session` cookie, and one trusted task-owned local policy
file. The Session DTO contains a required `account` enum reference, with explicit
`anonymous`, mapped `one`, and denied `unmapped` variants. Unknown fields,
unsupported variants, null, and missing account fields fail decoding. Payloads
are capped at 256 bytes. Principal is never serialized.

The resolver clones the exact `RedisSession<IdentityRef>` extension, awaits an
authoritative read, reads policy afresh, and constructs the server-only identity
`urn:yetlet:yt125` / `account-one`, scheme `valkey`, tenant `one`. The private
resource independently checks current `can_view` and the identity/tenant binding.
There is no authorization cache or Memory fallback.

The policy authority is a trusted **single-host local regular JSON file**, e.g.
`{"enabled":true,"can_view":true}`. Each decision opens it afresh with
`O_NONBLOCK | O_NOFOLLOW`, checks the opened file type/size, reads at most 1025
bytes, accepts at most 1024 bytes, and applies a 250 ms timeout. Nonregular files,
symlinks, malformed/oversized data, unavailable files and timeouts return 503.
Tests replace it by atomic rename in the same directory. No HTTP route can edit
policy. This is not a network filesystem, multi-host policy service, cache
invalidation implementation, or production authority. A decision remains a
request-local snapshot; concurrent changes do not cancel work already admitted.

| Route | Selected behavior |
|---|---|
| `GET /health` | Public, outside Session and Auth; returns only A/B instance marker |
| `GET /auth/form` | Only this login page can explicitly create anonymous Session state; native no-JS form |
| `POST /auth/login` | Bounded form and Session CSRF; fixed synthetic identity; acknowledged `replace_and_rotate` before redirect |
| `GET /auth/private` | Required Auth plus independent fresh resource permission; native logout form |
| `GET /auth/optional` | Valid explicit anonymous record becomes `None`; mapped identity is signed in; all failures remain failures |
| `POST /auth/logout` | Required Auth and CSRF, acknowledged destroy; no manual cookie clearing |

Provider middleware wraps Auth and owns no-store, `Vary: Cookie`, HttpOnly,
SameSite and finalization. The body limit is 4096 bytes. All redirects are fixed
local 303 responses. Missing/expired/rotated/revoked records redirect to the login
form, including optional access; disabled/unmapped accounts or denied resource
permissions return 403. Malformed input/cookies are 400, oversized bodies 413,
failed CSRF 403, missing/misconfigured layers 500, CAS conflicts 409, and remote
or policy uncertainty 503. Outages never become anonymous or login navigation.

No script/WASM, credential, Principal, Session ID, or operation ticket is emitted
in logs or hydration. The only dynamic HTML field is a validated provider CSRF
token in a native form. Strict CSP disables scripts and limits form submission
to the same origin. HTTP assertions check CSP/HTML; no browser execution or TLS
cookie-enforcement qualification is claimed.

The explicit local-HTTP cookie override is fixture-only. Reads do not mint or
renew cookies; observed stale finalization suppresses Set, and logout emits no
Set/Clear. Previously prepared responses retain the provider's documented
[pure-SSR late-cookie limits](../../docs/YT-066B-COOKIE-ORDERING.md): an old
response may replace a newer browser cookie and force another login, but cannot
restore retired server authority. This test does not strengthen that guarantee.

## Running the bounded checks

Use the repository toolchain and cached dependencies:

```sh
cargo test --offline --locked --manifest-path qualification/yt125-auth-valkey/Cargo.toml
cargo clippy --offline --locked --manifest-path qualification/yt125-auth-valkey/Cargo.toml --all-targets -- -D warnings
cargo fmt --manifest-path qualification/yt125-auth-valkey/Cargo.toml -- --check
```

The default test command runs local unit checks and reports the live test as
ignored. Ignored is not live qualification success. The live test requires a
**dedicated disposable Valkey 9.1.2 instance owned by the caller**, with no other
users or data. `CLIENT PAUSE ALL/WRITE` affects that whole server. The test never
starts Docker, changes an unrelated service, or treats a missing endpoint as a
passing skip. Supply the caller-selected loopback port explicitly:

```sh
YT125_VALKEY_URL=redis://127.0.0.1:PORT/ cargo test --offline --locked \
  --manifest-path qualification/yt125-auth-valkey/Cargo.toml \
  --test multi_instance -- --ignored --exact two_process_auth_lifecycle_and_policy
```

The one sequential live test starts two binary OS children, uses actual reqwest
HTTP with proxies and redirect following disabled, and verifies anonymous and
required/optional behavior, CSRF/ID rotation, A-to-B continuity, rejected browser
identity assertions, resource permission changes, account disable, malformed or
missing policy, process restart, malformed/duplicate cookies, B logout, read
outage/recovery, and uncertain login/logout results. Finite 750 ms write pauses
allow dispatched mutations to time out before later committing; the response
must remain 503 without a cookie, and the retired old authority must fail after
recovery. Mutations are never blindly retried or reconciled into signed-in state.
Finite pauses expire on failure. No historical provider/browser/K3s suite runs.

The executable reads only `YT125_VALKEY_URL`, `YT125_NAMESPACE`,
`YT125_POLICY_FILE`, `YT125_INSTANCE` (A/B), and `YT125_TTL_SECONDS` (1–1800).
The test selects a 60-second absolute lifetime. Startup accepts only the explicit
`redis://127.0.0.1:PORT/` endpoint form. It prints a safe ready/address/instance
line, then on SIGTERM drains HTTP serving and awaits `manager.shutdown()` before
printing `STOPPED drained`. The test verifies success exit/drain within five
seconds and kills/reaps owned children on panic or timeout. Temporary policy and
ready-output files are removed by their temporary-directory owner. The caller
must remove the dedicated Valkey container, including this isolated namespace.

## Limits and evidence boundary

This slice qualifies identity/lifecycle and fresh policy-decision behavior only.
It has no business-data mutation endpoint, cross-store fencing, atomic business
commit/revocation ordering, long-job revalidation, real OIDC callback, HA,
persistence/restore, physical-host failure, or production authentication claim.
It makes no claim that a previously admitted read or future business write cannot
complete after logout. The policy file and synthetic identity are explicit
qualification fixtures, not a recommended production identity system.

Actual execution and independent-review evidence belong in the parent YT-125
verification record. Source presence or a passing compile does not establish
live qualification. Historical YT-066/094/103 evidence retains its original scope.
