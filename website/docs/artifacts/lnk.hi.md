# LNK शॉर्टकट { #lnk-shortcuts }

## LNK फ़ाइलें क्या हैं { #what-lnk-files-are }

Windows Shell Link (`.lnk`) फ़ाइलें एक target के साथ-साथ launch arguments, working directory, icon,
window state, target metadata, volume/network जानकारी, shell namespace items और वैकल्पिक
distributed-link-tracking identifiers का वर्णन करती हैं। Windows और एप्लिकेशन इन्हें Recent Items,
Desktop, Start Menu और अन्य स्थानों में बनाते हैं; हमलावर भी दुर्भावनापूर्ण शॉर्टकट सीधे भेजते हैं।

Recent फ़ोल्डर की कोई LNK ऐसे target के साथ उपयोगकर्ता की interaction का समर्थन कर सकती है जो अब मौजूद नहीं है,
जिसमें removable drive या network share भी शामिल हैं। कोई भी मनमानी LNK कभी क्लिक किए बिना भी मौजूद हो सकती है।

<span class="evidence-strength">सर्वोत्तम निष्कर्ष:</span> शॉर्टकट दर्ज किए गए target context के साथ बनाया/अपडेट किया गया था।
उसका स्थान और आसपास के आर्टिफ़ैक्ट तय करते हैं कि वह उपयोग का समर्थन करता है या नहीं।

## स्थान और संग्रह { #locations-and-collection }

```text
%APPDATA%\Microsoft\Windows\Recent\*.lnk
%USERPROFILE%\Desktop\*.lnk
%APPDATA%\Microsoft\Windows\Start Menu\Programs\...
%PROGRAMDATA%\Microsoft\Windows\Start Menu\Programs\...
```

केवल `Recent` ही नहीं, बल्कि पूरी user profile और संग्रहीत archives/email सामग्री में खोजें। स्रोत फ़ाइल के
अपने समय और path को सुरक्षित रखें, क्योंकि स्थान बदलने से व्याख्या बदल जाती है।

## विश्लेषक के लिए सावधानियाँ { #analyst-cautions }

- Target times, size, MFT reference और volume data LNK में संग्रहीत snapshots हैं; ये शॉर्टकट फ़ाइल
  का अपना metadata नहीं हैं और पुराने हो सकते हैं।
- `MachineID`, MAC और tracker creation time distributed-link-tracking context का वर्णन करते हैं, जो आमतौर पर
  वह मशीन होती है जिसने link बनाया/अपडेट किया—ज़रूरी नहीं कि वही मशीन जहाँ आपको वह मिला।
- MAC जैसा दिखने वाला UUID node virtualization, randomization या non-network identifiers से आ सकता है।
- `Args` किसी हानिरहित executable (`powershell.exe`, `rundll32.exe`, `mshta.exe`) को ख़तरनाक बना सकते हैं।
- ANSI strings के लिए स्रोत सिस्टम का code page आवश्यक है; कई जापानी links के लिए `--code-page 932` का उपयोग करें।
- शॉर्टकट को गढ़ा या कॉपी किया जा सकता है। Prefetch, Jump List, RecentDocs, event-log या अन्य समर्थन के बिना
  इसके fields को निष्पादन के बराबर न मानें।

## Juntei से parse करें { #parse-with-juntei }

```text
Usage: juntei parse-file lnk <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find *.lnk
  -f, --file <FILE>      Parse one shortcut
      --code-page <CP>   ANSI code page (default 1252; Japanese commonly 932)

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-file lnk -d evidence/ -o lnk.csv
juntei parse-file lnk -d evidence/ --code-page 932 -o lnk-ja.csv
juntei parse-file lnk -f suspicious.lnk -t jsonl | jq .
```

## CSV आउटपुट fields { #csv-output-fields }

| फ़ील्ड समूह | फ़ील्ड और व्याख्या |
|---|---|
| स्रोत | `SourceFile`, `SourceCreated`, `SourceModified`, `SourceAccessed`: शॉर्टकट का path और host file system के समय। ये target times से अलग हैं। |
| Target times | `TargetCreated`, `TargetModified`, `TargetAccessed`: LNK में दर्ज snapshots; उपयोगी ऐतिहासिक सुराग, वर्तमान target metadata नहीं। |
| Target की मूल जानकारी | `FileSize`, `RelativePath`, `WorkingDirectory`, `FileAttributes`, `HeaderFlags`: दर्ज किया गया target/launch context। Arguments अर्थ बदल सकते हैं। |
| Volume | `DriveType`, `VolumeSerialNumber`, `VolumeLabel`: fixed/removable/network context की पहचान करें और अन्य शॉर्टकट से correlate करें। |
| Paths | `LocalPath`, `NetworkPath`, `CommonPath`, `TargetIDAbsolutePath`: पूरक path प्रस्तुतियाँ। असहमति और shell namespace paths की समीक्षा करें। |
| Launch व्यवहार | `Args`, `ShowWindow`, `HotKey`, `ShimLayer`, `DarwinId`: shell target को कैसे launch या configure करता है। `Args` अत्यंत महत्वपूर्ण है। |
| NTFS पहचान | `TargetMFTEntryNumber`, `TargetMFTSequenceNumber`: link बनाते समय का target reference; दोनों को `$MFT` से मिलाएँ। |
| Tracking | `MachineID`, `MachineMACAddress`, `TrackerCreatedOn`, `VolumeDroid`, `FileDroid`, `VolumeDroidBirth`, `FileDroidBirth`: distributed-link tracking। इसे correlation के लिए उपयोग करें, अकेले attribution के लिए नहीं। |
| अतिरिक्त blocks | `ExtraBlocksPresent`: उपलब्ध वैकल्पिक structures की सूची; बताती है कि कौन-से अन्य fields समर्थित हैं। |
| प्रस्तुति | `Name`, `IconLocation`, `IconIndex`, `IconEnvironment`, `EnvironmentVariables`: display और environment-expanded target data। हमलावर भ्रामक नाम/icons का उपयोग कर सकते हैं। |
| Shell फ़ोल्डर | `KnownFolder`, `SpecialFolder`: decode किए गए Windows shell folder identifiers। |
| नेटवर्क | `NetworkDevice`, `NetworkProvider`: network-share/device और provider context। |
| Encoding | `ConsoleCodePage`: जहाँ मौजूद हो, दर्ज किया गया console code page; parser के `--code-page` से अलग। |
| अनुगामी data | `AppendedBytes`: logical end के बाद के bytes। अप्रत्याशित data smuggling या malformed फ़ाइलों का संकेत दे सकता है। |
| सत्यापन | `Problems`: क्षति, truncation, असंगत blocks या decoding सीमाएँ। समीक्षा में शामिल करें। |

JSON/JSONL इसके अतिरिक्त हर target-ID item (kind, name, GUID, FAT times और MFT
reference), सभी extra-data blocks, property-store properties, console settings और कच्चे structural
संबंधों को सुरक्षित रखता है। किसी संदिग्ध शॉर्टकट के लिए केवल flat CSV row पर निर्भर रहने के बजाय इसका उपयोग करें।
Nested `TargetItems` times बिना संग्रहीत offset वाली FAT shell-item clocks से आते हैं।
उनका legacy `Z` rendering UTC सिद्ध नहीं करता; `auto-parse --timezone` उन्हें यथावत रखता है।
LNK header के target FILETIMEs और tracker times convert किए जा सकने वाले absolute timestamps बने रहते हैं।

## जाँच की रणनीति { #investigation-strategy }

1. Target fields की व्याख्या करने से पहले स्रोत स्थान और स्रोत समय की जाँच करें।
2. Target path, `Args`, `WorkingDirectory` और environment blocks से वास्तविक launch का पुनर्निर्माण करें।
3. Target MFT reference, volume serial, network path, droids, MachineID और MAC को सभी साक्ष्यों में pivot करें।
4. निष्पादन के लिए Prefetch और event logs जाँचें; उपयोगकर्ता interaction के लिए Jump Lists/RecentDocs जाँचें।
5. दुर्भावनापूर्ण attachments के लिए appended data सुरक्षित रखें और LNK का स्वयं hash/विश्लेषण करें।

## अधिक जानकारी { #further-details }

- [Microsoft Open Specifications: MS-SHLLINK](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-shllink/)
- [ys-lnk-parser](https://github.com/Yamato-Security/ys-lnk-parser)
- [Sootmark shell](https://github.com/Sootmark/shell)
- [LnkParse3](https://github.com/Matmaus/LnkParse3)
