# YT-102 — CLI contract baseline verification

Date: 2026-10-18 (Asia/Taipei).

**Historical design state: settings-first/create-and-execute revision independently reviewed PASS.** Later explicit implementation authority and source-matched results are recorded in [CLI implementation evidence](YT-102-CLI-IMPLEMENTATION.md). The remaining text preserves the original documentation-only verification boundary. This is a documentation-only CLI baseline slice within YT-102. The [target contract](CLI-CONTRACT.md) selects future behavior; no new CLI interface is implemented. YT-102 remains Todo because its full journey audit, measurements, DX comparisons and Extension Contract are outstanding.

## Authority and scope

The user requested execution of the supplied `Yetlet-CLI-Contract-Local-Handoff-20261018.md`, accompanied by `Yetlet-CLI-Contract-Review-20261017.md`. The handoff authorizes a contract/documentation slice, intentional naming breaks with migration, appropriate documentation checks and an independent read-only Reviewer. It forbids product/parser/template/test/lock changes, new checkouts/branches, tool installation, app/Vite/watcher/browser execution, Docker/K3s qualification, commit/push and deployment. The earlier review is static context, not an independent PASS, and its conservative alias suggestions do not override the later naming authorization.

Repository AGENTS.md requires English documentation and `cargo xtask check docs`, preserved architecture and unrelated work, and honest task status. The installed reasoning/materiality policies govern child selection and bounded review followups. No `$run-*` workflow or new validation infrastructure was introduced. Ordinary contract choices were resolved locally; no additional runtime authority was inferred from the attachments.

## Live source basis and preservation

- Checkout: `$REPO`; branch `feat/yt090-configuration`.
- Starting HEAD: `a883469862c5be3d88b7fe22262e10be8b53b92a`.
- Starting index: empty. Initial status: 13 tracked modified files, 5 untracked entries at normal Git status depth (including `artifacts/`).
- Existing dirty YT-101 source includes Compose/Docker/K8s, native filesystem adapters, scaffold/config code/tests and four untracked YT-101 source/template/verification files. Existing BACKLOG/ROADMAP M19/M20 content is preserved while adding only slice notes.
- Task-owned evidence: `artifacts/cli-contract-20261018/`. `baseline.json` contains starting status/index and SHA-256 for 726 tracked or explicitly selected untracked source/document files. BACKLOG/ROADMAP `.before` copies preserve the initial dirty versions. This is a local working-tree comparison, not a clean-HEAD or signed-source attestation. Existing artifact subtrees were not read as secrets or rewritten.

Inspected sources: AGENTS.md; README/MVP/architecture and ADRs 0001/0005/0006; YT-087/YT-088 plans; current BACKLOG/ROADMAP; CLI Cargo.toml, parser/help, main, app, doctor, framework/scaffold; filesystem and process boundaries; generator inspection/delivery; parser/command/Config/Tailwind/Vite/dev-bridge/generator tests; Vite dev instructions; YT-101 verification. Statements about current CLI behavior are static findings. No CLI binary, new app build or application runtime was exercised.

## Initial decisions and CR disposition — preceding revision

The table records initial closure. Later user-directed revisions below supersede target choices where stated; current implementation and historical evidence are preserved.

| Finding | Disposition in this slice |
|---|---|
| CR-01, help/version | Resolved in target contract sections 2–3: scoped exclusive help, root build-time version, side-effect-free discovery, scoped usage. Implementation and AC-01 remain future work. |
| CR-02, current guide contradictions | Resolved in current guide: one complete command inventory and YT-101 source matrix, corrected Config/Vite acceptance and Compose/K8s status. Historical verification and embedded template instructions remain unchanged. |
| CR-03, build/dev promises | Resolved in target sections 4–6: one Rust Cargo operation, explicit asset preparation, release/environment separation; integrated builds/supervision deliberately deferred. |
| CR-04, qualified generators | Resolved in target section 5 and current matrix: generated, unedited supported service/cloud profiles only; exact inspection, source revalidation, output and secret boundaries retained. Generalized app support deferred because it requires a separate accepted input contract. |
| CR-05, mixed switches | Resolved in target section 2: service/settings/Tailwind/assets/dev bridge/parent switches separated; framework source moved to Cargo settings in the amendment; no combinatorial template names or generic provider layer. |
| CR-06, project/path selection | Resolved in target section 4: explicit manifest wins, otherwise cwd only; all CLI paths invocation-relative, child cwd app-root; existing standalone/lock/native safety checks retained. Implementation remains future work. |
| CR-07, doctor scope/JSON/bounds | Resolved in target section 6: environment-only scope, explicit uncovered capabilities, schema 1, stable codes/streams, redaction and 3-second/64-KiB probe bounds plus 2-second owned cleanup. App-aware doctor deferred until capability selection is defined; no Node probe required. No fresh probe/OS acceptance claimed. |
| CR-08, child error attribution | Resolved in target section 6: neutral summary, preserve native identity and cleanup; specific repair guidance needs preflight evidence. Product code remains unchanged. |
| CR-09, incidental parser/byte restrictions | Resolved in target sections 7–8: explicit breaking migration, affected consumers/tests, no aliases, retained payload/metadata and safety invariants; generated usage instructions migrate later. Old PASS records do not certify the new interface. |

Current selected direction: creation/execution commands with a small useful option set; shared existing settings for persistent behavior. `new --vite` is Boolean, `dev` becomes `run`, and Vite root/start/build settings live under `[cli.vite]` in public base/profile TOMLs. Earlier mode/bridge/Cargo metadata proposals are superseded by the settings-first revision below. Framework package source declarations remain Cargo-owned dependencies. No target behavior is implemented by these documents.

## Actual changed files

| File | Change |
|---|---|
| `docs/CLI-CONTRACT.md` | Authoritative selected target, current/target/deferred boundary, naming/grammar/path/process/doctor decisions, migration and future acceptance cases |
| `docs/YT-102-CLI-CONTRACT-VERIFICATION.md` | This source, CR, validation, preservation and review record |
| `tools/yetlet-cli/README.md` | Current capability entry point, YT-101 contradictions and current limitations; uses existing spellings |
| `docs/README.md` | Contract/current-guide/verification navigation |
| `docs/BACKLOG.md` | Small baseline progress note; existing item rows/dependencies/statuses preserved |
| `docs/ROADMAP.md` | Small baseline progress note; existing M19/M20 gates preserved |

Only these six repository documents and task-owned evidence are part of this slice. No product Rust, parser, scaffold template, tests, lockfile or historical verification record was edited.

## Validation

Required checks completed against the complete candidate:

| Command/check | Result |
|---|---|
| `CARGO_NET_OFFLINE=true RUSTUP_AUTO_INSTALL=0 RUSTUP_SKIP_UPDATE_CHECK=1 cargo xtask check docs` | Exit 0; 75 required documents, 103 backlog items; PASS |
| `git diff --check` | Exit 0; no output |
| Direct inspection of six changed documents' links/fences/newlines/whitespace and preservation against starting hashes | Exit 0; no failures; only the four intended pre-existing documents differ, all other 722 inventoried files unchanged; BACKLOG/ROADMAP match their initial dirty content after removing only the added slice notes; index/HEAD/branch unchanged |

The repository alias expands xtask to `cargo run --locked --manifest-path xtask/Cargo.toml --`; the environment keeps resolution offline and disables tool auto-install. Logs/native exits are `docs-check.log`/`.exit`, `diff-check.log`/`.exit`; `focused-checks.json` records the direct inspection/comparison. These are task-owned outputs, not a new repository harness. The existing docs check validates its fixed required-document list plus recursive Markdown language and backlog DAG; it does not by itself promise link/fence checking of every newly added document.

Future acceptance cases AC-01–AC-14 are specifications, not tests that passed today. No full CLI suite, new-interface runtime test, Windows, browser/CSP, Docker/K3s, performance measurement or full DX journey audit was run by this slice. Historical evidence is retained separately and neither erased nor promoted to current/new-interface proof.

## Initial independent review — preceding revision

Initial independent native Reviewer `/root/cli_contract_review` returned **FAIL** with one required P2 correction: the current Config section still described the build-time root as mandatory despite the existing absolute `YETLET_CONFIG_ROOT` override. This violates CR-02's factual-guide requirement and can misdirect relocation/rebuild work. After reading the Materiality Gate, the parent verified `templates/config/config.rs:54–72` and changed only the guide paragraph to explain the default, explicit runtime root, public-file provisioning and invalid-root refusal. Product/templates/history remain unchanged. Focused re-review by the same Reviewer returned **PASS**, with empty `issues`, `required_followups` and `optional_notes`. Both decisions are retained in `reviewer-initial.json` and `reviewer-final.json`.

Review scope is the six documents, handoff/review criteria and immediately related current source/tests. Native trace confirms registered role `reviewer`, model `gpt-6-astra` (matches configured model), effective effort `high` (matches dispatched effort); selector causality is indeterminate. The shared resolver observation is `enforced`, with no degradation/conflict; this proves observed effort satisfaction, not selector causality. Bounded telemetry and requested/observed decisions are retained in `reviewer-trace.json` and `reviewer-reasoning-*.json`. Ordinary ad-hoc review uses adaptive deep reasoning, not formal assurance. No replacement Job, role emulation or self-review is used.

`review-targets.json` identifies the initial six-file candidate. Focused re-review covered the sole guide correction; post-PASS document edits only record acceptance/results and replace the contract's pending status line. They do not change the reviewed target behavior. Final document/preservation checks are recorded separately below; no duplicate Reviewer dispatch is needed for administrative closeout.

Workspace preflight reports `configured=true`, `health=ok`, `profile_eligibility=eligible`, `catalog_state=current`, profile `premium`, model set `openai@5`, reasoning projection `openai-gpt6-v1@2`. Inspection helper `/root/cli_current_inventory` used native `repo-scout`, observed `gpt-6-luna`, effective `high`; trace matched role/model/effort and resolver observation was `enforced`. It supplied source/README facts only and is not the independent acceptance reviewer.

## Initial closeout — preceding revision

After PASS and acceptance-status recording, the same offline/locked docs command
passed again (exit 0, 75 required documents / 103 backlog items); `git diff --check`
also returned 0. Six-document link/fence/whitespace checks found no failures.
`docs-final.log`/`.exit`, `diff-final.log`/`.exit` and `final-checks.json` retain
the results. This result paragraph is administrative recording after those checks.

Final branch/HEAD remain `feat/yt090-configuration` /
`a883469862c5be3d88b7fe22262e10be8b53b92a`. Index remains empty. Git status has
15 tracked modified files and 7 untracked entries at normal depth, including
the preserved YT-101 work, two new contract documents and `artifacts/`.
All 722 inventoried files outside the four intended existing-document edits are
byte-identical to the starting working tree; no unexpected source changes exist.
BACKLOG/ROADMAP reproduce their starting dirty contents after removing only
this slice's added notes. No staging, commit, push, branch/worktree creation,
tool installation, app/server/browser runtime or deployment occurred.

## User-directed source-configuration amendment — PASS

After initial acceptance, the user clarified that a released CLI should not
require a framework checkout path and that any local source selection belongs
in settings, rather than a CLI argument. The user then explicitly requested
Reviewer review. This is a substantive design revision: the earlier PASS is
preserved for its original revision and is not acceptance of these amendments.

The selected target removes `--framework-path` from both `new` and `doctor`.
Canonical release-facing creation is `yetlet new demo`; an optional `--parent`
is unnecessary in the normal cwd workflow. A release-qualified embedded
crate/version map generates ordinary version dependencies. Local development
uses Cargo-owned path dependencies or `[patch.crates-io]` in the app/workspace
Cargo.toml or local `.cargo/config.toml`, with compatible versions and an
explicit separate lock refresh. No new Yetlet source config, source environment
variable, CLI flag, directory guessing, registry query or network/install effect
is introduced during creation. Doctor no longer accepts/checks framework source;
it explicitly reports source readiness as not checked.

This is an option removal and dependency-manifest/source contract change, not
pure spelling. The migration table and AC-11 identify the changed manifests,
source/lock fixtures and release qualification. Current preview commands and
product/templates/tests remain unchanged; release/template implementation and
YT-096/YT-020 publication gates still apply. Registry declarations or local
patches do not inherit existing generator acceptance: the current qualified
local-source matrix, source/lock inspection and refusal boundaries remain.
A later release-template generator extension needs an explicit accepted input
contract and source-matched verification.

Only `docs/CLI-CONTRACT.md`, this verification record and the current guide's
opening target/current clarification changed in this amendment. The task-owned
`source-config-*.before` files and `source-config-before.json` preserve their
pre-amendment contents/identities. All original six-document work and existing
YT-101/M19/M20 changes are retained. No product or runtime work is authorized.

Cargo's official [override reference](https://doc.rust-lang.org/cargo/reference/overriding-dependencies.html#the-patch-section)
and [path dependency reference](https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html#specifying-path-dependencies)
were checked for the settings mechanism; these sources do not attest Yetlet's
unimplemented release path. This is a contract decision, not a Cargo runtime
experiment.

Amendment checks passed: the same offline/locked `cargo xtask check docs`
returned 0 (75 required documents / 103 backlog items), `git diff --check`
returned 0, and direct checks of these three documents found no broken local
links, fence/newline/whitespace failures or unexpected source changes. All 722
baseline files outside the original four existing-document edits are unchanged;
index/HEAD remain unchanged. Logs/exits and checks are retained under
`source-config-docs.*`, `source-config-diff.*` and `source-config-checks.json`.
Independent focused Reviewer `/root/cli_contract_review` returned **PASS**, with
empty issues, required followups and optional notes. Its native role/model/effort
trace again matches `reviewer` / `gpt-6-astra` / effective `high`; shared resolver
observation is `enforced`, without conflict/degradation. Selector causality
remains indeterminate. The result, bounded trace and observation are retained
in `source-config-reviewer-final.json`, `source-config-reviewer-trace.json` and
`source-config-reviewer-reasoning.json`. This PASS covers the three amended
documents and source/config/release/generator boundaries, not runtime or release
qualification. Earlier evidence is preserved. Subsequent edits record only this
acceptance/result; they do not alter reviewed behavior.

## User-directed Vite option amendment — PASS

The user rejected the inconsistent `--assets vite` / `--dev-bridge` split and
asked for one Vite entry point. The selected target is `new --vite MODE` with
required mode `build` or `dev`. `build` selects the existing production-only
Vite output; `dev` selects the same production setup plus the existing bounded
two-file/script dev bridge. Omission retains no-Vite behavior. A required mode
keeps NAME parsing deterministic and uses the normal separated/equal value
syntax. Missing/unknown/duplicate modes and the removed options are usage
errors. `--tailwind` stays independent and selects the official plugin with
either Vite mode; without Vite it retains the standalone integration.

This is a selected naming/argument-shape change, not new output or process
behavior. No mode starts Vite, installs tools, watches/restarts Rust, adds routes,
changes CSP/secrets, or broadens generator input. The two-terminal prerequisite,
initial production dist snapshot, root-only read-only bridge and dev-delivery
exclusion remain. The release/source-configuration gates from the preceding
amendment are preserved. Prior PASS results cover their earlier revisions;
independent focused review of this Vite amendment is still required.

Only the target contract, this verification record and the current guide's
opening target/current clarification are edited. Implemented flags and all
product/parser/template/test/lock bytes remain unchanged. Task-owned
`vite-options-*.before` copies and `vite-options-before.json` retain the
pre-amendment documentation. The migration rows, mode table and AC-05/AC-12
identify future implementation and verification, not newly passed runtime tests.

Focused checks passed: offline/locked `cargo xtask check docs` returned 0
(75 required documents / 103 backlog items), `git diff --check` returned 0,
and direct three-document links/fences/whitespace checks found no failures.
All 722 baseline files outside the four original existing-document edits are
unchanged, with no unexpected source edits; index/HEAD remain unchanged.
Results are retained in `vite-options-docs.*`, `vite-options-diff.*` and
`vite-options-checks.json`. Independent focused Reviewer
`/root/cli_contract_review` returned **PASS**, with no issues or required
followups. Native trace verifies `reviewer` / `gpt-6-astra` / effective `high`;
the shared resolver observation is `enforced` without conflict/degradation,
while selector causality remains indeterminate. Result/trace/observation are
retained in `vite-options-reviewer-final.json`, `vite-options-reviewer-trace.json`
and `vite-options-reviewer-reasoning.json`. This acceptance covers the naming,
grammar, output mapping and preserved boundaries in the three amended documents,
not implementation or runtime qualification. Post-PASS edits only record status
and evidence, with no further behavior changes.

## User-directed settings-first/create-and-execute revision — PASS

The user's later clarifications select simple Boolean `new --vite`, `run`
instead of `dev`, independent configurable frontend auto-start and auto-build,
a configurable frontend directory defaulting to `frontend/`, environment
variable overrides, and reuse of the existing settings mechanism rather than
Cargo metadata. The final direction is creation/execution with only useful
common options; persistent behavior belongs in settings. This explicit user
scope permits documenting these bounded future execution effects; it does not
authorize code, tool installation, servers, browser/Windows/Docker/K3s runtime
or full supervisor implementation now. Previous PASS results remain attached
to their previous revision.

The authoritative target contract is reconciled as one current design:

- Public `[cli.vite]` in existing base/profile TOMLs declares enabled/root/
  auto_start/auto_build, using the existing loader's underscore key rules.
  New --vite writes the enabled full Vite scaffold with automatic behavior
  explicitly declared; non-Vite defaults disable all frontend execution.
- Shared APP_ENVIRONMENT and existing layer/type/unknown-key rules remain.
  APP_CLI__VITE__ROOT/AUTO_START/AUTO_BUILD/ENABLED and
  APP_CLI__GENERATE__K8S__REPLICAS use the existing ENV mapping. No Cargo metadata
  fallback, separate settings parser, new provider/schema or YETLET_VITE_AUTO_*
  compatibility mode is introduced.
- CLI public settings reuse Loader.load_from_env with a filtered public ENV
  snapshot, no backend Development secret source, and strict typed extraction
  of cli from the merged public TOML. This is a future adapter using existing
  APIs, not a claim of an existing projection method. Generated application
  Settings must explicitly accommodate cli without disabling validation.
- Shared settings root selection preserves explicit absolute YETLET_CONFIG_ROOT.
  Frontend root remains relative to the selected manifest, not invocation or
  relocated config cwd. Frontend .env* loading is confined to that root, and
  backend credentials/secret selectors are not forwarded to Vite.
- run consults auto_start only; build consults auto_build only. Disabled branches
  preserve Rust-only execution; selected frontend build completes before Rust,
  and frontend failure stops the action. Existing frozen/no-install/native exit
  and owned cleanup rules remain. Two-tree native lifecycle needs its own
  bounded contract/evidence before implementation; no Rust watcher or general
  proxy/auth/production frontend server is implied.
- Common creation/target/output/image/format/release options remain. Runtime
  Vite options/modes disappear; replicas becomes a persistent typed setting.
  New generated namespaces/files/nested layout and combined Vite output are
  explicit content/effect changes, not naming aliases or inherited generator
  acceptance. Current exact-source generators retain their refusal boundary.

Existing-source inspection independently confirmed the constraints: Loader
requires base/profile files and merges declared same-kind keys, rejects hyphens,
and deserializes the whole root; generated Settings currently accepts only
server/service. There is no existing section-projection API. The helper
`/root/cli_current_inventory` supplied these facts read-only. No product source,
settings type, profile TOML/template, parser, test or lockfile is changed now.

CR-01/02/04/07/08 discovery, current-guide, generator, Doctor and native-diagnostic
boundaries remain explicit. CR-03 now selects conditional configured execution;
CR-05 selects settings-first option admission; CR-06 adds shared settings and
project-relative frontend roots; CR-09 explicitly migrates the resulting
namespaces/layout/ENV/effects. Contract AC-01–AC-14 are future acceptance cases,
not executed tests. No new backlog ID or completed overall YT-102 is implied.

This revision changes only the contract, this record and the current guide's
opening current/target explanation. `settings-first-*.before` and
`settings-first-before.json` preserve the preceding documents. Candidate checks
passed: offline/locked `cargo xtask check docs` exit 0 (75 required documents,
103 backlog items), `git diff --check` exit 0, direct three-document link/fence/
whitespace checks clean. All 722 unaffected baseline files remain byte-identical,
with no unexpected changes; index/HEAD unchanged. Logs/check results are retained
in `settings-first-docs.*`, `settings-first-diff.*` and
`settings-first-checks.json`. Independent focused Reviewer
`/root/cli_contract_review` returned **PASS**, with empty issues, required
followups and optional notes. Native trace confirms registered reviewer,
configured `gpt-6-astra` and effective `high`; shared resolver observation is
enforced without conflict/degradation. Selector causality remains indeterminate.
Result, bounded trace and observation are retained in
`settings-first-reviewer-final.json`, `settings-first-reviewer-trace.json` and
`settings-first-reviewer-reasoning.json`. The scope is the three amended
documents, shared-loader feasibility, settings/ENV/path/action behavior and
preserved safety/current-source boundaries, not runtime or release acceptance.
Prior outputs are retained; post-PASS edits record status/evidence only.

## Remaining work and handoff

The preceding revisions and current settings-first revision have no required review followup. No new CLI behavior is implemented. All selected interface behavior still needs explicitly authorized implementation and source-matched verification. Probe/process changes need honest Linux/Windows evidence; existing bridge/hosting limitations persist.

Next independently implementable proposal after acceptance and separate authorization: scoped help/version/usage plus neutral child-failure summaries, without taking on scaffold or doctor changes. Next research-only YT-102 slice: DX Extension Contract with real journey gaps, measured tool comparisons and process/asset/secret ownership. YT-103 and M20/release gates remain separate. No new backlog ID is allocated; YT-102 remains Todo, YT-020 remains Blocked.
