# 登錄自動啟動項目 { #registry-autostarts }

## 這些是什麼 { #what-they-are }

Windows 會從許多登錄位置啟動程式並載入 DLL：在開機時、登入時、程式啟動或結束時、
`cmd.exe` 開啟時、建立 COM 類別時，或是列印多工緩衝處理器、LSA、netsh 或時間服務載入其
外掛程式時。攻擊者會利用這些位置來達成持續存取、權限提升與 DLL 注入，而合法軟體也會使用它們。
Juntei 會從 `SOFTWARE`、`SYSTEM`、`NTUSER.DAT` 與 `UsrClass.dat` 讀取這些位置，並整合為一個
表格。它會為每個程式、DLL 或套件寫入一筆記錄，並為每筆記錄標記其 MITRE ATT&CK 技術。

<span class="evidence-strength">最佳結論：</span>登錄區設定了此程式或 DLL 要啟動或載入。
這並不能證明該程式曾經執行。要讓它執行，需要該機制所指定的條件（登入、被偵錯的程式啟動、
該類別被建立），以及其他執行證據。

## 位置 { #locations }

| 登錄區 | 讀取的內容 |
|---|---|
| `SOFTWARE` | `Microsoft\Windows\CurrentVersion\Run`、`RunOnce`（+`Setup`）、`RunOnceEx`、`RunServices(Once)` 與 `Policies\Explorer\Run`，以及它們的 `Wow6432Node` 與 Terminal Server 安裝模式副本；`Explorer\StartupApproved\Run`、`Run32` 與 `StartupFolder`；`Winlogon`（`Shell`、`Userinit`、`Taskman`、`AppSetup`、`GinaDLL`……以及 `Notify`）；`Windows` 的 `AppInit_DLLs` 與 `LoadAppInit_DLLs`；`Image File Execution Options\*` 的 `Debugger`（含 `UseFilter`）與 `VerifierDlls`；`SilentProcessExit\*` 的 `MonitorProcess`；`Active Setup\Installed Components\*` 的 `StubPath`；`NetSh`；`Command Processor` 的 `AutoRun` |
| `SYSTEM` | `Control\Session Manager`（`BootExecute`、`SetupExecute`、`Execute`、`PendingFileRenameOperations`、`KnownDLLs`、`AppCertDlls`、`Environment` 的 .NET 分析工具變數）；`Control\Lsa` 的 `Authentication`、`Security` 與 `Notification Packages`（+`OSConfig`）；`SecurityProviders`；`Control\Print` 的監視器、提供者與列印處理器；`Services\W32Time\TimeProviders` |
| `NTUSER.DAT` | 使用者的 `Run` 機碼與 `StartupApproved`；`Windows NT\CurrentVersion\Windows` 的 `Load` 與 `Run`；`Winlogon` 的 `Shell`；`Environment`（`UserInitMprLogonScript`、.NET 分析工具）；`Control Panel\Desktop` 的 `SCRNSAVE.EXE`；`PackagedAppXDebug`；`Command Processor` 的 `AutoRun`；`Software\Classes`（與 `UsrClass.dat` 相同） |
| `UsrClass.dat` | `CLSID\*\InprocServer32` 與 `LocalServer32`（以及 `Wow6432Node` 的）；程式類型（`exefile`、`batfile`、`scrfile`……）以及自動提升權限之程式所啟動項目（`ms-settings`、`mscfile`、`Folder`……）的 `shell\open\command`；`ActivatableClasses\…\DebugInformation` |

請收集每個登錄區及其交易記錄檔。使用者登錄區位於 `Users\<name>\NTUSER.DAT` 與
`Users\<name>\AppData\Local\Microsoft\Windows\UsrClass.dat`。

## 分析注意事項 { #analyst-cautions }

- **預設值也會被回報。** 乾淨系統上的大多數記錄都是 Windows 預設值：
  - `Userinit` 為 `userinit.exe`；
  - `Shell` 為 `explorer.exe`；
  - `KnownDLLs`、LSA 套件、列印監視器、netsh 協助程式與 Active Setup 元件都有內建的項目。

  請找出與相同版本之乾淨建置不同的地方。
- **`StartupApproved` 只涵蓋工作管理員的「啟動」清單。** 它指出某個 `Run` 項目或「啟動」
  資料夾中的檔案是否在該處被停用，以及停用的時間。沒有對應 `Run` 值的項目也會被回報，其詳細資訊
  為「only in StartupApproved」。它可能顯示已被移除的自動啟動項目。
- **`OverridesMachineRegistration` 需要系統的 `SOFTWARE` 登錄區。** 使用者的 COM 類別或命令會
  與其進行比對，而它的尋找方式與主機的 `SYSTEM` 登錄區相同。覆寫即為 COM 劫持模式（T1546.015）。
  OneDrive 與其他以使用者為單位的安裝也會註冊它們自己的類別，而這些類別並未覆寫任何項目。
- **只有在能夠分辨時，才會從命令列中拆分出 `ImagePath` 與 `Args`。**
  - 加上引號的程式會在其結束引號處拆分。
  - 不含資料夾的第一個字會被視為程式（`cmd /c …`）。
  - 否則，會在以程式副檔名結尾的最短開頭處拆分。

  若存在其他程式（`C:\Program.exe`），含有空格但未加引號的路徑可能會執行不同的程式。
- **`GlobalFlag` 決定 `VerifierDlls` 與 `MonitorProcess` 的 `Enabled`。** `VerifierDlls` 只有在
  `0x100` 時才會載入。`MonitorProcess` 只有在 `0x200` 且 `SilentProcessExit\<program>`
  `ReportingMode` 為 `0x1` 時才會執行。模式為 `2`（傾印）或 `4`（通知）時不會執行任何程式，而
  `Details` 會列出該模式的旗標。
- **`KeyLastWrittenUTC` 是機碼的最後寫入時間**，而非值的最後寫入時間。多個值共用同一個時間。

## 使用 Juntei 剖析 { #parse-with-juntei }

```text
Usage: juntei parse-registry persistence <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SOFTWARE, SYSTEM, NTUSER.DAT and UsrClass.dat hives
  -f, --file <FILE>      Parse one of them
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Filtering:
      --all-control-sets  Every control set of a SYSTEM hive, not only the one in use

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry persistence -d collection/ -o persistence.csv
juntei parse-registry persistence -f NTUSER.DAT -t jsonl \
  | jq 'select(.Mechanism != "com_server" or .OverridesMachineRegistration == true)'
```

摘要會依機制計算記錄數量。它也會列出缺少對應 `Run` 值的 `StartupApproved\Run` 與 `Run32`
項目（「啟動」資料夾中的檔案沒有可遺失的值），以及覆寫電腦層級設定的使用者 COM 類別與命令。

與所有 `parse-registry` 命令一樣，即使使用 `-C`，它也會拒絕將 `-o` 指定為其所讀取的檔案之一。
這涵蓋登錄區、其交易記錄檔，以及系統的 SYSTEM 與 SOFTWARE 登錄區，無論是透過路徑、符號連結
或硬連結。當它無法判斷時，就不會寫入。

## 輸出欄位 { #output-fields }

| 欄位 | 判讀 |
|---|---|
| `Host`、`SourcePath`、`HiveType` | 主機脈絡、來源登錄區及其類型。 |
| `User`、`UserSID` | 對於使用者登錄區：設定檔資料夾與 SID。SID 取自 `UsrClass.dat` 的根機碼名稱，否則取自系統的 `ProfileList`。 |
| `Mechanism` | `run`、`run_once`、`winlogon`、`ifeo_debugger`、`silent_process_exit`、`appinit_dlls`、`appcert_dlls`、`boot_execute`、`lsa_security_package`、`com_server`、`uac_bypass_command`…… |
| `AttackTechnique` | 有適用者時，為 MITRE ATT&CK 技術（`T1547.001`、`T1546.012`……）。 |
| `KeyPath`、`ValueName` | 登錄區根目錄之下的機碼（名稱依原樣）與值（空白：預設值）。 |
| `Data` | 值的文字，或清單中的一個項目。無法讀取時，或只有 `StartupApproved` 提到該項目時為空。 |
| `ImagePath`、`Args` | 程式或 DLL，以及參數（在能夠分辨時，請參閱上方說明）。 |
| `Enabled`、`DisabledAtUTC` | `StartupApproved`、`LoadAppInit_DLLs`、`GlobalFlag`（搭配 `ReportingMode`）、`UseFilter`、`IsInstalled`、時間提供者的 `Enabled` 或 `ScreenSaveActive` 所表示的狀態。沒有任何資料表示時為空。 |
| `Details` | 更多資訊：擱置中的重新命名會做什麼、COM 類別的名稱、`ReportingMode`、`FilterFullPath`、Active Setup 的 `ShellComponent`…… |
| `OverridesMachineRegistration` | 對於使用者的 COM 類別或命令：電腦層級是否註冊了相同的項目。沒有系統的 `SOFTWARE` 登錄區時為空。 |
| `WOW64` | 位於 `Wow6432Node` 之下：適用於 32 位元程式。 |
| `KeyLastWrittenUTC` | 機碼的最後寫入時間。 |
| `DataRaw` | 當值不是文字或不是有效的 UTF-16 時，該值的位元組（十六進位）。 |
| `TransactionLogStatus`、`LogsApplied` | 登錄復原脈絡。 |
| `Problems` | 錯誤的值類型、無法讀取的資料、大小錯誤的 `StartupApproved` 項目。 |

## 調查策略 { #investigation-strategy }

1. 從摘要開始：
   - 覆寫電腦層級設定的 COM 類別與命令；
   - 沒有對應 `Run` 值的 `StartupApproved` 項目；
   - 很少有項目的機制（`ifeo_debugger`、`silent_process_exit`、`appinit_dlls`、`appcert_dlls`、`uac_bypass_command`、`file_association`、`logon_script`、`dotnet_profiler`）。
2. 將 LSA 套件、`BootExecute`、`KnownDLLs`、列印監視器與 netsh 協助程式與相同 Windows 版本的
   乾淨建置進行比較。
3. 依據檔案系統（$MFT、雜湊與簽章）解析 `ImagePath`，並檢查使用者設定檔、`ProgramData`
   與 `Temp` 路徑。
4. 將 `KeyLastWrittenUTC` 與 Security 及 Sysmon 登錄事件、處理程序建立、Prefetch 與 Amcache
   進行關聯分析。
5. 使用 `parse-registry read --recover-deleted` 找出已從這些機碼中移除的自動啟動項目。

## 延伸資訊 { #further-details }

- [MITRE ATT&CK T1547 Boot or Logon Autostart Execution](https://attack.mitre.org/techniques/T1547/)
- [MITRE ATT&CK T1546 Event Triggered Execution](https://attack.mitre.org/techniques/T1546/)
- [Microsoft：Run 與 RunOnce 登錄機碼](https://learn.microsoft.com/en-us/windows/win32/setupapi/run-and-runonce-registry-keys)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
