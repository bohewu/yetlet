# YT-102 — Native Windows Rust and Vite pair verification

Date: 2026-10-18 (Asia/Taipei).

## Authority and scope

The owner explicitly requested Windows Vite pair-supervisor verification through
WebCodex, followed by independent Reviewer review. This supplements the
[CLI implementation](YT-102-CLI-IMPLEMENTATION.md) and
[doctor follow-up](YT-102-DOCTOR-VERIFICATION.md).

The source of truth is `$REPO`, branch
`feat/yt090-configuration`, HEAD `a883469862c5be3d88b7fe22262e10be8b53b92a`
plus the preserved dirty working tree. Evidence is stored under
`artifacts/windows-vite-pair-20261018/`.

## Native environment and source

WebCodex Project: `agent:ncut-bohe-c420c5de1cac4bd4:yetlet-360e2631`.
Session: `wc_sess_ol95jcfM1xSqJN8i`. Existing Windows checkout:
`C:\repos\yetlet`, branch `maint/m10-qualification-20261010`, HEAD
`3128d23591feee15ce976d7b83a56dcd587048d7`. Its original status is artifacts only.
The existing checkout is preserved. A task-owned source copy uses the 393
verified current-source files from the prior doctor task, with the explicit
native pair test overlay recorded separately.

Already installed native tools: Rust/Cargo1.99.0 Windows GNU, GNU binutils,
Node24.18.0 and Vite8.3.4. Per-process PATH uses physical tool directories:
fnm's default alias is a reparse path and is not selected by the safe native
executable lookup. No tool installation or host default change is required.

## Verification results

Only `tools/yetlet-cli/tests/windows_build.rs` changes executable source in this
follow-up. The direct integration check extends its existing Windows console
fixture, protected application creator and Job ownership seam. It uses actual
Vite/npm and a real frozen Cargo run of a dependency-free Rust process tree.
The Vite config uses Node builtins and the installed absolute Vite entry; no
replacement frontend server or implicit dependency installation is involved.
The test overlay SHA-256 is
`6739a1fc3e2ce0a0bb80d231918f70e7323345777ac7988ec012e3e7fbecf71a`.

Native command, with process-local physical Node/Rust/binutils PATH and
`YETLET_TEST_VITE_JS` pointing to the installed absolute Vite entry:

```text
cargo test --offline --locked --manifest-path tools/yetlet-cli/Cargo.toml --test windows_build native_cli_vite_pair -- --ignored --exact --nocapture --test-threads=1
```

Rustup installation/update is disabled. The explicitly ignored test is selected
and executed; no scenario is skipped. Native WebCodex Job
`wc_job_mZAj2OvpP1i7Vd7i` completed exit0 in 57.028 seconds including compilation.
One integration test ran all six serialized native console cases in 13.69 seconds.

| Native case | Observed result |
|---|---|
| Ctrl-C after Vite and Rust startup | Real Vite8.3.4 serves `/@vite/client`; Rust parent and two resistant descendants run; native cancellation status `0xc000013a` retained |
| Targeted Ctrl-BREAK after startup | Same native cancellation status; both owned trees terminate within the existing bound |
| Real Vite exits23 after both ready | CLI exits23; Vite and Rust handles signaled, listener released |
| Real Rust parent exits23 after both ready | CLI exits23; Vite and resistant Rust descendants terminated |
| Port5173 already occupied | CLI exits125 before either frontend or Rust starts; existing listener retained |
| `auto_start=false` with Port5173 occupied | Frontend does not start; Rust executes and its exit23 is retained |

The four live-pair cases assert synthetic backend APP/secret selectors are
excluded from the frontend, the public VITE marker and project context reach it,
and the backend retains its synthetic marker. Every case preserves Cargo.lock.
Live-pair cases pin Vite and all three Rust process handles and assert their
termination, port release and unrelated sentinel isolation **before** the outer
test Job's safety cleanup. The outer forced reap is not used as product proof.
Final inspection finds no task target executable left running.

All 393 source hashes match the recorded tested manifest after execution.
The existing Windows checkout HEAD/status and host defaults remain unchanged.
Native receipts are in `windows-results/` under the local evidence directory;
`result.json` records literal arguments and native exit, `final-receipt.json`
contains six case receipts, and `native-pair.stdout` retains the actual output.
The returned ZIP SHA-256 is
`eef18dafb499e7a759275487467e78328fefed1cf40f11afb871b12c60096dfc`.

Local test-source formatting and Windows MSVC cross-target test compilation
passed separately. `cargo xtask check docs`, CLI formatting and `git diff --check`
also pass. The implementation record, contract, CLI guide and navigation now
link this selected native evidence while retaining the original compile-only
history. The CLI guide is the only subsequent documentation difference within
the 393-file transferred set; current runtime/test bytes still match native execution.

Independent Reviewer: **PASS**, zero issues and zero required follow-ups.
The review independently compared all 393 tested file hashes and the exact ZIP
and test overlay, then checked assertions and native cleanup receipts. Registered
role, configured model and effective effort match the child trace. Bounded review
and source-preservation receipts are review-result.json, review-trace.json and
local-final.json in the local evidence directory. Local HEAD and index are preserved.

## Boundary

This is bounded CLI process-lifecycle verification. A subsequent
[Windows browser CSS HMR check](YT-102-WINDOWS-HMR-VERIFICATION.md) records the
separate generated-app browser evidence. Wider generated asset/runtime
qualification, MSVC native execution, publication and
production readiness remain separate. YT-102 broader DX research stays Todo;
YT-020 stays Blocked. No commit, push or deployment is authorized by this task.
