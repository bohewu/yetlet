#!/usr/bin/env python3
"""Separate new verification caches/evidence; inventory and preview only, never delete."""
from __future__ import annotations

import argparse
from datetime import datetime, timezone
import json
import os
from pathlib import Path
import re
import stat
import sys

MARKER = '.yetlet-build-cache.json'
SKIP = {'.git', '.cache', 'target', 'artifacts', 'node_modules', 'dist',
        'build', '__pycache__', '.codex', '.codegraph'}
PROFILES = {'debug', 'release'}


def redirected(path):
    """Do not traverse symlinks or Windows reparse points, including ancestors."""
    for part in (path, *path.parents):
        try:
            info = part.lstat()
        except FileNotFoundError:
            continue
        if stat.S_ISLNK(info.st_mode) or getattr(info, 'st_file_attributes', 0) & 0x400:
            return True
    return False


def absolute(path):
    # abspath preserves redirections so redirected() can reject them before resolve().
    return Path(os.path.abspath(path))


def measure(path):
    """Logical bytes, not allocated disk blocks; never follow redirected paths."""
    result = {'bytes': 0, 'files': 0, 'links_skipped': 0, 'errors': []}
    seen = set()
    pending = [path]
    while pending:
        current = pending.pop()
        try:
            info = current.lstat()
            if stat.S_ISLNK(info.st_mode) or getattr(info, 'st_file_attributes', 0) & 0x400:
                result['links_skipped'] += 1
            elif stat.S_ISDIR(info.st_mode):
                pending.extend(current.iterdir())
            elif stat.S_ISREG(info.st_mode):
                identity = (info.st_dev, info.st_ino)
                if identity not in seen:
                    seen.add(identity)
                    result['bytes'] += info.st_size
                    result['files'] += 1
        except OSError as error:
            result['errors'].append({'path': str(current), 'error': type(error).__name__})
    return result


def prepare(root, run):
    if not re.fullmatch(r'[a-z0-9][a-z0-9_-]{0,79}', run):
        raise ValueError('run must be 1-80 lowercase letters, digits, hyphens or underscores')
    cache = root / '.cache/yetlet/builds' / run
    evidence = root / 'artifacts/verification' / run
    for path in (root, cache, evidence):
        if redirected(path):
            raise ValueError('redirected roots are not supported')
    if cache.exists() or evidence.exists():
        raise ValueError('run already exists; choose a new run ID, never overwrite evidence')
    evidence.mkdir(parents=True, exist_ok=False)
    cache.mkdir(parents=True, exist_ok=False)
    record = {'schema': 1, 'run': run, 'root': str(root),
              'cargo_target_dir': str(cache), 'evidence_dir': str(evidence),
              'created_utc': datetime.now(timezone.utc).isoformat()}
    (cache / MARKER).write_text(json.dumps(record, indent=2) + '\n', encoding='utf-8')
    (evidence / 'layout.json').write_text(json.dumps(record, indent=2) + '\n', encoding='utf-8')
    return record


def managed(cache, root):
    try:
        marker = cache / MARKER
        if redirected(marker):
            return False
        record = json.loads(marker.read_text(encoding='utf-8'))
        evidence = root / 'artifacts/verification' / cache.name
        return (record.get('schema') == 1 and record.get('run') == cache.name
                and record.get('root') == str(root)
                and record.get('cargo_target_dir') == str(cache)
                and record.get('evidence_dir') == str(evidence)
                and not redirected(evidence / 'layout.json')
                and json.loads((evidence / 'layout.json').read_text(encoding='utf-8')) == record)
    except (OSError, ValueError, TypeError, AttributeError):
        return False


def targets(root):
    """Discover in-checkout Cargo target roots, without entering generated trees."""
    errors = []
    found = []
    def failed(error):
        errors.append({'path': error.filename, 'error': type(error).__name__})
    for directory, children, files in os.walk(root, onerror=failed, followlinks=False):
        parent = Path(directory)
        children[:] = sorted(name for name in children
                             if name not in SKIP and not redirected(parent / name))
        if 'Cargo.toml' in files and (parent / 'target').exists():
            found.append(parent / 'target')
    return found, errors


def inventory(root, extra_targets=(), extra_evidence=()):
    if not root.is_dir() or redirected(root):
        raise ValueError('root must be an existing directory without redirection')
    rows = []
    discovery_errors = []
    def add(path, kind, reason):
        if not path.exists() and not path.is_symlink():
            return
        if redirected(path):
            rows.append({'path': str(path), 'kind': 'preserve', 'reason': 'redirected path',
                         'bytes': 0, 'files': 0, 'links_skipped': 1, 'errors': []})
            return
        data = measure(path)
        if data['errors'] or data['links_skipped']:
            kind, reason = 'preserve', 'incomplete measurement or redirected descendant'
        rows.append({'path': str(path), 'kind': kind, 'reason': reason, **data})

    legacy, discovery_errors = targets(root)
    # Reject overlapping report roots to avoid counting evidence twice or as a cache.
    selected = [root / '.cache/yetlet/builds', root / 'artifacts', *legacy,
                *map(absolute, extra_targets), *map(absolute, extra_evidence)]
    for index, left in enumerate(selected):
        for right in selected[index + 1:]:
            if left == right or left in right.parents or right in left.parents:
                raise ValueError('overlapping inventory roots; select each tree once')

    builds = root / '.cache/yetlet/builds'
    if builds.exists():
        if redirected(builds):
            add(builds, 'preserve', 'redirected path')
        else:
            for cache in sorted(builds.iterdir()):
                owned = managed(cache, root)
                add(cache, 'rebuildable-preview' if owned else 'preserve',
                    'paired new-run cache; review before any future cleanup' if owned
                    else 'unrecognized or incomplete ownership/evidence marker')
    add(root / 'artifacts', 'preserve', 'verification evidence; never a cleanup candidate')
    for evidence in map(absolute, extra_evidence):
        add(evidence, 'preserve', 'explicit evidence root; retain failures and release inputs')
    for target in [*legacy, *map(absolute, extra_targets)]:
        if redirected(target):
            add(target, 'preserve', 'redirected legacy target')
            continue
        if not target.exists():
            continue
        for child in sorted(target.iterdir()):
            # Old target directories may contain records even inside a profile.
            # Name recognition is a hint for manual review, never deletion authority.
            kind = 'legacy-review' if child.name in PROFILES else 'preserve'
            add(child, kind, 'legacy Cargo profile; inspect contents and retain evidence first'
                if kind == 'legacy-review' else 'unclassified historical output; preserve')
    return {'schema': 1, 'mode': 'read-only', 'root': str(root),
            'measured_utc': datetime.now(timezone.utc).isoformat(),
            'size_semantics': 'logical bytes; hard links deduplicated per row, not across rows',
            'discovery_errors': discovery_errors, 'entries': rows,
            'totals': {kind: sum(row['bytes'] for row in rows if row['kind'] == kind)
                       for kind in ('rebuildable-preview', 'legacy-review', 'preserve')},
            'deletion_performed': False}


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('command', choices=('prepare', 'inventory', 'plan'))
    parser.add_argument('--root', type=Path, default=Path(__file__).resolve().parent.parent)
    parser.add_argument('--run', help='unique verification run ID; prepare only')
    parser.add_argument('--target-dir', action='append', default=[], help='extra legacy target root')
    parser.add_argument('--evidence-dir', action='append', default=[], help='extra preserved evidence root')
    parser.add_argument('--json', action='store_true')
    options = parser.parse_args(argv)
    root = absolute(options.root)
    try:
        if options.command == 'prepare':
            if not options.run or options.target_dir or options.evidence_dir:
                raise ValueError('prepare requires --run and does not accept extra roots')
            if not (root / 'Cargo.toml').is_file():
                raise ValueError('prepare requires a Cargo checkout root')
            result = prepare(root, options.run)
        else:
            if options.run:
                raise ValueError('--run is only valid with prepare')
            result = inventory(root, options.target_dir, options.evidence_dir)
        if options.json or options.command == 'prepare':
            print(json.dumps(result, indent=2))
        else:
            print('READ-ONLY: no files deleted; bytes are logical sizes, not guaranteed reclaimable space.')
            for row in sorted(result['entries'], key=lambda entry: entry['bytes'], reverse=True):
                print(f"{row['bytes']:14,d}  {row['kind']:20s}  {row['path']}")
            print(json.dumps({'totals': result['totals'], 'discovery_errors': result['discovery_errors']}))
        return 0
    except (OSError, ValueError) as error:
        print(f'artifact operation rejected: {error}', file=sys.stderr)
        return 1


if __name__ == '__main__':
    sys.exit(main())
