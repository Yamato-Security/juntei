# ထုတ်ယူမှု ပုံစံများ { #output-formats }

## Terminal အနှစ်ချုပ်များ { #terminal-summaries }

`-o` သို့မဟုတ် ဖွဲ့စည်းထားသော `-t` မပါလျှင် ညွှန်ကြားချက်များသည် ယေဘုယျအားဖြင့် အရေအတွက်များ၊ အချိန်အပိုင်းအခြားများ၊ ပျက်စီးမှုနှင့်
တန်ဖိုးမြင့် တွေ့ရှိချက်များကို ပြသပါသည်။ အနှစ်ချုပ်များသည် ကြီးမားသော output မဖန်တီးမီ နယ်ပယ်ကို နားလည်ရန် ကူညီပါသည်။

## CSV { #csv }

CSV သည် မူလ သိမ်းဆည်းမည့် ပုံစံ ဖြစ်ပါသည်။ ၎င်းသည် spreadsheet tool များ၊ SQL import နှင့် timeline viewer များအတွက်
အဆင်ပြေပါသည်။ Parser အချို့သည် ၎င်းတို့၏ artifact များတွင် မတူညီသော record type များ ပါဝင်သောကြောင့် table အများအပြားကို
ဖန်တီးပါသည် (Amcache၊ Jump Lists နှင့် SRUM)။

!!! warning "အထောက်အထား column များကို စာသားအဖြစ် သဘောထားပါ"
    Path သို့မဟုတ် command line တစ်ခုသည် `=`၊ `+`၊ `-` သို့မဟုတ် `@` ဖြင့် စတင်နိုင်ပါသည်။ Spreadsheet software က ၎င်းကို
    formula အဖြစ် အဓိပ္ပာယ်ဖော်နိုင်ပါသည်။ Field အားလုံးကို စာသားအဖြစ် import လုပ်ပါ သို့မဟုတ် forensic အတွက် ဘေးကင်းသော CSV
    viewer ကို အသုံးပြုပါ။ Juntei သည် အထောက်အထား string များကို ပြောင်းလဲခြင်းမပြုဘဲ ထိန်းသိမ်းပါသည်။

## Field အမည်များ { #field-names }

CSV column header များနှင့် JSON/JSONL field အမည်များသည် Amcache နှင့် `auto-parse` manifest များ အပါအဝင်
PascalCase ကို အသုံးပြုပါသည်။ အတိုကောက်များသည် စာလုံးကြီးအတိုင်း ရှိပါသည်: `ProcessID`, `TimestampUTC`, `UserSID`,
`SHA256`။ အတိုကောက်နောက်တွင် လိုက်သော စကားလုံးကို hyphen ဖြင့် ခွဲထားပါသည်: `IP-Address`, `MFT-Entry`,
`USN-Journal`။ `jq` ဖြင့် hyphen ပါသော field ကို quote လုပ်ပါ: `.["IP-Address"]`။

မူလ MFT CSV သည် `EntryNum`, `Ext`, `Dir`, `HasADS`,
`Size`, `Created (SI)` နှင့် `Created (FN)` ကဲ့သို့ ပိုတိုပြီး ဖတ်ရလွယ်သော heading များကို အသုံးပြုပါသည်။ ၎င်း၏ CSV schema
အပြည့်အစုံအတွက် [MFT field ကိုးကားချက်](../artifacts/mft.md) ကို ကြည့်ပါ။ Timestamp အတွဲများသည် MACB အစီအစဉ်
(DataModified, Accessed, MetadataChanged, Created) ကို လိုက်နာပြီး SI သည် FN ၏ ရှေ့တွင် ရှိပါသည်။ CSV tool များတွင်
space သို့မဟုတ် သင်္ကေတ ပါဝင်သော heading များကို quote လုပ်ပါ။

အထောက်အထား တန်ဖိုးများသည် ၎င်းတို့၏ မူလ စာလုံးပေါင်းကို ထိန်းထားပါသည်။ `Details`/`DataRaw` အောက်ရှိ registry
value အမည်များနှင့် `ZoneIdentifier` အောက်ရှိ `Zone.Identifier` အမည်များကဲ့သို့ ရင်းမြစ်မှ ပေးသော dictionary key များကိုလည်း
မပြောင်းလဲဘဲ ထားပါသည်။ SRUM ၏ `ValueState` နှင့် `Raw` dictionary များသည် ၎င်းတို့၏ ဒေတာ column များနှင့် တူညီသော
label များကို အသုံးပြုပါသည်။ Normalization တိုက်မိမှုကို field တစ်ခုကို တိတ်တဆိတ် ဖယ်ရှားခြင်းမပြုဘဲ အစီရင်ခံပါသည်။

၎င်းသည် ယခင် export schema များကို ပြောင်းလဲပါသည်။ ရှိပြီးသား ရလဒ်များကို ပြန်လည်ထုတ်လုပ်ပြီး သိမ်းဆည်းထားသော query များကို
update လုပ်ပါ (ဥပမာ `.record_type` သည် `.RecordType` ဖြစ်လာပြီး Amcache `full_path` သည် `FullPath` ဖြစ်လာပါသည်)။
လူဖတ်ရန် စာသား အစီရင်ခံစာများနှင့် input/configuration ပုံစံများသည် ၎င်းတို့၏ ရှိပြီးသား စည်းမျဉ်းများကို ထိန်းထားပါသည်။

## အတိုချုံ့ထားသော field အမည်များ { #abbreviated-field-names }

အောက်ပါ label များသည် အလိုအလျောက် parsing နှင့် manifest များ အပါအဝင် CSV၊ JSON နှင့် JSONL export များနှင့်
သက်ဆိုင်ပါသည်။ ရလဒ်အဟောင်းများကို ပြန်လည်ထုတ်လုပ်သည့်အခါ သိမ်းဆည်းထားသော `jq`၊ `xsv` နှင့် spreadsheet filter များကို update လုပ်ပါ။
Output label များသာ ပြောင်းလဲပြီး အထောက်အထား တန်ဖိုးများနှင့် raw registry-value အမည်များသည် ၎င်းတို့၏ မူလ စာလုံးပေါင်းကို ထိန်းထားပါသည်။

| ယခင် label | လက်ရှိ label |
|---|---|
| `ADS` | `HasADS` |
| `Application` | `App` |
| `ApplicationCompany` | `AppCompany` |
| `ApplicationCompanyRaw` | `AppCompanyRaw` |
| `ApplicationPath` | `AppPath` |
| `Arguments` | `Args` |
| `Attributes` | `Attr` |
| `AttributeType` | `AttrType` |
| `BytesRecvd` | `BytesReceived` |
| `Computer` | `Comp` |
| `ComputerName` | `CompName` |
| `ComputerNameLastWrittenUTC` | `CompNameLastWrittenUTC` |
| `Directories` | `Dir` |
| `FileName` | `Filename` |
| `MicrosecondZeros` | `ZeroU-Secs` |
| `SI-BeforeFN` | `SI<FN` |
| `UserName` | `User` |
| `Username` | `User` |
| `UserNameSource` | `UserSource` |
| `Users` | `User` |

MFT CSV တွင် သီးသန့်ကော်လံအမည်များဖြစ်သော `File` (Filename), `SN` (Sequence Number), `ParentEntryNum` (ParentEntryNumber), `ParentSN` (Parent Sequence Number), `USN` (Update Sequence Number), `LogfileSN` (Logfile Sequence Number), `SID` (Security ID), `Type` (Name Type) နှင့် `RefCnt` (Reference Count) ကို သုံးပါသည်။ `auto-parse` ၏ MFT CSV များလည်း ထိုအမည်များကို သုံးပါသည်။ [MFT output fields](../artifacts/mft.md#csv-output-fields) ကို ကြည့်ပါ။

Native system-information JSON အချိန်သည် `CompNameLastWritten` ဖြစ်ပါသည် (CSV:
`CompNameLastWrittenUTC`)။ `HasADS` သည် Boolean indicator ဖြစ်ပြီး Prefetch ရှိ `Dir` တွင် မှတ်တမ်းတင်ထားသော
ဖိုင်တွဲစာရင်း ပါဝင်ပါသည်။ `User` သည် ၎င်း၏ artifact အလိုက် type ကို ထိန်းထားပါသည်: account အမည်အတွက် string၊
သို့မဟုတ် USB output တွင် ဆက်စပ်သော အသုံးပြုသူများ၏ list ဖြစ်ပါသည်။ ၎င်းသည် execute လုပ်သော identity ကို မဆိုလိုပါ။
Query များတွင် သင်္ကေတများကို quote လုပ်ပါ: `."SI<FN"` နှင့် `.["ZeroU-Secs"]`။

## JSON { #json }

JSON သည် ရလဒ်အပြည့်အစုံကို document တစ်ခုအဖြစ် သိမ်းဆည်းပါသည်။ ၎င်းသည် array များ၊ nested ဖွဲ့စည်းပုံများ၊ null များ၊
ကြီးမားသော integer များနှင့် flat CSV က အမြဲ ကိုယ်စားမပြုနိုင်သော ကွဲပြားချက်များကို ထိန်းသိမ်းပါသည်။

```bash
juntei parse-file lnk -f sample.lnk -o lnk.json -t json
```

## JSON Lines { #json-lines }

JSONL သည် စာကြောင်းတစ်ကြောင်းလျှင် လွတ်လပ်သော JSON object တစ်ခုကို ရေးသားပါသည်။ ၎င်းသည် stream ကောင်းစွာ လုပ်နိုင်ပြီး
`jq` နှင့် သဘာဝကျစွာ အလုပ်လုပ်ပါသည်။

```bash
juntei parse-registry services -d evidence/ -t jsonl \
  | jq -c 'select(.StartType == "automatic") | {Host, ServiceName, ImagePath}'
```

ဖွဲ့စည်းထားသော output ကို standard output သို့ ပို့သည့်အခါ Juntei သည် pipe က ဒေတာကိုသာ လက်ခံရရှိစေရန် progress နှင့်
message များကို standard error သို့ ပို့ပါသည်။

`auto-parse -j timeline.jsonl` သည် အမျိုးအစားခွဲထားသော CSV ရလဒ်များကို ထိန်းထားရင်း host အများအပြားပါသော collection
အပြည့်အစုံအတွက် ထပ်ဆောင်း JSONL dataset တစ်ခုကို ဖန်တီးပါသည်။ တစ်ခုချင်း ရလဒ်ပုံစံကို ပြောင်းရန် `-t json` သို့မဟုတ်
`-t jsonl` ကို ထည့်ပါ။ Record တစ်ခုစီတွင် `Comp` နှင့် `Artifact` field များ ပါဝင်သောကြောင့် SIEM ထည့်သွင်းပြီးနောက်တွင်လည်း
၎င်း၏ ရင်းမြစ် ရှင်းလင်းစွာ ရှိနေပါသည်။

## UTC နှင့် တိကျမှု { #utc-and-precision }

Juntei သည် parse လုပ်ထားသော အချိန်များကို ပုံစံက ခွင့်ပြုသည့်နေရာတွင် RFC 3339 ကို အသုံးပြု၍ UTC ဖြင့် format လုပ်ပါသည်။
Artifact အချို့တွင် 100-nanosecond တိကျမှု ရှိပြီး အချို့သည် စက္ကန့်များ၊ FAT အချိန်၊ စုစည်းမှု ကာလများ သို့မဟုတ် အချိန်လုံးဝ မရှိခြင်းကို
သိမ်းဆည်းပါသည်။ ဂဏန်းများသည် တိကျမှန်ကန်မှု မဟုတ်ပါ: ၎င်း၏ artifact စာမျက်နှာတွင် ဖော်ပြထားသော ရင်းမြစ် field ကို အမြဲ နားလည်ပါ။

UTC သည် Windows event-log ပုံစံ `2026-10-20T08:46:38.2001668Z` ကို အသုံးပြုပြီး ရရှိနိုင်သော ဒဿမ တိကျမှုကို
ထိန်းထားပါသည်။ `auto-parse -Z "+09:00"` သည် parse လုပ်ထားသော အခိုက်အတန့်များကို ၎င်း၏ ရလဒ်ပုံစံ အားလုံးနှင့် ရွေးချယ်နိုင်သော
`-j` export တွင် တိကျသော ပုံသေ offset ဖြင့် ပြသပါသည်။ Juntei ကို လည်ပတ်နေသော ကွန်ပျူတာ မည်သို့ပင်ဖြစ်စေ မူလတန်ဖိုးသည်
UTC ဖြစ်ပါသည်။ Raw တန်ဖိုးများနှင့် zone မပါသော wall clock များကို မှတ်တမ်းတင်ထားသည့်အတိုင်း ထားပါသည်။
[output time zone များ](../commands/auto-parse.md#output-time-zone) ကို ကြည့်ပါ။

## ပျက်စီးမှုနှင့် မရှိသော တန်ဖိုးများ { #damage-and-missing-values }

ဗလာ၊ `null`၊ `NA`၊ `Problems` နှင့် parser အလိုက် အခြေအနေ field များသည် အဓိပ္ပာယ်ရှိပါသည်။ ၎င်းတို့သည် artifact က
တန်ဖိုးကို မသိမ်းဆည်းခဲ့ခြင်း၊ record ပျက်စီးနေခြင်း၊ ပုံစံက ၎င်းကို မပံ့ပိုးခြင်း သို့မဟုတ် parser က ၎င်းကို ဖြေရှင်းမရခြင်းကို
ဆိုလိုနိုင်ပါသည်။ ဗလာတန်ဖိုး အားလုံးကို သုညဖြင့် အစားမထိုးပါနှင့်။

## Normalize လုပ်ထားသော timeline { #normalized-timeline }

[`super-timeline`](../commands/super-timeline.md) သည် label တပ်ထားသော timestamp အဓိပ္ပာယ်များနှင့် source reference များ
ပါဝင်သည့် ဘုံ CSV/JSONL event schema တစ်ခုကို ဖန်တီးပါသည်။ Output ကို မူလအားဖြင့် စီထားခြင်း မရှိပါ။ `--sort`
သည် အချိန်အစဉ်လိုက် စီခြင်းနှင့် ထပ်နေသည်များ ပေါင်းစည်းခြင်းကို ဖွင့်ပေးပါသည်။ ၎င်းသည် Timesketch import profile
ကိုလည်း ပံ့ပိုးပါသည်။ Native parser export များနှင့် `auto-parse -j` တို့သည် ၎င်းတို့၏ ရှိပြီးသား schema များကို ထိန်းထားပါသည်။
