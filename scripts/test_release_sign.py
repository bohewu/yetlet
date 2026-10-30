#!/usr/bin/env python3
"""Focused tests of new signer boundaries; synthetic bytes, no official signing."""
import base64
import copy
import hashlib
import importlib.util
import io
import json
import os
from pathlib import Path
import tarfile
import tempfile
import unittest
import urllib.error
from unittest.mock import patch
import zipfile

spec = importlib.util.spec_from_file_location("release_sign", Path(__file__).with_name("release-sign.py"))
s = importlib.util.module_from_spec(spec)
spec.loader.exec_module(s)
S = "a" * 40


# Retained alpha test data, now explicit rather than command defaults.
FIXTURE_RECORD = {
    'schema': 1, 'version': '0.9.0-alpha.1', 'source': 'bf3363801132f4481263ceb411d9bba3349df86d',
    'cohort': 'sixteen', 'build': {'run': '40669449040', 'attempt': '1', 'artifact': '12807757693',
        'sha256': 'a09c7e3e32d254a702c171333bfa01a1d08c2ed96f36eb5463ee9745a66e3dca'},
    'custody_verifier': {'run': '40688772221', 'attempt': '1', 'environment_id': '25525950662'},
    'signer_commit': S, 'provenance': {'sha256': '4d4de1a8ce9cc077c0aa62a1aafb3ab9799eed312deddb66d9e6f8fc6fef2d68',
        'size': 9158}, 'release_id': '426670086'}


def prepare_request():
    raw = '-----BEGIN AGE ENCRYPTED FILE-----\nTE9DQUw=\n-----END AGE ENCRYPTED FILE-----\n'
    return {"mode": "prepare", "signer_commit": S, "qualification_asset": "1",
            "qualification_sha256": "b" * 64, "qualification_manifest_sha256": "c" * 64,
            "qualification_handoff": raw, "qualification_handoff_sha256": s.sha(raw.encode()),
            "qualification_expires": str(int(s.time.time()) + 2400)}


def sign_request():
    return {"mode": "sign", "signer_commit": S, "run": "2", "attempt": "1", "artifact": "3",
            "digest": "d" * 64, "receipt_sha256": "e" * 64, "provenance_sha256": s.PUBLIC_SHA,
            "manifest_sha256": "f" * 64, "d6": "approve-exact-signing-inputs"}


class SignerTests(unittest.TestCase):
    def setUp(self):
        s.bind_record(copy.deepcopy(FIXTURE_RECORD))
        env = patch.dict(os.environ, {'RELEASE_RECORD': json.dumps(FIXTURE_RECORD)})
        env.start()
        self.addCleanup(env.stop)

    def test_age_bootstrap_owned_module_cache_is_removable(self):
        go = os.environ.get('YETLET_CLEANUP_TEST_GO')
        proxy = os.environ.get('YETLET_CLEANUP_TEST_PROXY')
        if not go or not proxy:
            self.skipTest('requires the retained authenticated Go and offline module fixture')
        fixture = Path(proxy)/'sigsum.org/sigsum-go/@v/v0.13.1.zip'
        self.assertEqual(s.sha(fixture.read_bytes()), '747d88da11131cd3e26841965e8e3b95057a6cfabda993dc0f43cd913512479a')
        script = (Path(__file__).with_name('bootstrap-release-age.sh')).read_text()
        flags = s.re.search(r'GOFLAGS=(\S*) GOWORK=off', script).group(1)
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            for label, flag, should_remove in [('original', '', False), ('configured', flags, True)]:
                cache = root/label
                env = dict(os.environ, GOTOOLCHAIN='local', GOENV='off', GOWORK='off', GOFLAGS=flag,
                           GOMODCACHE=str(cache), GOPATH=str(root/'gopath'), GOCACHE=str(root/'gocache'),
                           GOPROXY=Path(proxy).as_uri(), GOSUMDB='off', GOPRIVATE='', GONOPROXY='', GONOSUMDB='')
                try:
                    s.subprocess.run([go, 'mod', 'download', '-json', 'sigsum.org/sigsum-go@v0.13.1'],
                                     env=env, check=True, stdout=s.subprocess.PIPE, stderr=s.subprocess.PIPE, timeout=30)
                    removed = s.subprocess.run(['rm', '-rf', str(cache)], capture_output=True, timeout=30)
                    self.assertEqual(removed.returncode == 0, should_remove, label)
                    self.assertEqual(not cache.exists(), should_remove, label)
                finally:
                    if cache.exists():
                        s.subprocess.run([go, 'clean', '-modcache'], env=env, check=True,
                                         stdout=s.subprocess.PIPE, stderr=s.subprocess.PIPE, timeout=30)

    def handoff_fixture(self, seconds=2400):
        r = prepare_request()
        r["qualification_expires"] = str(int(s.time.time()) + seconds)
        claims = base64.urlsafe_b64encode(json.dumps({'exp': int(r['qualification_expires'])}).encode()).decode().rstrip('=')
        expires = s.datetime.fromtimestamp(int(r['qualification_expires']), s.datetime.now().astimezone().tzinfo).astimezone(__import__('datetime').timezone.utc).isoformat()
        query = s.urllib.parse.urlencode({'se': expires, 'sp': 'r', 'spr': 'https', 'jwt': 'header.' + claims + '.signature'})
        url = 'https://release-assets.githubusercontent.com/github-production-release-asset/1409587171/LOCAL?' + query
        p = {'schema': 1, 'source': s.C, 'release': s.RELEASE_ID, 'signer_commit': r['signer_commit'],
             'asset': r['qualification_asset'], 'zip_sha256': r['qualification_sha256'],
             'manifest_sha256': r['qualification_manifest_sha256'], 'expires': r['qualification_expires'], 'url': url}
        return r, p

    def test_handoff_rejects_plaintext_tampering_and_expiry(self):
        r, payload = self.handoff_fixture()
        s.handoff_input(r); s.handoff_url(payload['url'], r['qualification_expires'])
        waiting = dict(r); waiting['qualification_expires'] = str(int(s.time.time()) + 600)
        s.handoff_input(waiting, minimum_seconds=300)
        with self.assertRaises(ValueError): s.handoff_input(waiting, minimum_seconds=1200)
        for key, value in [('qualification_handoff', payload['url']),
                           ('qualification_handoff_sha256', '0' * 64),
                           ('qualification_expires', str(int(s.time.time()) - 1))]:
            bad = dict(r); bad[key] = value
            with self.subTest(key=key), self.assertRaises(ValueError): s.handoff_input(bad)
        for url in [payload['url'].replace('https:', 'http:', 1),
                    payload['url'].replace('release-assets.githubusercontent.com', '127.0.0.1'),
                    payload['url'].replace('release-assets.githubusercontent.com', 'release-assets.githubusercontent.com.evil.invalid'),
                    payload['url'].replace('/1409587171/', '/7/'),
                    payload['url'].replace('sp=r&', 'sp=rw&'), payload['url'] + '#PRIVATE-FRAGMENT']:
            with self.subTest(url=url), self.assertRaises(ValueError): s.handoff_url(url, r['qualification_expires'])

    def test_handoff_download_has_no_auth_redirect_and_requires_exact_zip(self):
        r, payload = self.handoff_fixture(); raw = b'LOCAL encrypted ZIP'; r['qualification_sha256'] = s.sha(raw)
        response = io.BytesIO(raw); response.status = 200
        with patch.object(s.urllib.request, 'build_opener') as opener, patch.dict(os.environ, {'GH_TOKEN': 'LOCAL-NEVER-FORWARD'}):
            opener.return_value.open.return_value = response
            self.assertEqual(s.qualification_download(payload['url'], r), raw)
            self.assertIsInstance(opener.call_args.args[0], s.api.NoRedirect)
            req = opener.return_value.open.call_args.args[0]
            self.assertEqual(req.headers, {'Accept': 'application/octet-stream'})
            response = io.BytesIO(b'LOCAL tampered ZIP'); response.status = 200
            opener.return_value.open.return_value = response
            with self.assertRaises(ValueError): s.qualification_download(payload['url'], r)
            private = io.BytesIO(b'PRIVATE-RESPONSE')
            opener.return_value.open.side_effect = urllib.error.HTTPError(payload['url'], 302, 'PRIVATE-REDIRECT', {}, private)
            with self.assertRaises(urllib.error.HTTPError): s.qualification_download(payload['url'], r)
            self.assertTrue(private.closed)

    def test_handoff_receive_binds_metadata_cleans_plaintext_and_stops_bad_source(self):
        r, payload = self.handoff_fixture()
        with tempfile.TemporaryDirectory() as t:
            root = Path(t)
            def decrypt(args): s.json_write(Path(args[args.index('--output') + 1]), payload)
            with patch.object(s, 'command', side_effect=decrypt), patch.object(s, 'qualification_download', return_value=b'LOCAL ZIP') as fetch, \
                    patch.object(s, 'unpack_zip') as unpack, patch.object(s, 'qualification_transport'), \
                    patch.dict(os.environ, {'SIGN_AGE': 'LOCAL synthetic age'}):
                s.receive_qualification(r, root)
                unpack.assert_called_once(); fetch.assert_called_once()
                self.assertFalse((root/'qualification-capability.json').exists())
                self.assertFalse((root/'qualification-handoff.age').exists())
                payload['source'] = '0' * 40; fetch.reset_mock()
                with self.assertRaises(s.GateRejected) as error: s.receive_qualification(r, root)
                self.assertEqual(error.exception.stage, 'qualification-handoff')
                fetch.assert_not_called()
                self.assertFalse((root/'qualification-capability.json').exists())

    def test_owner_handoff_uses_fixed_host_read_only_and_removes_capability(self):
        r, payload = self.handoff_fixture(seconds=299)
        cipher = b'LOCAL encrypted qualification'; receipt = {'schema': 1, 'source': s.C,
            'manifest_sha256': r['qualification_manifest_sha256'], 'ciphertext_sha256': s.sha(cipher)}
        archive = io.BytesIO()
        with zipfile.ZipFile(archive, 'w') as z:
            z.writestr('qualification.tar.age', cipher); z.writestr('qualification-receipt.json', json.dumps(receipt))
        raw = archive.getvalue(); r['qualification_sha256'] = s.sha(raw)
        template = {k: v for k, v in r.items() if k not in {'qualification_handoff', 'qualification_handoff_sha256', 'qualification_expires'}}
        responses = [
            {'protected': True, 'commit': {'sha': S}},
            {'draft': True, 'tag_name': s.TAG, 'target_commitish': s.C},
            [{'id': 1, 'name': 'qualification-inputs.zip', 'digest': 'sha256:' + r['qualification_sha256'], 'size': 1}]]
        native = []
        def encrypt(args):
            native.append(args); target = Path(args[args.index('--output') + 1]); target.write_text(r['qualification_handoff'])
        def open_read(req, **kwargs):
            self.assertEqual(req.get_method(), 'GET'); self.assertTrue(req.full_url.startswith(s.api.API + '/'))
            self.assertEqual(req.get_header('Authorization'), 'Bearer LOCAL-OWNER')
            if responses: return io.BytesIO(json.dumps(responses.pop(0)).encode())
            raise urllib.error.HTTPError(req.full_url, 302, 'LOCAL redirect', {'Location': payload['url']}, io.BytesIO())
        # BytesIO fixtures carry the same public 200 status as provider responses.
        def successful_read(req, **kwargs):
            out = open_read(req, **kwargs); out.status = 200; return out
        with tempfile.TemporaryDirectory() as t:
            p = Path(t); repo = p/'repo'; (repo/'.github').mkdir(parents=True); (repo/'scripts').mkdir()
            recipient = b'LOCAL public recipient\n'; (repo/'.github/release-evidence-recipient.txt').write_bytes(recipient)
            age = p/'age'; age.write_bytes(b'LOCAL tool'); s.json_write(p/'template.json', template)
            with patch.object(s,'HERE',repo/'scripts'),patch.object(s,'RECIPIENT_SHA',s.sha(recipient)),patch.object(s,'AGE_SHA',s.sha(age.read_bytes())), \
                    patch.object(s.subprocess,'check_output',return_value='LOCAL-OWNER\n') as auth,patch.object(s.urllib.request,'build_opener') as opener, \
                    patch.object(s,'qualification_download',return_value=raw) as fetch,patch.object(s,'command',side_effect=encrypt), \
                    patch.dict(os.environ,{'SIGN_AGE':str(age)}):
                opener.return_value.open.side_effect = successful_read
                s.make_handoff(p/'template.json', p/'output')
                self.assertEqual(auth.call_args.args[0], ['gh','auth','token','--hostname','github.com'])
                fetch.assert_called_once(); self.assertEqual(len(native), 1)
                self.assertEqual(sorted(x.name for x in (p/'output').iterdir()), ['HANDOFF-RECEIPT.json','request.json'])
                output=s.load((p/'output/request.json').read_bytes());s.handoff_input(output)
                self.assertNotIn(payload['url'], (p/'output/request.json').read_text())

    def test_qualification_transport_rejects_wrong_manifest_before_private_decrypt(self):
        r = prepare_request()
        with tempfile.TemporaryDirectory() as t:
            p = Path(t); (p/'qualification.tar.age').write_bytes(b'LOCAL cipher')
            receipt = {'schema': 1, 'source': s.C, 'manifest_sha256': r['qualification_manifest_sha256'],
                       'ciphertext_sha256': s.sha(b'LOCAL cipher')}
            s.json_write(p/'qualification-receipt.json', receipt); s.qualification_transport(r, p)
            receipt['manifest_sha256'] = '0' * 64
            (p/'qualification-receipt.json').unlink(); s.json_write(p/'qualification-receipt.json', receipt)
            with self.assertRaises(ValueError): s.qualification_transport(r, p)

    def test_intake_captures_five_minute_capability_and_removes_identity_on_failure(self):
        r, payload = self.handoff_fixture(seconds=299)
        cipher = b'LOCAL qualification ciphertext'
        receipt = {'schema': 1, 'source': s.C, 'manifest_sha256': r['qualification_manifest_sha256'],
                   'ciphertext_sha256': s.sha(cipher)}
        archive = io.BytesIO()
        with zipfile.ZipFile(archive, 'w') as z:
            z.writestr('qualification.tar.age', cipher)
            z.writestr('qualification-receipt.json', json.dumps(receipt))
        raw = archive.getvalue(); r['qualification_sha256'] = s.sha(raw); payload['zip_sha256'] = s.sha(raw)
        def decrypt(args): s.json_write(Path(args[args.index('--output') + 1]), payload)
        with tempfile.TemporaryDirectory() as temp, patch.dict(os.environ, {'SIGN_AGE': 'LOCAL age', 'GITHUB_RUN_ID': '8', 'GITHUB_RUN_ATTEMPT': '1'}):
            root = Path(temp); work = root/'work'; work.mkdir(); (work/'identity').write_text('LOCAL identity')
            with patch.object(s, 'command', side_effect=decrypt), patch.object(s, 'qualification_download', return_value=raw):
                s.intake(r, work, root/'output')
                self.assertEqual(sorted(x.name for x in (root/'output').iterdir()),
                                 ['QUALIFICATION-INTAKE.json', 'qualification-receipt.json', 'qualification.tar.age'])
                self.assertEqual(s.load(s.read(root/'output/QUALIFICATION-INTAKE.json')), s.intake_receipt(r, root/'output'))
                self.assertFalse((work/'identity').exists())
                self.assertFalse((work/'qualification-capability.json').exists())
                self.assertFalse((work/'qualification-handoff.age').exists())
                (work/'identity').write_text('LOCAL identity')
                payload['source'] = '0'*40
                with self.assertRaises(s.GateRejected): s.intake(r, work, root/'bad')
                self.assertFalse((work/'identity').exists()); self.assertFalse((root/'bad').exists())

    def test_intake_artifact_requires_same_attempt_source_inputs_and_digest_after_expiry(self):
        r, payload = self.handoff_fixture(seconds=299)
        env = {'GITHUB_RUN_ID': '8', 'GITHUB_RUN_ATTEMPT': '2', 'SIGN_INTAKE_ARTIFACT': '9'}
        with tempfile.TemporaryDirectory() as temp, patch.dict(os.environ, env):
            root = Path(temp); captured = root/'captured'; captured.mkdir()
            (captured/'qualification.tar.age').write_bytes(b'LOCAL encrypted')
            s.json_write(captured/'qualification-receipt.json', {'schema':1,'source':s.C,
                'manifest_sha256':r['qualification_manifest_sha256'],'ciphertext_sha256':s.sha(b'LOCAL encrypted')})
            s.json_write(captured/'QUALIFICATION-INTAKE.json', s.intake_receipt(r, captured))
            archive = io.BytesIO()
            with zipfile.ZipFile(archive, 'w') as z:
                for p in captured.iterdir(): z.writestr(p.name, p.read_bytes())
            raw = archive.getvalue(); os.environ['SIGN_INTAKE_SHA256'] = s.sha(raw)
            metadata = {'id':8,'run_attempt':2,'head_sha':S,'head_branch':'master',
                'repository':{'id':1409587171},'head_repository':{'id':1409587171},
                'event':'workflow_dispatch','path':'.github/workflows/release-sign.yml','status':'in_progress','conclusion':None}
            artifact = {'id':9,'name':'qualification-intake-8-2','expired':False,'digest':'sha256:'+s.sha(raw),'size_in_bytes':len(raw),
                'workflow_run':{'id':8,'head_sha':S,'repository_id':1409587171,'head_repository_id':1409587171}}
            def get(path):
                if path == '/actions/runs/8/attempts/2': return metadata
                if path == '/actions/artifacts/9': return artifact
                self.fail('unexpected API')
            with patch.object(s.api, 'api', side_effect=get), patch.object(s, 'download', return_value=raw), \
                    patch.object(s.time, 'time', return_value=int(r['qualification_expires'])+1), \
                    patch.object(s, 'qualification_download') as url_fetch:
                work = root/'work'; work.mkdir(); s.download_intake(s.validate_request(r), work)
                url_fetch.assert_not_called()
                with self.assertRaises(ValueError): s.handoff_input(r, 60)
                for obj, key, value in [(metadata,'run_attempt',1),(metadata,'head_sha','0'*40),
                                        (artifact,'digest','sha256:'+'0'*64),(artifact,'name','qualification-intake-8-1')]:
                    before=obj[key]; obj[key]=value
                    with self.subTest(key=key), self.assertRaises(ValueError): s.download_intake(r, root/'bad')
                    obj[key]=before
                (root/'wrong-input').mkdir()
                bad = dict(r); bad['qualification_asset']='77'
                with self.assertRaises(ValueError): s.download_intake(bad, root/'wrong-input')
                with patch.object(s.api, 'api', side_effect=KeyError('missing artifact')):
                    with self.assertRaises(KeyError): s.download_intake(r, root/'missing')

    def preflight_metadata(self):
        # Synthetic API responses only; no workflow token, identity or network.
        def run(ident, path):
            return {'id': int(ident), 'run_attempt': 1, 'head_sha': s.C,
                    'head_branch': s.TAG, 'repository': {'id': 1409587171, 'full_name': 'bohewu/yetlet'},
                    'head_repository': {'id': 1409587171, 'full_name': 'bohewu/yetlet'}, 'event': 'workflow_dispatch',
                    'path': path, 'status': 'completed', 'conclusion': 'success'}
        responses = {
            '/branches/master': {'protected': True, 'commit': {'sha': S}},
            '/environments/release-sign': {
                'name': 'release-sign', 'can_admins_bypass': False,
                'protection_rules': [{'type': 'required_reviewers', 'prevent_self_review': False,
                                     'reviewers': [{'type': 'User', 'reviewer': {'login': 'bohewu', 'id': 35250002}}]}],
                'deployment_branch_policy': {'protected_branches': False, 'custom_branch_policies': True}},
            '/environments/release-sign/deployment-branch-policies?per_page=100': {
                'total_count': 1, 'branch_policies': [{'name': 'master', 'type': 'branch'}]},
            '/actions/runs/' + s.BUILD + '/attempts/1': run(s.BUILD, '.github/workflows/release.yml'),
            '/actions/artifacts/' + s.BUILD_ARTIFACT: {
                'id': int(s.BUILD_ARTIFACT), 'name': 'encrypted-private-' + s.C + '-' + s.BUILD + '-1',
                'expired': False, 'digest': 'sha256:' + s.BUILD_ZIP, 'size_in_bytes': 1,
                'workflow_run': {'id': int(s.BUILD), 'head_sha': s.C,
                                 'repository_id': 1409587171, 'head_repository_id': 1409587171}},
            '/actions/runs/' + s.VERIFIER + '/attempts/1': run(s.VERIFIER, '.github/workflows/evidence-verify.yml'),
            '/actions/runs/' + s.VERIFIER + '/approvals': [
                {'state': 'approved', 'user': {'id': 35250002}, 'environments': [{'id': 25525950662}]}],
        }
        responses['/environments/release-evidence-verify'] = copy.deepcopy(responses['/environments/release-sign'])
        responses['/environments/release-evidence-verify'].update(name='release-evidence-verify', id=25525950662)
        responses['/environments/release-evidence-verify/deployment-branch-policies?per_page=100'] = {
            'total_count': 1, 'branch_policies': [{'name': s.TAG, 'type': 'tag'}]}
        return responses

    def test_preflight_never_reads_draft_and_http_rejection_stays_redacted(self):
        responses = self.preflight_metadata()
        with patch.dict(os.environ, {'SIGN_REQUEST': json.dumps(prepare_request())}), \
                patch.object(s.sys, 'argv', ['release-sign.py', 'preflight']), \
                patch.object(s, 'identity'), patch.object(s.api, 'api', side_effect=responses.__getitem__) as api, \
                patch.object(s, 'command') as native, patch.object(s, 'oidc_token') as oidc, \
                patch('sys.stderr', new_callable=io.StringIO) as stderr:
            self.assertEqual(s.cli(), 0)
            self.assertEqual(stderr.getvalue(), '')
            api.reset_mock()
            body = io.BytesIO(b'PRIVATE-BODY')
            responses['/actions/artifacts/' + s.BUILD_ARTIFACT] = urllib.error.HTTPError(
                'https://private.invalid/?token=PRIVATE-URL', 404, 'PRIVATE-REASON\nPRIVATE-CONTROL',
                {'PRIVATE-HEADER': 'PRIVATE-VALUE'}, body)
            def read(path):
                value = responses[path]
                if isinstance(value, Exception): raise value
                return value
            api.side_effect = read
            self.assertEqual(s.cli(), 1)
            self.assertEqual(stderr.getvalue(),
                'release signer rejected; gate=build-artifact class=http status=404; private diagnostics suppressed\n')
            self.assertNotIn('PRIVATE-', stderr.getvalue())
            self.assertTrue(body.closed)
            self.assertEqual(api.call_args.args, ('/actions/artifacts/' + s.BUILD_ARTIFACT,))
            native.assert_not_called(); oidc.assert_not_called()

    def test_preflight_metadata_and_transport_rejections_remain_closed(self):
        cases = [('metadata', 'environment-policy', None),
                 ('network', 'branch', urllib.error.URLError('PRIVATE-NETWORK')),
                 ('internal', 'branch', RuntimeError('PRIVATE-INTERNAL'))]
        for kind, stage, error in cases:
            responses = self.preflight_metadata()
            if kind == 'metadata': responses['/environments/release-sign']['can_admins_bypass'] = True
            def read(path):
                if error is not None: raise error
                return responses[path]
            with self.subTest(kind=kind), patch.dict(os.environ, {'SIGN_REQUEST': json.dumps(prepare_request())}), \
                    patch.object(s.sys, 'argv', ['release-sign.py', 'preflight']), \
                    patch.object(s, 'identity'), patch.object(s.api, 'api', side_effect=read), \
                    patch.object(s, 'command') as native, patch.object(s, 'oidc_token') as oidc, \
                    patch('sys.stderr', new_callable=io.StringIO) as stderr:
                self.assertEqual(s.cli(), 1)
                self.assertEqual(stderr.getvalue(),
                    f'release signer rejected; gate={stage} class={kind}; private diagnostics suppressed\n')
                native.assert_not_called(); oidc.assert_not_called()

    def test_preflight_cli_keeps_failures_outside_gate_private(self):
        with patch.object(s, 'main', side_effect=ValueError('PRIVATE-REQUEST')), \
                patch('sys.stderr', new_callable=io.StringIO) as stderr:
            self.assertEqual(s.cli(), 1)
            self.assertEqual(stderr.getvalue(), 'release signer rejected; private diagnostics suppressed\n')

    def test_closed_modes_exact_payload_approval_and_no_implicit_S(self):
        for r in [prepare_request(), sign_request()]:
            with patch.dict(os.environ, {"SIGN_REQUEST": json.dumps(r)}):
                self.assertEqual(s.request(), r)
        invalid = []
        for key, value in [("mode", "publish"), ("signer_commit", s.C), ("signer_commit", "master"),
                           ("d6", "yes"), ("provenance_sha256", "0" * 64), ("artifact", "0")]:
            r = sign_request(); r[key] = value; invalid.append(r)
        r = prepare_request(); r['id_token'] = True; invalid.append(r)
        for r in invalid:
            with self.subTest(r=r), patch.dict(os.environ, {"SIGN_REQUEST": json.dumps(r)}):
                with self.assertRaises(ValueError): s.request()
        with patch.dict(os.environ, {"SIGN_REQUEST": '{"mode":"sign","mode":"prepare"}'}):
            with self.assertRaises(ValueError): s.request()

    def test_environment_rejects_bypass_wrong_reviewer_and_tag_policy(self):
        e = {"name": "release-sign", "can_admins_bypass": False,
             "protection_rules": [{"type": "required_reviewers", "prevent_self_review": False,
                                   "reviewers": [{"type": "User", "reviewer": {"login": "bohewu", "id": 35250002}}]}],
             "deployment_branch_policy": {"protected_branches": False, "custom_branch_policies": True}}
        p = {"total_count": 1, "branch_policies": [{"name": "master", "type": "branch"}]}
        s.environment_gate(e, p)
        for change in ['bypass', 'reviewer', 'tag', 'extra']:
            ee, pp = copy.deepcopy(e), copy.deepcopy(p)
            if change == 'bypass': ee['can_admins_bypass'] = True
            if change == 'reviewer': ee['protection_rules'][0]['reviewers'][0]['reviewer']['id'] = 7
            if change == 'tag': pp['branch_policies'][0]['type'] = 'tag'
            if change == 'extra': pp['total_count'] = 2
            with self.assertRaises(ValueError): s.environment_gate(ee, pp)

    def test_foreign_source_stops_before_git_or_api(self):
        with patch.dict(os.environ, {"GITHUB_EVENT_NAME": "pull_request"}), patch.object(s.subprocess, 'check_output') as git:
            with self.assertRaises(ValueError): s.identity(prepare_request())
            git.assert_not_called()

    def test_zip_allows_only_pinned_ciphertext_and_receipt(self):
        def blob(entries):
            f = io.BytesIO()
            with zipfile.ZipFile(f, 'w') as z:
                for name, raw in entries: z.writestr(name, raw)
            return f.getvalue()
        with tempfile.TemporaryDirectory() as t:
            names = {'cipher.age': 32, 'safe.json': 32}
            good = blob([('cipher.age', b'cipher'), ('safe.json', b'{}')])
            out = Path(t)/'good'; s.unpack_zip(good, s.sha(good), out, names)
            self.assertEqual((out/'cipher.age').read_bytes(), b'cipher')
            for i, entries in enumerate([[('../cipher.age', b'x'), ('safe.json', b'{}')],
                                         [('cipher.age', b'x'), ('safe.json', b'{}'), ('identity', b'no')],
                                         [('cipher.age', b'x'), ('cipher.age', b'y')]]):
                raw = blob(entries)
                with self.assertRaises(ValueError): s.unpack_zip(raw, s.sha(raw), Path(t)/str(i), names)
            with self.assertRaises(ValueError): s.unpack_zip(good, '0'*64, Path(t)/'wrong', names)

    def test_flat_restore_rejects_symlinks_traversal_and_extra_file(self):
        with tempfile.TemporaryDirectory() as t:
            root = Path(t); src = root/'src'; src.mkdir(); (src/'evidence').write_bytes(b'original')
            s.checksums.run('create', str(src)); s.archive(src, root/'good.tar')
            s.restore(root/'good.tar', root/'restored')
            self.assertEqual((root/'restored/evidence').read_bytes(), b'original')
            for i, name in enumerate(['../outside', '/absolute', 'link']):
                p = root/(str(i)+'.tar')
                with tarfile.open(p, 'w', format=tarfile.USTAR_FORMAT) as z:
                    e = tarfile.TarInfo(name)
                    if name == 'link': e.type = tarfile.SYMTYPE; e.linkname = '/unrelated'
                    z.addfile(e)
                with self.assertRaises(ValueError): s.restore(p, root/('bad'+str(i)))
            (src/'extra').write_bytes(b'not manifest-bound'); s.archive(src, root/'extra.tar')
            with self.assertRaises(ValueError): s.restore(root/'extra.tar', root/'extra')

    def test_qualification_preserves_scopes_and_requires_failure_evidence(self):
        with tempfile.TemporaryDirectory() as t:
            p = Path(t); raw = b'LOCAL synthetic preserved result'
            (p/'record.json').write_bytes(raw)
            entry = {'name': 'record.json', 'size': len(raw), 'sha256': s.sha(raw),
                     'roles': sorted(s.ROLES), 'original_source': 'historical fixture',
                     'scope': 'LOCAL synthetic; no product qualification'}
            closure = {'schema': 1, 'source': s.C, 'entries': [entry]}
            s.json_write(p/'QUALIFICATION-CLOSURE.json', closure); s.checksums.run('create', str(p))
            self.assertEqual(s.qualification(p), closure)
            (p/s.MANIFEST).unlink(); closure['entries'][0]['roles'].remove('failures')
            (p/'QUALIFICATION-CLOSURE.json').unlink(); s.json_write(p/'QUALIFICATION-CLOSURE.json', closure)
            s.checksums.run('create', str(p))
            with self.assertRaises(ValueError): s.qualification(p)

    def test_tampered_preparation_receipt_stops_before_decryption(self):
        with tempfile.TemporaryDirectory() as t:
            p = Path(t); (p/'transport').mkdir(); (p/'transport/SIGNING-INPUTS.json').write_bytes(b'{}')
            with patch.object(s, 'command') as tool:
                with self.assertRaises(ValueError): s.signing_input(sign_request(), p)
                tool.assert_not_called()

    def test_identity_must_be_removed_before_any_signer_command(self):
        with tempfile.TemporaryDirectory() as t:
            p = Path(t); (p/'identity').write_bytes(b'LOCAL synthetic marker')
            with patch.object(s, 'command') as tool:
                with self.assertRaises(ValueError): s.sign(sign_request(), p, p/'out', p/'tools')
                tool.assert_not_called()
            (p/'identity').unlink()
            with patch.dict(os.environ, {'AGE_IDENTITY': 'LOCAL synthetic marker'}), patch.object(s, 'command') as tool:
                with self.assertRaises(ValueError): s.sign(sign_request(), p, p/'out', p/'tools')
                tool.assert_not_called()

    def test_oidc_claims_require_exact_workflow_digest_subject_and_invocation(self):
        claims = {"iss": s.ISSUER, "aud": "sigstore", "repository": "bohewu/yetlet",
                  "repository_id": "1409587171", "repository_owner_id": "35250002",
                  "ref": "refs/heads/master", "sha": S, "workflow_sha": S,
                  "workflow_ref": "bohewu/yetlet/.github/workflows/release-sign.yml@refs/heads/master",
                  "event_name": "workflow_dispatch", "runner_environment": "github-hosted",
                  "environment": "release-sign", "run_id": "8", "run_attempt": "1",
                  "sub": "repo:bohewu@35250002/yetlet@1409587171:environment:release-sign"}
        with patch.dict(os.environ, {"GITHUB_RUN_ID": "8", "GITHUB_RUN_ATTEMPT": "1"}):
            s.oidc_claims(claims, sign_request())
            for key, value in [('sub', 'repo:bohewu/yetlet:ref:refs/heads/master'), ('workflow_sha', s.C),
                               ('runner_environment', 'self-hosted'), ('aud', 'other'), ('run_id', '7')]:
                changed = dict(claims); changed[key] = value
                with self.assertRaises(ValueError): s.oidc_claims(changed, sign_request())

    def test_failed_capsule_encryption_removes_temporary_plaintext(self):
        with tempfile.TemporaryDirectory() as t:
            p = Path(t); source=p/'source'; source.mkdir()
            age = p/'age'; age.write_bytes(b'LOCAL synthetic tool')
            with patch.object(s, 'qualification'), patch.object(s, 'AGE_SHA', s.sha(age.read_bytes())), patch.object(s, 'command', side_effect=ValueError('LOCAL synthetic failure')), patch.dict(os.environ, {'SIGN_AGE': str(age)}):
                with self.assertRaises(ValueError): s.pack(source, p/'output')
            self.assertFalse((p/'output/qualification.tar').exists())

    def test_new_prepare_flat_assembly_keeps_scopes_original_manifest_and_exact_payloads(self):
        import shutil
        with tempfile.TemporaryDirectory() as t:
            p = Path(t); fixture=p/'fixture'; fixture.mkdir(); q=p/'q'; q.mkdir(); work=p/'work'; work.mkdir()
            subjects=[]
            for n in sorted(s.PACKAGES):
                name=n+'-0.9.0-alpha.1.crate'; raw=('LOCAL synthetic '+name).encode()
                (fixture/name).write_bytes(raw); subjects.append({'name':name,'digest':{'sha256':s.sha(raw)}})
            statement={'_type':'https://in-toto.io/Statement/v1','subject':subjects,'predicateType':'https://slsa.dev/provenance/v1',
                       'predicate':{'buildDefinition':{'externalParameters':{'cohort':'sixteen','workflow':{'repository':'https://github.com/bohewu/yetlet','path':'.github/workflows/release.yml','ref':'refs/tags/'+s.TAG}},'resolvedDependencies':[{'uri':'git+https://github.com/bohewu/yetlet','digest':{'gitCommit':s.C}}]},'runDetails':{'metadata':{'invocationId':'https://github.com/bohewu/yetlet/actions/runs/'+s.BUILD+'/attempts/1'}}}}
            raw=json.dumps(statement).encode(); raw+=b' '*(9158-len(raw)); (fixture/'PROVENANCE-UNSIGNED.json').write_bytes(raw)
            (fixture/'openssl-build.log').write_bytes(b'LOCAL original native log')
            (fixture/'openssl-build-result.json').write_bytes(b'{"native_exit_code":0}')
            s.checksums.run('create',str(fixture)); original_sha=s.sha((fixture/s.MANIFEST).read_bytes())
            record=b'LOCAL synthetic historical FAIL and bounded disposition'; (q/'record').write_bytes(record)
            closure={'schema':1,'source':s.C,'entries':[{'name':'record','size':len(record),'sha256':s.sha(record),'roles':sorted(s.ROLES),'original_source':'LOCAL original fixture','scope':'historical failure, not product proof'}]}
            s.json_write(q/'QUALIFICATION-CLOSURE.json',closure); s.checksums.run('create',str(q)); s.archive(q,p/'q.tar')
            qr=prepare_request(); qr['qualification_manifest_sha256']=s.sha((q/s.MANIFEST).read_bytes())
            (work/'qualification').mkdir(); shutil.copyfile(p/'q.tar',work/'qualification/qualification.tar.age')
            s.json_write(work/'qualification/qualification-receipt.json',{'schema':1,'source':s.C,'manifest_sha256':qr['qualification_manifest_sha256'],'ciphertext_sha256':s.sha((p/'q.tar').read_bytes())})
            def commands(args):
                args=[str(a) for a in args]
                if args[1:4]==['release','custody','verify']:
                    out=Path(args[args.index('--output')+1]); out.mkdir(); shutil.copytree(fixture,out/'private'); (out/'extras').mkdir()
                    shutil.copyfile(fixture/'openssl-build.log',out/'extras/openssl-build.log')
                    shutil.copyfile(fixture/'openssl-build-result.json',out/'extras/openssl-build-result.json')
                    s.json_write(out/'CUSTODY-INVENTORY.json',{'complete':True,'original_exit':0,'private_manifest_sha256':original_sha})
                elif args[1:3]==['release','checksums']: s.checksums.run(args[3],args[4])
                elif '--decrypt' in args: shutil.copyfile(args[-1],args[args.index('--output')+1])
                elif '--encrypt' in args: shutil.copyfile(args[-1],args[args.index('--output')+1])
                else: self.fail('unexpected command')
            env={'SIGN_AGE':'LOCAL synthetic age','SIGN_XTASK':'LOCAL synthetic custody','GITHUB_RUN_ID':'8','GITHUB_RUN_ATTEMPT':'1'}
            with patch.dict(os.environ, env):
                s.json_write(work/'qualification/QUALIFICATION-INTAKE.json', s.intake_receipt(qr, work/'qualification'))
            with patch.object(s,'PUBLIC_SHA',s.sha(raw)),patch.object(s,'command',side_effect=commands),patch.dict(os.environ,env):
                s.prepare(qr,work,p/'output')
                receipt=s.load((p/'output/SIGNING-INPUTS.json').read_bytes())
                self.assertEqual(receipt['provenance_sha256'],s.sha(raw)); self.assertEqual(receipt['original_private_manifest_sha256'],original_sha)
                self.assertEqual(receipt['manifest_sha256'],s.sha((work/'assembly'/s.MANIFEST).read_bytes()))
                self.assertEqual((work/'assembly/BUILD-PREPARATION-CHECKSUMS.json').read_bytes(),(fixture/s.MANIFEST).read_bytes())
                self.assertEqual((work/'assembly/record').read_bytes(),record)
                self.assertEqual((work/'assembly/BUILD-EXTRA-openssl-build.log').read_bytes(),(fixture/'openssl-build.log').read_bytes())
                self.assertEqual((work/'assembly/BUILD-EXTRA-openssl-build-result.json').read_bytes(),(fixture/'openssl-build-result.json').read_bytes())
                self.assertEqual((work/'assembly/openssl-build.log').read_bytes(),(fixture/'openssl-build.log').read_bytes())
                self.assertEqual(receipt['D6'],'Pending'); self.assertFalse((work/'assembly/SIGNING-INPUTS.json').exists())
                # The separately approved exact receipt/hash gate opens the SAME frozen bytes.
                signed=sign_request(); signed.update({'run':'8','provenance_sha256':s.sha(raw),'manifest_sha256':receipt['manifest_sha256'],'receipt_sha256':s.sha((p/'output/SIGNING-INPUTS.json').read_bytes())})
                sign_work=p/'sign-work'; sign_work.mkdir(); shutil.copytree(p/'output',sign_work/'transport')
                s.signing_input(signed,sign_work)
                self.assertEqual((sign_work/'assembly/PROVENANCE-UNSIGNED.json').read_bytes(),raw)

    def test_partial_signing_failure_seals_bundles_and_never_retries(self):
        with tempfile.TemporaryDirectory() as t:
            p=Path(t); out=p/'output'; out.mkdir(); age=p/'age'; age.write_bytes(b'LOCAL synthetic tool')
            raw=b'LOCAL synthetic unchecked bundle'; (out/'provenance.sigstore.json').write_bytes(raw)
            def encrypt(args): Path(args[-2]).write_bytes(b'LOCAL synthetic ciphertext')
            with patch.object(s,'AGE_SHA',s.sha(age.read_bytes())),patch.object(s,'command',side_effect=encrypt) as native,patch.dict(os.environ,{'SIGN_AGE':str(age),'GITHUB_RUN_ID':'8','GITHUB_RUN_ATTEMPT':'1'}):
                s.seal_failure(sign_request(),out)
                self.assertEqual(native.call_count,1)
            self.assertFalse((out/'provenance.sigstore.json').exists())
            receipt=s.load((out/'SIGNING-FAILURE.json').read_bytes())
            self.assertTrue(receipt['partial_sealing_complete']); self.assertFalse(receipt['automatic_retry'])
            self.assertEqual(receipt['status'],'FAIL'); self.assertNotIn(raw, (out/'SIGNING-FAILURE.json').read_bytes())

    def test_workflow_never_gives_prepare_oidc_or_publishes(self):
        import yaml
        workflow=yaml.load((Path(__file__).parents[1]/'.github/workflows/release-sign.yml').read_text(),Loader=yaml.BaseLoader)
        self.assertEqual(set(workflow['on']),{'workflow_dispatch'})
        self.assertNotIn('id-token',workflow['permissions'])
        for name in ['preflight','qualification_intake','prepare']:
            self.assertNotIn('id-token',workflow['jobs'][name].get('permissions',{}))
        self.assertEqual(workflow['jobs']['sign']['permissions']['id-token'],'write')
        for name in ['qualification_intake','prepare','sign']:
            self.assertEqual(workflow['jobs'][name]['environment'],'release-sign')
            self.assertTrue(any(step.get('if')=='always()' for step in workflow['jobs'][name]['steps']))
        for name,job in workflow['jobs'].items():
            self.assertNotIn('write',[v for k,v in job.get('permissions',{}).items() if k!='id-token'])
        intake = workflow['jobs']['qualification_intake']; prepare = workflow['jobs']['prepare']
        self.assertEqual(int(intake['timeout-minutes']) + int(prepare['timeout-minutes']), 30)
        self.assertIn('qualification_intake', prepare['needs'])
        capture = next(x for x in intake['steps'] if x.get('run', '').endswith('release-sign.py intake\n'))
        self.assertIn('unset AGE_IDENTITY', capture['run'])
        self.assertNotIn('GH_TOKEN', capture['env'])
        self.assertTrue(all(not s.re.search(r'\b(cargo|pip)\s', x.get('run','')) for x in intake['steps']))


if __name__ == '__main__':
    unittest.main()
