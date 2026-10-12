#!/usr/bin/env python3
"""Bounded YT-066a Docker proof for stateless routing and Memory Session limits."""
from __future__ import annotations

import argparse
from contextlib import closing
from html import unescape
import hashlib
import json
import os
from pathlib import Path
import re
import secrets
import signal
import shutil
import socket
import subprocess
import tempfile
import time
from urllib.error import HTTPError
from urllib.parse import urlencode
from urllib.request import HTTPRedirectHandler, ProxyHandler, Request, build_opener

ROOT = Path(__file__).resolve().parent.parent
QUALIFICATION = ROOT / "qualification" / "yt066a"
COMPOSE_FILE = QUALIFICATION / "compose.yaml"
DOCKERFILE = QUALIFICATION / "Dockerfile"
SCRATCH = ROOT / "target" / "yt066a"
DEFAULT_REPORT = SCRATCH / "multi-instance.json"
VENDOR = ROOT / "target" / "m10-delivery" / "docker" / "vendor"
RUST_IMAGE = "rust@sha256:b36c246742b4d323472588f789601be34eb4af053255e2af53969078ee8dcec7"
NGINX_IMAGE = "nginx@sha256:0985e772fb9f729e6fa0980da05fca5d9c468e870eed43071545afa9d2e27d94"
ALICE_CODE = "a" * 64
BOB_CODE = "b" * 64
COOKIE_MARKER = "yetlet-auth-notes="


class TerminationRequested(Exception):
    def __init__(self, signum: int) -> None:
        super().__init__(f"termination requested by signal {signum}")
        self.signum = signum


def enter_cleanup(signal_state: dict[str, int | bool | str | None]) -> None:
    """Enter non-raising cleanup mode; kept separate for transition regression tests."""
    signal_state["cleanup"] = True


class NoRedirect(HTTPRedirectHandler):
    def redirect_request(self, request, fp, code, msg, headers, newurl):
        return None


def require(condition: bool, message: str) -> None:
    if not condition:
        raise AssertionError(message)


def run_command(
    argv: list[str],
    *,
    env: dict[str, str] | None = None,
    timeout: float = 120,
    check: bool = True,
) -> subprocess.CompletedProcess[str]:
    result = subprocess.run(
        argv,
        cwd=ROOT,
        env=env,
        text=True,
        stdin=subprocess.DEVNULL,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        timeout=timeout,
    )
    if check and result.returncode != 0:
        raise RuntimeError(f"command failed with exit {result.returncode}: {argv[0]}")
    return result


def run_logged(
    argv: list[str],
    log_path: Path,
    *,
    env: dict[str, str] | None = None,
    timeout: float = 600,
) -> None:
    log_path.parent.mkdir(parents=True, exist_ok=True)
    with log_path.open("w", encoding="utf-8") as log:
        result = subprocess.run(
            argv,
            cwd=ROOT,
            env=env,
            text=True,
            stdin=subprocess.DEVNULL,
            stdout=log,
            stderr=subprocess.STDOUT,
            timeout=timeout,
        )
    if result.returncode != 0:
        raise RuntimeError(f"logged command failed with exit {result.returncode}: {argv[0]}")


def free_port() -> int:
    with closing(socket.socket()) as listener:
        listener.bind(("127.0.0.1", 0))
        return int(listener.getsockname()[1])


def call(
    base: str,
    method: str,
    path: str,
    cookie_value: str | None = None,
    fields: dict[str, str] | None = None,
) -> tuple[int, dict[str, str], str]:
    headers: dict[str, str] = {}
    payload = None
    if cookie_value:
        headers["Cookie"] = cookie_value
    if fields is not None:
        payload = urlencode(fields).encode()
        headers["Content-Type"] = "application/x-www-form-urlencoded"
    request = Request(base + path, method=method, data=payload, headers=headers)
    opener = build_opener(ProxyHandler({}), NoRedirect())
    try:
        with opener.open(request, timeout=3) as response:
            return (
                response.status,
                {name.lower(): value for name, value in response.headers.items()},
                response.read().decode(),
            )
    except HTTPError as error:
        return (
            error.code,
            {name.lower(): value for name, value in error.headers.items()},
            error.read().decode(),
        )


def cookie(headers: dict[str, str]) -> str:
    value = headers.get("set-cookie")
    if not value:
        raise AssertionError("Set-Cookie header missing")
    return value.split(";", 1)[0]


def csrf(html: str) -> str:
    match = re.search(r'name="csrf" value="([^"]+)"', html)
    if not match:
        raise AssertionError("CSRF field missing")
    return unescape(match.group(1))


def last_upstream(value: str) -> str:
    attempts = [item.strip() for item in value.split(",") if item.strip()]
    if not attempts:
        raise AssertionError("upstream header is empty")
    return attempts[-1]


def workspace_identity() -> dict[str, object]:
    head = run_command(["git", "rev-parse", "HEAD"]).stdout.strip()
    listed = run_command(["git", "ls-files", "-co", "--exclude-standard"]).stdout.splitlines()
    digest = hashlib.sha256()
    count = 0
    for relative in sorted(set(listed)):
        path = ROOT / relative
        if not path.exists() and not path.is_symlink():
            continue
        digest.update(relative.encode())
        digest.update(b"\0")
        if path.is_symlink():
            digest.update(b"symlink\0")
            digest.update(os.readlink(path).encode())
        elif path.is_file():
            digest.update(b"file\0")
            digest.update(path.read_bytes())
        digest.update(b"\0")
        count += 1
    status = run_command(["git", "status", "--porcelain=v1", "--untracked-files=all"]).stdout
    return {
        "head": head,
        "working_source_sha256": digest.hexdigest(),
        "source_paths": count,
        "dirty": bool(status.strip()),
    }


def safe_report_text(report: dict[str, object]) -> str:
    text = json.dumps(report, indent=2, sort_keys=True) + "\n"
    for forbidden in (ALICE_CODE, BOB_CODE, COOKIE_MARKER):
        if forbidden in text:
            raise AssertionError("sensitive fixture value entered report")
    return text


def write_report(path: Path, report: dict[str, object]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    text = safe_report_text(report)
    with tempfile.NamedTemporaryFile(
        mode="w", encoding="utf-8", dir=path.parent, prefix="yt066a-report-", delete=False
    ) as output:
        temporary = Path(output.name)
        try:
            output.write(text)
            output.flush()
            os.replace(temporary, path)
        finally:
            temporary.unlink(missing_ok=True)


class DockerLab:
    def __init__(self) -> None:
        token = secrets.token_hex(3)
        self.project = f"yetlet-yt066a-{os.getpid()}-{token}"
        self.build_container = f"{self.project}-build"
        self.app_image = f"yetlet-yt066a-app:{os.getpid()}-{token}"
        self.host_port = free_port()
        self.base = f"http://127.0.0.1:{self.host_port}"
        self.work = SCRATCH / f"work-{os.getpid()}-{token}"
        self.build = self.work / "build"
        self.image_context = self.work / "image"
        self.logs = SCRATCH / "logs"
        self.image_built = False
        self.compose_started = False
        self.env = dict(
            os.environ,
            YT066A_APP_IMAGE=self.app_image,
            YT066A_HOST_PORT=str(self.host_port),
            YT066A_ALICE_CODE=ALICE_CODE,
            YT066A_BOB_CODE=BOB_CODE,
        )

    def compose_argv(self, *arguments: str) -> list[str]:
        return [
            "docker",
            "compose",
            "--ansi",
            "never",
            "--project-name",
            self.project,
            "--file",
            str(COMPOSE_FILE),
            *arguments,
        ]

    def compose(
        self,
        *arguments: str,
        timeout: float = 120,
        check: bool = True,
    ) -> subprocess.CompletedProcess[str]:
        return run_command(
            self.compose_argv(*arguments), env=self.env, timeout=timeout, check=check
        )

    def prepare(self) -> None:
        require(VENDOR.is_dir(), "retained locked vendor directory is missing")
        require(COMPOSE_FILE.is_file(), "compose file is missing")
        require(DOCKERFILE.is_file(), "Dockerfile is missing")
        shutil.rmtree(self.work, ignore_errors=True)
        self.build.mkdir(parents=True)
        self.image_context.mkdir(parents=True)
        self.logs.mkdir(parents=True, exist_ok=True)

    def build_application(self) -> dict[str, str]:
        self.prepare()
        uid, gid = os.getuid(), os.getgid()
        build_script = """set -euo pipefail
export CARGO_HOME=/tmp/cargo
mkdir -p "$CARGO_HOME"
printf '[source.crates-io]\nreplace-with = "vendored-sources"\n[source.vendored-sources]\ndirectory = "/vendor"\n' > "$CARGO_HOME/config.toml"
rustc -vV
cargo --version
cargo build --release --locked --offline -p yetlet-auth-notes
"""
        run_logged(
            [
                "docker",
                "run",
                "--rm",
                "--name",
                self.build_container,
                "--label",
                "yetlet.qualification=yt066a-build",
                "--network",
                "none",
                "--read-only",
                "--user",
                f"{uid}:{gid}",
                "--cpus",
                "4",
                "--memory",
                "6g",
                "--pids-limit",
                "384",
                "--cap-drop",
                "ALL",
                "--security-opt",
                "no-new-privileges",
                "--tmpfs",
                f"/tmp:rw,exec,nosuid,nodev,size=1073741824,mode=1777,uid={uid},gid={gid}",
                "--mount",
                f"type=bind,src={ROOT},dst=/src,readonly",
                "--mount",
                f"type=bind,src={self.build},dst=/src/target",
                "--mount",
                f"type=bind,src={VENDOR},dst=/vendor,readonly",
                "--workdir",
                "/src",
                RUST_IMAGE,
                "bash",
                "-ceu",
                build_script,
            ],
            self.logs / "app-build.log",
            timeout=900,
        )
        binary = self.build / "release" / "yetlet-auth-notes"
        require(binary.is_file(), "container build did not produce auth-notes binary")
        destination = self.image_context / "yetlet-auth-notes"
        shutil.copy2(binary, destination)
        destination.chmod(0o555)
        binary_sha256 = hashlib.sha256(destination.read_bytes()).hexdigest()
        run_logged(
            [
                "docker",
                "build",
                "--network=none",
                "--pull=false",
                "--label",
                "yetlet.qualification=yt066a-app",
                "--tag",
                self.app_image,
                "--file",
                str(DOCKERFILE),
                str(self.image_context),
            ],
            self.logs / "image-build.log",
            timeout=300,
        )
        self.image_built = True
        image_id = run_command(
            ["docker", "image", "inspect", self.app_image, "--format", "{{.Id}}"]
        ).stdout.strip()
        return {"binary_sha256": binary_sha256, "image_id": image_id}

    def start(self) -> dict[str, object]:
        run_logged(
            ["docker", "pull", NGINX_IMAGE],
            self.logs / "proxy-pull.log",
            timeout=300,
        )
        self.compose("config", "--quiet")
        result = self.compose("up", "--detach", timeout=180)
        (self.logs / "compose-up.log").write_text(
            result.stdout + result.stderr, encoding="utf-8"
        )
        self.compose_started = True
        for instance in ("a", "b", "c"):
            self.wait_ready(f"/__yt066a/{instance}/healthz")
        self.wait_ready("/healthz")
        return self.verify_topology()

    def wait_ready(self, path: str, timeout: float = 20) -> None:
        deadline = time.monotonic() + timeout
        while time.monotonic() < deadline:
            try:
                status, _, body = call(self.base, "GET", path)
                if status == 200 and body == "ok":
                    return
            except OSError:
                pass
            time.sleep(0.1)
        raise TimeoutError(f"endpoint did not become ready: {path}")

    def verify_topology(self) -> dict[str, object]:
        containers: dict[str, object] = {}
        for service in ("app-a", "app-b", "app-c", "proxy"):
            container_id = self.compose("ps", "--quiet", service).stdout.strip()
            require(bool(container_id), f"container ID missing for {service}")
            details = json.loads(
                run_command(["docker", "inspect", container_id]).stdout
            )[0]
            containers[service] = details
        for service in ("app-a", "app-b", "app-c"):
            ports = containers[service]["NetworkSettings"]["Ports"] or {}
            require(
                not any(bindings for bindings in ports.values()),
                f"{service} unexpectedly publishes a host port",
            )

        proxy_bindings = containers["proxy"]["NetworkSettings"]["Ports"]["8080/tcp"]
        require(len(proxy_bindings) == 1, "proxy must have exactly one host binding")
        binding = proxy_bindings[0]
        require(binding["HostIp"] == "127.0.0.1", "proxy is not bound to host loopback")
        require(binding["HostPort"] == str(self.host_port), "proxy host port mismatch")

        app_network_sets = [
            set(containers[service]["NetworkSettings"]["Networks"])
            for service in ("app-a", "app-b", "app-c")
        ]
        require(all(len(networks) == 1 for networks in app_network_sets), "an app has extra networks")
        require(
            app_network_sets[0] == app_network_sets[1] == app_network_sets[2],
            "apps do not share exactly one backend network",
        )
        backend_name = next(iter(app_network_sets[0]))
        proxy_networks = containers["proxy"]["NetworkSettings"]["Networks"]
        require(set(proxy_networks) == {backend_name, f"{self.project}_front"}, "proxy network shape differs")
        backend_id = proxy_networks[backend_name]["NetworkID"]
        front_name = f"{self.project}_front"
        front_id = proxy_networks[front_name]["NetworkID"]
        backend = json.loads(run_command(["docker", "network", "inspect", backend_id]).stdout)[0]
        front = json.loads(run_command(["docker", "network", "inspect", front_id]).stdout)[0]
        require(backend.get("Internal") is True, "backend network is not internal")
        require(front.get("Internal") is False, "front network must permit loopback publication")

        nginx_version = self.compose("exec", "--no-TTY", "proxy", "nginx", "-v").stderr.strip()
        return {
            "applications": 3,
            "application_host_ports": 0,
            "proxy_host_ip": "127.0.0.1",
            "backend_network_internal": True,
            "front_network_internal": False,
            "nginx_version": nginx_version,
            "nginx_image": NGINX_IMAGE,
        }
    def sample_pool(self, count: int) -> set[str]:
        observed: set[str] = set()
        for _ in range(count):
            status, headers, body = call(self.base, "GET", "/healthz")
            require(status == 200 and body == "ok", "pooled health request failed")
            require("set-cookie" not in headers, "stateless health route created a Session")
            observed.add(last_upstream(headers.get("x-yetlet-upstream", "")))
        return observed

    def login_on_a(self) -> str:
        prefix = "/__yt066a/a"
        status, headers, html = call(self.base, "GET", prefix + "/auth/login")
        require(status == 200, "app-a login page failed")
        require(headers.get("cache-control") == "no-store", "login page is cacheable")
        anonymous_cookie = cookie(headers)
        anonymous_csrf = csrf(html)
        status, headers, _ = call(
            self.base,
            "POST",
            prefix + "/auth/login",
            anonymous_cookie,
            {"csrf": anonymous_csrf, "username": "alice", "code": ALICE_CODE},
        )
        require(status == 303, "app-a login failed")
        authenticated_cookie = cookie(headers)
        require(authenticated_cookie != anonymous_cookie, "login did not rotate Session ID")
        status, headers, _ = call(
            self.base, "GET", prefix + "/auth/notes", authenticated_cookie
        )
        require(status == 200, "app-a did not accept its authenticated Session")
        require(headers.get("cache-control") == "no-store", "private notes are cacheable")
        return authenticated_cookie

    def require_rejected(self, instance: str, cookie_value: str) -> int:
        status, headers, _ = call(
            self.base,
            "GET",
            f"/__yt066a/{instance}/auth/notes",
            cookie_value,
        )
        require(status == 401, f"app-{instance} accepted another process Session")
        require(headers.get("cache-control") == "no-store", "401 Session response is cacheable")
        return status

    def exercise(self) -> dict[str, object]:
        initial = self.sample_pool(18)
        require(len(initial) == 3, "proxy did not reach all three instances")
        authenticated_cookie = self.login_on_a()
        cross_instance = {
            "app_b": self.require_rejected("b", authenticated_cookie),
            "app_c": self.require_rejected("c", authenticated_cookie),
        }

        stop_started = time.monotonic()
        self.compose("stop", "--timeout", "5", "app-a", timeout=30)
        degraded = self.sample_pool(18)
        require(len(degraded) == 2, "degraded pool did not use exactly two survivors")
        degraded_seconds = round(time.monotonic() - stop_started, 3)

        restart_started = time.monotonic()
        self.compose("start", "app-a", timeout=30)
        self.wait_ready("/__yt066a/a/healthz")
        time.sleep(1.1)
        deadline = time.monotonic() + 10
        recovered: set[str] = set()
        while time.monotonic() < deadline:
            recovered |= self.sample_pool(12)
            if len(recovered) == 3:
                break
            time.sleep(0.2)
        require(len(recovered) == 3, "restarted instance did not rejoin the pool")
        recovery_seconds = round(time.monotonic() - restart_started, 3)
        restarted_status = self.require_rejected("a", authenticated_cookie)

        return {
            "stateless": {
                "initial_unique_upstreams": len(initial),
                "degraded_unique_upstreams": len(degraded),
                "recovered_unique_upstreams": len(recovered),
                "requests_per_sample": 18,
                "degraded_service_seconds": degraded_seconds,
                "recovery_seconds": recovery_seconds,
            },
            "memory_session": {
                "issuing_instance_status": 200,
                "cross_instance_statuses": cross_instance,
                "old_cookie_after_issuing_instance_restart": restarted_status,
                "shared_across_instances": False,
                "persistent_across_restart": False,
                "private_responses_no_store": True,
            },
        }

    def capture_failure_diagnostics(self) -> dict[str, object]:
        diagnostics: dict[str, object] = {}
        for name, arguments, path in (
            (
                "compose_ps",
                ("ps", "--all", "--format", "json"),
                self.logs / "failure-compose-ps.json",
            ),
            (
                "compose_logs",
                ("logs", "--no-color", "--no-log-prefix"),
                self.logs / "failure-compose.log",
            ),
        ):
            result = self.compose(*arguments, timeout=30, check=False)
            path.write_text(result.stdout + result.stderr, encoding="utf-8")
            diagnostics[name + "_exit"] = result.returncode
            diagnostics[name + "_captured"] = True
        return diagnostics

    def cleanup(self) -> dict[str, object]:
        actions: list[dict[str, object]] = []

        def attempt(
            argv: list[str], name: str, timeout: float = 60
        ) -> subprocess.CompletedProcess[str] | None:
            try:
                result = run_command(argv, env=self.env, timeout=timeout, check=False)
            except Exception as error:
                actions.append(
                    {
                        "action": name,
                        "exit": "exception",
                        "error_type": type(error).__name__,
                    }
                )
                return None
            actions.append({"action": name, "exit": result.returncode})
            return result

        def listed_ids(argv: list[str], name: str) -> list[str] | None:
            result = attempt(argv, name)
            if result is None or result.returncode != 0:
                return None
            return result.stdout.split()

        attempt(
            self.compose_argv(
                "down", "--volumes", "--remove-orphans", "--timeout", "5"
            ),
            "compose_down",
            120,
        )
        attempt(
            ["docker", "rm", "--force", self.build_container],
            "build_container_remove",
        )

        container_filter = f"label=com.docker.compose.project={self.project}"
        containers = listed_ids(
            ["docker", "ps", "--all", "--quiet", "--filter", container_filter],
            "container_list",
        )
        if containers:
            attempt(
                ["docker", "rm", "--force", *containers],
                "forced_container_remove",
            )
        networks = listed_ids(
            ["docker", "network", "ls", "--quiet", "--filter", container_filter],
            "network_list",
        )
        if networks:
            attempt(
                ["docker", "network", "rm", *networks],
                "forced_network_remove",
            )
        volumes = listed_ids(
            ["docker", "volume", "ls", "--quiet", "--filter", container_filter],
            "volume_list",
        )
        if volumes:
            attempt(
                ["docker", "volume", "rm", "--force", *volumes],
                "forced_volume_remove",
            )

        # The tag is unique to this run. Always attempt removal: termination can
        # occur after Docker creates the image but before image_built is recorded.
        attempt(
            ["docker", "image", "rm", "--force", self.app_image],
            "app_image_remove",
        )

        remaining: dict[str, list[str] | None] = {
            "containers": listed_ids(
                [
                    "docker",
                    "ps",
                    "--all",
                    "--quiet",
                    "--filter",
                    container_filter,
                ],
                "container_recheck",
            ),
            "networks": listed_ids(
                [
                    "docker",
                    "network",
                    "ls",
                    "--quiet",
                    "--filter",
                    container_filter,
                ],
                "network_recheck",
            ),
            "volumes": listed_ids(
                [
                    "docker",
                    "volume",
                    "ls",
                    "--quiet",
                    "--filter",
                    container_filter,
                ],
                "volume_recheck",
            ),
            "build_container": listed_ids(
                [
                    "docker",
                    "ps",
                    "--all",
                    "--quiet",
                    "--filter",
                    f"name=^{self.build_container}$",
                ],
                "build_container_recheck",
            ),
            "application_image": listed_ids(
                [
                    "docker",
                    "image",
                    "ls",
                    "--quiet",
                    "--no-trunc",
                    self.app_image,
                ],
                "app_image_recheck",
            ),
        }
        verification_complete = all(value is not None for value in remaining.values())
        clean = verification_complete and all(not value for value in remaining.values())
        return {
            "status": "pass" if clean else "failed",
            "owned_containers": 4,
            "owned_networks": 2,
            "owned_volumes": 0,
            "verification_complete": verification_complete,
            "remaining": {
                name: None if values is None else len(values)
                for name, values in remaining.items()
            },
            "actions": actions,
        }

def run(report_path: Path) -> int:
    started = time.monotonic()
    lab = DockerLab()
    report: dict[str, object] = {
        "task_id": "YT-066a",
        "status": "failed-for-docker-memory",
        "stage": "preflight",
        "source": workspace_identity(),
        "images": {"rust": RUST_IMAGE, "nginx": NGINX_IMAGE},
        "qualification": {
            "distributed_session": "not_verified",
            "redis": "not_started",
            "cross_host_ha": "not_verified",
            "load_or_slo": "not_verified",
            "production_recovery": "not_verified",
        },
    }
    signal_state: dict[str, int | bool | str | None] = {
        "cleanup": False,
        "received": False,
        "signum": None,
        "failure_type": None,
    }

    def mark_cancelled(
        signum: int, failure_type: str, *, finalization: bool = False
    ) -> None:
        report["outcome"] = "cancelled"
        report["failure_type"] = failure_type
        report["signal"] = signum
        if finalization:
            report["stage"] = "finalization"

    def ignore_cancellation_signals() -> None:
        signal.signal(signal.SIGTERM, signal.SIG_IGN)
        signal.signal(signal.SIGINT, signal.SIG_IGN)

    def handle_cancellation(signum: int, _frame) -> None:
        failure_type = "KeyboardInterrupt" if signum == signal.SIGINT else "SIGTERM"
        if not bool(signal_state["received"]):
            signal_state["received"] = True
            signal_state["signum"] = signum
            signal_state["failure_type"] = failure_type
        if bool(signal_state["cleanup"]):
            # Cleanup/finalization must continue after the first cancellation.
            # Ignore both signals so a different second signal cannot preempt
            # teardown.
            ignore_cancellation_signals()
            return
        # Mark cleanup mode before raising, so cancellation at any transition
        # into teardown cannot raise again or bypass the outer recovery path.
        signal_state["cleanup"] = True
        ignore_cancellation_signals()
        if signum == signal.SIGINT:
            raise KeyboardInterrupt
        raise TerminationRequested(signum)

    def capture_failure_diagnostics() -> None:
        if not lab.compose_started or "failure_diagnostics" in report:
            return
        try:
            report["failure_diagnostics"] = lab.capture_failure_diagnostics()
        except Exception as error:
            report["failure_diagnostics"] = {
                "status": "failed",
                "error_type": type(error).__name__,
            }

    def finalize() -> int:
        enter_cleanup(signal_state)
        try:
            cleanup = lab.cleanup()
        except Exception as error:
            cleanup = {
                "status": "failed",
                "verification_complete": False,
                "error_type": type(error).__name__,
            }
        report["cleanup"] = cleanup

        work_cleanup: dict[str, object] = {
            "status": "pass",
            "remaining": False,
        }
        try:
            if lab.work.exists():
                shutil.rmtree(lab.work)
            remaining = lab.work.exists()
            work_cleanup["remaining"] = remaining
            if remaining:
                work_cleanup["status"] = "failed"
        except Exception as error:
            work_cleanup = {
                "status": "failed",
                "remaining": lab.work.exists(),
                "error_type": type(error).__name__,
            }
        report["work_cleanup"] = work_cleanup

        if bool(signal_state["received"]):
            mark_cancelled(
                int(signal_state["signum"] or signal.SIGTERM),
                str(signal_state["failure_type"] or "SIGTERM"),
                finalization=True,
            )
        report["duration_seconds"] = round(time.monotonic() - started, 3)

        cleanup_passed = (
            isinstance(cleanup, dict)
            and cleanup.get("status") == "pass"
            and cleanup.get("verification_complete") is True
            and work_cleanup.get("status") == "pass"
            and work_cleanup.get("remaining") is False
        )
        exit_code = 1
        if report.get("outcome") == "pass-for-docker-memory" and cleanup_passed:
            report["status"] = "pass-for-docker-memory"
            report["stage"] = "complete"
            exit_code = 0

        report_write_error: str | None = None
        try:
            write_report(report_path, report)
        except Exception as error:
            report_write_error = type(error).__name__

        # Resource cleanup, work cleanup and the candidate report write are done.
        # Close the cancellation window, then reconcile any first signal that
        # arrived during those operations.
        ignore_cancellation_signals()
        if bool(signal_state["received"]):
            mark_cancelled(
                int(signal_state["signum"] or signal.SIGTERM),
                str(signal_state["failure_type"] or "SIGTERM"),
                finalization=True,
            )
            report["status"] = "failed-for-docker-memory"
            exit_code = 1
            if report_write_error is None:
                try:
                    write_report(report_path, report)
                except Exception as error:
                    report_write_error = type(error).__name__

        if report_write_error is not None:
            report["status"] = "failed-for-docker-memory"
            report["stage"] = "report_write"
            report["report_write"] = {
                "status": "failed",
                "error_type": report_write_error,
            }
            exit_code = 1
            # The old report was removed before execution. If a provisional PASS
            # was written but its required replacement failed, remove it rather
            # than leave stale successful evidence.
            try:
                report_path.unlink(missing_ok=True)
            except OSError:
                pass
        return exit_code

    previous_sigterm = signal.getsignal(signal.SIGTERM)
    previous_sigint = signal.getsignal(signal.SIGINT)
    try:
        try:
            signal.signal(signal.SIGTERM, handle_cancellation)
            signal.signal(signal.SIGINT, handle_cancellation)
            try:
                report_path.unlink(missing_ok=True)
                run_command(["docker", "version", "--format", "{{.Server.Version}}"])
                run_command(["docker", "compose", "version"])
                report["stage"] = "build"
                report["application_image"] = lab.build_application()
                report["stage"] = "compose_start"
                report["topology"] = lab.start()
                report["stage"] = "exercise"
                report["observations"] = lab.exercise()
                report["outcome"] = "pass-for-docker-memory"
            except KeyboardInterrupt:
                report["outcome"] = "cancelled"
                report["failure_type"] = str(
                    signal_state["failure_type"] or "KeyboardInterrupt"
                )
                if signal_state["signum"] is not None:
                    report["signal"] = int(signal_state["signum"])
                capture_failure_diagnostics()
            except TerminationRequested:
                raise
            except Exception as error:
                report["outcome"] = "failed-for-docker-memory"
                report["failure_type"] = type(error).__name__
                capture_failure_diagnostics()
            # A cancellation immediately before this transition raises through
            # the outer boundary after setting cleanup mode.
            enter_cleanup(signal_state)
        except (TerminationRequested, KeyboardInterrupt) as error:
            signum = (
                error.signum
                if isinstance(error, TerminationRequested)
                else int(signal_state["signum"] or signal.SIGINT)
            )
            failure_type = str(
                signal_state["failure_type"]
                or (
                    "SIGTERM"
                    if isinstance(error, TerminationRequested)
                    else "KeyboardInterrupt"
                )
            )
            mark_cancelled(signum, failure_type)
            capture_failure_diagnostics()
            signal_state["cleanup"] = True
        return finalize()
    finally:
        signal.signal(signal.SIGTERM, previous_sigterm)
        signal.signal(signal.SIGINT, previous_sigint)


def print_result(report_path: Path, exit_code: int) -> None:
    if exit_code == 0:
        print("PASS: three app containers served through a loopback-only Nginx proxy")
        print("PASS: one stopped instance left two stateless survivors and rejoined after restart")
        print("PASS: Memory Session was rejected cross-instance and lost after issuer restart")
        print("PASS: owned containers, networks, volumes, build container, app image and work state cleaned up")
    else:
        print("FAIL: YT-066a did not satisfy the bounded Docker/Memory gate")
    print(f"Report: {report_path.relative_to(ROOT)}")

def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--report",
        type=Path,
        default=DEFAULT_REPORT,
        help="sanitized JSON report path (default: target/yt066a/multi-instance.json)",
    )
    arguments = parser.parse_args(argv)
    path = arguments.report
    if not path.is_absolute():
        path = ROOT / path
    try:
        path.relative_to(ROOT)
    except ValueError:
        parser.error("report path must stay inside the repository")
    exit_code = run(path)
    print_result(path, exit_code)
    return exit_code


if __name__ == "__main__":
    raise SystemExit(main())
