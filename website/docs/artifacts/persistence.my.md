# Registry အလိုအလျောက် စတင်မှု { #registry-autostarts }

## ၎င်းတို့ ဆိုသည်မှာ { #what-they-are }

Windows သည် registry တည်နေရာ အများအပြားမှ ပရိုဂရမ်များကို စတင်ပြီး DLL များကို load လုပ်သည်- boot တွင်၊ logon တွင်၊
ပရိုဂရမ်တစ်ခု စတင်သည့်အခါ သို့မဟုတ် ထွက်သည့်အခါ၊ `cmd.exe` ဖွင့်သည့်အခါ၊ COM class တစ်ခု ဖန်တီးသည့်အခါ သို့မဟုတ်
spooler၊ LSA၊ netsh သို့မဟုတ် time service က ၎င်း၏ plug-in များကို load လုပ်သည့်အခါ။ တိုက်ခိုက်သူများသည် ဤတည်နေရာများကို
အမြဲတမ်းဝင်ရောက်နိုင်မှု (persistence)၊ အခွင့်အာဏာမြှင့်တင်ခြင်းနှင့် DLL injection အတွက် အသုံးပြုကြပြီး တရားဝင်ဆော့ဖ်ဝဲများကလည်း
အသုံးပြုကြသည်။ Juntei သည် ၎င်းတို့ကို `SOFTWARE`, `SYSTEM`, `NTUSER.DAT` နှင့် `UsrClass.dat` မှ ဇယားတစ်ခုတည်းသို့
ဖတ်ယူသည်။ ၎င်းသည် ပရိုဂရမ်၊ DLL သို့မဟုတ် package တစ်ခုလျှင် မှတ်တမ်းတစ်ခု ရေးသားပြီး တစ်ခုစီကို ၎င်း၏ MITRE ATT&CK
technique ဖြင့် tag တပ်သည်။

<span class="evidence-strength">အကောင်းဆုံး ကောက်ချက်:</span> hive သည် ဤပရိုဂရမ် သို့မဟုတ် DLL ကို စတင်ရန်
သို့မဟုတ် load လုပ်ရန် ပြင်ဆင်ထားခဲ့သည်။ ၎င်းသည် ပရိုဂရမ် run ခဲ့ကြောင်း သက်သေမပြပါ။ ၎င်းကို run ရန် ယန္တရားက
ဖော်ပြသော အခြေအနေများ (logon တစ်ခု၊ debug လုပ်ခံရသော ပရိုဂရမ် စတင်ခြင်း၊ class ဖန်တီးခံရခြင်း) နှင့် လုပ်ဆောင်ခဲ့ခြင်း၏
အခြားအထောက်အထားများ လိုအပ်သည်။

## တည်နေရာများ { #locations }

| Hive | ဖတ်သည့်အရာ |
|---|---|
| `SOFTWARE` | `Microsoft\Windows\CurrentVersion\Run`, `RunOnce` (+`Setup`), `RunOnceEx`, `RunServices(Once)` နှင့် `Policies\Explorer\Run`၊ ၎င်းတို့၏ `Wow6432Node` နှင့် Terminal Server install-mode မိတ္တူများနှင့်အတူ၊ `Explorer\StartupApproved\Run`, `Run32` နှင့် `StartupFolder`၊ `Winlogon` (`Shell`, `Userinit`, `Taskman`, `AppSetup`, `GinaDLL`, … နှင့် `Notify`)၊ `LoadAppInit_DLLs` နှင့်အတူ `Windows` `AppInit_DLLs`၊ `Image File Execution Options\*` `Debugger` (`UseFilter` နှင့်အတူ) နှင့် `VerifierDlls`၊ `SilentProcessExit\*` `MonitorProcess`၊ `Active Setup\Installed Components\*` `StubPath`၊ `NetSh`၊ `Command Processor` `AutoRun` |
| `SYSTEM` | `Control\Session Manager` (`BootExecute`, `SetupExecute`, `Execute`, `PendingFileRenameOperations`, `KnownDLLs`, `AppCertDlls`, `Environment` .NET profiler variable များ)၊ `Control\Lsa` `Authentication`, `Security` နှင့် `Notification Packages` (+`OSConfig`)၊ `SecurityProviders`၊ `Control\Print` monitor များ၊ provider များနှင့် print processor များ၊ `Services\W32Time\TimeProviders` |
| `NTUSER.DAT` | အသုံးပြုသူ၏ `Run` key များနှင့် `StartupApproved`၊ `Windows NT\CurrentVersion\Windows` `Load` နှင့် `Run`၊ `Winlogon` `Shell`၊ `Environment` (`UserInitMprLogonScript`, .NET profiler)၊ `Control Panel\Desktop` `SCRNSAVE.EXE`၊ `PackagedAppXDebug`၊ `Command Processor` `AutoRun`၊ `Software\Classes` (`UsrClass.dat` အဖြစ်) |
| `UsrClass.dat` | `CLSID\*\InprocServer32` နှင့် `LocalServer32` (နှင့် `Wow6432Node` ၏)၊ ပရိုဂရမ်အမျိုးအစားများ (`exefile`, `batfile`, `scrfile`, …) နှင့် auto-elevate ပရိုဂရမ်များ စတင်သည့်အရာများ (`ms-settings`, `mscfile`, `Folder`, …) ၏ `shell\open\command`၊ `ActivatableClasses\…\DebugInformation` |

Hive တစ်ခုစီကို ၎င်း၏ transaction log များနှင့်အတူ စုဆောင်းပါ။ အသုံးပြုသူ hive များသည် `Users\<name>\NTUSER.DAT` နှင့်
`Users\<name>\AppData\Local\Microsoft\Windows\UsrClass.dat` တွင် ရှိသည်။

## စိစစ်သူများ သတိပြုရန် { #analyst-cautions }

- **ပုံသေတန်ဖိုးများကိုလည်း ဖော်ပြသည်။** သန့်ရှင်းသော စနစ်ပေါ်ရှိ မှတ်တမ်းအများစုသည် Windows ပုံသေတန်ဖိုးများ ဖြစ်သည်-
  - `Userinit` သည် `userinit.exe` ဖြစ်သည်။
  - `Shell` သည် `explorer.exe` ဖြစ်သည်။
  - `KnownDLLs`၊ LSA package များ၊ print monitor များ၊ netsh helper များနှင့် Active Setup component များ အားလုံးတွင် မူလပါရှိသော entry များ ရှိသည်။

  တူညီသော ဗားရှင်း၏ သန့်ရှင်းသော build နှင့် ကွဲပြားသည့်အရာကို ရှာဖွေပါ။
- **`StartupApproved` သည် Task Manager ၏ Startup စာရင်းကိုသာ လွှမ်းခြုံသည်။** ၎င်းသည် `Run` entry သို့မဟုတ်
  Startup ဖိုင်တွဲရှိ ဖိုင်တစ်ခုကို ထိုနေရာတွင် ပိတ်ထားခြင်း ရှိ/မရှိနှင့် မည်သည့်အချိန်တွင် ပိတ်ခဲ့သည်ကို ဖော်ပြသည်။ ကိုက်ညီသော
  `Run` တန်ဖိုး မရှိသော entry ကို "only in StartupApproved" ဟူသော အသေးစိတ်ဖြင့် ဖော်ပြသည်။ ၎င်းသည် ဖယ်ရှားခဲ့သော
  autostart တစ်ခုကို ပြသနိုင်သည်။
- **`OverridesMachineRegistration` အတွက် စနစ်၏ `SOFTWARE` hive လိုအပ်သည်။** အသုံးပြုသူ၏ COM class သို့မဟုတ်
  ညွှန်ကြားချက်ကို ၎င်းနှင့် တိုက်ဆိုင်စစ်ဆေးပြီး host ၏ `SYSTEM` hive ကို ရှာဖွေသည့် နည်းတူ ၎င်းကို ရှာဖွေသည်။ Override
  တစ်ခုသည် COM hijacking ပုံစံ (T1546.015) ဖြစ်သည်။ OneDrive နှင့် အခြား အသုံးပြုသူအလိုက် ထည့်သွင်းမှုများကလည်း
  မည်သည့်အရာကိုမျှ override မလုပ်သော ၎င်းတို့၏ ကိုယ်ပိုင် class များကို မှတ်ပုံတင်ကြသည်။
- **`ImagePath` နှင့် `Args` ကို ခွဲခြားနိုင်သည့်နေရာတွင်သာ ညွှန်ကြားချက် စာကြောင်းမှ ခွဲထုတ်သည်။**
  - Quote ပါသော ပရိုဂရမ်ကို ၎င်း၏ ပိတ် quote တွင် ခွဲသည်။
  - ဖိုင်တွဲမပါသော ပထမစကားလုံးကို ပရိုဂရမ်အဖြစ် ယူသည် (`cmd /c …`)။
  - မဟုတ်ပါက ပရိုဂရမ်၏ extension ဖြင့် အဆုံးသတ်သော အတိုဆုံး အစတွင် ခွဲသည်။

  Space ပါသော quote မပါသည့် path သည် အခြားပရိုဂရမ် (`C:\Program.exe`) တည်ရှိပါက ထိုပရိုဂရမ်ကို run နိုင်သည်။
- **`GlobalFlag` သည် `VerifierDlls` နှင့် `MonitorProcess` အတွက် `Enabled` ကို ဆုံးဖြတ်သည်။** `VerifierDlls`
  သည် `0x100` ဖြင့်သာ load လုပ်သည်။ `MonitorProcess` သည် `0x200` နှင့် `SilentProcessExit\<program>`
  `ReportingMode` ၏ `0x1` ဖြင့်သာ run သည်။ Mode `2` (dump) သို့မဟုတ် `4` (အသိပေးချက်) သည် ဘာမျှ မ run ဘဲ
  `Details` က mode ၏ flag များကို အမည်ဖော်ပြသည်။
- **`KeyLastWrittenUTC` သည် key ၏ နောက်ဆုံးရေးသားချိန် ဖြစ်သည်**၊ တန်ဖိုး၏ မဟုတ်ပါ။ တန်ဖိုးများစွာက ၎င်းကို
  မျှဝေသုံးစွဲသည်။

## Juntei ဖြင့် ခွဲခြမ်းစိတ်ဖြာခြင်း { #parse-with-juntei }

```text
Usage: juntei parse-registry persistence <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SOFTWARE, SYSTEM, NTUSER.DAT and UsrClass.dat hives
  -f, --file <FILE>      Parse one of them
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Filtering:
      --all-control-sets  Every control set of a SYSTEM hive, not only the one in use

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry persistence -d collection/ -o persistence.csv
juntei parse-registry persistence -f NTUSER.DAT -t jsonl \
  | jq 'select(.Mechanism != "com_server" or .OverridesMachineRegistration == true)'
```

အနှစ်ချုပ်သည် မှတ်တမ်းများကို ယန္တရားအလိုက် ရေတွက်သည်။ ၎င်းသည် ၎င်းတို့၏ `Run` တန်ဖိုး မရှိတော့သော
`StartupApproved\Run` နှင့် `Run32` entry များ (Startup ဖိုင်တွဲရှိ ဖိုင်များတွင် ဆုံးရှုံးစရာ တန်ဖိုး မရှိပါ) နှင့်
စက်၏ class များကို override လုပ်သော အသုံးပြုသူ COM class များနှင့် ညွှန်ကြားချက်များကိုလည်း ဖော်ပြသည်။

`parse-registry` ညွှန်ကြားချက်တိုင်းကဲ့သို့ပင် ၎င်းသည် ၎င်းဖတ်သော ဖိုင်များထဲမှ တစ်ခုဖြစ်သည့် `-o` ကို `-C` ဖြင့်ပင်
ငြင်းပယ်သည်။ ၎င်းတွင် hive၊ ၎င်း၏ transaction log များနှင့် စနစ်၏ SYSTEM နှင့် SOFTWARE hive များကို path၊ symbolic
link သို့မဟုတ် hard link ဖြင့် ပါဝင်သည်။ မခွဲခြားနိုင်သည့်အခါ ၎င်းသည် မရေးသားပါ။

## ထုတ်ယူမှု field များ { #output-fields }

| Field | အဓိပ္ပာယ်ဖော်ပုံ |
|---|---|
| `Host`, `SourcePath`, `HiveType` | Host အခြေအနေ၊ source hive နှင့် ၎င်း၏ အမျိုးအစား။ |
| `User`, `UserSID` | အသုံးပြုသူ hive များအတွက်- profile ဖိုင်တွဲနှင့် SID။ SID သည် `UsrClass.dat` ၏ root key အမည်မှ လာပြီး မဟုတ်ပါက စနစ်၏ `ProfileList` မှ လာသည်။ |
| `Mechanism` | `run`, `run_once`, `winlogon`, `ifeo_debugger`, `silent_process_exit`, `appinit_dlls`, `appcert_dlls`, `boot_execute`, `lsa_security_package`, `com_server`, `uac_bypass_command`, … |
| `AttackTechnique` | ကိုက်ညီသည့်နေရာတွင် MITRE ATT&CK technique (`T1547.001`, `T1546.012`, …)။ |
| `KeyPath`, `ValueName` | Hive ၏ root အောက်ရှိ key (ရေးထားသည့်အတိုင်း အမည်များ) နှင့် တန်ဖိုး (ဗလာ- ပုံသေတန်ဖိုး)။ |
| `Data` | တန်ဖိုး၏ စာသား သို့မဟုတ် စာရင်းတစ်ခု၏ item တစ်ခု။ မဖတ်နိုင်သည့်အခါ သို့မဟုတ် `StartupApproved` တစ်ခုတည်းက entry ကို အမည်ဖော်ပြသည့်အခါ ဗလာ။ |
| `ImagePath`, `Args` | ခွဲခြားနိုင်သည့်နေရာတွင် ပရိုဂရမ် သို့မဟုတ် DLL နှင့် argument များ (အထက်တွင် ကြည့်ပါ)။ |
| `Enabled`, `DisabledAtUTC` | `StartupApproved`, `LoadAppInit_DLLs`, `GlobalFlag` (`ReportingMode` နှင့်အတူ), `UseFilter`, `IsInstalled`၊ time provider ၏ `Enabled` သို့မဟုတ် `ScreenSaveActive` က ဖော်ပြသည့်အရာ။ ဘာမျှ မဖော်ပြသည့်အခါ ဗလာ။ |
| `Details` | နောက်ထပ်အချက်များ- ဆိုင်းငံ့ထားသော အမည်ပြောင်းခြင်းက ပြုလုပ်သည့်အရာ၊ COM class ၏ အမည်၊ `ReportingMode`, `FilterFullPath`၊ Active Setup `ShellComponent`၊ … |
| `OverridesMachineRegistration` | အသုံးပြုသူ၏ COM class သို့မဟုတ် ညွှန်ကြားချက်အတွက်- စက်က တူညီသောအရာကို မှတ်ပုံတင်ထားခြင်း ရှိ/မရှိ။ စနစ်၏ `SOFTWARE` hive မရှိပါက ဗလာ။ |
| `WOW64` | `Wow6432Node` အောက်တွင်- 32-bit ပရိုဂရမ်များအတွက်။ |
| `KeyLastWrittenUTC` | Key ၏ နောက်ဆုံးရေးသားချိန်။ |
| `DataRaw` | စာသားမဟုတ်သည့်အခါ သို့မဟုတ် မှန်ကန်သော UTF-16 မဟုတ်သည့်အခါ တန်ဖိုး၏ byte များ (hex)။ |
| `TransactionLogStatus`, `LogsApplied` | Registry ပြန်လည်ရယူမှု အခြေအနေ။ |
| `Problems` | မှားယွင်းသော တန်ဖိုးအမျိုးအစားများ၊ မဖတ်နိုင်သော ဒေတာ၊ အရွယ်အစား မှားယွင်းသော `StartupApproved` entry များ။ |

## စုံစမ်းစစ်ဆေးမှု ဗျူဟာ { #investigation-strategy }

1. အနှစ်ချုပ်မှ စတင်ပါ-
   - စက်၏ class များကို override လုပ်သော COM class များနှင့် ညွှန်ကြားချက်များ၊
   - ၎င်းတို့၏ `Run` တန်ဖိုး မရှိသော `StartupApproved` entry များ၊
   - entry ရှိခဲသော ယန္တရားများ (`ifeo_debugger`, `silent_process_exit`, `appinit_dlls`, `appcert_dlls`, `uac_bypass_command`, `file_association`, `logon_script`, `dotnet_profiler`)။
2. LSA package များ၊ `BootExecute`, `KnownDLLs`၊ print monitor များနှင့် netsh helper များကို တူညီသော Windows
   ဗားရှင်း၏ သန့်ရှင်းသော build နှင့် နှိုင်းယှဉ်ပါ။
3. `ImagePath` ကို ဖိုင်စနစ် ($MFT၊ hash များနှင့် signature များ) နှင့် တိုက်ဆိုင်ဖြေရှင်းပြီး အသုံးပြုသူ profile၊
   `ProgramData` နှင့် `Temp` path များကို စစ်ဆေးပါ။
4. `KeyLastWrittenUTC` ကို Security နှင့် Sysmon registry event များ၊ process ဖန်တီးမှု၊ Prefetch နှင့် Amcache တို့နှင့်
   ချိတ်ဆက်စစ်ဆေးပါ။
5. ဤ key များမှ ဖယ်ရှားခဲ့သော autostart များကို ရှာဖွေရန် `parse-registry read --recover-deleted` ကို အသုံးပြုပါ။

## နောက်ထပ် အသေးစိတ်များ { #further-details }

- [MITRE ATT&CK T1547 Boot or Logon Autostart Execution](https://attack.mitre.org/techniques/T1547/)
- [MITRE ATT&CK T1546 Event Triggered Execution](https://attack.mitre.org/techniques/T1546/)
- [Microsoft: Run နှင့် RunOnce registry key များ](https://learn.microsoft.com/en-us/windows/win32/setupapi/run-and-runonce-registry-keys)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
