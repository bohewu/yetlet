# M12 — Complete Python Migration Ledger

**Task:** YT-082 plan; YT-084/YT-085 implementation; YT-086 cutover accepted at bounded scope. **Current state (2026-10-15):** Ten YT-084 mappings and thirty YT-085 mappings are accepted after their own independent Reviewer PASS. Twelve YT-086 file mappings, the transitive Rust-test caller and all three workflows are independently accepted at bounded migration scope ([YT-086 evidence](YT-086-VERIFICATION.md)). Original Python sources are preserved as frozen reference oracles; canonical code/workflow invocations are switched, but complete unexecuted runtime-graph and external-environment qualification remain open. Final independent YT-086 Reviewer `wc_job_6UfDDdHnNCBXkMhA` returned PASS with zero issues/followups.
**Frozen source:** clean `master` at `7cfac613f908c5c19fa581438288386809870b56`.See the [plan](M12-RUST-TOOLCHAIN-PLAN.md) for contracts/adoption gates and
[verification](YT-082-VERIFICATION.md) for actual documentation checks.

## Exact inventory and status rules

Inventory command: `git ls-files '*.py'`. Exactly **52 tracked Python files**,
**14,006 lines**: 35 under `scripts/`, eight scene-island, four Board, two
form-proof and three Redis-qualification files. The three workflows below are
additional invocation surfaces, not Python-file rows. Ignored `.pipeline-output`,
`target/` receipts and copied historical scripts are not tracked source inventory.

Every tracked executable/helper below is conservatively **active**, either a
current default/CI command or a deliberate on-demand feature/qualification tool.
A completed historical feature does not make its runnable verifier historical.
Immutable old reports/screenshots/acceptance prose are **historical evidence**;
they retain source/result identity and do not become developer workflows or new
Rust evidence. No path is retired, deleted, or accepted by this ledger.

For command rows, prepend `cargo xtask ` from the repository root. Angle-bracket
arguments are placeholders. `Rust test NAME` means the proposed direct counterpart
`cargo test --locked --manifest-path xtask/Cargo.toml --test NAME`; each preserves
the old suite's complete cases/assertions. YT-084 commands/tests below now exist; other mappings remain proposed. Preserve
old options/defaults and imported helper APIs as internal Rust functions only as
needed; Rust wrappers that invoke Python do **not** count as migration.

The SHA-256 column freezes each original file's bytes. The inventory digest is
`59a2eb8eb822e6d6054591e6c26625cbfdd2f7620c78dd45a28f19767b66b4e5`, computed
from sorted `git ls-files` order by concatenating UTF-8 lines
`<sha256>  <repository-relative-path>\n`. Later per-family parity snapshots must
also select actual build inputs/tools; this inventory is not a software test run.

## All 52 tracked files

| Old path | Planned Rust command or test counterpart | Responsible item | Active versus historical | Retained behavior/gate | Initial status | Baseline SHA-256 |
|---|---|---|---|---|---|---|
| `examples/form-css-proof/build_assets.py` | `build form-css-proof` | YT-084 | Active opt-in build; receipts historical | Pinned CSS/license/SVG, generated assets, --fetch/--check separation | Ported; parity and independent review PASS ([evidence](YT-084-VERIFICATION.md)) | `33eb18921fa6865e1a52b6cef38d3f0b8b7dcab32c6e469c82824ed350fb8b19` |
| `examples/form-css-proof/verify_browser.py` | `smoke form-css-proof-chrome` | YT-085 | Active opt-in browser; receipts historical | Real form/CSP/DOM journey; synthetic composition never equals OS IME | Rust scenario runtime exit 0, shared-helper regression completed; independent Reviewer PASS (YT-085) ([current evidence](YT-085-RUNTIME-VERIFICATION.md)) | `ac1a4b56013c5cf0b52ea273d68734dd806b28a149c3edffb5533b099ea0ad33` |
| `examples/lab-operations-board/build_assets.py` | `build board-assets` | YT-084 | Active opt-in build; receipts historical | Independent Board asset prefix/manifest/inventory/Rust bytes; pinned CSS integrity | Ported; parity and independent review PASS ([evidence](YT-084-VERIFICATION.md)) | `94de06d0abcc72e482a6e65010b4d01d8d10d729754a67f7d76bf8e1590a6508` |
| `examples/lab-operations-board/qualify_local.py` | `qualify board` | YT-086 | Active opt-in qualification; receipts historical | All five profiles, faults/raw samples/gates/WAL-FULL/SSE/tracing; old long-run receipts historical | Rust candidate; parity/runtime and separate review incomplete ([evidence](YT-086-VERIFICATION.md)) | `1ed36adc03bd42f1b7fb83139f2b74c5d3c75fb1342f7c7ad3eac3bd94e331db` |
| `examples/lab-operations-board/verify_firefox.py` | `smoke board-firefox` | YT-085 | Active opt-in browser; receipts historical | Native keyboard/Unicode/form/status; never physical IME or spoken AT proof | Rust runtime outcome recorded; selected Firefox/Board PASS, Chrome GPU unavailable remains nonzero; independent Reviewer PASS (YT-085) ([exact suite index](YT-085-RUNTIME-VERIFICATION.md)) | `85689c2a48897f1523b7583182de197d79730c6c2db293fd641d168cdd22af82` |
| `examples/lab-operations-board/verify_local.py` | `smoke board-chrome` | YT-085 | Active opt-in integration; receipts historical | Browser/HTTP/auth/restart/diagnostic load, private DB/Cookies, no write replay | Rust runtime outcome recorded; selected Firefox/Board PASS, Chrome GPU unavailable remains nonzero; independent Reviewer PASS (YT-085) ([exact suite index](YT-085-RUNTIME-VERIFICATION.md)) | `e0d67551640f38d4921bb110a761475b7e2b2fca227ec9667adee038e8da925b` |
| `examples/webgpu-scene-island/build.py` | `build scene-island` | YT-084 | Active opt-in build; receipts historical | Separate nested scene/root counter; offline release, source-bound candidate manifest | Ported; parity and independent review PASS ([evidence](YT-084-VERIFICATION.md)) | `b0b8ef2a6bc365633fb5a6045911b29f00033da93edc4ef58b291da0136e1cbf` |
| `examples/webgpu-scene-island/verify_browser.py` | `smoke scene-chrome` | YT-085 | Active opt-in diagnostic; receipts historical | Original actual-Wasm attempt/failure injection and PNG decoder; unsuccessful receipts retained | Rust runtime outcome recorded; selected Firefox/Board PASS, Chrome GPU unavailable remains nonzero; independent Reviewer PASS (YT-085) ([exact suite index](YT-085-RUNTIME-VERIFICATION.md)) | `a468b1c92973087bdf5ed18e7c3bb94511f2c98f6adafac3afca6162b4ef8411` |
| `examples/webgpu-scene-island/verify_candidate.py` | `smoke scene-candidate-firefox` | YT-085 | Active opt-in browser; receipts historical | Extracted candidate/free hooks, ownership/retirement and actual PNG assertions | Rust runtime outcome recorded; selected Firefox/Board PASS, Chrome GPU unavailable remains nonzero; independent Reviewer PASS (YT-085) ([exact suite index](YT-085-RUNTIME-VERIFICATION.md)) | `140ded0ae8d044e1b5e091bc1eebc904fc5c66cecc0fe6141ff95c480754ff73` |
| `examples/webgpu-scene-island/verify_dynamic.py` | `measure scene-dynamic` | YT-086 | Active opt-in measurement; receipts historical | Matched Canvas2D/WebGPU lanes, raw samples/pixels, deadlines, no overwrite/no speedup invention | Rust candidate; parity/runtime and separate review incomplete ([evidence](YT-086-VERIFICATION.md)) | `17554e2c53d108dea82025b8ff55774cf87348464da8a190aca57c97f96775b8` |
| `examples/webgpu-scene-island/verify_firefox.py` | `smoke scene-firefox` | YT-085 | Active opt-in browser; receipts historical | Selected Firefox X11 experimental preference, actual Wasm pixels/fallback/CSP/counter | Rust runtime outcome recorded; selected Firefox/Board PASS, Chrome GPU unavailable remains nonzero; independent Reviewer PASS (YT-085) ([exact suite index](YT-085-RUNTIME-VERIFICATION.md)) | `0fd4b2ac672e2c929252a373175871fb399f54f4f1654bdb377b2bc32870890c` |
| `examples/webgpu-scene-island/verify_lifecycle.py` | `smoke scene-lifecycle-firefox` | YT-085 | Active opt-in browser; receipts historical | Resize/loss/recovery/late completion/owner cleanup and actual PNG assertions | Rust runtime outcome recorded; selected Firefox/Board PASS, Chrome GPU unavailable remains nonzero; independent Reviewer PASS (YT-085) ([exact suite index](YT-085-RUNTIME-VERIFICATION.md)) | `1dd031df30f1d48a53a5d104eef2e19939d5675b71b1929487369a50587b74cf` |
| `examples/webgpu-scene-island/verify_picking.py` | `smoke scene-picking-firefox` | YT-085 | Active opt-in browser; receipts historical | Stable selection/stale rejection, portrait viewport, HTML alternatives and pixels | Rust runtime outcome recorded; selected Firefox/Board PASS, Chrome GPU unavailable remains nonzero; independent Reviewer PASS (YT-085) ([exact suite index](YT-085-RUNTIME-VERIFICATION.md)) | `5b6afa867a2923d601be9321a905ad805a5df584fb49d7867ba3285e11a2a8f5` |
| `examples/webgpu-scene-island/verify_support.py` | `smoke scene-support-firefox` | YT-085 | Active opt-in browser; receipts historical | Default-profile observation without experimental override; separate from selected opt-in acceptance | Rust runtime outcome recorded; selected Firefox/Board PASS, Chrome GPU unavailable remains nonzero; independent Reviewer PASS (YT-085) ([exact suite index](YT-085-RUNTIME-VERIFICATION.md)) | `10eaef427b3ca8407b6a6ba24d5c7ea97c528d8b3b0608222ad0bc8c8887b5da` |
| `qualification/yt066b/tests/cookie_browser.py` | `fixture redis-cookie-browser <base-url>` | YT-085 (caller YT-086) | Active Rust-test helper; receipts historical | No-JS Chrome, 180 delayed HTTP responses, retired-cookie rejection/relogin/logout, owned cleanup | Rust helper implemented; protocol/source tests only, external Redis fixture not rerun; caller migration YT-086 and independent Reviewer PASS (YT-085) ([limits](YT-085-RUNTIME-VERIFICATION.md)) | `81d1e6fcfa8f74be6be66f4369a42219af27cc637bfbba611d4e35530b9fca18` |
| `qualification/yt066b3/browser.py` | `fixture redis-three-app-browser <base-url>` | YT-085 (caller YT-086) | Active opt-in helper; receipts historical | Actual no-JS HTTP Cookie behavior across three apps; reusable browser probe | Rust helper implemented; protocol/source tests only, external Redis fixture not rerun; caller migration YT-086 and independent Reviewer PASS (YT-085) ([limits](YT-085-RUNTIME-VERIFICATION.md)) | `f95192b69ff8d02a11f1f922ab86cf8d20f41e3b5caee09d963fb0d0001c49c3` |
| `qualification/yt066b3/run.py` | `qualify redis-three-app` | YT-086 | Active opt-in qualification; receipts historical | Disposable three-app Redis, CAS/retirement/TTL/unknown outcomes; imported Docker and browser helpers | Rust candidate; parity/runtime and separate review incomplete ([evidence](YT-086-VERIFICATION.md)) | `abb1b8c805666382a59e1a3fc0e38baaa62fff2a364881b8268ca4b5a66e211d` |
| `scripts/build_browser.py` | `build browser` | YT-084 | Active build/CI; receipts historical | Separate Todo/Search builds; dist schema/hash/HTML/CSS/loaders; --release | Ported; parity and independent review PASS ([evidence](YT-084-VERIFICATION.md)) | `190b02dc8e334b3066fea85ca413a051c5130ffbb27cfdaec6383fa6cf335ac3` |
| `scripts/build_keyed_bindings.py` | `build keyed-bindings` | YT-084 | Active build/CI; receipts historical | Isolated M7b output and matching SSR/browser fixture; --release | Ported; parity and independent review PASS ([evidence](YT-084-VERIFICATION.md)) | `5de6d85161af7f851df69fbe6a7c09d31dac598bbc5c5ac4570cbc78f6c2aa1b` |
| `scripts/build_multi_island_dashboard.py` | `build multi-island-dashboard` | YT-084 | Active build/CI; receipts historical | Independently built Counter/Filter; dashboard prefix/manifest and byte isolation; --release | Ported; parity and independent review PASS ([evidence](YT-084-VERIFICATION.md)) | `dcfe11428b26c6241e86e5cf830afb015d9f1a8851390b89537d4a39ebbf5142` |
| `scripts/build_render_bench.py` | `build render-bench` | YT-084 | Active opt-in build; receipts historical | Isolated M7a benchmark output and untouched production dist; --release | Ported; parity and independent review PASS ([evidence](YT-084-VERIFICATION.md)) | `9da166685f0c36625c5955cfd2a58b4ba63476a719cabc618f3169f4b8ef75c8` |
| `scripts/check_docs.py` | `check docs` | YT-084 | Active check/CI; receipts historical | Required docs/English/links/fences/whitespace/backlog IDs/dependency DAG | Ported; parity and independent review PASS ([evidence](YT-084-VERIFICATION.md)) | `2853791fbbecb6aad13d0fd990784a4665f15ebb47dce3438c91e1b0f0d73ee5` |
| `scripts/check_samples.py` | `check samples` | YT-084 | Active check/CI; receipts historical | Catalog schema/type/paths/README command/routes/companions/dependency/browser-build identity | Ported; parity and independent review PASS ([evidence](YT-084-VERIFICATION.md)) | `e7ddad8713b8a52e1764da4b5bb45779941d4fcb8106c612b70ced5ed4d54c0a` |
| `scripts/measure_baseline.py` | `measure baseline` | YT-086 | Active measurement/CI; receipts historical | SSR first byte/assets/gzip/Chrome readiness, informational; profile/samples/browser-runs/out | Rust candidate; parity/runtime and separate review incomplete ([evidence](YT-086-VERIFICATION.md)) | `a75fb86da391e6e6a0585718c2207ea8c54b59206979b2e0654edf7ae1b56948` |
| `scripts/measure_render_strategies.py` | `measure render-strategies` | YT-086 | Active opt-in measurement; receipts historical | Production-before capture, seeded rotating strategies, equivalence/focus/counts/percentiles | Rust candidate; parity/runtime and separate review incomplete ([evidence](YT-086-VERIFICATION.md)) | `071f741b36aef37b439e8f0928523a1bd78950ef513d1fc1c2581617b9f9a0fc` |
| `scripts/measure_sample_dx.py` | `measure sample-dx` | YT-086 | Active opt-in measurement; receipts historical | Source facts and transitive sample-package/direct non-dev dependency closure; optional --out | Rust candidate; parity/runtime and separate review incomplete ([evidence](YT-086-VERIFICATION.md)) | `f70e701511fada24d81f091ea9df7d9bd2b10f477148088fc98d9565f05e41d0` |
| `scripts/qualify_local_operations.py` | `qualify local-operations` | YT-086 | Active opt-in qualification; receipts historical | Prebuilt processes/restart/faults, SQLite online backup/read-only/lock/corruption, redaction | Rust candidate; parity/runtime and separate review incomplete ([evidence](YT-086-VERIFICATION.md)) | `02c0016ce46024b9190a521444d29f26862ba9ff7b824f1154f957bbe204fc6d` |
| `scripts/qualify_multi_instance.py` | `qualify multi-instance` | YT-086 | Active opt-in qualification; receipts historical | Pinned loopback Docker/nginx, stateless routing versus Memory Session limits, exact-owner cleanup | Rust candidate; parity/runtime and separate review incomplete ([evidence](YT-086-VERIFICATION.md)) | `c507ad117b7e221749b7be1a1534710825617583b28af971b8b2f49b9c713e09` |
| `scripts/release_checksums.py` | `release checksums <create-or-verify> <directory>` | YT-086 | Active release check/CI helper; receipts historical | Flat unsigned checksum schema, POSIX no-follow, concurrent mutation/limits/duplicate rejection | Rust candidate; parity/runtime and separate review incomplete ([evidence](YT-086-VERIFICATION.md)) | `34a50ec747b3204ea40388e4c7e6cf955320dd437219d65b17a9452af6e33021` |
| `scripts/smoke_auth_notes.py` | `smoke auth-notes` | YT-085 | Active native/CI; receipts historical | Synthetic identity, session rotation, CSRF/IDOR, private logs and cleanup | Rust scenario runtime exit 0, shared-helper regression completed; independent Reviewer PASS (YT-085) ([current evidence](YT-085-RUNTIME-VERIFICATION.md)) | `d48d29ed1b311f1de80653c110ec47b2bdf392ac69dafcd98f4e4d6844adfb8c` |
| `scripts/smoke_browser.py` | `smoke browser-chrome` | YT-085 | Active browser/CI + shared helper; receipts historical | Todo/Search SSR/DOM/local/typed HTTP/CSP/lifetime races; CDP/process/profile helper parity | Rust scenario runtime exit 0, shared-helper regression completed; independent Reviewer PASS (YT-085) ([current evidence](YT-085-RUNTIME-VERIFICATION.md)) | `ee9c19f000cc0cbf381d39fcf377f875370066dbe808d1b12fd92f403e4222e7` |
| `scripts/smoke_cart.py` | `smoke cart` | YT-085 | Active native/CI; receipts historical | SSR Session cart, native forms, CSRF and refresh persistence | Rust scenario runtime exit 0, shared-helper regression completed; independent Reviewer PASS (YT-085) ([current evidence](YT-085-RUNTIME-VERIFICATION.md)) | `cf77507e74a1d0be84e7a74a463393b03e486a8ac15b5329d6cdf050ed99e46c` |
| `scripts/smoke_file_server.py` | `smoke file-server` | YT-085 | Active native/CI; receipts historical | Fixed static/opaque downloads, protected multipart bounds, upload cleanup | Rust scenario runtime exit 0, shared-helper regression completed; independent Reviewer PASS (YT-085) ([current evidence](YT-085-RUNTIME-VERIFICATION.md)) | `19c526d7c376d22903ab2051a06bf04aafb34a54bc4596aa867815149e20280d` |
| `scripts/smoke_firefox.py` | `smoke browser-firefox` | YT-085 | Active browser/CI + shared helper; receipts historical | Todo/Search release scenarios plus FirefoxDriver/await helpers, ownership and cleanup | Rust scenario runtime exit 0, shared-helper regression completed; independent Reviewer PASS (YT-085) ([current evidence](YT-085-RUNTIME-VERIFICATION.md)) | `bf8fdf7477bde517ec063a9159c2ede162053709b1a050d20a3b3787c0dcd4c6` |
| `scripts/smoke_forms_accessibility_firefox.py` | `smoke forms-accessibility-firefox` | YT-085 | Active browser/CI + shared helper; receipts historical | W3C key/focus/labels/English semantics, sanitized forms report; not IME/AT certification | Rust scenario runtime exit 0, shared-helper regression completed; independent Reviewer PASS (YT-085) ([current evidence](YT-085-RUNTIME-VERIFICATION.md)) | `7bb23c92d3ade0eaa17056f92b9dd2350bc1bae6ec633395287dc4a20edf29d1` |
| `scripts/smoke_forms_validation.py` | `smoke forms-validation` | YT-085 | Active native/CI + helper; receipts historical | Escaped errors/stateless submit/success/no-side-effect 303 PRG, raw HTTP client behavior | Rust scenario runtime exit 0, shared-helper regression completed; independent Reviewer PASS (YT-085) ([current evidence](YT-085-RUNTIME-VERIFICATION.md)) | `428a23b45a4147c35e1d3292b574e05ba78868f625a7b1d986bc42d1dcefed4d` |
| `scripts/smoke_hello_world.py` | `smoke hello-world` | YT-085 | Active native/CI; receipts historical | SSR named route/escaping/strict CSP and no browser requirements | Rust scenario runtime exit 0, shared-helper regression completed; independent Reviewer PASS (YT-085) ([current evidence](YT-085-RUNTIME-VERIFICATION.md)) | `6b15eaf400f6d904f2db6d793054c3c7ccd60a541578eb423bee0934bd5acf4e` |
| `scripts/smoke_keyed_bindings.py` | `smoke keyed-bindings-chrome` | YT-085 | Active browser/CI + helper; receipts historical | Keyed identity/slots, synchronous graph/batching/focus/selection/disposal; isolated static server | Rust scenario runtime exit 0, shared-helper regression completed; independent Reviewer PASS (YT-085) ([current evidence](YT-085-RUNTIME-VERIFICATION.md)) | `fed8d8b9529434d3b5bb74f45b0b004039108cfb4eaab7eabd738a4d9a97df22` |
| `scripts/smoke_keyed_bindings_firefox.py` | `smoke keyed-bindings-firefox` | YT-085 | Active browser/CI; receipts historical | Firefox keyed lifecycle/form-state/focus/selection/disposal | Rust scenario runtime exit 0, shared-helper regression completed; independent Reviewer PASS (YT-085) ([current evidence](YT-085-RUNTIME-VERIFICATION.md)) | `f0979db16cfa39c399e4364806a80243dd458d20164813172cada9bbf124ab93` |
| `scripts/smoke_middleware_lab.py` | `smoke middleware-lab` | YT-085 | Active native/CI; receipts historical | Native Tower scope/order/short circuit/stream and broken-stream cases | Rust scenario runtime exit 0, shared-helper regression completed; independent Reviewer PASS (YT-085) ([current evidence](YT-085-RUNTIME-VERIFICATION.md)) | `df196e65cbb7f8b763512efbd6803e2f3d28205b400531e2bef9db747b482a96` |
| `scripts/smoke_modular_shop.py` | `smoke modular-shop` | YT-085 | Active native/CI; receipts historical | Identity/cart/direct-SQL orders, transaction rollback/idempotency/authorization/disabled modules | Rust scenario runtime exit 0, shared-helper regression completed; independent Reviewer PASS (YT-085) ([current evidence](YT-085-RUNTIME-VERIFICATION.md)) | `7155dc94f80a07234e81b8f26338700de6ec01dbd8fc23075a047b73b5d7170c` |
| `scripts/smoke_multi_island_dashboard.py` | `smoke multi-island-dashboard-chrome` | YT-085 | Active browser/CI; receipts historical | Independent ownership/lazy bundles/disposal, malformed/partial manifest fallback | Rust scenario runtime exit 0, shared-helper regression completed; independent Reviewer PASS (YT-085) ([current evidence](YT-085-RUNTIME-VERIFICATION.md)) | `6d956eb9f4161c8f87454a48f594fc9adf68c046e4341d9a83c6965ac62e664f` |
| `scripts/smoke_multi_island_dashboard_firefox.py` | `smoke multi-island-dashboard-firefox` | YT-085 | Active browser/CI; receipts historical | Same independent-island and malformed-manifest semantics in Firefox | Rust scenario runtime exit 0, shared-helper regression completed; independent Reviewer PASS (YT-085) ([current evidence](YT-085-RUNTIME-VERIFICATION.md)) | `059a361b73840e9e97f61284aa8b129190c3c7525cfec62ffea74e2a974a5369` |
| `scripts/smoke_rest_api.py` | `smoke rest-api` | YT-085 | Active native/CI; receipts historical | Typed Book CRUD/errors/request context/request correlation, API-only dependency boundary | Rust scenario runtime exit 0, shared-helper regression completed; independent Reviewer PASS (YT-085) ([current evidence](YT-085-RUNTIME-VERIFICATION.md)) | `adfbb2eb21f38d4670ed9704a362062bf1c07b2aa7a671d4a85fdf52abe31adb` |
| `scripts/smoke_sqlite_crud.py` | `smoke sqlite-crud` | YT-085 | Active native/CI + helper; receipts historical | Restart persistence, transactions/optimistic versions/conflicts and real HTTP semantics | Rust scenario runtime exit 0, shared-helper regression completed; independent Reviewer PASS (YT-085) ([current evidence](YT-085-RUNTIME-VERIFICATION.md)) | `5570245cf375adead2059c5be28f6befd97789463099c2370a30e0aab7cfcc0b` |
| `scripts/smoke_sse_notifications.py` | `smoke sse-notifications` | YT-085 | Active native/CI; receipts historical | Heartbeat/publish/replay/Last-Event-ID/disconnect/subscriber cleanup | Rust scenario runtime exit 0, shared-helper regression completed; independent Reviewer PASS (YT-085) ([current evidence](YT-085-RUNTIME-VERIFICATION.md)) | `bafb19fa3c202bf31b51e5cfa9a3c9d9f12b2daef2ef7a98c95150b40d8818ef` |
| `scripts/test_check_samples.py` | `Rust test check_samples` | YT-084 | Active tool tests/CI; receipts historical | All existing positive/negative catalog test cases (listed below) | Ported; parity and independent review PASS ([evidence](YT-084-VERIFICATION.md)) | `808667a57631042e8c0ceb0c4f65637ab427417e58ee7fb4f6f58525616c81bf` |
| `scripts/test_local_operations.py` | `Rust test local_operations` | YT-086 | Active tool tests; receipts historical | All existing startup/cleanup/redaction/cancel/backup/deadline regressions (listed below) | Rust candidate; parity/runtime and separate review incomplete ([evidence](YT-086-VERIFICATION.md)) | `0d2c82577e15ef6790e63ce795bef2202ca8cfa81852dd2f4192165c05c8ac1b` |
| `scripts/test_qualify_multi_instance.py` | `Rust test qualify_multi_instance` | YT-086 | Active tool tests; receipts historical | All existing source/redaction/container/signal/report-write/cleanup regressions (listed below) | Rust candidate; parity/runtime and separate review incomplete ([evidence](YT-086-VERIFICATION.md)) | `850ea4037e55a68deaafb7a7dd534d12ee0ec147d724f34a15a107cf8e2276d7` |
| `scripts/test_release_checksums.py` | `Rust test release_checksums` | YT-086 | Active tool tests; receipts historical | All existing checksum/security/no-follow/limits/CLI regressions (listed below) | Rust candidate; parity/runtime and separate review incomplete ([evidence](YT-086-VERIFICATION.md)) | `412a4b6a8bf1c5c5d31d654900b78a0d7e9ec041aa1f2177e646e4737ef2d8d6` |
| `scripts/test_smoke_browser_harness.py` | `Rust test smoke_browser_harness` | YT-085 | Active tool tests/CI; receipts historical | Profile args, owned descendant reap, transient profile removal (listed below) | Rust cases PASS; independent Reviewer PASS (YT-085) ([evidence](YT-085-VERIFICATION.md)) | `c1eb7f67d2dc0ed370256d6915f4fae60568e6e12633ada17e53c768fec182c6` |
| `scripts/test_smoke_modular_shop.py` | `Rust test smoke_modular_shop` | YT-085 | Active tool tests/CI; receipts historical | Failed readiness preserves error, terminate/escalate/reap/stream-close (listed below) | Rust cases PASS; independent Reviewer PASS (YT-085) ([evidence](YT-085-VERIFICATION.md)) | `b6df3adc28e00b94334d93b55ff68ac4cef6bfe4b19f509cc0507c3061e7fd53` |

## Existing direct test cases retained as Rust fixtures

These names identify existing behavior, not a new proof framework. Preserve all
subcases/data mutations and assertions inside each named case, not merely their
count or a successful happy path. Platform-dependent tests retain their explicit
capability limits; no unsupported successful skip may satisfy a required gate.

### `scripts/test_check_samples.py` — 4 existing cases

- `test_current_catalog_passes`
- `test_false_command_route_browser_and_dependency_claims_fail`
- `test_repository_path_escape_fails`
- `test_schema_and_malformed_types_fail_without_crashing`

### `scripts/test_local_operations.py` — 10 existing cases

- `test_registered_before_failed_readiness_and_reaped`
- `test_stop_escalates_timeout_to_kill_and_wait`
- `test_all_cleanup_attempted_after_one_failure`
- `test_readiness_deadline_and_corrupt_candidate`
- `test_spawn_failure_redacted_after_cleanup`
- `test_cancel_after_spawn_still_reaps_and_reports`
- `test_success_report_written_only_after_cleanup`
- `test_cleanup_failure_prevents_pass_report`
- `test_online_backup_actual_stdlib_checkpoint_and_no_overwrite`
- `test_backup_progress_deadline_interrupts_busy_retry`

### `scripts/test_qualify_multi_instance.py` — 18 existing cases

- `test_last_upstream_uses_successful_retry`
- `test_report_rejects_fixture_codes_and_raw_cookie`
- `test_compose_is_loopback_only_and_apps_publish_no_ports`
- `test_dockerfile_uses_exact_runtime_and_nonroot_user`
- `test_write_report_is_atomic_and_bounded`
- `test_nginx_places_every_temp_path_on_tmpfs`
- `test_cleanup_attempts_forced_removal_after_down_failure`
- `test_cleanup_fails_when_final_queries_fail`
- `test_cleanup_fails_when_owned_image_remains`
- `test_sigterm_after_compose_start_cleans_and_cannot_pass`
- `test_first_sigterm_during_cleanup_records_cancel_and_finishes_cleanup`
- `test_first_sigterm_at_report_write_rewrites_nonpassing_result`
- `test_sigterm_at_cleanup_transition_still_runs_cleanup`
- `test_report_write_failure_still_removes_owned_work_and_cannot_pass`
- `test_owned_work_removal_failure_prevents_pass`
- `test_first_sigint_during_cleanup_records_cancel_and_finishes_cleanup`
- `test_sigterm_then_sigint_is_ignored_during_cleanup`
- `test_sigint_then_sigterm_is_ignored_during_cleanup`

### `scripts/test_release_checksums.py` — 16 existing cases

- `test_cli_success_and_manifest_exclusion`
- `test_canonical_content_for_identical_inputs`
- `test_no_overwrite`
- `test_tampered_artifact_and_hash`
- `test_missing_and_extra`
- `test_unsafe_manifest_paths_and_self_reference`
- `test_duplicate_entries_and_json_keys`
- `test_symlink_artifact_create_and_verify`
- `test_symlink_manifest_never_overwritten_or_read`
- `test_symlink_directory_and_ancestor`
- `test_directory_traversal_rejected`
- `test_nonregular_files`
- `test_nonempty_input`
- `test_count_file_total_and_manifest_limits`
- `test_malformed_manifest_and_types`
- `test_failure_output_does_not_disclose_paths`

### `scripts/test_smoke_browser_harness.py` — 3 existing cases

- `test_profile_argument_selects_debug_or_release`
- `test_owned_process_group_is_terminated_and_reaped`
- `test_owned_tree_retries_transient_not_empty_error`

### `scripts/test_smoke_modular_shop.py` — 2 existing cases

- `test_readiness_failure_terminates_reaps_and_closes_stream`
- `test_readiness_failure_kills_after_terminate_timeout`

## Transitive invocation and output adoption

- YT-084 owns exactly seven builders: four `scripts/build_*`, the two CSS builders
  and scene-island `build.py`. Keep root/nested lock selection and independent
  Cargo entry builds. Existing output roots remain `dist/`,
  `target/m7a-render-bench/{debug,release}`,
  `target/m7b-keyed-bindings/{debug,release}`,
  `target/multi-island-dashboard-dist`, `target/webgpu-scene-island-dist` and each
  CSS fixture's `assets/` plus `src/asset_data.rs`. Compare actual old selected
  bytes/manifest fields and generated outputs before switching commands.
- YT-085 replaces imports of `smoke_browser`, `smoke_firefox`, forms HTTP/key
  helpers, keyed static-server helpers and scene PNG/request/viewport helpers
  with Rust internal counterparts. The Board and qualification callers must use
  these implementations rather than keep hidden interpreter imports.
- `qualification/yt066b/tests/cookie_fencing.rs` directly invokes `python3` on
  `tests/cookie_browser.py`, passing its already-owned loopback base URL and
  checking status/output. YT-085 supplies the Rust browser fixture; YT-086 switches
  this isolated Rust test caller to that executable with explicit argv/status
  and retains server/provider shutdown and replacement-session assertions. It
  must not recursively invoke a parent Cargo test or require a Python fallback.
- `qualification/yt066b3/run.py` imports `browser.py` and
  `scripts/qualify_multi_instance.py`; the Docker helper in turn owns lifecycle,
  redaction and report logic. Port these transitive edges, not just `run.py`.
- Native/local operations reuse auth/SQLite smoke clients; benchmark tools reuse
  CDP and process helpers. No Rust command may spawn its former Python helper.
- Live references include root/sample READMEs, `docs/SAMPLES.md`,
  `examples/samples.toml`, `AGENTS.md`, CONTRIBUTING and active verification
  guidance. The runtime diagnostic in `examples/todo-server/src/lib.rs` also
  names the old browser build command; update that message with YT-084 adoption,
  without changing framework behavior. Catalog `browser_build` must become a
  validated Rust command identity together with its checker and exact README
  command lines. Root `cargo run/build/test` remain supported.
- Report paths/schemas, raw samples, source/tool identities and sanitized
  failure/cleanup evidence remain suite-specific. Examples include
  `target/qualification/forms-accessibility-firefox.json`, Board reports,
  `target/m5-baseline*.json` and source-bound scene records. Never overwrite an
  immutable receipt to make a port appear equivalent. Current test fixtures and
  fresh port reports have separate identities from historical evidence.

## Three workflows and inline Python (additional to the 52)

All three have a YT-086 Rust invocation candidate; no hosted execution or independent migration acceptance is claimed. YT-084 adopts only the docs/catalog checks and their direct Rust tests in `rust.yml`, after exact catalog parity; that narrow early adoption is necessary because the catalog now names validated Rust build commands. Build, smoke, qualification and release CI invocations remain unchanged for YT-086. No hosted workflow was triggered.

| Workflow at baseline | Existing Python/process surface | Planned Rust adoption and owner | Gates retained |
|---|---|---|---|
| `.github/workflows/rust.yml` | Docs/catalog and three harness-test invocations; ten native smokes; three builders; Chrome Todo/keyed/Dashboard suites; M5 baseline | YT-084/YT-085/YT-086 provide the mapped commands/tests above; YT-084 switches only docs/catalog/test invocations after parity; YT-086 switches all remaining invocations | MSRV 1.99 and rolling stable, fmt/Clippy/native/doctest/wasm targets, RustSec audit, exact Chrome release suites, informational baseline flags `--samples 15 --browser-runs 3 --out target/m5-baseline-ci.json`, job budgets and failure/baseline artifact policy |
| `.github/workflows/qualification.yml` | Python setup/version; three release builders; Todo/keyed/Dashboard Firefox and forms-accessibility suites | YT-086 removes Python prerequisites and switches to Rust equivalents after YT-084/085; native matrix stays native Cargo | Scheduled/manual activation boundary, native Ubuntu/Windows/macOS matrix and nine-crate selection, Firefox/geckodriver capability fail-closed checks, pinned matching wasm-bindgen 0.2.129, keyboard/sanitized forms evidence, stable identity logs, timeouts/concurrency/retention |
| `.github/workflows/private-rebuild.yml` | Python setup; **inline heredoc** archive/input/build/provenance logic; transitive checksum create/verify on extracted source | YT-086 `cargo xtask release private-rebuild` plus internal Rust checksum functions or explicit mapped command; Rust bootstrap isolated from package source/fresh caches | Manual-only, immutable clean source/ref/workflow identity and approved dispatch policy, nine-package allowlist/18 license aliases, bounded safe extraction, selected-input digests, pinned tools/fresh homes/locked fetch/offline package, unsigned provenance/checksums, read-only token, full-SHA actions, 25-minute budget and 7-day retention |

The private-rebuild heredoc is a real migration obligation despite not being a
`.py` path. Do not remove only setup-python and leave inline Python, replace its
selected-input/provenance logic with a generic build, or widen tokens/ref policy.
Current workflow provisioning commands are existing hosted configuration, not
permission for local tools to auto-install or this documentation task to execute
hosted work. No push or actual hosted trigger is part of YT-082–YT-086 migration
validation. YT-020's signing/publication/trust gates stay Blocked.

## Other selected baseline inputs

These are actual source fingerprints in addition to the 52 rows. Selection is
exactly the three workflows, root Cargo manifest/lock/toolchain, and the direct
Rust test caller. Other command-specific inputs must be snapshotted when that
family is migrated, without mutating old evidence.

| Input | Baseline SHA-256 |
|---|---|
| `.github/workflows/private-rebuild.yml` | `71baab935fcefc07060a1e7dedd7e0d10e30f30bd9a2797d58ab73d0114a8dda` |
| `.github/workflows/qualification.yml` | `a575fd7f714e85f8e65c3889d1fa75a44a7564f7d110940a70ef1b2fc557f7fb` |
| `.github/workflows/rust.yml` | `26d4587b8259e3e32e9bebbfafc15169fb90381f2dbd98bbc30e2ceaf0a06f90` |
| `Cargo.toml` | `b61d633b2e32dea7a2a5fff75a318963615f3a990ba6f09020893b3d4cf9b9c3` |
| `Cargo.lock` | `2e08330c49ff4e1740102a37b36fbe2fd044bd2033d8ea9a6aaf224a402866c3` |
| `rust-toolchain.toml` | `b2c19d86958ed6810741488db5cd352856f303c53bcf0ea774e72f94ee4d201e` |
| `qualification/yt066b/tests/cookie_fencing.rs` | `de96bbaa13a62f7a50daf59c02315433b35ce074584a45e75586dcda8dd06b67` |

## Closure rules

A mapping becomes migrated only with its responsible item's exact parity evidence
and independent Reviewer PASS; any retirement needs that evidence first. Current
YT-084 rows report ported/parity PASS with review pending; the other 42 file
rows and three whole workflows remain Not migrated. Track partial ports and
unavailable gates explicitly rather than relabeling active files historical.
YT-086 closure requires an audited Python-free transitive invocation graph,
controlled PATH/blocked-interpreter execution and nonzero unavailable-capability
checks. Old immutable qualification records may remain unchanged; absence of
Python is never achieved by deleting their acceptance coverage or reducing
scenario-specific gates. Full-load/physical IME/AT/production acceptance is not
rerun or newly claimed by the language port.

## YT-084 implementation and early adoption

The seven builders, two checkers and direct `check_samples` Rust test counterpart
are ported; the ten file mappings above remain subject to parent independent
review before YT-084 can become Done. See [actual verification](YT-084-VERIFICATION.md)
for per-profile counts, exact retained goldens and logs. All original Python
sources/tests remain byte-identical reference oracles; none is invoked by a
canonical Rust build/check command. Transitional differential tests explicitly
ran Python before adoption, then compared 128 independently generated files.

Current root/sample documentation, sample catalog and the Todo missing-assets
message now name the matching Rust commands. `browser_build` accepts the exact
supported `cargo xtask build browser` or `cargo xtask build multi-island-dashboard`
identity, checks implementation/package membership, command dispatch availability
and matching README/launch command. Legacy path identities remain accepted only
for transitional catalogs. No package, dependency, source-marker, route or path
assertion was dropped. The original Python checker is preserved and still serves
legacy catalog snapshots; it cannot validate the adopted command representation,
so the matching CI checker and direct tests switch together. Unrelated CI gates
and all old acceptance prose stay unchanged. No existing gate is reclassified.

YT-085/YT-086 tools, transitive Python imports, browser/qualification capabilities,
whole-workflow migration and the controlled-PATH Python-free graph proof are
still **Not migrated**. YT-083 foundation receipts are unchanged. No old M11
receipt attests this new tool, and no new GPU/browser or operating-envelope
acceptance is claimed. YT-087 remains outside scope.

## YT-085 initial terminal checkpoint (historical)

[YT-085 verification](YT-085-VERIFICATION.md) maps the original assertions and five
direct cases to actual Rust files/functions. Ten native, eight browser and two
browser-fixture commands have implementations, plus the two direct Rust test
counterparts. Their presence is not full parity: eight Board/scene commands are
absent, and the report lists remaining assertions/reporting gaps in implemented
suites. No missing command has a passing stub. Existing Python callers and live
smoke invocations remain active; no Python-free repository claim is made.

The new direct cases and protocol tests pass, as do full xtask tests, strict
Clippy/fmt and selected existing sample tests. Actual native/browser acceptance
could not start because owned loopback socket creation returns PermissionDenied
in this execution sandbox. All runtime gates remain Blocked/Not run, not PASS.
No scene/default-profile/GPU evidence was borrowed from M11. The four product
correction rounds and two operational retries are exhausted. YT-085 stays
In progress pending completion and parent independent Reviewer PASS; YT-086 and
all three whole-workflow migrations remain Not migrated. YT-084's accepted
review and existing receipts remain unchanged.

## YT-085 remaining-eight implementation checkpoint

All eight previously absent Board/scene commands now have separate actual Rust
scenarios. [YT-085 verification](YT-085-VERIFICATION.md#remaining-eight-implementation-checkpoint)
maps every original source and declared Dashboard/proof omission, current Rust
build-source manifest/report compatibility, offline checks, counter history and
exact parent runtime commands. Presence is not migration acceptance: these eight
runtime sequences are Not run, shared-helper/report deltas are unverified at
runtime, and independent review is pending. No row is mass-marked migrated.

The parent's separate host receipts cover ten native and the earlier eight
browser suites with exit 0 at their earlier exact sources. The two aggregate
receipts were read; their results do not extend to the new commands or changed
sources. The earlier workspace-write PermissionDenied remains historical terminal
evidence, with original exhausted 4/4 repairs and 2/2 operational retries. This
bounded first implementation of the missing commands uses its own 4-repair,
1-operational-retry limit and preserves those failures. Its final counters and
checks are in the verification record. No browser/server/socket probe ran here.

The old Python files and active CI/smoke invocations remain for transition.
Qualification callers and their actual runtime, Windows persistent-service
execution and final independent review are still separate gates; YT-086 cannot
start yet. The changed scene manifest's additive `rust_build_sources` requires a
current Rust scene build for the parent's batch. It preserves candidate hashes
and asset digest rules; M11 evidence remains immutable. No new GPU support,
public CLI, physical IME/AT/mobile or duration-qualification claim is made.
## YT-085 final independent acceptance (2026-10-15)

All 30 YT-085 Python-source mapping implementations, including the two Redis browser fixture ports whose **callers remain YT-086**, have passed final independent read-only Reviewer review (`target/m12/yt085/reviewer-board-retention-20261015/result.json`, `overall_status=pass`, no issues/followups). The 26 source-bound runtime receipts remain unchanged: 25 positive scenarios plus one nonzero `scene-chrome` GPU-unavailable result; default-profile Firefox Support verifies semantic fallback with `gpu_acceptance=false`. Subsequent focused Server, Browser and suite/Board cleanup-status/receipt repairs are evidence-scoped and did not replay historical browser journeys; 16 final focused Board tests, fmt and strict Clippy passed. The original 52 Python files and historical hashes are preserved. Windows persistent-Service and external Redis fixtures are **not verified**. The full active Python invocation graph and three workflows remain YT-086 work; no Python-free claim yet.



## YT-086 candidate checkpoint — 2026-10-15

The exact twelve rows above now have native Rust command/test counterparts.
Private-rebuild inline logic is implemented under `xtask/src/release/private.rs`;
all three workflow invocation surfaces and the Rust browser-fixture caller are
switched in the working tree. This supersedes earlier pending-cutover text only
for current implementation status. Source oracles, prior receipts and YT-085
acceptance are unchanged. [YT-086 verification](YT-086-VERIFICATION.md) records
actual passing checks and failed/unexecuted runtime paths, test coverage limits,
repair counters and remaining independent review. YT-086 is In progress, not Done;
complete parity and Python-free runtime invocation closure are not self-certified.
