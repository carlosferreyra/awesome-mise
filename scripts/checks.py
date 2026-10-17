"""Validate catalog data and recipe TOML without installing or executing resources."""
import json
from pathlib import Path
import re
import tomllib
from urllib.parse import urlsplit

ROOT = Path(__file__).resolve().parent.parent
KINDS = {"tool", "integration", "recipe", "learning"}


def validate(data, root=ROOT):
    def require(condition, message):
        if not condition:
            raise ValueError(message)

    require(isinstance(data, dict) and set(data) == {"categories"}, "Expected categories")
    categories = data["categories"]
    require(isinstance(categories, list) and categories, "Categories must be a non-empty list")
    slugs, names = set(), set()
    for category in categories:
        require(isinstance(category, dict) and set(category) == {"name", "slug", "entries"}, "Invalid category fields")
        slug = category["slug"]
        require(isinstance(slug, str) and re.fullmatch(r"[a-z0-9]+(?:-[a-z0-9]+)*", slug), "Invalid slug")
        require(slug not in slugs, "Duplicate category slug")
        slugs.add(slug)
        require(isinstance(category["name"], str) and category["name"].strip(), "Missing category name")
        require(isinstance(category["entries"], list) and category["entries"], "Category needs entries")
        for entry in category["entries"]:
            required = {"name", "type", "description", "url", "notes"}
            require(isinstance(entry, dict) and required <= set(entry) <= required | {"config", "usage"}, "Invalid entry fields")
            require(all(isinstance(v, str) and v.strip() for v in entry.values()), "Entry values must be non-empty strings")
            require(entry["type"] in KINDS, "Unknown resource type")
            name = entry["name"].casefold()
            require(name not in names, "Duplicate entry name")
            names.add(name)
            url = urlsplit(entry["url"])
            require(url.scheme == "https" and url.netloc and not any(c.isspace() for c in entry["url"]), "Expected HTTPS source URL")
            if entry["type"] == "recipe":
                require({"config", "usage"} <= set(entry), "Recipe needs config and usage")
                require(re.fullmatch(r"examples/[a-z0-9-]+/mise\.toml", entry["config"]), "Invalid recipe path")
                config = root / entry["config"]
                require(config.resolve().is_relative_to(root.resolve()), "Recipe must stay inside repository")
                tomllib.loads(config.read_text())
            else:
                require(not ({"config", "usage"} & set(entry)), "Only recipes have config and usage")
    return categories


def load_catalog():
    return validate(json.loads((ROOT / "catalog.json").read_text()))


if __name__ == "__main__":
    load_catalog()
    print("Catalog and recipe TOML are valid (no runtime or compatibility claims).")
