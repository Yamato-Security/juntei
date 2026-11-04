# ShellBags { #shellbags }

## ShellBags क्या हैं { #what-shellbags-are }

Explorer याद रखता है कि उपयोगकर्ता द्वारा खोला गया हर फ़ोल्डर कैसे दिखाया जाता है: उसका view, icon आकार, sort क्रम और
window स्थिति। किसी फ़ोल्डर की settings फिर से खोजने के लिए, यह फ़ोल्डर की पहचान उपयोगकर्ता की
registry में shell items के एक tree के रूप में रखता है, वही items जो किसी link की target ID list में होते हैं: एक drive, फिर
खोले गए फ़ोल्डर तक हर फ़ोल्डर। ये trees ShellBags हैं।

ये Explorer में और open तथा save dialogs में खोले गए फ़ोल्डरों को कवर करते हैं, जिनमें removable
drives, network shares, portable devices (phones, cameras), ZIP archives और FTP servers पर के फ़ोल्डर, और
Control Panel pages शामिल हैं। फ़ोल्डर, drive या network share के न रहने के बाद भी ये बने रहते हैं।

<span class="evidence-strength">सर्वोत्तम निष्कर्ष:</span> इस उपयोगकर्ता के Explorer ने यह फ़ोल्डर
(या इसके अंतर्गत कोई फ़ोल्डर) दिखाया। ShellBag यह नहीं दिखाता कि फ़ोल्डर में क्या किया गया, और यह
हर visit का समय नहीं बताता: नीचे दिए गए times देखें।

## स्थान { #location }

```text
C:\Users\<user>\AppData\Local\Microsoft\Windows\UsrClass.dat
Local Settings\Software\Microsoft\Windows\Shell\BagMRU               (Windows Vista and later)
Wow6432Node\Local Settings\Software\Microsoft\Windows\Shell\BagMRU   (32-bit Explorer)

C:\Users\<user>\NTUSER.DAT
Software\Microsoft\Windows\Shell\BagMRU         (desktop and network items)
Software\Microsoft\Windows\ShellNoRoam\BagMRU   (Windows XP: local folders)
```

एक `BagMRU` key क्रमांकित values रखती है, हर एक एक shell item, और एक `MRUListEx` value जो उनका
क्रम देती है, सबसे हाल में खोला गया पहले। हर value के अंतर्गत के items के लिए उसी नाम की एक subkey होती है, इसलिए
path `BagMRU` से किसी value तक के items हैं। किसी subkey का `NodeSlot` उसकी `Bags\<N>` key का
number है, जहाँ फ़ोल्डर की view settings होती हैं।

हर profile के लिए `UsrClass.dat` और `NTUSER.DAT` को उनके transaction logs (`.LOG1`, `.LOG2`) के साथ एकत्र करें,
और सिस्टम का `SOFTWARE` hive भी, जो `NTUSER.DAT` उपयोगकर्ता का SID देता है।

## कौन-सा समय क्या बताता है { #which-times-say-what }

- **Shell item के times** (`ItemModifiedUTC`, `ItemCreatedUTC`, `ItemAccessedUTC`) फ़ोल्डर के
  अपने file system times हैं (FAT precision: 2 seconds) जब Explorer ने item लिखा। ये
  फ़ोल्डर का समय बताते हैं, visit का नहीं, और बाद के visits पर update नहीं होते।
  FAT clocks में कोई संग्रहीत UTC offset नहीं होता। Legacy `UTC` headings और render किया गया `Z` suffix
  display conventions हैं, स्रोत time zone का प्रमाण नहीं। `auto-parse --timezone` इन
  fields को यथावत रखता है; इनकी व्याख्या स्रोत-सिस्टम के context के साथ करें।
- **किसी key का last-write समय** तब बदलता है जब उसका `MRUListEx` फिर से लिखा जाता है, यानी जब उसके किसी
  item को खोला जाता है और आगे ले जाया जाता है। इसलिए उस key का समय जिसमें किसी item की value है
  (`ParentKeyLastWrittenUTC`) वह समय है जब item अंतिम बार खोला गया था **केवल तब जब वह उस key का सबसे हाल का
  item हो** (`IsMostRecentInParent` true, `MruPosition` 0)। अन्य items के लिए यह बाद का
  समय है, जब किसी sibling को खोला गया।
- किसी item की अपनी key (`KeyLastWrittenUTC`) उसी तरह उसके सबसे हाल के child के खुलने का समय
  बताती है।

## विश्लेषक के लिए सावधानियाँ { #analyst-cautions }

- **अनुपस्थिति बहुत कम सिद्ध करती है।** Explorer ShellBags तब लिखता है जब किसी फ़ोल्डर का view सहेजा जाता है; थोड़ी देर के लिए
  देखा गया, या किसी अन्य तरीके से (command line, script, प्रोग्राम) खोला गया फ़ोल्डर, शायद कोई ShellBag न रखे।
  उपयोगकर्ता keys हटा सकता है।
- **NTFS reference फ़ोल्डर का होता है** (`MftEntry`, `MftSequence`) जब item लिखा गया था।
  यह देखने के लिए कि वही फ़ोल्डर अभी भी मौजूद है या किसी अन्य फ़ाइल ने entry फिर से उपयोग की, इसे `$MFT` में
  खोजें।
- **Removable drives** को उनके drive letter से नाम दिया जाता है। Letter वह है जो drive के पास उस समय था;
  device के लिए इसका मिलान USB और mounted-device history से करें।
- **नाम item से आते हैं।** बाद में rename किया गया फ़ोल्डर यहाँ अपना पुराना नाम रखता है। Root और known
  folders को वैसे नाम दिए जाते हैं जैसे forensic टूल आमतौर पर देते हैं (`This PC`, `Quick Access`, `Downloads`);
  `Guid` उनका identifier देता है।
- **Search results, libraries और Control Panel pages** views हैं, disk पर फ़ोल्डर नहीं
  (`property_view`, `control_panel_item`, `control_panel_category`)।
- **ऐसा item जिसे कोई parser नहीं जानता** अपने bytes `DataRaw` में रखता है, `ItemType` `unknown` और उसकी
  class byte `ItemClass` में। क्षतिग्रस्त या असामान्य items `Problems` में report किए जाते हैं, और उनके bytes
  भी रखे जाते हैं।
- **Dirty hive** में उसके logs के बिना नवीनतम items न हों ऐसा हो सकता है। Juntei hive के पास के logs को replay करता है
  जब तक `--no-logs` न हो; `TransactionLogStatus` बताता है कि कोई record उनसे आया या नहीं।

## Juntei से parse करें { #parse-with-juntei }

```text
Usage: juntei parse-registry shell-bags <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find UsrClass.dat and NTUSER.DAT hives
  -f, --file <FILE>      Parse one UsrClass.dat or NTUSER.DAT
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Parsing:
      --code-page <NUMBER>  Windows code page for ANSI names in shell items (default: 1252)

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry shell-bags -d evidence/ -o shellbags.csv
juntei parse-registry shell-bags -f UsrClass.dat -t jsonl \
  | jq 'select(.ItemType == "volume" and .GUID == "{F5FB2C77-0E2F-4A16-A381-3E560C68BC83}")'
```

Shell items को ys-lnk-parser से decode किया जाता है, जो `parse-file lnk` का shell item parser है।
Summary सबसे हाल में खोले गए फ़ोल्डरों (सबसे नए पहले) को उनकी parent key के last-write समय के आधार पर,
उनके उपयोगकर्ताओं के साथ सूचीबद्ध करता है।

## आउटपुट fields { #output-fields }

| Field | व्याख्या |
|---|---|
| `Host`, `User`, `UserSID`, `SourcePath`, `HiveType` | Host, profile फ़ोल्डर का नाम, उपयोगकर्ता का SID (`UsrClass.dat` की root key इसका नाम बताती है; `NTUSER.DAT` के लिए, सिस्टम का `ProfileList`), hive और उसका type। |
| `BagKey` | Item की key, hive के root के अंतर्गत (`…\BagMRU\4\3\0`): उसकी value उसके ऊपर की key में है। |
| `MRU-Position`, `IsMostRecentInParent` | उसकी parent key के `MRUListEx` में उसका स्थान (0: सबसे हाल में खोला गया); list में न होने पर खाली। |
| `NodeSlot` | उसकी `Bags\<N>` key का number (फ़ोल्डर की view settings)। |
| `Path` | उसके tree के शीर्ष से उस तक के items, `\` से जोड़े हुए। `[unknown]` ऐसे item को दर्शाता है जिसकी value missing है। |
| `ItemType`, `ItemClass` | Item क्या है (`root_folder`, `volume`, `directory`, `file`, `network_location`, `uri`, `ftp_folder`, `compressed_folder`, `portable_device`, `portable_device_folder`, `users_files_folder`, `property_view`, `control_panel_item`, `control_panel_category`, `cd_burn`, `unknown`), और उसकी class byte। |
| `ItemName`, `GUID` | उसका नाम (फ़ोल्डर का long name), और किसी root folder, Control Panel item या delegate folder का class identifier। |
| `ItemModifiedUTC`, `ItemCreatedUTC`, `ItemAccessedUTC` | **Shell item से:** item लिखे जाते समय फ़ोल्डर के times (FAT, 2-second precision)। |
| `MFT-Entry`, `MFT-Sequence` | **Shell item से:** फ़ोल्डर का NTFS file reference। |
| `ExtensionBlocks` | Item में कितने extension blocks हैं। |
| `KeyLastWrittenUTC` | **Registry से:** item की अपनी key का last write: जब उसका सबसे हाल का child खोला गया। |
| `ParentKeyLastWrittenUTC` | **Registry से:** उस key का last write जिसमें उसकी value है: जब वह अंतिम बार खोला गया, यदि `IsMostRecentInParent` true है। |
| `DataRaw` | Value के bytes (hex), ऐसे item के लिए जो पूरी तरह समझा नहीं गया या जिसमें समस्याएँ हैं। |
| `TransactionLogStatus`, `LogsApplied` | Registry recovery का context। |
| `Problems` | ऐसी value जिसका नाम उसकी key का `MRUListEx` लेता है पर key में नहीं है, ऐसी value जिसका नाम list नहीं लेती, अपनी value के बिना subkey, ऐसी value जो binary नहीं है, गलत type का `NodeSlot`, क्षतिग्रस्त shell item list। |

## जाँच की रणनीति { #investigation-strategy }

1. Removable drives (Removable Drives `Guid` वाला `volume`), network locations, FTP
   folders और portable devices, और उनके अंतर्गत के फ़ोल्डर सूचीबद्ध करें।
2. रुचि के फ़ोल्डर खोजें: उपयोगकर्ता द्वारा लिखने योग्य staging स्थान (`Temp`, `ProgramData`, `Public`),
   टूल और archive फ़ोल्डर, अन्य उपयोगकर्ताओं की profiles, और ऐसे फ़ोल्डर जो अब मौजूद नहीं हैं (`$MFT` में
   NTFS reference जाँचें)।
3. Visits का समय केवल उन items के `ParentKeyLastWrittenUTC` से निर्धारित करें जहाँ `IsMostRecentInParent` true है;
   बाकी को LNK फ़ाइलों, Jump Lists और RecentDocs से correlate करें, जो फ़ाइलों के खुलने का समय बताते हैं।
4. हटाई गई `BagMRU` keys और values के लिए `parse-registry read --recover-deleted` का उपयोग करें।

## अधिक जानकारी { #further-details }

- [libyal libfwsi: Windows Shell Item format](https://github.com/libyal/libfwsi/blob/main/documentation/Windows%20Shell%20Item%20format.asciidoc)
- [libyal winreg-kb: Most recently used (BagMRU)](https://winreg-kb.readthedocs.io/en/latest/sources/explorer-keys/Most-recently-used.html)

`ProfileList` से SID lookup के लिए एक अद्वितीय, syntactically मान्य SID आवश्यक है। अस्पष्ट profile-folder mappings और `S-1-...bak` जैसी backup keys `UserSid` को अज्ञात छोड़ देती हैं।
