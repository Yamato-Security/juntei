# Collecting Evidence

Juntei parses files already collected from Windows. Use a trusted acquisition tool and preserve original metadata where possible. VHDX disk images can be parsed without extracting them: see [Mounting Disk Images](../commands/mount.md).

## Common locations

| Artifact | Typical location |
|---|---|
| Amcache | `C:\Windows\AppCompat\Programs\Amcache.hve` |
| `$MFT` | NTFS volume metadata file `C:\$MFT` |
| `$LogFile` | NTFS volume metadata file `C:\$LogFile` |
| Prefetch | `C:\Windows\Prefetch\*.pf` |
| USN Journal | NTFS metadata stream `C:\$Extend\$UsnJrnl:$J` |
| SRUM | `C:\Windows\System32\sru\SRUDB.dat` |
| User Access Logging (Windows Server) | `C:\Windows\System32\LogFiles\Sum\` |
| Internet Explorer and legacy Edge (WebCache) | `%LOCALAPPDATA%\Microsoft\Windows\WebCache\` |
| Web browsers | `%LOCALAPPDATA%\Google\Chrome\User Data\`, `%LOCALAPPDATA%\Microsoft\Edge\User Data\`, `%APPDATA%\Mozilla\Firefox\Profiles\` and the other browsers' profile folders |
| LNK / Recent | `%APPDATA%\Microsoft\Windows\Recent\*.lnk` and other user/system locations |
| Jump Lists | `%APPDATA%\Microsoft\Windows\Recent\AutomaticDestinations` and `CustomDestinations` |
| ShimCache | `SYSTEM` hive: `...\Session Manager\AppCompatCache\AppCompatCache` |
| Defender quarantine | `C:\ProgramData\Microsoft\Windows Defender\Quarantine` |
| Machine registry | `C:\Windows\System32\config\SYSTEM`, `SOFTWARE`, `SAM`, `SECURITY` |
| User registry | `%USERPROFILE%\NTUSER.DAT`, `%LOCALAPPDATA%\Microsoft\Windows\UsrClass.dat` |

## Files that must travel together

- Collect registry hives with adjacent `.LOG`, `.LOG1`, and `.LOG2` files. Juntei can replay logs
  for registry commands and ShimCache when the hive is dirty.
- Collect SRUM's `SRUDB.dat` consistently. Juntei currently reports a dirty ESE database but does
  not replay `SRU*.log`; records existing only in those logs are unavailable.
- Collect `$MFT` beside `$Extend/$J` when possible. The USN record stores parent file references,
  not a complete path; the MFT improves path resolution.
- Collect `$LogFile` with the same volume's `$MFT` as close in time as possible. `$LogFile` wraps
  rapidly; later collection can lose the earlier redo/undo records entirely.
- Collect SRUM with the matching `SOFTWARE` hive to resolve account and Wi-Fi profile names.
- Collect the whole User Access Logging `Sum` directory: `SystemIdentity.mdb` gives the role names
  and lists the yearly `{GUID}.mdb` databases. As with SRUM, Juntei does not replay its `Svc*.log`
  transaction logs.
- Collect browser databases with the `-wal` files beside them (`places.sqlite-wal`,
  `cookies.sqlite-wal`): the newest changes, and the older page versions deleted entries are
  recovered from, are only there until the browser checkpoints.
- Collect the whole `WebCache` folder: Windows keeps `WebCacheV01.dat` open while the user is
  logged on, and its newest changes are only in the `V01*.log` transaction logs, which Juntei does
  not replay.
- Collect the whole Defender `Quarantine` tree so Entries, ResourceData, and Resources can be matched.

## Live-system cautions

Open files can change during a copy. Registry hives and ESE databases may be internally dirty.
ShimCache may not flush its newest in-memory entries until shutdown. A forensic collection tool
that uses volume snapshots or raw access is preferable to an ordinary file copy.

## Safety

Evidence can contain malware, exploit samples, malicious LNK files, and spreadsheet-formula text.
Analyze on an isolated system, do not execute recovered streams, and use `--extract` only when you
intend to handle dangerous content. Preserve the untouched source separately.
