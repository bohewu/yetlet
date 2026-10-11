#!/usr/bin/env python3
"""One disposable three-app Redis qualification; no shared or remote services."""
from __future__ import annotations
from concurrent.futures import ThreadPoolExecutor
from http.cookiejar import CookieJar
import hashlib
import json
import os
from pathlib import Path
import secrets
import shutil
import signal
import sys
import time
from urllib.error import HTTPError
from urllib.request import Request, build_opener, ProxyHandler, HTTPCookieProcessor

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "scripts"))
import qualify_multi_instance as existing

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import browser as browser_probe
SCRATCH = ROOT / "target" / "yt066b3-20261011"
REDIS_IMAGE = "redis@sha256:47670742d7924adbcdb404d1288b6327815b23141969c0939b8e5d61f78c2634"
require = existing.require
command = existing.run_command


class Lab(existing.DockerLab):
    def __init__(self):
        super().__init__()
        token = secrets.token_hex(4)
        self.project = f"yt066b3-{os.getpid()}-{token}"
        self.build_container = self.project + "-build"
        self.app_image = "yetlet-yt066b3:" + token
        self.work = SCRATCH / ("work-" + token)
        self.build = self.work / "build"
        self.image_context = self.work / "image"
        self.logs = SCRATCH / ("logs-" + token)
        self.namespace = "b3_" + token
        self.env = dict(os.environ, YT066B3_APP_IMAGE=self.app_image,
                        YT066B3_HOST_PORT=str(self.host_port), YT066B3_NAMESPACE=self.namespace)

    def compose_argv(self, *args):
        return ["docker", "compose", "--ansi", "never", "-p", self.project,
                "-f", str(HERE / "compose.yaml"), *args]

    def build_application(self):
        self.build.mkdir(parents=True)
        self.image_context.mkdir()
        self.logs.mkdir(parents=True)
        vendor = self.work / "vendor"
        existing.run_logged(["cargo", "vendor", "--locked", "--offline", "--manifest-path",
                             str(HERE / "Cargo.toml"), str(vendor)], self.logs / "vendor.log")
        uid, gid = os.getuid(), os.getgid()
        script = '''set -euo pipefail
export CARGO_HOME=/tmp/cargo
mkdir -p "$CARGO_HOME"
printf '[source.crates-io]\nreplace-with="vendored-sources"\n[source.vendored-sources]\ndirectory="/vendor"\n' > "$CARGO_HOME/config.toml"
rustc -vV
cargo --version
cargo build --release --locked --offline --manifest-path qualification/yt066b3/Cargo.toml
'''
        existing.run_logged(["docker", "run", "--rm", "--name", self.build_container,
            "--network", "none", "--read-only", "--user", f"{uid}:{gid}", "--cpus", "4",
            "--memory", "6g", "--pids-limit", "384", "--cap-drop", "ALL",
            "--security-opt", "no-new-privileges", "--tmpfs", "/tmp:rw,exec,size=1073741824,mode=1777",
            "--mount", f"type=bind,src={ROOT},dst=/src,readonly",
            "--mount", f"type=bind,src={vendor},dst=/vendor,readonly",
            "--mount", f"type=bind,src={self.build},dst=/build",
            "--env", "CARGO_TARGET_DIR=/build", "--workdir", "/src", existing.RUST_IMAGE,
            "bash", "-ceu", script], self.logs / "build.log", timeout=900)
        binary = self.image_context / "yetlet-redis-qualification"
        shutil.copy2(self.build / "release" / binary.name, binary)
        binary.chmod(0o555)
        existing.run_logged(["docker", "build", "--network=none", "--pull=false", "-t",
            self.app_image, "-f", str(HERE / "Dockerfile"), str(self.image_context)],
            self.logs / "image.log", timeout=300)
        return {"binary_sha256": hashlib.sha256(binary.read_bytes()).hexdigest(),
                "image_id": command(["docker", "image", "inspect", self.app_image,
                                     "--format", "{{.Id}}"] ).stdout.strip()}

    def request(self, instance, path, method="GET", cookie=None, hold=None, opener=None):
        headers = {}
        if cookie: headers["Cookie"] = cookie
        if hold: headers["X-Lab-Hold"] = hold
        url = self.base + (f"/__yt066b3/{instance}" if instance else "") + path
        opener = opener or build_opener(ProxyHandler({}))
        try:
            r = opener.open(Request(url, method=method, headers=headers), timeout=5)
        except HTTPError as e:
            r = e
        with r:
            return r.code, {k.lower(): v for k, v in r.headers.items()}, r.read().decode()

    def redis(self, *args):
        return self.compose("exec", "-T", "redis", "redis-cli", "--raw", *args).stdout.strip()

    def gate(self, instance, action):
        return self.request(instance, "/control/" + action,
                            "GET" if action == "entered" else "POST")

    def entered(self, instance):
        end = time.monotonic() + 2
        while time.monotonic() < end:
            if self.gate(instance, "entered")[2] == "1": return
            time.sleep(0.002)
        raise AssertionError("fixture gate not entered")

    def create(self, instance="a", short=False):
        r = self.request(instance, "/short/create" if short else "/create", "POST")
        require(r[0] == 201, "explicit create failed")
        return existing.cookie(r[1])

    def records(self):
        return set(filter(None, self.redis("--scan", "--pattern",
            f"yetlet:redis:v1:{self.namespace}:session:*").splitlines()))

    def hidden_cookie(self, previous):
        created = self.records() - previous
        require(len(created) == 1, "held create must allocate exactly one identity")
        return "yt_b3=" + created.pop().rsplit(":", 1)[1]

    def reads(self, cookie, count=None, status=200, path="/read"):
        for x in "abc":
            r = self.request(x, path, cookie=cookie)
            require(r[0] == status, "cross-app authoritative read category mismatch")
            require("set-cookie" not in r[1], "read emitted Cookie")
            if count is not None: require(json.loads(r[2])["count"] == count, "cross-app state mismatch")

    def start(self):
        self.compose("config", "--quiet")
        self.compose_started = True  # includes a partial failed start for cleanup
        self.compose("up", "-d", "--wait", "--wait-timeout", "30", timeout=60)
        for x in "abc":
            self.ready(x)
        details = {}
        for name in ["app-a", "app-b", "app-c", "redis", "proxy"]:
            cid = self.compose("ps", "-q", name).stdout.strip()
            details[name] = json.loads(command(["docker", "inspect", cid]).stdout)[0]
        backend = f"{self.project}_lab"
        for name in ["app-a", "app-b", "app-c", "redis"]:
            require(not any((details[name]["NetworkSettings"]["Ports"] or {}).values()), "backend host publication")
            require(set(details[name]["NetworkSettings"]["Networks"]) == {backend}, "backend network shape")
            require(details[name]["HostConfig"]["ReadonlyRootfs"], "backend writable root")
        binding = details["proxy"]["NetworkSettings"]["Ports"]["8080/tcp"]
        require(binding == [{"HostIp": "127.0.0.1", "HostPort": str(self.host_port)}], "loopback proxy publication")
        networks = json.loads(command(["docker", "network", "inspect", backend]).stdout)
        require(networks[0]["Internal"], "backend must be internal")
        require(self.redis("CONFIG", "GET", "appendonly") == "appendonly\nno", "AOF disabled")
        require(self.redis("CONFIG", "GET", "save").splitlines() == ["save"], "RDB disabled")
        return {"applications": 3, "redis_primaries": 1, "backend_host_ports": 0,
                "proxy_listener": "127.0.0.1", "backend_internal": True,
                "redis_persistence": "none; tmpfs", "redis_version": self.redis("INFO", "server").split("redis_version:")[1].splitlines()[0],
                "nginx_version": self.compose("exec", "-T", "proxy", "nginx", "-v").stderr.strip()}

    def ready(self, instance):
        end = time.monotonic() + 10
        while time.monotonic() < end:
            try:
                if self.request(instance, "/healthz")[0] == 200: return
            except OSError: pass
            time.sleep(0.05)
        raise AssertionError("app readiness deadline")

    def cleanup(self):
        result = super().cleanup()  # existing reviewed exact-owner removal/rechecks
        result["owned_containers"] = 5
        return result


def exercise(lab):
    observations = {}
    cookie = lab.create()
    lab.reads(cookie, 1)
    pooled = [lab.request(None, "/healthz") for _ in range(12)]
    require({r[2] for r in pooled} == set("abc"), "stateless distribution")
    require(all("set-cookie" not in r[1] for r in pooled), "stateless scope")
    observations["three_app_create_read_and_stateless_distribution"] = "pass"
    with ThreadPoolExecutor(max_workers=4) as pool:
        for x in "ab": lab.gate(x, "reset")
        futures = [pool.submit(lab.request, x, "/update", "POST", cookie, "cas") for x in "ab"]
        for x in "ab": lab.entered(x)
        for x in "ab": lab.gate(x, "release")
        results = [f.result() for f in futures]
        require(sorted(r[0] for r in results) == [200, 409], "exactly one CAS winner")
        require(all(json.loads(r[2])["callbacks"] == 1 for r in results), "once-only callbacks")
        lab.reads(cookie, 2)
        observations["cross_process_cas_once_only"] = "pass"

        # Before-finalizer race: observe the private synthetic identity, retire it,
        # then allow the real provider middleware to finalize the original response.
        before = lab.records(); lab.gate("a", "reset")
        pending = pool.submit(lab.request, "a", "/create", "POST", None, "before")
        lab.entered("a"); hidden = lab.hidden_cookie(before)
        replacement = lab.request("b", "/rotate", "POST", hidden)
        require(replacement[0] == 200, "rotation of held identity")
        fresh = existing.cookie(replacement[1]); lab.gate("a", "release")
        old = pending.result(); require(old[0] == 201 and "set-cookie" not in old[1], "pre-header stale suppression")
        lab.reads(hidden, status=401); lab.reads(fresh, 1)
        observations["staleness_observed_by_finalizer"] = "pass"

        # Real delayed HTTP response and standard-library HTTP Cookie jar.
        jar = CookieJar(); opener = build_opener(ProxyHandler({}), HTTPCookieProcessor(jar))
        before = lab.records(); lab.gate("a", "reset")
        pending = pool.submit(lab.request, "a", "/create", "POST", None, "after", opener)
        lab.entered("a"); hidden = lab.hidden_cookie(before)
        replacement = lab.request("b", "/rotate", "POST", hidden, opener=opener)
        require(replacement[0] == 200, "replacement before old response")
        fresh = existing.cookie(replacement[1]); lab.gate("a", "release")
        require(pending.result()[0] == 201, "prepared response delivery")
        require(lab.request("c", "/read", opener=opener)[0] == 401, "late Cookie must fail closed")
        lab.reads(fresh, 1)
        require(len(jar) == 1, "one fixed Cookie")
        lab.request("c", "/create", "POST", opener=opener)
        require(lab.request("b", "/read", opener=opener)[0] == 200, "explicit recovery")
        observations["prepared_response_reordering_permits_login_loss_only"] = "pass"

        # Delayed logout does not clear a later login's Cookie.
        old_cookie = lab.create(); lab.gate("a", "reset")
        pending = pool.submit(lab.request, "a", "/destroy", "POST", old_cookie, "before")
        lab.entered("a"); new_cookie = lab.create("b"); lab.gate("a", "release")
        require("set-cookie" not in pending.result()[1], "logout has no clear")
        lab.reads(old_cookie, status=401); lab.reads(new_cookie, 1)
        competing = [pool.submit(lab.request, x, "/rotate", "POST", new_cookie) for x in "bc"]
        responses = [f.result() for f in competing]
        require(sum(r[0] == 200 for r in responses) == 1, "one rotation winner")
        require(all("set-cookie" not in r[1] for r in responses if r[0] != 200), "loser Cookie suppression")
        winner = existing.cookie(next(r[1] for r in responses if r[0] == 200))
        lab.reads(new_cookie, status=401); lab.reads(winner, 1)
        require(lab.request("c", "/destroy", "POST", winner)[0] == 200, "revocation")
        lab.reads(winner, status=401)
        observations["cross_app_rotate_revoke_delayed_logout"] = "pass"

        # A write is held AFTER both snapshot read and once-only callback entry.
        uncertain = lab.create(); lab.gate("a", "reset")
        calls_before = json.loads(lab.request("a", "/read", cookie=uncertain)[2])["callbacks"]
        pending = pool.submit(lab.request, "a", "/update", "POST", uncertain, "cas")
        lab.entered("a"); lab.redis("CLIENT", "PAUSE", "700", "WRITE"); lab.gate("a", "release")
        result = pending.result(); require(result[0] == 202 and json.loads(result[2])["category"] == "unknown", "uncertain dispatch")
        require("set-cookie" not in result[1], "unknown emits no Cookie")
        require(json.loads(result[2])["callbacks"] == calls_before + 1, "unknown callback once")
        require(lab.request("a", "/read", cookie=uncertain)[0] == 504, "unknown retains slot")
        time.sleep(0.35)
        require(lab.request("b", "/update", "POST", uncertain)[0] == 200, "subsequent worker update")
        replacement = lab.request("c", "/rotate", "POST", uncertain)
        require(replacement[0] == 200, "subsequent rotation")
        fresh = existing.cookie(replacement[1]); lab.reads(fresh, 3)
        require(json.loads(lab.request("a", "/operation")[2])["category"] == "committed", "historical result independent of retirement")
        lab.reads(uncertain, status=401)
        observations["unknown_outcome_then_other_worker_update_rotate_and_resolve"] = "pass"

        time.sleep(5.1)
        require(json.loads(lab.request("a", "/operation")[2])["category"] == "unresolved", "expired result stays unresolved")
        lab.reads(uncertain, status=401)
        observations["expired_result_reconciliation_without_replay"] = "pass"

        # Cancellation retains a dispatched worker; shutdown reaps it while paused.
        cancel_cookie = lab.create(); lab.gate("a", "reset")
        pending = pool.submit(lab.request, "a", "/cancel-update", "POST", cancel_cookie)
        lab.entered("a"); lab.redis("CLIENT", "PAUSE", "5000", "WRITE"); lab.gate("a", "release")
        require(json.loads(pending.result()[2])["category"] == "caller_cancelled", "cancelled caller")
        # One active slot + one waiter: another read must be rejected as busy.
        waiter = pool.submit(lab.request, "a", "/read", "GET", cancel_cookie)
        time.sleep(0.02)
        require(lab.request("a", "/read", cookie=cancel_cookie)[0] == 429, "bounded admission")
        require(waiter.result()[0] == 504, "bounded wait")
        started = time.monotonic()
        require(lab.request("a", "/control/shutdown", "POST")[0] == 200, "explicit driver drain")
        require(time.monotonic() - started < 3, "bounded shutdown")
        require(lab.request("a", "/read", cookie=cancel_cookie)[0] == 503, "closed admission")
        require(lab.request("a", "/healthz")[0] == 200, "stateless route after shutdown")
        lab.redis("CLIENT", "UNPAUSE")
        observations["caller_cancellation_capacity_wait_and_shutdown"] = "pass"

    retained = lab.create("b")
    lab.compose("restart", "app-a"); lab.ready("a"); lab.reads(retained, 1)
    lab.compose("stop", "app-a")
    require({lab.request(None, "/healthz")[2] for _ in range(12)} == set("bc"), "two survivors")
    lab.compose("start", "app-a"); lab.ready("a"); lab.reads(retained, 1)
    observations["app_stop_restart_retains_redis_state"] = "pass"

    short = lab.create(short=True)
    time.sleep(0.3)
    require(lab.request("b", "/short/update", "POST", short)[0] == 200, "short TTL update")
    time.sleep(0.4); lab.reads(short, status=401, path="/short/read")
    observations["absolute_ttl_not_refreshed_by_update"] = "pass"
    lab.compose("stop", "redis")
    for x in "abc":
        require(lab.request(x, "/read", cookie=retained)[0] in (503, 504), "outage fail closed")
        require(lab.request(x, "/healthz")[0] == 200, "stateless during outage")
    lab.compose("start", "redis")
    lab.compose("up", "-d", "--wait", "--wait-timeout", "20", "redis")
    require(not lab.records(), "no-persistence restart clears records")
    # Deliberate app restart, not a claim of automatic reconnect recovery.
    lab.compose("restart", "app-a", "app-b", "app-c")
    for x in "abc": lab.ready(x)
    lab.reads(retained, status=401)
    renewed = lab.create("c"); lab.reads(renewed, 1)
    observations["outage_and_controlled_no_persistence_restart"] = "pass; explicit app restart recovery"
    return observations


def main():
    SCRATCH.mkdir(parents=True, exist_ok=True)
    lab = Lab(); report = {"task_id": "YT-066b3", "status": "failed", "project": lab.project,
        "source": existing.workspace_identity(), "images": {"rust": existing.RUST_IMAGE,
        "nginx": existing.NGINX_IMAGE, "redis": REDIS_IMAGE}, "stage": "build"}
    cancelled = []; cleaning = False
    def cancel(signum, _frame):
        nonlocal cleaning
        cancelled.append(signum)
        if not cleaning:
            cleaning = True
            raise existing.TerminationRequested(signum)
    def enter_browser_cleanup():
        nonlocal cleaning
        cleaning = True
    previous = {s: signal.getsignal(s) for s in (signal.SIGINT, signal.SIGTERM)}
    for s in previous: signal.signal(s, cancel)
    try:
        try:
            try:
                report["application_image"] = lab.build_application()
                report["stage"] = "topology"; report["topology"] = lab.start()
                report["stage"] = "exercise"; report["observations"] = exercise(lab)
                report["stage"] = "browser"
                # Same process: cancellation unwinds Chrome's finally before Docker cleanup.
                report["browser"] = browser_probe.run(lab.base, enter_browser_cleanup)
                report["outcome"] = "pass"
            except BaseException as error:
                report["failure_type"] = type(error).__name__
            # The outer boundary covers a first cancellation at this transition.
            cleaning = True
        except existing.TerminationRequested as error:
            report["failure_type"] = type(error).__name__
            cleaning = True
        report["cleanup"] = lab.cleanup()
        if lab.work.exists(): shutil.rmtree(lab.work)
        report["owned_work_removed"] = not lab.work.exists()
        if cancelled: report["cancellation"] = cancelled
        if (report.get("outcome") == "pass" and not cancelled and
            report["cleanup"]["status"] == "pass" and report["owned_work_removed"]):
            report["status"] = "pass-at-single-host-no-persistence-scope"
        report_path = SCRATCH / (lab.project + ".json")
        try:
            existing.write_report(report_path, report)
            # Close the signal window after report I/O, then reconcile cancellation.
            for s in previous: signal.signal(s, signal.SIG_IGN)
            if cancelled:
                report["status"] = "failed"
                report["cancellation"] = cancelled
                existing.write_report(report_path, report)
        except BaseException:
            report_path.unlink(missing_ok=True)
            raise
        print(json.dumps({k:report[k] for k in ("status", "project", "stage", "cleanup")}, indent=2))
        return 0 if report["status"].startswith("pass-") else 1
    finally:
        for s, handler in previous.items(): signal.signal(s, handler)


if __name__ == "__main__":
    raise SystemExit(main())
