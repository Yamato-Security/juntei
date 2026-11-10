# USN ဂျာနယ် (`$UsnJrnl:$J`) { #usn-journal-usnjrnlj }

## USN Journal ဆိုသည်မှာ { #what-the-usn-journal-is }

NTFS သည် ဖိုင်နှင့် ဖိုင်တွဲ ပြောင်းလဲမှုများကို Update Sequence Number (USN) Change Journal တွင် မှတ်တမ်းတင်ပါသည်။
Record တစ်ခုသည် ဖန်တီးခြင်း၊ ဒေတာ တိုးချဲ့ခြင်း/ထပ်ရေးခြင်း၊ attribute သို့မဟုတ် security ပြောင်းလဲမှုများ၊ အမည်ပြောင်းခြင်း၊
ပိတ်ခြင်းနှင့် ဖျက်ခြင်းတို့ကို ဖော်ပြနိုင်ပါသည်။ ၎င်းသည် file reference၊ parent reference၊ အမည်၊ reason များ၊ attribute များ၊
timestamp နှင့် တစ်ဖက်သတ် တိုးလာသော USN ကို သိမ်းဆည်းပါသည်။

Journal သည် တိုက်ခိုက်သူ၏ ဖိုင်လုပ်ဆောင်ချက် အစုလိုက်ကို ပြန်လည်တည်ဆောက်ရန် အထူးကောင်းမွန်ပြီး ဖိုင်များကို ဖျက်ထားသည့်အခါ
ပို၍ပင် ဖြစ်ပါသည်။ ၎င်းသည် ပြောင်းလဲခဲ့သော byte များကို မဟုတ်ဘဲ **metadata/အကြောင်းအရာ ပြောင်းလဲခဲ့ကြောင်း** ကို မှတ်တမ်းတင်ပါသည်။

<span class="evidence-strength">အကောင်းဆုံး ကောက်ချက်:</span> NTFS သည် ကိုးကားထားသော ဖိုင်အတွက် ဖော်ပြထားသော
ပြောင်းလဲမှု reason များကို timestamp တွင် မှတ်တမ်းတင်ခဲ့ပါသည်။ Reason များသည် `CLOSE` record တစ်ခုအထိ မကြာခဏ စုပုံပါသည်။

## တည်နေရာနှင့် စုဆောင်းခြင်း { #location-and-collection }

```text
C:\$Extend\$UsnJrnl:$J
```

`$J` သည် NTFS metadata ၏ အမည်ပါ stream တစ်ခု ဖြစ်ပါသည်။ ၎င်းကို raw-aware forensic tool ဖြင့် စုဆောင်းပါ။ USN record များသည်
path အပြည့်အစုံအစား parent file reference များကို သိမ်းဆည်းသောကြောင့် တူညီသော volume ၏ `$MFT` ကိုလည်း စုဆောင်းပါ။

## စိစစ်သူများ သတိပြုရန် { #analyst-cautions }

- Journal သည် စက်ဝိုင်းပုံစံဖြစ်ပြီး size ကန့်သတ်ထားပါသည်။ Record အဟောင်းများ ထပ်ရေးခံရပါသည်။
- `FILE_DELETE` သည် NTFS က ဖျက်ခြင်းကို လုပ်ဆောင်ခဲ့ကြောင်း ဖော်ပြပါသည်။ အကြောင်းအရာကို အခြားနေရာတွင် ပြန်လည်ရယူနိုင်ဆဲ ဖြစ်နိုင်ပါသည်။
- `DATA_OVERWRITE` ကဲ့သို့ reason တစ်ခုသည် byte များ၊ တာဝန်ရှိသော process သို့မဟုတ် အသုံးပြုသူကို မဖော်ပြပါ။
- Record တစ်ခုတွင် reason အများအပြား ပေါင်းစပ်နိုင်ပါသည်။ `CLOSE` သည် စုပုံထားသော ပြောင်းလဲမှုများ၏ အဆုံးကို မကြာခဏ အမှတ်အသားပြုပါသည်။
- Path များကို ပြန်လည်တည်ဆောက်ထားခြင်း ဖြစ်ပါသည်။ Parent ဖိုင်တွဲများကို အမည်ပြောင်းခဲ့၊ ဖျက်ခဲ့ သို့မဟုတ် ပြန်လည်အသုံးပြုခဲ့နိုင်ပါသည်။
- File-reference sequence number များ အရေးကြီးပါသည်။ Entry number တစ်ခုတည်းသည် မှားယွင်းသော generation နှင့် ချိတ်ဆက်နိုင်ပါသည်။
- Carving သည် မှန်ကန်သော record များကို ၎င်းတို့၏ မူလ journal context မပါဘဲ ပြန်လည်ရယူနိုင်ပါသည်။ ထိုကွာခြားချက်ကို အစီရင်ခံပါ။

## Juntei ဖြင့် ခွဲခြမ်းစိတ်ဖြာခြင်း { #parse-with-juntei }

```text
Usage: juntei parse-ntfs usn-journal <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find $J files
  -f, --file <FILE>      Parse one $J, or any file with --carve
  -m, --mft <FILE>       Use the matching volume's $MFT for paths
      --carve            Search a disk image/unallocated file for records

Output:
  -o, --output <PATH>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-ntfs usn-journal -f 'C/$Extend/$J' -m 'C/$MFT' -o usn.csv
juntei parse-ntfs usn-journal -d evidence/ -o usn-jsonl -t jsonl
juntei parse-ntfs usn-journal -f unallocated.bin --carve -o carved-usn.csv
```

## CSV ထုတ်ယူမှု field များ { #csv-output-fields }

| Field | အဓိပ္ပာယ်ဖော်ချက် |
|---|---|
| `Name` | ဤ USN record တွင် သိမ်းဆည်းထားသော ဖိုင်/ဖိုင်တွဲ အမည်။ path အပြည့်အစုံ မဟုတ်ပါ။ |
| `Extension` | `Name` မှ parse လုပ်ထားသော အဆင်ပြေစေရန် extension။ ယုံကြည်ရသော content type အဖြစ် မသုံးပါနှင့်။ |
| `EntryNumber`, `SequenceNumber` | ဖိုင်၏ NTFS reference။ Record generation ကို ခွဲခြားရန် နှစ်ခုလုံးကို အသုံးပြုပါ။ |
| `ParentEntryNumber`, `ParentSequenceNumber` | ပြောင်းလဲမှုအချိန်က parent ဖိုင်တွဲ၏ NTFS reference။ |
| `ParentPath` | Juntei ၏ အချိန်ကို သိရှိ၍ ပြန်လည်တည်ဆောက်ထားသော parent path။ Unknown အမှတ်အသားများသည် ancestry ကို ဖြေရှင်းမရကြောင်း ဆိုလိုပါသည်။ |
| `UpdateSequenceNumber` | USN အစီအစဉ် တန်ဖိုး။ တည်ငြိမ်သော စီခြင်းနှင့် Amcache/MFT pivot များအတွက် အသုံးဝင်ပါသည်။ |
| `UpdateTimestamp` | V2/V3 record များတွင် သိမ်းဆည်းထားသော timestamp။ Carve လုပ်ထားသော context ၏ timestamp များကို မည်သည့် record မဆိုကဲ့သို့ မှန်ကန်မှု စစ်ဆေးမှုဖြင့် ကိုင်တွယ်ပါ။ |
| `UpdateReasons` | `|` ဖြင့် ဆက်ထားသော အမည်ပါ ပြောင်းလဲမှု flag တစ်ခု သို့မဟုတ် အများအပြား။ ပေါင်းစပ်မှုများနှင့် `CLOSE` ကို အတူတကွ အဓိပ္ပာယ်ဖော်ပါ။ |
| `FileAttributes` | ပြောင်းလဲမှုအချိန်က မှတ်တမ်းတင်ထားသော NTFS attribute များ။ မသိသော bit များသည် hexadecimal အဖြစ် ကျန်ရှိပါသည်။ |
| `OffsetToData` | Input အတွင်း byte offset။ မှန်ကန်မှု စစ်ဆေးခြင်းနှင့် carve လုပ်ထားသော record များအတွက် မရှိမဖြစ် လိုအပ်ပါသည်။ |
| `SourceFile` | Input journal/image။ host များနှင့် volume များကို ခွဲခြားပါသည်။ |
| `SourceInfo` | ပြောင်းလဲမှု journal ထဲသို့ မည်သို့ ဝင်ရောက်ခဲ့သည်ကို ဖော်ပြသော source flag များ။ များသောအားဖြင့် context ဖြစ်ပြီး attribution မဟုတ်ပါ။ |
| `SecurityID` | Record အတွက် NTFS security identifier index ဖြစ်ပြီး အသုံးပြုသူ SID မဟုတ်ပါ။ |
| `Version` | USN record ဗားရှင်း။ Juntei သည် V2/V3 ကို CSV သို့ ရေးသားပြီး JSON သည် V4 range-tracking extent များကို ထိန်းထားပါသည်။ |

JSON/JSONL သည် 128-bit V3 reference များကို မှန်ကန်စွာ ကိုယ်စားပြုပြီး V4 range-tracking record များနှင့် ၎င်းတို့၏ extent
များကို ထိန်းသိမ်းပါသည်။ အဆင့်နိမ့် ဗားရှင်းအလိုက် field များ အရေးကြီးသည့်အခါ ၎င်းကို အသုံးပြုပါ။

## စုံစမ်းစစ်ဆေးမှု ဗျူဟာ { #investigation-strategy }

1. အချိန်၊ အမည်၊ extension၊ parent path သို့မဟုတ် file reference ဖြင့် စစ်ထုတ်ပါ။
2. Journal ၏ တိကျသော အစီအစဉ်ကို မြင်ရန် အချိန်အပြင် `UpdateSequenceNumber` ဖြင့်လည်း စီပါ။
3. ရွှေ့ခြင်း/အမည်ပြောင်းခြင်းများအတွက် `RENAME_OLD_NAME` ကို `RENAME_NEW_NAME` နှင့် တွဲပါ။
4. Record အခြေအနေ၊ path များ၊ size များနှင့် timestamp များကို မှန်ကန်မှု စစ်ဆေးရန် `$MFT` ကို အသုံးပြုပါ။
5. Event log များ၊ EDR၊ Prefetch၊ service/task များ သို့မဟုတ် Defender မှ process attribution ကို ဆက်စပ်စစ်ဆေးပါ။

## နောက်ထပ် အသေးစိတ်များ { #further-details }

- [Microsoft: Change Journals](https://learn.microsoft.com/en-us/windows/win32/fileio/change-journals)
- [Microsoft: USN_RECORD_V2](https://learn.microsoft.com/en-us/windows/win32/api/winioctl/ns-winioctl-usn_record_v2)
- [Microsoft: Change Journal reason flag များ](https://learn.microsoft.com/en-us/windows/win32/api/winioctl/ns-winioctl-usn_record_v2#members)
- [ys-ntfs-parser ၏ usn crate](https://github.com/Yamato-Security/ys-ntfs-parser/tree/main/usn)
- [Sootmark usn](https://github.com/Sootmark/usn)
