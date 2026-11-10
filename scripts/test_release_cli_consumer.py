"""Deterministic installed-consumer assertions; no registry or browser required."""
import json
import os
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

import release_cli_consumer as consumer


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

    def fixture(self, project, template='browser-style', requested=None, source='registry-refresh'):
        versions = ['0.9.0-alpha.6', '0.9.0-alpha.7']
        policy = json.dumps({'revision':'fixture-v1', 'templates':{template:versions}}).encode()
        names = ['yetlet', 'yetlet-browser'] if template.startswith('browser') else ['yetlet']
        cache = {'records': []}
        packages, locked, dependencies = [], [], []
        for name in names:
            body = '\n'.join(json.dumps({'name':name, 'vers':v, 'yanked':False, 'rust_version':'1.99.0',
                                         'cksum':'f'*64}) for v in versions)
            cache['records'].append({'name':name, 'body':body, 'sha256':consumer.sha(body.encode()), 'validated_at':100})
            packages.append({'name':name, 'checksum':'f'*64, 'metadata_sha256':consumer.sha(body.encode()), 'validated_at':100})
            locked.append(f'[[package]]\nname="{name}"\nversion="{versions[-1]}"\n'
                          'source="registry+https://github.com/rust-lang/crates.io-index"\nchecksum="' + 'f'*64 + '"\n')
            dependencies.append(f'{name}={{version="={versions[-1]}"}}\n')
        lock = ('version=4\n' + ''.join(locked)).encode()
        (project / 'Cargo.lock').write_bytes(lock)
        (project / 'Cargo.toml').write_text('[target.\'cfg(target_arch="wasm32")\'.dependencies]\n' + ''.join(dependencies))
        receipt = {'schema':1, 'cli_version':'0.3.0', 'cargo_lock_sha256':consumer.sha(lock),
                   'sdk':{'requested':requested, 'source':source, 'template':template, 'policy':'fixture-v1',
                          'policy_sha256':consumer.sha(policy), 'registry':'https://index.crates.io/',
                          'version':versions[-1], 'packages':packages}}
        (project / '.yetlet').mkdir()
        (project / '.yetlet/sdk-resolution.json').write_text(json.dumps(receipt))
        return cache, policy, receipt

    def test_default_and_offline_use_captured_metadata_not_cli_build_version(self):
        for source in ('registry-refresh', 'offline-cache'):
            with tempfile.TemporaryDirectory() as temp:
                root = Path(temp)
                cache, policy, receipt = self.fixture(root, source=source)
                self.assertEqual(consumer.inspect_project(root, '0.3.0', 'browser-style', None, source, cache, policy), receipt)

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
                    consumer.inspect_project(root, '0.3.0', 'browser-style', '0.9.0-alpha.7', 'registry-refresh', cache, policy)

    def test_stable_first_and_numeric_prerelease_order(self):
        self.assertGreater(consumer.version_key('0.8.0'), consumer.version_key('0.9.0-alpha.10'))
        self.assertGreater(consumer.version_key('0.9.0-alpha.10'), consumer.version_key('0.9.0-alpha.9'))


if __name__ == '__main__':
    unittest.main()
