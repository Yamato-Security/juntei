# हाल में उपयोग की गई (MRU) सूचियाँ (RunMRU, TypedPaths, WordWheelQuery, TypedURLs) { #most-recently-used-lists-runmru-typedpaths-wordwheelquery-typedurls }

## ये सूचियाँ क्या हैं { #what-the-lists-are }

Windows और उसके प्रोग्राम उपयोगकर्ता द्वारा सबसे हाल में टाइप की गई या खोली गई चीज़ों की छोटी सूचियाँ उपयोगकर्ता की
`NTUSER.DAT` में रखते हैं:

- **RunMRU:** Run dialog (Win+R) में टाइप किए गए commands: `cmd`, `powershell -ep bypass`,
  `\\server\share`, `regedit`।
- **TypedPaths:** Explorer के address bar में टाइप किए गए paths।
- **WordWheelQuery:** Explorer के search box में खोजे गए शब्द।
- **TypedURLs:** Internet Explorer के address bar में टाइप किए गए पते, और Windows 8 से
  `TypedURLsTime`, यानी हर पता कब टाइप किया गया।
- **Paint और WordPad:** हर एक द्वारा सबसे हाल में खोली गई फ़ाइलें (`Recent File List`)।
- **ACMru (Windows XP):** Search Assistant के खोज शब्द, खोज के प्रकार के अनुसार।

<span class="evidence-strength">सर्वोत्तम निष्कर्ष:</span> उपयोगकर्ता ने यह command, path,
खोज शब्द या पता टाइप किया, या प्रोग्राम ने यह फ़ाइल खोली। हर सूची की केवल सबसे हाल की entry
का समय उसकी key के last-write time से मिलता है; अपवाद टाइप किए गए पते हैं, जिनका समय `TypedURLsTime` देता है।

## स्थान { #location }

```text
C:\Users\<user>\NTUSER.DAT
Software\Microsoft\Windows\CurrentVersion\Explorer\RunMRU          a, b, … in MRUList order
Software\Microsoft\Windows\CurrentVersion\Explorer\TypedPaths      url1 (the most recent), url2, …
Software\Microsoft\Windows\CurrentVersion\Explorer\WordWheelQuery  0, 1, … in MRUListEx order
Software\Microsoft\Internet Explorer\TypedURLs                     url1, url2, …
Software\Microsoft\Internet Explorer\TypedURLsTime                 url1, url2, … (FILETIME)
Software\Microsoft\Windows\CurrentVersion\Applets\Paint\Recent File List    File1, File2, …
Software\Microsoft\Windows\CurrentVersion\Applets\Wordpad\Recent File List  File1, File2, …
Software\Microsoft\Search Assistant\ACMru\<kind>                   000, 001, … (Windows XP)
```

`ACMru` की subkeys खोज के प्रकार हैं: `5603` फ़ाइल और फ़ोल्डर के नाम, `5604` फ़ाइलों में शब्द या वाक्यांश,
`5647` कंप्यूटर और लोग, `5001` Internet।

हर profile के लिए `NTUSER.DAT` को उसके transaction logs के साथ एकत्र करें।

## विश्लेषक के लिए सावधानियाँ { #analyst-cautions }

- **हर सूची के लिए एक समय।** सूची बदलने पर key का last-write time बदलता है, इसलिए यह केवल
  सबसे हाल की entry का समय बताता है (`IsMostRecent`, `LastUsedUTC`)। बाकी entries क्रम में हैं, पर बिना समय के।
  `TypedURLsTime` हर टाइप किए गए पते का समय देता है (`TypedTimeUTC`)।
- **टाइप किया गया, चलाया गया नहीं।** RunMRU वह दर्ज करता है जो Run dialog में टाइप किया गया, चाहे वह
  सफलतापूर्वक चला हो या नहीं। Windows हर command को अंत में `\1` के साथ संग्रहीत करता है; Juntei उसे हटा देता है।
  Prefetch, BAM, UserAssist और process creation events से correlate करें।
- **सूचियाँ छोटी होती हैं।** हर सूची केवल अपनी सबसे हाल की entries रखती है, और privacy settings तथा cleanup
  tools उन्हें साफ़ कर देते हैं। हटाई गई values के लिए `parse-registry read --recover-deleted` का उपयोग करें।
- **शब्द वही हैं जो टाइप किए गए।** WordWheelQuery और ACMru टाइप किया गया text रखते हैं, न कि जो मिला।
- **क्रम सूची से आता है।** RunMRU का `MRUList` और WordWheelQuery का `MRUListEx` क्रम
  देते हैं; बाकी के लिए value की संख्या क्रम देती है (`url1`, `File1` और `000` सबसे हाल के हैं)।

## Juntei से parse करें { #parse-with-juntei }

```text
Usage: juntei parse-registry mru <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find NTUSER.DAT hives
  -f, --file <FILE>      Parse one NTUSER.DAT
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Filtering:
      --list <LISTS>  runmru, typedpaths, wordwheelquery, typedurls, paint, wordpad, acmru

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry mru -d evidence/ -o mru.csv
juntei parse-registry mru -f NTUSER.DAT --list wordwheelquery,typedpaths -t jsonl
juntei parse-registry mru -f NTUSER.DAT --list runmru -t jsonl \
  | jq 'select(.Value | test("powershell|cmd|mshta"; "i"))'
```

Summary हर सूची की entries और जिन entries का समय अंत में दर्ज हुआ (टाइप किए गए पते, और हर
सूची की सबसे हाल की entry) सबसे नई पहले क्रम में दिखाता है।

## आउटपुट fields { #output-fields }

| फ़ील्ड | व्याख्या |
|---|---|
| `Host`, `User`, `UserSID`, `SourcePath` | Host, profile का स्वामी (path या shell folders से), सिस्टम के `ProfileList` से SID, और hive। |
| `List` | `RunMRU`, `TypedPaths`, `WordWheelQuery`, `TypedURLs`, `Paint`, `Wordpad` या `ACMru`। |
| `Subkey` | `ACMru` की subkey: खोज का प्रकार (`5603`, `5604`, `5647`, `5001`)। |
| `MRU-Position`, `IsMostRecent` | सूची में स्थान (0 सबसे हाल का है); जब सूची value का नाम नहीं देती तो ख़ाली। |
| `ValueName` | `a`, `0`, `url1`, `File1`, `000`, … |
| `Value` | Command (उसके `\1` के बिना), path, खोज शब्द, पता या फ़ाइल। |
| `TypedTimeUTC` | पता कब टाइप किया गया, `TypedURLsTime` से। |
| `LastUsedUTC` | Key का last-write time, केवल सबसे हाल की entry के लिए। |
| `KeyPath`, `KeyLastWrittenUTC` | Key और उसका last write: उसकी सबसे हाल की entry का समय, हर row का नहीं। |
| `DataRaw` | जब value के bytes decode नहीं होते तो वे bytes (hex)। |
| `TransactionLogStatus`, `LogsApplied` | Registry recovery context। |
| `Problems` | वे values जिनका नाम सूची देती है पर key में नहीं हैं, वे values जिनका नाम सूची नहीं देती, ग़लत नाम वाली values, ग़लत types, अधूरे कटे शब्द, और ऐसे समय जो FILETIME नहीं हैं। |

## जाँच की रणनीति { #investigation-strategy }

1. उपयोगकर्ता द्वारा टाइप किए गए commands, scripts, network shares और administration tools के लिए RunMRU पढ़ें।
2. उपयोगकर्ता ने जिन फ़ोल्डरों और फ़ाइलों को खोजा, उनके लिए TypedPaths और WordWheelQuery पढ़ें:
   credentials, exports, archives, removable और network स्थान।
3. `TypedTimeUTC` के साथ टाइप किए गए पतों को timeline पर रखें।
4. Prefetch, BAM, UserAssist, ShellBags, LNK फ़ाइलों, jump lists और browser history से correlate करें।

## अधिक जानकारी { #further-details }

- [libyal winreg-kb: हाल में उपयोग की गई (MRU) keys](https://github.com/libyal/winreg-kb/blob/main/docs/sources/explorer-keys/Most-recently-used.md)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
