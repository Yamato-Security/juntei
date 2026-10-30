# Common Output Patterns

## One artifact or a collection

```bash
juntei parse-prefetch -f 'Prefetch/POWERSHELL.EXE-12345678.pf'
juntei parse-prefetch -d evidence/
```

`-d` searches recursively for files appropriate to that command. Source-path columns distinguish
hosts and artifacts in combined output.

## Save CSV

```bash
juntei parse-prefetch -d evidence/ -o prefetch.csv
```

Some parsers need a directory because they emit multiple CSV tables:

```bash
juntei parse-amcache -f Amcache.hve -o amcache-tables
juntei parse-srum -f SRUDB.dat -o srum-tables
juntei parse-jump-lists -d evidence/ -o jump-list-tables
```

## Stream JSONL

```bash
juntei parse-registry services -d evidence/ -t jsonl \
  | jq -c 'select(.image_path? | test("temp|public"; "i"))'
```

Messages and progress go to standard error; standard output remains parseable.

## Overwrite intentionally

Juntei refuses to overwrite existing output by default. Confirm the target, preserve any earlier
result that matters, then add `-C` / `--clobber`:

```bash
juntei parse-prefetch -d evidence/ -o prefetch.csv --clobber
```

## Quiet and no-color modes

Use `-q` for automation and `-K` when terminal color codes are inappropriate:

```bash
juntei parse-mft -f '$MFT' -o mft.csv -q -K
```

## Case and paths

Quote Windows paths containing spaces, `$`, shell metacharacters, or backslashes as needed by your
shell. On Bash-like shells, single quotes prevent `$MFT` and `$J` from becoming variables:

```bash
juntei parse-mft -f '/evidence/C/$MFT' -o mft.csv
```
