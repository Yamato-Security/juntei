# Command List

Run `juntei --help` for the current list and `juntei <COMMAND> --help` for one command.

| Command | Input artifact | Detailed guide |
|---|---|---|
| `auto-parse` | Multi-computer Windows evidence collection | [Automatic Collection Parsing](auto-parse.md) |
| `parse-amcache` | `Amcache.hve` | [Amcache](../artifacts/amcache.md) |
| `parse-mft` | NTFS `$MFT` | [Master File Table](../artifacts/mft.md) |
| `parse-logfile` | NTFS `$LogFile` | [NTFS Transaction Log](../artifacts/logfile.md) |
| `parse-prefetch` | `*.pf` | [Prefetch](../artifacts/prefetch.md) |
| `parse-usn-journal` | `$UsnJrnl:$J` | [USN Journal](../artifacts/usn-journal.md) |
| `parse-i30` | `$I30` (`$INDEX_ALLOCATION`), or a disk image | [Directory Indexes ($I30)](../artifacts/i30.md) |
| `parse-srum` | `SRUDB.dat` | [SRUM](../artifacts/srum.md) |
| `parse-lnk` | `*.lnk` | [LNK Shortcuts](../artifacts/lnk.md) |
| `parse-jump-lists` | `*.automaticDestinations-ms`, `*.customDestinations-ms` | [Jump Lists](../artifacts/jump-lists.md) |
| `parse-shimcache` | `SYSTEM` or raw AppCompatCache | [ShimCache](../artifacts/shimcache.md) |
| `parse-defender-quarantine` | Defender quarantine tree or entry | [Defender Quarantine](../artifacts/defender-quarantine.md) |
| `parse-registry read` | Any primary registry hive | [Registry Hives](../artifacts/registry.md) |
| `parse-registry services` | `SYSTEM` | [Windows Services](../artifacts/services.md) |
| `parse-registry recentdocs` | `NTUSER.DAT` | [RecentDocs](../artifacts/recentdocs.md) |
| `parse-registry taskcache` | `SOFTWARE` | [Task Scheduler Cache](../artifacts/taskcache.md) |
| `parse-registry sam` | `SAM` | [SAM Local Accounts](../artifacts/sam.md) |

## Top-level usage

```text
Usage: juntei <COMMAND>

Options:
  -h, --help     Print help
  -V, --version  Print version
```

The pages in this section reproduce the command usage and give short examples. The artifact guides
explain the evidence, output fields, limitations, and investigation strategy.
