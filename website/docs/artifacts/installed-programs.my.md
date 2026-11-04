# ထည့်သွင်းထားသော ပရိုဂရမ်များ { #installed-programs }

## Windows က သိမ်းဆည်းထားသည့်အရာများ { #what-windows-keeps }

Windows နှင့် installer များသည် ထည့်သွင်းထားသော software များကို registry စာရင်းအများအပြားတွင် မှတ်တမ်းတင်သည်။
စာရင်းတစ်ခုစီတွင် ပရိုဂရမ်၊ product၊ distribution သို့မဟုတ် package တစ်ခုလျှင် key တစ်ခုစီရှိပြီး Juntei က key
တစ်ခုလျှင် record တစ်ခု ထုတ်ပေးသည်။

| Source | Hive နှင့် key | ပါဝင်သည့်အရာ |
|---|---|---|
| `uninstall` | `SOFTWARE` နှင့် `NTUSER.DAT` `…\CurrentVersion\Uninstall\*` နှင့် `SOFTWARE\Wow6432Node\…` | Settings နှင့် *Programs and Features* တွင် ပြသသည့်အရာများ- အမည်၊ version၊ publisher၊ ထည့်သွင်းသည့်ရက်စွဲ၊ ဖိုင်တွဲများ၊ uninstall ညွှန်ကြားချက်များ၊ အရွယ်အစားနှင့် ဝှက်ထားခြင်း ရှိ/မရှိ |
| `msi_product` | `SOFTWARE\Classes\Installer\Products\*`၊ `NTUSER.DAT` `Software\Microsoft\Installer\Products\*` | Windows Installer product များ- product အမည်နှင့် version၊ package ၏ ဖိုင်အမည်နှင့် ၎င်းကို ထည့်သွင်းခဲ့သည့် ဖိုင်တွဲ |
| `app_path` | `SOFTWARE` နှင့် `NTUSER.DAT` `…\CurrentVersion\App Paths\*` | Windows က အမည်ဖြင့် စတင်ဖွင့်ပေးသော ပရိုဂရမ်များ (`Win+R`, `start tool`)- ပရိုဂရမ်၏ path နှင့် ၎င်း၏ ဖိုင်တွဲ |
| `wsl` | `NTUSER.DAT` `…\CurrentVersion\Lxss\{GUID}` | Windows Subsystem for Linux distribution များ- အမည်၊ ၎င်း၏ ဖိုင်များ သို့မဟုတ် disk ရှိရာနေရာ၊ WSL 1 သို့မဟုတ် 2 |
| `package` | `SOFTWARE` `…\Appx\AppxAllUserStore` | Package ပြုလုပ်ထားသော (Store) app များ- အသုံးပြုသူအားလုံးအတွက် provision လုပ်ထားသည့်အရာများ၊ Windows ၏ ကိုယ်ပိုင် app များ၊ staged၊ deprovisioned နှင့် အသုံးပြုသူတစ်ဦးချင်းစီ၏ app များ |

<span class="evidence-strength">အကောင်းဆုံး ကောက်ချက်:</span> ဤပရိုဂရမ်၊ product သို့မဟုတ် package ကို
စနစ်ပေါ်တွင် အသုံးပြုသူအားလုံးအတွက် သို့မဟုတ် ဤအသုံးပြုသူအတွက် ထည့်သွင်းခဲ့သည် (သို့မဟုတ် register လုပ်ခဲ့သည်)။
ထည့်သွင်းသည့်ရက်စွဲမှာ installer က ရေးသားခဲ့သည့်အရာဖြစ်ပြီး key ၏ နောက်ဆုံးရေးသားချိန်မှာ entry နောက်ဆုံးပြောင်းလဲခဲ့သည့်အချိန် ဖြစ်သည်။

## တည်နေရာ { #location }

```text
C:\Windows\System32\config\SOFTWARE
  Microsoft\Windows\CurrentVersion\Uninstall\<name or {ProductCode}>
  Wow6432Node\Microsoft\Windows\CurrentVersion\Uninstall\<…>          32-bit programs
  Classes\Installer\Products\<packed product code>[\SourceList]
  Microsoft\Windows\CurrentVersion\App Paths\<program.exe>
  Microsoft\Windows\CurrentVersion\Appx\AppxAllUserStore\{Applications,InboxApplications,Staged,Deprovisioned,<SID>,EndOfLife\<SID>}\<package>
C:\Users\<user>\NTUSER.DAT
  Software\Microsoft\Windows\CurrentVersion\Uninstall\<…>
  Software\Microsoft\Installer\Products\<packed product code>
  Software\Microsoft\Windows\CurrentVersion\App Paths\<program.exe>
  Software\Microsoft\Windows\CurrentVersion\Lxss\{GUID}
```

`SOFTWARE`၊ `NTUSER.DAT` အားလုံးနှင့် ၎င်းတို့၏ transaction log များကို စုဆောင်းပါ။ `SYSTEM` က host ၏ အမည်ကို
ဖော်ပြပြီး `SOFTWARE` ၏ `ProfileList` က အသုံးပြုသူတစ်ဦးချင်းစီ၏ SID ကို ပေးသည်။

## စိစစ်သူများ သတိပြုရန် { #analyst-cautions }

- **ထည့်သွင်းသည့်ရက်စွဲနှင့် key အချိန်။** `InstallDate` သည် installer က ဒေသစံတော်ချိန်ဖြင့် ရေးသားထားသော ရက်စွဲ
  (အချိန်မပါ) ဖြစ်သည်။ အချို့ installer များက ၎င်းကို မရေးသားဘဲ အချို့က အခြားပုံစံဖြင့် ရေးသားသည် (`InstallDateRaw`
  တွင် ရေးသားထားသည့်အတိုင်း ထိန်းသိမ်းထားသည်)။ Update တစ်ခုက ရက်စွဲကို မပြောင်းလဲဘဲ entry ကို ပြန်ရေးနိုင်သလို
  ပရိုဂရမ် များများစားစား မပြောင်းလဲဘဲ ရက်စွဲကို ပြောင်းလဲနိုင်သည်- `InstallDate` ကို `KeyLastWrittenUTC` နှင့် နှိုင်းယှဉ်ပါ။
- **Uninstall လုပ်ထားသော software။** သန့်ရှင်းစွာ ဖယ်ရှားခဲ့သော ပရိုဂရမ်သည် ၎င်း၏ key များကို ဆုံးရှုံးသည်။ ဖျက်ထားသော
  `Uninstall` key များအတွက် `parse-registry read
  --recover-deleted` ကို အသုံးပြုပြီး ကျန်ရစ်ခဲ့သော သဲလွန်စများအတွက် Amcache၊ Prefetch နှင့် `$MFT` ကို အသုံးပြုပါ။
- **အသုံးပြုသူအလိုက် register လုပ်ခြင်းများ။** `NTUSER.DAT` entry များသည် profile တစ်ခုအတွက် register လုပ်ထားသော
  software ကို ဖော်ထုတ်ပေးသည်။ ထို scope က installer ကို မည်သူ run ခဲ့သည် သို့မဟုတ် administrator အခွင့်အရေး
  အသုံးပြုခဲ့ခြင်း ရှိ/မရှိကို သက်သေမပြပါ။
- **ဝှက်ထားသော entry များ။** `SystemComponent` က entry ကို *Programs and Features* မှ ဝှက်ထားသည်။ `DisplayName`
  မပါသော entry များ (update များ၊ component များ) ကိုလည်း မပြသပါ။ ၎င်းတို့အားလုံးကို အစီရင်ခံသည်။
- **Product code များ။** GUID ပုံစံရှိသော `Uninstall` key ကို `ProductCode` မှတစ်ဆင့် pack လုပ်ထားသော
  `msi_product` key နှင့် တိုက်ဆိုင်နိုင်သည်။ GUID ပုံစံအမည် တစ်ခုတည်းက Windows Installer က entry ကို
  ဖန်တီးခဲ့ကြောင်း သက်သေမပြပါ။
- **App Paths။** ၎င်းတို့သည် ပရိုဂရမ်များကို ရှာဖွေရန် အသုံးပြုသော registration များဖြစ်ပြီး လုပ်ဆောင်မှု (execution)
  မှတ်တမ်းများ မဟုတ်ပါ။ `Path` value တွင် ရှာဖွေရေး directory အများအပြား ပါဝင်နိုင်ပြီး ထည့်သွင်းထားသည့် directory
  ဖြစ်ရန် မလိုပါ။
- **Publisher။** ၎င်းသည် installer က ပေးထားသော registry metadata ဖြစ်ပြီး အတည်ပြုထားသော signature အချက်အလက် မဟုတ်ပါ။
- **Architecture။** `Wow6432Node` တွင် 64-bit Windows ပေါ်ရှိ 32-bit ပရိုဂရမ်များ ပါဝင်သည် (`Architecture` `x86`,
  `Wow64` `True`)။ အခြား entry များသည် စနစ်၏ ကိုယ်ပိုင် architecture ဖြစ်သည် (ဗလာ)။ Package ၏ architecture
  မှာ ၎င်း၏ ကိုယ်ပိုင်ဖြစ်သည် (`x64`, `neutral`, …)။
- **WSL။** `Version` သည် distribution ၏ `Flags` မှ WSL version ဖြစ်သည် (`0x8`: WSL 2 ၏ virtual machine တွင်
  run သည်)။ Distribution ၏ ကိုယ်ပိုင် `Version` value (`Details` တွင်) မှာ ၎င်း၏ ဖိုင်စနစ် version ဖြစ်ပြီး WSL 1
  distribution များတွင်လည်း 2 ဖြစ်သည်။ `BasePath` တွင် ၎င်း၏ `ext4.vhdx` (WSL 2) သို့မဟုတ် `rootfs` (WSL 1)
  ရှိသည်- အတွင်းကို ကြည့်ရန် ၎င်းကို စုဆောင်းပါ။
- **Package များ။** `<SID>` စာရင်းများသည် အသုံးပြုသူတစ်ဦးချင်းစီအတွက် register လုပ်ထားသော package များဖြစ်ပြီး
  အများစုမှာ Windows ၏ ကိုယ်ပိုင်ဖြစ်သည်။ `Deprovisioned` package များကို အသုံးပြုသူအသစ်များအတွက် ဖယ်ရှားခဲ့ပြီး
  `EndOfLife\<SID>` package များကို ထိုအသုံးပြုသူအတွက် ဖယ်ရှားခဲ့သည်။
- **ပျက်စီးနေသော value များ။** ပုံစံမမှန်သော နံပါတ်များ၊ မှားယွင်းသော value အမျိုးအစားများနှင့် မမှန်ကန်သော UTF-16
  တို့ကို `Problems` တွင် အစီရင်ခံပြီး `DataRaw` တွင် ထိန်းသိမ်းထားသည်။ Byte သုည ရှိသော value တစ်ခု ရှိနေပါက
  ဗလာ hex string လည်း ပါဝင်သည်။ Optional ကိန်းဂဏန်း field တွင် သန့်ရှင်းသော ဗလာ string ကို မရှိဟု သတ်မှတ်သည်။
  Package version တွင် ဒဿမ အစိတ်အပိုင်း လေးခု ပါဝင်ရမည်ဖြစ်ပြီး တစ်ခုစီသည် unsigned 16-bit အတိုင်းအတာအတွင်း
  ရှိရမည်။ `-t` က record များကို stdout သို့ stream လုပ်သည့်အခါ hive အဆင့် သတိပေးချက်များသည် stderr တွင် ကျန်ရှိသည်။

## Juntei ဖြင့် ခွဲခြမ်းစိတ်ဖြာခြင်း { #parse-with-juntei }

```text
Usage: juntei parse-registry installed-programs <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SOFTWARE and NTUSER.DAT hives
  -f, --file <FILE>      Parse one SOFTWARE or NTUSER.DAT hive
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry installed-programs -d evidence/ -o installed.csv
juntei parse-registry installed-programs -d evidence/ -t jsonl \
  | jq 'select(.Scope == "user" and .Source != "package")'
```

အနှစ်ချုပ်တွင် နောက်ဆုံးထည့်သွင်းခဲ့သော ပရိုဂရမ်များ (`InstallDate` အလိုက်)၊ အသုံးပြုသူများ ကိုယ်တိုင်အတွက်
ထည့်သွင်းခဲ့သည့်အရာများနှင့် WSL distribution များကို ဖော်ပြသည်။

## ထုတ်ယူမှု field များ { #output-fields }

| Field | အဓိပ္ပာယ်ဖော်ပုံ |
|---|---|
| `Host`, `SourcePath`, `HiveType` | Host၊ hive ဖိုင်နှင့် `SOFTWARE` သို့မဟုတ် `NTUSER.DAT`။ |
| `User`, `UserSID` | အသုံးပြုသူ hive ၏ အသုံးပြုသူ (၎င်း၏ profile ဖိုင်တွဲနှင့် `ProfileList` SID) သို့မဟုတ် package ကို စာရင်းသွင်းထားသော အသုံးပြုသူ။ |
| `Source`, `Scope` | စာရင်း (`uninstall`, `msi_product`, `app_path`, `wsl`, `package`) နှင့် `machine` သို့မဟုတ် `user`။ |
| `Name`, `Version`, `Publisher` | `DisplayName`, `DisplayVersion`, `Publisher`။ Product ၏ `ProductName` နှင့် `Version`။ App Path ၏ ပရိုဂရမ်ဖိုင်အမည်။ Distribution ၏ အမည်နှင့် WSL version။ Package ၏ အမည်နှင့် version။ |
| `InstallDate`, `InstallDateRaw` | `YYYYMMDD` အဖြစ် ရေးသားထားပါက `InstallDate` ကို ရက်စွဲအဖြစ်၊ မဟုတ်ပါက ရေးသားထားသည့်အတိုင်း။ |
| `KeyLastWrittenUTC` | Entry ၏ key နောက်ဆုံးပြောင်းလဲခဲ့သည့်အချိန်။ |
| `InstallLocation`, `InstallSource` | `InstallLocation` (App Path ၏ `Path`၊ distribution ၏ `BasePath`၊ package ၏ `Path`)။ `InstallSource` (product ၏ နောက်ဆုံးအသုံးပြုခဲ့သော source ဖိုင်တွဲ)။ |
| `Program` | App Path ၏ ပရိုဂရမ်။ |
| `UninstallCommand`, `QuietUninstallCommand` | `UninstallString`, `QuietUninstallString`။ |
| `SizeKB`, `SystemComponent`, `WindowsInstaller` | KiB ဖြင့် `EstimatedSize`။ ဝှက်ထားခြင်း ရှိ/မရှိ။ Windows Installer က ထည့်သွင်းခဲ့ခြင်း ရှိ/မရှိ။ |
| `ProductCode` | Windows Installer product code (`{GUID}`)။ |
| `Architecture`, `WOW64` | `Wow6432Node` အောက်တွင် `x86` သို့မဟုတ် package ၏ architecture။ Key သည် `Wow6432Node` အောက်တွင် ရှိ/မရှိ။ |
| `Details` | နောက်ထပ်အချက်များ- `DisplayIcon`, `ModifyPath`, `ParentKeyName`, `ReleaseType`, `Inno Setup: User` (Inno Setup installer ကို run ခဲ့သော အကောင့်)။ Product ၏ `PackageCode`, `PackageName`။ Distribution ၏ `Flags`, `State`, `DefaultUid`, `Version`, `PackageFamilyName`, `Default`။ Package ၏ `List`, `ResourceId`, `PublisherId`။ |
| `KeyPath` | Entry ၏ key။ |
| `DataRaw` | ၎င်းတို့၏ အမည်က ဖော်ပြသည့်အရာနှင့် မကိုက်ညီသော value များ၏ byte များ (hex)။ |
| `TransactionLogStatus`, `LogsApplied` | Registry ပြန်လည်ရယူမှု အခြေအနေ။ |
| `Problems` | ဖတ်၍မရသော သို့မဟုတ် ၎င်းတို့၏ အမည်က ဖော်ပြသည့်အရာနှင့် မကိုက်ညီသော value များ။ |

## စုံစမ်းစစ်ဆေးမှု ဗျူဟာ { #investigation-strategy }

1. ဖြစ်ရပ်ဝန်းကျင်တွင် `InstallDate` နှင့် `KeyLastWrittenUTC` အလိုက် စီပါ။ အဝေးမှ ဝင်ရောက်သည့် tool များ၊
   tunnelling နှင့် ဖိုင်လွှဲပြောင်းရေး client များ၊ archiver များ၊ scripting runtime များနှင့် လုံခြုံရေး tool များကို ကြည့်ပါ။
2. `Scope` `user` record များကို ပြန်လည်သုံးသပ်ပါ- administrator အခွင့်အရေးမပါဘဲ ထည့်သွင်းခဲ့သော software။
3. အသုံးပြုသူ ရေးသားနိုင်သော ဖိုင်တွဲများ (`Downloads`, `AppData`, `Temp`, `ProgramData`) အတွက်
   `InstallLocation`၊ `InstallSource` နှင့် `UninstallCommand` ကို စစ်ဆေးပါ။
4. WSL distribution များကို ရှာဖွေပြီး ၎င်းတို့၏ `BasePath` ကို စုဆောင်းပါ။
5. Amcache နှင့် Prefetch (installer နှင့် ပရိုဂရမ် run ခဲ့ခြင်း)၊ `$MFT` (ထည့်သွင်းသည့် ဖိုင်တွဲ ဖန်တီးခြင်း) နှင့်
   Application event log (MsiInstaller event များ) တို့နှင့် ချိတ်ဆက်စစ်ဆေးပါ။

## နောက်ထပ် အသေးစိတ်များ { #further-details }

- [Microsoft: Uninstall registry key](https://learn.microsoft.com/en-us/windows/win32/msi/uninstall-registry-key)
- [Microsoft: App registration (App Paths)](https://learn.microsoft.com/en-us/windows/win32/shell/app-registration)
- [Microsoft: Package identity overview](https://learn.microsoft.com/en-us/windows/apps/desktop/modernize/package-identity-overview)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
