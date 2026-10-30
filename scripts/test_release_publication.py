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
import zipfile

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
    def test_publish_success_uses_official_optional_warnings_not_krate(self):
        for response in ({}, {'warnings': None}, {'warnings': {}},
                         {'crate': {'name': 'fixture'}, 'warnings': {'other': []}},
                         {'warnings': {'invalid_categories': [], 'invalid_badges': [], 'other': ['private warning']},
                          'future_field': True}):
            counts = registry.publish_response(records.canonical(response))
            self.assertEqual(set(counts), {'invalid_categories', 'invalid_badges', 'other'})
            self.assertNotIn('private', json.dumps(counts))
        self.assertEqual(counts['other'], 1)
        for response in ([], {'errors': [{'detail': 'private error'}]}, {'errors': {}},
                         {'warnings': []}, {'warnings': {'other': 'bad'}},
                         {'warnings': {'invalid_categories': [False]}}):
            with self.subTest(response=response), self.assertRaises(ValueError):
                registry.publish_response(records.canonical(response))

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
            registry.request(registry.OWNER_API)
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
                return records.canonical({'warnings': {'invalid_categories': [], 'invalid_badges': [], 'other': []}})
            def read(row, version, attempts):
                complete.add(row['name'])
                return {'index_sha256': row['sha256'], 'download_sha256': row['sha256']}
            with (patch.object(registry, 'request', side_effect=upload),
                  patch.object(registry, 'readback', side_effect=read)):
                result = registry.publish(plan, bodies, root/'receipt.json')
            self.assertEqual(len(complete), 16)
            self.assertEqual(result['status'], 'registry_verified_consumer_pending')


class RegistryContinuationTests(unittest.TestCase):
    def fixture(self, root):
        r, subjects = archives(root)
        plan, bodies = registry.prepare(r, root, subjects)
        selection = {'existing': ['yetlet-actions'], 'remaining': [x['name'] for x in plan['packages'][1:]]}
        return plan, bodies, selection

    def test_only_fifteen_original_bodies_are_sent_in_dependency_order(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            plan, bodies, selection = self.fixture(root)
            rows = {x['name']: x for x in plan['packages']}
            published, writes = {'yetlet-actions'}, []
            def remote(url, method='GET', body=None):
                if method == 'PUT':
                    size = struct.unpack_from('<I', body)[0]
                    name = json.loads(body[4:4+size])['name']
                    self.assertNotEqual(name, 'yetlet-actions')
                    self.assertNotIn(name, published)
                    self.assertEqual(body, bodies[name])
                    self.assertTrue(set(rows[name]['requires']) <= published)
                    writes.append(name); published.add(name)
                    return b'{"warnings":{"other":[]}}'
                if url == registry.OWNER_API:
                    return b'{"users":[{"login":"bohewu","kind":"user"}]}'
                if url.startswith('https://index.crates.io/'):
                    name = url.rsplit('/', 1)[1]
                    if name not in published: raise registry.RegistryError('http', 404)
                    return records.canonical({'name': name, 'vers': plan['version'],
                        'cksum': rows[name]['sha256'], 'yanked': False})
                name = url.split('/')[4]
                return (root / rows[name]['filename']).read_bytes()
            receipt = root / 'continuation.json'
            with patch.object(registry, 'request', side_effect=remote) as network:
                result = registry.publish(plan, bodies, receipt, selection=selection)
                self.assertEqual(writes, selection['remaining'])
                self.assertEqual(len(result['packages']), 15)
                self.assertEqual(result['existing'][0]['status'], 'verified_existing')
                self.assertTrue(all(e['status'] == 'verified' for e in result['packages']))
                calls = network.call_count
                with self.assertRaises(ValueError): registry.publish(plan, bodies, receipt, selection=selection)
                self.assertEqual(network.call_count, calls)

    def test_unknown_selection_is_rejected_before_network(self):
        with tempfile.TemporaryDirectory() as temporary:
            plan, _, good = self.fixture(Path(temporary))
            for bad in ({'existing': [], 'remaining': good['remaining']},
                        {'existing': ['yetlet-core'], 'remaining': good['remaining']},
                        dict(good, remaining=list(reversed(good['remaining']))),
                        dict(good, extra=True)):
                with patch.object(registry, 'request') as network, self.assertRaises(ValueError):
                    registry.reconcile(plan, bad)
                network.assert_not_called()

    def test_existing_drift_owner_change_or_target_presence_prevents_all_writes(self):
        for failure in ('content', 'owner', 'target'):
            with self.subTest(failure=failure), tempfile.TemporaryDirectory() as temporary:
                root = Path(temporary); plan, bodies, selection = self.fixture(root)
                def remote(url, method='GET', body=None):
                    self.assertEqual(method, 'GET')
                    if url == registry.OWNER_API:
                        return records.canonical({'users': [{'login': 'other' if failure == 'owner' else 'bohewu', 'kind': 'user'}]})
                    return records.canonical({'name': selection['remaining'][0], 'vers': plan['version']})
                with (patch.object(registry, 'readback', side_effect=ValueError('changed') if failure == 'content' else None,
                                   return_value={'index_sha256': 'a'*64, 'download_sha256': 'a'*64}),
                      patch.object(registry, 'request', side_effect=remote) as network):
                    with self.assertRaises((ValueError, registry.RegistryError)):
                        registry.publish(plan, bodies, root/'result.json', selection=selection)
                self.assertTrue(all(len(c.args) < 2 or c.args[1] != 'PUT' for c in network.call_args_list))
                proof = json.loads((root/'result.json').read_text())
                self.assertEqual(proof['packages'], [])
                self.assertEqual(proof['status'], 'incomplete')

    def test_newly_present_version_is_not_resent_after_initial_reconciliation(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary); plan, bodies, selection = self.fixture(root)
            count = 0
            def remote(url, method='GET', body=None):
                nonlocal count
                self.assertEqual(method, 'GET')
                if url == registry.OWNER_API: return b'{"users":[{"login":"bohewu","kind":"user"}]}'
                count += 1
                if count <= 15: raise registry.RegistryError('http', 404)
                return records.canonical({'name': selection['remaining'][0], 'vers': plan['version']})
            with patch.object(registry, 'readback', return_value={}), patch.object(registry, 'request', side_effect=remote):
                with self.assertRaises(registry.RegistryError) as caught:
                    registry.publish(plan, bodies, root/'result.json', selection=selection)
            self.assertEqual(caught.exception.kind, 'version_exists')
            self.assertEqual(json.loads((root/'result.json').read_text())['packages'], [])

    def test_uncertain_put_stops_with_one_new_intent_and_keeps_existing_separate(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary); plan, bodies, selection = self.fixture(root)
            def remote(url, method='GET', body=None):
                self.assertEqual(method, 'PUT')
                raise registry.RegistryError('transport')
            with (patch.object(registry, 'reconcile', return_value={'name': 'yetlet-actions', 'status': 'verified_existing'}),
                  patch.object(registry, 'require_absent'), patch.object(registry, 'request', side_effect=remote) as write):
                with self.assertRaises(registry.RegistryError): registry.publish(plan, bodies, root/'result.json', selection=selection)
            write.assert_called_once()
            proof = json.loads((root/'result.json').read_text())
            self.assertEqual(proof['existing'][0]['name'], 'yetlet-actions')
            self.assertEqual([(p['name'], p['status']) for p in proof['packages']], [(selection['remaining'][0], 'attempted')])
            self.assertEqual(proof['consumer'], 'not_attempted')


class PublicationTests(unittest.TestCase):
    def test_github_failure_keeps_safe_diagnostics_and_stops_registry(self):
        cases = [(publication.pub.UploadError('http', 411), 'upload', 'http', 411),
                 (publication.pub.ApiError('GET', 403), 'api-read', 'http', 403),
                 (ValueError('private URL/token/body'), 'validation', 'validation', None)]
        for error, operation, kind, status in cases:
            with self.subTest(operation=operation), tempfile.TemporaryDirectory() as temporary:
                output = Path(temporary) / 'output'
                publication.s.bind_record(record())
                stderr = io.StringIO()
                with (patch.object(publication.sys, 'argv', ['release-publish.py', 'publish']),
                      patch.dict(os.environ, {'PUBLICATION_OUTPUT': str(output)}),
                      patch.object(publication, 'request', return_value={'notes': 'Fixture'}),
                      patch.object(publication.s, 'outside', side_effect=Path),
                      patch.object(publication, 'gate'),
                      patch.object(publication, 'distribution', return_value=({}, {}, {}, {})),
                      patch.object(publication.pub, 'operate', side_effect=error) as github,
                      patch.object(registry, 'publish') as promote,
                      patch.object(publication, 'consumer') as consumer,
                      patch.object(publication.sys, 'stderr', stderr)):
                    with self.assertRaises(type(error)): publication.main()
                github.assert_called_once()
                promote.assert_not_called()
                consumer.assert_not_called()
                proof = json.loads((output / 'PUBLICATION-RESULT.json').read_text())
                self.assertEqual(proof['failure'], {'phase': 'github-stage', 'operation': operation,
                    'class': kind, 'http_status': status})
                self.assertEqual((proof['status'], proof['github'], proof['registry']),
                    ('incomplete', 'stage_attempted', 'not_attempted'))
                self.assertFalse(proof['automatic_retry'])
                self.assertNotIn('private', json.dumps(proof) + stderr.getvalue())

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


class PublicationContinuationTests(unittest.TestCase):
    def fixture(self, root):
        rec, subjects = archives(root)
        reg, bodies = registry.prepare(rec, root, subjects)
        publication.s.bind_record(rec)
        old = {'publication_commit': 'e'*40, 'verifier_commit': 'd'*40, 'verifier_run': '21',
            'verifier_attempt': '1', 'artifact': '22', 'artifact_sha256': 'a'*64,
            'verification_sha256': 'b'*64, 'name': 'Fixture', 'notes': 'Fixture notes',
            'notes_sha256': registry.sha(b'Fixture notes'), 'registry_plan_sha256': records.digest(reg)}
        p = {'schema': 2, 'record_sha256': records.digest(rec), 'release_id': rec['release_id'],
            'name': old['name'], 'notes_sha256': old['notes_sha256'], 'verification_sha256': old['verification_sha256'],
            'approval_sha256': 'a'*64, 'registry_plan_sha256': records.digest(reg),
            'publication_request_sha256': records.digest(old), 'assets': [
                {'name': n, 'size': 1, 'sha256': 'a'*64} for n in sorted(publication.pub.names())]}
        github = {'schema': 1, 'action': 'publish', 'record_sha256': records.digest(rec),
            'plan_sha256': records.digest(p), 'release_id': rec['release_id'], 'source': rec['source'],
            'tag': 'v'+rec['version'], 'notes_sha256': old['notes_sha256'],
            'assets': [dict(a, id=i+1) for i,a in enumerate(p['assets'])],
            'automatic_retry': False, 'registry_upload': False}
        prior = {'PUBLICATION-PLAN.json': p, 'REGISTRY-PLAN.json': reg, 'GITHUB-PUBLICATION.json': github,
            'PUBLICATION-RESULT.json': {'schema': 1, 'status': 'incomplete', 'github': 'published_and_verified',
                'registry': 'attempted', 'automatic_retry': False, 'record_sha256': records.digest(rec),
                'publication_request_sha256': records.digest(old), 'failure': {'phase': 'registry-publish',
                    'operation': 'validation', 'class': 'validation', 'http_status': None}},
            'REGISTRY-RESULT.json': {'schema': 1, 'status': 'incomplete', 'plan_sha256': records.digest(reg),
                'packages': [{'name': 'yetlet-actions', 'status': 'attempted', 'sha256': reg['packages'][0]['sha256']}],
                'consumer': 'not_attempted', 'error': {'class': 'validation', 'http_status': None}}}
        request = dict(old, publication_commit='f'*40, continuation={'run': '31', 'attempt': '1',
            'artifact': '32', 'sha256': 'c'*64, 'existing': ['yetlet-actions'],
            'remaining': [r['name'] for r in reg['packages'][1:]]})
        return rec, request, old, p, reg, bodies, github, prior

    def prior_api(self, request, raw):
        c = request['continuation']
        run = {'id': 31, 'run_attempt': 1, 'head_sha': 'e'*40, 'head_branch': 'master',
            'event': 'workflow_dispatch', 'path': '.github/workflows/release-publish.yml',
            'status': 'completed', 'conclusion': 'failure', 'repository': {'id': 1409587171},
            'head_repository': {'id': 1409587171}}
        artifact = {'id': 32, 'name': 'publication-result-31-1', 'expired': False,
            'digest': 'sha256:'+registry.sha(raw), 'size_in_bytes': len(raw),
            'workflow_run': {'id': 31, 'head_sha': 'e'*40, 'repository_id': 1409587171, 'head_repository_id': 1409587171}}
        c['sha256'] = registry.sha(raw)
        return {'/actions/runs/31/attempts/1': run, '/actions/artifacts/32': artifact,
            '/environments/release-publish': {'id': 44}, '/actions/runs/31/approvals': [
                {'state': 'approved', 'user': {'id': 35250002}, 'environments': [{'id': 44}]}]}

    def prior_zip(self, prior):
        raw = io.BytesIO()
        with zipfile.ZipFile(raw, 'w') as archive:
            for name, value in prior.items(): archive.writestr(name, records.canonical(value))
        return raw.getvalue()

    def test_closed_continuation_request_rejects_other_subsets_and_extra_fields(self):
        with tempfile.TemporaryDirectory() as temporary:
            rec, r, *_ = self.fixture(Path(temporary))
            with patch.dict(os.environ, {'RELEASE_RECORD': json.dumps(rec), 'PUBLICATION_REQUEST': json.dumps(r)}):
                self.assertEqual(publication.request(), r)
            for field, value in [('existing', []), ('remaining', r['continuation']['remaining'][:-1]),
                                  ('remaining', ['yetlet-actions']*15), ('attempt', '0'), ('extra', True)]:
                bad = copy.deepcopy(r); bad['continuation'][field] = value
                with patch.dict(os.environ, {'RELEASE_RECORD': json.dumps(rec), 'PUBLICATION_REQUEST': json.dumps(bad)}):
                    with self.assertRaises(ValueError): publication.request()

    def test_original_failed_artifact_and_request_are_bound_without_rewriting_receipts(self):
        for corruption in (None, 'request', 'prefix', 'approval', 'run'):
            with self.subTest(corruption=corruption), tempfile.TemporaryDirectory() as temporary:
                root = Path(temporary); rec, r, old, p, reg, bodies, github, prior = self.fixture(root)
                if corruption == 'request': prior['PUBLICATION-RESULT.json']['publication_request_sha256'] = '0'*64
                if corruption == 'prefix': prior['REGISTRY-RESULT.json']['packages'].append({'name': 'yetlet-core'})
                raw = self.prior_zip(prior); api = self.prior_api(r, raw)
                if corruption == 'approval': api['/actions/runs/31/approvals'][0]['user']['id'] = 1
                if corruption == 'run': api['/actions/runs/31/attempts/1']['conclusion'] = 'success'
                work = root/'work'; work.mkdir()
                with (patch.object(publication.pub, 'api', side_effect=lambda path: api[path]),
                      patch.object(publication.s, 'download', return_value=raw),
                      patch.object(publication, 'distribution', return_value=(p, {}, reg, bodies)) as derive,
                      patch.object(publication.pub, 'operate') as mutate):
                    if corruption:
                        with self.assertRaises(ValueError): publication.continuation_distribution(r, work)
                    else:
                        actual = publication.continuation_distribution(r, work)
                        self.assertEqual(actual, (p, {}, reg, bodies, github))
                        self.assertEqual(derive.call_args.args[0], old)
                        for name, value in prior.items():
                            self.assertEqual((work/'prior'/name).read_bytes(), records.canonical(value))
                mutate.assert_not_called()

    def test_published_asset_identity_and_content_drift_is_rejected(self):
        with tempfile.TemporaryDirectory() as temporary:
            _, _, _, p, _, _, previous, _ = self.fixture(Path(temporary))
            assets = [dict(a, state='uploaded', digest='sha256:'+a['sha256']) for a in previous['assets']]
            for corruption in (None, 'id', 'digest'):
                actual = copy.deepcopy(assets)
                if corruption == 'id': actual[0]['id'] += 100
                if corruption == 'digest': actual[0]['digest'] = 'sha256:'+'0'*64
                with patch.object(publication.pub, 'release', return_value={'assets': actual}), \
                        patch.object(publication.pub, 'operate') as mutate:
                    if corruption:
                        with self.assertRaises(ValueError): publication.published_release(p, previous)
                    else:
                        self.assertFalse(publication.published_release(p, previous)['github_mutations'])
                mutate.assert_not_called()

    def test_registry_only_branch_never_stages_or_publishes_github(self):
        for mode in ('preflight', 'publish'):
            with self.subTest(mode=mode), tempfile.TemporaryDirectory() as temporary:
                root = Path(temporary); _, r, _, p, reg, bodies, previous, _ = self.fixture(root)
                selection = {k:r['continuation'][k] for k in ('existing', 'remaining')}
                with (patch.object(publication.sys, 'argv', ['release-publish.py', mode]),
                      patch.dict(os.environ, {'PUBLICATION_OUTPUT': str(root/'output'),
                                             'GITHUB_STEP_SUMMARY': str(root/'summary')}),
                      patch.object(publication, 'request', return_value=r),
                      patch.object(publication.s, 'outside', side_effect=Path), patch.object(publication, 'gate'),
                      patch.object(publication, 'continuation_distribution', return_value=(p, {}, reg, bodies, previous)),
                      patch.object(publication, 'published_release', return_value={'github_mutations': False}),
                      patch.object(registry, 'reconcile', return_value={'name': 'yetlet-actions'}),
                      patch.object(registry, 'publish', return_value={'status': 'registry_verified_consumer_pending'}) as upload,
                      patch.object(publication, 'consumer', return_value={'status': 'PASS'}) as consumer,
                      patch.object(publication.pub, 'operate') as mutate):
                    publication.main()
                mutate.assert_not_called()
                proof = json.loads((root/'output/PUBLICATION-RESULT.json').read_text())
                self.assertFalse((root/'output/GITHUB-PUBLICATION.json').exists())
                if mode == 'publish':
                    self.assertEqual(upload.call_args.kwargs, {'selection': selection})
                    consumer.assert_called_once_with(record()['version'], root/'output')
                    self.assertEqual(proof['github'], 'existing_public_release_verified')
                    self.assertEqual(proof['status'], 'complete')
                    self.assertTrue((root/'output/GITHUB-PRESERVED.json').is_file())
                else:
                    upload.assert_not_called(); consumer.assert_not_called()
                    self.assertEqual(proof['status'], 'preflight_pass')
                    self.assertEqual((proof['github'], proof['registry']), ('not_attempted', 'not_attempted'))

    def test_new_owner_approval_and_readonly_job_are_required(self):
        with tempfile.TemporaryDirectory() as temporary:
            _, r, *_ = self.fixture(Path(temporary))
            env = {'GITHUB_EVENT_NAME': 'workflow_dispatch', 'GITHUB_REPOSITORY': 'bohewu/yetlet',
                'GITHUB_REPOSITORY_ID': '1409587171', 'GITHUB_REPOSITORY_OWNER_ID': '35250002',
                'GITHUB_REF': 'refs/heads/master', 'GITHUB_SHA': r['publication_commit'],
                'PUBLICATION_WORKFLOW_SHA': r['publication_commit'],
                'PUBLICATION_WORKFLOW_REF': 'bohewu/yetlet/.github/workflows/release-publish.yml@refs/heads/master',
                'GITHUB_RUN_ATTEMPT': '1', 'GITHUB_RUN_ID': '41', 'GITHUB_JOB': 'registry-continuation'}
            for job, owner in [('publish', 35250002), ('registry-continuation', 1), ('registry-continuation', 35250002)]:
                api = {'/branches/master': {'protected': True, 'commit': {'sha': r['publication_commit']}},
                    '/environments/release-publish': {'id': 44},
                    '/actions/runs/41/approvals': [{'state': 'approved', 'user': {'id': owner}, 'environments': [{'id':44}]}]}
                with (patch.dict(os.environ, dict(env, GITHUB_JOB=job), clear=True),
                      patch.object(publication.policy, 'automatic', return_value=True),
                      patch.object(publication.policy, 'ci'), patch.object(publication.s, 'environment_gate'),
                      patch.object(publication.s, 'run_gate'), patch.object(publication.s, 'artifact_gate'),
                      patch.object(publication.subprocess, 'check_output', return_value=r['publication_commit']),
                      patch.object(publication.pub, 'api', side_effect=lambda path: api.get(path, {}))):
                    if job == 'registry-continuation' and owner == 35250002: publication.gate(r, True)
                    else:
                        with self.assertRaises(ValueError): publication.gate(r, True)


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
