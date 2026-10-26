# YT-181 — Opt-in commit-time write authorization reference

Date: 2026-10-25. Status: **Done at bounded opt-in application-reference scope after independent
implementation Reviewer PASS, zero issues and required followups.**

## Authority and accepted baseline

The maintainer authorizes the smallest opt-in implementation of the
[accepted contract](YT-181-SENSITIVE-WRITE-CONTRACT.md), retaining middleware,
existing limits/security and no generic AOP; no old passed verification replay,
independent Reviewer before Done and no new-change Commit/Push. The five
contract documents are committed as `0bf46e7`, strict/builtin Guard and hook
PASS, zero findings, tree `886591955db690015c491cc7b8e0339b3d939345`; no Push.
Contract review and older YT-126/137/138/139 evidence retain their exact scopes.

## Application-only implementation

- `examples/lab-operations-board/Cargo.toml` adds default-off `commit-write-auth`.
  Admin explicitly forwards it; no dependencies or lockfile change. The Board
  constructors selecting the mode are application APIs, not framework/Auth APIs.
- A fresh selected Store creates the existing item/history schema plus the
  private `0002_write_authorization.sql` table, marker v2 and exactly one initial
  Alice/project-one allow. Existing v1/empty files refuse this mode; normal opens
  still use v1 and refuse v2. No automatic migration or existing-data rewrite.
- `database/write_authorization.rs` validates actual table shape/strictness and
  bounded row domain, positive generation and disabled terminal maximum. Missing
  logical grant denies; malformed/unavailable authority fails closed. Readiness
  validates selected authority before its normal business query.
- The server-only non-Clone/non-Debug ticket binds account/project/generation to
  an `Arc` Store brand. The private issuer is parent-module-visible only; tickets
  are neither serialized nor accepted from a request. Each `save` automatically
  acquires its ticket after existing input/Principal validation and checks it in
  a fresh IMMEDIATE transaction before the original domain/item/history effect.
  A missing ticket cannot bypass an enabled Store. Native create/update and
  triage continue calling this single primitive; middleware/Auth/CSRF are retained.
- `Board::set_project_write_permission` is trusted native application control,
  not a public endpoint. It accepts only the selected local account/project.
  Disable/enable transactionally changes the SQL generation; no-op deny confirms
  committed state and terminal generation cannot be re-enabled. Normal request
  policy remains an intersecting condition, so SQL allow cannot broaden it.
- Admin's existing typed Settings adds optional `write_authorization.enabled`,
  default false, and its sole environment spelling
  `APP_WRITE_AUTHORIZATION__ENABLED`. A true setting without the feature is
  rejected before application startup; selected constructors preserve existing
  OIDC/frontend/Session inputs. No ambient library environment lookup, provider
  registry, hot reload or new handler/control route.

### Conservative reopen/recovery choice

A v2 reopen, including another instance opening the same file, first commits
write-disabled and advances an enabled generation using the same IMMEDIATE
ordering. Already-denied rows/generations remain unchanged. This is a monotonic
revocation, **not a reset/reseed or automatic old-allow promotion**. Fresh initial
bootstrap is the only implicit allow and requires a newly created file.

This conservative implementation choice satisfies the contract's no automatic
restored/rolled-back allow condition without inventing an external rollback
oracle. All old instances/workers must be excluded for restore; normal reopen
also denies conservatively. Trusted application code must reconcile current
permission/recovery authority before explicit re-enable. A process/Store brand
cannot be reused across reopen; no online replacement/mixed-version migration
is supported. This choice sacrifices automatic write availability at startup;
it is not production restore, cross-host SQLite or HA qualification. Other
instances' old tickets invalidate when an opener's deny commits.

## New verification only

Accepted previous benchmarks, browser/CSP, provider/IdP/Valkey, Windows, old
Board/Admin cases and previous contract checks are not executed again. Every
Cargo test uses `--offline --locked`; only the new name filter runs. Tests use
owned temporary SQLite files and direct in-process routers, not live services.

| New command/scope | Result |
| --- | --- |
| `cargo test --offline --locked -p yetlet-lab-operations-board --features commit-write-auth --lib commit_write_ -- --test-threads=1` | **6 passed**, 0 failed; 20 old tests filtered |
| Admin binary test filter `commit_write_auth_selection`, feature `commit-write-auth` | **1 passed**, 19 old tests filtered |
| Same new Admin selection case, feature absent | **1 passed**, 19 old tests filtered; different compile/setting condition |

There are seven new test definitions/eight executions. The Board cases establish:
normal item/history atomicity and history-failure rollback; no missing-ticket
bypass; old ticket issued before pause/revoke/re-enable rejecting afterward;
fresh allow success and cross-Store ticket rejection; held real write transaction
using the production check/effect helpers preventing premature revoke success;
wrong account/project, missing/corrupt authority and terminal generation denial;
reopen refusing implicit allow/reseed, old ticket/legacy/empty-file refusal;
and actual native/triage composition sharing the SQL deny while preserving
private/CSP/no-cookie failure and item/history versions. No same-mutex-only
claim substitutes for the independent connection tests.

The held writer test receives Busy from the real second-connection revoke;
only after that earlier effect commits is a new explicit control action issued.
This is not a production retry. The paused test uses a private ticket and a
oneshot test barrier before entering the effect transaction; it does not add a
runtime hook, observer, generic harness or benchmark.

The first six-case execution reports a `private_interfaces` warning: the ticket
issuer was wider than its private ticket. It is narrowed to parent-module
visibility, without changing behavior; that accepted six-case execution is not
replayed. Later feature-on compilation/lint verifies the final visibility.
No failed test or runtime result is discarded; receipts retain original output.

Strict Clippy **PASS** for Board (feature plus tests compiled) and Admin
(`commit-write-auth,auth-valkey` composed); neither command runs tests. Admin
feature-off/on configuration executions also compile the relevant Board/OIDC
constructors. New-document/format/whitespace results are recorded in the local
evidence directory. Compile-only checks do not imply
live Valkey/IdP/browser/platform or released-artifact qualification.

## Limits and acceptance boundary

Only the new SQL-owned write permission gets commit-time ordering. Existing
one-Session logout, account disable/file-policy replacement and IdP lifecycle
remain at their documented current freshness/race scope. No automatic Session
revoke-to-SQL synchronization, distributed lock, generic Auth/AOP or new Core
capability is introduced. Four workers, one-second worker admission, 250ms SQLite
busy wait, item/history limits, cancellation ownership and drain remain unchanged.
The original business-effect SQL is factored into a private helper; no alternate
handler path, retry, timestamp-based fence or UI-supplied generation is used.

Outcome uncertainty and current-authority reconciliation still follow YT-137.
No exactly-once, idempotency-key, write throughput, power-loss durability, public
endpoint or HA promise follows. Dedicated v2 fixture/direct SQL owner access is
a trusted application boundary, not a defense against an arbitrary file owner.
This implementation has no public operational control UI; trusted integration
code owns explicit re-enable after its current-authority/recovery decision.

All test futures/workers finish and drain; temporary directories/files are owned
by their tests and removed by their lifetime. No Docker, Azure, browser or
external server is started. Independent review of source/diff/new proof and
these limits is accepted below before YT-181 Done. YT-182–184 remain Todo;
YT-096 D1–D6 Pending and YT-020 Blocked remain unchanged. All new changes stay
unstaged/uncommitted/unpushed.


## Independent implementation acceptance

Reviewer `/root/yt181_commit_write_auth_implementation_review_20261025`:
**PASS**, zero issues/required followups. All 19 changed-file, 41 evidence and
12 retained bindings match before/after review; native reviewer role/model/high
effort match the saved configuration/resolver. Review inspects actual source,
diff, cases/receipts and scope without executing tests, builds, services or old
qualification. It accepts the conservative reopen deny as consistent with the
no-reset/no-reseed/no-automatic-restored-allow contract.

Mark YT-181 **Done only at this optional application-reference scope**. Seven
new definitions/eight case executions, both strict Clippy compositions, format
and new-document/whitespace checks pass. The accepted six-case result predates
only the issuer visibility narrowing, subsequently compiled/linted; no passing
case is replayed. Runtime/config/SQL/test bytes stay frozen after review; only
six documentation status/review references are updated, followed by new document
checks. Local `closeout.json` lists the actual 19 paths, final hashes, receipts,
unchanged inputs and Git state. New changes remain unstaged/uncommitted/unpushed.
No YT-182–184 implementation, broader Session/file revoke guarantee, production
claim, old benchmark replay or release activation follows automatically.
