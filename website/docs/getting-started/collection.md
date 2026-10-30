# Collecting Evidence

Juntei parses files already collected from Windows. Use a trusted acquisition tool and preserve
original metadata where possible.

## Common locations

| Artifact | Typical location |
|---|---|
| Amcache | `C:\Windows\AppCompat\Programs\Amcache.hve` |
| `$MFT` | NTFS volume metadata file `C:\$MFT` |
| Prefetch | `C:\Windows\Prefetch\*.pf` |
| USN Journal | NTFS metadata stream `C:\$Extend\$UsnJrnl:$J` |
| SRUM | `C:\Windows\System32\sru\SRUDB.dat` |
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
- Collect SRUM with the matching `SOFTWARE` hive to resolve account and Wi-Fi profile names.
- Collect the whole Defender `Quarantine` tree so Entries, ResourceData, and Resources can be matched.

## Live-system cautions

Open files can change during a copy. Registry hives and ESE databases may be internally dirty.
ShimCache may not flush its newest in-memory entries until shutdown. A forensic collection tool
that uses volume snapshots or raw access is preferable to an ordinary file copy.

## Safety

Evidence can contain malware, exploit samples, malicious LNK files, and spreadsheet-formula text.
Analyze on an isolated system, do not execute recovered streams, and use `--extract` only when you
intend to handle dangerous content. Preserve the untouched source separately.
