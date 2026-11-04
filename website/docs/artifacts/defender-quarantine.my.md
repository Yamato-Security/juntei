# Microsoft Defender သီးခြားခွဲထားသော ဒေတာ (Quarantine) { #microsoft-defender-quarantine }

## Quarantine တွင် ပါဝင်သည့်အရာ { #what-the-quarantine-contains }

Microsoft Defender quarantine သည် detection တစ်ခုနှင့် ပတ်သက်သော metadata နှင့် ၎င်း quarantine လုပ်ခဲ့သော
resource များ၏ encrypt လုပ်ထားသော အကြောင်းအရာကို ထိန်းသိမ်းထားသည်။ Juntei သည် Entries၊ ResourceData နှင့် Resources
record များကို ဆက်စပ်ပေးပြီး ခြိမ်းခြောက်မှု အမည်၊ quarantine အချိန်၊ မူလ logical နှင့် physical path များ၊ ဖိုင် metadata၊
ပိုင်ရှင်၊ download မူရင်းနေရာ `Zone.Identifier`၊ hash များနှင့် integrity ပြဿနာများကို အစီရင်ခံသည်။

ဤအထောက်အထားသည် detection/remediation ကို မူလက မှတ်တမ်းတင်ခဲ့သော event log များထက် ပိုကြာရှည်စွာ ကျန်ရှိနိုင်သည်။
၎င်းသည် သီးခြားခွဲထားသော reverse engineering အတွက် သံသယဖြစ်ဖွယ် byte အတိအကျကိုလည်း ထိန်းသိမ်းထားနိုင်သည်။

<span class="evidence-strength">အကောင်းဆုံး ကောက်ချက်:</span> Defender သည် အမည်ဖော်ပြထားသော ခြိမ်းခြောက်မှု/resource
အတွက် quarantine record ကို မှတ်တမ်းတင်ထားသော အချိန်တွင် ဖန်တီးခဲ့သည်။ ထုတ်ကုန်၏ အမည်တပ်ချက်သည် malware ၏
သီးခြားလွတ်လပ်သော သက်သေ မဟုတ်ပါ။ Quarantine တစ်ခုတည်းက ယခင်က လုပ်ဆောင်ခဲ့ကြောင်း သက်သေမပြပါ။

## တည်နေရာနှင့် စုဆောင်းခြင်း { #location-and-collection }

```text
C:\ProgramData\Microsoft\Windows Defender\Quarantine\
  Entries\{GUID}
  ResourceData\<prefix>\<SHA-1>
  Resources\<prefix>\<SHA-1>
```

`Quarantine` directory တစ်ခုလုံးကို စုဆောင်းပါ။ ကိုက်ညီသော content/record ဖိုင်များ မရှိသည့် entry သည် အသုံးဝင်သော
metadata ဖြစ်နေဆဲဖြစ်သော်လည်း ဖိုင် byte များ သို့မဟုတ် တွက်ချက်ထားသော hash များကို မထုတ်ပေးနိုင်ပါ။

## ဘေးကင်းရေးနှင့် စိစစ်သူများ သတိပြုရန် { #safety-and-analyst-cautions }

- ထုတ်ယူထားသော အကြောင်းအရာသည် အန္တရာယ်ရှိဖွယ် များသည်။ သီးခြားခွဲထားသော ခွဲခြမ်းစိတ်ဖြာရေး စနစ်ကို အသုံးပြုပြီး ၎င်းကို ဘယ်တော့မှ မ run ပါနှင့်။
- `ThreatName` သည် ထိုအချိန်က Defender ၏ signature/အမျိုးအစားခွဲခြားမှု ဖြစ်သည်။ False positive များနှင့် နောက်ပိုင်း
  အမျိုးအစား ပြန်လည်ခွဲခြားမှုများ ဖြစ်နိုင်သည်။
- Quarantine အချိန်သည် remediation/quarantine အချိန်ဖြစ်သည်—download၊ ဖန်တီးမှု သို့မဟုတ် လုပ်ဆောင်ချိန် ဖြစ်ချင်မှ ဖြစ်မည်။
- Quarantine လုပ်ထားသော ဖိုင်ကို လုပ်ဆောင်ခြင်းမပြုမီ သို့မဟုတ် run ပြီးနောက် detect လုပ်ခဲ့နိုင်သည်။
- မူလ path နှင့် Zone.Identifier သည် တန်ဖိုးရှိသော်လည်း မရှိနိုင်သလို တိုက်ခိုက်သူက ထိန်းချုပ်ထားနိုင်သည်။
- ထိန်းသိမ်းထားမှု/ရှင်းလင်းမှု၊ မပြည့်စုံသော စုဆောင်းမှု သို့မဟုတ် ပျက်စီးမှုကြောင့် မိဘမဲ့ (orphan) record များ ဖြစ်ပေါ်နိုင်သည်။
- Juntei သည် အရွယ်အစားများ၊ CRC များ၊ ID များနှင့် hash များကို စစ်ဆေးသည်။ ပြန်လည်ရယူထားသော byte များကို မယုံကြည်မီ `Problems` ကို ပြန်လည်သုံးသပ်ပါ။

## Juntei ဖြင့် ခွဲခြမ်းစိတ်ဖြာခြင်း { #parse-with-juntei }

```text
Usage: juntei parse-file defender-quarantine <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find quarantine folders
  -f, --file <FILE>      Parse one Entries\{GUID} file

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
      --extract <DIR>       Extract dangerous content with a .quarantined extension
  -C, --clobber             Overwrite confirmed output targets
```

```bash
juntei parse-file defender-quarantine -d evidence/ -o quarantine.csv
juntei parse-file defender-quarantine -d evidence/ -t jsonl \
  | jq '{QuarantineTime, ThreatName, Path, SHA256: .Content.SHA256, Problems}'

# Only in an isolated malware-analysis environment
juntei parse-file defender-quarantine -d evidence/ -o quarantine.csv --extract extracted/
```

ဥပမာ `tool.exe` သည် `tool.exe.quarantined` ဖြစ်လာသည်။ အရန် stream များသည် ၎င်းတို့၏ အမျိုးအစားကို အမည်တွင်
ထည့်သွင်းထားသည်၊ ဥပမာ `tool.exe.Zone.Identifier.quarantined`။ Juntei သည် execute ခွင့်ပြုချက်ကိုလည်း ဖယ်ရှားသည်။
Extension နှင့် ခွင့်ပြုချက်များက မတော်တဆ run မိခြင်းကို လျှော့ချပေးသော်လည်း ပြန်လည်ရယူထားသော byte များကို malware
အဖြစ် ဆက်လက် ဆက်ဆံရမည်။

## ထုတ်ယူမှု field များ { #output-fields }

| Field | အဓိပ္ပာယ်ဖော်ပုံ |
|---|---|
| `Record` | `resource`၊ resource မပါသော entry၊ `orphan-content` သို့မဟုတ် `orphan-record`။ Orphan များက မပြည့်စုံသော ချိတ်ဆက်မှု/စုဆောင်းမှုကို ဖော်ပြသည်။ |
| `QuarantineTime` | UTC ဖြင့် Defender quarantine/remediation အချိန်၊ လုပ်ဆောင်ချိန် မဟုတ်ပါ။ |
| `ThreatName`, `ThreatID` | Defender အမျိုးအစားခွဲခြားမှုနှင့် နံပါတ် ID။ Vendor ၏ စီရင်ချက်/အခြေအနေအဖြစ် ဆက်ဆံပါ။ |
| `ResourceType` | ဖိုင်၊ registry key၊ ဝန်ဆောင်မှု၊ startup item၊ scheduled task သို့မဟုတ် အခြား resource အမျိုးအစား။ |
| `Path`, `PhysicalPath` | မူလ logical/physical တည်နေရာ။ အဓိက ချိတ်ဆက်စစ်ဆေးရေး pivot။ စာလုံးပေါင်း အတိအကျကို ထိန်းသိမ်းပါ။ |
| `FileSize` | Resource metadata က မှတ်တမ်းတင်ထားသော အရွယ်အစား။ `ContentSize` နှင့် MFT တို့နှင့် နှိုင်းယှဉ်ပါ။ |
| `CreationTime`, `LastAccessTime`, `LastWriteTime` | Quarantine လုပ်ထားသော resource အတွက် ဖမ်းယူထားသော metadata။ Quarantine အချိန်နှင့် သီးခြားဖြစ်ပြီး timestomping ခံရနိုင်သည်။ |
| `Attr` | မှတ်တမ်းတင်ထားသော ဖိုင် attribute များ။ Hidden/system အခြေအနေသည် အသုံးဝင်သော အခြေအနေ ဖြစ်နိုင်သည်။ |
| `Owner` | ပြန်လည်ရယူထားသော security descriptor မှ SID။ ပိုင်ဆိုင်မှုသည် လုပ်ဆောင်ခဲ့သော အသုံးပြုသူ ဖြစ်ချင်မှ ဖြစ်မည်။ |
| `ZoneID`, `ReferrerURL`, `HostURL` | `Zone.Identifier` မှ download-zone နှင့် URL value များ။ ရှိပါက ခိုင်မာသော မူရင်းနေရာ သဲလွန်စများ ဖြစ်သည်။ ADS မရှိနိုင်/ဖယ်ရှားခံရနိုင်သည်။ |
| `ContentSize` | အမှန်တကယ် ပြန်လည်ရယူထားသော အဓိက အကြောင်းအရာ၏ အရွယ်အစား။ မှတ်တမ်းတင်ထားသော `FileSize` နှင့် နှိုင်းယှဉ်ပါ။ |
| `MD5`, `SHA1`, `SHA256` | ပြန်လည်ရယူထားသော byte များမှ တွက်ချက်ထားသည်။ ခေတ်သစ် အကြောင်းအရာ ခွဲခြားသတ်မှတ်ရန် SHA-256 ကို အသုံးပြုပါ။ အပြန်အလှန် အသုံးပြုနိုင်ရန် အားလုံးကို ထိန်းသိမ်းပါ။ |
| `Streams` | ပြန်လည်ရယူထားသော stream အမျိုးအစား/အမည်/အရွယ်အစားများ။ ADS၊ security ဒေတာ သို့မဟုတ် registry hive ပါဝင်နိုင်သည်။ |
| `ResourceID` | Resource metadata/content record များကို ချိတ်ဆက်ပေးသော identifier။ |
| `EntryID` | Quarantine entry GUID။ Defender history/event အထောက်အထားများနှင့်လည်း ချိတ်ဆက်ပေးသည်။ |
| `ScanID` | Resource/action များကို အုပ်စုဖွဲ့ရန် Defender scan identifier။ |
| `QuarantineFolder` | Root source ဖိုင်တွဲ/host ခွဲခြားမှု။ |
| `EntryFile`, `ContentFile`, `ResourceRecordFile` | Row ကို တည်ဆောက်ရန် အသုံးပြုခဲ့သော source ဖိုင်များ အတိအကျ။ မှန်ကန်မှု စစ်ဆေးရန် မရှိမဖြစ် လိုအပ်သည်။ |
| `ExtractedFile` | `--extract` က ရေးသားခဲ့သော အဓိက quarantine ဖိုင်။ အဓိက ထုတ်ယူမှု မဖြစ်ပွားပါက ဗလာ။ အရန် stream များကို ၎င်းဘေးတွင် ရေးသားသည်။ `auto-parse -x` သည် ၎င်းကို ၎င်း၏ output directory မှ စတင်၍ ဖော်ပြသည် (`./HOST1/SecurityProducts/DefenderQuarantined/…`)။ |
| `Problems` | Integrity မကိုက်ညီမှု၊ ပျောက်နေသော အကြောင်းအရာ၊ ID ကွဲလွဲမှု၊ ပျက်စီးမှု သို့မဟုတ် orphan အခြေအနေ။ ဖုံးကွယ်မထားပါနှင့်။ |

JSON/JSONL သည် CSV ၏ ပေါင်းထားသော စာသားအစား ဖွဲ့စည်းထားသော resource၊ stream စာရင်း၊ record entry များနှင့် ဆက်နွယ်မှု
အသေးစိတ် အားလုံးကို ထိန်းသိမ်းထားသည်။

## ထုတ်ယူမှု အပြုအမူ { #extraction-behavior }

Quarantine entry က မူလဖိုင်အမည်ကို မှတ်တမ်းတင်ထားပါက Juntei သည် ၎င်းကို နောက်ဆုံး `.quarantined` extension ဖြင့်
အသုံးပြုသည်။ Orphan ဖြစ်နေသော သို့မဟုတ် ဖိုင်မဟုတ်သော resource များအတွက် content identifier ကို အသုံးပြုပြီး ဖိုင်အမည်
ထပ်တူဖြစ်မှုများကို ဖြေရှင်းရန် identifier ကို ထည့်ပေါင်းသည်။ Windows တွင် ခွင့်မပြုသော ဖိုင်အမည် စာလုံးများကို အစားထိုးသည်။
Security နှင့် alternate stream များကို အဓိက ဖိုင်ဘေးတွင် ရေးသားသည်။ Security stream တစ်ခုစီသည် ဖတ်ရှုနိုင်သော စာသား
အစီရင်ခံစာတစ်ခုကိုလည်း ရရှိသည်။ Juntei သည် execute ခွင့်ပြုချက်ကို ဖယ်ရှားသည်။
ရှိပြီးသား ကွဲပြားသော အကြောင်းအရာကို `-C` မပါဘဲ overwrite မလုပ်ပါ။ ဤထိန်းချုပ်မှုများက မတော်တဆမှုများကို လျှော့ချပေးသော်လည်း
byte များကို ဘေးကင်းအောင် မပြုလုပ်ပေးပါ။

### ပြန်လည်ရယူထားသော security descriptor ကို ဖတ်ရှုခြင်း { #reading-the-recovered-security-descriptor }

`.security.quarantined` ဖိုင်တွင် binary Windows security metadata ပါဝင်သောကြောင့် text editor ဖြင့် ဖတ်၍ မရပါ။
၎င်းသည် သီးခြား backup stream ဖြစ်ပြီး executable ၏ နောက်ထပ်မိတ္တူ မဟုတ်ပါ။ Juntei သည် `parse-file defender-quarantine --extract`
သို့မဟုတ် `auto-parse -x` ဖြင့် ထုတ်ယူသည့်အခါ ၎င်းကို အလိုအလျောက် decode လုပ်သည်-

```text
tool.exe.quarantined              Original file content; treat as malware
tool.exe.security.quarantined     Original binary security descriptor, preserved unchanged
tool.exe.security_descriptor.txt  Readable ownership and permission report
```

စာသားဖိုင်အမည်သည် မည်သည့် collision identifier မဆို အပါအဝင် အမှန်တကယ် binary အမည်အတိုင်း လိုက်နာသည်။ ထုတ်ပေးထားသော
စာသားသည် သာမန် UTF-8 ဖြစ်ပြီး အခြား ထုတ်ယူထားသော ဖိုင်များကဲ့သို့ overwrite ကာကွယ်မှုများကို အသုံးပြုသည်။ အစီရင်ခံစာကို
၎င်း၏ source နှင့် တိုက်ဆိုင်စစ်ဆေးနိုင်ရန် binary ဖိုင်အမည်၊ byte အရေအတွက်နှင့် SHA-256 ပါဝင်သည်။

| အစီရင်ခံစာ field | အဓိပ္ပာယ်ဖော်ပုံ |
|---|---|
| ပိုင်ရှင်နှင့် primary group | Descriptor တွင် မှတ်တမ်းတင်ထားသော SID များ။ လူသိများသော SID များသည် `BUILTIN\Administrators` သို့မဟုတ် `NT AUTHORITY\SYSTEM` ကဲ့သို့သော အမည်များ ရရှိသည်။ အခြား SID များကို မဖြေရှင်းဘဲ ထားသည်။ ရင်းနှီးသော RID တစ်ခုတည်းဖြင့် local/domain အကောင့်ကို အမည်ပေးရန် မလုံလောက်ပါ။ ပိုင်ဆိုင်မှုက ဖိုင်ကို မည်သူ ဖန်တီးခဲ့သည် သို့မဟုတ် run ခဲ့သည်ကို သက်သေမပြပါ။ |
| Control flag များ | ACL ရှိမှု၊ inheritance နှင့် protection အပါအဝင် မူလ နံပါတ် flag များနှင့် ၎င်းတို့၏ အမည်များ။ |
| DACL | အစီအစဉ်တကျ စီထားသော access-control entry (ACE) များ- allow/deny အမျိုးအစား၊ အကောင့် SID၊ permission mask၊ အခွင့်အရေးများနှင့် inheritance flag များ။ ၎င်းတို့သည် မှတ်တမ်းတင်ထားသော entry များဖြစ်ပြီး အသုံးပြုသူ၏ token အတွက် ထိရောက်သော ဝင်ရောက်ခွင့် တွက်ချက်မှု မဟုတ်ပါ။ |
| SACL | မှတ်တမ်းတင်ထားသော audit သို့မဟုတ် mandatory-integrity entry များ။ Audit-success/failure flag များသည် ပြင်ဆင်သတ်မှတ်ချက်ကို ဖော်ပြပြီး ဝင်ရောက်မှု သမိုင်းကြောင်း မဟုတ်ပါ။ SACL မရှိခြင်းက audit အားလုံးကို ပိတ်ထားကြောင်း မသက်သေပြပါ။ |
| Object GUID များ | Object-specific ACE များအတွက် ရွေးချယ်နိုင်သော GUID များ။ Object-specific အခွင့်အရေးများကို ဖိုင်အခွင့်အရေးများအဖြစ် မှားယွင်း အမည်တပ်မည့်အစား နံပါတ်အဖြစ် ထိန်းသိမ်းထားသည်။ |
| SDDL | ပိုင်ရှင်၊ group နှင့် ပံ့ပိုးထားသော ACL entry များ၏ Security Descriptor Definition Language ဖော်ပြချက်။ Control flag များကို သီးခြား ကြည့်ပါ။ အစိတ်အပိုင်းတစ်ခုခု ပျက်စီးနေလျှင် သို့မဟုတ် ပံ့ပိုးမထားလျှင် Juntei သည် တစ်စိတ်တစ်ပိုင်း SDDL string ကို မထုတ်ပေးပါ။ |
| Decode အခြေအနေနှင့် ပြဿနာများ | အကြောင်းရင်းများနှင့်အတူ `complete` သို့မဟုတ် `incomplete`။ ပံ့ပိုးမထားသော ACE များသည် အစီရင်ခံစာတွင် ၎င်းတို့၏ raw hex ကို ထိန်းသိမ်းထားသည်။ ထုတ်ယူမှု အောင်မြင်သည့်အခါ မူလ binary ကို အမြဲ ထိန်းသိမ်းထားသည်။ Decode ပြဿနာများသည် quarantine record ၏ `Problems` field တွင်လည်း ပေါ်လာသည်။ |

DACL မရှိခြင်း၊ NULL DACL နှင့် ဗလာ DACL ကို သီးခြားစီ အစီရင်ခံသည်။ DACL မရှိခြင်း သို့မဟုတ် NULL DACL သည်
discretionary ဝင်ရောက်ခွင့် ကန့်သတ်ချက် မပေးပါ။ ဗလာ DACL တွင် ဝင်ရောက်ခွင့်ပေးသော ACE မရှိပါ။ အခြား Windows ဝင်ရောက်ခွင့်
စစ်ဆေးမှုများနှင့် privilege များသည် အရေးပါဆဲ ဖြစ်သည်။ အမွေဆက်ခံထားသော နှင့် inherit-only ACE များကို ရှင်းလင်းစွာ အမှတ်အသားပြုထားဆဲ ဖြစ်သည်။

ဥပမာ Administrators ကို အမွေဆက်ခံထားသော full control ပေးသည့် entry တစ်ခုသည် အောက်ပါအတိုင်း ပေါ်နိုင်သည်-

```text
Entry 1: ALLOW (type 0x00)
  Flags: 0x10 (INHERITED_ACE)
  Account: S-1-5-32-544 (BUILTIN\Administrators)
  Access mask: 0x001F01FF
  Rights: Full control (FILE_ALL_ACCESS)
```

Decoder သည် Windows API များ၊ Python သို့မဟုတ် အကောင့်ရှာဖွေမှု မလိုဘဲ Windows၊ macOS နှင့် Linux ပေါ်တွင် offline run သည်။
၎င်းသည် အခြေခံ allow/deny/audit/alarm ACE များ၊ object ACE များနှင့် mandatory-integrity label များကို decode လုပ်သည်။ Conditional၊
callback၊ resource-attribute နှင့် အခြား ပံ့ပိုးမထားသော ACE များကို ရှင်းလင်းစွာ ခွဲခြားသတ်မှတ်ပြီး raw ပုံစံဖြင့်
ထိန်းသိမ်းထားသည်။ ၎င်းတို့၏ ဝင်ရောက်ခွင့် အခြေအနေများကို အကဲမဖြတ်ပါ။ ပျက်စီးနေသော descriptor တစ်ခုသည် အခြား stream များ၏
ထုတ်ယူမှုကို ရပ်တန့်မစေပါ။

## စုံစမ်းစစ်ဆေးမှု ဗျူဟာ { #investigation-strategy }

1. `ThreatName`၊ `Path`၊ `QuarantineTime`၊ `EntryId` နှင့် hash များမှ စတင်ပါ။
2. `Problems` ကို စစ်ဆေးပြီး မှတ်တမ်းတင်ထားသော အရွယ်အစားကို ပြန်လည်ရယူထားသော အကြောင်းအရာနှင့် နှိုင်းယှဉ်ပါ။
3. SHA-256 ကို အခြား host များ၊ EDR၊ sandbox နှင့် threat-intelligence ဒေတာများသို့ pivot လုပ်ပါ။
4. မူလ path/အချိန်ကို `$MFT`၊ USN Journal၊ Prefetch၊ Amcache နှင့် event log များနှင့် ချိတ်ဆက်စစ်ဆေးပါ။
5. ကနဦးဝင်ရောက်မှုကို စုံစမ်းရန် URL/zone field များကို အသုံးပြုသော်လည်း browser/proxy/DNS အထောက်အထားများဖြင့် အတည်ပြုပါ။

## နောက်ထပ် အသေးစိတ်များ { #further-details }

- [ys-defender-history-parser](https://github.com/Yamato-Security/ys-defender-history-parser)
- [Sootmark defender](https://github.com/Sootmark/defender)
- [dissect.target Defender plugin](https://docs.dissect.tools/en/latest/projects/dissect.target/plugins/os/windows/defender.html)
- [Microsoft: Microsoft Defender Antivirus remediation and quarantine](https://learn.microsoft.com/en-us/defender-endpoint/restore-quarantined-files-microsoft-defender-antivirus)
- [defender-dump](https://github.com/knez/defender-dump)
- [Microsoft: Security descriptors](https://learn.microsoft.com/en-us/windows/win32/secauthz/security-descriptors)
- [Microsoft: Security Descriptor String Format](https://learn.microsoft.com/en-us/windows/win32/secauthz/security-descriptor-string-format)
- [Microsoft: ACE strings](https://learn.microsoft.com/en-us/windows/win32/secauthz/ace-strings)
