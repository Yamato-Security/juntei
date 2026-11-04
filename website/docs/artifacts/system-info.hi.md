# सिस्टम जानकारी { #system-information }

## यह क्या है { #what-it-is }

किसी system के चार machine hives वह सब बताते हैं जो किसी विश्लेषक को उसकी timeline पढ़ने से पहले चाहिए:

- उसके नाम (NetBIOS, DNS host name, DNS domain) और वह domain या workgroup जिसमें वह है;
- उसका Windows संस्करण, build और update revision, और Windows कब install या अंतिम बार upgrade हुआ;
- उसका time zone, जो बताता है कि system ने कौन-से local times रखे;
- वह अंतिम बार कब साफ़ तरीके से shut down हुआ;
- उसका processor architecture और code pages;
- अंतिम बार sign in करने वाला उपयोगकर्ता, और sign in कर चुके हर उपयोगकर्ता के profiles;
- उसके SIDs: computer का (स्थानीय accounts इसी के साथ एक RID होते हैं) और उसके domain का;
- वे Windows संस्करण जिनसे इसे upgrade किया गया था।

`parse-registry system-info` प्रत्येक system के लिए एक record लिखता है जिसमें ये सब होते हैं।

<span class="evidence-strength">सर्वोत्तम निष्कर्ष:</span> configuration वैसा जैसा वह hives के
संग्रह (या अंतिम write) के समय था। Key times बताते हैं कि उसके भाग अंतिम बार कब बदले।

## स्थान { #location }

```text
C:\Windows\System32\config\SYSTEM     (the control set Windows used last: Select\Current)
  ControlSet00N\Control\ComputerName\ComputerName     ComputerName
  ControlSet00N\Services\Tcpip\Parameters             Hostname, Domain
  ControlSet00N\Control\TimeZoneInformation           TimeZoneKeyName, Bias, ActiveTimeBias, …
  ControlSet00N\Control\Windows                       ShutdownTime
  ControlSet00N\Control\Session Manager\Environment   PROCESSOR_ARCHITECTURE
  ControlSet00N\Control\Nls\CodePage                  ACP, OEMCP
  Setup\Source OS (Updated on M/D/YYYY HH:MM:SS)      the version before each upgrade

C:\Windows\System32\config\SOFTWARE
  Microsoft\Windows NT\CurrentVersion                 ProductName, EditionID, DisplayVersion,
                                                      CurrentBuild, UBR, InstallDate, InstallTime, …
  Microsoft\Windows NT\CurrentVersion\ProfileList     one key per profile (SID)
  Microsoft\Windows\CurrentVersion\Authentication\LogonUI   LastLoggedOnUser, …

C:\Windows\System32\config\SAM
  SAM\Domains\Account                                 V: the computer's SID at its end

C:\Windows\System32\config\SECURITY
  Policy\PolAcDmN, PolAcDmS                           the computer's name and SID
  Policy\PolPrDmN, PolPrDmS                           its domain (or workgroup) and the domain's SID
  Policy\PolDnDDN, PolDnTrN, PolDnDmG                 the domain's and forest's DNS names, GUID
  Policy\PolMachineAccountS                           the computer account's SID in the domain
```

चारों hives को उनके transaction logs के साथ संग्रहित करें। इनमें से कोई भी अकेला hive वही देता है जो उसमें है।

## विश्लेषक के लिए सावधानियाँ { #analyst-cautions }

- **Registry times UTC में होते हैं; time zone बाकी सब के लिए है।** Key times, FILETIMEs और अधिकांश
  event times UTC में होते हैं। Text logs, कुछ application logs, और नामों या strings में लिखी तिथियाँ
  (जैसे `Source OS (Updated on …)`) local हो सकती हैं। `Bias` local standard
  time से UTC तक के minutes है (`UTC = local + Bias`)। `UtcOffset` और `DaylightUtcOffset` दोनों offsets देते हैं,
  और `TimeZoneDescription` बताता है कि daylight time कब शुरू और समाप्त होता है।
- **Time zone बदल सकता है।** `TimeZoneLastWrittenUTC` key का अंतिम write है। Windows
  `ActiveTimeBias` (उस समय लागू offset) को update करते समय key को स्वयं भी फिर से लिखता है, इसलिए
  हाल का समय यह सिद्ध नहीं करता कि किसी ने zone बदला। Event logs से तुलना करें: Security 4616
  (system time बदला) और System log के `Microsoft-Windows-Kernel-General` events।
- **`StandardStart` और `DaylightStart` kernel के `TIME_FIELDS` हैं, `SYSTEMTIME` नहीं।**
  Weekday उनका अंतिम field है, तीसरा नहीं (जैसा `SOFTWARE` में zone की `TZI` value में होता है)।
  US zones में दोनों layouts एक जैसे पढ़े जाते हैं, क्योंकि weekday रविवार (0) है। Juntei इन्हें
  `TIME_FIELDS` के रूप में decode करता है: एक सार्वजनिक Easter Island system (पहला शनिवार 22:00 बजे)
  यह अंतर दिखाता है। Month का 0 होने का अर्थ है कोई daylight saving time नहीं।
- **Windows 11 "Windows 10" कहता है।** Windows 11 पर `ProductName` में `Windows 10 …` ही रहता है। Builds 22000
  और बाद के Windows 11 हैं, और summary यही बताता है। Windows 8.1, 10
  और 11 पर `CurrentVersion` `6.3` होता है; `Version` उपलब्ध होने पर `CurrentMajorVersionNumber` (10) का उपयोग करता है।
- **`InstallDate` feature upgrades के साथ बदलता है।** किसी नए release पर in-place upgrade (उदाहरण के लिए 1809 से
  21H2) इसे reset कर देता है। प्रत्येक upgrade द्वारा बदला गया release, अपने स्वयं के `InstallDate` के साथ,
  `PreviousVersions` (`Setup\Source OS`) में होता है। इनमें से सबसे पुराना `InstallDate` मूल
  installation के सबसे निकट है। किसी `Source OS` key के नाम में दी गई तिथि उसका time zone नहीं बताती।
- **`ShutdownTime` अंतिम साफ़ shutdown है।** Crash या बिजली जाने पर यह update नहीं होता।
  इसकी तुलना event logs (6006, 6008, 1074) और hives के अंतिम writes के समय से करें।
- **`LastLoggedOnUser` sign-in screen का अंतिम उपयोगकर्ता है,** हर logon नहीं। Remote Desktop,
  `runas`, services और scheduled tasks सभी इसे सेट नहीं करते। `LogonUiLastWrittenUTC` लगभग वह समय है जब
  इसे अंतिम बार सेट किया गया था।
- **Profiles:** `ProfileList` हर उस उपयोगकर्ता (और service) को सूचीबद्ध करता है जिसका system पर कभी
  profile रहा है, जिसमें हटाए गए accounts भी उनका profile हटाए जाने तक शामिल हैं। Windows 10 और बाद के संस्करण
  `LocalProfileLoadTime` और `LocalProfileUnloadTime` लिखते हैं: लगभग अंतिम sign-in और sign-out।
- **SIDs:** स्थानीय accounts `MachineSid-<RID>` होते हैं, domain accounts `DomainSid-<RID>`।
  `MachineSid` `SAM` से आता है (अन्यथा `SECURITY` के `PolAcDmS` से)। दोनों मेल खाने चाहिए;
  जब वे मेल नहीं खाते, तो `Problems` यह बताता है, जैसा अलग-अलग systems के hives के साथ होता। Workgroup में
  `DomainSid` खाली होता है।
- **एक directory, एक system।** `-d` के साथ, एक directory के hives एक ही system के माने जाते हैं।
  `RegBack` प्रतियाँ और उसी प्रकार का दूसरा hive (`SYSTEM.old`) अपने अलग records होते हैं।
  `-f` के साथ, दिए गए hives एक system के माने जाते हैं, और अन्य hives उनके बगल में खोजे जाते हैं।
- **Control sets:** केवल वह control set पढ़ा जाता है जिसे Windows ने अंतिम बार उपयोग किया था। `parse-registry read`
  अन्य दिखाता है।

## Juntei से parse करें { #parse-with-juntei }

```text
Usage: juntei parse-registry system-info <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SYSTEM, SOFTWARE, SAM and SECURITY hives
  -f, --file <FILE>      A hive of one system; repeat for each
      --no-logs          Do not replay logs

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry system-info -d collection/ -o system-info.csv
juntei parse-registry system-info -f SYSTEM -f SOFTWARE -t jsonl \
  | jq '{Host, TimeZoneDescription, ProductName, Build, LastShutdown}'
```

Summary प्रत्येक system के लिए देता है: उसका Windows संस्करण, वह कब install हुआ, शब्दों में उसका time zone,
key के अंतिम write के समय लागू offset, उसका अंतिम shutdown, उसका domain या
workgroup, sign in करने वाला अंतिम उपयोगकर्ता, उसका SID, उसके कितने profiles हैं, और वे संस्करण जिनसे इसे
upgrade किया गया था। `auto-parse` भी मिले systems को सूचीबद्ध करते समय प्रत्येक system का time zone print करता है।

## आउटपुट fields { #output-fields }

| Field | व्याख्या |
|---|---|
| `Host`, `SourcePath`, `SoftwarePath`, `SamPath`, `SecurityPath` | Host, और पढ़े गए hives (`SourcePath` `SYSTEM` hive है, अन्यथा पढ़ा गया पहला hive)। |
| `CompName`, `Hostname`, `Domain` | NetBIOS नाम (`SYSTEM`, अन्यथा `SECURITY` का `PolAcDmN`), DNS host name और DNS domain (`Tcpip\Parameters`)। |
| `PrimaryDomain`, `DNS-Domain` | वह domain जिससे computer joined है, या उसका workgroup (`PolPrDmN`), और domain का DNS नाम (`PolDnDDN`)। |
| `ProductName`, `Edition`, `DisplayVersion`, `Version`, `Build`, `Ubr`, `ServicePack`, `InstallationType` | `CurrentVersion`: लिखा गया product name, `EditionID`, `DisplayVersion` (अन्यथा `ReleaseId`), major और minor version (अन्यथा `CurrentVersion`), build, update build revision, `CSDVersion`, और `Client`, `Server` या `Server Core`। |
| `InstallDateUTC`, `InstallTimeUTC` | `InstallDate` (second तक) और `InstallTime` (100 ns तक, Windows 10 और बाद के): installation या अंतिम feature upgrade। |
| `RegisteredOwner`, `RegisteredOrganization`, `SystemRoot` | जैसा लिखा गया। |
| `TimeZone`, `TimeZoneDescription` | `TimeZoneKeyName` (अन्यथा `StandardName`), और शब्दों में zone, उसके offsets और daylight saving तिथियों के साथ। |
| `UTC-Offset`, `DaylightUTC-Offset` | UTC से local standard time का offset, और daylight time का (daylight saving time न होने पर खाली)। |
| `BiasMinutes`, `ActiveBiasMinutes`, `StandardBiasMinutes`, `DaylightBiasMinutes` | `Bias`, `ActiveTimeBias` (key लिखे जाते समय लागू), `StandardBias`, `DaylightBias`। |
| `StandardName`, `DaylightName` | जैसा लिखा गया, अक्सर एक resource reference (`@tzres.dll,-112`)। |
| `StandardStart`, `DaylightStart` | Standard और daylight time कब शुरू होते हैं, शब्दों में (`first Sunday of November, 02:00`)। |
| `DynamicDaylightTimeDisabled`, `RealTimeIsUniversal` | Dynamic daylight saving time बंद है; hardware clock UTC रखती है। |
| `TimeZoneLastWrittenUTC` | Time zone key का अंतिम write। |
| `LastShutdownUTC` | `ShutdownTime`: अंतिम साफ़ shutdown। |
| `CurrentControlSet`, `ProcessorArchitecture`, `AnsiCodePage`, `OemCodePage` | पढ़ा गया control set, `PROCESSOR_ARCHITECTURE`, `ACP` और `OEMCP`। |
| `LastLoggedOnUser`, `LastLoggedOnSamUser`, `LastLoggedOnUserSID`, `LastLoggedOnDisplayName`, `LogonUiLastWrittenUTC` | Sign-in screen का अंतिम उपयोगकर्ता, और उसकी key का अंतिम write। |
| `MachineSID`, `DomainSID`, `MachineAccountSID` | Computer का SID, उसके domain का SID (`PolPrDmS`) और domain में computer account का SID (`PolMachineAccountS`)। |
| `CompNameLastWrittenUTC`, `PrimaryDomainLastWrittenUTC`, `VersionLastWrittenUTC` | लगभग कब computer का अंतिम बार नामकरण हुआ, अंतिम बार किसी domain या workgroup से join हुआ, और अंतिम बार update हुआ (`CurrentVersion` की key)। |
| `PreviousVersions` | प्रत्येक `Source OS` key: संस्करण, उसकी install तिथि, और key के नाम में दी गई तिथि (JSON: प्रत्येक के लिए एक object)। |
| `Profiles` | प्रत्येक profile: SID, folder, key time, अंतिम load और unload (JSON: प्रत्येक के लिए एक object)। |
| `TransactionLogStatus`, `LogsApplied` | चारों hives पर registry recovery का संदर्भ। |
| `Problems` | गलत type या size की values (उनके bytes रखे जाते हैं), और ऐसे `SAM` और `SECURITY` जिनके computer SIDs भिन्न हों। |

## जाँच की रणनीति { #investigation-strategy }

1. पहले time zone पढ़ें: `UtcOffset`, daylight saving तिथियाँ, और क्या zone हाल ही में
   बदला गया था, नोट करें। Local-time logs को इसी से convert करें।
2. संस्करण, build और install तिथि नोट करें। Upgrades के लिए `PreviousVersions` जाँचें, और install तिथि की तुलना
   सबसे पुराने file system और event log times से करें।
3. `LastShutdownUTC` नोट करें: उसके बाद की गतिविधि, बिना किसी नए shutdown time के, crash, बिजली
   जाने या live collection का संकेत देती है।
4. अन्य स्थानों (BAM, `ProfileList`, event logs) के SIDs में स्थानीय और domain accounts में भेद करने के लिए
   `MachineSid` और `DomainSid` का उपयोग करें, और अंतिम sign-ins के लिए profiles के load और unload times का।
5. Renames और domain joins के लिए computer और domain नामों को event logs से, और `CompNameLastWrittenUTC`
   तथा `PrimaryDomainLastWrittenUTC` से correlate करें।

## अधिक जानकारी { #further-details }

- [Microsoft: TIME_ZONE_INFORMATION](https://learn.microsoft.com/en-us/windows/win32/api/timezoneapi/ns-timezoneapi-time_zone_information)
- [Microsoft: DYNAMIC_TIME_ZONE_INFORMATION](https://learn.microsoft.com/en-us/windows/win32/api/timezoneapi/ns-timezoneapi-dynamic_time_zone_information)
- [Microsoft: TIME_FIELDS](https://learn.microsoft.com/en-us/windows-hardware/drivers/ddi/wdm/ns-wdm-_time_fields)
- [Microsoft: Security identifiers](https://learn.microsoft.com/en-us/windows-server/identity/ad-ds/manage/understand-security-identifiers)
- [libyal winreg-kb: Time zones](https://github.com/libyal/winreg-kb/blob/main/docs/sources/system-keys/Time-zones.md)
- [libyal winreg-kb: Windows product information](https://github.com/libyal/winreg-kb/blob/main/docs/sources/system-keys/Windows-product-information.md)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
