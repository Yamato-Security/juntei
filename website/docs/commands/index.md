# Command List

Individual parsers are grouped by their source type. Run `juntei --help`,
`juntei <SOURCE> --help`, or `juntei <SOURCE> <ARTIFACT> --help` to explore them:

```bash
juntei parse-ntfs --help
juntei parse-registry amcache --help
juntei parse-file prefetch -d evidence/ -o prefetch.csv
```

For a multi-computer Windows evidence collection, use
[`juntei auto-parse -d <EVIDENCE> -o <OUTPUT>`](auto-parse.md). It invokes the same grouped commands
and organizes results and logs by computer and investigative category.

For a chronologically sorted timeline across parsed artifacts, use
[`juntei super-timeline -d <PARSED> -o SuperTimeline.csv`](super-timeline.md).
It expands and labels timestamp meanings; `auto-parse -j` continues to retain native records.

## [Disk images](mount.md)

| Command | Input | Detailed guide |
|---|---|---|
| `mount` | VHDX / E01 / Ex01 disk image | [Mounting Disk Images](mount.md) |

## [ESE commands](user-activity.md)

| Command | Input artifact | Detailed guide |
|---|---|---|
| `parse-ese srum` | ESE `SRUDB.dat` | [SRUM](../artifacts/srum.md) |
| `parse-ese ual` | ESE `Current.mdb`, `{GUID}.mdb`, `SystemIdentity.mdb` | [User Access Logging (UAL)](../artifacts/ual.md) |
| `parse-ese webcache` | ESE `WebCacheV01.dat` | [Internet Explorer and Legacy Edge](../artifacts/webcache.md) |

## [File commands](execution.md)

| Command | Input artifact | Detailed guide |
|---|---|---|
| `parse-file defender-quarantine` | Defender quarantine tree or entry | [Defender Quarantine](../artifacts/defender-quarantine.md) |
| `parse-file jump-lists` | `*.automaticDestinations-ms`, `*.customDestinations-ms` | [Jump Lists](../artifacts/jump-lists.md) |
| `parse-file lnk` | `*.lnk` | [LNK Shortcuts](../artifacts/lnk.md) |
| `parse-file outlook` | `*.pst`, `*.ost`, `*.msg`, `*.oft` | [Outlook](../artifacts/outlook.md) |
| `parse-file prefetch` | `*.pf` | [Prefetch](../artifacts/prefetch.md) |

## [NTFS commands](filesystem.md)

| Command | Input artifact | Detailed guide |
|---|---|---|
| `parse-ntfs boot` | `$Boot`, or an NTFS volume image | [Boot Sector ($Boot)](../artifacts/boot.md) |
| `parse-ntfs i30` | `$I30` (`$INDEX_ALLOCATION`), or a disk image | [Directory Indexes ($I30)](../artifacts/i30.md) |
| `parse-ntfs logfile` | NTFS `$LogFile` | [NTFS Transaction Log](../artifacts/logfile.md) |
| `parse-ntfs mft` | NTFS `$MFT` | [Master File Table](../artifacts/mft.md) |
| `parse-ntfs secure-sds` | `$Secure:$SDS` (`$Secure_$SDS`) | [Security Descriptors ($Secure:$SDS)](../artifacts/secure-sds.md) |
| `parse-ntfs usn-journal` | `$UsnJrnl:$J` | [USN Journal](../artifacts/usn-journal.md) |

## [Registry commands](security-registry.md)

| Command | Input artifact | Detailed guide |
|---|---|---|
| `parse-registry amcache` | `Amcache.hve` | [Amcache](../artifacts/amcache.md) |
| `parse-registry networks` | `SOFTWARE`, `SYSTEM` | [Networks](../artifacts/networks.md) |
| `parse-registry lsa-secrets` | `SECURITY` | [LSA Secrets](../artifacts/lsa-secrets.md) |
| `parse-registry bcd` | `BCD` (and `BCD-Template`) | [Boot Configuration Data (BCD)](../artifacts/bcd.md) |
| `parse-registry app-compat` | `SOFTWARE`, `NTUSER.DAT` | [Application Compatibility](../artifacts/appcompat.md) |
| `parse-registry auditpol` | `SECURITY` | [Audit Policy](../artifacts/auditpol.md) |
| `parse-registry office` | `NTUSER.DAT` | [Microsoft Office](../artifacts/office.md) |
| `parse-registry feature-usage` | `NTUSER.DAT` | [FeatureUsage, RecentApps and Taskbar Pins](../artifacts/featureusage.md) |
| `parse-registry app-history` | `NTUSER.DAT` | [Application History](../artifacts/app-history.md) |
| `parse-registry mru` | `NTUSER.DAT` | [Most-Recently-Used Lists](../artifacts/mru.md) |
| `parse-registry bam` | `SYSTEM` (with `SAM`, `SOFTWARE` for user names) | [BAM and DAM](../artifacts/bam.md) |
| `parse-registry muicache` | `UsrClass.dat` (`NTUSER.DAT` on Windows XP) | [MuiCache](../artifacts/muicache.md) |
| `parse-registry installed-programs` | `SOFTWARE`, `NTUSER.DAT` | [Installed Programs](../artifacts/installed-programs.md) |
| `parse-registry open-save-mru` | `NTUSER.DAT` | [Open and Save As Dialogs](../artifacts/opensavemru.md) |
| `parse-registry persistence` | `SOFTWARE`, `SYSTEM`, `NTUSER.DAT`, `UsrClass.dat` | [Registry Autostarts](../artifacts/persistence.md) |
| `parse-registry read` | Any primary registry hive | [Registry Hives](../artifacts/registry.md) |
| `parse-registry recent-docs` | `NTUSER.DAT` | [RecentDocs](../artifacts/recentdocs.md) |
| `parse-registry remote-access` | `NTUSER.DAT` (with `SOFTWARE` for user SIDs) | [Remote Access (User Side)](../artifacts/remote-access.md) |
| `parse-registry sam` | `SAM` | [SAM Local Accounts](../artifacts/sam.md) |
| `parse-registry security-settings` | `SOFTWARE`, `SYSTEM`, `NTUSER.DAT` | [Security Settings](../artifacts/security-settings.md) |
| `parse-registry services` | `SYSTEM` | [Windows Services](../artifacts/services.md) |
| `parse-registry shimcache` | `SYSTEM` or an exported AppCompatCache value | [ShimCache](../artifacts/shimcache.md) |
| `parse-registry system-info` | `SYSTEM`, `SOFTWARE`, `SAM`, `SECURITY` of one system | [System Information](../artifacts/system-info.md) |
| `parse-registry taskcache` | `SOFTWARE` | [Task Scheduler Cache](../artifacts/taskcache.md) |
| `parse-registry usb` | `SYSTEM`, with `SOFTWARE` and `NTUSER.DAT` | [USB Devices](../artifacts/usb.md) |
| `parse-registry user-assist` | `NTUSER.DAT` | [UserAssist](../artifacts/userassist.md) |
| `parse-registry shell-bags` | `UsrClass.dat`, `NTUSER.DAT` | [ShellBags](../artifacts/shellbags.md) |

## [SQLite commands](sqlite.md)

| Command | Input artifact | Detailed guide |
|---|---|---|
| `parse-sqlite browser` | `History`, `Cookies`, `Web Data`, `places.sqlite`, `cookies.sqlite`, `formhistory.sqlite` | [Web Browsers](../artifacts/browser.md) |

## Source types and format changes

Groups describe the formats Juntei currently supports. `parse-registry amcache` reads
`Amcache.hve`; it does not imply support for SQLite Amcache inventory databases. Artifact parser
crates remain separate so support for another storage format can be added independently.

The former flat parser names have been replaced by the source-and-artifact paths above. Keep
existing parser options after the artifact name. `auto-parse` and existing registry subcommands
keep their names.

## [Persistence Hunter](persistence-hunter.md)

`juntei persistence-hunter` hunts across several source formats, so it is a top-level analysis
command. It complements the full configuration inventory from `parse-registry persistence` with
131 checks, allow/block classification, payload metadata, coverage, and timestamp correlation.

## Top-level usage

```text
Usage: juntei <COMMAND>

Commands:
  auto-parse          Discover and parse Windows evidence
  help                Print this message or the help of the given subcommand(s)
  mount               Mount a VHDX or E01 disk image read-only (FUSE)
  parse-ese           Parse ESE databases
  parse-file          Parse file artifacts
  parse-ntfs          Parse NTFS artifacts
  parse-registry      Parse registry hives
  parse-sqlite        Parse SQLite databases
  persistence-hunter  Hunt offline Windows persistence

Options:
  -h, --help     Print help
  -V, --version  Print version
```

The source pages reproduce command usage and give short examples. The artifact guides explain
the evidence, output fields, limitations, and investigation strategy.
