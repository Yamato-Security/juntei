# ShimCache (AppCompatCache) { #shimcache-appcompatcache }

## ShimCache क्या है { #what-shimcache-is }

Windows का Application Compatibility subsystem compatibility/shim व्यवहार का मूल्यांकन करते समय executable paths और metadata
cache करता है। यह cache फ़ाइलों के rename होने, हटाए जाने, या removable volume के साथ disconnect होने के बाद भी
बचा रहता है। इसका layout और उपलब्ध fields Windows XP से Windows 11 तक अलग-अलग होते हैं।

ShimCache ऐतिहासिक paths की खोज के लिए मूल्यवान है और, जहाँ format में विश्वसनीय execution
flag होता है, निष्पादन के लिए अतिरिक्त समर्थन देता है। Cache किया गया समय सामान्यतः **फ़ाइल का last-modified समय** होता है,
न कि वह समय जब Windows ने इसे देखा या चलाया।

<span class="evidence-strength">सर्वोत्तम निष्कर्ष:</span> Windows ने compatibility processing के लिए
path को देखा। केवल एक entry उपस्थिति/अवलोकन का साक्ष्य है; `Executed` का उपयोग केवल वहीं करें जहाँ यह समर्थित हो।

## स्थान और संग्रह { #location-and-collection }

Bytes `SYSTEM` registry hive में होते हैं, आमतौर पर इसके अंतर्गत:

```text
ControlSet00N\Control\Session Manager\AppCompatCache\AppCompatCache
```

`SYSTEM`, `SYSTEM.LOG1` और `SYSTEM.LOG2` एकत्र करें। Juntei सभी control sets पढ़ता है और जहाँ संभव हो
`Select\Current` की पहचान करता है।

!!! warning "सबसे नई live entries शायद hive में न हों"
    Windows ShimCache updates को memory में रख सकता है और उन्हें shutdown/reboot के दौरान flush कर सकता है। चलते हुए
    सिस्टम से copy किए गए hive में शायद सबसे नए अवलोकन न हों। Cache का registry last-write
    समय यह बताने में मदद करता है कि इसे कब सहेजा गया, लेकिन यह प्रति-entry निष्पादन समय नहीं है।

## विश्लेषक के लिए सावधानियाँ { #analyst-cautions }

- अपने-आप में एक cache entry **निष्पादन का प्रमाण नहीं** है।
- `Executed` `Yes`, `No` या `NA` हो सकता है; उपलब्ध अर्थ format/version पर निर्भर करते हैं।
- `LastModifiedTimeUTC` संदर्भित फ़ाइल का है, अवलोकन या निष्पादन का नहीं।
- क्रम उपयोगी है लेकिन सार्वभौमिक सटीक timeline नहीं; control sets/slack में duplicates होते हैं।
- `FromSlack` entries सक्रिय cache के बाहर से recover की गई थीं और अतिरिक्त सावधानी की माँग करती हैं।
- उपयोग योग्य logs के बिना dirty `SYSTEM` वर्तमान records छोड़ सकता है। Recovery summary पढ़ें।
- Findings संदिग्ध naming/path patterns दर्शाते हैं, malware के निर्णय नहीं।

## Juntei से parse करें { #parse-with-juntei }

```text
Usage: juntei parse-registry shimcache <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SYSTEM hives
  -f, --file <FILE>      Parse SYSTEM or a raw AppCompatCache value
      --no-logs          Do not replay SYSTEM.LOG1/SYSTEM.LOG2

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry shimcache -f SYSTEM -o shimcache.csv
juntei parse-registry shimcache -d evidence/ -t jsonl \
  | jq 'select(.Executed == true or (.Findings | length > 0))'
```

## आउटपुट fields { #output-fields }

| फ़ील्ड | व्याख्या |
|---|---|
| `ControlSet` | Cache रखने वाला numeric control set। किसी एक को चुनने से पहले `IsCurrent` की तुलना करें। |
| `CacheEntryPosition` | Decode किए गए cache में स्थिति/क्रम। सापेक्ष क्रम के लिए उपयोगी, समय नहीं। |
| `Path` | Cache किया गया executable/package path। प्राथमिक hunting pivot। |
| `LastModifiedTimeUTC` | Entry में संग्रहीत फ़ाइल modification समय—निष्पादन/अवलोकन का समय नहीं। |
| `Executed` | समर्थित format flags के अनुसार `Yes`, `No` या `NA`। केवल version संदर्भ के साथ उपयोग करें। |
| `Duplicate` | वही entry पहले के किसी control set या slack में पहले ही देखी जा चुकी थी; दोहरी गिनती से बचने में मदद करता है। |
| `SourceFile` | Input hive/raw value। Hosts के बीच आवश्यक। |
| `FromSlack` | सक्रिय सूची के बजाय cache-value slack से recover की गई entry। कम विश्वसनीयता और संभवतः पुरानी। |
| `CacheFormat` | Decode किया गया Windows/cache layout। Fields की उपलब्धता और अर्थ निर्धारित करता है। |
| `CacheLastWrittenUTC` | Registry key/cache के सहेजे जाने का समय। यह साझा संदर्भ है, हर entry का run समय नहीं। |
| `InsertFlags`, `ShimFlags` | कच्चे compatibility flags; जहाँ समर्थित हो, `Executed` Juntei द्वारा व्याख्यायित परिणाम है। |
| `DataSize` | इस entry के data का आकार, संरचनात्मक सत्यापन के लिए उपयोगी। |
| `Machine` | नए formats द्वारा दर्ज architecture (`x64`, `x86`, `ARM64`)। |
| `FileSize`, `LastUpdateTimeUTC` | जहाँ मौजूद हो, Windows XP-विशिष्ट cache किया गया metadata। |
| `PackageName`, `PackageVersion`, `PackagePublisherID` | समर्थित entries में packaged-application की पहचान। Publisher ID संदर्भ है, signature का प्रमाण नहीं। |
| `Findings` | Juntei anomaly codes, जैसे स्थानांतरित system binary या संदिग्ध staging path। केवल सुराग। |
| `RegistryPath`, `RegistryValueName` | सटीक hive key/value स्रोत। सत्यापन के लिए सुरक्षित रखें। |
| `CacheValueSize` | पूरी registry value का आकार, अलग entry का `DataSize` नहीं। |
| `IsCurrent` | क्या इस entry का control set `SYSTEM\Select\Current` के बराबर है; सुरक्षित रूप से ज्ञात न होने पर खाली। |
| `SelectCurrent`, `SelectDefault`, `SelectLastKnownGood`, `SelectFailed` | संदर्भ और recovery विश्लेषण के लिए कच्चे `SYSTEM\Select` control-set numbers। |

JSON/JSONL अतिरिक्त रूप से entry data, tags, packaged-app subrecords, संरचित findings,
cache समस्याएँ और recovery विवरण सुरक्षित रखते हैं।

## जाँच की रणनीति { #investigation-strategy }

1. वर्तमान control set को प्राथमिकता दें, लेकिन अन्य sets और slack को ऐतिहासिक साक्ष्य के रूप में रखें।
2. Paths और findings में खोजें; `LastModifiedTimeUTC` को निष्पादन समय के रूप में उपयोग न करें।
3. जहाँ `Executed == Yes` हो, Prefetch, SRUM, event logs या अन्य runtime निशानों से पुष्टि करें।
4. निर्माण, deletion और metadata इतिहास निर्धारित करने के लिए paths को `$MFT`/USN से मिलाएँ।
5. Report करें कि क्या transaction logs replay किए गए और क्या live cache शायद flush नहीं हुआ था।

## अधिक जानकारी { #further-details }

- [ys-shimcache-parser](https://github.com/Yamato-Security/ys-shimcache-parser)
- [Eric Zimmerman द्वारा AppCompatCacheParser](https://github.com/EricZimmerman/AppCompatCacheParser)
- [Security Ronin shimcache-forensic](https://github.com/SecurityRonin/shimcache-forensic)
- [Mandiant: Leveraging the Application Compatibility Cache in Forensic Investigations](https://www.mandiant.com/resources/blog/leveraging-application-compatibility-cache-forensic-investigations)
- [Maxim Suhanov द्वारा Windows registry format specification](https://github.com/msuhanov/regf)
