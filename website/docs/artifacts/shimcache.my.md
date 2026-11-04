# ShimCache (AppCompatCache) { #shimcache-appcompatcache }

## ShimCache ဆိုသည်မှာ { #what-shimcache-is }

Windows ၏ Application Compatibility subsystem သည် လိုက်ဖက်မှု/shim အပြုအမူကို အကဲဖြတ်နေစဉ် executable path များနှင့်
metadata ကို cache လုပ်ထားသည်။ ဖိုင်များကို အမည်ပြောင်းခြင်း၊ ဖယ်ရှားခြင်း သို့မဟုတ် removable volume နှင့်အတူ
ချိတ်ဆက်မှုဖြုတ်ခြင်း ခံရသော်လည်း cache သည် ဆက်လက်ရှိနေသည်။ ၎င်း၏ layout နှင့် ရရှိနိုင်သော field များသည် Windows XP မှ
Windows 11 အထိ ကွဲပြားသည်။

ShimCache သည် သမိုင်းဆိုင်ရာ path များကို ရှာဖွေတွေ့ရှိရန်နှင့် ပုံစံတွင် ယုံကြည်စိတ်ချရသော execution flag ပါရှိသည့်နေရာတွင်
လုပ်ဆောင်ခဲ့ခြင်း (execution) ကို ထပ်ဆောင်း ထောက်ခံရန် တန်ဖိုးရှိသည်။ Cache လုပ်ထားသော အချိန်သည် ပုံမှန်အားဖြင့်
**ဖိုင်၏ နောက်ဆုံးပြင်ဆင်ချိန်** ဖြစ်ပြီး Windows က ၎င်းကို မြင်ခဲ့သည့် သို့မဟုတ် run ခဲ့သည့်အချိန် မဟုတ်ပါ။

<span class="evidence-strength">အကောင်းဆုံး ကောက်ချက်:</span> Windows သည် လိုက်ဖက်မှု စီမံဆောင်ရွက်ရန်အတွက် path ကို
တွေ့မြင်ခဲ့သည်။ Entry တစ်ခုတည်းသည် တည်ရှိမှု/တွေ့မြင်မှု အထောက်အထားသာ ဖြစ်သည်။ `Executed` ကို ပံ့ပိုးထားသည့်နေရာတွင်သာ အသုံးပြုပါ။

## တည်နေရာနှင့် စုဆောင်းခြင်း { #location-and-collection }

Byte များသည် `SYSTEM` registry hive တွင် ရှိပြီး အများအားဖြင့် အောက်ပါနေရာတွင် ရှိသည်-

```text
ControlSet00N\Control\Session Manager\AppCompatCache\AppCompatCache
```

`SYSTEM`၊ `SYSTEM.LOG1` နှင့် `SYSTEM.LOG2` ကို စုဆောင်းပါ။ Juntei သည် control set အားလုံးကို ဖတ်ပြီး ဖြစ်နိုင်သည့်နေရာတွင်
`Select\Current` ကို ခွဲခြားသတ်မှတ်သည်။

!!! warning "နောက်ဆုံး live entry များသည် hive ထဲတွင် မရှိနိုင်ပါ"
    Windows သည် ShimCache အပ်ဒိတ်များကို memory တွင် ထိန်းထားပြီး shutdown/reboot လုပ်စဉ် flush လုပ်နိုင်သည်။ Run နေသော
    စနစ်မှ ကူးယူထားသော hive တွင် နောက်ဆုံးတွေ့မြင်မှုများ မပါဝင်နိုင်ပါ။ Cache ၏ registry last-write အချိန်သည် ၎င်းကို
    မည်သည့်အချိန်တွင် သိမ်းဆည်းခဲ့သည်ကို ဖော်ပြရာတွင် အထောက်အကူပြုသော်လည်း entry တစ်ခုချင်းစီ၏ လုပ်ဆောင်ချိန် မဟုတ်ပါ။

## စိစစ်သူများ သတိပြုရန် { #analyst-cautions }

- Cache entry တစ်ခုတည်းသည် **လုပ်ဆောင်ခဲ့ခြင်း၏ သက်သေ မဟုတ်ပါ**။
- `Executed` သည် `Yes`၊ `No` သို့မဟုတ် `NA` ဖြစ်နိုင်သည်။ ရရှိနိုင်သော အဓိပ္ပာယ်သည် ပုံစံ/ဗားရှင်းပေါ် မူတည်သည်။
- `LastModifiedTimeUTC` သည် ရည်ညွှန်းထားသော ဖိုင်နှင့် သက်ဆိုင်ပြီး တွေ့မြင်မှု သို့မဟုတ် လုပ်ဆောင်မှုနှင့် မသက်ဆိုင်ပါ။
- အစီအစဉ်သည် အသုံးဝင်သော်လည်း နေရာတိုင်းတွင် တိကျသော timeline မဟုတ်ပါ။ Control set များ/slack တစ်လျှောက် ထပ်နေသော entry များ ရှိတတ်သည်။
- `FromSlack` entry များကို active cache ၏ ပြင်ပမှ ပြန်လည်ရယူထားခြင်းဖြစ်ပြီး ထပ်ဆောင်း သတိထားရန် လိုအပ်သည်။
- အသုံးပြုနိုင်သော log များမရှိသည့် dirty `SYSTEM` တွင် လက်ရှိ record များ ပျောက်နေနိုင်သည်။ Recovery အနှစ်ချုပ်ကို ဖတ်ပါ။
- Finding များသည် သံသယဖြစ်ဖွယ် အမည်ပေးပုံ/path ပုံစံများကို ညွှန်ပြခြင်းသာဖြစ်ပြီး malware စီရင်ချက် မဟုတ်ပါ။

## Juntei ဖြင့် ခွဲခြမ်းစိတ်ဖြာခြင်း { #parse-with-juntei }

```text
Usage: juntei parse-registry shimcache <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SYSTEM hives
  -f, --file <FILE>      Parse SYSTEM or a raw AppCompatCache value
      --no-logs          Do not replay SYSTEM.LOG1/SYSTEM.LOG2

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry shimcache -f SYSTEM -o shimcache.csv
juntei parse-registry shimcache -d evidence/ -t jsonl \
  | jq 'select(.Executed == true or (.Findings | length > 0))'
```

## ထုတ်ယူမှု field များ { #output-fields }

| Field | အဓိပ္ပာယ်ဖော်ပုံ |
|---|---|
| `ControlSet` | Cache ကို ထိန်းထားသော control set နံပါတ်။ တစ်ခုကို မရွေးချယ်မီ `IsCurrent` ကို နှိုင်းယှဉ်ပါ။ |
| `CacheEntryPosition` | Decode လုပ်ထားသော cache အတွင်းရှိ နေရာ/အစီအစဉ်။ နှိုင်းရ အစီအစဉ်အတွက် အသုံးဝင်ပြီး အချိန်မဟုတ်ပါ။ |
| `Path` | Cache လုပ်ထားသော executable/package path။ အဓိက ရှာဖွေရေး pivot။ |
| `LastModifiedTimeUTC` | Entry တွင် သိမ်းထားသော ဖိုင်ပြင်ဆင်ချိန်—လုပ်ဆောင်ချိန်/တွေ့မြင်ချိန် မဟုတ်ပါ။ |
| `Executed` | ပံ့ပိုးထားသော ပုံစံ flag များအရ `Yes`၊ `No` သို့မဟုတ် `NA`။ ဗားရှင်း အခြေအနေနှင့်သာ အသုံးပြုပါ။ |
| `Duplicate` | တူညီသော entry ကို ယခင် control set သို့မဟုတ် slack တွင် တွေ့ပြီးဖြစ်သည်။ နှစ်ခါရေတွက်ခြင်းကို ကာကွယ်ရန် အထောက်အကူပြုသည်။ |
| `SourceFile` | Input hive/raw value။ Host များတစ်လျှောက် မရှိမဖြစ် လိုအပ်သည်။ |
| `FromSlack` | Active စာရင်းမှမဟုတ်ဘဲ cache-value slack မှ ပြန်လည်ရယူထားသော entry။ ယုံကြည်မှုနိမ့်ပြီး ပိုဟောင်းနိုင်သည်။ |
| `CacheFormat` | Decode လုပ်ထားသော Windows/cache layout။ Field ရရှိနိုင်မှုနှင့် အဓိပ္ပာယ်ကို ထိန်းချုပ်သည်။ |
| `CacheLastWrittenUTC` | Registry key/cache သိမ်းဆည်းချိန်။ ၎င်းသည် မျှဝေထားသော အခြေအနေဖြစ်ပြီး entry တစ်ခုချင်းစီ၏ run ချိန် မဟုတ်ပါ။ |
| `InsertFlags`, `ShimFlags` | Raw လိုက်ဖက်မှု flag များ။ `Executed` သည် ပံ့ပိုးထားသည့်နေရာတွင် Juntei ၏ အဓိပ္ပာယ်ဖော်ထားသော ရလဒ်ဖြစ်သည်။ |
| `DataSize` | ဤ entry ၏ ဒေတာအရွယ်အစား။ ဖွဲ့စည်းပုံ မှန်ကန်မှု စစ်ဆေးရန် အသုံးဝင်သည်။ |
| `Machine` | ပုံစံအသစ်များက မှတ်တမ်းတင်ထားသော architecture (`x64`, `x86`, `ARM64`)။ |
| `FileSize`, `LastUpdateTimeUTC` | ရှိပါက Windows XP အတွက်သာ cache လုပ်ထားသော metadata။ |
| `PackageName`, `PackageVersion`, `PackagePublisherID` | ပံ့ပိုးထားသော entry များရှိ packaged-application ကိုယ်ပိုင်အမှတ်အသား။ Publisher ID သည် အခြေအနေသာဖြစ်ပြီး signature သက်သေ မဟုတ်ပါ။ |
| `Findings` | နေရာရွှေ့ထားသော system binary သို့မဟုတ် သံသယဖြစ်ဖွယ် staging path ကဲ့သို့သော Juntei ၏ ပုံမှန်မဟုတ်မှု code များ။ သဲလွန်စများသာ။ |
| `RegistryPath`, `RegistryValueName` | Hive key/value source အတိအကျ။ မှန်ကန်မှု စစ်ဆေးရန် ထိန်းသိမ်းပါ။ |
| `CacheValueSize` | Registry value တစ်ခုလုံး၏ အရွယ်အစား၊ entry တစ်ခုချင်း၏ `DataSize` မဟုတ်ပါ။ |
| `IsCurrent` | ဤ entry ၏ control set သည် `SYSTEM\Select\Current` နှင့် တူညီခြင်း ရှိ/မရှိ။ ဘေးကင်းစွာ မသိရှိနိုင်ပါက ဗလာ။ |
| `SelectCurrent`, `SelectDefault`, `SelectLastKnownGood`, `SelectFailed` | အခြေအနေနှင့် recovery ခွဲခြမ်းစိတ်ဖြာမှုအတွက် raw `SYSTEM\Select` control-set နံပါတ်များ။ |

JSON/JSONL သည် ထို့အပြင် entry ဒေတာ၊ tag များ၊ packaged-app subrecord များ၊ ဖွဲ့စည်းထားသော finding များ၊
cache ပြဿနာများနှင့် recovery အသေးစိတ်များကို ထိန်းသိမ်းထားသည်။

## စုံစမ်းစစ်ဆေးမှု ဗျူဟာ { #investigation-strategy }

1. လက်ရှိ control set ကို ဦးစားပေးသော်လည်း အခြား set များနှင့် slack ကို သမိုင်းဆိုင်ရာ အထောက်အထားအဖြစ် ထိန်းသိမ်းထားပါ။
2. Path များနှင့် finding များကို ရှာဖွေပါ။ `LastModifiedTimeUTC` ကို လုပ်ဆောင်ချိန်အဖြစ် မသုံးပါနှင့်။
3. `Executed == Yes` ဖြစ်သည့်နေရာတွင် Prefetch၊ SRUM၊ event log များ သို့မဟုတ် အခြား runtime သဲလွန်စများဖြင့် ထပ်မံအတည်ပြုပါ။
4. ဖန်တီးမှု၊ ဖျက်ခြင်းနှင့် metadata သမိုင်းကြောင်းကို ဆုံးဖြတ်ရန် path များကို `$MFT`/USN နှင့် တိုက်ဆိုင်စစ်ဆေးပါ။
5. Transaction log များကို replay လုပ်ခဲ့ခြင်း ရှိ/မရှိနှင့် live cache ကို flush မလုပ်ရသေးခြင်း ဖြစ်နိုင်/မဖြစ်နိုင်ကို အစီရင်ခံပါ။

## နောက်ထပ် အသေးစိတ်များ { #further-details }

- [ys-shimcache-parser](https://github.com/Yamato-Security/ys-shimcache-parser)
- [Eric Zimmerman ၏ AppCompatCacheParser](https://github.com/EricZimmerman/AppCompatCacheParser)
- [Security Ronin shimcache-forensic](https://github.com/SecurityRonin/shimcache-forensic)
- [Mandiant: Leveraging the Application Compatibility Cache in Forensic Investigations](https://www.mandiant.com/resources/blog/leveraging-application-compatibility-cache-forensic-investigations)
- [Maxim Suhanov ၏ Windows registry ပုံစံ သတ်မှတ်ချက်](https://github.com/msuhanov/regf)
