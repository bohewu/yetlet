# Native Windows development HTTPS verification

Date: 2026-10-18. This record covers the requested development HTTPS CLI
run/stop/build followup. No commit, push, deployment or trust-store installation.

WebCodex project: `agent:ncut-bohe-244143825ae3488e:localtasks`; Session:
`wc_sess_xyk2oRJn-xhzWjYn`. Owned root:
`C:\repos\localtasks\windows-dev-https-20261018`.
The initial source ZIP SHA-256 was
`c413cb412119844c5fa5063600a8859bf17a17fa00431a45c324ee8058b4235c`.
All 197 selected files matched the final inventory after the parent's test-only
`tools/yetlet-cli/src/docker/tests.rs` delta; compiled runtime inputs did not
change. [Final source verification](../artifacts/dev-https-20261018/windows-native-logs/source-final-verification.json).

## Prerequisites and isolation

Native tools were Rust/Cargo 1.99 MSVC, Visual Studio 2022 Professional
14.44.35207, Node 24.18.0, npm 12.2.0 and Python 3.14.3. The process-local
[environment](../artifacts/dev-https-20261018/windows-native-logs/environment.ps1)
reused the retained protected Cargo, static OpenSSL 3.5.9 and Cargo cache from
the previous CLI-125 task, and selected the physical Node installation.
Global Cargo remains unchanged and unsafe permissions remain refused by the CLI.
The [prior remediation](WINDOWS-CLI-125-VERIFICATION.md) remains applicable.

The new task root received an owner-only inheritable DACL before tools, source
or credentials were written. mkcert 1.4.4 was downloaded from its official
GitHub release and checked against the Scoop Extras manifest's pinned SHA-256
`d2660b50a9ed59eada480750561c96abc2ed4c9a38c6a24d93e30e0977631398`.
An isolated `CAROOT` created a CA and localhost/127.0.0.1/::1 certificate without
`-install`. Python used `ssl.create_default_context(cafile=...)`, retaining
certificate-chain and hostname validation. Neither Windows root store changed.
[mkcert identity](../artifacts/dev-https-20261018/windows-native-logs/mkcert-identity.json).

## Commands and native results

The source-matched CLI SHA-256 was
`94c892aa04fd8dab68cde3c78206ba0f0fa64781cd24d5cfcffc5a1df73816d1`.
`cargo build --locked --offline --manifest-path source/tools/yetlet-cli/Cargo.toml`
exited 0. Its complete 312,297-byte linker-warning/build log is retained under
the native task root; the [build result](../artifacts/dev-https-20261018/windows-native-logs/cli-build.json)
records the command and exit. The linker emitted a warning; linking succeeded.

Actual CLI creation commands exited 0:

```text
yetlet new native-rust --parent <owned-root> --settings
yetlet new native-vite --parent <owned-root> --vite
yetlet new native-http --parent <owned-root> --settings
```

Only disposable applications received explicit local framework path overrides,
separate `cargo generate-lockfile` / `cargo fetch --locked --target
x86_64-pc-windows-msvc`, and `npm ci --offline --ignore-scripts --no-audit
--no-fund`. All completed successfully. These are unpublished-source dependency
preparation steps, not implicit CLI behavior.
[Public fixture settings](../artifacts/dev-https-20261018/windows-native-logs/fixture-public-settings.json).

| Check | Native result |
|---|---|
| `yetlet build --manifest-path native-rust/Cargo.toml`, Development TLS enabled, certificate directory absent | 0 |
| Same build for `native-vite`, including frontend build, certificate directory absent | 0 |
| Default HTTP `native-http` build | 0 |
| Rust HTTPS `yetlet run`, explicit isolated CA, localhost and 127.0.0.1 | HTTP 200; chain and both hostnames accepted; wrong hostname rejected |
| Rust HTTPS real console Ctrl-C | CLI and Rust status `0xc000013a` (3221225786); subsequent inspection found no task processes or listeners |
| Malformed Rust certificate/key | Run exited 1; no listener observed during startup, no listening log, material absent from errors |
| Vite HTTPS `/` and `/@vite/client`, explicit isolated CA | 200; localhost and 127.0.0.1 certificate hostnames validated |
| Vite HMR WSS handshake with the current client token | 101 |
| Fixed internal Rust HTTP backend | `http://127.0.0.1:3000/` returned 200 |
| Vite real console Ctrl-Break | CLI `0xc000013a`; six identified CLI/Cargo/Rust/Node handles exited, ports 3000/5173 closed, unrelated sentinel alive, 1.0324 seconds |
| Default HTTP run and real Ctrl-C | 200; CLI `0xc000013a`, port closed, 2.0952 seconds |

The [build results](../artifacts/dev-https-20261018/windows-native-logs/app-build-results.json),
[malformed-material result](../artifacts/dev-https-20261018/windows-native-logs/invalid-material.json),
[Vite native result](../artifacts/dev-https-20261018/windows-native-logs/native-vite-direct.json)
and [default HTTP result](../artifacts/dev-https-20261018/windows-native-logs/native-http.json)
retain the corresponding observations. The exact direct command scripts and
public logs are in the same directory. Application lock hashes stayed unchanged
across build/run.

## Preserved harness failures

Two initial wrapper failures remain evidence, not product failures or passes.
The first helper attempted `AttachConsole` while already attached. One focused
correction called `FreeConsole` first. The rerun reached valid Rust HTTPS and a
real Ctrl-C/native exit, then its immediate zero-time wait found unidentified
PID 26224 still active. A later inspection found no task processes or listeners;
the PID's identity and exact exit delay were not established. No instantaneous
Rust cleanup claim is made. The wrapper was not repeatedly repaired.
[Initial failure](../artifacts/dev-https-20261018/windows-native-logs/native-console-initial.log),
[second failure](../artifacts/dev-https-20261018/windows-native-logs/native-console.log),
[later process inspection](../artifacts/dev-https-20261018/windows-native-logs/console-post-failure-processes.json).

The still-uncovered Vite case used a bounded direct native command with the
working console/event operations and named process identities, allowing up to
five seconds for process/listener cleanup. It passed as recorded above. No
product code or verification framework was added by this Windows followup.
The failed initial PowerShell scalar-splat creation command was a command-input
error; the explicit literal-argv creation commands above succeeded.

## Cleanup and limits

The [cleanup record](../artifacts/dev-https-20261018/windows-native-logs/cleanup.json)
confirms zero task processes/listeners, all 197 final source files unchanged,
all 196 previous task source files and the previous CLI unchanged, unchanged
global Cargo DACLs and unchanged CurrentUser/LocalMachine root stores. All three
application fixtures, the CA private key and all leaf private keys were removed.
Public source, logs, protected tools, public certificates and reusable build
cache remain. Exported public files match their native SHA-256 values in the
[file manifest](../artifacts/dev-https-20261018/windows-native-file-hashes.json).

Native Jobs: CLI build `wc_job_1k86OY4cX8RK9pDD`, dependency preparation
`wc_job_BYTLO4RxOgfZv597`, application builds `wc_job_YnB0j_xIQtH2apnZ`; each
observed terminal exit 0. Direct runtime checks and final cleanup completed
synchronously with exit 0. An initial GitHub API prerequisite request was lost;
the official binary was instead downloaded through verified TLS on the host and
imported with matching hashes. No additional OpenSSL bootstrap was performed.

This is native Windows build/run, certificate validation and scoped real console
stop evidence. It does not establish browser system-trust integration, physical
keyboard behavior, full HMR/browser qualification, production TLS, Windows GNU,
deployment or release qualification. Existing broader backlog items remain
outside this followup.
