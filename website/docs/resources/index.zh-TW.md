# 文章與參考資料 { #articles-and-references }

本頁收集了高品質的入門資源，供想要深入了解跡證指南以外內容的讀者參考。
各跡證頁面的**進一步資訊**章節則包含最具針對性的低階技術連結。

## 給初學者的 DFIR 入門導覽 { #beginner-dfir-orientation }

- [SANS Windows Forensic Analysis 海報](https://www.sans.org/posters/windows-forensic-analysis/)——常見 Windows 跡證及其位置的視覺化地圖。
- [SANS Hunt Evil 海報](https://www.sans.org/posters/hunt-evil/)——實用的獵捕樞紐與 Windows 調查提示。
- [DFIR Artifact Museum](https://github.com/AndrewRathbun/DFIRArtifactMuseum)——供學習與剖析器驗證使用的公開跡證樣本。
- [Velociraptor 跡證參考資料](https://docs.velociraptor.app/artifact_references/)——以收集為導向的說明，以及許多 Windows 跡證的 VQL 實作。
- [Plaso 說明文件](https://plaso.readthedocs.io/)——用於跨跡證分析的時間軸概念與剖析器。

## 檔案系統跡證 { #file-system-artifacts }

- [Microsoft：Master File Table](https://learn.microsoft.com/en-us/windows/win32/fileio/master-file-table)
- [Microsoft Open Specifications：NTFS](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-ntfs/)
- [Microsoft：Change Journals](https://learn.microsoft.com/en-us/windows/win32/fileio/change-journals)
- [Microsoft：USN_RECORD_V2](https://learn.microsoft.com/en-us/windows/win32/api/winioctl/ns-winioctl-usn_record_v2)
- [Microsoft：安全性描述元](https://learn.microsoft.com/en-us/windows/win32/secauthz/security-descriptors)與 [SDDL](https://learn.microsoft.com/en-us/windows/win32/secauthz/security-descriptor-string-format)
- [Microsoft Open Specifications：MS-DTYP](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-dtyp/)——安全性描述元、ACE 與條件運算式。
- [libfsntfs 格式說明文件](https://github.com/libyal/libfsntfs/tree/main/documentation)
- [Omer Ben-Amram 的 mft 剖析器](https://github.com/omerbenamram/mft)
- [dfir_ntfs](https://github.com/msuhanov/dfir_ntfs)——具備 INDX 記錄與索引閒置空間（slack）還原能力的 NTFS 剖析器。
- [INDXRipper](https://github.com/harelsegev/INDXRipper)——從 `$I30` 閒置空間還原檔案名稱。
- [LogFileParser](https://github.com/jschicht/LogFileParser)——詳盡的 NTFS 交易記錄研究與剖析。
- [ntfs-linker](https://github.com/strozfriedberg/ntfs-linker)——交叉關聯 `$LogFile`、`$MFT` 與 USN 歷程。

## 執行與清查跡證 { #execution-and-inventory-artifacts }

- [Security Ronin amcache-forensic](https://github.com/SecurityRonin/amcache-forensic)
- [libyal libscca](https://github.com/libyal/libscca)——Prefetch 格式／函式庫研究。
- [Mandiant：Leveraging the Application Compatibility Cache in Forensic Investigations](https://www.mandiant.com/resources/blog/leveraging-application-compatibility-cache-forensic-investigations)

## 使用者活動跡證 { #user-activity-artifacts }

- [Microsoft Open Specifications：Shell Link（MS-SHLLINK）](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-shllink/)
- [Microsoft Open Specifications：Compound File Binary（MS-CFB）](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-cfb/)
- [LnkParse3](https://github.com/Matmaus/LnkParse3)
- [Microsoft：Extensible Storage Engine](https://learn.microsoft.com/en-us/windows/win32/extensible-storage-engine/extensible-storage-engine)

## 登錄與持續存取 { #registry-and-persistence }

- [Microsoft：登錄值類型](https://learn.microsoft.com/en-us/windows/win32/sysinfo/registry-value-types)
- [Maxim Suhanov 的 Windows 登錄格式規格](https://github.com/msuhanov/regf)
- [yarp](https://github.com/msuhanov/yarp) 與 [libregf](https://github.com/libyal/libregf)——低階登錄區剖析與復原的參考實作。
- [Microsoft：Services 登錄樹狀結構](https://learn.microsoft.com/en-us/windows-hardware/drivers/install/hklm-system-currentcontrolset-services-registry-tree)
- [Microsoft：工作排程器結構描述](https://learn.microsoft.com/en-us/windows/win32/taskschd/task-scheduler-schema)
- [MITRE ATT&CK：Scheduled Task/Job](https://attack.mitre.org/techniques/T1053/)
- [Microsoft Open Specifications：SAM Remote Protocol](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-samr/)

## Defender 與惡意程式處理 { #defender-and-malware-handling }

- [Microsoft：在 Defender 中還原隔離的檔案](https://learn.microsoft.com/en-us/defender-endpoint/restore-quarantined-files-microsoft-defender-antivirus)
- [dissect.target Defender 外掛程式](https://docs.dissect.tools/en/latest/projects/dissect.target/plugins/os/windows/defender.html)
- [defender-dump](https://github.com/knez/defender-dump)——另一套獨立的研究／擷取實作。
- [FIRST Malware Analysis Fundamentals](https://www.first.org/global/sigs/malware/analysis/)——惡意程式處理與分析資源。

## Juntei 剖析器專案 { #juntei-parser-projects }

- [ys-amcache-parser](https://github.com/Yamato-Security/ys-amcache-parser)
- [ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser)（`$MFT`、`$I30`、USN Journal、`$LogFile`、`$Boot`、`$Secure:$SDS`）
- [ys-prefetch-parser](https://github.com/Yamato-Security/ys-prefetch-parser)
- [ys-srum-parser](https://github.com/Yamato-Security/ys-srum-parser)
- [ys-lnk-parser](https://github.com/Yamato-Security/ys-lnk-parser)
- [ys-jump-lists-parser](https://github.com/Yamato-Security/ys-jump-lists-parser)
- [ys-shimcache-parser](https://github.com/Yamato-Security/ys-shimcache-parser)
- [ys-defender-history-parser](https://github.com/Yamato-Security/ys-defender-history-parser)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)

!!! note "外部資料會隨時間變動"
    請針對受影響的 Windows 版本與你所收集的資料驗證各項說法。一篇有用的文章可能只
    描述特定的作業系統世代、剖析器版本或實驗，而非普遍適用的規則。
