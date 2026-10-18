# YT-101 — Optional cloud asset and configuration delivery

**State: Done at bounded YT-101 opt-in generator scope.** The original
implementation Job completed Linux local/native checks; the parent subsequently
completed real Linux Docker, Compose and three-node K3s runtime acceptance and
source-bound Windows MSVC native validation. Independent Reviewer
`wc_job_fYtD9F8KXkXyzvYQ` returned `overall_status=pass` with zero
issues/required followups. A subsequent source-bound Windows GNU 1.99
native requalification also passed after installing official MSYS2 MinGW64
GCC/Binutils; see supplemental evidence below. The implementation Job itself
did not launch Docker or K3s. No Commit/Push/release/production deployment occurred.

## Source and bounded contract

Work used the existing `$REPO` checkout, branch
`feat/yt090-configuration`, starting HEAD
`2987e072c5ac035c3c7e3b55cb76c9ae17a92a9b`. Pre-existing M19/M20 owner planning
in BACKLOG/ROADMAP and untracked artifacts were preserved. No new checkout,
branch, staging, commit or push was performed. Repository documentation is
English. YT-102 and all unrelated implementation remain outside this task.

The existing native no-overwrite writer and exact-source inspection boundary
now accept the following **generated, unedited** cloud profiles:

| Scaffold selection | Docker | Compose | Kubernetes |
|---|---|---|---|
| `--cloud` (legacy) | Existing bytes | Existing bytes | Existing bytes |
| `--cloud --assets vite` | Built dist | Same image | Same image |
| `--cloud --assets vite --with-tailwind` | Official plugin dist | Same image | Same image |
| Either Vite row plus `--with-config` | Dist and public config | Required runtime token | Existing Secret reference |
| `--cloud --with-config` | Public config only | Required runtime token | Existing Secret reference |

Standalone Tailwind, edited/general Cargo projects and the Vite development
bridge are outside this delivery profile. The CLI never executes npm, Cargo,
Docker or kubectl during generation, and never exports sources automatically.
A Vite build must be produced explicitly by the operator using pinned npm
inputs. Generated Rust/frontend sources and public TOMLs must match the selected
built-in profile (CRLF/LF normalization is allowed for text). Existing no-config
and no-assets defaults remain optional and do not gain a token requirement.
Ordinary YT-097 local settings keep their app-root fallback unless the operator
explicitly selects the new runtime root.

Three physically separate contexts remain mandatory: generated configuration
as the only default context, a sanitized app-src export, and a sanitized
yetlet-src export. The inventory lists every selected file with byte length and
SHA-256. The default context permits only Dockerfile, overlay and its ignore
file. The app export contains the original lock, exact selected Rust sources,
optional built dist/manifest and optional four public TOMLs. Frontend sources,
node_modules, source maps, dotenv, credentials, .git and target are excluded.
Nothing recursively uploads the live checkout. Operators must physically reject
extra/missing/link/reparse/special files and compare every inventory hash before
passing any context to BuildKit; named-context ignore behavior is insufficient.

The original core/html/server source closure and workspace inheritance remain
unchanged. Config delivery adds only the exact isolated yetlet-config
Cargo.toml and src/lib.rs, with bounded native reads and no arbitrary new crate
or source lookup. The app overlay changes only the two framework dependency
paths and, when selected, the config dependency path. The image builds exactly
the requested native Rust binary with `--locked --release --bin NAME`.

Vite inspection bounds the manifest to 128 KiB, 128 records, 512 references,
256 distinct files, 1 MiB per native input and 16 MiB aggregate assets. Only flat
`dist/assets` JS/CSS/SVG files and `dist/.vite/manifest.json` are accepted.
References, entry types, cycles, CSS types and physical file lists are checked.
Missing, extra, source-map, unsafe path, symlink/reparse and oversized inputs
fail closed. Manifest and files must be at least as new as inspected generated
inputs; this catches stale output but is **not authenticated build provenance**.
The operator must trust/freeze the build tree. A second complete inspection
before native publication rejects changed source/config/frontend/build bytes.

## Runtime roots, secrets and retention

The image delivers assets to `/opt/yetlet/assets` and public configuration to
`/opt/yetlet/config`, selected through explicit absolute `YETLET_ASSET_ROOT`
and `YETLET_CONFIG_ROOT`. The YETLET config selector avoids colliding with typed
APP override keys. Relative and parent-traversing config roots fail with fixed,
non-sensitive errors. The asset adapter retains its existing absolute-root,
immutable startup snapshot, same-origin routes, MIME, strict CSP and no-store
behavior. No Node/Vite runtime or developer checkout path enters the runtime
configuration/inventory.

Only exact generated public base/development/staging/production TOMLs are
selected. Their token declaration must remain empty; token-bearing or edited
TOMLs are rejected. Operators can override public values through existing typed
APP environment keys. Config profiles explicitly select Production and
APP_SERVER__HOST=0.0.0.0 / APP_SERVER__PORT=8080; legacy host/port defaults and
health probes stay consistent. No configuration is mandatory for plain profiles.

The token is never read by the generator. Docker supplies no token default.
Compose declares required host `${APP_SERVICE__TOKEN:?Set APP_SERVICE__TOKEN}`
interpolation. Kubernetes uses only a non-optional existing Secret reference
`PACKAGE-runtime`, key `service-token`, where PACKAGE is the unchanged package
name. The operator deliberately selects that name/key or reviews/edits the
reference to an independently managed Secret; generation creates no Secret.
Never put token values in source, inventory, Dockerfile, YAML or command history.
Do not select development-secret files in a cloud image.

Kubernetes remains JSON-as-YAML with the same selected image, ports, probes,
non-root identity, read-only root filesystem, dropped capabilities and disabled
service-account token mount. Existing Docker and Compose process/signal behavior
is retained. No Kubernetes SDK, production-controller or hosting API is added.

An immutable image does not solve old HTML/lazy-chunk requests during rolling
replacement. Cache-Control remains no-store. Retain prior images and all old
static/lazy assets through the entire client/rollout lifetime, or use a
coordinated stop-and-replace preview with no surviving old clients. Shared asset
retention, rolling/HA qualification, immutable HTTP caching and production
availability remain open operational decisions, not guarantees of these files.

## Local execution evidence — 2026-10-17

Commands ran on Linux with Cargo/Rust 1.99, Node 24.21.0 and npm 12.2.0, using
existing cached dependencies only. The implementation did not install any global
package. Native Windows source adapters were updated but **Windows execution,
GNU/MSVC byte parity and native Windows runtime behavior are unverified** here.

- `cargo test --offline --locked --manifest-path tools/yetlet-cli/Cargo.toml`:
  57 unit tests and 19 integration tests PASS; three explicitly gated generated
  app tests are ignored by this default command. Windows-only tests do not run.
- `cargo clippy --offline --locked --manifest-path tools/yetlet-cli/Cargo.toml
  --all-targets -- -D warnings`: PASS.
- `cargo fmt --manifest-path tools/yetlet-cli/Cargo.toml -- --check`: PASS.
- `cargo check --offline --locked --manifest-path tools/yetlet-cli/Cargo.toml
  --target x86_64-pc-windows-msvc --all-targets`: PASS using the installed target.
  This is cross-platform type checking only, not native Windows execution.
- Five disposable real generated cloud apps (plain Vite, plugin Vite, each with
  config, and config-only) passed `cargo generate-lockfile --offline`,
  `cargo fmt --all -- --check`, `cargo test --offline --locked` and strict
  all-target Clippy. Each Vite app separately passed operator-invoked
  `npm ci --offline --no-audit --no-fund`, `npm run typecheck` and `npm run build`.
  Vite apps passed 10 native tests each without config and 14 each with config;
  config-only passed four. These include in-process route/CSP/MIME/snapshot
  tests against actual built assets, not a live container/browser claim.
- All five real optional apps generated all three output kinds: 15/15 PASS.
  The original legacy app was generated before implementation and again after:
  all 8 Docker, 2 Compose and 3 Kubernetes files, including inventory bytes,
  compare byte-for-byte equal (13/13).
- Focused generator tests cover all five profile combinations, deterministic
  inventories/per-file hashes, runtime roots, required Compose interpolation,
  Kubernetes existing-Secret references and optional absence, synthetic secret
  exclusions, stale/missing/oversized/malicious manifest/build files, changed
  config crate closure, symlink paths and source-change revalidation. Existing
  native writer race/no-overwrite and legacy regression tests continue to pass.
- The existing opt-in `generated_config_apps_offline_contract` integration test
  also passed explicitly with `--test config generated_config_apps_offline_contract
  -- --ignored --nocapture`: both local/config and cloud/config generated apps
  passed offline fmt/check/test/strict Clippy/build, fixed-error startup failure
  cases and (for cloud) all three generator commands. Its HTTP/signals lane was
  intentionally not enabled.
- A disposable physical app-src export (18 files) and yetlet-src export (16
  files) for the real configured-plugin app matched every inventory byte count
  and SHA-256 with no extra files. After translating only the three `/work/yetlet`
  dependency roots to the disposable local export path, `cargo metadata
  --offline --locked --format-version 1` and `cargo build --offline --locked
  --release --bin configured-plugin` passed. This proves the sanitized native
  source closure locally; it is not an actual OCI build/runtime proof.
- `cargo xtask check docs` and `cargo xtask check samples`: PASS.

The first fixture-based matrix run correctly refused group-writable synthetic
dist directories. One fixture-only correction creates protected directories;
the focused rerun passed. Product repair 1 corrected real Cargo lock semantics:
Vite's app lock entry includes tempfile/tower dev dependencies, and the pinned
TOML package's actual version is `0.9.12+spec-1.1.0`. The structural fixture now
uses those real semantics. No generator validation was weakened to accept
arbitrary sources or dependencies. The initial baseline command also rejected
an invalid image spelling; one command-only retry used the existing valid
`localhost:5000/yt101:local` grammar. Repair budget used: 1/2; operational retries
used: 1/2. These histories do not stand in for independent review.

Local command logs/results remain in `/tmp/yt101-work/logs`,
`local-results.json`, `generation-results.json` and `legacy-parity-hashes.json`
under that task-owned parent. All 30 manually created generated-app/output,
source-context and build directories were removed after verification. TempDir
fixtures cleaned their owned trees; all one-shot commands exited, and the
existing config test reaped its startup-failure children. No application server,
watcher, browser, container or cluster resource was started by this Job.
BACKLOG/ROADMAP bytes outside M18 compare equal to their pre-work snapshots;
unrelated artifacts were neither read nor modified.

## Reproduce and hand off to the parent live lane

Use a fresh private operator-owned parent (0700 / protected local NTFS ACL).
On Linux select `umask 077` before explicitly building dist, so native input
directories remain protected. Do not modify an existing generated destination.
For example, with a separately built local CLI and absolute literal paths:

```text
yetlet new demo --framework-path FRAMEWORK --dir PRIVATE_PARENT --cloud --assets vite --with-tailwind --with-config
```

Omit `--with-tailwind` for plain Vite, `--with-config` for no config, or both
asset flags for config-only. In the app, explicitly run:

```sh
npm ci --offline --no-audit --no-fund
npm run typecheck
npm run build
cargo generate-lockfile --offline
cargo test --offline --locked
```

Skip npm entirely for config-only. Offline commands require an already populated
operator-selected public dependency cache; do not silently fetch missing inputs.
The generator commands remain unchanged:

```text
yetlet generate docker --manifest-path APP/Cargo.toml --output NEW_DOCKER_DIR
yetlet generate compose --manifest-path APP/Cargo.toml --output NEW_COMPOSE_DIR --image localhost:5000/team/demo:yt101
yetlet generate k8s --manifest-path APP/Cargo.toml --output NEW_K8S_DIR --image localhost:5000/team/demo:yt101
```

Follow the generated Docker README's exact physical allowlist/hash recipe for
all three contexts before any BuildKit upload. Its image-build command and
optional runtime notes are also in the repository
[delivery template](../tools/yetlet-cli/templates/docker/DELIVERY.md) and
[runtime notes](../tools/yetlet-cli/templates/docker/PROFILE-RUNTIME.md).
Keep the same separately built image selection in Compose and Kubernetes.
Supply a runtime token only for config profiles and select the existing
Kubernetes Secret deliberately. Record actual engine/BuildKit/Compose/K3s
versions and native exit codes in the parent's bounded live evidence.

**Historical implementation-only handoff (superseded):** The parent then
needed actual Linux Docker image/runtime and asset bytes, SSR/CSP/health/cleanup,
Compose interpolation/runtime, isolated K3s Pod/Service, selected Windows native
checks and independent Reviewer PASS. That implementation Job did not itself
certify those results. The later parent evidence and independent Reviewer PASS
are recorded below; this paragraph does not indicate a current open gate.
YT-102 is the next dependency-ready research item after YT-101, under separate
authority; it is not implemented here. YT-020 remains Blocked.

## Parent live Linux/Windows acceptance — 2026-10-17

These parent-owned validations supersede the implementation-job-only pending
gates above. The accepted uncommitted source remains on
`feat/yt090-configuration` at HEAD `2987e072c5ac035c3c7e3b55cb76c9ae17a92a9b`.
The independently hashed 726-file source snapshot, including flattened root
license symlinks for safe Windows extraction, is recorded in
`artifacts/yt101-native-20261017/yt101-source-manifest.json`.
Its ZIP SHA-256 is
`c561bdcccab5fcd5c01015c4f2c00e5d6872db2aec5d37f0df13c2a09764323e`.
All 726 file hashes were later rechecked against the actual Ubuntu worktree:
**726/726 unchanged**. Do not use committed HEAD alone as source identity:
YT-101 remains uncommitted.

- **Actual Docker Buildx** on Linux Docker 29.9.1 / Buildx 0.38.0: generated
  Cloud+Config+official-Tailwind Vite app, pinned offline npm install, TypeScript,
  Vite production build and offline Cargo lockfile PASS. Three *physically
  separate* operator-owned BuildKit contexts matched all inventory bytes and
  SHA-256: 3 default entries, 18 app-src entries and 16 yetlet-src entries.
  The generated Linux/amd64 Rust 1.99 Release Docker image built and loaded
  under `localhost:5000/yt101live:qa`. The local tag was never pushed.
- **Real Docker runtime:** initial container failed before listening because
  private 0600 asset/public TOML files and 0700 subdirectories remained
  unreadable under runtime UID 65532. Minimal product correction emits
  `RUN chmod -R a+rX` for *only* the selected public assets/config roots,
  **after copying and before USER 65532:65532**. This preserves private
  sanitized export permissions and byte-identical legacy cloud-only output.
  The focused generator regression/strict Clippy and actual image rebuild PASS.
  Final real non-root container PASS: meaningful escaped SSR without JS, exact
  strict production CSP, health live/ready HTTP 200, four HTML-linked CSS/JS/SVG
  URLs byte-equal to pinned Vite build with nosniff, and no synthetic token in
  rendered HTML. SIGTERM, container removal and localhost port cleanup PASS.
  See `artifacts/yt101-live-20261017-HknRjez4/docker-runtime-result-r3.json`.
- **Real Compose runtime** using the *same* image and a disposable named
  project: generated config, required synthetic runtime token interpolation,
  SSR/CSP, health, fetched immutable asset bytes and image identity PASS.
  `docker compose down`, project container removal and port cleanup PASS.
  See `artifacts/yt101-live-20261017-HknRjez4/compose-runtime-result.json`.
- **Real three-node K3s** v1.36.4+k3s1: same operator-owned image imported on
  all three existing laboratory nodes; isolated test namespace, synthetic
  existing-Secret reference, client and server dry-runs, real Deployment/Pod
  Ready, Service ClusterIP and direct Pod-IP SSR/CSP/health PASS. The direct
  Service and Pod paths independently fetched and byte-hash-matched **all
  seven reachable app.ts asset files**. The eighth built `companion` entry is
  *deliberately excluded* from the application-owned app.ts Snapshot and
  should remain 404, per the pre-existing YT-098/YT-099 security contract.
  Earlier Kubernetes API-server proxy 404 and then overbroad “all dist must be
  served” assertions were test-harness errors, not supported product failures.
  The final test's namespace, three task-only image imports and transfer archive
  were all removed; no existing OpenBao/Valkey resources were changed.
  Evidence: `artifacts/yt101-live-20261017-HknRjez4/k3s-runtime-result-r5.json`.
- **Windows MSVC Rust 1.99 native execution:** original verified snapshot
  **726/726 exact source hashes**, real CLI tests, strict all-target Clippy
  and Build PASS. The first generated-output attempt correctly refused an
  untrusted inherited TEMP directory ACL; the retry used an existing
  CLI-created protected parent, and actual Windows Config-only cloud
  Docker/Compose/K8s generation, lockfile and emitted runtime contracts PASS.
  The separate task scratch directories were cleaned. Evidence:
  `artifacts/yt101-native-20261017/windows-msvc-native.json` and
  `windows-msvc-protected-r2.json`, with their paired logs.
- **Historical first Windows GNU Rust 1.99 attempt:** the same source archive
  independently matched **726/726** hashes, but Cargo could not finish because
  the runner lacked `dlltool.exe` (`getrandom`/`windows-sys` dependency build
  failure). This was an operational toolchain limitation, not a Yetlet
  regression. GNU PASS was **not claimed at that time**; this initial failure
  is superseded by the supplemental GNU PASS below. Original evidence:
  `artifacts/yt101-native-20261017/windows-gnu-native.json` and paired log.

The parent runtime test briefly used synthetic local-only values and
task-owned containers/Namespaces; no real secrets, production deployment,
registry publication or release were involved. The first Docker runtime
failure was a bounded real product correction; earlier HTTP-header casing
and Vite asset-graph assumptions were QA harness corrections. Their failed
evidence remains preserved and must not be presented as successful tests.

**Final YT-101 state: Done** at the bounded opt-in cloud generator scope.
Real Linux Docker/Compose/three-node K3s runtime and Windows MSVC native gates
PASS, with 726/726 source identity and all task-owned runtime resources removed.
At the original independent Reviewer gate Windows GNU was still
toolchain-blocked; that historical state is superseded by the supplemental
Windows GNU native PASS recorded below.
The independent read-only, role-directed Reviewer Job
`wc_job_fYtD9F8KXkXyzvYQ` completed with process exit 0 and canonical
`overall_status=pass`, `issues=[]`, `required_followups=[]`. The saved
`reviewer` model was `gpt-6-astra` at resolver-selected `high` effort,
not native managed dispatch or current-session self-review. The original result
is in `artifacts/yt101-native-20261017/yt101-independent-reviewer.json`.
The reviewer independently inspected source and runtime/native proof, and
confirmed GNU's missing `dlltool.exe` is not an explicit blocking YT-101 gate.
All 17 scoped sources passed SHA-256 revalidation before independent review.

The remaining product limits are unchanged: no built-in immutable rolling
asset-retention store, supported Windows containers, production HA/availability
or release qualification. YT-102 remains Todo research after YT-101;
YT-092–YT-096 remain Todo and YT-020 Blocked. This work is **not staged,
committed, pushed, merged, released or deployed to production**.

## Supplemental Windows GNU native qualification — 2026-10-17

This is **additional post-Reviewer evidence**, not a new implementation or a
retroactive claim about the initial GNU run. The original GNU failure above
remains a preserved, accurate record of the previously missing toolchain.

The owner authorized installation of missing build tools on the already
registered Windows GNU Runner `NCUT-BOHE-2024`. Read-only inspection found
no GCC, linker or `dlltool.exe`; the runner already had the pinned
`1.99.0-x86_64-pc-windows-gnu` Rust toolchain. WinGet installed the
publisher's `MSYS2.MSYS2` **20260611** in the current user's profile after
verifying the installer SHA-256. MSYS2's package manager then installed only
`mingw-w64-x86_64-binutils` and `mingw-w64-x86_64-gcc` (after the official
package-system update). This provided GCC **16.2.0** and GNU Binutils
**2.47.20260726**, including `dlltool.exe`, `ld.exe`, `ar.exe` and `as.exe`.
The user's PATH now includes the MinGW64 binary directory. No administrator
account, system reboot, application launch, Windows-container build or Git
checkout mutation was needed. An already-running Runner process may require
a future restart to inherit the updated user PATH automatically; the proof
used an explicitly selected process-local PATH without restarting it.

The same frozen **726-file** Ubuntu source archive
(`yt101-source.zip`, SHA-256
`c561bdcccab5fcd5c01015c4f2c00e5d6872db2aec5d37f0df13c2a09764323e`)
was extracted into a new, task-owned Windows TEMP directory and every source
file was independently hashed (**726/726 exact matches**). Rust 1.99 GNU
passed `cargo test --offline --locked`, all-targets Strict Clippy,
`cargo build --offline --locked` and `cargo fmt --all -- --check` for the
CLI. A real Windows GNU generated `--cloud --with-config` consumer passed
its offline Cargo lockfile and native application tests. The unchanged public
CLI generated valid Docker, Compose and Kubernetes configuration in a
CLI-created protected parent, including non-root public config file
readability, required Compose runtime-token interpolation and a Kubernetes
existing-Secret reference. The temporary consumer and build directories were
removed and the original Windows Git checkout was untouched.

Evidence retained under `artifacts/yt101-native-20261017/`:

- `windows-gnu-revalidated-r2.json` and `windows-gnu-revalidated-r2.log`:
  actual GNU exit-zero tests, build, generator outputs and cleanup.
- `windows-gnu-msys2-install.log` and `windows-gnu-mingw64-packages.log`:
  the official package installation and version provenance.
- `windows-gnu-native.json` and `windows-gnu-native.log`: original failed
  GNU environment attempt, deliberately retained without rewriting history.

**Updated qualification:** Linux Docker/Compose/K3s and Windows native
**MSVC + GNU** acceptance are now PASS at their explicitly bounded scopes.
The earlier independent Reviewer PASS remains valid for unchanged YT-101
product source; the separate GNU evidence is an additional native run, not
attributed to a Reviewer who did not observe it. This remains an unpublished
experimental framework: no Windows container deployment or production
rolling-update asset-retention/HA guarantee is claimed. No Commit or Push.
