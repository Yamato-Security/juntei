# BAM and DAM

## What BAM and DAM are

The Background Activity Moderator (BAM) and the Desktop Activity Moderator (DAM) are Windows
services (Windows 10 1709 and later) that manage what applications may do in the background.
They keep, per user, the last time each program ran, in the `SYSTEM` hive:

- one key per user, named after the user's SID;
- one value per program: its name is the program's path on its device
  (`\Device\HarddiskVolume3\Windows\System32\cmd.exe`) or a packaged app's ID
  (`Microsoft.WindowsCalculator_8wekyb3d8bbwe`);
- the first 8 bytes of the value's data are the time it last ran (FILETIME).

BAM ties a program to a user and a time without the user's own hive, and it lists programs run
from removable and network devices.

<span class="evidence-strength">Best conclusion:</span> a process of this program ran in this
user's session; it last ran at this time. BAM doesn't count runs, and an entry can stay after the
program has been deleted.

## Location

```text
C:\Windows\System32\config\SYSTEM
ControlSet00N\Services\bam\State\UserSettings\<SID>   Windows 10 1809 and later
ControlSet00N\Services\bam\UserSettings\<SID>         Windows 10 1709 to 1803
ControlSet00N\Services\dam\State\UserSettings\<SID>
ControlSet00N\Services\dam\UserSettings\<SID>
```

Each user's key also has `Version` and `SequenceNumber` values; they are not programs and are
left out. A system upgraded from 1709 or 1803 can have both layouts.

Collect `SYSTEM` with its transaction logs, and the system's `SAM` and `SOFTWARE`: they name the
users.

## Analyst cautions

- **One time per program and user.** Only the most recent run is kept. Prefetch, Amcache and
  event logs give earlier runs.
- **Recent activity.** Entries of programs that haven't run for a while are reported to be
  removed when Windows starts, so BAM mostly shows recent activity: a program missing from it
  may still have run long ago.
- **Device paths, not drive letters.** `\Device\HarddiskVolume3` is the third volume Windows
  numbered, not `C:`. `DevicePath` and `Path` split the device from the path on it; match the
  volume with `MountedDevices`, the `$MFT` or other artifacts before writing a drive letter.
- **Not every program.** BAM and DAM track programs in users' sessions; services mostly don't
  appear. `S-1-5-18` entries are typically processes of setup and the sign-in screen
  (`FirstLogonAnim.exe`), and `S-1-5-90-0-<n>` ones the Desktop Window Manager (`DWM-<n>`).
- **Control sets.** By default the control set Windows used last is read. `--all-control-sets`
  reads every one: an older control set can keep an entry the current one no longer has.
- **User names.** A local account's name comes from the `SAM` (the computer's SID and the
  account's RID); a domain account's from its profile folder in `ProfileList`. `UserNameSource`
  says which (`sam`, `profile_list`, `well_known`). Without them, only the SID is given. A RID
  the `SAM` no longer has (a deleted account) stays unnamed.

## Parse with Juntei

```text
Usage: juntei parse-registry bam <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SYSTEM hives
  -f, --file <FILE>      Parse one SYSTEM hive
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Filtering:
      --all-control-sets  Every control set, not only the one in use

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry bam -d evidence/ -o bam.csv
juntei parse-registry bam -f SYSTEM -t jsonl \
  | jq 'select(.path? // "" | test("\\\\(Users|ProgramData|Temp)\\\\"; "i"))'
```

The summary lists each user with its SID, name and number of entries, and the programs run last
(newest first).

## Output fields

| Field | Interpretation |
|---|---|
| `Host`, `SourcePath` | Host, and the `SYSTEM` hive. |
| `ControlSet`, `IsCurrentControlSet` | The control set the entry is in, and whether it is the one Windows used last. |
| `Moderator` | `bam` or `dam`. |
| `UserSid`, `UserName`, `UserNameSource` | The SID the key is named after, the user's name, and where it comes from (`sam`, `profile_list`, `well_known`). |
| `Program` | The value's name as Windows wrote it: a device path or a packaged app's ID. |
| `DevicePath`, `Path` | For a device path, the device (`\Device\HarddiskVolume3`) and the path on it (`\Windows\System32\cmd.exe`); empty for a packaged app. |
| `LastRunUTC` | When the program last ran (to the 100 ns). |
| `KeyPath`, `KeyLastWrittenUTC` | The user's key and its last write: the most recent update to any of its entries. |
| `DataRaw` | The value's bytes (hex), when they don't hold a time: not `REG_BINARY`, or fewer than 8 bytes. |
| `TransactionLogStatus`, `LogsApplied` | Registry recovery context. |
| `Problems` | Data that doesn't hold a time, and values that can't be read. |

## Investigation strategy

1. Sort by `LastRunUTC` and look at programs in user-writable places (`\Users\`, `Downloads`,
   `Temp`, `ProgramData`), on devices other than the system volume, and administration and
   remote access tools.
2. Check which user ran them, and accounts that don't match a current profile.
3. Correlate with Prefetch (run counts and earlier runs), UserAssist (Explorer starts), Amcache
   and ShimCache (presence), the `$MFT` (the program's file) and event logs.
4. Use `--all-control-sets`, and `parse-registry read --recover-deleted` for deleted BAM values.

## Further Details

- [libyal winreg-kb: Background activity moderator (BAM)](https://github.com/libyal/winreg-kb/blob/main/docs/sources/system-keys/Background-activity-moderator.md)
- [Microsoft: Security identifiers (well-known SIDs)](https://learn.microsoft.com/en-us/windows-server/identity/ad-ds/manage/understand-security-identifiers)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
