# Windows Search इंडेक्स { #windows-search-index }

## इंडेक्स में क्या दर्ज होता है { #what-the-index-records }

Windows Search (`WSearch` service) user profiles, Start menu, Outlook के mailboxes, Internet Explorer के
history, offline files और अपने scope में आने वाली हर अन्य चीज़ को crawl करता है, और जो उसने पढ़ा उसे
`Windows.edb` नामक एक ESE database में रखता है। जाँच के लिए इसके दो हिस्से महत्वपूर्ण हैं:

- **property store** (`SystemIndex_PropertyStore`; Windows 7 पर `SystemIndex_0A`), जिसमें हर indexed
  item के लिए एक row होती है: उसका path, नाम, type और आकार, उसका owner, वे created, modified और accessed
  समय जो indexer ने फ़ाइल से copy किए, indexer ने उसे कब पढ़ा, किसी document का title और author, उसके
  text की शुरुआत, computer का नाम, और सैकड़ों अन्य properties;
- **gatherer** tables (`SystemIndex_Gthr`, `SystemIndex_GthrPth`), जिनमें हर item का crawler का अपना
  record होता है: उसका URL path, queue में डाले जाते समय उसका modification time, crawl number और flags,
  और एक deletion counter।

फ़ाइल, message या page के हट जाने के बाद भी item index में तब तक रहता है जब तक crawler को deletion का पता
नहीं चलता। इसलिए index उन फ़ाइलों के नाम बताता है जो अब मौजूद नहीं हैं, उनके metadata और उनके text के एक
extract के साथ, जैसे वे index किए जाते समय थीं।

<span class="evidence-strength">सर्वोत्तम निष्कर्ष:</span> indexer ने दर्ज path पर, दर्ज metadata के साथ,
`GatherTime` पर, नामित computer पर item को पढ़ा। Index यह दिखाता है कि फ़ाइल मौजूद थी और उसमें क्या था। यह
नहीं दिखाता कि किसी user ने उसे खोला।

## स्थान और संग्रह { #location-and-collection }

```text
C:\ProgramData\Microsoft\Search\Data\Applications\Windows\Windows.edb
```

Windows 7 से Windows 11 21H2 तक, और उन्हीं वर्षों के Windows Server releases, index को `Windows.edb` में
रखते हैं। Windows 11 22H2 और बाद के संस्करण इसे उसी folder में SQLite databases (`Windows.db`,
`Windows-gather.db`, `Windows-usn.db`) में रखते हैं; Juntei उन्हें ढूँढ़कर उनके नाम बताता है, और उन्हें
unsupported नए format के रूप में report करता है।

पूरा `Applications\Windows` folder किसी disk image, volume shadow copy से, या `WSearch` service को रोकने
के बाद एकत्र करें: service के चलते रहते database खुला रहता है, और उसके सबसे नए बदलाव केवल `MSS*.log`
transaction logs में होते हैं, जिन्हें Juntei replay नहीं करता। चल रहे system से copy किया गया index dirty
रह जाता है; output यह बताता है, और जो disk पर है उसे पढ़ता है। Directory input और `auto-parse` किसी भी
गहराई पर मौजूद हर `Windows.edb` को पढ़ते हैं; एक फ़ाइल के लिए `-f` का उपयोग करें।

## विश्लेषक के लिए सावधानियाँ { #analyst-cautions }

- `GatherTime` वह समय है जब indexer ने item को पढ़ा, न कि जब किसी user ने उसे बनाया, बदला या खोला।
  Indexing background में चलती है, अक्सर किसी बदलाव के मिनटों से घंटों बाद, और computer के व्यस्त रहने या
  battery पर चलने के दौरान रुक जाती है।
- `DateCreated`, `DateModified` और `DateAccessed` वे copies हैं जो indexer ने फ़ाइल से लीं। Item के दोबारा
  index होने पर ये refresh हो जाती हैं, इसलिए ये फ़ाइल को `GatherTime` के समय की स्थिति में दर्शाती हैं।
- `ItemPath` display path है। `ItemURL` और `GatherPath` crawler के URLs हैं: फ़ाइलों के लिए `file:`
  (Windows 8 और बाद में volume `[GUID]` के रूप में), Internet Explorer के history के लिए
  `iehistory://{SID}/`, Outlook items के लिए `mapi://`, offline files के लिए `csc://{SID}/` और Windows
  apps की सामग्री के लिए `winrt://{SID}/`।
- `User` item के path में नामित profile है (`C:\Users\<name>\…`)। यह बताता है कि item कहाँ था, यह नहीं कि
  उसे किसने बनाया; `FileOwner` फ़ाइल का owner है जैसा indexer ने देखा।
- `AutoSummary` item के text की शुरुआत है। यह ऐसे document को उद्धृत कर सकता है जो बाद में delete या
  encrypt कर दिया गया हो, और यह एक truncated extract है, document नहीं।
- Dirty database में ऐसे pages हो सकते हैं जिन तक उसके branch pages अभी नहीं पहुँचते। Juntei leaf pages
  की अपनी chain का अनुसरण करता है और सारांश में बताता है कि उसने इस तरह कितने पढ़े। उसके सबसे नए बदलाव
  फिर भी transaction logs में ही रहते हैं।
- Gatherer tables में मौजूद ऐसा item जिसकी property-store row नहीं है (`InTables` `Gthr` है), queue में
  डाला गया या crawl किया गया था, और उसकी properties अभी लिखी नहीं गईं या हटा दी गई हैं। उसका path, नाम
  और gatherer के समय फिर भी वहाँ होते हैं।
- Windows 7 strings को अपनी खुद की compressed, obfuscated encoding में संग्रहीत करता है। Juntei उसे decode
  करता है; जो value decode नहीं होती वह hexadecimal ही रहती है, `Problems` में एक note के साथ।

## Juntei से parse करें { #parse-with-juntei }

```text
Usage: juntei parse-ese windows-search <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Directory to search recursively for Windows Search indexes
                         (Windows.edb; Windows.db is reported as the unsupported SQLite index)
  -f, --file <FILE>      File path to one Windows.edb

Output:
  -o, --output <FILE>       Save every indexed item to this file
  -t, --output-type <TYPE>  csv (default), json, or jsonl
  -C, --clobber             Overwrite the output file when saving
```

```bash
juntei parse-ese windows-search -f Windows.edb -o windows-search.csv
juntei parse-ese windows-search -d evidence/ -t jsonl \
  | jq 'select(.Store == "file") | {ItemPath, DateModified, GatherTime, FileOwner, AutoSummary}'
```

CSV डिफ़ॉल्ट फ़ाइल format है। `-t csv`, `-t json` या `-t jsonl` के साथ `-o` न देने पर records
stdout पर लिखे जाते हैं। दोनों में से कोई विकल्प न देने पर केवल सारांश दिखता है। मौजूदा output को overwrite करने के
लिए `-C` जोड़ें। Output कभी भी किसी input फ़ाइल के ऊपर नहीं लिखा जाता, `-C` के साथ भी नहीं।

Juntei हर indexed item के लिए एक record लिखता है, जिसमें property store और gatherer tables को item के
identifier पर join किया जाता है। सारांश में databases schema के अनुसार, dirty databases, दोनों tables में,
केवल gatherer में और केवल property store में मौजूद items, folders, deleted चिह्नित items, प्रति store items,
बिना संग्रहीत समय वाले items, और problems गिने जाते हैं।

क्षति run को नहीं रोकती। बीच में कटा database अपने बचे हुए pages में मौजूद items रखता है, और सारांश बताता है
कि क्या पढ़ा नहीं जा सका। `Windows.edb` नाम की ऐसी फ़ाइल जो Windows Search index नहीं है, report करके छोड़
दी जाती है। बाकी databases फिर भी पढ़े जाते हैं।

## आउटपुट fields { #output-fields }

| फ़ील्ड | व्याख्या |
|---|---|
| `DocumentID` | Index में item का identifier: property store का `WorkID` (Windows 7 पर `DocID`) और gatherer row का `DocumentID`। |
| `InTables` | `Gthr+PropertyStore`, `Gthr` (queue में डाला गया या crawl किया गया item जिसकी properties नहीं हैं) या `PropertyStore`। |
| `ItemPath` | `System_ItemPathDisplay`: path जैसा Windows उसे दिखाता है। Property store में row न होने पर खाली। |
| `ItemURL` | `System_ItemUrl`: item के लिए crawler का URL (`file:`, `iehistory://`, `mapi://`, `csc://`, `winrt://`)। |
| `GatherPath` | Gatherer का path, scope tree और row के फ़ाइल नाम को जोड़कर बनाया गया। |
| `Name` | `System_ItemName`, या gatherer row का फ़ाइल नाम। |
| `ItemType`, `ItemTypeText` | `System_ItemType` (extension, या `Directory`) और उसका display name। |
| `Kind` | `System_Kind`: `document`, `folder`, `link`, `program`, `email`, `picture`, …; कई होने पर `\|` से जोड़े जाते हैं। |
| `MimeType` | `System_MIMEType`। |
| `Store` | `System_Search_Store`: `file`, `mapi` (Outlook), `iehistory`, `csc` (offline files) या `winrt` (Windows apps)। |
| `IsFolder`, `IsDeleted` | `System_IsFolder` और `System_IsDeleted`, जैसे संग्रहीत हैं। |
| `DeletedCount` | Gatherer row का deletion counter। |
| `Size` | `System_Size` bytes में। Folders के लिए खाली, जिनमें एक placeholder होता है। |
| `FileAttributes` | `System_FileAttributes` एक संख्या के रूप में (16 directory, 32 archive, 2 hidden, …)। |
| `FileOwner` | `System_FileOwner` (`DOMAIN\user`) जैसा indexer ने पढ़ा। |
| `DateCreated`, `DateModified`, `DateAccessed` | फ़ाइल के समय जैसे indexer ने उन्हें copy किया (UTC)। |
| `ItemDate` | `System_ItemDate`: item की अपनी date, जैसे किसी message का sent time या किसी photo की date taken; फ़ाइल के लिए आमतौर पर second तक उसका modification time। |
| `GatherTime` | `System_Search_GatherTime`: जब indexer ने item को पढ़ा (UTC)। |
| `GatherLastModified` | Gatherer row का `LastModified`: item का modification time जब crawler ने उसे queue में डाला (UTC)। |
| `Title`, `Author` | `System_Title` और `System_Author`। |
| `AutoSummary` | `System_Search_AutoSummary`: item के text की शुरुआत। |
| `CompName` | `System_ComputerName`: वह computer जिसका यह index है। |
| `TargetURL`, `DateVisited` | Internet Explorer history item का URL और visit time (`System_Link_TargetUrl`, `System_Link_DateVisited`)। |
| `ActivityApp`, `ActivityText`, `ActivityContentURI`, `ActivityStartTime`, `ActivityEndTime` | Windows 10 activity history item (`System_Activity_*`, `System_ActivityHistory_*`)। |
| `User`, `UserSource` | Item के path में नामित profile, और वह किस path से पढ़ा गया (`ItemPath`, `ItemURL` या `GatherPath`)। Profile के बाहर खाली। |
| `ScopeID`, `SDID` | Gatherer row के scope और security descriptor identifiers। |
| `CrawlNumber` | Gatherer row का `CrawlNumberCrawled`। |
| `GatherFlags` | Gatherer row का `TransactionFlags`। |
| `Priority` | अभी भी queue में मौजूद item की crawl priority। Item के crawl हो जाने पर ESE इसे NULL चिह्नित कर देता है, इसलिए अधिकांश items के लिए यह खाली होता है। |
| `PropertyCount` | Property store में item के लिए कितनी properties हैं। |
| `OtherProperties` | बाकी हर property, `; ` से अलग की गई `Name=Value` के रूप में। |
| `SourceFile` | पढ़ा गया database, input directory के सापेक्ष, या `-f` के साथ उसका नाम। |
| `Problems` | जो values decode नहीं हुईं, जो समय समय नहीं हैं, duplicate rows, और क्षति। इसे दबाएँ नहीं। |

JSON और JSONL यही fields typed values के साथ रखते हैं: संख्याएँ संख्या ही रहती हैं और लापता values
`null` होती हैं। `OtherProperties` के बजाय उनमें `Properties` होता है: item की हर property, उसके नाम,
numeric identifier, column type और value के साथ, और किसी समय के लिए उसका raw FILETIME tick count उस समय
के बगल में जो उससे पढ़ा जाता है। `Gather` में gatherer row के columns होते हैं।

## Timeline व्यवहार { #timeline-behavior }

`auto-parse` records को `FileExistence/WindowsSearch.csv` (या `.json`/`.jsonl`) के रूप में सहेजता है और
उन्हें संयुक्त `-j` timeline में जोड़ता है। `super-timeline` `DateCreated`, `DateModified` और
`DateAccessed` (फ़ाइल के समय जैसे Windows Search ने दर्ज किए), `GatherTime` (जब item index किया गया),
Internet Explorer history item के `DateVisited` और किसी activity के `ActivityStartTime` और
`ActivityEndTime` से events बनाता है। `GatherLastModified` केवल उस item के लिए event बनता है जिसके लिए
property store में कोई modification time नहीं है। इनमें से कोई भी न रखने वाला item parse किए गए output में
रहता है और बिना समय वाले record के रूप में गिना जाता है।

## जाँच की रणनीति { #investigation-strategy }

1. `Store` को `file` पर filter करें और case की फ़ाइलों, नामों और शब्दों के लिए `ItemPath`, `Name` और
   `AutoSummary` में खोजें। Index उन फ़ाइलों के नाम बताता है जो बाद में delete कर दी गईं।
2. `GatherTime` को उस आखिरी समय के रूप में पढ़ें जब फ़ाइल दर्ज metadata के साथ मौजूद थी, और
   `DateModified` को उस क्षण उसके modification time के रूप में।
3. किसी फ़ाइल को किसी account से जोड़ने के लिए `FileOwner` और `User` का उपयोग करें, फिर `$MFT`, USN
   Journal और `$Secure:$SDS` से पुष्टि करें।
4. E-mail (`Store` `mapi`) और Internet Explorer history (`iehistory`) के लिए पूरे records हेतु Outlook की
   फ़ाइलों और WebCache में pivot करें।
5. Index की तुलना hosts के बीच करें: `CompName` उस computer का नाम बताता है जिसका यह index है, और किसी
   image से एकत्र किया गया index उस image की फ़ाइलों के नाम बताता है।

## परीक्षित संस्करण { #tested-versions }

परीक्षण ऐसे सार्वजनिक indexes का उपयोग करते हैं जिनके परिणाम अन्य tools स्वतंत्र रूप से पढ़ते हैं:

- plaso का Windows 7 index (`SystemIndex_0A`), जिसे libyal के `esedbexport` से जाँचा गया, Windows 7 की
  string encodings सहित।
- DFIRArtifactMuseum का Windows 10 index, dirty छोड़ा हुआ, जिसे Stroz Friedberg के SIDR से जाँचा गया, और
  उसका Windows 11 index (ESE format revision 200), जिसे `esedbexport` से जाँचा गया।
- DFIRArtifactMuseum का Windows 11 22H2 SQLite index, जो unsupported नए format के रूप में report होता है।
- बीच में कटे, खाली, या किसी अन्य प्रकार के databases।

## अधिक जानकारी { #further-details }

- [Microsoft — Windows Search overview](https://learn.microsoft.com/en-us/windows/win32/search/-search-3x-wds-overview)
- [Microsoft — Extensible Storage Engine](https://learn.microsoft.com/en-us/windows/win32/extensible-storage-engine/extensible-storage-engine)
- [libesedb — ESE database file format](https://github.com/libyal/libesedb/blob/main/documentation/Extensible%20Storage%20Engine%20%28ESE%29%20Database%20File%20%28EDB%29%20format.asciidoc)
- [esedb-kb — Windows Search](https://github.com/libyal/esedb-kb/blob/main/documentation/Windows%20Search.asciidoc)
- [Stroz Friedberg — SIDR](https://github.com/strozfriedberg/sidr)
- [Microsoft — Extensible Storage Engine source](https://github.com/microsoft/Extensible-Storage-Engine)
