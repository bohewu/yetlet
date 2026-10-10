#!/usr/bin/env python3
"""Direct YT-066 lifecycle, redaction and backup tests; no sample servers start."""
import io
import json
from pathlib import Path
import sqlite3
import subprocess
import sys
import tempfile
import unittest
from contextlib import closing, redirect_stderr, redirect_stdout
from unittest.mock import MagicMock, patch

sys.dont_write_bytecode = True
import qualify_local_operations as operations


class LocalOperationsTests(unittest.TestCase):
    def setUp(self):
        operations.SCRATCH.mkdir(parents=True, exist_ok=True)
        self.temporary = tempfile.TemporaryDirectory(prefix="operations-test-", dir=operations.SCRATCH)
        self.addCleanup(self.temporary.cleanup)
        self.folder = Path(self.temporary.name)
        self.scratch = self.folder / "evidence"
        self.report_path = self.scratch / "local-operations.json"
        self.patches = [patch.object(operations, "SCRATCH", self.scratch),
                        patch.object(operations, "REPORT", self.report_path),
                        patch.object(operations, "binary_identity", return_value={"observed_binary_version": "not_exposed"}),
                        patch.object(Path, "is_file", return_value=True)]
        for item in self.patches:
            item.start()
            self.addCleanup(item.stop)

    def assert_clean(self):
        self.assertEqual(list(self.scratch.glob("local-operations-*")), [])
        self.assertEqual(list(self.scratch.glob("local-report-*")), [])

    def test_registered_before_failed_readiness_and_reaped(self):
        process = MagicMock()
        process.poll.return_value = None
        process.wait.side_effect = lambda **kwargs: setattr(process.poll, "return_value", -2) or -2
        owned = operations.Processes()

        def readiness(*args, **kwargs):
            self.assertEqual(owned.owned, [process])
            raise RuntimeError("startup fixture")

        with patch.object(operations.sql, "free_port", return_value=12345), \
                patch.object(operations.subprocess, "Popen", return_value=process), \
                patch.object(owned, "ready", side_effect=readiness):
            with self.assertRaises(RuntimeError):
                operations.launch(owned, operations.AUTH)
        self.assertTrue(owned.cleanup())
        process.send_signal.assert_called_once_with(operations.signal.SIGINT)
        process.wait.assert_called_once_with(timeout=5)

    def test_stop_escalates_timeout_to_kill_and_wait(self):
        process = MagicMock()
        process.poll.return_value = None
        process.wait.side_effect = [subprocess.TimeoutExpired("fixture", 5), -9]
        operations.Processes().stop(process)
        process.kill.assert_called_once_with()
        self.assertEqual([call.kwargs["timeout"] for call in process.wait.call_args_list], [5, 3])

    def test_all_cleanup_attempted_after_one_failure(self):
        owned = operations.Processes()
        first, second = MagicMock(), MagicMock()
        owned.owned = [first, second]
        with patch.object(owned, "stop", side_effect=[RuntimeError("fixture"), None]) as stop:
            self.assertFalse(owned.cleanup())
        self.assertEqual([call.args[0] for call in stop.call_args_list], [second, first])

    def test_readiness_deadline_and_corrupt_candidate(self):
        owned = operations.Processes()
        process = MagicMock()
        process.poll.return_value = None
        with patch.object(operations.time, "monotonic", side_effect=[0, 11]):
            with self.assertRaises(TimeoutError):
                owned.ready(process, "http://127.0.0.1:12345")
        process.poll.return_value = 1
        process.wait.return_value = 1
        owned.ready(process, "http://127.0.0.1:12345", corrupt=True)
        process.poll.return_value = None
        with patch.object(operations.sql, "call", return_value=(200, {}, b"")):
            with self.assertRaises(AssertionError):
                owned.ready(process, "http://127.0.0.1:12345", corrupt=True)

    def test_spawn_failure_redacted_after_cleanup(self):
        fixture_message = "redaction-test-only: cookie, csrf, code and log payload"
        first = MagicMock()
        first.poll.return_value = None
        first.wait.side_effect = lambda **kwargs: setattr(first.poll, "return_value", 0) or 0

        def scenario(owned, folder):
            owned.start(operations.AUTH, {})
            owned.start(operations.AUTH, {})

        output = io.StringIO()
        with patch.object(operations.subprocess, "Popen", side_effect=[first, OSError(fixture_message)]), \
                patch.object(operations, "session_scenario", side_effect=scenario), \
                redirect_stdout(output), redirect_stderr(output):
            self.assertEqual(operations.main([]), 1)
        report = json.loads(self.report_path.read_text())
        self.assertEqual(report["cleanup"]["status"], "pass")
        self.assertEqual(report["cleanup"]["owned_processes"], 1)
        self.assertNotIn(fixture_message, self.report_path.read_text() + output.getvalue())
        first.send_signal.assert_called_once()
        self.assert_clean()

    def test_cancel_after_spawn_still_reaps_and_reports(self):
        process = MagicMock()
        process.poll.return_value = None
        process.wait.side_effect = lambda **kwargs: setattr(process.poll, "return_value", -2) or -2

        def scenario(owned, folder):
            owned.start(operations.AUTH, {})
            raise KeyboardInterrupt

        with patch.object(operations.subprocess, "Popen", return_value=process), \
                patch.object(operations, "session_scenario", side_effect=scenario):
            self.assertEqual(operations.run(), 1)
        report = json.loads(self.report_path.read_text())
        self.assertEqual(report["outcome"], "cancelled")
        self.assertEqual(report["cleanup"]["status"], "pass")
        self.assert_clean()

    def test_success_report_written_only_after_cleanup(self):
        original = operations.write_report

        def write(report):
            self.assert_clean()
            self.assertEqual(report["cleanup"]["status"], "pass")
            original(report)

        with patch.object(operations, "session_scenario", return_value={"status": "pass-for-local"}), \
                patch.object(operations, "sqlite_scenario", return_value={"status": "pass-for-local"}), \
                patch.object(operations, "write_report", side_effect=write):
            self.assertEqual(operations.run(), 0)
        report = json.loads(self.report_path.read_text())
        self.assertEqual(report["status"], "pass-for-local")
        self.assertTrue(all(value == "not_verified" for value in report["qualification"].values()))

    def test_cleanup_failure_prevents_pass_report(self):
        with patch.object(operations, "session_scenario", return_value={"status": "pass-for-local"}), \
                patch.object(operations, "sqlite_scenario", return_value={"status": "pass-for-local"}), \
                patch.object(operations.Processes, "cleanup", return_value=False):
            self.assertEqual(operations.run(), 1)
        report = json.loads(self.report_path.read_text())
        self.assertEqual(report["status"], "failed-for-local")
        self.assertEqual(report["cleanup"]["status"], "failed")
        self.assert_clean()

    def test_online_backup_actual_stdlib_checkpoint_and_no_overwrite(self):
        source, checkpoint, restored = [self.folder / name for name in ("source.db", "backup.db", "restore.db")]
        with closing(sqlite3.connect(source)) as db:
            db.execute("PRAGMA journal_mode=WAL")
            db.execute("CREATE TABLE fixture(value TEXT)")
            db.execute("INSERT INTO fixture VALUES ('checkpoint')")
            db.commit()
            operations.online_backup(source, checkpoint)
            db.execute("INSERT INTO fixture VALUES ('late')")
            db.commit()
            operations.online_backup(checkpoint, restored)
        with closing(sqlite3.connect(restored)) as db:
            self.assertEqual(db.execute("SELECT * FROM fixture").fetchall(), [("checkpoint",)])
        self.assertEqual(checkpoint.stat().st_mode & 0o777, 0o600)
        with self.assertRaises(FileExistsError):
            operations.online_backup(source, checkpoint)

    def test_backup_progress_deadline_interrupts_busy_retry(self):
        source = MagicMock()
        destination = MagicMock()

        def backup(dst, **kwargs):
            self.assertEqual(kwargs["pages"], 16)
            self.assertEqual(kwargs["sleep"], 0.01)
            kwargs["progress"](sqlite3.SQLITE_BUSY, 1, 1)

        source.backup.side_effect = backup
        with patch.object(operations.sqlite3, "connect", side_effect=[source, destination]), \
                patch.object(operations.time, "monotonic", side_effect=[0, 6]):
            with self.assertRaises(TimeoutError):
                operations.online_backup(self.folder / "source.db", self.folder / "deadline.db")
        source.close.assert_called_once()
        destination.close.assert_called_once()


if __name__ == "__main__":
    unittest.main()
