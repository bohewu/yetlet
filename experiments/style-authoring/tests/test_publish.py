"""Focused product tests for the local publication driver, not a harness API."""
import importlib.util
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch

SPEC = importlib.util.spec_from_file_location("publisher", Path(__file__).parents[1] / "publish.py")
p = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(p)
OUTPUT = "assets/generated/app.generated.css"
FIRST = b".app-x { color: #fff; }\n"
SECOND = b".app-x { color: #000; }\n"


class PublicationTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix="yt238-product-test-")
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name).resolve()
        self.target = self.root / OUTPUT
        self.record = self.root / ".stylegen/publication.json"
        self.pending = self.root / ".stylegen/pending.json"

    def accepted(self):
        self.assertEqual(p.publish(self.root, OUTPUT, FIRST), "published")
        return self.target.read_bytes(), self.record.read_bytes()

    def assert_preserved(self, before):
        self.assertEqual((self.target.read_bytes(), self.record.read_bytes()), before)
        self.assertFalse(self.pending.exists())
        self.assertFalse(list(self.root.rglob(".yt238-*")))

    def test_success_replace_and_noop_preserve_both_mtimes(self):
        self.accepted()
        before = (self.target.stat().st_mtime_ns, self.record.stat().st_mtime_ns)
        with patch.object(p.os, "replace", side_effect=AssertionError("no rewrite allowed")):
            self.assertEqual(p.publish(self.root, OUTPUT, FIRST), "unchanged")
        self.assertEqual(before, (self.target.stat().st_mtime_ns, self.record.stat().st_mtime_ns))
        self.assertEqual(p.publish(self.root, OUTPUT, SECOND), "published")
        self.assertEqual(self.target.read_bytes(), SECOND)
        self.assertEqual(p.read_record(self.record, OUTPUT)["sha256"], p.digest(SECOND))

    def test_paths_reject_source_config_lock_cache_absolute_traversal(self):
        for name in ["/tmp/app.generated.css", "../assets/generated/app.generated.css", "assets/generated/../app.generated.css", "assets//generated/app.generated.css", "src/app.generated.css", "frontend/app.generated.css", "Cargo.lock", "Cargo.toml", ".cache/generated/app.generated.css", "assets/generated/app.css", "assets/generated/a/b.generated.css", r"assets\generated\app.generated.css"]:
            with self.subTest(name=name), self.assertRaises(p.PublicationError):
                p.publish(self.root, name, FIRST)
        self.assertEqual(list(self.root.iterdir()), [])

    def test_symlink_and_nonregular_components_and_metadata_rejected(self):
        outside = self.root / "outside"
        outside.mkdir()
        (self.root / "assets").symlink_to(outside, target_is_directory=True)
        with self.assertRaisesRegex(p.PublicationError, "symlink"):
            p.publish(self.root, OUTPUT, FIRST)
        (self.root / "assets").unlink()
        self.target.mkdir(parents=True)
        with self.assertRaisesRegex(p.PublicationError, "nonregular"):
            p.publish(self.root, OUTPUT, FIRST)
        self.target.rmdir()
        os.mkfifo(self.target)
        with self.assertRaisesRegex(p.PublicationError, "nonregular"):
            p.publish(self.root, OUTPUT, FIRST)
        self.target.unlink()
        (self.root / ".stylegen").symlink_to(outside, target_is_directory=True)
        with self.assertRaisesRegex(p.PublicationError, "symlink"):
            p.publish(self.root, OUTPUT, FIRST)

    def test_unowned_manual_change_missing_and_corrupt_record_conflicts(self):
        self.target.parent.mkdir(parents=True)
        self.target.write_bytes(FIRST)
        with self.assertRaisesRegex(p.PublicationError, "unowned"):
            p.publish(self.root, OUTPUT, FIRST)
        self.target.unlink()
        self.accepted()
        self.target.write_bytes(SECOND)
        with self.assertRaisesRegex(p.PublicationError, "manually changed"):
            p.publish(self.root, OUTPUT, SECOND)
        self.target.unlink()
        with self.assertRaisesRegex(p.PublicationError, "missing"):
            p.publish(self.root, OUTPUT, FIRST)
        self.record.write_text("{}")
        with self.assertRaisesRegex(p.PublicationError, "unowned"):
            p.publish(self.root, OUTPUT, FIRST)

    def test_invalid_artifact_preserves_accepted_output(self):
        before = self.accepted()
        for data in [b"", b"\xff", b"x" * (p.LIMIT + 1)]:
            with self.assertRaises(p.PublicationError):
                p.publish(self.root, OUTPUT, data)
            self.assert_preserved(before)

    def test_capture_failure_utf8_and_size_never_publish(self):
        before = self.accepted()
        for program in ["import sys; print('partial'); sys.exit(7)", "import sys; sys.stdout.buffer.write(b'\\xff')", f"print('x' * {p.LIMIT})", "pass"]:
            with self.assertRaises(p.PublicationError):
                css = p.capture([sys.executable, "-c", program], self.root)
                p.publish(self.root, OUTPUT, css)
            self.assert_preserved(before)
        css = p.capture([sys.executable, "-c", "import sys; sys.stderr.write('controlled diagnostic\\n'); print('.x {}')"], self.root)
        self.assertEqual(css, b".x {}\n")

    def test_prepublication_write_failure_preserves_bytes_and_record(self):
        before = self.accepted()
        original = p.os.replace
        def fail_css(source, destination):
            if destination == self.target:
                raise OSError("controlled pre-rename failure")
            return original(source, destination)
        with patch.object(p.os, "replace", side_effect=fail_css), self.assertRaises(OSError):
            p.publish(self.root, OUTPUT, SECOND)
        self.assert_preserved(before)

    def test_post_css_metadata_failure_is_unaccepted_requires_reconciliation(self):
        before = self.accepted()
        original = p.os.replace
        def fail_metadata(source, destination):
            if destination == self.record:
                raise OSError("controlled metadata failure")
            return original(source, destination)
        with patch.object(p.os, "replace", side_effect=fail_metadata), self.assertRaisesRegex(p.PublicationError, "unaccepted partial"):
            p.publish(self.root, OUTPUT, SECOND)
        self.assertEqual(self.target.read_bytes(), SECOND)
        self.assertEqual(self.record.read_bytes(), before[1])
        self.assertTrue(self.pending.exists())
        with self.assertRaisesRegex(p.PublicationError, "reconciliation"):
            p.publish(self.root, OUTPUT, SECOND)
        self.assertEqual(p.reconcile(self.root, OUTPUT), "explicitly reconciled")
        self.assertFalse(self.pending.exists())
        self.assertEqual(p.publish(self.root, OUTPUT, SECOND), "unchanged")

    def test_crash_after_first_css_rename_retains_intent(self):
        # A controlled SystemExit at the post-rename metadata boundary models
        # interrupted acceptance; the retained intent is also what a crash leaves.
        original = p.os.replace
        def crash_metadata(source, destination):
            if destination == self.record:
                raise SystemExit("controlled crash boundary")
            return original(source, destination)
        with patch.object(p.os, "replace", side_effect=crash_metadata), self.assertRaisesRegex(p.PublicationError, "unaccepted partial"):
            p.publish(self.root, OUTPUT, FIRST)
        self.assertFalse(self.record.exists())
        self.assertEqual(self.target.read_bytes(), FIRST)
        with self.assertRaises(p.PublicationError):
            p.publish(self.root, OUTPUT, FIRST)
        self.target.write_bytes(SECOND)
        with self.assertRaisesRegex(p.PublicationError, "mismatch"):
            p.reconcile(self.root, OUTPUT)
        self.target.write_bytes(FIRST)
        self.assertEqual(p.reconcile(self.root, OUTPUT), "explicitly reconciled")

    def test_platform_claim_is_linux_only(self):
        with patch.object(p.sys, "platform", "win32"), self.assertRaisesRegex(p.PublicationError, "Windows deferred"):
            p.publish(self.root, OUTPUT, FIRST)


if __name__ == "__main__":
    unittest.main()
