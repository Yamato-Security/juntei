# UserAssist

## What UserAssist is

Windows Explorer counts the programs and shortcuts a user starts from the Explorer shell: the
Start menu, the taskbar, the desktop, and double-clicks in folders. It keeps the counts in the
user's `NTUSER.DAT`. Each item has:

- a run count;
- on Windows 7 and later, how many times and how long (in milliseconds) it had the focus;
- when it last ran.

The value names are ROT13-encoded. A program's name usually starts with a known folder's GUID
(`{1AC14E77-…}` is `System32`).

UserAssist shows what a particular user started interactively, which most other execution
evidence doesn't attribute to a user.

<span class="evidence-strength">Best conclusion:</span> the user started this program or shortcut
through Explorer, this many times, last at this time. Programs started another way (a command
line, a service, a scheduled task, another program) are not counted.

## Location

```text
C:\Users\<user>\NTUSER.DAT
Software\Microsoft\Windows\CurrentVersion\Explorer\UserAssist\{GUID}\Count
```

| GUID | Holds |
|---|---|
| `{CEBFF5CD-ACE2-4F4F-9178-9926F41749EA}` | Programs (`executable`), Windows 7 and later |
| `{F4E57C4B-2036-45F0-A9AB-443BCFE33D9F}` | Shortcuts (`shortcut`), Windows 7 and later |
| `{75048700-EF1F-11D0-9888-006097DEACF9}` | Programs and shortcuts (`active_desktop`), Windows 2000 to Vista |
| `{5E6AB780-7743-11CF-A12B-00AA004AE837}` | Internet toolbar (`internet_toolbar`), Windows 2000 to Vista |

Other GUIDs are reported as `other`. Collect `NTUSER.DAT` with its transaction logs for every
profile, and the system's `SOFTWARE` hive, which gives the user's SID.

## Analyst cautions

- **New profiles are seeded.** On Windows 7 and later, a new profile starts with default items
  (Getting Started, Calculator, Paint, Sticky Notes, …). Their run counts count down from 14,
  and they all share one last-run time, around when the profile was created. They aren't user
  activity.
- **Version 3 counts from 5** (Windows 2000 to Vista). The stored count starts at 5, so 5 is
  subtracted from it. A stored count under 5 is reported in `Problems`.
- **Paths use the known folder's default path** (`%windir%\system32`, `%ProgramFiles%`,
  `%APPDATA%\…`). A folder redirected elsewhere still shows its default. App IDs
  (`Microsoft.Windows.Explorer`, `Microsoft.WindowsNotepad_8wekyb3d8bbwe!App`) and Control Panel
  items (`::{…}`) have no path.
- **Tracking can be turned off** with `Explorer\Advanced` `Start_TrackProgs` = 0,
  `Policies\Explorer` `NoInstrumentation` = 1, or `UserAssist\Settings` `NoLog` = 1. The summary
  lists the hives where it is off. An empty or missing UserAssist on an active profile can mean
  anti-forensics.
- **`NoEncrypt` = 1** makes Windows write names without ROT13. Such names decode wrongly; a
  problem says so.
- **The last-run time and the counts are kept per item,** but the key's last-write time
  (`KeyLastWrittenUTC`) is only the time of the most recent update to any item.
- **Run counts can be 0** for items that only had the focus. Focus counts and times exist only in
  version 5 (Windows 7 and later).

## Parse with Juntei

```text
Usage: juntei parse-registry userassist <INPUT> [OPTIONS]

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
juntei parse-registry userassist -d evidence/ -o userassist.csv
juntei parse-registry userassist -f NTUSER.DAT -t jsonl \
  | jq 'select(.path? // "" | test("\\\\(Users|ProgramData|Temp)\\\\"; "i"))'
```

The summary lists the items run last (newest first) with their users, and the hives where
UserAssist is turned off.

## Output fields

| Field | Interpretation |
|---|---|
| `Host`, `Username`, `UserSid`, `SourcePath` | Host, the profile folder's name, the user's SID from the system's `ProfileList` (with the system's `SOFTWARE` hive), and the hive. |
| `Guid`, `Kind` | The `{GUID}` key and what it holds (`executable`, `shortcut`, `active_desktop`, `internet_toolbar`, `other`). |
| `NameRot13`, `Name` | The value's name as stored, and decoded. |
| `Path` | The decoded name as a path, with a known folder's GUID replaced by its default path; empty for app IDs and shell items. |
| `RunCount` | How many times it was started from Explorer (version 3: the stored count minus 5). |
| `FocusCount`, `FocusTimeMs` | How many times it had the focus, and for how long in milliseconds (version 5). |
| `LastRunUTC` | When it was last started. Empty when Windows stored 0. |
| `Format`, `SessionId` | The data's layout (5: 72 bytes, 3: 16 bytes) and the session number it was last written in. |
| `KeyLastWrittenUTC` | The `Count` key's last write: the most recent update to any of its items. |
| `DataRaw` | The value's bytes (hex), when they don't decode. |
| `TransactionLogStatus`, `LogsApplied` | Registry recovery context. |
| `Problems` | Data of an unknown size, a size that disagrees with the key's `Version`, a version 3 count under 5. |

## Investigation strategy

1. Sort by `LastRunUTC` and set aside the seeded items: they share one time, around the profile's
   creation.
2. Look for programs in user-writable places (`%USERPROFILE%`, `Downloads`, `Temp`,
   `ProgramData`), removable drives and network paths, and for administration and remote access
   tools.
3. Correlate with Prefetch (execution with run times), Amcache and ShimCache (presence),
   Jump Lists and LNK files (what was opened with them), and the `$MFT` for the program files.
4. Check the hives where tracking is off, and profiles whose UserAssist is empty, for clearing.
5. Use `parse-registry read --recover-deleted` for deleted UserAssist values.

## Further Details

- [Microsoft: KNOWNFOLDERID](https://learn.microsoft.com/en-us/windows/win32/shell/knownfolderid)
- [libyal winreg-kb: User Assist keys](https://github.com/libyal/winreg-kb/blob/main/documentation/User%20Assist%20keys.asciidoc)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
