# संग्रहीत डेटा का स्वचालित विश्लेषण { #automatic-collection-parsing }

`auto-parse` कई computers के Windows triage संग्रह को एक संरचित विश्लेषण directory में बदलने का
सबसे तेज़ तरीका है। इसे evidence root और एक नई output directory दें। यह संग्रहीत systems को खोजता है,
उनके computer names की पहचान करता है, और हर संगत Juntei parser को समानांतर रूप से चलाता है।

```bash
juntei auto-parse -d <EVIDENCE> -o <OUTPUT>
```

Default आउटपुट CSV है। उन CSV परिणामों को रखते हुए `jq` या SIEM import के लिए एक JSONL timeline
जोड़ें:

```bash
juntei auto-parse -d <EVIDENCE> -o <OUTPUT> -j <OUTPUT>/timeline.jsonl
```

!!! warning "Output और evidence trees को अलग रखें"
    Juntei ऐसी output directory को अस्वीकार करता है जो `<EVIDENCE>` के अंदर हो या जिसमें `<EVIDENCE>` हो। यह
    उत्पन्न डेटा को फिर से साक्ष्य के रूप में खोजे जाने से रोकता है और `--clobber` को source संग्रह की
    files को छूने से रोकता है।

## VHDX और E01 डिस्क इमेज { #vhdx-and-e01-disk-images }

`--vhdx` के साथ `auto-parse` evidence directory की हर VHDX disk image के अंदर के systems को parse करता है, और `--e01` के साथ हर E01 image के अंदर के systems को (split image को उसके पहले segment, `*.E01` या `*.Ex01`, से)। दोनों options एक साथ दिए जा सकते हैं। हर image को parsing के दौरान FUSE से read-only mount किया जाता है; files extract नहीं की जातीं। [डिस्क इमेज माउंट करना](mount.md) देखें।

```bash
juntei auto-parse -d /cases/vhdx --vhdx -o parsed
juntei auto-parse -d /cases/e01 --e01 -o parsed
```

यदि कोई NTFS वॉल्यूम नहीं खुलता, तो Juntei समस्या बताता है और पढ़ने योग्य वॉल्यूम को बनाए रखता है। आंशिक रूप से पढ़ी जा सकने वाली इमेज के लिए `auto-parse --vhdx` और `--e01` निकास कोड 1 लौटाते हैं; `mount` भी अनमाउंट करने के बाद 1 लौटाता है।

## आउटपुट time zone { #output-time-zone }

Parse किए गए timestamps default रूप से UTC में होते हैं, विश्लेषक के computer के local time zone से स्वतंत्र। UTC
Windows event-log रूप `2026-10-20T08:46:38.2001668Z` का उपयोग करता है: एक ISO 8601 date/time, तिथि और समय के बीच `T`,
और UTC के लिए `Z`। उपलब्ध fractional अंक सुरक्षित रखे जाते हैं।
`manifest.json` का `GeneratedUTC` भी इसी रूप का उपयोग करता है। Console के `Start time` और `End time`
विश्लेषक के computer के local time में होते हैं, जैसे `2026-11-04 09:15:02`, output time
zone चाहे जो भी हो।

`-Z, --timezone` से एक निश्चित display offset चुनें:

```bash
juntei auto-parse -d evidence/ -o parsed-jst/ --timezone "+09:00"
juntei auto-parse -d evidence/ -o parsed-offset/ -Z "-01:00"
juntei auto-parse -d evidence/ -o parsed-half-hour/ -Z "+02:30"
```

Offset एक signed, दो अंकों वाला `HH:MM` value है (घंटे `00–23`, minutes `00–59`)। `JST` जैसे नाम,
`+9` जैसे छोटे values, और अमान्य ranges उदाहरण syntax के साथ error उत्पन्न करते हैं।
UTC के लिए option को छोड़ दें या `--timezone "+00:00"` का उपयोग करें। निश्चित offset daylight
saving नियम लागू नहीं करता।

यह option प्रत्येक आर्टिफ़ैक्ट के CSV, JSON और JSONL परिणामों में parse किए गए timestamps और
अतिरिक्त `-j` JSONL export पर लागू होता है। उदाहरण के लिए, `2026-10-20T08:46:38.2001668Z`
`2026-10-20T17:46:38.2001668+09:00` बन जाता है; दोनों एक ही क्षण का वर्णन करते हैं।

Field names स्थिर रहते हैं, जिनमें `UTC` पर समाप्त होने वाले मौजूदा नाम भी शामिल हैं। Convert की गई values स्पष्ट रूप से
चुना गया offset रखती हैं। दोनों manifests में `OutputTimeZone` अनुरोधित display
offset दर्ज करता है, जबकि `GeneratedUTC` UTC run metadata ही रहता है। प्रत्येक parser के logs parsers के मूल UTC summaries बनाए रखते हैं।

Raw FILETIME/epoch numbers, registry-value dictionaries, source time-zone configuration, text
metadata, केवल-तिथि values और बिना ज्ञात offset वाले local clock values सुरक्षित रखे जाते हैं। Juntei
किसी अज्ञात local time में `Z` नहीं जोड़ता। FAT से प्राप्त shell-item clocks (ShellBags/OpenSaveMRU में
`Item*UTC` और LNK/Jump List `TargetItems`) में कोई दर्ज offset नहीं होता; उनके पुराने
UTC labels या render किया गया `Z` suffix UTC सिद्ध नहीं करते, इसलिए offset option उन्हें सुरक्षित रखता है।
Free-text summaries वाले CSV columns अपना
दर्ज text बनाए रखते हैं; JSON/JSONL nested parse किए गए timestamps को संरचित fields के रूप में सुरक्षित रखता है।

UTC मौजूदा parser आउटपुट का सीधे उपयोग करता है। स्पष्ट nonzero offset प्रत्येक parser worker के भीतर एक streaming
formatting pass जोड़ता है; यह साक्ष्य बदले बिना display values बदलता है।

यदि कोई offset किसी timestamp को वर्ष `0000–9999` से बाहर ले जाए, तो वह value अपना मूल
स्पष्ट offset रखती है। नौ से अधिक fractional अंकों वाली values भी round किए जाने के बजाय
अपरिवर्तित रहती हैं। Juntei parser की `*-logs.txt` file में एक warning दर्ज करता है और अन्य
timestamps को convert करता है। Malformed CSV/JSON या output error फिर भी task को विफल करता है और प्रभावित
मूल परिणाम file को अपरिवर्तित छोड़ता है।

## यह क्या खोजता है { #what-it-discovers }

Juntei recursively `Windows` नाम की ऐसी directories खोजता है जिनमें `System32`, `Prefetch`
या `AppCompat` directory हो, case-insensitively। अंतिम दो को पहचानना ऐसे लक्षित
संग्रहों का भी समर्थन करता है जिनमें केवल Prefetch या Amcache हो। यह
`AppData\Roaming\Microsoft\Windows` जैसी user-profile directories को बाहर रखता है। `C:\Windows.old\Windows`
जैसे nested पुराने installations duplicate computers बनने के बजाय बाहरी संग्रहीत volume को सौंपे जाते हैं।
Default अधिकतम गहराई 10 है। यह सामान्य Velociraptor paths को कवर करती है, जैसे:

```text
<EVIDENCE>/<collection>/uploads/auto/C%3A/Windows
```

सीमा केवल तभी बदलें जब संग्रह अधिक गहराई में nested हो:

```bash
juntei auto-parse -d evidence -o parsed --max-depth 15
```

प्रत्येक खोजे गए system के लिए, computer name इस क्रम में चुना जाता है:

1. निकटतम ancestor `client_info.json` में `hostname` (जिसकी value manifest में `FQDN` के रूप में रखी जाती है)।
2. संग्रहीत `SYSTEM` registry hive में `ComputerName`।
3. निकटतम उपयोगी collection-directory नाम, जिसमें drive और Velociraptor `uploads/auto`
   scaffolding छोड़ दी जाती है और `_F.…` जैसा suffix हटा दिया जाता है।

नामों को directory names के रूप में उपयोग के लिए सुरक्षित बनाया जाता है। Duplicate नामों को उनकी
output directory और `Comp` value दोनों में एक संख्यात्मक suffix मिलता है, ताकि अलग-अलग systems JSON में पहचाने जा सकें।
Manifest hostname को ठीक वैसा ही, जैसा खोजा गया था, एक अलग `Hostname` field में रखता है।

Systems सूचीबद्ध करने के बाद, `auto-parse` `Parser tasks` के ठीक ऊपर एक `Timezone:` line print करता है।
यह चुना गया output timezone दिखाता है: default रूप से `UTC`, या `--timezone "+09:00"` का उपयोग होने पर
`UTC+09:00` जैसी value। Label हरा होता है और value अन्य summary values की तरह
terminal के default text color का उपयोग करती है। `-K, --no-color` line को सादा रखता है।
संग्रहीत systems का time-zone configuration
[सिस्टम जानकारी](../artifacts/system-info.md) परिणामों में उपलब्ध है।

## यह कौन-से parsers चलाता है { #parsers-it-runs }

एक parser का विफल होना अन्य को नहीं रोकता। `auto-parse` हर खोजे गए
computer के लिए ये jobs चलाता है। Jobs manual parsing जैसे ही source-grouped commands का उपयोग करते हैं, जैसे
`parse-ntfs mft`, `parse-registry amcache`, `parse-ese srum` और `parse-file prefetch`:

- Amcache
- `$MFT`
- `$LogFile` NTFS redo/undo records
- Prefetch records और एक अलग Prefetch execution timeline
- USN Journal
- `$I30` directory indexes
- `$Boot` boot sectors और `$Secure:$SDS` security descriptors
- SRUM
- User Access Logging (UAL)
- Web browser का history, downloads, cookies और form history (Chromium- और Firefox-based browsers)
- Internet Explorer और legacy Edge का WebCache: history, downloads, cookies और cache entries
- LNK shortcuts
- Jump Lists
- ShimCache
- Persistence Hunter (allow/block profiles के साथ 131 cross-source persistence checks)
- Defender quarantine metadata
- Open और Save As संवाद इतिहास
- सभी registry keys और values, recover योग्य deleted डेटा सहित
- services, RecentDocs, TaskCache, SAM accounts, registry autostart locations, UserAssist, BAM,
  installed programs, MuiCache, ShellBags और USB devices
- system जानकारी: नाम, Windows संस्करण, time zone, अंतिम shutdown, profiles और SIDs
- audit policy: system को कौन-से security events log करने के लिए सेट किया गया था
- boot configuration (BCD और BCD-Template)
- network profiles, interfaces, shares, port proxies और routes
- LSA secrets के नाम और times, और उपयोग में cached domain logons (कुछ भी decrypt नहीं किया जाता)

Persistence Hunter हर severity (`--min-severity info`) को बनाए रखता है, जिसमें आंशिक allow matches भी शामिल हैं।
`LOW` जाँच के लिए एक वर्गीकरण है; signature metadata का अनुपस्थित होना दुर्भावनापूर्ण
commands के लिए भी इसे उत्पन्न कर सकता है। स्वचालित inventory manual command के
default medium threshold का उपयोग करके इन entries को नहीं छोड़ती।

Hunter प्रत्येक स्वचालित parser process के भीतर एक worker का उपयोग करता है। यह `auto-parse -T <N>` को
हर child में किसी अन्य CPU-आकार के pool से अपने process budget को गुणा करने से रोकता है। स्वतंत्र
`persistence-hunter -T <N>` hunter के अपने worker threads को नियंत्रित करता है।

CSV `$MFT` आउटपुट में सभी timestamps और short names शामिल होते हैं। Registry आउटपुट में raw डेटा और
`REG_QWORD` values की FILETIME व्याख्या शामिल होती है। JSON और JSONL SRUM आउटपुट में value-state
metadata शामिल होता है। Defender files और resident `$MFT` streams स्वचालित रूप से **extract नहीं** किए जाते।

`--mft-slack` के साथ, `$MFT` आउटपुट में FILE records के slack में बचे file names भी शामिल होते हैं, उसी रूप में जैसे [`parse-ntfs mft -x`](../artifacts/mft.md) उन्हें लिखता है: CSV में `FromSlack` से चिह्नित rows, और JSON तथा JSONL में `SlackFileNames`। `--code-page` LNK files, Jump Lists, ShellBags और OpenSaveMRU पर लागू होता है।

जो आर्टिफ़ैक्ट संग्रहित नहीं किया गया, वह एक सामान्य `skipped` task है, विफलता नहीं।

## आउटपुट layout { #output-layout }

परिणाम computer और जाँच श्रेणी के अनुसार व्यवस्थित होते हैं। श्रेणी directories
PascalCase का उपयोग करती हैं, जैसे `Accounts` और `FileAndFolderOpening`। Amcache और Jump Lists tables
filename prefixes के साथ सीधे अपनी श्रेणियों में रहती हैं। Computer directory names अपनी खोजी गई spelling बनाए रखते हैं। मौजूदा output
directories migrate नहीं की जातीं; पुराने naming से बदलते समय एक नई output directory चुनें। प्रत्येक parser अपने परिणामों के बगल में एक
`<artifact>-logs.txt` file लिखता है, तब भी जब आर्टिफ़ैक्ट संग्रहित नहीं किया गया था।
CSV अलग-अलग tables रखता है क्योंकि प्रत्येक आर्टिफ़ैक्ट के columns अलग होते हैं:

```text
<OUTPUT>/
├── manifest.json
├── manifest.csv
├── HOST1/
│   ├── Accounts/
│   │   ├── Registry-LsaSecrets.csv
│   │   ├── Registry-LsaSecrets-logs.txt
│   │   ├── Registry-SAM.csv
│   │   └── Registry-SAM-logs.txt
│   ├── Applications/
│   │   ├── Registry-AppHistory.csv
│   │   ├── Registry-AppHistory-logs.txt
│   │   ├── Registry-InstalledPrograms.csv
│   │   └── Registry-InstalledPrograms-logs.txt
│   ├── BrowserActivity/
│   │   ├── Browser-Cookies.csv
│   │   ├── Browser-Downloads.csv
│   │   ├── Browser-FormHistory.csv
│   │   ├── Browser-History.csv
│   │   ├── Browser-logs.txt
│   │   ├── WebCache-Containers.csv
│   │   ├── WebCache-Cookies.csv
│   │   ├── WebCache-Downloads.csv
│   │   ├── WebCache-Entries.csv
│   │   ├── WebCache-History.csv
│   │   └── WebCache-logs.txt
│   ├── ExternalDevices/
│   │   ├── Registry-USB.csv
│   │   └── Registry-USB-logs.txt
│   ├── FileAndFolderOpening/
│   │   ├── JumpLists-AutomaticDestinations.csv
│   │   ├── JumpLists-CustomDestinations.csv
│   │   ├── JumpLists-logs.txt
│   │   ├── LNK.csv
│   │   ├── LNK-logs.txt
│   │   ├── Registry-Office.csv
│   │   ├── Registry-Office-logs.txt
│   │   ├── Registry-MRU.csv
│   │   ├── Registry-MRU-logs.txt
│   │   ├── Registry-OpenSaveMRU.csv
│   │   ├── Registry-OpenSaveMRU-logs.txt
│   │   ├── Registry-RecentDocs.csv
│   │   ├── Registry-RecentDocs-logs.txt
│   │   ├── Registry-ShellBags.csv
│   │   └── Registry-ShellBags-logs.txt
│   ├── FileExistence/
│   │   ├── Registry-Amcache-Devices.csv
│   │   ├── Registry-Amcache-Files.csv
│   │   ├── Registry-Amcache-Findings.csv
│   │   ├── Registry-Amcache-Summary.csv
│   │   ├── Registry-Amcache-logs.txt
│   │   ├── Registry-ShimCache.csv
│   │   └── Registry-ShimCache-logs.txt
│   ├── FileSystem/
│   │   ├── Boot.csv
│   │   ├── Boot-logs.txt
│   │   ├── I30.csv
│   │   ├── I30-logs.txt
│   │   ├── LogFile.csv
│   │   ├── LogFile-logs.txt
│   │   ├── MFT.csv
│   │   ├── MFT-logs.txt
│   │   ├── SecureSDS.csv
│   │   ├── SecureSDS-logs.txt
│   │   ├── UsnJournal.csv
│   │   └── UsnJournal-logs.txt
│   ├── NetworkActivity/
│   │   ├── Registry-Networks.csv
│   │   ├── Registry-Networks-logs.txt
│   │   ├── SRUM-NetworkConnections.csv
│   │   ├── SRUM-NetworkUsages.csv
│   │   └── SRUM-PushNotifications.csv
│   ├── LateralMovement/
│   │   ├── Registry-RemoteAccess.csv
│   │   ├── Registry-RemoteAccess-logs.txt
│   │   ├── UAL-Clients.csv
│   │   ├── UAL-RoleAccess.csv
│   │   ├── UAL-SystemIdentity.csv
│   │   └── UAL-logs.txt
│   ├── Persistence/
│   │   ├── Registry-BCD.csv
│   │   ├── Registry-BCD-logs.txt
│   │   ├── Registry-Persistence.csv
│   │   ├── Registry-Persistence-logs.txt
│   │   ├── Registry-Services.csv
│   │   ├── Registry-Services-logs.txt
│   │   ├── Registry-TaskCache.csv
│   │   └── Registry-TaskCache-logs.txt
│   ├── ProcessExecution/
│   │   ├── Registry-AppCompat.csv
│   │   ├── Registry-AppCompat-logs.txt
│   │   ├── Registry-FeatureUsage.csv
│   │   ├── Registry-FeatureUsage-logs.txt
│   │   ├── Registry-BAM.csv
│   │   ├── Registry-BAM-logs.txt
│   │   ├── Registry-MuiCache.csv
│   │   ├── Registry-MuiCache-logs.txt
│   │   ├── Prefetch.csv
│   │   ├── Prefetch-logs.txt
│   │   ├── PrefetchTimeline.csv
│   │   ├── PrefetchTimeline-logs.txt
│   │   ├── SRUM-AppResourceUseInfo.csv
│   │   ├── SRUM-AppTimelineProvider.csv
│   │   ├── Registry-UserAssist.csv
│   │   └── Registry-UserAssist-logs.txt
│   ├── Registry/
│   │   ├── Registry-Dump.csv
│   │   └── Registry-Dump-logs.txt
│   ├── SecurityProducts/
│   │   ├── Registry-SecuritySettings.csv
│   │   ├── Registry-SecuritySettings-logs.txt
│   │   ├── DefenderQuarantine.csv
│   │   ├── DefenderQuarantine-logs.txt
│   │   └── DefenderQuarantined/  # only with -x
│   └── SystemInformation/
│       ├── Registry-AuditPolicy.csv
│       ├── Registry-AuditPolicy-logs.txt
│       ├── Registry-SystemInfo.csv
│       ├── Registry-SystemInfo-logs.txt
│       ├── SRUM-IdentityMap.csv
│       ├── SRUM-VfuProvider.csv
│       └── SRUM-logs.txt
└── HOST2/
    └── …
```

| श्रेणी | आर्टिफ़ैक्ट |
|---|---|
| `Accounts` | SAM स्थानीय खाते, LSA secrets के नाम और times |
| `Applications` | ऐप्लिकेशन इतिहास (third-party tools के निशान) |
| `Applications` | इंस्टॉल किए गए प्रोग्राम |
| `BrowserActivity` | Web browser का history, downloads, cookies और form history, recover की गई deleted entries सहित; Internet Explorer और legacy Edge का WebCache |
| `ExternalDevices` | USB devices और volumes |
| `FileAndFolderOpening` | Jump Lists, LNK shortcuts, Microsoft Office, MRU lists, Open और Save As संवाद, RecentDocs, ShellBags |
| `FileExistence` | Amcache, ShimCache |
| `FileSystem` | `$Boot`, `$I30`, `$LogFile`, `$MFT`, `$Secure:$SDS`, USN Journal |
| `NetworkActivity` | Network profiles, interfaces, shares, port proxies और routes; SRUM connections, traffic और push notifications |
| `LateralMovement` | उपयोगकर्ताओं के outbound Remote Desktop, PuTTY और WinSCP connections, mapped drives और shares; User Access Logging (किसी server के roles के clients) |
| `Persistence` | Boot configuration (BCD), registry autostarts, services, TaskCache |
| `ProcessExecution` | Application compatibility, BAM और DAM, FeatureUsage और pins, MuiCache, Prefetch, Prefetch timeline, SRUM application/CPU गतिविधि, UserAssist |
| `Registry` | सामान्य hive keys और values, recover किए गए records सहित |
| `SecurityProducts` | Defender quarantine metadata और वैकल्पिक रूप से extract किया गया content; security settings (Defender, UAC, credential protection, RDP, firewall, logging) |
| `SystemInformation` | नाम, Windows संस्करण, time zone, अंतिम shutdown, profiles, SIDs और audit policy; SRUM energy/capacity, identity maps और अज्ञात providers |

श्रेणियाँ जाँच को व्यवस्थित करने में मदद करती हैं; वे किसी विशेष गतिविधि का प्रमाण नहीं हैं। उदाहरण के लिए,
केवल Amcache और ShimCache entries execution स्थापित नहीं करतीं। SRUM resource उपयोग दर्ज करता है;
किसी program के चलने के बारे में निष्कर्ष निकालने से पहले आर्टिफ़ैक्ट मार्गदर्शिका देखें।

CSV, JSON और JSONL समान श्रेणियों का उपयोग करते हैं। प्रत्येक parser job के लिए एक `<artifact>-logs.txt` file में उसका
सटीक command और messages होते हैं; उदाहरण के लिए, `HOST1/FileSystem/MFT-logs.txt` और
`HOST1/ProcessExecution/Prefetch-logs.txt`। JSON और JSONL समान log filenames का उपयोग करते हैं।
Amcache और Jump Lists प्रत्येक के लिए उनकी prefixed tables के बगल में एक साझा log होता है। SRUM provider tables सीधे
`NetworkActivity`, `ProcessExecution` और `SystemInformation` में रखी जाती हैं, एक साझा
`SystemInformation/SRUM-logs.txt` file के साथ। कोई `logs/` subdirectories नहीं बनाई जातीं।

Default CSV filenames `Registry-SAM.csv`, `LNK.csv`, `Registry-RecentDocs.csv`, `Registry-ShimCache.csv`, `LogFile.csv`,
`I30.csv`, `MFT.csv`, `UsnJournal.csv`, `Boot.csv`, `SecureSDS.csv`, `Registry-Networks.csv`, `Registry-Persistence.csv`, `Registry-Services.csv`, `Registry-TaskCache.csv`, `Registry-BCD.csv`,
`Registry-SecuritySettings.csv`,
`PrefetchTimeline.csv`, `Prefetch.csv`, `Registry-UserAssist.csv`, `Registry-AppCompat.csv`, `Registry-Office.csv`, `Registry-FeatureUsage.csv`, `Registry-AppHistory.csv`, `Registry-MRU.csv`, `Registry-SystemInfo.csv`, `Registry-RemoteAccess.csv`, `Registry-OpenSaveMRU.csv`, `Registry-InstalledPrograms.csv`, `Registry-BAM.csv`, `Registry-MuiCache.csv`, `Registry-ShellBags.csv`, `Registry-USB.csv`, `Registry-SecuritySettings.csv`, `Registry-AuditPolicy.csv`, `Registry-Dump.csv` और
`DefenderQuarantine.csv` हैं। Registry से प्राप्त tables `Registry-` prefix का उपयोग करती हैं, जिनमें
`Registry-ShimCache.csv`, `Registry-UserAssist.csv`, `Registry-BAM.csv` और `Registry-SystemInfo.csv` शामिल हैं।
सामान्य hive keys और values `Registry-Dump.csv` में होते हैं। Amcache tables
`FileExistence/Registry-Amcache-Devices.csv`, `Registry-Amcache-Files.csv`,
`Registry-Amcache-Findings.csv` और `Registry-Amcache-Summary.csv` हैं। Jump Lists
`FileAndFolderOpening/JumpLists-AutomaticDestinations.csv` और
`JumpLists-CustomDestinations.csv` का उपयोग करती हैं। Amcache या Jump Lists की कोई subdirectories नहीं होतीं।
SRUM tables अपनी श्रेणी के ठीक अंतर्गत `SRUM-` filenames का उपयोग करती हैं, उदाहरण के लिए
`NetworkActivity/SRUM-NetworkConnections.csv`।
User Access Logging tables `LateralMovement` में होती हैं, हर table के लिए एक file, जैसे
`UAL-Clients.csv` और `UAL-SystemIdentity.csv`, साथ में `UAL-logs.txt`;
[उपयोगकर्ता एक्सेस लॉगिंग (UAL)](../artifacts/ual.md#automatic-parsing) देखें।
Web browser tables `BrowserActivity` में होती हैं: `Browser-History.csv`, `Browser-Downloads.csv`,
`Browser-Cookies.csv` और `Browser-FormHistory.csv`, साथ में `Browser-logs.txt`;
[वेब ब्राउज़र](../artifacts/browser.md#automatic-parsing) देखें।
Internet Explorer और legacy Edge की WebCache tables भी वहीं होती हैं: `WebCache-History.csv`, `WebCache-Downloads.csv`,
`WebCache-Cookies.csv`, `WebCache-Entries.csv` और `WebCache-Containers.csv`, साथ में `WebCache-logs.txt`;
[Internet Explorer और लेगेसी Edge](../artifacts/webcache.md#automatic-parsing) देखें।
[पूरी SRUM placement table](../artifacts/srum.md#categorized-auto-parse-output) देखें।

Logs capitalized आर्टिफ़ैक्ट नाम से मेल खाते हैं, जैसे `Registry-SAM-logs.txt`, `PrefetchTimeline-logs.txt`
और `Registry-UserAssist-logs.txt`। JSON/JSONL files समान stems का उपयोग करती हैं, उदाहरण के लिए `MFT.jsonl`; queries के लिए
record का `Artifact` identifier `mft` है। Multi-table outputs की tables के बगल में
`Registry-Amcache-logs.txt` या `JumpLists-logs.txt` होता है। साझा SRUM log
`SystemInformation` में होता है।


`manifest.json` मूल खोजा गया `Hostname`, अद्वितीय output/record `Comp`
पहचान, hostname का स्रोत, evidence paths, आर्टिफ़ैक्ट `Category`, command lines, अवधि, exit
code, result path, log path, अनुरोधित JSONL timeline और कोई भी export error, क्या Defender extraction का अनुरोध किया गया था,
और क्या प्रत्येक task
सफल हुआ, skip हुआ या विफल हुआ, यह दर्ज करता है।

Auto-parse कोई absolute path नहीं लिखता, इसलिए परिणामों को यह दिखाए बिना साझा किया जा सकता है कि विश्लेषण system पर साक्ष्य,
परिणाम या Juntei कहाँ हैं:

- Evidence paths evidence directory के सापेक्ष होते हैं। Terminal प्रत्येक computer को साक्ष्य में उसकी
  directory के साथ सूचीबद्ध करता है (`HOST1: ./HOST1_F`), और parsers evidence directory में चलते हैं और
  उन्हें ऐसे ही paths दिए जाते हैं। परिणामों के source path columns (`SourceFile`, `SourcePath`, …), logs
  और manifests के `EvidenceRoot` तथा `WindowsDirectories` दिखाते हैं कि प्रत्येक file साक्ष्य में कहाँ है
  (`./HOST1_F/C/Windows/Prefetch/CMD.EXE-6D6290C5.pf`)।
- Manifests और logs के result paths (`Output`, `OutputFiles`, `Log`, `JSON-Timeline`, `Command` में
  output paths) और Defender का `ExtractedFile` output directory के सापेक्ष होते हैं
  (`./HOST1/ProcessExecution/Prefetch.csv`, इसके बगल की timeline के लिए `../siem.jsonl`)। `Command`
  program के नाम (`juntei`) से शुरू होता है; इसके evidence paths evidence
  directory के सापेक्ष और इसके output paths output directory के सापेक्ष होते हैं।
- `EvidenceDirectory` और `OutputDirectory` दोनों directories के नाम हैं। `ManifestVersion` 2
  उन manifests को चिह्नित करता है जो output directory से result paths दर्ज करते हैं; पुराने manifests में कोई
  `ManifestVersion` नहीं होता और वे इन्हें वैसे दर्ज करते हैं जैसे `-o` को दिए गए थे। `-C` rerun दोनों को पढ़ता है।
- Terminal output या timeline path को relative होने पर वैसे ही दिखाता है जैसा दिया गया था, और absolute होने पर
  current directory से (`../results/`)।

Parallel processes की संख्या के बाद, terminal मिले Windows systems की files का कुल आकार दिखाता है
(`Total filesize: 26.1 MiB`)।

Progress message सक्रिय computer और आर्टिफ़ैक्ट दिखाता है, उसके बाद अनुमानित input
आकार, उदाहरण के लिए `HOST1 / prefetch (~2.0 MiB)`। Files का समूह उनके संयुक्त आकार का उपयोग करता है; प्रत्येक
MFT, LogFile या USN Journal task अपना अलग input आकार दिखाता है। अनुमानों में सहायक transaction
logs और enrichment files शामिल नहीं होते और ये parse किए गए आउटपुट के आकार का पूर्वानुमान नहीं लगाते।

`manifest.csv` वही task index spreadsheet-अनुकूल table के रूप में देता है, प्रत्येक
parser task के लिए एक row के साथ। इसमें `Comp`, मूल `Hostname`, `FQDN`, `HostnameSource`, `EvidenceRoot`,
`Artifact`, `Category`, `OutputType`, `Purpose`, `Status`, `DurationMs`, `ExitCode`, `Output`, `OutputFiles`, `Log`,
`Error` और `Command` शामिल हैं। `Purpose` वर्गीकृत परिणामों के लिए `artifact` या CSV के साथ उपयोग किए गए
अतिरिक्त native JSONL pass के लिए `json_timeline` होता है। `OutputType` वास्तविक parser output format है।
`Command` executable और उसके arguments का JSON array है, जो spaces और quoting को सुरक्षित रखता है।
SRUM के लिए, `Output` computer directory है, `OutputFiles` उसके सटीक
provider output paths का JSON array है, और `Category` साझा log स्थान (`SystemInformation`) का नाम देता है।
Files को अलग-अलग वर्गीकृत किया जाता है; `OutputFiles` JSON manifest में भी दिखाई देता है।

समीक्षा की आवश्यकता वाले tasks खोजने के लिए `Status` को `skipped` या `failed` पर filter करें; skip हुए task का `Error`
खाली हो सकता है, और उसकी व्याख्या उसकी `Log` file में होती है।

Terminal summary में output directory के नीचे वर्तमान में मौजूद regular files की कुल संख्या और आकार शामिल होता है,
जिसमें दोनों manifests, parser logs और extract की गई files शामिल हैं। Symbolic links को
follow नहीं किया जाता। आकार `Total filesize` जैसे ही formatter का उपयोग करके स्वचालित रूप से scale होते हैं,
`KiB`, `MiB` और `GiB` जैसी units के साथ:

```text
Results saved: output/ ( 125 files / 40.5 MiB )
```

जब कई source databases संग्रहित किए गए हों, तो Amcache और SRUM tables `-Source1`, `-Source2` आदि का उपयोग करती हैं।
साझा parser log मूल-से-अंतिम paths दर्ज करता है, और JSON/JSONL
records source provenance बनाए रखते हैं। CSV Amcache/Jump Lists tasks अपने सटीक table paths को
manifest `OutputFiles` में सूचीबद्ध करते हैं; उनका `Output` जाँच श्रेणी की directory है। JSON और JSONL
Amcache और Jump Lists exports पहले से ही `Registry-Amcache.json[l]` और
`JumpLists.json[l]` नाम की एकल files होते हैं। संयुक्त `-j` timeline समान record contents और `Artifact` values बनाए रखती है।
`--clobber` पिछले manifest का उपयोग करके दर्ज outputs को साफ़ करता है, पुराने नामों सहित,
जबकि असंबंधित श्रेणी files को सुरक्षित रखता है। एक सामान्य single-volume संग्रह `$MFT` और USN Journal records को सीधे
`FileSystem/MFT.csv` और `FileSystem/UsnJournal.csv` में लिखता है। अतिरिक्त संग्रहीत volumes क्रमांकित
sibling files का उपयोग करते हैं, जैसे `FileSystem/MFT-2.csv` और `FileSystem/MFT-2-logs.txt`।

## JSON और SIEM ingestion { #json-and-siem-ingestion }

प्रत्येक JSON या JSONL record को दो top-level fields मिलते हैं:

- `Comp`: source system।
- `Artifact`: वह Juntei parser जिसने record उत्पन्न किया, जैसे `mft` या `prefetch-timeline`।

`-j, --json-timeline <FILE>` अलग-अलग परिणामों के format की परवाह किए बिना एक अतिरिक्त JSONL file बनाता है।
सभी वर्गीकृत outputs और उनके logs बनाए रखे जाते हैं:

```bash
# Default CSV tables plus a SIEM timeline
juntei auto-parse -d evidence -o parsed -j parsed/timeline.jsonl

# Pretty JSON results plus a SIEM timeline
juntei auto-parse -d evidence -o parsed-json -t json -j parsed-json/timeline.jsonl

# Individual JSONL results plus a consolidated JSONL timeline
juntei auto-parse -d evidence -o parsed-jsonl -t jsonl -j parsed-jsonl/timeline.jsonl
```

Filename का उपयोग वैसे ही किया जाता है जैसा दिया गया है; relative paths current working directory के सापेक्ष होते हैं।
यह output root में, किसी अन्य non-reserved output subdirectory में, या output
directory के बाहर हो सकता है। Evidence, manifests, प्रत्येक computer के results/logs और अस्थायी parser directories को
destination के रूप में उपयोग नहीं किया जा सकता। मौजूदा file के लिए `-C` आवश्यक है; विफल export उसे अक्षुण्ण छोड़ता है।

**CSV mode** में, Juntei एक अतिरिक्त native JSONL pass करता है ताकि SIEM file
संख्यात्मक और boolean types, nested डेटा, raw fields और metadata को सुरक्षित रखे, जिन्हें CSV tables छोड़ सकती हैं।
अतिरिक्त jobs समान `-T` concurrency सीमा साझा करते हैं, quarantine contents को फिर से extract नहीं करते, और
उनके अलग `*-json-timeline-logs.txt` sidecar logs होते हैं। दोनों manifests उनके उद्देश्य और
output format में भेद करते हैं। सफल run के बाद अस्थायी JSONL files हटा दी जाती हैं; यदि कोई
parser या export विफल हो तो उन्हें बनाए रखा जाता है। अतिरिक्त processing समय और अस्थायी disk space की योजना बनाएँ।

**JSON mode** में, Juntei साक्ष्य को फिर से parse किए बिना मौजूदा परिणामों को compact JSONL में stream करता है।
Amcache reports को native JSONL जैसे ही अलग-अलग summary, finding, file और device
records में विस्तारित किया जाता है। **JSONL mode** में, मौजूदा annotated records को सीधे संयुक्त किया जाता है।
सभी modes समान SIEM record संरचना का उपयोग करते हैं। Dataset native timestamps और आर्टिफ़ैक्ट
fields बनाए रखता है; यह कोई सामान्य event timestamp नहीं गढ़ता और न ही विषम आर्टिफ़ैक्ट को वैश्विक रूप से sort करता है।

```bash
jq -c 'select(.Comp == "HOST1" and .Artifact == "prefetch")' parsed/timeline.jsonl
jq -r 'select(.Artifact == "mft") | .FullPath? // empty' parsed/timeline.jsonl
```

`manifest.json` अनुरोधित `JSON-Timeline` path और कोई भी `JSON-TimelineError` दर्ज करता है। इसके task
records, और `manifest.csv`, अभी भी बनाए रखे गए आर्टिफ़ैक्ट outputs की ओर संकेत करते हैं। सफल CSV केवल-timeline
jobs हटाई गई अस्थायी files के बजाय अंतिम JSONL file का संदर्भ देते हैं। यदि कोई parser विफल होता है, तो
run असफल रूप से समाप्त होता है और timeline में केवल सफल परिणाम होते हैं; इसे पूर्ण मानने से पहले manifests
और श्रेणी logs की जाँच करें। File-count/size summary output directory के अंदर की files को कवर करता है;
बाहरी timeline की रिपोर्ट अलग से `JSONL timeline: <FILE>` के रूप में की जाती है।

## प्रदर्शन { #performance }

Parser jobs अलग processes होते हैं, इसलिए CPU-भारी आर्टिफ़ैक्ट एक साथ चल सकते हैं और किसी
एक parser में क्षति पूरे run को समाप्त नहीं कर सकती। Default रूप से, Juntei logical CPU count का उपयोग करता है। जब साक्ष्य
धीमी disk पर हो या system में कम memory हो, तो concurrency सीमित करें:

```bash
juntei auto-parse -d evidence -o parsed -T 4
```

## उपयोग { #usage }

```text
Version: 1.1.0 (Dev Build)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei auto-parse -d <EVIDENCE> -o <OUTPUT> [OPTIONS]

Input:
  -d, --directory <EVIDENCE>  Evidence directory containing one or more collected Windows systems
      --max-depth <DEPTH>     Maximum directory depth at which to discover a Windows directory [default: 10]
      --vhdx                  Parse the files inside every VHDX disk image in the evidence directory, each mounted read-only with FUSE while it is parsed
      --e01                   Parse the files inside every E01 disk image in the evidence directory (split images from their first segment), each mounted read-only with FUSE while it is parsed

Output:
  -o, --output <DIR>                  New directory in which to create per-computer results
  -t, --output-type <TYPE>            Output type: csv (default), json, or jsonl [default: csv]
  -j, --json-timeline <FILE>          Create a single JSONL timeline to import into a SIEM
  -Z, --timezone <OFFSET>             Display parsed timestamps using a fixed UTC offset (UTC by default) [default: +00:00]
  -C, --clobber                       Overwrite outputs in an existing directory (never deletes unrelated files)
  -x, --extract-defender-quarantined  Extract Defender-quarantined files

Performance:
  -T, --threads <N>  Parser processes to run at once (default: logical CPU count)

Parsing:
      --code-page <NUMBER>  Windows code page for ANSI strings in LNK files, Jump Lists, ShellBags and OpenSaveMRU [default: 1252]
      --mft-slack           Recover the file names left in the slack of $MFT records

General Options:
  -h, --help  Show the help menu

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

जहाँ व्यावहारिक हो, नई output directory का उपयोग करें। `-C` इस run के लिए केवल सटीक उत्पन्न
targets को हटाता या overwrite करता है, ताकि अब अनुपस्थित कोई आर्टिफ़ैक्ट पुराना परिणाम न छोड़ सके। यह
असंबंधित files या पूरे output tree को delete नहीं करता। यह parsing से पहले प्रत्येक computer की
`SecurityProducts/DefenderQuarantined/` directory, और पिछले flat layout की आरक्षित `defender-quarantined/`
directory को भी हटाता है, चाहे नया run `-x` का उपयोग करे या नहीं।

पुराने flat layout से बदलते समय नई output directory का उपयोग करें। पुरानी CSV/JSON files
स्वचालित रूप से स्थानांतरित या delete नहीं की जातीं।

!!! danger "Extract किए गए Defender content को malware मानें"
    `-x, --extract-defender-quarantined` quarantined content को प्रत्येक computer की
    `SecurityProducts/DefenderQuarantined/` directory के अंतर्गत disk पर लिखता है। Recover की गई binary streams `.quarantined` पर समाप्त होती हैं, और Juntei
    execute permission हटा देता है। प्रत्येक binary security descriptor को ownership, permissions और SDDL के साथ एक पठनीय `.security_descriptor.txt`
    report भी मिलती है। Recover किए गए payloads को कभी execute न करें। एक पृथक
    विश्लेषण system का उपयोग करें और अपेक्षा रखें कि antivirus या EDR software recover किए गए content पर alert करेगा या उसे हटा देगा।

Report fields और उनकी forensic सीमाओं के लिए [recover किए गए security descriptor को पढ़ना](../artifacts/defender-quarantine.md#reading-the-recovered-security-descriptor)
देखें।

Auto-parse उपयोगकर्ताओं के remote connections (RDP, PuTTY, WinSCP और mapped drives) भी पढ़ता है।
