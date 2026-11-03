# Boot Sector (`$Boot`)

## What `$Boot` is

`$Boot` is MFT entry 7 of every NTFS volume: the volume's first 8 KiB. Its first sector, the
**volume boot record**, holds the BIOS parameter block that describes the file system:

- the sector and cluster sizes, the number of sectors in the volume, and the **hidden sectors**
  (the sectors on the disk before the volume, that is where its partition starts);
- where the `$MFT` and `$MFTMirr` start, and the sizes of FILE and INDX records;
- the 64-bit **volume serial number**, set when the volume is formatted.

The rest of the first sector and the 15 sectors after it hold the **boot code** that loads
Windows.

The low 32 bits of the serial number are what Windows shows as `XXXX-XXXX` (`vol`, `dir`) and what
[LNK shortcuts](lnk.md), [Jump Lists](jump-lists.md) and [Prefetch](prefetch.md) record of the
volume a file was on. `$Boot` ties those records to a volume of a collected host.

<span class="evidence-strength">Best conclusion:</span> a LNK, Jump List or Prefetch volume serial
number equal to a `$Boot`'s `VolumeSerialNumber` points to that volume, or to another volume that
was given, or copied with, the same serial number. Serial numbers can be changed and are not
unique across hosts.

## Location and collection

```text
C:\$Boot          (MFT entry 7; the first 8 KiB of the volume)
```

Collect it with a raw-aware forensic tool, or read the first sector of a volume image: both are
the same bytes. Juntei reads any file that starts with an NTFS boot sector, so a volume image works
as input. A disk image starts with the partition table instead; give Juntei the partition's volume.

## Analyst cautions

- The serial number changes when the volume is formatted. A LNK file whose serial number matches
  no current volume may point to removable media, a network share's server, or a volume formatted
  since.
- The boot code is the same on volumes formatted by the same Windows: it is identical on the
  Windows 10, 11 and Server 2012 R2 `$Boot` files of the DFIR Artifact Museum. A volume boot record
  bootkit changes it, or the hidden sectors field, which the boot code uses to find the rest of
  `$Boot`. Compare `BootCodeSHA256`, `JumpInstruction` and `HiddenSectors` with other hosts and with the partition
  table before concluding anything: a boot manager or encryption software can also change the boot
  code.
- An image of a BitLocker-encrypted volume starts with BitLocker's own boot sector (OEM ID
  `-FVE-FS-`). Given with `-f`, Juntei reports it, and `Problems` says it is not NTFS; the NTFS boot
  sector is in the encrypted data. A `$Boot` collected from the unlocked volume is the NTFS one.
- `Problems` lists recognized structural inconsistencies and truncation, rather than providing a complete Windows mountability test. Use it to assess carved sectors and copies, not as an attack finding by itself. The boot-code hash excludes the initial jump, BIOS parameter block and end signature; inspect those fields too.

## Parse with Juntei

```text
Usage: juntei parse-ntfs boot <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find $Boot files (named $Boot, or starting with an NTFS boot sector)
  -f, --file <FILE>      Parse one $Boot file, or an NTFS volume image (its first sector)

Output:
  -o, --output <FILE>       Save every boot sector
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-ntfs boot -f '$Boot'
juntei parse-ntfs boot -d evidence/ -o boot.csv
juntei parse-ntfs boot -f volume.dd -t json
```

The summary lists each boot sector's volume serial number (as Windows shows it, and as LNK and
Prefetch fields show it), size, cluster size and `$MFT` cluster. When several hosts are parsed, it
groups them by boot code hash, so that a volume whose boot code differs from the others stands out.

## CSV output fields

| Field | Interpretation |
|---|---|
| `SourcePath` | Input file. |
| `VolumeSerialNumber` | The 32-bit serial number, as LNK, Jump List and Prefetch output show it (`180B1E67`). Windows shows it as `180B-1E67`. |
| `VolumeSerialNumber64` | The whole 64-bit serial number. |
| `OEM-ID` | `NTFS` on an NTFS volume. |
| `BytesPerSector`, `SectorsPerCluster`, `ClusterSize` | The volume's geometry. Sectors per cluster over 128 are recorded as a power of two, decoded here. Empty when not valid. |
| `TotalSectors`, `VolumeSize` | Sectors in the volume and their size in bytes, without the copy of the boot sector at its end. |
| `HiddenSectors` | Sectors on the disk before the volume: where its partition starts. |
| `MFT-Cluster`, `MFT-Offset` | Where the `$MFT` starts, in clusters and in bytes from the start of the volume. |
| `MFT-MirrorCluster`, `MFT-MirrorOffset` | Where the `$MFTMirr` starts. |
| `FileRecordSize`, `IndexRecordSize` | Bytes per FILE record (usually 1,024) and per INDX record (usually 4,096). |
| `MediaDescriptor` | `F8` for a hard disk. |
| `JumpInstruction` | The first three bytes (`EB 52 90`): a jump over the BIOS parameter block to the boot code. |
| `EndSignature` | `55 AA` at the end of a valid boot sector. |
| `BootCodeSHA256` | SHA-256 of the boot code: the first sector's bytes 0x54 to 0x1FD and the rest of the 8 KiB. |
| `Problems` | What Windows would not mount: an OEM ID other than `NTFS`, no `55 AA`, invalid sizes, FAT fields NTFS keeps at zero set, the `$MFT` past the end of the volume; and a file shorter than 8 KiB. |

JSON/JSONL have the same fields and names, with numbers as numbers, `null` for unknown values and
`Problems` as a list.

## Investigation strategy

1. Put each host's `VolumeSerialNumber` next to the volume serial numbers of its LNK files, Jump
   Lists and Prefetch files: the ones that match no collected volume are other volumes (USB
   drives, shares, volumes formatted since).
2. Look the other serial numbers up in the [USB devices](usb.md) output, which lists the volume
   serial numbers of mounted removable volumes.
3. Compare `BootCodeSHA256` and `HiddenSectors` across hosts. Investigate a differing boot code
   with a disassembler and the disk's partition table, and look for boot-related event log entries.
4. Use `MFT-Offset` and `ClusterSize` to find the `$MFT` in a volume image.

## Further Details

- [Microsoft: NTFS overview](https://learn.microsoft.com/en-us/windows-server/storage/file-server/ntfs-overview)
- [libfsntfs: NTFS format documentation](https://github.com/libyal/libfsntfs/blob/main/documentation/New%20Technologies%20File%20System%20(NTFS).asciidoc) (the volume header)
- [The Sleuth Kit: fsstat](https://www.sleuthkit.org/sleuthkit/man/fsstat.html)
- [ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) (its `boot` crate)
