"""Focused publication-label and actual site-root checks; no product suite replay."""

import tempfile
import tomllib
import unittest
from pathlib import Path

import build_docs

ROOT = Path(__file__).resolve().parents[1]
SITE_EDITION = tomllib.loads((ROOT / "site/pages.toml").read_text())["edition"]


class PublicationLabels(unittest.TestCase):
    commit = "a" * 40
    edition = SITE_EDITION

    def label(self, *, dirty=False, edition=None, public=True):
        return build_docs.page_label(self.commit, dirty, "9.9.9-alpha.1", "9.9.9",
                                     self.edition if edition is None else edition, public)

    def test_public_labels_bind_full_commit_and_explicit_site_edition(self):
        label = self.label()
        self.assertIn(f"/tree/{self.commit}", label)
        self.assertIn(f"SDK `{self.edition['sdk']}`", label)
        self.assertIn(f"CLI `{self.edition['cli']}`", label)
        self.assertNotIn("Local documentation preview", label)

    def test_workspace_versions_do_not_implicitly_promote_the_reader_edition(self):
        label = self.label()
        self.assertNotIn("9.9.9-alpha.1", label)
        self.assertNotIn("9.9.9", label)

    def test_missing_public_edition_is_refused(self):
        with self.assertRaisesRegex(ValueError, "site/pages.toml"):
            build_docs.page_label(self.commit, False, "9.9.9-alpha.1", "9.9.9", None, True)

    def test_dirty_public_input_is_refused(self):
        with self.assertRaisesRegex(ValueError, "clean checkout"):
            self.label(dirty=True)

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
