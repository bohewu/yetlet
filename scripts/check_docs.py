#!/usr/bin/env python3
"""Read-only documentation / backlog consistency gate for Yetlet M0.

Usage: python3 scripts/check_docs.py
No dependencies beyond the Python 3 standard library.
"""
from __future__ import annotations

from pathlib import Path
import re
import sys

ROOT = Path(__file__).resolve().parent.parent
REQUIRED = (
    "README.md",
    "AGENTS.md",
    "docs/ARCHITECTURE.md",
    "docs/MVP.md",
    "docs/M2-VERIFICATION.md",
    "docs/ROADMAP.md",
    "docs/BACKLOG.md",
    "docs/adr/0001-separation-of-concerns.md",
    "docs/adr/0002-events-and-actions.md",
    "docs/adr/0003-renderers-and-loading.md",
)
TASK_RE = re.compile(r"^\|\s*(YT-\d{3})\s*\|")
LINK_RE = re.compile(r"\]\(([^)]+)\)")


def check() -> list[str]:
    failures: list[str] = []
    for name in REQUIRED:
        path = ROOT / name
        if not path.is_file():
            failures.append(f"missing: {name}")
            continue
        text = path.read_text(encoding="utf-8")
        if not text.endswith("\n"):
            failures.append(f"missing EOF newline: {name}")
        if sum(line.lstrip().startswith(chr(96) * 3) for line in text.splitlines()) % 2:
            failures.append(f"unbalanced code fences: {name}")
        for number, line in enumerate(text.splitlines(), 1):
            if line.rstrip() != line:
                failures.append(f"trailing whitespace: {name}:{number}")
        for target in LINK_RE.findall(text):
            file_part = target.split("#", 1)[0]
            if not file_part or file_part.startswith(("https://", "http://", "mailto:")):
                continue
            if not (path.parent / file_part).is_file():
                failures.append(f"broken link: {name} -> {target}")

    backlog = ROOT / "docs/BACKLOG.md"
    if not backlog.is_file():
        return failures
    rows: dict[str, list[str]] = {}
    for line in backlog.read_text(encoding="utf-8").splitlines():
        match = TASK_RE.match(line)
        if not match:
            continue
        item_id = match.group(1)
        if item_id in rows:
            failures.append(f"duplicate backlog ID: {item_id}")
        rows[item_id] = [part.strip() for part in line.split("|")]
    if not rows:
        failures.append("no backlog work items")
    edges: dict[str, list[str]] = {}
    for item_id, fields in rows.items():
        if len(fields) < 9:
            failures.append(f"malformed backlog row: {item_id}")
            continue
        deps = re.findall(r"YT-\d{3}", fields[6])
        edges[item_id] = deps
        for dep in deps:
            if dep not in rows:
                failures.append(f"unknown backlog dependency: {item_id} -> {dep}")
            if dep == item_id:
                failures.append(f"self dependency: {item_id}")

    visiting: set[str] = set()
    visited: set[str] = set()

    def walk(item_id: str) -> None:
        if item_id in visiting:
            failures.append(f"dependency cycle involving: {item_id}")
            return
        if item_id in visited:
            return
        visiting.add(item_id)
        for dep in edges.get(item_id, []):
            if dep in rows:
                walk(dep)
        visiting.remove(item_id)
        visited.add(item_id)

    for task in rows:
        walk(task)
    print(f"Documents: {len(REQUIRED)}, backlog items: {len(rows)}")
    return failures


def main() -> int:
    failures = check()
    if failures:
        print(f"FAIL ({len(failures)}):")
        print("\n".join(f" - {error}" for error in failures))
        return 1
    print("PASS: docs, internal links, Markdown structure, whitespace and DAG")
    return 0


if __name__ == "__main__":
    sys.exit(main())
