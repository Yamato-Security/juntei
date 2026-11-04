# UserAssist { #userassist }

## UserAssist ဆိုသည်မှာ { #what-userassist-is }

Windows Explorer သည် အသုံးပြုသူက Explorer shell မှ စတင်သော ပရိုဂရမ်များနှင့် ဖြတ်လမ်းများကို ရေတွက်သည်- Start menu၊
taskbar၊ desktop နှင့် ဖိုင်တွဲများအတွင်း double-click လုပ်ခြင်းများ။ ၎င်းသည် အရေအတွက်များကို အသုံးပြုသူ၏ `NTUSER.DAT`
တွင် ထိန်းသိမ်းထားသည်။ Item တစ်ခုစီတွင်-

- run အကြိမ်ရေ၊
- Windows 7 နှင့် နောက်ပိုင်းတွင် focus ရရှိခဲ့သော အကြိမ်ရေနှင့် ကြာချိန် (millisecond ဖြင့်)၊
- ၎င်း နောက်ဆုံး run ခဲ့သည့်အချိန် ပါဝင်သည်။

တန်ဖိုးအမည်များကို ROT13 ဖြင့် encode လုပ်ထားသည်။ ပရိုဂရမ်တစ်ခု၏ အမည်သည် ပုံမှန်အားဖြင့် known folder တစ်ခု၏ GUID
ဖြင့် စတင်သည် (`{1AC14E77-…}` သည် `System32` ဖြစ်သည်)။

UserAssist သည် အသုံးပြုသူတစ်ဦးက interactive စတင်ခဲ့သည့်အရာကို ပြသပြီး ၎င်းကို အခြား execution အထောက်အထား အများစုက
အသုံးပြုသူနှင့် မချိတ်ဆက်ပေးပါ။

<span class="evidence-strength">အကောင်းဆုံး ကောက်ချက်:</span> အသုံးပြုသူသည် ဤပရိုဂရမ် သို့မဟုတ် ဖြတ်လမ်းကို
Explorer မှတစ်ဆင့် ဤအကြိမ်ရေအတိုင်း စတင်ခဲ့ပြီး နောက်ဆုံး ဤအချိန်တွင် စတင်ခဲ့သည်။ အခြားနည်းဖြင့် (command line၊
ဝန်ဆောင်မှု၊ scheduled task၊ အခြားပရိုဂရမ်) စတင်ခဲ့သော ပရိုဂရမ်များကို မရေတွက်ပါ။

## တည်နေရာ { #location }

```text
C:\Users\<user>\NTUSER.DAT
Software\Microsoft\Windows\CurrentVersion\Explorer\UserAssist\{GUID}\Count
```

| GUID | ပါဝင်သည့်အရာ |
|---|---|
| `{CEBFF5CD-ACE2-4F4F-9178-9926F41749EA}` | ပရိုဂရမ်များ (`executable`)၊ Windows 7 နှင့် နောက်ပိုင်း |
| `{F4E57C4B-2036-45F0-A9AB-443BCFE33D9F}` | ဖြတ်လမ်းများ (`shortcut`)၊ Windows 7 နှင့် နောက်ပိုင်း |
| `{75048700-EF1F-11D0-9888-006097DEACF9}` | ပရိုဂရမ်များနှင့် ဖြတ်လမ်းများ (`active_desktop`)၊ Windows 2000 မှ Vista အထိ |
| `{5E6AB780-7743-11CF-A12B-00AA004AE837}` | Internet toolbar (`internet_toolbar`)၊ Windows 2000 မှ Vista အထိ |

အခြား GUID များကို `other` အဖြစ် ဖော်ပြသည်။ Profile တိုင်းအတွက် `NTUSER.DAT` ကို ၎င်း၏ transaction log များနှင့်အတူ
စုဆောင်းပြီး အသုံးပြုသူ၏ SID ကို ပေးသော စနစ်၏ `SOFTWARE` hive ကိုလည်း စုဆောင်းပါ။

## စိစစ်သူများ သတိပြုရန် { #analyst-cautions }

- **Profile အသစ်များကို ကနဦးဒေတာ ထည့်ထားသည်။** Windows 7 နှင့် နောက်ပိုင်းတွင် profile အသစ်တစ်ခုသည် ပုံသေ item များ
  (Getting Started, Calculator, Paint, Sticky Notes, …) ဖြင့် စတင်သည်။ ၎င်းတို့၏ run အကြိမ်ရေများသည် 14 မှ
  လျော့ချရေတွက်ပြီး ၎င်းတို့အားလုံးသည် profile ဖန်တီးခဲ့ချိန်ခန့်တွင် last-run အချိန်တစ်ခုတည်းကို မျှဝေကြသည်။ ၎င်းတို့သည်
  အသုံးပြုသူ လှုပ်ရှားမှု မဟုတ်ပါ။
- **ဗားရှင်း 3 သည် 5 မှ စတင်ရေတွက်သည်** (Windows 2000 မှ Vista အထိ)။ သိမ်းဆည်းထားသော အရေအတွက်သည် 5 မှ
  စတင်သောကြောင့် ၎င်းမှ 5 ကို နုတ်သည်။ 5 အောက် သိမ်းဆည်းထားသော အရေအတွက်ကို `Problems` တွင် ဖော်ပြသည်။
- **Path များသည် known folder ၏ ပုံသေ path ကို အသုံးပြုသည်** (`%windir%\system32`, `%ProgramFiles%`,
  `%APPDATA%\…`)။ အခြားနေရာသို့ redirect လုပ်ထားသော ဖိုင်တွဲသည် ၎င်း၏ ပုံသေ path ကိုသာ ပြသနေဆဲ ဖြစ်သည်။ App ID များ
  (`Microsoft.Windows.Explorer`, `Microsoft.WindowsNotepad_8wekyb3d8bbwe!App`) နှင့် Control Panel item များ
  (`::{…}`) တွင် path မရှိပါ။
- **ခြေရာခံခြင်းကို ပိတ်နိုင်သည်**- `Explorer\Advanced` `Start_TrackProgs` = 0၊ `Policies\Explorer`
  `NoInstrumentation` = 1 သို့မဟုတ် `UserAssist\Settings` `NoLog` = 1 ဖြင့်။ အနှစ်ချုပ်သည် ၎င်းကို ပိတ်ထားသော hive
  များကို ဖော်ပြသည်။ လှုပ်ရှားနေသော profile ပေါ်တွင် UserAssist ဗလာဖြစ်နေခြင်း သို့မဟုတ် မရှိခြင်းသည် anti-forensics ကို
  ဆိုလိုနိုင်သည်။
- **`NoEncrypt` = 1** သည် Windows ကို ROT13 မပါဘဲ အမည်များ ရေးစေသည်။ ထိုကဲ့သို့သော အမည်များသည် မှားယွင်းစွာ decode
  ဖြစ်သောကြောင့် ထို hive ၏ item တိုင်းက `Problems` တွင် (မှတ်တမ်းများ stdout သို့ သွားသည့်အခါ stderr တွင်လည်း)
  ထိုသို့ ဖော်ပြသည်။
- **Last-run အချိန်နှင့် အရေအတွက်များကို item အလိုက် ထိန်းသိမ်းထားသည်၊** သို့သော် key ၏ last-write အချိန်
  (`KeyLastWrittenUTC`) သည် item တစ်ခုခုကို နောက်ဆုံးအပ်ဒိတ်လုပ်ခဲ့သည့်အချိန်သာ ဖြစ်သည်။
- **Focus သာ ရရှိခဲ့သော item များအတွက် run အကြိမ်ရေ 0 ဖြစ်နိုင်သည်။** Focus အကြိမ်ရေနှင့် အချိန်များသည် ဗားရှင်း 5
  (Windows 7 နှင့် နောက်ပိုင်း) တွင်သာ ရှိသည်။

## Juntei ဖြင့် ခွဲခြမ်းစိတ်ဖြာခြင်း { #parse-with-juntei }

```text
Usage: juntei parse-registry user-assist <INPUT> [OPTIONS]

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
juntei parse-registry user-assist -d evidence/ -o userassist.csv
juntei parse-registry user-assist -f NTUSER.DAT -t jsonl \
  | jq 'select(.Path? // "" | test("\\\\(Users|ProgramData|Temp)\\\\"; "i"))'
```

အနှစ်ချုပ်သည် နောက်ဆုံး run ခဲ့သော item များကို (အသစ်ဆုံးကို ပထမ) ၎င်းတို့၏ အသုံးပြုသူများနှင့်အတူ ဖော်ပြပြီး UserAssist
ကို ပိတ်ထားသော hive များကိုလည်း ဖော်ပြသည်။

## ထုတ်ယူမှု field များ { #output-fields }

| Field | အဓိပ္ပာယ်ဖော်ပုံ |
|---|---|
| `Host`, `User`, `UserSID`, `SourcePath` | Host၊ profile ဖိုင်တွဲ၏ အမည်၊ စနစ်၏ `ProfileList` မှ အသုံးပြုသူ၏ SID (စနစ်၏ `SOFTWARE` hive ဖြင့်) နှင့် hive။ |
| `GUID`, `Kind` | `{GUID}` key နှင့် ၎င်းတွင် ပါဝင်သည့်အရာ (`executable`, `shortcut`, `active_desktop`, `internet_toolbar`, `other`)။ |
| `NameRot13`, `Name` | သိမ်းဆည်းထားသည့်အတိုင်းနှင့် decode လုပ်ထားသော တန်ဖိုး၏ အမည်။ |
| `Path` | Known folder ၏ GUID ကို ၎င်း၏ ပုံသေ path ဖြင့် အစားထိုးထားသော path အဖြစ် decode လုပ်ထားသည့် အမည်။ App ID များနှင့် shell item များအတွက် ဗလာ။ |
| `RunCount` | Explorer မှ စတင်ခဲ့သော အကြိမ်ရေ (ဗားရှင်း 3- သိမ်းဆည်းထားသော အရေအတွက် အနုတ် 5)။ |
| `FocusCount`, `FocusTimeMs` | Focus ရရှိခဲ့သော အကြိမ်ရေနှင့် millisecond ဖြင့် ကြာချိန် (ဗားရှင်း 5)။ |
| `LastRunUTC` | ၎င်းကို နောက်ဆုံး စတင်ခဲ့ချိန်။ Windows က 0 သိမ်းဆည်းခဲ့သည့်အခါ သို့မဟုတ် သိမ်းဆည်းထားသော အချိန်သည် 9999 ကို ကျော်လွန်သည့်အခါ ဗလာ (ထိုအခါ `Problems` က ၎င်းကို ပေးပြီး `DataRaw` က byte များကို ထိန်းသိမ်းထားသည်)။ |
| `Format`, `SessionID` | ဒေတာ၏ layout (5: 72 byte, 3: 16 byte) နှင့် ၎င်းကို နောက်ဆုံးရေးသားခဲ့သော session နံပါတ်။ |
| `KeyLastWrittenUTC` | `Count` key ၏ နောက်ဆုံးရေးသားချိန်- ၎င်း၏ item တစ်ခုခုကို နောက်ဆုံးအပ်ဒိတ်လုပ်ခဲ့ချိန်။ |
| `DataRaw` | Decode မလုပ်နိုင်သည့်အခါ တန်ဖိုး၏ byte များ (hex)။ |
| `TransactionLogStatus`, `LogsApplied` | Registry ပြန်လည်ရယူမှု အခြေအနေ။ |
| `Problems` | မသိသော အရွယ်အစားရှိ ဒေတာ၊ key ၏ `Version` နှင့် မကိုက်ညီသော အရွယ်အစား၊ 5 အောက် ဗားရှင်း 3 အရေအတွက်၊ 9999 ကို ကျော်လွန်သော last run၊ `NoEncrypt`။ |

## စုံစမ်းစစ်ဆေးမှု ဗျူဟာ { #investigation-strategy }

1. `LastRunUTC` အလိုက် စီပြီး ကနဦးထည့်ထားသော item များကို ဘေးဖယ်ထားပါ- ၎င်းတို့သည် profile ဖန်တီးခဲ့ချိန်ခန့်တွင်
   အချိန်တစ်ခုတည်းကို မျှဝေကြသည်။
2. အသုံးပြုသူ ရေးသားနိုင်သော နေရာများ (`%USERPROFILE%`, `Downloads`, `Temp`, `ProgramData`)၊ removable
   drive များနှင့် network path များရှိ ပရိုဂရမ်များကိုလည်းကောင်း၊ စီမံခန့်ခွဲရေးနှင့် အဝေးမှ ဝင်ရောက်ရေး tool များကိုလည်းကောင်း
   ရှာဖွေပါ။
3. Prefetch (run ချိန်များပါသော execution)၊ Amcache နှင့် ShimCache (တည်ရှိမှု)၊ Jump Lists နှင့် LNK ဖိုင်များ
   (၎င်းတို့ဖြင့် ဖွင့်ခဲ့သည့်အရာ) နှင့် ပရိုဂရမ်ဖိုင်များအတွက် `$MFT` တို့နှင့် ချိတ်ဆက်စစ်ဆေးပါ။
4. ခြေရာခံခြင်း ပိတ်ထားသော hive များနှင့် UserAssist ဗလာဖြစ်နေသော profile များကို ရှင်းလင်းဖျက်ပစ်ခြင်းအတွက် စစ်ဆေးပါ။
5. ဖျက်ပစ်ထားသော UserAssist တန်ဖိုးများအတွက် `parse-registry read --recover-deleted` ကို အသုံးပြုပါ။

## နောက်ထပ် အသေးစိတ်များ { #further-details }

- [Microsoft: KNOWNFOLDERID](https://learn.microsoft.com/en-us/windows/win32/shell/knownfolderid)
- [libyal winreg-kb: User Assist key များ](https://github.com/libyal/winreg-kb/blob/main/documentation/User%20Assist%20keys.asciidoc)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
