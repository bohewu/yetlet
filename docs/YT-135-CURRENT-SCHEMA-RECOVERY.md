# YT-135 — Optional Starter current-schema recovery reference

Date: 2026-10-23. Status: **Done at bounded optional current-schema recovery reference scope**.
Independent Reviewer `/root/yt135_recovery_performance_plan_review_20261023`: **PASS**,
zero issues and required followups; all six reviewed hashes matched before/after.
Controller acceptance records the bounded recovery proof and YT-141 scheduling;
YT-141 remains Todo and no performance campaign is activated.
This opt-in application runbook and disposable local exercise follow the
[conditional contract](PRODUCTION-DEPLOYMENT-PROFILES.md#conditional-future-work-contracts).
Consumer hardware, retention, operators, SLO/RPO/RTO and production DR choices
remain separate; none is a repository Done gate. No Core/runtime change,
production deployment, performance campaign or automatic rollback is selected.

## Contract before execution

Use the exact current Starter source and schema 2, including
[table migration](../examples/production-starter/migrations/0001_init.sql),
[status index](../examples/production-starter/migrations/0002_status_index.sql),
[store](../examples/production-starter/src/store.rs) and
[HTTP application](../examples/production-starter/src/web.rs).
Reuse the accepted YT-133 release executable only if its SHA-256 is
`1ce6aea73a10c4ed15c52af87a842bd7fbc6e83daac5276cb258c3daee9a5796`
and all 345 recorded build inputs match. A mismatch stops this exercise;
rebuilding silently would invalidate reuse. Bind current HEAD/branch, source,
lockfiles, binary, copied Production configuration and both migration hashes
before startup and again after cleanup. Store the exact pre-execution document
copy/hash, command/script/hash, fixture selection and failure policy separately.

The selected fixture is one native release process at a time on a new high
loopback port, private unique ignored `target/` directories on the actual ext4
mount, copied `base.toml`/`production.toml`, metrics off, a protected ephemeral
Bearer token and `TOKIO_WORKER_THREADS=2`. Native child controls are affinity to
two allowed CPUs, Nice 10, 2 GiB address-space limit, 512 open descriptors and
30 seconds of process CPU. These differ from YT-133's optional systemd cgroup
caps: no CPU quota, task-count cap, dedicated service UID or installed unit is
claimed. Startup is externally bounded to 60 seconds and stop to 20 seconds;
the app retains its existing 10-second graceful shutdown. The sole sequential
HTTP client has an 8-second request timeout and no automatic retries. Same-UID
local processes remain trusted. No edge/proxy/SSH throughput is exercised.

Create three synthetic issues through authenticated HTTP, update issue 1 to
in_progress/version 2 and issue 2 to closed/version 2; preserve every title/body,
ID/status/version, the full schema, status index and sequence 3. Pause the sole
writer for SQLite backup. Deliberately acknowledge exactly two later mutations:
issue 1 to closed/version 3 and create issue 4. The predeclared **synthetic loss
decision** permits discarding only these two mutations after their exact
acknowledgments and source readback agree. This cannot approve loss of consumer
data. Unknown outcomes, extra mutations or any manifest difference stop cutover.

Remove the sole client's admission latch, finish all issued calls, SIGTERM the
source, require exit 0 plus `stopped`, no `shutdown_incomplete` and a closed
listener before restoring or starting another writer. Preserve the stopped
source and its history through comparison. The latch is an operator/client
control in this isolated fixture, not a new server or network access fence.
A real private deployment uses the accepted
[YT-133 withdrawal/drain procedure](../examples/production-starter/deployment/private-operator/README.md#withdrawal-stop-and-re-entry)
and must also stop direct-local callers. Readiness alone cannot fence writers.

Use one bad schema-2 candidate with the status index removed: keep cutover
closed on full operator-preflight refusal, and record the actual app startup
refusal separately. Restore the untouched backup into a fresh empty directory,
verify bytes/schema/index/all business fields/sequence/integrity before any
startup, then readiness and authenticated legacy plus v2 filtered readback.
Record one post-restore create (ID 4 is reused), stop cleanly and restart the
same exact release on the verified existing current-schema file for minimal
forward-restart readback. This proves no arbitrary future patch or migration.

Failure handling: preserve raw failures and data observations, never repair
product code, alter schema/version to bypass checks, retune or broaden this
exercise. At most one smallest in-place correction of an ordinary run-local
command is permitted for a harness defect; repeated signature stops. Cleanup
runs on every exit, stops/reaps only owned children, checks listener closure,
closes HTTP/SQLite/log handles and removes the owned fixture and credentials.
Retain redacted synthetic manifests and observations in ordinary `/tmp` evidence.

## Reusable operator procedure

1. Record the exact application/binary/configuration and migrations; identify the
   intended existing database and complete business manifest. Use private 0700
   directories and 0600 files. Select backup protection, retention and separate
   operator authority for retained data. Keep the API secret outside the SQLite
   backup; do not put it on a command line or in retained logs. Set absolute
   `SOURCE_DB`, `BACKUP_DB`, `RESTORE_DB`, `EXPECTED_MANIFEST`, and `CANDIDATE_MANIFEST`
   explicitly. A completed expected manifest must come from the selected,
   reconciled business recovery point, not a guessed path or arbitrary file.
2. Pause all writers and wait for their known outcomes. Take a consistent SQLite
   backup through its supported API, including committed WAL state. Never copy
   only the live main file, delete live WAL/SHM or publish a partial backup.

   ```sh
   python3 - <<'PY'
   import os, sqlite3
   from pathlib import Path
   from contextlib import closing
   source, backup = (Path(os.environ[k]) for k in ('SOURCE_DB', 'BACKUP_DB'))
   assert source.is_absolute() and source.is_file() and not source.is_symlink()
   assert source.stat().st_size > 0 and backup.is_absolute() and backup.parent.is_dir()
   with backup.open('xb'):
       os.chmod(backup, 0o600)
   with closing(sqlite3.connect(source.as_uri() + '?mode=ro', uri=True)) as src:
       with closing(sqlite3.connect(backup)) as dst:
           src.backup(dst)
   PY
   ```

3. Inspect the selected consistent backup using this ordinary Python command.
   `REPO_ROOT` identifies the exact recorded source checkout. Use the same command
   for any candidate by setting `CHECK_DB`; use a new `CANDIDATE_MANIFEST` output
   each time. Its JSON representation includes all rows, not only critical IDs
   or the HTTP list's 100-row maximum. Store the selected backup's output as
   `EXPECTED_MANIFEST` only after reconciling it to acknowledgments/business truth.

   ```sh
   python3 - <<'PY'
   import json, os, sqlite3
   from pathlib import Path
   from contextlib import closing
   path, repo = Path(os.environ['CHECK_DB']), Path(os.environ['REPO_ROOT'])
   assert path.is_absolute() and path.is_file() and not path.is_symlink()
   assert path.stat().st_size > 0
   with closing(sqlite3.connect(path.as_uri() + '?mode=ro', uri=True)) as db:
       manifest = {
           'user_version': db.execute('PRAGMA user_version').fetchone()[0],
           'schema': db.execute('SELECT type,name,tbl_name,sql FROM sqlite_schema ORDER BY type,name').fetchall(),
           'rows': db.execute('SELECT id,title,body,status,version FROM issues ORDER BY id').fetchall(),
           'sequence': db.execute('SELECT name,seq FROM sqlite_sequence ORDER BY name').fetchall(),
           'integrity': db.execute('PRAGMA integrity_check').fetchall(),
           'foreign_keys': db.execute('PRAGMA foreign_key_check').fetchall(),
       }
   ddl = {row[1]: row[3] for row in manifest['schema']}
   migrations = repo / 'examples/production-starter/migrations'
   assert manifest['user_version'] == 2 and manifest['rows']
   assert ddl['issues'] == (migrations / '0001_init.sql').read_text().strip().rstrip(';')
   assert ddl['issues_status_id'] == (migrations / '0002_status_index.sql').read_text().strip().rstrip(';')
   assert manifest['integrity'] == [('ok',)] and manifest['foreign_keys'] == []
   with Path(os.environ['CANDIDATE_MANIFEST']).open('x') as output:
       json.dump(manifest, output, indent=2, ensure_ascii=True)
       output.write('\n')
   PY
   ```

   Compare the candidate JSON to the already accepted expected manifest in full:
   `cmp -- "$EXPECTED_MANIFEST" "$CANDIDATE_MANIFEST"`. Failure keeps access
   closed. Equal bytes prove agreement with that selected manifest; they cannot
   establish business authority by themselves. The existing YT-133
   [receipt helper](../examples/production-starter/deployment/private-operator/preflight.py)
   checks file/parent identity, schema and accepted nonemptiness, **not all business
   rows, sequence or integrity**. It cannot replace this comparison.
4. Before cutover remove business access, drain callers and all old writers,
   require clean native stop and closed old listener. Retain source data and
   acknowledged/unknown write history. A timeout/disconnect can precede a late
   commit; reconcile the actual issue/marker/version against storage before
   deciding. Creates lack idempotency: never blindly retry. Any later writes
   excluded by the backup need explicit application-owned loss/reconciliation
   acceptance. Without it, preserve current schema/data and choose forward fix.
5. Create a new private empty restore directory on the verified intended mount.
   Copy only the completed closed backup using exclusive creation; no old
   WAL/SHM siblings may exist. Do not restore over an active path.

   ```sh
   python3 - <<'PY'
   import hashlib, os, shutil
   from pathlib import Path
   backup, restored = (Path(os.environ[k]) for k in ('BACKUP_DB', 'RESTORE_DB'))
   assert backup.is_absolute() and backup.is_file() and not backup.is_symlink()
   assert restored.is_absolute() and restored.parent.is_dir()
   assert not restored.parent.is_symlink() and list(restored.parent.iterdir()) == []
   with backup.open('rb') as source, restored.open('xb') as destination:
       os.chmod(restored, 0o600)
       shutil.copyfileobj(source, destination)
   assert hashlib.sha256(backup.read_bytes()).digest() == hashlib.sha256(restored.read_bytes()).digest()
   PY
   ```

   Run step 3 on `RESTORE_DB`, compare the full expected manifest and validate
   actual excluded acknowledged work **before startup**. The application
   initializes missing/empty files; green readiness on a replacement cannot
   prove recovery. Missing, corrupt, unexpected, unsupported or unacceptable-loss
   candidates stay closed. Never force `user_version`, remove constraints/index
   or recreate an empty database to bypass a refusal. After full restore checks,
   a YT-133 receipt may be explicitly recorded for the new file identity.
6. Start only the exact verified compatible executable with existing Production
   settings, explicit absolute config/database paths, protected API token and
   a private loopback listener. Require the bounded private readiness hook,
   liveness/readiness 200, authenticated `GET /api/issues/{id}` for expected
   records, legacy `GET /api/issues`, and
   `GET /api/v2/issues?status=in_progress` (plus other relevant statuses) matching
   complete expected fields/order. Preserve API Bearer and no-Origin policy.
   Recheck the full storage manifest, since HTTP lists are bounded. Only then
   admit business traffic after the operator's loss decision. Record each
   post-restore mutation separately and reconcile its returned ID/version.

## Upgrade and forward-fix rules

Reuse [YT-131's accepted migration/overlap/refusal matrix](YT-131-MIXED-VERSION-COMPATIBILITY.md#observed-compatibility-matrix)
and [upgrade conditions](YT-131-MIXED-VERSION-COMPATIBILITY.md#upgrade-rollback-and-forward-fix-conditions).
This exercise does not rerun v1-to-v2 migration. Stop all old writers before
migration: a live v1 can write after readiness fails. Schema-2 writes require a
compatible forward fix/restart; no down migration or arbitrary old-binary
rollback is supported. A fresh restore is a separately verified historical
recovery point and may discard later acknowledgments, even when schema matches.

Refresh every client after restore: old IDs may be reused and old versions may
refer to a different history. Memory Sessions/CSRF are process-local and lost on
stop; reload forms rather than resubmitting old tokens. Starter has anonymous
SSR, no login identity. The separate operator-supplied Bearer key is neither
recovered nor rotated by SQLite restore. Existing YT-130/YT-131 session behavior
and YT-133 edge/access proof remain accepted at their original scope.

## Timing declaration and limits

Use one local monotonic clock and UTC annotations. Report backup start/end
interval, recovery-point-age proxy from backup start to withdrawal (backup-end
age separately), last included acknowledgment age, and exact excluded
acknowledged mutation count/fields. These are fixture observations, not precise
commit timestamps, stable-media durability or a consumer RPO commitment.

RTO begins immediately before source withdrawal/SIGTERM and ends only after
clean stop, rejected-candidate check, fresh restore, complete pre-start manifest,
restored readiness and authenticated legacy/v2 business readback make local
cutover eligible. Include the intentional refusal rehearsal in this declared
boundary and report it separately; exclude pre-outage backup, later post-restore
write/restart, human delay, DNS/proxy switching and client reconnection. No
threshold or production RTO guarantee is selected.

Reuse historical YT-130 schema-1/debug timings only as historical observations;
no transfer to this release/schema-2 fixture. No load/soak, browser/CSP, Windows,
IdP, Valkey, power-loss/media-loss/off-host recovery, HA, multi-instance recovery
or real traffic switch is verified here. Consumer retention, monitoring and
production DR remain unqualified. Parent/controller records independent review
and aggregate documentation validation separately before Done.

## Actual local results — 2026-10-23

The direct command completed with exit 0 on its first invocation. There were
zero operational retries, harness corrections or product repairs. Independent
review remains **Pending**; this records executor evidence, not Reviewer PASS.
The pre-execution copy/hash was retained before application startup; this results
section is additive. Initial HEAD was
`c994d120d6b41829d88dd25a5cfa2bccad158e94`, branch
`docs/yt113-123-opt-in-capability-plan`. All 345 accepted build inputs and the
release executable SHA-256 matched both before startup and after cleanup; no
build ran. Source facts and unchanged accepted proof retain their original scope.

Evidence root: `/tmp/yetlet-yt135-recovery-20261023-5wk7yp5c`.
`contract-before-execution.md` SHA-256:
`7cd8a08c43681e76cb498c543c954a08fce925235c64b4f1962d27f2c02e9953`.
`exact-recovery-command.py` SHA-256:
`16245f35ddd18f2f1a6253b2d9654823f55a461bb6eedad9c8e74290e9b37070`.
`preexecution.json` binds source/build inputs, both migrations, copied settings,
command, selected synthetic data, resources, timing boundaries and failure policy.
`exact-executable-command.txt` retains the exact invocation:

```sh
python3 /tmp/yetlet-yt135-recovery-20261023-5wk7yp5c/exact-recovery-command.py > /tmp/yetlet-yt135-recovery-20261023-5wk7yp5c/recovery-command.log 2>&1
```

The exercise used private fixture
`target/yt135-recovery-wg4h4y97` on `/dev/nvme0n1p2`, ext4 (`rw,relatime`),
loopback `127.0.0.1:48997`, CPU affinity 0–1 and the declared native limits.
Copied Production settings had metrics disabled. Each app's process identity,
start ticks, executable hash, affinity, limits and observed status are retained
in `processes.json`. Address space is not RSS; affinity is not a cgroup CPU quota.
There was no installed service, proxy, cloud resource or real traffic.

### Complete business recovery point

The snapshot's exact complete rows were:

| ID | Title | Body | Status | Version |
|---|---|---|---|---|
| 1 | Recovery alpha &lt;quote&gt; | Synthetic body: apostrophe ' &amp; omega Ω | in_progress | 2 |
| 2 | Recovery beta | Synthetic second body | closed | 2 |
| 3 | Recovery gamma | Synthetic third body | open | 1 |

The table DDL and constraints matched `0001_init.sql` exactly; schema version
was 2, `issues_status_id` exactly matched `0002_status_index.sql`, and
`sqlite_sequence` was `[["issues",3]]`. SQLite integrity was `ok`, with no
foreign-key violations. The source before/after paused-writer backup and the
completed backup had identical full manifests. The fresh restore had identical
backup bytes/SHA-256 and complete manifest before startup, after authenticated
HTTP checks and before its first deliberate new write. Inspector observations
do not measure the server's SQLite runtime version, per-connection synchronous
setting or stable-media flush behavior.

`synthetic-loss-decision.json` records exactly two excluded acknowledged
mutations: ID 1's later status/version became closed/3; ID 4 was created with
title `Excluded acknowledgment marker`, body `Deliberate synthetic post-snapshot
create`, open/version 1. Source sequence became 4. The stopped source's full
manifest preserved those changes through final history comparison. There were
four source creates and three source status updates in total, five mutations
included in the recovery history and two deliberately excluded. There were
**zero unknown outcomes or other excluded mutations**. No retry was performed
and no real-data loss approval was granted.

The controlled client's scheduling latch closed with zero calls in flight;
it is not a server/network request fence. Source SIGTERM then produced exit 0,
`stopped`, no incomplete shutdown and a closed old listener before any candidate
startup. This local zero-in-flight stop does not repeat YT-133's held-request
edge/drain proof. The source's complete data remained intact for reconciliation.

A copied schema-2 candidate with only the status index removed retained valid
SQLite integrity and its business rows. Full operator preflight refused it and
kept cutover closed. A separate deliberate private launch of the exact release
also exited 1 with `starter failed: database_unavailable`, no `started` event and
no listener. Its full manifest remained unchanged. These are distinct operator
and actual app refusals, not a claim that startup detects every data problem.

The valid restored process passed liveness/readiness, authenticated legacy list
and all three detail reads, and authenticated v2 filtered reads for open,
in_progress and closed with exact expected fields and ordering. Only after
these checks was the synthetic fixture eligible for re-entry. One later create
returned ID 4/open/version 1 with title `Fresh recovery history` and body
`ID 4 reuse after explicit synthetic snapshot loss`. This differs from the
excluded source ID 4: old IDs and versions must be refreshed after restore.
The final restore had four exact rows and sequence 4; a clean stop and restart
of the same release preserved its complete manifest, readiness, legacy and v2
HTTP readback. No arbitrary future forward-fix binary was tested.

### Observed timing

| Observation | Milliseconds |
|---|---:|
| Supported SQLite backup start–end interval | 10.63 |
| Backup-start age at source withdrawal, conservative recovery-point proxy | 39.07 |
| Backup-end age at withdrawal | 28.44 |
| Last included acknowledgment age at withdrawal | 46.90 |
| Backup start to last excluded source acknowledgment, separate exposure window | 37.51 |
| Fresh exclusive copy interval | 0.32 |
| RTO: withdrawal through clean source stop, intentional refusal rehearsal, fresh restore and complete eligibility checks | 113.14 |

These use the predeclared single local monotonic-clock boundaries. UTC annotations
are in `timeline.json`; exact arithmetic is in `timings.json`. Actual loss is the
two named mutations, not a rate extrapolation. The RTO includes the deliberate
bad-candidate operator/app refusal, and excludes pre-outage backup, post-restore
mutation/restart, human delay and external traffic switching. Warm local cache,
tiny synthetic data and same-host client/storage materially limit interpretation.
No target, SLO, real DR objective, power-loss durability or production capacity
follows from these small timings.

### Evidence and cleanup

- `reuse-before.json`, `reuse-after.json`, `preexecution.json`: exact unchanged
  input/binary binding and pre-start fixture/command/settings declaration.
- `snapshot-manifest.json`, `source-before-backup.json`,
  `source-after-two-acknowledgments.json`, `source-stopped.json`,
  `source-final-preserved-history.json`, `backup-final.json`: complete history,
  schema/index/sequence/integrity and reconciliation.
- `operator-refusal.json`, `bad-before.json`, `bad-after-app-refusal.json`,
  `refused.log`: distinct operator and actual application refusal.
- `copy-bytes.json`, `restored-prestartup.json`, `restored-after-readback.json`,
  `postrestore-after-one-acknowledgment.json`, `forward-prestart.json`,
  `forward-after-readback.json`: byte equality, exact recovery point and accounted
  post-restore history including same-release forward restart.
- `requests.json`, `timeline.json`, `withdrawal.json`, `synthetic-loss-decision.json`,
  `timings.json`, process logs, `result.json`, `processes.json`, `cleanup.json`:
  raw HTTP/startup/stop/acknowledgment, timing, ownership and teardown evidence.

All four children were reaped: source, restored and forward restart exited 0
with `stopped`; the deliberate refused candidate exited 1. PIDs were absent,
port 48997 was closed, HTTP/SQLite/log handles were closed and no forced cleanup
was required. Only the owned fixture was removed, including its database/WAL/SHM,
configuration and 0600 ephemeral token file; no credential values are retained
in evidence. The source and backup survive as complete synthetic manifest/history
records, not retained database files. No task process or fixture remains.

Changed repository file for this execution: this runbook only. The controller
owns integration/status documents, aggregate English/link/structure and whitespace
checks, independent review and final acceptance. No compile/unit/build/lint,
browser/CSP, Windows, old compatibility matrix, load/soak, edge/proxy, IdP or
Valkey verification was rerun. No open architectural decision or product gap
was found. The next unblocked optional backlog item is YT-136;
YT-141 performance validation remains separately scheduled future work.
