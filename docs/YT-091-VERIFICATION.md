# YT-091 — Opt-in Tailwind CSS 4 scaffolding

Date: 2026-10-17. Base: `feat/yt090-configuration`,
`52ff065500ea702c9f1e8f679a5bbc67821e61ee` (completed YT-097).
Status: **Done at bounded unpublished opt-in Tailwind CLI / Linux-native / Windows GNU-MSVC verification scope, after independent Reviewer PASS.**
YT-097 remains Done and YT-020 release remains Blocked.

## Bounded implementation

`yetlet new NAME --with-tailwind` composes with both `--cloud` and
`--with-config`. All four variants retain their app-owned native routing,
configuration and health/shutdown selection. CLI parsing/help/dispatch and a
new opt-in scaffold entry point select root `tailwind.css`, fixed
`src/tailwind.rs`, styled Rust handlers and explicit build instructions. The
native generated-subdirectory allowlist gains only `src/tailwind.rs`; Linux and
Windows no-replace/reparse/rollback logic is unchanged. Existing no-Tailwind
templates and scaffold function behavior are unchanged.

The generated manifest adds app-only SHA-256 and a test-only native Tower
utility dependency. No core, HTML/browser framework dependency or public
rendering API changes. There is no plugin system, build hook, watcher, HMR,
browser compiler, Node dependency, implicit subprocess/network action, secret
lookup, new Cargo workspace in this repository or new canonical sample.

The app validates and snapshots at most 1 MiB of UTF-8 compiled CSS, with exact
Tailwind 4.3.3 banner and nonempty body, before binding. The fixed path is
build-time app-root `dist/tailwind.css`, independent of cwd; the final file must
be regular and not a symlink. Banner validation is not artifact authentication
or a CSS sanitizer: the operator must own trusted sources and verify their
selected compiler digest. Digest bytes determine both the immutable in-memory
response and the sole `/assets/tailwind.<sha256>.css` route. Requests cannot
select files. CSS responses set MIME, nosniff, immutable cache policy and
`DEFAULT_CSP`; HTML uses no-store. The escaped framework document receives a
link containing only a fixed prefix/suffix and lowercase hex digest. Render
errors retain strict CSP. No inline styles/scripts or external fonts/assets
are introduced.

Existing Docker/Compose/Kubernetes generators retain their fixed-source
acceptance and reject every Tailwind variant, including cloud/config. No
container image, cluster action, release, external deployment, stage, commit,
push, merge, branch, worktree or global setting change is included.

## Selected upstream tool and commands

The parent supplied an already-verified standalone tool; this executor did not
download/install it. A fresh local SHA-256 and `--help` check matched:

| Official v4.3.3 asset | SHA-256 | Execution here |
|---|---|---|
| `tailwindcss-linux-x64` | `dc61b3ac6b8c9ca874c0cc4c57b2409791a64c5540404ca5f5367360babc313a` | Actual Linux build |
| `tailwindcss-windows-x64.exe` | `e0e260ce048014e9268f6237ff18f8ccf02cef521cbd0ae04e82c2cdf7aa3955` | Later actual parent-operated native GNU/MSVC builds; see source-bound qualification below |

Sources: [official tagged release](https://github.com/tailwindlabs/tailwindcss/releases/tag/v4.3.3),
[standalone CLI documentation](https://tailwindcss.com/docs/installation/tailwind-cli),
[explicit source detection and complete class names](https://tailwindcss.com/docs/detecting-classes-in-source-files).
No Tailwind configuration JS is required. Root input is exactly:

```css
@import "tailwindcss" source(none);
@source "./src";
```

The selected local tool is `/tmp/yt091-tailwindcss-linux-x64-4.3.3`. From each
generated app root, the explicit command is:

```sh
"$TAILWIND_BIN" -i ./tailwind.css -o ./dist/tailwind.css --minify
```

The generated [README](../tools/yetlet-cli/templates/tailwind/README.md) records
the exact Linux/PowerShell tool verification, build order, finite config setup,
relocation and restart requirements. `dist` output is ignored and created only
by explicit developer action, never by `new`, `build`, `dev` or requests.

## Validation results

Toolchain: `rustc 1.99.0 (b940084d7 2026-09-28)` and
`cargo 1.99.0 (5f94df478 2026-08-27)`; cached dependencies only.

| Command / boundary | Actual result |
|---|---|
| `cargo test --offline --locked --manifest-path tools/yetlet-cli/Cargo.toml --test tailwind` | Exit 0; 4 focused tests passed, real-tool test explicitly ignored by default |
| `YETLET_TEST_TAILWIND_BIN=/tmp/yt091-tailwindcss-linux-x64-4.3.3 cargo test --offline --locked --manifest-path tools/yetlet-cli/Cargo.toml --test tailwind generated_tailwind_apps_offline_contract -- --ignored --nocapture` | PASS; all four app combinations, each command below exit 0; 22 generated unit tests total |
| `cargo fmt --manifest-path tools/yetlet-cli/Cargo.toml -- --check` | Exit 0 |
| `cargo clippy --offline --locked --manifest-path tools/yetlet-cli/Cargo.toml --all-targets -- -D warnings` | Exit 0, including final fixture source |
| `cargo test --offline --locked --manifest-path tools/yetlet-cli/Cargo.toml` | Existing library/integration sections: 61 passed. Aggregate invocation failed in a newly added no-flag fixture; one fixture correction and the focused test-file rerun above passed. Not claimed as a final green aggregate run |
| `cargo xtask check docs` | PASS; docs, links, structure, 92 backlog items and DAG |
| `cargo xtask check samples` | PASS; unchanged 12-sample catalog |
| `git diff --check` | Exit 0 |

The real-tool integration explicitly digest-checks the supplied executable,
builds CSS with the exact command above and checks literal Rust utilities
`.p-8`, `.font-sans`, `.text-3xl`, `.font-bold`, `.max-w-3xl` and
`.text-slate-900` in the actual output, with no remaining `@import`.
Per app, it executes `cargo generate-lockfile --offline`,
`cargo fmt --all -- --check`, `cargo check --offline --locked --all-targets`,
`cargo test --offline --locked`,
`cargo clippy --offline --locked --all-targets -- -D warnings`, and
`cargo build --offline --locked`. Each native command returned 0; lock bytes
remained unchanged after generation. Native/cloud apps passed 4 tests each;
config/config-cloud apps passed 7 each (including the 3 existing config tests).

Socketless Axum service requests verify the actual generated `/hello/{name}`
handler's escaping and digest stylesheet link, digest route CSS bytes/headers,
unknown/unversioned URL rejection, strict CSP on successful/error rendering,
and HTML no-store. Unit tests reject missing, uncompiled, wrong-version, empty,
oversized and non-UTF-8 CSS. Actual generated binaries fail with the fixed CSS
build instruction before listening when output is absent, from a different cwd.
All twelve deployment-generator invocations (three per variant) reject with
native exit 1 and no output after a real app lock exists.

Focused scaffold tests run `new` with an unusable PATH/Cargo cache, assert the
exact root/source file lists, no output CSS or lock, and no overwrite. Linux
symlink, unsafe-parent and existing symlink destination refusals preserve user
resources. All four no-flag CLI variants match every file byte from the unchanged
original scaffold APIs. `git diff HEAD -- tools/yetlet-cli/templates
tools/yetlet-cli/Cargo.toml tools/yetlet-cli/Cargo.lock crates` is empty for
tracked files: no existing template, CLI dependency or framework source changed.

Actual changed files:

- `tools/yetlet-cli/src/lib.rs`, `src/main.rs`, `src/scaffold.rs` and
  `src/platform/fs/mod.rs` within that same CLI directory.
- `tools/yetlet-cli/templates/tailwind/README.md`, `input.css`, `handlers.rs`
  and `tailwind.rs` within that new template directory.
- `tools/yetlet-cli/tests/tailwind.rs` and `tools/yetlet-cli/README.md`.
- `docs/BACKLOG.md`, `docs/ROADMAP.md` and this verification record.

Local evidence logs retained in `/tmp` (not repository artifacts):
`yt091-focused.log`, `yt091-cli-tests.log`, `yt091-tailwind-focused-final.log`,
`yt091-cli-clippy-final.log`, `yt091-generated-apps.log`, `yt091-docs.log`,
`yt091-samples.log`, `yt091-generated-apps-initial-format-failure.log`,
`yt091-generated-apps-repair1-format-failure.log` and
`yt091-generated-diagnostic.log`. Earlier aggregate shell wrappers ended with
`tail`, so their wrapper exit is not evidence of the underlying Cargo exit;
the retained native per-app statuses and focused final command status are the
passing evidence. No final aggregate CLI PASS is inferred from those wrappers.

## Failure accounting and handoff boundary

The first generated-app check compiled CSS successfully, then failed
`cargo fmt --all -- --check` on generated Rust formatting. One product repair
corrected the lines and placement of generated tests; its rerun exposed a
remaining trailing empty line. A diagnostic compilation of the formatted temporary app also found an
incorrect `.attr` call; the existing API is `.attribute`. The second bounded
correction fixes that call and the remaining trailing empty line. Original logs are retained rather than replaced by later
passing evidence. These are product formatting/API failures, not tool failures.

The added no-flag byte-comparison fixture initially created child parents with
the host umask, which made them group-writable. The safe writer correctly
refused them. This was **harness_failure**; its one canonical fixture correction
sets only those temporary Linux parents to 0700. One focused rerun passed.
Product filesystem policy and implementation were not weakened.

Counters: `repair_budget=2`, `repair_attempts_used=2`;
`operational_retry_limit=2`, `operational_retries_used=1`;
one harness correction. The operational retry corrected one documentation patch
context mismatch; it did not alter product scope or start another validation
run. The unresolved operational signature is loopback bind EPERM below. Both
product repairs and the fixture correction have passing focused followup
evidence; no further product repair is authorized by this envelope.

A direct local `socket.bind(("127.0.0.1", 0))` probe returned
`PermissionError: [Errno 1] Operation not permitted`. This is
**operational_failure** in the runtime sandbox. No actual listener/HTTP/browser
pass is claimed. Socketless Axum service tests are separate application-layer
evidence. No unrelated harness repair or permission escalation is authorized.

Parent followups: run actual generated-app HTTP/CSP/CSS/escaping and cloud
health/signals on an authorized native host, run source-bound Windows GNU/MSVC
CLI/generated-app checks, and dispatch the mandatory independent read-only
reviewer. Browser rendering, production/rolling deployment, old digest asset
retention, non-x64 tools and release qualification are unverified. The next
unblockable action is YT-091 parent verification/review; this executor does not
activate another implementation task or mark YT-091 Done.

Pre-existing untracked `artifacts/` and ignored sensitive local resources were
not read or changed. Temporary generated apps/compiled outputs were owned by
temporary-directory lifetimes. Every spawned compiler/Cargo/app child completed
and was reaped; the final four-app root and failed-attempt/diagnostic roots were
explicitly checked absent. No HTTP server, browser, watcher, socket or background
process was retained. Local text evidence logs and the parent-owned supplied
compiler remain available; neither is a generated app runtime resource.

## Parent source-bound native qualification — 2026-10-17

After the initial sandboxed implementation attempt (retained above), the parent
operated the native Ubuntu WebCodex Runner with network sockets permitted.
Job `wc_job_dZ7Ot5BXxbqRSxaq` ran the existing bounded generated-app test with
`YETLET_TEST_TAILWIND_BIN` pointing at the SHA-256-verified official Linux
4.3.3 executable and `YETLET_TEST_NATIVE_HTTP=1`. It finished **exit 0**.
All four actual generated SSR/cloud/config combinations independently compiled
Tailwind CSS, built their native Rust binaries and served actual HTTP. The test
verified HTTP 200 HTML with escaped hostile route text, `DEFAULT_CSP` without
`unsafe-inline`, an exact SHA-256 stylesheet link, digest-route CSS 200,
`text/css`, `nosniff`, one-year immutable caching, unrelated CSS route 404,
and no synthetic secret in HTML. Cloud cases also returned 200 for
`/health/live` and `/health/ready`, then passed real SIGTERM/SIGINT shutdown
and listener closure. The non-cloud cases were terminated as owned children;
all generated app/test directories and listener resources were removed.
The added optional Linux native socket test itself is directly exercised product
verification, not a separate generic validation harness.

The parent then exported an exact-source ZIP under ignored
`artifacts/yt091-native-validation-20261017/yt091-source.zip`,
SHA-256
`a4e899d7625d82b82a6291afcc4a5dda2467b1b5f58fbc2ebcca92afde02763c`.
It includes **534** non-sensitive application/framework/CLI files with a
per-file SHA-256 manifest, including the newly tested native integration source.
Existing `artifacts/`, local secrets, untracked sensitive paths and generated
target directories were excluded. Separate Windows GNU and MSVC Runners
verified all 534 file hashes before executing the same source. Both independently
verified the **official** Windows Tailwind 4.3.3 executable against
`e0e260ce048014e9268f6237ff18f8ccf02cef521cbd0ae04e82c2cdf7aa3955`
and ran a real standalone CSS build for all four generated applications. The
two machines used native Rust 1.99 and existing user-level GNU binutils or MSVC;
only public Cargo dependencies were fetched to their existing user caches.
All later application builds/tests were offline and locked.

| Native environment | Source-bound Job | Evidence |
|---|---|---|
| Windows GNU / Rust 1.99 | `wc_job_AKvVuSCHLWInUaKK` | PASS, 534/534 hashes; CLI all-targets; four real standalone Tailwind builds, generated-app offline lock/fmt/check/test/Clippy/build; strict CLI Clippy, YT-090 native tests; isolated temp cleanup |
| Windows MSVC / Rust 1.99 | `wc_job_XUFgC0-kVXS8JlWI` | PASS, identical exact source/hash, same four app build/test stages, strict CLI Clippy, YT-090 native tests, owned cleanup |

The native Windows generated-app tests exercised source-backed static Axum
route/CSP/digest/escaping assertions without opening Windows TCP listeners.
Thus Linux real HTTP and native Windows compile/scaffold/CSS coverage are
separate scoped guarantees; browser rendering, Windows real HTTP/console
signals, adversarial ACL racing, production image/Kubernetes, accessibility,
rollout retention of old hashed CSS URLs and formal public release remain
**Not verified**.

Finally, the parent reran a fresh **complete** Linux
`cargo test --offline --locked --manifest-path tools/yetlet-cli/Cargo.toml --all-targets --quiet`
with **exit 0** (including 52 CLI library tests and all default-on integration
tests), plus `cargo fmt`, strict Clippy, `cargo xtask check docs` (75
documents, 92 backlog work items), `cargo xtask check samples` (12 samples)
and `git diff --check`; all **PASS**. This later test supersedes the earlier
aggregate-failed fixture observation **without erasing its history**. All
app-executable/test files in this source-bound Windows snapshot are the
same as those exercised on the Linux Runner; subsequent edits in this file and
in backlog/roadmap record the results only.

At the parent qualification checkpoint YT-091 remains **In progress** awaiting
the independent read-only Reviewer. YT-020 remains **Blocked**. No YT-091
commit, push, merge, registry publication, cloud deployment, or release is
authorized by these tests.

## Final independent read-only Reviewer acceptance

After the native and cross-platform qualification, a separate role-directed
read-only Codex Reviewer Job `wc_job_T7Blu0CYQUOmaQTL` terminated **exit 0**.
Its structured result is `overall_status=pass`, `issues=[]`,
`required_followups=[]`. The Reviewer used the workspace-bound `gpt-6-astra`
role configuration and shared reasoning-policy `high` selection; this was an
independent ad-hoc review, **not** self-review, native managed `agent_type`
dispatch or formal release assurance.

The Reviewer inspected the actual implementation and adjacent unchanged
framework/CLI security contracts, all **13** changed and new task files,
and the complete changed-source SHA-256 manifest retained at
`target/m17/yt091/reviewer/review-inputs.json`. It independently verified all
13 hashes at the beginning and end (zero source drift). The Reviewer examined
native tests and retained local Linux evidence, cross-platform source-bound
qualification records, no-flag byte parity, Tailwind build/asset boundaries,
strict CSP and HTML escaping, and existing deployment-generator refusals.
It did not rerun tests or fetch remote Windows Job logs itself; the parent
recorded those completed native Job outcomes separately. There were no material
P0–P2 blockers. At review closeout, only completion-status documentation was
updated. Subsequent commit preparation removed one trailing blank line from
`templates/tailwind/handlers.rs`; the scaffold already applies `trim_end()`
to that test suffix, so emitted application bytes are unchanged. No executable
Rust logic, CSS input, test assertion or dependency changed.

**YT-091 / M17 Done** for the limited unpublished native developer-preview
scope. This does not make Tailwind a framework default, enable dynamic style
compilation or change any public release gate. Windows live TCP/console,
real browser visual/IME/accessibility, long-lived digest asset coexistence,
container/Kubernetes release and production-readiness remain unverified.
YT-020 remains **Blocked**. The subsequent owner request authorizes the local
YT-091 commit after strict staged validation. Push, Merge, Auto-merge, registry
publication and deployment remain unauthorized and have not been performed.
