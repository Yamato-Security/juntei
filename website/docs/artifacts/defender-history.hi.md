# Microsoft Defender पहचान इतिहास { #microsoft-defender-detection-history }

## इतिहास में क्या दर्ज होता है { #what-the-history-records }

Microsoft Defender हर detection (पहचान) के लिए एक detection history फ़ाइल लिखता है। हर फ़ाइल में दर्ज होता है:

- threat का identifier, नाम, severity और category;
- Defender ने क्या किया और वह सफल हुआ या नहीं;
- threat चल रहा था या नहीं;
- संबंधित user और process, और किसने उसे detect किया;
- quarantine entry, यदि कोई हो;
- हर वह resource जिसमें threat पाया गया: फ़ाइलें, archive members, downloads, processes, registry values,
  services, command lines और behaviors।

Defender उन threats के लिए भी ये फ़ाइलें लिखता है जिन्हें उसने कभी quarantine नहीं किया: जिन्हें उसने केवल detect किया,
जिन्हें allow किया गया, और जिन्हें वह हटाने में विफल रहा। Windows Security का protection history और WMI की
`MSFT_MpThreatDetection` class (`Get-MpThreatDetection`) ये records दिखाते हैं। जब Orange
Cyberdefense ने किसी फ़ाइल में values बदलीं, तो WMI ने बदली हुई values report कीं।

<span class="evidence-strength">सर्वोत्तम निष्कर्ष:</span> Defender ने सूचीबद्ध resources में नामित
threat का detection, दर्ज status और समयों के साथ, record किया। Threat का नाम product का निर्णय है। केवल
detection record यह नहीं दिखाता कि threat चला, या कि उसे पूरी तरह हटा दिया गया।

## स्थान और संग्रह { #location-and-collection }

```text
C:\ProgramData\Microsoft\Windows Defender\Scans\History\Service\DetectionHistory\<nn>\<GUID>
```

हर फ़ाइल का नाम उसके `DetectionID` पर होता है। पूरा `DetectionHistory` folder एकत्र करें। Directory
input और `auto-parse` किसी `DetectionHistory` folder के अंदर, किसी भी गहराई पर, हर उस फ़ाइल को पढ़ते हैं जो
detection history फ़ाइल की तरह शुरू होती है। ऐसे folder के बाहर रखी copy के लिए `-f` का उपयोग करें। Defender
configure किए जा सकने वाले दिनों की संख्या के बाद अपने scan history से items हटा देता है (`Set-MpPreference` में
`ScanPurgeItemsAfterDelay`), इसलिए पुराने detections मौजूद न हों, ऐसा हो सकता है।

Quarantine एक अलग artifact है: [Defender क्वारंटीन](defender-quarantine.md) देखें। किसी
detection का `QuarantineID` quarantine output के `EntryID` जैसी ही वर्तनी (upper case, braces के बिना) का
उपयोग करता है, इसलिए दोनों को join किया जा सकता है।

## विश्लेषक के लिए सावधानियाँ { #analyst-cautions }

- Detection, remediation का प्रयास और सफल removal अलग-अलग states हैं। `ThreatStatus`
  (`Detected`, `Quarantined`, `Removed`, `Allowed`, `RemoveFailed`, …) को
  `ThreatStatusErrorCode` के साथ पढ़ें, जो अंतिम status परिवर्तन सफल होने पर 0 होता है।
- `ExecutionStatus` detection के समय Defender का दृष्टिकोण है (`Blocked`, `Allowed`, `Executing`,
  `NotExecuting`, `Unknown`)। यह न तो इस बात का प्रमाण है कि threat चला, और न इस बात का कि वह कभी नहीं चला।
  Prefetch, Amcache, event logs और process telemetry से पुष्टि करें।
- `ThreatName` उस समय का signature नाम है। False positives और बाद में पुनर्वर्गीकरण संभव हैं।
  EICAR जैसी test फ़ाइलें वास्तविक threats की तरह report होती हैं।
- `User` वह `DomainUser` value है जो Defender ने दर्ज की (`UserSource` `DomainUser` है)। यह
  खाली, कोई service account, या फ़ाइल बनाने वाले से भिन्न user हो सकता है।
- Defender एक threat के लिए कई फ़ाइलें लिख सकता है, और Juntei उनमें से हर एक को रखता है। दो फ़ाइलों में
  एक ही `DetectionID` भी हो सकता है, जैसा copy की गई या restore की गई फ़ाइल में होता है। `DetectionID` और
  `SourceFile` दोनों को साथ लेकर group करें।
- जिन values का अर्थ ज्ञात नहीं है, उन्हें रखा जाता है, उनकी व्याख्या नहीं की जाती: उनका section और position
  `UnknownValues` में होते हैं।

## Juntei से parse करें { #parse-with-juntei }

```text
Usage: juntei parse-file defender-history <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find detection history files
  -f, --file <FILE>      Parse one DetectionHistory\<nn>\<GUID> file

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
  -C, --clobber             Overwrite an existing output
```

```bash
juntei parse-file defender-history -d evidence/ -o defender-history.csv
juntei parse-file defender-history -d evidence/ -t jsonl \
  | jq '{InitialDetectionTime, ThreatName, ThreatStatus, ResourceType, Path, SHA256}'
```

CSV डिफ़ॉल्ट फ़ाइल format है। `-t csv`, `-t json` या `-t jsonl` के साथ `-o` न देने पर records
stdout पर लिखे जाते हैं। दोनों में से कोई विकल्प न देने पर केवल सारांश दिखता है। मौजूदा output को overwrite करने के
लिए `-C` जोड़ें। Output कभी भी किसी input फ़ाइल के ऊपर नहीं लिखा जाता, `-C` के साथ भी नहीं।

Juntei हर detection के हर resource के लिए एक record लिखता है। बिना resources वाले detection को खाली
resource fields वाला एक record मिलता है। एक detection के सभी records उसका `DetectionID` साझा करते हैं और
संग्रहीत क्रम में `ResourceIndex` द्वारा क्रमांकित होते हैं।

क्षति run को नहीं रोकती। बीच में कटी फ़ाइल क्षति से पहले पढ़ी गई values रखती है, और
`Problems` बताता है कि क्या लापता है। `DetectionHistory` folder में मौजूद कोई फ़ाइल जो detection
history फ़ाइल नहीं है, report करके छोड़ दी जाती है। बाकी फ़ाइलें फिर भी पढ़ी जाती हैं।

## आउटपुट fields { #output-fields }

| फ़ील्ड | व्याख्या |
|---|---|
| `Record` | `resource`, या बिना resources वाले detection के लिए `detection`। |
| `InitialDetectionTime` | जब Defender ने पहली बार threat को detect किया (UTC)। फ़ाइल में 0 होने पर खाली; जो value समय नहीं है, वह `UnknownValues` में रखी जाती है। |
| `LastThreatStatusChangeTime` | जब threat का status अंतिम बार बदला (UTC)। |
| `RemediationTime` | जब Defender ने threat का remediation किया (UTC)। यह एक प्रयास का समय है, removal का प्रमाण नहीं; `ThreatStatus` जाँचें। |
| `ThreatName`, `ThreatID` | Defender का signature नाम और numeric identifier (`Get-MpThreatCatalog`)। |
| `Severity`, `SeverityID` | `Low`, `Moderate`, `High`, `Severe`, या `Unknown`, संग्रहीत संख्या के साथ। |
| `Category`, `CategoryID` | Threat category, जैसे `TROJAN`, `TOOL` या `BEHAVIOR`, संग्रहीत संख्या के साथ। |
| `ThreatStatus`, `ThreatStatusID` | Threat के लिए Defender का status। जिस संख्या का कोई documented नाम नहीं है, वह खाली `ThreatStatus` के साथ रखी जाती है। |
| `ThreatStatusErrorCode` | अंतिम status परिवर्तन का error code; 0 का अर्थ है कि वह सफल रहा। |
| `ExecutionStatus`, `ExecutionStatusID` | Detect होते समय Defender ने threat को चलते हुए देखा या नहीं। |
| `DetectionSource`, `DetectionSourceTypeID` | किसने इसे detect किया: `Real-time`, `IOAV` (downloads और attachments), `User`, `System`, `AMSI`, और अन्य। |
| `User`, `UserSource` | Defender द्वारा दर्ज `DomainUser` value, और उसका आधार (`DomainUser`)। फ़ाइल में किसी user का नाम न होने पर खाली। |
| `Process` | संबंधित process (`ProcessName`); `Unknown` हो सकता है। |
| `ResourceIndex`, `ResourceCount` | इस resource की position, और detection में resources की संख्या। |
| `ResourceType` | `file`, `containerfile`, `webfile`, `process`, `regkeyvalue`, `service`, `CmdLine`, `behavior`, `amsi`, और अन्य, जैसा Defender उन्हें नाम देता है। |
| `Path` | Resource: एक path, एक archive member (`archive.zip->member`), एक registry value, एक command line या एक process। `webfile` अपना URL और download करने वाला process (`pid:…,ProcessStart:…`) भी बताता है। सटीक वर्तनी सुरक्षित रखें। |
| `ResourceFlags` | Hexadecimal में resource flags; इनका अर्थ documented नहीं है। |
| `TrackingStartTime` | Resource का `ThreatTrackingStartTime`, एक integer जिसे FILETIME (UTC) के रूप में पढ़ा जाता है, उसी तरह जैसे plaso किसी detection का समय तय करता है। |
| `SHA256`, `SHA1`, `MD5`, `TrackingSize` | Resource के tracking data से hashes और आकार। ये फ़ाइल को उस स्थिति में दर्शाते हैं जब Defender ने उसे scan किया था। |
| `TrackingID` | Resource का `ThreatTrackingId`। |
| `OtherTracking` | बाकी हर tracking value, संग्रहीत क्रम में `Key=Value` के रूप में, जैसे behavior detections के लिए `ThreatTrackingScanSource` या `ThreatTrackingBMContextRich`। |
| `QuarantineID` | Quarantine entry (`Quarantine\Entries\{GUID}`); फ़ाइल में शून्य GUID होने पर खाली। |
| `DetectionID` | Detection का GUID; यही फ़ाइल का नाम भी है। मेल न खाने पर `Problems` में report होता है। |
| `MagicVersion` | Header का `Magic.Version:1.2`; इसके बिना लिखी गई फ़ाइलों में खाली, जैसे Windows Server 2016 की कुछ फ़ाइलें। |
| `UnknownValues` | जिन values का अर्थ ज्ञात नहीं है, `position=value` के रूप में (उदाहरण के लिए `6.13=NT AUTHORITY\SYSTEM`), और status के बाद की counted list `2.list` के रूप में। |
| `SourceFile` | पढ़ी गई फ़ाइल, input directory के सापेक्ष, या `-f` के साथ उसका नाम। |
| `Problems` | क्षति, लापता values, अप्रत्याशित types, या मेल न खाने वाला फ़ाइल नाम। इसे दबाएँ नहीं। |

JSON और JSONL यही fields typed values के साथ रखते हैं: संख्याएँ संख्या ही रहती हैं और लापता values
`null` होती हैं। `OtherTracking` के बजाय उनमें `Tracking` होता है: हर tracking value, Defender की key,
उसके type और उसकी value के साथ। `UnknownValues` types सहित positions की सूची बन जाता है। `CountedList`
में counted list होती है। संग्रहीत FILETIME अपना raw tick count उस समय के बगल में रखता है जो उससे पढ़ा जाता है।

## Timeline व्यवहार { #timeline-behavior }

`auto-parse` records को `SecurityProducts/DefenderHistory.csv` (या `.json`/`.jsonl`) के रूप में सहेजता है और
उन्हें संयुक्त `-j` timeline में जोड़ता है। `super-timeline` हर resource के record पर
`InitialDetectionTime`, `LastThreatStatusChangeTime` और `RemediationTime` से events बनाता है, और event के
details में status, error code, execution status और detection source रखता है। `TrackingStartTime` integer के
रूप में संग्रहीत होता है, इसलिए वह केवल `--include-interpreted-times` के साथ event बनता है। फ़ाइल के अपने समय कभी
उपयोग नहीं किए जाते। बिना उपयोगी समय वाला record parse किए गए output में रहता है और बिना समय वाले record के रूप में
गिना जाता है।

## जाँच की रणनीति { #investigation-strategy }

1. `ThreatName`, `Path`, `SHA256`, `InitialDetectionTime` और `ThreatStatus` से शुरुआत करें।
2. किसी threat को हटाया गया मानने से पहले `ThreatStatusErrorCode` और `Problems` जाँचें।
3. Quarantine किए गए bytes recover करने के लिए `QuarantineID` को quarantine के `EntryID` से join करें।
4. Hashes और paths को `$MFT`, USN Journal, Prefetch, Amcache, browser history और event
   logs (Defender का operational log: event IDs 1116, 1117, 1118 और 1119) में pivot करें।
5. `webfile` resources के लिए initial access की जाँच हेतु URL और process start का उपयोग करें, और
   उन्हें browser, proxy और DNS साक्ष्य से सत्यापित करें।

## परीक्षित संस्करण { #tested-versions }

परीक्षण ऐसी सार्वजनिक फ़ाइलों का उपयोग करते हैं जिनके परिणाम अन्य tools स्वतंत्र रूप से पढ़ते हैं:

- Orange Cyberdefense का Windows 10 sample (Defender 4.18.24090.11), जिसे WMI द्वारा report की गई values से
  जाँचा गया।
- plaso की दो Windows test फ़ाइलें, जिन्हें plaso के `windefender_history` parser से जाँचा गया।
- DFIRArtifactMuseum की APT simulator VM से 14 फ़ाइलें।
- Header के version के बिना लिखी गई एक copy, जैसी Windows Server 2016 की कुछ फ़ाइलें होती हैं।
- बीच में कटी, खाली, या किसी अन्य प्रकार की फ़ाइलें।

## अधिक जानकारी { #further-details }

- [ys-defender-history-parser](https://github.com/Yamato-Security/ys-defender-history-parser)
- [Sootmark defender](https://github.com/Sootmark/defender)
- [Orange Cyberdefense wddh-parser](https://github.com/cert-orangecyberdefense/wddh-parser)
- [plaso Defender history parser](https://github.com/log2timeline/plaso/blob/main/plaso/parsers/windefender_history.py)
- [Velociraptor Windows.Applications.DefenderHistory](https://github.com/Velocidex/velociraptor-docs/blob/master/content/exchange/artifacts/Windows.Applications.DefenderHistory.yaml)
- [Microsoft: MSFT_MpThreatDetection class](https://learn.microsoft.com/en-us/previous-versions/windows/desktop/defender/msft-mpthreatdetection)
- [Microsoft: Microsoft Defender Antivirus event IDs](https://learn.microsoft.com/en-us/defender-endpoint/troubleshoot-microsoft-defender-antivirus)
