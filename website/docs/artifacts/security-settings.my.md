# လုံခြုံရေးဆက်တင်များ { #security-settings }

## ၎င်းတို့ ဆိုသည်မှာ { #what-they-are }

တိုက်ခိုက်သူများသည် ပုန်းအောင်းနေရန်နှင့် credential များဆီ ရောက်ရှိရန် Windows ၏ လုံခြုံရေးဆက်တင်များကို ပြောင်းလဲကြသည်။
၎င်းတို့သည် Microsoft Defender ကို ပိတ်ခြင်း သို့မဟုတ် ၎င်းတို့၏ tool များကို ၎င်းမှ ချန်လှပ်ခြင်း (exclude)၊ Windows ကို
cleartext password များ သိမ်းထားစေခြင်း၊ Remote Desktop ကို ဖွင့်ခြင်း၊ firewall ကို ပိတ်ခြင်းနှင့် PowerShell logging ကို
တိတ်ဆိတ်စေခြင်းတို့ကို ပြုလုပ်ကြသည်။ Registry သည် ဤဆက်တင်များကို သိမ်းထားသည်-

- **`SOFTWARE`:** Defender၊ ၎င်း၏ exclusion များနှင့် မူဝါဒများ၊ UAC၊ Remote Desktop နှင့် firewall ၏ မူဝါဒများ၊
  PowerShell ၏ execution policy၊ logging မူဝါဒများနှင့် AMSI provider များ၊ System Restore၊ event log channel များ။
- **`SYSTEM`** (control set အလိုက်): `WDigest`၊ LSA protection၊ Credential Guard၊ Restricted Admin၊
  NTLM၊ Remote Desktop၊ firewall profile များ၊ SMBv1၊ last access အချိန်များ၊ event log တစ်ခုစီ၏ အရွယ်အစား။
- **`NTUSER.DAT`:** အသုံးပြုသူ၏ PowerShell execution policy နှင့် logging မူဝါဒများ၊ Windows Script Host အတွက် AMSI။

`parse-registry security-settings` သည် ၎င်းတွေ့ရှိသော ဆက်တင်တစ်ခုစီကို record တစ်ခုအဖြစ် အစီရင်ခံသည်။ Record တစ်ခုစီတွင်
ဆက်တင်၏ value၊ value ၏ အဓိပ္ပာယ်နှင့် ၎င်းက **မူလသတ်မှတ်ချက်ကို အားနည်းစေခြင်း** ရှိ/မရှိ ပါဝင်သည်- ၎င်းသည် ကာကွယ်မှု
သို့မဟုတ် အထောက်အထား source တစ်ခုကို ပိတ်ခြင်း၊ သို့မဟုတ် value မရှိသည့်အခါ လက်ရှိ Windows (10 နှင့် 11) လုပ်ဆောင်သည့်
အဆင့်အောက်သို့ လျှော့ချခြင်း ရှိ/မရှိ။

<span class="evidence-strength">အကောင်းဆုံး ကောက်ချက်:</span> Hive ကို သိမ်းဆည်းခဲ့ချိန်တွင် ဤဆက်တင်၌ ဤ value ရှိခဲ့ပြီး
key ကို ဤအချိန်တွင် နောက်ဆုံးရေးသားခဲ့သည်။ အားနည်းစေထားသော ဆက်တင်သည် သဲလွန်စတစ်ခုသာဖြစ်ပြီး ချိုးဖောက်ခံရမှု၏
သက်သေ မဟုတ်ပါ။ Administrator များ၊ Group Policy၊ အခြား လုံခြုံရေးထုတ်ကုန်များနှင့် Windows ဗားရှင်းဟောင်းများကလည်း
ဤ value အချို့ကို သတ်မှတ်ကြပြီး ၎င်းသည် အဖြစ်များသည့်နေရာတွင် အဓိပ္ပာယ်ဖော်ပြချက်က ထိုသို့ ဖော်ပြထားသည်။

## တည်နေရာ { #location }

```text
C:\Windows\System32\config\SOFTWARE
  Microsoft\Windows Defender[\Real-Time Protection|\Features|\Spynet]
  Microsoft\Windows Defender\Exclusions\Paths|Extensions|Processes|IpAddresses|TemporaryPaths
  Policies\Microsoft\Windows Defender\…
  Microsoft\Windows\CurrentVersion\Policies\System                 UAC
  Policies\Microsoft\Windows NT\Terminal Services                  RDP policy
  Policies\Microsoft\WindowsFirewall\<profile>                      firewall policy
  Microsoft\PowerShell\1\ShellIds\Microsoft.PowerShell             ExecutionPolicy
  Policies\Microsoft\Windows\PowerShell\ScriptBlockLogging|ModuleLogging|Transcription
  Microsoft\AMSI\Providers\{CLSID}
  Microsoft\Windows NT\CurrentVersion\SystemRestore                DisableSR
  Microsoft\Windows\CurrentVersion\WINEVT\Channels\<channel>       Enabled, MaxSize
C:\Windows\System32\config\SYSTEM (ControlSet00N\…)
  Control\SecurityProviders\WDigest                                UseLogonCredential
  Control\Lsa                                                      RunAsPPL, LsaCfgFlags, DisableRestrictedAdmin, LmCompatibilityLevel
  Control\Terminal Server[\WinStations\RDP-Tcp]                    fDenyTSConnections, PortNumber, UserAuthentication
  Services\SharedAccess\Parameters\FirewallPolicy\<profile>        EnableFirewall
  Services\LanmanServer\Parameters                                 SMB1
  Control\FileSystem                                               NtfsDisableLastAccessUpdate
  Services\EventLog\<log>                                          MaxSize
C:\Users\<user>\NTUSER.DAT
  Software\Microsoft\PowerShell\1\ShellIds\Microsoft.PowerShell    ExecutionPolicy
  Software\Policies\Microsoft\Windows\PowerShell\…
  Software\Microsoft\Windows Script\Settings                       AmsiEnable
```

Hive သုံးခုစလုံးကို ၎င်းတို့၏ transaction log များနှင့်အတူ စုဆောင်းပါ။ စုဆောင်းခြင်းမပြုမီ မကြာမီက ပြောင်းလဲခဲ့သော ဆက်တင်သည်
log များထဲတွင်သာ ရှိနိုင်သည်။

## စိစစ်သူများ သတိပြုရန် { #analyst-cautions }

- **သိမ်းဆည်းထားသော ပြင်ဆင်သတ်မှတ်ချက်သည် ထိရောက်သော ကာကွယ်မှု အခြေအနေ မဟုတ်ပါ။** စုဆောင်းထားသော registry value များနှင့်
  ပျက်စီးနေသော value စာရင်းများအတွက် diagnostic row များကို အစီရင်ခံသည်။ ပျောက်နေသော ဒေတာသည် မူလသတ်မှတ်ချက်များ၊
  မစုဆောင်းရသေးသော source များ သို့မဟုတ် အခြား စီမံခန့်ခွဲမှု နည်းလမ်းတစ်ခုကို ထင်ဟပ်နိုင်သည်။ ၎င်းသည် ကာကွယ်မှု ဖွင့်ထား
  သို့မဟုတ် ပိတ်ထားကြောင်း သက်သေ မဟုတ်ပါ။ `WeakensDefault` သည် လိုက်နာဆောင်ရွက်ခံရသည့်အခါ ဖော်ပြထားသော baseline ကို
  လျှော့ချနိုင်သည့် ဆက်တင်များကို အမှတ်အသားပြုသည်။
- **Legacy Defender flag များကို လျစ်လျူရှုနိုင်သည်။** `DisableAntiSpyware` နှင့် `DisableAntivirus` သည် Defender
  ရပ်တန့်သွားကြောင်း သက်သေမပြပါ။ သက်ဆိုင်မှုသည် platform ဗားရှင်း၊ onboarding နှင့် tamper protection ပေါ်
  မူတည်သည်။ [Microsoft က ဤ legacy ဆက်တင်များကို ရှင်းပြထားသည်](https://learn.microsoft.com/en-us/windows-hardware/customize/desktop/unattend/security-malware-windows-defender-disableantispyware)။
- **ခေတ်သစ် စီမံခန့်ခွဲထားသော exclusion များကို registry capture များတွင် မရရှိနိုင်ပါ။** Platform 4.18.25110.6 မှစ၍
  Defender for Endpoint configuration management ကို အသုံးပြုသော စက်များတွင် exclusion ပြင်ဆင်သတ်မှတ်ချက်ကို ရယူရန်
  ပံ့ပိုးထားသော Defender PowerShell cmdlet များ လိုအပ်သည်။ ထို့ကြောင့် offline registry တွင် မရှိခြင်းက exclusion များ
  မရှိကြောင်း မသက်သေပြပါ။
  [Microsoft ၏ ပြင်ဆင်သတ်မှတ်မှု လမ်းညွှန်](https://learn.microsoft.com/en-us/defender-endpoint/troubleshoot-settings)။
- **အားနည်းစေထားခြင်းသည် အန္တရာယ်ရှိသည်ဟု မဆိုလိုပါ။** ဥပမာများ-
  - `DisableAntiSpyware` = 1 ကို အခြား antivirus တစ်ခု မှတ်ပုံတင်ထားစဉ် Windows ကလည်း သတ်မှတ်သည်။
  - Server များတွင် Remote Desktop ကို ရည်ရွယ်ချက်ရှိရှိ ဖွင့်ထားကြသည်။
  - Windows 7 သည် event log များကို ပိုသေးငယ်စွာ ထားရှိပြီး last access အချိန်များကို အပ်ဒိတ်မလုပ်ခဲ့ပါ။
- **မူလသတ်မှတ်ချက်များသည် ဗားရှင်းပေါ် မူတည်သည်။** အချို့ value များသည် ယခင်က မူလသတ်မှတ်ချက် ဖြစ်ခဲ့သည်-
  - `NtfsDisableLastAccessUpdate` = 1 သည် Windows 10 1803 မတိုင်မီ မူလသတ်မှတ်ချက် ဖြစ်ခဲ့သည်။ ၎င်းကို စီရင်ချက်မပါဘဲ
    အစီရင်ခံသော်လည်း နောက်ပိုင်းဗားရှင်းတွင် လက်ဖြင့် သတ်မှတ်ထားသော `0x80000001` ကိုမူ အားနည်းစေထားသည်ဟု သတ်မှတ်သည်။
  - `RunAsPPL`၊ `LsaCfgFlags` နှင့် `UseLogonCredential` ကို လက်ရှိ Windows နှင့် နှိုင်းယှဉ်သည်။
- **Defender exclusion များ:** path၊ extension၊ process သို့မဟုတ် လိပ်စာ တစ်ခုစီသည် record တစ်ခု ဖြစ်သည်။ Exclusion တိုင်းက
  Defender ကို အားနည်းစေသည်။ Defender ကိုယ်တိုင် ထည့်သွင်းသော `TemporaryPaths` တွင် စီရင်ချက် မရှိပါ။ အသုံးပြုသူ ရေးသားနိုင်သော ဖိုင်တွဲများ၊
  `.ps1`/`.exe` extension များနှင့် LOLBin များ (`rundll32.exe`, `powershell.exe`) ကို ရှာပါ။
- **Policy key များ** (`Policy` = True) ကို Group Policy၊ tool များနှင့် မူဝါဒများက local ဆက်တင်များထက် အနိုင်ရကြောင်း
  သိသော တိုက်ခိုက်သူများက ရေးသားကြသည်။ ၎င်းတို့ကို domain ၏ GPO များနှင့် ချိတ်ဆက်စစ်ဆေးပါ။
- **Event log channel များ:** လုံခြုံရေးနှင့် သက်ဆိုင်သော channel 25 ခုကိုသာ Windows က တစ်ခုစီကို မူလအားဖြင့်
  ဖွင့်ထားခြင်း ရှိ/မရှိနှင့်အတူ အစီရင်ခံသည်။ ဥပမာ `TaskScheduler/Operational` နှင့် `DNS-Client/Operational`
  သည် မူလအားဖြင့် ပိတ်ထားသည်။ အခြားများအတွက် `parse-registry read` ကို အသုံးပြုပါ။
- **Key အချိန်များ:** `KeyLastWrittenUTC` သည် key ၏ အချိန်ဖြစ်သောကြောင့် ဤ value ပြောင်းလဲခဲ့သည့်အချိန် ဖြစ်ချင်မှ
  ဖြစ်မည်။ `Policies` key တစ်ခုလုံးကို တစ်ပြိုင်နက် ရေးသားခြင်းသည် အဖြစ်များသည်။
- **Control set များ:** မူလအားဖြင့် Windows နောက်ဆုံးအသုံးပြုခဲ့သော control set ကို ဖတ်သည်။ `--all-control-sets`
  သည် အားလုံးကို ဖတ်သည်။

## Juntei ဖြင့် ခွဲခြမ်းစိတ်ဖြာခြင်း { #parse-with-juntei }

```text
Usage: juntei parse-registry security-settings <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SOFTWARE, SYSTEM and NTUSER.DAT hives
  -f, --file <FILE>      Parse one hive
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Filtering:
      --all-control-sets  Every control set, not only the one in use
      --weakened-only     Only settings that weaken the default

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry security-settings -d evidence/ -o settings.csv
juntei parse-registry security-settings --weakened-only -d evidence/ -t jsonl \
  | jq -r '[.Host, .Area, .KeyPath, .ValueName, .Value, .Meaning] | @tsv'
```

အနှစ်ချုပ်သည် ဆက်တင်များကို area အလိုက် ရေတွက်သည်။ ၎င်းသည် မူလသတ်မှတ်ချက်ကို အားနည်းစေသော ဆက်တင်များနှင့် Defender
exclusion တိုင်းကို စာရင်းပြသည်။

## ထုတ်ယူမှု field များ { #output-fields }

| Field | အဓိပ္ပာယ်ဖော်ပုံ |
|---|---|
| `Host`, `SourcePath`, `HiveType` | Host၊ hive နှင့် ၎င်း၏ အမျိုးအစား (`SOFTWARE`, `SYSTEM`, `NTUSER.DAT`)။ |
| `ControlSet`, `IsCurrentControlSet` | `SYSTEM` အတွက်- control set နှင့် ၎င်းသည် Windows နောက်ဆုံးအသုံးပြုခဲ့သော control set ဖြစ်/မဖြစ်။ `Select\Current` ပျောက်နေလျှင် သို့မဟုတ် မမှန်ကန်လျှင် မသိရှိပါ။ |
| `User`, `UserSID` | `NTUSER.DAT` အတွက်- profile ဖိုင်တွဲ၏ အမည်နှင့် စနစ်၏ `ProfileList` မှ အသုံးပြုသူ၏ SID။ |
| `Area` | `defender`, `uac`, `credentials`, `remote_access`, `network`, `powershell`, `evidence`။ |
| `Setting` | ၎င်းသည် ဘာဖြစ်သနည်း- `defender_disable_realtime_monitoring`, `defender_exclusion_path`, `uac_local_account_token_filter_policy`, `wdigest_use_logon_credential`, `rdp_deny_connections`, `powershell_script_block_logging`, `event_channel_enabled`, … |
| `Value` | Value- ဒဿမ နံပါတ်၊ string သို့မဟုတ် exclusion ကိုယ်တိုင်။ Value သည် ဆက်တင်၏ အမျိုးအစား မဟုတ်ပါက ဗလာ။ |
| `Meaning` | Value ၏ အဓိပ္ပာယ်။ |
| `WeakensDefault` | Value က ၎င်းမရှိဘဲ လက်ရှိ Windows လုပ်ဆောင်သည့်အရာကို အားနည်းစေပါက `True`၊ အားမနည်းစေပါက `False`၊ မသက်ဆိုင်ပါက (အရွယ်အစား၊ port၊ AMSI provider) သို့မဟုတ် ဆုံးဖြတ်၍ မရပါက ဗလာ။ |
| `Policy` | Key သည် `Policies` key အောက်တွင် ရှိ/မရှိ။ |
| `KeyPath`, `ValueName` | Key နှင့် value။ Exclusion ၏ value အမည်သည် ချန်လှပ်ထားသော item ဖြစ်သည်။ AMSI provider တွင် value အမည် မရှိပါ။ |
| `KeyLastWrittenUTC` | Key ၏ နောက်ဆုံး ရေးသားချိန်။ |
| `DataRaw` | Value ၏ byte များ (hex)၊ ၎င်းတို့သည် ဆက်တင်၏ အမျိုးအစား မဟုတ်သည့်အခါ။ |
| `TransactionLogStatus`, `LogsApplied` | Registry recovery အခြေအနေ။ |
| `Problems` | အမျိုးအစား မှားယွင်းသော value များနှင့် ဖတ်၍မရသော key များ။ |

## စုံစမ်းစစ်ဆေးမှု ဗျူဟာ { #investigation-strategy }

1. Host တိုင်းတွင် `--weakened-only` ဖြင့် run ပြီး host များကို အချင်းချင်း နှိုင်းယှဉ်ပါ- host တစ်ခုတည်းတွင်သာ ရှိသော
   ဆက်တင်သည် ထင်ရှားစွာ ပေါ်လွင်သည်။
2. Defender exclusion တစ်ခုစီကို host ပေါ်တွင် တွေ့ရှိရသော tool များနှင့် Defender ၏ log များ (Event 5007 က
   ပြင်ဆင်သတ်မှတ်ချက် ပြောင်းလဲမှုများကို မှတ်တမ်းတင်သည်) နှင့် တိုက်ဆိုင်စစ်ဆေးပါ။
3. Key အချိန်များကို logon များ၊ ဝန်ဆောင်မှု ထည့်သွင်းမှုများနှင့် process ဖန်တီးမှုတို့နှင့် ချိတ်ဆက်စစ်ဆေးပါ။
   `reg.exe`၊ `Set-MpPreference`၊ `netsh advfirewall` နှင့် `powershell` command line များကို ရှာပါ (event
   4688၊ Sysmon 1 နှင့် 13၊ PowerShell 4104)။
4. Credential ဆက်တင်များ (`WDigest`, `RunAsPPL`, `LocalAccountTokenFilterPolicy`,
   `DisableRestrictedAdmin`) အတွက် နောက်ပိုင်းတွင် ဖြစ်ပွားသော credential dumping နှင့် pass-the-hash ကို ရှာပါ။
5. မူလသတ်မှတ်ချက်ကို ပြန်လည်ထားရှိရန် ဖျက်ခဲ့သော value များအတွက် `parse-registry read --recover-deleted` ကို လည်းကောင်း၊
   ပိုဟောင်းသော control set များအတွက် `--all-control-sets` ကို လည်းကောင်း အသုံးပြုပါ။

## နောက်ထပ် အသေးစိတ်များ { #further-details }

- [Microsoft: Configure and validate exclusions for Microsoft Defender Antivirus](https://learn.microsoft.com/en-us/defender-endpoint/configure-exclusions-microsoft-defender-antivirus)
- [Microsoft: User Account Control settings and configuration](https://learn.microsoft.com/en-us/windows/security/application-security/application-control/user-account-control/settings-and-configuration)
- [Microsoft: Configure added LSA protection](https://learn.microsoft.com/en-us/windows-server/security/credentials-protection-and-management/configuring-additional-lsa-protection)
- [Microsoft: about_Logging_Windows](https://learn.microsoft.com/en-us/powershell/module/microsoft.powershell.core/about/about_logging_windows)
- [MITRE ATT&CK T1562.001: Impair Defenses: Disable or Modify Tools](https://attack.mitre.org/techniques/T1562/001/)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
