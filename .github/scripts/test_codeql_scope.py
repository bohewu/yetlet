"""Offline regression cases for the CodeQL input boundary, not query results."""
import os
from pathlib import Path
import subprocess
import tempfile
from types import SimpleNamespace
import unittest
from unittest.mock import patch

import codeql_scope as scope


class LanguageInputs(unittest.TestCase):
    def test_root_python_and_its_dependency_tool_settings(self):
        for path in ('scripts/release_record.py', 'scripts/helpers.pyi', 'pyproject.toml',
                     'requirements-dev.txt', 'requirements.in', 'Pipfile.lock', 'uv.lock', '.python-version'):
            with self.subTest(path=path):
                self.assertEqual(scope.languages_for([path]), ['python'])

    def test_actions_and_javascript_data_and_dependency_inputs(self):
        for path in ('site/app.ts', 'site/index.html', 'package-lock.json', 'tsconfig.json',
                     'pnpm-lock.yaml', 'yarn.lock', '.node-version', 'vite.config.mjs', 'site/app.css'):
            with self.subTest(path=path):
                self.assertEqual(scope.languages_for([path]), ['javascript-typescript'])
        for path in ('.github/workflows/release.yml', 'tools/composite/action.yaml'):
            self.assertEqual(scope.languages_for([path]), ['actions', 'javascript-typescript'])

    def test_rust_includes_tests_manifests_locks_and_cargo_tool_settings(self):
        for path in ('tests/fixture.rs', 'vendor/lib/src/lib.rs', 'examples/app/build.rs', 'Cargo.toml',
                     'tools/yetlet-cli/Cargo.lock', 'rust-toolchain.toml', 'app/.cargo/config.toml'):
            self.assertEqual(scope.languages_for([path]), ['rust'])

    def test_package_assets_and_embedded_docs_are_rust_inputs(self):
        roots = {'tools/yetlet-cli/'}
        for path in ('tools/yetlet-cli/templates/styles/README.md', 'tools/yetlet-cli/resources/framework.json',
                     'tools/yetlet-cli/templates/browser/app.ts', 'tools/yetlet-cli/fixtures/input.py'):
            self.assertIn('rust', scope.languages_for([path], roots))
        self.assertEqual(scope.languages_for(['shared/included.md'], embedded={'shared/included.md'}), ['rust'])
        self.assertEqual(scope.languages_for(['docs/ROADMAP.md']), [])

    def test_policy_query_discovery_unknown_inputs_and_empty_revisions_fail_open(self):
        for path in ('.github/workflows/codeql.yml', '.github/scripts/codeql_scope.py', '.gitignore',
                     '.gitmodules', '.gitattributes', '.github/codeql/config.yml', 'models/qlpack.yml',
                     'queries/security.ql', 'queries/lib.qll', 'queries/default.qls', 'build-helper.sh', 'unknown.data'):
            self.assertEqual(scope.languages_for([path]), list(scope.LANGUAGES))
        for event, base, head in [('schedule', '', ''), ('workflow_dispatch', '', ''),
                                  ('push', '0'*40, 'b'*40), ('pull_request', 'missing', 'b'*40)]:
            self.assertEqual(scope.select(event, base, head)[0], list(scope.LANGUAGES))
        with patch.object(scope, 'git', return_value=b''):
            self.assertEqual(scope.select('push', 'a'*40, 'b'*40)[0], list(scope.LANGUAGES))
        with patch.object(scope, 'git', side_effect=subprocess.TimeoutExpired('git', 20)):
            self.assertEqual(scope.select('push', 'a'*40, 'b'*40)[0], list(scope.LANGUAGES))

    def test_complete_no_rename_diff_uses_base_and_head_input_sets(self):
        before = ({'deleted-package/'}, {'deleted-included.md'}, False, set())
        after = ({'new-package/'}, {'new-included.md'}, False, set())
        with patch.object(scope, 'git', return_value=b'deleted-included.md\0') as git, \
                patch.object(scope, 'rust_inputs', side_effect=[before, after]):
            self.assertEqual(scope.select('pull_request', 'a'*40, 'b'*40)[0], ['rust'])
            self.assertIn('--no-renames', git.call_args.args)
            self.assertIn('-z', git.call_args.args)
        self.assertEqual(scope.languages_for(['scripts/tool.py'], uncertain_rust=True), ['python', 'rust'])

    def test_tree_reads_literal_includes_and_refuses_unreviewed_build_scripts(self):
        ref = 'a'*40
        files = {'Cargo.toml':b'[workspace]\nmembers=["app"]\n',
                 'app/Cargo.toml':b'[package]\nname="app"\nversion="0.1.0"\n',
                 'app/src/lib.rs':b'const DATA: &str = include_str ! ("../../shared/data.md");',
                 'shared/data.md':b'Fixture'}
        def git(*args):
            if args[0] == 'ls-tree':
                return b''.join(b'100644 blob ' + b'b'*40 + b'\t' + path.encode() + b'\0' for path in files)
            return files[args[1].removeprefix(ref + ':')]
        grep = SimpleNamespace(returncode=0, stdout=(ref + ':app/src/lib.rs\0').encode())
        with patch.object(scope, 'git', side_effect=git), patch.object(scope.subprocess, 'run', return_value=grep):
            roots, embedded, uncertain, query_roots = scope.rust_inputs(ref)
            self.assertEqual(roots, {'app/'})
            self.assertIn('shared/data.md', embedded)
            self.assertFalse(uncertain)
            files['app/build.rs'] = b'fn main() {}'
            self.assertTrue(scope.rust_inputs(ref)[2])
            files['app/Cargo.toml'] += b'build="custom-build.rs"\n'
            self.assertTrue(scope.rust_inputs(ref)[2])
            files['security/models/qlpack.yml'] = b'name: fixture/models\n'
            self.assertEqual(scope.rust_inputs(ref)[3], {'security/models/'})

    def test_query_model_data_keeps_all_languages_regardless_of_data_extension(self):
        inputs = (set(), set(), False, {'security/models/'})
        with patch.object(scope, 'git', return_value=b'security/models/data/custom.yml\0'), \
                patch.object(scope, 'rust_inputs', return_value=inputs):
            self.assertEqual(scope.select('push', 'a'*40, 'b'*40)[0], list(scope.LANGUAGES))

    def test_nonliteral_include_without_reviewed_generator_keeps_rust(self):
        ref = 'a'*40
        files = {'app/Cargo.toml':b'[package]\nname="app"\nversion="0.1.0"\n',
                 'app/src/lib.rs':b'include!(concat!(env!("OUT_DIR"), "/generated.rs"));'}
        def git(*args):
            if args[0] == 'ls-tree':
                return b''.join(b'100644 blob ' + b'b'*40 + b'\t' + p.encode() + b'\0' for p in files)
            return files[args[1].removeprefix(ref + ':')]
        grep = SimpleNamespace(returncode=0, stdout=(ref + ':app/src/lib.rs\0').encode())
        with patch.object(scope, 'git', side_effect=git), patch.object(scope.subprocess, 'run', return_value=grep):
            self.assertTrue(scope.rust_inputs(ref)[2])

    def test_macro_delimiters_newlines_and_unparsed_comments_keep_external_input_coverage(self):
        ref = 'a'*40
        files = {'app/Cargo.toml':b'[package]\nname="app"\nversion="0.1.0"\n',
                 'app/src/lib.rs':b'', 'shared/data.md':b'Fixture'}
        def git(*args):
            if args[0] == 'ls-tree':
                return b''.join(b'100644 blob ' + b'b'*40 + b'\t' + p.encode() + b'\0' for p in files)
            return files[args[1].removeprefix(ref + ':')]
        forms = [b'include_str! { "../../shared/data.md" }', b'include_str!["../../shared/data.md"]',
                 b'include_str\n!("../../shared/data.md")', b'include_bytes!{r#"../../shared/data.md"#}',
                 b'include_str /* comment */ !("../../shared/data.md")',
                 b'include_str /* outer /* nested */ comment */ !("../../shared/data.md")',
                 b'include_str! /* comment */ ("../../shared/data.md")']
        real_run = subprocess.run
        with tempfile.TemporaryDirectory() as temporary, patch.object(scope, 'git', side_effect=git):
            root = Path(temporary)
            source_file = root / 'app/src/lib.rs'
            source_file.parent.mkdir(parents=True)
            def grep(args, **kwargs):
                # Exercise the real prefilter, including split-line invocations,
                # without creating fixture commits or using repository history.
                return real_run(['git', '-C', temporary, 'grep', '--no-index',
                                 *args[2:args.index(ref)], '--', '*.rs'], **kwargs)
            for source in forms:
                with self.subTest(source=source):
                    files['app/src/lib.rs'] = source
                    source_file.write_bytes(source)
                    with patch.object(scope.subprocess, 'run', side_effect=grep):
                        roots, embedded, uncertain, _ = scope.rust_inputs(ref)
                    self.assertIn('rust', scope.languages_for(['shared/data.md'], roots, embedded, uncertain))
            files['app/src/lib.rs'] = b'// Other structs may include\n// fields not used here.\npub struct Data;'
            source_file.write_bytes(files['app/src/lib.rs'])
            with patch.object(scope.subprocess, 'run', side_effect=grep):
                self.assertFalse(scope.rust_inputs(ref)[2])

    def test_scope_summary_never_claims_unselected_analysis_succeeded(self):
        with tempfile.TemporaryDirectory() as temp:
            output, summary = Path(temp)/'output', Path(temp)/'summary'
            with patch.dict(os.environ, {'GITHUB_OUTPUT':str(output), 'GITHUB_STEP_SUMMARY':str(summary)}), \
                    patch.object(scope, 'select', return_value=(['python'], 'fixture complete inputs')):
                scope.main()
            self.assertEqual(output.read_text(), 'languages=["python"]\n')
            self.assertIn('No database/SARIF reuse', summary.read_text())


if __name__ == '__main__':
    unittest.main()
