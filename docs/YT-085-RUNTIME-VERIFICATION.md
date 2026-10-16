# YT-085 — Current Rust Smoke Runtime and Remaining Boundaries

**State (2026-10-15):** Source-bound Linux runtime evidence retained and later focused cleanup/receipt repairs independently accepted (YT-085 Done; Reviewer PASS). The historical suite runs below predate the focused Server/Browser/Board repairs and were not unconditionally rerun. YT-086 cutover has not started. The source is the
same original Ubuntu `master` checkout at `7cfac613f908c5c19fa581438288386809870b56`
with the uncommitted isolated Rust tooling migration. No commit or push occurred.

## Original implementation and continuation

The first generalist Job `wc_job_mgAoJ2s-DpyNG6KB` ended exit 0 with an explicitly
**partial** implementation result: ten native/eight browser/two fixture commands,
43 xtask tests, 54 sample tests, no successful runtime and eight missing Board/
scene ports. Its Codex sandbox rejected loopback socket creation before starting
any browser/server. Its exhausted four repairs/two retries and failed receipts
remain at `target/m12/yt085/implementation/` and `implementation-DaoDRt/`.

After proving that execution and its exact process ended, the parent ran the
admitted WebCodex host tools without changing host permissions or the sandbox.
All ten native suites and eight first-port browser suites passed. A distinct
remaining-code generalist Job `wc_job_Kd-pdwsoAkwFKB2X` then implemented the eight
previously absent commands and the declared Dashboard/proof omissions. It ended
exit 0 with source-ready status, 49 pure/owned-process tests, strict Clippy/fmt/
check/docs PASS and no browser/network execution. That Job's three corrections
preserve current-owner versus retired-resource observations, bounded deadlines/
reports and one reusable HTTP agent per Board load client. Both roles used the
saved `gpt-6.1-sol` model and actual effective `high` effort; they are local
role-directed Jobs, not native managed agent-type dispatch.

## Current-source execution

A source-bound rebuild/test/lint preparation completed as Job
`wc_job_LdCppqfopRdaKzaS`, exit 0; logs are in
`target/m12/yt085/parent-remaining-runtime/`. The actual scene bundle remains
`f5584f407ee4f7b2`, 288,758 Wasm bytes, with the independent counter
`81c388455a88d98a`. This is the real existing Rust/wgpu application, not a
JavaScript recreation. No application runtime source was changed.

| Actual current Rust command family | Outcome |
| --- | --- |
| Native: hello-world, rest-api, cart, middleware-lab, forms-validation, file-server, sqlite-crud, sse-notifications, auth-notes, modular-shop | **10/10 exit 0**; original HTTP/state/negative/restart/disconnect scenarios and owned cleanup |
| Todo/Search: browser-chrome and browser-firefox | **2/2 exit 0**; actual browser inputs, DOM/lazy assets, typed payload and queued/in-flight disposal |
| Keyed bindings Chrome/Firefox; Dashboard Chrome/Firefox | **4/4 exit 0**; actual browser identity/state/lifecycle and corrected original HTTP/CSP assertions |
| forms-accessibility-firefox; form-css-proof-chrome | **2/2 exit 0**; actual existing browser/form/CSP scenarios and source report fields; no new physical IME/AT claim |
| board-chrome; board-firefox | **2/2 exit 0**; real browser forms/auth/conflicts/SSE/status and original bounded diagnostic/reset/restart/data checks |
| scene-firefox; scene-lifecycle-firefox; scene-picking-firefox; scene-candidate-firefox | **4/4 exit 0**; original distinct real hardware pixel, ownership, controlled loss/recovery and semantic input scenarios |
| scene-support-firefox | **exit 0**, actual default-profile `outcome=unavailable`; HTML/counter/keyboard/detail fallback passed, **GPU acceptance=false** |
| scene-chrome | **exit 1, gpu_unavailable**; original selected host's lack of a usable Chrome WebGPU adapter is retained, cleanup passed. **No GPU drawing PASS** or auto-retry |

The new seven-journey Board/Firefox batch ran as Job
`wc_job_xEPxiXNfM29_9PVb`, terminal exit 0, retained in
`target/m12/yt085/parent-remaining-runtime/first-browser-batch/`.
The shared Browser/HTTP helper and added Dashboard/proof assertions changed
since the first partial-port run, so the parent performed one current-source
regression of those ten native/eight browser suites as Job
`wc_job_mwsTXBhSNDXsYu5T`, terminal exit 0. This was not another GPU replay:
its source hash snapshot is in
`target/m12/yt085/current-shared-helper-regression/`, with zero source drift.
All actual suite outputs use distinct `target/m12/yt085/<suite>-<nonce>/`
directories rather than overwriting M11 proof. The 26-command receipt index is
`target/m12/yt085/parent-runtime-index.json`, SHA-256
`4f96573a041e21f915d330289d3c2aa8c417fcb7bbe8649acd9d89cc807a65cf`.
It records **25 positive suites and one explicitly unavailable GPU suite**,
not 26 successful GPU tests. Native/ordinary browser `result.json` is the
wrapper status; original scenario details and teardown/exit records are in
the same suite's report/log files. Scene receipts retain their detailed
cleanup, input hashes and individual real PNG evidence.

## Required compatibility correction before this batch

The remaining-code implementation initially changed default-profile Support
to return nonzero merely because WebGPU was absent, despite successfully
checking its HTML alternative. The parent compared
`examples/webgpu-scene-island/verify_support.py:138-179,213` and repaired this
single classification: that suite's purpose is default-profile behavior,
including fallback, so its original successful
`passed_yt080_default_profile` result is retained only after all semantic
assertions pass. The current receipt separately says `gpu_acceptance=false`.
This is **not** a skipped GPU drawing test. All GPU-only drawing commands,
missing required browser/tool, assertion failure, timeout, input mutation and
cleanup failure still return nonzero. No original assertion/tolerance was
relaxed. The existing 49 tests, strict Clippy, fmt and actual default-profile
browser command passed after the correction.

## Cross-platform and external fixture limits

The previously accepted YT-083 Windows MSVC owned-child foundation has its own
actual receipt. The YT-085 persistent-service extension has **not** been executed
on Windows and that old result is not re-labeled as a current full-toolchain test.
The existing Windows MSVC/Rust 1.99 runner was read-only verified as available.
One missing pinned dependency source, `windows-sys 0.52.0`, was fetched from its
public crate archive and checked against the existing Cargo.lock SHA-256
`282be5f36a8ce781fad8c8ae18fa3f9beff57ec1b52cb3de0789201425d9a33d`.
No toolchain/software installation or global settings change occurred. Offline
vendoring then succeeded without a lock change. The generated source/vendor
archive (546 source files) had SHA-256
`060b47bfa7c0cab424b5f874f36b1b1b38947f818da36df7018d7f710d124253`.
WebCodex `transfer_project_artifact` rejected its source path by policy before
transfer. **No rename, encoding or alternative path was used to bypass that
rejection; no Windows build/test was started.** This limit is explicit for the
reviewer to assess against the selected YT-085 requirements.

The two imported Redis browser helpers are ported, but no new Redis server or
three-application fixture was provisioned/executed in this smoke-port task.
Their remaining original Rust/Python qualification callers are explicitly
YT-086 work. Source-based helper parity and pure protocol tests must not be
reported as a new Redis deployment/integration qualification. Actual full
62-minute load, GPU performance matrix, physical IME/AT/mobile, default-enabled
WebGPU, release/publication and hosted workflows were not requalified.

## Gate

No Python or Node executable is used by these Rust smoke commands. Original
Python files remain frozen references until whole-graph adoption in YT-086.
The full migration is not Python-free yet, and no YT-086 task is accepted here.
A distinct independent Reviewer must compare the actual code, original scenario
maps, reports and the explicitly unavailable/non-executed boundaries before
YT-085 can become Done. No partial command may be treated as a successful stub.
## Reviewer acceptance addendum (2026-10-15)

YT-085 final independent verdict: `target/m12/yt085/reviewer-board-retention-20261015/result.json` — PASS, zero blocking issues and required followups. See `YT-085-VERIFICATION.md` for the repair sequence and scoped negative-test evidence. All 26 indexed original runtime receipts remain unchanged (25 successful suites, 1 Chrome GPU-unavailable nonzero); final repairs did **not** rerun browser journeys, and their source-bound verification is limited to the relevant changed paths. Windows persistent-Service behavior and external Redis integration remain unverified. Firefox default-profile Support's HTML fallback is **not** GPU acceptance.
