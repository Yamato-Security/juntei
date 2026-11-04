# MuiCache { #muicache }

## MuiCache ဆိုသည်မှာ { #what-muicache-is }

အသုံးပြုသူတစ်ဦး ပရိုဂရမ်တစ်ခုကို စတင်ဖွင့်သည့်အခါ Windows သည် shell က ပြသသင့်သော အမည် (ဥပမာ `cmd.exe`
အတွက် "Windows Command Processor" ဖြစ်သော `FileDescription`) ကို ပရိုဂရမ်၏ version resource များမှ ဖတ်ပြီး
အသုံးပြုသူအလိုက် `MuiCache` တွင် cache လုပ်ထားသည်။ Windows 8 နှင့် နောက်ပိုင်းတွင် ကုမ္ပဏီ (`CompanyName`)
ကိုလည်း cache လုပ်သည်။ Cache ကို ပရိုဂရမ်၏ full path ဖြင့် key သတ်မှတ်ထားသည်-

- **Windows 8 နှင့် နောက်ပိုင်း-** ပရိုဂရမ်တစ်ခုလျှင် value နှစ်ခု၊ `<path>.FriendlyAppName` နှင့်
  `<path>.AppCompany`။
- **Windows Vista နှင့် 7-** ပရိုဂရမ်တစ်ခုလျှင် value တစ်ခု၊ friendly name ပါဝင်သော `<path>`။
- **Windows XP-** တူညီသော ပုံစံဖြင့် `NTUSER.DAT` တွင်။

MuiCache သည် အသုံးပြုသူ၏ shell session အတွက် ပရိုဂရမ်လုပ်ဆောင်မှု သဲလွန်စများကို ပေးနိုင်ပြီး removable drive များ၊
network share များနှင့် ယခုမရှိတော့သော ဖိုင်တွဲများရှိ ပရိုဂရမ်များကိုလည်း ဖိုင်ကိုယ်တိုင်ပေးခဲ့သော အမည်နှင့်
ကုမ္ပဏီတို့နှင့်အတူ ပါဝင်သည်။

<span class="evidence-strength">အကောင်းဆုံး ကောက်ချက်:</span> ဤ shell profile သည် ဖိုင်တစ်ခု၏ path၊
ဖော်ပြချက်နှင့် ကုမ္ပဏီကို cache လုပ်ခဲ့သည်။ ၎င်းကို အခြား လုပ်ဆောင်မှု အထောက်အထားများနှင့် ချိတ်ဆက်စစ်ဆေးပါ။
Profile က process run ခဲ့သော အကောင့်ကို ဖော်ထုတ်မပေးပါ။ MuiCache တွင် ပရိုဂရမ်တစ်ခုချင်းစီအတွက် အချိန်နှင့်
run အကြိမ်ရေ မရှိပါ။

## တည်နေရာ { #location }

```text
C:\Users\<user>\AppData\Local\Microsoft\Windows\UsrClass.dat     Windows Vista and later
Local Settings\Software\Microsoft\Windows\Shell\MuiCache

C:\Documents and Settings\<user>\NTUSER.DAT                         Windows XP
Software\Microsoft\Windows\ShellNoRoam\MUICache
```

`LangID` (cache လုပ်ထားသော string များ၏ ဘာသာစကား) သည် ပရိုဂရမ်မဟုတ်သောကြောင့် ချန်လှပ်ထားသည်။
`@<file>,-<number>` (`@shell32.dll,-22019`) ဟု အမည်ပေးထားသော value များသည် shell က ၎င်း၏ ကိုယ်ပိုင် menu
များနှင့် ဖိုင်အမျိုးအစားများအတွက် cache လုပ်ထားသော resource string များဖြစ်ပြီး ပရိုဂရမ်များ မဟုတ်ပါ။ ၎င်းတို့ကို
ချန်လှပ်ထားပြီး အနှစ်ချုပ်တွင် ရေတွက်ထားသည်။ Windows Vista နှင့် နောက်ပိုင်းတွင် `NTUSER.DAT` တွင် MuiCache မရှိပါ။

Profile တိုင်းအတွက် `UsrClass.dat` (နှင့် `NTUSER.DAT`) ကို ၎င်းတို့၏ transaction log များနှင့်အတူ စုဆောင်းပြီး
host နှင့် အသုံးပြုသူအမည်များအတွက် စနစ်၏ `SYSTEM` နှင့် `SOFTWARE` hive များကို စုဆောင်းပါ။

## စိစစ်သူများ သတိပြုရန် { #analyst-cautions }

- **Profile ပိုင်ရှင်သည် process အကောင့် ဖြစ်ရန် မလိုပါ။** အခြားအကောင့်ဖြင့် စတင်ဖွင့်ခဲ့သော ပရိုဂရမ်သည် ဤအသုံးပြုသူ၏
  session တွင် window ကို ပြသနိုင်ပြီး ဤအသုံးပြုသူ၏ cache တွင် metadata ကို ချန်ထားခဲ့နိုင်သည်။
  `User` နှင့် `UserSid` သည် hive ၏ ပိုင်ရှင်ကို ဖော်ထုတ်ပေးပြီး process ၏ security context ကို မဟုတ်ပါ။
- **ပရိုဂရမ်တစ်ခုချင်းစီအတွက် အချိန်မရှိ။** `KeyLastWrittenUTC` သည် key ၏ နောက်ဆုံးရေးသားချိန် ဖြစ်သည်- ၎င်း၏
  ပရိုဂရမ်တစ်ခုခု၏ နောက်ဆုံးပြောင်းလဲမှုဖြစ်ပြီး သတ်မှတ်ထားသော ပရိုဂရမ်တစ်ခု run ခဲ့သည့်အချိန် မဟုတ်ပါ။ Prefetch၊ BAM၊
  UserAssist နှင့် event log များက အချိန်များကို ပေးသည်။
- **အသုံးပြုသူ ကိုယ်တိုင်စတင်ဖွင့်ခဲ့သော ပရိုဂရမ်များသာ မဟုတ်။** Shell ကိုယ်တိုင် load လုပ်ခဲ့သော DLL သို့မဟုတ် ပရိုဂရမ်
  (`Open with`၊ shell extension၊ installer ၏ child အတွက်) လည်း ပေါ်လာနိုင်သည်၊ ဥပမာ `explorerframe.dll` သို့မဟုတ်
  `shell32.dll`။
- **အမည်သည် ဖိုင်မှ လာသည်။** `FriendlyAppName` နှင့် `AppCompany` သည် ဖိုင် run ခဲ့သည့်အချိန်တွင် ၎င်း၏ version
  resource များက ဖော်ပြခဲ့သည့်အရာ ဖြစ်သည်။ Malware သည် မည်သည့် ဖော်ပြချက်နှင့် ကုမ္ပဏီကိုမဆို ဟန်ဆောင်ဖော်ပြနိုင်သည်။
  အမည်ပြောင်းထားသော tool သည် ၎င်း၏ မူရင်းဖော်ပြချက်ကို ဆက်ထိန်းထားပြီး ၎င်းက tool ကို ဖော်ထုတ်ပေးနိုင်သည်
  (friendly name မှာ အဝေးမှ ဝင်ရောက်သည့် tool ဖြစ်နေသော `svchost.exe` ဟု အမည်ပေးထားသည့် ဖိုင်)။
- **Path တစ်ခုလျှင် entry တစ်ခု။** တူညီသော ပရိုဂရမ်ကို ထပ်မံ run ခြင်းက entry အသစ် မထည့်ပါ။ အခြား path ရှိ
  မိတ္တူတစ်ခုက ထည့်သည်။ ဖိုင်ကို ဖျက်ပြီးနောက်တွင်လည်း ပရိုဂရမ်၏ entry ကျန်ရှိနိုင်သည်။
- **Path တစ်ခုအတွက် ပုံစံနှစ်မျိုးလုံး။** Path တစ်ခုတွင် ပုံစံနှစ်မျိုးလုံးဖြင့် friendly name ရှိသည့်အခါ (upgrade
  လုပ်ထားသော profile) တစ်ခုစီသည် ၎င်း၏ ကိုယ်ပိုင် record ကို ရရှိသောကြောင့် မည်သည့် value မျှ မဆုံးရှုံးပါ။
- **ပျက်စီးနေသော value များ။** String မဟုတ်သော၊ မှန်ကန်သော UTF-16 မဟုတ်သော သို့မဟုတ် အပြည့်အစုံ မဖတ်နိုင်သော value
  သည် ၎င်း၏ byte များ (`FriendlyAppNameRaw`, `AppCompanyRaw`) ကို ထိန်းသိမ်းထားပြီး `Problems` တွင် အစီရင်ခံသည်။
  မမှန်ကန်သော UTF-16 သည် ပြသထားသော စာသားတွင် အစားထိုးစာလုံးများကိုလည်း ဖြစ်ပေါ်စေနိုင်သည်။ မူရင်း byte များအတွက်
  raw column ကို အသုံးပြုပါ။

## Juntei ဖြင့် ခွဲခြမ်းစိတ်ဖြာခြင်း { #parse-with-juntei }

```text
Usage: juntei parse-registry muicache <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find UsrClass.dat and NTUSER.DAT hives
  -f, --file <FILE>      Parse one UsrClass.dat or NTUSER.DAT
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry muicache -d evidence/ -o muicache.csv
juntei parse-registry muicache -f UsrClass.dat -t jsonl \
  | jq 'select(.Path | test("\\\\(Users|ProgramData|Temp)\\\\"; "i"))'
```

အနှစ်ချုပ်တွင် Windows နှင့် Program Files ဖိုင်တွဲများ အပြင်ဘက်ရှိ ပရိုဂရမ်များ (အသုံးပြုသူ ဖိုင်တွဲများ၊ အခြား
drive များ၊ network share များ) ကို ၎င်းတို့၏ အသုံးပြုသူများနှင့်အတူ ဖော်ပြပြီး ချန်လှပ်ထားသော resource string
အရေအတွက်ကိုလည်း ဖော်ပြသည်။ အခြားအမည်များရှိသော Program Files ဖိုင်တွဲများ (အခြားဘာသာစကား Windows များတွင်) ကိုလည်း
ဖော်ပြသည်။ Relative path များနှင့် environment-variable ဖိုင်တွဲမှ ထွက်သွားသော path များကိုလည်း ဆက်လက်ဖော်ပြသည်။
သာမန် drive path များကို စစ်ဆေးသူ၏ ဖိုင်စနစ်ကို မဝင်ရောက်ဘဲ `.` နှင့် `..` အစိတ်အပိုင်းများကို ဖြေရှင်းပြီးနောက် စစ်ဆေးသည်။

## ထုတ်ယူမှု field များ { #output-fields }

| Field | အဓိပ္ပာယ်ဖော်ပုံ |
|---|---|
| `Host`, `User`, `UserSID`, `SourcePath` | Host၊ profile ဖိုင်တွဲ၏ အမည်၊ အသုံးပြုသူ၏ SID (`UsrClass.dat` ၏ root key က ၎င်းကို အမည်ပေးသည်၊ မဟုတ်ပါက စနစ်၏ `ProfileList`) နှင့် hive။ |
| `HiveType` | `UsrClass.dat` သို့မဟုတ် `NTUSER.DAT`။ |
| `Path` | ပရိုဂရမ်၏ path- `.FriendlyAppName` သို့မဟုတ် `.AppCompany` မပါသော value ၏ အမည်။ |
| `FriendlyAppName` | `<path>.FriendlyAppName` သို့မဟုတ် `<path>` မှ ပရိုဂရမ်၏ ဖော်ပြချက်။ |
| `AppCompany` | `<path>.AppCompany` မှ ပရိုဂရမ်၏ ကုမ္ပဏီ (Windows 8 နှင့် နောက်ပိုင်း)။ |
| `KeyPath`, `KeyLastWrittenUTC` | MuiCache key နှင့် ၎င်း၏ နောက်ဆုံးရေးသားချိန်- ၎င်း၏ ပရိုဂရမ်တစ်ခုခု၏ နောက်ဆုံးပြောင်းလဲမှု။ |
| `FriendlyAppNameRaw`, `AppCompanyRaw` | သန့်ရှင်းစွာ decode မလုပ်နိုင်သော စာသားဖြစ်သည့်အခါ value ၏ byte များ (hex)။ |
| `TransactionLogStatus`, `LogsApplied` | Registry ပြန်လည်ရယူမှု အခြေအနေ- log များက ပရိုဂရမ်၏ value တစ်ခုကို ရေးသားခဲ့သည့်အခါ `replayed`။ |
| `Problems` | String မဟုတ်သော၊ မှန်ကန်သော UTF-16 မဟုတ်သော သို့မဟုတ် မဖတ်နိုင်သော value များ။ |

## စုံစမ်းစစ်ဆေးမှု ဗျူဟာ { #investigation-strategy }

1. အသုံးပြုသူ ရေးသားနိုင်သော နေရာများ (`\Users\`, `Downloads`, `Temp`, `ProgramData`)၊ အခြား drive များနှင့်
   network share များရှိ ပရိုဂရမ်များနှင့် စီမံခန့်ခွဲရေး၊ အဝေးမှ ဝင်ရောက်ရေးနှင့် archive လုပ်ရေး tool များကို ကြည့်ပါ။
2. ဖိုင်အမည်ကို ၎င်း၏ friendly name နှင့် ကုမ္ပဏီတို့နှင့် နှိုင်းယှဉ်ပါ- မကိုက်ညီမှုက အမည်ပြောင်းထားသော tool သို့မဟုတ်
   အယောင်ဆောင်ထားသော ဖိုင်ကို ဖော်ထုတ်ပေးနိုင်သည်။
3. Prefetch၊ BAM နှင့် UserAssist (run ခဲ့သည့်အချိန်)၊ Amcache နှင့် ShimCache (ဖိုင်၏ hash နှင့် တည်ရှိမှု) နှင့်
   `$MFT` (ဖိုင် ရှိနေသေးခြင်း ရှိ/မရှိ) တို့နှင့် ချိတ်ဆက်စစ်ဆေးပါ။
4. ဖျက်ထားသော MuiCache value များအတွက် `UsrClass.dat` ပေါ်တွင် `parse-registry read --recover-deleted` ကို အသုံးပြုပါ။

## နောက်ထပ် အသေးစိတ်များ { #further-details }

- [Microsoft: VERSIONINFO resource](https://learn.microsoft.com/en-us/windows/win32/menurc/versioninfo-resource)
- [Cyber Triage: MuiCache စမ်းသပ်မှုများ၊ GUI session များနှင့် process-account ကန့်သတ်ချက်များ](https://www.cybertriage.com/blog/muicache-2026-guide/)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
