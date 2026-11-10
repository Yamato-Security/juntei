# UAL test databases

`server2019/` is the `Sum` directory of a Windows Server 2019 virtual machine from
[DFIRArtifactMuseum](https://github.com/AndrewRathbun/DFIRArtifactMuseum)
(`Windows/SUM/Win2019/GCVM-MUS-001/`, MIT license), without its transaction logs.

| File | Rows (`parse-ese ual`) |
| --- | --- |
| `Current.mdb` | 1: RoleAccess 1 (Print and Document Services) |
| `{32D77B8F-140E-4668-8289-F964D6E7C2F3}.mdb` | 0: the yearly database of 2023, still empty |
| `SystemIdentity.mdb` | 16: SystemIdentity 1, ChainedDatabases 2, Roles 13 |

The databases have no client rows: the client decoding is tested with rows built in
`src/ual.rs`. Every value `parse-ese ual` writes for these and 13 other real UAL databases (308 rows,
145 of them client rows) equals a decoding written independently from dissect.esedb's values.
