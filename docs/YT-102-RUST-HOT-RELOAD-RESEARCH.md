# YT-102 — Rust hot-reload options

Date: 2026-10-18 (Asia/Taipei). Status: research complete; independent review PASS.
Source baseline: local commit `78f3659f2b21cb80a41f563de0e47588ce3ee34a`.

The owner requested comparison of watch/rebuild/restart and in-process code
replacement, prioritizing existing tools, with no implementation. The previously
reviewed CLI/Vite work and its reviewed YT-101 dependencies were committed first.
Unrelated M19/M20 planning remains unstaged. This research installs no tools,
changes no dependencies or runtime code, and runs no watcher, server, benchmark
or proof of concept. Upstream platform claims below are not Yetlet qualification.

## Recommendation

**Prefer existing-tool watch/rebuild/restart as the first development direction.**
It preserves normal Cargo compilation and ordinary Axum/Tower handlers. Begin
with an external Watchexec workflow and the existing two-terminal fallback. If
one-command Rust-only restart later proves valuable, reuse an existing watcher
as an event source while Yetlet retains ownership of its Rust/Vite processes.
Do not build a new filesystem watcher or replace the current process adapters.

**Keep in-process replacement an optional, application-owned research branch.**
Subsecond is the stronger candidate for an ordinary Axum experiment because a
released helper exists; hot-lib-reloader fits a deliberately small reloadable
logic boundary. Neither establishes transparent preservation of arbitrary
server state, pending requests or tasks. No mandatory dylib/ABI boundary should
be added to Yetlet Core or the handler API.

These are proposed directions, not selected dependencies or an implemented
watch mode. Current `yetlet run` still executes one Cargo/Rust invocation,
optionally supervised with Vite. Future persistent policy should use the existing
settings mechanism; this document adds or reserves no setting names or CLI flags.

## What the two approaches actually preserve

| Concern | Watch/rebuild/restart | In-process replacement |
| --- | --- | --- |
| Update unit | Replace the Rust process after source changes | Patch functions or replace an application-owned library/future |
| Ordinary Rust type/router edits | Normal recompilation and fresh initialization | Tool-specific compatibility and re-instancing rules |
| Rust RAM and tasks | Recreated; memory-only state is lost | Only deliberately retained compatible state can survive |
| Database/external sessions | Depend on the application's persistence contract | Same; code patching is not a persistence mechanism |
| Requests/connections | May be interrupted; stop-before-build includes compile downtime | PID continuity alone does not preserve requests/listeners/tasks |
| Browser state/new SSR | Browser may stay open; new SSR needs another request/navigation | Same; patching Rust is not automatic DOM replacement |
| Feedback cost | Compile/link plus process startup | Patch build/load plus application update work; no measured speed ranking |

The state and interruption entries are consequences of these mechanisms, not
runtime results from this research. A stable PID is insufficient evidence of
safe state retention or uninterrupted HTTP service.

## Existing tools

Versions were checked against primary sources on the research date. CLI and
library version numbers are distinct; no dependency was added or pinned here.

| Candidate | Documented behavior and reuse | Assessment for Yetlet |
| --- | --- | --- |
| **Watchexec CLI 2.8.0** | Restart, filtering, debounce and event-only JSON output; shell-free execution requires `--shell=none`. [Release](https://github.com/watchexec/watchexec/releases/tag/v2.8.0), [versioned manual](https://raw.githubusercontent.com/watchexec/watchexec/v2.8.0/doc/watchexec.1.md) | First external candidate. Restart stops the command before the next invocation, so wrapping `cargo run` stops the old server before compilation. Compile failure does not normally end watching. Its process ownership must not be assumed equivalent to Yetlet's native adapters. |
| **Watchexec library 8.4.4 / Notify 8.2.0** | Existing event engines; Notify offers native platform and polling backends, with separate debouncers. [Watchexec](https://docs.rs/watchexec/8.4.4/watchexec/), [Notify](https://docs.rs/notify/8.2.0/notify/) | Candidates if native CLI integration is justified. Event-only use avoids a second process owner; embedding adds API/runtime integration, while external event mode adds executable/event-stream ownership. |
| **Bacon 3.26.0** | Configured Cargo jobs, headless mode and long-running `kill_then_restart` behavior. [Release](https://github.com/Canop/bacon/releases/tag/v3.26.0), [cookbook](https://dystroy.org/bacon/cookbook/) | Useful developer-selected workflow. Its UI/job interruption model is a poorer fit for embedding beneath Yetlet's existing supervisor. Killing before restarting also includes build downtime. |
| **cargo-watch 8.5.3** | Final release; repository archived in January 2025. [Repository and retirement notice](https://github.com/watchexec/cargo-watch) | Existing users may retain it; do not choose it as a new foundation. |
| **hot-lib-reloader 0.8.2** | Separate dylib, exported symbols, reload notifications and host-owned state patterns. [Published documentation](https://docs.rs/hot-lib-reloader/0.8.2/hot_lib_reloader/) | Suitable for isolated application logic; arbitrary Axum/router replacement would require a deliberate architecture split. |
| **Subsecond / Dioxus devtools 0.7.10** | Experimental hotpatch tooling and a released Axum entrypoint helper. [Runtime](https://docs.rs/subsecond/0.7.10/subsecond/), [released helper](https://github.com/DioxusLabs/dioxus/blob/v0.7.10/packages/devtools/src/lib.rs#L104) | Preferred future in-process experiment, with explicit state/task ownership and restart fallback. Requires its builder/devtools integration; it is not a drop-in watcher around current `cargo run`. |

The Watchexec website download listing lagged its GitHub release during
inspection; the table uses the release tag. Subsecond's published alpha was not
used to infer capabilities beyond the pinned stable release.

## In-process replacement: relevant limits

**hot-lib-reloader:** exported signatures and shared type layouts must remain
compatible; exported generic functions are unsupported. Reloadable globals need
care, and the documentation lists a known `tracing` problem. It supplies reload
coordination hooks, not automatic application task draining. These constraints
make a small synchronous logic function more plausible than exporting an entire
async handler/router. [Limitations](https://docs.rs/hot-lib-reloader/0.8.2/hot_lib_reloader/)

Rust's native ABI has no stability guarantee. Unmangled symbol names alone do not
establish compatible calls or layouts. The inference for Yetlet is that futures,
callbacks, trait objects and drop code retained after a call require an explicit
lifetime boundary before the old library can be unloaded. A loader cannot
establish application quiescence merely by resolving a symbol.
[Rust ABI](https://doc.rust-lang.org/reference/items/external-blocks.html#abi),
[loader safety contract](https://docs.rs/libloading/latest/libloading/struct.Library.html)

**Subsecond:** released `serve_subsecond` explicitly supports non-Dioxus Axum
applications. Its implementation drops the current outer future after a patch
and creates another, optionally cloning supplied outer arguments. It contains
no graceful request-draining sequence. Thus it can retain a process while
rebuilding its server/router and losing state initialized inside that future.
It requires the `serve` feature and brings Dioxus devtools dependencies.
[Released lifecycle](https://github.com/DioxusLabs/dioxus/blob/v0.7.10/packages/devtools/src/lib.rs#L104),
[dependency configuration](https://github.com/DioxusLabs/dioxus/blob/v0.7.10/packages/devtools/Cargo.toml)

Dropping the outer future does not establish cancellation of detached tasks:
dropping a Tokio `JoinHandle` leaves its task running. Task shutdown and deliberate
outer-owned state remain application responsibilities.
[Tokio cancellation semantics](https://docs.rs/tokio/latest/tokio/task/struct.JoinHandle.html#cancel-safety)
Subsecond also documents type-layout/re-instancing and thread-local limitations;
its stable docs still describe tip-crate patching. Its native runtime retains
patch libraries rather than unloading them, trading unload hazards for retained
libraries during the development session. Do not infer memory bounds or safe
arbitrary layout migration from the hotpatch label.
[Stable limitations](https://docs.rs/subsecond/0.7.10/subsecond/),
[released patch loading](https://github.com/DioxusLabs/dioxus/blob/v0.7.10/packages/subsecond/subsecond/src/lib.rs#L474)

## Fit with the current CLI

Current [execution selection](../tools/yetlet-cli/src/app.rs) uses independent
Vite `auto_start` and `auto_build` settings. The
[process boundary](../tools/yetlet-cli/src/platform/process/mod.rs) supplies literal
Cargo commands, protected working directories and frontend environment
separation. Both the [Linux](../tools/yetlet-cli/src/platform/process/foreground.rs)
and [Windows](../tools/yetlet-cli/src/platform/process/windows.rs) pair loops end
and clean both trees when either child exits. Consequently:

- Wrapping all of Vite-enabled `yetlet run` restarts Vite too. With existing
  `auto_start = false`, a separately started Vite can remain independently owned.
  Neither external wrapper workflow has been qualified by this research.
- Native Rust-only restart would need a persistent development-session owner and
  a replaceable Rust process group/Job. A file-change restart must be separate
  from the existing session cancellation signal; terminal ownership, bounded
  cleanup, protected paths and frontend secret exclusion still apply.
- A recoverable compilation error would need to leave the watch session active.
  Keeping Vite running through that error changes the current fail-either/end-both
  behavior and must be an explicit future implementation contract.
- Keep `--frozen` and offline/no-auto-install policy on every Cargo invocation.
  A watcher does not supply these guarantees automatically. Bacon's separate
  Cargo discovery step also needs assessment before adoption.
  [Cargo's frozen/locked/offline semantics](https://doc.rust-lang.org/cargo/commands/cargo-run.html#manifest-options)

Stop-before-build is the smaller candidate. Keeping the last successful Rust
server alive during compilation additionally requires separate build/run and
artifact handoff, including Windows executable handling; it is not an existing
capability. Browser reload/reconnection is another decision, separate from
Rust restart. Preserve the ordinary-handler contract in [ADR 0006](adr/0006-handler-first-mvc-optional.md);
development convenience must not require a new controller, DI or plugin model.

## Platform evidence and bounded next evaluation

Notify documents Linux and Windows backends; hot-lib-reloader's CI configuration
includes Ubuntu and Windows; Subsecond's stable docs list Linux and Windows
architectures. These are upstream declarations, not checked Yetlet results.
Exact compatibility with this repository's Rust1.99 Windows GNU/MSVC paths,
builder/linker requirements and offline preparation remains untested.
[Notify platforms](https://github.com/notify-rs/notify#platforms),
[hot-lib-reloader CI](https://github.com/rksm/hot-lib-reloader-rs/blob/master/.github/workflows/ci.yml),
[Subsecond platforms](https://docs.rs/subsecond/0.7.10/subsecond/)

A later, separately selected experiment should use one application-owned Axum
sample: compare an ordinary handler edit and compile-error recovery under an
existing watcher, then compare body/router/state-layout edits using Subsecond.
Observe edit-to-response time, process/listener identity, one tracked task and
one slow request. Explicitly record which state survives and which work stops.
Use the intended Linux/Windows toolchains only for the chosen candidate. Expand
testing only if a demonstrated integration problem requires it; do not certify
the upstream tools or repeat Vite's HMR matrix.

No PoC, latency measurement or cross-platform runtime result is claimed here.
YT-102's broader DX study remains Todo; YT-103 remains an independent next
unblocked backlog item. Research completion does not authorize implementation.

## Independent review

Independent reviewer `/root/rust_hot_reload_research_review` returned **PASS**
with zero issues and no required followups. It checked the exact document
hashes, current process adapters and selected primary sources; the review was
read-only and did not repeat runtime experiments. Docs and whitespace checks
passed. Evidence is retained in
`artifacts/cli-commit-research-20261018/research-review.json` with verified
dispatch trace. Only review-status bookkeeping changed after review. The
research files remain uncommitted; the earlier implementation commit is separate.

## Subsequent authorized experiment

After this research review, the owner requested a trial of the recommended
existing-tool path. The separate [Linux watch/restart experiment](YT-102-RUST-WATCH-POC.md)
records actual results and limits. The research-only scope and absence of runtime
measurements stated above describe the earlier research, not that later trial.
