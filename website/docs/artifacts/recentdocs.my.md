# RecentDocs { #recentdocs }

## RecentDocs ဆိုသည်မှာ { #what-recentdocs-is }

Windows Explorer သည် အသုံးပြုသူတစ်ဦးချင်းစီ၏ `NTUSER.DAT` တွင် နောက်ဆုံးအသုံးပြုခဲ့သော ဖိုင်များနှင့် ဖိုင်တွဲများ၏ စာရင်းများကို
သိမ်းဆည်းထားသည်။ Root စာရင်းသည် အမျိုးအစားအားလုံးရှိ မကြာသေးမီက item များကို ဖုံးလွှမ်းပြီး subkey များက `.docx` ကဲ့သို့သော
extension များအလိုက် အုပ်စုဖွဲ့ထားကာ `Folder` စာရင်းတစ်ခုလည်း ပါဝင်သည်။ `MRUListEx` က နံပါတ်တပ်ထားသော value များ၏ အစီအစဉ်ကို သတ်မှတ်သည်။

RecentDocs သည် နောက်ပိုင်းတွင် ဖျက်ခဲ့သော သို့မဟုတ် ရွှေ့ခဲ့သော ဖိုင်များ အပါအဝင် အသုံးပြုသူ ထိတွေ့ဆောင်ရွက်ခဲ့သော
စာရွက်စာတမ်းများကို ခွဲခြားသတ်မှတ်ရန် အသုံးဝင်သည်။ Value များတွင် Windows က အသုံးပြုသူ၏ Recent ဖိုင်တွဲတွင် အသုံးပြုခဲ့သော
ဖြတ်လမ်းအမည်လည်း ပါဝင်နိုင်ပြီး ၎င်းကို LNK အထောက်အထားနှင့် ချိတ်ဆက်စစ်ဆေးနိုင်သည်။

<span class="evidence-strength">အကောင်းဆုံး ကောက်ချက်:</span> Explorer သည် ထိုအသုံးပြုသူ၏ RecentDocs စာရင်းတွင် item ကို
မှတ်တမ်းတင်ခဲ့သည်။ Key အချိန်သည် ထိုသီးခြားစာရင်းရှိ နောက်ဆုံး item ကိုသာ ရက်စွဲသတ်မှတ်ပေးသည်။

## တည်နေရာ { #location }

```text
C:\Users\<user>\NTUSER.DAT
Software\Microsoft\Windows\CurrentVersion\Explorer\RecentDocs
```

သက်ဆိုင်သော profile တိုင်းအတွက် `NTUSER.DAT` နှင့် ၎င်း၏ transaction log များကို စုဆောင်းပြီး အသုံးပြုသူ၏ Recent
LNK directory ကိုလည်း စုဆောင်းပါ။

## စိစစ်သူများ သတိပြုရန် { #analyst-cautions }

- `MRUListEx` သည် အစီအစဉ်ကို ပေးသော်လည်း item တိုင်းအတွက် timestamp မပေးပါ။
- Key ၏ last-write အချိန်သည် နောက်ဆုံး အပ်ဒိတ်နှင့် ကိုက်ညီသည်။ MRU နေရာ 0 တစ်ခုတည်းသာ ၎င်းမှ ဆင်းသက်လာသော
  ခိုင်လုံသည့် `LastOpenedUTC` ကို ရရှိသည်။
- Item တစ်ခုသည် root စာရင်းနှင့် extension စာရင်း နှစ်ခုစလုံးတွင် ပေါ်နိုင်သည်။ ဖွင့်ခြင်း နှစ်ကြိမ်အဖြစ် နှစ်ခါ မရေတွက်ပါနှင့်။
- အပလီကေးရှင်းနှင့် Explorer ၏ အပြုအမူ ကွဲပြားနိုင်သည်။ အချို့ ဝင်ရောက်အသုံးပြုနည်းများက RecentDocs ကို အပ်ဒိတ်မလုပ်နိုင်ပါ။
- ရှင်းလင်းရေး/privacy tool များနှင့် စာရင်းအရွယ်အစား ကန့်သတ်ချက်များက ပိုဟောင်းသော အထောက်အထားများကို ဖယ်ရှားသည်။
- ဖိုင်အမည်ကို တိုက်ခိုက်သူက ထိန်းချုပ်နိုင်ပြီး အကြောင်းအရာ အမျိုးအစား၏ သက်သေ မဟုတ်ပါ။
- အသုံးပြုသူကို profile path များ သို့မဟုတ် shell-folder ဒေတာမှ ခန့်မှန်းသည်။ ဗလာ/မရှင်းလင်းသော အသုံးပြုသူအမည်များကို ပြန်လည်သုံးသပ်ပါ။

## Juntei ဖြင့် ခွဲခြမ်းစိတ်ဖြာခြင်း { #parse-with-juntei }

```text
Usage: juntei parse-registry recent-docs <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find NTUSER.DAT hives
  -f, --file <FILE>      Parse one NTUSER.DAT
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry recent-docs -f NTUSER.DAT -o recentdocs.csv
juntei parse-registry recent-docs -d evidence/ -t jsonl \
  | jq 'select(.Extension == ".docx" or .Extension == ".pdf")'
```

## ထုတ်ယူမှု field များ { #output-fields }

| Field | အဓိပ္ပာယ်ဖော်ပုံ |
|---|---|
| `Host` | ရရှိနိုင်ပါက စုဆောင်းမှု/စနစ် အခြေအနေမှ ဆင်းသက်လာသော host။ |
| `User` | Path သို့မဟုတ် shell-folder path များမှ ခန့်မှန်းထားသော profile ပိုင်ရှင်။ ဗလာဖြစ်ပါက မသေချာမှုကို ထိန်းသိမ်းပါ။ |
| `SourcePath` | Input `NTUSER.DAT`။ အသုံးပြုသူများကို ပေါင်းစပ်သည့်အခါ မရှိမဖြစ် လိုအပ်သည်။ |
| `Extension` | Sublist (`.docx`, `Folder` စသည်)။ Root RecentDocs စာရင်းအတွက် null။ |
| `MRU-Position` | သုညသည် ဤစာရင်းအတွင်း နောက်ဆုံးဖြစ်သည်။ ၎င်းသည် နှိုင်းရ အစီအစဉ်ဖြစ်ပြီး ကြာချိန် မဟုတ်ပါ။ |
| `ValueIndex` | `MRUListEx` က ရည်ညွှန်းသော နံပါတ်တပ်ထားသည့် registry value။ ဖွဲ့စည်းပုံဆိုင်ရာ pivot သာ။ |
| `Filename` | Decode လုပ်ထားသော item အမည်။ Path အပြည့်အစုံကို LNK၊ Jump List၊ MFT သို့မဟုတ် အပလီကေးရှင်း ဒေတာမှတစ်ဆင့် ချိတ်ဆက်စစ်ဆေးပါ။ |
| `LinkName` | Shell item တွင် encode လုပ်ထားသော ဖြတ်လမ်းအမည်။ အသုံးပြုသူ၏ Recent directory တွင် မကြာခဏ တွေ့ရသည်။ |
| `KeyLastWrittenUTC` | စာရင်း key နောက်ဆုံးပြောင်းလဲခဲ့သည့်အချိန်။ စာရင်းနှင့် သက်ဆိုင်ပြီး row တိုင်းနှင့် မသက်ဆိုင်ပါ။ |
| `LastOpenedUTC` | Key အချိန်က ထိုအဓိပ္ပာယ်ကို ထောက်ခံသည့် MRU နေရာ 0 အတွက်သာ ဖြည့်ထားသည်။ |
| `ValueDataRaw` | Decode လုပ်ခြင်း မအောင်မြင်ပါက သို့မဟုတ် အထောက်အထားကို မှန်ကန်မှု စစ်ဆေးရန် လိုအပ်ပါက raw byte များ။ |
| `TransactionLogStatus`, `LogsApplied` | Registry log များကို replay လုပ်ခဲ့ခြင်း ရှိ/မရှိနှင့် မည်သည့် log များကို။ |
| `Problems` | ပျောက်နေသော MRU value များ၊ စာရင်းတွင်မပါသော value များ၊ မှားယွင်းသော အမျိုးအစားများ သို့မဟုတ် decode ပျက်စီးမှု။ |

## စုံစမ်းစစ်ဆေးမှု ဗျူဟာ { #investigation-strategy }

1. အသုံးပြုသူတစ်ဦးချင်းစီ၏ root နှင့် extension စာရင်းများကို ခွဲခြားပါ။ တူညီသော item ကို ဂရုတစိုက် dedupe လုပ်ပါ။
2. အစီအစဉ်အတွက် MRU နေရာကို အသုံးပြုပြီး timestamp ပါသော အဆိုအတွက် `LastOpenedUTC` ကိုသာ အသုံးပြုပါ။
3. `LinkName`/`Filename` ကို Recent LNK၊ Jump List၊ `$MFT` နှင့် USN ဒေတာတို့နှင့် တိုက်ဆိုင်စစ်ဆေးပါ။
4. လှည့်စားသော အမည်များ၊ နှစ်ထပ် extension များ၊ network/removable ပစ်မှတ်များနှင့် ကနဦးဝင်ရောက်မှုနှင့် နီးကပ်သော စာရွက်စာတမ်းများကို ပြန်လည်သုံးသပ်ပါ။
5. လုပ်ဆောင်ခဲ့ခြင်းက အရေးပါသည့်အခါ အပလီကေးရှင်း Prefetch/SRUM နှင့် event log များနှင့် ချိတ်ဆက်စစ်ဆေးပါ။

## နောက်ထပ် အသေးစိတ်များ { #further-details }

- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
- [libyal libfwsi](https://github.com/libyal/libfwsi)
- [SANS Windows Forensic Analysis poster](https://www.sans.org/posters/windows-forensic-analysis/)
