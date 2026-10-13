#!/usr/bin/env python3
"""Bounded YT-074 local fault/load evidence; reuse the existing HTTP client.

No browser repetition, external services, production data, or automatic write retry.
"""
from __future__ import annotations
import concurrent.futures
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


def run():
    OUT.mkdir(parents=True, exist_ok=True)
    temp = Path(tempfile.mkdtemp(prefix="owned-", dir=OUT))
    os.chmod(temp, 0o700)
    db = temp / "board.sqlite"
    server = None
    handles, clients, threads = [], [], []
    stop_event = threading.Event()
    port = port_number()
    base = f"http://127.0.0.1:{port}"
    report = {"status": "fail", "checks": [], "rounds": [], "resources": [], "stream_errors": [], "stream_connections": [0] * 10, "cleanup": "pending"}
    report["envelope"] = {"initial_items": 10000, "initial_history": 90000, "clients": 25, "sse_clients": 10, "request_pattern": "four reads then one update", "max_requests_per_second_per_client": 2, "warmup_seconds": 30, "round_seconds": [60,60,60], "soak_seconds": 300}
    report["limits"] = ["single local Linux process and SQLite", "synthetic identity and data", "no real OS IME or spoken AT", "not the proposed 62-minute experiment", "no SLA, HA or production rollout claim"]
    source_paths = sorted(p for p in (ROOT / "examples/lab-operations-board").rglob("*") if p.is_file() and "__pycache__" not in p.parts) + [ROOT / "Cargo.toml", ROOT / "Cargo.lock"]
    identity = {str(p.relative_to(ROOT)): hashlib.sha256(p.read_bytes()).hexdigest() for p in source_paths}
    report["source_sha256"] = identity
    report["binary_sha256"] = hashlib.sha256(BINARY.read_bytes()).hexdigest()
    report["head"] = subprocess.check_output(["git", "rev-parse", "HEAD"], text=True).strip()
    report["environment"] = {"kernel": os.uname().release, "machine": os.uname().machine, "logical_cpus": os.cpu_count(), "rustc": subprocess.check_output([str(Path.home()/".cargo/bin/rustc"), "--version"], text=True).strip()}

    def start(path=db, expect_ready=True):
        nonlocal server
        log = temp / f"server-{len(handles)}.log"
        handle = log.open("x"); os.chmod(log, 0o600); handles.append(handle)
        server = subprocess.Popen([str(BINARY)], cwd=ROOT, env=dict(os.environ,YETLET_BOARD_LISTEN=f"127.0.0.1:{port}",YETLET_BOARD_DB=str(path)), stdout=handle, stderr=handle, start_new_session=True)
        if not expect_ready:
            server.wait(timeout=8)
            assert server.returncode != 0
            server = None
            with socket.socket() as s: assert s.connect_ex(("127.0.0.1",port)) != 0
            return None
        wait_until(lambda: "READY " in log.read_text(), "Board startup", seconds=12)
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
        sse_cookie = current.cookie
        stream_ready = [threading.Event() for _ in range(10)]
        def stream_worker(index):
            connection = None
            try:
                while not stop_event.is_set():
                    connection = HTTPConnection("127.0.0.1",port,timeout=12)
                    connection.request("GET", "/board/projects/1/events", headers={"Cookie":sse_cookie})
                    response = connection.getresponse()
                    assert response.status == 200
                    report["stream_connections"][index] += 1
                    while not stop_event.is_set():
                        line = response.readline(8192)
                        if not line: break
                        assert len(line) < 8192
                        if line.startswith(b"data:"):
                            data = json.loads(line[5:])
                            assert data["total"] == 10000
                            stream_ready[index].set()
                    connection.close(); connection = None
            except Exception as error:
                if not stop_event.is_set(): report["stream_errors"].append(type(error).__name__)
            finally:
                if connection is not None: connection.close()
        for index in range(10):
            thread = threading.Thread(target=stream_worker,args=(index,));thread.start();threads.append(thread)
        assert all(event.wait(12) for event in stream_ready)
        monitored_pid = server.pid
        def monitor():
            start_time = time.monotonic()
            while not stop_event.wait(1):
                try:
                    row=metrics(monitored_pid)
                    row.update(elapsed_seconds=time.monotonic()-start_time,threads=len(list(Path(f"/proc/{monitored_pid}/task").iterdir())),fds=len(list(Path(f"/proc/{monitored_pid}/fd").iterdir())))
                    report["resources"].append(row)
                except (OSError,ProcessLookupError): break
        t = threading.Thread(target=monitor);t.start();threads.append(t)

        def phase(label,seconds,retain=True):
            print(f"STAGE {label} seconds={seconds}",flush=True)
            gate=threading.Barrier(25)
            def worker(index):
                connection=Client(port)
                try:
                    connection.login(codes)
                    gate.wait(timeout=15)
                    begin=time.monotonic(); end=begin+seconds; operation=0; rows=[]
                    while time.monotonic()<end:
                        due=begin+operation*.5
                        delay=due-time.monotonic()
                        if delay>0: time.sleep(delay)
                        if time.monotonic()>=end: break
                        write=operation%5==4; then=time.perf_counter_ns()
                        if write:
                            status=connection.call("POST",f"/board/projects/1/items/{index+1}",connection.fields(f"Worker {index:02}",versions[index]))[0]
                            if status==303: versions[index]+=1
                        else:
                            status=connection.call("GET","/board/projects/1/items")[0]
                        elapsed=(time.perf_counter_ns()-then)/1e6
                        rows.append(("write" if write else "read",elapsed,status))
                        if status!=(303 if write else 200): raise RuntimeError("unexpected response; no automatic mutation retry")
                        operation+=1
                    return rows
                finally: connection.close()
            before=metrics(server.pid); begin=time.monotonic()
            with concurrent.futures.ThreadPoolExecutor(max_workers=25) as pool:
                all_rows=[row for group in pool.map(worker,range(25)) for row in group]
            wall=time.monotonic()-begin; after=metrics(server.pid)
            result={"name":label,"configured_seconds":seconds,"wall_seconds_including_login":wall,"requests":len(all_rows),"unexpected_responses":0,"achieved_rps_including_login_wall":len(all_rows)/wall,"cpu_seconds":after["cpu_seconds"]-before["cpu_seconds"],"rss_before_kib":before["rss_kib"],"rss_after_kib":after["rss_kib"]}
            for kind in ["read","write"]:
                values=[r[1] for r in all_rows if r[0]==kind]
                result[kind]={"count":len(values),"p50_ms":percentile(values,.5),"p95_ms":percentile(values,.95),"p99_ms":percentile(values,.99),"max_ms":max(values)}
            result["combined_p99_ms"]=percentile([r[1] for r in all_rows],.99)
            result["thresholds_pass"]=result["read"]["p95_ms"]<250 and result["write"]["p95_ms"]<500 and result["combined_p99_ms"]<1000
            if retain:
                report["rounds"].append(result)
                (OUT/f"{label}-samples.json").write_text(json.dumps(all_rows)+"\n")
            print(json.dumps(result),flush=True)
            assert result["thresholds_pass"]
            assert not report["stream_errors"]
        phase("warmup",30,False)
        for n in range(1,4): phase(f"round-{n}",60)
        phase("soak",300)
        with sqlite3.connect(db) as check:
            report["final_data"]={"items":check.execute("SELECT COUNT(*) FROM work_items").fetchone()[0],"history":check.execute("SELECT COUNT(*) FROM history").fetchone()[0]}
            assert report["final_data"]["items"]==10000 and report["final_data"]["history"]<100000
            assert check.execute("PRAGMA integrity_check").fetchone()[0]=="ok"
        stop_event.set()
        for x in clients:x.close()
        clients.clear()
        report["graceful_shutdown_seconds"] = stop()
        for t in threads:t.join(timeout=15);assert not t.is_alive()
        assert not report["stream_errors"] and all(n>=2 for n in report["stream_connections"])
        report["checks"].append("ten SSE streams read throughout load and reconnect after the five-minute lifetime; graceful exit joins readers")
        assert identity == {str(p.relative_to(ROOT)):hashlib.sha256(p.read_bytes()).hexdigest() for p in source_paths}
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
        report["cleanup"]="owned process reaped, threads joined, listener closed, private synthetic databases/logs removed"
        (OUT/"report.json").write_text(json.dumps(report,indent=2)+"\n")
        print(json.dumps({k:v for k,v in report.items() if k not in ['source_sha256','resources']},indent=2),flush=True)

if __name__ == "__main__":
    run()
