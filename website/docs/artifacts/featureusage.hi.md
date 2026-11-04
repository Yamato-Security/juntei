# FeatureUsage, RecentApps और Taskbar पिन { #featureusage-recentapps-and-taskbar-pins }

## ये क्या हैं { #what-they-are }

Explorer उपयोगकर्ता के `NTUSER.DAT` में इस बात के कई records रखता है कि उपयोगकर्ता applications का उपयोग कैसे करता है:

- **FeatureUsage** (Windows 10 और बाद के): प्रति application, उपयोग के प्रकार के अनुसार counters।
  - `AppSwitched`: उपयोगकर्ता कितनी बार application की window पर switch हुआ (taskbar click,
    Alt+Tab)।
  - `AppLaunch`: इसे कितनी बार taskbar से (एक pinned item से) शुरू किया गया।
  - `ShowJumpView`: इसकी jump list कितनी बार खोली गई (taskbar पर right-click)।
  - `AppBadgeUpdated`: इसका taskbar badge (unread count) कितनी बार बदला।
  - `TrayButtonClicked`: हर taskbar button (`StartButton`, `SearchButton`,
    `TaskViewButton`, `ClockButton`, …) कितनी बार click किया गया।

  Application या तो एक application user model ID होता है (`Microsoft.Windows.Explorer`,
  `Microsoft.WindowsCalculator_8wekyb3d8bbwe!App`), या एक path जिसका पहला भाग किसी known
  folder का GUID हो सकता है (`{1AC14E77-…}\cmd.exe` का अर्थ है `%windir%\system32\cmd.exe`), या उन windows के लिए
  एक process ID (`*PID00001d74`) जिन्हें Explorer किसी application से मिला नहीं सका।
- **RecentApps** (Windows 10, 1709 तक): वे applications जो उपयोगकर्ता ने हाल ही में चलाए, launch
  count और अंतिम समय के साथ; हर एक के अंतर्गत, उसके द्वारा खोली गई फ़ाइलें (`RecentItems`)।
- **Taskbar पिन** (`Taskband`): taskbar पर pin किए गए programs, shell items के रूप में
  (`Favorites`) और हर pinned shortcut के link के रूप में (`FavoritesResolve`)। Windows 10 और
  बाद के संस्करणों पर हर pin बताता है कि उसे Windows ने pin किया (`SystemPinned`) या उपयोगकर्ता ने (`UserPinned`)।
- **ProgramsCache** (`StartPage`, `StartPage2`; Windows XP से 7 तक): Start menu के cached
  programs (`ProgramsCache`), और Windows 7 पर Start menu और taskbar के pinned programs
  (`ProgramsCacheSMP`, `ProgramsCacheTBP`)।

<span class="evidence-strength">सर्वोत्तम निष्कर्ष:</span> इस उपयोगकर्ता ने इस application का इस तरीके से
इतनी बार उपयोग किया (FeatureUsage), इसे इस समय चलाया (RecentApps), या इसे pin किया।
Counters का अपना कोई समय नहीं होता; केवल key का last write और RecentApps के समय ही किसी चीज़ का
समय बताते हैं।

## स्थान { #location }

```text
C:\Users\<user>\NTUSER.DAT
Software\Microsoft\Windows\CurrentVersion\Explorer\FeatureUsage\<kind>     value: application, data: count
Software\Microsoft\Windows\CurrentVersion\Search\RecentApps\{GUID}         AppId, AppPath, LaunchCount, LastAccessedTime
Software\Microsoft\Windows\CurrentVersion\Search\RecentApps\{GUID}\RecentItems\{GUID}   DisplayName, Path, LastAccessedTime
Software\Microsoft\Windows\CurrentVersion\Explorer\Taskband                Favorites, FavoritesResolve
Software\Microsoft\Windows\CurrentVersion\Explorer\StartPage2              ProgramsCache, ProgramsCacheSMP, ProgramsCacheTBP
Software\Microsoft\Windows\CurrentVersion\Explorer\StartPage               ProgramsCache (Windows XP, Vista)
```

`FeatureUsage` का `KeyCreationTime` वह समय है जब इसने गिनती शुरू की; summary इसे दिखाती है। हर profile के लिए
`NTUSER.DAT` को उसके transaction logs के साथ, और host तथा उपयोगकर्ता के SID के लिए सिस्टम के `SYSTEM` और
`SOFTWARE` hives एकत्र करें।

## विश्लेषक के लिए सावधानियाँ { #analyst-cautions }

- **समय के बिना counts.** FeatureUsage count बताता है कितनी बार, कब नहीं। उस प्रकार की key का
  last-write समय (`KeyLastWrittenUTC`) उसके किसी भी count में हुआ सबसे हालिया बदलाव है।
- **हर run नहीं.** `AppSwitched` window switches गिनता है, starts नहीं; जो application
  बिना window के चला, या जिस पर उपयोगकर्ता कभी switch नहीं हुआ, वह गिना नहीं जाता। `AppLaunch` केवल
  taskbar से हुए starts गिनता है।
- **Packaged apps और paths.** Application का नाम वैसा ही होता है जैसे Windows ने उसकी windows को समूहित किया: किसी
  packaged app का ID, किसी program द्वारा अपने लिए सेट किया गया ID (`Chrome`, `VMware.Workstation.vmui`), या एक
  path। `AppPath` वह path देता है जिसमें known folder का GUID उस folder के
  default path से बदल दिया गया हो।
- **Default pins.** नए profiles को default pins मिलते हैं (File Explorer, Edge, the Store); Windows 10
  और बाद के संस्करणों पर वे `SystemPinned` बताते हैं। `UserPinned` pin उपयोगकर्ता का चुनाव है।
- **दो सूचियों में pins.** `Favorites` और `FavoritesResolve` एक ही pins को एक ही
  क्रम (`Index`) में वर्णित करते हैं, लेकिन उन्हें अलग-अलग records के रूप में दिया जाता है।
- **ProgramsCache paths** relative होते हैं: एक folder (`Programs\Accessories`) के बाद उसमें मौजूद
  shortcuts आते हैं (`Notepad.lnk`)।
- **क्षतिग्रस्त values.** जो count `REG_DWORD` नहीं है, जो समय `REG_QWORD` नहीं है, जो pin
  समझा नहीं गया या जो link वास्तव में link नहीं है, वह अपने bytes (`DataRaw`) बनाए रखता है और
  `Problems` में report किया जाता है।

## Juntei से parse करें { #parse-with-juntei }

```text
Usage: juntei parse-registry feature-usage <INPUT> [OPTIONS]

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
juntei parse-registry feature-usage -d evidence/ -o featureusage.csv
juntei parse-registry feature-usage -f NTUSER.DAT -t jsonl \
  | jq 'select(.Source == "AppSwitched") | [.Count, .AppPath // .App]'
```

Summary स्रोत के अनुसार records, हर hive के FeatureUsage ने गिनती कब शुरू की, सबसे अधिक switch किए गए
applications, हाल के apps और items (नवीनतम पहले) और taskbar के pins सूचीबद्ध करती है।

## आउटपुट fields { #output-fields }

| फ़ील्ड | व्याख्या |
|---|---|
| `Host`, `User`, `UserSID`, `SourcePath` | Host, profile folder का नाम, सिस्टम की `ProfileList` से उपयोगकर्ता का SID, और hive। |
| `Source` | `AppSwitched`, `AppLaunch`, `ShowJumpView`, `AppBadgeUpdated`, `TrayButtonClicked` (FeatureUsage), `RecentApps`, `RecentItems`, `Taskband` या `ProgramsCache`। |
| `App` | FeatureUsage value का नाम, किसी recent app का `AppId`, या किसी pin का application user model ID। |
| `AppPath` | Path के रूप में `App`, जिसमें known folder का GUID उसके default path से बदला गया हो; IDs के लिए खाली। |
| `Count` | FeatureUsage count, या किसी recent app का `LaunchCount`। |
| `LastAccessedUTC` | किसी recent app या item का `LastAccessedTime`। |
| `ItemPath` | किसी recent item का `Path` (या किसी recent app का `AppPath`); किसी pin या cached program का shell item path (`User Pinned\TaskBar\File Explorer.lnk`, `Applications\<app ID>`); किसी `FavoritesResolve` link का path। |
| `DisplayName` | किसी recent item का `DisplayName`। |
| `PinType` | `Favorites` pin के लिए `SystemPinned` या `UserPinned`। |
| `Index` | किसी pin या cached program का उसकी सूची में स्थान। |
| `ValueName`, `KeyPath`, `KeyLastWrittenUTC` | वह value और key जिससे यह आता है, और key का last write। |
| `DataRaw` | Bytes (hex) जो decode नहीं होते। |
| `TransactionLogStatus`, `LogsApplied` | Registry recovery का संदर्भ। |
| `Problems` | ग़लत type की values, pins या सूचियाँ जो समझी नहीं गईं, links जो वास्तव में links नहीं हैं। |

## जाँच की रणनीति { #investigation-strategy }

1. उपयोगकर्ता ने सबसे अधिक किन applications में काम किया, यह जानने के लिए `AppSwitched` को `Count` के अनुसार sort करें, और
   administration, remote access, archiving और transfer tools, तथा user-writable स्थानों के paths
   (`\Users\`, `Downloads`, `Temp`) खोजें।
2. Pins जाँचें: कोई `UserPinned` tool, या ऐसा pin जिसका shortcut किसी असामान्य स्थान पर हो।
3. उपयोग का समय निर्धारित करने के लिए RecentApps के समय का UserAssist, Prefetch, BAM और Jump Lists के साथ उपयोग करें।
4. `KeyCreationTime` की तुलना profile के निर्माण से करें: counts तभी से शुरू होते हैं।
5. हटाए गए counters और pins के लिए `parse-registry read --recover-deleted` का उपयोग करें।

## अधिक जानकारी { #further-details }

- [Microsoft: Application User Model IDs](https://learn.microsoft.com/en-us/windows/win32/shell/appids)
- [Microsoft: KNOWNFOLDERID](https://learn.microsoft.com/en-us/windows/win32/shell/knownfolderid)
- [libyal winreg-kb: Programs cache](https://winreg-kb.readthedocs.io/en/latest/sources/explorer-keys/Program-cache.html)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
