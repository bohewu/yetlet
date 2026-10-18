# YT-102 — External Rust watch/restart experiment

Date: 2026-10-18 (Asia/Taipei). Status: experiment complete; independent review PASS.
Source baseline: `78f3659f2b21cb80a41f563de0e47588ce3ee34a`.

After the [research comparison](YT-102-RUST-HOT-RELOAD-RESEARCH.md), the owner
asked to try the recommended existing-tool path. This bounded experiment used
an external Watchexec around the current `yetlet run` and one disposable plain
SSR app. It adds no CLI feature, runtime dependency, settings key or watcher
implementation. Broader YT-102 remains Todo; YT-103 remains independently unblocked.

## Setup and execution

The host was Linux x86_64 with Rust/Cargo 1.99.0. The current CLI was rebuilt
with `cargo build --offline --locked --manifest-path tools/yetlet-cli/Cargo.toml`
(exit 0). Its `new rust-watch-poc --parent DIRECTORY` generated the plain app.
Only that fixture's two unpublished Yetlet dependencies received explicit
checkout paths, preserving their exact version pins. Its own lockfile was
prepared with `cargo generate-lockfile --offline`, then `cargo build --frozen`
completed with exit 0. Initial dependency compilation took 5.940 seconds with
the host's existing Cargo cache; this is not a clean-machine benchmark.

Watchexec 2.8.0 was downloaded from its [official release](https://github.com/watchexec/watchexec/releases/tag/v2.8.0)
into `artifacts/rust-watch-poc-20261018/tools/`. The Linux GNU archive SHA-256
matched the digest published by the GitHub release API:
`21839e9840ef592b4d571885ccbd692f54ae5bd4231f3caf29fcf31fa7723afe`.
No global tool installation occurred.

From the generated app directory, the effective command was:

```sh
watchexec --watch /absolute/app/src --no-discover-ignore \
  --restart --debounce 200ms --shell=none \
  --stop-signal SIGTERM --stop-timeout 5s --print-events --timings \
  -- /absolute/yetlet run --manifest-path /absolute/app/Cargo.toml
```

The process inherited `/dev/null` as stdin. `YETLET_PORT` selected an available
loopback port, and `CARGO_TARGET_DIR` selected the task's own build directory.
Cargo remained offline with Rustup automatic installation/update checks disabled;
each CLI invocation retained its existing `cargo run --frozen` policy.
The explicit source watch root excluded build outputs and experiment logs.
Ignore discovery was disabled for this deliberately nested fixture, rather than
depending on ancestor repository ignore rules. Exact argv, environment values,
process identities and port are retained in the result receipt.

## Observed results

The focused experiment script exited 0. Its final receipt reports PASS:

| Step | Observed result | Elapsed seconds |
| --- | --- | --- |
| Initial run | HTTP 200; generated escaped SSR text; Rust PID 980848 | 0.361 |
| Edit handler text | Recompiled; old Rust PID gone; PID 981184 served new text with HTTP 200 | 1.576 |
| Intentional Rust compilation error | Cargo/CLI exit 101; watcher remained alive; backend absent and HTTP unavailable | 1.885 |
| Fix source | Same watcher PID 980823 resumed execution; Rust PID 981545 served recovered text with HTTP 200 | 0.573 |
| Send SIGINT to watcher | Owned watcher/CLI/Rust processes gone; selected port closed; no cleanup fallback used | 1.034 |

These are individual local observations from the corrected run, including
polling/debounce/cleanup overhead, not latency targets or tool rankings. The
watcher stops the previous Rust server before compiling its replacement. The
compilation-error stage directly observed the resulting backend outage.

**Exit-status caveat:** after the compilation error and subsequent successful
recovery, Watchexec itself returned **101** when stopped. The script's exit 0
means its stated behavior checks passed; it does not normalize the watcher's
native result. A later integration must decide how to present this observed
session status. No Yetlet exit behavior was changed to hide it.

The first attempt reached the intentional error, then timed out because the
script expected `[Command exited with 101]` while `--timings` produced
`[Command exited with 101, lasted ...]`. Its failed receipt and log are preserved.
One local assertion correction was made, followed by the focused rerun above.
No product repair was needed. Both attempts restored source and cleaned resources.

## Evidence and scope

Evidence directory: `artifacts/rust-watch-poc-20261018/`.

- `result.json`, `watchexec.log`, and `initial.html` / `edited.html` /
  `recovered.html`: corrected run, native statuses and actual HTTP responses.
- `try-watch.py`: direct, disposable-app experiment; not a production test runner.
- `first-attempt-result.json`, `first-attempt-watchexec.log`,
  `first-attempt-try-watch.py`, and `harness-correction.json`: original failure
  and the single bounded assertion correction.
- `preparation.json`, `prepare-0.log`, `prepare-1.log`, and
  `tools/release-asset.json`: frozen/offline setup and downloaded asset digest.
- `preservation-before.json` / `preservation-after.json`: prior document hashes
  and the unchanged unrelated backlog/roadmap content.

The fixture's original Rust source was restored. No owned processes or listener
remained. Existing CLI/runtime source, dependencies, settings and unrelated
planning edits were preserved. Changes outside the artifact directory are this
record and links in the documentation index and earlier research report. No
commit or push was made for the research or experiment.

This qualifies a plain SSR Rust edit/error/recovery/stop cycle on this Linux
host with null stdin and programmatically delivered signals. It does not qualify
interactive terminal input/Ctrl-C ownership, Windows, Vite persistence, frontend
HMR, broader watch roots/editor save patterns, in-flight requests or task/state
retention. Process replacement recreates Rust memory; this experiment does not
implement in-process hot replacement or automatic browser refresh.

The result supports continuing with an existing external watcher for ordinary
Rust development. Native one-command integration remains a separate decision,
using existing settings and process ownership if selected; Subsecond remains a
separate optional experiment. No additional implementation is required for this
bounded trial.

## Independent review and validation

Independent reviewer `/root/rust_watch_poc_review` returned **PASS**, with zero
issues or required followups. It checked exact target hashes, the experiment and
its one assertion correction, native exit statuses, HTTP captures, source and
planning preservation, absent processes and closed ports. The review did not
start another experiment or expand platform coverage. Its result and verified
role/model/effort trace are retained as `review.json` and `review-trace.json` in
the evidence directory. Only review-status bookkeeping changed after review.

`cargo xtask check docs` passed for 75 documents and 103 backlog items.
`git diff --check` and `git diff --exit-code HEAD -- crates tools` both exited 0.
Detailed command results are in `validation.json`.

## Subsequent Windows verification

The owner later requested the same workflow on Windows, including interactive
terminal Ctrl-C. The separate [Windows verification record](YT-102-WINDOWS-RUST-WATCH-VERIFICATION.md)
contains that source-matched evidence. The Windows and interactive-terminal
exclusions above describe this original Linux trial, not the later follow-up.
