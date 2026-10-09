#!/usr/bin/env python3
"""Validate Yetlet's machine-readable user-facing sample catalog."""
from __future__ import annotations

from pathlib import Path
import re
import sys
import tomllib
from urllib.parse import urlsplit

ROOT = Path(__file__).resolve().parent.parent
ROOT_RESOLVED = ROOT.resolve()
CATALOG = ROOT / "examples/samples.toml"
ID_RE = re.compile(r"^[a-z][a-z0-9-]*$")
KINDS = {"api", "full-stack", "server-rendered"}
STATUSES = {"verified"}
BROWSER_DEPENDENCIES = {
    "wasm-bindgen",
    "wasm-bindgen-futures",
    "web-sys",
    "js-sys",
    "yetlet-dom",
    "yetlet-dom-keyed",
    "yetlet-reactive",
}


def fail(message: str, failures: list[str]) -> None:
    failures.append(message)


def load_toml(path: Path) -> dict:
    with path.open("rb") as handle:
        return tomllib.load(handle)


def contained_path(
    value: object,
    label: str,
    failures: list[str],
    *,
    expect: str,
) -> Path | None:
    if not isinstance(value, str) or not value or Path(value).is_absolute():
        fail(f"{label} must be a non-empty repository-relative path", failures)
        return None
    if ".." in Path(value).parts:
        fail(f"{label} must not contain parent traversal: {value!r}", failures)
        return None
    resolved = (ROOT / value).resolve()
    try:
        resolved.relative_to(ROOT_RESOLVED)
    except ValueError:
        fail(f"{label} escapes the repository: {value!r}", failures)
        return None
    if expect == "file" and not resolved.is_file():
        fail(f"missing {label}: {value}", failures)
        return None
    if expect == "directory" and not resolved.is_dir():
        fail(f"missing {label}: {value}", failures)
        return None
    return resolved


def dependency_names(manifest: dict) -> set[str]:
    names: set[str] = set()
    for field in ("dependencies", "build-dependencies"):
        values = manifest.get(field, {})
        if isinstance(values, dict):
            names.update(name for name in values if isinstance(name, str))
    targets = manifest.get("target", {})
    if isinstance(targets, dict):
        for target in targets.values():
            if not isinstance(target, dict):
                continue
            for field in ("dependencies", "build-dependencies"):
                values = target.get(field, {})
                if isinstance(values, dict):
                    names.update(name for name in values if isinstance(name, str))
    return names


def workspace_packages(failures: list[str]) -> tuple[set[str], dict[str, tuple[str, dict]]]:
    workspace = load_toml(ROOT / "Cargo.toml")
    raw_members = workspace.get("workspace", {}).get("members", [])
    if not isinstance(raw_members, list):
        fail("workspace members must be a list", failures)
        return set(), {}
    members = {member for member in raw_members if isinstance(member, str)}
    packages: dict[str, tuple[str, dict]] = {}
    for member in sorted(members):
        manifest_path = ROOT / member / "Cargo.toml"
        if not manifest_path.is_file():
            fail(f"workspace member manifest missing: {member}/Cargo.toml", failures)
            continue
        manifest = load_toml(manifest_path)
        name = manifest.get("package", {}).get("name")
        if not isinstance(name, str) or not name:
            fail(f"workspace member has no package name: {member}", failures)
            continue
        if name in packages:
            fail(f"duplicate workspace package name: {name}", failures)
            continue
        packages[name] = (member, manifest)
    return members, packages


def route_is_documented(readme: str, path: str) -> bool:
    """Match exact inline route tokens or exact paths from documented URLs."""
    documented = set(re.findall(r"`(/[^`\s]*)`", readme))
    for url in re.findall(r"https?://[^\s`)>]+", readme):
        documented.add(urlsplit(url).path)
    return path in documented


def validate_route(
    route: object,
    label: str,
    sample_directory: Path,
    readme: str,
    failures: list[str],
) -> None:
    if not isinstance(route, dict):
        fail(f"{label} route must be an inline table", failures)
        return
    required = {"path", "source", "markers"}
    if set(route) != required:
        fail(f"{label} route fields must be exactly: markers, path, source", failures)
        return
    path = route.get("path")
    if not isinstance(path, str) or not path.startswith("/") or any(
        character.isspace() for character in path
    ):
        fail(f"{label} route path must be an absolute path", failures)
        return
    if not route_is_documented(readme, path):
        fail(f"{label} route is not documented by its sample README: {path}", failures)
    source = contained_path(route.get("source"), f"{label} route source", failures, expect="file")
    if source is None:
        return
    try:
        source.relative_to(sample_directory)
    except ValueError:
        fail(f"{label} route source must be inside the sample directory", failures)
        return
    markers = route.get("markers")
    if not isinstance(markers, list) or not markers or any(
        not isinstance(marker, str) or not marker for marker in markers
    ):
        fail(f"{label} route markers must be a non-empty string list", failures)
        return
    source_text = source.read_text(encoding="utf-8")
    for marker in markers:
        if marker not in source_text:
            fail(f"{label} route marker not found in {source.relative_to(ROOT)}: {marker!r}", failures)


def check() -> list[str]:
    failures: list[str] = []
    if not CATALOG.is_file():
        return ["missing: examples/samples.toml"]

    data = load_toml(CATALOG)
    if type(data.get("schema_version")) is not int or data["schema_version"] != 1:
        fail("samples catalog schema_version must be integer 1", failures)
    samples = data.get("sample")
    if not isinstance(samples, list) or not samples:
        fail("samples catalog must contain at least one [[sample]]", failures)
        return failures

    members, packages = workspace_packages(failures)
    seen_ids: set[str] = set()
    seen_packages: set[str] = set()
    seen_paths: set[str] = set()

    for index, sample in enumerate(samples, 1):
        label = f"sample #{index}"
        if not isinstance(sample, dict):
            fail(f"{label} must be a table", failures)
            continue
        required = {
            "id",
            "title",
            "package",
            "path",
            "kind",
            "status",
            "command",
            "companion_packages",
            "routes",
            "browser_runtime",
            "browser_packages",
            "browser_build",
            "optional_dependencies",
            "evidence",
        }
        missing = sorted(required - sample.keys())
        extra = sorted(sample.keys() - required)
        if missing:
            fail(f"{label} missing fields: {', '.join(missing)}", failures)
        if extra:
            fail(f"{label} unsupported fields: {', '.join(extra)}", failures)
        if missing or extra:
            continue

        sample_id = sample["id"]
        package = sample["package"]
        relative = sample["path"]
        valid_package = isinstance(package, str) and bool(package)
        if not isinstance(sample_id, str) or not ID_RE.fullmatch(sample_id):
            fail(f"{label} has invalid id: {sample_id!r}", failures)
        elif sample_id in seen_ids:
            fail(f"duplicate sample id: {sample_id}", failures)
        else:
            seen_ids.add(sample_id)

        if not valid_package:
            fail(f"{label} has invalid package", failures)
        elif package in seen_packages:
            fail(f"duplicate sample package: {package}", failures)
        else:
            seen_packages.add(package)

        directory = contained_path(relative, f"{label} path", failures, expect="directory")
        if directory is None:
            continue
        normalized_relative = directory.relative_to(ROOT_RESOLVED).as_posix()
        if normalized_relative in seen_paths:
            fail(f"duplicate sample path: {normalized_relative}", failures)
        seen_paths.add(normalized_relative)
        if normalized_relative not in members:
            fail(f"sample path is not a workspace member: {normalized_relative}", failures)

        manifest_path = directory / "Cargo.toml"
        readme_path = directory / "README.md"
        if not manifest_path.is_file():
            fail(f"missing sample manifest: {manifest_path.relative_to(ROOT)}", failures)
            continue
        manifest = load_toml(manifest_path)
        actual_package = manifest.get("package", {}).get("name")
        if actual_package != package:
            fail(
                f"sample package mismatch for {normalized_relative}: catalog={package!r}, manifest={actual_package!r}",
                failures,
            )
        if valid_package and (
            package not in packages or packages[package][0] != normalized_relative
        ):
            fail(f"sample package is not resolved by the workspace: {package!r}", failures)

        if not readme_path.is_file():
            fail(f"missing sample README: {readme_path.relative_to(ROOT)}", failures)
            readme = ""
        else:
            readme = readme_path.read_text(encoding="utf-8")

        kind = sample["kind"]
        status = sample["status"]
        title = sample["title"]
        command = sample["command"]
        if not isinstance(kind, str) or kind not in KINDS:
            fail(f"{label} has unsupported kind: {kind!r}", failures)
        if not isinstance(status, str) or status not in STATUSES:
            fail(f"{label} has unsupported status: {status!r}", failures)
        if not isinstance(title, str) or not title.strip():
            fail(f"{label} has empty title", failures)
        if not isinstance(command, str) or not command.strip():
            fail(f"{label} has empty command", failures)
        else:
            if valid_package and f"cargo run -p {package}" not in command:
                fail(f"{label} command does not launch package {package}", failures)
            if not any(line.strip() == command for line in readme.splitlines()):
                fail(f"sample README does not contain the exact catalog command: {normalized_relative}", failures)

        companions = sample["companion_packages"]
        related_packages: list[str] = [package] if valid_package else []
        if not isinstance(companions, list) or any(
            not isinstance(name, str) or not name for name in companions
        ):
            fail(f"{label} companion_packages must be a string list", failures)
            companions = []
        elif len(companions) != len(set(companions)) or package in companions:
            fail(f"{label} companion_packages must be unique and exclude the primary package", failures)
        for name in companions:
            if name not in packages:
                fail(f"{label} companion package is not in the workspace: {name}", failures)
            else:
                related_packages.append(name)

        routes = sample["routes"]
        if not isinstance(routes, list) or not routes:
            fail(f"{label} routes must be a non-empty list", failures)
        else:
            seen_routes: set[str] = set()
            for route_index, route in enumerate(routes, 1):
                route_path = route.get("path") if isinstance(route, dict) else None
                if isinstance(route_path, str) and route_path in seen_routes:
                    fail(f"{label} has duplicate route: {route_path}", failures)
                elif isinstance(route_path, str):
                    seen_routes.add(route_path)
                validate_route(route, f"{label} route #{route_index}", directory, readme, failures)

        dependency_map = {
            name: dependency_names(packages[name][1])
            for name in related_packages
            if name in packages
        }
        related_dependencies = set().union(*dependency_map.values()) if dependency_map else set()

        browser_runtime = sample["browser_runtime"]
        browser_packages = sample["browser_packages"]
        browser_build = sample["browser_build"]
        if not isinstance(browser_runtime, bool):
            fail(f"{label} browser_runtime must be boolean", failures)
        if not isinstance(browser_packages, list) or any(
            not isinstance(name, str) or not name for name in browser_packages
        ):
            fail(f"{label} browser_packages must be a string list", failures)
            browser_packages = []
        if len(browser_packages) != len(set(browser_packages)):
            fail(f"{label} browser_packages must be unique", failures)
        if kind == "api":
            if browser_runtime is not False:
                fail(f"{label} kind=api requires browser_runtime=false", failures)
            if browser_packages:
                fail(f"{label} kind=api requires empty browser_packages", failures)
            if browser_build != "":
                fail(f"{label} kind=api requires empty browser_build", failures)
            forbidden = sorted(BROWSER_DEPENDENCIES & related_dependencies)
            if forbidden:
                fail(
                    f"{label} kind=api has browser dependencies: {', '.join(forbidden)}",
                    failures,
                )

        if browser_runtime is True:
            if not browser_packages:
                fail(f"{label} browser runtime requires browser_packages", failures)
            for name in browser_packages:
                if name not in companions:
                    fail(f"{label} browser package must be a declared companion: {name}", failures)
                elif "wasm-bindgen" not in dependency_map.get(name, set()):
                    fail(f"{label} browser package lacks wasm-bindgen: {name}", failures)
            build_path = contained_path(browser_build, f"{label} browser build", failures, expect="file")
            if build_path is not None:
                build_text = build_path.read_text(encoding="utf-8")
                for name in browser_packages:
                    if name not in build_text:
                        fail(f"{label} browser build does not reference package: {name}", failures)
        elif browser_runtime is False:
            if browser_packages:
                fail(f"{label} browser_runtime=false requires empty browser_packages", failures)
            if browser_build != "":
                fail(f"{label} browser_runtime=false requires empty browser_build", failures)
            forbidden = sorted(BROWSER_DEPENDENCIES & related_dependencies)
            if forbidden:
                fail(f"{label} API/server sample has browser dependencies: {', '.join(forbidden)}", failures)

        dependencies = sample["optional_dependencies"]
        if not isinstance(dependencies, list) or any(
            not isinstance(name, str) or not name.startswith("yetlet-") for name in dependencies
        ):
            fail(f"{label} optional_dependencies must contain Yetlet package names", failures)
        else:
            if len(dependencies) != len(set(dependencies)):
                fail(f"{label} optional_dependencies must be unique", failures)
            for name in dependencies:
                if name not in packages:
                    fail(f"{label} optional dependency is not a workspace package: {name}", failures)
                if name not in related_dependencies:
                    fail(f"{label} optional dependency is not direct in primary/companion manifests: {name}", failures)

        contained_path(sample["evidence"], f"{label} evidence", failures, expect="file")

    print(f"Samples: {len(samples)}")
    return failures


def main() -> int:
    failures = check()
    if failures:
        print(f"FAIL ({len(failures)}):")
        print("\n".join(f" - {failure}" for failure in failures))
        return 1
    print("PASS: sample catalog, repository-backed routes, dependencies, READMEs and evidence")
    return 0


if __name__ == "__main__":
    sys.exit(main())
