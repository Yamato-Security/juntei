# ऐप्लिकेशन इतिहास { #application-history }

## टूल अपने पीछे क्या छोड़ते हैं { #what-tools-leave-behind }

कई third-party टूल अपनी settings और history उपयोगकर्ता के `NTUSER.DAT` में रखते हैं। इनमें से कई ऐसे
टूल हैं जिनका उपयोग हमलावर करते हैं, या जिनसे वे data ले जाने से पहले उसे तैयार (stage) करते हैं: Sysinternals टूल, archivers,
cloud sync clients और remote access टूल। Juntei इन्हें पढ़ता है:

| ऐप्लिकेशन | `HKCU\Software` के अंतर्गत key | यह क्या दिखाता है |
|---|---|---|
| Sysinternals | `Sysinternals\<tool>` | हर वह टूल जिसका licence उपयोगकर्ता ने स्वीकार किया (`EulaAccepted`): PsExec, ProcDump, Sysmon, … |
| WinRAR | `WinRAR\ArcHistory`, `DialogEditHistory\ArcName`, `DialogEditHistory\ExtrPath` | खोले गए archives; इसके dialogs में टाइप किए गए archive नाम और extraction फ़ोल्डर |
| 7-Zip | `7-Zip\Compression\ArcHistory`, `Extraction\PathHistory`, `FM\FolderHistory`, `FM\CopyHistory`, `FM\PanelPath0/1` | बनाए गए archives; extraction फ़ोल्डर; इसके file manager में browse किए गए और जिनमें copy किया गया वे फ़ोल्डर |
| WinZip | `Nico Mak Computing\WinZip\filemenu`, `extract`, `mru\archives` | इसके File menu पर archives, extraction फ़ोल्डर, हाल के archives |
| OneDrive | `Microsoft\OneDrive\Accounts\<account>` | हर account का e-mail पता, स्थानीय फ़ोल्डर और नाम; sync की गई libraries (`ScopeIdToMountPointPathCache`) और फ़ोल्डर (`Tenants\<tenant>`) |
| Adobe Acrobat और Reader | `Adobe\{Acrobat Reader,Adobe Acrobat}\<version>\AVGeneral\cRecentFiles`, `cRecentFolders` | हाल में खोली गई PDF फ़ाइलें और फ़ोल्डर, फ़ाइल नाम, आकार और तारीखों के साथ |
| TeamViewer | `TeamViewer` (और पुराने version का `Version<N>`) | हर value: account और user नाम, file transfer के शुरुआती फ़ोल्डर (`<partner ID>?<local>|<remote>`), … |
| CCleaner | `Piriform\CCleaner` | हर value: इसे क्या साफ़ करने के लिए set किया गया है, `UpdateKey`, … |

<span class="evidence-strength">सर्वोत्तम निष्कर्ष:</span> इस उपयोगकर्ता ने टूल चलाया (या उसका
licence स्वीकार किया), या टूल ने ये archives और फ़ोल्डर खोले, बनाए या extract किए, या यह cloud account
उपयोगकर्ता के लिए set up किया गया था। अधिकांश histories में प्रति entry कोई समय नहीं होता: केवल सबसे हाल की entry को
उसकी list key का last-write समय मिलता है।

## स्थान { #location }

```text
C:\Users\<user>\NTUSER.DAT
  Software\Sysinternals\<tool>                         EulaAccepted
  Software\WinRAR\ArcHistory                           0, 1, … (0 the most recent)
  Software\7-Zip\Compression                           ArcHistory (REG_BINARY UTF-16 list)
  Software\Microsoft\OneDrive\Accounts\<account>       UserEmail, UserFolder, …
  Software\Adobe\Acrobat Reader\<version>\AVGeneral\cRecentFiles\c1, c2, …
```

हर उपयोगकर्ता का `NTUSER.DAT` उसके transaction logs के साथ एकत्र करें, और उपयोगकर्ताओं के SIDs के लिए
सिस्टम का `SOFTWARE` भी।

## विश्लेषक के लिए सावधानियाँ { #analyst-cautions }

- **Sysinternals EULA की स्वीकृति।** `FirstUsedUTC` key के last-write समय से निकाला गया पहले उपयोग का
  अनुमान है, जब उसकी एकमात्र value `1` पर set एक पढ़ने योग्य `EulaAccepted` हो। अस्वीकार किए गए या
  क्षतिग्रस्त EULA को यह अनुमान नहीं मिलता। Registry settings को पहले से configure, copy या edit भी
  किया जा सकता है: निष्पादन और उसके समय की पुष्टि अन्य आर्टिफ़ैक्ट से करें।
  जो टूल अपनी settings वहीं रखते हैं (Process Explorer, Autoruns, …) वे बाद में key को फिर से लिखते हैं:
  तब `FirstUsedUTC` खाली रहता है और `KeyLastWrittenUTC` केवल settings का अंतिम बदलाव होता है। `-accepteula`
  के साथ, या किसी अन्य उपयोगकर्ता के रूप में चलाया गया टूल (PsExec का remote पक्ष SYSTEM के रूप में चलता है)
  उसी उपयोगकर्ता की key लिखता है।
- **स्थिति, समय नहीं।** List entries क्रमबद्ध होती हैं (`MruPosition` 0 सबसे हाल की है), लेकिन
  केवल list key का last write सबसे हाल की entry का समय बताता है। पुरानी entries पहले, किसी
  अज्ञात समय पर उपयोग की गई थीं।
- **टाइप किए गए नाम।** WinRAR का `ArcName` archive बनाते समय टाइप किया गया archive नाम है: अक्सर बिना फ़ोल्डर
  का केवल फ़ाइल नाम।
- **Adobe paths** वैसे ही रखे जाते हैं जैसे Adobe उन्हें लिखता है (`/C/Users/alice/a.pdf` का अर्थ
  `C:\Users\alice\a.pdf` है)। इसका `sDate` (`Details` में) एक PDF date string है, जैसा लिखा गया वैसा ही रखा जाता है।
- **CCleaner का `UpdateKey`** बिना time zone का स्थानीय समय है; इसे जैसा लिखा गया वैसा ही रखा जाता है।
- **TeamViewer।** Connection logs फ़ाइलें हैं (`Connections_incoming.txt`, `TeamViewer*_Logfile.log`),
  registry values नहीं: उन्हें भी एकत्र करें।
- **अनुपस्थिति।** उपयोगकर्ता और टूल इन histories को साफ़ कर सकते हैं (CCleaner स्वयं ऐसा कर सकता है)।

## Juntei से parse करें { #parse-with-juntei }

```text
Usage: juntei parse-registry app-history <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find NTUSER.DAT hives
  -f, --file <FILE>      Parse one NTUSER.DAT hive
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry app-history -d evidence/ -o app-history.csv
juntei parse-registry app-history -f NTUSER.DAT -t jsonl \
  | jq 'select(.App == "Sysinternals" and .FirstUsed != null)'
```

Summary में सबसे नए Sysinternals पहले-उपयोग अनुमान, हर list द्वारा सबसे हाल में उपयोग किए गए archives और फ़ोल्डर,
और OneDrive accounts सूचीबद्ध होते हैं।

## आउटपुट fields { #output-fields }

| Field | व्याख्या |
|---|---|
| `Host`, `SourcePath` | Host और `NTUSER.DAT`। |
| `User`, `UserSID` | Profile फ़ोल्डर का नाम, और सिस्टम के `ProfileList` से उसका SID। |
| `App` | `Sysinternals`, `WinRAR`, `7-Zip`, `WinZip`, `OneDrive`, `Adobe Acrobat Reader`, `Adobe Acrobat`, `TeamViewer`, `CCleaner`। |
| `Item` | Sysinternals टूल, list (`ArcHistory`, `ExtrPath`, `FolderHistory`, `filemenu`, `cRecentFiles`, …), OneDrive account या उसके sync किए गए फ़ोल्डरों की key, या value का नाम। |
| `MRU-Position` | अपनी list में entry की स्थिति, 0 सबसे हाल की। |
| `Path` | Archive, फ़ोल्डर या document। |
| `Value` | `EulaAccepted`, किसी OneDrive account का e-mail पता या sync की गई library का id, कोई TeamViewer या CCleaner value। |
| `FirstUsedUTC` | EULA स्वीकृति से पहले उपयोग का अनुमान (ऊपर सावधानियाँ देखें)। |
| `Details` | किसी OneDrive account का `DisplayName`, `UserCID`, `ConfiguredTenantId`, …; किसी Adobe document का `Version`, `tFileName`, `uFileSize`, `uPageCount`, `sDate`। |
| `KeyPath`, `KeyLastWrittenUTC`, `ValueName` | Record कहाँ से आता है, और वह key अंतिम बार कब बदली। |
| `DataRaw` | Value के bytes (hex), जब वे decode नहीं होते। |
| `TransactionLogStatus`, `LogsApplied` | Registry recovery का context। |
| `Problems` | ऐसा data जिसे decode या पढ़ा नहीं जा सका। |

## जाँच की रणनीति { #investigation-strategy }

1. ऐसे Sysinternals टूल देखें जिन्हें सामान्य उपयोगकर्ता नहीं चलाता (PsExec, ProcDump, PsLoggedOn,
   SDelete) और उनके EULA timestamps।
2. घटना के आसपास के archive नामों और extraction फ़ोल्डरों की समीक्षा करें: data के सिस्टम से बाहर जाने से पहले
   `ProgramData`, `Temp` या किसी उपयोगकर्ता के फ़ोल्डर में बनाए गए archives।
3. ऐसे OneDrive accounts जाँचें जो संगठन के नहीं हैं, और TeamViewer account नाम तथा file
   transfer फ़ोल्डर।
4. Prefetch, Amcache, BAM और UserAssist (टूल का चलना), `$MFT` और USN
   Journal (archive की फ़ाइल), LNK फ़ाइलों और jump lists (खोले गए documents), और event logs से correlate करें।

## अधिक जानकारी { #further-details }

- [Microsoft: Sysinternals](https://learn.microsoft.com/en-us/sysinternals/)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
