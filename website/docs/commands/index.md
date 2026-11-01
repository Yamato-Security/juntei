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

## [ESE commands](user-activity.md)

| Command | Input artifact | Detailed guide |
|---|---|---|
| `parse-ese srum` | ESE `SRUDB.dat` | [SRUM](../artifacts/srum.md) |

## [File commands](execution.md)

| Command | Input artifact | Detailed guide |
|---|---|---|
| `parse-file defender-quarantine` | Defender quarantine tree or entry | [Defender Quarantine](../artifacts/defender-quarantine.md) |
| `parse-file jump-lists` | `*.automaticDestinations-ms`, `*.customDestinations-ms` | [Jump Lists](../artifacts/jump-lists.md) |
| `parse-file lnk` | `*.lnk` | [LNK Shortcuts](../artifacts/lnk.md) |
| `parse-file prefetch` | `*.pf` | [Prefetch](../artifacts/prefetch.md) |

## [NTFS commands](filesystem.md)

| Command | Input artifact | Detailed guide |
|---|---|---|
| `parse-ntfs i30` | `$I30` (`$INDEX_ALLOCATION`), or a disk image | [Directory Indexes ($I30)](../artifacts/i30.md) |
| `parse-ntfs logfile` | NTFS `$LogFile` | [NTFS Transaction Log](../artifacts/logfile.md) |
| `parse-ntfs mft` | NTFS `$MFT` | [Master File Table](../artifacts/mft.md) |
| `parse-ntfs usn-journal` | `$UsnJrnl:$J` | [USN Journal](../artifacts/usn-journal.md) |

## [Registry commands](security-registry.md)

| Command | Input artifact | Detailed guide |
|---|---|---|
| `parse-registry amcache` | `Amcache.hve` | [Amcache](../artifacts/amcache.md) |
| `parse-registry persistence` | `SOFTWARE`, `SYSTEM`, `NTUSER.DAT`, `UsrClass.dat` | [Registry Autostarts](../artifacts/persistence.md) |
| `parse-registry read` | Any primary registry hive | [Registry Hives](../artifacts/registry.md) |
| `parse-registry recentdocs` | `NTUSER.DAT` | [RecentDocs](../artifacts/recentdocs.md) |
| `parse-registry sam` | `SAM` | [SAM Local Accounts](../artifacts/sam.md) |
| `parse-registry services` | `SYSTEM` | [Windows Services](../artifacts/services.md) |
| `parse-registry shimcache` | `SYSTEM` or an exported AppCompatCache value | [ShimCache](../artifacts/shimcache.md) |
| `parse-registry taskcache` | `SOFTWARE` | [Task Scheduler Cache](../artifacts/taskcache.md) |
| `parse-registry userassist` | `NTUSER.DAT` | [UserAssist](../artifacts/userassist.md) |

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
132 checks, allow/block classification, payload metadata, coverage, and timestamp correlation.

## Top-level usage

```text
Usage: juntei <COMMAND>

Commands:
  auto-parse         Discover and parse Windows evidence
  persistence-hunter  Hunt offline Windows persistence
  help               Print this message or the help of the given subcommand(s)
  parse-ese          Parse ESE databases
  parse-file         Parse file artifacts
  parse-ntfs         Parse NTFS artifacts
  parse-registry     Parse registry hives

Options:
  -h, --help     Print help
  -V, --version  Print version
```

The source pages reproduce command usage and give short examples. The artifact guides explain
the evidence, output fields, limitations, and investigation strategy.
