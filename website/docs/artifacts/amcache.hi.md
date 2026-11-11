# Amcache { #amcache }

## Amcache क्या है { #what-amcache-is }

Amcache एक Windows application-compatibility inventory है जो एक registry hive में संग्रहीत होती है। आधुनिक
सिस्टम executable फ़ाइलों को `InventoryApplicationFile` के अंतर्गत और Plug-and-Play devices को
`InventoryDevicePnp` के अंतर्गत दर्ज करते हैं; पुराने Windows संस्करण legacy `Root\File` layout का उपयोग करते हैं।

Executable की जाँच के लिए Amcache मूल्यवान है, क्योंकि मूल फ़ाइल के ग़ायब हो जाने के बाद भी यह पूरा path,
SHA-1, फ़ाइल का आकार, PE metadata, version जानकारी, publisher text, link date और registry timestamps
सुरक्षित रख सकता है। Device records जुड़े हुए hardware, drivers, installation तिथियों और identifiers की
पहचान कर सकते हैं।

<span class="evidence-strength">सर्वोत्तम निष्कर्ष:</span> Windows ने दर्ज की गई फ़ाइल या device को
inventory में शामिल किया। Executable record **उपस्थिति का साक्ष्य है, निष्पादन का प्रमाण नहीं**।

## स्थान और संग्रह { #location-and-collection }

```text
C:\Windows\AppCompat\Programs\Amcache.hve
```

प्रभावित सिस्टम से hive के साथ उसके पास मौजूद `Amcache.hve.LOG`, `.LOG1` और `.LOG2` फ़ाइलें भी एकत्र करें। Hive dirty होने पर `parse-registry amcache` विश्लेषण से पहले उपयोग योग्य transaction logs को मेमोरी में स्वतः replay करता है; साक्ष्य फ़ाइलें नहीं बदलतीं। Recovery विवरण parser log या terminal में मिलता है; records stdout पर भेजने पर diagnostics stderr पर जाते हैं। Logs गुम या अनुपयोगी होने पर records अधूरे रह सकते हैं या असंगत hive पढ़ा नहीं जा सकता।

## विश्लेषक के लिए सावधानियाँ { #analyst-cautions }

- Amcache की executable entry यह सिद्ध नहीं करती कि किसी उपयोगकर्ता ने फ़ाइल launch की।
- `Publisher` Windows द्वारा दर्ज किया गया PE/version-resource metadata है। यह सिद्ध नहीं करता कि
  collection के समय फ़ाइल पर वैध Authenticode signature था।
- Hashes अनुपस्थित हो सकते हैं, और legacy/आधुनिक schemas एक जैसे values उपलब्ध नहीं कराते।
- `LinkDate` PE header से आता है और इसे जाली बनाया, copy किया, या reproducible builds द्वारा उत्पन्न किया जा सकता है।
- Key का last-write समय registry record के update का वर्णन करता है, ज़रूरी नहीं कि download, निर्माण,
  पहले निष्पादन या अंतिम निष्पादन का।
- Paths removable या अब मौजूद न रहे volumes को संदर्भित कर सकते हैं।

## Juntei से parse करें { #parse-with-juntei }

```text
Usage: juntei parse-registry amcache <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find Amcache hives
  -f, --file <FILE>      Parse one Amcache.hve

Records:
      --files    Include every inventoried executable
      --devices  Include every inventoried PnP device
  -a, --all      Include both (default for saved structured output)

Output:
  -o, --output <PATH>       Save text/JSON/JSONL or a directory of CSV tables
  -t, --output-type <TYPE>  csv, json, jsonl, or text
```

```bash
# Inspect the summary and findings
juntei parse-registry amcache -f Amcache.hve

# Save all record types as CSV tables
juntei parse-registry amcache -f Amcache.hve -a -o amcache-output

# Hunt for one SHA-1 in JSONL
juntei parse-registry amcache -f Amcache.hve -t jsonl \
  | jq 'select(.RecordType == "file" and .SHA1 == "0123456789abcdef0123456789abcdef01234567")'
```

CSV आउटपुट में summary, findings, files और devices tables होती हैं। `auto-parse` के साथ ये
`Registry-Amcache-Summary.csv`, `Registry-Amcache-Findings.csv`, `Registry-Amcache-Files.csv` और
`Registry-Amcache-Devices.csv` होती हैं, जो हर कंप्यूटर की `FileExistence/` directory के ठीक नीचे बनती हैं, और
tables के बगल में `Registry-Amcache-logs.txt` होता है। कई hives होने पर extension से पहले `-Source1`, `-Source2`
आदि जोड़े जाते हैं; manifests में `OutputFiles` सटीक paths सूचीबद्ध करता है। Standalone CSV
exports अपने मूल table नाम और अनुरोधित output location बनाए रखते हैं। JSONL records
`RecordType` values `summary`, `finding`, `file` और `device` का उपयोग करते हैं।

## फ़ाइल आउटपुट fields { #file-output-fields }

| फ़ील्ड | विश्लेषक को इसका उपयोग कैसे करना चाहिए |
|---|---|
| `KeyName`, `SourceKeyPath` | सटीक स्रोत registry record और schema branch की पहचान करें। Reproducibility के लिए सुरक्षित रखें। |
| `KeyLastWrittenUTC`, `KeyLastWrittenFiletime` | Registry record के update का समय, पठनीय और कच्चे रूप में। इसे निष्पादन समय न कहें। |
| `SourceVolumeKeyLastWrittenUTC`, `SourceVolumeKeyLastWrittenFiletime` | Legacy schemas पर parent volume-key का समय; संदर्भात्मक, फ़ाइल पर हुई कार्रवाई का समय नहीं। |
| `FileID` | Amcache द्वारा दर्ज file identifier। बिल्कुल वैसा ही रखा जाता है; यह न मानें कि यह हमेशा SHA-1 होता है। |
| `SHA1`, `SHA256` | Defender, threat intelligence, EDR और file repositories में मज़बूत pivots। Null का अर्थ है दर्ज नहीं किया गया। |
| `Name`, `OriginalFileName`, `Description` | दर्ज किए गए PE/version नाम। Masquerading पकड़ने के लिए वास्तविक path से तुलना करें; metadata हमलावर-नियंत्रित होता है। |
| `FullPath` | स्थान के लिए सबसे उपयोगी pivot। Case को सावधानी से normalize करें, लेकिन मूल साक्ष्य string को बनाए रखें। |
| `LongPathHash` | Windows का path-संबंधी identifier; records का मिलान करने में उपयोगी, content hash नहीं। |
| `ProgramID` | जहाँ उपलब्ध हो, फ़ाइल को application inventory record से जोड़ता है। |
| `Publisher` | दर्ज किया गया publisher text। इसे metadata मानें, signature सत्यापन नहीं। |
| `Version`, `BinFileVersion`, `ProductName`, `ProductVersion`, `BinProductVersion` | Version-resource संदर्भ। अंतर या अविश्वसनीय values masquerading विश्लेषण का समर्थन कर सकते हैं। |
| `BinaryType` | दर्ज किया गया architecture/type संदर्भ; फ़ाइल recover होने पर PE विश्लेषण से तुलना करें। |
| `LinkDate` | दर्ज किया गया PE compile/link timestamp। केवल संदेह और पुष्टि के साथ उपयोगी। |
| `Size` | दर्ज किया गया आकार; `$MFT`, Defender content या recover की गई फ़ाइल से तुलना करें। |
| `Language` | PE language identifier; एक कमज़ोर संदर्भात्मक संकेत, attribution नहीं। |
| `IsPE-File`, `IsOS-Component` | Windows द्वारा दर्ज classification flags। False value हानिरहित होने का प्रमाण नहीं है। |
| `USN` | Amcache द्वारा दर्ज NTFS update sequence value; बचे हुए USN records तक pivot करने में मदद कर सकता है। |
| `APPX-PackageFullName`, `APPX-PackageRelativeID` | Packaged applications/components को फ़ाइल से जोड़ता है। |
| `RegistryValues` | हर मूल registry value, type, decode किए गए value और raw hex के साथ। जब typed field अनुपस्थित हो या भविष्य/legacy fields के लिए इसका उपयोग करें। |

## Device आउटपुट fields { #device-output-fields }

| फ़ील्ड समूह | Fields और व्याख्या |
|---|---|
| स्रोत | `KeyName`, `SourceKeyPath`, `KeyLastWrittenUTC`, `KeyLastWrittenFiletime`: सटीक device record और registry update का समय। |
| पहचान | `Description`, `BusDescription`, `Manufacturer`, `Model`: मनुष्यों के लिए device विवरण; vendor strings सामान्य हो सकती हैं। |
| Hardware identifiers | `HWID`, `CompID`, `Class`, `ClassGUID`, `ContainerID`, `DeviceInterfaceClasses`: device-family और instance के लिए सबसे मज़बूत pivots। |
| स्थिति | `DeviceState`, `ProblemCode`, `ParentID`, `MatchingID`, `Enumerator`: Windows enumeration और संबंधों का संदर्भ। |
| Driver | `DriverID`, `DriverName`, `DriverPackageStrongName`, `DriverVerDate`, `DriverVerVersion`, `Provider`, `Service`, `INF`: driver package और service से संबंध; install की गई फ़ाइलों और services से correlate करें। |
| Installation | `InstallDate`, `FirstInstallDate`, `InstallState`, `ExtendedInfs`: दर्ज किया गया install lifecycle। तिथि का न होना इस बात का प्रमाण नहीं कि device कभी जोड़ा ही नहीं गया। |
| Filters और stack | `LowerClassFilters`, `UpperClassFilters`, `LowerFilters`, `UpperFilters`, `StackID`, `LocationPaths`: उन्नत driver-stack संदर्भ; अप्रत्याशित filter drivers समीक्षा के योग्य हैं। |
| कच्चा संरक्षण | `RegistryValues`: सत्यापन और असमर्थित fields के लिए पूरे typed/कच्चे स्रोत values। |

CSV में array और raw-value columns `JSON` suffix का उपयोग करते हैं (उदाहरण के लिए, `HWID-JSON` और
`RegistryValuesJSON`); उनकी cells में JSON होता है, जबकि JSON/JSONL मूल arrays बनाए रखते हैं।

## जाँच की रणनीति { #investigation-strategy }

1. `FullPath`, `SHA1`, `Name` और `USN` पर pivot करें।
2. निष्पादन के समर्थन के लिए मेल खाता Prefetch खोजें।
3. फ़ाइल के निर्माण, rename, deletion और आकार को स्थापित करने के लिए `$MFT` और USN Journal का उपयोग करें।
4. Publisher/version metadata की तुलना स्वतंत्र रूप से recover किए गए PE और signature सत्यापन से करें।
5. Detection या removal के लिए Defender quarantine और event logs जाँचें।

## अधिक जानकारी { #further-details }

- [Juntei के ys-amcache-parser का output schema](https://github.com/Yamato-Security/ys-amcache-parser/blob/main/docs/output.md)
- [ys-amcache-parser](https://github.com/Yamato-Security/ys-amcache-parser)
- [Eric Zimmerman द्वारा AmcacheParser](https://github.com/EricZimmerman/AmcacheParser)
- [Security Ronin amcache-forensic](https://github.com/SecurityRonin/amcache-forensic)
- [Regipy Amcache plugin](https://github.com/mkorman90/regipy)
