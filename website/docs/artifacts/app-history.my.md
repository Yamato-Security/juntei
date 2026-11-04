# အပလီကေးရှင်း မှတ်တမ်း { #application-history }

## Tool များက ချန်ထားခဲ့သည့်အရာများ { #what-tools-leave-behind }

ပြင်ပ (third-party) tool အများအပြားသည် ဆက်တင်များနှင့် မှတ်တမ်းများကို အသုံးပြုသူ၏ `NTUSER.DAT` တွင် သိမ်းဆည်းထားသည်။
၎င်းတို့အနက် အများအပြားသည် တိုက်ခိုက်သူများ အသုံးပြုသော tool များ သို့မဟုတ် ဒေတာကို မယူဆောင်သွားမီ စုစည်းပြင်ဆင်ရန်
(stage) အသုံးပြုသော tool များ ဖြစ်သည်- Sysinternals tool များ၊ archiver များ၊ cloud sync client များနှင့် အဝေးမှ
ဝင်ရောက်ရေး tool များ။ Juntei သည် အောက်ပါတို့ကို ဖတ်သည်-

| အပလီကေးရှင်း | `HKCU\Software` အောက်ရှိ key | ပြသသည့်အရာ |
|---|---|---|
| Sysinternals | `Sysinternals\<tool>` | အသုံးပြုသူက လိုင်စင်ကို လက်ခံခဲ့သော (`EulaAccepted`) tool တစ်ခုစီ- PsExec၊ ProcDump၊ Sysmon၊ … |
| WinRAR | `WinRAR\ArcHistory`, `DialogEditHistory\ArcName`, `DialogEditHistory\ExtrPath` | ဖွင့်ခဲ့သော archive များ၊ ၎င်း၏ dialog များတွင် ရိုက်ထည့်ခဲ့သော archive အမည်များနှင့် ထုတ်ယူရာ ဖိုင်တွဲများ |
| 7-Zip | `7-Zip\Compression\ArcHistory`, `Extraction\PathHistory`, `FM\FolderHistory`, `FM\CopyHistory`, `FM\PanelPath0/1` | ဖန်တီးခဲ့သော archive များ၊ ထုတ်ယူရာ ဖိုင်တွဲများ၊ ၎င်း၏ file manager တွင် ကြည့်ရှုခဲ့ပြီး ကူးယူခဲ့သော ဖိုင်တွဲများ |
| WinZip | `Nico Mak Computing\WinZip\filemenu`, `extract`, `mru\archives` | ၎င်း၏ File menu ပေါ်ရှိ archive များ၊ ထုတ်ယူရာ ဖိုင်တွဲများ၊ မကြာသေးမီက archive များ |
| OneDrive | `Microsoft\OneDrive\Accounts\<account>` | အကောင့်တစ်ခုစီ၏ e-mail လိပ်စာ၊ local ဖိုင်တွဲနှင့် အမည်များ၊ sync လုပ်ထားသော library များ (`ScopeIdToMountPointPathCache`) နှင့် ဖိုင်တွဲများ (`Tenants\<tenant>`) |
| Adobe Acrobat နှင့် Reader | `Adobe\{Acrobat Reader,Adobe Acrobat}\<version>\AVGeneral\cRecentFiles`, `cRecentFolders` | နောက်ဆုံးဖွင့်ခဲ့သော PDF ဖိုင်များနှင့် ဖိုင်တွဲများ၊ ဖိုင်အမည်၊ အရွယ်အစားနှင့် ရက်စွဲများနှင့်အတူ |
| TeamViewer | `TeamViewer` (နှင့် ဗားရှင်းဟောင်း၏ `Version<N>`) | တန်ဖိုးအားလုံး- အကောင့်နှင့် အသုံးပြုသူအမည်များ၊ ဖိုင်လွှဲပြောင်းမှု စတင်ရာ ဖိုင်တွဲများ (`<partner ID>?<local>|<remote>`)၊ … |
| CCleaner | `Piriform\CCleaner` | တန်ဖိုးအားလုံး- ရှင်းလင်းရန် သတ်မှတ်ထားသည့်အရာ၊ `UpdateKey`၊ … |

<span class="evidence-strength">အကောင်းဆုံး ကောက်ချက်:</span> ဤအသုံးပြုသူသည် tool ကို run ခဲ့သည် (သို့မဟုတ်
၎င်း၏ လိုင်စင်ကို လက်ခံခဲ့သည်)၊ သို့မဟုတ် tool က ဤ archive နှင့် ဖိုင်တွဲများကို ဖွင့်ခဲ့/ဖန်တီးခဲ့/ထုတ်ယူခဲ့သည်၊
သို့မဟုတ် ဤ cloud အကောင့်ကို အသုံးပြုသူအတွက် စီစဉ်သတ်မှတ်ခဲ့သည်။ မှတ်တမ်းအများစုတွင် entry တစ်ခုချင်းစီအတွက် အချိန်မပါပါ-
နောက်ဆုံး entry တစ်ခုတည်းသာ ၎င်း၏ စာရင်း key ၏ last-write အချိန်ကို ရရှိသည်။

## တည်နေရာ { #location }

```text
C:\Users\<user>\NTUSER.DAT
  Software\Sysinternals\<tool>                         EulaAccepted
  Software\WinRAR\ArcHistory                           0, 1, … (0 the most recent)
  Software\7-Zip\Compression                           ArcHistory (REG_BINARY UTF-16 list)
  Software\Microsoft\OneDrive\Accounts\<account>       UserEmail, UserFolder, …
  Software\Adobe\Acrobat Reader\<version>\AVGeneral\cRecentFiles\c1, c2, …
```

အသုံးပြုသူတိုင်း၏ `NTUSER.DAT` ကို ၎င်း၏ transaction log များနှင့်အတူ စုဆောင်းပြီး အသုံးပြုသူများ၏ SID များအတွက်
စနစ်၏ `SOFTWARE` ကိုလည်း စုဆောင်းပါ။

## စိစစ်သူများ သတိပြုရန် { #analyst-cautions }

- **Sysinternals EULA လက်ခံမှု။** `FirstUsedUTC` သည် key ၏ တစ်ခုတည်းသော တန်ဖိုးမှာ `1` ဟု သတ်မှတ်ထားသော
  ဖတ်ရှုနိုင်သည့် `EulaAccepted` ဖြစ်သည့်အခါ key ၏ last-write အချိန်မှ ခန့်မှန်းထားသော ပထမဆုံးအသုံးပြုချိန် ဖြစ်သည်။
  ငြင်းပယ်ခဲ့သော သို့မဟုတ် ပျက်စီးနေသော EULA သည် ဤခန့်မှန်းချက်ကို မရရှိပါ။ Registry ဆက်တင်များကို ကြိုတင်ပြင်ဆင်ထားခြင်း၊
  ကူးယူခြင်း သို့မဟုတ် တည်းဖြတ်ခြင်းလည်း ပြုလုပ်နိုင်သည်- လုပ်ဆောင်ခဲ့ခြင်း (execution) နှင့် ၎င်း၏အချိန်ကို အခြား
  အထောက်အထားများဖြင့် အတည်ပြုပါ။
  ထိုနေရာတွင် ဆက်တင်များ သိမ်းဆည်းသော tool များ (Process Explorer၊ Autoruns၊ …) သည် key ကို နောက်ပိုင်းတွင်
  ပြန်ရေးသည်- ထိုအခါ `FirstUsedUTC` သည် ဗလာဖြစ်ပြီး `KeyLastWrittenUTC` သည် နောက်ဆုံး ဆက်တင်ပြောင်းလဲမှုသာ
  ဖြစ်သည်။ `-accepteula` ဖြင့် run သော tool သို့မဟုတ် အခြားအသုံးပြုသူအဖြစ် run သော tool (PsExec ၏ အဝေးဘက်သည်
  SYSTEM အဖြစ် run သည်) သည် ထိုအသုံးပြုသူ၏ key ကို ရေးသည်။
- **အချိန်မဟုတ်ဘဲ အစီအစဉ်နေရာများ။** စာရင်း entry များကို အစီအစဉ်တကျ စီထားသည် (`MruPosition` 0 သည် နောက်ဆုံး
  ဖြစ်သည်)၊ သို့သော် စာရင်း key ၏ နောက်ဆုံးရေးသားချိန်က နောက်ဆုံး entry ကိုသာ ရက်စွဲသတ်မှတ်ပေးသည်။ ပိုဟောင်းသော
  entry များကို မသိရှိရသော အချိန်တွင် စောစွာ အသုံးပြုခဲ့သည်။
- **ရိုက်ထည့်ထားသော အမည်များ။** WinRAR ၏ `ArcName` သည် archive တစ်ခု ဖန်တီးစဉ် ရိုက်ထည့်ခဲ့သော archive အမည်ဖြစ်သည်-
  မကြာခဏ ဖိုင်တွဲမပါသော ဖိုင်အမည် ဖြစ်သည်။
- **Adobe path များ** ကို Adobe ရေးသားသည့်အတိုင်း ထိန်းသိမ်းထားသည် (`/C/Users/alice/a.pdf` သည်
  `C:\Users\alice\a.pdf` ဖြစ်သည်)။ ၎င်း၏ `sDate` (`Details` တွင်) သည် PDF ရက်စွဲ string ဖြစ်ပြီး ရေးထားသည့်အတိုင်း
  ထိန်းသိမ်းထားသည်။
- **CCleaner ၏ `UpdateKey`** သည် time zone မပါသော local အချိန်ဖြစ်ပြီး ရေးထားသည့်အတိုင်း ထိန်းသိမ်းထားသည်။
- **TeamViewer။** ချိတ်ဆက်မှု log များသည် registry တန်ဖိုးများ မဟုတ်ဘဲ ဖိုင်များ (`Connections_incoming.txt`,
  `TeamViewer*_Logfile.log`) ဖြစ်သည်- ၎င်းတို့ကိုလည်း စုဆောင်းပါ။
- **မရှိခြင်း။** အသုံးပြုသူများနှင့် tool များသည် ဤမှတ်တမ်းများကို ရှင်းလင်းဖျက်ပစ်နိုင်သည် (CCleaner ကိုယ်တိုင်
  ဖျက်နိုင်သည်)။

## Juntei ဖြင့် ခွဲခြမ်းစိတ်ဖြာခြင်း { #parse-with-juntei }

```text
Usage: juntei parse-registry app-history <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find NTUSER.DAT hives
  -f, --file <FILE>      Parse one NTUSER.DAT hive
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry app-history -d evidence/ -o app-history.csv
juntei parse-registry app-history -f NTUSER.DAT -t jsonl \
  | jq 'select(.App == "Sysinternals" and .FirstUsed != null)'
```

အနှစ်ချုပ်တွင် နောက်ဆုံးရ Sysinternals ပထမဆုံးအသုံးပြုချိန် ခန့်မှန်းချက်များ၊ စာရင်းတစ်ခုစီက နောက်ဆုံးအသုံးပြုခဲ့သော
archive နှင့် ဖိုင်တွဲများ၊ OneDrive အကောင့်များကို ဖော်ပြသည်။

## ထုတ်ယူမှု field များ { #output-fields }

| Field | အဓိပ္ပာယ်ဖော်ပုံ |
|---|---|
| `Host`, `SourcePath` | Host နှင့် `NTUSER.DAT`။ |
| `User`, `UserSID` | Profile ဖိုင်တွဲ၏ အမည်နှင့် စနစ်၏ `ProfileList` မှ ၎င်း၏ SID။ |
| `App` | `Sysinternals`, `WinRAR`, `7-Zip`, `WinZip`, `OneDrive`, `Adobe Acrobat Reader`, `Adobe Acrobat`, `TeamViewer`, `CCleaner`။ |
| `Item` | Sysinternals tool၊ စာရင်း (`ArcHistory`, `ExtrPath`, `FolderHistory`, `filemenu`, `cRecentFiles`, …)၊ OneDrive အကောင့် သို့မဟုတ် ၎င်း၏ sync လုပ်ထားသော ဖိုင်တွဲများ၏ key သို့မဟုတ် တန်ဖိုး၏ အမည်။ |
| `MRU-Position` | ၎င်း၏ စာရင်းအတွင်း entry ၏ နေရာ၊ 0 သည် နောက်ဆုံး။ |
| `Path` | Archive၊ ဖိုင်တွဲ သို့မဟုတ် စာရွက်စာတမ်း။ |
| `Value` | `EulaAccepted`၊ OneDrive အကောင့်၏ e-mail လိပ်စာ သို့မဟုတ် sync လုပ်ထားသော library ၏ id၊ TeamViewer သို့မဟုတ် CCleaner တန်ဖိုး။ |
| `FirstUsedUTC` | EULA လက်ခံမှုမှ ခန့်မှန်းထားသော ပထမဆုံးအသုံးပြုချိန် (အထက်ပါ သတိပြုရန်များကို ကြည့်ပါ)။ |
| `Details` | OneDrive အကောင့်၏ `DisplayName`, `UserCID`, `ConfiguredTenantId`, …၊ Adobe စာရွက်စာတမ်း၏ `Version`, `tFileName`, `uFileSize`, `uPageCount`, `sDate`။ |
| `KeyPath`, `KeyLastWrittenUTC`, `ValueName` | မှတ်တမ်း၏ ရင်းမြစ်နှင့် ထို key နောက်ဆုံးပြောင်းလဲခဲ့သည့်အချိန်။ |
| `DataRaw` | Decode မလုပ်နိုင်သည့်အခါ တန်ဖိုး၏ byte များ (hex)။ |
| `TransactionLogStatus`, `LogsApplied` | Registry ပြန်လည်ရယူမှု အခြေအနေ။ |
| `Problems` | Decode မလုပ်နိုင်သော သို့မဟုတ် မဖတ်နိုင်သော ဒေတာ။ |

## စုံစမ်းစစ်ဆေးမှု ဗျူဟာ { #investigation-strategy }

1. သာမန်အသုံးပြုသူတစ်ဦး run လေ့မရှိသော Sysinternals tool များ (PsExec၊ ProcDump၊ PsLoggedOn၊ SDelete) နှင့်
   ၎င်းတို့၏ EULA timestamp များကို ရှာဖွေပါ။
2. ဖြစ်ရပ်ပတ်ဝန်းကျင်ရှိ archive အမည်များနှင့် ထုတ်ယူရာ ဖိုင်တွဲများကို ပြန်လည်သုံးသပ်ပါ- ဒေတာ စနစ်မှ မထွက်ခွာမီ
   `ProgramData`, `Temp` သို့မဟုတ် အသုံးပြုသူ၏ ဖိုင်တွဲတွင် ဖန်တီးခဲ့သော archive များ။
3. အဖွဲ့အစည်း၏ အကောင့်မဟုတ်သော OneDrive အကောင့်များနှင့် TeamViewer အကောင့်အမည်များ၊ ဖိုင်လွှဲပြောင်းမှု ဖိုင်တွဲများကို
   စစ်ဆေးပါ။
4. Prefetch၊ Amcache၊ BAM နှင့် UserAssist (tool run နေခြင်း)၊ `$MFT` နှင့် USN Journal (archive ဖိုင်)၊ LNK ဖိုင်များနှင့်
   Jump Lists (ဖွင့်ခဲ့သော စာရွက်စာတမ်းများ) နှင့် event log များနှင့် ချိတ်ဆက်စစ်ဆေးပါ။

## နောက်ထပ် အသေးစိတ်များ { #further-details }

- [Microsoft: Sysinternals](https://learn.microsoft.com/en-us/sysinternals/)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
