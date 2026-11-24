"""Test that the staging release picker chooses the newest prerelease."""
import ast
import unittest
from pathlib import Path

SOURCE = Path(__file__).resolve().parents[2] / "custom_components" / "klar_nlu" / "const.py"
TREE = ast.parse(SOURCE.read_text())
for node in TREE.body:
    if isinstance(node, ast.FunctionDef) and node.name == "pick_staging_release":
        namespace: dict = {}
        exec(compile(ast.Module(body=[node], type_ignores=[]), str(SOURCE), "exec"), namespace)
        pick_staging_release = namespace["pick_staging_release"]
        break
else:
    raise AssertionError("pick_staging_release not found")


class StagingReleaseTests(unittest.TestCase):
    def test_picks_newest_staging_by_created_at_not_api_order(self):
        releases = [
            {"tag_name": "2026.10.2-staging.5d794c0", "prerelease": True, "created_at": "2026-11-21T18:01:49Z"},
            {"tag_name": "2026.10.2-staging.3872503", "prerelease": True, "created_at": "2026-11-21T22:37:42Z"},
            {"tag_name": "2026.10.2-staging.2aed05f", "prerelease": True, "created_at": "2026-11-21T22:15:46Z"},
            {"tag_name": "2026.11.1", "prerelease": False, "created_at": "2026-11-16T18:51:03Z"},
        ]
        chosen = pick_staging_release(releases)
        self.assertIsNotNone(chosen)
        self.assertEqual(chosen["tag_name"], "2026.10.2-staging.3872503")


if __name__ == "__main__":
    unittest.main()
