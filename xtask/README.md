# Yetlet repository xtask foundation

YT-083 provides repository tooling only. This unpublished package has its own
workspace, lockfile and `xtask/target/`; no framework crate depends on it. Rust
1.99 or newer is required, with the repository selecting installed 1.99.0.
No Python interpreter is needed to start this tool.

From the repository root:

```sh
cargo xtask
cargo xtask --help
cargo xtask doctor
cargo xtask doctor --capability foundation --json
```

The alias is **root-only**. From another directory, supply the actual absolute
checkout path (quote paths containing spaces):

```sh
YETLET_REPO="$HOME/repos/yetlet" # Adjust to your local checkout
cargo run --offline --locked --manifest-path "$YETLET_REPO/xtask/Cargo.toml" -- doctor --json
```

For an offline bootstrap from the root, use:

```sh
cargo run --offline --locked --manifest-path xtask/Cargo.toml -- doctor --json
cargo test --offline --locked --manifest-path xtask/Cargo.toml
cargo fmt --manifest-path xtask/Cargo.toml --check
cargo clippy --offline --locked --manifest-path xtask/Cargo.toml --all-targets -- -D warnings
```

Cargo compiles the isolated tool before running it and needs pre-cached locked
dependencies for offline operation. Doctor itself only reads policy/manifests and
locks, and runs installed `rustc -vV` and `cargo --version` with bounded owned
processes. It disables rustup automatic installation for these probes. It checks
Rust/Cargo identity and minimum 1.99, and the root's selected 1.99.0 policy. It
never installs tools, changes the host, runs tests or opens browsers.
Only the `foundation` doctor capability exists. Unknown commands, capabilities,
repeated options and unsupported flags return usage errors. Builder/check
commands are listed below; browser suites, qualification, release and a public
Yetlet CLI remain unimplemented.

Human summaries and schema-version-1 doctor JSON go to stdout; failures also emit
a short stderr diagnostic. JSON reports `status`, `exit_code` and per-check
`origin`, `native_exit_code`, `signal`, `process_exit_code`, `cleanup_ok` and
`error_kind`. Validated tool/version tokens are reported; raw tool errors, argv,
environment and private paths are not printed. Exit meanings:

| Exit | Meaning |
|---|---|
| 0 | Full success including cleanup |
| 1 | Required dependency/version/policy unavailable |
| 2 | Usage error |
| 124 | Tool-owned timeout |
| 125 | Spawn, pipe, cancellation setup or cleanup infrastructure failure |
| 130 | Cancellation observed by the runner |

The internal runner preserves exact child status, including reserved-code
collisions by recording `origin=child`, and Unix signal identity (`128 + signal`).
A Windows DWORD is retained as `u32` independently of the portable exit mapping:
0 through 255 map directly; larger codes map to 1. Cargo's wrapper status is not
the native child-status record. Doctor maps a completed incompatible version
probe to capability exit 1 while retaining its native child status.

The runner accepts literal native executable/argv/environment values and an
explicit native cwd. It invokes no shell. Prefer absolute executable paths when
cwd also changes; relative executable resolution follows native `Command`
semantics. Repository location derives from the compiled manifest location;
user-relative paths remain relative to the invocation cwd.

Each stream retains only a caller-selected prefix, counts discarded bytes and
continues draining fairly. Reads are nonblocking on Unix and availability-checked
on Windows. No output-reader threads or unbounded pipe joins exist. Captures are
bytes and never automatically logged; later suite callers must apply their
existing redaction policy before persisting output. Doctor caps each stream at
16 KiB and each version probe at 10 seconds.

Unix children start in a new process group. The leader remains unreaped until
the owned group is killed, preventing signals through a reused leader PID.
Linux uses process-local subreaper adoption and reaps only the owned group;
other Unix systems depend on their orphan reaper and fail if the group persists
past cleanup. No signal is sent after the leader is reaped. Windows children
start suspended, enter a kill-on-close Job Object, then resume through their
retained process handle using `NtResumeProcess`. Failure to assign or resume
kills/waits the suspended child and fails. Nested Job restrictions are reported
as infrastructure failures without host policy changes.

On child completion, failure, timeout or cancellation, cleanup kills the owned
scope, waits for the direct child, verifies group/job termination and closes
output pipes. Cleanup has a three-second monotonic budget; RAII provides another
bounded kill/wait fallback after an early return or panic. Cleanup or pipe-EOF
failure prevents success. CLI SIGINT/SIGTERM on Unix and Ctrl-C/Ctrl-Break on
Windows only set an atomic cancellation flag; cleanup runs in normal code.
Library callers can provide their own atomic cancellation token.

Ownership covers normal descendants that remain in the Unix group or Windows
Job. Deliberately daemonizing Unix children that escape their group are outside
this foundation's supported workloads. Unix SIGKILL cannot run RAII; Windows
Job handle closure also covers abrupt owner exit. Tests cover a child leader
exiting while its descendants retain output pipes, rather than claiming Unix
cleanup after an uncatchable supervisor kill.

Linux and exact-source **Windows MSVC 1.99** checks are recorded in
[YT-083 verification](../docs/YT-083-VERIFICATION.md): all selected process tests
and doctor passed without global changes. A separate Windows GNU attempt failed
in its installed import-library tool before tests and is not labeled supported.
The final corrected foundation passed independent review (zero issues/followups).
YT-084 implements the build/check commands below. Browser/qualification commands
remain for later items; the foundation is not Python-removal proof.

## YT-084 builders and checks

The following commands are implemented in Rust and invoke no Python or Node:

```sh
cargo xtask build browser
cargo xtask build keyed-bindings --release
cargo xtask build multi-island-dashboard --release
cargo xtask build render-bench --release
cargo xtask build scene-island
cargo xtask build form-css-proof --check
cargo xtask build board-assets --check
cargo xtask check docs
cargo xtask check samples
```

The first four builders default to debug and accept only `--release`. Scene is
always an offline release build with its own nested manifest/lock, followed by
the separately built root Counter. Todo/Search and Counter/Filter likewise build
individually, preserving feature selection. Installed wasm-bindgen must report
exactly the applicable lock version. Discovery preserves Cargo's proxy filename;
processes use the YT-083 owned runner with literal native argv and cwd, inherited
environment, `RUSTUP_AUTO_INSTALL=0` and no global Cargo flag changes. The build
wait is bounded to 20 minutes; each captured stream is bounded to 16 MiB. Exact
child failures, signals, timeout, cancellation and cleanup failures stay nonzero;
failure diagnostics include native status and origin. Raw Cargo/bindgen output
stays on its original stream; command progress and PASS summaries use stdout.

CSS builds without flags reproduce the tracked assets; `--check` compares all
expected bytes and rejects extra outputs without writes. `--fetch` is a separate,
explicit acquisition operation, incompatible with `--check`. It only requests
the two fixed Bootstrap 5.3.8 HTTPS URLs, disallows redirects, bounds time/size
and verifies CSS SHA-384 and license content. It is never a fallback. No fetch
or network acquisition was run for YT-084. XML and attribute allowlists, CSS URL
registration and local loader checks reject untrusted embedded/remote assets.
The original generated Rust provenance comment is retained for byte parity.

HTML, JS, CSS, Wasm, inventories and ordered manifest JSON preserve the legacy
bytes; frozen browser target literals live in `src/build_templates.rs`. The
benchmark's legacy inert `data:,` favicon remains unchanged; it contains no JS.
Build/check negative fixtures and all four original catalog test cases are direct
Rust tests. Commands locate the checkout from the compiled manifest, independent
of caller cwd. No public Yetlet CLI or browser test is added.

See [YT-084 evidence](../docs/YT-084-VERIFICATION.md) for 128 matched snapshot files,
test commands, the early catalog CI adoption and remaining review gates. The
Python sources remain frozen reference oracles. YT-085 smoke implementations are accepted. YT-086 adds the measurement, qualification and release candidate; its separate review and acceptance limits are recorded in [verification](../docs/YT-086-VERIFICATION.md).


## YT-086 candidate commands

Run from the repository root; explicit user paths are relative to the invoking directory.

```sh
cargo xtask measure sample-dx --out target/sample-dx.json
cargo xtask measure baseline --profile release --samples 30 --browser-runs 5
cargo xtask measure render-strategies --capture-production-before target/m7a-production-before.json
cargo xtask measure render-strategies --profile release --production-before target/m7a-production-before.json
cargo xtask qualify local-operations
cargo xtask qualify multi-instance
cargo xtask qualify redis-three-app
cargo xtask qualify board --profile bounded
YT075_FIREFOX_HEADFUL_X11=1 cargo xtask measure scene-dynamic
cargo xtask release checksums create target/my-flat-bundle
cargo xtask release checksums verify target/my-flat-bundle
```

Measurements need the selected prebuilt binaries/assets and existing browser tools.
Board profiles also include `full`, `full-sync`, `sync-12m` and `staggered-12m`;
these deliberately retain long durations and are never launched by help/checks.
Qualification evidence uses fresh `target/m12/yt086/` directories; the local and
Docker commands do not overwrite old qualification reports. An explicit Docker
`--report` must be a new repository-contained path. Dynamic measurements use a
new output directory and retain incomplete lanes with a nonzero result.

`release private-rebuild` requires a clean immutable manual GitHub execution and
its source/workflow environment. CI bootstraps xtask into a separate runner-temp
target before checking source cleanliness, then uses fresh package caches/targets.
It prepares exactly nine unsigned source packages; no signing/publication occurs.
Checksums require Linux POSIX no-follow descriptors; unqualified platforms fail
closed. All 52 Python sources remain unchanged. See the verification record for
actual tests, runtime availability and the reserved independent Reviewer gate.
