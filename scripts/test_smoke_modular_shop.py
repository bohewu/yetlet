#!/usr/bin/env python3
"""Focused cleanup regressions for the Modular Shop smoke harness."""
from __future__ import annotations

import importlib.util
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch

ROOT = Path(__file__).resolve().parent.parent
SPEC = importlib.util.spec_from_file_location(
    "smoke_modular_shop", ROOT / "scripts" / "smoke_modular_shop.py"
)
assert SPEC is not None and SPEC.loader is not None
SMOKE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(SMOKE)


class FakeProcess:
    def __init__(self, *, timeout_once: bool = False) -> None:
        self.returncode = None
        self.timeout_once = timeout_once
        self.terminated = False
        self.killed = False
        self.wait_calls: list[int] = []
        self.stdout = None

    def poll(self):
        return self.returncode

    def terminate(self) -> None:
        self.terminated = True

    def kill(self) -> None:
        self.killed = True
        self.returncode = -9

    def wait(self, timeout: int):
        self.wait_calls.append(timeout)
        if self.timeout_once:
            self.timeout_once = False
            raise subprocess.TimeoutExpired("yetlet-modular-shop", timeout)
        if self.returncode is None:
            self.returncode = -15
        return self.returncode


class FailedStartCleanupTests(unittest.TestCase):
    def exercise_failure(self, *, timeout_once: bool) -> FakeProcess:
        process = FakeProcess(timeout_once=timeout_once)

        def fake_popen(*args, **kwargs):
            process.stdout = kwargs["stdout"]
            return process

        readiness_error = RuntimeError("simulated readiness failure")
        with tempfile.TemporaryDirectory(prefix="yetlet-shop-start-failure-") as folder:
            database = Path(folder) / "shop.sqlite3"
            with (
                patch.object(SMOKE, "free_port", return_value=39008),
                patch.object(SMOKE, "client", return_value=object()),
                patch.object(SMOKE.subprocess, "Popen", side_effect=fake_popen),
                patch.object(SMOKE, "wait_ready", side_effect=readiness_error),
            ):
                with self.assertRaisesRegex(RuntimeError, "simulated readiness failure") as raised:
                    SMOKE.start(database)

        self.assertIs(raised.exception, readiness_error)
        self.assertTrue(process.terminated)
        self.assertIsNotNone(process.stdout)
        self.assertTrue(process.stdout.closed)
        self.assertIsNotNone(process.returncode)
        return process

    def test_readiness_failure_terminates_reaps_and_closes_stream(self) -> None:
        process = self.exercise_failure(timeout_once=False)
        self.assertFalse(process.killed)
        self.assertEqual(process.wait_calls, [4])

    def test_readiness_failure_kills_after_terminate_timeout(self) -> None:
        process = self.exercise_failure(timeout_once=True)
        self.assertTrue(process.killed)
        self.assertEqual(process.wait_calls, [4, 4])
        self.assertEqual(process.returncode, -9)


if __name__ == "__main__":
    unittest.main()
