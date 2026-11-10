"""Conservative language inputs for the existing four CodeQL analyses.

This selects scans, never reuses databases/results or changes extractor paths.
Uncertain history/configuration keeps full analysis. No network or dependencies.
"""
import json
import os
from pathlib import PurePosixPath
import posixpath
import re
import subprocess
import tomllib

LANGUAGES = ('actions', 'javascript-typescript', 'python', 'rust')
# Reviewed build script reads only its package and two other Rust packages
# (style-showcase-browser and yetlet-cli). All their files are Rust inputs below.
# A changed/new script fails open until its input boundary is reviewed again.
BUILD_SCRIPTS = {'examples/style-showcase/build.rs': 'e687693d286ad762d4253314b29d7771593c5a37'}
IDENTIFIER = re.compile(r'\binclude(?:_str|_bytes)?\b')
LITERAL = re.compile(r'(?:("[^"\\]*")|r(#{0,255})"([^"]*)"\2)\s*[,)}\]]')


def git(*args):
    return subprocess.check_output(['git', *args], timeout=20)


def relative(parent, child):
    path = posixpath.normpath(posixpath.join(parent, child))
    if path.startswith(('/', '../')) or path == '..' or '\\' in path:
        raise ValueError('unbounded Rust include/build input')
    return path


def skip_trivia(source, position):
    """Skip Rust whitespace/comments; block comments may nest."""
    while position < len(source):
        whitespace = re.match(r'\s+|//[^\n]*(?:\n|$)', source[position:])
        if whitespace:
            position += whitespace.end()
        elif source.startswith('/*', position):
            start, depth = position + 2, 1
            for token in re.finditer(r'/\*|\*/', source[start:]):
                depth += 1 if token[0] == '/*' else -1
                if depth == 0:
                    position = start + token.end()
                    break
            else:
                return len(source)
        else:
            break
    return position


def rust_inputs(ref):
    entries = {}
    for row in git('ls-tree', '-rz', '--full-tree', ref).split(b'\0')[:-1]:
        meta, path = row.split(b'\t', 1)
        mode, kind, oid = meta.decode().split()
        entries[path.decode()] = (mode, kind, oid)
    roots, embedded, reviewed_packages = set(), set(), set()
    uncertain = False
    for path in entries:
        if PurePosixPath(path).name != 'Cargo.toml':
            continue
        manifest = tomllib.loads(git('show', f'{ref}:{path}').decode())
        if 'package' not in manifest:
            continue
        parent = posixpath.dirname(path)
        roots.add(parent + '/' if parent else '')
        build = manifest['package'].get('build', 'build.rs')
        if build is False:
            continue
        script = relative(parent, build if isinstance(build, str) else 'build.rs')
        if script in entries:
            if entries[script][2] != BUILD_SCRIPTS.get(script):
                uncertain = True
            else:
                reviewed_packages.add(parent + '/')
        elif 'build' in manifest['package']:
            uncertain = True
    # Search identifiers, not a single-line invocation: Rust permits newlines
    # and comments before !, and all three macro delimiters are valid.
    result = subprocess.run(['git', 'grep', '-l', '-z', '-w', '-e', 'include', '-e', 'include_str', '-e', 'include_bytes',
                             ref, '--', '*.rs'], capture_output=True, timeout=20)
    if result.returncode not in (0, 1):
        raise ValueError('Rust include inventory unavailable')
    for name in result.stdout.decode().split('\0')[:-1]:
        path = name.removeprefix(ref + ':')
        source = git('show', f'{ref}:{path}').decode()
        for identifier in IDENTIFIER.finditer(source):
            bang = skip_trivia(source, identifier.end())
            if source[bang:bang + 1] != '!':
                continue  # For example, ordinary prose saying "include".
            opening = skip_trivia(source, bang + 1)
            if source[opening:opening + 1] not in ('(', '{', '['):
                uncertain = True
                continue
            argument = source[skip_trivia(source, opening + 1):]
            literal = LITERAL.match(argument)
            if literal:
                child = json.loads(literal[1]) if literal[1] else literal[3]
                embedded.add(relative(posixpath.dirname(path), child))
            elif argument.startswith('\\'):
                # Escaped macro text inside generated Rust string literals.
                continue
            elif argument.startswith('concat!(env!("OUT_DIR")') and any(
                    path.startswith(package) for package in reviewed_packages):
                continue
            else:
                uncertain = True
    query_roots = {posixpath.dirname(path) + '/' if '/' in path else '' for path in entries
                   if PurePosixPath(path).name in ('qlpack.yml', 'qlpack.yaml')}
    return roots, embedded, uncertain, query_roots


def languages_for(paths, roots=(), embedded=(), uncertain_rust=False):
    selected = {'rust'} if uncertain_rust else set()
    for path in paths:
        p = PurePosixPath(path)
        name, suffix = p.name.lower(), p.suffix.lower()
        if (path in ('.gitignore', '.gitattributes', '.gitmodules')
                or path.startswith(('.github/codeql/', '.github/scripts/', '.github/actions/'))
                or path == '.github/workflows/codeql.yml' or 'codeql' in name
                or suffix in ('.ql', '.qll', '.qls') or name in ('qlpack.yml', 'qlpack.yaml')):
            return list(LANGUAGES)
        if path in embedded or any(path.startswith(root) for root in roots):
            selected.add('rust')
        if suffix == '.rs' or name in ('cargo.toml', 'cargo.lock', 'rust-toolchain', 'rust-toolchain.toml') \
                or '.cargo' in p.parts:
            selected.add('rust')
        elif suffix in ('.py', '.pyi', '.ipynb') or name in (
                'pyproject.toml', 'setup.cfg', 'pipfile', 'pipfile.lock', 'poetry.lock', 'uv.lock', '.python-version') \
                or name.startswith('requirements') and suffix in ('.txt', '.in'):
            selected.add('python')
            if suffix == '.ipynb': selected.add('javascript-typescript')
        elif suffix in ('.js', '.jsx', '.mjs', '.cjs', '.es', '.es6', '.ts', '.tsx', '.mts', '.cts',
                        '.html', '.htm', '.xhtml', '.xhtm', '.vue', '.hbs', '.ejs', '.njk', '.json',
                        '.yaml', '.yml', '.raml', '.xml', '.css', '.svg') or name in (
                        '.npmrc', '.nvmrc', '.node-version', '.yarnrc', 'yarn.lock', '.browserslistrc'):
            selected.add('javascript-typescript')
            if path.startswith('.github/workflows/') or name in ('action.yml', 'action.yaml'):
                selected.add('actions')
        elif suffix != '.md':
            return list(LANGUAGES)  # Unknown tools/data may feed any extractor.
    return [language for language in LANGUAGES if language in selected]


def select(event, base, head):
    if event not in ('push', 'pull_request') or not all(
            re.fullmatch('[0-9a-f]{40}', ref or '') and ref != '0' * 40 for ref in (base, head)):
        return list(LANGUAGES), 'scheduled/manual or uncertain revisions: full analysis'
    try:
        raw = git('diff', '--name-only', '--no-renames', '-z', base, head, '--')
        if not raw or not raw.endswith(b'\0'):
            raise ValueError('empty/incomplete diff')
        paths = [p.decode() for p in raw.split(b'\0')[:-1]]
        before, after = rust_inputs(base), rust_inputs(head)
        if any(path.startswith(root) for path in paths for root in before[3] | after[3]):
            return list(LANGUAGES), 'query/model pack input changed: full analysis'
        selected = languages_for(paths, before[0] | after[0], before[1] | after[1], before[2] or after[2])
        return selected, f'complete Git diff: {len(paths)} changed paths; language inputs selected'
    except (OSError, ValueError, subprocess.SubprocessError):
        return list(LANGUAGES), 'history/input inventory uncertain: full analysis'


def main():
    selected, reason = select(os.environ.get('GITHUB_EVENT_NAME'), os.environ.get('BASE_SHA'), os.environ.get('HEAD_SHA'))
    value = json.dumps(selected, separators=(',', ':'))
    with open(os.environ['GITHUB_OUTPUT'], 'a', encoding='utf-8') as output:
        output.write(f'languages={value}\n')
    print(f'Languages to analyze: {value}. {reason}. Unselected languages are not analyzed; no results reused.')
    if os.environ.get('GITHUB_STEP_SUMMARY'):
        with open(os.environ['GITHUB_STEP_SUMMARY'], 'a', encoding='utf-8') as summary:
            summary.write(f'CodeQL language scope: `{value}`. {reason}.\n\n'
                          'Scheduled/manual and uncertain inputs retain full scans. No database/SARIF reuse.\n')


if __name__ == '__main__':
    main()
