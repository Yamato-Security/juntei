# SAM स्थानीय खाते { #sam-local-accounts }

## SAM में क्या होता है { #what-the-sam-contains }

Security Account Manager (`SAM`) hive स्थानीय account records संग्रहीत करता है। Juntei account
names, RIDs, profile/comment fields, account-control flags, last-logon/password timestamps, counters,
password hints, Microsoft-account linkage, profile paths, और यह कि क्या Windows account को
sign-in UI से छिपाता है, decode करता है।

यह नए बनाए गए स्थानीय administrators, enable किए गए निष्क्रिय accounts, password
बदलाव, छिपे हुए accounts और lateral-movement staging accounts खोजने में उपयोगी है।

<span class="evidence-strength">सर्वोत्तम निष्कर्ष:</span> स्थानीय SAM ने account को
दर्ज state के साथ प्रस्तुत किया था। यह पूर्ण authentication इतिहास नहीं है और domain accounts का वर्णन नहीं करता।

## स्थान और संग्रह { #location-and-collection }

```text
C:\Windows\System32\config\SAM
C:\Windows\System32\config\SAM.LOG1
C:\Windows\System32\config\SAM.LOG2
```

Hive और logs को पर्याप्त privileges/raw access के साथ संग्रहित करें। Host की पहचान संग्रह में
संबंधित `SYSTEM` hive से समृद्ध की जा सकती है।

## विश्लेषक के लिए सावधानियाँ { #analyst-cautions }

- यह command **स्थानीय** accounts को parse करता है, Active Directory domain users को नहीं।
- Juntei जानबूझकर password hashes extract नहीं करता।
- RID 500 built-in Administrator account है, भले ही उसका नाम बदला गया हो; केवल नाम पर्याप्त नहीं है।
- `LastLogon` और counters registry state हैं, प्रत्येक session का पूर्ण audit trail नहीं।
- `HiddenFromLogonUI` किसी account को sign-in screen से छिपाता है; यह account को disable नहीं करता।
- Built-in `DefaultAccount` और `WDAGUtilityAccount` वैध रूप से छिपे हो सकते हैं।
- क्षतिग्रस्त/अनुपस्थित `F` value से आए null account flags की व्याख्या enabled के रूप में नहीं की जानी चाहिए।
- `NamesKeyLastWrittenUTC` account-name registration संदर्भ का अनुमान देता है, लेकिन यह गारंटीकृत creation time नहीं है।

## Juntei से parse करें { #parse-with-juntei }

```text
Usage: juntei parse-registry sam <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SAM hives
  -f, --file <FILE>      Parse one SAM hive
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Filtering:
      --rid <RID>  Parse one decimal RID (example: 500)

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry sam -f SAM -o accounts.csv
juntei parse-registry sam -f SAM --rid 500 -t jsonl | jq .
juntei parse-registry sam -d evidence/ -t jsonl \
  | jq 'select(.HiddenFromLogonUi == true or (.AccountFlags | index("disabled") | not))'
```

## आउटपुट fields { #output-fields }

| Field | व्याख्या |
|---|---|
| `Host`, `SourcePath` | Host संदर्भ और सटीक SAM hive। |
| `RID` | Relative Identifier। Account पहचान के लिए मज़बूत pivot; 500 built-in Administrator है। |
| `User` | वर्तमान स्थानीय account नाम। Rename किया जा सकता है; RID से correlate करें। |
| `FullName`, `Comment`, `UserComment` | वर्णनात्मक text; attacker द्वारा नियंत्रित किया जा सकता है और masquerading की समीक्षा के लिए उपयोगी है। |
| `AccountFlags` | Decode की गई states, जैसे disabled, locked out, password not required, normal account, password never expires। |
| `AccountFlagsValue` | सत्यापन और असमर्थित bits के लिए raw संख्यात्मक flags। |
| `CountryCode`, `CodePage` | Account का locale/code-page metadata; आमतौर पर संदर्भ, attribution नहीं। |
| `LastLogonUTC`, `LastLogonWholeSecondsUTC` | वही last-logon state, पूर्ण precision में और whole-second compatibility में। पूर्ण इतिहास नहीं। |
| `PasswordLastSetUTC` | दर्ज password-change time। अप्रत्याशित रूप से हाल के/पुराने values की जाँच करें। |
| `AccountExpirationUTC` | सीमित होने पर configure की गई expiry। Null का अर्थ never/not set हो सकता है। |
| `LastFailedLogonUTC` | SAM state में दर्ज अंतिम विफल स्थानीय logon; Security events से पुष्टि करें। |
| `FailedLogonCount`, `LogonCount` | Windows-विशिष्ट update व्यवहार वाले cumulative/state counters; संदर्भ के रूप में उपयोग करें, सटीक event गिनती के रूप में नहीं। |
| `PasswordHint` | उपयोगकर्ता द्वारा दिया गया संवेदनशील hint; password patterns उजागर कर सकता है। उचित रूप से संभालें। |
| `InternetUserName` | मौजूद होने पर linked Microsoft account की पहचान। |
| `HiddenFromLogonUI` | UI-hiding state। उच्च RIDs/अप्रत्याशित custom accounts की समीक्षा करें; इसका अर्थ disabled नहीं है। |
| `HomeDirectory`, `HomeDrive`, `ProfilePath`, `LogonScript` | स्थानीय account का environment/logon configuration; असामान्य scripts/UNC paths persistence का संकेत दे सकते हैं। |
| `KeyLastWrittenUTC` | RID account-key के बदलने का समय। Key-स्तर का, field-विशिष्ट नहीं। |
| `NamesKeyLastWrittenUTC` | Username mapping-key के बदलने का समय; account lifecycle का उपयोगी अनुमान। |
| `LastLogonFiletime`, `PasswordLastSetFiletime`, `AccountExpirationFiletime`, `LastFailedLogonFiletime` | सत्यापन और विशेष sentinel handling के लिए रखी गई raw 64-bit FILETIME values। |
| `TransactionLogStatus`, `LogsApplied` | Registry replay का संदर्भ। |
| `Problems` | F/V/Names के बीच असहमति, अनुपस्थित entries, malformed offsets, या decoding क्षति। |

## जाँच की रणनीति { #investigation-strategy }

1. RID, नाम, flags, profile path और `NamesKeyLastWrittenUTC` की तुलना host baseline से करें।
2. Enabled RID 1000+ accounts, password-not-required, never-expiring, छिपे हुए, या हाल में बदले गए accounts की समीक्षा करें।
3. Security 4720/4722/4724/4732 events, logon events, profile creation और remote गतिविधि से correlate करें।
4. यदि कोई account संग्रह से पहले हटा दिया गया था, तो registry recovery इतिहास खोजें।
5. वैध provisioning/default accounts को attacker द्वारा बनाए गए interactive accounts से अलग पहचानें।

## अधिक जानकारी { #further-details }

- [Microsoft Open Specifications: SAM Remote Protocol](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-samr/)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
- [dissect.target SAM plugin](https://docs.dissect.tools/en/latest/projects/dissect.target/plugins/os/windows/sam.html)
