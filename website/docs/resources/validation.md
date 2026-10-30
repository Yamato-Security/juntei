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

[ys-mft-parser](https://github.com/Yamato-Security/ys-mft-parser) is a maintained fork of Omer
Ben-Amram's [mft](https://github.com/omerbenamram/mft) crate. Across eight `$MFT` files—the crate's
sample, the NIST hacking case, and six DFIR Artifact Museum files—1,048,733 CSV rows and columns
matched a reference parser in order. Comparisons covered default output, `--all-times`,
`--short-names`, and `--cutoff`; timestamp presentation was the intentional difference.

## NTFS transaction log ($LogFile)

[ys-logfile-parser](https://github.com/Yamato-Security/ys-logfile-parser) is maintained from
[Sootmark ntfslog](https://github.com/Sootmark/ntfslog). Its validation covers six complete DFIR
Artifact Museum logs, public version 1.1 and 2.0 fixtures, decoded redo/undo metadata, and journal
writes compared against independently parsed NTFS data. See the parser repository for detailed
coverage and known differences.

Juntei adds CLI tests that compare every CSV field with JSON and JSONL on the public go-ntfs test
volume (2,129 records), verify separate redo/undo data and raw payload lengths, test damaged logs,
refuse evidence overwrites, and keep current MFT paths from following reused record numbers.
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

[ys-usn-journal-parser](https://github.com/Yamato-Security/ys-usn-journal-parser) is a maintained
fork of [Sootmark usn](https://github.com/Sootmark/usn). On six real journals, Juntei's
time-aware parent paths matched an independent parser wherever that parser resolved a path. Juntei
also resolved 4,321 records that the comparison left unresolved. Its carver produced no false
records while tested against 880 MB of `$MFT` data.

## Directory indexes ($I30)

The `i30` module of [ys-mft-parser](https://github.com/Yamato-Security/ys-mft-parser) reads INDX
records and their slack. On two `$I30` streams of a volume built with ntfs-3g (Juntei's
`test_files/i30/`) and on three public Windows-written `$I30` samples, its entries in use and in
slack matched those of Maxim Suhanov's [dfir_ntfs](https://github.com/msuhanov/dfir_ntfs), with the
same names and times. Where dfir_ntfs reports a parent reference past 32 bits (bytes written over
the parent), Juntei leaves the parent empty.

## SRUM

[ys-srum-parser](https://github.com/Yamato-Security/ys-srum-parser) is based on Sootmark's
[ese](https://github.com/Sootmark/ese) and [srum](https://github.com/Sootmark/srum) crates. Across
11 real databases, every column in the 45,255 emitted entries matched an independent decoding of
the values exposed by `dissect.esedb`.

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
