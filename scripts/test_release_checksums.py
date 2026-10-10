#!/usr/bin/env python3
"""YT-064 acceptance cases; temporary fixtures stay under ignored evidence."""
from contextlib import redirect_stderr, redirect_stdout
import io
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch

import release_checksums as checks

ROOT = Path(__file__).resolve().parent.parent
SCRATCH = ROOT / "target" / "m10-remaining"


class ReleaseChecksumsTests(unittest.TestCase):
    def setUp(self):
        SCRATCH.mkdir(parents=True, exist_ok=True)
        self.temp = tempfile.TemporaryDirectory(prefix="checksum-test-", dir=SCRATCH)
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.bundle = self.root / "bundle"
        self.bundle.mkdir()
        (self.bundle / "app.wasm").write_bytes(b"wasm fixture")
        (self.bundle / "inventory.cdx.json").write_bytes(b"{}\n")

    def create(self):
        checks.run("create", str(self.bundle))

    def verify(self):
        checks.run("verify", str(self.bundle))

    def rejects(self, command="verify", directory=None):
        with self.assertRaises((ValueError, OSError)):
            checks.run(command, str(directory or self.bundle))

    def mutate_manifest(self, change):
        manifest = self.bundle / checks.MANIFEST
        data = json.loads(manifest.read_bytes())
        change(data)
        manifest.write_text(json.dumps(data))

    def test_cli_success_and_manifest_exclusion(self):
        script = ROOT / "scripts" / "release_checksums.py"
        for command in ("create", "verify"):
            result = subprocess.run([sys.executable, str(script), command, str(self.bundle)],
                                    capture_output=True, text=True, timeout=5)
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertIn(checks.NOTICE, result.stdout)
        data = json.loads((self.bundle / checks.MANIFEST).read_bytes())
        self.assertEqual(len(data["artifacts"]), 2)
        self.assertEqual(data["notice"], checks.NOTICE)

    def test_canonical_content_for_identical_inputs(self):
        self.create()
        other = self.root / "other"
        other.mkdir()
        for name in ("inventory.cdx.json", "app.wasm"):
            (other / name).write_bytes((self.bundle / name).read_bytes())
        checks.run("create", str(other))
        self.assertEqual((other / checks.MANIFEST).read_bytes(),
                         (self.bundle / checks.MANIFEST).read_bytes())

    def test_no_overwrite(self):
        self.create()
        before = (self.bundle / checks.MANIFEST).read_bytes()
        self.rejects("create")
        self.assertEqual(before, (self.bundle / checks.MANIFEST).read_bytes())
        self.assertEqual((self.bundle / "app.wasm").read_bytes(), b"wasm fixture")

    def test_tampered_artifact_and_hash(self):
        self.create()
        (self.bundle / "app.wasm").write_bytes(b"evil fixture")
        self.rejects()
        (self.bundle / "app.wasm").write_bytes(b"wasm fixture")
        self.mutate_manifest(lambda data: data["artifacts"][0].update(sha256="0" * 64))
        self.rejects()

    def test_missing_and_extra(self):
        self.create()
        original = (self.bundle / "app.wasm").read_bytes()
        (self.bundle / "app.wasm").unlink()
        self.rejects()
        (self.bundle / "app.wasm").write_bytes(original)
        (self.bundle / "extra").write_bytes(b"extra")
        self.rejects()

    def test_unsafe_manifest_paths_and_self_reference(self):
        self.create()
        manifest = self.bundle / checks.MANIFEST
        original = manifest.read_bytes()
        for name in ("../secret", "/etc/passwd", "nested/file", "./app.wasm", "..",
                     "C:\\secret", "a\\b", "a\x00b", checks.MANIFEST):
            with self.subTest(name=name):
                manifest.write_bytes(original)
                self.mutate_manifest(lambda data: data["artifacts"][0].update(path=name))
                self.rejects()

    def test_duplicate_entries_and_json_keys(self):
        self.create()
        manifest = self.bundle / checks.MANIFEST
        original = manifest.read_bytes()
        self.mutate_manifest(lambda data: data["artifacts"].append(data["artifacts"][0]))
        self.rejects()
        manifest.write_bytes(original.replace(b'"schema_version": 1',
                                             b'"schema_version": 1, "schema_version": 1'))
        self.rejects()

    def test_symlink_artifact_create_and_verify(self):
        self.create()
        (self.bundle / "app.wasm").unlink()
        (self.bundle / "app.wasm").symlink_to(self.root / "outside")
        self.rejects()
        (self.bundle / checks.MANIFEST).unlink()
        self.rejects("create")

    def test_symlink_manifest_never_overwritten_or_read(self):
        outside = self.root / "outside"
        outside.write_bytes(b"external content")
        (self.bundle / checks.MANIFEST).symlink_to(outside)
        self.rejects("create")
        self.rejects()
        self.assertEqual(outside.read_bytes(), b"external content")

    def test_symlink_directory_and_ancestor(self):
        self.create()
        alias = self.root / "alias"
        alias.symlink_to(self.bundle, target_is_directory=True)
        self.rejects("verify", alias)
        parent_alias = self.root / "parent-alias"
        parent_alias.symlink_to(self.root, target_is_directory=True)
        self.rejects("verify", parent_alias / "bundle")
        self.rejects("create", parent_alias / "bundle")

    def test_directory_traversal_rejected(self):
        self.rejects("create", self.bundle / ".." / "bundle")

    def test_nonregular_files(self):
        for kind in ("directory", "fifo"):
            with self.subTest(kind=kind):
                path = self.bundle / "nonregular"
                if kind == "directory":
                    path.mkdir()
                else:
                    os.mkfifo(path)
                self.rejects("create")
                if kind == "directory":
                    path.rmdir()
                else:
                    path.unlink()

    def test_nonempty_input(self):
        for path in self.bundle.iterdir():
            path.unlink()
        self.rejects("create")

    def test_count_file_total_and_manifest_limits(self):
        for limit, value in (("MAX_FILES", 1), ("MAX_FILE_BYTES", 2), ("MAX_TOTAL_BYTES", 3)):
            with self.subTest(limit=limit), patch.object(checks, limit, value):
                self.rejects("create")
                self.assertFalse((self.bundle / checks.MANIFEST).exists())
        self.create()
        with patch.object(checks, "MAX_MANIFEST_BYTES", 2):
            self.rejects()

    def test_malformed_manifest_and_types(self):
        self.create()
        manifest = self.bundle / checks.MANIFEST
        original = manifest.read_bytes()
        for change in (lambda data: data.update(schema_version=True),
                       lambda data: data.update(notice="trusted"),
                       lambda data: data.update(artifacts=[]),
                       lambda data: data["artifacts"][0].update(size=True),
                       lambda data: data["artifacts"][0].update(sha256="invalid")):
            manifest.write_bytes(original)
            self.mutate_manifest(change)
            self.rejects()
        manifest.write_bytes(b"not JSON")
        self.rejects()

    def test_failure_output_does_not_disclose_paths(self):
        stderr = io.StringIO()
        with redirect_stderr(stderr), redirect_stdout(io.StringIO()):
            result = checks.main(["verify", str(self.root / "private-name")])
        self.assertEqual(result, 1)
        self.assertEqual(stderr.getvalue(), "release checksum preflight failed\n")


if __name__ == "__main__":
    unittest.main()
