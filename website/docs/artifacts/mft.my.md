# ပင်မဖိုင်ဇယား (`$MFT`) { #master-file-table-mft }

## MFT ဆိုသည်မှာ { #what-the-mft-is }

NTFS သည် ဖိုင်နှင့် directory တိုင်းကို Master File Table ရှိ record တစ်ခုဖြင့် ကိုယ်စားပြုသည်။ Record များတွင်
အမည်များ၊ parent reference များ၊ timestamp များ၊ အရွယ်အစားများ၊ flag များ၊ security identifier များ၊ reparse
အချက်အလက်နှင့် data-stream metadata ကဲ့သို့သော attribute များ ပါဝင်သည်။ ဖိုင်အကြောင်းအရာ အသေးများနှင့် alternate data
stream များသည် **resident** ဖြစ်နိုင်သည်၊ ဆိုလိုသည်မှာ ၎င်းတို့၏ byte များကို MFT record ကိုယ်တိုင်အတွင်း
သိမ်းဆည်းထားသည်။

MFT သည် volume တစ်ခုလုံးရှိ allocate လုပ်ထားသော record များနှင့် ဖျက်ပစ်ထားသော record များကို လွှမ်းခြုံသောကြောင့်
အခြေခံကျသည်။ ၎င်းသည် ဖိုင်တစ်ခု တည်ရှိခဲ့ကြောင်း၊ မည်သည့်နေရာတွင် link လုပ်ထားခဲ့ကြောင်း၊ ဖျက်ပစ်ခံရခြင်း ရှိ/မရှိနှင့်
၎င်း၏ metadata မည်သို့ ပြောင်းလဲခဲ့သည်ကို—ပုံမှန် directory ကြည့်ရှုခြင်းဖြင့် မမြင်ရတော့သည့်တိုင်—ပြသနိုင်သည်။

<span class="evidence-strength">အကောင်းဆုံး ကောက်ချက်:</span> ပြသထားသော metadata ပါဝင်သည့် NTFS ဖိုင် record
တစ်ခုသည် စုဆောင်းချိန်တွင် တည်ရှိခဲ့သည် သို့မဟုတ် unallocated record တစ်ခုတွင် ကျန်ရှိခဲ့သည်။ MFT တစ်ခုတည်းဖြင့်
လုပ်ဆောင်ခဲ့ခြင်း (execution) သို့မဟုတ် အသုံးပြုသူ၏ ရည်ရွယ်ချက်ကို သက်သေမပြပါ။

## တည်နေရာနှင့် စုဆောင်းခြင်း { #location-and-collection }

```text
C:\$MFT
```

`$MFT` သည် ကာကွယ်ထားသော NTFS metadata ဖြစ်သည်။ ၎င်းကို raw NTFS ဝင်ရောက်နိုင်သော forensic acquisition သို့မဟုတ်
triage tool ဖြင့် စုဆောင်းပါ။ သာမန် user-mode ကူးယူခြင်းဖြင့် ပုံမှန်အားဖြင့် ၎င်းကို မဖတ်နိုင်ပါ။ ဖြစ်နိုင်ပါက `$J`၊
`$LogFile` နှင့် volume metadata တို့နှင့် တွဲ၍ စုဆောင်းပါ။

## စိစစ်သူများ သတိပြုရန် { #analyst-cautions }

- ဖျက်ပစ်ထားသော record တစ်ခုကို တစ်စိတ်တစ်ပိုင်း ထပ်ရေးခံရနိုင်သည် သို့မဟုတ် ၎င်း၏ record နံပါတ်ကို ပြန်လည်အသုံးပြုနိုင်သည်။
- `SN` သည် တူညီသော entry နံပါတ်၏ မျိုးဆက်များကို ခွဲခြားပေးသည်။ Entry **နှင့်** sequence နှစ်ခုစလုံးကို
  တိုက်ဆိုင်ပါ။
- `$STANDARD_INFORMATION` (`0x10`) နှင့် `$FILE_NAME` (`0x30`) timestamp များသည် မတူညီသော အပ်ဒိတ်အပြုအမူ ရှိသည်။
  မည်သည့်တစ်ခုမျှ ခြယ်လှယ်ခြင်းမှ ကင်းလွတ်ခြင်း မရှိပါ။
- `SI<FN`, `uSecZeros` နှင့် `Copied` တို့သည် သဲလွန်စများဖြစ်ပြီး အလိုအလျောက် timestomping ဆုံးဖြတ်ချက်များ မဟုတ်ပါ။
- Hard link များသည် ဖိုင် record တစ်ခုအတွက် `$FILE_NAME` attribute နှင့် path များစွာကို ထုတ်ပေးသည်။
- NTFS သည် အမည်များစွာ (DOS 8.3 အပါအဝင်)၊ alternate data stream များနှင့် extension record များကို သိမ်းဆည်းနိုင်သည်။
- Access-time အပ်ဒိတ်များကို ပိတ်ထားခြင်း၊ နှောင့်နှေးခြင်း သို့မဟုတ် ပေါင်းစည်းခြင်း ဖြစ်နိုင်သည်။
- Path အပြည့်အစုံကို parent record များမှ ပြန်လည်တည်ဆောက်သည်။ `[Orphaned]`, `[Unknown]` သို့မဟုတ် `PathUnknown`
  သည် ပျောက်နေသော/ပြန်လည်အသုံးပြုထားသော/ပျက်စီးနေသော ဘိုးဘွားဆက်နွယ်မှုကို ညွှန်ပြသည်—စာသားအတိုင်း directory အမည်
  မဟုတ်ပါ။
- FILE record တစ်ခု၏ **slack** (အသုံးပြုနေသော အရွယ်အစားကို ကျော်လွန်သော ၎င်း၏ byte များ) သည် ၎င်း ယခင်က ထိန်းသိမ်းခဲ့သော
  အမည်များကို ထိန်းသိမ်းထားနိုင်သည်- index ကို index allocation သို့ ရွှေ့ခဲ့သော directory အသေး၏ index entry များ
  (၎င်းတို့အများစုသည် ထိုနေရာတွင် အသုံးပြုဆဲ ဖြစ်သည်)၊ ယခင်အမည် သို့မဟုတ် ပြန်လည်အသုံးပြုထားသော record ၏ ယခင်ပိုင်ရှင်။
  ပြန်လည်ရယူထားသော အမည်များတွင် `$STANDARD_INFORMATION` မရှိပါ၊ အမည်တစ်ခု၏ entry header ကို ထပ်ရေးခံရခဲ့နိုင်သည်။
  Record ၏ အသုံးပြုနေသော အရွယ်အစား သို့မဟုတ် attribute များက ၎င်း၏ slack စတင်သည့်နေရာကို မဖော်ပြသည့်အခါ ၎င်းမှ
  မည်သည့်အရာမျှ ပြန်လည်မရယူဘဲ ၎င်း၏ row ၏ `ParseErrors` တွင် `slack not read` နှင့် အကြောင်းရင်းကို ဖော်ပြသည်။

## Juntei ဖြင့် ခွဲခြမ်းစိတ်ဖြာခြင်း { #parse-with-juntei }

```text
Usage: juntei parse-ntfs mft <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find $MFT files
  -f, --file <FILE>      Parse one $MFT

Filtering:
  -r, --ranges <RANGES>  Parse selected entry numbers (example: 0-15,30)

Output:
  -o, --output <PATH>                   Save records
  -t, --output-type <TYPE>              csv, ys-ntfs-parser-csv, json, jsonl
  -e, --extract-resident-streams <DIR>  Extract resident $DATA streams
      --reconstruct-filesystem <DIR>  Recreate names and resident content in a new tree
      --reconstruction-i30 <FILE>      Supplement the tree with an I30 file (repeatable)

CSV Output:
  -a, --all-times      Include every $FILE_NAME time
  -s, --short-names    Include DOS 8.3-name rows
      --cutoff <DATE>  Keep recent SI-created or SI-modified records

Records:
  -x, --recover-slack  Recover file names from FILE record slack

Resident Data:
  -i, --include-resident-data      Put resident $DATA in the CSV
  -E, --resident-extensions <EXT>  Only these extensions (ex: ps1,bat,vbs,js)
  -M, --resident-max-size <BYTES>  Only data up to this size

Inspection:
  -D, --dump-entry <ENTRY>           Show one entry in full detail
  -L, --list-directory <ENTRY|PATH>  List a directory's contents
```

```bash
juntei parse-ntfs mft -f '$MFT' -o mft.csv
juntei parse-ntfs mft -f '$MFT' --cutoff '2026-10-20T00:00:00Z' -a -o recent.csv
juntei parse-ntfs mft -f '$MFT' -t jsonl -r 32000-32100 | jq .
juntei parse-ntfs mft -f '$MFT' -x -o mft-with-slack.csv
juntei parse-ntfs mft -f '$MFT' -i -E ps1,bat,vbs,js -o scripts.csv
juntei parse-ntfs mft -f '$MFT' -D 0x270-5
juntei parse-ntfs mft -f '$MFT' -L '\Users\alice\Downloads'
```

`-e` ဖြင့် ထုတ်ယူထားသော resident stream များကို ဘယ်သောအခါမျှ မ run ပါနှင့်။ Juntei က ၎င်းတို့ကို `.dontrun` အမည်များ
ပေးထားခြင်းမှာ အကြောင်းရင်းရှိသည်။ `-i` ၏ resident data column များသည် တူညီသော byte များ ဖြစ်သည်- ၎င်းတို့ကို
မယုံကြည်ရသော အကြောင်းအရာအဖြစ် သဘောထားပါ။

`-D` သည် entry တစ်ခုကို ပုံနှိပ်ပြသည်- ၎င်း၏ header၊ attribute တိုင်း (data run များနှင့်အတူ) နှင့် `-t json -x` က
ရေးသားသည့်အတိုင်း ၎င်း၏ slack ရှိ အမည်များ၊ ထို့နောက် record ၏ byte များ။ `-L` သည် directory တစ်ခုရှိ အမည်တိုင်းကို
အသုံးပြုဆဲနှင့် ဖျက်ပစ်ထားသည်များ နှစ်မျိုးစလုံး၊ ၎င်း၏ entry နှင့် sequence နံပါတ်၊ အမျိုးအစား၊ အရွယ်အစားနှင့် ပြင်ဆင်ချိန်တို့နှင့်အတူ
ဖော်ပြသည်။ Directory ကို entry (`30`, `0x1E`, `30-1`) ဖြင့် သို့မဟုတ် path (`\Windows\System32`, `C:/Windows`၊
စာလုံးအကြီးအသေး မခွဲခြားပါ) ဖြင့် ပေးသည်။

## CSV ထုတ်ယူမှု field များ { #csv-output-fields }

| Field | အဓိပ္ပာယ်ဖော်ပုံ |
|---|---|
| `EntryNum` (Entry Number) | MFT record နံပါတ်။ `SN` နှင့် ပေါင်းစပ်ပါ- record နံပါတ်များကို ပြန်လည်အသုံးပြုသည်။ |
| `SN` (Sequence Number) | NTFS reference များက အသုံးပြုသော မျိုးဆက် counter။ မကိုက်ညီမှုသည် ခေတ်မမီတော့သော/ဖျက်ပစ်ထားသော ဆက်နွယ်မှုများကို ဖော်ပြနိုင်သည်။ |
| `InUse` | စုဆောင်းချိန်က allocation flag။ `false` ဆိုသည်မှာ record သည် free/ဖျက်ပစ်ထားခြင်း ဖြစ်ပြီး byte အားလုံး မပျက်မစီး ရှိသည်ဟု မဆိုလိုပါ။ |
| `ParentEntryNum` (ParentEntryNumber), `ParentSN` (Parent Sequence Number) | `$FILE_NAME` မှ parent directory reference။ အပိုင်းနှစ်ခုစလုံးကို အတည်ပြုပါ။ |
| `ParentPath` | ပြန်လည်တည်ဆောက်ထားသော parent path။ Unknown/orphan အမှတ်အသားများက မဖြေရှင်းနိုင်သေးသော ဘိုးဘွားဆက်နွယ်မှုကို ဖော်ပြသည်။ |
| `File` (Filename) | လက်ရှိ row ၏ `$FILE_NAME` သို့မဟုတ် stream မှ အမည်။ Record တစ်ခုတွင် row များစွာ ရှိနိုင်သည်။ |
| `Ext` (Extension) | အမည်မှ ခွဲထုတ်ထားသော အဆင်ပြေစေရန် extension။ ယုံကြည်ရသော ဖိုင်အမျိုးအစား ဆုံးဖြတ်ချက် မဟုတ်ပါ။ |
| `Size` (File Size) | Row/stream နှင့် ဆက်စပ်သော logical အရွယ်အစား။ Sparse/compressed allocation သည် ကွဲပြားနိုင်သည်။ |
| `RefCnt` (Reference Count) | NTFS hard-link reference အရေအတွက်။ ကျန်ရှိနေသော အမည်အရေအတွက်နှင့် သတိထား၍ နှိုင်းယှဉ်ပါ။ |
| `ReparseTarget` | Junction/symlink ကဲ့သို့ ပံ့ပိုးထားသော reparse point များအတွက် decode လုပ်ထားသော ပစ်မှတ်။ Path များ လမ်းကြောင်းလွှဲနိုင်သည်။ |
| `Dir` (Directory) | Record တွင် directory flag ရှိသည်။ |
| `HasADS` (Has Alternate Data Streams) | Record တွင် အမည်ပါသော alternate data stream များ ပါဝင်သည်။ ၎င်းက အန္တရာယ်ရှိစေသည် မဟုတ်ပါ။ |
| `IsADS` (Is Alternate Data Stream) | ဤ row သည် အမည်မဲ့ ဖိုင်ဒေတာ မဟုတ်ဘဲ အမည်ပါသော alternate data stream ကို ကိုယ်စားပြုသည်။ |
| `SI<FN` (Standard Information Before File Name) | SI ဖန်တီးချိန်သည် ကွဲပြားသော FN ဖန်တီးချိန်ထက် စောသည်။ ကူးယူခြင်း/timestomping နှင့် တရားဝင်အကြောင်းရင်းများကို စုံစမ်းပါ။ |
| `uSecZeros` (Microsecond Zeros) | SI ဖန်တီးချိန်/ပြင်ဆင်ချိန်သည် millisecond နယ်နိမိတ်ပေါ်တွင် အတိအကျ ကျရောက်သည်။ အားနည်းသော timestomping heuristic သာ ဖြစ်သည်။ |
| `Copied` | ပြင်ဆင်ချိန်သည် ဖန်တီးချိန်ထက် စောသည်။ ဖိုင်ကူးယူခြင်းနှင့် ကိုက်ညီလေ့ရှိသည်။ တိုက်ခိုက်သူ၏ လုပ်ဆောင်ချက် သက်သေ မဟုတ်ပါ။ |
| `SI-Flags` (Standard Information Flags) | `$STANDARD_INFORMATION` ဖိုင် attribute/flag များ။ Hidden/system flag များသည် အသုံးဝင်သော pivot များ ဖြစ်နိုင်သည်။ |
| `Type` (Name Type) | `$FILE_NAME` namespace (Win32၊ DOS၊ POSIX သို့မဟုတ် ပေါင်းစပ်)။ အမည်များကို နှစ်ကြိမ်ရေတွက်ခြင်းမှ ရှောင်ရန် အသုံးပြုပါ။ |
| `DataModified (SI)`, `Accessed (SI)`, `MetadataChanged (SI)`, `Created (SI)` | SI အချိန်များ။ အသုံးများသော timeline field များဖြစ်သော်လည်း user-mode tool များက ၎င်းတို့အနက် အများအပြားကို ပြောင်းလဲနိုင်သည်။ |
| `DataModified (FN)`, `Accessed (FN)`, `MetadataChanged (FN)`, `Created (FN)` | FN အချိန်များ။ ကွဲပြားစွာ အပ်ဒိတ်လုပ်ပြီး ပုံမှန် API များဖြင့် ပြောင်းလဲရန် ယေဘုယျအားဖြင့် ပိုခက်သည်။ သို့သော် ပြောင်းလဲ၍မရသည် မဟုတ်ပါ။ |
| `USN` (Update Sequence Number) | Record ၏ SI attribute နှင့် ဆက်စပ်သော နောက်ဆုံး USN။ ထို USN ကျန်ရှိနေပါက `$J` သို့ pivot လုပ်ပါ။ |
| `LogfileSN` (Logfile Sequence Number) | FILE record header မှ NTFS `$LogFile` sequence number။ အနိမ့်ပိုင်းအဆင့် transaction ခွဲခြမ်းစိတ်ဖြာမှုတွင် အသုံးဝင်သည်။ |
| `SID` (Security ID) | NTFS `$Secure` သို့ index။ ၎င်းကိုယ်တိုင် Windows SID မဟုတ်ပါ။ ဖိုင်၏ ပိုင်ရှင်နှင့် ACL များအတွက် ၎င်းကို [`parse-ntfs secure-sds`](secure-sds.md) ၏ `SecurityID` နှင့် ပေါင်းစပ်ပါ။ |
| `ObjectID-FileDroid` | ရှိပါက Object ID / distributed-link-tracking identifier။ LNK/Jump List droid များနှင့် ချိတ်ဆက်စစ်ဆေးပါ။ |
| `LoggedUtilStream` | ပံ့ပိုးထားသည့်နေရာတွင် decode လုပ်ထားသော `$LOGGED_UTILITY_STREAM` အခြေအနေ။ မရင်းနှီးသော တန်ဖိုးများကို ပြန်လည်သုံးသပ်ပါ။ |
| `ZoneID-Contents` | ZoneId နှင့် download URL များကဲ့သို့ resident `Zone.Identifier` အကြောင်းအရာ။ မရှိခြင်း၊ ဖယ်ရှားထားခြင်း သို့မဟုတ် non-resident ဖြစ်နိုင်သည်။ |
| `SourceFile` | Input `$MFT`၊ host များ သို့မဟုတ် volume များကို ပေါင်းစပ်သည့်အခါ မရှိမဖြစ် လိုအပ်သည်။ |
| `ResidentDataBase64`, `ResidentDataHex`, `ResidentDataASCII` | `-i` ဖြင့်- row ၏ resident `$DATA` (ဖိုင်၏ သို့မဟုတ် alternate data stream ၏) အကြောင်းအရာကို Base64 အဖြစ်၊ byte များ (`41-42`) အဖြစ်နှင့် ပုံနှိပ်နိုင်သော ASCII ဖြစ်ပါက စာသားအဖြစ်။ ဒေတာသည် non-resident ဖြစ်သည့်အခါ သို့မဟုတ် `-E`/`-M` က စစ်ထုတ်လိုက်သည့်အခါ ဗလာ။ |
| `ParseErrors` | ဤ row ကို သက်ရောက်သော ပျက်စီးမှု သို့မဟုတ် ပံ့ပိုးမထားသော ဆက်နွယ်မှုများ။ စစ်ထုတ်စဉ် ဤ column ကို ဘယ်သောအခါမျှ မပယ်ပါနှင့်။ |
| `FromSlack` | `-x` ဖြင့်- FILE record ၏ slack မှ ပြန်လည်ရယူထားသော အမည်အတွက် `True`။ ၎င်း၏ `EntryNum` နှင့် `SN` သည် အမည်က ညွှန်ပြခဲ့သော ဖိုင် ဖြစ်သည် (၎င်း၏ entry header ကို ထပ်ရေးခံရပါက ဗလာ)၊ ပြန်လည်ရယူထားသော timestamp များသည် `(FN)` column များတွင် ရှိပြီး `InUse` သည် `False` ဖြစ်သည်။ |
| `SlackRecord`, `SlackOffset` | `-x` ဖြင့်- အမည်ကို ၎င်း၏ slack တွင် ထိန်းသိမ်းထားသော FILE record (၎င်းအတွင်း ရေးထားသော record နံပါတ်ဖြင့် မဟုတ်ဘဲ `$MFT` ရှိ ၎င်း၏ နေရာဖြင့်၊ ၎င်း၏ အမည်များကို base record နှင့်အတူ ဖော်ပြထားသော်လည်း extension record ၏ ကိုယ်ပိုင်) နှင့် `$MFT` ရှိ အမည်၏ byte offset။ Slack row ၏ `ParseErrors` သည် ထို record ၏ ပြဿနာများ ဖြစ်သည်၊ ဥပမာ `fixup values do not match`- ပြန်လည်ရယူထားသော byte များ ထိခိုက်နိုင်သည်။ |
| `CopyOfEntryInUse` | `-x` ဖြင့်- အသုံးပြုဆဲ ဖိုင်တစ်ခုတွင် တူညီသော directory ၌ ဤအမည် ရှိနေဆဲ ဖြစ်သည် (reference ကျန်ရှိပါက တူညီသော ဖိုင် ဖြစ်သည်)။ အသုံးပြုဆဲ ဖိုင်တစ်ခုမျှ မပိုင်ဆိုင်သော အမည်များသည် စိတ်ဝင်စားဖွယ်ရာများ ဖြစ်သည်- ဖျက်ပစ်ထားသော၊ အမည်ပြောင်းထားသော သို့မဟုတ် ပြန်လည်အသုံးပြုထားသော record ၏ ယခင်ပိုင်ရှင်။ |

Timestamp column များသည် MACB အစီအစဉ် (Modified, Accessed, Changed, Birth/creation) ကို လိုက်နာသည်-
`DataModified` SI/FN၊ `Accessed` SI/FN၊ `MetadataChanged` SI/FN၊ ထို့နောက် `Created` SI/FN။

`(SI)` ဆိုသည်မှာ `$STANDARD_INFORMATION` ဖြစ်ပြီး `(FN)` ဆိုသည်မှာ `$FILE_NAME` ဖြစ်သည်။ ဤအမည်များသည် ပြီးပြည့်စုံသော
record အတွက် ပုံမှန် ရင်းမြစ်ကို ဖော်ပြသည်။ SI attribute မပါသော ပုံမှန် filename row အတွက် ရှိပြီးသား fallback သည်
FN ဖန်တီးချိန်ကို `Created (FN)` တွင် ထားပြီး FN ပြင်ဆင်ချိန်/record ပြောင်းလဲချိန်/ဝင်ရောက်ချိန် တန်ဖိုးများကို
`DataModified (SI)`, `MetadataChanged (SI)` နှင့် `Accessed (SI)` တွင် အသီးသီး ထားသည်။
ပြန်လည်ရယူထားသော slack row များသည် အချိန်လေးခုလုံးကို `(FN)` column များတွင် ထားပြီး SI column များကို ဗလာထားသည်။
ပုံမှန် row ၏ fallback တန်ဖိုးများသည် လွတ်လပ်သော SI အထောက်အထား **မဟုတ်ပါ**။ Timestamp ရင်းမြစ်ကို ဆုံးဖြတ်သည့်အခါ မူရင်း
JSON attribute များနှင့် slack ၏ ရင်းမြစ်မှတ်တမ်းကို အသုံးပြုပါ။

`-t ys-ntfs-parser-csv` (ယခင်က `ys-mft-parser-csv`၊ ယခုလည်း လက်ခံဆဲ) သည် ထိန်းသိမ်းထားသော parser ၏ entry
ဗဟိုပြု schema ကို အသုံးပြုသည်။ ဤဖတ်ရလွယ်သော column အမည်များသည် `auto-parse` ၏ `MFT.csv` အပါအဝင် ပုံသေ `-t csv`
output နှင့် သက်ဆိုင်သည်။ JSON/JSONL သည် attribute တိုင်း (resident `$DATA` ကို hexadecimal ဖြင့် အပါအဝင်) နှင့်
`FullPath` ကို ထိန်းသိမ်းထားသည်။ Flat စုံစမ်းစစ်ဆေးရေး CSV တွင် အနိမ့်ပိုင်းအဆင့် ဖွဲ့စည်းပုံ ချန်လှပ်ထားသည့်အခါ ၎င်းကို
အသုံးပြုပါ။ `-x` ဖြင့် JSON entry တစ်ခုစီတွင် `SlackFileNames` လည်း ပါဝင်သည်။

## စုံစမ်းစစ်ဆေးမှု ဗျူဟာ { #investigation-strategy }

1. အချိန်ကာလတစ်ခုကို စစ်ထုတ်ပါ၊ သို့သော် ဖျက်ပစ်ထားသော record များနှင့် timestamp အစုံ နှစ်ခုစလုံးကို ထိန်းသိမ်းထားပါ။
2. Path အပြည့်အစုံ၊ သံသယဖြစ်ဖွယ် double extension များ၊ ADS၊ staging directory များနှင့် သိရှိထားသော hash/အမည်များကို
   ရှာဖွေပါ။ `-x` ဖြင့် အသုံးပြုဆဲ ဖိုင်တစ်ခုမျှ မပိုင်ဆိုင်သော slack အမည်များ (`CopyOfEntryInUse` `False`) ကို
   ထည့်သွင်းပါ။ `-i -E ps1,bat,vbs,js` ဖြင့် script အသေးများကို CSV မှ တိုက်ရိုက် ဖတ်ပါ။
3. Entry/sequence နှင့် `USN` ကို USN Journal သို့ pivot လုပ်ပါ။
4. Prefetch/LNK/Jump List MFT reference များကို ပစ်မှတ် record နှင့် နှိုင်းယှဉ်ပါ။
5. ဖျက်ပစ်ခြင်းကို Defender quarantine သို့မဟုတ် USN `FILE_DELETE`/အမည်ပြောင်းခြင်း လှုပ်ရှားမှုနှင့် ချိတ်ဆက်စစ်ဆေးပါ။

## နောက်ထပ် အသေးစိတ်များ { #further-details }

- [Microsoft: Master File Table](https://learn.microsoft.com/en-us/windows/win32/fileio/master-file-table)
- [Microsoft Open Specifications: NTFS](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-ntfs/)
- [ys-ntfs-parser ၏ mft crate](https://github.com/Yamato-Security/ys-ntfs-parser/tree/main/mft)
- [Omer Ben-Amram ၏ mft parser](https://github.com/omerbenamram/mft)
- [libfsntfs format စာရွက်စာတမ်း](https://github.com/libyal/libfsntfs/tree/main/documentation)

ဥပမာသည် ကြည့်ရှုနိုင်သော ဖိုင်လမ်းကြောင်းကို ပြန်တည်ဆောက်သည်။ Non-resident အကြောင်းအရာသည် ဖိုင်အလွတ်ဖြစ်သည်။ အသေးစိတ်: [ဖိုင်စနစ် ပြန်လည်တည်ဆောက်ခြင်း](../commands/auto-parse.md#filesystem-reconstruction).

```bash
juntei parse-ntfs mft -f '$MFT' --reconstruct-filesystem reconstructed
```
