# YT-094 — Windows native execution slice

Date: 2026-10-19. Candidate: `4bb8f082e08f6652e8abd0d798d921b431ce2d0d`.
This follows the owner's Linux UI acceptance and authorized local commit, then
continues YT-094 at native Windows scope. It changes no application or CLI source.
YT-094 remains In progress; native HTTP checks do not qualify browser input,
rendered layout, OS IME or assistive technology.

## Source and environment

WebCodex Project: `agent:ncut-bohe-244143825ae3488e:localtasks`.
Session: `wc_sess_qdz-qnuk2S_do1Dg`. Task-owned source/build root:
`C:\repos\localtasks\yt094-windows-20261019`.
Windows 10.0.26300 x86_64, Rust/Cargo 1.99.0 MSVC, Node 24.18.0 and npm 12.2.0.
The committed archive SHA-256 is
`be3e254959c6c24e4840947b5aebd1b81e8b42c6a9be729f1a217e5425ee939b`.
All 820 files matched the archive inventory before execution and after cleanup.

The CLI was rebuilt from that exact source, copied into the task's inherited
owner-protected tools directory, and used for the final build/run/stop checks.
Its SHA-256 is
`0b48c547e8549ea9de540b0b4f37caa3b14fd9ec050679c85c031d85876baad2`.
The separately prepared Cargo executable retains SHA-256
`6f4bef66261261fcb43131be8720bab817d403a09edec7455c371974b90bdb7e`.
Only process-local toolchain/PATH/build/cache variables changed. Existing
protected-tool preparation follows the [Windows CLI guide](WINDOWS-CLI-125-VERIFICATION.md#reuse-the-prepared-runner-tools).
The fnm physical Node directory is selected; its reparse alias is not selected.
No global tools, permissions, trust stores, services or user browser were changed.

## Commands and results

Dependencies were prepared separately with locked Cargo fetches and
`npm ci --offline --ignore-scripts --no-audit --no-fund`. The CLI did not install
dependencies. The Visual Studio compiler environment and the prepared Cargo cache
were process-local. Application and repository workspaces used their own locks.

| Operation | Result |
|---|---|
| `cargo build --offline --locked --manifest-path <source>/tools/yetlet-cli/Cargo.toml` | Exit 0; source-matched CLI |
| `yetlet build --manifest-path <source>/examples/admin-dashboard/Cargo.toml` | Exit 0; selected Tailwind/Vite build then frozen Rust build |
| `npm run typecheck --prefix <app>/frontend` | Exit 0 |
| `cargo test --offline --locked --manifest-path <app>/Cargo.toml --all-targets` | 12 passed, 0 failed; the additional Linux-only filesystem test is not applicable |
| `cargo test --offline --locked --manifest-path <source>/Cargo.toml -p yetlet-lab-operations-board --all-targets` | 15 library tests passed, 0 failed; the binary target contains 0 tests |
| `yetlet run --manifest-path <app>/Cargo.toml` | Actual native listener, synthetic login/session and selected Vite CSS served |
| Native HTTP workflow | Chinese UTF-8 title/description create, edit, detail/history and escaped script-shaped text passed; CSS MIME, CSP and nosniff present; ordinary HTML contains no script |
| Restart | Record and history persisted; previous in-memory session rejected with 401; login codes replaced |
| Two native console `CTRL_C_EVENT` stops | Exit 0 both times; app drain marker, owned process-handle completion and port release verified |
| `APP_CLI__VITE__AUTO_BUILD=false yetlet build ...` | Exit 0; explicitly skipped frontend build |

Console cancellation reused the existing Windows HTTPS check's native console
attachment/control-event pattern. It sends Ctrl-C to the owned console; this is
not a physical-keyboard claim. Runtime databases were disposable and separate
from the owner's Linux demonstration. Private startup codes were redacted before
retaining stdout and removed with their private logs and SQLite fixtures.

## Retained failures and interpretation

- The first frontend spawn exited 125 while PATH selected fnm's reparse alias.
  Selecting its already-installed physical Node directory allowed the frontend
  build to proceed. No executable trust rule or global ACL was weakened.
- Frozen/offline Rust commands initially lacked the app's `rusqlite` dependency
  and the workspace's `wasm-bindgen` index data. Separate locked cache preparation
  resolved those setup failures; source and lockfiles remained unchanged.
- The initial console script failed at Windows command quoting before spawning
  the CLI. Reusing the prepared PowerShell environment corrected that seam.
- The legacy console assertion required `STATUS_CONTROL_C_EXIT`, but this app
  handles Ctrl-C, drains and returns 0. The assertion now accepts the documented
  cancellation/graceful outcomes and records the actual exit. A subsequent
  zero-time process-handle observation raced final teardown; a bounded five-second
  handle wait verified completion. Neither assertion failure demonstrated an
  application source defect; the original observations remain recorded.

The [historical JSON result](https://github.com/bohewu/yetlet/blob/096f8db1e2c5b0f0fd1e664e776503cc21a74727/artifacts/yt094-windows-20261019/result.json)
records native outcomes, source/CLI identities, cleanup and SHA-256 inventories.
Current `artifacts/` outputs are intentionally Git-ignored; the original result
remains available in repository history. Full sanitized logs remain in the named
Runner Project under `artifacts/yt094-windows-20261019/evidence/`.
The JSON is a structured capture of WebCodex output; its log hashes identify
the original Runner bytes.
Final checks found zero task processes, private login logs or runtime databases,
and all 820 source files remained unchanged. The existing Linux demo was left
running for the owner.

## Remaining qualification

Windows browser, real keyboard/OS IME, actual spoken assistive technology and
Windows Clippy were not executed. This Host advertised WebCodex browser metadata
without exposing its callable browser/gateway surface; no replacement browser
control infrastructure was added. Earlier Linux browser/IME-protocol evidence
is separate and does not fill these Windows gates. Production OIDC, TLS hosting,
shared sessions, durability/HA and authenticated frontend HMR remain outside this
native slice. The next bounded YT-094 slice should select a usable browser/native
input surface before claiming those interactions.

Independent Reviewer `/root/yt094_slice_review` PASS: zero issues and required
followups. The reviewer independently inspected the original Runner test, build,
runtime, identity and cleanup logs through WebCodex; it did not rebuild source or
download/re-hash original log bytes. Acceptance is limited to this Windows native
slice. No follow-up commit or push is implied.
