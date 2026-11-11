# Prefetch { #prefetch }

## Prefetch ဆိုသည်မှာ { #what-prefetch-is }

Windows Prefetch ဖိုင်များသည် အပလီကေးရှင်း စတင်မှုကို ပိုမိုမြန်ဆန်စေရန် အသုံးပြုသော အချက်အလက်များကို မှတ်တမ်းတင်သည်။
ပရိုဂရမ်တစ်ခုအတွက် သီးသန့် `.pf` ဖိုင်တစ်ခုတွင် ပုံမှန်အားဖြင့် executable အမည်နှင့် hash၊ run အကြိမ်ရေ၊ မကြာသေးမီက
run ခဲ့သည့်အချိန်များ၊ volume အချက်အလက်၊ directory များနှင့် စတင်စဉ် ကိုးကားခဲ့သော ဖိုင်များကို သိမ်းဆည်းထားသည်။

ပံ့ပိုးထားသော Windows ဗားရှင်းများအတွက် မှန်ကန်သော အပလီကေးရှင်း Prefetch record တစ်ခုသည် executable run ခဲ့ကြောင်း
ပုံမှန်အားဖြင့် ခိုင်မာသော အထောက်အထား ဖြစ်သည်။ ၎င်းသည် event log မဟုတ်သော လုပ်ဆောင်ခဲ့ခြင်း (execution) အထောက်အထားများအနက်
အသုံးဝင်ဆုံးတစ်ခု ဖြစ်သည်။

<span class="evidence-strength">အကောင်းဆုံး ကောက်ချက်:</span> အမည်ဖော်ထားသော executable ကို မှတ်တမ်းတင်ထားသော
မကြာသေးမီက run ချိန်(များ)တွင် စတင်ခဲ့သည်၊ operating system ၏ အပြုအမူနှင့် parser ၏ မှန်ကန်မှုပေါ် မူတည်သည်။

## တည်နေရာနှင့် စုဆောင်းခြင်း { #location-and-collection }

```text
C:\Windows\Prefetch\*.pf
```

Juntei သည် format ဗားရှင်း 17 မှ 31 အထိ (Windows XP မှ Windows 11 အထိ) ကို compress လုပ်ထားသော Windows 8+
ဖိုင်များ အပါအဝင် ပံ့ပိုးသည်။ Executable-name hash များနှင့် အနီးရှိ record များကို နှိုင်းယှဉ်နိုင်ရန် directory တစ်ခုလုံးကို
စုဆောင်းပါ။

## စိစစ်သူများ သတိပြုရန် { #analyst-cautions }

- Windows သည် ဗားရှင်းဟောင်းများတွင် သမိုင်းဆိုင်ရာ run ချိန် တစ်ခုသာ သိမ်းဆည်းပြီး ဗားရှင်းအသစ်များတွင် ရှစ်ခုအထိ သိမ်းဆည်းသည်။
  `RunCount` သည် ရရှိနိုင်သော timestamp အရေအတွက်ထက် ကျော်လွန်နိုင်သည်။
- Prefetch ကို ပိတ်ထားခြင်း၊ ရှင်းလင်းထားခြင်း၊ ချန်လှပ်ထားခြင်း သို့မဟုတ် စနစ်/role အချို့တွင် မရရှိနိုင်ခြင်း ဖြစ်နိုင်သည်။
  မရှိခြင်းသည် ပရိုဂရမ် မ run ခဲ့ကြောင်း သက်သေ မဟုတ်ပါ။
- Load လုပ်ထားသော ဖိုင်စာရင်းများသည် စတင်မှုဆိုင်ရာ မှီခိုမှုများနှင့် ဝင်ရောက်မှုများကို ဖော်ပြသည်။ ၎င်းတို့သည် ဖော်ပြထားသော
  ဖိုင်တိုင်း အန္တရာယ်ရှိကြောင်း၊ တမင်ဖွင့်ခဲ့ကြောင်း သို့မဟုတ် အပြည့်အဝ ဖတ်ခဲ့ကြောင်း သက်သေမပြပါ။
- Prefetch header ရှိ executable အမည်သည် အရှည် ကန့်သတ်ထားသည်။ Juntei သည် အမည်တူ တင်ထားသော
  လမ်းကြောင်းများကို SCCA လမ်းကြောင်း hash ဖြင့် ခွဲခြားသည်။ ဖြစ်နိုင်သော လမ်းကြောင်း တစ်ခုထက်ပို၍
  ခွဲခြား၍ မရလျှင် header အမည်ကို ထားရှိပြီး PathAmbiguous ကို သတ်မှတ်သည်။ တင်ထားသော ဖိုင်စာရင်းကို
  ဆက်လက် ကြည့်ရှုနိုင်သည်။ hash မကိုက်ညီသော်လည်း ဖြစ်နိုင်သော လမ်းကြောင်း တစ်ခုတည်းသာ ရှိလျှင်
  ထိုလမ်းကြောင်းကို ထားရှိပြီး PathValidated ကို false အဖြစ် သတ်မှတ်သည်။ Hosting process များသည်
  command-line ဒေတာကို hash ထဲ ထည့်သွင်းနိုင်သဖြင့် အတည်မပြုရသေးသော လမ်းကြောင်းသည် မှားသည်ဟု
  အလိုအလျောက် မဆိုနိုင်ပါ။
- `.pf` source ဖိုင်၏ ဖိုင်စနစ် timestamp များသည် ထည့်သွင်းထားသော run ချိန်များနှင့် သီးခြားဖြစ်သည်။
- `Findings` သည် hunting လေ့လာတွေ့ရှိချက်များ ဖြစ်သည်။ Installer များနှင့် တရားဝင် စီမံခန့်ခွဲမှုများသည် staging path များကို
  မကြာခဏ အသုံးပြုကြသည်။

## Juntei ဖြင့် ခွဲခြမ်းစိတ်ဖြာခြင်း { #parse-with-juntei }

```text
Usage: juntei parse-file prefetch <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find *.pf files
  -f, --file <FILE>      Parse one Prefetch file

Records:
  -T, --timeline  Write one row per retained execution time, sorted by time

Output:
  -o, --output <FILE>       Save every record
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-file prefetch -d Prefetch/ -o prefetch.csv
juntei parse-file prefetch -d Prefetch/ -T -o prefetch-timeline.csv
juntei parse-file prefetch -d Prefetch/ -t jsonl \
  | jq 'select(.ExecutableName? | test("powershell|rundll32"; "i"))'
```

## စံ CSV field များ { #standard-csv-fields }

| Field | အဓိပ္ပာယ်ဖော်ပုံ |
|---|---|
| `SourceFile` | Input `.pf`။ Host များနှင့် ထပ်နေသော executable အမည်များကို ခွဲခြားရန် ၎င်းကို ထိန်းသိမ်းထားပါ။ |
| `ExecutableName` | Header ရှိ executable အမည်။ Hunting အများစုအတွက် စာလုံးအကြီးအသေး မခွဲခြားပါ။ မူရင်းစာလုံးပေါင်းကို ထိန်းသိမ်းထားပါ။ |
| `ExecutablePath` | ဖြစ်နိုင်သည့်အခါ Juntei က ဖြေရှင်းထားသော load လုပ်ထားသည့် path။ ၎င်းကို `Files` နှင့် အခြားအထောက်အထားများနှင့် အတည်ပြုပါ။ |
| `PathValidated` | ရွေးထားသော လမ်းကြောင်းသည် SCCA လမ်းကြောင်း hash နှင့် ကိုက်ညီသည်။ ဖိုင်အကြောင်းအရာ သို့မဟုတ် ယုံကြည်ရမှုကို အတည်ပြုခြင်း မဟုတ်ပါ။ |
| `PathAmbiguous` | တင်ထားသော လမ်းကြောင်းတစ်ခုထက်ပို၍ ဖြစ်နိုင်သေးသောကြောင့် `ExecutablePath` တွင် header အမည်ကို ထားရှိသည်။ `Files` နှင့် အခြားအထောက်အထားများကို စစ်ဆေးပါ။ |
| `PrefetchHash` | Windows က `.pf` ဖိုင်အမည်တွင် အသုံးပြုသော hash။ Cryptographic ဖိုင်အကြောင်းအရာ hash မဟုတ်ပါ။ |
| `FormatVersion` | Prefetch ဖွဲ့စည်းပုံ ဗားရှင်း။ ရရှိနိုင်သော field များနှင့် timestamp အရေအတွက်ကို ဆုံးဖြတ်သည်။ |
| `RunCount` | Windows က မှတ်တမ်းတင်ထားသော စုစုပေါင်း counter။ အကြိမ်ရေအတွက် အသုံးဝင်ပြီး မှတ်တမ်းအတိအကျ မဟုတ်ပါ။ |
| `LastRun` | ထိန်းသိမ်းထားသော နောက်ဆုံး execution အချိန်။ |
| `PreviousRun1` … `PreviousRun7` | ထိန်းသိမ်းထားသော ယခင် execution အချိန်များ၊ အသစ်ဆုံးမှ အဟောင်းဆီသို့။ Format/မှတ်တမ်းတွင် မပါဝင်သည့်နေရာတွင် ဗလာ။ |
| `VolumeCount` | မှတ်တမ်းတင်ထားသော source volume အရေအတွက်။ |
| `VolumeDevicePaths` | စတင်စဉ် အသုံးပြုခဲ့သော volume များအတွက် NT device path များ။ Serial များနှင့် ချိတ်ဆက်စစ်ဆေးပါ။ |
| `VolumeSerialNumbers` | Volume identifier များ။ Removable သို့မဟုတ် ပြန်လည် format လုပ်ထားသော drive အထောက်အထားများကို ချိတ်ဆက်ရန် အသုံးဝင်သည်။ |
| `VolumeCreated` | မှတ်တမ်းတင်ထားသော volume ဖန်တီးချိန်များ၊ executable ဖန်တီးချိန်များ မဟုတ်ပါ။ |
| `DirectoryCount` | မှတ်တမ်းတင်ထားသော directory string အရေအတွက်။ |
| `FileCount` | မှတ်တမ်းတင်ထားသော load လုပ်ထားသည့် ဖိုင် reference အရေအတွက်။ |
| `Findings` | နေရာပြောင်းထားသော system binary များ သို့မဟုတ် သံသယဖြစ်ဖွယ် execution path များကဲ့သို့ Juntei ၏ လေ့လာတွေ့ရှိချက်များ။ ကိုယ်တိုင် ပြန်လည်သုံးသပ်ပါ။ |
| `Files` | ရရှိနိုင်သော NTFS reference များနှင့်အတူ load လုပ်ထားသော/ကိုးကားထားသော ဖိုင်များ။ အလွန်ကောင်းမွန်သော DLL/path pivot ဖြစ်ပြီး ဖိုင်ရည်ရွယ်ချက်၏ သက်သေ မဟုတ်ပါ။ |
| `Dir` | စတင်မှုနှင့် ဆက်စပ်သော directory string များ။ Load လုပ်ထားသော ဖိုင် path တစ်ခုချင်းစီ မပြည့်စုံသည့်အခါ အသုံးဝင်သည်။ |

## Timeline CSV field များ (`-T`) { #timeline-csv-fields-t }

| Field | အဓိပ္ပာယ်ဖော်ပုံ |
|---|---|
| `RunTime` | ထိန်းသိမ်းထားသော Prefetch execution အချိန် တစ်ခု။ |
| `ExecutableName`, `ExecutablePath` | ပရိုဂရမ်၏ ကိုယ်ပိုင်အမှတ်အသား/path အခြေအနေ။ |
| `PathValidated` | ရွေးထားသော လမ်းကြောင်းသည် SCCA လမ်းကြောင်း hash နှင့် ကိုက်ညီသည်။ ဖိုင်အကြောင်းအရာ သို့မဟုတ် ယုံကြည်ရမှုကို အတည်ပြုခြင်း မဟုတ်ပါ။ |
| `PathAmbiguous` | တင်ထားသော လမ်းကြောင်းတစ်ခုထက်ပို၍ ဖြစ်နိုင်သေးသောကြောင့် `ExecutablePath` တွင် header အမည်ကို ထားရှိသည်။ `Files` နှင့် အခြားအထောက်အထားများကို စစ်ဆေးပါ။ |
| `PrefetchHash` | Executable အမည်တူသော မူကွဲ/path များကို ခွဲခြားပေးသည်။ |
| `RunCount` | Prefetch ဖိုင်မှ counter။ Timeline row တစ်ခုစီတွင် ထပ်ခါထပ်ခါ ပါဝင်သည်။ |
| `SourceFile` | အတည်ပြုရန်အတွက် source `.pf`။ |

JSON/JSONL သည် volume များ၊ directory များ၊ load လုပ်ထားသော ဖိုင်များနှင့် NTFS ဖိုင် reference များကို comma ဖြင့်
ဆက်ထားသော CSV cell များအစား ဖွဲ့စည်းပုံရှိသော array များအဖြစ် ထိန်းသိမ်းထားသည်။

## စုံစမ်းစစ်ဆေးမှု ဗျူဟာ { #investigation-strategy }

1. ထိန်းသိမ်းထားသော run များကို timeline တစ်ခုသို့ ထည့်ရန် `-T` ကို အသုံးပြုပါ။
2. Executable အမည်နှင့် Prefetch hash ပေါင်း၍ အုပ်စုဖွဲ့ပါ- တူညီသော အမည်တွင် ကွဲပြားသော path/hash များ ရှိနိုင်သည်။
3. `ExecutablePath`၊ staging path များ၊ နေရာပြောင်းထားသော system-binary တွေ့ရှိချက်များနှင့် load လုပ်ထားသော DLL များကို
   ပြန်လည်သုံးသပ်ပါ။
4. ပစ်မှတ် MFT reference များနှင့် ဖိုင် path များကို `$MFT`/USN record များနှင့် တိုက်ဆိုင်ပါ။
5. Run ချိန်များကို event log များ၊ SRUM၊ network telemetry၊ ဝန်ဆောင်မှုများ၊ task များနှင့် အသုံးပြုသူ အထောက်အထားများနှင့်
   ချိတ်ဆက်စစ်ဆေးပါ။

## နောက်ထပ် အသေးစိတ်များ { #further-details }

- [ys-prefetch-parser](https://github.com/Yamato-Security/ys-prefetch-parser)
- [ForensicRS frnsc-prefetch](https://github.com/ForensicRS/frnsc-prefetch)
- [libyal libscca](https://github.com/libyal/libscca)
- [Security Ronin prefetch-forensic](https://github.com/SecurityRonin/prefetch-forensic)
