# RDP Bitmap Cache

## What the RDP bitmap cache records

When the Remote Desktop client (`mstsc.exe`) connects with persistent bitmap caching on, which is
the default, it saves pieces of the remote screen to disk so that later sessions don't have to
receive them again. Each piece is a tile of at most 64 × 64 pixels. Together the tiles are small
fragments of what the user saw on the remote computers: windows, text, file names, commands and
desktops.

| Client | Cache files | Tiles |
|---|---|---|
| Windows 7 and later | `Cache0000.bin`, `Cache0001.bin`, …, up to 100 MB each | 32-bit, one after another |
| Windows XP, Server 2003 and earlier | `bcache2.bmc` (8 bits per pixel), `bcache22.bmc` (16), `bcache23.bmc` (24) and `bcache24.bmc` (32), up to 10 MB each | One per 64 × 64 slot; some compressed |

Windows 10 and later also leave an empty `bcache24.bmc` beside the `.bin` files.

<span class="evidence-strength">Best conclusion:</span> the user whose profile holds the cache
connected with the Remote Desktop client to a computer that showed the cached content. The cache
doesn't say which computer it was, or when.

## Locations and collection

```text
%LOCALAPPDATA%\Microsoft\Terminal Server Client\Cache\Cache????.bin    Windows 7 and later
%LOCALAPPDATA%\Microsoft\Terminal Server Client\Cache\bcache24.bmc     (empty on Windows 10 and later)
%USERPROFILE%\Local Settings\Application Data\Microsoft\Terminal Server Client\Cache\bcache*.bmc    Windows XP and earlier
```

The `BitmapPersistCacheLocation` value of `HKCU\Software\Microsoft\Terminal Server Client` can
move the cache. Collect the whole `Cache` folder of every profile, also from volume shadow copies:
an older copy of a cache file can hold tiles the current one no longer has.

## Analyst cautions

- Tiles have no times, and the cache doesn't name the computers they came from. Relate them to
  RDP connections from other evidence: the client's `Terminal Server Client` registry keys
  (`parse-registry remote-access`), `Default.rdp` and other `.rdp` files, the Jump Lists of
  `mstsc.exe`, the cache files' times in `$MFT`, and the remote computers' logon events.
- A cache file mixes tiles of many sessions and of several remote computers. The client replaces
  old tiles as a file fills, so a tile can be days or months old.
- Tiles are pieces of screens, not whole screens. Tiles that were next to each other on the screen
  are often next to each other in the file, but not always. Rebuild screens with
  [RdpCacheStitcher](https://github.com/BSI-Bund/RdpCacheStitcher), or by eye from the collages.
- Most tiles are 64 × 64 pixels. Tiles at a screen's right and bottom edges are narrower or
  shorter, depending on the resolution: 58 or 62 pixels wide, or 28 or 35 high, for example.
  Juntei reads every tile at its own size. Tools that read every tile as 64 pixels wide show the
  narrow ones skewed.
- Most single-color tiles (`SingleColor`) are background. Filter them out to find content.
- In `.bmc` files, 8-bit tiles are colors of the session's palette, which the cache doesn't keep.
  Juntei uses the Windows default palette, so their colors can be wrong. Compressed tiles use the
  Remote Desktop protocol's interleaved RLE (MS-RDPBCGR 3.1.9), which Juntei decompresses as the
  specification defines it.
- A `.bmc` tile smaller than its 64 × 64 slot leaves the end of an earlier tile after it. Juntei
  saves those leftover rows as a separate image (`RemnantImage`). They belong to an older tile,
  not to the tile in the slot.

## Parse with Juntei

```text
Usage: juntei parse-file rdp-cache <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find Cache????.bin and bcache*.bmc files
  -f, --file <FILE>      Parse one cache file, whatever its name

Output:
  -o, --output <FILE>       Save one record per tile
  -t, --output-type <TYPE>  csv, json, or jsonl
  -x, --extract <DIR>       Save each tile as a BMP and a PNG collage of each cache file
      --collage-only        With --extract, save only the collages
      --collage-width <N>   Tiles per collage row (default 64)
```

```bash
juntei parse-file rdp-cache -d evidence/ -o rdp-cache.csv --extract rdp-cache/
juntei parse-file rdp-cache -f Cache0000.bin --extract rdp-cache/ --collage-only
juntei parse-file rdp-cache -d evidence/ -t jsonl | jq 'select(.SingleColor == false)'
```

With `-d`, Juntei reads files named `Cache`, digits and `.bin`, and `bcache`, digits and `.bmc`.
A `Cache????.bin` file that doesn't start with the `RDP8bmp` signature belongs to another program:
it is counted in the summary and skipped. With `-f`, Juntei reads the file whatever its name.

### Images

`--extract` creates a folder for each cache file, `<DIR>/<user>/<cache file>/`. When two cache
files would share one, such as a cache file and its copy from a volume shadow copy, the second
folder is numbered (`Cache0001.bin_2`). The folder holds:

- `<cache file>_<index>.bmp`: each tile, at its own size. The names are the ones bmc-tools uses,
  so RdpCacheStitcher can open the folder.
- `<cache file>_old_<index>.bmp`: `.bmc` only, the end of an earlier tile left after a smaller one.
- `<cache file>_collage.png`: every tile in file order, 64 to a row (`--collage-width`), each in a
  64 × 64 cell from its top left. The tile with index *i* is in row *i* / 64 and column *i* % 64,
  counted from 0. Magenta fills the cells' empty parts: beside a narrow tile, below a short one,
  and where a tile could not be read.

`--collage-only` saves only the collages. An image that already exists is not replaced without
`-C`, and its tile's `Problems` says so. The cache files themselves are never replaced, even with
`-C`.

## CSV output fields

| Field | Meaning |
|---|---|
| `SourceFile` | The cache file. |
| `User` | The user whose profile folder the cache file is in, from its path. |
| `TileIndex` | The tile's number in the file, counted from 0, as in its image's name. |
| `Width`, `Height` | The tile's size in pixels. |
| `SingleColor` | `true` when every pixel of the tile has the same color. |
| `TileImage` | With `--extract`: the tile's BMP file. |
| `Collage` | With `--extract`: the cache file's collage. |
| `RemnantImage` | `.bmc` with `--extract`: the end of an earlier tile left in the slot. |
| `Key` | The tile's 64-bit key in the Remote Desktop protocol, in hexadecimal. |
| `Format`, `FormatVersion` | `bin` with the version after its signature (6 on Windows 10 and later), or `bmc`. |
| `BitsPerPixel` | 32 for `.bin`; 8, 16, 24 or 32 for `.bmc`. |
| `Compressed`, `Flags` | `.bmc`: whether the tile was compressed, and its header's flags. |
| `Offset` | Where the tile's header starts in the file. |
| `Problems` | Damage: a file that ends inside a tile, a tile header that no tile could have, data that doesn't fit the tile's size, compressed data that can't be decompressed, an image that could not be saved. |

JSON and JSONL have the same fields, with numbers and booleans typed, `null` for values a tile
doesn't have, and `Problems` as a list. Damage outside any one tile, such as data after the last
tile, is reported in the run's messages.

## Automatic parsing

`auto-parse` writes `LateralMovement/RdpCache.csv` (`RdpCache.json` or `RdpCache.jsonl` with
`-t json` or `-t jsonl`), with `LateralMovement/RdpCache-logs.txt`, and saves each cache file's
collage in a `LateralMovement/RdpCache/` folder. Add `--extract-rdp-tiles` to also save every
tile, which can be tens of thousands of files for one user. From disk images, the cache files are
copied. The combined `-j` timeline includes every row with the `Artifact` identifier `rdp-cache`.
The tiles have no times, so [`super-timeline`](../commands/super-timeline.md) makes no events from
them.

## Investigation strategy

1. Open the collages and look for text: computer and user names, file paths, commands, console
   windows, browser address bars and dialog boxes.
2. Rebuild screens with RdpCacheStitcher from a folder of tiles saved with `--extract`.
3. Find the computers the user connected to, from the client's registry (`parse-registry
   remote-access`), `.rdp` files and the Jump Lists of `mstsc.exe`. On those computers, look for the
   matching RDP logons and User Access Logging records.
4. Use the cache files' `$MFT` times, and those of their copies in volume shadow copies, to narrow
   down when tiles were written.

## Further Details

- [ANSSI (CERT-FR) — CERTFR-2016-ACT-017: the RDP bitmap cache (in French)](https://www.cert.ssi.gouv.fr/actualite/CERTFR-2016-ACT-017/)
- [ANSSI — bmc-tools](https://github.com/ANSSI-FR/bmc-tools)
- [BSI — RdpCacheStitcher](https://github.com/BSI-Bund/RdpCacheStitcher)
- [Velociraptor — Windows.Forensics.RDPCache](https://github.com/Velocidex/velociraptor/blob/master/artifacts/definitions/Windows/Forensics/RDPCache.yaml)
- [dissect.target — RDP cache plugin](https://github.com/fox-it/dissect.target/blob/main/dissect/target/plugins/os/windows/rdpcache.py)
- [Microsoft — MS-RDPBCGR 3.1.9: Interleaved RLE-Based Bitmap Compression](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-rdpbcgr/b6a3f5c2-0804-4c10-9d25-a321720fd23e)
- [FreeRDP — persistent bitmap cache](https://github.com/FreeRDP/FreeRDP/blob/master/libfreerdp/cache/persistent.c)
