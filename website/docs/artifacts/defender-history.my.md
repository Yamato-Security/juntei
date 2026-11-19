# Microsoft Defender ရှာဖွေတွေ့ရှိမှု မှတ်တမ်း { #microsoft-defender-detection-history }

## မှတ်တမ်းတွင် သိမ်းဆည်းသည့်အရာ { #what-the-history-records }

Microsoft Defender သည် detection (ရှာဖွေတွေ့ရှိမှု) တစ်ခုစီအတွက် detection history ဖိုင်တစ်ခု ရေးသားသည်။ ဖိုင်တစ်ခုစီတွင်
အောက်ပါတို့ကို မှတ်တမ်းတင်ထားသည်-

- ခြိမ်းခြောက်မှု၏ identifier၊ အမည်၊ severity နှင့် category၊
- Defender ဘာလုပ်ခဲ့သည်နှင့် ၎င်း အောင်မြင်ခဲ့/မအောင်မြင်ခဲ့၊
- ခြိမ်းခြောက်မှု run နေခဲ့/မနေခဲ့၊
- ပါဝင်ပတ်သက်သော အသုံးပြုသူနှင့် process၊ ၎င်းကို မည်သည့်အရာက detect လုပ်ခဲ့သည်၊
- quarantine entry (ရှိပါက)၊
- ၎င်းကို တွေ့ရှိခဲ့သော resource တိုင်း- ဖိုင်များ၊ archive member များ၊ download များ၊ process များ၊ registry value များ၊
  ဝန်ဆောင်မှုများ၊ command line များနှင့် behavior များ။

Defender သည် ၎င်း quarantine မလုပ်ခဲ့ဖူးသော ခြိမ်းခြောက်မှုများအတွက်လည်း ဤဖိုင်များကို ရေးသားသည်- detect သာ လုပ်ခဲ့သည့်
ခြိမ်းခြောက်မှုများ၊ ခွင့်ပြုခဲ့သည့် ခြိမ်းခြောက်မှုများနှင့် ဖယ်ရှားရန် မအောင်မြင်ခဲ့သည့် ခြိမ်းခြောက်မှုများ။ Windows Security ၏
protection history နှင့် WMI ၏ `MSFT_MpThreatDetection` class (`Get-MpThreatDetection`) တို့သည် ဤ record များကို
ပြသသည်။ Orange Cyberdefense က ဖိုင်တစ်ခုအတွင်းရှိ value များကို ပြောင်းလဲသောအခါ WMI သည် ပြောင်းလဲထားသော value များကို
ဖော်ပြခဲ့သည်။

<span class="evidence-strength">အကောင်းဆုံး ကောက်ချက်:</span> Defender သည် ဖော်ပြထားသော resource များတွင် အမည်ဖော်ပြထားသော
ခြိမ်းခြောက်မှု၏ detection ကို မှတ်တမ်းတင်ထားသော status နှင့် အချိန်များဖြင့် မှတ်တမ်းတင်ခဲ့သည်။ ခြိမ်းခြောက်မှု အမည်သည်
ထုတ်ကုန်၏ စီရင်ချက် ဖြစ်သည်။ Detection record တစ်ခုတည်းက ခြိမ်းခြောက်မှု run ခဲ့ကြောင်း သို့မဟုတ် ၎င်းကို လုံးဝ
ဖယ်ရှားခဲ့ကြောင်း မပြပါ။

## တည်နေရာနှင့် စုဆောင်းခြင်း { #location-and-collection }

```text
C:\ProgramData\Microsoft\Windows Defender\Scans\History\Service\DetectionHistory\<nn>\<GUID>
```

ဖိုင်တစ်ခုစီကို ၎င်း၏ `DetectionID` အတိုင်း အမည်ပေးထားသည်။ `DetectionHistory` ဖိုင်တွဲ တစ်ခုလုံးကို စုဆောင်းပါ။ Directory
input နှင့် `auto-parse` သည် `DetectionHistory` ဖိုင်တွဲ အတွင်း မည်သည့်အဆင့်တွင်မဆို detection history ဖိုင်ကဲ့သို့
စတင်သော ဖိုင်တိုင်းကို ဖတ်သည်။ ထိုသို့သော ဖိုင်တွဲ ပြင်ပတွင် ထားသော မိတ္တူအတွက် `-f` ကို အသုံးပြုပါ။ Defender သည်
သတ်မှတ်နိုင်သော ရက်အရေအတွက် ကုန်လွန်ပြီးနောက် ၎င်း၏ scan history မှ item များကို ဖယ်ရှားသည် (`Set-MpPreference` ရှိ
`ScanPurgeItemsAfterDelay`)။ ထို့ကြောင့် ပိုဟောင်းသော detection များ ပျောက်သွားနိုင်သည်။

Quarantine သည် သီးခြား artifact တစ်ခု ဖြစ်သည်- [Defender သီးခြားခွဲထားသော ဒေတာ](defender-quarantine.md) ကို ကြည့်ပါ။
Detection တစ်ခု၏ `QuarantineID` သည် quarantine output ၏ `EntryID` နှင့် တူညီသော စာလုံးပေါင်း (စာလုံးကြီး၊
တွန့်ကွင်း မပါ) ကို အသုံးပြုသောကြောင့် နှစ်ခုကို join လုပ်နိုင်သည်။

## စိစစ်သူများ သတိပြုရန် { #analyst-cautions }

- Detection၊ remediation ကြိုးပမ်းမှုနှင့် အောင်မြင်သော ဖယ်ရှားမှုတို့သည် သီးခြား အခြေအနေများ ဖြစ်သည်။ `ThreatStatus`
  (`Detected`၊ `Quarantined`၊ `Removed`၊ `Allowed`၊ `RemoveFailed`၊ …) ကို `ThreatStatusErrorCode` နှင့်အတူ
  ဖတ်ပါ။ နောက်ဆုံး status ပြောင်းလဲမှု အောင်မြင်ခဲ့ပါက ၎င်းသည် 0 ဖြစ်သည်။
- `ExecutionStatus` သည် detection အချိန်က Defender ၏ အမြင် ဖြစ်သည် (`Blocked`၊ `Allowed`၊ `Executing`၊
  `NotExecuting`၊ `Unknown`)။ ၎င်းသည် ခြိမ်းခြောက်မှု run ခဲ့ကြောင်း သက်သေ မဟုတ်သလို ဘယ်တော့မှ မ run ခဲ့ကြောင်း
  သက်သေလည်း မဟုတ်ပါ။ Prefetch၊ Amcache၊ event log များနှင့် process telemetry တို့ဖြင့် တိုက်ဆိုင်စစ်ဆေးပါ။
- `ThreatName` သည် ထိုအချိန်က signature အမည် ဖြစ်သည်။ False positive များနှင့် နောက်ပိုင်း အမျိုးအစား ပြန်လည်ခွဲခြားမှုများ
  ဖြစ်နိုင်သည်။ EICAR ကဲ့သို့ စမ်းသပ်ဖိုင်များကို တကယ့် ခြိမ်းခြောက်မှုများကဲ့သို့ ဖော်ပြသည်။
- `User` သည် Defender မှတ်တမ်းတင်ခဲ့သော `DomainUser` value ဖြစ်သည် (`UserSource` သည် `DomainUser` ဖြစ်သည်)။
  ၎င်းသည် ဗလာ၊ service account တစ်ခု သို့မဟုတ် ဖိုင်ကို ဖန်တီးခဲ့သူနှင့် မတူသော အသုံးပြုသူ ဖြစ်နိုင်သည်။
- Defender သည် ခြိမ်းခြောက်မှု တစ်ခုအတွက် ဖိုင်များစွာ ရေးနိုင်ပြီး Juntei သည် ၎င်းတို့ တစ်ခုစီကို ထိန်းသိမ်းသည်။ ကူးယူထားသော
  သို့မဟုတ် restore လုပ်ထားသော ဖိုင်ကဲ့သို့ ဖိုင်နှစ်ခုတွင် တူညီသော `DetectionID` ပင် ပါနိုင်သည်။ `DetectionID` နှင့်
  `SourceFile` နှစ်ခုလုံးကို တွဲ၍ အုပ်စုဖွဲ့ပါ။
- အဓိပ္ပာယ် မသိရသော value များကို အဓိပ္ပာယ်မဖော်ဘဲ ထိန်းသိမ်းထားသည်- ၎င်းတို့၏ section နှင့် position ကို
  `UnknownValues` တွင် တွေ့နိုင်သည်။

## Juntei ဖြင့် ခွဲခြမ်းစိတ်ဖြာခြင်း { #parse-with-juntei }

```text
Usage: juntei parse-file defender-history <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find detection history files
  -f, --file <FILE>      Parse one DetectionHistory\<nn>\<GUID> file

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
  -C, --clobber             Overwrite an existing output
```

```bash
juntei parse-file defender-history -d evidence/ -o defender-history.csv
juntei parse-file defender-history -d evidence/ -t jsonl \
  | jq '{InitialDetectionTime, ThreatName, ThreatStatus, ResourceType, Path, SHA256}'
```

မူလဖိုင်ပုံစံမှာ CSV ဖြစ်သည်။ `-t csv`၊ `-t json` သို့မဟုတ် `-t jsonl` ပါပြီး `-o` မပါလျှင် record များကို stdout သို့
ရေးသည်။ ရွေးချယ်စရာနှစ်ခုလုံး မပါလျှင် အနှစ်ချုပ်သာ ပြသည်။ ရှိပြီးသား output ကို ထပ်ရေးရန် `-C` ထည့်ပါ။ `-C`
ပါသော်လည်း output ကို input ဖိုင်တစ်ခုခုပေါ်သို့ ဘယ်တော့မှ ထပ်မရေးပါ။

Juntei သည် detection တစ်ခုစီ၏ resource တစ်ခုလျှင် record တစ်ခု ရေးသည်။ Resource မပါသော detection သည် ဗလာ
resource field များပါသော record တစ်ခုကို ရရှိသည်။ Detection တစ်ခု၏ record များသည် ၎င်း၏ `DetectionID` ကို
မျှဝေပြီး သိမ်းဆည်းထားသည့် အစီအစဉ်အတိုင်း `ResourceIndex` ဖြင့် နံပါတ်တပ်ထားသည်။

ပျက်စီးမှုက run ကို မရပ်တန့်စေပါ။ အဆုံးမတိုင်မီ ပြတ်တောက်နေသော ဖိုင်သည် ပျက်စီးမှု မတိုင်မီ ဖတ်ထားသော value များကို
ထိန်းသိမ်းထားပြီး `Problems` က ပျောက်နေသည့်အရာကို ဖော်ပြသည်။ `DetectionHistory` ဖိုင်တွဲ အတွင်းရှိ detection
history ဖိုင် မဟုတ်သော ဖိုင်ကို ဖော်ပြပြီး ကျော်သွားသည်။ အခြားဖိုင်များကို ဆက်လက် ဖတ်သည်။

## ထုတ်ယူမှု field များ { #output-fields }

| Field | အဓိပ္ပာယ်ဖော်ပုံ |
|---|---|
| `Record` | `resource` သို့မဟုတ် resource မပါသော detection အတွက် `detection`။ |
| `InitialDetectionTime` | Defender က ခြိမ်းခြောက်မှုကို ပထမဆုံး detect လုပ်ခဲ့သည့်အချိန် (UTC)။ ဖိုင်တွင် 0 ပါလျှင် ဗလာ။ အချိန်မဟုတ်သော value ကို `UnknownValues` တွင် ထိန်းသိမ်းထားသည်။ |
| `LastThreatStatusChangeTime` | ခြိမ်းခြောက်မှု၏ status နောက်ဆုံး ပြောင်းလဲခဲ့သည့်အချိန် (UTC)။ |
| `RemediationTime` | Defender က ခြိမ်းခြောက်မှုကို remediate လုပ်ခဲ့သည့်အချိန် (UTC)။ ၎င်းသည် ကြိုးပမ်းမှု၏ အချိန်ဖြစ်ပြီး ဖယ်ရှားခဲ့ကြောင်း သက်သေ မဟုတ်ပါ။ `ThreatStatus` ကို စစ်ဆေးပါ။ |
| `ThreatName`, `ThreatID` | Defender ၏ signature အမည်နှင့် နံပါတ် identifier (`Get-MpThreatCatalog`)။ |
| `Severity`, `SeverityID` | `Low`၊ `Moderate`၊ `High`၊ `Severe` သို့မဟုတ် `Unknown`၊ သိမ်းဆည်းထားသော နံပါတ်နှင့်အတူ။ |
| `Category`, `CategoryID` | `TROJAN`၊ `TOOL` သို့မဟုတ် `BEHAVIOR` ကဲ့သို့ ခြိမ်းခြောက်မှု category၊ သိမ်းဆည်းထားသော နံပါတ်နှင့်အတူ။ |
| `ThreatStatus`, `ThreatStatusID` | ခြိမ်းခြောက်မှုအတွက် Defender ၏ status။ စာတမ်းတွင် အမည်မဖော်ပြထားသော နံပါတ်ကို ဗလာ `ThreatStatus` ဖြင့် ထိန်းသိမ်းထားသည်။ |
| `ThreatStatusErrorCode` | နောက်ဆုံး status ပြောင်းလဲမှု၏ error code။ 0 ဆိုလျှင် ၎င်း အောင်မြင်ခဲ့သည်။ |
| `ExecutionStatus`, `ExecutionStatusID` | Detect လုပ်ခဲ့ချိန်တွင် Defender က ခြိမ်းခြောက်မှု run နေသည်ကို မြင်ခဲ့/မမြင်ခဲ့။ |
| `DetectionSource`, `DetectionSourceTypeID` | ၎င်းကို detect လုပ်ခဲ့သည့်အရာ- `Real-time`၊ `IOAV` (download များနှင့် attachment များ)၊ `User`၊ `System`၊ `AMSI` နှင့် အခြားများ။ |
| `User`, `UserSource` | Defender မှတ်တမ်းတင်ခဲ့သော `DomainUser` value နှင့် ၎င်း၏ အခြေခံ (`DomainUser`)။ ဖိုင်တွင် အသုံးပြုသူ အမည် မပါလျှင် ဗလာ။ |
| `Process` | ပါဝင်ပတ်သက်သော process (`ProcessName`)။ `Unknown` ဖြစ်နိုင်သည်။ |
| `ResourceIndex`, `ResourceCount` | ဤ resource ၏ position နှင့် detection အတွင်းရှိ resource အရေအတွက်။ |
| `ResourceType` | Defender အမည်ပေးထားသည့်အတိုင်း `file`၊ `containerfile`၊ `webfile`၊ `process`၊ `regkeyvalue`၊ `service`၊ `CmdLine`၊ `behavior`၊ `amsi` နှင့် အခြားများ။ |
| `Path` | Resource- path၊ archive member (`archive.zip->member`)၊ registry value၊ command line သို့မဟုတ် process။ `webfile` သည် ၎င်း၏ URL နှင့် download လုပ်ခဲ့သော process (`pid:…,ProcessStart:…`) ကိုလည်း ဖော်ပြသည်။ စာလုံးပေါင်း အတိအကျကို ထိန်းသိမ်းပါ။ |
| `ResourceFlags` | Hexadecimal ဖြင့် resource flag များ။ ၎င်းတို့၏ အဓိပ္ပာယ်ကို စာတမ်းတွင် မဖော်ပြထားပါ။ |
| `TrackingStartTime` | Resource ၏ `ThreatTrackingStartTime`၊ plaso က detection ၏ အချိန်ကို သတ်မှတ်သည့်နည်းအတိုင်း FILETIME (UTC) အဖြစ် ဖတ်သော integer။ |
| `SHA256`, `SHA1`, `MD5`, `TrackingSize` | Resource ၏ tracking ဒေတာမှ hash များနှင့် အရွယ်အစား။ ၎င်းတို့သည် Defender က ဖိုင်ကို scan လုပ်ခဲ့ချိန်က ဖိုင်ကို ဖော်ပြသည်။ |
| `TrackingID` | Resource ၏ `ThreatTrackingId`။ |
| `OtherTracking` | အခြား tracking value တိုင်း၊ သိမ်းဆည်းထားသည့် အစီအစဉ်အတိုင်း `Key=Value` အဖြစ်၊ ဥပမာ behavior detection များအတွက် `ThreatTrackingScanSource` သို့မဟုတ် `ThreatTrackingBMContextRich`။ |
| `QuarantineID` | Quarantine entry (`Quarantine\Entries\{GUID}`)။ ဖိုင်တွင် သုည GUID ပါလျှင် ဗလာ။ |
| `DetectionID` | Detection ၏ GUID ဖြစ်ပြီး ဖိုင်၏ အမည်လည်း ဖြစ်သည်။ မကိုက်ညီမှုကို `Problems` တွင် ဖော်ပြသည်။ |
| `MagicVersion` | Header ၏ `Magic.Version:1.2`။ ၎င်းမပါဘဲ ရေးထားသော ဖိုင်များတွင် ဗလာ၊ ဥပမာ Windows Server 2016 မှ ဖိုင်အချို့။ |
| `UnknownValues` | အဓိပ္ပာယ် မသိရသော value များကို `position=value` အဖြစ် (ဥပမာ `6.13=NT AUTHORITY\SYSTEM`)၊ status နောက်ရှိ counted list ကို `2.list` အဖြစ် ဖော်ပြသည်။ |
| `SourceFile` | ဖတ်ခဲ့သော ဖိုင်၊ input directory မှ relative path သို့မဟုတ် `-f` ဖြင့်ဆိုလျှင် ၎င်း၏ အမည်။ |
| `Problems` | ပျက်စီးမှု၊ ပျောက်နေသော value များ၊ မမျှော်လင့်ထားသော type များ သို့မဟုတ် မကိုက်ညီသော ဖိုင်အမည်။ ဖုံးကွယ်မထားပါနှင့်။ |

JSON နှင့် JSONL သည် type ပါသော value များဖြင့် တူညီသော field များကို ထိန်းသိမ်းသည်- နံပါတ်များသည် နံပါတ်အဖြစ် ကျန်ပြီး
ပျောက်နေသော value များသည် `null` ဖြစ်သည်။ `OtherTracking` အစား ၎င်းတို့တွင် `Tracking` ပါသည်- tracking value
တိုင်းကို Defender ၏ key၊ ၎င်း၏ type နှင့် ၎င်း၏ value တို့ဖြင့် ဖော်ပြသည်။ `UnknownValues` သည် type ပါသော position
စာရင်း ဖြစ်လာသည်။ `CountedList` တွင် counted list ပါသည်။ သိမ်းဆည်းထားသော FILETIME သည် ၎င်း၏ raw tick count ကို
၎င်းမှ ဖတ်ရသော အချိန်၏ ဘေးတွင် ထိန်းသိမ်းထားသည်။

## Timeline အပြုအမူ { #timeline-behavior }

`auto-parse` သည် record များကို `SecurityProducts/DefenderHistory.csv` (သို့မဟုတ် `.json`/`.jsonl`) အဖြစ်
သိမ်းဆည်းပြီး ပေါင်းစပ်ထားသော `-j` timeline ထဲသို့ ထည့်သည်။ `super-timeline` သည် resource တစ်ခုစီ၏ record ပေါ်ရှိ
`InitialDetectionTime`၊ `LastThreatStatusChangeTime` နှင့် `RemediationTime` တို့မှ event များကို ဖန်တီးပြီး status၊
error code၊ execution status နှင့် detection source တို့ကို event ၏ အသေးစိတ်တွင် ထည့်သွင်းသည်။ `TrackingStartTime`
ကို integer အဖြစ် သိမ်းဆည်းထားသောကြောင့် `--include-interpreted-times` ပါမှသာ event ဖြစ်လာသည်။ ဖိုင်၏ ကိုယ်ပိုင်
အချိန်များကို ဘယ်တော့မှ အသုံးမပြုပါ။ အသုံးပြုနိုင်သော အချိန်မရှိသည့် record သည် parse လုပ်ထားသော output ထဲတွင်
ကျန်ရှိပြီး အချိန်မပါသော record အဖြစ် ရေတွက်သည်။

## စုံစမ်းစစ်ဆေးမှု ဗျူဟာ { #investigation-strategy }

1. `ThreatName`၊ `Path`၊ `SHA256`၊ `InitialDetectionTime` နှင့် `ThreatStatus` တို့မှ စတင်ပါ။
2. ခြိမ်းခြောက်မှုကို ဖယ်ရှားခဲ့သည်ဟု ကောက်ချက်မချမီ `ThreatStatusErrorCode` နှင့် `Problems` ကို စစ်ဆေးပါ။
3. Quarantine လုပ်ထားသော byte များကို ပြန်လည်ရယူရန် `QuarantineID` ကို quarantine ၏ `EntryID` နှင့် join လုပ်ပါ။
4. Hash များနှင့် path များကို `$MFT`၊ USN Journal၊ Prefetch၊ Amcache၊ browser history နှင့် event
   log များ (Defender ၏ operational log- event ID 1116၊ 1117၊ 1118 နှင့် 1119) သို့ pivot လုပ်ပါ။
5. `webfile` resource များအတွက် ကနဦးဝင်ရောက်မှုကို စုံစမ်းရန် URL နှင့် process စတင်ချိန်ကို အသုံးပြုပြီး
   ၎င်းတို့ကို browser၊ proxy နှင့် DNS အထောက်အထားများဖြင့် အတည်ပြုပါ။

## စမ်းသပ်ထားသော ဗားရှင်းများ { #tested-versions }

စမ်းသပ်မှုများသည် အခြား tool များက သီးခြားလွတ်လပ်စွာ ဖတ်ရှုထားသော ရလဒ်များပါသည့် အများသုံး ဖိုင်များကို အသုံးပြုသည်-

- Orange Cyberdefense ၏ Windows 10 နမူနာ (Defender 4.18.24090.11)၊ WMI ဖော်ပြခဲ့သော value များနှင့် တိုက်စစ်ထားသည်။
- plaso ၏ Windows စမ်းသပ်ဖိုင် နှစ်ခု၊ plaso ၏ `windefender_history` parser နှင့် တိုက်စစ်ထားသည်။
- DFIRArtifactMuseum ၏ APT simulator VM မှ ဖိုင် 14 ခု။
- Windows Server 2016 မှ ဖိုင်အချို့ကဲ့သို့ header ၏ version မပါဘဲ ရေးထားသော မိတ္တူတစ်ခု။
- အဆုံးမတိုင်မီ ပြတ်တောက်နေသော၊ ဗလာဖြစ်နေသော သို့မဟုတ် အခြားအမျိုးအစား ဖိုင်များ။

## နောက်ထပ် အသေးစိတ်များ { #further-details }

- [ys-defender-history-parser](https://github.com/Yamato-Security/ys-defender-history-parser)
- [Sootmark defender](https://github.com/Sootmark/defender)
- [Orange Cyberdefense wddh-parser](https://github.com/cert-orangecyberdefense/wddh-parser)
- [plaso Defender history parser](https://github.com/log2timeline/plaso/blob/main/plaso/parsers/windefender_history.py)
- [Velociraptor Windows.Applications.DefenderHistory](https://github.com/Velocidex/velociraptor-docs/blob/master/content/exchange/artifacts/Windows.Applications.DefenderHistory.yaml)
- [Microsoft: MSFT_MpThreatDetection class](https://learn.microsoft.com/en-us/previous-versions/windows/desktop/defender/msft-mpthreatdetection)
- [Microsoft: Microsoft Defender Antivirus event IDs](https://learn.microsoft.com/en-us/defender-endpoint/troubleshoot-microsoft-defender-antivirus)
