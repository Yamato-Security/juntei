# RecentDocs { #recentdocs }

## RecentDocs क्या है { #what-recentdocs-is }

Windows Explorer हर उपयोगकर्ता के `NTUSER.DAT` में फ़ाइलों और folders की most-recently-used सूचियाँ संग्रहीत करता है।
Root सूची सभी प्रकारों के हाल के items को कवर करती है; subkeys `.docx` जैसे extensions के अनुसार समूह बनाती हैं, साथ ही एक
`Folder` सूची भी होती है। `MRUListEx` क्रमांकित values का क्रम निर्दिष्ट करता है।

RecentDocs उन documents की पहचान के लिए उपयोगी है जिनके साथ उपयोगकर्ता ने interact किया, जिनमें वे फ़ाइलें भी शामिल हैं जो
बाद में हटा दी गईं या move की गईं। Values में वह shortcut नाम भी हो सकता है जिसे Windows ने उपयोगकर्ता के Recent
folder में उपयोग किया, जिसे LNK साक्ष्य से correlate किया जा सकता है।

<span class="evidence-strength">सर्वोत्तम निष्कर्ष:</span> Explorer ने उस उपयोगकर्ता की
RecentDocs सूची में item दर्ज किया। Key का समय केवल उस विशिष्ट सूची के सबसे हाल के item का समय बताता है।

## स्थान { #location }

```text
C:\Users\<user>\NTUSER.DAT
Software\Microsoft\Windows\CurrentVersion\Explorer\RecentDocs
```

हर प्रासंगिक profile के लिए `NTUSER.DAT` और उसके transaction logs, तथा उपयोगकर्ता की Recent
LNK directory एकत्र करें।

## विश्लेषक के लिए सावधानियाँ { #analyst-cautions }

- `MRUListEx` क्रम देता है, हर item के लिए timestamp नहीं।
- Key का last-write समय सबसे हाल के update से मेल खाता है; केवल MRU स्थिति 0 को ही इससे निकाला गया
  बचाव-योग्य `LastOpenedUTC` मिलता है।
- कोई item root सूची और किसी extension सूची दोनों में दिखाई दे सकता है। इसे दो बार खोलना न गिनें।
- Application और Explorer का व्यवहार भिन्न हो सकता है; access के कुछ तरीके RecentDocs को update नहीं कर सकते।
- Cleanup/privacy tools और सूची के आकार की सीमाएँ पुराने साक्ष्य हटा देती हैं।
- Filename हमलावर-नियंत्रित हो सकता है और content type का प्रमाण नहीं है।
- उपयोगकर्ता का अनुमान profile paths या shell-folder data से आता है; खाली/अस्पष्ट usernames की समीक्षा करें।

## Juntei से parse करें { #parse-with-juntei }

```text
Usage: juntei parse-registry recent-docs <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find NTUSER.DAT hives
  -f, --file <FILE>      Parse one NTUSER.DAT
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry recent-docs -f NTUSER.DAT -o recentdocs.csv
juntei parse-registry recent-docs -d evidence/ -t jsonl \
  | jq 'select(.Extension == ".docx" or .Extension == ".pdf")'
```

## आउटपुट fields { #output-fields }

| फ़ील्ड | व्याख्या |
|---|---|
| `Host` | जहाँ उपलब्ध हो, collection/system संदर्भ से निकाला गया host। |
| `User` | Path या shell-folder paths से अनुमानित profile owner; खाली होने पर अनिश्चितता बनाए रखें। |
| `SourcePath` | Input `NTUSER.DAT`। उपयोगकर्ताओं को मिलाते समय आवश्यक। |
| `Extension` | Sublist (`.docx`, `Folder` आदि); root RecentDocs सूची के लिए null। |
| `MRU-Position` | शून्य इस सूची में सबसे हाल का है। यह सापेक्ष क्रम है, बीता हुआ समय नहीं। |
| `ValueIndex` | `MRUListEx` द्वारा संदर्भित क्रमांकित registry value। केवल संरचनात्मक pivot। |
| `Filename` | Decode किया गया item नाम। पूरे path को LNK, Jump List, MFT या application data के माध्यम से correlate करें। |
| `LinkName` | Shell item में encode किया गया shortcut नाम, जो अक्सर उपयोगकर्ता की Recent directory में मिलता है। |
| `KeyLastWrittenUTC` | सूची की key आख़िरी बार बदलने का समय। सूची पर लागू होता है, हर row पर नहीं। |
| `LastOpenedUTC` | केवल MRU स्थिति 0 के लिए भरा जाता है, जहाँ key का समय उस व्याख्या का समर्थन करता है। |
| `ValueDataRaw` | जब decoding विफल हो या साक्ष्य को सत्यापन की आवश्यकता हो तो कच्चे bytes। |
| `TransactionLogStatus`, `LogsApplied` | क्या registry logs replay किए गए और कौन-से। |
| `Problems` | लापता MRU values, सूची में न दी गई values, ग़लत types, या decoding में क्षति। |

## जाँच की रणनीति { #investigation-strategy }

1. हर उपयोगकर्ता की root और extension सूचियों को अलग करें; एक ही item को सावधानी से deduplicate करें।
2. क्रम के लिए MRU स्थिति का और timestamp वाले दावे के लिए केवल `LastOpenedUTC` का उपयोग करें।
3. `LinkName`/`Filename` को Recent LNK, Jump List, `$MFT` और USN data से मिलाएँ।
4. Decoy नाम, double extensions, network/removable targets, और initial access के आसपास के documents की समीक्षा करें।
5. जब निष्पादन मायने रखता हो, तो application के Prefetch/SRUM और event logs से correlate करें।

## अधिक जानकारी { #further-details }

- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
- [libyal libfwsi](https://github.com/libyal/libfwsi)
- [SANS Windows Forensic Analysis poster](https://www.sans.org/posters/windows-forensic-analysis/)
