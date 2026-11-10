# Windows ဝန်ဆောင်မှုများ { #windows-services }

## ဝန်ဆောင်မှု ပြင်ဆင်မှုက မှတ်တမ်းတင်သည့်အရာ { #what-service-configuration-records }

Windows ဝန်ဆောင်မှု (service) များနှင့် kernel driver များကို `SYSTEM` control set တစ်ခုစီအောက်တွင် ပြင်ဆင်ထားသည်။
မှတ်တမ်းများတွင် executable/driver သို့မဟုတ် service DLL၊ start mode၊ service type၊ အကောင့်၊ load-order group၊
delayed start၊ error အပြုအမူနှင့် မအောင်မြင်ပြီးနောက် Windows ဆောင်ရွက်သော action များကို ဖော်ပြထားသည်။

တိုက်ခိုက်သူများသည် အမြဲတမ်းဝင်ရောက်နိုင်မှု (persistence)၊ အခွင့်အာဏာမြှင့်တင်ခြင်း၊ driver loading နှင့်
ကွန်ရက်အတွင်း ဘေးတိုက်ရွေ့လျားမှုတို့အတွက် ဝန်ဆောင်မှုများကို ဖန်တီး သို့မဟုတ် ပြင်ဆင်ကြသည်။ တရားဝင်ဆော့ဖ်ဝဲများကလည်း
ယန္တရားအတိအကျကို အသုံးပြုသောကြောင့် path များ၊ အကောင့်များ၊ signing၊ ဖန်တီးမှုအခြေအနေနှင့် runtime အထောက်အထားတို့သည်
အရေးကြီးသည်။

<span class="evidence-strength">အကောင်းဆုံး ကောက်ချက်:</span> ခွဲခြမ်းစိတ်ဖြာထားသော control set သည်
ဝန်ဆောင်မှု/driver ကို ပြထားသည့်အတိုင်း ပြင်ဆင်ထားခဲ့သည်။ ပြင်ဆင်မှုတစ်ခုတည်းဖြင့် ဝန်ဆောင်မှု အောင်မြင်စွာ
စတင်ခဲ့ကြောင်း သက်သေမပြပါ။

## တည်နေရာ { #location }

```text
SYSTEM\ControlSet00N\Services\<ServiceName>
SYSTEM\Select\Current   (which control set Windows used)
```

`SYSTEM` ကို ၎င်း၏ transaction log များနှင့်အတူ စုဆောင်းပါ။ System 7045 နှင့် Security 4697 ကဲ့သို့သော event ID
များက ထည့်သွင်းမှုကို မှတ်တမ်းတင်နိုင်ပြီး service-control နှင့် process event များက အမှန်တကယ် စတင်ခြင်း/လုပ်ဆောင်ခြင်းကို
ထောက်ခံနိုင်သည်။

## စိစစ်သူများ သတိပြုရန် { #analyst-cautions }

- `ImagePath` တွင် environment variable များ၊ quote များ၊ argument များ၊ device path များ သို့မဟုတ် space ပါသော
  quote မပါသည့် path ပါဝင်နိုင်သည်။ Normalize မလုပ်မီ ၎င်းကို အတိအကျ ထိန်းသိမ်းထားပါ။
- `ServiceDll` သည် `svchost.exe` က host လုပ်သော ဝန်ဆောင်မှုများအတွက် အရေးကြီးသည်။ `ImagePath` တစ်ခုတည်းက
  host ကိုသာ အမည်ဖော်ထားနိုင်သည်။
- `StartType=automatic` သည် ဝန်ဆောင်မှု စတင်ခဲ့ကြောင်း သို့မဟုတ် ဆက်လက်ဖွင့်ထားကြောင်း သက်သေမပြပါ။
- `ObjectName` သည် ပြင်ဆင်ထားသော အကောင့်ဖြစ်ပြီး ဝန်ဆောင်မှုကို ထည့်သွင်းခဲ့သူ အသုံးပြုသူဟု မဆိုလိုပါ။
- Driver များနှင့် user-mode ဝန်ဆောင်မှုများသည် မတူညီသော `ServiceType` flag များနှင့် path သဘောတူညီချက်များကို
  အသုံးပြုသည်။
- Failure action များသည် မအောင်မြင်မှုတစ်ခုပြီးနောက် ဝန်ဆောင်မှုအကောင့်အဖြစ် ညွှန်ကြားချက်တစ်ခုကို လုပ်ဆောင်နိုင်သည်။
  ၎င်းသည် လွတ်သွားလွယ်သော persistence မျက်နှာပြင် ဖြစ်သည်။
- အခြား control set များက ယခင်ပြင်ဆင်မှုကို ထိန်းသိမ်းထားနိုင်သည်။ `IsCurrentControlSet` ကို အသုံးပြုပါ။

## Juntei ဖြင့် ခွဲခြမ်းစိတ်ဖြာခြင်း { #parse-with-juntei }

```text
Usage: juntei parse-registry services <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SYSTEM hives
  -f, --file <FILE>      Parse one SYSTEM hive
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Filtering:
      --service <NAME>    Only this service name
      --all-control-sets  Parse every ControlSet00N

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry services -f SYSTEM -o services.csv
juntei parse-registry services -f SYSTEM --all-control-sets -t jsonl \
  | jq 'select(.FailureCommand != null or (.ImagePath? | test("temp|public"; "i")))'
```

## ထုတ်ယူမှု field များ { #output-fields }

| Field | အဓိပ္ပာယ်ဖော်ပုံ |
|---|---|
| `Host`, `SourcePath` | စနစ်၏ ကိုယ်ပိုင်အမှတ်အသားနှင့် input hive။ စုဆောင်းမှုများတစ်လျှောက် ထိန်းသိမ်းထားပါ။ |
| `ControlSet`, `IsCurrentControlSet` | Source control set နှင့် `Select\Current` က ၎င်းကို အမည်ဖော်ထားခြင်း ရှိ/မရှိ။ လက်ရှိအခြေအနေဆိုင်ရာ ကောက်ချက်များအတွက် current ကို ဦးစားပေးပါ။ |
| `ServiceName` | Registry key/အတွင်းပိုင်း ဝန်ဆောင်မှုအမည်။ Event များနှင့် Service Control Manager ညွှန်ကြားချက်များသို့ pivot လုပ်ရန် ခိုင်မာသော အမှတ်။ |
| `DisplayName`, `Description` | အသုံးပြုသူမြင်ရသော စာသား။ Resource reference များ သို့မဟုတ် တိုက်ခိုက်သူ ထိန်းချုပ်ထားသော စာသား ဖြစ်နိုင်သည်။ |
| `ImagePath` | ပြင်ဆင်ထားသည့်အတိုင်း executable/driver ညွှန်ကြားချက်။ Quote၊ argument၊ environment variable နှင့် device path များကို ခွဲခြမ်းစိတ်ဖြာပါ။ |
| `ServiceDLL` | Host ဝန်ဆောင်မှုက load လုပ်သော DLL။ Shared-process ဝန်ဆောင်မှုများအတွက် အမှန်တကယ် payload ဖြစ်လေ့ရှိသည်။ |
| `StartType`, `StartValue` | ဖတ်ရလွယ်သော/raw start mode- boot (0), system (1), automatic (2), manual (3), disabled (4)။ |
| `DelayedAutostart` | Automatic ဝန်ဆောင်မှုသည် ကနဦး boot အဆင့်ပြီးနောက် စတင်သည်။ persistence အတွက် ဆက်လက်အသုံးပြုနိုင်သည်။ |
| `ServiceType`, `TypeValue` | Decode လုပ်ထားသော/raw flag များ- kernel driver, file-system driver, own/share process, user service စသည်။ |
| `ErrorControl` | Boot/start error ကိုင်တွယ်မှု အဆင့်။ အဓိကအားဖြင့် လုပ်ငန်းဆောင်ရွက်မှုဆိုင်ရာ အခြေအနေ။ |
| `ObjectName` | ပြင်ဆင်ထားသော ဝန်ဆောင်မှုအကောင့် သို့မဟုတ် driver object အမည်။ ထည့်သွင်းသူ၏ ကိုယ်ပိုင်အမှတ်အသား မဟုတ်ပါ။ |
| `Group` | Load-order group၊ boot/system driver များအတွက် အထူးအရေးကြီးသည်။ |
| `FailureResetPeriod` | Failure count ပြန်လည်သတ်မှတ်မီ စက္ကန့်များ။ `4294967295` သည် ဘယ်တော့မှ မပြန်လည်သတ်မှတ်ဟု ဆိုလိုသည်။ |
| `FailureCommand` | `run_command` failure action အတွက် ပြင်ဆင်ထားသော ညွှန်ကြားချက်။ တန်ဖိုးမြင့်သော persistence field။ |
| `FailureActions` | Decode လုပ်ထားသော action စာရင်း အပြည့်အစုံ (`restart`, `reboot`, `run_command`, `none`) နှင့် delay များ။ |
| `FailureActionType`, `FailureActionDelayMs` | စစ်ထုတ်ရန်အတွက် ပြားချပ်ထားသော ပထမ action။ စာရင်းအပြည့်အစုံကိုလည်း စစ်ဆေးပါ။ |
| `FailureActionsRaw` | Decode မပြည့်စုံ/ပျက်စီးနေသည့်အခါ မူရင်း byte များ။ |
| `KeyLastWrittenUTC` | ဝန်ဆောင်မှု key ၏ နောက်ဆုံးပြောင်းလဲမှု၊ ထည့်သွင်းချိန်/စတင်ချိန်ဟု မဆိုလိုပါ။ |
| `TransactionLogStatus`, `LogsApplied` | Registry-replay အခြေအနေ။ |
| `Problems` | Type မကိုက်ညီမှု၊ ပုံစံမမှန်သော failure ဖွဲ့စည်းပုံ သို့မဟုတ် ပျောက်နေသော/ဖတ်မရသော field များ။ |

## စုံစမ်းစစ်ဆေးမှု ဗျူဟာ { #investigation-strategy }

1. စံမဟုတ်သော path များ၊ ရေးသားနိုင်သော directory များ၊ ပုံမှန်မဟုတ်သော အကောင့်များ၊ key အချိန်အသစ်များနှင့် failure command များကို ပြန်လည်သုံးသပ်ပါ။
2. Environment variable များကို စိစစ်သူ၏ စက်မဟုတ်ဘဲ သက်ရောက်ခံရသော host ၏ ပြင်ဆင်မှုကို အသုံးပြု၍ ချဲ့ထွင်ပါ။
3. စုဆောင်းထားပါက အမှန်တကယ် executable သို့မဟုတ် DLL ကို hash/signature စစ်ဆေးပါ။
4. ဝန်ဆောင်မှုအမည်/path ကို Event 7045/4697၊ process ဖန်တီးမှု၊ Prefetch၊ `$MFT` နှင့် USN တို့နှင့် ချိတ်ဆက်စစ်ဆေးပါ။
5. ပြောင်းလဲထားသော/ဖျက်ထားသော persistence အတွက် current နှင့် non-current control set များကို နှိုင်းယှဉ်ပါ။

## နောက်ထပ် အသေးစိတ်များ { #further-details }

- [Microsoft: Device နှင့် driver ဝန်ဆောင်မှုများအတွက် Registry tree](https://learn.microsoft.com/en-us/windows-hardware/drivers/install/hklm-system-currentcontrolset-services-registry-tree)
- [Microsoft: SERVICE_STATUS / ဝန်ဆောင်မှု type များနှင့် အခြေအနေများ](https://learn.microsoft.com/en-us/windows/win32/api/winsvc/ns-winsvc-service_status)
- [Microsoft: ChangeServiceConfig2 failure action များ](https://learn.microsoft.com/en-us/windows/win32/api/winsvc/nf-winsvc-changeserviceconfig2w)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
