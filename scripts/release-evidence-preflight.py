#!/usr/bin/env python3
"""YT-096 versioned release-evidence API gate and two-file ciphertext download.

GitHub's protected Environment job gate supplies approval, not workflow_dispatch.
The owner must review the Environment settings before installing its secret.
API contracts: https://docs.github.com/en/rest/actions/workflow-runs
https://docs.github.com/en/rest/actions/artifacts
https://docs.github.com/en/rest/deployments/environments
https://docs.github.com/en/rest/deployments/branch-policies
"""
import hashlib
import io
import json
import os
from pathlib import Path
import re
import stat
import sys
import urllib.error
import urllib.parse
import urllib.request
import zipfile

# Support the existing importlib consumer as well as direct script execution.
sys.path.insert(0, str(Path(__file__).resolve().parent))
import release_record as records
import release_policy as policy

API = "https://api.github.com/repos/bohewu/yetlet"
ENVIRONMENT = "release-evidence-verify"
CIPHER = "private-evidence.tar.age"
RECEIPT = "transport-receipt.json"
LIMIT = 259 * 1024 * 1024


def require(condition):
    if not condition:
        raise ValueError("release evidence gate rejected")


def inputs():
    r = records.environment('custody')
    b = r['build']
    return {'source': r['source'], 'tag': 'v' + r['version'], 'run': b['run'],
            'attempt': b['attempt'], 'artifact': b['artifact'], 'digest': b['sha256']}



def check_metadata(run, artifact, environment, policies, expected):
    require(run["id"] == int(expected["run"]))
    require(run["run_attempt"] == int(expected["attempt"]))
    require(run["head_sha"] == expected["source"])
    require(run["head_branch"] == expected["tag"])
    require(run["event"] == "workflow_dispatch")
    require(run["path"] in (".github/workflows/release.yml",
                            ".github/workflows/release.yml@refs/tags/" + expected["tag"],
                            ".github/workflows/release.yml@" + expected["tag"]))
    require(run["repository"]["full_name"] == "bohewu/yetlet")
    require(run["head_repository"]["full_name"] == "bohewu/yetlet")
    require(run["status"] == "completed" and run["conclusion"] in ("success", "failure"))
    require(artifact["id"] == int(expected["artifact"]))
    require(artifact["name"] == "encrypted-private-{source}-{run}-{attempt}".format(**expected))
    require(artifact["expired"] is False)
    require(0 < artifact["size_in_bytes"] <= LIMIT)
    require(artifact["digest"] == "sha256:" + expected["digest"])
    require(artifact["workflow_run"]["id"] == int(expected["run"]))
    require(artifact["workflow_run"]["head_sha"] == expected["source"])
    require(artifact["workflow_run"]["repository_id"] == run["repository"]["id"])
    require(artifact["workflow_run"]["head_repository_id"] == run["repository"]["id"])
    require(environment["name"] == ENVIRONMENT and type(environment["id"]) is int
            and environment["can_admins_bypass"] is False)
    policy.reviewers(environment)
    require(environment["deployment_branch_policy"] ==
            {"protected_branches": False, "custom_branch_policies": True})
    require(policies["total_count"] == 1 and len(policies["branch_policies"]) == 1)
    policy = policies["branch_policies"][0]
    require(policy["name"] == expected["tag"] and policy["type"] == "tag")


class NoRedirect(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, req, fp, code, msg, headers, newurl):
        return None


def request(url, authenticated, limit):
    require(url.startswith("https://") and not urllib.parse.urlsplit(url).username)
    headers = {"Accept": "application/vnd.github+json", "X-GitHub-Api-Version": "2026-03-10"}
    if authenticated:
        require(url.startswith(API + "/"))
        headers["Authorization"] = "Bearer " + os.environ["GH_TOKEN"]
    opener = urllib.request.build_opener(NoRedirect())
    try:
        response = opener.open(urllib.request.Request(url, headers=headers), timeout=120)
    except urllib.error.HTTPError as error:
        if error.code == 302:
            return None, error.headers["Location"]
        raise
    with response:
        body = response.read(limit + 1)
        require(len(body) <= limit)
        return body, None


def api(path):
    body, redirect = request(API + path, True, 1024 * 1024)
    require(redirect is None)
    return json.loads(body)


def gate(expected):
    run = api("/actions/runs/{run}/attempts/{attempt}".format(**expected))
    artifact = api("/actions/artifacts/{artifact}".format(**expected))
    environment = api("/environments/" + ENVIRONMENT)
    policies = api("/environments/" + ENVIRONMENT + "/deployment-branch-policies?per_page=100")
    check_metadata(run, artifact, environment, policies, expected)


def unpack(body, expected, output):
    require(hashlib.sha256(body).hexdigest() == expected["digest"])
    with zipfile.ZipFile(io.BytesIO(body)) as archive:
        entries = archive.infolist()
        require(len(entries) == 2 and {e.filename for e in entries} == {CIPHER, RECEIPT})
        total = 0
        for entry in entries:
            mode = entry.external_attr >> 16
            require(not entry.is_dir() and not (entry.flag_bits & 1))
            require(stat.S_IFMT(mode) in (0, stat.S_IFREG))
            require(entry.compress_type in (zipfile.ZIP_STORED, zipfile.ZIP_DEFLATED))
            maximum = 64 * 1024 if entry.filename == RECEIPT else 258 * 1024 * 1024
            require(0 < entry.file_size <= maximum)
            total += entry.file_size
        require(total <= LIMIT)
        os.mkdir(output, 0o700)
        for entry in entries:
            with archive.open(entry) as source:
                data = source.read(entry.file_size + 1)
            require(len(data) == entry.file_size)
            fd = os.open(os.path.join(output, entry.filename),
                         os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW, 0o600)
            with os.fdopen(fd, "wb") as target:
                target.write(data)


def main():
    require(sys.argv[1:] in (["preflight"], ["download"]))
    expected = inputs()
    gate(expected)  # Repeated in the protected job before any private-key step.
    if sys.argv[1] == "download":
        body, location = request(API + "/actions/artifacts/{artifact}/zip".format(**expected), True, LIMIT)
        for _ in range(3):
            if location is None:
                break
            body, location = request(location, False, LIMIT)  # Never forward the token to storage.
        require(location is None and body is not None)
        unpack(body, expected, os.environ["CUSTODY_TRANSPORT"])
    print("release evidence metadata gate: PASS")


if __name__ == "__main__":
    try:
        main()
    except Exception:
        # API URLs, signed download URLs, tokens and response bodies are not diagnostics.
        print("release evidence metadata/download rejected; no private key accessed", file=sys.stderr)
        sys.exit(1)
