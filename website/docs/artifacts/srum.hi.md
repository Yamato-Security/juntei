# SRUM { #srum }

## SRUM क्या है { #what-srum-is }

System Resource Usage Monitor (SRUM) applications और services के resource
उपयोग का एक ESE database संग्रहीत करता है। Windows संस्करण के आधार पर, यह network bytes, interface/profile identifiers,
connections, CPU time, foreground/background गतिविधि, disk गतिविधि, ऊर्जा उपयोग, push
notifications और application timeline डेटा दर्ज करता है। Entries `SruDbIdMapTable` के माध्यम से application IDs और उपयोगकर्ता
SIDs से जुड़ी होती हैं।

SRUM विशेष रूप से तब उपयोगी है जब कोई executable delete हो चुका हो: उसका application ID, उपयोगकर्ता और
resource-use rows file से अधिक समय तक बने रह सकते हैं। Network totals किसी exfiltration परिकल्पना का समर्थन कर सकते हैं, लेकिन
SRUM packet capture नहीं है और आमतौर पर हर remote endpoint या स्थानांतरित file की पहचान नहीं करता।

<span class="evidence-strength">सर्वोत्तम निष्कर्ष:</span> Windows ने table की measurement अवधि के दौरान दर्ज resource
उपयोग का श्रेय application/account को दिया।

## स्थान और संग्रह { #location-and-collection }

```text
C:\Windows\System32\sru\SRUDB.dat
C:\Windows\System32\config\SOFTWARE   (context for account and Wi-Fi names)
```

SRUM Windows 8 और बाद के संस्करणों पर मौजूद है। `SRUDB.dat` को सुसंगत रूप से copy करें, आदर्श रूप से snapshot से या
forensic collector से। `sru` directory में मौजूद `SOFTWARE` और ESE transaction/log files भी
संग्रहित करें।

!!! warning "Dirty databases"
    Juntei dirty `SRUDB.dat` को जैसा संग्रहित किया गया वैसा पढ़ता है और इसकी रिपोर्ट करता है। यह वर्तमान में
    `SRU*.log` को replay नहीं करता, इसलिए केवल उन logs में मौजूद बदलाव अनुपस्थित रहते हैं। किसी
    dirty database को चुपचाप पूर्ण के रूप में वर्णित न करें।

## विश्लेषक के लिए सावधानियाँ { #analyst-cautions }

- कई rows आवधिक aggregation दर्शाती हैं, आमतौर पर घंटे की सीमाओं के आसपास—सटीक क्रिया का समय नहीं।
- Byte totals अपने आप में content, इरादे, protocol या destination की पहचान नहीं करते।
- Application identity कोई path, service, packaged app या object हो सकती है; `AppType` की जाँच करें।
- उपयोगकर्ता और Wi-Fi नाम मेल खाने वाले `SOFTWARE` hive से enrichment हैं। नाम अनुपस्थित होने का अर्थ यह नहीं कि
  मूल ID अनुपस्थित है।
- `null`, zero, अनुपस्थित columns, असमर्थित types और क्षतिग्रस्त values अलग-अलग states हैं।
- Provider schemas Windows संस्करण के अनुसार भिन्न होते हैं। अज्ञात tables और मूल column names सुरक्षित रखें।
- Background Windows services वैध रूप से बड़े resource totals उत्पन्न करती हैं।

## Juntei से parse करें { #parse-with-juntei }

```text
Usage: juntei parse-ese srum <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SRUDB.dat
  -f, --file <FILE>      Parse one SRUDB.dat
  -r, --software <FILE>  Matching SOFTWARE hive for names

Output:
  -o, --output <PATH>        Directory of CSV tables, or JSON/JSONL file
  -t, --output-type <TYPE>   csv, json, or jsonl
      --include-value-state  Add per-value read state and raw bytes to JSON/JSONL
```

```bash
juntei parse-ese srum -f SRUDB.dat -r SOFTWARE -o srum-csv
juntei parse-ese srum -f SRUDB.dat -t jsonl \
  | jq 'select(.Provider == "NetworkUsages" and (.BytesSent // 0) > 100000000)'
juntei parse-ese srum -f SRUDB.dat -t jsonl --include-value-state \
  | jq '{Provider, Timestamp, ValueState, Raw}'
```

## आउटपुट tables { #output-tables }

CSV `SruDbIdMapTable` के लिए `SRUM-IdentityMap.csv` और प्रत्येक provider के लिए एक table बनाता है। ज्ञात आउटपुट नामों में शामिल हैं:

| Provider/table | जाँच का उद्देश्य |
|---|---|
| `NetworkUsages` | प्रत्येक application, उपयोगकर्ता, interface और profile के लिए भेजे/प्राप्त bytes। |
| `NetworkConnections` | Connection-count/duration प्रकार की गतिविधि और connection start times। |
| `AppResourceUseInfo` | Foreground/background CPU, disk, network और संबंधित application उपयोग। |
| `AppTimelineProvider` | End-time और usage-state fields के साथ application timeline गतिविधि। |
| `EnergyUsage` | अल्पकालिक और दीर्घकालिक energy/provider measurements; `IsLt` इनमें भेद करता है। |
| `PushNotifications` | Push-notification गतिविधि। |
| `VfuProvider` | `vfuprov` provider; इसका विस्तृत उद्देश्य अच्छी तरह documented नहीं है, इसलिए इसके fields की व्याख्या सावधानी से करें। |
| `EnergyEstimation`, `TaggedEnergy` | संस्करण-विशिष्ट energy और usage providers। |
| `ProcessorTime`, `SizeInBytes`, `TotalUsed`, `NetworkBytes` | Windows Server provider tables। |
| अज्ञात table नाम | Juntei अज्ञात providers को उनके GUID/table नाम के अंतर्गत सुरक्षित रखता है और हर catalog column आउटपुट करता है। |

चूँकि SRUM catalog-driven है और Windows provider columns जोड़ता रहता है, इसलिए जानबूझकर provider-विशिष्ट fields की कोई निश्चित
सीमित सूची नहीं है। Juntei **database में मौजूद हर column को Juntei के PascalCase field labels के साथ** आउटपुट करता है।
निम्नलिखित fields स्थिर/सामान्य आउटपुट अनुबंध हैं।

## वर्गीकृत auto-parse आउटपुट { #categorized-auto-parse-output }

`auto-parse` प्रत्येक SRUM table को सीधे उसकी जाँच श्रेणी में रखता है। CSV, JSON और JSONL
समान `SRUM-` filename prefix का उपयोग करते हैं; कोई आवरण `SRUM/` directory नहीं होती।

| श्रेणी | CSV files |
|---|---|
| `NetworkActivity` | `SRUM-NetworkConnections.csv`, `SRUM-NetworkUsages.csv`, `SRUM-NetworkBytes.csv`, `SRUM-PushNotifications.csv` |
| `ProcessExecution` | `SRUM-AppResourceUseInfo.csv`, `SRUM-AppTimelineProvider.csv`, `SRUM-ProcessorTime.csv` |
| `SystemInformation` | `SRUM-EnergyUsage.csv`, `SRUM-EnergyEstimation.csv`, `SRUM-TaggedEnergy.csv`, `SRUM-SizeInBytes.csv`, `SRUM-TotalUsed.csv`, `SRUM-IdentityMap.csv`, `SRUM-VfuProvider.csv` और अज्ञात-provider tables |

केवल capture में मौजूद tables export की जाती हैं। Network counters और notification records को
उनके मुख्य जाँच उपयोग के अनुसार रखा जाता है; इसका यह दावा नहीं है कि हर row किसी remote
connection का वर्णन करती है। Identity map कई श्रेणियों का समर्थन करता है। VFU और अज्ञात providers
`SystemInformation` में रहते हैं क्योंकि केवल उनकी उपस्थिति execution स्थापित नहीं करती।

`SystemInformation/SRUM-logs.txt` में parser diagnostics और मूल-से-अंतिम आउटपुट
mapping होती है। दोनों manifests `OutputFiles` में वास्तविक table paths सूचीबद्ध करते हैं; SRUM task का `Output`
computer directory है और `Category` उसके साझा log/metadata स्थान की पहचान करता है।

कई databases होने पर, filenames में extension से पहले `-Source1`, `-Source2` आदि जुड़ जाते हैं।
Log मूल capture aliases को सुरक्षित रखता है, और JSON/JSONL rows अपना `SourceFile`,
`Table`, `Provider`, `Comp` और `Artifact` provenance बनाए रखती हैं। कोई table कई
श्रेणियों में copy नहीं की जाती। संयुक्त `-j` SIEM timeline में हर रखा गया SRUM record एक बार शामिल होता है।

स्वतंत्र `parse-ese srum` अपने अनुरोधित आउटपुट स्थान को बनाए रखता है। इसकी CSV table files में भी
`SRUM-` prefix होता है; स्पष्ट JSON/JSONL आउटपुट filenames विश्लेषक द्वारा चुने गए ही रहते हैं।

## सामान्य आउटपुट fields { #common-output-fields }

| Field | व्याख्या |
|---|---|
| `SourceFile` | Input database; JSON/JSONL में मौजूद और कई hosts में उपयोगी। |
| `Table` | मूल ESE table नाम/GUID। संस्करणों या अज्ञात providers की तुलना करते समय सुरक्षित रखें। |
| `Provider` | सुगम provider/आउटपुट नाम, या अज्ञात होने पर मूल table नाम। |
| `ID` | Row का `AutoIncId`; database row identifier, process ID नहीं। |
| `Timestamp` | जब SRUM ने आँकड़े दर्ज/aggregate किए, milliseconds तक rounded। आमतौर पर सटीक event time नहीं। |
| `EXE-Info` | `AppID` से resolve की गई application/service/package पहचान। मुख्य program pivot। |
| `EXE-InfoDescription` | Composite SRUM application नामों से निकाला गया description। केवल संदर्भ। |
| `EXE-Timestamp` | मौजूद होने पर composite application identity में निहित समय; row का usage time नहीं। |
| `AppType` | `NormalApp`, `Service`, `ModernApp` या `Object`; यह नियंत्रित करता है कि `EXE-Info` की व्याख्या कैसे की जाए। |
| `SID` | `UserID` से resolve किया गया account SID। उपयोगकर्ता/service पहचान के लिए मज़बूत pivot। |
| `User` | `SOFTWARE` से समृद्ध सुगम profile नाम; `SID` मान्य रहते हुए भी null हो सकता है। |
| `UserID`, `AppID` | `SruDbIdMapTable` में raw संख्यात्मक references; resolve न हुए या duplicate नामों के लिए सुरक्षित रखें। |
| `IsLt` | दीर्घकालिक EnergyUsage table के लिए `true`, अल्पकालिक के लिए `false`। |
| `InterfaceLUID` | Provider rows से raw network interface LUID। |
| `InterfaceType` | Decode किया गया interface class (उदाहरण के लिए `IF_TYPE_IEEE80211`); class, कोई अद्वितीय adapter नहीं। |
| `L2ProfileID` | संख्यात्मक network profile reference। |
| `ProfileName` | `SOFTWARE` से समृद्ध Wi-Fi/profile नाम; संभावित रूप से संवेदनशील और वर्तमान connection का प्रमाण नहीं। |
| `ConnectStartTime`, `EventTimestamp`, `EndTime`, `StartTime` | UTC में decode किए गए ज्ञात provider FILETIME columns। इनका सटीक अर्थ provider पर निर्भर है, कोई सार्वभौमिक execution time नहीं। |
| Provider-विशिष्ट columns | Export किए गए provider columns, जैसे `BytesSent`, `BytesReceived` (database column `BytesRecvd`), CPU/disk counters, durations, counts, flags या estimates। Units की व्याख्या provider के अनुसार करें और एक ही schema के भीतर rows की तुलना करें। |

## Diagnostic fields (`--include-value-state`) { #diagnostic-fields-include-value-state }

| Field | व्याख्या |
|---|---|
| `ValueState` | Database field से `parsed`, `null`, `absent`, `unsupported` या `damaged` तक का map। यह साक्ष्य की अनुपस्थिति को parser विफलता से अलग करता है। |
| `Raw` | Bytes वाली values के लिए hexadecimal में मूल bytes। सत्यापन और भविष्य के decoding के लिए उपयोग करें। |

`SRUM-IdentityMap.csv` (`SruDbIdMapTable` से) में `IdIndex`, `IdType` और `IdBlob` होते हैं: raw index, identity श्रेणी,
और provider rows द्वारा उपयोग किया गया decode किया गया program/SID/object text।

## जाँच की रणनीति { #investigation-strategy }

1. `EXE-Info`, `AppType` और `AppID` से program की पहचान करें; `Id` को PID न समझें।
2. Network rows को program, उपयोगकर्ता, profile, interface और समय अंतराल के अनुसार group करें।
3. भेजे/प्राप्त bytes की तुलना सामान्य baselines और अन्य hosts से करें।
4. संदिग्ध अंतरालों को Prefetch, event logs, DNS/proxy/firewall डेटा और file access से correlate करें।
5. Dirty-database status और transaction-log replay की अनुपस्थिति को संग्रह सीमा के रूप में रिपोर्ट करें।

## अधिक जानकारी { #further-details }

- [ys-srum-parser](https://github.com/Yamato-Security/ys-srum-parser)
- [Eric Zimmerman का SrumECmd](https://github.com/EricZimmerman/Srum)
- [dissect.esedb](https://github.com/fox-it/dissect.esedb)
- [libyal libesedb](https://github.com/libyal/libesedb)
- [Microsoft: Extensible Storage Engine](https://learn.microsoft.com/en-us/windows/win32/extensible-storage-engine/extensible-storage-engine)
