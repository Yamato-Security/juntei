# Supported Evidence

| Evidence source | Juntei command | Most useful for | Important limitation |
|---|---|---|---|
| Amcache | `parse-registry amcache` | File inventory, paths, SHA-1, PE metadata, devices | Presence is not execution; publisher is not signature validation |
| NTFS `$MFT` | `parse-ntfs mft` | Files, deleted records, paths, timestamps, ADS | Reused records and timestomping can mislead |
| NTFS `$LogFile` | `parse-ntfs logfile` | Redo/undo metadata, old names and timestamps, writes to the USN Journal | Short circular retention; metadata times are not transaction times; no replay |
| Prefetch | `parse-file prefetch` | Program execution, run count, recent run times, loaded files | Only a limited run-time history is retained |
| USN Journal `$J` | `parse-ntfs usn-journal` | Creates, deletes, renames, writes, historical paths | Circular journal; records changes, not file contents |
| Directory indexes `$I30` | `parse-ntfs i30` | Files in a directory; names, times, and sizes of possible deleted or renamed files left in index slack | Slack is partial and says nothing about when an entry was removed |
| SRUM | `parse-ese srum` | Per-app/user network and resource use | Usually aggregated; not packet or destination-level proof |
| LNK shortcuts | `parse-file lnk` | Opened targets, arguments, volumes, shares, source machine | A shortcut can be created without the target running |
| Jump Lists | `parse-file jump-lists` | Application-opened items, pinning, usage and embedded LNK data | Application-specific behavior and retention |
| ShimCache | `parse-registry shimcache` | Historical executable paths and compatibility checks | Cache may not be flushed until shutdown; entry alone is not execution |
| Defender quarantine | `parse-file defender-quarantine` | Detection, original path, hashes, download origin, recovered bytes | Detection name is a product verdict; extraction is hazardous |
| Registry hives | `parse-registry read` | Configuration, persistence, deleted/replayed values | Key time is not a value time; live hives need transaction logs |
| Services | `parse-registry services` | Service/driver persistence and failure actions | Configuration does not prove the service started |
| RecentDocs | `parse-registry recentdocs` | Recently opened files and folders per user | Only the most recent item in each list gets a defensible open time |
| TaskCache | `parse-registry taskcache` | Scheduled-task actions, metadata, hidden tasks | Correlate with task XML and event logs for runtime proof |
| Registry autostarts | `parse-registry persistence` | Run keys, Winlogon, IFEO, AppInit/AppCert DLLs, LSA packages, COM hijacks and other autostart locations | Configuration, not proof of execution; most entries are Windows defaults |
| UserAssist | `parse-registry userassist` | Programs and shortcuts a user started from Explorer, with run counts, focus time and last run | Only Explorer starts are counted; new profiles are seeded with default items |
| SAM | `parse-registry sam` | Local accounts, flags, RIDs, logon/password metadata | Local accounts only; last-logon values are not a complete logon history |

## Collection types

Juntei accepts a single artifact with `-f` or searches a collection recursively with `-d`.
Directory mode is designed for common triage layouts where each host contains paths such as
`C/Windows/System32/config/SYSTEM` or `C/Users/alice/NTUSER.DAT`.

See [Collecting Evidence](../getting-started/collection.md) before analyzing copied registry or ESE
databases: related transaction logs can be essential to reconstructing the state Windows saw.
