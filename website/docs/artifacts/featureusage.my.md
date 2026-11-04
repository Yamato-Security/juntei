# FeatureUsage၊ RecentApps နှင့် Taskbar တွင် ပင်ထိုးထားသော အရာများ { #featureusage-recentapps-and-taskbar-pins }

## ၎င်းတို့ ဆိုသည်မှာ { #what-they-are }

Explorer သည် အသုံးပြုသူတစ်ဦး အပလီကေးရှင်းများကို အသုံးပြုပုံ၏ မှတ်တမ်းများစွာကို ထိုအသုံးပြုသူ၏ `NTUSER.DAT` တွင် သိမ်းထားသည်-

- **FeatureUsage** (Windows 10 နှင့် နောက်ပိုင်း)- အသုံးပြုမှု အမျိုးအစားအလိုက် အပလီကေးရှင်းတစ်ခုချင်းစီ၏ counter များ။
  - `AppSwitched`: အသုံးပြုသူက အပလီကေးရှင်း၏ window သို့ ပြောင်းခဲ့သည့် အကြိမ်ရေ (taskbar ကို နှိပ်ခြင်း၊
    Alt+Tab)။
  - `AppLaunch`: taskbar (ပင်ထိုးထားသော item) မှ စတင်ဖွင့်ခဲ့သည့် အကြိမ်ရေ။
  - `ShowJumpView`: ၎င်း၏ jump list ကို ဖွင့်ခဲ့သည့် အကြိမ်ရေ (taskbar ပေါ်တွင် right-click နှိပ်ခြင်း)။
  - `AppBadgeUpdated`: ၎င်း၏ taskbar badge (မဖတ်ရသေးသော အရေအတွက်) ပြောင်းလဲခဲ့သည့် အကြိမ်ရေ။
  - `TrayButtonClicked`: taskbar ခလုတ်တစ်ခုချင်းစီ (`StartButton`, `SearchButton`,
    `TaskViewButton`, `ClockButton`, …) ကို နှိပ်ခဲ့သည့် အကြိမ်ရေ။

  အပလီကေးရှင်းတစ်ခုဆိုသည်မှာ application user model ID (`Microsoft.Windows.Explorer`,
  `Microsoft.WindowsCalculator_8wekyb3d8bbwe!App`)၊ ပထမအပိုင်းသည် known folder တစ်ခု၏ GUID ဖြစ်နိုင်သော
  path (`{1AC14E77-…}\cmd.exe` သည် `%windir%\system32\cmd.exe` ဖြစ်သည်) သို့မဟုတ် Explorer က အပလီကေးရှင်းနှင့်
  မကိုက်ညီနိုင်ခဲ့သော window များအတွက် process ID (`*PID00001d74`) ဖြစ်သည်။
- **RecentApps** (Windows 10 မှ 1709 အထိ)- အသုံးပြုသူ မကြာသေးမီက run ခဲ့သော အပလီကေးရှင်းများ၊ စတင်ဖွင့်မှု
  အကြိမ်ရေနှင့် နောက်ဆုံးအချိန်တို့နှင့်အတူ။ တစ်ခုချင်းစီ၏ အောက်တွင် ၎င်းဖွင့်ခဲ့သော ဖိုင်များ (`RecentItems`) ရှိသည်။
- **Taskbar ပင်များ** (`Taskband`)- taskbar တွင် ပင်ထိုးထားသော ပရိုဂရမ်များကို shell item များအဖြစ်
  (`Favorites`) နှင့် ပင်ထိုးထားသော ဖြတ်လမ်းတစ်ခုချင်းစီ၏ link အဖြစ် (`FavoritesResolve`)။ Windows 10 နှင့်
  နောက်ပိုင်းတွင် ပင်တစ်ခုစီက Windows က ပင်ထိုးခဲ့သည် (`SystemPinned`) သို့မဟုတ် အသုံးပြုသူက ပင်ထိုးခဲ့သည် (`UserPinned`) ကို ဖော်ပြသည်။
- **ProgramsCache** (`StartPage`, `StartPage2`; Windows XP မှ 7 အထိ)- Start menu ၏ cache လုပ်ထားသော
  ပရိုဂရမ်များ (`ProgramsCache`) နှင့် Windows 7 တွင် Start menu နှင့် taskbar တွင် ပင်ထိုးထားသော ပရိုဂရမ်များ
  (`ProgramsCacheSMP`, `ProgramsCacheTBP`)။

<span class="evidence-strength">အကောင်းဆုံး ကောက်ချက်:</span> ဤအသုံးပြုသူသည် ဤအပလီကေးရှင်းကို ဤနည်းဖြင့်
ဤမျှ အကြိမ်ရေ အသုံးပြုခဲ့သည် (FeatureUsage)၊ ဤအချိန်တွင် run ခဲ့သည် (RecentApps) သို့မဟုတ် ပင်ထိုးခဲ့သည်။
Counter များတွင် ကိုယ်ပိုင်အချိန် မရှိပါ။ Key ၏ နောက်ဆုံးရေးသားချိန်နှင့် RecentApps ၏ အချိန်များသာ ရက်စွဲသတ်မှတ်ပေးနိုင်သည်။

## တည်နေရာ { #location }

```text
C:\Users\<user>\NTUSER.DAT
Software\Microsoft\Windows\CurrentVersion\Explorer\FeatureUsage\<kind>     value: application, data: count
Software\Microsoft\Windows\CurrentVersion\Search\RecentApps\{GUID}         AppId, AppPath, LaunchCount, LastAccessedTime
Software\Microsoft\Windows\CurrentVersion\Search\RecentApps\{GUID}\RecentItems\{GUID}   DisplayName, Path, LastAccessedTime
Software\Microsoft\Windows\CurrentVersion\Explorer\Taskband                Favorites, FavoritesResolve
Software\Microsoft\Windows\CurrentVersion\Explorer\StartPage2              ProgramsCache, ProgramsCacheSMP, ProgramsCacheTBP
Software\Microsoft\Windows\CurrentVersion\Explorer\StartPage               ProgramsCache (Windows XP, Vista)
```

`FeatureUsage` ၏ `KeyCreationTime` သည် ရေတွက်မှု စတင်ခဲ့သည့်အချိန် ဖြစ်ပြီး အနှစ်ချုပ်တွင် ၎င်းကို ပြသည်။
Profile တိုင်းအတွက် `NTUSER.DAT` ကို ၎င်း၏ transaction log များနှင့်အတူ စုဆောင်းပြီး host နှင့် အသုံးပြုသူ၏ SID
အတွက် စနစ်၏ `SYSTEM` နှင့် `SOFTWARE` hive များကို စုဆောင်းပါ။

## စိစစ်သူများ သတိပြုရန် { #analyst-cautions }

- **အချိန်မပါသော အရေအတွက်များ။** FeatureUsage အရေအတွက်က မည်မျှ မကြာခဏဆိုသည်ကို ဖော်ပြပြီး မည်သည့်အချိန်ဆိုသည်ကို
  မဖော်ပြပါ။ အမျိုးအစား key ၏ last-write အချိန် (`KeyLastWrittenUTC`) သည် ၎င်း၏ အရေအတွက်တစ်ခုခု၏
  နောက်ဆုံးပြောင်းလဲမှု ဖြစ်သည်။
- **Run တိုင်းကို မရေတွက်။** `AppSwitched` သည် စတင်ဖွင့်မှုများကို မဟုတ်ဘဲ window ပြောင်းမှုများကို ရေတွက်သည်။
  Window မပါဘဲ run ခဲ့သော သို့မဟုတ် အသုံးပြုသူ တစ်ကြိမ်မျှ ပြောင်းမသွားခဲ့သော အပလီကေးရှင်းကို မရေတွက်ပါ။
  `AppLaunch` သည် taskbar မှ စတင်ဖွင့်မှုများကိုသာ ရေတွက်သည်။
- **Package app များနှင့် path များ။** အပလီကေးရှင်းကို Windows က ၎င်း၏ window များကို အုပ်စုဖွဲ့ခဲ့သည့်အတိုင်း
  အမည်ပေးသည်- package app ၏ ID၊ ပရိုဂရမ်က ကိုယ်တိုင်သတ်မှတ်ထားသော ID (`Chrome`, `VMware.Workstation.vmui`)
  သို့မဟုတ် path တစ်ခု။ `AppPath` သည် known folder ၏ GUID ကို ထိုဖိုင်တွဲ၏ ပုံသေ path ဖြင့် အစားထိုးထားသော path ကို ပေးသည်။
- **ပုံသေ ပင်များ။** Profile အသစ်များသည် ပုံသေ ပင်များ (File Explorer, Edge, Store) ကို ရရှိသည်။ Windows 10 နှင့်
  နောက်ပိုင်းတွင် ၎င်းတို့က `SystemPinned` ဟု ဖော်ပြသည်။ `UserPinned` ပင်သည် အသုံးပြုသူ၏ ရွေးချယ်မှု ဖြစ်သည်။
- **စာရင်းနှစ်ခုရှိ ပင်များ။** `Favorites` နှင့် `FavoritesResolve` သည် တူညီသော ပင်များကို တူညီသော အစီအစဉ်
  (`Index`) ဖြင့် ဖော်ပြသော်လည်း ၎င်းတို့ကို သီးခြား record များအဖြစ် ပေးသည်။
- **ProgramsCache path များ** သည် relative ဖြစ်သည်- ဖိုင်တွဲတစ်ခု (`Programs\Accessories`) ၏ နောက်တွင်
  ၎င်းအတွင်းရှိ ဖြတ်လမ်းများ (`Notepad.lnk`) လိုက်သည်။
- **ပျက်စီးနေသော value များ။** `REG_DWORD` မဟုတ်သော အရေအတွက်၊ `REG_QWORD` မဟုတ်သော အချိန်၊ နားမလည်နိုင်သော
  ပင် သို့မဟုတ် link မဟုတ်သော link သည် ၎င်း၏ byte များ (`DataRaw`) ကို ထိန်းသိမ်းထားပြီး `Problems` တွင် အစီရင်ခံသည်။

## Juntei ဖြင့် ခွဲခြမ်းစိတ်ဖြာခြင်း { #parse-with-juntei }

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

အနှစ်ချုပ်တွင် source အလိုက် record များ၊ hive တစ်ခုစီ၏ FeatureUsage ရေတွက်မှု စတင်ခဲ့သည့်အချိန်၊ အများဆုံး
ပြောင်းသွားခဲ့သော အပလီကေးရှင်းများ၊ မကြာသေးမီက app များနှင့် item များ (အသစ်ဆုံး ဦးစွာ) နှင့် taskbar ၏ ပင်များကို ဖော်ပြသည်။

## ထုတ်ယူမှု field များ { #output-fields }

| Field | အဓိပ္ပာယ်ဖော်ပုံ |
|---|---|
| `Host`, `User`, `UserSID`, `SourcePath` | Host၊ profile ဖိုင်တွဲ၏ အမည်၊ စနစ်၏ `ProfileList` မှ အသုံးပြုသူ၏ SID နှင့် hive။ |
| `Source` | `AppSwitched`, `AppLaunch`, `ShowJumpView`, `AppBadgeUpdated`, `TrayButtonClicked` (FeatureUsage), `RecentApps`, `RecentItems`, `Taskband` သို့မဟုတ် `ProgramsCache`။ |
| `App` | FeatureUsage value ၏ အမည်၊ မကြာသေးမီက app ၏ `AppId` သို့မဟုတ် ပင်၏ application user model ID။ |
| `AppPath` | `App` ကို path အဖြစ်၊ known folder ၏ GUID ကို ၎င်း၏ ပုံသေ path ဖြင့် အစားထိုးထားသည်။ ID များအတွက် ဗလာ။ |
| `Count` | FeatureUsage အရေအတွက် သို့မဟုတ် မကြာသေးမီက app ၏ `LaunchCount`။ |
| `LastAccessedUTC` | မကြာသေးမီက app သို့မဟုတ် item ၏ `LastAccessedTime`။ |
| `ItemPath` | မကြာသေးမီက item ၏ `Path` (သို့မဟုတ် မကြာသေးမီက app ၏ `AppPath`)။ ပင် သို့မဟုတ် cache လုပ်ထားသော ပရိုဂရမ်၏ shell item path (`User Pinned\TaskBar\File Explorer.lnk`, `Applications\<app ID>`)။ `FavoritesResolve` link ၏ path။ |
| `DisplayName` | မကြာသေးမီက item ၏ `DisplayName`။ |
| `PinType` | `Favorites` ပင်အတွက် `SystemPinned` သို့မဟုတ် `UserPinned`။ |
| `Index` | ပင် သို့မဟုတ် cache လုပ်ထားသော ပရိုဂရမ်၏ ၎င်း၏ စာရင်းအတွင်းရှိ နေရာ။ |
| `ValueName`, `KeyPath`, `KeyLastWrittenUTC` | ၎င်းလာရာ value နှင့် key နှင့် key ၏ နောက်ဆုံးရေးသားချိန်။ |
| `DataRaw` | Decode မလုပ်နိုင်သော byte များ (hex)။ |
| `TransactionLogStatus`, `LogsApplied` | Registry ပြန်လည်ရယူမှု အခြေအနေ။ |
| `Problems` | အမျိုးအစားမှားသော value များ၊ နားမလည်နိုင်သော ပင်များ သို့မဟုတ် စာရင်းများ၊ link မဟုတ်သော link များ။ |

## စုံစမ်းစစ်ဆေးမှု ဗျူဟာ { #investigation-strategy }

1. အသုံးပြုသူ အများဆုံး အလုပ်လုပ်ခဲ့သော အပလီကေးရှင်းများအတွက် `AppSwitched` ကို `Count` အလိုက် စီပြီး
   စီမံခန့်ခွဲရေး၊ အဝေးမှ ဝင်ရောက်ရေး၊ archive လုပ်ရေးနှင့် လွှဲပြောင်းရေး tool များနှင့် အသုံးပြုသူ ရေးသားနိုင်သော
   နေရာများရှိ path များ (`\Users\`, `Downloads`, `Temp`) ကို ရှာဖွေပါ။
2. ပင်များကို စစ်ဆေးပါ- `UserPinned` tool တစ်ခု သို့မဟုတ် ၎င်း၏ ဖြတ်လမ်းသည် ပုံမှန်မဟုတ်သော နေရာတွင် ရှိသော ပင်တစ်ခု။
3. အသုံးပြုမှုကို ရက်စွဲသတ်မှတ်ရန် RecentApps ၏ အချိန်များကို UserAssist၊ Prefetch၊ BAM နှင့် Jump Lists တို့နှင့်အတူ အသုံးပြုပါ။
4. `KeyCreationTime` ကို profile ၏ ဖန်တီးချိန်နှင့် နှိုင်းယှဉ်ပါ- အရေအတွက်များသည် ထိုအချိန်မှ စတင်သည်။
5. ဖျက်ထားသော counter များနှင့် ပင်များအတွက် `parse-registry read --recover-deleted` ကို အသုံးပြုပါ။

## နောက်ထပ် အသေးစိတ်များ { #further-details }

- [Microsoft: Application User Model IDs](https://learn.microsoft.com/en-us/windows/win32/shell/appids)
- [Microsoft: KNOWNFOLDERID](https://learn.microsoft.com/en-us/windows/win32/shell/knownfolderid)
- [libyal winreg-kb: Programs cache](https://winreg-kb.readthedocs.io/en/latest/sources/explorer-keys/Program-cache.html)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
