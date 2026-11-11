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

## ဝဘ် ဘရောက်ဇာများ { #web-browsers }

[ys-browser-parser](https://github.com/Yamato-Security/ys-browser-parser) သည် Sootmark ၏ browser၊ sqlite နှင့်
plist crate များ၏ ထိန်းသိမ်းထားသော fork ဖြစ်ပြီး အခြား Rust library တစ်ခုနှင့်လည်းကောင်း၊ တူညီသော database များပေါ်တွင်
Plaso၊ Velociraptor (SQLiteHunter) နှင့် Hindsight တို့နှင့်လည်းကောင်း နှိုင်းယှဉ်ပြီးနောက် ရွေးချယ်ခဲ့ခြင်း ဖြစ်ပါသည်။
`parse-sqlite browser` ကို database တစ်ခုစီ၏ မိတ္တူကို ၎င်း၏ `-wal` ဖိုင်နှင့်အတူ ဖတ်သော `sqlite3` library နှင့်
အများသုံး database 38 ခုပေါ်တွင် နှိုင်းယှဉ်ခဲ့ပါသည်: Plaso ၏ test ဖိုင်များ (Chrome 8 မှ 68၊ Firefox 2 မှ 118)၊
Chromium 150 နှင့် Firefox 140 တို့က ဖန်တီးထားသော profile များနှင့် ys-browser-parser ၏ form history test database
များ။ Visit 293 ခု၊ download 21 ခု၊ cookie 692 ခုနှင့် form entry 41 ခု အားလုံး ကိုက်ညီခဲ့ပါသည်။ ဖျက်ထားသော entry
များကို entry များ အမှန်တကယ် ဖျက်ခံထားရသည့်နေရာတွင်သာ ပြန်လည်ရယူခဲ့ပါသည်: Firefox 140 history မှ ဖျက်ထားသော
စာမျက်နှာ နှစ်ခုနှင့် visit နှစ်ခု (၎င်းတို့၏ အချိန်၊ URL နှင့် title များနှင့်အတူ) နှင့် test database များ၏ ဖျက်ထားသော
form entry များ ဖြစ်ပါသည်။

## Internet Explorer နှင့် legacy Edge (WebCache) { #internet-explorer-and-legacy-edge-webcache }

`parse-ese webcache` သည် `WebCacheV01.dat` ကို
[ys-browser-parser](https://github.com/Yamato-Security/ys-browser-parser) ဖြင့် ဖတ်ပါသည်။ ၎င်းကို Plaso ၏ အများသုံး
WebCache database နှစ်ခုပေါ်တွင် `dissect.database` ၏ ESE reader နှင့် နှိုင်းယှဉ်ခဲ့ပါသည်: Internet Explorer 11 ၏
database တစ်ခုနှင့် legacy Edge ၏ history၊ download များနှင့် cookie များ ပါဝင်သော Windows 10 ၏ dirty database
တစ်ခု ဖြစ်ပါသည်။ Row 4,156 ခု အားလုံးသည် value တစ်ခုချင်းစီအထိ ကိုက်ညီခဲ့ပါသည်: container 44 ခု၊ history entry
182 ခု၊ download 6 ခု၊ cookie 170 ခုနှင့် အခြား entry 3,754 ခု။ နှိုင်းယှဉ်မှုသည် download record များကို ကိုယ်ပိုင်နည်းဖြင့်
သီးခြား decode လုပ်ခဲ့ပါသည်။ Database များက ဖျက်ထားသည်ဟု အမှတ်အသားပြုထားသော record များကို နှစ်ဖက်စလုံးမှ
ချန်လှပ်ထားခဲ့ပါသည်။

## Outlook { #outlook }

`parse-file outlook` သည် PST၊ OST၊ `.msg` နှင့် `.oft` ဖိုင်များကို
[ys-outlook-parser](https://github.com/Yamato-Security/ys-outlook-parser) ဖြင့် ဖတ်ပါသည်။ ၎င်း၏ PST နှင့် OST
reader သည် Microsoft ၏ [outlook-pst-rs](https://github.com/microsoft/outlook-pst-rs) ၏ fork ဖြစ်ပြီး ၎င်း၏ `.msg`
reader ကို Microsoft ၏ open specification များမှ ရေးသားထားပါသည်။

- **PST နှင့် OST**: အများသုံး PST နှင့် OST ဖိုင် 66 ခု (Apache Tika၊ artemis၊ Aspose.Email ၏ ဥပမာများ၊ DFIR
  Artifact Museum၊ Enron၊ GroupDocs ၏ ဥပမာများ၊ java-libpst၊ libyal ၏ test data၊ outlook-pst-rs နှင့်
  pst-extractor) ကို libpff နှင့် property တစ်ခုချင်းစီ နှိုင်းယှဉ်ခဲ့ပါသည်: store၊ ဖိုင်တွဲ၊ message၊ လက်ခံသူနှင့်
  ပူးတွဲဖိုင် တိုင်း၏ property တိုင်းနှင့် ပူးတွဲဖိုင်တိုင်း၏ data။ အချက်အလက် 324,175 ခု တူညီခဲ့ပြီး ရှင်းလင်းချက်
  မရှိဘဲ ကွဲပြားသည့် အချက် တစ်ခုမျှ မရှိပါ။ ကျန်အချက်များမှာ ys-outlook-parser တစ်ခုတည်းသာ ဖတ်နိုင်သော အရာများ
  (libpff မအောင်မြင်သော message များ၊ လက်ခံသူများနှင့် ပူးတွဲဖိုင်များ၏ အချက် 15,846 ခုနှင့် ၎င်း မဖွင့်နိုင်သော
  store တစ်ခု၏ အချက်များ၊ ဝှက်ထားသော item များ၏ အချက် 8,223 ခု)၊ row များက မရှိကြောင်း ဖော်ပြထားသော်လည်း
  libpff က ဗလာ သို့မဟုတ် သုည အဖြစ် ပေးသော လက်ခံသူ cell များ (49,077)၊ MAPI ကဲ့သို့ တွက်ချက်ထားသော ဖိုင်တွဲ
  entry id များနှင့် type များ (2,004) နှင့် libpff က ၎င်းတို့၏ raw data အဖြစ် ပေးသော object များနှင့် ထည့်သွင်းထားသော
  message များ (480) ဖြစ်ပါသည်။ ဖိုင်နှစ်ခုကို reader နှစ်ခုစလုံး လုံးဝ မဖတ်နိုင်ပါ။
- **`.msg` နှင့် `.oft`**: message များ၊ လက်ခံသူများနှင့် ပူးတွဲဖိုင်များ 2,404 ခုနှင့် property 41,002 ခု ပါဝင်သော
  အများသုံး ဖိုင် 214 ခုကို extract-msg နှင့် နှိုင်းယှဉ်ခဲ့ပါသည်: အချက် 110,114 ခု တူညီခဲ့ပြီး ကွဲပြားသော 791 ခု
  တစ်ခုစီတိုင်းကို ရှင်းပြနိုင်ပါသည်: 601 ခုမှာ အချိန်များဆိုင်ရာ extract-msg ၏ အမှားများ၊ 35 ခုမှာ ၎င်းက မှားယွင်းသော
  code page ဖြင့် decode လုပ်သော ANSI ဖိုင်များမှ လာခြင်း ဖြစ်ပြီး ကျန်အချက်များမှာ ၎င်းက ချန်လှပ်သော သို့မဟုတ်
  ထပ်ထည့်သော အရာများ ဖြစ်ပါသည်။
- **ပျက်စီးနေသော input**: mutate လုပ်ထားသော `.msg` ဖိုင် 260,000 ကျော်နှင့် mutate လုပ်ထားသော PST 160,000 ခုနှင့်
  crafted ဖိုင်များကို crash၊ hang သို့မဟုတ် အကန့်အသတ်မဲ့ memory အသုံးပြုမှု မရှိဘဲ ဖတ်ခဲ့ပါသည်။

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

## အမှိုက်ပုံး (Recycle Bin) { #recycle-bin }

`parse-file recycle-bin` သည် Juntei ၏ ကိုယ်ပိုင် parser ဖြစ်ပါသည်။ [rifiuti2](https://github.com/abelcheung/rifiuti2) ၏
အများသုံး sample များပေါ်တွင် rifiuti2 ဖတ်နိုင်သော record တိုင်း၏ index၊ ဖျက်ချိန် (rifiuti2 ပြသသည့်အတိုင်း စက္ကန့်အထိ)၊
bin မှ ဖယ်ရှားမှု အခြေအနေ၊ အရွယ်အစားနှင့် path တို့သည် တူညီခဲ့ပါသည်- record 92 ခု ဖြစ်ပြီး 67 ခုသည် Windows 95 မှ
Server 2003 အထိ INFO2 ဖိုင် ဆယ့်တစ်ခုမှ ဖြစ်ကာ 25 ခုသည် Windows Vista နှင့် Windows 10 `$I` ဖိုင်များ ပါသော ဖိုင်တွဲ
သုံးခုမှ ဖြစ်ပါသည်။ INFO2 ၏ ANSI path များသည် rifiuti2 ၏ ကိုယ်ပိုင် test များ အသုံးပြုသော code page များတွင်
ကိုက်ညီခဲ့ပါသည်။ Juntei သည် rifiuti2 ငြင်းပယ်သည့်အရာများကိုလည်း ဖတ်ပါသည်- `$I` ဗားရှင်း နှစ်မျိုးလုံး ရောနှောနေသော
ဖိုင်တွဲ၊ path မပါသော `$I` header နှင့် ဖြတ်တောက်ခံထားရသော record များ ဖြစ်ပြီး တစ်ခုချင်းစီ၏ ပျက်စီးမှုကို
အစီရင်ခံပါသည်။ Plaso ၏ test ဒေတာရှိ `$I` နှင့် INFO2 ဖိုင်များသည် Plaso ၏ test များ မျှော်လင့်ထားသော အချိန်၊
အရွယ်အစား၊ path၊ index နှင့် drive များကို ပေးပါသည်။ Digital Corpora ၏ အများသုံး M57-Jean disk image (Windows XP) ကို
`auto-parse --e01` ဖြင့် ဖတ်သောအခါ INFO2 record သည် rifiuti2 နှင့် ကိုက်ညီပြီး ၎င်းဘေးရှိ recycle လုပ်ထားသော ဖိုင်
`Dc1.jpg` ၏ အမည်သည်လည်း Juntei ပေးသော အမည်နှင့် တူညီပါသည်။

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
