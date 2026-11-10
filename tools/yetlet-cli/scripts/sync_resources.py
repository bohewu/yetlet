#!/usr/bin/env python3
"""Refresh the CLI's package-owned public resources from a reviewed checkout."""
import json
import hashlib
from pathlib import Path
import subprocess
import tomllib

ROOT = Path(__file__).resolve().parents[3]
CLI = ROOT / 'tools/yetlet-cli'


def main():
    workspace = (ROOT / 'Cargo.toml').read_bytes().decode().replace('\r\n', '\n')
    version = tomllib.loads(workspace)['workspace']['package']['version']
    dependency = tomllib.loads((CLI / 'Cargo.toml').read_text())['dependencies']['yetlet']
    if dependency.get('version') != '=' + version or 'path' in dependency:
        raise SystemExit('Select the same published SDK version before refreshing CLI resources.')
    # The identity must name committed SDK bytes, even when CLI edits are pending.
    subprocess.run(['git', 'diff', '--quiet', 'HEAD', '--', 'Cargo.toml', 'Cargo.lock',
                    'crates/yetlet', 'assets/wasm-startup.mjs', 'assets/wasm-startup.css'],
                   cwd=ROOT, check=True)
    source = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip()
    # Only reviewed Git inputs; never copy ignored/untracked local files or secrets.
    names = subprocess.check_output(['git', 'ls-files', '-z', '--', 'crates/yetlet'], cwd=ROOT)
    files = {}
    for name in names.decode().rstrip('\0').split('\0'):
        if name == 'crates/yetlet/Cargo.lock':
            continue
        path = ROOT / name
        if not path.is_file() or any(p.is_symlink() for p in [path, *path.parents]):
            raise SystemExit('Public SDK resource must be a regular physical file.')
        files[name] = path.read_bytes().decode().replace('\r\n', '\n')
    if not files or 'crates/yetlet/src/lib.rs' not in files:
        raise SystemExit('Reviewed SDK source closure is missing.')
    (CLI / 'resources/framework.json').write_text(json.dumps(files, sort_keys=True, indent=2, ensure_ascii=False) + '\n')
    (CLI / 'resources/workspace.toml').write_text(workspace)
    (CLI / 'resources/framework.lock').write_bytes((ROOT / 'Cargo.lock').read_bytes())
    for name in ['wasm-startup.mjs', 'wasm-startup.css']:
        (CLI / 'templates/browser' / name).write_bytes((ROOT / 'assets' / name).read_bytes())
    paths = ['resources/framework.json', 'resources/workspace.toml', 'resources/framework.lock',
             'templates/browser/wasm-startup.mjs', 'templates/browser/wasm-startup.css']
    identity = {'schema': 1, 'build_sdk': version, 'snapshot_sdk': version,
                'snapshot_source': source,
                'files': {p: hashlib.sha256((CLI / p).read_bytes()).hexdigest() for p in paths},
                'policy_sha256': hashlib.sha256((CLI / 'resources/sdk-compatibility.json').read_bytes()).hexdigest()}
    (CLI / 'resources/sdk-identity.json').write_text(json.dumps(identity, sort_keys=True, indent=2) + '\n')
    print(f'Packaged {len(files)} public SDK resources for {version}.')


if __name__ == '__main__':
    main()
