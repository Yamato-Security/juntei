# SRUM { #srum }

## SRUM ဆိုသည်မှာ { #what-srum-is }

System Resource Usage Monitor (SRUM) သည် application များနှင့် service များ၏ resource အသုံးပြုမှုကို ESE database
တစ်ခုတွင် သိမ်းဆည်းပါသည်။ Windows ဗားရှင်းပေါ် မူတည်၍ ၎င်းသည် ကွန်ရက် byte များ၊ interface/profile identifier များ၊
ချိတ်ဆက်မှုများ၊ CPU အချိန်၊ foreground/background လှုပ်ရှားမှု၊ disk လှုပ်ရှားမှု၊ စွမ်းအင်အသုံးပြုမှု၊ push notification များနှင့်
application timeline ဒေတာတို့ကို မှတ်တမ်းတင်ပါသည်။ Entry များကို `SruDbIdMapTable` မှတစ်ဆင့် application ID များနှင့်
အသုံးပြုသူ SID များနှင့် ချိတ်ဆက်ထားပါသည်။

Executable တစ်ခုကို ဖျက်ထားသည့်အခါ SRUM သည် အထူးအသုံးဝင်ပါသည်။ ၎င်း၏ application ID၊ အသုံးပြုသူနှင့် resource-use
row များသည် ဖိုင်ထက် ကြာရှည်စွာ ကျန်ရှိနိုင်ပါသည်။ ကွန်ရက် စုစုပေါင်းများသည် exfiltration ယူဆချက်ကို ထောက်ခံနိုင်သော်လည်း
SRUM သည် packet capture မဟုတ်ဘဲ ယေဘုယျအားဖြင့် remote endpoint တိုင်း သို့မဟုတ် လွှဲပြောင်းခဲ့သော ဖိုင်တိုင်းကို ခွဲခြားမသတ်မှတ်ပါ။

<span class="evidence-strength">အကောင်းဆုံး ကောက်ချက်:</span> Windows သည် မှတ်တမ်းတင်ထားသော resource အသုံးပြုမှုကို
table ၏ တိုင်းတာမှုကာလအတွင်း application/account ထံ သတ်မှတ်ခဲ့ပါသည်။

## တည်နေရာနှင့် စုဆောင်းခြင်း { #location-and-collection }

```text
C:\Windows\System32\sru\SRUDB.dat
C:\Windows\System32\config\SOFTWARE   (context for account and Wi-Fi names)
```

SRUM သည် Windows 8 နှင့် နောက်ပိုင်းတွင် ရှိပါသည်။ `SRUDB.dat` ကို ကိုက်ညီမှုရှိစွာ ကူးယူပါ၊ snapshot မှ သို့မဟုတ်
forensic collector ဖြင့် ကူးယူခြင်းက အကောင်းဆုံး ဖြစ်ပါသည်။ `SOFTWARE` နှင့် `sru` ဖိုင်တွဲရှိ ESE transaction/log ဖိုင်များကိုလည်း
စုဆောင်းပါ။

!!! warning "Dirty database များ"
    Juntei သည် dirty `SRUDB.dat` ကို စုဆောင်းထားသည့်အတိုင်း ဖတ်ပြီး ၎င်းကို အစီရင်ခံပါသည်။ လက်ရှိတွင်
    `SRU*.log` ကို replay မလုပ်သောကြောင့် ထို log များတွင်သာ ရှိသော ပြောင်းလဲမှုများ ပျောက်နေပါသည်။ Dirty database ကို
    ပြည့်စုံသည်ဟု တိတ်တဆိတ် မဖော်ပြပါနှင့်။

## စိစစ်သူများ သတိပြုရန် { #analyst-cautions }

- Row များစွာသည် အချိန်အပိုင်းအခြားအလိုက် စုစည်းမှုကို ကိုယ်စားပြုပြီး များသောအားဖြင့် နာရီအလိုက် နယ်နိမိတ်များ ဝန်းကျင်တွင် ဖြစ်ပါသည်—တိကျသော လုပ်ဆောင်ချိန် မဟုတ်ပါ။
- Byte စုစုပေါင်းများသည် ၎င်းတို့ချည်းသက်သက် အကြောင်းအရာ၊ ရည်ရွယ်ချက်၊ protocol သို့မဟုတ် ဦးတည်ရာကို ခွဲခြားမသတ်မှတ်ပါ။
- Application identity သည် path၊ service၊ packaged app သို့မဟုတ် object ဖြစ်နိုင်ပါသည်။ `AppType` ကို စစ်ဆေးပါ။
- အသုံးပြုသူနှင့် Wi-Fi အမည်များသည် ကိုက်ညီသော `SOFTWARE` hive မှ ဖြည့်စွက်ထားခြင်း ဖြစ်ပါသည်။ အမည်များ မရှိခြင်းက
  အခြေခံ ID မရှိကြောင်း မဆိုလိုပါ။
- `null`၊ သုည၊ မရှိသော column များ၊ မပံ့ပိုးသော type များနှင့် ပျက်စီးနေသော value များသည် ကွဲပြားသော အခြေအနေများ ဖြစ်ပါသည်။
- Provider schema များသည် Windows ဗားရှင်းအလိုက် ကွဲပြားပါသည်။ မသိသော table များနှင့် မူလ column အမည်များကို ထိန်းသိမ်းပါ။
- နောက်ခံ Windows service များသည် ကြီးမားသော resource စုစုပေါင်းများကို တရားဝင် ထုတ်ပေးပါသည်။

## Juntei ဖြင့် ခွဲခြမ်းစိတ်ဖြာခြင်း { #parse-with-juntei }

```text
Usage: juntei parse-ese srum <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SRUDB.dat
  -f, --file <FILE>      Parse one SRUDB.dat
  -r, --software <FILE>  Matching SOFTWARE hive for names

Output:
  -o, --output <PATH>        Directory of CSV tables, or JSON/JSONL file
  -t, --output-type <TYPE>   csv, json, or jsonl
      --include-value-state  Add per-value read state and raw bytes to JSON/JSONL
```

```bash
juntei parse-ese srum -f SRUDB.dat -r SOFTWARE -o srum-csv
juntei parse-ese srum -f SRUDB.dat -t jsonl \
  | jq 'select(.Provider == "NetworkUsages" and (.BytesSent // 0) > 100000000)'
juntei parse-ese srum -f SRUDB.dat -t jsonl --include-value-state \
  | jq '{Provider, Timestamp, ValueState, Raw}'
```

## Output table များ { #output-tables }

CSV သည် `SruDbIdMapTable` အတွက် `SRUM-IdentityMap.csv` နှင့် provider တစ်ခုလျှင် table တစ်ခုကို ဖန်တီးပါသည်။ သိရှိထားသော output အမည်များတွင် အောက်ပါတို့ ပါဝင်ပါသည်။

| Provider/table | စုံစမ်းစစ်ဆေးရေး ရည်ရွယ်ချက် |
|---|---|
| `NetworkUsages` | Application၊ အသုံးပြုသူ၊ interface နှင့် profile တစ်ခုချင်းစီအလိုက် ပို့/လက်ခံ byte များ။ |
| `NetworkConnections` | ချိတ်ဆက်မှု အရေအတွက်/ကြာချိန် ပုံစံ လှုပ်ရှားမှုနှင့် ချိတ်ဆက်မှု စတင်ချိန်များ။ |
| `AppResourceUseInfo` | Foreground/background CPU၊ disk၊ ကွန်ရက်နှင့် ဆက်စပ် application အသုံးပြုမှု။ |
| `AppTimelineProvider` | End-time နှင့် usage-state field များပါသော application timeline လှုပ်ရှားမှု။ |
| `EnergyUsage` | ရေတို နှင့် ရေရှည် စွမ်းအင်/provider တိုင်းတာမှုများ။ `IsLt` က ၎င်းတို့ကို ခွဲခြားပါသည်။ |
| `PushNotifications` | Push-notification လှုပ်ရှားမှု။ |
| `VfuProvider` | `vfuprov` provider။ ၎င်း၏ အသေးစိတ် ရည်ရွယ်ချက်ကို ကောင်းစွာ စာတမ်းမပြုထားသဖြင့် ၎င်း၏ field များကို သတိထား၍ အဓိပ္ပာယ်ဖော်ပါ။ |
| `EnergyEstimation`, `TaggedEnergy` | ဗားရှင်းအလိုက် စွမ်းအင်နှင့် အသုံးပြုမှု provider များ။ |
| `ProcessorTime`, `SizeInBytes`, `TotalUsed`, `NetworkBytes` | Windows Server provider table များ။ |
| မသိသော table အမည် | Juntei သည် မသိသော provider များကို ၎င်းတို့၏ GUID/table အမည်အောက်တွင် ထိန်းသိမ်းပြီး catalog column တိုင်းကို ထုတ်ပေးပါသည်။ |

SRUM သည် catalog-driven ဖြစ်ပြီး Windows က provider column များကို ထပ်ထည့်သောကြောင့် provider-specific field များ၏
ပုံသေ အကန့်အသတ်ရှိ စာရင်းကို ရည်ရွယ်ချက်ရှိရှိ မထားပါ။ Juntei သည် **database တွင် ရှိသော column တိုင်းကို Juntei ၏
PascalCase field label များဖြင့်** ထုတ်ပေးပါသည်။ အောက်ပါ field များသည် တည်ငြိမ်/ဘုံ output contract ဖြစ်ပါသည်။

## အမျိုးအစားခွဲထားသော auto-parse output { #categorized-auto-parse-output }

`auto-parse` သည် SRUM table တစ်ခုစီကို ၎င်း၏ စုံစမ်းစစ်ဆေးရေး အမျိုးအစားထဲသို့ တိုက်ရိုက် ထားပါသည်။ CSV၊ JSON နှင့် JSONL
တို့သည် တူညီသော `SRUM-` ဖိုင်အမည် prefix ကို အသုံးပြုပြီး ဝန်းရံထားသော `SRUM/` ဖိုင်တွဲ မရှိပါ။

| အမျိုးအစား | CSV ဖိုင်များ |
|---|---|
| `NetworkActivity` | `SRUM-NetworkConnections.csv`, `SRUM-NetworkUsages.csv`, `SRUM-NetworkBytes.csv`, `SRUM-PushNotifications.csv` |
| `ProcessExecution` | `SRUM-AppResourceUseInfo.csv`, `SRUM-AppTimelineProvider.csv`, `SRUM-ProcessorTime.csv` |
| `SystemInformation` | `SRUM-EnergyUsage.csv`, `SRUM-EnergyEstimation.csv`, `SRUM-TaggedEnergy.csv`, `SRUM-SizeInBytes.csv`, `SRUM-TotalUsed.csv`, `SRUM-IdentityMap.csv`, `SRUM-VfuProvider.csv` နှင့် မသိသော provider table များ |

Capture တွင် ရှိသော table များကိုသာ export လုပ်ပါသည်။ ကွန်ရက် counter များနှင့် notification record များကို ၎င်းတို့၏
အဓိက စုံစမ်းစစ်ဆေးရေး အသုံးပြုမှုအလိုက် ထားပါသည်။ ၎င်းသည် row တိုင်းက remote ချိတ်ဆက်မှုကို ဖော်ပြသည်ဟု ဆိုလိုခြင်း မဟုတ်ပါ။
Identity map သည် အမျိုးအစား အများအပြားကို ထောက်ပံ့ပါသည်။ VFU နှင့် မသိသော provider များသည် ၎င်းတို့ ရှိနေခြင်း တစ်ခုတည်းဖြင့်
execution ကို အတည်မပြုနိုင်သောကြောင့် `SystemInformation` တွင် ကျန်ရှိပါသည်။

`SystemInformation/SRUM-logs.txt` တွင် parser diagnostic များနှင့် မူလမှ နောက်ဆုံး output သို့ mapping ပါဝင်ပါသည်။
Manifest နှစ်ခုလုံးသည် `OutputFiles` တွင် တကယ့် table path များကို ဖော်ပြပါသည်။ SRUM task ၏ `Output`
သည် computer ဖိုင်တွဲ ဖြစ်ပြီး `Category` သည် ၎င်း၏ မျှဝေထားသော log/metadata တည်နေရာကို ခွဲခြားသတ်မှတ်ပါသည်။

Database အများအပြား ရှိလျှင် ဖိုင်အမည်များသည် extension မတိုင်မီ `-Source1`၊ `-Source2` စသည်တို့ကို ရရှိပါသည်။
Log သည် မူလ capture alias များကို ထိန်းသိမ်းပြီး JSON/JSONL row များသည် ၎င်းတို့၏ `SourceFile`၊
`Table`၊ `Provider`၊ `Comp` နှင့် `Artifact` provenance ကို ထိန်းထားပါသည်။ မည်သည့် table ကိုမျှ အမျိုးအစား အများအပြားထဲသို့
မကူးယူပါ။ ပေါင်းစပ်ထားသော `-j` SIEM timeline တွင် ထိန်းထားသော SRUM record တိုင်းကို တစ်ကြိမ်စီ ထည့်သွင်းပါသည်။

သီးခြား `parse-ese srum` သည် တောင်းဆိုထားသော output တည်နေရာကို ထိန်းထားပါသည်။ ၎င်း၏ CSV table ဖိုင်များတွင်လည်း
`SRUM-` prefix ပါရှိပြီး တိကျစွာ သတ်မှတ်ထားသော JSON/JSONL output ဖိုင်အမည်များကို analyst က ရွေးချယ်သည့်အတိုင်း ထားပါသည်။

## ဘုံ output field များ { #common-output-fields }

| Field | အဓိပ္ပာယ်ဖော်ချက် |
|---|---|
| `SourceFile` | Input database။ JSON/JSONL တွင် ရှိပြီး host များအကြား အသုံးဝင်ပါသည်။ |
| `Table` | မူလ ESE table အမည်/GUID။ ဗားရှင်းများ သို့မဟုတ် မသိသော provider များကို နှိုင်းယှဉ်သည့်အခါ ထိန်းသိမ်းပါ။ |
| `Provider` | ဖတ်ရလွယ်သော provider/output အမည်၊ သို့မဟုတ် မသိလျှင် မူလ table အမည်။ |
| `ID` | Row ၏ `AutoIncId`။ database row identifier ဖြစ်ပြီး process ID မဟုတ်ပါ။ |
| `Timestamp` | SRUM က ကိန်းဂဏန်းများကို မှတ်တမ်းတင်/စုစည်းခဲ့သည့်အချိန်၊ millisecond သို့ ပတ်ထားသည်။ များသောအားဖြင့် တိကျသော event အချိန် မဟုတ်ပါ။ |
| `EXE-Info` | `AppID` မှ ဖြေရှင်းထားသော application/service/package identity။ အဓိက ပရိုဂရမ် pivot။ |
| `EXE-InfoDescription` | ပေါင်းစပ်ထားသော SRUM application အမည်များမှ ထုတ်ယူထားသော ဖော်ပြချက်။ Context သာ ဖြစ်ပါသည်။ |
| `EXE-Timestamp` | ရှိလျှင် ပေါင်းစပ် application identity ထဲတွင် ထည့်သွင်းထားသော အချိန်။ row ၏ အသုံးပြုချိန် မဟုတ်ပါ။ |
| `AppType` | `NormalApp`, `Service`, `ModernApp`, သို့မဟုတ် `Object`။ `EXE-Info` ကို မည်သို့ အဓိပ္ပာယ်ဖော်သင့်သည်ကို ထိန်းချုပ်ပါသည်။ |
| `SID` | `UserID` မှ ဖြေရှင်းထားသော account SID။ ခိုင်မာသော အသုံးပြုသူ/service identity pivot။ |
| `User` | `SOFTWARE` မှ ဖြည့်စွက်ထားသော ဖတ်ရလွယ်သည့် profile အမည်။ `SID` မှန်ကန်နေစဉ် null ဖြစ်နိုင်ပါသည်။ |
| `UserID`, `AppID` | `SruDbIdMapTable` ထဲသို့ raw ဂဏန်း reference များ။ ဖြေရှင်းမရသော သို့မဟုတ် ထပ်နေသော အမည်များအတွက် ထိန်းသိမ်းပါ။ |
| `IsLt` | ရေရှည် EnergyUsage table အတွက် `true`၊ ရေတိုအတွက် `false`။ |
| `InterfaceLUID` | Provider row များမှ raw ကွန်ရက် interface LUID။ |
| `InterfaceType` | Decode လုပ်ထားသော interface class (ဥပမာ `IF_TYPE_IEEE80211`)။ class သာ ဖြစ်ပြီး တစ်မူထူးခြားသော adapter မဟုတ်ပါ။ |
| `L2ProfileID` | ဂဏန်း ကွန်ရက် profile reference။ |
| `ProfileName` | `SOFTWARE` မှ ဖြည့်စွက်ထားသော Wi-Fi/profile အမည်။ အရေးကြီးနိုင်ပြီး လက်ရှိ ချိတ်ဆက်မှု၏ သက်သေ မဟုတ်ပါ။ |
| `ConnectStartTime`, `EventTimestamp`, `EndTime`, `StartTime` | UTC သို့ decode လုပ်ထားသော သိရှိထားသည့် provider FILETIME column များ။ ၎င်းတို့၏ တိကျသော အဓိပ္ပာယ်သည် provider နှင့် သက်ဆိုင်ပြီး universal execution အချိန် မဟုတ်ပါ။ |
| Provider-specific column များ | `BytesSent`၊ `BytesReceived` (database column `BytesRecvd`)၊ CPU/disk counter များ၊ ကြာချိန်များ၊ အရေအတွက်များ၊ flag များ သို့မဟုတ် ခန့်မှန်းချက်များကဲ့သို့ export လုပ်ထားသော provider column များ။ ယူနစ်များကို provider အလိုက် အဓိပ္ပာယ်ဖော်ပြီး schema တူညီသော row များအတွင်း နှိုင်းယှဉ်ပါ။ |

## Diagnostic field များ (`--include-value-state`) { #diagnostic-fields-include-value-state }

| Field | အဓိပ္ပာယ်ဖော်ချက် |
|---|---|
| `ValueState` | Database field မှ `parsed`, `null`, `absent`, `unsupported`, သို့မဟုတ် `damaged` သို့ map။ ၎င်းသည် အထောက်အထား မရှိခြင်းကို parser မအောင်မြင်ခြင်းမှ ခွဲခြားပေးပါသည်။ |
| `Raw` | Byte ရှိသော value များအတွက် hexadecimal ဖြင့် မူလ byte များ။ မှန်ကန်မှု စစ်ဆေးရန်နှင့် အနာဂတ် decode လုပ်ရန် အသုံးပြုပါ။ |

`SRUM-IdentityMap.csv` (`SruDbIdMapTable` မှ) တွင် `IdIndex`၊ `IdType` နှင့် `IdBlob` ပါဝင်ပါသည်: provider row များက
အသုံးပြုသော raw index၊ identity အမျိုးအစားနှင့် decode လုပ်ထားသော ပရိုဂရမ်/SID/object စာသား ဖြစ်ပါသည်။

## စုံစမ်းစစ်ဆေးမှု ဗျူဟာ { #investigation-strategy }

1. ပရိုဂရမ်ကို `EXE-Info`၊ `AppType` နှင့် `AppID` ဖြင့် ခွဲခြားသတ်မှတ်ပါ။ `Id` ကို PID နှင့် မရောထွေးပါနှင့်။
2. ကွန်ရက် row များကို ပရိုဂရမ်၊ အသုံးပြုသူ၊ profile၊ interface နှင့် အချိန်ကာလအလိုက် အုပ်စုဖွဲ့ပါ။
3. ပို့/လက်ခံ byte များကို ပုံမှန် baseline များနှင့် အခြား host များနှင့် နှိုင်းယှဉ်ပါ။
4. သံသယဖြစ်ဖွယ် ကာလများကို Prefetch၊ event log များ၊ DNS/proxy/firewall ဒေတာနှင့် ဖိုင်ဝင်ရောက်မှုတို့နှင့် ဆက်စပ်စစ်ဆေးပါ။
5. Dirty-database အခြေအနေနှင့် transaction-log replay မရှိခြင်းကို collection ကန့်သတ်ချက်အဖြစ် အစီရင်ခံပါ။

## နောက်ထပ် အသေးစိတ်များ { #further-details }

- [ys-srum-parser](https://github.com/Yamato-Security/ys-srum-parser)
- [Eric Zimmerman ၏ SrumECmd](https://github.com/EricZimmerman/Srum)
- [dissect.esedb](https://github.com/fox-it/dissect.esedb)
- [libyal libesedb](https://github.com/libyal/libesedb)
- [Microsoft: Extensible Storage Engine](https://learn.microsoft.com/en-us/windows/win32/extensible-storage-engine/extensible-storage-engine)
