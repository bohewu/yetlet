#!/usr/bin/env python3
"""Offline release-document checks; no packaging, credentials or publication."""
from pathlib import Path
import re
import shlex
import sys
import tomllib


# Current reader entry points only. Dated task/evidence records retain history.
CURRENT_DOCS = (
    'README.md', 'docs/README.md', 'docs/API-REFERENCE.md', 'docs/CAPABILITIES.md',
    'docs/COMPATIBILITY.md', 'docs/DOCUMENTATION-SCOPE.md',
    'docs/RELEASE-PUBLICATION.md', 'tools/yetlet-cli/README.md',
    'crates/yetlet/README.md', 'crates/yetlet-browser/README.md',
    'crates/yetlet-browser/guides/dom.md', 'examples/static-site/README.md',
    'examples/docs-site/README.md',
    'docs/ARCHITECTURE.md', 'docs/RELEASE-READINESS.md',
    'docs/VIEW-COMPONENTS.md', 'docs/EXTENSIONS.md', 'docs/HTTP-PIPELINE.md',
    'docs/CONFIGURATION.md', 'docs/SECRET-PROVIDERS.md',
    'docs/SETTINGS-SECRET-PROVIDER.md', 'docs/SETTINGS-OPTIONS-LOGGING.md',
)
PINNED_GUIDES = ('README.md', 'docs/FIRST-APP.md')
# Existing accepted records, not development manifests, establish distribution.
PUBLICATION_RECORDS = {
    'SDK': ('docs/ALPHA6-RELEASE-PREPARATION.md', 'registry consumer'),
    'CLI': ('docs/CLI-020-RELEASE-PREPARATION.md', 'installed consumer'),
}
STALE = re.compile(
    r'publication\s+(?:is\s+)?pending|'
    r'unreleased\s+(?:checkout|opt-in|static\s+HTML|ordinary\s+HTML|controlled\s+DOM)|'
    r'(?:these\s+(?:APIs|additions)|this\s+(?:release|version|mode))\s+'
    r'(?:are|is)\s+not\s+yet\s+(?:available|distributed|published)', re.I)


def require(ok, location, reason):
    if not ok:
        raise ValueError(f'release documentation rejected: {location}: {reason}')


def current_text(text):
    """Explicit historical sections are not current availability statements."""
    return re.split(r'(?m)^## Historical\b', text, maxsplit=1)[0]


def check_status(text, versions, location):
    text = current_text(text).replace('`', '').replace('**', '')
    require(not STALE.search(text), location, 'stale current availability wording')
    for paragraph in re.split(r'\n\s*\n|(?m:^[*-] )', text):
        paragraph = ' '.join(paragraph.split())
        for subject, version in versions.items():
            labels = [version]
            if subject == 'SDK' and '-' in version:
                labels.append(version.split('-', 1)[1])
            selected = any(re.search(r'\b' + subject + r'(?:\s+(?:release|version))?\s*:?\s*' + re.escape(label) +
                                    r'(?![0-9A-Za-z+-]|\.[0-9A-Za-z-])', paragraph, re.I) for label in labels)
            pending = re.search(r'\bpending\b|\bunreleased\b|\bnot yet (?:available|published|distributed)\b', paragraph, re.I)
            require(not (selected and pending), location, 'selected version marked pending/unreleased')


def field(text, label, location):
    values = re.findall(r'\*\*' + re.escape(label) + r':\*\*\s*`([^`\n]+)`', text)
    require(len(values) == 1, location, f'exactly one {label} reference required')
    return values[0]


def check_installs(text, version, location):
    commands = re.findall(r'\bcargo(?:\s+\+[^\s]+)?\s+install\s+yetlet-cli(?=\s|$)[^\n]*',
                          current_text(text).replace('\\\n', ' '))
    require(bool(commands), location, 'explicit registry installation command required')
    for command in commands:
        args = shlex.split(command)
        require(not any(arg.split('=', 1)[0] in ('--git', '--path', '--index', '--registry')
                        for arg in args), location, 'registry recipe cannot select another source')
        values = []
        for index, arg in enumerate(args):
            if arg == '--version' and index + 1 < len(args):
                values.append(args[index + 1])
            elif arg.startswith('--version='):
                values.append(arg.split('=', 1)[1])
        require(values == [version] and '--locked' in args, location,
                'installation must select the exact documented version with --locked')


def check_readme(name, version, text, sdk=None):
    """Same checks for source README and the actual hash-bound archive README."""
    text = current_text(text)
    check_status(text, {'CLI' if name == 'yetlet-cli' else 'SDK': version}, name)
    if name == 'yetlet-cli':
        require(field(text, 'CLI release', name) == version, name, 'CLI version differs from manifest')
        require(field(text, 'Generated SDK', name) == sdk, name, 'SDK pin differs from dependency')
        check_installs(text, version, name)
        outputs = re.findall(r'`yetlet ([0-9][0-9A-Za-z.+-]*)`', text)
        require(bool(outputs) and all(value == version for value in outputs), name,
                'expected binary version differs from manifest')
    else:
        introduction = re.split(r'(?m)^## ', text, maxsplit=1)[0]
        values = re.findall(r'\bversion\s+`([^`\n]+)`', introduction, re.I)
        require(values == [version], name, 'SDK version differs from manifest')
        for snippet in re.findall(r'(?ms)^```toml[^\n]*\n(.*?)^```', text):
            dependencies = tomllib.loads(snippet).get('dependencies', {})
            for package in ('yetlet', 'yetlet-browser'):
                if package in dependencies:
                    dependency = dependencies[package]
                    pin = dependency if isinstance(dependency, str) else dependency.get('version')
                    require(pin == '=' + version, name, 'SDK example pin differs from manifest')
    # A frozen package README must work before and after upload. Availability is
    # recorded separately after successful publication, never guessed in C.
    require(not re.search(r'\b(?:is|are|already)\s+(?:now\s+)?published\b',
                          current_text(text), re.I), name, 'use release-neutral package wording')


def read(root, relative):
    path = root / relative
    require(path.is_file() and not path.is_symlink(), relative, 'regular source file required')
    require(path.stat().st_size <= 1024 * 1024, relative, 'document is too large')
    return path.read_text(encoding='utf-8')


def published_versions(root):
    versions = {}
    for subject, (relative, consumer) in PUBLICATION_RECORDS.items():
        text = read(root, relative)
        sections = re.split(r'(?m)^## Owner-approved publication and ' + consumer + r'[^\n]*\n', text)
        require(len(sections) == 2, relative, 'accepted publication section required')
        section = re.split(r'(?m)^## ', sections[1], maxsplit=1)[0]
        prefix = 'yetlet-cli-v' if subject == 'CLI' else 'v'
        values = re.findall(r'https://github\.com/bohewu/yetlet/releases/tag/' + prefix +
                            r'([0-9][0-9A-Za-z.+-]*)\)', section)
        require(len(values) == 1, relative, 'one accepted release tag reference required')
        versions[subject] = values[0]
    return versions


def check_distribution(text, versions, location):
    for subject, version in versions.items():
        require(field(current_text(text), 'Published ' + subject, location) == version,
                location, 'current distribution differs from accepted publication record')


def check_edition(text, edition, location):
    paragraphs = [p for p in re.split(r'\n\s*\n', current_text(text))
                  if 'public reader/first-app edition' in ' '.join(p.split())]
    require(len(paragraphs) == 1, location, 'one current reader-edition paragraph required')
    labels = re.findall(r'\b(SDK|CLI)\s*([0-9][0-9A-Za-z.+-]*|alpha\.[0-9]+)',
                        paragraphs[0].replace('`', ''))
    expected = {'SDK': edition['sdk'], 'CLI': edition['cli']}
    require({subject for subject, _ in labels} == set(expected), location,
            'reader edition must identify SDK and CLI')
    for subject, value in labels:
        version = expected[subject]
        require(value.rstrip('.') in (version, version.split('-', 1)[-1]), location,
                'reader edition differs from site/pages.toml')


def check_current_module(text, location):
    # Only maintained module guides/current composition prose; old package
    # READMEs and explicit historical sections remain valid evidence.
    text = re.split(r'(?m)^<details>\n<summary>Historical\b', current_text(text), maxsplit=1)[0]
    require(not re.search(r'\bunpublished\b|sixteen library candidates|'
                          r'public activation remains blocked', text, re.I),
            location, 'stale current module distribution wording')


def check_source(root, cohort='all'):
    root = Path(root)
    require(cohort in ('all', 'cli', 'consolidated'), 'cohort', 'unsupported current cohort')
    workspace = tomllib.loads(read(root, 'Cargo.toml'))['workspace']['package']['version']
    cli = tomllib.loads(read(root, 'tools/yetlet-cli/Cargo.toml'))
    version = cli['package']['version']
    sdk = cli['dependencies']['yetlet']['version']
    if cohort in ('all', 'cli'):
        check_readme('yetlet-cli', version, read(root, 'tools/yetlet-cli/README.md'), sdk)
    if cohort in ('all', 'consolidated'):
        for name in ('yetlet', 'yetlet-browser'):
            check_readme(name, workspace, read(root, f'crates/{name}/README.md'))
    versions = {}
    if cohort in ('all', 'cli'):
        versions['CLI'] = version
    if cohort in ('all', 'consolidated'):
        versions['SDK'] = workspace
    for relative in CURRENT_DOCS:
        check_status(read(root, relative), versions, relative)
    for relative in ('docs/COMPATIBILITY.md', 'docs/VIEW-COMPONENTS.md'):
        require('feature/view-authoring-api' not in current_text(read(root, relative)),
                relative, 'obsolete typed-authoring branch guidance')
    published = published_versions(root)
    for relative in ('docs/ARCHITECTURE.md', 'docs/RELEASE-READINESS.md'):
        check_distribution(read(root, relative), published, relative)
    for relative in ('AGENTS.md', 'docs/ARCHITECTURE.md'):
        introduction = read(root, relative).split('## ', 1)[0] if relative != 'AGENTS.md' else read(root, relative).split('## Documentation language', 1)[0]
        require(not re.search(r'\bunpublished\b|nine library guides|'
                              r'Public release and production qualification remain open', introduction),
                relative, 'stale project entry status')
    module_guides = sorted((root / 'crates/yetlet/guides').glob('*.md'))
    module_guides += sorted((root / 'crates/yetlet-browser/guides').glob('*.md'))
    for path in module_guides:
        relative = path.relative_to(root).as_posix()
        check_current_module(read(root, relative), relative)
    edition = tomllib.loads(read(root, 'site/pages.toml'))['edition']
    for relative in PINNED_GUIDES:
        check_installs(read(root, relative), edition['cli'], relative)
    for relative in ('docs/COMPATIBILITY.md', 'docs/DOCUMENTATION-SCOPE.md'):
        check_edition(read(root, relative), edition, relative)


def main():
    require(sys.argv[1:] == ['check'], 'arguments', 'use check')
    check_source(Path(__file__).resolve().parent.parent)
    print('release documentation: PASS (manifest references, installation, status and pinned edition)')


if __name__ == '__main__':
    try:
        main()
    except (ValueError, KeyError, OSError) as error:
        print(str(error), file=sys.stderr)
        sys.exit(1)
