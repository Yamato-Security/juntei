# Event log carving fixtures

## `evtx/`

Unmodified test samples of [Omer Ben-Amram's evtx crate](https://github.com/omerbenamram/evtx)
(`samples/`), distributed under the MIT or Apache-2.0 license (`evtx/LICENSE-MIT`,
`evtx/LICENSE-APACHE`). Each is one 64 KiB chunk after the 4 KiB file header.

| File | Channel | What it exercises | SHA-256 |
| --- | --- | --- | --- |
| `new-user-security.evtx` | Security | allocated records and old records in the chunk's slack | `6f9fe51d0a5dec63dc68f819e23bc14da5db0b75db4a68dfdf6b39f02868a379` |
| `Microsoft-Windows-HelloForBusiness%4Operational.evtx` | Microsoft-Windows-HelloForBusiness/Operational | a records checksum that does not match | `f03dd7f04d728a118d33ee3e5f3900912a9e3cb6e26e13491ec754c47e1d29d2` |
| `Microsoft-Windows-WorkFolders%4WHC.evtx` | (empty page) | an empty page holding two old records | `d9595557c0b9172703989753b93ab182550457880b56ac539d3a3d2fd8a0fda3` |
| `Application_no_crc32.evtx` | Application | header and records checksums left at zero | `2b128dc61812123a34ab2808502c6d48a0db6018e6b21bfb4f0be12d6dad2596` |

The tests also use `../persistence/task-creation.evtx` (CC0 1.0; see its README).

## `sample.raw.gz`

A raw 16 MiB disk (an MBR with one NTFS volume at sector 63, 4 KiB clusters) made by
`make_sample.py` from the logs above with mkntfs and ntfs-3g. It holds two live logs in an
NTFS-compressed folder, two deleted logs (one compressed), an older copy of a live chunk and two
other chunks in the free clusters of deleted files (one compressed), and a chunk in `pagefile.sys`.
The script's docstring lists every file. The disk's SHA-256 is
`32153134319a57b1c09553f3086ad8f6c9900f5287cc474aa9f5161902e9d764`
(`sample.raw.gz`: `e1b88c16dce44848cedeed00fede2c165ae29af277417206d0fd605dadd17766`).
