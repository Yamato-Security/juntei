# ShimCache (AppCompatCache)

## What ShimCache is

Windows' Application Compatibility subsystem caches executable paths and metadata while evaluating
compatibility/shim behavior. The cache survives files being renamed, removed, or disconnected with
a removable volume. Its layout and available fields differ across Windows XP through Windows 11.

ShimCache is valuable for historical path discovery and, where the format has a reliable execution
flag, additional execution support. The cached time is normally the **file's last-modified time**,
not the time Windows saw or ran it.

<span class="evidence-strength">Best conclusion:</span> Windows observed the path for compatibility
processing. An entry alone is presence/observation evidence; use `Executed` only where supported.

## Location and collection

The bytes are in the `SYSTEM` registry hive, commonly under:

```text
ControlSet00N\Control\Session Manager\AppCompatCache\AppCompatCache
```

Collect `SYSTEM`, `SYSTEM.LOG1`, and `SYSTEM.LOG2`. Juntei reads all control sets and identifies
`Select\Current` where possible.

!!! warning "The newest live entries may not be in the hive"
    Windows can hold ShimCache updates in memory and flush them during shutdown/reboot. A hive copied
    from a running system may not contain the newest observations. The cache's registry last-write
    time helps describe when it was saved, but is not a per-entry execution time.

## Analyst cautions

- A cache entry by itself is **not proof of execution**.
- `Executed` can be `Yes`, `No`, or `NA`; available semantics depend on the format/version.
- `LastModifiedTimeUTC` belongs to the referenced file, not the observation or execution.
- Order is useful but not a universal precise timeline; duplicates occur across control sets/slack.
- `FromSlack` entries were recovered outside the active cache and require additional caution.
- A dirty `SYSTEM` without usable logs may omit current records. Read the recovery summary.
- Findings indicate suspicious naming/path patterns, not malware verdicts.

## Parse with Juntei

```text
Usage: juntei parse-shimcache <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SYSTEM hives
  -f, --file <FILE>      Parse SYSTEM or a raw AppCompatCache value
      --no-logs          Do not replay SYSTEM.LOG1/SYSTEM.LOG2

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-shimcache -f SYSTEM -o shimcache.csv
juntei parse-shimcache -d evidence/ -t jsonl \
  | jq 'select(.executed == true or (.findings | length > 0))'
```

## Output fields

| Field | Interpretation |
|---|---|
| `ControlSet` | Numeric control set holding the cache. Compare `IsCurrent` before choosing one. |
| `CacheEntryPosition` | Position/order within the decoded cache. Useful for relative ordering, not a time. |
| `Path` | Cached executable/package path. Primary hunting pivot. |
| `LastModifiedTimeUTC` | File modification time stored in the entry—not execution/observation time. |
| `Executed` | `Yes`, `No`, or `NA` according to supported format flags. Use only with version context. |
| `Duplicate` | Same entry was already seen in an earlier control set or slack; helps prevent double-counting. |
| `SourceFile` | Input hive/raw value. Essential across hosts. |
| `FromSlack` | Entry recovered from cache-value slack rather than active list. Lower confidence and possibly older. |
| `CacheFormat` | Decoded Windows/cache layout. Governs field availability and semantics. |
| `CacheLastWrittenUTC` | Registry key/cache save time. It is shared context, not each entry's run time. |
| `InsertFlags`, `ShimFlags` | Raw compatibility flags; `Executed` is Juntei's interpreted result where supported. |
| `DataSize` | Size of this entry's data, useful for structural validation. |
| `Machine` | Architecture recorded by newer formats (`x64`, `x86`, `ARM64`). |
| `FileSize`, `LastUpdateTimeUTC` | Windows XP-specific cached metadata where present. |
| `PackageName`, `PackageVersion`, `PackagePublisherId` | Packaged-application identity in supported entries. Publisher ID is context, not signature proof. |
| `Findings` | Juntei anomaly codes such as relocated system binary or suspicious staging path. Leads only. |
| `RegistryPath`, `RegistryValueName` | Exact hive key/value source. Preserve for validation. |
| `CacheValueSize` | Whole registry value size, not the individual entry's `DataSize`. |
| `IsCurrent` | Whether this entry's control set equals `SYSTEM\Select\Current`; empty if not safely known. |
| `SelectCurrent`, `SelectDefault`, `SelectLastKnownGood`, `SelectFailed` | Raw `SYSTEM\Select` control-set numbers for context and recovery analysis. |

JSON/JSONL additionally preserves entry data, tags, packaged-app subrecords, structured findings,
cache problems, and recovery details.

## Investigation strategy

1. Prefer the current control set but retain other sets and slack as historical evidence.
2. Search paths and findings; do not use `LastModifiedTimeUTC` as execution time.
3. Where `Executed == Yes`, corroborate with Prefetch, SRUM, event logs, or other runtime traces.
4. Match paths against `$MFT`/USN to determine creation, deletion, and metadata history.
5. Report whether transaction logs were replayed and whether the live cache may not have been flushed.

## Further Details

- [ys-shimcache-parser](https://github.com/Yamato-Security/ys-shimcache-parser)
- [Eric Zimmerman's AppCompatCacheParser](https://github.com/EricZimmerman/AppCompatCacheParser)
- [Security Ronin shimcache-forensic](https://github.com/SecurityRonin/shimcache-forensic)
- [Mandiant: Leveraging the Application Compatibility Cache in Forensic Investigations](https://www.mandiant.com/resources/blog/leveraging-application-compatibility-cache-forensic-investigations)
- [Maxim Suhanov's Windows registry format specification](https://github.com/msuhanov/regf)
