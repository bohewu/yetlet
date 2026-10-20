# YT-102 — Developer workflow audit and improvement roadmap

Date: 2026-10-20. Source HEAD: `649be69`. This completes the owner-selected
research scope after the separately authorized CLI, watch, HTTPS, signing and
browser-starter implementations. It changes documentation only. The prior
reviewed YT-095 research remains uncommitted and is preserved separately.
Independent Reviewer `/root/yt102_dx_audit_review_oct20` PASS has zero issues or
required followups; YT-102 is Done at research-only scope.

## Conclusion

Keep the current create-and-execute CLI and settings-first behavior. Retain
Cargo for compilation/testing, the editor/rust-analyzer for debugging,
Watchexec for file notifications and Vite for frontend packaging/HMR. The
current `run`/`watch` supervisor already supplies the missing ownership boundary;
another dev command, dependency installer or tool registry is unnecessary.

The most concrete remaining code candidate is a bounded study of Linux
completion latency: a warm CLI build spends about half a second beyond Cargo,
and the current supervisor unconditionally waits a 500 ms cleanup grace after
natural exit. Preserve all descendant, cancellation and terminal guarantees
before considering an early-completion path. This is a measured usability cost,
not a newly discovered cleanup defect or permission to weaken cleanup.

Other practical costs are explicit first-use prerequisites and the lack of a
read-only view of effective project settings/stages. Prioritize a short setup
guide before another command or a dependency refactor. Usable registry-backed
scaffolds remain a release gate, not a task to hide with a framework-path flag.

## Method and evidence boundaries

The [accepted contract](CLI-CONTRACT.md), [current guide](../tools/yetlet-cli/README.md),
parser, main dispatcher, settings projection, scaffold templates, doctor,
frontend environment filtering, native process adapters and existing focused
test definitions were inspected. [ADR 0005](adr/0005-opt-in-capability-modules.md)
and [ADR 0006](adr/0006-handler-first-mvc-optional.md) still apply. No source,
dependency, settings schema, CLI grammar or generated template was changed.

Current Ubuntu observations used a freshly rebuilt CLI and a disposable
CLI-created native app. Its two unpublished Yetlet dependencies received
explicit compatible Cargo path entries; an offline lock was prepared separately.
Only synthetic handler text changed inside that disposable app. No package
download, Node install, external service, Docker, WebCodex or Windows session
was started. Existing Windows and frontend evidence was inspected and retained;
it is not presented as a rerun of every current source byte.

Local observations, literal argv/logs, fixture source/lock, cleanup and preserved
input hashes are under `/tmp/yetlet-yt102-dx-20261020-4k24ykhn/`:
`initial.json`, `observations.json`, `observe.py`, `bootstrap.json` and
`preserved-inputs.json`. These are small direct observations, not a new test
framework or portable qualification runner. No credentials are used.

## Journey map

| Journey | Current path and actual user cost | Evidence and interpretation |
| --- | --- | --- |
| Discover and create | Scoped help/version; `new NAME`, optional `--template browser`, `--settings`, `--vite`, `--tailwind`, `--service`. Creation writes files only | Current parser/templates and actual help/new below. Browser template selection no longer depends on a parent environment variable; [YT-104](YT-104-BROWSER-STARTER.md) owns that amendment |
| Prepare dependencies | Native app needs an explicit compatible dependency source, lock and cached crates. Optional frontend/WASM/signing paths require their selected tools | Generated READMEs and actual offline lock/build below. No implicit install or lock refresh is appropriate. Registry-only first use remains unqualified |
| Diagnose environment | `doctor [--json]` checks Rust/Cargo identities and compiled filesystem capability. Schema 1 explicitly reports app/assets/execution as not checked | Actual report below and [doctor evidence](YT-102-DOCTOR-VERIFICATION.md). `ok=true` means environment scope only; it does not guarantee app, Node, OpenSSL or asset readiness |
| Native run/build/stop | `build` uses frontend-before-Cargo when selected; `run` uses frozen Cargo and settings-selected frontend startup. Ctrl-C/termination stops owned trees | Actual plain-native observations below; prior [Windows pair](YT-102-WINDOWS-VITE-VERIFICATION.md) and [Windows build diagnosis](WINDOWS-CLI-125-VERIFICATION.md) retain their native scope and original failures |
| Rust edit feedback | `run` honors `cli.rust.watch`; `watch` enables it for this invocation. External Watchexec emits events; Yetlet owns Rust generations | Actual `watch` edit below and [integration](YT-102-RUST-WATCH-IMPLEMENTATION.md). Rust RAM is recreated, compile failure can leave the backend unavailable, and Vite is kept across Rust restarts |
| Frontend edit feedback | `new --vite` includes runtime settings. `enabled` declares integration; independent `auto_start`/`auto_build` select actions. Default root is `frontend/` | Source and [stateful integration](YT-102-STATEFUL-HMR-IMPLEMENTATION.md), [Windows CSS](YT-102-WINDOWS-HMR-VERIFICATION.md) and [JS/TS follow-up](YT-102-WINDOWS-JS-TS-VERIFICATION.md). Vite owns HMR semantics; no new HMR feature matrix is needed for this audit |
| Browser-only application | `new tasks --template browser --tailwind`, prepare dependencies, `build`, then `run` existing static output; stop with Ctrl-C | [YT-104 proof](YT-104-BROWSER-STARTER.md) covers CLI/static/browser and selected Windows GNU behavior. Rust watch and native TLS are deliberately refused; browser-source changes require rebuilding in this starter |
| Test and debug | Use native Cargo test/check/clippy, npm typecheck where selected, and editor debugging/runnables | Existing Starter/Admin/browser README commands. `yetlet build` does not claim tests ran. No Yetlet test wrapper or debug-adapter integration exists or is necessary for this contract |
| Configure and handle secrets | Shared Loader base/profile/declared ENV semantics, backend-only Development secrets, optional narrow SecretProvider bindings | [Configuration](CONFIGURATION.md), [secret integration](SETTINGS-SECRET-PROVIDER.md) and current CLI projection. `--release` does not select Production; CLI policy changes require a new invocation |
| Preview deployment | Qualified `generate docker/compose/k8s` emits configuration; browser dist can be hosted independently | [Current generators](YT-102-GENERATOR-COMPATIBILITY.md), [Windows generator follow-up](YT-102-WINDOWS-GENERATOR-VERIFICATION.md), [YT-103](YT-103-ROLLING-ASSET-VERIFICATION.md) and [YT-104](YT-104-BROWSER-STARTER.md). Generation is not source export, image build/push or deployment; Vite preview is local checking |

Frontend-disabled native apps do not require Node. `auto_start=false` preserves
manual/two-terminal operation; it does not become an error solely because Vite
is installed. For custom frontend roots, direct npm commands must carry the
documented project-root context for Tailwind. CLI and backend public settings
can share files without loading backend Development secrets into tool decisions.

## Current Ubuntu measurements

Host/tool identities: Linux x86_64, Rust/Cargo 1.99.0, existing Watchexec 2.8.0.
The same-host Cargo cache was populated. The first build used a new empty
task-owned target directory, not a clean machine or a download. Native default
debug profile, no frontend, Production settings, null stdin and loopback HTTP
were selected. Each operation has **one sample**; no percentiles or speed ranking
are inferred. Readiness polling adds up to 25 ms plus HTTP overhead. The direct
Cargo lane ran after the CLI lane had warmed the same target.

| Observation | Wall seconds | Native result |
| --- | --- | --- |
| CLI help | 0.001 | Exit 0 |
| Environment-only doctor | 0.480 | Exit 0, scope=environment, completed/ok true; app/assets/execution not checked |
| CLI native creation | 0.002 | Exit 0; no lock/build/listener |
| Explicit offline lock setup | 0.064 | Exit 0; separate from creation |
| CLI build, empty app target / populated crate cache | 6.377 | Exit 0 |
| CLI build, warm target | 0.565 | Exit 0; Cargo log reports 0.01 s |
| Direct `cargo build --frozen`, same warm target | 0.032 | Exit 0 |
| Direct `cargo run --frozen`, warm target to HTTP | 0.035 | HTTP 200; later group SIGINT produced -2 |
| CLI run, warm target to HTTP | 0.052 | HTTP 200 |
| Scripted edit + CLI stop + CLI run to changed HTTP | 1.574 | Changed text served; includes no human reaction time |
| CLI watch, warm target to HTTP | 0.052 | HTTP 200 |
| File write under running CLI watch to changed HTTP | 0.989 | Changed text served |
| CLI run stop / watch stop | 1.015 / 0.514 | Programmatic SIGINT to CLI; both -2; port closed |

`-2` is Python's native SIGINT termination representation, not exit 0. Direct
Cargo was signalled as a process group; CLI was signalled at its parent PID.
This null-stdin observation does not replace interactive terminal qualification
or normalize older PTY cancellation results. The lock was unchanged; no forced
cleanup was needed, the selected port closed and the owned target was removed.
Fixture inputs and logs remain for inspection.

The [Linux foreground supervisor](../tools/yetlet-cli/src/platform/process/foreground.rs)
declares `GRACE=500ms`; `Owned::grace` waits until its deadline. Single-process
execution invokes it after natural Cargo exit, and cancellation also invokes a
preceding grace. This source chain explains a fixed contribution consistent
with the measured build/stop costs. It does not assign every measured
millisecond to that cause or justify skipping descendant cleanup.

An actual prerequisite failure also occurred: rebuilding the CLI without an
OpenSSL development prefix exited 101 at `openssl-sys`. Reusing the already
prepared prefix via `OPENSSL_DIR` and `OPENSSL_STATIC=1` succeeded; nothing was
installed or changed globally. The CLI currently depends on `yetlet-signing`
unconditionally, including for plain CLI builds. The current guide documents
OpenSSL, but its short initial build command assumes the prerequisite is ready.
This is an onboarding cost, distinct from generated plain apps needing OpenSSL.

### Historical comparison without invented parity

The [external Watchexec Linux trial](YT-102-RUST-WATCH-POC.md) measured initial
warm-cache startup 0.361 s, edit feedback 1.576 s and recovery 0.573 s; its
initial cached dependency build was 5.940 s. The [Windows external trial](YT-102-WINDOWS-RUST-WATCH-VERIFICATION.md)
measured an edit at 3.686 s and recovery at 3.349 s. These have different dated
sources, hosts, output/logging and timing boundaries. They demonstrate available
workflows, not a Linux/Windows or external/integrated performance ranking.
No new frontend build/HMR latency, Windows cold/warm run, clean-network setup,
load, browser or profiling result is claimed. Those missing measurements limit
comparisons; they do not invalidate the retained functional PASS results.

## Evidence reconciliation

- The [2026-10-18 contract audit](YT-102-CLI-CONTRACT-AUDIT-20261018.md) remains
  historical, including its concrete F-01 linked-dotenv failure. The separately
  reviewed [dotenv correction](YT-102-DOTENV-BOUNDARY-FIX.md) and current
  `validate_frontend_dotenv` / generated loader checks address that path.
  The generated `selected-dotenv.mjs` still matches its recorded tested hash;
  five other recorded files have since changed. No whole-current-HEAD dotenv
  recertification is implied.
- The old missing Windows generator case was subsequently completed by the
  [native GNU follow-up](YT-102-WINDOWS-GENERATOR-VERIFICATION.md). Its recorded
  `docker.rs`, `k8s.rs` and selected generator fixture hashes still match;
  `compose.rs` and the process adapter have later changes. Preserve the original
  failures, source boundary and later fixes rather than claim current all-file
  parity. No Windows execution was started for this research.
- Existing integrated-watch, HTTPS, browser-template and real Admin trials have
  their own scopes. Additive later source and a passing research review do not
  turn them into a new complete Linux/Windows matrix. Owner-waived routine
  IME/assistive-technology and Windows OIDC matrices remain closed.
- The original `dev` name, separate Vite creation flags and framework-path
  proposals in older research are superseded by current `run`/`watch`, boolean
  `--vite`, `--template browser` and Cargo-owned dependencies. Their historical
  existence is not a missing feature or a compatibility promise.

## Prioritized gap matrix and candidate slices

These are research recommendations, not authorization to implement them.

| Priority / gap | Concrete evidence and practical effect | Smallest next action / focused proof |
| --- | --- | --- |
| P2 / completion latency | Warm CLI build 0.565 s versus later direct Cargo 0.032 s; unconditional 500 ms natural-exit grace in source. Short successful commands incur visible fixed waiting | First selected code candidate: study a safe natural-exit completion path using existing ownership state. Keep resistant descendants, exit attribution, cancellation and PTY restoration tests; compare a few matched warm builds. Accept the delay if earlier completion cannot preserve cleanup; do not remove grace globally |
| P2 / first-use prerequisites | CLI rebuild failed until an existing OpenSSL prefix was selected; generated apps need separately prepared source/locks and optional frontend tools | First documentation candidate: one compact native/Vite/browser setup checklist and honest doctor scope, linking existing guides. Treat optional signing feature/dependency changes as a later measured proposal, not an immediate packaging refactor |
| P2 / effective project selection is opaque | Doctor checks no project; strict settings projection returns one fixed invalid-settings error; selected `target`, profile/root and skip reasons must be traced through files/ENV | Document the public decision table first. If recurring support cases justify code, propose an explicitly requested read-only project report reusing preflight/projection, with allowlisted fields and no secret-file access or child startup. No command spelling or JSON/schema change is selected here |
| P2 / configuration failures are too broad for troubleshooting | Missing profile, unknown key/type and unsafe root can collapse to the same fixed settings error | Consider closed safe categories such as profile, public CLI schema and path stage using existing errors. Prove synthetic failures retain redaction and refuse before effects; never print TOML, ENV dumps or provider details |
| Release gate / fresh registry onboarding | New scaffolds intentionally use unpublished exact declarations, so `new` alone cannot yield a registry-buildable application | Keep explicit Cargo overrides for local research. YT-096/YT-020 must qualify supported published dependencies before presenting registry quick start as ready; no source locator flag or implicit resolver |
| Accepted limits / scope | Fixed loopback frontend port, initial dist prerequisite, browser explicit rebuild, limited bridge and no universal IDE/deploy wrapper | Document and keep current behavior. Expand only for a concrete consumer; no general proxy, hot-patching, auto-install or cloud operation follows from this audit |

## DX extension contract

This records a decision boundary using current mechanisms, not a plugin API:

1. **Creation versus execution:** template selects an application shape at
   `new`; existing commands execute it. Recurring policy belongs in the existing
   typed settings. Add a flag only for an actual frequent invocation choice.
2. **Settings authority:** reuse Loader precedence, strict declared keys and
   public projection. No Cargo metadata execution settings, second config
   language, implicit ancestor search or backend Development-secret reads for
   tool decisions. Normal profile selection and Cargo optimization are distinct.
3. **Dependency preparation:** Cargo/npm and the operator own sources, locks,
   installs and tool versions. CLI operations use literal fixed argv; adding a
   tool does not authorize a download or a shell command from settings.
4. **One process owner:** each native adapter supervises the trees it starts,
   attributes failures and proves bounded cleanup. External watchers supply
   events; Vite owns frontend HMR. Preserve manual/two-terminal fallback.
5. **Frontend boundary:** frontend root, dotenv confinement and child ENV
   filtering apply together. Browser output and `VITE_*` values are public.
   Runtime Node is not required for native production asset serving or static
   dist hosting. Package scripts still have effects; this is not a sandbox.
6. **Assets and optional packages:** reuse the existing app-owned asset manifest
   and Vite adapter, or a separately selected producer. Chart.js/CodeMirror use
   private app adapters, not a new CLI registry. Core/browser do not acquire
   native secret/signing/service capabilities automatically.
7. **Diagnostics:** preserve native exit/signal information, bounded stage
   attribution and secret-safe summaries. Only doctor currently has its scoped
   versioned JSON contract; human logs are not a new machine protocol.
8. **Testing and IDEs:** pass normal Rust modules/constructors/traits to tests;
   use Cargo and the editor directly. Existing per-app request tracing and
   resource drains remain application responsibilities; CLI does not invent a
   persistent runtime circuit or mandatory debug service.
9. **Deployment and compatibility:** generators publish qualified configuration
   without executing it. New producers, templates, schema meanings or supported
   platforms need their own bounded contract/proof. Preserve accepted evidence
   and source boundaries; research Done never grants publication or deployment.

Primary references checked on this research date: [Cargo run and frozen execution](https://doc.rust-lang.org/cargo/commands/cargo-run.html),
[Watchexec 2.8.0 release](https://github.com/watchexec/watchexec/releases/tag/v2.8.0),
[Vite build/preview deployment guide](https://vite.dev/guide/static-deploy.html)
and [rust-analyzer editor support](https://rust-analyzer.github.io/book/vs_code.html).
The Cargo reference confirms frozen means locked plus offline; Vite identifies
preview as local checking rather than a production server. These do not qualify
Yetlet's integration or imply that another tool's entire feature set was tested.

## Research closeout and limits

The journey map, measured costs, reused-tool comparison, prioritized proposals
and extension boundary satisfy the selected research deliverables. Full source
tests and platform/browser suites are not rerun for this documentation-only
audit. `cargo xtask check docs` passed with exit 0 (links, Markdown structure,
whitespace and backlog DAG); `git diff --check` passed with exit 0. The independent
reviewer verified source/observation claims, preservation and cleanup, and passed
its own whitespace check. Research-only closure is approved.
No proposal in this report is implemented or required to mark research Done.

If selected next, use the existing Linux supervisor tests for the bounded
completion-latency slice; profiling beyond the observed fixed wait is premature.
YT-096 remains the separately unblockable release-candidate decision, with its
own owner-approved source/packages and release/security checks. The prior
YT-095 study and this research are not implicitly committed or published.

The subsequently selected [Linux natural-completion slice](YT-102-COMPLETION-LATENCY.md)
implements the first bounded latency candidate with focused correctness and
timing evidence. It preserves this dated research and its original measurements;
independent implementation Reviewer PASS completes that bounded slice with zero
issues or required followups.

## Selected first-use documentation slice (2026-10-20)

After committing the reviewed research, completion-latency implementation and
Windows verification at `ec31c63`, the owner selected the next compact
documentation candidate. The CLI guide now places a
[first-use prerequisite table](../tools/yetlet-cli/README.md#first-use-prerequisites)
before its first build command: native/Vite/browser tools, the CLI's own
OpenSSL requirement, separately prepared dependencies/locks/assets, optional
watch/HTTPS tools, settings-enabled startup secrets and the limited doctor
scope. The Windows notes distinguish native MSVC single-Cargo evidence from
the still compile-only MSVC Vite-pair scope.

The guide was checked against the current CLI dependency manifest, generated
template READMEs/package engines, browser WASM build script and existing
native records. This changes documentation only; accepted process/browser/
Windows tests and performance measurements are reused, not rerun. No command,
settings schema, dependency, installer or diagnostic behavior changes.
`cargo xtask check docs` and `git diff --check` pass with exit 0. The first
review found that raw pipes in the Node version expression truncated the
rendered Vite row. Replacing them with plain-language alternatives fixes that
row; the existing Markdown table parser confirms all six rows, including the
header, have three complete cells. Independent Reviewer
`/root/yt102_first_use_guide_review_oct20` followup PASS has zero issues or
required followups; the original rendering failure remains recorded here.
The next separately
selectable DX candidate is documenting effective project/settings selection;
this slice does not select a new report command or error schema.
