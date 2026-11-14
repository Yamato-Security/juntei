# RDP ဘစ်မက် ကက်ရှ် { #rdp-bitmap-cache }

## RDP ဘစ်မက် ကက်ရှ်က မှတ်တမ်းတင်ထားသည့်အရာ { #what-the-rdp-bitmap-cache-records }

Remote Desktop client (`mstsc.exe`) သည် persistent bitmap caching ကို ဖွင့်ထားလျက် (၎င်းသည် မူလ setting ဖြစ်သည်)
ချိတ်ဆက်သောအခါ နောက်ပိုင်း session များတွင် ထပ်မံ လက်ခံရန် မလိုစေရန် အဝေးရှိ မျက်နှာပြင်၏ အပိုင်းအစများကို disk
ပေါ်တွင် သိမ်းဆည်းသည်။ အပိုင်းအစတစ်ခုစီသည် အများဆုံး 64 × 64 pixel ရှိသော tile တစ်ခု ဖြစ်သည်။ tile များကို
စုပေါင်းလိုက်လျှင် အသုံးပြုသူက အဝေးရှိ ကွန်ပျူတာများပေါ်တွင် မြင်ခဲ့ရသည့်အရာများ၏ အပိုင်းအစငယ်များ ဖြစ်သည်- window
များ၊ စာသား၊ ဖိုင်အမည်များ၊ command များနှင့် desktop များ။

| Client | Cache ဖိုင်များ | Tile များ |
|---|---|---|
| Windows 7 နှင့် နောက်ပိုင်း | `Cache0000.bin`, `Cache0001.bin`, …၊ တစ်ခုလျှင် 100 MB အထိ | 32-bit၊ တစ်ခုပြီးတစ်ခု ဆက်တိုက် |
| Windows XP၊ Server 2003 နှင့် ယခင် | `bcache2.bmc` (pixel တစ်ခုလျှင် 8 bit)၊ `bcache22.bmc` (16)၊ `bcache23.bmc` (24) နှင့် `bcache24.bmc` (32)၊ တစ်ခုလျှင် 10 MB အထိ | 64 × 64 slot တစ်ခုလျှင် တစ်ခု၊ အချို့ကို compress လုပ်ထားသည် |

Windows 10 နှင့် နောက်ပိုင်းသည် `.bin` ဖိုင်များ၏ ဘေးတွင် ဗလာ `bcache24.bmc` တစ်ခုကိုလည်း ချန်ထားသည်။

<span class="evidence-strength">အကောင်းဆုံး ကောက်ချက်:</span> cache ပါရှိသော profile ကို ပိုင်ဆိုင်သည့် အသုံးပြုသူသည်
Remote Desktop client ဖြင့် cache ထဲရှိ အကြောင်းအရာကို ပြသခဲ့သော ကွန်ပျူတာတစ်လုံးသို့ ချိတ်ဆက်ခဲ့သည်။ ထိုကွန်ပျူတာသည်
မည်သည့်ကွန်ပျူတာ ဖြစ်သည်ကိုလည်းကောင်း၊ မည်သည့်အချိန်က ဖြစ်သည်ကိုလည်းကောင်း cache က မဖော်ပြပါ။

## တည်နေရာများနှင့် စုဆောင်းခြင်း { #locations-and-collection }

```text
%LOCALAPPDATA%\Microsoft\Terminal Server Client\Cache\Cache????.bin    Windows 7 and later
%LOCALAPPDATA%\Microsoft\Terminal Server Client\Cache\bcache24.bmc     (empty on Windows 10 and later)
%USERPROFILE%\Local Settings\Application Data\Microsoft\Terminal Server Client\Cache\bcache*.bmc    Windows XP and earlier
```

`HKCU\Software\Microsoft\Terminal Server Client` ၏ `BitmapPersistCacheLocation` value သည် cache ကို အခြားနေရာသို့
ရွှေ့ထားနိုင်သည်။ profile တိုင်း၏ `Cache` ဖိုင်တွဲတစ်ခုလုံးကို volume shadow copy များမှပါ စုဆောင်းပါ- cache ဖိုင်၏
ယခင် မိတ္တူတစ်ခုတွင် လက်ရှိဖိုင်၌ မရှိတော့သော tile များ ပါနေနိုင်သည်။

## စိစစ်သူများ သတိပြုရန် { #analyst-cautions }

- Tile များတွင် အချိန် မပါဝင်သလို ၎င်းတို့ လာရာ ကွန်ပျူတာများ၏ အမည်ကိုလည်း cache က မဖော်ပြပါ။ ၎င်းတို့ကို အခြား
  အထောက်အထားများမှ ရရှိသော RDP ချိတ်ဆက်မှုများနှင့် ဆက်စပ်ပါ- client ၏ `Terminal Server Client` registry key များ
  (`parse-registry remote-access`)၊ `Default.rdp` နှင့် အခြား `.rdp` ဖိုင်များ၊ `mstsc.exe` ၏ Jump Lists၊ `$MFT`
  ရှိ cache ဖိုင်များ၏ အချိန်များနှင့် အဝေးရှိ ကွန်ပျူတာများ၏ logon event များ။
- Cache ဖိုင်တစ်ခုတွင် session များစွာနှင့် အဝေးရှိ ကွန်ပျူတာ အများအပြားမှ tile များ ရောနှောနေသည်။ ဖိုင်ပြည့်လာသည့်အခါ
  client က tile အဟောင်းများကို အစားထိုးသောကြောင့် tile တစ်ခုသည် ရက်ပေါင်းများစွာ သို့မဟုတ် လပေါင်းများစွာ
  ဟောင်းနေနိုင်သည်။
- Tile များသည် မျက်နှာပြင်တစ်ခုလုံး မဟုတ်ဘဲ မျက်နှာပြင်၏ အပိုင်းအစများ ဖြစ်သည်။ မျက်နှာပြင်ပေါ်တွင် ကပ်လျက်ရှိခဲ့သော
  tile များသည် ဖိုင်ထဲတွင်လည်း ကပ်လျက် ရှိလေ့ရှိသော်လည်း အမြဲတမ်း မဟုတ်ပါ။ မျက်နှာပြင်များကို
  [RdpCacheStitcher](https://github.com/BSI-Bund/RdpCacheStitcher) ဖြင့် သို့မဟုတ် collage များကို မျက်စိဖြင့် ကြည့်၍
  ပြန်လည်တည်ဆောက်ပါ။
- Tile အများစုသည် 64 × 64 pixel ဖြစ်သည်။ မျက်နှာပြင်၏ ညာဘက်နှင့် အောက်ခြေ အစွန်းရှိ tile များသည် resolution ပေါ်မူတည်၍
  ပိုကျဉ်းခြင်း သို့မဟုတ် ပိုနိမ့်ခြင်း ရှိသည်- ဥပမာ အကျယ် 58 သို့မဟုတ် 62 pixel၊ သို့မဟုတ် အမြင့် 28 သို့မဟုတ် 35 pixel။
  Juntei သည် tile တစ်ခုစီကို ၎င်း၏ ကိုယ်ပိုင် အရွယ်အစားဖြင့် ဖတ်သည်။ tile တိုင်းကို 64 pixel အကျယ်ဟု ယူဆ၍ ဖတ်သော
  tool များသည် ကျဉ်းသော tile များကို စောင်းနေသကဲ့သို့ ပြသသည်။
- အရောင်တစ်မျိုးတည်းသာ ပါသော tile (`SingleColor`) အများစုသည် နောက်ခံ ဖြစ်သည်။ အကြောင်းအရာကို ရှာရန် ၎င်းတို့ကို
  filter ဖြင့် ဖယ်ထုတ်ပါ။
- `.bmc` ဖိုင်များတွင် 8-bit tile များ၏ အရောင်များသည် session ၏ palette မှ လာပြီး ထို palette ကို cache က မသိမ်းဆည်းပါ။
  Juntei သည် Windows ၏ မူလ palette ကို အသုံးပြုသောကြောင့် ၎င်းတို့၏ အရောင်များ မှားနိုင်သည်။ Compress လုပ်ထားသော
  tile များသည် Remote Desktop protocol ၏ interleaved RLE (MS-RDPBCGR 3.1.9) ကို အသုံးပြုပြီး Juntei က ၎င်းကို
  specification သတ်မှတ်ထားသည့်အတိုင်း decompress လုပ်သည်။
- ၎င်း၏ 64 × 64 slot ထက် သေးငယ်သော `.bmc` tile သည် ၎င်း၏နောက်တွင် ယခင် tile တစ်ခု၏ အဆုံးပိုင်းကို ချန်ထားခဲ့သည်။
  Juntei သည် ထိုကျန်ရှိနေသော row များကို သီးခြား ပုံတစ်ပုံ (`RemnantImage`) အဖြစ် သိမ်းဆည်းသည်။ ၎င်းတို့သည် slot
  ထဲရှိ tile ၏ အစိတ်အပိုင်း မဟုတ်ဘဲ ပိုဟောင်းသော tile တစ်ခုမှ ဖြစ်သည်။

## Juntei ဖြင့် ခွဲခြမ်းစိတ်ဖြာခြင်း { #parse-with-juntei }

```text
Usage: juntei parse-file rdp-cache <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find Cache????.bin and bcache*.bmc files
  -f, --file <FILE>      Parse one cache file, whatever its name

Output:
  -o, --output <FILE>       Save one record per tile
  -t, --output-type <TYPE>  csv, json, or jsonl
  -x, --extract <DIR>       Save each tile as a BMP and a PNG collage of each cache file
      --collage-only        With --extract, save only the collages
      --collage-width <N>   Tiles per collage row (default 64)
```

```bash
juntei parse-file rdp-cache -d evidence/ -o rdp-cache.csv --extract rdp-cache/
juntei parse-file rdp-cache -f Cache0000.bin --extract rdp-cache/ --collage-only
juntei parse-file rdp-cache -d evidence/ -t jsonl | jq 'select(.SingleColor == false)'
```

`-d` ဖြင့်ဆိုလျှင် Juntei သည် `Cache` နောက်တွင် ဂဏန်းများ လိုက်ပြီး `.bin` ဖြင့် ဆုံးသော အမည်ရှိ ဖိုင်များနှင့် `bcache`
နောက်တွင် ဂဏန်းများ လိုက်ပြီး `.bmc` ဖြင့် ဆုံးသော အမည်ရှိ ဖိုင်များကို ဖတ်သည်။ `RDP8bmp` signature ဖြင့် မစတင်သော
`Cache????.bin` ဖိုင်သည် အခြား program တစ်ခု၏ ဖိုင် ဖြစ်သည်- ၎င်းကို summary တွင် ရေတွက်ပြီး ကျော်သွားသည်။ `-f`
ဖြင့်ဆိုလျှင် ဖိုင်အမည် မည်သို့ပင် ဖြစ်စေ ထိုဖိုင်ကို ဖတ်သည်။

### ပုံများ { #images }

`--extract` သည် cache ဖိုင်တစ်ခုစီအတွက် ဖိုင်တွဲတစ်ခု `<DIR>/<user>/<cache file>/` ကို ဖန်တီးသည်။ cache ဖိုင်နှစ်ခုသည်
ဖိုင်တွဲတစ်ခုတည်းကို မျှဝေရမည့်အခါ (ဥပမာ cache ဖိုင်တစ်ခုနှင့် volume shadow copy မှ ၎င်း၏ မိတ္တူ) ဒုတိယ ဖိုင်တွဲကို
နံပါတ်တပ်သည် (`Cache0001.bin_2`)။ ဖိုင်တွဲတွင် အောက်ပါတို့ ပါဝင်သည်-

- `<cache file>_<index>.bmp`: tile တစ်ခုစီ၊ ၎င်း၏ ကိုယ်ပိုင် အရွယ်အစားဖြင့်။ အမည်များသည် bmc-tools အသုံးပြုသော
  အမည်များ ဖြစ်သောကြောင့် RdpCacheStitcher က ထိုဖိုင်တွဲကို ဖွင့်နိုင်သည်။
- `<cache file>_old_<index>.bmp`: `.bmc` အတွက်သာ၊ ပိုသေးငယ်သော tile တစ်ခု၏ နောက်တွင် ကျန်ရစ်ခဲ့သော ယခင် tile ၏
  အဆုံးပိုင်း။
- `<cache file>_collage.png`: tile အားလုံးကို ဖိုင်ထဲရှိ အစီအစဉ်အတိုင်း row တစ်ခုလျှင် 64 ခု (`--collage-width`)
  စီထားပြီး တစ်ခုစီကို 64 × 64 cell တစ်ခုထဲတွင် ဘယ်ဘက်အပေါ်ထောင့်မှ စ၍ ထားသည်။ 0 မှ စ၍ ရေတွက်လျှင် index *i* ရှိသော
  tile သည် row *i* / 64 နှင့် column *i* % 64 တွင် ရှိသည်။ cell များ၏ ဗလာ အပိုင်းများကို magenta ဖြင့် ဖြည့်ထားသည်-
  ကျဉ်းသော tile ၏ ဘေးတွင်၊ နိမ့်သော tile ၏ အောက်တွင်နှင့် tile ကို ဖတ်မရခဲ့သည့်နေရာတွင်။

`--collage-only` သည် collage များကိုသာ သိမ်းဆည်းသည်။ `-C` မပါလျှင် ရှိပြီးသား ပုံကို အစားမထိုးဘဲ ၎င်း၏ tile ၏
`Problems` တွင် ထိုအကြောင်းကို ဖော်ပြသည်။ cache ဖိုင်များကိုယ်တိုင်ကိုမူ `-C` ပါသော်လည်း ဘယ်တော့မှ အစားမထိုးပါ။

## CSV ထုတ်ယူမှု field များ { #csv-output-fields }

| Field | အဓိပ္ပာယ် |
|---|---|
| `SourceFile` | Cache ဖိုင်။ |
| `User` | cache ဖိုင် ရှိနေသော profile ဖိုင်တွဲ၏ အသုံးပြုသူ၊ ဖိုင်၏ path မှ ရယူသည်။ |
| `TileIndex` | ဖိုင်ထဲရှိ tile ၏ နံပါတ်၊ 0 မှ စ၍ ရေတွက်ပြီး ၎င်း၏ ပုံအမည်ထဲရှိ နံပါတ်နှင့် တူသည်။ |
| `Width`, `Height` | tile ၏ အရွယ်အစား pixel ဖြင့်။ |
| `SingleColor` | tile ၏ pixel တိုင်းတွင် အရောင်တူလျှင် `true`။ |
| `TileImage` | `--extract` ဖြင့်: tile ၏ BMP ဖိုင်။ |
| `Collage` | `--extract` ဖြင့်: cache ဖိုင်၏ collage။ |
| `RemnantImage` | `--extract` ပါသော `.bmc`: slot ထဲတွင် ကျန်ရစ်ခဲ့သော ယခင် tile ၏ အဆုံးပိုင်း။ |
| `Key` | Remote Desktop protocol ရှိ tile ၏ 64-bit key၊ hexadecimal ဖြင့်။ |
| `Format`, `FormatVersion` | `bin` နှင့် ၎င်း၏ signature နောက်ရှိ version (Windows 10 နှင့် နောက်ပိုင်းတွင် 6)၊ သို့မဟုတ် `bmc`။ |
| `BitsPerPixel` | `.bin` အတွက် 32၊ `.bmc` အတွက် 8၊ 16၊ 24 သို့မဟုတ် 32။ |
| `Compressed`, `Flags` | `.bmc`: tile ကို compress လုပ်ထားခြင်း ရှိ/မရှိနှင့် ၎င်း၏ header ၏ flag များ။ |
| `Offset` | ဖိုင်ထဲတွင် tile ၏ header စတင်သည့်နေရာ။ |
| `Problems` | ပျက်စီးမှု: tile တစ်ခု၏ အလယ်တွင် ဆုံးသွားသော ဖိုင်၊ မည်သည့် tile တွင်မျှ မရှိနိုင်သော tile header၊ tile ၏ အရွယ်အစားနှင့် မကိုက်ညီသော ဒေတာ၊ decompress လုပ်၍ မရသော compress လုပ်ထားသည့် ဒေတာ၊ သိမ်းဆည်း၍ မရခဲ့သော ပုံ။ |

JSON နှင့် JSONL တွင် တူညီသော field များ ပါရှိပြီး ဂဏန်းများနှင့် boolean များကို ၎င်းတို့၏ type အတိုင်း ဖော်ပြကာ
tile တွင် မရှိသော value များအတွက် `null` ကို သုံးပြီး `Problems` ကို list အဖြစ် ထားသည်။ နောက်ဆုံး tile နောက်ရှိ ဒေတာ
ကဲ့သို့ မည်သည့် tile တစ်ခုတည်းနှင့်မျှ မသက်ဆိုင်သော ပျက်စီးမှုကို run ၏ message များတွင် ဖော်ပြသည်။

## အလိုအလျောက် ခွဲခြမ်းစိတ်ဖြာခြင်း { #automatic-parsing }

`auto-parse` သည် `LateralMovement/RdpCache.csv` (`-t json` သို့မဟုတ် `-t jsonl` ဖြင့်ဆိုလျှင် `RdpCache.json`
သို့မဟုတ် `RdpCache.jsonl`) ကို `LateralMovement/RdpCache-logs.txt` နှင့်အတူ ရေးသားပြီး cache ဖိုင်တစ်ခုစီ၏ collage
ကို `LateralMovement/RdpCache/` ဖိုင်တွဲထဲတွင် သိမ်းဆည်းသည်။ tile တိုင်းကိုပါ သိမ်းဆည်းရန် `--extract-rdp-tiles` ကို
ထည့်ပါ- အသုံးပြုသူ တစ်ဦးတည်းအတွက်ပင် ဖိုင် သောင်းနှင့်ချီ ဖြစ်နိုင်သည်။ Disk image များမှဆိုလျှင် cache ဖိုင်များကို
ကူးယူသည်။ ပေါင်းစပ်ထားသော `-j` timeline တွင် row တိုင်းကို `Artifact` identifier `rdp-cache` ဖြင့် ထည့်သွင်းသည်။
Tile များတွင် အချိန် မပါသောကြောင့် [`super-timeline`](../commands/super-timeline.md) သည် ၎င်းတို့မှ event မပြုလုပ်ပါ။

## စုံစမ်းစစ်ဆေးမှု ဗျူဟာ { #investigation-strategy }

1. Collage များကို ဖွင့်ပြီး စာသားများကို ရှာပါ- ကွန်ပျူတာအမည်နှင့် အသုံးပြုသူအမည်များ၊ ဖိုင် path များ၊ command များ၊
   console window များ၊ browser ၏ address bar များနှင့် dialog box များ။
2. `--extract` ဖြင့် သိမ်းဆည်းထားသော tile ဖိုင်တွဲမှ RdpCacheStitcher ဖြင့် မျက်နှာပြင်များကို ပြန်လည်တည်ဆောက်ပါ။
3. client ၏ registry (`parse-registry remote-access`)၊ `.rdp` ဖိုင်များနှင့် `mstsc.exe` ၏ Jump Lists မှ အသုံးပြုသူ
   ချိတ်ဆက်ခဲ့သော ကွန်ပျူတာများကို ရှာပါ။ ထိုကွန်ပျူတာများပေါ်တွင် ကိုက်ညီသော RDP logon များနှင့် အသုံးပြုသူ
   ဝင်ရောက်မှု မှတ်တမ်း (UAL) record များကို ရှာပါ။
4. Tile များကို မည်သည့်အချိန်တွင် ရေးခဲ့သည်ကို ကျဉ်းမြောင်းစေရန် cache ဖိုင်များ၏ `$MFT` အချိန်များနှင့် volume shadow
   copy များထဲရှိ ၎င်းတို့၏ မိတ္တူများ၏ အချိန်များကို အသုံးပြုပါ။

## နောက်ထပ် အသေးစိတ်များ { #further-details }

- [ANSSI (CERT-FR) — CERTFR-2016-ACT-017: RDP bitmap cache (ပြင်သစ်ဘာသာ)](https://www.cert.ssi.gouv.fr/actualite/CERTFR-2016-ACT-017/)
- [ANSSI — bmc-tools](https://github.com/ANSSI-FR/bmc-tools)
- [BSI — RdpCacheStitcher](https://github.com/BSI-Bund/RdpCacheStitcher)
- [Velociraptor — Windows.Forensics.RDPCache](https://github.com/Velocidex/velociraptor/blob/master/artifacts/definitions/Windows/Forensics/RDPCache.yaml)
- [dissect.target — RDP cache plugin](https://github.com/fox-it/dissect.target/blob/main/dissect/target/plugins/os/windows/rdpcache.py)
- [Microsoft — MS-RDPBCGR 3.1.9: Interleaved RLE-Based Bitmap Compression](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-rdpbcgr/b6a3f5c2-0804-4c10-9d25-a321720fd23e)
- [FreeRDP — persistent bitmap cache](https://github.com/FreeRDP/FreeRDP/blob/master/libfreerdp/cache/persistent.c)
