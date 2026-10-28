#!/usr/bin/env python3
"""YT-096: data-only preparation/signing of one retained alpha release.

No product build, tag mutation, publication, ambient personal login or retries.
Run only reviewed code at S. Private material must remain outside the checkout.
"""
import base64
import hashlib
import importlib.util
import io
import json
import os
from pathlib import Path
import re
import shutil
import stat
import subprocess
import sys
import tarfile
import urllib.error

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import release_checksums as checksums

spec = importlib.util.spec_from_file_location("custody_api", HERE / "release-evidence-preflight.py")
api = importlib.util.module_from_spec(spec)
spec.loader.exec_module(api)
C = "bf3363801132f4481263ceb411d9bba3349df86d"
TAG = "v0.9.0-alpha.1"
BUILD = "40669449040"
VERIFIER = "40688772221"
BUILD_ARTIFACT = "12807757693"
BUILD_ZIP = "a09c7e3e32d254a702c171333bfa01a1d08c2ed96f36eb5463ee9745a66e3dca"
PUBLIC_SHA = "4d4de1a8ce9cc077c0aa62a1aafb3ab9799eed312deddb66d9e6f8fc6fef2d68"
RECIPIENT_SHA = "b861a3c7e2abf3caf4c7071c045f6114704db94e8cc39ce010e7fa10711ff506"
SAN = "https://github.com/bohewu/yetlet/.github/workflows/release-sign.yml@refs/heads/master"
ISSUER = "https://token.actions.githubusercontent.com"
ROOT_SHA = "6494e21ea73fa7ee769f85f57d5a3e6a08725eae1e38c755fc3517c9e6bc0b66"
CONFIG_SHA = "0f5f38554e29e770d4d5d6f0e1b51fcbf84f61dc6934530a09b7a901eaad5bee"
COSIGN_SHA = "424e2c97d0b45fd7a6b608a87f090f35633f4c23ece5f092d48bba11aaf4bbfa"
AGE_SHA = "eb7dd1b518f0a307c99cd97782623c5321da049154b04acd2d98d21aa7bc9b2c"
RELEASE_ID = 426670086
MANIFEST = checksums.MANIFEST
ROLES = {"qualification", "security", "inventory", "support", "review", "failures"}
PACKAGES = {"yetlet-" + s for s in (
    "actions", "auth", "config", "core", "dom", "dom-keyed", "html", "rate-limit",
    "reactive", "request-id", "secrets", "server-axum", "session", "session-redis", "signing", "wgpu")}


def require(ok):
    if not ok:
        raise ValueError("signer input rejected")


class GateRejected(Exception):
    """Only fixed stages, error categories and numeric HTTP status may reach logs."""
    def __init__(self, stage, error):
        super().__init__("signer metadata gate rejected")
        self.stage = stage
        self.status = None
        if isinstance(error, urllib.error.HTTPError):
            self.kind = "http"
            if type(error.code) is int and 400 <= error.code <= 599:
                self.status = error.code
            error.close()
        elif isinstance(error, (urllib.error.URLError, TimeoutError, ConnectionError)):
            self.kind = "network"
        elif isinstance(error, (ValueError, KeyError, TypeError, IndexError)):
            self.kind = "metadata"
        else:
            self.kind = "internal"


def sha(raw):
    return hashlib.sha256(raw).hexdigest()


def load(raw):
    require(len(raw) <= checksums.MAX_MANIFEST_BYTES)
    return json.loads(raw, object_pairs_hook=checksums.unique_object)


def read(path, limit=checksums.MAX_FILE_BYTES):
    path = Path(path)
    with checksums.directory_fd(str(path.parent)) as fd:
        handle, before = checksums.read_file(fd, path.name, limit)
        with os.fdopen(handle, "rb") as f:
            raw = f.read(limit + 1)
            require(len(raw) == before.st_size <= limit)
            require(checksums.fingerprint(before) == checksums.fingerprint(os.fstat(f.fileno())))
            return raw


def write(path, raw):
    with open(path, "xb") as f:
        os.fchmod(f.fileno(), 0o600)
        f.write(raw)


def json_write(path, value):
    write(path, (json.dumps(value, sort_keys=True, indent=2) + "\n").encode())


def number(value):
    return isinstance(value, str) and re.fullmatch(r"[1-9][0-9]{0,19}", value)


def digest(value):
    return isinstance(value, str) and re.fullmatch(r"[0-9a-f]{64}", value)


def request():
    r = load(os.environ["SIGN_REQUEST"].encode())
    mode = r.get("mode")
    common = {"mode", "signer_commit"}
    fields = ({"qualification_asset", "qualification_sha256", "qualification_manifest_sha256"}
              if mode == "prepare" else
              {"run", "attempt", "artifact", "digest", "receipt_sha256", "provenance_sha256",
               "manifest_sha256", "d6"})
    require(mode in {"prepare", "sign"} and set(r) == common | fields)
    require(re.fullmatch(r"[0-9a-f]{40}", r["signer_commit"]) and r["signer_commit"] != C)
    for key in fields - {"d6"}:
        require(number(r[key]) if key in {"qualification_asset", "run", "attempt", "artifact"}
                else digest(r[key]))
    if mode == "sign":
        require(r["d6"] == "approve-exact-signing-inputs" and r["provenance_sha256"] == PUBLIC_SHA)
    return r


def identity(r):
    for key, value in {
        "GITHUB_EVENT_NAME": "workflow_dispatch", "GITHUB_REPOSITORY": "bohewu/yetlet",
        "GITHUB_REPOSITORY_ID": "1409587171", "GITHUB_REPOSITORY_OWNER_ID": "35250002",
        "GITHUB_REF": "refs/heads/master", "GITHUB_SHA": r["signer_commit"],
        "SIGN_WORKFLOW_SHA": r["signer_commit"],
        "SIGN_WORKFLOW_REF": "bohewu/yetlet/.github/workflows/release-sign.yml@refs/heads/master",
    }.items():
        require(os.environ.get(key) == value)
    require(subprocess.check_output(["git", "rev-parse", "HEAD"], text=True).strip() == r["signer_commit"])


def environment_gate(environment, policies):
    require(environment["name"] == "release-sign" and environment["can_admins_bypass"] is False)
    rules = [v for v in environment["protection_rules"] if v["type"] == "required_reviewers"]
    require(len(rules) == 1 and rules[0]["prevent_self_review"] is False)
    require(len(rules[0]["reviewers"]) == 1)
    reviewer = rules[0]["reviewers"][0]
    require(reviewer["type"] == "User" and reviewer["reviewer"]["login"] == "bohewu"
            and reviewer["reviewer"]["id"] == 35250002)
    require(environment["deployment_branch_policy"] ==
            {"protected_branches": False, "custom_branch_policies": True})
    require(policies["total_count"] == 1 and len(policies["branch_policies"]) == 1)
    require(policies["branch_policies"][0]["name"] == "master"
            and policies["branch_policies"][0]["type"] == "branch")


def run_gate(run, run_id, source, branch, path, attempt="1"):
    require(run["id"] == int(run_id) and run["run_attempt"] == int(attempt))
    require(run["head_sha"] == source and run["head_branch"] == branch)
    require(run["repository"]["id"] == 1409587171 and run["head_repository"]["id"] == 1409587171)
    require(run["event"] == "workflow_dispatch" and run["path"] == path)
    require(run["status"] == "completed" and run["conclusion"] == "success")


def artifact_gate(a, run, source, artifact_id, expected_sha, name):
    require(a["id"] == int(artifact_id) and a["name"] == name and a["expired"] is False)
    require(a["digest"] == "sha256:" + expected_sha and 0 < a["size_in_bytes"] <= api.LIMIT)
    require(a["workflow_run"]["id"] == int(run) and a["workflow_run"]["head_sha"] == source)
    require(a["workflow_run"]["repository_id"] == 1409587171
            and a["workflow_run"]["head_repository_id"] == 1409587171)


def gate(r):
    stage = "identity"
    try:
        identity(r)
        stage = "branch"
        branch = api.api("/branches/master")
        require(branch["protected"] is True and branch["commit"]["sha"] == r["signer_commit"])
        stage = "environment"
        environment = api.api("/environments/release-sign")
        stage = "branch-policy"
        policies = api.api("/environments/release-sign/deployment-branch-policies?per_page=100")
        stage = "environment-policy"
        environment_gate(environment, policies)
        if r["mode"] == "prepare":
            stage = "build-run"
            run_gate(api.api("/actions/runs/" + BUILD + "/attempts/1"), BUILD, C, TAG,
                     ".github/workflows/release.yml")
            stage = "build-artifact"
            artifact_gate(api.api("/actions/artifacts/" + BUILD_ARTIFACT), BUILD, C,
                          BUILD_ARTIFACT, BUILD_ZIP, "encrypted-private-" + C + "-" + BUILD + "-1")
            stage = "verifier-run"
            run_gate(api.api("/actions/runs/" + VERIFIER + "/attempts/1"), VERIFIER, C, TAG,
                     ".github/workflows/evidence-verify.yml")
            stage = "verifier-approval"
            approvals = api.api("/actions/runs/" + VERIFIER + "/approvals")
            require(any(a["state"] == "approved" and a["user"]["id"] == 35250002
                        and any(e["id"] == 25525950662 for e in a["environments"]) for a in approvals))
            stage = "qualification-release"
            release = api.api("/releases/" + str(RELEASE_ID))
            require(release["draft"] is True and release["tag_name"] == TAG
                    and release["target_commitish"] == C)
            stage = "qualification-assets"
            assets = api.api("/releases/" + str(RELEASE_ID) + "/assets?per_page=100")
            require(any(a["id"] == int(r["qualification_asset"])
                        and a["digest"] == "sha256:" + r["qualification_sha256"]
                        and a["name"] == "qualification-inputs.zip"
                        and 0 < a["size"] <= api.LIMIT for a in assets))
        else:
            stage = "prepare-run"
            run_gate(api.api("/actions/runs/" + r["run"] + "/attempts/" + r["attempt"]),
                     r["run"], r["signer_commit"], "master", ".github/workflows/release-sign.yml", r["attempt"])
            stage = "prepare-artifact"
            artifact_gate(api.api("/actions/artifacts/" + r["artifact"]), r["run"], r["signer_commit"],
                          r["artifact"], r["digest"], "signing-inputs-" + r["run"] + "-" + r["attempt"])
    except Exception as error:
        raise GateRejected(stage, error) from None


def download(path, binary=False):
    # The binary release-asset endpoint needs octet-stream; never forward GH_TOKEN to storage.
    if binary:
        import urllib.request
        import urllib.error
        headers = {"Accept": "application/octet-stream", "Authorization": "Bearer " + os.environ["GH_TOKEN"],
                   "X-GitHub-Api-Version": "2026-03-10"}
        try:
            response = urllib.request.build_opener(api.NoRedirect()).open(
                urllib.request.Request(api.API + path, headers=headers), timeout=120)
        except urllib.error.HTTPError as e:
            require(e.code == 302)
            raw, location = None, e.headers["Location"]
        else:
            with response:
                raw, location = response.read(api.LIMIT + 1), None
    else:
        raw, location = api.request(api.API + path, True, api.LIMIT)
    for _ in range(3):
        if location is None:
            break
        raw, location = api.request(location, False, api.LIMIT)
    require(location is None and raw is not None and len(raw) <= api.LIMIT)
    return raw


def unpack_zip(raw, expected, out, names):
    import zipfile
    require(sha(raw) == expected)
    with zipfile.ZipFile(io.BytesIO(raw)) as z:
        entries = z.infolist()
        require(len(entries) == len(names) and {e.filename for e in entries} == set(names))
        require(sum(e.file_size for e in entries) <= api.LIMIT)
        out.mkdir(mode=0o700)
        for e in entries:
            require(not e.is_dir() and not (e.flag_bits & 1)
                    and stat.S_IFMT(e.external_attr >> 16) in (0, stat.S_IFREG)
                    and e.compress_type in (zipfile.ZIP_STORED, zipfile.ZIP_DEFLATED))
            limit = names[e.filename]
            require(0 < e.file_size <= limit)
            write(out / e.filename, z.read(e))


def command(argv):
    # Suppress private paths, evidence, tokens and native-tool diagnostics.
    subprocess.run([str(a) for a in argv], check=True, stdout=subprocess.PIPE,
                   stderr=subprocess.PIPE, timeout=240)


def archive(source, target):
    with tarfile.open(target, "x", format=tarfile.GNU_FORMAT) as t:
        for f in sorted(source.iterdir()):
            require(f.is_file() and not f.is_symlink())
            t.add(f, arcname=f.name, recursive=False)


def restore(archive_path, out):
    out.mkdir(mode=0o700)
    total = 0
    # A bounded regular file and streaming reader also bound malformed hidden GNU headers.
    require(archive_path.stat().st_size <= api.LIMIT)
    with tarfile.open(archive_path, "r|") as t:
        names = set()
        for e in t:
            require(e.name == MANIFEST or checksums.NAME.fullmatch(e.name))
            require(e.isfile() and not e.pax_headers and e.name not in names)
            names.add(e.name)
            require(len(names) <= checksums.MAX_FILES + 1 and 0 <= e.size <= checksums.MAX_FILE_BYTES)
            total += e.size
            require(total <= checksums.MAX_TOTAL_BYTES + checksums.MAX_MANIFEST_BYTES)
            write(out / e.name, t.extractfile(e).read(e.size + 1))
    checksums.run("verify", str(out))


def qualification(directory):
    checksums.run("verify", str(directory))
    closure = load(read(directory / "QUALIFICATION-CLOSURE.json"))
    require(set(closure) == {"schema", "source", "entries"} and closure["schema"] == 1
            and closure["source"] == C and 1 <= len(closure["entries"]) < checksums.MAX_FILES)
    roles, names = set(), set()
    for e in closure["entries"]:
        require(set(e) == {"name", "sha256", "size", "roles", "original_source", "scope"})
        checksums.safe_name(e["name"])
        require(e["name"] not in names and e["name"] != "QUALIFICATION-CLOSURE.json")
        names.add(e["name"])
        require(set(e["roles"]) <= ROLES and e["roles"])
        roles.update(e["roles"])
        require(isinstance(e["original_source"], str) and 1 <= len(e["original_source"]) <= 256)
        require(isinstance(e["scope"], str) and 1 <= len(e["scope"]) <= 4096)
        raw = read(directory / e["name"])
        require(sha(raw) == e["sha256"] and len(raw) == e["size"])
    require(roles == ROLES and {p.name for p in directory.iterdir()} ==
            names | {MANIFEST, "QUALIFICATION-CLOSURE.json"})
    return closure


def payloads(directory):
    checksums.run("verify", str(directory))
    raw = read(directory / "PROVENANCE-UNSIGNED.json")
    require(len(raw) == 9158 and sha(raw) == PUBLIC_SHA)
    p = load(raw)
    require(p["_type"] == "https://in-toto.io/Statement/v1"
            and p["predicateType"] == "https://slsa.dev/provenance/v1")
    names = {n + "-0.9.0-alpha.1.crate" for n in PACKAGES}
    require(len(p["subject"]) == 16 and {s["name"] for s in p["subject"]} == names)
    for s in p["subject"]:
        require(s["digest"] == {"sha256": sha(read(directory / s["name"]))})
    definition = p["predicate"]["buildDefinition"]
    require(definition["externalParameters"] == {"cohort": "sixteen", "workflow": {
        "repository": "https://github.com/bohewu/yetlet", "path": ".github/workflows/release.yml",
        "ref": "refs/tags/" + TAG}})
    require({"uri": "git+https://github.com/bohewu/yetlet", "digest": {"gitCommit": C}}
            in definition["resolvedDependencies"])
    require(p["predicate"]["runDetails"]["metadata"]["invocationId"] ==
            "https://github.com/bohewu/yetlet/actions/runs/" + BUILD + "/attempts/1")
    return PUBLIC_SHA, sha(read(directory / MANIFEST, checksums.MAX_MANIFEST_BYTES))


def tools(directory):
    for name, expected in [("cosign", COSIGN_SHA), ("trusted_root.json", ROOT_SHA),
                           ("signing_config.json", CONFIG_SHA)]:
        require(sha(read(directory / name, 160 * 1024 * 1024)) == expected)
    proof = load(read(directory / "bootstrap-receipt.json"))
    require(proof == {"schema": 1, "tuf_authenticated": True, "artifact_key_verified": True,
                      "upstream_keyless_verified": True, "cosign_sha256": COSIGN_SHA,
                      "root_sha256": ROOT_SHA, "config_sha256": CONFIG_SHA})


def prepare(r, work, output):
    restored = work / "original"
    command([os.environ["SIGN_XTASK"], "release", "custody", "verify", "--age", os.environ["SIGN_AGE"],
             "--recipient", HERE.parent / ".github/release-evidence-recipient.txt", "--source", C,
             "--run", BUILD, "--attempt", "1", "--transport", work / "transport",
             "--identity", work / "identity", "--output", restored])
    inventory = load(read(restored / "CUSTODY-INVENTORY.json"))
    require(inventory["complete"] is True and inventory["original_exit"] == 0)
    original_manifest = read(restored / "private" / MANIFEST, checksums.MAX_MANIFEST_BYTES)
    require(sha(original_manifest) == inventory["private_manifest_sha256"])
    command([os.environ["SIGN_AGE"], "--decrypt", "--identity", work / "identity", "--output",
             work / "qualification.tar", work / "qualification" / "qualification.tar.age"])
    capsule_receipt = load(read(work / "qualification" / "qualification-receipt.json"))
    require(capsule_receipt == {"schema": 1, "source": C,
                               "manifest_sha256": r["qualification_manifest_sha256"],
                               "ciphertext_sha256": sha(read(work / "qualification" / "qualification.tar.age", api.LIMIT))})
    q = work / "qualification-files"
    restore(work / "qualification.tar", q)
    require(sha(read(q / MANIFEST)) == r["qualification_manifest_sha256"])
    closure = qualification(q)
    final = work / "assembly"
    final.mkdir(mode=0o700)
    extra_labels = {}
    for directory, old_name in [(restored / "private", "BUILD-PREPARATION-CHECKSUMS.json"),
                                (restored / "extras", None), (q, "QUALIFICATION-CHECKSUMS.json")]:
        for f in sorted(directory.iterdir()):
            name = old_name if f.name == MANIFEST else f.name
            require(name is not None)
            if directory == restored / "extras":
                name = "BUILD-EXTRA-" + f.name
                checksums.safe_name(name)
                extra_labels[f.name] = name
            write(final / name, read(f))
    json_write(final / "EVIDENCE-CLOSURE.json", {
        "schema": 1, "source": C, "build_run": BUILD, "build_attempt": "1", "verifier_run": VERIFIER,
        "original_inventory_sha256": sha(read(restored / "CUSTODY-INVENTORY.json")),
        "original_private_manifest_sha256": sha(original_manifest), "qualification": closure,
        "original_extra_labels": extra_labels,
        "limits": "Selected evidence scopes/failures retained; tool digests do not assert retained executable bytes"})
    command([os.environ["SIGN_XTASK"], "release", "checksums", "create", final])
    command([os.environ["SIGN_XTASK"], "release", "checksums", "verify", final])
    provenance, manifest = payloads(final)
    archive(final, work / "frozen.tar")
    output.mkdir(mode=0o700)
    command([os.environ["SIGN_AGE"], "--encrypt", "--recipients-file",
             HERE.parent / ".github/release-evidence-recipient.txt", "--output",
             output / "frozen-signing-inputs.tar.age", work / "frozen.tar"])
    json_write(output / "SIGNING-INPUTS.json", {
        "schema": 1, "source": C, "signer_commit": r["signer_commit"], "tag": TAG,
        "build_run": BUILD, "build_attempt": "1", "verifier_run": VERIFIER, "verifier_attempt": "1",
        "build_artifact": BUILD_ARTIFACT, "build_zip_sha256": BUILD_ZIP,
        "prepare_run": os.environ["GITHUB_RUN_ID"], "prepare_attempt": os.environ["GITHUB_RUN_ATTEMPT"],
        "qualification_asset": r["qualification_asset"], "qualification_sha256": r["qualification_sha256"],
        "qualification_manifest_sha256": r["qualification_manifest_sha256"],
        "original_private_manifest_sha256": sha(original_manifest),
        "provenance_sha256": provenance, "manifest_sha256": manifest,
        "closure_sha256": sha(read(final / "EVIDENCE-CLOSURE.json")),
        "ciphertext_sha256": sha(read(output / "frozen-signing-inputs.tar.age", api.LIMIT)),
        "cosign_sha256": COSIGN_SHA, "root_sha256": ROOT_SHA, "config_sha256": CONFIG_SHA,
        "age_sha256": AGE_SHA, "recipient_sha256": RECIPIENT_SHA,
        "signer_san": SAN, "issuer": ISSUER, "D6": "Pending"})


def signing_input(r, work):
    receipt_raw = read(work / "transport" / "SIGNING-INPUTS.json")
    require(sha(receipt_raw) == r["receipt_sha256"])
    receipt = load(receipt_raw)
    expected = {"schema": 1, "source": C, "signer_commit": r["signer_commit"], "tag": TAG,
                "build_run": BUILD, "build_attempt": "1", "verifier_run": VERIFIER, "verifier_attempt": "1",
                "build_artifact": BUILD_ARTIFACT, "build_zip_sha256": BUILD_ZIP,
                "prepare_run": r["run"], "prepare_attempt": r["attempt"], "provenance_sha256": PUBLIC_SHA,
                "manifest_sha256": r["manifest_sha256"], "cosign_sha256": COSIGN_SHA,
                "root_sha256": ROOT_SHA, "config_sha256": CONFIG_SHA, "signer_san": SAN,
                "age_sha256": AGE_SHA, "recipient_sha256": RECIPIENT_SHA,
                "issuer": ISSUER, "D6": "Pending"}
    require(all(receipt.get(k) == v for k, v in expected.items()))
    require(sha(read(work / "transport" / "frozen-signing-inputs.tar.age", api.LIMIT)) == receipt["ciphertext_sha256"])
    command([os.environ["SIGN_AGE"], "--decrypt", "--identity", work / "identity", "--output",
             work / "frozen.tar", work / "transport" / "frozen-signing-inputs.tar.age"])
    restore(work / "frozen.tar", work / "assembly")
    require(payloads(work / "assembly") == (r["provenance_sha256"], r["manifest_sha256"]))
    require(sha(read(work / "assembly/EVIDENCE-CLOSURE.json")) == receipt["closure_sha256"])


def certificate(bundle, r):
    from cryptography import x509
    from cryptography.x509.oid import ObjectIdentifier
    vm = bundle["verificationMaterial"]
    raw = (vm["certificate"]["rawBytes"] if "certificate" in vm else
           vm["x509CertificateChain"]["certificates"][0]["rawBytes"])
    cert = x509.load_der_x509_certificate(base64.b64decode(raw, validate=True))
    # Cosign verifies chain/SCT/log/timestamps first. These are additional exact claims.
    expected = {8: ISSUER, 9: SAN, 10: r["signer_commit"], 11: "github-hosted",
                12: "https://github.com/bohewu/yetlet", 13: r["signer_commit"],
                14: "refs/heads/master", 15: "1409587171", 16: "https://github.com/bohewu",
                17: "35250002", 18: SAN, 19: r["signer_commit"], 20: "workflow_dispatch",
                21: "https://github.com/bohewu/yetlet/actions/runs/" + os.environ["GITHUB_RUN_ID"] +
                    "/attempts/" + os.environ["GITHUB_RUN_ATTEMPT"],
                22: "public", 23: "release-sign",
                24: "repo:bohewu@35250002/yetlet@1409587171:environment:release-sign"}
    for suffix, text in expected.items():
        value = cert.extensions.get_extension_for_oid(ObjectIdentifier("1.3.6.1.4.1.57264.1." + str(suffix))).value.value
        encoded = text.encode()
        # Every expected claim is <128 bytes; reject other ASN.1 encodings.
        require(len(encoded) < 128 and value == bytes([12, len(encoded)]) + encoded)


def sign(r, work, output, tool_dir):
    require("AGE_IDENTITY" not in os.environ and not (work / "identity").exists())
    tools(tool_dir)
    final = work / "assembly"
    require(payloads(final) == (r["provenance_sha256"], r["manifest_sha256"]))
    output.mkdir(mode=0o700)
    cosign = tool_dir / "cosign"
    gate(r)  # Recheck trusted code, Environment and original frozen artifact before OIDC.
    token_path = work / "oidc-token"
    token = oidc_token(r)
    write(token_path, token.encode())
    common = ["--yes", "--identity-token", token_path, "--fulcio-auth-flow=token",
              "--trusted-root", tool_dir / "trusted_root.json", "--signing-config", tool_dir / "signing_config.json"]
    verify = ["--certificate-identity", SAN, "--certificate-oidc-issuer", ISSUER,
              "--trusted-root", tool_dir / "trusted_root.json", "--use-signed-timestamps"]
    provenance = final / "PROVENANCE-UNSIGNED.json"
    try:
        command([cosign, "attest-blob", "--statement", provenance, "--bundle", output / "provenance.sigstore.json", *common])
        require(payloads(final) == (r["provenance_sha256"], r["manifest_sha256"]))
        command([cosign, "sign-blob", final / MANIFEST, "--bundle", output / "manifest.sigstore.json", *common])
    finally:
        token_path.unlink(missing_ok=True)
    first = sorted(PACKAGES)[0] + "-0.9.0-alpha.1.crate"
    command([cosign, "verify-blob-attestation", final / first, "--bundle", output / "provenance.sigstore.json",
             "--type", "slsaprovenance1", *verify])
    command([cosign, "verify-blob", final / MANIFEST, "--bundle", output / "manifest.sigstore.json", *verify])
    b = load(read(output / "provenance.sigstore.json"))
    require(base64.b64decode(b["dsseEnvelope"]["payload"], validate=True) == read(provenance))
    for name in ["provenance.sigstore.json", "manifest.sigstore.json"]:
        certificate(load(read(output / name)), r)
    require(payloads(final) == (r["provenance_sha256"], r["manifest_sha256"]))
    # Manifest bundle remains private: encrypt it before allowing any upload.
    command([os.environ["SIGN_AGE"], "--encrypt", "--recipients-file",
             HERE.parent / ".github/release-evidence-recipient.txt", "--output",
             output / "manifest.sigstore.json.age", output / "manifest.sigstore.json"])
    (output / "manifest.sigstore.json").unlink()
    json_write(output / "SIGNING-EXECUTION.json", {"schema": 1, "source": C,
        "signer_commit": r["signer_commit"], "approved_inputs": r,
        "sign_run": os.environ["GITHUB_RUN_ID"], "sign_attempt": os.environ["GITHUB_RUN_ATTEMPT"],
        "provenance_bundle_sha256": sha(read(output / "provenance.sigstore.json")),
        "encrypted_manifest_bundle_sha256": sha(read(output / "manifest.sigstore.json.age")),
        "same_job_verification": True, "independent_verification": "Pending",
        "final_owner_acceptance": "Pending", "publication": False})


def pack(source, output):
    # Input is an explicitly reviewed flat qualification set, not a directory crawler.
    outside(str(source))
    qualification(source)
    require(sha(read(HERE.parent / ".github/release-evidence-recipient.txt")) == RECIPIENT_SHA)
    require(sha(read(os.environ["SIGN_AGE"])) == AGE_SHA)
    output.mkdir(mode=0o700)
    archive(source, output / "qualification.tar")
    try:
        command([os.environ["SIGN_AGE"], "--encrypt", "--recipients-file",
                 HERE.parent / ".github/release-evidence-recipient.txt", "--output",
                 output / "qualification.tar.age", output / "qualification.tar"])
    finally:
        (output / "qualification.tar").unlink(missing_ok=True)
    json_write(output / "qualification-receipt.json", {"schema": 1, "source": C,
        "manifest_sha256": sha(read(source / MANIFEST)),
        "ciphertext_sha256": sha(read(output / "qualification.tar.age", api.LIMIT))})
    import zipfile
    with zipfile.ZipFile(output / "qualification-inputs.zip", "x", compression=zipfile.ZIP_STORED) as z:
        for name in ["qualification.tar.age", "qualification-receipt.json"]:
            z.write(output / name, name)


def oidc_claims(claims, r):
    expected = {"iss": ISSUER, "aud": "sigstore", "repository": "bohewu/yetlet",
                "repository_id": "1409587171", "repository_owner_id": "35250002",
                "ref": "refs/heads/master", "sha": r["signer_commit"],
                "workflow_sha": r["signer_commit"],
                "workflow_ref": "bohewu/yetlet/.github/workflows/release-sign.yml@refs/heads/master",
                "event_name": "workflow_dispatch", "runner_environment": "github-hosted",
                "environment": "release-sign",
                "sub": "repo:bohewu@35250002/yetlet@1409587171:environment:release-sign"}
    require(all(claims.get(k) == v for k, v in expected.items()))
    require(str(claims.get("run_id")) == os.environ["GITHUB_RUN_ID"]
            and str(claims.get("run_attempt")) == os.environ["GITHUB_RUN_ATTEMPT"])


def oidc_token(r):
    import urllib.parse
    import urllib.request
    u = urllib.parse.urlsplit(os.environ["ACTIONS_ID_TOKEN_REQUEST_URL"])
    require(u.scheme == "https" and not u.username and not u.fragment
            and u.hostname.endswith(".actions.githubusercontent.com"))
    query = urllib.parse.parse_qsl(u.query, keep_blank_values=True)
    require(not any(k == "audience" for k, _ in query))
    url = urllib.parse.urlunsplit(u._replace(query=urllib.parse.urlencode(query + [("audience", "sigstore")])))
    req = urllib.request.Request(url, headers={"Authorization": "Bearer " + os.environ["ACTIONS_ID_TOKEN_REQUEST_TOKEN"]})
    # No redirects: the bearer is sent only to the trusted runner-provided GitHub endpoint.
    with urllib.request.build_opener(api.NoRedirect()).open(req, timeout=30) as response:
        raw = response.read(65537)
    token = load(raw)["value"]
    require(isinstance(token, str) and len(token) <= 32768 and len(token.split(".")) == 3)
    body = token.split(".")[1]
    claims = load(base64.urlsafe_b64decode(body + "=" * (-len(body) % 4)))
    # TLS-origin claim preflight only; Fulcio verifies the JWT signature, Cosign the certificate.
    oidc_claims(claims, r)
    return token


def main():
    action = sys.argv[1]
    if action == "pack" and len(sys.argv) == 4:
        pack(Path(sys.argv[2]), outside(sys.argv[3]))
        return
    r = request()
    identity(r)
    if action == "preflight":
        gate(r)
        return
    work = outside(os.environ["SIGN_WORK"])
    tool_dir = Path(os.environ["SIGN_TOOLS"])
    if action == "download":
        gate(r)
        work.mkdir(mode=0o700)
        if r["mode"] == "prepare":
            raw = download("/actions/artifacts/" + BUILD_ARTIFACT + "/zip")
            api.unpack(raw, {"digest": BUILD_ZIP}, str(work / "transport"))
            raw = download("/releases/assets/" + r["qualification_asset"], binary=True)
            unpack_zip(raw, r["qualification_sha256"], work / "qualification",
                       {"qualification.tar.age": api.LIMIT, "qualification-receipt.json": 65536})
        else:
            raw = download("/actions/artifacts/" + r["artifact"] + "/zip")
            unpack_zip(raw, r["digest"], work / "transport",
                       {"frozen-signing-inputs.tar.age": api.LIMIT, "SIGNING-INPUTS.json": 65536})
    elif action == "decrypt":
        tools(tool_dir)
        require(sha(read(os.environ["SIGN_AGE"])) == AGE_SHA)
        require(sha(read(HERE.parent / ".github/release-evidence-recipient.txt")) == RECIPIENT_SHA)
        if r["mode"] == "prepare":
            prepare(r, work, outside(os.environ["SIGN_OUTPUT"]))
        else:
            signing_input(r, work)
    elif action == "sign" and r["mode"] == "sign":
        output = outside(os.environ["SIGN_OUTPUT"])
        try:
            sign(r, work, output, tool_dir)
        except Exception:
            seal_failure(r, output)
            raise
    else:
        raise ValueError("unknown operation")


def outside(value):
    p = Path(value)
    require(p.is_absolute() and ".." not in p.parts)
    with checksums.directory_fd(str(p.parent)):
        pass
    require(not p.resolve().is_relative_to(HERE.parent))
    require(not p.is_symlink())
    return p


def seal_failure(r, output):
    # Preserve partial issuance as ciphertext. Never retry signing or upload an unchecked bundle.
    if not output.exists():
        output.mkdir(mode=0o700)
    preserved = []
    seal_ok = True
    for name in ["provenance.sigstore.json", "manifest.sigstore.json"]:
        path = output / name
        if not path.exists():
            continue
        encrypted = output / ("failure-" + name + ".age")
        try:
            require(sha(read(os.environ["SIGN_AGE"])) == AGE_SHA)
            require(sha(read(HERE.parent / ".github/release-evidence-recipient.txt")) == RECIPIENT_SHA)
            command([os.environ["SIGN_AGE"], "--encrypt", "--recipients-file",
                     HERE.parent / ".github/release-evidence-recipient.txt", "--output", encrypted, path])
            preserved.append({"name": encrypted.name, "sha256": sha(read(encrypted))})
        except Exception:
            seal_ok = False
            encrypted.unlink(missing_ok=True)
        finally:
            path.unlink(missing_ok=True)
    json_write(output / "SIGNING-FAILURE.json", {"schema": 1, "source": C,
        "signer_commit": r["signer_commit"], "approved_inputs": r, "status": "FAIL",
        "sign_run": os.environ["GITHUB_RUN_ID"], "sign_attempt": os.environ["GITHUB_RUN_ATTEMPT"],
        "partial_bundle_custody": preserved, "partial_sealing_complete": seal_ok,
        "automatic_retry": False, "independent_verification": "Pending", "publication": False})


def cli():
    try:
        main()
    except GateRejected as error:
        status = "" if error.status is None else f" status={error.status}"
        print(f"release signer rejected; gate={error.stage} class={error.kind}{status}; private diagnostics suppressed",
              file=sys.stderr)
        return 1
    except Exception:
        print("release signer rejected; private diagnostics suppressed", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(cli())
