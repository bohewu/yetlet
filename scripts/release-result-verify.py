#!/usr/bin/env python3
"""Separate protected read-only verification. Never requests OIDC or signs."""
import base64
import importlib.util
import os
from pathlib import Path
import shutil
import subprocess
import sys

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
spec = importlib.util.spec_from_file_location('result_signer', HERE / 'release-sign.py')
s = importlib.util.module_from_spec(spec)
spec.loader.exec_module(s)
ENVIRONMENT = 'release-result-verify'
WORKFLOW = '.github/workflows/release-result-verify.yml'


def request():
    s.bind_record(s.records.environment())
    r = s.records.loads(os.environ['RESULT_REQUEST'])
    s.require(type(r) is dict and set(r) == {'verifier_commit', 'signing_request', 'sign_run',
        'sign_attempt', 'sign_artifact', 'sign_digest', 'execution_sha256'})
    s.require(s.records.token(r['verifier_commit'], s.records.COMMIT))
    s.require(s.validate_request(r['signing_request'])['mode'] == 'sign')
    for key in ('sign_run', 'sign_attempt', 'sign_artifact'):
        s.require(s.records.token(r[key], s.records.NUM))
    for key in ('sign_digest', 'execution_sha256'):
        s.require(s.records.token(r[key], s.records.SHA))
    s.require((r['sign_run'], r['sign_attempt']) !=
              (r['signing_request']['run'], r['signing_request']['attempt']))
    return r


def identity(r):
    expected = {'GITHUB_EVENT_NAME': 'workflow_dispatch', 'GITHUB_REPOSITORY': 'bohewu/yetlet',
        'GITHUB_REPOSITORY_ID': '1409587171', 'GITHUB_REPOSITORY_OWNER_ID': '35250002',
        'GITHUB_REF': 'refs/heads/master', 'GITHUB_SHA': r['verifier_commit'],
        'RESULT_WORKFLOW_SHA': r['verifier_commit'],
        'RESULT_WORKFLOW_REF': 'bohewu/yetlet/' + WORKFLOW + '@refs/heads/master'}
    s.require(all(os.environ.get(k) == v for k, v in expected.items()))
    s.require(subprocess.check_output(['git', 'rev-parse', 'HEAD'], text=True).strip() == r['verifier_commit'])
    s.require('ACTIONS_ID_TOKEN_REQUEST_URL' not in os.environ)


def gate(r):
    identity(r)
    branch = s.api.api('/branches/master')
    s.require(branch['protected'] is True and branch['commit']['sha'] == r['verifier_commit'])
    env = s.api.api('/environments/' + ENVIRONMENT)
    policies = s.api.api('/environments/' + ENVIRONMENT + '/deployment-branch-policies?per_page=100')
    s.environment_gate(env, policies, ENVIRONMENT)
    sr = r['signing_request']
    for run, attempt, artifact, digest, prefix in (
            (sr['run'], sr['attempt'], sr['artifact'], sr['digest'], 'signing-inputs-'),
            (r['sign_run'], r['sign_attempt'], r['sign_artifact'], r['sign_digest'], 'signing-results-')):
        s.run_gate(s.api.api('/actions/runs/' + run + '/attempts/' + attempt), run,
                   sr['signer_commit'], 'master', '.github/workflows/release-sign.yml', attempt)
        s.artifact_gate(s.api.api('/actions/artifacts/' + artifact), run, sr['signer_commit'],
                       artifact, digest, prefix + run + '-' + attempt)


def execution(r, work):
    directory = work / 'results'
    raw = s.read(directory / 'SIGNING-EXECUTION.json', 65536)
    s.require(s.sha(raw) == r['execution_sha256'])
    e = s.load(raw)
    s.require(set(e) == {'schema', 'record_sha256', 'source', 'signer_commit', 'approved_inputs',
        'sign_run', 'sign_attempt', 'provenance_bundle_sha256', 'encrypted_manifest_bundle_sha256',
        'same_job_verification', 'independent_verification', 'final_owner_acceptance', 'publication'})
    expected = {'schema': 1, 'record_sha256': s.records.digest(s.RECORD), 'source': s.C,
        'signer_commit': s.RECORD['signer_commit'], 'approved_inputs': r['signing_request'],
        'sign_run': r['sign_run'], 'sign_attempt': r['sign_attempt'],
        'independent_verification': 'Pending', 'final_owner_acceptance': 'Pending'}
    s.require(all(e[k] == v for k, v in expected.items()))
    s.require(e['same_job_verification'] is True and e['publication'] is False)
    for name, key in [('provenance.sigstore.json', 'provenance_bundle_sha256'),
                      ('manifest.sigstore.json.age', 'encrypted_manifest_bundle_sha256')]:
        s.require(s.sha(s.read(directory / name)) == e[key])
    return e


def download(r, work):
    gate(r)
    work.mkdir(mode=0o700)
    sr = r['signing_request']
    for artifact, digest, directory, names in (
        (sr['artifact'], sr['digest'], 'transport',
         {'frozen-signing-inputs.tar.age': s.api.LIMIT, 'SIGNING-INPUTS.json': 65536}),
        (r['sign_artifact'], r['sign_digest'], 'results',
         {'provenance.sigstore.json': 1024 * 1024, 'manifest.sigstore.json.age': 1024 * 1024,
          'SIGNING-EXECUTION.json': 65536})):
        raw = s.download('/actions/artifacts/' + artifact + '/zip')
        s.unpack_zip(raw, digest, work / directory, names)
    execution(r, work)  # Reject partial/failure receipts before key access.


def verify(r, work, tool_dir):
    """Returns safe data only; the caller removes plaintext before emitting a receipt."""
    s.tools(tool_dir)
    s.require(s.sha(s.read(os.environ['SIGN_AGE'])) == s.AGE_SHA)
    s.require(s.sha(s.read(HERE.parent / '.github/release-evidence-recipient.txt')) == s.RECIPIENT_SHA)
    e = execution(r, work)
    sr = r['signing_request']
    s.signing_input(sr, work)  # Full frozen manifest and all actual entries/payloads.
    final = work / 'assembly'
    results = work / 'results'
    private = work / 'manifest.sigstore.json'
    s.command([os.environ['SIGN_AGE'], '--decrypt', '--identity', work / 'identity',
               '--output', private, results / 'manifest.sigstore.json.age'])
    common = ['--certificate-identity', s.SAN, '--certificate-oidc-issuer', s.ISSUER,
              '--trusted-root', tool_dir / 'trusted_root.json', '--use-signed-timestamps']
    for name in sorted(s.PACKAGES):
        s.command([tool_dir / 'cosign', 'verify-blob-attestation',
                   final / (name + '-' + s.RECORD['version'] + '.crate'),
                   '--bundle', results / 'provenance.sigstore.json', '--type', 'slsaprovenance1', *common])
    s.command([tool_dir / 'cosign', 'verify-blob', final / s.MANIFEST, '--bundle', private, *common])
    public_bundle = s.load(s.read(results / 'provenance.sigstore.json'))
    s.require(base64.b64decode(public_bundle['dsseEnvelope']['payload'], validate=True) ==
              s.read(final / 'PROVENANCE-UNSIGNED.json'))
    for bundle in (public_bundle, s.load(s.read(private))):
        s.certificate(bundle, sr, r['sign_run'], r['sign_attempt'])
    s.require(s.payloads(final) == (sr['provenance_sha256'], sr['manifest_sha256']))
    entries = s.load(s.read(final / s.MANIFEST))['artifacts']
    return {'schema': 1, 'status': 'PASS', 'independent_verification': True,
        'record_sha256': s.records.digest(s.RECORD), 'source': s.C,
        'signer_commit': sr['signer_commit'], 'verifier_commit': r['verifier_commit'],
        'verifier_run': os.environ['GITHUB_RUN_ID'], 'verifier_attempt': os.environ['GITHUB_RUN_ATTEMPT'],
        'prepare_run': sr['run'], 'prepare_attempt': sr['attempt'], 'prepare_receipt_sha256': sr['receipt_sha256'],
        'sign_run': r['sign_run'], 'sign_attempt': r['sign_attempt'], 'execution_sha256': r['execution_sha256'],
        'provenance_sha256': sr['provenance_sha256'], 'manifest_sha256': sr['manifest_sha256'],
        'provenance_bundle_sha256': e['provenance_bundle_sha256'],
        'encrypted_manifest_bundle_sha256': e['encrypted_manifest_bundle_sha256'],
        'manifest_bundle_sha256': s.sha(s.read(private)), 'subjects_verified': len(s.PACKAGES),
        'entries_verified': len(entries), 'final_owner_acceptance': 'Pending', 'publication': False}


def main():
    s.require(sys.argv[1:] in (['preflight'], ['download'], ['verify']))
    r = request()
    if sys.argv[1] == 'preflight':
        gate(r)
        return
    work = s.outside(os.environ['RESULT_WORK'])
    if sys.argv[1] == 'download':
        download(r, work)
        return
    output = s.outside(os.environ['RESULT_OUTPUT'])
    try:
        gate(r)  # Recheck exact V and metadata in protected no-OIDC job.
        proof = verify(r, work, Path(os.environ['RESULT_TOOLS']))
    finally:
        shutil.rmtree(work)  # A cleanup failure prevents receipt output and success.
    s.json_write(output, proof)


if __name__ == '__main__':
    try:
        main()
    except Exception:
        print('independent release-result verification rejected; no signing or retry', file=sys.stderr)
        sys.exit(1)
