# Microsoft Office (हाल की फ़ाइलें, विश्वसनीय दस्तावेज़, macro settings) { #microsoft-office-recent-files-trusted-documents-macro-settings }

## Office क्या दर्ज करता है { #what-office-records }

Microsoft Office प्रत्येक उपयोगकर्ता के लिए `NTUSER.DAT` में, और प्रत्येक संस्करण तथा application के लिए, निम्न रखता है:

- **हाल की files और स्थान:** `File MRU`, यानी application द्वारा खोले गए दस्तावेज़, और `Place MRU`,
  यानी वे folders जिनसे उसने उन्हें खोला, प्रत्येक item के साथ उसके अंतिम बार खोले जाने का समय। Microsoft 365
  प्रत्येक signed-in account के लिए एक list भी रखता है (`User MRU\<account>`)।
- **Trusted Documents:** जब कोई उपयोगकर्ता किसी दस्तावेज़ पर *Enable Editing* या *Enable Content* पर click करता है, तो
  Office उसे `TrustRecords` में याद रखता है: दस्तावेज़ का path, उसे कब trust किया गया, और क्या
  उसके macros enable किए गए थे।
- **Macro settings:** `VBAWarnings` (कौन-से macros चलते हैं), `BlockContentExecutionFromInternet`
  (क्या Internet से आई files के macros block होते हैं), `AccessVBOM` (क्या programs
  VBA projects को बदल सकते हैं) और **Trusted Locations**, यानी ऐसे folders जिनके दस्तावेज़ों के macros बिना
  पूछे चलते हैं। Group Policy यही settings `Software\Policies` के अंतर्गत सेट कर सकती है।

किसी दुर्भावनापूर्ण दस्तावेज़ को आमतौर पर उपयोगकर्ता से अपने macros enable करवाने की आवश्यकता होती है: ऐसा trust record जिसके macros
enable किए गए थे, दस्तावेज़, उपयोगकर्ता और समय को आपस में जोड़ता है।

<span class="evidence-strength">सर्वोत्तम निष्कर्ष:</span> उपयोगकर्ता ने यह दस्तावेज़ इस
application में खोला (और अंतिम बार इस समय); उपयोगकर्ता ने इस दस्तावेज़ की editing या macros को इस
समय enable किया; Office को macros इस तरह चलाने के लिए configure किया गया था।

## स्थान { #location }

```text
C:\Users\<user>\NTUSER.DAT
Software\Microsoft\Office\<version>\<application>\File MRU                   Item 1, Item 2, …
Software\Microsoft\Office\<version>\<application>\Place MRU
Software\Microsoft\Office\<version>\<application>\User MRU\<account>\File MRU
Software\Microsoft\Office\<version>\<application>\Security\Trusted Documents\TrustRecords
Software\Microsoft\Office\<version>\<application>\Security                   VBAWarnings, …
Software\Microsoft\Office\<version>\<application>\Security\Trusted Locations\LocationN
Software\Policies\Microsoft\Office\<version>\<application>\Security          by policy
```

संस्करण: `16.0` Office 2016 और बाद के (2019, 2021, 2024, Microsoft 365) हैं, `15.0` Office 2013,
`14.0` Office 2010, `12.0` Office 2007।

एक item `[F00000000][T01D86AF6D3C287B0][O00000000]*C:\Users\alice\report.docx` जैसा दिखता है: `T`
अंतिम बार खोले जाने का समय है (hexadecimal में एक FILETIME), path `*` के बाद आता है। Trust record 24
bytes का होता है: वह FILETIME जब दस्तावेज़ को trust किया गया, फिर अंतिम 4 bytes में उसके flags,
macros enable होने पर `0x7FFFFFFF` और केवल editing enable होने पर `0x00000001`।

## विश्लेषक के लिए सावधानियाँ { #analyst-cautions }

- **समय।** `LastOpenedUTC` प्रत्येक item का अपना समय है। `TrustedAtUTC` वह समय है जब उपयोगकर्ता ने
  दस्तावेज़ को trust किया, यानी वह क्षण जब *Enable Editing* या *Enable Content* पर click किया गया, जो
  सबसे हाल के खोले जाने से पहले का हो सकता है।
- **Macros enable हुए, चले नहीं।** `MacrosEnabled` बताता है कि उपयोगकर्ता ने दस्तावेज़ के macros enable किए; यह
  सिद्ध नहीं करता कि कोई macro चला या उसने क्या किया। Process creation, Prefetch, PowerShell
  logs और स्वयं दस्तावेज़ से correlate करें।
- **Paths वैसे ही जैसे Office उन्हें संग्रहीत करता है।** Trust records दस्तावेज़ को ऐसे path से दर्शाते हैं जिसमें
  `%USERPROFILE%`, forward slashes, `file:///` URL और URL encoding (`%20`) हो सकते हैं। Cloud दस्तावेज़ों के
  MRU items URLs होते हैं।
- **Defaults और प्रभावी policy।** Office default trusted locations (`Templates`,
  `XLSTART`, `STARTUP`, …) बनाता है। किसी location का `Date` एक उपयोगी संकेत है, यह प्रमाण नहीं कि उसे किसने बनाया:
  administrators, scripts, policies और copy की गई registry settings भी उसे भर सकती हैं। केवल
  `VBAWarnings` का अनुपस्थित होना प्रभावी macro policy स्थापित नहीं करता। Office संस्करण, Internet
  file marking और अन्य management settings भी मायने रखती हैं।
  [Microsoft Internet macro blocking को समझाता है](https://learn.microsoft.com/en-us/DeployOffice/security/internet-macros-blocked)।
- **हटाए गए दस्तावेज़।** Lists दस्तावेज़ों से अधिक समय तक रहती हैं; अनुपस्थित file भी यहाँ
  दिख सकती है। Lists छोटी होती हैं और Office के *Clear unpinned items* तथा cleanup tools से साफ़ हो जाती हैं।

## Juntei से parse करें { #parse-with-juntei }

```text
Usage: juntei parse-registry office <INPUT> [OPTIONS]

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
juntei parse-registry office -d evidence/ -o office.csv
juntei parse-registry office -f NTUSER.DAT -t jsonl | jq 'select(.MacrosEnabled == true)'
```

Summary उन दस्तावेज़ों को सूचीबद्ध करता है जिनके macros enable किए गए थे, ऐसी settings जो macros को अधिक
स्वतंत्रता से चलने देती हैं (`VBAWarnings` 1, `BlockContentExecutionFromInternet` 0, `AccessVBOM` 1,
`AllowNetworkLocations` 1), उपयोगकर्ताओं द्वारा जोड़ी गई या policy द्वारा सेट trusted locations, और अंतिम बार
खोले गए दस्तावेज़।

## आउटपुट fields { #output-fields }

| Field | व्याख्या |
|---|---|
| `Host`, `User`, `UserSID`, `SourcePath` | Host, profile का स्वामी (path या shell folders से), system के `ProfileList` से SID, और hive। |
| `OfficeVersion`, `App` | संस्करण की key (`16.0`) और application (`Word`, `Excel`, …)। |
| `List` | `File MRU`, `Place MRU`, `TrustRecords`, `Security` या `Trusted Locations`। |
| `Account` | किसी `User MRU` list का Microsoft 365 account। |
| `Policy` | क्या setting या location policy द्वारा सेट है। |
| `MRU-Position`, `IsMostRecent` | List में item का स्थान (`Item 1` 0 है, सबसे हाल का)। |
| `Path` | दस्तावेज़ या folder; trusted document का संग्रहीत path; trusted location का `Path`। |
| `LastOpenedUTC` | Item अंतिम बार कब खोला गया। |
| `TrustedAtUTC`, `MacrosEnabled`, `TrustFlags` | दस्तावेज़ कब trust किया गया, क्या उसके macros enable किए गए थे, और record के flags। अज्ञात flags `MacrosEnabled` को खाली छोड़ते हैं और एक diagnostic उत्पन्न करते हैं। |
| `Setting`, `Value` | एक macro setting और उसकी value, उसके अर्थ के साथ (`1 (all macros enabled)`); trusted location की key (`Location3`) और उसकी अन्य values (`AllowSubFolders=1; Date=…; Description=…`)। |
| `KeyPath`, `KeyLastWrittenUTC` | Key और उसका अंतिम write। |
| `DataRaw` | Decode न होने पर value के bytes (hex)। |
| `TransactionLogStatus`, `LogsApplied` | Registry recovery का संदर्भ। |
| `Problems` | बिना `*` वाले या अमान्य समय वाले items, 24 bytes के न होने वाले trust records, बिना `Path` वाली locations, गलत types। |

## जाँच की रणनीति { #investigation-strategy }

1. `MacrosEnabled` true से शुरू करें: कौन-से दस्तावेज़, कब, और किसके द्वारा। Files (`$MFT`,
   USN Journal, `Zone.Identifier`), उनका स्रोत (browser और mail history) और उसके बाद क्या चला, यह खोजें।
2. Macro settings और trusted locations जाँचें: सभी macros enabled, Internet macros का
   block न होना, VBA project access, या उपयोगकर्ता द्वारा जोड़ी गई location (`Downloads`, `Temp`, किसी network share में)
   defaults को कमज़ोर करते हैं।
3. दस्तावेज़ों के paths के लिए, removable और network locations सहित, `LastOpenedUTC` को LNK files, jump lists और RecentDocs
   के साथ timeline पर रखें।

## अधिक जानकारी { #further-details }

- [Microsoft: Enable or disable macros in Microsoft 365 files](https://support.microsoft.com/en-us/office/enable-or-disable-macros-in-microsoft-365-files-12b036fd-d140-4e74-b45e-16fed1a7e5c6)
- [Microsoft: Trusted documents](https://support.microsoft.com/en-us/office/trusted-documents-cf872bd8-47ec-4c02-baa5-1fdba1a11b53)
- [Microsoft: Add, remove, or change a trusted location](https://support.microsoft.com/en-us/office/add-remove-or-change-a-trusted-location-in-microsoft-office-7ee1cdc2-483e-4cbb-bcb3-4e7c67147fb4)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
