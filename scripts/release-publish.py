#!/usr/bin/env python3
"""Single owner gate: verified artifacts -> GitHub prerelease -> crates.io."""
import importlib.util
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import release_policy as policy
import release_registry as registry

spec = importlib.util.spec_from_file_location('distribution_public', HERE / 'release-public.py')
pub = importlib.util.module_from_spec(spec)
spec.loader.exec_module(pub)
s = pub.s


def request():
    s.bind_record(s.records.environment())
    r = s.records.loads(os.environ['PUBLICATION_REQUEST'])
    s.require(type(r) is dict and set(r) == {'publication_commit', 'verifier_commit', 'verifier_run',
        'verifier_attempt', 'artifact', 'artifact_sha256', 'verification_sha256', 'name', 'notes',
        'notes_sha256', 'registry_plan_sha256'})
    for key in ('publication_commit', 'verifier_commit'):
        s.require(s.records.token(r[key], s.records.COMMIT))
    for key in ('verifier_run', 'verifier_attempt', 'artifact'):
        s.require(s.records.token(r[key], s.records.NUM))
    for key in ('artifact_sha256', 'verification_sha256', 'notes_sha256', 'registry_plan_sha256'):
        s.require(s.records.token(r[key], s.records.SHA))
    s.require(type(r['name']) is str and 0 < len(r['name']) <= 256)
    s.require(type(r['notes']) is str and 0 < len(r['notes'].encode()) <= 8192
              and s.sha(r['notes'].encode()) == r['notes_sha256'])
    return r


def gate(r, approved=False):
    s.require(policy.automatic())
    for key, expected in {'GITHUB_EVENT_NAME': 'workflow_dispatch', 'GITHUB_REPOSITORY': 'bohewu/yetlet',
        'GITHUB_REPOSITORY_ID': '1409587171', 'GITHUB_REPOSITORY_OWNER_ID': '35250002',
        'GITHUB_REF': 'refs/heads/master', 'GITHUB_SHA': r['publication_commit'],
        'PUBLICATION_WORKFLOW_SHA': r['publication_commit'],
        'PUBLICATION_WORKFLOW_REF': 'bohewu/yetlet/.github/workflows/release-publish.yml@refs/heads/master',
        'GITHUB_RUN_ATTEMPT': '1'}.items():
        s.require(os.environ.get(key) == expected)
    s.require(subprocess.check_output(['git', 'rev-parse', 'HEAD'], text=True).strip() == r['publication_commit'])
    s.require('ACTIONS_ID_TOKEN_REQUEST_URL' not in os.environ)
    branch = pub.api('/branches/master')
    s.require(branch['protected'] is True and branch['commit']['sha'] == r['publication_commit'])
    policy.ci(r['publication_commit'], pub.api)
    environment = pub.api('/environments/release-publish')
    policies = pub.api('/environments/release-publish/deployment-branch-policies?per_page=100')
    s.environment_gate(environment, policies, 'release-publish')
    s.run_gate(pub.api('/actions/runs/' + r['verifier_run'] + '/attempts/' + r['verifier_attempt']),
        r['verifier_run'], r['verifier_commit'], 'master', '.github/workflows/release-result-verify.yml',
        r['verifier_attempt'])
    s.artifact_gate(pub.api('/actions/artifacts/' + r['artifact']), r['verifier_run'], r['verifier_commit'],
        r['artifact'], r['artifact_sha256'], 'verified-publication-' + r['verifier_run'] + '-' + r['verifier_attempt'])
    if approved:
        approvals = pub.api('/actions/runs/' + os.environ['GITHUB_RUN_ID'] + '/approvals')
        s.require(any(a['state'] == 'approved' and a['user']['id'] == 35250002 and
            any(e['id'] == environment['id'] for e in a['environments']) for a in approvals))


def distribution(r, work):
    """Re-derive both frozen plans and their joined approval before any mutation."""
    original = work / 'verified'
    names = pub.names() - {'SIGNING-APPROVAL.json'} | {'RESULT-VERIFICATION.json'}
    raw = s.download('/actions/artifacts/' + r['artifact'] + '/zip')
    s.unpack_zip(raw, r['artifact_sha256'], original, {n: s.checksums.MAX_FILE_BYTES for n in names})
    verification = s.read(original / 'RESULT-VERIFICATION.json')
    s.require(s.sha(verification) == r['verification_sha256'])
    proof = s.load(verification)
    s.require(proof['verifier_commit'] == r['verifier_commit'] and
        proof['verifier_run'] == r['verifier_run'] and proof['verifier_attempt'] == r['verifier_attempt'])
    subjects = {v['name']: v['digest']['sha256'] for v in s.load(
        s.read(original / 'PROVENANCE-UNSIGNED.json'))['subject']}
    reg, bodies = registry.prepare(s.RECORD, original, subjects)
    s.require(s.records.digest(reg) == r['registry_plan_sha256'])
    approval = {'schema': 2, 'record_sha256': s.records.digest(s.RECORD), 'source': s.C,
        'signer_commit': s.RECORD['signer_commit'], 'verification_sha256': r['verification_sha256'],
        'manifest_sha256': proof['manifest_sha256'], 'provenance_bundle_sha256': proof['provenance_bundle_sha256'],
        'accepted': True, 'scope': 'Experimental source packages',
        'publication': 'GitHub pre-release + crates.io', 'registry_upload': True,
        'registry_plan_sha256': r['registry_plan_sha256'], 'publication_request_sha256': s.records.digest(r)}
    # This is a private preflight calculation. Only the approved job may publish it.
    directory = work / 'public'
    directory.mkdir(mode=0o700)
    for name in pub.names() - {'SIGNING-APPROVAL.json'}:
        s.write(directory / name, s.read(original / name))
    s.json_write(directory / 'SIGNING-APPROVAL.json', approval)
    plan = {'schema': 2, 'record_sha256': s.records.digest(s.RECORD), 'release_id': str(s.RELEASE_ID),
        'name': r['name'], 'notes_sha256': r['notes_sha256'], 'verification_sha256': r['verification_sha256'],
        'approval_sha256': s.sha(s.read(directory / 'SIGNING-APPROVAL.json')),
        'registry_plan_sha256': r['registry_plan_sha256'], 'publication_request_sha256': s.records.digest(r),
        'assets': [{'name': n, 'size': len(s.read(directory / n)), 'sha256': s.sha(s.read(directory / n))}
                   for n in sorted(pub.names())]}
    pub.plan(plan)
    blobs = pub.public_inputs(plan, directory, r['notes'].encode(), verification)
    # A contents:read token cannot read a Draft. Only public tag reads occur here.
    # The approved stage checks Draft metadata/assets before its first mutation.
    pub.tag()
    return plan, blobs, reg, bodies


def consumer(version, output):
    """Fresh registry-only consumer; no publish token, path patch or source replacement."""
    with tempfile.TemporaryDirectory(prefix='yetlet-registry-consumer-') as temporary:
        root = Path(temporary)
        (root / 'src').mkdir()
        dependencies = '\n'.join(n + ' = "=' + version + '"' for n in sorted(s.PACKAGES))
        (root / 'Cargo.toml').write_text('[workspace]\n[package]\nname="yetlet-registry-consumer"\n'
            'version="0.0.0"\nedition="2024"\npublish=false\n[dependencies]\n' + dependencies + '\n')
        (root / 'src/main.rs').write_text('fn main() { let _ = yetlet_core::View::text("registry consumer"); }\n')
        env = {k: os.environ[k] for k in ('PATH', 'HOME', 'RUSTUP_HOME', 'SSL_CERT_FILE', 'SSL_CERT_DIR',
               'HTTPS_PROXY', 'HTTP_PROXY', 'NO_PROXY') if k in os.environ}
        env.update({'CARGO_HOME': str(root / 'cargo-home'), 'RUSTUP_TOOLCHAIN': '1.99.0',
                    'CARGO_BUILD_JOBS': '2', 'CARGO_TERM_COLOR': 'never'})
        for label, args in [('resolve', ['generate-lockfile']), ('check', ['check', '--locked'])]:
            result = subprocess.run(['cargo', '+1.99.0', *args], cwd=root, env=env,
                                    capture_output=True, timeout=600)
            (output / ('consumer-' + label + '.stdout')).write_bytes(result.stdout)
            (output / ('consumer-' + label + '.stderr')).write_bytes(result.stderr)
            s.require(result.returncode == 0)
        lock = (root / 'Cargo.lock').read_bytes()
        (output / 'REGISTRY-CONSUMER.lock').write_bytes(lock)
        return {'status': 'PASS', 'lock_sha256': s.sha(lock), 'sixteen_exact_versions': version,
                'path_patches': False, 'credentials_forwarded': False}


def main():
    s.require(sys.argv[1:] in (['preflight'], ['publish']))
    r = request()
    approved = sys.argv[1] == 'publish'
    output = s.outside(os.environ['PUBLICATION_OUTPUT'])
    output.mkdir(mode=0o700)
    receipt = {'schema': 1, 'record_sha256': s.records.digest(s.RECORD),
        'publication_request_sha256': s.records.digest(r), 'status': 'not_attempted',
        'github': 'not_attempted', 'registry': 'not_attempted', 'automatic_retry': False}
    phase = 'gate'
    try:
        gate(r, approved)
        with tempfile.TemporaryDirectory(prefix='yetlet-publication-') as temporary:
            phase = 'distribution'
            p, blobs, reg, bodies = distribution(r, Path(temporary))
            s.json_write(output / 'PUBLICATION-PLAN.json', p)
            s.json_write(output / 'REGISTRY-PLAN.json', reg)
            if not approved:
                with open(os.environ['GITHUB_STEP_SUMMARY'], 'a') as summary:
                    summary.write('Publication preflight PASS\n\nSource: `' + s.C + '`\n\nVersion: `' +
                        s.RECORD['version'] + '`\n\nRequest SHA256: `' + s.records.digest(r) +
                        '`\n\nGitHub plan SHA256: `' + s.records.digest(p) + '`\n\nRegistry plan SHA256: `' +
                        s.records.digest(reg) + '`\n\nDestinations: GitHub pre-release, then crates.io (16 packages).\n')
                receipt['status'] = 'preflight_pass'
                return
            phase = 'publication-recheck'
            gate(r, True)  # Recheck approval, source/CI and original artifact immediately before writes.
            receipt['status'] = 'attempted'
            receipt['github'] = 'stage_attempted'
            registry.save(output / 'PUBLICATION-RESULT.json', receipt)
            phase = 'github-stage'
            pub.operate('stage', p, blobs, r['notes'].encode())
            receipt['github'] = 'publish_attempted'
            registry.save(output / 'PUBLICATION-RESULT.json', receipt)
            phase = 'github-publish'
            gh = pub.operate('publish', p, blobs, r['notes'].encode(), 'publish-exact-approved-plan')
            s.json_write(output / 'GITHUB-PUBLICATION.json', gh)
            receipt['github'] = 'published_and_verified'
            receipt['registry'] = 'attempted'
            registry.save(output / 'PUBLICATION-RESULT.json', receipt)
            phase = 'registry-publish'
            reg_result = registry.publish(reg, bodies, output / 'REGISTRY-RESULT.json')
            receipt['registry'] = 'published_and_verified'
            registry.save(output / 'PUBLICATION-RESULT.json', receipt)
            phase = 'registry-consumer'
            reg_result['consumer'] = consumer(s.RECORD['version'], output)
            reg_result['status'] = 'complete'
            registry.save(output / 'REGISTRY-RESULT.json', reg_result)
            receipt['status'] = 'complete'
    except Exception as error:
        receipt['status'] = 'incomplete'
        detail = pub.failure_details(error)
        if isinstance(error, registry.RegistryError):
            detail = {'operation': 'registry',
                'class': error.kind if error.kind in {'http', 'transport', 'not_indexed'} else 'internal',
                'http_status': error.status if type(error.status) is int and 400 <= error.status <= 599 else None}
        receipt['failure'] = dict(detail, phase=phase)
        print('publication failure: ' + json.dumps(receipt['failure'], sort_keys=True), file=sys.stderr)
        raise
    finally:
        registry.save(output / 'PUBLICATION-RESULT.json', receipt)


if __name__ == '__main__':
    try:
        main()
    except Exception:
        print('publication stopped; retained partial receipts, no automatic retry', file=sys.stderr)
        sys.exit(1)
