# Maintainer development workflow

Use this guide for authorized changes to Yetlet itself. Application authors
should start with [your first app](FIRST-APP.md). The
[contribution policy](../CONTRIBUTING.md) still excludes unsolicited external
code PRs; forks and bug reports remain welcome.

## Normal path

`bounded change -> focused checks -> independent review when required ->
authorized commit/push -> PR CI -> authorized merge -> source reconciliation`

The actual default branch is `master`. Use a short-lived `feature/*` or `codex/*`
branch and a PR into protected `master`. A `dev` integration branch is optional,
not a prerequisite or a setting enabled by this guide. If selected separately,
use PRs into `dev` and then a PR into `master`; neither branch replaces review.
Ordinary development does not run release jobs or access release secrets.

| Step | Do | Exit condition |
|---|---|---|
| 1. Bound the work | Read [AGENTS.md](../AGENTS.md), the item/ADR and relevant guide; inspect branch, HEAD, status and existing evidence | One clear change; unrelated work preserved |
| 2. Make the change | Work in the current checkout by default; keep opt-in/package/target boundaries | Behavior and public API changes match the accepted scope |
| 3. Verify the delta | Choose existing checks for changed behavior and affected consumers | Required checks pass, with exact command/source/feature/target recorded |
| 4. Review | Obtain independent Reviewer PASS where the item requires it; separate self-review from independent review | No unresolved material issue; original failures remain recorded |
| 5. Commit and deliver | Obtain Git authority, stage only the intended paths, run strict/builtin Guard, then normal hooks | Reviewed scope and index tree match the actual commit; outgoing range passes before Push |
| 6. PR and merge | Wait for required CI on the exact PR head; merge only under the owner's authority | Expected head checked; actual merged SHA/tree recorded |
| 7. Close the item | Update Backlog/Roadmap from actual evidence | Done only at the verified scope; release/support claims remain separate |

Do not reset, clean, overwrite another change or create a detached worktree to
make a delivery look clean. A separate branch/worktree or history reorganization
needs the authority appropriate to that operation.

## Local checks and CI

Keep new verification logs, original failures and accepted artifacts outside
rebuildable Cargo caches. Use the [artifact layout and read-only space preview](DEV-ARTIFACTS.md)
for isolated runs and WebCodex runners; existing mixed historical `target/` trees
must not be cleaned wholesale. Ordinary daily builds reuse stable caches.

Release-tool/document-only changes use a bounded CI lane when the complete Git
delta matches an explicit allowlist and both required checks have already passed
on the exact protected-master base. Both required job names still execute and
record the reused SDK source/run. CI has only `contents: read` and `actions: read`;
the latter reads baseline run/job metadata. This is source correspondence, not a new
Rust/browser qualification. Changed release tools run their focused Python tests,
and documentation runs the existing checker. Any SDK, package/lockfile, sample,
xtask or unknown-path change uses the full original checks. Missing history,
uncertain scope or unavailable/failed baseline CI also selects the full lane.
Release preparation/signing/result/publisher gates and Commit Guard are unchanged.


Run the smallest existing check set that proves the change. Examples of selection:

| Change | Verification to select |
|---|---|
| Documentation only | `cargo xtask check docs`, `git diff --check`, exact new-file content/whitespace and link/snippet review |
| Rust behavior | Owning package's meaningful tests, affected feature/consumer checks, applicable format/lint |
| Browser/loader behavior | Affected WASM build and the relevant browser/CSP/lifecycle flow; report browser evidence separately |
| Dependency/package metadata | Affected lock/feature/target, package and inventory/security checks; reconcile unchanged evidence |
| Release tooling | Focused guard/identity/format tests and review; real hosted/key/signing actions need release authority |

Do not rerun an accepted suite merely because a later document changed. Reuse
evidence only when its source/input/feature/target correspondence is established.
New or affected inputs need new proof; a green result for another source is not
a substitute. A README included in a Cargo archive is a package input.

[rust.yml](../.github/workflows/rust.yml) is the existing unprivileged PR/master
CI lane, with required check names **Rust 1.99 MSRV** and **Stable native,
samples, and Chrome**. Preserve their names and protection requirements. Local
focused checks do not replace required CI. CI success covers its actual matrix,
not every platform or feature. A squash merge has a new commit identity: record
its actual SHA/tree and the resulting master CI without relabeling the PR run.

## Batch small documentation changes

Collect related spelling, wording, navigation and recorded-status corrections
into one coherent delivery at a useful checkpoint. Prepare the complete local
change before opening a PR; update that PR for an affected correction instead of
creating another PR for every small edit. Unchanged accepted checks are evidence
to reuse at their original scope, not work to repeat for a new paragraph.

| Actual effect | Proportional delivery and checking |
|---|---|
| Wording, formatting or navigation only | Batch related edits; use documentation, link and whitespace checks plus the existing bounded CI lane when eligible |
| Commands/examples, API or security meaning, versions/checksums, publication instructions | Review the changed meaning and affected source/evidence; use the relevant existing checks even when the edited file is Markdown |
| Code, dependencies, package inputs or workflow behavior | Complete affected local checks and required PR CI; select full qualification when the existing scope classifier requires it |

This guidance retains protected `master`, Git authorization, strict/builtin Guard
and required check names. It does not select direct-to-master delivery or change
branch protection. A future alternative delivery policy needs a separate owner
decision. Under the existing path, reduce PR frequency by batching work and CI
cost by selecting the bounded lane, while keeping the required checks present.
GitHub documents that skipping a required workflow with path filters or
`[skip ci]` can leave its checks Pending and block Merge; use the existing
[bounded lane](#local-checks-and-ci) rather than that skip mechanism.
See [GitHub's explanation](https://docs.github.com/en/actions/how-tos/manage-workflow-runs/skip-workflow-runs)
and the [release retrospective](RELEASE-RETROSPECTIVE.md).

## CodeQL scope and activation

[codeql.yml](../.github/workflows/codeql.yml) prepares advanced setup with the
same four language families as the current GitHub default setup: Actions,
JavaScript/TypeScript, Python and Rust, using the default query suite. Its jobs
remain disabled until the repository variable `CODEQL_ADVANCED_ENABLED` is
`true`; preparing this file does not change the active remote configuration.

For PR and protected `master` push events, a quick scope job reads the complete
Git diff. Analysis is skipped only when every changed path ends in `.md`.
Mixed changes, source doc comments, dependency/lockfiles, scripts, templates and
workflow changes still select all four analyses. Renames include both paths;
missing history, an unknown base or an empty diff select analysis. Scheduled
weekly and manual runs always analyze. The scope job does not use GitHub event
path filters or the release-only CI classifier: their file limits or broader
allowlists must not suppress security analysis. Required Rust and documentation
checks remain unchanged, including checks of executable examples in Markdown.

YT-236 adds workflow-level concurrency to cancel superseded scans **only for
the same pull request**. Different PRs remain independent, and unique run IDs
preserve all default-branch, weekly, and manually requested CodeQL runs,
including pending jobs. The four-language matrix, triggers, and documented
Markdown-only admission are unchanged. The observed Rust CodeQL PR execution
was diff-informed but did not use an overlay-base cache; Rust overlay support
is not presumed or forced. See [YT-236 diagnostics](CODEQL-CI-PERFORMANCE.md#historical-codeql-pr-concurrency-and-rust-analysis-evidence).

Activation is a separate delivery step. After the workflow is merged and its
local checks pass, record the current default-setup settings, disable default
setup, set `CODEQL_ADVANCED_ENABLED=true`, and run one manual analysis to verify
all four uploads. Active default setup rejects advanced analysis uploads, so do
not run both configurations together. If activation fails, retain the failure,
disable the advanced variable and restore the recorded default setup without
automatically retrying. Do not remove the existing alert dismissals, change
branch protection or add vulnerability ignores. Include any additional protected
development branch in the push trigger if the branch model changes.

See GitHub's [advanced workflow configuration](https://docs.github.com/en/code-security/reference/code-scanning/workflow-configuration-options)
and [default/advanced setup interaction](https://docs.github.com/en/code-security/reference/code-scanning/troubleshoot-analysis-errors/results-different-than-expected).

## Git authority and strict Commit Guard

For agents, implementation authority is not Commit, Push, Merge, tag or release
authority. An owner may approve a concrete batch once, including its exact scope,
CI condition, merge method and subsequent steps. Continue that approved batch
without asking again for each already-authorized step. A passing check grants
no additional authority.

1. Stage only the approved delivery paths; inspect the staged filenames.
2. Invoke the installed trusted Guard directly with `check --staged --profile
   strict --engine builtin --format json`; hook presence is not a replacement.
3. Require exit0, effective strict/builtin and no blocking findings. Immediately
   compare `tree_oid` with `git write-tree` before Commit; rerun after any index
   mutation. Keep normal pre-commit hooks enabled.
4. Before an authorized Push, check the exact accepted remote base to outgoing
   tip with `check --range BASE..TIP --profile strict --engine builtin --format
   json`; keep normal pre-push hooks enabled. Inspect the actual outgoing refs.
5. Preserve Guard outputs, commit/tree and remote readback. Strict is a bounded
   first-screen, not proof that all historical content is clean.

Use the tool path from the local managed installation; never commit personal
installation paths. If a new remote ref would scan unaccepted history or exceed
limits, stop and establish an explicitly accepted existing-remote boundary.
Where the approved batch accepts an existing remote base, a new remote branch
can first be created at that already-existing base, then receive the separately
Guard-checked outgoing commits. Record the live base and actual ref update;
this is not acceptance of newly introduced findings or historical cleanup.
Do not default to an orphan branch, history rewrite, force push, audit profile,
`--no-verify` or hook edits. The existing-remote **tag** path and used-binding
retirement are release-specific controls, not everyday branch steps.

## API, documentation and failures

Follow [naming and migration guidance](NAMING.md), relevant ADRs and
[guide structure](GUIDE-TEMPLATE.md). Update API/setting compatibility notes and
consumers when their contracts change. Keep all repository documentation in
English. Use current source/rustdoc for API contracts; dated reports retain their
own verdicts and limits.

Classify an observed failure before acting. Repair the smallest real defect.
A local wrapper error may need a focused correction, not a new qualification
campaign. Preserve passed evidence and the original failure; no timer, allocation,
key-use or attempt budget is renewed by renaming a run. Before repeating an
uncertain external write, inspect its actual state. A failed hosted job, partial
signing or exhausted authorization does not permit an automatic retry.

For source freezing, tags, protected jobs, signing and publication, switch to
[the release workflow](RELEASING.md). The release freeze is a bounded promotion
window, not a permanent block on development: feature branches may continue,
while master merges wait only when the selected signer gate requires it.

## Rust tooling migration contracts and historical inventory

Use the repository-root `cargo xtask` route documented in
[xtask usage](../xtask/README.md); it is independent of the published CLI.
The alias's relative manifest path is **root-only**. From a subdirectory use
the quoted absolute xtask/Cargo.toml path. The program locates its repository
from its compiled manifest; explicit relative user outputs resolve against the
invoking cwd. xtask is unpublished with its own workspace/lock/target; framework
and browser production dependency graphs do not depend on it.

M12's original inventory at7cfac613f908c5c19fa581438288386809870b56 contained
**52 tracked Python files/14,006 lines**:35 scripts, eight scene, four Board,
two form-proof, three Redis-qualification files, plus **three workflow surfaces**.
Its sorted `<sha256>  <path>\n` inventory digest was
59a2eb8eb822e6d6054591e6c26625cbfdd2f7620c78dd45a28f19767b66b4e5.
The exact52 rows/file hashes, original direct cases, workflow/root-input hashes
and changing checkpoint statuses remain in the
[original migration ledger](https://github.com/bohewu/yetlet/blob/40b66f1ad9bd0eb5666a9e1c8f113310993a3544/docs/M12-PYTHON-MIGRATION.md#all-52-tracked-files)
and [original adoption plan](https://github.com/bohewu/yetlet/blob/40b66f1ad9bd0eb5666a9e1c8f113310993a3544/docs/M12-RUST-TOOLCHAIN-PLAN.md).
Those Git objects/headings are locally verified; remote reachability is
unverified. They describe2026-10-15 migration, **not today's Python inventory**.
Later release/docs helpers can exist; frozen sources are compatibility oracles,
not a claim that every current task must avoid Python or that an old checker
understands later catalogs. Do not run scripts/check_docs.py as the live gate:
its historical REQUIRED list intentionally still names retired documents.

| Accepted mapping family | Canonical Rust route and retained scope |
|---|---|
| Seven YT-084 builders | `build browser`, `keyed-bindings`, `multi-island-dashboard`, `render-bench`, `scene-island`, `form-css-proof`, `board-assets`; independent Todo/Search, Counter/Filter and nested scene/root counter builds, matching locks/tool profiles/assets, no interpreter wrapper |
| Two checks/direct catalog test | `check docs`, `check samples`, `cargo test --locked --manifest-path xtask/Cargo.toml --test check_samples`; English/links/fences/whitespace/Backlog DAG and catalog types/path/route/command/dependency/source-marker negatives preserved |
| Thirty YT-085 mappings | Separate native/browser/Board/scene scenarios, CDP/WebDriver/BiDi/PNG helpers and two direct harness suites; retain assertions, raw/native exit and ownership, not a generic successful smoke. Internal Redis browser fixtures take the already-owned base URL explicitly |
| Twelve YT-086 mappings | `measure baseline`, `render-strategies`, `sample-dx`, `scene-dynamic`; `qualify local-operations`, `multi-instance`, `redis-three-app`, `board`; `release checksums create/verify`; three direct local-operation/Docker/checksum test counterparts as recorded in the ledger |
| Additional invocation graph | Rust cookie_fencing caller launches the Rust fixture with explicit argv/status, not Python or recursive parent Cargo tests. Redis-three-app uses ported browser/Docker helpers. rust.yml, qualification.yml and private-rebuild.yml switched their mapped calls, including real inline private-rebuild archive/provenance/checksum logic, without widening gate/ref/token/timeout controls |

Keep old supported options/defaults, profile selection, output schemas and
report locations; unknown/repeated/unsupported flags are usage errors. Help and
doctor launch no suites/server/browser. Doctor probes installed tools read-only,
never installs; Cargo bootstrap itself can write cache and needs pre-resolved
dependencies offline. Missing tools/targets/browser/driver/capabilities fail
nonzero with Not run/Blocked, never a successful skip or automatic install.
Explicit pinned CSS `--fetch` is separate from build/check, never fallback.

Processes use literal native argv and explicit cwd/env, with spaces/non-ASCII
and native separators preserved, no shell concatenation or Python/Node wrappers.
Keep executable discovery and inherited configured environment, with tool
identity verified and no global default change. Build children select
RUSTUP_AUTO_INSTALL=0; rolling stable stays explicit. Match wasm-bindgen to the
applicable root/nested lock (0.2.129 at migration), not a timeless hardcoded pin.
stdout reports/progress and stderr diagnostics remain separate; continuously
drain bounded captures without deadlocks or secret/raw-cookie/exception leakage.
Monotonic readiness/protocol/process/cleanup deadlines preserve suite budgets;
timeout never retries an uncertain write or truncates a chosen long profile.

Success0 requires complete checks **and cleanup**; validation/capability1,
usage2, owned timeout124, spawn/pipe/setup/cleanup infrastructure125, cancellation
130 at the runner boundary. Preserve exact child-native exit/origin separately,
including reserved-code collisions; Unix signals keep identity/128+signal,
Windows DWORD survives separately from portable mapping/Cargo wrapper status.
Register ownership immediately, including partial startup: Unix owned groups,
Windows suspended child→Job Object→resume; terminate/escalate/wait/reap/close
streams/remove owned profiles and verify absence on every path. Never kill by
executable name or attach to user browsers. Detached daemon groups and Unix
SIGKILL cleanup remain unsupported; unavailable Windows primitives fail closed.
Keep first SIGINT/SIGTERM identity through final report write, even mixed signals;
cleanup/report/removal failures prevent provisional PASS.

Positive and negative parity precede live adoption. Preserve build digest
domains (`yetlet-v1`, `yt075-v1`, isolated domains), sorted filename/NUL/bytes,
truncation/JSON/generation bytes, target/output roots, asset MIME/cache/CSP and
unrelated-bundle isolation. Explicit --check must reject missing/extra/stale CSS
without writes. Catalog Rust-command representation/checker/README adoption must
land together; no route/package/path/dependency assertion is dropped. New raw
measurements/goldens/receipts stay distinct from old acceptance, with exact
source/tools/argv/cwd hashes and no overwrite.

Keep all original direct-case subassertions, not merely test counts: four
catalog cases,10 local-operation cases,18 original Docker cleanup/signal cases,
16 checksum cases,three browser-process cases and two shop-readiness cases.
The existing test suites are the product verification; no generic migration
framework is required. Controlled-PATH failing Python/Node sentinels prove only
the branches actually executed. Source graph audit plus one branch cannot prove
every unexecuted runtime/platform path. Full-load, physical IME/AT and cloud
acceptance must not be rerun or borrowed merely for a language port; see the
[migration qualification limits](QUALIFICATION-MATRIX.md#m12-migration-acceptance-and-unexecuted-paths).

## Opt-in local HTTPS preparation and lifecycle

Settings-enabled CLI scaffolds, including `new --vite`, declare public
`server.tls` keys. Prepare a private application-owned `.dev/certs/` directory
outside the frontend root and Git, then explicitly select Development:

```toml
[server.tls]
enabled = true
certificate = ".dev/certs/localhost.pem"
private_key = ".dev/certs/localhost-key.pem"
```

Install mkcert separately and generate a loopback leaf from the app directory:

```text
mkcert -cert-file .dev/certs/localhost.pem -key-file .dev/certs/localhost-key.pem localhost 127.0.0.1 ::1
```

`mkcert -install` is an explicit operator trust-store action. Keep the CA private
key under operator control and protect certificate/key parents and files.
Public settings carry paths only. Normal CLI new/build/run never install trust
or generate certificates. Supply existing app secret inputs separately. The
usual loader precedence applies, and its public CLI projection skips the
Development secret file.

`build` selects optional `yetlet-dev-tls` compilation and configured frontend
build without reading TLS material or starting listeners. Direct run/watch
uses ordinary Axum with axum-server/rustls. Paired automatic Vite startup exposes
HTTPS/WSS at 127.0.0.1:5173 and fixes its private Rust backend to HTTP
127.0.0.1:3000. Disabling automatic Vite starts direct Rust HTTPS. The existing
GET/HEAD root-page bridge retains exact Host/Origin, confined paths, nonce/CSP
and no-cookie/no-credential forwarding. Frontend build environments remove TLS
paths. Missing/invalid run material fails closed with owned-process teardown.
See [CLI settings](CLI-CONTRACT.md#development-https) and
[dated Linux/MSVC evidence](QUALIFICATION-MATRIX.md#development-https-evidence-and-original-failures).

The 2026-10-18 Linux and MSVC checks used isolated CAROOT and explicit client CA
trust with hostname validation. System browser trust, physical keyboard input,
full browser HMR, Windows GNU and production TLS/PKI remain unqualified by those
checks. Production TLS deployment guidance is separately scoped.

## Protected Windows tools and historical exit 125

Use physical protected executable directories and absolute PATH entries with
trusted owners and no unrelated write grants or reparse traversal. Initialize
Visual Studio's native compiler environment only for the selected process,
then select its protected Cargo directory on PATH. Scope Cargo home/target and
OpenSSL include/library/static settings to the task and retain source/tool hashes.
An ordinary copy inheriting unsafe ACLs does not establish protection. Keep an
attached processed-input console for run/cancellation checks.

The original 2026-10-18 signing build failed before child creation or Job
assignment because both rustup-proxy and physical MSVC Cargo allowed unrelated
writes. Byte-identical protected Cargo changed the same build from125 to0.
The [CLI guide](CLI-GUIDE.md#protected-windows-tool-preparation) owns setup and
[qualification](QUALIFICATION-MATRIX.md#windows-exit-125-diagnosis-and-native-results)
retains original errors, native results and source limits. Global ACLs and PATH
were preserved during that historical remediation.
