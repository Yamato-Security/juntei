# Prefetch { #prefetch }

## Prefetch क्या है { #what-prefetch-is }

Windows Prefetch फ़ाइलें ऐप्लिकेशन के startup को optimize करने के लिए उपयोग की जाने वाली जानकारी दर्ज करती हैं। किसी प्रोग्राम-विशिष्ट
`.pf` फ़ाइल में आमतौर पर executable का नाम और hash, run count, हाल के run times, volume
जानकारी, directories, और startup के दौरान संदर्भित फ़ाइलें संग्रहीत होती हैं।

समर्थित Windows versions के लिए, एक मान्य application Prefetch record आमतौर पर इस बात का मज़बूत साक्ष्य होता है
कि executable चला। यह event log के अलावा निष्पादन के सबसे उपयोगी आर्टिफ़ैक्ट में से एक है।

<span class="evidence-strength">सर्वोत्तम निष्कर्ष:</span> नामित executable दर्ज किए गए
हाल के run time(s) पर launch किया गया था, operating system के व्यवहार और parser की वैधता के अधीन।

## स्थान और संग्रह { #location-and-collection }

```text
C:\Windows\Prefetch\*.pf
```

Juntei format versions 17 से 31 (Windows XP से Windows 11) तक का समर्थन करता है, जिसमें compressed
Windows 8+ फ़ाइलें भी शामिल हैं। पूरी directory एकत्र करें ताकि executable-name hashes और पड़ोसी records की
तुलना की जा सके।

## विश्लेषक के लिए सावधानियाँ { #analyst-cautions }

- Windows पुराने versions पर केवल एक ऐतिहासिक run time और नए versions पर अधिकतम आठ संग्रहीत करता है।
  `RunCount` उपलब्ध timestamps की संख्या से अधिक हो सकता है।
- कुछ systems/roles पर Prefetch disabled, साफ़ किया गया, बाहर रखा गया या अनुपलब्ध हो सकता है। अनुपस्थिति इस बात का
  प्रमाण नहीं है कि कोई प्रोग्राम नहीं चला।
- Loaded-file lists startup dependencies और accesses का वर्णन करती हैं; ये सिद्ध नहीं करतीं कि हर सूचीबद्ध फ़ाइल
  दुर्भावनापूर्ण थी, जानबूझकर खोली गई थी, या पूरी पढ़ी गई थी।
- Prefetch header में executable का नाम लंबाई-सीमित होता है; जहाँ अद्वितीय रूप से संभव हो, Juntei loaded
  files से path resolve करता है।
- किसी `.pf` स्रोत फ़ाइल के file-system timestamps embedded run times से अलग होते हैं।
- `Findings` hunting के अवलोकन हैं। Installers और वैध administration अक्सर staging paths का उपयोग करते हैं।

## Juntei से parse करें { #parse-with-juntei }

```text
Usage: juntei parse-file prefetch <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find *.pf files
  -f, --file <FILE>      Parse one Prefetch file

Records:
  -T, --timeline  Write one row per retained execution time, sorted by time

Output:
  -o, --output <FILE>       Save every record
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-file prefetch -d Prefetch/ -o prefetch.csv
juntei parse-file prefetch -d Prefetch/ -T -o prefetch-timeline.csv
juntei parse-file prefetch -d Prefetch/ -t jsonl \
  | jq 'select(.ExecutableName? | test("powershell|rundll32"; "i"))'
```

## मानक CSV fields { #standard-csv-fields }

| Field | व्याख्या |
|---|---|
| `SourceFile` | Input `.pf`; hosts और duplicate executable नामों में अंतर करने के लिए इसे रखें। |
| `ExecutableName` | Header में executable का नाम। अधिकांश hunting के लिए case-insensitive; मूल वर्तनी सुरक्षित रखें। |
| `ExecutablePath` | जहाँ संभव हो, Juntei द्वारा resolve किया गया loaded path। इसकी पुष्टि `Files` और अन्य आर्टिफ़ैक्ट से करें। |
| `PrefetchHash` | वह hash जिसे Windows `.pf` फ़ाइल नाम में उपयोग करता है; यह cryptographic file-content hash नहीं है। |
| `FormatVersion` | Prefetch structure version; उपलब्ध fields और timestamp count निर्धारित करता है। |
| `RunCount` | Windows द्वारा दर्ज संचयी counter। आवृत्ति के लिए उपयोगी, सटीक इतिहास के लिए नहीं। |
| `LastRun` | सबसे हाल में रखा गया execution समय। |
| `PreviousRun1` … `PreviousRun7` | पहले के रखे गए execution times, नए से पुराने। जहाँ format/history में न हों वहाँ खाली। |
| `VolumeCount` | दर्ज स्रोत volumes की संख्या। |
| `VolumeDevicePaths` | Startup के दौरान उपयोग किए गए volumes के NT device paths। Serials से correlate करें। |
| `VolumeSerialNumbers` | Volume identifiers; removable या reformat किए गए drive के साक्ष्य को जोड़ने में उपयोगी। |
| `VolumeCreated` | दर्ज volume creation times, executable creation times नहीं। |
| `DirectoryCount` | दर्ज directory strings की संख्या। |
| `FileCount` | दर्ज loaded-file references की संख्या। |
| `Findings` | Juntei के अवलोकन, जैसे स्थानांतरित system binaries या संदिग्ध execution paths। मैन्युअल रूप से समीक्षा करें। |
| `Files` | उपलब्ध NTFS references के साथ loaded/संदर्भित फ़ाइलें। DLL/path pivot के लिए उत्कृष्ट, फ़ाइल की मंशा का प्रमाण नहीं। |
| `Dir` | Startup से जुड़े directory strings। तब उपयोगी जब अलग-अलग loaded-file paths अधूरे हों। |

## Timeline CSV fields (`-T`) { #timeline-csv-fields-t }

| Field | व्याख्या |
|---|---|
| `RunTime` | एक रखा गया Prefetch execution समय। |
| `ExecutableName`, `ExecutablePath` | प्रोग्राम की पहचान/path context। |
| `PrefetchHash` | एक ही executable नाम साझा करने वाले variants/paths को अलग करता है। |
| `RunCount` | Prefetch फ़ाइल से counter; हर timeline row पर दोहराया जाता है। |
| `SourceFile` | सत्यापन के लिए स्रोत `.pf`। |

JSON/JSONL volumes, directories, loaded files और NTFS file references को comma-joined CSV cells
के बजाय structured arrays के रूप में सुरक्षित रखता है।

## जाँच की रणनीति { #investigation-strategy }

1. रखे गए runs को timeline में जोड़ने के लिए `-T` का उपयोग करें।
2. Executable नाम और Prefetch hash पर समूह बनाएँ; एक ही नाम के अलग-अलग paths/hashes हो सकते हैं।
3. `ExecutablePath`, staging paths, स्थानांतरित system-binary findings और loaded DLLs की समीक्षा करें।
4. Target MFT references और file paths का मिलान `$MFT`/USN records से करें।
5. Run times को event logs, SRUM, network telemetry, services, tasks और user artifacts से correlate करें।

## अधिक जानकारी { #further-details }

- [ys-prefetch-parser](https://github.com/Yamato-Security/ys-prefetch-parser)
- [ForensicRS frnsc-prefetch](https://github.com/ForensicRS/frnsc-prefetch)
- [libyal libscca](https://github.com/libyal/libscca)
- [Security Ronin prefetch-forensic](https://github.com/SecurityRonin/prefetch-forensic)
