# Evidence and Interpretation

Digital forensics is not just reading fields. It is deciding what observation the field supports,
how strong that support is, and what could produce the same observation innocently.

## Four useful evidence categories

| Category | Meaning | Examples |
|---|---|---|
| Presence | A path or file was known to Windows | Amcache file record, ShimCache entry, `$MFT` record |
| Execution | Windows recorded a program launch or execution state | Prefetch run time; supported ShimCache execution flag |
| User interaction | A user or application referenced/opened an item | RecentDocs, Jump List, Recent-folder LNK |
| System activity | Windows recorded resource use or configuration | SRUM counters, service configuration, TaskCache action |

An artifact can support more than one category, but never promote it automatically. For example,
an Amcache entry with a suspicious path supports **presence**. A matching Prefetch run time adds
stronger **execution** evidence. A USN create record and `$MFT` path can place the file on disk, and
Defender quarantine can explain why it disappeared.

## Timestamps are statements, not universal truth

Ask what caused each timestamp:

- `$STANDARD_INFORMATION` and `$FILE_NAME` times describe NTFS metadata and can diverge.
- A Prefetch run time describes a recorded launch, but only recent run times remain.
- A ShimCache time is usually the cached file's last-modified time—not execution time.
- A registry key last-write time applies to the key as a whole, not an individual value.
- RecentDocs gives the key's time to the most recent item; older items have order but usually no open time.
- SRUM timestamps often identify an aggregation interval rather than the exact network operation.

Use UTC during analysis, retain the original value when available, and document any conversion.

## Positive and negative evidence

The presence of a well-understood record can be strong positive evidence. Absence is usually
weaker because artifacts roll over, caches are capped, collection may be incomplete, settings vary,
and cleanup tools exist. Say “not observed in the collected artifact” instead of “did not happen.”

## Corroboration ladder

1. Validate the source file and collection path.
2. Confirm the parser reported no relevant damage or unapplied transaction logs.
3. Interpret the field according to the artifact page.
4. Find an independent artifact supporting the same action.
5. Check event logs, network telemetry, memory, or file content when available.
6. Record the exact Juntei version, command, filters, and source hash.

!!! tip "Write conclusions at the evidence's strength"
    Prefer “Prefetch records `tool.exe` running at 10:02:11Z” over “the attacker executed the
    malware at exactly 10:02:11Z.” The second statement adds identity and intent that Prefetch
    alone cannot establish.
