# YT-185 — Application opt-in startup options

Date: 2026-10-26 (Asia/Taipei). Status: **Done at application integration scope
after independent Reviewer PASS, zero issues/required followups.**

## Requirement and scope

The maintainer explicitly requests consistent YT-181–184 opt-in integration,
typed options, separate compile-time and startup selection, optional Settings
loading, preserved security semantics and independent Reviewer acceptance before
Done. No Commit/Push is authorized. This is one application integration cleanup,
not a new Core API, common capability registry, AOP, crate or configuration service.
[ADR 0005](adr/0005-opt-in-capability-modules.md),
[ADR 0006](adr/0006-handler-first-mvc-optional.md) and the
[naming guide](NAMING.md) remain authoritative.

The initial checkout contains 17 accepted, unstaged YT-184 paths at HEAD
`23cdb869f4fe167d0585045ccb293c6fe0a1da71`. Preserve those bytes except shared
Backlog/Roadmap/index/capability navigation additions. Preserve their retained
review, cloud, failure and cleanup records. No Azure/provider/benchmark replay.

## Selected contract

1. Cargo features determine which native capabilities and optional dependencies
   are compiled. Feature names/dependency graphs stay unchanged. Compiling a
   feature does not select an application instance's runtime behavior.
2. Application startup loads settings through its chosen source, builds a plain
   typed options value, then explicitly constructs that instance. Library
   constructors never read ambient environment or load settings implicitly.
   Admin keeps its existing opt-in `yetlet_config::Loader` integration; Shop
   keeps native environment loading. Admin already chooses this loader as an
   application dependency; there is no new `settings` Cargo feature or required
   loader for other hosts. No mandatory Settings provider is added.
3. Public `BoardOptions` owns frontend paths, Session backend, and independent
   `write_authorization` / `request_tracing` booleans. `Default` keeps the existing
   embedded frontend, Memory Sessions and both YT-18x capabilities disabled.
   `Board::open_with_options` and `Board::open_with_oidc_and_options` consume it
   for demo/OIDC identity respectively. Existing pre-18x frontend/Session
   convenience constructors remain default-off and delegate to this path;
   they have real existing consumers, rather than new capability combinations.
4. Public Shop `DatabaseOptions` contains `catalog_cache: bool`, default false;
   `Database::open_with_options` consumes it. `Database::open` delegates using
   defaults. Plain options fields exist in both feature-on/off builds so hosts
   can map configuration consistently; requesting an uncompiled capability
   returns a fixed configuration error before creating/opening the database.
   No silent downgrade. Board uses existing `Failure::Configuration`; Shop
   uses existing `DatabaseError::Io` with a fixed InvalidInput message.
5. Board/Admin map the two existing enabled settings independently into the
   same Board options for demo and OIDC. Tracing never accepts a positional
   Write Auth boolean or implicitly selects it. The host still owns subscriber
   initialization; a Board constructor installs no global subscriber.
6. Existing external keys, exact true/false parsing, precedence and redacted
   diagnostics stay unchanged. Runtime selection is startup-only. Feature
   availability validation remains both in the existing app loader and at the
   options constructor for embedded callers. Options do not reconfigure a live
   instance or establish trust/permissions merely by being constructed.
7. YT-184's native provider TLS is transport selection through existing `tls`
   and `rediss://`, and `YT184_TRANSPORT_CONFIG_FILE` is private experiment
   fixture input. They are different scopes from app instance switches and
   remain unchanged. No generic options layer, TLS toggle, feature rename or
   mandatory dependency is introduced to disguise that difference.

## Intentional pre-public Rust migration

| Removed application constructor | Canonical replacement |
|---|---|
| `Board::open_with_commit_write_auth` | `Board::open_with_options` with `BoardOptions.write_authorization = true` |
| `Board::open_with_oidc_and_commit_write_auth` | `Board::open_with_oidc_and_options` with the same field |
| `Board::open_with_request_tracing` | `Board::open_with_options` with `BoardOptions.request_tracing = true` |
| `Board::open_with_oidc_and_request_tracing` | `Board::open_with_oidc_and_options` with the same field |
| `Database::open_with_catalog_cache` | `Database::open_with_options` with `DatabaseOptions.catalog_cache = true` |

Migrate current examples and test consumers in the same change; retain no stale
aliases. The earlier YT-181/182/183 contracts and verification are dated evidence
of those former entry points, not proof of the new ones. Current sample guides
and this migration table own current usage. Existing simpler default-off
constructors remain useful and unchanged; no blanket source-compatibility claim.

## Preserved invariants and minimum proof

Preserve YT-181's fresh-v2 selection and SQL transaction/generation checks,
conservative reopen deny, ordinary identity/policy/CSRF checks; YT-182's trusted
correlation, allowlisted fields, scoped worker context, response/drop distinction
and subscriber ownership; YT-183's bounded display freshness/generation invalidation,
uncached checkout/Session/authorization; and all YT-184 transport/recovery limits.
Do not modify worker/admission limits, retry/queue policies, output/security,
API handlers, schemas, external keys, locks or provider code.

Use new focused product tests for options defaults, every Board capability
combination (available selection or pre-storage refusal), actual selected schema
and response correlation, Admin settings-to-options mapping and Shop cache
selection/refusal. Compile migrated existing test consumers without running
accepted old tests. Necessary changed-source format/lint/docs and default normal
dependency inspection are in scope; no new harness framework or full qualification.
Record native commands/results and failures under
`target/yt185-opt-in-20261026/evidence/`; baseline hashes bind preserved YT-184 work.
Only final independent Reviewer PASS permits YT-185 Done. YT-096/D1–D6 and
YT-020 release decisions remain unchanged.


## Minimum implementation and retained evidence

The application-owned options now select the existing initialization paths.
Board/Admin share one options mapping for demo and OIDC; Shop maps its unchanged
native environment parser into `DatabaseOptions`. Five capability-specific
constructors are removed and all current Rust call sites migrated. Existing
pre-18x default-off helpers are kept for their unchanged consumers. No manifest,
lock, provider, handler, authority, cache, tracing or database-worker logic changes.

Four new focused test functions have **10 successful executions**:

| Surface | Actual commands and result |
|---|---|
| Board typed options | `cargo test --offline --locked -p yetlet-lab-operations-board --features <selection> --lib opt_in_options -- --nocapture` with `oidc`, `oidc,commit-write-auth`, `oidc,request-tracing`, `oidc,request-tracing,commit-write-auth`: 1 pass each, 26/32/31/37 older cases filtered |
| Admin Settings mapping | In `examples/admin-dashboard`: `cargo test --offline --locked [--features commit-write-auth,request-tracing] opt_in_options -- --nocapture`: 1 pass each for default/both; 21 older unit cases filtered in each build; integration targets selected zero tests |
| Shop options and native startup | `cargo test --offline --locked -p yetlet-modular-shop [--features catalog-cache] opt_in_options -- --nocapture`: 1 library and 1 binary pass each; 9/16 older library and 1 older binary cases filtered |

The Board test checks all four runtime selections in each compiled configuration:
unsupported selections return Configuration without a database file; supported
selections produce the selected v1/v2 schema and correlation header independently,
with existing CSP, Session cookie and no-store behavior. Admin exercises the two
existing keys independently and checks frontend/Session options. Shop verifies
actual display-cache selection versus uncached results and fixed pre-file refusal.

Both changed-source strict Clippy commands pass with `--offline --locked
--all-targets --all-features -- -D warnings` (root packages Board/Shop; standalone
Admin manifest). This compiles the migrated old test consumers without replaying
them. Scoped Cargo formatting passes. Normal default dependency trees for Board
and Shop contain none of Moka, OpenID Connect, optional request-ID, Valkey Session
or Settings loader packages. No dependency/feature declaration is modified here.

Original failures are retained in the evidence directory: one in-place migration
script stopped on a mismatched tracing fixture signature before tests; its
remaining call-site edits were corrected. The new trace assertion initially used
`x-request-id` instead of the unchanged provider's `x-yetlet-request-id`; only that
failed configuration was rerun. Strict lint rejected production items after
Admin's test module and a constant-only Shop test assertion; the minimum ordering
and equivalent assertion fixes passed the failed lint checks. Passing runtime
proof was not replayed for those non-runtime changes. Exact commands/native exits
are in each evidence JSON/log; original failure files were not overwritten.

The original 13 non-navigation YT-184 paths remain byte-identical, including
provider tests/runtime, both standalone locks/manifests, the Azure fixture and its
accepted report. Four shared navigation files receive only this item's additions.
No browser/CSP qualification suite, provider/server, Azure, benchmark, Windows,
security scan, signing or publishing was executed. Existing authority/tracing/cache
proof remains historical evidence, not a new qualification of every combination.
Independent Reviewer `/root/yt185_opt_in_options_independent_review_20261026`
returns PASS, zero issues/required followups. Native role/model/effort match the
saved profile and resolver; 19 source, 45 evidence and 13 preserved YT-184 file
bindings match before/after review, as do HEAD/branch/status (staged zero).
All four shared navigation files recover their exact prior hashes when only
YT-185 additions are removed. Only completion/reference bookkeeping follows.

Changed files for YT-185: Board `README.md`, `src/lib.rs`, `src/tests.rs`,
`src/tests/request_trace_tests.rs`; Admin `README.md`, `src/main.rs`; Shop
`README.md`, `src/lib.rs`, `src/database.rs`, `src/main.rs`, `src/tests.rs`,
`src/catalog_cache/tests.rs`, `src/tests/catalog_cache_tests.rs`; documentation
`BACKLOG.md`, `ROADMAP.md`, `README.md`, `CAPABILITIES.md`, `NAMING.md` and this
contract. These are 19 affected paths, including four shared YT-184 navigation
files; the combined checkout has 32 paths, no staged files or new commit.
No new unblocked implementation is selected. YT-096 remains In progress with
D1–D6 Pending, and YT-020 remains Blocked.
