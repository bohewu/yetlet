# YT-074 — Local Board Qualification

**Status (2026-10-13):** Local automated qualification PASS. YT-074 remains **Blocked for full acceptance**: real OS IME/spoken AT and the broader proposed duration/target envelope are not verified or separately waived. YT-073 is Done at the selected loopback application scope.
**Continuation authority:** The owner requested continuing the non-GPU implementation through YT-074, retaining actual screenshots and startup instructions. No GPU, deployment, credentials for an external IdP, commit or push is requested by this continuation.

## Selected measurement and claim boundary

Exercise the existing loopback-only, single-process SQLite/Memory application with synthetic data. The original research's durations and numeric operating envelope were proposed assumptions, not executed evidence or a universal production guarantee. The sustained local check selects 10,000 records and 90,000 initial history rows, preserving write headroom under the unchanged 100,000 history cap. Exact item/history cap rejection is a separate native negative test, not successful write throughput.

Run 25 authenticated clients, each paced to at most two requests per second, with four reads then one update (80/20), and ten independently read/reconnected SSE connections. Warm up for 30 seconds; measure three 60-second rounds and a 300-second soak. Record actual elapsed time, request counts/mix, p50/p95/p99, unexpected responses, CPU, RSS/thread/FD observations and cleanup. The short prior 1,000-record rounds remain distinct. Compare measured read p95 with 250 ms, write p95 with 500 ms and combined p99 with 1 second; report failures rather than changing the thresholds. This establishes only this bounded workload, not the original suggested 62-minute experiment, maximum throughput, real-user concurrency, SLA, HA or leak freedom.

Failure evidence must cover write contention, read-only/unavailable storage, an acknowledged-to-storage but unread HTTP response without replay, authoritative conflict handling, process restart with Session retirement, bounded graceful shutdown and rejection of an incompatible storage candidate followed by recovery of the prior known-good version-1 database. The latter is a local rollback rehearsal with the same source/binary, not cross-version migration or production deployment rollback.

Chrome functional and rendered evidence, Firefox native keyboard/Unicode/form behavior and responsive layout are separate lanes. Actual OS IME and spoken screen-reader evidence remain **Not verified** unless an admitted real-device/tool surface supplies them. Headless Unicode typing, DOM accessibility structure and synthetic composition do not close those cells. No independent or formal assurance is supplied by the owner-permitted current-session self-review. Final acceptance must state these limitations and cannot certify the original broader proposal from these bounded results.

## Results

### Exact execution

Sustained/fault Job `wc_job_dyJlYei47wcEM5Dq` was observed terminal **completed, exit 0**. It ran `python3 examples/lab-operations-board/qualify_local.py`. Binary SHA-256: `085011ee5c237aa141283a08fab15975e45385c7feb35dca16e9ad4a8ef00346`; baseline HEAD: `d96f7af1073b12eab1a547d57f0d1b4d97589d7c` plus the uncommitted implementation. The report contains per-input hashes, checked equal before/after execution. Later README/qualification updates are documentation-only deltas, not relabeled executed inputs or signed release evidence.

The retained report is `target/m11-board/sustained/report.json`, SHA-256 `f20c5e8a65002a41d71a76bcce7fac368b98bba2f2ba7c1825d17841e5640920`. Four raw sample files remain beside it. Environment: Linux x86_64, kernel `7.0.0-38-generic`, 20 logical CPUs, Rust 1.99.0. This is a same-host loopback measurement, not a WAN or production deployment result.

### Sustained results

| Phase | Seconds | Requests | Read p95 (ms) | Write p95 (ms) | Write p99 (ms) | Combined p99 (ms) |
|---|---:|---:|---:|---:|---:|---:|
| Round 1 | 60 | 3,000 | 31.940 | 80.301 | 561.024 | 80.939 |
| Round 2 | 60 | 3,000 | 31.568 | 76.292 | 80.806 | 76.292 |
| Round 3 | 60 | 3,000 | 30.892 | 76.811 | 82.049 | 76.811 |
| Soak | 300 | 15,000 | 31.162 | 76.286 | 81.150 | 76.286 |

All predeclared local thresholds passed. The measured 24,000 requests contain 19,200 reads and 4,800 writes, with zero unexpected responses. A separate 30-second warmup is excluded from these counts. Achieved rate was approximately 49.97-49.99 requests/second, intentionally paced; it is not maximum throughput. Round 1 has a visible write-tail outlier: p99 561.024 ms and maximum 587.620 ms. Do not hide that by reporting only the better soak or combined percentile.

Data ended at 10,000 work items and 95,102 history rows. The independent native test fills the exact 10,000-item/100,000-history caps and verifies rejection without partial mutation; the load does not count capacity rejection as a successful write.

Resource observation contains 510 one-second samples: RSS ranged 10,284-15,360 KiB, thread count stayed 25 and file descriptors stayed 50. At the start/end of the measured phases RSS was 13,120/15,312 KiB; the 300-second soak consumed 21.77 process CPU seconds and RSS changed from 14,860 to 15,312 KiB. These bounded trends do not prove leak freedom or an indefinitely flat memory footprint. Internal SQL queue depth was not separately instrumented; bounded admission/cancellation is covered by native tests, not a fabricated queue metric.

### Failure, browser and cleanup evidence

- Actual external SQLite write-lock contention produced 503 without a record/history mutation; stateless health stayed 200 and an explicit later write succeeded.
- A real HTTP write committed while its response remained unread. Authoritative storage/read resolved that exact outcome; an explicitly submitted stale version returned 409. No automatic mutation replay was used.
- An incompatible schema candidate refused process startup. Selecting the known-good version-1 backup restored the latest confirmed data; the old Memory Cookie was rejected and a fresh login worked. Same source/binary and schema-compatible recovery only; no cross-version migration or deployment-platform guarantee.
- Ten SSE readers remained active through the measured workload. Each established two connections, including renewal after the five-minute server lifetime; no stream error was recorded. Graceful application exit took 0.0032 seconds in this run; all reader/monitor threads joined and the listener closed.
- Fourteen native tests passed with zero ignored/filtered tests; strict Clippy and formatting passed. New targeted coverage exercises actual `query_only` rejection, unavailable schema/readiness versus liveness, recovery and exact capacity boundaries. These are controlled native fixtures, not simulated evidence of a real disk disaster.
- The repaired Chrome 155.0.8059.39 journey passed login/create/edit/409 retention/status/reconnect, strict CSP/no-external-asset checks, responsive layout, process restart and three earlier short 1,000-record rounds. Its one intentional 409 network-console message is classified only at the already asserted negative step; unrelated errors still fail.
- A separate Firefox 157.0.1 journey passed native login, Tab/Unicode input, plain-text persistence/history, error associations/draft retention/correction and actual status updates. It does not assert that Unicode typing was an OS IME session or that labels were spoken by a screen reader.
- Five fresh Board screenshots were exported through the admitted artifact surface and visually inspected. See [gallery](screenshots/m11-board/README.md). The prior denied form-proof image was not retrieved by another path.

All owned application/browser/driver processes, listener ports, threads and private synthetic databases/logs/profiles were cleaned. No application is deliberately left running. No role-directed agent was dispatched; review is owner-permitted **current-session self-review**, not independent assurance.

### Remaining acceptance

Real OS IME candidate selection/composition and spoken assistive technology remain Not verified. Read-only Computer discovery exposed Windows targets but no Computer target for this Ubuntu workspace; installed local packages alone do not establish a usable, admitted real-device journey. No unrelated Windows application, remote production service or alternate UI backend was launched for this gap.

The original research's two-minute warmup, three ten-minute rounds and thirty-minute soak were not run: the exact shorter experiment above is retained rather than renamed as the longer one. Edge/Safari/mobile device, real IdP, production rollout, cross-version migration, independent security assurance and public package qualification are also unverified or outside this local implementation. Full YT-074 completion requires the remaining selected acceptance evidence or an explicit owner scope amendment; this session does not grant that amendment to itself.

No rerun of the now-passing automated journeys is needed merely to restate these results. [Startup guide](BOARD-START.md), [historical failure record](M11-P-BOARD-VERIFICATION.md) and [resumable handoff](M11-P-IMPLEMENTATION-HANDOFF.md) preserve the boundaries.
