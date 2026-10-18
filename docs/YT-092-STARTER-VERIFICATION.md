# YT-092 — Issue Tracker starter verification

Date: 2026-10-18. Candidate implementation on `feat/yt090-configuration`, based on
`782d2320f815c3ff38c36561456fccf3d72f9d1e`. Implementation and independent review
completed before commit authorization. After local interactive create/read/update
passed, the owner authorized a local YT-092 commit. Push, publication and cloud
deployment remain outside this delivery. Pre-existing planning and YT-103
documentation/artifacts are preserved separately.

## Accepted first application slice

The owner selected a small Issue Tracker to learn from a real application before
expanding scaffolding: create/list/detail/status, native SSR forms and a JSON API,
SQLite selected explicitly, restart persistence, existing settings and CLI,
operational startup/health/logging/shutdown, and independent review. No framework
core, CLI grammar, generator, Vite, login/roles or DI implementation changes.

The independent nested workspace is [production-starter](../examples/production-starter/README.md).
Its own lockfile permits existing frozen CLI build/run. It explicitly selects
`yetlet-config`, request IDs, anonymous Session CSRF and bundled `rusqlite`.

## Linux native evidence

Source/app-owned tests: `cargo test --manifest-path examples/production-starter/Cargo.toml --locked --offline`:
**15 passed, 0 failed** (3 configuration, 5 store, 7 HTTP contracts).
Strict Clippy (`--all-targets -- -D warnings`) and `cargo fmt --check` pass.
The existing CLI frozen build passes without a frontend stage.

Task artifacts: `artifacts/yt092-starter-20261018/` retains commands' logs,
`http-smoke.py`, and `linux-http-final/result.json`. Actual native loopback HTTP
proves SSR form create/detail/status, escaped hostile text/CSP, Bearer API
read/update, stale-write 409, unauthorized 401, persistent issue/version after
process restart, startup failure before bind and structured correlated logs
without token/query/database path exposure. Both native app Ctrl-C exits are 0;
CLI Ctrl-C preserves SIGINT (`-2` from Python) after its child emits `stopped`.
All owned children exited and their listener closed. Test data remains under the
task artifact directory, never in source settings.

Retained development failures: a dotenv test fixture used an unquoted value with
spaces (corrected by quoting); strict Clippy rejected large Response error values
(replaced by bounded status/message tuples). The initial smoke expected CLI
cancellation to exit 0; the product correctly preserved SIGINT. The task-local
assertion was corrected without changing CLI behavior. Initial failure logs and
native lifecycle logs remain alongside the final evidence.

## Windows native evidence

WebCodex session `wc_sess_NYdCC5UwJx958pJE` completed native Windows GNU
qualification. The task-owned source snapshot under
`artifacts/yt092-starter-20261018/source` matched **48 app/framework/input files**
by SHA-256 before/after execution, including the final metadata and Windows path
fixes. `qualified-source.json` locally records those hashes. The app's locked,
offline tests passed **16/16** (4 configuration, 5 store, 7 HTTP); the extra
Windows-only test rejects drive-relative/root-only database paths that could
otherwise resolve against process cwd. Existing CLI frozen build passed.

Actual native SSR form create/detail/status, JSON read/update/conflict,
restart persistence, invalid startup, structured logging and console Ctrl-C
passed. Both direct application processes exited 0; CLI preserved its child's
normal exit 0 after Ctrl-C. Every owned child exited and its listener closed.
The final receipt is `windows-results/windows-http-accepted-result.json` locally;
raw test/build/lifecycle logs, build environment and preservation receipts are in
`artifacts/yt092-starter-20261018/windows-results/`.

The existing Windows checkout remains at
`3128d23591feee15ce976d7b83a56dcd587048d7`, index tree
`9ddecefe5ec3a22462347798b8b90eeb653c1a61`, status `?? artifacts/`. No reset,
pull, commit, source checkout or global environment/toolchain changes occurred.
Only this task's copied app ACL was made private, matching the existing CLI's
filesystem trust contract.

Retained operational history: initial GNU job `wc_job_KKuj7q-hhmW7iWJu` could
not compile SQLite because Rust's linker-only GCC lacks `cc1`; installed Visual
Studio instances also lacked C++ tools. A portable **Zig 0.15.2** C compiler was
extracted only inside task artifacts, using the [official archive and checksum](https://ziglang.org/download/index.json)
(SHA-256 `3a0ed1e8799a2f8ce2a6e6290a9ff22e6906f8227865911fb7ddedc3cc14cb0c`).
Task-local CC flags select Windows GNU and ordinary unsanitized C compilation;
Zig's default sanitizer instrumentation cannot link through the separate Rust
GNU linker. The original failed logs remain on Windows; final environment values
are retained locally. This is native Windows GNU execution with bundled SQLite,
not MSVC or sanitizer qualification, and introduces no application dependency on
Zig.

The console probe initially propagated its own ignore-Ctrl-C policy into a
restarted child. Resetting that probe flag restored genuine console Ctrl-C
handling. A later probe expected only CLI exit 130, but this platform legitimately
preserved the stopped child's exit 0. The corrected probe accepts either normal
child success or observed cancellation while requiring native app exit 0,
`shutdown_requested`/`stopped`, closed listeners and no remaining owned child.
No CLI runtime change was made for these probe defects.

## Local interactive follow-up

The preview was restarted through the existing CLI executable's `yetlet run`
command, with Development settings and the explicitly selected private secret
file. No Python or outer Cargo launcher is required. The home page and readiness
endpoint returned HTTP 200, and the owner reported create/read/update working.
The starter README now demonstrates direct CLI run/build. This manual functional
confirmation does not claim a formal visual, accessibility or cross-browser gate.

## Application boundaries and lessons

- Existing handler-first App/Page, native Axum, explicit State, settings and
  optional modules compose without new framework APIs or mandatory Controllers.
- SQLite lifecycle belongs to the app: versioned SQL at startup, parameterized
  transactions, bounded blocking work, per-operation connection closure, real
  schema/connection readiness and optimistic updates.
- The shared `[cli]` namespace works, but a strict app currently repeats its small
  typed projection. A future scaffold may copy this pattern; this is evidence
  for later YT-102 work, not authorization for another settings mechanism.
- Explicit Development secrets remain separate from public settings and CLI
  planning. Staging/Production ignore the Development file. External settings
  and relative database paths anchor to an explicit root rather than arbitrary cwd.
- Existing CLI filesystem ownership/mode checks apply to copied apps. The
  checkout's group-writable default required changing only new app input modes.
- English pages must explicitly select English metadata; framework defaults
  remain unchanged. Static form HTML and HTTP CSP are verified independently of
  any browser or visual/accessibility qualification.
- Readiness includes storage and lifecycle; liveness remains independent. Logs
  deliberately omit request targets, bodies, credentials and raw error chains.
- Anonymous loopback SSR and Bearer command API have distinct access policies.
  This starter is not public authentication, identity management or a hosting,
  HA, OIDC or security guarantee. No browser/HMR, load, container or hosted proof
  is inferred from these native tests.

## Review and remaining work

Independent reviewer `/root/yt092_review` returned **PASS**, with zero issues,
required followups or optional notes, after reviewing final source and Linux/Windows
evidence. Runtime trace verification matched registered reviewer, `gpt-6-astra`
and effective `high`; the shared policy accepted the observation as enforced.
The bounded receipt is `artifacts/yt092-starter-20261018/review.json`.

Initial review found one material board-language defect; explicit English
metadata and board/error/detail assertions closed it. A separate narrow Windows
path correction rejects drive-relative/root-only paths; its native regression
passed. No framework or CLI source changes were needed.

YT-092 is **Done at this reference-application scope**. No supported public
hosting/authentication/HA or release gate is inferred. YT-093 is the next
independently unblockable item; YT-094 remains a later dashboard/auth application.
