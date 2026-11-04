# BAM / DAM { #bam-and-dam }

## BAM နှင့် DAM ဆိုသည်မှာ { #what-bam-and-dam-are }

Background Activity Moderator (BAM) နှင့် Desktop Activity Moderator (DAM) တို့သည် အပလီကေးရှင်းများ နောက်ခံ
(background) တွင် ပြုလုပ်နိုင်သည့်အရာများကို စီမံခန့်ခွဲသော Windows ဝန်ဆောင်မှုများ (Windows 10 1709 နှင့် နောက်ပိုင်း)
ဖြစ်သည်။ ၎င်းတို့သည် ပရိုဂရမ်တစ်ခုစီ နောက်ဆုံး run ခဲ့သည့်အချိန်ကို အသုံးပြုသူအလိုက် `SYSTEM` hive တွင် သိမ်းဆည်းထားသည်-

- အသုံးပြုသူတစ်ဦးလျှင် key တစ်ခု၊ အသုံးပြုသူ၏ SID ဖြင့် အမည်ပေးထားသည်။
- ပရိုဂရမ်တစ်ခုလျှင် တန်ဖိုးတစ်ခု- ၎င်း၏အမည်မှာ ၎င်း၏ device ပေါ်ရှိ ပရိုဂရမ်၏ path
  (`\Device\HarddiskVolume3\Windows\System32\cmd.exe`) သို့မဟုတ် packaged app ၏ ID
  (`Microsoft.WindowsCalculator_8wekyb3d8bbwe`) ဖြစ်သည်။
- တန်ဖိုး၏ ဒေတာ၏ ပထမ 8 byte သည် ၎င်း နောက်ဆုံး run ခဲ့သည့်အချိန် (FILETIME) ဖြစ်သည်။

BAM သည် အသုံးပြုသူ၏ ကိုယ်ပိုင် hive မလိုဘဲ ပရိုဂရမ်တစ်ခုကို အသုံးပြုသူနှင့် အချိန်တစ်ခုနှင့် ချိတ်ဆက်ပေးပြီး removable နှင့်
network device များမှ run ခဲ့သော ပရိုဂရမ်များကိုလည်း ဖော်ပြသည်။

<span class="evidence-strength">အကောင်းဆုံး ကောက်ချက်:</span> ဤပရိုဂရမ်၏ process တစ်ခုသည် ဤအသုံးပြုသူ၏
session တွင် run ခဲ့သည်။ ၎င်းသည် ဤအချိန်တွင် နောက်ဆုံး run ခဲ့သည်။ BAM သည် run အကြိမ်ရေကို မရေတွက်ပါ၊ ပရိုဂရမ်ကို
ဖျက်ပစ်ပြီးနောက်တွင်လည်း entry ကျန်ရှိနေနိုင်သည်။

## တည်နေရာ { #location }

```text
C:\Windows\System32\config\SYSTEM
ControlSet00N\Services\bam\State\UserSettings\<SID>   Windows 10 1809 and later
ControlSet00N\Services\bam\UserSettings\<SID>         Windows 10 1709 to 1803
ControlSet00N\Services\dam\State\UserSettings\<SID>
ControlSet00N\Services\dam\UserSettings\<SID>
```

အသုံးပြုသူတစ်ဦးစီ၏ key တွင် `Version` နှင့် `SequenceNumber` တန်ဖိုးများလည်း ရှိသည်- ၎င်းတို့သည် ပရိုဂရမ်များ
မဟုတ်သောကြောင့် ချန်လှပ်ထားသည်။ 1709 သို့မဟုတ် 1803 မှ upgrade လုပ်ထားသော စနစ်တွင် layout နှစ်မျိုးစလုံး ရှိနိုင်သည်။

`SYSTEM` ကို ၎င်း၏ transaction log များနှင့်အတူ စုဆောင်းပြီး စနစ်၏ `SAM` နှင့် `SOFTWARE` ကိုလည်း စုဆောင်းပါ- ၎င်းတို့က
အသုံးပြုသူများ၏ အမည်ကို ဖော်ပြသည်။

## စိစစ်သူများ သတိပြုရန် { #analyst-cautions }

- **ပရိုဂရမ်နှင့် အသုံးပြုသူတစ်ဦးလျှင် အချိန်တစ်ခု။** နောက်ဆုံး run ခြင်းကိုသာ ထိန်းသိမ်းထားသည်။ Prefetch၊ Amcache နှင့်
  event log များက ယခင် run ခြင်းများကို ပေးသည်။
- **မကြာသေးမီက လှုပ်ရှားမှု။** အချိန်အတော်ကြာ မ run ခဲ့သော ပရိုဂရမ်များ၏ entry များကို Windows စတင်သည့်အခါ ဖယ်ရှားသည်ဟု
  ဆိုကြသောကြောင့် BAM သည် အများအားဖြင့် မကြာသေးမီက လှုပ်ရှားမှုကို ပြသည်- ၎င်းတွင် မပါသော ပရိုဂရမ်တစ်ခုသည် ကြာမြင့်စွာက
  run ခဲ့နိုင်သေးသည်။
- **Drive letter များမဟုတ်ဘဲ device path များ။** `\Device\HarddiskVolume3` သည် Windows က နံပါတ်တပ်ခဲ့သော တတိယ volume
  ဖြစ်ပြီး `C:` မဟုတ်ပါ။ `DevicePath` နှင့် `Path` သည် device ကို ၎င်းပေါ်ရှိ path မှ ခွဲထုတ်ထားသည်။ Drive letter
  မရေးမီ volume ကို `MountedDevices`၊ `$MFT` သို့မဟုတ် အခြားအထောက်အထားများနှင့် တိုက်ဆိုင်ပါ။
- **ပရိုဂရမ်တိုင်း မဟုတ်ပါ။** BAM နှင့် DAM သည် အသုံးပြုသူများ၏ session များရှိ ပရိုဂရမ်များကို ခြေရာခံသည်။ ဝန်ဆောင်မှုများ
  အများစု မပေါ်ပါ။ `S-1-5-18` entry များသည် ပုံမှန်အားဖြင့် setup နှင့် sign-in မျက်နှာပြင်၏ process များ
  (`FirstLogonAnim.exe`) ဖြစ်ပြီး `S-1-5-90-0-<n>` entry များသည် Desktop Window Manager (`DWM-<n>`) ဖြစ်သည်။
- **Control set များ။** ပုံသေအားဖြင့် Windows နောက်ဆုံးအသုံးပြုခဲ့သော control set ကို ဖတ်သည်။ `--all-control-sets`
  သည် အားလုံးကို ဖတ်သည်- control set ဟောင်းတစ်ခုတွင် လက်ရှိ control set တွင် မရှိတော့သော entry ကျန်ရှိနိုင်သည်။
- **အသုံးပြုသူအမည်များ။** Local အကောင့်၏ အမည်သည် `SAM` (ကွန်ပျူတာ၏ SID နှင့် အကောင့်၏ RID) မှ လာသည်။ Domain
  အကောင့်၏ အမည်သည် `ProfileList` ရှိ ၎င်း၏ profile ဖိုင်တွဲမှ လာသည်။ `UserSource` က မည်သည့်ရင်းမြစ်ဖြစ်သည်ကို ဖော်ပြသည်
  (`sam`, `profile_list`, `well_known`)။ ၎င်းတို့ မရှိပါက SID ကိုသာ ပေးသည်။ `SAM` တွင် မရှိတော့သော RID
  (ဖျက်ပစ်ထားသော အကောင့်) သည် အမည်မဲ့ ကျန်ရှိနေသည်။

## Juntei ဖြင့် ခွဲခြမ်းစိတ်ဖြာခြင်း { #parse-with-juntei }

```text
Usage: juntei parse-registry bam <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SYSTEM hives
  -f, --file <FILE>      Parse one SYSTEM hive
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Filtering:
      --all-control-sets  Every control set, not only the one in use

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry bam -d evidence/ -o bam.csv
juntei parse-registry bam -f SYSTEM -t jsonl \
  | jq 'select(.Path? // "" | test("\\\\(Users|ProgramData|Temp)\\\\"; "i"))'
```

အနှစ်ချုပ်တွင် အသုံးပြုသူတစ်ဦးစီကို ၎င်း၏ SID၊ အမည်နှင့် entry အရေအတွက်တို့နှင့်အတူ ဖော်ပြပြီး နောက်ဆုံး run ခဲ့သော
ပရိုဂရမ်များကို (အသစ်ဆုံးကို ပထမ) ဖော်ပြသည်။

## ထုတ်ယူမှု field များ { #output-fields }

| Field | အဓိပ္ပာယ်ဖော်ပုံ |
|---|---|
| `Host`, `SourcePath` | Host နှင့် `SYSTEM` hive။ |
| `ControlSet`, `IsCurrentControlSet` | Entry ပါဝင်သော control set နှင့် ၎င်းသည် Windows နောက်ဆုံးအသုံးပြုခဲ့သော control set ဟုတ်/မဟုတ်။ |
| `Moderator` | `bam` သို့မဟုတ် `dam`။ |
| `UserSID`, `User`, `UserSource` | Key ကို အမည်ပေးထားသော SID၊ အသုံးပြုသူ၏ အမည်နှင့် ၎င်း၏ ရင်းမြစ် (`sam`, `profile_list`, `well_known`)။ |
| `Program` | Windows ရေးသားသည့်အတိုင်း တန်ဖိုး၏ အမည်- device path သို့မဟုတ် packaged app ၏ ID။ |
| `DevicePath`, `Path` | Device path အတွက် device (`\Device\HarddiskVolume3`) နှင့် ၎င်းပေါ်ရှိ path (`\Windows\System32\cmd.exe`)။ packaged app အတွက် ဗလာ။ |
| `LastRunUTC` | ပရိုဂရမ် နောက်ဆုံး run ခဲ့သည့်အချိန် (100 ns အထိ)။ |
| `KeyPath`, `KeyLastWrittenUTC` | အသုံးပြုသူ၏ key နှင့် ၎င်း၏ နောက်ဆုံးရေးသားချိန်- ၎င်း၏ entry တစ်ခုခုကို နောက်ဆုံးအပ်ဒိတ်လုပ်ခဲ့သည့်အချိန်။ |
| `DataRaw` | အချိန်တစ်ခု မပါဝင်သည့်အခါ (`REG_BINARY` မဟုတ်ခြင်း သို့မဟုတ် 8 byte ထက်နည်းခြင်း) တန်ဖိုး၏ byte များ (hex)။ |
| `TransactionLogStatus`, `LogsApplied` | Registry ပြန်လည်ရယူမှု အခြေအနေ။ |
| `Problems` | အချိန်မပါဝင်သော ဒေတာနှင့် ဖတ်၍မရသော တန်ဖိုးများ။ |

## စုံစမ်းစစ်ဆေးမှု ဗျူဟာ { #investigation-strategy }

1. `LastRunUTC` အလိုက် စီပြီး အသုံးပြုသူ ရေးသားနိုင်သော နေရာများ (`\Users\`, `Downloads`, `Temp`,
   `ProgramData`)၊ စနစ် volume မှလွဲ၍ အခြား device များပေါ်ရှိ ပရိုဂရမ်များနှင့် စီမံခန့်ခွဲရေးနှင့် အဝေးမှ ဝင်ရောက်ရေး
   tool များကို ကြည့်ပါ။
2. ၎င်းတို့ကို မည်သည့်အသုံးပြုသူက run ခဲ့သည်နှင့် လက်ရှိ profile နှင့် မကိုက်ညီသော အကောင့်များကို စစ်ဆေးပါ။
3. Prefetch (run အကြိမ်ရေနှင့် ယခင် run ခြင်းများ)၊ UserAssist (Explorer မှ စတင်ခြင်းများ)၊ Amcache နှင့် ShimCache
   (တည်ရှိမှု)၊ `$MFT` (ပရိုဂရမ်၏ ဖိုင်) နှင့် event log များနှင့် ချိတ်ဆက်စစ်ဆေးပါ။
4. `--all-control-sets` ကို အသုံးပြုပြီး ဖျက်ပစ်ထားသော BAM တန်ဖိုးများအတွက် `parse-registry read --recover-deleted`
   ကို အသုံးပြုပါ။

## နောက်ထပ် အသေးစိတ်များ { #further-details }

- [libyal winreg-kb: Background activity moderator (BAM)](https://github.com/libyal/winreg-kb/blob/main/docs/sources/system-keys/Background-activity-moderator.md)
- [Microsoft: Security identifier များ (well-known SID များ)](https://learn.microsoft.com/en-us/windows-server/identity/ad-ds/manage/understand-security-identifiers)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)

အပိုင်းအခြားပြင်ပရှိ last-run FILETIME များသည် `LastRunUTC` ကို ဗလာထားပြီး မူရင်းတန်ဖိုးကို `DataRaw` တွင်
ထိန်းသိမ်းကာ `Problems` တွင် ရှင်းလင်းချက် ပေးသည်။ မကြာသေးမီက လှုပ်ရှားမှု အနှစ်ချုပ်သည် host တစ်ခုစီကို ခွဲခြားဖော်ပြပြီး
သီးခြား hive/အသုံးပြုသူ လေ့လာတွေ့ရှိချက်များကို ခွဲခြားထားသည်။
