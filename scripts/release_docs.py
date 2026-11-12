#!/usr/bin/env python3
"""Offline release-document checks; no packaging, credentials or publication."""
from pathlib import Path
import re
import shlex
import sys
import tomllib
import json
import hashlib
import sync_workspace_versions as sdk_versions
import release_record as records
from release_version_authority import PUBLICATION_RECORDS, published_versions


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


def markdown_links(text):
    """Inline Markdown links in package-carried documents (not prose or code URLs)."""
    for match in re.finditer(r'!?\[[^\]\n]*\]\(([^)\n]+)\)', text):
        value = match.group(1).strip().split(maxsplit=1)[0].strip('<>')
        if value:
            yield value


def check_public_readme_links(text, location):
    """crates.io renders package README outside its original repository tree."""
    for url in markdown_links(text):
        require(url.startswith('#') or re.match(r'^(?:https?://|mailto:)', url, re.I),
                location, 'package README links need public URLs or local anchors')


def check_guide_links(root, relative, package_root):
    """Package guide relatives may refer only to files carried by that package."""
    package = (root / package_root).resolve()
    for url in markdown_links(read(root, relative)):
        if url.startswith('#') or re.match(r'^(?:https?://|mailto:)', url, re.I):
            continue
        require(not re.match(r'^(?:[a-z][a-z0-9+.-]*:|//)', url, re.I),
                relative, 'unsupported guide link scheme')
        target_ref = url.split('#', 1)[0].split('?', 1)[0]
        target = ((root / relative).parent / target_ref).resolve()
        require(bool(target_ref) and target.is_relative_to(package) and target.is_file(),
                relative, 'relative guide link escapes the package or names a missing file')


SDK_SNAPSHOT_PATHS = (
    'resources/framework.json', 'resources/framework.lock', 'resources/workspace.toml',
    'templates/browser/wasm-startup.mjs', 'templates/browser/wasm-startup.css',
)
SDK_RESOURCE_PATHS = SDK_SNAPSHOT_PATHS + ('resources/sdk-identity.json', 'resources/sdk-compatibility.json')


def check_sdk_resources(files, build_sdk, readme, *, context, cli_version):
    """Check the same immutable policy/snapshot closure in source and CLI archives."""
    location = 'CLI SDK identity'
    require(all(p in files for p in SDK_RESOURCE_PATHS), location, 'resource closure incomplete')
    identity = json.loads(files['resources/sdk-identity.json'])
    require(field(readme, 'CLI release', location) == cli_version, location, 'CLI package identity differs')
    policy = records.sdk_policy(files['resources/sdk-compatibility.json'], cli_version, context=context)
    require(identity['schema'] == 1 and '=' + identity['build_sdk'] == build_sdk, location,
            'build SDK identity differs from exact manifest dependency')
    workspace = tomllib.loads(files['resources/workspace.toml'].decode())
    require(workspace['workspace']['package']['version'] == identity['snapshot_sdk'], location,
            'snapshot identity differs from embedded workspace')
    require(re.fullmatch(r'[0-9a-f]{40}', identity['snapshot_source']), location,
            'snapshot source must be an immutable Git identity')
    require(set(identity['files']) == set(SDK_SNAPSHOT_PATHS), location, 'snapshot path set differs')
    for path in SDK_SNAPSHOT_PATHS:
        require(hashlib.sha256(files[path]).hexdigest() == identity['files'][path], location,
                'snapshot resource digest differs')
    require(hashlib.sha256(files['resources/sdk-compatibility.json']).hexdigest() == identity['policy_sha256'],
            location, 'policy digest differs')
    require(field(readme, 'SDK policy', location) == policy['revision'], location,
            'README policy differs from packaged authority')



def check_cli_published_commands(text, selected, published):
    """Do not attribute later source-only Style selectors to an already shipped CLI."""
    if selected in records.CLI_SDK_VERSIONS and selected not in records.LEGACY_CLI_VERSIONS:
        heading = '## CLI command and capability inventory'
        sections = text.split(heading)
        require(len(sections) == 2, 'yetlet-cli', 'one selected CLI inventory required')
        inventory = sections[1].split('\n## ', 1)[0]
        require(all(option in inventory for option in
                    ('native-style', 'browser-style', '--sdk-version', '--offline', 'Cargo.lock')),
                'yetlet-cli', 'selected CLI inventory omits resolver/Style/lock behavior')
        return
    if selected != published:
        return
    heading = '## Published CLI command and capability inventory'
    sections = text.split(heading)
    require(len(sections) == 2, 'yetlet-cli', 'one published CLI inventory required')
    inventory = sections[1].split('\n## ', 1)[0]
    require('native-style' not in inventory and 'browser-style' not in inventory,
            'yetlet-cli', 'source-only Style templates attributed to published CLI')


def check_readme(name, version, text, sdk=None):
    """Same checks for source README and the actual hash-bound archive README."""
    check_public_readme_links(text, name)
    text = current_text(text)
    check_status(text, {'CLI' if name == 'yetlet-cli' else 'SDK': version}, name)
    if name == 'yetlet-cli':
        require(field(text, 'CLI release', name) == version, name, 'CLI version differs from manifest')
        label = 'Build SDK' if '**SDK policy:**' in text else 'Generated SDK'
        if version in records.CLI_SDK_VERSIONS and version not in records.LEGACY_CLI_VERSIONS:
            require(label == 'Build SDK', name, 'resolver CLI requires separate build SDK/policy identity')
        require(field(text, label, name) == sdk, name, 'build SDK pin differs from dependency')
        if label == 'Build SDK':
            revision = field(text, 'SDK policy', name)
            require(revision in ('sdk-compat-v1', 'sdk-latest-v2'), name,
                    'supported SDK policy required')
            if version in records.CLI_SDK_POLICIES:
                require(revision == records.CLI_SDK_POLICIES[version], name,
                        'SDK policy differs from selected CLI release')
        check_installs(text, version, name)
        if version in records.CLI_SDK_VERSIONS and version not in records.LEGACY_CLI_VERSIONS:
            check_cli_published_commands(text, version, None)
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


def check_reader_publication(edition, published):
    for subject in ('SDK', 'CLI'):
        require(edition[subject.lower()] == published[subject], 'site/pages.toml',
                f'reader {subject} must match its accepted publication; complete the reviewed reader update')


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
        resources = root / 'tools/yetlet-cli/resources'
        if (resources / 'sdk-identity.json').is_file():
            check_sdk_resources({p: (root / 'tools/yetlet-cli' / p).read_bytes() for p in SDK_RESOURCE_PATHS},
                                sdk, read(root, 'tools/yetlet-cli/README.md'), context='source', cli_version=version)
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
    if cohort in ('all', 'cli'):
        check_cli_published_commands(read(root, 'tools/yetlet-cli/README.md'),
                                     version, published['CLI'])
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
        check_guide_links(root, relative, '/'.join(relative.split('/')[:2]))
    edition = tomllib.loads(read(root, 'site/pages.toml'))['edition']
    check_reader_publication(edition, published)
    sdk_versions.check_reader_manifest(root, edition, 'examples/static-site/Cargo.toml')
    for relative in PINNED_GUIDES:
        check_installs(read(root, relative), edition['cli'], relative)
    for relative in ('docs/COMPATIBILITY.md', 'docs/DOCUMENTATION-SCOPE.md'):
        check_edition(read(root, relative), edition, relative)
    if root.resolve() == Path(__file__).resolve().parent.parent:
        sdk_versions.check(root)


def main():
    root = Path(__file__).resolve().parent.parent
    if sys.argv[1:] == ['sync']:
        changed = sdk_versions.sync(root)
        check_source(root)
        print(f'release documentation: synchronized {len(changed)} version-derived file(s)')
        for relative in changed:
            print(f'  {relative}')
        return
    require(sys.argv[1:] == ['check'], 'arguments', 'use sync or check')
    check_source(root)
    print('release documentation: PASS (manifest-derived package versions, publication records, reader edition, SDK path consumers and module guides)')


if __name__ == '__main__':
    try:
        main()
    except (ValueError, KeyError, OSError) as error:
        print(str(error), file=sys.stderr)
        sys.exit(1)
