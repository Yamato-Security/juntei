# ပထမဆုံး စုံစမ်းစစ်ဆေးမှု { #first-investigation }

ဤလုပ်ငန်းစဉ်သည် သင် `evidence/` တွင် offline triage စုဆောင်းမှုတစ်ခုကို လက်ခံရရှိထားသည်ဟု ယူဆသည်။

## 1. ရင်းမြစ်ကို ထိန်းသိမ်းပါ { #1-preserve-the-source }

အတည်ပြုထားသော မိတ္တူမှ အလုပ်လုပ်ပါ။ Hash များ၊ စုဆောင်းချိန်၊ time zone၊ host၊ volume နှင့် စုဆောင်းရေး tool ကို
မှတ်တမ်းတင်ပါ။ Juntei သည် အထောက်အထားကို ဖတ်ရုံသာ ဖြစ်သော်လည်း resident-stream သို့မဟုတ် quarantine ထုတ်ယူခြင်းကဲ့သို့သော
option များသည် output ဖိုင်အသစ်များကို ဖန်တီးသည်။ ၎င်းတို့ကို သီးခြား ခွဲခြမ်းစိတ်ဖြာရေး directory သို့ ညွှန်ပါ။

## 2. သင့်တွင် ရှိသည့်အရာကို စာရင်းကောက်ပါ { #2-inventory-what-you-have }

[အထောက်အထား စုဆောင်းခြင်း](collection.md) ရှိ path များကို ရှာဖွေပါ။ အထောက်အထား ပျောက်နေခြင်းသည် ပုံမှန်ဖြစ်သည်။
Registry hive ပါဝင်သည်ဆိုရုံဖြင့် စုဆောင်းမှုတွင် transaction log များ ပါဝင်သည်ဟု မယူဆပါနှင့်။

## 3. အနှစ်ချုပ်များကို ဦးစွာ run ပါ { #3-run-summaries-first }

ညွှန်ကြားချက် အများအပြားသည် output ဖိုင် မတောင်းဆိုသည့်အခါ တိုတောင်းသော အနှစ်ချုပ်ကို ပုံနှိပ်ပြသည်-

```bash
juntei parse-file prefetch -d evidence/
juntei parse-registry shimcache -d evidence/
juntei parse-file defender-quarantine -d evidence/
```

မှတ်တမ်းများကို မယုံကြည်မီ ပျက်စီးမှု၊ dirty-database နှင့် အသုံးမချရသေးသော log သတိပေးချက်များကို ဖတ်ပါ။

ကွန်ပျူတာများစွာပါဝင်သော စုဆောင်းမှုရှိ ပံ့ပိုးထားသော အထောက်အထားတိုင်းကို လုပ်ဆောင်ချက်တစ်ခုတည်းဖြင့် ခွဲခြမ်းစိတ်ဖြာရန်
အောက်ပါကို အသုံးပြုပါ-

```bash
juntei auto-parse -d evidence/ -o analysis/
```

Host ရှာဖွေခြင်း၊ parallelism၊ output layout နှင့် SIEM သို့ ထည့်သွင်းရန် ပေါင်းစည်းထားသော JSONL အတွက်
[စုဆောင်းထားသော ဒေတာကို အလိုအလျောက် ခွဲခြမ်းခြင်း](../commands/auto-parse.md) ကို ကြည့်ပါ။

## 4. ဖွဲ့စည်းပုံရှိသော output ကို သိမ်းဆည်းပါ { #4-save-structured-output }

```bash
mkdir -p analysis
juntei parse-file prefetch -d evidence/ -T -o analysis/prefetch.csv
juntei parse-ntfs mft -d evidence/ -o analysis/mft
juntei parse-ntfs usn-journal -d evidence/ -o analysis/usn
juntei parse-registry services -d evidence/ -o analysis/services.jsonl -t jsonl
```

Scripting နှင့် nested တန်ဖိုးများအတွက် JSONL ကိုလည်းကောင်း၊ spreadsheet သို့မဟုတ် timeline viewer အတွက် CSV ကိုလည်းကောင်း
ဦးစားပေးပါ။

## 5. ယုံကြည်ရသော identifier တစ်ခုမှ pivot လုပ်ပါ { #5-pivot-from-a-reliable-identifier }

ဖိုင်အမည်၊ path အပြည့်အစုံ၊ SHA-1/SHA-256၊ အသုံးပြုသူ SID၊ MFT entry/sequence၊ task GUID၊ ဝန်ဆောင်မှုအမည် သို့မဟုတ်
timestamp ကာလတစ်ခုဖြင့် စတင်ပါ။ Windows က စာလုံးအကြီးအသေး မခွဲခြားဘဲ ကိုင်တွယ်သော field များအတွက်သာ case ကို normalize
လုပ်ပါ။

```bash
juntei parse-file prefetch -d evidence/ -t jsonl \
  | jq 'select(.ExecutableName? | ascii_downcase == "suspicious.exe")'
```

## 6. ချိတ်ဆက်စစ်ဆေးပါ { #6-correlate }

[အထောက်အထားများ ချိတ်ဆက်စစ်ဆေးခြင်း](../resources/workflows.md) ကို အသုံးပြုပါ။ လက်တွေ့ကျသော အစီအစဉ်မှာ-

1. Execution နှင့် မကြာသေးမီက run ချိန်များအတွက် Prefetch။
2. ဖန်တီးမှု၊ path များ၊ အမည်ပြောင်းမှုများနှင့် ဖျက်ပစ်မှုအတွက် `$MFT` နှင့် USN Journal။
3. သမိုင်းဆိုင်ရာ တည်ရှိမှုနှင့် metadata အတွက် Amcache နှင့် ShimCache။
4. အသုံးပြုသူ/အပလီကေးရှင်း အခြေအနေအတွက် LNK၊ Jump Lists၊ RecentDocs နှင့် SRUM။
5. Persistence အတွက် registry ဝန်ဆောင်မှုများနှင့် TaskCache။
6. ရှာဖွေတွေ့ရှိမှု၊ ဖယ်ရှားမှု၊ မူလရင်းမြစ်နှင့် hash များအတွက် Defender quarantine။

## 7. ပြန်လည်ထုတ်လုပ်နိုင်မှုကို မှတ်တမ်းတင်ပါ { #7-document-reproducibility }

Source path၊ source hash၊ Juntei ဗားရှင်း၊ ညွှန်ကြားချက်အတိအကျ၊ output hash၊ စနစ် time zone နှင့် နောက်ပိုင်း စစ်ထုတ်မှု
မှန်သမျှကို မှတ်တမ်းတင်ပါ။ ထုတ်ပေးထားသော အထောက်အထား output ကို တိတ်တဆိတ် ဘယ်သောအခါမျှ မတည်းဖြတ်ပါနှင့်။
