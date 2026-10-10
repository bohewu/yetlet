#!/usr/bin/env python3
"""Focused standard-library tests for the Chrome E2E process harness."""
from __future__ import annotations

import errno
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import time
import unittest
from unittest import mock

import smoke_browser


class BrowserHarnessTests(unittest.TestCase):
    def test_profile_argument_selects_debug_or_release(self) -> None:
        self.assertEqual(smoke_browser.parse_args([]).profile, "debug")
        self.assertEqual(smoke_browser.parse_args(["--profile", "release"]).profile, "release")

    @unittest.skipUnless(os.name == "posix", "process-group cleanup is POSIX-specific")
    def test_owned_process_group_is_terminated_and_reaped(self) -> None:
        process = subprocess.Popen(
            [
                sys.executable,
                "-c",
                "import subprocess,sys,time; "
                "subprocess.Popen([sys.executable,'-c','import time; time.sleep(60)']); "
                "time.sleep(60)",
            ],
            start_new_session=True,
        )
        smoke_browser.stop_owned_process(process, process_group=True)
        self.assertIsNotNone(process.poll())
        deadline = time.monotonic() + 3
        while time.monotonic() < deadline:
            try:
                os.killpg(process.pid, 0)
            except ProcessLookupError:
                break
            time.sleep(0.05)
        else:
            self.fail("owned Chrome-style process group remained after cleanup")

    def test_owned_tree_retries_transient_not_empty_error(self) -> None:
        with tempfile.TemporaryDirectory(prefix="yetlet-harness-test-") as directory:
            profile = Path(directory) / "profile"
            profile.mkdir()
            (profile / "state").write_text("test")
            real_rmtree = smoke_browser.shutil.rmtree
            calls = 0

            def flaky_rmtree(path):
                nonlocal calls
                calls += 1
                if calls == 1:
                    raise OSError(errno.ENOTEMPTY, "transient Chrome write")
                return real_rmtree(path)

            with mock.patch.object(smoke_browser.shutil, "rmtree", side_effect=flaky_rmtree):
                smoke_browser.remove_owned_tree(profile, seconds=1)
            self.assertGreaterEqual(calls, 2)
            self.assertFalse(profile.exists())


if __name__ == "__main__":
    unittest.main()
