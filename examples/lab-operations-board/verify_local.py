#!/usr/bin/env python3
"""One owned browser/HTTP/restart/diagnostic-load journey. No deployment or IME claim.

Only this script's temporary database, processes and profiles are touched. Local
synthetic login codes and Cookies remain in memory/private temporary files and
are excluded from output. A failed write is never automatically replayed.
"""
from __future__ import annotations
import concurrent.futures
import base64
import hashlib
from http.client import HTTPConnection
import json
import os
from pathlib import Path
import re
import selectors
import shutil
import signal
import socket
import sqlite3
import subprocess
import sys
import tempfile
import threading
import time
import urllib.parse

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "scripts"))
from smoke_browser import ChromeDevTools, http, port_number, wait_until, stop_owned_process, remove_owned_tree
OUT = ROOT / "target/m11-board"

class Client:
    def __init__(self, port):
        self.port = port
        self.http = HTTPConnection("127.0.0.1", port, timeout=8)
        self.cookie = None
        self.csrf = None
    def call(self, method, path, fields=None):
        headers = {}
        if self.cookie: headers["Cookie"] = self.cookie
        body = None
        if fields is not None:
            body = urllib.parse.urlencode(fields)
            headers.update({"Content-Type": "application/x-www-form-urlencoded", "Origin": f"http://127.0.0.1:{self.port}"})
        self.http.request(method, path, body, headers)
        response = self.http.getresponse()
        status, result_headers = response.status, {k.lower(): v for k, v in response.getheaders()}
        data = response.read(2 * 1024 * 1024 + 1)
        assert len(data) <= 2 * 1024 * 1024
        if "set-cookie" in result_headers: self.cookie = result_headers["set-cookie"].split(";", 1)[0]
        text = data.decode()
        token = re.search(r'name="csrf" value="([0-9a-f]+)"', text)
        if token: self.csrf = token.group(1)
        assert "unsafe-inline" not in result_headers.get("content-security-policy", "")
        return status, result_headers, text
    def login(self, codes, name="alice"):
        assert self.call("GET", "/board/login")[0] == 200
        assert self.call("POST", "/board/login", {"csrf":self.csrf,"username":name,"code":codes[name]})[0] == 303
        project = 2 if name == "bob" else 1
        assert self.call("GET", f"/board/projects/{project}/items")[0] == 200
    def fields(self, title, version, status="open"):
        return {"csrf":self.csrf,"expected_version":version,"title":title,"description":"Local qualification record.","status":status,"priority":"normal","assignee":""}
    def close(self): self.http.close()

def percentile(values, quantile):
    values = sorted(values)
    import math
    return values[max(0, math.ceil(len(values) * quantile) - 1)]

def metrics(pid):
    stat = Path(f"/proc/{pid}/stat").read_text().split()
    status = Path(f"/proc/{pid}/status").read_text()
    rss = re.search(r"^VmRSS:\s+(\d+)", status, re.M)
    return {"cpu_seconds": (int(stat[13]) + int(stat[14])) / os.sysconf("SC_CLK_TCK"), "rss_kib": int(rss.group(1)) if rss else None}

def run():
    OUT.mkdir(parents=True, exist_ok=True)
    temp = Path(tempfile.mkdtemp(prefix="local-", dir=OUT))
    os.chmod(temp, 0o700)
    binary = ROOT / "target/release/yetlet-lab-operations-board"
    report = {"status":"fail", "task_ids":["YT-073","YT-074"], "checks":[], "load":[], "not_verified":["real OS IME", "spoken assistive technology", "Firefox/Edge/Safari/mobile", "visual screenshot review", "10k items / 100k history operating envelope", "approved duration soak", "real IdP", "production rollout/HA", "external security review"], "cleanup":"pending"}
    server = chrome = driver = None
    clients, streams = [], []
    server_port, chrome_port = port_number(), port_number()
    assert server_port != chrome_port
    base = f"http://127.0.0.1:{server_port}"
    startup = 0
    logs = []
    def start():
        nonlocal server, startup
        startup += 1
        log = temp / f"server-{startup}.log"
        handle = log.open("x"); os.chmod(log, 0o600); logs.append(handle)
        server = subprocess.Popen([str(binary)], cwd=ROOT, env=dict(os.environ,YETLET_BOARD_LISTEN=f"127.0.0.1:{server_port}",YETLET_BOARD_DB=str(temp / "board.sqlite")), stdout=handle, stderr=handle, start_new_session=True)
        wait_until(lambda: "READY " in log.read_text(), "owned Board startup", seconds=12)
        assert http(base + "/readyz")[0] == 200
        codes = {m.group(1):m.group(2) for m in re.finditer(r"^LOCAL_LOGIN (alice|bob|viewer) ([0-9a-f]{64})$",log.read_text(),re.M)}
        assert len(codes) == 3
        return codes
    def stop():
        nonlocal server
        assert server is not None
        server.send_signal(signal.SIGINT)
        server.wait(timeout=6)
        assert server.returncode == 0
        with socket.socket() as sock: assert sock.connect_ex(("127.0.0.1",server_port)) != 0
        report["checks"].append("owned server graceful exit and listener closed")
        server = None
    def api():
        client = Client(server_port); clients.append(client); return client
    try:
        assert binary.is_file()
        report["binary_sha256"] = hashlib.sha256(binary.read_bytes()).hexdigest()
        report["head"] = subprocess.check_output(["git","rev-parse","HEAD"],cwd=ROOT,text=True).strip()
        source_files = sorted(p for p in (ROOT / "examples/lab-operations-board").rglob("*") if p.is_file()) + [ROOT / "Cargo.lock", ROOT / "Cargo.toml"]
        source_identity = {str(p.relative_to(ROOT)):hashlib.sha256(p.read_bytes()).hexdigest() for p in source_files}
        codes = start()
        operator = api(); operator.login(codes)
        viewer = api(); viewer.login(codes,"viewer")
        bob = api(); bob.login(codes,"bob")
        assert viewer.call("POST","/board/projects/1/items",viewer.fields("Denied",0))[0] == 403
        assert bob.call("GET","/board/projects/1/items")[0] == 403
        report["checks"].append("real HTTP viewer mutation and cross-project denial")

        executable = shutil.which("google-chrome"); assert executable
        chrome = subprocess.Popen([executable,"--headless=new","--disable-gpu","--no-first-run","--no-default-browser-check","--remote-allow-origins=http://127.0.0.1",f"--remote-debugging-port={chrome_port}",f"--user-data-dir={temp / 'profile'}","about:blank"],stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL,start_new_session=True)
        targets = wait_until(lambda: json.loads(http(f"http://127.0.0.1:{chrome_port}/json")[2]),"owned Chrome",seconds=10)
        pages = [page for page in targets if page.get("type") == "page"]; assert len(pages) == 1
        driver = ChromeDevTools(pages[0]["webSocketDebuggerUrl"])
        report["browser"] = driver.call("Browser.getVersion")["product"]
        for method in ["Page.enable","Runtime.enable","Network.enable","Log.enable"]: driver.call(method)
        def js(expression): return driver.evaluate(expression)
        def assert_js(expression, label):
            assert js(expression) is True, label
            report["checks"].append(label)
        def navigate(path):
            driver.call("Page.navigate", {"url":base + path})
            wait_until(lambda: js("document.readyState === 'complete' && location.pathname === " + json.dumps(path.split('?')[0])), "page readiness")
        def click(selector):
            assert js(f"document.querySelectorAll({json.dumps(selector)}).length") == 1
            js(f"document.querySelector({json.dumps(selector)}).click()")
            js("document.readyState")
        def fill(selector, value):
            assert js(f"document.querySelectorAll({json.dumps(selector)}).length") == 1
            js(f"document.querySelector({json.dumps(selector)}).value = {json.dumps(value)}")
            js("document.readyState")
        shots = ROOT / "docs/screenshots/m11-board"
        shots.mkdir(parents=True, exist_ok=True)
        report["screenshots"] = []
        def capture(name):
            assert js("document.readyState === 'complete'")
            assert js("[...document.querySelectorAll('input[type=password]')].every(e => !e.value)")
            data = driver.call("Page.captureScreenshot", {"format":"png", "captureBeyondViewport":False})["data"]
            path = shots / (name + ".png")
            path.write_bytes(base64.b64decode(data))
            report["screenshots"].append(str(path.relative_to(ROOT)))
        driver.call("Emulation.setDeviceMetricsOverride",{"width":1280,"height":960,"deviceScaleFactor":1,"mobile":False})
        navigate("/board/login")
        assert_js("document.querySelector('.board-form').getBoundingClientRect().width > 400 && document.querySelectorAll('script').length === 0", "script-free full-width login layout")
        fill("#username","alice"); fill("#code",codes["alice"])
        click(".board-form button[type=submit]")
        wait_until(lambda:js("location.pathname === '/board/projects/1/items' && document.readyState === 'complete'"),"native login redirect")
        assert_js("document.body.textContent.includes('No work items match')", "actual empty authorized list")
        click(".board-nav a[href$='/new']")
        wait_until(lambda:js("document.getElementById('title') !== null && document.readyState === 'complete'"),"new item form")
        assert_js("!document.querySelector('[aria-invalid=true]') && document.querySelector('label[for=description]').control === document.getElementById('description')", "new form has no premature errors and has associated description label")
        fill("#title","Browser <mark>literal</mark>"); fill("#description","Leading plain text\n<unsafe> remains text.")
        click(".board-form button[type=submit]")
        wait_until(lambda:js("location.pathname === '/board/projects/1/items/1' && document.readyState === 'complete'"),"native create PRG")
        assert_js("document.body.textContent.includes('Browser <mark>literal</mark>') && !document.querySelector('mark') && document.body.textContent.includes('History')", "stored create/read/history use literal text")
        click(".board-main a[href$='/edit']")
        wait_until(lambda:js("document.querySelector('[name=expected_version]')?.value === '1'"),"versioned edit")
        fill("#status","closed"); fill("#priority","high")
        click(".board-form button[type=submit]")
        wait_until(lambda:js("location.pathname === '/board/projects/1/items/1' && document.body.textContent.includes('version 2')"),"native edit PRG")
        click(".board-main a[href$='/edit']")
        wait_until(lambda:js("document.querySelector('[name=expected_version]')?.value === '2'"),"stale draft preflight")
        assert operator.call("POST","/board/projects/1/items/1",operator.fields("Other client",2,"open"))[0] == 303
        fill("#title","Retained browser draft")
        click(".board-form button[type=submit]")
        wait_until(lambda:js("document.body.textContent.includes('Another change was saved first')"),"conflict page")
        assert_js("document.getElementById('title').value === 'Retained browser draft' && document.querySelector('[name=expected_version]').value === '2'", "real conflict preserves draft without silently overwriting")
        capture("03-conflict-draft")
        assert driver.failed_responses == [f"409 {base}/board/projects/1/items/1"]
        # Chrome logs the intentional HTTP 409 as a network error as well.
        # Admit only that exact diagnostic at this already-asserted negative step.
        expected_console = "Failed to load resource: the server responded with a status of 409 (Conflict)"
        report["expected_conflict_console_count"] = driver.exceptions.count(expected_console)
        driver.exceptions[:] = [entry for entry in driver.exceptions if entry != expected_console]
        assert not driver.exceptions, "unexpected browser error at conflict boundary"
        driver.failed_responses.clear()
        navigate("/board/projects/1/items?live=true")
        wait_until(lambda:js("document.querySelector('[data-board-stream]')?.dataset.streamState === 'current'"),"initial authoritative SSE snapshot")
        assert operator.call("POST","/board/projects/1/items/1",operator.fields("Live update",3,"active"))[0] == 303
        wait_until(lambda:js("document.querySelector('[data-board-stream]')?.dataset.currentRevision === '4'"),"real persisted stream update")
        assert_js("document.querySelector('[data-stream-message]').textContent.includes('Refresh authoritative records')", "status invalidation keeps an explicit authoritative refresh path")
        click("[data-stream-stop]")
        assert_js("document.querySelector('[data-board-stream]').dataset.streamState === 'paused'", "explicit live-stream pause")
        click("[data-stream-retry]")
        wait_until(lambda:js("document.querySelector('[data-board-stream]')?.dataset.streamState === 'current'"),"full snapshot after explicit reconnect")
        click("[data-stream-stop]")
        driver.call("Emulation.setDeviceMetricsOverride",{"width":390,"height":844,"deviceScaleFactor":1,"mobile":False})
        assert_js("document.documentElement.scrollWidth <= 390", "narrow Board has no page-level horizontal overflow")
        capture("04-narrow-live-status")
        # Fresh screenshots requested by the owner use actual persisted synthetic records.
        titles = ["Review backup restoration evidence", "Verify lab TLS certificate renewal", "Update onboarding documentation", "Inspect storage capacity trend", "Reconcile test environment access", "Record weekly maintenance outcome"]
        for index, title in enumerate(titles):
            values = operator.fields(title, 0, ["open", "active", "blocked"][index % 3])
            values["priority"] = ["high", "normal", "low"][index % 3]
            values["assignee"] = "alice"
            values["description"] = "Synthetic demonstration record. Track the result and retain the review history."
            assert operator.call("POST", "/board/projects/1/items", values)[0] == 303
        driver.call("Emulation.setDeviceMetricsOverride",{"width":1440,"height":1000,"deviceScaleFactor":1,"mobile":False})
        navigate("/board/projects/1/items")
        capture("01-work-items")
        navigate("/board/projects/1/items/2")
        capture("02-item-history")
        navigate("/board/projects/1/items/2/edit")
        capture("05-edit-work-item")
        assert not driver.exceptions and not driver.failed_responses
        assert all(url.startswith(base) or url == "about:blank" for url in driver.requests)
        report["checks"].append("actual browser CSP/JS/network clean; no external requests")
        driver.close(); driver = None
        stop_owned_process(chrome,process_group=True); chrome = None

        old_cookie = operator.cookie
        for client in clients: client.close()
        clients.clear()
        stop()
        # Seed only the owned, stopped qualification database. Not application data.
        connection=sqlite3.connect(temp / "board.sqlite")
        with connection:
            for item_id in range(1,1001):
                connection.execute("INSERT OR IGNORE INTO work_items(id,project,title,description,status,priority,assignee,version) VALUES(?,1,?,'Synthetic load record','open','normal','',1)",(item_id,f"Load record {item_id:04}"))
                previous=connection.execute("SELECT COALESCE(MAX(version),0) FROM history WHERE item_id=?",(item_id,)).fetchone()[0]
                for version in range(previous+1,11):
                    connection.execute("INSERT INTO history(item_id,project,version,actor,action,title,status) VALUES(?,1,?,'alice',?,?,'open')",(item_id,version,"create" if version==1 else "update",f"Load record {item_id:04}"))
                connection.execute("UPDATE work_items SET version=10,status='open' WHERE id=?",(item_id,))
        report["seed"]={"items":connection.execute("SELECT COUNT(*) FROM work_items").fetchone()[0],"history":connection.execute("SELECT COUNT(*) FROM history").fetchone()[0]}
        connection.close()
        codes=start()
        retired=api(); retired.cookie=old_cookie
        assert retired.call("GET","/board/projects/1/items/1")[0] == 401
        fresh=api(); fresh.login(codes)
        assert "Live update" in fresh.call("GET","/board/projects/1/items/1")[2]
        report["checks"].append("actual process restart preserves records/history and rejects old Memory login")
        # Ten independently owned native SSE connections stay open during load.
        for _ in range(10):
            conn=HTTPConnection("127.0.0.1",server_port,timeout=8)
            conn.request("GET","/board/projects/1/events",headers={"Cookie":fresh.cookie})
            response=conn.getresponse(); assert response.status==200; assert response.read(1)
            streams.append((conn,response))
        versions=[10]*25
        for run_number in range(3):
            before=metrics(server.pid)
            barrier=threading.Barrier(25)
            def worker(index):
                client=Client(server_port)
                try:
                    client.login(codes)
                    for _ in range(4): assert client.call("GET","/board/projects/1/items")[0]==200
                    barrier.wait(timeout=12)
                    observations=[]
                    for operation in range(50):
                        write=operation%5==4
                        start=time.perf_counter_ns()
                        if write:
                            status=client.call("POST",f"/board/projects/1/items/{index+1}",client.fields(f"Load worker {index:02}",versions[index]))[0]
                            if status==303: versions[index]+=1
                        else:
                            status=client.call("GET","/board/projects/1/items")[0]
                        duration=time.perf_counter_ns()-start
                        observations.append(("write" if write else "read",duration,status))
                        if status!=(303 if write else 200): raise RuntimeError("load request failed; no automatic mutation retry")
                    return observations
                finally: client.close()
            begin=time.perf_counter()
            with concurrent.futures.ThreadPoolExecutor(max_workers=25) as pool:
                results=list(pool.map(worker,range(25)))
            wall=time.perf_counter()-begin
            after=metrics(server.pid)
            observations=[row for rows in results for row in rows]
            result={"run":run_number+1,"clients":25,"sse_clients":10,"requests":len(observations),"unexpected_responses":0,"lifecycle_seconds_including_login_and_warmup":wall,"cpu_seconds":after["cpu_seconds"]-before["cpu_seconds"],"rss_before_kib":before["rss_kib"],"rss_after_kib":after["rss_kib"]}
            for kind in ["read","write"]:
                values=[row[1]/1e6 for row in observations if row[0]==kind]
                result[kind]={"samples":len(values),"p50_ms":percentile(values,.50),"p95_ms":percentile(values,.95),"p99_ms":percentile(values,.99),"max_ms":max(values)}
            report["load"].append(result)
            (OUT/f"load-{run_number+1}.json").write_text(json.dumps(observations)+"\n")
        report["checks"].append("three diagnostic closed-loop 25-client 80/20 read/write rounds with ten SSE connections")
        for client in clients: client.close()
        clients.clear()
        stop()
        for conn,response in streams:
            assert len(response.read(65536))<65536
            conn.close()
        streams.clear()
        report["checks"].append("graceful shutdown reaches EOF on all ten owned SSE clients")
        assert source_identity == {str(p.relative_to(ROOT)):hashlib.sha256(p.read_bytes()).hexdigest() for p in source_files}
        report["source_sha256"]=source_identity
        report["status"]="pass"
    except Exception as error:
        report["failure_type"]=type(error).__name__
        # Future failed journeys retain bounded diagnostics, never form values/codes/Cookies.
        # This observation must not retry an input or navigate the page.
        if driver is not None:
            try:
                report["browser_failure"] = driver.evaluate("({pathname: location.pathname, ready: document.readyState, invalidControls: ['username','code','title','description'].filter(id => {const e=document.getElementById(id); return e && e.validity && !e.validity.valid;})})")
                report["failed_http_statuses"] = [entry.split(' ',1)[0] for entry in driver.failed_responses[-8:]]
                report["browser_exception_count"] = len(driver.exceptions)
            except Exception as observation_error:
                report["browser_observation_error"] = type(observation_error).__name__
        raise
    finally:
        if driver is not None: driver.close()
        for client in clients: client.close()
        for conn,_ in streams: conn.close()
        for process in (chrome,server): stop_owned_process(process,process_group=True)
        for handle in logs: handle.close()
        remove_owned_tree(temp)
        for port in [server_port,chrome_port]:
            with socket.socket() as sock: assert sock.connect_ex(("127.0.0.1",port))!=0
        report["cleanup"]="owned application/browser processes reaped; listeners closed; temporary profile, logs and synthetic database removed"
        (OUT/"qualification.json").write_text(json.dumps(report,indent=2)+"\n")
        print(json.dumps({key:value for key,value in report.items() if key!="source_sha256"},indent=2))

if __name__=="__main__": run()
