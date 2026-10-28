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
from unittest.mock import patch
import zipfile

spec = importlib.util.spec_from_file_location("release_sign", Path(__file__).with_name("release-sign.py"))
s = importlib.util.module_from_spec(spec)
spec.loader.exec_module(s)
S = "a" * 40


def prepare_request():
    return {"mode": "prepare", "signer_commit": S, "qualification_asset": "1",
            "qualification_sha256": "b" * 64, "qualification_manifest_sha256": "c" * 64}


def sign_request():
    return {"mode": "sign", "signer_commit": S, "run": "2", "attempt": "1", "artifact": "3",
            "digest": "d" * 64, "receipt_sha256": "e" * 64, "provenance_sha256": s.PUBLIC_SHA,
            "manifest_sha256": "f" * 64, "d6": "approve-exact-signing-inputs"}


class SignerTests(unittest.TestCase):
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
                    s.json_write(out/'CUSTODY-INVENTORY.json',{'complete':True,'original_exit':0,'private_manifest_sha256':original_sha})
                elif args[1:3]==['release','checksums']: s.checksums.run(args[3],args[4])
                elif '--decrypt' in args: shutil.copyfile(args[-1],args[args.index('--output')+1])
                elif '--encrypt' in args: shutil.copyfile(args[-1],args[args.index('--output')+1])
                else: self.fail('unexpected command')
            env={'SIGN_AGE':'LOCAL synthetic age','SIGN_XTASK':'LOCAL synthetic custody','GITHUB_RUN_ID':'8','GITHUB_RUN_ATTEMPT':'1'}
            with patch.object(s,'PUBLIC_SHA',s.sha(raw)),patch.object(s,'command',side_effect=commands),patch.dict(os.environ,env):
                s.prepare(qr,work,p/'output')
                receipt=s.load((p/'output/SIGNING-INPUTS.json').read_bytes())
                self.assertEqual(receipt['provenance_sha256'],s.sha(raw)); self.assertEqual(receipt['original_private_manifest_sha256'],original_sha)
                self.assertEqual(receipt['manifest_sha256'],s.sha((work/'assembly'/s.MANIFEST).read_bytes()))
                self.assertEqual((work/'assembly/BUILD-PREPARATION-CHECKSUMS.json').read_bytes(),(fixture/s.MANIFEST).read_bytes())
                self.assertEqual((work/'assembly/record').read_bytes(),record)
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
        for name in ['preflight','prepare']:
            self.assertNotIn('id-token',workflow['jobs'][name].get('permissions',{}))
        self.assertEqual(workflow['jobs']['sign']['permissions']['id-token'],'write')
        for name in ['prepare','sign']:
            self.assertEqual(workflow['jobs'][name]['environment'],'release-sign')
            self.assertTrue(any(step.get('if')=='always()' for step in workflow['jobs'][name]['steps']))
        for name,job in workflow['jobs'].items():
            self.assertNotIn('write',[v for k,v in job.get('permissions',{}).items() if k!='id-token'])


if __name__ == '__main__':
    unittest.main()
