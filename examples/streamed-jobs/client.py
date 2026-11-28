"""Run one job, drop a live SSE connection, then compare the recovered receipt."""

import hashlib
import json
import os
import sys
import time
import urllib.request


def events(response):
    for line in response:
        if line.startswith(b"data:"):
            yield json.loads(line[5:])


def main(base_url, path):
    http = urllib.request.build_opener(urllib.request.ProxyHandler({}))
    job = base_url + "/jobs/demo"
    with http.open(urllib.request.Request(job, method="POST"), timeout=5) as started:
        if started.status != 202:
            raise RuntimeError(f"unexpected start status {started.status}")

    url = job + "/events"
    with http.open(url, timeout=65) as first:
        for last in events(first):
            if last["phase"] == "upload" and 0 < last["sent"] < last["total"]:
                print("disconnect", last)
                break
        else:
            raise RuntimeError("upload ended before disconnect")

    time.sleep(0.3)
    with http.open(url, timeout=65) as second:
        updates = events(second)
        recovered = next(updates)
        print("reconnected", recovered)
        if recovered["phase"] != "upload" or recovered["sent"] <= last["sent"]:
            raise RuntimeError("expected offline upload progress")
        current = recovered
        for update in updates:
            if update["phase"] != current["phase"]:
                print("live", update)
            current = update

    if current["phase"] != "done":
        raise RuntimeError(f"unexpected final state {current}")
    with open(path, "rb") as source:
        digest = hashlib.file_digest(source, "sha256").hexdigest()
    if current["receipt"] != {"bytes": os.path.getsize(path), "sha256": digest}:
        raise RuntimeError("receiver receipt differs from source")
    print("source and receipt match", digest)


if __name__ == "__main__":
    main(*sys.argv[1:])
