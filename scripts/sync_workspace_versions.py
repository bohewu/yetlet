#!/usr/bin/env python3
"""Version synchronization for current Yetlet SDK consumers and docs."""
from pathlib import Path
import argparse
import re
import subprocess
import tomllib
from release_version_authority import published_versions, publication_links
from release_record import cli_sdk_version

ROOT = Path(__file__).resolve().parents[1]
SDK_PACKAGES = ("yetlet", "yetlet-browser")
SECTIONS = ("dependencies", "dev-dependencies", "build-dependencies")
VERSION_PATTERN = r"[0-9]+\.[0-9]+\.[0-9]+(?:-[0-9A-Za-z.-]+)?(?:\+[0-9A-Za-z.-]+)?"
VERSION_SHORT_PATTERN = r"[0-9A-Za-z.+-]+"
FEATURE_INTRODUCTION_PATTERN = re.compile(
    r"\b(?:since|introduced\s+(?:in|with)|available\s+(?:since|from|in)|"
    r"opt[- ]in\s+since|first\s+available\s+in|added\s+in|requires)\s+SDK\s*\x60?"
    + VERSION_SHORT_PATTERN + r"\x60?",
    re.IGNORECASE,
)



def manifest_paths(root):
    raw = subprocess.check_output(["git", "ls-files", "-z", "--cached", "--others", "--exclude-standard", "--", "*Cargo.toml"], cwd=root)
    return sorted({Path(x.decode()) for x in raw.split(b"\0") if x})


TOML_KEY_PATTERN = r"""(?:[A-Za-z0-9_-]+|"(?:\\.|[^"\\])*"|'[^']*')"""


def dependency_tables(manifest):
    tables = []
    for section in SECTIONS:
        table = manifest.get(section)
        if isinstance(table, dict):
            tables.append(((section,), table))
    targets = manifest.get("target", {})
    if isinstance(targets, dict):
        for target_name, target in targets.items():
            if isinstance(target, dict):
                for section in SECTIONS:
                    table = target.get(section)
                    if isinstance(table, dict):
                        tables.append((("target", target_name, section), table))
    workspace = manifest.get("workspace", {})
    if isinstance(workspace, dict):
        for section in SECTIONS:
            table = workspace.get(section)
            if isinstance(table, dict):
                tables.append((("workspace", section), table))
    return tables


def sdk_package_roots(root):
    roots = {}
    for name in SDK_PACKAGES:
        manifest_path = root / "crates" / name / "Cargo.toml"
        try:
            package = tomllib.loads(manifest_path.read_text(encoding="utf-8")).get("package")
        except (OSError, tomllib.TOMLDecodeError) as exc:
            raise ValueError(f"{manifest_path}: cannot verify SDK package identity") from exc
        if not isinstance(package, dict) or package.get("name") != name:
            raise ValueError(f"{manifest_path}: expected package identity {name}")
        roots[name] = manifest_path.parent.resolve()
    return roots


def matching_dependencies(root, relative, parsed):
    sdk_roots = sdk_package_roots(root)
    found = []
    for table_path, table in dependency_tables(parsed):
        for alias, dep in table.items():
            if not isinstance(dep, dict) or "path" not in dep:
                continue
            package = dep.get("package", alias)
            is_sdk_package = isinstance(package, str) and package in SDK_PACKAGES
            dependency_path = dep["path"]
            if not isinstance(dependency_path, str) or not dependency_path.strip():
                if alias in SDK_PACKAGES or is_sdk_package:
                    raise ValueError(f"{relative}: invalid SDK path declaration for {alias}")
                continue
            target = (root / relative).parent.joinpath(dependency_path).resolve()
            target_package = next(
                (name for name, sdk_root in sdk_roots.items() if sdk_root == target),
                None,
            )
            if target_package is None:
                if alias in SDK_PACKAGES or is_sdk_package:
                    raise ValueError(
                        f"{relative}: SDK path for {alias} does not resolve to its declared package"
                    )
                continue
            if package != target_package:
                raise ValueError(
                    f"{relative}: {alias} path resolves to SDK package {target_package}, "
                    f"but package identity is {package!r}"
                )
            found.append((table_path, alias, target_package, dependency_path))
    return found


def parse_table_header(line, location):
    stripped = line.strip()
    if not stripped.startswith("[") or stripped.startswith("[["):
        return None
    try:
        parsed = tomllib.loads(stripped + "\n")
    except tomllib.TOMLDecodeError as exc:
        raise ValueError(f"{location}: cannot parse TOML table header") from exc
    path = []
    node = parsed
    while isinstance(node, dict) and len(node) == 1:
        key, child = next(iter(node.items()))
        if not isinstance(child, dict):
            break
        path.append(key)
        node = child
    if not path:
        raise ValueError(f"{location}: ambiguous TOML table header")
    return tuple(path)


def assignment_key(line, location):
    match = re.match(r"^\s*(?P<key>" + TOML_KEY_PATTERN + r")\s*=", line)
    if not match:
        return None
    try:
        parsed = tomllib.loads(match.group("key") + " = 0\n")
    except tomllib.TOMLDecodeError as exc:
        raise ValueError(f"{location}: cannot parse dependency key") from exc
    if len(parsed) != 1:
        raise ValueError(f"{location}: ambiguous dependency key")
    return next(iter(parsed))


def inline_table_close(line, opening, location, alias):
    depth = 0
    quote = None
    escaped = False
    for index in range(opening, len(line)):
        char = line[index]
        if quote is not None:
            if quote == '"' and escaped:
                escaped = False
            elif quote == '"' and char == "\\":
                escaped = True
            elif char == quote:
                quote = None
            continue
        if char in ("'", '"'):
            quote = char
        elif char == "#":
            break
        elif char == "{":
            depth += 1
        elif char == "}":
            depth -= 1
            if depth == 0:
                return index
    raise ValueError(f"{location}: unsupported multiline inline dependency {alias}")


def split_inline_table_fields(body, location, alias):
    fields = []
    start = 0
    quote = None
    escaped = False
    array_depth = 0
    table_depth = 0
    for index, char in enumerate(body):
        if quote is not None:
            if quote == '"' and escaped:
                escaped = False
            elif quote == '"' and char == "\\":
                escaped = True
            elif char == quote:
                quote = None
            continue
        if char in ("'", '"'):
            quote = char
        elif char == "[":
            array_depth += 1
        elif char == "]":
            array_depth -= 1
        elif char == "{":
            table_depth += 1
        elif char == "}":
            table_depth -= 1
        elif char == "," and array_depth == 0 and table_depth == 0:
            fields.append((start, index))
            start = index + 1
        if array_depth < 0 or table_depth < 0:
            raise ValueError(f"{location}: malformed inline dependency {alias}")
    if quote is not None or array_depth or table_depth:
        raise ValueError(f"{location}: malformed inline dependency {alias}")
    fields.append((start, len(body)))
    return fields


def replace_inline_table_version(line, version, location, alias):
    key_match = re.match(r"^\s*(?:" + TOML_KEY_PATTERN + r")\s*=", line)
    if not key_match:
        raise ValueError(f"{location}: cannot locate dependency key {alias}")
    value_start = key_match.end()
    while value_start < len(line) and line[value_start].isspace():
        value_start += 1
    if value_start >= len(line) or line[value_start] != "{":
        raise ValueError(f"{location}: {alias} must use a single-line inline table")
    close = inline_table_close(line, value_start, location, alias)
    body = line[value_start + 1:close]
    version_fields = []
    for start, end in split_inline_table_fields(body, location, alias):
        field = body[start:end]
        if not field.strip():
            continue
        try:
            parsed_field = tomllib.loads(field.strip() + "\n")
        except tomllib.TOMLDecodeError as exc:
            raise ValueError(f"{location}: unsupported inline field in {alias}") from exc
        if "version" in parsed_field:
            if set(parsed_field) != {"version"} or not isinstance(parsed_field["version"], str):
                raise ValueError(f"{location}: invalid version field in {alias}")
            version_fields.append((start, end, field))
    if len(version_fields) > 1:
        raise ValueError(f"{location}: ambiguous version fields in {alias}")
    if not version_fields:
        new_body = (f' version = "{version}",' + body) if body.strip() else f' version = "{version}"' + body
    else:
        start, end, field = version_fields[0]
        match = re.fullmatch(
            r"""(?P<prefix>\s*version\s*=\s*)(?P<quote>['"])(?P<value>[^'"]*)(?P=quote)(?P<suffix>\s*)""",
            field,
        )
        if not match:
            raise ValueError(f"{location}: unsupported version literal in {alias}")
        updated_field = (
            match.group("prefix") + match.group("quote") + version +
            match.group("quote") + match.group("suffix")
        )
        new_body = body[:start] + updated_field + body[end:]
    updated = line[:value_start + 1] + new_body + line[close:]
    try:
        parsed = tomllib.loads(updated)
    except tomllib.TOMLDecodeError as exc:
        raise ValueError(f"{location}: rewritten dependency {alias} is invalid TOML") from exc
    entry = next(iter(parsed.values()))
    if not isinstance(entry, dict) or entry.get("version") != version:
        raise ValueError(f"{location}: rewritten dependency {alias} failed version readback")
    return updated


def multiline_string_close(line, start, delimiter):
    quote = delimiter[0]
    index = start
    while index < len(line):
        if delimiter == '"""' and line[index] == "\\":
            index += 2
            continue
        if line[index] == quote:
            run_end = index
            while run_end < len(line) and line[run_end] == quote:
                run_end += 1
            if run_end - index >= 3:
                return run_end
            index = run_end
            continue
        index += 1
    return None


def manifest_code_lines(text, location):
    """Hide TOML multiline-string content from the source-line rewriter."""
    result = []
    active_delimiter = None
    for line in text.splitlines(keepends=True):
        quote = None
        escaped = False
        multiline_started = False
        index = 0
        if active_delimiter is not None:
            close = multiline_string_close(line, 0, active_delimiter)
            if close is None:
                result.append(None)
                continue
            active_delimiter = None
            multiline_started = True
            index = close

        while index < len(line):
            char = line[index]
            if quote is not None:
                if quote == '"' and escaped:
                    escaped = False
                elif quote == '"' and char == "\\":
                    escaped = True
                elif char == quote:
                    quote = None
                index += 1
                continue
            if char == "#":
                break
            if line.startswith('"""', index) or line.startswith("'''", index):
                delimiter = line[index:index + 3]
                multiline_started = True
                close = multiline_string_close(line, index + 3, delimiter)
                if close is None:
                    active_delimiter = delimiter
                    break
                index = close
                continue
            if char in ("'", '"'):
                quote = char
            index += 1
        result.append(None if multiline_started else line)

    if active_delimiter is not None:
        raise ValueError(f"{location}: unterminated TOML multiline string")
    return result


def rewrite_manifest(root, relative, text, dependencies, version):
    sdk_roots = sdk_package_roots(root)
    specs = {}
    for table_path, alias, package, dependency_path in dependencies:
        key = (table_path, alias)
        if key in specs:
            raise ValueError(f"{relative}: ambiguous dependency binding for {alias}")
        specs[key] = (package, dependency_path)
    lines = text.splitlines(keepends=True)
    source_lines = manifest_code_lines(text, relative)
    current_table = None
    found = set()
    for index, line in enumerate(source_lines):
        if line is None:
            continue
        if line.lstrip().startswith("["):
            current_table = parse_table_header(line, relative)
            continue
        alias = assignment_key(line, relative)
        if alias is None:
            continue
        key = (current_table, alias)
        if key not in specs:
            continue
        if key in found:
            raise ValueError(f"{relative}: duplicate dependency declaration for {alias}")
        try:
            entry = tomllib.loads(line)
        except tomllib.TOMLDecodeError as exc:
            raise ValueError(
                f"{relative}: unsupported multiline dependency declaration {alias}"
            ) from exc
        if set(entry) != {alias}:
            raise ValueError(f"{relative}: ambiguous dependency declaration for {alias}")
        dependency = entry[alias]
        package, dependency_path = specs[key]
        if (
            not isinstance(dependency, dict)
            or dependency.get("path") != dependency_path
            or dependency.get("package", alias) != package
        ):
            raise ValueError(f"{relative}: dependency binding changed for {alias}")
        target = (root / relative).parent.joinpath(dependency_path).resolve()
        if target != sdk_roots[package]:
            raise ValueError(f"{relative}: dependency path no longer resolves to {package}")
        updated_line = replace_inline_table_version(line, version, relative, alias)
        updated_entry = tomllib.loads(updated_line)[alias]
        if (
            updated_entry.get("version") != version
            or updated_entry.get("path") != dependency_path
            or updated_entry.get("package", alias) != package
        ):
            raise ValueError(f"{relative}: rewritten dependency identity changed for {alias}")
        lines[index] = updated_line
        found.add(key)
    missing = sorted(set(specs) - found)
    if missing:
        table_path, alias = missing[0]
        raise ValueError(
            f"{relative}: cannot locate exact single-line dependency {alias} "
            f"in [{'.'.join(table_path)}]"
        )
    return "".join(lines)


def sync_manifests(root, paths, version):
    updates = {}
    inventory = []
    for relative in paths:
        text = (root / relative).read_text(encoding="utf-8")
        parsed = tomllib.loads(text)
        dependencies = matching_dependencies(root, relative, parsed)
        if not dependencies:
            continue
        aliases = sorted({alias for _, alias, _, _ in dependencies})
        inventory.append((relative.as_posix(), aliases))
        updated = rewrite_manifest(root, relative, text, dependencies, version)
        if updated != text:
            updates[relative.as_posix()] = updated
    return updates, inventory


def split_historical(text):
    markers = [m.start() for pattern in
               (r"(?m)^<details>\n<summary>Historical\b", r"(?m)^## Historical\b")
               for m in re.finditer(pattern, text)]
    if not markers:
        return text, ""
    index = min(markers)
    return text[:index], text[index:]


def sync_guide_text(text, version, location):
    current, historical = split_historical(text)
    protected = []

    def preserve_feature_introduction(match):
        marker = "\x00YT246_FEATURE_INTRO_" + str(len(protected)) + "\x00"
        protected.append((marker, match.group(0)))
        return marker

    current = FEATURE_INTRODUCTION_PATTERN.sub(preserve_feature_introduction, current)
    tick = chr(96)
    pattern = r"(\bSDK\s+" + re.escape(tick) + r")" + VERSION_PATTERN + r"(" + re.escape(tick) + r")"
    current = re.sub(pattern, lambda m: m.group(1) + version + m.group(2), current)
    for marker, original in protected:
        current = current.replace(marker, original)
    current = sync_readme_dependencies(current, SDK_PACKAGES, version, location)
    return current + historical


def sync_guides(root, version, write=False):
    changed = []
    for base in (root / "crates/yetlet/guides", root / "crates/yetlet-browser/guides"):
        for path in sorted(base.glob("*.md")):
            original = path.read_text(encoding="utf-8")
            updated = sync_guide_text(original, version, path.relative_to(root).as_posix())
            if updated != original:
                changed.append(path.relative_to(root).as_posix())
                if write:
                    path.write_text(updated, encoding="utf-8")
    return changed


def sync_current_docs(root, version, write=False):
    changes = {}
    tick = chr(96)

    relative = "docs/COMPATIBILITY.md"
    text = (root / relative).read_text(encoding="utf-8")
    current, historical = split_historical(text)
    distribution = re.search(r"(?m)^## Current distribution\b", current)
    if distribution:
        local, remainder = current[:distribution.start()], current[distribution.start():]
    else:
        local, remainder = current, ""
    local = re.sub(r"(?m)^## Selected local SDK [^ \n]+ (.*)$",
                   lambda m: f"## Selected local SDK {version} {m.group(1)}", local)
    local, count = re.subn(r"(selects SDK\s*\n\s*" + re.escape(tick) +
                           r")" + VERSION_PATTERN + r"(" + re.escape(tick) + r")",
                           lambda m: m.group(1) + version + m.group(2), local, count=1)
    if count != 1:
        raise ValueError(relative + ": selected local SDK version source field missing")
    updated = local + remainder + historical
    if updated != text:
        changes[relative] = updated

    specifications = {
        "docs/API-REFERENCE.md": (
            r"(selects SDK " + re.escape(tick) + r")" + VERSION_PATTERN + r"(" +
            re.escape(tick) + r" locally)"),
        "docs/CAPABILITIES.md": (
            r"(selects local SDK\s*" + re.escape(tick) +
            r")" + VERSION_PATTERN + r"(" + re.escape(tick) + r" contents)"),
        "docs/VIEW-COMPONENTS.md": (
            r"(The locally selected SDK " + re.escape(tick) +
            r")" + VERSION_PATTERN + r"(" + re.escape(tick) + r" includes)"),
    }
    for relative, pattern in specifications.items():
        text = (root / relative).read_text(encoding="utf-8")
        current, historical = split_historical(text)
        updated, count = re.subn(pattern, lambda m: m.group(1) + version + m.group(2),
                                 current, count=1)
        if count != 1:
            raise ValueError(relative + ": selected local SDK version source field missing")
        updated += historical
        if updated != text:
            changes[relative] = updated
    if write:
        for relative, value in changes.items():
            (root / relative).write_text(value, encoding="utf-8")
    return changes




def replace_dependency_lines(text, package, version, exact=True, location="dependency"):
    lines = text.splitlines(keepends=True)
    indexes = [i for i, line in enumerate(lines)
               if re.match(r"^\s*" + re.escape(package) + r"\s*=", line)]
    if not indexes:
        raise ValueError(f"{location}: missing {package} dependency")
    required = ("=" if exact else "") + version
    for index in indexes:
        line = lines[index]
        if "{" in line:
            line, count = re.subn(r'(version\s*=\s*)"(?:=)?[^"]+"',
                                  lambda m: m.group(1) + f'"{required}"', line, count=1)
        else:
            prefix, rhs = line.split("=", 1)
            rhs, count = re.subn(r'"(?:=)?[^"]+"', f'"{required}"', rhs, count=1)
            line = prefix + "=" + rhs
        if count != 1:
            raise ValueError(f"{location}: cannot update {package} dependency version")
        lines[index] = line
    return "".join(lines)


def sync_readme_dependencies(text, packages, version, location):
    fence = chr(96) * 3
    def update(match):
        block = match.group(1)
        parsed = tomllib.loads(block)
        deps = parsed.get("dependencies", {})
        for package in packages:
            if package in deps:
                block = replace_dependency_lines(block, package, version, True, location)
        return fence + "toml\n" + block + fence
    return re.sub(r"(?ms)^" + re.escape(fence) + r"toml\n(.*?)^" + re.escape(fence), update, text)


def sync_package_readmes(root, sdk, cli_version, cli_sdk):
    updates = {}
    specs = [
        ("crates/yetlet/README.md",
         r"(?m)^(\x60)[^\x60]+(\x60\. See \[GitHub releases\])", sdk, ("yetlet",)),
        ("crates/yetlet-browser/README.md",
         r"(?m)^(Version \x60)[^\x60]+(\x60; see \[GitHub releases\])", sdk,
         ("yetlet", "yetlet-browser")),
    ]
    for relative, pattern, version, packages in specs:
        text = (root / relative).read_text(encoding="utf-8")
        updated, count = re.subn(pattern, lambda m: m.group(1) + version + m.group(2),
                                 text, count=1)
        if count != 1:
            raise ValueError(f"{relative}: current package version heading not found")
        updated = sync_readme_dependencies(updated, packages, version, relative)
        if updated != text:
            updates[relative] = updated

    relative = "tools/yetlet-cli/README.md"
    text = (root / relative).read_text(encoding="utf-8")
    updated, count = re.subn(
        r"(?m)^(\*\*CLI release:\*\* \x60)[^\x60]+(\x60\. \*\*(?:Generated|Build) SDK:\*\* \x60)[^\x60]+(\x60\.)$",
        lambda m: m.group(1) + cli_version + m.group(2) + cli_sdk + m.group(3),
        text, count=1)
    if count != 1:
        raise ValueError(f"{relative}: CLI release and SDK dependency heading not found")
    updated = re.sub(
        r"(cargo(?:\s+\+[^ \t\n]+)?\s+install\s+yetlet-cli\b[^\n]*?\s--version(?:=|\s+))([^\s]+)",
        lambda m: m.group(1) + cli_version, updated)
    updated, count = re.subn(
        r"(\x60yetlet --version\x60 reports exactly\n\x60yetlet )[^\x60\n]+(\x60\.)",
        lambda m: m.group(1) + cli_version + m.group(2),
        updated, count=1)
    if count != 1:
        raise ValueError(f"{relative}: CLI expected version output not found")
    if updated != text:
        updates[relative] = updated
    return updates


def check_reader_manifest(root, edition, relative="examples/docs-site/Cargo.toml"):
    parsed = tomllib.loads((root / relative).read_text(encoding="utf-8"))
    expected = "=" + edition["sdk"]
    found = {package: [] for package in SDK_PACKAGES}
    for _, table in dependency_tables(parsed):
        for package in SDK_PACKAGES:
            if package not in table:
                continue
            dependency = table[package]
            if isinstance(dependency, str):
                version = dependency
            elif isinstance(dependency, dict):
                if "path" in dependency or "git" in dependency:
                    raise ValueError(
                        f"{relative}: {package} must remain a registry dependency selected by site/pages.toml"
                    )
                if dependency.get("package", package) != package:
                    raise ValueError(f"{relative}: unexpected package identity for {package}")
                version = dependency.get("version")
            else:
                raise ValueError(f"{relative}: unsupported registry dependency declaration for {package}")
            if not isinstance(version, str):
                raise ValueError(f"{relative}: missing exact registry version for {package}")
            found[package].append(version)
    for package, versions in found.items():
        if not versions:
            raise ValueError(f"{relative}: missing {package} registry dependency")
        if any(version != expected for version in versions):
            raise ValueError(
                f"{relative}: {package} registry pin does not match site/pages.toml edition "
                f"{expected}; refusing to rewrite registry dependencies"
            )


def replace_once(text, pattern, callback, location, label):
    updated, count = re.subn(pattern, callback, text, count=1)
    if count != 1:
        raise ValueError(f"{location}: expected one {label} field")
    return updated


def sync_published_docs(root, published, cli_sdk, edition, base_changes=None):
    sdk = published["SDK"]
    cli = published["CLI"]
    sdk_short = re.sub(r"^[0-9]+\.[0-9]+\.[0-9]+-", "", sdk)
    cli_sdk_value = cli_sdk_version(cli)
    cli_sdk_short = re.sub(r"^[0-9]+\.[0-9]+\.[0-9]+-", "", cli_sdk_value)
    reader_sdk_short = re.sub(r"^[0-9]+\.[0-9]+\.[0-9]+-", "", edition["sdk"])
    tick = re.escape(chr(96))
    changes = dict(base_changes or {})

    def apply(relative, operations):
        original = changes.get(relative)
        if original is None:
            original = (root / relative).read_text(encoding="utf-8")
        updated = original
        for pattern, callback, label in operations:
            updated = replace_once(updated, pattern, callback, relative, label)
        if updated != original:
            changes[relative] = updated

    def one(pattern, version, label):
        return (pattern, lambda m: m.group(1) + version + m.group(2), label)

    def pair(pattern, first, second, label):
        return (pattern, lambda m: m.group(1) + first + m.group(2) +
                second + m.group(3), label)

    links = publication_links(root)
    for relative in ('README.md', 'docs/ARCHITECTURE.md',
                     'docs/RELEASE-READINESS.md', 'docs/RELEASE-PUBLICATION.md'):
        for subject, version in published.items():
            apply(relative, [one(r'(\*\*Published ' + subject + r':\*\* ' + tick + ')' +
                                 VERSION_PATTERN + '(' + tick + ')', version,
                                 'published ' + subject)])
    for relative in ('README.md', 'docs/COMPATIBILITY.md', 'docs/RELEASE-READINESS.md',
                     'docs/RELEASE-PUBLICATION.md', 'docs/DOCUMENTATION-SCOPE.md'):
        for subject, link in links.items():
            destination = link if relative == 'README.md' else link.removeprefix('docs/')
            apply(relative, [(r'(\[' + subject + r' publication record\]\()[^)]+(\))',
                              lambda m, destination=destination: m[1] + destination + m[2],
                              subject + ' publication record link')])

    apply("docs/API-REFERENCE.md", [
        one(r"(SDK \(" + tick + ")" + VERSION_PATTERN + r"(" + tick +
            r"\) from independent CLI tooling and checkout samples)", sdk, "published SDK"),
        pair(r"(Published SDK)" + VERSION_SHORT_PATTERN + r"(/CLI)" + VERSION_PATTERN + r"(\s+artifacts)",
             sdk_short, cli, "published SDK/CLI Style boundary"),
    ])
    apply("docs/CAPABILITIES.md", [
        one(r"(version " + tick + ")" + VERSION_PATTERN + r"(" + tick +
            r", available from crates\.io)", sdk, "published SDK"),
        pair(r"(Published SDK)" + VERSION_SHORT_PATTERN + r"(/CLI)" + VERSION_PATTERN + r"(\s+artifacts)",
             sdk_short, cli, "published SDK/CLI Style boundary"),
    ])
    apply("docs/COMPATIBILITY.md", [
        one(r"(now \*\*published in SDK " + tick + ")" + VERSION_PATTERN + r"(" + tick + r"\*\*)",
            sdk, "typed-tag published SDK"),
        pair(r"(published SDK )" + VERSION_SHORT_PATTERN + r"( and CLI )" + VERSION_PATTERN + r"( packages)",
             sdk_short, cli, "published SDK/CLI packages"),
        one(r"(The published SDK is " + tick + "yetlet" + tick + " / " +
            tick + "yetlet-browser" + tick + " " + tick + ")" + VERSION_PATTERN +
            r"(" + tick + r";\s*the independently)", sdk, "current published SDK"),
        (r"(published CLI is " + tick + "yetlet-cli" + tick + " " + tick + ")" +
         VERSION_PATTERN + tick + r"(?:, generating apps pinned to SDK|, with release build SDK)" + VERSION_SHORT_PATTERN + r"(\.)",
         lambda m: m.group(1) + cli + tick + ', with release build SDK' + cli_sdk_short + m.group(2),
         "current published CLI and immutable build SDK"),
        pair(r"(SDK)" + VERSION_SHORT_PATTERN + r"(/CLI)" + VERSION_PATTERN + r"( in \[the site manifest\])",
             reader_sdk_short, edition["cli"], "public reader edition"),
        one(r"(Style templates are distributed in CLI )" + VERSION_PATTERN + r"(\.)",
            cli, "published Style CLI"),
    ])
    apply("docs/VIEW-COMPONENTS.md", [
        one(r"(Experimental SDK " + tick + ")" + VERSION_PATTERN +
            r"(" + tick + r", published on crates\.io\.)", sdk, "published SDK"),
        one(r"(available from published SDK\s*" + tick + ")" + VERSION_PATTERN + r"(" + tick + r")",
            sdk, "typed-tag published SDK"),
    ])
    apply("docs/DOCUMENTATION-SCOPE.md", [
        pair(r"(\n" + tick + ")" + VERSION_PATTERN +
             r"(" + tick + r"; the independently published CLI is " + tick + "yetlet-cli" + tick + " " + tick + ")" +
             VERSION_PATTERN + r"(" + tick + r"\.)",
             sdk, cli, "published SDK/CLI scope"),
        pair(r"(public reader/first-app edition is the explicitly selected SDK)" +
             VERSION_SHORT_PATTERN + r"(/CLI)" + VERSION_PATTERN + r"(\s)",
             reader_sdk_short, edition["cli"], "reader edition scope"),
    ])

    for relative in (
        "docs/CONFIGURATION.md", "docs/EXTENSIONS.md", "docs/HTTP-PIPELINE.md",
        "docs/SECRET-PROVIDERS.md", "docs/SETTINGS-SECRET-PROVIDER.md",
    ):
        original = changes.get(relative)
        if original is None:
            original = (root / relative).read_text(encoding="utf-8")
        updated, count = re.subn(
            r"(modules are distributed in SDK\s*\n\s*" + tick + ")" +
            VERSION_PATTERN + r"(" + tick + r")",
            lambda m: m.group(1) + sdk + m.group(2), original, count=1)
        if count != 1:
            raise ValueError(f"{relative}: expected one published SDK integration field")
        if relative == "docs/SETTINGS-SECRET-PROVIDER.md":
            updated, count = re.subn(
                r'(yetlet\s*=\s*\{\s*version\s*=\s*")=?' + VERSION_PATTERN + r'(")',
                lambda m: m.group(1) + "=" + sdk + m.group(2), updated, count=1)
            if count != 1:
                raise ValueError(f"{relative}: expected one exact SDK dependency example")
        if updated != original:
            changes[relative] = updated

    apply("docs/OPT-IN-QUICK-REFERENCE.md", [
        one(r"(published Experimental SDK\s*\n\s*" + tick + ")" +
            VERSION_PATTERN + r"(" + tick + r")", sdk, "published SDK"),
    ])
    apply("docs/TROUBLESHOOTING.md", [
        pair(r"(The SDK " + tick + ")" + VERSION_PATTERN +
             r"(" + tick + r" and independent source-installed CLI " + tick + ")" +
             VERSION_PATTERN + r"(" + tick + r" are published)",
             sdk, cli, "published SDK/CLI"),
    ])
    apply("docs/BROWSER-APP.md", [
        pair(r"(published CLI " + tick + ")" + VERSION_PATTERN +
             r"(" + tick + r" and SDK\s*\n" + tick + ")" +
             VERSION_PATTERN + r"(" + tick + r")",
             edition["cli"], edition["sdk"], "reader CLI/SDK pair"),
    ])
    relative = "docs/BROWSER-APP.md"
    text = changes.get(relative, (root / relative).read_text(encoding="utf-8"))
    updated = sync_reader_commands(text, edition)
    if updated != text:
        changes[relative] = updated
    apply("docs/README.md", [
        (r"(\[SDK " + tick + ")" + VERSION_PATTERN + r"(" + tick + r"\]\()[^)]+(\))",
         lambda m: m[1] + sdk + m[2] + links['SDK'].removeprefix('docs/') + m[3], 'SDK release index'),
        (r"(\[CLI " + tick + ")" + VERSION_PATTERN + r"(" + tick + r"\]\()[^)]+(\))",
         lambda m: m[1] + cli + m[2] + links['CLI'].removeprefix('docs/') + m[3], 'CLI release index'),
        pair(r"(CLI)" + VERSION_PATTERN + r"( → protected standalone project → SDK)" + VERSION_PATTERN + r"( → app lock/cache)",
             edition["cli"], edition["sdk"], "first-app reader edition index"),
    ])

    relative = "README.md"
    text = changes.get(relative)
    if text is None:
        text = (root / relative).read_text(encoding="utf-8")
    start = text.find("## Start with native SSR")
    end = text.find("\n## ", start + 1)
    if start < 0 or end < 0:
        raise ValueError("README.md: native SSR reader section not found")
    section = text[start:end]
    section = re.sub(r"0\.9\.0-alpha\.[0-9]+", edition["sdk"], section)
    section = re.sub(
        r"(CLI " + tick + ")" + VERSION_PATTERN + r"(" + tick + ")",
        lambda m: m.group(1) + edition["cli"] + m.group(2), section)
    section = re.sub(
        r"(--version\s+)" + VERSION_PATTERN + r"(\s+--locked)",
        lambda m: m.group(1) + edition["cli"] + m.group(2), section)
    section = re.sub(
        r"(Expected output: " + tick + "yetlet )" + VERSION_PATTERN + r"(" + tick + ")",
        lambda m: m.group(1) + edition["cli"] + m.group(2), section)
    section = sync_reader_commands(section, edition)
    if section != text[start:end]:
        changes[relative] = text[:start] + section + text[end:]

    relative = "docs/FIRST-APP.md"
    text = changes.get(relative)
    if text is None:
        text = (root / relative).read_text(encoding="utf-8")
    end = text.find("\n## Prerequisites")
    if end < 0:
        raise ValueError(f"{relative}: reader edition introduction not found")
    intro = text[:end]
    intro = re.sub(r"0\.9\.0-alpha\.[0-9]+", edition["sdk"], intro)
    intro = re.sub(
        r"(CLI " + tick + ")" + VERSION_PATTERN + r"(" + tick + ")",
        lambda m: m.group(1) + edition["cli"] + m.group(2), intro)
    remainder = text[end:]
    remainder, count = re.subn(
        r"(independent of SDK " + tick + ")" + VERSION_PATTERN + r"(" + tick + r")",
        lambda m: m.group(1) + edition["sdk"] + m.group(2), remainder, count=1)
    if count != 1:
        raise ValueError(f"{relative}: reader SDK independence field missing")
    remainder, count = re.subn(
        r'(yetlet\s*=\s*")=?' + VERSION_PATTERN + r'("' + tick + r' from crates\.io)',
        lambda m: m.group(1) + "=" + edition["sdk"] + m.group(2), remainder, count=1)
    if count != 1:
        raise ValueError(f"{relative}: generated SDK pin field missing")
    remainder = re.sub(
        r"(cargo(?:\s+\+[^ \t\n]+)?\s+install\s+yetlet-cli\b[^\n]*?\s--version(?:=|\s+))([^\s]+)",
        lambda m: m.group(1) + edition["cli"], remainder)
    updated = intro + remainder
    updated = sync_reader_commands(updated, edition)
    updated = re.sub(r"(SDK\s*" + tick + r")" + VERSION_PATTERN + r"(" + tick + r")",
                     lambda m: m[1] + edition["sdk"] + m[2], updated)
    updated = re.sub(r"(CLI " + tick + r")" + VERSION_PATTERN + r"(" + tick + r")",
                     lambda m: m[1] + edition["cli"] + m[2], updated)
    if updated != text:
        changes[relative] = updated

    apply("examples/docs-site/README.md", [
        one(r"(registry SDK)" + VERSION_PATTERN + r"(,)", edition["sdk"], "docs-site reader SDK"),
    ])
    relative = "examples/docs-site/README.md"
    text = changes.get(relative)
    if text is None:
        text = (root / relative).read_text(encoding="utf-8")
    updated, count = re.subn(
        r"(pinned to SDK)" + VERSION_SHORT_PATTERN + r"(/CLI)" + VERSION_PATTERN,
        lambda m: m.group(1) + reader_sdk_short + m.group(2) + edition["cli"],
        text, count=1)
    if count != 1:
        raise ValueError(f"{relative}: reader SDK/CLI pin missing")
    if updated != text:
        changes[relative] = updated

    apply("examples/hello-world/README.md", [
        one(r"(included in the published SDK " + tick + ")" + VERSION_PATTERN + r"(" + tick + r")",
            sdk, "hello-world published typed tags"),
    ])
    return changes


def sync_reader_commands(text, edition):
    text = re.sub(r"(cargo(?:\s+\+[^ \t\n]+)?\s+install\s+yetlet-cli\b[^\n]*?\s--version(?:=|\s+))" +
                  VERSION_PATTERN, lambda m: m[1] + edition["cli"], text)
    text = re.sub(r"(\byetlet )" + VERSION_PATTERN,
                  lambda m: m[1] + edition["cli"], text)
    return re.sub(r"(--sdk-version\s+)" + VERSION_PATTERN,
                  lambda m: m[1] + edition["sdk"], text)





def plan(root):
    sdk = tomllib.loads((root / "Cargo.toml").read_text())["workspace"]["package"]["version"]
    cli = tomllib.loads((root / "tools/yetlet-cli/Cargo.toml").read_text())
    cli_version = cli["package"]["version"]
    cli_dependency = cli["dependencies"]["yetlet"]
    cli_sdk = cli_dependency if isinstance(cli_dependency, str) else cli_dependency["version"]
    edition = tomllib.loads((root / "site/pages.toml").read_text())["edition"]
    if set(edition) != {"sdk", "cli"}:
        raise ValueError("site/pages.toml: explicit SDK and CLI reader edition required")
    check_reader_manifest(root, edition)
    paths = manifest_paths(root)
    updates, inventory = sync_manifests(root, paths, sdk)
    updates.update(sync_package_readmes(root, sdk, cli_version, cli_sdk))
    local_docs = sync_current_docs(root, sdk)
    updates.update(sync_published_docs(root, published_versions(root), cli_sdk, edition, local_docs))
    for relative in sync_guides(root, sdk):
        text = (root / relative).read_text(encoding="utf-8")
        updates[relative] = sync_guide_text(text, sdk, relative)
    return sdk, cli_version, cli_sdk, edition, paths, inventory, updates


def sync(root):
    sdk, cli, cli_sdk, edition, paths, inventory, updates = plan(root)
    for relative, content in updates.items():
        (root / relative).write_text(content, encoding='utf-8')
    check(root)
    return sorted(updates)


def check(root):
    sdk, cli, cli_sdk, edition, paths, inventory, updates = plan(root)
    if updates:
        raise ValueError('stale current SDK/CLI versions; run sync')
    return sdk, cli, cli_sdk, edition, len(paths), inventory


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('command', choices=('sync', 'check'))
    args = parser.parse_args()
    sdk, cli, cli_sdk, edition, paths, inventory, updates = plan(ROOT)
    if args.command == 'sync':
        for relative, content in updates.items():
            (ROOT / relative).write_text(content, encoding='utf-8')
        print(f'SDK {sdk}; CLI {cli}; reader edition SDK {edition["sdk"]} / CLI {edition["cli"]}')
        print(f'Synchronized {len(updates)} files across {len(paths)} tracked manifests and {len(inventory)} SDK path-dependency manifests')
        for relative in sorted(updates):
            print('  ' + relative)
    else:
        if updates:
            raise SystemExit('stale current versions; run sync')
        print(f'PASS: {len(paths)} tracked manifests, {len(inventory)} SDK path-dependency manifests, package READMEs, CLI README, reader pins, guides and current docs')
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
