# DFIR ဝေါဟာရစာရင်း { #dfir-glossary }

| ဝေါဟာရ | ရိုးရှင်းသော အဓိပ္ပာယ် |
|---|---|
| Allocated | Artifact အရ record/object ကို လက်ရှိ အသုံးပြုနေခြင်း။ |
| Alternate Data Stream (ADS) | ဖိုင်၏ ပုံမှန် အမည်မဲ့ အကြောင်းအရာအပြင် ဖိုင်နှင့် တွဲထားသော အမည်ပါ NTFS ဒေတာ၊ ဥပမာ `Zone.Identifier`။ |
| Artifact | Forensic မေးခွန်းများကို ဖြေဆိုနိုင်သော operating system/application က ထုတ်ပေးသည့် ကြာရှည်တည်ရှိသော ဒေတာ။ |
| Carving | Record များကို ၎င်းတို့၏ ပုံမှန် container/index ကို အားမကိုးဘဲ ဖွဲ့စည်းပုံ/signature ဖြင့် ရှာဖွေခြင်း။ |
| Control set | SYSTEM ပြင်ဆင်မှု၏ ဗားရှင်းတစ်ခု (`ControlSet001` စသည်)။ `Select\Current` က active ဖြစ်နေသော တစ်ခုကို ဖော်ပြပါသည်။ |
| Corroboration | တူညီသော ကောက်ချက်ကို ထောက်ခံသော လွတ်လပ်သည့် အထောက်အထား။ |
| Dirty database/hive | စုဆောင်းစဉ်က primary ဖိုင်ကို အပြည့်အဝ synchronize မလုပ်ရသေးခြင်း။ လိုအပ်သော အခြေအနေကို transaction log များက ထိန်းထားနိုင်ပါသည်။ |
| Droid | ဖိုင်/volume များကို ရွှေ့ပြောင်းမှုများအကြား ခြေရာခံရန် အသုံးပြုသော GUID ကဲ့သို့ distributed-link-tracking identifier။ |
| ESE / JET Blue | SRUM နှင့် အခြား Windows ဒေတာများ အသုံးပြုသော Microsoft ၏ Extensible Storage Engine database ပုံစံ။ |
| FILETIME | 1601-01-01 UTC မှစ၍ 100-nanosecond ကာလများ၏ အရေအတွက်။ ဂဏန်းတစ်ခုသည် ၎င်း၏ schema က ထိုသို့ ဖော်ပြမှသာ အချိန်အဖြစ် အဓိပ္ပာယ်ရှိပါသည်။ |
| File reference | MFT record ၏ generation တစ်ခုကို ခွဲခြားသတ်မှတ်ပေးသော NTFS entry number နှင့် sequence number။ |
| Hive | Windows Registry tree ပါဝင်သော binary ဖိုင်။ |
| INDX record | ဖိုင်တွဲ၏ ဖိုင်များကို ဖော်ပြသော NTFS directory index (`$I30`) ၏ block (များသောအားဖြင့် 4 KiB)။ ၎င်း၏ slack သည် ဖျက်ထားသော သို့မဟုတ် အမည်ပြောင်းထားသော ဖိုင်အမည်များ၏ ဖြစ်နိုင်ခြေရှိသော အထောက်အထားကို ထိန်းထားနိုင်ပါသည်။ |
| LNK | Target နှင့် ကျယ်ပြန့်သော metadata ပါဝင်သော Windows Shell Link/shortcut။ |
| MRU | Most Recently Used အစီအစဉ်။ များသောအားဖြင့် item တိုင်းအတွက် အချိန်မဟုတ်ဘဲ နှိုင်းယှဉ် အစီအစဉ် ဖြစ်ပါသည်။ |
| MFT | ဖိုင်/ဖိုင်တွဲ record များနှင့် attribute များ ပါဝင်သော NTFS Master File Table။ |
| Prefetch hash | `.pf` အမည်ပေးရာတွင် အသုံးပြုသော Windows identifier။ cryptographic အကြောင်းအရာ hash မဟုတ်ပါ။ |
| Registry key last-write | Key ပြောင်းလဲခဲ့သည့်အချိန်။ Windows သည် များသောအားဖြင့် မည်သည့် value ပြောင်းလဲခဲ့သည်ကို မဖော်ပြပါ။ |
| Resident data | ပြင်ပ data run များအစား MFT attribute ထဲတွင် တိုက်ရိုက် သိမ်းဆည်းထားသော ဖိုင်/stream byte များ။ |
| RID | Relative Identifier၊ Windows SID authority/domain အောက်ရှိ နောက်ဆုံး account/group အစိတ်အပိုင်း။ |
| SDDL | Security Descriptor Definition Language: `O:S-1-5-18D:(A;;0x001f01ff;;;S-1-5-32-544)` ကဲ့သို့ စာသားဖြင့် ရေးထားသော security descriptor။ |
| Security descriptor | ဖိုင်၏ owner၊ primary group၊ DACL (မည်သူက ဘာလုပ်နိုင်သည်) နှင့် SACL (auditing နှင့် integrity label)။ NTFS သည် ကွဲပြားသော တစ်ခုစီကို security ID တစ်ခုအောက်တွင် `$Secure:$SDS` ၌ တစ်ကြိမ်သာ သိမ်းဆည်းပါသည်။ |
| SID | Account၊ service၊ group သို့မဟုတ် security principal အတွက် Windows Security Identifier။ |
| Slack | ဒေတာအဟောင်းကို ထိန်းထားနိုင်သော allocate လုပ်ထားသည့် ဖွဲ့စည်းပုံ၏ အတွင်း/နောက်ရှိ အသုံးမပြုသော byte များ။ |
| SRUM | Application/အသုံးပြုသူ resource အသုံးပြုမှု၏ System Resource Usage Monitor database။ |
| Timestomping | စုံစမ်းစစ်ဆေးမှုကို လမ်းလွဲစေရန် ရည်ရွယ်ချက်ရှိရှိ timestamp ပြင်ဆင်ခြင်း။ Heuristic များသည် သက်သေ မဟုတ်ပါ။ |
| Transaction log | Registry သို့မဟုတ် database ကို ကိုက်ညီသော အခြေအနေသို့ ပြန်လည်ရယူရန် အသုံးပြုသော ဆိုင်းငံ့ထားသော/commit လုပ်ပြီးသော ပြောင်းလဲမှုများ။ |
| USN | NTFS change-journal record များနှင့် ဆက်စပ် metadata များက အသုံးပြုသော Update Sequence Number။ |
| UTC | Coordinated Universal Time။ စနစ်ဖြတ်ကျော် timeline များအတွက် ဦးစားပေး ဘုံအချိန်အခြေခံ။ |
| Volume serial number | Volume ကို format လုပ်သည့်အခါ NTFS က ပေးသော နံပါတ် (`$Boot`)။ ၎င်း၏ အနိမ့် 32 bit သည် LNK၊ Jump List နှင့် Prefetch ဖိုင်များက ဖိုင်၏ volume အဖြစ် မှတ်တမ်းတင်သည့်အရာ ဖြစ်ပါသည်။ |
