# NTFS အပြောင်းအလဲမှတ်တမ်း (`$LogFile`) { #ntfs-transaction-log-logfile }

## ၎င်းဆိုသည်မှာ နှင့် ၎င်း၏ အရေးပါပုံ { #what-it-is-and-why-it-matters }

NTFS သည် Windows က ၎င်း၏ system volume အတွက် အများအားဖြင့် အသုံးပြုသော ဖိုင်စနစ် ဖြစ်သည်။ Crash မှ ပြန်လည်ကောင်းမွန်ရန်
၎င်းသည် ဖိုင်စနစ် metadata တွင် ပြုလုပ်ရန် ရည်ရွယ်ထားသော ပြောင်းလဲမှုများကို `$LogFile` ဟု အမည်ရသော ဝှက်ထားသည့်
volume ဖိုင်တွင် မှတ်တမ်းတင်သည်။ အပ်ဒိတ်တစ်ခုစီတွင် **redo** byte များ (အခြေအနေသစ်) နှင့် **undo** byte များ (ပြောင်းလဲမှုကို
ပြန်လှည့်ရန် လိုအပ်သော ယခင်အခြေအနေ) ပါဝင်နိုင်သည်။ ၎င်းသည် Windows Event Log များ၊ registry `.LOG1` ဖိုင်များနှင့်
[USN ဂျာနယ်](usn-journal.md) တို့နှင့် သီးခြားဖြစ်သည်။

စုံစမ်းစစ်ဆေးသူအတွက် ယခင်အခြေအနေသည် အထူးအသုံးဝင်သည်။ Undo record တစ်ခုသည် အမည်မပြောင်းမီ ဖိုင်အမည်၊ ဖျက်ထားသော
directory entry သို့မဟုတ် metadata အပ်ဒိတ်မတိုင်မီ timestamp များကို ထိန်းသိမ်းထားနိုင်သည်။ အချို့ record များတွင်
NTFS က USN Journal သို့ ရေးသားခဲ့သော byte များလည်း ပါဝင်ပြီး စုဆောင်းထားသော `$J` တွင် မရှိတော့သည့် record များလည်း
ပါဝင်သည်။ သို့သော် transaction အများအပြားသည် ပုံမှန် operating system လှုပ်ရှားမှုနှင့် သက်ဆိုင်ပြီး တိုက်ခိုက်မှုကို မဖော်ပြပါ။

**ဥပမာ-** ဖယ်ရှားခဲ့သော index entry တစ်ခုက `report.txt` ကို ဖော်ပြပြီး နောက်ပိုင်းတွင် ထည့်သွင်းခဲ့သော entry တစ်ခုက
`report.txt.encrypted` ကို ဖော်ပြသည်။ ဖိုင် reference အပြည့်အစုံ၊ parent၊ transaction ကွင်းဆက်နှင့် အခြား
အထောက်အထားများကို တိုက်ဆိုင်စစ်ဆေးခြင်းက အမည်ပြောင်းခြင်းကို ထောက်ခံနိုင်သည်။ ထိုအမည်နှစ်ခုတည်းဖြင့် ransomware၊
၎င်း၏ operator သို့မဟုတ် encrypt လုပ်ခဲ့သည့် အချိန်အတိအကျကို ခွဲခြားသတ်မှတ်၍ မရပါ။

## တည်နေရာနှင့် စုဆောင်းခြင်း { #location-and-collection }

```text
C:\$LogFile
D:\$LogFile
```

NTFS volume တိုင်းတွင် ၎င်း၏ ကိုယ်ပိုင် log ရှိသည်။ NTFS metadata ကို ဖတ်နိုင်သော သို့မဟုတ် မှုခင်းဆိုင်ရာ image သို့မဟုတ်
snapshot မှ export လုပ်နိုင်သော tool ဖြင့် ရယူပါ။ သာမန် Windows ဖိုင်ကူးယူမှုဖြင့် ၎င်းကို ဝင်ရောက်၍ မရနိုင်ပါ။ တူညီသော
volume ၏ `$MFT` နှင့် `$Extend\$UsnJrnl:$J` ကို အချိန်နီးကပ်နိုင်သမျှ နီးကပ်စွာ စုဆောင်းပါ။ မူရင်းဖိုင်များနှင့် hash များကို
ထိန်းသိမ်းပါ။ နောက်ပိုင်း ရယူမှုတွင် ကွဲပြားသော လှုပ်ရှားမှုကာလ ပါဝင်နိုင်သည်။

`$LogFile` သည် **စက်ဝိုင်းပုံစံ log** ဖြစ်သည်- record အသစ်များက အဟောင်းများကို overwrite လုပ်သည်။ ထိန်းသိမ်းထားမှုသည် log
အရွယ်အစားနှင့် workload ပေါ် မူတည်ပြီး မိနစ် သို့မဟုတ် နာရီအနည်းငယ်သာ ဖြစ်နိုင်သည်။ ဖိုင်အမည် သို့မဟုတ် လုပ်ဆောင်ချက်တစ်ခု
မရှိခြင်းသည် ၎င်း ဘယ်တော့မှ မဖြစ်ပွားခဲ့ကြောင်း အထောက်အထား မဟုတ်ပါ။

## Record တစ်ခုက သက်သေပြသည့်အရာနှင့် သတိပြုရန်များ { #what-a-record-proves-and-cautions }

- အပ်ဒိတ်တစ်ခုသည် NTFS က recovery အတွက် log လုပ်ခဲ့သော metadata ကို မှတ်တမ်းတင်သည်။ Redo entry တစ်ခုတည်းက
  အဆင့်မြင့် လုပ်ဆောင်ချက်တစ်ခုလုံး ပြီးဆုံးခဲ့ကြောင်း သို့မဟုတ် သီးခြားလူတစ်ဦးက စတင်ခဲ့ကြောင်း သက်သေမပြပါ။ ဆက်စပ်
  record များ၊ transaction ပြီးဆုံးမှု၊ ရလဒ်ဖြစ်သော ဖိုင်စနစ်နှင့် သီးခြားလွတ်လပ်သော အထောက်အထားများကို ချိတ်ဆက်စစ်ဆေးပါ။
- **ယေဘုယျ transaction timestamp မရှိပါ။** ထည့်သွင်းထားသော timestamp များသည် ဖိုင် metadata ကို ဖော်ပြပြီး တစ်ခါတစ်ရံ
  ခေတ်မမီတော့ခြင်း၊ ကူးယူထားခြင်း သို့မဟုတ် တိုက်ခိုက်သူက ပြင်ဆင်ထားခြင်း ဖြစ်နိုင်သည်။ `CreatedOn` ကို incident timeline
  တွင် အမည်ပြောင်းချိန် သို့မဟုတ် timestamp ပြောင်းလဲချိန်အဖြစ် မထည့်ပါနှင့်။ USN record များက သီးခြား timestamp ပေးနိုင်သည်။
- LSN (log sequence number) များသည် record များကို **volume တစ်ခု၏ log အတွင်း** အစီအစဉ်ချသည်။ ၎င်းတို့သည် နာရီအချိန်များ
  မဟုတ်ဘဲ host များ သို့မဟုတ် volume များကြား နှိုင်းယှဉ်ခြင်း မပြုရပါ။ ယခင် circular-buffer အပတ်များမှ record များသည်
  လှုပ်ရှားမှုအသစ်များနှင့်အတူ ကျန်ရှိနေနိုင်သည်။
- Transaction-table slot ကို ပြန်လည်အသုံးပြုသည်။ `TransactionId` တစ်ခုတည်းဖြင့်မဟုတ်ဘဲ `TransactionFirstLsn`၊ `TransactionId`၊
  client identity နှင့် source ဖြင့် အုပ်စုဖွဲ့ပါ။ ကွင်းဆက်များသည် ထိန်းသိမ်းထားသော ဒေတာ၏ ပြင်ပသို့ ညွှန်နိုင်သည်။
- ဖိုင် reference များတွင် entry number **နှင့် sequence number** ပါဝင်သည်။ မဟုတ်ပါက MFT entry ပြန်လည်အသုံးပြုခြင်းက
  မသက်ဆိုင်သော ဖိုင်များကို တူညီသော ဖိုင်ဖြစ်သည်ဟု ထင်ရစေနိုင်သည်။
- `$MFT` က ပေးသော path များသည် **စုဆောင်းထားသော MFT အခြေအနေ** ကို ဖော်ပြပြီး ပြန်လည်တည်ဆောက်ထားသော သမိုင်းဆိုင်ရာ
  path မဟုတ်ပါ။ Juntei သည် sequence number များကို တိုက်ဆိုင်စစ်ဆေးပြီး မသိသော အစိတ်အပိုင်းများကို ရှင်းလင်းစွာ ထားရှိသည်။
  Transaction နောက်ပိုင်းတွင် parent ကို အမည်ပြောင်းခဲ့နိုင်ပြီး hard link များက လက်ရှိအမည် တစ်ခုထက်ပို၍ ပေးနိုင်သည်။
- `TargetInferred` ဆိုသည်မှာ open-attribute mapping သည် log ၏ အခြားကာလတစ်ခုမှ လာကြောင်း ဆိုလိုသည်။ ထို mapping
  ကို ထပ်ဆောင်း သတိထား၍ ဆက်ဆံပါ။ မဖြေရှင်းရသေးသော index entry များကို ၎င်းတို့၏ layout တစ်ခုတည်းဖြင့် decode
  လုပ်နိုင်သည်။ ၎င်းတို့၏ အမည်များကို raw byte များနှင့် အခြား source များဖြင့် အတည်ပြုပါ။
- Torn page ကို ကျော်သွားသည်။ ဖတ်ရှုနိုင်သော ပတ်ဝန်းကျင် record များကို ဆက်လက် ရရှိနိုင်သည်။ `Truncated` သည် ၎င်း၏
  ဆက်လက်အပိုင်း ပျောက်နေသော record ကို အမှတ်အသားပြုသည်။ ထုတ်ယူမှုကို ပြည့်စုံသည်ဟု မဆက်ဆံမီ parser diagnostic များကို
  ပြန်လည်သုံးသပ်ပါ။ အရှည် သုည input သည် error ဖြစ်သည်။ မရေးသားရသေးသော all-`FF` log တွင် row မရှိခြင်းသည် တရားဝင်ပါသည်။
- Parser သည် သမိုင်းဆိုင်ရာ MFT ကို ပြန်လည်တည်ဆောက်ရန် transaction များကို **replay မလုပ်ပါ**။ ၎င်းသည် non-resident
  ဖိုင်အကြောင်းအရာ အမျိုးမျိုးကို ပြန်လည်မရယူပါ၊ စုဆောင်းထားသော ဒေတာ မည်သည်ကိုမျှ run မလုပ်ပါ။ DAX CRC-protected
  page များကို ပံ့ပိုးမထားပါ။ အမှန်တကယ် အထောက်အထားဖြင့် မှန်ကန်မှု စစ်ဆေးခြင်းသည် အဓိကအားဖြင့် 4 KiB log page များကို ဖုံးလွှမ်းသည်။

## Juntei ဖြင့် ခွဲခြမ်းစိတ်ဖြာခြင်း { #parse-with-juntei }

```bash
# Single log, current paths from an adjacent $MFT when present
juntei parse-ntfs logfile -f '$LogFile' -o logfile.csv

# Explicit MFT from the SAME volume
juntei parse-ntfs logfile -f '$LogFile' -m '$MFT' -t jsonl -o logfile.jsonl

# Multiple collected logs; SourceFile distinguishes every input
juntei parse-ntfs logfile -d evidence -o collected-logs.csv

# Preserve the underlying payloads for low-level validation
juntei parse-ntfs logfile -f '$LogFile' --raw-data -t jsonl -o logfile-raw.jsonl

# Automatic per-computer output
juntei auto-parse -d evidence -o parsed
```

Option အားလုံးအတွက် [ညွှန်ကြားချက် ကိုးကားချက်](../commands/filesystem.md#parse-ntfs-logfile) ကို ကြည့်ပါ။ Directory
ရှာဖွေမှုသည် `$LogFile`၊ `LogFile`၊ ၎င်းတို့၏ `.bin` ပုံစံများနှင့် `C_$LogFile` သို့မဟုတ်
`C_$LogFile.bin` ကဲ့သို့သော အမည်များကို စာလုံးအကြီးအသေး မခွဲခြားဘဲ လက်ခံသည်။ `-f` ဖြင့် ပေးသော ဖိုင်တစ်ခုတည်းသည် မည်သည့်အမည်မဆို ဖြစ်နိုင်သည်။

အလိုအလျောက် MFT တွဲဖက်ခြင်းသည် export ၏ volume prefix ကို လေးစားလိုက်နာသည်- `C_$LogFile` သည် `C_$MFT` ကို အသုံးပြုပြီး
`D_$LogFile.bin` သည် ပထမဦးစွာ `D_$MFT.bin` ကို စမ်းကြည့်ပြီးနောက် `D_$MFT` ကို စမ်းကြည့်သည်။ Prefix ပါသော log များသည် ရိုးရိုး `$MFT`
ကို fallback အဖြစ် ဘယ်တော့မှ မသုံးပါ။ Prefix မပါသော `LogFile` သည် `MFT` သို့မဟုတ် `$MFT` ကိုလည်း လက်ခံသည်။ သင့် export တွင်
အခြား အမည်ပေးပုံ ရှိပါက ကိုက်ညီသော MFT ကို ရှင်းလင်းစွာ ရွေးချယ်ရန် `-f` နှင့်အတူ `-m` ကို အသုံးပြုပါ။

ထုတ်ယူမှုသည် update၊ checkpoint နှင့် မသိသော client record များ အပါအဝင် LSN တစ်ခုလျှင် record တစ်ခု ဖြစ်သည်။ JSON သည်
pretty-print လုပ်ထားသော object များ၏ stream ဖြစ်ပြီး JSONL တွင် line တစ်ကြောင်းလျှင် object တစ်ခု ရှိသည်။ CSV တွင် တူညီသော field
အမည်များ ရှိပြီး array များနှင့် ဖွဲ့စည်းထားသော field များကို CSV cell အတွင်း compact JSON အဖြစ် ထည့်ထားသည်။ မသိသော scalar
များသည် CSV တွင် ဗလာဖြစ်ပြီး JSON တွင် `null` ဖြစ်သည်။ ဗလာ array များသည် item များကို decode မလုပ်ခဲ့ကြောင်း ဆိုလိုပြီး
မရှိခဲ့ကြောင်း သက်သေ မဟုတ်ပါ။

Log တစ်ခုလုံးနှင့် ၎င်း၏ ခွဲခြမ်းစိတ်ဖြာထားသော record များကို တစ်ကြိမ်လျှင် တစ်ခုစီ memory တွင် ထိန်းထားသည်။ MFT path များကို
ဖြေရှင်းခြင်းက memory နှင့် အလုပ်ကို တိုးစေသည်။ `--raw-data` သည် ထုတ်ယူမှု အရွယ်အစားကို အလွန် တိုးစေနိုင်သည်။ `auto-parse`
တွင် memory ကန့်သတ်ထားသော စက်ပေါ်၌ တစ်ပြိုင်နက် run သော parser process များကို ကန့်သတ်ရန် `-T` ကို အသုံးပြုပါ။

ဘေးချင်းကပ် MFT ကို ဖတ်၍ မရပါက Juntei သည် သတိပေးပြီး log ကို path များမပါဘဲ export လုပ်သည်။ ထိုအခါ `MftSource`
သည် null အဖြစ် ကျန်ရှိသည်။ ဤ enrichment မအောင်မြင်မှုသည် ဖတ်ရှုနိုင်သော အဓိက အထောက်အထားကို စွန့်ပစ်ခြင်း မပြုပါ။
ပျောက်နေသော သို့မဟုတ် မရှင်းလင်းသော MFT တွဲဖက်မှုများသည်လည်း သတိပေးချက်နှင့် အနှစ်ချုပ်တွင် `Logs without a matching MFT`
အရေအတွက်ကို ထုတ်ပေးသည်။ ၎င်းတို့ကို parse မအောင်မြင်မှုများအဖြစ် မရေတွက်ပါ။

## ထုတ်ယူမှု field များ { #output-fields }

### Record ကိုယ်ပိုင်အမှတ်အသား၊ integrity နှင့် မူရင်းအရင်းအမြစ် { #record-identity-integrity-and-provenance }

| Field | အဓိပ္ပာယ်နှင့် အဓိပ္ပာယ်ဖော်ပုံ |
|---|---|
| `SourceFile`, `FileOffset` | Input path နှင့် ဤ record ၏ header ကို ဖတ်ခဲ့သော byte offset။ Byte များကို အတည်ပြုရန် ၎င်းတို့ကို အသုံးပြုပါ။ |
| `LSN` | Record ၏ log sequence number။ တန်ဖိုးကြီးများသည် integer များဖြစ်ပြီး timestamp များ မဟုတ်ပါ။ |
| `PreviousLSN`, `UndoNextLSN` | ယခင် record နှင့် rollback record များသို့ link များ။ သုညသည် link မရှိကြောင်း ဆိုလိုသည်။ သုညမဟုတ်သော link များသည် ထိန်းသိမ်းထားသော ဒေတာ၏ ပြင်ပသို့ ညွှန်နိုင်သည်။ |
| `TransactionID`, `TransactionFirstLSN` | Transaction-table slot နှင့် ဤ transaction နှင့် ချိတ်ဆက်ထားသော အစောဆုံး LSN။ Slot တစ်ခုတည်းသည် တစ်မူထူးခြားသည် မဟုတ်ပါ။ |
| `RecordType` | `ClientRecord` (update)၊ `ClientRestart` (checkpoint) သို့မဟုတ် `Other`။ |
| `ClientIndex`, `ClientSequence` | Log client ၏ ကိုယ်ပိုင်အမှတ်အသား၊ ပုံမှန်အားဖြင့် NTFS။ ၎င်းတို့သည် အသုံးပြုသူအကောင့် သို့မဟုတ် process ID မဟုတ်ပါ။ |
| `Flags`, `SpansPages` | Raw record flag များနှင့် client ဒေတာသည် အခြား page သို့ ဆက်သွားခြင်း ရှိ/မရှိ။ Page များကို ဖြတ်ကျော်ခြင်းသည် ပုံမှန်ဖြစ်သည်။ |
| `Truncated` | ဤ LSN အတွက် ပျောက်နေသော ဆက်လက်အပိုင်းကို parser က တွေ့ရှိပါက True။ ၎င်းသည် ဖိုင်တစ်ခုလုံးရှိ ပျက်စီးမှုအားလုံးကို အနှစ်ချုပ်ခြင်း မဟုတ်ပါ။ |
| `ClusterSize`, `ClusterSizeSource` | Target offset များအတွက် အသုံးပြုသော cluster တစ်ခုလျှင် byte အရေအတွက်။ Source သည် `Caller`၊ `Checkpoint` သို့မဟုတ် `Inferred` ဖြစ်သည်။ သိရှိပါက အမှန်တကယ် value ကို `--cluster-size` ဖြင့် ပေးပါ။ |
| `LogVersion` | ဖတ်ရှုနိုင်သော နောက်ဆုံး restart page မှ ဗားရှင်း၊ ပုံမှန်အားဖြင့် `1.1` သို့မဟုတ် `2.0`။ ဖတ်ရှုနိုင်သော restart page မကျန်ပါက null။ |
| `MFT-Source` | ရှိပါက လက်ရှိ path enrichment အတွက် အသုံးပြုခဲ့သော MFT။ ၎င်းသည် တူညီသော volume နှင့် သက်ဆိုင်ရမည်။ |

`auto-parse` JSON/JSONL တွင် အခြား parser များကဲ့သို့ `Comp` နှင့် `Artifact` ကို ထည့်သွင်းသည်။ Task ၏
`.txt` log တွင် diagnostic များ ပါဝင်ပြီး manifest များက task အောင်မြင်ခဲ့ခြင်း၊ မအောင်မြင်ခဲ့ခြင်း သို့မဟုတ် ကျော်သွားခဲ့ခြင်းကို ဖော်ပြသည်။
ကယ်တင်နိုင်သော ပျက်စီးမှုသည် ၎င်း၏ log တွင် သုညမဟုတ်သော `Parse problems` အရေအတွက်ပါသည့် အောင်မြင်သော task ကို ထုတ်ပေးနိုင်သည်။

### လုပ်ဆောင်ချက်များနှင့် ပစ်မှတ် { #operations-and-target }

| Field | အဓိပ္ပာယ်နှင့် အဓိပ္ပာယ်ဖော်ပုံ |
|---|---|
| `RedoOperation`, `UndoOperation` | NTFS လုပ်ဆောင်ချက် အမည်များ၊ သို့မဟုတ် မသိသော code အတွက် hexadecimal။ Redo သည် ရည်ရွယ်ထားသော အပ်ဒိတ်ကို ဖော်ပြပြီး undo သည် ၎င်း၏ ပြောင်းပြန်ကို ဖော်ပြသည်။ |
| `TargetEntryNumber`, `TargetSequenceNumber` | ဖိုင် record ပေါ်ရှိ လုပ်ဆောင်ချက်အတွက်- သက်ရောက်သော MFT record နံပါတ်၊ sequence number ကို မခန့်မှန်းပါ။ အခြားအခြေအနေတွင်- ဖြေရှင်းထားသော open attribute ကို ပိုင်ဆိုင်သော ဖိုင် reference။ Inline directory-index အပ်ဒိတ်သည် directory ကို ပစ်မှတ်ထားပြီး ၎င်း၏ entry က ဖော်ပြသော child ကို မဟုတ်ပါ။ |
| `TargetAttributeType`, `TargetAttributeName` | လုပ်ဆောင်ချက်က အသုံးပြုသော open attribute ၏ နံပါတ်အမျိုးအစားနှင့် အမည်။ ဖိုင်-record လုပ်ဆောင်ချက်များအတွက် ၎င်းသည် ပုံမှန်အားဖြင့် `$MFT` ၏ `$DATA` ဖြစ်သည်။ အသုံးများသော အမျိုးအစားများ- 16 (`$STANDARD_INFORMATION`)၊ 48 (`$FILE_NAME`)၊ 128 (`$DATA`)၊ 144 (`$INDEX_ROOT`)၊ 160 (`$INDEX_ALLOCATION`)၊ 176 (`$BITMAP`)။ |
| `TargetPath` | ပစ်မှတ်၏ reference အပြည့်အစုံကို သိရှိပါက လက်ရှိ MFT path၊ သို့မဟုတ် မသိသော အစိတ်အပိုင်းများ။ သမိုင်းဆိုင်ရာ generation မသိသော ဖိုင်-record လုပ်ဆောင်ချက်များအတွက် ဗလာ။ Record နံပါတ်တစ်ခုတည်းကို ၎င်း၏ လက်ရှိ နေထိုင်သူသို့ မဖြေရှင်းရပါ။ |
| `OpenAttributeEntryNumber`, `OpenAttributeSequenceNumber`, `OpenAttributePath` | လုပ်ဆောင်ချက်က ရေးသားရာ open attribute ကို ပိုင်ဆိုင်သော ဖိုင်နှင့် ၎င်း၏ လက်ရှိ MFT path။ ဖိုင်-record လုပ်ဆောင်ချက်များအတွက် ၎င်းတို့သည် `$MFT` အခြေအနေကို သက်ရောက်သော record မှ သီးခြား ထိန်းသိမ်းထားသည်။ |
| `TargetAttribute` | Log ၏ open-attribute table အတွင်းရှိ offset။ ၎င်းသည် attribute ၏ အမျိုးအစား သို့မဟုတ် MFT identifier နှင့် ကွဲပြားသည်။ |
| `TargetInferred` | Target mapping သည် log ၏ အခြားကာလတစ်ခုမှ လာပါက True။ ဤ flag သည် false ဖြစ်နေသော်လည်း null target သည် မဖြေရှင်းရသေးဘဲ ကျန်ရှိသည်။ |
| `TargetVCN`, `Lcns` | Target အတွင်းရှိ virtual cluster နှင့် physical cluster နံပါတ်များ။ အချိန်သတ်မှတ်ရန်ထက် low-level မှန်ကန်မှု စစ်ဆေးရန် အသုံးဝင်သည်။ |
| `RecordOffset`, `AttributeOffset` | Attribute/entry ၏ offset နှင့် ၎င်းအတွင်းရှိ အပ်ဒိတ်၏ offset။ |
| `ClusterBlockOffset`, `TargetBlockSize` | 512 byte ယူနစ်ဖြင့် offset နှင့် target အရွယ်အစား။ |
| `OffsetInTarget` | သက်ရောက်သော ဖိုင် record သို့မဟုတ် non-resident stream အတွင်း တွက်ချက်ထားသော byte offset။ Cluster နှင့် MFT record အရွယ်အစားများပေါ် မူတည်သည်။ |
| `RedoLength`, `UndoLength` | ကြေညာထားသော ဒေတာအရှည်များ။ Zeroing လုပ်ဆောင်ချက်သည် payload byte အရေအတွက် ထိုမျှ မပါဘဲ အရှည်ကို ကြေညာနိုင်သည်။ |
| `RedoBytesAvailable`, `UndoBytesAvailable` | Decode လုပ်ရန် အမှန်တကယ် ရရှိနိုင်သော payload byte များ။ ကြေညာထားသော အရှည်များနှင့် လုပ်ဆောင်ချက် အဓိပ္ပာယ်တို့နှင့် နှိုင်းယှဉ်ပါ။ |

### အဆင်ပြေသော ဖိုင် metadata column များ { #convenient-file-metadata-columns }

ဤ column များသည် redo ဘက်မှ ဦးစားပေး DOS မဟုတ်သော ဖိုင်အမည်တစ်ခုကို ပြသပြီး မရှိပါက undo ဘက်သို့ ပြန်လှည့်သည်။
ဘက်နှစ်ဘက်စလုံးတွင် ဖိုင်အမည် မပါပါက timestamp-update field များသည် redo timestamp များကို ဦးစားပေးပြီးနောက် undo ကို အသုံးပြုသည်။
Directory-entry အပ်ဒိတ်များသည်လည်း ၎င်းတို့၏ အချိန်များ၊ အရွယ်အစားနှင့် flag များကို undo ထက် redo ကို ဦးစားပေး၍ တိုက်ရိုက် ဖော်ပြသည်။
Decode လုပ်ထားသော `$DATA` attribute များသည် ဖိုင်အမည် မပါသည့်တိုင် ၎င်းတို့၏ အရွယ်အစားကို ဖော်ပြသည်။ Decode လုပ်ထားသော
ဖိုင် record ၏ အမည်မဲ့ `$DATA` အရွယ်အစားသည် ခေတ်မမီနိုင်သော ဖိုင်အမည် အရွယ်အစားထက် ဦးစားပေးခံရသည်။
၎င်းတို့သည် ကြည့်ရှုရာတွင် အထောက်အကူပြုရန်သာ ဖြစ်သည်။ ပြည့်စုံသော မတိုင်မီ/ပြီးနောက် နှိုင်းယှဉ်မှုအတွက် သီးခြား ဘက် field များကို အသုံးပြုပါ။

| Field | အဓိပ္ပာယ်နှင့် အဓိပ္ပာယ်ဖော်ပုံ |
|---|---|
| `MetadataSide` | `Redo` သို့မဟုတ် `Undo`၊ အဆင်ပြေသော ဖိုင်အမည်/timestamp column များကို ပေးသော ဘက်ကို ခွဲခြားသတ်မှတ်သည်။ |
| `EntryNumber`, `SequenceNumber` | Decode လုပ်ထားသော ဖိုင်အမည်/ဖိုင်-record/attribute metadata သို့မဟုတ် တစ်စိတ်တစ်ပိုင်း `$STANDARD_INFORMATION` timestamp အပ်ဒိတ်က ဖော်ပြသော ဖိုင်၊ သိရှိပါက sequence နှင့်အတူ။ Child ကို ခွဲခြားမဖော်ပြသော directory-entry အပ်ဒိတ်နှင့် raw size/run-list အပ်ဒိတ်များအတွက် ဗလာ။ သက်ရောက်သော record အတွက် `TargetEntryNumber` ကို အသုံးပြုပါ။ |
| `Filename` | ဦးစားပေး decode လုပ်ထားသော အမည်။ DOS alias များနှင့် hard link များ အပါအဝင် အမည်အားလုံးသည် ဘက် array များတွင် ကျန်ရှိသည်။ |
| `ParentEntryNumber`, `ParentSequenceNumber`, `ParentPath` | Parent reference နှင့် လက်ရှိ MFT path။ သမိုင်းဆိုင်ရာ path ပြန်လည်တည်ဆောက်မှု မဟုတ်ပါ။ |
| `CreatedOn`, `ContentModifiedOn`, `RecordModifiedOn`, `LastAccessedOn` | 100 ns အထိ တိကျသော UTC metadata အချိန်များ။ ၎င်းတို့သည် transaction timestamp များ မဟုတ်ပါ။ ပျောက်နေသော/မမှန်ကန်သော အချိန်များသည် ဗလာ/null အဖြစ် ကျန်ရှိသည်။ |
| `FileSize`, `FileAttributes` | ရွေးချယ်ထားသော metadata မှ အရွယ်အစားနှင့် နံပါတ် Windows flag များ။ Directory-entry value များသည် ခေတ်မမီနိုင်ပါ။ `$DATA` attribute များနှင့် decode လုပ်ထားသော ဖိုင် record များက ပိုမိုတိုက်ရိုက်သော အရွယ်အစားကို ပေးနိုင်သည်။ |
| `AttrType` | Decode လုပ်ထားသော attribute record ၏ နံပါတ်အမျိုးအစား။ Decode လုပ်ထားသော directory-entry metadata အပ်ဒိတ်အတွက် 48။ လုပ်ဆောင်ချက်က ပစ်မှတ်ထားသော open attribute နှင့် သီးခြားဖြစ်သည်။ |

### ပြည့်စုံသော redo/undo metadata နှင့် payload များ { #complete-redoundo-metadata-and-payloads }

| Field | အဓိပ္ပာယ်နှင့် အဓိပ္ပာယ်ဖော်ပုံ |
|---|---|
| `RedoFileNames`, `UndoFileNames` | ထိုဘက်ရှိ decode လုပ်ထားသော `$FILE_NAME` အားလုံး၏ array များ။ ထည့်သွင်းထားသော/အသစ် နှင့် ဖယ်ရှားထားသော/အဟောင်း အမည်များကို ထိန်းသိမ်းထားသည်။ |
| `RedoTimes`, `UndoTimes` | ဘက်တစ်ခုစီက ရေးသားခဲ့သော `$STANDARD_INFORMATION` timestamp များ။ တစ်စိတ်တစ်ပိုင်း အပ်ဒိတ်များသည် မပြောင်းလဲသော timestamp များကို ခန့်မှန်းမည့်အစား မပါဘဲ ထားခဲ့သည်။ |
| `RedoDecoded`, `UndoDecoded` | `Kind` ဖြင့် ခွဲခြားသတ်မှတ်ထားသော ဖွဲ့စည်းထားသည့် ဖိုင် record၊ attribute၊ index entry သို့မဟုတ် directory-entry အချက်အလက်။ Null သည် ပံ့ပိုးမထားသော လုပ်ဆောင်ချက် သို့မဟုတ် byte မလုံလောက်ခြင်းကို ဆိုလိုနိုင်သည်။ |
| `JournalWriteOffset`, `JournalWriteLength` | `$DATA:$J` သို့ ရေးသားသော redo ဒေတာ၏ offset နှင့် byte အရေအတွက်။ ၎င်းသည် မဟုတ်ပါက ပျောက်ဆုံးသွားမည့် USN record များ၏ ဖြစ်နိုင်သော source ကို ခွဲခြားသတ်မှတ်သည်။ ပြန်လည်တည်ဆောက်ထားသော journal ကို export မလုပ်ပါ။ |
| `Checkpoint` | Checkpoint ဗားရှင်း၊ start LSN၊ cluster အရွယ်အစားနှင့် open-attribute၊ attribute-name၊ dirty-page နှင့် transaction-table dump များ၏ LSN/အရှည်။ |
| `OtherRecordType` | အသိအမှတ်မပြုနိုင်သော သို့မဟုတ် decode မလုပ်နိုင်သော body အတွက် raw client record အမျိုးအစား။ |
| `RedoData`, `UndoData`, `OtherData` | `--raw-data` ပါမှသာ hex payload များ။ မဟုတ်ပါက null/ဗလာ။ ပံ့ပိုးမထားသော သို့မဟုတ် ပျက်စီးနေသော ဖွဲ့စည်းပုံများကို မှန်ကန်မှု စစ်ဆေးရန် ၎င်းတို့ကို ထိန်းသိမ်းပါ။ |

ဖိုင်အမည် object တစ်ခုစီတွင် ဖိုင်နှင့် parent reference များ (`Raw`, `EntryNumber`,
`SequenceNumber`)၊ `Name`၊ `NameEscaped`၊ `Namespace`၊ လက်ရှိ `ParentPath`၊ timestamp လေးခု၊
`Size`၊ `AllocatedSize`၊ `Attr` နှင့် `ReparseValue` ပါဝင်သည်။ `NameEscaped` သည် escape များဖြင့် ဖော်ပြထားသော
မမှန်ကန်သည့် UTF-16 ကို ခွဲခြားသတ်မှတ်သည်။ Reparse value သည် reparse tag သို့မဟုတ် extended-attribute အရွယ်အစား ဖြစ်နိုင်သည်။

Timestamp object များသည် `Utc` နှင့် integer `Filetime` နှစ်ခုစလုံးကို ထိန်းသိမ်းထားသည်။ Sentinel သို့မဟုတ် မမှန်ကန်သော value
တစ်ခုတွင် null UTC ဖော်ပြချက် ရှိနိုင်ပြီး ၎င်း၏ မူလ integer ကိုမူ ထိန်းသိမ်းထားသည်။ `RedoTimes` နှင့် `UndoTimes`
တွင် `Created`၊ `Modified`၊ `Changed` နှင့် `Accessed` ပါဝင်သည်။ ပျောက်နေသော member များ၏ value များသည် null ဖြစ်သည်။

Decode လုပ်ထားသော `FileRecord` object များတွင် နံပါတ်၊ sequence၊ hard-link အရေအတွက်၊ flag များ၊ base reference၊
header သာ ကျန်ရှိခြင်း ရှိ/မရှိနှင့် ပါဝင်သော attribute များ ပါဝင်သည်။ Decode လုပ်ထားသော attribute တစ်ခုစီသည်
အမျိုးအစား၊ အမည်၊ resident အခြေအနေ၊ flag များ၊ ID၊ အရွယ်အစားနှင့် allocated အရွယ်အစားကို ထိန်းသိမ်းထားသည်။ `$STANDARD_INFORMATION` က
အချိန်လေးခု၊ attribute များ၊ owner/security ID များ၊ quota နှင့် USN ကို ထပ်ထည့်သည်။ ဤ ID များကို သီးခြား ဖြေရှင်းရန် လိုအပ်ပြီး
ဖိုင်ကို ပြောင်းလဲခဲ့သော အသုံးပြုသူ၏ သက်သေ မဟုတ်ပါ။ Index entry များတွင် flag များ၊ key အရှည်နှင့် ရွေးချယ်နိုင်သော
ဖိုင်အမည် ပါဝင်သည်။ Directory-entry အပ်ဒိတ်များသည် အမည်ကိုယ်တိုင်ကို log မလုပ်ခဲ့သည့်တိုင် အချိန်လေးခု၊ အရွယ်အစားများ၊
attribute များနှင့် reparse value ကို ထိန်းသိမ်းထားသည်။

## စုံစမ်းစစ်ဆေးမှု လုပ်ငန်းစဉ် { #investigation-workflow }

1. ပျောက်နေသော restart page များ၊ torn page များနှင့် ဖြတ်တောက်ခံရသော record များအတွက် diagnostic များကို ပြန်လည်သုံးသပ်ပါ။
2. သံသယဖြစ်ဖွယ် ဖိုင်အမည် သို့မဟုတ် extension အတွက် `RedoFileNames` နှင့် `UndoFileNames` နှစ်ခုစလုံးကို ရှာဖွေပါ။
3. Entry **နှင့် sequence** နံပါတ်များကို တိုက်ဆိုင်စစ်ဆေးပြီးနောက် transaction ၏ LSN ကွင်းဆက်နှင့် လုပ်ဆောင်ချက်များကို စစ်ဆေးပါ။
4. ပြောင်းလဲမှုများအတွက် redo နှင့် undo timestamp များကို နှိုင်းယှဉ်ပါ။ ပုံမှန် ကူးယူခြင်း၊ ထုတ်ယူခြင်းနှင့် ပြန်လည်ထားရှိခြင်းတို့ကလည်း
   timestamp များကို ပြောင်းလဲစေနိုင်သည်။ ကွာခြားချက်သည် သဲလွန်စတစ်ခုသာဖြစ်ပြီး အလိုအလျောက် timestomping စီရင်ချက် မဟုတ်ပါ။
5. [USN ဂျာနယ်](usn-journal.md)၊ [$MFT](mft.md) နှင့် [$I30](i30.md) တို့နှင့် ချိတ်ဆက်စစ်ဆေးပါ။ လုပ်ဆောင်ခဲ့ခြင်း သို့မဟုတ်
   တာဝန်ရှိသူကို သတ်မှတ်ရန် event log များ၊ [Prefetch](prefetch.md) နှင့် အသုံးပြုသူ လှုပ်ရှားမှု အထောက်အထားများကို အသုံးပြုပါ။

ဥပမာ ဘက်တစ်ဘက်ဘက်တွင် encrypt လုပ်ထားသော ဖိုင်အမည် ပါဝင်သည့် record အားလုံးကို ရှာပါ-

```bash
jq 'select(any(.RedoFileNames[]?, .UndoFileNames[]?;
  .Name | endswith(".encrypted"))) |
  {Lsn, TransactionId, TransactionFirstLsn, RedoOperation, UndoOperation,
   RedoFileNames, UndoFileNames}' logfile.jsonl
```

၎င်းသည် ဖိုင်အမည် ရှာဖွေမှုသာဖြစ်ပြီး ပြည့်စုံသော ransomware detector မဟုတ်ပါ။ Journal payload များကို စစ်ဆေးရန်
`--raw-data` ဖြင့် export လုပ်ပြီး null မဟုတ်သော `JournalWriteOffset` ပါသည့် record များကို ရွေးချယ်ပါ။ `RedoData`
hex သည် ထို offset တွင် ရေးသားခဲ့သော byte များကို ကိုယ်စားပြုသည်။ ထိုထည့်သွင်းထားသော USN record များကို ခွဲခြမ်းစိတ်ဖြာခြင်းနှင့်
dedupe လုပ်ခြင်းသည် သီးခြားအဆင့် ဖြစ်သည်။ Juntei သည် ၎င်းတို့ကို `parse-ntfs usn-journal` ထုတ်ယူမှုနှင့် မပေါင်းစပ်ရသေးပါ။

## နောက်ထပ် အသေးစိတ်များ { #further-details }

- [ys-ntfs-parser ၏ logfile crate](https://github.com/Yamato-Security/ys-ntfs-parser/tree/main/logfile)- ပံ့ပိုးထားသော ဖွဲ့စည်းပုံများ၊ မှန်ကန်မှု စစ်ဆေးရေး corpus နှင့် သိရှိထားသော ကန့်သတ်ချက်များ။
- [Sootmark ntfslog](https://github.com/Sootmark/ntfslog)- upstream Rust implementation။
- [libfsntfs NTFS ပုံစံ စာရွက်စာတမ်း](https://github.com/libyal/libfsntfs/tree/main/documentation)- log page များ၊ fixup များနှင့် metadata ဖွဲ့စည်းပုံများ။
- [dfir_ntfs](https://github.com/msuhanov/dfir_ntfs)- သီးခြားလွတ်လပ်သော NTFS ခွဲခြမ်းစိတ်ဖြာမှု သုတေသနနှင့် implementation။
- [Joakim Schicht ၏ LogFileParser](https://github.com/jschicht/LogFileParser)- အသေးစိတ် transaction-log ခွဲခြမ်းစိတ်ဖြာမှု။
- [Stroz Friedberg ၏ ntfs-linker](https://github.com/strozfriedberg/ntfs-linker)- NTFS source များကို ဖိုင်သမိုင်းကြောင်းအဖြစ် ချိတ်ဆက်ခြင်း။
- [DFIR Artifact Museum](https://github.com/AndrewRathbun/DFIRArtifactMuseum)- အမှန်တကယ် Windows NTFS နမူနာများ။
- [Velocidex go-ntfs](https://github.com/Velocidex/go-ntfs)- NTFS implementation နှင့် အများသုံး စမ်းသပ် volume။
