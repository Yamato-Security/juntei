# အထောက်အထားများ ချိတ်ဆက်စစ်ဆေးခြင်း { #cross-artifact-workflows }

မည်သည့်အထောက်အထားမျှ အရေးကြီးသော ကောက်ချက်တစ်ခုကို တစ်ခုတည်း မထမ်းဆောင်သင့်ပါ။ ဤလုပ်ငန်းစဉ်များသည် သဲလွန်စတစ်ခုကို
ခိုင်လုံသော အထောက်အထား ကွင်းဆက်အဖြစ် ပြောင်းလဲပုံကို ပြသသည်။

## သံသယဖြစ်ဖွယ် executable လုပ်ဆောင်ခြင်း { #suspicious-executable-execution }

1. **Prefetch-** ထိန်းသိမ်းထားသော run ချိန်၊ path/hash အခြေအနေ၊ run အကြိမ်ရေနှင့် load လုပ်ထားသော ဖိုင်များကို သတ်မှတ်ပါ။
2. **`$MFT`-** path၊ အရွယ်အစား၊ timestamp များ၊ ADS၊ ဖျက်ပစ်ထားသော အခြေအနေနှင့် MFT reference ကို ရှာဖွေပါ။
3. **USN Journal-** ဖန်တီးမှု၊ ရေးသားမှုများ၊ အမည်ပြောင်းမှုနှင့် ဖျက်ပစ်မှုကို ပြန်လည်တည်ဆောက်ပါ။
4. **Amcache-** SHA-1၊ PE metadata၊ ဗားရှင်းနှင့် သမိုင်းဆိုင်ရာ path ကို ထပ်ထည့်ပါ။
5. **ShimCache-** compatibility-cache path/အစီအစဉ်နှင့် ပံ့ပိုးထားသော execution flag ကို ထပ်ထည့်ပါ။
6. **SRUM-** တူညီသော executable/အသုံးပြုသူ၏ resource/network အသုံးပြုမှုကို ရှာဖွေပါ။
7. **Defender-** quarantine အချိန်၊ မူရင်း path၊ hash နှင့် download ရင်းမြစ်ကို စစ်ဆေးပါ။
8. **Event log များ/EDR-** process parent၊ command line၊ အကောင့်၊ network နှင့် လုံခြုံရေး အခြေအနေကို ခွဲခြားသတ်မှတ်ပါ။

ကောက်ချက်သည် ဖိုင်တည်ရှိမှု၊ လုပ်ဆောင်ခဲ့ခြင်း (execution) နှင့် လူတစ်ဦး သို့မဟုတ် ကျူးကျော်ဝင်ရောက်မှုတစ်ခုသို့ တာဝန်သတ်မှတ်ခြင်းကို
ခွဲခြားသင့်သည်။

## Ransomware ဖိုင် လှုပ်ရှားမှု { #ransomware-file-activity }

1. `$MFT` တွင် encrypt လုပ်ထားသော extension များနှင့် ထိခိုက်ခံရသော path များကို ခွဲခြားသတ်မှတ်ပါ။
2. အစုလိုက် ရေးသားမှု/အမည်ပြောင်းမှု/ဖျက်ပစ်မှုများကို ပြန်လည်တည်ဆောက်ရန် USN reason များနှင့် အစီအစဉ်ကို အသုံးပြုပါ။
3. ဖြစ်နိုင်ချေရှိသော encryptor များနှင့် run ချိန်များကို ခွဲခြားသတ်မှတ်ရန် Prefetch ကို အသုံးပြုပါ။
4. ဖြစ်နိုင်ချေရှိသော process ၏ CPU/disk/network မြင့်တက်မှုများအတွက် SRUM ကို အသုံးပြုပါ။
5. ဖြန့်ချိမှု/persistence အတွက် ဝန်ဆောင်မှုများနှင့် TaskCache ကို စစ်ဆေးပါ။
6. ရှာဖွေတွေ့ရှိမှု/ပြုပြင်ဖယ်ရှားမှုအတွက် Defender quarantine နှင့် event log များကို စစ်ဆေးပါ။

MFT/USN သည် execution အထောက်အထား မကျန်ရှိသည့်တိုင် ဖိုင်စနစ်အပေါ် သက်ရောက်မှုကို သက်သေပြနိုင်သည်။ ၎င်းတို့သည် process
telemetry သို့မဟုတ် ချိတ်ဆက်စစ်ဆေးမှု မပါဘဲ တာဝန်ရှိသော process ကို ယေဘုယျအားဖြင့် ခွဲခြားသတ်မှတ်မပေးပါ။

## အန္တရာယ်ရှိသော စာရွက်စာတမ်း / ဖြတ်လမ်းဖြင့် ကနဦး ဝင်ရောက်မှု { #malicious-document-shortcut-initial-access }

1. RecentDocs နှင့် Jump Lists သည် အသုံးပြုသူ/အပလီကေးရှင်း ထိတွေ့ဆောင်ရွက်မှုနှင့် MRU အခြေအနေကို ခွဲခြားသတ်မှတ်ပေးသည်။
2. ပစ်မှတ်၊ volume/share၊ MFT reference နှင့် argument များအတွက် ကိုက်ညီသော Recent-folder LNK ကို ခွဲခြမ်းစိတ်ဖြာပါ။
3. ရင်းမြစ် URL/zone များအတွက် `$MFT` resident ADS သို့မဟုတ် Defender အကြောင်းအရာမှ Zone.Identifier ကို အသုံးပြုပါ။
4. စာရွက်စာတမ်း အပလီကေးရှင်းနှင့် child process များအတွက် Prefetch/event log များကို အသုံးပြုပါ။
5. ထုတ်ယူထားသော payload ၏ ဖန်တီးမှုနှင့် ဖျက်ပစ်မှုကို ခြေရာခံရန် USN/MFT ကို အသုံးပြုပါ။

## ဝန်ဆောင်မှု persistence { #service-persistence }

1. ဝန်ဆောင်မှုအမည်၊ `ImagePath`, `ServiceDll`၊ အကောင့်၊ start type နှင့် failure action များကို decode လုပ်ပါ။
2. Environment variable များနှင့် quote ပါ/မပါ path အပြုအမူကို ဖြေရှင်းပါ။
3. `$MFT` နှင့် USN မှတ်တမ်းတွင် executable/DLL ကို ရှာဖွေပါ။
4. အမှန်တကယ် ဝန်ဆောင်မှု/process စတင်ခြင်းအတွက် Prefetch နှင့် event log များကို စစ်ဆေးပါ။
5. ယခင်ပြင်ဆင်မှုအတွက် လက်ရှိမဟုတ်သော control set များနှင့် ပြန်လည်ရယူထားသော registry မှတ်တမ်းများကို နှိုင်းယှဉ်ပါ။

## Scheduled-task persistence { #scheduled-task-persistence }

1. TaskCache action များ၊ principal/context၊ မှတ်ပုံတင်မှု/runtime metadata နှင့် hidden အခြေအနေကို decode လုပ်ပါ။
2. `C:\Windows\System32\Tasks` XML နှင့် နှိုင်းယှဉ်ပါ။
3. Task အမည်/GUID ကို Security နှင့် Task Scheduler Operational event များနှင့် ချိတ်ဆက်စစ်ဆေးပါ။
4. Executable/script ကို ရှာဖွေပြီး hash လုပ်ပါ။ Prefetch နှင့် ဖိုင်စနစ် မှတ်တမ်းကို စစ်ဆေးပါ။
5. Task ကို ဖယ်ရှားခဲ့ပါက ဖျက်ပစ်ထားသော TaskCache မှတ်တမ်းများကို ပြန်လည်ရယူပါ။

## သံသယရှိသော ဒေတာ ခိုးထုတ်မှု { #suspected-exfiltration }

1. App/အသုံးပြုသူ/profile အလိုက် ပေးပို့ခဲ့သော ပုံမှန်မဟုတ်သည့် byte များနှင့် တိုင်းတာမှုကာလကို ခွဲခြားသတ်မှတ်ရန် SRUM ကို
   အသုံးပြုပါ။
2. Executable ၏ ကိုယ်ပိုင်အမှတ်အသားကို Prefetch၊ Amcache နှင့် `$MFT` ဖြင့် အတည်ပြုပါ။
3. Jump Lists၊ LNK၊ RecentDocs နှင့် event/EDR ဒေတာမှတစ်ဆင့် ဝင်ရောက်ခဲ့/ဖွင့်ခဲ့သော ဖိုင်များကို ခွဲခြားသတ်မှတ်ပါ။
4. ဦးတည်ရာ/protocol အတွက် proxy၊ firewall၊ DNS၊ packet သို့မဟုတ် EDR telemetry ကို အသုံးပြုပါ။

SRUM ပမာဏသည် ဒေတာခိုးထုတ်မှု ယူဆချက်ကို ထောက်ခံသော်လည်း ၎င်းတစ်ခုတည်းဖြင့် လွှဲပြောင်းခဲ့သော အကြောင်းအရာကို
ခွဲခြားသတ်မှတ်ခြင်း သို့မဟုတ် အန္တရာယ်ရှိသော ရည်ရွယ်ချက်ကို သက်သေပြခြင်း မပြုပါ။

## ထိန်းသိမ်းထိုက်သော ချိတ်ဆက်စစ်ဆေးရေး key များ { #correlation-keys-worth-preserving }

| Key | အထောက်အထားများ |
|---|---|
| Path အပြည့်အစုံ / ဖိုင်အမည် | အထောက်အထား အားလုံးနီးပါး |
| SHA-1 / SHA-256 | Amcache၊ Defender၊ ပြန်လည်ရယူထားသော ဖိုင်များ |
| MFT entry + sequence | `$MFT`၊ Prefetch၊ LNK၊ Jump Lists၊ USN |
| USN | Amcache၊ `$MFT`၊ USN Journal |
| Volume serial / droid များ | Prefetch၊ LNK၊ Jump Lists၊ MFT object ID များ |
| SID / အသုံးပြုသူအမည် | SRUM၊ SAM၊ registry၊ event log များ |
| Task GUID / task အမည် | TaskCache၊ task XML၊ event log များ |
| ဝန်ဆောင်မှုအမည် | ဝန်ဆောင်မှုများ၊ event log များ၊ process/driver ဒေတာ |
| Defender EntryId / ScanId | Quarantine၊ Defender မှတ်တမ်း၊ event log များ |
