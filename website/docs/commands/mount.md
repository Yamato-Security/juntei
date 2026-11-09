# Mounting Disk Images

`juntei mount` makes the files inside a VHDX disk image readable at a directory, read-only, for as long as it runs. Tools such as Hayabusa and Juntei can scan them without extracting files or requiring root. It uses FUSE and works on Linux. The reader and mounting library are maintained in [ys-vhdx-parser](https://github.com/Yamato-Security/ys-vhdx-parser).

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
Version: 1.1.0 (Dev Build)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei mount <IMAGE> <MOUNTPOINT> [OPTIONS]

Input:
  <IMAGE>       The disk image (VHDX)
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

The artifact parsers need filesystem paths. Juntei therefore streams the files exposed by the
reader's NTFS volumes into a private system-temporary workspace, then runs the usual parsers.
**Allow enough temporary disk space for those files**, in addition to the results. This is a
copy of the exposed filesystem, not just the artifacts that eventually produce records.
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

## OS-visible mount requirements

`juntei mount` requires Linux FUSE 3 (`fusermount3`) and access to `/dev/fuse`. On Windows and macOS, use the native parsing command above.
