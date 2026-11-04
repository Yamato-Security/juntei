# ဘုံထုတ်ယူမှု နည်းလမ်းများ { #common-output-patterns }

## အထောက်အထား တစ်ခု သို့မဟုတ် စုဆောင်းမှုတစ်ခု { #one-artifact-or-a-collection }

```bash
juntei parse-file prefetch -f 'Prefetch/POWERSHELL.EXE-12345678.pf'
juntei parse-file prefetch -d evidence/
```

`-d` သည် ထိုညွှန်ကြားချက်နှင့် သင့်လျော်သော ဖိုင်များကို recursive ရှာဖွေသည်။ Source-path column များသည် ပေါင်းစပ်ထားသော
output တွင် host များနှင့် အထောက်အထားများကို ခွဲခြားပေးသည်။

## CSV သိမ်းဆည်းခြင်း { #save-csv }

```bash
juntei parse-file prefetch -d evidence/ -o prefetch.csv
```

Parser အချို့သည် CSV ဇယားများစွာ ထုတ်ပေးသောကြောင့် directory တစ်ခု လိုအပ်သည်-

```bash
juntei parse-registry amcache -f Amcache.hve -o amcache-tables
juntei parse-ese srum -f SRUDB.dat -o srum-tables
juntei parse-file jump-lists -d evidence/ -o jump-list-tables
```

## JSONL ကို stream လုပ်ခြင်း { #stream-jsonl }

```bash
juntei parse-registry services -d evidence/ -t jsonl \
  | jq -c 'select(.ImagePath? | test("temp|public"; "i"))'
```

မက်ဆေ့ချ်များနှင့် တိုးတက်မှုအခြေအနေသည် standard error သို့ သွားသည်။ Standard output သည် ခွဲခြမ်းစိတ်ဖြာ၍ ရနိုင်သည့်အတိုင်း
ရှိနေသည်။

## ရည်ရွယ်ချက်ရှိရှိ ထပ်ရေးခြင်း { #overwrite-intentionally }

Juntei သည် ပုံသေအားဖြင့် ရှိပြီးသား output ကို ထပ်ရေးရန် ငြင်းပယ်သည်။ ပစ်မှတ်ကို အတည်ပြုပြီး အရေးပါသော ယခင်ရလဒ်ကို
ထိန်းသိမ်းထားပြီးနောက် `-C` / `--clobber` ကို ထည့်ပါ-

```bash
juntei parse-file prefetch -d evidence/ -o prefetch.csv --clobber
```

## Quiet နှင့် အရောင်မပါသော mode များ { #quiet-and-no-color-modes }

Automation အတွက် `-q` ကို အသုံးပြုပြီး terminal အရောင် code များ မသင့်လျော်သည့်အခါ `-K` ကို အသုံးပြုပါ-

```bash
juntei parse-ntfs mft -f '$MFT' -o mft.csv -q -K
```

## စာလုံးအကြီးအသေးနှင့် path များ { #case-and-paths }

Space၊ `$`၊ shell metacharacter များ သို့မဟုတ် backslash များ ပါဝင်သော Windows path များကို သင်၏ shell လိုအပ်သည့်အတိုင်း
quote လုပ်ပါ။ Bash ကဲ့သို့သော shell များတွင် single quote များက `$MFT` နှင့် `$J` ကို variable များအဖြစ် မပြောင်းလဲစေရန်
ကာကွယ်ပေးသည်-

```bash
juntei parse-ntfs mft -f '/evidence/C/$MFT' -o mft.csv
```
