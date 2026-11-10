"""Installed resolver/Style consumer shared by local candidates and publication.

The caller owns installation and a fresh, credential-free Cargo home. This is
qualification of an already installed executable, never a second publisher.
"""
import base64
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import socket
import subprocess
import tomllib
from urllib.parse import urljoin

import smoke_browser as smoke
import release_record as records


def require(value, message):
    if not value:
        raise ValueError('CLI consumer: ' + message)


def sha(raw):
    return hashlib.sha256(raw).hexdigest()


def protected_rust_env(root, env):
    """Copy the installed toolchain; never change shared Runner permissions."""
    rustc = Path(subprocess.check_output(
        ['rustup', 'which', '--toolchain', '1.99.0', 'rustc'], env=env,
        stderr=subprocess.PIPE, timeout=30).decode().strip())
    source = rustc.parent.parent
    target = root / 'rust'
    shutil.copytree(source, target, ignore=shutil.ignore_patterns('share'))
    # Rustup installations may be group-writable. CLI probes reject every
    # writable ancestor, so use physical binaries in an owned private tree.
    for path in [target, *target.rglob('*')]:
        path.chmod(path.stat().st_mode & ~0o022)
    return dict(env, PATH=str(target / 'bin') + os.pathsep + env['PATH'],
                RUSTUP_TOOLCHAIN=str(target))


def version_key(version):
    """Canonical exact SDK SemVer precedence, including numeric prereleases."""
    require(type(version) is str and len(version) <= 128
            and re.fullmatch(records.VERSION, version), 'noncanonical SDK version')
    core, _, pre = version.partition('-')
    parts = pre.split('.') if pre else []
    require(all(not p.isdigit() or p == '0' or not p.startswith('0') for p in parts),
            'noncanonical SDK prerelease')
    return (tuple(map(int, core.split('.'))), not pre,
            tuple((0, int(p)) if p.isdigit() else (1, p) for p in parts))


def feature_available(row, name, seen=None):
    """Check the selected template's feature closure in captured index metadata."""
    seen = set() if seen is None else seen
    if name in seen:
        return True
    seen.add(name)
    dependencies = row['deps']
    values = row.get('features2', {}).get(name, row['features'].get(name))
    if values is None:
        return any(d['name'] == name and d.get('optional', False) for d in dependencies)
    for value in values:
        if value.startswith('dep:'):
            valid = any(d['name'] == value[4:] and d.get('optional', False) for d in dependencies)
        elif '/' in value:
            valid = any(d['name'] == value.split('/', 1)[0].removesuffix('?') for d in dependencies)
        else:
            valid = feature_available(row, value, seen)
        if not valid:
            return False
    return True


def inspect_project(project, cli_version, template, requested, source, cache, policy):
    policy_bytes = policy
    policy = records.sdk_policy(policy_bytes, cli_version)
    receipt = json.loads((project / '.yetlet/sdk-resolution.json').read_bytes())
    lock = (project / 'Cargo.lock').read_bytes()
    sdk = receipt['sdk']
    require(receipt['schema'] == 1 and receipt['cli_version'] == cli_version
            and receipt['cargo_lock_sha256'] == sha(lock), 'receipt/lock identity')
    require(sdk['requested'] == requested and sdk['source'] == source
            and sdk['template'] == template and sdk['policy'] == policy['revision']
            and sdk['policy_sha256'] == sha(policy_bytes)
            and sdk['registry'] == 'https://index.crates.io/', 'resolution authority')
    names = {'yetlet', 'yetlet-browser'} if template.startswith('browser') else {'yetlet'}
    require({p['name'] for p in sdk['packages']} == names, 'SDK package set')
    captured = {r['name']: r for r in cache['records']}
    require(set(captured) == names, 'complete captured metadata set')
    rows = {name: {r['vers']: r for r in map(json.loads, captured[name]['body'].splitlines())}
            for name in names}
    historical = policy['revision'] == 'sdk-compat-v1'
    versions = policy['templates'][template] if historical else rows['yetlet']
    eligible = []
    for version in versions:
        try:
            version_key(version)
        except ValueError:
            continue  # Index build metadata is not a canonical exact SDK request.
        selected = [rows[n].get(version) for n in names]
        if not all(r and not r['yanked'] for r in selected):
            continue
        if historical and not all(r.get('rust_version') and
                (tuple(map(int, r['rust_version'].split('.'))) + (0, 0))[:3] <= (1, 99, 0)
                for r in selected):
            continue
        eligible.append(version)
    if requested is not None:
        version_key(requested)
    def selection_key(version):
        core, stable, pre = version_key(version)
        return (stable, core, pre) if historical else (core, stable, pre)
    require(eligible and sdk['version'] in eligible
            and sdk['version'] == (requested if requested is not None else max(eligible, key=selection_key)),
            'selection differs from captured metadata and policy')
    manifest = tomllib.loads((project / 'Cargo.toml').read_text())
    require(not {'patch', 'replace'} & manifest.keys(), 'SDK source override')
    dependencies = dict(manifest.get('dependencies', {}))
    for target in manifest.get('target', {}).values():
        dependencies.update(target.get('dependencies', {}))
    packages = tomllib.loads(lock.decode())['package']
    for package in sdk['packages']:
        name = package['name']
        record = captured[name]
        require(record['sha256'] == sha(record['body'].encode()) == package['metadata_sha256']
                and record['validated_at'] == package['validated_at'], 'metadata provenance')
        row = rows[name][sdk['version']]
        dependency = dependencies[name]
        required = ('dom', 'dom-keyed') if name == 'yetlet-browser' else (
            ('reactive',) if template.startswith('browser') else ('server-axum',))
        require(all(feature_available(row, f) for f in set(required) | set(dependency.get('features', []))),
                'selected SDK required feature missing or invalid')
        require(dependency['version'] == '=' + sdk['version']
                and not {'path', 'git', 'registry', 'registry-index'} & dependency.keys(), 'exact registry pin')
        locked = [p for p in packages if p['name'] == name]
        require(len(locked) == 1 and locked[0]['version'] == sdk['version']
                and locked[0]['checksum'] == row['cksum'] == package['checksum']
                and locked[0]['source'] == 'registry+https://github.com/rust-lang/crates.io-index',
                'registry lock checksum')
        if name == 'yetlet-browser':
            # Cargo already solved VersionReq during creation. Verify its actual
            # dependency edge as well as the two independently checked SDK pins.
            require(any((d.get('package') or d['name']) == 'yetlet' for d in row['deps']),
                    'browser metadata lacks Yetlet dependency')
            require(any(edge in ('yetlet', 'yetlet ' + sdk['version'],
                                 'yetlet ' + sdk['version'] + ' (registry+https://github.com/rust-lang/crates.io-index)')
                        for edge in locked[0].get('dependencies', [])), 'browser lock dependency differs')
    return receipt


def chrome_smoke(base, root, output, env):
    chrome = shutil.which('google-chrome-stable', path=env['PATH']) or shutil.which('google-chrome', path=env['PATH'])
    require(chrome, 'Chrome prerequisite missing')
    port = smoke.port_number()
    profile = root / 'chrome-profile'
    process = client = None
    result = {}
    try:
        with (output / 'browser-chrome.log').open('wb') as log:
            process = subprocess.Popen([chrome, '--headless=new', '--no-sandbox', '--disable-gpu',
                '--disable-background-networking', '--no-first-run', '--disable-extensions',
                '--remote-debugging-address=127.0.0.1', '--remote-allow-origins=*',
                f'--remote-debugging-port={port}', f'--user-data-dir={profile}', 'about:blank'],
                stdout=log, stderr=subprocess.STDOUT, start_new_session=True, env=env)
            def target():
                return next((p['webSocketDebuggerUrl'] for p in json.loads(
                    smoke.http(f'http://127.0.0.1:{port}/json/list')[2]) if p['type'] == 'page'), None)
            client = smoke.ChromeDevTools(smoke.wait_until(target, 'Chrome debugger'))
            for method in ('Runtime.enable', 'Page.enable', 'Network.enable', 'Log.enable'):
                client.call(method)
            client.call('Page.navigate', {'url': base})
            smoke.wait_until(lambda: client.evaluate(
                "document.querySelector('[data-wasm-startup]')?.dataset.startupState === 'ready'"),
                'generated browser WASM mount')
            result = client.evaluate("""(() => {
                const toolbar = getComputedStyle(document.querySelector('.app-toolbar'));
                return {padding: toolbar.paddingTop, weight: toolbar.fontWeight,
                    toolbar: toolbar.display, ready: !document.querySelector('#workspace').disabled,
                    inlineStyles: document.querySelectorAll('[style],style').length};
            })()""")
            require(result == {'padding': '12px', 'weight': '700', 'toolbar': 'flex',
                               'ready': True, 'inlineStyles': 0}, 'computed Style output')
            client.evaluate("document.querySelector('.app-toolbar input[type=checkbox]').click()")
            require(client.evaluate("getComputedStyle(document.querySelector('.app-toolbar')).paddingTop") == '12px',
                    'keyed update lost Style class')
            require(not client.exceptions and not client.failed_responses, 'browser console/CSP/network errors')
            screenshot = client.call('Page.captureScreenshot', {'format': 'png'})
            (output / 'browser-style.png').write_bytes(base64.b64decode(screenshot['data']))
            result.update(console_errors=client.exceptions, failed_responses=client.failed_responses,
                          wasm_requests=[u for u in client.requests if '.wasm' in u])
            require(result['wasm_requests'], 'actual browser WASM request')
    finally:
        if client:
            client.close()
        smoke.stop_owned_process(process, process_group=True)
        smoke.remove_owned_tree(profile)
        result.update(chrome_exit=process.returncode if process else None,
                      chrome_profile_removed=not profile.exists())
        (output / 'browser-chrome-result.json').write_text(json.dumps(result, indent=2) + '\n')
    return result


def runtime(executable, project, env, output, browser):
    label = 'browser' if browser else 'native'
    port = smoke.port_number()
    base = f'http://127.0.0.1:{port}/'
    # Preview port is an app-owned Vite setting; retain the exact customization.
    if browser:
        config = project / 'frontend/vite.config.mjs'
        original = config.read_bytes()
        require(b'port: 5173, strictPort: true' in original, 'preview port setting')
        config.write_bytes(original.replace(b'port: 5173, strictPort: true',
                                           f'port: {port}, strictPort: true'.encode()))
    process = None
    result = {'port': port, 'app_owned_preview_port': browser}
    try:
        with (output / (label + '-server.log')).open('wb') as log:
            process = subprocess.Popen([str(executable), 'run', '--manifest-path', str(project / 'Cargo.toml')],
                cwd=project, env=dict(env, YETLET_PORT=str(port)), stdout=log,
                stderr=subprocess.STDOUT, start_new_session=True)
            smoke.wait_until(lambda: smoke.http(base)[0] == 200, label + ' CLI runtime', seconds=45)
            status, headers, html = smoke.http(base)
            links = re.findall(r'<link\b[^>]*href="([^"]+\.css)"', html.decode())
            require(status == 200 and links, 'served HTML/stylesheet')
            css = b''
            for link in links:
                code, css_headers, body = smoke.http(urljoin(base, link))
                require(code == 200 and css_headers.get('content-type', '').startswith('text/css'), 'CSS HTTP/MIME')
                css += body
            require(b'.app-shell' in css and b'.app-toolbar' in css, 'served generated Style CSS')
            if browser:
                dist = project / 'frontend/dist'
                wasm = list(dist.rglob('*.wasm'))
                require(len(wasm) == 1, 'one emitted WASM asset')
                code, _, body = smoke.http(urljoin(base, wasm[0].relative_to(dist).as_posix()))
                require(code == 200 and body[:4] == b'\0asm', 'served WASM bytes')
                require(b'unsafe-inline' not in html and b'Content-Security-Policy' in html, 'production CSP')
                result['chrome'] = chrome_smoke(base, project.parent, output, env)
                result['wasm_sha256'] = sha(body)
            else:
                require(b'<script' not in html and b'class="app-shell"' in html
                        and b'Hello &lt;Yetlet&gt; &amp; Rust!' in html, 'native no-JS SSR/classes/content')
                require('unsafe-inline' not in headers.get('content-security-policy', 'unsafe-inline'), 'native CSP')
            result.update(status='PASS', html_sha256=sha(html), css_sha256=sha(css), stylesheet_links=links)
    finally:
        smoke.stop_owned_process(process, process_group=True)
        with socket.socket() as sock:
            closed = sock.connect_ex(('127.0.0.1', port)) != 0
        result.update(exit=process.returncode if process else None, port_closed=closed)
        (output / (label + '-runtime.json')).write_text(json.dumps(result, indent=2) + '\n')
        if browser:
            config.write_bytes(original)
        require(closed, 'owned server port remains open')
        if result.get('status') == 'PASS':
            require(process.returncode in (-15, 143), 'CLI cancellation/cleanup exit')
    return result


def qualify(executable, version, sdk_version, root, output, env, policy, install_source):
    """No ambient credential/config forwarding: env is supplied by the installer."""
    output.mkdir(parents=True, exist_ok=True)
    require(install_source in ('candidate-archive', 'crates.io'), 'installation provenance')
    require(not any('TOKEN' in k or k.startswith(('APP_', 'YETLET_')) for k in env), 'credential/environment leakage')
    commands = []
    def run(label, args, cwd=root, expected=0, extra_env=None):
        process = subprocess.Popen(list(map(str, args)), cwd=cwd, env=extra_env or env,
                                   stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                                   start_new_session=True, umask=0o077)
        try:
            stdout, stderr = process.communicate(timeout=900)
        except subprocess.TimeoutExpired:
            smoke.stop_owned_process(process, process_group=True)
            stdout, stderr = process.communicate(timeout=5)
            raise
        finally:
            smoke.stop_owned_process(process, process_group=True)
            if 'stdout' in locals():
                (output / (label + '.stdout')).write_bytes(stdout)
                (output / (label + '.stderr')).write_bytes(stderr)
            commands.append({'label': label, 'args': list(map(str, args)), 'exit': process.returncode})
            (output / 'commands.json').write_text(json.dumps(commands, indent=2) + '\n')
        result = subprocess.CompletedProcess(args, process.returncode, stdout, stderr)
        require(result.returncode == expected, label + ' exit ' + str(result.returncode))
        return result
    require(run('version', [executable, '--version']).stdout.strip() == ('yetlet ' + version).encode(), 'binary version')
    run('help', [executable, '--help'])
    help_text = run('new-help', [executable, 'new', '--help']).stdout.decode()
    require(all(v in help_text for v in ('native-style', 'browser-style', '--sdk-version', '--offline')), 'new options')
    empty_env = dict(env, CARGO_HOME=str(root / 'empty-cargo-home'))
    miss = run('offline-miss', [executable, 'new', 'missingapp', '--offline', '--parent', root],
               expected=1, extra_env=empty_env)
    require(b'SDK_OFFLINE_CACHE_MISS' in miss.stderr and not (root / 'missingapp').exists(), 'atomic offline miss')
    projects, runtimes = {}, {}
    for label, template, exact in [('default', 'native', False), ('native', 'native', True),
                                   ('browser', 'browser', True), ('native-style', 'native-style', True),
                                   ('browser-style', 'browser-style', True), ('offline', 'browser-style', False)]:
        name = 'consumer-' + label
        args = [executable, 'new', name, '--parent', root]
        if label != 'default':
            args += ['--template', template]
        if exact:
            args += ['--sdk-version', sdk_version]
        if label == 'offline':
            args += ['--offline']
        run(label + '-new', args)
        project = root / name
        cache_file = Path(env['CARGO_HOME']) / ('yetlet/sdk-index-v1/crates-io-' +
                     ('browser' if template.startswith('browser') else 'native') + '.json')
        cache = json.loads(cache_file.read_bytes())
        receipt = inspect_project(project, version, template, sdk_version if exact else None,
            'offline-cache' if label == 'offline' else 'registry-refresh', cache, policy)
        (output / (label + '-metadata.json')).write_bytes(cache_file.read_bytes())
        (output / (label + '-resolution.json')).write_text(json.dumps(receipt, indent=2) + '\n')
        lock = (project / 'Cargo.lock').read_bytes()
        (output / (label + '-Cargo.lock')).write_bytes(lock)
        run(label + '-repeat', args, expected=1)
        require((project / 'Cargo.lock').read_bytes() == lock, 'repeat changed existing lock')
        projects[label] = {'sdk_version': receipt['sdk']['version'], 'source': receipt['sdk']['source'],
                           'lock_sha256': sha(lock)}
        if label in ('native-style', 'browser-style'):
            closure = {p.relative_to(project).as_posix(): sha(p.read_bytes())
                       for directory in ('src', 'stylegen') for p in (project / directory).rglob('*') if p.is_file()}
            (output / (label + '-source-closure.json')).write_text(json.dumps(closure, indent=2) + '\n')
            (output / (label + '-Cargo.toml')).write_bytes((project / 'Cargo.toml').read_bytes())
            style_lock = (project / 'stylegen/Cargo.lock').read_bytes()
            (output / (label + '-stylegen-Cargo.lock')).write_bytes(style_lock)
            style_manifest = tomllib.loads((project / 'stylegen/Cargo.toml').read_text())
            require(not style_manifest.get('dependencies') and not {'patch', 'replace'} & style_manifest.keys(),
                    'standalone dependency-free stylegen')
            run(label + '-fetch', ['cargo', 'fetch', '--locked'], cwd=project)
            if template.startswith('browser'):
                run(label + '-npm-ci', ['npm', 'ci', '--ignore-scripts', '--no-audit', '--no-fund'], cwd=project / 'frontend')
            run(label + '-build', [executable, 'build', '--manifest-path', project / 'Cargo.toml'], cwd=project)
            runtimes[label] = runtime(executable, project, env, output, template.startswith('browser'))
            require((project / 'Cargo.lock').read_bytes() == lock
                    and (project / 'stylegen/Cargo.lock').read_bytes() == style_lock, 'build/run changed locks')
    result = {'status': 'PASS', 'cli_exact_version': version, 'cli_install_source': install_source,
              'credentials_forwarded': False, 'path_patches': False, 'projects': projects, 'runtime': runtimes}
    (output / 'CLI-CONSUMER.json').write_text(json.dumps(result, indent=2) + '\n')
    return result
