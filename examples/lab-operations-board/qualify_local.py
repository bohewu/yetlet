#!/usr/bin/env python3
"""Bounded YT-074 local fault/load evidence; reuse the existing HTTP client.

No browser repetition, external services, production data, or automatic write retry.
"""
from __future__ import annotations
import argparse
import concurrent.futures
from datetime import datetime, timezone
import hashlib
from http.client import HTTPConnection
import json
import os
from pathlib import Path
import re
import signal
import socket
import sqlite3
import subprocess
import tempfile
import threading
import time
import urllib.parse

from verify_local import ROOT, Client, http, metrics, percentile, port_number, wait_until, stop_owned_process, remove_owned_tree

OUT = ROOT / "target/m11-board/sustained"
BINARY = ROOT / "target/release/yetlet-lab-operations-board"
PROFILES = {
    "bounded": {"warmup": 30, "rounds": [60, 60, 60], "soak": 300, "rps": 2},
    # Owner-selected pace preserves all 62 minutes below the unchanged history cap.
    "full": {"warmup": 120, "rounds": [600, 600, 600], "soak": 1800, "rps": 0.5, "stagger_cycle": True},
    # Separately authorized once after the paired 12-minute diagnostics.
    "full-sync": {"warmup": 120, "rounds": [600, 600, 600], "soak": 1800, "rps": 0.5, "stagger_cycle": False, "trace_sync": True},
    # Owner-authorized paired diagnostic; only cycle positions differ.
    "sync-12m": {"warmup": 120, "rounds": [600], "soak": 0, "rps": 0.5, "stagger_cycle": False, "trace_sync": True},
    "staggered-12m": {"warmup": 120, "rounds": [600], "soak": 0, "rps": 0.5, "stagger_cycle": True, "trace_sync": True},
}


def run(profile="bounded"):
    config = PROFILES[profile]
    out = OUT if profile == "bounded" else ROOT / "target/m11-board" / (profile + datetime.now(timezone.utc).strftime("-%Y%m%dT%H%M%S%fZ"))
    out.mkdir(parents=True, exist_ok=profile == "bounded")
    print(f"ARTIFACT_DIRECTORY {out.relative_to(ROOT)}", flush=True)
    temp = Path(tempfile.mkdtemp(prefix="owned-", dir=out))
    os.chmod(temp, 0o700)
    db = temp / "board.sqlite"
    server = None
    handles, clients, threads = [], [], []
    stop_event = threading.Event()
    port = port_number()
    base = f"http://127.0.0.1:{port}"
    report = {"status": "fail", "profile": profile, "started_utc": datetime.now(timezone.utc).isoformat(), "checks": [], "rounds": [], "resources": [], "stream_errors": [], "stream_connections": [0] * 10, "stream_logins": [0] * 10, "stream_events": [], "active_streams": 0, "cleanup": "pending"}
    report["envelope"] = {"initial_items": 10000, "initial_history": 90000, "clients": 25, "sse_clients": 10, "request_pattern": "four reads then one update", "max_requests_per_second_per_client": 2, "selected_requests_per_second_per_client": config["rps"], "warmup_seconds": config["warmup"], "round_seconds": config["rounds"], "soak_seconds": config["soak"], "session_relogin_seconds": 1200}
    report["envelope"]["initial_cycle_positions"] = [i % 5 if config.get("stagger_cycle", False) else 0 for i in range(25)]
    report["envelope"]["cycle_scheduling"] = "five staggered cohorts; each slot has 20 reads and 5 writes" if config.get("stagger_cycle", False) else "all clients start at cycle position zero; synchronized write bursts"
    report["thresholds"] = {"read_p95_ms_below": 250, "write_p95_ms_below": 500, "combined_p99_ms_below": 1000, "unexpected_response_fraction_below": 0.001}
    if config.get("trace_sync"):
        report["sync_tracing"] = {"tool": "strace -D -f --seccomp-bpf -qq -ttt -T -yy -e trace=fsync,fdatasync -e signal=none", "invocations": [], "effect": "syscall tracing can affect timings; traced timings are not uninstrumented qualification"}
    report["limits"] = ["single local Linux process and SQLite", "synthetic identity and data", "no real OS IME or spoken AT", "paced throughput, not maximum capacity", "SQL queue depth: Not instrumented", "bounded RSS observations do not prove leak freedom", "no SLA, HA or production rollout claim"]
    if profile == "bounded": report["limits"].append("not the proposed 62-minute experiment")
    if config.get("trace_sync") and profile != "full-sync": report["limits"].append("12-minute paired diagnostic only; no soak or full-duration qualification")
    report["fault_evidence"] = "executed separately before timing" if profile == "bounded" else "retained bounded run target/m11-board/sustained/report.json; no repeated fault injections"
    if profile in ("full", "full-sync"):
        previous = ROOT / "target/m11-board/sustained/report.json"
        report["retained_fault_report_sha256"] = hashlib.sha256(previous.read_bytes()).hexdigest()
    candidates = subprocess.check_output(["git", "ls-files", "--cached", "--others", "--exclude-standard", "-z"], cwd=ROOT).decode().split("\0")
    source_paths = sorted(ROOT / p for p in candidates if p and (ROOT / p).is_file() and (p.endswith((".rs", "Cargo.toml")) or p in ["Cargo.lock", "rust-toolchain.toml", "scripts/smoke_browser.py"] or (p.startswith("examples/lab-operations-board/") and not p.endswith(".md"))))
    identity = {str(p.relative_to(ROOT)): hashlib.sha256(p.read_bytes()).hexdigest() for p in source_paths}
    report["source_sha256"] = identity
    report["binary_sha256"] = hashlib.sha256(BINARY.read_bytes()).hexdigest()
    report["head"] = subprocess.check_output(["git", "rev-parse", "HEAD"], text=True).strip()
    report["dirty_state"] = subprocess.check_output(["git", "status", "--porcelain=v1", "--untracked-files=all"], cwd=ROOT, text=True).splitlines()
    report["diff_sha256"] = hashlib.sha256(subprocess.check_output(["git", "diff", "HEAD", "--binary"], cwd=ROOT)).hexdigest()
    cpu_model = re.search(r"^model name\s*:\s*(.+)$", Path("/proc/cpuinfo").read_text(), re.M)
    report["environment"] = {"os_release": Path("/etc/os-release").read_text().splitlines(), "kernel": os.uname().release, "machine": os.uname().machine, "cpu_model": cpu_model.group(1) if cpu_model else "unavailable", "logical_cpus": os.cpu_count(), "memory_total": Path("/proc/meminfo").read_text().splitlines()[0], "network": "same-host IPv4 loopback, no proxy", "rustc": subprocess.check_output([str(Path.home()/".cargo/bin/rustc"), "--version"], text=True).strip()}
    stream_lock = threading.Lock()
    observed_start = time.monotonic()

    def stream_event(index, event):
        with stream_lock:
            report["stream_events"].append({"reader": index, "event": event, "elapsed_seconds": time.monotonic() - observed_start})
            if event == "connected": report["active_streams"] += 1
            if event == "closed": report["active_streams"] -= 1

    def start(path=db, expect_ready=True):
        nonlocal server
        log = temp / f"server-{len(handles)}.log"
        handle = log.open("x"); os.chmod(log, 0o600); handles.append(handle)
        command = [str(BINARY)]
        if config.get("trace_sync"):
            trace = out / f"fsync-start-{len(handles)}.trace"
            command = ["/usr/bin/strace", "-D", "-f", "--seccomp-bpf", "-qq", "-ttt", "-T", "-yy", "-e", "trace=fsync,fdatasync", "-e", "signal=none", "-o", str(trace)] + command
        server = subprocess.Popen(command, cwd=ROOT, env=dict(os.environ,YETLET_BOARD_LISTEN=f"127.0.0.1:{port}",YETLET_BOARD_DB=str(path)), stdout=handle, stderr=handle, start_new_session=True)
        if not expect_ready:
            server.wait(timeout=8)
            assert server.returncode != 0
            server = None
            with socket.socket() as s: assert s.connect_ex(("127.0.0.1",port)) != 0
            return None
        wait_until(lambda: "READY " in log.read_text(), "Board startup", seconds=12)
        if config.get("trace_sync"):
            assert Path(f"/proc/{server.pid}/exe").resolve() == BINARY.resolve()
            tracer = re.search(r"^TracerPid:\s+(\d+)$", Path(f"/proc/{server.pid}/status").read_text(), re.M)
            assert tracer and int(tracer.group(1)) > 0
            report["sync_tracing"]["invocations"].append({"application_pid": server.pid, "tracer_pid": int(tracer.group(1)), "trace": trace.name, "port": port})
        assert http(base + "/readyz")[0] == 200
        codes = {m.group(1):m.group(2) for m in re.finditer(r"^LOCAL_LOGIN (alice|bob|viewer) ([0-9a-f]{64})$", log.read_text(), re.M)}
        assert len(codes) == 3
        return codes

    def stop():
        nonlocal server
        assert server is not None
        t = time.monotonic()
        server.send_signal(signal.SIGINT); server.wait(timeout=6)
        assert server.returncode == 0
        server = None
        with socket.socket() as s: assert s.connect_ex(("127.0.0.1",port)) != 0
        return time.monotonic() - t

    def client(codes):
        c = Client(port); clients.append(c); c.login(codes); return c

    try:
        print("STAGE initialize isolated database", flush=True)
        start(); stop()
        with sqlite3.connect(db) as c:
            c.execute("BEGIN")
            c.executemany("INSERT INTO work_items(id,project,title,description,status,priority,assignee,version) VALUES(?,1,?,'Synthetic local qualification','open','normal','',9)", ((i,f"Work record {i:05}") for i in range(1,10001)))
            c.executemany("INSERT INTO history(item_id,project,version,actor,action,title,status) VALUES(?,1,?,'alice',?,?,'open')", ((i,v,"create" if v == 1 else "update",f"Work record {i:05}") for i in range(1,10001) for v in range(1,10)))
            c.commit()
        codes = start()
        if profile == "bounded":
            c = client(codes)
            print("STAGE contention and unread-response fault cases", flush=True)
            with sqlite3.connect(db) as lock:
                lock.execute("BEGIN IMMEDIATE")
                status, _, _ = c.call("POST", "/board/projects/1/items/1", c.fields("Must not commit",9))
                assert status == 503
                assert c.call("GET", "/healthz")[0] == 200
                lock.rollback()
            with sqlite3.connect(db) as check:
                assert check.execute("SELECT version FROM work_items WHERE id=1").fetchone()[0] == 9
                assert check.execute("SELECT COUNT(*) FROM history").fetchone()[0] == 90000
            assert c.call("POST", "/board/projects/1/items/1", c.fields("Recovered after contention",9))[0] == 303
            report["checks"].append("actual writer-lock contention returns 503 without mutation; health remains 200; explicit later write succeeds")
            body = urllib.parse.urlencode(c.fields("Committed after response loss",10)).encode()
            request = (f"POST /board/projects/1/items/1 HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nOrigin: {base}\r\nCookie: {c.cookie}\r\nContent-Type: application/x-www-form-urlencoded\r\nContent-Length: {len(body)}\r\nConnection: close\r\n\r\n").encode() + body
            with socket.create_connection(("127.0.0.1",port),timeout=5) as lost:
                lost.sendall(request)
                # Leave response unread until authoritative storage confirms this exact write.
                def committed():
                    with sqlite3.connect(db) as check:
                        return check.execute("SELECT version FROM work_items WHERE id=1").fetchone()[0] == 11
                wait_until(committed, "authoritative commit after response loss", seconds=5)
            assert c.call("GET", "/board/projects/1/items/1")[2].count("Committed after response loss") >= 1
            assert c.call("POST", "/board/projects/1/items/1", c.fields("Explicit stale draft",10))[0] == 409
            report["checks"].append("real HTTP write committed with response unread; authoritative read resolves it; explicit stale version rejected with 409; no automatic replay")
            old_cookie = c.cookie
            for x in clients: x.close()
            clients.clear()
            stop()
            backup = temp / "known-good.sqlite"
            with sqlite3.connect(db) as original, sqlite3.connect(backup) as saved: original.backup(saved)
            candidate = temp / "incompatible.sqlite"
            with sqlite3.connect(backup) as saved, sqlite3.connect(candidate) as future:
                saved.backup(future); future.execute("PRAGMA user_version=2")
            start(candidate, expect_ready=False)
            codes = start(backup)
            retired = Client(port); clients.append(retired); retired.cookie = old_cookie
            assert retired.call("GET", "/board/projects/1/items/1")[0] == 401
            current = client(codes)
            assert "Committed after response loss" in current.call("GET", "/board/projects/1/items/1")[2]
            report["checks"].append("incompatible schema candidate refuses startup; rollback to known-good version-1 database retains latest data and requires fresh Memory login")
            db = backup
            versions = [11] + [9] * 24
        else:
            current = client(codes)
            versions = [9] * 25
        stream_ready = [threading.Event() for _ in range(10)]
        def stream_worker(index):
            connection = None
            logged_in = Client(port)
            login_time = None
            connected = False
            try:
                while not stop_event.is_set():
                    if login_time is None or time.monotonic() - login_time >= 1200:
                        logged_in.login(codes)
                        login_time = time.monotonic()
                        report["stream_logins"][index] += 1
                        stream_event(index, "login")
                    connection = HTTPConnection("127.0.0.1",port,timeout=12)
                    connection.request("GET", "/board/projects/1/events", headers={"Cookie":logged_in.cookie})
                    response = connection.getresponse()
                    assert response.status == 200
                    report["stream_connections"][index] += 1
                    stream_event(index, "connected"); connected = True
                    while not stop_event.is_set():
                        line = response.readline(8192)
                        if not line: break
                        assert len(line) < 8192
                        if line.startswith(b"data:"):
                            data = json.loads(line[5:])
                            assert data["total"] == 10000
                            stream_ready[index].set()
                    connection.close(); connection = None
                    stream_event(index, "closed"); connected = False
            except Exception as error:
                if not stop_event.is_set():
                    report["stream_errors"].append(type(error).__name__)
                    stop_event.set()
            finally:
                if connection is not None: connection.close()
                if connected: stream_event(index, "closed")
                logged_in.close()
        for index in range(10):
            thread = threading.Thread(target=stream_worker,args=(index,));thread.start();threads.append(thread)
        assert all(event.wait(12) for event in stream_ready)
        monitored_pid = server.pid
        def monitor():
            start_time = time.monotonic()
            while not stop_event.wait(1):
                try:
                    row=metrics(monitored_pid)
                    row.update(elapsed_seconds=time.monotonic()-start_time,threads=len(list(Path(f"/proc/{monitored_pid}/task").iterdir())),fds=len(list(Path(f"/proc/{monitored_pid}/fd").iterdir())),active_sse=report["active_streams"])
                    report["resources"].append(row)
                except (OSError,ProcessLookupError): break
        t = threading.Thread(target=monitor);t.start();threads.append(t)

        def phase(label,seconds,retain=True):
            print(f"STAGE {label} seconds={seconds}",flush=True)
            gate=threading.Barrier(25)
            all_rows, errors, login_counts = [], [], [0] * 25
            rows_lock = threading.Lock()
            def worker(index):
                connection=Client(port)
                try:
                    connection.login(codes)
                    login_counts[index] += 1
                    login_time=time.monotonic()
                    gate.wait(timeout=15)
                    # Owner-approved mixed workload: rotate the same five-step
                    # cycle without lowering any client's pace or write share.
                    begin=time.monotonic(); end=begin+seconds; operation=index%5 if config.get("stagger_cycle", False) else 0; due=begin
                    while time.monotonic()<end and not stop_event.is_set():
                        if stop_event.wait(max(0,due-time.monotonic())): break
                        if time.monotonic()>=end: break
                        if time.monotonic()-login_time >= 1200:
                            connection.login(codes)
                            login_counts[index] += 1
                            login_time=time.monotonic()
                        # No catch-up bursts after a slow operation or explicit re-login.
                        due=time.monotonic()+1/config["rps"]
                        write=operation%5==4; then=time.perf_counter_ns()
                        if write:
                            status=connection.call("POST",f"/board/projects/1/items/{index+1}",connection.fields(f"Worker {index:02}",versions[index]))[0]
                            if status==303: versions[index]+=1
                        else:
                            status=connection.call("GET","/board/projects/1/items")[0]
                        elapsed=(time.perf_counter_ns()-then)/1e6
                        with rows_lock: all_rows.append(("write" if write else "read",elapsed,status))
                        if status!=(303 if write else 200): raise RuntimeError("unexpected response; no automatic mutation retry")
                        operation+=1
                except Exception as error:
                    with rows_lock: errors.append({"client":index,"type":type(error).__name__})
                    stop_event.set(); gate.abort()
                finally: connection.close()
            before=metrics(server.pid); started_unix=time.time(); begin=time.monotonic()
            with concurrent.futures.ThreadPoolExecutor(max_workers=25) as pool:
                list(pool.map(worker,range(25)))
            wall=time.monotonic()-begin; after=metrics(server.pid)
            finished_unix=time.time()
            unexpected=sum(status != (303 if kind=="write" else 200) for kind,_,status in all_rows)
            result={"name":label,"configured_seconds":seconds,"wall_seconds_including_login":wall,"requests":len(all_rows),"unexpected_responses":unexpected,"errors":errors,"logins":sum(login_counts),"authentication_requests_excluded_from_mix":3*sum(login_counts),"achieved_rps_including_login_wall":len(all_rows)/wall,"cpu_seconds":after["cpu_seconds"]-before["cpu_seconds"],"rss_before_kib":before["rss_kib"],"rss_after_kib":after["rss_kib"]}
            if config.get("trace_sync"):
                result.update(started_unix=started_unix, finished_unix=finished_unix)
            for kind in ["read","write"]:
                values=[r[1] for r in all_rows if r[0]==kind]
                result[kind]={"count":len(values),"p50_ms":percentile(values,.5) if values else None,"p95_ms":percentile(values,.95) if values else None,"p99_ms":percentile(values,.99) if values else None,"max_ms":max(values) if values else None}
            result["combined_p99_ms"]=percentile([r[1] for r in all_rows],.99) if all_rows else None
            result["combined_max_ms"]=max((r[1] for r in all_rows),default=None)
            result["thresholds_pass"]=bool(all_rows and result["write"]["count"] and result["read"]["count"] and result["read"]["p95_ms"]<250 and result["write"]["p95_ms"]<500 and result["combined_p99_ms"]<1000 and unexpected/len(all_rows)<0.001 and not errors and not stop_event.is_set())
            (out/f"{label}-samples.json").write_text(json.dumps(all_rows)+"\n")
            if retain:
                report["rounds"].append(result)
            else:
                report["warmup"] = result
            print(json.dumps(result),flush=True)
            assert not errors and not unexpected and not stop_event.is_set()
            # Full-profile acceptance covers measured rounds; retain warmup tails
            # as evidence without treating them as steady-state measurements.
            if retain or profile == "bounded": assert result["thresholds_pass"]
            assert not report["stream_errors"]
        phase("warmup",config["warmup"],False)
        for n,seconds in enumerate(config["rounds"],1): phase(f"round-{n}",seconds)
        if config["soak"]:
            phase("soak",config["soak"])
        with sqlite3.connect(db) as check:
            report["final_data"]={"items":check.execute("SELECT COUNT(*) FROM work_items").fetchone()[0],"history":check.execute("SELECT COUNT(*) FROM history").fetchone()[0]}
            assert report["final_data"]["items"]==10000 and report["final_data"]["history"]<100000
            initial_history=90002 if profile=="bounded" else 90000
            writes=report["warmup"]["write"]["count"]+sum(r["write"]["count"] for r in report["rounds"])
            assert report["final_data"]["history"]==initial_history+writes
            assert check.execute("PRAGMA integrity_check").fetchone()[0]=="ok"
        stop_event.set()
        for x in clients:x.close()
        clients.clear()
        report["graceful_shutdown_seconds"] = stop()
        for t in threads:t.join(timeout=15);assert not t.is_alive()
        assert not report["stream_errors"] and all(n>=2 for n in report["stream_connections"])
        assert report["active_streams"]==0
        report["checks"].append("ten SSE streams read throughout load and reconnect after the five-minute lifetime; graceful exit joins readers")
        assert identity == {str(p.relative_to(ROOT)):hashlib.sha256(p.read_bytes()).hexdigest() for p in source_paths}
        assert report["binary_sha256"]==hashlib.sha256(BINARY.read_bytes()).hexdigest()
        report["source_and_binary_unchanged"]=True
        report["status"]="pass"
    except Exception as error:
        report["failure_type"]=type(error).__name__
        raise
    finally:
        stop_event.set()
        for x in clients:x.close()
        stop_owned_process(server,process_group=True)
        for t in threads:t.join(timeout=15)
        assert all(not t.is_alive() for t in threads)
        for handle in handles:handle.close()
        remove_owned_tree(temp)
        with socket.socket() as s:assert s.connect_ex(("127.0.0.1",port))!=0
        if config.get("trace_sync"):
            for invocation in report["sync_tracing"]["invocations"]:
                for pid in [invocation["application_pid"], invocation["tracer_pid"]]:
                    wait_until(lambda pid=pid: not Path(f"/proc/{pid}").exists() or ") Z " in Path(f"/proc/{pid}/stat").read_text(), "owned app/tracer exit", seconds=5)
            report["sync_tracing"]["cleanup"] = "owned applications exited; tracers exited (absent or zombie); loopback listener closed"
        report["cleanup"]="owned process reaped, threads joined, listener closed, private synthetic databases/logs removed"
        report["finished_utc"]=datetime.now(timezone.utc).isoformat()
        (out/"report.json").write_text(json.dumps(report,indent=2)+"\n")
        artifacts=sorted(out.glob("*.json"))
        (out/"SHA256SUMS").write_text("".join(f"{hashlib.sha256(p.read_bytes()).hexdigest()}  {p.name}\n" for p in artifacts))
        print(json.dumps({k:v for k,v in report.items() if k not in ['source_sha256','resources','stream_events']},indent=2),flush=True)

if __name__ == "__main__":
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--profile",choices=PROFILES,default="bounded")
    args=parser.parse_args()
    run(args.profile)
