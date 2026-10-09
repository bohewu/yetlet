#!/usr/bin/env python3
"""Measure bounded, reproducible authoring/dependency facts for M8 samples."""
from __future__ import annotations

import argparse
import json
from pathlib import Path
import re
import tomllib

ROOT = Path(__file__).resolve().parent.parent
CATALOG = ROOT / "examples/samples.toml"
WORKSPACE = ROOT / "Cargo.toml"
ASYNC_FN_RE = re.compile(r"(?m)^\s*(?:pub(?:\([^)]*\))?\s+)?async\s+fn\s+")


def load_toml(path: Path) -> dict:
    with path.open("rb") as handle:
        return tomllib.load(handle)


def production_dependencies(manifest: dict) -> set[str]:
    """Return direct non-dev dependencies, including target-specific sections."""
    names = set(manifest.get("dependencies", {}))
    names.update(manifest.get("build-dependencies", {}))
    for target in manifest.get("target", {}).values():
        if isinstance(target, dict):
            names.update(target.get("dependencies", {}))
            names.update(target.get("build-dependencies", {}))
    return names


def source_metrics(package_path: Path) -> dict[str, int]:
    files = sorted(package_path.rglob("*.rs"))
    production_lines = 0
    test_lines = 0
    async_functions = 0
    for source in files:
        lines = source.read_text(encoding="utf-8").splitlines()
        relative_parts = source.relative_to(package_path).parts
        if "tests" in relative_parts:
            production_part: list[str] = []
            test_part = lines
        else:
            boundary = next(
                (index for index, line in enumerate(lines) if line.strip() == "#[cfg(test)]"),
                len(lines),
            )
            production_part = lines[:boundary]
            test_part = lines[boundary:]
        production_lines += sum(bool(line.strip()) for line in production_part)
        test_lines += sum(bool(line.strip()) for line in test_part)
        async_functions += len(ASYNC_FN_RE.findall("\n".join(production_part)))
    return {
        "rust_files": len(files),
        "production_nonblank_lines": production_lines,
        "test_nonblank_lines": test_lines,
        "production_async_functions": async_functions,
    }


def workspace_packages() -> dict[str, dict]:
    workspace = load_toml(WORKSPACE)
    result: dict[str, dict] = {}
    for member in workspace["workspace"]["members"]:
        path = ROOT / member
        manifest = load_toml(path / "Cargo.toml")
        name = manifest["package"]["name"]
        result[name] = {
            "name": name,
            "path": member,
            "manifest": manifest,
            "dependencies": production_dependencies(manifest),
        }
    return result


def application_package_closure(
    entry_packages: list[str], packages: dict[str, dict]
) -> list[str]:
    """Include transitive workspace dependencies owned under examples/."""
    selected = set(entry_packages)
    pending = list(entry_packages)
    while pending:
        current = pending.pop()
        for dependency in packages[current]["dependencies"]:
            candidate = packages.get(dependency)
            if candidate is None or not candidate["path"].startswith("examples/"):
                continue
            if dependency not in selected:
                selected.add(dependency)
                pending.append(dependency)
    return sorted(selected)


def measure() -> dict:
    packages = workspace_packages()
    catalog = load_toml(CATALOG)
    samples = []
    for sample in catalog["sample"]:
        entry_packages = [sample["package"], *sample["companion_packages"]]
        application_packages = application_package_closure(entry_packages, packages)
        package_metrics = {}
        direct_dependencies: set[str] = set()
        totals = {
            "rust_files": 0,
            "production_nonblank_lines": 0,
            "test_nonblank_lines": 0,
            "production_async_functions": 0,
        }
        for name in application_packages:
            package = packages[name]
            metrics = source_metrics(ROOT / package["path"])
            package_metrics[name] = {
                "path": package["path"],
                **metrics,
                "direct_dependencies": sorted(package["dependencies"]),
            }
            direct_dependencies.update(package["dependencies"])
            for key in totals:
                totals[key] += metrics[key]

        workspace_dependencies = sorted(name for name in direct_dependencies if name in packages)
        framework_dependencies = sorted(
            name
            for name in workspace_dependencies
            if packages[name]["path"].startswith("crates/")
        )
        example_dependencies = sorted(
            name
            for name in workspace_dependencies
            if packages[name]["path"].startswith("examples/")
        )
        external_dependencies = sorted(
            name for name in direct_dependencies if name not in packages
        )
        capability_source = {}
        for name in sample["optional_dependencies"]:
            package = packages.get(name)
            if package is not None:
                capability_source[name] = source_metrics(ROOT / package["path"])

        samples.append(
            {
                "id": sample["id"],
                "title": sample["title"],
                "kind": sample["kind"],
                "browser_runtime": sample["browser_runtime"],
                "launch_command": sample["command"],
                "route_count": len(sample["routes"]),
                "entry_packages": entry_packages,
                "application_packages": application_packages,
                "package_metrics": package_metrics,
                **totals,
                "direct_dependency_count": len(direct_dependencies),
                "direct_dependencies": sorted(direct_dependencies),
                "workspace_dependencies": workspace_dependencies,
                "framework_dependencies": framework_dependencies,
                "example_dependencies": example_dependencies,
                "external_dependencies": external_dependencies,
                "optional_capabilities": sample["optional_dependencies"],
                "optional_capability_source": capability_source,
            }
        )

    return {
        "schema_version": 1,
        "methodology": {
            "application_package_scope": (
                "catalog primary/companion packages plus transitive workspace "
                "dependencies under examples/"
            ),
            "source_lines": (
                "non-blank physical Rust lines; in-file source before the first "
                "#[cfg(test)] is production and the remainder is test source"
            ),
            "dependencies": (
                "union of direct non-dev normal/build and target-specific dependencies "
                "across application-owned packages"
            ),
            "routes": "catalog-declared repository-backed routes",
        },
        "samples": samples,
    }


def print_table(report: dict) -> None:
    print(
        "ID | App packages | Rust files | Production lines | Test lines | "
        "Async fns | Routes | Direct deps | Browser"
    )
    print("---|---:|---:|---:|---:|---:|---:|---:|---:")
    for sample in report["samples"]:
        print(
            f"{sample['id']} | {len(sample['application_packages'])} | "
            f"{sample['rust_files']} | {sample['production_nonblank_lines']} | "
            f"{sample['test_nonblank_lines']} | "
            f"{sample['production_async_functions']} | {sample['route_count']} | "
            f"{sample['direct_dependency_count']} | "
            f"{'yes' if sample['browser_runtime'] else 'no'}"
        )


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--out", type=Path)
    arguments = parser.parse_args()
    report = measure()
    print_table(report)
    if arguments.out is not None:
        output = arguments.out
        if not output.is_absolute():
            output = ROOT / output
        output = output.resolve()
        try:
            relative_output = output.relative_to(ROOT.resolve())
        except ValueError:
            parser.error("--out must resolve inside the repository")
        output.parent.mkdir(parents=True, exist_ok=True)
        output.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
        print(f"Wrote {relative_output}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
