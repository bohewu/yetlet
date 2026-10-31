"""Focused publication-label and actual site-root checks; no product suite replay."""

import tempfile
import unittest
from pathlib import Path

import build_docs


class PublicationLabels(unittest.TestCase):
    commit = "a" * 40
    edition = {"sdk": "0.9.0-alpha.5", "cli": "0.1.1"}

    def label(self, *, dirty=False, edition=None, public=True):
        return build_docs.page_label(self.commit, dirty, self.edition["sdk"],
                                     self.edition["cli"], edition or self.edition, public)

    def test_public_labels_bind_full_commit_and_explicit_edition(self):
        label = self.label()
        self.assertIn(f"/tree/{self.commit}", label)
        self.assertIn("0.9.0-alpha.5", label)
        self.assertIn("0.1.1", label)
        self.assertNotIn("Local documentation preview", label)

    def test_dirty_public_input_is_refused(self):
        with self.assertRaisesRegex(ValueError, "clean checkout"):
            self.label(dirty=True)

    def test_workspace_version_does_not_silently_change_public_edition(self):
        with self.assertRaisesRegex(ValueError, "reviewed SDK/CLI"):
            self.label(edition={"sdk": "0.9.0-alpha.6", "cli": "0.1.1"})

    def test_preview_still_allows_local_edits(self):
        label = self.label(dirty=True, public=False)
        self.assertIn("Local documentation preview", label)
        self.assertIn("with local edits", label)


class SiteOutput(unittest.TestCase):
    pages = [{"output": "index.md"}, {"output": "next.md"}]

    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        (self.root / "index.html").write_text('<a href="next.html#step">Next</a>')
        (self.root / "next.html").write_text('<h1 id="step">Step</h1><a href="/yetlet/index.html">Home</a>')

    def test_selected_project_root_and_anchors(self):
        self.assertEqual(build_docs.validate_output(self.root, self.pages), 2)

    def test_parent_directory_is_not_a_valid_upload_root(self):
        parent = self.root / "parent"
        parent.mkdir()
        with self.assertRaisesRegex(ValueError, "Missing rendered chapter"):
            build_docs.validate_output(parent, self.pages)

    def test_broken_step_anchor_is_refused(self):
        (self.root / "index.html").write_text('<a href="next.html#old-step">Next</a>')
        with self.assertRaisesRegex(ValueError, "Missing local anchor"):
            build_docs.validate_output(self.root, self.pages)

    def test_link_outside_upload_root_is_refused(self):
        (self.root / "index.html").write_text('<a href="../outside.html">Outside</a>')
        with self.assertRaisesRegex(ValueError, "out-of-root"):
            build_docs.validate_output(self.root, self.pages)

    def test_source_and_manifest_are_not_publishable_assets(self):
        for name in ("source.md", "settings.toml", "source.rs", "EXPORT-MANIFEST.json"):
            with self.subTest(name=name):
                path = self.root / name
                path.write_text("not a site asset")
                with self.assertRaisesRegex(ValueError, "source/manifest"):
                    build_docs.validate_output(self.root, self.pages)
                path.unlink()


if __name__ == "__main__":
    unittest.main()
