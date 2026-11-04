# खोलें और इस रूप में सहेजें संवाद (OpenSaveMRU) { #open-and-save-as-dialogs-opensavemru }

## संवाद इतिहास क्या है { #what-the-dialog-history-is }

जब कोई program सामान्य Open या Save As संवाद दिखाता है, तो Windows उपयोगकर्ता के
`NTUSER.DAT` में दर्ज करता है:

- **कौन-सी file खोली या सहेजी गई:** `OpenSavePidlMRU` में प्रत्येक extension (`docx`,
  `ps1`, …) के लिए एक subkey और हर extension के लिए `*` होता है, और प्रत्येक एक most-recently-used list है;
- **किस program ने संवाद का उपयोग किया, और वह किस folder में था:** `LastVisitedPidlMRU` (और
  पुराने संवादों का उपयोग करने वाले programs के लिए `LastVisitedPidlMRULegacy`);
- **वे programs जिन्होंने संवाद दिखाया:** `CIDSizeMRU` (प्रत्येक program के लिए संवाद का आकार), और
  `FirstFolder` (किसी program का path और एक folder);
- Windows 2000 और XP पर, `OpenSaveMRU` (प्रत्येक extension के लिए paths) और `LastVisitedMRU` (programs और
  उनके folders, paths के रूप में)।

Windows Vista से files और folders shell item lists होते हैं, यानी वही संरचनाएँ जो किसी
shortcut के target की होती हैं: प्रत्येक item path का एक भाग होता है (This PC, कोई drive, कोई folder, file), और
file या folder item अपने स्वयं के times और NTFS file reference रख सकता है।

<span class="evidence-strength">सर्वोत्तम निष्कर्ष:</span> इस उपयोगकर्ता के session में किसी program ने
इस file को खोलने या सहेजने के लिए, या इस folder में, Open या Save As संवाद का उपयोग किया। प्रत्येक list की केवल सबसे हाल की
entry का समय ज्ञात होता है: उसकी key का last-write time।

## स्थान { #location }

```text
C:\Users\<user>\NTUSER.DAT
Software\Microsoft\Windows\CurrentVersion\Explorer\ComDlg32\
  OpenSavePidlMRU\<extension>   shell item lists, MRUListEx order (Vista and later)
  OpenSavePidlMRU\*             every extension
  LastVisitedPidlMRU            program name, then a shell item list
  LastVisitedPidlMRULegacy      the same
  CIDSizeMRU                    program name
  FirstFolder                   program path, then a folder path
  OpenSaveMRU\<extension>       paths, MRUList order (2000 and XP)
  LastVisitedMRU                program name, then a folder path (2000 and XP)
```

हर profile के लिए `NTUSER.DAT` को उसके transaction logs के साथ संग्रहित करें।

## विश्लेषक के लिए सावधानियाँ { #analyst-cautions }

- **प्रत्येक list के लिए एक समय।** किसी key का last-write time तब बदलता है जब उसकी कोई entry अपनी list के
  आगे आती है, इसलिए यह केवल position 0 की entry का समय बताता है (`IsMostRecent`, `LastUsedUTC`)।
  अन्य entries क्रम में होती हैं, list में नीचे की ओर सबसे नई से सबसे पुरानी, लेकिन उनका समय ज्ञात नहीं होता।
- **दो lists में एक ही file।** कोई file अपने extension की list में और `*` में होती है; इसे
  दो उपयोग न गिनें। `*` सभी extensions में क्रम बताता है।
- **खोली गई या सहेजी गई।** Lists यह नहीं बतातीं कि कौन-सा। File के `$MFT` और USN
  Journal records, LNK files और jump lists से correlate करें।
- **Item times file के होते हैं, संवाद के नहीं।** `ItemModifiedUTC`, `ItemCreatedUTC` और
  `ItemAccessedUTC` item लिखे जाने के समय file के times होते हैं, 2 seconds की सटीकता तक (FAT)। Known
  folders (`CLSID_ThisPCDocumentsRegFolder`, `Downloads`, …) के items में अक्सर कोई समय नहीं होता। इन FAT
  timestamps में कोई timezone offset नहीं होता। `UTC` column suffix एक display convention है, इस बात का
  प्रमाण नहीं कि मूल clock UTC थी; registry FILETIMEs से तुलना करने से पहले source system के timezone और
  अन्य timestamps से correlate करें। Microsoft का
  [file-time guidance](https://learn.microsoft.com/en-us/windows/win32/sysinfo/file-times) देखें।
- **Paths shell paths होते हैं।** ये desktop से शुरू होते हैं: `This PC\C:\Users\…`, कोई known folder, कोई
  library (`UsersLibraries\Documents\…`) या कोई network location। जो path किसी folder
  या file से शुरू होता है, वह desktop पर है। खाली path स्वयं desktop है।
- **Decode न हुए items।** जिस type के item को shell item parser नहीं जानता, उसे उसके
  class (`[class 0x00]`) से दिखाया जाता है, उसका type `unknown` होता है, और list के bytes `DataRaw` में होते हैं।
- **Programs नाम होते हैं।** `LastVisitedPidlMRU` और `CIDSizeMRU` एक file नाम
  (`WINWORD.EXE`) रखते हैं, path नहीं। `FirstFolder` एक path रखता है।
- **Lists छोटी होती हैं और साफ़ की जाती हैं।** प्रत्येक list सीमित संख्या में entries रखती है, और cleanup tools
  keys को delete कर देते हैं। हटाई गई values के लिए `parse-registry read --recover-deleted` का उपयोग करें।

## Juntei से parse करें { #parse-with-juntei }

```text
Usage: juntei parse-registry open-save-mru <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find NTUSER.DAT hives
  -f, --file <FILE>      Parse one NTUSER.DAT
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs
      --code-page <CP>   Code page of shell item names that aren't Unicode (default 1252)

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry open-save-mru -d evidence/ -o opensavemru.csv
juntei parse-registry open-save-mru -f NTUSER.DAT -t jsonl \
  | jq 'select(.List == "OpenSavePidlMRU" and .Extension != "*")'
```

Summary प्रत्येक list की entries, संवादों का उपयोग करने वाले programs, और प्रत्येक hive की सबसे
हाल की file (`*` की पहली entry) को सूचीबद्ध करता है।

## आउटपुट fields { #output-fields }

| Field | व्याख्या |
|---|---|
| `Host`, `User`, `UserSID`, `SourcePath` | Host, profile का स्वामी (path या shell folders से), system के `ProfileList` से SID, और hive। |
| `List` | `OpenSavePidlMRU`, `LastVisitedPidlMRU`, `LastVisitedPidlMRULegacy`, `CIDSizeMRU`, `FirstFolder`, `OpenSaveMRU` या `LastVisitedMRU`। |
| `Extension` | Extension की subkey (`docx`, `*`); अन्य lists के लिए खाली। |
| `MRU-Position`, `IsMostRecent` | Key की list में स्थान (0 सबसे हाल का है); जब list value का नाम नहीं देती तो खाली। |
| `ValueName` | Value की संख्या (`MRUListEx`) या अक्षर (`MRUList`)। |
| `Program` | वह program जिसने संवाद दिखाया। |
| `Path` | File या folder: shell items के नाम जोड़कर, या text के रूप में संग्रहीत path। |
| `ItemType` | अंतिम shell item का type: `file`, `directory`, `root_folder`, `volume`, `network_location`, `property_view`, `unknown`, … |
| `ItemModifiedUTC`, `ItemCreatedUTC`, `ItemAccessedUTC` | File या folder के times, जैसा उसके shell item ने दर्ज किया। |
| `MFT-Entry`, `MFT-Sequence` | Shell item द्वारा दर्ज NTFS file reference। |
| `LastUsedUTC` | Key का last-write time, केवल सबसे हाल की entry के लिए। |
| `KeyPath`, `KeyLastWrittenUTC` | Key और उसका अंतिम write: उसकी सबसे हाल की entry का समय, हर row का नहीं। |
| `DataRaw` | Decode न होने पर value के bytes (hex), या item का type अज्ञात होने पर shell item list। |
| `TransactionLogStatus`, `LogsApplied` | Registry recovery का संदर्भ। |
| `Problems` | ऐसी values जिनका नाम list देती है पर key में नहीं हैं, ऐसी values जिनका नाम list नहीं देती, गलत types, अधूरे कटे नाम या lists। |

## जाँच की रणनीति { #investigation-strategy }

1. प्रत्येक उपयोगकर्ता के लिए `*` को क्रम में पढ़ें, और सबसे हाल की file के लिए `LastUsedUTC` देखें।
2. User-writable, removable या network स्थानों में scripts, archives, executables और दस्तावेज़ खोजें,
   और ऐसी files खोजें जिनके नाम credentials या exports जैसे हों।
3. किसी program को उसके द्वारा अंतिम बार उपयोग किए गए folder से जोड़ने के लिए `LastVisitedPidlMRU` का उपयोग करें, और संवादों का
   उपयोग करने वाले किसी भी program के लिए `CIDSizeMRU` तथा `FirstFolder` का।
4. Paths और `MftEntry` को `$MFT`, USN Journal, LNK files, jump lists,
   ShellBags और RecentDocs से correlate करें।

## अधिक जानकारी { #further-details }

- [libyal winreg-kb: Most recently used (MRU) keys](https://github.com/libyal/winreg-kb/blob/main/docs/sources/explorer-keys/Most-recently-used.md)
- [libyal libfwsi: Windows Shell Item format](https://github.com/libyal/libfwsi/blob/main/documentation/Windows%20Shell%20Item%20format.asciidoc)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
- [ys-lnk-parser](https://github.com/Yamato-Security/ys-lnk-parser)
