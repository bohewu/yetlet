"""Bounded offline checks for current references and frozen package README gates."""
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

import release_docs as docs
import release_record as records
import release_registry as registry
from test_release_publication import archives


def cli_readme(version='7.8.9-beta.2', sdk='=6.5.4-alpha.3'):
    return (f'**CLI release:** `{version}`. **Generated SDK:** `{sdk}`.\n\n'
            f'cargo +1.99.0 install yetlet-cli --version {version} --locked\n\n'
            f'Expected output: `yetlet {version}`.\n')


class ReleaseDocsTests(unittest.TestCase):
    def test_cli_versions_and_sdk_pins_are_selected_values_not_current_release_literals(self):
        for version, sdk in [('7.8.9-beta.2', '=6.5.4-alpha.3'), ('12.0.0', '=11.0.0')]:
            docs.check_readme('yetlet-cli', version, cli_readme(version, sdk), sdk)
            with self.assertRaises(ValueError):
                docs.check_readme('yetlet-cli', version + '-wrong', cli_readme(version, sdk), sdk)
            with self.assertRaises(ValueError):
                docs.check_readme('yetlet-cli', version, cli_readme(version, sdk), '=wrong')

    def test_install_requires_exact_version_and_locked(self):
        text = cli_readme()
        command = 'cargo +1.99.0 install yetlet-cli --version 7.8.9-beta.2 --locked'
        for bad in ['cargo install yetlet-cli --locked', command.replace('7.8.9-beta.2', '^7.8.9'),
                    command.replace(' --locked', ''), command + ' --version 7.8.9-beta.2',
                    command + ' --git https://example.invalid/source', command + ' --registry=other']:
            with self.subTest(command=bad), self.assertRaises(ValueError):
                docs.check_readme('yetlet-cli', '7.8.9-beta.2', text.replace(command, bad), '=6.5.4-alpha.3')
        docs.check_installs('cargo install yetlet-cli --version=12.0.0 \\\n --locked', '12.0.0', 'recipe')

    def test_neutral_package_wording_and_missing_or_duplicate_references(self):
        text = cli_readme()
        for bad in [text + '\nThis release is published.\n', text.replace('CLI release', 'CLI'),
                    text + '\n**CLI release:** `7.8.9-beta.2`.\n']:
            with self.subTest(text=bad), self.assertRaises(ValueError):
                docs.check_readme('yetlet-cli', '7.8.9-beta.2', bad, '=6.5.4-alpha.3')

    def test_original_stale_phrases_and_wrapped_selected_version_are_blocked(self):
        for stale in ['CLI7.8.9-beta.2 publication is pending',
                      'CLI `7.8.9-beta.2` is not yet available',
                      'SDK 7.8.9-beta.2 is\nnot yet published',
                      '**CLI release:** `7.8.9-beta.2`. Status: pending.',
                      'SDKbeta.2 remains pending.',
                      '**Unreleased checkout additions:** new options',
                      '## Unreleased static HTML build']:
            with self.subTest(text=stale), self.assertRaises(ValueError):
                docs.check_status(stale, {'CLI': '7.8.9-beta.2', 'SDK': '7.8.9-beta.2'}, 'guide')

    def test_dated_history_and_old_versions_do_not_become_current_claims(self):
        docs.check_status('CLI7.8.9-beta.20 remains pending.', {'CLI': '7.8.9-beta.2'}, 'guide')
        docs.check_status('Current release.\n\n## Historical preparation — 2020-01-01\n'
                          'CLI7.8.9-beta.2 publication is pending.', {'CLI': '7.8.9-beta.2'}, 'guide')
        with self.assertRaises(ValueError):
            docs.check_readme('yetlet-cli', '7.8.9-beta.2', '## Historical\n' + cli_readme(), '=6.5.4-alpha.3')

    def test_sdk_readme_uses_selected_manifest_version(self):
        for name in ['yetlet', 'yetlet-browser']:
            docs.check_readme(name, '8.0.0-alpha.1', 'Version `8.0.0-alpha.1`; see release history.')
            with self.assertRaises(ValueError):
                docs.check_readme(name, '8.0.0-alpha.1', 'Version `7.0.0`.')

    def test_expected_binary_and_sdk_example_pins_cannot_remain_on_old_version(self):
        with self.assertRaises(ValueError):
            docs.check_readme('yetlet-cli', '7.8.9-beta.2',
                              cli_readme().replace('`yetlet 7.8.9-beta.2`', '`yetlet 1.0.0`'), '=6.5.4-alpha.3')
        snippet = 'Version `8.0.0`.\n```toml\n[dependencies]\nyetlet={version="=8.0.0"}\n```\n'
        docs.check_readme('yetlet', '8.0.0', snippet)
        with self.assertRaises(ValueError):
            docs.check_readme('yetlet', '8.0.0', snippet.replace('=8.0.0', '=7.0.0'))


    def test_current_checkout_and_explicit_pinned_edition(self):
        root = Path(__file__).resolve().parent.parent
        docs.check_source(root)
        # An accepted old guide pin remains separate from the development CLI.
        docs.check_installs('cargo install yetlet-cli --version 1.0.0 --locked', '1.0.0', 'pinned')
        with self.assertRaises(ValueError):
            docs.check_installs('cargo install yetlet-cli --version 1.0.0 --locked', '2.0.0', 'pinned')
        with tempfile.TemporaryDirectory() as temp, self.assertRaises(ValueError):
            docs.check_source(temp)

    def test_distribution_is_bound_to_accepted_records_not_development_versions(self):
        root = Path(__file__).resolve().parent.parent
        published = docs.published_versions(root)
        text = '\n'.join(f'**Published {subject}:** `{version}`.'
                         for subject, version in published.items())
        docs.check_distribution(text, published, 'status')
        for bad in [text.replace(published['SDK'], '0.1.0'),
                    text.replace(published['CLI'], '0.1.1'), 'Unpublished 0.1.0']:
            with self.subTest(text=bad), self.assertRaises(ValueError):
                docs.check_distribution(bad, published, 'status')
        # The record parser is not another list of selected-version literals.
        with tempfile.TemporaryDirectory() as temp:
            temp = Path(temp)
            for subject, (relative, consumer) in docs.PUBLICATION_RECORDS.items():
                path = temp / relative
                path.parent.mkdir(parents=True, exist_ok=True)
                prefix = 'yetlet-cli-v' if subject == 'CLI' else 'v'
                path.write_text(f'Historical preparation: unpublished 1.0.0\n\n'
                                f'## Owner-approved publication and {consumer} — 2030-01-01\n'
                                f'[Accepted](https://github.com/bohewu/yetlet/releases/tag/{prefix}7.8.9)\n')
            self.assertEqual(docs.published_versions(temp), {'SDK': '7.8.9', 'CLI': '7.8.9'})

    def test_guide_edition_prose_matches_site_manifest_but_history_keeps_old_pins(self):
        edition = {'sdk': '7.8.9-alpha.2', 'cli': '6.5.4'}
        text = ('The public reader/first-app edition remains pinned to\n'
                'SDKalpha.2/CLI6.5.4.\n\n## Historical preparation\n'
                'The public reader/first-app edition was SDKalpha.1/CLI1.0.0.')
        docs.check_edition(text, edition, 'compatibility')
        docs.check_edition(text.replace('SDKalpha.2/CLI6.5.4.',
                                        'SDK `7.8.9-alpha.2` / CLI `6.5.4`.'), edition, 'compatibility')
        for bad in [text.replace('SDKalpha.2', 'SDKalpha.1'),
                    text.replace('CLI6.5.4', 'CLI1.0.0'), 'No edition paragraph']:
            with self.subTest(text=bad), self.assertRaises(ValueError):
                docs.check_edition(bad, edition, 'compatibility')
        with self.assertRaises(ValueError):
            docs.check_edition(text, {'sdk': '7.8.9-alpha.3', 'cli': '6.5.4'}, 'compatibility')

    def test_current_module_claims_reject_old_packaging_and_preserve_history(self):
        for old in ['Experimental, unpublished.', 'One of sixteen library candidates.',
                    'public activation remains blocked']:
            with self.subTest(text=old), self.assertRaises(ValueError):
                docs.check_current_module(old, 'module')
            docs.check_current_module('Experimental SDK module.\n\n## Historical delivery\n' + old, 'module')
            docs.check_current_module('Experimental SDK module.\n\n<details>\n'
                                      '<summary>Historical delivery context and evidence</summary>\n' + old,
                                      'module')
        docs.check_current_module('Source-only Style generation is absent from published CLI.', 'module')

    def test_source_gate_catches_original_entry_drift_and_allows_source_only_additions(self):
        root = Path(__file__).resolve().parent.parent
        original = docs.read
        for relative, bad in [
                ('docs/ARCHITECTURE.md', 'Experimental, unpublished 0.1.0; nine library guides.'),
                ('docs/RELEASE-READINESS.md', 'Current distribution: SDKalpha.5'),
                ('AGENTS.md', 'Public release and production qualification remain open.'),
                ('docs/COMPATIBILITY.md', 'The public reader/first-app edition is SDKalpha.5/CLI0.1.1.'),
                ('docs/VIEW-COMPONENTS.md', 'Use feature/view-authoring-api for typed tags.'),
                ('crates/yetlet/guides/core.md', 'Experimental alpha.5, unpublished.')]:
            with (self.subTest(path=relative),
                  patch.object(docs, 'read', side_effect=lambda r, p: bad if p == relative else original(r, p)),
                  self.assertRaises(ValueError)):
                docs.check_source(root)
        # Current docs now include published typed tags and source-only Style,
        # as well as historical publication-pending sections.
        docs.check_source(root)

    def test_package_readme_links_are_publicly_resolvable_not_workspace_relative(self):
        valid = ('[Docs](https://docs.rs/yetlet) [Source](https://github.com/bohewu/yetlet)'
                 ' [Mail](mailto:maintainer@example.invalid) [Local](#features)')
        docs.check_public_readme_links(valid, 'README')
        for broken in ['[App](../../docs/FIRST-APP.md)',
                       '[Guide](guides/core.md)', '[Sibling](../yetlet-browser/README.md)',
                       '[Bad](javascript:alert)', '[Root](/docs/guide.md)']:
            with self.subTest(link=broken), self.assertRaises(ValueError):
                docs.check_public_readme_links(broken, 'README')
        with self.assertRaises(ValueError):
            docs.check_readme('yetlet', '8.0.0',
                              'Version '+chr(96)+'8.0.0'+chr(96)+'.\n[Guide](guides/core.md)\n')

    def test_packaged_guide_links_are_self_contained_or_public_urls(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            base = root / 'crates/yetlet'
            (base / 'guides').mkdir(parents=True)
            (base / 'README.md').write_text('# Package\n', encoding='utf-8')
            guide = base / 'guides/entry.md'
            guide.write_text('[Package](../README.md) '
                             '[Source](https://github.com/bohewu/yetlet)\n', encoding='utf-8')
            docs.check_guide_links(root, 'crates/yetlet/guides/entry.md', 'crates/yetlet')
            for broken in ['[Outside](../../../docs/README.md)',
                           '[Missing](missing.md)', '[Invalid](javascript:example)']:
                with self.subTest(link=broken), self.assertRaises(ValueError):
                    guide.write_text(broken, encoding='utf-8')
                    docs.check_guide_links(root, 'crates/yetlet/guides/entry.md',
                                           'crates/yetlet')


    def test_published_cli_inventory_excludes_later_source_only_style_templates(self):
        text = ('## Published CLI command and capability inventory\n'
                'new --template native|browser\n\n'
                '## Source-only application-owned Rust CSS\n'
                'native-style and browser-style are checkout-only.\n')
        docs.check_cli_published_commands(text, '2.1.0', '2.1.0')
        with self.assertRaises(ValueError):
            docs.check_cli_published_commands(
                text.replace('native|browser', 'native|browser|native-style'),
                '2.1.0', '2.1.0')
        with self.assertRaises(ValueError):
            docs.check_cli_published_commands(
                text.replace('native|browser', 'native|browser|browser-style'),
                '2.1.0', '2.1.0')
        # A later source version has a separate selected packaging decision.
        docs.check_cli_published_commands(text, '2.2.0', '2.1.0')


    def test_source_identity_invokes_gate_before_current_release_build(self):
        version = '8.0.0-alpha.1'
        record = {'schema': 2, 'version': version, 'source': 'a' * 40, 'cohort': 'consolidated'}
        env = {'GITHUB_EVENT_NAME': 'workflow_dispatch', 'GITHUB_REPOSITORY': 'bohewu/yetlet',
               'GITHUB_REPOSITORY_ID': '1409587171', 'GITHUB_REPOSITORY_OWNER_ID': '35250002',
               'GITHUB_REF': 'refs/tags/v' + version, 'GITHUB_SHA': record['source'],
               'RECORD_WORKFLOW_SHA': record['source'],
               'RECORD_WORKFLOW_REF': 'bohewu/yetlet/.github/workflows/release.yml@refs/tags/v' + version}
        with (patch.dict('os.environ', env, clear=True), patch.object(records, 'source_version'),
              patch.object(records.subprocess, 'check_output', side_effect=[record['source'], record['source'], '', '']),
              patch.object(docs, 'check_source', side_effect=ValueError('stale docs')) as gate):
            with self.assertRaises(ValueError):
                records.source_identity(record, Path('.'), 'release.yml')
            gate.assert_called_once_with(Path('.'), 'consolidated')

    def test_frozen_readme_is_checked_before_registry_envelope_or_network(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            record, subjects = archives(root, cli=True, readme_suffix='\nPublication is pending.\n')
            with (patch.object(registry, 'envelope') as envelope, patch.object(registry, 'request') as network):
                with self.assertRaises(ValueError):
                    registry.prepare(record, root, subjects)
                envelope.assert_not_called()
                network.assert_not_called()


if __name__ == '__main__':
    unittest.main()
