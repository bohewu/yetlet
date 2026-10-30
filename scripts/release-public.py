#!/usr/bin/env python3
"""Explicit exact-ID public staging/readback/publication; no signing or registry access."""
import argparse
import base64
import importlib.util
import os
from pathlib import Path
import re
import subprocess
import sys
import urllib.error
import urllib.parse
import urllib.request

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
spec = importlib.util.spec_from_file_location('public_signer', HERE / 'release-sign.py')
s = importlib.util.module_from_spec(spec)
spec.loader.exec_module(s)
API = s.api.API


class UploadError(Exception):
    """Only bounded transport diagnostics may leave the upload subprocess."""
    def __init__(self, kind, http_status=None):
        super().__init__('release asset upload rejected')
        self.kind = kind
        self.http_status = http_status


class ApiError(Exception):
    """Preserve a bounded HTTP failure without its URL, headers or body."""
    def __init__(self, method, status):
        super().__init__('release API request rejected')
        self.method = method
        self.http_status = status


def failure_details(error):
    if isinstance(error, UploadError):
        kind = error.kind if error.kind in {'timeout', 'unavailable', 'http', 'cli', 'response'} else 'internal'
        operation, status = 'upload', error.http_status
    elif isinstance(error, ApiError):
        operation = {'GET': 'api-read', 'PATCH': 'release-update'}.get(error.method, 'api-request')
        kind, status = 'http', error.http_status
    else:
        operation, status = 'validation', None
        kind = 'validation' if isinstance(error, (ValueError, KeyError, TypeError)) else 'internal'
        if isinstance(error, TimeoutError):
            operation, kind = 'api-request', 'timeout'
        elif isinstance(error, urllib.error.URLError):
            operation, kind = 'api-request', 'network'
    return {'operation': operation, 'class': kind,
            'http_status': status if type(status) is int and 300 <= status <= 599 else None}


def upload(url, body):
    # Use the same GitHub CLI transport as the accepted alpha.3 delivery.
    # Raw stdin preserves bytes; credentials stay in the existing environment.
    try:
        result = subprocess.run(['gh', 'api', '--method', 'POST', url,
            '--header', 'Accept: application/vnd.github+json',
            '--header', 'Content-Type: application/octet-stream',
            '--header', 'Content-Length: ' + str(len(body)), '--input', '-'],
            input=body, stdout=subprocess.PIPE, stderr=subprocess.PIPE, timeout=120)
    except subprocess.TimeoutExpired:
        raise UploadError('timeout') from None
    except OSError:
        raise UploadError('unavailable') from None
    if result.returncode:
        codes = re.findall(rb'\(HTTP ([45][0-9]{2})\)', result.stderr)
        raise UploadError('http' if codes else 'cli', int(codes[-1]) if codes else None)
    try:
        return s.load(result.stdout)
    except (ValueError, RecursionError):
        raise UploadError('response') from None


def http(url, method='GET', body=None, binary=False, authenticated=True, redirects=0):
    s.require(redirects <= 3)
    headers = {'Accept': 'application/octet-stream' if binary else 'application/vnd.github+json',
               'X-GitHub-Api-Version': '2026-03-10', 'Cache-Control': 'no-cache'}
    if authenticated:
        s.require(url.startswith(API + '/') or url.startswith(
            'https://uploads.github.com/repos/bohewu/yetlet/releases/'))
        headers['Authorization'] = 'Bearer ' + os.environ['GH_TOKEN']
    if method == 'POST' and binary:
        s.require(authenticated and url.startswith(
            'https://uploads.github.com/repos/bohewu/yetlet/releases/'))
        return upload(url, body)
    if body is not None:
        headers['Content-Type'] = 'application/octet-stream' if binary else 'application/json'
    request = urllib.request.Request(url, data=body, headers=headers, method=method)
    try:
        response = urllib.request.build_opener(s.api.NoRedirect()).open(request, timeout=120)
    except urllib.error.HTTPError as e:
        try:
            if e.code == 404 and method == 'GET' and url == API + '/releases/latest' and not binary:
                return None
            if not (method == 'GET' and binary and e.code == 302 and redirects < 3):
                raise ApiError(method, e.code) from None
            location = urllib.parse.urlsplit(e.headers['Location'])
            s.require(location.scheme == 'https' and location.netloc == 'release-assets.githubusercontent.com'
                      and not location.fragment and not location.username)
            # No credential forwarding; provider URL never reaches logs or receipts.
            return http(location.geturl(), binary=True, authenticated=False, redirects=redirects + 1)
        finally:
            e.close()
    with response:
        raw = response.read(s.checksums.MAX_FILE_BYTES + 1)
        s.require(len(raw) <= s.checksums.MAX_FILE_BYTES)
        return raw if binary and method == 'GET' else s.load(raw)


def api(path, authenticated=True):
    return http(API + path, authenticated=authenticated)


def names():
    return {p + '-' + s.RECORD['version'] + '.crate' for p in s.PACKAGES} | {
        'PROVENANCE-UNSIGNED.json', s.MANIFEST, 'provenance.sigstore.json', 'SIGNING-APPROVAL.json'}


def plan(value):
    fields = {'schema', 'record_sha256', 'release_id', 'name', 'notes_sha256', 'assets',
              'verification_sha256', 'approval_sha256'}
    s.require(type(value) is dict and type(value.get('schema')) is int and value['schema'] in (1, 2))
    s.require(set(value) == fields | ({'registry_plan_sha256', 'publication_request_sha256'}
                                    if value['schema'] == 2 else set()))
    if value['schema'] == 2:
        s.require(all(s.records.token(value[k], s.records.SHA)
            for k in ('registry_plan_sha256', 'publication_request_sha256')))
    s.require(value['record_sha256'] == s.records.digest(s.RECORD) and value['release_id'] == str(s.RELEASE_ID))
    s.require(type(value['name']) is str and 0 < len(value['name']) <= 256
              and all(ord(c) >= 32 for c in value['name']))
    for key in ('notes_sha256', 'verification_sha256', 'approval_sha256'):
        s.require(s.records.token(value[key], s.records.SHA))
    s.require(type(value['assets']) is list and len(value['assets']) == 20)
    found = set()
    for a in value['assets']:
        s.require(type(a) is dict and set(a) == {'name', 'size', 'sha256'})
        s.require(a['name'] in names() and a['name'] not in found)
        found.add(a['name'])
        s.require(type(a['size']) is int and 0 < a['size'] <= s.checksums.MAX_FILE_BYTES)
        s.require(s.records.token(a['sha256'], s.records.SHA))
    s.require(sum(a['size'] for a in value['assets']) <= s.checksums.MAX_TOTAL_BYTES)
    return value


def public_inputs(p, directory, notes, verification):
    """All local validation completes before a network write; no manifest-body discovery."""
    s.require(s.sha(notes) == p['notes_sha256'])
    notes.decode('utf-8')
    s.require(s.sha(verification) == p['verification_sha256'])
    v = s.load(verification)
    s.require(type(v['schema']) is int and v['schema'] == 1 and v['status'] == 'PASS' and v['independent_verification'] is True)
    s.require(v['record_sha256'] == p['record_sha256'] and v['source'] == s.C
              and v['signer_commit'] == s.RECORD['signer_commit'] and v['subjects_verified'] == 16)
    s.require(type(v['entries_verified']) is int and 0 < v['entries_verified'] <= s.checksums.MAX_FILES)
    s.require(v['provenance_sha256'] == s.PUBLIC_SHA and s.digest(v['manifest_sha256']))
    blobs = {}
    for a in p['assets']:
        raw = s.read(directory / a['name'])
        s.require(len(raw) == a['size'] and s.sha(raw) == a['sha256'])
        blobs[a['name']] = raw
    s.require(s.sha(blobs['SIGNING-APPROVAL.json']) == p['approval_sha256'])
    approval = s.load(blobs['SIGNING-APPROVAL.json'])
    expected_approval = {'schema': 1, 'record_sha256': p['record_sha256'], 'source': s.C,
        'signer_commit': s.RECORD['signer_commit'], 'verification_sha256': p['verification_sha256'],
        'manifest_sha256': v['manifest_sha256'], 'provenance_bundle_sha256': v['provenance_bundle_sha256'],
        'accepted': True, 'scope': 'Experimental source packages', 'publication': 'GitHub pre-release',
        'registry_upload': False}
    if p['schema'] == 2:
        expected_approval.update({'schema': 2, 'publication': 'GitHub pre-release + crates.io',
            'registry_upload': True, 'registry_plan_sha256': p['registry_plan_sha256'],
            'publication_request_sha256': p['publication_request_sha256']})
    s.require(approval == expected_approval)
    s.require(approval['accepted'] is True)
    s.require(s.sha(blobs['provenance.sigstore.json']) == v['provenance_bundle_sha256'])
    bundle = s.load(blobs['provenance.sigstore.json'])
    s.require(base64.b64decode(bundle['dsseEnvelope']['payload'], validate=True) == blobs['PROVENANCE-UNSIGNED.json'])
    s.provenance(directory)  # Original raw statement and all sixteen subjects, C/tag/run.
    checksum = s.load(blobs[s.MANIFEST])
    s.require(set(checksum) == {'schema_version', 'notice', 'artifacts'} and checksum['schema_version'] == 1
              and checksum['notice'] == s.checksums.NOTICE)
    expected = names() - {s.MANIFEST, 'provenance.sigstore.json', 'SIGNING-APPROVAL.json'}
    s.require(len(checksum['artifacts']) == 17 and {e['path'] for e in checksum['artifacts']} == expected)
    for e in checksum['artifacts']:
        s.require(set(e) == {'path', 'size', 'sha256'} and e['sha256'] == s.sha(blobs[e['path']])
                  and e['size'] == len(blobs[e['path']]))
    return blobs


def tag(authenticated=True):
    tag = api('/git/ref/tags/' + s.TAG, authenticated)
    # Current unsigned lightweight-tag contract; never create or move a tag.
    s.require(tag['ref'] == 'refs/tags/' + s.TAG and tag['object'] == {'sha': s.C, 'type': 'commit',
              'url': 'https://api.github.com/repos/bohewu/yetlet/git/commits/' + s.C})


def release(p, draft, authenticated=True):
    tag(authenticated)
    r = api('/releases/' + str(s.RELEASE_ID), authenticated)
    s.require(r['id'] == s.RELEASE_ID and r['tag_name'] == s.TAG and r['target_commitish'] == s.C)
    s.require(r['draft'] is draft and r['prerelease'] is True and r['name'] == p['name'])
    s.require(s.sha(r['body'].encode('utf-8')) == p['notes_sha256'])
    return r


def assets(p, authenticated=True, complete=True):
    items = api('/releases/' + str(s.RELEASE_ID) + '/assets?per_page=100', authenticated)
    expected = {a['name']: a for a in p['assets']}
    s.require(type(items) is list and len(items) <= len(expected))
    seen, ids = set(), set()
    for a in items:
        s.require(a['name'] in expected and a['name'] not in seen and type(a['id']) is int
                  and a['id'] > 0 and a['id'] not in ids and a['state'] == 'uploaded')
        seen.add(a['name']); ids.add(a['id'])
        wanted = expected[a['name']]
        s.require(a['size'] == wanted['size'] and a['digest'] == 'sha256:' + wanted['sha256'])
        raw = http(API + '/releases/assets/' + str(a['id']), binary=True, authenticated=authenticated)
        s.require(len(raw) == wanted['size'] and s.sha(raw) == wanted['sha256'])
    s.require(not complete or seen == set(expected))
    return items


def operate(action, p, blobs, notes, confirm=None):
    # Exact Draft, tag and all pre-existing bytes must pass before the first write.
    if action == 'stage':
        release(p, True)
        existing = assets(p, complete=False)
        seen = {a['name'] for a in existing}
        for a in p['assets']:
            if a['name'] in seen:
                continue
            http('https://uploads.github.com/repos/bohewu/yetlet/releases/' + str(s.RELEASE_ID) +
                 '/assets?name=' + urllib.parse.quote(a['name'], safe=''), 'POST', blobs[a['name']], binary=True)
        release(p, True)
        items = assets(p)
    elif action == 'publish':
        s.require(confirm == 'publish-exact-approved-plan')
        release(p, True)
        assets(p)
        body = {'tag_name': s.TAG, 'target_commitish': s.C, 'name': p['name'], 'body': notes.decode('utf-8'),
                'draft': False, 'prerelease': True, 'make_latest': 'false'}
        http(API + '/releases/' + str(s.RELEASE_ID), 'PATCH', s.records.canonical(body))
        release(p, False)
        items = assets(p)
        release(p, False, False)
        assets(p, False)  # Includes actual anonymous public-bundle download, no auth forwarding.
        latest = api('/releases/latest', False)
        s.require(latest is None or latest['id'] != s.RELEASE_ID)
    else:
        s.require(action == 'readback')
        current = api('/releases/' + str(s.RELEASE_ID))
        s.require(type(current['draft']) is bool)
        release(p, current['draft'])
        items = assets(p)
        if current['draft'] is False:
            release(p, False, False)
            assets(p, False)
            latest = api('/releases/latest', False)
            s.require(latest is None or latest['id'] != s.RELEASE_ID)
    return {'schema': 1, 'action': action, 'record_sha256': p['record_sha256'], 'plan_sha256': s.records.digest(p),
        'release_id': str(s.RELEASE_ID), 'source': s.C, 'tag': s.TAG,
        'notes_sha256': p['notes_sha256'], 'assets': [
            {'id': a['id'], 'name': a['name'], 'size': a['size'], 'sha256': a['digest'][7:]} for a in items],
        'automatic_retry': False, 'registry_upload': False}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('action', choices=['stage', 'readback', 'publish'])
    for name in ('plan', 'public-dir', 'notes', 'verification', 'output'):
        parser.add_argument('--' + name, required=True)
    parser.add_argument('--confirm')
    args = parser.parse_args()
    s.bind_record(s.records.environment())
    output = s.outside(args.output)
    s.require(not output.exists())
    p = plan(s.load(s.read(args.plan)))
    s.require(p['schema'] == 1)  # Combined distribution is gated only by release-publish.yml.
    notes = s.read(args.notes, 65536)
    verification = s.read(args.verification, 65536)
    blobs = public_inputs(p, Path(args.public_dir), notes, verification)
    try:
        result = operate(args.action, p, blobs, notes, args.confirm)
    except Exception as error:
        failure = {'schema': 1, 'status': 'FAIL', 'action': args.action,
            'record_sha256': p['record_sha256'], 'plan_sha256': s.records.digest(p),
            'release_id': str(s.RELEASE_ID), 'remote_partial_state_may_exist': True,
            'automatic_retry': False, 'registry_upload': False}
        if isinstance(error, (UploadError, ApiError)):
            failure['transport_error'] = failure_details(error)
        s.json_write(output, failure)
        raise
    s.json_write(output, result)


def cli():
    try:
        main()
    except (UploadError, ApiError) as error:
        detail = failure_details(error)
        status = '' if detail['http_status'] is None else ' http_status=' + str(detail['http_status'])
        print('public release operation rejected; operation=' + detail['operation'] + ' class=' + detail['class'] + status +
              '; inspect retained partial state, no retry', file=sys.stderr)
        return 1
    except Exception:
        print('public release operation rejected; inspect retained partial state, no retry', file=sys.stderr)
        return 1
    return 0


if __name__ == '__main__':
    sys.exit(cli())
