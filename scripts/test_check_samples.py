#!/usr/bin/env python3
"""Focused negative cases for the bounded sample catalog checker."""
from __future__ import annotations

from contextlib import redirect_stdout
from copy import deepcopy
from io import StringIO
from pathlib import Path
import sys
import unittest
from unittest.mock import patch

ROOT = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(ROOT / "scripts"))
import check_samples  # noqa: E402

ORIGINAL_LOAD_TOML = check_samples.load_toml


class SampleCatalogChecks(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.catalog = check_samples.load_toml(check_samples.CATALOG)

    def validate(self, data: dict) -> list[str]:
        def load(path: Path) -> dict:
            if path == check_samples.CATALOG:
                return deepcopy(data)
            return ORIGINAL_LOAD_TOML(path)

        with patch.object(check_samples, "load_toml", side_effect=load):
            with redirect_stdout(StringIO()):
                return check_samples.check()

    def mutated(self, field: str, value: object) -> dict:
        data = deepcopy(self.catalog)
        data["sample"][0][field] = value
        return data

    def test_current_catalog_passes(self) -> None:
        self.assertEqual(self.validate(self.catalog), [])

    def test_false_command_route_browser_and_dependency_claims_fail(self) -> None:
        for field, value in (
            ("command", "cargo"),
            (
                "routes",
                [
                    {
                        "path": "/ap",
                        "source": "examples/todo-server/src/lib.rs",
                        "markers": ['.get("/", root)'],
                    }
                ],
            ),
            ("browser_runtime", False),
            (
                "routes",
                [
                    {
                        "path": "/todos",
                        "source": "examples/todo-server/src/lib.rs",
                        "markers": ['.group("/api"', '"/todos"'],
                    }
                ],
            ),
            ("kind", "api"),
            ("optional_dependencies", ["yetlet-does-not-exist"]),
        ):
            with self.subTest(field=field):
                self.assertTrue(self.validate(self.mutated(field, value)))

    def test_repository_path_escape_fails(self) -> None:
        for field, value in (
            ("evidence", "docs/../../../../../etc/passwd"),
            ("path", "examples/../../../../../tmp"),
        ):
            with self.subTest(field=field):
                failures = self.validate(self.mutated(field, value))
                self.assertTrue(
                    any("traversal" in failure or "escapes" in failure for failure in failures)
                )

    def test_schema_and_malformed_types_fail_without_crashing(self) -> None:
        data = deepcopy(self.catalog)
        data["schema_version"] = True
        self.assertTrue(self.validate(data))
        for field, value in (
            ("command", 123),
            ("kind", []),
            ("package", []),
            ("package", {}),
        ):
            with self.subTest(field=field):
                self.assertTrue(self.validate(self.mutated(field, value)))


if __name__ == "__main__":
    unittest.main()
