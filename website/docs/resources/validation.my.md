# Parser မှန်ကန်မှု စစ်ဆေးခြင်း { #parser-validation }

Juntei က ထိန်းသိမ်းသော parser crate များကို အများသုံး အထောက်အထားများနှင့် လွတ်လပ်သော implementation များဖြင့်
စမ်းသပ်ထားပါသည်။ ဤနှိုင်းယှဉ်ချက်များသည် regression များနှင့် အဓိပ္ပာယ်ဖော်ပုံ ကွဲပြားမှုများကို ရှာဖွေရာတွင် ကူညီသော်လည်း
မည်သည့် parser ကိုမျှ အမှားကင်းစေခြင်း မရှိပါ။ ရင်းမြစ် အထောက်အထားကို ထိန်းသိမ်းပါ၊ record တစ်ခုစီ၏ problem သို့မဟုတ်
damage field များကို ပြန်လည်စစ်ဆေးပါ၊ အရေးကြီးသော တွေ့ရှိချက်များကို အခြား artifact သို့မဟုတ် implementation ဖြင့် အတည်ပြုပါ။

အောက်ပါဖော်ပြချက်များသည် ယခင်က Juntei ၏ ပင်မ README တွင် ထားရှိခဲ့သော မှန်ကန်မှု စစ်ဆေးခြင်း အချက်အလက်များကို
ထိန်းသိမ်းထားခြင်း ဖြစ်ပါသည်။ လက်ရှိ test fixture များနှင့် implementation အသေးစိတ်များအတွက် link ပါသော parser repository များကို ကြည့်ပါ။

## Amcache { #amcache }

Juntei သည် Security Ronin ၏
[amcache-forensic](https://github.com/SecurityRonin/amcache-forensic) ၏ Apache-2.0 fork ဖြစ်သော
[ys-amcache-parser](https://github.com/Yamato-Security/ys-amcache-parser) ကို အသုံးပြုပါသည်။ ၎င်း၏
[output စာတမ်း](https://github.com/Yamato-Security/ys-amcache-parser/blob/main/docs/output.md)
သည် executable၊ device၊ finding နှင့် summary field တိုင်းကို သတ်မှတ်ပါသည်။

## ပင်မဖိုင်ဇယား { #master-file-table }

[ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) ၏ mft crate သည် Omer
Ben-Amram ၏ [mft](https://github.com/omerbenamram/mft) crate ၏ ထိန်းသိမ်းထားသော fork ဖြစ်ပါသည်။ `$MFT` ဖိုင် ရှစ်ခု—crate ၏
နမူနာ၊ NIST hacking case နှင့် DFIR Artifact Museum ဖိုင် ခြောက်ခု—တွင် CSV row နှင့် column 1,048,733 ခုသည် ကိုးကား
parser နှင့် အစီအစဉ်အတိုင်း ကိုက်ညီခဲ့ပါသည်။ နှိုင်းယှဉ်ချက်များတွင် မူလ output၊ `--all-times`၊
`--short-names` နှင့် `--cutoff` ပါဝင်ပြီး timestamp ပြသပုံသည် ရည်ရွယ်ချက်ရှိရှိ ကွဲပြားချက် ဖြစ်ခဲ့ပါသည်။

## NTFS transaction log ($LogFile) { #ntfs-transaction-log-logfile }

[ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) ၏ logfile crate ကို
[Sootmark ntfslog](https://github.com/Sootmark/ntfslog) မှ ထိန်းသိမ်းထားပါသည်။ ၎င်း၏ မှန်ကန်မှု စစ်ဆေးခြင်းတွင် DFIR
Artifact Museum ၏ ပြည့်စုံသော log ခြောက်ခု၊ အများသုံး ဗားရှင်း 1.1 နှင့် 2.0 fixture များ၊ decode လုပ်ထားသော redo/undo metadata
နှင့် လွတ်လပ်စွာ parse လုပ်ထားသော NTFS ဒေတာနှင့် နှိုင်းယှဉ်ထားသော journal write များ ပါဝင်ပါသည်။ အသေးစိတ် coverage နှင့်
သိရှိထားသော ကွဲပြားချက်များအတွက် parser repository ကို ကြည့်ပါ။

Juntei သည် အများသုံး go-ntfs test volume (record 2,129 ခု) ပေါ်တွင် CSV field တိုင်းကို JSON နှင့် JSONL တို့နှင့် နှိုင်းယှဉ်သော၊
သီးခြား redo/undo ဒေတာနှင့် raw payload အရှည်များကို အတည်ပြုသော၊ ပျက်စီးနေသော log များကို စမ်းသပ်သော၊ အထောက်အထားကို
ထပ်ရေးခြင်းကို ငြင်းပယ်သော နှင့် လက်ရှိ MFT path များကို ပြန်လည်အသုံးပြုထားသော record number များနောက်သို့ မလိုက်စေသော CLI
စမ်းသပ်ချက်များကို ထပ်ထည့်ထားပါသည်။ Regression ကိစ္စများသည် သက်ရောက်မှုရှိသော MFT record များကို ၎င်းတို့ ရေးသားရာ
ဖွင့်ထားသော attribute မှ ခွဲခြားပြီး flat directory-entry/data-attribute metadata ကို စစ်ဆေးကာ volume ဖြတ်ကျော် MFT တွဲဖက်ခြင်းကို ကာကွယ်ပါသည်။
Volume အများအပြား `auto-parse` စမ်းသပ်ချက်များသည် သီးခြား CSV ဖိုင်များနှင့် provenance ပါသော JSONL ဖိုင်တစ်ခုတည်းကို လွှမ်းခြုံပါသည်။

ညွှန်ကြားချက်ကို ကိုယ်ပိုင် စုဆောင်းထားသော Windows log ခြောက်ခုပေါ်တွင်လည်း စစ်ဆေးခဲ့ပါသည်: export လုပ်ထားသော CSV
record 708,465 ခုလုံးသည် JSONL နှင့် field တစ်ခုချင်း ကိုက်ညီခဲ့ပြီး host တစ်ခုချင်းစီ၏ `auto-parse` export များသည် တူညီသော record
အရေအတွက်များကို ထိန်းထားခဲ့ပါသည်။ ရင်းမြစ် အထောက်အထားကို repository နှင့်အတူ ဖြန့်ဝေမထားပါ။ ဤစစ်ဆေးချက်များသည် output
ကိုက်ညီမှုကို အတည်ပြုခြင်းသာ ဖြစ်ပြီး log ထဲရှိ transaction တိုင်းသည် ပြီးမြောက်ခဲ့သော အသုံးပြုသူ လုပ်ဆောင်ချက်ကို ကိုယ်စားပြုကြောင်း မဟုတ်ပါ။

## Prefetch { #prefetch }

[ys-prefetch-parser](https://github.com/Yamato-Security/ys-prefetch-parser) သည်
ForensicRS ၏ [frnsc-prefetch](https://github.com/ForensicRS/frnsc-prefetch) ၏ ထိန်းသိမ်းထားသော fork ဖြစ်ပါသည်။ ၎င်း၏ decode
လုပ်ထားသော output ကို libyal ၏ လွတ်လပ်သော Prefetch implementation ဖြစ်သည့် [libscca](https://github.com/libyal/libscca)
နှင့် နှိုင်းယှဉ်ခဲ့ပါသည်။

## USN ဂျာနယ် { #usn-journal }

[ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) ၏ usn crate သည်
[Sootmark usn](https://github.com/Sootmark/usn) ၏ ထိန်းသိမ်းထားသော fork ဖြစ်ပါသည်။ တကယ့် journal ခြောက်ခုပေါ်တွင် Juntei ၏
အချိန်ကို သိရှိသော parent path များသည် လွတ်လပ်သော parser တစ်ခုက path ကို ဖြေရှင်းနိုင်ခဲ့သည့်နေရာတိုင်းတွင် ကိုက်ညီခဲ့ပါသည်။ Juntei
သည် နှိုင်းယှဉ်မှုက ဖြေရှင်းမရဘဲ ချန်ထားခဲ့သော record 4,321 ခုကိုလည်း ဖြေရှင်းခဲ့ပါသည်။ ၎င်း၏ carver သည် `$MFT` ဒေတာ 880 MB ဖြင့်
စမ်းသပ်စဉ် မှားယွင်းသော record တစ်ခုမျှ မထုတ်ပေးခဲ့ပါ။

## ဖိုင်တွဲ အညွှန်းများ ($I30) { #directory-indexes-i30 }

[ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) ၏ mft crate ၏ `i30` module သည် INDX
record များနှင့် ၎င်းတို့၏ slack ကို ဖတ်ပါသည်။ ntfs-3g ဖြင့် တည်ဆောက်ထားသော volume ၏ `$I30` stream နှစ်ခု (Juntei ၏
`test_files/i30/`) နှင့် Windows ရေးသားထားသော အများသုံး `$I30` နမူနာ သုံးခုပေါ်တွင် ၎င်း၏ အသုံးပြုဆဲနှင့် slack ရှိ entry များသည်
Maxim Suhanov ၏ [dfir_ntfs](https://github.com/msuhanov/dfir_ntfs) ၏ entry များနှင့် အမည်များနှင့် အချိန်များ တူညီစွာ
ကိုက်ညီခဲ့ပါသည်။ dfir_ntfs က 32 bit ကျော်သော parent reference (parent ကို ထပ်ရေးသွားသော byte များ) ကို အစီရင်ခံသည့်နေရာတွင်
Juntei သည် parent ကို ဗလာထားပါသည်။

## Boot sector ($Boot) { #boot-sector-boot }

[ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) ၏ boot crate သည် Yamato
Security ၏ ကိုယ်ပိုင် ဖြစ်ပါသည်။ DFIR Artifact Museum ၏ အများသုံး `$Boot` ဖိုင် ငါးခု (Windows 10၊ 11 နှင့်
Server 2012 R2) ပေါ်တွင် field တိုင်းသည် ကိုးကား parser နှင့် ကိုက်ညီခဲ့ပါသည်။ 512-၊
1,024- နှင့် 4,096-byte sector များနှင့် 4 KiB မှ 64 KiB cluster များဖြင့် `mkntfs` ဖြင့် ပြုလုပ်ထားသော volume များပေါ်တွင် size များ၊ `$MFT` နှင့် `$MFTMirr`
cluster များနှင့် serial number များသည် The Sleuth Kit ၏ `fsstat` နှင့် ကိုက်ညီခဲ့ပါသည်။ Museum ၏ Windows 11 `$Boot`
တွင် တူညီသော host ၏ LNK ဖိုင်က မှတ်တမ်းတင်ထားသော volume serial number ရှိပြီး ဖိုင်ငါးခုလုံးတွင် တူညီသော boot code ရှိပါသည်။

## လုံခြုံရေးဖော်ပြချက်များ ($Secure:$SDS) { #security-descriptors-securesds }

[ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) ၏ secure crate သည် Yamato
Security ၏ ကိုယ်ပိုင် ဖြစ်ပါသည်။ Museum ၏ `$SDS` stream ငါးခု (descriptor 7,808 ခု၊ ACE 60,781 ခု) တွင်
security ID များ၊ hash များ၊ offset များ၊ owner များ၊ group များ၊ control flag များနှင့် ACE အရေအတွက်များသည် ကိုးကား parser
နှင့် ကိုက်ညီခဲ့ပြီး ACE တိုင်း၏ type၊ flag များ၊ mask၊ SID နှင့် object type များသည် `dissect.ntfs` နှင့် ကိုက်ညီခဲ့ပါသည်။ မှတ်တမ်းတင်ထားသော
hash တိုင်းသည် ၎င်း၏ descriptor ၏ hash ဖြစ်ပြီး mirror မိတ္တူတိုင်းသည် ၎င်း၏ entry နှင့် ကိုက်ညီကာ descriptor တိုင်းတွင် ၎င်း၏
callback ACE 3,384 ခု၏ condition များ အပါအဝင် SDDL ရှိပါသည်။

## SRUM { #srum }

[ys-srum-parser](https://github.com/Yamato-Security/ys-srum-parser) သည် Sootmark ၏
[ese](https://github.com/Sootmark/ese) နှင့် [srum](https://github.com/Sootmark/srum) crate များကို အခြေခံထားပါသည်။ တကယ့်
database 11 ခုတွင် ထုတ်ပေးခဲ့သော entry 45,255 ခုရှိ column တိုင်းသည် `dissect.esedb` က ဖော်ပြသော တန်ဖိုးများ၏
လွတ်လပ်သော decoding နှင့် ကိုက်ညီခဲ့ပါသည်။

## အသုံးပြုသူ ဝင်ရောက်မှု မှတ်တမ်း { #user-access-logging }

`parse-ese ual` သည် UAL database များကို တူညီသော ESE reader ဖြင့် ဖတ်ပါသည်။ တကယ့် database 16 ခု (row 308 ခု၊
၎င်းတို့အနက် 145 ခုသည် client row များ) တွင် value တိုင်းသည် `dissect.esedb` က ဖော်ပြသော တန်ဖိုးများ၏
လွတ်လပ်သော decoding နှင့် ကိုက်ညီခဲ့ပါသည်။ Client row တိုင်းတွင် အရေအတွက်ရှိသော နေ့များသည် `InsertDate` ၏ UTC နေ့မှ
`LastAccess` ၏ UTC နေ့အထိ ရှိခဲ့ပြီး ၎င်းတို့၏ ပေါင်းလဒ်သည် `TotalAccesses` နှင့် ညီခဲ့ပါသည်။

## LNK ဖြတ်လမ်းများ { #lnk-shortcuts }

[ys-lnk-parser](https://github.com/Yamato-Security/ys-lnk-parser) သည်
Sootmark ၏ [shell](https://github.com/Sootmark/shell) crate ၏ ထိန်းသိမ်းထားသော fork ဖြစ်ပါသည်။ ၎င်းကို အများသုံး shortcut
1,118 ခုဖြင့် နှိုင်းယှဉ်ခဲ့ပါသည်။ ကိုးကား parser နှင့် မျှဝေသော CSV column 30 ခုထဲမှ တစ်ခုချင်း တန်ဖိုး 16 ခု ကွဲပြားခဲ့ပါသည်:
ပျက်စီးနေသော၊ ဖြတ်တောက်ခံရသော၊ placeholder၊ MTP သို့မဟုတ် Control Panel item များ ပါဝင်သော target path 13 ခု၊ မသိသော
သို့မဟုတ် ဖြတ်တောက်ခံရသော block များ ပါဝင်သော extra-data-block list နှစ်ခုနှင့် ကိုးကား parser က မတူညီသော byte များမှ
ဖတ်ခဲ့သော MFT reference တစ်ခု ဖြစ်ပါသည်။ Juntei သည် ပျက်စီးနေသော ဖွဲ့စည်းပုံများကို တိတ်တဆိတ် normalize မလုပ်ဘဲ အစီရင်ခံပါသည်။

## Jump Lists { #jump-lists }

[ys-jump-lists-parser](https://github.com/Yamato-Security/ys-jump-lists-parser) သည် Sootmark ၏
compound-file နှင့် Jump List လုပ်ငန်းကို အသုံးပြုပြီး ထည့်သွင်းထားသော link များကို `ys-lnk-parser` ဖြင့် decode လုပ်ပါသည်။ အများသုံး
Jump List 77 ခုတွင် 100 ns အထိ timestamp များ အပါအဝင် မျှဝေထားသော နှိုင်းယှဉ် cell 13,057 ခုလုံးသည် ကိုးကား parser နှင့်
ကိုက်ညီခဲ့ပါသည်။ ကိုးကား parser က ငြင်းပယ်ခဲ့သော ဖိုင်ခြောက်ခု (ဗလာ custom list များ၊ footer ပျောက်နေခြင်း သို့မဟုတ်
Jump List မဟုတ်သော ဖိုင်) ကို Juntei က decode လုပ်ခဲ့ခြင်း သို့မဟုတ် ရှင်းလင်းစွာ အစီရင်ခံခဲ့ခြင်း ဖြစ်ပါသည်။

## ShimCache { #shimcache }

[ys-shimcache-parser](https://github.com/Yamato-Security/ys-shimcache-parser) သည် Security Ronin ၏
[shimcache-forensic](https://github.com/SecurityRonin/shimcache-forensic) ၏ registry-hive နှင့်
transaction-log ပံ့ပိုးမှုပါသော ထိန်းသိမ်းထားသည့် fork ဖြစ်ပါသည်။ အများသုံး `SYSTEM` hive 20 ခု (control set 24 ခုနှင့် entry 7,290 ခု) နှင့်
သီးခြား AppCompatCache value 17 ခုပေါ်တွင် entry တိုင်းသည် ကိုးကား parser က ထုတ်ပေးသော path၊ အချိန်၊ execution flag၊ flag များနှင့်
position တို့နှင့် ကိုက်ညီခဲ့ပါသည်။

## Microsoft Defender quarantine { #microsoft-defender-quarantine }

[ys-defender-history-parser](https://github.com/Yamato-Security/ys-defender-history-parser) သည်
Sootmark ၏ [defender](https://github.com/Sootmark/defender) crate ၏ ထိန်းသိမ်းထားသော fork ဖြစ်ပါသည်။ Parser ၏ test tooling
က Defender ပုံစံဖြင့် ရေးသားထားသော quarantine ဖိုင်တွဲကို လွတ်လပ်သော `dissect.target` quarantine reader က ကိုက်ညီစွာ
decode လုပ်ပါသည်။

## Registry မှ ဆင်းသက်လာသော artifact များ { #registry-derived-artifacts }

[ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser) ကို allocate လုပ်ထားသော key များ၊ value များ၊
dirty-hive transaction replay၊ free-cell ပြန်လည်ရယူမှု၊ cell slack၊ hive bin များအပြင်ဘက်ရှိ ဒေတာနှင့် replay ကြောင့် ဖယ်ရှားခံရသော
သို့မဟုတ် အစားထိုးခံရသော record များအတွက် [yarp](https://github.com/msuhanov/yarp) နှင့် နှိုင်းယှဉ်ထားပါသည်။
Artifact အလိုက် နှိုင်းယှဉ်ချက်များတွင် အောက်ပါတို့ ပါဝင်ပါသည်။

- Windows 7 နှင့် dirty Windows 11 `SYSTEM` hive များမှ service များ၊ value များနှင့် failure action များ၊
- အများသုံး `NTUSER.DAT` hive ကိုးခုတွင် yarp နှင့် libfwsi ဖြင့် decode လုပ်ထားသော RecentDocs၊
- Windows 7–11၊ Server 2012 R2 နှင့် Server 2022 ကို လွှမ်းခြုံသော အများသုံး `SOFTWARE` hive ခုနစ်ခုတွင် yarp ဖြင့်
  လွတ်လပ်စွာ decode လုပ်ထားသော scheduled task များ၊ နှင့်
- အများသုံး `SAM` hive ခုနစ်ခုတွင် `dissect.target` နှင့် နှိုင်းယှဉ်ထားသော SAM account ဖွဲ့စည်းပုံများ။

ကိုးကား tool များ၊ ဖိုင်ပုံစံ သုတေသန၊ spec များနှင့် အများသုံး အထောက်အထား ရင်းမြစ်များ၏ စာရင်းအပြည့်အစုံအတွက်
[ကျေးဇူးတင်လွှာ](../acknowledgements.md) ကို ကြည့်ပါ။
