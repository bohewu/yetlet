"""New public-mode and build-artifact boundaries; no renderer suite replay."""
import importlib.util
import json
import os
from pathlib import Path
import subprocess
import tempfile
import textwrap
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
        (self.app / "Cargo.toml").write_text((guide.APP / "Cargo.toml").read_text())
        self.addCleanup(patch.stopall)
        patch.object(prepare, "APP", self.app).start()

    def test_public_source_and_edition_are_bound(self):
        with patch.object(prepare.subprocess, "check_output", side_effect=[self.commit, ""]):
            result = prepare.prepare(public=True)
        stored = json.loads((self.app / "src/generated/chapters.json").read_text())
        self.assertEqual(stored, result)
        self.assertTrue(result["public"])
        self.assertEqual(result["commit"], self.commit)
        self.assertEqual((result["sdk"], result["cli"]), ("0.9.0-alpha.6", "0.2.0"))
        self.assertIn("/blob/" + self.commit + "/", result["chapters"][0]["markdown"])
        self.assertEqual(result["chapters"][1]["route"], "/first-app/")

    def test_dirty_public_source_is_rejected_before_preparation(self):
        with patch.object(prepare.subprocess, "check_output", side_effect=[self.commit, " M README.md\n"]):
            with self.assertRaisesRegex(ValueError, "clean checkout"):
                prepare.prepare(public=True)
        self.assertFalse((self.app / "src/generated").exists())

    def test_new_workspace_version_retains_the_published_consumer_edition(self):
        selection = tomllib.loads((guide.ROOT / "site/pages.toml").read_text())
        consumer = tomllib.loads((self.app / "Cargo.toml").read_text())
        with patch.object(prepare.subprocess, "check_output", side_effect=[self.commit, ""]), \
             patch.object(prepare.tomllib, "loads", side_effect=[selection,
                          {"workspace": {"package": {"version": "0.9.0-alpha.7"}}},
                          {"package": {"version": "0.3.0"}}, consumer]):
            result = prepare.prepare(public=True)
        self.assertEqual((result["sdk"], result["cli"]), ("0.9.0-alpha.6", "0.2.0"))
        self.assertTrue((self.app / "src/generated/chapters.json").is_file())

    def test_unadmitted_candidate_edition_is_rejected_before_preparation(self):
        selection = tomllib.loads((guide.ROOT / "site/pages.toml").read_text())
        selection["edition"] = {"sdk": "0.9.0-alpha.7", "cli": "0.3.0"}
        with patch.object(prepare.subprocess, "check_output", side_effect=[self.commit, ""]), \
             patch.object(prepare.tomllib, "loads", side_effect=[selection,
                          {"workspace": {"package": {"version": "0.9.0-alpha.7"}}},
                          {"package": {"version": "0.3.0"}}]):
            with self.assertRaisesRegex(ValueError, "reviewed SDK/CLI"):
                prepare.prepare(public=True)
        self.assertFalse((self.app / "src/generated").exists())

    def test_local_preparation_stays_preview_with_edits(self):
        with patch.object(prepare.subprocess, "check_output", side_effect=[self.commit, " M README.md\n"]):
            self.assertFalse(prepare.prepare()["public"])

    def test_mismatched_consumer_sdk_is_rejected_before_preparation(self):
        manifest = self.app / "Cargo.toml"
        manifest.write_text(manifest.read_text().replace("=0.9.0-alpha.6", "=0.9.0-alpha.5", 1))
        with patch.object(prepare.subprocess, "check_output", side_effect=[self.commit, ""]):
            with self.assertRaisesRegex(ValueError, "selected registry SDK exactly"):
                prepare.prepare(public=True)
        self.assertFalse((self.app / "src/generated").exists())


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


class PagesWorkflow(unittest.TestCase):
    """Exercise only the new workflow/output connection, without build replay."""

    @classmethod
    def setUpClass(cls):
        cls.workflow = (guide.ROOT / ".github/workflows/docs.yml").read_text()
        cls.build, cls.deploy = cls.workflow.split("\n  deploy:\n", 1)
        step = cls.workflow.split("      - name: Select deployment eligibility\n", 1)[1]
        cls.mode = textwrap.dedent(step.split("        run: |\n", 1)[1].split("\n      - ", 1)[0])

    def run_mode(self, ref, event, enabled):
        with tempfile.TemporaryDirectory() as temp:
            output = Path(temp) / "output"
            result = subprocess.run(["bash", "-euo", "pipefail", "-c", self.mode],
                                    env={"PATH": os.environ["PATH"], "GITHUB_REF": ref,
                                         "GITHUB_EVENT_NAME": event, "DEPLOY_ENABLED": enabled,
                                         "GITHUB_OUTPUT": str(output)}, capture_output=True, text=True)
            self.assertEqual(result.returncode, 0, result.stderr)
            return output.read_text()

    def test_public_format_does_not_grant_deployment(self):
        for ref, event, enabled in [("refs/heads/master", "push", "false"),
                                    ("refs/pull/44/merge", "pull_request", "true")]:
            with self.subTest(ref=ref):
                self.assertEqual(self.run_mode(ref, event, enabled), "deploy=false\n")
        self.assertIn("python3 tools/build_yetlet_docs.py --public", self.build)

    def test_selected_master_emits_the_deploy_output(self):
        self.assertEqual(self.run_mode("refs/heads/master", "workflow_dispatch", "true"),
                         "deploy=true\n")
        self.assertIn("deploy: ${{ steps.mode.outputs.deploy }}", self.build)
        self.assertIn("needs.build.outputs.deploy == 'true'", self.deploy)
        self.assertIn("vars.PAGES_DEPLOY_ENABLED == 'true'", self.deploy)
        self.assertIn("github.ref == 'refs/heads/master'", self.deploy)

    def test_one_site_upload_keeps_hidden_files_and_records_separate(self):
        self.assertFalse((guide.ROOT / ".github/workflows/docs-preview.yml").exists())
        site = self.build.split("      - name: Upload only the generated site root\n", 1)[1]
        site, records = site.split("      - name: Retain source and export receipts separately\n", 1)
        self.assertIn("path: ${{ runner.temp }}/yetlet-guide/public/yetlet\n", site)
        self.assertIn("name: github-pages\n", site)
        self.assertIn("include-hidden-files: true", site)
        self.assertNotIn("EXPORT-MANIFEST", site)
        self.assertIn("EXPORT-MANIFEST.json", records)
        self.assertIn("STATIC-RECEIPTS/", records)
        self.assertIn("artifact_name: github-pages", self.deploy)
        self.assertIn("needs: build", self.deploy)

    def test_build_permissions_stay_readonly(self):
        permissions = self.workflow.split("\npermissions:\n", 1)[1].split("\njobs:\n", 1)[0]
        self.assertEqual(permissions.strip(), "contents: read")
        for privileged in ("pages: write", "id-token: write", "environment:", "secrets."):
            self.assertNotIn(privileged, self.build)
        self.assertIn("cancel-in-progress: false", self.deploy)


if __name__ == "__main__":
    unittest.main()
