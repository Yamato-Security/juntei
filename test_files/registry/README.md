# Registry test hives

`NTUSER.DAT` hives for `parse-registry recentdocs`, `SAM` hives for `parse-registry sam` and a
`SECURITY` hive for `parse-registry auditpol` and `lsa-secrets`,
gzipped; the tests unpack them. The `SYSTEM` hives the registry tests also use are in
`../shimcache/`.

| File | Source | Licence |
| --- | --- | --- |
| `regipy/NTUSER.DAT.gz` | [regipy](https://github.com/mkorman90/regipy)'s `regipy_tests/data/NTUSER.DAT.xz` (c6de929), recompressed with gzip: a clean Windows 7 hive with 22 recent items in 5 lists | MIT, `regipy/LICENSE` |
| `regipy/SAM.gz` | The same: `SAM.xz`, a Windows 7 hive with 3 accounts | MIT, `regipy/LICENSE` |
| `rathbun_w11/NTUSER.DAT.gz`, `ntuser.dat.LOG1.gz`, `ntuser.dat.LOG2.gz` | The [DFIR Artifact Museum](https://github.com/AndrewRathbun/DFIRArtifactMuseum)'s `Windows/Registry/Win11/RathbunVM/RathbunVM_W11RegistryHives.7z`: the user's dirty hive and its logs, from the same system as `../shimcache/SYSTEM.gz` | MIT, `rathbun_w11/LICENSE` |
| `rathbun_w11/SAM.gz` | The same archive: the system's `SAM` hive (5 accounts) | MIT, `rathbun_w11/LICENSE` |
| `rathbun_w11/SECURITY.gz`, `SECURITY.LOG1.gz`, `SECURITY.LOG2.gz` | The same archive: the system's dirty `SECURITY` hive and its logs (its audit policy, and the names and times of its LSA secrets and cached logons) | MIT, `rathbun_w11/LICENSE` |
