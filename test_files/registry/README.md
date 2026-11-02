# Registry test hives

`NTUSER.DAT` hives for `parse-registry recentdocs` and `SAM` hives for `parse-registry sam`,
gzipped; the tests unpack them. The `SYSTEM` hives the registry tests also use are in
`../shimcache/`.

| File | Source | Licence |
| --- | --- | --- |
| `regipy/NTUSER.DAT.gz` | [regipy](https://github.com/mkorman90/regipy)'s `regipy_tests/data/NTUSER.DAT.xz` (c6de929), recompressed with gzip: a clean Windows 7 hive with 22 recent items in 5 lists | MIT, `regipy/LICENSE` |
| `regipy/SAM.gz` | The same: `SAM.xz`, a Windows 7 hive with 3 accounts | MIT, `regipy/LICENSE` |
| `plaso/NTUSER-WIN7.DAT.gz` | [Plaso](https://github.com/log2timeline/plaso)'s `test_data/NTUSER-WIN7.DAT` (ee5a0ab), compressed with gzip: a Windows 7 hive with Word 2010's recent files and folders, 3 trusted documents and Office's trusted locations | Apache-2.0, `plaso/LICENSE` |
| `rathbun_w11/NTUSER.DAT.gz`, `ntuser.dat.LOG1.gz`, `ntuser.dat.LOG2.gz` | The [DFIR Artifact Museum](https://github.com/AndrewRathbun/DFIRArtifactMuseum)'s `Windows/Registry/Win11/RathbunVM/RathbunVM_W11RegistryHives.7z`: the user's dirty hive and its logs, from the same system as `../shimcache/SYSTEM.gz` | MIT, `rathbun_w11/LICENSE` |
| `rathbun_w11/SAM.gz` | The same archive: the system's `SAM` hive (5 accounts) | MIT, `rathbun_w11/LICENSE` |
