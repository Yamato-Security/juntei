# Windows Registry hive များ { #windows-registry-hives }

## Registry hive များ ဆိုသည်မှာ { #what-registry-hives-are }

Windows Registry သည် စနစ်နှင့် အသုံးပြုသူ ပြင်ဆင်မှုများကို binary hive ဖိုင်များတွင် သိမ်းဆည်းပါသည်။ Key များသည်
tree တစ်ခုကို ဖွဲ့စည်းပြီး key တစ်ခုအောက်ရှိ value များတွင် type သတ်မှတ်ထားသော ဒေတာ ပါဝင်ပါသည်။ Registry artifact များသည်
persistence၊ ထည့်သွင်းထားသော software၊ service များ၊ account များ၊ ကွန်ရက်ပြင်ဆင်မှု၊ အသုံးပြုသူ လှုပ်ရှားမှု၊ လုံခြုံရေးဆက်တင်များနှင့်
Windows အခြေအနေ၏ အစိတ်အပိုင်း အားလုံးနီးပါးကို ဖော်ပြပါသည်။

Juntei ၏ ယေဘုယျ reader သည် မည်သည့် primary hive ကိုမဆို decode လုပ်ပြီး transaction log များကို replay လုပ်ကာ ပျက်စီးမှုကို
အစီရင်ခံပြီး deleted/free-cell၊ cell-slack၊ remnant၊ pre-replay နှင့် transaction-log-only record များကို ပြန်လည်ရယူနိုင်ပါသည်။

<span class="evidence-strength">အကောင်းဆုံး ကောက်ချက်:</span> hive တွင် key/value အခြေအနေ ပါဝင်ခဲ့သည် သို့မဟုတ်
ပြန်လည်ရယူနိုင်သည့်ပုံစံဖြင့် ထိန်းသိမ်းထားခဲ့ပါသည်။ Key ၏ နောက်ဆုံးရေးသားချိန်သည် key ပြောင်းလဲခဲ့ကြောင်း ပြသသော်လည်း
မည်သည့် value ပြောင်းလဲခဲ့သည် သို့မဟုတ် အဘယ်ကြောင့်ဆိုသည်ကို မပြသပါ။

## အသုံးများသော hive များနှင့် တည်နေရာများ { #common-hives-and-locations }

| Hive | ပုံမှန် path | အထောက်အထား ဥပမာများ |
|---|---|---|
| `SYSTEM` | `C:\Windows\System32\config\SYSTEM` | Service များ၊ driver များ၊ control set များ၊ host အမည်၊ mount လုပ်ထားသော device များ၊ ShimCache |
| `SOFTWARE` | `C:\Windows\System32\config\SOFTWARE` | ထည့်သွင်းထားသော software၊ TaskCache၊ profile များ၊ Windows ပြင်ဆင်မှု |
| `SAM` | `C:\Windows\System32\config\SAM` | စက်တွင်း account များနှင့် account metadata |
| `SECURITY` | `C:\Windows\System32\config\SECURITY` | စက်တွင်း လုံခြုံရေး policy/secret metadata (ဝင်ရောက်ခွင့်အပေါ် မူတည်) |
| `DEFAULT` | `C:\Windows\System32\config\DEFAULT` | မူလ စနစ် profile ပြင်ဆင်မှု |
| `NTUSER.DAT` | `C:\Users\<user>\NTUSER.DAT` | အသုံးပြုသူ နှစ်သက်မှုများ၊ Run key များ၊ RecentDocs၊ application လှုပ်ရှားမှု |
| `UsrClass.dat` | `%LOCALAPPDATA%\Microsoft\Windows\UsrClass.dat` | Shell class များ၊ COM/အသုံးပြုသူ shell လှုပ်ရှားမှု |
| `Amcache.hve` | `C:\Windows\AppCompat\Programs\Amcache.hve` | Application inventory (အဓိပ္ပာယ်ပါ output အတွက် `parse-registry amcache` ကို အသုံးပြုပါ) |

ဘေးချင်းကပ်ရှိသော `.LOG`၊ `.LOG1` နှင့် `.LOG2` ဖိုင်များကို စုဆောင်းပါ။ Windows လည်ပတ်နေစဉ် ကူးယူထားသော hive သည်
dirty ဖြစ်နိုင်ပါသည်။ Transaction log များတွင် primary ဖိုင်တစ်ခုတည်းက မထင်ဟပ်သေးသော commit လုပ်ပြီးသည့် အခြေအနေ ပါဝင်ပါသည်။

## စိစစ်သူများ သတိပြုရန် { #analyst-cautions }

- Key ၏ နောက်ဆုံးရေးသားချိန် timestamp သည် value တစ်ခုချင်းစီ မဟုတ်ဘဲ **key** နှင့်သာ သက်ဆိုင်ပါသည်။
- Registry value များတွင် များသောအားဖြင့် လွတ်လပ်သော timestamp မရှိပါ။
- `CurrentControlSet` သည် `SYSTEM\Select` မှတစ်ဆင့် ဖြေရှင်းသော analyst/Windows alias ဖြစ်ပြီး
  စာသားအတိုင်း သိမ်းဆည်းထားသော key မဟုတ်ပါ။
- ဖျက်ထားသော record များသည် မပြည့်စုံခြင်း၊ ခေတ်မမီခြင်း၊ ပြန်လည်ရယူမှု ရင်းမြစ်များအကြား ထပ်နေခြင်း သို့မဟုတ် ထပ်ရေးခံရခြင်း ဖြစ်နိုင်ပါသည်။
- Transaction-log-only record များသည် သမိုင်းဝင်/ကြားကာလ အခြေအနေကို ပြသပါသည်။ `recovered_from` ကို ရှင်းပြပါ။
- `absent_value` ရလဒ်သည် query လုပ်ထားသော key ရှိသော်လည်း parse လုပ်ထားသော အခြေအနေတွင် ထို value မရှိကြောင်း
  ဆိုလိုပါသည်။ ထို value တစ်ခါမျှ မရှိခဲ့ကြောင်း မသက်သေပြပါ။
- String များ၊ path များနှင့် command line များသည် ပြင်ဆင်မှုများသာ ဖြစ်ပြီး ပြင်ဆင်ထားသော လုပ်ဆောင်ချက် လည်ပတ်ခဲ့ကြောင်း သက်သေ မဟုတ်ပါ။
- သက်ဆိုင်သော log များ မပါသည့် dirty hive သည် အခြေအနေကို ချန်လှပ်နိုင်သည် သို့မဟုတ် ခေတ်မမီသော အခြေအနေကို ပြသနိုင်ပါသည်။

## Juntei ဖြင့် ခွဲခြမ်းစိတ်ဖြာခြင်း { #parse-with-juntei }

```text
Usage: juntei parse-registry read <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find primary registry hives
  -f, --file <FILE>      Parse one hive
      --log1 <FILE>      Supply first transaction log explicitly
      --log2 <FILE>      Supply second transaction log explicitly
      --no-logs          Intentionally read the primary file without replay

Filtering:
  -k, --key <PATH>             Exact key (case-insensitive)
      --key-regex <REGEX>      Key-path regular expression
      --value <NAME>           Exact value name ("" is default)
      --value-regex <REGEX>    Value-name regular expression
      --data-regex <REGEX>     Rendered-data regular expression
      --timeline-start <DATE>  Key last-write lower bound
      --timeline-end <DATE>    Key last-write upper bound
      --recover-deleted        Recover deleted and transaction-history records

Output:
  -o, --output <FILE>       Save CSV/JSON/JSONL
  -t, --output-type <TYPE>  csv, json, or jsonl
      --raw-data            Include exact bytes for every value
      --qword-as-filetime   Also render REG_QWORD as FILETIME
```

```bash
# Inspect one persistence key
juntei parse-registry read -f NTUSER.DAT \
  -k 'Software\Microsoft\Windows\CurrentVersion\Run' -o run.csv

# Search allocated and recoverable values for a path/name
juntei parse-registry read -f SYSTEM --recover-deleted \
  --data-regex 'powershell|cmd\.exe|users\\public' -t jsonl

# Show raw and decoded data for forensic validation
juntei parse-registry read -f SOFTWARE --raw-data -o software.jsonl -t jsonl
```

## ထုတ်ယူမှု field များ { #output-fields }

| Field | အဓိပ္ပာယ်ဖော်ချက် |
|---|---|
| `Host` | ရရှိနိုင်သော စနစ် context မှ ရယူထားသော host အမည်။ Juntei က ဘေးကင်းစွာ ခွဲခြားမသတ်မှတ်နိုင်သည့်အခါ ဗလာ။ |
| `HiveType` | ရှာဖွေတွေ့ရှိထားသော hive အမျိုးအစား (`SYSTEM`, `SOFTWARE`, `NTUSER.DAT` စသည်)။ Path များကို အဓိပ္ပာယ်ဖော်ရန် အသုံးပြုပါ။ |
| `SourcePath` | တိကျသော input hive။ Directory mode က host/အသုံးပြုသူ များကို ပေါင်းစပ်သည့်အခါ မရှိမဖြစ် လိုအပ်ပါသည်။ |
| `RecordType` | `key`, `value`, `default_value`, သို့မဟုတ် `absent_value`။ မရှိခြင်းသည် query/အခြေအနေနှင့်သာ သက်ဆိုင်ပါသည်။ |
| `KeyPath` | Hive root အောက်ရှိ path။ `[unknown]` သည် ဖြေရှင်းမရသော ဖျက်ထားသည့် ancestry ကို အမှတ်အသားပြုပါသည်။ |
| `KeyLastWrittenUTC` | Key တစ်ခုလုံး၏ နောက်ဆုံးပြင်ဆင်ချိန်။ မည်သည့် value ပြောင်းလဲခဲ့သည်ကို ခွဲခြားမသတ်မှတ်နိုင်ပါ။ |
| `ValueName` | အမည်ပါ value။ default/အမည်မဲ့ value အတွက် ဗလာ။ |
| `ValueType` | Registry type (`REG_SZ`, `REG_DWORD`, `REG_BINARY` စသည်)။ Type သည် အဓိပ္ပာယ်ဖော်ခြင်းကို သက်ရောက်ပါသည်။ |
| `ValueData` | Type ကို သိရှိ၍ ပြသထားသော value။ Multi-string များသည် JSON တွင် list များအဖြစ် ရှိနေပါသည်။ |
| `DataSize` | Value က မှတ်တမ်းတင်ထားသော size။ raw byte များနှင့် `Problems` တို့နှင့် နှိုင်းယှဉ်ပါ။ |
| `ValueDataRaw` | Decode မအောင်မြင်သည့်အခါ သို့မဟုတ် `--raw-data` ကို အသုံးပြုသည့်အခါ တိကျသော byte များ။ မှန်ကန်မှု စစ်ဆေးရန် ထိန်းသိမ်းပါ။ |
| `ValueU64` | JavaScript precision ဆုံးရှုံးမှု မရှိဘဲ DWORD/QWORD အတွက် ကိန်းပြည့် ဂဏန်းဖော်ပြချက်။ |
| `ValueFiletimeUTC` | ရွေးချယ်နိုင်သော QWORD-as-FILETIME ပြသမှု။ Key ၏ schema က FILETIME ဟု သတ်မှတ်ထားမှသာ အဓိပ္ပာယ်ရှိပါသည်။ |
| `AllocationStatus` | `allocated`, `deleted_complete`, `deleted_partial`, `residual_value_only`, သို့မဟုတ် `transaction_log_recovered`။ ယုံကြည်မှုအဆင့်ကို ထိန်းချုပ်ပါသည်။ |
| `RecoveredFrom` | `free_cell`, `cell_slack`, `remnant_data`, `before_replay`, သို့မဟုတ် `SYSTEM.LOG1#1234` ကဲ့သို့ log entry။ |
| `TransactionLogStatus` | ဤ record/hive context အတွက် `clean`, `replayed`, `unchanged`, သို့မဟုတ် `not_applied`။ |
| `LogsApplied` | Juntei replay လုပ်ခဲ့သော transaction-log ဖိုင်အမည်များ။ ၎င်းတို့ကို အစီရင်ခံစာတွင် မှတ်တမ်းတင်ပါ။ |
| `Problems` | ပျက်စီးမှု၊ ပျောက်နေသော cell များ၊ type မကိုက်ညီမှု၊ ဖြေရှင်းမရသော list/parent များ သို့မဟုတ် အခြား ကန့်သတ်ချက်များ။ |

## ပြန်လည်ရယူမှု အခြေအနေ အဓိပ္ပာယ်ဖော်ခြင်း { #recovery-status-interpretation }

| အခြေအနေ | အဓိပ္ပာယ် |
|---|---|
| `deleted_complete` | ဖျက်ထားသော key/value ကို သိရှိထားသော path/ဒေတာ အပြည့်အစုံဖြင့် ပြန်လည်ရယူခဲ့ပါသည်။ သို့သော် သမိုင်းဝင်ဖြစ်ပြီး လက်ရှိ active အခြေအနေ မဟုတ်ပါ။ |
| `deleted_partial` | Path၊ value list၊ value သို့မဟုတ် ဒေတာ အချို့ကို ထပ်ရေးခံရ/ဖတ်မရ ဖြစ်ခဲ့ပါသည်။ ပြန်လည်အသုံးပြုထားသော cell များမှ ကွက်လပ်များကို မဖြည့်ပါနှင့်။ |
| `residual_value_only` | Value သည် လုံလောက်သော key context မပါဘဲ ကျန်ရှိခဲ့ပါသည်။ `KeyPath` မရှိနိုင်ပါ။ |
| `transaction_log_recovered` | အခြေအနေသည် pre-replay/log ဗားရှင်းများတွင်သာ ရှိပြီး လက်ရှိ allocated/free record များတွင် မရှိပါ။ |

## စုံစမ်းစစ်ဆေးမှု ဗျူဟာ { #investigation-strategy }

1. သိရှိထားလျှင် တိကျသော အဓိပ္ပာယ်ပါ key ဖြင့် စတင်ပါ။ ရှာဖွေတွေ့ရှိရန် regex/ဒေတာ ရှာဖွေခြင်းကို အသုံးပြုပါ။
2. အကြောင်းအရာမတိုင်မီ `TransactionLogStatus`၊ `AllocationStatus`၊ `RecoveredFrom` နှင့် `Problems` ကို စစ်ဆေးပါ။
3. ဒေတာကို key ၏ စာတမ်းပါ schema အတိုင်း အဓိပ္ပာယ်ဖော်ပါ။ `REG_BINARY` သည် ကိုယ်တိုင် မဖော်ပြနိုင်ပါ။
4. ပြင်ဆင်မှုကို runtime အထောက်အထား (event log များ၊ Prefetch၊ SRUM၊ service/task များ) နှင့် ဆက်စပ်စစ်ဆေးပါ။
5. အသစ် သို့မဟုတ် အငြင်းပွားဖွယ် field များအတွက် raw ဒေတာကို ထိန်းသိမ်းပါ။

## နောက်ထပ် အသေးစိတ်များ { #further-details }

- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
- [Maxim Suhanov ၏ registry ဖိုင်ပုံစံ specification](https://github.com/msuhanov/regf)
- [yarp registry parser](https://github.com/msuhanov/yarp)
- [libyal libregf](https://github.com/libyal/libregf)
- [Microsoft: Registry value type များ](https://learn.microsoft.com/en-us/windows/win32/sysinfo/registry-value-types)
