# Defender detection history test files

Public Microsoft Defender detection history files (`Scans\History\Service\DetectionHistory\<nn>\<GUID>`),
unmodified, each set with its license beside it. `tests/defender_history_cli.rs` checks
`parse-file defender-history` against what other tools read from the same files.

| Folder | Source | License | Expected (independent) |
| --- | --- | --- | --- |
| `orange/DetectionHistory/original/` | [wddh-parser](https://github.com/cert-orangecyberdefense/wddh-parser) (Orange Cyberdefense CERT), commit 0d7ff1efcb21f722f3465cc0a4916d5ab4b5fde2, `samples/original/` | Apache-2.0, `orange/LICENSE` | What WMI's `MSFT_MpThreatDetection` reported for the file on Windows 10 (Defender 4.18.24090.11), from the sample's notes: `ThreatID` 2147686744, `DomainUser` `DESKTOP-O8964S4\RaptorSniper`, `ThreatStatusID` 3, `CurrentThreatExecutionStatusID` 1, `ThreatStatusErrorCode` 0, `DetectionSourceTypeID` 3, the PowerShell process, `Resources` `file:_C:\Users\RaptorSniper\Downloads\a.zip` and its times |
| `orange/DetectionHistory/search_threat_id/` | The same, `samples/search_threat_id/`: Orange changed the status to 18 | Apache-2.0, `orange/LICENSE` | WMI's `ThreatStatusID` 18, a number without a documented name |
| `plaso/DetectionHistory/` | [plaso](https://github.com/log2timeline/plaso)'s `test_data/windefender_history/`, commit 91b6849503b125267d00ea134248e4fe59e5479c | Apache-2.0, `plaso/LICENSE` and `plaso/NOTICE` | plaso's `windefender_history` parser (log2timeline/plaso:20260720): the file, container file and web file detected, the time (the last resource's tracking start), the threat, user, process and SHA-256 |
| `museum/DetectionHistory/` | [DFIRArtifactMuseum](https://github.com/AndrewRathbun/DFIRArtifactMuseum), commit 7d16ff39c9efc0ddd7307fcb3d30ff5c6e8d99f3, `Windows/WindowsDefender/APTSimulatorVM/Windows Defender/APTSimulatorVM_WindowsDefenderArtifacts.zip`, `Scans/History/Service/DetectionHistory/` | MIT, `museum/LICENSE` | 14 detections and 30 resources (behaviors, command lines, files, processes and registry values), used for the `auto-parse` and `super-timeline` runs |

The same files are fixtures of [ys-defender-history-parser](https://github.com/Yamato-Security/ys-defender-history-parser)
(`tests/fixtures/`), which compares every value with Orange Cyberdefense's `wddh` and with plaso.
The tests also write a copy of the Orange sample without its header's `Magic.Version:1.2`, as some
Windows Server 2016 files are written, and damaged, empty and unrelated files.

SHA-256:

```
176520ad4b28b197ea31d3818967f6593d140e38b0df1ebb64355c6812280f1c  museum/DetectionHistory/00/513F108E-C674-4D19-AE23-79C9AD68A0E6
8d092f1094aa3ead7525be66daff3ed1bce07e9729aa8e1d5fe7bffbcd1daa59  museum/DetectionHistory/02/0F3D5CDF-677A-4AF9-BDE9-54E0ACD9900D
8bea4412a9a17f47b84c83eb4bc83311705bdd824f1853f258eaefcb1b4bb862  museum/DetectionHistory/06/9E32E01D-CD43-43F0-8CAA-B3CDD510DE46
4d0af564c03a56055418edfd70bf4750d82e60670749d822969084f85d01d95e  museum/DetectionHistory/07/954F8207-88D2-445F-8B53-5FD17757E16C
d16ce38c0e259d6168f7f000bf121c3e03a40405e4b0dfe75cbaf1e7b20a2d4e  museum/DetectionHistory/09/E36FC3A2-3409-4121-BA72-C495E2B33F75
ec942f3e0c264134a15ff122cd0cb26f7d09084206876e85bcdba072b462c913  museum/DetectionHistory/09/FDC89D27-9486-452A-AACC-DA003DE0327D
0963f8e1a76eb361f403e51fd75e3b7acdc4d4786d61784d3b54bbfd378750c0  museum/DetectionHistory/10/7A1AAF2D-54D6-401D-B2B6-654FF236A602
c9148141840daaa37903a72a2ed96b1b26be16d47eeb81d7271f6a98ed9a2451  museum/DetectionHistory/11/D28F6999-A792-49B2-A426-F27CCF487969
ced457977bee4324a4a2c2ddb964141793749662b203ad27c31148fdd6e5dee8  museum/DetectionHistory/12/4565EB79-8AE0-4C7D-9CBD-25E244ACE78F
11e671ae8b7a2ed396d8836cdd0eabbffc73322278376d768afb40d1019fb6e6  museum/DetectionHistory/13/DEAD4CDD-57BA-4E73-BEE0-6368EAE6D05D
1f9072de0115c3e1c744a6196606f0d49121bf6d15ee6ee168135a058c049d92  museum/DetectionHistory/17/22685A5C-1579-496A-8845-00EE5D5E3602
45500a55e129baf7bf612f24ca17225c5c2a97122ee077d2f4c5ccf0b156d909  museum/DetectionHistory/18/DC244D4E-9802-4EBB-99B0-9DCB63F4E8DF
ba5994d42fc96d462d2561fc6dbc9022e9fa6a40b1adc98a41999ebf5e2df8b9  museum/DetectionHistory/19/5FF32066-BADC-4665-940A-0A22C83B2981
d04bb546ec93f7a78fc57539613ba04737cb78cfde6c92a71bdbe41814b8ad63  museum/DetectionHistory/20/2E020CDD-FE3C-494C-905B-2943F112A836
84cf9c51ee6435e9d7814c8d9ac7477cc538b69ccafaab0d002ddbde593a437b  orange/DetectionHistory/original/94BBE9CF-CDEB-4885-9178-CC93FB10822D
11737faa63372baf5c1ef561a83fc403c60c6500a0fd5113e834397e6375a8be  orange/DetectionHistory/search_threat_id/94BBE9CF-CDEB-4885-9178-CC93FB10822D
85a1a3251ea3ec19cf34e79d76c0b4cb0fd7cc7d9eedc08e314eec4ce0822956  plaso/DetectionHistory/6AFE33A0-19BA-4FFF-892F-B700539D7D63
21c06655e5e1279de82e4691c78d5fa3a2035d69f2803447821bb06bd915e8d9  plaso/DetectionHistory/FC380697-A68D-4C94-B67F-9B6449039463
```
