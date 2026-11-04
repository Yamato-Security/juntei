# USN जर्नल (`$UsnJrnl:$J`) { #usn-journal-usnjrnlj }

## USN जर्नल क्या है { #what-the-usn-journal-is }

NTFS file और directory बदलावों को Update Sequence Number (USN) Change Journal में दर्ज करता है। एक
record creation, डेटा extension/overwrite, attribute या security बदलाव, rename,
close और deletion का वर्णन कर सकता है। यह एक file reference, parent reference, नाम, reasons, attributes,
timestamp और लगातार बढ़ने वाला USN संग्रहीत करता है।

यह journal attacker के file operations की किसी श्रृंखला को पुनर्निर्मित करने के लिए उत्कृष्ट है, विशेष रूप से जब
files delete हो चुकी हों। यह दर्ज करता है **कि metadata/content बदला**, न कि बदले गए bytes।

<span class="evidence-strength">सर्वोत्तम निष्कर्ष:</span> NTFS ने उस timestamp पर संदर्भित file के लिए
बताए गए change reasons दर्ज किए। Reasons अक्सर `CLOSE` record तक जमा होते रहते हैं।

## स्थान और संग्रह { #location-and-collection }

```text
C:\$Extend\$UsnJrnl:$J
```

`$J` NTFS metadata का एक named stream है। इसे raw-aware forensic tool से संग्रहित करें। उसी
volume का `$MFT` भी संग्रहित करें, क्योंकि USN records पूर्ण paths के बजाय parent file references संग्रहीत करते हैं।

## विश्लेषक के लिए सावधानियाँ { #analyst-cautions }

- Journal circular और आकार-सीमित होता है। पुराने records overwrite हो जाते हैं।
- `FILE_DELETE` बताता है कि NTFS ने deletion process किया; content अन्य स्थानों पर recover योग्य रह सकता है।
- `DATA_OVERWRITE` जैसा reason bytes, ज़िम्मेदार process या उपयोगकर्ता को प्रकट नहीं करता।
- एक record में कई reasons संयुक्त हो सकते हैं; `CLOSE` अक्सर जमा हुए बदलावों के अंत को चिह्नित करता है।
- Paths पुनर्निर्मित होते हैं। Parent directories rename, delete या पुनः उपयोग हो चुकी हो सकती हैं।
- File-reference sequence numbers महत्वपूर्ण हैं। केवल entry number गलत generation से जुड़ सकता है।
- Carving मान्य records को उनके मूल journal संदर्भ के बिना recover कर सकता है; इस अंतर की रिपोर्ट करें।

## Juntei से parse करें { #parse-with-juntei }

```text
Usage: juntei parse-ntfs usn-journal <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find $J files
  -f, --file <FILE>      Parse one $J, or any file with --carve
  -m, --mft <FILE>       Use the matching volume's $MFT for paths
      --carve            Search a disk image/unallocated file for records

Output:
  -o, --output <PATH>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-ntfs usn-journal -f 'C/$Extend/$J' -m 'C/$MFT' -o usn.csv
juntei parse-ntfs usn-journal -d evidence/ -o usn-jsonl -t jsonl
juntei parse-ntfs usn-journal -f unallocated.bin --carve -o carved-usn.csv
```

## CSV आउटपुट fields { #csv-output-fields }

| Field | व्याख्या |
|---|---|
| `Name` | इस USN record में संग्रहीत file/directory नाम। यह पूर्ण path नहीं है। |
| `Extension` | `Name` से parse किया गया सुविधाजनक extension; इसे विश्वसनीय content type के रूप में उपयोग न करें। |
| `EntryNumber`, `SequenceNumber` | File का NTFS reference। Record generation की पहचान के लिए दोनों का उपयोग करें। |
| `ParentEntryNumber`, `ParentSequenceNumber` | बदलाव के समय parent directory का NTFS reference। |
| `ParentPath` | Juntei द्वारा समय को ध्यान में रखकर पुनर्निर्मित parent path। Unknown markers का अर्थ है कि ancestry resolve नहीं हो सकी। |
| `UpdateSequenceNumber` | USN ordering value। स्थिर sorting और Amcache/MFT pivots के लिए उपयोगी। |
| `UpdateTimestamp` | V2/V3 records में संग्रहीत timestamp। Carved-context timestamps का सत्यापन किसी भी अन्य record की तरह ही करें। |
| `UpdateReasons` | एक या अधिक named change flags, `|` से जुड़े हुए। संयोजनों और `CLOSE` की व्याख्या एक साथ करें। |
| `FileAttributes` | बदलाव के समय दर्ज NTFS attributes; अज्ञात bits hexadecimal में रहते हैं। |
| `OffsetToData` | Input में byte offset। सत्यापन और carved records के लिए आवश्यक। |
| `SourceFile` | Input journal/image; hosts और volumes में भेद करता है। |
| `SourceInfo` | Source flags जो बताते हैं कि बदलाव journal में कैसे आया। आमतौर पर संदर्भात्मक, attribution नहीं। |
| `SecurityID` | Record के लिए NTFS security identifier index, उपयोगकर्ता SID नहीं। |
| `Version` | USN record संस्करण। Juntei V2/V3 को CSV में लिखता है; JSON V4 range-tracking extents को बनाए रखता है। |

JSON/JSONL 128-bit V3 references को सही ढंग से दर्शाता है और V4 range-tracking records तथा
उनके extents को सुरक्षित रखता है। जब low-level संस्करण-विशिष्ट fields मायने रखते हों तो इसका उपयोग करें।

## जाँच की रणनीति { #investigation-strategy }

1. समय, नाम, extension, parent path या file reference पर filter करें।
2. सटीक journal क्रम देखने के लिए समय के साथ-साथ `UpdateSequenceNumber` से भी sort करें।
3. Moves/renames के लिए `RENAME_OLD_NAME` को `RENAME_NEW_NAME` के साथ जोड़ें।
4. Record state, paths, sizes और timestamps को सत्यापित करने के लिए `$MFT` का उपयोग करें।
5. Event logs, EDR, Prefetch, services/tasks या Defender से process attribution को correlate करें।

## अधिक जानकारी { #further-details }

- [Microsoft: Change Journals](https://learn.microsoft.com/en-us/windows/win32/fileio/change-journals)
- [Microsoft: USN_RECORD_V2](https://learn.microsoft.com/en-us/windows/win32/api/winioctl/ns-winioctl-usn_record_v2)
- [Microsoft: Change Journal reason flags](https://learn.microsoft.com/en-us/windows/win32/api/winioctl/ns-winioctl-usn_record_v2#members)
- [ys-ntfs-parser का usn crate](https://github.com/Yamato-Security/ys-ntfs-parser/tree/main/usn)
- [Sootmark usn](https://github.com/Sootmark/usn)
