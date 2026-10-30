# NTFS transaction-log fixtures

- `go-ntfs/`: the version 1.1 `$LogFile` and matching `$MFT` exported from Velocidex's public
  NTFS test volume. Apache-2.0; provenance, uncompressed SHA-256 hashes, and the license are
  preserved in `NOTICE` and `LICENSE`. The command exports 2,129 records, including 17 checkpoints
  and 79 embedded redo/undo filename entries.
- `museum-win11/`: the first 1 MiB of the DFIR Artifact Museum's Windows 11 RathbunVM log
  (version 2.0). MIT; provenance and uncompressed hash are in `NOTICE`, with `LICENSE` alongside.
  Exercises journal writes and recovery with a damaged restart-page copy.

Copied without modification from ys-logfile-parser at
`d825dd28af9c3b3f2f6580fdfb29a634196bc18d`. `tests/logfile_cli.rs` decompresses them into temporary
directories. No private investigation evidence is committed.
