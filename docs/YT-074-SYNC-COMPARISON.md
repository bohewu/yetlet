# YT-074 — Synchronized and Staggered 12-Minute Comparison

**Date:** 2026-10-13. **Result:** both authorized diagnostics completed, terminal exit 0, with all original measured thresholds passing. The original synchronized full-run failure remains a failure. This comparison does not extend the accepted operating envelope or replace the passing owner-amended [mixed-load qualification](YT-074-QUALIFICATION.md).

**Later follow-up:** the owner separately authorized the [full synchronized attempt](YT-074-SYNC-FULL-QUALIFICATION.md). It failed round 3 write p95 at 548.889 ms after approximately 32 minutes, with soak not run under the original stop rule. This later execution preserves the short results and confirms that their PASS did not establish the full synchronized operating envelope.

**2026-10-14 reconciliation:** the [three-environment record](YT-074-CROSS-ENVIRONMENT-VERIFICATION.md) adds completed native Windows and Azure Ubuntu synchronized PASS results while preserving both short checks, the original Ubuntu FAIL, original thresholds and instrumentation limits. The follow-up proposal below is historical; no new load execution is authorized by this documentation closeout.

## Authority and executed candidate

After local delivery commit `e2eb2b50153a92de348cbe3a7573f4a602818ebb`, the owner explicitly authorized two sequential 12-minute runs, synchronized first and staggered second, retaining the original thresholds and fsync records. The owner asked for a decision about a later full-duration test and prohibited another commit or push. Only those two load runs were executed.

Both runs used the same existing release binary, SHA-256 `dc2d6d16e310ed5df2b118251546fd3d2e5a7b0656862e58032c7813406f4228`. The only program delta is the uncommitted existing qualification helper: two 120-second-warmup/600-second-round profiles, no soak, application-only fsync tracing, phase timestamps and owned tracer cleanup. Application Rust, JavaScript, database implementation and storage settings did not change. Helper SHA-256: `38c61458f78ef55c73d99ea52ac01b358656c36d772e4cf909a07a45c2d19f99`.

Commands:

```sh
python3 -u examples/lab-operations-board/qualify_local.py --profile sync-12m
python3 -u examples/lab-operations-board/qualify_local.py --profile staggered-12m
```

Each run created and removed its own private synthetic database on the existing ext4/NVMe repository mount. Both began with 10,000 items and 90,000 history rows, 25 clients at 0.5 requests/second each, ten SSE readers and the same four-read/one-update cycle. The synchronized profile starts all clients at cycle position zero; the staggered profile starts them at `index % 5`. Those cycle positions are the only workload-configuration difference. SQLite FULL, transaction/history/CAS behavior and the 100,000-history cap remain unchanged.

Measured thresholds remain read p95 <250 ms, write p95 <500 ms, combined p99 <1,000 ms and unexpected responses <0.1%. Any actual HTTP/SSE error stops execution. Warmup latency is retained but advisory. Authentication requests remain outside the 80/20 data mix. Existing bounded fault evidence was not repeated.

| Run | Taiwan time, UTC+08 | Actual warmup seconds | Actual measured seconds | Overall seconds, including setup/cleanup |
|---|---|---:|---:|---:|
| Synchronized | 13:23:37.211–13:35:38.013 | 120.045 | 600.079 | 720.802 |
| Staggered | 13:35:50.781–13:47:51.580 | 120.049 | 600.084 | 720.799 |

## Complete latency results

Each measured round contains exactly 7,500 data requests: 6,000 reads and 1,500 successful writes. Each warmup contains 1,500: 1,200 reads and 300 writes. Both measured achieved rates were 12.498 requests/second. Each run therefore made 9,000 data requests including warmup, with zero unexpected responses, worker errors or SSE errors. The two phases separately recorded 50 logins/150 authentication requests; SSE login accounting remains in the raw report. No slow request was discarded.

| Phase | Read p50 / p95 / p99 / max, ms | Write p50 / p95 / p99 / max, ms | Combined p99 / max, ms | Writes >=500 ms |
|---|---|---|---|---:|
| Synchronized warmup | 18.185 / 33.171 / 37.264 / 44.086 | 42.503 / 76.413 / 84.496 / 90.446 | 76.413 / 90.446 | 0 / 300 |
| Synchronized measured | 17.310 / 30.983 / 34.924 / 42.326 | 41.598 / **76.133** / 80.894 / 85.925 | 76.133 / 85.925 | 0 / 1,500 |
| Staggered warmup | 24.064 / 43.705 / 526.436 / 543.117 | 28.590 / 46.687 / 534.064 / 544.209 | 530.766 / 544.209 | 10 / 300 |
| Staggered measured | 23.393 / 43.646 / 360.510 / 544.272 | 28.372 / **45.240** / 356.627 / 542.924 | 360.510 / 544.272 | 8 / 1,500 |

The staggered measured round also retains 27 reads >=500 ms, yielding 35 such requests overall. Its eight slow writes are 0.533% of writes; this does not violate a write-p95 gate. The p99 and maximum columns preserve the tails separately. The original failed synchronized round had write p95 548.299 ms and 175/1,500 writes >=500 ms; this new short PASS does not overwrite it.

## Actual fsync records

Installed strace 6.19 traced each owned application with:

```text
strace -D -f --seccomp-bpf -qq -ttt -T -yy -e trace=fsync,fdatasync -e signal=none -o TRACE BINARY
```

The detached tracer preserves the application's direct parent and PID for the existing metrics/shutdown code. Seccomp filtering limits syscall tracing to disk-sync calls. Raw records retain syscall-entry Unix timestamps, durations, thread IDs, file descriptors and file paths. No HTTP data, login codes, Cookies or CSRF values were traced. A single owned-file capability probe succeeded before the load runs; its temporary probe file was removed. Both comparison runs were instrumented identically; tracing can affect timings, so these are instrumented diagnostics rather than uninstrumented qualification measurements.

Only calls whose entry timestamps fall within the corresponding phase window are counted below. All recorded calls were successful `fsync`; no `fdatasync` occurred. The >=100 ms column is descriptive and is not a new acceptance threshold.

| Phase | Sync calls | Sync p50 / p95 / p99 / max, ms | Calls >=100 ms | Calls >=500 ms |
|---|---:|---|---:|---:|
| Synchronized warmup | 308 | 2.848 / 3.556 / 5.500 / 5.775 | 0 | 0 |
| Synchronized measured | 1,530 | 2.852 / 3.473 / 5.480 / **5.878** | 0 | 0 |
| Staggered warmup | 308 | 3.132 / 5.489 / 5.730 / **505.682** | 2 | 2 |
| Staggered measured | 1,533 | 3.127 / 5.462 / 5.690 / **505.907** | 5 | 1 |

All seven >=100 ms calls in the staggered run targeted the WAL. Warmup calls began at 22.048 and 82.059 seconds, lasting 505.682 and 505.570 ms. Measured calls began at 324.066, 384.068, 444.078, 504.074 and 564.079 seconds, lasting 250.315, 327.257, 398.614, 477.349 and 505.907 ms. The roughly 60-second intervals are observed, but their underlying cause is unknown. There was no main-database sync entry within one second of any of those seven call-entry timestamps; this is a bounded trace observation, not exclusion of every checkpoint or storage mechanism.

The trace also confirms different burst shapes. Grouping measured WAL call-entry timestamps with a gap greater than one second gives 60 synchronized groups: 50 groups of 25 and ten of 27. The staggered run gives 300 groups: 286 of five, six of six and eight of seven. The raw trace records additional WAL and main-database syncs beyond successful write counts; these groups are descriptive and do not establish exact transaction boundaries.

The synchronized run experienced uniformly short sync calls, while the staggered run experienced much longer tails. The phases were sequential, not randomized or simultaneous, and ordinary workstation/Codex activity was not excluded. Thus the difference between 76.133 and 45.240 ms cannot be attributed solely to scheduling under identical disk conditions. Raw request samples contain kind, duration and response status; no per-request wall-clock start timestamp was added. Sync/request association is at phase scope rather than a proven one-to-one mapping.

## Integrity, resources and owned cleanup

Both runs observed exactly 10,000 items and 91,800 final history rows, matching 90,000 initial rows plus 1,800 successful writes, and passed the existing SQLite `integrity_check=ok` assertion before removing the private database. Neither reached the unchanged history cap.

| Observation | Synchronized | Staggered |
|---|---:|---:|
| Resource samples | 719 | 719 |
| RSS range, KiB | 11,024–17,828 | 12,184–15,872 |
| First / last RSS, KiB | 11,024 / 17,828 | 12,184 / 15,812 |
| Threads | 25 | 26 |
| FDs | 34–59 | 59 |
| Sampled active SSE | 10 | 10 |
| Connections per SSE reader | 3 | 3 |
| Warmup / measured CPU seconds | 3.35 / 16.17 | 3.32 / 16.32 |
| Graceful shutdown seconds | 0.003253 | 0.003276 |

Each reader reconnected across the normal five-minute stream lifetime; all stream errors were empty and final active streams zero. These observations do not prove leak freedom; SQL queue depth remains Not instrumented.

Both helpers reached terminal exit 0, joined all monitor/SSE threads and removed their private database/log directories. A post-exit check found synchronized application/tracer PIDs 336278/336283 and 336310/336315 absent, and staggered PIDs 348932/348937 and 348964/348969 absent. Ports 49109 and 46511 were closed. No task-owned app/tracer or uncertain execution remains.

All 175 source/build/asset/helper hashes and the release binary matched before/after each run, between both runs and in the post-exit comparison. Every original JSON artifact hash was verified, and every latency count, outcome, p50/p95/p99 and maximum was recalculated from all four raw sample files. The two envelopes match after removing their cycle-position/description fields.

## Retained artifacts and verification

Local synchronized directory: `target/m11-board/sync-12m-20261013T052337210368Z/`. Staggered directory: `target/m11-board/staggered-12m-20261013T053550780636Z/`. Each retains `report.json`, `warmup-samples.json`, `round-1-samples.json`, original `SHA256SUMS`, `fsync-start-1.trace` for initialization and `fsync-start-2.trace` for the timed application. Raw reports were not edited after execution.

| Artifact | SHA-256 |
|---|---|
| Synchronized `report.json` | `d03b25aa19d7a29f62abdaa5b8fd09bb11ed54496a21c8f7334b581011cfa66c` |
| Synchronized `fsync-start-1.trace` | `d35f84014dca3a0f9484db9926499e9ecd3a356373219f58587b4578cf952a49` |
| Synchronized `fsync-start-2.trace` | `ec66ca95d965eeb1ca6f47afea0b4227135cda0592e01b21d56ccca755382edb` |
| Staggered `report.json` | `2e29df1f5c9fee78498449d6f55f7e7c05139d7cf966999af3fc2fc845f43e50` |
| Staggered `fsync-start-1.trace` | `48093e754d1b1935cec12ec51510cc57b37b327f45b151692f0ec247c05890d4` |
| Staggered `fsync-start-2.trace` | `f6380912150fd7d9d93c27f0ee589a86c919b8f6474b97844264a87bb587eb47` |

Execution logs are `target/m11-board/sync-12m-20261013-execution.log` and `target/m11-board/staggered-12m-20261013-execution.log`. Derived `target/m11-board/sync-comparison-20261013-summary.json` contains all independently recalculated metrics, slow-sync event timestamps, resource summaries and artifact hashes; SHA-256 `2bcbaee72dcf1327c6e0617a3060344d42bb7f5dd10e43acb627c126e43b1090`. Combined `target/m11-board/sync-comparison-20261013-SHA256SUMS` contains 15 entries including both logs, all original artifacts and the summary; SHA-256 `b379c40c238bff9149429600dd9437d43a550d74a0233b19d9a5fa64922fb572`.

`python3 -m py_compile examples/lab-operations-board/qualify_local.py`, the updated CLI help, both actual profiles and `sha256sum --check --status target/m11-board/sync-comparison-20261013-SHA256SUMS` passed. `python3 scripts/check_docs.py` passed its 75 required entries, English checks across all Markdown and 81-item DAG. The existing checker also passed with the three changed report/handoff paths appended to `REQUIRED` in memory (78 entries), covering their links, fences, EOF and whitespace without changing the checker. `git diff --check` passed. The existing release binary was reused; no rebuild or browser/native-suite repetition was needed for this helper-only diagnostic.

Changed delivery paths are `examples/lab-operations-board/qualify_local.py`, this report, `docs/YT-074-QUALIFICATION.md` and `docs/M11-P-IMPLEMENTATION-HANDOFF.md`. Raw local artifacts remain ignored. Nothing is staged, committed or pushed by this continuation.

## Decision about a full-duration follow-up

**A complete synchronized run is worthwhile if the owner wants to qualify synchronized bursts as an additional operating envelope.** The short synchronized PASS establishes feasibility under short disk-sync latency, but it did not encounter the long stalls that remain present in the other run and previously caused the synchronized write-p95 failure. Its twelve minutes cannot establish the original 62-minute envelope or invalidate that failure. A single longer PASS would still need to be reported alongside the retained FAIL and intermittent-storage limitation rather than called proof of a root-cause fix.

The narrow proposed next execution is one synchronized 62-minute run at the same 25 clients, 0.5 requests/second, ten SSE readers, initial data, SQLite FULL and original thresholds, retaining fsync records and complete raw samples. The already passing full mixed-load test need not be repeated. This is additional qualification, not a new blocker for YT-074's already accepted mixed-load scope. No long execution was started; only the two short diagnostics were authorized.

YT-074 stays Done at the owner-amended mixed-load scope. No application repair, storage-policy change, GPU or release activation follows from this comparison. Dependency-ready YT-075 still requires separate GPU authority; YT-081 still depends on YT-080 and YT-020 remains Blocked.
