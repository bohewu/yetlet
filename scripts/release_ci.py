"""Reuse a green SDK baseline only for an explicit release-tool/document delta."""
import fnmatch
import importlib.util
import json
import os
from pathlib import Path
import re
import subprocess

import release_policy

PATTERNS = ('docs/*', 'README.md', 'scripts/release-*.py', 'scripts/release_*.py',
            'scripts/test_release_*.py', 'scripts/bootstrap-release-*.sh',
            'scripts/release-sign-tools.txt', '.github/workflows/rust.yml',
            '.github/workflows/release*.yml', '.github/workflows/evidence-verify.yml')


def tooling_only(paths):
    return bool(paths) and all(any(fnmatch.fnmatchcase(p, pattern) for pattern in PATTERNS)
                               for p in paths)


def select(event, head, api):
    base = event['pull_request']['base']['sha'] if 'pull_request' in event else event['before']
    if not all(re.fullmatch('[0-9a-f]{40}', x) and x != '0'*40 for x in (base, head)):
        return None
    if subprocess.check_output(['git', 'rev-parse', 'HEAD'], text=True).strip() != head:
        return None
    if subprocess.run(['git', 'merge-base', '--is-ancestor', base, head], capture_output=True).returncode:
        return None
    raw = subprocess.check_output(['git', 'diff', '--name-only', '--no-renames', '-z', base, head])
    paths = raw.decode('utf-8').rstrip('\0').split('\0') if raw else []
    if not tooling_only(paths):
        return None
    return release_policy.ci(base, api)


def main():
    proof = None
    try:
        spec = importlib.util.spec_from_file_location('ci_api', Path(__file__).with_name('release-evidence-preflight.py'))
        module = importlib.util.module_from_spec(spec); spec.loader.exec_module(module)
        proof = select(json.loads(Path(os.environ['GITHUB_EVENT_PATH']).read_text()),
                       os.environ['GITHUB_SHA'], module.api)
    except Exception:
        print('No verified release-only baseline; run the complete SDK checks.')
    with open(os.environ['GITHUB_OUTPUT'], 'a') as output:
        output.write('release_only=' + ('true' if proof else 'false') + '\n')
    if proof:
        message = ('Release tooling/docs only. SDK inputs unchanged from ' + proof['source'] +
                   '; reuse successful master CI ' + proof['run'] + '/' + proof['attempt'] + '.\n')
        print(message, end='')
        with open(os.environ['GITHUB_STEP_SUMMARY'], 'a') as summary:
            summary.write(message)


if __name__ == '__main__':
    main()
