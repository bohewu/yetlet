"""Deterministic installed-consumer assertions; no registry or browser required."""
import json
import os
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

import release_cli_consumer as consumer


HISTORICAL_POLICY = b'{\n  "revision": "sdk-compat-v1",\n  "msrv": "1.99",\n  "templates": {\n    "native": ["0.9.0-alpha.6", "0.9.0-alpha.7"],\n    "browser": ["0.9.0-alpha.6", "0.9.0-alpha.7"],\n    "native-style": ["0.9.0-alpha.7"],\n    "browser-style": ["0.9.0-alpha.7"]\n  }\n}\n'
LATEST_POLICY = b'{"revision":"sdk-latest-v2","selection":"latest-semver"}'


class ResolutionEvidenceTests(unittest.TestCase):
    def test_consumer_commands_use_private_umask_without_changing_parent(self):
        with tempfile.TemporaryDirectory() as temp, \
                patch.object(consumer.subprocess, 'Popen', side_effect=RuntimeError('stop at launch')) as launch:
            root = Path(temp)
            with self.assertRaisesRegex(RuntimeError, 'stop at launch'):
                consumer.qualify(root/'yetlet', '0.3.0', '0.9.0-alpha.7', root,
                                 root/'evidence', {'PATH':'/usr/bin:/bin'}, b'{}', 'crates.io')
            self.assertEqual(launch.call_args.kwargs['umask'], 0o077)

    def test_owned_rust_copy_removes_writable_modes_without_changing_source(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            source = root / 'installed'
            (source / 'bin').mkdir(parents=True)
            (source / 'lib').mkdir()
            for name in ['rustc', 'cargo']:
                (source / 'bin' / name).write_bytes(b'fixture')
                (source / 'bin' / name).chmod(0o775)
            (source / 'lib' / 'runtime').write_bytes(b'library')
            source.chmod(0o775)
            with patch.object(consumer.subprocess, 'check_output',
                              return_value=str(source / 'bin/rustc').encode()):
                env = consumer.protected_rust_env(root, {'PATH':'/usr/bin:/bin'})
            self.assertEqual(env['PATH'].split(os.pathsep)[0], str(root / 'rust/bin'))
            self.assertEqual(env['RUSTUP_TOOLCHAIN'], str(root / 'rust'))
            self.assertEqual((root / 'rust/lib/runtime').read_bytes(), b'library')
            self.assertEqual(source.stat().st_mode & 0o777, 0o775)
            self.assertEqual((source / 'bin/rustc').stat().st_mode & 0o777, 0o775)
            self.assertTrue(all(p.stat().st_mode & 0o022 == 0 for p in
                                [root / 'rust', *(root / 'rust').rglob('*')]))

    def test_chrome_uses_consumer_environment_and_path_without_publisher_credentials(self):
        safe = {'PATH':'/usr/bin:/bin', 'HOME':'/tmp/consumer-home'}
        with tempfile.TemporaryDirectory() as temp, \
                patch.dict(os.environ, {'GH_TOKEN':'synthetic-owner', 'CARGO_REGISTRY_TOKEN':'synthetic-publisher'}), \
                patch.object(consumer.shutil, 'which', return_value='/usr/bin/google-chrome') as which, \
                patch.object(consumer.subprocess, 'Popen', side_effect=RuntimeError('stop at launch')) as launch:
            with self.assertRaisesRegex(RuntimeError, 'stop at launch'):
                consumer.chrome_smoke('http://127.0.0.1/', Path(temp), Path(temp), safe)
            self.assertEqual(launch.call_args.kwargs['env'], safe)
            self.assertNotIn('GH_TOKEN', launch.call_args.kwargs['env'])
            self.assertNotIn('CARGO_REGISTRY_TOKEN', launch.call_args.kwargs['env'])
            which.assert_called_once_with('google-chrome-stable', path=safe['PATH'])

    def fixture(self, project, template='browser-style', requested=None, source='registry-refresh',
                versions=None, selected=None, historical=False, mutate=None):
        versions = versions or ['0.9.0-alpha.6', '0.9.0-alpha.7']
        selected = selected or versions[-1]
        policy = HISTORICAL_POLICY if historical else LATEST_POLICY
        cli_version = '0.3.0' if historical else '0.3.1'
        names = ['yetlet', 'yetlet-browser'] if template.startswith('browser') else ['yetlet']
        cache = {'records': []}
        packages, locked, dependencies = [], [], []
        for name in names:
            rows = [{'name':name, 'vers':v, 'yanked':False, 'rust_version':'1.99.0', 'cksum':'f'*64,
                     'features':{f:[] for f in ('server-axum', 'config', 'reactive', 'dom', 'dom-keyed')},
                     'deps':[{'name':'yetlet', 'req':'='+v}] if name == 'yetlet-browser' else []}
                    for v in versions]
            if mutate:
                mutate(name, rows)
            body = '\n'.join(json.dumps(row) for row in rows)
            cache['records'].append({'name':name, 'body':body, 'sha256':consumer.sha(body.encode()), 'validated_at':100})
            packages.append({'name':name, 'checksum':'f'*64, 'metadata_sha256':consumer.sha(body.encode()), 'validated_at':100})
            locked.append(f'[[package]]\nname="{name}"\nversion="{selected}"\n'
                          'source="registry+https://github.com/rust-lang/crates.io-index"\nchecksum="' + 'f'*64 + '"\n'
                          + ('dependencies=["yetlet"]\n' if name == 'yetlet-browser' else ''))
            dependencies.append(f'{name}={{version="={selected}"}}\n')
        lock = ('version=4\n' + ''.join(locked)).encode()
        (project / 'Cargo.lock').write_bytes(lock)
        (project / 'Cargo.toml').write_text('[target.\'cfg(target_arch="wasm32")\'.dependencies]\n' + ''.join(dependencies))
        receipt = {'schema':1, 'cli_version':cli_version, 'cargo_lock_sha256':consumer.sha(lock),
                   'sdk':{'requested':requested, 'source':source, 'template':template, 'policy':json.loads(policy)['revision'],
                          'policy_sha256':consumer.sha(policy), 'registry':'https://index.crates.io/',
                          'version':selected, 'packages':packages}}
        (project / '.yetlet').mkdir()
        (project / '.yetlet/sdk-resolution.json').write_text(json.dumps(receipt))
        return cache, policy, receipt

    def test_default_and_offline_use_captured_metadata_not_cli_build_version(self):
        for source in ('registry-refresh', 'offline-cache'):
            with tempfile.TemporaryDirectory() as temp:
                root = Path(temp)
                cache, policy, receipt = self.fixture(root, source=source)
                self.assertEqual(consumer.inspect_project(root, '0.3.1', 'browser-style', None, source, cache, policy), receipt)

    def test_provenance_lock_manifest_pair_and_yank_mismatches_are_rejected(self):
        for defect in ('lock', 'pin', 'checksum', 'metadata', 'yanked', 'pair', 'source', 'policy'):
            with self.subTest(defect=defect), tempfile.TemporaryDirectory() as temp:
                root = Path(temp)
                cache, policy, receipt = self.fixture(root, requested='0.9.0-alpha.7')
                if defect == 'lock': (root / 'Cargo.lock').write_bytes(b'changed')
                if defect == 'pin': (root / 'Cargo.toml').write_text('[dependencies]\nyetlet={version="^0.9"}\n')
                if defect == 'checksum': receipt['sdk']['packages'][0]['checksum'] = 'a'*64
                if defect == 'metadata': cache['records'][0]['body'] += '\n'
                if defect == 'yanked': cache['records'][0]['body'] = cache['records'][0]['body'].replace('false', 'true')
                if defect == 'pair': cache['records'].pop()
                if defect == 'source': receipt['sdk']['source'] = 'offline-cache'
                if defect == 'policy': policy += b'\n'
                (root / '.yetlet/sdk-resolution.json').write_text(json.dumps(receipt))
                with self.assertRaises((ValueError, KeyError)):
                    consumer.inspect_project(root, '0.3.1', 'browser-style', '0.9.0-alpha.7', 'registry-refresh', cache, policy)

    def test_semver_precedence_and_canonical_exact_versions(self):
        ordered = ['0.8.0', '0.9.0-alpha.9', '0.9.0-alpha.10', '0.9.0-beta',
                   '0.9.0-rc.1', '0.9.0', '1.0.0-0', '1.0.0-alpha', '1.0.0-alpha.1', '1.0.0']
        self.assertEqual(sorted(reversed(ordered), key=consumer.version_key), ordered)
        for invalid in ('01.0.0', '1.0.0-alpha.01', '1.0', '1.0.0+build', '>=1.0.0'):
            with self.subTest(version=invalid), self.assertRaises(ValueError):
                consumer.version_key(invalid)

    def test_future_versions_use_unchanged_policy_without_msrv_ceiling(self):
        for latest in ('0.9.0-alpha.10', '2.0.0-rc.10', '12.0.0'):
            for source in ('registry-refresh', 'offline-cache'):
                with self.subTest(version=latest, source=source), tempfile.TemporaryDirectory() as temp:
                    root = Path(temp)
                    def future_metadata(name, rows):
                        for row in rows:
                            row['rust_version'] = None if name == 'yetlet' else '999.0'
                    cache, policy, receipt = self.fixture(root, source=source,
                        versions=['0.8.0', '0.9.0-alpha.7', latest], mutate=future_metadata)
                    self.assertEqual(policy, LATEST_POLICY)
                    self.assertEqual(consumer.inspect_project(root, '0.3.1', 'browser-style', None,
                                     source, cache, policy), receipt)

    def test_availability_yanks_exact_override_and_newest_feature_failure(self):
        versions = ['0.9.0-alpha.7', '2.0.0-rc.10']
        for defect in ('yanked', 'pair', 'feature', 'dependency', 'explicit', 'fallback'):
            with self.subTest(defect=defect), tempfile.TemporaryDirectory() as temp:
                root = Path(temp)
                def mutate(name, rows):
                    if defect == 'yanked': rows[-1]['yanked'] = True
                    if defect == 'pair' and name == 'yetlet-browser': rows.pop()
                    if defect in ('feature', 'fallback') and name == 'yetlet':
                        rows[-1]['features'].pop('reactive')
                    if defect == 'dependency' and name == 'yetlet-browser': rows[-1]['deps'] = []
                selected = versions[0] if defect in ('yanked', 'pair', 'explicit', 'fallback') else versions[-1]
                requested = selected if defect == 'explicit' else None
                cache, policy, receipt = self.fixture(root, versions=versions, selected=selected,
                                                      requested=requested, mutate=mutate)
                if defect in ('feature', 'dependency', 'fallback'):
                    with self.assertRaises(ValueError):
                        consumer.inspect_project(root, '0.3.1', 'browser-style', requested,
                                                 'registry-refresh', cache, policy)
                else:
                    self.assertEqual(consumer.inspect_project(root, '0.3.1', 'browser-style', requested,
                                     'registry-refresh', cache, policy), receipt)

    def test_feature_aliases_and_cargo_browser_edge(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            def aliases(name, rows):
                for row in rows:
                    if name == 'yetlet':
                        row['features'].pop('reactive')
                        row['features2'] = {'reactive':['dep:signals', 'signals?/runtime']}
                        row['deps'] = [{'name':'signals', 'req':'1', 'optional':True}]
            cache, policy, receipt = self.fixture(root, mutate=aliases)
            consumer.inspect_project(root, '0.3.1', 'browser-style', None, 'registry-refresh', cache, policy)
            lock = (root / 'Cargo.lock').read_bytes().replace(b'dependencies=["yetlet"]', b'dependencies=[]')
            (root / 'Cargo.lock').write_bytes(lock)
            receipt['cargo_lock_sha256'] = consumer.sha(lock)
            (root / '.yetlet/sdk-resolution.json').write_text(json.dumps(receipt))
            with self.assertRaisesRegex(ValueError, 'browser lock dependency'):
                consumer.inspect_project(root, '0.3.1', 'browser-style', None, 'registry-refresh', cache, policy)

    def test_historical_policy_keeps_finite_sets_msrv_and_exact_hash(self):
        self.assertEqual(consumer.sha(HISTORICAL_POLICY),
                         '8bddcb15647fd21a755df38f38b6308cad6e8e4f29c4838849923122aefdbbc7')
        for defect in (None, 'future', 'msrv', 'policy', 'cli'):
            with self.subTest(defect=defect), tempfile.TemporaryDirectory() as temp:
                root = Path(temp)
                def metadata(name, rows):
                    if defect == 'msrv': rows[1]['rust_version'] = '999.0'
                selected = '2.0.0' if defect == 'future' else '0.9.0-alpha.7'
                cache, policy, receipt = self.fixture(root, historical=True, selected=selected,
                    versions=['0.9.0-alpha.6', '0.9.0-alpha.7', '2.0.0'], mutate=metadata)
                if defect == 'policy':
                    policy = policy.replace(b'alpha.7', b'alpha.8')
                    receipt['sdk']['policy_sha256'] = consumer.sha(policy)
                cli_version = '0.3.1' if defect == 'cli' else '0.3.0'
                receipt['cli_version'] = cli_version
                (root / '.yetlet/sdk-resolution.json').write_text(json.dumps(receipt))
                if defect:
                    with self.assertRaises(ValueError):
                        consumer.inspect_project(root, cli_version, 'browser-style', None,
                                                 'registry-refresh', cache, policy)
                else:
                    self.assertEqual(consumer.inspect_project(root, cli_version, 'browser-style', None,
                                     'registry-refresh', cache, policy), receipt)


if __name__ == '__main__':
    unittest.main()
