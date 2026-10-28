#!/usr/bin/env python3
"""Authenticate pinned public signer tools; never request an identity or sign."""
import base64
import hashlib
import json
from pathlib import Path
import subprocess
import sys
import urllib.request
from tuf.ngclient import Updater

from importlib.util import spec_from_file_location, module_from_spec
spec = spec_from_file_location("release_sign", Path(__file__).with_name("release-sign.py"))
signer = module_from_spec(spec)
spec.loader.exec_module(signer)


def get(url, path, expected, limit):
    with urllib.request.urlopen(url, timeout=120) as r:
        raw = r.read(limit + 1)
    signer.require(len(raw) <= limit and hashlib.sha256(raw).hexdigest() == expected)
    signer.write(path, raw)


def main(root):
    root = Path(root)
    metadata, targets = root / "tuf-metadata", root / "tuf-targets"
    metadata.mkdir(mode=0o700)
    targets.mkdir(mode=0o700)
    get("https://raw.githubusercontent.com/sigstore/root-signing/09d1e2645cec6b7b2ea10f32b5dcb0c2773df18a/metadata/root_history/10.root.json",
        metadata / "root.json", "836bff947925edfc23eb9ce17af66fb1e43bb5e2bdd240520985ae52b585eae9", 65536)
    authenticate(root)


def authenticate(root):
    metadata, targets = root / "tuf-metadata", root / "tuf-targets"
    updater = Updater(str(metadata), "https://tuf-repo-cdn.sigstore.dev/", str(targets),
                      "https://tuf-repo-cdn.sigstore.dev/targets/",
                      bootstrap=(metadata / "root.json").read_bytes())
    updater.refresh()  # Threshold signatures, rollback and metadata expiry are enforced by python-tuf.
    for name, destination, digest in [
        ("artifact.pub", "artifact.pub", "59ebf97a9850aecec4bc39c1f5c1dc46e6490a6b5fd2a6cacdcac0c3a6fc4cbf"),
        ("trusted_root.json", "trusted_root.json", signer.ROOT_SHA),
        ("signing_config_rekor_v2.v0.2.json", "signing_config.json", signer.CONFIG_SHA),
    ]:
        info = updater.get_targetinfo(name)
        signer.require(info is not None)
        path = updater.download_target(info)
        raw = Path(path).read_bytes()
        signer.require(signer.sha(raw) == digest)
        signer.write(root / destination, raw)
    base = "https://github.com/sigstore/cosign/releases/download/v3.1.4/"
    get(base + "cosign-linux-amd64", root / "cosign", signer.COSIGN_SHA, 160 * 1024 * 1024)
    get(base + "cosign-linux-amd64-kms.sigstore.json", root / "kms.json",
        "b1282021bf1d6f1031d5df46cba12386c9b438d56d276d72f4df184a0008512b", 65536)
    get(base + "cosign-linux-amd64.sigstore.json", root / "keyless.json",
        "355244bb0186e950ced20f03106640ba3a4630e1223d0467ff34bcebe10bb18a", 65536)
    kms = json.loads((root / "kms.json").read_bytes())
    signer.write(root / "kms.sig", base64.b64decode(kms["messageSignature"]["signature"], validate=True))
    # Never execute the downloaded verifier before independently checking its artifact-key signature.
    signer.command(["openssl", "dgst", "-sha256", "-verify", root / "artifact.pub", "-signature",
                    root / "kms.sig", root / "cosign"])
    (root / "cosign").chmod(0o700)
    signer.command([root / "cosign", "verify-blob", "--bundle", root / "keyless.json",
                    "--trusted-root", root / "trusted_root.json", "--certificate-identity",
                    "keyless@projectsigstore.iam.gserviceaccount.com", "--certificate-oidc-issuer",
                    "https://accounts.google.com", root / "cosign"])
    signer.json_write(root / "bootstrap-receipt.json", {"schema": 1, "tuf_authenticated": True,
        "artifact_key_verified": True, "upstream_keyless_verified": True,
        "cosign_sha256": signer.COSIGN_SHA, "root_sha256": signer.ROOT_SHA, "config_sha256": signer.CONFIG_SHA})
    print("pinned Cosign 3.1.4/TUF bootstrap: PASS; no Yetlet identity requested")


if __name__ == "__main__":
    try:
        main(sys.argv[1])
    except Exception:
        print("signer tool authentication failed; no private key or OIDC accessed", file=sys.stderr)
        sys.exit(1)
