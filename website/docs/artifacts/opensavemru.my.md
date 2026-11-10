# ဖွင့်ရန်နှင့် အမည်သစ်ဖြင့် သိမ်းရန် dialog များ (OpenSaveMRU) { #open-and-save-as-dialogs-opensavemru }

## Dialog မှတ်တမ်း ဆိုသည်မှာ { #what-the-dialog-history-is }

ပရိုဂရမ်တစ်ခုက ဘုံ Open သို့မဟုတ် Save As dialog ကို ပြသသည့်အခါ Windows သည် အသုံးပြုသူ၏ `NTUSER.DAT` တွင်
အောက်ပါတို့ကို မှတ်တမ်းတင်ပါသည်။

- **မည်သည့်ဖိုင်ကို ဖွင့်ခဲ့သည် သို့မဟုတ် သိမ်းခဲ့သည်:** `OpenSavePidlMRU` တွင် extension တစ်ခုလျှင် subkey တစ်ခု
  (`docx`, `ps1`, …) နှင့် extension အားလုံးအတွက် `*` ရှိပြီး တစ်ခုစီသည် most-recently-used စာရင်း ဖြစ်ပါသည်။
- **မည်သည့်ပရိုဂရမ်က dialog ကို အသုံးပြုခဲ့ပြီး ၎င်းသည် မည်သည့်ဖိုင်တွဲတွင် ရှိခဲ့သည်:** `LastVisitedPidlMRU`
  (နှင့် dialog အဟောင်းများကို အသုံးပြုသော ပရိုဂရမ်များအတွက် `LastVisitedPidlMRULegacy`)၊
- **Dialog ကို ပြသခဲ့သော ပရိုဂရမ်များ:** `CIDSizeMRU` (ပရိုဂရမ်တစ်ခုချင်းစီအလိုက် dialog ၏ size) နှင့်
  `FirstFolder` (ပရိုဂရမ်၏ path နှင့် ဖိုင်တွဲတစ်ခု)၊
- Windows 2000 နှင့် XP တွင် `OpenSaveMRU` (extension တစ်ခုချင်းစီအလိုက် path များ) နှင့် `LastVisitedMRU` (ပရိုဂရမ်များနှင့်
  ၎င်းတို့၏ ဖိုင်တွဲများကို path များအဖြစ်)။

Windows Vista မှစ၍ ဖိုင်များနှင့် ဖိုင်တွဲများသည် shell item list များ ဖြစ်ပြီး shortcut ၏ target နှင့် ဖွဲ့စည်းပုံ
တူညီပါသည်။ item တစ်ခုစီသည် path ၏ အစိတ်အပိုင်းတစ်ခု (This PC၊ drive တစ်ခု၊ ဖိုင်တွဲတစ်ခု၊ ဖိုင်) ဖြစ်ပြီး ဖိုင် သို့မဟုတ်
ဖိုင်တွဲ item တစ်ခုသည် ၎င်း၏ ကိုယ်ပိုင်အချိန်များနှင့် NTFS file reference ကို သယ်ဆောင်နိုင်ပါသည်။

<span class="evidence-strength">အကောင်းဆုံး ကောက်ချက်:</span> ဤအသုံးပြုသူ၏ session တွင် ပရိုဂရမ်တစ်ခုသည် Open သို့မဟုတ်
Save As dialog ကို အသုံးပြု၍ ဤဖိုင်ကို ဖွင့်ခဲ့ သို့မဟုတ် သိမ်းခဲ့သည်၊ သို့မဟုတ် ဤဖိုင်တွဲတွင် အသုံးပြုခဲ့ပါသည်။ စာရင်းတစ်ခုစီ၏
နောက်ဆုံး entry ကိုသာ ရက်စွဲသတ်မှတ်နိုင်ပါသည်: ၎င်း၏ key ၏ နောက်ဆုံးရေးသားချိန် ဖြစ်ပါသည်။

## တည်နေရာ { #location }

```text
C:\Users\<user>\NTUSER.DAT
Software\Microsoft\Windows\CurrentVersion\Explorer\ComDlg32\
  OpenSavePidlMRU\<extension>   shell item lists, MRUListEx order (Vista and later)
  OpenSavePidlMRU\*             every extension
  LastVisitedPidlMRU            program name, then a shell item list
  LastVisitedPidlMRULegacy      the same
  CIDSizeMRU                    program name
  FirstFolder                   program path, then a folder path
  OpenSaveMRU\<extension>       paths, MRUList order (2000 and XP)
  LastVisitedMRU                program name, then a folder path (2000 and XP)
```

Profile တိုင်းအတွက် `NTUSER.DAT` ကို ၎င်း၏ transaction log များနှင့်အတူ စုဆောင်းပါ။

## စိစစ်သူများ သတိပြုရန် { #analyst-cautions }

- **စာရင်းတစ်ခုလျှင် အချိန်တစ်ခု။** Key ၏ နောက်ဆုံးရေးသားချိန်သည် ၎င်း၏ entry တစ်ခု စာရင်း၏ ရှေ့ဆုံးသို့ ရောက်သည့်အခါ
  ပြောင်းလဲသောကြောင့် ၎င်းသည် position 0 ရှိ entry ကိုသာ ရက်စွဲသတ်မှတ်ပေးပါသည် (`IsMostRecent`, `LastUsedUTC`)။
  အခြား entry များသည် စာရင်းအတိုင်း အသစ်ဆုံးမှ အဟောင်းဆုံးသို့ အစီအစဉ်ကျ ရှိသော်လည်း ရက်စွဲ မရှိပါ။
- **ဖိုင်တစ်ခုတည်း စာရင်းနှစ်ခုတွင်။** ဖိုင်တစ်ခုသည် ၎င်း၏ extension စာရင်းနှင့် `*` တွင် ရှိပါသည်။ ၎င်းကို အသုံးပြုမှု
  နှစ်ကြိမ်အဖြစ် မရေတွက်ပါနှင့်။ `*` သည် extension များအကြား အစီအစဉ်ကို ပေးပါသည်။
- **ဖွင့်ခဲ့ခြင်း သို့မဟုတ် သိမ်းခဲ့ခြင်း။** စာရင်းများက မည်သည့်တစ်ခု ဖြစ်သည်ကို မဖော်ပြပါ။ ဖိုင်၏ `$MFT` နှင့် USN
  Journal record များ၊ LNK ဖိုင်များနှင့် jump list များနှင့် ဆက်စပ်စစ်ဆေးပါ။
- **Item အချိန်များသည် ဖိုင်၏ အချိန်များဖြစ်ပြီး dialog ၏ အချိန်များ မဟုတ်ပါ။** `ItemModifiedUTC`၊ `ItemCreatedUTC` နှင့်
  `ItemAccessedUTC` တို့သည် item ကို ရေးသားခဲ့သည့်အချိန်က ဖိုင်၏ အချိန်များဖြစ်ပြီး 2 စက္ကန့် (FAT) အထိ တိကျပါသည်။ သိရှိထားသော
  ဖိုင်တွဲများ (`CLSID_ThisPCDocumentsRegFolder`, `Downloads`, …) ၏ item များတွင် မကြာခဏ အချိန် မရှိပါ။ ဤ FAT
  timestamp များတွင် timezone offset မရှိပါ။ `UTC` column suffix သည် ပြသပုံ စည်းမျဉ်းသာ ဖြစ်ပြီး မူလနာရီသည် UTC ဖြစ်ခဲ့ကြောင်း
  အထောက်အထား မဟုတ်ပါ။ Registry FILETIME များနှင့် နှိုင်းယှဉ်မီ ရင်းမြစ်စနစ်၏ timezone နှင့် အခြား timestamp များနှင့်
  ဆက်စပ်စစ်ဆေးပါ။ Microsoft ၏ [file-time လမ်းညွှန်](https://learn.microsoft.com/en-us/windows/win32/sysinfo/file-times) ကို ကြည့်ပါ။
- **Path များသည် shell path များ ဖြစ်ပါသည်။** ၎င်းတို့သည် desktop မှ စတင်ပါသည်: `This PC\C:\Users\…`၊ သိရှိထားသော
  ဖိုင်တွဲ၊ library (`UsersLibraries\Documents\…`) သို့မဟုတ် ကွန်ရက် location။ ဖိုင်တွဲ သို့မဟုတ် ဖိုင်ဖြင့် စတင်သော path သည်
  desktop ပေါ်တွင် ရှိပါသည်။ ဗလာ path သည် desktop ကိုယ်တိုင် ဖြစ်ပါသည်။
- **Decode မလုပ်ထားသော item များ။** Shell item parser က မသိသော type ၏ item ကို ၎င်း၏ class (`[class 0x00]`) ဖြင့်
  ပြသပြီး ၎င်း၏ type သည် `unknown` ဖြစ်ကာ စာရင်း၏ byte များသည် `DataRaw` တွင် ရှိပါသည်။
- **ပရိုဂရမ်များသည် အမည်များ ဖြစ်ပါသည်။** `LastVisitedPidlMRU` နှင့် `CIDSizeMRU` သည် path မဟုတ်ဘဲ ဖိုင်အမည်
  (`WINWORD.EXE`) ကို ထိန်းထားပါသည်။ `FirstFolder` သည် path ကို ထိန်းထားပါသည်။
- **စာရင်းများသည် တိုပြီး ရှင်းလင်းခံရပါသည်။** စာရင်းတစ်ခုစီသည် အကန့်အသတ်ရှိသော entry အရေအတွက်ကို ထိန်းထားပြီး
  cleanup tool များက key များကို ဖျက်ပါသည်။ ဖျက်ထားသော တန်ဖိုးများအတွက် `parse-registry read --recover-deleted` ကို အသုံးပြုပါ။

## Juntei ဖြင့် ခွဲခြမ်းစိတ်ဖြာခြင်း { #parse-with-juntei }

```text
Usage: juntei parse-registry open-save-mru <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find NTUSER.DAT hives
  -f, --file <FILE>      Parse one NTUSER.DAT
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs
      --code-page <CP>   Code page of shell item names that aren't Unicode (default 1252)

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry open-save-mru -d evidence/ -o opensavemru.csv
juntei parse-registry open-save-mru -f NTUSER.DAT -t jsonl \
  | jq 'select(.List == "OpenSavePidlMRU" and .Extension != "*")'
```

အနှစ်ချုပ်တွင် စာရင်းတစ်ခုချင်းစီအလိုက် entry များ၊ dialog များကို အသုံးပြုခဲ့သော ပရိုဂရမ်များနှင့် hive တစ်ခုစီ၏
နောက်ဆုံးဖိုင် (`*` ၏ ပထမ entry) ကို ဖော်ပြပါသည်။

## ထုတ်ယူမှု field များ { #output-fields }

| Field | အဓိပ္ပာယ်ဖော်ချက် |
|---|---|
| `Host`, `User`, `UserSID`, `SourcePath` | Host၊ profile ပိုင်ရှင် (path သို့မဟုတ် shell folder များမှ)၊ စနစ်၏ `ProfileList` မှ SID နှင့် hive။ |
| `List` | `OpenSavePidlMRU`, `LastVisitedPidlMRU`, `LastVisitedPidlMRULegacy`, `CIDSizeMRU`, `FirstFolder`, `OpenSaveMRU` သို့မဟုတ် `LastVisitedMRU`။ |
| `Extension` | Extension ၏ subkey (`docx`, `*`)။ အခြားစာရင်းများအတွက် ဗလာ။ |
| `MRU-Position`, `IsMostRecent` | Key ၏ စာရင်းအတွင်း နေရာ (0 သည် အသစ်ဆုံး)။ စာရင်းက တန်ဖိုးကို မဖော်ပြသည့်အခါ ဗလာ။ |
| `ValueName` | တန်ဖိုး၏ နံပါတ် (`MRUListEx`) သို့မဟုတ် စာလုံး (`MRUList`)။ |
| `Program` | Dialog ကို ပြသခဲ့သော ပရိုဂရမ်။ |
| `Path` | ဖိုင် သို့မဟုတ် ဖိုင်တွဲ: shell item များ၏ အမည်များကို ဆက်ထားခြင်း သို့မဟုတ် စာသားအဖြစ် သိမ်းဆည်းထားသော path။ |
| `ItemType` | နောက်ဆုံး shell item ၏ type: `file`, `directory`, `root_folder`, `volume`, `network_location`, `property_view`, `unknown`, … |
| `ItemModifiedUTC`, `ItemCreatedUTC`, `ItemAccessedUTC` | ၎င်း၏ shell item က မှတ်တမ်းတင်ခဲ့သည့်အတိုင်း ဖိုင် သို့မဟုတ် ဖိုင်တွဲ၏ အချိန်များ။ |
| `MFT-Entry`, `MFT-Sequence` | Shell item က မှတ်တမ်းတင်ခဲ့သော NTFS file reference။ |
| `LastUsedUTC` | နောက်ဆုံး entry အတွက်သာ key ၏ နောက်ဆုံးရေးသားချိန်။ |
| `KeyPath`, `KeyLastWrittenUTC` | Key နှင့် ၎င်း၏ နောက်ဆုံးရေးသားချိန်: row တိုင်း၏ အချိန်မဟုတ်ဘဲ ၎င်း၏ နောက်ဆုံး entry ၏ အချိန်။ |
| `DataRaw` | Decode မလုပ်နိုင်သည့်အခါ တန်ဖိုး၏ byte များ (hex)၊ သို့မဟုတ် item ၏ type မသိသည့်အခါ shell item list။ |
| `TransactionLogStatus`, `LogsApplied` | Registry ပြန်လည်ရယူမှု context။ |
| `Problems` | စာရင်းက ဖော်ပြသော်လည်း key တွင် မရှိသော တန်ဖိုးများ၊ စာရင်းက မဖော်ပြသော တန်ဖိုးများ၊ မှားယွင်းသော type များ၊ ဖြတ်တောက်ခံရသော အမည်များ သို့မဟုတ် စာရင်းများ။ |

## စုံစမ်းစစ်ဆေးမှု ဗျူဟာ { #investigation-strategy }

1. အသုံးပြုသူတစ်ဦးချင်းစီအလိုက် `*` ကို အစီအစဉ်အတိုင်း ဖတ်ပြီး နောက်ဆုံးဖိုင်အတွက် `LastUsedUTC` ကို ဖတ်ပါ။
2. အသုံးပြုသူ ရေးသားနိုင်သော၊ ဖယ်ရှားနိုင်သော သို့မဟုတ် ကွန်ရက် နေရာများရှိ script များ၊ archive များ၊ executable များနှင့်
   စာရွက်စာတမ်းများကို ရှာပြီး credential များ သို့မဟုတ် export များကဲ့သို့ အမည်ပေးထားသော ဖိုင်များကို ရှာပါ။
3. ပရိုဂရမ်တစ်ခုကို ၎င်း နောက်ဆုံးအသုံးပြုခဲ့သော ဖိုင်တွဲနှင့် ချိတ်ဆက်ရန် `LastVisitedPidlMRU` ကို အသုံးပြုပြီး dialog များကို
   အသုံးပြုခဲ့ဖူးသော ပရိုဂရမ်များအတွက် `CIDSizeMRU` နှင့် `FirstFolder` ကို အသုံးပြုပါ။
4. Path များနှင့် `MftEntry` ကို `$MFT`၊ USN Journal၊ LNK ဖိုင်များ၊ jump list များ၊ ShellBags နှင့် RecentDocs
   တို့နှင့် ဆက်စပ်စစ်ဆေးပါ။

## နောက်ထပ် အသေးစိတ်များ { #further-details }

- [libyal winreg-kb: Most recently used (MRU) key များ](https://github.com/libyal/winreg-kb/blob/main/docs/sources/explorer-keys/Most-recently-used.md)
- [libyal libfwsi: Windows Shell Item ဖိုင်ပုံစံ](https://github.com/libyal/libfwsi/blob/main/documentation/Windows%20Shell%20Item%20format.asciidoc)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
- [ys-lnk-parser](https://github.com/Yamato-Security/ys-lnk-parser)
