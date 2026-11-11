# Parser Validation

Juntei's maintained parser crates are tested against public evidence and independent
implementations. These comparisons help detect regressions and interpretation differences; they
do not make any parser infallible. Preserve the source evidence, review each record's problem or
damage fields, and validate consequential findings with another artifact or implementation.

The statements below preserve the validation information that was formerly kept in Juntei's main
README. Follow the linked parser repositories for current test fixtures and implementation details.

## Amcache

Juntei uses [ys-amcache-parser](https://github.com/Yamato-Security/ys-amcache-parser), an
Apache-2.0 fork of Security Ronin's
[amcache-forensic](https://github.com/SecurityRonin/amcache-forensic). Its
[output documentation](https://github.com/Yamato-Security/ys-amcache-parser/blob/main/docs/output.md)
defines every executable, device, finding, and summary field.

## Master File Table

The mft crate of [ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) is a maintained fork of Omer
Ben-Amram's [mft](https://github.com/omerbenamram/mft) crate. Across eight `$MFT` files—the crate's
sample, the NIST hacking case, and six DFIR Artifact Museum files—1,048,733 CSV rows and columns
matched a reference parser in order. Comparisons covered default output, `--all-times`,
`--short-names`, and `--cutoff`; timestamp presentation was the intentional difference.

## NTFS transaction log ($LogFile)

The logfile crate of [ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) is maintained from
[Sootmark ntfslog](https://github.com/Sootmark/ntfslog). Its validation covers six complete DFIR
Artifact Museum logs, public version 1.1 and 2.0 fixtures, decoded redo/undo metadata, and journal
writes compared against independently parsed NTFS data. See the parser repository for detailed
coverage and known differences.

Juntei adds CLI tests that compare every CSV field with JSON and JSONL on the public go-ntfs test
volume (2,129 records), verify separate redo/undo data and raw payload lengths, test damaged logs,
refuse evidence overwrites, and keep current MFT paths from following reused record numbers.
Regression cases distinguish affected MFT records from the open attribute they write through,
check flat directory-entry/data-attribute metadata, and prevent cross-volume MFT pairing.
Multi-volume `auto-parse` tests cover separate CSV files and a single JSONL file with provenance.

The command was also checked on six privately collected Windows logs: all 708,465 exported CSV
records matched JSONL field for field, and per-host `auto-parse` exports retained the same record
counts. The source evidence is not distributed with the repository. These checks verify output
consistency, not that every logged transaction represents a completed user action.

## Prefetch

[ys-prefetch-parser](https://github.com/Yamato-Security/ys-prefetch-parser) is a maintained fork of
ForensicRS's [frnsc-prefetch](https://github.com/ForensicRS/frnsc-prefetch). Its decoded output was
compared with [libscca](https://github.com/libyal/libscca), libyal's independent Prefetch
implementation.

## USN Journal

The usn crate of [ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) is a maintained
fork of [Sootmark usn](https://github.com/Sootmark/usn). On six real journals, Juntei's
time-aware parent paths matched an independent parser wherever that parser resolved a path. Juntei
also resolved 4,321 records that the comparison left unresolved. Its carver produced no false
records while tested against 880 MB of `$MFT` data.

## Directory indexes ($I30)

The `i30` module of [ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser)'s mft crate reads INDX
records and their slack. On two `$I30` streams of a volume built with ntfs-3g (Juntei's
`test_files/i30/`) and on three public Windows-written `$I30` samples, its entries in use and in
slack matched those of Maxim Suhanov's [dfir_ntfs](https://github.com/msuhanov/dfir_ntfs), with the
same names and times. Where dfir_ntfs reports a parent reference past 32 bits (bytes written over
the parent), Juntei leaves the parent empty.

## Boot sector ($Boot)

The boot crate of [ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) is Yamato
Security's own. On the five public `$Boot` files of the DFIR Artifact Museum (Windows 10, 11 and
Server 2012 R2), every field matched a reference parser. On volumes made with `mkntfs` with 512-,
1,024- and 4,096-byte sectors and 4 KiB to 64 KiB clusters, the sizes, the `$MFT` and `$MFTMirr`
clusters and the serial numbers matched The Sleuth Kit's `fsstat`. The Museum's Windows 11 `$Boot`
has the volume serial number that the same host's LNK file records, and the five files have the
same boot code.

## Security descriptors ($Secure:$SDS)

The secure crate of [ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) is Yamato
Security's own. Across the Museum's five `$SDS` streams (7,808 descriptors, 60,781 ACEs), the
security IDs, hashes, offsets, owners, groups, control flags and ACE counts matched a reference
parser, and every ACE's type, flags, mask, SID and object types matched `dissect.ntfs`. Every
recorded hash is its descriptor's, every mirror copy matches its entry, and every descriptor has
SDDL, including the conditions of its 3,384 callback ACEs.

## SRUM

[ys-srum-parser](https://github.com/Yamato-Security/ys-srum-parser) is based on Sootmark's
[ese](https://github.com/Sootmark/ese) and [srum](https://github.com/Sootmark/srum) crates. Across
11 real databases, every column in the 45,255 emitted entries matched an independent decoding of
the values exposed by `dissect.esedb`.

## User Access Logging

`parse-ese ual` reads UAL databases with the same ESE reader. Across 16 real databases (308 rows,
145 of them client rows), every value matched an independent decoding of the values exposed by
`dissect.esedb`. In every client row, the first day with a count was the UTC day of `InsertDate`,
the last one the UTC day of `LastAccess`, and the counts added up to `TotalAccesses`.

## Web browsers

[ys-browser-parser](https://github.com/Yamato-Security/ys-browser-parser) is a maintained fork of
Sootmark's browser, sqlite and plist crates, chosen after comparing it with another Rust library
and with Plaso, Velociraptor (SQLiteHunter) and Hindsight on the same databases. `parse-sqlite
browser` was compared with the `sqlite3` library reading a copy of each database with its `-wal`
file, on 38 public databases: Plaso's test files (Chrome 8 to 68, Firefox 2 to 118), profiles made
by Chromium 150 and Firefox 140, and ys-browser-parser's form history test databases. All 293
visits, 21 downloads, 692 cookies and 41 form entries matched. Deleted entries were recovered only
where entries had been deleted: the two pages and two visits deleted from the Firefox 140 history,
with their times, URLs and titles, and the deleted form entries of the test databases.

## Internet Explorer and legacy Edge (WebCache)

`parse-ese webcache` reads `WebCacheV01.dat` with
[ys-browser-parser](https://github.com/Yamato-Security/ys-browser-parser). It was compared with
`dissect.database`'s ESE reader on Plaso's two public WebCache databases: one of Internet
Explorer 11, and a dirty one of Windows 10 with legacy Edge's history, downloads and cookies. All
4,156 rows matched, value by value: 44 containers, 182 history entries, 6 downloads, 170 cookies
and 3,754 other entries. The comparison decoded the download records on its own. Records the
databases flag as deleted were left out of both.

## Outlook

`parse-file outlook` reads PST, OST, `.msg` and `.oft` files with
[ys-outlook-parser](https://github.com/Yamato-Security/ys-outlook-parser), whose PST and OST
reader is a fork of Microsoft's [outlook-pst-rs](https://github.com/microsoft/outlook-pst-rs) and
whose `.msg` reader is written from Microsoft's open specifications.

- **PST and OST**: 66 public PST and OST files (Apache Tika, artemis, the Aspose.Email examples,
  the DFIR Artifact Museum, Enron, the GroupDocs examples, java-libpst, libyal's test data,
  outlook-pst-rs and pst-extractor) were compared with libpff, property by property: every
  property of every store, folder, message, recipient and attachment, and every attachment's data.
  324,175 facts were the same and none differed without an explanation. The others are what only
  ys-outlook-parser reads (15,846 facts of messages, recipients and attachments libpff fails on,
  and of a store it cannot open; 8,223 of hidden items), recipient cells libpff gives as empty
  or zero though their rows say they are absent (49,077), folder entry ids and types computed as
  MAPI does (2,004), and objects and embedded messages libpff gives as their raw data (480). Two
  of the files neither reader reads at all.
- **`.msg` and `.oft`**: 214 public files, with 2,404 messages, recipients and attachments and
  41,002 properties, were compared with extract-msg: 110,114 facts were the same, and each of the
  791 that differ is explained: 601 are extract-msg's errors with times, 35 come from ANSI files it
  decodes in the wrong code page, and the others are what it leaves out or adds.
- **Damaged input**: more than 260,000 mutated `.msg` files and 160,000 mutated PSTs, and crafted
  files, were read without a crash, a hang or unbounded memory.

## LNK shortcuts

[ys-lnk-parser](https://github.com/Yamato-Security/ys-lnk-parser) is a maintained fork of
Sootmark's [shell](https://github.com/Sootmark/shell) crate. It was compared across 1,118 public
shortcuts. Of the 30 CSV columns shared with the reference parser, 16 individual values differed:
13 target paths involving damaged, truncated, placeholder, MTP, or Control Panel items; two
extra-data-block lists involving unknown or truncated blocks; and one MFT reference that the
reference parser read from different bytes. Juntei reports damaged structures instead of silently
normalizing them.

## Jump Lists

[ys-jump-lists-parser](https://github.com/Yamato-Security/ys-jump-lists-parser) uses Sootmark's
compound-file and Jump List work, with embedded links decoded by `ys-lnk-parser`. Across 77 public
Jump Lists, all 13,057 shared comparison cells—including timestamps to 100 ns—matched the
reference parser. Six files the reference refused (empty custom lists, a missing footer, or a file
that was not a Jump List) were either decoded or explicitly reported by Juntei.

## Recycle Bin

`parse-file recycle-bin` is Juntei's own parser. On the public samples of
[rifiuti2](https://github.com/abelcheung/rifiuti2), every record rifiuti2 reads had the same index,
deletion time (to the second, as rifiuti2 shows it), removal state, size and path: 92 records, 67 in
eleven INFO2 files from Windows 95 to Server 2003 and 25 in three folders of Windows Vista and
Windows 10 `$I` files. INFO2's ANSI paths matched in the code pages rifiuti2's own tests use. Juntei
also reads what rifiuti2 rejects: a folder mixing both `$I` versions, a `$I` header without a path,
and truncated records, each with its damage reported. The `$I` and INFO2 files of Plaso's test
data give the times, sizes, paths, indexes and drives Plaso's tests expect. On Digital Corpora's
public M57-Jean disk image (Windows XP), read with `auto-parse --e01`, the INFO2 record matches
rifiuti2, and the recycled file beside it, `Dc1.jpg`, has the name Juntei gives it.

## ShimCache

[ys-shimcache-parser](https://github.com/Yamato-Security/ys-shimcache-parser) is a maintained fork
of Security Ronin's
[shimcache-forensic](https://github.com/SecurityRonin/shimcache-forensic), with registry-hive and
transaction-log support. On 20 public `SYSTEM` hives (24 control sets and 7,290 entries) plus 17
standalone AppCompatCache values, every entry matched the path, time, execution flag, flags, and
position emitted by the reference parser.

## Microsoft Defender quarantine

[ys-defender-history-parser](https://github.com/Yamato-Security/ys-defender-history-parser) is a
maintained fork of Sootmark's
[defender](https://github.com/Sootmark/defender) crate. A quarantine folder written in Defender's
format by the parser's test tooling is decoded consistently by the independent
`dissect.target` quarantine reader.

## Registry-derived artifacts

[ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser) is compared with
[yarp](https://github.com/msuhanov/yarp) for allocated keys, values, dirty-hive transaction replay,
free-cell recovery, cell slack, data beyond hive bins, and records removed or replaced by replay.
Artifact-specific comparisons include:

- services, values, and failure actions from Windows 7 and dirty Windows 11 `SYSTEM` hives;
- RecentDocs decoded with yarp and libfwsi across nine public `NTUSER.DAT` hives;
- scheduled tasks decoded independently with yarp across seven public `SOFTWARE` hives covering
  Windows 7–11, Server 2012 R2, and Server 2022; and
- SAM account structures compared with `dissect.target` across seven public `SAM` hives.

See [Acknowledgements](../acknowledgements.md) for the complete list of reference tools, format
research, specifications, and public evidence sources.
