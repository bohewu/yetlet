"""Changed registry transport and single-publication policy; no real credentials/network."""
import copy
import importlib.util
import io
import json
import os
from pathlib import Path
import struct
import tarfile
import tempfile
import unittest
from unittest.mock import patch

import release_policy as policy
import release_record as records
import release_registry as registry

HERE = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location('publication_tests', HERE / 'release-publish.py')
publication = importlib.util.module_from_spec(spec)
spec.loader.exec_module(publication)


def record():
    return {'schema': 1, 'version': '1.2.3-alpha.4', 'source': 'c' * 40, 'cohort': 'sixteen',
        'build': {'run': '11', 'attempt': '1', 'artifact': '12', 'sha256': 'b' * 64},
        'custody_verifier': {'run': '13', 'attempt': '1', 'environment_id': '14'},
        'signer_commit': 'b' * 40, 'provenance': {'sha256': 'e' * 64, 'size': 123}, 'release_id': '15'}


def archives(directory, cycle=False):
    r = record()
    subjects = {}
    for name in records.PACKAGES:
        manifest = '[package]\nname=' + json.dumps(name) + '\nversion=' + json.dumps(r['version']) + '\n'
        manifest += 'readme="README.md"\nlicense="MIT"\ndescription="Fixture"\nrust-version="1.99"\n'
        dep = lambda n: n + ' = { version="' + r['version'] + '" }\n'
        if name == 'yetlet-config':
            manifest += '[dependencies]\n' + dep('yetlet-secrets')
        if name == 'yetlet-secrets':
            manifest += '[dev-dependencies]\n' + dep('yetlet-core')
            if cycle:
                manifest += dep('yetlet-config')
        if name == 'yetlet-auth':
            manifest += '[target.\'cfg(unix)\'.dev-dependencies]\nrenamed = {package="upstream",version="1.0",optional=false,default-features=false,features=["feature"]}\n'
        raw = io.BytesIO()
        with tarfile.open(fileobj=raw, mode='w:gz') as archive:
            for path, data in [('Cargo.toml', manifest.encode()), ('README.md', b'Fixture')]:
                info = tarfile.TarInfo(name + '-' + r['version'] + '/' + path)
                info.size = len(data)
                archive.addfile(info, io.BytesIO(data))
        filename = name + '-' + r['version'] + '.crate'
        (directory / filename).write_bytes(raw.getvalue())
        subjects[filename] = registry.sha(raw.getvalue())
    return r, subjects


class RegistryTests(unittest.TestCase):
    def test_all_sixteen_original_archive_bytes_and_dependency_metadata_are_frozen(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            r, subjects = archives(root)
            plan, bodies = registry.prepare(r, root, subjects)
            self.assertEqual(len(plan['packages']), 16)
            self.assertLess([x['name'] for x in plan['packages']].index('yetlet-secrets'),
                            [x['name'] for x in plan['packages']].index('yetlet-config'))
            for row in plan['packages']:
                body = bodies[row['name']]
                size = struct.unpack_from('<I', body)[0]
                meta = json.loads(body[4:4+size])
                length = struct.unpack_from('<I', body, 4+size)[0]
                raw = body[8+size:]
                self.assertEqual(raw, (root / row['filename']).read_bytes())
                self.assertEqual(length, len(raw))
                self.assertEqual(registry.sha(raw), subjects[row['filename']])
                if row['name'] == 'yetlet-auth':
                    self.assertEqual(meta['deps'], [{'name': 'upstream', 'version_req': '1.0',
                        'features': ['feature'], 'optional': False, 'default_features': False,
                        'kind': 'dev', 'target': 'cfg(unix)', 'registry': None,
                        'explicit_name_in_toml': 'renamed'}])

    def test_development_cycle_and_changed_archive_reject_before_network(self):
        with tempfile.TemporaryDirectory() as temporary, patch.object(registry, 'request') as network:
            root = Path(temporary)
            r, subjects = archives(root, cycle=True)
            with self.assertRaises(ValueError): registry.prepare(r, root, subjects)
            r, subjects = archives(root)
            next(root.glob('*.crate')).write_bytes(b'changed')
            with self.assertRaises(ValueError): registry.prepare(r, root, subjects)
            network.assert_not_called()

    def test_failed_write_is_journaled_once_and_not_implicitly_resumed(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            r, subjects = archives(root)
            plan, bodies = registry.prepare(r, root, subjects)
            receipt = root / 'receipt.json'
            with patch.object(registry, 'request', side_effect=registry.RegistryError('transport')) as network:
                with self.assertRaises(registry.RegistryError): registry.publish(plan, bodies, receipt)
                self.assertEqual(network.call_count, 1)
                proof = json.loads(receipt.read_text())
                self.assertEqual(proof['status'], 'incomplete')
                self.assertEqual(proof['packages'][0]['status'], 'attempted')
                with self.assertRaises(ValueError): registry.publish(plan, bodies, receipt)
                self.assertEqual(network.call_count, 1)

    def test_readback_binds_index_and_download_digest(self):
        row = {'name': 'yetlet-core', 'filename': 'yetlet-core-1.2.3.crate',
               'sha256': registry.sha(b'archive'), 'size': 7}
        index = records.canonical({'name': row['name'], 'vers': '1.2.3', 'cksum': row['sha256'], 'yanked': False})
        with patch.object(registry, 'request', side_effect=[index, b'archive']):
            self.assertEqual(registry.readback(row, '1.2.3')['download_sha256'], row['sha256'])
        with patch.object(registry, 'request', side_effect=[index, b'changed']):
            with self.assertRaises(ValueError): registry.readback(row, '1.2.3')

    def test_transport_token_is_only_sent_to_exact_publish_endpoint(self):
        class Response(io.BytesIO):
            pass
        seen = []
        class Opener:
            def open(self, request, timeout):
                seen.append(request)
                return Response(b'{}')
        with (patch.dict(os.environ, {'CARGO_REGISTRY_TOKEN': 'synthetic-test-token'}),
              patch.object(registry.urllib.request, 'build_opener', return_value=Opener())):
            registry.request(registry.API, 'PUT', b'body')
            registry.request('https://index.crates.io/ye/tl/yetlet-core')
            registry.request('https://static.crates.io/crates/yetlet-core/file.crate')
            self.assertEqual(seen[0].get_header('Authorization'), 'synthetic-test-token')
            self.assertTrue(all(r.get_header('Authorization') is None for r in seen[1:]))
            with self.assertRaises(ValueError): registry.request('https://example.invalid', 'PUT', b'body')
        redirect = registry.NoRedirect()
        self.assertIsNone(redirect.redirect_request(seen[0], None, 302, '', {}, 'https://example.invalid'))

    def test_success_records_each_readback_before_uploading_a_dependent(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            r, subjects = archives(root)
            plan, bodies = registry.prepare(r, root, subjects)
            complete = set()
            def upload(url, method, body):
                size = struct.unpack_from('<I', body)[0]
                name = json.loads(body[4:4+size])['name']
                row = next(x for x in plan['packages'] if x['name'] == name)
                self.assertTrue(set(row['requires']) <= complete)
                return records.canonical({'krate': {'name': name}})
            def read(row, version, attempts):
                complete.add(row['name'])
                return {'index_sha256': row['sha256'], 'download_sha256': row['sha256']}
            with (patch.object(registry, 'request', side_effect=upload),
                  patch.object(registry, 'readback', side_effect=read)):
                result = registry.publish(plan, bodies, root/'receipt.json')
            self.assertEqual(len(complete), 16)
            self.assertEqual(result['status'], 'registry_verified_consumer_pending')


class PublicationTests(unittest.TestCase):
    def test_preflight_never_reads_a_draft_and_stage_checks_it_before_writing(self):
        s, pub = publication.s, publication.pub
        r = record()
        s.bind_record(r)
        proof = {'verifier_commit': 'd'*40, 'verifier_run': '21', 'verifier_attempt': '1',
            'manifest_sha256': 'a'*64, 'provenance_bundle_sha256': 'b'*64}
        proof_raw = records.canonical(proof)
        request = {'verifier_commit': proof['verifier_commit'], 'verifier_run': '21',
            'verifier_attempt': '1', 'artifact': '22', 'artifact_sha256': 'c'*64,
            'verification_sha256': s.sha(proof_raw), 'registry_plan_sha256': records.digest({}),
            'name': 'Fixture', 'notes': 'Fixture notes', 'notes_sha256': s.sha(b'Fixture notes')}
        def unpack(raw, digest, directory, names):
            directory.mkdir()
            for name in names: (directory/name).write_bytes(b'Fixture')
            (directory/'RESULT-VERIFICATION.json').write_bytes(proof_raw)
            (directory/'PROVENANCE-UNSIGNED.json').write_bytes(b'{"subject":[]}')
        calls = []
        def api(path, authenticated=True):
            calls.append(path)
            if path.startswith('/git/ref/tags/'):
                return {'ref': 'refs/tags/'+s.TAG, 'object': {'sha': s.C, 'type': 'commit',
                    'url': pub.API+'/git/commits/'+s.C}}
            raise AssertionError('read-only preflight attempted a Draft API request')
        with tempfile.TemporaryDirectory() as temporary:
            with (patch.object(s, 'download', return_value=b'zip'),
                  patch.object(s, 'unpack_zip', side_effect=unpack),
                  patch.object(registry, 'prepare', return_value=({}, {})),
                  patch.object(pub, 'public_inputs', return_value={}),
                  patch.object(pub, 'api', side_effect=api)):
                plan, blobs, _, _ = publication.distribution(request, Path(temporary))
            self.assertEqual(calls, ['/git/ref/tags/'+s.TAG])
            with (patch.object(pub, 'release', side_effect=ValueError('Draft metadata mismatch')) as draft,
                  patch.object(pub, 'assets') as assets, patch.object(pub, 'http') as write):
                with self.assertRaises(ValueError): pub.operate('stage', plan, blobs, b'Fixture notes')
                draft.assert_called_once_with(plan, True)
                assets.assert_not_called()
                write.assert_not_called()

    def test_github_precedes_registry_and_partial_registry_failure_stops_consumer(self):
        for fail in (False, True):
            with self.subTest(fail=fail), tempfile.TemporaryDirectory() as temporary:
                root = Path(temporary)
                events = []
                r = {'notes': 'Fixture'}
                def operate(action, *args):
                    events.append('github-' + action)
                    return {'status': 'PASS'}
                def promote(*args):
                    events.append('registry')
                    if fail: raise registry.RegistryError('transport')
                    return {'status': 'registry_verified_consumer_pending'}
                def consume(*args):
                    events.append('consumer')
                    return {'status': 'PASS'}
                publication.s.bind_record(record())
                with (patch.object(publication.sys, 'argv', ['release-publish.py', 'publish']),
                      patch.dict(os.environ, {'PUBLICATION_OUTPUT': str(root/'output')}),
                      patch.object(publication, 'request', return_value=r),
                      patch.object(publication.s, 'outside', side_effect=Path),
                      patch.object(publication, 'gate'),
                      patch.object(publication, 'distribution', return_value=({}, {}, {}, {})),
                      patch.object(publication.pub, 'operate', side_effect=operate),
                      patch.object(registry, 'publish', side_effect=promote),
                      patch.object(publication, 'consumer', side_effect=consume)):
                    if fail:
                        with self.assertRaises(registry.RegistryError): publication.main()
                    else:
                        publication.main()
                proof = json.loads((root/'output/PUBLICATION-RESULT.json').read_text())
                self.assertEqual(events[:3], ['github-stage', 'github-publish', 'registry'])
                self.assertEqual(proof['github'], 'published_and_verified')
                self.assertEqual(proof['status'], 'incomplete' if fail else 'complete')
                self.assertEqual('consumer' in events, not fail)


class PolicyTests(unittest.TestCase):
    def test_real_custody_metadata_gate_supports_legacy_and_automatic_modes(self):
        spec = importlib.util.spec_from_file_location('custody_policy_test', HERE/'release-evidence-preflight.py')
        custody = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(custody)
        e = {'source': 'c'*40, 'tag': 'v1.2.3-alpha.4', 'run': '11', 'attempt': '1',
             'artifact': '12', 'digest': 'b'*64}
        run = {'id': 11, 'run_attempt': 1, 'head_sha': e['source'], 'head_branch': e['tag'],
            'event': 'workflow_dispatch', 'path': '.github/workflows/release.yml',
            'repository': {'id': 1409587171, 'full_name': 'bohewu/yetlet'},
            'head_repository': {'full_name': 'bohewu/yetlet'}, 'status': 'completed', 'conclusion': 'success'}
        artifact = {'id': 12, 'name': 'encrypted-private-'+e['source']+'-11-1', 'expired': False,
            'size_in_bytes': 1, 'digest': 'sha256:'+e['digest'], 'workflow_run': {'id': 11,
            'head_sha': e['source'], 'repository_id': 1409587171, 'head_repository_id': 1409587171}}
        environment = {'name': custody.ENVIRONMENT, 'id': 14, 'can_admins_bypass': False,
            'deployment_branch_policy': {'protected_branches': False, 'custom_branch_policies': True},
            'protection_rules': []}
        policies = {'total_count': 1, 'branch_policies': [{'name': e['tag'], 'type': 'tag'}]}
        reviewer = {'type': 'required_reviewers', 'reviewers': [{'type': 'User',
            'reviewer': {'login': 'bohewu', 'id': 35250002}}]}
        with patch.dict(os.environ, {'RELEASE_AUTOMATION': 'single-publication-v1'}):
            custody.check_metadata(run, artifact, environment, policies, e)
            wrong = copy.deepcopy(policies)
            wrong['branch_policies'][0]['name'] = '*'
            with self.assertRaises(ValueError): custody.check_metadata(run, artifact, environment, wrong, e)
            environment['protection_rules'] = [reviewer]
            with self.assertRaises(ValueError): custody.check_metadata(run, artifact, environment, policies, e)
        with patch.dict(os.environ, {}, clear=True):
            custody.check_metadata(run, artifact, environment, policies, e)
            environment['protection_rules'] = []
            with self.assertRaises(ValueError): custody.check_metadata(run, artifact, environment, policies, e)

    def test_automatic_workers_and_single_owner_publication_gate(self):
        worker = {'protection_rules': []}
        owner = {'protection_rules': [{'type': 'required_reviewers', 'prevent_self_review': False,
            'reviewers': [{'type': 'User', 'reviewer': {'login': 'bohewu', 'id': 35250002}}]}]}
        with patch.dict(os.environ, {'RELEASE_AUTOMATION': 'single-publication-v1'}):
            policy.reviewers(worker)
            policy.reviewers(owner, publication=True)
            with self.assertRaises(ValueError): policy.reviewers(worker, publication=True)
            with self.assertRaises(ValueError): policy.reviewers(owner)

    def test_automation_signing_authority_is_explicit_not_a_legacy_manual_claim(self):
        s = publication.s
        r = record()
        s.bind_record(r)
        request = {'mode': 'sign', 'signer_commit': r['signer_commit'], 'run': '1',
            'attempt': '1', 'artifact': '2', 'digest': 'a' * 64, 'receipt_sha256': 'b' * 64,
            'provenance_sha256': r['provenance']['sha256'], 'manifest_sha256': 'c' * 64,
            'd6': 'reviewed-automation'}
        with patch.dict(os.environ, {'RELEASE_AUTOMATION': 'single-publication-v1'}):
            s.validate_request(request)
            request['d6'] = 'approve-exact-signing-inputs'
            with self.assertRaises(ValueError): s.validate_request(request)

    def test_combined_plan_cannot_be_substituted_for_a_legacy_github_only_plan(self):
        publication.s.bind_record(record())
        plan = {'schema': 2, 'record_sha256': records.digest(record()), 'release_id': '15',
            'name': 'Fixture', 'notes_sha256': 'a'*64, 'verification_sha256': 'b'*64,
            'approval_sha256': 'c'*64, 'registry_plan_sha256': 'd'*64,
            'publication_request_sha256': 'e'*64, 'assets': [
                {'name': n, 'size': 1, 'sha256': 'f'*64} for n in sorted(publication.pub.names())]}
        publication.pub.plan(plan)
        del plan['registry_plan_sha256']
        with self.assertRaises(ValueError): publication.pub.plan(plan)

    def test_independent_export_never_includes_private_evidence(self):
        spec = importlib.util.spec_from_file_location('export_test', HERE / 'release-result-verify.py')
        verifier = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(verifier)
        verifier.s.bind_record(record())
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            (root / 'assembly').mkdir()
            (root / 'results').mkdir()
            for n in verifier.s.PACKAGES:
                (root / 'assembly' / (n+'-'+record()['version']+'.crate')).write_bytes(b'Fixture')
            (root / 'assembly/PROVENANCE-UNSIGNED.json').write_bytes(b'{}')
            (root / 'assembly/PRIVATE.txt').write_bytes(b'Private fixture')
            (root / 'assembly/RELEASE-CHECKSUMS.json').write_bytes(b'Private full manifest fixture')
            (root / 'results/provenance.sigstore.json').write_bytes(b'{}')
            verifier.export_public(root, root / 'public')
            self.assertEqual(len(list((root / 'public').iterdir())), 19)
            self.assertFalse((root / 'public/PRIVATE.txt').exists())
            manifest = json.loads((root / 'public/RELEASE-CHECKSUMS.json').read_text())
            self.assertEqual(len(manifest['artifacts']), 17)
            self.assertNotIn('PRIVATE.txt', {x['path'] for x in manifest['artifacts']})

    def test_ci_requires_both_real_jobs_on_exact_source(self):
        source = 'a' * 40
        run = {'id': 1, 'run_attempt': 1, 'head_sha': source, 'head_branch': 'master',
            'event': 'push', 'status': 'completed', 'conclusion': 'success', 'repository': {'id': 1409587171}}
        jobs = {'total_count': 2, 'jobs': [{'name': n, 'status': 'completed', 'conclusion': 'success'}
            for n in ('Rust 1.99 MSRV', 'Stable native, samples, and Chrome')]}
        api = lambda path: {'workflow_runs': [run]} if 'workflows' in path else jobs
        self.assertEqual(policy.ci(source, api)['source'], source)
        jobs['jobs'][1]['conclusion'] = 'skipped'
        with self.assertRaises(ValueError): policy.ci(source, api)

    def test_publication_cannot_run_without_actual_owner_approval(self):
        r = {'publication_commit': 'a' * 40, 'verifier_commit': 'd' * 40,
            'verifier_run': '21', 'verifier_attempt': '1', 'artifact': '22', 'artifact_sha256': 'e' * 64}
        environment = {'name': 'release-publish', 'id': 7, 'can_admins_bypass': False,
            'protection_rules': [{'type': 'required_reviewers', 'prevent_self_review': False,
                'reviewers': [{'type': 'User', 'reviewer': {'login': 'bohewu', 'id': 35250002}}]}],
            'deployment_branch_policy': {'protected_branches': False, 'custom_branch_policies': True}}
        policies = {'total_count': 1, 'branch_policies': [{'name': 'master', 'type': 'branch'}]}
        run = {'id': 21, 'run_attempt': 1, 'head_sha': r['verifier_commit'], 'head_branch': 'master',
            'event': 'workflow_dispatch', 'path': '.github/workflows/release-result-verify.yml',
            'repository': {'id': 1409587171}, 'head_repository': {'id': 1409587171},
            'status': 'completed', 'conclusion': 'success'}
        artifact = {'id': 22, 'name': 'verified-publication-21-1', 'expired': False,
            'digest': 'sha256:' + r['artifact_sha256'], 'size_in_bytes': 1,
            'workflow_run': {'id': 21, 'head_sha': r['verifier_commit'], 'repository_id': 1409587171,
                             'head_repository_id': 1409587171}}
        approvals = []
        def api(path):
            if path == '/branches/master': return {'protected': True, 'commit': {'sha': r['publication_commit']}}
            if path.endswith('/approvals'): return approvals
            if '/deployment-branch-policies' in path: return policies
            if path.startswith('/environments/'): return environment
            if path.startswith('/actions/artifacts/'): return artifact
            return run
        env = {'RELEASE_AUTOMATION': 'single-publication-v1', 'GITHUB_EVENT_NAME': 'workflow_dispatch',
            'GITHUB_REPOSITORY': 'bohewu/yetlet', 'GITHUB_REPOSITORY_ID': '1409587171',
            'GITHUB_REPOSITORY_OWNER_ID': '35250002', 'GITHUB_REF': 'refs/heads/master',
            'GITHUB_SHA': r['publication_commit'], 'PUBLICATION_WORKFLOW_SHA': r['publication_commit'],
            'PUBLICATION_WORKFLOW_REF': 'bohewu/yetlet/.github/workflows/release-publish.yml@refs/heads/master',
            'GITHUB_RUN_ID': '31', 'GITHUB_RUN_ATTEMPT': '1'}
        with (patch.dict(os.environ, env, clear=True), patch.object(publication.subprocess, 'check_output',
                return_value=r['publication_commit']+'\n'), patch.object(publication.pub, 'api', side_effect=api),
                patch.object(policy, 'ci')):
            publication.gate(r, False)
            with self.assertRaises(ValueError): publication.gate(r, True)
            approvals.append({'state': 'approved', 'user': {'id': 35250002}, 'environments': [{'id': 7}]})
            publication.gate(r, True)


if __name__ == '__main__':
    unittest.main()
