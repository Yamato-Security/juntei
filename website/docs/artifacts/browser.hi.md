# वेब ब्राउज़र { #web-browsers }

## ब्राउज़र databases क्या हैं { #what-the-browser-databases-are }

Browsers अपना history, downloads, cookies और form entries हर उपयोगकर्ता की profile में SQLite databases में
रखते हैं। Chromium-based browsers (Chrome, Edge, Brave, Opera, Vivaldi) और Firefox-based browsers अपनी-अपनी
files और tables का उपयोग करते हैं, लेकिन एक ही प्रकार की गतिविधि दर्ज करते हैं:

- **History**: देखे गए pages; हर visit उसके समय के साथ, browser उस page पर कैसे पहुँचा (टाइप किया गया
  पता, कोई link, कोई redirect, कोई bookmark) और, Chromium में, page कितनी देर सामने रहा।
- **Downloads**: वह URL जहाँ से file आई (और रास्ते के redirects), उसे कहाँ save किया गया, उसका size,
  download कब शुरू और कब समाप्त हुआ, और क्या वह पूरा हुआ।
- **Cookies**: उन्हें set करने वाली sites, वे कब set हुईं, आखिरी बार कब भेजी गईं और कब expire होंगी।
- **Form history**: web forms और search boxes में टाइप किए गए values, कितनी बार, पहली और आखिरी बार कब।

Browser गतिविधि अक्सर इस बात का सबसे सीधा रिकॉर्ड होती है कि उपयोगकर्ता ने क्या किया: वह phishing page
जिस पर वह गया, वह tool जो उसने download किया, वह webmail जिसका उसने उपयोग किया, वे search terms जो उसने टाइप किए।

<span class="evidence-strength">सर्वोत्तम निष्कर्ष:</span> browser profile ने दर्ज समय पर URL के visit को
दर्ज किया, या download को दर्ज path पर save किया; database यह नहीं बताता कि keyboard पर कौन बैठा था।

## स्थान और संग्रह { #location-and-collection }

```text
%LOCALAPPDATA%\Google\Chrome\User Data\<Profile>\History
%LOCALAPPDATA%\Google\Chrome\User Data\<Profile>\Network\Cookies
%LOCALAPPDATA%\Google\Chrome\User Data\<Profile>\Web Data
%LOCALAPPDATA%\Microsoft\Edge\User Data\<Profile>\...
%LOCALAPPDATA%\BraveSoftware\Brave-Browser\User Data\<Profile>\...
%APPDATA%\Opera Software\Opera Stable\...
%APPDATA%\Mozilla\Firefox\Profiles\<profile>\places.sqlite
%APPDATA%\Mozilla\Firefox\Profiles\<profile>\cookies.sqlite
%APPDATA%\Mozilla\Firefox\Profiles\<profile>\formhistory.sqlite
```

`<Profile>` का मान `Default`, `Profile 1`, `Profile 2` आदि होता है। Chromium ने version 96 में `Cookies` को
`Network` में स्थानांतरित किया; पुराने versions इसे `History` के बगल में रखते हैं। Firefox version 25 तक अपने
downloads `downloads.sqlite` में रखता था; बाद के versions उन्हें `places.sqlite` में रखते हैं।

Profile की हर file संग्रहित करें, और हर database को उसके बगल वाली उन files के साथ जिनके नाम उसी के नाम से शुरू
होते हैं: `places.sqlite-wal`, `cookies.sqlite-wal`, `formhistory.sqlite-wal`, `History-journal`।

!!! warning "Write-ahead logs"
    Firefox अपने databases को write-ahead log mode में रखता है: सबसे नए बदलाव तब तक केवल `-wal` file में
    रहते हैं जब तक browser उन्हें database में copy नहीं कर देता (checkpoint)। Juntei हर database को उसकी
    `-wal` file के साथ पढ़ता है, जैसे browser पढ़ता। इसके बिना हाल के visits, cookies और form entries
    अनुपस्थित रहते हैं, और log में delete की गई entries अब भी live दिखती हैं।

## हटाई गई entries { #deleted-entries }

जब कोई browser history delete करता है, तो rows tables से हट जाती हैं, लेकिन उनके records database के free
space में, उसकी freelist पर, या database के pages के पुराने versions में बचे रह सकते हैं: `-wal` file में, या
database file में, जहाँ किसी `-wal` ने उन pages को replace कर दिया हो। Juntei इनसे deleted visits, pages,
downloads और form entries को ऐसी rows के रूप में recover करता है जिनका `Status` `Deleted` होता है:

- `Confidence` बताता है कि record का उसकी table से मिलान कितना निश्चित है: `High` ऐसे पूरे record के लिए
  जिसकी हर value उसके column के type की है, `Medium` या `Low` क्षतिग्रस्त records के लिए या ऐसे records के लिए
  जो कई tables में fit होते हैं।
- `RecoveredFrom` बताता है कि record कहाँ था: page का क्षेत्र (एक cell, एक freeblock, unallocated space, एक
  free page), कौन-सा page और उसका कौन-सा version, और page में record का offset। WAL frames की गिनती 0 से
  होती है।
- `LostColumns` उन columns की सूची देता है जो किसी क्षतिग्रस्त record में अब नहीं बचे: उनके fields खाली रहते हैं।

Browsers SQLite के `secure_delete` के साथ delete करते हैं: database में मुक्त हुआ space शून्य से भर दिया जाता
है। इसलिए जो recover किया जा सकता है, वह ज़्यादातर पुराने page versions में होता है: वे जो `-wal` file में
अब भी हैं, और database file में उन pages की अपनी copies जिन्हें किसी `-wal` ने replace किया, अगले checkpoint
तक। `-wal` file को उसके database के साथ copy करें, और browser के फिर से चलने से पहले करें।

## विश्लेषक के लिए सावधानियाँ { #analyst-cautions }

- समय UTC में हैं। Chromium और Firefox उन्हें microsecond तक दर्ज करते हैं; Chromium का form history
  second तक।
- Visit का अर्थ यह नहीं कि उपयोगकर्ता ने टाइप या click किया: redirects, frames और अन्य programs द्वारा खोले
  गए pages भी visits होते हैं। `Transition`, `Typed` और `Hidden` पढ़ें।
- Chromium cookie values को encrypt करता है: उनके लिए `Value` खाली होता है। Juntei decrypt नहीं करता।
- `Removed` form-history row Firefox के `moz_deleted_formhistory` से आती है, जिसे Android वाला Firefox Sync के
  लिए भरता है। Desktop Firefox इसे सामान्यतः खाली छोड़ता है।
- `Low` confidence वाली deleted entry किसी अन्य record का टुकड़ा हो सकती है: इसे live rows और अन्य साक्ष्य से
  मिलाकर जाँचें।
- कोई deleted row न मिलने का अर्थ यह नहीं कि कुछ भी delete नहीं हुआ: शून्य से भरा गया space और checkpoint
  recover करने के लिए कुछ नहीं छोड़ते। Deleted cookies अभी recover नहीं की जातीं।
- डेटाबेस की सामग्री ब्राउज़र परिवार निर्धारित करती है। संगत प्रोफ़ाइल पथ `Browser` को Chrome या Firefox जैसे उत्पाद तक सीमित करता है; विरोधी पथ परिवार नहीं बदलता। अन्यथा नाम `Chromium-based`, `Firefox-based` या `Safari` है (अनिश्चित होने पर `Unknown`)। `User` और `Profile` पथ से आते हैं और फ़ाइल कॉपी करने पर खो सकते हैं।
- Rollback journals (`-journal`) नहीं पढ़े जाते, और न ही Internet Explorer तथा legacy Edge का `WebCacheV01.dat`।

## Juntei से parse करें { #parse-with-juntei }

```text
Usage: juntei parse-sqlite browser <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find History, Cookies, Web Data, places.sqlite, ...
  -f, --file <FILE>      Parse one browser database (and its -wal file)

Output:
  -o, --output <PATH>       Directory of CSV tables, or JSON/JSONL file
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-sqlite browser -d Users -o browser-csv
juntei parse-sqlite browser -d Users -t jsonl \
  | jq 'select(.Table == "Downloads") | {StartTime, URL, TargetPath, Browser, User}'
```

`-d` files को नाम से खोजता है और हर file को उसकी tables से पहचानता है: किसी अन्य program की `History` file
छोड़ दी जाती है। `-f` किसी database को पढ़ता है, उसका नाम चाहे जो हो।

## आउटपुट tables { #output-tables }

CSV चार files बनाता है; rows न होने पर हर file में केवल उसका header होता है। JSON और JSONL rows में
`SourceFile` के बाद `Table` (`History`, `Downloads`, `Cookies` या `FormHistory`) होता है।

| CSV file | सामग्री |
|---|---|
| `Browser-History.csv` | Visits, live और deleted; ऐसे deleted pages जिनका URL किसी live page का नहीं है। |
| `Browser-Downloads.csv` | Downloads, live और deleted। |
| `Browser-Cookies.csv` | Cookies। |
| `Browser-FormHistory.csv` | Form entries, live और deleted; Firefox के removal records। |

## Fields { #fields }

हर table इन fields से शुरू होती है:

| Field | व्याख्या |
|---|---|
| `SourceFile` | वह database जिससे row पढ़ी गई। |
| `User` | वह Windows उपयोगकर्ता जिसकी profile में path है (`Users` के बाद वाला folder)। |
| `Browser` | सामग्री से निर्धारित ब्राउज़र परिवार, संगत प्रोफ़ाइल पथ से अधिक विशिष्ट नाम। |
| `Profile` | Browser profile: database का folder। |
| `Status` | `Live`, `Deleted` (recover किया गया), या `Removed` (Firefox का removal record)। |

### History { #history }

| Field | व्याख्या |
|---|---|
| `RecordType` | `Visit`, या किसी ऐसे deleted page के लिए `Page` जिसके visits भी शायद मिट चुके हों। |
| `VisitTime` | Visit कब शुरू हुआ (UTC)। |
| `URL`, `Title` | Page और उसका title, जैसा आखिरी बार देखा गया। |
| `Transition` | Browser page पर कैसे पहुँचा। Chromium: core type और फिर उसके qualifiers (`LINK`, `TYPED`, `AUTO_BOOKMARK`, `FORM_SUBMIT`, `RELOAD`, … साथ में `CHAIN_START`, `SERVER_REDIRECT`, `FROM_ADDRESS_BAR`, …); Firefox: `LINK`, `TYPED`, `BOOKMARK`, `EMBED`, `REDIRECT_PERMANENT`, `REDIRECT_TEMPORARY`, `DOWNLOAD`, `FRAMED_LINK`, `RELOAD`। |
| `VisitDuration` | Page कितने seconds सामने रहा (Chromium; न मापे जाने पर 0)। |
| `VisitCount` | Page के कुल visits। |
| `Typed`, `TypedCount` | क्या URL कभी टाइप किया गया, और कितनी बार (Chromium)। |
| `Hidden` | History list से छिपा हुआ: subframes, redirect sources, errors। |
| `Frecency` | Address bar suggestions के लिए Firefox द्वारा page की ranking। |
| `VisitID`, `FromVisitID` | Visit का row id, और उस visit का row id जिससे यह आया (referring page, redirect का source)। |
| `LastVisitTime` | किसी deleted page का आखिरी visit (UTC)। |
| `PageSource` | किसी deleted visit का URL और title कहाँ से आते हैं: `Live` (कोई live page), `Recovered` (कोई deleted page) या `NotFound`। |
| `Confidence`, `RecoveredFrom`, `LostColumns` | कोई deleted entry कैसे recover हुई; [हटाई गई entries](#deleted-entries) देखें। |

### Downloads { #downloads }

| Field | व्याख्या |
|---|---|
| `StartTime`, `EndTime` | Download कब शुरू और कब समाप्त हुआ (UTC)। |
| `URL`, `RedirectChain` | File कहाँ से आई, और रास्ते का हर URL (Chromium; CSV में उनके बीच ` -> `)। |
| `TargetPath`, `CurrentPath` | File कहाँ save की गई (Firefox में एक `file://` URI), और download के दौरान वह कहाँ लिखी जा रही थी। |
| `ReceivedBytes`, `TotalBytes` | प्राप्त bytes, और file का size। |
| `State` | `Complete`, `InProgress`, `Cancelled`, `Interrupted`, `Paused`, `Blocked` या `Dirty` (malware के रूप में blocked)। |
| `DangerType` | File पर Chromium का निर्णय: 0 खतरनाक नहीं, 1 खतरनाक file type, 2 खतरनाक URL, 3 खतरनाक content, 4 संभवतः खतरनाक, 5 असामान्य, 6 उपयोगकर्ता द्वारा रखी गई, 7 खतरनाक host, 8 संभावित रूप से अवांछित, … |
| `InterruptReason` | कोई interrupted download क्यों रुका (Chromium; कोई कारण न होने पर 0)। |
| `Referrer`, `TabURL` | Referring page, और उस tab का page जिसने download शुरू किया। |
| `MimeType`, `Opened` | File का type जैसा server ने बताया, और क्या उसे browser से खोला गया (Chromium)। |
| `FileDeletedByUser` | क्या उपयोगकर्ता ने browser की download list के ज़रिए download की गई file को disk से delete किया (Firefox)। |
| `DownloadID` | Download का row id। |

### Cookies { #cookies }

| Field | व्याख्या |
|---|---|
| `Created`, `LastAccessed`, `Expires` | Cookie कब set हुई, आखिरी बार कब भेजी गई, और कब expire होगी (UTC; session cookie के लिए `Expires` खाली)। |
| `Host`, `Name`, `Path` | वह site जिसके लिए cookie है, उसका नाम और path। |
| `Value` | Stored value: Chromium द्वारा encrypt किए जाने पर खाली। |
| `Secure`, `HttpOnly`, `Persistent` | केवल HTTPS पर भेजी जाती है, scripts से छिपी रहती है, sessions के पार रखी जाती है (Chromium)। |
| `GoogleAnalytics` | किसी Google Analytics cookie (`__utma`, `__utmb`, `__utmt`, `__utmz`) के decoded fields: visit times, sessions, और campaign source तथा search terms। |
| `RowID` | Cookie का row id। |

### Form history { #form-history }

| Field | व्याख्या |
|---|---|
| `FirstUsed`, `LastUsed` | Value पहली और आखिरी बार कब दर्ज की गई (UTC)। |
| `RemovedTime` | Firefox का removal record entry के हटाए जाने का जो समय बताता है (UTC)। |
| `FieldName`, `Value` | Form field (`email`, `q`, Firefox का search bar: `searchbar-history`) और जो टाइप किया गया। |
| `Count` | Value कितनी बार दर्ज की गई। |
| `GUID` | Entry का Firefox identifier, जिसका उल्लेख removal records करते हैं। |
| `Sources` | Firefox ने entry कहाँ से save की, उन versions में जो इसे रखते हैं (CSV में vertical bar से अलग)। |
| `RowID` | प्रविष्टि की पंक्ति ID; पुनर्प्राप्ति में खोने पर खाली/null। वास्तव में दर्ज शून्य `0` रहता है। |

## स्वचालित विश्लेषण { #automatic-parsing }

`auto-parse` tables को `BrowserActivity` में `Browser-History.csv`, `Browser-Downloads.csv`,
`Browser-Cookies.csv` और `Browser-FormHistory.csv` के रूप में लिखता है (`-t json` या `-t jsonl` के साथ एक ही
`Browser.json` या `Browser.jsonl` file), साथ में `BrowserActivity/Browser-logs.txt` भी। संयुक्त `-j` timeline
में हर row शामिल होती है, `Comp` और `Artifact` identifier `browser` के साथ। `-Z` `VisitTime`, `LastVisitTime`,
`StartTime`, `EndTime`, `Created`, `LastAccessed`, `Expires`, `FirstUsed`, `LastUsed` और `RemovedTime` को
convert करता है।

## अधिक जानकारी { #further-details }

- [Chromium — page transition types](https://source.chromium.org/chromium/chromium/src/+/main:ui/base/page_transition_types.h)
- [Chromium — download danger types](https://source.chromium.org/chromium/chromium/src/+/main:components/download/public/common/download_danger_type.h)
- [Mozilla — Places database](https://firefox-source-docs.mozilla.org/browser/places/)
- [SQLite — write-ahead logging](https://www.sqlite.org/wal.html)
- [Plaso — browser SQLite parsers](https://github.com/log2timeline/plaso/tree/main/plaso/parsers/sqlite_plugins)
- [Velociraptor — SQLiteHunter](https://github.com/Velocidex/SQLiteHunter)
- [Hindsight](https://github.com/obsidianforensics/hindsight)
