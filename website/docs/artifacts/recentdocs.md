# RecentDocs

## What RecentDocs is

Windows Explorer stores most-recently-used lists of files and folders in each user's `NTUSER.DAT`.
The root list covers recent items across types; subkeys group extensions such as `.docx`, plus a
`Folder` list. `MRUListEx` specifies the order of numbered values.

RecentDocs is useful for identifying documents a user interacted with, including files that were
later deleted or moved. Values can also contain the shortcut name Windows used in the user's Recent
folder, which can be correlated with LNK evidence.

<span class="evidence-strength">Best conclusion:</span> Explorer recorded the item in that user's
RecentDocs list. The key time dates only the most recent item in that specific list.

## Location

```text
C:\Users\<user>\NTUSER.DAT
Software\Microsoft\Windows\CurrentVersion\Explorer\RecentDocs
```

Collect `NTUSER.DAT` and its transaction logs for every relevant profile, plus the user's Recent
LNK directory.

## Analyst cautions

- `MRUListEx` gives order, not a timestamp for every item.
- The key last-write time corresponds to the most recent update; only MRU position 0 gets a
  defensible `LastOpenedUTC` derived from it.
- An item can appear in the root list and an extension list. Do not double-count as two opens.
- Application and Explorer behavior can vary; some access methods may not update RecentDocs.
- Cleanup/privacy tools and list size limits remove older evidence.
- A filename can be attacker-controlled and is not proof of content type.
- User inference comes from profile paths or shell-folder data; review empty/ambiguous usernames.

## Parse with Juntei

```text
Usage: juntei parse-registry recentdocs <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find NTUSER.DAT hives
  -f, --file <FILE>      Parse one NTUSER.DAT
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry recentdocs -f NTUSER.DAT -o recentdocs.csv
juntei parse-registry recentdocs -d evidence/ -t jsonl \
  | jq 'select(.extension == ".docx" or .extension == ".pdf")'
```

## Output fields

| Field | Interpretation |
|---|---|
| `Host` | Host derived from collection/system context where available. |
| `Username` | Profile owner inferred from path or shell-folder paths; preserve uncertainty when empty. |
| `SourcePath` | Input `NTUSER.DAT`. Essential when combining users. |
| `Extension` | Sublist (`.docx`, `Folder`, etc.); null for root RecentDocs list. |
| `MruPosition` | Zero is most recent within this list. This is relative order, not elapsed time. |
| `ValueIndex` | Numbered registry value referenced by `MRUListEx`. Structural pivot only. |
| `Filename` | Decoded item name. Correlate full path through LNK, Jump List, MFT, or application data. |
| `LinkName` | Shortcut name encoded in the shell item, often found in the user's Recent directory. |
| `KeyLastWrittenUTC` | Time the list key last changed. Applies to the list, not every row. |
| `LastOpenedUTC` | Populated only for MRU position 0, where key time supports that interpretation. |
| `ValueDataRaw` | Raw bytes when decoding fails or evidence needs validation. |
| `TransactionLogStatus`, `LogsApplied` | Whether registry logs were replayed and which ones. |
| `Problems` | Missing MRU values, unlisted values, wrong types, or decoding damage. |

## Investigation strategy

1. Separate each user's root and extension lists; deduplicate same item carefully.
2. Use MRU position for ordering and only `LastOpenedUTC` for a timestamped claim.
3. Match `LinkName`/`Filename` to Recent LNK, Jump List, `$MFT`, and USN data.
4. Review decoy names, double extensions, network/removable targets, and documents near initial access.
5. Correlate with application Prefetch/SRUM and event logs when execution matters.

## Further Details

- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
- [libyal libfwsi](https://github.com/libyal/libfwsi)
- [Eric Zimmerman's RECmd and Registry Explorer](https://github.com/EricZimmerman/RECmd)
- [SANS Windows Forensic Analysis poster](https://www.sans.org/posters/windows-forensic-analysis/)
