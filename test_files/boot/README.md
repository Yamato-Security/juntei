# `$Boot` fixture

`Boot` is the DFIR Artifact Museum's Windows 11 `$Boot` (8,192 bytes), unmodified: MIT License,
provenance and SHA-256 in `NOTICE`, the license in `LICENSE`. The same file is a test fixture of
the boot crate of [ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser).

| Field | Value |
| --- | --- |
| Volume serial number | `180B1E67` (`EA180B53180B1E67`), as the same host's LNK file records it |
| Geometry | 512-byte sectors, 4,096-byte clusters, 124,370,943 sectors (63,677,922,816 bytes), 239,616 hidden sectors |
| `$MFT`, `$MFTMirr` | clusters 786,432 and 2; FILE records of 1,024 bytes, INDX records of 4,096 |
| Boot code SHA-256 | `2354794a213560f40d3eb095ba35808108faf2c27c1bd849839921e469d5f13a`, the same as the Museum's Windows 10 and Server 2012 R2 `$Boot` files |

`tests/ntfs_metadata_cli.rs` copies it under the names collections give it. No private evidence is
committed.
