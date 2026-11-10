# Amcache { #amcache }

## Amcache ဆိုသည်မှာ { #what-amcache-is }

Amcache သည် registry hive တစ်ခုတွင် သိမ်းဆည်းထားသော Windows application-compatibility စာရင်း (inventory) ဖြစ်သည်။
ခေတ်မီစနစ်များသည် executable ဖိုင်များကို `InventoryApplicationFile` အောက်တွင်၊ Plug-and-Play စက်ပစ္စည်းများကို
`InventoryDevicePnp` အောက်တွင် မှတ်တမ်းတင်သည်။ Windows ဗားရှင်းဟောင်းများတွင် legacy `Root\File` ဖွဲ့စည်းပုံကို အသုံးပြုသည်။

Executable စုံစမ်းစစ်ဆေးမှုအတွက် Amcache သည် မူရင်းဖိုင် ပျောက်ကွယ်သွားပြီးနောက်တွင်ပင် full path၊ SHA-1၊
ဖိုင်အရွယ်အစား၊ PE metadata၊ version အချက်အလက်၊ publisher စာသား၊ link ရက်စွဲနှင့် registry timestamp များကို
ထိန်းသိမ်းထားနိုင်သောကြောင့် တန်ဖိုးရှိသည်။ စက်ပစ္စည်း record များက ချိတ်ဆက်ခဲ့သော hardware၊ driver များ၊
ထည့်သွင်းသည့်ရက်စွဲများနှင့် identifier များကို ဖော်ထုတ်ပေးနိုင်သည်။

<span class="evidence-strength">အကောင်းဆုံး ကောက်ချက်:</span> Windows သည် မှတ်တမ်းတင်ထားသော ဖိုင် သို့မဟုတ်
စက်ပစ္စည်းကို စာရင်းသွင်းခဲ့သည်။ Executable record တစ်ခုသည် **တည်ရှိမှု၏ အထောက်အထားဖြစ်ပြီး လုပ်ဆောင်ခဲ့ခြင်း (execution) ၏ သက်သေ မဟုတ်ပါ**။

## တည်နေရာနှင့် စုဆောင်းခြင်း { #location-and-collection }

```text
C:\Windows\AppCompat\Programs\Amcache.hve
```

သက်ရောက်ခံရသော installation မှ hive ကို စုဆောင်းပါ။ `parse-registry amcache` သည် လက်ရှိတွင် primary hive ကို
ခွဲခြမ်းစိတ်ဖြာသော်လည်း ကပ်လျက်ရှိသော Amcache transaction log များကို replay **မလုပ်ပါ**။ ထို့ကြောင့် ထို log
များတွင်သာ ရှိသော record များသည် live သို့မဟုတ် မညီညွတ်သော စုဆောင်းမှုမှ ပျောက်နေနိုင်သည်။ ထိုကန့်သတ်ချက်ကို
အစီရင်ခံစာတွင် မှတ်သားပါ။

## စိစစ်သူများ သတိပြုရန် { #analyst-cautions }

- Amcache executable entry တစ်ခုက အသုံးပြုသူသည် ဖိုင်ကို စတင်ဖွင့်ခဲ့ကြောင်း သက်သေမပြပါ။
- `Publisher` သည် Windows က မှတ်တမ်းတင်ထားသော PE/version-resource metadata ဖြစ်သည်။ စုဆောင်းသည့်အချိန်တွင်
  ဖိုင်၌ မှန်ကန်သော Authenticode signature ရှိခဲ့ကြောင်း သက်သေမပြပါ။
- Hash များ မပါရှိနိုင်ပြီး legacy/modern schema များသည် တူညီသော value များကို မဖော်ပြပါ။
- `LinkDate` သည် PE header မှ လာပြီး အတုလုပ်ခြင်း၊ ကူးယူခြင်း သို့မဟုတ် reproducible build များမှ ထုတ်လုပ်ခြင်း ဖြစ်နိုင်သည်။
- Key ၏ last-write အချိန်သည် registry record ၏ အပ်ဒိတ်ကို ဖော်ပြပြီး download၊ ဖန်တီးမှု၊ ပထမဆုံး
  လုပ်ဆောင်မှု သို့မဟုတ် နောက်ဆုံးလုပ်ဆောင်မှုကို ဖော်ပြသည်ဟု မဆိုလိုပါ။
- Path များသည် removable သို့မဟုတ် ယခုမရှိတော့သော volume များကို ရည်ညွှန်းနိုင်သည်။

## Juntei ဖြင့် ခွဲခြမ်းစိတ်ဖြာခြင်း { #parse-with-juntei }

```text
Usage: juntei parse-registry amcache <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find Amcache hives
  -f, --file <FILE>      Parse one Amcache.hve

Records:
      --files    Include every inventoried executable
      --devices  Include every inventoried PnP device
  -a, --all      Include both (default for saved structured output)

Output:
  -o, --output <PATH>       Save text/JSON/JSONL or a directory of CSV tables
  -t, --output-type <TYPE>  csv, json, jsonl, or text
```

```bash
# Inspect the summary and findings
juntei parse-registry amcache -f Amcache.hve

# Save all record types as CSV tables
juntei parse-registry amcache -f Amcache.hve -a -o amcache-output

# Hunt for one SHA-1 in JSONL
juntei parse-registry amcache -f Amcache.hve -t jsonl \
  | jq 'select(.RecordType == "file" and .SHA1 == "0123456789abcdef0123456789abcdef01234567")'
```

CSV ထုတ်ယူမှုတွင် summary၊ findings၊ files နှင့် devices ဇယားများ ပါဝင်သည်။ `auto-parse` ဖြင့်ဆိုလျှင် ၎င်းတို့မှာ
ကွန်ပျူတာတစ်လုံးချင်းစီ၏ `FileExistence/` directory အောက်တွင် တိုက်ရိုက်ရှိသော
`Registry-Amcache-Summary.csv`၊ `Registry-Amcache-Findings.csv`၊ `Registry-Amcache-Files.csv` နှင့်
`Registry-Amcache-Devices.csv` ဖြစ်ပြီး ဇယားများဘေးတွင် `Registry-Amcache-logs.txt` ရှိသည်။ Hive အများအပြား
ရှိပါက extension မတိုင်မီ `-Source1`၊ `-Source2` စသည်ဖြင့် ထည့်သည်။ Manifest များရှိ `OutputFiles` တွင်
တိကျသော path များကို ဖော်ပြထားသည်။ သီးခြား CSV export များသည် ၎င်းတို့၏ မူလဇယားအမည်များနှင့် တောင်းဆိုထားသော
ထုတ်ယူမှုတည်နေရာကို ထိန်းသိမ်းထားသည်။ JSONL record များသည် `RecordType` value များဖြစ်သော `summary`၊
`finding`၊ `file` နှင့် `device` ကို အသုံးပြုသည်။

## ဖိုင် ထုတ်ယူမှု field များ { #file-output-fields }

| Field | စိစစ်သူက အသုံးပြုသင့်ပုံ |
|---|---|
| `KeyName`, `SourceKeyPath` | တိကျသော source registry record နှင့် schema အကိုင်းအခက်ကို ဖော်ထုတ်ပါ။ ပြန်လည်ထုတ်လုပ်နိုင်စေရန် ထိန်းသိမ်းထားပါ။ |
| `KeyLastWrittenUTC`, `KeyLastWrittenFiletime` | ဖတ်ရလွယ်သော ပုံစံနှင့် raw ပုံစံဖြင့် registry record အပ်ဒိတ်အချိန်။ ၎င်းကို လုပ်ဆောင်ချိန်ဟု အမည်မတပ်ပါနှင့်။ |
| `SourceVolumeKeyLastWrittenUTC`, `SourceVolumeKeyLastWrittenFiletime` | Legacy schema များရှိ parent volume-key အချိန်။ နောက်ခံအခြေအနေအတွက်သာဖြစ်ပြီး ဖိုင်လုပ်ဆောင်ချက် အချိန် မဟုတ်ပါ။ |
| `FileID` | Amcache က မှတ်တမ်းတင်ထားသော ဖိုင် identifier။ အတိအကျ ထိန်းသိမ်းထားသည်။ ၎င်းသည် အမြဲ SHA-1 ဖြစ်သည်ဟု မယူဆပါနှင့်။ |
| `SHA1`, `SHA256` | Defender၊ threat intelligence၊ EDR နှင့် ဖိုင် repository များသို့ pivot လုပ်ရန် အားကောင်းသော သဲလွန်စများ။ Null ဆိုသည်မှာ မှတ်တမ်းမတင်ထားခြင်း ဖြစ်သည်။ |
| `Name`, `OriginalFileName`, `Description` | မှတ်တမ်းတင်ထားသော PE/version အမည်များ။ အယောင်ဆောင်ခြင်း (masquerading) ကို ရှာဖွေရန် အမှန်တကယ် path နှင့် နှိုင်းယှဉ်ပါ။ Metadata ကို တိုက်ခိုက်သူက ထိန်းချုပ်နိုင်သည်။ |
| `FullPath` | အသုံးဝင်ဆုံး တည်နေရာ pivot။ စာလုံးအကြီးအသေးကို ဂရုတစိုက် normalize လုပ်သော်လည်း မူရင်းအထောက်အထား string ကို ထိန်းသိမ်းထားပါ။ |
| `LongPathHash` | Windows ၏ path နှင့်ဆိုင်သော identifier။ Record များကို တိုက်ဆိုင်ရန် အသုံးဝင်ပြီး content hash မဟုတ်ပါ။ |
| `ProgramID` | ရရှိနိုင်ပါက ဖိုင်ကို အပလီကေးရှင်း inventory record နှင့် ချိတ်ဆက်ပေးသည်။ |
| `Publisher` | မှတ်တမ်းတင်ထားသော publisher စာသား။ Signature စစ်ဆေးခြင်းအဖြစ် မဟုတ်ဘဲ metadata အဖြစ် သဘောထားပါ။ |
| `Version`, `BinFileVersion`, `ProductName`, `ProductVersion`, `BinProductVersion` | Version-resource အခြေအနေ။ ကွဲလွဲချက်များ သို့မဟုတ် မဖြစ်နိုင်သော value များက masquerading ခွဲခြမ်းစိတ်ဖြာမှုကို ထောက်ခံနိုင်သည်။ |
| `BinaryType` | မှတ်တမ်းတင်ထားသော architecture/အမျိုးအစား အခြေအနေ။ ဖိုင်ကို ပြန်လည်ရရှိပါက PE ခွဲခြမ်းစိတ်ဖြာမှုနှင့် နှိုင်းယှဉ်ပါ။ |
| `LinkDate` | မှတ်တမ်းတင်ထားသည့်အတိုင်း PE compile/link timestamp။ သံသယထားပြီး ထောက်ခံချက်ဖြင့်သာ အသုံးဝင်သည်။ |
| `Size` | မှတ်တမ်းတင်ထားသော အရွယ်အစား။ `$MFT`၊ Defender content သို့မဟုတ် ပြန်လည်ရရှိသော ဖိုင်နှင့် နှိုင်းယှဉ်ပါ။ |
| `Language` | PE language identifier။ အားနည်းသော နောက်ခံသဲလွန်စဖြစ်ပြီး တာဝန်ရှိသူ သတ်မှတ်ခြင်း (attribution) မဟုတ်ပါ။ |
| `IsPE-File`, `IsOS-Component` | Windows က မှတ်တမ်းတင်ထားသော အမျိုးအစားခွဲခြားမှု flag များ။ False value က အန္တရာယ်ကင်းကြောင်း သက်သေမပြပါ။ |
| `USN` | Amcache က မှတ်တမ်းတင်ထားသော NTFS update sequence value။ ကျန်ရှိနေသော USN record များသို့ pivot လုပ်ရာတွင် အထောက်အကူပြုနိုင်သည်။ |
| `APPX-PackageFullName`, `APPX-PackageRelativeID` | Package ပြုလုပ်ထားသော အပလီကေးရှင်း/component များကို ဖိုင်နှင့် ဆက်စပ်ပေးသည်။ |
| `RegistryValues` | အမျိုးအစား၊ decode လုပ်ထားသော value နှင့် raw hex ပါဝင်သော မူရင်း registry value အားလုံး။ Typed field မရှိသည့်အခါ သို့မဟုတ် အနာဂတ်/legacy field များအတွက် အသုံးပြုပါ။ |

## စက်ပစ္စည်း ထုတ်ယူမှု field များ { #device-output-fields }

| Field အုပ်စု | Field များနှင့် အဓိပ္ပာယ်ဖော်ပုံ |
|---|---|
| Source | `KeyName`, `SourceKeyPath`, `KeyLastWrittenUTC`, `KeyLastWrittenFiletime`: တိကျသော စက်ပစ္စည်း record နှင့် registry အပ်ဒိတ်အချိန်။ |
| ကိုယ်ပိုင်အမှတ်အသား | `Description`, `BusDescription`, `Manufacturer`, `Model`: လူဖတ်ရန် စက်ပစ္စည်းဖော်ပြချက်များ။ Vendor string များသည် ယေဘုယျဖြစ်နိုင်သည်။ |
| Hardware identifier များ | `HWID`, `CompID`, `Class`, `ClassGUID`, `ContainerID`, `DeviceInterfaceClasses`: စက်ပစ္စည်းမျိုးနွယ်နှင့် instance အတွက် အားအကောင်းဆုံး pivot များ။ |
| အခြေအနေ | `DeviceState`, `ProblemCode`, `ParentID`, `MatchingID`, `Enumerator`: Windows enumeration နှင့် ဆက်နွယ်မှု အခြေအနေ။ |
| Driver | `DriverID`, `DriverName`, `DriverPackageStrongName`, `DriverVerDate`, `DriverVerVersion`, `Provider`, `Service`, `INF`: driver package နှင့် service ဆက်နွယ်မှုများ။ ထည့်သွင်းထားသော ဖိုင်များနှင့် service များနှင့် ချိတ်ဆက်စစ်ဆေးပါ။ |
| ထည့်သွင်းခြင်း | `InstallDate`, `FirstInstallDate`, `InstallState`, `ExtendedInfs`: မှတ်တမ်းတင်ထားသော ထည့်သွင်းမှု lifecycle။ ရက်စွဲမရှိခြင်းက စက်ပစ္စည်းကို တစ်ကြိမ်မျှ မချိတ်ဆက်ခဲ့ကြောင်း သက်သေမပြပါ။ |
| Filter များနှင့် stack | `LowerClassFilters`, `UpperClassFilters`, `LowerFilters`, `UpperFilters`, `StackID`, `LocationPaths`: အဆင့်မြင့် driver-stack အခြေအနေ။ မမျှော်လင့်ထားသော filter driver များကို ပြန်လည်သုံးသပ်သင့်သည်။ |
| Raw ထိန်းသိမ်းခြင်း | `RegistryValues`: မှန်ကန်မှုစစ်ဆေးရန်နှင့် ပံ့ပိုးမထားသော field များအတွက် ပြည့်စုံသော typed/raw source value များ။ |

CSV ရှိ array နှင့် raw-value column များသည် `JSON` suffix ကို အသုံးပြုသည် (ဥပမာ `HWID-JSON` နှင့်
`RegistryValuesJSON`)။ ၎င်းတို့၏ cell များတွင် JSON ပါဝင်ပြီး JSON/JSONL တွင်မူ မူလ array များကို ထိန်းသိမ်းထားသည်။

## စုံစမ်းစစ်ဆေးမှု ဗျူဟာ { #investigation-strategy }

1. `FullPath`၊ `SHA1`၊ `Name` နှင့် `USN` ပေါ်တွင် pivot လုပ်ပါ။
2. လုပ်ဆောင်ခဲ့ခြင်းကို ထောက်ခံရန် ကိုက်ညီသော Prefetch ကို ရှာဖွေပါ။
3. ဖိုင်ဖန်တီးမှု၊ အမည်ပြောင်းမှု၊ ဖျက်မှုနှင့် အရွယ်အစားကို အတည်ပြုရန် `$MFT` နှင့် USN Journal ကို အသုံးပြုပါ။
4. Publisher/version metadata ကို သီးခြားပြန်လည်ရရှိထားသော PE နှင့် signature စစ်ဆေးမှုတို့နှင့် နှိုင်းယှဉ်ပါ။
5. ရှာဖွေတွေ့ရှိမှု သို့မဟုတ် ဖယ်ရှားမှုအတွက် Defender quarantine နှင့် event log များကို စစ်ဆေးပါ။

## နောက်ထပ် အသေးစိတ်များ { #further-details }

- [Juntei ၏ ys-amcache-parser ထုတ်ယူမှု schema](https://github.com/Yamato-Security/ys-amcache-parser/blob/main/docs/output.md)
- [ys-amcache-parser](https://github.com/Yamato-Security/ys-amcache-parser)
- [Eric Zimmerman ၏ AmcacheParser](https://github.com/EricZimmerman/AmcacheParser)
- [Security Ronin amcache-forensic](https://github.com/SecurityRonin/amcache-forensic)
- [Regipy Amcache plugin](https://github.com/mkorman90/regipy)
