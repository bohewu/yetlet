# YT-112 — Browser Rust development loop

Date: 2026-10-22. Status: **Done** at the selected Linux browser-watch scope, independently reviewed.
Owner-authorized amendment to the historical [YT-104 starter](YT-104-BROWSER-STARTER.md)
and [YT-111 inspection](YT-111-IMPLEMENTATION.md); their original evidence remains scoped.

Current continuation: the generation-module query defect is corrected and all ten
focused Linux browser/lifecycle groups pass. Independent follow-up Reviewer
`wc_job_8jX83hMPaed3M1xI` returned PASS with zero issues or required followups.
The earlier failures below remain retained history; see
[successful runtime continuation](#successful-runtime-continuation-2026-10-22).

## Selected design gate (before source edits)

- Explicit browser `yetlet watch` selects the existing owned frontend process.
  Both `cli.vite.auto_start` and `auto_build` must be true; otherwise watch
  visibly skips without children. Browser `run` remains built static preview;
  `build` remains production packaging. `cli.rust.watch` remains a native-only
  setting and must be false for browser projects. No new public command/flag.
- The browser npm dev entry selects preview normally and Vite development only
  through the CLI-owned `YETLET_BROWSER_WATCH=1` environment. The CLI clears any
  inherited value before selecting it. Existing environment filtering, protected
  frontend/dotenv checks and Linux group/Windows Job ownership remain in use;
  platform adapters do not change.
- Reuse Vite 8.3.4's watcher for app `src/`, `Cargo.toml`, `Cargo.lock` and
  `build.rs`. An initial frozen/offline optimized WASM build and exact
  wasm-bindgen 0.2.129 generation precede usable HTML. Serialize builds; debounce
  edits and coalesce events during a build into one subsequent build. Compiler
  failure waits for another edit. External path-dependency watching is not inferred.
- Generate into private, task-owned temporary directories. Publish an immutable
  JS/WASM pair only after both build stages succeed. A dev request names one
  generation; retain previous generations until stop so in-flight requests cannot
  mix pairs. Failed generations are removed; last good output stays usable.
  Before any success return an explicit 503, with correction retried on edit.
  Successful publication invalidates Vite modules and sends full reload. Rust
  state/graph/keyed identity resets; Vite owns CSS/TS/JS update semantics and the
  entry does not self-accept/remount a live Rust runtime.
- Only loopback 127.0.0.1:5173, strict port, same-origin HTTP/WebSocket and
  confined frontend sources are served. Cargo/TOML, dotenv, configuration,
  secrets, arbitrary `@fs`, raw queries and symlink escapes stay refused.
  Dev-only document responses use a random request-local nonce, Vite's
  `html.cspNonce` placeholder and post-transform replacement; CSS HMR reads the
  emitted nonce meta tag. Preserve production CSP, allow only the exact dev
  WebSocket origin, never unsafe-inline or general unsafe-eval.
- SIGINT/SIGTERM closes Vite, aborts the active compiler, waits for completion
  and removes all owned generations. The existing supervisor remains the
  process-tree cancellation authority. No detached children or OS adapter changes.
  Production `generated/` and `dist/` never contain watch generations or nonce
  placeholders. Toolchains/dependencies remain separately prepared; no installs.

Vite's [CSP guidance](https://vite.dev/guide/features.html#content-security-policy-csp)
requires request-unique nonce replacement; its
[plugin seams](https://vite.dev/guide/api-plugin.html) support request middleware
and HTML transforms. Installed 8.3.4 source was inspected: the HTML hook emits
`meta[property=csp-nonce]`, and the client applies its nonce to CSS HMR styles.

## Implementation and evidence

Implemented the selected contract in `tools/yetlet-cli/src/app.rs`,
`src/platform/process/mod.rs`, `src/browser.rs`, the finite publisher path list,
and browser `build-wasm.mjs` / `serve.mjs` templates. The existing
`examples/browser-tasks` reference uses the same scripts. Complete JS/WASM pairs
are retained in memory under immutable generation URLs; private build directories
are removed after each attempt and the owned temporary root is removed on stop.
No platform process adapter, dependency/lockfile, runtime crate or layout changes.
Existing browser projects need the refreshed scripts and npm dev entry; the CLI
does not migrate trusted app-owned package scripts automatically.

The YT-111 inspection consumer now explains browser watch as supported and
selects its WASM-watch/Vite stages only when both automatic switches are true.
Native plans, schema 1, settings protections, no-effects behavior, escaping
(including DEL) and the environment-only doctor contract are retained. Its
[original accepted evidence](YT-111-IMPLEMENTATION.md) remains historical; this
later amendment does not retroactively extend that review to YT-112.

Focused local evidence (2026-10-22):

| Check | Result |
| --- | --- |
| `cargo test --manifest-path tools/yetlet-cli/Cargo.toml --offline --locked --test commands --test browser` | PASS: 7 command tests and 1 browser scaffold test; includes inspect no-effects, secret filtering, profiles/overrides, switch combinations and DEL regression |
| `cargo test --manifest-path tools/yetlet-cli/Cargo.toml --offline --locked --lib settings::` | PASS: 8 settings tests |
| `cargo test --manifest-path tools/yetlet-cli/Cargo.toml --offline --locked --test frontend_process browser_build_has_no_native_child_preserves_failure_and_honors_skip` | PASS: build failure propagation and no-child skips, including either disabled browser-watch switch |
| `cargo test --manifest-path tools/yetlet-cli/Cargo.toml --offline --locked --test frontend_process browser_run_requires_static_output_and_respects_frontend_root` | PASS: existing static-preview preflight and custom frontend root |
| `node tools/yetlet-cli/tests/vite-dev.mjs` | PASS: 13 existing source/nonce/origin/dotenv/TLS boundary tests; does not qualify the new browser-watch runtime |
| `node --check` on both browser scripts and the existing extended headless smoke | PASS: syntax only |
| `cargo clippy --manifest-path tools/yetlet-cli/Cargo.toml --offline --locked --all-targets -- -D warnings` | PASS after one redundant test borrow was removed |
| `cargo fmt --manifest-path tools/yetlet-cli/Cargo.toml -- --check` | PASS |
| `cargo xtask check docs` | PASS: links, language/structure and 112-item DAG |
| Actual browser development loop | **FAIL** at initial page activation in the later parent run: generation module request with Vite's `?import` suffix returns 403. Initial compiler-error/503/recovery and normal stop cleanup pass; later interactive/HMR/queued-edit/active-build-stop cases were not reached |
| New native Windows browser-watch lifecycle | **Not verified**; no process adapter changes; historical YT-104 evidence is not new qualification |

Two bounded executor-local repairs were used: the initial scaffold check found the missing
finite `serve.mjs` publisher path; strict Clippy found a redundant borrow in the
changed process test. Both focused reruns passed. No operational retries were
used. The loopback capability failure did not disprove product behavior.
No Vite/compiler watcher, browser, browser profile or server was started by the
executor. One-shot test processes exited and their ordinary fixtures cleaned up.
The later parent continuation below prepared the exact tool and ran headless
Chrome; its failure evidence supersedes the earlier missing-runtime status, not
the accepted static checks. Production build/run runtime compatibility is still
unverified because the sequence stopped at the earlier page-activation gate.

## Bounded parent browser verification

After this executor is terminal, use the parent's authorized loopback/headless
environment with a private disposable prepared copy of the existing reference.
Retain its lock; point its local Cargo path dependencies at the trusted checkout,
copy the already prepared frontend dependencies and prepare exact
wasm-bindgen-cli 0.2.129 offline in task-owned tools if unavailable. No global
install or lock refresh is required. With prepared tools on PATH, run once:

```sh
node tools/yetlet-cli/tests/vite-hmr-browser.mjs --browser-watch \
  --app "$YT112_APP" --cli "$YT112_CLI" --out "$YT112_OUT"
```

`YT112_APP` is the disposable prepared reference, `YT112_CLI` the current built
CLI, and `YT112_OUT` a new evidence directory. The existing smoke's new bounded
browser branch covers initial failure/503/recovery, usable interaction, CSS HMR,
TS update and one-action listeners, Rust rebuild/reload, last-good immutable
JS/WASM after a later error, correction, nonce/CSP/console/path checks,
production build/static preview, screenshot and Ctrl-C cleanup receipts. It
restores sources, closes headless Chrome, removes its profile and checks recorded
CLI descendants absent and port 5173 closed. Keep `result.json`, `cli.log` and
`watch.png`; do not repeat a passing smoke solely for more proof. The parent extended this same bounded sequence with a disposable delayed Cargo
build-script fixture for queued edits and active-build cancellation. Those cases
were not reached after the page-activation failure and remain Not verified.

## Earlier parent runtime continuation (retained blocker)

The executor was terminal before parent validation started. Exact wasm-bindgen
0.2.129 was built offline into task-local tools, without a global installation,
dependency upgrade or lock change. The parent used the same disposable reference
and canonical headless test, retaining each failed attempt rather than replacing
its evidence. Raw records are under `/tmp/yetlet-yt111112-20261022-xd7g59vt`.

- `yt112-headless`: initial compile failure and recovery pass, but Vite's prior
  TypeScript transform normalizes import quoting before the existing replacement.
  The parent added `enforce: 'pre'` to the same template/reference plugin.
- `yt112-headless-repair`: quoting is fixed; Vite import analysis still tries to
  resolve a literal dynamic generation URL as a filesystem module. The parent
  changed that controlled same-origin dynamic import to use a URL variable.
- `yt112-headless-module`: the generation is published, but the browser requests
  `tasks.js?import`; the exact generation-route matcher rejects it with 403.
  Page activation times out. The failure screenshot and captured console also
  record a blocked blob worker from the Vite error path. This is not permission
  to relax CSP or allow arbitrary queries.

These are two parent integration corrections after the executor's two recorded
local repairs; they are not extra model-dispatched results or erased failures.
No further runtime repair is performed in this continuation. The concrete next
step is to reconcile Vite's generation-module request shape with the bounded
asset route, retaining its method/origin/path/CSP restrictions, then execute the
unreached browser/HMR/recovery/lifecycle gates and independent follow-up review.

Every attempted watch stopped with SIGINT; recorded owned descendants are absent,
port 5173 is closed, temporary generation directories and Chrome profiles were
removed, and fixture sources were restored. `failure.png` in the last result
directory shows the still-disabled loading page, not a passing UI.

## Earlier independent review disposition

Read-only Reviewer role-directed Codex Job `wc_job_QjDC_X77fgVrT8pg` completed
with process exit 0 but an overall **FAIL** verdict: one P1 generation-module
routing defect and one P2 missing-runtime-evidence gate. Exit 0 means the review
process finished, not that the product passed. The exact reviewed source hashes
were unchanged during review, and bounded runtime model/effort metadata matched
the saved role configuration. No self-review or native managed-role identity
is claimed.

The required followups are narrowly scoped: make the Vite-generated module
request work without widening asset/origin/CSP authority, then use the existing
focused browser test to prove the currently unreached interaction, TS/CSS HMR,
Rust reload, last-good assets, queued edits, active-build cancellation and
production build/run compatibility. No new validation framework or unrelated
platform matrix is required. The next continuation must retain these original
failure records and obtain independent follow-up acceptance.

YT-112 remains **In progress**, not accepted or independently PASS. YT-111's
accepted read-only scope stays Done; its new browser-watch plan is an explicit
YT-112 amendment, not proof that this runtime works. No stage, commit, push,
deployment or release action occurred.

## Successful runtime continuation (2026-10-22)

After a fresh owner instruction, the parent confirmed the earlier Jobs and all
owned processes were terminal, preserved YT-111 and the dirty checkout, and
continued only the material YT-112 routing/evidence followups. Both template and
reference `serve.mjs` now accept exactly `tasks.js` or `tasks.js?import`, plus the
query-free `tasks_bg.wasm`, under a positive numeric generation ID. Filename dots
are literal; arbitrary, repeated or valued queries remain denied. Method,
Host/Origin, source confinement and CSP checks are unchanged.

The first new run (`yt112-headless-query`) reached the active task UI, but the
nonce assertion tried to read a navigation response body after Vite had reloaded.
That is a harness-only lifecycle race, not a new product failure. The existing
assertion now reads two independent complete HTML responses and requires present,
different nonces. No product policy was relaxed and no new harness was added.
CSP-blocked Vite reconnect-worker messages arose on the failed run's server
teardown; the successful interaction checks below had no console or CSP errors.

The same focused command then passed as WebCodex Job `wc_job_E28NC7HiC2pARfVn`,
exit 0. Evidence remains under the existing task directory in
`yt112-headless-query-nonce/` (`result.json`, `cli.log`, `watch.png`), with
`yt112-browser-success-receipt.json` binding source hashes and fresh cleanup.
The prepared disposable reference, exact wasm-bindgen 0.2.129, Vite 8.3.4,
Playwright 1.64.1 and installed headless Google Chrome 156.0.8078.47 were reused;
no dependency upgrade, global install or lock change occurred.

| Actual sequence / boundary | Result |
| --- | --- |
| Initial compile error -> 503 -> source correction -> usable page | PASS; no restart storm |
| CSS update with document/state retained and nonce-bearing style | PASS; one action adds one task |
| TS entry update through Vite | PASS; full reload allowed, one-action listeners retained |
| Rust edit -> frozen WASM build -> new generation -> full reload | PASS; expected new task text and interaction |
| Exact JS `?import`, HEAD, bytes and content type | PASS; same JS bytes, empty HEAD body |
| Extra/valued/repeated queries, wrong filename, WASM `?import`, cross-Origin and POST | PASS; exact refusals retained |
| Later compile failure, immutable last-good JS/WASM, then correction | PASS; original pair remains usable and correct recovery loads |
| Multiple edits during a real Cargo build | PASS; serialized generations reach the latest edit |
| Ctrl-C during active Cargo build | PASS; owned descendants absent, port closed and generations removed |
| Production build and existing static `run` preview | PASS; no dev nonce/client/generation markers in dist, actual task action works |

The successful smoke was not rerun for redundant proof. Fresh parent inspection
confirmed all recorded child PIDs absent, port 5173 closed, no watch-generation
roots or Chrome profile, and restored disposable source files. Native/platform
adapters, YT-111 accepted source outside the explicit earlier YT-112 amendment,
and release gates remain unchanged. This is Linux browser-watch qualification;
new Windows browser-watch execution remains Not verified.

## Independent follow-up acceptance

Read-only role-directed Codex Reviewer Job `wc_job_8jX83hMPaed3M1xI` completed
with process exit 0 and overall PASS, zero issues and zero required followups.
It inspected the exact routing repair, immediate regressions, existing acceptance
contract and source-bound successful runtime/cleanup evidence. The 27 changed
source/document files were unchanged during review. The configured reviewer
model and resolver-selected effort matched bounded runtime telemetry, with no
resolver conflict or degradation. This is independent role-directed Job evidence,
not native managed-role identity or self-review.

The initial P1/P2 FAIL remains in the preceding section and original evidence.
YT-112 is Done at this selected Linux scope; YT-111's earlier acceptance stays
Done. No additional browser run was started after success. No commit, push,
release-candidate requalification or deployment followed from this acceptance.
