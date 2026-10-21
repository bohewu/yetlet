# M10g Verification — Local and Docker Operations

**Date:** 2026-10-10
**Task:** YT-066 / M10g
**Status:** Done at single-host Docker multi-instance and selected Redis-provider scope. Redis is experimental, single-primary and explicitly no-persistence.
**Final closeout review:** PASS; independent read-only reviewer thread `01a12a61-fd85-7d40-8df2-4ce9b5977dd7` reached terminal exit 0 with verified `gpt-6-astra` / `high` and unchanged inputs. Local commit is authorized; push is not. See the [current closeout](#yt-066-bounded-framework-closeout--2026-10-11). YT-020 remains Blocked.

The original local/Memory sections and later b1/Cookie/b2/b3 entries below retain their historical task, source, review and delivery states. Their In progress or unstaged statements are not current task-state assertions; they do not become new executions at the final closeout HEAD.

## Authority and bounded scopes

The original local/Memory work follows the [M10 remaining plan](M10-REMAINING-PLAN.md), [provider operations contract](PROVIDER-OPERATIONS.md), [ADR 0005](adr/0005-opt-in-capability-modules.md) and the independently reviewed [YT-065 / YT-066 plan](YT-065-066-PLAN.md). It preserves the existing synchronous process-local `SessionManager<T>` / `Session<T>` API and does not introduce Redis, a generic store trait, a second HTTP stack, DI, ORM or production deployment.

Two different evidence slices must remain separate:

1. the earlier local-process Session isolation and local SQLite concurrency/backup/restore experiment; and
2. YT-066a, a single Ubuntu host running three isolated Auth Notes containers behind one loopback-published reverse proxy.

Neither slice proves a shared Session store, Redis semantics, physical-host HA, production failover, load/SLOs or recovery objectives. YT-066c cross-VM work remains cancelled. The changes were unstaged and uncommitted at that local/Memory checkpoint; later deliveries and the current closeout are recorded separately below.

## Existing local-process and SQLite evidence

The prior source/lifecycle tests passed 10/10. Main-session execution then used actual Auth Notes and SQLite CRUD binaries and produced `target/m10-remaining/local-operations.json`:

- an Auth Notes cookie was valid on its issuing process (200), rejected by a second process (401), and rejected after issuer restart (401);
- two SQLite processes sharing one local WAL file produced one 200 and one 409 for synchronized same-version updates, final version 2 and exactly two successful audit rows;
- online backup, integrity/foreign-key/schema checks, exact isolated restore, excluded late write and corrupt-candidate rejection passed;
- seven owned processes and all temporary database/WAL/SHM state were removed; and
- raw cookies, CSRF values, fixture codes, response bodies and process logs were excluded from the report.

Independent read-only reviewer `wc_job_YclpZ0R5tS4m8Y1I` returned PASS for that local slice. It remains local evidence only.

## YT-066a source and topology

The Docker/Memory slice adds or changes only the bounded fixture surface:

- `examples/auth-notes/src/lib.rs`, `main.rs`, tests and README: preserve the default loopback-only sample, plus a separately named `app_for_isolated_container_lab` entrypoint that accepts only loopback or wildcard listeners when `YETLET_AUTH_ISOLATED_CONTAINER_LAB=1` is explicitly selected;
- `qualification/yt066a/Dockerfile`, `compose.yaml` and `nginx.conf`: pinned images, three nonpublished application containers, an internal backend network and a separate front bridge for a proxy published only on host `127.0.0.1`;
- `scripts/qualify_multi_instance.py`: offline build, bounded execution, fixed observations, redacted post-cleanup report and ownership cleanup; and
- `scripts/test_qualify_multi_instance.py`: source-shape, redaction, temp-path, report and cleanup tests.

The default Auth Notes path still rejects `0.0.0.0`. The opt-in lab path is explicitly insecure HTTP fixture behavior, not a production listener mode. App containers run read-only as UID/GID 10001, drop all capabilities, use `no-new-privileges`, have bounded CPU/memory/PIDs and publish no host ports. Nginx 1.30.5 runs read-only as UID/GID 101 with all temp paths on its `/tmp` tmpfs, drops all capabilities and publishes exactly one host-loopback port. The backend network is Docker-internal; the front bridge exists only because Docker 29.9 suppresses published ports for a container attached solely to an internal network.

## Source validation

The current harness recovery checks and retained validation of unchanged application/topology inputs are:

| Check | Actual result |
|---|---|
| Python compile without bytecode | PASS for both YT-066a scripts after the 2026-10-11 syntax repair |
| `scripts/test_qualify_multi_instance.py` | 18/18 PASS on 2026-10-11; existing cleanup/finalization/IO and SIGTERM/SIGINT regressions, no Docker launch |
| Compose configuration | PASS for the dual-network topology |
| Pinned Nginx read-only `nginx -t` | PASS |
| `cargo fmt --all -- --check` | PASS |
| `cargo test --locked -p yetlet-auth-notes --all-targets` | 12/12 PASS |
| `cargo clippy --locked -p yetlet-auth-notes --all-targets -- -D warnings` | PASS |
| `python3 scripts/check_samples.py` | 12 samples PASS |
| `git diff --check` | PASS |

The actual application build used pinned `rust@sha256:af306cfa71d987911a781c37b59d7d67d934f49684058f96cf72079c3626bfe0`, Rust/Cargo 1.88.0, locked vendored dependencies, no build-network access and a read-only source mount. The release binary SHA-256 was `41bc64432004e84101d3c917dd4f7a58decf4988ec5fdfb23848d3bb087cba45`. The proxy used pinned `nginx@sha256:0985e772fb9f729e6fa0980da05fca5d9c468e870eed43071545afa9d2e27d94` and reported nginx 1.30.5.

## Actual YT-066a execution

Final WebCodex validation Job `wc_job_cgCLjWo0A9YvXAK1` reached terminal exit 0. The sanitized report is `target/yt066a/multi-instance.json`, SHA-256 `912b8c463371bfe1c1579d1937b128f3090bf3c28e733441ce463bec461af219`. It records source HEAD `8e157b43244be38c9afc22ed7bcd7bf06351416e` and working-source digest `1deb6e81bbfdd6e1de06195e12d3ac221c051a63b624b472acc87c23a8163e4d` before the later documentation and harness-only repairs. It is retained historical execution evidence, not a run of the final repaired harness.

Observed results:

- initial pooled health traffic reached all three distinct application instances with no Session cookie;
- after stopping app A, 18 pooled requests succeeded through exactly two remaining upstreams;
- after restarting app A, the pool again reached three upstreams;
- the measured stop/degraded observation was 3.194 seconds and rejoin observation was 2.167 seconds; these are observations, not thresholds or SLOs;
- login on app A produced an authenticated Session accepted on A (200) and rejected on B and C (401 each);
- after restarting A, A rejected the old cookie (401), proving current Memory Session is neither cross-instance nor restart-persistent; and
- private success/rejection responses retained `Cache-Control: no-store`.

Cleanup was part of PASS: four owned service containers, the build container, two owned networks, zero volumes and the generated application image were removed. Fresh Docker and process checks found zero related containers, networks, volumes, generated images or host processes. The retained report contains fixed statuses, counts, durations, digests and versions, not raw cookies, CSRF tokens, fixture codes, response bodies, environment dumps or service logs.

## Classified operational failures

Two terminal attempts preceded the successful topology, and both cleaned up fully:

1. Job `wc_job__gNGzQ1M5HD-6iML` timed out at readiness because read-only Nginx still used default FastCGI/UWSGI/SCGI temp paths under `/var/cache/nginx`. A pinned no-service `nginx -t` reproduced the exact failure. Moving every temp path to `/tmp` repaired only the fixture configuration. The retained report is `target/yt066a/first-attempt-readiness-timeout.json`, SHA-256 `118d3ae795301752adecba70a7fa78f394333e8812df6711e72ccf9d2f48baf7`.
2. Job `wc_job_GPKY1fnmtpWILo6P` reached four running containers but host readiness still timed out. Captured Compose state showed `PublishedPort: 0`; bounded diagnostic Job `wc_job_TfzzyZ6JfDzKwrrK` confirmed Docker 29.9 suppresses publication when the proxy is attached only to an internal network. A one-container dual-network diagnostic then proved loopback publication with an internal backend plus ordinary front bridge. The retained second report is `target/yt066a/second-attempt-internal-network-port-disabled.json`, SHA-256 `bdc7304d564621972193c3b4882daf75f87e36692f5629ee1ccae71eb0dc833e`.

These were operational fixture failures, not Yetlet product defects or evidence for wider retries. The final acceptance condition passed once after the minimum evidenced topology repair and was not rerun.

## Reviewer findings, repair and remaining work

First independent read-only reviewer Job `wc_job_B-D4_gtfJX2FtYcR` reached terminal exit 0 with the saved `gpt-6-astra` model and high effort, but returned **FAIL** with two material P2 findings. The review target manifests, retained reports, Git status and HEAD stayed unchanged.

1. Cleanup could falsely report PASS when a final Docker query failed, because an unsuccessful query's empty output was indistinguishable from verified absence. The generated application image was also missing from the final existence check, including the interruption window after Docker created it but before the harness recorded `image_built`.
2. `SIGTERM` was not converted into a controlled cancellation, so process termination after Compose startup could bypass the Python `finally` block and leave detached owned resources.

The bounded repair changes only the existing harness and focused tests:

- Docker list/recheck commands now return an explicit unknown state on exception or nonzero exit; cleanup passes only when every final query succeeds and every owned-resource result is empty.
- The run-unique application image is always removal-attempted and is queried by exact tag during final verification, independent of an in-memory build flag.
- `SIGTERM` raises a controlled cancellation before cleanup, records a nonpassing outcome, captures bounded diagnostics when Compose started and enters cleanup.
- Mocked regressions prove all-query failure cannot pass, a retained image cannot pass and SIGTERM after Compose startup runs diagnostics/cleanup yet remains nonpassing.

Repeat reviewer Job `wc_job_zyL9q109gG0IEJVL` then reached terminal exit 0 with `gpt-6-astra`/high and found one remaining P2: the harness installed `SIG_IGN` immediately on entering cleanup, so a **first** SIGTERM received only during cleanup could be discarded and leave an otherwise successful run as PASS. The reviewer again explicitly required only a focused mocked repair and no Docker rerun.

The second bounded repair installs a cleanup-time handler when no earlier cancellation exists. Its first SIGTERM records cancellation without interrupting resource removal, switches only subsequent SIGTERM signals to ignore, and prevents PASS even when cleanup itself succeeds. When cancellation was already recorded before cleanup, a cleanup-time SIGTERM is already subsequent and may be ignored. Both paths restore the previous handler.

Third reviewer Job `wc_job_wRoP0ELjQoxNNmw_` reached terminal exit 0 with `gpt-6-astra`/high and found one final P2 in the same state machine: the cancellation flag was reconciled before `write_report` and owned work-directory removal, so a first SIGTERM at report-write entry could be recorded yet leave exit 0 and a PASS report. The reviewer again explicitly prohibited a redundant Docker rerun and requested one focused mocked regression.

The third bounded repair keeps the recording handler active through the candidate report write and remaining owned-work cleanup. It then closes the cancellable finalization window, checks whether a signal arrived, and if so forces cancelled/non-PASS status, exit 1 and an atomic replacement report. A focused test injects the first SIGTERM at report-write entry and verifies completed cleanup, removed work state, nonzero exit, final cancelled/non-PASS content and restored handler.

Fourth reviewer Job `wc_job_bOM29UZoZ9E8pn4s` reached terminal exit 0 with `gpt-6-astra`/high and found two finalizer P2 findings. First, the raising handler could still fire at the transition into the old `finally` block before its cleanup handler was installed, escaping without Docker cleanup, report output or handler restoration. Second, `shutil.rmtree(..., ignore_errors=True)` could hide work-directory removal failure and still permit PASS, while report-writer failure occurred before that removal and could skip it entirely. The reviewer again required focused regressions and no Docker rerun.

The fourth-round repair made the finalizer a single phase-aware state machine. The SIGTERM handler marks cleanup mode before raising, and an outer recovery boundary catches a signal at any transition into teardown. During cleanup/finalization the first signal records cancellation without raising, switches only subsequent signals to ignore and is reconciled before success. Docker cleanup and owned-work removal are independent required gates; removal errors are recorded and prevent PASS. Work state is removed before report writing, report-I/O failure exits nonzero and removes any stale provisional report, and the original signal handler is restored by the outermost `finally`. Three tests inject SIGTERM at the cleanup transition, force report-write failure and force work-directory deletion failure. That historical candidate passed 15 focused tests.

Fifth reviewer Codex thread `01a1267b-aa58-77e2-b842-1f03cceb9014` reached terminal exit 0 but returned FAIL because SIGINT could still interrupt cleanup/finalization and leave resources or a provisional PASS. The existing follow-up extended the same phase-aware handler to SIGINT, ignores either subsequent cancellation signal during teardown and restores both original handlers. It added three focused regressions for first SIGINT during cleanup and mixed SIGTERM/SIGINT sequences.

Sixth reviewer Codex thread `01a12684-0c67-7aa0-8fab-b80ad54653b5` reached terminal exit 0 but returned FAIL: a stale duplicated finalization fragment caused `IndentationError`, so that candidate could not import and its claimed current-candidate 18-test PASS was invalid. The 2026-10-11 recovery reproduced that failure, removed only the eight stale lines, then actually compiled both scripts in memory and ran the existing 18 focused tests successfully. No test source, Rust source, Cargo input or container configuration changed in this recovery.

All six earlier reviewer results require only focused verification for these repairs, not a Docker qualification rerun. The three retained Docker report digests still match their recorded values. The seven Auth Notes and container-topology files are byte-identical to the first YT-066a review manifest. The successful behavior report therefore remains bounded application/topology evidence; the current focused tests cover the repaired termination, transition, IO and false-PASS paths. Review and recovery evidence is retained under ignored `target/yt066a-closeout-local-20261011/`.

## Final independent closeout review — 2026-10-11

| Observation | Result |
|---|---|
| Dispatch | Local ordinary `codex exec`, role-directed with the saved repository reviewer configuration and canonical reviewer contract; no WebCodex Job or native managed-role identity claimed |
| Reviewer thread | `01a1286a-5829-7411-9ecc-dbc3f00fe001` |
| Execution | Terminal exit 0; observed model `gpt-6-astra`, requested and observed effort high, read-only sandbox |
| Verdict | Valid canonical JSON: PASS, zero issues, zero required followups, zero optional notes |
| Review boundary | Corrected harness, existing focused tests, application/topology inputs, retained reports and adjacent status documents; target contents, reports, HEAD and Git status unchanged during review |
| Current checks | Both scripts compile in memory; 18 existing focused tests PASS; documentation/link/whitespace checks PASS for 75 configured documents and 79 with the four new Markdown paths included in memory, 67 backlog items |
| Preservation and resources | Original single `master` checkout; no staged files, commit or push; no Docker qualification rerun; passive checks found no owned containers, networks, volumes, generated app images, application processes or work directories |

Only factual review-outcome and task-status records were updated after PASS; the reviewed harness and application/topology inputs remain unchanged. The report at `target/yt066a-closeout-local-20261011/review-evidence.json` retains the result and bounded execution observations. This closes YT-066a at single-host Docker/Memory proof scope, not distributed Session or production qualification.

YT-066 remains **In progress**. The next separately authorized stage is YT-066b design and real Redis semantics prototype: async opt-in provider shape, versioned/CAS update, rotation/revocation, stale cookie response ordering, TTL, fail-closed outage behavior, bounded pool/queue/timeouts, ambiguous writes, serialization/version handling and selected persistence boundary. Existing YT-066b design documents were preserved and not advanced by this closeout. Redis HA/Cluster, physical-host HA, production RPO/RTO, load/SLOs, SQL Session and YT-020 publication remain outside this slice.

## Subsequent YT-066b1 execution — 2026-10-11

After the independent YT-066b0 design PASS, the maintainer separately authorized a private real-Redis semantics spike and local independent review. [YT-066b1 verification](YT-066B1-VERIFICATION.md) records the isolated crate/client/image identities, six passing Rust 1.88 integration tests, the later counter-order correction with two affected passing tests, strict stable Clippy and complete owned-container cleanup. The HTTP cookie jar reproduces replacement-cookie loss after the final check or delayed Set/Clear delivery; that unresolved guarantee blocks full-provider implementation. This follow-on neither reruns nor relabels YT-066a evidence. Independent local read-only reviewer thread `01a12884-69a7-7e20-beb5-9710e561c4d9` returned canonical PASS, exit 0, with observed `gpt-6-astra` / `high` and unchanged workspace. YT-066 remains In progress.

The maintainer subsequently authorized the local YT-066b1 commit and a cookie-ordering repair without weakening the guarantee. [Cookie gate evidence](YT-066B-COOKIE-ORDERING.md) records commit `6fd6876`, the separately uncommitted candidate, actual Redis/HTTP observations, two material repairs and final independent reviewer **FAIL**. Reviewer thread `01a1289c-e4e8-77b2-b374-c6f4153e3a4f` completed read-only with observed `gpt-6-astra` / `high`, exit 0 and unchanged workspace. Delayed lower cookies can evict the replacement or logout fence; a focused actual-Chrome storage probe confirmed that boundary. The original guarantee was unchanged and unresolved at that failed-review checkpoint. YT-066b2 did not start on that evidence.

### Later selected pure-SSR cookie contract — 2026-10-11

The maintainer explicitly accepted no mandatory JS, preserved authorization safety, and possible login loss after extreme response reordering. The [cookie record](YT-066B-COOKIE-ORDERING.md) distinguishes that availability tradeoff from the historical failed guarantee. A fixed-name cookie now emits only for proven create/rotate/replace, with current Redis validation; read/update/error/logout responses emit no Set/Clear, retired IDs fail closed, and no alternate cookie is selected. Four focused policy/Redis/HTTP tests passed; after one navigation-wait harness correction, the affected Chrome test passed with script execution disabled. Both exact Redis projects and the owned browser were cleaned up. Independent read-only review passed: thread `01a128b4-dbc6-7922-87e7-ab67c171665a`, exit 0, observed `gpt-6-astra` / `high`, 268 inputs unchanged, no issues or required followups. This opens the separately authorized YT-066b2 stage. No YT-066a rerun, Memory API change, commit or push accompanies this correction.

### YT-066b2 implementation checkpoint — 2026-10-11

After Cookie PASS, the authorized [experimental provider implementation](YT-066B2-VERIFICATION.md) began in a separate unpublished crate/workspace. Rust 1.88 compilation, strict stable Clippy and the actual-Redis cancellation/drain test passed. The CAS callback test stalled twice because its intended synchronization correction was not applied before the second launch. Both exact Redis projects were cleaned up; the repeated-harness stop rule ended retries at that checkpoint.

The maintainer then explicitly authorized the barrier correction, requiring confirmation that the edit was applied before testing. The parent verified the one-line `block_in_place` correction and unchanged provider runtime hashes before rerunning only the provider executable: all five tests passed on Rust 1.88, followed by stable strict Clippy and formatting checks. The exact new Redis project was cleaned up with no containers/networks/volumes left. The cancellation/drain case and prior broad Docker/browser evidence were retained without rerunning. Independent read-only implementation review passed: thread `01a128d5-37d1-7f40-a7c7-84fc0c923771`, observed `gpt-6-astra` / `high`, exit 0, 276 inputs unchanged, no issues or required followups. YT-066b2 is Done at bounded experimental-provider scope. YT-066 remains In progress; YT-066b3 has not begun. No commit or push occurred.

### YT-066b2 local delivery and YT-066b3 actual qualification — 2026-10-11

The maintainer explicitly authorized peon to commit reviewed b2 before proceeding. Commit `1f38d20b8ebaf9b679aa69ce461b495db1856db2` contains the exact 24 reviewed provider/Cookie paths; staged guard exited 0 under the existing audit/builtin configuration with four document-path warnings, and working bytes were preserved. No push occurred.

The separately authorized [b3 qualification](YT-066B3-VERIFICATION.md) then passed one actual offline Rust 1.88 build and three-app/Nginx/single-Redis execution. Cross-process CAS, rotation/revocation, pre-finalizer stale suppression, prepared HTTP Cookie reorder, no-JS Chrome behavior, TTL, unknown-result reconciliation/expiry, caller cancellation/admission/drain, application restart and controlled Redis outage/no-persistence restart passed. The five service containers, two networks, build container, generated image, work and browser resources were removed. Exact execution source/build identities and retained evidence are recorded in b3 verification. A first independent reviewer found one material browser-cancellation cleanup defect; the smallest cleanup repair and four affected checks passed without rerunning Docker. Final local read-only reviewer thread `01a129d2-3cce-72d3-a0de-bd286565dade` reached terminal exit 0 with observed `gpt-6-astra` / `high`, 286 inputs unchanged, no issues or required followups. YT-066b3 is Done at bounded single-host/single-primary/no-persistence scope; YT-066 remains In progress and YT-020 Blocked. These b3 changes are unstaged/uncommitted, with no push.

### Later b3 local delivery and persistence/recovery planning — 2026-10-11

The next explicit instruction authorized peon to commit the reviewed b3 scope.
Commit `4da0f6c1879c9a56598caec7cad449cd9127c7de` has parent `1f38d20b8ebaf9b679aa69ce461b495db1856db2`
and tree `e500a7baeed459e5a32ca7dedfe1da104f3fafd6`; exactly 18 paths committed,
all 286 working-input hashes preserved, index/worktree clean immediately afterward.
Native peon trace matched `gpt-6-luna` / `high`. Direct staged guard exited 0 with
audit/builtin first-screen coverage, `warn` and two findings; no bypass or policy
change. No push or redundant test execution occurred.

The [production persistence/recovery scope](YT-066-PERSISTENCE-RECOVERY-PLAN.md)
is now drafted as documentation only. Proposed persistent clean restart and
fenced invalidation after crash/older restore remain Not verified and unapproved
for implementation. Production decisions remain open; YT-066 In progress,
YT-020 Blocked. The earlier unstaged b3 statement is its historical closeout state.

### Framework/application responsibility clarification — 2026-10-11

The maintainer challenged treating production operations as framework work and
requested independent review of the corrected documents. The [reconciled
boundary](YT-066-PERSISTENCE-RECOVERY-PLAN.md) removes TLS/ACL deployment policy,
backup protection, business RPO/RTO and operational ownership from mandatory
Yetlet delivery and YT-066 completion gates. Provider semantics, supported
configuration/error behavior, redaction and explicit recovery assumptions remain
within framework scope. Proposed persistence/restore rehearsals are optional,
unexecuted and not newly required tests. This correction does not establish
production durability or alter the recorded b1/b2/b3/Cookie evidence.
YT-066 remains In progress pending bounded framework documentation closeout;
YT-020 remains Blocked. Independent review of this reconciliation passed.

The ordinary local read-only reviewer process used the saved repository reviewer
contract and shared-resolver effort. Thread `01a12a04-d58f-7210-a995-c6f35860437c`
reached terminal exit 0 with runtime `gpt-6-astra` / `high`, valid canonical PASS,
no issues or required followups, and all 287 working inputs, HEAD/status and the
empty index unchanged. No WebCodex Job or native managed-role identity is claimed.
The [full receipt and exact 11-file list](YT-066-PERSISTENCE-RECOVERY-PLAN.md#independent-responsibility-scope-review--2026-10-11)
retain the reviewed input identity. Existing documentation checks passed at
75 required / 100 total Markdown documents and 67 backlog items; no runtime
tests were repeated. Only these outcome receipts were added after review.
The documentation changes remain unstaged/uncommitted; no push or implementation
occurred. Next is bounded framework documentation closeout, not mandatory
production deployment work.

## YT-066 bounded framework closeout — 2026-10-11

### Authority, completion label and preserved contract

The maintainer explicitly requested completing this documentation-only closeout,
then independent repository reviewer review, without WebCodex, a new checkout or
repeating tests with sufficient retained evidence. A local commit is authorized
only after the final independent PASS; push, publication and deployment are not
authorized. The preceding responsibility-scope review is not reused as the new
final closeout review.

YT-066 is **Done at single-host Docker multi-instance and selected Redis-provider
scope**, accepted by the final independent documentation review PASS.
That label includes the unchanged synchronous Memory limitations, selected
experimental async Redis provider and one single-host/single-primary topology
with `appendonly no`, `save ""`, tmpfs and no persistent volume. It is not a broad
distributed-provider, durable-storage or production support label. The terminal
final review gate is recorded below; it is not new runtime qualification.

Keep native Axum/Tower composition, handler/component-first APIs, opt-in
Session/Auth/DB/Cache/SSE, SQL-first application access and Core/default/browser
dependency isolation. No generic store framework, mandatory DI, registry, ORM,
Controller hierarchy, second HTTP stack or persistent browser event circuit is
introduced. The selected Cookie contract stays pure SSR: a retired identity
cannot authenticate, while a prepared late Set may cause login loss and require
explicit login. This does not claim the former unconditional Cookie-retention
guarantee passed or change Memory's recorded API/test contract.

### Retained acceptance evidence, kept source-aware

| Slice | Retained acceptance and boundary |
|---|---|
| Local Session/SQLite | Actual process-local isolation/restart limitations, local WAL conflict, backup/isolated restore/corrupt rejection and owned cleanup; reviewer `wc_job_YclpZ0R5tS4m8Y1I` PASS; no shared-memory-to-distributed inference |
| YT-066a | Actual three-app Docker/Memory stateless distribution, worker stop/restart, cross-worker rejection and issuer-restart loss; retained execution plus focused cleanup repairs; final reviewer `01a1286a-5829-7411-9ecc-dbc3f00fe001` PASS |
| YT-066b1 | [Real-Redis private spike](YT-066B1-VERIFICATION.md), direct mature-client choice and actual CAS/lifecycle/uncertain-result/bounds evidence at its recorded source; original Cookie gate failure remains historical, not relabeled PASS |
| Selected Cookie gate | [Explicit pure-SSR contract](YT-066B-COOKIE-ORDERING.md), focused Redis/HTTP/no-JS Chrome evidence; separate reviewer `01a128b4-dbc6-7922-87e7-ab67c171665a` PASS |
| YT-066b2 | [Provider implementation](YT-066B2-VERIFICATION.md), applied barrier correction, five Rust 1.88 provider tests and retained cancellation/drain, Clippy/format/isolation checks; reviewer `01a128d5-37d1-7f40-a7c7-84fc0c923771` PASS |
| YT-066b3 | [Actual three-application Redis fixture](YT-066B3-VERIFICATION.md), cross-worker CAS/rotate/revoke, TTL, Cookie boundary, unknown results, cancellation/drain, app restart and fail-closed outage/no-persistence restart; focused cleanup repair; final reviewer `01a129d2-3cce-72d3-a0de-bd286565dade` PASS |
| Responsibility boundary | [Framework/application reconciliation](YT-066-PERSISTENCE-RECOVERY-PLAN.md), reviewer `01a12a04-d58f-7210-a995-c6f35860437c` PASS; this is prior documentation evidence, not the final closeout review |

The existing records retain each exact source/lock/image/process/run identity.
For example, b3 executed at baseline `1f38d20b8ebaf9b679aa69ce461b495db1856db2`
plus 285 working inputs, source-map SHA-256
`0378994d46c5eb52a98a3de3cd5ce47c464207098bae8efbf8ac0291182e9ece`, with
runtime-input map `47f8376be53e4404319e6b5ae2090622ad805a8fcbf2bddeeffa0d17b8decd77`.
Its later 286-input review and commit `4da0f6c1879c9a56598caec7cad449cd9127c7de`
are separate identities. The final closeout starts from that commit and changes
Markdown only; none of these historical executions is described as a fresh run
of the final documentation source. Hosted qualification run `38043565238` remains
bound to `c38837835dac683b09194b475baab2d8eda5a875`, not this checkout.

A read-only comparison of the 16 b3 execution inputs finds only its later
fixture README and two cleanup scripts (`run.py`, `browser.py`) differ; the other
13 inputs, including provider/application/dependency/topology bytes, still match.
The script delta has retained focused runner/browser cleanup checks, the four
affected final browser checks and final independent b3 review. Current
non-Markdown inputs match the committed baseline and pre-closeout snapshot. No concrete changed runtime behavior lacks retained evidence, so no
Docker, Redis, Rust or browser qualification is rerun to refresh dates or SHAs.
Only existing documentation/language/link/backlog and whitespace checks run for
this closeout; no validator framework is added.

### Explicit unknowns and deployment responsibilities

Automatic client reconnect recovery, `rediss://`/TLS/ACL qualification,
AOF/RDB durability, fsync acknowledgment, crash/older-backup restore, Redis
failover/Cluster/HA, cross-host partitions, load/soak/SLO, SQL Session and production
support remain **Not verified / optional future qualification**. An application
requiring them must select and qualify its own environment. Older storage may
lose revocation history: documented traffic fencing and reconciliation or fresh
namespace invalidation preconditions still apply; arbitrary restore safety is
not established. The existing provider accepts `redis://` only.

Production certificate/ACL policy, backup protection/retention, business RPO/RTO,
SLA and operational/on-call ownership belong to applications/operators. They are
not framework completion blockers. Provider semantics, supported configuration
and error behavior, bounds and redaction remain framework responsibilities.
The p1/p2 rehearsals remain unexecuted optional proposals; cancelled YT-066c stays
cancelled. Real IME, spoken assistive-technology output, WebKit/mobile, touch/soft
keyboard and zoom/reflow remain optional unverified YT-065 cells. No accessibility
conformance, device certification, external audit or production guarantee follows.

### Task states and next decision

YT-065 is Done at automated native/desktop-browser qualification scope; YT-066
has the bounded label above; YT-067 is Done at its internal pre-patch source-scan
and independently reviewed small-fix scope. No post-fix whole-repository scan is
claimed. YT-020 remains Blocked and the repository remains Private. Required CI
checks and the scheduled/manual workflow are unchanged.

The 67-item backlog contains 61 bounded Done items, five Deferred research items
and one Blocked item (YT-020), with no automatically ready implementation task.
A future public-release activation or application-driven provider/deployment
qualification requires separate scope and authority. No package, release, PR,
push, visibility or GitHub-setting change is included here.

### Final independent documentation review

The final reviewer inspected the actual 16-file documentation diff, the new
responsibility/persistence document and adjacent authoritative contracts and
retained evidence. Fresh workspace profile status was healthy/current/eligible;
the saved repository reviewer model was `gpt-6-astra`, tier strong. The shared
resolver selected ordinary adaptive `review` / `ad-hoc-review`, effort `high`.
The exact role contract was `${CODEX_HOME}/agents-pipeline/agents/reviewer.md`.

A separate ordinary local `codex exec` process used that contract and read-only
sandbox, not WebCodex, `codex exec review`, self-review or native managed-role
dispatch. Thread `01a12a61-fd85-7d40-8df2-4ce9b5977dd7` reached terminal exit **0**;
the exact process is gone. Runtime metadata matched expected `gpt-6-astra` and
requested `high`, read-only sandbox and approval policy `never`; observed-effort
resolution had no conflict or degradation. Canonical JSON:

```json
{"overall_status":"pass","issues":[],"required_followups":[],"optional_notes":[]}
```

All 287 tracked/nonignored working-input hashes, HEAD, Git status and empty index
were identical before/after review, using symlink target text for Git symlinks.
Reviewed source-map SHA-256:
`088eda860f147bcee523e843e6b15d26150d8a892c32da2910050b18988c7a20`.
Reviewed baseline HEAD: `4da0f6c1879c9a56598caec7cad449cd9127c7de`.
Ignored `target/yt066-closeout-20261011/` retains source delta, doc validation,
exact input/output and bounded runtime evidence. No WebCodex Job ID or native
child trace is claimed.

The exact final changed-file list is:

- `README.md`
- `docs/BACKLOG.md`
- `docs/COMPATIBILITY.md`
- `docs/EXTENSIONS.md`
- `docs/M10-REMAINING-PLAN.md`
- `docs/M10G-VERIFICATION.md`
- `docs/M10H-VERIFICATION.md`
- `docs/PROVIDER-OPERATIONS.md`
- `docs/RELEASE-READINESS.md`
- `docs/ROADMAP.md`
- `docs/YT-065-066-PLAN.md`
- `docs/YT-065-066-PLANNING-VERIFICATION.md`
- `docs/YT-066-PERSISTENCE-RECOVERY-PLAN.md`
- `docs/YT-066B-REDIS-SESSION-DESIGN.md`
- `docs/YT-066B3-VERIFICATION.md`
- `docs/adr/0005-opt-in-capability-modules.md`

Existing `python3 scripts/check_docs.py` passed 75 required documents and 67
backlog items. The same checker also passed all 100 repository Markdown documents
selected in memory without checker changes. Tracked/new-file whitespace, English,
internal links, Markdown/backlog structure and DAG checks passed. Non-Markdown
inputs matched the baseline/pre-closeout snapshot. No Rust, Redis, Docker or
browser test was rerun and no runtime resource was started for this closeout.

After terminal PASS, only this factual gate/status/receipt was added; reviewed
completion scope, limitations and non-Markdown inputs did not change. The same
bounded doc/whitespace checks cover the receipt. This is the review record before
local delivery: the maintainer explicitly authorized committing these 16 documents
after PASS. The deterministic staged guard must run against exactly that final
index and match `git write-tree` immediately before commit. No push or release is
authorized, and later commit metadata does not change historical execution IDs.
