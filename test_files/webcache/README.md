# WebCache test databases

Kept gzipped; the tests decompress them.

| File | Source | Rows (`parse-ese webcache`) |
| --- | --- | --- |
| `plaso/WebCacheV01.dat.gz` | [Plaso](https://github.com/log2timeline/plaso)'s `test_data/WebCacheV01.dat` (f75784e), compressed with gzip: Internet Explorer 11's database of user `test`, clean | 113 history entries, 2 downloads, 1,434 other entries, 21 containers |
| `plaso/PartitionsEx-WebCacheV01.dat.gz` | The same: `test_data/PartitionsEx-WebCacheV01.dat`, a dirty Windows 10 database of user `John Doe` with legacy Edge's history, downloads and cookies (`PartitionsEx`, `CookieEntryEx` tables) | 69 history entries, 4 downloads, 170 cookies, 2,320 other entries, 23 containers |

Both are Apache-2.0 (`plaso/LICENSE`). Every value `parse-ese webcache` writes for them, 4,156 rows,
equals a reading with dissect.database's ESE reader, the download records decoded independently.
