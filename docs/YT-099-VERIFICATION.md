# YT-099 — Opt-in Vite production CLI scaffolding

Date: 2026-10-17. Base and unchanged HEAD:
`4b6975b1c6c0de0c202149ea58f3e109b56531af`, branch
`feat/yt090-configuration`, existing Yetlet repository checkout.
**Done at bounded experimental production-scaffold scope: Plain Vite and
Cloud+Config+Tailwind Plugin Chrome checks with JavaScript on/off and fresh
independent Reviewer acceptance passed.** The earlier implementation,
Windows qualification, operational failures and source-review sections are
historical evidence; their outstanding-browser statements describe the earlier
execution environment. No prior failure or qualification boundary is erased.

## Authority, scope and preserved work

The owner authorized YT-099 after the accepted YT-098 commit, without a new
commit. Implementation ran in the same checkout through a separate
role-directed Codex Job `wc_job_QDSzB9-CAk-CvB1-` with the workspace-resolved
registered generalist model `gpt-6.1-sol`. The shared reasoning resolver used
`inherit` mode, with no effort selector; neither native managed `agent_type`
identity nor independently enforced effort is claimed. The executor read the
registered generalist definition and Materiality Gate and returned a
`status=partial` JSON result with both product repairs used. Its output does
not itself satisfy the separate Reviewer requirement.
ADR 0005/0006, the YT-099 row, YT-098 verification, neutral adapter sources and
predecessor Vite experiment were read before implementation. The previously
accepted spike/YT-098 browser and Reviewer results are prerequisites, not browser
or review evidence for this changed source.

Only the YT-099 row and the M18 roadmap table line were edited in the existing
planning files. The pre-existing M19/M20 additions, all five YT-092–YT-096 Todo
rows, and YT-020 Blocked were preserved byte-for-byte. Existing untracked
`artifacts/` and ignored secrets were not opened, copied, edited or removed.
The original no-assets templates, 12 canonical samples, root Cargo files,
Core/browser and both predecessor experiments remain unchanged. No worktree,
clone, branch, stage, commit, push, merge, release, deployment, external service
or global install was performed. The original Ubuntu executor operated from
cached public packages; later separate Windows native test Jobs explicitly
populated ordinary user Cargo/npm caches as noted below.

Changed existing files:

- `tools/yetlet-cli/src/lib.rs`: additive `Cli::NewVite`, exact `--assets vite`
  parsing and help. Existing no-assets `Cli::New` equality is retained.
- `tools/yetlet-cli/src/main.rs`: dispatch the opt-in variant.
- `tools/yetlet-cli/src/scaffold.rs`: additive `create_vite`; existing `create`,
  `create_cloud`, `create_config`, `create_tailwind` signatures and bytes remain.
- `tools/yetlet-cli/src/platform/fs/mod.rs`: add only `src/vite.rs` to the fixed
  source-path allowlist. Native Linux/Windows writers and rollback are unchanged.
- `tools/yetlet-cli/README.md`, `docs/BACKLOG.md`, `docs/ROADMAP.md`.

New files:

- `tools/yetlet-cli/tests/vite.rs`: focused parser, inventory/tool-independence,
  no-overwrite and Linux protected-path tests; common tests also target Windows.
- `tools/yetlet-cli/templates/vite/README.md`, `handlers.rs`, `vite.rs`.
- In that same template directory: `app.ts`, `companion.ts`, `shared.ts`,
  `lazy.ts`, `plain.css`, `tailwind.css`, `shared.css`, `lazy.css`, `mark.svg`,
  `tsconfig.json`, `vite.config.mjs`, `plain-package.json`,
  `plain-package-lock.json`, `tailwind-package.json`,
  `tailwind-package-lock.json`.
- This verification record and later parent-added
  `tools/yetlet-cli/tests/vite-browser.mjs`, a focused opt-in browser product
  test using existing repository Playwright Core only (not scaffold output).
  Its Node syntax test passes; live browser execution remains unverified.

No generic harness, validator, certification framework or public assets crate
was added. Generated `src/vite.rs` embeds the existing YT-098 neutral loader and
its focused product fixture in one application-owned module. This is a copied,
unpublished implementation, not a shared stable API or retroactive YT-098 claim.

## Generated contract

```text
yetlet new NAME --framework-path PATH [--dir DIRECTORY] [--cloud] [--with-config] --assets vite [--with-tailwind]
```

The Vite profile has exactly 17 files (23 with config), under root and `src/`
only. There is no generated frontend/dist/node_modules directory or Cargo.lock.
Scaffolding copies one of two pinned frontend locks and runs no subprocess,
resolution, npm, network, install, compiler or application build. Unknown,
duplicate, malformed and misplaced options fail closed. The eight established
no-assets profiles keep their exact existing inventories and all 64 file bytes.

Plain Vite pins Vite 8.3.4 and TypeScript 7.0.2 without any Tailwind dependency.
The selected plugin profile additionally pins Tailwind and `@tailwindcss/vite`
4.3.3. It never selects the standalone compiler or dual CSS compilation.
Both locks retain public-registry resolved URLs and SHA-512 integrity entries.
The Tailwind lock is the trusted spike lock with only its Playwright QA package
removed and root metadata aligned. The plain lock was pruned from that trusted
lock by an explicit operator-invoked offline, scripts-disabled package-lock-only
preparation command. Neither operation occurs in `yetlet new`.

The frontend files are root-based. Vite 8 uses `rolldownOptions` with real
`app.ts` and companion entries, shared static JS/CSS/SVG, and lazy JS/CSS.
`envDir: false`, `publicDir: false`, `assetsInlineLimit: 0`, disabled module
preload polyfill and source maps keep production outputs external and explicit.
No dev server/HMR mutation is generated. Deliberately referenced `VITE_*` values
remain public; users must never put secrets there or in frontend source.

Startup requires explicit absolute `YETLET_ASSET_ROOT` pointing at the trusted,
quiescent dist. It validates and snapshots `.vite/manifest.json` and reachable
safe CSS/JS/SVG bytes once before listening. Bounds: 128 KiB manifest, 128 nodes,
512 edges, 256 paths, 2 MiB/file and 16 MiB total. Unknown schema, unsafe metadata,
paths/types, symlinks below root, missing references/files and cycles fail closed.
The selected entry must be JavaScript and the generated consumer expects one
static SVG. No HTTP request opens files or invokes a compiler.

Static dependency-first DFS preserves import/CSS array order and deduplicates
first occurrence. Dynamic CSS is excluded from initial links. App-owned Axum
State/Arc composes the snapshot with typed settings; no framework DI is added.
Escaped meaningful Rust SSR, the exact DEFAULT_CSP, same-origin `/static/`
whitelisting, fixed MIME, nosniff and no-store are preserved. Config retains
SecretString/startup validation, and cloud retains bind/health/signal/drain code.
Existing Docker/Compose/Kubernetes generation rejects the new source contract.
No-store does not guarantee rolling old-hash retention; YT-101 must address that
separately. This is not atomic hostile-filesystem isolation or SVG sanitization.

## Observed verification

Ignored evidence root: `target/m18/yt099/`. Native command results are recorded
with exact arguments, output and exit codes; generated apps remain available for
the parent. Tool versions: Node v24.21.0, npm 12.2.0, existing Rust 1.99 toolchain.
All dependency operations were offline using existing cached public packages.

| Command / boundary | Actual result |
|---|---|
| `git rev-parse HEAD`, `git branch --show-current`, `git status --short` | Required HEAD/branch; initially only the two pre-existing planning edits and untracked artifacts |
| Before/after compiled CLI generation of eight no-assets profiles | PASS: exact inventories and SHA-256 equal for all 64 files; `legacy-before.json`, `legacy-compatibility.json` |
| `cargo test --offline --locked --manifest-path tools/yetlet-cli/Cargo.toml --test vite` | Exit 0: 3 passed; eight combinations, exact New/NewVite results, malformed/misplaced/duplicate/non-UTF8 values, exact inventories, no tools/cache/overwrite and Linux protection |
| `cargo test --offline --locked --manifest-path tools/yetlet-cli/Cargo.toml` | Final exit 0: 68 passed, 3 existing opt-in tests ignored; native Windows-only tests compile to zero tests on Linux, not Windows evidence |
| CLI `cargo fmt -- --check` and `cargo clippy --offline --locked --all-targets -- -D warnings` with the same manifest | Final exits 0 |
| In each of eight final generated apps: explicit `npm ci --offline --ignore-scripts --no-audit --no-fund --cache target/m18/yt099/npm-cache` (normalized repo-relative representation of the selected task cache), `npm run typecheck`, `npm run build` | All 24 commands exit 0; actual pinned plain/plugin outputs, each with 9 files including real manifest/entry/shared/lazy JS/CSS and external SVG |
| In each app: explicit `cargo generate-lockfile --offline` | All 8 exit 0; app-owned lockfiles, not copied framework locks |
| In each app: `cargo fmt --all -- --check`, `cargo check --offline --locked --all-targets`, `cargo test --offline --locked`, `cargo clippy --offline --locked --all-targets -- -D warnings`, `cargo build --offline --locked` | All 40 commands exit 0; 92 tests total (10 per no-config app, 13 per config app), zero failed/ignored |
| Repeat ci/typecheck/build in plain native and plugin+config+cloud consumers | All 6 commands exit 0; lock bytes and all 9 dist files per consumer identical; synthetic backend sentinel absent |
| Actual binary startup with absent selection, relative root or unbuilt root; config blank token or invalid port with valid assets | 32/32 expected nonzero exits before Listening; bounded errors, synthetic token absent |
| Actual CLI generate docker/compose/k8s against all eight locked apps | 24/24 expected exit 1; exact-source layout refusal and no output directory |
| Native loopback capability probe | BLOCKED: socket creation itself raises `PermissionError: [Errno 1] Operation not permitted`; no listener, HTTP or browser started |

Product tests verify schema/entry rules, graph/size/file bounds, traversal and
symlink refusal, deterministic CSS order/dedup, no initial lazy CSS, actual built
manifest/static/dynamic assets, strict headers and escaped HTML through native
Axum in-process service calls. This proves response behavior without a socket;
it does not prove computed browser CSS, actual network interaction or shutdown.

Key evidence files: `final-app-checks.json`, `frontend-reproducibility.json`,
`startup-generator-refusals.json`, `loopback-probe.json`, `preservation.json`,
`cli-all-tests-final.log`, `cli-clippy-final.log`, and the per-app final logs.
The docs/sample checks and final preservation/cleanup results are recorded below.

## Historical failures and bounded accounting

- Initial template assembly failed rustfmt with E0753 (an inner doc comment after
  a module). Product repair 1 corrected the copied-module integration and import.
- Initial generated-app fmt reported wrapping/import differences for all eight
  main files. Initial generated config-only apps also had an unused Arc import;
  strict Clippy exited 101 there. Product repair 2 corrected only the opt-in copy.
  The original failed logs and `initial-native.json` / `initial-app-tests.json`
  are retained; final successes do not erase those failures.
- One lock-preparation npm command could not write the read-only global npm
  cache (`EROFS`). This was operational, not a product defect. One bounded cache
  retry copied only integrity-addressed public package blobs into the task's
  writable cache. The template dependency set was preserved; no registry fetch.
  Offline ci then passed for both locks and all eight generated apps.
- Loopback EPERM is a separate operational blocker. It was not retried and did
  not trigger product changes, browser downloads, policy bypass or a new run.

Repair attempts used: **2/2**. Operational retries used: **1/2**.
No further product repair is authorized within this executor attempt. Last
unresolved failure signature: `socket.socket: EPERM / Operation not permitted`.

## Remaining parent-owned gates and handoff

1. On a valid native Ubuntu runner, test these exact generated sources/binaries
   with actual HTTP and Chrome/Playwright: plain and plugin consumers, JavaScript
   on/off, meaningful escaped SSR, computed plain/Tailwind/shared styles, SVG,
   two lazy clicks, lazy CSS/JS MIME, zero CSP/console errors, same-origin paths,
   secret exclusion, config+cloud health and SIGTERM/SIGINT drain, and cleanup.
   Use the existing minimal browser fixture pattern; no new generic harness.
2. **Parent native Windows GNU/MSVC qualification is now complete**, as recorded
   below, without any change to the original application-owned loader or CLI
   runtime. The source-derived Windows-only unused-import issue in
   `tests/vite.rs` was corrected with the smallest `#[cfg(target_os = "linux")]`
   test import gate; this is a targeted test harness correction after the
   executor's 2/2 product repair budget, not an additional application fix.
   Linux fmt/Clippy and focused tests were rerun before the source-bound ZIP.
   Actual Windows TCP listeners, console signals and browser journeys remain
   distinct unverified gates.
3. Parent dispatched an independent read-only Reviewer which checked 28/28
   changed source hashes, found no material code defect, but returned overall
   FAIL solely for missing real HTTP/Chrome evidence. A permitted browser run
   and fresh independent acceptance of that evidence remain required before
   Done; the earlier YT-098 Reviewer result does not certify this source.

Final apps: `target/m18/yt099/final-apps/vite-CLOUD-CONFIG-TAILWIND/` (each bit 0
or 1). Binaries: `target/m18/yt099/cargo-target/debug/vite-CLOUD-CONFIG-TAILWIND`.
Use an absolute per-app `dist` for `YETLET_ASSET_ROOT`; config apps require a
synthetic operator-selected `APP_SERVICE__TOKEN`, with `APP_SERVER__PORT` rather
than `YETLET_PORT`. Cloud no-config uses `YETLET_HOST`/`YETLET_PORT`. Their
READMEs document explicit frontend/Cargo steps and Windows syntax. Rebuild from
current templates for Windows rather than copying Ubuntu binaries.

YT-100 and YT-101 remain Todo and depend on accepted YT-099. YT-092 is the next
independently dependency-ready backlog item (YT-090/YT-097/YT-098 are Done), but
it is outside this task. No open architectural decision or public API promotion
is needed to finish the remaining YT-099 verification gates.

## Final repository and cleanup checks

- `CARGO_NET_OFFLINE=true cargo xtask check docs`: exit 0; language, links,
  backlog and roadmap checks passed.
- `CARGO_NET_OFFLINE=true cargo xtask check samples`: exit 0; all 12 canonical
  samples passed without catalog or sample changes.
- `git diff --check`: exit 0. Root/Core/browser and both predecessor experiment
  diffs are empty; the original scaffold implementation is an unchanged prefix
  with only the additive Vite implementation appended.
- `preservation.json`: pre-existing M19/M20 sections are byte-identical, all five
  rows remain Todo and YT-020 remains Blocked. The index remains unchanged.
- `cleanup.json`: PASS; all one-shot builds/tests completed, no task process
  remains, and no server, watcher or browser was started. Generated sources,
  node_modules, dist, locks and logs under the ignored evidence root are retained
  for parent verification; no user files were cleaned.

No commit/push/merge/release was performed. The corrected Windows test import
and source-bound GNU/MSVC results close the native Windows build gate. The
independent source review result is recorded below; actual Chrome evidence and
its follow-up acceptance remain the outstanding completion requirement.

## Parent source-bound Windows GNU/MSVC validation

The parent created a sanitized source ZIP at
artifacts/yt099-native-validation-20261017/yt099-source.zip, with SHA-256
939220f35113ca0b6702b2ffa122ca62d862e37faeec0ffc801757840285ed9e.
It contained exactly 557 source files with individually verified SHA-256
identities, including both pinned Vite npm lockfiles and the narrow Windows
test-only cfg import fix. Original user artifacts, ignored secrets and other
checkouts were not included. Only this documentation and the focused
browser QA script were added after this native source snapshot; the CLI
executable code and generated-app templates remained unchanged.

Both Windows Runners verified every source member before running native Rust
1.99 and pinned Node 24. GNU reactivated previously provisioned user-local
MinGW binutils only in its test process. Both independently ran the CLI
all-targets tests, strict Clippy, optional configuration tests, and TWO
representative real generated apps: Plain Vite and Cloud+Config+Tailwind Plugin.
Each generated app passed explicit npm ci, TypeScript check, Vite production
build, Cargo lock/fmt/check/test/Clippy/build. The generator itself ran no npm,
Cargo, network requests or asset builds. All task-owned temporary source and
target directories were cleaned at completion.

| Platform | Native WebCodex Job | Result |
|---|---|---|
| Windows GNU, Rust 1.99 | wc_job_cdz8Rnld4vwcuUXh | PASS: 557/557 hashes, CLI tests/Clippy, both generated Vite app builds and tests, cleanup |
| Windows MSVC, Rust 1.99 | wc_job_Hm6tlHKGM2mXyeZh | PASS: same source hashes, source checks, builds/tests, cleanup |

This is real Windows compilation/execution of Cargo and frontend builds,
not Windows external TCP, GUI/Chrome, Ctrl-C, adversarial NTFS ACL race
qualification or production deployment. No such unsupported claim is made.

## Product-focused Linux Chrome verification (not yet run)

The parent added the task-owned test file
tools/yetlet-cli/tests/vite-browser.mjs. Node syntax checking passed, but
no real browser journey has been executed on the new YT-099 source. The script
owns its launched backend and Chrome and captures bounded browser JSON,
screenshots and backend output under a newly selected output directory.
It checks actual strict CSP, script/no-script escaped Rust SSR, same-origin
CSS/JS/SVG, computed styles, lazy CSS loading/order/dedup, headers,
synthetic-only configuration token isolation and cloud health.

On an authorized Ubuntu host with the existing pinned Playwright Core
dependency and real Chrome, from the repository root, select unused loopback
ports and run these two examples (do not overwrite existing evidence):

~~~sh
export PATH="$HOME/.cargo/bin:$HOME/.nvm/versions/node/v24.21.0/bin:$PATH"
OUT="$(mktemp -d /tmp/yt099-browser-qa.XXXXXX)"
node tools/yetlet-cli/tests/vite-browser.mjs \
  --binary "$PWD/target/m18/yt099/cargo-target/debug/vite-0-0-0" \
  --dist "$PWD/target/m18/yt099/final-apps/vite-0-0-0/dist" \
  --mode plain --port 3048 --out "$OUT/plain"
node tools/yetlet-cli/tests/vite-browser.mjs \
  --binary "$PWD/target/m18/yt099/cargo-target/debug/vite-1-1-1" \
  --dist "$PWD/target/m18/yt099/final-apps/vite-1-1-1/dist" \
  --mode tailwind --config --cloud --port 3049 --out "$OUT/tailwind"
~~~

The task's original Codex sandbox returned socket EPERM, and an earlier
platform safety restriction separately prevented coordinator browser launch
on YT-098. No denied action was retried using an alternate tool. Actual
Chrome evidence for YT-099 remains outstanding and may be collected only
when execution is explicitly permitted. A fresh independent Reviewer must
assess the source-matched browser evidence before YT-099 can become Done.
YT-100/YT-101 and YT-092..096 remain Todo and YT-020 remains Blocked.

## Independent Reviewer closeout — outstanding browser evidence

A separate read-only registered Reviewer Job
`wc_job_3DoleYAMuU-spO_G` completed with process exit 0. It inspected
all 28 changed/new YT-099 files and related existing contracts. All 28
review-input SHA-256 identities matched at the beginning and end, with no
source drift, stage, commit or external mutation. It independently checked
the legacy eight profiles' 64 byte-identical files, native Linux generation
records and task-specific Rust source. Windows GNU/MSVC native Job results are
reported separately above; Reviewer did not rerun those Jobs.

The Reviewer's **overall_status=fail** is solely one
**[evidence][P2]**: the original required actual HTTP/Chrome checks have
not run against these newly generated Vite applications. There was no
material P0-P2 source-logic or security finding in this review. In-process
Axum tests, CLI/Windows native builds and an earlier YT-098 Chrome PASS
do not substitute for YT-099 browser-computed CSS, lazy JS/CSS order and
dedup, CSP enforcement, JS-disabled SSR, same-origin requests and owned
server/Chrome cleanup. The Codex sandbox denied local socket creation;
the parent did not bypass the prior platform-level browser-operation
refusal. The follow-up is a real, source-matched browser acceptance run
only once permitted, followed by independent verification of that result.

This was an ordinary read-only, role-directed `gpt-6-astra` process under
the inherited-effort policy, not formal release assurance or native managed
role identity. The Reviewer did not launch browsers, modify files or access
private secrets. After its verdict, only this factual documentation and
the M18 Backlog/Roadmap status text were changed; compiled/Windows-tested
Rust/CLI, Vite template, lockfile and QA source bytes remain unchanged.

**YT-099 remains In progress.** YT-100/YT-101 and YT-092..096 remain
Todo, YT-020 remains Blocked, and no commit/push/merge/release/deployment
was performed.

## Local Ubuntu Chrome continuation — 2026-10-17

This continuation used the existing checkout, required branch and unchanged
base HEAD above. The initial Git status matched the handoff and the index was
empty. All 25 CLI/template/test/guide files matched the prior independent
review's SHA-256 identities; only Backlog, Roadmap and this verification record
had changed after that review. Both existing representative binaries and their
nine-file dist trees were present. No source reimplementation, rebuild,
installation, clone, worktree, core/default change or external operation was
needed to run the browser gate.

New retained evidence root, relative to the repository:
`target/m18/yt099/local-acceptance-1o5mwmsr/`.
Fresh original browser output: `/tmp/yt099-browser-local.i_d7ufrc/`.
The browser JSON, backend logs and screenshots were copied byte-for-byte into
`browser/plain/` and `browser/tailwind/` under the retained evidence root.
Historical failed evidence, Windows evidence references, previous reviewer
results and existing artifacts remain intact.

### Actual Chrome commands and results

From the repository root, using the already provisioned tools:

```sh
export PATH="$HOME/.cargo/bin:$HOME/.nvm/versions/node/v24.21.0/bin:$PATH"
node tools/yetlet-cli/tests/vite-browser.mjs \
  --binary "$PWD/target/m18/yt099/cargo-target/debug/vite-0-0-0" \
  --dist "$PWD/target/m18/yt099/final-apps/vite-0-0-0/dist" \
  --mode plain --port 3048 \
  --out /tmp/yt099-browser-local.i_d7ufrc/plain
node tools/yetlet-cli/tests/vite-browser.mjs \
  --binary "$PWD/target/m18/yt099/cargo-target/debug/vite-1-1-1" \
  --dist "$PWD/target/m18/yt099/final-apps/vite-1-1-1/dist" \
  --mode tailwind --config --cloud --port 3049 \
  --out /tmp/yt099-browser-local.i_d7ufrc/tailwind
```

Both commands returned native **exit 0** using real installed **Google Chrome
155.0.8059.84**, headless through the existing pinned Playwright Core. Each
`browser.json` reports `status=pass`, `js_on=pass` and `js_off=pass`:

| Generated consumer | JavaScript enabled | JavaScript disabled |
|---|---|---|
| Plain `vite-0-0-0` | PASS: escaped Rust SSR, TypeScript, computed plain/shared CSS and SVG, two lazy interactions | PASS: meaningful escaped SSR, computed CSS/SVG, lazy content unchanged |
| Cloud+Config+Tailwind Plugin `vite-1-1-1` | PASS: typed production configuration, computed Tailwind/shared CSS and SVG, two lazy interactions | PASS: meaningful escaped SSR and computed Tailwind/shared CSS/SVG |

Both journeys verified dependency-first initial CSS order and dedup, no lazy
asset response before interaction, stable initial stylesheet prefix after
loading, one added lazy stylesheet and no duplicate CSS. The exact strict CSP
matched; same-origin-only routing, CSP event arrays and console/page error
checks passed. The emitted JS/CSS/SVG response MIME values were additionally
checked against the recorded real responses. All responses retained nosniff
and no-store. Selected missing, encoded and manifest asset URLs returned 404.
The config/cloud consumer passed `/health/live` and `/health/ready` HTTP 200.

The harness supplied only an allowlisted environment and the synthetic token
`000123-yt099-synthetic-only`; no local secrets were supplied. That sentinel
was absent from HTML, all 18 actual dist files and backend logs. The logs
contain only the expected local Listening messages. Both screenshots were
visually inspected and show the expected styled SSR, SVG and loaded lazy result;
no sensitive content was found in the retained logs, JSON or screenshots.

`browser-inputs.json` records **87 SHA-256 identities**: 25 existing scoped
CLI/template/test/guide inputs, two actual native binaries, all 18 dist files
and 42 generated application source/lock files. All matched after browser and
native execution. `browser-summary.json`, `browser-sha256.json` and
`browser-commands.json` record assertions, output identities, exact argv, native
exit codes, selected ports and UTC execution times.

| Evidence | SHA-256 |
|---|---|
| `browser-inputs.json` | `d830987a2b748967ea113bac5780eff30ef6bf29dcb71f790516622f02647c3f` |
| `browser/plain/browser.json` | `bd9a2298397ceb509c5bbcc85fbb410728da21450935d6cd7ddb38c7fca1be35` |
| `browser/tailwind/browser.json` | `9e028247588cc9355069adad9b4edc70bc72000793534c83ecf5ffcd51f4f577` |
| `browser/plain/plain.png` | `823a889c03e3e83054e86f4d3e0a9ed5fd1b13fabbed0c1e2228b61a683220be` |
| `browser/tailwind/tailwind.png` | `b8a489c6ab6268584eab8718751c8a2b043eae9d4753b84b52b9c1460e1a569b` |

### Native regressions and owned-process cleanup

Fresh commands, all with native **exit 0**:

```sh
cargo test --offline --locked --manifest-path tools/yetlet-cli/Cargo.toml --test vite
CARGO_TARGET_DIR="$PWD/target/m18/yt099/cargo-target" cargo test --offline --locked \
  --manifest-path target/m18/yt099/final-apps/vite-0-0-0/Cargo.toml
CARGO_TARGET_DIR="$PWD/target/m18/yt099/cargo-target" cargo test --offline --locked \
  --manifest-path target/m18/yt099/final-apps/vite-1-1-1/Cargo.toml
```

The focused CLI suite passed **3/3**, covering eight-profile inventory, argument
compatibility and protected writer behavior. The representative generated apps
passed **10/10** and **13/13**, including real built manifests, bounded snapshots,
strict routes, escaping and typed configuration. `native-results.json` and the
three logs retain full commands, environment overrides and native statuses.
The earlier 68-test CLI / 92-test generated-app results, eight frontend builds,
legacy 64-file parity and native Windows qualification remain historical
source-bound evidence; this continuation does not relabel them as fresh runs.

The browser harness closed both Chrome instances and both backends. Plain
native SSR has no cloud signal handler and ended by the owned SIGTERM signal
(`server_exit=null`, `server_signal=SIGTERM`). Cloud+Config ended gracefully
with `server_exit=0`, `server_signal=null` after SIGTERM. Both report
`browser_closed=true` and `server_closed=true`. Independent coordinator checks
found no listeners on ports 3048/3049 and no exact owned backend process.

A focused additional cloud SIGINT check launched the same reviewed binary and
dist with the harness-equivalent allowlisted synthetic production environment,
selected unused port 57117, waited for `/health/ready` HTTP 200, sent SIGINT
through Python `Popen.send_signal`, and observed exit **0** within six seconds.
The child was reaped and `ss` found no listener on 57117. The direct process
command was the absolute `target/m18/yt099/cargo-target/debug/vite-1-1-1`
binary, with no arguments, and cwd `target/m18/yt099/final-apps/vite-1-1-1`.
`cloud-sigint.json` and `cloud-sigint.log` retain its signal, selected port,
identity, health, status and cleanup evidence. No user service was terminated,
no browser download occurred and no permission restriction was bypassed.

There were no failed product/browser/native checks or product/harness edits in
this local continuation. The independent Reviewer subsequently
accepted the completed evidence as recorded below. Only after that PASS was
YT-099 marked Done and the M18/guide/README status text reconciled.
YT-100/YT-101 and YT-092–YT-096 remain Todo; YT-020 remains Blocked.

## Independent local acceptance and final closeout

Separate native registered Reviewer task
`/root/yt099_local_browser_acceptance_review`, agent ID
`01a1483a-1ff1-7001-b1e2-1329be571b7b`, returned:

```json
{"overall_status":"pass","issues":[],"required_followups":[]}
```

The workspace profile preflight was healthy/current/eligible/trusted
(`premium`, `openai@5`). The shared resolver selected ordinary ad-hoc review,
deep class, adaptive `high`, with no conflict or degradation. The native child
trace verified `agent_role=reviewer`, `model=gpt-6-astra`, effective effort
`high`, and role/model/effort matches all true. Reconciliation returned
`enforcement_status=enforced`. This was an independent read-only managed agent,
not current-session self-review, the CLI review command or formal assurance.

The Reviewer inspected the actual source/assertions, both browser results,
screenshots, commands, native logs and cleanup records. All 28 reviewed files,
21 evidence hashes and 87 browser-input identities matched at both start and
end. It independently checked all 24 recorded browser MIME/header responses,
token absence in all 18 dist files and three backend logs, and the absence of
listeners/owned backends on 3048, 3049 and 57117. The prior sole Evidence P2 is
closed; no material P0–P2 defect or required followup remains. No tests, services
or browser were launched by the Reviewer, and it did not inspect artifacts or
private secrets.

Retained independent-review evidence under the local evidence root:

- `review-inputs.json`: 28 source/document and 21 evidence identities;
  SHA-256 `97f5649ce5f295741549e494c9679a7637ef788939aa4bb2e3021ba5d0504266`.
- `reviewer-result.json`: exact structured decision and scoped notes;
  SHA-256 `f8c5c89d286bcf76a6451e8d11d5d3d7623fc452ba30c48427206e618ac30c17`.
- `reviewer-trace.json`: bounded native role/model/effective-effort verification.
- `reviewer-reasoning-decision.json` and `reviewer-reasoning-observed.json`:
  pre-dispatch selection and observed reconciliation.

Only after independent PASS were YT-099's Backlog state and M18 status updated.
The CLI guide and Vite template README changed only their obsolete status labels
and evidence-link wording. These two README edits followed the source review;
all **23** other CLI/template/test/lock files remain byte-identical to the
browser-tested/source-reviewed inputs. The two binaries, all 18 dist files and
all 42 tested generated-app source/lock files also remain byte-identical.
Thus **85 of 87** original browser-input identities still match; the two
intentional differences are only README documentation. Final identities are
recorded separately in `final-source-hashes.json`, rather than replacing the
historical reviewed manifest or claiming that its documentation hashes remain
current. No runtime product or browser-harness repair occurred.

The required post-label generator regression was rerun:

```sh
cargo test --offline --locked --manifest-path tools/yetlet-cli/Cargo.toml --test vite
```

It returned **exit 0, 3/3 PASS** after recompiling the CLI's embedded template
README. The eight-profile inventory, finite source contract, argument parsing
and protected writer checks passed (`cli-vite-after-doc-labels.json` / log).
No frontend or generated runtime source rebuild was needed for status labels.
From the original checkout, `cargo xtask check samples` returned **exit 0,
12 samples PASS** (`samples-final.json` / log).

### Final docs, scope and Git evidence

The final documentation check uses the existing unchanged checker against the
source-only input view `/tmp/yt099-local-doc-source-x9ft_ynk`:

```sh
CARGO_NET_OFFLINE=true CARGO_TARGET_DIR=/tmp/yt098-doc-target cargo xtask check docs
```

The target directory is the existing local xtask compiler cache; no historical
YT-098 evidence is overwritten. The input view contains current tracked source
plus all 28 task targets, excludes artifacts, Git metadata and ignored secrets,
and is neither a clone nor a worktree. Exact checkout/input-view byte equality
is checked for all 28 targets. `docs-final.json` and `docs-final.log` retain the
actual native result, document/item counts, command, environment and full output.
No checker changes or broader validation infrastructure were introduced.
Observed result: **exit 0, 75 documents / 101 Backlog items PASS**, including
language, internal links, Markdown structure, whitespace and dependency DAG.

`git diff --check` passed. All **21 untracked task files** were separately
checked with `git diff --no-index --check -- /dev/null <file>`, with no whitespace
diagnostics. Exit 1 is an expected content difference for these no-index checks
and is accepted only when diagnostics are empty; exit 0 is also accepted.
`diff-final.json` records the native statuses. `preservation.json` records exact
M19/M20 section/table-row preservation, unchanged prior sources and all required
states: YT-099 Done; YT-100/YT-101 and YT-092–YT-096 Todo; YT-020 Blocked.

The branch, base HEAD, empty index and Git modified/untracked path groups remain
those at handoff. This continuation changed only this record, Backlog, Roadmap
and the two README status descriptions; all pre-existing implementation changes
remain uncommitted and intact. Existing artifacts and local secrets were not
opened, copied, modified, staged or removed. No commit, push, merge, release,
deployment, global install or external-environment change occurred.

Windows GNU/MSVC compilation and source-bound execution remain the earlier
qualified evidence, with the later documentation-label delta stated above.
Windows TCP/browser/console-signal behavior, HMR, cloud packaging/rollout retention,
other browsers and production/public API qualification are not claimed.
M18 remains In progress because YT-100 and YT-101 remain Todo. The next asset-line
item is YT-100; YT-101 and the independently ready YT-092 may also be scheduled
under separate authorization. None was started by this acceptance work.
