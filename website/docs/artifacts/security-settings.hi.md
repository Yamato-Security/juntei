# सुरक्षा सेटिंग { #security-settings }

## ये क्या हैं { #what-they-are }

हमलावर छिपे रहने और credentials तक पहुँचने के लिए Windows की security settings बदलते हैं। वे
Microsoft Defender को बंद कर देते हैं या अपने tools को उससे बाहर (exclude) कर देते हैं, Windows से cleartext passwords रखवाते हैं,
Remote Desktop खोलते हैं, firewall बंद करते हैं, और PowerShell logging को चुप कर देते हैं। Registry
ये settings रखती है:

- **`SOFTWARE`:** Defender, उसकी exclusions और policies; UAC; Remote Desktop और firewall की
  policies; PowerShell की execution policy, logging policies और AMSI providers; System Restore;
  event log channels।
- **`SYSTEM`** (प्रति control set): `WDigest`, LSA protection, Credential Guard, Restricted Admin,
  NTLM; Remote Desktop; firewall profiles; SMBv1; last access times; हर event log का आकार।
- **`NTUSER.DAT`:** उपयोगकर्ता की PowerShell execution policy और logging policies, और
  Windows Script Host के लिए AMSI।

`parse-registry security-settings` हर मिली setting को एक record के रूप में report करता है। हर record में
setting का value, उस value का अर्थ, और यह होता है कि क्या यह **default को कमज़ोर करता है**: क्या यह किसी
protection या साक्ष्य स्रोत को बंद करता है, या उसे उस स्तर से नीचे ले जाता है जो वर्तमान Windows (10 और 11)
उस value के बिना करता है।

<span class="evidence-strength">सर्वोत्तम निष्कर्ष:</span> जब hive सहेजा गया था तब इस setting का यह value था,
और key आख़िरी बार इस समय लिखी गई थी। कमज़ोर की गई setting एक सुराग है, compromise का प्रमाण नहीं।
Administrators, Group Policy, अन्य security products और पुराने Windows संस्करण भी इनमें से
कुछ values सेट करते हैं, और जहाँ यह आम है वहाँ अर्थ (meaning) यह बताता है।

## स्थान { #location }

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

तीनों hives को उनके transaction logs के साथ एकत्र करें। Collection से कुछ समय पहले बदली गई setting
केवल logs में हो सकती है।

## विश्लेषक के लिए सावधानियाँ { #analyst-cautions }

- **संग्रहीत configuration प्रभावी protection स्थिति नहीं है।** एकत्र की गई registry values और क्षतिग्रस्त value सूचियों के लिए diagnostic rows
  report की जाती हैं। लापता data defaults, एकत्र न किए गए स्रोतों या किसी अन्य management
  तंत्र को दर्शा सकता है; यह इस बात का प्रमाण नहीं कि protection enabled या disabled है। `WeakensDefault` उन
  settings को flag करता है जो लागू किए जाने पर बताए गए baseline को घटा सकती हैं।
- **Legacy Defender flags को अनदेखा किया जा सकता है।** `DisableAntiSpyware` और `DisableAntivirus` यह
  सिद्ध नहीं करते कि Defender रुक गया। लागू होना platform version, onboarding और tamper
  protection पर निर्भर करता है। [Microsoft इन legacy settings की व्याख्या करता है](https://learn.microsoft.com/en-us/windows-hardware/customize/desktop/unattend/security-malware-windows-defender-disableantispyware)।
- **आधुनिक managed exclusions registry captures में अनुपलब्ध हो सकती हैं।** Platform
  4.18.25110.6 से शुरू होकर, Defender for Endpoint configuration management का उपयोग करने वाले devices को exclusion configuration
  प्राप्त करने के लिए समर्थित Defender PowerShell cmdlets की आवश्यकता होती है। इसलिए offline registry में अनुपस्थिति
  exclusions की अनुपस्थिति स्थापित नहीं करती।
  [Microsoft configuration मार्गदर्शन](https://learn.microsoft.com/en-us/defender-endpoint/troubleshoot-settings)।
- **कमज़ोर का अर्थ malicious नहीं।** उदाहरण:
  - `DisableAntiSpyware` = 1 Windows द्वारा भी सेट किया जाता है जब कोई अन्य antivirus registered हो।
  - Servers पर Remote Desktop जानबूझकर खोला जाता है।
  - Windows 7 छोटे event logs रखता था और last access times update नहीं करता था।
- **Defaults version पर निर्भर करते हैं।** कुछ values पहले default थीं:
  - `NtfsDisableLastAccessUpdate` = 1 Windows 10 1803 से पहले default था। इसे बिना निर्णय के report
    किया जाता है, लेकिन बाद के version पर हाथ से सेट किया गया `0x80000001` कमज़ोर माना जाता है।
  - `RunAsPPL`, `LsaCfgFlags` और `UseLogonCredential` की तुलना वर्तमान Windows से की जाती है।
- **Defender exclusions:** हर path, extension, process या address एक record है। हर exclusion
  Defender को कमज़ोर करती है। `TemporaryPaths`, जिसे Defender स्वयं जोड़ता है, पर कोई निर्णय नहीं दिया जाता। User-writable folders,
  `.ps1`/`.exe` extensions और LOLBins (`rundll32.exe`, `powershell.exe`) खोजें।
- **Policy keys** (`Policy` = True) Group Policy, tools, और उन हमलावरों द्वारा लिखी जाती हैं जो
  जानते हैं कि policies local settings पर भारी पड़ती हैं। इन्हें domain के GPOs से correlate करें।
- **Event log channels:** केवल 25 security-प्रासंगिक channels report किए जाते हैं, साथ में यह कि Windows
  हर एक को default रूप से चालू करता है या नहीं। उदाहरण के लिए, `TaskScheduler/Operational` और `DNS-Client/Operational`
  default रूप से बंद होते हैं। अन्य के लिए `parse-registry read` का उपयोग करें।
- **Key के समय:** `KeyLastWrittenUTC` key का है, इसलिए यह ज़रूरी नहीं कि यह value
  तब बदली। पूरी `Policies` key का एक साथ लिखा जाना आम है।
- **Control sets:** default रूप से वह control set पढ़ा जाता है जिसका Windows ने आख़िरी बार उपयोग किया। `--all-control-sets`
  हर control set पढ़ता है।

## Juntei से parse करें { #parse-with-juntei }

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

Summary settings को क्षेत्र के अनुसार गिनती है। यह default को कमज़ोर करने वाली settings और हर
Defender exclusion सूचीबद्ध करती है।

## आउटपुट fields { #output-fields }

| फ़ील्ड | व्याख्या |
|---|---|
| `Host`, `SourcePath`, `HiveType` | Host, hive, और उसका प्रकार (`SOFTWARE`, `SYSTEM`, `NTUSER.DAT`)। |
| `ControlSet`, `IsCurrentControlSet` | `SYSTEM` के लिए: control set, और क्या यह वही है जिसका Windows ने आख़िरी बार उपयोग किया; `Select\Current` लापता या अमान्य होने पर अज्ञात। |
| `User`, `UserSID` | `NTUSER.DAT` के लिए: profile folder का नाम, और सिस्टम की `ProfileList` से उपयोगकर्ता का SID। |
| `Area` | `defender`, `uac`, `credentials`, `remote_access`, `network`, `powershell`, `evidence`। |
| `Setting` | यह क्या है: `defender_disable_realtime_monitoring`, `defender_exclusion_path`, `uac_local_account_token_filter_policy`, `wdigest_use_logon_credential`, `rdp_deny_connections`, `powershell_script_block_logging`, `event_channel_enabled`, … |
| `Value` | Value: decimal में एक number, एक string, या स्वयं exclusion। जब value setting के type का न हो तो खाली। |
| `Meaning` | Value का क्या अर्थ है। |
| `WeakensDefault` | `True` जब value उसे कमज़ोर करता है जो वर्तमान Windows उसके बिना करता है, `False` जब नहीं करता, खाली जब यह लागू नहीं होता (आकार, port, AMSI provider) या बताया नहीं जा सकता। |
| `Policy` | क्या key किसी `Policies` key के अंतर्गत है। |
| `KeyPath`, `ValueName` | Key और value। किसी exclusion का value नाम excluded item होता है; AMSI provider का कोई नहीं होता। |
| `KeyLastWrittenUTC` | Key का last write। |
| `DataRaw` | Value के bytes (hex), जब वे setting के type के न हों। |
| `TransactionLogStatus`, `LogsApplied` | Registry recovery का संदर्भ। |
| `Problems` | ग़लत type की values और ऐसी keys जो पढ़ी नहीं जा सकतीं। |

## जाँच की रणनीति { #investigation-strategy }

1. हर host पर `--weakened-only` के साथ चलाएँ, और hosts की एक-दूसरे से तुलना करें: ऐसी setting
   जो केवल एक host पर है, अलग से दिखती है।
2. हर Defender exclusion को host पर मिले tools और Defender के logs से जाँचें
   (Event 5007 configuration बदलाव दर्ज करता है)।
3. Key के समय को logons, service installs और process creation से correlate करें।
   `reg.exe`, `Set-MpPreference`, `netsh advfirewall` और `powershell` command lines खोजें (event
   4688, Sysmon 1 और 13, PowerShell 4104)।
4. Credential settings (`WDigest`, `RunAsPPL`, `LocalAccountTokenFilterPolicy`,
   `DisableRestrictedAdmin`) के लिए, उसके बाद credential dumping और pass-the-hash खोजें।
5. Default को बहाल करने के लिए हटाई गई values के लिए `parse-registry read --recover-deleted` का, और
   पुराने control sets के लिए `--all-control-sets` का उपयोग करें।

## अधिक जानकारी { #further-details }

- [Microsoft: Microsoft Defender Antivirus के लिए exclusions configure और validate करें](https://learn.microsoft.com/en-us/defender-endpoint/configure-exclusions-microsoft-defender-antivirus)
- [Microsoft: User Account Control settings और configuration](https://learn.microsoft.com/en-us/windows/security/application-security/application-control/user-account-control/settings-and-configuration)
- [Microsoft: Added LSA protection configure करें](https://learn.microsoft.com/en-us/windows-server/security/credentials-protection-and-management/configuring-additional-lsa-protection)
- [Microsoft: about_Logging_Windows](https://learn.microsoft.com/en-us/powershell/module/microsoft.powershell.core/about/about_logging_windows)
- [MITRE ATT&CK T1562.001: Impair Defenses: Disable or Modify Tools](https://attack.mitre.org/techniques/T1562/001/)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
