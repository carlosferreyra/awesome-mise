# Trying the recipes

Copy one recipe's `mise.toml` into an empty scratch directory. Run the corresponding
sample commands below from that directory. They create small input files and replace
any previous sample files with the same names. The configurations use an existing
Python 3.11+ executable named `python3`; the CSV recipe also needs a POSIX shell.

## Document inventory

Use `document-inventory/mise.toml`:

```sh
mkdir -p documents
printf 'hello\n' > documents/hello.txt
mise run inventory
cat build/inventory.csv
```

Expected records: the header `path,bytes` followed by `hello.txt,6`.
The inventory includes regular files recursively, with paths relative to documents/.
Symbolic links are excluded. The generated CSV replaces build/inventory.csv.

## CSV category totals

Use `csv-category-totals/mise.toml`:

```sh
printf 'category,amount\nbooks,12\nfood,8\nbooks,3\n' > input.csv
mise run report
cat build/totals.json
```

Expected JSON totals: `{"books": 15, "food": 8}`. Amounts must be integers.
The preparation task creates build/ before the report task writes build/totals.json.
The report replaces an earlier file at that path.

## Notes archive

Use `notes-archive/mise.toml`:

```sh
mkdir -p notes
printf 'hello\n' > notes/hello.txt
mise run archive
python3 -m tarfile -l build/notes.tar.gz
```

Expected archive members include notes/hello.txt containing `hello` and a newline.
The archive replaces build/notes.tar.gz. It is a snapshot, without retention,
encryption, off-site storage, or recovery testing.

## Verification

On October 17, 2026, all three recipes passed `mise tasks validate` and ran with
mise 2026.10.4 and Python 3.15.0 on macOS arm64. Checks inspected the generated CSV
rows, JSON totals, and archived file bytes against the samples above; the inventory
check also confirmed symbolic links were excluded. Runs used temporary mise config,
data, state, and cache directories, with no tool installations or global config edits.
Other platforms and Python versions were not executed in this check.
