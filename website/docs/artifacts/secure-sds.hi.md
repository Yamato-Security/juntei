# सुरक्षा विवरणक (`$Secure:$SDS`) { #security-descriptors-securesds }

## `$SDS` क्या है { #what-sds-is }

NTFS पर हर फ़ाइल और directory का एक **security descriptor** होता है: उसका owner, उसका primary group, एक
**DACL** (कौन उसके साथ क्या कर सकता है) और एक **SACL** (किस access का audit होता है, और उसका integrity label)।
NTFS हर विशिष्ट descriptor को एक बार, `$Secure` फ़ाइल (MFT entry 9) की `$SDS` stream में रखता है,
और उसे एक **security ID** देता है। किसी फ़ाइल का `$STANDARD_INFORMATION` केवल वह security ID दर्ज करता है:
[`parse-ntfs mft`](mft.md) का `SecurityID` column। `$SDS` इसे एक owner और ACLs में बदलता है।

`$SDS` 256 KiB blocks में होता है, जिनमें से हर एक के बाद उसकी एक copy (उसका mirror) होती है। हर entry में
उसके descriptor का एक hash होता है, जिसका उपयोग NTFS पहले से मौजूद descriptor खोजने के लिए करता है (`$SDH`), और
security ID होता है (`$SII` इसे index करता है)।

<span class="evidence-strength">सर्वोत्तम निष्कर्ष:</span> जिन फ़ाइलों का `SecurityID` किसी
descriptor का नाम देता है, उनके पास, जब उनका `$MFT` record आख़िरी बार लिखा गया था, वही owner और वे ACLs थे। किसी फ़ाइल का
owner आमतौर पर वह account होता है जिसने उसे बनाया, लेकिन कोई administrator, installer, या ownership लेने वाला
program इसे बदल सकता है।

## स्थान और संग्रह { #location-and-collection }

```text
C:\$Secure:$SDS   (MFT entry 9, the $SDS data stream)
```

इसे raw-aware forensic tool से एकत्र करें; collections आमतौर पर इसका नाम `$Secure_$SDS` रखते हैं, क्योंकि `:`
file name में नहीं हो सकता। इसके साथ उसी volume का `$MFT` एकत्र करें।

## विश्लेषक के लिए सावधानियाँ { #analyst-cautions }

- Owner एक SID है। केवल well-known SIDs (`NT AUTHORITY\SYSTEM`, `BUILTIN\Administrators`,
  `NT SERVICE\TrustedInstaller`, …) और किसी machine या domain के built-in accounts (`-500`
  `Administrator`, …) के नाम दिए जाते हैं। अन्य को host के `SAM`
  ([SAM खाते](sam.md)), उसकी profile सूची ([सिस्टम जानकारी](system-info.md)), या domain से resolve करें।
- एक descriptor समान owner और ACLs वाली हर फ़ाइल द्वारा साझा किया जाता है, और आमतौर पर उन फ़ाइलों के हटाए जाने के बाद भी
  `$SDS` में बना रहता है। `$MFT` में फ़ाइलों के बिना कोई descriptor अपने-आप में हटाई गई फ़ाइल नहीं है।
- ACE access का इतिहास नहीं है। DACL बताता है कि कौन फ़ाइल तक *पहुँच सकता है*; event logs और अन्य
  artifacts बताते हैं कि किसने पहुँच की। SACL की audit entries configuration का वर्णन करती हैं, न कि क्या log हुआ।
- ACEs ACL में अपने क्रम में सूचीबद्ध होते हैं। Juntei effective access की गणना नहीं करता: deny
  entries, inheritance, group membership, privileges और owner के implicit rights सभी मायने रखते हैं।
- `DACL` और `SACL` हर ACE को वैसे ही लिखते हैं जैसे `icacls` लिखता है। `SDDL` समर्थित permissions को दर्शाता है, जिसमें masks hexadecimal में और accounts SIDs के रूप में होते हैं। यह binary descriptor की byte-for-byte copy नहीं है; जिस conditional text को दर्शाया नहीं जा सकता, वह इसे खाली छोड़ देता है और उसका diagnostic दिया जाता है।

## Juntei से parse करें { #parse-with-juntei }

```text
Usage: juntei parse-ntfs secure-sds <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find $SDS streams (with $SDS in their name, or starting with an $SDS entry)
  -f, --file <FILE>      Parse one $Secure:$SDS stream (often exported as $Secure_$SDS)

Output:
  -o, --output <FILE>       Save every security descriptor
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-ntfs secure-sds -f '$Secure_$SDS'
juntei parse-ntfs secure-sds -d evidence/ -o sds.csv
juntei parse-ntfs secure-sds -f '$Secure_$SDS' -t jsonl
```

Summary descriptors गिनती है और उनके owners को सूचीबद्ध करती है, साथ में यह कि हर owner कितने descriptors का स्वामी है,
NULL DACL वाले descriptors (कोई प्रतिबंध नहीं), और कोई भी क्षति।

## CSV आउटपुट fields { #csv-output-fields }

| फ़ील्ड | व्याख्या |
|---|---|
| `SourcePath` | Input फ़ाइल। |
| `SecurityID` | Security ID: इसे उसी volume के लिए [`parse-ntfs mft`](mft.md) के `SecurityID` से join करें। |
| `Hash` | Descriptor के साथ दर्ज hash। |
| `EntryOffset` | Stream में entry कहाँ है। |
| `FromMirror` | Mirror block से पढ़ी गई copy के लिए `True`: ऐसी copy जो अपनी entry से भिन्न है (उसके बाद सूचीबद्ध), या किसी क्षतिग्रस्त entry की copy। |
| `Size` | Entry का आकार, उसके 20-byte header सहित। |
| `Owner`, `OwnerName` | Owner का SID, और well known होने पर उसका नाम। |
| `Group`, `GroupName` | Primary group का SID, और well known होने पर उसका नाम। Windows इसका उपयोग access checks के लिए नहीं करता। |
| `Control` | Descriptor के control flags: `SE_DACL_PRESENT`, `SE_DACL_PROTECTED` (inheritance अवरुद्ध), `SE_DACL_AUTO_INHERITED`, … `|` से जुड़े हुए। |
| `DACL-ACE-Count`, `DACL` | DACL की entries, `; ` से अलग की गई। कोई DACL न होने पर खाली; यदि descriptor सभी को सब कुछ देता है तो `NULL DACL: no restriction`। |
| `SACL-ACE-Count`, `SACL` | SACL की entries: audit entries, mandatory integrity label, process trust labels, और resource attributes। |
| `SDDL` | पूरा descriptor SDDL में। केवल तभी खाली जब उसमें कुछ decode न हुआ हो। |
| `Problems` | ऐसा hash जो descriptor का नहीं है, ऐसे भाग जो decode नहीं किए जा सकते, ऐसे ACEs जिनका type ज्ञात नहीं है। |

`DACL` और `SACL` की हर entry वैसे ही लिखी जाती है जैसे `icacls` लिखता है: account, type यदि वह
allow entry नहीं है, inheritance flags, और rights।

| भाग | अर्थ |
|---|---|
| Account | Well-known नाम, या SID। |
| `(DENY)`, `(AUDIT)`, `(ACCESS_ALLOWED_CALLBACK)`, … | ACE type, जब वह allow entry न हो। Mandatory label का कोई type नहीं दिखाया जाता। |
| `(I)` | Parent directory से inherited। |
| `(OI)`, `(CI)`, `(IO)`, `(NP)` | फ़ाइलों द्वारा inherited (object inherit), directories द्वारा (container inherit), केवल उनके द्वारा और इस object द्वारा नहीं (inherit only), और केवल एक स्तर नीचे तक (no propagate)। |
| `(SA)`, `(FA)` | सफल या विफल access के लिए audit entry। |
| `(F)`, `(M)`, `(RX)`, `(R)`, `(W)` | Full control, modify, read and execute, read, write। |
| `(RD,WD,AD,…)` | अन्यथा विशिष्ट rights: read data, write data, append data, extended attributes पढ़ना और लिखना (`REA`, `WEA`), execute (`X`), delete child (`DC`), attributes पढ़ना और लिखना (`RA`, `WA`), delete (`DE`), read control (`RC`), write DAC (`WDAC`), write owner (`WO`), synchronize (`S`), और generic rights `GA`, `GR`, `GW`, `GE`। `N` का अर्थ है कोई access नहीं। |
| `(NW)`, `(NR)`, `(NX)` | Mandatory label की policy: no write up, no read up, no execute up। |
| `if (…)` | Callback ACE की शर्त, जैसे `(WIN://SYSAPPID Contains "…")`: entry केवल उन processes पर लागू होती है जिनके लिए यह सत्य है। Windows इनका उपयोग app container packages को access देने के लिए करता है। |

Process trust labels और resource attributes अपना mask hexadecimal में दिखाते हैं, और resource
attribute अपना claim भी (`("IMAGELOAD",TU,0x0,1)`)।

JSON/JSONL में वही fields और नाम होते हैं, numbers numbers के रूप में, खाली values के लिए `null` और
`Problems` एक सूची के रूप में।

## जाँच की रणनीति { #investigation-strategy }

1. उसी volume के `$MFT` को पार्स करें और उसके `SecurityID` को इस output के security ID से join करें: हर फ़ाइल को उसका
   owner और ACLs मिलते हैं।
2. उन owners को देखें जो user accounts हैं (`S-1-5-21-…`): कोई संदिग्ध account किन फ़ाइलों का
   स्वामी है, और कौन-से accounts system directories में फ़ाइलों के स्वामी हैं।
3. उन directories पर permissive ACLs खोजें जहाँ से programs चलते या load होते हैं: `(F)`, `(M)` या `(W)` के साथ `Everyone`,
   `Authenticated Users` या `User`, और NULL DACLs।
4. केवल एक फ़ाइल द्वारा, या फ़ाइलों के किसी असामान्य समूह द्वारा उपयोग किया गया descriptor ऐसी फ़ाइलें दिखा सकता है जो
   सामान्य installation के बाहर बनाई या बदली गईं।
5. Ownership और permission बदलावों की पुष्टि Security event log entries (4670, SACL के साथ 4663)
   और [USN जर्नल](usn-journal.md) के `SECURITY_CHANGE` records से करें।

## अधिक जानकारी { #further-details }

- [Microsoft: Security descriptors](https://learn.microsoft.com/en-us/windows/win32/secauthz/security-descriptors)
- [Microsoft Open Specifications: MS-DTYP](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-dtyp/) (security descriptors, ACEs, conditional expressions)
- [Microsoft: Security Descriptor String Format (SDDL)](https://learn.microsoft.com/en-us/windows/win32/secauthz/security-descriptor-string-format)
- [Microsoft: icacls](https://learn.microsoft.com/en-us/windows-server/administration/windows-commands/icacls)
- [libfsntfs: NTFS format documentation](https://github.com/libyal/libfsntfs/blob/main/documentation/New%20Technologies%20File%20System%20(NTFS).asciidoc) (`$Secure`)
- [ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) (इसका `secure` crate)

जब structured output stdout पर हो (`-o` के बिना `-t`), तो stream-स्तर की recovery चेतावनियाँ stderr पर जाती हैं, ताकि pipes क्षतिग्रस्त या recover किए गए क्षेत्रों को छिपाए बिना पठनीय CSV/JSON/JSONL बनाए रखें।
