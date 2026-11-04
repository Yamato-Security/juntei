# ShimCache (AppCompatCache) { #shimcache-appcompatcache }

## 什麼是 ShimCache { #what-shimcache-is }

Windows 的應用程式相容性子系統在評估相容性／shim 行為時，會快取執行檔路徑與中繼資料。
即使檔案被重新命名、移除，或隨卸除式磁碟區中斷連線，此快取仍會保留。其配置與
可用欄位在 Windows XP 至 Windows 11 之間各不相同。

ShimCache 對於發掘歷史路徑很有價值；在格式具有可靠執行旗標的情況下，
也能提供額外的執行佐證。快取的時間通常是**檔案的最後修改時間**，
而非 Windows 看到或執行它的時間。

<span class="evidence-strength">最佳結論：</span>Windows 曾為了相容性處理而觀察到此路徑。
單一項目只是存在／觀察的證據；僅在受支援時才使用 `Executed`。

## 位置與收集 { #location-and-collection }

這些位元組位於 `SYSTEM` 登錄區中，通常在：

```text
ControlSet00N\Control\Session Manager\AppCompatCache\AppCompatCache
```

請收集 `SYSTEM`、`SYSTEM.LOG1` 與 `SYSTEM.LOG2`。Juntei 會讀取所有控制集，並在可能時識別
`Select\Current`。

!!! warning "最新的即時項目可能不在登錄區中"
    Windows 可能將 ShimCache 的更新保存在記憶體中，並在關機／重新開機時才寫出。從執行中系統
    複製的登錄區可能不包含最新的觀察記錄。快取的登錄最後寫入
    時間有助於說明它何時被儲存，但並非個別項目的執行時間。

## 分析注意事項 { #analyst-cautions }

- 快取項目本身**並不能證明執行**。
- `Executed` 可能為 `Yes`、`No` 或 `NA`；可用的語意取決於格式／版本。
- `LastModifiedTimeUTC` 屬於所參照的檔案，而非觀察或執行。
- 順序很有用，但並非普遍適用的精確時間軸；跨控制集／slack 會出現重複項目。
- `FromSlack` 項目是從作用中快取之外復原的，需要更加謹慎。
- 沒有可用交易記錄的不一致（dirty）`SYSTEM` 可能遺漏目前的記錄。請閱讀復原摘要。
- 發現項目表示可疑的命名／路徑模式，而非惡意軟體判定。

## 使用 Juntei 剖析 { #parse-with-juntei }

```text
Usage: juntei parse-registry shimcache <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SYSTEM hives
  -f, --file <FILE>      Parse SYSTEM or a raw AppCompatCache value
      --no-logs          Do not replay SYSTEM.LOG1/SYSTEM.LOG2

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry shimcache -f SYSTEM -o shimcache.csv
juntei parse-registry shimcache -d evidence/ -t jsonl \
  | jq 'select(.Executed == true or (.Findings | length > 0))'
```

## 輸出欄位 { #output-fields }

| 欄位 | 判讀 |
|---|---|
| `ControlSet` | 保存此快取的控制集編號。選擇之前請先比較 `IsCurrent`。 |
| `CacheEntryPosition` | 在解碼後快取中的位置／順序。適用於相對排序，而非時間。 |
| `Path` | 快取的執行檔／套件路徑。主要的獵捕樞紐。 |
| `LastModifiedTimeUTC` | 儲存在項目中的檔案修改時間，並非執行／觀察時間。 |
| `Executed` | 依據受支援的格式旗標為 `Yes`、`No` 或 `NA`。僅在具備版本脈絡時使用。 |
| `Duplicate` | 相同項目已在較早的控制集或 slack 中出現過；有助於避免重複計算。 |
| `SourceFile` | 輸入的登錄區／原始值。跨主機分析時不可或缺。 |
| `FromSlack` | 從快取值的 slack 而非作用中清單復原的項目。可信度較低，且可能較舊。 |
| `CacheFormat` | 解碼後的 Windows／快取配置。決定欄位的可用性與語意。 |
| `CacheLastWrittenUTC` | 登錄機碼／快取的儲存時間。這是共用的脈絡，而非個別項目的執行時間。 |
| `InsertFlags`、`ShimFlags` | 原始的相容性旗標；`Executed` 是 Juntei 在受支援時所判讀的結果。 |
| `DataSize` | 此項目資料的大小，適用於結構驗證。 |
| `Machine` | 較新格式所記錄的架構（`x64`、`x86`、`ARM64`）。 |
| `FileSize`、`LastUpdateTimeUTC` | Windows XP 特有的快取中繼資料（若存在）。 |
| `PackageName`、`PackageVersion`、`PackagePublisherID` | 受支援項目中的封裝應用程式身分。發行者 ID 是脈絡資訊，而非簽章證明。 |
| `Findings` | Juntei 的異常代碼，例如被搬移的系統二進位檔或可疑的暫存路徑。僅作為線索。 |
| `RegistryPath`、`RegistryValueName` | 確切的登錄區機碼／值來源。請保留以供驗證。 |
| `CacheValueSize` | 整個登錄值的大小，而非個別項目的 `DataSize`。 |
| `IsCurrent` | 此項目的控制集是否等於 `SYSTEM\Select\Current`；若無法確實得知則為空。 |
| `SelectCurrent`、`SelectDefault`、`SelectLastKnownGood`、`SelectFailed` | 原始的 `SYSTEM\Select` 控制集編號，用於脈絡與復原分析。 |

JSON／JSONL 另外會保留項目資料、標籤、封裝應用程式的子記錄、結構化的發現項目、
快取問題與復原細節。

## 調查策略 { #investigation-strategy }

1. 優先採用目前的控制集，但保留其他控制集與 slack 作為歷史證據。
2. 搜尋路徑與發現項目；請勿將 `LastModifiedTimeUTC` 當作執行時間。
3. 當 `Executed == Yes` 時，以 Prefetch、SRUM、事件記錄或其他執行期痕跡加以佐證。
4. 將路徑與 `$MFT`／USN 比對，以判斷建立、刪除與中繼資料的歷程。
5. 回報是否已重播交易記錄，以及即時快取是否可能尚未寫出。

## 延伸資訊 { #further-details }

- [ys-shimcache-parser](https://github.com/Yamato-Security/ys-shimcache-parser)
- [Eric Zimmerman 的 AppCompatCacheParser](https://github.com/EricZimmerman/AppCompatCacheParser)
- [Security Ronin 的 shimcache-forensic](https://github.com/SecurityRonin/shimcache-forensic)
- [Mandiant：在鑑識調查中運用應用程式相容性快取](https://www.mandiant.com/resources/blog/leveraging-application-compatibility-cache-forensic-investigations)
- [Maxim Suhanov 的 Windows 登錄格式規格](https://github.com/msuhanov/regf)
