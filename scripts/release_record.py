#!/usr/bin/env python3
"""Closed, data-only release records. No defaults, credentials or executable inputs."""
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import sys
import tomllib

PACKAGES = frozenset('yetlet-' + n for n in (
    'actions', 'auth', 'config', 'core', 'dom', 'dom-keyed', 'html', 'rate-limit',
    'reactive', 'request-id', 'secrets', 'server-axum', 'session', 'session-redis', 'signing', 'wgpu'))
CORE = {'schema', 'version', 'source', 'cohort'}
COMPLETE = CORE | {'build', 'custody_verifier', 'signer_commit', 'provenance', 'release_id'}
NUM = r'[1-9][0-9]{0,19}'
SHA = r'[0-9a-f]{64}'
COMMIT = r'[0-9a-f]{40}'
# Canonical SemVer without build metadata; numeric prerelease identifiers cannot have leading zeros.
VERSION = r'(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)(?:-([0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*))?'


def require(ok):
    if not ok:
        raise ValueError('release record rejected')


def token(value, pattern):
    return type(value) is str and re.fullmatch(pattern, value) is not None


def unique(pairs):
    value = {}
    for key, item in pairs:
        require(key not in value)
        value[key] = item
    return value


def loads(raw):
    require(type(raw) in (str, bytes) and len(raw) <= 16384)
    return json.loads(raw, object_pairs_hook=unique,
                      parse_constant=lambda _: require(False))


def canonical(value):
    return (json.dumps(value, sort_keys=True, separators=(',', ':'), ensure_ascii=True) + '\n').encode()


def digest(value):
    return hashlib.sha256(canonical(value)).hexdigest()


def validate(value, stage='build'):
    require(type(value) is dict and CORE <= set(value) <= COMPLETE)
    require(type(value['schema']) is int and value['schema'] == 1 and value['cohort'] == 'sixteen')
    version = value['version']
    require(token(version, VERSION) and len(version) <= 128)
    if '-' in version:
        require(all(not p.isdigit() or p == '0' or not p.startswith('0')
                    for p in version.split('-', 1)[1].split('.')))
    require(token(value['source'], COMMIT))
    if 'build' in value:
        b = value['build']
        require(type(b) is dict and set(b) == {'run', 'attempt', 'artifact', 'sha256'})
        require(all(token(b[k], NUM) for k in ('run', 'attempt', 'artifact')) and token(b['sha256'], SHA))
    if 'custody_verifier' in value:
        v = value['custody_verifier']
        require(type(v) is dict and set(v) == {'run', 'attempt', 'environment_id'})
        require(all(token(v[k], NUM) for k in v))
    if 'signer_commit' in value:
        require(token(value['signer_commit'], COMMIT) and value['signer_commit'] != value['source'])
    if 'provenance' in value:
        p = value['provenance']
        require(type(p) is dict and set(p) == {'sha256', 'size'} and token(p['sha256'], SHA))
        require(type(p['size']) is int and 0 < p['size'] <= 1024 * 1024)
    if 'release_id' in value:
        require(token(value['release_id'], NUM))
    require(stage in {'build', 'custody', 'complete'})
    if stage == 'custody':
        require('build' in value)
    if stage == 'complete':
        require(set(value) == COMPLETE)
    return value


def environment(stage='complete'):
    return validate(loads(os.environ['RELEASE_RECORD']), stage)


def source_version(record, root):
    """No Cargo invocation or dependency resolution. Check all sixteen selected manifests."""
    root = Path(root)
    workspace = tomllib.loads((root / 'Cargo.toml').read_text())['workspace']['package']['version']
    require(workspace == record['version'])
    for name in PACKAGES:
        p = tomllib.loads((root / 'crates' / name / 'Cargo.toml').read_text())['package']
        version = workspace if p['version'] == {'workspace': True} else p['version']
        require(p['name'] == name and version == record['version'])


def source_identity(record, root, workflow):
    tag = 'refs/tags/v' + record['version']
    for key, expected in {'GITHUB_EVENT_NAME': 'workflow_dispatch', 'GITHUB_REPOSITORY': 'bohewu/yetlet',
                          'GITHUB_REPOSITORY_ID': '1409587171', 'GITHUB_REPOSITORY_OWNER_ID': '35250002',
                          'GITHUB_REF': tag, 'GITHUB_SHA': record['source'],
                          'RECORD_WORKFLOW_SHA': record['source'],
                          'RECORD_WORKFLOW_REF': 'bohewu/yetlet/.github/workflows/' + workflow + '@' + tag}.items():
        require(os.environ.get(key) == expected)
    def git(*args):
        return subprocess.check_output(['git', '-C', str(root), *args], stderr=subprocess.PIPE, text=True).strip()
    require(git('rev-parse', 'HEAD') == record['source'])
    require(git('rev-parse', '--verify', tag + '^{commit}') == record['source'])
    git('merge-base', '--is-ancestor', record['source'], 'refs/remotes/origin/master')
    require(not git('status', '--porcelain'))
    source_version(record, root)


def main():
    if len(sys.argv) == 3 and sys.argv[1] == 'check':
        r = environment(sys.argv[2])
        print('release record: PASS sha256=' + digest(r))
        return
    require(len(sys.argv) == 3 and sys.argv[1] == 'source' and sys.argv[2] in {'release.yml', 'evidence-verify.yml'})
    r = environment('build' if sys.argv[2] == 'release.yml' else 'custody')
    source_identity(r, Path(__file__).resolve().parent.parent, sys.argv[2])
    print('release source/record: PASS')


if __name__ == '__main__':
    try:
        main()
    except Exception:
        print('release source/record rejected', file=sys.stderr)
        sys.exit(1)
