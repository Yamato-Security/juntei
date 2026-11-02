# Audit Policy

## What it is

The audit policy says which security events Windows logs to the Security event log. It is set
per subcategory on Windows Vista and later (59 on Windows 10 and 11), and per category on
Windows XP and Server 2003. Each one logs successes, failures, both, or nothing.

LSA keeps the policy in force in the `SECURITY` hive, whether it came from `auditpol`, local
security policy or Group Policy.

<span class="evidence-strength">Best conclusion:</span> the system was set to log these events (or
not) when the hive was last written. When a subcategory isn't audited, a missing event says
nothing about whether the activity happened.

## Location

```text
C:\Windows\System32\config\SECURITY
Policy\PolAdtEv          (default value)
```

Windows Vista and later keep a 12-byte header, then one setting per subcategory, category by
category, then how many subcategories each category has. Versions differ only in those counts:
each adds subcategories at the end of a category (56 on the Windows 8.x and Server 2012 R2 hives
read, 59 on Windows 10, 11 and Server 2022). Windows XP and Server 2003 keep whether auditing is
on (one byte, then padding Windows doesn't clear) and nine category settings. That layout was
checked on the public Windows XP image of NIST's CFReDS Hacking Case, which audits nothing; the
values of its category settings follow Microsoft's documentation.

Collect `SECURITY` with its transaction logs, and the system's `SYSTEM` hive, which names the
host.

## Analyst cautions

- **The order is Windows' own.** Within a category, the settings are in the order of the
  subcategories' message IDs, not their GUIDs. Special Logon (`%%12548`, Logon/Logoff's index
  4) comes before IPsec Quick Mode. Tools that read them in GUID order report Windows 10's
  default as Special Logon "No Auditing" and IPsec Quick Mode "Success". `SubcategoryId` is the
  same ID event 4719 (System audit policy was changed) gives, so the two can be matched.
- **The policy in force, not its source.** Group Policy and local policy write the same value.
  Group Policy's `audit.csv` (in SYSVOL and `C:\Windows\System32\GroupPolicy`) shows what was
  meant to be applied (`Machine\Microsoft\Windows NT\Audit\audit.csv` of a policy).
- **Changes over time:** `KeyLastWrittenUTC` is about when the policy last changed. Event 4719
  logs each change (as long as the Security log keeps it). An attacker can turn auditing off,
  and then turn it back on.
- **Subcategories versus categories.** On Windows Vista and later, the subcategories are what
  Windows uses (unless the `SCENoApplyLegacyAuditPolicy` security option is off and a legacy
  category policy overrides them).
- **Windows XP and 2003:** with auditing off (`AuditingEnabled` False), no category is audited,
  whatever its setting.
- **Events:** `Events` lists the main events of each subcategory, from Microsoft's documentation
  of the advanced audit policy settings. These are what an investigation usually looks for, not
  every event the subcategory logs.

## Parse with Juntei

```text
Usage: juntei parse-registry auditpol <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SECURITY hives
  -f, --file <FILE>      Parse one SECURITY hive
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry auditpol -d collection/ -o auditpol.csv
juntei parse-registry auditpol -f SECURITY -t jsonl | jq -r 'select(.setting == "none") | .subcategory'
```

For each hive, the summary gives its layout, how many settings are audited and when the policy
was last written. It lists what is audited for success and failure, success only and failure
only, then the subcategories that aren't audited, with their main events: those events are
missing whatever happened.

## Output fields

| Field | Interpretation |
|---|---|
| `Host`, `SourcePath` | Host (from the system's `SYSTEM` hive), and the `SECURITY` hive. |
| `Layout` | `subcategories` (Windows Vista and later) or `categories` (XP and 2003); empty when the value doesn't decode. |
| `Category`, `Subcategory` | As `auditpol` names them. |
| `SubcategoryGuid`, `SubcategoryId` | The subcategory's GUID, and its message ID as event 4719 gives it (`%%12548`). |
| `Setting` | `none`, `success`, `failure` or `success_and_failure`. |
| `Success`, `Failure` | Whether successes and failures are logged. |
| `AuditingEnabled` | Windows XP and 2003: whether auditing is on at all. |
| `Events` | The subcategory's main event IDs. |
| `KeyLastWrittenUTC` | `PolAdtEv`'s last write: about when the policy last changed. |
| `DataRaw` | The value's bytes (hex), when it doesn't decode whole. |
| `TransactionLogStatus`, `LogsApplied` | Registry recovery context. |
| `Problems` | An unknown layout or size, counts that don't add up, and values past 3. |

## Investigation strategy

1. Before concluding that something didn't happen, check whether its subcategory was audited:
   Process Creation (4688), Logon (4624, 4625), Special Logon (4672), Other Object Access Events
   (scheduled tasks, 4698), Security System Extension (service installs, 4697), User Account and
   Security Group Management (4720, 4732), File Share (5140), Kerberos (4768, 4769).
2. Compare the policy with the Security log: events of subcategories that aren't audited now
   mean the policy changed. Look for event 4719 and for a recent `KeyLastWrittenUTC`.
3. Compare with Group Policy's `audit.csv` to tell a domain-wide policy from a local change.
4. Use `parse-registry read --recover-deleted` and the transaction logs for earlier versions of
   `PolAdtEv`.

## Further Details

- [Microsoft: Advanced security audit policy settings](https://learn.microsoft.com/en-us/previous-versions/windows/it-pro/windows-10/security/threat-protection/auditing/advanced-security-audit-policy-settings)
- [Microsoft: Auditing constants (subcategory GUIDs)](https://learn.microsoft.com/en-us/windows/win32/secauthz/auditing-constants)
- [Microsoft: 4719(S) System audit policy was changed](https://learn.microsoft.com/en-us/previous-versions/windows/it-pro/windows-10/security/threat-protection/auditing/event-4719)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
