# आउटपुट प्रारूप { #output-formats }

## Terminal summaries { #terminal-summaries }

`-o` या किसी संरचित `-t` के बिना, commands आमतौर पर counts, time ranges, क्षति और उच्च
मूल्य वाले findings दिखाते हैं। Summaries बड़ा आउटपुट बनाने से पहले दायरा समझने में आपकी मदद करते हैं।

## CSV { #csv }

CSV default सहेजा जाने वाला format है। यह spreadsheet tools, SQL import और timeline
viewers के लिए सुविधाजनक है। कुछ parsers कई tables बनाते हैं क्योंकि उनके आर्टिफ़ैक्ट में अलग-अलग record
types होते हैं (Amcache, Jump Lists और SRUM)।

!!! warning "साक्ष्य columns को text मानें"
    कोई path या command line `=`, `+`, `-` या `@` से शुरू हो सकती है। Spreadsheet software
    इसे formula के रूप में व्याख्या कर सकता है। सभी fields को text के रूप में import करें या forensic-safe CSV viewer का उपयोग करें। Juntei
    साक्ष्य strings को बदलने के बजाय सुरक्षित रखता है।

## Field नाम { #field-names }

CSV column headers और JSON/JSONL field names PascalCase का उपयोग करते हैं, जिनमें Amcache और
`auto-parse` manifests भी शामिल हैं। Acronyms uppercase रहते हैं: `ProcessID`, `TimestampUTC`, `UserSID`,
`SHA256`। Acronym के बाद आने वाला शब्द hyphen से अलग किया जाता है: `IP-Address`, `MFT-Entry`,
`USN-Journal`। `jq` के साथ, hyphen वाले field को quote करें: `.["IP-Address"]`।

Default MFT CSV छोटे, पठनीय headings का उपयोग करता है, जैसे `EntryNum`, `Ext`, `Dir`, `HasADS`,
`Size`, `Created (SI)` और `Created (FN)`। इसके पूर्ण CSV schema के लिए [MFT field reference](../artifacts/mft.md)
देखें। Timestamp pairs MACB क्रम का पालन करते हैं: DataModified, Accessed,
MetadataChanged, Created, जिसमें SI, FN से पहले आता है। CSV tools में spaces या punctuation वाले headings को
quote करें।

साक्ष्य values अपनी मूल spelling बनाए रखती हैं। Source द्वारा दी गई dictionary keys, जैसे `Details`/`DataRaw` के अंतर्गत registry
value names और `ZoneIdentifier` के अंतर्गत `Zone.Identifier` names, भी
अपरिवर्तित रहती हैं। SRUM की `ValueState` और `Raw` dictionaries अपने data
columns जैसे ही labels का उपयोग करती हैं। Normalization collision की रिपोर्ट की जाती है, किसी field को चुपचाप छोड़ा नहीं जाता।

यह पिछले export schemas को बदलता है। मौजूदा परिणामों को फिर से उत्पन्न करें और सहेजी गई queries update करें
(उदाहरण के लिए, `.record_type` `.RecordType` बन जाता है, और Amcache `full_path` `FullPath` बन जाता है)।
मानव-पठनीय text reports और input/configuration formats अपनी मौजूदा परिपाटियाँ बनाए रखते हैं।

## संक्षिप्त field नाम { #abbreviated-field-names }

निम्नलिखित labels CSV, JSON और JSONL exports पर लागू होते हैं, जिनमें स्वचालित parsing और
manifests भी शामिल हैं। पुराने परिणामों को फिर से उत्पन्न करते समय सहेजे गए `jq`, `xsv` और spreadsheet filters update करें।
केवल output labels बदलते हैं; साक्ष्य values और raw registry-value names अपनी मूल spelling बनाए रखते हैं।

| पिछला label | वर्तमान label |
|---|---|
| `ADS` | `HasADS` |
| `Application` | `App` |
| `ApplicationCompany` | `AppCompany` |
| `ApplicationCompanyRaw` | `AppCompanyRaw` |
| `ApplicationPath` | `AppPath` |
| `Arguments` | `Args` |
| `Attributes` | `Attr` |
| `AttributeType` | `AttrType` |
| `BytesRecvd` | `BytesReceived` |
| `Computer` | `Comp` |
| `ComputerName` | `CompName` |
| `ComputerNameLastWrittenUTC` | `CompNameLastWrittenUTC` |
| `Directories` | `Dir` |
| `FileName` | `Filename` |
| `MicrosecondZeros` | `ZeroU-Secs` |
| `SI-BeforeFN` | `SI<FN` |
| `UserName` | `User` |
| `Username` | `User` |
| `UserNameSource` | `UserSource` |
| `Users` | `User` |

Native system-information JSON time `CompNameLastWritten` है (CSV:
`CompNameLastWrittenUTC`)। `HasADS` एक Boolean संकेतक है; Prefetch में `Dir` में
दर्ज directory list होती है। `User` अपना आर्टिफ़ैक्ट-विशिष्ट type बनाए रखता है: account name के लिए string,
या USB आउटपुट में संबंधित उपयोगकर्ताओं की list। यह किसी execute करने वाली पहचान का संकेत नहीं देता।
Queries में punctuation को quote करें: `."SI<FN"` और `.["ZeroU-Secs"]`।

## JSON { #json }

JSON पूरे परिणाम को एक document के रूप में संग्रहीत करता है। यह arrays, nested structures, nulls,
बड़े integers और ऐसे भेद सुरक्षित रखता है जिन्हें flat CSV हमेशा प्रदर्शित नहीं कर सकता।

```bash
juntei parse-file lnk -f sample.lnk -o lnk.json -t json
```

## JSON Lines { #json-lines }

JSONL प्रति line एक स्वतंत्र JSON object लिखता है। यह अच्छी तरह stream होता है और `jq` के साथ स्वाभाविक रूप से काम करता है:

```bash
juntei parse-registry services -d evidence/ -t jsonl \
  | jq -c 'select(.StartType == "automatic") | {Host, ServiceName, ImagePath}'
```

जब संरचित आउटपुट standard output पर जाता है, तो Juntei progress और messages को standard
error पर भेजता है ताकि pipe को केवल डेटा मिले।

`auto-parse -j timeline.jsonl` वर्गीकृत CSV परिणामों को बनाए रखते हुए एक पूर्ण multi-host
संग्रह के लिए एक अतिरिक्त JSONL dataset बनाता है। अलग-अलग परिणामों का format बदलने के लिए `-t json` या `-t jsonl` जोड़ें।
प्रत्येक record में `Comp` और `Artifact` fields होते हैं ताकि SIEM ingestion के बाद भी उसका स्रोत स्पष्ट रहे।

## UTC और precision { #utc-and-precision }

जहाँ format अनुमति देता है, Juntei parse किए गए समय को RFC 3339 का उपयोग करके UTC में format करता है। कुछ आर्टिफ़ैक्ट में
100-nanosecond precision होती है; अन्य seconds, FAT time, aggregation intervals संग्रहीत करते हैं, या कोई समय नहीं रखते।
अंक सटीकता नहीं हैं: हमेशा उसके आर्टिफ़ैक्ट page पर वर्णित source field को समझें।

UTC Windows event-log रूप `2026-10-20T08:46:38.2001668Z` का उपयोग करता है, उपलब्ध
fractional precision को बनाए रखते हुए। `auto-parse -Z "+09:00"` parse किए गए क्षणों को अपने सभी परिणाम formats और
वैकल्पिक `-j` export में एक स्पष्ट निश्चित offset के साथ दिखाता है। Default UTC है, चाहे Juntei
किसी भी computer पर चल रहा हो। Raw values और बिना zone वाली wall clocks जैसी दर्ज हैं वैसी ही रहती हैं;
[output time zones](../commands/auto-parse.md#output-time-zone) देखें।

## क्षति और अनुपस्थित values { #damage-and-missing-values }

खाली, `null`, `NA`, `Problems` और parser-विशिष्ट state fields अर्थपूर्ण हैं। इनका अर्थ हो सकता है कि
आर्टिफ़ैक्ट ने कोई value संग्रहीत नहीं की, record क्षतिग्रस्त था, format इसका समर्थन नहीं करता, या
parser इसे resolve नहीं कर सका। सभी खाली values को zero से न बदलें।
