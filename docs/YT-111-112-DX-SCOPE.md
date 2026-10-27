# YT-111 / YT-112 — Developer workflow follow-up registration

Date: 2026-10-22. Planning base: `71b5f084fdeedd3feade202740077fc199f950ae` (PR #11 squash).
Registration snapshot: **YT-111 Todo; YT-112 Todo.** Registration and planning only; no CLI, runtime, generated-template, dependency, configuration, browser asset or release behavior changes are authorized by this document.

Subsequent owner selection (2026-10-22): YT-111 is **Done** at the
[selected inspect contract](CLI-CONTRACT.md#read-only-project-inspection--yt-111)
and [reviewed bounded implementation](YT-111-IMPLEMENTATION.md). Independent
Reviewer follow-up PASS closes the retained initial DEL-escaping P2. This later
implementation does not change the registration review's planning-only scope.
YT-112 is now **Done at the selected Linux scope** under the separately
owner-authorized [development-loop amendment](YT-112-IMPLEMENTATION.md): all ten
focused browser/lifecycle groups and independent follow-up review pass. Original
failures and unverified Windows scope remain explicit. The registration snapshot
below remains historical.

## Existing contract (do not reimplement)

- [YT-102 DX audit](YT-102-DX-AUDIT.md), [CLI contract](CLI-CONTRACT.md), and the [current CLI guide](../tools/yetlet-cli/README.md#current-command-and-capability-inventory) already establish settings-first `new`/`build`/`run`/`watch`, owned Rust watch/restart, optional Vite/HMR, environment-only `doctor [--json]`, and source/profile/values error hints.
- [YT-102 completion latency](YT-102-COMPLETION-LATENCY.md) already improves the bounded Linux natural-exit path. No second supervisor, HMR framework, dependency installer, diagnostic registry, or new generic `dev` command is selected.
- [YT-104 browser starter](YT-104-BROWSER-STARTER.md) builds a browser-only Rust/WASM static application and previews built assets. At this baseline browser `watch` is deliberately refused; source changes require an explicit rebuild.
- [YT-110 DOM capabilities](YT-110-DOM-CAPABILITIES.md) are already accepted. Neither follow-up is a reason to reopen DOM, keyed, reactive, SSR hydration, or native-control APIs.

## YT-111 — Project-aware diagnostics

**Goal.** Give maintainers and coding agents a **read-only, secret-safe explanation of a selected project's effective CLI execution plan** instead of requiring manual reconstruction from public TOML, profile selection and stage rules. The existing `doctor --json` environment-only schema stays unchanged unless a separately reviewed compatible extension is selected.

**Design gate before implementation.** Compare a clearly scoped extension to `doctor` with a separate read-only inspection command. Select command spelling, human/optional machine output, and compatibility semantics using the existing CLI contract; do not preselect a new public flag or JSON version through registration.

**Proposed bounded acceptance:**

1. Reuse existing explicit manifest and protected public-settings selection: no implicit ancestor search, new settings authority, filesystem traversal or alternate Loader. Report selected environment profile, execution target (native/browser), Rust watch, Vite enabled/root/build/start stage selection, and reasons for skipped stages using an allowlisted project-facing representation.
2. Distinguish *settings-derived intended stages* from actual environment readiness, assets present and process execution. Never promise that doctor or a read-only project view has built assets, run tools, validated application secrets, or made a deployment ready.
3. Do not read Development secret files, expand or expose token values, dump ENV/TOML, disclose operator-specific absolute paths, start Cargo/Vite/Watchexec, modify settings, create output trees, or initiate network traffic as a side effect of inspection. Retain safe source/profile/values failure groups and fail-closed path protections.
4. Reuse focused existing CLI settings/doctor tests with native and browser synthetic settings, explicit profile/overrides, missing or invalid public settings, and secret sentinels. Verify exit/stderr contract and no-child/no-write behavior; independent Reviewer PASS before implementation Done. No new validation framework or broad platform matrix unless a changed native boundary actually needs one.

**Dependency:** YT-102 and YT-097. **Registration state:** Todo; new behavior and public CLI contract were unimplemented at registration. Current YT-111 status is recorded above.

## YT-112 — Browser-only Rust development loop

**Goal.** Offer an **explicitly selected, owned edit/rebuild/reload loop** for the existing browser-only Rust/WASM starter, without restarting a preview manually after every Rust edit.

**Design gate before implementation.** Define browser `watch` admission and Vite dev-versus-preview behavior, prerequisite/preflight ordering, build-output publish boundary, failed-generation recovery, and stop/cleanup contract. Reuse existing `watch` where feasible; do not silently change native `run` or the browser `run` build-preview contract.

**Proposed bounded acceptance:**

1. For selected browser-only projects, use an existing watcher/event provider and owned process lifecycle to rebuild Rust/WASM and run matching wasm-bindgen generation. Vite continues to own TS/JS/CSS HMR; a successful Rust/WASM generation may trigger a full page reload. No hot-swap of Rust graph/state or automatic hydration is promised.
2. Define and verify how the last-good static output remains intact or an explicit unavailable/error state is shown on compile/build failure; correction must recover without duplicate listeners or orphaned watchers. Separate build-time asset publication from partial writes, and keep strict CSP, same-origin asset resolution and secret filtering.
3. Keep source dependencies/toolchains/locks/operator installs explicit; do not auto-install Cargo, Watchexec, wasm-bindgen or npm packages. Browser-only targets still have no native backend; no persistent server circuit, mandatory state preservation, global proxy or development hot reload in production output.
4. Reuse YT-104's real browser starter and the existing CLI ownership/browser checks. Verify Rust edit -> rebuilt WASM -> reload, TS/CSS HMR, compile error -> recovery, duplicate-process avoidance, Ctrl-C cancellation and owned cleanup. Include targeted Windows native lifecycle evidence if its platform adapter changes. Independent Reviewer PASS per selected implementation slice; no unrelated CI/release qualification.

**Dependencies:** YT-102, YT-104 and existing YT-100 Vite/HMR decisions. **State:** Todo. Browser `watch` is still unsupported as of registration.

## Scope and sequencing

YT-111 is the recommended first DX implementation selection because it has a smaller no-effects boundary and improves human/agent troubleshooting. YT-112 can follow independently; it is **not blocked by YT-111**. Each needs a separate selected contract/implementation and focused verification, not automatic approval through this registration.

Neither item changes completed YT-100/102/104/107–110 scope; YT-096 remains In progress and YT-020 Blocked. No public release, production hosting guarantee, registry publication, automatic merge or push is included in this planning-only registration.

## Registration review (2026-10-22)

Independent read-only role-directed Codex Reviewer Job
`wc_job_2uhj3JW_bsalxr4T` completed with process exit 0 and
`overall_status: pass`, zero issues and zero required followups. It read the
four registration files against the selected master base and existing CLI/DX
contracts, inspected local links, headings, whitespace and the 112-item
backlog DAG. The accepted `cargo xtask check docs` result was provided by the
parent rather than rerun by the Reviewer. This was a separate Codex Job, not
native managed `agent_type` identity and not self-review.

The PASS accepts only the **planning/registration scope**. Both items remain
Todo, with implementation, platform/browser verification and future release
decisions subject to their separate gates.
