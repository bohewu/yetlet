# Windows CLI exit 125 diagnosis and native verification

Date: 2026-10-18. Task: YT-093 / CLI compatibility followup. Source checkout:
`c726af4a4f2f3cf3cf6225de0c793541b564249d` plus the existing signing work and
the bounded executable diagnostic/regression below. No commit or push.

## Root cause and correction

The historical signing-sample build failures were executable-trust rejections,
before child creation or Job assignment. The Runner's default rustup proxy
`$USERPROFILE\.cargo\bin\cargo.exe` has three inherited allow ACEs granting
unrelated identities `0x1301bf`, including file-write rights. Its physical Rust
1.99 MSVC Cargo also has unrelated writable grants. Existing
`platform/fs/windows.rs::protected` rejects these permissions as a policy
`io::ErrorKind::Other`; there is no corresponding native Win32 error code.

The same updated CLI, original signing sample, source, toolchain and environment
returned 125 with the default PATH. Changing only process-local PATH to select a
byte-identical Cargo copy under the task's owner-protected directory made the
build exit 0. Both copies have SHA-256
`6f4bef66261261fcb43131be8720bab817d403a09edec7455c371974b90bdb7e`.
The [trust evidence](../artifacts/windows-cli-125-20261018/cargo-trust.json)
retains the exact owners, DACLs and hashes. This establishes an environment
permission issue, not an incompatible nested Job or signing regression.

The product change adds fixed `CLI_PROCESS stage=executable` guidance, error kind
and optional numeric native error evidence. It prints no selected executable
path, argument, environment value or arbitrary error text. Exit 125 and all
executable, cwd, stdio, suspended creation, private Job and cleanup checks remain
intact. One integration regression deliberately grants a disposable Cargo copy
Everyone write access, asserts rejection and no build output, then proves the
protected copy builds the same fixture successfully.

The default installation remains unsafe and is still refused. The working
remediation is the explicitly prepared process-local tool directory below;
global ACLs and PATH were not changed. Original current/baseline failure logs in
`artifacts/yt093-signing-20261018/windows-native-logs/` and their native report
remain unchanged.

## Native evidence

WebCodex project: `agent:ncut-bohe-244143825ae3488e:localtasks`; compatible Session:
`wc_sess_NjdlXFEnIO3u4QAE`. Owned root:
`C:\repos\localtasks\windows-cli-125-20261018`.
All 196 selected public source, lock, template and test files matched the native
verification checkpoint's [SHA-256 inventory](../artifacts/windows-cli-125-20261018/source-hashes.json).
Only a documentation link in the CLI README was synchronized after tests; all
compiled inputs remained unchanged.

| Check | Native exit / result |
|---|---|
| Verified OpenSSL 3.5.9 task-local static MSVC build | 0; `no-shared no-asm no-tests`, matching prior test prerequisite |
| Locked CLI and original signing-sample dependency fetch | 0 / 0 |
| `cargo test --locked --target x86_64-pc-windows-msvc --manifest-path <source>/tools/yetlet-cli/Cargo.toml --test windows_build -- --test-threads=1 --nocapture` | 0; 6 passed, 0 failed, 2 explicitly ignored Vite/watch cases |
| Updated `yetlet.exe build --manifest-path <source>/examples/signing/Cargo.toml`, default Cargo PATH | 125; `stage=executable`, `kind=Other`, `native_os_error=None` |
| Same CLI/sample build, protected Cargo PATH only | 0 |
| `yetlet.exe run --manifest-path <source>/examples/signing/Cargo.toml`, protected Cargo PATH | 0; all six signing demonstration checks passed |
| Existing finite console run / frozen-lock and offline cases | Native application 23 propagated; Cargo lock/cache failures 101 and preflight/headless refusals retained |
| Existing real console Ctrl-C and Ctrl-Break cases | Both CLI statuses `0xc000013a` (3221225786); resistant owned descendants exited, unrelated sentinel survived |
| Final source and process cleanup | 0; 196 source files unchanged, zero task processes, global Cargo ACL unchanged |
| Local Linux `OPENSSL_DIR=/tmp/yt093-native-tools/openssl-install OPENSSL_STATIC=1 cargo test --offline --locked --manifest-path tools/yetlet-cli/Cargo.toml --lib` | 0; 80 passed |

The cancellation test allocates an owned `CREATE_NEW_CONSOLE`, generates actual
`CTRL_C_EVENT` / `CTRL_BREAK_EVENT` through `GenerateConsoleCtrlEvent`, waits for
the CLI, and checks three pinned descendant handles and their real forwarded
Break markers before outer cleanup. It also checks the unrelated process group
and frozen lock. This is native console event evidence; no `\u0003` stdin byte
is used. The original finite signing-sample run separately used
`CREATE_NEW_CONSOLE`, file stdout/stderr and NUL stdin. The product opened
`CONIN$` and required processed input successfully. The disposable password
existed only in the child environment; generated private identities stayed in
memory and no key files were written.

The [native test log](../artifacts/windows-cli-125-20261018/windows-build.log),
[default failure](../artifacts/windows-cli-125-20261018/sample-default-build.log),
[protected build](../artifacts/windows-cli-125-20261018/sample-protected-build.log),
[sample run](../artifacts/windows-cli-125-20261018/sample-console-run.json) and
[run output](../artifacts/windows-cli-125-20261018/sample-console-run.stdout.log)
preserve commands/results. Native Jobs: prerequisite `wc_job_SLCMn_hQnWFYK7Pd`,
fetch `wc_job_PX7BnY2V9JcAjG54`, test/build `wc_job_Q6_Q3ec208i7RFYi`, cleanup
`wc_job_jfApgmzywStgEl8P`; all observed terminal exit 0. The finite sample run
completed synchronously with native exit 0.

## Reuse the prepared Runner tools

Run this in a normal attached, processed-input Windows console. These assignments
affect this process and its children only. The retained tool directory was created
inside an explicitly owner-protected task root before binaries were copied;
an arbitrary new directory or an ordinary copy under unsafe inherited ACLs does
not establish the same trust boundary.

```powershell
$r = 'C:\repos\localtasks\windows-cli-125-20261018'
if ((Get-FileHash "$r\tools\cargo.exe").Hash -ne '6f4bef66261261fcb43131be8720bab817d403a09edec7455c371974b90bdb7e') { throw 'Cargo identity changed' }
if ((Get-FileHash "$r\tools\yetlet.exe").Hash -ne '1cccc6815f63ce8e3318aefa3ce7dbb550b1015694a2725f5b32e326cf0034a9') { throw 'CLI identity changed' }
$vc = & cmd /c 'call "C:\Program Files\Microsoft Visual Studio\2022\Professional\VC\Auxiliary\Build\vcvars64.bat" >nul && set'
foreach ($line in $vc) {
    if ($line -match '^([^=]+)=(.*)$') {
        [Environment]::SetEnvironmentVariable($Matches[1], $Matches[2], 'Process')
    }
}
$env:PATH = "$r\tools;$env:PATH"
$env:RUSTUP_TOOLCHAIN = '1.99.0-x86_64-pc-windows-msvc'
$env:CARGO_HOME = "$r\cargo-home"
$env:CARGO_TARGET_DIR = "$r\target"
$env:OPENSSL_DIR = "$r\openssl-install"
$env:OPENSSL_LIB_DIR = "$r\openssl-install\lib"
$env:OPENSSL_INCLUDE_DIR = "$r\openssl-install\include"
$env:OPENSSL_STATIC = '1'
$env:OPENSSL_NO_VENDOR = '1'
$env:APP_ENVIRONMENT = 'Development'
& "$r\tools\yetlet.exe" build --manifest-path "$r\source\examples\signing\Cargo.toml"
if ($LASTEXITCODE) { throw 'Sample build failed' }
$env:YETLET_SIGNING_PASSWORD = [Guid]::NewGuid().ToString('N')
try {
    & "$r\tools\yetlet.exe" run --manifest-path "$r\source\examples\signing\Cargo.toml"
    if ($LASTEXITCODE) { throw 'Sample run failed' }
} finally {
    Remove-Item Env:YETLET_SIGNING_PASSWORD
}
```

Source-matched retained CLI SHA-256:
`1cccc6815f63ce8e3318aefa3ce7dbb550b1015694a2725f5b32e326cf0034a9`.
CLI runtime settings remain restart-to-load.

## Cleanup and limits

The [cleanup record](../artifacts/windows-cli-125-20261018/cleanup.json) confirms
zero task processes and unchanged source/global Cargo permissions. Bootstrap
OpenSSL source/archive, portable Perl/archive and disposable runtime fixtures
were removed. Retained public source, protected `tools/`, static
`openssl-install/`, isolated `cargo-home/`, build `target/` and logs are reusable
task artifacts. The signing sample still contains only its two original
key-directory `.gitignore` placeholders. No server, browser, watcher, listener,
global tool, commit or push was started by this followup.

This verifies native Windows x86-64 MSVC build/run and the selected existing
console lifecycle cases. Vite/watch ignored cases, GNU, physical keyboard input,
HTTPS, production crypto performance/upstream OpenSSL tests, deployment and
release qualification are outside this followup. No signing or HTTPS behavior
changed. Independent review is recorded separately by the parent closeout.
YT-102 remains Todo; its broader DX research is separate from this followup.

## Independent review

Final independent Reviewer: **PASS**, zero issues and zero required followups.
The reviewer checked the bounded diagnostic/regression, all 196 source hashes,
the seven permitted existing-file changes against the 799-file baseline, native
root-cause comparison, real console cancellation assertions and cleanup. The
registered reviewer role, configured model and effective effort matched the
native trace. The [review result](../artifacts/windows-cli-125-20261018/reviewer-final.json)
retains the protected-tool remediation scope: the global Cargo installation is
unchanged and remains refused.

## Commit preparation boundary

The authorized commit preparation normalized trailing blank lines in eight
signing-sample configuration/ignore files and replaced one personal directory
with `$USERPROFILE` in this report. Commit Guard also identified a public PEM
format-label literal as a private-key block despite no key material being
present. Compile-time `concat!` now expresses that same label with identical
bytes; the encrypted-only check and OpenSSL parsing remain unchanged. All eleven
local signing integration tests passed again. The native inventories and logs
retain their original identities; no Windows rerun is claimed for these
behavior-preserving source/configuration formatting changes. Independent commit
review checks this delta separately from native execution.
