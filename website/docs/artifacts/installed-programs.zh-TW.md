# 已安裝程式 { #installed-programs }

## Windows 保存了哪些資料 { #what-windows-keeps }

Windows 與安裝程式會將已安裝的軟體記錄在數個登錄清單中。每份清單中，每個程式、產品、
發行版或套件各有一個機碼；Juntei 會為每個機碼產生一筆記錄。

| 來源 | 登錄區與機碼 | 保存的內容 |
|---|---|---|
| `uninstall` | `SOFTWARE` 與 `NTUSER.DAT` 的 `…\CurrentVersion\Uninstall\*`，以及 `SOFTWARE\Wow6432Node\…` | 「設定」與*程式和功能*所列出的內容：名稱、版本、發行者、安裝日期、資料夾、解除安裝命令、大小、是否隱藏 |
| `msi_product` | `SOFTWARE\Classes\Installer\Products\*`、`NTUSER.DAT` 的 `Software\Microsoft\Installer\Products\*` | Windows Installer 產品：產品名稱與版本、套件的檔名，以及安裝來源資料夾 |
| `app_path` | `SOFTWARE` 與 `NTUSER.DAT` 的 `…\CurrentVersion\App Paths\*` | Windows 可依名稱啟動的程式（`Win+R`、`start tool`）：程式的路徑及其資料夾 |
| `wsl` | `NTUSER.DAT` 的 `…\CurrentVersion\Lxss\{GUID}` | Windows 子系統 Linux 版（WSL）發行版：名稱、其檔案或磁碟的位置、WSL 1 或 2 |
| `package` | `SOFTWARE` 的 `…\Appx\AppxAllUserStore` | 封裝（Store）應用程式：為所有使用者佈建的、Windows 內建的、已暫存的、已取消佈建的，以及各使用者的 |

<span class="evidence-strength">最佳結論：</span>此程式、產品或套件曾安裝（或註冊）於
系統上，供所有使用者或此使用者使用。安裝日期是安裝程式所寫入的值；
機碼的最後寫入時間則是該項目最後變更的時間。

## 位置 { #location }

```text
C:\Windows\System32\config\SOFTWARE
  Microsoft\Windows\CurrentVersion\Uninstall\<name or {ProductCode}>
  Wow6432Node\Microsoft\Windows\CurrentVersion\Uninstall\<…>          32-bit programs
  Classes\Installer\Products\<packed product code>[\SourceList]
  Microsoft\Windows\CurrentVersion\App Paths\<program.exe>
  Microsoft\Windows\CurrentVersion\Appx\AppxAllUserStore\{Applications,InboxApplications,Staged,Deprovisioned,<SID>,EndOfLife\<SID>}\<package>
C:\Users\<user>\NTUSER.DAT
  Software\Microsoft\Windows\CurrentVersion\Uninstall\<…>
  Software\Microsoft\Installer\Products\<packed product code>
  Software\Microsoft\Windows\CurrentVersion\App Paths\<program.exe>
  Software\Microsoft\Windows\CurrentVersion\Lxss\{GUID}
```

請收集 `SOFTWARE`、每個 `NTUSER.DAT` 及其交易記錄。`SYSTEM` 提供主機名稱，
`SOFTWARE` 的 `ProfileList` 則提供每位使用者的 SID。

## 分析注意事項 { #analyst-cautions }

- **安裝日期與機碼時間。**`InstallDate` 是由安裝程式以當地時間寫入的日期（不含時間）；
  有些安裝程式不會寫入，有些則以其他格式寫入（依原樣保留在
  `InstallDateRaw` 中）。更新可能重寫該項目卻不變更日期，也可能變更日期
  而程式本身沒什麼改變：請比較 `InstallDate` 與 `KeyLastWrittenUTC`。
- **已解除安裝的軟體。**被乾淨移除的程式會失去其機碼。請使用 `parse-registry read
  --recover-deleted` 找出已刪除的 `Uninstall` 機碼，並使用 Amcache、Prefetch 與 `$MFT` 找出
  殘留的痕跡。
- **個別使用者的註冊。**`NTUSER.DAT` 項目識別的是為某個設定檔註冊的軟體。
  此範圍並不能確定是誰執行了安裝程式，也不能確定是否使用了系統管理員權限。
- **隱藏的項目。**`SystemComponent` 會讓項目不顯示在*程式和功能*中；
  沒有 `DisplayName` 的項目（更新、元件）也不會顯示。這些項目全都會被回報。
- **產品代碼。**GUID 形式的 `Uninstall` 機碼可以透過 `ProductCode` 與經過壓縮編碼的
  `msi_product` 機碼比對。單憑 GUID 形式的名稱並不能證明
  該項目是由 Windows Installer 建立的。
- **App Paths。**這些是用來尋找程式的註冊資訊，並非執行
  記錄。`Path` 值可以包含多個搜尋目錄，且不一定是
  安裝目錄。
- **發行者。**這是由安裝程式提供的登錄中繼資料，並非經過驗證的簽章
  資訊。
- **架構。**`Wow6432Node` 保存的是 64 位元 Windows 上的 32 位元程式（`Architecture` 為 `x86`、
  `Wow64` 為 `True`）；其他項目則屬於系統本身的架構（空白）。套件的
  架構則是其本身的架構（`x64`、`neutral`……）。
- **WSL。**`Version` 是取自發行版 `Flags` 的 WSL 版本（`0x8`：在 WSL 2 的
  虛擬機器中執行）。發行版本身的 `Version` 值（位於 `Details` 中）是其檔案系統的
  版本，在 WSL 1 發行版上也是 2。`BasePath` 存放其 `ext4.vhdx`（WSL 2）或 `rootfs`
  （WSL 1）：請收集它以檢視其內容。
- **套件。**`<SID>` 清單是為各使用者註冊的套件；大多數是 Windows
  內建的。`Deprovisioned` 套件是對新使用者移除的套件，`EndOfLife\<SID>` 則是對該使用者移除的套件。
- **損毀的值。**格式錯誤的數字、錯誤的值類型與無效的 UTF-16 會在
  `Problems` 中回報，並保留在 `DataRaw` 中；對於存在但長度為零位元組的值，則保留空的十六進位字串。
  選用數值欄位中的乾淨空字串會被視為不存在。套件
  版本必須包含四個十進位部分，且每個部分都在無號 16 位元的範圍內。
  當 `-t` 將記錄串流輸出至 stdout 時，登錄區層級的警告會留在 stderr。

## 使用 Juntei 剖析 { #parse-with-juntei }

```text
Usage: juntei parse-registry installed-programs <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SOFTWARE and NTUSER.DAT hives
  -f, --file <FILE>      Parse one SOFTWARE or NTUSER.DAT hive
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry installed-programs -d evidence/ -o installed.csv
juntei parse-registry installed-programs -d evidence/ -t jsonl \
  | jq 'select(.Scope == "user" and .Source != "package")'
```

摘要會列出最近安裝的程式（依 `InstallDate`）、使用者為自己安裝的
軟體，以及 WSL 發行版。

## 輸出欄位 { #output-fields }

| 欄位 | 判讀 |
|---|---|
| `Host`、`SourcePath`、`HiveType` | 主機、登錄區檔案，以及 `SOFTWARE` 或 `NTUSER.DAT`。 |
| `User`、`UserSID` | 使用者登錄區的使用者（其設定檔資料夾與 `ProfileList` 的 SID），或套件所列出的使用者。 |
| `Source`、`Scope` | 清單（`uninstall`、`msi_product`、`app_path`、`wsl`、`package`），以及 `machine` 或 `user`。 |
| `Name`、`Version`、`Publisher` | `DisplayName`、`DisplayVersion`、`Publisher`；產品的 `ProductName` 與 `Version`；App Path 的程式檔名；發行版的名稱與 WSL 版本；套件的名稱與版本。 |
| `InstallDate`、`InstallDateRaw` | 當 `InstallDate` 以 `YYYYMMDD` 寫入時以日期呈現；否則依原樣呈現。 |
| `KeyLastWrittenUTC` | 該項目機碼最後變更的時間。 |
| `InstallLocation`、`InstallSource` | `InstallLocation`（App Path 的 `Path`、發行版的 `BasePath`、套件的 `Path`）；`InstallSource`（產品最後使用的來源資料夾）。 |
| `Program` | App Path 的程式。 |
| `UninstallCommand`、`QuietUninstallCommand` | `UninstallString`、`QuietUninstallString`。 |
| `SizeKB`、`SystemComponent`、`WindowsInstaller` | 以 KiB 為單位的 `EstimatedSize`；是否隱藏；是否由 Windows Installer 安裝。 |
| `ProductCode` | Windows Installer 產品代碼（`{GUID}`）。 |
| `Architecture`、`WOW64` | 位於 `Wow6432Node` 下時為 `x86`，或套件的架構；該機碼是否位於 `Wow6432Node` 下。 |
| `Details` | 其他資訊：`DisplayIcon`、`ModifyPath`、`ParentKeyName`、`ReleaseType`、`Inno Setup: User`（執行 Inno Setup 安裝程式的帳戶）；產品的 `PackageCode`、`PackageName`；發行版的 `Flags`、`State`、`DefaultUid`、`Version`、`PackageFamilyName`、`Default`；套件的 `List`、`ResourceId`、`PublisherId`。 |
| `KeyPath` | 該項目的機碼。 |
| `DataRaw` | 內容與其名稱不符之值的位元組（十六進位）。 |
| `TransactionLogStatus`、`LogsApplied` | 登錄復原脈絡。 |
| `Problems` | 無法讀取或內容與其名稱不符的值。 |

## 調查策略 { #investigation-strategy }

1. 依 `InstallDate` 與 `KeyLastWrittenUTC` 排序事件前後的項目；留意遠端存取
   工具、通道與檔案傳輸用戶端、壓縮工具、指令碼執行環境與資安工具。
2. 檢視 `Scope` 為 `user` 的記錄：未使用系統管理員權限安裝的軟體。
3. 檢查 `InstallLocation`、`InstallSource` 與 `UninstallCommand` 是否指向使用者可寫入的資料夾
   （`Downloads`、`AppData`、`Temp`、`ProgramData`）。
4. 尋找 WSL 發行版，並收集其 `BasePath`。
5. 與 Amcache 和 Prefetch（安裝程式與程式的執行）、`$MFT`（安裝
   資料夾的建立）以及 Application 事件記錄（MsiInstaller 事件）交叉比對。

## 延伸資訊 { #further-details }

- [Microsoft：Uninstall 登錄機碼](https://learn.microsoft.com/en-us/windows/win32/msi/uninstall-registry-key)
- [Microsoft：應用程式註冊（App Paths）](https://learn.microsoft.com/en-us/windows/win32/shell/app-registration)
- [Microsoft：套件身分識別概觀](https://learn.microsoft.com/en-us/windows/apps/desktop/modernize/package-identity-overview)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
