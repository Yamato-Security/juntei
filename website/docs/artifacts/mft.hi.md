# मास्टर फ़ाइल टेबल (`$MFT`) { #master-file-table-mft }

## MFT क्या है { #what-the-mft-is }

NTFS हर फ़ाइल और directory को Master File Table में एक record से दर्शाता है। Records में
names, parent references, timestamps, sizes, flags, security identifiers,
reparse जानकारी और data-stream metadata जैसे attributes होते हैं। छोटी फ़ाइल की सामग्री और alternate data streams
**resident** हो सकते हैं, यानी उनके bytes MFT record के भीतर ही संग्रहीत होते हैं।

MFT आधारभूत है क्योंकि यह पूरे volume में allocated और हटाए गए records को कवर करता है।
यह दिखा सकता है कि कोई फ़ाइल मौजूद थी, वह कहाँ linked थी, क्या उसे हटाया गया था, और उसका metadata कैसे
बदला—तब भी जब सामान्य directory browsing में वह अब नहीं दिखती।

<span class="evidence-strength">सर्वोत्तम निष्कर्ष:</span> दिखाए गए
metadata वाला एक NTFS file record संग्रह के समय मौजूद था या किसी unallocated record में बचा रहा। केवल MFT
निष्पादन या उपयोगकर्ता की मंशा सिद्ध नहीं करता।

## स्थान और संग्रह { #location-and-collection }

```text
C:\$MFT
```

`$MFT` संरक्षित NTFS metadata है। इसे raw NTFS access में सक्षम forensic acquisition या triage टूल से
एकत्र करें; सामान्य user-mode copy आमतौर पर इसे पढ़ नहीं सकती। जहाँ संभव हो इसे `$J`, `$LogFile`
और volume metadata के साथ जोड़ें।

## विश्लेषक के लिए सावधानियाँ { #analyst-cautions }

- हटाया गया record आंशिक रूप से overwrite हो सकता है या उसका record number फिर से उपयोग हो सकता है।
- `SequenceNumber` एक ही entry number की पीढ़ियों में अंतर करता है। Entry **और** sequence दोनों का मिलान करें।
- `$STANDARD_INFORMATION` (`0x10`) और `$FILE_NAME` (`0x30`) timestamps का update
  व्यवहार अलग होता है। कोई भी manipulation से अछूता नहीं है।
- `SI<FN`, `ZeroU-Secs` और `Copied` सुराग हैं, timestomping के स्वचालित निर्णय नहीं।
- Hard links एक file record के लिए कई `$FILE_NAME` attributes और paths बनाते हैं।
- NTFS कई names (DOS 8.3 सहित), alternate data streams और extension records संग्रहीत कर सकता है।
- Access-time updates disabled, विलंबित या एकत्रित (coalesced) हो सकते हैं।
- पूरा path parent records से पुनर्निर्मित किया जाता है। `[Orphaned]`, `[Unknown]` या `PathUnknown`
  missing/reused/damaged ancestry का संकेत देते हैं—ये शाब्दिक directory नाम नहीं हैं।
- किसी FILE record का **slack** (उपयोग में आकार से आगे के उसके bytes) पहले रखे गए names रख सकता है: किसी
  छोटी directory की index entries जिसका index किसी index allocation में चला गया (उनमें से अधिकांश वहाँ
  अभी भी उपयोग में हैं), कोई पहले का नाम, या reuse किए गए record का पिछला occupant। Recover किए गए names
  में `$STANDARD_INFORMATION` नहीं होता, और किसी नाम का entry header overwrite हो चुका हो सकता है। जब किसी
  record का उपयोग में आकार या attributes यह नहीं बताते कि उसका slack कहाँ शुरू होता है, तो उससे कुछ भी recover नहीं किया जाता
  और उसकी row का `ParseErrors` `slack not read` और उसका कारण बताता है।

## Juntei से parse करें { #parse-with-juntei }

```text
Usage: juntei parse-ntfs mft <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find $MFT files
  -f, --file <FILE>      Parse one $MFT

Filtering:
  -r, --ranges <RANGES>  Parse selected entry numbers (example: 0-15,30)

Output:
  -o, --output <PATH>                   Save records
  -t, --output-type <TYPE>              csv, ys-ntfs-parser-csv, json, jsonl
  -e, --extract-resident-streams <DIR>  Extract resident $DATA streams

CSV Output:
  -a, --all-times      Include every $FILE_NAME time
  -s, --short-names    Include DOS 8.3-name rows
      --cutoff <DATE>  Keep recent SI-created or SI-modified records

Records:
  -x, --recover-slack  Recover file names from FILE record slack

Resident Data:
  -i, --include-resident-data      Put resident $DATA in the CSV
  -E, --resident-extensions <EXT>  Only these extensions (ex: ps1,bat,vbs,js)
  -M, --resident-max-size <BYTES>  Only data up to this size

Inspection:
  -D, --dump-entry <ENTRY>           Show one entry in full detail
  -L, --list-directory <ENTRY|PATH>  List a directory's contents
```

```bash
juntei parse-ntfs mft -f '$MFT' -o mft.csv
juntei parse-ntfs mft -f '$MFT' --cutoff '2026-10-20T00:00:00Z' -a -o recent.csv
juntei parse-ntfs mft -f '$MFT' -t jsonl -r 32000-32100 | jq .
juntei parse-ntfs mft -f '$MFT' -x -o mft-with-slack.csv
juntei parse-ntfs mft -f '$MFT' -i -E ps1,bat,vbs,js -o scripts.csv
juntei parse-ntfs mft -f '$MFT' -D 0x270-5
juntei parse-ntfs mft -f '$MFT' -L '\Users\alice\Downloads'
```

`-e` से extract किए गए resident streams को कभी execute न करें; Juntei किसी कारण से ही उन्हें `.dontrun` नाम देता है।
`-i` के resident data columns वही bytes हैं: उन्हें अविश्वसनीय सामग्री मानें।

`-D` एक entry print करता है: उसका header, हर attribute (data runs के साथ) और उसके slack के names जैसे
`-t json -x` उन्हें लिखता है, फिर record के bytes। `-L` किसी directory का हर नाम सूचीबद्ध करता है, उपयोग में और
हटाए गए दोनों, उसकी entry और sequence number, type, size और modified समय के साथ। Directory को
entry (`30`, `0x1E`, `30-1`) या path (`\Windows\System32`, `C:/Windows`; case-insensitive) से दिया जाता है।

## CSV आउटपुट fields { #csv-output-fields }

| Field | व्याख्या |
|---|---|
| `EntryNum` | MFT record number। `SequenceNumber` के साथ जोड़ें; record numbers फिर से उपयोग होते हैं। |
| `SequenceNumber` | NTFS references द्वारा उपयोग किया जाने वाला generation counter। Mismatch पुराने/हटाए गए संबंधों को प्रकट कर सकता है। |
| `InUse` | संग्रह के समय allocation flag। `false` का अर्थ है record free/हटाया गया है, न कि सभी bytes अक्षुण्ण हैं। |
| `ParentEntryNumber`, `ParentSequenceNumber` | `$FILE_NAME` से parent directory reference। दोनों भागों को सत्यापित करें। |
| `ParentPath` | पुनर्निर्मित parent path। Unknown/orphan markers अनसुलझी ancestry उजागर करते हैं। |
| `Filename` | वर्तमान row के `$FILE_NAME` या stream से नाम। एक record की कई rows हो सकती हैं। |
| `Ext` | नाम से parse किया गया सुविधाजनक extension; file type का भरोसेमंद निर्धारण नहीं। |
| `Size` | Row/stream से जुड़ा logical आकार। Sparse/compressed allocation अलग हो सकता है। |
| `ReferenceCount` | NTFS hard-link reference count; बचे हुए names की संख्या से सावधानी से तुलना करें। |
| `ReparseTarget` | Junctions/symlinks जैसे समर्थित reparse points के लिए decode किया गया target। Paths redirect कर सकते हैं। |
| `Dir` | Record में directory flag है। |
| `HasADS` | Record में named alternate data streams हैं। इससे वह दुर्भावनापूर्ण नहीं हो जाता। |
| `IsADS` | यह row unnamed file data के बजाय किसी named alternate data stream को दर्शाती है। |
| `SI<FN` | SI creation किसी अलग FN creation समय से पहले है। Copying/timestomping और वैध कारणों की जाँच करें। |
| `ZeroU-Secs` | SI created/modified समय ठीक millisecond सीमा पर पड़ता है। केवल कमज़ोर timestomping heuristic। |
| `Copied` | Modified समय created समय से पहले है। अक्सर file copy के अनुरूप; हमलावर की कार्रवाई का प्रमाण नहीं। |
| `SI-Flags` | `$STANDARD_INFORMATION` file attributes/flags। Hidden/system flags उपयोगी pivots हो सकते हैं। |
| `NameType` | `$FILE_NAME` namespace (Win32, DOS, POSIX या संयुक्त)। Names की दोहरी गिनती से बचने के लिए उपयोग करें। |
| `DataModified (SI)`, `Accessed (SI)`, `MetadataChanged (SI)`, `Created (SI)` | SI times। सामान्य timeline fields, लेकिन user-mode टूल इनमें से कई को बदल सकते हैं। |
| `DataModified (FN)`, `Accessed (FN)`, `MetadataChanged (FN)`, `Created (FN)` | FN times। अलग तरीके से update होते हैं और सामान्य APIs के लिए बदलना आमतौर पर कठिन होता है; फिर भी अपरिवर्तनीय नहीं। |
| `UpdateSequenceNumber` | Record के SI attribute से जुड़ा अंतिम USN। यदि वह USN बचा है तो `$J` में pivot करें। |
| `LogfileSequenceNumber` | FILE record header से NTFS `$LogFile` sequence number; low-level transaction विश्लेषण में उपयोगी। |
| `SecurityID` | NTFS `$Secure` में index; अपने-आप में Windows SID नहीं। फ़ाइल के owner और ACLs के लिए इसे [`parse-ntfs secure-sds`](secure-sds.md) के `SecurityID` से join करें। |
| `ObjectID-FileDroid` | जहाँ मौजूद हो, Object ID / distributed-link-tracking identifier; LNK/Jump List droids से correlate करें। |
| `LoggedUtilStream` | जहाँ समर्थित हो, decode किया गया `$LOGGED_UTILITY_STREAM` context। अपरिचित values की समीक्षा करें। |
| `ZoneID-Contents` | Resident `Zone.Identifier` सामग्री, जैसे ZoneId और download URLs। यह अनुपस्थित, हटाई गई या non-resident हो सकती है। |
| `SourceFile` | Input `$MFT`, hosts या volumes को जोड़ते समय आवश्यक। |
| `ResidentDataBase64`, `ResidentDataHex`, `ResidentDataASCII` | `-i` के साथ: row के resident `$DATA` (फ़ाइल का, या alternate data stream का) की सामग्री, Base64 के रूप में, bytes (`41-42`) के रूप में, और printable ASCII होने पर text के रूप में। Data non-resident होने या `-E`/`-M` द्वारा filter होने पर खाली। |
| `ParseErrors` | इस row को प्रभावित करने वाली क्षति या असमर्थित संबंध। Filtering के दौरान इस column को कभी न हटाएँ। |
| `FromSlack` | `-x` के साथ: FILE record के slack से recover किए गए नाम के लिए `True`। इसके `EntryNum` और `SequenceNumber` वह फ़ाइल हैं जिसकी ओर नाम संकेत करता था (entry header overwrite होने पर खाली), इसके recover किए गए timestamps `(FN)` columns में हैं, और `InUse` `False` है। |
| `SlackRecord`, `SlackOffset` | `-x` के साथ: वह FILE record जिसके slack में नाम था, उसके भीतर लिखे record number के बजाय `$MFT` में उसके स्थान से (extension record का अपना, भले ही उसके names उसके base record के साथ सूचीबद्ध हों), और `$MFT` में नाम का byte offset। Slack row के `ParseErrors` उस record की समस्याएँ हैं, जैसे `fixup values do not match`: recover किए गए bytes प्रभावित हो सकते हैं। |
| `CopyOfEntryInUse` | `-x` के साथ: उपयोग में मौजूद कोई फ़ाइल अभी भी उसी directory में यह नाम रखती है (और वही फ़ाइल है, जब reference बचा हो)। ऐसे names जो उपयोग में किसी फ़ाइल के पास नहीं हैं, वही रोचक हैं: हटाए गए, rename किए गए, या reuse किए गए record का पिछला occupant। |

Timestamp columns MACB क्रम (Modified, Accessed, Changed, Birth/creation) का पालन करते हैं:
`DataModified` SI/FN, `Accessed` SI/FN, `MetadataChanged` SI/FN, फिर `Created` SI/FN।

`(SI)` का अर्थ `$STANDARD_INFORMATION` है; `(FN)` का अर्थ `$FILE_NAME` है। ये नाम किसी पूर्ण record के
सामान्य स्रोत का वर्णन करते हैं। बिना SI attribute वाली सामान्य filename row के लिए,
मौजूदा fallback FN creation को `Created (FN)` में और FN modified/record-change/access
values को क्रमशः `DataModified (SI)`, `MetadataChanged (SI)` और `Accessed (SI)` में रखता है।
Recover की गई slack rows चारों times `(FN)` columns में रखती हैं और SI columns खाली छोड़ती हैं।
सामान्य-row fallback values स्वतंत्र SI साक्ष्य **नहीं** हैं। Timestamp स्रोत निर्धारित करते समय मूल JSON attributes
और slack provenance का उपयोग करें।

`-t ys-ntfs-parser-csv` (पहले `ys-mft-parser-csv`, जो अभी भी स्वीकार्य है) maintained parser के
entry-oriented schema का उपयोग करता है। ये पठनीय column नाम default `-t csv` आउटपुट पर लागू होते हैं,
जिसमें `auto-parse` का `MFT.csv` भी शामिल है। JSON/JSONL हर
attribute (resident `$DATA` सहित, hexadecimal में) और `FullPath` सुरक्षित रखता है; जब flat
investigative CSV low-level structure छोड़ देता है तब इसका उपयोग करें। `-x` के साथ, हर JSON entry में
`SlackFileNames` भी होता है।

## जाँच की रणनीति { #investigation-strategy }

1. किसी समय-अवधि से filter करें, लेकिन हटाए गए records और दोनों timestamp sets रखें।
2. पूरे paths, संदिग्ध double extensions, ADS, staging directories और ज्ञात hashes/names खोजें।
   `-x` के साथ, वे slack names शामिल करें जो उपयोग में किसी फ़ाइल के पास नहीं हैं (`CopyOfEntryInUse` `False`);
   `-i -E ps1,bat,vbs,js` के साथ, छोटे scripts सीधे CSV से पढ़ें।
3. Entry/sequence और `UpdateSequenceNumber` को USN Journal में pivot करें।
4. Prefetch/LNK/Jump List MFT references की तुलना target record से करें।
5. Deletion को Defender क्वारंटीन या USN `FILE_DELETE`/rename गतिविधि से correlate करें।

## अधिक जानकारी { #further-details }

- [Microsoft: Master File Table](https://learn.microsoft.com/en-us/windows/win32/fileio/master-file-table)
- [Microsoft Open Specifications: NTFS](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-ntfs/)
- [ys-ntfs-parser का mft crate](https://github.com/Yamato-Security/ys-ntfs-parser/tree/main/mft)
- [Omer Ben-Amram का mft parser](https://github.com/omerbenamram/mft)
- [libfsntfs format documentation](https://github.com/libyal/libfsntfs/tree/main/documentation)
