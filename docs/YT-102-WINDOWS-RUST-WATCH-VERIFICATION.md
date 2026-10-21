# YT-102 — Windows Rust watch/restart and keyboard Ctrl-C verification

Date: 2026-10-18 (Asia/Taipei). Status: verification complete; independent review PASS.

## Authority and source

The owner requested the [Linux external-watcher trial](YT-102-RUST-WATCH-POC.md)
on Windows through WebCodex, including interactive terminal Ctrl-C and an
independent review. This is a verification follow-up, with no native CLI watcher
implementation, new settings, runtime dependency or global installation.

Local baseline: `78f3659f2b21cb80a41f563de0e47588ce3ee34a`, branch
`feat/yt090-configuration`. The existing Windows checkout remained at
`3128d23591feee15ce976d7b83a56dcd587048d7`, branch
`maint/m10-qualification-20261010`, with only untracked `artifacts/`.
A task-owned source copy contained 394 current source/workspace files; every
returned SHA-256 matches the transferred manifest and current local bytes.
The older Windows checkout was not used as the implementation under test.

WebCodex project: `agent:ncut-bohe-c420c5de1cac4bd4:yetlet-360e2631`.
Session: `wc_sess_sfnSG64EjfbaQrvV`. Windows evidence root:
`C:\repos\yetlet\artifacts\windows-rust-watch-20261018`.
Local evidence: `artifacts/windows-rust-watch-20261018/`.

## Preparation

The host reported Windows NT 10.0.26300.0. Existing Rust/Cargo 1.99.0 Windows
GNU and its installed GNU binutils were selected through process-local PATH.
No default toolchain or host environment change was made. The source-matched CLI
build used `cargo build --offline --locked --manifest-path SOURCE/tools/yetlet-cli/Cargo.toml`.
Preparation Job `wc_job_lB6Y2GFxXG_jN8rO` completed exit 0.

Watchexec 2.8.0's official x86_64 Windows MSVC executable archive was downloaded
into the task directory. Its SHA-256 matched the GitHub release API digest:
`0c872fc18c8e801f21614c6daf2504a5826d5c705bb67f7a583d879f5410d9ea`.
The Watchexec executable distribution does not change the tested app's GNU
Rust toolchain. See the [official release](https://github.com/watchexec/watchexec/releases/tag/v2.8.0).

The rebuilt CLI generated a plain SSR app using `new rust-watch-win --parent`.
Only its two unpublished Yetlet dependencies received explicit local source
paths, retaining their exact version pins. Separate offline lock generation and
`cargo build --frozen` both completed exit 0. No Vite or Node process was used.

The watcher command, from that application directory, was:

```text
watchexec.exe --watch APP/src --no-discover-ignore --restart --debounce 200ms --shell=none --print-events --timings -- yetlet.exe run --manifest-path APP/Cargo.toml
```

Each CLI run retained its existing frozen/offline Cargo execution. Source watch
scope excluded build outputs and evidence logs. Linux's explicit SIGTERM and
five-second stop flags were omitted: the upstream manual describes Windows
termination as forceful. These observations do not qualify graceful draining.
Watchexec's distinct raw-input `--interactive` feature was not enabled.

## Results

Corrected verification Job `wc_job_1vggIqE_Nj32BoIu` completed **exit 0**. Its
single plain-SSR cycle recorded:

| Step | Native observation |
| --- | --- |
| Initial run | HTTP 200 with original escaped SSR text; Rust PID 4960 |
| Edit Rust handler | Rebuild/restart; old held process handle signaled; PID 4276 served updated text with HTTP 200 |
| Intentional compilation error | Cargo/CLI exit 101; old backend stopped, HTTP unavailable, same watcher still alive |
| Fix Rust source | Watcher PID 54828 resumed without manual restart; Rust PID 18932 served recovered text with HTTP 200 |
| Terminal keyboard Ctrl-C | Console received CTRL_C_EVENT; watcher, CLI, Cargo and backend handles became signaled; port 64166 closed before helper teardown |
| Final cleanup | Helper and its console host exited; no task-owned process/listener remained, no forced cleanup fallback; source restored and Cargo.lock unchanged |

The single handler-edit observation was 3.686 seconds and recovery was 3.349
seconds, including debounce, compilation, polling and process inspection. These
are local observations, not cross-host performance comparisons. The final step's
27.204-second recorded interval includes waiting for keyboard delivery and is
not a cancellation-latency measurement.

Watchexec's final native status was **101**, despite successful recovery, as in
the Linux trial. Verification-script exit 0 means the behavior assertions passed;
the native watcher status was not rewritten. Individual cancelled child exit
codes were not captured; signaled held handles establish their termination.

## Actual interactive Ctrl-C

A task-owned helper used `CREATE_NEW_CONSOLE`, inherited console stdin and
recorded console input mode 503, including `ENABLE_PROCESSED_INPUT`. The observed
desktop window was titled `Yetlet Rust Watch PoC 20261018`; its actual screenshot
was inspected and retained. The helper stayed alive to permit child cleanup
inspection before the terminal was closed.

WebCodex's computer key interface rejected both lowercase and uppercase letter
keys before delivery. That API path was stopped. A WebCodex native PowerShell
execution then called Windows `SendInput` with four keyboard events: Control
down, C down, C up, Control up. It first required foreground HWND **3802028** to
equal the exact HWND held by the task console; otherwise it would send nothing.
All four events were accepted. The console helper recorded event 0
(`CTRL_C_EVENT`) and the watcher logged `Keyboard signal=Interrupt` before exit.
This exercised keyboard input processing, rather than writing a pipe byte or
calling `GenerateConsoleCtrlEvent`.

Implementation and receipt: `keyboard-input.ps1`, `keyboard-ctrl-c.json`,
`console-ready.json`, `watcher-exit.json`, and `terminal-before-ctrl-c.jpg`.
The successful `SendInput` count is the delivery check; the receipt's saved
last-error field is not interpreted as an error after success.
[SendInput contract](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-sendinput),
[processed console input](https://learn.microsoft.com/en-us/windows/console/console-input-buffer).

## Preserved failure and bounded correction

First Job `wc_job_iCrlCs6qpnDngiL-` completed exit 1. Rust edit/error/recovery and
keyboard delivery succeeded, but its final assertion waited for `conhost.exe`
while deliberately retaining the helper's console. The timeout was a harness
lifetime mistake. Its result, scripts, log, screenshot, keyboard receipt and
restored fixture are retained under `windows-results/first-attempt/`.

One assertion correction distinguished the held helper and its own console host
from the watcher/application trees. The corrected cycle still required all held
watcher/CLI/Cargo/backend handles to signal and HTTP to stop **before** helper
teardown, then checked all task processes after teardown. No product source
changed, and the first failed record was not promoted to PASS.

## Evidence, preservation and boundary

Returned `windows-results.zip` SHA-256:
`828319d95482fdfc475e30682822f385373a0ddee8c52f68d0f2fefda680d706`.
Its extracted `windows-results/` contains native commands, compiler/watcher logs,
HTTP responses, process identities, held-handle checks, keyboard implementation
and receipts, screenshot, source manifests and Windows preservation receipt.
Local `ui-actions.json` records observation/control evidence and the rejected
computer-key calls. Parent copies of both Python scripts and the keyboard script
match the actual returned executed bytes.

The Windows checkout HEAD/status, host defaults, generated source and lockfile
were preserved. Existing unrelated local backlog/roadmap edits were preserved.
Repository documentation changes are this record, its documentation-index link,
and a follow-up link in the Linux trial. No commit, push or deployment occurred.

This qualifies the external-watcher plain-SSR workflow and one real Windows
console keyboard Ctrl-C path using this host's GNU Rust toolchain. It does not
qualify native CLI watch integration, Windows MSVC Rust builds, other terminal
emulators, Vite persistence, frontend HMR, in-process code replacement, state/task
retention, request draining or automatic browser refresh. Broader YT-102 remains
Todo; YT-103 remains the independently unblocked backlog item. Selecting native
watcher integration and its settings/exit-status policy remains an open decision.

## Independent review and validation

Independent reviewer `/root/windows_rust_watch_review` returned **PASS**, with
zero issues or required followups. It verified the exact targets and extracted
archive, all 394 source hashes, executed scripts, actual terminal screenshot,
keyboard receipts, HTTP captures, termination before helper teardown, final
cleanup and restoration. Its role/model/effort trace matched the dispatch.
The review ran no new experiments. Only review-status bookkeeping changed after
review; records are `review.json`, `review-trace.json` and `validation.json`.

Local `cargo xtask check docs` passed for 75 documents and 103 backlog items;
`git diff --check` and the product-source diff against HEAD both exited 0.
WebCodex closeout recorded no active Jobs, zero unresolved failures, clean
artifact integrity and four verified outputs. Its automatic validation-source
status remained `unproven`, and the dirty preserved checkout could not receive
a Git tree seal. This record relies on the explicit source-hash comparison and
native receipts above; no automatic current-HEAD certification is claimed.
