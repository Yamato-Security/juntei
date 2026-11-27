# Event log များကို carve လုပ်ခြင်း { #carve-evtx }

Log ကို clear လုပ်ပြီးနောက် သို့မဟုတ် ဖျက်ပြီးနောက် ကျန်ရှိနေသော event log chunk များကို disk image၊ unallocated space၊ pagefile သို့မဟုတ် memory image မှ ပြန်လည်ရယူသည်။ 64 KiB EVTX chunk တစ်ခုစီတွင် ကိုယ်ပိုင် string table နှင့် template table ပါရှိသောကြောင့် log ၏ ကျန်အပိုင်းများ မရှိသော်လည်း ၎င်း၏ record များကို ဖတ်နိုင်သည်။ Juntei သည် ပြန်လည်ရယူထားသော chunk များကို channel တစ်ခုလျှင် ဖိုင်တစ်ခုစီဖြင့် သာမန် `.evtx` ဖိုင်များအဖြစ် ရေးပေးသောကြောင့် Hayabusa သို့မဟုတ် Event Viewer ဖြင့် ဖွင့်နိုင်သည်။

```bash
juntei carve-evtx -f host.E01 -o carved/
juntei carve-evtx -f host.vhdx --free-space-only -o carved-free/
juntei carve-evtx -f pagefile.sys --live-logs evidence/C/Windows/System32/winevt/Logs/ -o carved-pagefile/
juntei carve-evtx -f memory.raw -o carved-memory/ -t jsonl
```

`-o` မပါလျှင် အနှစ်ချုပ်ကိုသာ ပြသည်။

## Chunk များကို ရှာဖွေခြင်း { #finding-chunks }

- Chunk signature ကို byte offset တိုင်းတွင် ရှာဖွေပြီး header တစ်ခုစီ၏ တန်ဖိုးများ (header အရွယ်အစား၊ record နံပါတ်များ၊ offset များ) သည် ယုတ္တိရှိရမည်။ Checksum များကို ဖော်ပြသော်လည်း မှန်ကန်ရန် မလိုအပ်ပါ။ တစ်စိတ်တစ်ပိုင်း overwrite ဖြစ်သွားသော chunk တွင်လည်း ဖတ်နိုင်သော record များ ကျန်ရှိနေနိုင်သောကြောင့် ဖြစ်သည်။
- NTFS ဖြင့် compress လုပ်ထားသော log များ၏ chunk များကို decompress လုပ်သည် (LZNT1၊ cluster 16 ခုစီ ယူနစ်)။ Disk image တွင် volume ၏ cluster အရွယ်အစားကို သုံးပြီး အခြားဖိုင်များအတွက် `--cluster-size <BYTES>` (ပုံမှန် 4096) ကို သုံးသည်။
- Record များကို header တွင်ပါသော နောက်ဆုံး record offset အထိ အရွယ်အစားအလိုက် တစ်ခုပြီးတစ်ခု ဖတ်သွားပြီး record ID ထပ်နေရုံဖြင့် မရပ်ပါ။ ပျက်စီးနေသော record တစ်ခုတွေ့လျှင် နောက်ထပ် မှန်ကန်သော record မှ ဆက်ဖတ်သည်။ Chunk ၏ slack ထဲရှိ record ဟောင်းများကိုလည်း စာရင်းပြုစုသည်။

## Disk image များနှင့် NTFS volume များ { #disk-images }

E01 သို့မဟုတ် VHDX image၊ သို့မဟုတ် NTFS ပါသော raw disk သို့မဟုတ် volume တွင် chunk တိုင်းကို ၎င်းတည်ရှိရာ cluster အလိုက် အမျိုးအစားခွဲသည်-

| တည်နေရာ | အဓိပ္ပာယ် |
|---|---|
| `FreeSpace` | မည်သည့်ဖိုင်မှ အသုံးမပြုသော cluster။ |
| `DeletedLog` | ဖျက်ထားသော log ၏ `$MFT` record က ဖော်ပြထားဆဲဖြစ်သော free cluster သို့မဟုတ် ထို cluster များမှ ပြန်တည်ဆောက်ထားသော log။ |
| `LiveLog` | အသုံးပြုနေဆဲ event log ၏ cluster။ `--keep-live` မပါလျှင် ချန်လှပ်ထားသည်။ |
| `AllocatedFile` | `pagefile.sys` သို့မဟုတ် volume shadow copy ၏ store ဖိုင်ကဲ့သို့ အခြားဖိုင်တစ်ခု၏ cluster။ |
| `AllocatedUnknown` | မည်သည့် `$MFT` record ကမှ မဖော်ပြထားသော allocated cluster။ |
| `OutsideVolumes` | Disk ၏ NTFS volume အားလုံး၏ အပြင်ဘက်တွင် ရှိသည်။ |
| `Unclassified` | Input တွင် NTFS volume မရှိပါ (pagefile သို့မဟုတ် memory image)။ |

`$MFT` record က free cluster များကိုသာ ဖော်ပြထားသော ဖျက်ထားသည့် log များကို compress လုပ်ထားသည်များအပါအဝင် တစ်ခုလုံး ပြန်တည်ဆောက်ပြီး `Recovered/` အောက်တွင် သိမ်းသည်။ `--free-space-only` သည် free cluster များကိုသာ ရှာဖွေသည်။ ပိုမြန်သော်လည်း pagefile၊ volume shadow copy နှင့် အခြားဖိုင်များကို ကျော်သွားသည်။

## ထပ်နေသော chunk များ ဖယ်ရှားခြင်း { #deduplication }

Chunk တစ်ခုစီကို တစ်ကြိမ်သာ ရေးသည်။ Chunk တစ်ခုသည် live log (အသုံးပြုနေဆဲ log) တစ်ခုတွင် ပါဝင်နေလျှင်၊ အခြား chunk တစ်ခုနှင့် ထပ်တူဖြစ်နေလျှင် သို့မဟုတ် ၎င်း၏ record အားလုံးသည် live log များ သို့မဟုတ် ရေးပြီးသား chunk တစ်ခုတွင် ရှိပြီးဖြစ်လျှင် ချန်လှပ်ထားသည်။ Record ပိုများသော chunk များကို ဦးစွာ စဉ်းစားသောကြောင့် chunk တစ်ခု၏ ပိုဟောင်းပြီး ပိုတိုသော မိတ္တူသည် ဘာမှ ထပ်မတိုးပါ။ NTFS မပါသော input အတွက် live log များကို `--live-logs <DIR>` ဖြင့် ပေးပါ။

## ရလဒ်များ { #results }

- `Carved/<CHANNEL>.evtx`: channel တစ်ခုလျှင် log တစ်ခု၊ Windows က log များကို အမည်ပေးသည့်အတိုင်း အမည်ပေးသည် (`Microsoft-Windows-Sysmon%4Operational.evtx`)။ Chunk header ရှိ record နံပါတ်များကို နံပါတ်ပြန်တပ်ပြီး checksum များကို ပြန်တွက်သည်။ ပျက်စီးနေသော record နှင့် ၎င်းနောက်ရှိ အရာအားလုံးသည် slack ဖြစ်သွားသည်။ Hayabusa ၏ `-x` သည် အလွတ်ဟု မှတ်သားထားသော chunk များကို ရှာဖွေသည်။ တစ်စိတ်တစ်ပိုင်း အသုံးပြုထားသော chunk များ၏ slack နေရာတွင် ကျန်နေသော record များကို Juntei ၏ `records.csv` တွင် ကြည့်နိုင်သည်။
- `Recovered/Volume<N>/...`: `$MFT` record များမှ ပြန်တည်ဆောက်ထားသော ဖျက်ထားသည့် log များ။
- `chunks.csv` / `chunks.jsonl`: တွေ့ရှိသော chunk တိုင်း၊ ၎င်းရှိခဲ့သည့်နေရာ၊ ၎င်းကို မည်သို့ ကိုင်တွယ်ခဲ့သည် (`Status`)၊ channel၊ record ID များ၊ အချိန်များနှင့် checksum အခြေအနေ။
- `records.csv` / `records.jsonl`: ရေးထားသော chunk များ၏ record တိုင်း။ `OnlyInCarvedData` သည် live log များတွင် မရှိသော record များကို အမှတ်အသားပြုသည်။

```bash
hayabusa csv-timeline -d carved/Carved -a -o carved-timeline.csv
```

Carving သည် ဆက်တိုက်ရှိသော chunk များကိုသာ ရှာတွေ့နိုင်သည်။ ဆက်တိုက်မဟုတ်သော cluster များ သို့မဟုတ် memory ထဲရှိ ဆက်တိုက်မဟုတ်သော physical page များကို ဖြတ်၍ ခွဲနေသော chunk သည် မပြည့်စုံဘဲ ၎င်း၏ နောက်ပိုင်း record များ ပျက်စီးနေခြင်း သို့မဟုတ် ပျောက်နေခြင်း ဖြစ်နိုင်သည်။ `hiberfil.sys` ကို ဦးစွာ decompress လုပ်ပါ။ ရှာဖွေမှု ပြီးဆုံးလျှင် exit code သည် 0 ဖြစ်ပြီး အမှားဖြစ်လျှင် 1 ဖြစ်သည်။

## အသေးစိတ်အချက်အလက် { #further-details }

- [အသုံးပြုပုံနှင့် field အသေးစိတ် (အင်္ဂလိပ်)](https://github.com/Yamato-Security/juntei/blob/main/docs/carve-evtx.md)
- [အသုံးပြုပုံနှင့် field အသေးစိတ် (ဂျပန်)](https://github.com/Yamato-Security/juntei/blob/main/docs/carve-evtx.ja.md)
- [EVTX format (libevtx စာတမ်း)](https://github.com/libyal/libevtx/blob/main/documentation/Windows%20XML%20Event%20Log%20(EVTX).asciidoc)
