"""Focused public preparation binding/no-clobber tests; no real keys or Git mutation."""
import hashlib
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

SPEC = importlib.util.spec_from_file_location("rotation", Path(__file__).with_name("release-evidence-rotation.py"))
rotation = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(rotation)
BASE = "a" * 40
OLD = b"age1" + b"a" * 58 + b"\n"
NEW = b"age1" + b"b" * 58 + b"\n"


class PreparationTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.source = self.root / "source"
        self.source.mkdir()
        digest = hashlib.sha256(OLD).hexdigest().encode()
        self.files = {
            rotation.PATHS[0]: OLD,
            rotation.PATHS[1]: (b'pub const RECIPIENT_SHA: &str = "' + digest + b'";\n'
                                b'            "' + OLD.strip() + b'".into(),\n'),
            rotation.PATHS[2]: (b"          printf '%s  %s\\n' " + digest + b" \\\n"
                                b"            .github/release-evidence-recipient.txt | sha256sum --check --strict\n"),
        }
        for name, data in self.files.items():
            path = self.source / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(data)
        self.recipient = self.root / "new-public.txt"
        self.recipient.write_bytes(NEW)
        self.output = self.root / "prepared"
        self.git = patch.object(rotation, "git", side_effect=self.git_read)
        self.git.start()
        self.addCleanup(self.git.stop)

    def git_read(self, source, *args):
        if args == ("rev-parse", "HEAD"):
            return (BASE + "\n").encode()
        if args == ("rev-parse", "--show-toplevel"):
            return (str(self.source) + "\n").encode()
        return self.files[args[1].split(":", 1)[1]]

    def prepare(self):
        return rotation.prepare(self.source, BASE, self.recipient, self.output)

    def test_public_patch_and_hashes_leave_sources_unchanged(self):
        manifest, updated = self.prepare()
        self.assertFalse(manifest["applied"])
        self.assertEqual(manifest, json.loads((self.output / "manifest.json").read_bytes()))
        self.assertEqual(manifest["patch_sha256"], rotation.sha((self.output / "rotation.patch").read_bytes()))
        self.assertEqual(updated[rotation.PATHS[0]], NEW)
        for name in rotation.PATHS:
            self.assertEqual((self.source / name).read_bytes(), self.files[name])
            self.assertEqual(manifest["new_file_sha256"][name], rotation.sha(updated[name]))
        self.assertIn(rotation.sha(NEW).encode(), updated[rotation.PATHS[1]])
        self.assertIn(rotation.sha(NEW).encode(), updated[rotation.PATHS[2]])

    def test_wrong_base_rejected(self):
        with self.assertRaises(ValueError):
            rotation.prepare(self.source, "b" * 40, self.recipient, self.output)
        self.assertFalse(self.output.exists())

    def test_uncommitted_operational_edit_rejected(self):
        (self.source / rotation.PATHS[2]).write_bytes(b"changed")
        with self.assertRaises(ValueError):
            self.prepare()
        self.assertFalse(self.output.exists())

    def test_missing_duplicate_changed_anchors_rejected(self):
        for name, old in list(self.files.items())[1:]:
            for changed in (b"missing", old + old, old.replace(rotation.sha(OLD).encode(), b"c" * 64)):
                with self.subTest(name=name, changed=changed[:10]):
                    altered = dict(self.files)
                    altered[name] = changed
                    with self.assertRaises(ValueError):
                        rotation.replacement(altered, NEW)

    def test_identity_multiline_and_unchanged_input_rejected(self):
        for content in (b"AGE-SECRET-KEY-LOCAL-TEST\n", NEW + NEW, NEW.rstrip(), OLD):
            self.recipient.write_bytes(content)
            with self.assertRaises(ValueError):
                self.prepare()
            self.assertFalse(self.output.exists())

    def test_existing_output_and_dangling_symlink_refused(self):
        self.output.mkdir()
        marker = self.output / "preserve"
        marker.write_bytes(b"keep")
        with self.assertRaises(ValueError):
            self.prepare()
        self.assertEqual(marker.read_bytes(), b"keep")
        marker.unlink()
        self.output.rmdir()
        self.output.symlink_to(self.root / "absent")
        with self.assertRaises(ValueError):
            self.prepare()
        self.assertTrue(self.output.is_symlink())

    def test_symlink_operational_input_rejected(self):
        path = self.source / rotation.PATHS[0]
        target = self.root / "old-public.txt"
        path.rename(target)
        path.symlink_to(target)
        with self.assertRaises(ValueError):
            self.prepare()
        self.assertFalse(self.output.exists())

    def test_changed_binding_before_publication_refused(self):
        with patch.object(rotation, "bound_files", side_effect=[self.files, {}]):
            with self.assertRaises(ValueError):
                self.prepare()
        self.assertFalse(self.output.exists())

    def test_failed_write_leaves_no_partial_output(self):
        original_write = rotation.write
        def fail_manifest(path, data):
            if path.name == "manifest.json":
                raise OSError("simulated write failure")
            original_write(path, data)
        with patch.object(rotation, "write", side_effect=fail_manifest):
            with self.assertRaises(OSError):
                self.prepare()
        self.assertFalse(self.output.exists())


if __name__ == "__main__":
    unittest.main()
