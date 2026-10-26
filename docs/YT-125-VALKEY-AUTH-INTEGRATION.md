# YT-125 — Minimal application-owned Valkey/Auth integration

**Date:** 2026-10-22. **Status:** Done at the selected local application-integration scope after independent Reviewer PASS.
**Source baseline:** `f2109514fba7397b6f3f264407d90fb78b100ede` plus the independently
reviewed, uncommitted [YT-124 contract](YT-124-MULTI-INSTANCE-AUTH-SESSION-CONTRACT.md).

## Selected implementation and authority

After YT-124 research passed independent review, the owner requested continuation
to its minimum implementation candidate. This selects one application-owned
async resolver and focused local qualification. Commit/push remains unselected.
The previous six documentation changes stay in the current checkout.

The consumer is a standalone, unpublished native qualification application in
`qualification/yt125-auth-valkey`, outside the default workspace and runnable
sample catalog. It composes existing `yetlet-auth` with the existing provider
aliased as `yetlet-session-valkey` (package `yetlet-session-redis`). It selects
no generic Auth helper, new capability library, framework feature, provider registry or Core
API. Memory remains synchronous and unchanged. This slice does not migrate
the Admin/Board application's existing identity model.

## Concrete local authority and topology

- Two actual application processes bind ephemeral IPv4 loopback listeners and
  share one isolated namespace in the selected Valkey service.
- The local server uses the YT-094 pinned Valkey 9.1.2 image,
  `valkey/valkey@sha256:418652cfb58ef879d4978c33553735d7147016032d5aefaa14c828e611eb9dfd`.
  Only a new task-owned, loopback-published, non-persistent container is selected.
- Session state carries a server-owned local account reference, not serialized
  generic Principal or trusted permission fields. The synthetic preverified
  fixture identity is `account-one` in issuer `urn:yetlet:yt125`, scheme `valkey`
  and tenant `one`; this is not a deployable credential-verification service.
- Both processes read one task-owned bounded local JSON permission file for
  current account enablement and resource-view permission. Test/operator updates
  atomically replace that file; there is no process cache or public policy-edit
  endpoint. Malformed, missing or unavailable authority must fail closed.
- This is a single-host fixture authority, not a production or distributed
  authorization datastore. It qualifies subsequent permission decisions, not
  instantaneous cancellation of an already-running response.

The slice exposes native login/form/logout and required/optional identity plus
resource-read paths, with stateless health outside the capability scope. It
includes no business-data mutation. Cross-store transactional fencing and strict
write-after-revocation behavior remain outside this qualification and require a
later explicit application decision, as YT-124 requires.

## Required behavior for this slice

The application resolver uses the selected typed async Session handle and
authoritative read, then its current local authority lookup before constructing
the native Principal. Account identity and resource permission remain distinct.
Only an explicitly valid anonymous remote record becomes optional None; absent
or retired remote state remains the selected failure response even for optional
extraction. A provider or authority outage is 503, never anonymous or login
navigation. A disabled account or denied resource is 403.

Login/logout require native form CSRF; login replaces and rotates Session/CSRF,
logout acknowledges authoritative revocation. Every subsequent lookup on either
process must reject the retired identity. Preserve the provider's pure-SSR cookie
semantics: no logout Set/Clear, no ordinary-read renewal, and no claim that a
prepared old response can never cause login loss. Unknown mutation results must
not acknowledge success, emit a success cookie, or trigger blind callback replay.
Application shutdown drains the existing manager's owned work.

## Verification boundary

Only new consumer checks will run: compile/lint/format, one focused opt-in
two-process actual-HTTP lifecycle/policy/failure test against the task-owned
Valkey, baseline dependency comparison and documentation checks. The test must
actually execute; a filtered or ignored test is not a PASS. Children and the
container require bounded owned cleanup on success/failure.

Existing [YT-066b3](YT-066B3-VERIFICATION.md), [YT-094 Valkey/TTL](YT-094-VALKEY-SESSIONS.md),
[shared SQLite](YT-094-SHARED-SQLITE.md), [YT-103 rolling assets](YT-103-ROLLING-ASSET-VERIFICATION.md)
and YT-114/115 results retain their original source/topology limits and are not
rerun. No browser, real IdP, K3s, Windows, HA, persistence/restore, production
TLS/ACL, benchmark or full security rescan is implied by this local slice.

Actual results and independent Reviewer PASS are recorded below. The earlier Auth security scan does not certify this newly added
consumer.

## Implementation and actual focused evidence

Source is [the standalone application](../qualification/yt125-auth-valkey/src/main.rs),
[its direct product tests](../qualification/yt125-auth-valkey/tests/multi_instance.rs),
[manifest](../qualification/yt125-auth-valkey/Cargo.toml), matching standalone
lockfile and [usage/limits](../qualification/yt125-auth-valkey/README.md).
The strict DTO requires an explicit anonymous/mapped/unmapped enum reference;
missing fields, null, unsupported values and permission-object injection fail
decoding. The two service-free unit tests cover that shape and explicit
missing-layer/provider-error mapping. Source inspection and the actual HTTP
exercise establish the application-owned resolver; generic framework/provider
sources remain unchanged.

| Command | Actual result |
|---|---|
| `cargo generate-lockfile --offline --manifest-path qualification/yt125-auth-valkey/Cargo.toml` | PASS, standalone lock only |
| `cargo test --offline --locked --manifest-path qualification/yt125-auth-valkey/Cargo.toml` | Verified executor reports two unit tests PASS; live test ignored at this compile/unit stage, not counted as live proof |
| `cargo clippy --offline --locked --manifest-path qualification/yt125-auth-valkey/Cargo.toml --all-targets -- -D warnings` | Verified executor PASS |
| `cargo fmt --manifest-path qualification/yt125-auth-valkey/Cargo.toml -- --check` | Verified executor PASS |
| `cargo test --offline --locked --manifest-path qualification/yt125-auth-valkey/Cargo.toml --test multi_instance -- --ignored --exact two_process_auth_lifecycle_and_policy` | Parent actual live execution PASS: 1 passed, 0 failed, 0 ignored, 0 filtered; test 4.64 seconds, complete command 5.55 seconds |
| Four existing `cargo tree --locked --offline --edges normal -p PACKAGE` baselines, with `--target wasm32-unknown-unknown` for browser packages | Before/after byte-identical for Core, server adapter, Todo Browser and Search Island; metadata comparison, not repeated compile/browser qualification |
| Standalone `cargo tree --offline --locked --manifest-path qualification/yt125-auth-valkey/Cargo.toml --edges normal` | Auth plus existing remote provider selected explicitly; no Memory provider in the normal graph |

The executor role, `gpt-6-astra` model and effective high effort were verified
through the native trace and shared resolver before accepting its results.
Its raw unit/lint stdout was not separately retained; the verified child report
is the provenance for those results. Parent live stdout/stderr, topology/version,
command result, normal graphs and cleanup observations were retained separately.
No passing workload was replayed merely to create another log. The initial
large-native-Response lint diagnostic was resolved with a documented local
expectation matching the existing native error contract; no global lint was
weakened and no runtime defect was established by that diagnostic.

### Actual HTTP and policy observations

The one sequential live test starts A and B as distinct owned OS children,
restarts B while Valkey survives, and uses explicit Cookie headers with redirects
and proxies disabled. It exercises these newly integrated paths:

- Public health remains outside Session/Auth, without Set-Cookie or no-store.
- Missing credentials on protected required/optional routes fail explicitly;
  a valid anonymous Session is None only on optional access.
- Native login rejects invalid CSRF, account/permission/return-URL input injection
  and oversized bodies; successful login changes both identity and CSRF. The
  old anonymous identity cannot authenticate, and B accepts A's new Session.
- Both processes deny the private resource after `can_view=false`, while the
  identity remains authenticated. Account disable denies required and optional
  identity paths on both. Malformed/oversized/missing policy is 503; restoring
  the shared authority restores valid decisions.
- Restarted B accepts the same unexpired Session. Malformed/duplicate cookies
  are 400. B's acknowledged CSRF-protected logout produces no Set/Clear and both
  processes reject the old identity afterward.
- A finite 1300 ms `CLIENT PAUSE ALL` makes the concurrent protected requests
  on A/B return 503 without cookies or login navigation; health remains 200 and
  reads recover when the pause expires.
- Separate 750 ms `CLIENT PAUSE WRITE` cases make dispatched logout and login
  return 503 without success cookies. After recovery, the retired old identity
  is rejected on both; the test never replays those mutations and a deliberate
  fresh login can recover. This observes a failed acknowledgment with a later
  authoritative retirement, not a universal cancellation or retry guarantee.
- Scoped responses preserve no-store, Vary: Cookie, nosniff and strict CSP
  headers. Native form source contains no script. This is actual HTTP/HTML/header
  evidence, not browser rendering or CSP enforcement.

Pauses were issued only to the fresh task-owned disposable Valkey, expired
within their finite limits and did not touch shared services. The image was
absent locally and was pulled by its existing pinned digest; the running server
reported Valkey 9.1.2. The test uses a 60-second absolute Session lifetime, four
provider-admitted calls, 256-byte Session payloads, a 4096-byte HTTP form bound
and a 1024-byte policy bound. These are fixture choices, not wider production
capacity or timing claims.

### Cleanup, limits and review gate

The live test verified graceful SIGTERM success and the provider-drained marker
for A, original B and restarted B. Panic/drop cleanup also owns kill/reap of its
children; no new process-control framework is introduced. Parent observation
found zero fixture processes remaining. Valkey exited 0; the exact owned
container was removed, its loopback port was closed and no persistent volume
was created. Only the downloaded pinned image remains as local cache.

The selected single-host policy file, synthetic identity, native-only executable
and plaintext loopback transport remain explicit limits. No Admin migration,
real credential/IdP verification, business write, cross-store ordering, browser,
Windows, persistence, HA or multi-host guarantee is claimed.

Independent Reviewer `/root/yt125_valkey_auth_integration_review_20261022` returned
PASS with zero issues, required followups and optional notes. Native trace
verified reviewer role, `gpt-6-astra` and effective high effort. All 12 target
hashes and the four retained before/after baseline normal graphs matched through
review completion. Only status/review bookkeeping changed afterward; the five
consumer source/manifest/lock/README/test files stay exactly as reviewed.

`cargo xtask check docs` PASS (75 fixed required documents, 125 backlog items,
English corpus, links/Markdown/whitespace and dependency DAG). Changed/new-file
link, EOF/fence and whitespace checks PASS. No prior passing runtime workload
was rerun and no commit/push was performed.
