# Jump Lists { #jump-lists }

## Jump Lists ဆိုသည်မှာ { #what-jump-lists-are }

Jump Lists သည် taskbar နှင့် Start-menu ရှိ အပလီကေးရှင်း entry များမှ ပြသသော menu များ၏ နောက်ကွယ်ရှိ ဒေတာ ဖြစ်သည်။
Automatic Jump Lists သည် အပလီကေးရှင်းတစ်ခုက အသုံးပြုခဲ့သော item များ၏ Windows စီမံသော မှတ်တမ်းများ ဖြစ်သည်။ Custom
Jump Lists သည် အပလီကေးရှင်းက သတ်မှတ်သော category များနှင့် task များ ဖြစ်သည်။ နှစ်မျိုးစလုံးတွင် ပစ်မှတ် path များ၊ အချိန်များ၊
volume/network အခြေအနေ၊ MFT reference များနှင့် စတင်ဖွင့်ရာတွင် သုံးသော argument များပါဝင်သည့် LNK ဖွဲ့စည်းပုံများကို
ထည့်သွင်းထားနိုင်သည်။

Automatic မှတ်တမ်းများသည် အပလီကေးရှင်းတစ်ခုက ဖွင့်ခဲ့သော ဖိုင် သို့မဟုတ် ဖိုင်တွဲများ၊ နောက်ဆုံးအသုံးပြုမှု အစီအစဉ်/အချိန်၊
ထိတွေ့ဆောင်ရွက်မှု အကြိမ်ရေနှင့် ပင်ထိုးထားမှု အခြေအနေကို ဖော်ပြနိုင်သည်။ Custom စာရင်းများသည် “မကြာသေးမီက ပိတ်ခဲ့သော”
စာမျက်နှာများ သို့မဟုတ် အပလီကေးရှင်း task များကို—အန္တရာယ်ရှိသော ညွှန်ကြားချက် argument များအပါအဝင်—ဖော်ပြနိုင်သည်။

<span class="evidence-strength">အကောင်းဆုံး ကောက်ချက်:</span> ပိုင်ဆိုင်သော အပလီကေးရှင်းက item/task ကို
၎င်း၏ Jump List တွင် မှတ်တမ်းတင်ခဲ့ သို့မဟုတ် ဖော်ပြခဲ့သည်။ အပြုအမူသည် အပလီကေးရှင်းအလိုက် ကွဲပြားသည်- အမှန်တကယ်
လုပ်ဆောင်ခြင်း/ဝင်ရောက်ခြင်းကို အတည်ပြုပါ။

## တည်နေရာများနှင့် စုဆောင်းခြင်း { #locations-and-collection }

```text
%APPDATA%\Microsoft\Windows\Recent\AutomaticDestinations\*.automaticDestinations-ms
%APPDATA%\Microsoft\Windows\Recent\CustomDestinations\*.customDestinations-ms
```

Hexadecimal ဖိုင်အမည် ရှေ့ဆက်သည် AppID ဖြစ်သည်။ Directory နှစ်ခုစလုံးကို စုဆောင်းပြီး အသုံးပြုသူ profile path ကို
ထိန်းသိမ်းထားပါ။ Automatic စာရင်းများသည် `DestList` stream နှင့် နံပါတ်တပ်ထားသော LNK stream များပါဝင်သည့် OLE
Compound File container များ ဖြစ်သည်။ Custom စာရင်းများသည် ၎င်းတို့၏ ကိုယ်ပိုင် category/link format ကို အသုံးပြုသည်။

## စိစစ်သူများ သတိပြုရန် { #analyst-cautions }

- AppID ဖော်ပြချက်များသည် သိရှိထားသော mapping မှ လာသည်။ မသိသော AppID တစ်ခုသည် ၎င်းကိုယ်တိုင် သံသယဖြစ်ဖွယ် မဟုတ်ပါ။
- `LastModified` / last-used field များသည် DestList entry တစ်ခုနှင့် သက်ဆိုင်သည်။ ထည့်သွင်းထားသော ပစ်မှတ်အချိန်များသည် မတူပါ။
- ထိတွေ့ဆောင်ရွက်မှု အကြိမ်ရေများသည် ပံ့ပိုးထားသော ဗားရှင်းများတွင်သာ ရှိပြီး ဖွင့်ခြင်း အကြိမ်ရေ အပြည့်အစုံ မဟုတ်ပါ။
- ပင်ထိုးခြင်းသည် item များကို ပုံမှန် မကြာသေးမီ အစီအစဉ်ထက်ကျော်၍ ထိန်းသိမ်းထားသည်။ ရှင်းလင်းခြင်း သို့မဟုတ် အပလီကေးရှင်း
  ဆက်တင်များက အခြား item များကို ဖယ်ရှားနိုင်သည်။
- စာရင်းတစ်ခုတွင် ခေတ်မမီတော့သော/မချိတ်ဆက်ထားသော stream များ သို့မဟုတ် ယခင်က ပိုရှည်ခဲ့သော custom စာရင်းမှ byte များ
  ပါဝင်နိုင်သည်။
- ထည့်သွင်းထားသော LNK argument များနှင့် path များကို တိုက်ခိုက်သူက ထိန်းချုပ်နိုင်သည်။ အပလီကေးရှင်းနှင့် ဆက်စပ်မှုသည်
  အခြေအနေသာဖြစ်ပြီး ယုံကြည်မှု မဟုတ်ပါ။

## Juntei ဖြင့် ခွဲခြမ်းစိတ်ဖြာခြင်း { #parse-with-juntei }

```text
Usage: juntei parse-file jump-lists <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find automatic/custom destination files
  -f, --file <FILE>      Parse one Jump List
      --code-page <CP>   ANSI code page for embedded links

Output:
  -o, --output <PATH>       Directory of CSV tables, or JSON/JSONL file
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-file jump-lists -d evidence/ -o jump-list-csv
juntei parse-file jump-lists -f sample.automaticDestinations-ms -t jsonl | jq .
```

`auto-parse` ဖြင့် CSV ဇယားများသည် ကွန်ပျူတာတစ်လုံးစီ၏ တိုက်ရိုက်အောက်တွင်
`FileAndFolderOpening/JumpLists-AutomaticDestinations.csv` နှင့်
`FileAndFolderOpening/JumpLists-CustomDestinations.csv` ဖြစ်သည်။ မျှဝေထားသော log သည် ထို category ရှိ
`JumpLists-logs.txt` ဖြစ်သည်။ Manifest များသည် `OutputFiles` တွင် path အတိအကျကို ဖော်ပြသည်။ သီးခြား CSV export
များသည် သင်ရွေးချယ်သော directory တွင် `AutomaticDestinations.csv` နှင့် `CustomDestinations.csv` ကို ဆက်လက်
အသုံးပြုသည်။ JSON နှင့် JSONL automatic export များသည် `JumpLists.json` သို့မဟုတ် `JumpLists.jsonl` ဖိုင်တစ်ခုတည်း
ဖြစ်သည်။ မှတ်တမ်းဖွဲ့စည်းပုံ မပြောင်းလဲပါ။

## AutomaticDestinations CSV field များ { #automaticdestinations-csv-fields }

| Field အုပ်စု | Field များနှင့် အဓိပ္ပာယ်ဖော်ပုံ |
|---|---|
| Source | `SourceFile`, `SourceCreated`, `SourceModified`, `SourceAccessed`: Jump List container နှင့် ၎င်း၏ ဖိုင်စနစ် အချိန်များ။ |
| အပလီကေးရှင်း | `AppId`, `AppIdDescription`: ဖိုင်အမည်ရှိ AppID နှင့် သိရှိထားသော အပလီကေးရှင်း mapping။ မသိသော hash မှ ကိုယ်ပိုင်အမှတ်အသားကို ကောက်ချက်မချပါနှင့်။ |
| Container | `HasSps`, `DestListVersion`, `LastUsedEntryNumber`: DestList ဖွဲ့စည်းပုံ/ဗားရှင်းနှင့် header အခြေအနေ။ |
| အစီအစဉ် | `MRU`, `EntryNumber`: နောက်ဆုံးအသုံးပြုမှု နေရာနှင့် hexadecimal entry/stream identifier။ |
| အသုံးပြုမှု metadata | `CreationTime`, `LastModified`, `InteractionCount`, `PinStatus`, `PinPosition`: DestList ၏ သက်တမ်း/အသုံးပြုမှု/ပင်ထိုးမှု အခြေအနေ။ ဗားရှင်းအလိုက် အဓိပ္ပာယ်ဖော်ပါ။ |
| Host/path | `Hostname`, `MacAddress`, `Path`, `DisplayPath`: ပစ်မှတ် တည်နေရာနှင့် tracking အခြေအနေ။ Host/MAC သည် တာဝန်ရှိသူ သတ်မှတ်ရန် တစ်ခုတည်းသော အချက် မဟုတ်ပါ။ |
| Tracking ID များ | `FileBirthDroid`, `FileDroid`, `VolumeBirthDroid`, `VolumeDroid`: ရွှေ့ပြောင်းထားသော ပစ်မှတ်များနှင့် ထည့်သွင်းထားသော LNK tracking ကို ချိတ်ဆက်စစ်ဆေးရန်။ |
| မှတ်ချက်များ | `Notes`: မချိတ်ဆက်ထားသော stream များ၊ ပြန်လည်ရယူထားသော ကျန်ရှိနေသည့်အရာများ သို့မဟုတ် အခြား parser အခြေအနေကို ဖော်ပြသည်။ |
| ထည့်သွင်းထားသော LNK | `TargetCreated`, `TargetModified`, `TargetAccessed`, `FileSize`, `RelativePath`, `WorkingDirectory`, `FileAttributes`, `HeaderFlags`, `DriveType`, `VolumeSerialNumber`, `VolumeLabel`, `LocalPath`, `CommonPath`, `TargetIDAbsolutePath`, `TargetMFTEntryNumber`, `TargetMFTSequenceNumber`, `MachineID`, `MachineMACAddress`, `TrackerCreatedOn`, `ExtraBlocksPresent`, `Args`: [LNK ဖြတ်လမ်းများ](lnk.md) တွင် ဖော်ပြထားသည့်အတိုင်း အဓိပ္ပာယ်ဖော်ပါ။ |
| အပို LNK | `NetworkPath`, `Name`, `IconLocation`: ထည့်သွင်းထားသော ဖြတ်လမ်းမှ network နှင့် ဖော်ပြပုံ field များ။ |
| မှန်ကန်မှု စစ်ဆေးခြင်း | `Problems`: ပျောက်နေသော stream များ၊ ဖြတ်တောက်ခံရမှု၊ အရေအတွက် ကွဲလွဲမှု သို့မဟုတ် link ပျက်စီးမှု။ |

## CustomDestinations CSV field များ { #customdestinations-csv-fields }

Custom output သည် `SourceFile`, `SourceCreated`, `SourceModified`, `SourceAccessed`, `AppId`,
`AppIdDescription` နှင့် `EntryName` (category အမည် သို့မဟုတ် `Tasks`) ဖြင့် စတင်ပြီး အထက်တွင် ဖော်ပြထားသော
ထည့်သွင်းထားသည့် LNK field အားလုံး၊ ထို့နောက် `Notes`, `NetworkPath`, `Name`, `IconLocation` နှင့် `Problems`
တို့ ဆက်လက်ပါဝင်သည်။

JSON/JSONL သည် ထို့အပြင် DestList header၊ entry တစ်ခုစီ၏ score/checksum/property store၊
`DestListPropertyStore`၊ compound-file stream များ၊ custom category များ၊ နောက်ဆုံး category နောက်ရှိ byte များနှင့်
ထည့်သွင်းထားသော LNK တိုင်း၏ nested field များကို ထိန်းသိမ်းထားသည်။

## စုံစမ်းစစ်ဆေးမှု ဗျူဟာ { #investigation-strategy }

1. အသုံးပြုသူနှင့် source profile ကို ခွဲခြားသတ်မှတ်ပြီးနောက် AppID ကို အပလီကေးရှင်းနှင့် သတိထား၍ ချိတ်ဆက်ပါ။
2. Automatic entry များကို `LastModified` နှင့် `MRU` အလိုက် စီပါ။ ပင်ထိုးထားသော item များကို ခွဲထားပါ။
3. Path များ၊ network share များ၊ removable volume serial များ၊ MFT reference များနှင့် argument များကို ပြန်လည်သုံးသပ်ပါ။
4. `Notes` နှင့် မချိတ်ဆက်ထားသော/ကျန်ရှိနေသော link များကို စစ်ဆေးပါ—ဖျက်ပစ်ထားသော သို့မဟုတ် ယခင် item များ ထိုနေရာတွင်
   ကျန်ရှိနိုင်သည်။
5. RecentDocs၊ သီးခြား Recent LNK များ၊ Prefetch၊ SRUM၊ `$MFT` နှင့် event log များနှင့် ချိတ်ဆက်စစ်ဆေးပါ။

## နောက်ထပ် အသေးစိတ်များ { #further-details }

- [Microsoft: Compound Binary File format (MS-CFB)](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-cfb/)
- [Microsoft: Shell Link format (MS-SHLLINK)](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-shllink/)
- [ys-jump-lists-parser](https://github.com/Yamato-Security/ys-jump-lists-parser)
- [JumpList parser စမ်းသပ်/ကိုးကား project](https://github.com/AbdulRhmanAlfaifi/jumplist_parser)
