# FeatureUsage, RecentApps and Taskbar Pins

## What they are

Explorer keeps several records of how a user uses applications in the user's `NTUSER.DAT`:

- **FeatureUsage** (Windows 10 and later): counters per application, by kind of use.
  - `AppSwitched`: how many times the user switched to the application's window (taskbar click,
    Alt+Tab).
  - `AppLaunch`: how many times it was started from the taskbar (a pinned item).
  - `ShowJumpView`: how many times its jump list was opened (right-click on the taskbar).
  - `AppBadgeUpdated`: how many times its taskbar badge (an unread count) changed.
  - `TrayButtonClicked`: how many times each taskbar button (`StartButton`, `SearchButton`,
    `TaskViewButton`, `ClockButton`, …) was clicked.

  An application is an application user model ID (`Microsoft.Windows.Explorer`,
  `Microsoft.WindowsCalculator_8wekyb3d8bbwe!App`), a path whose first part may be a known
  folder's GUID (`{1AC14E77-…}\cmd.exe` is `%windir%\system32\cmd.exe`), or a process ID
  (`*PID00001d74`) for windows Explorer couldn't match to an application.
- **RecentApps** (Windows 10 up to 1709): the applications a user ran recently, with a launch
  count and the last time; under each, the files it opened (`RecentItems`).
- **Taskbar pins** (`Taskband`): the programs pinned to the taskbar, as shell items
  (`Favorites`) and as the link of each pinned shortcut (`FavoritesResolve`). On Windows 10 and
  later each pin says whether Windows pinned it (`SystemPinned`) or the user did (`UserPinned`).
- **ProgramsCache** (`StartPage`, `StartPage2`; Windows XP to 7): the Start menu's cached
  programs (`ProgramsCache`), and on Windows 7 the Start menu's and the taskbar's pinned ones
  (`ProgramsCacheSMP`, `ProgramsCacheTBP`).

<span class="evidence-strength">Best conclusion:</span> this user used this application this
many times in this way (FeatureUsage), ran it at this time (RecentApps), or pinned it. The
counters have no time of their own; only the key's last write and RecentApps' times date
anything.

## Location

```text
C:\Users\<user>\NTUSER.DAT
Software\Microsoft\Windows\CurrentVersion\Explorer\FeatureUsage\<kind>     value: application, data: count
Software\Microsoft\Windows\CurrentVersion\Search\RecentApps\{GUID}         AppId, AppPath, LaunchCount, LastAccessedTime
Software\Microsoft\Windows\CurrentVersion\Search\RecentApps\{GUID}\RecentItems\{GUID}   DisplayName, Path, LastAccessedTime
Software\Microsoft\Windows\CurrentVersion\Explorer\Taskband                Favorites, FavoritesResolve
Software\Microsoft\Windows\CurrentVersion\Explorer\StartPage2              ProgramsCache, ProgramsCacheSMP, ProgramsCacheTBP
Software\Microsoft\Windows\CurrentVersion\Explorer\StartPage               ProgramsCache (Windows XP, Vista)
```

`FeatureUsage`'s `KeyCreationTime` is when it started counting; the summary shows it. Collect
`NTUSER.DAT` with its transaction logs for every profile, and the system's `SYSTEM` and
`SOFTWARE` hives for the host and the user's SID.

## Analyst cautions

- **Counts without times.** A FeatureUsage count says how often, not when. The kind's key's
  last-write time (`KeyLastWrittenUTC`) is the most recent change to any of its counts.
- **Not every run.** `AppSwitched` counts window switches, not starts; an application that ran
  without a window, or that the user never switched to, isn't counted. `AppLaunch` counts only
  starts from the taskbar.
- **Packaged apps and paths.** An application is named as Windows grouped its windows: a
  packaged app's ID, an ID a program set for itself (`Chrome`, `VMware.Workstation.vmui`), or a
  path. `AppPath` gives the path with a known folder's GUID replaced by the folder's
  default path.
- **Default pins.** New profiles get default pins (File Explorer, Edge, the Store); on Windows 10
  and later they say `SystemPinned`. A `UserPinned` pin is the user's choice.
- **Pins in two lists.** `Favorites` and `FavoritesResolve` describe the same pins in the same
  order (`Index`), but they are given as separate records.
- **ProgramsCache paths** are relative: a folder (`Programs\Accessories`) is followed by the
  shortcuts in it (`Notepad.lnk`).
- **Damaged values.** A count that isn't a `REG_DWORD`, a time that isn't a `REG_QWORD`, a pin
  that isn't understood or a link that isn't one keeps its bytes (`DataRaw`) and is reported in
  `Problems`.

## Parse with Juntei

```text
Usage: juntei parse-registry feature-usage <INPUT> [OPTIONS]

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
juntei parse-registry feature-usage -d evidence/ -o featureusage.csv
juntei parse-registry feature-usage -f NTUSER.DAT -t jsonl \
  | jq 'select(.Source == "AppSwitched") | [.Count, .AppPath // .App]'
```

The summary lists records by source, when each hive's FeatureUsage started counting, the
applications switched to most, the recent apps and items (newest first) and the taskbar's pins.

## Output fields

| Field | Interpretation |
|---|---|
| `Host`, `User`, `UserSID`, `SourcePath` | Host, the profile folder's name, the user's SID from the system's `ProfileList`, and the hive. |
| `Source` | `AppSwitched`, `AppLaunch`, `ShowJumpView`, `AppBadgeUpdated`, `TrayButtonClicked` (FeatureUsage), `RecentApps`, `RecentItems`, `Taskband` or `ProgramsCache`. |
| `App` | A FeatureUsage value's name, a recent app's `AppId`, or a pin's application user model ID. |
| `AppPath` | `App` as a path, a known folder's GUID replaced by its default path; empty for IDs. |
| `Count` | A FeatureUsage count, or a recent app's `LaunchCount`. |
| `LastAccessedUTC` | A recent app's or item's `LastAccessedTime`. |
| `ItemPath` | A recent item's `Path` (or a recent app's `AppPath`); a pin's or cached program's shell item path (`User Pinned\TaskBar\File Explorer.lnk`, `Applications\<app ID>`); a `FavoritesResolve` link's path. |
| `DisplayName` | A recent item's `DisplayName`. |
| `PinType` | `SystemPinned` or `UserPinned`, for a `Favorites` pin. |
| `Index` | A pin's or cached program's place in its list. |
| `ValueName`, `KeyPath`, `KeyLastWrittenUTC` | The value and key it comes from, and the key's last write. |
| `DataRaw` | Bytes (hex) that don't decode. |
| `TransactionLogStatus`, `LogsApplied` | Registry recovery context. |
| `Problems` | Values of the wrong type, pins or lists that aren't understood, links that aren't links. |

## Investigation strategy

1. Sort `AppSwitched` by `Count` for the applications a user worked in most, and look for
   administration, remote access, archiving and transfer tools, and paths in user-writable places
   (`\Users\`, `Downloads`, `Temp`).
2. Check the pins: a `UserPinned` tool, or a pin whose shortcut is in an unusual place.
3. Use RecentApps' times, with UserAssist, Prefetch, BAM and Jump Lists, to date the use.
4. Compare `KeyCreationTime` with the profile's creation: counts start then.
5. Use `parse-registry read --recover-deleted` for deleted counters and pins.

## Further Details

- [Microsoft: Application User Model IDs](https://learn.microsoft.com/en-us/windows/win32/shell/appids)
- [Microsoft: KNOWNFOLDERID](https://learn.microsoft.com/en-us/windows/win32/shell/knownfolderid)
- [libyal winreg-kb: Programs cache](https://winreg-kb.readthedocs.io/en/latest/sources/explorer-keys/Program-cache.html)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
