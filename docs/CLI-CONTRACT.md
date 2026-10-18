# Yetlet CLI Contract

Date: 2026-10-18 (Asia/Taipei). Task: YT-102, accepted CLI contract and authorized local implementation slice.

- **Current implementation:** the [CLI guide](../tools/yetlet-cli/README.md#current-command-and-capability-inventory) describes the revised working-tree CLI. [Implementation evidence](YT-102-CLI-IMPLEMENTATION.md) separates native Linux tests, subsequent selected Windows GNU verification, MSVC compile-only evidence and remaining release/generator limits.
- **Accepted revised design:** implementation was explicitly authorized after independent design review. The latest user direction makes `new --vite` imply `--settings`; plain new retains the runtime module opt-in. The [design verification record](YT-102-CLI-CONTRACT-VERIFICATION.md) preserves prior revisions.
- **Remaining qualification:** [selected native Windows GNU lifecycle cases](YT-102-WINDOWS-VITE-VERIFICATION.md) now have direct evidence; [current generator compatibility](YT-102-GENERATOR-COMPATIBILITY.md) records the separately extended local-source representation. Additional platform/runtime and registry release qualification remain open. YT-102 overall stays Todo because its broader DX research is incomplete; public release stays blocked.

## 1. Historical starting implementation and authority

Inspected checkout: `$REPO`, branch `feat/yt090-configuration`, HEAD `a883469862c5be3d88b7fe22262e10be8b53b92a` plus its dirty tree. HEAD alone is not the source identity; preserved hashes are recorded separately.

| Starting surface before this implementation | Historical behavior |
|---|---|
| Discovery | Root help only; no scoped help or CLI version query. |
| `new` | NAME before options; required local `--framework-path`; existing `--dir`, `--cloud`, `--with-config`, `--with-tailwind`, `--assets vite`, `--with-vite-dev`. Creates files without lock/dependency setup or child startup. |
| `doctor` | Rust/Cargo probes, optional framework check, new-platform capability; unversioned JSON, no explicit probe deadline/output bound. |
| `build` / `dev` | Explicit manifest, protected standalone app and lock; one foreground frozen Cargo build/run. No frontend automation. |
| Generators | Configuration only, exact qualified generated-source/lock inspection, source revalidation and native no-overwrite publication. YT-101's local-source matrix remains bounded. |
| Settings | `yetlet-config` merges base/profile/explicit Development secrets/OS overrides into caller-owned typed settings. Existing generated Settings has server/service only. CLI/Vite settings are not implemented. |

Explicit user direction and safety invariants take priority over earlier proposed names. Historical verification remains valid only for its recorded source/scope. Yetlet is independent, small, Rust-native and HTML-first; it does not target another framework's API or DX parity. [ADRs 0001](adr/0001-separation-of-concerns.md), [0005](adr/0005-opt-in-capability-modules.md) and [0006](adr/0006-handler-first-mvc-optional.md) remain: no mandatory Controller, DI, frontend runtime, provider registry or new project system. This CLI/tooling revision does not change those framework decisions.

## 2. CLI chooses actions; settings choose persistent behavior

The user-selected rule is: keep creation/execution commands short; put persistent tool choices, paths and automatic behavior in the existing settings system. Retain an option only for a common immediate choice, output formatting or explicit target/destination. Do not create a flag for every settings key, add a general `--set` language, or duplicate Cargo test/fmt/lint/setup/deploy commands.

Implemented working-tree command syntax:

```text
yetlet --help | -h
yetlet --version | -V
yetlet new NAME [--parent DIRECTORY] [--vite] [--tailwind] [--service] [--settings]
yetlet doctor [--json]
yetlet run [--manifest-path PATH]
yetlet watch [--manifest-path PATH]
yetlet build [--manifest-path PATH] [--release]
yetlet generate docker [--manifest-path PATH] --output DIRECTORY
yetlet generate compose [--manifest-path PATH] --output DIRECTORY --image IMAGE
yetlet generate k8s [--manifest-path PATH] --output DIRECTORY --image IMAGE
```

| Surface | Selected responsibility |
|---|---|
| `new` options | One-time choices about files/code to generate. `--vite` and `--tailwind` are simple switches. `--service` retains the bind/health/shutdown scaffold; `--settings` retains app-runtime typed-settings scaffolding. These do not start processes. |
| `--parent` / `--manifest-path` | Choose creation destination/project; useful before a new project has settings or when executing from elsewhere. |
| `--release` | Common per-build Cargo profile selection; independent of application environment. |
| `--output` / `--image` | Explicit generator destination and per-artifact image identity; keep existing bounded validation. |
| `--json`, help/version | Machine/report/discovery formatting. |
| Rust watch, Vite root, automatic startup/build, K8s replicas | Persistent settings, not more CLI options. `--replicas` moves to settings. |
| `watch` | Common execution shortcut: share `run` and enable Rust watch for this invocation, without changing settings files. |

`dev` is renamed to `run`: it executes the selected project. No second `dev` synonym, `--vite MODE`, `--assets`, `--dev-bridge`, runtime `--vite`, root/auto-start/auto-build flags, or framework-source flag is selected. No old-name aliases or deprecation window are required for this unpublished preview; fixed migration hints are allowed. Existing generated apps/instructions are never rewritten automatically.

Examples use the selected contract:

```text
yetlet new demo
yetlet new webapp --vite --tailwind
yetlet run
yetlet build --release
yetlet generate k8s --output /work/out/service-k8s --image localhost:5000/team/service:v1
```

The generator example requires a separately qualified source representation.
Current service/settings/Vite scaffolds use explicit compatible local Cargo
path overrides and a separately prepared lock; registry-only delivery remains
outside this acceptance. See the linked compatibility record.

## 3. One settings mechanism

Reuse [yetlet-config](CONFIGURATION.md) and its existing base/profile/typed-value/OS-override rules. **CLI and Vite behavior do not live in Cargo metadata.** Cargo.toml retains its package/dependency responsibility; existing historical generator markers are not a new configuration system.

The target adds public tool settings to the same `base.toml` and selected `development.toml`, `staging.toml` or `production.toml`. No new CLI-specific file format, schema-version layer, global search path or provider framework is selected.

Example public base declarations; selected profiles override existing declared keys:

```toml
[cli.rust]
watch = false

[cli.vite]
enabled = true
root = "frontend/"
auto_start = true
auto_build = true

[cli.generate.k8s]
replicas = 1
```

All new scaffolds declare this public CLI namespace, including `cli.rust.watch = false`. The Rust watch setting is independent of Vite and the runtime `--settings` opt-in. An absent `cli.rust` section defaults to disabled for existing apps; a present section must contain a boolean `watch` and rejects unknown keys. Without `--vite`, the default is `enabled = false`, `auto_start = false`, `auto_build = false`; root remains `frontend/`. `new --vite` selects the complete Vite build/development scaffold, implies the runtime `--settings` module, and writes the three true values shown above. Operators can change them through normal settings. `--settings` still controls whether app-runtime typed settings/code/dependencies are generated; CLI public settings alone do not add a runtime configuration dependency to a plain SSR app.

Keys use the existing loader's snake_case rules: `auto_start` and `auto_build`, not the earlier conversational hyphenated spelling. Current key/type/unknown-field failures remain errors. The shared source precedence, low to high, is:

1. Declared `base.toml` values, or caller-provided defaults where explicitly applicable.
2. Exactly one environment overlay, selected by `APP_ENVIRONMENT` (`Development`, `Staging`, `Production`; existing default Production).
3. An explicitly enabled Development secret source for the **backend application only**.
4. Real scoped OS environment overrides.

For CLI public settings, step 3 is deliberately not enabled: no `YETLET_DEV_SECRETS` file is opened to decide tool behavior. Environment names follow the existing APP-prefix mapping:

| Environment variable | Public setting |
|---|---|
| `APP_CLI__RUST__WATCH` | `cli.rust.watch` |
| `APP_CLI__VITE__ENABLED` | `cli.vite.enabled` |
| `APP_CLI__VITE__ROOT` | `cli.vite.root` |
| `APP_CLI__VITE__AUTO_START` | `cli.vite.auto_start` |
| `APP_CLI__VITE__AUTO_BUILD` | `cli.vite.auto_build` |
| `APP_CLI__GENERATE__K8S__REPLICAS` | `cli.generate.k8s.replicas` |

The earlier proposed `YETLET_VITE_AUTO_START`/`YETLET_VITE_AUTO_BUILD` names and `[package.metadata.yetlet.vite]` proposal are superseded, not implemented compatibility promises. Do not silently parse `${VARIABLE}` expressions in TOML or add shell expansion. `APP_ENVIRONMENT` remains the shared file selector; neither `run` nor `--release` secretly changes it.

### Shared loader integration

Loader deserializes the whole merged root and has no section-projection API. The CLI reuses `Loader::load_from_env::<serde_json::Value>()` with only APP_ENVIRONMENT and related APP_CLI variables, leaves Development secrets disabled, then strictly deserializes the cli table. The shared parser owns merge/key/type/profile semantics. CLI public files receive native protected-path preflight. Selected generated runtime Settings explicitly contains the known strict CLI namespace; global unknown-field validation remains enabled. Legacy internal scaffold APIs retain their exact previous representation for generator qualification.

New project settings files are required inputs once selected. For legacy standalone apps with no CLI namespace, retain disabled-watch/disabled-Vite/default-generator behavior; do not infer tools from filenames/dependencies. Explicit CLI overrides require declared CLI keys. Missing required profile files, partial configuration, malformed sources and invalid values never trigger a silent fallback. No automatic edit of an old application's settings/Rust types occurs.

## 4. Project, paths and frontend layout

`run`, `build` and generators use explicit `--manifest-path` when supplied, otherwise exactly cwd/Cargo.toml. Preserve standalone `[package]` plus own `[workspace]`, existing lock and protected native path/size checks. Missing/virtual/inherited/multi-package workspaces fail; no ancestor search, sibling selection or guessing.

CLI path arguments resolve against the captured invocation cwd. `new --parent P NAME` creates P/NAME; P defaults to cwd and must already exist/protected. Generator `--output D` names the complete new destination D under an existing protected parent. No merge, overwrite or force option. Existing NAME/reserved-name checks remain, with unchanged narrower K8s DNS-label validation.

CLI public settings default to the selected app root. An explicit absolute `YETLET_CONFIG_ROOT` can select the common public settings directory using its existing absolute/no-parent-traversal contract. Backend runtime retains its existing app-owned root/default and explicit-secret behavior. `cli.vite.root` resolves relative to the selected app's Cargo.toml directory, not invocation cwd or a relocated settings directory. Use safe native reads/identity checks and reject traversal, links/reparse escapes and untrusted paths before effects.

Default Vite layout, selected by `new --vite`:

```text
app/
  Cargo.toml
  base.toml
  development.toml
  staging.toml
  production.toml
  src/
  frontend/
    package.json
    vite.config.mjs
    .env.example
    dist/                 # produced by an explicit selected build
```

`root` may select another protected project-relative directory; run/build/frontend dotenv loading use the same resolved root. Frontend dotenv loading is confined to that directory's `.env*` convention, with no repository-root/ancestor discovery and no backend settings/secret-file loading. CLI/backend settings retain their explicit Development-secret rules; do not add backend automatic `.env` discovery. Frontend `VITE_*` values/imports are public. Child environments must keep backend APP credentials and Development secret selectors out of Vite; directory separation alone does not isolate inherited OS variables.

The former Vite scaffold was flat with separate opt-ins and disabled dotenv discovery. Complete Vite scaffolding, the nested root and confined frontend dotenv behavior are intentional generated-output/file-access changes. Retained legacy library fixtures do not qualify the new representation. Keep build-time versioned assets, no runtime Node in production, strict production CSP and no implicit installs.

### Framework package sources

A release-facing `new` emits the CLI release's qualified crate/version declarations without querying a registry or selecting a developer checkout. Actual usable registry-backed templates require YT-096/YT-020 qualification; no released version is asserted. Local crate overrides remain Cargo path/patch declarations with compatible versions and a separately refreshed app lock. These are dependency settings, not CLI/Vite execution configuration. The executable no longer accepts `--framework-path`; no source-directory guessing, hidden source environment variable or custom resolver is added.

## 5. Execution follows the selected settings

| Command | Result/effects and success boundary |
|---|---|
| `new` | Validate and atomically publish one owned scaffold tree; no lock generation, dependency resolution/install/download, child startup or listener. |
| `doctor` | Bounded environment report only; no app/CLI settings or framework-source input, builds, package scripts, install or listener. |
| `run` | Validate app/settings first. Select Rust watch only when `cli.rust.watch = true`. With Vite disabled or `auto_start = false`, supervise only Rust and the optional watcher. With Vite enabled and `auto_start = true`, also supervise the frontend dev server from the configured root. No frontend production build is implicit. |
| `watch` | Validate the same app/settings as `run`, then enable Rust watch for this invocation even when the setting is false or absent. Vite selection still uses `enabled && auto_start`; settings files are unchanged. |
| `build` | Validate app/settings first. With Vite disabled or `auto_build = false`, compile Rust only and state that frontend build was skipped. With Vite enabled and `auto_build = true`, complete the frontend production build first, then frozen Cargo build. Frontend failure stops before Rust compilation. Success identifies completed/skipped stages, not invented asset freshness. `cli.rust.watch` has no execution effect and Watchexec is not required. |
| `generate docker/compose/k8s` | Qualified source/lock inspection and revalidation, then protected no-replace configuration publication. No tool execution, source export, image build/pull, credentials lookup or deployment. K8s replicas come from settings (default 1, integer 1–100). |

Presence/enabled status declares integration; it does not override `auto_start` or `auto_build`. The two switches are independent. A skipped frontend build requires separately prepared assets wherever the app needs them; no success claim about missing/stale assets. Auto-start likewise retains the current bridge's initial production dist prerequisite. Dependencies and tools are prepared separately; no npm install/ci, Rust install, lock refresh or network fallback is hidden in either action. Package scripts/build/app code still have their own effects; frozen toolchain resolution is not a sandbox.

The selected frontend operations are the generated package's documented build/dev scripts, with literal argv and fixed action selection. No configurable arbitrary command strings or unknown-argument forwarding. Keep the bounded read-only root-page bridge, loopback/strict-port/CSP/nonce/no-auth/no-cookie/no-POST boundaries until a separate accepted extension changes them. No general proxy, browser launch or production HMR is implied. Rust restart requires the explicit watch setting below.

The generated Vite entry/shared/lazy modules now retain their interaction counter
and DOM across accepted development HMR updates, with disposal of old listeners.
User-defined JavaScript and TypeScript files may use normal frontend-relative
imports and subdirectories without registration. Physical root confinement and
private/tooling-file refusals remain. The CLI owns configured run/stop/build;
Vite owns HMR. [Implementation and integration evidence](YT-102-STATEFUL-HMR-IMPLEMENTATION.md)
records the state boundary, tested edits/restores and remaining scope.

With Rust watch disabled, the pair supervisor uses bounded frontend readiness, native signal ownership, stage attribution and cleanup of both owned trees. Current native Linux tests cover failure, cancellation, descendants and PTY restoration; [native Windows GNU evidence](YT-102-WINDOWS-VITE-VERIFICATION.md) covers six selected real Vite/Rust cases for startup, failure, C/BREAK, port collision and disabled auto-start. In that non-watch mode, child failure must stop/reap the other owned tree; cleanup uncertainty is failure. Native Windows console/headless limits remain explicit; compile-only evidence cannot prove lifecycle support. Manual frontend startup and the current two-terminal fallback remain usable when automatic startup is disabled. Windows cross-compilation alone does not satisfy the native qualification gate.

### Opt-in Rust watch

Invoke `yetlet watch` to enable Rust watch for this invocation. It shares the `run` pipeline and overrides a false or absent watch setting after normal strict settings validation, without modifying settings files. Alternatively, set `[cli.rust] watch = true` in the selected public settings, or use `APP_CLI__RUST__WATCH=true` after declaring the key in base.toml, then invoke `yetlet run`. Prepare Watchexec separately on PATH; integration targets Watchexec 2.8.0 and does not install it. No runtime flag, Cargo metadata or watch-list settings are introduced. `yetlet build` ignores watch execution and does not require Watchexec.

Watchexec emits JSON file events only. The parent CLI owns the initial frozen/offline Cargo run and each later Rust generation, separately from the watcher and optional Vite tree. A relevant event stops and reaps the old Rust tree before starting its replacement. A compile error or Rust runtime exit is reported and leaves watch waiting for the next change. Rust restarts retain the same Vite process. User cancellation, Vite exit or watcher death ends the session and cleans up all owned trees; cleanup uncertainty remains failure.

Watch scope is the selected app root: Rust `*.rs` files (including `build.rs`), Cargo.toml and Cargo.lock. It excludes the configured frontend root, target, node_modules and dist directories, plus CARGO_TARGET_DIR when it is under the app root. Symlinks are not followed. Local dependencies outside the app root are not watched. Special target directories selected through Cargo configuration need separate exclusion if required; use CARGO_TARGET_DIR for an excluded in-root target directory or keep it outside the watched root.

CLI policy settings are loaded once per run; restart the CLI to change that policy. Each fresh backend process reads its own runtime settings. Rust restart does not promise browser refresh, preservation of in-process state or graceful request draining. Native watch lifecycle evidence is recorded separately from the historical Vite-pair verification; the earlier pair results do not qualify this added mode.

Generators retain the recorded YT-101 unedited local-source representation and
extend exact inspection to current `new --service`, optional settings, and Vite
plain/plugin scaffolds. Vite implies settings. Current pinned dependency
declarations need explicit compatible absolute local Cargo paths and a separately
prepared lock; no registry resolver or framework-source option is added. All
public CLI profiles are strictly validated; runtime public tables remain exact
and token declarations empty. The Production public Vite root selects the
verified built graph, independently of valid Development/Staging roots. Docker's
additive `app-src-source-paths` inventory maps canonical exported files back to
their actual project-relative inputs. The existing explicit absolute
YETLET_ASSET_ROOT selects the runtime image graph. Generation never starts watch,
Vite or a frontend build. Legacy dev bridge, standalone Tailwind, edited/general
apps and unknown dependencies/layouts remain refused. Runtime token handling,
sanitized three-context exports, bounded public inputs and no-overwrite remain
unchanged. Actual evidence and remaining qualification are in the
[compatibility record](YT-102-GENERATOR-COMPATIBILITY.md).

## 6. Grammar, diagnostics and process contract

Options follow their complete command path; NAME may occur anywhere within new's options. Value options accept separated/equal forms; booleans are presence-only. Unknown/duplicate/misplaced options, unsupported values, empty/blank values and extra positionals are usage errors before effects. No short names beyond help/version, abbreviation, arbitrary lists, implicit CLI flags from environment or `--` argument forwarding. Native paths/spaces/Unicode remain literal with no lossy conversion, shell/glob/tilde expansion or unsafe canonicalization.

Root, every command, generate namespace and all generator leaves have exclusive `--help`/`-h`; no ordinary operands/options accompany help. Root-only exclusive `--version`/`-V` emits build-time CLI identity, not a clean-source claim. Empty root arguments show help. Bare generate is scoped usage error. Help/version require no project/settings/toolchain reads, probe, write or child startup, including unsupported OSes.

Human success/help/version/doctor reports use stdout; safe scoped CLI errors and usage go to stderr. Build/run retain native child diagnostics/streams, which are not JSON or guaranteed secret-redacted. No CLI message echoes raw configuration, environment values, secrets or arbitrary argv. Include origin/stage and stable codes such as `CLI_USAGE`, `CLI_PROJECT`, `CLI_SOURCE_UNSUPPORTED`, `CLI_PUBLISH`, `CLI_SPAWN`, `CLI_CHILD_EXIT`, `CLI_CANCELLED`, `CLI_CLEANUP`. A child exit gets a neutral summary; lock/cache advice requires direct preflight evidence, not any nonzero child.

CLI-owned status remains 0 success/help, 1 preflight/generation/report failure, 2 usage, 125 spawn/supervision/cleanup failure. Native Cargo/application exits and OS cancellation keep their identity after verified cleanup; these may overlap CLI-owned numeric values. No universal POSIX/Windows code mapping, guessed cause, automatic repair/retry or process-name-based kills. Preserve Linux process groups/subreaping/signal re-raise and Windows suspended creation/Job assignment/native console behavior; deliberately escaped daemons remain unsupported.

Doctor stays environment-only: Rust/Cargo 1.99+ identity and compiled new-filesystem capability. Report app/framework/settings/assets/generator/console/execution readiness as not checked; no Node probe requirement. Version probes are fixed trusted local Rust/Cargo commands with no shell, null stdin, installer/update/online-resolution disabled. Bound each to 3 seconds/64 KiB combined output, drain both streams, terminate/reap within an additional 2-second cleanup budget; failed/uncertain cleanup cannot pass. Render only validated identity or fixed errors, never raw probe output.

Doctor-only `--json` emits one UTF-8 object/newline on a valid request, including probe failures; no progress/raw logs. Usage failures leave stdout empty with safe stderr (exit 2). Report-construction failure leaves stdout empty (exit 1, or 125 for cleanup failure). Schema 1 requires `schema_version`, `cli_version`, `scope="environment"`, `completed`, `ok`, and `diagnostics`; each diagnostic requires `code`, `origin`, `capability`, `status`, `message`. Stable codes include `DOCTOR_RUSTC_VERSION`, `DOCTOR_CARGO_VERSION`, `DOCTOR_FRAMEWORK`, `DOCTOR_NEW_PLATFORM`, `DOCTOR_APP_READINESS`, `DOCTOR_ASSETS`, `DOCTOR_BUILD_DEV`, `DOCTOR_CONSOLE`, `DOCTOR_GENERATOR`; retained code spelling does not preserve the removed dev command.

Statuses are pass, fail, unsupported, not-selected, not-checked. Required environment checks must pass for ok; uncovered capabilities never count as PASS. Completed means the scoped attempt finished, not complete app qualification. Consumers match codes, ignore additive fields/codes, and refuse unknown schema/status semantics. Removed/type/meaning/success changes require a schema increment; today's unversioned JSON is intentionally replaced. Human messages/array order are not machine identity.

## 7. Intentional migration

Rows describe the intentional migration. Current evidence and uncovered qualification are recorded separately in the implementation record. No aliases are selected; migrate current instructions, templates, fixtures and real callers together. Historical PASS and original failures remain unchanged. New generated content/layout/settings and execution effects are explicitly distinguished from renaming.

| Previous spelling/behavior | Selected target | Change and reason | Affected surfaces / future proof |
|---|---|---|---|
| `--cloud` | `new --service` | Scaffold spelling; preserve bind/health/shutdown scope | Parser/help, cloud templates/tests, generator fixtures; payload/safety equivalence for naming slice |
| `--with-config` | `new --settings` | App-runtime settings opt-in; CLI public settings are separate in responsibility, same files/mechanism | Config template/types/tests, shared settings adapter; exact new namespace/type behavior |
| `--with-tailwind` | `new --tailwind` | Creation convenience; standalone or official Vite plugin | Tailwind/Vite tests, instructions; no hidden install/build |
| `--assets vite`, `--with-vite-dev`, proposed `--vite build/dev` / `--dev-bridge` | Boolean `new --vite`; settings decide later actions | Intentional complete scaffold/layout/settings change, not same-output alias | Parser/template/asset/dev tests, qualified source fixtures; full Vite output and disabled settings cases |
| `dev` | `run` | Rename execute action; separately gated optional frontend supervision changes effects | Main/parser/app/process tests/docs; Rust-only equivalence and real two-tree lifecycle separately |
| Rust-only build | Conditional `cli.vite.auto_build` integration | Explicit new frontend execution effect, independent of auto_start | Frontend failure/skip/order/env/cleanup tests; no false complete-build result |
| Proposed Cargo Vite metadata and YETLET_VITE_AUTO_* names | `[cli.vite]` in existing settings; APP_CLI__VITE__... | Supersede unimplemented parallel settings proposal | Shared loader/types/files/ENV mapping; no metadata fallback or second parser |
| Flat Vite files; disabled frontend dotenv discovery | Configurable root, default frontend/; confined frontend .env* | Output/path/file-access change | New root/native path tests; no repo-root dotenv or backend-secret leakage |
| `--replicas` | `cli.generate.k8s.replicas` | Persistent generator choice; default/range retained | K8s/settings fixtures and boundary counts; no cluster action |
| `--framework-path` | No target source flag; qualified release versions and Cargo-owned local dependency overrides | Option removal and later dependency/source representation change | Scaffold/doctor/package/lock/export tests; release/generator qualification, no hidden lookup |
| `--dir` | `--parent` | Same parent/default meaning | Creation tests; no-overwrite/race/traversal/owned cleanup preservation |
| Required manifest | Explicit override, otherwise cwd/Cargo.toml | Small project-selection default change | All execution/generator parser/preflight tests; no ancestor/workspace guessing |
| Existing release/output/image/json | Retain narrow operation/format roles | No deployment/network/source-safety expansion | Existing image/count/publication/native exit restrictions and updated consumers |
| Root-only help/no version, separated values/NAME-first | Scoped discovery/version and uniform grammar | Deliberate parser compatibility breaks | No-project/no-tool/no-effect discovery and literal/equal/duplicate/unknown tests |
| Unconditional cache hint/unversioned doctor | Neutral native attribution/versioned scoped report | Diagnostic/schema change | Exit 23/native signals, redaction/bounds/schema/stream tests |

## 8. Required behavior and qualification

| ID | Required result; see implementation record for actual evidence |
|---|---|
| AC-01 | Scoped exclusive help/version work without settings/project/tools/effects; usage is scoped and safe. |
| AC-02 | Boolean creation switches, literal/equal value options and NAME placement normalize deterministically; removed/unknown/duplicate/empty/misplaced arguments refuse before effects. |
| AC-03 | Explicit/default manifest, common public settings root and project-relative Vite root have the documented bases; unsupported workspaces, partial/malformed settings and unsafe paths fail. |
| AC-04 | Plain SSR has no Node requirement; old apps without CLI settings retain disabled behavior. New base/profile declarations and strict app/CLI types accommodate cli without disabling unknown-field/type validation. |
| AC-05 | Boolean --vite generates complete configured frontend/ output; no creation-time tools/installs. Dist, CSP/root-only bridge and package scripts retain qualified limits; old output/metadata/layout acceptance is not reused. |
| AC-06 | Generators retain current source/lock/config/asset/export/refusal boundaries. New settings/layout/release representations require separate acceptance; replicas setting defaults/range are exact. |
| AC-07 | Doctor scope/schema/status/streams/bounds/redaction are implemented with no app or Node-readiness PASS; real OS probe cleanup failures remain failures. |
| AC-08 | run selects Vite through auto_start and Rust watch through cli.rust.watch; watch shares run and enables Rust watch for this invocation after strict settings validation, with unchanged Vite selection; build selects Vite through auto_build and never starts watch. Independent flags/env/profile combinations include disabled/skipped branches, no installs, ordered frontend-before-Rust builds and stop-on-frontend-failure. |
| AC-09 | Real Linux and Windows evidence proves owned Rust/frontend startup, native status/cancellation and cleanup of both trees; attached-console/headless limits remain honest. |
| AC-10 | Current usage/evidence remain source-scoped; target features are not called implemented. No sandbox, release, general proxy/auth or production qualification claim. |
| AC-11 | No CLI framework-source flag or execution metadata config; dependency versions/local overrides remain Cargo-owned with explicit lock and release gates. |
| AC-12 | No mandatory Vite mode or repeated runtime tool options; the retained option set matches the creation/execution policy and emits actionable migration hints. |
| AC-13 | Shared Loader merge/key/type/profile semantics, APP_CLI overrides and CLI-only typed projection work without loading backend Development secrets or exposing APP credentials to Vite. Earlier hyphenated/unimplemented names are not silently accepted. |
| AC-14 | Custom/default frontend root confines frontend .env* and subprocess cwd; no repo-root/ancestor dotenv scan, symlink escape or backend secret forwarding. |

## 9. Remaining work

The user explicitly authorized this CLI implementation and independent review. This authorization supersedes the former documentation-only next-step proposal. Current source-matched evidence is in [YT-102 implementation](YT-102-CLI-IMPLEMENTATION.md); prior design reviews remain historical evidence.

Selected native Windows GNU startup/cancellation/cleanup has [direct bounded evidence](YT-102-WINDOWS-VITE-VERIFICATION.md). The current generator local-source extension has its own [compatibility evidence](YT-102-GENERATOR-COMPATIBILITY.md); additional platform/runtime qualification, registry-only source/export representations, usable registry templates and release qualification remain open. Rust watch/recompile/restart is an explicit settings-controlled mode with the boundaries above; arbitrary proxy/auth/actions/multi-page bridging and alternative provider registries stay deferred. YT-102's broader journey audit, measured cold/warm/change-feedback comparison and prioritized DX roadmap remain Todo. YT-103 rolling asset retention and M20/YT-020 release remain separate. No commit, publication or deployment is implied.
