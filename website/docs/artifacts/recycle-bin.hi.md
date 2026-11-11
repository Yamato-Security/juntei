# रीसायकल बिन { #recycle-bin }

## रीसायकल बिन क्या दर्ज करता है { #what-the-recycle-bin-records }

जब कोई उपयोगकर्ता फ़ाइल एक्सप्लोरर में किसी फ़ाइल या फ़ोल्डर को delete करता है, तो Windows उसे उसी volume
के रीसायकल बिन में ले जाता है और उसके पास एक छोटा index record लिखता है: item का पहले वाला path, उसका आकार,
और वह कब delete किया गया। हर उपयोगकर्ता का हर volume पर एक bin फ़ोल्डर होता है, जिसका नाम उपयोगकर्ता के SID
पर रखा जाता है, इसलिए फ़ोल्डर यह भी बताता है कि item किस account ने delete किया।

| Windows संस्करण | Index | हटाई गई सामग्री |
|---|---|---|
| Vista और बाद के संस्करण | हर item के लिए एक `$I…` फ़ाइल (Vista से 8.1 तक format version 1, Windows 10 और बाद के संस्करणों में version 2) | `$R…`, `$R` के बाद वही नाम: फ़ाइल, या अपनी सामग्री सहित फ़ोल्डर |
| NT 4.0, 2000, XP, Server 2003 | हर bin के लिए एक `INFO2` फ़ाइल: 800-byte records, जिनमें एक ANSI और एक Unicode path होता है | `D<drive letter><index><extension>`, जैसे `Dc1.txt` |
| 95, 98, ME | एक `INFO2` फ़ाइल (Windows 95 पर `INFO`): 280-byte records, जिनमें केवल ANSI path होता है | वही `D…` नाम |

<span class="evidence-strength">सर्वोत्तम निष्कर्ष:</span> जिस account के bin में record है, उसने मूल path
वाले item को दर्ज समय पर रीसायकल बिन में भेजा। Shift+Delete से, command line से या अधिकांश programs से delete
करने पर item bin में नहीं जाता और कोई record नहीं बचता।

## स्थान और संग्रह { #locations-and-collection }

```text
C:\$Recycle.Bin\<SID>\$I…        Windows Vista and later (each volume has its own bin)
C:\$Recycle.Bin\<SID>\$R…        the deleted file or folder
C:\RECYCLER\<SID>\INFO2          Windows NT 4.0, 2000, XP and Server 2003 (NTFS)
C:\RECYCLED\INFO2                Windows 95, 98 and ME, and FAT volumes (INFO on Windows 95)
```

हर volume के bin की `$I` फ़ाइलें या INFO2 एकत्र करें। `$R` और `D…` फ़ाइलें भी एकत्र करने से पता चलता है कि
कौन-से deleted items अभी भी वहाँ हैं; उनके बिना हर item के लिए `RecycledFileFound` `false` होता है। हटाई
गई सामग्री अपने आप में साक्ष्य है और malware भी हो सकती है: उसे उसी के अनुसार संभालें।

## विश्लेषक के लिए सावधानियाँ { #analyst-cautions }

- `DeletedTime` वह समय है जब item bin में ले जाया गया, system clock के अनुसार, UTC FILETIME के रूप में।
  यह bin खाली किए जाने का समय नहीं है। Item के अपने creation और modification times `$MFT` और `$R` फ़ाइल
  में होते हैं।
- SID उस account का है जिसने item delete किया; ज़रूरी नहीं कि वह फ़ाइल का owner हो। जिस SID की SOFTWARE
  hive के `ProfileList` में कोई profile नहीं है (हटाया गया account, या ऐसा domain account जिसने कभी logon
  नहीं किया), उसका कोई `User` नहीं होता।
- जब bin खाली किया जाता है, या कोई item restore किया जाता है या bin से delete किया जाता है, तो Windows Vista
  और बाद के संस्करण उसकी `$I` और `$R` फ़ाइलें delete कर देते हैं। उनके नाम USN Journal, `$MFT` और `$I30`
  slack में बचे रह सकते हैं; Juntei उनसे deleted `$I` records recover नहीं करता। INFO2 ऐसे records रखे रहता
  है और उनके ANSI path का पहला byte मिटा देता है: `RemovedFromBin` `true` होता है, और Juntei वह character
  drive number से वापस भर देता है।
- `$I` फ़ाइल में `FileSize` item का आकार है; फ़ोल्डर के लिए, उसकी सामग्री का कुल आकार। INFO2 में यह वह
  जगह है जो item ने disk पर घेरी थी, पूरे clusters में (किसी छोटी फ़ाइल के लिए 4,096 bytes)।
- INFO2 का ANSI path अक्सर 8.3 रूप में होता है और उसके लिए system का code page आवश्यक है: जापानी Windows
  के लिए `--code-page 932`, चीनी के लिए 936 या 950 का उपयोग करें। Unicode path, जहाँ मौजूद हो, `OriginalPath`
  में होता है।
- कुछ Windows Vista systems ने 543-byte `$I` फ़ाइलें लिखीं, जिनका size field एक byte छोटा है। Juntei उनका
  समय और path पढ़ता है, और `FileSize` को खाली छोड़कर एक problem दर्ज करता है।
- `RecycledFileFound` संग्रह का वर्णन करता है, endpoint का नहीं: `false` का अर्थ यह हो सकता है कि संग्रह के
  दौरान `$R` फ़ाइलें नहीं ली गईं, या यह कि item अब bin में नहीं है।
- जिन volumes पर Windows installation नहीं है, उनके bins user names संग्रह के एकमात्र Windows installation
  से लेते हैं, बशर्ते ठीक एक ही installation हो।

## Juntei से parse करें { #parse-with-juntei }

```text
Usage: juntei parse-file recycle-bin <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find $I files and INFO2
  -f, --file <FILE>      Parse one $I or INFO2 file (told apart by its contents if renamed)
      --code-page <CP>   INFO2's ANSI code page (default 1252; Japanese commonly 932)

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-file recycle-bin -d evidence/ -o recycle-bin.csv
juntei parse-file recycle-bin -d xp-evidence/ --code-page 932 -o recycle-bin-ja.csv
juntei parse-file recycle-bin -f '$IABC123.docx' -t jsonl | jq .
```

`-d` के साथ, Juntei वे फ़ाइलें पढ़ता है जिनके नाम में `$I` के बाद छह अक्षर या अंक हों (किसी भी extension के
साथ), `INFO2`, और `RECYCLED` फ़ोल्डर में `INFO`। `-f` के साथ, वह फ़ाइल को पढ़ता है, उसका नाम चाहे जो हो।

## CSV आउटपुट fields { #csv-output-fields }

| Field | अर्थ |
|---|---|
| `SourceFile` | `$I` या INFO2 फ़ाइल। |
| `DeletedTime` | Item कब bin में ले जाया गया (UTC)। |
| `OriginalPath` | Item कहाँ था। INFO2 के लिए, record में Unicode path हो तो वह, अन्यथा ANSI path। |
| `FileSize` | `$I`: item का आकार bytes में (फ़ोल्डर का कुल आकार)। INFO2: disk पर उसकी जगह, पूरे clusters में। पढ़ा न जा सके तो खाली। |
| `User`, `SID` | Bin फ़ोल्डर का SID, और वह user जिसके नाम पर उसकी `ProfileList` profile रखी गई है। |
| `RecycledFile`, `RecycledFileFound` | Index के पास रखी हटाई गई सामग्री का path (`$R…`, `D<drive><index><extension>`), और क्या वह संग्रह में है। |
| `Format`, `FormatVersion` | `$I` 1 (Vista से 8.1) या 2 (Windows 10 और बाद के संस्करण); `INFO2` 0 (95), 2 (NT 4.0), 4 (98) या 5 (ME, 2000 से Server 2003)। |
| `RecordIndex`, `Drive`, `RecordOffset` | INFO2: record की संख्या (उसके `D…` file name में मौजूद संख्या), drive, और INFO2 में record कहाँ से शुरू होता है। UNC path में कोई drive नहीं होता। |
| `RemovedFromBin` | INFO2: `true`, जब item restore किया गया हो या bin से delete किया गया हो। |
| `ShortPath` | INFO2: ANSI path, जब वह `OriginalPath` से अलग हो (अक्सर 8.3 नाम)। |
| `Problems` | क्षति: truncated या अपेक्षा से लंबी फ़ाइल, अमान्य drive number, ऐसा path जो वैध UTF-16 नहीं है, ऐसा deletion time जो वैध समय नहीं है। |

JSON और JSONL में यही fields होते हैं: numbers और booleans typed होते हैं, जो values किसी record में नहीं हैं
उनके लिए `null` होता है, और `Problems` एक list होती है।

## स्वचालित विश्लेषण { #automatic-parsing }

`auto-parse` `FileExistence/RecycleBin.csv` लिखता है (`-t json` या `-t jsonl` के साथ `RecycleBin.json` या
`RecycleBin.jsonl`), साथ में `FileExistence/RecycleBin-logs.txt` भी। उसका `--code-page` INFO2 पर लागू होता
है। Disk images से `$I` फ़ाइलें और INFO2 कॉपी की जाती हैं; `$R` और `D…` फ़ाइलें खाली placeholders होती हैं,
इसलिए `RecycledFileFound` फिर भी भरा जाता है। संयुक्त `-j` timeline में हर row `Artifact` identifier
`recycle-bin` के साथ शामिल होती है, `-Z` `DeletedTime` को convert करता है, और
[`super-timeline`](../commands/super-timeline.md) हर item के लिए एक `Recycle Bin deletion (moved to the bin)`
event बनाता है।

## जाँच की रणनीति { #investigation-strategy }

1. हर user के लिए `DeletedTime` के अनुसार sort करें, और जिस गतिविधि की आप जाँच कर रहे हैं उसके समयों से तुलना करें।
2. LNK, Jump Lists, ShellBags और RecentDocs में `OriginalPath` खोजें, ताकि पता चले कि item delete होने से पहले
   उपयोग किया गया था या नहीं।
3. USN Journal में deletion लगभग `DeletedTime` पर `$Recycle.Bin` में rename के रूप में दिखता है; `$MFT` item
   के अपने times देता है।
4. जब `RecycledFileFound` `true` हो, तो `$R` या `D…` सामग्री का hash निकालें और उसकी जाँच करें।
5. यदि bin खाली कर दिया गया था, तो USN Journal और `$MFT` में `$I` और `$R` नाम खोजें।

## अधिक जानकारी { #further-details }

- [libyal dtformats — Windows Recycle.Bin file formats](https://github.com/libyal/dtformats/blob/main/documentation/Windows%20Recycle.Bin%20file%20formats.asciidoc)
- [libyal dtformats — Windows Recycler file formats](https://github.com/libyal/dtformats/blob/main/documentation/Windows%20Recycler%20file%20formats.asciidoc)
- [Plaso — Recycle Bin and INFO2 parsers](https://github.com/log2timeline/plaso/blob/main/plaso/parsers/recycler.py)
- [Velociraptor — Windows.Forensics.RecycleBin](https://github.com/Velocidex/velociraptor/blob/master/artifacts/definitions/Windows/Forensics/RecycleBin.yaml)
- [rifiuti2](https://github.com/abelcheung/rifiuti2)
