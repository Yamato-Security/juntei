# စနစ်အချက်အလက် { #system-information }

## ၎င်း ဆိုသည်မှာ { #what-it-is }

စနစ်တစ်ခု၏ machine hive လေးခုသည် analyst တစ်ဦးက ၎င်း၏ timeline ကို မဖတ်မီ သိရန် လိုအပ်သည်များကို ဖော်ပြပါသည်။

- ၎င်း၏ အမည်များ (NetBIOS၊ DNS host အမည်၊ DNS domain) နှင့် ၎င်းပါဝင်သော domain သို့မဟုတ် workgroup၊
- ၎င်း၏ Windows ဗားရှင်း၊ build နှင့် update revision၊ Windows ကို ထည့်သွင်းခဲ့သည့် သို့မဟုတ် နောက်ဆုံး upgrade လုပ်ခဲ့သည့်အချိန်၊
- ၎င်း၏ time zone၊ စနစ်က ထိန်းသိမ်းခဲ့သော local အချိန်များကို သိစေပါသည်၊
- ၎င်းကို နောက်ဆုံး ပုံမှန်အတိုင်း shutdown လုပ်ခဲ့သည့်အချိန်၊
- ၎င်း၏ processor architecture နှင့် code page များ၊
- နောက်ဆုံး sign in ဝင်ခဲ့သော အသုံးပြုသူနှင့် sign in ဝင်ခဲ့ဖူးသော အသုံးပြုသူတိုင်း၏ profile များ၊
- ၎င်း၏ SID များ: ကွန်ပျူတာ၏ SID (စက်တွင်း account များသည် ၎င်းနှင့် RID တစ်ခု) နှင့် ၎င်း၏ domain ၏ SID၊
- ၎င်းကို upgrade လုပ်ခဲ့သည့် မူလ Windows ဗားရှင်းများ။

`parse-registry system-info` သည် ဤအရာအားလုံးပါဝင်သော record တစ်ခုကို စနစ်တစ်ခုလျှင် ရေးသားပါသည်။

<span class="evidence-strength">အကောင်းဆုံး ကောက်ချက်:</span> hive များကို စုဆောင်းခဲ့သည့်အချိန် (သို့မဟုတ် နောက်ဆုံးရေးသားခဲ့သည့်အချိန်)
ရှိ ပြင်ဆင်မှု ဖြစ်ပါသည်။ Key အချိန်များသည် ၎င်း၏ အစိတ်အပိုင်းများ နောက်ဆုံး ပြောင်းလဲခဲ့သည့်အချိန်ကို ဖော်ပြပါသည်။

## တည်နေရာ { #location }

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

Hive လေးခုကို ၎င်းတို့၏ transaction log များနှင့်အတူ စုဆောင်းပါ။ ၎င်းတို့ထဲမှ မည်သည့်တစ်ခုတည်းကမဆို ၎င်းတွင် ပါဝင်သည့်အရာကို ပေးပါသည်။

## စိစစ်သူများ သတိပြုရန် { #analyst-cautions }

- **Registry အချိန်များသည် UTC ဖြစ်ပြီး time zone သည် အခြားအရာအားလုံးအတွက် ဖြစ်ပါသည်။** Key အချိန်များ၊ FILETIME များနှင့်
  event အချိန်အများစုသည် UTC ဖြစ်ပါသည်။ Text log များ၊ application log အချို့နှင့် အမည်များ သို့မဟုတ် string များထဲ ရေးထားသော ရက်စွဲများ
  (`Source OS (Updated on …)` ကဲ့သို့) သည် local ဖြစ်နိုင်ပါသည်။ `Bias` သည် local standard အချိန်မှ UTC သို့ မိနစ်ဖြစ်ပါသည်
  (`UTC = local + Bias`)။ `UtcOffset` နှင့် `DaylightUtcOffset` က offset နှစ်ခုကို ပေးပြီး
  `TimeZoneDescription` သည် daylight အချိန် စတင်ချိန်နှင့် ကုန်ဆုံးချိန်ကို ဖော်ပြပါသည်။
- **Time zone ပြောင်းလဲနိုင်ပါသည်။** `TimeZoneLastWrittenUTC` သည် key ၏ နောက်ဆုံးရေးသားချိန် ဖြစ်ပါသည်။ Windows သည်
  `ActiveTimeBias` (ထိုအချိန်က သက်ရောက်နေသော offset) ကို update လုပ်သည့်အခါ key ကို ကိုယ်တိုင် ပြန်ရေးသောကြောင့်
  မကြာသေးမီ အချိန်တစ်ခုသည် တစ်စုံတစ်ယောက်က zone ကို ပြောင်းခဲ့ကြောင်း မသက်သေပြပါ။ Event log များနှင့် နှိုင်းယှဉ်ပါ: Security 4616
  (စနစ်အချိန် ပြောင်းလဲခဲ့သည်) နှင့် System log ၏ `Microsoft-Windows-Kernel-General` event များ။
- **`StandardStart` နှင့် `DaylightStart` တို့သည် kernel ၏ `TIME_FIELDS` ဖြစ်ပြီး `SYSTEMTIME` များ မဟုတ်ပါ။**
  ရက်သတ္တပတ်၏နေ့သည် ၎င်းတို့၏ တတိယ field မဟုတ်ဘဲ (`SOFTWARE` ရှိ zone ၏ `TZI` value ကဲ့သို့) နောက်ဆုံး field ဖြစ်ပါသည်။
  US zone များတွင် ရက်သတ္တပတ်၏နေ့သည် တနင်္ဂနွေ (0) ဖြစ်သောကြောင့် layout နှစ်မျိုးလုံး တူညီစွာ ဖတ်ရပါသည်။ Juntei သည် ၎င်းတို့ကို
  `TIME_FIELDS` အဖြစ် decode လုပ်ပါသည်။ အများသုံး Easter Island စနစ်တစ်ခု (ပထမ စနေနေ့ 22:00) က ကွာခြားချက်ကို ပြသပါသည်။
  လ 0 သည် daylight saving time မရှိကြောင်း ဆိုလိုပါသည်။
- **Windows 11 သည် "Windows 10" ဟု ဖော်ပြပါသည်။** `ProductName` သည် Windows 11 တွင် `Windows 10 …` ကို ထိန်းထားပါသည်။
  Build 22000 နှင့် နောက်ပိုင်းသည် Windows 11 ဖြစ်ပြီး အနှစ်ချုပ်က ထိုသို့ ဖော်ပြပါသည်။ `CurrentVersion` သည် Windows 8.1၊ 10
  နှင့် 11 တွင် `6.3` ဖြစ်ပါသည်။ `Version` သည် `CurrentMajorVersionNumber` (10) ရှိလျှင် ၎င်းကို အသုံးပြုပါသည်။
- **`InstallDate` သည် feature upgrade များနှင့်အတူ ရွေ့ပါသည်။** ထုတ်ဝေမှုအသစ်သို့ in-place upgrade (ဥပမာ 1809 မှ
  21H2) သည် ၎င်းကို reset လုပ်ပါသည်။ Upgrade တစ်ခုစီက အစားထိုးခဲ့သော ထုတ်ဝေမှုကို ၎င်း၏ ကိုယ်ပိုင် `InstallDate` နှင့်အတူ
  `PreviousVersions` (`Setup\Source OS`) တွင် တွေ့နိုင်ပါသည်။ ၎င်းတို့၏ အစောဆုံး `InstallDate` သည် မူလ ထည့်သွင်းမှုနှင့်
  အနီးစပ်ဆုံး ဖြစ်ပါသည်။ `Source OS` key ၏ အမည်ရှိ ရက်စွဲသည် ၎င်း၏ time zone ကို မဖော်ပြပါ။
- **`ShutdownTime` သည် နောက်ဆုံး ပုံမှန် shutdown ဖြစ်ပါသည်။** Crash သို့မဟုတ် ပါဝါပြတ်တောက်မှုက ၎င်းကို update မလုပ်ပါ။
  ၎င်းကို event log များ (6006, 6008, 1074) နှင့် hive များသို့ နောက်ဆုံးရေးသားခဲ့သည့် အချိန်များနှင့် နှိုင်းယှဉ်ပါ။
- **`LastLoggedOnUser` သည် sign-in မျက်နှာပြင်၏ နောက်ဆုံးအသုံးပြုသူ ဖြစ်ပြီး** logon တိုင်း မဟုတ်ပါ။ Remote Desktop၊
  `runas`၊ service များနှင့် scheduled task များ အားလုံးက ၎င်းကို မသတ်မှတ်ပါ။ `LogonUiLastWrittenUTC` သည် ၎င်းကို နောက်ဆုံး
  သတ်မှတ်ခဲ့သည့် အချိန်ခန့် ဖြစ်ပါသည်။
- **Profile များ:** `ProfileList` သည် စနစ်ပေါ်တွင် profile ရှိခဲ့ဖူးသော အသုံးပြုသူ (နှင့် service) တိုင်းကို ဖော်ပြပြီး ဖျက်ထားသော
  account များကိုလည်း ၎င်းတို့၏ profile ကို မဖယ်ရှားမချင်း ပါဝင်ပါသည်။ Windows 10 နှင့် နောက်ပိုင်းတွင်
  `LocalProfileLoadTime` နှင့် `LocalProfileUnloadTime` ကို ရေးသားပါသည်: နောက်ဆုံး sign-in နှင့် sign-out ခန့် ဖြစ်ပါသည်။
- **SID များ:** စက်တွင်း account များသည် `MachineSid-<RID>`၊ domain account များသည် `DomainSid-<RID>` ဖြစ်ပါသည်။
  `MachineSid` သည် `SAM` မှ လာပါသည် (မဟုတ်လျှင် `SECURITY` ၏ `PolAcDmS` မှ)။ နှစ်ခုသည် ကိုက်ညီသင့်ပါသည်။
  မကိုက်ညီလျှင် မတူညီသော စနစ်များ၏ hive များကဲ့သို့ `Problems` က ထိုသို့ ဖော်ပြပါသည်။ Workgroup တွင် `DomainSid` သည်
  ဗလာ ဖြစ်ပါသည်။
- **ဖိုင်တွဲတစ်ခု၊ စနစ်တစ်ခု။** `-d` ဖြင့် ဖိုင်တွဲတစ်ခုအတွင်းရှိ hive များကို စနစ်တစ်ခု၏ hive များအဖြစ် မှတ်ယူပါသည်။
  `RegBack` မိတ္တူများနှင့် အမျိုးအစားတူ ဒုတိယ hive (`SYSTEM.old`) တို့သည် ကိုယ်ပိုင် record များ ဖြစ်ပါသည်။
  `-f` ဖြင့် ပေးထားသော hive များကို စနစ်တစ်ခု၏ hive များအဖြစ် မှတ်ယူပြီး အခြား hive များကို ၎င်းတို့၏ ဘေးတွင် ရှာဖွေပါသည်။
- **Control set များ:** Windows နောက်ဆုံးအသုံးပြုခဲ့သော control set ကိုသာ ဖတ်ပါသည်။ `parse-registry read` က
  အခြား control set များကို ပြသပါသည်။

## Juntei ဖြင့် ခွဲခြမ်းစိတ်ဖြာခြင်း { #parse-with-juntei }

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

အနှစ်ချုပ်သည် စနစ်တစ်ခုစီအတွက် အောက်ပါတို့ကို ပေးပါသည်: ၎င်း၏ Windows ဗားရှင်း၊ ထည့်သွင်းခဲ့သည့်အချိန်၊ စကားလုံးဖြင့်
ဖော်ပြထားသော ၎င်း၏ time zone၊ key ကို နောက်ဆုံးရေးသားခဲ့စဉ်က သက်ရောက်နေသော offset၊ ၎င်း၏ နောက်ဆုံး shutdown၊ ၎င်း၏ domain
သို့မဟုတ် workgroup၊ နောက်ဆုံး sign in ဝင်ခဲ့သော အသုံးပြုသူ၊ ၎င်း၏ SID၊ ၎င်းတွင်ရှိသော profile အရေအတွက်နှင့် ၎င်းကို upgrade
လုပ်ခဲ့သည့် မူလဗားရှင်းများ။ `auto-parse` သည်လည်း တွေ့ရှိသော စနစ်များကို ဖော်ပြသည့်အခါ စနစ်တစ်ခုစီ၏ time zone ကို ပုံနှိပ်ပါသည်။

## ထုတ်ယူမှု field များ { #output-fields }

| Field | အဓိပ္ပာယ်ဖော်ချက် |
|---|---|
| `Host`, `SourcePath`, `SoftwarePath`, `SamPath`, `SecurityPath` | Host နှင့် ဖတ်ခဲ့သော hive များ (`SourcePath` သည် `SYSTEM` hive၊ မဟုတ်လျှင် ပထမဆုံးဖတ်ခဲ့သော hive)။ |
| `CompName`, `Hostname`, `Domain` | NetBIOS အမည် (`SYSTEM`၊ မဟုတ်လျှင် `SECURITY` ၏ `PolAcDmN`)၊ DNS host အမည်နှင့် DNS domain (`Tcpip\Parameters`)။ |
| `PrimaryDomain`, `DNS-Domain` | ကွန်ပျူတာ join လုပ်ထားသော domain သို့မဟုတ် ၎င်း၏ workgroup (`PolPrDmN`) နှင့် domain ၏ DNS အမည် (`PolDnDDN`)။ |
| `ProductName`, `Edition`, `DisplayVersion`, `Version`, `Build`, `Ubr`, `ServicePack`, `InstallationType` | `CurrentVersion`: ရေးထားသည့်အတိုင်း product အမည်၊ `EditionID`၊ `DisplayVersion` (မဟုတ်လျှင် `ReleaseId`)၊ major နှင့် minor ဗားရှင်း (မဟုတ်လျှင် `CurrentVersion`)၊ build၊ update build revision၊ `CSDVersion` နှင့် `Client`၊ `Server` သို့မဟုတ် `Server Core`။ |
| `InstallDateUTC`, `InstallTimeUTC` | `InstallDate` (စက္ကန့်အထိ) နှင့် `InstallTime` (100 ns အထိ၊ Windows 10 နှင့် နောက်ပိုင်း): ထည့်သွင်းမှု သို့မဟုတ် နောက်ဆုံး feature upgrade။ |
| `RegisteredOwner`, `RegisteredOrganization`, `SystemRoot` | ရေးထားသည့်အတိုင်း။ |
| `TimeZone`, `TimeZoneDescription` | `TimeZoneKeyName` (မဟုတ်လျှင် `StandardName`) နှင့် ၎င်း၏ offset များနှင့် daylight saving ရက်စွဲများပါသော စကားလုံးဖြင့် ဖော်ပြထားသည့် zone။ |
| `UTC-Offset`, `DaylightUTC-Offset` | UTC မှ local standard အချိန်၏ offset နှင့် daylight အချိန်၏ offset (daylight saving time မရှိလျှင် ဗလာ)။ |
| `BiasMinutes`, `ActiveBiasMinutes`, `StandardBiasMinutes`, `DaylightBiasMinutes` | `Bias`၊ `ActiveTimeBias` (key ကို ရေးသားခဲ့စဉ်က သက်ရောက်နေသော)၊ `StandardBias`၊ `DaylightBias`။ |
| `StandardName`, `DaylightName` | ရေးထားသည့်အတိုင်း၊ မကြာခဏ resource reference (`@tzres.dll,-112`) ဖြစ်ပါသည်။ |
| `StandardStart`, `DaylightStart` | Standard နှင့် daylight အချိန် စတင်ချိန်ကို စကားလုံးဖြင့် (`first Sunday of November, 02:00`)။ |
| `DynamicDaylightTimeDisabled`, `RealTimeIsUniversal` | Dynamic daylight saving time ကို ပိတ်ထားခြင်း၊ hardware နာရီက UTC ကို ထိန်းထားခြင်း။ |
| `TimeZoneLastWrittenUTC` | Time zone key ၏ နောက်ဆုံးရေးသားချိန်။ |
| `LastShutdownUTC` | `ShutdownTime`: နောက်ဆုံး ပုံမှန် shutdown။ |
| `CurrentControlSet`, `ProcessorArchitecture`, `AnsiCodePage`, `OemCodePage` | ဖတ်ခဲ့သော control set၊ `PROCESSOR_ARCHITECTURE`၊ `ACP` နှင့် `OEMCP`။ |
| `LastLoggedOnUser`, `LastLoggedOnSamUser`, `LastLoggedOnUserSID`, `LastLoggedOnDisplayName`, `LogonUiLastWrittenUTC` | Sign-in မျက်နှာပြင်၏ နောက်ဆုံးအသုံးပြုသူနှင့် ၎င်း၏ key ၏ နောက်ဆုံးရေးသားချိန်။ |
| `MachineSID`, `DomainSID`, `MachineAccountSID` | ကွန်ပျူတာ၏ SID၊ ၎င်း၏ domain ၏ SID (`PolPrDmS`) နှင့် domain အတွင်းရှိ ကွန်ပျူတာ account ၏ SID (`PolMachineAccountS`)။ |
| `CompNameLastWrittenUTC`, `PrimaryDomainLastWrittenUTC`, `VersionLastWrittenUTC` | ကွန်ပျူတာကို နောက်ဆုံး အမည်ပေးခဲ့သည့်၊ domain သို့မဟုတ် workgroup သို့ နောက်ဆုံး join လုပ်ခဲ့သည့်နှင့် နောက်ဆုံး update လုပ်ခဲ့သည့် (`CurrentVersion` ၏ key) အချိန်ခန့်။ |
| `PreviousVersions` | `Source OS` key တစ်ခုစီ: ဗားရှင်း၊ ၎င်း၏ ထည့်သွင်းသည့်ရက်စွဲနှင့် key ၏ အမည်ရှိ ရက်စွဲ (JSON: တစ်ခုလျှင် object တစ်ခု)။ |
| `Profiles` | Profile တစ်ခုစီ: SID၊ ဖိုင်တွဲ၊ key အချိန်၊ နောက်ဆုံး load နှင့် unload (JSON: တစ်ခုလျှင် object တစ်ခု)။ |
| `TransactionLogStatus`, `LogsApplied` | Hive လေးခုလုံးအတွက် registry ပြန်လည်ရယူမှု context။ |
| `Problems` | Type သို့မဟုတ် size မှားသော value များ (၎င်းတို့၏ byte များကို ထိန်းထားသည်) နှင့် ကွန်ပျူတာ SID များ ကွဲပြားသော `SAM` နှင့် `SECURITY`။ |

## စုံစမ်းစစ်ဆေးမှု ဗျူဟာ { #investigation-strategy }

1. Time zone ကို ဦးစွာ ဖတ်ပါ: `UtcOffset`၊ daylight saving ရက်စွဲများနှင့် zone ကို မကြာသေးမီက ပြောင်းခဲ့/မပြောင်းခဲ့ကို
   မှတ်သားပါ။ Local-time log များကို ၎င်းဖြင့် ပြောင်းလဲပါ။
2. ဗားရှင်း၊ build နှင့် ထည့်သွင်းသည့်ရက်စွဲကို မှတ်သားပါ။ Upgrade များအတွက် `PreviousVersions` ကို စစ်ဆေးပြီး ထည့်သွင်းသည့်ရက်စွဲကို
   အဟောင်းဆုံး ဖိုင်စနစ်နှင့် event log အချိန်များနှင့် နှိုင်းယှဉ်ပါ။
3. `LastShutdownUTC` ကို မှတ်သားပါ: ၎င်းနောက်ပိုင်း shutdown အချိန်အသစ် မရှိဘဲ လှုပ်ရှားမှု ရှိခြင်းသည် crash၊ ပါဝါပြတ်တောက်မှု
   သို့မဟုတ် live collection ကို ညွှန်ပြပါသည်။
4. အခြားနေရာများရှိ SID များ (BAM၊ `ProfileList`၊ event log များ) တွင် စက်တွင်းနှင့် domain account များကို ခွဲခြားရန်
   `MachineSid` နှင့် `DomainSid` ကို အသုံးပြုပြီး နောက်ဆုံး sign-in များအတွက် profile များ၏ load နှင့် unload အချိန်များကို အသုံးပြုပါ။
5. ကွန်ပျူတာနှင့် domain အမည်များကို event log များနှင့် ဆက်စပ်စစ်ဆေးပြီး အမည်ပြောင်းခြင်းနှင့် domain join များအတွက်
   `CompNameLastWrittenUTC` နှင့် `PrimaryDomainLastWrittenUTC` တို့နှင့် ဆက်စပ်စစ်ဆေးပါ။

## နောက်ထပ် အသေးစိတ်များ { #further-details }

- [Microsoft: TIME_ZONE_INFORMATION](https://learn.microsoft.com/en-us/windows/win32/api/timezoneapi/ns-timezoneapi-time_zone_information)
- [Microsoft: DYNAMIC_TIME_ZONE_INFORMATION](https://learn.microsoft.com/en-us/windows/win32/api/timezoneapi/ns-timezoneapi-dynamic_time_zone_information)
- [Microsoft: TIME_FIELDS](https://learn.microsoft.com/en-us/windows-hardware/drivers/ddi/wdm/ns-wdm-_time_fields)
- [Microsoft: Security identifier များ](https://learn.microsoft.com/en-us/windows-server/identity/ad-ds/manage/understand-security-identifiers)
- [libyal winreg-kb: Time zone များ](https://github.com/libyal/winreg-kb/blob/main/docs/sources/system-keys/Time-zones.md)
- [libyal winreg-kb: Windows product အချက်အလက်](https://github.com/libyal/winreg-kb/blob/main/docs/sources/system-keys/Windows-product-information.md)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
