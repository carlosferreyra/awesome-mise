"""Generate README.md, or check freshness without rewriting it (Python 3.11+)."""
import argparse
from checks import ROOT, load_catalog


def render():
    categories = load_catalog()
    contents = "\n".join(f"- [{c['name']}](#{c['slug']})" for c in categories)
    sections = []
    for category in categories:
        lines = [f'<a id="{category["slug"]}"></a>', "", f"## {category['name']}"]
        for entry in sorted(category["entries"], key=lambda e: e["name"].casefold()):
            lines += ["", f"### [{entry['name']}]({entry['url']})", "",
                      f"**{entry['type'].capitalize()}** — {entry['description']}", "", entry["notes"]]
            if entry["type"] == "recipe":
                config = (ROOT / entry["config"]).read_text().rstrip()
                lines += ["", f"Copy [`{entry['config']}`]({entry['config']}) as `mise.toml`:",
                          "", "```toml", config, "```", "", "Run from that directory:",
                          "", "```sh", entry["usage"], "```"]
        sections.append("\n".join(lines))
    template = (ROOT / "template/README.md").read_text()
    return template.replace("@@CONTENTS@@", contents).replace("@@CATALOG@@", "\n\n".join(sections))


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    result = render()
    if args.check:
        if (ROOT / "README.md").read_text() != result:
            parser.exit(1, "README.md is stale; run python3 scripts/readme.py\n")
        print("README.md is current.")
    else:
        (ROOT / "README.md").write_text(result)
        print("Generated README.md.")
