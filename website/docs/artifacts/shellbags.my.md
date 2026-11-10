# ShellBags { #shellbags }

## ShellBags ဆိုသည်မှာ { #what-shellbags-are }

Explorer သည် အသုံးပြုသူ ဖွင့်သော ဖိုင်တွဲတစ်ခုစီကို မည်သို့ ပြသသည်ကို မှတ်သားထားသည်- ၎င်း၏ view၊ icon အရွယ်အစား၊
စီစဉ်ပုံနှင့် window နေရာ။ ဖိုင်တွဲတစ်ခု၏ ဆက်တင်များကို ပြန်လည်ရှာဖွေနိုင်ရန် ၎င်းသည် ဖိုင်တွဲ၏ ကိုယ်ပိုင်အမှတ်အသားကို
အသုံးပြုသူ၏ registry တွင် shell item များ၏ သစ်ပင်ပုံစံ (tree) အဖြစ် သိမ်းဆည်းထားသည်- link ၏ target ID list တွင်
ပါဝင်သော item များနှင့် တူညီသည်- drive တစ်ခု၊ ထို့နောက် ဖွင့်ခဲ့သော ဖိုင်တွဲအထိ ဆင်းသွားသော ဖိုင်တွဲတစ်ခုစီ။ ဤ tree များသည်
ShellBags ဖြစ်သည်။

၎င်းတို့သည် Explorer တွင်လည်းကောင်း၊ ဖွင့်ရန်နှင့် သိမ်းရန် dialog များတွင်လည်းကောင်း ဖွင့်ခဲ့သော ဖိုင်တွဲများကို
removable drive များ၊ network share များ၊ portable device များ (ဖုန်းများ၊ ကင်မရာများ)၊ ZIP archive များနှင့် FTP
server များပေါ်ရှိ ဖိုင်တွဲများ၊ Control Panel စာမျက်နှာများ အပါအဝင် လွှမ်းခြုံသည်။ ဖိုင်တွဲ၊ drive သို့မဟုတ် network share
မရှိတော့ပြီးနောက်တွင်လည်း ၎င်းတို့ ကျန်ရှိနေသည်။

<span class="evidence-strength">အကောင်းဆုံး ကောက်ချက်:</span> ဤအသုံးပြုသူ၏ Explorer က ဤဖိုင်တွဲ (သို့မဟုတ်
၎င်းအောက်ရှိ ဖိုင်တွဲတစ်ခု) ကို ပြသခဲ့သည်။ ShellBag တစ်ခုသည် ဖိုင်တွဲအတွင်း ပြုလုပ်ခဲ့သည့်အရာကို မပြသပါ၊ ဝင်ရောက်မှု
တိုင်းကိုလည်း ရက်စွဲမသတ်မှတ်ပေးပါ- အောက်ပါ အချိန်များကို ကြည့်ပါ။

## တည်နေရာ { #location }

```text
C:\Users\<user>\AppData\Local\Microsoft\Windows\UsrClass.dat
Local Settings\Software\Microsoft\Windows\Shell\BagMRU               (Windows Vista and later)
Wow6432Node\Local Settings\Software\Microsoft\Windows\Shell\BagMRU   (32-bit Explorer)

C:\Users\<user>\NTUSER.DAT
Software\Microsoft\Windows\Shell\BagMRU         (desktop and network items)
Software\Microsoft\Windows\ShellNoRoam\BagMRU   (Windows XP: local folders)
```

`BagMRU` key တစ်ခုတွင် နံပါတ်တပ်ထားသော တန်ဖိုးများ (တစ်ခုစီသည် shell item တစ်ခု) နှင့် ၎င်းတို့၏ အစီအစဉ်ကို ပေးသော
`MRUListEx` တန်ဖိုး (နောက်ဆုံးဖွင့်ခဲ့သည်ကို ပထမ) ပါဝင်သည်။ တန်ဖိုးတစ်ခုစီတွင် ၎င်းအောက်ရှိ item များအတွက် အမည်တူ subkey
တစ်ခု ရှိသောကြောင့် path တစ်ခုသည် `BagMRU` မှ တန်ဖိုးတစ်ခုအထိ ဆင်းသွားသော item များ ဖြစ်သည်။ Subkey ၏ `NodeSlot`
သည် ဖိုင်တွဲ၏ view ဆက်တင်များ ရှိရာ ၎င်း၏ `Bags\<N>` key ၏ နံပါတ် ဖြစ်သည်။

Profile တိုင်းအတွက် `UsrClass.dat` နှင့် `NTUSER.DAT` ကို ၎င်းတို့၏ transaction log များ (`.LOG1`, `.LOG2`) နှင့်အတူ
စုဆောင်းပြီး `NTUSER.DAT` အသုံးပြုသူ၏ SID ကို ပေးသော စနစ်၏ `SOFTWARE` hive ကိုလည်း စုဆောင်းပါ။

## မည်သည့်အချိန်က မည်သည့်အရာကို ဖော်ပြသနည်း { #which-times-say-what }

- **Shell item ၏ အချိန်များ** (`ItemModifiedUTC`, `ItemCreatedUTC`, `ItemAccessedUTC`) သည် Explorer က item ကို
  ရေးသားခဲ့ချိန်က ဖိုင်တွဲ၏ ကိုယ်ပိုင် ဖိုင်စနစ် အချိန်များ (FAT တိကျမှု- 2 စက္ကန့်) ဖြစ်သည်။ ၎င်းတို့သည် ဝင်ရောက်မှုကို
  မဟုတ်ဘဲ ဖိုင်တွဲကို ရက်စွဲသတ်မှတ်ပေးပြီး နောက်ပိုင်း ဝင်ရောက်မှုများတွင် အပ်ဒိတ်မလုပ်ပါ။
  FAT နာရီများတွင် သိမ်းဆည်းထားသော UTC offset မရှိပါ။ Legacy `UTC` ခေါင်းစဉ်များနှင့် ပြသထားသော `Z` suffix သည်
  ဖော်ပြပုံ သဘောတူညီချက်များသာ ဖြစ်ပြီး source time zone ၏ သက်သေ မဟုတ်ပါ။ `auto-parse --timezone` သည် ဤ field များကို
  ထိန်းသိမ်းထားသည်။ ၎င်းတို့ကို source စနစ်၏ အခြေအနေနှင့်အတူ အဓိပ္ပာယ်ဖော်ပါ။
- **Key တစ်ခု၏ last-write အချိန်** သည် ၎င်း၏ `MRUListEx` ကို ပြန်ရေးသည့်အခါ ပြောင်းလဲသည်၊ ဆိုလိုသည်မှာ ၎င်း၏ item
  တစ်ခုကို ဖွင့်ပြီး ရှေ့ဆုံးသို့ ရွှေ့သည့်အခါ ဖြစ်သည်။ ထို့ကြောင့် item ၏ တန်ဖိုး ပါဝင်သော key ၏ အချိန်
  (`ParentKeyLastWrittenUTC`) သည် **၎င်းသည် ထို key ရှိ နောက်ဆုံး item ဖြစ်သည့်အခါတွင်သာ** (`IsMostRecentInParent`
  true, `MruPosition` 0) item ကို နောက်ဆုံးဖွင့်ခဲ့သည့်အချိန် ဖြစ်သည်။ အခြား item များအတွက် ၎င်းသည် ညီအစ်ကို item
  တစ်ခုကို ဖွင့်ခဲ့သည့် နောက်ပိုင်းအချိန် ဖြစ်သည်။
- Item ၏ ကိုယ်ပိုင် key (`KeyLastWrittenUTC`) သည် ၎င်း၏ နောက်ဆုံး child ကို ဖွင့်ခဲ့ချိန်ကို ထိုနည်းတူ ရက်စွဲသတ်မှတ်ပေးသည်။

## စိစစ်သူများ သတိပြုရန် { #analyst-cautions }

- **မရှိခြင်းသည် အနည်းငယ်သာ သက်သေပြသည်။** Explorer သည် ဖိုင်တွဲ၏ view ကို သိမ်းဆည်းသည့်အခါ ShellBags ကို ရေးသည်။
  ခဏတာသာ ကြည့်ခဲ့သော ဖိုင်တွဲ သို့မဟုတ် အခြားနည်းဖြင့် (command line၊ script၊ ပရိုဂရမ်) ဖွင့်ခဲ့သော ဖိုင်တွဲတွင် မရှိနိုင်ပါ။
  အသုံးပြုသူက key များကို ဖျက်နိုင်သည်။
- **NTFS reference တစ်ခုသည် item ကို ရေးသားခဲ့ချိန်က ဖိုင်တွဲ၏ reference ဖြစ်သည်** (`MftEntry`, `MftSequence`)။
  တူညီသော ဖိုင်တွဲ ရှိနေဆဲ ဟုတ်/မဟုတ် သို့မဟုတ် အခြားဖိုင်က entry ကို ပြန်လည်အသုံးပြုခဲ့ခြင်း ရှိ/မရှိ သိရန် `$MFT` တွင်
  ရှာဖွေပါ။
- **Removable drive များ** ကို ၎င်းတို့၏ drive letter ဖြင့် အမည်ပေးထားသည်။ Letter သည် ထိုစဉ်က drive တွင် ရှိခဲ့သော letter
  ဖြစ်သည်။ Device အတွက် USB နှင့် mount လုပ်ခဲ့သော device မှတ်တမ်းနှင့် တိုက်ဆိုင်ပါ။
- **အမည်များသည် item မှ လာသည်။** နောက်ပိုင်းတွင် အမည်ပြောင်းခဲ့သော ဖိုင်တွဲသည် ဤနေရာတွင် ၎င်း၏ အမည်ဟောင်းကို
  ဆက်ထိန်းထားသည်။ Root နှင့် known ဖိုင်တွဲများကို forensic tool များက အများအားဖြင့် အမည်ပေးသည့်အတိုင်း (`This PC`,
  `Quick Access`, `Downloads`) အမည်ပေးထားသည်။ `Guid` သည် ၎င်းတို့၏ identifier ကို ပေးသည်။
- **ရှာဖွေမှုရလဒ်များ၊ library များနှင့် Control Panel စာမျက်နှာများ** သည် view များဖြစ်ပြီး disk ပေါ်ရှိ ဖိုင်တွဲများ မဟုတ်ပါ
  (`property_view`, `control_panel_item`, `control_panel_category`)။
- **မည်သည့် parser မျှ မသိသော item** သည် ၎င်း၏ byte များကို `DataRaw` တွင် `ItemType` `unknown` နှင့် `ItemClass`
  ရှိ ၎င်း၏ class byte တို့နှင့်အတူ ထိန်းသိမ်းထားသည်။ ပျက်စီးနေသော သို့မဟုတ် ပုံမှန်မဟုတ်သော item များကို `Problems` တွင်
  ဖော်ပြပြီး ၎င်းတို့၏ byte များကိုလည်း ထိန်းသိမ်းထားသည်။
- **Dirty hive** တစ်ခုသည် ၎င်း၏ log များ မပါလျှင် ၎င်း၏ နောက်ဆုံး item များ ပျောက်နေနိုင်သည်။ Juntei သည် `--no-logs`
  မဟုတ်လျှင် hive ဘေးရှိ log များကို replay လုပ်သည်။ `TransactionLogStatus` က မှတ်တမ်းတစ်ခု ၎င်းတို့မှ လာခြင်း ရှိ/မရှိကို
  ဖော်ပြသည်။

## Juntei ဖြင့် ခွဲခြမ်းစိတ်ဖြာခြင်း { #parse-with-juntei }

```text
Usage: juntei parse-registry shell-bags <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find UsrClass.dat and NTUSER.DAT hives
  -f, --file <FILE>      Parse one UsrClass.dat or NTUSER.DAT
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Parsing:
      --code-page <NUMBER>  Windows code page for ANSI names in shell items (default: 1252)

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry shell-bags -d evidence/ -o shellbags.csv
juntei parse-registry shell-bags -f UsrClass.dat -t jsonl \
  | jq 'select(.ItemType == "volume" and .GUID == "{F5FB2C77-0E2F-4A16-A381-3E560C68BC83}")'
```

Shell item များကို `parse-file lnk` ၏ shell item parser ဖြစ်သော ys-lnk-parser ဖြင့် decode လုပ်သည်။ အနှစ်ချုပ်သည်
၎င်းတို့၏ parent key ၏ last-write အချိန်အရ နောက်ဆုံးဖွင့်ခဲ့သော ဖိုင်တွဲများကို (အသစ်ဆုံးကို ပထမ) ၎င်းတို့၏
အသုံးပြုသူများနှင့်အတူ ဖော်ပြသည်။

## ထုတ်ယူမှု field များ { #output-fields }

| Field | အဓိပ္ပာယ်ဖော်ပုံ |
|---|---|
| `Host`, `User`, `UserSID`, `SourcePath`, `HiveType` | Host၊ profile ဖိုင်တွဲ၏ အမည်၊ အသုံးပြုသူ၏ SID (`UsrClass.dat` ၏ root key က ၎င်းကို အမည်ဖော်သည်၊ `NTUSER.DAT` အတွက် စနစ်၏ `ProfileList`)၊ hive နှင့် ၎င်း၏ အမျိုးအစား။ |
| `BagKey` | Hive ၏ root အောက်ရှိ item ၏ key (`…\BagMRU\4\3\0`)- ၎င်း၏ တန်ဖိုးသည် ၎င်းအထက်ရှိ key တွင် ရှိသည်။ |
| `MRU-Position`, `IsMostRecentInParent` | ၎င်း၏ parent key ၏ `MRUListEx` ရှိ နေရာ (0- နောက်ဆုံးဖွင့်ခဲ့သည်)။ စာရင်းတွင် မပါလျှင် ဗလာ။ |
| `NodeSlot` | ၎င်း၏ `Bags\<N>` key ၏ နံပါတ် (ဖိုင်တွဲ၏ view ဆက်တင်များ)။ |
| `Path` | ၎င်း၏ tree ထိပ်မှ ၎င်းအထိ item များကို `\` ဖြင့် ဆက်ထားသည်။ `[unknown]` သည် တန်ဖိုး ပျောက်နေသော item ကို ကိုယ်စားပြုသည်။ |
| `ItemType`, `ItemClass` | Item ၏ အမျိုးအစား (`root_folder`, `volume`, `directory`, `file`, `network_location`, `uri`, `ftp_folder`, `compressed_folder`, `portable_device`, `portable_device_folder`, `users_files_folder`, `property_view`, `control_panel_item`, `control_panel_category`, `cd_burn`, `unknown`) နှင့် ၎င်း၏ class byte။ |
| `ItemName`, `GUID` | ၎င်း၏ အမည် (ဖိုင်တွဲ၏ အမည်ရှည်) နှင့် root ဖိုင်တွဲ၊ Control Panel item သို့မဟုတ် delegate ဖိုင်တွဲ၏ class identifier။ |
| `ItemModifiedUTC`, `ItemCreatedUTC`, `ItemAccessedUTC` | **Shell item မှ-** item ကို ရေးသားခဲ့ချိန်က ဖိုင်တွဲ၏ အချိန်များ (FAT၊ 2 စက္ကန့် တိကျမှု)။ |
| `MFT-Entry`, `MFT-Sequence` | **Shell item မှ-** ဖိုင်တွဲ၏ NTFS ဖိုင် reference။ |
| `ExtensionBlocks` | Item တွင် ပါဝင်သော extension block အရေအတွက်။ |
| `KeyLastWrittenUTC` | **Registry မှ-** item ၏ ကိုယ်ပိုင် key ၏ နောက်ဆုံးရေးသားချိန်- ၎င်း၏ နောက်ဆုံး child ကို ဖွင့်ခဲ့ချိန်။ |
| `ParentKeyLastWrittenUTC` | **Registry မှ-** ၎င်း၏ တန်ဖိုး ပါဝင်သော key ၏ နောက်ဆုံးရေးသားချိန်- `IsMostRecentInParent` true ဖြစ်ပါက ၎င်းကို နောက်ဆုံးဖွင့်ခဲ့ချိန်။ |
| `DataRaw` | အပြည့်အဝ နားမလည်နိုင်သော သို့မဟုတ် ပြဿနာရှိသော item အတွက် တန်ဖိုး၏ byte များ (hex)။ |
| `TransactionLogStatus`, `LogsApplied` | Registry ပြန်လည်ရယူမှု အခြေအနေ။ |
| `Problems` | Key ၏ `MRUListEx` က အမည်ဖော်ထားသော်လည်း key တွင် မရှိသော တန်ဖိုး၊ စာရင်းက အမည်မဖော်သော တန်ဖိုး၊ ၎င်း၏ တန်ဖိုး မပါသော subkey၊ binary မဟုတ်သော တန်ဖိုး၊ အမျိုးအစားမှားသော `NodeSlot`၊ ပျက်စီးနေသော shell item list။ |

## စုံစမ်းစစ်ဆေးမှု ဗျူဟာ { #investigation-strategy }

1. Removable drive များ (Removable Drives `Guid` ပါသော `volume`)၊ network တည်နေရာများ၊ FTP ဖိုင်တွဲများနှင့်
   portable device များနှင့် ၎င်းတို့အောက်ရှိ ဖိုင်တွဲများကို စာရင်းပြုစုပါ။
2. စိတ်ဝင်စားဖွယ် ဖိုင်တွဲများကို ရှာဖွေပါ- အသုံးပြုသူ ရေးသားနိုင်သော staging နေရာများ (`Temp`, `ProgramData`,
   `Public`)၊ tool နှင့် archive ဖိုင်တွဲများ၊ အခြားအသုံးပြုသူများ၏ profile များနှင့် မရှိတော့သော ဖိုင်တွဲများ (`$MFT` ရှိ
   NTFS reference ကို စစ်ဆေးပါ)။
3. `IsMostRecentInParent` true ဖြစ်သော item များ၏ `ParentKeyLastWrittenUTC` ဖြင့်သာ ဝင်ရောက်မှုများကို
   ရက်စွဲသတ်မှတ်ပါ။ ကျန်အရာများကို ဖိုင်ဖွင့်ခြင်းများကို ရက်စွဲသတ်မှတ်ပေးသော LNK ဖိုင်များ၊ Jump Lists နှင့် RecentDocs
   တို့နှင့် ချိတ်ဆက်စစ်ဆေးပါ။
4. ဖျက်ပစ်ထားသော `BagMRU` key များနှင့် တန်ဖိုးများအတွက် `parse-registry read --recover-deleted` ကို အသုံးပြုပါ။

## နောက်ထပ် အသေးစိတ်များ { #further-details }

- [libyal libfwsi: Windows Shell Item format](https://github.com/libyal/libfwsi/blob/main/documentation/Windows%20Shell%20Item%20format.asciidoc)
- [libyal winreg-kb: Most recently used (BagMRU)](https://winreg-kb.readthedocs.io/en/latest/sources/explorer-keys/Most-recently-used.html)

`ProfileList` မှ SID ရှာဖွေခြင်းသည် syntax အရ မှန်ကန်သော တစ်မူထူးခြားသည့် SID တစ်ခု လိုအပ်သည်။ ရှင်းလင်းမှုမရှိသော
profile ဖိုင်တွဲ mapping များနှင့် `S-1-...bak` ကဲ့သို့သော backup key များသည် `UserSid` ကို မသိရှိသည့်အတိုင်း ထားသည်။
