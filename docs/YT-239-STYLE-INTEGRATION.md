# YT-239 — Optional application-owned CSS integration

**Date:** 2026-11-08. **Status:** **Done at source and bounded Linux integration
scope after independent Reviewer PASS.** Job `wc_job_3s_tnZYc7hUssjK5`,
exit 0, zero issues/followups. Public release and YT-240 qualification remain gated.

The owner authorized promotion of the YT-238 SIMPLIFY recommendation: keep the
builder private to generated applications. This follows the accepted
[registration contract](YT-237-STYLE-REGISTRATION-CONTRACT.md),
[CLI contract](CLI-CONTRACT.md), [asset boundary](ASSETS.md), and ADRs
[0003](adr/0003-renderers-and-loading.md) and
[0005](adr/0005-opt-in-capability-modules.md). Core, View, renderers, HTTP APIs
and `DEFAULT_CSP` are unchanged. Ordinary CSS remains the default.

## Selection, schema and layout

Use the existing selection mechanism:

```sh
yetlet new demo --template native-style
yetlet new tasks --template browser-style --tailwind
yetlet new web --template native-style --vite --tailwind
```

Ordinary `native` and `browser` choices keep their prior behavior. Style variants
compose with applicable `--settings`, `--service`, `--tailwind` and `--vite`
choices; browser still refuses native service scaffolding. No styles flag exists.
Creation uses the existing protected Linux/Windows creator and a finite allowlist
of flat paths; it executes no generator, app dependency resolution or install.

```text
app/
  src/style_names.rs
  stylegen/Cargo.toml
  stylegen/Cargo.lock
  stylegen/main.rs
  stylegen/builder.rs
  stylegen/styles.rs
  stylegen/layout.rs
  stylegen/dialog.rs
```

The complete `styles::stylesheet()` composes useful layout and dialog modules,
including the conditional-only dialog class. Views and the generator share class
constants; neither discovers styles by executing Views. Add a module by creating
`stylegen/panel.rs`, declaring `#[path = "panel.rs"] mod panel;` in `styles.rs`,
and adding `panel::styles()` to the existing composition. A new file alone does
not register styles. Builder sources are package-local template copies, with no
public style crate or package include outside the CLI.

```toml
[cli.styles]
enabled = true
manifest = "stylegen/Cargo.toml"
bin = "app-stylegen"
output = "assets/generated/app.generated.css"
```

The existing base/profile/APP_CLI Loader owns precedence and type checking. The
optional section has exactly these four required fields and rejects unknown
fields. `enabled=false` still validates shape and performs no style work; absence
means no style discovery, probes, directories or subprocesses. Inspect reports
selected/skipped stages without reading generator/frontend files or running tools.
Generated strict native runtime settings accept and validate the optional section
without relaxing unknown-field rejection.

This delivery deliberately supports the private `stylegen/Cargo.toml` selection,
one explicit `main.rs` binary and a std-only standalone package with its own lock,
`publish=false`, no build script and no target/dependency graph. Bin names are
bounded ASCII Cargo target names. The finite output is
`assets/generated/app.generated.css` without Vite, or
`<selected-frontend-root>/style-output/app.generated.css` with Vite. Other output
or manifest layouts are refused. Paths remain anchored to the selected app root,
even when `YETLET_CONFIG_ROOT` chooses another approved settings directory.
Frontend roots overlapping source, generator or private state are refused.

The official generated new/build/run/watch path uses Rust, plus the existing Node
coordinator for selected Vite/browser behavior. It requires no Python. The
historical `experiments/style-authoring/` Python publisher and its reviewed proof
remain preserved; they are not the production integration path. Existing Python
product fixtures and the artifact helper remain auxiliary repository tools.

## Host execution, ownership and publication

From the protected generator directory, the CLI captures selected `rustc -vV`
and `rustc --print sysroot`, then pins the same compiler/toolchain and actual host:

```text
cargo run --frozen --quiet --manifest-path Cargo.toml --bin app-stylegen \
  --no-default-features --target x86_64-unknown-linux-gnu \
  --target-dir <selected-cargo-cache>/yetlet-styles/<app-root-sha256>
```

`RUSTC` points to that sysroot's compiler; `RUSTUP_TOOLCHAIN` pins its sysroot.
Offline Cargo and disabled rustup installation/update checks prevent automatic
resolution, installs or lock rewriting. App lock/cache setup remains explicit.
The generator's stdout is CSS only; diagnostics stay on stderr. The CLI captures
complete successful nonempty UTF-8 output bounded to 2 MiB, with owned process
cleanup and a 120-second producer deadline, before publication. Rust code and CSS
are trusted developer inputs, not a sandbox or general CSS sanitizer.

Host compiler output uses a separate subtree of the selected Cargo cache, not
private metadata or served assets. Existing cache roots must satisfy protected
ownership/write policy; a group/world-writable selected cache is refused rather
than chmodded. Native default `target/` can be created beneath the protected app.
For this isolated run a newly created private `cargo/` inside the allocated cache
was selected; no existing directory permissions were changed. One early run's
exact task-created compiler subtree was moved intact into the allocated cache,
retained as `retained-initial-native-stylegen-target`, without deletion.

Private `.yetlet/styles/coordinator.lock` serializes coordinators.
`owner.json` records logical `app-css`, output path and full lowercase SHA-256.
First claim requires absent output; later publication verifies prior ownership
and exact old content. Manual changes conflict. Protected ancestors and native
regular-file/no-link checks reject source/config/cache and redirected destinations.
A sibling temporary and platform-specific rename replace complete bytes;
byte-identical CSS and records are not rewritten. Static export stays create-only,
and the settings/key adapter's separate 64 KiB contract is unchanged.

Before replacing accepted CSS or a production record, the coordinator durably
writes `owner.pending.json` or `production.pending.json` beside the private
records. Each intent retains the exact previous accepted record (or null for a
first claim) and the intended next record. Both producers and the generated
native loader refuse either pending marker on ordinary retry/startup, even when
the current record and CSS hashes match. Same-byte CSS remains a no-op.

Prepublication write failures preserve prior bytes/record and clear only an intent
created by that attempt when no accepted destination was replaced. Linux temporary
creation is exclusive; write/sync/check failures clean only the temporary whose
identity matches the attempt's open handle. Initially preexisting unknown
temporaries remain untouched. A later failure, including immediately after a
record rename, leaves the generation **UNACCEPTED** with prior metadata retained
inside the pending evidence. It cannot claim prior CSS bytes survived or that the
new source is fresh.

Acceptance commits at successful native removal of the pending marker. All
fallible writes, directory syncs and boundary checks precede that removal; no
post-removal sync/check can return an unaccepted error after consumers see
acceptance. A crash before removal leaves pending state. Power loss may resurrect
an unsynced removal and conservatively require reconciliation; this is not a
power-loss transaction guarantee. Reconciliation is an explicit operator action:
preserve the pending evidence and current records/assets, then restore both CSS
and record to the prior acceptance, or independently verify the intended record
and exact CSS bytes before approving rollforward. Only after that decision remove
the corresponding pending marker. Ordinary build/run/watch never clears retained
pending state. No automatic reset, two-file transaction or hostile concurrent
filesystem isolation is claimed.

## Lifecycle and consumers

| Selection | Behavior |
|---|---|
| Native build | Generate/publish source CSS, selected frontend build, app build, then production acceptance |
| Native run | Generate before selected startup; no unrequested production frontend build |
| Native watch or configured Rust watch | Stop/reap old backend; serialize CSS preparation/publication and app restart; source failure waits for correction |
| Browser build | Preserve auto_build skip; selected CSS runs before existing WASM/Vite stages |
| Browser run | Preview existing dist, without generator or new style state |
| Browser watch | Require auto_start and auto_build; existing Node loop owns serialized CSS-before-WASM preparation and publication |
| Inspect | Settings-only stages and skip reasons, without tool/file freshness claims |

Watch includes app Rust/shared names and generator Rust/manifests/locks, excluding
outputs, private state and owned targets. Input snapshots and pending-event checks
discard obsolete candidates. Failures wait for edits; they do not automatically
retry. Settings/policy changes require restart. External mutable inputs and escaped
daemons are unsupported. No second style watcher is added.

The browser coordinator receives a bounded finite private selection containing
only app root, public style choices and selected frontend root. It does not regain
APP_* or backend secret selectors or load another default profile. Its private
prepare/publish protocol invokes the same Rust boundary; Node retains abortable
owned children. A failed CSS stage suppresses new WASM; a queued edit suppresses
obsolete publication. Accepted CSS and WASM are retained together in the dev
loop, with Vite's existing HMR/nonce/CSP behavior and source confinement.

Standalone native development links the stable CSS URL. After all selected
production stages succeed, the CLI publishes
`assets/generated/app.<full-lowercase-sha256>.css` and accepts the private
`production.json` logical record. Mismatched existing digest assets are refused.
A later app/frontend failure preserves the previous accepted record/assets.
The app-owned adapter verifies the digest against the exact startup bytes before
linking and exposes only its selected in-memory same-origin CSS route. CSS uses
`text/css; charset=utf-8`, nosniff, no-store and exact `DEFAULT_CSP`; requests never
read private records/source or compile code. Production packaging must carry the
accepted record and referenced assets. Older graph retention, deployment, HA and
automatic cleanup are not supplied.

Standalone Tailwind owns document page/install and composes Tailwind then generated
CSS links. Vite variants import generated CSS after app.css in the normal graph;
browser-style moves app.css from its former HTML link into that entry graph.
Vite owns hashing/manifest/link order; no standalone second generated link or
second hash is added. Later source order only breaks ties after importance,
layers and specificity. In particular normal unlayered CSS can outrank layered
utilities; use deliberate names and authored override ordering.

## Commands and observed results

Evidence directory: `artifacts/verification/yt239-style-integration-20261108/`.
The run preserves initial failures, command/output logs and JSON results.
Allocated cache: `.cache/yetlet/builds/yt239-style-integration-20261108/`;
CLI builds/tests use that root and real app builds later select its protected
`cargo/` child. Exact app argv and cache selections are recorded in JSON/logs.

Toolchain: Rust/Cargo 1.99.0, rustc `b940084d7`, host
`x86_64-unknown-linux-gnu`. The selected toolchain's sysroot was resolved with
`rustc --print sysroot`; exact host-local paths remain in ignored run evidence.
Node v24.22.1, Vite 8.3.4, Tailwind Vite 4.3.3, wasm-bindgen 0.2.129,
Watchexec 2.8.0. The parent prepared the official SHA-256-verified Watchexec
executable in this run's isolated cache (`parent-test-tools.json`); the worker
reused it. Existing Rust/Node/WASM installations and dependency caches were reused.
No global installation, global PATH change or permission-policy change occurred.

| Command/evidence | Observed result |
|---|---|
| `cargo check --locked --offline --manifest-path tools/yetlet-cli/Cargo.toml` | PASS |
| Focused style settings/publisher unit cases | 5 PASS: strict shape, no-op/conflict/stale handling, digest verification and pre/post-replacement failures |
| Bounded owned stdout capture case | PASS: successful bytes, producer failure, invalid UTF-8 and output overflow |
| Final CLI library test run, excluding the two recorded cloud-lock failures | 100 PASS, 1 existing ignored, 2 explicitly filtered |
| Commands/browser/frontend/scaffold integration suites | 28 PASS, 2 existing ignored; frontend includes 2 new style sequence/gate cases |
| Actual protected creation | 13 style/option combinations, all emitted Rust parses; ordinary creation regression retains 16 option combinations |
| Real native build/watch/recovery and HTTP exercise | PASS: 10 recorded assertions including edit, failure/correction, queued obsolete candidate, digest/CSP/MIME/no-store and five private-path refusals |
| Real native-style Vite/Tailwind production build | PASS: generated CSS present in Vite output/manifest, no standalone production record |
| Real browser-style Tailwind WASM/Vite production build | PASS after the recorded inner-doc placement correction; generated CSS in the normal Vite bundle |
| Later dependent native Cargo build failure | Expected exit 101; prior production record and digest asset hashes preserved while new stable CSS was prepared |
| `cargo fmt --manifest-path tools/yetlet-cli/Cargo.toml -- --check` | PASS |
| `cargo xtask check docs` | PASS: docs/language/links/structure/whitespace/DAG |

The initial broad library run had 94 PASS, 2 cloud-lock fixture failures and 1
ignored before the six new unit cases were added. The two failures are
`current_selected_profiles_keep_pinned_overrides_public_settings_and_asset_mapping`
and `delivery_profile_matrix_exact_inventory_env_and_no_secrets_or_developer_paths`.
The legacy delivery fixture calls unchanged legacy scaffold/lock/source paths;
these unresolved failures are retained, not relabeled PASS or repaired in this
slice. The final library command filters those exact two cases explicitly.

Three bounded product repairs were used: duplicate dependency insertion for
native settings, the browser's original HTML CSS connection, and an inner-doc
placement error in emitted browser Rust. Canonical fixture corrections separately
fixed private TempDir setup and an assertion expecting native config in a browser
starter. Original failures and classifications remain recorded.

The real HTTP/watch exercise stopped both owned leaders and observed all 23
tracked owned PIDs absent. Browser sequence tests cancel an active actual Rust
CSS generator and verify its PID is gone; existing process regression cases retain
exact group cleanup. Build commands are foreground and one-shot; no GUI/browser,
watcher, Node service or backend is intentionally left running. A final resource
check is recorded separately.

## Narrow publication/Tailwind continuation

The prior execution remains terminal partial with three product repairs and two
operational retries recorded. This separately admitted continuation changes only
the pending acceptance boundary, owned temporary recovery and the native Tailwind
class composition defect. It adds no validation framework, dependency or global
tool setup. Its initial implementation and focused verification passed without
additional product repairs or operational retries (cumulative counters: 3 and 2).
Independent review and backlog status remain parent-owned.

Continuation evidence uses `continuation-*` names in the same evidence directory:

| Evidence | Result |
|---|---|
| `continuation-styles-tests.log` | 10 PASS: prior style/settings cases plus post-owner/production-record-rename faults, preserved previous metadata, pending retry refusal, explicit rollback/rollforward, prepublication recovery and owned write/sync temporary cleanup/unknown preservation |
| `continuation-scaffold-tests.log` | Style creation combinations PASS; generated main elements preserve Tailwind utilities and SHELL in one class attribute |
| `continuation-real-new.log`, `continuation-real-lock.log`, `continuation-real-build.log` | Fresh native-style Tailwind app created from changed templates, explicitly locked offline and built successfully against the registry SDK using the existing protected Cargo cache |
| `continuation-real-fixture.json` | Existing successful Vite/Tailwind compiled CSS reused as trusted standalone adapter input; no Tailwind compiler or Vite qualification repeated |
| `continuation-real-exercise.json`, `continuation-response-*.html` | Actual localhost HTTP: home and escaped hello routes return 200 in development/production; one combined main class, Tailwind then generated links, both CSS routes with MIME/nosniff/exact CSP |
| `continuation-pending-*.log`, `continuation-retained-*.pending.json` | Both pending markers independently refuse development and production startup and ordinary CLI build retry despite matching records/bytes; evidence remains unchanged until explicit verified rollforward, then both modes recover |

Fault injection runs immediately after the native record rename and before its
sync/verification. The old record may already be replaced; its exact prior bytes
remain in pending evidence. The generated-consumer exercise separately injects
the same pending shape into the fresh application with matching accepted hashes.
It does not claim a process-kill or power-loss fault test. All 10 subprocesses from
the HTTP/refusal exercise were reaped and their exact PIDs verified absent; no
watcher, GUI/browser or Node service was started. The build commands were bounded
foreground commands. Visual layout remains unverified, as requested.

## Limits and subsequent qualification

Windows native adapters are supplied with the existing API shape. The attempted
`cargo check --target x86_64-pc-windows-msvc` stopped in openssl-sys because the
cross OpenSSL/MSVC environment is unavailable. It did not qualify compilation of
the changed CLI or any Windows runtime. No external Windows task was dispatched.
A selected native Windows check remains required before claiming that support.
The styles capture path currently requires an attached Windows console with
processed Ctrl-C input; headless/raw-console execution is refused before a child
starts. The existing Windows executable allowlist also remains in force.

No real-browser computed layout, viewport/CSP interaction matrix, Windows/browser
rendering qualification, production deployment or release is claimed. Those
selected qualification decisions remain YT-240. Independent YT-239 review has
passed; YT-240 still requires owner selection. No commit, push, PR, merge,
worktree, release, permission change to existing files or cache deletion was
performed by this worker.

## Independent review and completion

YT-239 is **Done at source and bounded Linux integration scope** after independent
Reviewer PASS on 2026-11-08. The reviewed work includes the main implementation
and the narrow publication/Tailwind continuation.

| Review evidence | Observed result |
|---|---|
| Independent Reviewer Job | `wc_job_3s_tnZYc7hUssjK5`, terminal completed, exit 0 |
| Decision | `overall_status=pass`; zero issues, required followups and optional notes |
| Role configuration | Saved `gpt-6-astra`, resolver-selected `high`; both match process telemetry |
| Source identity | All 61 worktree source hashes unchanged during review: 43 YT-239 targets and 18 preserved historical files |
| Review form | Separate role-directed ad hoc Codex Job; independent implementation review, without a native managed-role or formal-assurance claim |
| Prior work preservation | All 17 prototype files and the YT-238 proof match the pre-YT-239 baseline |

Evidence is retained under
`artifacts/verification/yt239-style-integration-20261108/`, including
`review-input.json`, `reviewed-source/`, `review-diff.patch`,
`reviewer-result.json` and `review-acceptance.json`.
Post-review edits record completion/status only; the final handoff records their
identity and final document/whitespace checks.

The two recorded cloud-lock failures remain unresolved, and Windows compilation
and runtime remain unverified. This PASS accepts the scoped YT-239 integration,
not those unexecuted or failing qualification paths. YT-240 remains Todo and
requires owner selection before its real-browser/platform work. Public release
and Git delivery remain separately gated.
