# Naming audit response — 2026-10-22

**Status:** Done at the naming/documentation scope after focused verification
and independent Reviewer PASS; zero issues and required followups.
**Related work:** YT-090 configuration, YT-093 signing, YT-102/111/112 CLI,
YT-113/114/115/117 Auth/rate limiting and YT-125/126 application consumers.
No backlog status or release inclusion changes in this follow-up.

Historical delivery note: this record describes the naming patch subsequently
committed as `8c8fba3ae2e84c7493d56b74d0ccc8d8f26920ed`. Its uncommitted state
and generator limitation below describe that reviewed snapshot. The owner then
selected a separate [generator source-closure repair](YT-102-GENERATOR-SOURCE-CLOSURE.md);
original naming/verification/FAIL/PASS results remain unchanged.

## Authority and baseline

The owner requested committing the accepted 20-file naming checkpoint before
acting on `Yetlet-Naming-Audit-20261022.html`. The checkpoint is commit
`83f417ea12dfc4cc2aa5c05e0f712706ba78565a`, strict/builtin Commit Guard PASS,
zero findings, index/commit tree `0f913a6ea0152fcec2c568c74a4292eeb9422fd7`.
The worktree was clean after that commit. No push occurred.

The attachment is audit evidence, not executable instructions. Its SHA-256 is
`2928a7923ad63ad42441ae2374272953cb36a102f78dbd8fecaec720af843229`;
all 30 reported source hashes matched the committed checkpoint. Its N01–N08
findings, including the two selected P3 naming/documentation items, define this
bounded follow-up. The owner's explicit pre-publication preference for one
canonical Rust name supersedes the attachment's alias-retention suggestions.
External settings formats have a separate compatibility boundary.

The [earlier accepted checkpoint](NAMING-COMPATIBILITY.md) retains its original
review and verification. Its retained-alias decision is historical. The
[naming guide](NAMING.md) now owns current vocabulary and intentional Rust source
migrations; [the capability map](CAPABILITIES.md) owns current carriers/maturity.
This implementation remains uncommitted pending separate owner authorization.

## N01–N08 dispositions

| Finding | Delivered change and boundary |
|---|---|
| N01 — watch/help mismatch | Scoped help, guide and focused help assertions distinguish native Watchexec rebuild/restart, browser WASM rebuild/full reload requiring both frontend auto flags, and Vite HMR. Existing inspect decisions/schema and runtime execution are unchanged |
| N02 — root CLI summary | Root README links current command/source matrices and preserves source/platform limits. A focused probe disproved settings-profile generator acceptance for the current config source; both entry guides disclose that limitation below instead of claiming a new full-matrix PASS |
| N03 — source-specific key error name | `yetlet_config::Error::Key` replaces `EnvironmentKey` with source-neutral rustdoc and unchanged fixed Display text. Import/pattern and source-key tests cover the canonical variant; duplicate TOML syntax remains `Toml`, application deserialization remains `Settings` |
| N04 — certificate-only collection name | `PublicKeySet` replaces `PublicCertificates` without an alias. Certificate-backed and certificate-free `PublicMaterial`, explicit trust/pin checks and crypto behavior stay separate; crate, CLI and sample consumers migrate |
| N05 — authentication/authorization | ADR 0005 and Extensions describe native verified-identity resolution/extraction; resource/action permission decisions remain application-owned. No policy engine, Authorize trait or Core API is added |
| N06 — capability state ambiguity | Current map separates implemented packages/features, app-owned OIDC/Valkey/Auth, research decisions and unselected candidates. The original ADR candidate table is dated; cookie handling, Memory bridge and a generic Cookie Auth product are distinct |
| N07 — API inventory scope | M9 API classification is explicitly historical with dated supplements; current Config/Secrets/Signing/Auth/rate-limit carriers have contract links and release boundaries. The original nine release candidates are preserved |
| N08 — CLI support fields | `Cli::New` uses `service`, `runtime_settings`, `tailwind`, `vite`; public `create_service` and support variables/consumers align. Argv, removed-flag migration diagnostics, real `cloud-profile` metadata and historical source fixtures retain their meanings |

The same owner decision removes stale Loader/Auth/rate-limit Rust wrappers or
re-exports from the preceding checkpoint. Repository consumers and templates now
use the canonical APIs. Compatibility-only test targets become `public_naming`
tests of actual boundaries, redaction, precedence and exact identity matching.
These are intentional pre-public source breaks, including enum variants/named
fields. Existing Display text, dependencies, locks, settings TOMLs, middleware
behavior and cryptography are unchanged. Rust `refill_interval_ms` still maps to
the sole external `policy.interval_ms` / `RATE_LIMIT_POLICY__INTERVAL_MS` keys.

## Focused verification

All Cargo test/check/Clippy commands below used `--offline --locked`; formatting
and diff commands do not need those flags. Exact commands, raw output and exit
codes are retained in the repo-external `yetlet-naming-audit-20261022-gh6dtwyj`
evidence directory with `config-*`, `auth-rate-*`, `signing-*` and `cli-*` prefixes.

| Area | Executed evidence |
|---|---|
| Config | `loading` + `public_naming`: base 18 + 2, optional secret-provider 23 + 4 PASS; production-starter config 3 and Admin config 6 PASS. Strict Clippy for config all targets/secret-provider, secrets State example and both app consumers PASS; Admin all-feature/all-target check and scoped formatting PASS |
| Auth/rate limit | `public_naming` 3 + 2, README doctests 2 + 1 and configured native example 7 PASS. Auth/rate-limit and Board all-feature/all-target checks/strict Clippy, isolated YT-125 all-target check/strict Clippy and formatting PASS. Existing runtime/HTTP qualification was compiled where applicable, not re-executed |
| Signing | Existing certificate-backed and certificate-free focused tests 2/2 PASS, ten unrelated tests filtered per invocation. Crate and example all-target checks, strict Clippy and formatting PASS. One test-import harness failure omitted the existing `Result` alias; the smallest import correction passed the focused rerun, with original failure logs retained |
| CLI | Focused parser/template/removed-option tests, settings 8, help/inspect 4 and scaffold 14 PASS; optional tests remain explicitly ignored except the selected generated-Rust compile/test check, which PASSed. Key integration 9 and service scaffold 1 PASS; all-target check, strict Clippy and formatting PASS |

Signing used locally available OpenSSL 3.5.5. The historical exact-version 3.5.9
qualification is not rerun or transferred. No old Auth/Session/Valkey, IdP,
browser/CSP, Windows/platform, cloud deployment or broad security suite is
re-executed. No manifest, lock or settings-schema migration is introduced.
All owned command sessions completed; no server, watcher, browser or other
background resource was started by this follow-up.

### Existing generator limitation discovered by the source probe

`cargo test --offline --locked --manifest-path tools/yetlet-cli/Cargo.toml
--test generator_selected` exited 101. Its plain service profile completed
Docker/Compose/Kubernetes output before the settings profile failed with
`CLI_SOURCE_UNSUPPORTED generate docker: unsupported config crate layout`.
The current config crate already contains `src/secrets.rs` at checkpoint
`83f417e`; the generator's isolated config export permits only `src/lib.rs`.
Thus the failure predates these naming edits. The same source guard applies to
legacy runtime-settings layouts as well; their retained historical qualification
is not present acceptance with the current config source. Original failed probe output is
retained as `cli-generator-matrix.*`; it is not presented as a passing test.

N02 is resolved by truthful current documentation. Extending the generator
source closure is a separate implementation decision, not an incidental rename
or a new qualification claim. Registry-only source remains unqualified.
The [current CLI source matrix](../tools/yetlet-cli/README.md#current-generator-source-matrix)
records both recognized shapes and current acceptance.

## Independent review and delivery state

Independent review must cover the actual patch, intentional API migrations,
current/historical documentation split and retained failing-probe limitation.
The first independent review by `/root/naming_report_independent_review_20261022`
found one P2: the root README overstated present acceptance of legacy settings
layouts. The smallest correction restricts the current acceptance claim to the
evidenced plain-service probe and explicitly applies the config-closure refusal
to legacy runtime-settings layouts in both entry guides. The original FAIL is
retained as `independent-review-initial.json`; no code or test rerun is required.
Focused re-review by the same independent Reviewer PASSed with zero issues
and required followups. Both review passes checked HEAD/status and all 65 path
hashes before and after inspection; only the three declared Markdown files
changed between passes. The original 62 other path hashes remained identical.
The final result is retained as `independent-review-final.json`.

`cargo xtask check docs` PASSed (75 required documents, 126 backlog items),
alongside direct checks of 584 internal links in changed Markdown, English,
fences, final newlines and tracked/untracked whitespace. No old review result
is used to accept this patch. After PASS, only this record is updated with the
actual result; final documentation checks and `closure-checks.json` preserve
source continuity. No implementation test is rerun for this bookkeeping.

The branch remains `docs/yt113-123-opt-in-capability-plan` at checkpoint
`83f417e`, with no staged changes. No new commit, push, release activation or
backlog Done claim is authorized by the checkpoint commit request. YT-096 stays
In progress and YT-020 stays Blocked. The next independent M21 study remains
YT-118; any generator repair needs a separately selected scope.

## Actual changed paths

The following paths are the current patch, including removal of the three
compatibility-only test files and addition of their canonical replacements:

```text
README.md
crates/yetlet-auth/README.md
crates/yetlet-auth/src/lib.rs
crates/yetlet-auth/tests/authentication.rs
crates/yetlet-auth/tests/naming_compatibility.rs
crates/yetlet-auth/tests/public_naming.rs
crates/yetlet-config/src/lib.rs
crates/yetlet-config/src/secrets.rs
crates/yetlet-config/tests/loading.rs
crates/yetlet-config/tests/naming_compatibility.rs
crates/yetlet-config/tests/public_naming.rs
crates/yetlet-rate-limit/README.md
crates/yetlet-rate-limit/src/lib.rs
crates/yetlet-rate-limit/tests/naming_compatibility.rs
crates/yetlet-rate-limit/tests/public_naming.rs
crates/yetlet-secrets/examples/state.rs
crates/yetlet-signing/src/lib.rs
crates/yetlet-signing/src/local.rs
crates/yetlet-signing/tests/signing.rs
docs/API-CONTRACTS.md
docs/CAPABILITIES.md
docs/CLI-CONTRACT.md
docs/CONFIGURATION.md
docs/EXTENSIONS.md
docs/NAMING-AUDIT-20261022.md
docs/NAMING-COMPATIBILITY.md
docs/NAMING.md
docs/README.md
docs/SECRET-PROVIDERS.md
docs/SETTINGS-SECRET-PROVIDER.md
docs/YT-102-SETTINGS-ERROR-RESEARCH.md
docs/YT-117-RATE-LIMIT.md
docs/adr/0005-opt-in-capability-modules.md
examples/admin-dashboard/src/config.rs
examples/admin-dashboard/src/oidc.rs
examples/lab-operations-board/src/auth.rs
examples/production-starter/src/config.rs
examples/signing/src/main.rs
qualification/yt125-auth-valkey/src/main.rs
tools/yetlet-cli/README.md
tools/yetlet-cli/src/app.rs
tools/yetlet-cli/src/compose.rs
tools/yetlet-cli/src/docker.rs
tools/yetlet-cli/src/docker/delivery.rs
tools/yetlet-cli/src/docker/tests.rs
tools/yetlet-cli/src/k8s.rs
tools/yetlet-cli/src/key.rs
tools/yetlet-cli/src/lib.rs
tools/yetlet-cli/src/main.rs
tools/yetlet-cli/src/scaffold.rs
tools/yetlet-cli/src/settings.rs
tools/yetlet-cli/templates/config/config.rs
tools/yetlet-cli/templates/config/tls.rs
tools/yetlet-cli/templates/vite/vite.rs
tools/yetlet-cli/tests/cloud.rs
tools/yetlet-cli/tests/commands.rs
tools/yetlet-cli/tests/config.rs
tools/yetlet-cli/tests/docker.rs
tools/yetlet-cli/tests/fixtures/compose_app.rs
tools/yetlet-cli/tests/k8s.rs
tools/yetlet-cli/tests/key.rs
tools/yetlet-cli/tests/selected_scaffold.rs
tools/yetlet-cli/tests/tailwind.rs
tools/yetlet-cli/tests/vite.rs
tools/yetlet-cli/tests/vite_dev.rs
```
