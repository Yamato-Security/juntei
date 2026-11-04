# USB 裝置 { #usb-devices }

## 什麼是 USB 歷程 { #what-the-usb-history-is }

Windows 會記錄它安裝的每個裝置。對於 USB 儲存裝置、其他 USB 裝置與可攜式裝置（手機、相機、
媒體播放器），`SYSTEM` 登錄區會為每個裝置保存一個機碼，其中包含其名稱、序號，以及 Windows
安裝它、最後一次偵測到它接上與最後一次偵測到它移除的時間。`MountedDevices` 記錄分配給其磁碟區的
磁碟區 GUID 與磁碟機代號。`SOFTWARE` 登錄區保存 Explorer 為可攜式裝置與磁碟區命名的名稱，以及
ReadyBoost 對磁碟區序號的記錄。每位使用者的 `NTUSER.DAT` 則記錄該使用者曾掛載的磁碟區。

`parse-registry usb` 會將它們整合為每個裝置與磁碟區一筆記錄：是哪個裝置、何時安裝與連接、
它使用哪個磁碟機代號與磁碟區，以及哪些使用者曾掛載該磁碟區。

<span class="evidence-strength">最佳結論：</span>具有此廠商、產品與序號的裝置曾連接到此系統，
首次與最後一次分別在這些時間；其磁碟區使用此磁碟機代號；這些使用者曾掛載它。登錄記錄並不會
顯示複製了哪些檔案。

## 位置 { #location }

```text
C:\Windows\System32\config\SYSTEM
  ControlSet00N\Enum\USBSTOR\<Disk&Ven_…&Prod_…&Rev_…>\<serial>&<LUN>
  ControlSet00N\Enum\USB\<VID_xxxx&PID_yyyy>\<serial>
  ControlSet00N\Enum\SWD\WPDBUSENUM\<device path>
    Properties\{83da6326-97a6-4088-9453-a1923f573b29}\0064 … 0067
  MountedDevices
C:\Windows\System32\config\SOFTWARE
  Microsoft\Windows Portable Devices\Devices
  Microsoft\Windows NT\CurrentVersion\EMDMgmt
C:\Users\<user>\NTUSER.DAT
  Software\Microsoft\Windows\CurrentVersion\Explorer\MountPoints2
```

| 裝置屬性 | 欄位 | 意義（`devpkey.h`） |
|---|---|---|
| `0064` | `DriverLastInstalledUTC` | `DEVPKEY_Device_InstallDate`：其驅動程式最後一次安裝的時間 |
| `0065` | `DeviceFirstInstallUTC` | `DEVPKEY_Device_FirstInstallDate`：裝置首次安裝的時間 |
| `0066` | `LastArrivalUTC` | `DEVPKEY_Device_LastArrivalDate`：最後一次連接的時間（Windows 8 及之後版本） |
| `0067` | `LastRemovalUTC` | `DEVPKEY_Device_LastRemovalDate`：最後一次移除的時間（Windows 8 及之後版本） |

Windows 8 及之後的版本將每個時間保存為 `0064` … `0067` 機碼的預設值。Windows 7 則保存在
`00000064\00000000\Data` 中。Juntei 兩者都會讀取。

許多 DFIR 參考資料將 `0064` 標示為首次安裝、將 `0065` 標示為安裝。Juntei 依循上方 Microsoft
`devpkey.h` 的定義，並依各時間所記錄的內容為欄位命名。

請將 `SYSTEM` 登錄區提供給 `-f` 或 `-d`。系統的 `SOFTWARE` 登錄區及其使用者的 `NTUSER.DAT`
登錄區會在其旁邊被找到，也就是收集資料保存它們的方式（`Windows\System32\config\SOFTWARE`、
`Users\<user>\NTUSER.DAT`），或者在登錄區被一起匯出時，位於同一目錄及其下一層。請收集所有登錄區
及其交易記錄檔。

## 記錄如何整合 { #how-the-records-are-joined }

- **裝置：**每個裝置機碼一筆記錄：`usbstor`、`usb` 與 `wpdbusenum`（`DeviceClass`）。USB
  儲存裝置會從具有相同容器 ID（否則為相同序號）的 USB 裝置取得其廠商與產品 ID。
- **磁碟區：**USB 儲存裝置的磁碟區，是 `MountedDevices` 中資料為指名該裝置之路徑的
  `\??\Volume{GUID}` 值。具有兩個磁碟區的裝置會有兩筆記錄。其磁碟機代號是具有相同資料的
  `\DosDevices\X:` 值。
- **名稱與序號：**磁碟區名稱來自 `Windows Portable Devices`，以及指名相同裝置的可攜式裝置。
  磁碟區標籤與序號來自指名它的 `EMDMgmt` 機碼。
- **使用者：**`MountPoints2` 中具有該磁碟區 GUID 的使用者，以及該機碼的最後寫入時間。
- **沒有裝置機碼的磁碟區**（`volume`）：沒有任何裝置機碼指名的 `MountedDevices` 磁碟區（內部
  磁碟的磁碟區，或 `Enum` 機碼已消失之已移除裝置的磁碟區）、沒有任何磁碟區 GUID 值與其共用資料的
  磁碟機代號，以及只有使用者的 `MountPoints2` 才有的磁碟區。
- **只有 `SOFTWARE` 指名的裝置**（`portable_device`、`readyboost`）。

`SourceKeys` 列出整合成一筆記錄的每個機碼與值，以及其所屬的登錄區。

## 分析注意事項 { #analyst-cautions }

- **磁碟機代號會被重複使用。** `MountedDevices` 保存最後分配到各代號的裝置。只有在裝置的資料
  仍是該代號的資料時，才會在該裝置上顯示該代號。
- **序號可能是編造的。** 當執行個體的第二個字元為 `&` 時，代表裝置未回報序號，由 Windows 自行
  產生；此時 `Serial` 為空。這類執行個體在另一台電腦上可能不同。
- **USB 儲存裝置的執行個體以邏輯單元結尾**（`&0`）。`Serial` 會省略它；`Instance` 則會保留。
- **時間因 Windows 版本而異。** Windows 7 只有安裝時間。`LastArrivalUTC` 與 `LastRemovalUTC`
  只代表最後一次連接，而非每一次連接。事件記錄（`Microsoft-Windows-Partition/Diagnostic`
  1006、Kernel-PnP、`DriverFrameworks-UserMode`）與 `setupapi.dev.log` 可提供更多資訊。
- **控制集：**預設會讀取 Windows 最後使用的控制集。`--all-control-sets` 會讀取所有控制集；
  較舊的控制集可能仍列有目前控制集已不再有的裝置。
- **機碼的最後寫入時間並非連接時間。** Windows 會因為許多原因重新寫入裝置機碼。
- **已移除的裝置會留下痕跡。** 具有廠商與產品但沒有裝置機碼的磁碟區記錄，代表該裝置的 `Enum`
  機碼已被移除（因解除安裝或清理工具），但其磁碟區仍存在於 `MountedDevices` 中。
- **ReadyBoost 序號**是磁碟區的序號（`VolumeSerials`，以 `dir` 顯示的形式），LNK 檔案與
  Jump Lists 也會記錄它。

## 使用 Juntei 剖析 { #parse-with-juntei }

```text
Usage: juntei parse-registry usb <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SYSTEM hives (with their SOFTWARE and NTUSER.DAT hives)
  -f, --file <FILE>      Parse one SYSTEM hive (with its SOFTWARE and NTUSER.DAT hives)
      --log1/--log2      Supply the SYSTEM hive's transaction logs explicitly
      --no-logs          Do not replay logs

Filtering:
      --all-control-sets  Read every control set

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry usb -d evidence/ -o usb.csv
juntei parse-registry usb -f C/Windows/System32/config/SYSTEM -t jsonl \
  | jq 'select(.DeviceClass == "usbstor") | {Vendor, Product, Serial, LastArrival, DriveLetters, Users}'
```

摘要會列出最後接上的 USB 儲存裝置（由新到舊）及其磁碟機代號，以及有多少磁碟區屬於沒有裝置機碼的
USB 儲存裝置。

## 輸出欄位 { #output-fields }

| 欄位 | 判讀 |
|---|---|
| `Host` | 電腦名稱，取自 `SYSTEM`。 |
| `DeviceClass` | `usbstor`、`usb`、`wpdbusenum`，或在沒有裝置機碼時為 `volume`、`portable_device` 或 `readyboost`。 |
| `ControlSet` | 裝置機碼所在的控制集。 |
| `Vendor`、`Product`、`Revision` | USB 儲存裝置：取自裝置名稱（`Disk&Ven_…&Prod_…&Rev_…`），也適用於磁碟區或 `SOFTWARE` 機碼所指名的裝置。 |
| `Vid`、`PID` | USB 廠商與產品 ID。 |
| `Serial`、`Instance` | 序號，以及依原樣寫入的執行個體機碼名稱。 |
| `FriendlyName`、`DeviceDesc`、`Service`、`ContainerID` | 裝置機碼的值，依原樣（`DeviceDesc` 通常為 `@<inf>,%…%;<text>`）。可攜式裝置的 `FriendlyName` 通常是其磁碟區的標籤或磁碟機。 |
| `DeviceFirstInstallUTC`、`DriverLastInstalledUTC`、`LastArrivalUTC`、`LastRemovalUTC` | 裝置屬性時間（請參閱上方說明）。 |
| `DriveLetters` | `MountedDevices` 資料與該磁碟區相同的磁碟機代號。 |
| `VolumeGUID`、`VolumeData` | 磁碟區的 GUID，以及其 `MountedDevices` 資料所指出的所在位置：裝置路徑、MBR 磁碟簽章與分割區位移，或 GPT 分割區。 |
| `VolumeNames`、`VolumeSerials` | Explorer 為裝置或其磁碟區命名的名稱、ReadyBoost 標籤，以及磁碟區序號（`1234-ABCD`）。 |
| `User` | `MountPoints2` 中有該磁碟區的使用者，以及該機碼的最後寫入時間（JSON：`User`、`MountPoints2LastWritten`、`SourcePath`）。 |
| `FoundIn`、`SourceKeys` | 發現的位置（`Enum`、`MountedDevices`、`Windows Portable Devices`、`EMDMgmt`、`MountPoints2`），以及整合成該記錄的每個機碼與值。 |
| `KeyPath`、`KeyLastWrittenUTC` | 裝置的執行個體機碼及其最後寫入時間。 |
| `SourcePath`、`SoftwarePath` | `SYSTEM` 登錄區，以及提供了資料時的 `SOFTWARE` 登錄區。 |
| `TransactionLogStatus`、`LogsApplied` | 整合成該記錄之各登錄區的登錄復原脈絡。 |
| `Problems` | 無法讀取的機碼，以及不是 FILETIME 的裝置屬性（連同其位元組）。 |

## 調查策略 { #investigation-strategy }

1. 依 `LastArrivalUTC`（否則依 `DriverLastInstalledUTC`）排序 USB 儲存裝置，並將電腦本身的
   讀卡機與集線器排除在外。
2. 記下每個裝置的廠商、產品、序號、磁碟區 GUID、磁碟機代號與磁碟區序號，並在 LNK 檔案、
   Jump Lists、ShellBags 與 `$MFT`（位於該磁碟機代號或磁碟區上的路徑）中尋找它們。
3. 檢查 `User`，了解誰曾掛載該磁碟區以及時間（`MountPoints2` 機碼的最後寫入時間）。
4. 尋找指名 USB 儲存裝置的 `volume` 記錄：即機碼已被移除的裝置。
5. 與事件記錄進行關聯分析，以取得每一次連接，而不僅是最後一次。
6. 使用 `parse-registry read --recover-deleted` 找出已刪除的裝置機碼。

## 延伸資訊 { #further-details }

- [Microsoft：裝置屬性（devpkey.h）](https://learn.microsoft.com/en-us/windows-hardware/drivers/install/devpkey-device-installdate)
- [Microsoft：支援掛載管理員要求](https://learn.microsoft.com/en-us/windows-hardware/drivers/storage/supporting-mount-manager-requests-in-a-storage-class-driver)
- [`devpkey.h`（Wine 標頭檔中的版本）](https://github.com/wine-mirror/wine/blob/master/include/devpkey.h)：裝置屬性編號
- [libyal winreg-kb：USB storage](https://github.com/libyal/winreg-kb/blob/main/docs/sources/system-keys/USB-storage.md)、
  [Mounted devices](https://github.com/libyal/winreg-kb/blob/main/docs/sources/system-keys/Mounted-devices.md) 與
  [Mount points](https://github.com/libyal/winreg-kb/blob/main/docs/sources/explorer-keys/Mount-points.md)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)

使用者整合依據每個登錄區的來源身分進行，因此具有相同設定檔名稱的匯出登錄區會保持分開。`SourceKeys` 會保留提供資料之使用者登錄區的路徑。與一般遞迴尋找登錄區的行為一致，尋找相關使用者登錄區時會略過符號連結。VID/PID 關聯需要相符且可用的容器 ID；若沒有，序號比對必須產生明確無歧義的 ID。相互衝突的候選項目會維持未知，並附上問題說明。
