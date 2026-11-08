# पार्सर सत्यापन { #parser-validation }

Juntei के अनुरक्षित parser crates का परीक्षण सार्वजनिक साक्ष्य और स्वतंत्र
implementations के विरुद्ध किया जाता है। ये तुलनाएँ regressions और व्याख्या के अंतरों का पता लगाने में मदद करती हैं; ये
किसी parser को अचूक नहीं बनातीं। Source साक्ष्य सुरक्षित रखें, प्रत्येक record के problem या
damage fields की समीक्षा करें, और महत्वपूर्ण findings को किसी अन्य आर्टिफ़ैक्ट या implementation से सत्यापित करें।

नीचे दिए गए कथन वह सत्यापन जानकारी सुरक्षित रखते हैं जो पहले Juntei के मुख्य
README में रखी जाती थी। वर्तमान test fixtures और implementation विवरण के लिए linked parser repositories देखें।

## Amcache { #amcache }

Juntei [ys-amcache-parser](https://github.com/Yamato-Security/ys-amcache-parser) का उपयोग करता है, जो
Security Ronin के
[amcache-forensic](https://github.com/SecurityRonin/amcache-forensic) का Apache-2.0 fork है। इसका
[output documentation](https://github.com/Yamato-Security/ys-amcache-parser/blob/main/docs/output.md)
हर executable, device, finding और summary field को परिभाषित करता है।

## मास्टर फ़ाइल टेबल { #master-file-table }

[ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) का mft crate Omer
Ben-Amram के [mft](https://github.com/omerbenamram/mft) crate का अनुरक्षित fork है। आठ `$MFT` files में—crate का
sample, NIST hacking case और छह DFIR Artifact Museum files—1,048,733 CSV rows और columns
क्रम में एक संदर्भ parser से मेल खाए। तुलनाओं में default output, `--all-times`,
`--short-names` और `--cutoff` शामिल थे; timestamp की प्रस्तुति जानबूझकर रखा गया अंतर था।

## NTFS ट्रांज़ैक्शन लॉग ($LogFile) { #ntfs-transaction-log-logfile }

[ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) का logfile crate
[Sootmark ntfslog](https://github.com/Sootmark/ntfslog) से अनुरक्षित है। इसके सत्यापन में छह पूर्ण DFIR
Artifact Museum logs, सार्वजनिक version 1.1 और 2.0 fixtures, decode किया गया redo/undo metadata, और
स्वतंत्र रूप से parse किए गए NTFS डेटा से तुलना किए गए journal writes शामिल हैं। विस्तृत
coverage और ज्ञात अंतरों के लिए parser repository देखें।

Juntei ऐसे CLI tests जोड़ता है जो सार्वजनिक go-ntfs test
volume (2,129 records) पर हर CSV field की JSON और JSONL से तुलना करते हैं, अलग redo/undo डेटा और raw payload lengths सत्यापित करते हैं, क्षतिग्रस्त logs का परीक्षण करते हैं,
साक्ष्य overwrites को अस्वीकार करते हैं, और वर्तमान MFT paths को पुनः उपयोग हुए record numbers का अनुसरण करने से रोकते हैं।
Regression cases प्रभावित MFT records को उस खुले attribute से अलग करते हैं जिसके माध्यम से वे लिखते हैं,
flat directory-entry/data-attribute metadata की जाँच करते हैं, और cross-volume MFT pairing को रोकते हैं।
Multi-volume `auto-parse` tests अलग CSV files और provenance के साथ एक एकल JSONL file को कवर करते हैं।

Command को छह निजी रूप से संग्रहित Windows logs पर भी जाँचा गया: export किए गए सभी 708,465 CSV
records field-दर-field JSONL से मेल खाए, और प्रत्येक host के `auto-parse` exports ने समान record
counts बनाए रखे। Source साक्ष्य repository के साथ वितरित नहीं किया जाता। ये जाँचें आउटपुट की
सुसंगतता सत्यापित करती हैं, यह नहीं कि हर logged transaction किसी पूर्ण उपयोगकर्ता क्रिया को दर्शाता है।

## Prefetch { #prefetch }

[ys-prefetch-parser](https://github.com/Yamato-Security/ys-prefetch-parser)
ForensicRS के [frnsc-prefetch](https://github.com/ForensicRS/frnsc-prefetch) का अनुरक्षित fork है। इसके decode किए गए आउटपुट की
तुलना libyal के स्वतंत्र Prefetch implementation [libscca](https://github.com/libyal/libscca) से की गई।

## USN जर्नल { #usn-journal }

[ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) का usn crate
[Sootmark usn](https://github.com/Sootmark/usn) का अनुरक्षित fork है। छह वास्तविक journals पर, Juntei के
समय को ध्यान में रखने वाले parent paths एक स्वतंत्र parser से हर उस स्थान पर मेल खाए जहाँ उस parser ने path resolve किया। Juntei
ने ऐसे 4,321 records भी resolve किए जिन्हें तुलना ने unresolved छोड़ दिया था। 880 MB `$MFT` डेटा के विरुद्ध परीक्षण के दौरान
इसके carver ने कोई false records उत्पन्न नहीं किए।

## डायरेक्टरी इंडेक्स ($I30) { #directory-indexes-i30 }

[ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) के mft crate का `i30` module INDX
records और उनके slack को पढ़ता है। ntfs-3g से बनाए गए volume के दो `$I30` streams (Juntei के
`test_files/i30/`) पर और Windows द्वारा लिखे गए तीन सार्वजनिक `$I30` samples पर, इसकी उपयोग में और
slack में मौजूद entries Maxim Suhanov के [dfir_ntfs](https://github.com/msuhanov/dfir_ntfs) की entries से
समान नामों और times के साथ मेल खाईं। जहाँ dfir_ntfs 32 bits से आगे का parent reference रिपोर्ट करता है (parent पर
overwrite हुए bytes), वहाँ Juntei parent को खाली छोड़ता है।

## बूट सेक्टर ($Boot) { #boot-sector-boot }

[ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) का boot crate Yamato
Security का अपना है। DFIR Artifact Museum की पाँच सार्वजनिक `$Boot` files (Windows 10, 11 और
Server 2012 R2) पर, हर field एक संदर्भ parser से मेल खाया। 512-,
1,024- और 4,096-byte sectors तथा 4 KiB से 64 KiB clusters के साथ `mkntfs` से बनाए गए volumes पर, sizes, `$MFT` और `$MFTMirr`
clusters तथा serial numbers The Sleuth Kit के `fsstat` से मेल खाए। Museum के Windows 11 `$Boot`
में वही volume serial number है जो उसी host की LNK file दर्ज करती है, और पाँचों files में
समान boot code है।

## सुरक्षा विवरणक ($Secure:$SDS) { #security-descriptors-securesds }

[ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) का secure crate Yamato
Security का अपना है। Museum के पाँच `$SDS` streams (7,808 descriptors, 60,781 ACEs) में,
security IDs, hashes, offsets, owners, groups, control flags और ACE counts एक संदर्भ
parser से मेल खाए, और हर ACE का type, flags, mask, SID और object types `dissect.ntfs` से मेल खाए। हर
दर्ज hash उसी के descriptor का है, हर mirror copy अपनी entry से मेल खाती है, और हर descriptor में
SDDL है, जिसमें उसके 3,384 callback ACEs की conditions भी शामिल हैं।

## SRUM { #srum }

[ys-srum-parser](https://github.com/Yamato-Security/ys-srum-parser) Sootmark के
[ese](https://github.com/Sootmark/ese) और [srum](https://github.com/Sootmark/srum) crates पर आधारित है।
11 वास्तविक databases में, उत्पन्न 45,255 entries का हर column
`dissect.esedb` द्वारा उजागर values की स्वतंत्र decoding से मेल खाया।

## उपयोगकर्ता एक्सेस लॉगिंग { #user-access-logging }

`parse-ese ual` उसी ESE reader से UAL databases पढ़ता है। 16 वास्तविक databases (308 rows, जिनमें से 145
client rows) में, हर value `dissect.esedb` द्वारा उजागर values की स्वतंत्र decoding से मेल खाई। हर client row में,
गिनती वाला पहला दिन `InsertDate` का UTC दिन था, आखिरी दिन `LastAccess` का UTC दिन, और गिनतियों का योग
`TotalAccesses` के बराबर था।

## वेब ब्राउज़र { #web-browsers }

[ys-browser-parser](https://github.com/Yamato-Security/ys-browser-parser) Sootmark के browser, sqlite और plist
crates का अनुरक्षित fork है, जिसे एक अन्य Rust library से, और उन्हीं databases पर Plaso, Velociraptor
(SQLiteHunter) और Hindsight से तुलना करने के बाद चुना गया। `parse-sqlite browser` की तुलना `sqlite3` library
से की गई, जो हर database की एक copy को उसकी `-wal` file के साथ पढ़ती है; यह तुलना 38 सार्वजनिक databases पर हुई:
Plaso की test files (Chrome 8 से 68, Firefox 2 से 118), Chromium 150 और Firefox 140 द्वारा बनाई गई profiles, और
ys-browser-parser के form history test databases। सभी 293 visits, 21 downloads, 692 cookies और 41 form entries
मेल खाईं। Deleted entries केवल वहीं recover हुईं जहाँ entries सचमुच delete की गई थीं: Firefox 140 history से
delete किए गए दो pages और दो visits, उनके समय, URLs और titles के साथ, और test databases की deleted form entries।

## LNK शॉर्टकट { #lnk-shortcuts }

[ys-lnk-parser](https://github.com/Yamato-Security/ys-lnk-parser)
Sootmark के [shell](https://github.com/Sootmark/shell) crate का अनुरक्षित fork है। इसकी तुलना 1,118 सार्वजनिक
shortcuts पर की गई। संदर्भ parser के साथ साझा 30 CSV columns में, 16 अलग-अलग values भिन्न थीं:
क्षतिग्रस्त, truncated, placeholder, MTP या Control Panel items वाले 13 target paths; अज्ञात या truncated blocks वाली दो
extra-data-block lists; और एक MFT reference जिसे
संदर्भ parser ने अलग bytes से पढ़ा। Juntei क्षतिग्रस्त संरचनाओं को चुपचाप
normalize करने के बजाय उनकी रिपोर्ट करता है।

## Jump Lists { #jump-lists }

[ys-jump-lists-parser](https://github.com/Yamato-Security/ys-jump-lists-parser) Sootmark के
compound-file और Jump List कार्य का उपयोग करता है, जिसमें embedded links `ys-lnk-parser` द्वारा decode किए जाते हैं। 77 सार्वजनिक
Jump Lists में, सभी 13,057 साझा तुलना cells—100 ns तक के timestamps सहित—संदर्भ
parser से मेल खाए। जिन छह files को संदर्भ ने अस्वीकार किया (खाली custom lists, अनुपस्थित footer, या ऐसी file
जो Jump List नहीं थी), उन्हें Juntei ने या तो decode किया या स्पष्ट रूप से रिपोर्ट किया।

## ShimCache { #shimcache }

[ys-shimcache-parser](https://github.com/Yamato-Security/ys-shimcache-parser) Security Ronin के
[shimcache-forensic](https://github.com/SecurityRonin/shimcache-forensic) का अनुरक्षित fork है, जिसमें registry-hive और
transaction-log समर्थन है। 20 सार्वजनिक `SYSTEM` hives (24 control sets और 7,290 entries) तथा 17
स्वतंत्र AppCompatCache values पर, हर entry संदर्भ parser द्वारा उत्पन्न path, time, execution flag, flags और
position से मेल खाई।

## Microsoft Defender क्वारंटीन { #microsoft-defender-quarantine }

[ys-defender-history-parser](https://github.com/Yamato-Security/ys-defender-history-parser)
Sootmark के [defender](https://github.com/Sootmark/defender) crate का अनुरक्षित fork है।
Parser के test tooling द्वारा Defender के format में लिखा गया quarantine folder स्वतंत्र
`dissect.target` quarantine reader द्वारा सुसंगत रूप से decode किया जाता है।

## Registry से प्राप्त आर्टिफ़ैक्ट { #registry-derived-artifacts }

[ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser) की तुलना
[yarp](https://github.com/msuhanov/yarp) से allocated keys, values, dirty-hive transaction replay,
free-cell recovery, cell slack, hive bins से परे के डेटा, और replay द्वारा हटाए या बदले गए records के लिए की जाती है।
आर्टिफ़ैक्ट-विशिष्ट तुलनाओं में शामिल हैं:

- Windows 7 और dirty Windows 11 `SYSTEM` hives से services, values और failure actions;
- नौ सार्वजनिक `NTUSER.DAT` hives में yarp और libfwsi से decode किए गए RecentDocs;
- Windows 7–11, Server 2012 R2 और Server 2022 को कवर करने वाले सात सार्वजनिक `SOFTWARE` hives में
  yarp से स्वतंत्र रूप से decode किए गए scheduled tasks; और
- सात सार्वजनिक `SAM` hives में `dissect.target` से तुलना की गई SAM account संरचनाएँ।

संदर्भ tools, format
शोध, specifications और सार्वजनिक साक्ष्य स्रोतों की पूरी सूची के लिए [आभार](../acknowledgements.md) देखें।
