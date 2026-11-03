"""Focused tests for cache/evidence separation and non-destructive previews."""
import contextlib
import io
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

import dev_artifacts as artifacts


class DevArtifactsTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name).resolve()
        (self.root / 'Cargo.toml').write_text('[workspace]\n')

    def write(self, relative, content='original failure'):
        path = self.root / relative
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(content)
        return path

    def test_prepare_separates_roots_and_refuses_run_reuse(self):
        record = artifacts.prepare(self.root, 'yt233-fixture')
        self.assertNotIn(Path(record['evidence_dir']), Path(record['cargo_target_dir']).parents)
        self.assertTrue(artifacts.managed(Path(record['cargo_target_dir']), self.root))
        self.write('artifacts/verification/yt233-fixture/failed.log')
        with self.assertRaises(ValueError):
            artifacts.prepare(self.root, 'yt233-fixture')
        self.assertEqual(len(artifacts.inventory(self.root)['entries']), 2)

    def test_plan_preserves_failures_unknown_paths_and_all_bytes(self):
        record = artifacts.prepare(self.root, 'yt233-fixture')
        paths = [self.write('target/old-failed-run/failure.log'),
                 self.write('target/debug/failure.log'),
                 self.write('artifacts/verification/yt233-fixture/failed.log'),
                 self.write('.cache/yetlet/builds/yt233-fixture/debug/a', 'binary')]
        before = {str(path): path.read_bytes() for path in paths}
        report = artifacts.inventory(self.root)
        rows = {row['path']: row['kind'] for row in report['entries']}
        self.assertEqual(rows[record['cargo_target_dir']], 'rebuildable-preview')
        self.assertEqual(rows[str(self.root / 'target/debug')], 'legacy-review')
        self.assertEqual(rows[str(self.root / 'target/old-failed-run')], 'preserve')
        self.assertFalse(report['deletion_performed'])
        self.assertEqual(before, {str(path): path.read_bytes() for path in paths})

    def test_tampered_or_missing_evidence_marker_cannot_be_a_candidate(self):
        record = artifacts.prepare(self.root, 'test')
        self.write('artifacts/verification/test/layout.json', '{}')
        rows = artifacts.inventory(self.root)['entries']
        self.assertEqual(next(row['kind'] for row in rows
                              if row['path'] == record['cargo_target_dir']), 'preserve')

    def test_independent_workspace_targets_are_discovered_without_entering_outputs(self):
        self.write('examples/sample/Cargo.toml', '[workspace]')
        self.write('examples/sample/target/release/a', 'abc')
        self.write('target/old-source/Cargo.toml', '[workspace]')
        found, errors = artifacts.targets(self.root)
        self.assertEqual(set(found), {self.root / 'target', self.root / 'examples/sample/target'})
        self.assertFalse(errors)
        rows = artifacts.inventory(self.root)['entries']
        self.assertEqual(next(row['bytes'] for row in rows
                              if Path(row['path']) == self.root / 'examples/sample/target/release'), 3)

    @unittest.skipUnless(hasattr(Path, 'symlink_to'), 'symlink API unavailable')
    def test_symlinks_are_preserved_and_never_traversed(self):
        secret = self.write('elsewhere/retained.log')
        linked = self.root / 'target/debug'
        linked.parent.mkdir()
        try:
            linked.symlink_to(secret.parent, target_is_directory=True)
        except OSError:
            self.skipTest('symlink creation unavailable')
        row = artifacts.inventory(self.root)['entries'][0]
        self.assertEqual((row['kind'], row['bytes'], row['links_skipped']), ('preserve', 0, 1))
        with self.assertRaises(ValueError):
            artifacts.prepare(linked, 'test')

    def test_nested_link_or_unreadable_measurement_blocks_candidate(self):
        record = artifacts.prepare(self.root, 'test')
        with patch.object(artifacts, 'measure', return_value={
                'bytes': 3, 'files': 1, 'links_skipped': 0,
                'errors': [{'path': 'denied', 'error': 'PermissionError'}]}):
            rows = artifacts.inventory(self.root)['entries']
        self.assertEqual(next(row['kind'] for row in rows
                              if row['path'] == record['cargo_target_dir']), 'preserve')

    def test_explicit_roots_are_read_only_and_overlap_is_rejected(self):
        self.write('legacy/debug/a', 'abc')
        self.write('retained/failure.log')
        report = artifacts.inventory(self.root, [self.root / 'legacy'], [self.root / 'retained'])
        self.assertEqual(report['totals']['legacy-review'], 3)
        with self.assertRaises(ValueError):
            artifacts.inventory(self.root, [self.root / 'artifacts'])

    def test_cli_has_no_delete_option_and_plan_does_not_write(self):
        self.write('target/debug/original.log')
        with contextlib.redirect_stdout(io.StringIO()) as output:
            self.assertEqual(artifacts.main(['plan', '--root', str(self.root), '--json']), 0)
        self.assertFalse(json.loads(output.getvalue())['deletion_performed'])
        with contextlib.redirect_stderr(io.StringIO()), self.assertRaises(SystemExit):
            artifacts.main(['plan', '--apply'])

    def test_invalid_run_names_rejected_before_creation(self):
        for run in ('../target', '.', 'a/b', 'a\\b', 'A', 'a' * 81):
            with self.subTest(run=run), self.assertRaises(ValueError):
                artifacts.prepare(self.root, run)
        self.assertFalse((self.root / '.cache').exists())


if __name__ == '__main__':
    unittest.main()
