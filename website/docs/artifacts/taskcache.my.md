# Task Scheduler cache { #task-scheduler-cache }

## TaskCache ဆိုသည်မှာ { #what-taskcache-is }

Windows Task Scheduler သည် task path များကို GUID များ၊ trigger/စာရင်းများ၊ principal များ၊ action များ၊ runtime metadata
နှင့် security descriptor များနှင့် ချိတ်ဆက်ပေးသော registry cache တစ်ခုကို `SOFTWARE` အောက်တွင် ထားရှိသည်။ Juntei သည်
execution နှင့် COM-handler action များအပြင် legacy email/message action များ၊ task timestamp များနှင့် ပုံမှန်
Task Scheduler interface မှ ဝှက်ထားသော task များ၏ လက္ခဏာများကို decode လုပ်သည်။

Task များသည် အမြဲတမ်းဝင်ရောက်နိုင်မှု (persistence) နှင့် အဝေးမှ လုပ်ဆောင်ခြင်းအတွက် အသုံးများသော နည်းလမ်းဖြစ်သည်။
တိုက်ခိုက်သူက task ၏ XML ဖိုင်ကို ဖျက်ခြင်း သို့မဟုတ် ပြင်ဆင်ခြင်း ပြုလုပ်သည့်အခါတွင်လည်း registry cache သည် တန်ဖိုးရှိသည်။

<span class="evidence-strength">အကောင်းဆုံး ကောက်ချက်:</span> TaskCache သည် task/action ကို ပြင်ဆင်သတ်မှတ်ခဲ့သည် သို့မဟုတ်
ထိန်းသိမ်းထားခဲ့သည်။ သီးခြား လုပ်ဆောင်မှုတစ်ခုကို သက်သေပြရန် runtime field များနှင့် သီးခြားလွတ်လပ်သော log များ လိုအပ်သည်။

## တည်နေရာများ { #locations }

```text
SOFTWARE\Microsoft\Windows NT\CurrentVersion\Schedule\TaskCache
C:\Windows\System32\Tasks\...   (task XML/files; collect separately)
```

`SOFTWARE` ကို transaction log များနှင့်အတူ စုဆောင်းပြီး `System32\Tasks` tree တစ်ခုလုံးကိုလည်း စုဆောင်းပါ။ Task Scheduler
Operational နှင့် Security event log များက ဖန်တီးမှု/အပ်ဒိတ်/runtime အထောက်အထားများကို ဖြည့်စွက်ပေးသည်။

## စိစစ်သူများ သတိပြုရန် { #analyst-cautions }

- ပြင်ဆင်ခံရခြင်း၊ ဖျက်ခံရခြင်း၊ မပြည့်စုံသော မှတ်ပုံတင်ခြင်း သို့မဟုတ် ပျက်စီးခြင်းနောက်ပိုင်း registry cache နှင့် task XML တို့ ကွဲလွဲနိုင်သည်။
- Juntei ၏ `Hidden` သည် ဖွဲ့စည်းပုံဆိုင်ရာ ဝှက်ထားမှု (သက်ဆိုင်ရာ အခြေအနေများတွင် Tree ချိတ်ဆက်မှု/security descriptor
  ပျောက်နေခြင်း) ကို ခွဲခြားသတ်မှတ်ခြင်းဖြစ်ပြီး ပုံမှန် “Hidden” UI flag ဖြင့် ပြင်ဆင်ထားသော task ကို ဆိုလိုခြင်း မဟုတ်ပါ။
- Windows 7 သည် task action များကို cache ၏ `Actions` value တွင်မဟုတ်ဘဲ XML တွင် သိမ်းဆည်းသည်။ ထိုနေရာတွင် ဗလာ action
  တစ်ခုသည် အလိုအလျောက် သံသယဖြစ်ဖွယ် မဟုတ်ပါ။
- `LastRun` နှင့် `LastResult` သည် အသုံးဝင်သော်လည်း ပြည့်စုံပြီး ပြင်ဆင်၍မရသော လုပ်ဆောင်မှု သမိုင်းကြောင်း မဟုတ်ပါ။
- `Context` သည် ပြင်ဆင်သတ်မှတ်ထားသော principal/context ကို ဖော်ပြပြီး task ကို မှတ်ပုံတင်ခဲ့သော အသုံးပြုသူ မဟုတ်ပါ။
- COM handler CLSID များကို registry/ဖိုင်များမှ သီးခြား ဖြေရှင်းရန် လိုအပ်သည်။

## Juntei ဖြင့် ခွဲခြမ်းစိတ်ဖြာခြင်း { #parse-with-juntei }

```text
Usage: juntei parse-registry taskcache <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SOFTWARE hives
  -f, --file <FILE>      Parse one SOFTWARE hive
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Filtering:
      --task <NAME>  Full task path or final task name

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry taskcache -f SOFTWARE -o tasks.csv
juntei parse-registry taskcache -f SOFTWARE -t jsonl \
  | jq 'select(.Hidden == true or (.Executable? | test("powershell|cmd|mshta"; "i")))'
```

## ထုတ်ယူမှု field များ { #output-fields }

| Field | အဓိပ္ပာယ်ဖော်ပုံ |
|---|---|
| `Host`, `SourcePath` | Host အခြေအနေနှင့် source `SOFTWARE` hive။ |
| `TaskName` | Scheduler path အပြည့်အစုံ။ XML နှင့် event log များသို့ အဓိက pivot။ |
| `TaskGUID` | Tree နှင့် Tasks cache entry များကို ချိတ်ဆက်ပေးသော GUID။ ခိုင်မာသော ချိတ်ဆက်စစ်ဆေးရေး key။ |
| `TaskIndex` | Scheduler အသုံးပြုသော Tree/index metadata။ အခြေအနေအတွက်သာဖြစ်ပြီး လုပ်ဆောင်မှု အရေအတွက် မဟုတ်ပါ။ |
| `TaskLists` | Trigger လုပ်ပုံ/ကိုင်တွယ်ပုံကို အမျိုးအစားခွဲပေးသော cache စာရင်းများ (`Boot`, `Logon`, `Plain`, `Maintenance`)။ |
| `Hidden` | ဖွဲ့စည်းပုံဆိုင်ရာ hidden-task ညွှန်ကိန်း။ ပြင်ဆင်ခံရသည်ဟု ကောက်ချက်မချမီ `InTree`၊ `InTasks` နှင့် descriptor အခြေအနေကို ပြန်လည်သုံးသပ်ပါ။ |
| `InTree`, `InTasks` | Cache နှစ်ဘက်စလုံး ရှိ/မရှိ။ ချိတ်ဆက်မှု ပျောက်နေခြင်းသည် ဖျက်ခံရခြင်း/ပြင်ဆင်ခံရခြင်း/မပြည့်စုံသော မှတ်ပုံတင်ခြင်းကို ညွှန်ပြနိုင်သည်။ |
| `HasSecurityDescriptor` | Tree entry တွင် `SD` ရှိ/မရှိ။ မရှိခြင်းသည် သက်ဆိုင်ရာ ဗားရှင်းများတွင် hidden-task အပြုအမူကို ဖြစ်စေနိုင်သည်။ |
| `TreeKeyLastWrittenUTC`, `TaskKeyLastWrittenUTC` | Cache ဘက်တစ်ခုချင်းစီအတွက် key အဆင့် ပြောင်းလဲမှုများ။ ဖန်တီးချိန်/run ချိန် အတိအကျ မဟုတ်ပါ။ |
| `ActionIndex` | Task အတွင်းရှိ သုညမှစသော action။ Task တစ်ခုက row အများအပြား ထုတ်ပေးနိုင်သည်။ |
| `ActionType` | `exec`၊ `com_handler`၊ `send_email` သို့မဟုတ် `show_message`။ |
| `Executable`, `Args`, `WorkingDirectory` | ထိရောက်သော execution action။ Quote လုပ်ပုံ၊ variable များ၊ relative path များနှင့် LOLBin များကို အဓိပ္ပာယ်ဖော်ပါ။ |
| `COM-ClassID`, `ActionData` | COM handler CLSID နှင့် ဒေတာ၊ သို့မဟုတ် legacy email/message payload။ CLSID ကို သီးခြား ဖြေရှင်းပါ။ |
| `Context` | Action များနှင့်အတူ encode လုပ်ထားသော principal context (`LocalSystem`, `Author` စသည်)။ ပြင်ဆင်သတ်မှတ်ထားသော identity ဖြစ်ပြီး မှတ်ပုံတင်သူ မဟုတ်ပါ။ |
| `Author`, `URI`, `Description` | Task metadata။ တိုက်ခိုက်သူက အလွယ်တကူ ထိန်းချုပ်နိုင်ပြီး ယုံကြည်မှု အထောက်အထား မဟုတ်ပါ။ |
| `LastRegisteredUTC` | ရှိပါက DynamicInfo ၏ မှတ်ပုံတင်/အပ်ဒိတ် အချိန်။ |
| `LastRunUTC`, `LastSuccessfulRunUTC`, `LastResult` | Cache လုပ်ထားသော runtime/ရလဒ် metadata။ Operational log များနှင့် process အထောက်အထားများနှင့် ချိတ်ဆက်စစ်ဆေးပါ။ |
| `ActionsRaw` | Decode လုပ်ခြင်း မပြည့်စုံပါက သို့မဟုတ် မှန်ကန်မှု စစ်ဆေးရန် လိုအပ်ပါက မူရင်း action byte များ။ |
| `TransactionLogStatus`, `LogsApplied` | Registry recovery အခြေအနေ။ |
| `Problems` | Path/GUID မကိုက်ညီမှု၊ မှားယွင်းသော value အမျိုးအစား၊ ဖြတ်တောက်ခံရသော action ဖွဲ့စည်းပုံ သို့မဟုတ် ပျောက်နေသော အစိတ်အပိုင်းများ။ |

## စုံစမ်းစစ်ဆေးမှု ဗျူဟာ { #investigation-strategy }

1. ဝှက်ထားသော/မကိုက်ညီသော task များနှင့် Microsoft မဟုတ်သော path များကို ဦးစွာ ပြန်လည်သုံးသပ်ပါ။
2. Argument များ၊ working directory၊ environment ချဲ့ထွင်မှုနှင့် COM ဖြေရှင်းမှု အပါအဝင် ထိရောက်သော ညွှန်ကြားချက်များကို ပြန်လည်တည်ဆောက်ပါ။
3. Registry record ကို task XML နှင့် နှိုင်းယှဉ်ပြီး ရည်ညွှန်းထားသော executable ကို hash/timestamp စစ်ဆေးပါ။
4. `TaskName`/GUID/အချိန်ကို Task Scheduler event များ၊ Security 4698/4702၊ process log များ၊ Prefetch နှင့် MFT/USN တို့နှင့် ချိတ်ဆက်စစ်ဆေးပါ။
5. တိုက်ခိုက်သူက task ကို ဖယ်ရှားခဲ့ပါက ဖျက်ထားသော/ပြန်လည်ရယူထားသော registry ဒေတာကို ရှာဖွေပါ။

## နောက်ထပ် အသေးစိတ်များ { #further-details }

- [Microsoft: Task Scheduler schema](https://learn.microsoft.com/en-us/windows/win32/taskschd/task-scheduler-schema)
- [Microsoft: Task Scheduler event log](https://learn.microsoft.com/en-us/windows/win32/taskschd/task-scheduler-1-0-events)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
- [MITRE ATT&CK T1053.005 Scheduled Task](https://attack.mitre.org/techniques/T1053/005/)
