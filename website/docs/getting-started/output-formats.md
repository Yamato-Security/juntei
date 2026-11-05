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

## Field names

CSV column headers and JSON/JSONL field names use PascalCase, including Amcache and the
`auto-parse` manifests. Acronyms remain uppercase: `ProcessID`, `TimestampUTC`, `UserSID`,
`SHA256`. A word following an acronym is separated with a hyphen: `IP-Address`, `MFT-Entry`,
`USN-Journal`. With `jq`, quote a hyphenated field: `.["IP-Address"]`.

The default MFT CSV uses shorter, readable headings such as `EntryNum`, `Ext`, `Dir`, `HasADS`,
`Size`, `Created (SI)` and `Created (FN)`. See the [MFT field reference](../artifacts/mft.md)
for its full CSV schema. Timestamp pairs follow MACB order: DataModified, Accessed,
MetadataChanged, Created, with SI before FN. Quote headings containing spaces or punctuation
in CSV tools.

Evidence values keep their original spelling. Source-provided dictionary keys, such as registry
value names under `Details`/`DataRaw` and `Zone.Identifier` names under `ZoneIdentifier`, also
remain unchanged. SRUM's `ValueState` and `Raw` dictionaries use the same labels as their data
columns. A normalization collision is reported rather than silently dropping a field.

This changes previous export schemas. Regenerate existing results and update saved queries
(for example, `.record_type` becomes `.RecordType`, and Amcache `full_path` becomes `FullPath`).
Human-readable text reports and input/configuration formats keep their existing conventions.

## Abbreviated field names

The following labels apply to CSV, JSON and JSONL exports, including automatic parsing and
manifests. Update saved `jq`, `xsv` and spreadsheet filters when regenerating older results.
Only output labels change; evidence values and raw registry-value names keep their original spelling.

| Previous label | Current label |
|---|---|
| `ADS` | `HasADS` |
| `Application` | `App` |
| `ApplicationCompany` | `AppCompany` |
| `ApplicationCompanyRaw` | `AppCompanyRaw` |
| `ApplicationPath` | `AppPath` |
| `Arguments` | `Args` |
| `Attributes` | `Attr` |
| `AttributeType` | `AttrType` |
| `BytesRecvd` | `BytesReceived` |
| `Computer` | `Comp` |
| `ComputerName` | `CompName` |
| `ComputerNameLastWrittenUTC` | `CompNameLastWrittenUTC` |
| `Directories` | `Dir` |
| `FileName` | `Filename` |
| `MicrosecondZeros` | `ZeroU-Secs` |
| `SI-BeforeFN` | `SI<FN` |
| `UserName` | `User` |
| `Username` | `User` |
| `UserNameSource` | `UserSource` |
| `Users` | `User` |

MFT CSV output uses artifact-specific labels: `File` (Filename), `SN` (Sequence Number), `ParentEntryNum` (ParentEntryNumber), `ParentSN` (Parent Sequence Number), `USN` (Update Sequence Number), `LogfileSN` (Logfile Sequence Number), `SID` (Security ID), `Type` (Name Type), and `RefCnt` (Reference Count). These labels also apply to `auto-parse` MFT CSVs; see [MFT output fields](../artifacts/mft.md#csv-output-fields).

The native system-information JSON time is `CompNameLastWritten` (CSV:
`CompNameLastWrittenUTC`). `HasADS` is a Boolean indicator; `Dir` in Prefetch contains the
recorded directory list. `User` retains its artifact-specific type: a string for an account name,
or the list of associated users in USB output. It does not imply an executing identity.
Quote punctuation in queries: `."SI<FN"` and `.["ZeroU-Secs"]`.

## JSON

JSON stores the complete result as one document. It preserves arrays, nested structures, nulls,
large integers, and distinctions that a flat CSV cannot always represent.

```bash
juntei parse-file lnk -f sample.lnk -o lnk.json -t json
```

## JSON Lines

JSONL writes one independent JSON object per line. It streams well and works naturally with `jq`:

```bash
juntei parse-registry services -d evidence/ -t jsonl \
  | jq -c 'select(.StartType == "automatic") | {Host, ServiceName, ImagePath}'
```

When structured output goes to standard output, Juntei sends progress and messages to standard
error so a pipe receives data only.

`auto-parse -j timeline.jsonl` creates an additional JSONL dataset for a complete multi-host
collection while retaining the categorized CSV results. Add `-t json` or `-t jsonl` to change the
individual result format. Each record has `Comp` and `Artifact` fields so its source remains clear
after SIEM ingestion.

## UTC and precision

Juntei formats parsed times in UTC using RFC 3339 where the format permits it. Some artifacts have
100-nanosecond precision; others store seconds, FAT time, aggregation intervals, or no time at all.
Digits are not accuracy: always understand the source field described on its artifact page.

UTC uses the Windows event-log form `2026-10-20T08:46:38.2001668Z`, retaining available
fractional precision. `auto-parse -Z "+09:00"` displays parsed instants with an explicit fixed
offset in all its result formats and its optional `-j` export. The default is UTC, regardless of
the computer running Juntei. Raw values and unzoned wall clocks remain as recorded; see
[output time zones](../commands/auto-parse.md#output-time-zone).

## Damage and missing values

Empty, `null`, `NA`, `Problems`, and parser-specific state fields are meaningful. They can mean the
artifact did not store a value, the record was damaged, the format does not support it, or the
parser could not resolve it. Do not replace all empty values with zero.
