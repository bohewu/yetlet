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
CONSOLIDATED = frozenset({'yetlet', 'yetlet-browser'})
CLI = frozenset({'yetlet-cli'})
CLI_SDK_VERSIONS = {
    '0.1.0': '0.9.0-alpha.5',
    '0.1.1': '0.9.0-alpha.5',
    '0.2.0': '0.9.0-alpha.6',
    '0.3.0': '0.9.0-alpha.7',
    '0.3.1': '0.9.0-alpha.7',
    '0.4.0': '0.9.0-alpha.7',
}

# Resolver policy is separate from the immutable CLI build-SDK binding.
CLI_SDK_POLICIES = {'0.3.0': 'sdk-compat-v1', '0.3.1': 'sdk-latest-v2', '0.4.0': 'sdk-latest-v2'}


def sdk_version_key(version):
    """The CLI's canonical exact SemVer, including Rust's u64 core limit."""
    require(type(version) is str and len(version) <= 128 and token(version, VERSION))
    core, _, pre = version.partition('-')
    numbers = tuple(map(int, core.split('.')))
    require(all(n <= 18446744073709551615 for n in numbers))
    parts = pre.split('.') if pre else []
    require(all(not p.isdigit() or p == '0' or not p.startswith('0') for p in parts))
    return (numbers, not pre, tuple((0, int(p)) if p.isdigit() else (1, p) for p in parts))


def sdk_policy(raw, cli_version=None, *, context):
    """Validate checkout authority or the policy bound to an archive identity."""
    require(type(raw) is bytes and len(raw) <= 16384 and context in ('source', 'archive'))
    policy = loads(raw.decode('utf-8'))
    require(type(policy) is dict)
    if context == 'archive':
        sdk_version_key(cli_version)
        historical = {
            '0.3.0': '8bddcb15647fd21a755df38f38b6308cad6e8e4f29c4838849923122aefdbbc7',
            '0.3.1': 'de1d1a1f32c3b5013ca48e9f5931e437802fe40474dbf14208826c5805d2a70b',
        }
        if cli_version in historical:
            require(hashlib.sha256(raw).hexdigest() == historical[cli_version])
            require(policy['revision'] == CLI_SDK_POLICIES[cli_version])
            return policy
        require(cli_version not in LEGACY_CLI_VERSIONS)
    require(set(policy) == {'revision', 'selection', 'scaffold_capabilities'}
            and policy['revision'] == 'sdk-latest-v2' and policy['selection'] == 'latest-semver')
    capabilities = policy['scaffold_capabilities']
    require(type(capabilities) is dict and set(capabilities) == {'m26_startup'})
    rows = capabilities['m26_startup']
    require(type(rows) is list)
    versions = set()
    for row in rows:
        require(type(row) is dict and set(row) == {'sdk_version', 'yetlet_sha256'})
        sdk_version_key(row['sdk_version'])
        require(row['sdk_version'] not in versions and token(row['yetlet_sha256'], SHA))
        versions.add(row['sdk_version'])
    return policy


# Immutable scaffold behavior of already published pre-resolver CLI releases.
LEGACY_CLI_VERSIONS = frozenset({'0.1.0', '0.1.1', '0.2.0'})


def cli_sdk_version(version):
    """Trusted release binding; never reinterpret a historical CLI record."""
    require(type(version) is str and version in CLI_SDK_VERSIONS)
    return CLI_SDK_VERSIONS[version]

def packages(record):
    validate(record)
    return {1: PACKAGES, 2: CONSOLIDATED, 3: CLI}[record['schema']]


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
    require(type(value['schema']) is int and (value['schema'], value['cohort']) in ((1, 'sixteen'), (2, 'consolidated'), (3, 'cli')))
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


def tag(record):
    validate(record)
    return ('yetlet-cli-v' if record['schema'] == 3 else 'v') + record['version']


def source_version(record, root):
    """No Cargo invocation or dependency resolution. Check the closed selected manifests."""
    root = Path(root)
    if record['schema'] == 3:
        manifest = tomllib.loads((root / 'tools/yetlet-cli/Cargo.toml').read_text())
        p = manifest['package']
        require(p['name'] == 'yetlet-cli' and p['version'] == record['version'] and p['publish'] is True)
        dependency = manifest['dependencies']['yetlet']
        require(dependency['version'] == '=' + cli_sdk_version(record['version']) and 'path' not in dependency)
        require(any(b['name'] == 'yetlet' for b in manifest['bin']))
        return
    workspace = tomllib.loads((root / 'Cargo.toml').read_text())['workspace']['package']['version']
    require(workspace == record['version'])
    for name in packages(record):
        p = tomllib.loads((root / 'crates' / name / 'Cargo.toml').read_text())['package']
        version = workspace if p['version'] == {'workspace': True} else p['version']
        require(p['name'] == name and version == record['version'])


def source_identity(record, root, workflow):
    tag_ref = 'refs/tags/' + tag(record)
    for key, expected in {'GITHUB_EVENT_NAME': 'workflow_dispatch', 'GITHUB_REPOSITORY': 'bohewu/yetlet',
                          'GITHUB_REPOSITORY_ID': '1409587171', 'GITHUB_REPOSITORY_OWNER_ID': '35250002',
                          'GITHUB_REF': tag_ref, 'GITHUB_SHA': record['source'],
                          'RECORD_WORKFLOW_SHA': record['source'],
                          'RECORD_WORKFLOW_REF': 'bohewu/yetlet/.github/workflows/' + workflow + '@' + tag_ref}.items():
        require(os.environ.get(key) == expected)
    def git(*args):
        return subprocess.check_output(['git', '-C', str(root), *args], stderr=subprocess.PIPE, text=True).strip()
    require(git('rev-parse', 'HEAD') == record['source'])
    require(git('rev-parse', '--verify', tag_ref + '^{commit}') == record['source'])
    git('merge-base', '--is-ancestor', record['source'], 'refs/remotes/origin/master')
    require(not git('status', '--porcelain'))
    source_version(record, root)
    if record['schema'] in (2, 3):
        import release_docs
        release_docs.check_source(root, record['cohort'])


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
