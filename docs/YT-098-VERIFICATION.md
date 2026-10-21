# YT-098 — Application-owned producer-neutral asset proof

Date: 2026-10-17. Base/unchanged HEAD:
`c91dc12670387182bd369d7ab0bb2648cc1c0277`, branch
`feat/yt090-configuration`, existing Yetlet checkout (repository root).
**Done at the application-owned experimental scope: all four local Ubuntu
Chrome checks and the independent Reviewer passed.**
The earlier implementation, failed execution and source-review sections below
are historical evidence retained unchanged in meaning. Their pending-browser
statements describe that earlier environment, not the local continuation.
The accepted VITE-SPIKE acceptance gate was not rerun.

## Scope and exact source inventory

The [contract and extraction decision](ASSETS.md) keeps the implementation
application-owned. Both actual producers ground one narrow neutral module;
there is no new shared crate or ADR/API change. The experiment's standalone
Cargo workspace uses Rust 1.99 and locked offline dependencies, no new core,
CLI or browser dependency. The original experiment is used only as a build tool
and producer of real manifests/assets; its source/behavior stays unchanged.

New files:

- `experiments/asset-contract/Cargo.toml`, `Cargo.lock`, `README.md`
- `experiments/asset-contract/standalone.css`
- `experiments/asset-contract/src/assets.rs`
- `experiments/asset-contract/src/assets/tests.rs`
- `experiments/asset-contract/src/vite.rs`
- `experiments/asset-contract/src/standalone.rs`
- `experiments/asset-contract/src/main.rs`
- `experiments/asset-contract/src/tests.rs`
- `experiments/asset-contract/verify-browser.mjs`
- `docs/ASSETS.md`, `docs/YT-098-VERIFICATION.md`

Existing files changed by this executor: only YT-098's row in `docs/BACKLOG.md`
(to In progress with evidence) and M18's status/paragraph in `docs/ROADMAP.md`.
The parent's exact M19/M20 sections and all five YT-092 through YT-096 Todo
entries are preserved. Existing untracked `artifacts/` and ignored sensitive
local files are not opened, copied, edited, staged or removed. No Git branch,
worktree, clone, index update, commit, push, merge, publication or deployment.

Generated task tools are ignored `experiments/asset-contract/target/` and
`dist/standalone/assets/standalone.css`. The already ignored Vite `dist/` was
rebuilt using its existing `node_modules`; no installation or automatic fetch.
Task evidence is under `/tmp/yt098-browser/`. Existing artifacts are not evidence
inputs. No validation framework or pipeline scripts were added.

## Observed build/native evidence

Commands below were run from the existing checkout unless stated otherwise.
All Cargo commands use existing offline dependencies; no lockfile relaxation.

| Command | Actual result |
|---|---|
| `git rev-parse HEAD`, `git branch --show-current`, `git status --short` | Required HEAD/branch; initially only parent BACKLOG/ROADMAP modifications and untracked artifacts |
| `sha256sum /tmp/yt091-tailwindcss-linux-x64-4.3.3` | Exit 0; `dc61b3ac6b8c9ca874c0cc4c57b2409791a64c5540404ca5f5367360babc313a`, matches YT-091's verified 4.3.3 executable |
| `node --version`, `npm --version` | Exit 0; Node v24.21.0, npm 12.2.0 |
| In `experiments/vite-integration`: `npm run typecheck && npm run build` | Exit 0; both actual Vite 8.3.4 profiles emit manifest, entry/shared/lazy JS, entry/shared/lazy CSS and external SVG; Tailwind-Vite 4.3.3 is selected only for its variant |
| `cargo generate-lockfile --offline --manifest-path experiments/asset-contract/Cargo.toml` | Exit 0; new isolated lockfile, 56 packages compatible with Rust 1.99 |
| `/tmp/yt091-tailwindcss-linux-x64-4.3.3 --input experiments/asset-contract/standalone.css --output experiments/asset-contract/dist/standalone/assets/standalone.css --minify` | Exit 0; actual standalone compiler reports v4.3.3; CSS banner and Rust-scanned font-bold utility verified in native consumer test; no Node/Vite compilation for this output |
| `BACKEND_ONLY_SECRET=YT098_SYNTHETIC_SERVER_ONLY cargo test --offline --locked --manifest-path experiments/asset-contract/Cargo.toml` | Final exit 0; 11 passed, 0 failed/ignored. Reads both actual Vite profiles and actual standalone CSS, tests independent entries and synthetic sentinel exclusion |
| `cargo fmt --manifest-path experiments/asset-contract/Cargo.toml -- --check` | Exit 0 |
| `cargo clippy --offline --locked --manifest-path experiments/asset-contract/Cargo.toml --all-targets -- -D warnings` | Exit 0 |
| `cargo build --offline --locked --manifest-path experiments/asset-contract/Cargo.toml` | Exit 0; native server binary built |
| `node --check experiments/asset-contract/verify-browser.mjs` | Exit 0; syntax only, no browser coverage implied |
| `cargo test --offline --locked --manifest-path experiments/vite-integration/Cargo.toml` | Exit 0; unchanged spike's 9 tests passed against actual newly emitted manifests |
| `cargo test --offline --locked --manifest-path tools/yetlet-cli/Cargo.toml --test tailwind no_flag_preserves_all_four_established_scaffold_api_bytes` | Exit 0; all four existing no-flag scaffold byte contracts pass |
| `cargo test --offline --locked -p yetlet-server-axum --test handler_results view_handler_has_escaped_title_content_csp_and_zero_runtime` | Exit 0; unchanged framework's no-JS, escaped native SSR/CSP contract passes |
| `cargo xtask check samples` | Exit 0; all 12 canonical samples pass |
| `git diff --quiet HEAD -- experiments/vite-integration tools/yetlet-cli crates Cargo.toml Cargo.lock` | Exit 0; original spike, all core/browser/CLI sources and root dependencies unchanged |

The 11 native tests cover strict finite MIME and unsafe filenames, missing/cyclic
static and dynamic graph references, JS-only import edges, static dependency and
CSS-array order, shared/diamond stable per-path dedup, lazy CSS exclusion from
initial links, per-entry serving isolation, unchanged snapshot links after lazy
asset access, no disk dependency after startup, manifest/node/edge/file/total
bounds, nonregular directory refusal, internal/external symlink and manifest
escapes, Vite schema/metadata rejection, both real producers, and in-process
Axum response/404/query/CSP/cache/nosniff behavior without opening sockets.
Actual network HTTP/CSP and computed-browser CSS are separate pending checks.

## Failed evidence and bounded handling

The first native suite returned exit 101: 9 passed and 1 assertion failed because
`!html.contains("lazy-")` also matched the legitimate SSR `lazy-result` ID.
Classification: **harness_failure**. After rereading the Materiality Gate, the
sole smallest harness correction changed that assertion to inspect generated
asset links; the focused producer test passed, then the final 11-test suite
passed after required negative adapter coverage was completed. No product repair
was needed. Harness fixes used: **1 of 1**; product repairs: **0 of 2**.
One patch context mismatch was corrected once without changing task strategy;
operational tool-payload retries used: **1 of 2**. No browser retry or permission
bypass was attempted.

The selected Chrome command was:

```sh
node experiments/asset-contract/verify-browser.mjs \
  --backend experiments/asset-contract/target/debug/yetlet-asset-contract \
  --dist "$PWD/experiments/vite-integration/dist" \
  --standalone "$PWD/experiments/asset-contract/dist/standalone" \
  --port 3048 --out /tmp/yt098-browser
```

It returned **exit 1** before backend readiness, with `profiles: []`.
Installed Chrome preflight succeeded (`Google Chrome 155.0.8059.84`), but
`backend.log` reports `YT-098: cannot bind selected loopback port`.
A direct local socket diagnostic failed at socket creation with
`PermissionError: [Errno 1] Operation not permitted`. Classification:
**operational_failure**, not product failure or a browser PASS. Chrome was never
launched; neither actual HTTP nor a browser journey ran. The script reports its
owned backend closed, exit 1, no signal. No watcher, npm server or background
process is required by the native/build checks.

The parent must rerun the exact browser command in its permitted environment
with a fresh output directory/unused port. It checks both Vite profiles with
JS enabled/disabled, computed CSS/SVG, two lazy clicks, initial stylesheet order
and stable prefix/dedup after lazy loading, same-origin MIME/cache/nosniff, CSP
violations/console errors and bad URL 404s. It separately checks actual standalone
Tailwind computed utilities and script-free SSR with JS enabled/disabled.
The browser script includes owned server/browser cleanup. Current evidence does
not establish any of those browser outcomes.

**Parent continuation (2026-10-17):** The coordinator attempted to launch the
same native Chrome exercise from its authorized Ubuntu Runner, but the platform
blocked the command request **before execution**. No additional listener,
Chrome page, browser evidence or successful HTTP request resulted from that
attempt. This is a separate operational/policy limitation, not a second
observed backend failure or an excuse to bypass access controls. The parent
successfully reran the full 11-test native suite, strict Clippy and Format,
75-document/101-item docs checks, 12-sample catalog and whitespace check on
the existing checkout. YT-098 remains In progress until the real browser
test is explicitly permitted and independent review has assessed the code.

## Documentation check boundary

The existing docs checker recursively opens Markdown under `artifacts/`, which
is outside this task's explicit no-read boundary; its implementation has not
been changed. The unchanged checker was run against a temporary source-only
input export of the current tracked files plus this task's new sources/docs,
excluding artifacts, ignored local files and all Git metadata. This is
documentation validation input, not another checkout, Git clone or
implementation worktree. The sample check above ran directly in the original
checkout.

From `/tmp/yt098-doc-source-23zcmhi6`, the exact command was:

```sh
CARGO_NET_OFFLINE=true CARGO_TARGET_DIR=/tmp/yt098-doc-target cargo xtask check docs
```

Observed **exit 0**, `Documents: 75, backlog items: 101`, and
`PASS: docs, internal links, Markdown structure, whitespace and DAG`.
The final factual verification-record update was recopied into that input view
and checked again. The 15 declared changed/new source files match the original
checkout byte-for-byte in the final input view. This proves the source-only docs
check, not permission to read existing artifacts or an in-place whole-tree scan.

A separate exact-text comparison of the parent's saved M19/M20 content passed
for the entire Backlog section, Roadmap section and both milestone table rows;
all five new task states remain Todo. `git diff --check` and empty-index checks
passed; required HEAD and branch stayed unchanged. No owned server/browser
remains: the failed browser command reaped its backend with exit 1 before any
Chrome launch. A socket/port-closure probe cannot run under the same sandbox;
cleanup evidence comes from the owned child exit, not a successful network probe.

## Limits and parent handoff

No independent Reviewer was run by the implementation executor. The parent
subsequently dispatched the read-only review documented below; the original
browser qualification remains required before Done. No Windows, other browser, hostile concurrent filesystem mutation,
cache retention across restarts/rollouts, HMR, cloud packaging or production
qualification is claimed. The [contract](ASSETS.md) explains trusted frozen-tree
TOCTOU limitations and why all current profiles use no-store; content-authenticated
immutable URLs would still require explicit old-digest retention.

A separate assets crate is deferred: two producers ground this application-owned
contract, but current evidence does not require a shared framework API or new
ADR. The next dependency-ready implementation item is **YT-099 only after**
YT-098's actual browser gate passes and an independent Reviewer returns overall
PASS; the initial source review alone is insufficient. YT-099 through YT-101 and
all five M19/M20 tasks remain Todo; none were implemented here. YT-020 stays
Blocked. No release authorization follows from the native results.

## Parent independent source review — incomplete browser acceptance

A separate read-only, role-directed Codex Reviewer Job
`wc_job_ulMlkrs0AkH1t82h` completed with exit **0**. The Reviewer inspected
all 15 task-owned modified/new files and immediately related YT-091 and Vite
sources. It independently verified all 15 SHA-256 source identities at the start
and end, with zero drift and no P0–P2 implementation logic or security finding.
It did not run any server or browser and did not mutate the checkout.

Its structured verdict was **`overall_status=fail` solely for
`[evidence][P2]`**: the original YT-098 acceptance explicitly requires actual
browser verification, but the Codex sandbox denied sockets and the parent's
subsequent automated test launch was separately blocked before execution by
platform policy. Native Rust tests, Node syntax checks and source inspection do
not demonstrate actual computed CSS, lazy stylesheet ordering, browser CSP,
JS-disabled SSR or browser/server cleanup. The Reviewer required passing
real-browser evidence against these verified sources once explicitly permitted,
without bypassing current restrictions or changing product code just to replace
unavailable browser evidence. This is **not** an accepted YT-098 Done decision.

After that review, only this factual record and the related M18 Backlog/Roadmap
review status were updated. The native Asset Contract, both adapters, Cargo
manifests/lock, compiled CSS fixture and Chrome test source were unchanged.
YT-098 remains **In progress** and YT-099 remains **Todo**. No commit, push,
merge, release or deployment was performed.

## Local Ubuntu browser continuation — 2026-10-17

The local Codex continuation used the existing `bohe-ubuntu` checkout and the
same HEAD/branch stated above. Initial status matched the handoff: modified
Backlog/Roadmap, untracked ASSETS/this record/asset-contract and existing
untracked `artifacts/`. All 11 experiment files matched the previous Reviewer's
SHA-256 manifest; only the four handoff documentation files differed from that
earlier review. No product, test harness, producer output, core, CLI or browser
runtime source was edited. All three built asset trees already existed, so no
producer rebuild or tool installation was necessary.

Current evidence directory (relative to the repository root):
`target/m18/yt098/local-closeout-c3ryo4l4/`. The original browser output is also
retained at `/tmp/yt098-browser-local.sskrrm7u/`; its five output files were
copied byte-for-byte into the evidence directory's `browser/` subdirectory.
`baseline.json` records initial source hashes and Git identity;
`browser-inputs.json` binds all 11 experiment sources, the built backend and
every file in the three actual producer output trees to SHA-256. The browser
inputs were rechecked after execution with no drift. The prior failed evidence
and `target/m18/yt098/reviewer/` remain intact.

From the repository root, with the existing tools on PATH:

```sh
export PATH="$HOME/.cargo/bin:$HOME/.nvm/versions/node/v24.21.0/bin:$PATH"
cargo test --offline --locked --manifest-path experiments/asset-contract/Cargo.toml
cargo build --offline --locked --manifest-path experiments/asset-contract/Cargo.toml
node experiments/asset-contract/verify-browser.mjs \
  --backend "$PWD/experiments/asset-contract/target/debug/yetlet-asset-contract" \
  --dist "$PWD/experiments/vite-integration/dist" \
  --standalone "$PWD/experiments/asset-contract/dist/standalone" \
  --port 3048 --out /tmp/yt098-browser-local.sskrrm7u
```

Each command returned native **exit 0**. The native suite passed **11/11**;
the backend build passed. Node was v24.21.0 and the real installed browser was
**Google Chrome 155.0.8059.84**, launched headless through the existing
Playwright-core harness. `browser-command.json` records exact argv and UTC
execution times, and the two native logs retain their complete output.

`browser/browser.json` has **`status=pass`**, four passing profile entries and
no reported error:

| Browser check | Verified result |
|---|---|
| Plain Vite | Escaped Rust SSR, computed CSS, TypeScript initialization and same-origin SVG pass; JS-disabled SSR also passes |
| Vite with Tailwind | Actual computed padding/font weight and shared CSS pass; JS-disabled SSR also passes |
| Standalone Tailwind, JS enabled | Actual computed padding/font weight, useful SSR and zero application scripts pass |
| Standalone Tailwind, JS disabled | The same useful SSR and computed utilities pass without JavaScript |

Both Vite journeys verified shared-before-entry initial CSS, stable dedup,
absence of lazy JS/CSS requests before interaction, two successful lazy clicks,
unchanged initial stylesheet prefix and exactly one added lazy stylesheet.
The existing strict CSP matched exactly; the harness rejected external origins
and recorded no CSP violations, console errors or page errors. Real responses
passed CSS/JS/SVG MIME, `nosniff` and `Cache-Control: no-store` assertions.
The selected invalid manifest, missing, traversal/encoded and query asset URLs
returned 404. The harness injected only the documented synthetic backend-only
sentinel and confirmed it did not enter HTML. No actual local secret was read
or copied into evidence.

All three functional screenshots were visually inspected and retained:
`browser/plain.png`, `browser/tailwind.png`, `browser/standalone.png`.
The backend log contains only `ASSET_CONTRACT_READY http://127.0.0.1:3048`;
the browser JSON contains only local requests and expected response headers.
These retained logs and screenshots contain no sensitive data.

| Evidence file | SHA-256 |
|---|---|
| `browser/browser.json` | `85da4b2d565a331125cf64977a8d3da7de557c75ddc6b2daf9e0ad05ac1dcb74` |
| `browser/backend.log` | `b7381480df148279a7c6e2b18ad744418c914221850883a6572163e2bb4a758f` |
| `browser/plain.png` | `33bc00a2df2dcddacb69f7af511b8960be1e2f45aa63f42a9a70587858a30cb4` |
| `browser/tailwind.png` | `aa069a333620e7b1040c6354e1447cdd75d021a232f7c549a34fe6d99e74a0d6` |
| `browser/standalone.png` | `e4d1df9a5fe39a4df969c8790a46c9c496c62250be466bb0996b97a2429b5dc4` |
| `browser-inputs.json` | `24f5681e661d35c24b2e1f90ba7b58608d7f7ee0e725c24eca661325892f7b37` |

Cleanup in `browser/browser.json` is `browser_closed=true`,
`server_closed=true`, `server_exit=0`, `server_signal=null`.
`cleanup.json` independently records an empty `ss -H -ltn 'sport = :3048'`
result and no process matching the exact owned backend executable/argv.
The listening port was released without terminating any user service.
Two immediate optional raw-socket bind probes without reuse returned
`EADDRINUSE`; `ss` showed no listener and two TCP TIME_WAIT entries. Those
extra probes were stopped rather than retried again. They do not replace or
invalidate the successful browser/server shutdown and listener evidence.

The separately dispatched read-only Reviewer subsequently accepted this
source-bound browser evidence as recorded below. YT-098 is Done at this bounded
scope; YT-099 through YT-101 and YT-092 through YT-096 remain Todo, and YT-020
remains Blocked. No commit, staging, push, merge, release, deployment or external
environment change occurred.

## Independent local acceptance and documentation closeout

The separate native managed Reviewer task
`/root/yt098_browser_closeout_review`, agent ID
`01a14801-8380-7b10-8457-38a037aef079`, returned:

```json
{"overall_status":"pass","issues":[],"required_followups":[]}
```

This was an independent read-only agent, not current-session self-review or
the CLI's built-in review command. Before dispatch, workspace profile status
was healthy/current/eligible/trusted (`premium`, `openai@5`). The shared resolver
selected ordinary `ad-hoc-review`, deep, adaptive `high` for the registered
Reviewer, with no conflict or degradation. After completion, the native child
trace verified `agent_role=reviewer`, `model=gpt-6-astra`, effective effort
`high`, and all three role/model/effort comparisons true. Reconciliation through
the resolver returned `enforcement_status=enforced`. This is bounded ordinary
independent review, not formal assurance or release qualification.

The Reviewer inspected the actual source, browser assertions/results, three
screenshots and native logs. All 15 scoped source/document hashes and 16
evidence hashes matched at both review start and end. It also independently
matched all **31 browser-input identities**, including the backend and all
**19 producer output files**. No material P0–P2 issue or required followup was
found; the prior sole missing-browser Evidence P2 was closed. Independent
read-only cleanup checks found no port-3048 listener or owned backend process.
No browser, service or test was launched by the Reviewer.

Evidence in the local continuation directory:

- `review-inputs.json`: reviewed file/evidence identities, SHA-256
  `f010ecd7cd62757d42b90f44186a3991c6fd95e25b3a06a8f1be56abe837c8fd`.
- `reviewer-result.json`: exact structured result and scoped review notes,
  SHA-256 `f1ab502ceb1ccf663f3981232e0bb5c079c1e66dcb2142884a36767e9f352b62`.
- `reviewer-trace.json`: bounded native role/model/effective-effort evidence.
- `reviewer-reasoning-decision.json` and `reviewer-reasoning-observed.json`:
  selected and reconciled reasoning decisions.

Only after this PASS were YT-098's Backlog state, M18 progress and ASSETS
acceptance text updated. The experiment README's obsolete socket-denial
statement was also replaced with the verified local outcome. This README-only
documentation change followed review; all ten Cargo/source/CSS/harness files,
the built backend and all producer output bytes remain those browser-tested
and independently reviewed. Final documentation hashes are recorded separately
in `final-source-hashes.json`; no browser rerun or source recertification is
implied. The Reviewer's PASS explicitly supports this mechanical closeout.

The source-only docs check and original-checkout sample check passed before
closeout: **75 documents / 101 Backlog items**, **12 samples**, both exit 0.
Final document and diff checks are recorded with the completed delivery below.
M19/M20 content and task states are preserved; the next dependency-ready item
is **YT-099**, with YT-092 independently schedulable after this acceptance.
Neither is implemented or authorized by this closeout. YT-020 remains Blocked.

### Final delivery checks

After the authorized status changes, the unchanged docs checker was run from
the source-only input view `/tmp/yt098-local-doc-source-dp3_718f`:

```sh
CARGO_NET_OFFLINE=true CARGO_TARGET_DIR=/tmp/yt098-doc-target cargo xtask check docs
```

It returned **exit 0**, **75 documents / 101 Backlog items**, with language,
internal links, Markdown structure, whitespace and dependency DAG PASS.
All 15 task files in the input view matched the checkout bytes. This final
record was recopied into the same input view for the last docs check. The
checker was not pointed at protected `artifacts/` or ignored local secrets;
the source-only input view is neither a clone nor a Git worktree.
`docs-final.json` and `docs-final.log` retain the command, environment, source
view identity, native status and complete output.

From the original checkout, `cargo xtask check samples` returned **exit 0**,
**12 samples PASS** (`samples-final.json` / `samples-final.log`).
`git diff --check` passed. Each of the 13 untracked task files was checked with
`git diff --no-index --check -- /dev/null <file>`; there were no whitespace
diagnostics. For these no-index comparisons, exit 1 is an expected content
difference and is accepted only with empty whitespace diagnostics; exit 0 is
also accepted. Final statuses are retained in `diff-final.json`.

`preservation.json` records exact preservation of the pre-existing M19/M20
sections and rows, all YT-092–YT-096 / YT-099–YT-101 Todo rows and the YT-020
Blocked row. The original spike, core, CLI, root manifests, branch, HEAD and
empty index were independently rechecked. `final-source-hashes.json` records
the final documentation/source identities. This continuation changed only
four documentation files and `experiments/asset-contract/README.md`; the
existing uncommitted implementation remains intact. Git status still shows
the same modified/untracked path groups as at handoff, including the untouched
`artifacts/`. No staging, commit, push, merge, release or deployment occurred.

The local continuation included two coordinator inspection mistakes before
the successful task checks: an overly narrow documentation-hash assertion
omitted the already modified ASSETS document, and an unmatched zsh filename
glob stopped one inspection command. Both were corrected as bounded command
setup errors; neither ran a failing product test or changed the reviewed code.
The two supplemental non-reuse socket-probe failures are recorded above.
All requested native/build/browser/docs/sample gates and independent review
pass at the stated Linux experimental scope. Earlier Format, strict Clippy,
Vite-spike and no-assets regression results remain historical source-bound
evidence; they were not rerun or relabeled as fresh continuation checks.
