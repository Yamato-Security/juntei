# ऐप्लिकेशन संगतता { #application-compatibility }

## Windows क्या रखता है { #what-windows-keeps }

ShimCache (`parse-registry shimcache`) और Amcache (`parse-registry amcache`) के अलावा,
Windows की application compatibility सुविधाएँ ये जानकारी
`Microsoft\Windows NT\CurrentVersion\AppCompatFlags` के अंतर्गत रखती हैं:

| स्रोत | Hive और key | यह क्या दिखाता है |
|---|---|---|
| `layers` | `SOFTWARE` (और `Wow6432Node`) और `NTUSER.DAT` `Layers` | Compatibility layers के साथ चलने के लिए सेट किए गए programs: `RUNASADMIN` (हमेशा administrator के रूप में), `WIN7RTM`, `HIGHDPIAWARE`, … |
| `pca_store` | `NTUSER.DAT` `Compatibility Assistant\Store` (Windows 8 और बाद के) | वे programs जिन्हें Program Compatibility Assistant ने उपयोगकर्ता को चलाते देखा |
| `pca_persisted` | `NTUSER.DAT` `Compatibility Assistant\Persisted` (Windows 7) | वही, Windows 7 पर |
| `cit_module` | `SOFTWARE` `CIT\Module\<module>` | वे programs जिन्होंने किसी tracked module को load किया, और उन्होंने आख़िरी बार ऐसा कब किया |
| `cit_system` | `SOFTWARE` `CIT\System` (Windows 7 से प्रारंभिक Windows 10 तक) | Compressed CIT databases में program paths और command lines |
| `installed_sdb` | `SOFTWARE` `InstalledSDB\{GUID}` | `sdbinst` से install किए गए custom shim databases: फ़ाइल, विवरण, install का समय |
| `custom_sdb` | `SOFTWARE` `Custom\<program>` | हर program पर लागू custom shim databases, उनके install समय के साथ |

<span class="evidence-strength">सर्वोत्तम निष्कर्ष:</span> layers और shim databases ऐसा
configuration हैं जो यह बदलता है कि कोई program कैसे चलता है; Program Compatibility Assistant और CIT entries
दिखाती हैं कि कोई program इस उपयोगकर्ता के रूप में या इस सिस्टम पर चला (या उसने कोई module load किया)।

Custom shim databases एक persistence और injection तकनीक हैं (MITRE ATT&CK T1546.011): जब भी कोई program
चलता है, shim उसमें एक DLL load कर सकता है या कोई अन्य program शुरू कर सकता है।

## स्थान { #location }

```text
C:\Windows\System32\config\SOFTWARE
  Microsoft\Windows NT\CurrentVersion\AppCompatFlags\Layers                <program> = <layers>
  Microsoft\Windows NT\CurrentVersion\AppCompatFlags\CIT\Module\<module>   <program> = FILETIME
  Microsoft\Windows NT\CurrentVersion\AppCompatFlags\CIT\System            <id> = compressed database
  Microsoft\Windows NT\CurrentVersion\AppCompatFlags\InstalledSDB\{GUID}   DatabasePath, …
  Microsoft\Windows NT\CurrentVersion\AppCompatFlags\Custom\<program.exe>  {GUID}.sdb = FILETIME
C:\Users\<user>\NTUSER.DAT
  Software\Microsoft\Windows NT\CurrentVersion\AppCompatFlags\Layers
  Software\Microsoft\Windows NT\CurrentVersion\AppCompatFlags\Compatibility Assistant\Store
  Software\Microsoft\Windows NT\CurrentVersion\AppCompatFlags\Compatibility Assistant\Persisted
```

`SOFTWARE`, हर `NTUSER.DAT` और उनके transaction logs, तथा shim database के path में नामित `.sdb` फ़ाइलें
(आमतौर पर `C:\Windows\AppPatch\Custom\`) एकत्र करें।

## विश्लेषक के लिए सावधानियाँ { #analyst-cautions }

- **CIT expansion सीमित है।** जो values 64 MiB से अधिक uncompressed आकार घोषित करती हैं, वे एक diagnostic
  के साथ कच्चे bytes के रूप में रखी जाती हैं। विकृत compression और आकार के mismatch report किए जाते हैं।


- **Program Compatibility Assistant entries के लिए run का समय नहीं।** `Store` data (`SACP`, जो
  `DataRaw` में रखा जाता है) में एक समय होता है, लेकिन Windows 8.1, 10 और 11 पर यह एक hive के हर program के लिए
  समान होता है, और अन्य सिस्टमों के लिए: यह वह समय नहीं है जब program चला। केवल `Store` key का last write
  सबसे हाल के बदलाव का समय बताता है।
- **Removable media पर programs** का नाम `SIGN.MEDIA=<id> <path>` होता है: `Path` media पर का path है,
  और `Details` में media id होती है।
- **CIT databases** LZNT1-compressed होते हैं और उनकी संरचना सार्वजनिक रूप से प्रलेखित नहीं है: Juntei
  हर database में पाए गए program paths और command lines को, प्रति database एक बार, बिना counts या
  समय के सूचीबद्ध करता है। Command line अपना अलग record है, अपने program से बंधा नहीं। Windows 10 और बाद के संस्करण
  यहाँ कम ही जानकारी रखते हैं।
- **Shim databases.** हर `custom_sdb` और `installed_sdb` record जाँचें: वैध software
  शायद ही कभी इन्हें install करता है। Shim क्या करता है, यह स्वयं `.sdb` फ़ाइल बताती है।
- किसी उपयोगकर्ता द्वारा सेट की गई **Layers** केवल उसी उपयोगकर्ता पर लागू होती हैं; `SOFTWARE` वाली सभी पर। `RUNASADMIN`
  program को हर बार elevation माँगने पर मजबूर करता है।

## Juntei से parse करें { #parse-with-juntei }

```text
Usage: juntei parse-registry app-compat <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SOFTWARE and NTUSER.DAT hives
  -f, --file <FILE>      Parse one SOFTWARE or NTUSER.DAT hive
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry app-compat -d evidence/ -o appcompat.csv
juntei parse-registry app-compat -d evidence/ -t jsonl \
  | jq 'select(.Source == "installed_sdb" or .Source == "custom_sdb")'
```

Summary में custom shim databases, layers वाले programs, और CIT modules के अंतिम उपयोग
सूचीबद्ध होते हैं।

## आउटपुट fields { #output-fields }

| फ़ील्ड | व्याख्या |
|---|---|
| `Host`, `SourcePath`, `HiveType` | Host, hive फ़ाइल, और `SOFTWARE` या `NTUSER.DAT`। |
| `User`, `UserSID` | User hive के profile folder का नाम, और सिस्टम की `ProfileList` से उसका SID। |
| `Source`, `Scope` | `layers`, `pca_store`, `pca_persisted`, `cit_module`, `cit_system`, `installed_sdb`, `custom_sdb`; `machine` या `user`। |
| `Path` | Program (value का नाम), CIT database में कोई path या command line, या वह program जिस पर shim database लागू होता है। |
| `Layers` | Compatibility layers, जैसी लिखी गई हैं (`~ RUNASADMIN`)। |
| `LastSeenUTC` | किसी program ने आख़िरी बार CIT module कब load किया। |
| `SdbGUID`, `SdbPath`, `SdbInstalledUTC` | Shim database का GUID, फ़ाइल और install का समय। |
| `Details` | CIT module का नाम, CIT database का value नाम, program की removable media id (`SignMedia`), shim database का `DatabaseDescription` और `DatabaseType`। |
| `KeyPath`, `KeyLastWrittenUTC`, `ValueName`, `WOW64` | Record कहाँ से आता है, वह key आख़िरी बार कब बदली, और क्या वह `Wow6432Node` के अंतर्गत है। |
| `DataRaw` | `Store` value का `SACP` data, या ऐसा data जो अपेक्षित रूप में नहीं है (hex)। |
| `TransactionLogStatus`, `LogsApplied` | Registry recovery का संदर्भ। |
| `Problems` | ऐसा data जिसे decode या पढ़ा नहीं जा सका। |

## जाँच की रणनीति { #investigation-strategy }

1. हर custom shim database की समीक्षा करें: उसकी `.sdb` फ़ाइल, install का समय, और वे programs जिन पर वह लागू
   होता है; Prefetch, Amcache और process creation events में `sdbinst.exe` से correlate करें।
2. Program Compatibility Assistant entries में user-writable folders के programs, और
   removable media के programs खोजें।
3. असामान्य programs पर `RUNASADMIN` और अन्य layers जाँचें।
4. निष्पादन समय के लिए ShimCache, Amcache, BAM, Prefetch और UserAssist से correlate करें।

## अधिक जानकारी { #further-details }

- [Microsoft: Application compatibility database](https://learn.microsoft.com/en-us/windows/win32/devnotes/application-compatibility-database)
- [MITRE ATT&CK T1546.011: Application Shimming](https://attack.mitre.org/techniques/T1546/011/)
- [Microsoft: MS-XCA (LZNT1)](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-xca/)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
