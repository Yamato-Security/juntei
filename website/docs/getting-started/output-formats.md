# Output Formats

## Terminal summaries

Without `-o` or a structured `-t`, commands generally show counts, time ranges, damage, and high
value findings. Summaries help you understand scope before creating large output.

## CSV

CSV is the default saved format. It is convenient for spreadsheet tools, SQL import, and timeline
viewers. Some parsers create multiple tables because their artifacts contain different record
types (Amcache, Jump Lists, and SRUM).

!!! warning "Treat evidence columns as text"
    A path or command line can begin with `=`, `+`, `-`, or `@`. Spreadsheet software may interpret
    it as a formula. Import all fields as text or use a forensic-safe CSV viewer. Juntei preserves
    evidence strings rather than altering them.

## JSON

JSON stores the complete result as one document. It preserves arrays, nested structures, nulls,
large integers, and distinctions that a flat CSV cannot always represent.

```bash
juntei parse-lnk -f sample.lnk -o lnk.json -t json
```

## JSON Lines

JSONL writes one independent JSON object per line. It streams well and works naturally with `jq`:

```bash
juntei parse-registry services -d evidence/ -t jsonl \
  | jq -c 'select(.start_type == "automatic") | {host, service_name, image_path}'
```

When structured output goes to standard output, Juntei sends progress and messages to standard
error so a pipe receives data only.

## UTC and precision

Juntei formats parsed times in UTC using RFC 3339 where the format permits it. Some artifacts have
100-nanosecond precision; others store seconds, FAT time, aggregation intervals, or no time at all.
Digits are not accuracy: always understand the source field described on its artifact page.

## Damage and missing values

Empty, `null`, `NA`, `Problems`, and parser-specific state fields are meaningful. They can mean the
artifact did not store a value, the record was damaged, the format does not support it, or the
parser could not resolve it. Do not replace all empty values with zero.
