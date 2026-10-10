#!/usr/bin/env python3
"""YT-066: one bounded local experiment using prebuilt samples; no provider model."""
from __future__ import annotations

import argparse
from concurrent.futures import ThreadPoolExecutor
from contextlib import closing
import hashlib
import json
import os
from pathlib import Path
import signal
import sqlite3
import subprocess
import sys
import tempfile
from threading import Barrier
import time
import tomllib

sys.dont_write_bytecode = True
import smoke_auth_notes as auth
import smoke_sqlite_crud as sql

ROOT = Path(__file__).resolve().parent.parent
SCRATCH = ROOT / "target/m10-remaining"
REPORT = SCRATCH / "local-operations.json"
AUTH = ROOT / "target/debug/yetlet-auth-notes"
SQLITE = ROOT / "target/debug/yetlet-sqlite-crud"
NOT_VERIFIED = ["distributed_provider", "multinode", "high_availability",
                "performance_SLO", "provider_outage", "redis_failover", "load",
                "real_OIDC", "key_rotation", "production_RPO_RTO"]


def require(condition: bool) -> None:
    # Never attach response bodies, cookies, codes, paths or exception text.
    if not condition:
        raise AssertionError("local operation expectation failed")


class Processes:
    """Own only directly launched sample processes, including failed startups."""

    def __init__(self) -> None:
        self.owned: list[subprocess.Popen] = []

    def start(self, binary: Path, overrides: dict[str, str]) -> subprocess.Popen:
        process = subprocess.Popen([str(binary)], cwd=ROOT,
                                   env=dict(os.environ, **overrides),
                                   stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        self.owned.append(process)  # Registered before any readiness operation.
        return process

    def ready(self, process: subprocess.Popen, base: str, *, corrupt: bool = False) -> None:
        deadline = time.monotonic() + 10
        while time.monotonic() < deadline:
            if process.poll() is not None:
                if corrupt:
                    require(process.wait(timeout=1) != 0)
                    return
                raise RuntimeError("startup exited")
            try:
                status, _, _ = sql.call(base, "GET", "/healthz")
                if status == 200:
                    require(not corrupt)
                    return
            except OSError:
                pass
            time.sleep(0.05)
        raise TimeoutError("startup deadline")

    def stop(self, process: subprocess.Popen) -> None:
        if process.poll() is None:
            process.send_signal(signal.SIGINT)
            try:
                process.wait(timeout=5)
            except subprocess.TimeoutExpired:
                process.kill()
                process.wait(timeout=3)
        else:
            process.wait(timeout=1)

    def cleanup(self) -> bool:
        succeeded = True
        for process in reversed(self.owned):
            try:
                self.stop(process)
            except (Exception, KeyboardInterrupt):
                succeeded = False
        return succeeded and all(process.poll() is not None for process in self.owned)


def launch(owned: Processes, binary: Path, **settings: str) -> tuple[subprocess.Popen, str]:
    port = sql.free_port()
    listen_key = "YETLET_AUTH_LISTEN" if binary == AUTH else "YETLET_SQLITE_LISTEN"
    process = owned.start(binary, dict(settings, **{listen_key: f"127.0.0.1:{port}"}))
    base = f"http://127.0.0.1:{port}"
    owned.ready(process, base)
    return process, base


def private_status(base: str, cookie: str, expected: int) -> None:
    status, headers, _ = auth.call(base, "GET", "/auth/notes", cookie)
    require(status == expected and headers.get("cache-control") == "no-store")


def session_scenario(owned: Processes, folder: Path) -> dict:
    del folder  # All Session state is in the real sample's separate memory maps.
    settings = {"YETLET_AUTH_ALICE_CODE": auth.ALICE_CODE,
                "YETLET_AUTH_BOB_CODE": auth.BOB_CODE}
    first, base_a = launch(owned, AUTH, **settings)
    second, base_b = launch(owned, AUTH, **settings)
    # Reuse the real login/cookie/CSRF helpers; no cookies/tokens enter evidence.
    cookie_a, _, _, _ = auth.login(base_a, "alice", auth.ALICE_CODE)
    private_status(base_a, cookie_a, 200)
    private_status(base_b, cookie_a, 401)
    cookie_b, _, _, _ = auth.login(base_b, "alice", auth.ALICE_CODE)
    require(cookie_a != cookie_b)
    private_status(base_b, cookie_b, 200)
    private_status(base_a, cookie_b, 401)
    private_status(base_a, cookie_a, 200)
    owned.stop(first)
    _, restarted = launch(owned, AUTH, **settings)
    private_status(restarted, cookie_a, 401)
    private_status(base_b, cookie_b, 200)
    owned.stop(second)
    return {"status": "pass-for-local", "same_cookie_worker_statuses": [200, 401],
            "separate_worker_cookie_statuses": [200, 401], "cookies_differ": True,
            "old_cookie_after_restart": 401, "private_responses_no_store": True,
            "limitation": "Session is process-local, not shared, and lost on restart"}


def read_database(path: Path) -> tuple:
    with closing(sqlite3.connect(path.as_uri() + "?mode=ro", uri=True, timeout=0.2)) as db:
        require(db.execute("PRAGMA integrity_check").fetchall() == [("ok",)])
        require(db.execute("PRAGMA foreign_key_check").fetchall() == [])
        version = db.execute("PRAGMA user_version").fetchone()[0]
        require(version == 1)
        notes = db.execute("SELECT * FROM notes ORDER BY id").fetchall()
        audit = db.execute("SELECT * FROM note_audit ORDER BY id").fetchall()
        return version, notes, audit


def online_backup(source: Path, destination: Path) -> None:
    # Destination is a new, owned candidate. No raw copy of a live WAL database.
    fd = os.open(destination, os.O_CREAT | os.O_EXCL | os.O_WRONLY, 0o600)
    os.close(fd)
    deadline = time.monotonic() + 5

    def progress(status: int, remaining: int, total: int) -> None:
        del status, remaining, total
        if time.monotonic() >= deadline:
            raise TimeoutError("backup deadline")

    with closing(sqlite3.connect(source.as_uri() + "?mode=ro", uri=True, timeout=0.2)) as src:
        with closing(sqlite3.connect(destination, timeout=0.2)) as dst:
            src.backup(dst, pages=16, progress=progress, sleep=0.01)


def sqlite_scenario(owned: Processes, folder: Path) -> dict:
    database = folder / "live.sqlite3"
    first, base_a = launch(owned, SQLITE, YETLET_SQLITE_PATH=str(database))
    second, base_b = launch(owned, SQLITE, YETLET_SQLITE_PATH=str(database))
    status, headers, body = sql.call(base_a, "POST", "/api/notes",
                                     {"title": "Checkpoint note", "body": "initial"})
    require(status == 201 and headers.get("cache-control") == "no-store")
    created = sql.decoded(body)
    require(created["id"] == 1 and created["version"] == 1)
    for base in (base_a, base_b):
        status, _, body = sql.call(base, "GET", "/api/notes/1")
        require(status == 200 and sql.decoded(body) == created)
    barrier = Barrier(2, timeout=2)

    def update(base: str, title: str) -> tuple:
        barrier.wait()
        return sql.call(base, "PUT", "/api/notes/1",
                        {"title": title, "body": "winner", "expected_version": 1})

    with ThreadPoolExecutor(max_workers=2) as executor:
        futures = [executor.submit(update, base_a, "Writer A"),
                   executor.submit(update, base_b, "Writer B")]
        results = [future.result(timeout=8) for future in futures]
    require(sorted(result[0] for result in results) == [200, 409])
    winner = sql.decoded(next(body for status, _, body in results if status == 200))
    require(winner["version"] == 2)
    for base in (base_a, base_b):
        status, _, body = sql.call(base, "GET", "/api/notes/1")
        require(status == 200 and sql.decoded(body) == winner)
    state = read_database(database)
    require(len(state[1]) == 1 and len(state[2]) == 2)
    require([(row[2], row[3]) for row in state[2]] == [("create", 1), ("update", 2)])
    with closing(sqlite3.connect(database, timeout=0.2)) as db:
        require(db.execute("PRAGMA journal_mode").fetchone()[0] == "wal")
    checkpoint = folder / "checkpoint.sqlite3"
    online_backup(database, checkpoint)  # No mutations occur during this call.
    require(read_database(checkpoint) == state)
    checkpoint_digest = hashlib.sha256(checkpoint.read_bytes()).digest()
    status, _, body = sql.call(base_b, "POST", "/api/notes",
                               {"title": "Late note", "body": "after backup"})
    require(status == 201 and sql.decoded(body)["id"] == 2)
    owned.stop(first)
    owned.stop(second)
    restored = folder / "restored.sqlite3"
    online_backup(checkpoint, restored)
    require(read_database(restored) == state)
    restored_process, restored_base = launch(owned, SQLITE, YETLET_SQLITE_PATH=str(restored))
    status, _, body = sql.call(restored_base, "GET", "/api/notes")
    require(status == 200 and sql.decoded(body) == {"items": [winner], "count": 1})
    require(sql.call(restored_base, "GET", "/api/notes/2")[0] == 404)
    require(read_database(restored) == state)
    owned.stop(restored_process)
    corrupt = folder / "corrupt-candidate.sqlite3"
    online_backup(checkpoint, corrupt)
    with corrupt.open("r+b") as damaged:
        damaged.write(b"invalid SQLite header")
    port = sql.free_port()
    bad_process = owned.start(SQLITE, {"YETLET_SQLITE_PATH": str(corrupt),
                                      "YETLET_SQLITE_LISTEN": f"127.0.0.1:{port}"})
    owned.ready(bad_process, f"http://127.0.0.1:{port}", corrupt=True)
    require(read_database(checkpoint) == state)
    require(hashlib.sha256(checkpoint.read_bytes()).digest() == checkpoint_digest)
    require(all(path.stat().st_mode & 0o777 == 0o600
                for path in (checkpoint, restored, corrupt)))
    return {"status": "pass-for-local", "scope": "two processes, one local WAL file",
            "concurrent_update_statuses": [200, 409], "final_version": 2,
            "checkpoint_notes": 1, "checkpoint_audit_rows": 2, "user_version": 1,
            "integrity_check": "ok", "foreign_key_violations": 0,
            "exact_checkpoint_restored": True, "late_write_absent": True,
            "corrupt_candidate_nonzero_before_health": True,
            "original_checkpoint_unchanged": True, "candidate_mode": "0600"}


def binary_identity(binary: Path, sample: str) -> dict:
    # These binaries expose no version endpoint/flag. Source version is labeled.
    manifest = tomllib.loads((ROOT / "examples" / sample / "Cargo.toml").read_text())
    version = manifest["package"]["version"]
    if isinstance(version, dict):
        version = tomllib.loads((ROOT / "Cargo.toml").read_text())["workspace"]["package"]["version"]
    return {"binary_sha256": hashlib.sha256(binary.read_bytes()).hexdigest(),
            "checkout_package_version": version, "observed_binary_version": "not_exposed",
            "application_SQLite_version": "not_exposed" if sample == "sqlite-crud" else "not_applicable"}


def write_report(report: dict) -> None:
    # Called only after process and owned-directory cleanup have been attempted.
    with tempfile.NamedTemporaryFile(mode="w", dir=SCRATCH, prefix="local-report-",
                                     delete=False) as output:
        path = Path(output.name)
        try:
            json.dump(report, output, indent=2)
            output.write("\n")
            output.flush()
            os.replace(path, REPORT)
        finally:
            path.unlink(missing_ok=True)


def run() -> int:
    started = time.monotonic()
    report = {"task_id": "YT-066", "status": "failed-for-local",
              "scope": "prebuilt samples, loopback, synthetic data, local filesystem only",
              "qualification": {name: "not_verified" for name in NOT_VERIFIED},
              "python_SQLite_version": sqlite3.sqlite_version, "scenarios": {}}
    owned = Processes()
    temporary = None
    phase = "preflight"
    outcome = "none"
    cleanup_ok = False
    SCRATCH.mkdir(parents=True, exist_ok=True)
    try:
        require(AUTH.is_file() and SQLITE.is_file())
        report["applications"] = {"auth_notes": binary_identity(AUTH, "auth-notes"),
                                  "sqlite_crud": binary_identity(SQLITE, "sqlite-crud")}
        temporary = tempfile.TemporaryDirectory(prefix="local-operations-", dir=SCRATCH)
        folder = Path(temporary.name)
        require(folder.stat().st_mode & 0o777 == 0o700)
        for name, scenario in (("session_isolation_restart", session_scenario),
                               ("sqlite_concurrency_backup_restore", sqlite_scenario)):
            phase = name
            scenario_start = time.monotonic()
            result = scenario(owned, folder)
            result["duration_seconds"] = round(time.monotonic() - scenario_start, 3)
            report["scenarios"][name] = result
        outcome = "success"
    except KeyboardInterrupt:
        outcome = "cancelled"
    except Exception:
        outcome = "expectation_or_operation_failed"
    finally:
        # Ignore further cancellation only during bounded teardown.
        previous = {}
        for sig in (signal.SIGINT, signal.SIGTERM):
            previous[sig] = signal.signal(sig, signal.SIG_IGN)
        try:
            cleanup_ok = owned.cleanup()
            if temporary is not None:
                try:
                    temporary.cleanup()
                    cleanup_ok = cleanup_ok and not Path(temporary.name).exists()
                except Exception:
                    cleanup_ok = False
        finally:
            for sig, handler in previous.items():
                signal.signal(sig, handler)
    report["cleanup"] = {"status": "pass" if cleanup_ok else "failed",
                         "owned_processes": len(owned.owned),
                         "processes_reaped": all(p.poll() is not None for p in owned.owned),
                         "temporary_directory_removed": temporary is None or not Path(temporary.name).exists()}
    report["outcome"] = outcome
    report["last_phase"] = phase
    report["duration_seconds"] = round(time.monotonic() - started, 3)
    if outcome == "success" and cleanup_ok:
        report["status"] = "pass-for-local"
    write_report(report)
    return 0 if report["status"] == "pass-for-local" else 1


def main(argv: list[str] | None = None) -> int:
    argparse.ArgumentParser(description=__doc__).parse_args(argv)

    def cancel(signum, frame):
        del signum, frame
        raise KeyboardInterrupt

    previous = signal.signal(signal.SIGTERM, cancel)
    try:
        result = run()
        print("local operations: pass-for-local" if result == 0 else "local operations: failed-for-local")
        return result
    except (Exception, KeyboardInterrupt):
        print("local operations: report unavailable", file=sys.stderr)
        return 1
    finally:
        signal.signal(signal.SIGTERM, previous)


if __name__ == "__main__":
    sys.exit(main())
