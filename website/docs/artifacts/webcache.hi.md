# Internet Explorer और लेगेसी Edge { #internet-explorer-and-legacy-edge }

## WebCache database क्या है { #what-the-webcache-database-is }

`WebCacheV01.dat` हर उपयोगकर्ता की profile में Windows के Internet functions (WinINet) का database है, जो एक
ESE database है। Internet Explorer 10 और 11 अपना history, downloads, cache और cookies इसी में रखते हैं, और
legacy Edge भी, यानी Windows 10 का वह Edge जो Edge के Chromium-based होने से पहले था। हर प्रकार का record एक
container है, जो अपनी अलग table है:

- **History** (`History`, और किसी एक दिन या एक सप्ताह के लिए `MSHist01…`): देखे गए URLs, हर एक उस account
  के साथ जिसके लिए वह दर्ज हुआ, और यह कि उसका आखिरी visit कब हुआ और कितनी बार हुआ। फ़ाइल एक्सप्लोरर जिन
  local files को खोलता है, उन्हें भी यहीं `file:///` URLs के रूप में दर्ज करता है।
- **Downloads** (`iedownload`): वह URL जहाँ से file आई, उसे कहाँ save किया गया और download के दौरान कहाँ लिखा
  गया, उसका size, download कब शुरू हुआ, referring page और MIME type।
- **Cookies**: Windows 10 और बाद के versions में स्वयं cookies (`CookieEntryEx` tables), उनके host, name,
  value और समयों के साथ; उससे पहले, वे cookie files जिनमें cookies रखी जाती थीं।
- **Cache** (`Content`): pages दिखाने के लिए download की गई files, हर एक अपने URL के साथ, disk पर उसकी copy कहाँ
  है, और HTTP response headers। WinINet का उपयोग करने वाले अन्य programs भी अपने downloads यहीं cache करते हैं।
- Web storage (`DOMStore`), compatibility lists (`iecompat`) और अन्य।

यह database browsers से अधिक समय तक बना रहता है: Internet Explorer retire हो चुका है और Chromium-based Edge
अपने अलग SQLite databases रखता है ([वेब ब्राउज़र](browser.md) देखें), फिर भी यह database हर profile में
अब भी मौजूद है, उसमें वह सब है जो पहले दर्ज हुआ था, और वह भी जो फ़ाइल एक्सप्लोरर और अन्य programs अब भी दर्ज
करते हैं।

<span class="evidence-strength">सर्वोत्तम निष्कर्ष:</span> उपयोगकर्ता की profile ने दर्ज समय पर URL, download
या cache की गई file को दर्ज किया; database यह नहीं बताता कि keyboard पर कौन बैठा था, या यह कि cache की गई
file खोली गई थी।

## स्थान और संग्रह { #location-and-collection }

```text
%LOCALAPPDATA%\Microsoft\Windows\WebCache\WebCacheV01.dat
%LOCALAPPDATA%\Microsoft\Windows\WebCache\V01.log
%LOCALAPPDATA%\Microsoft\Windows\WebCache\V01*.log
%LOCALAPPDATA%\Microsoft\Windows\WebCache\V01.chk
```

Containers जिन files के नाम देते हैं (cache की गई files, cookie files), वे अपने अलग folders में होती हैं:
`%LOCALAPPDATA%\Microsoft\Windows\INetCache\`, `INetCookies\`, `History\`, और apps के लिए
`%LOCALAPPDATA%\Packages\` के अंतर्गत।

पूरा `WebCache` folder संग्रहित करें। उपयोगकर्ता के logged on रहते Windows database को खुला रखता है, इसलिए
चल रहे system से ली गई copy आमतौर पर dirty होती है: उसके सबसे नए बदलाव केवल `V01*.log` transaction logs में
होते हैं।

!!! warning "Transaction logs"
    Juntei ESE transaction logs को replay नहीं करता। जो database ठीक से बंद नहीं हुआ, उसे summary में dirty
    छोड़े गए database के रूप में गिना जाता है और problem के रूप में रिपोर्ट किया जाता है: जो बदलाव अब भी केवल
    उसके logs में हैं, वे अनुपस्थित रहते हैं। उन्हें शामिल करने के लिए पहले folder की एक copy में logs को replay
    करें: उस copy में Windows का `esentutl /r V01 /d` चलाएँ।

## विश्लेषक के लिए सावधानियाँ { #analyst-cautions }

- समय UTC में हैं, 100 nanoseconds तक, सिवाय किसी दिन या सप्ताह के history container के `ModifiedTime` के: वह
  system का local time है, बिना offset के लिखा हुआ, और `-Z` उसे जैसा है वैसा ही छोड़ देता है। उसे source system के उस समय के
  time zone से convert करें। `AccessedTime` से उसका अंतर अक्सर उसी time zone का offset होता है, लेकिन हमेशा नहीं:
  दोनों अलग-अलग क्षण हो सकते हैं।
- History entry एक URL है, visit नहीं: `AccessedTime` उसका आखिरी visit है और `AccessCount` बताता है कि कितने
  visits हुए। `History` container हर URL के लिए एक entry रखता है; किसी दिन या सप्ताह का container उस अवधि की
  entries रखता है, और उसका नाम अवधि का पहला दिन और आखिरी दिन के बाद वाला दिन बताता है
  (`MSHist012026110120261102`: 1 नवंबर 2026)।
- History में `file:///` URLs फ़ाइल एक्सप्लोरर और अन्य programs से खोली गई local files हैं, web pages नहीं।
- `Account` वह account है जिसके लिए entry दर्ज की गई (`Visited: alice@…`); `User` वह profile folder है जिसमें
  database का path है। दोनों अलग हो सकते हैं: profile folder का नाम हमेशा उसके account के नाम पर नहीं होता,
  और कहीं और copy किए गए database के path में कोई profile folder नहीं होता।
- `Package` बताता है कि entry किसकी है: legacy Edge की (`microsoft.microsoftedge_8wekyb3d8bbwe`), Enhanced
  Protected Mode में Internet Explorer की (`windows_ie_ac_001`), या किसी अन्य app की। खाली होने पर entry
  उपयोगकर्ता की अपनी है: Internet Explorer, फ़ाइल एक्सप्लोरर और अन्य programs।
- Microsoft download record का दस्तावेज़ीकरण नहीं करता। Juntei उसका start time और size केवल तभी पढ़ता है जब
  उसका header record के भीतर समा जाता हो, और उसके paths तथा अन्य strings केवल तभी जब आखिरी दो strings एक URL और एक
  Windows path हों। अन्यथा row में entry का अपना URL होता है, और start न पढ़ा जा सके तो उसका access time,
  साथ में एक problem। `Publisher` वह नाम है जो browser ने दर्ज किया, file के signature की जाँच नहीं।
- Cache entry बताती है कि file download और cache की गई थी। Disk पर उसकी copy हट चुकी हो सकती है, बदली जा चुकी
  हो सकती है, या कभी खोली ही न गई हो।
- Deleted entries अभी recover नहीं की जातीं: ESE किसी deleted record को अपने pages में तब तक रख सकता है जब तक
  वह उन्हें साफ़ नहीं कर देता, लेकिन Juntei केवल live records पढ़ता है।

## Juntei से parse करें { #parse-with-juntei }

```text
Usage: juntei parse-ese webcache <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find WebCacheV01.dat
  -f, --file <FILE>      Parse one WebCache database

Output:
  -o, --output <PATH>       Directory of CSV tables, or JSON/JSONL file
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-ese webcache -d Users -o webcache-csv
juntei parse-ese webcache -d Users -t jsonl \
  | jq 'select(.Table == "Downloads") | {StartTime, URL, TargetPath, User}'
```

`-d` `WebCacheV01.dat` को नाम से खोजता है; `-f` किसी database को पढ़ता है, उसका नाम चाहे जो हो।

## आउटपुट tables { #output-tables }

CSV पाँच files बनाता है; rows न होने पर हर file में केवल उसका header होता है। JSON और JSONL rows में
`SourceFile` के बाद `Table` (`History`, `Downloads`, `Cookies`, `Entries` या `Containers`) होता है।

| CSV file | सामग्री |
|---|---|
| `WebCache-History.csv` | History containers की entries। |
| `WebCache-Downloads.csv` | Downloads। |
| `WebCache-Cookies.csv` | Cookies (Windows 10 और बाद के versions)। |
| `WebCache-Entries.csv` | हर अन्य container की entries: cache की गई files, cookie files, web storage, compatibility lists और बाकी सब। |
| `WebCache-Containers.csv` | Containers। |

## Fields { #fields }

हर table इन fields से शुरू होती है:

| Field | व्याख्या |
|---|---|
| `SourceFile` | वह database जिससे row पढ़ी गई। |
| `User` | वह Windows उपयोगकर्ता जिसकी profile में path है (`Users` के बाद वाला folder)। |
| `Package` | वह app जिसका data यह row है: row के container की directory (या उसकी cookie table की directory) में `Packages` के अंतर्गत वाला folder। उपयोगकर्ता के अपने data के लिए खाली। |

### History { #history }

| Field | व्याख्या |
|---|---|
| `Container` | Container: `History`, या `MSHist01` और उसके बाद उसकी अवधि का पहला दिन और आखिरी दिन के बाद वाला दिन। |
| `AccessedTime` | URL का आखिरी visit (UTC); किसी दिन या सप्ताह के container में, उस अवधि का आखिरी visit। |
| `ModifiedTime` | Entry में आखिरी बदलाव कब हुआ (UTC); किसी दिन या सप्ताह के container में, system का local time, बिना offset के। |
| `CreationTime` | Entry कब बनी (UTC)। |
| `ExpiryTime`, `SyncTime` | Entry का history से हटना कब तय है, और आखिरी बार कब synchronize हुई (UTC)। |
| `Account` | वह account जिसके लिए entry दर्ज की गई। |
| `URL` | Page या file, `Visited: account@` prefix के बिना। |
| `AccessCount`, `SyncCount` | URL कितनी बार visit किया गया, और entry कितनी बार synchronize हुई। |
| `EntryType`, `Flags` | Entry का type और flags, जैसे stored हैं। |
| `EntryID`, `ContainerID` | उसके container में entry का id, और container का id। |

### Downloads { #downloads }

| Field | व्याख्या |
|---|---|
| `StartTime` | Download कब शुरू हुआ (UTC)। |
| `URL` | File कहाँ से आई। |
| `TargetPath`, `CurrentPath` | File कहाँ save की गई, और download के दौरान वह कहाँ लिखी जा रही थी: legacy Edge में एक `.partial` file, Internet Explorer में cache की copy। |
| `TotalBytes` | File का size। |
| `Referrer` | Referring page। |
| `MimeType` | File का type जैसा server ने बताया (Internet Explorer)। |
| `Publisher` | File के publisher का वह नाम जो legacy Edge ने दर्ज किया। |
| `AccessedTime` | Entry का अपना access time (UTC), अधिकतर download की शुरुआत। |
| `DownloadID`, `ContainerID` | उसके container में entry का id, और container का id। |

### Cookies { #cookies }

| Field | व्याख्या |
|---|---|
| `LastModified`, `Expires` | Cookie कब set हुई, और कब expire होगी (UTC)। |
| `Host`, `Name`, `Path` | वह site जिसके लिए cookie है, उसका नाम और path। |
| `Value` | Value, जैसी stored है। |
| `Flags` | Cookie के flags, जैसे stored हैं। |
| `Directory` | वह directory जो database cookie table को देता है: यह बताती है कि cookies किसकी हैं (`…\INetCookies\ESE\`, legacy Edge की `…\MicrosoftEdge\Cookies\ESE\`, किसी app की)। |
| `CookieTable`, `EntryID` | Table (`CookieEntryEx_<n>`) और उसमें cookie का id। |

### Entries { #entries }

| Field | व्याख्या |
|---|---|
| `Container` | Container: `Content` (cache की गई files), `Cookies` (cookie files), `DOMStore` (web storage), `iecompat`, …। |
| `AccessedTime` | Entry का आखिरी बार उपयोग कब हुआ (UTC)। |
| `ModifiedTime` | Resource का आखिरी modification, जैसा server ने बताया (UTC)। |
| `CreationTime` | Entry कब बनी (UTC)। |
| `ExpiryTime`, `SyncTime` | Entry कब expire होगी, और server से आखिरी बार कब जाँची गई (UTC)। |
| `URL` | URL जैसा stored है: cache की गई file के लिए एक URL, cookie file के लिए `Cookie:alice@example.com/`, `DOMStore:…`, `iecompat:…`। |
| `Account` | किसी cookie file का account। |
| `Filename`, `FilePath` | Entry की file, और वह कहाँ है: container की directory और वह subfolder जिसका नाम entry देती है। |
| `FileSize` | File का size। |
| `AccessCount`, `SyncCount` | Entry कितनी बार उपयोग हुई, और कितनी बार server से जाँची गई। |
| `EntryType`, `Flags` | Entry का type और flags, जैसे stored हैं। |
| `RedirectURL` | URL कहाँ redirect हुआ। |
| `ResponseHeaders`, `RequestHeaders` | HTTP response headers (`HTTP/1.1 200 OK`, `Content-Type`, `Server`, …) और request headers, जैसे stored हैं, उनकी सभी lines सहित। |
| `EntryID`, `ContainerID` | उसके container में entry का id, और container का id। |

### Containers { #containers }

| Field | व्याख्या |
|---|---|
| `ContainerID`, `Name` | Container का id और नाम। |
| `Directory` | Disk पर उसकी files कहाँ हैं। |
| `SecureDirectories` | `Directory` के वे subfolders जिनमें उसकी files हैं (CSV में उनके बीच comma)। |
| `PartitionID` | उपयोगकर्ता के अपने containers के लिए `M`, Internet Explorer के Protected Mode के containers (`Low` folders) के लिए `L`, और किसी app के container के लिए उस app container का SID। |
| `LastAccessTime`, `LastScavengeTime` | Container का आखिरी बार उपयोग कब हुआ, और आखिरी बार उसकी सफ़ाई (cleanup) कब हुई (UTC)। |

## स्वचालित विश्लेषण { #automatic-parsing }

`auto-parse` tables को `BrowserActivity` में `WebCache-History.csv`, `WebCache-Downloads.csv`,
`WebCache-Cookies.csv`, `WebCache-Entries.csv` और `WebCache-Containers.csv` के रूप में लिखता है (`-t json` या
`-t jsonl` के साथ एक ही `WebCache.json` या `WebCache.jsonl` file), साथ में `BrowserActivity/WebCache-logs.txt`
भी। संयुक्त `-j` timeline में हर row शामिल होती है, `Comp` और `Artifact` identifier `webcache` के साथ। `-Z`
हर समय को convert करता है, यानी `Time` पर समाप्त होने वाले fields, `LastModified` और `Expires`, सिवाय किसी दिन
या सप्ताह के history container के local `ModifiedTime` के।

## अधिक जानकारी { #further-details }

- [Microsoft — About WinINet](https://learn.microsoft.com/en-us/windows/win32/wininet/about-wininet)
- [Microsoft — Extensible Storage Engine](https://learn.microsoft.com/en-us/windows/win32/extensible-storage-engine/extensible-storage-engine)
- [libesedb — ESE database file format](https://github.com/libyal/libesedb/blob/main/documentation/Extensible%20Storage%20Engine%20%28ESE%29%20Database%20File%20%28EDB%29%20format.asciidoc)
- [Plaso — MSIE WebCache parser](https://github.com/log2timeline/plaso/blob/main/plaso/parsers/esedb_plugins/msie_webcache.py)
- [dissect.database](https://github.com/fox-it/dissect.database)
