# Contributing to Awesome Mise

Suggest a useful resource through an issue or a pull request. Prefer a small addition with a
clear use case over a long list of registry entries. Tools, integrations, reusable recipes,
learning resources, and real-world configurations all belong here when they help readers do
something with mise. Use an existing outcome-based category where possible.

## What earns a place

Every entry should explain what makes it worth discovering and why mise is useful in that
context. Installation through mise alone is insufficient. Prefer practical depth and original
sources over a registry mirror or a collection of introductory documentation links.

- Tools: describe a distinctive capability and a realistic use in a mise workflow.
- Recipes: solve a concrete problem and produce an observable result. Include input requirements,
  commands, output locations, overwrite behavior, and expected results in the entry notes.
- Integrations: explain the connection to mise and link the upstream project and compatibility information.
- Real-world configurations: use `learning` for linked case studies. Point to specific files and
  patterns; explain project dependencies and side effects. Using mise alone does not merit inclusion.
- Guides: select for depth, clarity, or a particularly useful application. Basic official setup
  links belong in Start here rather than as standalone catalog entries.

Organize entries by their main use: tools worth discovering, project environments, tasks and
automation, integrations, real-world configurations, or guides and learning. Machine setup and
dotfiles are also in scope; introduce that category when a worthwhile verified entry is ready.
Publish categories with useful entries, and avoid quotas or empty placeholders. Resource type
and category serve different purposes: a task workflow may feature a tool, recipe, or guide.

## Catalog format

`catalog.json` is the canonical resource index. Each category has a `name`, a stable lowercase
hyphenated `slug`, and an `entries` array. Each entry has these non-empty string fields:

| Field | Meaning |
| :--- | :--- |
| `name` | Unique display name across the catalog |
| `type` | `tool`, `integration`, `recipe`, or `learning` |
| `description` | A short practical use case |
| `url` | HTTPS primary source: official docs, upstream repository, or original configuration |
| `notes` | Prerequisites, limitations, and relevant context |

Recipes additionally require `config` (an `examples/<slug>/mise.toml` path) and `usage`
(the command to run from the directory containing the copied configuration). Other types do
not use these fields. Keep complete recipe TOML in the referenced file; the generator embeds
it in the README, so the copyable version and downloadable version stay identical.

For tools, explain the registry key or backend and why the tool is useful with mise. Do not
infer support on every platform from registry membership. For integrations, explain how they
connect to mise rather than presenting them as installable tools. For real-world configurations,
link the original, explain the useful pattern, and respect the source license before copying.
Do not copy secrets, personal paths, or unexplained destructive commands.

## Evidence and verification

Check syntax against current primary mise documentation. In the PR, include the source URL,
the date checked, and any actual commands run, mise version, OS, results, and limitations.
Clearly separate documentation review, TOML parsing, runtime execution, and tool installation.
Only claim compatibility you have evidence for. A source link alone is not an execution test.
For recipes, run the documented command in an isolated project when practical. Do not modify
global configuration or bulk-install tools to validate an addition.

## Local checks

Use Python 3.11 or newer; the scripts have no third-party dependencies. From the repository root:

```sh
python3 scripts/checks.py
python3 -m unittest discover -s tests -v
python3 scripts/readme.py
python3 scripts/readme.py --check
git diff --check
```

Edit `template/README.md` for introductory prose and `catalog.json` or `examples/` for resources.
Include the regenerated README in your PR. Preserve existing category slugs so links stay stable.
CI validates structure, parses recipe TOML, runs maintenance tests, and checks generated output.
It does not execute contributed recipes, install catalog tools, verify remote link availability,
or commit changes. Review source links and runtime evidence separately.

Keep maintenance changes focused and describe their validation in the default PR template.
Follow the [code of conduct](CODE_OF_CONDUCT.md). Contributions are licensed under [MIT](LICENSE).
