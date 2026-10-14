# YT-074 — Local Board Qualification

## Current reconciliation — 2026-10-14

**YT-074 remains Done at the owner-amended local mixed-load scope.** The [cross-environment synchronized record](YT-074-CROSS-ENVIRONMENT-VERIFICATION.md) now reconciles original Ubuntu **FAIL** (round-3 write p95 548.88905 ms; soak Not run), native Windows **PASS** (44,850 actual formal requests), and Azure Ubuntu **PASS** (45,000 actual formal requests). Both passing environments completed the full configured duration with zero HTTP/SSE errors and the original thresholds. Their results do not relabel the original-host failure or establish a repair there.

The practical diagnosis is an environment-specific system/storage-path limitation of the original Ubuntu workstation: a standalone untraced file probe reproduced the stall without SQLite or the application, and Azure Ubuntu did not reproduce it. The precise SSD/driver/kernel/scheduling trigger remains unresolved. Windows flush timing was Not instrumented; Azure's maximum timed fsync was 45.535 ms. No framework-core repair is established as necessary by this evidence.

Azure exposed a first-SSE-login harness bug on a newly booted host. The existing helper now uses an explicit unset timer and authenticates before the first connection, retaining 1,200-second renewal. The failed startup, corrected short PASS, full PASS, actual sample counts, source identities and verified Azure cleanup are recorded separately in the cross-environment record. Documentation and uncommitted-diff/SSE review completed without another load run; the [handoff](M11-P-IMPLEMENTATION-HANDOFF.md) records subsequent authority for one local commit after strict/builtin Guard success, with no push or load repeat. Historical sections below retain their original dates, scope and review boundaries.

## Historical repair and mixed-load qualification — 2026-10-13

**YT-074 is Done at the owner-amended local mixed-load scope.** Real OS Chewing IME and the repaired Firefox/Orca spoken journey pass. The selected Chrome/Firefox canonical journeys pass on the final binary. The owner-authorized mixed-load full run passed in terminal exit 0, and fresh independent review passed with no material findings. The original synchronized-burst failure remains an explicit limitation. The earlier independent review applies only to the historical blocked closeout below.

**Subsequent diagnostic follow-up:** after local commit `e2eb2b5`, the owner authorized two [synchronized/staggered 12-minute comparisons](YT-074-SYNC-COMPARISON.md) with original thresholds and fsync tracing, and prohibited another commit/push. Both completed PASS on the same binary; synchronized write p95 was 76.133 ms and staggered 45.240 ms. The staggered trace still captured a 505.907 ms measured WAL sync. The original synchronized full-run FAIL remains, and these short instrumented diagnostics do not extend the accepted operating envelope. A full synchronized run is recommended only for separately requested additional qualification; it was not started.

**Later authorized full synchronized attempt:** the owner then authorized one [full synchronized profile](YT-074-SYNC-FULL-QUALIFICATION.md) at the same 0.5 pace, FULL durability, original thresholds and fsync tracing. It exited **FAIL** after approximately 32 minutes: rounds 1/2 passed, round 3 write p95 was **548.889 ms**, and the 30-minute soak was **Not run** under the original stop rule. All 22,500 measured responses and SSE behavior were error-free, but 183/1,500 round-3 writes were >=500 ms. Seven round-3 WAL syncs lasted about 505–508 ms. Raw samples/traces and verified owned cleanup are retained. No replacement, product/host repair, mixed-load repeat, commit or push followed. The existing owner-amended mixed-load acceptance remains unchanged.

**Latest cause investigation:** the owner requested diagnosis and permitted the registered debugger. A [three-minute untraced private-file probe](YT-074-FSYNC-DIAGNOSIS.md) reproduced three 505–508 ms stalls, sampled journal-commit waiting and matching 503–505 ms whole-device flush accounting. The stall is localized below the application/SQLite boundary; its underlying driver/device/scheduling trigger is unresolved. Existing serial database admission explains amplification across synchronized write waves. No application/storage repair, load qualification repeat, commit or push was performed.

The executed candidate was `master` at `f00ec597e43f129a649fd1b8805fdd89edb93fbb`, ahead of `origin/master` by two commits, with uncommitted changes. The owner subsequently authorized a local delivery commit on 2026-10-13 and requested consideration of a synchronized-burst retest. Push and new load execution are not requested by that instruction. Final application binary SHA-256: `dc2d6d16e310ed5df2b118251546fd3d2e5a7b0656862e58032c7813406f4228`.

| Gate | Current evidence |
|---|---|
| A — real OS IME | **PASS**, retained real Chewing composition, exact storage, validation and conflict observations in [desktop acceptance](YT-074-DESKTOP-ACCEPTANCE.md). |
| B — spoken AT | **PASS**, actual Orca/Speech Dispatcher output for the required journey, including the repaired live notice and native linear caption/header/row reading. [Attributed transcript](YT-074-SPOKEN-AT-EVIDENCE.json). |
| C — selected browsers | **PASS**, Chrome 155.0.8059.39 and Firefox 157.0.1 on the final binary. The owner replaced mandatory Edge with this pair; Windows Edge remains unverified. |
| D — full duration | **PASS**, the single authorized 62-minute run at the explicitly amended five-cohort mixed workload; 45,000 measured requests, zero HTTP/SSE errors, original thresholds and SQLite FULL retained. |
| E — independent review | **PASS**, fresh ordinary read-only review of the repaired candidate and completed evidence; zero issues or required followups. |

### Measured bottleneck and explicit workload amendment

The failed synchronized workload remains a limitation. In the retained full attempt, every client started at the same four-read/one-write cycle position, producing bursts of 25 writes. Round 1 write p95 was 548.299 ms, failing the unchanged <500 ms limit. A separate 75-second diagnosis recorded an actual WAL `fsync` lasting 0.497406 seconds and the corresponding 25 requests at 499.379–582.061 ms. No main-database checkpoint sync occurred in that episode. This supports a storage-sync stall and serialized write queue explanation; it does not establish a particular SSD firmware, power-management or filesystem cause.

A bounded comparison of `fsync` and `fdatasync` on two owned files also showed sync tails. No host storage/power settings, SQLite FULL durability, transaction boundaries, history semantics, admission limits or hard caps were changed. The existing database stays on the repository's ext4/NVMe mount. Moving it to memory or accepting weaker durability was not used to meet the gate.

| Preserved diagnosis artifact under `target/m11-board/diagnosis-20261013/` | SHA-256 |
|---|---|
| `requests.json` (950 actual requests) | `000c8d55eaaebcdeb45e67608b5ef27c9e49e9444bbec9edc4af3ee1e46930d8` |
| `sync.trace` (sync syscalls only) | `85e0c49bce9abea40bf040da2664e254f6708438e155c5cac523627e7130cffd` |
| `file-sync-comparison.json` (900 sync calls) | `af93c2d5d6bc96d586493a3efef1467323a752da25e9180665fd678fceb1bd73` |

The owner explicitly accepted **new mixed-load qualification**: clients start at cycle positions `index % 5`. Each nominal two-second slot contains 20 reads and five writes; every client retains the same five-operation cycle at 0.5 requests/second. The original default `bounded` profile remains unchanged. This is a changed accepted traffic shape, not a claim that the original synchronized burst has been fixed or qualified.

The full profile preserves 10,000 items, 90,000 initial history rows, 25 clients, ten SSE readers, 120-second warmup, three 600-second rounds and 1,800-second soak. Thresholds remain read p95 <250 ms, write p95 <500 ms, combined p99 <1,000 ms, unexpected responses <0.1%; actual HTTP/SSE errors stop execution. Warmup latency is recorded but not a steady-state gate. Explicit Session re-login avoids changing the normal 30-minute lifetime. Authentication requests are counted separately from the 80/20 data workload.

A single admitted short probe passed before the new full execution: 30-second warmup, one 120-second round and a 180-second soak with the same 25 clients, 0.5 pace and ten SSE readers. Round read/write p95 were 43.016/43.548 ms; soak 42.989/44.396 ms. Write maxima remained 543.522/544.019 ms and combined p99 519.706/532.528 ms. Zero unexpected responses; final history 90,825; integrity and owned cleanup passed. This 330-second probe is not substituted for the full experiment. Raw report: `target/m11-board/mixed-probe-20261013T031957445041Z/report.json`, SHA-256 `9c557ba1f14275f0886dd708bacff06bcd4c40bbcf6693605c46a9595e11ec58`.

### Application repair and targeted regression

Only the optional live-status presentation changed: retain the semantic status container, expose its message as an explicit polite/atomic generic live child for Firefox/Orca, write the changed message before adjacent counts, and avoid unchanged text mutations. [Desktop acceptance](YT-074-DESKTOP-ACCEPTANCE.md#live-status-compatibility-repair-and-its-limits) records the failed intermediate prototypes and successful actual final speech. The finite generated JavaScript asset, manifest/inventory and embedded asset reference were regenerated. Security, data and transport contracts remain unchanged.

The canonical Firefox journey now also observes that unchanged SSE snapshots do not rewrite either text node and that a persisted change mutates message before counts. Fourteen Board native tests passed with zero failures/ignored/filtered; Board format, strict Clippy, release build, asset check and Python compilation passed. Previously sufficient form/CSS checks remain retained because their source did not change. Chrome and Firefox canonical journeys were rerun because the application's live behavior materially changed.

| Final browser report | SHA-256 |
|---|---|
| `target/m11-board/chrome-accepted-20261013/qualification.json` | `6144480e9d85ba2026a3e4dc72804443aa289bb9daeba18a137fe490b43de0f7` |
| `target/m11-board/firefox-accepted-20261013/firefox.json` | `cf41d9984ccb748c4505bb9a441b4c8d72f0974195569b80698cafd2ff21b60c` |
| `docs/YT-074-SPOKEN-AT-EVIDENCE.json` | `5b9cd8b3c9ba2f68c8154e75020f2a9ba05567689645631f3350b6a724375fda` |

Chrome's existing helper writes screenshot paths under the original gallery. Fresh captures were archived in `target/m11-board/chrome-accepted-20261013/screenshots/`, and the original gallery bytes were restored. The raw report retains its emitted paths; use the archived files for that run. The old Firefox report was likewise preserved while the fresh report was copied to the unique directory above. Intermediate diagnostic browser reports are retained locally but are not the final candidate's acceptance evidence.

### Full mixed-load execution — terminal exit 0

The single new execution is `python3 -u examples/lab-operations-board/qualify_local.py --profile full`, started at `2026-10-13T04:00:15Z`. Artifacts: `target/m11-board/full-20261013T040015634705Z/`; console log: `target/m11-board/full-mixed-20261013-execution.log`. All source/build inputs and the release binary are fixed during the run. The desktop fixture was fully cleaned before this run; documentation edits, occasional read-only progress queries and unrelated workstation activity are not excluded, so this is not a controlled idle-host or maximum-throughput benchmark.

The same execution ended at `2026-10-13T05:02:16.801532Z`, exit 0. Actual phase durations total 3,720.468 seconds; initialization and cleanup bring overall wall time to 3,721.167 seconds. All phases completed:

| Phase | Actual seconds, including login | Data requests | Read p95 ms | Write p95 ms | Write p99 ms | Combined p99 ms | Maximum ms |
|---|---:|---:|---:|---:|---:|---:|---:|
| Warmup (advisory) | 120.046 | 1,500 | 43.990 | 46.294 | 540.864 | 536.484 | 555.184 |
| Round 1 | 600.077 | 7,500 | 43.731 | 45.526 | 511.009 | 511.434 | 545.234 |
| Round 2 | 600.079 | 7,500 | 43.440 | 44.529 | 390.561 | 364.739 | 545.330 |
| Round 3 | 600.077 | 7,500 | 43.856 | 44.842 | 532.763 | 529.739 | 547.484 |
| Soak | 1,800.190 | 22,500 | 43.028 | 44.194 | 443.526 | 425.966 | 550.236 |

Each completed measured round contains exactly 6,000 reads and 1,500 writes, zero unexpected responses and no worker errors. All three measured rounds pass the unchanged thresholds. Their achieved rates are 12.498 requests/second including login wall time; CPU deltas are 15.70, 15.78 and 15.64 seconds. This is paced traffic, not a capacity result. The raw per-request samples preserve all tails; no percentile was calculated after dropping slow requests.

The soak also passed: 18,000 reads and 4,500 writes, zero unexpected responses or worker errors, 12.499 requests/second and 47.47 CPU seconds. Its write maximum was 549.007 ms and read maximum 550.236 ms. The measured total is **45,000 requests: 36,000 reads and 9,000 writes**, with zero unexpected responses. Including warmup, 46,500 data requests contain 37,200 reads and 9,300 writes. Re-login requests are outside that mix: the five phases recorded 150 logins/450 authentication requests. All p50/p95/p99/max values and every raw latency sample remain in the report/sample files below. No fault injections were mixed into these timings; retained bounded fault evidence has SHA-256 `f20c5e8a65002a41d71a76bcce7fac368b98bba2f2ba7c1825d17841e5640920`.

Final database checks observed exactly 10,000 items and **99,300 history rows**, matching 90,000 initial rows plus all 9,300 successful writes, with SQLite `integrity_check=ok`. The cap remains 100,000. No rejected capacity response was counted as successful throughput.

The existing monitor retained 3,719 resource samples. RSS ranged 12,036–15,540 KiB; first/last samples were 12,036/15,500 KiB, and first/last 60-sample medians were 12,576/15,484 KiB. Threads ranged 25–26 and FDs 46–59. All sampled active SSE counts were ten; each reader made 13 connections and four explicit logins across normal lifetime renewal. Stream errors were empty and final active streams zero. Summed phase CPU was 97.80 seconds, about 2.63% of one logical CPU over measured wall time. These finite observations do not prove leak freedom; SQL queue depth remains **Not instrumented**.

Graceful shutdown took 0.003234 seconds. The helper joined all monitor/SSE threads, reaped the application and removed the private database/log directory. After terminal exit, application PID 242077 and helper PID 242031 were absent, loopback port 56883 was closed, and no owned directory remained. All desktop/browser fixtures had already been cleaned. No task-owned active or uncertain execution remains from qualification.

### Full-run identity and artifacts

The report records HEAD `f00ec597e43f129a649fd1b8805fdd89edb93fbb`, startup dirty state and tracked diff SHA-256 `77f35a28adbce6e543b96b9609685961daf30493ab8c5d5d6438ee0ad7bc5340`. Canonical helper SHA-256: `f4931cefd3629bd0b5f1607dba3f2d80eeca6be3d8a18ca33882390bef682e30`. All **175 source/build/asset/helper inputs**, including the newly generated untracked JavaScript asset, and the final binary remained unchanged throughout the run. An independent post-exit hash comparison also matched all 175 inputs. Later Markdown, transcript and tracker edits are documentation-only changes; the startup diff hash is not relabeled as their final patch identity.

| Artifact under `target/m11-board/full-20261013T040015634705Z/` | SHA-256 |
|---|---|
| `report.json` | `5d0857ebd22f6018d9a267a94194e36fada88729eb973fd605598f6916427e22` |
| `warmup-samples.json` | `e88dae6cfa47f57fa76a844f1aeb322c183e1751caeccd336cf9c65d66a64d83` |
| `round-1-samples.json` | `d18641b855ef6ae20d5935435a394340c4fb952ac0c4731bf59e8a2125bb8aae` |
| `round-2-samples.json` | `8b839bedf7646013b2e1ade1a4372fb521eab85ccaeb3bd6a2d9192b61e56e1c` |
| `round-3-samples.json` | `5b96d4332a9e09cc486b871d16d852103ce630a1b99aa246f9e44bf72474bd12` |
| `soak-samples.json` | `6311828a4487f5787de477e7a37240d8d9234dece2e933b20542af9cf09cdb1f` |
| `SHA256SUMS` | `540eea932e4995239e0955bd70b2783c27c5832c11fdab1f2e4ffc8b8a84d419` |

Console log `target/m11-board/full-mixed-20261013-execution.log` SHA-256: `da3854e6ab7569b594638fa8aa40f8ac1b68541aac16e01aae29a97a0c489c47`. Ignored local raw artifacts remain available at these paths. The original burst failure, aborted warmup attempt and earlier review below are preserved as historical evidence. No replacement or automatic repeat is authorized; subsequent local commit authority is recorded above.

### Current independent review and delivery

Fresh independent review `/root/yt074_mixed_final_review_20261013` returned `overall_status=pass`, with zero issues, required followups or optional notes. It covered planning baseline `d96f7af1073b12eab1a547d57f0d1b4d97589d7c..f00ec597e43f129a649fd1b8805fdd89edb93fbb` plus all 32 current tracked/untracked paths, selected evidence, the original failure and explicit workload amendment. The historical review below is not reused as review of this repair.

The reviewer independently matched all 175 source/build/asset/helper input hashes and the release binary, verified the full-run artifact hashes, and recalculated counts, outcomes and all latency percentiles/maxima from all five raw sample files. It checked the retained failure and sync trace, current browser results, all 546 attributed speech utterances, real IME observations, all ten IME/AT screenshots, and owned cleanup. It inspected the security, Session, database, transaction/history/CAS/capacity, finite-asset and SSE boundaries and found no material defect. This was ordinary read-only review, not formal assurance, a new execution or production approval.

The workspace profile was healthy, current and eligible. Local dispatch-trace verification confirmed registered role `reviewer`, model `gpt-6-astra` and effective effort `high`, matching the resolver contract, distinct from the parent's effort and recorded as enforced. Returned review: `target/m11-board/yt074-mixed-independent-review.json`, SHA-256 `2d410161423d6daefc98bce00941584893b6d2482b6f6a832b7bb7479a7fd5ce`.

Final tracker reconciliation marks YT-074 Done only at the explicitly amended scope: 81 items comprise 73 Done, one Blocked and seven Todo. The post-review edits only record this result and reconcile documentation/status; application sources and acceptance evidence did not change. YT-020 remains Blocked; YT-075–YT-081 remain Todo. No commit or push occurred.


### Validation and actual delivery paths

Final Board checks passed: `cargo fmt -p yetlet-lab-operations-board -- --check`, `cargo test --locked --offline -p yetlet-lab-operations-board` (14 tests, zero failed/ignored/filtered; binary/doc targets had zero tests), strict `cargo clippy --locked --offline -p yetlet-lab-operations-board --all-targets -- -D warnings`, and `cargo build --release --locked --offline -p yetlet-lab-operations-board`. `python3 examples/lab-operations-board/build_assets.py --check` and Python compilation of the changed helpers passed. The unchanged form/CSS proof retains its earlier ten tests, format, Clippy and asset evidence; no redundant suite repetition is claimed. After independent review and Done reconciliation, `python3 scripts/check_docs.py` passed for 75 documents and 81 backlog items, `python3 scripts/check_samples.py` passed for all 12 samples, and `git diff --check` passed. The ledger contains 73 Done, one Blocked and seven Todo; HEAD is unchanged and the staged index is empty.

The current delta has 32 paths: 16 modified tracked files, one deleted obsolete generated JavaScript asset and 15 new files. Nothing is staged. Actual paths follow; the older 20-file count below belongs to the historical closeout.

- `docs/BACKLOG.md`
- `docs/M11-P-BOARD-VERIFICATION.md`
- `docs/M11-P-IMPLEMENTATION-HANDOFF.md`
- `docs/README.md`
- `docs/ROADMAP.md`
- `docs/YT-074-QUALIFICATION.md`
- `examples/lab-operations-board/README.md`
- `examples/lab-operations-board/assets/enhance.js`
- `examples/lab-operations-board/assets/generated/enhance-75be407d3a05031dcca447303611fae595332a8e3733de8fa6a2d60d2a5d0b2e.js`
- `examples/lab-operations-board/assets/inventory.json`
- `examples/lab-operations-board/assets/manifest.json`
- `examples/lab-operations-board/qualify_local.py`
- `examples/lab-operations-board/src/asset_data.rs`
- `examples/lab-operations-board/src/views.rs`
- `examples/lab-operations-board/verify_firefox.py`
- `kanban.md`
- `todo-ledger.json`
- `docs/YT-074-DESKTOP-ACCEPTANCE.md`
- `docs/YT-074-DESKTOP-EVIDENCE.json`
- `docs/YT-074-SPOKEN-AT-EVIDENCE.json`
- `docs/screenshots/yt074-at/accepted-authoritative.png`
- `docs/screenshots/yt074-at/accepted-live-focus.png`
- `docs/screenshots/yt074-at/conflict-focus.png`
- `docs/screenshots/yt074-ime/01-real-chewing-candidates.png`
- `docs/screenshots/yt074-ime/02-real-ime-committed-form.png`
- `docs/screenshots/yt074-ime/03-ime-persisted-detail.png`
- `docs/screenshots/yt074-ime/04-validation-retained.png`
- `docs/screenshots/yt074-ime/06-conflict-page.png`
- `docs/screenshots/yt074-ime/07-field-validation.png`
- `docs/screenshots/yt074-ime/08-authoritative-version.png`
- `docs/screenshots/yt074-ime/README.md`
- `examples/lab-operations-board/assets/generated/enhance-1005530f29d529e4ec1c3d145719f13a42c937893984922834f11502a751b21e.js`

`BOARD-START.md` remains unchanged because the application's normal startup contract did not change. Raw performance, browser and diagnostic artifacts remain under ignored `target/m11-board/`. No source commit or push had occurred at qualification closeout. The owner subsequently authorized local delivery as `fix: qualify Board mixed load and Orca announcements`; strict/builtin Commit Guard remains mandatory before that action.

No authorized non-GPU implementation task remains after YT-074 closes. YT-081 still depends on YT-080 unless the owner explicitly defers that renderer gate; it cannot be marked Done automatically. Dependency-ready YT-075 requires separate GPU authorization and remains unactivated. YT-020 remains Blocked. This delivery does not activate GPU, CORS, a real IdP, public packages, deployment or production qualification.

## Historical blocked closeout — earlier 2026-10-13 continuation

**YT-074 remains Blocked.** Real Ubuntu Chewing IME now passes; the owner-selected Chrome/Firefox coverage retains its passing evidence. Spoken AT remains incomplete. The corrected full profile terminated with a measured write-p95 failure, so the remaining rounds and soak did not run. Independent read-only review of the candidate and this Blocked closeout passed with no material findings; it does not convert missing acceptance into PASS.

Candidate: `master` at `f00ec597e43f129a649fd1b8805fdd89edb93fbb`, ahead of `origin/master` by two commits. The current delta changes the canonical qualification helper and evidence/documentation only. No application Rust, assets, core API, authentication policy, CORS or GPU behavior changed. No commit or push is authorized.

| Gate | Current actual result |
|---|---|
| A — real OS IME | **PASS:** real Fcitx5 Chewing candidate selection and trusted composition in headful Firefox; exact persisted text, multiline description, 422 relationships and retained 409 draft. See [desktop evidence](YT-074-DESKTOP-ACCEPTANCE.md) and [screenshots](screenshots/yt074-ime/README.md). |
| B — spoken AT | **Blocked:** real Orca/Speech Dispatcher startup and key echo occurred, but Firefox did not register on the desktop AT-SPI bus and no Board content was spoken. One bounded setup correction failed; checklist retained. |
| C — selected desktop browsers | **PASS at amended scope:** the owner selected Chrome and Firefox in place of mandatory Windows Edge. Retained Chrome 155.0.8059.39 and Firefox 157.0.1 journeys apply to unchanged application code. Edge is unverified, not silently claimed to pass. |
| D — original-duration profile | **FAIL / incomplete:** 120-second warmup and the first 600-second measured round completed; write p95 was 548.299 ms against the fixed <500 ms threshold. Rounds 2/3 and 1,800-second soak were not started. |
| E — independent reviewer | **PASS:** native registered reviewer inspected `d96f7af..HEAD` plus the uncommitted delta and returned no material findings. Ordinary review only; full acceptance stays Blocked. |

### Profile and owner decisions

The canonical command is `python3 -u examples/lab-operations-board/qualify_local.py --profile full`. The default remains the retained `bounded` profile (30-second warmup, three 60-second rounds, 300-second soak, two requests/second/client); its meaning was not silently changed. The new full profile preserves 10,000 items, 90,000 initial history rows, 25 clients, four reads then one update, ten SSE readers, 120-second warmup, three 600-second rounds and 1,800-second soak.

The owner selected **0.5 requests/second/client**, below the two/second ceiling, to keep the projected 9,300 writes within the unchanged 100,000-history cap. This is paced throughput, not maximum capacity. Normal Session re-login is configured at 1,200 seconds without extending the application's 30-minute lifetime; this longer-lifetime path was **not reached or qualified** in the failed 12-minute execution. Existing fault injections were not repeated; the retained bounded report remains their evidence.

The first full execution stopped after warmup because the inherited harness applied steady-state latency acceptance to warmup. One narrow correction made full-profile warmup latency diagnostic while retaining all samples and immediate HTTP/SSE error rejection; measured latency thresholds were unchanged. The owner explicitly authorized one corrected full attempt. That attempt failed a real measured threshold and was not restarted or repaired into a pass.

### Corrected full attempt — terminal exit 1

Execution: 2026-10-13 02:08:07.660168 to 02:20:08.385820 UTC. Output: `target/m11-board/full-20261013T020807659975Z/`. The process was tracked to terminal exit 1, not treated as completed full-duration acceptance.

| Phase | Configured seconds | Actual wall seconds including login | Requests (read/write) | Achieved requests/s | Process CPU seconds | RSS start/end KiB |
|---|---:|---:|---:|---:|---:|---:|
| Warmup, diagnostic | 120 | 120.047 | 1,500 (1,200/300) | 12.495 | 3.15 | 9,108 / 14,496 |
| Round 1, measured | 600 | 600.074 | 7,500 (6,000/1,500) | 12.498 | 15.57 | 14,496 / 15,732 |

| Phase/type | p50 ms | p95 ms | p99 ms | Maximum ms |
|---|---:|---:|---:|---:|
| Warmup read | 16.834 | 30.860 | 36.298 | 42.959 |
| Warmup write | 49.343 | 559.408 | 576.998 | 587.164 |
| Round 1 read | 16.752 | 30.571 | 33.280 | 40.391 |
| Round 1 write | 47.042 | **548.299** | 573.986 | 584.154 |

Round-1 combined p99 was 548.299 ms and maximum 584.154 ms. Read p95 <250 ms, combined p99 <1,000 ms and unexpected responses <0.1% passed; **write p95 <500 ms failed**. Warmup combined p99 was 559.408 ms. Both phases recorded zero unexpected HTTP responses and zero worker errors. SSE recorded zero errors. Authentication requests are excluded from the read/write mix: each phase logged in 25 workers (75 authentication requests), with ten separate initial SSE logins (30 requests).

There are 719 one-second resource observations: RSS 10,636-15,840 KiB, 25 threads, 59 file descriptors and ten active SSE readers at every sampled point. Each reader established three connections, renewing near 301 and 601 seconds, then closed during cleanup. CPU advanced from 0.12 to 18.74 seconds across sampled observations. These finite samples do not prove leak freedom. SQL queue depth is **Not instrumented**. The full completion-only database count/integrity check and timed graceful-drain metric were not reached; successful HTTP writes must not be presented as a separately observed final row count.

Environment: Ubuntu 26.04.1 LTS, x86_64 kernel 7.0.0-38-generic, Intel Core i5-13600K, 20 logical CPUs, 64,524,124 KiB total memory, Rust 1.99.0 (`b940084d7`, 2026-09-28). Same-host IPv4 loopback, no proxy. The separate headful desktop fixture overlapped part of this run on the same workstation; its database and port were isolated, but this was not an otherwise idle-host benchmark. No causal explanation for the tail regression is claimed.

### Input identity, artifacts and cleanup

Full-run HEAD: `f00ec597e43f129a649fd1b8805fdd89edb93fbb`; recorded dirty state: modified `examples/lab-operations-board/qualify_local.py`. Recorded diff SHA-256: `300e5008e5278246b3cd7ac846150a8c4ed048e47dd769342cb4edc6bb8f7a4d`. Binary SHA-256: `085011ee5c237aa141283a08fab15975e45385c7feb35dca16e9ad4a8ef00346`. Qualification-helper SHA-256: `c8fcb041dae113d533ade79c3998c7a34f11ee9e3a581da94d730e62e9003aec`.

The report preserves hashes for 175 source/build/asset/helper inputs. A read-only comparison after terminal exit found all 175 and the release binary unchanged. The failure path did not reach the in-harness success-only identity assertion; this later comparison is separate evidence. Documentation and screenshot additions are not relabeled executed program inputs.

| Local artifact | SHA-256 |
|---|---|
| `full-20261013T020807659975Z/report.json` | `220b3c9493cef4dd312b64d49deacd9a76cece4a6a2d3b6d4f0030d8809b6bdd` |
| `full-20261013T020807659975Z/warmup-samples.json` | `57027bd275871f802925b494fba1c7a469cf4b2f0e58eb73660a8fb23c7c31d2` |
| `full-20261013T020807659975Z/round-1-samples.json` | `bba94d5fb73d821b370e1d2f3b35f599c3fb0004614f46a609b8d0e7c752bac9` |
| `full-20261013T020807659975Z/SHA256SUMS` | `730adc457b768614a3d1b13c3d0838d51a6d8ec81e6ff0c1c2dd4711958d964c` |
| `full-20261013-corrected-execution.log` | `49783cab79ce4f202fe662374a868a8064a55e29a83fa7c6e26b08f52ca1138a` |
| First attempt `full-20261013T020432662332Z/report.json` | `c3fa04eacfa9dd1d9f7bc74f07814a9519c4bcba240e987032df258119175536` |
| First attempt `full-20261013T020432662332Z/warmup-samples.json` | `079afe53acc8e85cdd89b4175a783a61e420c2ce627bc440025e0df8d926e3d1` |
| First attempt `full-20261013T020432662332Z/SHA256SUMS` | `87359459ada855d2de82406c89d2fd3e015ea035cf9417d2fd2f7c229fcb62f0` |
| First attempt `full-20261013-execution.log` | `855216e5ff8a0d6c42db84a78af2becec7e86d725f7498d147715fc48ac21faf` |

Paths in this table are relative to `target/m11-board/`; these retained local raw artifacts are ignored by Git. The first attempt's 120.043-second warmup produced 1,500 requests, no errors, write p95 517.361 ms, p99 552.541 ms and maximum 561.999 ms before its harness-placement failure. It is preserved, not substituted for the corrected attempt.

Both load attempts reaped their owned server, joined all monitor/SSE threads, verified their loopback listener closed and removed private synthetic database/log directories. Corrected report `active_streams` ended at zero. The separate [desktop cleanup](YT-074-DESKTOP-ACCEPTANCE.md#owned-resource-cleanup) also completed. No active or uncertain task-owned process/Job remains.

### Required regression and next work

Asset checks for both applications, `cargo fmt ... -- --check`, `cargo test --locked --offline -p yetlet-form-css-proof -p yetlet-lab-operations-board --all-targets`, strict Clippy for those targets and the release Board build passed. Tests: 10 form/CSS plus 14 Board tests, zero failures/ignored/filtered; binary targets contained zero tests. Python compilation/help and a focused check of the warmup gate branch passed. Final documentation/sample checks passed; the independent review result is recorded below.

| Required command | Observed result |
|---|---|
| `python3 examples/form-css-proof/build_assets.py --check` | PASS |
| `python3 examples/lab-operations-board/build_assets.py --check` | PASS |
| `cargo fmt -p yetlet-form-css-proof -p yetlet-lab-operations-board -- --check` | PASS |
| `cargo test --locked --offline -p yetlet-form-css-proof -p yetlet-lab-operations-board --all-targets` | PASS: 24 tests, zero failed/ignored/filtered |
| `cargo clippy --locked --offline -p yetlet-form-css-proof -p yetlet-lab-operations-board --all-targets -- -D warnings` | PASS |
| `cargo build --release --locked --offline -p yetlet-lab-operations-board` | PASS |
| `python3 scripts/check_docs.py` | PASS: 75 documents / 81 backlog items |
| `python3 scripts/check_samples.py` | PASS: 12 catalog samples |
| `git diff --check` | PASS |

The next bounded non-GPU work is to resolve the observed write-p95 failure using retained samples and establish a usable Firefox/Orca application-content journey. The authorized corrected long execution has been consumed; do not start another full run without renewed owner authorization. Do not alter security, transaction, conflict or capacity contracts to make qualification easier. YT-074 stays Blocked, YT-075-080 remain Todo/unactivated and YT-020 remains Blocked.

### Independent read-only review — terminal PASS

Native managed dispatch `/root/yt074_independent_review_20261013` completed with `overall_status=pass`, zero issues and zero required followups. The registered `reviewer` reviewed planning baseline `d96f7af1073b12eab1a547d57f0d1b4d97589d7c..f00ec597e43f129a649fd1b8805fdd89edb93fbb` plus the helper, documentation and untracked evidence delta. The workspace profile was healthy, current and eligible; the central resolver selected ordinary ad-hoc deep review. Local trace verification confirmed role `reviewer`, configured model `gpt-6-astra` and effective effort `high`, matching dispatch and distinct from the parent's effort. The resolver recorded the matching contract as enforced. This is independent ordinary review, not formal assurance or production approval.

The reviewer found no material code/evidence defect requiring changes, independently checked raw samples/latency, all 175 input hashes, unchanged binary, resource observations, retained browser reports, seven screenshot hashes and visual contents, cleanup and documentation consistency. It explicitly retained the spoken-AT and failed full-profile acceptance gaps. No server, browser, load execution, installation or workspace mutation was performed by the reviewer. Only terminal-result reconciliation and delivery metadata were added after review; no program or acceptance evidence was changed.

Returned JSON: `target/m11-board/yt074-independent-review.json`, SHA-256 `75b635bcc6d4887e0eded5fdd7938609fd3a237b89b0625602c8ea9cb16cfa12`. The result does not authorize another full run, commit or push.

### Delivery files and repository state

Twenty files are changed or new, all unstaged: ten tracked modifications and ten new files. HEAD remains `f00ec597e43f129a649fd1b8805fdd89edb93fbb`, `master` ahead of `origin/master` by two commits; no local commit or push was made by this continuation.

| Group | Actual repository paths |
|---|---|
| Canonical helper | `examples/lab-operations-board/qualify_local.py` |
| Current reports and handoff | `docs/YT-074-QUALIFICATION.md`, `docs/YT-074-DESKTOP-ACCEPTANCE.md`, `docs/YT-074-DESKTOP-EVIDENCE.json`, `docs/M11-P-BOARD-VERIFICATION.md`, `docs/M11-P-IMPLEMENTATION-HANDOFF.md` |
| Entry points and trackers | `examples/lab-operations-board/README.md`, `docs/README.md`, `docs/ROADMAP.md`, `docs/BACKLOG.md`, `todo-ledger.json`, `kanban.md` |
| Desktop images | `docs/screenshots/yt074-ime/README.md` and its seven explicitly named PNGs in the [gallery](screenshots/yt074-ime/README.md) |

`BOARD-START.md` is unchanged because startup behavior did not change. Local raw load/desktop/reviewer artifacts remain under ignored `target/m11-board/` with hashes above. If the owner later authorizes a commit, a suitable message is `test: add YT-074 full profile and blocked qualification evidence`; the required strict/builtin Commit Guard still precedes that separately authorized action.

## Historical bounded qualification — retained pre-handoff evidence

The following checkpoint predates this local continuation. Its manual gaps, source identity and self-review statements describe that earlier execution only; the current table above supersedes its open-cell status.

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
