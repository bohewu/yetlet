# YT-089b — Configuration-only Docker Generator Verification

**Date:** 2026-10-16. **Final state:** Independently accepted Docker configuration-only generator at bounded Linux/amd64 BuildKit and Linux/Windows GNU/MSVC byte-identical output scope; Reviewer `wc_job_IC92mEXoQMsG4nTg` PASS, zero issues/followups. Earlier Generalist pending and failed receipts remain historical below.
YT-089 remains In progress; YT-020 remains Blocked. Compose and Kubernetes
implementation belong to later slices.

## Checkout, authority and preserved evidence

Work stayed in `$HOME/repos/yetlet`, branch `master`, HEAD
`d24a05033c57e5b98dd57bf83189bc1297f6f28c`, with the existing uncommitted M15 and
YT-089a changes. This is the requested role-directed Codex work in the WebCodex
session using the canonical Generalist contract; no native managed-agent dispatch,
verified role model/effort claim or child-agent delegation is made. The canonical
Generalist definition, REASONING_POLICY, MATERIALITY_GATE, repository AGENTS.md,
ADR 0006, [accepted design](YT-089-CLOUD-NATIVE-GENERATORS-PLAN.md) and
[YT-089a verification](YT-089A-VERIFICATION.md) were read before implementation.
YT-089a independent PASS `wc_job_0dvTDNAV22Vbpc5j` and the earlier closed SIGINT P2
remain unchanged, as does design Reviewer `wc_job_9IplTI1HHkyZK9hv` PASS.

A 665-file SHA-256 baseline was captured before editing. Existing cloud/default
scaffold source, templates, runtime fixtures, YT-089a evidence, design document,
artifacts, framework source and root/xtask Cargo manifests/locks were preserved.
`scaffold::create(name, framework, directory)` retains its three-argument ABI;
`create_cloud` and old new/default/cloud flags retain their behavior. No clone,
worktree, stash, reset, clean, stage, commit, push, global install, publication,
hosted deployment, K8s operation or image push occurred.

## Delivered behavior

```text
yetlet generate docker --manifest-path APP/Cargo.toml --output NEW_DIRECTORY
```

Arguments remain literal, strict and command-specific. Missing, unknown,
duplicate or misplaced options fail with usage exit 2. Generation preflight
failures use exit 1. Only Docker is supported under `generate` in this slice.
The operation invokes no Cargo, Docker/buildx, kubectl, shell, network client or
source-export process and modifies no selected app files.

`platform/fs/open_app` validates and pins the selected standalone application.
Only the generated cloud-profile manifest, existing explicit Cargo.lock and
known `src/main.rs` are read as app inputs. Both the parsed Rust main AST and its
normalized text must match the accepted cloud template; a metadata marker alone,
manual/edited source, extra source targets, build script, app Cargo configuration,
features or dependencies is rejected. CRLF/LF comparison is normalized for the
source contract; hashes retain actual input bytes. A second inspection before
publication detects changed snapshots. Linux retains the app's directory-handle
ancestry; Windows uses the existing no-delete-share/ACL/reparse-safe handles.

Output uses the existing native `create_project` no-replace adapter, with one
additional fixed child layout `cargo-overlay/Cargo.toml`. Arbitrary nested paths
are rejected before writing. A fresh private sibling is populated and published
without replacing the destination; cleanup checks held identities and never
recursively deletes unknown entries. The output parent must already exist and
be protected; the Windows Docker path additionally verifies its owner's ACL.
Unsafe parents, traversal, symlink/junction paths, existing destinations and
source changes fail closed. Spaces and Unicode are covered within OS limits.

Exactly eight files are generated:

- `Dockerfile`
- `.dockerignore`
- `cargo-overlay/Cargo.toml`
- `framework-workspace.toml`
- `app-src.dockerignore`
- `yetlet-src.dockerignore`
- `build-context-inventory.json`
- `README.md`

The mandatory three-context design is implemented. DIR is the only default
context, containing generated configuration, with deny-by-default root ignore
rules allowing only Dockerfile and the overlay. `app-src` is a **separate manually
prepared** export containing only byte-exact app Cargo.lock, main source and its
generated ignore file. The raw app Cargo.toml and its absolute framework paths
are excluded. `yetlet-src` is another manually prepared export: the generated
minimal workspace root, three package manifests, nine known source files and its
generated ignore file. No app/framework files are automatically copied.

The supported framework dependency closure is explicitly core/HTML/server-axum.
Package manifests are parsed and compared against the known inheritance and
path/registry dependency contract; unknown packages, path dependencies, inherited
keys, features, build scripts and source layouts are refused. The framework
baseline is deliberately narrow: exact Rust 1.99 / edition 2024 / version 0.1.0
workspace.package values and current axum/tokio workspace.dependencies. A different
baseline requires a later reviewed profile rather than an invented build graph.
The minimal root contains exactly three members, resolver 3, and the exact
inspected inherited package/dependency values. Framework root Cargo configuration,
workspace lock, unrelated crates, tests and examples are not exported.

The app overlay is reconstructed from parsed TOML. A semantic roundtrip restores
exactly the two original dependency paths and compares equality with the original
manifest; only core/server paths change to `/work/yetlet/crates/...`. No raw source
manifest, personal framework path, environment value or unknown source file is
serialized into generated output. Inventory contains only fixed relative paths,
byte lengths and nonsecret SHA-256 checksums, without timestamps.

The generated README requires physical separation of all contexts, explicit
manual file selection, public-input audit, no links/reparse/special files, safe
ownership, exact file/directory inventories and all hash/length comparisons before
any upload. `.env`, SSH keys, `.git`, target and other unknown inputs must be
absent. It explicitly says DIR/.dockerignore does not sanitize named contexts
and named-context ignore behavior cannot substitute for physical allowlists.
No exporter or new validation framework is introduced.

Dockerfile uses named `COPY --from=app-src` and `COPY --from=yetlet-src`, followed
by the app overlay, `cargo metadata --locked` and
`cargo build --locked --release`. Registry downloads can require network; this
is not an offline container build. Versioned mutable bases are
`rust:1.99.0-bookworm` and `debian:bookworm-slim`, Linux/amd64 GNU/glibc. Their
availability, effective digests and runtime compatibility remain Docker-gate
checks. Operator digest pinning is documented without fabricating registry
results. Runtime uses UID/GID 65532, exec entrypoint, `YETLET_HOST=0.0.0.0`,
`YETLET_PORT=8080` and SIGTERM. No HEALTHCHECK assumes curl/wget or another
unprovided executable. No credentials, cluster commands or image publication
are generated.

## Actual changed files for this task

These are incremental YT-089b edits, not all differences against HEAD:

- `tools/yetlet-cli/Cargo.toml`, `tools/yetlet-cli/Cargo.lock`: pinned SHA-256 and
  Rust source-parser dependencies in the isolated CLI workspace only.
- `tools/yetlet-cli/src/lib.rs`, `tools/yetlet-cli/src/main.rs`: strict command,
  help and dispatch.
- `tools/yetlet-cli/src/docker.rs`, `tools/yetlet-cli/src/docker/tests.rs`:
  inspection, deterministic rendering, inventory, gating and focused tests.
- `tools/yetlet-cli/src/platform/fs/mod.rs`,
  `tools/yetlet-cli/src/platform/fs/linux.rs`,
  `tools/yetlet-cli/src/platform/fs/safe_dir.rs`,
  `tools/yetlet-cli/src/platform/fs/windows.rs`,
  `tools/yetlet-cli/src/platform/fs/windows_tests.rs`,
  `tools/yetlet-cli/src/platform/fs/unsupported.rs`: pinned input inspection,
  fixed overlay child layout and native safety tests/refusal paths.
- `tools/yetlet-cli/templates/docker/Dockerfile`,
  `tools/yetlet-cli/templates/docker/README.md`: build and manual export contract.
- `tools/yetlet-cli/tests/docker.rs`,
  `tools/yetlet-cli/tests/fixtures/docker_framework.rs`: executable-level
  no-tools/secret/no-overwrite checks and a self-contained source-layout fixture.
- `tools/yetlet-cli/README.md`, `docs/BACKLOG.md`, `docs/README.md`,
  `docs/ROADMAP.md`, `docs/YT-089B-VERIFICATION.md`: command documentation and
  explicit pending acceptance gates. Existing YT-089a text/evidence is preserved.

## Native Linux evidence

Rust `1.99.0 (b940084d7 2026-09-28)` and Cargo
`1.99.0 (5f94df478 2026-08-27)` on Linux. Logs and native exit receipts are under
`target/m15/yt089b/`; `linux-cli-results.json` lists exact argument arrays.
The following commands each returned native exit **0**:

```sh
cargo fmt --manifest-path tools/yetlet-cli/Cargo.toml --all -- --check
cargo check --manifest-path tools/yetlet-cli/Cargo.toml --offline --locked --all-targets
cargo test --manifest-path tools/yetlet-cli/Cargo.toml --offline --locked --all-targets
cargo clippy --manifest-path tools/yetlet-cli/Cargo.toml --offline --locked --all-targets -- -D warnings
```

Results: **35 unit tests passed**, plus one cloud/default scaffold integration,
one existing frozen build/dev integration and one new Docker CLI integration
passed. The pre-existing explicitly opt-in generated-cloud-app runtime test
remained ignored; its accepted YT-089a runtime gate was not reopened. Native
Windows-specific binaries have zero Linux tests, not Windows PASS evidence.

Focused cases cover literal parsing, duplicate/invalid flags, output traversal,
missing parents/locks, unsupported source/manifest/dependency layouts,
metadata-only rejection, parsed cloud main verification, deterministic files,
exact two-path overlay rewrite and minimal workspace values, selected output
files, existing destination preservation, unsafe parents, source changes, app
rename races, symlink refusal, late output creation and fixed-child partial
cleanup. The secret fixture includes app `.env`, `.env.production`, SSH/private
inputs, `.git` and target. Neither their contents nor original app/framework
absolute paths appear in generated files. Executable-level generation succeeds
with a nonexistent tools PATH and Cargo home and an invalid Docker endpoint;
no Cargo home, staging residue or extra output file appears. The implementation
contains no subprocess/network dispatch. Ordinary CLI regression tests themselves
explicitly invoke Cargo as their established test workload.

A separate explicit fixture preparation created `target/m15/yt089b/cloudproof`
using the real checkout and the unchanged cloud scaffolder, then explicitly ran
`cargo generate-lockfile --offline` for that new app. The actual generator, with
PATH/Cargo-home set to nonexistent task-owned paths, successfully wrote
`target/m15/yt089b/real-source container` against the real core/HTML/server source.
All three commands returned native exit 0; exact arguments and logs are in
`real-source-generation.json` and `real-source-{0,1,2}.log`. This proves real-source
configuration generation, not container compilation or HTTP. No sanitized source
export was created or sent to Docker. The CLI dependency lock was explicitly
updated once with `cargo generate-lockfile --offline`; existing locked package
versions were preserved and only the new SHA-256 closure/direct dependencies
were added. No network preparation or installation was performed.

Repository docs/sample/diff command receipts are in `repository-checks.json`
and corresponding logs under the same evidence directory. With
`CARGO_NET_OFFLINE=true RUSTUP_AUTO_INSTALL=0`, `cargo xtask check docs` returned
0 (75 documents / 89 backlog items), and `cargo xtask check samples` returned 0
(12 samples). `git diff --check` returned 0. All seven new files separately
passed `git diff --no-index --check -- /dev/null FILE`, with empty whitespace
diagnostics and verified LF/final-newline bytes. For no-index checks, native
0 or 1 is acceptable only with empty whitespace diagnostics; 1 may simply
represent the expected file difference. `new-file-checks.json` records each
actual code and file hash.

`real-source-inventory.json` verifies all 27 inventory entries against the
actual app/framework inputs and generated templates, without creating exports
or running Docker. All hashes and byte lengths match. Preservation receipt
`preservation.json` confirms exactly 14 intended baseline-file changes and
seven new files, 651 other baseline files byte-identical, no missing baseline
files, unchanged master HEAD and an empty staged diff. `baseline-sha256.json`
retains the original 665-file baseline. `cleanup.json` records no remaining
Cargo/test processes or staging directories and no started server/container.

## Historical failures, repair bounds and cleanup

Initial all-targets tests returned native exit **101**: 30 unit tests passed and
five failed with `src/docker.rs:174:25: index not found`. The initial minimal
workspace renderer indexed a missing TOML key instead of inserting it into a
table. This was a **product_failure**, not an environment failure. Its original
log/receipt remain `initial-cli-test.log` / `initial-cli-test.json`. Initial strict
Clippy already returned 0. Repair 1 changed only the minimal table insertion;
`repair1-cli-test.log` records all 35 unit tests and the three integrations passing.

A bounded code inspection then identified the missing Docker-specific Windows
output-parent ACL check required by the original request. Repair 2 added that
check before native staging and focused native Windows overlay/Unicode,
late-destination, junction and foreign-writable-parent cases. The existing
scaffold policy was retained. Linux required verification was rerun successfully;
Windows execution remains unverified. **repair_attempts_used = 2/2**;
**operational_retries_used = 0**. No further repair is authorized within this job.
Any new material failure must be returned to the parent without weakening safety.

All foreground Cargo/test processes completed. No task server, browser, watcher,
Docker daemon/container or cluster resource was started. Disposable test fixtures
use automatic cleanup, and the output tests explicitly check staging absence.
The real-source app/configuration fixture and logs are intentionally retained
under the task-owned ignored evidence directory for the parent's next verification;
they are not background resources or a source export.

## Remaining gates and next action

1. Parent: read-only verify the same checkout/HEAD, this report, preserved source
   and current generator files. Native Windows GNU/MSVC must use identical
   checksummed CLI/template/test inputs and separately prepared offline caches
   for the new SHA-256/parser closure; no auto-download fallback is permitted.
2. Parent: manually prepare protected, physically separate app-src/yetlet-src
   exports using the exact generated inventory, including the minimal workspace
   root, and verify every path/hash/length plus planted-secret absence. Inspect
   actual effective context transfer. Then perform the separately authorized
   disposable Linux/amd64 BuildKit build, locked metadata/release compilation,
   source-lock preservation, non-root HTTP health/SSR/CSP/escaping, real SIGTERM
   drain and complete listener/container cleanup. Resolve base/frontend image
   availability/digests from actual evidence. No push or cluster operation.
3. Parent: native GNU/MSVC fmt/check/tests/strict Clippy and source-bound generation
   comparison of all eight output files for identical fixture bytes, including
   no-overwrite, ACL/junction and spaces/Unicode cases. Compilation alone is not
   output parity.
4. Separate independent Reviewer: inspect final source and all actual Linux,
   Docker and Windows receipts. This implementation session is not that review.

The next unblockable work is these parent verification gates for **YT-089b**.
Only after separate acceptance should YT-089c Compose become the next generator
implementation. No broader release, production readiness, Compose/K8s support,
Windows containers, multiarchitecture build or image-publication claim is made.

## Parent-run Linux Docker and Windows GNU/MSVC follow-up (2026-10-16)

The original Generalist's 2/2 exhausted repair budget and pending acceptance statements above remain historical. The parent completed separately authorized follow-up checks in the same checkout; there was no production security-gate relaxation.

**Real Linux Docker smoke:** The operator explicitly prepared two physically sanitized named BuildKit source contexts and used the configuration-only generated default context. The actual Linux/amd64 Docker build compiled the minimal three-crate Cargo workspace with locked metadata and release compilation and produced a disposable image using the real generated cloudproof SSR application. Real container HTTP calls returned 200, CSP and escaped HTML, both health endpoints responded and Docker stop exercised SIGTERM cleanup. The task-owned container and image were removed, and subsequent Docker listings found neither. Retained original records: target/m15/yt089b/docker-buildx-actual.log, docker-runtime-actual.log and docker-smoke-summary.json, together with checked source-context manifests and HTTP headers/bodies. None of these Docker commands was invoked by Yetlet generation.

**Windows native fixture failure and correction:** The original 38-file Windows source candidate initially lacked offline cached sha2 dependencies; operator-side cache preparation was performed explicitly. Both GNU and MSVC then reached 34 passed/3 failed because the new Docker tests selected an inherited group-writable Windows TempDir for output and correctly received an untrusted-ACL refusal. A test-only fixture correction selected a separate CLI-created owner-protected output parent. The Win32 race/rename fixture also created its protected directory atomically with the native security descriptor and checked it via a READ_CONTROL-enabled handle. Intermediate fixture errors (a stale output marker and a test-only unused field) were repaired without modifying the production Docker, filesystem or process security policy. Final current-source ZIP SHA-256 is c6488eea209334ed097a3320d3c152da60b73b408fcc508140332be26f04feb4, with all 38 embedded source digests independently checked by both Windows Runners.

| Source-bound result | Ubuntu Linux | Windows GNU | Windows MSVC |
|---|---|---|---|
| Cargo fmt/check/strict Clippy (offline, locked) | PASS | PASS | PASS |
| Unit tests | 35/35 PASS | 37/37 PASS | 37/37 PASS |
| Existing and new integrations | PASS | Cloud, Docker, 5 build, 3 Windows new PASS | Same, PASS |
| NTFS owner-only output/junction/no-replace negative test | Not applicable | Executed, PASS | Executed, PASS |
| Disposable BuildKit image + HTTP/CSP/SIGTERM | PASS on Linux | Linux OCI target only | Linux OCI target only |

**Eight-file byte parity:** GNU and MSVC each constructed the same deterministic framework fixture, used native CLI cloud scaffolding and an explicit synthetic Cargo.lock, then invoked the real generate docker command into a separate protected output parent. Independently created Linux output is the reference. Each of the eight emitted files has exactly matching SHA-256 across all three environments: Dockerfile, .dockerignore, README.md, app-src.dockerignore, yetlet-src.dockerignore, cargo-overlay/Cargo.toml, framework-workspace.toml and build-context-inventory.json. No compiler, Docker daemon, registry, cluster or environment credential access was triggered by CLI generation. Windows raw test and parity receipts are byte-verified and retained at target/m15/yt089b/windows-final-20261016/{gnu,msvc}/ with summary.json; the Linux reference inventory is target/m15/yt089b/parity/linux/parity.json.

**Scope:** This proves a bounded Linux/amd64 image can be built from the manually sanitized three-context source contract and verifies identical configuration output on GNU/MSVC. It does not claim Windows container support, automatic source exports, production registry/publishing or Kubernetes deployment. First-failure receipts remain preserved. Independent read-only Reviewer `wc_job_IC92mEXoQMsG4nTg` subsequently returned `overall_status=pass`, `issues=[]` and `required_followups=[]` after inspecting source/ACL safety, exactly two Cargo path relocations and minimal workspace inheritance, three physically sanitized BuildKit contexts, disposable real Docker build/HTTP/SIGTERM and cleanup, and all eight native Windows GNU/MSVC byte-identical output receipts. **YT-089b is Done at this bounded configuration-only scope.** Compose, Kubernetes and final cloud-generator parity remain separate pending slices; neither a cluster deployment nor public release was accepted.
