# Mounting Disk Images

`juntei mount` makes the files inside a VHDX / E01 / Ex01 disk image readable at a directory, read-only, for as long as it runs. Tools such as Hayabusa and Juntei can scan them without extracting files or requiring root. It uses FUSE and works on Linux. The reader and mounting library are maintained in [ys-vhdx-parser](https://github.com/Yamato-Security/ys-vhdx-parser).

```bash
mkdir /mnt/host1
juntei mount host1-triage.vhdx /mnt/host1
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
- Names are listed as NTFS stores them and are looked up exactly. DOS 8.3 names and alternate
  data streams are not shown.
- Compressed files are decompressed (up to 1 GiB each). EFS-encrypted files can't be read.

## The evidence is not changed

The image is opened read-only and the mount refuses writes. A VHDX that was not closed cleanly is read with its pending log applied in memory, never written back. Some DiscUtils writers leave the NTFS boot sector's jump instruction and `55 AA` signature zeroed; Juntei reads these volumes as they are.

## Usage

```text
Version: 1.1.0 (CODE BLUE Release)
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

## Parsing images natively with auto-parse { #parsing-every-image-with-auto-parse }

With `--vhdx`, `auto-parse` reads VHDX images natively on **Linux, macOS and Windows**, using
[ys-vhdx-parser](https://github.com/Yamato-Security/ys-vhdx-parser). No OS mount, FUSE driver,
or administrator privileges are needed.

```bash
# A directory of images, searched recursively
juntei auto-parse -d /cases/vhdx --vhdx -o parsed
# One image
juntei auto-parse -d host1.vhdx --vhdx -o parsed -j timeline.jsonl
```

The artifact parsers need filesystem paths. Juntei therefore copies the files they read from the
reader's NTFS volumes into a private staging workspace, then runs the usual parsers: registry
hives and their transaction logs, NTFS metadata files, databases, prefetch files, shortcuts, jump
lists, Outlook files, Recycle Bin index files and Defender quarantine folders. Every other file is an empty placeholder
with its original modification time, so directories and paths look as they do on the disk.
persistence-hunter reads a placeholder's content from the image only when it needs it, such as
an executable that a persistence entry refers to.
**Allow enough temporary disk space for the copied files**, in addition to the results.
Reads use bounded buffers; zero-filled regions are written as holes where the host filesystem
supports them. Original filenames, directory structure, hard links, and file modification times
are preserved. LNK and Jump List source creation/modification/access times come from a snapshot of the original NTFS metadata, never the temporary files. The workspace is removed on normal completion, including reported parser errors;
a forced termination can leave temporary files. The source image is opened read-only, and pending
VHDX log updates are applied in memory only.

Computer discovery, per-host output, CSV/JSON/JSONL, and `-j` use the same parsing pipeline as
ordinary evidence directories. `Image` in `manifest.json` identifies the original image and
`EvidenceRoot` identifies the relative path inside the prepared image tree. Unreadable images,
volumes, files, and filenames that cannot be represented safely are reported in `ImageErrors`
and failed rows of `manifest.csv`. Readable evidence is still parsed, and the process exits with
status 1 if any image input was incomplete. Unsafe or ambiguous names are rejected rather than
silently renamed. Named NTFS streams are not synthesized as extra files; already-collected
artifacts stored as ordinary files inside a triage image remain available.

The separate [`mount` command](mount.md) exposes a filesystem to other programs and still needs
Linux FUSE. It is not used by `auto-parse --vhdx`.

Staging uses a private directory on the output filesystem by default. Use `--temp-dir` to choose an existing staging directory on another disk. Original source metadata is kept separately on the output filesystem, so a full staging disk does not prevent parsing already-readable files. Reserve space for both staged files and results; if the output filesystem itself is full, results cannot be saved. Ctrl+C stops parser processes, cleans the private workspace and exits with code 130. A forced termination such as SIGKILL or power loss can still leave copies. Inaccessible subdirectories are skipped during image discovery; an unreadable input root remains an error.

The staging directory must be outside the output directory. On Linux and macOS, SIGTERM and SIGHUP also stop parser processes and clean up the workspace.

```bash
juntei auto-parse --vhdx -d evidence.vhdx -o parsed --temp-dir /large-disk/work
```

## E01 and Ex01 disk images

`--e01` reads physical EWF1 (`.E01`) and EWF2 (`.Ex01`) images natively on **Linux, macOS and Windows**, without FUSE, an OS mount, or administrator privileges. It uses [ys-e01-parser](https://github.com/Yamato-Security/ys-e01-parser) and the same NTFS parsing pipeline described above.

Provide the first segment of a split image. Keep all companions, including `.E02`, beside it with their original names. Both whole disks (MBR/GPT) and single NTFS volumes are supported. Use `--vhdx --e01` for a directory containing both formats.

The same temporary-space and source-metadata requirements apply. E01 images usually hold whole disks, and every image is staged before parsing starts, but only the files the parsers read are copied: usually a small part of the used space. Images are read-only. Acquisition gaps and damage found during opening or reading appear in `ImageErrors` and `manifest.csv`; readable results are retained with exit status 1. This checks accessed data, not every chunk or the image's full-media hash. Logical `.L01` and `.Lx01` files are rejected; there is no password option.

The separate `mount` command still requires Linux FUSE; native automatic parsing does not.

```bash
juntei auto-parse --e01 -d evidence -o parsed
juntei auto-parse --e01 -d disk.E01 -o parsed
juntei auto-parse --vhdx --e01 -d evidence -o parsed -j timeline.jsonl
```

## OS-visible mount requirements

`juntei mount` requires Linux FUSE 3 (`fusermount3`) and access to `/dev/fuse`. On Windows and macOS, use the native parsing command above.
