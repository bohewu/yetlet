# YT-097 — Opt-in CLI configuration scaffolding

## Status and source

Implemented in the existing Yetlet repository checkout on
`feat/yt090-configuration`, based on
`1c0294fe1a49adf9ce81047a5e695bd837f0bb98`, on 2026-10-17.
**Done at bounded unpublished optional CLI + Linux/Windows native verification scope after independent changed-source Reviewer PASS.** Following
the initial sandbox-restricted implementation run, the parent obtained actual
Linux SSR/CSP/health/signal execution and native GNU/MSVC source-bound tests
(see later qualification). Earlier YT-090/YT-089 reviews do not accept this
changed source. YT-091 stays reserved for Tailwind and YT-020 remains Blocked.
No stage, commit, push, public publication or deployment is authorized.

Authority: the user requested one integrated YT-097 implementation and the safe
tracked synthetic YT-090 example filename change. ADRs [0005](adr/0005-opt-in-capability-modules.md)
and [0006](adr/0006-handler-first-mvc-optional.md) retain explicit optional
composition, native State/Arc, handler-first SSR and no DI container.

## Changed surface and contract

- CLI `src/lib.rs`, `src/main.rs`, `src/scaffold.rs`, `src/framework.rs`:
  explicit unique `new NAME --framework-path PATH [--dir DIRECTORY] [--cloud]
  --with-config`, opt-in local config-package inspection and separate templates.
- `src/platform/fs/mod.rs`: only `src/config.rs` joins `src/main.rs` in the
  fixed `src` path allowlist. Existing Linux and Windows native writers,
  identity/ACL/reparse/no-follow/no-replace/rollback logic are unchanged.
- New `templates/config/{main.rs,cloud.rs,config.rs,base.toml,README.md}`:
  generated root base/finite profile TOML and empty synthetic `.env.example`,
  explicit unpublished path dependency, pinned Serde derive/Secrecy serde,
  strong app-owned settings, required SecretString, bounded fixed startup errors,
  validation before listener and native State/Arc. No Serialize or Debug settings.
- `tests/config.rs`, `tests/cloud.rs` and parser/unit tests: opted-in semantics,
  default bytes, filesystem refusal, offline generated apps, fail-closed deployment
  generation and explicitly selected native HTTP coverage.
- CLI README, configuration guide, backlog/roadmap and this record: operator
  lock/environment/PowerShell/cloud boundaries and pending acceptance state.
- YT-090 tracked synthetic `examples/settings/dotenv-sample.txt` was renamed in
  the working tree to `.env.example`, with guide/verification references updated.
  Its bytes are unchanged. The old untracked sensitive-looking path and existing
  artifacts were not opened, inspected, changed, renamed or deleted.

The original `create`/`create_cloud` implementations and no-config templates
remain unchanged. Full generated Cargo.toml/.gitignore bytes and main/README
bytes are asserted for both original profiles, with exactly the original files.
The config profile alone generates root settings and `src/config.rs`, without
an additional settings directory. No generator invokes Cargo, installs tools,
creates a lock, interpolates OS values, discovers dotenv, generates keys or
loads secrets. Test compilation/lock generation is a separate explicit action.

Settings resolve from absolute build-time `CARGO_MANIFEST_DIR`, independent of
runtime cwd; relocation requires explicit provisioning/rebuild. `APP_ENVIRONMENT`
is finite and defaults to Production. The YT-090 loader merges base, selected
overlay, explicitly selected Development file, then typed `APP_*` OS overrides.
Only Development inspects `YETLET_DEV_SECRETS`; selected paths must be absolute.
Other profiles do not inspect that path. No `.env.local` autoload occurs.
Blank required token, invalid host/port/public message and loader errors fail
before listening. Strings such as `000123` keep their exact shape.

Cloud config keeps the original bounded host policy, local health and
pre-registered signal/graceful-drain semantics in separate source. Existing
Docker/Compose/Kubernetes exact-app allowlists are unchanged: with a real valid
app lock present, all three reject the new config cloud app, exit 1 and create
no output. Supporting its container configuration/secret delivery requires a
separate reviewed design. The retained cloud marker is not source attestation.

## Actual local commands and outcomes

The original implementation leaf used cached dependencies and offline/locked
Cargo commands; it did not provision toolchains, fetch dependencies, access
Windows or deploy cloud resources. Later parent-operated Windows tests explicitly
fetched missing public crates into their existing user Cargo cache, then ran
source-bound compile/tests in offline/locked mode as documented below.

| Command | Actual result |
|---|---|
| `cargo test --offline --locked --manifest-path tools/yetlet-cli/Cargo.toml` | Exit 0; 61 tests passed, 2 explicit generated-app tests ignored in the default suite; Windows-gated binaries ran 0 tests |
| `cargo test --offline --locked --manifest-path tools/yetlet-cli/Cargo.toml --test cloud --test config` | Exit 0; original default byte parity and 2 config safety tests pass; 2 explicit tests ignored |
| `cargo fmt --manifest-path tools/yetlet-cli/Cargo.toml -- --check` | Exit 0 |
| `cargo clippy --offline --locked --manifest-path tools/yetlet-cli/Cargo.toml --all-targets -- -D warnings` | Exit 0 |
| `cargo test --offline --locked --manifest-path tools/yetlet-cli/Cargo.toml --test config generated_config_apps_offline_contract -- --ignored --nocapture` | Exit 0; actual generated SSR and cloud applications independently lock/check/test/Clippy/build offline; 3 generated unit tests per app, 6 bounded real startup failures per binary, all child processes reaped; all 3 deployment generators reject config cloud with no output |
| `cargo xtask check docs` | Exit 0; 75 documents, 91 backlog items; language/links/structure/DAG PASS |
| `cargo xtask check samples` | Exit 0; 12 sample catalog entries PASS |
| `git diff --check` | Exit 0 |
| `cargo test --offline --locked --manifest-path crates/yetlet-config/Cargo.toml` | Exit 0; 16 loader tests and 1 doctest; unchanged runtime API |
| `YTCONFIG_ENVIRONMENT=Development cargo run --offline --locked --manifest-path crates/yetlet-config/Cargo.toml --example typed -- --check --dev-secrets crates/yetlet-config/examples/settings/.env.example` | Exit 0; fixed valid summary, no listener |

The explicitly requested generated-app check performs, per profile:
`cargo generate-lockfile --offline`, `cargo fmt --all -- --check`,
`cargo check --offline --locked --all-targets`, `cargo test --offline --locked`,
`cargo clippy --offline --locked --all-targets -- -D warnings`, and on Linux
`cargo build --offline --locked`. The lock bytes remain unchanged afterward.
Generated tests cover finite profiles, exact string secrets, required token,
bounded host/port, unknown/invalid overrides, explicit Development file and OS
priority, and non-Development ignoring a selected missing dev file. Real binary
failure cases cover empty/blank token, unknown key, invalid selector, missing
selected Development file and relative selected path from a different cwd.
Errors are fixed/redacted and shorter than 256 bytes.

Logs for parent inspection (local `/tmp`, not committed artifacts):
`/tmp/yt097-cli-tests.log`, `/tmp/yt097-focused-tests.log`,
`/tmp/yt097-cli-clippy.log`, `/tmp/yt097-generated-apps-final.log`,
`/tmp/yt097-config-runtime-tests.log`, `/tmp/yt097-safe-example.log`.

## Preserved failure and limits

Initial implementation check: `product_failure`, exit 101, missing closing
brace in new SSR template and missing `with_config` in an existing parser test.
The smallest correction used **1 of 2 product repair attempts**. Subsequent
focused checks passed. No security policy or YT-090 runtime API repair occurred.

Explicit HTTP attempt:
`YETLET_TEST_NATIVE_HTTP=1 cargo test --offline --locked --manifest-path tools/yetlet-cli/Cargo.toml --test config generated_config_apps_offline_contract -- --ignored --nocapture`
returned exit 101 at `TcpListener::bind("127.0.0.1:0")` with OS code 1,
`PermissionDenied`, `Operation not permitted`, after first-profile offline
checks passed. This is **operational_failure**, not evidence of a product
network defect. Log: `/tmp/yt097-generated-apps.log`. One bounded operational
adaptation reran the existing test without the optional HTTP flag, completing
both offline profiles and no-listener startup failures. No permissions were
escalated and no alternate validation infrastructure was created.

At the initial sandboxed executor checkpoint, actual HTTP, SSR response/CSP,
health, readiness and Unix signal/drain were **Not verified** because its
loopback bind was denied by that sandbox. This was superseded by the parent's
**actual same-source Linux native HTTP execution** below. Actual native Windows
GNU/MSVC CLI and generated-app build/test evidence also followed. These do not
establish Windows interactive console/CTRL-C or ACL adversarial coverage for this
new configuration app, a browser-specific CSP gate, any production image/cluster
deployment, public release or general production qualification.

Generated test applications/builds/dev fixtures are owned by tempfile and were
removed on return, including the failed HTTP attempt. Bounded startup children
were waited/reaped by the test and its drop guard. No task server or background
watcher remains. Existing artifacts were preserved. Git index and HEAD are
unchanged; new source/example files remain unstaged for the parent.

## Parent-operated Linux and Windows source-bound qualification

After the executor's sandboxed loopback denial, the parent invoked the unchanged
existing opt-in integration test on the Ubuntu WebCodex Runner with
`YETLET_TEST_NATIVE_HTTP=1`, using the actual modified checkout. Job
`wc_job_1LrjHX_s3NAmfvRt` completed **exit 0**. Both configured SSR and cloud
apps passed real HTTP 200, strict CSP header, escaped user-visible HTML without
secret disclosure, three finite Production/Staging/Development journeys and
owned cleanup; cloud additionally verified `/health/live`, `/health/ready`
(200), SIGTERM/SIGINT and closed listeners. Six actual bounded redacted startup
failures per generated app passed. The test generated locked standalone app
builds, tested three app-owned typed-setting assertions per app, verified
Clippy, and explicitly rejected Docker/Compose/Kubernetes generation for the
configuration cloud app, with no generated output.

The parent then exported a sanitized exact-source validation ZIP containing
528 source files and per-file SHA-256 entries, excluding unrelated untracked
`artifacts/` and the sensitive-looking old untracked dotenv path. The initial
archive and Windows attempts remain historical (Windows source spelling
fixture used a verbatim-path result rejected by the existing NTFS adapter; the
GNU process initially lacked its existing binutils activation). The **r2**
ZIP SHA-256 is `6274cafe18b3ed2ee3629889efc020d121c1f347edfaf0d00e6b2a1294d2f5ff`.
Relative to the first source-bound snapshot, **only**
`tools/yetlet-cli/tests/config.rs` changed: the Windows fixture now derives
two lexical parents from Cargo's already absolute `CARGO_MANIFEST_DIR` instead
of using `canonicalize()`'s unsupported verbatim `\\?\` spelling; the actual
native OS security adapter remains unchanged. The changed fixture passed Linux
format and focused tests before this archive.

Two separate Windows WebCodex Runners each rehashed all 528 extracted source
files, preserved their existing checkout, and used their native Rust 1.99.0
GNU/MSVC toolchain with task-owned temporary Cargo target directories. GNU
reactivated the previously installed user-owned MinGW tools only in the current
PowerShell process; no OS/global PATH changes or new toolchain install occurred.
Both first completed the native CLI `--all-targets` suite successfully, then
encountered a missing public `secrecy` dependency in their offline registry
cache. A separate explicit `cargo fetch --locked` of the isolated YT-090
crate's public dependencies populated the existing **user Cargo cache**; no
source or lockfile edits were made. The subsequent offline/locked commands all
passed, with task-owned source/target directories removed afterward:

| Native target | Runner job | Exact-source result |
|---|---|---|
| Windows GNU, Rust 1.99.0 | `wc_job_GyAVx5fxW8FrlAPA` | PASS, source hashes (528/528), generated SSR/cloud Cargo lock, check/test/Clippy and original generators' fail-closed rejection, CLI strict Clippy, YT-090 crate tests (15 integration, 2 example), cleanup |
| Windows MSVC, Rust 1.99.0 | `wc_job_AJQ5kSwsgBLg8aPo` | PASS, identical source hashes (528/528) and native generated-app, Clippy, YT-090 tests and cleanup |

The r2 snapshot records code/test content and the earlier documentation before
this qualification note. Neither Windows run claims this subsequent
*documentation-only* update or Windows console/production-certification
coverage. These checks do not publish a package, modify another checkout or
exercise production Kubernetes.

## Final independent reviewer acceptance

After native qualification, the parent computed a 20-path source/rename review
inventory containing 19 readable SHA-256 identities and the one intentional
tracked example deletion, retained in ignored
`target/m16/yt097/reviewer/review-inputs.json`. It independently confirmed
that all 12 config-related CLI executable/test/template inputs were byte-equal
to the exact Windows GNU/MSVC native-tested r2 source snapshot. Only
`docs/BACKLOG.md`, `docs/ROADMAP.md` and this verification document had changed
since the native snapshot, solely to record subsequent qualification.

Independent read-only role-directed Codex Reviewer
`wc_job_efQ_nYLzdZltPEjt` (`gpt-6-astra`, shared resolver-selected `high`)
terminated with **exit 0** and JSON `overall_status=pass`, `issues=[]`,
`required_followups=[]`. The reviewer inspected all 20 changed/deleted paths,
rechecked all 19 readable SHA-256 identities at start and end (zero drift),
validated safe `.env.example` byte-equivalent rename, existing no-config CLI
outputs, native filesystem writer isolation, opt-in secret/config semantics,
source/test claims and deliberate cloud generator rejection. The reviewer ran
no additional tests, performed no source writes and did not access the excluded
old untracked sensitive-named file. This was a **separate independent role-directed
process Job**, not a native managed `agent_type` or formal assurance. The verdict
covers the changed executable tree before the final documentation-only Done
status update, which does not modify source or tests.

**YT-097 Done** at the specified opt-in CLI, Linux real HTTP and Windows
GNU/MSVC source-bound native verification scope. Windows interactive console,
Windows hostile ACL race stress, production Kubernetes, an external secret vault,
public release and CI/CD activation remain out of scope. YT-091 Tailwind is
future work and YT-020 remains **Blocked**. The repository stays on the existing
feature branch with original HEAD; no stage, commit, push, merge, auto-merge,
release or deployment has occurred during YT-097.

## Post-review repository ignore-policy narrowing

The advisory workspace hygiene check after YT-097 acceptance identified the
pre-existing Git-visible `.env.local.example` as a high-severity secret-like
**untracked filename**, not an observed credential disclosure. This local file
was never read or included in source verification. To finish the owner's
safe-example filename correction without bypassing file-policy protections,
the repository `.gitignore` was narrowed by removing the broad
`!.env.*.example` unignore exception. The sole allowed exception remains
`.env.example`, so the old untracked sensitive-named file stays physically
untouched while becoming ignored by ordinary Git operations. This changes no
Rust runtime code, toolchain, actual secrets, or Commit Guard rule. A separate
focused independent read-only role-directed Reviewer
`wc_job_koIrHlo9ykP5sKJj` completed exit 0 with
`overall_status=pass`, `issues=[]` and `required_followups=[]`. Both reviewed
file hashes matched before and after the review. The Reviewer also reconfirmed
that the prior-reviewed CLI source/test/template/README hashes were unchanged
and that Git excludes `.env.local.example` but not `.env.example`. The only
later change was this documentary reviewer-status closeout; no code, sample,
Git index or sensitive-file modification followed.
