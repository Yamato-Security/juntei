# Recycle Bin test files

Openly licensed Recycle Bin index files from other projects' test data for `parse-file recycle-bin`.
The files are unchanged. `tests/cli.rs` reads them, and they were compared with
[rifiuti2](https://github.com/abelcheung/rifiuti2) and with the values Plaso's tests expect.

## rifiuti2

From [abelcheung/rifiuti2](https://github.com/abelcheung/rifiuti2) `test/samples` (`40f185b`).

| File | What it is | Rows | Code page |
| --- | --- | --- | --- |
| `INFO-95-ja-1` | Windows 95 `INFO` (version 0, 280-byte records): Japanese ANSI paths on drive D: | 11 | 932 |
| `INFO-NT-en-1` | Windows NT 4.0 INFO2 (version 2, 800-byte records) | 6 | 1252 |
| `INFO2-sample2` | Windows 98 INFO2 (version 4, 280-byte records), 2 items removed from the bin | 7 | 1252 |
| `INFO2-ME-en-1` | Windows ME INFO2 (version 5, 280-byte records), 3 items removed from the bin | 6 | 1252 |
| `INFO2-me-en-uncpath` | Windows ME INFO2 with UNC paths (drive number 26) | 3 | 1252 |
| `INFO2-2k-cht-1` | Windows 2000 INFO2 with Traditional Chinese paths: 8.3 ANSI paths and Unicode paths, a 258-character path | 5 | 950 |
| `INFO2-2k-tw-uncpath` | Windows 2000 INFO2 with UNC paths | 4 | 950 |
| `INFO2-03-tw-uncpath` | Windows XP or Server 2003 INFO2 with UNC paths | 4 | 950 |
| `INFO2-sample1` | Windows XP or Server 2003 INFO2 with Simplified Chinese paths, 1 item removed from the bin | 16 | 936 |
| `INFO2-trunc` | Windows 2000 INFO2 whose last record is 5 bytes short, with a deletion time in the year 3000 | 5 | 1252 |
| `INFO2-empty` | INFO2 of an empty bin: the 20-byte header | 0 | |
| `dir-sample1/` | Windows Vista to 8.1 `$I` files (version 1) and two `$R` files; two `$I` files are 543 bytes long | 15 | |
| `dir-win10-01/` | Windows 10 `$I` files (version 2) with three `$R` files; a 246-character path, characters outside the BMP and a 12 GiB size | 7 | |
| `dir-2019-uncpath/` | Windows 10 or later `$I` files (version 2) with UNC paths | 3 | |
| `dir-mixed/` | A version 1 and a version 2 `$I` file in one folder | 2 | |
| `dir-badfiles/` | Damaged version 2 `$I` files: a header without a path, a truncated path, a deletion time in 1990 | 4 | |
| `dir-bad-uni/` | Version 1 `$I` files whose paths contain unpaired UTF-16 surrogates | 4 | |

Licence: BSD-3-Clause (`rifiuti2/LICENSE`).

## Plaso

From [log2timeline/plaso](https://github.com/log2timeline/plaso) `test_data/recycler` (`f75784e`).

| File | What it is | Rows |
| --- | --- | --- |
| `$II3DF3L.zip` | Windows Vista `$I` file (version 1) | 1 |
| `$I103S5F.jpg` | Windows 10 `$I` file (version 2) | 1 |
| `INFO2` | Windows 2000 to Server 2003 INFO2 (version 5, 800-byte records) | 4 |

Licence: Apache-2.0 (`plaso/LICENSE`).
