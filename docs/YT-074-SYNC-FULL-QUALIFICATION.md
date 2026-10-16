# YT-074 — Instrumented Full Synchronized Qualification

**Date:** 2026-10-13. **Terminal result: FAIL, exit 1.** The single authorized 62-minute-profile synchronized attempt stopped after warmup and three measured rounds because round 3 write p95 was **548.889 ms**, exceeding the unchanged <500 ms gate. Actual execution was approximately 32 minutes; the 30-minute soak was **Not run**. Existing [mixed-load acceptance](YT-074-QUALIFICATION.md), the [paired short comparison](YT-074-SYNC-COMPARISON.md) and historical synchronized failures remain recorded separately.

**Later investigation:** the separately requested [fsync diagnosis](YT-074-FSYNC-DIAGNOSIS.md) reproduced three 505–508 ms stalls in an untraced private file without SQLite or HTTP, with journal-commit waiting and matching device flush accounting. The underlying driver/device/scheduling trigger remains unresolved. No product/host fix was verified; this failed result is unchanged.

**2026-10-14 reconciliation:** separately authorized [native Windows and Azure Ubuntu full synchronized runs](YT-074-CROSS-ENVIRONMENT-VERIFICATION.md) subsequently passed the original gates in their recorded environments. This document remains the original Ubuntu FAIL record; its missing soak/final SQL observations are not filled from another host. Current authority is documentation/static review only, with no load repeat, commit or push.

## Authority and fixed execution

The owner explicitly authorized one complete synchronized test with the original thresholds, 0.5 requests/second per client, SQLite FULL, fsync records and every slow sample. The owner excluded a mixed-load repeat and prohibited commit/push. This is one execution, with no automatic retry or replacement. Existing measured gate failures and actual HTTP/SSE errors stop the execution; incomplete phases must be reported as not run.

Candidate HEAD is `e2eb2b50153a92de348cbe3a7573f4a602818ebb` in the current Ubuntu `master` checkout with the preserved uncommitted diagnostic/report delta. Release binary SHA-256 is `dc2d6d16e310ed5df2b118251546fd3d2e5a7b0656862e58032c7813406f4228`; canonical helper SHA-256 is `387e4daa34771ee242e38dba4d35cb1372246f4bf4d69ea329a6865ba995df08`. Only the existing helper gained a full synchronized profile and accurate trace-scope metadata. Application, security, transaction/history/CAS/capacity, assets and storage settings were not modified.

```sh
python3 -u examples/lab-operations-board/qualify_local.py --profile full-sync
```

Configuration: fresh private SQLite database on the existing ext4/NVMe repository mount; 10,000 items and 90,000 initial history rows; 25 clients, all starting at cycle position zero; the same four-read/one-update cycle at 0.5 requests/second; ten SSE readers; 120-second warmup, three 600-second rounds and 1,800-second soak. Full completion would make 45,000 measured data requests and 46,500 including warmup, with 9,300 successful writes and 99,300 final history rows below the unchanged 100,000 cap. Authentication requests are separate from the data mix; the normal Session lifetime and explicit re-login remain unchanged.

Thresholds: read p95 <250 ms, write p95 <500 ms, combined p99 <1,000 ms and unexpected responses <0.1%. Actual HTTP/SSE errors stop the run. Warmup latency is retained but advisory. All raw samples, including tails, are retained. Existing bounded fault evidence is reused rather than injected into timings.

Installed strace 6.19 records only owned-application `fsync`/`fdatasync` calls with `-D -f --seccomp-bpf -qq -ttt -T -yy -e trace=fsync,fdatasync -e signal=none`. No HTTP payload, credentials or Cookies are traced. Phase wall timestamps permit bounded trace association. This is instrumented execution; tracing may affect latency and the result is not relabeled as uninstrumented qualification. Ordinary workstation/Codex activity is not excluded. No per-request wall-clock timestamp, SQL queue-depth instrumentation, maximum-capacity result or leak-freedom proof is added.

Console output is retained in `target/m11-board/full-sync-20261013-execution.log`; the unique output directory is `target/m11-board/full-sync-20261013T074209233775Z/`. The helper reached terminal exit 1 with `status=fail` and `failure_type=AssertionError` at the measured gate assertion. No retry, replacement, mixed-load repeat or further execution followed.

## Actual duration and complete latency results

The run started at `2026-10-13T07:42:09.233964Z` and finished at `2026-10-13T08:14:10.126090Z`: Taiwan time **15:42:09.234–16:14:10.126**, overall 1,920.892 seconds. Completed phase wall times total 1,920.280 seconds; initialization/cleanup account for the remainder. This is a failed full-profile attempt, not a completed 62-minute test.

| Phase | Actual seconds including login | Data requests | Read p95, ms | Write p95, ms | Combined p99, ms | Gate |
|---|---:|---:|---:|---:|---:|---|
| Warmup | 120.048 | 1,500 | 32.929 | 76.481 | 76.481 | Advisory |
| Round 1 | 600.076 | 7,500 | 31.719 | 76.202 | 76.202 | PASS |
| Round 2 | 600.075 | 7,500 | 33.035 | 115.936 | 115.936 | PASS |
| Round 3 | 600.081 | 7,500 | 31.329 | **548.889** | 548.889 | **FAIL: write p95** |
| Soak | Not run | Not measured | Not measured | Not measured | Not measured | Not evaluated |

Each measured round contains exactly 6,000 reads and 1,500 successful writes, with zero unexpected responses and no worker errors. Achieved measured rates are 12.498 requests/second, preserving the selected pace and exact 80/20 mix. Total measured data requests are **22,500: 18,000 reads and 4,500 writes**. Including warmup, 24,000 contain 19,200 reads and 4,800 successful writes. The four phases recorded 100 logins/300 authentication requests, excluded from that mix; SSE authentication is separately recorded. No slow samples were dropped or treated as errors.

| Phase | Read p50 / p99 / max, ms | Write p50 / p99 / max, ms | Writes >=500 ms |
|---|---|---|---:|
| Warmup | 18.180 / 35.977 / 39.605 | 42.481 / 83.177 / 91.705 | 0 / 300 |
| Round 1 | 17.641 / 35.066 / 40.729 | 41.491 / 81.503 / 86.125 | 0 / 1,500 |
| Round 2 | 18.060 / 37.066 / 45.746 | 44.549 / 331.361 / 377.163 | 0 / 1,500 |
| Round 3 | 17.433 / 34.775 / 40.574 | 48.432 / **574.480 / 585.186** | **183 / 1,500 (12.2%)** |

Round 3 read p95, combined p99 and response/error criteria still pass. The only measured threshold failure is write p95. No HTTP or SSE error triggered the stop. The original failed synchronized round's 548.299 ms write p95 remains distinct from this new 548.889 ms result. The shorter synchronized PASS did not establish behavior once the long storage stalls appeared.

## Complete fsync observation

The first trace covers isolated schema initialization; `fsync-start-2.trace` covers the timed application, including startup/shutdown calls outside phase windows. Counts below select only calls whose entry timestamps fall within the corresponding phase. All selected calls were successful `fsync`; no `fdatasync` occurred. The >=100 ms column is descriptive, not a new gate.

| Phase | Sync calls | Sync p50 / p95 / p99 / max, ms | Calls >=100 ms | Calls >=500 ms |
|---|---:|---|---:|---:|
| Warmup | 308 | 2.838 / 3.597 / 5.456 / 5.575 | 0 | 0 |
| Round 1 | 1,533 | 2.850 / 3.683 / 5.473 / 5.879 | 0 | 0 |
| Round 2 | 1,530 | 2.846 / 3.684 / 5.498 / 302.980 | 3 | 0 |
| Round 3 | 1,533 | 2.848 / 3.639 / 5.531 / **508.232** | 9 | **7** |

All >=100 ms calls targeted the WAL. Round 2 captured 114.475, 240.234 and 302.980 ms calls beginning at 368.058, 488.065 and 548.070 seconds. Round 3 captured calls at approximately 68, 128, 188, 248, 308, 368, 428, 488 and 548 seconds: 383.148, 443.140, 505.271, 505.984, 505.848, 505.904, 505.646, 507.711 and 508.232 ms. The third-round spacing is approximately 60 seconds; the storage/filesystem/device or background-work cause is still unknown.

The seven >=500 ms WAL calls had no main-database sync entry within one second of their entry timestamps. The earlier 383.148 and 443.140 ms WAL calls did have nearby main-database sync entries, approximately 416 and 448 ms later. This bounded observation does not rule out every checkpoint/storage mechanism or identify a root cause.

Grouping measured WAL call-entry timestamps using a gap greater than one second confirms the synchronized shape: each round has 60 groups, predominantly 25 calls, with extra WAL/main-database syncs beyond the successful write count. Round 1 and round 3 each have 49 groups of 25 and eleven of 27; round 2 has 49 of 25, nine of 27 and two of 26. These groups are descriptive rather than exact transaction boundaries.

The serial database connection and synchronized write waves, together with repeated long WAL syncs and the third-round slow-write distribution, support the previously observed queue-amplification explanation. Per-request wall-clock start times were not added, so this report does not claim a proven one-to-one syscall/request mapping. The same roughly half-second disk-sync stall remains reproducible; no backend or host-policy fix was made or verified by this run.

## Resources, integrity limits and owned cleanup

There are 1,919 resource observations. RSS ranges 10,848–17,612 KiB; first/last samples are 10,848/17,612 KiB. Threads remain 26, FDs 59, and every sampled active-SSE count is ten. Measured application CPU deltas are 16.45, 16.77 and 16.21 seconds, plus 3.37 seconds during warmup. These finite observations do not prove leak freedom.

Each of the ten SSE readers made seven connections and two explicit logins across normal stream lifetime and Session renewal. Stream errors are empty and final active streams zero. The gate failure precedes the helper's final database-count and integrity assertions, so **final database counts and `integrity_check` were not evaluated** in this attempt. The 4,800 successful responses imply an expected history count of 94,800 under the unchanged transaction contract, but that number is not relabeled as an observed SQL count. The private database was intentionally removed by existing failure cleanup.

The helper joined all monitor/SSE threads, reaped its application and removed private synthetic database/log files. A post-exit check found application/tracer PIDs 473310/473315 and 473355/473360 absent, port 56661 closed and no `owned-*` directory remaining. No task-owned app, tracer or uncertain execution remains. The helper was observed terminal exit 1; this was a measured envelope failure, not an operational or harness failure.

The raw report's normal end-of-run source-equality assertion was not reached after the gate failure. A separate post-exit check nevertheless matched **all 175 recorded source/build/asset/helper input hashes** and the release binary against current bytes. Every original JSON hash was verified, and counts, outcomes, p50/p95/p99 and maxima were independently recalculated from all four raw sample files. The raw report was not edited to insert these later observations.

## Retained artifacts and closeout

| Artifact under `target/m11-board/full-sync-20261013T074209233775Z/` | SHA-256 |
|---|---|
| `report.json` | `d81b280fa67694d1bca6dee7e3f6b6510d76a303f73d1c88ee2acfff5d604e45` |
| `warmup-samples.json` | `105d707605304320a0cda493364860f4be3e6a315acc30a01ee9ce1c7fdf2373` |
| `round-1-samples.json` | `4eaba2ccc8c8907a567f451be2e273caa1a57269e498bfb6321bee5a8c4eac82` |
| `round-2-samples.json` | `c6e1b91b314aa0238765f8c47906ccc2682a91d11db82e7c2f20d5ccc19dd6b4` |
| `round-3-samples.json` | `c8ea93e7ce7053199e98bdfac28ed2f8d95bb39bcaa5f851ddb52d482c4a53ee` |
| `fsync-start-1.trace` | `b1f638b181e1eca74bd057c12ff7573bc836857cc623ac0b0fa09f8df996c852` |
| `fsync-start-2.trace` | `94f3abb1f85877eb0df024020312452937e9c9e39b90b1e1e9ff6c8626b1b3fb` |
| Original `SHA256SUMS` | `d4d175e5e054ed9a003344b1ea500af9c3d5b6f72db08ce9848f205481ebe73a` |

Console log `target/m11-board/full-sync-20261013-execution.log` has SHA-256 `de0de640783293d1a6afeefbf1daea0555d1c8b5dd8341591bc4e0f6fbca5f12`. Derived `target/m11-board/full-sync-20261013-summary.json` retains the recomputed metrics, all slow-sync timestamps, resource summaries and post-exit verification, SHA-256 `c0833abfadc08b596acc166d1f0f7d3ef43991de9fc9a9f26e835197c028a650`. Combined `target/m11-board/full-sync-20261013-SHA256SUMS` covers ten entries, including original artifacts, console log and summary, SHA-256 `2a2b7da89062082580ef6d7a276db0420f56dc4339db57d6cd0498f539c16068`. All raw samples and trace files remain under ignored local `target/`; no historical evidence was overwritten.

Pre-execution Python compilation, updated CLI help and whitespace checks passed. The full profile itself failed the write-p95 gate as reported. `sha256sum --check --status target/m11-board/full-sync-20261013-SHA256SUMS` passed for all ten entries. `python3 scripts/check_docs.py` passed its 75 required entries, English checks across all Markdown and 81-item DAG; the existing checker also passed with the four changed report/handoff paths appended to `REQUIRED` in memory (79 entries), covering their links, fences, EOF and whitespace without changing the checker. `git diff --check` passed. HEAD remains `e2eb2b50153a92de348cbe3a7573f4a602818ebb` and the staged index is empty. No native/browser/build repetition was needed for this helper-only execution; the existing application binary was reused unchanged.

Current uncommitted delivery paths are `examples/lab-operations-board/qualify_local.py`, this report, `docs/YT-074-SYNC-COMPARISON.md`, `docs/YT-074-QUALIFICATION.md` and `docs/M11-P-IMPLEMENTATION-HANDOFF.md`. The paired-comparison record preserves its earlier executed helper hash; this later profile does not relabel those inputs. Nothing is staged, committed or pushed by this continuation.

## Result boundary and next decision

On the original Ubuntu workstation, synchronized bursts remain unqualified against the original write-p95 requirement, and the full 62-minute operating envelope was not completed. The later passing environments are recorded separately above. The owner-amended mixed-load PASS and YT-074 Done status at that selected scope remain valid; no mixed-load repeat or broader acceptance change was performed. The renewed one-run authorization has been consumed.

Repeating the same long test without a concrete change would not verify a fix. A separately authorized repair should first investigate the observed roughly 60-second WAL-sync stall pattern and its storage/host cause, or evaluate the smallest behavior-preserving way to address the proven serialized write-queue effect while retaining FULL durability and transaction semantics. This record does not authorize that work or another execution. No application/host repair, GPU, release or deployment action follows automatically. Dependency-ready YT-075 requires separate GPU authority; YT-081 still depends on YT-080 and YT-020 remains Blocked.
