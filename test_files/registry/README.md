# Registry test hives

`NTUSER.DAT` hives for `parse-registry recentdocs`, `userassist`, `app-history` and `installed-programs`,
`SAM` hives for `parse-registry sam`, and a `UsrClass.dat` for `parse-registry shellbags` and
`parse-registry muicache`, gzipped; the tests unpack them. The `SYSTEM` hives the registry tests
also use are in `../shimcache/`.

| File | Source | Licence |
| --- | --- | --- |
| `regipy/NTUSER.DAT.gz` | [regipy](https://github.com/mkorman90/regipy)'s `regipy_tests/data/NTUSER.DAT.xz` (c6de929), recompressed with gzip: a clean Windows 7 hive with 22 recent items in 5 lists and WinRAR's history | MIT, `regipy/LICENSE` |
| `regipy/NTUSER-WSL.DAT.gz` | The same: `NTUSER-WSL.DAT.xz`, a Windows 10 hive with three WSL distributions, four App Paths and a user's `Uninstall` entry, for `parse-registry installed-programs` | MIT, `regipy/LICENSE` |
| `regipy/SAM.gz` | The same: `SAM.xz`, a Windows 7 hive with 3 accounts | MIT, `regipy/LICENSE` |
| `plaso/NTUSER-WIN7.DAT.gz` | [Plaso](https://github.com/log2timeline/plaso)'s `test_data/NTUSER-WIN7.DAT` (ee5a0ab), compressed with gzip: a Windows 7 hive with Word 2010's recent files and folders, 3 trusted documents and Office's trusted locations | Apache-2.0, `plaso/LICENSE` |
| `regipy/UsrClass.dat.gz`, `UsrClass.dat.LOG1.gz`, `UsrClass.dat.LOG2.gz` | The same: `UsrClass.dat.xz` and its logs, a dirty Windows 10 hive whose ShellBags hold 29 items, 38 with its logs (removable drives, search results, Control Panel pages) | MIT, `regipy/LICENSE` |
| `rathbun_w11/NTUSER.DAT.gz`, `ntuser.dat.LOG1.gz`, `ntuser.dat.LOG2.gz` | The [DFIR Artifact Museum](https://github.com/AndrewRathbun/DFIRArtifactMuseum)'s `Windows/Registry/Win11/RathbunVM/RathbunVM_W11RegistryHives.7z`: the user's dirty hive and its logs, from the same system as `../shimcache/SYSTEM.gz` | MIT, `rathbun_w11/LICENSE` |
| `rathbun_w11/SAM.gz` | The same archive: the system's `SAM` hive (5 accounts) | MIT, `rathbun_w11/LICENSE` |
| `plaso/NTUSER.DAT.gz` | [Plaso](https://github.com/log2timeline/plaso)'s `test_data/regf/NTUSER.DAT` (ee5a0ab), compressed with gzip: a clean hive with the Open and Save As dialogs' history (`OpenSavePidlMRU`, `LastVisitedPidlMRU`, `CIDSizeMRU`, `FirstFolder`) | Apache-2.0, `plaso/LICENSE` |
| `rathbun_w11/UsrClass.dat.gz`, `UsrClass.dat.LOG1.gz`, `UsrClass.dat.LOG2.gz` | The same archive: the user's dirty `UsrClass.dat` and its logs (12 programs in MuiCache) | MIT, `rathbun_w11/LICENSE` |
| `notatin/win7_ntuser.dat.gz` | [notatin](https://github.com/strozfriedberg/notatin)'s `test_data/win7_ntuser.dat` (ae3a706), compressed with gzip: a clean Windows 7 hive with Remote Desktop, PuTTY, WinSCP, mapped drive and `MountPoints2` keys | Apache-2.0, `notatin/LICENSE` |
| `aptsimulator_w10/NTUSER.DAT.gz`, `ntuser.dat.LOG1.gz`, `ntuser.dat.LOG2.gz` | The [DFIR Artifact Museum](https://github.com/AndrewRathbun/DFIRArtifactMuseum)'s `Windows/Registry/Win10/APTSimulatorVM/APTSimulatorVM_RegistryHives.7z`: `TestUser`'s dirty hive and its logs (byte-identical to the archive's), with the Sysinternals keys PsExec and Sysmon left | MIT, `aptsimulator_w10/LICENSE` |
