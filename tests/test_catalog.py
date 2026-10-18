"""Regression checks for malformed contributions and generated output drift."""
import copy
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "scripts"))
from checks import ROOT, validate
from readme import render, render_recipes


class CatalogTests(unittest.TestCase):
    def setUp(self):
        self.data = json.loads((ROOT / "catalog.json").read_text())

    def test_duplicate_names_rejected(self):
        entry = self.data["categories"][0]["entries"][0]
        self.data["categories"][0]["entries"].append(copy.deepcopy(entry))
        with self.assertRaisesRegex(ValueError, "Duplicate entry"):
            validate(self.data)

    def test_invalid_url_rejected(self):
        self.data["categories"][0]["entries"][0]["url"] = "javascript:alert(1)"
        with self.assertRaisesRegex(ValueError, "HTTPS"):
            validate(self.data)

    def test_recipe_requires_config(self):
        recipe = next(e for c in self.data["categories"] for e in c["entries"] if e["type"] == "recipe")
        del recipe["config"]
        with self.assertRaisesRegex(ValueError, "Recipe needs"):
            validate(self.data)

    def test_recipe_path_cannot_escape(self):
        recipe = next(e for c in self.data["categories"] for e in c["entries"] if e["type"] == "recipe")
        recipe["config"] = "../mise.toml"
        with self.assertRaisesRegex(ValueError, "recipe path"):
            validate(self.data)

    def test_recipe_pages_embed_exact_config_and_index_links_them(self):
        rendered = render()
        self.assertNotIn("```", rendered)
        for path, page in render_recipes().items():
            config = path.with_name("mise.toml")
            self.assertIn("```toml\n" + config.read_text().rstrip() + "\n```", page)
            self.assertIn(str(path.relative_to(ROOT)), rendered)

    def test_freshness_check_rejects_stale_output(self):
        import shutil
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            for name in ["scripts", "template", "examples"]:
                shutil.copytree(ROOT / name, root / name)
            shutil.copyfile(ROOT / "catalog.json", root / "catalog.json")
            (root / "README.md").write_text("stale\n")
            result = subprocess.run([sys.executable, str(root / "scripts/readme.py"), "--check"], capture_output=True, text=True)
            self.assertEqual(result.returncode, 1)
            self.assertIn("stale", result.stderr)
            self.assertEqual((root / "README.md").read_text(), "stale\n")


if __name__ == "__main__":
    unittest.main()
