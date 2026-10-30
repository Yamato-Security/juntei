# First Investigation

This workflow assumes you received an offline triage collection in `evidence/`.

## 1. Preserve the source

Work from a verified copy. Record hashes, collection time, time zone, host, volume, and collection
tool. Juntei reads evidence, but options such as resident-stream or quarantine extraction create
new output files; direct those to a separate analysis directory.

## 2. Inventory what you have

Look for the paths in [Collecting Evidence](collection.md). Missing artifacts are normal. Do not
assume a collection contains transaction logs just because it contains a registry hive.

## 3. Run summaries first

Many commands print a concise summary when no output file is requested:

```bash
juntei parse-prefetch -d evidence/
juntei parse-shimcache -d evidence/
juntei parse-defender-quarantine -d evidence/
```

Read damage, dirty-database, and unapplied-log warnings before trusting records.

## 4. Save structured output

```bash
mkdir -p analysis
juntei parse-prefetch -d evidence/ -T -o analysis/prefetch.csv
juntei parse-mft -d evidence/ -o analysis/mft
juntei parse-usn-journal -d evidence/ -o analysis/usn
juntei parse-registry services -d evidence/ -o analysis/services.jsonl -t jsonl
```

Prefer JSONL for scripting and nested values, and CSV for a spreadsheet or timeline viewer.

## 5. Pivot from a reliable identifier

Start with a file name, complete path, SHA-1/SHA-256, user SID, MFT entry/sequence, task GUID,
service name, or timestamp window. Normalize case only for fields Windows treats case-insensitively.

```bash
juntei parse-prefetch -d evidence/ -t jsonl \
  | jq 'select(.executable_name? | ascii_downcase == "suspicious.exe")'
```

## 6. Correlate

Use the [Cross-Artifact Workflows](../resources/workflows.md). A practical sequence is:

1. Prefetch for execution and recent run times.
2. `$MFT` and USN Journal for creation, paths, renames, and deletion.
3. Amcache and ShimCache for historical presence and metadata.
4. LNK, Jump Lists, RecentDocs, and SRUM for user/application context.
5. Registry services and TaskCache for persistence.
6. Defender quarantine for detection, removal, origin, and hashes.

## 7. Document reproducibility

Record the source path, source hash, Juntei version, exact command, output hash, system time zone,
and any later filtering. Never silently edit generated evidence output.
