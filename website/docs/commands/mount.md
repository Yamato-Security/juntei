# Mounting Disk Images

`juntei mount` makes the files inside a disk image, a VHDX or an E01 image, readable at a directory, read-only, for as long as it runs. Tools such as Hayabusa and Juntei can scan them without extracting files or requiring root. It uses FUSE and works on Linux. The VHDX reader and the mounting library are maintained in [ys-vhdx-parser](https://github.com/Yamato-Security/ys-vhdx-parser), and the E01 reader in [ys-e01-parser](https://github.com/Yamato-Security/ys-e01-parser).

```bash
mkdir /mnt/host1
juntei mount host1-triage.vhdx /mnt/host1
```

An E01 image (EnCase's Expert Witness Format, also `.Ex01`) is mounted the same way. A split image is given by its first segment, and Juntei finds the others (`host1.E02`, `host1.E03`, …) next to it:

```bash
juntei mount host1.E01 /mnt/host1
```

In another terminal:

```bash
hayabusa dfir-timeline -d /mnt/host1 -o host1-hayabusa.csv
juntei auto-parse -d /mnt/host1 -o host1-juntei
```

Press Ctrl+C in the first terminal to unmount, or run `fusermount3 -u /mnt/host1`.

## What the mount shows

- The NTFS volume's files appear at the mount point. A collected image can contain directories such as `C` and `LongFileNames`, along with volume metadata such as `$MFT`. Multiple NTFS volumes appear as `Volume1`, `Volume2`, and so on.
- Every file is read-only and belongs to the owner of the mount point. File times are the NTFS
  `$STANDARD_INFORMATION` times.
- An image of a single volume, without a partition table, is that volume.
- Names are listed as NTFS stores them and are looked up exactly. DOS 8.3 names and alternate
  data streams are not shown.
- Compressed files are decompressed (up to 1 GiB each). EFS-encrypted files can't be read.

## The evidence is not changed

The image is opened read-only and the mount refuses writes. A VHDX that was not closed cleanly is read with its pending log applied in memory, never written back. Some DiscUtils writers leave the NTFS boot sector's jump instruction and `55 AA` signature zeroed; Juntei reads these volumes as they are.

An E01 image's segments are opened read-only too. Each chunk is checked against its checksum when it is read: a damaged chunk can't be read, and the rest of the image can. Sectors that the acquisition could not read, and an acquisition that did not finish, are reported as warnings. A logical evidence file (`.L01`, `.Lx01`) holds files, not a disk, and is not mounted.

## Usage

```text
Version: 1.1.0 (Dev Build)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei mount <IMAGE> <MOUNTPOINT> [OPTIONS]

Input:
  <IMAGE>       The disk image (VHDX or E01; a split E01 image from its first segment)
  <MOUNTPOINT>  An existing directory to mount its NTFS volumes at, read-only, until Ctrl+C

General Options:
  -h, --help  Show the help menu

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

## Parsing every image with auto-parse

`auto-parse --vhdx` parses the Windows systems inside every VHDX disk image in the evidence
directory (or the one image given with `-d`), and `auto-parse --e01` those inside every E01 image,
without a separate mount. The two options can be given together:

```bash
juntei auto-parse -d /cases/vhdx --vhdx -o parsed
juntei auto-parse -d /cases/e01 --e01 -o parsed
```

- Each image is mounted read-only in a new temporary directory, in a folder named after the image
  (`host1-triage`), and the systems found there are parsed as in a collection on disk. The images
  are unmounted at the end, and on Ctrl+C.
- The terminal lists the images and the image of each system:
  `HOST1: ./host1-triage/C (./host1-triage.vhdx)`.
- `--e01` takes the first segment of each E01 image (`*.E01`, `*.Ex01`); the other segments are
  read with it.
- In the manifest, `EvidenceDirectory` is the directory given and each system's `Image` names its
  image. `EvidenceRoot` and the parser commands use the mount folder (`./host1-triage/C`).
- If the image contains files from Volume Shadow Copies in folders such as `VSS32`, each folder is found and parsed as another system (`HOST1_2`, …).
- An image that can't be mounted is reported; the others are still parsed, and the run ends with
  exit status 1.

If an NTFS volume cannot be opened, Juntei reports it and keeps the readable volumes. Partial images cause `auto-parse --vhdx` and `--e01` to return exit status 1; `mount` also returns 1 after unmounting.

## Requirements

- Linux: FUSE 3 (`fusermount3`, the `fuse3` package) and access to `/dev/fuse`, which normal users
  usually have. Root is not needed.
- Windows and macOS are not supported. On Windows, attach a VHDX image read-only in an elevated
  PowerShell instead, and give the tools its drive letter. Windows can't attach an E01 image:
  mount it with a tool that can, read-only, and give the tools that drive.

```powershell
Mount-DiskImage -ImagePath D:\cases\host1-triage.vhdx -Access ReadOnly
juntei auto-parse -d E:\ -o host1-juntei
Dismount-DiskImage -ImagePath D:\cases\host1-triage.vhdx
```
