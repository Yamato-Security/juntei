# SAM Local Accounts

## What the SAM contains

The Security Account Manager (`SAM`) hive stores local account records. Juntei decodes account
names, RIDs, profile/comment fields, account-control flags, last-logon/password timestamps, counters,
password hints, Microsoft-account linkage, profile paths, and whether Windows hides the account
from the sign-in UI.

This is useful for finding newly created local administrators, enabled dormant accounts, password
changes, hidden accounts, and lateral-movement staging accounts.

<span class="evidence-strength">Best conclusion:</span> the local SAM represented the account with
the recorded state. It is not a complete authentication history and does not describe domain accounts.

## Location and collection

```text
C:\Windows\System32\config\SAM
C:\Windows\System32\config\SAM.LOG1
C:\Windows\System32\config\SAM.LOG2
```

Collect the hive and logs with sufficient privileges/raw access. Host identity can be enriched from
the corresponding `SYSTEM` hive in a collection.

## Analyst cautions

- This command parses **local** accounts, not Active Directory domain users.
- Juntei intentionally does not extract password hashes.
- RID 500 is the built-in Administrator account even if renamed; name alone is insufficient.
- `LastLogon` and counters are registry state, not a complete per-session audit trail.
- `HiddenFromLogonUI` hides an account from the sign-in screen; it does not disable the account.
- Built-in `DefaultAccount` and `WDAGUtilityAccount` can be legitimately hidden.
- Null account flags from a damaged/missing `F` value must not be interpreted as enabled.
- `NamesKeyLastWrittenUTC` approximates account-name registration context but is not a guaranteed creation time.

## Parse with Juntei

```text
Usage: juntei parse-registry sam <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SAM hives
  -f, --file <FILE>      Parse one SAM hive
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Filtering:
      --rid <RID>  Parse one decimal RID (example: 500)

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry sam -f SAM -o accounts.csv
juntei parse-registry sam -f SAM --rid 500 -t jsonl | jq .
juntei parse-registry sam -d evidence/ -t jsonl \
  | jq 'select(.hidden_from_logon_ui == true or (.account_flags | index("disabled") | not))'
```

## Output fields

| Field | Interpretation |
|---|---|
| `Host`, `SourcePath` | Host context and exact SAM hive. |
| `Rid` | Relative Identifier. Strong account identity pivot; 500 is built-in Administrator. |
| `Username` | Current local account name. Can be renamed; correlate by RID. |
| `FullName`, `Comment`, `UserComment` | Descriptive text; attacker-controlled and useful for masquerading review. |
| `AccountFlags` | Decoded states such as disabled, locked out, password not required, normal account, password never expires. |
| `AccountFlagsValue` | Raw numeric flags for validation and unsupported bits. |
| `CountryCode`, `CodePage` | Account locale/code-page metadata; usually context, not attribution. |
| `LastLogonUTC`, `LastLogonWholeSecondsUTC` | Same last-logon state at full precision and whole-second compatibility. Not a complete history. |
| `PasswordLastSetUTC` | Recorded password-change time. Investigate unexpected recent/old values. |
| `AccountExpirationUTC` | Configured expiry when finite. Null can mean never/not set. |
| `LastFailedLogonUTC` | Last recorded failed local logon in SAM state; corroborate Security events. |
| `FailedLogonCount`, `LogonCount` | Cumulative/state counters with Windows-specific update behavior; use as context, not exact event count. |
| `PasswordHint` | Sensitive user-provided hint; may expose password patterns. Handle appropriately. |
| `InternetUserName` | Linked Microsoft account identity where present. |
| `HiddenFromLogonUI` | UI-hiding state. Review high RIDs/unexpected custom accounts; does not mean disabled. |
| `HomeDirectory`, `HomeDrive`, `ProfilePath`, `LogonScript` | Local account environment/logon configuration; unusual scripts/UNC paths can indicate persistence. |
| `KeyLastWrittenUTC` | RID account-key change time. Key-level, not field-specific. |
| `NamesKeyLastWrittenUTC` | Username mapping-key change time; useful account lifecycle approximation. |
| `LastLogonFiletime`, `PasswordLastSetFiletime`, `AccountExpirationFiletime`, `LastFailedLogonFiletime` | Raw 64-bit FILETIME values retained for validation and special sentinel handling. |
| `TransactionLogStatus`, `LogsApplied` | Registry replay context. |
| `Problems` | Disagreement between F/V/Names, missing entries, malformed offsets, or decoding damage. |

## Investigation strategy

1. Compare RID, name, flags, profile path, and `NamesKeyLastWrittenUTC` against the host baseline.
2. Review enabled RID 1000+ accounts, password-not-required, never-expiring, hidden, or recently changed accounts.
3. Correlate with Security 4720/4722/4724/4732 events, logon events, profile creation, and remote activity.
4. Search registry recovery history if an account was removed before collection.
5. Distinguish legitimate provisioning/default accounts from interactive attacker-created accounts.

## Further Details

- [Microsoft Open Specifications: SAM Remote Protocol](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-samr/)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
- [dissect.target SAM plugin](https://docs.dissect.tools/en/latest/projects/dissect.target/plugins/os/windows/sam.html)
- [Eric Zimmerman's Registry Explorer](https://ericzimmerman.github.io/#!index.md)
