#!/usr/bin/env python3
"""YT-209 checks: synthetic fixtures and loopback CLI; no keys, external network or signing."""
import base64
import copy
import http.server
import importlib.util
import io
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import threading
import unittest
from unittest.mock import patch
import urllib.error

import release_record as records

HERE = Path(__file__).resolve().parent
ROOT = HERE.parent


def module(name, filename):
    spec = importlib.util.spec_from_file_location(name, HERE / filename)
    m = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(m)
    return m


v = module('result_tests', 'release-result-verify.py')
pub = module('public_tests', 'release-public.py')
s = v.s


def record():
    return {'schema': 1, 'version': '1.2.3-alpha.4', 'source': 'c' * 40, 'cohort': 'sixteen',
        'build': {'run': '11', 'attempt': '2', 'artifact': '12', 'sha256': 'b' * 64},
        'custody_verifier': {'run': '13', 'attempt': '3', 'environment_id': '14'},
        'signer_commit': 'a' * 40, 'provenance': {'sha256': 'e' * 64, 'size': 123}, 'release_id': '15'}


def fixture(directory, consolidated=False):
    r = record()
    if consolidated: r.update(schema=2, cohort="consolidated")
    subjects = []
    for name in sorted(records.packages(r)):
        name += '-' + r['version'] + '.crate'
        raw = ('SYNTHETIC ' + name).encode()
        (directory / name).write_bytes(raw)
        subjects.append({'name': name, 'digest': {'sha256': s.sha(raw)}})
    statement = {'_type': 'https://in-toto.io/Statement/v1', 'predicateType': 'https://slsa.dev/provenance/v1',
        'subject': subjects, 'predicate': {'buildDefinition': {'externalParameters': {'cohort': r['cohort'],
        'workflow': {'repository': 'https://github.com/bohewu/yetlet', 'path': '.github/workflows/release.yml',
        'ref': 'refs/tags/v' + r['version']}}, 'resolvedDependencies': [
        {'uri': 'git+https://github.com/bohewu/yetlet', 'digest': {'gitCommit': r['source']}}]},
        'runDetails': {'metadata': {'invocationId': 'https://github.com/bohewu/yetlet/actions/runs/11/attempts/2'}}}}
    raw = records.canonical(statement)
    (directory / 'PROVENANCE-UNSIGNED.json').write_bytes(raw)
    r['provenance'] = {'sha256': s.sha(raw), 'size': len(raw)}
    return r


def sign_request(r):
    return {'mode': 'sign', 'signer_commit': r['signer_commit'], 'run': '21', 'attempt': '4', 'artifact': '22',
        'digest': 'd' * 64, 'receipt_sha256': 'e' * 64, 'provenance_sha256': r['provenance']['sha256'],
        'manifest_sha256': 'f' * 64, 'd6': 'approve-exact-signing-inputs'}


def result_request(r):
    return {'verifier_commit': 'd' * 40, 'signing_request': sign_request(r), 'sign_run': '31',
        'sign_attempt': '5', 'sign_artifact': '32', 'sign_digest': '1' * 64, 'execution_sha256': '2' * 64}


def environment(name):
    return {'name': name, 'id': 44, 'can_admins_bypass': False, 'protection_rules': [
        {'type': 'required_reviewers', 'prevent_self_review': False, 'reviewers': [
        {'type': 'User', 'reviewer': {'login': 'bohewu', 'id': 35250002}}]}],
        'deployment_branch_policy': {'protected_branches': False, 'custom_branch_policies': True}}


def policy(name='master', kind='branch'):
    return {'total_count': 1, 'branch_policies': [{'name': name, 'type': kind}]}


class RecordTests(unittest.TestCase):
    def test_closed_schema_and_two_versions_without_alpha_fallback(self):
        for version in ('1.2.3-alpha.4', '2.0.0', '0.0.0-0.A-b'):
            r = record(); r['version'] = version
            self.assertEqual(records.validate(r, 'complete'), r)
        for value in ('01.2.3', '1.2', '1.2.3-01', '1.2.3-alpha..4', '1.2.3+metadata', '../bad', '1.2.3\n'):
            r = record(); r['version'] = value
            with self.subTest(value=value), self.assertRaises(ValueError): records.validate(r)
        for key, value in [('schema', True), ('source', 'C' * 40), ('cohort', 'nine'),
                           ('script', 'arbitrary'), ('release_id', 15), ('signer_commit', 'c' * 40),
                           ('build', {'run': '01', 'attempt': '1', 'artifact': '2', 'sha256': 'e' * 64})]:
            r = record(); r[key] = value
            with self.subTest(key=key), self.assertRaises(ValueError): records.validate(r, 'complete')
        for raw in ('{"schema":1,"schema":1}', '{"value":NaN}', ' ' * 16385):
            with self.assertRaises(ValueError): records.loads(raw)

    def test_two_package_cohort_is_closed_and_cannot_use_legacy_names(self):
        r = record(); r.update(schema=2, cohort='consolidated')
        self.assertEqual(records.packages(r), {'yetlet', 'yetlet-browser'})
        s.bind_record(r); self.assertEqual(s.PACKAGES, records.CONSOLIDATED)
        for schema, cohort in ((1, 'consolidated'), (2, 'sixteen'), (3, 'consolidated')):
            bad = dict(r, schema=schema, cohort=cohort)
            with self.assertRaises(ValueError): records.validate(bad)
        with self.assertRaises(ValueError): records.validate(dict(r, packages=['arbitrary']))
        s.bind_record(record()); self.assertEqual(s.PACKAGES, records.PACKAGES)

    def test_signer_provenance_and_public_names_follow_validated_cohort(self):
        for consolidated in (False, True):
            with tempfile.TemporaryDirectory() as temp:
                directory = Path(temp); r = fixture(directory, consolidated)
                pub.s.bind_record(r)
                self.assertEqual(pub.s.provenance(directory), r['provenance']['sha256'])
                self.assertEqual(len(pub.names()), len(records.packages(r)) + 4)
                statement = records.loads((directory / 'PROVENANCE-UNSIGNED.json').read_bytes())
                statement['subject'].append(statement['subject'][0])
                raw = records.canonical(statement); (directory / 'PROVENANCE-UNSIGNED.json').write_bytes(raw)
                r['provenance'] = {'sha256': s.sha(raw), 'size': len(raw)}
                pub.s.bind_record(r)
                with self.assertRaises(ValueError): pub.s.provenance(directory)
        pub.s.bind_record(record())

    def test_stage_requirements_and_canonical_digest(self):
        r = record(); core = {k: r[k] for k in records.CORE}
        records.validate(core)
        with self.assertRaises(ValueError): records.validate(core, 'custody')
        custody = dict(core, build=r['build']); records.validate(custody, 'custody')
        with self.assertRaises(ValueError): records.validate(custody, 'complete')
        self.assertEqual(records.digest(r), records.digest(dict(reversed(list(r.items())))))
        other = copy.deepcopy(r); other['build']['attempt'] = '9'
        self.assertNotEqual(records.digest(r), records.digest(other))
        with patch.dict(os.environ, {}, clear=True), self.assertRaises(KeyError): records.environment()

    def test_source_version_reads_actual_consolidated_manifests_and_rejects_mismatch(self):
        import tomllib
        r = record(); r.update(schema=2, cohort='consolidated'); r['version'] = tomllib.loads((ROOT / 'Cargo.toml').read_text())['workspace']['package']['version']
        records.source_version(r, ROOT)
        r['version'] = '1.2.3'
        with self.assertRaises(ValueError): records.source_version(r, ROOT)

    def test_source_identity_requires_tag_workflow_checkout_ancestry_and_clean(self):
        r = record(); tag = 'refs/tags/v' + r['version']
        env = {'GITHUB_EVENT_NAME': 'workflow_dispatch', 'GITHUB_REPOSITORY': 'bohewu/yetlet',
            'GITHUB_REPOSITORY_ID': '1409587171', 'GITHUB_REPOSITORY_OWNER_ID': '35250002',
            'GITHUB_REF': tag, 'GITHUB_SHA': r['source'], 'RECORD_WORKFLOW_SHA': r['source'],
            'RECORD_WORKFLOW_REF': 'bohewu/yetlet/.github/workflows/release.yml@' + tag}
        with patch.dict(os.environ, env, clear=True), patch.object(records, 'source_version') as versions, \
                patch.object(records.subprocess, 'check_output', side_effect=[r['source'], r['source'], '', '']) as git:
            records.source_identity(r, ROOT, 'release.yml')
            self.assertIn(tag + '^{commit}', git.call_args_list[1].args[0]); versions.assert_called_once()
        for key in ('GITHUB_SHA', 'RECORD_WORKFLOW_SHA', 'GITHUB_REF', 'GITHUB_REPOSITORY_ID'):
            bad = dict(env); bad[key] = 'foreign'
            with patch.dict(os.environ, bad, clear=True), patch.object(records.subprocess, 'check_output') as git:
                with self.assertRaises(ValueError): records.source_identity(r, ROOT, 'release.yml')
                git.assert_not_called()

    def test_record_binds_signer_and_nondefault_attempts(self):
        r = record(); s.bind_record(r)
        self.assertEqual((s.C, s.TAG, s.BUILD_ATTEMPT, s.VERIFIER_ATTEMPT),
                         ('c' * 40, 'v1.2.3-alpha.4', '2', '3'))
        good = sign_request(r); s.validate_request(good)
        for key, value in [('signer_commit', 'b' * 40), ('provenance_sha256', '0' * 64), ('d6', 'yes')]:
            bad = dict(good); bad[key] = value
            with self.assertRaises(ValueError): s.validate_request(bad)

    def test_record_check_cli_is_local_data_only(self):
        result = subprocess.run([sys.executable, str(HERE / 'release_record.py'), 'check', 'complete'],
            env={'RELEASE_RECORD': json.dumps(record())}, text=True, capture_output=True)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn(records.digest(record()), result.stdout)

    def test_prepare_uses_exact_record_attempts_and_original_custody_gate(self):
        import time
        r = record(); s.bind_record(r)
        raw = '-----BEGIN AGE ENCRYPTED FILE-----\nTE9DQUw=\n-----END AGE ENCRYPTED FILE-----\n'
        request = {'mode': 'prepare', 'signer_commit': r['signer_commit'], 'qualification_asset': '99',
            'qualification_sha256': 'a' * 64, 'qualification_manifest_sha256': 'f' * 64,
            'qualification_handoff': raw, 'qualification_handoff_sha256': s.sha(raw.encode()),
            'qualification_expires': str(int(time.time()) + 2400)}
        def run(ident, attempt, path):
            return {'id': int(ident), 'run_attempt': int(attempt), 'head_sha': s.C, 'head_branch': s.TAG,
                'repository': {'id': 1409587171, 'full_name': 'bohewu/yetlet'},
                'head_repository': {'id': 1409587171, 'full_name': 'bohewu/yetlet'}, 'event': 'workflow_dispatch',
                'path': path, 'status': 'completed', 'conclusion': 'success'}
        env = environment('release-evidence-verify'); env['id'] = 14
        data = {'/branches/master': {'protected': True, 'commit': {'sha': r['signer_commit']}},
            '/environments/release-sign': environment('release-sign'),
            '/environments/release-sign/deployment-branch-policies?per_page=100': policy(),
            '/actions/runs/11/attempts/2': run('11', '2', '.github/workflows/release.yml'),
            '/actions/runs/13/attempts/3': run('13', '3', '.github/workflows/evidence-verify.yml'),
            '/actions/artifacts/12': {'id': 12, 'name': 'encrypted-private-' + s.C + '-11-2', 'expired': False,
                'size_in_bytes': 1, 'digest': 'sha256:' + r['build']['sha256'],
                'workflow_run': {'id': 11, 'head_sha': s.C, 'repository_id': 1409587171, 'head_repository_id': 1409587171}},
            '/environments/release-evidence-verify': env,
            '/environments/release-evidence-verify/deployment-branch-policies?per_page=100': policy(s.TAG, 'tag'),
            '/actions/runs/13/approvals': [{'state': 'approved', 'user': {'id': 35250002}, 'environments': [{'id': 14}]}]}
        with patch.object(s, 'identity'), patch.object(s.api, 'api', side_effect=lambda p: data[p]), \
                patch.object(s, 'command') as command:
            s.gate(request)
        command.assert_not_called()


class ResultTests(unittest.TestCase):
    def setUp(self):
        self.r = record(); s.bind_record(self.r); self.q = result_request(self.r)

    def metadata(self):
        q = self.q; sr = q['signing_request']
        responses = {'/branches/master': {'protected': True, 'commit': {'sha': q['verifier_commit']}},
            '/environments/' + v.ENVIRONMENT: environment(v.ENVIRONMENT),
            '/environments/' + v.ENVIRONMENT + '/deployment-branch-policies?per_page=100': policy()}
        for run, attempt, artifact, digest, prefix in ((sr['run'], sr['attempt'], sr['artifact'], sr['digest'], 'signing-inputs-'),
                (q['sign_run'], q['sign_attempt'], q['sign_artifact'], q['sign_digest'], 'signing-results-')):
            responses['/actions/runs/' + run + '/attempts/' + attempt] = {'id': int(run), 'run_attempt': int(attempt),
                'head_sha': sr['signer_commit'], 'head_branch': 'master', 'repository': {'id': 1409587171},
                'head_repository': {'id': 1409587171}, 'event': 'workflow_dispatch',
                'path': '.github/workflows/release-sign.yml', 'status': 'completed', 'conclusion': 'success'}
            responses['/actions/artifacts/' + artifact] = {'id': int(artifact), 'name': prefix + run + '-' + attempt,
                'expired': False, 'size_in_bytes': 123, 'digest': 'sha256:' + digest,
                'workflow_run': {'id': int(run), 'head_sha': sr['signer_commit'], 'repository_id': 1409587171,
                                 'head_repository_id': 1409587171}}
        return responses

    def test_closed_request_requires_exact_S_and_actual_distinct_sign_run(self):
        env = {'RELEASE_RECORD': json.dumps(self.r), 'RESULT_REQUEST': json.dumps(self.q)}
        with patch.dict(os.environ, env): self.assertEqual(v.request(), self.q)
        for key, value in [('verifier_commit', 'bad'), ('sign_attempt', '0'), ('execution_sha256', 'f'), ('extra', True)]:
            bad = dict(self.q); bad[key] = value
            with patch.dict(os.environ, dict(env, RESULT_REQUEST=json.dumps(bad))), self.assertRaises(ValueError): v.request()
        bad = copy.deepcopy(self.q); bad['signing_request']['signer_commit'] = 'b' * 40
        with patch.dict(os.environ, dict(env, RESULT_REQUEST=json.dumps(bad))), self.assertRaises(ValueError): v.request()

    def test_verifier_identity_is_V_not_original_S_and_has_no_oidc(self):
        q = self.q
        env = {'GITHUB_EVENT_NAME': 'workflow_dispatch', 'GITHUB_REPOSITORY': 'bohewu/yetlet',
            'GITHUB_REPOSITORY_ID': '1409587171', 'GITHUB_REPOSITORY_OWNER_ID': '35250002',
            'GITHUB_REF': 'refs/heads/master', 'GITHUB_SHA': q['verifier_commit'],
            'RESULT_WORKFLOW_SHA': q['verifier_commit'],
            'RESULT_WORKFLOW_REF': 'bohewu/yetlet/' + v.WORKFLOW + '@refs/heads/master'}
        with patch.dict(os.environ, env, clear=True), patch.object(v.subprocess, 'check_output', return_value=q['verifier_commit']):
            v.identity(q)
        for key, value in [('GITHUB_SHA', self.r['signer_commit']), ('RESULT_WORKFLOW_SHA', 'f' * 40),
                           ('ACTIONS_ID_TOKEN_REQUEST_URL', 'never-access')]:
            with patch.dict(os.environ, dict(env, **{key: value}), clear=True), self.assertRaises(ValueError): v.identity(q)

    def test_preflight_rejects_environment_run_artifact_and_live_source_before_key(self):
        data = self.metadata()
        with patch.object(v, 'identity'), patch.object(s.api, 'api', side_effect=lambda p: data[p]): v.gate(self.q)
        for path, key, value in [('/branches/master', 'protected', False),
                ('/environments/' + v.ENVIRONMENT, 'can_admins_bypass', True),
                ('/actions/runs/31/attempts/5', 'head_sha', 'f' * 40),
                ('/actions/artifacts/32', 'digest', 'sha256:' + 'f' * 64)]:
            bad = copy.deepcopy(data); bad[path][key] = value
            with patch.object(v, 'identity'), patch.object(s.api, 'api', side_effect=lambda p: bad[p]), \
                    patch.object(s, 'command') as native, self.assertRaises(ValueError):
                v.gate(self.q)
            native.assert_not_called()

    def frozen(self, base, age_sha=None):
        final = base / 'original'; final.mkdir()
        r = fixture(final); s.bind_record(r)
        (final / 'EVIDENCE-CLOSURE.json').write_bytes(b'{"retained":"SYNTHETIC private evidence"}')
        (final / 'private-record').write_bytes(b'SYNTHETIC private proof')
        s.checksums.run('create', str(final))
        q = result_request(r); sr = q['signing_request']
        sr['manifest_sha256'] = s.sha((final / s.MANIFEST).read_bytes())
        work = base / 'work'; work.mkdir(); (work / 'transport').mkdir(); (work / 'results').mkdir()
        s.archive(final, work / 'transport/frozen-signing-inputs.tar.age')
        receipt = {'schema': 1, 'record_sha256': records.digest(r), 'source': s.C, 'signer_commit': r['signer_commit'],
            'tag': s.TAG, 'build_run': s.BUILD, 'build_attempt': s.BUILD_ATTEMPT, 'verifier_run': s.VERIFIER,
            'verifier_attempt': s.VERIFIER_ATTEMPT, 'build_artifact': s.BUILD_ARTIFACT, 'build_zip_sha256': s.BUILD_ZIP,
            'prepare_run': sr['run'], 'prepare_attempt': sr['attempt'], 'provenance_sha256': s.PUBLIC_SHA,
            'manifest_sha256': sr['manifest_sha256'], 'cosign_sha256': s.COSIGN_SHA, 'root_sha256': s.ROOT_SHA,
            'config_sha256': s.CONFIG_SHA, 'signer_san': s.SAN, 'age_sha256': age_sha or s.AGE_SHA,
            'recipient_sha256': s.RECIPIENT_SHA, 'issuer': s.ISSUER, 'D6': 'Pending',
            'ciphertext_sha256': s.sha((work / 'transport/frozen-signing-inputs.tar.age').read_bytes()),
            'closure_sha256': s.sha((final / 'EVIDENCE-CLOSURE.json').read_bytes())}
        s.json_write(work / 'transport/SIGNING-INPUTS.json', receipt)
        sr['receipt_sha256'] = s.sha((work / 'transport/SIGNING-INPUTS.json').read_bytes())
        raw = records.canonical({'dsseEnvelope': {'payload': base64.b64encode((final / 'PROVENANCE-UNSIGNED.json').read_bytes()).decode()}})
        (work / 'results/provenance.sigstore.json').write_bytes(raw)
        (work / 'results/manifest.sigstore.json.age').write_bytes(b'{"SYNTHETIC":"private bundle"}')
        e = {'schema': 1, 'record_sha256': records.digest(r), 'source': s.C, 'signer_commit': r['signer_commit'],
            'approved_inputs': sr, 'sign_run': q['sign_run'], 'sign_attempt': q['sign_attempt'],
            'provenance_bundle_sha256': s.sha(raw),
            'encrypted_manifest_bundle_sha256': s.sha((work / 'results/manifest.sigstore.json.age').read_bytes()),
            'same_job_verification': True, 'independent_verification': 'Pending', 'final_owner_acceptance': 'Pending', 'publication': False}
        s.json_write(work / 'results/SIGNING-EXECUTION.json', e)
        q['execution_sha256'] = s.sha((work / 'results/SIGNING-EXECUTION.json').read_bytes())
        return r, q, work

    def test_full_frozen_result_checks_all_subjects_entries_and_original_certificate_run(self):
        with tempfile.TemporaryDirectory() as temp:
            base = Path(temp)
            age = base / 'age'; age.write_bytes(b'SYNTHETIC executable marker, never executed')
            r, q, work = self.frozen(base, s.sha(age.read_bytes()))
            calls = []
            def command(args):
                args = list(map(str, args)); calls.append(args)
                if '--decrypt' in args: shutil.copyfile(args[-1], args[args.index('--output') + 1])
                else: self.assertIn(args[1], ('verify-blob-attestation', 'verify-blob'))
            with patch.object(s, 'tools'), patch.object(s, 'AGE_SHA', s.sha(age.read_bytes())), \
                    patch.object(s, 'command', side_effect=command), patch.object(s, 'certificate') as cert, \
                    patch.dict(os.environ, {'SIGN_AGE': str(age), 'GITHUB_RUN_ID': '41', 'GITHUB_RUN_ATTEMPT': '1'}):
                proof = v.verify(q, work, base)
            self.assertEqual(proof['subjects_verified'], 16)
            self.assertEqual(proof['entries_verified'], 19)
            self.assertEqual(sum(c[1] == 'verify-blob-attestation' for c in calls), 16)
            self.assertEqual(sum(c[1] == 'verify-blob' for c in calls), 1)
            self.assertEqual(len(cert.call_args_list), 2)
            self.assertTrue(all(c.args[2:] == ('31', '5') for c in cert.call_args_list))
            self.assertNotIn('private-record', json.dumps(proof))
            self.assertEqual(proof['record_sha256'], records.digest(r))

    def test_record_or_result_tamper_stops_before_private_decrypt(self):
        with tempfile.TemporaryDirectory() as temp:
            r, q, work = self.frozen(Path(temp))
            s.RECORD = dict(r, release_id='16')
            with patch.object(s, 'command') as native, self.assertRaises(ValueError): v.execution(q, work)
            native.assert_not_called()
            s.bind_record(r); (work / 'results/manifest.sigstore.json.age').write_bytes(b'tampered')
            with patch.object(s, 'command') as native, self.assertRaises(ValueError): v.execution(q, work)
            native.assert_not_called()

    def test_cleanup_precedes_receipt_and_failure_never_emits_success(self):
        for failure in (False, True):
            with tempfile.TemporaryDirectory() as temp:
                base = Path(temp); work = base / 'private'; work.mkdir(); (work / 'identity').write_text('SYNTHETIC')
                output = base / 'safe.json'
                env = {'RESULT_WORK': str(work), 'RESULT_OUTPUT': str(output), 'RESULT_TOOLS': str(base)}
                with patch.dict(os.environ, env), patch.object(sys, 'argv', ['verify', 'verify']), \
                        patch.object(v, 'request', return_value=self.q), patch.object(v, 'gate'), \
                        patch.object(v, 'verify', side_effect=ValueError('synthetic failure') if failure else None,
                                     return_value={'safe': True}):
                    if failure:
                        with self.assertRaises(ValueError): v.main()
                    else: v.main()
                self.assertFalse(work.exists()); self.assertEqual(output.exists(), not failure)

    def test_cleanup_failure_prevents_a_verification_receipt(self):
        with tempfile.TemporaryDirectory() as temp:
            base = Path(temp); work = base / 'private'; work.mkdir(); output = base / 'safe.json'
            env = {'RESULT_WORK': str(work), 'RESULT_OUTPUT': str(output), 'RESULT_TOOLS': str(base)}
            with patch.dict(os.environ, env), patch.object(sys, 'argv', ['verify', 'verify']), \
                    patch.object(v, 'request', return_value=self.q), patch.object(v, 'gate'), \
                    patch.object(v, 'verify', return_value={'safe': True}), \
                    patch.object(v.shutil, 'rmtree', side_effect=OSError('synthetic cleanup failure')):
                with self.assertRaises(OSError): v.main()
            self.assertFalse(output.exists())


class PublicTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(); self.addCleanup(self.temp.cleanup)
        self.directory = Path(self.temp.name)
        self.r = fixture(self.directory); pub.s.bind_record(self.r)
        s.checksums.run('create', str(self.directory))
        raw = records.canonical({'dsseEnvelope': {'payload': base64.b64encode(
            (self.directory / 'PROVENANCE-UNSIGNED.json').read_bytes()).decode()}})
        (self.directory / 'provenance.sigstore.json').write_bytes(raw)
        self.v = {'schema': 1, 'status': 'PASS', 'independent_verification': True,
            'source': self.r['source'], 'signer_commit': self.r['signer_commit'],
            'record_sha256': records.digest(self.r), 'subjects_verified': 16, 'entries_verified': 19,
            'provenance_sha256': self.r['provenance']['sha256'], 'manifest_sha256': 'f' * 64,
            'provenance_bundle_sha256': s.sha(raw)}
        self.verification = records.canonical(self.v)
        acceptance = {'schema': 1, 'record_sha256': records.digest(self.r), 'source': self.r['source'],
            'signer_commit': self.r['signer_commit'], 'verification_sha256': s.sha(self.verification),
            'manifest_sha256': self.v['manifest_sha256'], 'provenance_bundle_sha256': s.sha(raw),
            'accepted': True, 'scope': 'Experimental source packages', 'publication': 'GitHub pre-release', 'registry_upload': False}
        (self.directory / 'SIGNING-APPROVAL.json').write_bytes(records.canonical(acceptance))
        self.blobs = {name: (self.directory / name).read_bytes() for name in pub.names()}
        self.notes = b'SYNTHETIC reviewed release notes\n'
        self.p = {'schema': 1, 'record_sha256': records.digest(self.r), 'release_id': self.r['release_id'],
            'name': 'Yetlet v' + self.r['version'], 'notes_sha256': s.sha(self.notes),
            'verification_sha256': s.sha(self.verification),
            'approval_sha256': s.sha(self.blobs['SIGNING-APPROVAL.json']),
            'assets': [{'name': name, 'size': len(raw), 'sha256': s.sha(raw)} for name, raw in sorted(self.blobs.items())]}
        pub.plan(self.p)

    def test_asset_upload_uses_gh_json_response_and_exact_binary_stdin(self):
        asset = self.p['assets'][0]
        url = 'https://uploads.github.com/repos/bohewu/yetlet/releases/15/assets?name=' + asset['name']
        response = records.canonical({'id': 123, 'name': asset['name']})
        process = subprocess.CompletedProcess([], 0, response, b'')
        with patch.dict(os.environ, {'GH_TOKEN': 'SYNTHETIC token'}), \
                patch.object(pub.subprocess, 'run', return_value=process) as run, \
                patch.object(pub.urllib.request, 'build_opener') as native:
            self.assertEqual(pub.http(url, 'POST', self.blobs[asset['name']], binary=True), json.loads(response))
        args, kwargs = run.call_args
        self.assertEqual(args[0], ['gh', 'api', '--method', 'POST', url,
            '--header', 'Accept: application/vnd.github+json',
            '--header', 'Content-Type: application/octet-stream',
            '--header', 'Content-Length: ' + str(len(self.blobs[asset['name']])), '--input', '-'])
        self.assertEqual(kwargs['input'], self.blobs[asset['name']])
        self.assertEqual(kwargs['timeout'], 120)
        self.assertNotIn('SYNTHETIC token', ' '.join(args[0]))
        self.assertNotIn('shell', kwargs)
        run.assert_called_once(); native.assert_not_called()

    @unittest.skipUnless(shutil.which('gh'), 'GitHub CLI unavailable for loopback transport check')
    def test_real_gh_stdin_has_known_length_and_exact_binary_body(self):
        seen = []
        class Handler(http.server.BaseHTTPRequestHandler):
            def do_POST(self):
                length = self.headers.get('Content-Length')
                body = self.rfile.read(int(length)) if length else b''
                seen.append((length, self.headers.get('Transfer-Encoding'), body))
                self.send_response(201 if length else 411)
                raw = b'{"id":123}'
                self.send_header('Content-Type', 'application/json')
                self.send_header('Content-Length', str(len(raw)))
                self.end_headers()
                self.wfile.write(raw)
            def log_message(self, *args):
                pass
        server = http.server.HTTPServer(('127.0.0.1', 0), Handler)
        worker = threading.Thread(target=server.serve_forever, daemon=True)
        worker.start()
        try:
            env = {'PATH': os.environ.get('PATH', ''),
                'GH_CONFIG_DIR': str(self.directory), 'GH_TOKEN': 'synthetic-loopback-only',
                'GH_ENTERPRISE_TOKEN': 'synthetic-loopback-only', 'NO_PROXY': '127.0.0.1',
                'GH_PROMPT_DISABLED': '1'}
            raw = b'SYNTHETIC\x00\xff\r\n' * 1024
            with patch.dict(os.environ, env, clear=True):
                self.assertEqual(pub.upload('http://127.0.0.1:' + str(server.server_port) + '/upload', raw), {'id': 123})
            self.assertEqual(seen, [(str(len(raw)), None, raw)])
        finally:
            server.shutdown()
            server.server_close()
            worker.join(timeout=2)
        self.assertFalse(worker.is_alive())

    def test_api_http_status_survives_without_url_headers_or_body(self):
        error = urllib.error.HTTPError('https://private.invalid/?token=secret', 403,
            'private message', {'Private': 'header'}, io.BytesIO(b'private body'))
        with patch.dict(os.environ, {'GH_TOKEN': 'SYNTHETIC'}), \
                patch.object(pub.urllib.request, 'build_opener') as opener:
            opener.return_value.open.side_effect = error
            with self.assertRaises(pub.ApiError) as caught: pub.api('/releases/15')
        detail = pub.failure_details(caught.exception)
        self.assertEqual(detail, {'operation': 'api-read', 'class': 'http', 'http_status': 403})
        self.assertTrue(error.fp.closed)
        self.assertNotIn('private', json.dumps(detail) + str(caught.exception))

    def test_upload_rejects_anonymous_or_foreign_destination_before_gh(self):
        upload_url = 'https://uploads.github.com/repos/bohewu/yetlet/releases/15/assets?name=public'
        with patch.dict(os.environ, {'GH_TOKEN': 'SYNTHETIC token'}), \
                patch.object(pub.subprocess, 'run') as run:
            with self.assertRaises(ValueError):
                pub.http(upload_url, 'POST', b'bytes', binary=True, authenticated=False)
            with self.assertRaises(ValueError):
                pub.http('https://example.invalid/assets', 'POST', b'bytes', binary=True)
        run.assert_not_called()

    def test_upload_failure_reports_only_safe_status_and_never_retries(self):
        process = subprocess.CompletedProcess([], 1, b'', b'gh: SYNTHETIC private diagnostic (HTTP 422)')
        with patch.object(pub.subprocess, 'run', return_value=process) as run:
            with self.assertRaises(pub.UploadError) as caught:
                pub.upload('https://uploads.github.com/repos/bohewu/yetlet/releases/15/assets', b'bytes')
        run.assert_called_once()
        error = caught.exception
        self.assertEqual((error.kind, error.http_status), ('http', 422))
        stderr = io.StringIO()
        with patch.object(pub, 'main', side_effect=error), patch.object(sys, 'stderr', stderr):
            self.assertEqual(pub.cli(), 1)
        self.assertIn('operation=upload class=http http_status=422', stderr.getvalue())
        self.assertNotIn('private diagnostic', stderr.getvalue())

    def test_upload_timeout_missing_cli_and_invalid_response_stop(self):
        for failure, kind in [(subprocess.TimeoutExpired('gh', 120), 'timeout'),
                              (FileNotFoundError('SYNTHETIC path'), 'unavailable')]:
            with self.subTest(kind=kind), patch.object(pub.subprocess, 'run', side_effect=failure) as run:
                with self.assertRaises(pub.UploadError) as caught: pub.upload('unused', b'bytes')
                self.assertEqual(caught.exception.kind, kind); run.assert_called_once()
        process = subprocess.CompletedProcess([], 0, b'not JSON', b'')
        with patch.object(pub.subprocess, 'run', return_value=process), self.assertRaises(pub.UploadError) as caught:
            pub.upload('unused', b'bytes')
        self.assertEqual(caught.exception.kind, 'response')

    def test_failed_upload_retains_safe_receipt_and_stops_publication(self):
        plan = self.directory / 'plan.json'; plan.write_bytes(records.canonical(self.p))
        notes = self.directory / 'notes.md'; notes.write_bytes(self.notes)
        verification = self.directory / 'verified.json'; verification.write_bytes(self.verification)
        output = self.directory / 'failed.json'
        argv = ['release-public.py', 'stage', '--plan', str(plan), '--public-dir', str(self.directory),
                '--notes', str(notes), '--verification', str(verification), '--output', str(output)]
        with patch.dict(os.environ, {'RELEASE_RECORD': json.dumps(self.r)}), patch.object(sys, 'argv', argv), \
                patch.object(pub, 'operate', side_effect=pub.UploadError('http', 422)) as operate:
            with self.assertRaises(pub.UploadError): pub.main()
        operate.assert_called_once()
        receipt = json.loads(output.read_bytes())
        self.assertEqual(receipt['transport_error'], {'operation': 'upload', 'class': 'http', 'http_status': 422})
        self.assertEqual(receipt['status'], 'FAIL')
        self.assertTrue(receipt['remote_partial_state_may_exist'])
        self.assertFalse(receipt['automatic_retry'])
        self.assertFalse(receipt['registry_upload'])

    def test_public_allowlist_and_local_proof_are_closed(self):
        self.assertEqual(pub.public_inputs(self.p, self.directory, self.notes, self.verification), self.blobs)
        for name in ('manifest.sigstore.json', 'private-evidence.tar.age', '../private', self.p['assets'][1]['name']):
            bad = copy.deepcopy(self.p); bad['assets'][0]['name'] = name
            with self.assertRaises(ValueError): pub.plan(bad)
        with self.assertRaises(ValueError): pub.public_inputs(self.p, self.directory, b'changed', self.verification)
        with self.assertRaises(ValueError): pub.public_inputs(self.p, self.directory, self.notes, self.verification + b' ')
        (self.directory / 'provenance.sigstore.json').write_bytes(b'tampered')
        with self.assertRaises(ValueError): pub.public_inputs(self.p, self.directory, self.notes, self.verification)

    def test_consolidated_six_assets_validate_stage_readback_and_publish(self):
        with tempfile.TemporaryDirectory() as temp:
            self.directory = Path(temp)
            self.r = fixture(self.directory, consolidated=True); pub.s.bind_record(self.r)
            s.checksums.run('create', str(self.directory))
            bundle = records.canonical({'dsseEnvelope': {'payload': base64.b64encode(
                (self.directory/'PROVENANCE-UNSIGNED.json').read_bytes()).decode()}})
            (self.directory/'provenance.sigstore.json').write_bytes(bundle)
            proof = dict(self.v, record_sha256=records.digest(self.r), subjects_verified=2,
                         provenance_sha256=self.r['provenance']['sha256'], provenance_bundle_sha256=s.sha(bundle))
            self.verification = records.canonical(proof)
            approval = {'schema':2,'record_sha256':records.digest(self.r),'source':self.r['source'],
                'signer_commit':self.r['signer_commit'],'verification_sha256':s.sha(self.verification),
                'manifest_sha256':proof['manifest_sha256'],'provenance_bundle_sha256':s.sha(bundle),
                'accepted':True,'scope':'Experimental source packages','publication':'GitHub pre-release + crates.io',
                'registry_upload':True,'registry_plan_sha256':'d'*64,'publication_request_sha256':'e'*64}
            (self.directory/'SIGNING-APPROVAL.json').write_bytes(records.canonical(approval))
            self.blobs = {n:(self.directory/n).read_bytes() for n in pub.names()}
            self.p = dict(self.p, schema=2, record_sha256=records.digest(self.r),
                verification_sha256=s.sha(self.verification), approval_sha256=s.sha(self.blobs['SIGNING-APPROVAL.json']),
                registry_plan_sha256='d'*64,publication_request_sha256='e'*64,
                assets=[{'name':n,'size':len(b),'sha256':s.sha(b)} for n,b in sorted(self.blobs.items())])
            pub.plan(self.p)
            self.assertEqual(len(self.p['assets']), 6)
            self.assertEqual(pub.public_inputs(self.p,self.directory,self.notes,self.verification),self.blobs)
            for bad in (dict(self.p,assets=self.p['assets'][:-1]),
                        dict(self.p,assets=self.p['assets']+[{'name':'private-evidence.tar.age','size':1,'sha256':'a'*64}])):
                with self.assertRaises(ValueError): pub.plan(bad)
            state, http = self.remote(0)
            with patch.object(pub,'http',side_effect=http):
                pub.operate('stage',self.p,self.blobs,self.notes)
                pub.operate('readback',self.p,self.blobs,self.notes)
                result=pub.operate('publish',self.p,self.blobs,self.notes,'publish-exact-approved-plan')
            self.assertEqual(state['uploads'],6); self.assertEqual(len(result['assets']),6)
            self.assertFalse(state['draft'])

    def remote(self, count=20, fail_upload=False):
        state = {'draft': True, 'assets': [], 'writes': [], 'anonymous': [], 'uploads': 0}
        def asset(a, ident):
            return dict(id=ident, name=a['name'], size=a['size'], digest='sha256:' + a['sha256'], state='uploaded')
        state['assets'] = [asset(a, 100 + i) for i, a in enumerate(self.p['assets'][:count])]
        def http(url, method='GET', body=None, binary=False, authenticated=True, **kwargs):
            if not authenticated: state['anonymous'].append(url)
            if method == 'POST':
                state['writes'].append((url, method, body)); state['uploads'] += 1
                if fail_upload: raise TimeoutError('SYNTHETIC uncertain write')
                name = pub.urllib.parse.parse_qs(pub.urllib.parse.urlsplit(url).query)['name'][0]
                a = next(a for a in self.p['assets'] if a['name'] == name)
                self.assertEqual(body, self.blobs[name]); state['assets'].append(asset(a, 200 + state['uploads']))
                return state['assets'][-1]
            if method == 'PATCH':
                value = json.loads(body); state['writes'].append((url, method, value)); state['draft'] = value['draft']
                return {}
            if url.endswith('/git/ref/tags/' + pub.s.TAG):
                return {'ref': 'refs/tags/' + pub.s.TAG, 'object': {'sha': self.r['source'], 'type': 'commit',
                    'url': pub.API + '/git/commits/' + self.r['source']}}
            if url.endswith('/releases/15'):
                return {'id': 15, 'tag_name': pub.s.TAG, 'target_commitish': self.r['source'], 'draft': state['draft'],
                    'prerelease': True, 'name': self.p['name'], 'body': self.notes.decode()}
            if '/assets?per_page=100' in url: return copy.deepcopy(state['assets'])
            if '/releases/assets/' in url:
                a = next(a for a in state['assets'] if str(a['id']) == url.rsplit('/', 1)[1])
                return self.blobs[a['name']]
            if url.endswith('/releases/latest'): return None
            self.fail('unexpected API path')
        return state, http

    def test_stage_uploads_missing_once_and_validates_actual_readback(self):
        state, http = self.remote(19)
        with patch.object(pub, 'http', side_effect=http): result = pub.operate('stage', self.p, self.blobs, self.notes)
        self.assertEqual(state['uploads'], 1); self.assertEqual(len(result['assets']), 20)
        self.assertTrue(all(w[1] == 'POST' for w in state['writes']))
        self.assertTrue(state['draft'])

    def test_unexpected_existing_asset_stops_before_any_write(self):
        state, http = self.remote(1); state['assets'][0]['name'] = 'private-evidence.tar.age'
        with patch.object(pub, 'http', side_effect=http), self.assertRaises(ValueError):
            pub.operate('stage', self.p, self.blobs, self.notes)
        self.assertEqual(state['writes'], [])

    def test_uncertain_upload_is_not_retried_and_no_publish_follows(self):
        state, http = self.remote(18, True)
        with patch.object(pub, 'http', side_effect=http), self.assertRaises(TimeoutError):
            pub.operate('stage', self.p, self.blobs, self.notes)
        self.assertEqual(state['uploads'], 1); self.assertTrue(state['draft'])

    def test_publish_requires_confirmation_and_preserves_every_binding(self):
        state, http = self.remote()
        with patch.object(pub, 'http', side_effect=http):
            with self.assertRaises(ValueError): pub.operate('publish', self.p, self.blobs, self.notes)
            self.assertEqual(state['writes'], [])
            pub.operate('publish', self.p, self.blobs, self.notes, 'publish-exact-approved-plan')
        self.assertEqual(len(state['writes']), 1)
        self.assertEqual(state['writes'][0][2], {'tag_name': pub.s.TAG, 'target_commitish': self.r['source'],
            'name': self.p['name'], 'body': self.notes.decode(), 'draft': False, 'prerelease': True, 'make_latest': 'false'})
        self.assertTrue(any('/releases/assets/' in u for u in state['anonymous']))

    def test_readback_detects_tampered_download_without_writes(self):
        state, http = self.remote()
        def tamper(url, **kwargs):
            return b'changed' if '/releases/assets/' in url else http(url, **kwargs)
        with patch.object(pub, 'http', side_effect=tamper), self.assertRaises(ValueError):
            pub.operate('readback', self.p, self.blobs, self.notes)
        self.assertEqual(state['writes'], [])

    def test_http_redirect_never_forwards_token_and_rejects_foreign_host(self):
        error = urllib.error.HTTPError(pub.API + '/releases/assets/1', 302, 'redirect',
            {'Location': 'https://release-assets.githubusercontent.com/SYNTHETIC'}, None)
        body = io.BytesIO(b'public bytes'); body.status = 200
        with patch.dict(os.environ, {'GH_TOKEN': 'SYNTHETIC token'}), patch.object(pub.urllib.request, 'build_opener') as opener:
            opener.return_value.open.side_effect = [error, body]
            self.assertEqual(pub.http(pub.API + '/releases/assets/1', binary=True), b'public bytes')
            calls = opener.return_value.open.call_args_list
            self.assertIsNotNone(calls[0].args[0].get_header('Authorization'))
            self.assertIsNone(calls[1].args[0].get_header('Authorization'))
        error = urllib.error.HTTPError(pub.API + '/releases/assets/1', 302, 'redirect',
            {'Location': 'https://evil.invalid/private'}, None)
        with patch.dict(os.environ, {'GH_TOKEN': 'SYNTHETIC token'}), patch.object(pub.urllib.request, 'build_opener') as opener:
            opener.return_value.open.side_effect = error
            with self.assertRaises(ValueError): pub.http(pub.API + '/releases/assets/1', binary=True)
            self.assertEqual(opener.return_value.open.call_count, 1)

    def test_release_identity_mismatch_blocks_before_upload_or_publication(self):
        for key, value in [('tag_name', 'wrong'), ('target_commitish', 'e' * 40), ('prerelease', False)]:
            state, http = self.remote(19)
            def wrong(url, **kwargs):
                value_out = http(url, **kwargs)
                if url.endswith('/releases/15'): value_out[key] = value
                return value_out
            with patch.object(pub, 'http', side_effect=wrong), self.assertRaises(ValueError):
                pub.operate('stage', self.p, self.blobs, self.notes)
            self.assertEqual(state['writes'], [])


class WorkflowTests(unittest.TestCase):
    def test_new_manual_no_oidc_verifier_and_versioned_workflow_inputs(self):
        import yaml
        for filename in ('release.yml', 'evidence-verify.yml', 'release-sign.yml', 'release-result-verify.yml'):
            path = ROOT / '.github/workflows' / filename
            w = yaml.load(path.read_text(), Loader=yaml.BaseLoader)
            self.assertEqual(set(w['on']), {'workflow_dispatch'})
            self.assertIn('record', w['on']['workflow_dispatch']['inputs'])
            self.assertIn('RELEASE_RECORD', w.get('env', w['jobs'].get('preparation', {}).get('env', {})))
            for job in w['jobs'].values():
                for step in job['steps']:
                    if 'run' in step:
                        result = subprocess.run(['bash', '-n'], input=step['run'], text=True, capture_output=True)
                        self.assertEqual(result.returncode, 0, result.stderr)
        w = yaml.load((ROOT / '.github/workflows/release-result-verify.yml').read_text(), Loader=yaml.BaseLoader)
        import ast, re
        for filename in ('release.yml', 'evidence-verify.yml', 'release-sign.yml', 'release-result-verify.yml'):
            source = (ROOT / '.github/workflows' / filename).read_text()
            for match in re.finditer(r"<<'(PY[A-Z]*)'\n(.*?)^          \1$", source, re.M | re.S):
                import textwrap
                ast.parse(textwrap.dedent(match.group(2)))
        self.assertEqual(w['permissions'], {'contents': 'read', 'actions': 'read'})
        self.assertEqual(w['jobs']['verify']['environment'], 'release-result-verify')
        for job in w['jobs'].values(): self.assertNotIn('permissions', job)
        self.assertTrue(any(s.get('if') == 'always()' for s in w['jobs']['verify']['steps']))
        self.assertTrue(any(step.get('with', {}).get('path') == '${{ runner.temp }}/RESULT-VERIFICATION.json' for step in w['jobs']['verify']['steps']))


if __name__ == '__main__':
    unittest.main()
