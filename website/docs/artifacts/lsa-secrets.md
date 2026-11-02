# LSA Secrets

## What they are

The Local Security Authority (LSA) keeps secrets in the `SECURITY` hive. These include the
autologon password, the passwords of accounts services run as, the computer account's domain
password, the DPAPI system keys, and the key that cached domain logons are encrypted with. It
also keeps cached domain logons, so that domain users can sign in without a domain controller.

`parse-registry lsa-secrets` lists which secrets a system holds and when they were set, and how
many cached logons it keeps and uses. **It reads names and times only: nothing is decrypted, and
no secret's bytes are read or written.** Juntei reads evidence; it doesn't extract credentials.

<span class="evidence-strength">Best conclusion:</span> the system held a secret of this name, set
at this time. A service's secret (`_SC_<service>`) means LSA stores the password of the account
that service runs as.

## Location

```text
C:\Windows\System32\config\SECURITY
Policy\Secrets\<name>\CurrVal    the current value (encrypted)
Policy\Secrets\<name>\OldVal     the previous value
Policy\Secrets\<name>\CupdTime   when the current value was set (FILETIME)
Policy\Secrets\<name>\OupdTime   when the previous value was set
Cache\NL$1 … NL$n                cached domain logons, one slot each
Cache\NL$Control                 how many slots Windows keeps
```

| Name | Kind | What it is |
|---|---|---|
| `DefaultPassword` | `autologon` | The password Winlogon signs in with automatically. |
| `_SC_<service>` | `service_account` | The password of the account `<service>` runs as. |
| `$MACHINE.ACC` | `machine_account` | The computer account's password in its domain. |
| `DPAPI_SYSTEM` | `dpapi_system` | The system's DPAPI keys. |
| `NL$KM` | `cached_logon_key` | The key the cached logons are encrypted with. |
| `L$_SQSA_<SID>` | `security_questions` | A local account's security questions (Windows 10 and later). |
| `RasDialParams!<SID>#n`, `L$_Ras…` | `ras_credentials` | Dial-up and VPN credentials. |
| `L$…`, `G$…`, `M$…` | `local`, `global`, `machine` | Other private data, by the prefix `LsaStorePrivateData` gives it. |

Collect `SECURITY` with its transaction logs, and the system's `SYSTEM` hive, which names the
host.

## Analyst cautions

- **Names and times, not values.** Whether a value holds data comes from its size. Its content
  isn't read.
- **`DefaultPassword` doesn't mean autologon.** The secret exists on most systems, including
  ones that never sign in automatically. `SOFTWARE`'s
  `Microsoft\Windows NT\CurrentVersion\Winlogon` (`AutoAdminLogon`, `DefaultUserName`) says
  whether autologon is configured. A recent `CurrentSetUTC` is a change worth explaining.
- **Service secrets:** `_SC_<service>` appears when a service is set to run as an account with
  a password. Compare it with `parse-registry services` (`ObjectName`), and look for services
  that were installed recently or are unknown. The secret can stay after the service is gone.
- **Times:** `CurrentSetUTC` and `OldSetUTC` are when Windows set the current and previous
  values. The `$MACHINE.ACC` password changes every 30 days by default, so its times move.
  `KeyLastWrittenUTC` and the `CurrVal`/`OldVal` key times are about when the secret last
  changed.
- **Cached logons:** a slot is in use when its record names a user. The record's header is not
  encrypted; only its user name length is read. `CachedLogonsKept` is `NL$Control`'s count,
  which follows `CachedLogonsCount` (10 by default). Slots in use mean domain users signed in
  interactively, and their credentials can be cracked offline from the hive. The users' names
  are encrypted and aren't decoded. Only validated binary records with a complete header and consistent user-name length count as in use. If `Problems` reports damaged slots, the in-use count covers validated slots only; do not interpret the damaged slots as empty.

## Parse with Juntei

```text
Usage: juntei parse-registry lsa-secrets <INPUT> [OPTIONS]

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
juntei parse-registry lsa-secrets -d collection/ -o lsa-secrets.csv
juntei parse-registry lsa-secrets -f SECURITY -t jsonl | jq 'select(.Kind == "service_account")'
```

For each hive, the summary lists each secret with its kind and when it was set. It then names
the services whose account passwords LSA stores, and counts the cached logon slots in use.

## Output fields

| Field | Interpretation |
|---|---|
| `Host`, `SourcePath` | Host (from the system's `SYSTEM` hive), and the `SECURITY` hive. |
| `Secret`, `Kind` | The secret's name, and what it is (see the table above); `Cache` and `cached_logons` for the cached logons' record. |
| `ServiceName`, `UserSID` | The service an `_SC_` secret is for, and the user's SID a name gives (`L$_SQSA_`, `RasDialParams`). |
| `CurrentSetUTC`, `OldSetUTC` | `CupdTime` and `OupdTime`: when the current and previous values were set. |
| `HasCurrentValue`, `HasOldValue` | Whether `CurrVal` and `OldVal` hold data. |
| `CurrentValueLastWrittenUTC`, `OldValueLastWrittenUTC`, `KeyLastWrittenUTC` | The `CurrVal`, `OldVal` and secret keys' last writes. |
| `CachedLogons`, `CachedLogonsInUse`, `CachedLogonsKept` | The cached logons' record: slots, slots in use, and `NL$Control`'s count. |
| `TransactionLogStatus`, `LogsApplied` | Registry recovery context. |
| `Problems` | Malformed or out-of-range times (only time metadata is shown), damaged cache headers, wrong cache value types, and keys that can't be read. |

## Investigation strategy

1. Look for `_SC_` secrets of unknown or recently installed services, and match them with
   `parse-registry services` and event 7045 (a service was installed).
2. Check `DefaultPassword`'s times against `Winlogon`'s autologon settings.
3. On domain members, note the cached logon slots in use, and when `Cache` was last written.
4. Compare the secrets' times with the timeline: a secret set during an intrusion window can
   mark a new service account, autologon or machine account change.

## Further Details

- [Microsoft: Private data object](https://learn.microsoft.com/en-us/windows/win32/secmgmt/private-data-object)
- [Microsoft: Cached domain logon information](https://learn.microsoft.com/en-us/troubleshoot/windows-server/user-profiles-and-logon/cached-domain-logon-information)
- [libyal winreg-kb: Cached credentials](https://github.com/libyal/winreg-kb/blob/main/docs/sources/system-keys/Cached-credentials.md)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
