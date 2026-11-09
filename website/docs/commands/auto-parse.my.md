# စုဆောင်းထားသော ဒေတာကို အလိုအလျောက် ခွဲခြမ်းခြင်း { #automatic-collection-parsing }

`auto-parse` သည် ကွန်ပျူတာ အများအပြားမှ Windows triage collection ကို ဖွဲ့စည်းထားသော ခွဲခြမ်းစိတ်ဖြာမှု ဖိုင်တွဲအဖြစ်
အမြန်ဆုံး ပြောင်းလဲပေးသော နည်းလမ်း ဖြစ်ပါသည်။ ၎င်းကို အထောက်အထား root နှင့် output ဖိုင်တွဲအသစ်တစ်ခု ပေးပါ။ ၎င်းသည်
စုဆောင်းထားသော စနစ်များကို ရှာဖွေတွေ့ရှိပြီး ၎င်းတို့၏ ကွန်ပျူတာအမည်များကို ခွဲခြားသတ်မှတ်ကာ ကိုက်ညီသော Juntei parser
တိုင်းကို အပြိုင် လည်ပတ်စေပါသည်။

```bash
juntei auto-parse -d <EVIDENCE> -o <OUTPUT>
```

မူလ output သည် CSV ဖြစ်ပါသည်။ ထို CSV ရလဒ်များကို ထိန်းထားရင်း `jq` သို့မဟုတ် SIEM import အတွက် JSONL timeline
တစ်ခုကို ထပ်ထည့်ပါ။

```bash
juntei auto-parse -d <EVIDENCE> -o <OUTPUT> -j <OUTPUT>/timeline.jsonl
```

!!! warning "Output နှင့် အထောက်အထား tree များကို သီးခြားထားပါ"
    Juntei သည် `<EVIDENCE>` အတွင်းရှိသော သို့မဟုတ် `<EVIDENCE>` ကို ပါဝင်သော output ဖိုင်တွဲကို လက်မခံပါ။ ၎င်းသည်
    ထုတ်လုပ်ထားသော ဒေတာကို အထောက်အထားအဖြစ် ပြန်လည်ရှာဖွေတွေ့ရှိခြင်းမှ ကာကွယ်ပြီး `--clobber` က ရင်းမြစ် collection
    ရှိ ဖိုင်များကို ထိခိုက်ခြင်းမှ ကာကွယ်ပါသည်။

## VHDX နှင့် E01 disk image များ { #vhdx-and-e01-disk-images }

`--vhdx` ဖြင့် `auto-parse` သည် အထောက်အထား directory ရှိ VHDX disk image တစ်ခုစီအတွင်းမှ system များကို parse လုပ်ပြီး `--e01` ဖြင့် E01 image တစ်ခုစီ (split image ဖြစ်ပါက ၎င်း၏ ပထမ segment၊ `*.E01` သို့မဟုတ် `*.Ex01`) အတွင်းမှ system များကို parse လုပ်သည်။ Option နှစ်ခုကို အတူတကွ ပေးနိုင်သည်။ Parsing လုပ်နေစဉ် image ကို FUSE ဖြင့် read-only mount လုပ်သဖြင့် ဖိုင်များကို ထုတ်ယူခြင်းမရှိပါ။ [Disk image များကို mount လုပ်ခြင်း](mount.md) ကို ကြည့်ပါ။

```bash
juntei auto-parse -d /cases/vhdx --vhdx -o parsed
juntei auto-parse -d /cases/e01 --e01 -o parsed
```

NTFS volume တစ်ခုကို မဖွင့်နိုင်ပါက Juntei က ပြဿနာကို ဖော်ပြပြီး ဖတ်နိုင်သော volume များကို ဆက်လက်သုံးသည်။ Image ကို တစ်စိတ်တစ်ပိုင်းသာ ဖတ်နိုင်ပါက `auto-parse --vhdx` နှင့် `--e01` သည် exit status 1 ကို ပြန်ပေးပြီး `mount` သည်လည်း unmount ပြီးနောက် 1 ကို ပြန်ပေးသည်။

## Output time zone { #output-time-zone }

Parse လုပ်ထားသော timestamp များသည် analyst ကွန်ပျူတာ၏ local time zone နှင့် မသက်ဆိုင်ဘဲ မူလအားဖြင့် UTC ဖြစ်ပါသည်။ UTC
သည် Windows event-log ပုံစံ `2026-10-20T08:46:38.2001668Z` ကို အသုံးပြုပါသည်: ISO 8601 ရက်စွဲ/အချိန်၊ ရက်စွဲနှင့် အချိန်ကြားတွင်
`T` နှင့် UTC အတွက် `Z` ဖြစ်ပါသည်။ ရရှိနိုင်သော ဒဿမ ဂဏန်းများကို ထိန်းသိမ်းထားပါသည်။
`manifest.json` ၏ `GeneratedUTC` သည်လည်း ဤပုံစံကို အသုံးပြုပါသည်။ Console ၏ `Start time` နှင့် `End time`
တို့သည် output time zone မည်သို့ပင်ဖြစ်စေ `2026-11-04 09:15:02` ကဲ့သို့ analyst ကွန်ပျူတာ၏ local အချိန် ဖြစ်ပါသည်။

ပုံသေ ပြသမည့် offset ကို `-Z, --timezone` ဖြင့် ရွေးချယ်ပါ။

```bash
juntei auto-parse -d evidence/ -o parsed-jst/ --timezone "+09:00"
juntei auto-parse -d evidence/ -o parsed-offset/ -Z "-01:00"
juntei auto-parse -d evidence/ -o parsed-half-hour/ -Z "+02:30"
```

Offset သည် အပေါင်း/အနှုတ် လက္ခဏာပါသော ဂဏန်းနှစ်လုံး `HH:MM` တန်ဖိုး ဖြစ်ပါသည် (နာရီ `00–23`၊ မိနစ် `00–59`)။
`JST` ကဲ့သို့ အမည်များ၊ `+9` ကဲ့သို့ အတိုချုံ့ထားသော တန်ဖိုးများနှင့် မမှန်ကန်သော အပိုင်းအခြားများသည် ဥပမာ syntax ပါသော error
ကို ထုတ်ပေးပါသည်။ UTC အတွက် option ကို ချန်လှပ်ပါ သို့မဟုတ် `--timezone "+00:00"` ကို အသုံးပြုပါ။ ပုံသေ offset သည်
daylight saving စည်းမျဉ်းများကို အသုံးမချပါ။

Option သည် artifact တစ်ခုချင်းစီ၏ CSV၊ JSON နှင့် JSONL ရလဒ်များနှင့် ထပ်ဆောင်း `-j` JSONL export ရှိ parse လုပ်ထားသော
timestamp များနှင့် သက်ဆိုင်ပါသည်။ ဥပမာ `2026-10-20T08:46:38.2001668Z` သည်
`2026-10-20T17:46:38.2001668+09:00` ဖြစ်လာပါသည်။ နှစ်ခုလုံးသည် တူညီသော အခိုက်အတန့်ကို ဖော်ပြပါသည်။

`UTC` ဖြင့် အဆုံးသတ်သော ရှိပြီးသား အမည်များ အပါအဝင် field အမည်များသည် တည်ငြိမ်စွာ ရှိနေပါသည်။ ပြောင်းလဲထားသော တန်ဖိုးများသည်
ရွေးချယ်ထားသော offset ကို တိကျစွာ သယ်ဆောင်ပါသည်။ Manifest နှစ်ခုလုံးရှိ `OutputTimeZone` သည် တောင်းဆိုထားသော ပြသမည့်
offset ကို မှတ်တမ်းတင်ပြီး `GeneratedUTC` သည် UTC run metadata အဖြစ် ကျန်ရှိပါသည်။ Parser တစ်ခုချင်းစီ၏ log များသည် parser
များ၏ မူလ UTC အနှစ်ချုပ်များကို ထိန်းထားပါသည်။

Raw FILETIME/epoch ဂဏန်းများ၊ registry-value dictionary များ၊ ရင်းမြစ် time-zone ပြင်ဆင်မှု၊ text metadata၊ ရက်စွဲသာပါသော
တန်ဖိုးများနှင့် သိရှိထားသော offset မရှိသည့် local နာရီတန်ဖိုးများကို ထိန်းသိမ်းထားပါသည်။ Juntei သည် မသိသော local အချိန်တွင်
`Z` ကို မထည့်ပါ။ FAT မှ ဆင်းသက်လာသော shell-item နာရီများ (ShellBags/OpenSaveMRU ရှိ `Item*UTC` နှင့် LNK/Jump List
`TargetItems`) တွင် မှတ်တမ်းတင်ထားသော offset မရှိပါ။ ၎င်းတို့၏ legacy UTC label များ သို့မဟုတ် ပြသထားသော `Z` suffix သည် UTC
ဖြစ်ကြောင်း မသက်သေပြသောကြောင့် offset option က ၎င်းတို့ကို ထိန်းသိမ်းထားပါသည်။
လွတ်လပ်သော စာသား အနှစ်ချုပ်များ ပါဝင်သော CSV column များသည် ၎င်းတို့၏ မှတ်တမ်းတင်ထားသော စာသားကို ထိန်းထားပြီး
JSON/JSONL သည် nested ဖြစ်သော parse လုပ်ထားသည့် timestamp များကို ဖွဲ့စည်းထားသော field များအဖြစ် ထိန်းသိမ်းပါသည်။

UTC သည် ရှိပြီးသား parser output ကို တိုက်ရိုက် အသုံးပြုပါသည်။ သုညမဟုတ်သော offset ကို တိကျစွာ သတ်မှတ်လျှင် parser worker
တစ်ခုစီအတွင်း streaming formatting pass တစ်ခု ထပ်ထည့်ပါသည်။ ၎င်းသည် အထောက်အထားကို မပြောင်းလဲဘဲ ပြသမည့် တန်ဖိုးများကိုသာ ပြောင်းလဲပါသည်။

Offset က timestamp ကို နှစ် `0000–9999` အပြင်ဘက်သို့ ရွှေ့မည်ဆိုလျှင် ထိုတန်ဖိုးသည် ၎င်း၏ မူလ တိကျသော offset ကို ထိန်းထားပါသည်။
ဒဿမ ဂဏန်း ကိုးလုံးထက် ပိုသော တန်ဖိုးများကိုလည်း ပတ်ခြင်းမပြုဘဲ မပြောင်းလဲဘဲ ထားပါသည်။ Juntei သည် parser ၏ `*-logs.txt` ဖိုင်တွင်
warning တစ်ခုကို မှတ်တမ်းတင်ပြီး အခြား timestamp များကို ပြောင်းလဲပါသည်။ ပုံစံမမှန်သော CSV/JSON သို့မဟုတ် output error သည်
task ကို မအောင်မြင်စေဆဲ ဖြစ်ပြီး သက်ရောက်မှုရှိသော မူလ ရလဒ်ဖိုင်ကို မပြောင်းလဲဘဲ ထားပါသည်။

## ၎င်း ရှာဖွေတွေ့ရှိသည့် အရာများ { #what-it-discovers }

Juntei သည် `System32`၊ `Prefetch` သို့မဟုတ် `AppCompat` ဖိုင်တွဲ ပါဝင်သော `Windows` အမည်ရှိ ဖိုင်တွဲများကို
စာလုံးအကြီးအသေး မခွဲခြားဘဲ recursive ရှာဖွေပါသည်။ နောက်ဆုံးနှစ်ခုကို အသိအမှတ်ပြုခြင်းဖြင့် Prefetch သို့မဟုတ် Amcache သာ
ပါဝင်သော ရည်ရွယ်ချက်ရှိ collection များကိုလည်း ပံ့ပိုးပါသည်။ ၎င်းသည် `AppData\Roaming\Microsoft\Windows` ကဲ့သို့
user-profile ဖိုင်တွဲများကို ဖယ်ထုတ်ပါသည်။ `C:\Windows.old\Windows` ကဲ့သို့ nested ဖြစ်သော legacy ထည့်သွင်းမှုများကို
ထပ်နေသော ကွန်ပျူတာများ မဖြစ်စေဘဲ အပြင်ဘက် စုဆောင်းထားသော volume သို့ သတ်မှတ်ပါသည်။ မူလ အများဆုံး depth သည် 10
ဖြစ်ပါသည်။ ၎င်းသည် အောက်ပါကဲ့သို့ အသုံးများသော Velociraptor path များကို လွှမ်းခြုံပါသည်။

```text
<EVIDENCE>/<collection>/uploads/auto/C%3A/Windows
```

Collection ကို ပိုမိုနက်ရှိုင်းစွာ nest လုပ်ထားမှသာ ကန့်သတ်ချက်ကို ပြောင်းပါ။

```bash
juntei auto-parse -d evidence -o parsed --max-depth 15
```

ရှာဖွေတွေ့ရှိသော စနစ်တစ်ခုစီအတွက် ကွန်ပျူတာအမည်ကို ဤအစီအစဉ်အတိုင်း ရွေးချယ်ပါသည်။

1. အနီးဆုံး ancestor `client_info.json` ရှိ `hostname` (၎င်း၏ တန်ဖိုးကို manifest တွင် `FQDN` အဖြစ် ထိန်းထားသည်)။
2. စုဆောင်းထားသော `SYSTEM` registry hive ရှိ `ComputerName`။
3. Drive နှင့် Velociraptor `uploads/auto` scaffolding ကို ကျော်ပြီး `_F.…` ကဲ့သို့ suffix ကို ဖယ်ရှားထားသော
   အနီးဆုံး အသုံးဝင်သည့် collection ဖိုင်တွဲ အမည်။

အမည်များကို ဖိုင်တွဲအမည်များအဖြစ် အသုံးပြုရန် ဘေးကင်းအောင် ပြုလုပ်ပါသည်။ ထပ်နေသော အမည်များသည် ၎င်းတို့၏ output ဖိုင်တွဲနှင့်
`Comp` တန်ဖိုး နှစ်ခုလုံးတွင် ဂဏန်း suffix ရရှိသောကြောင့် သီးခြားစနစ်များကို JSON တွင် ခွဲခြားနိုင်ဆဲ ဖြစ်ပါသည်။
Manifest သည် hostname ကို ရှာဖွေတွေ့ရှိသည့်အတိုင်း သီးခြား `Hostname` field တွင် ထိန်းထားပါသည်။

စနစ်များကို ဖော်ပြပြီးနောက် `auto-parse` သည် `Parser tasks` ၏ အပေါ်တွင် တိုက်ရိုက် `Timezone:` စာကြောင်းတစ်ကြောင်းကို
ပုံနှိပ်ပါသည်။ ၎င်းသည် ရွေးချယ်ထားသော output timezone ကို ပြသပါသည်: မူလအားဖြင့် `UTC`၊ သို့မဟုတ် `--timezone "+09:00"`
ကို အသုံးပြုသည့်အခါ `UTC+09:00` ကဲ့သို့ တန်ဖိုး ဖြစ်ပါသည်။ Label သည် အစိမ်းရောင်ဖြစ်ပြီး တန်ဖိုးသည် အခြား အနှစ်ချုပ်
တန်ဖိုးများကဲ့သို့ terminal ၏ မူလ စာသားအရောင်ကို အသုံးပြုပါသည်။ `-K, --no-color` က စာကြောင်းကို ရိုးရိုး ထားပါသည်။
စုဆောင်းထားသော စနစ်များ၏ time-zone ပြင်ဆင်မှုကို [စနစ်အချက်အလက်](../artifacts/system-info.md) ရလဒ်များတွင်
ရရှိနိုင်ပါသည်။

## ၎င်း လည်ပတ်စေသော parser များ { #parsers-it-runs }

Parser တစ်ခု မအောင်မြင်ခြင်းက အခြား parser များကို မရပ်တန့်စေပါ။ `auto-parse` သည် ရှာဖွေတွေ့ရှိသော ကွန်ပျူတာတိုင်းအတွက်
ဤ job များကို လည်ပတ်စေပါသည်။ Job များသည် `parse-ntfs mft`၊ `parse-registry amcache`၊ `parse-ese srum` နှင့်
`parse-file prefetch` ကဲ့သို့ manual parsing နှင့် တူညီသော ရင်းမြစ်အလိုက် အုပ်စုဖွဲ့ထားသည့် ညွှန်ကြားချက်များကို အသုံးပြုပါသည်။

- Amcache
- `$MFT`
- `$LogFile` NTFS redo/undo record များ
- Prefetch record များနှင့် သီးခြား Prefetch execution timeline
- USN Journal
- `$I30` directory index များ
- `$Boot` boot sector များနှင့် `$Secure:$SDS` security descriptor များ
- SRUM
- အသုံးပြုသူ ဝင်ရောက်မှု မှတ်တမ်း (UAL)
- ဝဘ် ဘရောက်ဇာ history၊ download များ၊ cookie များနှင့် form history (Chromium-based နှင့် Firefox-based ဘရောက်ဇာများ)
- Internet Explorer နှင့် legacy Edge ၏ WebCache: history၊ download များ၊ cookie များနှင့် cache entry များ
- LNK shortcut များ
- Jump Lists
- ShimCache
- Persistence Hunter (allow/block profile များပါသော ရင်းမြစ်ဖြတ်ကျော် persistence စစ်ဆေးချက် 131 ခု)
- Defender quarantine metadata
- Open နှင့် Save As dialog မှတ်တမ်း
- ပြန်လည်ရယူနိုင်သော ဖျက်ထားသည့် ဒေတာ အပါအဝင် registry key နှင့် value အားလုံး
- Service များ၊ RecentDocs၊ TaskCache၊ SAM account များ၊ registry autostart တည်နေရာများ၊ UserAssist၊ BAM၊
  ထည့်သွင်းထားသော ပရိုဂရမ်များ၊ MuiCache၊ ShellBags နှင့် USB device များ
- စနစ်အချက်အလက်: အမည်များ၊ Windows ဗားရှင်း၊ time zone၊ နောက်ဆုံး shutdown၊ profile များနှင့် SID များ
- Audit policy: စနစ်ကို မည်သည့် security event များ log လုပ်ရန် သတ်မှတ်ထားခဲ့သည်
- Boot ပြင်ဆင်မှု (BCD နှင့် BCD-Template)
- ကွန်ရက် profile များ၊ interface များ၊ share များ၊ port proxy များနှင့် route များ
- LSA secret များ၏ အမည်များနှင့် အချိန်များ၊ အသုံးပြုဆဲ cached domain logon များ (မည်သည့်အရာကိုမျှ decrypt မလုပ်ပါ)

Persistence Hunter သည် တစ်စိတ်တစ်ပိုင်း allow ကိုက်ညီမှုများ အပါအဝင် severity တိုင်းကို ထိန်းထားပါသည် (`--min-severity info`)။
`LOW` သည် စုံစမ်းရန် အမျိုးအစားခွဲခြားချက် ဖြစ်ပါသည်။ Signature metadata မရှိခြင်းကြောင့် အန္တရာယ်ရှိသော command များအတွက်လည်း
၎င်းကို ထုတ်ပေးနိုင်ပါသည်။ အလိုအလျောက် inventory သည် ဤ entry များကို manual ညွှန်ကြားချက်၏ မူလ medium threshold ဖြင့်
မဖယ်ရှားပါ။

Hunter သည် အလိုအလျောက် parser process တစ်ခုစီအတွင်း worker တစ်ခုကို အသုံးပြုပါသည်။ ၎င်းသည် `auto-parse -T <N>`
၏ process budget ကို child တိုင်းရှိ CPU-size pool နောက်တစ်ခုဖြင့် မြှောက်ခြင်းမှ ကာကွယ်ပါသည်။ သီးခြား
`persistence-hunter -T <N>` သည် hunter ၏ ကိုယ်ပိုင် worker thread များကို ထိန်းချုပ်ပါသည်။

CSV `$MFT` output တွင် timestamp အားလုံးနှင့် short name များ ပါဝင်ပါသည်။ Registry output တွင် raw ဒေတာနှင့်
`REG_QWORD` value များ၏ FILETIME အဓိပ္ပာယ်ဖော်ချက် ပါဝင်ပါသည်။ JSON နှင့် JSONL SRUM output တွင် value-state
metadata ပါဝင်ပါသည်။ Defender ဖိုင်များနှင့် resident `$MFT` stream များကို အလိုအလျောက် **မထုတ်ယူပါ**။

`--mft-slack` ဖြင့် `$MFT` output တွင် FILE record များ၏ slack ၌ ကျန်ရှိနေသော file name များကိုလည်း [`parse-ntfs mft -x`](../artifacts/mft.md) ရေးသည့်အတိုင်း ထည့်သွင်းပါသည်- CSV တွင် `FromSlack` အမှတ်အသားပါ row များ၊ JSON နှင့် JSONL တွင် `SlackFileNames` ဖြစ်ပါသည်။ `--code-page` ကို LNK ဖိုင်များ၊ Jump List များ၊ ShellBags နှင့် OpenSaveMRU တို့တွင် အသုံးပြုပါသည်။

စုဆောင်းမထားသော artifact သည် ပုံမှန် `skipped` task ဖြစ်ပြီး မအောင်မြင်မှု မဟုတ်ပါ။

## Output layout { #output-layout }

ရလဒ်များကို ကွန်ပျူတာနှင့် စုံစမ်းစစ်ဆေးရေး အမျိုးအစားအလိုက် စီစဉ်ထားပါသည်။ အမျိုးအစား ဖိုင်တွဲများသည် `Accounts` နှင့်
`FileAndFolderOpening` ကဲ့သို့ PascalCase ကို အသုံးပြုပါသည်။ Amcache နှင့် Jump Lists table များသည် ဖိုင်အမည် prefix များဖြင့်
၎င်းတို့၏ အမျိုးအစားများထဲတွင် တိုက်ရိုက် ရှိပါသည်။ ကွန်ပျူတာ ဖိုင်တွဲအမည်များသည် ရှာဖွေတွေ့ရှိသည့် စာလုံးပေါင်းအတိုင်း ရှိပါသည်။ ရှိပြီးသား output
ဖိုင်တွဲများကို migrate မလုပ်ပါ။ အမည်ပေးပုံ အဟောင်းမှ ပြောင်းသည့်အခါ output ဖိုင်တွဲအသစ်ကို ရွေးချယ်ပါ။ Parser တစ်ခုစီသည်
artifact ကို မစုဆောင်းထားသည့်အခါ အပါအဝင် ၎င်း၏ ရလဒ်များဘေးတွင် `<artifact>-logs.txt` ဖိုင်တစ်ခုကို ရေးသားပါသည်။
Artifact တစ်ခုစီတွင် မတူညီသော column များ ရှိသောကြောင့် CSV သည် သီးခြား table များကို ထားပါသည်။

```text
<OUTPUT>/
├── manifest.json
├── manifest.csv
├── HOST1/
│   ├── Accounts/
│   │   ├── Registry-LsaSecrets.csv
│   │   ├── Registry-LsaSecrets-logs.txt
│   │   ├── Registry-SAM.csv
│   │   └── Registry-SAM-logs.txt
│   ├── Applications/
│   │   ├── Registry-AppHistory.csv
│   │   ├── Registry-AppHistory-logs.txt
│   │   ├── Registry-InstalledPrograms.csv
│   │   └── Registry-InstalledPrograms-logs.txt
│   ├── BrowserActivity/
│   │   ├── Browser-Cookies.csv
│   │   ├── Browser-Downloads.csv
│   │   ├── Browser-FormHistory.csv
│   │   ├── Browser-History.csv
│   │   ├── Browser-logs.txt
│   │   ├── WebCache-Containers.csv
│   │   ├── WebCache-Cookies.csv
│   │   ├── WebCache-Downloads.csv
│   │   ├── WebCache-Entries.csv
│   │   ├── WebCache-History.csv
│   │   └── WebCache-logs.txt
│   ├── ExternalDevices/
│   │   ├── Registry-USB.csv
│   │   └── Registry-USB-logs.txt
│   ├── FileAndFolderOpening/
│   │   ├── JumpLists-AutomaticDestinations.csv
│   │   ├── JumpLists-CustomDestinations.csv
│   │   ├── JumpLists-logs.txt
│   │   ├── LNK.csv
│   │   ├── LNK-logs.txt
│   │   ├── Registry-Office.csv
│   │   ├── Registry-Office-logs.txt
│   │   ├── Registry-MRU.csv
│   │   ├── Registry-MRU-logs.txt
│   │   ├── Registry-OpenSaveMRU.csv
│   │   ├── Registry-OpenSaveMRU-logs.txt
│   │   ├── Registry-RecentDocs.csv
│   │   ├── Registry-RecentDocs-logs.txt
│   │   ├── Registry-ShellBags.csv
│   │   └── Registry-ShellBags-logs.txt
│   ├── FileExistence/
│   │   ├── Registry-Amcache-Devices.csv
│   │   ├── Registry-Amcache-Files.csv
│   │   ├── Registry-Amcache-Findings.csv
│   │   ├── Registry-Amcache-Summary.csv
│   │   ├── Registry-Amcache-logs.txt
│   │   ├── Registry-ShimCache.csv
│   │   └── Registry-ShimCache-logs.txt
│   ├── FileSystem/
│   │   ├── Boot.csv
│   │   ├── Boot-logs.txt
│   │   ├── I30.csv
│   │   ├── I30-logs.txt
│   │   ├── LogFile.csv
│   │   ├── LogFile-logs.txt
│   │   ├── MFT.csv
│   │   ├── MFT-logs.txt
│   │   ├── SecureSDS.csv
│   │   ├── SecureSDS-logs.txt
│   │   ├── UsnJournal.csv
│   │   └── UsnJournal-logs.txt
│   ├── NetworkActivity/
│   │   ├── Registry-Networks.csv
│   │   ├── Registry-Networks-logs.txt
│   │   ├── SRUM-NetworkConnections.csv
│   │   ├── SRUM-NetworkUsages.csv
│   │   └── SRUM-PushNotifications.csv
│   ├── LateralMovement/
│   │   ├── Registry-RemoteAccess.csv
│   │   ├── Registry-RemoteAccess-logs.txt
│   │   ├── UAL-Clients.csv
│   │   ├── UAL-RoleAccess.csv
│   │   ├── UAL-SystemIdentity.csv
│   │   └── UAL-logs.txt
│   ├── Persistence/
│   │   ├── Registry-BCD.csv
│   │   ├── Registry-BCD-logs.txt
│   │   ├── Registry-Persistence.csv
│   │   ├── Registry-Persistence-logs.txt
│   │   ├── Registry-Services.csv
│   │   ├── Registry-Services-logs.txt
│   │   ├── Registry-TaskCache.csv
│   │   └── Registry-TaskCache-logs.txt
│   ├── ProcessExecution/
│   │   ├── Registry-AppCompat.csv
│   │   ├── Registry-AppCompat-logs.txt
│   │   ├── Registry-FeatureUsage.csv
│   │   ├── Registry-FeatureUsage-logs.txt
│   │   ├── Registry-BAM.csv
│   │   ├── Registry-BAM-logs.txt
│   │   ├── Registry-MuiCache.csv
│   │   ├── Registry-MuiCache-logs.txt
│   │   ├── Prefetch.csv
│   │   ├── Prefetch-logs.txt
│   │   ├── PrefetchTimeline.csv
│   │   ├── PrefetchTimeline-logs.txt
│   │   ├── SRUM-AppResourceUseInfo.csv
│   │   ├── SRUM-AppTimelineProvider.csv
│   │   ├── Registry-UserAssist.csv
│   │   └── Registry-UserAssist-logs.txt
│   ├── Registry/
│   │   ├── Registry-Dump.csv
│   │   └── Registry-Dump-logs.txt
│   ├── SecurityProducts/
│   │   ├── Registry-SecuritySettings.csv
│   │   ├── Registry-SecuritySettings-logs.txt
│   │   ├── DefenderQuarantine.csv
│   │   ├── DefenderQuarantine-logs.txt
│   │   └── DefenderQuarantined/  # only with -x
│   └── SystemInformation/
│       ├── Registry-AuditPolicy.csv
│       ├── Registry-AuditPolicy-logs.txt
│       ├── Registry-SystemInfo.csv
│       ├── Registry-SystemInfo-logs.txt
│       ├── SRUM-IdentityMap.csv
│       ├── SRUM-VfuProvider.csv
│       └── SRUM-logs.txt
└── HOST2/
    └── …
```

| အမျိုးအစား | Artifact များ |
|---|---|
| `Accounts` | SAM စက်တွင်း account များ၊ LSA secret များ၏ အမည်များနှင့် အချိန်များ |
| `Applications` | Application မှတ်တမ်း (third-party tool များ၏ ခြေရာများ) |
| `Applications` | ထည့်သွင်းထားသော ပရိုဂရမ်များ |
| `BrowserActivity` | ဝဘ် ဘရောက်ဇာ history၊ download များ၊ cookie များနှင့် form history၊ ပြန်လည်ရယူထားသော ဖျက်ထားသည့် entry များ အပါအဝင်။ Internet Explorer နှင့် legacy Edge ၏ WebCache |
| `ExternalDevices` | USB device များနှင့် volume များ |
| `FileAndFolderOpening` | Jump Lists၊ LNK shortcut များ၊ Microsoft Office၊ MRU စာရင်းများ၊ Open နှင့် Save As dialog များ၊ RecentDocs၊ ShellBags |
| `FileExistence` | Amcache, ShimCache |
| `FileSystem` | `$Boot`, `$I30`, `$LogFile`, `$MFT`, `$Secure:$SDS`, USN Journal |
| `NetworkActivity` | ကွန်ရက် profile များ၊ interface များ၊ share များ၊ port proxy များနှင့် route များ၊ SRUM ချိတ်ဆက်မှုများ၊ traffic နှင့် push notification များ |
| `LateralMovement` | အသုံးပြုသူများ၏ အပြင်ဘက်သို့ Remote Desktop၊ PuTTY နှင့် WinSCP ချိတ်ဆက်မှုများ၊ map လုပ်ထားသော drive များနှင့် share များ။ အသုံးပြုသူ ဝင်ရောက်မှု မှတ်တမ်း (server တစ်ခု၏ role များကို အသုံးပြုသော client များ) |
| `Persistence` | Boot ပြင်ဆင်မှု (BCD)၊ registry autostart များ၊ service များ၊ TaskCache |
| `ProcessExecution` | Application compatibility၊ BAM နှင့် DAM၊ FeatureUsage နှင့် pin များ၊ MuiCache၊ Prefetch၊ Prefetch timeline၊ SRUM application/CPU လှုပ်ရှားမှု၊ UserAssist |
| `Registry` | ပြန်လည်ရယူထားသော record များ အပါအဝင် ယေဘုယျ hive key များနှင့် value များ |
| `SecurityProducts` | Defender quarantine metadata နှင့် ရွေးချယ်နိုင်သော ထုတ်ယူထားသည့် အကြောင်းအရာ၊ လုံခြုံရေးဆက်တင်များ (Defender၊ UAC၊ credential ကာကွယ်မှု၊ RDP၊ firewall၊ logging) |
| `SystemInformation` | အမည်များ၊ Windows ဗားရှင်း၊ time zone၊ နောက်ဆုံး shutdown၊ profile များ၊ SID များနှင့် audit policy၊ SRUM စွမ်းအင်/capacity၊ identity map များနှင့် မသိသော provider များ |

အမျိုးအစားများသည် စုံစမ်းစစ်ဆေးမှုများကို စီစဉ်ရာတွင် ကူညီသော်လည်း သီးခြားလှုပ်ရှားမှုတစ်ခု၏ သက်သေ မဟုတ်ပါ။ ဥပမာ
Amcache နှင့် ShimCache entry များ တစ်ခုတည်းဖြင့် execution ကို အတည်မပြုနိုင်ပါ။ SRUM သည် resource အသုံးပြုမှုကို
မှတ်တမ်းတင်ပါသည်။ ပရိုဂရမ်တစ်ခု လည်ပတ်ခဲ့ကြောင်း ကောက်ချက်မချမီ artifact လမ်းညွှန်ကို ကိုးကားပါ။

CSV၊ JSON နှင့် JSONL တို့သည် တူညီသော အမျိုးအစားများကို အသုံးပြုပါသည်။ Parser job တစ်ခုလျှင် `<artifact>-logs.txt` ဖိုင်တစ်ခုတွင်
၎င်း၏ တိကျသော command နှင့် message များ ပါဝင်ပါသည်။ ဥပမာ `HOST1/FileSystem/MFT-logs.txt` နှင့်
`HOST1/ProcessExecution/Prefetch-logs.txt` ဖြစ်ပါသည်။ JSON နှင့် JSONL သည် တူညီသော log ဖိုင်အမည်များကို အသုံးပြုပါသည်။
Amcache နှင့် Jump Lists တို့တွင် ၎င်းတို့၏ prefix ပါသော table များဘေး၌ မျှဝေထားသော log တစ်ခုစီ ရှိပါသည်။ SRUM provider table များကို
`NetworkActivity`၊ `ProcessExecution` နှင့် `SystemInformation` တွင် တိုက်ရိုက် ထားပြီး မျှဝေထားသော
`SystemInformation/SRUM-logs.txt` ဖိုင် ရှိပါသည်။ `logs/` subdirectory များကို မဖန်တီးပါ။

မူလ CSV ဖိုင်အမည်များမှာ `Registry-SAM.csv`, `LNK.csv`, `Registry-RecentDocs.csv`, `Registry-ShimCache.csv`, `LogFile.csv`,
`I30.csv`, `MFT.csv`, `UsnJournal.csv`, `Boot.csv`, `SecureSDS.csv`, `Registry-Networks.csv`, `Registry-Persistence.csv`, `Registry-Services.csv`, `Registry-TaskCache.csv`, `Registry-BCD.csv`,
`Registry-SecuritySettings.csv`,
`PrefetchTimeline.csv`, `Prefetch.csv`, `Registry-UserAssist.csv`, `Registry-AppCompat.csv`, `Registry-Office.csv`, `Registry-FeatureUsage.csv`, `Registry-AppHistory.csv`, `Registry-MRU.csv`, `Registry-SystemInfo.csv`, `Registry-RemoteAccess.csv`, `Registry-OpenSaveMRU.csv`, `Registry-InstalledPrograms.csv`, `Registry-BAM.csv`, `Registry-MuiCache.csv`, `Registry-ShellBags.csv`, `Registry-USB.csv`, `Registry-SecuritySettings.csv`, `Registry-AuditPolicy.csv`, `Registry-Dump.csv` နှင့်
`DefenderQuarantine.csv` တို့ ဖြစ်ပါသည်။ Registry မှ ဆင်းသက်လာသော table များသည်
`Registry-ShimCache.csv`၊ `Registry-UserAssist.csv`၊ `Registry-BAM.csv` နှင့် `Registry-SystemInfo.csv` အပါအဝင် `Registry-` prefix ကို အသုံးပြုပါသည်။
ယေဘုယျ hive key များနှင့် value များသည် `Registry-Dump.csv` တွင် ရှိပါသည်။ Amcache table များမှာ
`FileExistence/Registry-Amcache-Devices.csv`၊ `Registry-Amcache-Files.csv`၊
`Registry-Amcache-Findings.csv` နှင့် `Registry-Amcache-Summary.csv` ဖြစ်ပါသည်။ Jump Lists သည်
`FileAndFolderOpening/JumpLists-AutomaticDestinations.csv` နှင့်
`JumpLists-CustomDestinations.csv` ကို အသုံးပြုပါသည်။ Amcache သို့မဟုတ် Jump Lists subdirectory များ မရှိပါ။
SRUM table များသည် ၎င်းတို့၏ အမျိုးအစားအောက်တွင် တိုက်ရိုက် `SRUM-` ဖိုင်အမည်များကို အသုံးပြုပါသည်၊ ဥပမာ
`NetworkActivity/SRUM-NetworkConnections.csv`။
အသုံးပြုသူ ဝင်ရောက်မှု မှတ်တမ်း table များသည် `LateralMovement` တွင် table တစ်ခုလျှင် ဖိုင်တစ်ခုစီ ရှိပါသည်၊ ဥပမာ
`UAL-Clients.csv` နှင့် `UAL-SystemIdentity.csv` တို့ကို `UAL-logs.txt` နှင့်အတူ ထားပါသည်။
[အသုံးပြုသူ ဝင်ရောက်မှု မှတ်တမ်း (UAL)](../artifacts/ual.md#automatic-parsing) ကို ကြည့်ပါ။
ဝဘ် ဘရောက်ဇာ table များသည် `BrowserActivity` တွင် ရှိပါသည်: `Browser-History.csv`၊ `Browser-Downloads.csv`၊
`Browser-Cookies.csv` နှင့် `Browser-FormHistory.csv` တို့ကို `Browser-logs.txt` နှင့်အတူ ထားပါသည်။
[ဝဘ် ဘရောက်ဇာများ](../artifacts/browser.md#automatic-parsing) ကို ကြည့်ပါ။
Internet Explorer နှင့် legacy Edge ၏ WebCache table များလည်း ထိုနေရာတွင် ရှိပါသည်: `WebCache-History.csv`၊ `WebCache-Downloads.csv`၊
`WebCache-Cookies.csv`၊ `WebCache-Entries.csv` နှင့် `WebCache-Containers.csv` တို့ကို `WebCache-logs.txt` နှင့်အတူ ထားပါသည်။
[Internet Explorer နှင့် legacy Edge](../artifacts/webcache.md#automatic-parsing) ကို ကြည့်ပါ။
[SRUM နေရာချထားမှု table အပြည့်အစုံ](../artifacts/srum.md#categorized-auto-parse-output) ကို ကြည့်ပါ။

Log များသည် `Registry-SAM-logs.txt`၊ `PrefetchTimeline-logs.txt`
နှင့် `Registry-UserAssist-logs.txt` ကဲ့သို့ စာလုံးကြီးဖြင့် စသော artifact အမည်နှင့် ကိုက်ညီပါသည်။ JSON/JSONL ဖိုင်များသည် တူညီသော stem များကို
အသုံးပြုပါသည်၊ ဥပမာ `MFT.jsonl`။ Query များအတွက် record ၏ `Artifact` identifier သည် `mft` ဖြစ်ပါသည်။ Table အများအပြားပါသော output များတွင်
၎င်းတို့၏ table များဘေး၌ `Registry-Amcache-logs.txt` သို့မဟုတ် `JumpLists-logs.txt` ရှိပါသည်။ မျှဝေထားသော SRUM log သည်
`SystemInformation` တွင် ရှိပါသည်။


`manifest.json` သည် မူလ ရှာဖွေတွေ့ရှိထားသော `Hostname`၊ တစ်မူထူးခြားသော output/record `Comp`
identity၊ hostname ၏ ဇစ်မြစ်၊ အထောက်အထား path များ၊ artifact `Category`၊ command line များ၊ ကြာချိန်၊ exit
code၊ ရလဒ် path၊ log path၊ တောင်းဆိုထားသော JSONL timeline နှင့် export error မည်သည်မဆို၊ Defender ထုတ်ယူမှုကို တောင်းဆိုခဲ့/မတောင်းဆိုခဲ့
နှင့် task တစ်ခုစီ
အောင်မြင်ခဲ့သည်၊ ကျော်ခဲ့သည် သို့မဟုတ် မအောင်မြင်ခဲ့သည်ကို မှတ်တမ်းတင်ပါသည်။

Auto-parse သည် absolute path ကို မရေးသားသောကြောင့် အထောက်အထား၊ ရလဒ်များ သို့မဟုတ် Juntei တို့သည် ခွဲခြမ်းစိတ်ဖြာရေး စနစ်ပေါ်တွင်
မည်သည့်နေရာ၌ ရှိသည်ကို မပြသဘဲ ရလဒ်များကို မျှဝေနိုင်ပါသည်။

- အထောက်အထား path များသည် အထောက်အထား ဖိုင်တွဲနှင့် နှိုင်းယှဉ်ထားသော relative path များ ဖြစ်ပါသည်။ Terminal သည် ကွန်ပျူတာတစ်ခုစီကို
  အထောက်အထားအတွင်းရှိ ၎င်း၏ ဖိုင်တွဲနှင့်အတူ ဖော်ပြပြီး (`HOST1: ./HOST1_F`) parser များသည် အထောက်အထား ဖိုင်တွဲတွင် လည်ပတ်ကာ
  ထိုကဲ့သို့ path များကို ရရှိပါသည်။ ရလဒ်များ၏ ရင်းမြစ် path column များ (`SourceFile`, `SourcePath`, …)၊ log များ
  နှင့် manifest များ၏ `EvidenceRoot` နှင့် `WindowsDirectories` သည် ဖိုင်တစ်ခုစီ အထောက်အထားအတွင်း မည်သည့်နေရာတွင်
  ရှိသည်ကို ပြသပါသည် (`./HOST1_F/C/Windows/Prefetch/CMD.EXE-6D6290C5.pf`)။
- Manifest များနှင့် log များ၏ ရလဒ် path များ (`Output`, `OutputFiles`, `Log`, `JSON-Timeline`၊ `Command` ရှိ
  output path များ) နှင့် Defender ၏ `ExtractedFile` တို့သည် output ဖိုင်တွဲနှင့် နှိုင်းယှဉ်ထားသော relative path များ ဖြစ်ပါသည်
  (`./HOST1/ProcessExecution/Prefetch.csv`၊ ၎င်းဘေးရှိ timeline အတွက် `../siem.jsonl`)။ `Command`
  သည် ပရိုဂရမ်အမည် (`juntei`) ဖြင့် စတင်ပါသည်။ ၎င်း၏ အထောက်အထား path များသည် အထောက်အထား ဖိုင်တွဲနှင့် နှိုင်းယှဉ်ထားပြီး
  output path များသည် output ဖိုင်တွဲနှင့် နှိုင်းယှဉ်ထားပါသည်။
- `EvidenceDirectory` နှင့် `OutputDirectory` တို့သည် ဖိုင်တွဲနှစ်ခု၏ အမည်များ ဖြစ်ပါသည်။ `ManifestVersion` 2
  သည် output ဖိုင်တွဲမှ ရလဒ် path များကို မှတ်တမ်းတင်သော manifest များကို အမှတ်အသားပြုပါသည်။ ယခင် manifest များတွင်
  `ManifestVersion` မရှိဘဲ ၎င်းတို့ကို `-o` သို့ ပေးထားသည့်အတိုင်း မှတ်တမ်းတင်ပါသည်။ `-C` rerun သည် နှစ်မျိုးလုံးကို ဖတ်ပါသည်။
- Terminal သည် output သို့မဟုတ် timeline path ကို relative ဖြစ်လျှင် ပေးထားသည့်အတိုင်း ပြသပြီး absolute ဖြစ်လျှင်
  လက်ရှိ ဖိုင်တွဲမှ ပြသပါသည် (`../results/`)။

Parallel process အရေအတွက်ပြီးနောက် terminal သည် တွေ့ရှိသော Windows စနစ်များ၏ ဖိုင်များ၏ စုစုပေါင်း size ကို
ပြသပါသည် (`Total filesize: 26.1 MiB`)။

Progress message သည် လက်ရှိ ကွန်ပျူတာနှင့် artifact ကို ခန့်မှန်း input size နှင့်အတူ ပြသပါသည်၊ ဥပမာ
`HOST1 / prefetch (~2.0 MiB)`။ ဖိုင်အုပ်စုတစ်ခုသည် ၎င်းတို့၏ ပေါင်းစပ် size ကို အသုံးပြုပါသည်။ MFT၊ LogFile သို့မဟုတ်
USN Journal task တစ်ခုစီသည် ၎င်း၏ တစ်ခုချင်း input size ကို ပြသပါသည်။ ခန့်မှန်းချက်များသည် အကူအညီ transaction log များနှင့်
ဖြည့်စွက်ဖိုင်များကို မပါဝင်ဘဲ parse လုပ်ထားသော output ၏ size ကို ကြိုမခန့်မှန်းပါ။

`manifest.csv` သည် တူညီသော task index ကို spreadsheet နှင့် အဆင်ပြေသော table အဖြစ် parser task တစ်ခုလျှင်
row တစ်ခုဖြင့် ပေးပါသည်။ ၎င်းတွင် `Comp`၊ မူလ `Hostname`၊ `FQDN`၊ `HostnameSource`၊ `EvidenceRoot`၊
`Artifact`, `Category`, `OutputType`, `Purpose`, `Status`, `DurationMs`, `ExitCode`, `Output`, `OutputFiles`, `Log`,
`Error` နှင့် `Command` ပါဝင်ပါသည်။ `Purpose` သည် အမျိုးအစားခွဲထားသော ရလဒ်များအတွက် `artifact` သို့မဟုတ် CSV နှင့်အတူ
အသုံးပြုသော ထပ်ဆောင်း native JSONL pass အတွက် `json_timeline` ဖြစ်ပါသည်။ `OutputType` သည် တကယ့် parser output ပုံစံ ဖြစ်ပါသည်။
`Command` သည် executable နှင့် ၎င်း၏ argument များ၏ JSON array ဖြစ်ပြီး space များနှင့် quoting ကို ထိန်းသိမ်းပါသည်။
SRUM အတွက် `Output` သည် ကွန်ပျူတာ ဖိုင်တွဲ ဖြစ်ပြီး `OutputFiles` သည် ၎င်း၏ တိကျသော provider output path များ၏
JSON array ဖြစ်ကာ `Category` သည် မျှဝေထားသော log တည်နေရာ (`SystemInformation`) ကို ဖော်ပြပါသည်။
ဖိုင်များကို တစ်ခုချင်းစီ အမျိုးအစားခွဲထားပြီး `OutputFiles` သည် JSON manifest တွင်လည်း ပါဝင်ပါသည်။

ပြန်လည်စစ်ဆေးရန် လိုအပ်သော task များကို ရှာရန် `Status` ကို `skipped` သို့မဟုတ် `failed` ဖြင့် စစ်ထုတ်ပါ။ ကျော်ခဲ့သော task ၏ `Error`
သည် ဗလာ ဖြစ်နိုင်ပြီး ရှင်းလင်းချက်ကို ၎င်း၏ `Log` ဖိုင်တွင် တွေ့နိုင်ပါသည်။

Terminal အနှစ်ချုပ်တွင် manifest နှစ်ခုလုံး၊ parser log များနှင့် ထုတ်ယူထားသော ဖိုင်များ အပါအဝင် output ဖိုင်တွဲအောက်တွင်
လက်ရှိရှိသော ပုံမှန်ဖိုင်များ၏ စုစုပေါင်း အရေအတွက်နှင့် size ပါဝင်ပါသည်။ Symbolic link များကို မလိုက်ပါ။ Size များကို
`Total filesize` နှင့် တူညီသော formatter ဖြင့် `KiB`၊ `MiB` နှင့် `GiB` ကဲ့သို့ ယူနစ်များဖြင့် အလိုအလျောက် ချိန်ညှိပါသည်။

```text
Results saved: output/ ( 125 files / 40.5 MiB )
```

ရင်းမြစ် database အများအပြားကို စုဆောင်းထားသည့်အခါ Amcache နှင့် SRUM table များသည် `-Source1`၊ `-Source2` စသည်တို့ကို
အသုံးပြုပါသည်။ မျှဝေထားသော parser log သည် မူလမှ နောက်ဆုံး path များကို မှတ်တမ်းတင်ပြီး JSON/JSONL
record များသည် ရင်းမြစ် provenance ကို ထိန်းထားပါသည်။ CSV Amcache/Jump Lists task များသည် ၎င်းတို့၏ တိကျသော table path များကို
manifest `OutputFiles` တွင် ဖော်ပြပြီး ၎င်းတို့၏ `Output` သည် စုံစမ်းစစ်ဆေးရေး အမျိုးအစား ဖိုင်တွဲ ဖြစ်ပါသည်။ JSON နှင့် JSONL
Amcache နှင့် Jump Lists export များသည် `Registry-Amcache.json[l]` နှင့်
`JumpLists.json[l]` အမည်ရှိ ဖိုင်တစ်ခုတည်းများ ဖြစ်ပြီးသား ဖြစ်ပါသည်။ ပေါင်းစပ်ထားသော `-j` timeline သည် တူညီသော record အကြောင်းအရာများနှင့် `Artifact` တန်ဖိုးများကို ထိန်းထားပါသည်။
`--clobber` သည် ယခင် အမည်များ အပါအဝင် မှတ်တမ်းတင်ထားသော output များကို ရှင်းလင်းရန် ယခင် manifest ကို အသုံးပြုပြီး
မသက်ဆိုင်သော အမျိုးအစား ဖိုင်များကို ထိန်းသိမ်းပါသည်။ ပုံမှန် volume တစ်ခုတည်း collection သည် `$MFT` နှင့် USN Journal record များကို
`FileSystem/MFT.csv` နှင့် `FileSystem/UsnJournal.csv` သို့ တိုက်ရိုက် ရေးသားပါသည်။ ထပ်ဆောင်း စုဆောင်းထားသော volume များသည်
`FileSystem/MFT-2.csv` နှင့် `FileSystem/MFT-2-logs.txt` ကဲ့သို့ နံပါတ်တပ်ထားသော ညီအစ်ကို ဖိုင်များကို အသုံးပြုပါသည်။

## JSON နှင့် SIEM ထည့်သွင်းခြင်း { #json-and-siem-ingestion }

JSON သို့မဟုတ် JSONL record တိုင်းသည် top-level field နှစ်ခုကို ရရှိပါသည်။

- `Comp`: ရင်းမြစ်စနစ်။
- `Artifact`: record ကို ထုတ်ပေးခဲ့သော Juntei parser၊ ဥပမာ `mft` သို့မဟုတ် `prefetch-timeline`။

`-j, --json-timeline <FILE>` သည် တစ်ခုချင်း ရလဒ်ပုံစံ မည်သို့ပင်ဖြစ်စေ ထပ်ဆောင်း JSONL ဖိုင်တစ်ခုကို ဖန်တီးပါသည်။
အမျိုးအစားခွဲထားသော output အားလုံးနှင့် ၎င်းတို့၏ log များကို ထိန်းထားပါသည်။

```bash
# Default CSV tables plus a SIEM timeline
juntei auto-parse -d evidence -o parsed -j parsed/timeline.jsonl

# Pretty JSON results plus a SIEM timeline
juntei auto-parse -d evidence -o parsed-json -t json -j parsed-json/timeline.jsonl

# Individual JSONL results plus a consolidated JSONL timeline
juntei auto-parse -d evidence -o parsed-jsonl -t jsonl -j parsed-jsonl/timeline.jsonl
```

ဖိုင်အမည်ကို ပေးထားသည့်အတိုင်း အသုံးပြုပါသည်။ Relative path များသည် လက်ရှိ working directory နှင့် နှိုင်းယှဉ်ထားပါသည်။
၎င်းသည် output root၊ အခြား reserve မလုပ်ထားသော output subdirectory သို့မဟုတ် output ဖိုင်တွဲ၏ အပြင်ဘက်တွင် ရှိနိုင်ပါသည်။
အထောက်အထား၊ manifest များ၊ ကွန်ပျူတာတစ်ခုချင်းစီ၏ ရလဒ်/log များနှင့် ယာယီ parser ဖိုင်တွဲများကို ဦးတည်ရာအဖြစ် အသုံးမပြုနိုင်ပါ။
ရှိပြီးသား ဖိုင်တစ်ခုအတွက် `-C` လိုအပ်ပါသည်။ မအောင်မြင်သော export သည် ၎င်းကို မထိခိုက်ဘဲ ထားပါသည်။

**CSV mode** တွင် Juntei သည် SIEM ဖိုင်က CSV table များ ချန်လှပ်နိုင်သော ဂဏန်းနှင့် boolean type များ၊ nested ဒေတာ၊ raw
field များနှင့် metadata ကို ထိန်းသိမ်းနိုင်ရန် ထပ်ဆောင်း native JSONL pass တစ်ခုကို လုပ်ဆောင်ပါသည်။ ထပ်ဆောင်း job များသည်
တူညီသော `-T` concurrency ကန့်သတ်ချက်ကို မျှဝေပြီး quarantine အကြောင်းအရာများကို ထပ်မံ မထုတ်ယူဘဲ သီးခြား
`*-json-timeline-logs.txt` sidecar log များ ရှိပါသည်။ Manifest နှစ်ခုလုံးသည် ၎င်းတို့၏ ရည်ရွယ်ချက်နှင့် output ပုံစံကို
ခွဲခြားပါသည်။ ယာယီ JSONL ဖိုင်များကို အောင်မြင်သော run ပြီးနောက် ဖယ်ရှားပြီး parser သို့မဟုတ် export မအောင်မြင်လျှင်
ထိန်းထားပါသည်။ ထပ်ဆောင်း လုပ်ဆောင်ချိန်နှင့် ယာယီ disk နေရာအတွက် စီစဉ်ထားပါ။

**JSON mode** တွင် Juntei သည် အထောက်အထားကို ပြန်လည် parse မလုပ်ဘဲ ရှိပြီးသား ရလဒ်များကို compact JSONL အဖြစ် stream လုပ်ပါသည်။
Amcache အစီရင်ခံစာများကို native JSONL ကဲ့သို့ တူညီသော တစ်ခုချင်း summary၊ finding၊ file နှင့် device record များအဖြစ်
ချဲ့ထွင်ပါသည်။ **JSONL mode** တွင် ရှိပြီးသား annotate လုပ်ထားသော record များကို တိုက်ရိုက် ပေါင်းစပ်ပါသည်။
Mode အားလုံးသည် တူညီသော SIEM record ဖွဲ့စည်းပုံကို အသုံးပြုပါသည်။ Dataset သည် native timestamp များနှင့် artifact
field များကို ထိန်းထားပြီး ဘုံ event timestamp တစ်ခုကို တီထွင်ခြင်း သို့မဟုတ် မတူညီသော artifact များကို တစ်ကမ္ဘာလုံးဆိုင်ရာ စီခြင်း မပြုလုပ်ပါ။

```bash
jq -c 'select(.Comp == "HOST1" and .Artifact == "prefetch")' parsed/timeline.jsonl
jq -r 'select(.Artifact == "mft") | .FullPath? // empty' parsed/timeline.jsonl
```

`manifest.json` သည် တောင်းဆိုထားသော `JSON-Timeline` path နှင့် `JSON-TimelineError` မည်သည်မဆိုကို မှတ်တမ်းတင်ပါသည်။ ၎င်း၏ task
record များနှင့် `manifest.csv` တို့သည် ထိန်းထားသော artifact output များကို ညွှန်ပြဆဲ ဖြစ်ပါသည်။ အောင်မြင်သော CSV timeline-only
job များသည် ဖျက်ထားသော ယာယီဖိုင်များအစား နောက်ဆုံး JSONL ဖိုင်ကို ကိုးကားပါသည်။ မည်သည့် parser မဆို မအောင်မြင်လျှင်
run သည် မအောင်မြင်စွာ ထွက်ပြီး timeline တွင် အောင်မြင်သော ရလဒ်များသာ ပါဝင်ပါသည်။ ၎င်းကို ပြည့်စုံသည်ဟု မမှတ်ယူမီ manifest များ
နှင့် အမျိုးအစား log များကို စစ်ဆေးပါ။ ဖိုင်အရေအတွက်/size အနှစ်ချုပ်သည် output ဖိုင်တွဲအတွင်းရှိ ဖိုင်များကို လွှမ်းခြုံပြီး
ပြင်ပ timeline ကို `JSONL timeline: <FILE>` အဖြစ် သီးခြား အစီရင်ခံပါသည်။

## စွမ်းဆောင်ရည် { #performance }

Parser job များသည် သီးခြား process များ ဖြစ်သောကြောင့် CPU အသုံးများသော artifact များကို တစ်ပြိုင်နက် လည်ပတ်နိုင်ပြီး
parser တစ်ခုရှိ ပျက်စီးမှုက run တစ်ခုလုံးကို မရပ်တန့်စေနိုင်ပါ။ မူလအားဖြင့် Juntei သည် logical CPU အရေအတွက်ကို အသုံးပြုပါသည်။
အထောက်အထားသည် နှေးကွေးသော disk ပေါ်တွင် ရှိသည့်အခါ သို့မဟုတ် စနစ်တွင် memory နည်းသည့်အခါ concurrency ကို ကန့်သတ်ပါ။

```bash
juntei auto-parse -d evidence -o parsed -T 4
```

## အသုံးပြုပုံ { #usage }

```text
Version: 1.1.0 (Dev Build)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei auto-parse -d <EVIDENCE> -o <OUTPUT> [OPTIONS]

Input:
  -d, --directory <EVIDENCE>  Evidence directory containing one or more collected Windows systems
      --max-depth <DEPTH>     Maximum directory depth at which to discover a Windows directory [default: 10]
      --vhdx                  Parse the files inside every VHDX disk image in the evidence directory, each mounted read-only with FUSE while it is parsed
      --e01                   Parse the files inside every E01 disk image in the evidence directory (split images from their first segment), each mounted read-only with FUSE while it is parsed

Output:
  -o, --output <DIR>                  New directory in which to create per-computer results
  -t, --output-type <TYPE>            Output type: csv (default), json, or jsonl [default: csv]
  -j, --json-timeline <FILE>          Create a single JSONL timeline to import into a SIEM
  -Z, --timezone <OFFSET>             Display parsed timestamps using a fixed UTC offset (UTC by default) [default: +00:00]
  -C, --clobber                       Overwrite outputs in an existing directory (never deletes unrelated files)
  -x, --extract-defender-quarantined  Extract Defender-quarantined files

Performance:
  -T, --threads <N>  Parser processes to run at once (default: logical CPU count)

Parsing:
      --code-page <NUMBER>  Windows code page for ANSI strings in LNK files, Jump Lists, ShellBags and OpenSaveMRU [default: 1252]
      --mft-slack           Recover the file names left in the slack of $MFT records

General Options:
  -h, --help  Show the help menu

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

ဖြစ်နိုင်သည့်အခါ output ဖိုင်တွဲအသစ်ကို အသုံးပြုပါ။ `-C` သည် ဤ run အတွက် ထုတ်လုပ်ထားသော တိကျသည့် target များကိုသာ
ဖယ်ရှားခြင်း သို့မဟုတ် ထပ်ရေးခြင်း ပြုလုပ်သောကြောင့် ယခု မရှိတော့သော artifact သည် ခေတ်မမီသော ရလဒ်ကို မချန်ထားနိုင်ပါ။ ၎င်းသည်
မသက်ဆိုင်သော ဖိုင်များ သို့မဟုတ် output tree တစ်ခုလုံးကို မဖျက်ပါ။ ၎င်းသည် parse မလုပ်မီ ကွန်ပျူတာတစ်ခုစီ၏
`SecurityProducts/DefenderQuarantined/` ဖိုင်တွဲနှင့် ယခင် flat layout မှ reserve လုပ်ထားသော `defender-quarantined/`
ဖိုင်တွဲကိုလည်း run အသစ်က `-x` ကို အသုံးပြုသည်ဖြစ်စေ မပြုသည်ဖြစ်စေ ဖယ်ရှားပါသည်။

ယခင် flat layout မှ ပြောင်းသည့်အခါ output ဖိုင်တွဲအသစ်ကို အသုံးပြုပါ။ ယခင် CSV/JSON ဖိုင်များကို
အလိုအလျောက် နေရာမရွှေ့ မဖျက်ပါ။

!!! danger "ထုတ်ယူထားသော Defender အကြောင်းအရာကို malware အဖြစ် သဘောထားပါ"
    `-x, --extract-defender-quarantined` သည် quarantine လုပ်ထားသော အကြောင်းအရာကို ကွန်ပျူတာတစ်ခုစီ၏
    `SecurityProducts/DefenderQuarantined/` ဖိုင်တွဲအောက်ရှိ disk သို့ ရေးသားပါသည်။ ပြန်လည်ရယူထားသော binary stream များသည် `.quarantined` ဖြင့် အဆုံးသတ်ပြီး Juntei
    သည် execute ခွင့်ကို ဖယ်ရှားပါသည်။ Binary security descriptor တစ်ခုစီသည် ပိုင်ဆိုင်မှု၊ ခွင့်ပြုချက်များနှင့် SDDL ပါဝင်သော ဖတ်ရှုနိုင်သည့် `.security_descriptor.txt`
    အစီရင်ခံစာကိုလည်း ရရှိပါသည်။ ပြန်လည်ရယူထားသော payload များကို ဘယ်တော့မှ execute မလုပ်ပါနှင့်။ သီးခြားခွဲထားသော
    ခွဲခြမ်းစိတ်ဖြာရေး စနစ်ကို အသုံးပြုပြီး antivirus သို့မဟုတ် EDR software က ပြန်လည်ရယူထားသော အကြောင်းအရာအပေါ် alert ပေးခြင်း သို့မဟုတ် ဖယ်ရှားခြင်းကို မျှော်လင့်ထားပါ။

အစီရင်ခံစာ field များနှင့် ၎င်းတို့၏ forensic ကန့်သတ်ချက်များအတွက် [ပြန်လည်ရယူထားသော security descriptor ကို ဖတ်ရှုခြင်း](../artifacts/defender-quarantine.md#reading-the-recovered-security-descriptor)
ကို ကြည့်ပါ။

Auto-parse သည် အသုံးပြုသူများ၏ remote ချိတ်ဆက်မှုများ (RDP၊ PuTTY၊ WinSCP နှင့် map လုပ်ထားသော drive များ) ကိုလည်း ဖတ်ပါသည်။
