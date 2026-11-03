# `$Secure:$SDS` fixture

`SDS.gz` is the DFIR Artifact Museum's Windows 11 `$Secure:$SDS` (1,473,492 bytes), stored
gzip-compressed and otherwise unmodified: MIT License, provenance and SHA-256 in `NOTICE`, the
license in `LICENSE`. The same stream is a test fixture of the secure crate of
[ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser).

It holds 1,716 security descriptors (security IDs 256 to 1,971), every one with its hash and its
mirror copy intact. 789 of them have callback ACEs with app container conditions
(`WIN://SYSAPPID`), 399 have process trust labels, and two have resource attribute claims.

`tests/ntfs_metadata_cli.rs` decompresses it into temporary directories. No private evidence is
committed.
