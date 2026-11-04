# LNK ဖြတ်လမ်းများ { #lnk-shortcuts }

## LNK ဖိုင်များ ဆိုသည်မှာ { #what-lnk-files-are }

Windows Shell Link (`.lnk`) ဖိုင်များသည် ပစ်မှတ် (target) တစ်ခုနှင့်အတူ စတင်ဖွင့်ရာတွင် သုံးသော argument များ၊
လုပ်ငန်းဆောင်ရွက်ရာ directory၊ icon၊ window အခြေအနေ၊ ပစ်မှတ် metadata၊ volume/network အချက်အလက်၊ shell namespace
item များနှင့် ရွေးချယ်နိုင်သော distributed-link-tracking identifier များကို ဖော်ပြသည်။ Windows နှင့် အပလီကေးရှင်းများက
၎င်းတို့ကို Recent Items၊ Desktop၊ Start Menu နှင့် အခြားနေရာများတွင် ဖန်တီးကြပြီး တိုက်ခိုက်သူများကလည်း
အန္တရာယ်ရှိသော ဖြတ်လမ်းများကို တိုက်ရိုက်ပေးပို့ကြသည်။

Recent ဖိုင်တွဲရှိ LNK တစ်ခုသည် ယခုမရှိတော့သော ပစ်မှတ် (removable drive သို့မဟုတ် network share အပါအဝင်) နှင့်
အသုံးပြုသူ၏ ထိတွေ့ဆောင်ရွက်မှုကို ထောက်ခံနိုင်သည်။ မည်သည့် LNK မဆို တစ်ကြိမ်မျှ နှိပ်ခြင်းမရှိဘဲ တည်ရှိနိုင်သည်။

<span class="evidence-strength">အကောင်းဆုံး ကောက်ချက်:</span> ဖြတ်လမ်းကို မှတ်တမ်းတင်ထားသော ပစ်မှတ်
အခြေအနေဖြင့် ဖန်တီး/အပ်ဒိတ်လုပ်ခဲ့သည်။ ၎င်းသည် အသုံးပြုမှုကို ထောက်ခံခြင်း ရှိ/မရှိကို ၎င်း၏ တည်နေရာနှင့်
ပတ်ဝန်းကျင်ရှိ အခြားအထောက်အထားများက ဆုံးဖြတ်ပေးသည်။

## တည်နေရာများနှင့် စုဆောင်းခြင်း { #locations-and-collection }

```text
%APPDATA%\Microsoft\Windows\Recent\*.lnk
%USERPROFILE%\Desktop\*.lnk
%APPDATA%\Microsoft\Windows\Start Menu\Programs\...
%PROGRAMDATA%\Microsoft\Windows\Start Menu\Programs\...
```

`Recent` တစ်ခုတည်းသာမက အသုံးပြုသူ profile တစ်ခုလုံးနှင့် စုဆောင်းထားသော archive/email အကြောင်းအရာများကိုပါ
ရှာဖွေပါ။ တည်နေရာပြောင်းလဲခြင်းက အဓိပ္ပာယ်ဖော်ပုံကို ပြောင်းလဲစေသောကြောင့် source ဖိုင်၏ ကိုယ်ပိုင်အချိန်များနှင့်
path ကို ထိန်းသိမ်းထားပါ။

## စိစစ်သူများ သတိပြုရန် { #analyst-cautions }

- ပစ်မှတ်၏ အချိန်များ၊ အရွယ်အစား၊ MFT reference နှင့် volume ဒေတာတို့သည် LNK အတွင်း သိမ်းထားသော snapshot များ
  ဖြစ်သည်။ ၎င်းတို့သည် ဖြတ်လမ်းဖိုင်၏ ကိုယ်ပိုင် metadata မဟုတ်ဘဲ ခေတ်မမီတော့သည်လည်း ဖြစ်နိုင်သည်။
- `MachineID`၊ MAC နှင့် tracker ဖန်တီးချိန်တို့သည် distributed-link-tracking အခြေအနေကို ဖော်ပြသည်။ အများအားဖြင့်
  link ကို ဖန်တီး/အပ်ဒိတ်လုပ်ခဲ့သော စက်ဖြစ်ပြီး—သင်တွေ့ရှိခဲ့သော စက်ဟု မဆိုလိုပါ။
- MAC နှင့်တူသော UUID node သည် virtualization၊ randomization သို့မဟုတ် network မဟုတ်သော identifier များမှ
  လာနိုင်သည်။
- `Args` က ပုံမှန်အန္တရာယ်ကင်းသော executable (`powershell.exe`, `rundll32.exe`, `mshta.exe`) ကို
  အန္တရာယ်ရှိအောင် ပြုလုပ်နိုင်သည်။
- ANSI string များအတွက် source စနစ်၏ code page လိုအပ်သည်။ ဂျပန် link အများအပြားအတွက် `--code-page 932` ကို သုံးပါ။
- ဖြတ်လမ်းကို အတုဖန်တီးနိုင်သလို ကူးယူနိုင်သည်။ Prefetch၊ Jump List၊ RecentDocs၊ event-log သို့မဟုတ် အခြား
  ထောက်ခံချက်မရှိဘဲ ၎င်း၏ field များကို လုပ်ဆောင်ခဲ့ခြင်း (execution) နှင့် တူညီသည်ဟု မယူဆပါနှင့်။

## Juntei ဖြင့် ခွဲခြမ်းစိတ်ဖြာခြင်း { #parse-with-juntei }

```text
Usage: juntei parse-file lnk <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find *.lnk
  -f, --file <FILE>      Parse one shortcut
      --code-page <CP>   ANSI code page (default 1252; Japanese commonly 932)

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-file lnk -d evidence/ -o lnk.csv
juntei parse-file lnk -d evidence/ --code-page 932 -o lnk-ja.csv
juntei parse-file lnk -f suspicious.lnk -t jsonl | jq .
```

## CSV ထုတ်ယူမှု field များ { #csv-output-fields }

| Field အုပ်စု | Field များနှင့် အဓိပ္ပာယ်ဖော်ပုံ |
|---|---|
| Source | `SourceFile`, `SourceCreated`, `SourceModified`, `SourceAccessed`: ဖြတ်လမ်း၏ path နှင့် host ဖိုင်စနစ်ရှိ အချိန်များ။ ၎င်းတို့သည် ပစ်မှတ်အချိန်များနှင့် သီးခြားဖြစ်သည်။ |
| ပစ်မှတ် အချိန်များ | `TargetCreated`, `TargetModified`, `TargetAccessed`: LNK တွင် မှတ်တမ်းတင်ထားသော snapshot များ။ အသုံးဝင်သော သမိုင်းဆိုင်ရာ သဲလွန်စများဖြစ်ပြီး လက်ရှိ ပစ်မှတ် metadata မဟုတ်ပါ။ |
| ပစ်မှတ် အခြေခံ | `FileSize`, `RelativePath`, `WorkingDirectory`, `FileAttributes`, `HeaderFlags`: မှတ်တမ်းတင်ထားသော ပစ်မှတ်/စတင်ဖွင့်မှု အခြေအနေ။ Argument များက အဓိပ္ပာယ်ကို ပြောင်းလဲစေနိုင်သည်။ |
| Volume | `DriveType`, `VolumeSerialNumber`, `VolumeLabel`: fixed/removable/network အခြေအနေကို ခွဲခြားပြီး အခြား ဖြတ်လမ်းများနှင့် ချိတ်ဆက်စစ်ဆေးရန်။ |
| Path များ | `LocalPath`, `NetworkPath`, `CommonPath`, `TargetIDAbsolutePath`: တစ်ခုနှင့်တစ်ခု ဖြည့်စွက်ပေးသော path ဖော်ပြပုံများ။ ကွဲလွဲချက်များနှင့် shell namespace path များကို ပြန်လည်သုံးသပ်ပါ။ |
| စတင်ဖွင့်မှု အပြုအမူ | `Args`, `ShowWindow`, `HotKey`, `ShimLayer`, `DarwinId`: shell က ပစ်မှတ်ကို စတင်ဖွင့်ပုံ သို့မဟုတ် ပြင်ဆင်ပုံ။ `Args` သည် အရေးကြီးဆုံးဖြစ်သည်။ |
| NTFS ကိုယ်ပိုင်အမှတ်အသား | `TargetMFTEntryNumber`, `TargetMFTSequenceNumber`: link ဖန်တီးချိန်က ပစ်မှတ် reference။ နှစ်ခုစလုံးကို `$MFT` နှင့် တိုက်ဆိုင်စစ်ဆေးပါ။ |
| Tracking | `MachineID`, `MachineMACAddress`, `TrackerCreatedOn`, `VolumeDroid`, `FileDroid`, `VolumeDroidBirth`, `FileDroidBirth`: distributed-link tracking။ ချိတ်ဆက်စစ်ဆေးရန်သာ အသုံးပြုပြီး ၎င်းတစ်ခုတည်းဖြင့် တာဝန်ရှိသူကို သတ်မှတ်ခြင်း မပြုပါနှင့်။ |
| အပို block များ | `ExtraBlocksPresent`: ရရှိနိုင်သော optional ဖွဲ့စည်းပုံများ၏ စာရင်း။ အခြား မည်သည့် field များကို ထောက်ပံ့ထားသည်ကို ရှင်းပြသည်။ |
| ဖော်ပြပုံ | `Name`, `IconLocation`, `IconIndex`, `IconEnvironment`, `EnvironmentVariables`: ပြသမှုနှင့် environment ချဲ့ထွင်ထားသော ပစ်မှတ်ဒေတာ။ တိုက်ခိုက်သူများက လှည့်စားသော အမည်/icon များကို အသုံးပြုနိုင်သည်။ |
| Shell ဖိုင်တွဲများ | `KnownFolder`, `SpecialFolder`: decode လုပ်ထားသော Windows shell ဖိုင်တွဲ identifier များ။ |
| Network | `NetworkDevice`, `NetworkProvider`: network-share/device နှင့် provider အခြေအနေ။ |
| Encoding | `ConsoleCodePage`: ရှိပါက မှတ်တမ်းတင်ထားသော console code page။ parser ၏ `--code-page` နှင့် သီးခြားဖြစ်သည်။ |
| နောက်ဆက်တွဲ ဒေတာ | `AppendedBytes`: logical အဆုံးနောက်ရှိ byte များ။ မမျှော်လင့်ထားသော ဒေတာသည် ခိုးသွင်းခြင်း (smuggling) သို့မဟုတ် ပုံစံမမှန်သော ဖိုင်များကို ညွှန်ပြနိုင်သည်။ |
| မှန်ကန်မှု စစ်ဆေးခြင်း | `Problems`: ပျက်စီးမှု၊ ဖြတ်တောက်ခံရမှု၊ မကိုက်ညီသော block များ သို့မဟုတ် decode ကန့်သတ်ချက်များ။ ပြန်လည်သုံးသပ်မှုတွင် ထည့်သွင်းပါ။ |

JSON/JSONL သည် ထို့အပြင် target-ID item တိုင်း (kind, name, GUID, FAT အချိန်များနှင့် MFT reference)၊
extra-data block အားလုံး၊ property-store property များ၊ console ဆက်တင်များနှင့် raw ဖွဲ့စည်းပုံဆိုင်ရာ ဆက်နွယ်မှုများကို
ထိန်းသိမ်းထားသည်။ သံသယဖြစ်ဖွယ် ဖြတ်လမ်းအတွက် flat CSV row တစ်ခုတည်းကိုသာ အားမကိုးဘဲ ၎င်းကို အသုံးပြုပါ။
Nested `TargetItems` အချိန်များသည် offset သိမ်းဆည်းမထားသော FAT shell-item နာရီများမှ လာသည်။
၎င်းတို့၏ legacy `Z` ဖော်ပြပုံက UTC ဖြစ်ကြောင်း သက်သေမပြပါ။ `auto-parse --timezone` က ၎င်းတို့ကို မပြောင်းလဲဘဲ ထိန်းသိမ်းထားသည်။
LNK header ၏ ပစ်မှတ် FILETIME များနှင့် tracker အချိန်များမူ ပြောင်းလဲနိုင်သော absolute timestamp များအဖြစ် ဆက်ရှိနေသည်။

## စုံစမ်းစစ်ဆေးမှု ဗျူဟာ { #investigation-strategy }

1. ပစ်မှတ် field များကို အဓိပ္ပာယ်မဖော်မီ source တည်နေရာနှင့် source အချိန်များကို စစ်ဆေးပါ။
2. ပစ်မှတ် path၊ `Args`၊ `WorkingDirectory` နှင့် environment block များမှ အမှန်တကယ် စတင်ဖွင့်မှုကို ပြန်လည်တည်ဆောက်ပါ။
3. ပစ်မှတ် MFT reference၊ volume serial၊ network path၊ droid များ၊ MachineID နှင့် MAC ကို အထောက်အထားများတစ်လျှောက် pivot လုပ်ပါ။
4. လုပ်ဆောင်ခဲ့ခြင်းအတွက် Prefetch နှင့် event log များကို စစ်ဆေးပါ။ အသုံးပြုသူ၏ ထိတွေ့ဆောင်ရွက်မှုအတွက် Jump Lists/RecentDocs ကို စစ်ဆေးပါ။
5. အန္တရာယ်ရှိသော ပူးတွဲဖိုင်များအတွက် appended ဒေတာကို ထိန်းသိမ်းပြီး LNK ကိုယ်တိုင်ကို hash လုပ်/ခွဲခြမ်းစိတ်ဖြာပါ။

## နောက်ထပ် အသေးစိတ်များ { #further-details }

- [Microsoft Open Specifications: MS-SHLLINK](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-shllink/)
- [ys-lnk-parser](https://github.com/Yamato-Security/ys-lnk-parser)
- [Sootmark shell](https://github.com/Sootmark/shell)
- [LnkParse3](https://github.com/Matmaus/LnkParse3)
- [Eric Zimmerman ၏ LECmd](https://github.com/EricZimmerman/LECmd)
