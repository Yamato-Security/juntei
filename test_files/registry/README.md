# Registry test hives

`NTUSER.DAT` hives for `parse-registry recentdocs`, gzipped; the tests unpack them. The `SYSTEM`
hives the registry tests also use are in `../shimcache/`.

| File | Source | Licence |
| --- | --- | --- |
| `regipy/NTUSER.DAT.gz` | [regipy](https://github.com/mkorman90/regipy)'s `regipy_tests/data/NTUSER.DAT.xz` (c6de929), recompressed with gzip: a clean Windows 7 hive with 22 recent items in 5 lists | MIT, `regipy/LICENSE` |
| `rathbun_w11/NTUSER.DAT.gz`, `ntuser.dat.LOG1.gz`, `ntuser.dat.LOG2.gz` | The [DFIR Artifact Museum](https://github.com/AndrewRathbun/DFIRArtifactMuseum)'s `Windows/Registry/Win11/RathbunVM/RathbunVM_W11RegistryHives.7z`: the user's dirty hive and its logs, from the same system as `../shimcache/SYSTEM.gz` | MIT, `rathbun_w11/LICENSE` |
