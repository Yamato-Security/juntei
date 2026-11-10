# Jump Lists { #jump-lists }

## Jump Lists क्या हैं { #what-jump-lists-are }

Jump Lists taskbar और Start-menu की application entries से दिखाए जाने वाले menus का आधार हैं। Automatic Jump
Lists किसी application द्वारा उपयोग किए गए items की Windows-प्रबंधित histories हैं। Custom Jump Lists
application-परिभाषित categories और tasks हैं। दोनों में target paths, times,
volume/network context, MFT references और launch arguments वाली LNK structures embed हो सकती हैं।

Automatic records किसी application द्वारा खोली गई फ़ाइलें या फ़ोल्डर, अंतिम उपयोग का क्रम/समय,
interaction count और pinned स्थिति प्रकट कर सकते हैं। Custom lists "हाल में बंद किए गए" pages या application
tasks उजागर कर सकती हैं—जिनमें दुर्भावनापूर्ण command arguments भी शामिल हैं।

<span class="evidence-strength">सर्वोत्तम निष्कर्ष:</span> स्वामी application ने अपनी Jump List में
item/task को दर्ज या उजागर किया। व्यवहार application के अनुसार भिन्न होता है; वास्तविक निष्पादन/access की पुष्टि करें।

## स्थान और संग्रह { #locations-and-collection }

```text
%APPDATA%\Microsoft\Windows\Recent\AutomaticDestinations\*.automaticDestinations-ms
%APPDATA%\Microsoft\Windows\Recent\CustomDestinations\*.customDestinations-ms
```

Hexadecimal file-name prefix एक AppID है। दोनों directories एकत्र करें और user profile
path सुरक्षित रखें। Automatic lists OLE Compound File containers हैं जिनमें एक `DestList` stream और क्रमांकित LNK
streams होते हैं; custom lists अपने स्वयं के category/link format का उपयोग करती हैं।

## विश्लेषक के लिए सावधानियाँ { #analyst-cautions }

- AppID विवरण एक ज्ञात mapping से आते हैं; अज्ञात AppID अपने-आप में संदिग्ध नहीं है।
- `LastModified` / last-used fields किसी DestList entry के होते हैं; embedded target times अलग होते हैं।
- Interaction counts केवल समर्थित versions में होते हैं और खोलने की पूर्ण गिनती नहीं हैं।
- Pinning items को सामान्य recency से परे सुरक्षित रखती है; cleanup या application settings अन्य items हटा सकती हैं।
- किसी list में पुराने/unattached streams या किसी पहले की, लंबी custom list के bytes हो सकते हैं।
- Embedded LNK arguments और paths हमलावर द्वारा नियंत्रित किए जा सकते हैं। Application से संबंध context है, भरोसे का आधार नहीं।

## Juntei से parse करें { #parse-with-juntei }

```text
Usage: juntei parse-file jump-lists <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find automatic/custom destination files
  -f, --file <FILE>      Parse one Jump List
      --code-page <CP>   ANSI code page for embedded links

Output:
  -o, --output <PATH>       Directory of CSV tables, or JSON/JSONL file
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-file jump-lists -d evidence/ -o jump-list-csv
juntei parse-file jump-lists -f sample.automaticDestinations-ms -t jsonl | jq .
```

`auto-parse` के साथ, CSV tables `FileAndFolderOpening/JumpLists-AutomaticDestinations.csv`
और `FileAndFolderOpening/JumpLists-CustomDestinations.csv` होती हैं, सीधे हर computer के अंतर्गत।
साझा log उस category में `JumpLists-logs.txt` है। Manifests `OutputFiles` में सटीक paths
सूचीबद्ध करते हैं। Standalone CSV exports आपके द्वारा चुनी गई directory में `AutomaticDestinations.csv` और
`CustomDestinations.csv` रखते हैं। JSON और JSONL automatic exports
एकल `JumpLists.json` या `JumpLists.jsonl` फ़ाइलें होते हैं; record structure अपरिवर्तित रहता है।

## AutomaticDestinations CSV fields { #automaticdestinations-csv-fields }

| Field समूह | Fields और व्याख्या |
|---|---|
| स्रोत | `SourceFile`, `SourceCreated`, `SourceModified`, `SourceAccessed`: Jump List container और उसके file-system times। |
| Application | `AppId`, `AppIdDescription`: file-name AppID और ज्ञात application mapping। अज्ञात hash से पहचान का अनुमान न लगाएँ। |
| Container | `HasSps`, `DestListVersion`, `LastUsedEntryNumber`: DestList structure/version और header context। |
| क्रम | `MRU`, `EntryNumber`: most-recent-use स्थिति और hexadecimal entry/stream identifier। |
| उपयोग metadata | `CreationTime`, `LastModified`, `InteractionCount`, `PinStatus`, `PinPosition`: DestList lifecycle/उपयोग/pin स्थिति। Version के अनुसार व्याख्या करें। |
| Host/path | `Hostname`, `MacAddress`, `Path`, `DisplayPath`: target स्थान और tracking context। Host/MAC अकेले attribution नहीं है। |
| Tracking IDs | `FileBirthDroid`, `FileDroid`, `VolumeBirthDroid`, `VolumeDroid`: स्थानांतरित targets और embedded LNK tracking को correlate करें। |
| Notes | `Notes`: unattached streams, recover किए गए leftovers या अन्य parser context के बारे में बताता है। |
| Embedded LNK | `TargetCreated`, `TargetModified`, `TargetAccessed`, `FileSize`, `RelativePath`, `WorkingDirectory`, `FileAttributes`, `HeaderFlags`, `DriveType`, `VolumeSerialNumber`, `VolumeLabel`, `LocalPath`, `CommonPath`, `TargetIDAbsolutePath`, `TargetMFTEntryNumber`, `TargetMFTSequenceNumber`, `MachineID`, `MachineMACAddress`, `TrackerCreatedOn`, `ExtraBlocksPresent`, `Args`: [LNK शॉर्टकट](lnk.md) पर बताए अनुसार व्याख्या करें। |
| अतिरिक्त LNK | `NetworkPath`, `Name`, `IconLocation`: embedded शॉर्टकट के network और presentation fields। |
| सत्यापन | `Problems`: missing streams, truncation, गिनती में असहमति या link की क्षति। |

## CustomDestinations CSV fields { #customdestinations-csv-fields }

Custom आउटपुट `SourceFile`, `SourceCreated`, `SourceModified`, `SourceAccessed`, `AppId`,
`AppIdDescription` और `EntryName` (category नाम या `Tasks`) से शुरू होता है, उसके बाद ऊपर सूचीबद्ध सभी embedded LNK fields,
फिर `Notes`, `NetworkPath`, `Name`, `IconLocation` और `Problems` आते हैं।

JSON/JSONL इसके अतिरिक्त DestList header, प्रति-entry score/checksum/property store,
`DestListPropertyStore`, compound-file streams, custom categories, अंतिम category के बाद के bytes,
और हर embedded LNK के nested fields सुरक्षित रखता है।

## जाँच की रणनीति { #investigation-strategy }

1. उपयोगकर्ता और स्रोत profile की पहचान करें, फिर AppID को सावधानी से application से map करें।
2. Automatic entries को `LastModified` और `MRU` के अनुसार sort करें; pinned items को अलग करें।
3. Paths, network shares, removable volume serials, MFT references और arguments की समीक्षा करें।
4. `Notes` और unattached/leftover links की जाँच करें—हटाए गए या पहले के items वहाँ बचे रह सकते हैं।
5. RecentDocs, standalone Recent LNKs, Prefetch, SRUM, `$MFT` और event logs से correlate करें।

## अधिक जानकारी { #further-details }

- [Microsoft: Compound Binary File format (MS-CFB)](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-cfb/)
- [Microsoft: Shell Link format (MS-SHLLINK)](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-shllink/)
- [ys-jump-lists-parser](https://github.com/Yamato-Security/ys-jump-lists-parser)
- [Eric Zimmerman का JLECmd](https://github.com/EricZimmerman/JLECmd)
- [JumpList parser test/reference project](https://github.com/AbdulRhmanAlfaifi/jumplist_parser)
