# MuiCache

## What MuiCache is

When a user starts a program, Windows reads the program's version resources for the name the
shell should show (the `FileDescription`, for example "Windows Command Processor" for
`cmd.exe`) and caches it per user in `MuiCache`. Windows 8 and later also cache the company
(`CompanyName`). The cache is keyed by the program's full path:

- **Windows 8 and later:** two values per program, `<path>.FriendlyAppName` and
  `<path>.ApplicationCompany`.
- **Windows Vista and 7:** one value per program, `<path>`, holding the friendly name.
- **Windows XP:** the same form, in `NTUSER.DAT`.

MuiCache can provide program-execution leads for a user's shell session, including programs
on removable drives, network shares and folders that no longer exist, with the name and company
the file itself gave.

<span class="evidence-strength">Best conclusion:</span> this shell profile cached a file's path,
description and company. Correlate it with other execution artifacts. The profile does not identify
the account the process ran as. MuiCache has no time per program and no run count.

## Location

```text
C:\Users\<user>\AppData\Local\Microsoft\Windows\UsrClass.dat     Windows Vista and later
Local Settings\Software\Microsoft\Windows\Shell\MuiCache

C:\Documents and Settings\<user>\NTUSER.DAT                         Windows XP
Software\Microsoft\Windows\ShellNoRoam\MUICache
```

`LangID` (the language of the cached strings) is not a program and is left out. Values named
`@<file>,-<number>` (`@shell32.dll,-22019`) are resource strings the shell cached for its own
menus and file types, not programs; they are left out and counted in the summary. On Windows Vista
and later, `NTUSER.DAT` has no MuiCache.

Collect `UsrClass.dat` (and `NTUSER.DAT`) with their transaction logs for every profile, and the
system's `SYSTEM` and `SOFTWARE` hives for the host and user names.

## Analyst cautions

- **Profile owner is not necessarily the process account.** A program started with another
  account can display a window in this user's session and leave metadata in this user's cache.
  `Username` and `UserSid` identify the hive's owner, not the process security context.
- **No time per program.** `KeyLastWrittenUTC` is the key's last write: the most recent change to
  any of its programs, not when a particular one ran. Prefetch, BAM, UserAssist and event logs give
  times.
- **Not only programs the user started by hand.** A DLL or program the shell itself loaded (for
  `Open with`, a shell extension, an installer's child) can appear, such as `explorerframe.dll` or
  `shell32.dll`.
- **The name comes from the file.** `FriendlyAppName` and `ApplicationCompany` are what the file's
  version resources said when it ran. Malware can claim any description and company; a renamed
  tool keeps its original description, which can reveal it (a file named `svchost.exe` whose
  friendly name is a remote access tool).
- **One entry per path.** Running the same program again doesn't add an entry; a copy at another
  path does. A program's entry can stay after the file is deleted.
- **Both forms for one path.** When a path has a friendly name in both forms (an upgraded
  profile), each gets its own record, so neither value is lost.
- **Damaged values.** A value that isn't a string, isn't valid UTF-16 or can't be read whole keeps
  its bytes (`FriendlyAppNameRaw`, `ApplicationCompanyRaw`) and is reported in `Problems`.
  Invalid UTF-16 can also produce replacement characters in the displayed text; use the raw
  column for the original bytes.

## Parse with Juntei

```text
Usage: juntei parse-registry muicache <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find UsrClass.dat and NTUSER.DAT hives
  -f, --file <FILE>      Parse one UsrClass.dat or NTUSER.DAT
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry muicache -d evidence/ -o muicache.csv
juntei parse-registry muicache -f UsrClass.dat -t jsonl \
  | jq 'select(.Path | test("\\\\(Users|ProgramData|Temp)\\\\"; "i"))'
```

The summary lists the programs outside the Windows and Program Files folders (user folders,
other drives, network shares) with their users, and the number of resource strings left out.
Program Files folders with other names (on Windows in other languages) are listed too.
Relative paths and paths that escape an environment-variable folder remain listed. Ordinary
drive paths are checked after resolving `.` and `..` components without accessing the examiner's
file system.

## Output fields

| Field | Interpretation |
|---|---|
| `Host`, `Username`, `UserSID`, `SourcePath` | Host, the profile folder's name, the user's SID (a `UsrClass.dat`'s root key names it; else the system's `ProfileList`), and the hive. |
| `HiveType` | `UsrClass.dat` or `NTUSER.DAT`. |
| `Path` | The program's path: the value's name without `.FriendlyAppName` or `.ApplicationCompany`. |
| `FriendlyAppName` | The program's description, from `<path>.FriendlyAppName` or `<path>`. |
| `ApplicationCompany` | The program's company, from `<path>.ApplicationCompany` (Windows 8 and later). |
| `KeyPath`, `KeyLastWrittenUTC` | The MuiCache key, and its last write: the most recent change to any of its programs. |
| `FriendlyAppNameRaw`, `ApplicationCompanyRaw` | The value's bytes (hex), when they aren't text that decodes cleanly. |
| `TransactionLogStatus`, `LogsApplied` | Registry recovery context: `replayed` when the logs wrote one of the program's values. |
| `Problems` | Values that aren't strings, aren't valid UTF-16 or can't be read. |

## Investigation strategy

1. Look at programs in user-writable places (`\Users\`, `Downloads`, `Temp`, `ProgramData`), on
   other drives and on network shares, and at administration, remote access and archiving tools.
2. Compare the file name with its friendly name and company: a mismatch can reveal a renamed
   tool or a masquerading file.
3. Correlate with Prefetch, BAM and UserAssist (when it ran), Amcache and ShimCache (the file's
   hash and presence), and the `$MFT` (whether the file is still there).
4. Use `parse-registry read --recover-deleted` on `UsrClass.dat` for deleted MuiCache values.

## Further Details

- [Microsoft: VERSIONINFO resource](https://learn.microsoft.com/en-us/windows/win32/menurc/versioninfo-resource)
- [Cyber Triage: MuiCache experiments, GUI sessions, and process-account limitations](https://www.cybertriage.com/blog/muicache-2026-guide/)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
