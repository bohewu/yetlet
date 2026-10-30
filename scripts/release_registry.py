#!/usr/bin/env python3
"""Frozen closed-cohort registry promotion. No packaging, signing or write retries."""
import hashlib
import io
import json
import os
from pathlib import Path
import struct
import tarfile
import time
import tomllib
import urllib.error
import urllib.request

import release_record as records

API = 'https://crates.io/api/v1/crates/new'
OWNER_API = 'https://crates.io/api/v1/crates/yetlet-actions/owners'
MAX = 32 * 1024 * 1024


def require(value):
    if not value:
        raise ValueError('registry publication rejected')


def sha(raw):
    return hashlib.sha256(raw).hexdigest()


def metadata(raw, name, version):
    """Parse only already hash-bound Cargo archives; never extract paths to disk."""
    files, total = {}, 0
    prefix = name + '-' + version + '/'
    with tarfile.open(fileobj=io.BytesIO(raw), mode='r|gz') as archive:
        for item in archive:
            require(item.name.startswith(prefix) and (item.isfile() or item.isdir()))
            relative = item.name[len(prefix):].rstrip('/')
            require(not relative or all(p not in ('', '.', '..') for p in relative.split('/')))
            require('\\' not in relative and len(relative) <= 1024)
            if item.isdir():
                continue
            total += item.size
            require(0 <= item.size <= MAX and total <= MAX and len(files) < 2048)
            require(relative not in files)
            files[relative] = archive.extractfile(item).read(item.size + 1)
            require(len(files[relative]) == item.size)
    manifest = tomllib.loads(files['Cargo.toml'].decode())
    p = manifest['package']
    require(p['name'] == name and p['version'] == version)
    dependencies = []
    for target, container in [(None, manifest), *manifest.get('target', {}).items()]:
        for kind, section in [('normal', 'dependencies'), ('build', 'build-dependencies'),
                              ('dev', 'dev-dependencies')]:
            for alias, value in container.get(section, {}).items():
                d = {'version': value} if isinstance(value, str) else value
                require(not {'workspace', 'path', 'git', 'registry', 'registry-index'} & set(d))
                require(type(d.get('version')) is str and d['version'] != '*')
                dependencies.append({'name': d.get('package', alias), 'version_req': d['version'],
                    'features': d.get('features', []), 'optional': d.get('optional', False),
                    'default_features': d.get('default-features', True), 'target': target,
                    'kind': kind, 'registry': None,
                    'explicit_name_in_toml': alias if 'package' in d else None})
    readme = p.get('readme')
    require(type(readme) is str and readme in files)
    return {'name': name, 'vers': version, 'deps': dependencies,
        'features': manifest.get('features', {}), 'authors': p.get('authors', []),
        'description': p.get('description'), 'documentation': p.get('documentation'),
        'homepage': p.get('homepage'), 'readme': files[readme].decode(), 'readme_file': readme,
        'keywords': p.get('keywords', []), 'categories': p.get('categories', []),
        'license': p.get('license'), 'license_file': p.get('license-file'),
        'repository': p.get('repository'), 'badges': {}, 'links': p.get('links'),
        'rust_version': p.get('rust-version')}


def envelope(meta, raw):
    encoded = records.canonical(meta)
    require(len(encoded) <= 1024 * 1024 and 0 < len(raw) <= MAX)
    return struct.pack('<I', len(encoded)) + encoded + struct.pack('<I', len(raw)) + raw


def prepare(record, directory, subjects):
    records.validate(record, 'complete')
    packages = records.packages(record)
    expected = {n + '-' + record['version'] + '.crate' for n in packages}
    require(set(subjects) == expected)
    rows, blobs = {}, {}
    for name in sorted(packages):
        filename = name + '-' + record['version'] + '.crate'
        path = Path(directory) / filename
        require(path.is_file() and not path.is_symlink() and 0 < path.stat().st_size <= MAX)
        raw = path.read_bytes()
        require(sha(raw) == subjects[filename])
        meta = metadata(raw, name, record['version'])
        dependencies = sorted({d['name'] for d in meta['deps'] if d['name'] in packages})
        for d in meta['deps']:
            if d['name'] in packages:
                require(d['version_req'] in (record['version'], '^' + record['version'], '=' + record['version']))
        body = envelope(meta, raw)
        rows[name] = {'name': name, 'filename': filename, 'size': len(raw), 'sha256': sha(raw),
            'metadata_sha256': sha(records.canonical(meta)), 'body_sha256': sha(body),
            'requires': dependencies}
        blobs[name] = body
    remaining = {n: set(r['requires']) for n, r in rows.items()}
    order = []
    while remaining:
        ready = sorted(n for n, deps in remaining.items() if not deps)
        require(ready)  # Includes development, optional and target name dependencies.
        for n in ready:
            order.append(n)
            del remaining[n]
        for deps in remaining.values():
            deps.difference_update(ready)
    return {'schema': 1, 'record_sha256': records.digest(record), 'source': record['source'],
        'version': record['version'], 'release_id': record['release_id'],
        'packages': [rows[n] for n in order]}, blobs


class RegistryError(Exception):
    def __init__(self, kind, status=None):
        super().__init__('registry request rejected')
        self.kind, self.status = kind, status


class NoRedirect(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, req, fp, code, msg, headers, newurl):
        return None


def request(url, method='GET', body=None):
    require(method in ('GET', 'PUT'))
    names = records.PACKAGES | records.CONSOLIDATED
    allowed_read = url == OWNER_API or any(
        url == 'https://index.crates.io/ye/tl/' + name or (
            url.startswith('https://static.crates.io/crates/' + name + '/' + name + '-')
            and records.token(url.removeprefix('https://static.crates.io/crates/' + name + '/' + name + '-').removesuffix('.crate'), records.VERSION)
            and url.endswith('.crate')) for name in names)
    require(url == API if method == 'PUT' else allowed_read)
    headers = {'Accept': 'application/json', 'User-Agent': 'Yetlet-release/1 (https://github.com/bohewu/yetlet)',
               'Cache-Control': 'no-cache'}
    if method == 'PUT':
        token = os.environ.get('CARGO_REGISTRY_TOKEN')
        require(type(token) is str and token and '\n' not in token and '\r' not in token)
        headers.update({'Authorization': token, 'Content-Type': 'application/octet-stream'})
    try:
        with urllib.request.build_opener(NoRedirect()).open(
                urllib.request.Request(url, body, headers, method=method), timeout=120) as response:
            raw = response.read(MAX + 1)
            require(len(raw) <= MAX)
            return raw
    except urllib.error.HTTPError as error:
        code = error.code
        error.close()
        raise RegistryError('http', code) from None
    except (OSError, TimeoutError):
        raise RegistryError('transport') from None


def readback(row, version, attempts=1):
    for attempt in range(attempts):
        try:
            entries = request('https://index.crates.io/ye/tl/' + row['name'])
            entries = [json.loads(line) for line in entries.splitlines() if line]
            entry = next((v for v in entries if v['vers'] == version), None)
            if entry is None:
                raise RegistryError('not_indexed')
            require(entry['name'] == row['name'] and entry['cksum'] == row['sha256']
                    and entry.get('yanked', False) is False)
            url = 'https://static.crates.io/crates/' + row['name'] + '/' + row['filename']
            raw = request(url)
            require(len(raw) == row['size'] and sha(raw) == row['sha256'])
            return {'index_sha256': entry['cksum'], 'download_sha256': sha(raw)}
        except RegistryError as error:
            if (error.kind != 'not_indexed' and error.status != 404) or attempt + 1 == attempts:
                raise
            time.sleep(2)  # Bounded readiness reads only; never retry a write.


def save(path, value):
    path = Path(path)
    temporary = path.with_name(path.name + '.tmp')
    with temporary.open('xb') as output:
        os.chmod(temporary, 0o600)
        output.write(records.canonical(value))
    os.replace(temporary, path)


def publish_response(raw):
    """Cargo publish success has optional warnings; identity comes from readback."""
    response = json.loads(raw)
    require(type(response) is dict and response.get('errors') in (None, []))
    warnings = response.get('warnings')
    require(warnings is None or type(warnings) is dict)
    counts = {}
    for key in ('invalid_categories', 'invalid_badges', 'other'):
        values = (warnings or {}).get(key)
        require(values is None or (type(values) is list and all(type(v) is str for v in values)))
        counts[key] = len(values or [])
    return counts  # Never retain provider strings, URLs or error bodies.


def continuation_rows(plan, selection):
    require(type(selection) is dict and set(selection) == {'existing', 'remaining'})
    names = [r['name'] for r in plan['packages']]
    require(len(names) == 16 and set(names) == records.PACKAGES)
    require(selection['existing'] == ['yetlet-actions'] and names[0] == 'yetlet-actions')
    require(selection['remaining'] == names[1:])
    return plan['packages'][0], plan['packages'][1:]


def require_absent(row, version):
    try:
        raw = request('https://index.crates.io/ye/tl/' + row['name'])
    except RegistryError as error:
        if error.kind == 'http' and error.status == 404:
            return
        raise
    entries = [json.loads(line) for line in raw.splitlines() if line]
    require(entries and all(e['name'] == row['name'] for e in entries))
    if any(e['vers'] == version for e in entries):
        raise RegistryError('version_exists')


def reconcile(plan, selection):
    """Read-only, known one-plus-fifteen boundary; no implicit recovery decisions."""
    existing, remaining = continuation_rows(plan, selection)
    proof = readback(existing, plan['version'])
    owners = json.loads(request(OWNER_API))
    require(any(u.get('login') == 'bohewu' and u.get('kind') == 'user' for u in owners['users']))
    for row in remaining:
        require_absent(row, plan['version'])
    return {'name': existing['name'], 'status': 'verified_existing', 'sha256': existing['sha256'],
            'owner': 'bohewu', **proof}


def publish(plan, bodies, receipt_path, *, selection=None, before_put=None):
    """Caller must authenticate the final GitHub approval and public release first."""
    require(not Path(receipt_path).exists())  # No implicit retry/resume of an uncertain batch.
    proof = {'schema': 1, 'plan_sha256': records.digest(plan), 'status': 'in_progress',
             'packages': [], 'consumer': 'not_attempted'}
    save(receipt_path, proof)
    try:
        complete = set()
        rows = plan['packages']
        if selection is not None:
            require(callable(before_put))
            existing = reconcile(plan, selection)  # All state checks precede the first PUT.
            proof['existing'] = [existing]
            complete.add(existing['name'])
            _, rows = continuation_rows(plan, selection)
            save(receipt_path, proof)
        for row in rows:
            require(set(row['requires']) <= complete and sha(bodies[row['name']]) == row['body_sha256'])
            if selection is not None:
                before_put(row)  # Caller-owned pacing and authority recheck; no PUT has been attempted yet.
                require_absent(row, plan['version'])  # Stop on state drift; never resend or silently skip.
            entry = {'name': row['name'], 'status': 'attempted', 'sha256': row['sha256']}
            proof['packages'].append(entry)
            save(receipt_path, proof)  # Record intent before a possibly successful uncertain write.
            counts = publish_response(request(API, 'PUT', bodies[row['name']]))
            entry['upload_response'] = 'accepted'
            entry['warning_counts'] = counts
            save(receipt_path, proof)
            entry.update(readback(row, plan['version'], 30))
            entry['status'] = 'verified'
            complete.add(row['name'])
            save(receipt_path, proof)
        proof['status'] = 'registry_verified_consumer_pending'
        save(receipt_path, proof)
        return proof
    except Exception as error:
        proof['status'] = 'incomplete'
        proof['error'] = {'class': error.kind, 'http_status': error.status} if isinstance(
            error, RegistryError) else {'class': 'validation', 'http_status': None}
        save(receipt_path, proof)
        raise
