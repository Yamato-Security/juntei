# အမှိုက်ပုံး (Recycle Bin) { #recycle-bin }

## Recycle Bin က မှတ်တမ်းတင်ထားသည့်အရာ { #what-the-recycle-bin-records }

အသုံးပြုသူက File Explorer တွင် ဖိုင် သို့မဟုတ် ဖိုင်တွဲတစ်ခုကို ဖျက်လိုက်သောအခါ Windows သည် ၎င်းကို ၎င်းတည်ရှိရာ
volume ၏ Recycle Bin ထဲသို့ ရွှေ့ပြီး ဘေးတွင် index record အသေးတစ်ခုကို ရေးထားသည်။ ထို record တွင် item ၏ မူလ path၊
အရွယ်အစားနှင့် ဖျက်ခဲ့သည့်အချိန် ပါဝင်သည်။ အသုံးပြုသူတစ်ဦးစီတွင် volume တစ်ခုစီပေါ်၌ ၎င်း၏ SID ဖြင့် အမည်ပေးထားသော
bin ဖိုင်တွဲ ရှိသောကြောင့် ထိုဖိုင်တွဲက item ကို မည်သည့်အကောင့်က ဖျက်ခဲ့သည်ကိုလည်း ပြသသည်။

| Windows ဗားရှင်း | Index | ဖျက်ထားသော အကြောင်းအရာ |
|---|---|---|
| Vista နှင့် နောက်ပိုင်း | item တစ်ခုလျှင် `$I…` ဖိုင်တစ်ခု (Vista မှ 8.1 တွင် format version 1၊ Windows 10 နှင့် နောက်ပိုင်းတွင် version 2) | `$R…` (`$R` နောက်ရှိ အမည်သည် index ဖိုင်နှင့် တူသည်): ဖိုင်ကိုယ်တိုင် သို့မဟုတ် အတွင်းရှိ အကြောင်းအရာများပါဝင်သော ဖိုင်တွဲ |
| NT 4.0၊ 2000၊ XP၊ Server 2003 | bin တစ်ခုလျှင် `INFO2` ဖိုင်တစ်ခု: ANSI path နှင့် Unicode path ပါသော 800-byte record များ | `D<drive letter><index><extension>`၊ ဥပမာ `Dc1.txt` |
| 95၊ 98၊ ME | `INFO2` ဖိုင်တစ်ခု (Windows 95 တွင် `INFO`): ANSI path သာ ပါသော 280-byte record များ | အလားတူ `D…` အမည်များ |

<span class="evidence-strength">အကောင်းဆုံး ကောက်ချက်:</span> record ပါရှိသော bin ကို ပိုင်ဆိုင်သည့် အကောင့်သည်
မူလ path ရှိ item ကို မှတ်တမ်းတင်ထားသော အချိန်တွင် Recycle Bin သို့ ရွှေ့ခဲ့သည်။ Shift+Delete ဖြင့် ဖျက်ခြင်း၊ command
line မှ ဖျက်ခြင်း သို့မဟုတ် program အများစုမှ ဖျက်ခြင်းသည် bin ကို ကျော်သွားပြီး မည်သည့် record မျှ မကျန်ခဲ့ပါ။

## တည်နေရာများနှင့် စုဆောင်းခြင်း { #locations-and-collection }

```text
C:\$Recycle.Bin\<SID>\$I…        Windows Vista and later (each volume has its own bin)
C:\$Recycle.Bin\<SID>\$R…        the deleted file or folder
C:\RECYCLER\<SID>\INFO2          Windows NT 4.0, 2000, XP and Server 2003 (NTFS)
C:\RECYCLED\INFO2                Windows 95, 98 and ME, and FAT volumes (INFO on Windows 95)
```

volume တိုင်း၏ bin မှ `$I` ဖိုင်များ သို့မဟုတ် INFO2 ကို စုဆောင်းပါ။ `$R` နှင့် `D…` ဖိုင်များကိုပါ စုဆောင်းလျှင်
ဖျက်ထားသော item များအနက် မည်သည်တို့ ကျန်ရှိနေသေးသည်ကို သိရသည်။ ၎င်းတို့ မပါလျှင် item တိုင်းအတွက်
`RecycledFileFound` သည် `false` ဖြစ်မည်။ ဖျက်ထားသော အကြောင်းအရာသည် ၎င်းကိုယ်တိုင် အထောက်အထား ဖြစ်ပြီး malware
လည်း ဖြစ်နိုင်သည်- ထိုသို့ သဘောထား၍ ကိုင်တွယ်ပါ။

## စိစစ်သူများ သတိပြုရန် { #analyst-cautions }

- `DeletedTime` သည် item ကို bin ထဲသို့ ရွှေ့ခဲ့သည့်အချိန် ဖြစ်ပြီး စနစ်နာရီအတိုင်း UTC FILETIME အဖြစ် မှတ်တမ်းတင်ထားသည်။
  ၎င်းသည် bin ကို ရှင်းလင်း (empty) ခဲ့သည့်အချိန် မဟုတ်ပါ။ item ၏ ကိုယ်ပိုင် ဖန်တီးချိန်နှင့် ပြင်ဆင်ချိန်တို့သည် `$MFT`
  နှင့် `$R` ဖိုင်တွင် ရှိသည်။
- SID သည် item ကို ဖျက်ခဲ့သော အကောင့် ဖြစ်ပြီး ဖိုင်၏ ပိုင်ရှင် ဖြစ်ရမည်ဟု မဆိုလိုပါ။ SOFTWARE hive ၏ `ProfileList` တွင် profile
  မရှိသော SID (ဖျက်ထားသော အကောင့် သို့မဟုတ် တစ်ခါမျှ logon မဝင်ခဲ့ဖူးသော domain အကောင့်) တွင် `User` မရှိပါ။
- bin ကို ရှင်းလင်းသောအခါ သို့မဟုတ် item တစ်ခုကို bin မှ restore ပြန်လုပ်သောအခါ သို့မဟုတ် bin ထဲမှ ဖျက်လိုက်သောအခါ
  Windows Vista နှင့် နောက်ပိုင်းသည် ၎င်း၏ `$I` နှင့် `$R` ဖိုင်များကို ဖျက်သည်။ ၎င်းတို့၏ အမည်များသည် USN Journal၊
  `$MFT` နှင့် `$I30` slack တွင် ကျန်ရှိနေနိုင်သော်လည်း Juntei သည် ထိုနေရာများမှ ဖျက်ထားသော `$I` record များကို
  ပြန်လည်မရယူပါ။ INFO2 မူ ထိုကဲ့သို့သော record များကို ဆက်ထားပြီး ၎င်းတို့၏ ANSI path ၏ ပထမ byte ကို ရှင်းပစ်သည်-
  ထိုအခါ `RemovedFromBin` သည် `true` ဖြစ်ပြီး Juntei က ထို character ကို drive နံပါတ်မှ ပြန်ဖြည့်ပေးသည်။
- `$I` ဖိုင်ရှိ `FileSize` သည် item ၏ အရွယ်အစား ဖြစ်ပြီး ဖိုင်တွဲအတွက်မူ ၎င်းအတွင်းရှိ အကြောင်းအရာများ၏ စုစုပေါင်း
  အရွယ်အစား ဖြစ်သည်။ INFO2 တွင်မူ item က disk ပေါ်တွင် ယူထားသော နေရာ ဖြစ်ပြီး cluster အပြည့်များဖြင့် တွက်သည်
  (ဖိုင်ငယ်တစ်ခုအတွက် 4,096 byte)။
- INFO2 ၏ ANSI path သည် 8.3 ပုံစံ ဖြစ်လေ့ရှိပြီး စနစ်၏ code page လိုအပ်သည်- ဂျပန် Windows အတွက် `--code-page 932`၊
  တရုတ်အတွက် 936 သို့မဟုတ် 950 ကို သုံးပါ။ Unicode path ရှိလျှင် ၎င်းသည် `OriginalPath` ဖြစ်သည်။
- Windows Vista စနစ်အချို့သည် size field တစ် byte တိုနေသော 543-byte `$I` ဖိုင်များကို ရေးခဲ့သည်။ Juntei သည်
  ၎င်းတို့၏ အချိန်နှင့် path ကို ဖတ်ပြီး `FileSize` ကို ဗလာထားကာ problem တစ်ခုအဖြစ် မှတ်သားသည်။
- `RecycledFileFound` သည် endpoint ကို မဟုတ်ဘဲ စုဆောင်းထားသော collection ကို ဖော်ပြသည်- `false` ဆိုသည်မှာ
  collector က `$R` ဖိုင်များကို မယူခဲ့ခြင်း သို့မဟုတ် item သည် bin ထဲတွင် မရှိတော့ခြင်း ဖြစ်နိုင်သည်။
- စနစ်မဟုတ်သော volume များရှိ bin များအတွက် collection metadata သို့မဟုတ် သိရှိထားသော drive/volume folder ဖွဲ့စည်းပုံအရ တူညီသော ကွန်ပျူတာ၏ profile ကိုသာ အသုံးပြုသည်။ သက်ဆိုင်မှု မရှင်းလင်းလျှင် အသုံးပြုသူကို မသိရှိဟု ထားပြီး အခြား host ၏ profile ကို မယူပါ။

## Juntei ဖြင့် ခွဲခြမ်းစိတ်ဖြာခြင်း { #parse-with-juntei }

```text
Usage: juntei parse-file recycle-bin <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find $I files and INFO2
  -f, --file <FILE>      Parse one $I or INFO2 file (told apart by its contents if renamed)
      --code-page <CP>   INFO2's ANSI code page (default 1252; Japanese commonly 932)

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-file recycle-bin -d evidence/ -o recycle-bin.csv
juntei parse-file recycle-bin -d xp-evidence/ --code-page 932 -o recycle-bin-ja.csv
juntei parse-file recycle-bin -f '$IABC123.docx' -t jsonl | jq .
```

`-d` ဖြင့်ဆိုလျှင် Juntei သည် `$I` နောက်တွင် စာလုံး သို့မဟုတ် ဂဏန်း ခြောက်လုံး လိုက်သော အမည်ရှိ ဖိုင်များ (extension
မည်သည်ဖြစ်စေ)၊ `INFO2` နှင့် `RECYCLED` ဖိုင်တွဲထဲရှိ `INFO` ကို ဖတ်သည်။ `-f` ဖြင့်ဆိုလျှင် ဖိုင်အမည် မည်သို့ပင်
ဖြစ်စေ ထိုဖိုင်ကို ဖတ်သည်။

## CSV ထုတ်ယူမှု field များ { #csv-output-fields }

| Field | အဓိပ္ပာယ် |
|---|---|
| `SourceFile` | `$I` သို့မဟုတ် INFO2 ဖိုင်။ |
| `DeletedTime` | item ကို bin ထဲသို့ ရွှေ့ခဲ့သည့်အချိန် (UTC)။ |
| `OriginalPath` | item ရှိခဲ့သော နေရာ။ INFO2 အတွက် record တွင် Unicode path ပါလျှင် ၎င်းကို၊ မပါလျှင် ANSI path ကို သုံးသည်။ |
| `FileSize` | `$I`: item ၏ အရွယ်အစား byte ဖြင့် (ဖိုင်တွဲဆိုလျှင် စုစုပေါင်း)။ INFO2: disk ပေါ်တွင် ယူထားသော နေရာ၊ cluster အပြည့်များဖြင့်။ ဖတ်မရလျှင် ဗလာ။ |
| `User`, `SID` | bin ဖိုင်တွဲ၏ SID နှင့် ၎င်း၏ `ProfileList` profile အမည်မှ ရရှိသော အသုံးပြုသူ။ |
| `RecycledFile`, `RecycledFileFound` | index ဘေးရှိ ဖျက်ထားသော အကြောင်းအရာ၏ path (`$R…`၊ `D<drive><index><extension>`) နှင့် ၎င်းသည် collection ထဲတွင် ပါ/မပါ။ |
| `Format`, `FormatVersion` | `$I` 1 (Vista မှ 8.1) သို့မဟုတ် 2 (Windows 10 နှင့် နောက်ပိုင်း)၊ `INFO2` 0 (95)၊ 2 (NT 4.0)၊ 4 (98) သို့မဟုတ် 5 (ME၊ 2000 မှ Server 2003)။ |
| `RecordIndex`, `Drive`, `RecordOffset` | INFO2: record ၏ နံပါတ် (၎င်း၏ `D…` ဖိုင်အမည်ထဲရှိ နံပါတ်)၊ drive နှင့် INFO2 ထဲတွင် record စတင်သည့်နေရာ။ UNC path တွင် drive မရှိပါ။ |
| `RemovedFromBin` | INFO2: item ကို bin မှ restore ပြန်လုပ်ခဲ့လျှင် သို့မဟုတ် bin ထဲမှ ဖျက်ခဲ့လျှင် `true`။ |
| `ShortPath` | INFO2: `OriginalPath` နှင့် ကွဲပြားသည့်အခါ ANSI path (8.3 အမည်များ ဖြစ်လေ့ရှိသည်)။ |
| `Problems` | ပျက်စီးမှု: ဖြတ်တောက်ခံထားရသော သို့မဟုတ် ပုံမှန်ထက် ရှည်နေသော ဖိုင်၊ မမှန်ကန်သော drive နံပါတ်၊ မှန်ကန်သော UTF-16 မဟုတ်သည့် path၊ အချိန်အဖြစ် အဓိပ္ပာယ်မရသော ဖျက်ချိန်။ |

JSON နှင့် JSONL တွင် တူညီသော field များ ပါရှိပြီး ဂဏန်းများနှင့် boolean များကို ၎င်းတို့၏ type အတိုင်း ဖော်ပြကာ
record တွင် မရှိသော value များအတွက် `null` ကို သုံးပြီး `Problems` ကို list အဖြစ် ထားသည်။

Bin directory ကို စာရင်းမပြုနိုင်လျှင် သို့မဟုတ် content အမည် မရှင်းလင်းလျှင် `RecycledFileFound` သည် မသိရှိ (CSV cell အလွတ် သို့မဟုတ် JSON `null`) ဖြစ်ပြီး အကြောင်းရင်းကို `Problems` တွင် မှတ်တမ်းတင်သည်။ Hard link မှတစ်ဆင့် အပါအဝင် bin content ကို output ဖြင့် အစားမထိုးနိုင်သကဲ့သို့ ဖျက်ထားသော folder အတွင်းသို့လည်း မရေးနိုင်ပါ။ Index တစ်ခုမျှ မဖတ်နိုင်လျှင် ရှိပြီးသားရလဒ်များကို ထိန်းသိမ်းထားသည်။

## အလိုအလျောက် ခွဲခြမ်းစိတ်ဖြာခြင်း { #automatic-parsing }

`auto-parse` သည် `FileExistence/RecycleBin.csv` (`-t json` သို့မဟုတ် `-t jsonl` ဖြင့်ဆိုလျှင် `RecycleBin.json`
သို့မဟုတ် `RecycleBin.jsonl`) ကို `FileExistence/RecycleBin-logs.txt` နှင့်အတူ ရေးသားသည်။ ၎င်း၏ `--code-page` သည်
INFO2 အတွက် သက်ရောက်သည်။ Disk image များမှဆိုလျှင် `$I` ဖိုင်များနှင့် INFO2 ကို ကူးယူပြီး `$R` နှင့် `D…` ဖိုင်များမှာ
ဗလာ placeholder များအဖြစ် ရှိနေသောကြောင့် `RecycledFileFound` ကို ဆက်လက် ဖြည့်ပေးနိုင်သည်။ ပေါင်းစပ်ထားသော `-j` timeline တွင်
row တိုင်းကို `Artifact` identifier `recycle-bin` ဖြင့် ထည့်သွင်းပြီး `-Z` သည် `DeletedTime` ကို ပြောင်းလဲပေးကာ
[`super-timeline`](../commands/super-timeline.md) သည် item တစ်ခုလျှင် `Recycle Bin deletion (moved to the bin)`
event တစ်ခု ပြုလုပ်သည်။

## စုံစမ်းစစ်ဆေးမှု ဗျူဟာ { #investigation-strategy }

1. အသုံးပြုသူတစ်ဦးချင်းအလိုက် `DeletedTime` ဖြင့် စီပြီး သင်စုံစမ်းနေသော လှုပ်ရှားမှု၏ အချိန်များနှင့် နှိုင်းယှဉ်ပါ။
2. item ကို မဖျက်မီ အသုံးပြုခဲ့ခြင်း ရှိ/မရှိ သိရန် LNK၊ Jump Lists၊ ShellBags နှင့် RecentDocs တွင် `OriginalPath` ကို ရှာပါ။
3. USN Journal တွင် ဖျက်ခြင်းသည် `DeletedTime` ဝန်းကျင်တွင် `$Recycle.Bin` ထဲသို့ rename လုပ်ခြင်းအဖြစ် ပေါ်သည်။
   `$MFT` က item ၏ ကိုယ်ပိုင်အချိန်များကို ပေးသည်။
4. `RecycledFileFound` သည် `true` ဖြစ်လျှင် `$R` သို့မဟုတ် `D…` အကြောင်းအရာကို hash လုပ်ပြီး စစ်ဆေးပါ။
5. bin ကို ရှင်းလင်းထားပြီး ဖြစ်လျှင် USN Journal နှင့် `$MFT` တွင် `$I` နှင့် `$R` အမည်များကို ရှာပါ။

## နောက်ထပ် အသေးစိတ်များ { #further-details }

- [libyal dtformats — Windows Recycle.Bin file formats](https://github.com/libyal/dtformats/blob/main/documentation/Windows%20Recycle.Bin%20file%20formats.asciidoc)
- [libyal dtformats — Windows Recycler file formats](https://github.com/libyal/dtformats/blob/main/documentation/Windows%20Recycler%20file%20formats.asciidoc)
- [Plaso — Recycle Bin and INFO2 parsers](https://github.com/log2timeline/plaso/blob/main/plaso/parsers/recycler.py)
- [Velociraptor — Windows.Forensics.RecycleBin](https://github.com/Velocidex/velociraptor/blob/master/artifacts/definitions/Windows/Forensics/RecycleBin.yaml)
- [rifiuti2](https://github.com/abelcheung/rifiuti2)
