# Windows Search index test databases

Kept gzipped; the tests decompress them. Every file is unmodified and carries its license beside it.

| File | Source | Windows | What `parse-ese windows-search` reads |
| --- | --- | --- | --- |
| `plaso/Windows.edb.gz` | [plaso](https://github.com/log2timeline/plaso)'s `test_data/Windows.edb` (added in baf54908), Apache-2.0 (`plaso/LICENSE`) | Windows 7 (`SystemIndex_0A` schema, ESE format revision 17, dirty shutdown; host `STUDENT-PC1`) | 339 items: 322 with properties, 17 known to the gatherer only; 81 summaries in the Windows 7 string encoding |
| `museum/Win10-Windows.edb.gz` | [DFIRArtifactMuseum](https://github.com/AndrewRathbun/DFIRArtifactMuseum)'s `Windows/WindowsSearchDB/Win10/APTSimulatorVM/Dirty/Windows.edb` (tree b26e860), MIT (`museum/LICENSE`) | Windows 10 (`SystemIndex_PropertyStore`, revision 110, dirty shutdown; host `DESKTOP-TMKU40H`) | 935 items: 930 with properties (230 of them in leaf pages that only the sibling chain reaches), 5 gatherer only |
| `museum/Win11-Windows.edb.gz` | The same repository's `Windows/WindowsSearchDB/Win11/RathbunVM/Windows.edb`, MIT | Windows 11 before 22H2 (`SystemIndex_PropertyStore`, ESE format revision 200, dirty shutdown; host `GOLDPAPERW11`) | 855 items: 825 with properties, 27 gatherer only, 3 property store only |
| `museum/Win11-22H2-Windows.db.gz`, `museum/Win11-22H2-Windows-gather.db.gz` | The same repository's `Windows/WindowsSearchDB/Win11/RathbunVM2/preBrowsing/`, MIT | Windows 11 22H2 and later (SQLite) | Nothing: reported as the unsupported newer format |

SHA-256 of the uncompressed files:

```
e96837a0ad37be5a55222fa98991bec6b5c5ba956610f9d241aa8c65f2e0714c  plaso/Windows.edb
ff024095f2880817e64e0413d8b5760965bd8e5247ad163045426354aa589b5b  museum/Win10-Windows.edb
35ca37a022869e9311defe4211253948b68f55599f79258c16390ee4aa14dbf9  museum/Win11-Windows.edb
97eb1c6176cc039eea42148f5a436fbfd02e0b3124f96a72c6ad760c23219f5d  museum/Win11-22H2-Windows.db
7805ba163294f81118d818f8e63f7e7705ee7c23e6a18c05716bed03d6e5653a  museum/Win11-22H2-Windows-gather.db
```

Independent expected results, checked by `tests/windows_search_cli.rs`:

- libesedb's `esedbexport` (20240420), which identifies all three ESE files as Windows Search
  databases and decodes the Windows 7 strings and times: 322, 930 and 828 property-store rows, 339,
  935 and 852 gatherer rows. Every path, owner, type, computer name, size and time of the Windows 7
  and Windows 11 property stores compared equal (6,691 Windows 11 values; the Windows 7 summaries
  including the run-length and 8-bit encodings).
- Strozfriedberg's SIDR (0.9.2) for the Windows 10 database: 6,332 path, time, type, owner, size,
  summary and computer-name values of its 930 file records compared equal. SIDR's own ESE library
  cannot open the revision-200 Windows 11 file and reads no Windows 7 table.

Two values differ from those readings on purpose:

- `Priority` (and `TransactionExtendedFlags` on Windows 8 and later) is empty for most gatherer
  rows: ESE marks the columns NULL in the record's fixed-column NULL bitmap, which Juntei's ESE
  reader honours as ESE itself does (`ErrRECIFixedColumnInRecord` in Microsoft's published
  source). `esedbexport` and SIDR read the fixed columns without the bitmap and print the bytes
  still in the record, the priority of the item's last crawl (45 for the Windows 7 item 3).
- The Windows 7 items 149 and 314, the roots of two users' browser history, hold one FILETIME tick
  in their four time properties. `esedbexport` prints them as `Jan 01, 1601 00:00:00.000000100`;
  Juntei keeps the number and no time, with a note in `Problems`.
