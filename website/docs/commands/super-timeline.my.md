# စူပါအချိန်လိုင်း { #super-timeline }

`super-timeline` သည် parse လုပ်ထားသော artifact များကို ဘုံ schema တစ်ခုရှိသည့် event များအဖြစ် ပြောင်းလဲပေးပါသည်။ Output ကို **မူလအားဖြင့် စီထားခြင်း မရှိပါ**: ၎င်းသည် worker များ ပြီးဆုံးသည့် အစီအစဉ်နှင့် မသက်ဆိုင်ဘဲ ရှာဖွေတွေ့ရှိထားသော input ဖိုင် အစီအစဉ်နှင့် parser တစ်ခုစီ၏ projection အစီအစဉ်ကို လိုက်နာပါသည်။ အချိန်အစဉ်လိုက် စီရန်နှင့် ထပ်နေသည်များကို ပေါင်းစည်းရန် `--sort` ကို ထည့်ပါ။ Prefetch record တစ်ခုတွင် ပရိုဂရမ် run ခဲ့သည့်အချိန် (execution time) အများအပြား ပါဝင်နိုင်ပြီး MFT record တစ်ခုတွင် ဖိုင် timestamp ရှစ်ခု သို့မဟုတ် ထို့ထက်ပို၍ ပါဝင်နိုင်ပါသည်။ ၎င်းတို့တစ်ခုစီသည် သီးခြား label တပ်ထားသော event တစ်ခု ဖြစ်လာပါသည်။ ထို့ကြောင့် timestamp တစ်ခုစီ၏ အမှန်တကယ် အဓိပ္ပာယ်ကို မဆုံးရှုံးစေဘဲ မတူညီသော ရင်းမြစ်များမှ အထောက်အထားများကို နှိုင်းယှဉ်ရန် ပိုမိုလွယ်ကူပါသည်။

မူလ အထောက်အထား ဖိုင်တွဲမှ မဟုတ်ဘဲ [`auto-parse`](auto-parse.md) ၏ ရလဒ်များမှ စတင်ပါ:

```bash
juntei auto-parse -d evidence/ -o parsed/
juntei super-timeline -d parsed/ -o SuperTimeline.csv
```

Parse လုပ်ထားသော ဖိုင်များ၊ manifest များနှင့် မူလ အထောက်အထားကို ဆက်လက် ထိန်းသိမ်းထားပါ။ Timeline သည် ခွဲခြမ်းစိတ်ဖြာရန်အတွက် ထုတ်ယူထားသော projection တစ်ခုသာ ဖြစ်ပါသည်။ ၎င်းသည် ထို record များကို အစားမထိုးသကဲ့သို့ Plaso ၏ native storage ဖိုင်ကိုလည်း မဖန်တီးပါ။

## အသုံးပြုပုံ { #usage }

```bash
# Native CSV (default)
juntei super-timeline -d parsed/ -o SuperTimeline.csv

# Chronological CSV with duplicate consolidation
juntei super-timeline -d parsed/ -o SortedTimeline.csv --sort

# Native JSON Lines
juntei super-timeline -d parsed/ -o SuperTimeline.jsonl -t jsonl

# Combine native records and an existing Hayabusa JSONL timeline
juntei super-timeline -f NativeRecords.jsonl --hayabusa Hayabusa.jsonl \
  -o SuperTimeline.jsonl -t jsonl

# Timesketch import profile; also available with CSV
juntei super-timeline -d parsed/ -o Timesketch.jsonl -t jsonl --profile timesketch

# One standalone CSV needs an artifact identifier and, if unavailable, a computer name
juntei super-timeline -f MFT.csv --input-artifact mft --computer HOST01 \
  -o Host01Timeline.csv

# Reduce the time range, memory used by sorting, or number of workers
juntei super-timeline -d parsed/ -o IncidentTimeline.jsonl -t jsonl \
  --timeline-start "2026-10-20T06:00:00Z" --timeline-end "2026-10-20T14:00:00Z" \
  --sort --memory-limit 64 -T 4 --temp-dir scratch/
```

`--hayabusa FILE` ကို အကြိမ်ကြိမ် ထပ်၍ ပေးနိုင်ပြီး အခြား input မပါဘဲ သီးသန့်လည်း အသုံးပြုနိုင်ပါသည်။ ၎င်းသည် Hayabusa ကို run စရာမလိုဘဲ ရှိပြီးသား event/alert timestamp များနှင့် rule context ကို import လုပ်ပါသည်။ `Details` သည် string ဖြစ်သော informational event များကိုလည်း ပံ့ပိုးပါသည်။ `--artifact mft,prefetch` သည် projection ကို ရွေးချယ်ထားသော artifact identifier များသို့သာ ကန့်သတ်ပါသည်။ အချိန်ကန့်သတ်ချက်များတွင် နယ်နိမိတ်တန်ဖိုးများလည်း ပါဝင်ပြီး (inclusive) UTC offset သို့မဟုတ် `Z` ကို တိကျစွာ ဖော်ပြရန် လိုအပ်ပါသည်။

### Input ရွေးချယ်ခြင်း { #choosing-an-input }

| Input | ခွဲခြားသတ်မှတ်ပုံ |
|---|---|
| `-d DIR` | `manifest.json` က အောင်မြင်ခဲ့သော primary task များ၊ artifact ID များ၊ ကွန်ပျူတာများနှင့် output ဖိုင်အတိအကျများကို ပေးပါသည်။ မိတ္တူနှစ်ခုလုံးကို import မလုပ်မိစေရန် secondary `auto-parse -j` task များကို ဖယ်ထုတ်ထားပါသည်။ |
| `-f FILE` | Native JSON/JSONL သည် ၎င်းအတွင်း ပါရှိသော `Artifact` နှင့် `Comp` field များကို အသုံးပြုပါသည်။ Tag မပါသော export များအတွက် `--input-artifact` လိုအပ်ပြီး standalone CSV အတွက်မူ ၎င်းကို အမြဲ လိုအပ်ပါသည်။ |
| `--computer NAME` | Input record တွင် ကွန်ပျူတာအမည် မပါသည့်အခါ အစားထိုး အသုံးပြုမည့် တန်ဖိုး ဖြစ်ပါသည်။ ၎င်းကို မပေးထားလျှင် မသိသော ကွန်ပျူတာကို `Unknown` ဟု ရှင်းလင်းစွာ ဖော်ပြပါသည်။ |
| `--hayabusa FILE` | Hayabusa timestamp များ၊ rule metadata၊ ကွန်ပျူတာနှင့် ပါရှိပါက `EvtxFile` ပါဝင်သော JSONL။ |

လက်ရှိ manifest များသည် parse လုပ်ထားသော ဖိုင်တွဲကို နေရာရွှေ့ခြင်းကို ပံ့ပိုးပါသည်: မှတ်တမ်းတင်ထားသော absolute output path များကို ဖိုင်တွဲအသစ်အောက်တွင် ဖြေရှင်းပါသည်။ Symbolic link ၏ target များ အပါအဝင် input များသည် ထိုဖိုင်တွဲအတွင်း၌သာ ရှိရပါမည်။ Artifact တစ်ခုချင်းစီအလိုက် ခွဲထားသော legacy Amcache၊ SRUM နှင့် Jump Lists ဖိုင်တွဲများကို ပံ့ပိုးပါသည်။ Category ဖိုင်တွဲများအတွက်မူ `OutputFiles` ကို တိကျစွာ ဖော်ပြထားရန် လိုအပ်ပါသည်။

Output နှင့် နှိုင်းယှဉ်ထားသော relative path များနှင့် flatten လုပ်ထားသော multi-table export များ ပါဝင်သည့် လက်ရှိ manifest များကို ပံ့ပိုးပါသည်။ ဤညွှန်ကြားချက်သည် ပိုမိုသစ်သော ဘရောက်ဇာ၊ WebCache၊ Outlook နှင့် UAL table များ အပါအဝင် အလိုအလျောက် parsing က အစီအစဉ်ချထားသော artifact တိုင်းကို အသိအမှတ်ပြုပါသည်။

Manifest က VHDX သို့မဟုတ် E01 image ကို ဖော်ပြထားသည့်အခါ source reference များတွင် image နှင့် image အတွင်းရှိ path ပါဝင်ပြီး image အမည်ကို event context တွင် ထိန်းထားပါသည်။ Parse လုပ်ထားသော ရလဒ်များကို ဖတ်ရန် ယာယီ staging workspace မလိုအပ်ပါ။

Raw source ကို record မှ သို့မဟုတ် အဓိပ္ပာယ် မကွဲပြားသော manifest command မှ ရယူပါသည်။ `-d` ဖြင့် raw-source reference များသည် အထောက်အထား ဖိုင်တွဲနှင့် နှိုင်းယှဉ်ထားသော relative path များ ဖြစ်ပြီး `Details.ParsedFile` နှင့် summary ၏ `InputFiles` တို့သည် parse လုပ်ထားသော ဖိုင်တွဲနှင့် နှိုင်းယှဉ်ထားသော relative path များ ဖြစ်ပါသည်။ Summary ရှိ output/အစီရင်ခံစာ အမည်များသည် timeline ၏ output ဖိုင်တွဲနှင့် နှိုင်းယှဉ်ထားသော relative path များ ဖြစ်ပါသည်။ SRUM CSV table များကို host တွင် တစ်ခုတည်းဟု ခွဲခြားသိနိုင်သော စုဆောင်းထားသည့် SRUDB.dat တစ်ခုသာ ရှိသည့်အခါမှသာ database တစ်ခုနှင့် ချိတ်ပေးပါသည်။ ထိုသို့ မဟုတ်ပါက `SourceFile` သည် parse လုပ်ထားသော ဖိုင်ကို ညွှန်ပြပြီး `Details.RawSourceAvailable` သည် `false` ဖြစ်ပါသည်။

Standalone input ကို ၎င်း၏ ဖိုင်အမည်ဖြင့် reference လုပ်ပါသည်။ ဖိုင်အမည်တူသော input များသည် တစ်ခုနှင့်တစ်ခု ကွဲပြားသော `./Inputs/<ID>/<filename>` reference များကို အသုံးပြုပါသည်။ သိရှိထားသော အထောက်အထား root များ၏ အပြင်ဘက်ရှိ absolute raw-source path များသည် `./Sources/<ID>/<filename>` identifier များကို အသုံးပြုပါသည်။ ဤ opaque reference များသည် analyst ၏ absolute ဖိုင်တွဲအမည်များကို export မလုပ်ဘဲ ရင်းမြစ်များကြား ကွဲပြားမှုကို ထိန်းထားပါသည်။ မူလ ရင်းမြစ်ကို သိရန် `ParsedFile` နှင့် `InputRecord` ကို အသုံးပြု၍ native record ကို ကြည့်ပါ။ Windows executable path များကဲ့သို့ အထောက်အထားထဲတွင် မှတ်တမ်းတင်ထားသော path များကိုမူ မပြောင်းလဲဘဲ ထားပါသည်။

## ဘုံ event field များ { #common-event-fields }

| Field | အဓိပ္ပာယ် |
|---|---|
| `Timestamp` | UTC RFC 3339 ဖြစ်ပြီး အစီအစဉ် တသမတ်တည်း ဖြစ်စေရန် ဒဿမ ဂဏန်း ကိုးလုံးဖြင့် ရေးပါသည်။ Padding ဖြည့်ခြင်းက တိကျမှုကို မတိုးစေပါ။ |
| `Comp` | Input record နှင့် ဆက်စပ်သော ကွန်ပျူတာ။ |
| `Artifact` | `mft`၊ `prefetch`၊ `srum` သို့မဟုတ် `hayabusa` ကဲ့သို့ တည်ငြိမ်သော artifact identifier။ |
| `TimestampType` | `MFT SI Data modified` ကဲ့သို့ timestamp ၏ အဓိပ္ပာယ်ကို ဖော်ပြချက်။ |
| `User` | Adapter တွင် ရှိပါက မှတ်တမ်းတင်ထားသော account သို့မဟုတ် SID။ ဗလာ ဖြစ်နိုင်ပါသည်။ |
| `Path` | သက်ရောက်သော ဖိုင်၊ registry key သို့မဟုတ် artifact အလိုက် အခြား object။ |
| `Message` | Timeline viewer အတွက် အတိုချုပ် ဖော်ပြချက်။ |
| `SourceFile` | အထောက်အထား ဖိုင်တွဲနှင့် နှိုင်းယှဉ်ထားသော raw-source reference၊ ၎င်း၏ root ကို မသိပါက relative source identifier၊ ထိုနှစ်ခုလုံး မရပါက အထက်တွင် ဖော်ပြခဲ့သည့် parse လုပ်ထားသော ဖိုင်ကို အစားထိုး (fallback) ညွှန်ပြချက်။ |
| `SourceField` | Timestamp အတွက် အသုံးပြုခဲ့သော parse လုပ်ထားသည့် field အတိအကျ သို့မဟုတ် nested တည်နေရာ။ |
| `RecordID` | Artifact အလိုက် record identity။ ၎င်းသည် EVTX Record ID ဖြစ်သည်ဟု အမြဲ မဆိုလိုပါ။ |
| `Details` | မူလ timestamp၊ တိကျမှု၊ အချိန် အခြေခံ (time basis)၊ အချိန်၏ အခန်းကဏ္ဍ (time role)၊ parse လုပ်ထားသော ဖိုင်၊ input record/တည်နေရာနှင့် ရရှိနိုင်သော recovery/rule metadata တို့ ပါဝင်သည့် ဖွဲ့စည်းထားသော context။ |

CSV တွင် `Details` သည် quote လုပ်ထားသော JSON cell တစ်ခု ဖြစ်ပါသည်။ JSONL တွင်မူ object တစ်ခု ဖြစ်ပါသည်။ Timesketch profile သည် အခြား provenance field များကို ထိန်းထားရင်း `Timestamp`၊ `TimestampType` နှင့် `Message` ကို `datetime`၊ `timestamp_desc` နှင့် `message` သို့ အသီးသီး ပြောင်းလဲပါသည်။ ၎င်းတို့သည် [Timesketch import အတွက် မဖြစ်မနေ လိုအပ်သော field သုံးခု](https://timesketch.org/guides/user/import-from-json-csv/) ဖြစ်ပါသည်။

`Details` တွင် binary ဒေတာ အပြည့်အစုံ သို့မဟုတ် native record field အားလုံးကို မထည့်ဘဲ ရွေးချယ်ထားသော context ကိုသာ ရည်ရွယ်ချက်ရှိရှိ ထည့်ထားပါသည်။ ရှည်လျားသော string များ၊ array များနှင့် object များကို အရွယ်အစား ကန့်သတ်ပြီး `ProjectionOmitted` ဟု အမှတ်အသား ပြုပါသည်။ Native record အပြည့်အစုံကို စစ်ဆေးရန် `Details.ParsedFile`၊ `InputRecord` နှင့် `Location` ကို ကြည့်ပါ။ CSV အတွက် record နံပါတ်သည် header နောက်ရှိ CSV record များကို ရေတွက်ခြင်း ဖြစ်ပြီး quote လုပ်ထားသော cell များအတွင်းရှိ ရုပ်ပိုင်းဆိုင်ရာ စာကြောင်းများကို ရေတွက်ခြင်း မဟုတ်ပါ။ JSONL အတွက် ၎င်းသည် စာကြောင်းနံပါတ် ဖြစ်ပြီး JSON အတွက် ထိပ်တန်းအဆင့် document နံပါတ်နှင့် nested တည်နေရာ ဖြစ်ပါသည်။

## အချိန်များ၏ အဓိပ္ပာယ် { #what-the-times-mean }

Timestamp သည် မှတ်တမ်းတင်ထားသော object တစ်ခုအကြောင်း အထောက်အထား ဖြစ်ပါသည်။ ၎င်းတစ်ခုတည်းဖြင့် ပရိုဂရမ် run ခဲ့ခြင်း (execution) သို့မဟုတ် တိုက်ခိုက်မှုကို သက်သေမပြနိုင်ပါ။ Adapter များသည် အောက်ပါ ကွဲပြားချက်များကို ရည်ရွယ်ချက်ရှိရှိ ထိန်းထားပါသည်:

| Artifact များ | Timeline တွင် အဓိပ္ပာယ်ဖော်ပုံ |
|---|---|
| MFT | SI နှင့် FN တို့၏ ပြင်ဆင်ချိန်၊ ဝင်ရောက်ချိန်၊ metadata ပြောင်းလဲချိန်နှင့် ဖန်တီးချိန်များကို သီးခြားစီ ထားပါသည်။ အမည်အလိုက် FN attribute များကိုလည်း တစ်ခုနှင့်တစ်ခု ခွဲခြားထားပါသည်။ `--sort` ဖြင့် filename row များပေါ်တွင် ထပ်နေသော SI field များကို ပေါင်းစည်းပါသည်။ ADS row များ၏ base ဖိုင်နှင့် မျှဝေထားသော အချိန်များကို mode နှစ်ခုလုံးတွင် ဖယ်ထုတ်ပါသည်။ FILE record ၏ slack မှ ပြန်လည်ရယူထားသော အမည်များကို ပြန်လည်ရယူထားသည့် FN event များအဖြစ် သီးခြား ထားပါသည်။ |
| I30 | သိမ်းဆည်းထားသော FN/index metadata ဖြစ်ပြီး ရရှိနိုင်ပါက slack/recovery context ပါဝင်ပါသည်။ ၎င်းတို့သည် directory index ကို ရေးသားခဲ့သည့်အချိန်၏ timestamp များ မဟုတ်ပါ။ |
| USN Journal | Journal record ၏ timestamp နှင့် update reason များ။ Record တစ်ခုသည် ပြောင်းလဲမှု အများအပြားကို ဖော်ပြနိုင်ပြီး ၎င်းတို့ စတင်ခဲ့သည့် အချိန်အတိအကျကို မှတ်တမ်းတင်ထားသည်ဟု မဆိုလိုပါ။ |
| LogFile | သိမ်းဆည်းထားသော redo/undo timestamp snapshot များနှင့် filename metadata။ Redo နှင့် undo ကို သီးခြားစီ ထားပါသည်။ LSN များနှင့် ဖိုင် offset များသည် identifier များ ဖြစ်ပြီး ရက်စွဲများအဖြစ် ဘယ်သောအခါမျှ မပြောင်းလဲပါ။ |
| Prefetch | မှတ်တမ်းတင်ထားသော ပရိုဂရမ် run ခဲ့သည့်အချိန် (execution time) များ။ `--sort` ဖြင့် ၎င်းတို့ကို ကိုက်ညီသော Prefetch timeline export နှင့် ပေါင်းစည်းပါသည်။ ကိုးကားထားသော volume ဖန်တီးချိန် metadata ကို သီးခြား label တပ်ပါသည်။ |
| ဘရောက်ဇာ SQLite database များ | မှတ်တမ်းတင်ထားသော visit များ၊ ပြန်လည်ရယူထားသော စာမျက်နှာ၏ နောက်ဆုံး visit metadata၊ download စတင်ချိန်/ပြီးဆုံးချိန်၊ cookie ဖန်တီးချိန်/ဝင်ရောက်ချိန်/သက်တမ်းကုန်ချိန်နှင့် form history ၏ ပထမ/နောက်ဆုံး အသုံးပြုချိန် သို့မဟုတ် ဖျက်ခြင်းကို မှတ်ထားသော tombstone record များ။ Download ပြီးဆုံးချိန် တစ်ခုတည်းဖြင့် download အောင်မြင်ခဲ့ကြောင်း သို့မဟုတ် ဖိုင်ကို run ခဲ့ကြောင်း သက်သေမပြနိုင်ပါ။ Cookie သက်တမ်းကုန်ချိန်သည် ကြိုတင် သတ်မှတ်ထားသော နောက်ဆုံးရက် ဖြစ်ပါသည်။ Recovery အခြေအနေ၊ confidence နှင့် origin တို့ကို context တွင် ထိန်းထားပါသည်။ |
| Internet Explorer / legacy Edge WebCache | သိမ်းဆည်းထားသော entry ၏ ဝင်ရောက်ချိန်၊ ဖန်တီးချိန်/ပြင်ဆင်ချိန်၊ သက်တမ်းကုန်ချိန်နှင့် synchronization၊ download စတင်ချိန်များ၊ cookie metadata နှင့် container ဝင်ရောက်ချိန်/scavenging (ရှင်းလင်းဖယ်ရှားခြင်း)။ Cache ၏ စီမံထိန်းသိမ်းမှု မှတ်တမ်း (bookkeeping) သည် အသုံးပြုသူက စာမျက်နှာကို ဖွင့်ခဲ့ကြောင်း အလိုအလျောက် အထောက်အထား မဖြစ်ပါ။ |
| Outlook PST၊ OST၊ MSG နှင့် OFT | Message ကို submit လုပ်ခဲ့သည့်အချိန်/ရောက်ရှိခဲ့သည့်အချိန်၊ ဖိုင်တွဲ၊ item နှင့် ပူးတွဲဖိုင် ဖန်တီးချိန်/ပြင်ဆင်ချိန်၊ ပြက္ခဒိန် စတင်ချိန်/ပြီးဆုံးချိန်/reminder များ၊ task စတင်ချိန်/သတ်မှတ်ရက်/ပြီးစီးချိန်/reminder များနှင့် time type အဖြစ် တိကျစွာ သတ်မှတ်ထားသော property များ။ စီစဉ်ထားသော အချိန်များသည် အစည်းအဝေး ကျင်းပခဲ့ကြောင်း၊ reminder ပေါ်လာခဲ့ကြောင်း သို့မဟုတ် ပရိုဂရမ်တစ်ခု run ခဲ့ကြောင်း သက်သေမပြပါ။ သိမ်းဆည်းထားသော ရောက်ရှိချိန်သည် message ကို ဖတ်ခဲ့ကြောင်း သက်သေမပြပါ။ |
| အသုံးပြုသူ ဝင်ရောက်မှု မှတ်တမ်း (UAL) | ပထမ/နောက်ဆုံး ဝင်ရောက်မှု metadata၊ database ထဲသို့ ထည့်သွင်းချိန်/ဖန်တီးချိန်၊ ပထမ/နောက်ဆုံး တွေ့ရှိချိန် (first/last seen) နှင့် သိမ်းဆည်းထားသော operating system boot အချိန်။ Role၊ account၊ source address နှင့် နေ့စဉ် counter များကို context အဖြစ်သာ ထားပါသည်။ နေ့စဉ် ရက်စွဲ/အရေအတွက်များကို သန်းခေါင်ယံအချိန် logon များအဖြစ် တီထွင်ဖန်တီးခြင်း မပြုပါ။ အချိန်မပါသော (untimed) identity နှင့် DNS row များကို တွဲဖက် အစီရင်ခံစာတွင် ဆက်လက် ရရှိနိုင်ပါသည်။ |
| Amcache | Inventory key ၏ last-write အချိန်များ၊ parent volume key context၊ မှတ်တမ်းတင်ထားသော install ရက်စွဲများနှင့် PE link metadata။ Inventory ထဲတွင် ရှိနေခြင်း သို့မဟုတ် LinkDate တစ်ခုတည်းဖြင့် ပရိုဂရမ် run ခဲ့ကြောင်း သက်သေမပြနိုင်ပါ။ |
| ShimCache | သိမ်းဆည်းထားသော ဖိုင်ပြင်ဆင်ချိန်၊ cache update metadata နှင့် cache key ၏ last-write အချိန်။ ၎င်းတို့အနက် မည်သည့်အချိန်မျှ execution timestamp (ပရိုဂရမ် run ခဲ့သည့်အချိန်) အဖြစ် အလိုအလျောက် မမှတ်ယူနိုင်ပါ။ |
| SRUM | Provider timestamp/bucket၊ connection စတင်ချိန်၊ တိကျစွာ ဖော်ပြထားသော start/end interval field များနှင့် application identifier metadata။ Byte အရေအတွက်၊ ကြာချိန်နှင့် အခြား counter များသည် ရက်စွဲများ မဟုတ်ပါ။ |
| LNK | Target ၏ MAC timestamp များ၊ tracking identifier ဖန်တီးချိန်နှင့် shell-item အချိန်များ။ ၎င်းတို့သည် သိမ်းဆည်းထားသော target metadata ကို ဖော်ပြခြင်း ဖြစ်ပြီး shortcut ကို ဖွင့်ခဲ့သည့်အချိန်ကို အမြဲ ဖော်ပြသည် မဟုတ်ပါ။ |
| Jump Lists | Entry ၏ နောက်ဆုံး အသုံးပြုချိန် metadata၊ tracking identifier ဖန်တီးချိန်နှင့် ထည့်သွင်းပါရှိသော LNK ၏ target အချိန်များ။ မူလ setting တွင် စုဆောင်းသည့်ဘက်ရှိ container ဖိုင်အချိန်များကို ချန်လှပ်ထားပါသည်။ |
| Defender quarantine | Entry ၏ timestamp နှင့် သိမ်းဆည်းထားသော resource ၏ ဖန်တီးချိန်/ဝင်ရောက်ချိန်/ရေးသားချိန်များ။ ထုတ်ယူ (extract) ခဲ့သည့်အချိန်များသည် endpoint ပေါ်ရှိ လှုပ်ရှားမှု၏ အထောက်အထား မဟုတ်ပါ။ |
| BAM၊ UserAssist၊ FeatureUsage | ရှိပါက မှတ်တမ်းတင်ထားသော လှုပ်ရှားမှု field များ ဖြစ်ပြီး key last-write context ကို သီးခြား ခွဲထားပါသည်။ အချိန်မပါသော counter သည် event ကို မဖန်တီးပါ။ |
| RecentDocs၊ MRU၊ Open/Save MRU၊ ShellBags၊ Office | မှတ်တမ်းတင်ထားသော အသုံးပြုချိန်/trust အချိန်/shell-item အချိန်များနှင့် key context။ စာရင်းတစ်ခု၏ key အချိန်သည် ထိုစာရင်းထဲရှိ item တစ်ခုချင်းစီ၏ နောက်ဆုံး အသုံးပြုချိန် မဟုတ်ပါ။ |
| SAM နှင့် LSA secret များ | မှတ်တမ်းတင်ထားသော account logon/password/expiration သို့မဟုတ် secret သတ်မှတ်ချိန် metadata နှင့် ဆက်စပ်သော key အချိန်များ။ ၎င်းတို့သည် authentication event log များကို အစားမထိုးနိုင်ပါ။ |
| Service များ၊ TaskCache၊ registry persistence၊ BCD | Key/configuration metadata နှင့် တိကျစွာ မှတ်တမ်းတင်ထားသော task အချိန်များ။ Service key ကို ရေးသားခဲ့ခြင်း တစ်ခုတည်းဖြင့် service run ခဲ့ကြောင်း သက်သေမပြနိုင်ပါ။ |
| ကွန်ရက်များနှင့် USB | Network profile/lease နှင့် device install/ချိတ်ဆက်ချိန်/ဖြုတ်ချိန် field များ ဖြစ်ပြီး registry key context ပါဝင်ပါသည်။ Native USB output ရှိ အသုံးပြုသူ MountPoints2 အချိန်များသည် သက်ဆိုင်ရာ user hive ရင်းမြစ်ကို ထိန်းထားပါသည်။ |
| System Info၊ ထည့်သွင်းထားသော ပရိုဂရမ်များ၊ AppCompat၊ အပလီကေးရှင်း မှတ်တမ်း | တိကျစွာ မှတ်တမ်းတင်ထားသော install၊ shutdown၊ inventory/လှုပ်ရှားမှုနှင့် key metadata field များ။ Native System Info ၏ profile/version array များသည် ၎င်းတို့အတွင်းရှိ nested အချိန်များကို ထိန်းထားပါသည်။ |
| Audit policy၊ လုံခြုံရေးဆက်တင်များ၊ MuiCache၊ generic registry | ပေးထားပါက key last-write အချိန်များ။ အချိန်မပါသော value/counter များကို native input တွင် ဆက်လက် ရရှိနိုင်ပါသည်။ |
| Persistence Hunter | ထောက်ခံသော key ၏ last-write အချိန်နှင့် ချိတ်ဆက်စစ်ဆေးထားသော ပြောင်းလဲချိန်။ Finding တစ်ခုသည် ၎င်းကိုယ်တိုင် timestamp ပါသော install လုပ်ခြင်း သို့မဟုတ် compromise ဖြစ်ခဲ့ကြောင်း ဆုံးဖြတ်ချက် မဟုတ်ပါ။ |
| Hayabusa | Rule၊ level၊ channel၊ event ID နှင့် ရရှိနိုင်သော detail context တို့ ပါဝင်သည့် event/alert timestamp။ Rule အများအပြားနှင့် ကွဲပြားသော correlation row များကို သီးခြားစီ ထားပါသည်။ |

မူလ export သည် ထပ်နေသော metadata များ အပါအဝင် timestamp projection အားလုံးကို ထိန်းထားပါသည်။ ၎င်းသည် output တစ်ခုလုံးကို ခြုံ၍ (global) စီခြင်း သို့မဟုတ် ထပ်နေသည်များ ပေါင်းစည်းခြင်းကို မပြုလုပ်ပါ။ `--sort` ဖြင့် ထပ်နေသော projection များကို ပေါင်းစည်းပါသည်။ ထို့ကြောင့် တူညီသော input များမှ စီမထားသော export ထက် row အရေအတွက် နည်းနိုင်ပါသည်။ အဓိပ္ပာယ် သို့မဟုတ် အထောက်အထား ရင်းမြစ် ကွဲပြားသော တူညီသည့် timestamp အများအပြားကို mode နှစ်ခုလုံးတွင် ထိန်းထားပါသည်။ ထပ်နေခြင်းကို ဆုံးဖြတ်သည့် identity တွင် ကွန်ပျူတာ၊ ရင်းမြစ်၊ artifact၊ record၊ အချိန်၏ အဓိပ္ပာယ်နှင့် recovery အခြေအနေတို့ ပါဝင်ပါသည်။ ပေါင်းစည်းထားသော row များတွင် `DuplicateCount` နှင့် အရေအတွက် ကန့်သတ်ထားသော အခြား path/source field များကို မှတ်တမ်းတင်ပါသည်။ ကွဲပြားသော ရင်းမြစ်များကို ၎င်းတို့၏ path များနှင့် timestamp များ ဆင်တူသည်ဆိုရုံမျှဖြင့် ဘယ်သောအခါမျှ ပေါင်းစည်းခြင်း မပြုပါ။ Raw source ကို ဖြေရှင်း၍ မရသည့်အခါ မတူညီသော capture များကို တစ်ခုတည်းအဖြစ် မပေါင်းမိစေရန် input record ၏ identity ကို ထိန်းထားပါသည်။

### တိကျမှု၊ local အချိန်များနှင့် ယူဆချက်များ { #precision-local-times-and-assumptions }

UTC ဖြစ်ကြောင်း သိရှိထားသော field များနှင့် offset ကို တိကျစွာ ဖော်ပြထားသော timestamp များကို UTC သို့ normalize လုပ်ပါသည်။ 100 ns FILETIME များ အပါအဝင် မူလ တန်ဖိုးများနှင့် ၎င်းတို့၏ မှတ်တမ်းတင်ထားသော တိကျမှုကို ထိန်းထားပါသည်။ ရင်းမြစ် parser သို့မဟုတ် downstream viewer က အချိန်ကို ပတ် (round) ပြီးသား ဖြစ်နိုင်ပါသည်။ ဤညွှန်ကြားချက်သည် ဆုံးရှုံးသွားသော တိကျမှုကို ပြန်လည်မရယူနိုင်ပါ။

Upstream parser က `Z` ဖြင့် serialize လုပ်ထားသည့်တိုင် shell-item/FAT field အချို့သည် local time အဓိပ္ပာယ် ရှိပါသည်။ Timeline သည် parse လုပ်ထားသော တန်ဖိုးနှင့် field ၏ မူလရင်းမြစ်ကို ထိန်းထားသော်လည်း မရှိသော endpoint time zone ကို ပြန်လည်မရယူနိုင်သကဲ့သို့ upstream ပြောင်းလဲမှုကိုလည်း အတည်မပြုနိုင်ပါ။ ထိုကဲ့သို့သော အချိန်ကို host များအကြား တိကျစွာ ချိတ်ဆက်စစ်ဆေးရန် မသုံးမီ raw artifact နှင့် endpoint setting များကို စစ်ဆေးအတည်ပြုပါ။

Artifact က time zone ကို သတ်မှတ်မထားသော naive timestamp ကို UTC အဖြစ် တိတ်တဆိတ် မသတ်မှတ်ဘဲ diagnostic အဖြစ် အစီရင်ခံပါသည်။ အထောက်အထားက ခိုင်လုံသော အကြောင်းပြချက် ပေးနိုင်ပါက အပေါင်း/အနှုတ် လက္ခဏာပါသော ပုံသေ offset ကို ပေးပါ:

```bash
juntei super-timeline -f Amcache.jsonl --input-artifact amcache \
  --assume-offset "+09:00" -o Timeline.csv
```

ရက်စွဲသာပါသော တန်ဖိုးများအတွက်လည်း ထို option လိုအပ်ပါသည်။ ၎င်းတို့၏ သန်းခေါင်ယံ anchor ကို **ရက်အဆင့် တိကျမှု (day precision)** ရှိသော ယူဆချက်အဖြစ် label တပ်ပါသည်။ ၎င်းသည် သန်းခေါင်ယံတွင် လေ့လာတွေ့ရှိခဲ့သော event မဟုတ်ပါ။ Time zone အမည်များနှင့် daylight saving စည်းမျဉ်းများကို လက်မခံပါ။ မရှိသော၊ သုည၊ မမှန်ကန်သော နှင့် အပိုင်းအခြား ပြင်ပရှိ တန်ဖိုးများသည် ဖန်တီးထားသော (synthetic) epoch event များကို မထုတ်ပေးပါ။ တိကျစွာ မှတ်တမ်းတင်ထားသော မှန်ကန်သည့် 1970 ရက်စွဲကိုမူ ထိန်းထားပါသည်။

ရွေးချယ်နိုင်သော category နှစ်ခုကို မူလအားဖြင့် ဖယ်ထုတ်ထားပါသည်:

- `--include-collection-times`: စုဆောင်းမှု/ခွဲခြမ်းစိတ်ဖြာမှု filesystem ပေါ်ရှိ LNK/Jump List source ဖိုင် metadata။ ဤ event များကို endpoint လှုပ်ရှားမှုနှင့် ခွဲခြားရန် `collection_filesystem` ဟု label တပ်ပါသည်။
- `--include-interpreted-times`: registry reader က FILETIME အဖြစ် ပြသထားသော generic registry QWORD value များ။ ထိုကဲ့သို့ value အများအပြားသည် အမှန်တကယ်တွင် counter များ ဖြစ်သောကြောင့် ၎င်းတို့ကို `interpreted_value` ဟု label တပ်ပါသည်။

## တွဲဖက် အစီရင်ခံစာများ { #companion-reports }

`SuperTimeline.csv` အတွက် ဤညွှန်ကြားချက်သည် အောက်ပါဖိုင်များကိုလည်း ရေးပါသည်:

- `SuperTimeline.csv-Untimed.jsonl`: အသုံးပြုနိုင်သော map လုပ်ထားသည့် event timestamp မရှိသော native record တည်နေရာများ။
- `SuperTimeline.csv-Diagnostics.jsonl`: မမှန်ကန်သော/အဓိပ္ပာယ် မရေရာသော အချိန်များ၊ ရည်ရွယ်ချက်ရှိရှိ ကျော်ထားသော မျှဝေ metadata နှင့် မူလ parse လုပ်ထားသော record ကို ကြည့်ရန် လိုအပ်သည့် fidelity ကန့်သတ်ချက်များ။
- `SuperTimeline.csv-Summary.json`: input ဖိုင်များ၊ artifact အလိုက် အရေအတွက်များ၊ filter လုပ်ထားသော/candidate/ပေါင်းစည်းထားသော event များ၊ output အရေအတွက်၊ worker များ၊ sorting ဖွင့်ထားခြင်း ရှိ/မရှိ၊ sort budget/chunk/pass များနှင့် ကြာချိန်။

တွဲဖက်ဖိုင်အမည်များတွင် output ဖိုင်အမည် အပြည့်အစုံ ပါဝင်ပါသည်။ ဥပမာ `SuperTimeline.jsonl` တွင် ၎င်း၏ကိုယ်ပိုင် `SuperTimeline.jsonl-Summary.json` နှင့် သက်ဆိုင်ရာ untimed/diagnostic အစီရင်ခံစာများ ရှိပါသည်။ Stem တူသော CSV နှင့် JSONL export များ အတူတကွ ရှိနိုင်ပါသည်။ ရှိပြီးသား ရလဒ်အစုကို အစားထိုးမှသာ `-C` လိုအပ်ပါသည်။

ခိုင်လုံသော time zone ယူဆချက်ကို စောင့်ဆိုင်းနေသော မှန်ကန်သည့် ရက်စွဲများသည် `InvalidTimestamps` ကို မဟုတ်ဘဲ `UnresolvedTimeZones` ကို တိုးစေပါသည်။ နှစ်မျိုးလုံးကို diagnostic များတွင် ခြေရာခံနိုင်ပြီး မည်သည့် timestamp ကိုမျှ တီထွင်ဖန်တီးခြင်း မပြုပါ။

တောင်းဆိုထားသော အပိုင်းအခြား ပြင်ပတွင် မှန်ကန်သော အချိန်ရှိသည့် record များကို untimed အဖြစ် မဟုတ်ဘဲ filter လုပ်ထားသည်များအဖြစ် ရေတွက်ပါသည်။ Candidate event အရေအတွက်များသည် ထပ်နေသည်များ ပေါင်းစည်းခြင်း မပြုမီ အရေအတွက်များ ဖြစ်ပါသည်။ `Sorted` နှင့် `DuplicateConsolidation` တို့သည် `--sort` ကို ဖွင့်ထားခဲ့ခြင်း ရှိ/မရှိကို မှတ်တမ်းတင်ပါသည်။ ၎င်းမပါလျှင် `DuplicateEvents`၊ `LargestSerializedSortRowBytes`၊ `InitialSortChunks` နှင့် `MergePasses` တို့သည် သုည ဖြစ်ပြီး sort buffer ကို allocate မလုပ်သောကြောင့် `SortMemoryBudgetMiB` သည် `null` ဖြစ်ပါသည်။ `Complete: true` ဆိုသည်မှာ ရွေးချယ်ထားသော parse လုပ်ပြီး input အားလုံးကို အောင်မြင်စွာ process လုပ်ခဲ့သည်ဟု ဆိုလိုပါသည်။ Artifact တိုင်းကို စုဆောင်းခဲ့သည်၊ field တိုင်းတွင် timestamp ပါသည် သို့မဟုတ် တိုက်ခိုက်မှုတိုင်းကို ရှာဖွေတွေ့ရှိနိုင်သည်ဟု မဆိုလိုပါ။

CSV export များသည် nested metadata ကို flatten လုပ်နိုင်ပါသည်: System Info profile များ၊ USB အသုံးပြုသူ mount point များ၊ shell item များနှင့် အခြား ဖွဲ့စည်းထားသော context များသည် native JSON/JSONL တွင် ပိုမို ပြည့်စုံနိုင်ပါသည်။ ထို field များ အရေးပါသည့်အခါ native input ကို အသုံးပြုပါ။ CSV export က ချန်လှပ်ခဲ့သော ဒေတာကို timeline က ပြန်လည် တည်ဆောက်၍ မရပါ။ မပံ့ပိုးသော artifact ID များနှင့် ပုံစံမမှန်သော input ဖိုင်များသည် တစ်စိတ်တစ်ပိုင်း ရလဒ်များပါသော အောင်မြင်မှုအဖြစ် မပြီးဆုံးဘဲ error ဖြစ်စေပါသည်။

## စွမ်းဆောင်ရည်နှင့် output ဘေးကင်းမှု { #performance-and-output-safety }

Input များကို stream လုပ်ပြီး အပြိုင် process လုပ်ပါသည်။ မူလအားဖြင့် worker တစ်ခုစီသည် format လုပ်ပြီးသော event fragment တစ်ခုကို ရေးပါသည်။ Fragment များကို စီခြင်း သို့မဟုတ် ၎င်းတို့၏ event ဖွဲ့စည်းပုံများကို ပြန်ဖတ်ခြင်း မပြုဘဲ input ဖိုင် အစီအစဉ်အတိုင်း ဆက်ပေါင်းပါသည်။ ဖိုင်အများအပြားကို process လုပ်သည့်အခါတွင်ပင် CSV တွင် header တစ်ခုသာ ရှိပါသည်။ Output အစီအစဉ်သည် worker အရေအတွက် မည်မျှပင်ဖြစ်စေ အမြဲ တူညီ (deterministic) ပါသည်။ ၎င်းသည် အချိန်အစဉ်လိုက် ဖြစ်သည်ဟု မဆိုလိုပါ။

`Input files` သည် ရွေးချယ်ထားသော ခွဲခြမ်းစိတ်ဖြာထားသည့် ထည့်သွင်းမှုများ၏ ပေါင်းစပ်အရွယ်အစားနှင့် Hayabusa အချိန်ဇယားများကို ပြသည်။ `Results saved` သည် သိမ်းဆည်းထားသော အချိန်လိုင်းဖိုင်၏ အရွယ်အစားကို ပြသည်။ နှစ်ခုလုံးသည် တူညီသော လူသားဖတ်နိုင်သော `KiB`/`MiB`/`GiB` ယူနစ်များကို `auto-parse` အဖြစ် အသုံးပြုသည်။

`--sort` ဖြင့် buffer များကို private ယာယီ ဖိုင်တွဲသို့ spill လုပ်ပြီးနောက် အကန့်အသတ်ရှိသော merge pass များက အချိန်အစဉ်လိုက် output ကို ထုတ်ပေးပါသည်။ စီရာတွင် 1970 မတိုင်မီ ရက်စွဲများ အပါအဝင် integer စက္ကန့်နှင့် nanosecond အပြည့်အစုံကို အသုံးပြုပါသည်။ အချိန်တူသော event များ၏ အစီအစဉ်သည် worker အရေအတွက်နှင့် sort budget မည်သို့ပင်ဖြစ်စေ deterministic ဖြစ်ပါသည်။ Progress bar သည် ထပ်ခါထပ်ခါ pass များ အပါအဝင် ကြားဖြတ် merge နှင့် နောက်ဆုံး merge များအတွင်း process လုပ်သော event တစ်ခုစီအတွက် ရှေ့သို့ တိုးပါသည်။ ၎င်းကို terminal တွင် ပြသပြီး output ကို redirect လုပ်ထားသည့်အခါ ဖျောက်ထားပါသည်။

`--memory-limit` သည် `--sort` နှင့်သာ သက်ဆိုင်ပါသည်: ၎င်းသည် စုစုပေါင်း **sort-buffer budget** ဖြစ်ပြီး process RSS ကန့်သတ်ချက် မဟုတ်ပါ။ Input record များ၊ JSON object/string overhead၊ CSV buffer များနှင့် merge record များအတွက် memory ထပ်မံ လိုအပ်ပါသည်။ Budget သေးငယ်ပါက worker အရေအတွက်ကို လျှော့ချပါသည်။ `--max-record-size` သည် parse လုပ်ထားသော input record တစ်ခု၏ အရွယ်အစားကို ကန့်သတ်ပါသည် (မူလ 16 MiB)။ Pretty JSON array/အစီရင်ခံစာ တစ်ခုကို ထိပ်တန်းအဆင့် input document တစ်ခုအဖြစ် ရေတွက်ပါသည်။ ပိုကြီးသော document များအတွက် ကန့်သတ်ချက်ကို မြှင့်ရန် သို့မဟုတ် ပို၍ကောင်းသည်မှာ JSONL ကို အသုံးပြုရန် လိုအပ်နိုင်ပါသည်။ CSV parser ၏ read-ahead သည်လည်း ဤကန့်သတ်ချက်ထဲသို့ ပါဝင်ပါသည်။

ယာယီ disk နေရာသည် နောက်ဆုံး ရလဒ်များအပြင် format လုပ်ထားသော event fragment များ (မူလ) သို့မဟုတ် ချဲ့ထွင်ထားသော sort record များနှင့် merge pass များ (`--sort`) ကို ဆံ့ရပါမည်။ Mode နှစ်ခုလုံးတွင် ယာယီ output staging နှင့် မအောင်မြင်သော input များ၏ ရလဒ်ကို publish မလုပ်မိစေရန် ကာကွယ်မှုကို ဆက်လက် ထားရှိပါသည်။ Record တစ်ခုမှ event အများအပြား ထွက်ပေါ်နိုင်သောကြောင့် timeline သည် input ထက် များစွာ ကြီးနိုင်ပါသည်။ သင့်လျော်သည့်အခါ အချိန်အပိုင်းအခြား/artifact များကို filter လုပ်ပြီး လုံလောက်သော နေရာလွတ်ရှိသည့် ယာယီ volume ကို ရွေးချယ်ပါ။

Ctrl+C သည် cooperative နည်းဖြင့် ပယ်ဖျက်ပြီး private sort/staging ဖိုင်တွဲများကို ဖယ်ရှားကာ code `130` ဖြင့် ထွက်ပါသည်။ Linux နှင့် macOS တွင် SIGTERM နှင့် SIGHUP အတွက်လည်း ဤအတိုင်း ဖြစ်ပါသည်။ Publish မစတင်မီ ဖတ်ခြင်း၊ normalize လုပ်ခြင်း၊ merge လုပ်ခြင်းနှင့် အစီရင်ခံစာ ရေးခြင်း ပြုလုပ်နေစဉ် ပယ်ဖျက်မှုကို စစ်ဆေးပါသည်။ ဖမ်းယူ၍ မရသော termination သို့မဟုတ် လျှပ်စစ်ဓာတ်အား ပြတ်တောက်ခြင်းကြောင့် ယာယီဖိုင်များ ကျန်ရှိနိုင်ပါသေးသည်။

ရှိပြီးသား output များအတွက် `-C` လိုအပ်ပါသည်။ Timeline နှင့် တွဲဖက်ဖိုင်များသည် input များ၊ manifest များ၊ ကိုးကားထားသော local အထောက်အထား သို့မဟုတ် ၎င်းတို့၏ ကာကွယ်ထားသော အထောက်အထား ဖိုင်တွဲများကို overwrite မလုပ်ရပါ။ Input error များ သို့မဟုတ် sort မအောင်မြင်မှုများသည် ရှိပြီးသား output များကို မထိခိုက်စေပါ။ ပြီးဆုံးသော ဖိုင်များကို ရည်မှန်းထားသော နေရာ၏ ဘေးတွင် staging လုပ်ပြီး တစ်ခုချင်း အစားထိုးကာ timeline ကို နောက်ဆုံးမှ publish လုပ်ပါသည်။ ၎င်းသည် ဖိုင်လေးခုလုံးကို လွှမ်းခြုံသော transaction **မဟုတ်ပါ**: နောက်ဆုံး publish လုပ်စဉ် I/O မအောင်မြင်မှု ဖြစ်ပါက တွဲဖက် အစီရင်ခံစာ အချို့ အစားထိုးပြီးသား ဖြစ်နေနိုင်ပါသည်။ ယခင် ပြည့်စုံသော export ကို ထိန်းထားရန် အရေးကြီးပါက run တစ်ကြိမ်စီအတွက် output basename အသစ်ကို အသုံးပြုပါ။

## နောက်ထပ် အသေးစိတ်များ { #further-details }

- [Timesketch CSV/JSONL import](https://timesketch.org/guides/user/import-from-json-csv/)
- [Plaso timestamp ချဲ့ထွင်ခြင်းနှင့် legacy CSV](https://plaso.readthedocs.io/en/latest/sources/user/Output-format-l2tcsv.html)
- [Plaso output နှင့် formatting](https://plaso.readthedocs.io/en/latest/sources/user/Output-and-formatting.html)
- [အထောက်အထားနှင့် အဓိပ္ပာယ်ဖော်ခြင်း](../overview/evidence-model.md)
- [ထုတ်ယူမှု ပုံစံများ](../getting-started/output-formats.md)

လက်ရှိ Juntei input များသည် `Comp`၊ `Filename`၊ `Args` နှင့် `Attr` ကဲ့သို့ အတိုချုံ့ထားသော label များကို အသုံးပြုပါသည်။ MFT CSV သည် လက်ရှိ `File`၊ `SN`၊ `ParentEntryNum`၊ `ParentSN`၊ `Type`၊ `USN`၊ `LogfileSN` နှင့် `HasADS` header များကိုလည်း လက်ခံပြီး sequence number generation များနှင့် DOS/ADS ကွဲပြားချက်များကို မပျက်မစီး ထိန်းထားပါသည်။ ယခင် export label များကိုလည်း ဆက်လက် လက်ခံပြီး `SourceField` သည် အမှန်တကယ် input ထဲရှိ စာလုံးပေါင်းကို ထိန်းထားပါသည်။ Hayabusa input သည် ၎င်း၏ကိုယ်ပိုင် `Computer` field ကို ထိန်းထားပါသည်။ Boot sector နှင့် security descriptor metadata သည် event timestamp ကို မပေးသောကြောင့် ၎င်းကို untimed အစီရင်ခံစာတွင် ထိန်းထားပါသည်။

## ပိုမိုသစ်သော multi-table parser များ { #newer-multi-table-parsers }

Native JSON/JSONL တွင် table အမည်တစ်ခုစီ ပါဝင်ပါသည်။ Standalone CSV အတွက် ဘရောက်ဇာ၊ WebCache နှင့် Outlook adapter များသည် column schema မှ ၎င်း၏ table ကို ခွဲခြားသိရှိသောကြောင့် CSV ကို အမည်ပြောင်းခြင်း သို့မဟုတ် အလိုအလျောက် parsing ၏ source suffix များကို အသုံးပြုခြင်းက timestamp အဓိပ္ပာယ်ဖော်ပုံကို မပြောင်းလဲစေပါ။ UAL သည် ၎င်း၏ တိကျစွာ သိရှိထားသော timestamp field များကို ထိန်းထားပါသည်။ Access counter များနှင့် မသက်ဆိုင်သော ရက်စွဲနှင့်တူသည့် စာသားများကို ရက်စွဲများအဖြစ် ခန့်မှန်းယူဆခြင်း မပြုပါ။

Outlook property row များကို ၎င်းတို့၏ ကြေညာထားသော type သည် time သို့မဟုတ် multiple-time type ဖြစ်မှသာ project လုပ်ပါသည်။ ရက်စွဲပါဝင်သော string သည် string အဖြစ်သာ ကျန်ရှိပါသည်။ Embedded message များသည် ၎င်းတို့၏ item ID များနှင့် parent reference များကို ထိန်းထားပြီး ပူးတွဲဖိုင်ကို extract လုပ်ခဲ့သည့်အချိန်များကို သိမ်းဆည်းထားသော ပူးတွဲဖိုင် timestamp များအစား အစားထိုး အသုံးမပြုပါ။ အချိန်မပါသော file၊ recipient၊ contact သို့မဟုတ် property row များကို native output နှင့် untimed အစီရင်ခံစာတွင် ခြေရာခံနိုင်ပါသည်။

ဤ parser များအတွက်လည်း တူညီသော CSV နှင့် JSONL output schema များကို အသုံးပြုပါသည်။ Artifact အသစ်တစ်ခု ထပ်ထည့်ခြင်းက CSV column ရာနှင့်ချီ မတိုးစေပါ။ Artifact အလိုက် context ကို `Details` တွင်သာ ထားပါသည်။
