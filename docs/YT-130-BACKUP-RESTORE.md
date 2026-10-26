# YT-130 — Starter backup, fresh restore and operator cutover

Date: 2026-10-23. Status: **Done at bounded local B2 scope after independent Reviewer PASS**.
Scope: Production Starter application procedure, original Phase 2 B2. Depends on
independently accepted [YT-129](YT-129-LOAD-SOAK-RECOVERY.md). This adds no runtime,
schema, dependency, storage engine or Session policy. See the
[Phase 2 contract](PRODUCTION-READINESS-PHASE2.md#yt-130--b2-application-backup-restore-and-recovery).

## Selected configuration and evidence identity

One synthetic localhost exercise used the existing debug executable, Production
settings and an absolute isolated database directory under repository `target/`.
The directory was on `/dev/nvme0n1p2`, ext4; this was a local file recovery exercise,
without a power-loss, disk-loss or backup-media durability test. Source, restored
and rejected candidates ran serially on `127.0.0.1:45199`. Metrics were enabled;
logging was `info`. The unique random API token existed only in process memory.
No real business data or external environment was touched.

Existing policies remained SQL capacity 4/no queue, busy timeout 2 seconds,
HTTP capacity 32/no queue, request deadline 5 seconds, readiness budget 1 second,
shutdown grace 10 seconds, WAL, and Memory Session TTL 1800 seconds/capacity 1024.
The source had no competing writers during backup; two later mutations were
deliberately acknowledged before shutdown to expose snapshot data loss.

Branch: `docs/yt113-123-opt-in-capability-plan`.
HEAD: `5db8b8c61b6d319a641814dff4af493ce3e5d965`.
The checkout already contained accepted T0–T3 changes; HEAD alone does not identify
the exercised application. Executable SHA-256:
`4dc7dec913722dbcf86a8677790e69d20fa27a3676e855e31fd9f238a40fcb29`.

Raw records are retained in ignored directory
`.pipeline-output/prod-readiness-phase2-20261023T020406Z/observations/b2/`:

- `source-environment-settings.json`: settings, environment, dirty state and all
  application source/configuration/migration/lock/test hashes, saved before startup.
- `exact-recovery-command.txt`, `recovery-command.log`: complete one-shot direct
  Python standard-library/HTTP/SQLite commands and successful output. This is a
  retained exercise command, without an installed harness or dependency.
- `snapshot-manifest.json`, `source-after-late-writes.json`,
  `restored-prestartup-manifest.json`, `restored-poststartup-manifest.json`:
  exact schema, all issue fields, sequence, integrity and foreign-key observations.
- `requests.json`, `source.log`, `restored.log`, `refused.log`, `timeline.json`,
  `run-result.json`, `derived-recovery-timings.json`: HTTP/startup/refusal results,
  UTC and monotonic timings; derived timings use arithmetic without another run.
- `after-source-cleanup.json`: unchanged runtime/input/binary hashes, process
  exits/reaping, closed listener/connections and removed temporary data.

## Operator runbook

This historical B2 runbook and its commands target the retained **v1** executable
and `PRAGMA user_version = 1`. The current C1 application uses schema 2; follow
the [C1 version-specific procedure](YT-131-MIXED-VERSION-COMPATIBILITY.md#upgrade-rollback-and-forward-fix-conditions)
for that binary. B2's accepted proof and recovery timings remain v1-only.
The executable initializes a missing/empty database. Therefore **a green readiness
probe alone cannot establish a successful restore**: check a nonempty existing
candidate and the expected business-data manifest before startup. Never point
recovery startup at the old active path or a guessed/missing path.

1. Record the application source/binary identity, selected settings, backup UTC
   interval, schema version, expected critical IDs/fields/versions and approved
   loss window. Supply the API secret separately through the existing operator
   secret source. Keep database, manifests and backups private: manifests contain
   business data. Protection, retention, off-host copies and recovery objectives
   are deployment-owner choices; this exercise does not establish them.
2. Produce a consistent snapshot using SQLite's supported backup API, including
   committed WAL state. Example direct command below requires absolute existing
   `SOURCE_DB` and a new `BACKUP_DB` in a private directory. It refuses an existing
   destination. On any error, do not publish that candidate; retain the active
   database and investigate. Never copy only a live main file or delete WAL/SHM
   files. This exercise paused its sole synthetic writer during the backup.
   With real concurrent writers, use the backup completion interval as uncertainty
   and reconcile against an application-owned acknowledgment record.

   ```bash
   python3 - <<'PY'
   import os, sqlite3
   from pathlib import Path
   from contextlib import closing
   source = Path(os.environ['SOURCE_DB'])
   backup = Path(os.environ['BACKUP_DB'])
   assert source.is_absolute() and source.is_file() and source.stat().st_size > 0
   assert backup.is_absolute() and backup.parent.is_dir()
   with backup.open('xb'):
       os.chmod(backup, 0o600)
   with closing(sqlite3.connect(source.as_uri() + '?mode=ro', uri=True)) as src:
       with closing(sqlite3.connect(backup)) as dst:
           src.backup(dst)
   with closing(sqlite3.connect(backup.as_uri() + '?mode=ro', uri=True)) as check:
       assert check.execute('PRAGMA integrity_check').fetchall() == [('ok',)]
       assert check.execute('PRAGMA foreign_key_check').fetchall() == []
       assert check.execute('PRAGMA user_version').fetchone() == (1,)
   PY
   ```

3. At recovery/cutover, stop incoming writes through the existing operator access
   boundary; disconnect old callers and preserve uncertain-write records. Send
   SIGTERM to the recorded source PID, wait for exit 0 and `stopped`, and confirm
   its listener is closed. Exit 1/`shutdown_incomplete`, an active writer or an
   unknown outcome means **STOPPED**: reconcile database state before proceeding.
   Keep the old data directory intact. Do not leave the source writable while
   directing users to a restored snapshot. Stop the recovery clock at validated
   eligibility only if that is the declared measurement boundary.
4. Create a fresh private empty directory on the selected local volume. Restore
   the completed backup to a new filename using exclusive creation. Never merge
   old WAL/SHM siblings into it. With `BACKUP_DB` and `RESTORE_DB` set explicitly:

   ```bash
   python3 - <<'PY'
   import hashlib, os, shutil, sqlite3
   from pathlib import Path
   from contextlib import closing
   backup, restored = (Path(os.environ[k]) for k in ('BACKUP_DB', 'RESTORE_DB'))
   assert backup.is_absolute() and backup.is_file() and backup.stat().st_size > 0
   assert restored.is_absolute() and restored.parent.is_dir()
   assert list(restored.parent.iterdir()) == []
   with backup.open('rb') as source, restored.open('xb') as destination:
       shutil.copyfileobj(source, destination)
   os.chmod(restored, 0o600)
   assert hashlib.sha256(backup.read_bytes()).digest() == hashlib.sha256(restored.read_bytes()).digest()
   observations = []
   for path in (backup, restored):
       with closing(sqlite3.connect(path.as_uri() + '?mode=ro', uri=True)) as c:
           assert c.execute('PRAGMA integrity_check').fetchall() == [('ok',)]
           assert c.execute('PRAGMA foreign_key_check').fetchall() == []
           assert c.execute('PRAGMA user_version').fetchone() == (1,)
           observations.append((
               c.execute('SELECT type,name,tbl_name,sql FROM sqlite_schema ORDER BY name').fetchall(),
               c.execute('SELECT id,title,body,status,version FROM issues ORDER BY id').fetchall(),
               c.execute('SELECT name,seq FROM sqlite_sequence ORDER BY name').fetchall()))
   assert observations[0] == observations[1]
   print('Copy, integrity, version and complete snapshot equality: PASS')
   PY
   ```

   Equality proves transfer against that backup, not that the backup was the
   intended business recovery point. Compare its manifest to recorded expected
   business data and approved loss before allowing startup. Any missing file,
   corruption, unsupported schema, unexpected row/version/status, or unacceptable
   loss means **STOPPED**; preserve evidence and keep traffic disabled. Do not
   change `user_version`, suppress an error or initialize an empty replacement to
   make a candidate pass.
5. Start the verified existing executable with `APP_ENVIRONMENT=Production`, the
   explicit absolute `YETLET_CONFIG_ROOT`, `APP_DATABASE__PATH="$RESTORE_DB"`,
   an isolated loopback port and the existing secret source. Observe startup exit
   and `started`. Check `GET /health/live` and `GET /health/ready` are both 200;
   authenticated `GET /api/issues/{id}` must reproduce the selected critical
   title/body/status/version values. Recheck schema/integrity and complete data
   against the saved snapshot. The exact exercised startup/HTTP commands are in
   the retained direct command; no CLI flags or application restore mode were added.
6. Switch the existing operator access boundary only after all checks pass,
   the old writer is stopped and the operator accepts the measured missing writes.
   This exercise reached local **eligible** state; it did not change a proxy/DNS
   or any external service. Have callers discard old pages, cached IDs/versions
   and CSRF state and reload. A restored AUTOINCREMENT sequence can reuse an ID
   that existed only after the backup; stale client assumptions are unsafe.
   Never blindly replay uncertain creates or updates. After a new write to the
   restored instance, returning to the old source requires reconciliation of both
   histories; it is not an automatic safe rollback.
7. Stop the rehearsal instance, wait/reap, confirm its listener is closed, close
   SQLite/HTTP clients and remove only rehearsal data. Keep approved backups and
   the old production database under the existing retention policy. A rejected
   candidate stays out of service. Record actual recovery/loss and any omissions.

## Actual fresh restore, loss and refusal

The supported Python `sqlite3.Connection.backup` snapshot was taken while the
source was live and its sole writer idle. Snapshot SHA-256 was
`e4be5652b3cb416e7f65555ee1d40cb592eff0d5d574bd2c089927c527fffd5b`.
Three complete issue records and the full SQL schema matched before and after
restore startup; `integrity_check` returned `ok`, foreign-key checks were empty,
schema version was 1 and `sqlite_sequence` was 3.

| Issue | Recovered status | Recovered version | HTTP readback |
|---|---|---|---|
| 1 / B2 alpha | `in_progress` | 2 | Exact title/body/status/version, 200 |
| 2 / B2 beta | `closed` | 2 | Exact title/body/status/version, 200 |
| 3 / B2 gamma | `open` | 1 | Exact title/body/status/version, 200 |

Both health endpoints returned 200. The later source create, ID 4, returned 404
on restore; the later source update of issue 1 to `closed`/version 3 was absent.
These were **two acknowledged mutations lost** by selecting that snapshot.
A controlled postrestore create succeeded as ID 4/version 1, demonstrating the
recovered sequence and also the documented possibility of reusing discarded IDs.

UTC timestamps below are all on 2026-10-23; durations use the corresponding
monotonic clock, avoiding UTC clock adjustments:

| Boundary | UTC |
|---|---|
| Last acknowledgment included in snapshot | 04:29:23.009058 |
| Backup start | 04:29:23.019106 |
| Backup complete | 04:29:23.028872 |
| Last acknowledged excluded mutation | 04:29:23.058371 |
| Outage starts before source SIGTERM | 04:29:23.060407 |
| Source exit/listener closed | 04:29:23.064009 |
| Fresh copy/schema/integrity/data preflight complete | 04:29:23.068181 |
| Restored readiness and business checks complete; local cutover eligible | 04:29:23.112772 |

Measured **RPO is 41.30 ms**, defined here as age of the backup-start recovery-point
proxy at outage. The consistent snapshot lies within the 9.77 ms backup interval;
using its start bounds its age conservatively (backup-end age was 31.54 ms).
The last included acknowledgment was 51.35 ms before outage; precise SQL commit
timestamps were not collected. The retained raw `RPO.exposure_seconds` is a
separate 39.26 ms acknowledged-write window from backup start to last source
acknowledgment, not the outage-to-recovery-point age. Actual loss was the two
specific mutations above, not a rate extrapolation or a zero-loss promise.

Measured **RTO is 52.36 ms**, from outage start immediately before source SIGTERM
through clean source stop, fresh copy, integrity/schema/data preflight, restored
startup, liveness/readiness and exact critical business readback to local operator
cutover eligibility. It excludes backup creation before outage, failure rehearsal
afterward, human delay, proxy/DNS switching and client reconnection. These small
synthetic measurements are observations, not deployment targets or guarantees.

For failure evidence, a separate fresh candidate copied the same snapshot and
changed only `user_version` to 999. SQLite integrity remained `ok`; the operator
schema check explicitly stopped cutover. Starting the existing application on
that rejected candidate exited **1**, with fixed error
`starter failed: database_unavailable`, no `started` event and no listener. This
demonstrates actual startup refusal in addition to operator STOPPED state; it
does not claim every corruption pattern is caught automatically at startup.

## Session and Valkey policy retained

The Starter selects Memory Session for anonymous SSR CSRF protection, not login
identity. Its sessions are absent from SQLite and are all lost on process stop.
The original cookie plus original CSRF token returned 403 after restore startup;
users must load a fresh page. There is no Starter login to restore or revoke.
The separate Bearer API secret stays operator-supplied and outside the backup;
restoring the database neither rotates nor recovers it.

The [YT-066 responsibility boundary](YT-066-PERSISTENCE-RECOVERY-PLAN.md) and
[YT-094 Valkey evidence](YT-094-VALKEY-SESSIONS.md), summarized in
[YT-094 closeout](YT-094-CLOSEOUT.md), retain their original single-host scope.
Admin explicitly selecting nonpersistent Valkey can retain sessions through an
application restart; nonpersistent Valkey restart loses them and requires login.
That does not give this Starter Valkey integration or durable Session restore.
Restoring an older persistent Session store can resurrect revoked identities;
follow the existing separately selected deployment fencing/invalidation contract
before exposure. This task neither selects persistent Valkey nor invents a new
revocation policy. No shared provider/SQLite qualification was rerun.

## Validation, cleanup and next item

The exact direct exercise command exited 0. Source/restored processes exited 0;
the intentional refusal exited 1. All three PIDs were reaped and absent, port
45199 was closed, HTTP/SQL connections and logs were closed, and the isolated
`target/yt130-b2-*` directory including all WAL/SHM files was removed. Source,
settings, migration, lockfile, tests and binary hashes were identical before/after.
Accepted A2 focused test/build/lint evidence and B1 measurements remain bounded
to unchanged inputs; this documentation slice did not repeat them or browser/CSP,
provider, asset, load or engine qualification. New verification is the application
recovery exercise above, `cargo xtask check docs` and `git diff --check`.

Changed repository files: this runbook, the Starter README, and only relevant
status/evidence entries in `BACKLOG.md`, `ROADMAP.md` and the Phase 2 document.
Independent Reviewer PASS has zero issues and required followups; controller acceptance completes YT-130 at bounded local B2 scope.
YT-131 has subsequently been independently accepted at local C1 scope. The next
remaining decision is **YT-096 D1–D6**, Pending. Candidate `ce56e08`,
YT-096 In progress/D1–D6 Pending and YT-020 Blocked remain unchanged.

Unqualified: real data scale, release binary, concurrent-write backup timing,
power-loss/storage durability, off-host media, retention/protection operations,
restore of persistent Session providers, cross-host DR/HA, deployment RPO/RTO,
external traffic cutover and automatic rollback. No new architecture or
security/data-policy decision was needed for this bounded procedure.

Independent review record: `.pipeline-output/prod-readiness-phase2-20261023T020406Z/observations/T4/independent-review.json`. Source hashes were stable during review; controller edits only record acceptance.
