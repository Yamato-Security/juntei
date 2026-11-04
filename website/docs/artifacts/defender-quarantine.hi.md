# Microsoft Defender क्वारंटीन { #microsoft-defender-quarantine }

## क्वारंटीन में क्या होता है { #what-the-quarantine-contains }

Microsoft Defender क्वारंटीन किसी detection के बारे में metadata और उसके द्वारा quarantine किए गए resources
का encrypted content सुरक्षित रखता है। Juntei Entries, ResourceData और Resources records को आपस में जोड़कर
threat label, quarantine का समय, मूल logical और physical paths, फ़ाइल metadata, owner,
download-origin `Zone.Identifier`, hashes और integrity समस्याएँ report करता है।

यह artifact उन event logs से अधिक समय तक बचा रह सकता है जिन्होंने मूल रूप से detection/remediation दर्ज किया था। यह
isolated reverse engineering के लिए सटीक संदिग्ध bytes भी सुरक्षित रख सकता है।

<span class="evidence-strength">सर्वोत्तम निष्कर्ष:</span> Defender ने दर्ज किए गए समय पर नामित
threat/resource के लिए quarantine record बनाया। Product label malware का स्वतंत्र प्रमाण नहीं है,
और केवल quarantine होना पहले हुए निष्पादन को सिद्ध नहीं करता।

## स्थान और संग्रह { #location-and-collection }

```text
C:\ProgramData\Microsoft\Windows Defender\Quarantine\
  Entries\{GUID}
  ResourceData\<prefix>\<SHA-1>
  Resources\<prefix>\<SHA-1>
```

पूरी `Quarantine` directory एकत्र करें। मेल खाती content/record फ़ाइलों के बिना भी कोई entry उपयोगी
metadata है, लेकिन उससे फ़ाइल के bytes या गणना किए गए hashes प्राप्त नहीं हो सकते।

## सुरक्षा और विश्लेषक के लिए सावधानियाँ { #safety-and-analyst-cautions }

- Extract किया गया content संभवतः malicious है। Isolated analysis system का उपयोग करें और इसे कभी execute न करें।
- `ThreatName` उस समय का Defender का signature/classification है। False positives और बाद में
  पुनर्वर्गीकरण संभव हैं।
- Quarantine का समय remediation/quarantine का समय है—ज़रूरी नहीं कि download, निर्माण या निष्पादन का।
- Quarantine की गई फ़ाइल निष्पादन से पहले या उसके चलने के बाद detect हुई हो सकती है।
- मूल path और Zone.Identifier मूल्यवान हैं, लेकिन अनुपस्थित या हमलावर-नियंत्रित हो सकते हैं।
- Orphan records retention/cleanup, अधूरे collection या corruption के कारण हो सकते हैं।
- Juntei आकार, CRCs, IDs और hashes सत्यापित करता है; recover किए गए bytes पर भरोसा करने से पहले `Problems` की समीक्षा करें।

## Juntei से parse करें { #parse-with-juntei }

```text
Usage: juntei parse-file defender-quarantine <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find quarantine folders
  -f, --file <FILE>      Parse one Entries\{GUID} file

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
      --extract <DIR>       Extract dangerous content with a .quarantined extension
  -C, --clobber             Overwrite confirmed output targets
```

```bash
juntei parse-file defender-quarantine -d evidence/ -o quarantine.csv
juntei parse-file defender-quarantine -d evidence/ -t jsonl \
  | jq '{QuarantineTime, ThreatName, Path, SHA256: .Content.SHA256, Problems}'

# Only in an isolated malware-analysis environment
juntei parse-file defender-quarantine -d evidence/ -o quarantine.csv --extract extracted/
```

उदाहरण के लिए, `tool.exe` बन जाता है `tool.exe.quarantined`। Auxiliary streams के नाम में उनका type शामिल
होता है, जैसे `tool.exe.Zone.Identifier.quarantined`। Juntei execute permission भी हटा देता है।
Extension और permissions आकस्मिक निष्पादन की संभावना घटाते हैं, लेकिन recover किए गए bytes को फिर भी
malware ही मानना चाहिए।

## आउटपुट fields { #output-fields }

| फ़ील्ड | व्याख्या |
|---|---|
| `Record` | `resource`, बिना resources वाली entry, `orphan-content`, या `orphan-record`। Orphans अधूरे linkage/collection को उजागर करते हैं। |
| `QuarantineTime` | UTC में Defender quarantine/remediation का समय, निष्पादन का समय नहीं। |
| `ThreatName`, `ThreatID` | Defender classification और numeric ID। इसे vendor का निर्णय/संदर्भ मानें। |
| `ResourceType` | फ़ाइल, registry key, service, startup item, scheduled task, या अन्य resource श्रेणी। |
| `Path`, `PhysicalPath` | मूल logical/physical स्थान। प्राथमिक correlation pivot; सटीक वर्तनी सुरक्षित रखें। |
| `FileSize` | Resource metadata द्वारा दर्ज आकार। `ContentSize` और MFT से तुलना करें। |
| `CreationTime`, `LastAccessTime`, `LastWriteTime` | Quarantine किए गए resource के लिए capture किया गया metadata; quarantine के समय से अलग और timestomping के प्रति असुरक्षित। |
| `Attr` | दर्ज किए गए file attributes। Hidden/system status उपयोगी संदर्भ हो सकता है। |
| `Owner` | Recover किए गए security descriptor से SID; owner ज़रूरी नहीं कि execute करने वाला उपयोगकर्ता हो। |
| `ZoneID`, `ReferrerURL`, `HostURL` | `Zone.Identifier` से download-zone और URL values। मौजूद होने पर उत्पत्ति के मज़बूत सुराग; ADS अनुपस्थित/हटाया हुआ हो सकता है। |
| `ContentSize` | Recover किए गए primary content का वास्तविक आकार। दर्ज `FileSize` से तुलना करें। |
| `MD5`, `SHA1`, `SHA256` | Recover किए गए bytes से गणना किए गए। आधुनिक content पहचान के लिए SHA-256 का उपयोग करें; interoperability के लिए सभी रखें। |
| `Streams` | Recover किए गए stream के प्रकार/नाम/आकार; इनमें ADS, security data या registry hive शामिल हो सकते हैं। |
| `ResourceID` | Resource metadata/content records को जोड़ने वाला identifier। |
| `EntryID` | Quarantine entry GUID; Defender history/event साक्ष्य से भी जोड़ता है। |
| `ScanID` | Resources/actions को समूहित करने के लिए Defender scan identifier। |
| `QuarantineFolder` | Root source folder/host का भेद। |
| `EntryFile`, `ContentFile`, `ResourceRecordFile` | Row बनाने में उपयोग की गई सटीक स्रोत फ़ाइलें। सत्यापन के लिए आवश्यक। |
| `ExtractedFile` | `--extract` द्वारा लिखी गई primary quarantine फ़ाइल; जब कोई primary extraction नहीं हुआ तो खाली। Auxiliary streams इसके बगल में लिखी जाती हैं। `auto-parse -x` इसे अपनी output directory से देता है (`./HOST1/SecurityProducts/DefenderQuarantined/…`)। |
| `Problems` | Integrity mismatch, लापता content, ID असहमति, क्षति, या orphan संदर्भ। इसे दबाएँ नहीं। |

JSON/JSONL, CSV के जोड़े गए text के बजाय संरचित resource, stream सूची, record entries और सभी
संबंध विवरण सुरक्षित रखता है।

## Extraction का व्यवहार { #extraction-behavior }

जब quarantine entry मूल filename दर्ज करती है, तो Juntei उसी filename का उपयोग अंत में `.quarantined`
extension के साथ करता है। Orphaned या non-file resources के लिए यह content identifier का उपयोग करता है, और
filename टकराव सुलझाने के लिए identifier जोड़ता है। Windows में अमान्य filename characters बदल दिए जाते हैं। Security
और alternate streams primary फ़ाइल के बगल में लिखी जाती हैं। हर security stream को एक पठनीय
text report भी मिलती है। Juntei execute permission हटा देता है;
पहले से मौजूद भिन्न content `-C` के बिना overwrite नहीं किया जाता। ये नियंत्रण दुर्घटनाओं को घटाते हैं, लेकिन
bytes को सुरक्षित नहीं बनाते।

### Recover किए गए security descriptor को पढ़ना { #reading-the-recovered-security-descriptor }

`.security.quarantined` फ़ाइल में binary Windows security metadata होता है, इसलिए इसे
text editor में नहीं पढ़ा जा सकता। यह एक अलग backup stream है, executable की दूसरी copy नहीं। Juntei
extract करते समय इसे अपने-आप decode करता है, चाहे `parse-file defender-quarantine --extract` से हो या `auto-parse -x` से:

```text
tool.exe.quarantined              Original file content; treat as malware
tool.exe.security.quarantined     Original binary security descriptor, preserved unchanged
tool.exe.security_descriptor.txt  Readable ownership and permission report
```

Text filename वास्तविक binary नाम का अनुसरण करता है, किसी भी collision identifier सहित। Generate किया गया
text साधारण UTF-8 है और अन्य extract की गई फ़ाइलों जैसी ही overwrite सुरक्षा का उपयोग करता है। इसमें
binary filename, byte count और SHA-256 शामिल होते हैं, ताकि report को उसके स्रोत से मिलाया जा सके।

| Report फ़ील्ड | इसकी व्याख्या कैसे करें |
|---|---|
| Owner और primary group | Descriptor में दर्ज SIDs। Well-known SIDs को `BUILTIN\Administrators` या `NT AUTHORITY\SYSTEM` जैसे नाम मिलते हैं। अन्य SIDs unresolved रहते हैं; किसी local/domain account का नाम बताने के लिए केवल परिचित RID पर्याप्त नहीं है। Ownership यह सिद्ध नहीं करता कि फ़ाइल किसने बनाई या चलाई। |
| Control flags | मूल numeric flags और उनके नाम, जिनमें ACL की उपस्थिति, inheritance और protection शामिल हैं। |
| DACL | क्रमबद्ध access-control entries (ACEs): allow/deny type, account SID, permission mask, rights और inheritance flags। ये दर्ज entries हैं, किसी उपयोगकर्ता के token के लिए effective access की गणना नहीं। |
| SACL | दर्ज की गई audit या mandatory-integrity entries। Audit-success/failure flags configuration का वर्णन करते हैं, access के इतिहास का नहीं। SACL का अनुपस्थित होना यह स्थापित नहीं करता कि सारी auditing disabled थी। |
| Object GUIDs | Object-specific ACEs के लिए वैकल्पिक GUIDs, जिनमें object-specific rights को file rights के रूप में ग़लत label करने के बजाय numeric रूप में रखा जाता है। |
| SDDL | Owner, group और समर्थित ACL entries का Security Descriptor Definition Language प्रतिनिधित्व। Control flags अलग से देखें। यदि कोई घटक क्षतिग्रस्त या असमर्थित है तो Juntei आंशिक SDDL string नहीं देता। |
| Decode status और समस्याएँ | `complete` या `incomplete`, कारणों सहित। असमर्थित ACEs report में अपना raw hex बनाए रखते हैं। Extraction सफल होने पर मूल binary हमेशा रखी जाती है। Decode समस्याएँ quarantine record के `Problems` फ़ील्ड में भी दिखाई देती हैं। |

अनुपस्थित DACL, NULL DACL और खाली DACL अलग-अलग report किए जाते हैं। अनुपस्थित या NULL DACL कोई
discretionary access प्रतिबंध नहीं देता; खाली DACL में access देने वाला कोई ACE नहीं होता। अन्य Windows access
checks और privileges फिर भी मायने रखते हैं। Inherited और inherit-only ACEs स्पष्ट रूप से flag किए रहते हैं।

उदाहरण के लिए, Administrators को inherited full control देने वाली entry इस प्रकार दिख सकती है:

```text
Entry 1: ALLOW (type 0x00)
  Flags: 0x10 (INHERITED_ACE)
  Account: S-1-5-32-544 (BUILTIN\Administrators)
  Access mask: 0x001F01FF
  Rights: Full control (FILE_ALL_ACCESS)
```

Decoder Windows, macOS और Linux पर Windows APIs, Python या account lookup के बिना offline चलता है।
यह basic allow/deny/audit/alarm ACEs, object ACEs और mandatory-integrity labels को decode करता है। Conditional,
callback, resource-attribute और अन्य असमर्थित ACEs को स्पष्ट रूप से पहचाना जाता है और कच्चे रूप में सुरक्षित
रखा जाता है; उनकी access शर्तों का मूल्यांकन नहीं किया जाता। क्षतिग्रस्त descriptor अन्य streams के
extraction को नहीं रोकता।

## जाँच की रणनीति { #investigation-strategy }

1. `ThreatName`, `Path`, `QuarantineTime`, `EntryId` और hashes से शुरुआत करें।
2. `Problems` सत्यापित करें और दर्ज आकार की तुलना recover किए गए content से करें।
3. SHA-256 को अन्य hosts, EDR, sandbox और threat-intelligence data में pivot करें।
4. मूल path/समय को `$MFT`, USN Journal, Prefetch, Amcache और event logs से correlate करें।
5. Initial access की जाँच के लिए URL/zone fields का उपयोग करें, लेकिन browser/proxy/DNS साक्ष्य से सत्यापित करें।

## अधिक जानकारी { #further-details }

- [ys-defender-history-parser](https://github.com/Yamato-Security/ys-defender-history-parser)
- [Sootmark defender](https://github.com/Sootmark/defender)
- [dissect.target Defender plugin](https://docs.dissect.tools/en/latest/projects/dissect.target/plugins/os/windows/defender.html)
- [Microsoft: Microsoft Defender Antivirus remediation और quarantine](https://learn.microsoft.com/en-us/defender-endpoint/restore-quarantined-files-microsoft-defender-antivirus)
- [defender-dump](https://github.com/knez/defender-dump)
- [Microsoft: Security descriptors](https://learn.microsoft.com/en-us/windows/win32/secauthz/security-descriptors)
- [Microsoft: Security Descriptor String Format](https://learn.microsoft.com/en-us/windows/win32/secauthz/security-descriptor-string-format)
- [Microsoft: ACE strings](https://learn.microsoft.com/en-us/windows/win32/secauthz/ace-strings)
