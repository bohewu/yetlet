# YT-074 — Cross-Environment Synchronized Verification

**Date:** 2026-10-14. **Recorded outcomes: original Ubuntu FAIL; native Windows PASS; Azure Ubuntu PASS.** This reconciles completed evidence without a new load execution. The owner requested documentation closeout and review of the uncommitted delta and SSE startup correction, with no commit or push.

YT-074 remains Done at its previously approved [local mixed-load scope](YT-074-QUALIFICATION.md). Windows and Azure add passing synchronized-profile observations in their recorded environments. They do not change the original host's failed verdict, establish a repair there, or expand browser, GPU, release or production acceptance. The earlier independent review covers the mixed-load delivery; this reconciliation is current-agent offline/static review, not a new independent review.

## Preserved profile and actual observations

The configured synchronized profile is 10,000 items, 90,000 initial history rows, 25 HTTP clients at 0.5 data requests/second each, four reads then one write with every client starting at cycle position zero, and ten SSE readers. Duration is 120 seconds warmup, three 600-second rounds and 1,800 seconds soak. Authentication traffic is excluded from the data mix. The selected application retains SQLite WAL/FULL; Windows has source verification but no separate returned runtime PRAGMA inspection. No automatic mutation retry or durability relaxation was introduced.

Original gates remain **read p95 <250 ms, write p95 <500 ms, combined p99 <1,000 ms and unexpected-response fraction <0.001**. Actual HTTP/SSE errors remain stop conditions. Warmup latency is advisory; the original Linux measured-gate stop rule is retained. No new request-start-spread or fsync threshold was added.

| Environment | Native outcome / overall wall time | Actual formal reads / writes | Write p95 ms: rounds 1 / 2 / 3 / soak | HTTP/SSE errors |
| --- | --- | ---: | --- | --- |
| Original Ubuntu workstation, 2026-10-13 | **FAIL, exit 1**, 1,920.892 s; soak **Not run** | 18,000 / 4,500 = **22,500** | 76.202 / 115.936 / **548.889** / Not measured | 0 |
| Native Windows, 2026-10-14 | **PASS, exit 0**, 3,731.509 s; every configured phase complete | 35,950 / 8,900 = **44,850** | 7.6731 / 8.2077 / 7.9527 / 7.4721 | 0 |
| Azure Ubuntu, 2026-10-14 | **PASS, exit 0**, 3,722.086 s; every configured phase complete | 36,000 / 9,000 = **45,000** | 137.822 / 134.017 / 138.624 / 126.480 | 0 |

The original failure's exact round-3 write p95 is **548.88905 ms**, above the unchanged 500 ms limit; 183/1,500 round-3 writes were at least 500 ms. The aborted soak and final database checks are not supplied from another run. See the [original complete record](YT-074-SYNC-FULL-QUALIFICATION.md).

Including warmup, original Ubuntu recorded 24,000 data requests, Windows **46,350** (37,150 reads / 9,200 writes), and Azure **46,500** (37,200 reads / 9,300 writes). Windows' observed mix is 80.151025% reads / 19.848975% writes at 12.448845 requests/second across phase wall time; Azure's is exactly 80/20 at 12.496783. Nominal counts are not substituted for the observed Windows counts. Windows' maximum HTTP latency was 78.7224 ms; Azure's was 208.192032 ms, both including warmup. All samples and original percentiles remain retained.

Successful final fixture counts were Windows 10,000 items / **99,200 history**, and Azure 10,000 / **99,300**, matching initial history plus successful writes and remaining below 100,000. Both helpers passed their existing integrity assertion before deleting private fixtures; no independent post-deletion SQL query is claimed. Original Ubuntu stopped before its final SQL checks, so its expected 94,800 history rows are **not an observed count**.

The original full mixed-load PASS remains separate: 45,000 formal requests, zero errors and 99,300 final history rows. The paired original-host 12-minute synchronized/staggered checks both remain PASS; neither replaces the failed original-host full profile. Earlier Windows Runner interruptions retain unknown native exits and incomplete phases. No partial results are merged into the completed native Windows run.

## Environment, identity and instrumentation boundaries

| Environment | Recorded system/storage | fsync evidence |
| --- | --- | --- |
| Original Ubuntu | Ubuntu 26.04.1, kernel 7.0.0-38-generic, i5-13600K; ext4 on Neo Forza NFP055PCI51-34H1000 NVMe | Full profile traced with strace 6.19; round-3 maximum **508.232 ms**. A separate untraced file probe reproduced three 505–508 ms stalls. |
| Native Windows | Windows 11 Pro, i5-13500; NTFS on WD Blue SN580 NVMe | **Not instrumented**: no WPR/ETW/fsync/FlushFileBuffers timing. HTTP latency is not disk-flush timing. |
| Azure Ubuntu | Ubuntu 24.04.4, kernel 6.17.0-1022-azure; Standard_D2as_v5, two vCPUs / 8 GiB; ext4 on Premium SSD P10 128 GiB, host caching None | strace 6.8; 9,512 successful fsync calls including initialization, 9,503 inside phase windows. Maximum timed fsync **45.535 ms**; none at least 100 ms. |

The original application binary is dc2d6d16e310ed5df2b118251546fd3d2e5a7b0656862e58032c7813406f4228. Windows and Azure use source snapshot f79ccde012a1642d77331d36cccabc98a5d83044, derived from e2eb2b50153a92de348cbe3a7573f4a602818ebb. Windows' recorded 166 source entries match the published snapshot; Azure's 293 package entries match its declared snapshot/helper overlay, including all 94 Rust files matching the local application. Platform builds are distinct: Windows binary 83a54ff96427bb8d33eb3a36390f866def67e86312f486bceae4c04abc2d3eab; Azure binary 3e4eb7fcf03eaac6dce1fc1a03e00548c1c7e01e13e0f6dd9ea6195ec8331023. There is no application-source repair between these observations.

Configured cycle cohorts are synchronized, but actual HTTP starts are not proven identical across environments. Windows' maximum within-burst start spread was **479.583 ms**; original Linux raw request rows lack comparable client/start timestamps. Fsync entry groups are a different boundary. Hardware, kernel, filesystem, VM scheduling, platform harness and tracing differ; Azure-versus-Windows latency is not a controlled OS or SSD benchmark. The Windows return corrects its inherited nonadministrator explanation: its local token was observed to be administrator after completion, while tracing remained Not instrumented.

## Practical attribution

The evidence supports classifying this stall as an **environment-specific system/storage-path limitation of the original Ubuntu workstation**. Its owned-file reproduction used no SQLite, HTTP, SSE, application or strace and observed journal waiting with matching device flush accounting. Windows and a second Ubuntu environment passed. The original stall therefore does not establish a general Ubuntu incompatibility or a framework-core defect requiring repair before continued development.

The exact device/firmware/driver/kernel/scheduling trigger remains unresolved; a physically defective SSD is not established. The sample application's serialized database connection still amplifies long commits across queued writes. No original-host repair or framework redesign was performed or verified. The [diagnosis](YT-074-FSYNC-DIAGNOSIS.md) preserves these limits. If host repair is selected later, start from its bounded file reproduction and missing event-level trace rather than automatically repeating a long qualification.

## Retained Azure startup failure and SSE correction

The first Azure short-profile startup, sync-12m-20261014T033705599943Z, exited 1 after **12.864464 seconds**, before warmup, with zero latency samples and no SSE logins/connections. It is a retained harness failure, not a measured load failure. A fresh VM's monotonic clock was below 1,200 seconds; initializing the SSE login timer to zero skipped authentication and Python rejected a None Cookie header.

The canonical helper's only subsequent correction is:

```diff
-            login_time = 0
+            login_time = None
 ...
-                    if time.monotonic() - login_time >= 1200:
+                    if login_time is None or time.monotonic() - login_time >= 1200:
```

This ensures login before the first SSE request and preserves the 1,200-second renewal interval, HTTP/SSE stop behavior, pace, thresholds, tracing and application binary. Old helper SHA-256 is 387e4daa34771ee242e38dba4d35cb1372246f4bf4d69ea329a6865ba995df08; corrected SHA-256 is **1ed36adc03bd42f1b7fb83139f2b74c5d3c75fb1342f7c7ad3eac3bd94e331db**. The Windows platform harness is separate and unchanged.

The corrected Azure 12-minute stage, sync-12m-20261014T034243843694Z, began at guest monotonic time **1,063.418 seconds**, exercising the fix before the old cutoff. It passed with 7,500 formal requests, write p95 135.827658 ms, zero errors and 91,800 final history rows. The subsequent full stage is one complete execution; neither corrected stage replaces the failed startup's files.

## Retained evidence and cleanup

Raw artifacts remain ignored under target/m11-board; the following identities keep this formal summary useful when those files are unavailable in another checkout.

| Artifact | SHA-256 |
| --- | --- |
| Original Ubuntu full synchronized report | d81b280fa67694d1bca6dee7e3f6b6510d76a303f73d1c88ee2acfff5d604e45 |
| Windows return ZIP, 637,130 bytes | 8f3b51f4ff27643c0ea7c2e5448fd894570517b16f939ca349739da43eaa4d4e |
| Windows full-sync/report.json inside ZIP | 3421cc41df53a740b62f3d984a1ec0e4db10e62620044a4f29f5b9afe841f756 |
| Azure returned TAR.GZ, 583,673 bytes | ff0d445feb047e647a20726ef29dc7ba16032a0392076878b80fdd6dad08c845 |
| Azure full synchronized report | 13e5c5453c54af899e10b7463f2118022e78d886b9fe7e5d27927b1d1cdb8973 |

Windows' local review is target/m11-board/windows-return-review-20261014/REVIEW.md, with verification.json and the unchanged RETURN-HANDOFF.md. ZIP CRC, the supplied ZIP hash, all 30 root-manifest entries and nested manifests were verified. All raw counts and latency statistics were recalculated during return review. The returned evidence records owned process/thread/listener/fixture cleanup; no new live Windows inspection was performed for this reconciliation.

Azure's local record is target/m11-board/azure-ubuntu-20261014T032202Z/RESULT.md. Download verification covers 60 regular archive files, 59 enclosing-manifest entries and ten nested JSON checks. OFFLINE-VERIFICATION.json retains raw-statistic recalculation; verified-return contains the original failure, corrected short run, full run and all fsync traces. Full native execution was **2026-10-14 03:55:44.269471–04:57:46.355058 UTC**. Windows native execution was **00:21:29.3283176–01:23:40.8372968 UTC** that day; these native brackets include setup/cleanup and are distinct from internal report/phase timestamps.

All dedicated Azure resources were verified absent after archive retrieval: rg-yt074-verify-20261014 and its seven resources at **05:10:03 UTC**. The automatically created, unused NetworkWatcher_southeastasia and NetworkWatcherRG were separately removed, with empty subscription resource and resource-group lists verified at **05:23:31 UTC**. DELETION-VERIFICATION.json and AUTOMATIC-NETWORK-CLEANUP.json retain independent GET/list results. Temporary task SSH keys were removed; subscription-wide Network Watcher auto-enablement was not changed. Later billing may still reflect earlier usage; no actual-billing claim is made.

## Closeout review and remaining scope

This closeout changes this record, [qualification](YT-074-QUALIFICATION.md), [short comparison](YT-074-SYNC-COMPARISON.md), [original-host full record](YT-074-SYNC-FULL-QUALIFICATION.md), [diagnosis](YT-074-FSYNC-DIAGNOSIS.md), [handoff](M11-P-IMPLEMENTATION-HANDOFF.md), [backlog](BACKLOG.md) and [roadmap](ROADMAP.md): eight Markdown files. The existing qualification helper remains uncommitted but byte-identical throughout this closeout; no new code edit was made.

Review of the entire uncommitted helper delta covers the previously authorized full-sync and paired short profiles, trace metadata/startup, phase timestamps, zero-soak handling, owned tracer cleanup and the two-line first-login correction. Original gates, FULL durability, data-cycle configurations and no-retry semantics are retained. No additional application/core/asset change is required by this review.

Network-free checks exercised the actual extracted SSE worker at monotonic clocks 0, 1, 719, 1199.999, 1200 and 50000; every first connection authenticated. A reconnect sequence at 1, 1200.999 and 1201 authenticated initially, reused the session just before 1,200 elapsed seconds and renewed exactly at the boundary. Syntax checks passed. No socket, workload, subprocess or sleep was needed for these checks, and the helper bytes were not changed during closeout.

Validation passed: python3 scripts/check_docs.py (75 required documents / 81 backlog items), and the same checker with all eight current uncommitted Markdown paths appended to REQUIRED in memory (81 distinct documents / 81 backlog items), covering report links/fences/EOF/whitespace without changing validation tooling. Original-host and Azure manifests and Windows ZIP CRC/hash/30 root entries verified; raw phase counts, statuses, write/read p95, combined p99, history arithmetic and report identities were rechecked offline. AST comparison with HEAD confirmed original bounded/full profile configurations and threshold expressions unchanged. All backlog states match HEAD, the staged index is empty, and git diff --check passes. Current-agent static review found no material issue requiring another correction. Existing native/build/browser/CSP/IME/AT evidence is retained separately; none was rerun for this documentation-only reconciliation.

YT-074 stays Done at the earlier owner-amended scope; YT-020 remains Blocked. Dependency-ready YT-075 still requires separate GPU activation, YT-076–YT-080 remain unactivated, and YT-081 retains its YT-080 dependency unless the owner changes scope. No new load, host tuning, external resource operation, staging, commit, push or release activation was part of that documentation/static closeout. The owner subsequently authorized one local delivery commit conditional on strict/builtin Guard success; the [handoff](M11-P-IMPLEMENTATION-HANDOFF.md) records that separate authority. Push remains prohibited.
