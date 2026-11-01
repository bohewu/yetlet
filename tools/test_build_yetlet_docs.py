"""New public-mode and build-artifact boundaries; no renderer suite replay."""
import importlib.util
import json
from pathlib import Path
import tempfile
import tomllib
import unittest
from unittest.mock import patch

import build_yetlet_docs as guide

spec = importlib.util.spec_from_file_location("guide_prepare", guide.APP / "prepare.py")
prepare = importlib.util.module_from_spec(spec)
spec.loader.exec_module(prepare)


class PublicPreparation(unittest.TestCase):
    commit = "a" * 40

    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.app = Path(self.temp.name)
        (self.app / "src").mkdir()
        self.addCleanup(patch.stopall)
        patch.object(prepare, "APP", self.app).start()

    def test_public_source_and_edition_are_bound(self):
        with patch.object(prepare.subprocess, "check_output", side_effect=[self.commit, ""]):
            result = prepare.prepare(public=True)
        stored = json.loads((self.app / "src/generated/chapters.json").read_text())
        self.assertEqual(stored, result)
        self.assertTrue(result["public"])
        self.assertEqual(result["commit"], self.commit)
        self.assertEqual((result["sdk"], result["cli"]), ("0.9.0-alpha.5", "0.1.1"))
        self.assertIn("/blob/" + self.commit + "/", result["chapters"][0]["markdown"])
        self.assertEqual(result["chapters"][1]["route"], "/first-app/")

    def test_dirty_public_source_is_rejected_before_preparation(self):
        with patch.object(prepare.subprocess, "check_output", side_effect=[self.commit, " M README.md\n"]):
            with self.assertRaisesRegex(ValueError, "clean checkout"):
                prepare.prepare(public=True)
        self.assertFalse((self.app / "src/generated").exists())

    def test_workspace_version_cannot_relabel_the_selected_edition(self):
        selection = tomllib.loads((guide.ROOT / "site/pages.toml").read_text())
        with patch.object(prepare.subprocess, "check_output", side_effect=[self.commit, ""]), \
             patch.object(prepare.tomllib, "loads", side_effect=[selection,
                          {"workspace": {"package": {"version": "0.9.0-alpha.6"}}},
                          {"package": {"version": "0.1.1"}}]):
            with self.assertRaisesRegex(ValueError, "reviewed SDK/CLI"):
                prepare.prepare(public=True)
        self.assertFalse((self.app / "src/generated").exists())

    def test_local_preparation_stays_preview_with_edits(self):
        with patch.object(prepare.subprocess, "check_output", side_effect=[self.commit, " M README.md\n"]):
            self.assertFalse(prepare.prepare()["public"])


class ArtifactBoundary(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        routes = ["/", "/first-app/", "/views/", "/http/", "/browser/", "/features/", "/api/", "/troubleshooting/"]
        self.prepared = {"public": True, "commit": "a" * 40,
                         "chapters": [{"route": r} for r in routes]}
        for route in routes:
            page = self.root / route.strip("/") / "index.html"
            page.parent.mkdir(exist_ok=True)
            page.write_text('<a href="https://github.com/bohewu/yetlet/tree/' + "a" * 40 + '">Source</a>')
        (self.root / ".nojekyll").write_text("")
        (self.root / "assets").mkdir()
        for name in ("guide.css", "search.mjs", "search-data.mjs", "wasm-startup.mjs", "docs_search.js", "docs_search_bg.wasm"):
            (self.root / "assets" / name).write_text("")

    def test_public_artifact_retains_no_legacy_aliases_or_private_manifest(self):
        self.assertEqual(guide.validate_output(self.root, self.prepared), 0)
        leaked = self.root / "EXPORT-MANIFEST.json"
        leaked.write_text("{}")
        with self.assertRaisesRegex(ValueError, "exactly eight chapters"):
            guide.validate_output(self.root, self.prepared)

    def test_local_disclosure_cannot_enter_public_mode(self):
        page = self.root / "index.html"
        page.write_text(page.read_text() + "Local, unpublished checkout")
        with self.assertRaisesRegex(ValueError, "source disclosure"):
            guide.validate_output(self.root, self.prepared)


if __name__ == "__main__":
    unittest.main()
