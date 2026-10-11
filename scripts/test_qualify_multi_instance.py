#!/usr/bin/env python3
"""Focused source, redaction and cleanup tests for the YT-066a Docker harness."""
from __future__ import annotations

import json
import signal
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch

sys.dont_write_bytecode = True
import qualify_multi_instance as multi


class MultiInstanceHarnessTests(unittest.TestCase):
    def test_last_upstream_uses_successful_retry(self):
        self.assertEqual(multi.last_upstream("10.0.0.2:3010"), "10.0.0.2:3010")
        self.assertEqual(
            multi.last_upstream("10.0.0.1:3010, 10.0.0.3:3010"),
            "10.0.0.3:3010",
        )
        with self.assertRaises(AssertionError):
            multi.last_upstream(" , ")

    def test_report_rejects_fixture_codes_and_raw_cookie(self):
        for value in (multi.ALICE_CODE, multi.BOB_CODE, multi.COOKIE_MARKER + "raw"):
            with self.assertRaises(AssertionError):
                multi.safe_report_text({"unexpected": value})
        text = multi.safe_report_text(
            {
                "memory_session": {
                    "cross_instance_status": 401,
                    "shared_across_instances": False,
                },
                "sha256": "f" * 64,
            }
        )
        self.assertEqual(json.loads(text)["memory_session"]["cross_instance_status"], 401)

    def test_compose_is_loopback_only_and_apps_publish_no_ports(self):
        text = multi.COMPOSE_FILE.read_text(encoding="utf-8")
        self.assertEqual(text.count("ports:"), 1)
        self.assertIn('"127.0.0.1:${YT066A_HOST_PORT:?set YT066A_HOST_PORT}:8080"', text)
        for service in ("app-a:", "app-b:", "app-c:"):
            self.assertIn(service, text)
        self.assertIn("front: {}", text)
        self.assertEqual(text.count("internal: true"), 1)
        self.assertIn("- front\n      - lab", text)
        self.assertIn(multi.NGINX_IMAGE, text)
        self.assertNotIn("network_mode: host", text)
        self.assertNotIn("privileged: true", text)

    def test_dockerfile_uses_exact_runtime_and_nonroot_user(self):
        text = multi.DOCKERFILE.read_text(encoding="utf-8")
        self.assertIn(f"FROM {multi.RUST_IMAGE}", text)
        self.assertIn("USER 10001:10001", text)
        self.assertNotIn("ARG RUNTIME_IMAGE", text)
        self.assertNotIn("apt-get", text)
        self.assertNotIn("ENV ", text)

    def test_write_report_is_atomic_and_bounded(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "report.json"
            multi.write_report(path, {"status": "pass-for-docker-memory"})
            self.assertEqual(json.loads(path.read_text())["status"], "pass-for-docker-memory")
            self.assertEqual(list(Path(directory).glob("yt066a-report-*")), [])

    def test_nginx_places_every_temp_path_on_tmpfs(self):
        text = (multi.QUALIFICATION / "nginx.conf").read_text(encoding="utf-8")
        for name in ("client-body", "proxy", "fastcgi", "uwsgi", "scgi"):
            self.assertIn(f"/tmp/{name}", text)
        self.assertNotIn("/var/cache/nginx", text)

    def test_cleanup_attempts_forced_removal_after_down_failure(self):
        lab = multi.DockerLab()
        lab.image_built = False
        calls: list[list[str]] = []
        list_counts = {"container": 0, "network": 0, "volume": 0, "build": 0}

        def completed(argv, code=0, stdout=""):
            return subprocess.CompletedProcess(argv, code, stdout, "")

        def fake_run(argv, **kwargs):
            calls.append(argv)
            joined = " ".join(argv)
            if " compose " in f" {joined} " and " down " in f" {joined} ":
                return completed(argv, 1)
            if argv[:3] == ["docker", "ps", "--all"] and "com.docker.compose.project" in joined:
                list_counts["container"] += 1
                return completed(argv, stdout="container-id\n" if list_counts["container"] == 1 else "")
            if argv[:3] == ["docker", "network", "ls"]:
                list_counts["network"] += 1
                return completed(argv, stdout="network-id\n" if list_counts["network"] == 1 else "")
            if argv[:3] == ["docker", "volume", "ls"]:
                list_counts["volume"] += 1
                return completed(argv, stdout="volume-id\n" if list_counts["volume"] == 1 else "")
            if argv[:3] == ["docker", "ps", "--all"] and f"name=^{lab.build_container}$" in joined:
                list_counts["build"] += 1
                return completed(argv, stdout="")
            return completed(argv)

        with patch.object(multi, "run_command", side_effect=fake_run):
            cleanup = lab.cleanup()
        self.assertEqual(cleanup["status"], "pass")
        joined_calls = [" ".join(call) for call in calls]
        self.assertTrue(any("rm --force container-id" in call for call in joined_calls))
        self.assertTrue(any("network rm network-id" in call for call in joined_calls))
        self.assertTrue(any("volume rm --force volume-id" in call for call in joined_calls))
        self.assertTrue(any(f"image rm --force {lab.app_image}" in call for call in joined_calls))

    def test_cleanup_fails_when_final_queries_fail(self):
        lab = multi.DockerLab()

        with patch.object(multi, "run_command", side_effect=RuntimeError("docker unavailable")):
            cleanup = lab.cleanup()

        self.assertEqual(cleanup["status"], "failed")
        self.assertFalse(cleanup["verification_complete"])
        self.assertTrue(all(value is None for value in cleanup["remaining"].values()))

    def test_cleanup_fails_when_owned_image_remains(self):
        lab = multi.DockerLab()

        def completed(argv, code=0, stdout=""):
            return subprocess.CompletedProcess(argv, code, stdout, "")

        def fake_run(argv, **kwargs):
            if argv[:3] == ["docker", "image", "rm"]:
                return completed(argv, 1)
            if argv[:3] == ["docker", "image", "ls"]:
                return completed(argv, stdout="sha256:retained-image\n")
            return completed(argv)

        with patch.object(multi, "run_command", side_effect=fake_run):
            cleanup = lab.cleanup()

        self.assertEqual(cleanup["status"], "failed")
        self.assertTrue(cleanup["verification_complete"])
        self.assertEqual(cleanup["remaining"]["application_image"], 1)

    def test_sigterm_after_compose_start_cleans_and_cannot_pass(self):
        multi.SCRATCH.mkdir(parents=True, exist_ok=True)
        with tempfile.TemporaryDirectory(dir=multi.SCRATCH) as directory:
            report_path = Path(directory) / "sigterm.json"
            work_path = Path(directory) / "work"
            work_path.mkdir()
            current_handler = {
                signal.SIGTERM: signal.SIG_DFL,
                signal.SIGINT: signal.getsignal(signal.SIGINT),
            }
            cleanup_calls = []
            diagnostics_calls = []

            def fake_signal(signum, handler):
                previous = current_handler[signum]
                current_handler[signum] = handler
                return previous

            class FakeLab:
                compose_started = False
                work = work_path

                def build_application(self):
                    return {"binary_sha256": "f" * 64, "image_id": "sha256:test"}

                def start(self):
                    self.compose_started = True
                    current_handler[signal.SIGTERM](signal.SIGTERM, None)

                def capture_failure_diagnostics(self):
                    diagnostics_calls.append(True)
                    return {"compose_ps_captured": True, "compose_ps_exit": 0}

                def cleanup(self):
                    cleanup_calls.append(True)
                    return {
                        "status": "pass",
                        "verification_complete": True,
                        "remaining": {
                            "containers": 0,
                            "networks": 0,
                            "volumes": 0,
                            "build_container": 0,
                            "application_image": 0,
                        },
                    }

            fake_lab = FakeLab()
            with (
                patch.object(multi, "DockerLab", return_value=fake_lab),
                patch.object(
                    multi,
                    "workspace_identity",
                    return_value={"head": "test", "working_source_sha256": "e" * 64},
                ),
                patch.object(
                    multi,
                    "run_command",
                    return_value=subprocess.CompletedProcess([], 0, "", ""),
                ),
                patch.object(multi.signal, "signal", side_effect=fake_signal),
            ):
                exit_code = multi.run(report_path)

            report = json.loads(report_path.read_text(encoding="utf-8"))
            self.assertEqual(exit_code, 1)
            self.assertEqual(report["outcome"], "cancelled")
            self.assertEqual(report["failure_type"], "SIGTERM")
            self.assertEqual(report["signal"], signal.SIGTERM)
            self.assertEqual(report["status"], "failed-for-docker-memory")
            self.assertEqual(cleanup_calls, [True])
            self.assertEqual(diagnostics_calls, [True])
            self.assertEqual(current_handler[signal.SIGTERM], signal.SIG_DFL)

    def test_first_sigterm_during_cleanup_records_cancel_and_finishes_cleanup(self):
        multi.SCRATCH.mkdir(parents=True, exist_ok=True)
        with tempfile.TemporaryDirectory(dir=multi.SCRATCH) as directory:
            report_path = Path(directory) / "cleanup-sigterm.json"
            work_path = Path(directory) / "work"
            work_path.mkdir()
            current_handler = {
                signal.SIGTERM: signal.SIG_DFL,
                signal.SIGINT: signal.getsignal(signal.SIGINT),
            }
            cleanup_calls = []
            handler_after_first_signal = []

            def fake_signal(signum, handler):
                previous = current_handler[signum]
                current_handler[signum] = handler
                return previous

            class FakeLab:
                compose_started = True
                work = work_path

                def build_application(self):
                    return {"binary_sha256": "f" * 64, "image_id": "sha256:test"}

                def start(self):
                    return {"applications": 3}

                def exercise(self):
                    return {"stateless": {"recovered_unique_upstreams": 3}}

                def cleanup(self):
                    cleanup_calls.append("started")
                    current_handler[signal.SIGTERM](signal.SIGTERM, None)
                    handler_after_first_signal.append(current_handler[signal.SIGTERM])
                    cleanup_calls.append("completed")
                    return {
                        "status": "pass",
                        "verification_complete": True,
                        "remaining": {
                            "containers": 0,
                            "networks": 0,
                            "volumes": 0,
                            "build_container": 0,
                            "application_image": 0,
                        },
                    }

            with (
                patch.object(multi, "DockerLab", return_value=FakeLab()),
                patch.object(
                    multi,
                    "workspace_identity",
                    return_value={"head": "test", "working_source_sha256": "e" * 64},
                ),
                patch.object(
                    multi,
                    "run_command",
                    return_value=subprocess.CompletedProcess([], 0, "", ""),
                ),
                patch.object(multi.signal, "signal", side_effect=fake_signal),
            ):
                exit_code = multi.run(report_path)

            report = json.loads(report_path.read_text(encoding="utf-8"))
            self.assertEqual(exit_code, 1)
            self.assertEqual(report["outcome"], "cancelled")
            self.assertEqual(report["failure_type"], "SIGTERM")
            self.assertEqual(report["signal"], signal.SIGTERM)
            self.assertEqual(report["status"], "failed-for-docker-memory")
            self.assertEqual(report["cleanup"]["status"], "pass")
            self.assertEqual(cleanup_calls, ["started", "completed"])
            self.assertEqual(handler_after_first_signal, [signal.SIG_IGN])
            self.assertEqual(current_handler[signal.SIGTERM], signal.SIG_DFL)


    def test_first_sigterm_at_report_write_rewrites_nonpassing_result(self):
        multi.SCRATCH.mkdir(parents=True, exist_ok=True)
        with tempfile.TemporaryDirectory(dir=multi.SCRATCH) as directory:
            report_path = Path(directory) / "report-sigterm.json"
            work_path = Path(directory) / "work"
            work_path.mkdir()
            current_handler = {
                signal.SIGTERM: signal.SIG_DFL,
                signal.SIGINT: signal.getsignal(signal.SIGINT),
            }
            cleanup_calls = []
            write_snapshots = []
            original_write_report = multi.write_report

            def fake_signal(signum, handler):
                previous = current_handler[signum]
                current_handler[signum] = handler
                return previous

            class FakeLab:
                compose_started = True
                work = work_path

                def build_application(self):
                    return {"binary_sha256": "f" * 64, "image_id": "sha256:test"}

                def start(self):
                    return {"applications": 3}

                def exercise(self):
                    return {"stateless": {"recovered_unique_upstreams": 3}}

                def cleanup(self):
                    cleanup_calls.append(True)
                    return {
                        "status": "pass",
                        "verification_complete": True,
                        "remaining": {
                            "containers": 0,
                            "networks": 0,
                            "volumes": 0,
                            "build_container": 0,
                            "application_image": 0,
                        },
                    }

            def signal_on_first_write(path, report):
                write_snapshots.append(json.loads(json.dumps(report)))
                if len(write_snapshots) == 1:
                    current_handler[signal.SIGTERM](signal.SIGTERM, None)
                original_write_report(path, report)

            with (
                patch.object(multi, "DockerLab", return_value=FakeLab()),
                patch.object(
                    multi,
                    "workspace_identity",
                    return_value={"head": "test", "working_source_sha256": "e" * 64},
                ),
                patch.object(
                    multi,
                    "run_command",
                    return_value=subprocess.CompletedProcess([], 0, "", ""),
                ),
                patch.object(multi.signal, "signal", side_effect=fake_signal),
                patch.object(multi, "write_report", side_effect=signal_on_first_write),
            ):
                exit_code = multi.run(report_path)

            report = json.loads(report_path.read_text(encoding="utf-8"))
            self.assertEqual(exit_code, 1)
            self.assertEqual(len(write_snapshots), 2)
            self.assertEqual(write_snapshots[0]["status"], "pass-for-docker-memory")
            self.assertEqual(report["outcome"], "cancelled")
            self.assertEqual(report["failure_type"], "SIGTERM")
            self.assertEqual(report["signal"], signal.SIGTERM)
            self.assertEqual(report["status"], "failed-for-docker-memory")
            self.assertEqual(report["stage"], "finalization")
            self.assertEqual(report["cleanup"]["status"], "pass")
            self.assertEqual(cleanup_calls, [True])
            self.assertFalse(work_path.exists())
            self.assertEqual(current_handler[signal.SIGTERM], signal.SIG_DFL)


    def test_sigterm_at_cleanup_transition_still_runs_cleanup(self):
        multi.SCRATCH.mkdir(parents=True, exist_ok=True)
        with tempfile.TemporaryDirectory(dir=multi.SCRATCH) as directory:
            report_path = Path(directory) / "transition-sigterm.json"
            work_path = Path(directory) / "work"
            work_path.mkdir()
            current_handler = {
                signal.SIGTERM: signal.SIG_DFL,
                signal.SIGINT: signal.getsignal(signal.SIGINT),
            }
            transition_calls = []
            cleanup_calls = []

            def fake_signal(signum, handler):
                previous = current_handler[signum]
                current_handler[signum] = handler
                return previous

            def signal_on_first_transition(state):
                transition_calls.append(True)
                if len(transition_calls) == 1:
                    current_handler[signal.SIGTERM](signal.SIGTERM, None)
                state["cleanup"] = True

            class FakeLab:
                compose_started = True
                work = work_path

                def build_application(self):
                    return {"binary_sha256": "f" * 64, "image_id": "sha256:test"}

                def start(self):
                    return {"applications": 3}

                def exercise(self):
                    return {"stateless": {"recovered_unique_upstreams": 3}}

                def capture_failure_diagnostics(self):
                    return {"compose_ps_captured": True, "compose_ps_exit": 0}

                def cleanup(self):
                    cleanup_calls.append(True)
                    return {
                        "status": "pass",
                        "verification_complete": True,
                        "remaining": {
                            "containers": 0,
                            "networks": 0,
                            "volumes": 0,
                            "build_container": 0,
                            "application_image": 0,
                        },
                    }

            with (
                patch.object(multi, "DockerLab", return_value=FakeLab()),
                patch.object(
                    multi,
                    "workspace_identity",
                    return_value={"head": "test", "working_source_sha256": "e" * 64},
                ),
                patch.object(
                    multi,
                    "run_command",
                    return_value=subprocess.CompletedProcess([], 0, "", ""),
                ),
                patch.object(multi.signal, "signal", side_effect=fake_signal),
                patch.object(multi, "enter_cleanup", side_effect=signal_on_first_transition),
            ):
                exit_code = multi.run(report_path)

            report = json.loads(report_path.read_text(encoding="utf-8"))
            self.assertEqual(exit_code, 1)
            self.assertEqual(report["outcome"], "cancelled")
            self.assertEqual(report["failure_type"], "SIGTERM")
            self.assertEqual(report["status"], "failed-for-docker-memory")
            self.assertEqual(report["cleanup"]["status"], "pass")
            self.assertEqual(cleanup_calls, [True])
            self.assertEqual(len(transition_calls), 2)
            self.assertFalse(work_path.exists())
            self.assertEqual(current_handler[signal.SIGTERM], signal.SIG_DFL)

    def test_report_write_failure_still_removes_owned_work_and_cannot_pass(self):
        multi.SCRATCH.mkdir(parents=True, exist_ok=True)
        with tempfile.TemporaryDirectory(dir=multi.SCRATCH) as directory:
            report_path = Path(directory) / "write-failure.json"
            report_path.write_text('{"status":"stale-pass"}\n', encoding="utf-8")
            work_path = Path(directory) / "work"
            work_path.mkdir()
            current_handler = {
                signal.SIGTERM: signal.SIG_DFL,
                signal.SIGINT: signal.getsignal(signal.SIGINT),
            }
            cleanup_calls = []

            def fake_signal(signum, handler):
                previous = current_handler[signum]
                current_handler[signum] = handler
                return previous

            class FakeLab:
                compose_started = True
                work = work_path

                def build_application(self):
                    return {"binary_sha256": "f" * 64, "image_id": "sha256:test"}

                def start(self):
                    return {"applications": 3}

                def exercise(self):
                    return {"stateless": {"recovered_unique_upstreams": 3}}

                def cleanup(self):
                    cleanup_calls.append(True)
                    return {
                        "status": "pass",
                        "verification_complete": True,
                        "remaining": {
                            "containers": 0,
                            "networks": 0,
                            "volumes": 0,
                            "build_container": 0,
                            "application_image": 0,
                        },
                    }

            with (
                patch.object(multi, "DockerLab", return_value=FakeLab()),
                patch.object(
                    multi,
                    "workspace_identity",
                    return_value={"head": "test", "working_source_sha256": "e" * 64},
                ),
                patch.object(
                    multi,
                    "run_command",
                    return_value=subprocess.CompletedProcess([], 0, "", ""),
                ),
                patch.object(multi.signal, "signal", side_effect=fake_signal),
                patch.object(multi, "write_report", side_effect=OSError("write failed")),
            ):
                exit_code = multi.run(report_path)

            self.assertEqual(exit_code, 1)
            self.assertEqual(cleanup_calls, [True])
            self.assertFalse(work_path.exists())
            self.assertFalse(report_path.exists())
            self.assertEqual(current_handler[signal.SIGTERM], signal.SIG_DFL)

    def test_owned_work_removal_failure_prevents_pass(self):
        multi.SCRATCH.mkdir(parents=True, exist_ok=True)
        with tempfile.TemporaryDirectory(dir=multi.SCRATCH) as directory:
            report_path = Path(directory) / "work-cleanup-failure.json"
            work_path = Path(directory) / "work"
            work_path.mkdir()
            current_handler = {
                signal.SIGTERM: signal.SIG_DFL,
                signal.SIGINT: signal.getsignal(signal.SIGINT),
            }
            cleanup_calls = []

            def fake_signal(signum, handler):
                previous = current_handler[signum]
                current_handler[signum] = handler
                return previous

            class FakeLab:
                compose_started = True
                work = work_path

                def build_application(self):
                    return {"binary_sha256": "f" * 64, "image_id": "sha256:test"}

                def start(self):
                    return {"applications": 3}

                def exercise(self):
                    return {"stateless": {"recovered_unique_upstreams": 3}}

                def cleanup(self):
                    cleanup_calls.append(True)
                    return {
                        "status": "pass",
                        "verification_complete": True,
                        "remaining": {
                            "containers": 0,
                            "networks": 0,
                            "volumes": 0,
                            "build_container": 0,
                            "application_image": 0,
                        },
                    }

            with (
                patch.object(multi, "DockerLab", return_value=FakeLab()),
                patch.object(
                    multi,
                    "workspace_identity",
                    return_value={"head": "test", "working_source_sha256": "e" * 64},
                ),
                patch.object(
                    multi,
                    "run_command",
                    return_value=subprocess.CompletedProcess([], 0, "", ""),
                ),
                patch.object(multi.signal, "signal", side_effect=fake_signal),
                patch.object(multi.shutil, "rmtree", side_effect=OSError("remove failed")),
            ):
                exit_code = multi.run(report_path)

            report = json.loads(report_path.read_text(encoding="utf-8"))
            self.assertEqual(exit_code, 1)
            self.assertEqual(cleanup_calls, [True])
            self.assertEqual(report["status"], "failed-for-docker-memory")
            self.assertEqual(report["work_cleanup"]["status"], "failed")
            self.assertEqual(report["work_cleanup"]["error_type"], "OSError")
            self.assertTrue(report["work_cleanup"]["remaining"])
            self.assertTrue(work_path.exists())
            self.assertEqual(current_handler[signal.SIGTERM], signal.SIG_DFL)


    def test_first_sigint_during_cleanup_records_cancel_and_finishes_cleanup(self):
        multi.SCRATCH.mkdir(parents=True, exist_ok=True)
        with tempfile.TemporaryDirectory(dir=multi.SCRATCH) as directory:
            report_path = Path(directory) / "cleanup-sigint.json"
            work_path = Path(directory) / "work"
            work_path.mkdir()
            previous_sigint = signal.getsignal(signal.SIGINT)
            current_handler = {
                signal.SIGTERM: signal.SIG_DFL,
                signal.SIGINT: previous_sigint,
            }
            cleanup_calls = []
            handler_after_first_signal = []

            def fake_signal(signum, handler):
                previous = current_handler[signum]
                current_handler[signum] = handler
                return previous

            class FakeLab:
                compose_started = True
                work = work_path

                def build_application(self):
                    return {"binary_sha256": "f" * 64, "image_id": "sha256:test"}

                def start(self):
                    return {"applications": 3}

                def exercise(self):
                    return {"stateless": {"recovered_unique_upstreams": 3}}

                def cleanup(self):
                    cleanup_calls.append("started")
                    current_handler[signal.SIGINT](signal.SIGINT, None)
                    handler_after_first_signal.append(
                        (
                            current_handler[signal.SIGTERM],
                            current_handler[signal.SIGINT],
                        )
                    )
                    cleanup_calls.append("completed")
                    return {
                        "status": "pass",
                        "verification_complete": True,
                        "remaining": {
                            "containers": 0,
                            "networks": 0,
                            "volumes": 0,
                            "build_container": 0,
                            "application_image": 0,
                        },
                    }

            with (
                patch.object(multi, "DockerLab", return_value=FakeLab()),
                patch.object(
                    multi,
                    "workspace_identity",
                    return_value={"head": "test", "working_source_sha256": "e" * 64},
                ),
                patch.object(
                    multi,
                    "run_command",
                    return_value=subprocess.CompletedProcess([], 0, "", ""),
                ),
                patch.object(multi.signal, "signal", side_effect=fake_signal),
            ):
                exit_code = multi.run(report_path)

            report = json.loads(report_path.read_text(encoding="utf-8"))
            self.assertEqual(exit_code, 1)
            self.assertEqual(report["outcome"], "cancelled")
            self.assertEqual(report["failure_type"], "KeyboardInterrupt")
            self.assertEqual(report["signal"], signal.SIGINT)
            self.assertEqual(report["status"], "failed-for-docker-memory")
            self.assertEqual(report["cleanup"]["status"], "pass")
            self.assertEqual(cleanup_calls, ["started", "completed"])
            self.assertEqual(
                handler_after_first_signal,
                [(signal.SIG_IGN, signal.SIG_IGN)],
            )
            self.assertEqual(current_handler[signal.SIGTERM], signal.SIG_DFL)
            self.assertEqual(current_handler[signal.SIGINT], previous_sigint)

    def test_sigterm_then_sigint_is_ignored_during_cleanup(self):
        multi.SCRATCH.mkdir(parents=True, exist_ok=True)
        with tempfile.TemporaryDirectory(dir=multi.SCRATCH) as directory:
            report_path = Path(directory) / "sigterm-then-sigint.json"
            work_path = Path(directory) / "work"
            work_path.mkdir()
            previous_sigint = signal.getsignal(signal.SIGINT)
            current_handler = {
                signal.SIGTERM: signal.SIG_DFL,
                signal.SIGINT: previous_sigint,
            }
            cleanup_calls = []

            def fake_signal(signum, handler):
                previous = current_handler[signum]
                current_handler[signum] = handler
                return previous

            class FakeLab:
                compose_started = True
                work = work_path

                def build_application(self):
                    return {"binary_sha256": "f" * 64, "image_id": "sha256:test"}

                def start(self):
                    current_handler[signal.SIGTERM](signal.SIGTERM, None)

                def capture_failure_diagnostics(self):
                    return {"compose_ps_captured": True, "compose_ps_exit": 0}

                def cleanup(self):
                    cleanup_calls.append(True)
                    if current_handler[signal.SIGINT] != signal.SIG_IGN:
                        raise AssertionError("SIGINT was not ignored after SIGTERM")
                    return {
                        "status": "pass",
                        "verification_complete": True,
                        "remaining": {
                            "containers": 0,
                            "networks": 0,
                            "volumes": 0,
                            "build_container": 0,
                            "application_image": 0,
                        },
                    }

            with (
                patch.object(multi, "DockerLab", return_value=FakeLab()),
                patch.object(
                    multi,
                    "workspace_identity",
                    return_value={"head": "test", "working_source_sha256": "e" * 64},
                ),
                patch.object(
                    multi,
                    "run_command",
                    return_value=subprocess.CompletedProcess([], 0, "", ""),
                ),
                patch.object(multi.signal, "signal", side_effect=fake_signal),
            ):
                exit_code = multi.run(report_path)

            report = json.loads(report_path.read_text(encoding="utf-8"))
            self.assertEqual(exit_code, 1)
            self.assertEqual(report["failure_type"], "SIGTERM")
            self.assertEqual(report["status"], "failed-for-docker-memory")
            self.assertEqual(cleanup_calls, [True])
            self.assertEqual(current_handler[signal.SIGTERM], signal.SIG_DFL)
            self.assertEqual(current_handler[signal.SIGINT], previous_sigint)

    def test_sigint_then_sigterm_is_ignored_during_cleanup(self):
        multi.SCRATCH.mkdir(parents=True, exist_ok=True)
        with tempfile.TemporaryDirectory(dir=multi.SCRATCH) as directory:
            report_path = Path(directory) / "sigint-then-sigterm.json"
            work_path = Path(directory) / "work"
            work_path.mkdir()
            previous_sigint = signal.getsignal(signal.SIGINT)
            current_handler = {
                signal.SIGTERM: signal.SIG_DFL,
                signal.SIGINT: previous_sigint,
            }
            cleanup_calls = []

            def fake_signal(signum, handler):
                previous = current_handler[signum]
                current_handler[signum] = handler
                return previous

            class FakeLab:
                compose_started = True
                work = work_path

                def build_application(self):
                    return {"binary_sha256": "f" * 64, "image_id": "sha256:test"}

                def start(self):
                    current_handler[signal.SIGINT](signal.SIGINT, None)

                def capture_failure_diagnostics(self):
                    return {"compose_ps_captured": True, "compose_ps_exit": 0}

                def cleanup(self):
                    cleanup_calls.append(True)
                    if current_handler[signal.SIGTERM] != signal.SIG_IGN:
                        raise AssertionError("SIGTERM was not ignored after SIGINT")
                    return {
                        "status": "pass",
                        "verification_complete": True,
                        "remaining": {
                            "containers": 0,
                            "networks": 0,
                            "volumes": 0,
                            "build_container": 0,
                            "application_image": 0,
                        },
                    }

            with (
                patch.object(multi, "DockerLab", return_value=FakeLab()),
                patch.object(
                    multi,
                    "workspace_identity",
                    return_value={"head": "test", "working_source_sha256": "e" * 64},
                ),
                patch.object(
                    multi,
                    "run_command",
                    return_value=subprocess.CompletedProcess([], 0, "", ""),
                ),
                patch.object(multi.signal, "signal", side_effect=fake_signal),
            ):
                exit_code = multi.run(report_path)

            report = json.loads(report_path.read_text(encoding="utf-8"))
            self.assertEqual(exit_code, 1)
            self.assertEqual(report["failure_type"], "KeyboardInterrupt")
            self.assertEqual(report["status"], "failed-for-docker-memory")
            self.assertEqual(cleanup_calls, [True])
            self.assertEqual(current_handler[signal.SIGTERM], signal.SIG_DFL)
            self.assertEqual(current_handler[signal.SIGINT], previous_sigint)


if __name__ == "__main__":
    unittest.main()
