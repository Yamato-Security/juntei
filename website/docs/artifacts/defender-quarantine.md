# Microsoft Defender Quarantine

## What the quarantine contains

Microsoft Defender quarantine preserves metadata about a detection and encrypted content for the
resources it quarantined. Juntei associates Entries, ResourceData, and Resources records to report
the threat label, quarantine time, original logical and physical paths, file metadata, owner,
download-origin `Zone.Identifier`, hashes, and integrity problems.

This artifact can outlive the event logs that originally recorded detection/remediation. It can
also preserve the exact suspicious bytes for isolated reverse engineering.

<span class="evidence-strength">Best conclusion:</span> Defender created the quarantine record for
the named threat/resource at the recorded time. The product label is not independent proof of malware,
and quarantine alone does not prove prior execution.

## Location and collection

```text
C:\ProgramData\Microsoft\Windows Defender\Quarantine\
  Entries\{GUID}
  ResourceData\<prefix>\<SHA-1>
  Resources\<prefix>\<SHA-1>
```

Collect the entire `Quarantine` directory. An entry without matching content/record files is still
useful metadata but cannot yield the file bytes or computed hashes.

## Safety and analyst cautions

- Extracted content is likely malicious. Use an isolated analysis system and never execute it.
- `ThreatName` is Defender's signature/classification at that time. False positives and later
  reclassification are possible.
- Quarantine time is remediation/quarantine time—not necessarily download, creation, or execution.
- A quarantined file can have been detected before execution or after it ran.
- Original path and Zone.Identifier are valuable but can be absent or attacker-controlled.
- Orphan records can result from retention/cleanup, incomplete collection, or corruption.
- Juntei verifies sizes, CRCs, IDs, and hashes; review `Problems` before trusting recovered bytes.

## Parse with Juntei

```text
Usage: juntei parse-defender-quarantine <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find quarantine folders
  -f, --file <FILE>      Parse one Entries\{GUID} file

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
      --extract <DIR>       Extract dangerous content with a .quarantined extension
  -C, --clobber             Overwrite confirmed output targets
```

```bash
juntei parse-defender-quarantine -d evidence/ -o quarantine.csv
juntei parse-defender-quarantine -d evidence/ -t jsonl \
  | jq '{quarantine_time, threat_name, path, sha256, problems}'

# Only in an isolated malware-analysis environment
juntei parse-defender-quarantine -d evidence/ -o quarantine.csv --extract extracted/
```

For example, `tool.exe` becomes `tool.exe.quarantined`. Auxiliary streams include their type in
the name, such as `tool.exe.Zone.Identifier.quarantined`. Juntei also removes execute permission.
The extension and permissions reduce accidental execution, but the recovered bytes must still be
treated as malware.

## Output fields

| Field | Interpretation |
|---|---|
| `Record` | `resource`, entry without resources, `orphan-content`, or `orphan-record`. Orphans expose incomplete linkage/collection. |
| `QuarantineTime` | Defender quarantine/remediation time in UTC, not execution time. |
| `ThreatName`, `ThreatId` | Defender classification and numeric ID. Treat as vendor verdict/context. |
| `ResourceType` | File, registry key, service, startup item, scheduled task, or other resource category. |
| `Path`, `PhysicalPath` | Original logical/physical location. Primary correlation pivot; preserve exact spelling. |
| `FileSize` | Size recorded by the resource metadata. Compare with `ContentSize` and MFT. |
| `CreationTime`, `LastAccessTime`, `LastWriteTime` | Metadata captured for the quarantined resource; separate from quarantine time and vulnerable to timestomping. |
| `Attributes` | Recorded file attributes. Hidden/system status can be useful context. |
| `Owner` | SID from the recovered security descriptor; ownership is not necessarily executing user. |
| `ZoneId`, `ReferrerUrl`, `HostUrl` | Download-zone and URL values from `Zone.Identifier`. Strong origin leads when present; ADS can be absent/removed. |
| `ContentSize` | Actual recovered primary content size. Compare with recorded `FileSize`. |
| `MD5`, `SHA1`, `SHA256` | Computed from recovered bytes. Use SHA-256 for modern content identification; keep all for interoperability. |
| `Streams` | Recovered stream kinds/names/sizes; can include ADS, security data, or a registry hive. |
| `ResourceId` | Identifier linking resource metadata/content records. |
| `EntryId` | Quarantine entry GUID; also links to Defender history/event evidence. |
| `ScanId` | Defender scan identifier for grouping resources/actions. |
| `QuarantineFolder` | Root source folder/host distinction. |
| `EntryFile`, `ContentFile`, `ResourceRecordFile` | Exact source files used to build the row. Essential for validation. |
| `ExtractedFile` | Primary quarantined file written by `--extract`; empty when no primary extraction occurred. Auxiliary streams are written beside it. |
| `Problems` | Integrity mismatch, missing content, ID disagreement, damage, or orphan context. Do not suppress. |

JSON/JSONL preserves the structured resource, stream list, record entries, and all relationship
details rather than the CSV's joined text.

## Extraction behavior

Juntei uses the original filename with a final `.quarantined` extension when the quarantine entry
records one. It uses the content identifier for orphaned or non-file resources, and adds the
identifier to resolve filename collisions. Windows-invalid filename characters are replaced. Security
and alternate streams are written beside the primary file. Each security stream also gets a readable
text report. Juntei removes execute permission;
existing different content is not overwritten without `-C`. These controls reduce accidents but do
not make the bytes safe.

### Reading the recovered security descriptor

A `.security.quarantined` file contains binary Windows security metadata, so it is not readable in a
text editor. It is a separate backup stream, not another copy of the executable. Juntei automatically
decodes it when extracting, with either `parse-defender-quarantine --extract` or `auto-parse -x`:

```text
tool.exe.quarantined           Original file content; treat as malware
tool.exe.security.quarantined  Original binary security descriptor, preserved unchanged
tool.exe.security.txt          Readable ownership and permission report
```

The text filename follows the actual binary name, including any collision identifier. The generated
text is ordinary UTF-8 and uses the same overwrite protections as other extracted files. It includes
the binary filename, byte count and SHA-256 so the report can be matched to its source.

| Report field | How to interpret it |
|---|---|
| Owner and primary group | SIDs recorded in the descriptor. Well-known SIDs get names such as `BUILTIN\Administrators` or `NT AUTHORITY\SYSTEM`. Other SIDs remain unresolved; a familiar RID alone is insufficient to name a local/domain account. Ownership does not prove who created or ran a file. |
| Control flags | Original numeric flags and their names, including ACL presence, inheritance and protection. |
| DACL | Ordered access-control entries (ACEs): allow/deny type, account SID, permission mask, rights and inheritance flags. These are recorded entries, not a calculation of effective access for a user's token. |
| SACL | Recorded audit or mandatory-integrity entries. Audit-success/failure flags describe configuration, not a history of access. An absent SACL does not establish that all auditing was disabled. |
| Object GUIDs | Optional GUIDs for object-specific ACEs, with object-specific rights retained numerically rather than mislabeled as file rights. |
| SDDL | Security Descriptor Definition Language representation of the owner, group and supported ACL entries. Consult the control flags separately. Juntei does not emit a partial SDDL string if any component is damaged or unsupported. |
| Decode status and problems | `complete` or `incomplete`, with reasons. Unsupported ACEs retain their raw hex in the report. The original binary is always retained when extraction succeeds. Decode problems also appear in the quarantine record's `Problems` field. |

An absent DACL, a NULL DACL and an empty DACL are reported separately. An absent or NULL DACL provides
no discretionary access restriction; an empty DACL has no ACEs granting access. Other Windows access
checks and privileges still matter. Inherited and inherit-only ACEs remain explicitly flagged.

For example, an entry granting inherited full control to Administrators can appear as:

```text
Entry 1: ALLOW (type 0x00)
  Flags: 0x10 (INHERITED_ACE)
  Account: S-1-5-32-544 (BUILTIN\Administrators)
  Access mask: 0x001F01FF
  Rights: Full control (FILE_ALL_ACCESS)
```

The decoder runs offline on Windows, macOS and Linux without Windows APIs, Python or account lookup.
It decodes basic allow/deny/audit/alarm ACEs, object ACEs and mandatory-integrity labels. Conditional,
callback, resource-attribute and other unsupported ACEs are identified explicitly and preserved in
raw form; their access conditions are not evaluated. A damaged descriptor does not stop extraction
of the other streams.

## Investigation strategy

1. Start from `ThreatName`, `Path`, `QuarantineTime`, `EntryId`, and hashes.
2. Validate `Problems` and compare recorded size with recovered content.
3. Pivot SHA-256 into other hosts, EDR, sandbox, and threat-intelligence data.
4. Correlate original path/time with `$MFT`, USN Journal, Prefetch, Amcache, and event logs.
5. Use URL/zone fields to investigate initial access, but validate with browser/proxy/DNS evidence.

## Further Details

- [ys-defender-history-parser](https://github.com/Yamato-Security/ys-defender-history-parser)
- [Sootmark defender](https://github.com/Sootmark/defender)
- [dissect.target Defender plugin](https://docs.dissect.tools/en/latest/projects/dissect.target/plugins/os/windows/defender.html)
- [Microsoft: Microsoft Defender Antivirus remediation and quarantine](https://learn.microsoft.com/en-us/defender-endpoint/restore-quarantined-files-microsoft-defender-antivirus)
- [defender-dump](https://github.com/knez/defender-dump)
- [Microsoft: Security descriptors](https://learn.microsoft.com/en-us/windows/win32/secauthz/security-descriptors)
- [Microsoft: Security Descriptor String Format](https://learn.microsoft.com/en-us/windows/win32/secauthz/security-descriptor-string-format)
- [Microsoft: ACE strings](https://learn.microsoft.com/en-us/windows/win32/secauthz/ace-strings)
