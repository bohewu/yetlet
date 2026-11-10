"""Regression tests for canonical Yetlet SDK version synchronization."""
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch
import tomllib
import sync_workspace_versions as sync
from release_version_authority import PUBLICATION_RECORDS

TICK = chr(96)
FENCE = TICK * 3


def put(root, relative, text):
    path = root / relative
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text, encoding="utf-8")


def put_sdk_package_manifests(root):
    put(root, "crates/yetlet/Cargo.toml",
        '[package]\nname = "yetlet"\nversion = "0.1.0"\n')
    put(root, "crates/yetlet-browser/Cargo.toml",
        '[package]\nname = "yetlet-browser"\nversion = "0.1.0"\n')


def populate_current_docs(root):
    tick = TICK
    status = '**Published SDK:** `0.1.0`.\n**Published CLI:** `0.1.0`.\n'
    references = ('[SDK publication record](old-sdk.md#old)\n'
                  '[CLI publication record](old-cli.md#old)\n')
    for name in ['ARCHITECTURE', 'RELEASE-READINESS', 'RELEASE-PUBLICATION']:
        put(root, 'docs/'+name+'.md', status + (references if name != 'ARCHITECTURE' else ''))
    put(root, PUBLICATION_RECORDS['SDK'][0],
        "## Owner-approved publication and registry consumer — fixture\n"
        "https://github.com/bohewu/yetlet/releases/tag/v6.5.4-alpha.3)\n")
    put(root, PUBLICATION_RECORDS['CLI'][0],
        "## Owner-approved publication and installed consumer — fixture\n"
        "https://github.com/bohewu/yetlet/releases/tag/yetlet-cli-v2.2.0)\n")
    put(root, "docs/API-REFERENCE.md",
        "SDK (" + tick + "0.1.0" + tick + ") from independent CLI tooling and checkout samples.\n"
        "Available since SDK " + tick + "0.9.0-alpha.6" + tick + ".\n"
        "Published SDKalpha.6/CLI0.2.0 artifacts serve separate roles.\n"
        "YT-245 selects SDK " + tick + "0.1.0" + tick + " locally.\n")
    put(root, "docs/CAPABILITIES.md",
        "version " + tick + "0.1.0" + tick + ", available from crates.io.\n"
        "Published SDKalpha.6/CLI0.2.0 artifacts serve separate roles.\n"
        "YT-245 selects local SDK\n" + tick + "0.1.0" + tick + " contents.\n")
    put(root, "docs/COMPATIBILITY.md",
        "## Selected local SDK 0.1.0 typed helpers\n"
        "YT-245 selects SDK\n" + tick + "0.1.0" + tick + " locally.\n"
        "The methods are now **published in SDK " + tick + "0.1.0" + tick + "**;\n"
        "The published SDK alpha.6 and CLI 0.2.0 packages.\n"
        "The published SDK is " + tick + "yetlet" + tick + " / " +        tick + "yetlet-browser" + tick + " " + tick + "0.1.0" + tick + ";\n"
        "the independently published CLI is " + tick + "yetlet-cli" + tick + " " +
        tick + "0.1.0" + tick + ", generating apps pinned to SDKalpha.6.\n"
        "SDKalpha.6/CLI0.2.0 in [the site manifest]\n"
        "Style templates are distributed in CLI 0.2.0.\n" + references)
    put(root, "docs/VIEW-COMPONENTS.md",
        "Experimental SDK " + tick + "0.1.0" + tick + ", published on crates.io.\n"
        "The locally selected SDK " + tick + "0.1.0" + tick + " includes typed tags.\n"
        "available from published SDK " + tick + "0.1.0" + tick + ".\n")
    put(root, "docs/DOCUMENTATION-SCOPE.md",
        "The published Experimental SDK is yetlet and yetlet-browser\n"
        + tick + "0.1.0" + tick + "; the independently published CLI is "
        + tick + "yetlet-cli" + tick + " " + tick + "0.1.0" + tick + ".\n"
        "The public reader/first-app edition is the explicitly selected SDKalpha.6/CLI0.2.0\n"
        + references)
    for name in ("CONFIGURATION", "EXTENSIONS", "HTTP-PIPELINE",
                 "SECRET-PROVIDERS", "SETTINGS-SECRET-PROVIDER"):
        text = "Experimental SDK integration. These modules are distributed in SDK\n"
        text += tick + "0.1.0" + tick + "; use the documented features and target.\n"
        if name == "SETTINGS-SECRET-PROVIDER":
            text += "yetlet = { version = \"=0.1.0\", path = \"/absolute/yetlet/crates/yetlet\" }\n"
        put(root, "docs/" + name + ".md", text)
    put(root, "docs/OPT-IN-QUICK-REFERENCE.md",
        "describes the published Experimental SDK\n" + tick + "0.1.0" + tick + ".\n")
    put(root, "docs/TROUBLESHOOTING.md",
        "The SDK " + tick + "0.1.0" + tick + " and independent source-installed CLI "
        + tick + "0.1.0" + tick + " are published Experimental versions.\n")
    put(root, "docs/BROWSER-APP.md",
        "published CLI " + tick + "0.1.0" + tick + " and SDK\n"
        + tick + "0.9.0-alpha.6" + tick + ".\n")
    put(root, "docs/README.md",
        "[SDK " + tick + "0.1.0" + tick + "](ALPHA7-HOSTED-RECORD.md)\n"
        "[CLI " + tick + "0.1.0" + tick + "](CLI-020-RELEASE-PREPARATION.md)\n"
        "CLI0.1.0 → protected standalone project → SDK0.9.0-alpha.6 → app lock/cache\n")
    put(root, "README.md",
        status + references + "## Start with native SSR\n"
        "This quickstart uses SDK " + tick + "0.9.0-alpha.6" + tick +
        " and independently versioned CLI " + tick + "0.1.0" + tick + ".\n"
        "https://crates.io/crates/yetlet/0.9.0-alpha.6\n"
        "https://github.com/bohewu/yetlet/releases/tag/v0.9.0-alpha.6\n"
        "cargo +1.99.0 install yetlet-cli --version 0.1.0 --locked\n"
        "Expected version output: `yetlet 0.1.0`.\n"
        "yetlet new firstapp --sdk-version 0.9.0-alpha.6\n"
        "Expected output: " + tick + "yetlet 0.1.0" + tick + ".\n"
        "The server listens on 127.0.0.1:3000.\n"
        "\n## Next section\nDo not change this section.\n")
    put(root, "docs/FIRST-APP.md",
        "Create an app with CLI " + tick + "0.1.0" + tick +
        " and SDK " + tick + "0.9.0-alpha.6" + tick + ".\n"
        "\n## Prerequisites\n"
        "The CLI version is independent of SDK " + tick + "0.9.0-alpha.6" + tick + ".\n"
        "The generated manifest selects " + tick + 'yetlet = "=0.9.0-alpha.6"' +
        tick + " from crates.io.\n"
        "cargo +1.99.0 install yetlet-cli --version 0.1.0 --locked\n"
        "\n## Next: static site\nThe static-site sample is pinned separately.\n")
    put(root, "examples/docs-site/README.md",
        "registry SDK0.9.0-alpha.6, reader edition pinned to SDKalpha.6/CLI0.1.0.\\n")
    put(root, "examples/hello-world/README.md",
        "The typed helpers are included in the published SDK " + tick + "0.1.0" + tick + ".\n")


class WorkspaceManifestTests(unittest.TestCase):
    def test_only_matching_sdk_path_dependencies_are_updated(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            put_sdk_package_manifests(root)
            relative = Path("examples/sample/Cargo.toml")
            source = (
                "[dependencies]\n"
                'yetlet = { path = "../../crates/yetlet" }\n'
                'framework = { package = "yetlet", path = "../../crates/yetlet" }\n'
                'yetlet-core = { path = "../../crates/yetlet-core" }\n'
                'other = { package = "yetlet", version = "1.2.3" }\n'
                "[target.wasm32.dependencies]\n"
                'yetlet-browser = { path = "../../crates/yetlet-browser" }\n')
            put(root, relative.as_posix(), source)
            updates, inventory = sync.sync_manifests(root, [relative], "7.8.9-alpha.2")
            updated = updates[relative.as_posix()]
            self.assertEqual(len(inventory), 1)
            self.assertEqual(updated.count('version = "7.8.9-alpha.2"'), 3)
            self.assertIn('yetlet-core = { path = "../../crates/yetlet-core" }', updated)
            self.assertIn('other = { package = "yetlet", version = "1.2.3" }', updated)

    def test_registry_only_dependency_with_path_comment_is_not_rewritten(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            put_sdk_package_manifests(root)
            relative = Path("examples/sample/Cargo.toml")
            source = (
                "[dependencies]\n"
                'yetlet = { path = "../../crates/yetlet" }\n'
                "[dev-dependencies]\n"
                'yetlet = { version = "=1.2.3" } # path = "../../crates/yetlet"\n'
            )
            put(root, relative.as_posix(), source)
            updates, _ = sync.sync_manifests(root, [relative], "7.8.9-alpha.2")
            updated = updates[relative.as_posix()]
            self.assertIn('version = "7.8.9-alpha.2"', updated)
            self.assertIn(
                'yetlet = { version = "=1.2.3" } # path = "../../crates/yetlet"',
                updated,
            )
            self.assertEqual(
                tomllib.loads(updated)["dev-dependencies"]["yetlet"]["version"],
                "=1.2.3",
            )

    def test_single_quoted_sdk_path_version_is_updated_without_duplicate_keys(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            put_sdk_package_manifests(root)
            relative = Path("examples/sample/Cargo.toml")
            source = (
                "[dependencies]\n"
                "yetlet = { path = \"../../crates/yetlet\", version = '=1.2.3' }\n"
            )
            put(root, relative.as_posix(), source)
            updates, _ = sync.sync_manifests(root, [relative], "7.8.9-alpha.2")
            updated = updates[relative.as_posix()]
            self.assertIn("version = '7.8.9-alpha.2'", updated)
            self.assertEqual(
                tomllib.loads(updated)["dependencies"]["yetlet"]["version"],
                "7.8.9-alpha.2",
            )

    def test_mixed_inline_and_nested_table_dependencies_fail_closed(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            put_sdk_package_manifests(root)
            relative = Path("examples/sample/Cargo.toml")
            source = (
                "[dependencies]\n"
                'yetlet = { path = "../../crates/yetlet", version = "1.2.3" }\n'
                "[dev-dependencies.yetlet]\n"
                'path = "../../crates/yetlet"\n'
                'version = "1.2.3"\n'
            )
            put(root, relative.as_posix(), source)
            with self.assertRaisesRegex(ValueError, "cannot locate exact single-line dependency"):
                sync.sync_manifests(root, [relative], "7.8.9-alpha.2")
            self.assertEqual((root / relative).read_text(encoding="utf-8"), source)


    def test_multiline_string_examples_cannot_be_rewritten_as_dependency_tables(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            put_sdk_package_manifests(root)
            relative = Path("examples/sample/Cargo.toml")
            source = (
                "[package]\n"
                'name = "fixture"\n'
                'version = "0.1.0"\n'
                'metadata = ["""first""", """\n'
                "Example Cargo snippet:\n"
                "[dependencies]\n"
                'yetlet = { path = "../../crates/yetlet" }\n'
                '""""] # """ in a comment\n'
                "[dependencies.yetlet]\n"
                'path = "../../crates/yetlet"\n'
                'version = "1.2.3"\n'
            )
            put(root, relative.as_posix(), source)
            with self.assertRaisesRegex(ValueError, "cannot locate exact single-line dependency"):
                sync.sync_manifests(root, [relative], "7.8.9-alpha.2")
            self.assertEqual((root / relative).read_text(encoding="utf-8"), source)

    def test_sdk_path_that_escapes_the_declared_package_fails_closed(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            put_sdk_package_manifests(root)
            relative = Path("examples/sample/Cargo.toml")
            source = '[dependencies]\nyetlet = { path = "../../outside" }\n'
            put(root, relative.as_posix(), source)
            with self.assertRaisesRegex(ValueError, "does not resolve to its declared package"):
                sync.sync_manifests(root, [relative], "7.8.9-alpha.2")
            self.assertEqual((root / relative).read_text(encoding="utf-8"), source)

    def test_sdk_path_package_identity_must_match_resolved_crate(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            put_sdk_package_manifests(root)
            relative = Path("examples/sample/Cargo.toml")
            source = (
                "[dependencies]\n"
                'framework = { package = "yetlet-browser", path = "../../crates/yetlet" }\n'
            )
            put(root, relative.as_posix(), source)
            with self.assertRaisesRegex(ValueError, "package identity"):
                sync.sync_manifests(root, [relative], "7.8.9-alpha.2")
            self.assertEqual((root / relative).read_text(encoding="utf-8"), source)

    @patch.object(sync, 'cli_sdk_version', return_value='6.5.4-alpha.3')
    def test_plan_is_idempotent_and_preserves_historical_and_published_versions(self, binding):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            put(root, "Cargo.toml", '[workspace.package]\nversion = "7.8.9-alpha.2"\n')
            put(root, "tools/yetlet-cli/Cargo.toml",
                '[package]\nversion = "3.2.1"\n[dependencies]\nyetlet = "=6.5.4-alpha.3"\n')
            put(root, "site/pages.toml",
                '[edition]\nsdk = "6.5.4-alpha.3"\ncli = "2.2.0"\n')
            put(root, "crates/yetlet/Cargo.toml",
                '[package]\nname = "yetlet"\nversion = "0.1.0"\n')
            put(root, "crates/yetlet-browser/Cargo.toml",
                '[package]\nname = "yetlet-browser"\nversion = "0.1.0"\n'
                '[dependencies]\nyetlet = { path = "../yetlet", version = "0.1.0" }\n')
            put(root, "examples/sample/Cargo.toml",
                '[dependencies]\nyetlet = { path = "../../crates/yetlet" }\n'
                '[target.wasm32.dependencies]\nyetlet-browser = { path = "../../crates/yetlet-browser" }\n')
            put(root, "crates/yetlet/README.md",
                '# SDK\nVersion\n' + TICK + '0.1.0' + TICK + '. See [GitHub releases](https://example.invalid)\n'
                + FENCE + 'toml\n[dependencies]\nyetlet = "=0.1.0"\n' + FENCE + '\n')
            put(root, "crates/yetlet-browser/README.md",
                'Version ' + TICK + '0.1.0' + TICK + '; see [GitHub releases](https://example.invalid)\n'
                + FENCE + 'toml\n[dependencies]\nyetlet = { version = "=0.1.0" }\n'
                'yetlet-browser = "=0.1.0"\n' + FENCE + '\n')
            put(root, "tools/yetlet-cli/README.md",
                '**CLI release:** ' + TICK + '0.1.0' + TICK + '. **Generated SDK:** ' + TICK + '=0.1.0' + TICK + '.\n'
                'cargo +1.99.0 install yetlet-cli --version 0.1.0 --locked\n'
                + TICK + 'yetlet --version' + TICK + ' reports exactly\n'
                + TICK + 'yetlet 0.1.0' + TICK + '. Actual publication status and scope belong to the\n'
                + '[versioned release records](https://example.invalid), separately from this installation reference.\n'
                + 'Static export available since CLI ' + TICK + '0.2.0' + TICK + '.\n')
            put(root, "examples/docs-site/Cargo.toml",
                '[dependencies]\nyetlet = { version = "=6.5.4-alpha.3" }\n'
                '[target.wasm32.dependencies]\nyetlet-browser = "=6.5.4-alpha.3"\n')
            reader_manifest_before = (root / "examples/docs-site/Cargo.toml").read_text(encoding="utf-8")
            put(root, "crates/yetlet/guides/core.md",
                'Current SDK ' + TICK + '0.9.0-alpha.6' + TICK + '. Requires Rust 1.99.0.\n'
                '<details>\n<summary>Historical delivery evidence</summary>\n'
                'Experimental ' + TICK + '0.9.0-alpha.5' + TICK + ', unpublished.\n</details>\n')
            put(root, "crates/yetlet-browser/guides/wgpu.md",
                'Current SDK ' + TICK + '0.9.0-alpha.6' + TICK + '. Requires Rust 1.99.0.\n'
                + FENCE + 'toml\n[dependencies]\nyetlet-browser = { version = "=0.9.0-alpha.6" }\nwgpu = "=27.0.1"\n' + FENCE + '\n'
                '<details>\n<summary>Historical delivery evidence</summary>\n'
                'Experimental ' + TICK + '0.9.0-alpha.5' + TICK + ', unpublished.\n</details>\n')
            put(root, "docs/COMPATIBILITY.md",
                '## Selected local SDK alpha.6 typed helpers\n'
                'YT-245 selects SDK\n' + TICK + '0.9.0-alpha.6' + TICK + ' locally.\n'
                '## Current distribution\nPublished SDK is ' + TICK + '0.9.0-alpha.6' + TICK + '.\n')
            put(root, "docs/API-REFERENCE.md",
                'YT-245 selects SDK ' + TICK + '0.9.0-alpha.6' + TICK + ' locally.\n')
            put(root, "docs/CAPABILITIES.md",
                'YT-245 selects local SDK\n' + TICK + '0.9.0-alpha.6' + TICK + ' contents.\n')
            put(root, "docs/VIEW-COMPONENTS.md",
                'The locally selected SDK ' + TICK + '0.9.0-alpha.6' + TICK + ' includes typed tags.\n')
            populate_current_docs(root)
            subprocess.run(["git", "init", "-q"], cwd=root, check=True)
            subprocess.run(["git", "add", "."], cwd=root, check=True)
            put(root, "examples/new-untracked/Cargo.toml",
                '[dependencies]\nyetlet = { path = "../../crates/yetlet" }\n')
            result = sync.plan(root)
            updates = result[-1]
            self.assertTrue(updates)
            self.assertIn("examples/new-untracked/Cargo.toml", updates)
            self.assertNotIn("examples/docs-site/Cargo.toml", updates)
            self.assertEqual(
                (root / "examples/docs-site/Cargo.toml").read_text(encoding="utf-8"),
                reader_manifest_before,
            )
            for relative, value in updates.items():
                (root / relative).write_text(value, encoding="utf-8")
            self.assertEqual(sync.plan(root)[-1], {})
            self.assertEqual(
                (root / "examples/docs-site/Cargo.toml").read_text(encoding="utf-8"),
                reader_manifest_before,
            )
            cli_readme = (root / "tools/yetlet-cli/README.md").read_text(encoding="utf-8")
            self.assertIn(TICK + "yetlet 3.2.1" + TICK + ". Actual publication status", cli_readme)
            self.assertNotIn(TICK + "yetlet 0.1.0" + TICK, cli_readme)
            api = (root / "docs/API-REFERENCE.md").read_text()
            self.assertIn("Available since SDK " + TICK + "0.9.0-alpha.6" + TICK, api)
            self.assertIn("SDK (" + TICK + "6.5.4-alpha.3" + TICK + ")", api)
            self.assertIn("selects SDK " + TICK + "7.8.9-alpha.2" + TICK + " locally", api)
            self.assertIn("127.0.0.1:3000", (root / "README.md").read_text())
            self.assertIn("6.5.4-alpha.3", (root / "README.md").read_text())
            self.assertIn("2.2.0", (root / "README.md").read_text())
            first_app = (root / "docs/FIRST-APP.md").read_text()
            self.assertIn("yetlet = \"=6.5.4-alpha.3\"", first_app)
            self.assertIn("--version 2.2.0 --locked", first_app)
            self.assertIn("registry SDK6.5.4-alpha.3", (root / "examples/docs-site/README.md").read_text())
            self.assertIn("available from published SDK " + TICK + "6.5.4-alpha.3" + TICK, (root / "docs/VIEW-COMPONENTS.md").read_text())
            self.assertIn("included in the published SDK " + TICK + "6.5.4-alpha.3" + TICK, (root / "examples/hello-world/README.md").read_text())

            compatibility = (root / "docs/COMPATIBILITY.md").read_text()
            self.assertIn('The published SDK is ' + TICK + 'yetlet' + TICK + ' / ' + TICK + 'yetlet-browser' + TICK + ' ' + TICK + '6.5.4-alpha.3' + TICK + ';', compatibility)
            self.assertIn('published CLI is ' + TICK + 'yetlet-cli' + TICK + ' ' + TICK + '2.2.0' + TICK, compatibility)
            self.assertIn('with release build SDKalpha.3.', compatibility)
            binding.assert_called_with('2.2.0')
            guide = (root / "crates/yetlet/guides/core.md").read_text()
            self.assertIn('Experimental ' + TICK + '0.9.0-alpha.5' + TICK + ', unpublished.', guide)
            self.assertIn('Rust 1.99.0', guide)
            wgpu_guide = (root / "crates/yetlet-browser/guides/wgpu.md").read_text()
            self.assertIn('wgpu = "=27.0.1"', wgpu_guide)
            self.assertIn('Rust 1.99.0', wgpu_guide)
            self.assertIn('version = "7.8.9-alpha.2"',
                          (root / "examples/sample/Cargo.toml").read_text())
            self.assertIn('yetlet-browser = "=6.5.4-alpha.3"',
                          (root / "examples/docs-site/Cargo.toml").read_text())


    @patch.object(sync, 'cli_sdk_version', return_value='6.5.4-alpha.3')
    def test_published_sdk_does_not_advance_pinned_reader_index(self, binding):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            populate_current_docs(root)
            published = {'SDK': '8.0.0-alpha.7', 'CLI': '2.2.0'}
            edition = {'sdk': '6.5.4-alpha.3', 'cli': '2.2.0'}
            changes = sync.sync_published_docs(root, published,
                                               '=6.5.4-alpha.3', edition)
            index = changes['docs/README.md']
            self.assertIn('[SDK ' + TICK + '8.0.0-alpha.7' + TICK + ']', index)
            self.assertIn('CLI2.2.0 → protected standalone project → SDK6.5.4-alpha.3 → app lock/cache', index)
            self.assertNotIn('CLI2.2.0 → protected standalone project → SDK8.0.0-alpha.7', index)

    def test_new_cli_build_sdk_cannot_change_published_binding_or_reader_manifest_text(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            populate_current_docs(root)
            changes = sync.sync_published_docs(
                root, {'SDK': '0.9.0-alpha.7', 'CLI': '0.2.0'}, '=0.9.0-alpha.7',
                {'sdk': '0.9.0-alpha.6', 'cli': '0.2.0'})
            self.assertIn('yetlet = "=0.9.0-alpha.6"', changes['docs/FIRST-APP.md'])
            self.assertIn('with release build SDKalpha.6.', changes['docs/COMPATIBILITY.md'])
            self.assertNotIn('with release build SDKalpha.7.', changes['docs/COMPATIBILITY.md'])

    def test_style_publication_sync_preserves_historical_absence_and_reader_pin(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            populate_current_docs(root)
            path = root / 'docs/COMPATIBILITY.md'
            path.write_text(path.read_text() + '\n## Historical Style checkpoint\n'
                            'Style was absent from published CLI0.2.0.\n')
            changes = sync.sync_published_docs(
                root, {'SDK':'0.9.0-alpha.7', 'CLI':'0.3.0'}, '=0.9.0-alpha.7',
                {'sdk':'0.9.0-alpha.6', 'cli':'0.2.0'})
            for name in ['API-REFERENCE', 'CAPABILITIES']:
                self.assertIn('Published SDKalpha.7/CLI0.3.0 artifacts', changes['docs/'+name+'.md'])
            self.assertIn('Style templates are distributed in CLI 0.3.0.', changes['docs/COMPATIBILITY.md'])
            self.assertIn('Style was absent from published CLI0.2.0.', changes['docs/COMPATIBILITY.md'])
            self.assertIn('SDKalpha.6/CLI0.2.0 in [the site manifest]', changes['docs/COMPATIBILITY.md'])
            self.assertIn('**Published CLI:** `0.3.0`', changes['README.md'])
            self.assertIn('[CLI publication record](' + PUBLICATION_RECORDS['CLI'][0] +
                          '#owner-approved-publication-and-installed-consumer--fixture)', changes['README.md'])
            self.assertIn(PUBLICATION_RECORDS['CLI'][0].removeprefix('docs/') +
                          '#owner-approved-publication-and-installed-consumer--fixture)', changes['docs/README.md'])


class ReaderManifestTests(unittest.TestCase):
    def test_registry_pin_mismatch_fails_without_rewriting_the_manifest(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            relative = "examples/docs-site/Cargo.toml"
            source = (
                "[dependencies]\n"
                'yetlet = { version = "=0.9.0-alpha.6" }\n'
                "[target.wasm32.dependencies]\n"
                'yetlet-browser = "=0.9.0-alpha.6"\n'
            )
            for relative in (relative, "examples/static-site/Cargo.toml"):
                put(root, relative, source)
                with self.assertRaisesRegex(ValueError, "does not match site/pages.toml"):
                    sync.check_reader_manifest(root, {"sdk": "0.9.0-alpha.7"}, relative)
                self.assertEqual((root / relative).read_text(encoding="utf-8"), source)


class GuideVersionSyncTests(unittest.TestCase):
    def test_reader_commands_follow_the_selected_edition(self):
        text = ('cargo +1.99.0 install yetlet-cli --version 0.2.0 --locked\n'
                '`yetlet 0.2.0`\nyetlet new app --sdk-version 0.9.0-alpha.6\n')
        self.assertEqual(sync.sync_reader_commands(text, {'sdk': '1.2.3', 'cli': '4.5.6'}),
                         'cargo +1.99.0 install yetlet-cli --version 4.5.6 --locked\n'
                         '`yetlet 4.5.6`\nyetlet new app --sdk-version 1.2.3\n')
    def test_current_package_labels_update_but_feature_introduction_versions_are_preserved(self):
        text = (
            "Current package: " + TICK + "yetlet" + TICK + " SDK " + TICK + "0.9.0-alpha.6" + TICK + ".\n"
            "**Since SDKalpha.6:** YT-220 DOM support.\n"
            "**Opt-in since SDKalpha.6:** YT-221 image URL support.\n"
            "Since SDK " + TICK + "0.9.0-alpha.6" + TICK + ": YT-220 historical feature.\n"
            "Available since SDK " + TICK + "0.9.0-alpha.6" + TICK + ": YT-221 historical feature.\n"
            "Introduced in SDK " + TICK + "0.9.0-alpha.6" + TICK + ": another feature.\n"
            "Requires SDK " + TICK + "0.9.0-alpha.6" + TICK + ": minimum supported version.\n"
        )
        updated = sync.sync_guide_text(text, "0.9.0-alpha.7", "fixture.md")
        self.assertIn("Current package: " + TICK + "yetlet" + TICK + " SDK " + TICK + "0.9.0-alpha.7" + TICK + ".", updated)
        for historical in (
            "**Since SDKalpha.6:** YT-220 DOM support.",
            "**Opt-in since SDKalpha.6:** YT-221 image URL support.",
            "Since SDK " + TICK + "0.9.0-alpha.6" + TICK + ": YT-220 historical feature.",
            "Available since SDK " + TICK + "0.9.0-alpha.6" + TICK + ": YT-221 historical feature.",
            "Introduced in SDK " + TICK + "0.9.0-alpha.6" + TICK + ": another feature.",
            "Requires SDK " + TICK + "0.9.0-alpha.6" + TICK + ": minimum supported version.",
        ):
            with self.subTest(historical=historical):
                self.assertIn(historical, updated)
        self.assertNotIn("Since SDK " + TICK + "0.9.0-alpha.7" + TICK, updated)
        self.assertNotIn("Opt-in since SDK " + TICK + "0.9.0-alpha.7" + TICK, updated)


if __name__ == "__main__":
    unittest.main()
