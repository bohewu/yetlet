#!/usr/bin/env python3
"""Refresh the CLI's package-owned public resources from a reviewed checkout."""
import json
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
    print(f'Packaged {len(files)} public SDK resources for {version}.')


if __name__ == '__main__':
    main()
